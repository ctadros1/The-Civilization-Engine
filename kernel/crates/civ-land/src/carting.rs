//! Wheels on the land (M8b slice CP; ADR-0029 §2, §6, §7; research 10-07 §1.2): where a cart can
//! go, how hard it is to pull there, and the quickest way for it between two places. A cart goes
//! only where the ground takes its wheels: along a way that serves carts and is as wide as it is,
//! on a worn trail no narrower than a trail's tread, or across a crossing whose deck is as wide
//! as it is; never across open ground, through water or over a log laid for walkers.
//!
//! **The pull** a loaded cart needs is `M·g·(C + s)` (10-07 §1.2): its mass with its load, the
//! ground's rolling resistance `C` and the grade `s` it climbs, rise over run, negative going
//! down. It goes where that is no more than its puller can give, at the speed its puller's power
//! gives against it, never faster than they walk; down a grade steeper than they can hold it
//! back on, not at all. The grade is the ground's own between neighbouring cells, so a steep way
//! costs a cart what it costs, and a cart may take a longer way round a hill (10-07 §1.2: "agents
//! should sometimes choose a longer contouring route").

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use civ_world::{WATER_LAND, WATER_RIVER};

/// Standard gravity, metres a second squared.
pub const G: f64 = 9.81;

/// The tread a worn trail gives wheels, metres: the upper end of research 10-07 §2.4's footpath
/// tread of 0.6–1.2 m (a design prior). A cart no wider than this goes on a worn trail.
pub const TRAIL_TREAD_M: f64 = 1.2;

/// A loaded cart as the ground sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wheels {
    /// The cart's own mass and its load's, kilograms.
    pub mass_kg: f64,
    /// Its running width, metres.
    pub width_m: f64,
    /// Rolling resistance on a way that serves carts, and on a worn trail.
    pub way_c: f64,
    pub trail_c: f64,
    /// The most its puller can pull, newtons, and their sustained power, watts.
    pub pull_n: f64,
    pub power_w: f64,
    /// Its puller's walking speed, metres a second: the fastest it goes.
    pub walk_m_s: f64,
    /// The steepest grade down, rise over run, its puller can hold it back on.
    pub brake_grade: f64,
}

/// The pull, newtons, that a mass of `mass_kg` needs on ground of rolling resistance `c`
/// climbing grade `grade` (rise over run; negative going down): `M·g·(C + s)`.
pub fn pull_n(mass_kg: f64, c: f64, grade: f64) -> f64 {
    mass_kg * G * (c + grade)
}

impl Wheels {
    /// Its speed, metres a second, on ground of rolling resistance `c` climbing `grade`; `None`
    /// where its puller cannot pull it up, or hold it back down.
    pub fn speed(&self, c: f64, grade: f64) -> Option<f64> {
        if grade < -self.brake_grade {
            return None;
        }
        let f = pull_n(self.mass_kg, c, grade);
        if f > self.pull_n {
            return None;
        }
        Some(if f <= 0.0 {
            self.walk_m_s
        } else {
            (self.power_w / f).min(self.walk_m_s)
        })
    }

    /// The rolling resistance a cell gives these wheels, if they can go on it: `water` its
    /// water class, `deck_m` the width of an open crossing's deck over it, `way_m` the widest
    /// usable width of a way serving carts through it, and `trail` whether it is worn trail.
    pub fn resistance(
        &self,
        water: u8,
        deck_m: Option<f64>,
        way_m: Option<f64>,
        trail: bool,
    ) -> Option<f64> {
        match water {
            WATER_RIVER => deck_m.filter(|&d| d >= self.width_m).map(|_| self.way_c),
            WATER_LAND => {
                if way_m.is_some_and(|w| w >= self.width_m) {
                    Some(self.way_c)
                } else if trail && TRAIL_TREAD_M >= self.width_m {
                    Some(self.trail_c)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl crate::Land {
    /// The rolling resistance cell `cell` of `map` gives `wheels` today, if they can go on it:
    /// its water, the deck of an open crossing over it (its members side by side), the widest
    /// usable width of a way serving carts through it, and whether its wear is trail as last
    /// surveyed.
    pub fn cart_resistance(
        &self,
        map: &civ_world::WorldMap,
        cell: u32,
        wheels: &Wheels,
    ) -> Option<f64> {
        let water = *map.water.get(cell as usize)?;
        let deck_m = if water == WATER_RIVER {
            self.crossings.open_at(cell).map(|i| {
                let c = &self.crossings.list[i];
                f64::from(c.members) * f64::from(c.diameter_cm) / 100.0
            })
        } else {
            None
        };
        let way_m = self
            .ways
            .at(cell)
            .filter(|&k| self.ways.list[k].modes & crate::ways::CART != 0)
            .flat_map(|k| {
                self.ways.list[k]
                    .segments
                    .iter()
                    .filter(|s| s.cells.contains(&cell))
                    .map(|s| f64::from(s.usable_m))
            })
            .reduce(f64::max);
        let trail = self.wear.is_trail(cell as usize);
        wheels.resistance(water, deck_m, way_m, trail)
    }
}

/// The ground a cart routes over: elevations row by row, the map's width and height in cells,
/// and metres a cell.
#[derive(Clone, Copy, Debug)]
pub struct Terrain<'a> {
    pub elevation: &'a [f32],
    pub width: u32,
    pub height: u32,
    pub cell_m: f64,
}

/// The quickest way for a cart.
#[derive(Clone, Debug, PartialEq)]
pub struct CartRoute {
    /// Cells from the first to the last, in order.
    pub cells: Vec<u32>,
    /// Seconds on reaching each of `cells`, from 0.
    pub at_s: Vec<f64>,
    pub seconds: f64,
    pub metres: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Open {
    seconds: f64,
    cell: u32,
}

impl Eq for Open {}

impl Ord for Open {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .seconds
            .total_cmp(&self.seconds)
            .then(other.cell.cmp(&self.cell))
    }
}

impl PartialOrd for Open {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Terrain<'_> {
    /// The cells a cart may step to from `cell`, with the metres of each step: its eight
    /// neighbours, a diagonal only where both cells beside it take the wheels too, so a cart
    /// never squeezes between two it cannot go on.
    fn steps(&self, cell: u32, resistance: &dyn Fn(u32) -> Option<f64>) -> Vec<(u32, f64)> {
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let (x, y) = (i64::from(cell) % w, i64::from(cell) / w);
        let at = |dx: i64, dy: i64| {
            let (nx, ny) = (x + dx, y + dy);
            (nx >= 0 && ny >= 0 && nx < w && ny < h).then(|| (ny * w + nx) as u32)
        };
        let mut out = Vec::with_capacity(8);
        for (dx, dy) in [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ] {
            let Some(n) = at(dx, dy) else { continue };
            if resistance(n).is_none() {
                continue;
            }
            if dx != 0 && dy != 0 {
                let side = |c: Option<u32>| c.and_then(resistance).is_some();
                if !(side(at(dx, 0)) && side(at(0, dy))) {
                    continue;
                }
                out.push((n, self.cell_m * std::f64::consts::SQRT_2));
            } else {
                out.push((n, self.cell_m));
            }
        }
        out
    }

    /// Seconds for `wheels` to go one step of `metres` from `a` to `b`, on the mean of their
    /// rolling resistances (`ca`, `cb`), at the grade between them; `None` where it cannot.
    fn step_s(
        &self,
        wheels: &Wheels,
        (a, ca): (u32, f64),
        (b, cb): (u32, f64),
        metres: f64,
    ) -> Option<f64> {
        let za = f64::from(*self.elevation.get(a as usize)?);
        let zb = f64::from(*self.elevation.get(b as usize)?);
        let v = wheels.speed(0.5 * (ca + cb), (zb - za) / metres)?;
        (v > 0.0).then(|| metres / v)
    }

    /// The quickest way for `wheels` from any of `from` to any of `to`, on the cells
    /// `resistance` says take them (their rolling resistance), directed by the grade; `None`
    /// where none joins them. Ties go to the lower cell, so it is the same every run.
    pub fn route(
        &self,
        wheels: &Wheels,
        resistance: &dyn Fn(u32) -> Option<f64>,
        from: &[u32],
        to: &[u32],
    ) -> Option<CartRoute> {
        let mut best: HashMap<u32, f64> = HashMap::new();
        let mut came: HashMap<u32, (u32, f64)> = HashMap::new();
        let mut heap = BinaryHeap::new();
        for &c in from {
            if resistance(c).is_some() {
                best.insert(c, 0.0);
                heap.push(Open {
                    seconds: 0.0,
                    cell: c,
                });
            }
        }
        while let Some(Open { seconds, cell }) = heap.pop() {
            if best.get(&cell).is_some_and(|&b| seconds > b) {
                continue;
            }
            if to.contains(&cell) {
                return Some(Self::path(cell, &came, &best, seconds));
            }
            let Some(c_here) = resistance(cell) else {
                continue;
            };
            for (next, metres) in self.steps(cell, resistance) {
                let Some(c_next) = resistance(next) else {
                    continue;
                };
                let Some(cost) = self.step_s(wheels, (cell, c_here), (next, c_next), metres) else {
                    continue;
                };
                let s = seconds + cost;
                if best.get(&next).is_none_or(|&b| s < b) {
                    best.insert(next, s);
                    came.insert(next, (cell, metres));
                    heap.push(Open {
                        seconds: s,
                        cell: next,
                    });
                }
            }
        }
        None
    }

    fn path(
        end: u32,
        came: &HashMap<u32, (u32, f64)>,
        best: &HashMap<u32, f64>,
        seconds: f64,
    ) -> CartRoute {
        let mut cells = vec![end];
        let mut at_s = vec![best[&end]];
        let mut metres = 0.0;
        let mut at = end;
        while let Some(&(prev, m)) = came.get(&at) {
            metres += m;
            cells.push(prev);
            at_s.push(best[&prev]);
            at = prev;
        }
        cells.reverse();
        at_s.reverse();
        CartRoute {
            cells,
            at_s,
            seconds,
            metres,
        }
    }

    /// The seconds of [`Terrain::route`] found again by relaxing every step among `cells` until
    /// nothing changes (Bellman-Ford): the oracle a test checks routes against (ADR-0029 §7).
    #[doc(hidden)]
    pub fn oracle_seconds(
        &self,
        wheels: &Wheels,
        resistance: &dyn Fn(u32) -> Option<f64>,
        cells: &[u32],
        from: &[u32],
        to: &[u32],
    ) -> Option<f64> {
        let mut best: HashMap<u32, f64> = from
            .iter()
            .filter(|&&c| resistance(c).is_some())
            .map(|&c| (c, 0.0))
            .collect();
        loop {
            let mut changed = false;
            for &a in cells {
                let (Some(&sa), Some(ca)) = (best.get(&a), resistance(a)) else {
                    continue;
                };
                for (b, metres) in self.steps(a, resistance) {
                    let Some(cb) = resistance(b) else { continue };
                    let Some(cost) = self.step_s(wheels, (a, ca), (b, cb), metres) else {
                        continue;
                    };
                    if best.get(&b).is_none_or(|&sb| sa + cost < sb) {
                        best.insert(b, sa + cost);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        to.iter()
            .filter_map(|c| best.get(c).copied())
            .min_by(f64::total_cmp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A handcart of the transport brief's priors, loaded: 40 kg of cart and 150 kg of load.
    fn cart() -> Wheels {
        Wheels {
            mass_kg: 190.0,
            width_m: 0.9,
            way_c: 0.02,
            trail_c: 0.05,
            pull_n: 200.0,
            power_w: 75.0,
            walk_m_s: 1.3,
            brake_grade: 0.15,
        }
    }

    #[test]
    fn a_five_percent_climb_takes_three_and_a_half_times_the_level_pull_at_two_hundredths() {
        // Research 10-07 §1.2: at C = 0.02 a 5 % ascent raises the tractive force to about 3.5
        // times its level-ground value.
        let level = pull_n(190.0, 0.02, 0.0);
        let climb = pull_n(190.0, 0.02, 0.05);
        assert!((climb / level - 3.5).abs() < 1e-9, "{climb} / {level}");
        assert!((level - 190.0 * 9.81 * 0.02).abs() < 1e-9);
    }

    #[test]
    fn a_cart_goes_at_a_walk_on_the_level_slower_uphill_and_not_up_or_down_what_is_too_steep() {
        let w = cart();
        // Level way: 37 N against 75 W would be 2 m/s, past the 1.3 m/s walk.
        assert_eq!(w.speed(0.02, 0.0), Some(1.3));
        // 5 % up: 130 N, 0.58 m/s.
        let up = w.speed(0.02, 0.05).expect("it climbs 5 %");
        assert!(
            (up - 75.0 / pull_n(190.0, 0.02, 0.05)).abs() < 1e-12 && up < 0.6,
            "{up}"
        );
        // A trail's rougher ground is slower up the same grade.
        assert!(w.speed(0.05, 0.05).expect("and on a trail") < up);
        // 10 % up needs 224 N, past the puller's 200: it cannot go.
        assert_eq!(w.speed(0.02, 0.10), None);
        // Down: at a walk, until it is too steep to hold back.
        assert_eq!(w.speed(0.02, -0.10), Some(1.3));
        assert_eq!(w.speed(0.02, -0.20), None);
    }

    #[test]
    fn a_cart_goes_on_ways_and_trails_and_decks_wide_enough_never_on_open_ground_or_a_log() {
        let w = cart();
        assert_eq!(w.resistance(WATER_LAND, None, Some(2.5), false), Some(0.02));
        assert_eq!(w.resistance(WATER_LAND, None, Some(0.8), true), Some(0.05));
        assert_eq!(w.resistance(WATER_LAND, None, Some(0.8), false), None);
        assert_eq!(w.resistance(WATER_LAND, None, None, true), Some(0.05));
        assert_eq!(w.resistance(WATER_LAND, None, None, false), None);
        // A log footbridge of two 20 cm logs is 0.4 m across: never a cart's.
        assert_eq!(w.resistance(WATER_RIVER, Some(0.4), None, false), None);
        assert_eq!(w.resistance(WATER_RIVER, None, None, true), None);
        assert_eq!(
            w.resistance(WATER_RIVER, Some(1.5), None, false),
            Some(0.02)
        );
        assert_eq!(
            w.resistance(civ_world::WATER_LAKE, Some(3.0), Some(3.0), true),
            None
        );
        // A cart wider than a trail's tread keeps to ways.
        let wide = Wheels { width_m: 1.5, ..w };
        assert_eq!(wide.resistance(WATER_LAND, None, None, true), None);
    }

    /// A 20 × 12 map of 8 m cells, flat but for a ridge across columns 8–10 rising 1.2 m a cell
    /// (15 %) on the way up and falling as steeply, with a pass at rows 0–1 kept level. Every
    /// cell is worn trail.
    fn ridge() -> (Vec<f32>, u32, u32) {
        let (w, h) = (20u32, 12u32);
        let mut z = vec![0.0f32; (w * h) as usize];
        for y in 2..h {
            for x in 0..w {
                let rise = match x {
                    8 | 10 => 1.2,
                    9 => 2.4,
                    _ => 0.0,
                };
                z[(y * w + x) as usize] = rise;
            }
        }
        (z, w, h)
    }

    #[test]
    fn a_cart_goes_round_a_ridge_too_steep_to_climb_where_a_light_one_goes_over() {
        let (z, w, h) = ridge();
        let terrain = Terrain {
            elevation: &z,
            width: w,
            height: h,
            cell_m: 8.0,
        };
        let wheels = cart();
        let all = |c: u32| (c < w * h).then_some(wheels.trail_c);
        let (from, to) = (6 * w + 2, 6 * w + 17);
        let route = terrain
            .route(&wheels, &all, &[from], &[to])
            .expect("a way round by the pass");
        // Over the ridge is 15 cells of 8 m; round by the pass is longer, and every cell it
        // takes lies on the level rows or off the ridge.
        assert!(route.metres > 15.0 * 8.0 + 1.0, "{}", route.metres);
        assert!(route.cells.iter().all(|&c| {
            let (x, y) = (c % w, c / w);
            y < 2 || !(8..=10).contains(&x)
        }));
        assert_eq!(route.cells.first(), Some(&from));
        assert_eq!(route.cells.last(), Some(&to));
        // Times only grow along the route.
        assert!(route.at_s.windows(2).all(|p| p[1] >= p[0]));
        let cells: Vec<u32> = (0..w * h).collect();
        let oracle = terrain.oracle_seconds(&wheels, &all, &cells, &[from], &[to]);
        assert!((oracle.expect("the oracle finds it") - route.seconds).abs() < 1e-6);
        // Unladen it goes over.
        let light = Wheels {
            mass_kg: 40.0,
            ..wheels
        };
        let over = terrain.route(&light, &all, &[from], &[to]).expect("over");
        assert!(over.metres < route.metres);
    }

    #[test]
    fn up_a_slope_is_slower_than_down_it() {
        // A straight way 30 cells long climbing 0.24 m a cell (3 %).
        let (w, h) = (30u32, 1u32);
        let z: Vec<f32> = (0..w).map(|x| 0.24 * x as f32).collect();
        let terrain = Terrain {
            elevation: &z,
            width: w,
            height: h,
            cell_m: 8.0,
        };
        let wheels = cart();
        let all = |c: u32| (c < w).then_some(wheels.way_c);
        let up = terrain.route(&wheels, &all, &[0], &[w - 1]).expect("up");
        let down = terrain.route(&wheels, &all, &[w - 1], &[0]).expect("down");
        assert!((up.metres - down.metres).abs() < 1e-9);
        assert!(
            up.seconds > down.seconds * 1.2,
            "{} against {}",
            up.seconds,
            down.seconds
        );
        assert!((down.seconds - down.metres / 1.3).abs() < 1e-6);
    }

    #[test]
    fn routes_agree_with_the_oracle_on_broken_ground() {
        // A 24 × 24 map with hills and gaps drawn from a fixed sequence.
        let (w, h) = (24u32, 24u32);
        let mut s: u64 = 0x2545_f491_4f6c_dd1d;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let z: Vec<f32> = (0..w * h).map(|_| (next() % 100) as f32 * 0.012).collect();
        let open: Vec<bool> = (0..w * h).map(|_| next() % 5 != 0).collect();
        let terrain = Terrain {
            elevation: &z,
            width: w,
            height: h,
            cell_m: 8.0,
        };
        let wheels = cart();
        let res = |c: u32| {
            open.get(c as usize)
                .copied()
                .unwrap_or(false)
                .then_some(0.03)
        };
        let cells: Vec<u32> = (0..w * h).collect();
        let mut agreed = 0;
        for k in 0..30 {
            let a = (next() % u64::from(w * h)) as u32;
            let b = (next() % u64::from(w * h)) as u32;
            let r = terrain.route(&wheels, &res, &[a], &[b]).map(|r| r.seconds);
            let o = terrain.oracle_seconds(&wheels, &res, &cells, &[a], &[b]);
            match (r, o) {
                (Some(r), Some(o)) => {
                    assert!((r - o).abs() < 1e-6, "pair {k}: {r} against {o}");
                    agreed += 1;
                }
                (None, None) => {}
                (r, o) => panic!("pair {k}: route {r:?}, oracle {o:?}"),
            }
        }
        assert!(agreed >= 10, "only {agreed} pairs were joined");
    }
}
