//! Moving between settlements (M5a slice AN, ADR-0018 §5). A household reviews where it lives once
//! a year on a day of its own, and when an event prompts it (it formed): it weighs joining each
//! settlement it knows against staying, in points as its other choices are, and moves only when
//! the same place has won the content's number of reviews running (research 10-01 §2.3: a
//! sustained advantage, so households do not swing back and forth). A household out of food that
//! gives up (`departures`) goes to a known settlement that draws it, or else beyond the map, as
//! before. Moving is a household's: its people go together, take what they hold, and make camp
//! beside the other settlement's hearth as a sent family does; the ground, home and workshop they
//! leave stand as a household's that left (ADR-0007 §2 decides who works the ground next).

use super::*;
use crate::ResidenceWhy;
use crate::places::{Leaning, PlaceHow};
use civ_core::time::DAYS_PER_YEAR;

/// Purpose tag for the day of the year a household reviews where it lives.
pub const PURPOSE_REVIEW: u64 = 0x7265_7669_6577_3031; // "review01"

/// What a household's members find at a settlement: close kin living there (outside the
/// household) and the familiarity and warmth of those they know there, summed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Draw {
    kin: usize,
    known: f64,
}

impl Population {
    /// Households whose day of the year it is, or whom an event prompted, review where they live
    /// (ADR-0018 §5).
    pub(super) fn residence_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land.settlements.len() < 2 {
            self.review_due.clear();
            return;
        }
        let hh = &self.hh_index;
        self.leanings.retain(|h, _| hh.contains_key(h));
        let mut ids: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement.is_some() && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .filter(|&h| {
                self.review_due.contains(&h)
                    || Rng64::from_key(&[ctx.seed, PURPOSE_REVIEW, h.get()]).next_u64()
                        % DAYS_PER_YEAR as u64
                        == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        ids.sort_unstable();
        self.review_due.clear();
        if ids.is_empty() {
            return;
        }
        self.refresh_kin();
        for h in ids {
            self.review_residence(ctx, h, day);
        }
    }

    /// Household `household` weighs moving to each settlement it knows against staying; a place
    /// that wins the content's number of reviews running is where it goes.
    fn review_residence(&mut self, ctx: &mut Ctx, household: PermanentId, day: i64) {
        let best = self
            .move_worth(ctx, household)
            .into_iter()
            .filter(|&(_, v)| v > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)));
        let Some((to, _)) = best else {
            self.leanings.remove(&household);
            return;
        };
        let reviews = match self.leanings.get(&household) {
            Some(l) if l.settlement == to => l.reviews + 1,
            _ => 1,
        };
        if reviews >= ctx.params.moving.reviews.max(1) {
            self.leanings.remove(&household);
            self.relocate(ctx, household, to);
        } else {
            self.leanings.insert(
                household,
                Leaning {
                    settlement: to,
                    reviews,
                    day,
                },
            );
        }
    }

    /// What moving `household` to each settlement it knows would be worth against staying,
    /// points (ADR-0018 §5): kin and those known there against here, the food a member saw there
    /// against what they see at home, the grievances its members hold, against the harvest its
    /// fields here should bring, the work of a new home and fields, and the walk there once.
    fn move_worth(&self, ctx: &Ctx, household: PermanentId) -> Vec<(PermanentId, f64)> {
        let Some(x) = self.household(household) else {
            return Vec::new();
        };
        let Some(home) = x.settlement else {
            return Vec::new();
        };
        let mp = &ctx.params.moving;
        let draws = self.draws(ctx, household);
        let here = draws
            .iter()
            .find(|d| d.0 == home)
            .map_or(Draw::default(), |d| d.1);
        let fed_here = self.fed_share(ctx, home);
        let grievance = self.grievance_of(ctx, household);
        let stake = self
            .outlook(ctx, household)
            .map_or(0.0, |o| (o.harvest / o.year_need.max(1.0)).clamp(0.0, 1.0));
        let sat = |s: f64| s / (1.0 + s);
        let mut out = Vec::new();
        for k in self.known_places.of(household) {
            let Some(s) = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == k.settlement && s.abandoned.is_none())
            else {
                continue;
            };
            let there = draws
                .iter()
                .find(|d| d.0 == s.id)
                .map_or(Draw::default(), |d| d.1);
            let fed = k.food.map_or(0.0, |f| f64::from(f) - fed_here);
            let walk_h = self.walk_hours(ctx, home, s.id);
            let worth = mp.w_kin * (there.kin as f64 - here.kin as f64)
                + mp.w_ties * (sat(there.known) - sat(here.known))
                + mp.w_fed * fed
                + mp.w_grievance * grievance
                - mp.w_stake * stake
                - mp.cost
                - ctx.params.decision.w_walk_hour * walk_h;
            out.push((s.id, worth));
        }
        out
    }

    /// Where a household out of food that gives up goes (ADR-0018 §3, §5): the known settlement
    /// that draws it most, by kin and those known there and the food a member saw there against
    /// home, if any draws it at all; else beyond the map, as before.
    pub(super) fn refuge(&self, ctx: &Ctx, household: PermanentId) -> Option<PermanentId> {
        let home = self.household(household)?.settlement?;
        if self.known_places.of(household).is_empty() {
            return None;
        }
        let mp = &ctx.params.moving;
        let draws = self.draws(ctx, household);
        let fed_here = self.fed_share(ctx, home);
        let sat = |s: f64| s / (1.0 + s);
        self.known_places
            .of(household)
            .iter()
            .filter(|k| {
                ctx.land
                    .settlements
                    .iter()
                    .any(|s| s.id == k.settlement && s.abandoned.is_none())
            })
            .map(|k| {
                let there = draws
                    .iter()
                    .find(|d| d.0 == k.settlement)
                    .map_or(Draw::default(), |d| d.1);
                let fed = k.food.map_or(0.0, |f| f64::from(f) - fed_here);
                let pull =
                    mp.w_kin * there.kin as f64 + mp.w_ties * sat(there.known) + mp.w_fed * fed;
                (k.settlement, pull)
            })
            .filter(|&(_, pull)| pull > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|(s, _)| s)
    }

    /// What `household`'s members find at each settlement: close kin living there outside the
    /// household, and those they know there.
    fn draws(&self, ctx: &Ctx, household: PermanentId) -> Vec<(PermanentId, Draw)> {
        let Some(x) = self.household(household) else {
            return Vec::new();
        };
        let day = ctx.now.day_index();
        let lives_in = |q: PermanentId| {
            self.person(q)
                .filter(|p| p.household != household)
                .and_then(|p| self.household(p.household))
                .and_then(|h| h.settlement)
        };
        let mut out: Vec<(PermanentId, Draw)> = Vec::new();
        let mut add =
            |s: PermanentId, kin: usize, known: f64| match out.iter_mut().find(|e| e.0 == s) {
                Some(e) => {
                    e.1.kin += kin;
                    e.1.known += known;
                }
                None => out.push((s, Draw { kin, known })),
            };
        for &m in &x.members {
            for q in self.close_kin(m) {
                if let Some(s) = lives_in(q) {
                    add(s, 1, 0.0);
                }
            }
            for t in self.ties.of(m) {
                if let Some(s) = lives_in(t.to) {
                    add(s, 0, t.known_at(day, &ctx.params.ties));
                }
            }
        }
        out
    }

    /// The share of those living in `settlement` who are not going hungry (drawing on their
    /// body's reserve), as anyone living there sees.
    fn fed_share(&self, ctx: &Ctx, settlement: PermanentId) -> f64 {
        let (mut all, mut fed) = (0u32, 0u32);
        for (_, x) in self.households.iter() {
            if x.settlement != Some(settlement) {
                continue;
            }
            for p in x.members.iter().filter_map(|&m| self.person(m)) {
                all += 1;
                if !may_eat_reserve(p, ctx.now, ctx.params) {
                    fed += 1;
                }
            }
        }
        if all == 0 {
            1.0
        } else {
            f64::from(fed) / f64::from(all)
        }
    }

    /// The most keenly felt grievance a member of `household` holds now, 0–1 (ADR-0016 §2).
    fn grievance_of(&self, ctx: &Ctx, household: PermanentId) -> f64 {
        let Some(x) = self.household(household) else {
            return 0.0;
        };
        let (wp, day) = (&ctx.params.word, ctx.now.day_index());
        self.word
            .grievances
            .iter()
            .filter(|g| x.members.contains(&g.holder))
            .map(|g| g.activation_on(day, wp.half_life(g.issue)))
            .fold(0.0, f64::max)
            .clamp(0.0, 1.0)
    }

    /// Hours' walk from `from`'s hearth to `to`'s, at the pace people walk off trails.
    fn walk_hours(&self, ctx: &Ctx, from: PermanentId, to: PermanentId) -> f64 {
        let at = |s: PermanentId| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        };
        let (Some(a), Some(b)) = (at(from), at(to)) else {
            return 0.0;
        };
        let metres = f64::from((a.0 - b.0).hypot(a.1 - b.1));
        metres / ctx.nav.params().top_speed_ms().max(0.1) / 3600.0
    }

    /// Household `from` moves to settlement `to` (ADR-0018 §5): its people form a household there
    /// with what it holds, camped beside the hearth as a sent family, and the household they leave
    /// is no more: its ground, home and workshop stand as a household's that left.
    pub(super) fn relocate(&mut self, ctx: &mut Ctx, from: PermanentId, to: PermanentId) {
        let now = ctx.now;
        let Some(left) = self.household(from).and_then(|x| x.settlement) else {
            return;
        };
        let Some(hearth) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == to)
            .map(|s| s.hearth_m)
        else {
            return;
        };
        if left == to {
            return;
        }
        self.settle_household(ctx, from);
        let Some(old) = self.household(from).cloned() else {
            return;
        };
        let id = ctx.ids.allocate();
        let home = self.new_home_site(ctx, hearth, Some(hearth), id);
        self.insert_household(Household {
            id,
            members: Vec::new(),
            home,
            settlement: Some(to),
            stores: old.stores.clone(),
            stores_at: now,
            water_l: 0.0,
            water_at: now,
            known: Vec::new(),
            sheltered: false,
            keeping: crate::person::Keeping::default(),
            flows: Flows::default(),
            offers: Vec::new(),
            taste: old.taste,
            admired: old.admired,
            midden: crate::person::Midden::begun(now),
        });
        // Its people and what they know of other places go with it; where they lived is now a
        // place they know.
        for &m in &old.members {
            if let Some(p) = self.person_mut(m) {
                p.household = id;
            }
            self.note_residence(m, Some(to), now, ResidenceWhy::Moved);
        }
        if let Some(x) = self.household_mut(id) {
            x.members = old.members.clone();
        }
        self.sort_members(id);
        self.known_places.bring(from, id, Some(to));
        self.known_places.forget(from);
        let day = now.day_index();
        self.known_places
            .learn(id, left, day, PlaceHow::Lived, None);
        self.leanings.remove(&from);
        // The household they leave: its workshops close and its ground and home stand empty.
        if let Some(hd) = self.hh_index.remove(&from)
            && let Some(gone) = self.households.remove(hd)
        {
            self.homes_moved += 1;
            self.flows_gone.absorb(&gone.flows);
        }
        self.hand_over_land(ctx, from, None, false);
        self.review_land(ctx, left);
        self.review_land(ctx, to);
        let people = old.members.len() as u32;
        self.contacts
            .moved(day.div_euclid(DAYS_PER_YEAR), left, to, people);
        let name = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == to)
            .map_or_else(String::new, |s| s.name.clone());
        self.chronicle_push(
            now,
            ChronicleKind::Moved,
            old.members.clone(),
            Some(to),
            Some(home),
            f64::from(people),
            name,
        );
    }

    /// Where `id`, exiled from `home` and of household `household`, goes (M5a slice AN): the
    /// settlement their household knows whose kin, those they know and fed look draw them most,
    /// if any does.
    pub(super) fn exile_refuge(
        &self,
        ctx: &Ctx,
        id: PermanentId,
        household: PermanentId,
        home: PermanentId,
    ) -> Option<PermanentId> {
        let mp = &ctx.params.moving;
        let day = ctx.now.day_index();
        let fed_here = self.fed_share(ctx, home);
        let lives_in = |q: PermanentId| {
            self.person(q)
                .and_then(|p| self.household(p.household))
                .and_then(|h| h.settlement)
        };
        let kin = self.close_kin(id);
        let sat = |s: f64| s / (1.0 + s);
        self.known_places
            .of(household)
            .iter()
            .filter(|k| {
                k.settlement != home
                    && ctx
                        .land
                        .settlements
                        .iter()
                        .any(|s| s.id == k.settlement && s.abandoned.is_none())
            })
            .map(|k| {
                let kin_there = kin
                    .iter()
                    .filter(|&&q| lives_in(q) == Some(k.settlement))
                    .count();
                let known: f64 = self
                    .ties
                    .of(id)
                    .iter()
                    .filter(|t| lives_in(t.to) == Some(k.settlement))
                    .map(|t| t.known_at(day, &ctx.params.ties))
                    .sum();
                let fed = k.food.map_or(0.0, |f| f64::from(f) - fed_here);
                let pull = mp.w_kin * kin_there as f64 + mp.w_ties * sat(known) + mp.w_fed * fed;
                (k.settlement, pull)
            })
            .filter(|&(_, pull)| pull > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|(s, _)| s)
    }

    /// `id`, exiled from household `from` (already out of the world's tables, as `p`), lives on
    /// in settlement `to` in a household of their own beside its hearth, with nothing but what
    /// their household knew of other places.
    pub(super) fn exile_to(
        &mut self,
        ctx: &mut Ctx,
        id: PermanentId,
        p: &Person,
        from: PermanentId,
        to: PermanentId,
    ) {
        let now = ctx.now;
        let Some(hearth) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == to)
            .map(|s| s.hearth_m)
        else {
            return;
        };
        let left = self.household(from).and_then(|x| x.settlement);
        let (taste, admired) = self
            .household(from)
            .map_or((Default::default(), None), |x| (x.taste, x.admired));
        let hh = ctx.ids.allocate();
        let home = self.new_home_site(ctx, hearth, Some(hearth), hh);
        self.insert_household(Household {
            id: hh,
            members: vec![id],
            home,
            settlement: Some(to),
            stores: vec![0.0; ctx.catalog.goods.len()],
            stores_at: now,
            water_l: 0.0,
            water_at: now,
            known: Vec::new(),
            sheltered: false,
            keeping: crate::person::Keeping::default(),
            flows: Flows::default(),
            offers: Vec::new(),
            taste,
            admired,
            midden: crate::person::Midden::begun(now),
        });
        let mut person = p.clone();
        person.household = hh;
        person.partner = None;
        person.carrying = Default::default();
        person.pos = p.position_at(now.minutes() as f64);
        person.trip = None;
        self.insert_person(person);
        self.note_residence(id, Some(to), now, ResidenceWhy::Exiled);
        self.known_places.bring(from, hh, Some(to));
        let day = now.day_index();
        if let Some(left) = left {
            self.known_places
                .learn(hh, left, day, PlaceHow::Lived, None);
            self.contacts
                .moved(day.div_euclid(DAYS_PER_YEAR), left, to, 1);
        }
        let name = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == to)
            .map_or_else(String::new, |s| s.name.clone());
        self.chronicle_push(
            now,
            ChronicleKind::Moved,
            vec![id],
            Some(to),
            Some(home),
            1.0,
            name,
        );
        // They set out from where they stand, deciding afresh what to do.
        self.begin(ctx, id);
    }
}
