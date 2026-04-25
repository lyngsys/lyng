use super::descriptors::{
    builtin_function_atom_property, builtin_function_symbol_property, data_atom_property,
    readonly_builtin_attributes, writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, DateFamilyBuiltins, DateFamilyPrototypes, FamilyInstallContext,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{
    BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic, BuiltinPropertyDescriptor,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_date_builtin, js3_date_get_date_builtin, js3_date_get_day_builtin,
    js3_date_get_full_year_builtin, js3_date_get_hours_builtin, js3_date_get_milliseconds_builtin,
    js3_date_get_minutes_builtin, js3_date_get_month_builtin, js3_date_get_seconds_builtin,
    js3_date_get_time_builtin, js3_date_get_timezone_offset_builtin, js3_date_get_utc_date_builtin,
    js3_date_get_utc_day_builtin, js3_date_get_utc_full_year_builtin,
    js3_date_get_utc_hours_builtin, js3_date_get_utc_milliseconds_builtin,
    js3_date_get_utc_minutes_builtin, js3_date_get_utc_month_builtin,
    js3_date_get_utc_seconds_builtin, js3_date_now_builtin, js3_date_parse_builtin,
    js3_date_set_date_builtin, js3_date_set_full_year_builtin, js3_date_set_hours_builtin,
    js3_date_set_milliseconds_builtin, js3_date_set_minutes_builtin, js3_date_set_month_builtin,
    js3_date_set_seconds_builtin, js3_date_set_time_builtin, js3_date_set_utc_date_builtin,
    js3_date_set_utc_full_year_builtin, js3_date_set_utc_hours_builtin,
    js3_date_set_utc_milliseconds_builtin, js3_date_set_utc_minutes_builtin,
    js3_date_set_utc_month_builtin, js3_date_set_utc_seconds_builtin,
    js3_date_to_date_string_builtin, js3_date_to_iso_string_builtin, js3_date_to_json_builtin,
    js3_date_to_locale_date_string_builtin, js3_date_to_locale_string_builtin,
    js3_date_to_locale_time_string_builtin, js3_date_to_primitive_builtin,
    js3_date_to_string_builtin, js3_date_to_temporal_instant_builtin,
    js3_date_to_time_string_builtin, js3_date_to_utc_string_builtin, js3_date_utc_builtin,
    js3_date_value_of_builtin, BuiltinFunctionId, ObjectRef, RealmRef, Value, WellKnownSymbolId,
};

pub(in crate::public) fn install_date_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: DateFamilyPrototypes,
) -> DateFamilyBuiltins {
    let statics = install_date_static_methods(agent, cx);
    let formatters = install_date_format_methods(agent, cx);
    let getters = install_date_getter_methods(agent, cx);
    let setters = install_date_setter_methods(agent, cx);
    let conversions = install_date_conversion_methods(agent, cx);

    DateFamilyBuiltins {
        date: install_public_builtin_function(
            agent,
            cx,
            js3_date_builtin(),
            Some(prototypes.date_prototype),
        ),
        date_prototype: prototypes.date_prototype,
        date_now: statics.now,
        date_parse: statics.parse,
        date_utc: statics.utc,
        date_to_string: formatters.string,
        date_to_date_string: formatters.date_string,
        date_to_time_string: formatters.time_string,
        date_to_locale_string: formatters.locale_string,
        date_to_locale_date_string: formatters.locale_date_string,
        date_to_locale_time_string: formatters.locale_time_string,
        date_value_of: conversions.value_of,
        date_get_time: getters.time,
        date_get_full_year: getters.full_year,
        date_get_utc_full_year: getters.utc_full_year,
        date_get_month: getters.month,
        date_get_utc_month: getters.utc_month,
        date_get_date: getters.date,
        date_get_utc_date: getters.utc_date,
        date_get_day: getters.day,
        date_get_utc_day: getters.utc_day,
        date_get_hours: getters.hours,
        date_get_utc_hours: getters.utc_hours,
        date_get_minutes: getters.minutes,
        date_get_utc_minutes: getters.utc_minutes,
        date_get_seconds: getters.seconds,
        date_get_utc_seconds: getters.utc_seconds,
        date_get_milliseconds: getters.milliseconds,
        date_get_utc_milliseconds: getters.utc_milliseconds,
        date_get_timezone_offset: getters.timezone_offset,
        date_set_time: setters.time,
        date_set_milliseconds: setters.milliseconds,
        date_set_utc_milliseconds: setters.utc_milliseconds,
        date_set_seconds: setters.seconds,
        date_set_utc_seconds: setters.utc_seconds,
        date_set_minutes: setters.minutes,
        date_set_utc_minutes: setters.utc_minutes,
        date_set_hours: setters.hours,
        date_set_utc_hours: setters.utc_hours,
        date_set_date: setters.date,
        date_set_utc_date: setters.utc_date,
        date_set_month: setters.month,
        date_set_utc_month: setters.utc_month,
        date_set_full_year: setters.full_year,
        date_set_utc_full_year: setters.utc_full_year,
        date_to_utc_string: conversions.to_utc_string,
        date_to_iso_string: conversions.to_iso_string,
        date_to_json: conversions.to_json,
        date_to_primitive: conversions.to_primitive,
        date_to_temporal_instant: conversions.to_temporal_instant,
    }
}

pub(in crate::public) fn date_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    if let Some(object) = date_static_builtin_object(builtins, entry) {
        return Some(object);
    }
    if let Some(object) = date_format_builtin_object(builtins, entry) {
        return Some(object);
    }
    if let Some(object) = date_getter_builtin_object(builtins, entry) {
        return Some(object);
    }
    if let Some(object) = date_setter_builtin_object(builtins, entry) {
        return Some(object);
    }
    date_conversion_builtin_object(builtins, entry)
}

fn date_static_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_date_builtin(), builtins.date),
        (js3_date_now_builtin(), builtins.date_now),
        (js3_date_parse_builtin(), builtins.date_parse),
        (js3_date_utc_builtin(), builtins.date_utc),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn date_format_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_date_to_string_builtin(), builtins.date_to_string),
        (
            js3_date_to_date_string_builtin(),
            builtins.date_to_date_string,
        ),
        (
            js3_date_to_time_string_builtin(),
            builtins.date_to_time_string,
        ),
        (
            js3_date_to_locale_string_builtin(),
            builtins.date_to_locale_string,
        ),
        (
            js3_date_to_locale_date_string_builtin(),
            builtins.date_to_locale_date_string,
        ),
        (
            js3_date_to_locale_time_string_builtin(),
            builtins.date_to_locale_time_string,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn date_getter_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_date_get_time_builtin(), builtins.date_get_time),
        (
            js3_date_get_full_year_builtin(),
            builtins.date_get_full_year,
        ),
        (
            js3_date_get_utc_full_year_builtin(),
            builtins.date_get_utc_full_year,
        ),
        (js3_date_get_month_builtin(), builtins.date_get_month),
        (
            js3_date_get_utc_month_builtin(),
            builtins.date_get_utc_month,
        ),
        (js3_date_get_date_builtin(), builtins.date_get_date),
        (js3_date_get_utc_date_builtin(), builtins.date_get_utc_date),
        (js3_date_get_day_builtin(), builtins.date_get_day),
        (js3_date_get_utc_day_builtin(), builtins.date_get_utc_day),
        (js3_date_get_hours_builtin(), builtins.date_get_hours),
        (
            js3_date_get_utc_hours_builtin(),
            builtins.date_get_utc_hours,
        ),
        (js3_date_get_minutes_builtin(), builtins.date_get_minutes),
        (
            js3_date_get_utc_minutes_builtin(),
            builtins.date_get_utc_minutes,
        ),
        (js3_date_get_seconds_builtin(), builtins.date_get_seconds),
        (
            js3_date_get_utc_seconds_builtin(),
            builtins.date_get_utc_seconds,
        ),
        (
            js3_date_get_milliseconds_builtin(),
            builtins.date_get_milliseconds,
        ),
        (
            js3_date_get_utc_milliseconds_builtin(),
            builtins.date_get_utc_milliseconds,
        ),
        (
            js3_date_get_timezone_offset_builtin(),
            builtins.date_get_timezone_offset,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn date_setter_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_date_set_time_builtin(), builtins.date_set_time),
        (
            js3_date_set_milliseconds_builtin(),
            builtins.date_set_milliseconds,
        ),
        (
            js3_date_set_utc_milliseconds_builtin(),
            builtins.date_set_utc_milliseconds,
        ),
        (js3_date_set_seconds_builtin(), builtins.date_set_seconds),
        (
            js3_date_set_utc_seconds_builtin(),
            builtins.date_set_utc_seconds,
        ),
        (js3_date_set_minutes_builtin(), builtins.date_set_minutes),
        (
            js3_date_set_utc_minutes_builtin(),
            builtins.date_set_utc_minutes,
        ),
        (js3_date_set_hours_builtin(), builtins.date_set_hours),
        (
            js3_date_set_utc_hours_builtin(),
            builtins.date_set_utc_hours,
        ),
        (js3_date_set_date_builtin(), builtins.date_set_date),
        (js3_date_set_utc_date_builtin(), builtins.date_set_utc_date),
        (js3_date_set_month_builtin(), builtins.date_set_month),
        (
            js3_date_set_utc_month_builtin(),
            builtins.date_set_utc_month,
        ),
        (
            js3_date_set_full_year_builtin(),
            builtins.date_set_full_year,
        ),
        (
            js3_date_set_utc_full_year_builtin(),
            builtins.date_set_utc_full_year,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn date_conversion_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_date_value_of_builtin(), builtins.date_value_of),
        (
            js3_date_to_utc_string_builtin(),
            builtins.date_to_utc_string,
        ),
        (
            js3_date_to_iso_string_builtin(),
            builtins.date_to_iso_string,
        ),
        (js3_date_to_json_builtin(), builtins.date_to_json),
        (js3_date_to_primitive_builtin(), builtins.date_to_primitive),
        (
            js3_date_to_temporal_instant_builtin(),
            builtins.date_to_temporal_instant,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

#[derive(Clone, Copy, Debug)]
struct DateStaticMethods {
    now: ObjectRef,
    parse: ObjectRef,
    utc: ObjectRef,
}

fn install_date_static_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateStaticMethods {
    DateStaticMethods {
        now: install_public_builtin_function(agent, cx, js3_date_now_builtin(), None),
        parse: install_public_builtin_function(agent, cx, js3_date_parse_builtin(), None),
        utc: install_public_builtin_function(agent, cx, js3_date_utc_builtin(), None),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateFormatMethods {
    string: ObjectRef,
    date_string: ObjectRef,
    time_string: ObjectRef,
    locale_string: ObjectRef,
    locale_date_string: ObjectRef,
    locale_time_string: ObjectRef,
}

fn install_date_format_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateFormatMethods {
    DateFormatMethods {
        string: install_public_builtin_function(agent, cx, js3_date_to_string_builtin(), None),
        date_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_date_string_builtin(),
            None,
        ),
        time_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_time_string_builtin(),
            None,
        ),
        locale_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_locale_string_builtin(),
            None,
        ),
        locale_date_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_locale_date_string_builtin(),
            None,
        ),
        locale_time_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_locale_time_string_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateGetterMethods {
    time: ObjectRef,
    full_year: ObjectRef,
    utc_full_year: ObjectRef,
    month: ObjectRef,
    utc_month: ObjectRef,
    date: ObjectRef,
    utc_date: ObjectRef,
    day: ObjectRef,
    utc_day: ObjectRef,
    hours: ObjectRef,
    utc_hours: ObjectRef,
    minutes: ObjectRef,
    utc_minutes: ObjectRef,
    seconds: ObjectRef,
    utc_seconds: ObjectRef,
    milliseconds: ObjectRef,
    utc_milliseconds: ObjectRef,
    timezone_offset: ObjectRef,
}

fn install_date_getter_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateGetterMethods {
    DateGetterMethods {
        time: install_public_builtin_function(agent, cx, js3_date_get_time_builtin(), None),
        full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_full_year_builtin(),
            None,
        ),
        utc_full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_full_year_builtin(),
            None,
        ),
        month: install_public_builtin_function(agent, cx, js3_date_get_month_builtin(), None),
        utc_month: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_month_builtin(),
            None,
        ),
        date: install_public_builtin_function(agent, cx, js3_date_get_date_builtin(), None),
        utc_date: install_public_builtin_function(agent, cx, js3_date_get_utc_date_builtin(), None),
        day: install_public_builtin_function(agent, cx, js3_date_get_day_builtin(), None),
        utc_day: install_public_builtin_function(agent, cx, js3_date_get_utc_day_builtin(), None),
        hours: install_public_builtin_function(agent, cx, js3_date_get_hours_builtin(), None),
        utc_hours: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_hours_builtin(),
            None,
        ),
        minutes: install_public_builtin_function(agent, cx, js3_date_get_minutes_builtin(), None),
        utc_minutes: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_minutes_builtin(),
            None,
        ),
        seconds: install_public_builtin_function(agent, cx, js3_date_get_seconds_builtin(), None),
        utc_seconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_seconds_builtin(),
            None,
        ),
        milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_milliseconds_builtin(),
            None,
        ),
        utc_milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_milliseconds_builtin(),
            None,
        ),
        timezone_offset: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_timezone_offset_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateSetterMethods {
    time: ObjectRef,
    milliseconds: ObjectRef,
    utc_milliseconds: ObjectRef,
    seconds: ObjectRef,
    utc_seconds: ObjectRef,
    minutes: ObjectRef,
    utc_minutes: ObjectRef,
    hours: ObjectRef,
    utc_hours: ObjectRef,
    date: ObjectRef,
    utc_date: ObjectRef,
    month: ObjectRef,
    utc_month: ObjectRef,
    full_year: ObjectRef,
    utc_full_year: ObjectRef,
}

fn install_date_setter_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateSetterMethods {
    DateSetterMethods {
        time: install_public_builtin_function(agent, cx, js3_date_set_time_builtin(), None),
        milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_milliseconds_builtin(),
            None,
        ),
        utc_milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_milliseconds_builtin(),
            None,
        ),
        seconds: install_public_builtin_function(agent, cx, js3_date_set_seconds_builtin(), None),
        utc_seconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_seconds_builtin(),
            None,
        ),
        minutes: install_public_builtin_function(agent, cx, js3_date_set_minutes_builtin(), None),
        utc_minutes: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_minutes_builtin(),
            None,
        ),
        hours: install_public_builtin_function(agent, cx, js3_date_set_hours_builtin(), None),
        utc_hours: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_hours_builtin(),
            None,
        ),
        date: install_public_builtin_function(agent, cx, js3_date_set_date_builtin(), None),
        utc_date: install_public_builtin_function(agent, cx, js3_date_set_utc_date_builtin(), None),
        month: install_public_builtin_function(agent, cx, js3_date_set_month_builtin(), None),
        utc_month: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_month_builtin(),
            None,
        ),
        full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_full_year_builtin(),
            None,
        ),
        utc_full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_full_year_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateConversionMethods {
    value_of: ObjectRef,
    to_utc_string: ObjectRef,
    to_iso_string: ObjectRef,
    to_json: ObjectRef,
    to_primitive: ObjectRef,
    to_temporal_instant: ObjectRef,
}

fn install_date_conversion_methods(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> DateConversionMethods {
    DateConversionMethods {
        value_of: install_public_builtin_function(agent, cx, js3_date_value_of_builtin(), None),
        to_utc_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_utc_string_builtin(),
            None,
        ),
        to_iso_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_iso_string_builtin(),
            None,
        ),
        to_json: install_public_builtin_function(agent, cx, js3_date_to_json_builtin(), None),
        to_primitive: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_primitive_builtin(),
            None,
        ),
        to_temporal_instant: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_temporal_instant_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn install_date_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = DateDescriptorAtoms::new(agent);
    install_date_constructor_descriptors(agent, cache, realm, atoms)?;
    install_date_prototype_descriptors(agent, cache, realm, builtins.date, atoms)
}

fn install_date_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: DateDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = date_static_method_specs(atoms)
        .map(|(atom, entry)| builtin_function_atom_property(atom, entry));
    install_intrinsic_descriptor_table(agent, cache, realm, BuiltinIntrinsic::Date, &descriptors)
}

fn install_date_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    date: ObjectRef,
    atoms: DateDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let constructor = [data_atom_property(
        WellKnownAtom::constructor.id(),
        Value::from_object_ref(date),
        writable_builtin_attributes(),
    )];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::DatePrototype,
        &constructor,
    )?;
    install_date_prototype_method_group(agent, cache, realm, date_format_method_specs(atoms))?;
    install_date_prototype_method_group(agent, cache, realm, date_value_method_specs())?;
    install_date_prototype_method_group(agent, cache, realm, date_getter_method_specs(atoms))?;
    install_date_prototype_method_group(agent, cache, realm, date_setter_method_specs(atoms))?;
    install_date_prototype_method_group(agent, cache, realm, date_conversion_method_specs(atoms))?;

    let to_primitive = [builtin_function_symbol_property(
        WellKnownSymbolId::ToPrimitive,
        js3_date_to_primitive_builtin(),
        readonly_builtin_attributes(),
    )];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::DatePrototype,
        &to_primitive,
    )
}

fn install_date_prototype_method_group<const N: usize>(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    specs: [(AtomId, BuiltinFunctionId); N],
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = specs.map(|(atom, entry)| builtin_function_atom_property(atom, entry));
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::DatePrototype,
        &descriptors,
    )
}

fn install_intrinsic_descriptor_table(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    target: BuiltinIntrinsic,
    descriptors: &[BuiltinPropertyDescriptor],
) -> Result<(), BuiltinBootstrapError> {
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(target),
            descriptors,
        )],
    )
}

#[derive(Clone, Copy)]
struct DateDescriptorAtoms {
    now: AtomId,
    parse: AtomId,
    utc: AtomId,
    to_date_string: AtomId,
    to_time_string: AtomId,
    to_locale_string: AtomId,
    to_locale_date_string: AtomId,
    to_locale_time_string: AtomId,
    get_time: AtomId,
    get_full_year: AtomId,
    get_utc_full_year: AtomId,
    get_month: AtomId,
    get_utc_month: AtomId,
    get_date: AtomId,
    get_utc_date: AtomId,
    get_day: AtomId,
    get_utc_day: AtomId,
    get_hours: AtomId,
    get_utc_hours: AtomId,
    get_minutes: AtomId,
    get_utc_minutes: AtomId,
    get_seconds: AtomId,
    get_utc_seconds: AtomId,
    get_milliseconds: AtomId,
    get_utc_milliseconds: AtomId,
    get_timezone_offset: AtomId,
    set_time: AtomId,
    set_milliseconds: AtomId,
    set_utc_milliseconds: AtomId,
    set_seconds: AtomId,
    set_utc_seconds: AtomId,
    set_minutes: AtomId,
    set_utc_minutes: AtomId,
    set_hours: AtomId,
    set_utc_hours: AtomId,
    set_date: AtomId,
    set_utc_date: AtomId,
    set_month: AtomId,
    set_utc_month: AtomId,
    set_full_year: AtomId,
    set_utc_full_year: AtomId,
    to_utc_string: AtomId,
    to_iso_string: AtomId,
    to_json: AtomId,
    to_temporal_instant: AtomId,
}

impl DateDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        Self {
            now: agent.atoms_mut().intern_collectible("now"),
            parse: agent.atoms_mut().intern_collectible("parse"),
            utc: agent.atoms_mut().intern_collectible("UTC"),
            to_date_string: agent.atoms_mut().intern_collectible("toDateString"),
            to_time_string: agent.atoms_mut().intern_collectible("toTimeString"),
            to_locale_string: agent.atoms_mut().intern_collectible("toLocaleString"),
            to_locale_date_string: agent.atoms_mut().intern_collectible("toLocaleDateString"),
            to_locale_time_string: agent.atoms_mut().intern_collectible("toLocaleTimeString"),
            get_time: agent.atoms_mut().intern_collectible("getTime"),
            get_full_year: agent.atoms_mut().intern_collectible("getFullYear"),
            get_utc_full_year: agent.atoms_mut().intern_collectible("getUTCFullYear"),
            get_month: agent.atoms_mut().intern_collectible("getMonth"),
            get_utc_month: agent.atoms_mut().intern_collectible("getUTCMonth"),
            get_date: agent.atoms_mut().intern_collectible("getDate"),
            get_utc_date: agent.atoms_mut().intern_collectible("getUTCDate"),
            get_day: agent.atoms_mut().intern_collectible("getDay"),
            get_utc_day: agent.atoms_mut().intern_collectible("getUTCDay"),
            get_hours: agent.atoms_mut().intern_collectible("getHours"),
            get_utc_hours: agent.atoms_mut().intern_collectible("getUTCHours"),
            get_minutes: agent.atoms_mut().intern_collectible("getMinutes"),
            get_utc_minutes: agent.atoms_mut().intern_collectible("getUTCMinutes"),
            get_seconds: agent.atoms_mut().intern_collectible("getSeconds"),
            get_utc_seconds: agent.atoms_mut().intern_collectible("getUTCSeconds"),
            get_milliseconds: agent.atoms_mut().intern_collectible("getMilliseconds"),
            get_utc_milliseconds: agent.atoms_mut().intern_collectible("getUTCMilliseconds"),
            get_timezone_offset: agent.atoms_mut().intern_collectible("getTimezoneOffset"),
            set_time: agent.atoms_mut().intern_collectible("setTime"),
            set_milliseconds: agent.atoms_mut().intern_collectible("setMilliseconds"),
            set_utc_milliseconds: agent.atoms_mut().intern_collectible("setUTCMilliseconds"),
            set_seconds: agent.atoms_mut().intern_collectible("setSeconds"),
            set_utc_seconds: agent.atoms_mut().intern_collectible("setUTCSeconds"),
            set_minutes: agent.atoms_mut().intern_collectible("setMinutes"),
            set_utc_minutes: agent.atoms_mut().intern_collectible("setUTCMinutes"),
            set_hours: agent.atoms_mut().intern_collectible("setHours"),
            set_utc_hours: agent.atoms_mut().intern_collectible("setUTCHours"),
            set_date: agent.atoms_mut().intern_collectible("setDate"),
            set_utc_date: agent.atoms_mut().intern_collectible("setUTCDate"),
            set_month: agent.atoms_mut().intern_collectible("setMonth"),
            set_utc_month: agent.atoms_mut().intern_collectible("setUTCMonth"),
            set_full_year: agent.atoms_mut().intern_collectible("setFullYear"),
            set_utc_full_year: agent.atoms_mut().intern_collectible("setUTCFullYear"),
            to_utc_string: agent.atoms_mut().intern_collectible("toUTCString"),
            to_iso_string: agent.atoms_mut().intern_collectible("toISOString"),
            to_json: agent.atoms_mut().intern_collectible("toJSON"),
            to_temporal_instant: agent.atoms_mut().intern_collectible("toTemporalInstant"),
        }
    }
}

fn date_static_method_specs(atoms: DateDescriptorAtoms) -> [(AtomId, BuiltinFunctionId); 3] {
    [
        (atoms.now, js3_date_now_builtin()),
        (atoms.parse, js3_date_parse_builtin()),
        (atoms.utc, js3_date_utc_builtin()),
    ]
}

fn date_format_method_specs(atoms: DateDescriptorAtoms) -> [(AtomId, BuiltinFunctionId); 6] {
    [
        (WellKnownAtom::toString.id(), js3_date_to_string_builtin()),
        (atoms.to_date_string, js3_date_to_date_string_builtin()),
        (atoms.to_time_string, js3_date_to_time_string_builtin()),
        (atoms.to_locale_string, js3_date_to_locale_string_builtin()),
        (
            atoms.to_locale_date_string,
            js3_date_to_locale_date_string_builtin(),
        ),
        (
            atoms.to_locale_time_string,
            js3_date_to_locale_time_string_builtin(),
        ),
    ]
}

fn date_value_method_specs() -> [(AtomId, BuiltinFunctionId); 1] {
    [(WellKnownAtom::valueOf.id(), js3_date_value_of_builtin())]
}

fn date_getter_method_specs(atoms: DateDescriptorAtoms) -> [(AtomId, BuiltinFunctionId); 18] {
    [
        (atoms.get_time, js3_date_get_time_builtin()),
        (atoms.get_full_year, js3_date_get_full_year_builtin()),
        (
            atoms.get_utc_full_year,
            js3_date_get_utc_full_year_builtin(),
        ),
        (atoms.get_month, js3_date_get_month_builtin()),
        (atoms.get_utc_month, js3_date_get_utc_month_builtin()),
        (atoms.get_date, js3_date_get_date_builtin()),
        (atoms.get_utc_date, js3_date_get_utc_date_builtin()),
        (atoms.get_day, js3_date_get_day_builtin()),
        (atoms.get_utc_day, js3_date_get_utc_day_builtin()),
        (atoms.get_hours, js3_date_get_hours_builtin()),
        (atoms.get_utc_hours, js3_date_get_utc_hours_builtin()),
        (atoms.get_minutes, js3_date_get_minutes_builtin()),
        (atoms.get_utc_minutes, js3_date_get_utc_minutes_builtin()),
        (atoms.get_seconds, js3_date_get_seconds_builtin()),
        (atoms.get_utc_seconds, js3_date_get_utc_seconds_builtin()),
        (atoms.get_milliseconds, js3_date_get_milliseconds_builtin()),
        (
            atoms.get_utc_milliseconds,
            js3_date_get_utc_milliseconds_builtin(),
        ),
        (
            atoms.get_timezone_offset,
            js3_date_get_timezone_offset_builtin(),
        ),
    ]
}

fn date_setter_method_specs(atoms: DateDescriptorAtoms) -> [(AtomId, BuiltinFunctionId); 15] {
    [
        (atoms.set_time, js3_date_set_time_builtin()),
        (atoms.set_milliseconds, js3_date_set_milliseconds_builtin()),
        (
            atoms.set_utc_milliseconds,
            js3_date_set_utc_milliseconds_builtin(),
        ),
        (atoms.set_seconds, js3_date_set_seconds_builtin()),
        (atoms.set_utc_seconds, js3_date_set_utc_seconds_builtin()),
        (atoms.set_minutes, js3_date_set_minutes_builtin()),
        (atoms.set_utc_minutes, js3_date_set_utc_minutes_builtin()),
        (atoms.set_hours, js3_date_set_hours_builtin()),
        (atoms.set_utc_hours, js3_date_set_utc_hours_builtin()),
        (atoms.set_date, js3_date_set_date_builtin()),
        (atoms.set_utc_date, js3_date_set_utc_date_builtin()),
        (atoms.set_month, js3_date_set_month_builtin()),
        (atoms.set_utc_month, js3_date_set_utc_month_builtin()),
        (atoms.set_full_year, js3_date_set_full_year_builtin()),
        (
            atoms.set_utc_full_year,
            js3_date_set_utc_full_year_builtin(),
        ),
    ]
}

fn date_conversion_method_specs(atoms: DateDescriptorAtoms) -> [(AtomId, BuiltinFunctionId); 4] {
    [
        (atoms.to_utc_string, js3_date_to_utc_string_builtin()),
        (atoms.to_iso_string, js3_date_to_iso_string_builtin()),
        (atoms.to_json, js3_date_to_json_builtin()),
        (
            atoms.to_temporal_instant,
            js3_date_to_temporal_instant_builtin(),
        ),
    ]
}
