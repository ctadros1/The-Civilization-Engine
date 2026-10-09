//! The watch at work (M4b slice AC, ADR-0015 §6): the one a law names keeps watch over the
//! households' stores at night, in rounds they choose to walk. A round is a set path through the
//! settlement's homes, a stand at each; standing there they are someone about, so a taker who
//! notices them turns back and one who does not is seen (research 12-04 §1.4: a patrol affects
//! the places it visits and the people who observe it). Nothing here reads an incident, and the
//! watch's hours are a reported measure, never an input.

use super::*;
use crate::crime::{Kept, Sighting};
use crate::decide::WatchFacts;
use crate::ledger::Leg;
use crate::polity::LawStatus;

/// The night a minute of the day belongs to: an evening's day, or the day before for the small
/// hours.
fn night_of(now: SimTime) -> i64 {
    let day = now.day_index();
    if now.minutes().rem_euclid(MINUTES_PER_DAY) < 12 * 60 {
        day - 1
    } else {
        day
    }
}

impl Population {
    /// The round `person` of household `hh` could walk now, if a law in force names them to keep
    /// watch where they live, it is dark, and they have rounds left tonight: the next homes of
    /// the settlement's set round, and what keeping it is worth to them.
    pub(super) fn watch_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
        dark: bool,
    ) -> Option<WatchFacts> {
        if !dark {
            return None;
        }
        let cp = &ctx.params.crime;
        let settlement = hh.settlement?;
        let polity = &self.polities[self.polity_of(settlement)?];
        let (_, law) = polity.watchers().find(|(h, _)| *h == person)?;
        let rounds = cp.rounds_per_night.max(1);
        let tonight = if law.watch.night == night_of(ctx.now) {
            u32::from(law.watch.tonight)
        } else {
            0
        };
        if tonight >= rounds {
            return None;
        }
        let stops = self.round_stops(ctx, settlement, hh.id, law.watch.next);
        if stops.is_empty() {
            return None;
        }
        Some(WatchFacts {
            stops: stops.into_iter().map(|(_, at)| at).collect(),
            points: cp.w_watch * (1.0 - f64::from(tonight) / f64::from(rounds)),
        })
    }

    /// The homes of settlement `settlement` a round starting at `next` passes, but for the
    /// watcher's own: the settlement's households in id order, from `next` on, round to the
    /// start, `round_stops` of them.
    fn round_stops(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        own: PermanentId,
        next: u32,
    ) -> Vec<(PermanentId, (f32, f32))> {
        let mut homes: Vec<(PermanentId, (f32, f32))> = self
            .households
            .iter()
            .filter(|(_, x)| {
                x.settlement == Some(settlement) && x.id != own && !x.members.is_empty()
            })
            .map(|(_, x)| (x.id, x.home))
            .collect();
        homes.sort_unstable_by_key(|h| h.0);
        let n = homes.len();
        if n == 0 {
            return Vec::new();
        }
        let start = next as usize % n;
        let take = (ctx.params.crime.round_stops as usize).clamp(1, n);
        (0..take).map(|k| homes[(start + k) % n]).collect()
    }

    /// `person` sets out on a round (M4b slice AC): the watch records it, and its next round
    /// begins where this one ends.
    pub(super) fn start_round(&mut self, ctx: &Ctx, person: PermanentId) {
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
        let night = night_of(ctx.now);
        let stops = ctx.params.crime.round_stops.max(1);
        let Some(law) = self.polities[pi].laws.iter_mut().find(|l| {
            l.status == LawStatus::InForce
                && l.kind == crate::polity::PolicyKind::KeepWatch
                && l.holder == Some(person)
        }) else {
            return;
        };
        let w = &mut law.watch;
        if w.night != night {
            w.night = night;
            w.tonight = 0;
        }
        w.tonight = w.tonight.saturating_add(1);
        w.rounds += 1;
        w.next = w.next.wrapping_add(stops);
    }

    /// `person` stood watch, or walked between stands, for `minutes`.
    pub(super) fn stood_watch(&mut self, person: PermanentId, minutes: u32) {
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
        if let Some(law) = self.polities[pi].laws.iter_mut().find(|l| {
            l.status == LawStatus::InForce
                && l.kind == crate::polity::PolicyKind::KeepWatch
                && l.holder == Some(person)
        }) {
            law.watch.minutes += f64::from(minutes);
        }
    }
}

/// Purpose tag for a watcher's choice about a taking they saw.
pub const PURPOSE_SIGHT: u64 = 0x7369_6768_7465_6431; // "sighted1"
/// Purpose tag for a taker's household's choice to pay a watcher to say nothing.
pub const PURPOSE_PAY: u64 = 0x7061_7973_696c_6e74; // "paysilnt"

impl Population {
    /// Whether `person` keeps the watch of the settlement household `household` lives in.
    pub(super) fn keeps_watch_over(&self, person: PermanentId, household: PermanentId) -> bool {
        self.household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
            .is_some_and(|pi| self.polities[pi].watchers().any(|(h, _)| h == person))
    }

    /// `officer`, who keeps the watch, saw `taker` take from household `victim` in incident
    /// `incident`, and chooses once what to do with it (M4b slice AC, ADR-0015 §6; research 09-09
    /// §1.3: ΔU = B + K − C − M − p̂(F + V)): bring it before the gathering under a law against
    /// taking they know of (or, with none, tell the household taken from), say nothing, or ask
    /// the taker's household for food to say nothing. Their duty and their regard for the
    /// household taken from weigh toward telling, their regard for the taker against it; what
    /// they would be paid weighs toward asking, against their objection to taking what is not
    /// theirs (above its moral filter they never ask) and the chance they believe they run of
    /// being found out. A household asked pays when what it would face in a case weighs more
    /// than the food; one that refuses is brought before the gathering.
    pub(super) fn officer_sees(
        &mut self,
        ctx: &mut Ctx,
        officer: PermanentId,
        incident: u32,
        victim: PermanentId,
        taker: PermanentId,
    ) -> Kept {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.crime;
        let day = now.day_index();
        let tp = &params.ties;
        let need_of = |pop: &Population, h: PermanentId| {
            pop.household(h).map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person
        };
        let regard_victim = self.elder_of(victim, now, params).map_or(0.0, |e| {
            self.ties.regard(officer, e, day, tp).clamp(0.0, 1.0)
        });
        let regard_taker = self.ties.regard(officer, taker, day, tp).clamp(0.0, 1.0);
        let report = cp.w_watch_report + cp.w_forgive * (regard_victim - regard_taker);
        let law = self.law_against_taking(ctx, victim, officer);
        let (Some(officer_hh), Some(taker_hh)) = (
            self.person(officer).map(|p| p.household),
            self.person(taker).map(|p| p.household),
        ) else {
            return Kept::LookedAway;
        };
        let (objection, risk) = self.person(officer).map_or((1.0, 1.0), |p| {
            (f64::from(p.objection), f64::from(p.risk_seen))
        });
        // Asking: only where a case is what saying nothing spares the taker.
        let ask = law
            .filter(|_| objection < cp.objection_filter)
            .and_then(|_| {
                let x = self.household(taker_hh)?;
                let need = need_of(self, taker_hh);
                let stores = stores_now(x, now, params, goods);
                let spare = (stock_kcal(&stores, goods) - cp.keep_days * need).max(0.0);
                let pay = (cp.ask_days * need).min(spare);
                (pay > 1.0).then(|| {
                    let worth = cp.w_demand_loss * pay / need_of(self, officer_hh);
                    (pay, worth - cp.w_objection * objection - cp.w_seen * risk)
                })
            });
        let mut scores = vec![0.0, report];
        if let Some((_, a)) = ask {
            scores.push(a);
        }
        let key = [ctx.seed, PURPOSE_SIGHT, officer.get(), u64::from(incident)];
        let u = Rng64::from_key(&key).next_f64();
        let pick = crate::demography::pick_softmax(&scores, u);
        let lost = self
            .order
            .amounts
            .iter()
            .rev()
            .find(|a| a.lost && a.household == victim && a.incident == incident)
            .map_or(0.0, |a| f64::from(a.kcal));
        let mut paid = 0.0;
        let kept = match pick {
            Some(1) => match law {
                Some((law, _)) => {
                    self.officer_case(ctx, incident, victim, officer, taker, lost, law);
                    Kept::Reported
                }
                None => Kept::Told,
            },
            Some(2) => {
                let (pay, _) = ask.unwrap_or_default();
                let need = need_of(self, taker_hh);
                let points = cp.w_comply_known - cp.w_comply_cost * pay / need;
                let key = [ctx.seed, PURPOSE_PAY, officer.get(), u64::from(incident)];
                let pays = Rng64::from_key(&key).next_f64() < crate::crime::logistic(points);
                let moved = pays
                    && self.household(taker_hh).is_some_and(|x| {
                        let stores = stores_now(x, now, params, goods);
                        !super::crime::take_goods(&stores, goods, f64::INFINITY, pay).is_empty()
                    });
                if moved {
                    let stores = self
                        .household(taker_hh)
                        .map(|x| stores_now(x, now, params, goods))
                        .unwrap_or_default();
                    let legs: Vec<Leg> =
                        super::crime::take_goods(&stores, goods, f64::INFINITY, pay)
                            .into_iter()
                            .map(|(good, amount)| Leg {
                                from: taker_hh,
                                to: officer_hh,
                                good,
                                amount,
                            })
                            .collect();
                    let kcal: f64 = legs
                        .iter()
                        .map(|l| l.amount * goods[l.good].kcal_per_kg)
                        .sum();
                    if self.transfer(now, params, goods, &legs, Channel::Bribe) {
                        paid = kcal;
                    }
                }
                if paid > 0.0 {
                    // Nothing came of it: being found out seems less likely (09-09 §1.5).
                    if let Some(&ph) = self.index.get(&officer)
                        && let Some(p) = self.people.get_mut(ph)
                    {
                        p.risk_seen = crate::crime::updated_risk(p.risk_seen, 0.0, cp.risk_alpha);
                    }
                    Kept::Paid
                } else {
                    if let Some((law, _)) = law {
                        self.officer_case(ctx, incident, victim, officer, taker, lost, law);
                    }
                    Kept::Refused
                }
            }
            _ => Kept::LookedAway,
        };
        self.order.sightings.push(Sighting {
            incident,
            officer,
            day,
            kept,
            kcal: paid as f32,
            points: (report as f32, ask.map(|a| a.1 as f32)),
        });
        kept
    }

    /// `officer` brings what they saw before the gathering, and the watch counts it.
    #[allow(clippy::too_many_arguments)]
    fn officer_case(
        &mut self,
        ctx: &mut Ctx,
        incident: u32,
        victim: PermanentId,
        officer: PermanentId,
        taker: PermanentId,
        lost: f64,
        law: PermanentId,
    ) {
        if self.order.cases.iter().any(|c| c.incident == incident) {
            return;
        }
        self.bring_case(ctx, incident, victim, officer, taker, lost, law);
        let settlement = self.household(victim).and_then(|x| x.settlement);
        if let Some(pi) = settlement.and_then(|s| self.polity_of(s))
            && let Some(l) = self.polities[pi].laws.iter_mut().find(|l| {
                l.status == LawStatus::InForce
                    && l.kind == crate::polity::PolicyKind::KeepWatch
                    && l.holder == Some(officer)
            })
        {
            l.watch.cases += 1;
        }
    }
}
