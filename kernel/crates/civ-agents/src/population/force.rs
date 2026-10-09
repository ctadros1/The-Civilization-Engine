//! Force (M4c slice AI, step four; ADR-0017 §3-4; research 09-06 §1.3, 04-10 §1.7-1.8, 06-10 §4):
//! one who keeps the watch may go to take what a gathering's finding owed and the household
//! refused, the execution of an order being a stage of its own (09-06 §1.3). The household's
//! adults each meet them as they choose: let it be taken, stand in the way, or strike (04-10
//! §1.8: violence is a person's choice, not a crowd's property); where any stood in the way, the
//! watcher chooses to take it by force or to turn back. Every blow is a record: who struck whom,
//! in which encounter, under which finding, and what it did (06-10 §4.C). Nothing here is drawn
//! but how hard a blow falls; nothing makes an encounter happen but people's choices.

use super::*;
use crate::Cause;
use crate::crime::{Encounter, Harm, Met, Owed, Standing};
use crate::ties::Act;
use crate::word::{Blamed, Grieved, Wrong};

/// Purpose tag for a person's reluctance to strike, keyed by person (a threshold, rebuilt on load:
/// ADR-0017 §3).
pub const PURPOSE_STRIKE: u64 = 0x7374_7269_6b65_7468; // "striketh"
/// Purpose tag for how hard a blow falls: the days it keeps the one struck from work.
pub const PURPOSE_BLOW: u64 = 0x626c_6f77_6461_7973; // "blowdays"
/// Purpose tag for whether a blow kills.
pub const PURPOSE_KILL: u64 = 0x626c_6f77_6b69_6c6c; // "blowkill"

impl Population {
    /// `watcher`, who keeps the watch at `settlement`, of threshold `threshold`, weighs going to
    /// take what a refused finding owed from a household there (M4c slice AI, step four): of the
    /// findings refused there and not tried within the faction review's span, the one most worth
    /// it, at `w_collect` and the norms they hold (the gathering binds: what it found should be
    /// done), with their regard for the household owed over their regard for the household
    /// owing. They go when it is worth more than their threshold.
    pub(super) fn consider_collection(
        &mut self,
        ctx: &mut Ctx,
        watcher: PermanentId,
        settlement: PermanentId,
        threshold: f64,
    ) {
        let (now, day, params) = (ctx.now, ctx.now.day_index(), ctx.params);
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        let Some(watch) = self.polities[pi]
            .watchers()
            .find(|w| w.0 == watcher)
            .map(|w| w.1.id)
        else {
            return;
        };
        let own = self.person(watcher).map(|p| p.household);
        let lately = day - i64::from(params.faction.review_days.max(1));
        let refused: Vec<usize> = (0..self.order.obligations.len())
            .filter(|&k| {
                let ob = &self.order.obligations[k];
                ob.standing == Standing::Refused
                    && ob.case.is_some()
                    && ob.left_kcal() > 1.0
                    && Some(ob.debtor) != own
                    && self
                        .household(ob.debtor)
                        .is_some_and(|x| x.settlement == Some(settlement))
                    && !self
                        .order
                        .encounters
                        .iter()
                        .any(|e| e.obligation == ob.id && e.day > lately)
            })
            .collect();
        if refused.is_empty() {
            return;
        }
        self.ensure_norm_state(ctx, watcher);
        let duty = params.crime.w_collect + self.norm_points(ctx, watcher).max(0.0);
        let tp = &params.ties;
        let regard_of = |hh: PermanentId| {
            self.elder_of(hh, now, params)
                .map_or(0.0, |e| self.ties.regard(watcher, e, day, tp))
        };
        let Some((worth, k)) = refused
            .iter()
            .map(|&k| {
                let ob = &self.order.obligations[k];
                let owed = match ob.kind {
                    Owed::Fine => 0.0,
                    _ => regard_of(ob.beneficiary),
                };
                (duty + owed - regard_of(ob.debtor), k)
            })
            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
        else {
            return;
        };
        if worth <= threshold {
            return;
        }
        self.encounter(ctx, watcher, watch, k, worth);
    }

    /// `officer` of the watch named by law `watch` comes to take what obligation `k` owes, at
    /// worth `worth` to them (M4c slice AI, step four). Each adult of the household meets them:
    /// letting it be taken is worth the norms they hold and their regard for the officer; resisting,
    /// how strongly their household refused it and their grievances against the watch and the
    /// gathering; one whose will to resist passes their own reluctance to strike strikes, one who
    /// resists short of it stands in the way. Where any resisted, the officer takes it by force if
    /// it is still worth it less `w_harm` for each who did; then each who struck strikes the
    /// officer and the officer strikes back. What is taken goes where the finding said; every blow
    /// is recorded with how long it keeps the one struck from work, or that it killed them.
    fn encounter(
        &mut self,
        ctx: &mut Ctx,
        officer: PermanentId,
        watch: PermanentId,
        k: usize,
        worth: f64,
    ) {
        let (now, day, params, goods) =
            (ctx.now, ctx.now.day_index(), ctx.params, &ctx.catalog.goods);
        let cp = &params.crime;
        let ob = self.order.obligations[k];
        let Some(debtor) = self.household(ob.debtor) else {
            return;
        };
        let settlement = debtor.settlement;
        let place = debtor.home;
        let Some(pi) = settlement.and_then(|s| self.polity_of(s)) else {
            return;
        };
        let polity = self.polities[pi].id;
        let mut adults = self.grown_of(ob.debtor, now, params);
        adults.sort_unstable();
        // How strongly the household refused it: its answer's points toward refusing.
        let refused = ob.answer.map_or(0.0, |a| -f64::from(a.1)).max(0.0);
        let mut met = Vec::with_capacity(adults.len());
        for &p in &adults {
            self.ensure_norm_state(ctx, p);
            let comply = self.norm_points(ctx, p).max(0.0)
                + params.polity.w_regard * self.ties.regard(p, officer, day, &params.ties);
            let resist = refused
                + self.keenness(ctx, p, Blamed::Office(watch))
                + self.keenness(ctx, p, Blamed::Body(polity));
            let u = Rng64::from_key(&[ctx.seed, PURPOSE_STRIKE, p.get()]).next_f64();
            let [lo, hi] = cp.strike_threshold;
            let reluctance = lo + (hi - lo) * u;
            let answer = if comply >= resist {
                Met::Yielded
            } else if resist - comply > reluctance {
                Met::Struck
            } else {
                Met::Barred
            };
            met.push((p, answer));
        }
        let against = met.iter().filter(|m| m.1 != Met::Yielded).count();
        let forced = against > 0 && worth - cp.w_harm * against as f64 > 0.0;
        // What is taken: the household's food up to what is still owed, whatever it keeps back
        // (an execution takes what is held: 09-06 §5.4, never what is not).
        let mut taken = 0.0;
        if against == 0 || forced {
            let Some(debtor) = self.household(ob.debtor) else {
                return;
            };
            let stores = stores_now(debtor, now, params, goods);
            let legs: Vec<Leg> = crime::take_goods(&stores, goods, f64::INFINITY, ob.left_kcal())
                .into_iter()
                .map(|(good, amount)| Leg {
                    from: ob.debtor,
                    to: ob.beneficiary,
                    good,
                    amount,
                })
                .collect();
            let kcal: f64 = legs
                .iter()
                .map(|l| l.amount * goods[l.good].kcal_per_kg)
                .sum();
            let channel = match ob.kind {
                Owed::Demanded | Owed::Restitution => Channel::Restitution,
                Owed::Compensation => Channel::Compensation,
                Owed::Fine => Channel::Fine,
            };
            if !legs.is_empty() && self.transfer(now, params, goods, &legs, channel) {
                taken = kcal;
                let o = &mut self.order.obligations[k];
                o.paid_kcal += kcal as f32;
                if o.left_kcal() <= 1.0 {
                    o.standing = Standing::Met;
                }
            }
        }
        // The blows: each who struck strikes the officer, and the officer strikes back.
        let mut harms = Vec::new();
        if forced {
            for &(p, answer) in met.iter() {
                if answer != Met::Struck {
                    continue;
                }
                harms.push(blow(ctx.seed, p, officer, day, cp));
                harms.push(blow(ctx.seed, officer, p, day, cp));
            }
        }
        let record = Encounter {
            id: self.order.encounters.len() as u32 + 1,
            day,
            officer,
            household: ob.debtor,
            obligation: ob.id,
            owed_kcal: ob.left_kcal() as f32,
            met,
            forced,
            taken_kcal: taken as f32,
            harms: harms.clone(),
        };
        let owed = (ob.kind != Owed::Fine).then_some(ob.beneficiary);
        let name = self.encounter_words(&record, owed, ctx);
        self.order.encounters.push(record);
        // What it leaves (04-10 §1.7): the household holds what was taken by force, and a blow
        // to one of its own, against the watch; whoever is struck thinks the worse of the one
        // who struck them.
        let need = self.day_need(ob.debtor, params);
        for &p in &adults {
            if forced && taken > 0.0 {
                self.grieve(
                    ctx,
                    p,
                    Grieved::Treatment,
                    Blamed::Office(watch),
                    watch,
                    taken / need.max(1.0),
                    Wrong::Forced,
                );
            }
            for h in harms.iter().filter(|h| h.by == officer) {
                self.grieve(
                    ctx,
                    p,
                    Grieved::Treatment,
                    Blamed::Office(watch),
                    watch,
                    f64::from(h.days),
                    Wrong::Struck,
                );
            }
        }
        for h in &harms {
            self.note_tie(ctx, h.to, h.by, Act::Struck, 1.0, 0.0);
        }
        let elder = self.elder_of(ob.debtor, now, params);
        self.chronicle_push(
            now,
            ChronicleKind::Encounter,
            std::iter::once(officer).chain(elder).collect(),
            settlement,
            Some(place),
            taken,
            name,
        );
        for h in harms.iter().filter(|h| h.killed) {
            self.die(ctx, h.to, Cause::Violence);
        }
    }

    /// An encounter in plain words (research 06-10 §4.C-D: who did what to whom, under which
    /// decision; plain verbs, nothing justified): "Bo, who keeps the watch, came to the household
    /// of Ada to take what a finding of the gathering owed; Ada stood in the way and Cal met Bo
    /// with blows; Bo took 9 days' food for the household of Tam by force; Cal struck Bo, who was kept from work 5 days; Bo
    /// struck Cal, who died of it."
    fn encounter_words(
        &self,
        e: &Encounter,
        beneficiary: Option<PermanentId>,
        ctx: &Ctx,
    ) -> String {
        let (now, params) = (ctx.now, ctx.params);
        let (met, officer, household) = (&e.met, e.officer, e.household);
        let (taken_kcal, harms) = (f64::from(e.taken_kcal), &e.harms);
        let who = self.name_of(officer);
        let of = self.elder_of(household, now, params).map_or_else(
            || "a household".to_owned(),
            |e| format!("the household of {}", self.name_of(e)),
        );
        let owed = match beneficiary {
            None => "the common store".to_owned(),
            Some(b) => self.elder_of(b, now, params).map_or_else(
                || "a household".to_owned(),
                |e| format!("the household of {}", self.name_of(e)),
            ),
        };
        let names = |m: Met| -> Vec<String> {
            met.iter()
                .filter(|x| x.1 == m)
                .map(|x| self.name_of(x.0))
                .collect()
        };
        let join = |v: &[String]| match v.len() {
            0 => String::new(),
            1 => v[0].clone(),
            n => format!("{} and {}", v[..n - 1].join(", "), v[n - 1]),
        };
        let (barred, struck) = (names(Met::Barred), names(Met::Struck));
        let mut how = Vec::new();
        if !barred.is_empty() {
            how.push(format!("{} stood in the way", join(&barred)));
        }
        if !struck.is_empty() {
            how.push(format!("{} met {who} with blows", join(&struck)));
        }
        let days = taken_kcal / self.day_need(household, params).max(1.0);
        let food = match days {
            d if d < 1.5 => "a day's food".to_owned(),
            d => format!("{d:.0} days' food"),
        };
        let ended = if how.is_empty() {
            if taken_kcal > 0.0 {
                format!("they let {who} take {food} for {owed}")
            } else {
                "there was nothing to take".to_owned()
            }
        } else if taken_kcal > 0.0 || harms.iter().any(|h| h.by == officer) {
            format!(
                "{}; {who} took {food} for {owed} by force",
                how.join(" and ")
            )
        } else {
            format!("{}; {who} turned back", how.join(" and "))
        };
        let mut s = format!(
            "{who}, who keeps the watch, came to {of} to take what a finding of the gathering owed; \
             {ended}"
        );
        for h in harms {
            let after = if h.killed {
                "who died of it".to_owned()
            } else {
                format!("who was kept from work {} days", h.days)
            };
            s.push_str(&format!(
                "; {} struck {}, {after}",
                self.name_of(h.by),
                self.name_of(h.to)
            ));
        }
        s.push('.');
        s
    }
}

/// A blow `by` strikes `to` on `day`: the days it keeps them from work, drawn in the content's
/// range, and whether it kills, at the content's chance (keyed draws: how hard a blow falls is
/// all that is drawn).
fn blow(
    seed: u64,
    by: PermanentId,
    to: PermanentId,
    day: i64,
    cp: &crate::crime::CrimeParams,
) -> Harm {
    let key = |purpose: u64| [seed, purpose, by.get(), to.get(), day as u64];
    let [lo, hi] = cp.hurt_days;
    let span = f64::from(hi.saturating_sub(lo));
    let u = Rng64::from_key(&key(PURPOSE_BLOW)).next_f64();
    let days = lo + (u * (span + 1.0)).floor().min(span) as u16;
    let killed = Rng64::from_key(&key(PURPOSE_KILL)).next_f64() < cp.kill_share;
    Harm {
        by,
        to,
        days,
        killed,
    }
}
