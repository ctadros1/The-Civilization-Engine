//! The polity at work (ADR-0013): each settlement's polity founded under the custom every world
//! shares, the weekly review in which those who may propose weigh their moves, the gathering at
//! the hearth that decides, the levy at threshing and relief from the store, and word of a law
//! going round. Every choice here is a person's, scored by what they forecast for their household
//! and those who regard them; no issue weighs toward any policy.

use super::*;
use crate::decide::GatheringFacts;
use crate::polity::{
    Belief, Deliberator, Gathering, IssueKind, Law, LawStatus, MoveOption, Outcome, Outlook,
    PolicyKind, Polity, RuleDeliberator, Stance, StanceRecord,
};

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
        self.tell_households(day);
        for pi in 0..self.polities.len() {
            let p = &self.polities[pi];
            if day - p.reviewed >= i64::from(ctx.params.polity.review_days.max(1)) {
                self.polities[pi].reviewed = day;
                self.review(ctx, pi);
            }
        }
    }

    /// A law that names someone lapses when they die or leave the settlement: the store they
    /// kept lies unkept again, which is a vacancy the next review may answer (succession,
    /// ADR-0013 §2). Nothing fills it by itself.
    fn lapse_keepers(&mut self, ctx: &mut Ctx) {
        for pi in 0..self.polities.len() {
            let settlement = self.polities[pi].settlement;
            let Some((holder, law)) = self.polities[pi].keeper().map(|(h, l)| (h, l.id)) else {
                continue;
            };
            let here = self
                .person(holder)
                .and_then(|p| self.household(p.household))
                .is_some_and(|x| x.settlement == Some(settlement));
            if here {
                continue;
            }
            let sheltered = self.polity_sheltered(pi);
            self.polities[pi].settle_stores(ctx.now, &ctx.catalog.goods, sheltered);
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
            let words = format!(
                "{} no longer keeps the common store at {name}: {why}.",
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
    fn members_of(
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

    /// The issues settlement `settlement` faces now (research 09-05 §1.1): its food ran short
    /// within the last year, or a household's food will not last until its next harvest
    /// (`pressed`); and its common store holds food nobody keeps under a roof (`unkept`).
    fn issues(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        pressed: bool,
        unkept: bool,
    ) -> Vec<IssueKind> {
        let short_now = ctx
            .land
            .settlements
            .iter()
            .any(|s| s.id == settlement && s.food_short);
        let since = ctx.now.day_index() - 365;
        let short_lately = self.chronicle.iter().rev().any(|e| {
            e.kind == ChronicleKind::FoodRanShort
                && e.settlement == Some(settlement)
                && e.at.day_index() >= since
        });
        let mut out = Vec::new();
        if short_now || short_lately || pressed {
            out.push(IssueKind::FoodShort);
        }
        if unkept {
            out.push(IssueKind::StoreUnkept);
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
        let issues = self.issues(ctx, settlement, !pressed.is_empty(), unkept);
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
        if open.is_empty() {
            return;
        }
        let members = self.members_of(settlement, now, params);
        let outlooks: Vec<(PermanentId, Outlook)> = households
            .iter()
            .filter_map(|&h| Some((h, self.outlook(ctx, h)?)))
            .collect();
        let just: Vec<Outlook> = outlooks.iter().map(|o| o.1).collect();
        let belief = self.belief(ctx, settlement, &just);
        // What a roof over the store would keep for each household a year.
        let kept = self.roof_saves(ctx, pi) / outlooks.len().max(1) as f64;
        // Who could keep the store: adults whose household is under a roof.
        let roofed: Vec<PermanentId> = members
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
                        .filter_map(|&h| self.elder_of(h, now, params)),
                )
                .chain(reached)
                .collect()
        };
        deliberators.sort_unstable();
        deliberators.dedup();
        deliberators.retain(|d| members.binary_search_by_key(d, |m| m.0).is_ok());
        let day = now.day_index();
        let tp = &params.ties;
        let deliberator = RuleDeliberator { params: pp };
        // What a move would bring household `h`: a store at a levy share, or a store kept.
        let gain_of = |m: &MoveOption, h: PermanentId| {
            outlooks
                .binary_search_by_key(&h, |o| o.0)
                .map_or(0.0, |i| match m.nominee {
                    Some(_) => crate::polity::kept_gain(&outlooks[i].1, kept, pp),
                    None => crate::polity::store_gain(&outlooks[i].1, &belief, m.levy_share, pp),
                })
        };
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
            for &(k, issue) in &open {
                let def = &ctx.catalog.policies[usize::from(k)];
                let blank = MoveOption {
                    policy: k,
                    levy_share: 0.0,
                    issue,
                    nominee: None,
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
                }
            }
            for m in &mut moves {
                // Those who regard them, by how much; and those they know, by where they would
                // stand with them as its sponsor (or with the one it names).
                let face = m.nominee.unwrap_or(d);
                let (mut weighed, mut weight, mut support, mut oppose) = (0.0, 0.0, 0, 0);
                for &(a, h) in &members {
                    if a == d {
                        continue;
                    }
                    let gain = gain_of(m, h);
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
                m.own_gain = gain_of(m, own);
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
                    .filter(|l| l.policy == m.policy && l.holder == m.nominee)
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
            let u =
                Rng64::from_key(&[ctx.seed, PURPOSE_DELIBERATE, d.get(), day as u64]).next_f64();
            let choice = deliberator.choose(&moves, u);
            if let Some(m) = choice.chosen.map(|i| moves[i]) {
                let stakes: Vec<(PermanentId, f32)> = outlooks
                    .iter()
                    .map(|(h, _)| (*h, (pp.w_gain * gain_of(&m, *h)) as f32))
                    .collect();
                self.propose(ctx, pi, d, m, stakes);
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
    ) {
        let (now, pp) = (ctx.now, &ctx.params.polity);
        let day = now.day_index();
        let def = &ctx.catalog.policies[usize::from(m.policy)];
        let id = ctx.ids.allocate();
        let meets = day + i64::from(pp.notice_days.max(1));
        let law = Law {
            id,
            policy: m.policy,
            levy_share: m.levy_share as f32,
            holder: m.nominee,
            relief_days: def.relief_days as f32,
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
        };
        // "Mira proposed themselves as keeper of the common store", not "Mira as keeper".
        let words = format!(
            "{}, because {}",
            crate::polity::law_words(&law, &ctx.catalog.policies, &|id| {
                if id == sponsor {
                    "themselves".to_owned()
                } else {
                    self.name_of(id)
                }
            }),
            m.issue.words()
        );
        let polity = &mut self.polities[pi];
        polity.laws.push(law);
        polity.gathering = Some(Gathering {
            law: id,
            day: meets,
            stakes,
            present: Vec::new(),
        });
        let settlement = polity.settlement;
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

    /// The gathering called at polity `pi` has sat (ADR-0013 §3, stages 2-4): each member who
    /// came takes a stance from what their household made of the law and their regard for its
    /// sponsor, the body decides by its rule, and if it passed, those who came know it. A thin
    /// gathering or a tie is recorded as what it was.
    fn decide_gathering(&mut self, ctx: &mut Ctx, pi: usize) {
        let (now, params) = (ctx.now, ctx.params);
        let pp = &params.polity;
        let Some(g) = self.polities[pi].gathering.take() else {
            return;
        };
        let settlement = self.polities[pi].settlement;
        let members = self.members_of(settlement, now, params);
        let Some(law) = self.polities[pi].laws.iter().find(|l| l.id == g.law) else {
            return;
        };
        let sponsor = law.sponsor;
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
                let (stance, regard_points) = if p == sponsor {
                    (Stance::Support, 0.0)
                } else {
                    crate::polity::stance(gain, regard, pp)
                };
                Some(StanceRecord {
                    person: p,
                    household,
                    stance,
                    gain: gain as f32,
                    regard: regard_points as f32,
                })
            })
            .collect();
        let polity = &mut self.polities[pi];
        let body = polity.body;
        let Some(law) = polity.law_mut(g.law) else {
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
            .position(|l| l.id == g.law)
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
        let tally = format!("{support} for, {oppose} against; {present} of {eligible} adults came");
        let words = match outcome {
            Outcome::Passed => format!("The gathering at {name} agreed to {what}: {tally}."),
            Outcome::Failed => format!("The gathering at {name} turned down {what}: {tally}."),
            Outcome::Tied => format!(
                "The gathering at {name} was evenly split on {what}, so it failed: {tally}."
            ),
            Outcome::NoQuorum => format!(
                "Too few came to the gathering at {name} to decide on {what}: {present} of \
                 {eligible} adults, where {} were needed.",
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
        let polity = &self.polities[self.polity_of(hh.settlement?)?];
        let g = polity.gathering.as_ref()?;
        let end = evening_start + i64::from(pp.gathering_minutes);
        if g.day != ctx.now.day_index()
            || minute < evening_start
            || minute + LEAST_SITTING_MIN > end
            || g.present.binary_search(&person).is_ok()
        {
            return None;
        }
        let sponsor = polity.laws.iter().find(|l| l.id == g.law)?.sponsor;
        let regard = self
            .ties
            .regard(person, sponsor, ctx.now.day_index(), &params.ties)
            .clamp(0.0, 1.0);
        let stake = g.stake(hh.id).abs() + pp.w_regard * regard;
        Some(GatheringFacts {
            points: pp.attend_base + pp.w_attend * stake,
            minutes: (end - minute) as f64,
        })
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
            let points = crate::polity::comply_points(stance, regard, cost, pp);
            let chance = crate::polity::comply_chance(points);
            let key = [ctx.seed, PURPOSE_LEVY, who.get(), now.minutes() as u64];
            if Rng64::from_key(&key).next_f64() >= chance {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.evaded += 1;
                c.withheld_kg += owed;
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
        let given = !legs.is_empty() && self.transfer(now, params, goods, &legs, Channel::Relief);
        let c = &mut self.polities[pi].laws[li].compliance;
        if given {
            c.relieved += 1;
            c.relief_kg += kg;
        } else {
            c.unanswered += 1;
        }
    }
}
