//! Closed-form need and body arithmetic. Needs are lazy (ADR-0003): each is stored as a value and
//! the time it was last brought up to date, and advanced exactly here when an activity step
//! starts or ends. Nothing is decremented minute by minute.

use std::f64::consts::PI;

use civ_core::time::{DAYS_PER_YEAR, MINUTES_PER_DAY};

use crate::params::{EnergyParams, SleepParams, interpolate};

/// Sex, for biology only (research 04-12: no sex multipliers on work).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sex {
    /// Female.
    Female,
    /// Male.
    Male,
}

/// Sunrise and sunset, minutes after midnight (local solar time), for a day of the year (0–364)
/// at a latitude. Polar day and night clamp to 0 and 1440.
pub fn daylight(latitude_deg: f64, day_of_year: i64) -> (i64, i64) {
    let doy = day_of_year.rem_euclid(DAYS_PER_YEAR) as f64;
    // Solar declination (Cooper's approximation), radians.
    let decl = (23.44f64).to_radians() * (2.0 * PI * (284.0 + doy + 1.0) / 365.0).sin();
    let lat = latitude_deg.to_radians();
    let cos_h = (-lat.tan() * decl.tan()).clamp(-1.0, 1.0);
    let half_day_h = cos_h.acos().to_degrees() / 15.0;
    let rise = ((12.0 - half_day_h) * 60.0).round() as i64;
    let set = ((12.0 + half_day_h) * 60.0).round() as i64;
    (
        rise.clamp(0, MINUTES_PER_DAY),
        set.clamp(0, MINUTES_PER_DAY),
    )
}

/// Circadian weight of sleep pressure at a minute of the day: the authored day factor in daylight
/// and through the evening, rising to 1 over the hour before the authored bedtime, and falling
/// back over the hour around sunrise.
pub fn circadian(sleep: &SleepParams, minute_of_day: i64, (rise, set): (i64, i64)) -> f64 {
    let m = minute_of_day as f64;
    let (rise, set) = (rise as f64, set as f64);
    let day = sleep.day_factor;
    if (m - rise).abs() <= 30.0 {
        let t = (m - rise + 30.0) / 60.0;
        return 1.0 + (day - 1.0) * t;
    }
    if m > rise && m < set {
        return day;
    }
    // Minutes since sunset, through midnight.
    let since = if m >= set {
        m - set
    } else {
        m + MINUTES_PER_DAY as f64 - set
    };
    let bedtime = sleep.bedtime_after_sunset_hours * 60.0;
    let t = ((since - (bedtime - 60.0)) / 60.0).clamp(0.0, 1.0);
    day + (1.0 - day) * t
}

/// Body mass at `age` years, kg.
pub fn mass_kg(energy: &EnergyParams, sex: Sex, age: f64) -> f64 {
    let table: Vec<(f64, f64)> = energy
        .mass_by_age
        .iter()
        .map(|&(a, m, f)| (a, if sex == Sex::Male { m } else { f }))
        .collect();
    interpolate(&table, age)
}

/// Basal metabolic rate, kcal per day.
pub fn bmr_kcal_day(energy: &EnergyParams, sex: Sex, age: f64, mass: f64) -> f64 {
    let bands = if sex == Sex::Male {
        &energy.bmr_male
    } else {
        &energy.bmr_female
    };
    let mut band = 0usize;
    for (i, &start) in energy.bmr_band_starts.iter().enumerate() {
        if age >= start {
            band = i;
        }
    }
    let (slope, intercept) = bands.get(band).copied().unwrap_or((0.0, 0.0));
    (slope * mass + intercept).max(50.0)
}

/// Sleep pressure after `minutes` awake (`asleep == false`) or asleep, starting from `p0`.
pub fn sleep_pressure(sleep: &SleepParams, p0: f64, minutes: f64, asleep: bool) -> f64 {
    let hours = minutes.max(0.0) / 60.0;
    if asleep {
        p0 * (-hours / sleep.tau_asleep_h).exp()
    } else {
        1.0 - (1.0 - p0) * (-hours / sleep.tau_awake_h).exp()
    }
}

/// Minutes of sleep needed to bring pressure from `p0` down to the wake threshold.
pub fn minutes_to_rest(sleep: &SleepParams, p0: f64) -> f64 {
    if p0 <= sleep.wake_pressure {
        return 0.0;
    }
    sleep.tau_asleep_h * (p0 / sleep.wake_pressure).ln() * 60.0
}

/// Relatedness after `minutes`, easing from `r0` toward `quality` with time constant `tau_h`.
pub fn relatedness(r0: f64, quality: f64, minutes: f64, tau_h: f64) -> f64 {
    quality + (r0 - quality) * (-(minutes.max(0.0) / 60.0) / tau_h).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sleep() -> SleepParams {
        SleepParams {
            tau_awake_h: 18.2,
            tau_asleep_h: 4.2,
            wake_pressure: 0.12,
            min_hours: 4.0,
            max_hours: 11.0,
            nap_min_minutes: 20.0,
            nap_max_minutes: 90.0,
            bedtime_after_sunset_hours: 3.3,
            day_factor: 0.15,
        }
    }

    #[test]
    fn days_are_long_in_summer_and_short_in_winter() {
        let (r_jun, s_jun) = daylight(48.0, 171);
        let (r_dec, s_dec) = daylight(48.0, 354);
        let (jun, dec) = (s_jun - r_jun, s_dec - r_dec);
        assert!((jun as f64 / 60.0 - 16.0).abs() < 0.6, "June: {jun} min");
        assert!((dec as f64 / 60.0 - 8.4).abs() < 0.6, "December: {dec} min");
        let (r_eq, s_eq) = daylight(48.0, 79);
        assert!(((s_eq - r_eq) as f64 / 60.0 - 12.0).abs() < 0.3);
    }

    #[test]
    fn circadian_is_low_by_day_and_full_at_night() {
        let s = sleep();
        let sun = (360, 1080);
        assert_eq!(circadian(&s, 720, sun), s.day_factor);
        assert_eq!(circadian(&s, 120, sun), 1.0);
        assert_eq!(circadian(&s, 1080, sun), s.day_factor, "sunset");
        assert_eq!(circadian(&s, 1140, sun), s.day_factor, "the evening");
        let gate = circadian(&s, 1080 + 168, sun);
        assert!(
            gate > s.day_factor && gate < 1.0,
            "the hour before bedtime: {gate}"
        );
        assert_eq!(circadian(&s, 1080 + 200, sun), 1.0, "after bedtime");
        let dawn = circadian(&s, 360, sun);
        assert!(dawn > s.day_factor && dawn < 1.0);
    }

    #[test]
    fn sleep_pressure_rises_awake_and_falls_asleep() {
        let s = sleep();
        let awake16 = sleep_pressure(&s, 0.1, 16.0 * 60.0, false);
        assert!(awake16 > 0.55 && awake16 < 0.7, "{awake16}");
        let slept = sleep_pressure(&s, awake16, minutes_to_rest(&s, awake16), true);
        assert!((slept - s.wake_pressure).abs() < 1e-9);
        let hours = minutes_to_rest(&s, awake16) / 60.0;
        assert!(hours > 6.0 && hours < 9.0, "a night's sleep: {hours} h");
        // Split invariance.
        let a = sleep_pressure(&s, 0.2, 300.0, false);
        let b = sleep_pressure(&s, sleep_pressure(&s, 0.2, 120.0, false), 180.0, false);
        assert!((a - b).abs() < 1e-12);
    }

    #[test]
    fn relatedness_eases_toward_the_company_quality() {
        let r = relatedness(0.2, 0.9, 1e9, 24.0);
        assert!((r - 0.9).abs() < 1e-9);
        let half = relatedness(0.0, 1.0, 24.0 * 60.0 * 2f64.ln(), 24.0);
        assert!((half - 0.5).abs() < 1e-9);
    }
}
