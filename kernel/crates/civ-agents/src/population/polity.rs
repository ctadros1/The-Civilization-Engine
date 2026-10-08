//! The polity at work (ADR-0013): each settlement's polity founded under the custom every world
//! shares, the weekly review in which those who may propose weigh their moves, the gathering at
//! the hearth that decides, the levy at threshing and relief from the store, and word of a law
//! going round. Every choice here is a person's, scored by what they forecast for their household
//! and those who regard them; no issue weighs toward any policy.

use super::*;
use crate::decide::GatheringFacts;
use crate::polity::{
    Belief, Body, CustomVersion, Deliberator, Gathering, IssueKind, Law, LawStatus, Membership,
    MoveOption, Outcome, Outlook, PolicyKind, Polity, RuleDeliberator, Stance, StanceRecord,
};
use crate::word::Blamed;

/// A decision someone saw made at a gathering (M4c slice AF): each stance taken, by person and
/// household, with what the household stood to gain in points, and whether it passed.
struct Seen {
    stances: Vec<(PermanentId, PermanentId, Stance, f64)>,
    passed: bool,
}

/// What a settlement's households expect (ADR-0013 §5), for forecasting what a move would bring
/// each: built once per review or month.
pub(super) struct Forecasts {
    /// Each household's outlook, by household id.
    pub(super) outlooks: Vec<(PermanentId, Outlook)>,
    /// What the settlement's people believe of their years.
    belief: Belief,
    /// What a roof over the store would keep each household a year, kcal.
    kept: f64,
    /// What each household knows of takings in the last year, by household id.
    takings: Vec<(PermanentId, crate::crime::TakingsKnown)>,
    /// Grown members of each household, whom a curfew keeps at home.
    grown: BTreeMap<PermanentId, u32>,
}

impl Forecasts {
    /// What move `m` would bring household `h`, gain units: a store at a levy share, a store
    /// kept, a law against taking with its bundle, a watch, or a curfew (an amendment is
    /// forecast from what was seen, apart).
    pub(super) fn gain(
        &self,
        m: &MoveOption,
        h: PermanentId,
        policies: &[crate::polity::PolicyDef],
        params: &PeopleParams,
    ) -> f64 {
        let pp = &params.polity;
        let Ok(i) = self.outlooks.binary_search_by_key(&h, |o| o.0) else {
            return 0.0;
        };
        let o = &self.outlooks[i].1;
        let known = || {
            self.takings
                .binary_search_by_key(&h, |t| t.0)
                .map_or_else(|_| Default::default(), |k| self.takings[k].1)
        };
        match policies.get(usize::from(m.policy)).map(|d| d.kind) {
            Some(PolicyKind::KeepStore) => crate::polity::kept_gain(o, self.kept, pp),
            Some(PolicyKind::AgainstTaking) => {
                let (recover, owe) =
                    known().under(&m.sanction, o.year_need / 365.0, params.crime.exile_days);
                crate::polity::against_gain(o, recover, owe, pp)
            }
            Some(PolicyKind::KeepWatch) => {
                let (recover, owe) = known().watched(params.crime.watch_guard);
                crate::polity::against_gain(o, recover, owe, pp)
            }
            // A curfew (M4b slice AD): as a watch, at its own share, less what keeping its grown
            // members at home in its hours costs it.
            Some(PolicyKind::Curfew) => {
                let (recover, owe) = known().watched(params.crime.curfew_guard);
                let home = params.crime.curfew_cost_days
                    * f64::from(crate::polity::hours_long(m.hours))
                    * f64::from(self.grown.get(&h).copied().unwrap_or(0))
                    * o.year_need
                    / 365.0;
                crate::polity::against_gain(o, recover, owe + home, pp)
            }
            Some(PolicyKind::AmendBody) => 0.0,
            _ => crate::polity::store_gain(o, &self.belief, m.levy_share, pp),
        }
    }
}

/// Purpose tag for a deliberator's draw among moves.
pub const PURPOSE_DELIBERATE: u64 = 0x6465_6c69_6265_7231; // "deliber1"
/// Purpose tag for the draw of whether someone pays a levy.
pub const PURPOSE_LEVY: u64 = 0x6c65_7679_7061_7931; // "levypay1"

/// The shortest sitting worth walking to: a gathering about to rise is not.
const LEAST_SITTING_MIN: i64 = 15;

impl Population {
    /// The index of settlement `settlement`'s polity.
    pub(crate) fn polity_of(&self, settlement: PermanentId) -> Option<usize> {
        self.polities
            .iter()
            .position(|p| p.settlement == settlement)
    }

    /// The polities' midnight (ADR-0013): every settlement without a polity gets one under the
    /// founding custom; yesterday's gathering decides; those at home hear of the laws their
    /// households know; and a polity due its review weighs its issues.
    pub(super) fn polity_day(&mut self, ctx: &mut Ctx) {
        for i in 0..ctx.land.settlements.len() {
            let s = &ctx.land.settlements[i];
            if self.polity_of(s.id).is_none() {
                let (settlement, founded) = (s.id, s.founded);
                let id = ctx.ids.allocate();
                self.polities
                    .push(Polity::found(id, settlement, founded, &ctx.params.polity));
            }
        }
        let day = ctx.now.day_index();
        self.lapse_keepers(ctx);
        for pi in 0..self.polities.len() {
            if self.polities[pi]
                .gathering
                .as_ref()
                .is_some_and(|g| g.day < day)
            {
                self.decide_gathering(ctx, pi);
            }
        }
        // Cases still waiting go before the next gathering (M4b slice AB).
        if self
            .order
            .cases
            .iter()
            .any(|c| c.stage == crate::crime::CaseStage::Open)
        {
            for pi in 0..self.polities.len() {
                self.put_cases(pi, day, ctx.params.polity.notice_days);
            }
        }
        self.tell_households(day);
        // Anyone new takes what they hold of each value and norm, and of the ideologies a founder
        // brings or a parent holds (M4c slice AG).
        self.take_new_holdings(ctx);
        self.take_ideology_starts(ctx);
        // Households tell their members of gatherings ahead, and those short of food who find
        // the common store empty hold it against it (M4c slice AE).
        self.word_day(ctx);
        self.grieve_empty_stores(ctx);
        for pi in 0..self.polities.len() {
            let p = &self.polities[pi];
            if day - p.reviewed >= i64::from(ctx.params.polity.review_days.max(1)) {
                self.polities[pi].reviewed = day;
                self.review(ctx, pi);
            }
        }
    }

    /// A law that names someone lapses when they die or leave the settlement: the store they
    /// kept lies unkept again, or the watch they kept is unkept, which is a vacancy the next
    /// review may answer (succession, ADR-0013 §2). Nothing fills it by itself.
    fn lapse_keepers(&mut self, ctx: &mut Ctx) {
        for pi in 0..self.polities.len() {
            let settlement = self.polities[pi].settlement;
            let offices: Vec<(PermanentId, PermanentId, PolicyKind)> = self.polities[pi]
                .laws
                .iter()
                .filter(|l| l.status == LawStatus::InForce)
                .filter_map(|l| Some((l.holder?, l.id, l.kind)))
                .collect();
            for (holder, law, kind) in offices {
                self.lapse_office(ctx, pi, settlement, holder, law, kind);
            }
        }
    }

    /// Law `law` of polity `pi` names `holder` to an office of kind `kind`: it lapses if they no
    /// longer live in the settlement.
    fn lapse_office(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        settlement: PermanentId,
        holder: PermanentId,
        law: PermanentId,
        kind: PolicyKind,
    ) {
        {
            let here = self
                .person(holder)
                .and_then(|p| self.household(p.household))
                .is_some_and(|x| x.settlement == Some(settlement));
            if here {
                return;
            }
            if kind == PolicyKind::KeepStore {
                let sheltered = self.polity_sheltered(pi);
                self.polities[pi].settle_stores(ctx.now, &ctx.catalog.goods, sheltered);
            }
            if let Some(l) = self.polities[pi].law_mut(law) {
                l.status = LawStatus::Lapsed;
            }
            let why = match self.records.get(&holder) {
                Some(r) if r.died.is_some() => "they died",
                Some(r) if r.left.is_some() => "they left the valley",
                _ => "they no longer live there",
            };
            let (place, name) = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map_or((None, String::new()), |s| {
                    (Some(s.hearth_m), s.name.clone())
                });
            let office = if kind == PolicyKind::KeepWatch {
                "keeps watch"
            } else {
                "keeps the common store"
            };
            let words = format!(
                "{} no longer {office} at {name}: {why}.",
                self.name_of(holder)
            );
            self.chronicle_push(
                ctx.now,
                ChronicleKind::LawLapsed,
                vec![holder],
                Some(settlement),
                place,
                0.0,
                words,
            );
        }
    }

    /// The adults living in settlement `settlement`, with their households, in id order: the
    /// gathering's members (ADR-0013 §1: membership is residence).
    pub(super) fn members_of(
        &self,
        settlement: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<(PermanentId, PermanentId)> {
        let mut out: Vec<(PermanentId, PermanentId)> = self
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= params.family.independent_age)
            .filter(|(_, p)| {
                self.household(p.household)
                    .is_some_and(|x| x.settlement == Some(settlement))
            })
            .map(|(_, p)| (p.id, p.household))
            .collect();
        out.sort_unstable();
        out
    }

    /// Of `adults` (adults and their households, in id order), those a body of membership `m`
    /// admits (M4c slice AF): all of them; each household's elder; or the adults of households
    /// holding a field among `fields`.
    pub(super) fn admitted(
        &self,
        m: Membership,
        adults: &[(PermanentId, PermanentId)],
        fields: &[civ_land::fields::Field],
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<(PermanentId, PermanentId)> {
        match m {
            Membership::Adults => adults.to_vec(),
            Membership::Elders => adults
                .iter()
                .copied()
                .filter(|&(p, h)| self.elder_of(h, now, params) == Some(p))
                .collect(),
            Membership::Landholders => {
                let mut holding: Vec<PermanentId> =
                    fields.iter().filter_map(|f| f.holder.household()).collect();
                holding.sort_unstable();
                holding.dedup();
                adults
                    .iter()
                    .copied()
                    .filter(|(_, h)| holding.binary_search(h).is_ok())
                    .collect()
            }
        }
    }

    /// The members of polity `pi`'s body now, with their households, in id order (ADR-0013 §2;
    /// M4c slice AF): the adults it admits under its custom.
    pub fn body_members(
        &self,
        fields: &[civ_land::fields::Field],
        pi: usize,
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<(PermanentId, PermanentId)> {
        let polity = &self.polities[pi];
        let adults = self.members_of(polity.settlement, now, params);
        self.admitted(polity.body.members, &adults, fields, now, params)
    }

    /// Whether adult `person` of household `hh` belongs to polity `pi`'s body (M4c slice AF).
    fn admits(
        &self,
        fields: &[civ_land::fields::Field],
        pi: usize,
        person: PermanentId,
        hh: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> bool {
        match self.polities[pi].body.members {
            Membership::Adults => true,
            Membership::Elders => self.elder_of(hh, now, params) == Some(person),
            Membership::Landholders => fields.iter().any(|f| f.holder.household() == Some(hh)),
        }
    }

    /// What household `household` expects of its coming year: the food it holds, an ordinary
    /// harvest of its fields as their records say (less the seed), and a year's need.
    pub(super) fn outlook(&self, ctx: &Ctx, household: PermanentId) -> Option<Outlook> {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let x = self.household(household)?;
        let need_day = x.members.len() as f64 * params.household.daily_kcal_per_person;
        let held = stock_kcal(&stores_now(x, now, params, goods), goods);
        let harvest = ctx.catalog.crops.get(params.farm.crop).map_or(0.0, |crop| {
            let fields = || ctx.land.fields.iter().filter(|f| f.household == household);
            let area: f64 = fields().map(Field::area_ha).sum();
            if area <= 0.0 {
                return 0.0;
            }
            let per_ha = farm::expected_yield_kg_ha(fields(), crop, now.day_index());
            let kcal = goods.get(crop.good).map_or(0.0, |g| g.kcal_per_kg);
            (per_ha - crop.seed_kg_per_ha).max(0.0) * area * kcal
        });
        Some(Outlook {
            held,
            harvest,
            year_need: need_day * 365.0,
        })
    }

    /// What settlement `settlement`'s people believe of their years: lean years are those its
    /// food ran short in (the chronicle's), over the years since it was founded, with the prior;
    /// its households' ordinary harvest is the mean of `outlooks`.
    fn belief(&self, ctx: &Ctx, settlement: PermanentId, outlooks: &[Outlook]) -> Belief {
        let mut lean: Vec<i32> = self
            .chronicle
            .iter()
            .filter(|e| e.kind == ChronicleKind::FoodRanShort && e.settlement == Some(settlement))
            .map(|e| civ_land::calendar_year(e.at.day_index()))
            .collect();
        lean.dedup();
        let founded = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or(ctx.now, |s| s.founded);
        let years = (ctx.now.day_index() - founded.day_index()).max(0) as f64 / 365.0;
        let mean_harvest = if outlooks.is_empty() {
            0.0
        } else {
            outlooks.iter().map(|o| o.harvest).sum::<f64>() / outlooks.len() as f64
        };
        Belief {
            p_lean: crate::polity::p_lean(lean.len() as f64, years, &ctx.params.polity),
            mean_harvest,
        }
    }

    /// Whether household `household`'s food will not see it through to its next harvest: the
    /// shortfall against the outlook that reaches its people (ADR-0013 §5).
    fn pressed(&self, ctx: &Ctx, household: PermanentId) -> bool {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(x) = self.household(household) else {
            return false;
        };
        let need = x.members.len() as f64 * params.household.daily_kcal_per_person;
        let days = stock_kcal(&stores_now(x, now, params, goods), goods) / need.max(1.0);
        let to_harvest = ctx.catalog.crops.get(params.farm.crop).map_or(0.0, |crop| {
            farm::days_to_harvest(
                crop,
                ctx.land.fields.iter().filter(|f| f.household == household),
                now.day_index(),
            )
        });
        days < to_harvest || days < params.household.short_food_days
    }

    /// Whether settlement `settlement`'s food is short now or ran short within the last year: a
    /// lean year, as its people have seen it (the chronicle's, ADR-0013 §5).
    pub(super) fn lean_lately(&self, ctx: &Ctx, settlement: PermanentId) -> bool {
        let short_now = ctx
            .land
            .settlements
            .iter()
            .any(|s| s.id == settlement && s.food_short);
        let since = ctx.now.day_index() - 365;
        short_now
            || self.chronicle.iter().rev().any(|e| {
                e.kind == ChronicleKind::FoodRanShort
                    && e.settlement == Some(settlement)
                    && e.at.day_index() >= since
            })
    }

    /// The issues settlement `settlement` faces now (research 09-05 §1.1): its food ran short
    /// within the last year, or a household's food will not last until its next harvest
    /// (`pressed`); its common store holds food nobody keeps under a roof (`unkept`); and a
    /// household found food taken from its store within the last year (`takings`).
    fn issues(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        pressed: bool,
        unkept: bool,
        takings: bool,
    ) -> Vec<IssueKind> {
        let mut out = Vec::new();
        if self.lean_lately(ctx, settlement) || pressed {
            out.push(IssueKind::FoodShort);
        }
        if unkept {
            out.push(IssueKind::StoreUnkept);
        }
        if takings {
            out.push(IssueKind::Takings);
        }
        out
    }

    /// Food polity `pi`'s store would keep over a year under a roof rather than in the open, kcal.
    fn roof_saves(&self, ctx: &Ctx, pi: usize) -> f64 {
        let goods = &ctx.catalog.goods;
        let left = |half: f64| {
            if half > 0.0 {
                0.5f64.powf(365.0 / half)
            } else {
                1.0
            }
        };
        self.polities[pi]
            .stores
            .iter()
            .zip(goods)
            .filter(|(_, g)| g.purpose == GoodUse::Food && !g.kept_back())
            .map(|(kg, g)| {
                let roofed = if g.sheltered_half_life_days > 0.0 {
                    g.sheltered_half_life_days
                } else {
                    g.half_life_days
                };
                kg.max(0.0) * g.kcal_per_kg * (left(roofed) - left(g.half_life_days)).max(0.0)
            })
            .sum()
    }

    /// Whether polity `pi`'s store is kept under a roof now: its keeper's household is under one.
    pub(crate) fn polity_sheltered(&self, pi: usize) -> bool {
        self.polities[pi]
            .keeper()
            .and_then(|(k, _)| self.person(k))
            .and_then(|p| self.household(p.household))
            .is_some_and(|x| x.sheltered)
    }

    /// What the households `households` of polity `pi`'s settlement expect, to forecast what a
    /// move would bring each (ADR-0013 §5): their outlooks, the settlement's belief about its
    /// years, what a roof over the store would keep each, what each knows of takings in the last
    /// year (M4b slice AB), and how many grown members each has among `adults`.
    pub(super) fn forecasts(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
    ) -> Forecasts {
        let outlooks: Vec<(PermanentId, Outlook)> = households
            .iter()
            .filter_map(|&h| Some((h, self.outlook(ctx, h)?)))
            .collect();
        let just: Vec<Outlook> = outlooks.iter().map(|o| o.1).collect();
        let belief = self.belief(ctx, self.polities[pi].settlement, &just);
        let kept = self.roof_saves(ctx, pi) / outlooks.len().max(1) as f64;
        let takings = households
            .iter()
            .map(|&h| (h, self.takings_known(h, ctx.now, ctx.params)))
            .collect();
        let mut grown: BTreeMap<PermanentId, u32> = BTreeMap::new();
        for &(_, h) in adults {
            *grown.entry(h).or_default() += 1;
        }
        Forecasts {
            outlooks,
            belief,
            kept,
            takings,
            grown,
        }
    }

    /// A polity's routine review (ADR-0013 §5): with nothing before the gathering and an issue
    /// present, its notables and the elders of households whose food will not last to their
    /// next harvest each weigh proposing what answers it, at each level the template allows,
    /// against proposing nothing. The first to propose puts it to a gathering called for a few
    /// days on.
    fn review(&mut self, ctx: &mut Ctx, pi: usize) {
        let (now, params) = (ctx.now, ctx.params);
        let pp = &params.polity;
        let polity = &self.polities[pi];
        if polity.agenda().is_some() || polity.gathering.is_some() {
            return;
        }
        let settlement = polity.settlement;
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let pressed: Vec<PermanentId> = households
            .iter()
            .copied()
            .filter(|&h| self.pressed(ctx, h))
            .collect();
        let polity = &self.polities[pi];
        let unkept = polity
            .in_force(&ctx.catalog.policies, PolicyKind::CommonStore)
            .next()
            .is_some()
            && polity.keeper().is_none()
            && stock_kcal(&polity.stores, &ctx.catalog.goods) > 0.0;
        // Households of the settlement that found food taken from their stores in the last year,
        // and since the last review (what they know: M4b slice AB).
        let day = now.day_index();
        let review_since = day - i64::from(pp.review_days.max(1));
        let robbed = |since: i64| -> Vec<PermanentId> {
            households
                .iter()
                .copied()
                .filter(|&h| self.order.lost_since(h, since))
                .collect()
        };
        let robbed_lately = robbed(review_since);
        let issues = self.issues(
            ctx,
            settlement,
            !pressed.is_empty(),
            unkept,
            !robbed(day - 365).is_empty(),
        );
        let polity = &self.polities[pi];
        // Templates that answer a present issue and are not already in force here.
        let open: Vec<(u16, IssueKind)> = ctx
            .catalog
            .policies
            .iter()
            .enumerate()
            .filter(|(k, _)| {
                !polity
                    .laws
                    .iter()
                    .any(|l| usize::from(l.policy) == *k && l.status == LawStatus::InForce)
            })
            .filter_map(|(k, d)| {
                let issue = d.answers.iter().copied().find(|a| issues.contains(a))?;
                Some((k as u16, issue))
            })
            .collect();
        // An amendment of the custom answers no settlement's issue: it is open to those a
        // gathering overruled (M4c slice AF), so a content that has one always reviews.
        let amendable = ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::AmendBody && !d.bodies.is_empty());
        // Holders of an ideology weigh its program when the problem it explains is before the
        // village (M4c slice AG).
        let programs = ctx
            .catalog
            .ideologies
            .iter()
            .any(|d| !d.program.is_empty() && issues.contains(&d.explains))
            && !self.ideologies.held.is_empty();
        if open.is_empty() && !amendable && !programs {
            return;
        }
        // The settlement's adults, and those of them its body admits (M4c slice AF): only members
        // propose and stand at its gatherings; anyone may hold an office it creates.
        let adults = self.members_of(settlement, now, params);
        let members = self.admitted(
            self.polities[pi].body.members,
            &adults,
            &ctx.land.fields,
            now,
            params,
        );
        // What each household expects, to forecast what a move would bring it.
        let fc = self.forecasts(ctx, pi, &households, &adults);
        let outlooks = &fc.outlooks;
        // Who could keep the store: adults whose household is under a roof.
        let roofed: Vec<PermanentId> = adults
            .iter()
            .filter(|(_, h)| self.household(*h).is_some_and(|x| x.sheltered))
            .map(|(p, _)| *p)
            .collect();
        // Who weighs moves (ADR-0014 §4): the notables, a compute tier; and those something
        // reached since the last review: the elders of households whose food will not last, and
        // whoever came to a gathering or learnt a law. With the tier off, every adult.
        let mut deliberators: Vec<PermanentId> = if self.every_adult_deliberates {
            members.iter().map(|m| m.0).collect()
        } else {
            let since = now.day_index() - i64::from(pp.review_days.max(1));
            let reached = self.polities[pi].laws.iter().flat_map(|l| {
                let came = l
                    .decided
                    .filter(|t| t.day_index() > since)
                    .map(|_| l.stances.iter().map(|r| r.person))
                    .into_iter()
                    .flatten();
                let learnt = l.known.iter().filter(move |k| k.1 > since).map(|k| k.0);
                came.chain(learnt)
            });
            self.standing
                .in_settlement(settlement)
                .filter(|r| r.notable)
                .map(|r| r.person)
                .chain(
                    pressed
                        .iter()
                        .chain(&robbed_lately)
                        .filter_map(|&h| self.elder_of(h, now, params)),
                )
                .chain(reached)
                // And whoever holds an ideology whose program answers a problem before the
                // village (M4c slice AG).
                .chain(
                    self.ideologies
                        .held
                        .iter()
                        .filter(|h| {
                            ctx.catalog
                                .ideologies
                                .get(usize::from(h.ideology))
                                .is_some_and(|d| {
                                    !d.program.is_empty() && issues.contains(&d.explains)
                                })
                        })
                        .map(|h| h.holder),
                )
                .collect()
        };
        deliberators.sort_unstable();
        deliberators.dedup();
        deliberators.retain(|d| members.binary_search_by_key(d, |m| m.0).is_ok());
        let tp = &params.ties;
        // What its gatherings decided within memory (M4c slice AF): each law and case, with the
        // stances taken there (open acclamation: those who came saw them), and who each kind of
        // body would admit now.
        // Only decisions the present custom made count: a change of custom is judged by how the
        // custom it would replace has decided, so a newly amended custom is not undone on the
        // strength of what an older one did (that let two camps toggle the custom week by week).
        let memory = i64::from(pp.vote_memory_days);
        let polity = &self.polities[pi];
        let custom_since = polity.versions.last().map_or(polity.founded, |v| v.since);
        let recent = |t: Option<SimTime>| {
            t.is_some_and(|t| day - t.day_index() <= memory && t >= custom_since)
        };
        // Amendments themselves are not among them: a rule is judged by the laws and cases it
        // decides, not by other changes of rule (which would let the custom feed on itself).
        let mut decisions: Vec<Seen> = polity
            .laws
            .iter()
            .filter(|l| recent(l.decided) && !l.stances.is_empty())
            .filter(|l| l.kind != PolicyKind::AmendBody)
            .map(|l| Seen {
                stances: l
                    .stances
                    .iter()
                    .map(|r| (r.person, r.household, r.stance, f64::from(r.gain)))
                    .collect(),
                passed: l.outcome == Some(Outcome::Passed),
            })
            .collect();
        decisions.extend(
            self.order
                .cases
                .iter()
                .filter(|c| recent(c.heard) && !c.stances.is_empty())
                .filter(|c| polity.laws.iter().any(|l| l.id == c.law))
                .map(|c| Seen {
                    stances: c
                        .stances
                        .iter()
                        .map(|r| (r.person, r.household, r.stance, f64::from(r.stake)))
                        .collect(),
                    passed: c.stage == crate::crime::CaseStage::Found,
                }),
        );
        let (current, polity_id) = (polity.body, polity.id);
        let admitted_by: Vec<Vec<PermanentId>> = Membership::ALL
            .iter()
            .map(|&m| {
                self.admitted(m, &adults, &ctx.land.fields, now, params)
                    .into_iter()
                    .map(|a| a.0)
                    .collect()
            })
            .collect();
        // Whether body `b` would have passed decision `s`: those of the stances it admits now,
        // by its rule (research 09-05 §1.4).
        let would_pass = |s: &Seen, b: &Body| {
            let admitted = &admitted_by[usize::from(b.members.code())];
            let (mut present, mut support, mut oppose) = (0, 0, 0);
            for &(p, _, stance, _) in &s.stances {
                if admitted.binary_search(&p).is_err() {
                    continue;
                }
                present += 1;
                match stance {
                    Stance::Support => support += 1,
                    Stance::Oppose => oppose += 1,
                    Stance::Abstain => {}
                }
            }
            b.decide(admitted.len() as u32, present, support, oppose) == Outcome::Passed
        };
        // What body `b` would bring household `h`, from decisions `seen` (research 09-02 §3.7,
        // 09-05 §1.2: a rule is judged by how it would have decided what was seen): each one it
        // would have turned the other way, at the household's stake in it.
        let amend_gain = |b: &Body, h: PermanentId, seen: &[usize]| -> f64 {
            seen.iter()
                .map(|&i| {
                    let s = &decisions[i];
                    let now_passes = would_pass(s, b);
                    if now_passes == s.passed {
                        return 0.0;
                    }
                    let stake = s.stances.iter().find(|r| r.1 == h).map_or(0.0, |r| r.3);
                    let sign = if now_passes { 1.0 } else { -1.0 };
                    sign * stake / pp.w_gain.max(1e-9)
                })
                .sum()
        };
        // The decisions those `seen` saw.
        let seen_by = |seen: &dyn Fn(PermanentId, PermanentId) -> bool| -> Vec<usize> {
            decisions
                .iter()
                .enumerate()
                .filter(|(_, s)| s.stances.iter().any(|r| seen(r.0, r.1)))
                .map(|(i, _)| i)
                .collect()
        };
        let deliberator = RuleDeliberator { params: pp };
        let policies = &ctx.catalog.policies;
        let gain_of = |m: &MoveOption, h: PermanentId| fc.gain(m, h, policies, params);
        for d in deliberators {
            let Some(own) = members
                .binary_search_by_key(&d, |m| m.0)
                .ok()
                .map(|i| members[i].1)
            else {
                continue;
            };
            // The moves open to them: each level of a store, and a keeper of their choosing:
            // themselves if their roof would do, or whoever under a roof they regard most.
            let mut moves = Vec::new();
            // With the laws the ideologies they hold propose for a problem before the village, at
            // the issue each explains (M4c slice AG).
            let mut wants = open.clone();
            for (k, issue) in self.programs_of(ctx, d, &issues) {
                let in_force = self.polities[pi]
                    .laws
                    .iter()
                    .any(|l| l.policy == k && l.status == LawStatus::InForce);
                if !in_force && !wants.iter().any(|w| w.0 == k) {
                    wants.push((k, issue));
                }
            }
            for &(k, issue) in &wants {
                let def = &ctx.catalog.policies[usize::from(k)];
                let blank = MoveOption {
                    policy: k,
                    levy_share: 0.0,
                    issue,
                    nominee: None,
                    sanction: Default::default(),
                    hours: (0, 0),
                    body: None,
                    own_gain: 0.0,
                    followers_gain: 0.0,
                    support: 0.5,
                };
                match def.kind {
                    PolicyKind::CommonStore => {
                        for &share in &def.levy_shares {
                            moves.push(MoveOption {
                                levy_share: share,
                                ..blank
                            });
                        }
                    }
                    PolicyKind::AgainstTaking => {
                        for &sanction in &def.bundles {
                            moves.push(MoveOption { sanction, ..blank });
                        }
                    }
                    PolicyKind::Curfew => {
                        for &hours in &def.hours {
                            moves.push(MoveOption { hours, ..blank });
                        }
                    }
                    PolicyKind::KeepStore => {
                        let pick = roofed
                            .iter()
                            .map(|&c| {
                                let r = if c == d {
                                    1.0
                                } else {
                                    self.ties.regard(d, c, day, tp).clamp(0.0, 1.0)
                                };
                                (r, c)
                            })
                            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
                        if let Some((_, c)) = pick {
                            moves.push(MoveOption {
                                nominee: Some(c),
                                ..blank
                            });
                        }
                    }
                    // An amendment answers no settlement's issue: it is weighed below.
                    PolicyKind::AmendBody => {}
                    // A watch: the adult they regard most, themselves at full regard (M4b slice
                    // AC).
                    PolicyKind::KeepWatch => {
                        let pick = adults
                            .iter()
                            .map(|&(c, _)| {
                                let r = if c == d {
                                    1.0
                                } else {
                                    self.ties.regard(d, c, day, tp).clamp(0.0, 1.0)
                                };
                                (r, c)
                            })
                            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
                        if let Some((_, c)) = pick {
                            moves.push(MoveOption {
                                nominee: Some(c),
                                ..blank
                            });
                        }
                    }
                }
            }
            // An amendment of the custom (M4c slice AF): open to one whom a gathering they came
            // to within memory overruled, or who holds a grievance against the gathering; each
            // body a template offers that differs from the custom and would keep them in it.
            let seen_d = seen_by(&|p, _| p == d);
            let overruled = seen_d.iter().any(|&i| {
                let s = &decisions[i];
                s.stances.iter().any(|r| {
                    r.0 == d
                        && match r.2 {
                            Stance::Support => !s.passed,
                            Stance::Oppose => s.passed,
                            Stance::Abstain => false,
                        }
                })
            }) || self
                .word
                .grievances_of(d)
                .any(|g| g.blamed == Blamed::Body(polity_id) && g.unresolved_days > 0.0);
            if overruled {
                for (k, def) in ctx.catalog.policies.iter().enumerate() {
                    if def.kind != PolicyKind::AmendBody {
                        continue;
                    }
                    for &b in &def.bodies {
                        let keeps = admitted_by[usize::from(b.members.code())]
                            .binary_search(&d)
                            .is_ok();
                        // One change at a time (research 09-05 §2.3: a sponsor weighs a few
                        // alternatives; §3.9: reform changes a gate within the procedure).
                        let changes = u32::from(b.members != current.members)
                            + u32::from((b.quorum_share - current.quorum_share).abs() > 1e-6)
                            + u32::from(b.pass != current.pass);
                        if changes == 1 && keeps {
                            moves.push(MoveOption {
                                policy: k as u16,
                                levy_share: 0.0,
                                issue: IssueKind::Overruled,
                                nominee: None,
                                sanction: Default::default(),
                                hours: (0, 0),
                                body: Some(b),
                                own_gain: 0.0,
                                followers_gain: 0.0,
                                support: 0.5,
                            });
                        }
                    }
                }
            }
            let gain_for = |m: &MoveOption, h: PermanentId| match m.body {
                Some(b) => amend_gain(&b, h, &seen_d),
                None => gain_of(m, h),
            };
            for m in &mut moves {
                // Those who regard them, by how much; and those they know, by where they would
                // stand with them as its sponsor (or with the one it names).
                let face = m.nominee.unwrap_or(d);
                let (mut weighed, mut weight, mut support, mut oppose) = (0.0, 0.0, 0, 0);
                for &(a, h) in &members {
                    if a == d {
                        continue;
                    }
                    let gain = gain_for(m, h);
                    let regard = self.ties.regard(a, d, day, tp).clamp(0.0, 1.0);
                    if regard > 0.0 {
                        weighed += regard * gain;
                        weight += regard;
                    }
                    if self.ties.known(d, a, day, tp) > 0.0 {
                        let to_face = if face == a {
                            1.0
                        } else {
                            self.ties.regard(a, face, day, tp)
                        };
                        match crate::polity::stance(pp.w_gain * gain, to_face, pp).0 {
                            Stance::Support => support += 1,
                            Stance::Oppose => oppose += 1,
                            Stance::Abstain => {}
                        }
                    }
                }
                // What it does to what they hold dear weighs beside their household's lot (M4c
                // slice AG); nobody sees another's, so those they count on are judged by their
                // households' lots alone.
                // And the ideology they hold that proposes it, if one does (M4c slice AG).
                let creed = self.creed_for(ctx, d, m.policy).map_or(0.0, |c| c.1);
                m.own_gain = gain_for(m, own)
                    + (self.value_points(ctx, d, m.policy) + creed) / pp.w_gain.max(1e-9);
                m.followers_gain = if weight > 0.0 { weighed / weight } else { 0.0 };
                m.support = if support + oppose > 0 {
                    f64::from(support) / f64::from(support + oppose)
                } else {
                    0.5
                };
                // What they saw (research 09-05 §1.2: a sponsor weighs expected success): one who
                // came to a gathering that decided the same proposal, within memory, expects no
                // more support for it than it got there.
                let seen = self.polities[pi]
                    .laws
                    .iter()
                    .rev()
                    .filter(|l| {
                        l.policy == m.policy
                            && l.holder == m.nominee
                            && l.sanction == m.sanction
                            && l.hours == m.hours
                            && l.body == m.body
                    })
                    .filter(|l| (f64::from(l.levy_share) - m.levy_share).abs() < 1e-4)
                    .filter(|l| {
                        l.decided
                            .is_some_and(|t| day - t.day_index() <= i64::from(pp.vote_memory_days))
                    })
                    .find(|l| l.stances.iter().any(|r| r.person == d));
                if let Some(l) = seen {
                    let (_, s, o) = l.counts();
                    if s + o > 0 {
                        m.support = m.support.min(f64::from(s) / f64::from(s + o));
                    }
                }
            }
            // An amendment is weighed only by one it would have served (research 09-02 §3.7:
            // institutional change is proposed for an expected benefit): with nothing to gain,
            // changing the custom is no move of theirs.
            moves.retain(|m| m.body.is_none() || m.own_gain > 0.0);
            let u =
                Rng64::from_key(&[ctx.seed, PURPOSE_DELIBERATE, d.get(), day as u64]).next_f64();
            let choice = deliberator.choose(&moves, u);
            if let Some(m) = choice.chosen.map(|i| moves[i]) {
                let stakes: Vec<(PermanentId, f32)> = outlooks
                    .iter()
                    .map(|(h, _)| {
                        // Each household judges an amendment by the decisions its own members
                        // saw (M4c slice AF).
                        let gain = match m.body {
                            Some(b) => amend_gain(&b, *h, &seen_by(&|_, hh| hh == *h)),
                            None => gain_of(&m, *h),
                        };
                        (*h, (pp.w_gain * gain) as f32)
                    })
                    .collect();
                let creed = self.creed_for(ctx, d, m.policy).map(|c| c.0);
                self.propose(ctx, pi, d, m, stakes, creed);
                return;
            }
        }
    }

    /// `sponsor` puts move `m` to the gathering (ADR-0013 §3, stage 1): the law is proposed, the
    /// sponsor knows it, and a gathering is called for `notice_days` on.
    fn propose(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        sponsor: PermanentId,
        m: MoveOption,
        stakes: Vec<(PermanentId, f32)>,
        creed: Option<u16>,
    ) {
        let (now, pp) = (ctx.now, &ctx.params.polity);
        let day = now.day_index();
        let def = &ctx.catalog.policies[usize::from(m.policy)];
        let id = ctx.ids.allocate();
        let meets = day + i64::from(pp.notice_days.max(1));
        let law = Law {
            id,
            policy: m.policy,
            kind: def.kind,
            levy_share: m.levy_share as f32,
            holder: m.nominee,
            relief_days: def.relief_days as f32,
            sanction: m.sanction,
            hours: m.hours,
            status: LawStatus::Proposed,
            sponsor,
            proposed: now,
            issue: m.issue,
            meets_day: meets,
            decided: None,
            outcome: None,
            eligible: 0,
            stances: Vec::new(),
            known: vec![(sponsor, day)],
            compliance: Default::default(),
            watch: Default::default(),
            body: m.body,
        };
        // "Mira proposed themselves as keeper of the common store", not "Mira as keeper"; and
        // the creed they proposed it under, if any (M4c slice AG).
        let held = creed
            .and_then(|k| ctx.catalog.ideologies.get(usize::from(k)))
            .map(|d| format!(", as one who holds to {}", d.name))
            .unwrap_or_default();
        let words = format!(
            "{}, because {}{held}",
            crate::polity::law_words(&law, &ctx.catalog.policies, &|id| {
                if id == sponsor {
                    "themselves".to_owned()
                } else {
                    self.name_of(id)
                }
            }),
            m.issue.words()
        );
        if let Some(k) = creed {
            self.ideologies.note_creed(id, k);
        }
        let polity = &mut self.polities[pi];
        polity.laws.push(law);
        polity.gathering = Some(Gathering {
            law: Some(id),
            day: meets,
            stakes,
            present: Vec::new(),
            cases: Vec::new(),
        });
        let settlement = polity.settlement;
        // The sponsor knows it is called; everyone else hears by word (M4c slice AE).
        self.call_word(settlement, meets, Some(id), &[sponsor], day);
        let place = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map(|s| s.hearth_m);
        self.chronicle_push(
            now,
            ChronicleKind::LawProposed,
            vec![sponsor],
            Some(settlement),
            place,
            m.levy_share,
            words,
        );
    }

    /// The gathering called at polity `pi` has sat: it decides the law before it, if any, and
    /// then hears its cases in the order they were brought (M4b slice AB).
    fn decide_gathering(&mut self, ctx: &mut Ctx, pi: usize) {
        let Some(g) = self.polities[pi].gathering.take() else {
            return;
        };
        if let Some(law) = g.law {
            self.decide_law(ctx, pi, &g, law);
        }
        for &case in &g.cases {
            self.hear_case(ctx, pi, &g, case);
        }
    }

    /// The gathering called at polity `pi` has sat on law `law_id` (ADR-0013 §3, stages 2-4):
    /// each member who came takes a stance from what their household made of the law and their
    /// regard for its sponsor, the body decides by its rule, and if it passed, those who came know
    /// it. A thin gathering or a tie is recorded as what it was.
    fn decide_law(&mut self, ctx: &mut Ctx, pi: usize, g: &Gathering, law_id: PermanentId) {
        let (now, params) = (ctx.now, ctx.params);
        let pp = &params.polity;
        let settlement = self.polities[pi].settlement;
        // Its body's members now (M4c slice AF: the custom says who they are).
        let members = self.body_members(&ctx.land.fields, pi, now, params);
        let Some(law) = self.polities[pi].laws.iter().find(|l| l.id == law_id) else {
            return;
        };
        let (sponsor, policy) = (law.sponsor, law.policy);
        let w_position = params.opinion.w_position;
        // Those who came weigh their regard for the one the law names, or else for its sponsor.
        let face = law.holder.unwrap_or(sponsor);
        let day = now.day_index();
        // The store spoils under the roof it had until a new keeper takes it.
        let sheltered = self.polity_sheltered(pi);
        self.polities[pi].settle_stores(now, &ctx.catalog.goods, sheltered);
        let stances: Vec<StanceRecord> = g
            .present
            .iter()
            .filter_map(|&p| {
                let i = members.binary_search_by_key(&p, |m| m.0).ok()?;
                let household = members[i].1;
                let gain = g.stake(household);
                let regard = if p == face {
                    1.0
                } else {
                    self.ties.regard(p, face, day, &params.ties)
                };
                // How far talk at the hearth has moved them from their household's lot (M4c
                // slice AG).
                let talk = self.opinion_points(p, policy, w_position);
                // And what it does to what they hold dear (M4c slice AG).
                let values = self.value_points(ctx, p, policy);
                let (stance, regard_points) = if p == sponsor {
                    (Stance::Support, 0.0)
                } else {
                    crate::polity::stance(gain + values + talk, regard, pp)
                };
                Some(StanceRecord {
                    person: p,
                    household,
                    stance,
                    gain: gain as f32,
                    regard: regard_points as f32,
                    opinion: talk as f32,
                    values: values as f32,
                })
            })
            .collect();
        let polity = &mut self.polities[pi];
        let body = polity.body;
        let Some(law) = polity.law_mut(law_id) else {
            return;
        };
        law.stances = stances;
        let (present, support, oppose) = law.counts();
        let eligible = members.len() as u32;
        let outcome = body.decide(eligible, present, support, oppose);
        law.eligible = eligible;
        law.decided = Some(now);
        law.outcome = Some(outcome);
        law.status = if outcome == Outcome::Passed {
            for p in &g.present {
                law.learn(*p, day);
            }
            LawStatus::InForce
        } else {
            LawStatus::Rejected
        };
        let law = &self.polities[pi].laws[self.polities[pi]
            .laws
            .iter()
            .position(|l| l.id == law_id)
            .unwrap_or(0)];
        let what = crate::polity::law_words(law, &ctx.catalog.policies, &|id| self.name_of(id));
        let (place, name) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or((None, String::new()), |s| {
                (Some(s.hearth_m), s.name.clone())
            });
        let who = match body.members {
            Membership::Adults => "adults",
            Membership::Elders => "elders",
            Membership::Landholders => "landholders",
        };
        let tally = format!("{support} for, {oppose} against; {present} of {eligible} {who} came");
        let words = match outcome {
            Outcome::Passed => format!("The gathering at {name} agreed to {what}: {tally}."),
            Outcome::Failed => format!("The gathering at {name} turned down {what}: {tally}."),
            Outcome::Tied => format!(
                "The gathering at {name} was evenly split on {what}, so it failed: {tally}."
            ),
            Outcome::NoQuorum => format!(
                "Too few came to the gathering at {name} to decide on {what}: {present} of \
                 {eligible} {who}, where {} were needed.",
                body.quorum(eligible)
            ),
        };
        self.chronicle_push(
            now,
            ChronicleKind::LawDecided,
            vec![sponsor],
            Some(settlement),
            place,
            f64::from(outcome as u8),
            words,
        );
        // An amendment passed changes the custom by its own procedure (M4c slice AF, ADR-0017
        // §1): a new version, the one it replaces superseded, told as an amendment.
        let amended = self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == law_id)
            .filter(|l| l.outcome == Some(Outcome::Passed) && l.kind == PolicyKind::AmendBody)
            .and_then(|l| l.body);
        if let Some(new) = amended {
            let polity = &mut self.polities[pi];
            for l in &mut polity.laws {
                if l.kind == PolicyKind::AmendBody
                    && l.status == LawStatus::InForce
                    && l.id != law_id
                {
                    l.status = LawStatus::Superseded;
                }
            }
            polity.body = new;
            polity.versions.push(CustomVersion {
                body: new,
                since: now,
                law: Some(law_id),
            });
            let words = format!(
                "The custom at {name} changed by its own procedure, on {}'s proposal: from now \
                 on, {}.",
                self.name_of(sponsor),
                new.clause()
            );
            self.chronicle_push(
                now,
                ChronicleKind::CustomAmended,
                vec![sponsor],
                Some(settlement),
                place,
                f64::from(self.polities[pi].versions.len() as u32),
                words,
            );
        }
    }

    /// Those at home hear of the laws in force their households know (ADR-0013 §3, stage 4):
    /// nobody keeps a law from their own household.
    fn tell_households(&mut self, day: i64) {
        for pi in 0..self.polities.len() {
            if !self.polities[pi]
                .laws
                .iter()
                .any(|l| l.status == LawStatus::InForce)
            {
                continue;
            }
            let settlement = self.polities[pi].settlement;
            let mut homes: Vec<(PermanentId, Vec<PermanentId>)> = self
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(settlement) && x.members.len() > 1)
                .map(|(_, x)| (x.id, x.members.clone()))
                .collect();
            homes.sort_unstable_by_key(|h| h.0);
            for law in &mut self.polities[pi].laws {
                if law.status != LawStatus::InForce {
                    continue;
                }
                for (_, members) in &homes {
                    if members.iter().any(|&m| law.knows(m)) {
                        for &m in members {
                            law.learn(m, day);
                        }
                    }
                }
            }
        }
    }

    /// `a` and `b` kept company at the hearth: each tells the other the laws in force there they
    /// know (ADR-0013 §3, stage 4).
    pub(crate) fn share_laws(&mut self, a: PermanentId, b: PermanentId, day: i64) {
        let Some(settlement) = self
            .person(a)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
        else {
            return;
        };
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        for law in &mut self.polities[pi].laws {
            if law.status != LawStatus::InForce {
                continue;
            }
            match (law.knows(a), law.knows(b)) {
                (true, false) => {
                    law.learn(b, day);
                }
                (false, true) => {
                    law.learn(a, day);
                }
                _ => {}
            }
        }
    }

    /// What attending the gathering sitting now in the settlement of household `hh` is worth to
    /// `person`, aged `age`, at `minute` of the day, with the evening from `evening_start`: what
    /// their household made of the law and their regard for its sponsor, over the custom's own
    /// pull. `None` when no gathering of theirs sits, they are not a member, or they are there.
    pub(crate) fn gathering_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        age: f64,
        hh: &Household,
        minute: i64,
        evening_start: i64,
    ) -> Option<GatheringFacts> {
        let params = ctx.params;
        let pp = &params.polity;
        if age < params.family.independent_age {
            return None;
        }
        let pi = self.polity_of(hh.settlement?)?;
        let polity = &self.polities[pi];
        let g = polity.gathering.as_ref()?;
        let end = evening_start + i64::from(pp.gathering_minutes);
        if g.day != ctx.now.day_index()
            || minute < evening_start
            || minute + LEAST_SITTING_MIN > end
            || g.present.binary_search(&person).is_ok()
            // Only those who heard it is called come (M4c slice AE, ADR-0016 §3).
            || !self.heard_of_gathering(person, polity.settlement, g.day)
            // And only the body's members (M4c slice AF).
            || !self.admits(&ctx.land.fields, pi, person, hh.id, ctx.now, params)
        {
            return None;
        }
        let day = ctx.now.day_index();
        let regard_for = |q: PermanentId| {
            if q == person {
                1.0
            } else {
                self.ties
                    .regard(person, q, day, &params.ties)
                    .clamp(0.0, 1.0)
            }
        };
        // What the law means to their household, and their regard for its sponsor; and for each
        // case, their regard for whichever party they regard more (their own household's case
        // counts as full regard).
        let law_stake = g
            .law
            .and_then(|id| polity.laws.iter().find(|l| l.id == id))
            .map_or(0.0, |l| {
                g.stake(hh.id).abs() + pp.w_regard * regard_for(l.sponsor)
            });
        let case_stake = g
            .cases
            .iter()
            .filter_map(|&c| self.order.case(c))
            .map(|c| {
                let party = hh.id == c.accuser || hh.id == c.accused_household;
                let r = if party {
                    1.0
                } else {
                    regard_for(c.by).max(regard_for(c.accused))
                };
                pp.w_regard * r
            })
            .fold(0.0, f64::max);
        let stake = law_stake.max(case_stake);
        Some(GatheringFacts {
            points: pp.attend_base + pp.w_attend * stake,
            minutes: (end - minute) as f64,
        })
    }

    /// A curfew in force where household `hh` lives and running at minute `minute` of the day
    /// (M4b slice AD): its polity and law, by index, and, if `person` knows of it, the points
    /// keeping it weighs with them (research 09-06 §1.5: the norms they hold that the gathering
    /// binds, where they stood on it and their regard for its sponsor, as a levy's; never below
    /// 0).
    pub(crate) fn curfew_now(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
        minute: i64,
    ) -> Option<(usize, usize, Option<f64>)> {
        let pi = self.polity_of(hh.settlement?)?;
        let laws = &self.polities[pi].laws;
        let li = laws.iter().position(|l| {
            l.status == LawStatus::InForce
                && l.kind == PolicyKind::Curfew
                && crate::polity::within_hours(l.hours, minute)
        })?;
        let law = &laws[li];
        let points = law.knows(person).then(|| {
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == person)
                .map_or(0.0, |r| r.stance.sign());
            let regard =
                self.ties
                    .regard(person, law.sponsor, ctx.now.day_index(), &ctx.params.ties);
            let norm = self.norm_points(ctx, person);
            crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity).max(0.0)
        });
        Some((pi, li, points))
    }

    /// `person` sits down at the gathering of their settlement.
    pub(crate) fn attend(&mut self, ctx: &Ctx, person: PermanentId) {
        let Some(settlement) = self
            .person(person)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
        else {
            return;
        };
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        let day = ctx.now.day_index();
        if let Some(g) = self.polities[pi]
            .gathering
            .as_mut()
            .filter(|g| g.day == day)
            && let Err(at) = g.present.binary_search(&person)
        {
            g.present.insert(at, person);
        }
    }

    /// `who` of household `household` threshed `grain_kg` of grain (good `grain`): under each
    /// common store in force where they live, they pay its share into the store, or cannot, or
    /// keep it back, or do not know they owe it (ADR-0013 §3, stages 6-7). Paying is their
    /// choice: the custom that the gathering binds, where they stood on it and their regard for
    /// its sponsor, against what it costs their household.
    pub(crate) fn levy(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        household: PermanentId,
        grain_kg: f64,
        grain: usize,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let pp = &params.polity;
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        let laws: Vec<usize> = (0..self.polities[pi].laws.len())
            .filter(|&li| {
                let l = &self.polities[pi].laws[li];
                l.status == LawStatus::InForce
                    && ctx
                        .catalog
                        .policies
                        .get(usize::from(l.policy))
                        .is_some_and(|d| d.kind == PolicyKind::CommonStore)
            })
            .collect();
        let kcal_per_kg = goods.get(grain).map_or(0.0, |g| g.kcal_per_kg);
        let day = now.day_index();
        for li in laws {
            let law = &self.polities[pi].laws[li];
            let owed = grain_kg * f64::from(law.levy_share);
            if owed <= 1e-9 {
                continue;
            }
            let (store, sponsor) = (self.polities[pi].id, law.sponsor);
            if !law.knows(who) {
                self.polities[pi].laws[li].compliance.unaware += 1;
                continue;
            }
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == who)
                .map_or(0.0, |r| r.stance.sign());
            let Some(outlook) = self.outlook(ctx, household) else {
                continue;
            };
            let kcal = owed * kcal_per_kg;
            if crate::polity::cannot_pay(outlook.held, kcal, outlook.year_need, pp) {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.could_not += 1;
                c.withheld_kg += owed;
                continue;
            }
            let regard = self.ties.regard(who, sponsor, day, &params.ties);
            let cost = pp.w_gain * crate::polity::levy_cost(&outlook, kcal, pp);
            // What the norms they hold add (M4c slice AG): someone new holds theirs now.
            self.ensure_norm_state(ctx, who);
            let norm = self.norm_points(ctx, who);
            let points = crate::polity::comply_points(norm, stance, regard, cost, pp);
            let chance = crate::polity::comply_chance(points);
            let key = [ctx.seed, PURPOSE_LEVY, who.get(), now.minutes() as u64];
            if Rng64::from_key(&key).next_f64() >= chance {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.evaded += 1;
                c.withheld_kg += owed;
                // What the household did is theirs to tell (M4c slice AG).
                self.norms.note_levy(household, day, false);
                continue;
            }
            let held = self
                .household(household)
                .and_then(|x| x.stores.get(grain))
                .copied()
                .unwrap_or(0.0)
                .max(0.0);
            let paid = owed.min(held);
            let leg = Leg {
                from: household,
                to: store,
                good: grain,
                amount: paid,
            };
            if paid > 1e-9 && self.transfer(now, params, goods, &[leg], Channel::Levy) {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.complied += 1;
                c.levied_kg += paid;
                self.norms.note_levy(household, day, true);
                // A levy taken in a lean year that leaves the household short of a year's food is
                // held against the gathering (M4c slice AE); in an ordinary year it is the
                // custom, however it weighs (research 04-10 §1.1).
                let paid_kcal = paid * kcal_per_kg;
                let short = (outlook.year_need - (outlook.held - paid_kcal)).min(paid_kcal);
                let settlement = self.polities[pi].settlement;
                if short > 0.0 && self.lean_lately(ctx, settlement) {
                    let law = self.polities[pi].laws[li].id;
                    self.grieve_levy(ctx, pi, law, household, short);
                }
            }
        }
    }

    /// The common store of `hh`'s settlement as somewhere to ask for food (ADR-0013 §4), when a
    /// common store is in force there that one of its members knows, and it holds food: as much
    /// as the law allows an ask, up to `want` kcal, at its keeper's home or else at the hearth
    /// `hearth`, walked to as `reach` says.
    pub(crate) fn relief_option(
        &self,
        ctx: &Ctx,
        hh: &Household,
        kcal_day: f64,
        want: f64,
        reach: &TravelField,
        hearth: (f32, f32),
    ) -> Option<GiverOption> {
        let polity = &self.polities[self.polity_of(hh.settlement?)?];
        let at = polity
            .keeper()
            .and_then(|(k, _)| self.person(k))
            .and_then(|p| self.household(p.household))
            .map_or(hearth, |x| x.home);
        let walk_min = f64::from(reach.seconds_to(cell_of(ctx.map, at))?) / 60.0;
        let law = polity
            .in_force(&ctx.catalog.policies, PolicyKind::CommonStore)
            .find(|l| hh.members.iter().any(|&m| l.knows(m)))?;
        let held = stock_kcal(&polity.stores, &ctx.catalog.goods);
        let kcal = held.min(want).min(f64::from(law.relief_days) * kcal_day);
        (kcal > 0.0).then_some(GiverOption {
            household: polity.id,
            walk_min,
            at,
            kcal,
        })
    }

    /// `asker` of household `to` asked polity `pi`'s store for food (ADR-0013 §4): it gives what
    /// the law allows an ask, up to what brings the household to the days of food it tries to
    /// keep and what one person carries, the most perishable first. An ask the store cannot
    /// answer is recorded.
    pub(crate) fn give_relief(&mut self, ctx: &Ctx, pi: usize, to: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(li) = self.polities[pi].laws.iter().position(|l| {
            l.status == LawStatus::InForce
                && ctx
                    .catalog
                    .policies
                    .get(usize::from(l.policy))
                    .is_some_and(|d| d.kind == PolicyKind::CommonStore)
        }) else {
            return;
        };
        let Some(t) = self.household(to) else {
            return;
        };
        let need = t.members.len() as f64 * params.household.daily_kcal_per_person;
        let held = stock_kcal(&stores_now(t, now, params, goods), goods);
        let law = &self.polities[pi].laws[li];
        let mut want = (f64::from(law.relief_days) * need)
            .min(params.household.food_target_days * need - held)
            .max(0.0);
        if want <= 0.0 {
            return;
        }
        let sheltered = self.polity_sheltered(pi);
        self.polities[pi].settle_stores(now, goods, sheltered);
        let store = self.polities[pi].id;
        let stores = self.polities[pi].stores.clone();
        let mut order: Vec<usize> = (0..goods.len())
            .filter(|&i| {
                goods[i].purpose == GoodUse::Food
                    && goods[i].kcal_per_kg > 0.0
                    && !goods[i].kept_back()
            })
            .collect();
        let keeps = |i: usize| match goods[i].half_life_days {
            h if h > 0.0 => h,
            _ => f64::INFINITY,
        };
        order.sort_by(|&a, &b| keeps(a).total_cmp(&keeps(b)).then(a.cmp(&b)));
        let mut carry = params.household.carry_kg;
        let mut legs = Vec::new();
        for i in order {
            if want <= 0.0 || carry <= 0.0 {
                break;
            }
            let kg = (want / goods[i].kcal_per_kg)
                .min(stores.get(i).copied().unwrap_or(0.0).max(0.0))
                .min(carry);
            if kg <= 1e-9 {
                continue;
            }
            legs.push(Leg {
                from: store,
                to,
                good: i,
                amount: kg,
            });
            carry -= kg;
            want -= kg * goods[i].kcal_per_kg;
        }
        let kg: f64 = legs.iter().map(|l| l.amount).sum();
        let kcal: f64 = legs
            .iter()
            .map(|l| l.amount * goods[l.good].kcal_per_kg)
            .sum();
        let given = !legs.is_empty() && self.transfer(now, params, goods, &legs, Channel::Relief);
        let store_law = self.polities[pi].laws[li].id;
        let c = &mut self.polities[pi].laws[li].compliance;
        if given {
            c.relieved += 1;
            c.relief_kg += kg;
            // What it gave redresses what was held against the store (M4c slice AE).
            self.redress_store(to, store_law, kcal / need.max(1.0));
        } else {
            c.unanswered += 1;
        }
    }
}

/// How near home a place is still on the household's own plot, metres: what a curfew allows (a
/// tuning value, about the reach of a home's own buildings).
pub(crate) const HOME_PLOT_M: f32 = 20.0;

/// Whether doing what `steps` say, from `pos`, takes someone off the plot of their home `home`:
/// a walk anywhere farther, or work where they stand while they are already farther. A walk home
/// is never away.
pub(crate) fn away_from_home(steps: &[Step], pos: (f32, f32), home: (f32, f32)) -> bool {
    let off = |at: (f32, f32)| (at.0 - home.0).hypot(at.1 - home.1) > HOME_PLOT_M;
    let mut walked = false;
    for s in steps {
        if let Step::Walk { to } = s {
            walked = true;
            if off(*to) {
                return true;
            }
        }
    }
    !walked && off(pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn going_out_is_away_and_walking_home_is_not() {
        let home = (100.0, 100.0);
        let walk = |to| Step::Walk { to };
        let work = Step::Work { minutes: 30 };
        // From home to the hearth 40 m off, and back: away.
        assert!(away_from_home(
            &[walk((140.0, 100.0)), work, walk(home)],
            home,
            home
        ));
        // Home from the hearth, to sleep: never away.
        assert!(!away_from_home(&[walk(home), work], (140.0, 100.0), home));
        // Work where they stand: away only if they stand off the plot.
        assert!(away_from_home(&[work], (140.0, 100.0), home));
        assert!(!away_from_home(&[work], (110.0, 100.0), home));
        // A store beside the home is on its plot.
        assert!(!away_from_home(&[walk((115.0, 105.0)), work], home, home));
    }
}
