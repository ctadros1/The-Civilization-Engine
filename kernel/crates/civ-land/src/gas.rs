//! Gas mains and connections (M10b slice DY, third part; ADR-0034 §8; research 12-09 §1.3–§1.5,
//! §5.2): a main carries a works' gas along ways to the buildings connected to it, its capacity a
//! rate; each evening its district is checked, the works' gas and the main's capacity against the
//! burners to be lit, and each connection is served or interrupted by a cause kept apart. Not a
//! flow solver: a capacity check per district (brief §2.1).
//!
//! A main is laid by work along ways from its works (the third part's second step): its route
//! keeps to the cells of ways whose stretches' keepers give leave ([`plan_main`]), and it carries
//! gas only once its whole length is laid, a metre of pipe at a time ([`MainParams`]).

use std::collections::VecDeque;

use civ_core::PermanentId;

use crate::ways::{Keeper, Ways};

/// What laying a main takes (M10b slice DY, third part, second step; research 12-09 §1.3: mains
/// are assets built of pipe, digging and restoring the street). Authored in content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainParams {
    /// The good its pipe is made of (an index into the catalog's goods).
    pub good: usize,
    /// Kilograms of that good a metre of main takes.
    pub good_kg_m: f64,
    /// Hours of a capable adult's work to dig, lay, joint and fill in a metre of main.
    pub lay_h_m: f64,
    /// The most gas a main of its bore carries an hour, kg.
    pub capacity_kg_h: f32,
}

/// Why a connection's burners did or did not light on an evening (research 12-09 §1.5:
/// interruptions keep four causes apart).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cause {
    /// Lit.
    #[default]
    Served,
    /// The works had too little gas for it.
    NoGas,
    /// The main could not carry its burners as well as those ahead of it.
    Overloaded,
    /// The main is broken.
    Broken,
    /// Its account is cut off.
    CutOff,
}

impl Cause {
    /// As saved.
    pub fn code(self) -> u8 {
        match self {
            Cause::Served => 0,
            Cause::NoGas => 1,
            Cause::Overloaded => 2,
            Cause::Broken => 3,
            Cause::CutOff => 4,
        }
    }

    /// From its saved code; an unknown one reads as served.
    pub fn from_code(code: u8) -> Cause {
        match code {
            1 => Cause::NoGas,
            2 => Cause::Overloaded,
            3 => Cause::Broken,
            4 => Cause::CutOff,
            _ => Cause::Served,
        }
    }
}

/// A main: the household whose works feeds it, the cells it runs along from the works, the most
/// gas it carries an hour, kg, whether it is broken, and its length and how much of it is laid,
/// metres.
#[derive(Clone, Debug, PartialEq)]
pub struct Main {
    pub id: PermanentId,
    pub works: PermanentId,
    pub cells: Vec<u32>,
    pub capacity_kg_h: f32,
    pub broken: bool,
    pub length_m: f32,
    pub laid_m: f32,
}

impl Main {
    /// Whether its whole length is laid, so that it carries gas.
    pub fn whole(&self) -> bool {
        self.laid_m + 1e-3 >= self.length_m
    }
}

/// The route of a main from cell `from` (its works) to cell `to` (a building) on a map `width`
/// cells wide: through cells of ways whose stretch's keeper `may` give leave, each step to one of
/// the eight neighbours, cells in order from `from` to `to` (both kept, as the pipe leaves the
/// works and enters the building); the fewest cells, ties broken by the order the neighbours are
/// tried, so the same land plans the same route. `None` where no such route joins them.
pub fn plan_main(
    ways: &Ways,
    width: u32,
    from: u32,
    to: u32,
    may: impl Fn(Keeper) -> bool,
) -> Option<Vec<u32>> {
    let mut open: Vec<u32> = ways
        .list
        .iter()
        .flat_map(|w| w.segments.iter())
        .filter(|s| may(s.keeper))
        .flat_map(|s| s.cells.iter().copied())
        .collect();
    open.push(from);
    open.push(to);
    open.sort_unstable();
    open.dedup();
    let w = i64::from(width.max(1));
    let mut came: Vec<(u32, u32)> = vec![(from, from)];
    let mut queue = VecDeque::from([from]);
    while let Some(c) = queue.pop_front() {
        if c == to {
            let mut path = vec![to];
            let mut at = to;
            while at != from {
                at = came[came.binary_search_by_key(&at, |&(k, _)| k).ok()?].1;
                path.push(at);
            }
            path.reverse();
            return Some(path);
        }
        let (x, y) = (i64::from(c) % w, i64::from(c) / w);
        for (dx, dy) in [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, 1),
            (-1, -1),
            (1, -1),
        ] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w {
                continue;
            }
            let Ok(n) = u32::try_from(ny * w + nx) else {
                continue;
            };
            if open.binary_search(&n).is_err() {
                continue;
            }
            if let Err(i) = came.binary_search_by_key(&n, |&(k, _)| k) {
                came.insert(i, (n, c));
                queue.push_back(n);
            }
        }
    }
    None
}

/// A building's burners on a main, the household that keeps it, whether its account is cut off,
/// and its last evening: the day checked (-1 never), the minutes its burners were lit from and
/// until, and why they did or did not light.
#[derive(Clone, Debug, PartialEq)]
pub struct Connection {
    pub building: PermanentId,
    pub household: PermanentId,
    pub main: PermanentId,
    pub burners: u8,
    pub cut_off: bool,
    pub evening: i64,
    pub lit_from: i64,
    pub lit_until: i64,
    pub cause: Cause,
}

impl Connection {
    /// A connection of `burners` on main `main`, never yet lit.
    pub fn new(
        building: PermanentId,
        household: PermanentId,
        main: PermanentId,
        burners: u8,
    ) -> Connection {
        Connection {
            building,
            household,
            main,
            burners,
            cut_off: false,
            evening: -1,
            lit_from: -1,
            lit_until: -1,
            cause: Cause::Served,
        }
    }
}

/// Every main and connection, in the order made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gas {
    pub mains: Vec<Main>,
    pub connections: Vec<Connection>,
}

impl Gas {
    /// Whether household `household`'s burners are lit at minute `minute`.
    pub fn lit(&self, household: PermanentId, minute: i64) -> bool {
        self.lit_until(household, minute).is_some()
    }

    /// The minute household `household`'s burners lit at minute `minute` go out, if they are lit.
    pub fn lit_until(&self, household: PermanentId, minute: i64) -> Option<i64> {
        self.connections
            .iter()
            .filter(|c| {
                c.household == household
                    && c.cause == Cause::Served
                    && c.lit_from <= minute
                    && minute < c.lit_until
            })
            .map(|c| c.lit_until)
            .max()
    }

    /// The evening of day `day` on main `main`, lit from minute `from` for `hours`, with
    /// `held_kg` of gas at its works and burners drawing `kg_h` an hour each: its connections in
    /// the order made are served while the main carries their rate and the works' gas lasts;
    /// one cut off, all on a broken main, and those beyond the capacity or the gas are not, each
    /// by its cause. Returns the gas drawn, kg.
    pub fn evening(
        &mut self,
        main: PermanentId,
        day: i64,
        from: i64,
        hours: f64,
        held_kg: f64,
        kg_h: f64,
    ) -> f64 {
        // A main not yet laid whole carries nothing, and its connections are not checked.
        let Some(m) = self.mains.iter().find(|m| m.id == main && m.whole()) else {
            return 0.0;
        };
        let (broken, capacity) = (m.broken, f64::from(m.capacity_kg_h));
        let until = from + (hours * 60.0).round() as i64;
        let (mut rate, mut drawn) = (0.0f64, 0.0f64);
        for c in self.connections.iter_mut().filter(|c| c.main == main) {
            let want_rate = f64::from(c.burners) * kg_h;
            let want = want_rate * hours;
            c.cause = if c.cut_off {
                Cause::CutOff
            } else if broken {
                Cause::Broken
            } else if rate + want_rate > capacity + 1e-9 {
                Cause::Overloaded
            } else if drawn + want > held_kg + 1e-9 {
                Cause::NoGas
            } else {
                rate += want_rate;
                drawn += want;
                Cause::Served
            };
            c.evening = day;
            (c.lit_from, c.lit_until) = if c.cause == Cause::Served {
                (from, until)
            } else {
                (-1, -1)
            };
        }
        drawn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("id")
    }

    fn gas(capacity_kg_h: f32, burners: &[u8]) -> Gas {
        Gas {
            mains: vec![Main {
                id: id(1),
                works: id(9),
                cells: vec![0, 1, 2],
                capacity_kg_h,
                broken: false,
                length_m: 30.0,
                laid_m: 30.0,
            }],
            connections: burners
                .iter()
                .enumerate()
                .map(|(i, &b)| Connection::new(id(100 + i as u64), id(200 + i as u64), id(1), b))
                .collect(),
        }
    }

    #[test]
    fn a_main_keeps_to_ways_whose_keepers_give_leave() {
        use crate::ways::{Rank, Segment, Surface, WALK, Way};
        let way = |n: u64, cells: Vec<u32>, keeper: Keeper| Way {
            id: id(n),
            begun: 0,
            segments: vec![Segment {
                cells,
                corridor_m: 4.0,
                built_m: 4.0,
                usable_m: 4.0,
                surface: Surface::Cleared,
                keeper,
                kept: 0,
            }],
            modes: WALK,
            trips: 0.0,
            trips_day: 0,
            rank: Rank::Lane,
            track: None,
        };
        // A row of a map ten cells wide: the works at cell 0, a building at cell 9, a way kept by
        // nobody over cells 1-5 and one kept by household 7 over cells 6-8.
        let mut ways = Ways::default();
        ways.add(way(1, vec![1, 2, 3, 4, 5], Keeper::Nobody));
        ways.add(way(2, vec![6, 7, 8], Keeper::Household(id(7))));
        let leave = |granted: u64| {
            move |k: Keeper| match k {
                Keeper::Nobody => true,
                Keeper::Household(h) => h == id(granted),
                Keeper::Polity(_) => false,
            }
        };
        assert_eq!(
            plan_main(&ways, 10, 0, 9, leave(7)),
            Some((0..10).collect::<Vec<u32>>())
        );
        // Household 7 not giving leave, no route keeps to ways.
        assert_eq!(plan_main(&ways, 10, 0, 9, leave(8)), None);
    }

    #[test]
    fn an_evening_serves_connections_while_the_main_and_the_works_gas_last_and_names_why_not() {
        // Two burners at 0.071 kg an hour for 4 hours want 0.568 kg; the main carries 0.3 kg an
        // hour, four burners' worth.
        let mut g = gas(0.3, &[2, 2, 2]);
        let drawn = g.evening(id(1), 5, 1_200, 4.0, 10.0, 0.071);
        assert!((drawn - 2.0 * 0.568).abs() < 1e-9, "{drawn}");
        let causes: Vec<Cause> = g.connections.iter().map(|c| c.cause).collect();
        assert_eq!(causes, [Cause::Served, Cause::Served, Cause::Overloaded]);
        assert!(g.lit(id(200), 1_200) && g.lit(id(200), 1_439) && !g.lit(id(200), 1_440));
        assert_eq!(g.lit_until(id(201), 1_300), Some(1_440));
        assert!(!g.lit(id(202), 1_300) && !g.lit(id(200), 1_199));
        // Gas for one connection only: the next goes without.
        let drawn = g.evening(id(1), 6, 2_640, 4.0, 0.6, 0.071);
        assert!((drawn - 0.568).abs() < 1e-9);
        assert_eq!(g.connections[1].cause, Cause::NoGas);
        // Cut off, and a broken main.
        g.connections[0].cut_off = true;
        g.mains[0].broken = true;
        assert_eq!(g.evening(id(1), 7, 4_080, 4.0, 10.0, 0.071), 0.0);
        let causes: Vec<Cause> = g.connections.iter().map(|c| c.cause).collect();
        assert_eq!(causes, [Cause::CutOff, Cause::Broken, Cause::Broken]);
        assert!(!g.lit(id(200), 4_100));
        // A main laid only in part carries nothing.
        g.mains[0].broken = false;
        g.mains[0].laid_m = 20.0;
        assert_eq!(g.evening(id(1), 8, 5_520, 4.0, 10.0, 0.071), 0.0);
        assert_eq!(g.connections[1].evening, 7, "not checked");
        for c in [
            Cause::Served,
            Cause::NoGas,
            Cause::Overloaded,
            Cause::Broken,
            Cause::CutOff,
        ] {
            assert_eq!(Cause::from_code(c.code()), c);
        }
    }
}
