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
    pub(super) fn deliver(&mut self, ctx: &Ctx, m: Move) {
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
    /// Someone, `who`, worked field `field` for `hours` today: kept for midnight if they shed a
    /// disease that passes by water today, so that a share of their day's shedding goes to the
    /// ground there rather than to their heap (ADR-0021 §4).
    pub(super) fn note_worked(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        field: PermanentId,
        hours: f64,
    ) {
        if hours <= 0.0 || !self.sheds_by_water(ctx, who) {
            return;
        }
        match self.worked.iter_mut().find(|w| w.0 == who && w.1 == field) {
            Some(w) => w.2 += hours as f32,
            None => self.worked.push((who, field, hours as f32)),
        }
    }

    /// Someone, `who`, worked on the ground at `at` for `hours` today, away from home: at a pit or
    /// a deposit, a gathering block or a way under work, at a fire, a well, a bank, a ditch, a
    /// palisade or a crossing, or at a building being built or a workshop, mill or firm, whose
    /// yard or privy is the ground there: kept by its land cell for midnight if they shed a
    /// disease that passes by water today, as [`Population::note_worked`] keeps a field (ADR-0021
    /// §4). On their home's own cell they are at home, and off the land (fishing from a bank,
    /// say) nothing is kept: either way they shed at home.
    pub(super) fn note_worked_ground(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        at: (f32, f32),
        hours: f64,
    ) {
        if hours <= 0.0 || !self.sheds_by_water(ctx, who) {
            return;
        }
        let cell = cell_of(ctx.map, at);
        if ctx.map.water.get(cell) != Some(&civ_world::WATER_LAND) {
            return;
        }
        let own = self.person(who).map(|p| p.household);
        let home = own
            .and_then(|h| self.household(h))
            .map(|x| cell_of(ctx.map, x.home));
        if home == Some(cell) {
            return;
        }
        // At another household's home its heap is the home's ground (ADR-0021 §4): the heap
        // beside it takes what they shed there, as it takes its own people's.
        let host = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| Some(x.id) != own && !x.members.is_empty())
            .find(|x| cell_of(ctx.map, x.home) == cell)
            .map(|x| x.id);
        if let Some(host) = host {
            match self
                .worked_homes
                .iter_mut()
                .find(|w| w.0 == who && w.1 == host)
            {
                Some(w) => w.2 += hours as f32,
                None => self.worked_homes.push((who, host, hours as f32)),
            }
            return;
        }
        let cell = cell as u32;
        match self
            .worked_ground
            .iter_mut()
            .find(|w| w.0 == who && w.1 == cell)
        {
            Some(w) => w.2 += hours as f32,
            None => self.worked_ground.push((who, cell, hours as f32)),
        }
    }

    /// Someone, `who`, worked on the ground at `at` for `hours` today, as
    /// [`Population::note_worked_ground`] keeps it: for tests.
    #[doc(hidden)]
    pub fn note_worked_ground_for_tests(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        at: (f32, f32),
        hours: f64,
    ) {
        self.note_worked_ground(ctx, who, at, hours);
    }

    /// Someone, `who`, worked `hours` making `made` (good, kilograms) into holder `holder`'s
    /// stock (ADR-0045): if they shed a disease that passes by water today and will tomorrow,
    /// `food_share` of what they shed over those hours goes onto the food made, by its share of
    /// the kilograms, taken ahead from tomorrow's shedding and never more than it.
    #[cold]
    #[inline(never)]
    pub(super) fn food_made(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        holder: PermanentId,
        made: &[(usize, f64)],
        hours: f64,
    ) {
        let share = ctx.land_params.contamination.food_share;
        if share <= 0.0 || hours <= 0.0 {
            return;
        }
        let goods = &ctx.catalog.goods;
        let food: Vec<(usize, f64)> = made
            .iter()
            .copied()
            .filter(|&(g, kg)| kg > 0.0 && goods.get(g).is_some_and(|d| d.edible()))
            .collect();
        let total: f64 = food.iter().map(|f| f.1).sum();
        if total <= 0.0 {
            return;
        }
        let day = ctx.now.day_index();
        let episodes: Vec<(u16, f64, f64)> = self
            .sickness
            .running()
            .iter()
            .filter_map(|&i| self.sickness.get(i))
            .filter(|e| e.person == who && e.shedding(day) && e.shedding(day + 1))
            .filter_map(|e| {
                let def = ctx.catalog.diseases.get(usize::from(e.disease))?;
                if !def.passes_by(crate::params::DiseaseRoute::Water) {
                    return None;
                }
                let rate = |d: i64| {
                    if e.ill_on(d) {
                        def.shed_ill_per_day
                    } else {
                        def.shed_silent_per_day
                    }
                };
                Some((e.disease, rate(day), rate(day + 1)))
            })
            .collect();
        for (disease, today, tomorrow) in episodes {
            let owed = self
                .contagion
                .owed
                .get(&(who, disease))
                .copied()
                .unwrap_or(0.0);
            let amount = (today * hours / 24.0 * share).min((tomorrow - owed).max(0.0));
            if amount <= 0.0 {
                continue;
            }
            for &(g, kg) in &food {
                let node = Node::Food {
                    holder,
                    good: g as u16,
                };
                let on = amount * kg / total;
                let stock = self.holder_kg(holder, g);
                self.contagion.food_add(node, disease, on, stock);
                self.contagion.record(Move {
                    day,
                    disease,
                    how: How::Handled,
                    from: Node::Person(who),
                    to: node,
                    amount: on,
                });
            }
            *self.contagion.owed.entry((who, disease)).or_default() += amount;
        }
    }

    /// `moved` kilograms of food good `good` went from holder `from` to holder `to`, which held
    /// `from_kg` and `to_kg` before (ADR-0045): the load on what `from` held goes with them by
    /// their share of its stock.
    #[cold]
    #[inline(never)]
    pub(super) fn food_moved(
        &mut self,
        day: i64,
        (from, to): (PermanentId, PermanentId),
        good: usize,
        moved: f64,
        (from_kg, to_kg): (f64, f64),
    ) {
        let good = good as u16;
        let (src, dst) = (
            Node::Food { holder: from, good },
            Node::Food { holder: to, good },
        );
        if !self.contagion.food_kg.contains_key(&src) || moved <= 0.0 || from_kg <= 0.0 {
            return;
        }
        self.contagion.food_reconcile(src, from_kg);
        self.contagion.food_reconcile(dst, to_kg);
        let share = (moved / from_kg).clamp(0.0, 1.0);
        for (disease, load) in self.contagion.food_loads(src) {
            let amount = self.contagion.take_amount(src, disease, load * share);
            self.contagion.food_add(dst, disease, amount, to_kg + moved);
            if amount > NEGLIGIBLE {
                self.contagion.record(Move {
                    day,
                    disease,
                    how: How::Handled,
                    from: src,
                    to: dst,
                    amount,
                });
            }
        }
        self.contagion.food_settled(src, from_kg - moved);
    }

    /// Someone, `who`, of household `household` ate `eaten` (good, kilograms, its stock before),
    /// (ADR-0045): from each loaded stock they take its load by the share of it they ate, a dose
    /// drawn at midnight with the day's other exposures.
    #[cold]
    #[inline(never)]
    pub(super) fn food_eaten(
        &mut self,
        who: PermanentId,
        household: PermanentId,
        eaten: &[(usize, f64, f64)],
    ) {
        for &(g, kg, stock) in eaten {
            let node = Node::Food {
                holder: household,
                good: g as u16,
            };
            if kg <= 0.0 || stock <= 0.0 || !self.contagion.food_kg.contains_key(&node) {
                continue;
            }
            self.contagion.food_reconcile(node, stock);
            let share = (kg / stock).clamp(0.0, 1.0);
            for (disease, _) in self.contagion.food_loads(node) {
                let dose = self.contagion.take(node, disease, share);
                if dose > 0.0 {
                    self.contagion.eaten.push((who, disease, dose, node));
                }
            }
            self.contagion.food_settled(node, stock - kg);
        }
    }

    /// Whether `who` sheds a disease that passes by water today.
    fn sheds_by_water(&self, ctx: &Ctx, who: PermanentId) -> bool {
        let day = ctx.now.day_index();
        self.sickness.running().iter().any(|&i| {
            self.sickness.get(i).is_some_and(|e| {
                e.person == who
                    && e.shedding(day)
                    && ctx
                        .catalog
                        .diseases
                        .get(usize::from(e.disease))
                        .is_some_and(|d| d.passes_by(crate::params::DiseaseRoute::Water))
            })
        })
    }

    /// Someone, `who`, worked field `field` for `hours` today, as [`Population::note_worked`]
    /// keeps it: for tests.
    #[doc(hidden)]
    pub fn note_worked_for_tests(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        field: PermanentId,
        hours: f64,
    ) {
        self.note_worked(ctx, who, field, hours);
    }

    pub(super) fn contagion_day(
        &mut self,
        ctx: &Ctx,
        day: i64,
        worked: &[(PermanentId, PermanentId, f32)],
        worked_ground: &[(PermanentId, u32, f32)],
        worked_homes: &[(PermanentId, PermanentId, f32)],
    ) {
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
        // Whoever sheds adds the day's shedding to their household's heap, less what they passed
        // to food yesterday ahead of it (ADR-0045).
        let owed = std::mem::take(&mut self.contagion.owed);
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
            // What they passed to food yesterday ahead of today's shedding is not shed again
            // (ADR-0045).
            let owed = owed.get(&(e.person, e.disease)).copied().unwrap_or(0.0);
            let amount = (amount - owed).max(0.0);
            // Its share of the day spent working a field yesterday goes to the ground there, the
            // rest to the household's heap (ADR-0021 §4; a design prior: shedding spread evenly
            // over the day's 24 hours).
            let mut home = amount;
            for &(_, field, hours) in worked.iter().filter(|w| w.0 == e.person) {
                if !ctx.land.fields.iter().any(|f| f.id == field) {
                    continue;
                }
                let there = (amount * f64::from(hours) / 24.0).min(home);
                if there <= 0.0 {
                    continue;
                }
                home -= there;
                shed.push(Move {
                    day,
                    disease: e.disease,
                    how: How::Shed,
                    from: Node::Person(e.person),
                    to: Node::Field(field),
                    amount: there,
                });
            }
            // And on other ground they worked: a pit or a deposit, a gathering block, a way.
            for &(_, cell, hours) in worked_ground.iter().filter(|w| w.0 == e.person) {
                let there = (amount * f64::from(hours) / 24.0).min(home);
                if there <= 0.0 {
                    continue;
                }
                home -= there;
                shed.push(Move {
                    day,
                    disease: e.disease,
                    how: How::Shed,
                    from: Node::Person(e.person),
                    to: Node::Ground(cell),
                    amount: there,
                });
            }
            // And at other households' homes, to their heaps.
            for &(_, host, hours) in worked_homes.iter().filter(|w| w.0 == e.person) {
                if host == p.household || self.household(host).is_none() {
                    continue;
                }
                let there = (amount * f64::from(hours) / 24.0).min(home);
                if there <= 0.0 {
                    continue;
                }
                home -= there;
                shed.push(Move {
                    day,
                    disease: e.disease,
                    how: How::Shed,
                    from: Node::Person(e.person),
                    to: Node::Midden(host),
                    amount: there,
                });
            }
            shed.push(Move {
                day,
                disease: e.disease,
                how: How::Shed,
                from: Node::Person(e.person),
                to: Node::Midden(p.household),
                amount: home,
            });
        }
        for &m in &shed {
            self.deliver(ctx, m);
        }
        // What homes served by a sewer today shed goes with their share to its outfall
        // (ADR-0041 §2).
        self.sewer_loads(ctx, day, &shed);
        // The day's surplus washes heaps off, and a share of each soaks in.
        let params = &ctx.land_params.contamination;
        let wash =
            (params.wash_per_mm * ctx.land.water.last_surplus_mm).clamp(0.0, params.wash_max);
        // Tips wash and seep as heaps do, from their own place (M11f, second part; ADR-0038 §3),
        // and sewage farms from their field's (ADR-0044 §4).
        let heaps: Vec<(Node, u16)> = self
            .contagion
            .loads
            .keys()
            .filter(|&&(node, _)| {
                matches!(
                    node,
                    Node::Midden(_) | Node::Tip(_) | Node::Field(_) | Node::Ground(_)
                )
            })
            .copied()
            .collect();
        for (from, disease) in heaps {
            // Where it lies: carried from the water under a law clearing middens, or at home
            // (M10c slice EE, third part); or the tip's place; or the farm field's centre.
            let place = match from {
                Node::Midden(household) => self
                    .household(household)
                    .map(|x| self.midden_place(ctx, x, day)),
                Node::Tip(tip) => self.tip_place(tip),
                Node::Field(field) => ctx
                    .land
                    .fields
                    .iter()
                    .find(|f| f.id == field)
                    .map(|f| f.rect.centre_m()),
                // Ground someone shedding worked, from its cell's middle (ADR-0021 §4).
                Node::Ground(cell) => Some(cell_centre(ctx.map, cell as usize)),
                _ => None,
            };
            let Some(home) = place else {
                // A heap whose household is no more, a tip nobody keeps, or a field gone, is let
                // go with it.
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
            // A tip all under earth washes off in no rain (ADR-0038 §6's covering; research 12-10
            // §5.4): it still seeps.
            let covered = matches!(from, Node::Tip(tip) if self.tip_covered(tip));
            if wash > 0.0 && !covered {
                let washed = self.contagion.take(from, disease, wash);
                let near = self.receptors(ctx, home, z, params.runoff_radius_m);
                let total: f64 = near.iter().map(|r| 1.0 / r.distance_m).sum();
                for r in near {
                    // A covered mouth takes only its well's share of what an open one would; the
                    // rest soaks away (M6a slice BA).
                    let mouth = match r.node {
                        Node::Well(w) => ctx.land.wells.get(w).map_or(1.0, |w| {
                            if w.covered {
                                ctx.catalog
                                    .wells
                                    .get(w.system)
                                    .map_or(1.0, |d| d.covered_runoff)
                            } else {
                                1.0
                            }
                        }),
                        _ => 1.0,
                    };
                    let m = Move {
                        day,
                        disease,
                        how: How::Runoff,
                        from,
                        to: r.node,
                        amount: washed / r.distance_m / total * mouth,
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
            self.seep_toward(ctx, day, (from, disease), home, seeped, def.decay_per_day);
        }
        self.contagion.prune_moves(day);
    }

    /// `seeped` doses of disease `disease` from node `from` soak into the ground at `place` on
    /// `day`: `capture_share` of them toward the wells and rivers within `link_radius_m` and no
    /// higher, split by nearness, each arriving after 03-02 §1.6's travel time through the unit
    /// of ground there, less what dies on the way at `decay_per_day` (M6a slice AZ); the rest
    /// soaks away. A heap's seepage goes this way, and so does what leaks from a sewer's worn
    /// walls (research 12-11 §4).
    pub(super) fn seep_toward(
        &mut self,
        ctx: &Ctx,
        day: i64,
        (from, disease): (Node, u16),
        place: (f32, f32),
        seeped: f64,
        decay_per_day: f64,
    ) {
        let params = &ctx.land_params.contamination;
        let cell = cell_of(ctx.map, place);
        let z = f64::from(ctx.map.elevation[cell]);
        let aq = &ctx.land.water.aquifer;
        let unit = aq
            .unit
            .get(ctx.land.patches.of_cell(cell, ctx.map.width))
            .map_or(0, |&u| usize::from(u));
        let k = aq.unit_k_m_day.get(unit).copied().unwrap_or(0.0);
        let n = aq.unit_sy.get(unit).copied().unwrap_or(0.1).max(1e-3);
        let links = self.receptors(ctx, place, z, params.link_radius_m);
        let total: f64 = links.iter().map(|r| 1.0 / r.distance_m).sum();
        for r in links {
            // 03-02 §1.6: water moves at K·i/n, so it takes d·n/(K·i).
            let i = (r.drop_m / r.distance_m).max(params.min_gradient);
            let tau = r.distance_m * n / (k * i).max(1e-12);
            if !tau.is_finite() || tau > LONGEST_TRAVEL_DAYS {
                continue;
            }
            let amount =
                seeped * params.capture_share / r.distance_m / total * (-decay_per_day * tau).exp();
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

    /// The day's turn for loads now, whatever else the day does: for tests.
    #[doc(hidden)]
    pub fn contagion_day_for_tests(&mut self, ctx: &Ctx) {
        let worked = std::mem::take(&mut self.worked);
        let ground = std::mem::take(&mut self.worked_ground);
        let homes = std::mem::take(&mut self.worked_homes);
        self.contagion_day(ctx, ctx.now.day_index(), &worked, &ground, &homes);
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
