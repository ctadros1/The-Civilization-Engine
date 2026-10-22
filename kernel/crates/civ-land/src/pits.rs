//! Water standing in a pit (M10a slice DS; ADR-0033 §7, amending ADR-0010 §2). A pit dug below
//! the water table's head takes in water through the body it is dug in, as a wide well does
//! (research 03-02 §1.4's steady inflow), toward that head; what stands in it must be lifted out
//! before digging below it goes on. A body that passes no water (its land profile rule gives no
//! conductivity) keeps its pit dry, as before. Nothing here decides who bails or digs.

use civ_core::{PermanentId, SimTime};

/// Kilograms in a cubic metre of water.
const WATER_KG_M3: f64 = 1_000.0;
/// Standard gravity, m/s².
const G: f64 = 9.81;
/// Joules in a watt-hour.
const J_PER_WH: f64 = 3_600.0;

/// The water standing in one pit when last worked out. Saved; a pit with none has no record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitWater {
    /// The pit, by earthwork id.
    pub work: PermanentId,
    /// Cubic metres standing in it.
    pub water_m3: f64,
    /// When that was so.
    pub at: SimTime,
}

/// How a pit's water stands at a moment ([`crate::Land::pit_water_now`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitState {
    /// Cubic metres standing in it.
    pub water_m3: f64,
    /// Its floor, metres of height.
    pub floor_m: f64,
    /// The ground at its mouth, metres of height.
    pub ground_m: f64,
    /// The water table's head under it, metres of height.
    pub head_m: f64,
    /// Its plan area, square metres.
    pub area_m2: f64,
    /// The rate it fills toward the head, a day.
    pub rate_per_day: f64,
}

impl PitState {
    /// The height water stands in it, metres above its floor.
    pub fn depth_m(&self) -> f64 {
        if self.area_m2 > 0.0 {
            self.water_m3 / self.area_m2
        } else {
            0.0
        }
    }

    /// The lift from the middle of its water to its mouth, metres.
    pub fn lift_m(&self) -> f64 {
        (self.ground_m - self.floor_m - self.depth_m() / 2.0).max(0.0)
    }

    /// Cubic metres a day that come in while it is kept dry: its rate times the head over its
    /// floor and its area.
    pub fn inflow_m3_day(&self) -> f64 {
        self.rate_per_day * (self.head_m - self.floor_m).max(0.0) * self.area_m2
    }
}

/// The level water stands at after `days`, from `level` toward `toward`, closing the gap at
/// `rate` a day: the steady inflow of a column toward the head around it (or out toward a head
/// below it), `dh/dt = rate (toward - h)`.
pub fn settle(level: f64, toward: f64, rate: f64, days: f64) -> f64 {
    if !(rate > 0.0 && days > 0.0) {
        return level;
    }
    toward + (level - toward) * (-rate * days).exp()
}

/// The rate a pit of plan area `area_m2` fills toward the head, a day, in a body of hydraulic
/// conductivity `k_m_day` that it reaches `saturated_m` below the head, over a radius of
/// influence `influence_m`: 03-02 §1.4's inflow `2πT/ln(R/r)` over the pit's area, the pit taken
/// as a well of its area's radius and `T = k b`.
pub fn inflow_rate(k_m_day: f64, saturated_m: f64, area_m2: f64, influence_m: f64) -> f64 {
    let t = k_m_day.max(0.0) * saturated_m.max(0.0);
    let radius = (area_m2.max(0.0) / std::f64::consts::PI).sqrt();
    crate::wells::refill_rate(t, radius, influence_m)
}

/// Hours a person working at `watts` lifts `m3` of water through `lift_m` by hand at
/// `efficiency` (bucket and rope; research 07-06 §2.2: 1 kWh lifts 36.7 m³ through 5 m at 50 %):
/// the work `ρ g V H` over the useful power.
pub fn bail_hours(m3: f64, lift_m: f64, watts: f64, efficiency: f64) -> f64 {
    let useful = watts * efficiency;
    if useful.is_nan() || useful <= 0.0 || m3 <= 0.0 {
        return 0.0;
    }
    WATER_KG_M3 * G * m3 * lift_m.max(0.0) / (useful * J_PER_WH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_rises_toward_the_head_and_never_past_it() {
        let h = settle(0.0, 3.0, 0.5, 1.0);
        assert!((h - 3.0 * (1.0 - (-0.5f64).exp())).abs() < 1e-12);
        assert!(settle(0.0, 3.0, 0.5, 1e6) <= 3.0);
        // Over two days or over two steps of a day, the same level.
        let once = settle(0.0, 3.0, 0.5, 2.0);
        let twice = settle(settle(0.0, 3.0, 0.5, 1.0), 3.0, 0.5, 1.0);
        assert!((once - twice).abs() < 1e-12);
        // Above a head that has fallen, it drains toward it.
        assert!(settle(3.0, 1.0, 0.5, 1.0) < 3.0);
        assert_eq!(settle(2.0, 3.0, 0.0, 5.0), 2.0);
    }

    #[test]
    fn lifting_water_by_hand_is_its_work_over_a_persons_power() {
        // 07-06 §2.2: 1 kWh lifts 36.7 m³ through 5 m at 50 %: a person putting in 75 W at 50 %
        // takes that kilowatt-hour's 1,000 Wh / 75 W = 13.3 hours.
        let h = bail_hours(36.7, 5.0, 75.0, 0.5);
        assert!((h - 1_000.0 / 75.0).abs() < 0.05, "{h}");
        assert_eq!(bail_hours(0.0, 5.0, 75.0, 0.5), 0.0);
        assert_eq!(bail_hours(1.0, 5.0, 0.0, 0.5), 0.0);
    }

    #[test]
    fn a_body_that_passes_no_water_takes_none_in() {
        assert_eq!(inflow_rate(0.0, 3.0, 9.0, 50.0), 0.0);
        assert_eq!(inflow_rate(0.5, 0.0, 9.0, 50.0), 0.0);
        assert!(inflow_rate(0.5, 3.0, 9.0, 50.0) > 0.0);
    }
}
