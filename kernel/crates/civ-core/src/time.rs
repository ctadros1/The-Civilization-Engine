//! Simulation time and the calendar.
//!
//! One tick is one in-game minute (plan §4.4). [`SimTime`] counts minutes since the calendar's
//! origin: year 1, month 1, day 1, 00:00. The calendar has 365-day years with the familiar month
//! lengths and no leap years. Months are numbered rather than named, because names belong to the
//! cultures that emerge, not to the engine.
//!
//! [`Season`] here is a calendar convention (meteorological quarters of a northern-temperate
//! world), not climate. Climate arrives in M3 and decides what a season actually feels like.

use std::fmt;

/// Minutes in an hour.
pub const MINUTES_PER_HOUR: i64 = 60;
/// Hours in a day.
pub const HOURS_PER_DAY: i64 = 24;
/// Minutes in a day.
pub const MINUTES_PER_DAY: i64 = MINUTES_PER_HOUR * HOURS_PER_DAY;
/// Days in a week.
pub const DAYS_PER_WEEK: i64 = 7;
/// Days in a year (no leap years).
pub const DAYS_PER_YEAR: i64 = 365;
/// Minutes in a year.
pub const MINUTES_PER_YEAR: i64 = MINUTES_PER_DAY * DAYS_PER_YEAR;
/// Length of each month, in days.
pub const MONTH_LENGTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
/// Day of the year (0-based) on which each month starts, plus the year length at the end.
pub const MONTH_STARTS: [i64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

/// A moment in simulation time, in whole minutes since year 1, month 1, day 1, 00:00.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimTime(i64);

impl SimTime {
    /// The calendar origin.
    pub const ZERO: SimTime = SimTime(0);

    /// A time from minutes since the origin.
    pub const fn from_minutes(minutes: i64) -> Self {
        SimTime(minutes)
    }

    /// Minutes since the origin.
    pub const fn minutes(self) -> i64 {
        self.0
    }

    /// The time at the given calendar position. `None` if month, day, hour or minute is out of
    /// range for the calendar.
    pub const fn from_date(year: i64, month: u8, day: u8, hour: u8, minute: u8) -> Option<Self> {
        if month < 1 || month > 12 || hour > 23 || minute > 59 {
            return None;
        }
        let m = (month - 1) as usize;
        if day < 1 || day as i64 > MONTH_LENGTHS[m] {
            return None;
        }
        let days = (year - 1) * DAYS_PER_YEAR + MONTH_STARTS[m] + (day as i64 - 1);
        Some(SimTime(
            days * MINUTES_PER_DAY + hour as i64 * MINUTES_PER_HOUR + minute as i64,
        ))
    }

    /// This time plus `minutes`, saturating at the representable range.
    pub const fn plus_minutes(self, minutes: i64) -> Self {
        SimTime(self.0.saturating_add(minutes))
    }

    /// Whole days since the origin (floor, so times before the origin count backwards).
    pub const fn day_index(self) -> i64 {
        self.0.div_euclid(MINUTES_PER_DAY)
    }

    /// Minutes since the start of the day, 0–1439.
    pub const fn minute_of_day(self) -> i64 {
        self.0.rem_euclid(MINUTES_PER_DAY)
    }

    /// The calendar position of this time.
    pub fn date(self) -> Date {
        let year_index = self.0.div_euclid(MINUTES_PER_YEAR);
        let in_year = self.0.rem_euclid(MINUTES_PER_YEAR);
        let day_of_year = in_year / MINUTES_PER_DAY;
        let minute_of_day = in_year % MINUTES_PER_DAY;
        let month_index = MONTH_STARTS
            .iter()
            .rposition(|&start| start <= day_of_year)
            .unwrap_or(0)
            .min(11);
        Date {
            year: year_index + 1,
            month: month_index as u8 + 1,
            day: (day_of_year - MONTH_STARTS[month_index]) as u8 + 1,
            hour: (minute_of_day / MINUTES_PER_HOUR) as u8,
            minute: (minute_of_day % MINUTES_PER_HOUR) as u8,
            day_of_year: day_of_year as u16 + 1,
        }
    }
}

impl fmt::Display for SimTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.date().fmt(f)
    }
}

/// A calendar position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// Year, starting at 1.
    pub year: i64,
    /// Month, 1–12.
    pub month: u8,
    /// Day of the month, 1–31.
    pub day: u8,
    /// Hour, 0–23.
    pub hour: u8,
    /// Minute, 0–59.
    pub minute: u8,
    /// Day of the year, 1–365.
    pub day_of_year: u16,
}

impl Date {
    /// The calendar season of this date.
    pub fn season(&self) -> Season {
        Season::of_month(self.month)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "year {}, month {}, day {}, {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }
}

/// Calendar quarters (a convention, not climate; see the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Season {
    /// Months 3–5.
    Spring,
    /// Months 6–8.
    Summer,
    /// Months 9–11.
    Autumn,
    /// Months 12, 1 and 2.
    Winter,
}

impl Season {
    /// The season a month falls in.
    pub const fn of_month(month: u8) -> Season {
        match month {
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    /// Lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Autumn => "autumn",
            Season::Winter => "winter",
        }
    }
}

/// The default moment a new world begins: the first morning of spring in year 1.
///
/// A start in winter would be a scenario choice, so the neutral default is spring. This is a
/// world-creation parameter, not a plot point.
pub const DEFAULT_WORLD_START: SimTime = match SimTime::from_date(1, 3, 1, 6, 0) {
    Some(t) => t,
    None => SimTime::ZERO,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_year_one_month_one() {
        let d = SimTime::ZERO.date();
        assert_eq!((d.year, d.month, d.day, d.hour, d.minute), (1, 1, 1, 0, 0));
        assert_eq!(d.day_of_year, 1);
    }

    #[test]
    fn from_date_and_date_are_inverse_over_several_years() {
        for minutes in (0..MINUTES_PER_YEAR * 3).step_by(9_973) {
            let t = SimTime::from_minutes(minutes);
            let d = t.date();
            assert_eq!(
                SimTime::from_date(d.year, d.month, d.day, d.hour, d.minute),
                Some(t)
            );
        }
    }

    #[test]
    fn month_boundaries_follow_month_lengths() {
        let end_of_feb = SimTime::from_date(1, 2, 28, 23, 59).expect("valid");
        let d = end_of_feb.plus_minutes(1).date();
        assert_eq!((d.month, d.day), (3, 1));
        assert_eq!(SimTime::from_date(1, 2, 29, 0, 0), None, "no leap years");
        let new_year = SimTime::from_date(1, 12, 31, 23, 59)
            .expect("valid")
            .plus_minutes(1)
            .date();
        assert_eq!((new_year.year, new_year.month, new_year.day), (2, 1, 1));
    }

    #[test]
    fn times_before_the_origin_have_dates() {
        let d = SimTime::from_minutes(-1).date();
        assert_eq!(
            (d.year, d.month, d.day, d.hour, d.minute),
            (0, 12, 31, 23, 59)
        );
    }

    #[test]
    fn invalid_dates_are_rejected() {
        assert_eq!(SimTime::from_date(1, 0, 1, 0, 0), None);
        assert_eq!(SimTime::from_date(1, 13, 1, 0, 0), None);
        assert_eq!(SimTime::from_date(1, 4, 31, 0, 0), None);
        assert_eq!(SimTime::from_date(1, 1, 1, 24, 0), None);
    }

    #[test]
    fn default_start_is_a_spring_morning() {
        let d = DEFAULT_WORLD_START.date();
        assert_eq!((d.year, d.month, d.day, d.hour), (1, 3, 1, 6));
        assert_eq!(d.season(), Season::Spring);
    }

    #[test]
    fn large_years_do_not_overflow() {
        let far = SimTime::from_date(100_000, 6, 15, 12, 0).expect("valid");
        assert_eq!(far.date().year, 100_000);
    }
}
