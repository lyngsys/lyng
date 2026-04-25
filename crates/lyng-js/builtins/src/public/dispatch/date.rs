use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_date_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_date_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_date_format_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_date_getter_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_date_setter_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_date_conversion_builtin(context, entry, invocation)
}

fn dispatch_date_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_builtin() {
        return super::date_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_now_builtin() {
        return super::date_now_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_parse_builtin() {
        return super::date_parse_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_utc_builtin() {
        return super::date_utc_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_date_format_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_to_string_builtin() {
        return super::date_to_string_builtin(context, invocation, super::DateStringKind::Full)
            .map(Some);
    }
    if entry == super::js3_date_to_date_string_builtin() {
        return super::date_to_string_builtin(context, invocation, super::DateStringKind::Date)
            .map(Some);
    }
    if entry == super::js3_date_to_time_string_builtin() {
        return super::date_to_string_builtin(context, invocation, super::DateStringKind::Time)
            .map(Some);
    }
    if entry == super::js3_date_to_locale_string_builtin() {
        return super::date_to_string_builtin(context, invocation, super::DateStringKind::Full)
            .map(Some);
    }
    if entry == super::js3_date_to_locale_date_string_builtin() {
        return super::date_to_string_builtin(context, invocation, super::DateStringKind::Date)
            .map(Some);
    }
    if entry == super::js3_date_to_locale_time_string_builtin() {
        return super::date_to_string_builtin(context, invocation, super::DateStringKind::Time)
            .map(Some);
    }
    Ok(None)
}

fn dispatch_date_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_value_of_builtin() || entry == super::js3_date_get_time_builtin() {
        return super::date_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_get_full_year_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::FullYear,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_full_year_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::FullYear,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_month_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Month,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_month_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Month,
            true,
        )
        .map(Some);
    }
    dispatch_date_getter_part_two(context, entry, invocation)
}

fn dispatch_date_getter_part_two<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_get_date_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Date,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_date_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Date,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_day_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Day,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_day_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Day,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_hours_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Hours,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_hours_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Hours,
            true,
        )
        .map(Some);
    }
    dispatch_date_getter_part_three(context, entry, invocation)
}

fn dispatch_date_getter_part_three<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_get_minutes_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Minutes,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_minutes_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Minutes,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_seconds_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Seconds,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_seconds_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Seconds,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_milliseconds_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Milliseconds,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_utc_milliseconds_builtin() {
        return super::date_get_component_builtin(
            context,
            invocation,
            super::DateComponent::Milliseconds,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_get_timezone_offset_builtin() {
        return super::date_get_timezone_offset_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_date_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_set_time_builtin() {
        return super::date_set_time_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_set_milliseconds_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Milliseconds,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_utc_milliseconds_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Milliseconds,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_seconds_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Seconds,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_utc_seconds_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Seconds,
            true,
        )
        .map(Some);
    }
    dispatch_date_setter_part_two(context, entry, invocation)
}

fn dispatch_date_setter_part_two<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_set_minutes_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Minutes,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_utc_minutes_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Minutes,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_hours_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Hours,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_utc_hours_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Hours,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_date_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Date,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_utc_date_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Date,
            true,
        )
        .map(Some);
    }
    dispatch_date_setter_part_three(context, entry, invocation)
}

fn dispatch_date_setter_part_three<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_set_month_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Month,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_utc_month_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::Month,
            true,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_full_year_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::FullYear,
            false,
        )
        .map(Some);
    }
    if entry == super::js3_date_set_utc_full_year_builtin() {
        return super::date_set_component_builtin(
            context,
            invocation,
            super::DateSetKind::FullYear,
            true,
        )
        .map(Some);
    }
    Ok(None)
}

fn dispatch_date_conversion_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_date_to_utc_string_builtin() {
        return super::date_to_utc_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_to_iso_string_builtin() {
        return super::date_to_iso_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_to_json_builtin() {
        return super::date_to_json_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_to_temporal_instant_builtin() {
        return super::date_to_temporal_instant_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_date_to_primitive_builtin() {
        return super::date_to_primitive_builtin(context, invocation).map(Some);
    }
    Ok(None)
}
