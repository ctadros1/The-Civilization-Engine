//! Factions at work (M4c slice AH, step one; ADR-0017 §2; research 04-10 §1.1–§1.5, §5.1–§5.4,
//! 09-12 §1.5, §2.2). Each adult reviews where they belong on a day of their own each month: a
//! member stays while belonging is worth enough to them and leaves when it falls well below their
//! threshold; one who belongs nowhere joins the faction they know of that is worth most to them, if
//! it is worth their threshold; failing that, one who feels a grievance against the gathering or
//! an office keenly, and has heard that others they trust hold one against it too, may found a
//! faction over it. A faction whose organizer is gone passes to its longest-standing member; one
//! with no members ends. A member's household gives a share of each threshing to its store, and a
//! household short of food may ask the store of a faction one of its members belongs to.

use super::*;
use crate::decide::GatheringFacts;
use crate::faction::{
    Faction, FactionEvent, FactionEventKind, Member, Petition, Why, attend_points, found_worth,
    shared, why_belong,
};
use crate::polity::{IssueKind, LawStatus, MoveOption, Outcome, PolicyKind};
use crate::word::{Blamed, ClaimKind, Grieved, Wrong};

/// Purpose tags for a person's review day and their threshold for belonging.
pub const PURPOSE_FACTION_DAY: u64 = 0x6661_6374_6461_7973; // "factdays"
pub const PURPOSE_FACTION_THRESHOLD: u64 = 0x6661_6374_7468_7265; // "factthre"
/// Purpose tag for whether someone is a free-rider at a petition (research 04-10 §1.4).
pub const PURPOSE_FREE_RIDER: u64 = 0x6672_6565_7269_6465; // "freeride"

impl Population {
    /// Midnight: memberships of those no longer here are let go, factions whose organizer is gone
    /// pass on or end, and those whose day it is review where they belong, in id order.
    pub(super) fn factions_day(&mut self, ctx: &mut Ctx) {
        self.keep_factions(ctx);
        let fp = &ctx.params.faction;
        let review = u64::from(fp.review_days.max(1));
        let (now, day) = (ctx.now, ctx.now.day_index());
        let today = day.rem_euclid(review as i64) as u64;
        let mut due: Vec<(PermanentId, PermanentId)> = self
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= ctx.params.family.independent_age)
            .filter(|(_, p)| {
                Rng64::from_key(&[ctx.seed, PURPOSE_FACTION_DAY, p.id.get()]).next_u64() % review
                    == today
            })
            .filter_map(|(_, p)| Some((p.id, self.household(p.household)?.settlement?)))
            .collect();
        due.sort_unstable();
        for (person, settlement) in due {
            if self.polity_of(settlement).is_some() {
                self.review_belonging(ctx, person, settlement);
            }
        }
    }

    /// Lets go of memberships of those dead or gone from the faction's settlement; passes a
    /// faction whose organizer no longer belongs to its longest-standing member; ends one with no
    /// members, giving what its store holds to its last organizer's household if it is still
    /// here.
    fn keep_factions(&mut self, ctx: &Ctx) {
        let gone: Vec<PermanentId> = self
            .factions
            .members
            .iter()
            .filter(|m| {
                let home = self
                    .person(m.person)
                    .and_then(|p| self.household(p.household))
                    .and_then(|x| x.settlement);
                let here = self.factions.get(m.faction).map(|f| f.settlement);
                home.is_none() || home != here
            })
            .map(|m| m.person)
            .collect();
        for person in gone {
            self.factions.leave(person);
            self.factions.left += 1;
        }
        let now = ctx.now;
        for fi in 0..self.factions.list.len() {
            if !self.factions.list[fi].is_live() {
                continue;
            }
            let id = self.factions.list[fi].id;
            let organizer = self.factions.list[fi].organizer;
            if self
                .factions
                .membership(organizer)
                .is_some_and(|m| m.faction == id)
            {
                continue;
            }
            let next = self
                .factions
                .members_of(id)
                .min_by_key(|m| (m.since, m.person))
                .map(|m| m.person);
            match next {
                Some(next) => {
                    let f = &mut self.factions.list[fi];
                    f.organizer = next;
                    f.history.push(FactionEvent {
                        at: now,
                        kind: FactionEventKind::Organizer,
                        who: Some(next),
                    });
                }
                None => self.end_faction(ctx, fi, organizer),
            }
        }
    }

    /// Faction `fi`, whose last organizer was `organizer`, has no members: it ends, and what its
    /// store holds goes to that organizer's household if it is still here.
    fn end_faction(&mut self, ctx: &Ctx, fi: usize, organizer: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let id = self.factions.list[fi].id;
        let sheltered = self.faction_sheltered(fi);
        self.factions.list[fi].settle_stores(now, goods, sheltered);
        let to = self
            .person(organizer)
            .map(|p| p.household)
            .filter(|h| self.household(*h).is_some());
        if let Some(to) = to {
            let legs: Vec<Leg> = self.factions.list[fi]
                .stores
                .iter()
                .enumerate()
                .filter(|(_, kg)| **kg > 1e-9)
                .map(|(good, &amount)| Leg {
                    from: id,
                    to,
                    good,
                    amount,
                })
                .collect();
            if !legs.is_empty() {
                self.transfer(now, params, goods, &legs, Channel::Gift);
            }
        }
        let f = &mut self.factions.list[fi];
        f.ended = Some(now);
        f.history.push(FactionEvent {
            at: now,
            kind: FactionEventKind::Ended,
            who: None,
        });
    }

    /// Whether faction `fi`'s store lies under a roof: its organizer's household's.
    pub(crate) fn faction_sheltered(&self, fi: usize) -> bool {
        self.factions
            .list
            .get(fi)
            .and_then(|f| self.person(f.organizer))
            .and_then(|p| self.household(p.household))
            .is_some_and(|x| x.sheltered)
    }

    /// Whether `party` is the body of polity `pi` or one of its offices.
    fn party_of(&self, pi: usize, party: Blamed) -> bool {
        let p = &self.polities[pi];
        match party {
            Blamed::Body(id) => id == p.id,
            Blamed::Office(law) => p.laws.iter().any(|l| l.id == law),
            Blamed::Household(_) | Blamed::Person(_) => false,
        }
    }

    /// How keenly `person` feels their keenest grievance against `party` today, 0 to 1.
    pub(super) fn keenness(&self, ctx: &Ctx, person: PermanentId, party: Blamed) -> f64 {
        let day = ctx.now.day_index();
        self.word
            .grievances_of(person)
            .filter(|g| g.blamed == party)
            .map(|g| g.activation_on(day, ctx.params.word.half_life(g.issue)))
            .fold(0.0, f64::max)
    }

    /// Why `person`, of threshold `threshold`, would belong to faction `id` today.
    fn why_for(&self, ctx: &Ctx, person: PermanentId, id: PermanentId, threshold: f64) -> Why {
        let (fp, tp, day) = (&ctx.params.faction, &ctx.params.ties, ctx.now.day_index());
        let Some(f) = self.factions.get(id) else {
            return Why::default();
        };
        let grievance = self.keenness(ctx, person, f.against);
        // Regard for its organizer is what draws others; an organizer stays for the grievance
        // and those who joined, so one nobody joins gives it up.
        let regard = if f.organizer == person {
            0.0
        } else {
            self.ties.regard(person, f.organizer, day, tp)
        };
        // Those they know, by how well they know them, and how many of them belong.
        let (mut all, mut belong, mut known) = (0.0, 0.0, 0usize);
        for t in self.ties.of(person) {
            let w = t.known_at(day, tp);
            if w <= 0.0 {
                continue;
            }
            all += w;
            if self
                .factions
                .membership(t.to)
                .is_some_and(|m| m.faction == id)
            {
                belong += w;
                known += 1;
            }
        }
        let share = if all > 0.0 { belong / all } else { 0.0 };
        why_belong(grievance, regard, share, known, threshold, fp)
    }

    /// `person` of `settlement` reviews where they belong.
    fn review_belonging(&mut self, ctx: &mut Ctx, person: PermanentId, settlement: PermanentId) {
        let (fp, tp) = (&ctx.params.faction, &ctx.params.ties);
        let (now, day) = (ctx.now, ctx.now.day_index());
        let u = Rng64::from_key(&[ctx.seed, PURPOSE_FACTION_THRESHOLD, person.get()]).next_f64();
        let threshold = fp.threshold_at(u);
        // A member stays while belonging is worth enough to them (04-10 §5.4: leaving has
        // conditions of its own).
        if let Some(m) = self.factions.membership(person).copied() {
            let why = self.why_for(ctx, person, m.faction, threshold);
            if why.worth() < threshold - fp.leave_margin {
                self.factions.leave(person);
                self.factions.left += 1;
            } else {
                self.factions.join(Member { why, ..m });
                // An organizer weighs calling a petition (M4c slice AH, step two).
                if self
                    .factions
                    .get(m.faction)
                    .is_some_and(|f| f.organizer == person && f.is_live())
                {
                    self.consider_petition(ctx, person, m.faction, threshold);
                }
            }
            return;
        }
        // The factions here they know of: whose organizer or a member of which they know.
        let knows = |q: PermanentId| self.ties.known(person, q, day, tp) > 0.0;
        let known: Vec<PermanentId> = self
            .factions
            .list
            .iter()
            .filter(|f| f.is_live() && f.settlement == settlement)
            .filter(|f| {
                knows(f.organizer) || self.factions.members_of(f.id).any(|m| knows(m.person))
            })
            .map(|f| f.id)
            .collect();
        let best = known
            .iter()
            .map(|&id| (self.why_for(ctx, person, id, threshold), id))
            .max_by(|a, b| a.0.worth().total_cmp(&b.0.worth()).then(b.1.cmp(&a.1)));
        if let Some((why, faction)) = best
            && why.worth() > threshold
        {
            self.factions.join(Member {
                person,
                faction,
                since: day,
                why,
            });
            self.factions.joined += 1;
            return;
        }
        // Founding one: over the party they hold their keenest grievance against, when they know
        // of no faction against it here and have heard that others they trust hold one too.
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        let keenest = self
            .word
            .grievances_of(person)
            .filter(|g| self.party_of(pi, g.blamed))
            .map(|g| {
                let k = g.activation_on(day, ctx.params.word.half_life(g.issue));
                (k, g.blamed)
            })
            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
        let Some((keen, against)) = keenest else {
            return;
        };
        if keen < fp.found_floor
            || known
                .iter()
                .any(|&id| self.factions.get(id).is_some_and(|f| f.against == against))
        {
            return;
        }
        // One whose faction ended lately expects no better of another (04-10 §3).
        let tried = self.factions.list.iter().any(|f| {
            f.founder == person
                && f.ended
                    .is_some_and(|t| day - t.day_index() < i64::from(fp.retry_days))
        });
        if tried {
            return;
        }
        // Each other one they have heard holds a grievance against it, counted once however
        // often it was told (04-06 §1.5), if they trust them.
        let mut sharers: Vec<PermanentId> = self
            .word
            .heard_by(person)
            .iter()
            .filter_map(|h| self.word.claim(h.claim))
            .filter(|c| c.kind == ClaimKind::Grievance && c.settlement == settlement)
            .filter(|c| c.grievance.is_some_and(|(b, _)| b == against))
            .filter_map(|c| c.subject)
            .filter(|&s| s != person && self.ties.regard(person, s, day, tp) >= fp.trust_floor)
            .collect();
        sharers.sort_unstable();
        sharers.dedup();
        if sharers.is_empty() {
            return;
        }
        let worth = found_worth(keen, shared(sharers.len(), fp.shared_full), fp);
        if worth <= threshold {
            return;
        }
        let id = ctx.ids.allocate();
        self.factions
            .list
            .push(Faction::found(id, settlement, against, person, now));
        let why = self.why_for(ctx, person, id, threshold);
        self.factions.join(Member {
            person,
            faction: id,
            since: day,
            why,
        });
        self.factions.joined += 1;
    }

    /// `who` of household `household` threshed and kept `grain_kg` of grain (good `grain`): if
    /// they belong to a faction whose store holds less than its reserve, their household gives its
    /// share to the store, unless it cannot spare it.
    pub(crate) fn pay_dues(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        household: PermanentId,
        grain_kg: f64,
        grain: usize,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(m) = self.factions.membership(who).copied() else {
            return;
        };
        let Some(fi) = self
            .factions
            .list
            .iter()
            .position(|f| f.id == m.faction && f.is_live())
        else {
            return;
        };
        let owed = grain_kg * params.faction.dues_share;
        if owed <= 1e-9 {
            return;
        }
        // Its reserve: `reserve_days` of its members' households' food.
        let mut households: Vec<PermanentId> = self
            .factions
            .members_of(m.faction)
            .filter_map(|x| self.person(x.person).map(|p| p.household))
            .collect();
        households.sort_unstable();
        households.dedup();
        let people: usize = households
            .iter()
            .filter_map(|&h| self.household(h))
            .map(|x| x.members.len())
            .sum();
        let reserve =
            params.faction.reserve_days * people as f64 * params.household.daily_kcal_per_person;
        if stock_kcal(&self.factions.list[fi].stores, goods) >= reserve {
            return;
        }
        let Some(outlook) = self.outlook(ctx, household) else {
            return;
        };
        let kcal = owed * goods.get(grain).map_or(0.0, |g| g.kcal_per_kg);
        if crate::polity::cannot_pay(outlook.held, kcal, outlook.year_need, &params.polity) {
            return;
        }
        let held = self
            .household(household)
            .and_then(|x| x.stores.get(grain))
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        let paid = owed.min(held);
        if paid <= 1e-9 {
            return;
        }
        let sheltered = self.faction_sheltered(fi);
        self.factions.list[fi].settle_stores(now, goods, sheltered);
        let leg = Leg {
            from: household,
            to: m.faction,
            good: grain,
            amount: paid,
        };
        self.transfer(now, params, goods, &[leg], Channel::Dues);
    }

    /// The store of a faction a member of `hh` belongs to, as somewhere to ask for food: the one
    /// that could give the most, up to `want` kcal and the days of food one ask may bring, at its
    /// organizer's home, walked to as `reach` says.
    pub(crate) fn aid_option(
        &self,
        ctx: &Ctx,
        hh: &Household,
        kcal_day: f64,
        want: f64,
        reach: &TravelField,
    ) -> Option<GiverOption> {
        let goods = &ctx.catalog.goods;
        let mut factions: Vec<PermanentId> = hh
            .members
            .iter()
            .filter_map(|&m| self.factions.membership(m).map(|x| x.faction))
            .collect();
        factions.sort_unstable();
        factions.dedup();
        factions
            .into_iter()
            .filter_map(|id| {
                let fi = self
                    .factions
                    .list
                    .iter()
                    .position(|f| f.id == id && f.is_live())?;
                let f = &self.factions.list[fi];
                let at = self
                    .person(f.organizer)
                    .and_then(|p| self.household(p.household))?
                    .home;
                let walk_min = f64::from(reach.seconds_to(cell_of(ctx.map, at))?) / 60.0;
                let kcal = stock_kcal(&f.stores, goods)
                    .min(want)
                    .min(ctx.params.faction.aid_days * kcal_day);
                (kcal > 0.0).then_some(GiverOption {
                    household: id,
                    walk_min,
                    at,
                    kcal,
                })
            })
            .max_by(|a, b| {
                a.kcal
                    .total_cmp(&b.kcal)
                    .then(b.household.cmp(&a.household))
            })
    }

    /// Household `to`, one of whose members belongs to faction `fi`, asked its store for food: it
    /// gives up to the days of food one ask may bring, what brings the household to the days of
    /// food it tries to keep and what one person carries, the most perishable first.
    pub(crate) fn give_aid(&mut self, ctx: &Ctx, fi: usize, to: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let id = self.factions.list[fi].id;
        let Some(t) = self.household(to) else {
            return;
        };
        if !t
            .members
            .iter()
            .any(|&m| self.factions.membership(m).is_some_and(|x| x.faction == id))
        {
            return;
        }
        let need = t.members.len() as f64 * params.household.daily_kcal_per_person;
        let held = stock_kcal(&stores_now(t, now, params, goods), goods);
        let mut want = (params.faction.aid_days * need)
            .min(params.household.food_target_days * need - held)
            .max(0.0);
        if want <= 0.0 {
            return;
        }
        let sheltered = self.faction_sheltered(fi);
        self.factions.list[fi].settle_stores(now, goods, sheltered);
        let stores = self.factions.list[fi].stores.clone();
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
                from: id,
                to,
                good: i,
                amount: kg,
            });
            carry -= kg;
            want -= kg * goods[i].kcal_per_kg;
        }
        if !legs.is_empty() {
            self.transfer(now, params, goods, &legs, Channel::Gift);
        }
    }

    /// What faction `faction` of polity `pi`'s settlement would petition the gathering for, if
    /// anything: its demand answers the party its members blame (research 04-10 §1.1; 09-05 §1.1:
    /// a proposal is one actor's response to an issue). Against the gathering, of a common store
    /// at another of its shares, or at none, in place of the one in force, the one its members'
    /// households forecast most for, if they forecast it to bring them anything. Against an
    /// office, another holder in place of the one in it: whom its organizer would name, as anyone
    /// proposing a keeper does (themselves if their roof would do, or whoever under one they
    /// regard most; for the watch, any adult), never the holder blamed. With it, what each
    /// household of the settlement forecasts it would bring them, gain units. With `asked`, the
    /// demand as that petition called it, whatever it is forecast to bring.
    fn petition_demand(
        &self,
        ctx: &Ctx,
        pi: usize,
        faction: PermanentId,
        asked: Option<&Petition>,
    ) -> Option<(MoveOption, Vec<(PermanentId, f32)>)> {
        let (now, params) = (ctx.now, ctx.params);
        let policies = &ctx.catalog.policies;
        let polity = &self.polities[pi];
        let f = self.factions.get(faction)?;
        let law = match f.against {
            Blamed::Body(id) if id == polity.id => polity
                .laws
                .iter()
                .find(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::CommonStore)?,
            Blamed::Office(office) => polity.laws.iter().find(|l| {
                l.id == office
                    && l.status == LawStatus::InForce
                    && matches!(l.kind, PolicyKind::KeepStore | PolicyKind::KeepWatch)
                    && l.holder.is_some()
            })?,
            _ => return None,
        };
        let def = policies.get(usize::from(law.policy))?;
        let settlement = polity.settlement;
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let adults = self.members_of(settlement, now, params);
        let fc = self.forecasts(ctx, pi, &households, &adults);
        let mut theirs: Vec<PermanentId> = self
            .factions
            .members_of(faction)
            .filter_map(|m| self.person(m.person).map(|p| p.household))
            .collect();
        theirs.sort_unstable();
        theirs.dedup();
        if theirs.is_empty() {
            return None;
        }
        let blank = MoveOption {
            policy: law.policy,
            levy_share: f64::from(law.levy_share),
            issue: IssueKind::Petition,
            nominee: law.holder,
            sanction: law.sanction,
            hours: law.hours,
            body: None,
            ends: Some(law.id),
            own_gain: 0.0,
            followers_gain: 0.0,
            support: 0.5,
        };
        let held = |h: PermanentId| fc.gain(&blank, h, policies, params);
        let w = params.polity.w_gain;
        // Another holder for the office (M4c slice AH).
        if law.kind != PolicyKind::CommonStore {
            let incumbent = law.holder?;
            let nominee = match asked {
                Some(p) => p.nominee?,
                None => {
                    let (day, tp) = (now.day_index(), &params.ties);
                    let organizer = f.organizer;
                    adults
                        .iter()
                        .filter(|&&(c, h)| {
                            c != incumbent
                                && (law.kind == PolicyKind::KeepWatch
                                    || self.household(h).is_some_and(|x| x.sheltered))
                        })
                        .map(|&(c, _)| {
                            let r = if c == organizer {
                                1.0
                            } else {
                                self.ties.regard(organizer, c, day, tp).clamp(0.0, 1.0)
                            };
                            (r, c)
                        })
                        .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))?
                        .1
                }
            };
            if nominee == incumbent || !adults.iter().any(|a| a.0 == nominee) {
                return None;
            }
            let m = MoveOption {
                nominee: Some(nominee),
                ..blank
            };
            let stakes = households
                .iter()
                .map(|&h| {
                    let gain = fc.gain(&m, h, policies, params) - held(h);
                    (h, (w * gain) as f32)
                })
                .collect();
            return Some((m, stakes));
        }
        let gain = |share: f64, h: PermanentId| {
            let m = MoveOption {
                levy_share: share,
                ..blank
            };
            let new = if share > 0.0 {
                fc.gain(&m, h, policies, params)
            } else {
                0.0
            };
            new - held(h)
        };
        let mut shares: Vec<f64> = match asked {
            Some(p) => vec![f64::from(p.levy_share)],
            None => std::iter::once(0.0)
                .chain(def.levy_shares.iter().copied())
                .filter(|s| (s - f64::from(law.levy_share)).abs() > 1e-6)
                .collect(),
        };
        shares.sort_by(f64::total_cmp);
        shares.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let mean =
            |share: f64| theirs.iter().map(|&h| gain(share, h)).sum::<f64>() / theirs.len() as f64;
        let (share, best) = shares
            .iter()
            .map(|&s| (s, mean(s)))
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.total_cmp(&a.0)))?;
        if asked.is_none() && best <= 1e-9 {
            return None;
        }
        let stakes: Vec<(PermanentId, f32)> = households
            .iter()
            .map(|&h| (h, (w * gain(share, h)) as f32))
            .collect();
        Some((
            MoveOption {
                levy_share: share,
                nominee: None,
                ..blank
            },
            stakes,
        ))
    }

    /// `organizer` of faction `faction`, of threshold `threshold`, weighs calling a petition
    /// (research 09-04 §5.5: petitioning beside complying, evading and leaving; 09-05 §1.2): when
    /// it has enough members, none of its petitions waits, its last is not too recent, and there
    /// is something to ask, it is worth their members' grievance, how many of those they know
    /// belong, less what calling costs.
    fn consider_petition(
        &mut self,
        ctx: &mut Ctx,
        organizer: PermanentId,
        faction: PermanentId,
        threshold: f64,
    ) {
        let fp = &ctx.params.faction;
        let (now, day) = (ctx.now, ctx.now.day_index());
        let Some(f) = self.factions.get(faction) else {
            return;
        };
        let (settlement, against) = (f.settlement, f.against);
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        let members: Vec<PermanentId> = self
            .factions
            .members_of(faction)
            .map(|m| m.person)
            .collect();
        if (members.len() as u32) < fp.petition_members.max(1) {
            return;
        }
        let waiting = self
            .factions
            .petitions
            .iter()
            .any(|p| p.faction == faction && !p.answered);
        let lately = self.factions.petitions.iter().any(|p| {
            p.faction == faction && day - p.called.day_index() < i64::from(fp.petition_days)
        });
        if waiting || lately {
            return;
        }
        let Some((demand, _)) = self.petition_demand(ctx, pi, faction, None) else {
            return;
        };
        // What it is worth to them: their members' grievance against its party and how many of
        // those they know belong.
        let keen = members
            .iter()
            .map(|&m| self.keenness(ctx, m, against))
            .sum::<f64>()
            / members.len() as f64;
        let why = self.why_for(ctx, organizer, faction, threshold);
        let worth = fp.w_grievance * keen + f64::from(why.belong) - fp.petition_cost;
        if worth <= threshold {
            return;
        }
        let id = ctx.ids.allocate();
        let sits = day + i64::from(ctx.params.polity.notice_days.max(1));
        self.factions.petitions.push(Petition {
            id,
            faction,
            settlement,
            organizer,
            called: now,
            day: sits,
            policy: demand.policy,
            levy_share: demand.levy_share as f32,
            nominee: demand.nominee,
            ends: demand.ends.unwrap_or(id),
            came: Vec::new(),
            law: None,
            answered: false,
        });
        // The organizer tells its members, who connect it (04-10 §1.5); others hear by word.
        self.petition_word(settlement, sits, id, organizer, &members, day);
    }

    /// The petition sitting at the hearth of `hh`'s settlement now that `person` heard of and may
    /// join (M4c slice AH): what joining it is worth to them and the minutes it still sits.
    pub(crate) fn petition_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        age: f64,
        hh: &Household,
        minute: i64,
        evening_start: i64,
    ) -> Option<GatheringFacts> {
        let params = ctx.params;
        if age < params.family.independent_age {
            return None;
        }
        let settlement = hh.settlement?;
        let day = ctx.now.day_index();
        let p = self
            .factions
            .petitions
            .iter()
            .find(|p| p.settlement == settlement && p.day == day && !p.answered)?;
        let end = evening_start + i64::from(params.polity.gathering_minutes);
        if minute < evening_start
            || minute + 15 > end
            || p.came.binary_search(&person).is_ok()
            || !self.heard_of_petition(person, settlement, day)
        {
            return None;
        }
        let f = self.factions.get(p.faction)?;
        let member = self
            .factions
            .membership(person)
            .is_some_and(|m| m.faction == p.faction);
        let tp = &params.ties;
        let regard = self.ties.regard(person, f.organizer, day, tp);
        // Those they know whom they expect to come: its members.
        let (mut all, mut expect) = (0.0, 0.0);
        for t in self.ties.of(person) {
            let w = t.known_at(day, tp);
            if w <= 0.0 {
                continue;
            }
            all += w;
            if self
                .factions
                .membership(t.to)
                .is_some_and(|m| m.faction == p.faction)
            {
                expect += w;
            }
        }
        let expect = if all > 0.0 { expect / all } else { 0.0 };
        let fp = &params.faction;
        let rider = Rng64::from_key(&[ctx.seed, PURPOSE_FREE_RIDER, person.get()]).next_f64()
            < fp.free_ride_share;
        let grievance = self.keenness(ctx, person, f.against);
        Some(GatheringFacts {
            points: attend_points(grievance, member, regard, expect, rider, fp),
            minutes: (end - minute) as f64,
        })
    }

    /// `person` came to the petition sitting at their settlement's hearth today.
    pub(crate) fn join_petition(&mut self, ctx: &Ctx, person: PermanentId) {
        let Some(settlement) = self
            .person(person)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
        else {
            return;
        };
        let day = ctx.now.day_index();
        if let Some(p) = self
            .factions
            .petitions
            .iter_mut()
            .find(|p| p.settlement == settlement && p.day == day && !p.answered)
            && let Err(at) = p.came.binary_search(&person)
        {
            p.came.insert(at, person);
        }
    }

    /// Midnight: a petition that has sat goes before the gathering as a proposal its organizer
    /// sponsors, once no gathering waits (one nobody came to ends there); and when the gathering
    /// has decided one, those who came hold its turning down against it.
    pub(super) fn petitions_day(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        for i in 0..self.factions.petitions.len() {
            let p = &self.factions.petitions[i];
            if p.answered || p.day >= day {
                continue;
            }
            let Some(pi) = self.polity_of(p.settlement) else {
                self.factions.petitions[i].answered = true;
                continue;
            };
            match p.law {
                None => {
                    if p.came.is_empty() {
                        self.factions.petitions[i].answered = true;
                        continue;
                    }
                    let polity = &self.polities[pi];
                    if polity.gathering.is_some() || polity.agenda().is_some() {
                        continue;
                    }
                    let asked = p.clone();
                    let (faction, organizer, ends) = (p.faction, p.organizer, p.ends);
                    // The demand as called, its stakes as they stand now; the law it would
                    // replace must still be the one in force, and its organizer here.
                    let demand = self
                        .petition_demand(ctx, pi, faction, Some(&asked))
                        .filter(|d| d.0.ends == Some(ends));
                    let (Some((demand, stakes)), Some(_)) = (demand, self.person(organizer)) else {
                        self.factions.petitions[i].answered = true;
                        continue;
                    };
                    let before = self.polities[pi].laws.len();
                    self.propose(ctx, pi, organizer, demand, stakes, None);
                    if let Some(law) = self.polities[pi].laws.get(before) {
                        let id = law.id;
                        self.factions.petitions[i].law = Some(id);
                        // Those who came know what they asked for.
                        let came = self.factions.petitions[i].came.clone();
                        if let Some(l) = self.polities[pi].law_mut(id) {
                            for q in came {
                                l.learn(q, day);
                            }
                        }
                    }
                }
                Some(law) => {
                    let Some(outcome) = self.polities[pi]
                        .laws
                        .iter()
                        .find(|l| l.id == law)
                        .and_then(|l| l.outcome)
                    else {
                        continue;
                    };
                    self.factions.petitions[i].answered = true;
                    if outcome == Outcome::Passed {
                        continue;
                    }
                    // A petition turned down, or too thinly heard to decide, is a new wrong to
                    // those who came (04-10 §3: failed petitioning is a precursor).
                    let polity = self.polities[pi].id;
                    let came = self.factions.petitions[i].came.clone();
                    let harm = ctx.params.faction.refused_days;
                    for q in came {
                        if self.person(q).is_some() {
                            self.grieve(
                                ctx,
                                q,
                                Grieved::Collective,
                                Blamed::Body(polity),
                                law,
                                harm,
                                Wrong::Refused,
                            );
                        }
                    }
                }
            }
        }
    }
}
