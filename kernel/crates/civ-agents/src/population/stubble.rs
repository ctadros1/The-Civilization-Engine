//! The stubble kept for those who cannot earn the harvest wage (§8 17, fourth, slice 2; research
//! 08-22 §1, §4): the Islip (1407) and Wymeswold bylaws barred from gleaning anyone who could earn
//! a penny, or two pence, a day at the harvest, so that what reaping leaves fed those who could
//! not, and the able went to the harvest wage. Here it is a law a polity may come to by its own
//! means: those who glean, and would glean what the able now take, may propose it; each household
//! weighs it by what its own gleaning would gain and what its own able would give up; once in
//! force, one of an age to work for hire who knows it weighs keeping it when they would glean, as
//! a curfew is kept, unless their household is going hungry.

use civ_core::{Handle, PermanentId};

use super::{Ctx, Population, may_eat_reserve, reapers};
use crate::person::{Household, Person, Target};
use crate::polity::{LawStatus, PolicyKind};

/// Findings of a breach a year that a household whose able glean on against a law keeping the
/// stubble expects to owe the law's fine for (§8 20, seventh; research 08-22 §4: the bylaws were
/// kept by the manor court, which fined those presented at its sitting): one, the harvest's
/// gleaning heard once. A design prior.
pub const STUBBLE_FINDINGS_A_YEAR: f64 = 1.0;

impl Population {
    /// What keeping the stubble for those who cannot earn would bring each of `households` a year
    /// and cost it, kcal: what the polity's people who could earn the harvest wage gleaned, a
    /// year at the rate the record fades, shared among the households whose others gleaned by
    /// what each of them gleaned (they would glean it in the place of those barred); less what
    /// its own who could earn gleaned. None where content has no such law, nobody who could earn
    /// gleaned, or nobody else did.
    pub(super) fn stubble_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64, f64)> {
        if !ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::KeepStubble)
        {
            return Vec::new();
        }
        let (day, params) = (ctx.now.day_index(), ctx.params);
        let half = params.places.use_half_life_days;
        let kcal = ctx
            .catalog
            .crops
            .get(params.farm.crop)
            .and_then(|c| ctx.catalog.goods.get(c.good))
            .map_or(0.0, |g| g.kcal_per_kg);
        let held: Vec<(PermanentId, f64, f64)> = households
            .iter()
            .filter_map(|&h| {
                let i = self
                    .order
                    .gleaners
                    .binary_search_by_key(&h, |g| g.household)
                    .ok()?;
                let (able, other) = self.order.gleaners[i].at(day, half);
                let year = |kg: f64| crate::uses::per_year(kg, half) * kcal;
                Some((h, year(able), year(other)))
            })
            .collect();
        let able: f64 = held.iter().map(|g| g.1).sum();
        let others: f64 = held.iter().map(|g| g.2).sum();
        if able <= 0.0 || others <= 0.0 {
            return Vec::new();
        }
        held.iter()
            .map(|&(h, a, o)| (h, able * o / others, a))
            .collect()
    }

    /// Whether activity `def` at `target` by someone of `hh` is gleaning on its polity's own
    /// stubble, where a law keeping the stubble binds (§8 19, seventh; research 08-22 §1: the
    /// custom was the place's own, its bylaws its own fields'). Elsewhere it is not the polity's
    /// to keep.
    pub(super) fn stubble_binds(
        &self,
        ctx: &Ctx,
        hh: &Household,
        def: u16,
        target: Target,
    ) -> bool {
        let gleans = ctx
            .catalog
            .activities
            .get(usize::from(def))
            .filter(|a| a.behavior == crate::params::Behavior::Gather)
            .and_then(|a| a.resource)
            .filter(|&r| ctx.land_params.resources.get(r).is_some_and(|r| r.reaped));
        let pi = hh.settlement.and_then(|s| self.polity_of(s));
        match (gleans, pi) {
            (Some(r), Some(pi)) => self.own_stubble(ctx, pi, r, target),
            _ => false,
        }
    }

    /// Whether `person` gleaning by activity `def` at `target` would weigh a law keeping the
    /// stubble now: for tests.
    #[doc(hidden)]
    pub fn stubble_binds_for_tests(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        def: u16,
        target: Target,
    ) -> bool {
        let (Some(p), Some(hh)) = (
            self.person(person),
            self.person(person)
                .and_then(|p| self.household(p.household)),
        ) else {
            return false;
        };
        self.stubble_now(ctx, p, hh).is_some() && self.stubble_binds(ctx, hh, def, target)
    }

    /// Whether gleaning resource `r` at `target` is gleaning polity `pi`'s own stubble (§8 19,
    /// seventh; research 08-22 §1): a field held by one of its households lies within the
    /// gathering's block, whose gleanings these are.
    pub(super) fn own_stubble(&self, ctx: &Ctx, pi: usize, r: usize, target: Target) -> bool {
        let Target::Patch(centre) = target else {
            return false;
        };
        let Some(res) = ctx.land_params.resources.get(r) else {
            return false;
        };
        let patches = &ctx.land.patches;
        let cols = patches.cols.max(1);
        let k = res.range_patches;
        let (cx, cy) = (centre % cols, centre / cols);
        ctx.land.fields.iter().any(|f| {
            let (x, y) = f.rect.centre_m();
            let fp = patches.of_point(x, y) as u32;
            let (fx, fy) = (fp % cols, fp / cols);
            fx.abs_diff(cx) <= k
                && fy.abs_diff(cy) <= k
                && self
                    .household(f.household)
                    .and_then(|x| x.settlement)
                    .is_some_and(|s| self.polities[pi].holds(Some(s)))
        })
    }

    /// Person `h` of `hh`, who could earn the harvest wage and knows a law keeping the stubble,
    /// gleans the polity's own stubble at `target` against it (§8 19, seventh, slice 2): the law
    /// counts a breach known, and, where anyone else was gleaning or working the same block just
    /// then, records it with who saw it (in id order), to be brought as a case.
    pub(super) fn stubble_breached(
        &mut self,
        ctx: &Ctx,
        h: Handle<Person>,
        hh: &Household,
        target: Target,
    ) {
        let Some(who) = self.people.get(h).map(|p| p.id) else {
            return;
        };
        let Some(pi) = hh.settlement.and_then(|s| self.polity_of(s)) else {
            return;
        };
        let Some(li) = self.polities[pi].laws.iter().position(|l| {
            l.kind == PolicyKind::KeepStubble
                && (l.status == LawStatus::InForce || self.keeps_struck(ctx, who, l))
        }) else {
            return;
        };
        let Target::Patch(centre) = target else {
            return;
        };
        let k = ctx
            .catalog
            .activities
            .iter()
            .filter_map(|a| a.resource)
            .filter_map(|r| ctx.land_params.resources.get(r))
            .filter(|r| r.reaped)
            .map(|r| r.range_patches)
            .max()
            .unwrap_or(0);
        let patches = &ctx.land.patches;
        let cols = patches.cols.max(1);
        let (cx, cy) = (centre % cols, centre / cols);
        let in_block = |p: u32| (p % cols).abs_diff(cx) <= k && (p / cols).abs_diff(cy) <= k;
        // Anyone else gleaning or working the same block (research 08-22 §4: the bylaws were
        // kept by neighbours in the field).
        let mut seen_by: Vec<PermanentId> = self
            .people
            .iter()
            .filter(|(_, q)| q.id != who && q.household != hh.id)
            .filter(|(_, q)| match q.act.target {
                Target::Patch(p) => in_block(p),
                Target::Field(f) => ctx.land.fields.iter().any(|x| {
                    x.id == f && {
                        let (fx, fy) = x.rect.centre_m();
                        in_block(patches.of_point(fx, fy) as u32)
                    }
                }),
                _ => false,
            })
            .map(|(_, q)| q.id)
            .collect();
        seen_by.sort_unstable();
        let day = ctx.now.day_index();
        let polity = &mut self.polities[pi];
        let law = polity.laws[li].id;
        polity.laws[li].compliance.broken += 1;
        if !seen_by.is_empty() {
            polity.stubble_breaches.push(crate::polity::StubbleBreach {
                day,
                law,
                person: who,
                household: hh.id,
                seen_by,
                case: None,
            });
        }
    }

    /// `person` gleans at `target` against a law keeping the stubble: for tests.
    #[doc(hidden)]
    pub fn stubble_breach_for_tests(&mut self, ctx: &Ctx, person: PermanentId, target: Target) {
        let (Some(&h), Some(hh)) = (
            self.index.get(&person),
            self.person(person)
                .and_then(|p| self.household(p.household))
                .cloned(),
        ) else {
            return;
        };
        self.stubble_breached(ctx, h, &hh, target);
    }

    /// [`Population::weigh_stubble_cases`] at polity `pi`: for tests.
    #[doc(hidden)]
    pub fn weigh_stubble_cases_for_tests(&mut self, ctx: &mut Ctx, pi: usize) {
        self.weigh_stubble_cases(ctx, pi);
    }

    /// Cases of the stubble gleaned against a law keeping it at polity `pi` (§8 19, seventh,
    /// slice 2), weighed at each review as a plan's encroachments are: for each breach of a law
    /// in force not yet brought, within the days a taking is remembered, each who saw it, grown,
    /// of the polity, knowing the law and not of the gleaner's household, weighs bringing it: their
    /// stance on the law and the norms they hold, less the cost of reporting and their regard for
    /// the gleaner. What a finding imposes goes to the common store, so it gives their household
    /// nothing. The keenest, if keen at all, brings it against the gleaner.
    pub(super) fn weigh_stubble_cases(&mut self, ctx: &mut Ctx, pi: usize) {
        use crate::crime::{Case, CaseStage};
        let (now, params) = (ctx.now, ctx.params);
        let (pp, cp, day) = (&params.polity, &params.crime, now.day_index());
        let since = day - i64::from(cp.remember_days);
        let pending: Vec<(usize, crate::polity::StubbleBreach)> = self.polities[pi]
            .stubble_breaches
            .iter()
            .enumerate()
            .filter(|(_, b)| b.case.is_none() && b.day >= since)
            .filter(|(_, b)| {
                self.polities[pi]
                    .laws
                    .iter()
                    .any(|l| l.id == b.law && l.status == LawStatus::InForce)
            })
            .map(|(i, b)| (i, b.clone()))
            .collect();
        for (bi, b) in pending {
            if self.person(b.person).is_none() {
                continue;
            }
            let Some(law) = self.polities[pi]
                .laws
                .iter()
                .find(|l| l.id == b.law)
                .cloned()
            else {
                continue;
            };
            let mut best: Option<(f64, PermanentId, PermanentId)> = None;
            for &w in &b.seen_by {
                let Some(person) = self.person(w) else {
                    continue;
                };
                let household = person.household;
                if household == b.household
                    || person.age_years(now) < params.family.independent_age
                    || !self
                        .household(household)
                        .is_some_and(|x| self.polities[pi].holds(x.settlement))
                    || !law.knows(w)
                {
                    continue;
                }
                let stance = law
                    .stances
                    .iter()
                    .find(|r| r.person == w)
                    .map_or(0.0, |r| r.stance.sign());
                let norm = self.norm_points(ctx, w);
                let regard = self
                    .ties
                    .regard(w, b.person, day, &params.ties)
                    .clamp(0.0, 1.0);
                let u = pp.w_stance * stance + norm - cp.report_cost - pp.w_regard * regard;
                if u > 0.0 && best.is_none_or(|x| u > x.0) {
                    best = Some((u, w, household));
                }
            }
            let Some((_, by, household)) = best else {
                continue;
            };
            let settlement = self.polities[pi].settlement;
            let id = self.order.cases.last().map_or(1, |c| c.id + 1);
            self.order.cases.push(Case {
                id,
                incident: 0,
                settlement,
                law: b.law,
                accuser: household,
                by,
                accused: b.person,
                accused_household: b.household,
                kcal: 0.0,
                leads: vec![by],
                opened: day,
                stage: CaseStage::Open,
                heard: None,
                eligible: 0,
                stances: Vec::new(),
                exiled: false,
                forum: None,
                challenge: None,
                tags: 0,
                departed: None,
                loan: None,
                building: None,
            });
            self.polities[pi].stubble_breaches[bi].case = Some(id);
            let (place, name) = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map_or((None, String::new()), |s| {
                    (Some(s.hearth_m), s.name.clone())
                });
            let words = format!(
                "brought a case at {name} against {} for gleaning the stubble the law keeps for \
                 those who cannot earn.",
                self.name_of(b.person)
            );
            self.chronicle_push(
                now,
                crate::history::ChronicleKind::CaseBrought,
                vec![by, b.person],
                Some(settlement),
                place,
                0.0,
                words,
            );
        }
    }

    /// What keeping the stubble for those who cannot earn weighs with `p` of `hh` now, points,
    /// when they would glean: a law of it in force in their polity (or one struck down they still
    /// keep), known to them, and they of an age to work for hire. None while someone of their
    /// household is going hungry (need over the gathering's word, as for a claim).
    pub(super) fn stubble_now(&self, ctx: &Ctx, p: &Person, hh: &Household) -> Option<f64> {
        let (now, params) = (ctx.now, ctx.params);
        if !reapers::could_earn(ctx.catalog, p.age_years(now)) {
            return None;
        }
        let pi = self.polity_of(hh.settlement?)?;
        let law = self.polities[pi].laws.iter().find(|l| {
            l.kind == PolicyKind::KeepStubble
                && (l.status == LawStatus::InForce || self.keeps_struck(ctx, p.id, l))
        })?;
        if !law.knows(p.id) {
            return None;
        }
        let hungry = hh.members.iter().any(|&m| {
            self.person(m)
                .is_some_and(|q| may_eat_reserve(q, now, params))
        });
        if hungry {
            return None;
        }
        let stance = law
            .stances
            .iter()
            .find(|r| r.person == p.id)
            .map_or(0.0, |r| r.stance.sign());
        let regard = self
            .ties
            .regard(p.id, law.sponsor, now.day_index(), &params.ties);
        let norm = self.norm_points(ctx, p.id);
        Some(crate::polity::comply_points(norm, stance, regard, 0.0, &params.polity).max(0.0))
    }

    /// What a law keeping the stubble for those who cannot earn would bring each household of
    /// settlement `settlement` a year and cost it, kcal: for tests.
    #[doc(hidden)]
    pub fn stubble_forecasts_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Vec<(PermanentId, f64, f64)> {
        let Some(pi) = self.polity_of(settlement) else {
            return Vec::new();
        };
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.stubble_forecasts(ctx, &households)
    }

    /// What a law keeping the stubble with `sanction` would bring each household of settlement
    /// `settlement`, points, as its members would weigh it at the gathering (§8 20, seventh): for
    /// tests.
    #[doc(hidden)]
    pub fn stubble_gains_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        sanction: crate::polity::Sanction,
    ) -> Vec<(PermanentId, f64)> {
        let Some(pi) = self.polity_of(settlement) else {
            return Vec::new();
        };
        let Some(policy) = ctx
            .catalog
            .policies
            .iter()
            .position(|d| d.kind == PolicyKind::KeepStubble)
        else {
            return Vec::new();
        };
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let adults = self.members_of(settlement, ctx.now, ctx.params);
        let fc = self.forecasts(ctx, pi, &households, &adults);
        let m = crate::polity::MoveOption {
            policy: policy as u16,
            levy_share: 0.0,
            issue: crate::polity::IssueKind::FoodShort,
            nominee: None,
            sanction,
            hours: (0, 0),
            body: None,
            ends: None,
            site: None,
            target: None,
            own_gain: 0.0,
            followers_gain: 0.0,
            support: 0.5,
        };
        let pp = &ctx.params.polity;
        households
            .iter()
            .map(|&h| {
                (
                    h,
                    pp.w_gain * fc.gain(&m, h, &ctx.catalog.policies, ctx.params),
                )
            })
            .collect()
    }

    /// What keeping the stubble weighs with `person` now when they would glean, points: for
    /// tests.
    #[doc(hidden)]
    pub fn stubble_points_for_tests(&self, ctx: &Ctx, person: PermanentId) -> Option<f64> {
        let p = self.person(person)?;
        let hh = self.household(p.household)?;
        self.stubble_now(ctx, p, hh)
    }
}
