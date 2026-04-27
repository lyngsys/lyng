use lyng_js_common::AtomId;
use lyng_js_types::{BuiltinFunctionId, ObjectRef, Value, WellKnownSymbolId};

/// Typed descriptor-install target used by the shared JS3 bootstrap tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinInstallTarget {
    GlobalObject,
    Intrinsic(BuiltinIntrinsic),
    Object(ObjectRef),
}

/// Typed intrinsic names used by builtin bootstrap tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinIntrinsic {
    Object,
    ObjectPrototype,
    Function,
    FunctionPrototype,
    AsyncFunction,
    AsyncFunctionPrototype,
    AsyncGeneratorFunction,
    AsyncGeneratorFunctionPrototype,
    AsyncGeneratorPrototype,
    GeneratorFunction,
    GeneratorFunctionPrototype,
    GeneratorPrototype,
    Array,
    ArrayPrototype,
    Map,
    MapPrototype,
    MapIteratorPrototype,
    Set,
    SetPrototype,
    SetIteratorPrototype,
    WeakMap,
    WeakMapPrototype,
    WeakSet,
    WeakSetPrototype,
    WeakRef,
    WeakRefPrototype,
    FinalizationRegistry,
    FinalizationRegistryPrototype,
    ArrayBuffer,
    ArrayBufferPrototype,
    SharedArrayBuffer,
    SharedArrayBufferPrototype,
    DataView,
    DataViewPrototype,
    Atomics,
    TypedArray,
    TypedArrayPrototype,
    Int8Array,
    Int8ArrayPrototype,
    Int16Array,
    Int16ArrayPrototype,
    Int32Array,
    Int32ArrayPrototype,
    Float32Array,
    Float32ArrayPrototype,
    Float64Array,
    Float64ArrayPrototype,
    BigInt64Array,
    BigInt64ArrayPrototype,
    BigUint64Array,
    BigUint64ArrayPrototype,
    Uint32Array,
    Uint32ArrayPrototype,
    Uint16Array,
    Uint16ArrayPrototype,
    Uint8ClampedArray,
    Uint8ClampedArrayPrototype,
    Uint8Array,
    Uint8ArrayPrototype,
    Iterator,
    IteratorPrototype,
    AsyncIteratorPrototype,
    AsyncFromSyncIteratorPrototype,
    ArrayIteratorPrototype,
    String,
    StringPrototype,
    StringIteratorPrototype,
    RegExp,
    RegExpPrototype,
    Date,
    DatePrototype,
    Number,
    NumberPrototype,
    Math,
    BigInt,
    BigIntPrototype,
    Boolean,
    BooleanPrototype,
    Symbol,
    SymbolPrototype,
    Json,
    Reflect,
    Proxy,
    Error,
    ErrorPrototype,
    EvalError,
    EvalErrorPrototype,
    RangeError,
    RangeErrorPrototype,
    ReferenceError,
    ReferenceErrorPrototype,
    SyntaxError,
    SyntaxErrorPrototype,
    TypeError,
    TypeErrorPrototype,
    UriError,
    UriErrorPrototype,
    AggregateError,
    AggregateErrorPrototype,
    SuppressedError,
    SuppressedErrorPrototype,
    Promise,
    PromisePrototype,
    DisposableStack,
    DisposableStackPrototype,
    AsyncDisposableStack,
    AsyncDisposableStackPrototype,
    ThrowTypeError,
}

/// Typed property-key surface used by builtin descriptor tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinPropertyKeySpec {
    Index(u32),
    Atom(AtomId),
    WellKnownSymbol(WellKnownSymbolId),
}

impl BuiltinPropertyKeySpec {
    #[inline]
    pub const fn from_atom(atom: AtomId) -> Self {
        Self::Atom(atom)
    }

    #[inline]
    pub const fn from_well_known_symbol(symbol: WellKnownSymbolId) -> Self {
        Self::WellKnownSymbol(symbol)
    }
}

/// Descriptor value payload used by builtin bootstrap tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinPropertyValueSpec {
    Data(Value),
    BuiltinFunction(BuiltinFunctionId),
    Accessor {
        get: Option<BuiltinFunctionId>,
        set: Option<BuiltinFunctionId>,
    },
}

/// Boolean descriptor attributes used by static builtin tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BuiltinAttributes {
    writable: bool,
    enumerable: bool,
    configurable: bool,
}

impl BuiltinAttributes {
    #[inline]
    pub const fn new(writable: bool, enumerable: bool, configurable: bool) -> Self {
        Self {
            writable,
            enumerable,
            configurable,
        }
    }

    #[inline]
    pub const fn writable(self) -> bool {
        self.writable
    }

    #[inline]
    pub const fn enumerable(self) -> bool {
        self.enumerable
    }

    #[inline]
    pub const fn configurable(self) -> bool {
        self.configurable
    }
}

/// One typed builtin property descriptor row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BuiltinPropertyDescriptor {
    key: BuiltinPropertyKeySpec,
    value: BuiltinPropertyValueSpec,
    attributes: BuiltinAttributes,
}

impl BuiltinPropertyDescriptor {
    #[inline]
    pub const fn new(
        key: BuiltinPropertyKeySpec,
        value: BuiltinPropertyValueSpec,
        attributes: BuiltinAttributes,
    ) -> Self {
        Self {
            key,
            value,
            attributes,
        }
    }

    #[inline]
    pub const fn key(self) -> BuiltinPropertyKeySpec {
        self.key
    }

    #[inline]
    pub const fn value(self) -> BuiltinPropertyValueSpec {
        self.value
    }

    #[inline]
    pub const fn attributes(self) -> BuiltinAttributes {
        self.attributes
    }
}

/// Static descriptor-table grouping for one target object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinDescriptorTable<'a> {
    target: BuiltinInstallTarget,
    descriptors: &'a [BuiltinPropertyDescriptor],
}

impl<'a> BuiltinDescriptorTable<'a> {
    #[inline]
    pub const fn new(
        target: BuiltinInstallTarget,
        descriptors: &'a [BuiltinPropertyDescriptor],
    ) -> Self {
        Self {
            target,
            descriptors,
        }
    }

    #[inline]
    pub const fn target(self) -> BuiltinInstallTarget {
        self.target
    }

    #[inline]
    pub const fn descriptors(self) -> &'a [BuiltinPropertyDescriptor] {
        self.descriptors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_descriptor_table_retains_target_and_rows() {
        static DESCRIPTORS: &[BuiltinPropertyDescriptor] = &[BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
            BuiltinPropertyValueSpec::Data(Value::undefined()),
            BuiltinAttributes::new(false, false, true),
        )];

        let table = BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ObjectPrototype),
            DESCRIPTORS,
        );

        assert_eq!(
            table.target(),
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ObjectPrototype)
        );
        assert_eq!(table.descriptors().len(), 1);
        assert_eq!(
            table.descriptors()[0].key(),
            BuiltinPropertyKeySpec::WellKnownSymbol(WellKnownSymbolId::ToStringTag)
        );
    }
}
