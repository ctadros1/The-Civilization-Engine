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

/// The longest a crossing is reckoned to last, years: beyond a century nobody counts (when its
/// system rots not at all).
pub const LONGEST_LIFE_YEARS: f64 = 100.0;

/// The years a `span_m` crossing of system `def`, its members of `diameter_cm` laid at
/// `quality`, lasts before rot leaves it unable to carry one walker (M5c slice AW, step two):
/// its margin falls with the cube of what rot leaves of the section, so it gives way once
/// `(1 − loss)³` times its margin when new is below 1. None when it could not carry one new.
pub fn life_years(
    def: &BridgeDef,
    timber: &Timber,
    span_m: f64,
    diameter_cm: f64,
    quality: f64,
) -> f64 {
    let new = margin(
        def,
        timber,
        span_m,
        def.members,
        diameter_cm,
        quality,
        0.0,
        1,
    );
    if new <= 1.0 {
        return 0.0;
    }
    let loss = 1.0 - new.powf(-1.0 / 3.0);
    if def.loss_per_year <= 0.0 {
        return LONGEST_LIFE_YEARS;
    }
    (loss / def.loss_per_year).min(LONGEST_LIFE_YEARS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_beam() -> BridgeDef {
        BridgeDef {
            id: "log".into(),
            name: "Log footbridge".into(),
            technique: None,
            skill: None,
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
            crew: 5,
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

    #[test]
    fn a_crossing_lasts_until_rot_leaves_it_unable_to_carry_a_walker() {
        let (def, t) = (log_beam(), oak());
        let life = life_years(&def, &t, 6.0, 20.0, 0.9);
        // The loss that life comes to leaves the logs just able to carry one walker.
        let loss = life * def.loss_per_year;
        let left = margin(&def, &t, 6.0, 2, 20.0, 0.9, loss, 1);
        assert!((left - 1.0).abs() < 1e-6, "{left}");
        // 11-07 §2.4: untreated log bridges last 10-20 years.
        assert!((10.0..20.0).contains(&life), "{life}");
        // Worse-laid logs and longer spans last less; logs that never rot, a century at most.
        assert!(life_years(&def, &t, 6.0, 20.0, 0.5) < life);
        assert!(life_years(&def, &t, 8.0, 20.0, 0.9) < life);
        let sound = BridgeDef {
            loss_per_year: 0.0,
            ..log_beam()
        };
        assert_eq!(life_years(&sound, &t, 6.0, 20.0, 0.9), LONGEST_LIFE_YEARS);
    }
}
