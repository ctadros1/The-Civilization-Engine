//! Bands (M6c slice BJ, ADR-0023 §3-4): who calls one and why, who joins, who comes at the
//! muster, the walk there, the encounter, what is taken, and the walk home. Every choice here is
//! a person's, from what they believe: the stores they hope to find, the defenders they know of,
//! their grievances and their regard for those who go. The encounter itself is `war::fight`'s.

use super::*;
use crate::Cause;
use crate::crime::{Belief, Incident, KnownAmount, Outcome, Source, logistic, report_points};
use crate::decide::BandFacts;
use crate::polity::{LawStatus, PolicyKind};
use crate::ties::Act;
use crate::war::{Band, BandPurpose, BandStage, Clash, Ended, Fighter, Holding, Side, fight};
use crate::word::{Blamed, Claim, ClaimKind};

/// Purpose tag for a person's reluctance to call a band (keyed per person and day).
pub const PURPOSE_CALL: u64 = 0x6361_6c6c_6261_6e64; // "callband"
/// Purpose tag for a person's reluctance to join one (keyed per person and band).
pub const PURPOSE_JOIN: u64 = 0x6a6f_696e_6261_6e64; // "joinband"
/// Purpose tag for a household's choice of whom it sends with a band a law sent.
pub const PURPOSE_SERVE: u64 = 0x7365_7276_6562_616e; // "servebnd"
/// Purpose tag for someone's choice to bring a raid before their own gathering.
pub const PURPOSE_RAID_CASE: u64 = 0x7261_6964_6361_7365; // "raidcase"
/// Purpose tag for an encounter's blows.
pub const PURPOSE_CLASH: u64 = 0x636c_6173_6862_6c77; // "clashblw"

/// Metres from its home within which someone counts as at home.
const AT_HOME_M: f32 = 2.0;
/// The steps of the band activity: to the hearth, the wait for the muster, to the target, the
/// stay there, and home.
const STEP_MUSTER: u8 = 1;
const STEP_THERE: u8 = 3;
const STEP_HOME: u8 = 4;

/// What someone believes a band would meet and bring home (ADR-0023 §3).
#[derive(Clone, Copy, Debug)]
struct Prospect {
    purpose: BandPurpose,
    target: PermanentId,
    place: (f32, f32),
    /// Walking minutes from the hearth.
    walk_min: f64,
    /// A load of food, kcal, as they hope to find it.
    load_kcal: f64,
    /// Those they believe would stand against it.
    defenders: f64,
}

impl Population {
    /// Midnight (ADR-0023 §3): each adult due their monthly review, or whose household came to
    /// know since yesterday that people of another settlement took from it, weighs calling a band;
    /// one who calls it tells those they regard, who each choose to join or not. Bands left
    /// waiting on a muster nobody came to disperse, and bands out longer than a day come home.
    pub(super) fn band_day(&mut self, ctx: &mut Ctx) {
        let (now, day, params) = (ctx.now, ctx.now.day_index(), ctx.params);
        let bp = &params.bands;
        let minute = now.minutes();
        for b in &mut self.order.bands {
            match b.stage {
                BandStage::Called if minute > b.muster + MINUTES_PER_DAY => {
                    b.stage = BandStage::Dispersed;
                    b.since = minute;
                }
                BandStage::Out if minute > b.since + MINUTES_PER_DAY => {
                    b.stage = BandStage::Home;
                    b.since = minute;
                }
                _ => {}
            }
        }
        self.bands_by_law(ctx);
        self.raid_reports(ctx);
        if bp.review_days == 0 {
            return;
        }
        let busy: Vec<PermanentId> = self
            .order
            .bands
            .iter()
            .filter(|b| !b.stage.over())
            .flat_map(|b| b.members.iter().copied())
            .collect();
        let mut due: Vec<(PermanentId, PermanentId)> = Vec::new();
        for (_, p) in self.people.iter() {
            let age = p.age_years(now);
            if age < bp.min_age || age > bp.max_age || busy.contains(&p.id) {
                continue;
            }
            let raided = self.raided_within(p.household, day - 1);
            let review = (p.id.get() as i64 + day).rem_euclid(i64::from(bp.review_days)) == 0;
            if raided || review {
                due.push((p.id, p.household));
            }
        }
        due.sort_unstable();
        for (p, hh) in due {
            if self
                .order
                .bands
                .iter()
                .any(|b| !b.stage.over() && b.joined(p))
            {
                continue;
            }
            let Some(pros) = self.prospect(ctx, p, hh, None) else {
                continue;
            };
            let worth = self.call_worth(ctx, p, hh, &pros);
            let [lo, hi] = bp.call_threshold;
            let u = Rng64::from_key(&[ctx.seed, PURPOSE_CALL, p.get(), day as u64]).next_f64();
            if worth <= lo + (hi - lo) * u {
                continue;
            }
            self.call_band(ctx, p, hh, pros);
        }
    }

    /// The settlement whose people raided household `household` most lately since day `since`, as
    /// its members know it (M6c slice BJ): a raid on their settlement they heard of that came to
    /// the household, or a taking from it whose taker one of them named.
    pub(super) fn raiders_known(&self, household: PermanentId, since: i64) -> Option<PermanentId> {
        let x = self.household(household)?;
        let mut best: Option<(i64, PermanentId)> = None;
        let mut note = |day: i64, s: PermanentId| {
            if day >= since && best.is_none_or(|b| (day, s) > b) {
                best = Some((day, s));
            }
        };
        for &m in &x.members {
            for h in self.word.heard_by(m) {
                let Some(c) = self
                    .word
                    .claim(h.claim)
                    .filter(|c| c.kind == ClaimKind::Raid && c.subject == Some(household))
                else {
                    continue;
                };
                let band = c
                    .event
                    .and_then(|id| self.order.clashes.iter().find(|k| k.id == id))
                    .and_then(|k| self.order.bands.iter().find(|b| b.id == k.band));
                if let Some(b) = band {
                    note(c.day, b.settlement);
                }
            }
        }
        let start = self.order.amounts.partition_point(|a| a.day < since);
        for a in &self.order.amounts[start..] {
            if !a.lost || a.household != household {
                continue;
            }
            let Some(across) = self.order.incident(a.incident).and_then(|i| i.across) else {
                continue;
            };
            let named = x.members.iter().any(|&m| {
                self.order
                    .belief(m, a.incident)
                    .is_some_and(|b| b.taker.is_some())
            });
            if named {
                note(a.day, across);
            }
        }
        best.map(|b| b.1)
    }

    /// The band `organizer` of household `hh` would call, if any: to take back from a household
    /// of another settlement they know took from theirs, or else to take from the nearest home of
    /// a settlement their people have seen, within a band's walk of their hearth.
    fn prospect(
        &mut self,
        ctx: &Ctx,
        organizer: PermanentId,
        hh: PermanentId,
        only: Option<PermanentId>,
    ) -> Option<Prospect> {
        let (now, day, params) = (ctx.now, ctx.now.day_index(), ctx.params);
        let bp = &params.bands;
        let settlement = self.household(hh)?.settlement?;
        let hearth = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)?
            .hearth_m;
        self.home_field(ctx, settlement, hearth);
        let x = self.household(hh)?;
        let reach = &self.homes.get(&settlement)?.reach;
        let goods = &ctx.catalog.goods;
        let density = goods
            .iter()
            .filter(|g| g.purpose == GoodUse::Food && !g.kept_back())
            .map(|g| g.kcal_per_kg)
            .fold(0.0, f64::max);
        let walk = |at: (f32, f32)| {
            reach
                .seconds_to(cell_of(ctx.map, at))
                .map(|s| f64::from(s) / 60.0)
        };
        let since = day - i64::from(params.crime.remember_days);
        // Takers of another settlement its people know of.
        let mut back: Option<(PermanentId, (f32, f32), f64)> = None;
        let start = self.order.amounts.partition_point(|a| a.day < since);
        for a in &self.order.amounts[start..] {
            if !a.lost || a.household != hh {
                continue;
            }
            let Some(i) = self
                .order
                .incident(a.incident)
                .filter(|i| i.across.is_some())
            else {
                continue;
            };
            let known = x.members.iter().any(|&m| {
                self.order
                    .belief(m, a.incident)
                    .is_some_and(|b| b.taker.is_some())
            });
            let Some((home, at)) = self
                .household(i.actor_household)
                .map(|t| (t.home, t.settlement))
            else {
                continue;
            };
            if only.is_some_and(|o| at != Some(o)) {
                continue;
            }
            if let (true, Some(w)) = (known, walk(home))
                && w <= bp.max_walk_minutes
            {
                back = Some((i.actor_household, home, w));
            }
        }
        let seen = self.homes_seen_elsewhere(hh, settlement);
        let (purpose, target, place, walk_min) = match back {
            Some((t, at, w)) => (BandPurpose::TakeBack, t, at, w),
            None => {
                let mut best: Option<(f64, PermanentId, (f32, f32))> = None;
                for (_, y) in self.households.iter() {
                    if y.members.is_empty() || y.settlement.is_none_or(|s| !seen.contains(&s)) {
                        continue;
                    }
                    if only.is_some_and(|o| y.settlement != Some(o)) {
                        continue;
                    }
                    if let Some(w) = walk(y.home).filter(|&w| w <= bp.max_walk_minutes)
                        && best.is_none_or(|(bw, bid, _)| w < bw || (w == bw && y.id < bid))
                    {
                        best = Some((w, y.id, y.home));
                    }
                }
                let (w, t, at) = best?;
                (BandPurpose::Take, t, at, w)
            }
        };
        let theirs = self.household(target)?;
        let food = theirs.settlement.map_or(bp.food_prior, |s| {
            self.known_places
                .of(hh)
                .iter()
                .find(|k| k.settlement == s)
                .and_then(|k| k.food)
                .map_or(bp.food_prior, f64::from)
        });
        let tp = &params.ties;
        let known = theirs
            .members
            .iter()
            .filter(|&&m| self.ties.tie(organizer, m, day, tp).is_some())
            .filter(|&&m| {
                self.person(m)
                    .is_some_and(|q| q.age_years(now) >= params.crime.guardian_age)
            })
            .count();
        Some(Prospect {
            purpose,
            target,
            place,
            walk_min,
            load_kcal: params.household.carry_kg * density * food,
            defenders: known as f64 + 1.0,
        })
    }

    /// What calling a band is worth to `p` of household `hh` (ADR-0023 §3; 13-05 §1.2): their
    /// household's load as they hope to find it, in days of its food, and their grievance against
    /// the household it goes to, less the danger of those they believe would stand against it
    /// (shared with at least one companion), the walk there and back, and their objection to
    /// taking.
    fn call_worth(&self, ctx: &Ctx, p: PermanentId, hh: PermanentId, pros: &Prospect) -> f64 {
        let params = ctx.params;
        let bp = &params.bands;
        let need = self.day_need(hh, params).max(1.0);
        let objection = self.person(p).map_or(1.0, |q| f64::from(q.objection));
        bp.w_spoils * pros.load_kcal / need
            + bp.w_grievance * self.keenness(ctx, p, Blamed::Household(pros.target))
            - bp.w_danger * pros.defenders / 2.0
            - params.decision.w_walk_hour * 2.0 * pros.walk_min / 60.0
            - bp.w_objection * objection
            - self.law_against_raiding(ctx, p).map_or(0.0, |_| bp.w_law)
    }

    /// The law against raiding in force where `p` lives, if they know of it (ADR-0023 §3: "any
    /// law against raiding they know"): its id and bundle.
    fn law_against_raiding(
        &self,
        ctx: &Ctx,
        p: PermanentId,
    ) -> Option<(PermanentId, crate::polity::Sanction)> {
        let settlement = self.household(self.person(p)?.household)?.settlement?;
        let polity = &self.polities[self.polity_of(settlement)?];
        polity
            .in_force(&ctx.catalog.policies, PolicyKind::AgainstRaiding)
            .find(|l| l.knows(p))
            .map(|l| (l.id, l.sanction))
    }

    /// What joining `organizer`'s band is worth to `q` (ADR-0023 §3): as calling it, with the
    /// danger shared among those who have joined, their regard for the organizer, and their
    /// regard for those who have joined.
    fn join_worth(
        &self,
        ctx: &Ctx,
        q: PermanentId,
        organizer: PermanentId,
        pros: &Prospect,
        joined: &[PermanentId],
    ) -> f64 {
        let (day, params) = (ctx.now.day_index(), ctx.params);
        let bp = &params.bands;
        let Some(hh) = self.person(q).map(|x| x.household) else {
            return f64::NEG_INFINITY;
        };
        let need = self.day_need(hh, params).max(1.0);
        let objection = self.person(q).map_or(1.0, |x| f64::from(x.objection));
        let tp = &params.ties;
        let regard = self.ties.regard(q, organizer, day, tp).clamp(0.0, 1.0);
        let follow = joined
            .iter()
            .map(|&m| self.ties.regard(q, m, day, tp).clamp(0.0, 1.0))
            .sum::<f64>()
            / joined.len().max(1) as f64;
        bp.w_spoils * pros.load_kcal / need
            + bp.w_grievance * self.keenness(ctx, q, Blamed::Household(pros.target))
            + bp.w_regard * regard
            + bp.w_follow * follow
            - bp.w_danger * pros.defenders / (joined.len() as f64 + 1.0)
            - params.decision.w_walk_hour * 2.0 * pros.walk_min / 60.0
            - bp.w_objection * objection
            - self.law_against_raiding(ctx, q).map_or(0.0, |_| bp.w_law)
    }

    /// `organizer` calls a band (ADR-0023 §3): they tell the adults of their settlement they
    /// regard, most regarded first, and each chooses to join or not; the band musters the next
    /// day at the content's hour if enough joined.
    fn call_band(
        &mut self,
        ctx: &mut Ctx,
        organizer: PermanentId,
        hh: PermanentId,
        pros: Prospect,
    ) {
        let (now, day, params) = (ctx.now, ctx.now.day_index(), ctx.params);
        let bp = &params.bands;
        let Some(settlement) = self.household(hh).and_then(|x| x.settlement) else {
            return;
        };
        let tp = &params.ties;
        let mut told: Vec<(f64, PermanentId)> = self
            .people
            .iter()
            .filter(|(_, q)| q.id != organizer)
            .filter(|(_, q)| {
                let age = q.age_years(now);
                age >= bp.min_age && age <= bp.max_age
            })
            .filter(|(_, q)| {
                self.household(q.household)
                    .is_some_and(|x| x.settlement == Some(settlement))
            })
            .map(|(_, q)| (self.ties.regard(organizer, q.id, day, tp), q.id))
            .filter(|&(r, _)| r >= bp.join_regard)
            .collect();
        told.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        told.truncate(bp.told_most as usize);
        let id = self.order.bands.last().map_or(1, |b| b.id + 1);
        let mut members = vec![organizer];
        let mut asked = Vec::new();
        for &(_, q) in &told {
            if self
                .order
                .bands
                .iter()
                .any(|b| !b.stage.over() && b.joined(q))
            {
                continue;
            }
            let worth = self.join_worth(ctx, q, organizer, &pros, &members);
            let [lo, hi] = bp.join_threshold;
            let u = Rng64::from_key(&[ctx.seed, PURPOSE_JOIN, q.get(), u64::from(id)]).next_f64();
            if worth > lo + (hi - lo) * u {
                members.push(q);
            } else {
                asked.push(q);
            }
        }
        asked.sort_unstable();
        if members.len() < bp.least.max(1) as usize {
            return;
        }
        let muster = (day + 1) * MINUTES_PER_DAY + i64::from(bp.muster_hour) * 60;
        self.order.bands.push(Band {
            id,
            organizer,
            settlement,
            purpose: pros.purpose,
            target: pros.target,
            place: pros.place,
            called: now.minutes(),
            muster,
            members,
            asked,
            mustered: Vec::new(),
            arrived: Vec::new(),
            home: Vec::new(),
            stage: BandStage::Called,
            since: now.minutes(),
            clash: None,
            incident: None,
            law: None,
            taken: Vec::new(),
            taken_kcal: 0.0,
        });
    }

    /// Each band a law sent (ADR-0023 §5): one not yet called is called the midnight after the
    /// law is in force, its leader the one the law names, against the settlement it names; each
    /// household of the settlement chooses one of its grown people to go, the one most willing,
    /// or keeps its share back. Once the band is home, or it never set off, the law has run its
    /// course and lapses.
    fn bands_by_law(&mut self, ctx: &mut Ctx) {
        let policies = &ctx.catalog.policies;
        let mut due: Vec<(usize, PermanentId, PermanentId, PermanentId)> = Vec::new();
        for (pi, polity) in self.polities.iter().enumerate() {
            for l in polity.in_force(policies, PolicyKind::SendBand) {
                if let (Some(leader), Some(target)) = (l.holder, l.target) {
                    due.push((pi, l.id, leader, target));
                }
            }
        }
        for (pi, law, leader, target) in due {
            let band = self.order.bands.iter().find(|b| b.law == Some(law));
            match band {
                Some(b) if !b.stage.over() => {}
                Some(_) => self.lapse_band_law(pi, law),
                None => {
                    if !self.call_band_by_law(ctx, pi, law, leader, target) {
                        self.lapse_band_law(pi, law);
                    }
                }
            }
        }
    }

    /// Law `law` of polity `pi`, which sent a band, has run its course.
    fn lapse_band_law(&mut self, pi: usize, law: PermanentId) {
        if let Some(l) = self.polities[pi].law_mut(law) {
            l.status = LawStatus::Lapsed;
        }
    }

    /// Calls the band law `law` of polity `pi` sends, led by `leader` against `target`: whether
    /// one was called.
    fn call_band_by_law(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        law: PermanentId,
        leader: PermanentId,
        target: PermanentId,
    ) -> bool {
        let (now, day, params) = (ctx.now, ctx.now.day_index(), ctx.params);
        let bp = &params.bands;
        let settlement = self.polities[pi].settlement;
        let Some(hh) = self
            .person(leader)
            .filter(|p| {
                let age = p.age_years(now);
                age >= bp.min_age && age <= bp.max_age
            })
            .map(|p| p.household)
        else {
            return false;
        };
        if self.household(hh).and_then(|x| x.settlement) != Some(settlement)
            || self
                .order
                .bands
                .iter()
                .any(|b| !b.stage.over() && b.joined(leader))
        {
            return false;
        }
        let Some(pros) = self.prospect(ctx, leader, hh, Some(target)) else {
            return false;
        };
        let id = self.order.bands.last().map_or(1, |b| b.id + 1);
        let busy: Vec<PermanentId> = self
            .order
            .bands
            .iter()
            .filter(|b| !b.stage.over())
            .flat_map(|b| b.members.iter().copied())
            .collect();
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && x.id != hh)
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let mut members = vec![leader];
        let mut asked = Vec::new();
        for h in households {
            let grown: Vec<PermanentId> = self
                .household(h)
                .map(|x| x.members.clone())
                .unwrap_or_default()
                .into_iter()
                .filter(|m| !busy.contains(m))
                .filter(|&m| {
                    self.person(m).is_some_and(|p| {
                        let age = p.age_years(now);
                        age >= bp.min_age && age <= bp.max_age
                    })
                })
                .collect();
            // The household sends the one most willing, or keeps its share back as a levy is
            // kept back (ADR-0023 §5): the custom that the gathering binds, where they stood on
            // the law and their regard for its sponsor, against the days of the household's
            // food the one it sends is away and at risk.
            let best = grown
                .iter()
                .map(|&m| (self.join_worth(ctx, m, leader, &pros, &members), m))
                .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
            let Some((_, m)) = best else {
                continue;
            };
            let Some(l) = self.polities[pi].law_mut(law) else {
                return false;
            };
            if !l.knows(m) {
                l.compliance.unaware += 1;
                asked.push(m);
                continue;
            }
            let stance = l
                .stances
                .iter()
                .find(|r| r.person == m)
                .map_or(0.0, |r| r.stance.sign());
            let sponsor = l.sponsor;
            let Some(outlook) = self.outlook(ctx, h) else {
                continue;
            };
            let pp = &params.polity;
            let regard = self.ties.regard(m, sponsor, day, &params.ties);
            let cost = pp.w_gain
                * crate::polity::levy_cost(
                    &outlook,
                    bp.service_days * outlook.year_need / 365.0,
                    pp,
                );
            self.ensure_norm_state(ctx, m);
            let norm = self.norm_points(ctx, m);
            let chance = crate::polity::comply_chance(crate::polity::comply_points(
                norm, stance, regard, cost, pp,
            ));
            let u = Rng64::from_key(&[ctx.seed, PURPOSE_SERVE, m.get(), u64::from(id)]).next_f64();
            if u < chance {
                members.push(m);
            } else {
                if let Some(l) = self.polities[pi].law_mut(law) {
                    l.compliance.evaded += 1;
                }
                asked.push(m);
            }
        }
        asked.sort_unstable();
        if members.len() < bp.least.max(1) as usize {
            return false;
        }
        let muster = (day + 1) * MINUTES_PER_DAY + i64::from(bp.muster_hour) * 60;
        self.order.bands.push(Band {
            id,
            organizer: leader,
            settlement,
            purpose: pros.purpose,
            target: pros.target,
            place: pros.place,
            called: now.minutes(),
            muster,
            members,
            asked,
            mustered: Vec::new(),
            arrived: Vec::new(),
            home: Vec::new(),
            stage: BandStage::Called,
            since: now.minutes(),
            clash: None,
            incident: None,
            law: Some(law),
            taken: Vec::new(),
            taken_kcal: 0.0,
        });
        true
    }

    /// A raid its own people made yesterday, where a law against raiding is in force (ADR-0023
    /// §5): each grown person of the band's settlement who heard of it, knows the law and did not
    /// go weighs bringing it before the gathering for the household raided, as one taken from
    /// would (research 09-07 §4.1), their regard for the one who led it against them; the first
    /// who chooses to brings it.
    fn raid_reports(&mut self, ctx: &mut Ctx) {
        let (day, params) = (ctx.now.day_index(), ctx.params);
        let cp = &params.crime;
        let due: Vec<(usize, u32)> = self
            .order
            .bands
            .iter()
            .enumerate()
            .filter_map(|(bi, b)| {
                let c = self.order.clashes.iter().find(|c| Some(c.id) == b.clash)?;
                (c.at.div_euclid(MINUTES_PER_DAY) == day - 1 && b.taken_kcal > 0.0)
                    .then_some((bi, c.id))
            })
            .collect();
        for (bi, clash) in due {
            let b = self.order.bands[bi].clone();
            let Some(incident) = b.incident else {
                continue;
            };
            if self.order.cases.iter().any(|c| c.incident == incident) {
                continue;
            }
            let Some(claim) = self
                .word
                .claims
                .iter()
                .find(|c| c.kind == ClaimKind::Raid && c.event == Some(clash))
                .map(|c| c.id)
            else {
                continue;
            };
            let need = self
                .household(b.target)
                .map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person;
            let mut hearers: Vec<PermanentId> = self
                .people
                .iter()
                .filter(|(_, p)| p.age_years(ctx.now) >= params.family.independent_age)
                .filter(|(_, p)| {
                    self.household(p.household).and_then(|x| x.settlement) == Some(b.settlement)
                })
                .map(|(_, p)| p.id)
                .filter(|p| b.mustered.binary_search(p).is_err())
                .filter(|&p| self.word.has_heard(p, claim))
                .collect();
            hearers.sort_unstable();
            for by in hearers {
                let Some((law, sanction)) = self.law_against_raiding(ctx, by) else {
                    continue;
                };
                let regard = self.ties.regard(by, b.organizer, day, &params.ties);
                let points = report_points(
                    1,
                    f64::from(b.taken_kcal) / need,
                    f64::from(sanction.compensation_days),
                    regard,
                    cp,
                );
                let key = [ctx.seed, PURPOSE_RAID_CASE, by.get(), u64::from(incident)];
                if Rng64::from_key(&key).next_f64() < logistic(points) {
                    self.bring_case_at(
                        ctx,
                        incident,
                        b.target,
                        by,
                        b.organizer,
                        f64::from(b.taken_kcal),
                        law,
                        b.settlement,
                    );
                    break;
                }
            }
        }
    }

    /// The band `person` joined that musters within the day, as a choice for them (ADR-0023 §3):
    /// what going is worth now they have said they will, where its hearth is, how long to wait
    /// there from `at`, where it goes and how long it stays.
    pub(crate) fn band_facts(
        &mut self,
        ctx: &Ctx,
        person: PermanentId,
        at: (f32, f32),
    ) -> Option<BandFacts> {
        let minute = ctx.now.minutes();
        let bp = &ctx.params.bands;
        let b = self.order.bands.iter().find(|b| {
            b.stage == BandStage::Called
                && b.joined(person)
                && minute < b.muster
                && b.muster - minute <= MINUTES_PER_DAY
        })?;
        let (id, settlement, muster, place) = (b.id, b.settlement, b.muster, b.place);
        let hearth = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)?
            .hearth_m;
        self.home_field(ctx, settlement, hearth);
        let walk = self
            .homes
            .get(&settlement)
            .and_then(|f| f.reach.seconds_to(cell_of(ctx.map, at)))
            .map_or(0.0, |s| f64::from(s) / 60.0);
        Some(BandFacts {
            band: id,
            points: bp.w_promise,
            hearth,
            wait_min: (muster as f64 - minute as f64 - walk).max(1.0),
            target: place,
            stay_min: bp.stay_minutes,
        })
    }

    /// `h` begins a work step of the band activity: at the hearth before the muster they are
    /// among those come; at the target they have arrived, and when every one who came at the
    /// muster and goes on has arrived, the encounter begins.
    pub(super) fn band_work(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        let Some(p) = self.people.get(h) else {
            return;
        };
        let (who, step) = (p.id, p.act.step);
        let Some(bi) = self
            .order
            .bands
            .iter()
            .position(|b| !b.stage.over() && b.joined(who))
        else {
            return;
        };
        let b = &mut self.order.bands[bi];
        match (step, b.stage) {
            (STEP_MUSTER, BandStage::Called) => {
                if let Err(i) = b.mustered.binary_search(&who) {
                    b.mustered.insert(i, who);
                }
            }
            (STEP_THERE, BandStage::Out) if b.mustered.binary_search(&who).is_ok() => {
                if !b.arrived.contains(&who) {
                    b.arrived.push(who);
                }
                let pending = b.clash.is_none();
                if pending && self.band_all_there(ctx, bi) {
                    self.clash(ctx, bi);
                }
            }
            _ => {}
        }
    }

    /// `h` ends a step of the band activity: the end of the wait at the hearth is the muster
    /// (those come go on, the rest go home); the end of the walk home is being home.
    pub(super) fn band_step_ended(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        let minute = ctx.now.minutes();
        let least = ctx.params.bands.least.max(1) as usize;
        let Some(p) = self.people.get(h) else {
            return;
        };
        let (who, step) = (p.id, p.act.step);
        let Some(bi) = self
            .order
            .bands
            .iter()
            .position(|b| !b.stage.over() && b.joined(who))
        else {
            return;
        };
        let first_out = !self.order.bands.iter().any(|b| b.stage == BandStage::Out);
        let mut set_off = false;
        let goes_on = {
            let b = &mut self.order.bands[bi];
            match step {
                STEP_MUSTER => {
                    if b.stage == BandStage::Called && minute + 1 >= b.muster {
                        b.since = minute;
                        b.stage = if b.mustered.len() >= least {
                            set_off = true;
                            BandStage::Out
                        } else {
                            BandStage::Dispersed
                        };
                    }
                    b.stage == BandStage::Out && b.mustered.binary_search(&who).is_ok()
                }
                STEP_HOME => {
                    if !b.home.contains(&who) {
                        b.home.push(who);
                    }
                    true
                }
                _ => true,
            }
        };
        if set_off {
            self.shirked(ctx, bi);
            // Those about look at a band walking while one is out.
            if first_out {
                let look = i64::from(ctx.params.bands.look_minutes.max(1));
                ctx.schedule
                    .push((ctx.now.plus_minutes(look), AgentEvent::BandLook));
            }
        }
        if !goes_on {
            // Not among those who went: home from the hearth.
            if let Some(p) = self.people.get_mut(h) {
                let next = p.act.step as usize + 1;
                p.act.steps.truncate(next);
            }
        }
        if step == STEP_HOME && self.band_all_home(bi) {
            let b = &mut self.order.bands[bi];
            b.stage = BandStage::Home;
            b.since = minute;
        }
    }

    /// Whether everyone who came at band `bi`'s muster and is still going with it has arrived.
    fn band_all_there(&self, ctx: &Ctx, bi: usize) -> bool {
        let b = &self.order.bands[bi];
        b.mustered.iter().all(|&m| {
            b.arrived.contains(&m)
                || self
                    .person(m)
                    .is_none_or(|p| !self.with_band(ctx, p) || p.act.step > STEP_THERE)
        })
    }

    /// Whether everyone who came at band `bi`'s muster is home or dead.
    fn band_all_home(&self, bi: usize) -> bool {
        let b = &self.order.bands[bi];
        b.mustered
            .iter()
            .all(|&m| b.home.contains(&m) || self.person(m).is_none())
    }

    /// Band `bi` has set off (ADR-0023 §3: shirking costs regard): each who went thinks less of
    /// each who said they would come and did not, and a little less of each asked who said no.
    fn shirked(&mut self, ctx: &Ctx, bi: usize) {
        let bp = &ctx.params.bands;
        let b = &self.order.bands[bi];
        let went = b.mustered.clone();
        let mut stayed: Vec<(PermanentId, f64)> = b
            .members
            .iter()
            .filter(|m| b.mustered.binary_search(m).is_err())
            .map(|&m| (m, bp.shirk_promised))
            .collect();
        stayed.extend(b.asked.iter().map(|&m| (m, bp.shirk_asked)));
        stayed.retain(|&(m, units)| units > 0.0 && self.person(m).is_some());
        for &g in &went {
            for &(m, units) in &stayed {
                self.note_tie(ctx, g, m, Act::StayedHome, units, 0.0);
            }
        }
    }

    /// Those about look at each band walking (ADR-0023 §8): anyone awake and grown, not of the
    /// band's settlement, within sight of one of it going hears that a band was seen, the claim
    /// made the first time; another look follows while any band is out.
    pub(super) fn band_look(&mut self, ctx: &mut Ctx) {
        let (now, day) = (ctx.now, ctx.now.day_index());
        let sight = ctx.params.crime.sight_m as f32;
        let adult = ctx.params.crime.guardian_age;
        let t = now.minutes() as f64;
        let out: Vec<usize> = (0..self.order.bands.len())
            .filter(|&i| self.order.bands[i].stage == BandStage::Out)
            .collect();
        if out.is_empty() {
            return;
        }
        for bi in out {
            let b = &self.order.bands[bi];
            let going: Vec<(f32, f32)> = b
                .mustered
                .iter()
                .filter(|m| !b.home.contains(m))
                .filter_map(|&m| self.person(m))
                .filter(|p| self.with_band(ctx, p))
                .map(|p| p.position_at(t))
                .collect();
            if going.is_empty() {
                continue;
            }
            let (id, settlement) = (b.id, b.settlement);
            let mut seers: Vec<PermanentId> = Vec::new();
            for (_, q) in self.people.iter() {
                if q.asleep || q.age_years(now) < adult || b.mustered.binary_search(&q.id).is_ok() {
                    continue;
                }
                if self.household(q.household).and_then(|x| x.settlement) == Some(settlement) {
                    continue;
                }
                let pos = q.position_at(t);
                if going
                    .iter()
                    .any(|g| (g.0 - pos.0).hypot(g.1 - pos.1) <= sight)
                {
                    seers.push(q.id);
                }
            }
            if seers.is_empty() {
                continue;
            }
            let claim = self
                .word
                .claims
                .iter()
                .find(|c| c.kind == ClaimKind::BandSeen && c.event == Some(id))
                .map(|c| c.id)
                .unwrap_or_else(|| {
                    self.word.make(Claim {
                        id: 0,
                        kind: ClaimKind::BandSeen,
                        settlement,
                        day,
                        subject: None,
                        grievance: None,
                        suspected: None,
                        fire: None,
                        event: Some(id),
                    })
                });
            for q in seers {
                self.word.hear(q, claim, day, None, Some(q));
            }
        }
        if self.order.bands.iter().any(|b| b.stage == BandStage::Out) {
            let look = i64::from(ctx.params.bands.look_minutes.max(1));
            ctx.schedule
                .push((now.plus_minutes(look), AgentEvent::BandLook));
        }
    }

    /// `who`, at the place a band went, leaves at once: the stay ends the next minute.
    fn leave_now(&mut self, ctx: &mut Ctx, who: PermanentId) {
        let Some(&h) = self.index.get(&who) else {
            return;
        };
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        if p.act.step != STEP_THERE {
            return;
        }
        p.act.step_ends = ctx.now.plus_minutes(1);
        p.act.version = p.act.version.wrapping_add(1);
        let (id, version, ends) = (p.id, p.act.version, p.act.step_ends);
        ctx.schedule_step(ends, id, version);
    }

    /// Whether `p` is on the band activity.
    fn with_band(&self, ctx: &Ctx, p: &Person) -> bool {
        ctx.catalog
            .activities
            .get(p.act.def as usize)
            .is_some_and(|a| a.behavior == Behavior::Band)
    }

    /// What each of `people` of household `household` holds into a fight: its spears first, then
    /// its axes, one each, in the order given (ADR-0023 §4: equipment is goods).
    fn holdings(&self, ctx: &Ctx, household: PermanentId, people: &[PermanentId]) -> Vec<Holding> {
        let goods = &ctx.catalog.goods;
        let count = |id: &str| {
            goods
                .iter()
                .position(|g| g.id == id)
                .and_then(|g| self.household(household).and_then(|x| x.stores.get(g)))
                .map_or(0usize, |&n| n.max(0.0).floor() as usize)
        };
        let (spears, axes) = (count("core:good/spear"), count("core:good/axe"));
        (0..people.len())
            .map(|k| {
                if k < spears {
                    Holding::Spear
                } else if k < spears + axes {
                    Holding::Axe
                } else {
                    Holding::Hands
                }
            })
            .collect()
    }

    /// Band `bi` has arrived (ADR-0023 §4): those of the household at home, and neighbours awake
    /// within sight who choose to, stand against it; the encounter runs; if the band holds the
    /// place it takes what it can carry, the household's seed left; every blow and death is
    /// recorded, and the taking is an incident like any other, known to those who saw it.
    fn clash(&mut self, ctx: &mut Ctx, bi: usize) {
        let (now, day, params, goods) =
            (ctx.now, ctx.now.day_index(), ctx.params, &ctx.catalog.goods);
        let (bp, cp, tp) = (&params.bands, &params.crime, &params.ties);
        let b = self.order.bands[bi].clone();
        let Some(theirs) = self.household(b.target) else {
            return;
        };
        let (home, members, their_settlement) =
            (theirs.home, theirs.members.clone(), theirs.settlement);
        let t = now.minutes() as f64;
        // The band: those arrived and not kept from work by a hurt.
        let mut fighters: Vec<Fighter> = Vec::new();
        let mut by_household: BTreeMap<PermanentId, Vec<PermanentId>> = BTreeMap::new();
        for &m in &b.arrived {
            if let Some(p) = self.person(m)
                && self.order.hurt_until(m).is_none_or(|d| d < day)
            {
                by_household.entry(p.household).or_default().push(m);
            }
        }
        for (&hh, people) in &by_household {
            for (k, held) in self.holdings(ctx, hh, people).into_iter().enumerate() {
                let m = people[k];
                let will = bp.will_band
                    + bp.w_grievance * self.keenness(ctx, m, Blamed::Household(b.target));
                fighters.push(Fighter {
                    person: m,
                    side: Side::Band,
                    holds: held,
                    will: will as f32,
                    regard_leader: self.ties.regard(m, b.organizer, day, tp).clamp(0.0, 1.0) as f32,
                    gave_way: None,
                    hurt_days: 0,
                    killed: false,
                });
            }
        }
        // Those against it: the household at home, awake or woken; neighbours awake in sight.
        let elder = self.elder_of(b.target, now, params);
        let mut defence: Vec<(PermanentId, PermanentId, f64)> = Vec::new();
        for &m in &members {
            if let Some(q) = self.person(m)
                && q.trip.is_none()
                && q.age_years(now) >= cp.guardian_age
                && ((q.pos.0 - home.0).powi(2) + (q.pos.1 - home.1).powi(2)).sqrt() <= AT_HOME_M
            {
                defence.push((m, q.household, bp.will_home));
            }
        }
        for (_, q) in self.people.iter() {
            if q.household == b.target || q.asleep || q.age_years(now) < cp.guardian_age {
                continue;
            }
            let pos = q.position_at(t);
            if ((pos.0 - b.place.0).powi(2) + (pos.1 - b.place.1).powi(2)).sqrt()
                > cp.sight_m as f32
            {
                continue;
            }
            if self.household(q.household).and_then(|x| x.settlement) != their_settlement
                || by_household.contains_key(&q.household)
            {
                continue;
            }
            let regard = elder.map_or(0.0, |e| self.ties.regard(q.id, e, day, tp).clamp(0.0, 1.0));
            let will = bp.will_neighbour * regard;
            if will > 0.0 {
                defence.push((q.id, q.household, will));
            }
        }
        defence.sort_by_key(|d| d.0);
        let mut by_defender: BTreeMap<PermanentId, Vec<(PermanentId, f64)>> = BTreeMap::new();
        for &(m, hh, w) in &defence {
            by_defender.entry(hh).or_default().push((m, w));
        }
        for (&hh, people) in &by_defender {
            let ids: Vec<PermanentId> = people.iter().map(|x| x.0).collect();
            for (k, held) in self.holdings(ctx, hh, &ids).into_iter().enumerate() {
                let (m, w) = people[k];
                fighters.push(Fighter {
                    person: m,
                    side: Side::Defence,
                    holds: held,
                    will: w as f32,
                    regard_leader: elder
                        .map_or(0.0, |e| self.ties.regard(m, e, day, tp).clamp(0.0, 1.0))
                        as f32,
                    gave_way: None,
                    hurt_days: 0,
                    killed: false,
                });
            }
        }
        let leads = [
            b.arrived.contains(&b.organizer).then_some(b.organizer),
            elder.filter(|e| defence.iter().any(|d| d.0 == *e)),
        ];
        let frontage = bp.fight.frontage_home;
        let seed = ctx.seed ^ PURPOSE_CLASH;
        let (blows, minutes, ended) = fight(
            &mut fighters,
            frontage,
            leads,
            now.minutes(),
            seed,
            &bp.fight,
        );
        // What the band carries off if it holds the place: a load each of those standing, the
        // densest food first, the household's seed left.
        let mut taken: Vec<(usize, f64)> = Vec::new();
        if ended.band_holds() {
            let standing: Vec<PermanentId> = fighters
                .iter()
                .filter(|f| f.side == Side::Band && f.gave_way.is_none() && !f.killed)
                .map(|f| f.person)
                .collect();
            for m in standing {
                let Some(to) = self.person(m).map(|p| p.household) else {
                    continue;
                };
                let stores = self
                    .household(b.target)
                    .map(|x| stores_now(x, now, params, goods))
                    .unwrap_or_default();
                let load =
                    crime::take_goods(&stores, goods, params.household.carry_kg, f64::INFINITY);
                let legs: Vec<Leg> = load
                    .iter()
                    .map(|&(good, amount)| Leg {
                        from: b.target,
                        to,
                        good,
                        amount,
                    })
                    .collect();
                if !legs.is_empty() && self.transfer(now, params, goods, &legs, Channel::Take) {
                    taken.extend(load);
                }
            }
        }
        let kcal: f64 = taken.iter().map(|&(g, kg)| kg * goods[g].kcal_per_kg).sum();
        let clash_id = self.order.clashes.last().map_or(1, |c| c.id + 1);
        let record = Clash {
            id: clash_id,
            at: now.minutes(),
            place: b.place,
            band: b.id,
            fighters: fighters.clone(),
            blows: blows.clone(),
            frontage,
            minutes,
            ended,
        };
        let words = self.clash_words(ctx, &b, &record, &taken);
        self.order.clashes.push(record);
        {
            let band = &mut self.order.bands[bi];
            band.clash = Some(clash_id);
            band.taken = taken.iter().map(|&(g, kg)| (g as u16, kg as f32)).collect();
            band.taken_kcal = kcal as f32;
        }
        // The taking, or the try, is an incident like any other (ADR-0015): those who stood
        // against the band saw it, and know whom of it they know.
        let organizer_hh = self.person(b.organizer).map(|p| p.household);
        let seen_by: Vec<PermanentId> = fighters
            .iter()
            .filter(|f| f.side == Side::Defence)
            .map(|f| f.person)
            .collect();
        if let Some(actor_household) = organizer_hh {
            let id = self.order.incidents.last().map_or(1, |i| i.id + 1);
            self.order.bands[bi].incident = Some(id);
            self.order.incidents.push(Incident {
                id,
                at: now,
                actor: b.organizer,
                actor_household,
                target: b.target,
                place: b.place,
                outcome: if kcal > 0.0 {
                    Outcome::Taken
                } else if ended.band_holds() {
                    Outcome::TurnedBack
                } else {
                    Outcome::Disturbed
                },
                goods: taken.iter().map(|&(g, kg)| (g as u16, kg as f32)).collect(),
                kcal: kcal as f32,
                seen_by: seen_by.clone(),
                noticed: true,
                food_days: f32::NAN,
                richer: f32::NAN,
                across: Some(b.settlement),
            });
            if kcal > 0.0 {
                self.order.amounts.push(KnownAmount {
                    household: b.target,
                    incident: id,
                    kcal: kcal as f32,
                    lost: true,
                    day,
                });
                self.order.amounts.push(KnownAmount {
                    household: actor_household,
                    incident: id,
                    kcal: kcal as f32,
                    lost: false,
                    day,
                });
            }
            let band_people: Vec<PermanentId> = fighters
                .iter()
                .filter(|f| f.side == Side::Band)
                .map(|f| f.person)
                .collect();
            for &w in &seen_by {
                let named = std::iter::once(b.organizer)
                    .chain(band_people.iter().copied())
                    .find(|&m| self.ties.tie(w, m, day, tp).is_some());
                self.come_to_believe(
                    ctx,
                    Belief {
                        holder: w,
                        incident: id,
                        taker: named,
                        source: Source::Saw,
                        from: None,
                        origin: Some(w),
                        day,
                    },
                );
            }
        }
        for s in &blows {
            self.note_tie(ctx, s.harm.to, s.harm.by, Act::Struck, 1.0, 0.0);
        }
        // Everyone there knows of the raid (ADR-0023 §8), and the household weighs where to live
        // at once (§10).
        if let Some(at) = their_settlement {
            let claim = self.word.make(Claim {
                id: 0,
                kind: ClaimKind::Raid,
                settlement: at,
                day,
                subject: Some(b.target),
                grievance: None,
                suspected: None,
                fire: None,
                event: Some(clash_id),
            });
            for f in fighters.iter().filter(|f| !f.killed) {
                self.word.hear(f.person, claim, day, None, Some(f.person));
            }
            self.review_due.insert(b.target);
        }
        // Those of the band who gave way go home at once, and all of it when it gave way or
        // parted (ADR-0023 §4).
        let all_go = matches!(ended, Ended::BandGaveWay | Ended::Parted);
        let leaving: Vec<PermanentId> = fighters
            .iter()
            .filter(|f| f.side == Side::Band && !f.killed && (all_go || f.gave_way.is_some()))
            .map(|f| f.person)
            .collect();
        for m in leaving {
            self.leave_now(ctx, m);
        }
        let mut people = vec![b.organizer];
        people.extend(elder);
        self.chronicle_push(
            now,
            ChronicleKind::Raid,
            people,
            their_settlement,
            Some(home),
            kcal,
            words,
        );
        let mut dead: Vec<PermanentId> = fighters
            .iter()
            .filter(|f| f.killed)
            .map(|f| f.person)
            .collect();
        dead.sort_unstable();
        dead.dedup();
        for p in dead {
            self.die(ctx, p, Cause::Violence);
        }
    }

    /// An encounter in plain words (06-10 §4.C-D): "Ada led 5 of Oakdale, 2 with spears, to the
    /// household of Bo at Thornmere to take food; Bo and Cal stood against them; Cal struck Dan,
    /// who died of it; those against them gave way; they took 60 kg of grain."
    fn clash_words(&self, ctx: &Ctx, b: &Band, c: &Clash, taken: &[(usize, f64)]) -> String {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let who = self.name_of(b.organizer);
        let from = self.settlement_name(ctx, b.settlement);
        let band: Vec<&Fighter> = c.fighters.iter().filter(|f| f.side == Side::Band).collect();
        let defence: Vec<&Fighter> = c
            .fighters
            .iter()
            .filter(|f| f.side == Side::Defence)
            .collect();
        let spears = band.iter().filter(|f| f.holds == Holding::Spear).count();
        let of = self.elder_of(b.target, now, params).map_or_else(
            || "a household".to_owned(),
            |e| format!("the household of {}", self.name_of(e)),
        );
        let at = self
            .household(b.target)
            .and_then(|x| x.settlement)
            .map_or_else(String::new, |s| {
                format!(" at {}", self.settlement_name(ctx, s))
            });
        let armed = if spears > 0 {
            format!(", {spears} with spears,")
        } else {
            String::new()
        };
        let mut s = format!(
            "{who} led {} of {from}{armed} to {of}{at} {}",
            band.len(),
            b.purpose.words()
        );
        let names = |fs: &[&Fighter]| -> String {
            let v: Vec<String> = fs.iter().map(|f| self.name_of(f.person)).collect();
            match v.len() {
                0 => String::new(),
                1 => v[0].clone(),
                n if n <= 4 => format!("{} and {}", v[..n - 1].join(", "), v[n - 1]),
                n => format!("{}, {} and {} others", v[0], v[1], n - 2),
            }
        };
        if defence.is_empty() {
            s.push_str("; nobody stood against them");
        } else {
            s.push_str(&format!("; {} stood against them", names(&defence)));
        }
        let mut hurt = 0;
        for st in &c.blows {
            if st.harm.killed {
                s.push_str(&format!(
                    "; {} struck {}, who died of it",
                    self.name_of(st.harm.by),
                    self.name_of(st.harm.to)
                ));
            } else {
                hurt += 1;
            }
        }
        if hurt > 0 {
            s.push_str(&format!(
                "; {hurt} {} kept people from work",
                if hurt == 1 { "blow" } else { "blows" }
            ));
        }
        s.push_str(match c.ended {
            Ended::NobodyStood => "",
            Ended::DefenceGaveWay => "; those against them gave way",
            Ended::BandGaveWay => "; the band gave way and went home",
            Ended::Parted => "; each side went its way",
        });
        let kg: f64 = taken.iter().map(|&(_, kg)| kg).sum();
        if kg > 0.0 {
            let what = taken
                .iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map_or("food", |&(g, _)| goods[g].name.as_str())
                .to_lowercase();
            s.push_str(&format!("; they took {kg:.0} kg of {what}"));
        } else if c.ended.band_holds() {
            s.push_str("; there was nothing to take");
        }
        s.push('.');
        s
    }
}
