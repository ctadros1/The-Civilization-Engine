//! Trade laws (M7c slice BV, ADR-0026 §1): a duty on goods a member's household buys at another
//! settlement's door, owed to the polity's store and paid at home in kind as the levy is, or kept
//! back; and an embargo on buying at the doors of a named settlement, kept or broken person by
//! person as a curfew is. The taxable event is the purchase: nothing is in transit (ADR-0019 §3).
//! What a household forecasts of either comes from what its own people bought and what it offers
//! at home, and from the trades the other markets remember of its settlement's people.

use super::*;
use crate::polity::{LawStatus, Membership, PolicyKind};

/// Purpose tag for whether one who owes a duty pays it (M7c slice BV).
pub const PURPOSE_DUTY: u64 = 0x6475_7479_7061_7931; // "dutypay1"
/// Purpose tags for whether a duty's collector sees a purchase, asks a payment, and whether it is
/// paid (M7c slice BY).
pub const PURPOSE_COLLECT: u64 = 0x636f_6c6c_6563_7431; // "collect1"
pub const PURPOSE_COLLECT_ASK: u64 = 0x636f_6c61_736b_3031; // "colask01"
pub const PURPOSE_COLLECT_PAY: u64 = 0x636f_6c70_6179_3031; // "colpay01"

/// A purchase someone of a settlement made at another settlement's door, as the seller's market
/// remembers it (M7c slice BV).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoughtAway {
    /// The buyer's household.
    pub household: PermanentId,
    /// The settlement whose door it was bought at.
    pub market: PermanentId,
    /// The good, by index in the catalog's goods.
    pub good: usize,
    pub units: f64,
    /// What was paid for it: the payment good, by index in the catalog's goods, and how much.
    pub payment: usize,
    pub paid: f64,
}

impl Population {
    /// Purchases by households of `settlement` at other settlements' doors on or after day
    /// `since`, from the trades each market remembers, in market then trade order.
    pub(crate) fn bought_away(&self, settlement: PermanentId, since: i64) -> Vec<BoughtAway> {
        let mut out = Vec::new();
        for m in self.markets.iter().filter(|m| m.settlement != settlement) {
            for t in m
                .recent
                .iter()
                .filter(|t| t.from == Some(settlement) && t.at.day_index() >= since)
            {
                out.push(BoughtAway {
                    household: t.buyer,
                    market: m.settlement,
                    good: usize::from(t.good),
                    units: f64::from(t.units),
                    payment: usize::from(t.payment),
                    paid: f64::from(t.paid),
                });
            }
        }
        out
    }

    /// Whether household `household` offers goods of `purpose` at its own door or its
    /// workshops' (M7c slice BV): one a duty may spare.
    pub(crate) fn offers_at_home(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        purpose: crate::params::GoodUse,
    ) -> bool {
        let goods = &ctx.catalog.goods;
        let of = |good: u16| {
            goods
                .get(usize::from(good))
                .is_some_and(|g| g.purpose == purpose)
        };
        self.household(household)
            .is_some_and(|x| x.offers.iter().any(|o| of(o.good)))
            || self
                .firms
                .iter()
                .filter(|f| f.owner == household && f.closed.is_none())
                .any(|f| f.offers.iter().any(|o| of(o.good)))
    }

    /// Whether polity `pi` recognizes organization `org` by a law in force (M7c slice BW).
    pub(crate) fn recognizes(&self, pi: usize, org: PermanentId) -> bool {
        self.polities[pi].laws.iter().any(|l| {
            l.kind == PolicyKind::Recognize
                && l.status == LawStatus::InForce
                && l.target == Some(org)
        })
    }

    /// Whether household `household` belongs to membership `m` of a closed body (M7c slice BZ,
    /// ADR-0026 §5): an organization it belongs to is recognized by the polity that holds its
    /// settlement (`Organized`) or holds one of its privileges in force (`Privileged`).
    pub(crate) fn closed_member(&self, m: Membership, household: PermanentId) -> bool {
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return false;
        };
        self.stake_organizations_of(household)
            .into_iter()
            .any(|org| match m {
                Membership::Organized => self.recognizes(pi, org),
                Membership::Privileged => self.polities[pi].laws.iter().any(|l| {
                    l.kind == PolicyKind::Privilege
                        && l.status == LawStatus::InForce
                        && l.target == Some(org)
                }),
                _ => false,
            })
    }

    /// The organizations by stake, not ended, that someone of household `household` belongs
    /// to, in id order (M7c slice BW).
    pub(crate) fn stake_organizations_of(&self, household: PermanentId) -> Vec<PermanentId> {
        let mut out: Vec<PermanentId> = self
            .household(household)
            .map(|x| {
                x.members
                    .iter()
                    .filter_map(|&p| self.factions.membership(p))
                    .filter_map(|m| self.factions.get(m.faction))
                    .filter(|f| f.ended.is_none() && f.stake.is_some())
                    .map(|f| f.id)
                    .collect()
            })
            .unwrap_or_default();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Whether household `household` owes polity `pi` no duty on goods of `purpose` because an
    /// organization of that purpose someone of it belongs to is recognized by a law in force
    /// (M7c slice BW).
    fn spared_by_recognition(
        &self,
        pi: usize,
        household: PermanentId,
        purpose: crate::params::GoodUse,
    ) -> bool {
        self.stake_organizations_of(household)
            .into_iter()
            .any(|org| {
                self.recognizes(pi, org)
                    && self
                        .factions
                        .get(org)
                        .and_then(|f| f.stake)
                        .is_some_and(|s| s.purpose == purpose)
            })
    }

    /// What a unit of good `good` is worth to household `hh`, kcal: its own cost in hours at what
    /// an hour of its work brings it in food (M7c slice BV); none when it cannot reckon the good.
    pub(crate) fn unit_kcal(&self, ctx: &Ctx, hh: &Household, good: usize) -> Option<f64> {
        let hours = self.own_costs_of(ctx, hh).get(good).copied().flatten()?;
        Some(hours * self.hour_kcal(ctx, hh))
    }

    /// The duties polity `pi` has that a purchase of `good` by household `household` at
    /// `market`'s doors is liable to, by law index: in force (or still kept by `who` after it was
    /// struck down), laid on the good's purpose, the market not one of the polity's own, and the
    /// household not spared as one that offers such goods at home.
    fn duties_on(
        &self,
        ctx: &Ctx,
        pi: usize,
        who: PermanentId,
        household: PermanentId,
        market: PermanentId,
        good: usize,
    ) -> Vec<usize> {
        let Some(purpose) = ctx.catalog.goods.get(good).map(|g| g.purpose) else {
            return Vec::new();
        };
        if self.polities[pi].holds(Some(market))
            || self.spared_by_recognition(pi, household, purpose)
        {
            return Vec::new();
        }
        (0..self.polities[pi].laws.len())
            .filter(|&li| {
                let l = &self.polities[pi].laws[li];
                let def = ctx
                    .catalog
                    .policies
                    .get(usize::from(l.policy))
                    .filter(|d| d.kind == PolicyKind::Duty);
                let Some(d) = def.and_then(|d| d.duty) else {
                    return false;
                };
                (l.status == LawStatus::InForce || self.keeps_struck(ctx, who, l))
                    && d.purpose == purpose
                    && !(d.exempt_sellers && self.offers_at_home(ctx, household, purpose))
            })
            .collect()
    }

    /// The share of good `good` bought at `market`'s doors that the duties in force of household
    /// `hh`'s polity take (M7c slice BV): what replacing a unit from there costs it the more.
    pub(crate) fn duty_rate(
        &self,
        ctx: &Ctx,
        hh: &Household,
        good: usize,
        market: PermanentId,
    ) -> f64 {
        let Some(pi) = hh.settlement.and_then(|s| self.polity_of(s)) else {
            return 0.0;
        };
        let Some(purpose) = ctx.catalog.goods.get(good).map(|g| g.purpose) else {
            return 0.0;
        };
        if self.polities[pi].holds(Some(market)) {
            return 0.0;
        }
        self.polities[pi]
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce)
            .filter_map(|l| {
                let d = ctx
                    .catalog
                    .policies
                    .get(usize::from(l.policy))
                    .filter(|d| d.kind == PolicyKind::Duty)?
                    .duty?;
                (d.purpose == purpose
                    && !(d.exempt_sellers && self.offers_at_home(ctx, hh.id, purpose)))
                .then(|| f64::from(l.levy_share))
            })
            .sum()
    }

    /// Person `who` of household `household` bought `units` of good `good` at `market`'s doors
    /// (M7c slice BV): each duty of their polity it is liable to is owed to the store, a share of
    /// the units, and paid at home in kind now if they choose to, as the levy is weighed (the
    /// norms they hold, where they stood on it and their regard for its sponsor, against what it
    /// costs); one who never heard of the law owes nothing they know of.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn pay_duties(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        household: PermanentId,
        market: PermanentId,
        good: usize,
        units: f64,
        (payment, paid): (usize, f64),
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let pp = &params.polity;
        // What a unit of the purchase is worth to the household: its own cost of it, else what
        // it gave for it (the payment's own worth to it, or the food in it).
        let unit_worth = self.household(household).map_or(0.0, |x| {
            self.unit_kcal(ctx, x, good).unwrap_or_else(|| {
                let each = self
                    .unit_kcal(ctx, x, payment)
                    .or_else(|| goods.get(payment).map(|g| g.kcal_per_kg))
                    .unwrap_or(0.0);
                each * paid / units.max(1e-9)
            })
        });
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        let day = now.day_index();
        for li in self.duties_on(ctx, pi, who, household, market, good) {
            let law = &self.polities[pi].laws[li];
            let owed = units * f64::from(law.levy_share);
            if owed <= 1e-9 {
                continue;
            }
            let (store, sponsor) = (self.polities[pi].id, law.sponsor);
            // A collector who sees the purchase assesses it, whatever the buyer knows (M7c slice
            // BY).
            if let Some(collector) = law.holder.filter(|&c| c != who) {
                let key = [
                    ctx.seed,
                    PURPOSE_COLLECT,
                    who.get(),
                    now.minutes() as u64,
                    li as u64,
                ];
                if Rng64::from_key(&key).next_f64() < params.crime.collector_sight {
                    self.collect_duty(ctx, pi, li, collector, household, good, owed, unit_worth);
                    continue;
                }
            }
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
            let kcal = unit_worth * owed;
            let regard = self.ties.regard(who, sponsor, day, &params.ties);
            let cost = pp.w_gain * crate::polity::levy_cost(&outlook, kcal, pp);
            self.ensure_norm_state(ctx, who);
            let norm = self.norm_points(ctx, who);
            let points = crate::polity::comply_points(norm, stance, regard, cost, pp);
            let chance = crate::polity::comply_chance(points);
            let key = [ctx.seed, PURPOSE_DUTY, who.get(), now.minutes() as u64];
            if Rng64::from_key(&key).next_f64() >= chance {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.evaded += 1;
                c.withheld_kg += owed;
                continue;
            }
            let held = self
                .household(household)
                .and_then(|x| x.stores.get(good))
                .copied()
                .unwrap_or(0.0)
                .max(0.0);
            let paid = owed.min(held);
            let leg = Leg {
                from: household,
                to: store,
                good,
                amount: paid,
            };
            if paid > 1e-9 && self.transfer(now, params, goods, &[leg], Channel::Duty) {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.complied += 1;
                c.levied_kg += paid;
            }
        }
    }

    /// Duty law `li` of polity `pi`'s collector `collector` saw household `household` bring home
    /// a purchase of good `good` owing `owed` units, each worth `unit` kcal to it (M7c slice BY, ADR-0026 §4; research 09-09
    /// §1.3: under-assessing for a payment). They assess it in full, or ask the household for
    /// `collector_ask_share` of what it owes, worth `w_demand_loss` a day of their own household's
    /// food against `w_objection` for their objection (never above its filter) and `w_seen` for
    /// the chance they believe they run of being found out. Asked, the household's elder pays when
    /// what it saves is worth more than their own objection; paid, nothing is assessed and the
    /// payment moves on the `Bribe` channel; refused, the duty is assessed in full and the
    /// household's grown members who know the law hold the asking against the office.
    #[allow(clippy::too_many_arguments)]
    fn collect_duty(
        &mut self,
        ctx: &Ctx,
        pi: usize,
        li: usize,
        collector: PermanentId,
        household: PermanentId,
        good: usize,
        owed: f64,
        unit: f64,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.crime;
        let need_of = |pop: &Population, h: PermanentId| {
            pop.household(h).map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person
        };
        let (store, law_id) = (self.polities[pi].id, self.polities[pi].laws[li].id);
        let owed_kcal = owed * unit;
        let Some((collector_hh, objection, risk)) = self
            .person(collector)
            .map(|p| (p.household, f64::from(p.objection), f64::from(p.risk_seen)))
        else {
            return;
        };
        let pay = cp.collector_ask_share * owed_kcal;
        let ask = (objection < cp.objection_filter && pay > 1.0).then(|| {
            cp.w_demand_loss * pay / need_of(self, collector_hh)
                - cp.w_objection * objection
                - cp.w_seen * risk
        });
        let key = [
            ctx.seed,
            PURPOSE_COLLECT_ASK,
            collector.get(),
            now.minutes() as u64,
        ];
        let asks = ask.is_some_and(|a| {
            crate::demography::pick_softmax(&[0.0, a], Rng64::from_key(&key).next_f64()) == Some(1)
        });
        if asks {
            self.polities[pi].laws[li].compliance.asked += 1;
            let elder = self.elder_of(household, now, params);
            let payer_objection = elder
                .and_then(|e| self.person(e))
                .map_or(1.0, |p| f64::from(p.objection));
            let points = cp.w_demand_loss * (owed_kcal - pay) / need_of(self, household)
                - cp.w_objection * payer_objection;
            let key = [
                ctx.seed,
                PURPOSE_COLLECT_PAY,
                household.get(),
                now.minutes() as u64,
            ];
            let pays =
                crate::demography::pick_softmax(&[0.0, points], Rng64::from_key(&key).next_f64())
                    == Some(1);
            let stores = self
                .household(household)
                .map(|x| stores_now(x, now, params, goods))
                .unwrap_or_default();
            let legs: Vec<Leg> = super::crime::take_goods(&stores, goods, f64::INFINITY, pay)
                .into_iter()
                .map(|(good, amount)| Leg {
                    from: household,
                    to: collector_hh,
                    good,
                    amount,
                })
                .collect();
            let kg: f64 = legs.iter().map(|l| l.amount).sum();
            if pays && kg > 1e-9 && self.transfer(now, params, goods, &legs, Channel::Bribe) {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.bribed += 1;
                c.bribe_kg += kg;
                return;
            }
            let harm = pay / need_of(self, household);
            for m in self.grown_of(household, now, params) {
                if self.polities[pi].laws[li].knows(m) {
                    self.grieve(
                        ctx,
                        m,
                        crate::word::Grieved::Treatment,
                        crate::word::Blamed::Office(law_id),
                        law_id,
                        harm,
                        crate::word::Wrong::Extorted,
                    );
                }
            }
        }
        let held = self
            .household(household)
            .and_then(|x| x.stores.get(good))
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        let paid = owed.min(held);
        let leg = Leg {
            from: household,
            to: store,
            good,
            amount: paid,
        };
        if paid > 1e-9 && self.transfer(now, params, goods, &[leg], Channel::Duty) {
            let c = &mut self.polities[pi].laws[li].compliance;
            c.complied += 1;
            c.assessed += 1;
            c.levied_kg += paid;
        }
    }

    /// The embargo of person `who`'s polity on `market`'s doors that they know of, if any, and
    /// whether they keep it (M7c slice BV): the points the norms they hold, where they stood on it
    /// and their regard for its sponsor give keeping a law, as for a curfew.
    pub(crate) fn embargo_on(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        household: PermanentId,
        market: PermanentId,
    ) -> Option<(usize, usize, bool)> {
        let pi = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))?;
        let li = (0..self.polities[pi].laws.len()).find(|&li| {
            let l = &self.polities[pi].laws[li];
            l.target == Some(market)
                && (l.status == LawStatus::InForce || self.keeps_struck(ctx, who, l))
                && ctx
                    .catalog
                    .policies
                    .get(usize::from(l.policy))
                    .is_some_and(|d| d.kind == PolicyKind::Embargo)
        })?;
        let law = &self.polities[pi].laws[li];
        if !law.knows(who) {
            return Some((pi, li, false));
        }
        let stance = law
            .stances
            .iter()
            .find(|r| r.person == who)
            .map_or(0.0, |r| r.stance.sign());
        let sponsor = law.sponsor;
        let regard = self
            .ties
            .regard(who, sponsor, ctx.now.day_index(), &ctx.params.ties);
        self.ensure_norm_state(ctx, who);
        let norm = self.norm_points(ctx, who);
        let points = crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity);
        Some((pi, li, points > 0.0))
    }

    /// Whether person `who` of household `household` keeps an embargo of their polity on the
    /// doors of the settlement seller `seller` (a household or a workshop) belongs to (M7c slice
    /// BV): one they know of and would keep, as [`Population::embargo_on`] weighs it, from the
    /// norms they already hold.
    pub(crate) fn keeps_embargo(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        household: PermanentId,
        seller: PermanentId,
    ) -> bool {
        let market = match self.firm(seller) {
            Some(f) => f.settlement,
            None => self.household(seller).and_then(|x| x.settlement),
        };
        let Some(market) = market else {
            return false;
        };
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return false;
        };
        self.polities[pi].laws.iter().any(|l| {
            l.target == Some(market)
                && l.knows(who)
                && (l.status == LawStatus::InForce || self.keeps_struck(ctx, who, l))
                && ctx
                    .catalog
                    .policies
                    .get(usize::from(l.policy))
                    .is_some_and(|d| d.kind == PolicyKind::Embargo)
                && {
                    let stance = l
                        .stances
                        .iter()
                        .find(|r| r.person == who)
                        .map_or(0.0, |r| r.stance.sign());
                    let regard =
                        self.ties
                            .regard(who, l.sponsor, ctx.now.day_index(), &ctx.params.ties);
                    let norm = self.norm_points(ctx, who);
                    crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity)
                        > 0.0
                }
        })
    }

    /// Whether a household of polity `pi` offers at home goods of a purpose that people of its
    /// settlement bought at another settlement's doors within a year, and holds a report of that
    /// market (M7c slice BV): the issue *goods fetched elsewhere*. A household knows the market
    /// sells such goods by its report; the purchases are the other markets' records.
    pub(super) fn fetched_elsewhere(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> bool {
        let settlement = self.polities[pi].settlement;
        let away = self.bought_away(settlement, ctx.now.day_index() - DAYS_PER_YEAR);
        if away.is_empty() {
            return false;
        }
        let goods = &ctx.catalog.goods;
        households.iter().any(|&h| {
            let reports = self.reports.of(h);
            away.iter().any(|b| {
                b.household != h
                    && goods.get(b.good).is_some_and(|g| {
                        self.offers_at_home(ctx, h, g.purpose)
                            && reports
                                .iter()
                                .any(|r| r.market == b.market && usize::from(r.good) == b.good)
                    })
            })
        })
    }
}
