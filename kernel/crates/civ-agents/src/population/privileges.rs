//! Privileges (M7c slice BX, ADR-0026 §3): a law that grants an organization by stake the sole
//! right to offer goods of a purpose at doors in the settlement (a licence), or to sell there such
//! goods fetched from another settlement (a staple). A household none of whose people belong to
//! it weighs keeping the privilege when it reviews what it offers, as it weighs paying a levy:
//! kept, it withdraws those offers; broken, they stand at its door, where every buyer sees them.
//! A breach becomes a case only if a member brings it (`cases.rs`).

use super::*;
use crate::crime::{Case, CaseStage};
use crate::market::Offer;
use crate::polity::{Grant, LawStatus, PolicyKind, PrivilegeDef};

/// Purpose tag for whether a household keeps a privilege at a review (M7c slice BX).
pub const PURPOSE_PRIVILEGE: u64 = 0x7072_6976_6b65_6570; // "privkeep"

impl Population {
    /// The privileges of the polity holding household `household`'s settlement that bind it (M7c
    /// slice BX): in force, for an organization that has not ended and that nobody of the
    /// household belongs to, by (polity, law index, terms).
    pub(crate) fn privileges_binding(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Vec<(usize, usize, PrivilegeDef)> {
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return Vec::new();
        };
        let own = self.stake_organizations_of(household);
        (0..self.polities[pi].laws.len())
            .filter_map(|li| {
                let l = &self.polities[pi].laws[li];
                if l.kind != PolicyKind::Privilege || l.status != LawStatus::InForce {
                    return None;
                }
                let org = l.target?;
                if own.contains(&org) || self.factions.get(org).is_none_or(|f| f.ended.is_some()) {
                    return None;
                }
                let def = ctx.catalog.policies.get(usize::from(l.policy))?.privilege?;
                Some((pi, li, def))
            })
            .collect()
    }

    /// The goods of `rows` a privilege `def` covers for household `household` (M7c slice BX): of
    /// its purpose for a licence; for a staple, of its purpose and bought by the household at
    /// another settlement's door within a year.
    fn covered(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        def: PrivilegeDef,
        rows: &[Offer],
    ) -> Vec<usize> {
        let goods = &ctx.catalog.goods;
        let of = |g: usize| goods.get(g).is_some_and(|d| d.purpose == def.purpose);
        let fetched: Vec<usize> = match def.grant {
            Grant::Licence => Vec::new(),
            Grant::Staple => self
                .household(household)
                .and_then(|x| x.settlement)
                .map(|s| {
                    self.bought_away(s, ctx.now.day_index() - DAYS_PER_YEAR)
                        .into_iter()
                        .filter(|b| b.household == household)
                        .map(|b| b.good)
                        .collect()
                })
                .unwrap_or_default(),
        };
        let mut out: Vec<usize> = rows
            .iter()
            .map(|o| usize::from(o.good))
            .filter(|&g| of(g) && (def.grant == Grant::Licence || fetched.contains(&g)))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Household `household` weighs each privilege that binds it over offers `rows` it is about
    /// to post, for itself or a workshop it owns (M7c slice BX): its elder, if they know the law,
    /// keeps it with the chance the norms they hold, where they stood on it and their regard for
    /// its sponsor give against what those offers would bring (the market's margin on them, as
    /// the household reckons a unit), as paying a levy is weighed; kept, the offers it covers are
    /// withdrawn. Each weighing is counted in the law's compliance: kept as complied, broken as
    /// broken, or broken unaware. The offers that stand, and the purposes it keeps a staple over.
    pub(crate) fn keep_privileges(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        mut rows: Vec<Offer>,
    ) -> (Vec<Offer>, Vec<crate::params::GoodUse>) {
        let mut staples = Vec::new();
        let binding = self.privileges_binding(ctx, household);
        if binding.is_empty() {
            return (rows, staples);
        }
        let (now, params) = (ctx.now, ctx.params);
        let (pp, day) = (&params.polity, now.day_index());
        let Some(who) = self.elder_of(household, now, params) else {
            return (rows, staples);
        };
        for (pi, li, def) in binding {
            let covered = self.covered(ctx, household, def, &rows);
            if covered.is_empty() {
                continue;
            }
            let law = &self.polities[pi].laws[li];
            if !law.knows(who) {
                self.polities[pi].laws[li].compliance.broken_unaware += 1;
                continue;
            }
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == who)
                .map_or(0.0, |r| r.stance.sign());
            let regard = self.ties.regard(who, law.sponsor, day, &params.ties);
            let Some(outlook) = self.outlook(ctx, household) else {
                continue;
            };
            let kcal: f64 = self.household(household).map_or(0.0, |x| {
                let hour = self.hour_kcal(ctx, x);
                rows.iter()
                    .filter(|o| covered.contains(&usize::from(o.good)))
                    .map(|o| {
                        f64::from(o.units)
                            * self
                                .unit_kcal(ctx, x, usize::from(o.good))
                                .unwrap_or(f64::from(o.ask_h) * hour)
                    })
                    .sum::<f64>()
                    * params.market.margin
            });
            let cost = pp.w_gain * crate::polity::levy_cost(&outlook, kcal, pp);
            self.ensure_norm_state(ctx, who);
            let norm = self.norm_points(ctx, who);
            let points = crate::polity::comply_points(norm, stance, regard, cost, pp);
            let chance = crate::polity::comply_chance(points);
            let key = [ctx.seed, PURPOSE_PRIVILEGE, who.get(), day as u64];
            let c = &mut self.polities[pi].laws[li].compliance;
            if Rng64::from_key(&key).next_f64() < chance {
                c.complied += 1;
                rows.retain(|o| !covered.contains(&usize::from(o.good)));
                if def.grant == Grant::Staple {
                    staples.push(def.purpose);
                }
            } else {
                c.broken += 1;
            }
        }
        (rows, staples)
    }

    /// The households of polity `pi`'s settlements that breach privilege law `li` now (M7c slice
    /// BX), in id order: ones it binds whose door, or whose open workshop's, offers goods it
    /// covers. An offer is posted at a door, so every buyer sees it.
    pub(crate) fn breaching(&self, ctx: &Ctx, pi: usize, li: usize) -> Vec<PermanentId> {
        let l = &self.polities[pi].laws[li];
        let (Some(org), Some(def)) = (
            l.target,
            ctx.catalog
                .policies
                .get(usize::from(l.policy))
                .and_then(|d| d.privilege),
        ) else {
            return Vec::new();
        };
        let mut out: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| !x.members.is_empty() && self.polities[pi].holds(x.settlement))
            .filter(|x| !self.stake_organizations_of(x.id).contains(&org))
            .filter(|x| {
                let mut rows = x.offers.clone();
                rows.extend(
                    self.firms
                        .iter()
                        .filter(|f| f.owner == x.id && f.is_open())
                        .flat_map(|f| f.offers.iter().copied()),
                );
                !self.covered(ctx, x.id, def, &rows).is_empty()
            })
            .map(|x| x.id)
            .collect();
        out.sort_unstable();
        out
    }

    /// Whether a member of an organization brings a case against a household breaching its
    /// privilege at polity `pi` (M7c slice BX, ADR-0026 §3), at the polity's review: for each
    /// privilege in force with a bundle and each household breaching it against which no case
    /// under it is open, each grown member living in the polity's settlements weighs U = V − C
    /// (research 09-07 §4.1), V what a finding would bring their household (the bundle's
    /// compensation, and its share of what the breach's offers would bring at the market's
    /// margin, among the members who offer such goods), in days of its food at the points a day
    /// is worth in a dispute, C the cost of bringing a case and their regard for the breaching
    /// household's elder. The one with the most to gain, if anyone, brings it; the gathering or a
    /// forum that hears privileges hears it.
    pub(super) fn weigh_breaches(&mut self, ctx: &mut Ctx, pi: usize) {
        let (now, params) = (ctx.now, ctx.params);
        let (pp, cp, day) = (&params.polity, &params.crime, now.day_index());
        let laws: Vec<usize> = (0..self.polities[pi].laws.len())
            .filter(|&li| {
                let l = &self.polities[pi].laws[li];
                l.kind == PolicyKind::Privilege
                    && l.status == LawStatus::InForce
                    && (l.sanction.compensation_days > 0.0 || l.sanction.fine_days > 0.0)
            })
            .collect();
        for li in laws {
            let (law_id, org, sanction) = {
                let l = &self.polities[pi].laws[li];
                let Some(org) = l.target else {
                    continue;
                };
                (l.id, org, l.sanction)
            };
            let Some(def) = ctx
                .catalog
                .policies
                .get(usize::from(self.polities[pi].laws[li].policy))
                .and_then(|d| d.privilege)
            else {
                continue;
            };
            // Its grown members of the polity, and how many offer such goods at home.
            let members: Vec<(PermanentId, PermanentId)> = self
                .factions
                .members_of(org)
                .filter_map(|m| {
                    let p = self.person(m.person)?;
                    let x = self.household(p.household)?;
                    (self.polities[pi].holds(x.settlement)
                        && p.age_years(now) >= params.family.independent_age)
                        .then_some((m.person, x.id))
                })
                .collect();
            let sellers = members
                .iter()
                .map(|m| m.1)
                .filter(|&h| self.offers_at_home(ctx, h, def.purpose))
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                .max(1) as f64;
            for hh in self.breaching(ctx, pi, li) {
                let open = self.order.cases.iter().any(|c| {
                    c.law == law_id && c.accused_household == hh && c.stage == CaseStage::Open
                });
                let Some(accused) = self.elder_of(hh, now, params).filter(|_| !open) else {
                    continue;
                };
                let Some(x) = self.household(hh) else {
                    continue;
                };
                let need = self.day_need(hh, params);
                let compensation = f64::from(sanction.compensation_days) * need;
                let hour = self.hour_kcal(ctx, x);
                let mut rows = x.offers.clone();
                rows.extend(
                    self.firms
                        .iter()
                        .filter(|f| f.owner == hh && f.is_open())
                        .flat_map(|f| f.offers.iter().copied()),
                );
                let covered = self.covered(ctx, hh, def, &rows);
                let sold: f64 = rows
                    .iter()
                    .filter(|o| covered.contains(&usize::from(o.good)))
                    .map(|o| f64::from(o.units) * f64::from(o.ask_h) * hour)
                    .sum::<f64>()
                    * params.market.margin
                    / sellers;
                let mut best: Option<(f64, PermanentId, PermanentId)> = None;
                for &(m, h) in &members {
                    let share = if self.offers_at_home(ctx, h, def.purpose) {
                        sold
                    } else {
                        0.0
                    };
                    let v = cp.w_demand_loss * (compensation + share) / self.day_need(h, params);
                    let regard = self
                        .ties
                        .regard(m, accused, day, &params.ties)
                        .clamp(0.0, 1.0);
                    let u = v - cp.report_cost - pp.w_regard * regard;
                    if u > 0.0 && best.is_none_or(|b| u > b.0) {
                        best = Some((u, m, h));
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
                    law: law_id,
                    accuser: household,
                    by,
                    accused,
                    accused_household: hh,
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
                });
                let (place, name) = ctx
                    .land
                    .settlements
                    .iter()
                    .find(|s| s.id == settlement)
                    .map_or((None, String::new()), |s| {
                        (Some(s.hearth_m), s.name.clone())
                    });
                let words = format!(
                    "brought a case at {name} against {}'s household for breaking {}.",
                    self.name_of(accused),
                    self.law_words_of(ctx, pi, law_id)
                );
                self.chronicle_push(
                    now,
                    ChronicleKind::CaseBrought,
                    vec![by, accused],
                    Some(settlement),
                    place,
                    0.0,
                    words,
                );
            }
        }
    }
}
