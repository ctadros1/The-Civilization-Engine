//! What people shed, moved through the ground and water (M6a slice AZ, step two; ADR-0021 §4;
//! research 12-02 §5.2-§5.4, 03-02 §1.6). At each day's turn: what passed the rivers yesterday
//! has gone downstream; every load decays by its disease's rate; what soaked through the ground
//! arrives where it was going; whoever sheds a disease adds the day's shedding to their household's
//! heap beside its home; the day's surplus washes a share of each heap to the wellheads and banks
//! within reach and no higher; and a share soaks in, toward the wells and rivers within reach and
//! no higher, arriving after its travel time (03-02 §1.6: `τ = d·n / (K·i)`), less what dies on
//! the way. Each heap's share is split among its receptors by nearness, so a load is never sent
//! whole to each (12-02 §5.3). Concentration is load over litres. Each move is recorded. Water
//! drawn carries its share of a well's or reach's load into the household's store, and what each
//! person drinks of the store is a dose (12-02 §5.4).

use super::*;
use crate::contagion::{How, Move, NEGLIGIBLE, Node, Transit};

/// A receptor of what washes or soaks off a heap: where, how far, how far below.
#[derive(Clone, Copy, Debug)]
struct Receptor {
    node: Node,
    distance_m: f64,
    drop_m: f64,
}

/// The longest a load soaks through the ground before it is taken to be held there, days.
const LONGEST_TRAVEL_DAYS: f64 = 3650.0;

impl Population {
    /// River cells by their reach, derived from the map when first needed.
    fn reach_cells(&mut self, map: &WorldMap) -> &BTreeMap<u32, u32> {
        self.reach_cells.get_or_insert_with(|| {
            let mut cells = BTreeMap::new();
            for r in &map.reaches {
                for &c in &r.cells {
                    if map.water.get(c as usize) == Some(&WATER_RIVER) {
                        cells.entry(c).or_insert(r.id);
                    }
                }
            }
            cells
        })
    }

    /// The reach a bank's water at cell `cell` comes from: the cell's own, or a neighbour's.
    fn bank_reach(&mut self, map: &WorldMap, cell: u32) -> Option<u32> {
        let (w, h) = (i64::from(map.width), i64::from(map.height));
        let (x, y) = (i64::from(cell) % w, i64::from(cell) / w);
        let cells = self.reach_cells(map);
        if let Some(&r) = cells.get(&cell) {
            return Some(r);
        }
        for (dx, dy) in [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            if let Some(&r) = cells.get(&((ny * w + nx) as u32)) {
                return Some(r);
            }
        }
        None
    }

    /// Litres passing reach `r` in a day at today's flow.
    fn reach_litres(ctx: &Ctx, r: u32) -> f64 {
        let flow = ctx.land.flow_factor(ctx.land_params);
        ctx.map.reaches.get(r as usize).map_or(0.0, |reach| {
            f64::from(reach.discharge_m3s) * flow * 86_400.0 * 1000.0
        })
    }

    /// Adds `amount` of `disease` to what passes reach `r` today and every reach below it: a
    /// river carries a load downstream within the day.
    fn add_to_river(&mut self, ctx: &Ctx, r: u32, disease: u16, amount: f64) {
        let mut at = Some(r);
        for _ in 0..=ctx.map.reaches.len() {
            let Some(r) = at else {
                break;
            };
            self.contagion.add(Node::Reach(r), disease, amount);
            at = ctx.map.reaches.get(r as usize).and_then(|x| x.downstream);
        }
    }

    /// The open wells and river reaches within `radius_m` of `at` whose ground lies no higher
    /// than `z`, the nearest cell of each reach.
    fn receptors(&mut self, ctx: &Ctx, at: (f32, f32), z: f64, radius_m: f64) -> Vec<Receptor> {
        let mut out = Vec::new();
        if radius_m <= 0.0 {
            return out;
        }
        let (ax, ay) = (f64::from(at.0), f64::from(at.1));
        for w in ctx.land.wells.list.iter().filter(|w| w.is_open()) {
            let (wx, wy) = w.rect.centre_m();
            let d = (f64::from(wx) - ax).hypot(f64::from(wy) - ay);
            let drop = z - f64::from(w.ground_m);
            if d <= radius_m && drop >= 0.0 {
                out.push(Receptor {
                    node: Node::Well(w.id),
                    distance_m: d.max(1.0),
                    drop_m: drop,
                });
            }
        }
        let map = ctx.map;
        let size = f64::from(map.cell_size_m);
        let (cx, cy) = ((ax / size).floor() as i64, (ay / size).floor() as i64);
        let span = (radius_m / size).ceil() as i64;
        let (w, h) = (i64::from(map.width), i64::from(map.height));
        let cells = self.reach_cells(map);
        let mut reaches: BTreeMap<u32, Receptor> = BTreeMap::new();
        for y in (cy - span).max(0)..=(cy + span).min(h - 1) {
            for x in (cx - span).max(0)..=(cx + span).min(w - 1) {
                let cell = (y * w + x) as u32;
                let Some(&r) = cells.get(&cell) else {
                    continue;
                };
                let (px, py) = ((x as f64 + 0.5) * size, (y as f64 + 0.5) * size);
                let d = (px - ax).hypot(py - ay);
                let drop = z - f64::from(map.elevation[cell as usize]);
                if d > radius_m || drop < 0.0 {
                    continue;
                }
                let near = Receptor {
                    node: Node::Reach(r),
                    distance_m: d.max(1.0),
                    drop_m: drop,
                };
                reaches
                    .entry(r)
                    .and_modify(|e| {
                        if near.distance_m < e.distance_m {
                            *e = near;
                        }
                    })
                    .or_insert(near);
            }
        }
        out.extend(reaches.into_values());
        out
    }

    /// Puts move `m`'s load at its end now and records it; a load too small to record dies on
    /// the way.
    fn deliver(&mut self, ctx: &Ctx, m: Move) {
        if m.amount <= NEGLIGIBLE {
            return;
        }
        match m.to {
            Node::Reach(r) => self.add_to_river(ctx, r, m.disease, m.amount),
            node => self.contagion.add(node, m.disease, m.amount),
        }
        self.contagion.record(m);
    }

    /// The day's turn for loads (see the module documentation), before anyone is exposed.
    pub(super) fn contagion_day(&mut self, ctx: &Ctx, day: i64) {
        let diseases = &ctx.catalog.diseases;
        // What passed the rivers yesterday has gone downstream.
        self.contagion
            .loads
            .retain(|(node, _), _| !matches!(node, Node::Reach(_)));
        self.contagion.decay(|d| {
            diseases
                .get(usize::from(d))
                .map_or(1.0, |x| x.decay_per_day)
        });
        // What soaked through the ground arrives.
        if !self.contagion.transit.is_empty() {
            let (due, later): (Vec<Transit>, Vec<Transit>) =
                std::mem::take(&mut self.contagion.transit)
                    .into_iter()
                    .partition(|t| t.arrives <= day);
            self.contagion.transit = later;
            for t in due {
                let m = Move {
                    day,
                    disease: t.disease,
                    how: How::Arrived,
                    from: t.from,
                    to: t.to,
                    amount: t.amount,
                };
                self.deliver(ctx, m);
            }
        }
        // Whoever sheds adds the day's shedding to their household's heap.
        let mut shed = Vec::new();
        for &i in self.sickness.running() {
            let Some(e) = self.sickness.get(i) else {
                continue;
            };
            let (Some(def), Some(p)) =
                (diseases.get(usize::from(e.disease)), self.person(e.person))
            else {
                continue;
            };
            if !e.shedding(day) || !def.passes_by(crate::params::DiseaseRoute::Water) {
                continue;
            }
            let amount = if e.ill_on(day) {
                def.shed_ill_per_day
            } else {
                def.shed_silent_per_day
            };
            shed.push(Move {
                day,
                disease: e.disease,
                how: How::Shed,
                from: Node::Person(e.person),
                to: Node::Midden(p.household),
                amount,
            });
        }
        for m in shed {
            self.deliver(ctx, m);
        }
        // The day's surplus washes heaps off, and a share of each soaks in.
        let params = &ctx.land_params.contamination;
        let wash =
            (params.wash_per_mm * ctx.land.water.last_surplus_mm).clamp(0.0, params.wash_max);
        let heaps: Vec<(PermanentId, u16)> = self
            .contagion
            .loads
            .keys()
            .filter_map(|&(node, d)| match node {
                Node::Midden(h) => Some((h, d)),
                _ => None,
            })
            .collect();
        for (household, disease) in heaps {
            let from = Node::Midden(household);
            let Some(home) = self.household(household).map(|x| x.home) else {
                // A heap whose household is no more is let go with it.
                self.contagion.take(from, disease, 1.0);
                continue;
            };
            let Some(def) = diseases.get(usize::from(disease)) else {
                continue;
            };
            let cell = cell_of(ctx.map, home);
            let z = f64::from(ctx.map.elevation[cell]);
            // Runoff: to the wellheads and banks within reach and no higher, nearer ones taking
            // more; with none there, what washes off soaks away.
            if wash > 0.0 {
                let washed = self.contagion.take(from, disease, wash);
                let near = self.receptors(ctx, home, z, params.runoff_radius_m);
                let total: f64 = near.iter().map(|r| 1.0 / r.distance_m).sum();
                for r in near {
                    let m = Move {
                        day,
                        disease,
                        how: How::Runoff,
                        from,
                        to: r.node,
                        amount: washed / r.distance_m / total,
                    };
                    self.deliver(ctx, m);
                }
            }
            // Seepage: toward the wells and rivers within reach and no higher, the water table
            // taken to follow the ground over so short a way.
            let seeped = self.contagion.take(from, disease, params.seep_per_day);
            if seeped <= 0.0 {
                continue;
            }
            let aq = &ctx.land.water.aquifer;
            let unit = aq
                .unit
                .get(ctx.land.patches.of_cell(cell, ctx.map.width))
                .map_or(0, |&u| usize::from(u));
            let k = aq.unit_k_m_day.get(unit).copied().unwrap_or(0.0);
            let n = aq.unit_sy.get(unit).copied().unwrap_or(0.1).max(1e-3);
            let links = self.receptors(ctx, home, z, params.link_radius_m);
            let total: f64 = links.iter().map(|r| 1.0 / r.distance_m).sum();
            for r in links {
                // 03-02 §1.6: water moves at K·i/n, so it takes d·n/(K·i).
                let i = (r.drop_m / r.distance_m).max(params.min_gradient);
                let tau = r.distance_m * n / (k * i).max(1e-12);
                if !tau.is_finite() || tau > LONGEST_TRAVEL_DAYS {
                    continue;
                }
                let amount = seeped * params.capture_share / r.distance_m / total
                    * (-def.decay_per_day * tau).exp();
                if amount <= NEGLIGIBLE {
                    continue;
                }
                self.contagion.transit.push(Transit {
                    arrives: day + (tau.ceil() as i64).max(1),
                    disease,
                    from,
                    to: r.node,
                    amount,
                });
                self.contagion.record(Move {
                    day,
                    disease,
                    how: How::Seep,
                    from,
                    to: r.node,
                    amount,
                });
            }
        }
        self.contagion.prune_moves(day);
    }

    /// The day's turn for loads now, whatever else the day does: for tests.
    #[doc(hidden)]
    pub fn contagion_day_for_tests(&mut self, ctx: &Ctx) {
        self.contagion_day(ctx, ctx.now.day_index());
    }

    /// Someone of `household` drew `share` of the water standing in well `well` on `day`: the
    /// same share of its load goes into the household's store.
    pub(super) fn drew_load_from_well(
        &mut self,
        well: PermanentId,
        household: PermanentId,
        day: i64,
        share: f64,
    ) {
        let from = Node::Well(well);
        let diseases: Vec<u16> = self
            .contagion
            .loads
            .range((from, 0)..=(from, u16::MAX))
            .map(|(&(_, d), _)| d)
            .collect();
        for disease in diseases {
            if self.contagion.load(from, disease) * share <= NEGLIGIBLE {
                continue;
            }
            let amount = self.contagion.take(from, disease, share);
            self.contagion.store_add(household, disease, from, amount);
            self.contagion.record(Move {
                day,
                disease,
                how: How::Drawn,
                from,
                to: Node::Store(household),
                amount,
            });
        }
    }

    /// Someone of `household` drew `litres` at the bank at cell `cell`: what those litres hold of
    /// what passes its reach today goes into the household's store.
    pub(super) fn drew_load_from_reach(
        &mut self,
        ctx: &Ctx,
        cell: u32,
        household: PermanentId,
        litres: f64,
    ) {
        if !self
            .contagion
            .loads
            .keys()
            .any(|(n, _)| matches!(n, Node::Reach(_)))
        {
            return;
        }
        let Some(r) = self.bank_reach(ctx.map, cell) else {
            return;
        };
        let volume = Self::reach_litres(ctx, r);
        if volume <= 0.0 {
            return;
        }
        let (from, day) = (Node::Reach(r), ctx.now.day_index());
        let diseases: Vec<u16> = self
            .contagion
            .loads
            .range((from, 0)..=(from, u16::MAX))
            .map(|(&(_, d), _)| d)
            .collect();
        let share = (litres / volume).min(1.0);
        for disease in diseases {
            if self.contagion.load(from, disease) * share <= NEGLIGIBLE {
                continue;
            }
            let amount = self.contagion.take(from, disease, share);
            self.contagion.store_add(household, disease, from, amount);
            self.contagion.record(Move {
                day,
                disease,
                how: How::Drawn,
                from,
                to: Node::Store(household),
                amount,
            });
        }
    }

    /// The day's doses drunk from each household's store (12-02 §5.4), by person and disease,
    /// with where most of each store's load was drawn; the share of each store its household uses
    /// in a day goes with the water. Stores of households no more, or dry, are let go.
    pub(super) fn water_doses(&mut self, ctx: &Ctx) -> Vec<(PermanentId, u16, f64, Node)> {
        let mut out = Vec::new();
        if self.contagion.stores.is_empty() {
            return out;
        }
        let drink = ctx.land_params.contamination.drink_l_per_day.max(0.0);
        let keys: Vec<(PermanentId, u16)> = self.contagion.stores.keys().copied().collect();
        for (household, disease) in keys {
            let Some(x) = self.household(household) else {
                self.contagion.stores.remove(&(household, disease));
                continue;
            };
            let use_l = x.water_use(&ctx.params.household);
            let used = x.members.len() as f64 * use_l;
            let litres = x.water_at_time(ctx.now, used);
            let members = x.members.clone();
            let load = self.contagion.store_load(household, disease);
            if litres <= 0.0 || load <= 0.0 {
                self.contagion.stores.remove(&(household, disease));
                continue;
            }
            let passes = ctx
                .catalog
                .diseases
                .get(usize::from(disease))
                .is_some_and(|d| d.passes_by(crate::params::DiseaseRoute::Water));
            if passes {
                let source = self.contagion.stores[&(household, disease)]
                    .iter()
                    .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)))
                    .map_or(Node::Store(household), |(n, _)| *n);
                let dose = drink.min(use_l) * load / litres;
                for m in members {
                    out.push((m, disease, dose, source));
                }
            }
            self.contagion
                .store_take(household, disease, (used / litres).min(1.0));
        }
        out
    }
}
