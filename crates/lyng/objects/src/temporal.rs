use lyng_js_common::AtomId;

/// Tagged Temporal family used by ordinary-object cold metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TemporalObjectKind {
    Instant,
    Duration,
    PlainDate,
    PlainDateTime,
    PlainTime,
    PlainYearMonth,
    PlainMonthDay,
    ZonedDateTime,
}

/// One `Temporal.Instant` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalInstantObjectData {
    epoch_nanoseconds: i128,
}

impl TemporalInstantObjectData {
    #[inline]
    pub const fn new(epoch_nanoseconds: i128) -> Self {
        Self { epoch_nanoseconds }
    }

    #[inline]
    pub const fn epoch_nanoseconds(self) -> i128 {
        self.epoch_nanoseconds
    }
}

/// One `Temporal.Duration` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalDurationObjectData {
    years: i128,
    months: i128,
    weeks: i128,
    days: i128,
    hours: i128,
    minutes: i128,
    seconds: i128,
    milliseconds: i128,
    microseconds: i128,
    nanoseconds: i128,
}

impl TemporalDurationObjectData {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        years: impl Into<i128>,
        months: impl Into<i128>,
        weeks: impl Into<i128>,
        days: impl Into<i128>,
        hours: impl Into<i128>,
        minutes: impl Into<i128>,
        seconds: impl Into<i128>,
        milliseconds: impl Into<i128>,
        microseconds: impl Into<i128>,
        nanoseconds: impl Into<i128>,
    ) -> Self {
        Self {
            years: years.into(),
            months: months.into(),
            weeks: weeks.into(),
            days: days.into(),
            hours: hours.into(),
            minutes: minutes.into(),
            seconds: seconds.into(),
            milliseconds: milliseconds.into(),
            microseconds: microseconds.into(),
            nanoseconds: nanoseconds.into(),
        }
    }

    #[inline]
    pub const fn years(self) -> i128 {
        self.years
    }

    #[inline]
    pub const fn months(self) -> i128 {
        self.months
    }

    #[inline]
    pub const fn weeks(self) -> i128 {
        self.weeks
    }

    #[inline]
    pub const fn days(self) -> i128 {
        self.days
    }

    #[inline]
    pub const fn hours(self) -> i128 {
        self.hours
    }

    #[inline]
    pub const fn minutes(self) -> i128 {
        self.minutes
    }

    #[inline]
    pub const fn seconds(self) -> i128 {
        self.seconds
    }

    #[inline]
    pub const fn milliseconds(self) -> i128 {
        self.milliseconds
    }

    #[inline]
    pub const fn microseconds(self) -> i128 {
        self.microseconds
    }

    #[inline]
    pub const fn nanoseconds(self) -> i128 {
        self.nanoseconds
    }
}

/// One `Temporal.PlainDate` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalPlainDateObjectData {
    year: i32,
    month: u8,
    day: u8,
    calendar: AtomId,
}

impl TemporalPlainDateObjectData {
    #[inline]
    pub const fn new(year: i32, month: u8, day: u8, calendar: AtomId) -> Self {
        Self {
            year,
            month,
            day,
            calendar,
        }
    }

    #[inline]
    pub const fn year(self) -> i32 {
        self.year
    }

    #[inline]
    pub const fn month(self) -> u8 {
        self.month
    }

    #[inline]
    pub const fn day(self) -> u8 {
        self.day
    }

    #[inline]
    pub const fn calendar(self) -> AtomId {
        self.calendar
    }
}

/// One `Temporal.PlainDateTime` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalPlainDateTimeObjectData {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
    microsecond: u16,
    nanosecond: u16,
    calendar: AtomId,
}

impl TemporalPlainDateTimeObjectData {
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
        calendar: AtomId,
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
            calendar,
        }
    }

    #[inline]
    pub const fn year(self) -> i32 {
        self.year
    }

    #[inline]
    pub const fn month(self) -> u8 {
        self.month
    }

    #[inline]
    pub const fn day(self) -> u8 {
        self.day
    }

    #[inline]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    #[inline]
    pub const fn minute(self) -> u8 {
        self.minute
    }

    #[inline]
    pub const fn second(self) -> u8 {
        self.second
    }

    #[inline]
    pub const fn millisecond(self) -> u16 {
        self.millisecond
    }

    #[inline]
    pub const fn microsecond(self) -> u16 {
        self.microsecond
    }

    #[inline]
    pub const fn nanosecond(self) -> u16 {
        self.nanosecond
    }

    #[inline]
    pub const fn calendar(self) -> AtomId {
        self.calendar
    }
}

/// One `Temporal.PlainTime` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalPlainTimeObjectData {
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
    microsecond: u16,
    nanosecond: u16,
}

impl TemporalPlainTimeObjectData {
    #[inline]
    pub const fn new(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
    ) -> Self {
        Self {
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
        }
    }

    #[inline]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    #[inline]
    pub const fn minute(self) -> u8 {
        self.minute
    }

    #[inline]
    pub const fn second(self) -> u8 {
        self.second
    }

    #[inline]
    pub const fn millisecond(self) -> u16 {
        self.millisecond
    }

    #[inline]
    pub const fn microsecond(self) -> u16 {
        self.microsecond
    }

    #[inline]
    pub const fn nanosecond(self) -> u16 {
        self.nanosecond
    }
}

/// One `Temporal.PlainYearMonth` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalPlainYearMonthObjectData {
    year: i32,
    month: u8,
    reference_day: u8,
    calendar: AtomId,
}

impl TemporalPlainYearMonthObjectData {
    #[inline]
    pub const fn new(year: i32, month: u8, reference_day: u8, calendar: AtomId) -> Self {
        Self {
            year,
            month,
            reference_day,
            calendar,
        }
    }

    #[inline]
    pub const fn year(self) -> i32 {
        self.year
    }

    #[inline]
    pub const fn month(self) -> u8 {
        self.month
    }

    #[inline]
    pub const fn reference_day(self) -> u8 {
        self.reference_day
    }

    #[inline]
    pub const fn calendar(self) -> AtomId {
        self.calendar
    }
}

/// One `Temporal.PlainMonthDay` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalPlainMonthDayObjectData {
    month: u8,
    day: u8,
    reference_year: i32,
    calendar: AtomId,
}

impl TemporalPlainMonthDayObjectData {
    #[inline]
    pub const fn new(month: u8, day: u8, reference_year: i32, calendar: AtomId) -> Self {
        Self {
            month,
            day,
            reference_year,
            calendar,
        }
    }

    #[inline]
    pub const fn month(self) -> u8 {
        self.month
    }

    #[inline]
    pub const fn day(self) -> u8 {
        self.day
    }

    #[inline]
    pub const fn reference_year(self) -> i32 {
        self.reference_year
    }

    #[inline]
    pub const fn calendar(self) -> AtomId {
        self.calendar
    }
}

/// One `Temporal.ZonedDateTime` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemporalZonedDateTimeObjectData {
    epoch_nanoseconds: i128,
    time_zone: AtomId,
    calendar: AtomId,
}

impl TemporalZonedDateTimeObjectData {
    #[inline]
    pub const fn new(epoch_nanoseconds: i128, time_zone: AtomId, calendar: AtomId) -> Self {
        Self {
            epoch_nanoseconds,
            time_zone,
            calendar,
        }
    }

    #[inline]
    pub const fn epoch_nanoseconds(self) -> i128 {
        self.epoch_nanoseconds
    }

    #[inline]
    pub const fn time_zone(self) -> AtomId {
        self.time_zone
    }

    #[inline]
    pub const fn calendar(self) -> AtomId {
        self.calendar
    }
}

/// One typed Temporal payload stored out-of-line from ordinary object properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TemporalObjectData {
    Instant(TemporalInstantObjectData),
    Duration(TemporalDurationObjectData),
    PlainDate(TemporalPlainDateObjectData),
    PlainDateTime(TemporalPlainDateTimeObjectData),
    PlainTime(TemporalPlainTimeObjectData),
    PlainYearMonth(TemporalPlainYearMonthObjectData),
    PlainMonthDay(TemporalPlainMonthDayObjectData),
    ZonedDateTime(TemporalZonedDateTimeObjectData),
}

impl TemporalObjectData {
    #[inline]
    pub const fn kind(self) -> TemporalObjectKind {
        match self {
            Self::Instant(_) => TemporalObjectKind::Instant,
            Self::Duration(_) => TemporalObjectKind::Duration,
            Self::PlainDate(_) => TemporalObjectKind::PlainDate,
            Self::PlainDateTime(_) => TemporalObjectKind::PlainDateTime,
            Self::PlainTime(_) => TemporalObjectKind::PlainTime,
            Self::PlainYearMonth(_) => TemporalObjectKind::PlainYearMonth,
            Self::PlainMonthDay(_) => TemporalObjectKind::PlainMonthDay,
            Self::ZonedDateTime(_) => TemporalObjectKind::ZonedDateTime,
        }
    }
}
