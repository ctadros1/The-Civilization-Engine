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
use crate::faction::{
    Faction, FactionEvent, FactionEventKind, Member, Why, found_worth, shared, why_belong,
};
use crate::word::{Blamed, ClaimKind};

/// Purpose tags for a person's review day and their threshold for belonging.
pub const PURPOSE_FACTION_DAY: u64 = 0x6661_6374_6461_7973; // "factdays"
pub const PURPOSE_FACTION_THRESHOLD: u64 = 0x6661_6374_7468_7265; // "factthre"

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
    fn keenness(&self, ctx: &Ctx, person: PermanentId, party: Blamed) -> f64 {
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
}
