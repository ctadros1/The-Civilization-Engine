//! What a crossing carries (M5c slice AW; research 11-07 §1.3, §2.2, §5.5): its main members as
//! a simply supported beam over the clear span and a bearing, bending under their own weight and
//! the walkers at mid-span (`M = wL²/8 + PL/4`), against the bending strength of their wood over
//! their section as rot has left it (`Z ∝ d³`, so losing a tenth of the diameter leaves 73 % of
//! the strength) and as well as they were laid (their quality). Worked out whenever it is
//! needed, never saved. Only bending is checked: rot at the bearings, rolling, and the deck are
//! not yet modes of their own.

use std::f64::consts::PI;

use civ_land::crossings::Crossing;

use crate::params::BridgeDef;
use crate::structure::Timber;

/// A person with baggage, newtons (11-07 §2.2's test body).
pub const WALKER_N: f64 = 1000.0;
const GRAVITY: f64 = 9.81;

/// The strength of members over a `span_m` crossing of system `def` against what bends them
/// with `walkers` at mid-span: `members` of `diameter_cm`, laid with `quality`, with `loss` of
/// their section gone. Below 1 they give way.
#[allow(clippy::too_many_arguments)]
pub fn margin(
    def: &BridgeDef,
    timber: &Timber,
    span_m: f64,
    members: u32,
    diameter_cm: f64,
    quality: f64,
    loss: f64,
    walkers: u32,
) -> f64 {
    let d = diameter_cm / 100.0;
    let n = f64::from(members.max(1));
    let l = span_m + def.bearing_m;
    let own = n * PI * d * d / 4.0 * def.density_kg_m3 * GRAVITY;
    let bending = own * l * l / 8.0 + f64::from(walkers) * WALKER_N * l / 4.0;
    let left = (1.0 - loss).clamp(0.0, 1.0);
    let section = n * PI * d.powi(3) / 32.0 * left.powi(3);
    timber.bending_pa * section * quality / bending.max(1e-9)
}

/// [`margin`] for crossing `c`.
pub fn crossing_margin(c: &Crossing, def: &BridgeDef, timber: &Timber, walkers: u32) -> f64 {
    margin(
        def,
        timber,
        f64::from(c.span_m),
        u32::from(c.members),
        f64::from(c.diameter_cm),
        f64::from(c.quality),
        f64::from(c.loss),
        walkers,
    )
}

/// The diameter builders cut the members of a `span_m` crossing of system `def` to: the smallest,
/// in 5 cm steps from the least it allows, that gives its `margin` with one walker on new,
/// well-laid members; `None` when the span is outside its envelope or no diameter it allows
/// does.
pub fn size_members(def: &BridgeDef, timber: &Timber, span_m: f64) -> Option<f64> {
    if !(def.span_m.0..=def.span_m.1).contains(&span_m) {
        return None;
    }
    let mut d = def.diameter_cm.0;
    while d <= def.diameter_cm.1 + 1e-9 {
        if margin(def, timber, span_m, def.members, d, 1.0, 0.0, 1) >= def.margin {
            return Some(d);
        }
        d += 5.0;
    }
    None
}

/// The labour a `span_m` crossing of system `def` takes, hours: its members' whole length.
pub fn labour_h(def: &BridgeDef, span_m: f64) -> f64 {
    f64::from(def.members) * (span_m + 2.0 * def.bearing_m) * def.labour_h_per_m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_beam() -> BridgeDef {
        BridgeDef {
            id: "log".into(),
            name: "Log footbridge".into(),
            technique: None,
            good: 0,
            density_kg_m3: 760.0,
            span_m: (2.0, 8.0),
            members: 2,
            diameter_cm: (20.0, 50.0),
            bearing_m: 1.0,
            deck_factor: 0.6,
            labour_h_per_m: 8.0,
            loss_per_year: 0.05,
            margin: 3.0,
            fall_kills: 0.3,
        }
    }

    fn oak() -> Timber {
        Timber {
            bending_pa: 57e6,
            compression_pa: 24.5e6,
            stiffness_pa: 8.6e9,
            creep: 1.0,
            sustained: 0.6,
        }
    }

    #[test]
    fn two_new_logs_carry_a_walker_many_times_over_and_rot_takes_it_away_by_the_cube() {
        let (def, t) = (log_beam(), oak());
        let new = margin(&def, &t, 6.0, 2, 20.0, 1.0, 0.0, 1);
        assert!((15.0..25.0).contains(&new), "{new}");
        // 11-07 §5.5: a tenth of the diameter gone leaves 0.9³ of the strength.
        let worn = margin(&def, &t, 6.0, 2, 20.0, 1.0, 0.1, 1);
        assert!((worn / new - 0.729).abs() < 1e-9);
        // Doubling the span multiplies the bending by about four (more with the walker's share
        // smaller): the margin falls by more than half.
        assert!(margin(&def, &t, 12.0, 2, 20.0, 1.0, 0.0, 1) < new / 2.0);
        // Rot of about five-eighths of the section takes the smallest log to failure.
        assert!(margin(&def, &t, 6.0, 2, 20.0, 1.0, 0.65, 1) < 1.0);
        // A walker weighs on it: more of them, less margin.
        assert!(margin(&def, &t, 6.0, 2, 20.0, 1.0, 0.0, 4) < new);
    }

    #[test]
    fn builders_cut_the_smallest_log_that_gives_the_margin_and_only_within_the_envelope() {
        let (def, t) = (log_beam(), oak());
        assert_eq!(size_members(&def, &t, 6.0), Some(20.0));
        assert_eq!(size_members(&def, &t, 15.0), None, "no log beam spans 15 m");
        let strict = BridgeDef {
            margin: 18.0,
            ..log_beam()
        };
        assert_eq!(size_members(&strict, &t, 8.0), Some(25.0));
        assert!((labour_h(&def, 6.0) - 2.0 * 8.0 * 8.0).abs() < 1e-9);
    }
}
