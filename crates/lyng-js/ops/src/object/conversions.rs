use super::{ordinary_get_own_property, ordinary_get_prototype_of};
use crate::{
    convert::{bigint_view_to_string, number_to_string},
    read,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::PrimitiveWrapperKind;
use lyng_js_types::{
    AbruptCompletion, Completion, ObjectRef, PropertyKey, Value, WellKnownSymbolId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToPrimitiveHint {
    Default,
    String,
    Number,
}

impl ToPrimitiveHint {
    #[inline]
    pub const fn hint_text(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::String => "string",
            Self::Number => "number",
        }
    }

    #[inline]
    pub const fn method_names(self) -> [lyng_js_common::AtomId; 2] {
        match self {
            Self::Default | Self::Number => {
                [WellKnownAtom::valueOf.id(), WellKnownAtom::toString.id()]
            }
            Self::String => [WellKnownAtom::toString.id(), WellKnownAtom::valueOf.id()],
        }
    }
}

pub trait ToPrimitiveContext {
    type Error;

    fn agent(&mut self) -> &mut Agent;

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error;

    fn type_error(&mut self) -> Self::Error;

    fn get_property_value(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error>;

    fn callable_object(&mut self, value: Value) -> Option<ObjectRef> {
        let object = value.as_object_ref()?;
        self.agent().objects().is_callable(object).then_some(object)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error>;

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error>;

    fn default_to_primitive_result(
        &mut self,
        _object: ObjectRef,
        _method_name: lyng_js_common::AtomId,
        _method_object: ObjectRef,
    ) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }
}

/// ECMAScript `ToPrimitive` over the shared wrapper and property substrate.
///
/// # Errors
/// Returns the caller-provided error type when property lookup or method calls
/// fail, or when the conversion cannot produce a primitive.
pub fn to_primitive<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    value: Value,
    hint: ToPrimitiveHint,
) -> Result<Value, Cx::Error> {
    let Some(object) = value.as_object_ref() else {
        return Ok(value);
    };

    let exotic = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::ToPrimitive)
    };
    if let Some(symbol) = exotic {
        if let Some(method) = get_method(cx, object, PropertyKey::from_symbol(symbol))? {
            let hint_value = {
                let agent = cx.agent();
                Value::from_string_ref(agent.alloc_runtime_string(
                    hint.hint_text(),
                    None,
                    AllocationLifetime::Default,
                ))
            };
            let result =
                cx.call_to_completion(method, Value::from_object_ref(object), &[hint_value])?;
            if !result.is_object() {
                return Ok(result);
            }
            return Err(cx.type_error());
        }
    }

    ordinary_to_primitive(cx, object, hint)
}

/// Object-aware ECMAScript `ToNumber`.
///
/// # Errors
/// Returns the caller-provided error type when `ToPrimitive` or the underlying
/// numeric conversion fails.
pub fn to_number<Cx: ToPrimitiveContext>(cx: &mut Cx, value: Value) -> Result<Value, Cx::Error> {
    let primitive = to_primitive(cx, value, ToPrimitiveHint::Number)?;
    let number = {
        let agent = cx.agent();
        read::to_number(agent.heap().view(), primitive)
    };
    map_completion(cx, number)
}

/// Object-aware ECMAScript `ToNumeric`.
///
/// # Errors
/// Returns the caller-provided error type when `ToPrimitive` or the underlying
/// numeric conversion fails.
pub fn to_numeric<Cx: ToPrimitiveContext>(cx: &mut Cx, value: Value) -> Result<Value, Cx::Error> {
    let primitive = to_primitive(cx, value, ToPrimitiveHint::Number)?;
    let numeric = {
        let agent = cx.agent();
        read::to_numeric(agent.heap().view(), primitive)
    };
    map_completion(cx, numeric)
}

fn map_completion<Cx: ToPrimitiveContext, T>(
    cx: &mut Cx,
    completion: Completion<T>,
) -> Result<T, Cx::Error> {
    completion.map_err(|completion| cx.abrupt(completion))
}

fn get_method<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<Option<ObjectRef>, Cx::Error> {
    let method = cx.get_property_value(object, key)?;
    if method.is_undefined() || method.is_null() {
        return Ok(None);
    }
    cx.require_callable_object(method).map(Some)
}

/// ECMAScript `OrdinaryToPrimitive` for callers that have already selected the
/// preferred hint and must bypass exotic `@@toPrimitive` dispatch.
///
/// # Errors
/// Returns the caller-provided error type when property lookup, method call, or
/// primitive extraction fails.
pub fn ordinary_to_primitive<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    hint: ToPrimitiveHint,
) -> Result<Value, Cx::Error> {
    for method_name in hint.method_names() {
        let key = PropertyKey::from_atom(method_name);
        let method = cx.get_property_value(object, key)?;
        let Some(method) = cx.callable_object(method) else {
            continue;
        };
        if let Some(result) = default_ordinary_to_primitive_result(cx, object, method_name, method)?
        {
            return Ok(result);
        }
        if let Some(result) = cx.default_to_primitive_result(object, method_name, method)? {
            return Ok(result);
        }
        let result = cx.call_to_completion(method, Value::from_object_ref(object), &[])?;
        if !result.is_object() {
            return Ok(result);
        }
    }
    Err(cx.type_error())
}

fn default_ordinary_to_primitive_result<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    method_name: lyng_js_common::AtomId,
    method_object: ObjectRef,
) -> Result<Option<Value>, Cx::Error> {
    if method_name != WellKnownAtom::toString.id() && method_name != WellKnownAtom::valueOf.id() {
        return Ok(None);
    }

    let wrapper_kind = {
        let agent = cx.agent();
        agent.objects().primitive_wrapper_kind(object)
    };
    let Some(wrapper_kind) = wrapper_kind else {
        return Ok(None);
    };
    if !is_default_object_prototype_method(cx, object, method_name, method_object)? {
        return Ok(None);
    }

    let wrapper_value = {
        let agent = cx.agent();
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), object)
    }
    .ok_or_else(|| cx.type_error())?;

    if method_name == WellKnownAtom::valueOf.id() {
        return match wrapper_kind {
            PrimitiveWrapperKind::String
            | PrimitiveWrapperKind::Number
            | PrimitiveWrapperKind::BigInt => Ok(Some(wrapper_value)),
            PrimitiveWrapperKind::Boolean | PrimitiveWrapperKind::Symbol => Ok(None),
        };
    }

    match wrapper_kind {
        PrimitiveWrapperKind::String => Ok(Some(wrapper_value)),
        PrimitiveWrapperKind::Number => {
            let text = number_to_string(
                wrapper_value
                    .as_f64()
                    .expect("wrapper number payload should expose a numeric value"),
            );
            let value = {
                let agent = cx.agent();
                Value::from_string_ref(agent.alloc_runtime_string(
                    &text,
                    None,
                    AllocationLifetime::Default,
                ))
            };
            Ok(Some(value))
        }
        PrimitiveWrapperKind::BigInt => {
            let text = {
                let agent = cx.agent();
                let bigint = wrapper_value
                    .as_bigint_ref()
                    .expect("wrapper bigint payload should expose a bigint value");
                let view = agent.heap().view().bigint_view(bigint);
                let Some(view) = view else {
                    return Err(cx.type_error());
                };
                bigint_view_to_string(view)
            };
            let value = {
                let agent = cx.agent();
                Value::from_string_ref(agent.alloc_runtime_string(
                    &text,
                    None,
                    AllocationLifetime::Default,
                ))
            };
            Ok(Some(value))
        }
        PrimitiveWrapperKind::Boolean | PrimitiveWrapperKind::Symbol => Ok(None),
    }
}

fn is_default_object_prototype_method<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    method_name: lyng_js_common::AtomId,
    method_object: ObjectRef,
) -> Result<bool, Cx::Error> {
    let key = PropertyKey::from_atom(method_name);
    let prototype = {
        let agent = cx.agent();
        ordinary_get_prototype_of(agent, object)
    };
    let Some(wrapper_prototype) = map_completion(cx, prototype)? else {
        return Ok(false);
    };

    let wrapper_descriptor = {
        let agent = cx.agent();
        ordinary_get_own_property(agent, wrapper_prototype, key)
    };
    if map_completion(cx, wrapper_descriptor)?.is_some() {
        return Ok(false);
    }

    let object_prototype = {
        let agent = cx.agent();
        ordinary_get_prototype_of(agent, wrapper_prototype)
    };
    let Some(object_prototype) = map_completion(cx, object_prototype)? else {
        return Ok(false);
    };

    let descriptor = {
        let agent = cx.agent();
        ordinary_get_own_property(agent, object_prototype, key)
    };
    let Some(descriptor) = map_completion(cx, descriptor)? else {
        return Ok(false);
    };

    Ok(descriptor.value() == Some(Value::from_object_ref(method_object)))
}
