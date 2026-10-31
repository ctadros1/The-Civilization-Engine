//! Climbing (ADR-0042 §1): a household's members living above the ground climb their storeys on
//! every trip out and back, and that time belongs in the hours it reckons. A home's design counts
//! it for the storeys its own members would sleep on, filled from the ground up; a lodger's
//! weighing of a room counts it for the storey the room's floor lies on, the highest storey of the
//! home with floor to spare (lets are filled from the top down). Pure arithmetic: nothing here
//! decides where anyone lives.

use civ_grammar::{Expansion, SpaceUse};

/// What a people profile asks of climbing (ADR-0042 §1; the profile's `[climbing]`, content API
/// 214). Design priors (research 11-14 §2.5: 10–20 s a storey walking up).
#[derive(Clone, Debug, PartialEq)]
pub struct ClimbParams {
    /// Seconds a person takes to climb a storey by a stair.
    pub up_s_per_storey: f64,
    /// Seconds a person takes to come down a storey by a stair.
    pub down_s_per_storey: f64,
}

impl Default for ClimbParams {
    fn default() -> ClimbParams {
        ClimbParams {
            up_s_per_storey: 15.0,
            down_s_per_storey: 8.0,
        }
    }
}

impl ClimbParams {
    /// Seconds a trip out and back from storey `storey` (0 the first) takes by the stair.
    pub fn stair_trip_s(&self, storey: usize) -> f64 {
        storey as f64 * (self.up_s_per_storey + self.down_s_per_storey).max(0.0)
    }
}

/// Living (sleeping) floor of each storey of expansion `e`, square metres, the first storey's
/// first; lofts and the roof hold none.
pub fn living_by_storey(e: &Expansion) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for s in e.spaces.iter().filter(|s| s.use_ == SpaceUse::Living) {
        let Some(t) = s.level.storey_index() else {
            continue;
        };
        if out.len() <= t {
            out.resize(t + 1, 0.0);
        }
        out[t] += s.area_m2.max(0.0);
    }
    out
}

/// The storey each of `members` sleeps on in a home whose living floor by storey is `by_storey`:
/// the household's own floor (`base_m2`) first, then each member's `per_m2`, filled from the
/// ground up. Members beyond its floor sleep on its top storey with living floor.
pub fn members_storeys(by_storey: &[f64], base_m2: f64, per_m2: f64, members: usize) -> Vec<usize> {
    let top = by_storey.iter().rposition(|&a| a > 0.0).unwrap_or(0);
    (0..members)
        .map(|i| {
            // The member's floor is reckoned at its middle.
            let at = base_m2.max(0.0) + per_m2.max(0.0) * (i as f64 + 0.5);
            let mut below = 0.0;
            for (t, &a) in by_storey.iter().enumerate() {
                below += a.max(0.0);
                if at < below - 1e-9 {
                    return t;
                }
            }
            top
        })
        .collect()
}

/// The storey a let of floor lies on in a home whose living floor by storey is `by_storey`: the
/// highest storey with floor to spare once its household's own `own_m2` is taken from the ground
/// up and `let_m2` already let from the top down (ADR-0042 §1). 0 where no storey has floor to
/// spare, or the home has one storey.
pub fn let_storey(by_storey: &[f64], own_m2: f64, let_m2: f64) -> usize {
    let total: f64 = by_storey.iter().map(|a| a.max(0.0)).sum();
    // Free floor lies between `own_m2` from the bottom and `total - let_m2` from the bottom.
    let free_top = total - let_m2.max(0.0);
    if free_top <= own_m2.max(0.0) + 1e-9 {
        return 0;
    }
    let mut below = 0.0;
    for (t, &a) in by_storey.iter().enumerate() {
        below += a.max(0.0);
        if free_top <= below + 1e-9 {
            return t;
        }
    }
    by_storey.len().saturating_sub(1)
}

/// Hours a year a household climbs when its members sleep on `storeys` and make `trips_year`
/// trips out and back between them, a trip from storey `t` taking `trip_s(t)` seconds: each
/// member's share of the trips times its own storey's climb.
pub fn climb_h_year(storeys: &[usize], trips_year: f64, trip_s: impl Fn(usize) -> f64) -> f64 {
    if storeys.is_empty() || trips_year <= 0.0 || !trips_year.is_finite() {
        return 0.0;
    }
    let per = trips_year / storeys.len() as f64;
    storeys
        .iter()
        .filter(|&&t| t > 0)
        .map(|&t| per * trip_s(t).max(0.0))
        .sum::<f64>()
        / 3600.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_fill_from_the_ground_and_lets_from_the_top() {
        // Two storeys of 40 m² each; a household's own 10 m² and 5 m² a sleeper.
        let by = [40.0, 40.0];
        // Six members: 10 + 6 × 5 = 40 m², all on the first storey.
        assert_eq!(members_storeys(&by, 10.0, 5.0, 6), vec![0; 6]);
        // Eight: the seventh's and eighth's floor lie above the first storey's 40 m².
        assert_eq!(
            members_storeys(&by, 10.0, 5.0, 8),
            vec![0, 0, 0, 0, 0, 0, 1, 1]
        );
        // One storey: nobody climbs, however many.
        assert_eq!(members_storeys(&[40.0], 10.0, 5.0, 12), vec![0; 12]);
        // A let lies on the highest storey with floor to spare.
        assert_eq!(let_storey(&by, 40.0, 0.0), 1);
        assert_eq!(let_storey(&by, 20.0, 40.0), 0);
        assert_eq!(let_storey(&by, 20.0, 30.0), 1);
        assert_eq!(let_storey(&[40.0], 20.0, 0.0), 0);
        assert_eq!(let_storey(&by, 80.0, 0.0), 0);
    }

    #[test]
    fn climbing_is_each_members_share_of_the_trips_times_its_storeys() {
        let p = ClimbParams::default();
        assert_eq!(p.stair_trip_s(0), 0.0);
        assert_eq!(p.stair_trip_s(2), 46.0);
        // Four members, two on the second storey, 1,000 trips a year between them: 500 trips of
        // 23 s.
        let h = climb_h_year(&[0, 0, 1, 1], 1000.0, |t| p.stair_trip_s(t));
        assert!((h - 500.0 * 23.0 / 3600.0).abs() < 1e-9);
        assert_eq!(climb_h_year(&[0, 0], 1000.0, |t| p.stair_trip_s(t)), 0.0);
        assert_eq!(climb_h_year(&[], 1000.0, |t| p.stair_trip_s(t)), 0.0);
    }
}
