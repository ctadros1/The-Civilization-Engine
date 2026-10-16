//! Patron gifts (M7c slice BY, ADR-0026 §4; research 09-09 §1.7: clientelism is contingent
//! reciprocity). When a law is put to the gathering, its sponsor reckons who would stand where,
//! as each member's household reckons the law and as they believe each regards them; if it would
//! not pass, they may give food to households whose members a gift would bring round, the
//! cheapest first, and only if the gifts would carry it and cost their household less than the
//! law is worth to it. The gifts are ordinary gifts, recorded with their reason. A household owes
//! a patron what such gifts brought it within memory, and its members' stances on the patron's
//! laws gain `w_owed` a day of its food it was given. A patron who saw a household they gave to
//! stand against them gives it nothing more. Only gifts so recorded are read: the tie's help
//! balance stays unread, so a world where nobody gives as a patron votes as before.

use super::*;
use crate::polity::{Outcome, PatronGift, Stance};

impl Population {
    /// What household `household` owes `patrons` for gifts before a gathering within memory,
    /// points: [`crate::polity::PolityParams::w_owed`] times the days of its food they brought it.
    pub(super) fn owed_points(
        &self,
        ctx: &Ctx,
        pi: usize,
        household: PermanentId,
        patrons: [PermanentId; 2],
    ) -> f64 {
        let pp = &ctx.params.polity;
        let since = ctx.now.day_index() - i64::from(pp.owed_memory_days);
        let days: f64 = self.polities[pi]
            .patron_gifts
            .iter()
            .filter(|g| g.client == household && g.day >= since && patrons.contains(&g.patron))
            .map(|g| f64::from(g.days))
            .sum();
        pp.w_owed * days
    }

    /// Whether `patron` saw household `client`, having given it before a gathering, stand
    /// against a law of theirs at a later one they came to, within memory.
    fn crossed(&self, ctx: &Ctx, pi: usize, patron: PermanentId, client: PermanentId) -> bool {
        let since = ctx.now.day_index() - i64::from(ctx.params.polity.owed_memory_days);
        let polity = &self.polities[pi];
        let Some(first) = polity
            .patron_gifts
            .iter()
            .filter(|g| g.patron == patron && g.client == client)
            .map(|g| g.day)
            .min()
        else {
            return false;
        };
        polity.laws.iter().any(|l| {
            l.sponsor == patron
                && l.decided.is_some_and(|d| d.day_index() >= since.max(first))
                && l.stances.iter().any(|r| r.person == patron)
                && l.stances
                    .iter()
                    .any(|r| r.household == client && r.stance == Stance::Oppose)
        })
    }

    /// Law `law_id` of polity `pi` has been put to the gathering: its sponsor weighs giving
    /// before it meets (see the module's words).
    pub(super) fn patron_gives(&mut self, ctx: &Ctx, pi: usize, law_id: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let pp = &params.polity;
        if pp.w_owed <= 0.0 || pp.patron_gift_days <= 0.0 {
            return;
        }
        let day = now.day_index();
        let polity = &self.polities[pi];
        let (Some(law), Some(g)) = (
            polity.laws.iter().find(|l| l.id == law_id),
            polity.gathering.as_ref(),
        ) else {
            return;
        };
        let (sponsor, face) = (law.sponsor, law.holder.unwrap_or(law.sponsor));
        let Some(giver) = self.person(sponsor).map(|p| p.household) else {
            return;
        };
        // What the law is worth to the patron's household, points.
        let worth = g.stake(giver);
        if worth <= 0.0 {
            return;
        }
        let members = self.body_members(&ctx.land.fields, pi, now, params);
        let need_day = |pop: &Population, h: PermanentId| {
            pop.household(h).map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person
        };
        // Each member's stance as the patron reckons it: their household's lot, their regard for
        // the one the law names, and what their household owes already.
        let reckoned: Vec<(PermanentId, f64)> = members
            .iter()
            .map(|&(p, h)| {
                let s = if p == sponsor {
                    f64::INFINITY
                } else {
                    let regard = if p == face {
                        1.0
                    } else {
                        self.ties.regard(p, face, day, &params.ties)
                    };
                    g.stake(h)
                        + pp.w_regard * regard.clamp(0.0, 1.0)
                        + self.owed_points(ctx, pi, h, [sponsor, face])
                };
                (h, s)
            })
            .collect();
        let stance = |s: f64| {
            if s > pp.stance_margin {
                Stance::Support
            } else if s < -pp.stance_margin {
                Stance::Oppose
            } else {
                Stance::Abstain
            }
        };
        let body = self.polities[pi].body;
        let eligible = body.eligible(members.len());
        let present = members.len() as u32;
        let (mut support, mut oppose) = (0u32, 0u32);
        for &(_, s) in &reckoned {
            match stance(s) {
                Stance::Support => support += 1,
                Stance::Oppose => oppose += 1,
                Stance::Abstain => {}
            }
        }
        if body.decide(eligible, present, support, oppose) == Outcome::Passed {
            return;
        }
        // For each household a gift would bring round: the food that carries its member who
        // wants most, and the stances it turns.
        let mut households: Vec<PermanentId> = reckoned.iter().map(|r| r.0).collect();
        households.sort_unstable();
        households.dedup();
        let mut offers: Vec<(f64, PermanentId, u32, u32)> = Vec::new();
        for h in households {
            if h == giver || self.crossed(ctx, pi, sponsor, h) {
                continue;
            }
            let short: Vec<f64> = reckoned
                .iter()
                .filter(|r| r.0 == h && stance(r.1) != Stance::Support)
                .map(|r| r.1)
                .collect();
            let Some(least) = short.iter().copied().reduce(f64::min) else {
                continue;
            };
            let days = (pp.stance_margin - least) / pp.w_owed + 0.01;
            if days > pp.patron_gift_days {
                continue;
            }
            let opposed = short
                .iter()
                .filter(|&&s| stance(s) == Stance::Oppose)
                .count() as u32;
            offers.push((days * need_day(self, h), h, short.len() as u32, opposed));
        }
        // The cheapest turn first: an opposed member turned counts twice.
        let per = |o: &(f64, PermanentId, u32, u32)| o.0 / f64::from(o.2 + o.3);
        offers.sort_by(|a, b| per(a).total_cmp(&per(b)).then(a.1.cmp(&b.1)));
        let mut chosen = Vec::new();
        let mut kcal = 0.0;
        for o in offers {
            kcal += o.0;
            support += o.2;
            oppose -= o.3;
            chosen.push(o);
            if body.decide(eligible, present, support, oppose) == Outcome::Passed {
                break;
            }
        }
        if chosen.is_empty() || body.decide(eligible, present, support, oppose) != Outcome::Passed {
            return;
        }
        let Some(outlook) = self.outlook(ctx, giver) else {
            return;
        };
        if pp.w_gain * crate::polity::levy_cost(&outlook, kcal, pp) >= worth
            || crate::polity::cannot_pay(outlook.held, kcal, outlook.year_need, pp)
        {
            return;
        }
        for (kcal, client, _, _) in chosen {
            let stores = self
                .household(giver)
                .map(|x| stores_now(x, now, params, goods))
                .unwrap_or_default();
            let legs: Vec<Leg> = super::crime::take_goods(&stores, goods, f64::INFINITY, kcal)
                .into_iter()
                .map(|(good, amount)| Leg {
                    from: giver,
                    to: client,
                    good,
                    amount,
                })
                .collect();
            let given: f64 = legs
                .iter()
                .map(|l| l.amount * goods[l.good].kcal_per_kg)
                .sum();
            if given <= 1e-6 || !self.transfer(now, params, goods, &legs, Channel::Gift) {
                continue;
            }
            let help_h = self.household(client).map_or(0.0, |t| {
                let costs = self.own_costs_of(ctx, t);
                legs.iter()
                    .map(|l| costs.get(l.good).copied().flatten().unwrap_or(0.0) * l.amount)
                    .sum::<f64>()
            });
            self.note_between(
                ctx,
                None,
                client,
                giver,
                crate::ties::Act::GiftReceived,
                crate::ties::Act::GiftGiven,
                help_h,
            );
            let days = given / need_day(self, client);
            self.polities[pi].patron_gifts.push(PatronGift {
                patron: sponsor,
                giver,
                client,
                law: law_id,
                day,
                kcal: given as f32,
                days: days as f32,
            });
        }
    }

    /// Forgets polity `pi`'s patron gifts older than memory.
    pub(super) fn forget_patron_gifts(&mut self, ctx: &Ctx, pi: usize) {
        let since = ctx.now.day_index() - i64::from(ctx.params.polity.owed_memory_days);
        self.polities[pi].patron_gifts.retain(|g| g.day >= since);
    }

    /// Polity `polity`'s sponsor of law `law` weighs giving before its gathering: for tests.
    #[doc(hidden)]
    pub fn patron_gives_for_tests(&mut self, ctx: &Ctx, polity: PermanentId, law: PermanentId) {
        if let Some(pi) = self.polities.iter().position(|p| p.id == polity) {
            self.patron_gives(ctx, pi, law);
        }
    }
}
