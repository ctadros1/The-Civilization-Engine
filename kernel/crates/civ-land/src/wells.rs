//! Wells (M6a slice AY, step three; ADR-0021 §3; research 03-02 §1.4, 12-01 §2.2): a shaft a
//! household digs and lines beside its home, a record of its own as a crossing is. It is dug
//! and lined a metre at a time toward the depth its diggers expect to meet water; at that depth
//! it meets water where the true water table stands above its floor, or it is dug deeper, or
//! given up. An open well's water is the column standing in its shaft: each load drawn lowers
//! it, and it refills toward the water table of its patch at the rate the ground lets water in
//! (03-02 §1.4: shaft storage plus inflow, `Q ≈ 2πT(h − h_w)/ln(r_e/r_w)`), so "depth alone does
//! not determine yield". What is drawn is taken from the aquifer at the day's end. Its lining
//! rots by the wetness of its ground, and when it has rotted through the shaft falls in.

use civ_core::{PermanentId, SimTime};

use crate::fields::RectCm;

/// What has become of a well. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WellState {
    /// Being dug and lined.
    Digging,
    /// Its shaft met water and it is in use, since a time.
    Open { since: SimTime },
    /// Dug as deep as its diggers would go without meeting water, and given up.
    GivenUp { on: SimTime },
    /// Its lining rotted through and the shaft fell in.
    FellIn { on: SimTime },
}

impl WellState {
    /// Its code in saves, and the time it holds (0 while digging).
    pub fn code(self) -> (u8, SimTime) {
        match self {
            WellState::Digging => (0, SimTime::ZERO),
            WellState::Open { since } => (1, since),
            WellState::GivenUp { on } => (2, on),
            WellState::FellIn { on } => (3, on),
        }
    }

    /// The state with code `code` and time `at`.
    pub fn from_code(code: u8, at: SimTime) -> Option<WellState> {
        Some(match code {
            0 => WellState::Digging,
            1 => WellState::Open { since: at },
            2 => WellState::GivenUp { on: at },
            3 => WellState::FellIn { on: at },
            _ => return None,
        })
    }
}

/// A well. Saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Well {
    pub id: PermanentId,
    /// Its well system, by index in the catalog's wells.
    pub system: usize,
    /// The terrain cell its shaft is sunk in.
    pub cell: u32,
    /// The ground its digging took: no plot, field or pit may be laid over it.
    pub rect: RectCm,
    /// The household it is the well of.
    pub household: PermanentId,
    /// The ground at its mouth, metres.
    pub ground_m: f32,
    /// How deep it is dug and lined, metres.
    pub depth_m: f32,
    /// How deep its diggers now mean to take it, metres.
    pub target_m: f32,
    /// Labour put into it, hours: each metre takes its system's digging and lining.
    pub work_h: f32,
    /// Its diggers' building skill, weighted by the hours each put in (ADR-0009 §6).
    pub skill_h: f32,
    /// Its lining's quality, drawn when it opens (0 until then).
    pub quality: f32,
    /// The share of its lining rot has taken: the lining gives way, and the shaft falls in, once
    /// rot has taken as much as its quality left (a lining laid poorly goes sooner).
    pub loss: f32,
    /// Hours of relining still owed, while its household is relining it (0 otherwise).
    pub mend_h: f32,
    /// The hours of walking a year each hour of the work owed on it saves its household, as its
    /// household weighed it when it began the work.
    pub worth: f32,
    /// The water surface in its shaft at `level_at`, metres.
    pub level_m: f64,
    pub level_at: SimTime,
    /// Litres drawn from it since the last day's end, taken from the aquifer then.
    pub drawn_l: f64,
    pub begun: SimTime,
    pub state: WellState,
    /// The day work on it was last begun and how many began a session of it that day, the most
    /// its shaft has room for being its system's crew.
    pub crew: (i64, u8),
}

impl Well {
    /// The floor of its shaft, metres.
    pub fn floor_m(&self) -> f64 {
        f64::from(self.ground_m) - f64::from(self.depth_m)
    }

    /// Whether it is open: its water may be drawn.
    pub fn is_open(&self) -> bool {
        matches!(self.state, WellState::Open { .. })
    }

    /// Whether work is owed on it: it is being dug, or relined.
    pub fn is_worked(&self) -> bool {
        self.state == WellState::Digging || (self.is_open() && self.mend_h > 0.0)
    }

    /// The water surface in its shaft at `now`, its patch's water table standing at `head` and
    /// its column refilling at `rate` a day: toward the water table (or its floor, if the table
    /// stands below it), never below its floor.
    pub fn level_now(&self, now: SimTime, head: f64, rate: f64) -> f64 {
        let floor = self.floor_m();
        let toward = head.max(floor);
        let days = (now.minutes() - self.level_at.minutes()).max(0) as f64 / 1440.0;
        let level = toward + (self.level_m - toward) * (-rate.max(0.0) * days).exp();
        level.max(floor)
    }

    /// Litres standing in its shaft at `now` over a column of `area_m2`.
    pub fn litres(&self, now: SimTime, head: f64, rate: f64, area_m2: f64) -> f64 {
        (self.level_now(now, head, rate) - self.floor_m()).max(0.0) * area_m2 * 1000.0
    }

    /// Brings its water up to `now`.
    pub fn settle(&mut self, now: SimTime, head: f64, rate: f64) {
        self.level_m = self.level_now(now, head, rate);
        self.level_at = now;
    }

    /// Draws `litres` at `now` if its shaft holds them; whether it did.
    pub fn draw(&mut self, now: SimTime, head: f64, rate: f64, area_m2: f64, litres: f64) -> bool {
        self.settle(now, head, rate);
        let held = (self.level_m - self.floor_m()).max(0.0) * area_m2 * 1000.0;
        if held < litres || area_m2 <= 0.0 {
            return false;
        }
        self.level_m -= litres / 1000.0 / area_m2;
        self.drawn_l += litres;
        true
    }
}

/// The rate a well's column refills toward the water table, a day: 03-02 §1.4's steady inflow
/// `2πT(h − h_w)/ln(r_e/r_w)` over the column's area `πr_w²`, for ground of transmissivity
/// `t_m2_day`, a column of radius `radius_m` and a radius of influence `influence_m`.
pub fn refill_rate(t_m2_day: f64, radius_m: f64, influence_m: f64) -> f64 {
    if !(t_m2_day > 0.0 && radius_m > 0.0) {
        return 0.0;
    }
    let ln = (influence_m / radius_m).max(1.0 + 1e-9).ln();
    2.0 * t_m2_day / (ln * radius_m * radius_m)
}

/// Every well, in the order begun. Saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wells {
    pub list: Vec<Well>,
}

impl Wells {
    /// The well with id `id`.
    pub fn get(&self, id: PermanentId) -> Option<&Well> {
        self.list.iter().find(|w| w.id == id)
    }

    /// The well with id `id`, to change.
    pub fn get_mut(&mut self, id: PermanentId) -> Option<&mut Well> {
        self.list.iter_mut().find(|w| w.id == id)
    }

    /// Whether any well, open, dug or fallen in, lies within `gap_cm` of `rect`.
    pub fn near(&self, rect: &RectCm, gap_cm: i32) -> bool {
        self.list.iter().any(|w| w.rect.near(rect, gap_cm))
    }

    /// What is wrong with saved wells, if anything.
    pub fn problems(&self, systems: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut ids: Vec<PermanentId> = self.list.iter().map(|w| w.id).collect();
        ids.sort_unstable();
        if ids.windows(2).any(|p| p[0] == p[1]) {
            out.push("two wells share an id".to_owned());
        }
        for w in &self.list {
            let finite = [
                w.ground_m, w.depth_m, w.target_m, w.work_h, w.skill_h, w.quality, w.mend_h,
                w.worth,
            ]
            .iter()
            .all(|v| v.is_finite())
                && w.level_m.is_finite()
                && w.drawn_l.is_finite();
            if !finite {
                out.push(format!("well {} has a value that is not a number", w.id));
            }
            if w.system >= systems {
                out.push(format!("well {} is of an unknown system", w.id));
            }
            if w.depth_m < 0.0
                || w.target_m < 0.0
                || w.work_h < 0.0
                || w.mend_h < 0.0
                || w.drawn_l < 0.0
            {
                out.push(format!(
                    "well {} has a negative depth, labour or draw",
                    w.id
                ));
            }
            if !(0.0..=1.0).contains(&w.loss) || !(0.0..=1.0).contains(&w.quality) {
                out.push(format!("well {} has a lining out of range", w.id));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well(depth: f32, level: f64) -> Well {
        Well {
            id: PermanentId::from_raw(1).expect("an id"),
            system: 0,
            cell: 0,
            rect: RectCm {
                x: 0,
                y: 0,
                w: 150,
                h: 150,
            },
            household: PermanentId::from_raw(2).expect("an id"),
            ground_m: 100.0,
            depth_m: depth,
            target_m: depth,
            work_h: 0.0,
            skill_h: 0.0,
            quality: 0.5,
            loss: 0.0,
            mend_h: 0.0,
            worth: 0.0,
            level_m: level,
            level_at: SimTime::ZERO,
            drawn_l: 0.0,
            begun: SimTime::ZERO,
            state: WellState::Open {
                since: SimTime::ZERO,
            },
            crew: (0, 0),
        }
    }

    #[test]
    fn a_column_refills_toward_the_water_table_and_never_below_its_floor() {
        // A 4 m shaft (floor at 96 m) in ground whose water table stands at 98 m.
        let area = std::f64::consts::PI * 0.6 * 0.6;
        let rate = refill_rate(10.0, 0.6, 50.0);
        let w = well(4.0, 96.5);
        let day = SimTime::from_minutes(1440);
        let refilled = w.level_now(day, 98.0, rate);
        assert!(refilled > 96.5 && refilled < 98.0, "{refilled}");
        let expect = 98.0 - 1.5 * (-rate).exp();
        assert!((refilled - expect).abs() < 1e-12);
        // Two metres of column hold about 2.26 m³ (03-02 §1.4's worked example).
        let full = w.litres(SimTime::from_minutes(1440 * 365), 98.0, rate, area);
        assert!((full - 2262.0).abs() < 1.0, "{full}");
        // A water table fallen below the floor leaves the shaft dry, not below it.
        assert_eq!(
            w.level_now(SimTime::from_minutes(1440 * 365), 90.0, rate),
            96.0
        );
        // Loads come out of the column; one it does not hold is refused.
        let mut w = w;
        assert!(w.draw(SimTime::from_minutes(1440 * 400), 98.0, rate, area, 15.0));
        assert!((w.drawn_l - 15.0).abs() < 1e-12);
        let mut dry = well(4.0, 96.0);
        assert!(!dry.draw(SimTime::ZERO, 90.0, rate, area, 15.0));
        assert_eq!(dry.drawn_l, 0.0);
    }

    #[test]
    fn tight_ground_refills_a_column_slowly_and_open_ground_at_once() {
        // 03-02 §2.2: silt at 0.001-0.1 m a day, sand at 0.1-100, over 10 m.
        let silt = refill_rate(0.01 * 10.0, 0.6, 50.0);
        let sand = refill_rate(30.0 * 10.0, 0.6, 50.0);
        assert!(1.0 / silt > 0.5, "silt refills in {} days", 1.0 / silt);
        assert!(1.0 / sand < 0.01, "sand refills in {} days", 1.0 / sand);
        assert_eq!(refill_rate(0.0, 0.6, 50.0), 0.0);
    }

    #[test]
    fn states_round_trip_their_codes() {
        let t = SimTime::from_minutes(77);
        for s in [
            WellState::Digging,
            WellState::Open { since: t },
            WellState::GivenUp { on: t },
            WellState::FellIn { on: t },
        ] {
            let (c, at) = s.code();
            assert_eq!(WellState::from_code(c, at), Some(s));
        }
        assert_eq!(WellState::from_code(4, t), None);
    }
}
