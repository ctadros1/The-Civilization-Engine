//! Sewers and treatment (ADR-0041 §1-§4; research 12-02 §1.5, §5.2; 12-10 §1.7, §5.8). A polity
//! lays a sewer by law (`lay_sewer`), as a line of current is strung together: along its ways,
//! with their keepers' leave, from a head among its homes to an outfall on a river, falling all the
//! way by the terrain at no less than the content's gradient (no pumping), a metre at a time by
//! its households' work and the walls and lime or cement they bring, carrying only once whole.
//! Homes beside it connect their privies: for nothing where a connection is free and no law asks
//! them to carry their heap, or where what laws ask of them to carry it a year exceeds a
//! connection's work. Each day a connected home's heap gives the sewer what it would have gained
//! that day of its decaying share (its ash still heaps), served in the order connected while the
//! sewer's capacity lasts; a home unserved heaps as before, its cause kept. What goes in reaches
//! the outfall that day, and what its people shed of a disease that day goes with it, recorded as
//! carried: the street is cleaner and the river below the outfall carries the load (12-02 §1.5: a
//! connection does not destroy load). A settling tank its keeper holds at the outfall splits the
//! day's load: the content's share of its mass becomes the good sludge in the keeper's store, with
//! the same share of each disease held at the tank's node; the rest goes on to the river.
//!
//! A sewage farm (ADR-0044 §4; research 12-10 §1.7, §2.2, 12-02 §1.6, §2.3): what leaves the tank,
//! or the whole flow without one, is led onto a field beside the route below its lowest
//! connection, so that the whole flow passes it, up to what its area takes (`farm_people_ha`
//! people's discharge a hectare); the rest goes on to the river. What the farm takes enters the
//! field's soil as dung at the heap's nitrogen a kilogram, so its crop answers only where its soil
//! held it back; its share of each disease lies at the field's node and washes and soaks in from
//! there as a tip's does. The field's household takes the flow, for nothing, after a harvest its
//! soil held back, and stops once a harvest is not. Nobody ties sickness to a field.
//!
//! Wear, silt and mending (ADR-0044 §5; research 12-02 §1.5, 12-08 §2.1, §4.3, §5.1-§5.4): a share
//! of what enters a day stays as deposit (mass only; its loads pass with the flow), and the room
//! the sewer has falls in proportion to what its length holds before it is blocked; the homes it
//! has no room for heap again, the last connected first. Each month every cell of its route wears
//! `1 / life_years` of its soundness times its ground's wetness and the month's rain, as a well's
//! lining does; a cell worn through collapses, and the homes whose drains enter at or above the
//! collapse nearest the outfall heap again while those below still flow. Each connection keeps the
//! days of the year it went unserved, by cause. The polity learns of a fault only as it shows: a
//! home backing up for want of room it once had, or a collapse. While the law that laid the sewer
//! is in force, its review weighs the fault as laying was weighed and, if worth it, asks the work
//! as laying's was asked, an equal share of each household: clearing, whose deposit becomes sludge
//! in the keeper's store, or relining, which rebuilds a collapsed cell at laying's hours and
//! materials a metre. Neither mends the other's fault; with the law ended, nobody mends.

use super::wells::{month_rain, wetness_at};
use super::*;
use crate::contagion::{How, Move, NEGLIGIBLE, Node};
use crate::history::{Conveyance, SewerStep};
use crate::polity::{LawStatus, PolicyKind, Sewer, SewerCause, SewerConnection, SewerDef, WaySite};
use crate::sickness::Outcome;
use civ_land::Limit;
use civ_land::fields::{Field, RectCm};
use civ_land::ways::Keeper;

impl Population {
    /// The terms of the first template that lays a sewer, if the content has one.
    pub(super) fn sewer_def<'a>(ctx: &Ctx<'a>) -> Option<&'a SewerDef> {
        ctx.catalog
            .policies
            .iter()
            .filter(|d| d.kind == PolicyKind::LaySewer)
            .find_map(|d| d.sewer.as_ref())
    }

    /// The polity and the index of sewer `sewer` in its list, if any polity lays it.
    fn sewer_at(&self, sewer: PermanentId) -> Option<(usize, usize)> {
        self.polities.iter().enumerate().find_map(|(pi, p)| {
            p.sewers
                .iter()
                .position(|s| s.id == sewer)
                .map(|si| (pi, si))
        })
    }

    /// Whether `way` is a sewer some polity lays.
    pub(super) fn is_sewer(&self, way: PermanentId) -> bool {
        self.sewer_at(way).is_some()
    }

    /// The settlement of the polity that lays sewer `sewer`.
    pub(super) fn sewer_settlement(&self, sewer: PermanentId) -> Option<PermanentId> {
        self.sewer_at(sewer)
            .map(|(pi, _)| self.polities[pi].settlement)
    }

    /// The route of a sewer polity `pi` would lay from cell `from` (ADR-0041 §1): along ways kept
    /// by nobody, by the polity or by a household of its settlements, falling all the way at no
    /// less than `fall`, to a river cell; its cells, length, metres, and the outfall's reach.
    fn sewer_route(
        &self,
        ctx: &Ctx,
        pi: usize,
        from: u32,
        fall: f64,
    ) -> Option<(Vec<u32>, f64, u32)> {
        let polity = &self.polities[pi];
        let map = ctx.map;
        let river = |c: u32| map.water.get(c as usize) == Some(&WATER_RIVER);
        let cells = civ_land::sewers::plan_sewer(
            &ctx.land.ways,
            map.width,
            map.height,
            &map.elevation,
            f64::from(map.cell_size_m),
            from,
            fall,
            river,
            |k| match k {
                Keeper::Nobody => true,
                Keeper::Polity(p) => p == polity.id,
                Keeper::Household(h) => self
                    .household(h)
                    .is_some_and(|x| polity.holds(x.settlement)),
            },
        )?;
        let outfall = *cells.last()?;
        let reach = map.reaches.iter().find(|r| r.cells.contains(&outfall))?.id;
        let length_m: f64 = civ_land::rail::runs(&cells, map.width, f64::from(map.cell_size_m))
            .iter()
            .sum();
        Some((cells, length_m, reach))
    }

    /// Whether cell `cell` lies on or beside one of `cells` (the eight neighbours' measure), as a
    /// home beside a main connects to it.
    fn beside(map: &WorldMap, cells: &[u32], cell: u32) -> bool {
        let w = i64::from(map.width.max(1));
        let (x, y) = (i64::from(cell) % w, i64::from(cell) / w);
        cells
            .iter()
            .any(|&c| (i64::from(c) % w - x).abs() <= 1 && (i64::from(c) / w - y).abs() <= 1)
    }

    /// The index of the route cell nearest the outfall among `cells` that cell `home` lies beside:
    /// where a home's drain enters (ADR-0044 §5).
    fn entry(map: &WorldMap, cells: &[u32], home: u32) -> Option<usize> {
        cells.iter().rposition(|&r| Self::beside(map, &[r], home))
    }

    /// The metres of route cell `i` of `cells`: half the run to it and half the run from it, so
    /// that the cells' metres sum to the route's length.
    fn cell_metres(map: &WorldMap, cells: &[u32], i: usize) -> f64 {
        let runs = civ_land::rail::runs(cells, map.width, f64::from(map.cell_size_m));
        (runs.get(i).copied().unwrap_or(0.0) + runs.get(i + 1).copied().unwrap_or(0.0)) / 2.0
    }

    /// The metres of sewer `s`'s collapsed cells (ADR-0044 §5).
    fn collapsed_metres(map: &WorldMap, s: &Sewer) -> f64 {
        (0..s.cells.len())
            .filter(|&i| s.collapsed(i))
            .map(|i| Self::cell_metres(map, &s.cells, i))
            .sum()
    }

    /// Whether household `h` is connected to any sewer.
    fn sewered(&self, h: PermanentId) -> bool {
        self.polities
            .iter()
            .flat_map(|p| p.sewers.iter())
            .any(|s| s.connections.iter().any(|c| c.household == h))
    }

    /// The hours a year laws ask household `x` to carry its heap (ADR-0041 §3): to a tip under a
    /// law carrying refuse to one, or from the water under one clearing middens where its home
    /// lies near a source; and a refuse round's fees for calls at its home, at what an hour of its
    /// work brings it.
    fn carry_h_year(&self, ctx: &Ctx, x: &Household) -> f64 {
        let year = DAYS_PER_YEAR as f64;
        let mut hours = 0.0;
        if let Some((_, _, _, t)) = self.tip_law(ctx, x) {
            hours += year / f64::from(t.every_days.max(1)) * f64::from(t.carry_h);
        } else if let Some(pi) = x.settlement.and_then(|s| self.polity_of(s)) {
            for l in self.polities[pi]
                .laws
                .iter()
                .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::ClearMiddens)
            {
                let Some(m) = ctx
                    .catalog
                    .policies
                    .get(usize::from(l.policy))
                    .and_then(|d| d.middens)
                else {
                    continue;
                };
                if Self::nearest_source(ctx, x.home, f64::from(m.distance_m)).is_some() {
                    hours += year / f64::from(m.keep_days.max(1)) * f64::from(m.carry_h);
                    break;
                }
            }
        }
        let fees: f64 = self
            .polities
            .iter()
            .flat_map(|p| p.rounds.iter())
            .filter(|r| r.law.is_none() && r.homes.contains(&x.id))
            .map(|r| f64::from(r.fee_kcal) * year / f64::from(r.every_days.max(1)))
            .sum();
        if fees > 0.0 {
            hours += fees / self.hour_kcal(ctx, x).max(1e-9);
        }
        hours
    }

    /// Deaths of disease within the year of each of `households`, by id.
    fn deaths_of_disease(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, usize)> {
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let mut deaths: Vec<(PermanentId, usize)> = Vec::new();
        for e in self
            .sickness
            .episodes()
            .iter()
            .rev()
            .take_while(|e| e.infected >= since - 120)
        {
            let (Some((d, Outcome::Died)), Some(h)) = (e.ended, e.household) else {
                continue;
            };
            if d <= since || !households.contains(&h) {
                continue;
            }
            match deaths.iter_mut().find(|x| x.0 == h) {
                Some(x) => x.1 += 1,
                None => deaths.push((h, 1)),
            }
        }
        deaths
    }

    /// The hours a year a sewer is reckoned to spare household `x` (ADR-0041 §3): those laws ask
    /// of it to carry its heap, and the template's share of its deaths of disease within the year
    /// (`deaths`, by household) at what a death weighs.
    fn sewer_spared_h(
        &self,
        ctx: &Ctx,
        def: &SewerDef,
        x: &Household,
        deaths: &[(PermanentId, usize)],
    ) -> f64 {
        let died = deaths.iter().find(|d| d.0 == x.id).map_or(0, |d| d.1);
        self.carry_h_year(ctx, x)
            + died as f64
                * f64::from(def.avoided_share)
                * ctx.params.suspicion.w_death
                * ctx.params.farm.work_hours_per_day
    }

    /// What `kg` of the first of `goods` costs at the cheapest any of `ours` can have it, hours
    /// (`costs`, each household's own costs by good), or, where none can put a figure on it,
    /// nothing if they hold `kg` of them between them; `None` otherwise.
    fn material_h(
        ours: &[&Household],
        costs: &[Vec<Option<f64>>],
        goods: &[usize],
        kg: f64,
    ) -> Option<f64> {
        let cheapest = goods
            .iter()
            .flat_map(|&g| {
                costs
                    .iter()
                    .filter_map(move |c| c.get(g).copied().flatten())
            })
            .reduce(f64::min);
        match cheapest {
            Some(h) => Some(kg * h),
            None => {
                let held: f64 = ours
                    .iter()
                    .flat_map(|x| {
                        goods
                            .iter()
                            .map(|&g| x.stores.get(g).copied().unwrap_or(0.0))
                    })
                    .map(|v| v.max(0.0))
                    .sum();
                (held + 1e-9 >= kg).then_some(0.0)
            }
        }
    }

    /// Where the settlement's people could lay a sewer together (ADR-0041 §3), if anywhere: from
    /// the home of one of its households to a river, falling all the way along ways with leave,
    /// where two or more homes beside it with a building, not yet connected to a sewer, and none of
    /// the polity's sewers is being laid. Each such household is reckoned to spare a year the hours
    /// laws ask of it to carry its heap, and the template's share of its deaths of disease within
    /// the year at what a death weighs; the sewer chosen is the one whose pooled hours spared over
    /// its life most exceed its laying and its walls and binder at the cheapest any of the
    /// settlement's households can have them. As a passing place: its head cell, its laying, its
    /// life, what it spares each household beside it, and the households who would share its work.
    pub(crate) fn public_sewer(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        households: &[PermanentId],
    ) -> Option<super::way_works::PublicWay> {
        let def = Self::sewer_def(ctx)?;
        let pi = self.polity_of(settlement)?;
        // One at a time: none of the polity's being laid.
        if self.polities[pi].sewers.iter().any(|s| !s.whole()) {
            return None;
        }
        let ours: Vec<&Household> = households
            .iter()
            .filter_map(|&h| self.household(h))
            .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
            .collect();
        // What a kilogram of each material costs at the cheapest any of them can have it, hours,
        // or, where none can put a figure on it, whether they hold `kg` of it between them.
        let costs: Vec<Vec<Option<f64>>> = ours.iter().map(|x| self.own_costs_of(ctx, x)).collect();
        let material = |goods: &[usize], kg: f64| Self::material_h(&ours, &costs, goods, kg);
        let ids: Vec<PermanentId> = ours.iter().map(|x| x.id).collect();
        let deaths = self.deaths_of_disease(ctx, &ids);
        let homes: Vec<(PermanentId, u32, f64)> = ours
            .iter()
            .filter(|x| {
                !self.sewered(x.id) && ctx.land.buildings.iter().any(|b| b.household == x.id)
            })
            .map(|x| {
                let spared = self.sewer_spared_h(ctx, def, x, &deaths);
                (x.id, cell_of(ctx.map, x.home) as u32, spared)
            })
            .collect();
        let fall = f64::from(def.fall);
        let mut best: Option<(f64, super::way_works::PublicWay)> = None;
        let mut heads: Vec<u32> = homes.iter().map(|h| h.1).collect();
        heads.sort_unstable();
        heads.dedup();
        for head in heads {
            let Some((cells, length_m, _)) = self.sewer_route(ctx, pi, head, fall) else {
                continue;
            };
            let mut saves: Vec<(PermanentId, f64)> = homes
                .iter()
                .filter(|h| Self::beside(ctx.map, &cells, h.1))
                .map(|h| (h.0, h.2))
                .collect();
            if saves.len() < 2 {
                continue;
            }
            saves.sort_unstable_by_key(|s| s.0);
            let (Some(walls), Some(binder)) = (
                material(&def.walls, length_m * f64::from(def.walls_kg_m)),
                material(&def.binders, length_m * f64::from(def.binder_kg_m)),
            ) else {
                continue;
            };
            let way = super::way_works::PublicWay {
                cell: head,
                labour_h: length_m * f64::from(def.lay_h_m),
                life: f64::from(def.life_years),
                saves,
                households: ours.len(),
            };
            let net = way.pooled_net_h() - walls - binder;
            if net > 0.0 && best.as_ref().is_none_or(|(b, _)| net > *b) {
                best = Some((net, way));
            }
        }
        best.map(|b| b.1)
    }

    /// Where the settlement's people could lay a sewer together, as its head cell, its laying and
    /// what it would spare each household a year: for tests.
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub fn public_sewer_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Option<(u32, f64, Vec<(PermanentId, f64)>)> {
        let mut households: Vec<PermanentId> = self.households.iter().map(|(_, x)| x.id).collect();
        households.sort_unstable();
        self.public_sewer(ctx, settlement, &households)
            .map(|p| (p.cell, p.labour_h, p.saves))
    }

    /// Begins a sewer of polity `pi` under law `law` from cell `from` (ADR-0041 §1): its route as
    /// [`Population::sewer_route`] finds it, marked out at its length with nothing yet laid, its
    /// capacity the content's. Returns its id; `None` where the content lays no sewer or no route
    /// falls all the way.
    pub fn begin_sewer(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        law: PermanentId,
        from: u32,
    ) -> Option<PermanentId> {
        let (fall, capacity) = {
            let def = Self::sewer_def(ctx)?;
            (f64::from(def.fall), def.capacity_kg_day)
        };
        let (cells, length_m, reach) = self.sewer_route(ctx, pi, from, fall)?;
        let loss = vec![0.0; cells.len()];
        let id = ctx.ids.allocate();
        self.polities[pi].sewers.push(Sewer {
            id,
            law,
            begun: ctx.now.day_index(),
            cells,
            reach,
            length_m: length_m as f32,
            laid_m: 0.0,
            capacity_kg_day: capacity,
            broken: None,
            keeper: None,
            connections: Vec::new(),
            day: -1,
            entered_kg: 0.0,
            deposited_kg: 0.0,
            settled_kg: 0.0,
            farm: None,
            farm_kg: 0.0,
            deposit_kg: 0.0,
            loss,
            clear_kg: 0.0,
            reline: false,
            relined_m: 0.0,
        });
        Some(id)
    }

    /// The sewer law `law` of polity `pi` passed names is marked out at once from the head cell
    /// its proposal named (ADR-0041 §3), the polity's, its laying its households'.
    pub(super) fn begin_public_sewer(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let Some(site) = self.polities[pi]
            .ways
            .iter()
            .find(|s| s.0 == law)
            .map(|s| s.1)
            .filter(|s: &WaySite| s.work.is_none())
        else {
            return;
        };
        let Some(sewer) = self.begin_sewer(ctx, pi, law, site.cell) else {
            return;
        };
        if let Some(s) = self.polities[pi].ways.iter_mut().find(|s| s.0 == law) {
            s.1.work = Some(sewer);
        }
    }

    /// The sewer law `law` of polity `pi` passed names is marked out: for tests.
    #[doc(hidden)]
    pub fn begin_public_sewer_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.begin_public_sewer(ctx, pi, law);
    }

    /// Kilograms of the first of `goods` household `x` holds, and which good it is, or of none.
    fn first_held(x: &Household, goods: &[usize]) -> Option<(usize, f64)> {
        goods
            .iter()
            .map(|&g| (g, x.stores.get(g).copied().unwrap_or(0.0).max(0.0)))
            .find(|&(_, kg)| kg > 0.0)
    }

    /// The hours left on sewer `sewer` household `x` could work now (ADR-0041 §1): it is being
    /// laid and the household holds a metre's walls and binder; or, whole, the mending its polity
    /// asked ([`Population::sewer_mend_left_h`]). `None` otherwise.
    pub(super) fn sewer_left_h(&self, ctx: &Ctx, x: &Household, sewer: PermanentId) -> Option<f64> {
        let def = Self::sewer_def(ctx)?;
        let (pi, si) = self.sewer_at(sewer)?;
        let s = &self.polities[pi].sewers[si];
        if s.whole() {
            return Self::sewer_mend_left_h(ctx.map, def, x, s);
        }
        let walls = Self::first_held(x, &def.walls)?.1;
        let binder = Self::first_held(x, &def.binders)?.1;
        if walls < f64::from(def.walls_kg_m) || binder < f64::from(def.binder_kg_m) {
            return None;
        }
        Some(f64::from((s.length_m - s.laid_m).max(0.0)) * f64::from(def.lay_h_m))
    }

    /// The hours left on sewer `sewer` household `x` could work now: for tests.
    #[doc(hidden)]
    pub fn sewer_left_for_tests(
        &self,
        ctx: &Ctx,
        x: &Household,
        sewer: PermanentId,
    ) -> Option<f64> {
        self.sewer_left_h(ctx, x, sewer)
    }

    /// The hours of mending left on whole sewer `s` household `x` could work now (ADR-0044 §5):
    /// relining its collapsed cells, where asked and the household holds a metre's walls and
    /// binder, and clearing the deposit asked cleared. `None` where nothing it could do is left.
    fn sewer_mend_left_h(map: &WorldMap, def: &SewerDef, x: &Household, s: &Sewer) -> Option<f64> {
        let mut left = 0.0;
        if s.reline && s.broken.is_some() {
            let walls = Self::first_held(x, &def.walls).map_or(0.0, |w| w.1);
            let binder = Self::first_held(x, &def.binders).map_or(0.0, |b| b.1);
            if walls >= f64::from(def.walls_kg_m) && binder >= f64::from(def.binder_kg_m) {
                let metres = Self::collapsed_metres(map, s) - f64::from(s.relined_m);
                left += metres.max(0.0) * f64::from(def.lay_h_m);
            }
        }
        if s.clear_kg > 0.0 && def.clear_h_t > 0.0 {
            left += s.clear_kg.min(s.deposit_kg).max(0.0) / 1000.0 * f64::from(def.clear_h_t);
        }
        (left > 1e-9).then_some(left)
    }

    /// `hours` of work by someone of `household` on sewer `sewer` its polity lays (ADR-0041 §1): a
    /// metre dug, walled and filled in each `lay_h_m` hours, each taking its walls and binder from
    /// the household's store (the first of each kind it holds), no more than they allow or the
    /// sewer has left. The hours it took are the household's given to the law, and to the polity's
    /// works, paid where a law of pay for common work is in force. On a whole sewer the hours go to
    /// the mending its polity asked ([`Population::sewer_mend`]).
    pub(super) fn sewer_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        sewer: PermanentId,
        hours: f64,
    ) {
        let Some(def) = Self::sewer_def(ctx).cloned() else {
            return;
        };
        let Some((pi, si)) = self.sewer_at(sewer) else {
            return;
        };
        if !self
            .household(household)
            .is_some_and(|x| self.polities[pi].holds(x.settlement))
        {
            return;
        }
        let (law, polity) = (self.polities[pi].sewers[si].law, self.polities[pi].id);
        if self.polities[pi].sewers[si].whole() {
            self.sewer_mend(ctx, &def, household, (pi, si), hours);
            return;
        }
        let left = f64::from(
            (self.polities[pi].sewers[si].length_m - self.polities[pi].sewers[si].laid_m).max(0.0),
        );
        let Some(x) = self.household_mut(household) else {
            return;
        };
        let (Some((wall, walls)), Some((bind, binder))) = (
            Self::first_held(x, &def.walls),
            Self::first_held(x, &def.binders),
        ) else {
            return;
        };
        let metres = (hours.max(0.0) / f64::from(def.lay_h_m).max(1e-9))
            .min(walls / f64::from(def.walls_kg_m).max(1e-9))
            .min(binder / f64::from(def.binder_kg_m).max(1e-9))
            .min(left);
        if metres <= 0.0 {
            return;
        }
        for (g, held, kg_m) in [
            (wall, walls, def.walls_kg_m),
            (bind, binder, def.binder_kg_m),
        ] {
            let used = (metres * f64::from(kg_m)).min(held);
            x.stores[g] = held - used;
            x.flows.add(Flow::Used, g, used);
        }
        let s = &mut self.polities[pi].sewers[si];
        s.laid_m = (s.laid_m + metres as f32).min(s.length_m);
        if s.length_m - s.laid_m < 1e-3 {
            s.laid_m = s.length_m;
        }
        // Laid whole by this work (backlog D, twelfth item): its route and length, to tell.
        let laid = s.whole().then(|| (s.cells.clone(), f64::from(s.length_m)));
        let took = metres * f64::from(def.lay_h_m);
        self.polities[pi].give_to_way(law, household, took as f32);
        self.work_given.push((polity, household, took));
        if let Some((cells, length_m)) = laid {
            self.tell_conveyance_laid(
                ctx,
                Conveyance::Sewer,
                household,
                (Some(pi), None),
                &cells,
                length_m,
            );
        }
    }

    /// `hours` of work by someone of `household` mending whole sewer `si` of polity `pi`
    /// (ADR-0044 §5), while the law that laid it is in force: first relining, where asked, the
    /// broken cell (the collapsed cell nearest the outfall) a metre each `lay_h_m` hours from the
    /// household's walls and binder, as laying takes them, until its metres are rebuilt and its
    /// loss is 0, then the next collapsed cell up; then clearing the deposit asked cleared, a
    /// tonne each `clear_h_t` hours, into sludge in the keeper's store. Relining clears nothing
    /// and clearing mends no cell. The hours it took are given to the law, as laying's are.
    fn sewer_mend(
        &mut self,
        ctx: &mut Ctx,
        def: &SewerDef,
        household: PermanentId,
        (pi, si): (usize, usize),
        hours: f64,
    ) {
        let (law, polity) = (self.polities[pi].sewers[si].law, self.polities[pi].id);
        if !self.polities[pi]
            .laws
            .iter()
            .any(|l| l.id == law && l.status == LawStatus::InForce)
        {
            return;
        }
        let map = ctx.map;
        let lay_h = f64::from(def.lay_h_m).max(1e-9);
        let mut left = hours.max(0.0);
        let mut took = 0.0;
        // Relining, the collapse nearest the outfall first.
        while left > 1e-9 && self.polities[pi].sewers[si].reline {
            let s = &self.polities[pi].sewers[si];
            let Some(b) = s.broken.map(|b| b as usize) else {
                let s = &mut self.polities[pi].sewers[si];
                (s.reline, s.relined_m) = (false, 0.0);
                break;
            };
            let need = (Self::cell_metres(map, &s.cells, b) - f64::from(s.relined_m)).max(0.0);
            let mut metres = need;
            if need > 1e-6 {
                let Some(x) = self.household_mut(household) else {
                    break;
                };
                let (Some((wall, walls)), Some((bind, binder))) = (
                    Self::first_held(x, &def.walls),
                    Self::first_held(x, &def.binders),
                ) else {
                    break;
                };
                metres = (left / lay_h)
                    .min(walls / f64::from(def.walls_kg_m).max(1e-9))
                    .min(binder / f64::from(def.binder_kg_m).max(1e-9))
                    .min(need);
                if metres <= 1e-9 {
                    break;
                }
                for (g, held, kg_m) in [
                    (wall, walls, def.walls_kg_m),
                    (bind, binder, def.binder_kg_m),
                ] {
                    let used = (metres * f64::from(kg_m)).min(held);
                    x.stores[g] = held - used;
                    x.flows.add(Flow::Used, g, used);
                }
                left -= metres * lay_h;
                took += metres * lay_h;
            }
            let s = &mut self.polities[pi].sewers[si];
            s.relined_m += metres as f32;
            if need - metres > 1e-3 {
                continue;
            }
            // Rebuilt: as new, and the break moves to the next collapse up, if any.
            s.loss[b] = 0.0;
            s.relined_m = 0.0;
            s.broken = s.lowest_collapse();
            if s.broken.is_none() {
                s.reline = false;
                self.tell_sewer_mended(ctx, household, (pi, si), b);
            }
        }
        // Clearing.
        let s = &self.polities[pi].sewers[si];
        if left > 1e-9 && s.clear_kg > 0.0 && def.clear_h_t > 0.0 {
            let per_t = f64::from(def.clear_h_t);
            let kg = (left / per_t * 1000.0)
                .min(s.clear_kg)
                .min(s.deposit_kg)
                .max(0.0);
            let s = &mut self.polities[pi].sewers[si];
            s.deposit_kg -= kg;
            s.clear_kg -= kg;
            if s.clear_kg < 1e-6 || s.deposit_kg < 1e-6 {
                s.clear_kg = 0.0;
            }
            if s.deposit_kg < 1e-9 {
                s.deposit_kg = 0.0;
            }
            took += kg / 1000.0 * per_t;
            if let Some(sludge) = def.sludge {
                self.give_keeper(ctx, (pi, si), sludge, kg);
            }
        }
        if took > 0.0 {
            self.polities[pi].give_to_way(law, household, took as f32);
            self.work_given.push((polity, household, took));
        }
    }

    /// The chronicle's word that the people of polity `pi` relined the last stretch of its sewer
    /// `si` that had fallen in, its route cell `at`, with work of `household`'s finishing it: it
    /// carries to the river again (backlog D, fifth item).
    fn tell_sewer_mended(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        (pi, si): (usize, usize),
        at: usize,
    ) {
        let settlement = self.polities[pi].settlement;
        let village = self.village_name(ctx, Some(settlement));
        let elder = self.elder_of(household, ctx.now, ctx.params);
        let by = elder.map_or_else(String::new, |e| {
            format!(", {}'s household finishing it", self.name_of(e))
        });
        let place = self.polities[pi].sewers[si]
            .cells
            .get(at)
            .map(|&c| crate::population::cell_centre(ctx.map, c as usize));
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Sewer,
            elder.into_iter().collect(),
            Some(settlement),
            place,
            f64::from(SewerStep::Mended as u8),
            format!(
                "{village}'s people relined their sewer where it had fallen in{by}: it carries to \
                 the river again."
            ),
        );
    }

    /// `kg` of good `good` into the store of sewer `si` of polity `pi`'s keeper (a household, or
    /// the polity's store).
    fn give_keeper(&mut self, ctx: &Ctx, (pi, si): (usize, usize), good: usize, kg: f64) {
        if kg <= 0.0 {
            return;
        }
        let keeper = self.polities[pi].sewers[si].keeper;
        let store = match keeper {
            Some(h) => self.household_mut(h).map(|x| &mut x.stores),
            None => Some(&mut self.polities[pi].stores),
        };
        if let Some(store) = store {
            if store.len() <= good {
                store.resize(ctx.catalog.goods.len().max(good + 1), 0.0);
            }
            store[good] += kg;
        }
    }

    /// `hours` of work by someone of `household` on sewer `sewer`: for tests.
    #[doc(hidden)]
    pub fn sewer_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        sewer: PermanentId,
        hours: f64,
    ) {
        self.sewer_work(ctx, household, sewer, hours);
    }

    /// Whether sewer `si` of polity `pi` settles today (ADR-0041 §4): its keeper (a household, or
    /// the polity's store) holds a whole settling tank, and the content has tank and sludge goods;
    /// the share that settles.
    fn settles(&self, def: &SewerDef, pi: usize, si: usize) -> Option<f64> {
        let (tank, _) = (def.tank?, def.sludge?);
        let held = match self.polities[pi].sewers[si].keeper {
            Some(h) => self.household(h)?.stores.get(tank).copied(),
            None => self.polities[pi].stores.get(tank).copied(),
        }
        .unwrap_or(0.0);
        (held >= 1.0 - 1e-9).then_some(f64::from(def.settle_share).clamp(0.0, 1.0))
    }

    /// Homes beside sewer `si` of polity `pi`, whole and not broken, connect (ADR-0041 §3): each
    /// with people and a building, not connected to a sewer, where a connection is free and no law
    /// asks it to carry its heap, or where what laws ask of it to carry its heap a year exceeds a
    /// connection's work. Connections are made in the order of their households' ids.
    fn sewer_connect(&mut self, ctx: &Ctx, def: &SewerDef, pi: usize, si: usize, day: i64) {
        let s = &self.polities[pi].sewers[si];
        if !s.whole() || s.broken.is_some() {
            return;
        }
        let mut along: Vec<(PermanentId, PermanentId)> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| !x.members.is_empty() && self.polities[pi].holds(x.settlement))
            .filter(|x| Self::beside(ctx.map, &s.cells, cell_of(ctx.map, x.home) as u32))
            .filter(|x| !self.sewered(x.id))
            .filter(|x| {
                let carry = self.carry_h_year(ctx, x);
                if carry > 0.0 {
                    carry > f64::from(def.connect_h)
                } else {
                    def.connect_h <= 0.0
                }
            })
            .filter_map(|x| {
                let b = ctx.land.buildings.iter().find(|b| b.household == x.id)?;
                Some((x.id, b.id))
            })
            .collect();
        along.sort_unstable();
        for (household, building) in along {
            self.polities[pi].sewers[si]
                .connections
                .push(SewerConnection {
                    household,
                    building,
                    since: day,
                    day: -1,
                    kg: 0.0,
                    cause: SewerCause::Served,
                    unserved: [0; 3],
                });
        }
    }

    /// The cells field rectangle `rect` covers, as a range of columns and of rows.
    fn rect_cells(map: &WorldMap, rect: &RectCm) -> ((i64, i64), (i64, i64)) {
        let size = f64::from(map.cell_size_m) * 100.0;
        let span = |from: i32, len: i32, n: u32| {
            let n = i64::from(n.max(1)) - 1;
            let a = (f64::from(from) / size).floor() as i64;
            let b = ((f64::from(from) + f64::from(len.max(1)) - 1.0) / size).floor() as i64;
            (a.clamp(0, n), b.clamp(0, n))
        };
        (
            span(rect.x, rect.w, map.width),
            span(rect.y, rect.h, map.height),
        )
    }

    /// Whether the household working field `f` takes a sewer's flow onto it in calendar year
    /// `year`, for nothing (ADR-0044 §4): it has people, and the field's last harvest, of this
    /// year or the last, was held back by its soil. Once a harvest is not, it stops.
    fn would_farm(&self, f: &Field, year: i32) -> bool {
        self.household(f.household)
            .is_some_and(|x| !x.members.is_empty())
            && f.soil
                .record
                .last()
                .is_some_and(|r| r.limit == Limit::Soil && r.year >= year - 1)
    }

    /// The field sewer `si` of polity `pi` is led onto as a farm today (ADR-0044 §4), by its index
    /// among the land's fields: one with a cell beside the route below the lowest cell any
    /// connected home lies beside, so that the whole flow passes it, whose household would take
    /// the flow ([`Population::would_farm`]). The field it is led onto already while it still
    /// qualifies; or else the first the flow passes, the lowest id first. `None` where the content
    /// makes no farm, nobody is connected, or no field qualifies.
    fn sewer_farm(&self, ctx: &Ctx, def: &SewerDef, pi: usize, si: usize) -> Option<usize> {
        if def.farm_people_ha <= 0.0 {
            return None;
        }
        let s = &self.polities[pi].sewers[si];
        let map = ctx.map;
        let lowest = s
            .connections
            .iter()
            .filter_map(|c| {
                let home = cell_of(map, self.household(c.household)?.home) as u32;
                s.cells.iter().rposition(|&r| Self::beside(map, &[r], home))
            })
            .max()?;
        let below = s.cells.get(lowest + 1..).filter(|b| !b.is_empty())?;
        let w = i64::from(map.width.max(1));
        let below: Vec<(i64, i64)> = below
            .iter()
            .map(|&c| (i64::from(c) % w, i64::from(c) / w))
            .collect();
        let year = i32::try_from(ctx.now.date().year).unwrap_or(i32::MAX);
        let mut best: Option<(usize, PermanentId, usize)> = None;
        for (fi, f) in ctx.land.fields.iter().enumerate() {
            if !self.would_farm(f, year) {
                continue;
            }
            let ((x0, x1), (y0, y1)) = Self::rect_cells(map, &f.rect);
            let Some(at) = below.iter().position(|&(x, y)| {
                (x0 - 1..=x1 + 1).contains(&x) && (y0 - 1..=y1 + 1).contains(&y)
            }) else {
                continue;
            };
            if s.farm == Some(f.id) {
                return Some(fi);
            }
            if best.is_none_or(|b| (at, f.id) < (b.0, b.1)) {
                best = Some((at, f.id, fi));
            }
        }
        best.map(|b| b.2)
    }

    /// The day's sewers (ADR-0041 §2, §4; ADR-0044 §4, §5): homes beside each connect, then each
    /// connection in the order made gives what its heap would have gained of its decaying share
    /// today, while the sewer's room lasts: its capacity less the share of it its deposit takes
    /// (the deposit over what its length holds before it is blocked). Once one is turned away,
    /// every one connected after it is too, so the last connected back up first: for want of the
    /// capacity (`Overloaded`), or of room the deposit took (`Blocked`). A home whose drain enters
    /// at or above the break gives nothing either (`Broken`); those below it still flow. Each
    /// unserved connection counts the day, by cause. Of what goes in, the content's share stays as
    /// deposit (mass only); of the rest, a settling tank's share becomes sludge in its keeper's
    /// store; of what leaves the tank, its farm field takes up to its area's limit into its soil
    /// as dung; the rest goes to the river. Nothing happens where no polity lays a sewer.
    pub(super) fn sewers_day(&mut self, ctx: &mut Ctx, day: i64) {
        if self.polities.iter().all(|p| p.sewers.is_empty()) {
            return;
        }
        let Some(def) = Self::sewer_def(ctx).cloned() else {
            return;
        };
        let (people, soil): (&PeopleParams, &civ_land::SoilParams) =
            (ctx.params, &ctx.land_params.soil);
        let params = &people.midden;
        for pi in 0..self.polities.len() {
            for si in 0..self.polities[pi].sewers.len() {
                self.sewer_connect(ctx, &def, pi, si, day);
                let s = &self.polities[pi].sewers[si];
                if !s.whole() {
                    continue;
                }
                let capacity = f64::from(s.capacity_kg_day);
                let holds = f64::from(def.block_kg_m) * f64::from(s.length_m);
                let room = if holds > 0.0 {
                    capacity * (1.0 - s.deposit_kg / holds).clamp(0.0, 1.0)
                } else {
                    capacity
                };
                let broken = s.broken.map(|b| b as usize);
                let map = ctx.map;
                let connections: Vec<(PermanentId, Option<usize>)> = s
                    .connections
                    .iter()
                    .map(|c| {
                        let at = self
                            .household(c.household)
                            .and_then(|x| Self::entry(map, &s.cells, cell_of(map, x.home) as u32));
                        (c.household, at)
                    })
                    .collect();
                let year = day.div_euclid(DAYS_PER_YEAR);
                let mut entered = 0.0f64;
                let mut turned: Option<SewerCause> = None;
                for (ci, (h, at)) in connections.into_iter().enumerate() {
                    let gained = match self.household_mut(h) {
                        Some(x) if !x.members.is_empty() => {
                            let members = x.members.len();
                            x.midden.settle(ctx.now, members, params);
                            params.after(0.0, members, 1.0).min(x.midden.kg).max(0.0)
                        }
                        _ => 0.0,
                    };
                    let cause = if broken.is_some_and(|b| at.is_none_or(|a| a <= b)) {
                        SewerCause::Broken
                    } else if let Some(cause) = turned {
                        cause
                    } else if entered + gained > capacity + 1e-9 {
                        *turned.insert(SewerCause::Overloaded)
                    } else if entered + gained > room + 1e-9 {
                        *turned.insert(SewerCause::Blocked)
                    } else {
                        SewerCause::Served
                    };
                    let kg = if cause == SewerCause::Served {
                        gained
                    } else {
                        0.0
                    };
                    if kg > 0.0
                        && let Some(x) = self.household_mut(h)
                    {
                        x.midden.kg -= kg;
                        entered += kg;
                        self.changed(h);
                    }
                    let c = &mut self.polities[pi].sewers[si].connections[ci];
                    // A new year counts afresh; a day counts once for each cause it ends in.
                    if c.day >= 0 && c.day.div_euclid(DAYS_PER_YEAR) != year {
                        c.unserved = [0; 3];
                    }
                    if cause != SewerCause::Served && (c.day != day || c.cause != cause) {
                        let k = usize::from(cause.code() - 1);
                        c.unserved[k] = c.unserved[k].saturating_add(1);
                    }
                    (c.day, c.kg, c.cause) = (day, kg, cause);
                }
                // A share stays as deposit, mass only (ADR-0044 §5).
                let deposited = entered * f64::from(def.deposit_share).clamp(0.0, 1.0);
                self.polities[pi].sewers[si].deposit_kg += deposited;
                let flow = entered - deposited;
                let settled = flow * self.settles(&def, pi, si).unwrap_or(0.0);
                if settled > 0.0
                    && let Some(sludge) = def.sludge
                {
                    self.give_keeper(ctx, (pi, si), sludge, settled);
                }
                // The farm takes what is left up to its area's limit, into its soil as dung.
                let farm = self.sewer_farm(ctx, &def, pi, si);
                let mut farm_kg = 0.0;
                if let Some(fi) = farm {
                    let f = &mut ctx.land.fields[fi];
                    let ha = f.area_ha();
                    let limit =
                        f64::from(def.farm_people_ha).max(0.0) * ha * params.kg_per_person_day;
                    farm_kg = (flow - settled).min(limit).max(0.0);
                    if farm_kg > 0.0 && ha > 0.0 {
                        f.soil.manured(soil, farm_kg * params.n_per_kg() / ha);
                    }
                }
                let farm = farm.map(|fi| ctx.land.fields[fi].id);
                let s = &mut self.polities[pi].sewers[si];
                (s.day, s.entered_kg, s.settled_kg) = (day, entered, settled);
                s.deposited_kg = deposited;
                (s.farm, s.farm_kg) = (farm, farm_kg);
            }
        }
    }

    /// What the people of homes served by a sewer today shed into their heaps today goes with
    /// their share to the outfall (ADR-0041 §2, §4), recorded as carried: a settling tank's share
    /// of each disease to the tank's node; of the rest, its farm's share of the day's kilograms
    /// leaving the tank to the farm field's node (ADR-0044 §4); the rest to the outfall's reach and
    /// on down the river. A deposit holds mass only: the loads pass with the flow (ADR-0044 §5).
    /// `shed` are today's moves from those shedding into heaps.
    pub(super) fn sewer_loads(&mut self, ctx: &Ctx, day: i64, shed: &[Move]) {
        if shed.is_empty() || self.polities.iter().all(|p| p.sewers.is_empty()) {
            return;
        }
        let Some(def) = Self::sewer_def(ctx).cloned() else {
            return;
        };
        for m in shed {
            let Node::Midden(h) = m.to else {
                continue;
            };
            let Some((pi, si)) = self.polities.iter().enumerate().find_map(|(pi, p)| {
                p.sewers
                    .iter()
                    .position(|s| {
                        s.connections.iter().any(|c| {
                            c.household == h && c.day == day && c.cause == SewerCause::Served
                        })
                    })
                    .map(|si| (pi, si))
            }) else {
                continue;
            };
            let (id, reach, farm, farm_share) = {
                let s = &self.polities[pi].sewers[si];
                let rest = s.entered_kg - s.deposited_kg - s.settled_kg;
                let share = if rest > 0.0 {
                    (s.farm_kg / rest).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (s.id, s.reach, s.farm, share)
            };
            let from = Node::Midden(h);
            let amount = self.contagion.take_amount(from, m.disease, m.amount);
            if amount <= 0.0 {
                continue;
            }
            let settled = amount * self.settles(&def, pi, si).unwrap_or(0.0);
            if settled > 0.0 {
                let to = Node::Sewer(id);
                self.contagion.add(to, m.disease, settled);
                if settled > NEGLIGIBLE {
                    self.contagion.record(Move {
                        day,
                        disease: m.disease,
                        how: How::Carried,
                        from,
                        to,
                        amount: settled,
                    });
                }
            }
            let mut farmed = 0.0;
            if let Some(field) = farm.filter(|_| farm_share > 0.0) {
                farmed = (amount - settled) * farm_share;
                let to = Node::Field(field);
                self.contagion.add(to, m.disease, farmed);
                if farmed > NEGLIGIBLE {
                    self.contagion.record(Move {
                        day,
                        disease: m.disease,
                        how: How::Carried,
                        from,
                        to,
                        amount: farmed,
                    });
                }
            }
            self.deliver(
                ctx,
                Move {
                    day,
                    disease: m.disease,
                    how: How::Carried,
                    from,
                    to: Node::Reach(reach),
                    amount: amount - settled - farmed,
                },
            );
        }
    }

    /// The metres along sewer `s`'s route from its cell `i` down to its outfall.
    fn above_outfall_m(map: &WorldMap, s: &Sewer, i: usize) -> f64 {
        let runs = civ_land::rail::runs(&s.cells, map.width, f64::from(map.cell_size_m));
        runs.iter().skip(i + 1).sum()
    }

    /// How many of sewer `s`'s connected homes have their drains enter at or above its route
    /// cell `b`, so that a break there backs them up (as [`Population::sewers_day`] reckons it).
    fn homes_above(&self, map: &WorldMap, s: &Sewer, b: usize) -> usize {
        s.connections
            .iter()
            .filter_map(|c| self.household(c.household))
            .filter(|x| {
                Self::entry(map, &s.cells, cell_of(map, x.home) as u32).is_none_or(|a| a <= b)
            })
            .count()
    }

    /// The chronicle's word that sewer `si` of polity `pi` fell in at its route cell `at`
    /// (ADR-0044 §5), worn through this month: how far above its outfall, and either how many
    /// homes whose drains enter above it now back up or that it was already broken below.
    fn tell_sewer_collapse(&mut self, ctx: &Ctx, (pi, si): (usize, usize), at: usize, broke: bool) {
        let map = ctx.map;
        let settlement = self.polities[pi].settlement;
        let s = &self.polities[pi].sewers[si];
        let up = Self::above_outfall_m(map, s, at);
        let homes = self.homes_above(map, s, at);
        let village = self.village_name(ctx, Some(settlement));
        let mut sentence = format!(
            "{village}'s sewer fell in {up:.0} m above its outfall, its walls worn through"
        );
        if !broke {
            sentence.push_str(", above where it had already fallen in.");
        } else if homes > 0 {
            sentence.push_str(&format!(
                ": {homes} home{} whose drains enter above it back up.",
                if homes == 1 { "" } else { "s" }
            ));
        } else {
            sentence.push('.');
        }
        let place = s
            .cells
            .get(at)
            .map(|&c| crate::population::cell_centre(map, c as usize));
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Sewer,
            Vec::new(),
            Some(settlement),
            place,
            f64::from(SewerStep::Collapsed as u8),
            sentence,
        );
    }

    /// The chronicle's word that polity `pi`'s review asked mending of its sewer `si`: relining
    /// the metres of it that fell in, clearing the silt that backs homes up, or both, and the
    /// hours asked of its households (backlog C's "Not yet"; as a passing place grown over is told
    /// when its clearing is asked).
    fn tell_sewer_mending_asked(
        &mut self,
        ctx: &Ctx,
        (pi, si): (usize, usize),
        reline: bool,
        clear_kg: f64,
        hours: f64,
    ) {
        let map = ctx.map;
        let settlement = self.polities[pi].settlement;
        let s = &self.polities[pi].sewers[si];
        let village = self.village_name(ctx, Some(settlement));
        let fallen = (Self::collapsed_metres(map, s) - f64::from(s.relined_m)).max(0.0);
        let mut what = Vec::new();
        if reline {
            what.push(format!("reline the {fallen:.0} m of it that fell in"));
        }
        if clear_kg > 0.0 {
            what.push(format!("clear {:.1} t of silt from it", clear_kg / 1000.0));
        }
        let sentence = format!(
            "{village}'s gathering asked its households to {} ({hours:.0} hours).",
            what.join(" and ")
        );
        let place = s
            .broken
            .and_then(|b| s.cells.get(b as usize))
            .or_else(|| s.cells.first())
            .map(|&c| crate::population::cell_centre(map, c as usize));
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Sewer,
            Vec::new(),
            Some(settlement),
            place,
            f64::from(SewerStep::MendingAsked as u8),
            sentence,
        );
    }

    /// A month of wear on every whole sewer (ADR-0044 §5): each cell of its route loses a twelfth
    /// of `1 / life_years` times its ground's wetness and the month's rain against its usual, as a
    /// well's lining does (ADR-0012 §5's factors); a cell whose loss reaches 1 collapses, and the
    /// sewer is broken at the collapse nearest its outfall. A sewer with a cell that collapsed this
    /// month is told in the chronicle once, at the lowest of them (backlog D, fifth item).
    pub(super) fn sewers_month(&mut self, ctx: &Ctx) {
        if self.polities.iter().all(|p| p.sewers.is_empty()) {
            return;
        }
        let Some(life) = Self::sewer_def(ctx)
            .map(|d| f64::from(d.life_years))
            .filter(|&l| l > 0.0)
        else {
            return;
        };
        let rain = month_rain(ctx);
        let mut fell: Vec<(usize, usize, usize, bool)> = Vec::new();
        for (pi, p) in self.polities.iter_mut().enumerate() {
            for (si, s) in p.sewers.iter_mut().enumerate().filter(|(_, s)| s.whole()) {
                s.loss.resize(s.cells.len(), 0.0);
                let mut lowest = None;
                for (i, (l, &c)) in s.loss.iter_mut().zip(&s.cells).enumerate() {
                    let was = *l;
                    let lost = rain * wetness_at(ctx, c) / life / 12.0;
                    *l = (*l + lost as f32).min(1.0);
                    if was < 1.0 && *l >= 1.0 {
                        lowest = Some(i);
                    }
                }
                let broken = s.lowest_collapse();
                // A collapse nearer the outfall is now the break; relining starts on it afresh.
                if broken != s.broken {
                    s.relined_m = 0.0;
                }
                let broke = broken != s.broken;
                s.broken = broken;
                if let Some(at) = lowest {
                    fell.push((pi, si, at, broke));
                }
            }
        }
        for (pi, si, at, broke) in fell {
            self.tell_sewer_collapse(ctx, (pi, si), at, broke);
        }
    }

    /// A month of wear on every whole sewer now: for tests.
    #[doc(hidden)]
    pub fn sewers_month_for_tests(&mut self, ctx: &mut Ctx) {
        self.sewers_month(ctx);
    }

    /// Polity `pi`'s review weighs the faults of its sewers it has learned of (ADR-0044 §5), each
    /// only while the law that laid it is in force: a collapse, or a home backing up for want of
    /// room the sewer once had (its last day's cause `Blocked`); nothing else is seen. It weighs a
    /// fault as laying was weighed: the hours a year the homes it backs up would be spared
    /// ([`Population::sewer_spared_h`]) over the sewer's life left (its life less its most worn
    /// sound cell's loss), against the work and materials. Relining takes laying's hours and
    /// walls and binder a metre of the collapsed cells, the materials at the cheapest any of its
    /// households can have them; clearing takes `clear_h_t` hours a tonne of the deposit. Where
    /// worth it, the polity asks the work as laying's was asked: the hours join the law's work,
    /// an equal share of each household, given as they choose and paid where a law pays for
    /// common work.
    pub(super) fn sewer_reviews(&mut self, ctx: &Ctx, pi: usize) {
        if self.polities[pi].sewers.is_empty() {
            return;
        }
        let Some(def) = Self::sewer_def(ctx) else {
            return;
        };
        let polity = &self.polities[pi];
        let in_force = |law: PermanentId| {
            polity
                .laws
                .iter()
                .any(|l| l.id == law && l.status == LawStatus::InForce)
        };
        let faulted: Vec<usize> = polity
            .sewers
            .iter()
            .enumerate()
            .filter(|(_, s)| s.whole() && in_force(s.law))
            .filter(|(_, s)| {
                (s.broken.is_some() && !s.reline)
                    || (s.clear_kg <= 0.0
                        && s.deposit_kg > 0.0
                        && s.connections
                            .iter()
                            .any(|c| c.day >= 0 && c.cause == SewerCause::Blocked))
            })
            .map(|(si, _)| si)
            .collect();
        if faulted.is_empty() {
            return;
        }
        let mut ours: Vec<&Household> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| polity.holds(x.settlement) && !x.members.is_empty())
            .collect();
        ours.sort_unstable_by_key(|x| x.id);
        let ids: Vec<PermanentId> = ours.iter().map(|x| x.id).collect();
        let deaths = self.deaths_of_disease(ctx, &ids);
        let costs: Vec<Vec<Option<f64>>> = ours.iter().map(|x| self.own_costs_of(ctx, x)).collect();
        // (sewer, reline, clear kg, hours asked)
        let mut asks: Vec<(usize, bool, f64, f64)> = Vec::new();
        for si in faulted {
            let s = &polity.sewers[si];
            let spared = |cause: SewerCause| -> f64 {
                s.connections
                    .iter()
                    .filter(|c| c.day >= 0 && c.cause == cause)
                    .filter_map(|c| self.household(c.household))
                    .filter(|x| !x.members.is_empty())
                    .map(|x| self.sewer_spared_h(ctx, def, x, &deaths))
                    .sum()
            };
            let life_left = f64::from(def.life_years)
                * (1.0 - s.worst().map_or(0.0, |w| f64::from(w.1))).max(0.0);
            let (mut reline, mut clear, mut hours) = (false, 0.0, 0.0);
            if s.broken.is_some() && !s.reline {
                let metres = Self::collapsed_metres(ctx.map, s) - f64::from(s.relined_m);
                let labour = metres.max(0.0) * f64::from(def.lay_h_m);
                let walls = Self::material_h(
                    &ours,
                    &costs,
                    &def.walls,
                    metres * f64::from(def.walls_kg_m),
                );
                let binder = Self::material_h(
                    &ours,
                    &costs,
                    &def.binders,
                    metres * f64::from(def.binder_kg_m),
                );
                if let (Some(walls), Some(binder)) = (walls, binder)
                    && spared(SewerCause::Broken) * life_left - labour - walls - binder > 0.0
                {
                    (reline, hours) = (true, labour);
                }
            }
            if s.clear_kg <= 0.0
                && s.deposit_kg > 0.0
                && def.clear_h_t > 0.0
                && s.connections
                    .iter()
                    .any(|c| c.day >= 0 && c.cause == SewerCause::Blocked)
            {
                let labour = s.deposit_kg / 1000.0 * f64::from(def.clear_h_t);
                if spared(SewerCause::Blocked) * life_left - labour > 0.0 {
                    clear = s.deposit_kg;
                    hours += labour;
                }
            }
            if reline || clear > 0.0 {
                asks.push((si, reline, clear, hours));
            }
        }
        for &(si, reline, clear, hours) in &asks {
            self.tell_sewer_mending_asked(ctx, (pi, si), reline, clear, hours);
        }
        for (si, reline, clear, hours) in asks {
            let p = &mut self.polities[pi];
            let s = &mut p.sewers[si];
            if reline {
                s.reline = true;
            }
            if clear > 0.0 {
                s.clear_kg = clear;
            }
            let (law, id, head) = (s.law, s.id, s.cells.first().copied().unwrap_or(0));
            match p.ways.iter_mut().find(|w| w.0 == law) {
                Some(w) => w.1.labour_h += hours,
                None => p.ways.push((
                    law,
                    WaySite {
                        cell: head,
                        labour_h: hours,
                        work: Some(id),
                    },
                )),
            }
        }
    }

    /// Polity `pi`'s review of its sewers' faults now: for tests.
    #[doc(hidden)]
    pub fn sewer_reviews_for_tests(&mut self, ctx: &mut Ctx, pi: usize) {
        self.sewer_reviews(ctx, pi);
    }

    /// The day's sewers now, with `shed` (household, disease, doses) shed into heaps today as
    /// shedding adds it, and carried as the day's turn carries it: for tests.
    #[doc(hidden)]
    pub fn sewer_day_for_tests(&mut self, ctx: &mut Ctx, shed: &[(PermanentId, u16, f64)]) {
        let day = ctx.now.day_index();
        self.sewers_day(ctx, day);
        let moves: Vec<Move> = shed
            .iter()
            .map(|&(h, disease, amount)| Move {
                day,
                disease,
                how: How::Shed,
                from: Node::Midden(h),
                to: Node::Midden(h),
                amount,
            })
            .collect();
        for m in &moves {
            self.contagion.add(m.to, m.disease, m.amount);
        }
        self.sewer_loads(ctx, day, &moves);
    }
}
