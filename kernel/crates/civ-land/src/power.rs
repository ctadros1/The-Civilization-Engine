//! Water power at a site (M10a slice DR; ADR-0033 §5; the M10 power brief §1.3): an installation
//! is a weir and headrace taking from a river reach to a wheel that falls `head_m` to its tail.
//! Its power is worked out each day from the reach's discharge and never saved: the water it
//! takes is the least of what its headrace carries and the share of the reach's flow an intake
//! may take (as a ditch's, ADR-0022 §11), and all of it goes back to the river at the tail, so a
//! wheel takes nothing from the reaches below. P = ρ g Q H η (research 07-06 §1.3: 0.5 m³/s
//! through 2 m at 50 % is 4.9 kW), no more than the wheel can deliver.

use civ_core::PermanentId;

/// Water's weight, kilonewtons a cubic metre: ρ g, so that ρ g Q H is kilowatts for Q in m³/s
/// and H in metres.
pub const WATER_KN_PER_M3: f64 = 9.81;

/// One installation: where its wheel stands, the reach its intake takes from, the fall its water
/// drops through at the wheel, what its headrace carries running full, the wheel's efficiency and
/// rated power, who owns it, and the building it drives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Installation {
    pub id: PermanentId,
    pub reach: u32,
    /// The wheel's place, metres from the map's corner.
    pub at: [f64; 2],
    /// The fall from the headrace's water to the tail at the wheel, metres: the bed's drop from
    /// intake to tail less the headrace's own fall.
    pub head_m: f64,
    /// What its headrace carries running full, m³/s (Manning's, as a ditch's).
    pub race_m3s: f64,
    /// The share of the water's power the wheel turns into work (research 07-06 §2.2: simple
    /// wheels 20-40 %, overshot 50-70 %).
    pub efficiency: f64,
    /// The most the wheel and its gearing deliver, kilowatts.
    pub rated_kw: f64,
    /// The household, firm or polity that owns it.
    pub owner: PermanentId,
    /// The building it drives, once one stands.
    pub building: Option<PermanentId>,
}

impl Installation {
    /// The water it takes today, m³/s, from a reach carrying `discharge_m3s`: no more than its
    /// headrace carries or `max_share` of the reach.
    pub fn flow_m3s(&self, discharge_m3s: f64, max_share: f64) -> f64 {
        let share = max_share.clamp(0.0, 1.0) * discharge_m3s.max(0.0);
        self.race_m3s.max(0.0).min(share)
    }

    /// Its power today, kilowatts, from a reach carrying `discharge_m3s`: ρ g Q H η, no more
    /// than its rated power. Nothing from a dry reach.
    pub fn power_kw(&self, discharge_m3s: f64, max_share: f64) -> f64 {
        let q = self.flow_m3s(discharge_m3s, max_share);
        (WATER_KN_PER_M3 * q * self.head_m.max(0.0) * self.efficiency.clamp(0.0, 1.0))
            .min(self.rated_kw.max(0.0))
    }

    /// Whether its record is whole: finite, of a positive fall, headrace and rating, and an
    /// efficiency within (0, 1].
    pub fn valid(&self) -> bool {
        let finite = [
            self.at[0],
            self.at[1],
            self.head_m,
            self.race_m3s,
            self.efficiency,
            self.rated_kw,
        ]
        .iter()
        .all(|v| v.is_finite());
        finite
            && self.head_m > 0.0
            && self.race_m3s > 0.0
            && self.efficiency > 0.0
            && self.efficiency <= 1.0
            && self.rated_kw > 0.0
    }
}

/// An engine set at a pit (M10a slice DS, third part; ADR-0033 §7): it burns fuel to pump the
/// pit's water out, its owner holding the engine itself and its fuel. Its rated power and the
/// share of its fuel's heat it turns into work are those of the engine set, kept with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Engine {
    pub id: PermanentId,
    /// The pit it pumps, by earthwork id.
    pub pit: PermanentId,
    /// The household that set it and keeps it going.
    pub owner: PermanentId,
    /// The most it delivers, kilowatts.
    pub rated_kw: f64,
    /// The share of its fuel's heat it turns into work (research 07-06 §2.3: 0.5-1.5 %
    /// atmospheric).
    pub efficiency: f64,
}

impl Engine {
    /// Whether its record is whole: finite, of a positive rating and an efficiency in (0, 1].
    pub fn valid(&self) -> bool {
        self.rated_kw.is_finite()
            && self.efficiency.is_finite()
            && self.rated_kw > 0.0
            && self.efficiency > 0.0
            && self.efficiency <= 1.0
    }
}

/// Kilowatt-hours of work at a pump's shaft that lift `m3` of water through `lift_m` with a pump
/// of efficiency `pump_efficiency` (research 07-06 §2.2: 1 kWh lifts 36.7 m³ through 5 m at
/// 50 %): ρ g V H over the pump's share, in kWh.
pub fn lift_kwh(m3: f64, lift_m: f64, pump_efficiency: f64) -> f64 {
    if m3 <= 0.0 || lift_m <= 0.0 || pump_efficiency.is_nan() || pump_efficiency <= 0.0 {
        return 0.0;
    }
    WATER_KN_PER_M3 * m3 * lift_m / (3_600.0 * pump_efficiency)
}

/// Kilograms of a fuel of `mj_per_kg` an engine of `efficiency` burns for `kwh` of work
/// (research 07-06 §2.3: 3.6 / (MJ η) a kWh, 14.4 kg of coal at 1 %).
pub fn fuel_kg(kwh: f64, mj_per_kg: f64, efficiency: f64) -> f64 {
    let heat = mj_per_kg * efficiency;
    if kwh <= 0.0 || heat.is_nan() || heat <= 0.0 {
        return 0.0;
    }
    3.6 * kwh / heat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_atmospheric_engine_burns_the_briefs_coal_for_its_water() {
        // 07-06 §2.3: a kWh at 1 % takes 14.4 kg of coal of 25 MJ; §2.2: 1 kWh lifts 36.7 m³
        // through 5 m at 50 %.
        assert!((fuel_kg(1.0, 25.0, 0.01) - 14.4).abs() < 1e-9);
        assert!((lift_kwh(36.7, 5.0, 0.5) - 1.0).abs() < 0.001);
        assert_eq!(lift_kwh(0.0, 5.0, 0.5), 0.0);
        assert_eq!(fuel_kg(1.0, 25.0, 0.0), 0.0);
        let e = Engine {
            id: PermanentId::from_raw(1).expect("id"),
            pit: PermanentId::from_raw(2).expect("id"),
            owner: PermanentId::from_raw(3).expect("id"),
            rated_kw: 4.0,
            efficiency: 0.01,
        };
        assert!(e.valid());
        assert!(
            !Engine {
                efficiency: 0.0,
                ..e
            }
            .valid()
        );
    }

    fn wheel() -> Installation {
        Installation {
            id: PermanentId::from_raw(1).expect("id"),
            reach: 0,
            at: [100.0, 200.0],
            head_m: 2.0,
            race_m3s: 0.5,
            efficiency: 0.5,
            rated_kw: 10.0,
            owner: PermanentId::from_raw(2).expect("id"),
            building: None,
        }
    }

    #[test]
    fn half_a_cubic_metre_through_two_metres_at_half_gives_the_briefs_kilowatts() {
        // Research 07-06 §1.3: 0.5 m³/s at 2 m and 50 % is 4.9 kW.
        let w = wheel();
        assert!((w.power_kw(10.0, 0.5) - 4.905).abs() < 1e-9);
        assert!(w.valid());
    }

    #[test]
    fn a_low_river_gives_its_share_and_a_dry_one_nothing() {
        let w = wheel();
        // A reach of 0.4 m³/s spares half of it: 0.2 m³/s, two fifths of the power.
        assert!((w.flow_m3s(0.4, 0.5) - 0.2).abs() < 1e-12);
        assert!((w.power_kw(0.4, 0.5) - 0.4 * 4.905).abs() < 1e-9);
        assert_eq!(w.power_kw(0.0, 0.5), 0.0);
        assert_eq!(w.power_kw(-1.0, 0.5), 0.0);
    }

    #[test]
    fn a_flood_gives_no_more_than_the_headrace_carries_or_the_wheel_delivers() {
        let mut w = wheel();
        assert_eq!(w.flow_m3s(1000.0, 0.5), 0.5);
        w.race_m3s = 5.0;
        assert_eq!(w.power_kw(1000.0, 0.5), 10.0);
    }

    #[test]
    fn a_record_without_a_fall_or_a_wheel_is_not_whole() {
        for broken in [
            Installation {
                head_m: 0.0,
                ..wheel()
            },
            Installation {
                efficiency: 1.5,
                ..wheel()
            },
            Installation {
                rated_kw: f64::NAN,
                ..wheel()
            },
            Installation {
                race_m3s: 0.0,
                ..wheel()
            },
        ] {
            assert!(!broken.valid());
        }
    }
}
