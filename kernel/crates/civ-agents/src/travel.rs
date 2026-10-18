//! Travel measured (M8b slice CR, first part; ADR-0029 §9; the transport brief §1.7: "measured,
//! never capped"; research 12-05 §4.1: "full distributions of daily travel time and distance,
//! separated by occupation and trip purpose"). Each settlement's year holds the trips its people
//! set off on, by purpose (the behaviour of the activity they went for) and by mode, the minutes
//! they waited at bottlenecks, and each resident's day of travel, by occupation, in bands of
//! minutes. It is a tally: nothing reads it to choose, so a world lives the same with it or
//! without it.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// The edges of the bands a day's travel falls in, minutes: none, under 15, under 30, under 45,
/// under 60, under 90, under 120, and 120 or more (12-05 §2.3's 45–90 minutes a check between).
pub const DAY_BANDS_MIN: [f32; 7] = [f32::MIN_POSITIVE, 15.0, 30.0, 45.0, 60.0, 90.0, 120.0];
/// How many bands a day's travel falls in.
pub const BANDS: usize = DAY_BANDS_MIN.len() + 1;

/// How someone goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Walk,
    Boat,
    Cart,
}

impl Mode {
    /// Every mode, in code order.
    pub const ALL: [Mode; 3] = [Mode::Walk, Mode::Boat, Mode::Cart];

    /// In words.
    pub fn name(self) -> &'static str {
        match self {
            Mode::Walk => "on foot",
            Mode::Boat => "by boat",
            Mode::Cart => "by cart",
        }
    }
}

/// What someone's household lives by, as the day's tally sees it: a workshop it keeps open, else
/// fields it holds, else neither; and the young, whatever their household does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Occupation {
    Workshop,
    Fields,
    Other,
    Young,
}

impl Occupation {
    /// Every occupation, in code order.
    pub const ALL: [Occupation; 4] = [
        Occupation::Workshop,
        Occupation::Fields,
        Occupation::Other,
        Occupation::Young,
    ];

    /// In words.
    pub fn name(self) -> &'static str {
        match self {
            Occupation::Workshop => "workshop households",
            Occupation::Fields => "farming households",
            Occupation::Other => "other households",
            Occupation::Young => "the young",
        }
    }
}

/// Trips and their minutes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tally {
    pub trips: u32,
    pub minutes: f64,
}

impl Tally {
    fn add(&mut self, minutes: f64) {
        self.trips += 1;
        self.minutes += minutes;
    }
}

/// A settlement's year of travel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TravelYear {
    /// Trips set off on, by the code of the behaviour they were for.
    pub purposes: BTreeMap<u8, Tally>,
    /// Trips set off on, by mode (in [`Mode::ALL`] order).
    pub modes: [Tally; 3],
    /// Waits at bottlenecks of a minute or more.
    pub waits: Tally,
    /// Residents' days, by occupation (in [`Occupation::ALL`] order) and band of their day's
    /// travel (see [`DAY_BANDS_MIN`]).
    pub days: [[u32; BANDS]; 4],
    /// Residents' minutes of travel, by occupation.
    pub minutes: [f64; 4],
}

impl TravelYear {
    /// The band the median day's travel of those of `occupation` falls in (see
    /// [`DAY_BANDS_MIN`]); `None` without a day.
    pub fn median_band(&self, occupation: Occupation) -> Option<usize> {
        let row = &self.days[occupation as usize];
        let all: u32 = row.iter().sum();
        if all == 0 {
            return None;
        }
        let mut seen = 0;
        row.iter().position(|&n| {
            seen += n;
            2 * seen >= all
        })
    }
}

/// What a household holds of its members' days of travel (M8b slice CR, second part): their
/// minutes and the member-days they came to, both fading at the places' half-life, as of `day`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HouseholdTravel {
    pub minutes: f64,
    pub days: f64,
    pub day: i64,
}

/// The world's travel tally (M8b slice CR, first part).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Travel {
    /// By year (from 0) and settlement.
    pub years: BTreeMap<(i64, PermanentId), TravelYear>,
    /// Each person's minutes of travel so far today, by person.
    pub today: BTreeMap<PermanentId, f32>,
    /// What each household holds of its members' days of travel, by household (the second
    /// part): what the residence review weighs.
    pub households: BTreeMap<PermanentId, HouseholdTravel>,
}

impl Travel {
    /// `person` of `settlement` set off in `year` for `purpose` (a behaviour's code) by `mode`,
    /// on a trip of `minutes`.
    pub fn trip(
        &mut self,
        year: i64,
        settlement: PermanentId,
        person: PermanentId,
        purpose: u8,
        mode: Mode,
        minutes: f32,
    ) {
        if !(minutes.is_finite() && minutes >= 0.0) {
            return;
        }
        let y = self.years.entry((year, settlement)).or_default();
        y.purposes
            .entry(purpose)
            .or_default()
            .add(f64::from(minutes));
        y.modes[mode as usize].add(f64::from(minutes));
        *self.today.entry(person).or_default() += minutes;
    }

    /// `person` of `settlement` waited `minutes` at a bottleneck in `year`; `beyond` when the
    /// wait lengthened a trip already counted (a walker's at a crossing or a gate), not one
    /// counted with its wait in it (a cart's at a narrows).
    pub fn waited(
        &mut self,
        year: i64,
        settlement: PermanentId,
        person: PermanentId,
        minutes: f32,
        beyond: bool,
    ) {
        if !(minutes.is_finite() && minutes > 0.0) {
            return;
        }
        let y = self.years.entry((year, settlement)).or_default();
        y.waits.add(f64::from(minutes));
        if beyond {
            *self.today.entry(person).or_default() += minutes;
        }
    }

    /// The day `day` of `year` is over: each of `residents` (person, household, settlement,
    /// occupation) has their day's travel counted in its band and held by their household, fading
    /// at `half_life_days`, and the day's minutes begin again.
    pub fn close_day(
        &mut self,
        year: i64,
        day: i64,
        half_life_days: f64,
        residents: impl IntoIterator<Item = (PermanentId, PermanentId, PermanentId, Occupation)>,
    ) {
        for (person, household, settlement, occupation) in residents {
            let minutes = self.today.get(&person).copied().unwrap_or(0.0);
            let band = DAY_BANDS_MIN.iter().filter(|&&e| minutes >= e).count();
            let y = self.years.entry((year, settlement)).or_default();
            y.days[occupation as usize][band] += 1;
            y.minutes[occupation as usize] += f64::from(minutes);
            let h = self.households.entry(household).or_insert(HouseholdTravel {
                day,
                ..HouseholdTravel::default()
            });
            if day > h.day {
                let f = f64::from(crate::uses::fading(day - h.day, half_life_days));
                h.minutes *= f;
                h.days *= f;
                h.day = day;
            }
            h.minutes += f64::from(minutes);
            h.days += 1.0;
        }
        self.today.clear();
    }

    /// The minutes a day household `household`'s members travel, as it holds them: none until it
    /// holds a week of member-days.
    pub fn household_daily_minutes(&self, household: PermanentId) -> Option<f64> {
        self.households
            .get(&household)
            .filter(|h| h.days >= 7.0)
            .map(|h| h.minutes / h.days)
    }

    /// The minutes a day settlement `settlement`'s residents have travelled, over every year
    /// counted: none without a day counted.
    pub fn settlement_daily_minutes(&self, settlement: PermanentId) -> Option<f64> {
        let (minutes, days) = self
            .years
            .iter()
            .filter(|((_, s), _)| *s == settlement)
            .fold((0.0, 0u64), |(m, d), (_, y)| {
                (
                    m + y.minutes.iter().sum::<f64>(),
                    d + y.days.iter().flatten().map(|&n| u64::from(n)).sum::<u64>(),
                )
            });
        (days > 0).then(|| minutes / days as f64)
    }

    /// What is wrong with the tally, for the invariant checks.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.today.values().any(|m| !(m.is_finite() && *m >= 0.0)) {
            out.push("a day's travel is malformed".to_owned());
        }
        if self.households.values().any(|h| {
            !(h.minutes.is_finite() && h.minutes >= 0.0 && h.days.is_finite() && h.days >= 0.0)
        }) {
            out.push("a household's days of travel are malformed".to_owned());
        }
        for ((year, s), y) in &self.years {
            let bad = |t: &Tally| !(t.minutes.is_finite() && t.minutes >= 0.0);
            if y.purposes.values().any(bad)
                || y.modes.iter().any(bad)
                || bad(&y.waits)
                || y.minutes.iter().any(|m| !(m.is_finite() && *m >= 0.0))
            {
                out.push(format!(
                    "settlement {s}'s travel in year {year} is malformed"
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn a_day_of_travel_is_counted_in_its_band_by_occupation_and_begins_again() {
        let mut t = Travel::default();
        let (s, a, b) = (id(1), id(2), id(3));
        t.trip(0, s, a, 4, Mode::Walk, 20.0);
        t.trip(0, s, a, 4, Mode::Walk, 30.0);
        t.waited(0, s, a, 2.0, true);
        t.trip(0, s, b, 7, Mode::Cart, 10.0);
        let home = id(9);
        t.close_day(
            0,
            5,
            180.0,
            [
                (a, home, s, Occupation::Fields),
                (b, home, s, Occupation::Fields),
                (id(4), home, s, Occupation::Young),
            ],
        );
        let y = &t.years[&(0, s)];
        assert_eq!(
            y.purposes[&4],
            Tally {
                trips: 2,
                minutes: 50.0
            }
        );
        assert_eq!(y.modes[Mode::Cart as usize].trips, 1);
        assert_eq!(
            y.waits,
            Tally {
                trips: 1,
                minutes: 2.0
            }
        );
        // 52 minutes falls in the 45–60 band (the fifth), 10 in the first with any travel, none
        // in the first.
        let fields = y.days[Occupation::Fields as usize];
        assert_eq!((fields[1], fields[4]), (1, 1));
        assert_eq!(y.days[Occupation::Young as usize][0], 1);
        assert!((y.minutes[Occupation::Fields as usize] - 62.0).abs() < 1e-9);
        assert_eq!(y.median_band(Occupation::Fields), Some(1));
        assert!(t.today.is_empty());
        // The household holds its three member-days, too few yet to say what its days are.
        assert_eq!(
            t.households[&home],
            HouseholdTravel {
                minutes: 62.0,
                days: 3.0,
                day: 5
            }
        );
        assert_eq!(t.household_daily_minutes(home), None);
        assert!((t.settlement_daily_minutes(s).expect("counted") - 62.0 / 3.0).abs() < 1e-9);
        assert!(t.problems().is_empty());
    }
}
