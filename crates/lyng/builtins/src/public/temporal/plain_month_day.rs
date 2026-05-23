use super::{
    allocate_builtin_function_object, define_builtin_accessor_property,
    define_builtin_data_property, public_builtin_metadata,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_types::{EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value};

#[derive(Clone, Copy)]
pub(super) struct PlainMonthDayBootstrapContext {
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    object_prototype: ObjectRef,
}

impl PlainMonthDayBootstrapContext {
    pub(super) const fn new(
        realm: RealmRef,
        global_env: EnvironmentRef,
        root_shape: ShapeId,
        function_prototype: ObjectRef,
        object_prototype: ObjectRef,
    ) -> Self {
        Self {
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct PlainMonthDayFunctions {
    constructor: ObjectRef,
    month_code_getter: ObjectRef,
    day_getter: ObjectRef,
    calendar_id_getter: ObjectRef,
    to_string: ObjectRef,
    to_json: ObjectRef,
    to_locale_string: ObjectRef,
    value_of: ObjectRef,
    equals: ObjectRef,
    with: ObjectRef,
    to_plain_date: ObjectRef,
    from: ObjectRef,
}

#[derive(Clone, Copy)]
pub(super) struct PlainMonthDayPrototypeProperties {
    pub(super) month_code_key: PropertyKey,
    pub(super) day_key: PropertyKey,
    pub(super) calendar_id_key: PropertyKey,
    pub(super) to_json_key: PropertyKey,
    pub(super) to_locale_string_key: PropertyKey,
    pub(super) equals_key: PropertyKey,
    pub(super) with_key: PropertyKey,
    pub(super) to_plain_date_key: PropertyKey,
    pub(super) to_string_tag_key: PropertyKey,
    pub(super) prototype_tag: Value,
}

#[allow(
    clippy::too_many_lines,
    reason = "PlainMonthDay builtin allocation follows the ordered Temporal function table"
)]
pub(super) fn allocate_functions(
    agent: &mut Agent,
    context: PlainMonthDayBootstrapContext,
    plain_month_day_prototype: ObjectRef,
) -> Option<PlainMonthDayFunctions> {
    let constructor = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_builtin())?,
        Some(plain_month_day_prototype),
    );
    let month_code_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_month_code_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_plain_month_day_month_code_getter_builtin(),
        )?,
        None,
    );
    let day_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_day_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_day_getter_builtin())?,
        None,
    );
    let calendar_id_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_calendar_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_plain_month_day_calendar_id_getter_builtin(),
        )?,
        None,
    );
    let to_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_to_string_builtin())?,
        None,
    );
    let to_json = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_to_json_builtin())?,
        None,
    );
    let to_locale_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_to_locale_string_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_plain_month_day_to_locale_string_builtin(),
        )?,
        None,
    );
    let value_of = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_value_of_builtin())?,
        None,
    );
    let equals = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_equals_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_equals_builtin())?,
        None,
    );
    let with = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_with_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_with_builtin())?,
        None,
    );
    let to_plain_date = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_to_plain_date_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_to_plain_date_builtin())?,
        None,
    );
    let from = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_month_day_from_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_month_day_from_builtin())?,
        None,
    );

    Some(PlainMonthDayFunctions {
        constructor,
        month_code_getter,
        day_getter,
        calendar_id_getter,
        to_string,
        to_json,
        to_locale_string,
        value_of,
        equals,
        with,
        to_plain_date,
        from,
    })
}

pub(super) fn install_temporal_object_property(
    agent: &mut Agent,
    temporal_object: ObjectRef,
    plain_month_day_key: PropertyKey,
    functions: PlainMonthDayFunctions,
) {
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_month_day_key,
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
}

pub(super) fn install_constructor_properties(
    agent: &mut Agent,
    functions: PlainMonthDayFunctions,
    from_key: PropertyKey,
) {
    define_builtin_data_property(
        agent,
        functions.constructor,
        from_key,
        Value::from_object_ref(functions.from),
        true,
        false,
        true,
    );
}

#[allow(
    clippy::too_many_lines,
    reason = "PlainMonthDay prototype descriptors are kept inline as an ordered Temporal table"
)]
pub(super) fn install_prototype_properties(
    agent: &mut Agent,
    plain_month_day_prototype: ObjectRef,
    functions: PlainMonthDayFunctions,
    properties: PlainMonthDayPrototypeProperties,
) {
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_month_day_prototype,
        properties.month_code_key,
        Some(functions.month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_month_day_prototype,
        properties.day_key,
        Some(functions.day_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_month_day_prototype,
        properties.calendar_id_key,
        Some(functions.calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(functions.to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        properties.to_json_key,
        Value::from_object_ref(functions.to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        properties.to_locale_string_key,
        Value::from_object_ref(functions.to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        properties.equals_key,
        Value::from_object_ref(functions.equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        properties.with_key,
        Value::from_object_ref(functions.with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        properties.to_plain_date_key,
        Value::from_object_ref(functions.to_plain_date),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(functions.value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        properties.to_string_tag_key,
        properties.prototype_tag,
        false,
        false,
        true,
    );
}
