use super::{
    allocate_temporal_plain_date_object, allocate_temporal_plain_date_time_object,
    allocate_temporal_zoned_date_time_object, current_temporal_instant_prototype,
    current_temporal_plain_date_prototype, current_temporal_plain_date_time_prototype,
    current_temporal_plain_time_prototype, current_temporal_zoned_date_time_prototype, instant,
    plain_time, string_value, temporal_now_instant_and_civil, temporal_plain_date_from_parts,
    temporal_plain_date_time_from_parts, temporal_time_zone_id_from_optional_value,
    temporal_zoned_date_time_from_parts, BuiltinFunctionId, BuiltinInvocation,
    PublicBuiltinDispatchContext, TemporalCurrentInstantRequest, TemporalDefaultTimeZoneRequest,
    TemporalPlainTimeObjectData, Value,
};

pub(super) fn dispatch_temporal_now_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_now_instant_builtin() {
        return temporal_now_instant_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_now_time_zone_id_builtin() {
        return temporal_now_time_zone_id_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_now_plain_date_iso_builtin() {
        return temporal_now_plain_date_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_now_plain_time_iso_builtin() {
        return temporal_now_plain_time_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_now_plain_date_time_iso_builtin() {
        return temporal_now_plain_date_time_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_now_zoned_date_time_iso_builtin() {
        return temporal_now_zoned_date_time_iso_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn temporal_now_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let instant = cx.temporal_current_instant(&TemporalCurrentInstantRequest {})?;
    let prototype = current_temporal_instant_prototype(cx)?;
    instant::allocate_temporal_instant_object(cx, prototype, instant.epoch_nanoseconds)
}

fn temporal_now_time_zone_id_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
    Ok(string_value(cx, &zone.time_zone_id))
}

fn temporal_now_plain_date_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (_, civil) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
    let date_time = civil.date_time;
    let data = temporal_plain_date_from_parts(
        cx,
        i64::from(date_time.year),
        i64::from(date_time.month),
        i64::from(date_time.day),
    )?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

fn temporal_now_plain_time_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (_, civil) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
    let date_time = civil.date_time;
    let data = TemporalPlainTimeObjectData::new(
        date_time.hour,
        date_time.minute,
        date_time.second,
        date_time.millisecond,
        date_time.microsecond,
        date_time.nanosecond,
    );
    let prototype = current_temporal_plain_time_prototype(cx)?;
    plain_time::allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_now_plain_date_time_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (_, civil) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
    let date_time = civil.date_time;
    let data = temporal_plain_date_time_from_parts(
        cx,
        i64::from(date_time.year),
        i64::from(date_time.month),
        i64::from(date_time.day),
        i64::from(date_time.hour),
        i64::from(date_time.minute),
        i64::from(date_time.second),
        i64::from(date_time.millisecond),
        i64::from(date_time.microsecond),
        i64::from(date_time.nanosecond),
    )?;
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

fn temporal_now_zoned_date_time_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (epoch_nanoseconds, _) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
    let data = temporal_zoned_date_time_from_parts(cx, epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}
