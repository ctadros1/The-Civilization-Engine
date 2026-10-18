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

/// The world's travel tally (M8b slice CR, first part).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Travel {
    /// By year (from 0) and settlement.
    pub years: BTreeMap<(i64, PermanentId), TravelYear>,
    /// Each person's minutes of travel so far today, by person.
    pub today: BTreeMap<PermanentId, f32>,
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

    /// The day of `year` is over: each of `residents` (person, settlement, occupation) has their
    /// day's travel counted in its band, and the day's minutes begin again.
    pub fn close_day(
        &mut self,
        year: i64,
        residents: impl IntoIterator<Item = (PermanentId, PermanentId, Occupation)>,
    ) {
        for (person, settlement, occupation) in residents {
            let minutes = self.today.get(&person).copied().unwrap_or(0.0);
            let band = DAY_BANDS_MIN.iter().filter(|&&e| minutes >= e).count();
            let y = self.years.entry((year, settlement)).or_default();
            y.days[occupation as usize][band] += 1;
            y.minutes[occupation as usize] += f64::from(minutes);
        }
        self.today.clear();
    }

    /// What is wrong with the tally, for the invariant checks.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.today.values().any(|m| !(m.is_finite() && *m >= 0.0)) {
            out.push("a day's travel is malformed".to_owned());
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
        t.close_day(
            0,
            [
                (a, s, Occupation::Fields),
                (b, s, Occupation::Fields),
                (id(4), s, Occupation::Young),
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
        assert!(t.problems().is_empty());
    }
}
