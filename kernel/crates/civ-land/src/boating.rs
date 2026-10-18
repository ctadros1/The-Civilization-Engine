//! The river for boats (M8b slice CO; ADR-0029 §5, §7): which reaches a hull of a given draft can
//! use today, its speed over the ground along each, and the quickest way by water between two
//! places, directed. A reach is navigable today where its stage at today's discharge exceeds the
//! hull's draft; downstream a boat goes at its paddlers' speed plus the reach's mean velocity,
//! upstream at their speed less it, and not at all where that is not positive. Low water closes
//! reaches and high water speeds or stops them by the day's discharge, never by a calendar.
//! Lakes and the sea are not yet water a boat uses.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use civ_world::hydraulics::ReachHydraulics;
use civ_world::{WATER_LAND, WATER_RIVER, WorldMap, grid};

/// A boat as the river sees it: its draft loaded and its paddlers' speed through still water.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hull {
    /// Metres of water it needs under it, loaded.
    pub draft_m: f64,
    /// Metres a second through still water.
    pub paddle_m_s: f64,
}

/// A reach's mean velocity at discharge `q_m3s`, metres a second: the discharge over the wetted
/// section, `q / (width · stage)` (continuity, with Manning's stage).
pub fn velocity(rh: &ReachHydraulics, q_m3s: f64) -> f64 {
    let stage = rh.stage(q_m3s);
    if stage <= 0.0 || rh.width_m <= 0.0 {
        0.0
    } else {
        q_m3s / (rh.width_m * stage)
    }
}

/// How a hull fares on one reach today.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Going {
    /// Today's stage, metres.
    pub stage_m: f64,
    /// Today's mean velocity, metres a second.
    pub velocity_m_s: f64,
    /// Speed over the ground downstream, metres a second; `None` where the reach is too shallow.
    pub down_m_s: Option<f64>,
    /// Speed over the ground upstream; `None` where too shallow or the current is too strong.
    pub up_m_s: Option<f64>,
}

/// How `hull` fares on reach hydraulics `rh` at today's discharge `q_m3s`.
pub fn going(rh: &ReachHydraulics, q_m3s: f64, hull: Hull) -> Going {
    let stage_m = rh.stage(q_m3s);
    let velocity_m_s = velocity(rh, q_m3s);
    let deep = stage_m > hull.draft_m;
    let down = hull.paddle_m_s + velocity_m_s;
    let up = hull.paddle_m_s - velocity_m_s;
    Going {
        stage_m,
        velocity_m_s,
        down_m_s: (deep && down > 0.0).then_some(down),
        up_m_s: (deep && up > 0.0).then_some(up),
    }
}

/// One step along a reach from one river cell to the next downstream, by node index.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Edge {
    from: u32,
    to: u32,
    reach: u32,
    metres: f64,
}

/// The rivers as a graph of cells, derived from the map: every step along a reach downstream,
/// and from a reach's last cell into the reach it joins. Each step is gone either way, at the
/// speed its own reach's water gives.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RiverGraph {
    /// River cells, ascending.
    nodes: Vec<u32>,
    edges: Vec<Edge>,
    /// For each node, the edges touching it (either end), by index.
    touching: Vec<Vec<u32>>,
    width: u32,
    height: u32,
}

/// The quickest way by water.
#[derive(Clone, Debug, PartialEq)]
pub struct RiverRoute {
    /// River cells from the one boarded to the one left, in order.
    pub cells: Vec<u32>,
    /// Seconds afloat.
    pub seconds: f64,
    /// Metres afloat.
    pub metres: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Open {
    seconds: f64,
    node: u32,
}

impl Eq for Open {}

impl Ord for Open {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .seconds
            .total_cmp(&self.seconds)
            .then(other.node.cmp(&self.node))
    }
}

impl PartialOrd for Open {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Seconds to go one step today, from its `from` to its `to` and back.
type Costs = Vec<(Option<f64>, Option<f64>)>;

impl RiverGraph {
    /// The graph of `map`'s rivers.
    pub fn new(map: &WorldMap) -> Self {
        let cell_m = f64::from(map.cell_size_m);
        let w = map.width as usize;
        let mut nodes: Vec<u32> = map
            .reaches
            .iter()
            .flat_map(|r| r.cells.iter().copied())
            .filter(|&c| map.water.get(c as usize) == Some(&WATER_RIVER))
            .collect();
        nodes.sort_unstable();
        nodes.dedup();
        let index = |c: u32| nodes.binary_search(&c).ok().map(|i| i as u32);
        let metres = |a: u32, b: u32| {
            let (a, b) = (nodes[a as usize] as usize, nodes[b as usize] as usize);
            let dx = (a % w) as f64 - (b % w) as f64;
            let dy = (a / w) as f64 - (b / w) as f64;
            dx.hypot(dy) * cell_m
        };
        let mut edges = Vec::new();
        for r in &map.reaches {
            let cells: Vec<u32> = r.cells.iter().copied().filter_map(index).collect();
            for pair in cells.windows(2) {
                if pair[0] != pair[1] {
                    edges.push(Edge {
                        from: pair[0],
                        to: pair[1],
                        reach: r.id,
                        metres: metres(pair[0], pair[1]),
                    });
                }
            }
            // Into the reach it joins, unless its last cell is already that reach's.
            let Some(dr) = r.downstream.and_then(|d| map.reaches.get(d as usize)) else {
                continue;
            };
            let Some(&last) = cells.last() else {
                continue;
            };
            let shared = dr.cells.iter().any(|&c| index(c) == Some(last));
            if let Some(next) = dr.cells.iter().copied().find_map(index)
                && !shared
                && next != last
            {
                edges.push(Edge {
                    from: last,
                    to: next,
                    reach: dr.id,
                    metres: metres(last, next),
                });
            }
        }
        let mut touching = vec![Vec::new(); nodes.len()];
        for (i, e) in edges.iter().enumerate() {
            touching[e.from as usize].push(i as u32);
            touching[e.to as usize].push(i as u32);
        }
        RiverGraph {
            nodes,
            edges,
            touching,
            width: map.width,
            height: map.height,
        }
    }

    /// Whether `cell` is a river cell of the graph.
    pub fn is_river(&self, cell: u32) -> bool {
        self.nodes.binary_search(&cell).is_ok()
    }

    /// How many river cells the graph has.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the map has no river.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The river cells a boat can be boarded into from land cell `bank`: its D8 neighbours that
    /// are river cells, ascending. Empty for a cell that is not a bank.
    pub fn boarding(&self, map: &WorldMap, bank: u32) -> Vec<u32> {
        if map.water.get(bank as usize) != Some(&WATER_LAND) {
            return Vec::new();
        }
        let (w, h) = (self.width as usize, self.height as usize);
        let mut out: Vec<u32> = grid::D8
            .iter()
            .filter_map(|&o| grid::neighbor(w, h, bank as usize, o))
            .map(|c| c as u32)
            .filter(|&c| self.is_river(c))
            .collect();
        out.sort_unstable();
        out
    }

    fn costs(&self, hyd: &[ReachHydraulics], q: &[f32], hull: Hull) -> Costs {
        let going: Vec<Option<Going>> = hyd
            .iter()
            .enumerate()
            .map(|(r, rh)| q.get(r).map(|&q| going(rh, f64::from(q), hull)))
            .collect();
        self.edges
            .iter()
            .map(|e| {
                going
                    .get(e.reach as usize)
                    .copied()
                    .flatten()
                    .map_or((None, None), |g| {
                        (
                            g.down_m_s.map(|v| e.metres / v),
                            g.up_m_s.map(|v| e.metres / v),
                        )
                    })
            })
            .collect()
    }

    /// The quickest way by water today for `hull` from any of the river cells `from` to any of
    /// `to`, with reach hydraulics `hyd` and today's discharges `q` (both by reach); `None` where
    /// no navigable way joins them. Ties go to the lower cell, so it is the same every run.
    pub fn route(
        &self,
        hyd: &[ReachHydraulics],
        q: &[f32],
        hull: Hull,
        from: &[u32],
        to: &[u32],
    ) -> Option<RiverRoute> {
        let costs = self.costs(hyd, q, hull);
        let targets: Vec<u32> = to
            .iter()
            .filter_map(|&c| self.nodes.binary_search(&c).ok().map(|i| i as u32))
            .collect();
        let n = self.nodes.len();
        let mut best = vec![f64::INFINITY; n];
        let mut came: Vec<Option<(u32, u32)>> = vec![None; n];
        let mut heap = BinaryHeap::new();
        for &c in from {
            if let Ok(i) = self.nodes.binary_search(&c) {
                best[i] = 0.0;
                heap.push(Open {
                    seconds: 0.0,
                    node: i as u32,
                });
            }
        }
        while let Some(Open { seconds, node }) = heap.pop() {
            if seconds > best[node as usize] {
                continue;
            }
            if targets.contains(&node) {
                return Some(self.path(node, &came, seconds));
            }
            for &ei in &self.touching[node as usize] {
                let e = self.edges[ei as usize];
                let (next, cost) = if e.from == node {
                    (e.to, costs[ei as usize].0)
                } else {
                    (e.from, costs[ei as usize].1)
                };
                let Some(cost) = cost else { continue };
                let s = seconds + cost;
                if s < best[next as usize] {
                    best[next as usize] = s;
                    came[next as usize] = Some((node, ei));
                    heap.push(Open {
                        seconds: s,
                        node: next,
                    });
                }
            }
        }
        None
    }

    fn path(&self, end: u32, came: &[Option<(u32, u32)>], seconds: f64) -> RiverRoute {
        let mut cells = vec![self.nodes[end as usize]];
        let mut metres = 0.0;
        let mut at = end;
        while let Some((prev, ei)) = came[at as usize] {
            metres += self.edges[ei as usize].metres;
            cells.push(self.nodes[prev as usize]);
            at = prev;
        }
        cells.reverse();
        RiverRoute {
            cells,
            seconds,
            metres,
        }
    }

    /// The seconds of [`RiverGraph::route`] found again by relaxing every step until nothing
    /// changes (Bellman-Ford): the oracle a test checks routes against (ADR-0029 §7).
    #[doc(hidden)]
    pub fn oracle_seconds(
        &self,
        hyd: &[ReachHydraulics],
        q: &[f32],
        hull: Hull,
        from: &[u32],
        to: &[u32],
    ) -> Option<f64> {
        let costs = self.costs(hyd, q, hull);
        let mut best = vec![f64::INFINITY; self.nodes.len()];
        for &c in from {
            if let Ok(i) = self.nodes.binary_search(&c) {
                best[i] = 0.0;
            }
        }
        loop {
            let mut changed = false;
            for (e, cost) in self.edges.iter().zip(&costs) {
                let (a, b) = (e.from as usize, e.to as usize);
                if let Some(c) = cost.0
                    && best[a] + c < best[b]
                {
                    best[b] = best[a] + c;
                    changed = true;
                }
                if let Some(c) = cost.1
                    && best[b] + c < best[a]
                {
                    best[a] = best[b] + c;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        to.iter()
            .filter_map(|&c| self.nodes.binary_search(&c).ok())
            .map(|i| best[i])
            .filter(|s| s.is_finite())
            .min_by(f64::total_cmp)
    }
}
