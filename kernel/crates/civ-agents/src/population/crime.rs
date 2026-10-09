//! Taking at work (M4b slice AA, ADR-0015): whose store someone short of food would take from,
//! what happens when they get there, how word of it goes round, and what the household taken
//! from chooses and is owed. Every choice here is a person's or a household's, from what its
//! members believe; the incident itself is read only to settle what happens and who could see.

use super::*;
use crate::crime::{
    Belief, Choice, Incident, KnownAmount, Obligation, Outcome, Owed, Response, Source, Standing,
    TakingsKnown, comply_points, demand_points, logistic, report_points, updated_risk,
};
use crate::decide::TakeOption;
use crate::ties::Act;

/// Purpose tag for the draws when someone takes: whether a sleeper wakes.
pub const PURPOSE_TAKE: u64 = 0x7461_6b65_6f6e_6531; // "takeone1"
/// Purpose tag for a household's choice to demand back what was taken.
pub const PURPOSE_RESPOND: u64 = 0x7265_7370_6f6e_6431; // "respond1"
/// Purpose tag for a household's choice to meet a demand.
pub const PURPOSE_ANSWER: u64 = 0x616e_7377_6572_3031; // "answer01"
/// Purpose tag for a household's choice to bring a case after a demand failed.
pub const PURPOSE_ESCALATE: u64 = 0x6573_6361_6c61_7465; // "escalate"

/// Metres from its home within which a member counts as at home.
const AT_HOME_M: f32 = 2.0;

/// A household whose store a household's members could take from, whoever of them goes: the
/// walk, the food, and the most any member regards its elder. Derived; part of a household's
/// view of its options.
#[derive(Clone, Copy, Debug)]
pub(super) struct TakeTarget {
    household: PermanentId,
    walk_min: f64,
    at: (f32, f32),
    kcal: f64,
    regard: f64,
}

/// What a taker would carry off a store `stores`: up to `carry_kg` and `want_kcal` of its food,
/// the densest first so a load carries the most (seed kept back is left), as `(good, kg)`.
pub(super) fn take_goods(
    stores: &[f64],
    goods: &[GoodDef],
    carry_kg: f64,
    want_kcal: f64,
) -> Vec<(usize, f64)> {
    let mut order: Vec<usize> = (0..goods.len())
        .filter(|&i| {
            goods[i].purpose == GoodUse::Food && goods[i].kcal_per_kg > 0.0 && !goods[i].kept_back()
        })
        .collect();
    order.sort_by(|&a, &b| {
        goods[b]
            .kcal_per_kg
            .total_cmp(&goods[a].kcal_per_kg)
            .then(a.cmp(&b))
    });
    let (mut carry, mut want) = (carry_kg, want_kcal);
    let mut out = Vec::new();
    for i in order {
        if carry <= 1e-9 || want <= 1e-6 {
            break;
        }
        let kg = stores
            .get(i)
            .copied()
            .unwrap_or(0.0)
            .max(0.0)
            .min(carry)
            .min(want / goods[i].kcal_per_kg);
        if kg <= 1e-9 {
            continue;
        }
        out.push((i, kg));
        carry -= kg;
        want -= kg * goods[i].kcal_per_kg;
    }
    out
}

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

impl Population {
    /// The household of its settlement whose store household `hh` (holding `held` kcal of food,
    /// needing `kcal_day` a day) would take from: of those within reach, the ones from which a
    /// load would carry off the most food it wants; among them, the nearest once a walk is
    /// weighed against how well its members regard the one who stands for each, as asking weighs
    /// it (ADR-0015 §2: targets are found as `Ask` finds a giver).
    pub(super) fn take_target(
        &self,
        ctx: &Ctx,
        hh: &Household,
        field_key: PermanentId,
        kcal_day: f64,
        held: f64,
    ) -> Option<TakeTarget> {
        let settlement = hh.settlement?;
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let want = (params.household.food_target_days * kcal_day - held).max(0.0);
        if want <= 0.0 {
            return None;
        }
        let reach = &self.homes.get(&field_key)?.reach;
        let (tp, day) = (&params.ties, now.day_index());
        let mut found: Vec<TakeTarget> = Vec::new();
        for (_, x) in self.households.iter() {
            if x.id == hh.id || x.settlement != Some(settlement) || x.members.is_empty() {
                continue;
            }
            let stores = stores_now(x, now, params, goods);
            let kcal: f64 = take_goods(&stores, goods, params.household.carry_kg, want)
                .iter()
                .map(|&(g, kg)| kg * goods[g].kcal_per_kg)
                .sum();
            if kcal <= 0.0 {
                continue;
            }
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, x.home)) else {
                continue;
            };
            let regard = self.elder_of(x.id, now, params).map_or(0.0, |e| {
                hh.members
                    .iter()
                    .map(|&m| self.ties.regard(m, e, day, tp))
                    .fold(0.0, f64::max)
                    .clamp(0.0, 1.0)
            });
            found.push(TakeTarget {
                household: x.id,
                walk_min: f64::from(secs) / 60.0,
                at: x.home,
                kcal,
                regard,
            });
        }
        let most = found.iter().map(|t| t.kcal).fold(0.0, f64::max);
        found
            .into_iter()
            .filter(|t| t.kcal >= most - 1e-6)
            .map(|t| (t.walk_min + tp.ask_known_min * t.regard, t))
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.household.cmp(&b.1.household)))
            .map(|(_, t)| t)
    }

    /// Whether `p` turned back or fled from a store and is still waiting before trying again
    /// (research 12-04 §1.4: they change their timing).
    pub(super) fn turned_back_lately(p: &Person, now: SimTime) -> bool {
        p.guarded.is_some_and(|(_, until)| now < until)
    }

    /// What weighs on `p` taking from `target` (ADR-0015 §2), or why they would not: an
    /// objection at or above the content's filter rules it out before anything is weighed
    /// (research 04-09 §5.3).
    pub(super) fn take_option(
        p: &Person,
        target: Option<TakeTarget>,
        params: &PeopleParams,
    ) -> Result<TakeOption, Reason> {
        let cp = &params.crime;
        let objection = f64::from(p.objection);
        if objection >= cp.objection_filter {
            return Err(Reason::WouldNotTake);
        }
        let t = target.ok_or(Reason::NothingToTake)?;
        let daring = (-cp.w_risk_trait * f64::from(p.traits.risk)).exp();
        Ok(TakeOption {
            household: t.household,
            walk_min: t.walk_min,
            at: t.at,
            kcal: t.kcal,
            objection: cp.w_objection * objection,
            risk: cp.w_seen * f64::from(p.risk_seen).clamp(0.0, 1.0) * daring,
            regard: cp.w_regard * t.regard,
        })
    }

    /// Household `hh`'s food now, in days of its need, and the share of the other households of
    /// its settlement holding more, ties counting half (an incident's measure for the dashboard;
    /// nothing chooses by it). NaN where there is nothing to compare.
    fn food_place(
        &self,
        hh: PermanentId,
        now: SimTime,
        params: &PeopleParams,
        goods: &[GoodDef],
    ) -> (f32, f32) {
        let days = |x: &Household| {
            let need = x.members.len().max(1) as f64 * params.household.daily_kcal_per_person;
            stock_kcal(&stores_now(x, now, params, goods), goods) / need
        };
        let Some(x) = self.household(hh) else {
            return (f32::NAN, f32::NAN);
        };
        let own = days(x);
        let Some(settlement) = x.settlement else {
            return (own as f32, f32::NAN);
        };
        let (mut more, mut others) = (0.0, 0u32);
        for (_, y) in self.households.iter() {
            if y.id == hh || y.settlement != Some(settlement) || y.members.is_empty() {
                continue;
            }
            let d = days(y);
            others += 1;
            if d > own {
                more += 1.0;
            } else if d == own {
                more += 0.5;
            }
        }
        let richer = if others == 0 {
            f32::NAN
        } else {
            (more / f64::from(others)) as f32
        };
        (own as f32, richer)
    }

    /// `h` has come to household `victim`'s home to take from its store (ADR-0015 §2). Whether
    /// anyone is there is settled now: a member home and awake, and they turn back; a sleeper who
    /// wakes, and they flee seen; otherwise they carry off what they can, and anyone awake in
    /// sight sees them. The incident is recorded as it happened; the taker's sense of the chance
    /// of being seen moves with what happened to them (research 04-09 §5.4).
    pub(super) fn take_from(&mut self, ctx: &mut Ctx, h: Handle<Person>, victim: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.crime;
        let Some(p) = self.people.get(h) else {
            return;
        };
        let (taker, taker_hh) = (p.id, p.household);
        let Some(vh) = self.household(victim) else {
            return;
        };
        if victim == taker_hh {
            return;
        }
        let (home, members, settlement) = (vh.home, vh.members.clone(), vh.settlement);
        let (food_days, richer) = self.food_place(taker_hh, now, params, goods);
        let t = now.minutes() as f64;
        // A capable guardian is one of an age to stop them (research 04-09 §1.1: routine activity
        // turns on the absence of capable guardians); a younger child at home or about sees them,
        // and does not stop them.
        let capable = |q: &Person| q.age_years(now) >= cp.guardian_age;
        let mut awake_home = false;
        let mut sleepers: Vec<PermanentId> = Vec::new();
        for &m in &members {
            if let Some(q) = self.person(m)
                && q.trip.is_none()
                && distance(q.pos, home) <= AT_HOME_M
            {
                if q.asleep {
                    if capable(q) {
                        sleepers.push(m);
                    }
                } else if capable(q) {
                    awake_home = true;
                }
            }
        }
        sleepers.sort_unstable();
        // Those awake in sight of the store: the taker notices each grown enough to stop them with
        // the content's chance, and turns back if they notice anyone; those they miss, and the
        // young, see them (research 12-04 §1.4).
        let mut about: Vec<(PermanentId, bool)> = self
            .people
            .iter()
            .filter(|(_, q)| {
                q.id != taker && !q.asleep && distance(q.position_at(t), home) <= cp.sight_m as f32
            })
            .map(|(_, q)| (q.id, capable(q)))
            .collect();
        about.sort_unstable();
        let mut rng = Rng64::from_key(&[ctx.seed, PURPOSE_TAKE, taker.get(), now.minutes() as u64]);
        let noticed = about
            .iter()
            .any(|&(_, grown)| grown && rng.next_f64() < cp.notice_chance);
        let about: Vec<PermanentId> = about.into_iter().map(|(q, _)| q).collect();
        let mut seen_by: Vec<PermanentId> = Vec::new();
        let mut taken: Vec<(usize, f64)> = Vec::new();
        let outcome = if awake_home || noticed {
            Outcome::TurnedBack
        } else if let Some(&w) = sleepers.iter().find(|_| rng.next_f64() < cp.wake_chance) {
            seen_by.push(w);
            seen_by.extend(about.iter().copied());
            Outcome::Disturbed
        } else {
            let need = self
                .household(taker_hh)
                .map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person;
            let held = self.household(taker_hh).map_or(0.0, |x| {
                stock_kcal(&stores_now(x, now, params, goods), goods)
            });
            let want = (params.household.food_target_days * need - held).max(0.0);
            let stores = self
                .household(victim)
                .map(|x| stores_now(x, now, params, goods))
                .unwrap_or_default();
            taken = take_goods(&stores, goods, params.household.carry_kg, want);
            let legs: Vec<Leg> = taken
                .iter()
                .map(|&(good, amount)| Leg {
                    from: victim,
                    to: taker_hh,
                    good,
                    amount,
                })
                .collect();
            if legs.is_empty() || !self.transfer(now, params, goods, &legs, Channel::Take) {
                taken.clear();
                Outcome::TurnedBack
            } else {
                // Those about whom the taker did not notice see them at it.
                seen_by.extend(about.iter().copied());
                Outcome::Taken
            }
        };
        seen_by.sort_unstable();
        seen_by.dedup();
        let kcal = taken
            .iter()
            .map(|&(g, kg)| kg * goods[g].kcal_per_kg)
            .sum::<f64>()
            .max(0.0);
        let id = self.order.incidents.last().map_or(1, |i| i.id + 1);
        self.order.incidents.push(Incident {
            id,
            at: now,
            actor: taker,
            actor_household: taker_hh,
            target: victim,
            place: home,
            outcome,
            goods: taken.iter().map(|&(g, kg)| (g as u16, kg as f32)).collect(),
            kcal: kcal as f32,
            seen_by: seen_by.clone(),
            noticed: false,
            food_days,
            richer,
        });
        // The taker learns from their own attempt: turned back or seen is a sign takers are
        // seen; an unseen taking that they are not.
        let s = if outcome == Outcome::Taken && seen_by.is_empty() {
            0.0
        } else {
            1.0
        };
        // One who turned back or fled waits before weighing it again, for between half and one and
        // a half times the content's hours: they change their timing (12-04 §1.4).
        let wait_min = cp.retry_hours * 60.0 * (0.5 + rng.next_f64());
        if let Some(p) = self.people.get_mut(h) {
            p.risk_seen = updated_risk(p.risk_seen, s, cp.risk_alpha);
            if outcome != Outcome::Taken {
                p.guarded = Some((victim, now.plus_minutes(wait_min.round() as i64)));
            }
        }
        // The taker knows what they did, and their household what came home (M4b slice AB).
        let day = now.day_index();
        if outcome == Outcome::Taken {
            self.come_to_believe(
                ctx,
                Belief {
                    holder: taker,
                    incident: id,
                    taker: Some(taker),
                    source: Source::Did,
                    from: None,
                    origin: Some(taker),
                    day,
                },
            );
            self.order.amounts.push(KnownAmount {
                household: taker_hh,
                incident: id,
                kcal: kcal as f32,
                lost: false,
                day,
            });
        }
        // Those who saw know who it was.
        for &w in &seen_by {
            self.come_to_believe(
                ctx,
                Belief {
                    holder: w,
                    incident: id,
                    taker: Some(taker),
                    source: Source::Saw,
                    from: None,
                    origin: Some(w),
                    day,
                },
            );
        }
        // Told plainly in the chronicle when someone saw it (06-10 §4): who did what to whom.
        if !seen_by.is_empty() {
            let elder = self
                .elder_of(victim, now, params)
                .map_or_else(|| "a household".to_owned(), |e| self.name_of(e));
            let names: Vec<String> = seen_by.iter().map(|&w| self.name_of(w)).collect();
            let saw = match names.len() {
                1 => format!("{} saw it", names[0]),
                2 => format!("{} and {} saw it", names[0], names[1]),
                3 => format!("{}, {} and 1 other saw it", names[0], names[1]),
                n => format!("{}, {} and {} others saw it", names[0], names[1], n - 2),
            };
            let kg: f64 = taken.iter().map(|&(_, kg)| kg).sum();
            let what = taken
                .iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map_or("food", |&(g, _)| goods[g].name.as_str())
                .to_lowercase();
            let name = match outcome {
                Outcome::Taken => {
                    format!("took {kg:.0} kg of {what} from the household of {elder}; {saw}.")
                }
                _ => format!(
                    "tried to take food from the household of {elder} and fled when seen; {saw}."
                ),
            };
            let mut people = vec![taker];
            people.extend(seen_by.iter().copied());
            self.chronicle_push(
                now,
                ChronicleKind::Taking,
                people,
                settlement,
                Some(home),
                kg,
                name,
            );
        }
    }

    /// `belief` comes to its holder (ADR-0015 §1). A belief new to them moves their sense of the
    /// chance a taker runs of being seen (word of a taker is a sign they are; a loss nobody saw,
    /// that they are not) and, if they know who took, their tie to the taker.
    pub(super) fn come_to_believe(&mut self, ctx: &Ctx, belief: Belief) {
        let target = self.order.incident(belief.incident).map(|i| i.target);
        if !self.order.learn(belief) {
            return;
        }
        // What a taker knows of their own taking moves nothing.
        if belief.source == Source::Did {
            return;
        }
        let cp = &ctx.params.crime;
        let s = if belief.taker.is_some() { 1.0 } else { 0.0 };
        if let Some(&ph) = self.index.get(&belief.holder)
            && let Some(p) = self.people.get_mut(ph)
        {
            p.risk_seen = updated_risk(p.risk_seen, s, cp.risk_alpha_told);
        }
        // Nobody thinks the worse of one of their own household for it.
        let own = |q: PermanentId| self.person(q).map(|p| p.household);
        if let Some(taker) = belief.taker
            && taker != belief.holder
            && own(taker) != own(belief.holder)
        {
            let ours = self
                .person(belief.holder)
                .is_some_and(|p| Some(p.household) == target);
            let act = if ours {
                Act::TookFromUs
            } else {
                Act::TookFromOthers
            };
            self.note_tie(ctx, belief.holder, taker, act, 1.0, 0.0);
        }
    }

    /// `a` and `b` kept company at the hearth: each tells the other who they know took from
    /// whom, unless the taker is of their own household (research 12-04 §1.2: word of misconduct
    /// travels along ties, and kin shield their own).
    pub(crate) fn share_takings(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        if self.order.beliefs.is_empty() {
            return;
        }
        let day = ctx.now.day_index();
        for (teller, hearer) in [(a, b), (b, a)] {
            let Some(own) = self.person(teller).map(|p| p.household) else {
                continue;
            };
            let tells: Vec<Belief> = self
                .order
                .held_by(teller)
                .iter()
                .filter(|x| {
                    x.taker.is_some_and(|t| {
                        t != hearer && self.person(t).is_none_or(|p| p.household != own)
                    }) && !self.order.quiet(teller, x.incident)
                })
                .copied()
                .collect();
            for x in tells {
                if self
                    .order
                    .belief(hearer, x.incident)
                    .is_some_and(|y| y.taker.is_some())
                {
                    continue;
                }
                self.come_to_believe(
                    ctx,
                    Belief {
                        holder: hearer,
                        source: Source::Told,
                        from: Some(teller),
                        day,
                        ..x
                    },
                );
            }
        }
    }

    /// Whether household `giver` refuses `asker`'s ask: one of its members believes the asker
    /// took from it, or from a household whose elder they regard (ADR-0015 §3), within what they
    /// remember.
    pub(super) fn refuses(&self, ctx: &Ctx, giver: PermanentId, asker: PermanentId) -> bool {
        if self.order.beliefs.is_empty() {
            return false;
        }
        let (now, params) = (ctx.now, ctx.params);
        let cp = &params.crime;
        let day = now.day_index();
        let since = day - i64::from(cp.remember_days);
        let Some(x) = self.household(giver) else {
            return false;
        };
        x.members.iter().any(|&m| {
            self.order.believes_took(m, asker, since, &|target| {
                target == giver
                    || self.elder_of(target, now, params).is_some_and(|e| {
                        self.ties.regard(m, e, day, &params.ties) >= cp.refuse_regard
                    })
            })
        })
    }

    /// Takings' midnight (ADR-0015 §1, §3, §5): households find what was taken from them;
    /// witnesses tell those taken from, as their ties lead them; households share what their
    /// members know; a household that has learnt who took from it chooses to let it go or demand
    /// the food back; and what is owed is answered, paid, or falls due.
    pub(super) fn crime_day(&mut self, ctx: &mut Ctx) {
        if self.order.incidents.is_empty() {
            return;
        }
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        // Losses found: a taking is found at the first midnight after it, so only the last day's
        // incidents can be unfound.
        let recent = self
            .order
            .incidents
            .partition_point(|i| i.at.day_index() < day - 1);
        for k in recent..self.order.incidents.len() {
            let inc = &self.order.incidents[k];
            if inc.outcome != Outcome::Taken || inc.noticed {
                continue;
            }
            let (id, target, kcal) = (inc.id, inc.target, inc.kcal);
            self.order.incidents[k].noticed = true;
            // The household knows what it found missing from its store.
            if self.household(target).is_some() {
                self.order.amounts.push(KnownAmount {
                    household: target,
                    incident: id,
                    kcal,
                    lost: true,
                    day,
                });
            }
            let members = self
                .household(target)
                .map(|x| x.members.clone())
                .unwrap_or_default();
            for m in members {
                self.come_to_believe(
                    ctx,
                    Belief {
                        holder: m,
                        incident: id,
                        taker: None,
                        source: Source::Noticed,
                        from: None,
                        origin: None,
                        day,
                    },
                );
            }
        }
        // Witnesses tell those taken from, unless they are of the taker's household or regard the
        // taker more than the one who stands for the household taken from.
        let witnessed: Vec<Belief> = self
            .order
            .beliefs
            .iter()
            .filter(|b| b.source == Source::Saw)
            .copied()
            .collect();
        for w in witnessed {
            let Some(inc) = self.order.incident(w.incident) else {
                continue;
            };
            let (target, taker) = (inc.target, inc.actor);
            let Some(own) = self.person(w.holder).map(|p| p.household) else {
                continue;
            };
            let taker_hh = self.person(taker).map(|p| p.household);
            if own == target || Some(own) == taker_hh {
                continue;
            }
            // One who keeps the watch chooses, once, what to do with what they saw (M4b slice
            // AC): one who looked away or was paid to tells nobody; one who reported or told
            // tells those taken from, whatever their regard.
            let watcher = self.keeps_watch_over(w.holder, target)
                || self.order.sighting(w.holder, w.incident).is_some();
            if watcher {
                let kept = match self.order.sighting(w.holder, w.incident) {
                    Some(s) => s.kept,
                    None => self.officer_sees(ctx, w.holder, w.incident, target, taker),
                };
                if kept.quiet() {
                    continue;
                }
                let members = self
                    .household(target)
                    .map(|x| x.members.clone())
                    .unwrap_or_default();
                for m in members {
                    if self
                        .order
                        .belief(m, w.incident)
                        .is_some_and(|b| b.taker.is_some())
                    {
                        continue;
                    }
                    self.come_to_believe(
                        ctx,
                        Belief {
                            holder: m,
                            source: Source::Told,
                            from: Some(w.holder),
                            day,
                            ..w
                        },
                    );
                }
                continue;
            }
            let Some(elder) = self.elder_of(target, now, params) else {
                continue;
            };
            let tp = &params.ties;
            let for_taker = self.ties.regard(w.holder, taker, day, tp);
            let for_victim = self.ties.regard(w.holder, elder, day, tp);
            if for_taker > for_victim {
                continue;
            }
            let members = self
                .household(target)
                .map(|x| x.members.clone())
                .unwrap_or_default();
            for m in members {
                if self
                    .order
                    .belief(m, w.incident)
                    .is_some_and(|b| b.taker.is_some())
                {
                    continue;
                }
                self.come_to_believe(
                    ctx,
                    Belief {
                        holder: m,
                        source: Source::Told,
                        from: Some(w.holder),
                        day,
                        ..w
                    },
                );
            }
        }
        // Households share what their members know of who took.
        let mut homes: Vec<(PermanentId, Vec<PermanentId>)> = self
            .households
            .iter()
            .filter(|(_, x)| x.members.len() > 1)
            .map(|(_, x)| (x.id, x.members.clone()))
            .collect();
        homes.sort_unstable_by_key(|h| h.0);
        for (_, members) in homes {
            let known: Vec<Belief> = members
                .iter()
                .flat_map(|&m| self.order.held_by(m).iter().copied())
                .filter(|b| b.taker.is_some() && !self.order.quiet(b.holder, b.incident))
                .collect();
            for b in known {
                for &m in &members {
                    if m == b.holder
                        || self
                            .order
                            .belief(m, b.incident)
                            .is_some_and(|x| x.taker.is_some())
                    {
                        continue;
                    }
                    self.come_to_believe(
                        ctx,
                        Belief {
                            holder: m,
                            source: Source::Told,
                            from: Some(b.holder),
                            day,
                            ..b
                        },
                    );
                }
            }
        }
        self.respond(ctx);
        self.answer_obligations(ctx);
    }

    /// Each household that has learnt who took from it, and has not yet chosen, chooses (ADR-0015
    /// §3) from what it knows, never from the incident: the one who stands for it weighs the
    /// days of food it found missing against their regard for the taker, and lets it go, demands
    /// the food back or, under a law against taking they know of, brings a case before the
    /// gathering (M4b slice AB).
    fn respond(&mut self, ctx: &mut Ctx) {
        let (now, params) = (ctx.now, ctx.params);
        let cp = &params.crime;
        let day = now.day_index();
        // Only losses people still remember can come to be known; and each is answered once.
        let since = day - i64::from(cp.remember_days);
        let answered: std::collections::BTreeSet<u32> = self
            .order
            .responses
            .iter()
            .rev()
            .take_while(|r| r.day >= since)
            .map(|r| r.incident)
            .collect();
        let start = self.order.amounts.partition_point(|a| a.day < since);
        let mut due: Vec<(u32, PermanentId, PermanentId, f64)> = Vec::new();
        for a in &self.order.amounts[start..] {
            if !a.lost || answered.contains(&a.incident) {
                continue;
            }
            let Some(x) = self.household(a.household) else {
                continue;
            };
            let taker = x.members.iter().find_map(|&m| {
                self.order
                    .belief(m, a.incident)
                    .and_then(|b| b.taker)
                    .filter(|&t| t != m)
            });
            if let Some(taker) = taker {
                due.push((a.incident, a.household, taker, f64::from(a.kcal)));
            }
        }
        for (incident, household, taker, kcal) in due {
            let Some(by) = self.elder_of(household, now, params) else {
                continue;
            };
            let need = self
                .household(household)
                .map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person;
            let regard = self.ties.regard(by, taker, day, &params.ties);
            let points = demand_points(kcal / need, regard, cp);
            let brought = self.order.cases.iter().any(|c| c.incident == incident);
            let report = self
                .law_against_taking(ctx, household, by)
                .filter(|_| !brought)
                .map(|(law, sanction)| {
                    let members = self
                        .household(household)
                        .map(|x| x.members.clone())
                        .unwrap_or_default();
                    let accounts = self.order.leads(&members, incident, taker).len();
                    let r = report_points(
                        accounts,
                        kcal / need,
                        f64::from(sanction.compensation_days),
                        regard,
                        cp,
                    );
                    (law, r)
                });
            let key = [ctx.seed, PURPOSE_RESPOND, u64::from(incident), day as u64];
            let u = Rng64::from_key(&key).next_f64();
            let choice = match report {
                // Letting it go or a demand: the choice slice AA made.
                None => {
                    if u < logistic(points) {
                        Choice::Demand
                    } else {
                        Choice::LetGo
                    }
                }
                Some((_, r)) => match crate::demography::pick_softmax(&[0.0, points, r], u) {
                    Some(1) => Choice::Demand,
                    Some(2) => Choice::Report,
                    _ => Choice::LetGo,
                },
            };
            self.order.responses.push(Response {
                incident,
                household,
                by,
                day,
                choice,
                points: points as f32,
                report_points: report.map(|r| r.1 as f32),
            });
            let debtor = self.person(taker).map(|p| p.household);
            match (choice, debtor) {
                (Choice::Demand, Some(debtor)) if debtor != household => {
                    let id = self.order.obligations.last().map_or(1, |o| o.id + 1);
                    self.order.obligations.push(Obligation {
                        id,
                        incident,
                        kind: Owed::Demanded,
                        case: None,
                        debtor,
                        beneficiary: household,
                        kcal: kcal as f32,
                        paid_kcal: 0.0,
                        made: day,
                        due: day + i64::from(cp.due_days),
                        standing: Standing::Open,
                        answer: None,
                    });
                }
                (Choice::Report, Some(_)) => {
                    if let Some((law, _)) = report {
                        self.bring_case(ctx, incident, household, by, taker, kcal, law);
                    }
                }
                _ => {}
            }
        }
    }

    /// A household whose demand was refused, or not met in full when due, may bring a case under
    /// a law against taking its chooser knows of (research 09-07 §1.1: negotiation can fail and the
    /// dispute go on to a forum); `left` is what is still owed, kcal.
    fn weigh_case_after_demand(
        &mut self,
        ctx: &mut Ctx,
        incident: u32,
        household: PermanentId,
        taker: PermanentId,
        left: f64,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let cp = &params.crime;
        let day = now.day_index();
        let Some(by) = self.elder_of(household, now, params) else {
            return;
        };
        let Some((law, sanction)) = self.law_against_taking(ctx, household, by) else {
            return;
        };
        if self.order.cases.iter().any(|c| c.incident == incident) {
            return;
        }
        let Some(x) = self.household(household) else {
            return;
        };
        let need = x.members.len().max(1) as f64 * params.household.daily_kcal_per_person;
        let members = x.members.clone();
        let accounts = self.order.leads(&members, incident, taker).len();
        let regard = self.ties.regard(by, taker, day, &params.ties);
        let r = report_points(
            accounts,
            left / need,
            f64::from(sanction.compensation_days),
            regard,
            cp,
        );
        let key = [ctx.seed, PURPOSE_ESCALATE, u64::from(incident), day as u64];
        let report = Rng64::from_key(&key).next_f64() < logistic(r);
        self.order.responses.push(Response {
            incident,
            household,
            by,
            day,
            choice: if report {
                Choice::Report
            } else {
                Choice::LetGo
            },
            points: 0.0,
            report_points: Some(r as f32),
        });
        if report {
            self.bring_case(ctx, incident, household, by, taker, left, law);
        }
    }

    /// What is owed is answered, paid or falls due (ADR-0015 §5). A household answers a demand
    /// once, weighing how many households of its settlement believe the taker took and its regard
    /// for the household owed against what paying would cost it; what a gathering's finding
    /// imposed it answers once for the whole finding, with the custom that the gathering binds
    /// behind paying. One that means to pay gives food it can spare day by day until it is paid
    /// or due: restitution first, then compensation, then the fine. An award never creates goods:
    /// what a household cannot spare stays owed, and is an arrear when due. A demand that fails
    /// may go on to a case.
    fn answer_obligations(&mut self, ctx: &mut Ctx) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.crime;
        let day = now.day_index();
        let mut failed: Vec<(u32, PermanentId, PermanentId, f64)> = Vec::new();
        for k in 0..self.order.obligations.len() {
            let ob = self.order.obligations[k];
            if ob.standing != Standing::Open {
                continue;
            }
            let Some(taker) = self.order.incident(ob.incident).map(|i| i.actor) else {
                continue;
            };
            // The one owed: a household, or for a fine the polity.
            let owed_here = if ob.kind == Owed::Fine {
                self.polities.iter().any(|p| p.id == ob.beneficiary)
            } else {
                self.household(ob.beneficiary).is_some()
            };
            let Some(debtor) = self.household(ob.debtor).filter(|_| owed_here) else {
                self.order.obligations[k].standing = Standing::Lapsed;
                continue;
            };
            let settlement = debtor.settlement;
            let need = debtor.members.len().max(1) as f64 * params.household.daily_kcal_per_person;
            let owed_members = if ob.kind == Owed::Fine {
                Vec::new()
            } else {
                self.household(ob.beneficiary)
                    .map(|x| x.members.clone())
                    .unwrap_or_default()
            };
            let place = debtor.home;
            if ob.answer.is_none() {
                // A finding is answered once, whole.
                let given = ob.case.and_then(|c| {
                    self.order.obligations[..k]
                        .iter()
                        .find(|o| o.case == Some(c))
                        .and_then(|o| o.answer)
                });
                let answer = match given {
                    Some(a) => a,
                    None => {
                        let Some(by) = self.elder_of(ob.debtor, now, params) else {
                            continue;
                        };
                        let households: Vec<&Household> = self
                            .households
                            .iter()
                            .map(|(_, x)| x)
                            .filter(|x| x.settlement == settlement && !x.members.is_empty())
                            .collect();
                        let since = day - i64::from(cp.remember_days);
                        let knowing = households
                            .iter()
                            .filter(|x| {
                                x.members
                                    .iter()
                                    .any(|&m| self.order.believes_took(m, taker, since, &|_| true))
                            })
                            .count();
                        let known = knowing as f64 / households.len().max(1) as f64;
                        // Regard for the household owed: for a finding, the one taken from.
                        let owed_household = match ob.case.and_then(|c| self.order.case(c)) {
                            Some(c) => c.accuser,
                            None => ob.beneficiary,
                        };
                        let regard = self
                            .elder_of(owed_household, now, params)
                            .map_or(0.0, |e| self.ties.regard(by, e, day, &params.ties));
                        let left: f64 = match ob.case {
                            Some(c) => self.order.obligations[k..]
                                .iter()
                                .filter(|o| o.case == Some(c) && o.standing == Standing::Open)
                                .map(Obligation::left_kcal)
                                .sum(),
                            None => ob.left_kcal(),
                        };
                        let found = if ob.case.is_some() {
                            cp.w_comply_found
                        } else {
                            0.0
                        };
                        let points = comply_points(known, regard, left / need, cp) + found;
                        let key = [ctx.seed, PURPOSE_ANSWER, u64::from(ob.id), day as u64];
                        let pays = Rng64::from_key(&key).next_f64() < logistic(points);
                        (pays, points as f32)
                    }
                };
                self.order.obligations[k].answer = Some(answer);
                if !answer.0 {
                    self.order.obligations[k].standing = Standing::Refused;
                    // A finding refused is held against the household that refused it (M4c
                    // slice AE).
                    if ob.kind != Owed::Fine
                        && let Some(law) = ob.case.and_then(|c| self.order.case(c)).map(|c| c.law)
                    {
                        self.grieve_refusal(ctx, law, ob.debtor, ob.beneficiary, ob.left_kcal());
                    }
                    if ob.kind != Owed::Fine {
                        for &m in &owed_members {
                            self.note_tie(ctx, m, taker, Act::RefusedRestitution, 1.0, 0.0);
                        }
                    }
                    self.restitution_note(ctx, k, place);
                    if ob.kind == Owed::Demanded {
                        failed.push((ob.incident, ob.beneficiary, taker, ob.left_kcal()));
                    }
                    continue;
                }
            }
            // Meant to pay: what it can spare today.
            let Some(debtor) = self.household(ob.debtor) else {
                continue;
            };
            let stores = stores_now(debtor, now, params, goods);
            let spare = (stock_kcal(&stores, goods) - cp.keep_days * need).max(0.0);
            let give = spare.min(self.order.obligations[k].left_kcal());
            if give > 1.0 {
                let legs: Vec<Leg> = take_goods(&stores, goods, f64::INFINITY, give)
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
                    self.order.obligations[k].paid_kcal += kcal as f32;
                }
            }
            if self.order.obligations[k].left_kcal() <= 1.0 {
                self.order.obligations[k].standing = Standing::Met;
                if matches!(ob.kind, Owed::Demanded | Owed::Restitution) {
                    for &m in &owed_members {
                        self.note_tie(ctx, m, taker, Act::Restored, 1.0, 0.0);
                    }
                }
                self.restitution_note(ctx, k, place);
            } else if day >= ob.due {
                self.order.obligations[k].standing = Standing::Defaulted;
                self.restitution_note(ctx, k, place);
                if ob.kind == Owed::Demanded {
                    let left = self.order.obligations[k].left_kcal();
                    failed.push((ob.incident, ob.beneficiary, taker, left));
                }
            }
        }
        for (incident, household, taker, left) in failed {
            self.weigh_case_after_demand(ctx, incident, household, taker, left);
        }
    }

    /// The chronicle's note of how obligation `k` ended.
    fn restitution_note(&mut self, ctx: &Ctx, k: usize, place: (f32, f32)) {
        let ob = self.order.obligations[k];
        let Some(taker) = self.order.incident(ob.incident).map(|i| i.actor) else {
            return;
        };
        let (now, params) = (ctx.now, ctx.params);
        let of = |hh: PermanentId| {
            self.elder_of(hh, now, params).map_or_else(
                || "a household".to_owned(),
                |e| format!("the household of {}", self.name_of(e)),
            )
        };
        let debtor = of(ob.debtor);
        let who = self.name_of(taker);
        let left = 100.0 * ob.left_kcal() / f64::from(ob.kcal).max(1.0);
        let mut name = match ob.kind {
            Owed::Demanded | Owed::Restitution => {
                let owed = of(ob.beneficiary);
                let under = if ob.kind == Owed::Restitution {
                    ", as the gathering found"
                } else {
                    ""
                };
                match ob.standing {
                    Standing::Met => {
                        format!("{debtor} gave back the food {who} took from {owed}{under}.")
                    }
                    Standing::Refused => {
                        format!(
                            "{debtor} refused to give back the food {who} took from {owed}{under}."
                        )
                    }
                    _ => format!(
                        "{debtor} could not give back all the food {who} took from {owed} in time: \
                         {left:.0}\u{a0}% of it is still owed."
                    ),
                }
            }
            Owed::Compensation | Owed::Fine => {
                let what = if ob.kind == Owed::Fine {
                    "the fine to the common store".to_owned()
                } else {
                    format!("what it owed {} for {who}'s taking", of(ob.beneficiary))
                };
                match ob.standing {
                    Standing::Met => format!("{debtor} paid {what}."),
                    Standing::Refused => format!("{debtor} refused to pay {what}."),
                    _ => format!(
                        "{debtor} could not pay all of {what} in time: {left:.0}\u{a0}% of it is \
                         still owed."
                    ),
                }
            }
        };
        name = name[..1].to_uppercase() + &name[1..];
        let settlement = self.household(ob.debtor).and_then(|x| x.settlement);
        self.chronicle_push(
            now,
            ChronicleKind::Restitution,
            vec![taker],
            settlement,
            Some(place),
            f64::from(ob.standing.code()),
            name,
        );
    }

    /// Gives everyone without an objection to taking a founder's, by a draw keyed to them, and
    /// everyone without a sense of the chance of being seen the content's prior: for a save from
    /// before takings (ADR-0015 §2). Nobody else is touched.
    pub fn give_crime_state(&mut self, params: &crate::crime::CrimeParams, seed: u64) {
        use crate::demography::{Draw, life_rng};
        for (_, p) in self.people.iter_mut() {
            if !(0.0..=1.0).contains(&p.objection) {
                let mut rng = life_rng(seed, p.id, 0, Draw::Objection);
                p.objection = crate::crime::draw_objection(None, None, params, &mut rng);
            }
            if !(0.0..=1.0).contains(&p.risk_seen) {
                p.risk_seen = params.risk_prior as f32;
            }
        }
    }

    /// What household `h` knows of takings in the year to `now` (M4b slice AB): what it lost to
    /// takers a member knows of, and what its members took, with the mean chance its grown
    /// members believe a taker runs of being seen.
    pub(super) fn takings_known(
        &self,
        h: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> TakingsKnown {
        let mut out = TakingsKnown::default();
        let since = now.day_index() - 365;
        let Some(x) = self.household(h) else {
            return out;
        };
        for a in self
            .order
            .amounts
            .iter()
            .rev()
            .take_while(|a| a.day >= since)
            .filter(|a| a.household == h)
        {
            if a.lost {
                let known = x.members.iter().any(|&m| {
                    self.order
                        .belief(m, a.incident)
                        .is_some_and(|b| b.taker.is_some())
                });
                if known {
                    out.lost_kcal += f64::from(a.kcal);
                    out.lost += 1;
                } else {
                    out.unknown_kcal += f64::from(a.kcal);
                }
            } else {
                out.took_kcal += f64::from(a.kcal);
                out.took += 1;
            }
        }
        let grown: Vec<f64> = x
            .members
            .iter()
            .filter_map(|&m| self.person(m))
            .filter(|p| p.age_years(now) >= params.family.independent_age)
            .map(|p| f64::from(p.risk_seen))
            .collect();
        out.risk = if grown.is_empty() {
            params.crime.risk_prior
        } else {
            grown.iter().sum::<f64>() / grown.len() as f64
        };
        out
    }

    /// Lets go of beliefs held by people no longer here, and of beliefs older than people keep
    /// them (on the first of the month, with ties).
    pub(super) fn prune_beliefs(&mut self, now: SimTime, params: &PeopleParams) {
        let since = now.day_index() - i64::from(params.crime.remember_days);
        let index = &self.index;
        self.order
            .beliefs
            .retain(|b| b.day >= since && index.contains_key(&b.holder));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good(name: &str, kcal_per_kg: f64, kept_back: bool) -> GoodDef {
        GoodDef {
            id: name.into(),
            name: name.into(),
            purpose: GoodUse::Food,
            kcal_per_kg,
            half_life_days: 0.0,
            sheltered_half_life_days: 0.0,
            eaten: crate::params::Eaten::Raw,
            shared: false,
            reserve_for: kept_back.then_some(0),
            tool: None,
            timber: None,
            store: None,
        }
    }

    #[test]
    fn a_taker_carries_off_the_densest_food_and_leaves_the_seed() {
        let goods = vec![
            good("grain", 3400.0, false),
            good("seed", 3400.0, true),
            good("berries", 600.0, false),
        ];
        let stores = [10.0, 50.0, 30.0];
        // A load of 20 kg: all the grain, then berries; the seed is never touched.
        let got = take_goods(&stores, &goods, 20.0, 1e9);
        assert_eq!(got, vec![(0, 10.0), (2, 10.0)]);
        // Only what the household wants.
        let got = take_goods(&stores, &goods, 20.0, 6800.0);
        assert_eq!(got, vec![(0, 2.0)]);
    }
}
