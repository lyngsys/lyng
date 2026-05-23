use crate::error::{HostError, HostResult};
use crate::ids::{
    HostAgentId, HostJobId, HostSharedBufferId, HostThreadId, HostTransferredBufferId,
};
use lyng_js_common::{Severity, SourceId, Span};
use lyng_js_types::{BackingStoreRef, BuiltinFunctionId};
use std::cmp::Ordering;

/// Host-facing diagnostic payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticReportRequest {
    pub severity: Severity,
    pub source: Option<SourceId>,
    pub span: Option<Span>,
    pub message: String,
}

/// Host-facing uncaught-exception report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UncaughtExceptionReport {
    pub source: Option<SourceId>,
    pub realm: Option<lyng_js_types::RealmRef>,
    pub thrown_value: lyng_js_types::Value,
    pub message: String,
}

/// Request to load top-level script text through the embedding host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptSourceRequest {
    pub path: String,
    pub referrer: Option<SourceId>,
    pub is_entry: bool,
}

/// Host-owned typed identity for one canonicalized module location.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModuleKey(Box<str>);

impl ModuleKey {
    #[inline]
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialOrd for ModuleKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ModuleKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

/// One module import attribute carried at the host boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleImportAttribute {
    pub key: String,
    pub value: String,
}

/// Request to load module source text through the embedding host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleSourceRequest {
    pub specifier: String,
    pub referrer: Option<ModuleKey>,
    pub attributes: Vec<ModuleImportAttribute>,
}

/// Host-returned source payload for scripts or modules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedSourceText {
    pub display_name: String,
    pub source_text: String,
}

impl LoadedSourceText {
    #[inline]
    pub fn new(display_name: impl Into<String>, source_text: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
            source_text: source_text.into(),
        }
    }
}

/// Host-returned source payload for one canonicalized module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedModuleSource {
    pub key: ModuleKey,
    pub display_name: String,
    pub source_text: String,
}

impl LoadedModuleSource {
    #[inline]
    pub fn new(
        key: ModuleKey,
        display_name: impl Into<String>,
        source_text: impl Into<String>,
    ) -> Self {
        Self {
            key,
            display_name: display_name.into(),
            source_text: source_text.into(),
        }
    }
}

/// Cold import.meta request sent to the embedding host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportMetaRequest {
    pub module: ModuleKey,
}

/// Host-provided primitive import.meta value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportMetaValue {
    String(String),
    Boolean(bool),
    Smi(i32),
    Null,
}

/// One import.meta property provided by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportMetaProperty {
    pub key: String,
    pub value: ImportMetaValue,
}

/// Host-returned import.meta data for one module.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportMetaProperties {
    pub properties: Vec<ImportMetaProperty>,
}

impl ImportMetaProperties {
    #[inline]
    pub const fn new(properties: Vec<ImportMetaProperty>) -> Self {
        Self { properties }
    }
}

/// Observable job family at the host boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostJobKind {
    Promise,
    Script,
    Module,
    Harness,
    Native(BuiltinFunctionId),
}

/// Observable lifecycle event for a runtime-owned job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostJobPhase {
    Enqueued,
    Started,
    Completed,
    Failed,
}

/// Cold-path job observation sent to the embedding host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobObservation {
    pub agent: Option<HostAgentId>,
    pub job_id: HostJobId,
    pub phase: HostJobPhase,
    pub kind: HostJobKind,
}

/// High-level reason for creating a new agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentSpawnKind {
    Dedicated,
    SharedMemory,
    Harness,
}

/// Request for the host to provision a new engine agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateAgentRequest {
    pub parent_agent: Option<HostAgentId>,
    pub kind: AgentSpawnKind,
    pub debug_name: Option<String>,
}

/// Response after the host provisions a new engine agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateAgentResponse {
    pub agent_id: HostAgentId,
}

/// Host thread-start category for a provisioned agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentThreadStartKind {
    AgentMain,
    Helper,
    Harness,
}

/// Request for the host to start running an existing agent on a thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartAgentThreadRequest {
    pub agent_id: HostAgentId,
    pub kind: AgentThreadStartKind,
    pub debug_name: Option<String>,
}

/// Response after the host accepts or starts a thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartAgentThreadResponse {
    pub thread_id: HostThreadId,
    pub started: bool,
}

/// Request to transfer detachable `ArrayBuffer` ownership across agents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArrayBufferTransferRequest {
    pub source_agent: HostAgentId,
    pub target_agent: HostAgentId,
    pub buffer_id: HostTransferredBufferId,
    pub byte_length: usize,
    pub detach_on_success: bool,
}

/// Response after the host accepts an `ArrayBuffer` transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArrayBufferTransferResponse {
    pub buffer_id: HostTransferredBufferId,
    pub detached: bool,
}

/// Request to share a `SharedArrayBuffer` backing-store handle across agents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedArrayBufferShareRequest {
    pub source_agent: HostAgentId,
    pub target_agent: HostAgentId,
    pub backing_store: BackingStoreRef,
    pub byte_length: usize,
}

/// Response after the host accepts a `SharedArrayBuffer` share request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedArrayBufferShareResponse {
    pub shared_buffer: HostSharedBufferId,
}

/// One shared-memory wait location coordinated by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaitLocation {
    pub backing_store: BackingStoreRef,
    pub byte_offset: u64,
}

impl WaitLocation {
    #[inline]
    pub const fn new(backing_store: BackingStoreRef, byte_offset: u64) -> Self {
        Self {
            backing_store,
            byte_offset,
        }
    }
}

/// Request to park an agent until wakeup, timeout, or interruption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParkAgentRequest {
    pub agent_id: HostAgentId,
    pub thread_id: Option<HostThreadId>,
    pub location: WaitLocation,
    pub timeout_ns: Option<u64>,
    pub allow_async: bool,
}

/// Host-observable result of a park attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParkAgentStatus {
    Parked,
    TimedOut,
    Interrupted,
}

/// Response from a host parking primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParkAgentResult {
    pub status: ParkAgentStatus,
}

/// Request to wake parked agents waiting on a shared-memory location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnparkAgentRequest {
    pub location: WaitLocation,
    pub max_count: u32,
}

/// Response after the host applies a wakeup primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnparkAgentResult {
    pub woken_agents: u32,
    pub remaining_waiters: bool,
}

/// Request to read the host clock as a Temporal instant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TemporalCurrentInstantRequest {}

/// Host-returned epoch-nanosecond instant for Temporal integration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemporalInstant {
    pub epoch_nanoseconds: i128,
}

impl TemporalInstant {
    #[inline]
    pub const fn new(epoch_nanoseconds: i128) -> Self {
        Self { epoch_nanoseconds }
    }
}

/// Request to read the host's default named time zone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TemporalDefaultTimeZoneRequest {}

/// Host-returned default named time zone for Temporal integration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalDefaultTimeZone {
    pub time_zone_id: String,
}

impl TemporalDefaultTimeZone {
    #[inline]
    pub fn new(time_zone_id: impl Into<String>) -> Self {
        Self {
            time_zone_id: time_zone_id.into(),
        }
    }
}

/// Civil date-time fields used by Temporal time-zone integration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemporalCivilDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub microsecond: u16,
    pub nanosecond: u16,
}

impl TemporalCivilDateTime {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
        }
    }
}

/// Host-resolved civil date-time and offset for one instant in one time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalCivilTime {
    pub date_time: TemporalCivilDateTime,
    pub offset_nanoseconds: i64,
}

/// Request to resolve one instant into civil time for one named zone.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemporalInstantToCivilRequest {
    pub time_zone_id: String,
    pub epoch_nanoseconds: i128,
}

/// Temporal disambiguation mode for civil-time-to-instant mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemporalDisambiguation {
    Compatible,
    Earlier,
    Later,
    Reject,
}

/// Request to resolve one civil time into an instant for one named zone.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemporalCivilToInstantRequest {
    pub time_zone_id: String,
    pub date_time: TemporalCivilDateTime,
    pub disambiguation: TemporalDisambiguation,
}

/// Host-resolved instant and effective offset for one civil-time lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalInstantWithOffset {
    pub epoch_nanoseconds: i128,
    pub offset_nanoseconds: i64,
}

const NANOS_PER_MICROSECOND: i128 = 1_000;
const NANOS_PER_MILLISECOND: i128 = 1_000_000;
const NANOS_PER_SECOND: i128 = 1_000_000_000;
const NANOS_PER_MINUTE: i128 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: i128 = 60 * NANOS_PER_MINUTE;
const NANOS_PER_DAY: i128 = 24 * NANOS_PER_HOUR;
const UTC_TIME_ZONE_ID: &str = "UTC";

fn parse_two_digits(bytes: &[u8], index: &mut usize) -> Option<i128> {
    let tens = *bytes.get(*index)?;
    let ones = *bytes.get(*index + 1)?;
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }
    *index += 2;
    Some(i128::from(tens - b'0') * 10 + i128::from(ones - b'0'))
}

fn parse_fixed_offset_time_zone_id(time_zone_id: &str) -> Option<i64> {
    if time_zone_id == UTC_TIME_ZONE_ID {
        return Some(0);
    }
    let bytes = time_zone_id.as_bytes();
    let sign = match bytes.first().copied()? {
        b'+' => 1_i128,
        b'-' => -1_i128,
        _ => return None,
    };

    let mut index = 1;
    let hours = parse_two_digits(bytes, &mut index)?;
    let mut minutes = 0_i128;
    let mut seconds = 0_i128;
    let mut fraction = 0_i128;
    if index < bytes.len() {
        if bytes[index] == b':' {
            index += 1;
        }
        minutes = parse_two_digits(bytes, &mut index)?;
        if index < bytes.len() {
            if bytes[index] != b':' {
                return None;
            }
            index += 1;
            seconds = parse_two_digits(bytes, &mut index)?;
            if index < bytes.len() {
                if bytes[index] != b'.' {
                    return None;
                }
                index += 1;
                let start = index;
                let mut scale = 100_000_000_i128;
                while let Some(byte) = bytes.get(index).copied() {
                    if !byte.is_ascii_digit() || index - start >= 9 {
                        return None;
                    }
                    fraction += i128::from(byte - b'0') * scale;
                    scale /= 10;
                    index += 1;
                }
                if index == start {
                    return None;
                }
            }
        }
    }
    if index != bytes.len() || hours > 23 || minutes > 59 || seconds > 59 {
        return None;
    }
    let total = ((hours * 60 + minutes) * 60 + seconds)
        .checked_mul(NANOS_PER_SECOND)?
        .checked_add(fraction)?
        .checked_mul(sign)?;
    i64::try_from(total).ok()
}

fn require_supported_time_zone(operation: &'static str, time_zone_id: &str) -> HostResult<i64> {
    parse_fixed_offset_time_zone_id(time_zone_id).map_or_else(
        || {
            Err(HostError::unsupported(
                operation,
                format!(
                    "host only provides deterministic `{UTC_TIME_ZONE_ID}` and fixed-offset resolution for `{time_zone_id}`"
                ),
            ))
        },
        Ok,
    )
}

const fn floor_div_rem(value: i128, divisor: i128) -> (i128, i128) {
    let mut quotient = value / divisor;
    let mut remainder = value % divisor;
    if remainder < 0 {
        quotient -= 1;
        remainder += divisor;
    }
    (quotient, remainder)
}

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

const fn days_in_month(year: i32, month: u8) -> Option<u8> {
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return None,
    })
}

fn civil_from_days(days_since_epoch: i128) -> HostResult<(i32, u8, u8)> {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i128::from(month <= 2);

    let year = i32::try_from(year).map_err(|_| {
        HostError::invalid_request(
            "temporal_instant_to_civil_time",
            format!("year `{year}` is outside the supported i32 range"),
        )
    })?;
    let month = u8::try_from(month).map_err(|_| {
        HostError::invalid_request(
            "temporal_instant_to_civil_time",
            format!("month `{month}` is outside the supported u8 range"),
        )
    })?;
    let day = u8::try_from(day).map_err(|_| {
        HostError::invalid_request(
            "temporal_instant_to_civil_time",
            format!("day `{day}` is outside the supported u8 range"),
        )
    })?;
    Ok((year, month, day))
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i128 {
    let year = i128::from(year) - i128::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = i128::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + i128::from(day) - 1;
    let day_of_era = yoe * 365 + yoe / 4 - yoe / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn validate_civil_date_time(
    date_time: TemporalCivilDateTime,
    operation: &'static str,
) -> HostResult<()> {
    let Some(max_day) = days_in_month(date_time.year, date_time.month) else {
        return Err(HostError::invalid_request(
            operation,
            format!("month `{}` is not a valid civil month", date_time.month),
        ));
    };
    if date_time.day == 0 || date_time.day > max_day {
        return Err(HostError::invalid_request(
            operation,
            format!(
                "day `{}` is not valid for {:04}-{:02}",
                date_time.day, date_time.year, date_time.month
            ),
        ));
    }
    if date_time.hour > 23
        || date_time.minute > 59
        || date_time.second > 59
        || date_time.millisecond > 999
        || date_time.microsecond > 999
        || date_time.nanosecond > 999
    {
        return Err(HostError::invalid_request(
            operation,
            "civil time fields are outside the supported UTC range",
        ));
    }
    Ok(())
}

fn utc_instant_to_civil_time(epoch_nanoseconds: i128) -> HostResult<TemporalCivilTime> {
    let (days, remainder) = floor_div_rem(epoch_nanoseconds, NANOS_PER_DAY);
    let (year, month, day) = civil_from_days(days)?;

    let hour = remainder / NANOS_PER_HOUR;
    let remainder = remainder % NANOS_PER_HOUR;
    let minute = remainder / NANOS_PER_MINUTE;
    let remainder = remainder % NANOS_PER_MINUTE;
    let second = remainder / NANOS_PER_SECOND;
    let remainder = remainder % NANOS_PER_SECOND;
    let millisecond = remainder / NANOS_PER_MILLISECOND;
    let remainder = remainder % NANOS_PER_MILLISECOND;
    let microsecond = remainder / NANOS_PER_MICROSECOND;
    let nanosecond = remainder % NANOS_PER_MICROSECOND;

    Ok(TemporalCivilTime {
        date_time: TemporalCivilDateTime::new(
            year,
            month,
            day,
            u8::try_from(hour).expect("hour should stay in u8 range"),
            u8::try_from(minute).expect("minute should stay in u8 range"),
            u8::try_from(second).expect("second should stay in u8 range"),
            u16::try_from(millisecond).expect("millisecond should stay in u16 range"),
            u16::try_from(microsecond).expect("microsecond should stay in u16 range"),
            u16::try_from(nanosecond).expect("nanosecond should stay in u16 range"),
        ),
        offset_nanoseconds: 0,
    })
}

fn utc_civil_time_to_instant(
    date_time: TemporalCivilDateTime,
) -> HostResult<TemporalInstantWithOffset> {
    validate_civil_date_time(date_time, "temporal_civil_time_to_instant")?;
    let days = days_from_civil(date_time.year, date_time.month, date_time.day);
    let epoch_nanoseconds = days * NANOS_PER_DAY
        + i128::from(date_time.hour) * NANOS_PER_HOUR
        + i128::from(date_time.minute) * NANOS_PER_MINUTE
        + i128::from(date_time.second) * NANOS_PER_SECOND
        + i128::from(date_time.millisecond) * NANOS_PER_MILLISECOND
        + i128::from(date_time.microsecond) * NANOS_PER_MICROSECOND
        + i128::from(date_time.nanosecond);
    Ok(TemporalInstantWithOffset {
        epoch_nanoseconds,
        offset_nanoseconds: 0,
    })
}

/// Cold-path host integration boundary.
pub trait HostHooks: Send + Sync {
    /// Reports one diagnostic emitted by the runtime.
    ///
    /// # Errors
    /// Returns an error when the host rejects or fails to record the diagnostic.
    #[inline]
    fn report_diagnostic(&self, _request: &DiagnosticReportRequest) -> HostResult<()> {
        Ok(())
    }

    /// Reports one uncaught exception observed by the runtime.
    ///
    /// # Errors
    /// Returns an error when the host rejects or fails to record the exception.
    #[inline]
    fn report_uncaught_exception(&self, _request: &UncaughtExceptionReport) -> HostResult<()> {
        Ok(())
    }

    /// Loads one script source requested by the runtime.
    ///
    /// # Errors
    /// Returns an error when the host cannot resolve or load the script source.
    #[inline]
    fn load_script_source(&self, request: &ScriptSourceRequest) -> HostResult<LoadedSourceText> {
        Err(HostError::unsupported(
            "load_script_source",
            format!("host does not provide script source `{}`", request.path),
        ))
    }

    /// Loads one module source requested by the runtime.
    ///
    /// # Errors
    /// Returns an error when the host cannot resolve or load the module source.
    #[inline]
    fn load_module_source(&self, request: &ModuleSourceRequest) -> HostResult<LoadedModuleSource> {
        Err(HostError::unsupported(
            "load_module_source",
            format!(
                "host does not provide module source `{}`",
                request.specifier
            ),
        ))
    }

    /// Produces host-owned `import.meta` properties for one already resolved module.
    ///
    /// # Errors
    /// Returns an error when the host rejects the request or cannot provide the metadata.
    #[inline]
    fn resolve_import_meta(
        &self,
        _request: &ImportMetaRequest,
    ) -> HostResult<ImportMetaProperties> {
        Ok(ImportMetaProperties::default())
    }

    /// Observes one job-lifecycle event emitted by the runtime.
    ///
    /// # Errors
    /// Returns an error when the host rejects or fails to record the observation.
    #[inline]
    fn observe_job(&self, _request: &JobObservation) -> HostResult<()> {
        Ok(())
    }

    /// Provisions one additional agent for the runtime.
    ///
    /// # Errors
    /// Returns an error when the host cannot create another agent.
    #[inline]
    fn create_agent(&self, _request: &CreateAgentRequest) -> HostResult<CreateAgentResponse> {
        Err(HostError::unsupported(
            "create_agent",
            "host does not provision additional agents yet",
        ))
    }

    /// Starts one host thread for a runtime-managed agent.
    ///
    /// # Errors
    /// Returns an error when the host cannot start the requested thread.
    #[inline]
    fn start_agent_thread(
        &self,
        _request: &StartAgentThreadRequest,
    ) -> HostResult<StartAgentThreadResponse> {
        Err(HostError::unsupported(
            "start_agent_thread",
            "host does not start agent threads yet",
        ))
    }

    /// Transfers ownership of one `ArrayBuffer` backing store between agents.
    ///
    /// # Errors
    /// Returns an error when the host cannot complete the requested transfer.
    #[inline]
    fn transfer_array_buffer(
        &self,
        _request: &ArrayBufferTransferRequest,
    ) -> HostResult<ArrayBufferTransferResponse> {
        Err(HostError::unsupported(
            "transfer_array_buffer",
            "host does not transfer ArrayBuffer ownership yet",
        ))
    }

    /// Shares one `SharedArrayBuffer` backing store with another agent.
    ///
    /// # Errors
    /// Returns an error when the host cannot complete the requested share.
    #[inline]
    fn share_array_buffer(
        &self,
        _request: &SharedArrayBufferShareRequest,
    ) -> HostResult<SharedArrayBufferShareResponse> {
        Err(HostError::unsupported(
            "share_array_buffer",
            "host does not share SharedArrayBuffer backing stores yet",
        ))
    }

    /// Parks one agent on a host wait location.
    ///
    /// # Errors
    /// Returns an error when the host cannot park the agent.
    #[inline]
    fn park_agent(&self, _request: &ParkAgentRequest) -> HostResult<ParkAgentResult> {
        Err(HostError::unsupported(
            "park_agent",
            "host does not park agents yet",
        ))
    }

    /// Unparks agents waiting on one host wait location.
    ///
    /// # Errors
    /// Returns an error when the host cannot wake the requested waiters.
    #[inline]
    fn unpark_agent(&self, _request: &UnparkAgentRequest) -> HostResult<UnparkAgentResult> {
        Err(HostError::unsupported(
            "unpark_agent",
            "host does not unpark agents yet",
        ))
    }

    /// Reads one host clock instant for Temporal integration.
    ///
    /// # Errors
    /// Returns an error when the host cannot provide a Temporal-compatible instant.
    #[inline]
    fn temporal_current_instant(
        &self,
        _request: &TemporalCurrentInstantRequest,
    ) -> HostResult<TemporalInstant> {
        Ok(TemporalInstant::new(0))
    }

    /// Reads the host's default named time zone for Temporal integration.
    ///
    /// # Errors
    /// Returns an error when the host cannot provide a default time-zone identifier.
    #[inline]
    fn temporal_default_time_zone(
        &self,
        _request: &TemporalDefaultTimeZoneRequest,
    ) -> HostResult<TemporalDefaultTimeZone> {
        Ok(TemporalDefaultTimeZone::new(UTC_TIME_ZONE_ID))
    }

    /// Checks whether the host's default time zone is UTC without requiring the
    /// host to allocate or clone the time-zone identifier on hot paths.
    ///
    /// # Errors
    /// Returns an error when the host cannot inspect its default time-zone identifier.
    #[inline]
    fn temporal_default_time_zone_is_utc(
        &self,
        request: &TemporalDefaultTimeZoneRequest,
    ) -> HostResult<bool> {
        Ok(self.temporal_default_time_zone(request)?.time_zone_id == UTC_TIME_ZONE_ID)
    }

    /// Resolves one instant to civil time for one named time zone.
    ///
    /// # Errors
    /// Returns an error when the host cannot resolve the requested zone.
    #[inline]
    fn temporal_instant_to_civil_time(
        &self,
        request: &TemporalInstantToCivilRequest,
    ) -> HostResult<TemporalCivilTime> {
        let offset_nanoseconds =
            require_supported_time_zone("temporal_instant_to_civil_time", &request.time_zone_id)?;
        let mut civil = utc_instant_to_civil_time(
            request
                .epoch_nanoseconds
                .checked_add(i128::from(offset_nanoseconds))
                .ok_or_else(|| {
                    HostError::invalid_request(
                        "temporal_instant_to_civil_time",
                        "epoch plus fixed offset is outside supported range",
                    )
                })?,
        )?;
        civil.offset_nanoseconds = offset_nanoseconds;
        Ok(civil)
    }

    /// Resolves one civil time to an instant for one named time zone.
    ///
    /// # Errors
    /// Returns an error when the host cannot resolve the requested zone.
    #[inline]
    fn temporal_civil_time_to_instant(
        &self,
        request: &TemporalCivilToInstantRequest,
    ) -> HostResult<TemporalInstantWithOffset> {
        let offset_nanoseconds =
            require_supported_time_zone("temporal_civil_time_to_instant", &request.time_zone_id)?;
        let instant = utc_civil_time_to_instant(request.date_time)?;
        Ok(TemporalInstantWithOffset {
            epoch_nanoseconds: instant
                .epoch_nanoseconds
                .checked_sub(i128::from(offset_nanoseconds))
                .ok_or_else(|| {
                    HostError::invalid_request(
                        "temporal_civil_time_to_instant",
                        "civil time minus fixed offset is outside supported range",
                    )
                })?,
            offset_nanoseconds,
        })
    }
}
