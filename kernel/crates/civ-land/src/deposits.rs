//! Deposits as bodies in the ground (ADR-0010 §1; M3b slice Q): clay, stone and flint placed once
//! per world from the seed by the land profile's rules on the generated terrain, each with a
//! finite inventory that never renews. Nothing here decides who finds or digs one.
//!
//! The research gives no figures for where bodies lie by slope, height above drainage or distance
//! to a channel, nor for their sizes or how many a square kilometre holds (research 03-05 §1.1,
//! §1.2 is qualitative: clay in floodplains and old channels, often buried; stone exposed by
//! erosion; flint in particular horizons, weathered into surface spreads), so every rule is a
//! tuning value of the land profile.

use civ_core::{PermanentId, Rng64};
use civ_world::{WATER_LAND, WorldMap, terrain};

/// Keyed-randomness purpose of where deposits lie.
const PURPOSE_DEPOSITS: u64 = 0x6465_706f_7369_7473; // "deposits"

/// The most bodies one rule places in a world, however large.
pub const MOST_PER_RULE: usize = 256;

/// Where the land profile lets one kind of body lie, and how large it is (all tuning values).
#[derive(Clone, Debug, PartialEq)]
pub struct DepositRule {
    /// The good it yields, by index in the goods.
    pub good: usize,
    /// Mean slope of the cell it lies in (rise over run), least and most.
    pub slope: (f64, f64),
    /// Height above the nearest channel, metres, least and most.
    pub hand_m: (f64, f64),
    /// Bodies expected on each square kilometre of land that qualifies.
    pub per_km2: f64,
    /// Radius of the body, metres.
    pub radius_m: (f64, f64),
    /// Depth of what covers it, metres; a body under none shows at the surface.
    pub top_m: (f64, f64),
    /// Thickness of the body, metres.
    pub thickness_m: (f64, f64),
    /// Quality, 0 to 1 (how much of what is dug is fit for use).
    pub quality: (f64, f64),
    /// The share of bodies under cover that still show somewhere (a stream cut, a slip).
    pub exposed_share: f64,
    /// Kilograms of the good in a cubic metre of the body.
    pub density_kg_m3: f64,
}

/// A body as placed, before it is given a permanent id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// The good it yields, by index in the goods.
    pub good: u16,
    /// Its centre, centimetres from the map's north-west corner.
    pub at_cm: (i64, i64),
    /// Its radius, centimetres.
    pub radius_cm: i32,
    /// The depth of what covers it, centimetres.
    pub top_cm: i32,
    /// Its thickness, centimetres.
    pub thickness_cm: i32,
    /// How much of what is dug is fit for use, 0 to 1.
    pub quality: f32,
    /// Whether it shows at the surface.
    pub exposed: bool,
    /// Kilograms of the good in it when the world began.
    pub initial_kg: f64,
}

/// A deposit: a body with a permanent id and what has been taken from it. What is taken plus
/// what is left is always what it began with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Deposit {
    /// Permanent id.
    pub id: PermanentId,
    /// The body.
    pub body: Body,
    /// Kilograms taken from it so far.
    pub taken_kg: f64,
}

impl Deposit {
    /// Kilograms left in it.
    pub fn left_kg(&self) -> f64 {
        (self.body.initial_kg - self.taken_kg).max(0.0)
    }
}

/// Every body the rules place on `map` for world `seed`, rule by rule: a count drawn about each
/// rule's `per_km2` times its qualifying land (at most [`MOST_PER_RULE`]), each body at a
/// qualifying land cell drawn at random, its sizes drawn within the rule's ranges. The same seed,
/// map and rules give the same bodies.
pub fn place(map: &WorldMap, rules: &[DepositRule], channel_area_m2: f32, seed: u64) -> Vec<Body> {
    if rules.is_empty() {
        return Vec::new();
    }
    let slope = terrain::slopes(map);
    let hand = terrain::height_above_drainage(map, channel_area_m2);
    let cell_m = f64::from(map.cell_size_m);
    let mut out = Vec::new();
    for (r, rule) in rules.iter().enumerate() {
        let fits = |(lo, hi): (f64, f64), v: f32| f64::from(v) >= lo && f64::from(v) <= hi;
        let cells: Vec<usize> = (0..map.water.len())
            .filter(|&i| {
                map.water[i] == WATER_LAND
                    && fits(rule.slope, slope[i])
                    && fits(rule.hand_m, hand[i])
            })
            .collect();
        if cells.is_empty() {
            continue;
        }
        let km2 = cells.len() as f64 * cell_m * cell_m / 1e6;
        let mut rng = Rng64::from_key(&[seed, PURPOSE_DEPOSITS, r as u64]);
        // A count about the expected one: its whole part, and one more by its fraction.
        let expected = (rule.per_km2.max(0.0) * km2).min(MOST_PER_RULE as f64);
        let mut count = expected.floor() as usize;
        if rng.next_f64() < expected.fract() {
            count += 1;
        }
        let draw = |rng: &mut Rng64, (lo, hi): (f64, f64)| lo + (hi - lo).max(0.0) * rng.next_f64();
        for _ in 0..count.min(MOST_PER_RULE) {
            let i = cells[((rng.next_f64() * cells.len() as f64) as usize).min(cells.len() - 1)];
            let (x, y) = (i % map.width as usize, i / map.width as usize);
            let at_cm = (
                ((x as f64 + rng.next_f64()) * cell_m * 100.0) as i64,
                ((y as f64 + rng.next_f64()) * cell_m * 100.0) as i64,
            );
            let radius = draw(&mut rng, rule.radius_m);
            let top = draw(&mut rng, rule.top_m);
            let thickness = draw(&mut rng, rule.thickness_m);
            let quality = draw(&mut rng, rule.quality).clamp(0.0, 1.0);
            let shows = rng.next_f64() < rule.exposed_share;
            out.push(Body {
                good: u16::try_from(rule.good).unwrap_or(u16::MAX),
                at_cm,
                radius_cm: (radius * 100.0).round() as i32,
                top_cm: (top * 100.0).round() as i32,
                thickness_cm: (thickness * 100.0).round() as i32,
                quality: quality as f32,
                exposed: top < 0.005 || shows,
                initial_kg: std::f64::consts::PI
                    * radius
                    * radius
                    * thickness
                    * rule.density_kg_m3.max(0.0),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(per_km2: f64) -> DepositRule {
        DepositRule {
            good: 3,
            slope: (0.0, 10.0),
            hand_m: (0.0, 1e6),
            per_km2,
            radius_m: (5.0, 20.0),
            top_m: (0.0, 1.0),
            thickness_m: (0.5, 2.0),
            quality: (0.4, 0.9),
            exposed_share: 0.3,
            density_kg_m3: 1_800.0,
        }
    }

    #[test]
    fn the_same_seed_places_the_same_bodies_on_dry_land_within_the_rule() {
        let map = crate::tests::map();
        let rules = [rule(400.0)];
        let bodies = place(&map, &rules, 1e6, 7);
        assert!(!bodies.is_empty());
        assert_eq!(bodies, place(&map, &rules, 1e6, 7));
        assert_ne!(bodies, place(&map, &rules, 1e6, 8));
        let cell_cm = f64::from(map.cell_size_m) * 100.0;
        for b in &bodies {
            let (x, y) = (
                (b.at_cm.0 as f64 / cell_cm) as usize,
                (b.at_cm.1 as f64 / cell_cm) as usize,
            );
            assert_eq!(map.water[y * map.width as usize + x], WATER_LAND);
            assert!((500..=2_000).contains(&b.radius_cm), "{b:?}");
            assert!((0.4..=0.9).contains(&b.quality), "{b:?}");
            // Its inventory is its volume times the rule's density.
            let r = f64::from(b.radius_cm) / 100.0;
            let t = f64::from(b.thickness_cm) / 100.0;
            let v = std::f64::consts::PI * r * r * t * 1_800.0;
            assert!((b.initial_kg - v).abs() < 0.02 * v, "{b:?}");
            if b.top_cm == 0 {
                assert!(b.exposed);
            }
        }
        // None where no land qualifies, and none without rules.
        let mut steep = rule(400.0);
        steep.slope = (50.0, 60.0);
        assert!(place(&map, &[steep], 1e6, 7).is_empty());
        assert!(place(&map, &[], 1e6, 7).is_empty());
    }

    #[test]
    fn what_is_taken_and_what_is_left_make_what_it_began_with() {
        let map = crate::tests::map();
        let body = place(&map, &[rule(400.0)], 1e6, 7)[0];
        let mut d = Deposit {
            id: PermanentId::from_raw(1).expect("nonzero"),
            body,
            taken_kg: 0.0,
        };
        d.taken_kg = body.initial_kg / 4.0;
        assert!((d.left_kg() + d.taken_kg - body.initial_kg).abs() < 1e-6);
        d.taken_kg = body.initial_kg * 2.0;
        assert_eq!(d.left_kg(), 0.0);
    }
}
