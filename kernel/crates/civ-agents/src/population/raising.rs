//! Buildings a polity raises together (M8d slice DC; ADR-0031 §6), on the shape of a passing
//! place cleared together (M8b slice CQ, third part). A program answers a loss the polity
//! measured (a storehouse its common store's spoiling): where one is measured and no building of
//! the program is raised or being raised, each household weighs its share of the loss averted
//! against its equal share of the work over the years it weighs, and someone who knows the
//! program's technique may propose it. Passed, it is laid out near the hearth, held in the
//! polity's name rather than a household's; its work is each household's to give by choice, as for a
//! passing place, the hours including making and carrying its materials. A storehouse finished
//! keeps the common store under its roof.

use super::*;
use crate::decide::RaiseFacts;
use crate::polity::{LawStatus, PolicyKind};

/// What raising a building together would bring the households of a settlement (M8d slice DC):
/// its template and program, the hours of a capable adult's work it takes, the years it is
/// weighed over, the food a year each household believes it would avert, and how many households
/// share the work.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PublicRaise {
    pub policy: u16,
    pub program: usize,
    pub labour_h: f64,
    pub life: f64,
    pub averted_kcal: f64,
    pub households: usize,
}

impl PublicRaise {
    /// Each household's equal share of the work, hours.
    pub fn share_h(&self) -> f64 {
        self.labour_h / self.households.max(1) as f64
    }
}

impl Population {
    /// The food a year polity `pi`'s common store lost to spoiling, kcal, over the years it
    /// measured (M8d slice DC).
    fn spoiled_a_year(&self, pi: usize) -> f64 {
        let m = &self.polities[pi].measured;
        if m.is_empty() {
            return 0.0;
        }
        m.iter().map(|y| y.spoiled_kcal).sum::<f64>() / m.len() as f64
    }

    /// The loss a year a building of program `program` would answer for polity `pi`, kcal: a
    /// storehouse its common store's spoiling; nothing measured for any other yet.
    fn loss_answered(&self, ctx: &Ctx, pi: usize, program: usize) -> f64 {
        match ctx.catalog.buildings.get(program).map(|d| d.use_) {
            Some(civ_land::PlotUse::Store) => self.spoiled_a_year(pi),
            _ => 0.0,
        }
    }

    /// Whether polity `pi` has a building of program `program` raised or being raised.
    fn raised(&self, ctx: &Ctx, pi: usize, program: usize) -> bool {
        let polity = self.polities[pi].id;
        ctx.land.buildings.iter().any(|b| {
            b.polity == Some(polity)
                && b.standing()
                && ctx.catalog.building_index(&b.spec.program) == Some(program)
        }) || self.polities[pi].laws.iter().any(|l| {
            l.kind == PolicyKind::RaiseBuilding
                && matches!(l.status, LawStatus::Proposed | LawStatus::InForce)
                && self.polities[pi]
                    .raise_site_of(l.id)
                    .is_some_and(|s| usize::from(s.program) == program && s.building.is_none())
        })
    }

    /// Whether the common store's spoiling is an issue for polity `pi` (M8d slice DC): measured,
    /// and no storehouse of its raised or being raised.
    pub(super) fn store_spoils(&self, ctx: &Ctx, pi: usize) -> bool {
        self.spoiled_a_year(pi) > 0.0
            && !ctx.catalog.policies.iter().any(|d| {
                d.raise.is_some_and(|r| {
                    ctx.catalog.buildings.get(r.program).map(|b| b.use_)
                        == Some(civ_land::PlotUse::Store)
                        && self.raised(ctx, pi, r.program)
                })
            })
    }

    /// What raising each building its templates name would bring polity `pi`'s households
    /// `households` (M8d slice DC): one for each whose loss is measured and none of whose
    /// program is raised or being raised.
    pub(crate) fn public_raises(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> Vec<PublicRaise> {
        let rp = &ctx.params.rules;
        let carry = ctx.params.household.carry_kg;
        let mut out = Vec::new();
        for (k, d) in ctx.catalog.policies.iter().enumerate() {
            let Some(r) = d.raise.filter(|_| d.kind == PolicyKind::RaiseBuilding) else {
                continue;
            };
            let loss = self.loss_answered(ctx, pi, r.program);
            if loss <= 0.0 || self.raised(ctx, pi, r.program) {
                continue;
            }
            let Some(labour_h) = ctx
                .catalog
                .buildings
                .get(r.program)
                .and_then(|b| b.shapes.first())
                .map(|c| c.hours(carry))
            else {
                continue;
            };
            out.push(PublicRaise {
                policy: k as u16,
                program: r.program,
                labour_h,
                life: rp.horizon_years,
                averted_kcal: loss * rp.avoided_share / households.len().max(1) as f64,
                households: households.len(),
            });
        }
        out
    }

    /// Polity `pi` lays out the building its law `law` raises, as its law passed would (M8d slice
    /// DC): near the hearth, on clear ground level enough to build on as it is, plot and building
    /// held in the polity's name. Nothing where no ground is clear.
    pub(super) fn begin_public_raise(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let Some(site) = self.polities[pi].raise_site_of(law) else {
            return;
        };
        if site.building.is_some() {
            return;
        }
        let Some(def) = ctx.catalog.buildings.get(usize::from(site.program)) else {
            return;
        };
        let Some(shape) = def.shapes.first().map(|c| c.shape) else {
            return;
        };
        let settlement = self.polities[pi].settlement;
        let Some(hearth) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map(|s| s.hearth_m)
        else {
            return;
        };
        // Held in the polity's own name, so that no household keeps, stores in, inherits or keeps
        // up the building (ADR-0031 §6: its upkeep falls to nobody unless a law asks it).
        let keeper = self.polities[pi].id;
        let goods = &ctx.catalog.goods;
        let design_at = |at: (f32, f32)| build::design_shape(def, goods, shape, at, Some(hearth));
        let lev = &ctx.params.build.levelling;
        let bed = |x: f64, y: f64| civ_land::earth::bed_height(ctx.map, (x, y));
        let level = |rect: &civ_land::RectCm| {
            (civ_land::earth::drop_across(rect, &bed) <= lev.from_m).then_some(0.0)
        };
        let Some(spec) = build::home_site(
            ctx.land,
            ctx.map,
            ctx.nav,
            def,
            &design_at,
            hearth,
            Some(hearth),
            RAISE_WITHIN_M,
            &level,
        ) else {
            return;
        };
        let rect = build::plot_rect(&spec, def);
        let (plot, id) = (ctx.ids.allocate(), ctx.ids.allocate());
        ctx.land.plots.push(civ_land::Plot {
            id: plot,
            household: keeper,
            rect,
            use_: def.use_,
            since: ctx.now,
            receipt: None,
            allotted: None,
        });
        ctx.land.buildings.push(Building {
            polity: Some(self.polities[pi].id),
            ..Building::new(id, keeper, plot, spec, ctx.now)
        });
        if let Some(s) = self.polities[pi].raises.iter_mut().find(|s| s.0 == law) {
            s.1.building = Some(id);
        }
    }

    /// Polity `pi` lays out the building its law `law` raises: for tests.
    #[doc(hidden)]
    pub fn begin_public_raise_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.begin_public_raise(ctx, pi, law);
    }

    /// The building household `x`'s people may help raise, if it is light (M8d slice DC): one
    /// their polity raises under a law in force `person` knows of, not yet finished; where its
    /// work is done, the walk there from home, the hours of a capable adult's work it still
    /// takes, and, while the household has not given its equal share, what keeping to the
    /// gathering's word is worth to `person`, as for a passing place cleared together.
    pub(super) fn raise_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        person: PermanentId,
        dark: bool,
    ) -> Option<RaiseFacts> {
        if dark {
            return None;
        }
        let pi = self.polity_of(x.settlement?)?;
        let polity = &self.polities[pi];
        let (law, site, b) = polity.raises.iter().find_map(|(l, site)| {
            let law = polity.laws.iter().find(|k| {
                k.id == *l
                    && k.status == LawStatus::InForce
                    && k.kind == PolicyKind::RaiseBuilding
                    && k.knows(person)
            })?;
            let b = ctx
                .land
                .buildings
                .iter()
                .find(|b| Some(b.id) == site.building && !b.finished() && b.standing())?;
            Some((law, *site, b))
        })?;
        let given: f64 = polity
            .raise_given
            .iter()
            .filter(|g| g.0 == law.id)
            .map(|g| f64::from(g.2))
            .sum();
        let households = self
            .households
            .iter()
            .filter(|(_, y)| y.settlement == x.settlement && !y.members.is_empty())
            .count()
            .max(1);
        let share = site.labour_h / households as f64;
        let mut duty = 0.0;
        if f64::from(polity.raise_given_by(law.id, x.id)) + 1e-6 < share {
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == person)
                .map_or(0.0, |r| r.stance.sign());
            let regard =
                self.ties
                    .regard(person, law.sponsor, ctx.now.day_index(), &ctx.params.ties);
            let norm = self.norm_points(ctx, person);
            duty = crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity)
                .max(0.0);
        }
        let at = build::centre_m(&b.spec);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        Some(RaiseFacts {
            building: b.id,
            at,
            walk_min: f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1)) / pace / 60.0,
            left_h: (site.labour_h - given).max(0.0),
            duty,
        })
    }

    /// `hours` of a capable adult's work by someone of `household` on building `building` its
    /// polity raises (M8d slice DC): the stage under way rises by them, its materials made and
    /// carried among the hours (each stage's labour scaled by its program's carrying), and the
    /// hours are the household's given to the law and to the polity's works. A storehouse
    /// finished keeps the common store under its roof.
    pub(super) fn raise_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        building: PermanentId,
        hours: f64,
    ) {
        let now = ctx.now;
        let Some(bi) = ctx.land.buildings.iter().position(|b| b.id == building) else {
            return;
        };
        let Some(polity) = ctx.land.buildings[bi].polity else {
            return;
        };
        let Some(pi) = self
            .household(household)
            .and_then(|x| self.polity_of(x.settlement?))
            .filter(|&pi| self.polities[pi].id == polity)
        else {
            return;
        };
        let Some(law) = self.polities[pi]
            .raises
            .iter()
            .find(|s| s.1.building == Some(building))
            .map(|s| s.0)
        else {
            return;
        };
        let catalog = ctx.catalog;
        let Some(def) = catalog
            .building_index(&ctx.land.buildings[bi].spec.program)
            .and_then(|i| catalog.buildings.get(i))
        else {
            return;
        };
        if ctx.land.buildings[bi].finished() {
            return;
        }
        let Some(needs) = self.needs_of(&ctx.land.buildings[bi], def, 0.0) else {
            return;
        };
        let carry = ctx.params.household.carry_kg;
        let scale = def
            .shapes
            .first()
            .filter(|c| c.labour_h > 0.0)
            .map_or(1.0, |c| c.hours(carry) / c.labour_h);
        let work = build::HomeWork::of(&ctx.land.buildings[bi], needs);
        let Some(stage) = work.needs() else {
            return;
        };
        let b = &mut ctx.land.buildings[bi];
        let done = b.work(
            hours,
            stage.labour_h * scale,
            &stage.materials_kg,
            &stage.materials_kg,
            now,
        );
        b.skill_h += (done.hours * condition::MIDDLING_SKILL) as f32;
        if let Some(finished) = done.finished {
            if let Ok(e) = civ_grammar::expand(&b.spec, &def.rules) {
                let spread = ctx.params.build.quality_spread;
                condition::install(
                    b,
                    &e,
                    finished,
                    condition::MIDDLING_SKILL,
                    ctx.seed,
                    spread,
                    now,
                );
            }
            b.skill_h = 0.0;
        }
        let finished = b.finished();
        if done.hours > 0.0 {
            self.polities[pi].give_to_raise(law, household, done.hours as f32);
            self.work_given.push((polity, household, done.hours));
        }
        if finished && def.use_ == civ_land::PlotUse::Store {
            self.polities[pi].storehouse = Some(building);
        }
    }

    /// `hours` of work by someone of `household` on building `building` its polity raises: for
    /// tests.
    #[doc(hidden)]
    pub fn raise_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        building: PermanentId,
        hours: f64,
    ) {
        self.raise_work(ctx, household, building, hours);
    }

    /// Each polity's storehouse is looked over (monthly): one no longer standing finished keeps
    /// the store no more (M8d slice DC).
    pub(super) fn look_over_storehouses(&mut self, ctx: &Ctx) {
        for p in &mut self.polities {
            if let Some(id) = p.storehouse
                && !ctx
                    .land
                    .buildings
                    .iter()
                    .any(|b| b.id == id && b.finished() && b.standing())
            {
                p.storehouse = None;
            }
        }
    }
}

/// How far from the hearth a polity looks for clear ground for a building it raises, metres.
pub const RAISE_WITHIN_M: f64 = 80.0;
