//! Trust and caution (ADR-0009 §6): what each settlement has seen of the buildings of each
//! technique, and how much stronger than usual its builders make the next ones.
//!
//! A settlement remembers the failures of a technique's buildings and the building-years they
//! stood, each fading by half over the profile's half-life (personal memory of a disaster fades
//! over 2-20 years, research 11-09 §2.3). Their ratio is the failure rate builders believe in;
//! caution rises with it toward the profile's most, and frame programs' joists and posts are made
//! that many times as strong ([`crate::build::design_cautious`]). Lessons are about a technique's
//! buildings as a whole, not yet about one span, material or load (11-06 §5.6).

use civ_core::{PermanentId, SimTime, time::MINUTES_PER_YEAR};

/// How builders answer what their settlement has seen (the people profile's `[build.caution]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CautionParams {
    /// Years over which what was seen fades by half.
    pub half_life_years: f64,
    /// The most caution comes to: how many times its usual strength a member is made at most.
    pub most: f64,
    /// Failures a building-year at which caution is half way to its most.
    pub half_rate: f64,
    /// How many failures more each death in one counts as.
    pub death_weight: f64,
}

/// Building-years of experience below which a settlement reckons as if it had this many: one
/// failure among a few new buildings is not taken for a rate of one in a few.
pub const LEAST_YEARS: f64 = 10.0;

/// What settlement `settlement` has seen of the buildings of technique `technique`: failures and
/// building-years of use, each fading by half every half-life, as of `at`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trust {
    /// The settlement.
    pub settlement: PermanentId,
    /// The technique, by index in the catalog's techniques.
    pub technique: u16,
    /// Failures seen, a death in one counting as [`CautionParams::death_weight`] more.
    pub failures: f64,
    /// Years its standing buildings of the technique have stood, one a building a year.
    pub years: f64,
    /// When both were last brought up to date.
    pub at: SimTime,
}

impl Trust {
    /// A settlement that has seen nothing yet of `technique`'s buildings, as of `at`.
    pub fn new(settlement: PermanentId, technique: u16, at: SimTime) -> Self {
        Self {
            settlement,
            technique,
            failures: 0.0,
            years: 0.0,
            at,
        }
    }

    /// Its failures and building-years as they stand at `now`, faded by half every
    /// `half_life_years`.
    pub fn faded(&self, now: SimTime, half_life_years: f64) -> (f64, f64) {
        let years = (now.minutes() - self.at.minutes()).max(0) as f64 / MINUTES_PER_YEAR as f64;
        let keep = 0.5f64.powf(years / half_life_years.max(1e-6));
        (self.failures * keep, self.years * keep)
    }

    /// Brings both up to `now`.
    pub fn fade(&mut self, now: SimTime, half_life_years: f64) {
        (self.failures, self.years) = self.faded(now, half_life_years);
        self.at = self.at.max(now);
    }

    /// How many times its usual strength builders make a member at `now`: 1 when nothing has
    /// failed, rising toward [`CautionParams::most`] as the failures they remember come to
    /// [`CautionParams::half_rate`] a building-year and beyond.
    pub fn caution(&self, now: SimTime, p: &CautionParams) -> f64 {
        let (failures, years) = self.faded(now, p.half_life_years);
        let rate = failures / years.max(LEAST_YEARS);
        1.0 + (p.most - 1.0).max(0.0) * rate / (rate + p.half_rate.max(1e-12))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> CautionParams {
        CautionParams {
            half_life_years: 8.0,
            most: 2.0,
            half_rate: 0.01,
            death_weight: 2.0,
        }
    }

    fn year(y: i64) -> SimTime {
        SimTime::from_minutes(y * MINUTES_PER_YEAR)
    }

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn nothing_seen_is_no_caution() {
        let t = Trust::new(id(1), 0, year(0));
        assert_eq!(t.caution(year(5), &params()), 1.0);
    }

    #[test]
    fn a_failure_among_few_buildings_counts_against_the_least_years() {
        let mut t = Trust::new(id(1), 0, year(0));
        t.failures = 1.0;
        t.years = 2.0;
        // 1 over 10 building-years: 0.1 a year, ten times the half rate.
        let c = t.caution(year(0), &params());
        assert!((c - (1.0 + 0.1 / 0.11)).abs() < 1e-9, "{c}");
    }

    #[test]
    fn caution_is_half_way_at_the_half_rate_and_fades_back_as_years_pass_without_failure() {
        let p = params();
        let mut t = Trust::new(id(1), 0, year(0));
        t.failures = 1.0;
        t.years = 100.0;
        assert!((t.caution(year(0), &p) - 1.5).abs() < 1e-9);
        // Eight more years of 20 buildings standing and nothing failing.
        for y in 1..=8 {
            t.fade(year(y), p.half_life_years);
            t.years += 20.0;
        }
        let later = t.caution(year(8), &p);
        assert!(later > 1.2 && later < 1.25, "{later}");
        // Fading alone keeps the ratio: what matters is the years that kept standing.
        let (f, y) = t.faded(year(16), p.half_life_years);
        assert!((f / y - t.failures / t.years).abs() < 1e-12);
    }

    #[test]
    fn caution_never_passes_its_most() {
        let p = params();
        let mut t = Trust::new(id(1), 0, year(0));
        t.failures = 1e6;
        t.years = 1.0;
        let c = t.caution(year(0), &p);
        assert!(c < 2.0 && c > 1.99, "{c}");
    }
}
