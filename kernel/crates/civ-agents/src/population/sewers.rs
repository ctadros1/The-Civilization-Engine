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

use super::*;
use crate::contagion::{How, Move, NEGLIGIBLE, Node};
use crate::polity::{LawStatus, PolicyKind, Sewer, SewerCause, SewerConnection, SewerDef, WaySite};
use crate::sickness::Outcome;
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
        let material = |goods: &[usize], kg: f64| -> Option<f64> {
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
        };
        let ids: Vec<PermanentId> = ours.iter().map(|x| x.id).collect();
        let deaths = self.deaths_of_disease(ctx, &ids);
        let day_h = ctx.params.farm.work_hours_per_day;
        let homes: Vec<(PermanentId, u32, f64)> = ours
            .iter()
            .filter(|x| {
                !self.sewered(x.id) && ctx.land.buildings.iter().any(|b| b.household == x.id)
            })
            .map(|x| {
                let died = deaths.iter().find(|d| d.0 == x.id).map_or(0, |d| d.1);
                let spared = self.carry_h_year(ctx, x)
                    + died as f64
                        * f64::from(def.avoided_share)
                        * ctx.params.suspicion.w_death
                        * day_h;
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
            broken: false,
            keeper: None,
            connections: Vec::new(),
            day: -1,
            entered_kg: 0.0,
            settled_kg: 0.0,
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
    /// laid and the household holds a metre's walls and binder. `None` otherwise.
    pub(super) fn sewer_left_h(&self, ctx: &Ctx, x: &Household, sewer: PermanentId) -> Option<f64> {
        let def = Self::sewer_def(ctx)?;
        let (pi, si) = self.sewer_at(sewer)?;
        let s = &self.polities[pi].sewers[si];
        if s.whole() || s.broken {
            return None;
        }
        let walls = Self::first_held(x, &def.walls)?.1;
        let binder = Self::first_held(x, &def.binders)?.1;
        if walls < f64::from(def.walls_kg_m) || binder < f64::from(def.binder_kg_m) {
            return None;
        }
        Some(f64::from((s.length_m - s.laid_m).max(0.0)) * f64::from(def.lay_h_m))
    }

    /// `hours` of work by someone of `household` on sewer `sewer` its polity lays (ADR-0041 §1): a
    /// metre dug, walled and filled in each `lay_h_m` hours, each taking its walls and binder from
    /// the household's store (the first of each kind it holds), no more than they allow or the
    /// sewer has left. The hours it took are the household's given to the law, and to the polity's
    /// works, paid where a law of pay for common work is in force.
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
        let left = {
            let s = &self.polities[pi].sewers[si];
            if s.whole() || s.broken {
                return;
            }
            f64::from((s.length_m - s.laid_m).max(0.0))
        };
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
        let took = metres * f64::from(def.lay_h_m);
        self.polities[pi].give_to_way(law, household, took as f32);
        self.work_given.push((polity, household, took));
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
        if !s.whole() || s.broken {
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
                });
        }
    }

    /// The day's sewers (ADR-0041 §2, §4): homes beside each connect, then each connection in the
    /// order made gives what its heap would have gained of its decaying share today, while the
    /// sewer's capacity lasts; a broken sewer and those beyond its capacity give nothing and keep
    /// their cause. Of what goes in, a settling tank's share becomes sludge in its keeper's store;
    /// the rest goes to the river. Nothing happens where no polity lays a sewer.
    pub(super) fn sewers_day(&mut self, ctx: &Ctx, day: i64) {
        if self.polities.iter().all(|p| p.sewers.is_empty()) {
            return;
        }
        let Some(def) = Self::sewer_def(ctx).cloned() else {
            return;
        };
        let params = &ctx.params.midden;
        for pi in 0..self.polities.len() {
            for si in 0..self.polities[pi].sewers.len() {
                self.sewer_connect(ctx, &def, pi, si, day);
                let s = &self.polities[pi].sewers[si];
                if !s.whole() {
                    continue;
                }
                let (broken, capacity) = (s.broken, f64::from(s.capacity_kg_day));
                let connections: Vec<PermanentId> =
                    s.connections.iter().map(|c| c.household).collect();
                let mut entered = 0.0f64;
                for (ci, h) in connections.into_iter().enumerate() {
                    let gained = match self.household_mut(h) {
                        Some(x) if !x.members.is_empty() => {
                            let members = x.members.len();
                            x.midden.settle(ctx.now, members, params);
                            params.after(0.0, members, 1.0).min(x.midden.kg).max(0.0)
                        }
                        _ => 0.0,
                    };
                    let cause = if broken {
                        SewerCause::Broken
                    } else if entered + gained > capacity + 1e-9 {
                        SewerCause::Overloaded
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
                    (c.day, c.kg, c.cause) = (day, kg, cause);
                }
                let settled = entered * self.settles(&def, pi, si).unwrap_or(0.0);
                if settled > 0.0
                    && let Some(sludge) = def.sludge
                {
                    let keeper = self.polities[pi].sewers[si].keeper;
                    let store = match keeper {
                        Some(h) => self.household_mut(h).map(|x| &mut x.stores),
                        None => Some(&mut self.polities[pi].stores),
                    };
                    if let Some(store) = store {
                        if store.len() <= sludge {
                            store.resize(ctx.catalog.goods.len().max(sludge + 1), 0.0);
                        }
                        store[sludge] += settled;
                    }
                }
                let s = &mut self.polities[pi].sewers[si];
                (s.day, s.entered_kg, s.settled_kg) = (day, entered, settled);
            }
        }
    }

    /// What the people of homes served by a sewer today shed into their heaps today goes with
    /// their share to the outfall (ADR-0041 §2, §4), recorded as carried: a settling tank's share
    /// of each disease to the tank's node, the rest to the outfall's reach and on down the river.
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
            let (id, reach) = {
                let s = &self.polities[pi].sewers[si];
                (s.id, s.reach)
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
            self.deliver(
                ctx,
                Move {
                    day,
                    disease: m.disease,
                    how: How::Carried,
                    from,
                    to: Node::Reach(reach),
                    amount: amount - settled,
                },
            );
        }
    }

    /// The day's sewers now, with `shed` (household, disease, doses) shed into heaps today as
    /// shedding adds it, and carried as the day's turn carries it: for tests.
    #[doc(hidden)]
    pub fn sewer_day_for_tests(&mut self, ctx: &Ctx, shed: &[(PermanentId, u16, f64)]) {
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
