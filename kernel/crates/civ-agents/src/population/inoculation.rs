//! Inoculation (M10c slice ED, second part; ADR-0035 §4; brief m10-services §2.3; research 05-05
//! §2.1): a disease that may be given on purpose names the technique that gives it, and how much
//! milder a course so given is (variolation: Jurin's 2.1% died after inoculation against 16.6% of
//! natural smallpox). While it runs anywhere, a household that believes it near (someone its
//! members have ties or kin with died of it within the sickness spell: the cause of a death is
//! known to those close to the dead) and weighs the inoculated course's deaths below the chance
//! it believes its people would take it, with members it does not protect, calls someone it knows
//! who knows the technique, kin asking nothing and others the call's fee. They come and inoculate
//! those members at a session at its home, each taking a mild course of it that sheds and can pass
//! as any.
//!
//! Vaccination (M10c slice ED, third part; research 05-05 §2.1, §4: Jenner, 1796): a disease that
//! may be prevented so names the technique, the good it is given from and a dose of it. A
//! household that believes the disease near calls one who knows vaccination before one who knows
//! inoculation, as it risks nobody, and wants it even where it would not risk inoculation. Their
//! household makes the matter while such a call stands at its door and it holds fewer doses than
//! the calls want; they come once it holds enough, and each member vaccinated takes a dose.
//! Vaccination turns away a share of the disease's hazards, whole for some years and then less.

use super::*;
use crate::decide::{CallFacts, TendFacts};
use crate::params::{InoculationDef, VaccinationDef};
use crate::sickness::{Acquired, SickDraw, Vaccinated, sickness_rng};

/// How long a call to inoculate stands unanswered, days (a design prior: a fortnight).
const INOCULATION_CALL_DAYS: i64 = 14;

/// A disease that may be given on purpose, or prevented by vaccination, and how.
#[derive(Clone, Copy, Debug)]
struct Giving {
    disease: u16,
    inoculation: Option<InoculationDef>,
    vaccination: Option<VaccinationDef>,
}

impl Giving {
    /// The inoculation a household weighs worth it: its course's deaths below the chance it
    /// believes its people would take the disease.
    fn inoculation_ok(&self) -> Option<InoculationDef> {
        self.inoculation.filter(|i| i.deaths < i.believed_take)
    }

    /// The techniques that give it, as a household would have them: vaccination first, as it
    /// risks nobody, then inoculation where worth it.
    fn techniques(&self) -> Vec<usize> {
        self.vaccination
            .map(|v| v.technique)
            .into_iter()
            .chain(self.inoculation_ok().map(|i| i.technique))
            .collect()
    }
}

impl Population {
    /// The diseases that run now and may be given on purpose or vaccinated against, with how (by
    /// index).
    fn inoculable(&self, ctx: &Ctx) -> Vec<Giving> {
        let mut out: Vec<Giving> = Vec::new();
        for &i in self.sickness.running() {
            let Some(e) = self.sickness.get(i) else {
                continue;
            };
            let Some(d) = ctx.catalog.diseases.get(usize::from(e.disease)) else {
                continue;
            };
            if d.inoculation.is_none() && d.vaccination.is_none() {
                continue;
            }
            if !out.iter().any(|o| o.disease == e.disease) {
                out.push(Giving {
                    disease: e.disease,
                    inoculation: d.inoculation,
                    vaccination: d.vaccination,
                });
            }
        }
        out.sort_unstable_by_key(|o| o.disease);
        out
    }

    /// The doses of vaccination `v`'s matter household `x` holds now.
    fn doses_held(&self, ctx: &Ctx, x: &Household, v: &VaccinationDef) -> usize {
        let goods = &ctx.catalog.goods;
        let held = stores_now(x, ctx.now, ctx.params, goods)
            .get(v.good)
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        (held / v.dose_kg.max(1e-12) + 1e-6).floor() as usize
    }

    /// The doses of good `good` household `household` would make for the calls to vaccinate
    /// standing at its door, with a dose's kilograms (M10c slice ED, third part): `None` where
    /// `good` vaccinates against nothing.
    pub(super) fn vaccine_wanted(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        good: usize,
    ) -> Option<(f64, usize)> {
        let v = ctx
            .catalog
            .diseases
            .iter()
            .find_map(|d| d.vaccination.filter(|v| v.good == good))?;
        let day = ctx.now.day_index();
        let needed = self
            .sickness
            .inoculation_calls
            .iter()
            .filter(|c| c.carer == household)
            .filter_map(|c| self.household(c.caller))
            .filter_map(|y| {
                let (g, n) = self.inoculation_wanted(ctx, y, day)?;
                g.vaccination.is_some_and(|w| w.good == good).then_some(n)
            })
            .sum();
        Some((v.dose_kg, needed))
    }

    /// Whether household `x` believes `disease` near on `day`: someone close kin of, or tied to,
    /// one of its members died of it within the sickness spell.
    fn believes_near(&self, ctx: &Ctx, x: &Household, disease: u16, day: i64) -> bool {
        let spell = i64::from(ctx.params.word.sickness_spell_days);
        // An infection that ended in the spell began within some months of it.
        let since = day - spell - 120;
        let dead: Vec<PermanentId> = self
            .sickness
            .episodes()
            .iter()
            .rev()
            .take_while(|e| e.infected >= since)
            .filter(|e| e.disease == disease)
            .filter(|e| {
                matches!(e.ended, Some((d, crate::sickness::Outcome::Died)) if day - d <= spell)
            })
            .map(|e| e.person)
            .collect();
        if dead.is_empty() {
            return false;
        }
        x.members.iter().any(|&m| {
            self.close_kin(m).iter().any(|k| dead.contains(k))
                || self.ties.of(m).iter().any(|t| dead.contains(&t.to))
        })
    }

    /// The members of household `x` to inoculate against `disease` on `day`: not protected by it
    /// (taking it now, protected by one lived through, or by a vaccination still whole) and not
    /// ill.
    fn to_inoculate(&self, x: &Household, disease: u16, day: i64) -> Vec<PermanentId> {
        x.members
            .iter()
            .copied()
            .filter(|&m| {
                !self.sickness.protected(m, disease, day)
                    && !self.sickness.vaccinated_whole(m, disease, day)
                    && !self.sickness.ill(m, day)
            })
            .collect()
    }

    /// The disease household `x` would have its people inoculated or vaccinated against on `day`,
    /// if any, and how many of them: one it believes near, that may be vaccinated against or
    /// whose inoculated course's deaths it weighs below the chance it believes they would take
    /// it, with members to protect.
    fn inoculation_wanted(&self, ctx: &Ctx, x: &Household, day: i64) -> Option<(Giving, usize)> {
        self.inoculable(ctx).into_iter().find_map(|g| {
            if g.techniques().is_empty() || !self.believes_near(ctx, x, g.disease, day) {
                return None;
            }
            let n = self.to_inoculate(x, g.disease, day).len();
            (n > 0).then_some((g, n))
        })
    }

    /// Whom person `who` of household `x` would call to inoculate its people (M10c slice ED,
    /// second part): where it wants them inoculated and has no such call standing, the household
    /// of kin, asking nothing, then of someone its members have ties with, for the call's fee,
    /// where someone able knows the technique; the nearest, as for a carer. One who knows
    /// vaccination is called before one who knows only inoculation (third part).
    pub(super) fn inoculation_call_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<CallFacts> {
        if self
            .sickness
            .inoculation_calls
            .iter()
            .any(|c| c.caller == x.id)
        {
            return None;
        }
        let day = ctx.now.day_index();
        let (g, n) = self.inoculation_wanted(ctx, x, day)?;
        let knows = |h: &PermanentId, t: usize| {
            self.household(*h).is_some_and(|y| {
                y.members.iter().any(|&m| {
                    !self.sickness.ill(m, day) && self.person(m).is_some_and(|p| p.knows(t))
                })
            })
        };
        let mut kin_all: Vec<PermanentId> = x
            .members
            .iter()
            .flat_map(|&m| self.close_kin(m))
            .filter_map(|k| self.person(k).map(|q| q.household))
            .filter(|h| *h != x.id)
            .collect();
        kin_all.sort_unstable();
        kin_all.dedup();
        let mut tied_all = self.tie_households(who, x);
        tied_all.retain(|h| *h != x.id);
        let any = |_: &Household| true;
        g.techniques().into_iter().find_map(|t| {
            let kin: Vec<PermanentId> = kin_all.iter().copied().filter(|h| knows(h, t)).collect();
            let tied: Vec<PermanentId> = tied_all.iter().copied().filter(|h| knows(h, t)).collect();
            self.choose_carer(ctx, x, field_key, &kin, &tied, &any, n as f64, false)
        })
    }

    /// Someone of household `caller` left a call to inoculate its people at household `carer`'s
    /// door, where it still wants them inoculated (M10c slice ED, second part): kin asking
    /// nothing, others the call's fee. Returns whether it was left.
    pub(super) fn call_inoculator(
        &mut self,
        ctx: &Ctx,
        caller: PermanentId,
        carer: PermanentId,
    ) -> bool {
        let day = ctx.now.day_index();
        let (Some(x), Some(y)) = (self.household(caller), self.household(carer)) else {
            return false;
        };
        if self
            .sickness
            .inoculation_calls
            .iter()
            .any(|c| c.caller == caller)
            || self.inoculation_wanted(ctx, x, day).is_none()
        {
            return false;
        }
        let kin = x
            .members
            .iter()
            .any(|&m| self.close_kin(m).iter().any(|k| y.members.contains(k)));
        let fee_kcal = if kin {
            0.0
        } else {
            self.call_fee(ctx, x) as f32
        };
        self.sickness
            .inoculation_calls
            .push(crate::sickness::CareCall {
                caller,
                carer,
                fee_kcal,
                day,
                paid: -1,
                birth: false,
            });
        true
    }

    /// The household person `who` of household `x` would go to inoculate (M10c slice ED, second
    /// part): one that called theirs, where they know the technique of a disease its people want
    /// inoculation against, with as many to inoculate as `owed`; the nearest of the most. One who
    /// knows vaccination goes once their household holds a dose for each to vaccinate there
    /// (third part).
    pub(super) fn inoculation_tend(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<TendFacts> {
        if self.sickness.inoculation_calls.is_empty() {
            return None;
        }
        let day = ctx.now.day_index();
        let p = self.person(who)?;
        let reach = &self.homes.get(&field_key)?.reach;
        let mut best: Option<TendFacts> = None;
        for c in self
            .sickness
            .inoculation_calls
            .iter()
            .filter(|c| c.carer == x.id)
        {
            let Some(y) = self.household(c.caller) else {
                continue;
            };
            let Some((g, n)) = self.inoculation_wanted(ctx, y, day) else {
                continue;
            };
            match g.vaccination.filter(|v| p.knows(v.technique)) {
                // The matter is made first.
                Some(v) if self.doses_held(ctx, x, &v) < n => continue,
                Some(_) => {}
                None if !g.inoculation_ok().is_some_and(|i| p.knows(i.technique)) => continue,
                None => {}
            }
            let owed = n as f64;
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, y.home)) else {
                continue;
            };
            let walk_min = f64::from(secs) / 60.0;
            if best.is_none_or(|b| owed > b.owed || (owed == b.owed && walk_min < b.walk_min)) {
                best = Some(TendFacts {
                    owed,
                    household: Some(y.id),
                    at: y.home,
                    walk_min,
                });
            }
        }
        best
    }

    /// Person `who` came to household `household` at activity `def` for `minutes` (M10c slice
    /// ED, second part): where it called theirs to inoculate its people and they know the
    /// technique, each of its members to inoculate takes a mild course of the disease from them,
    /// the call's fee is paid and the call answered. Returns whether they inoculated. One who
    /// knows vaccination vaccinates each with a dose of their household's matter while it lasts,
    /// and inoculates the rest where they know that and it is worth it (third part).
    pub(super) fn inoculate_at(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        def: u16,
        minutes: u32,
    ) -> bool {
        let day = ctx.now.day_index();
        let Some(own) = self.person(who).map(|p| p.household) else {
            return false;
        };
        let Some(i) = self
            .sickness
            .inoculation_calls
            .iter()
            .position(|c| c.caller == household && c.carer == own)
        else {
            return false;
        };
        let Some(x) = self.household(household) else {
            return false;
        };
        let Some((g, _)) = self.inoculation_wanted(ctx, x, day) else {
            return false;
        };
        let d = g.disease;
        let Some(p) = self.person(who) else {
            return false;
        };
        let vaccine = g.vaccination.filter(|v| p.knows(v.technique));
        let inoc = g.inoculation_ok().filter(|i| p.knows(i.technique));
        let doses = match (vaccine, self.household(own)) {
            (Some(v), Some(y)) => self.doses_held(ctx, y, &v),
            _ => 0,
        };
        if doses == 0 && inoc.is_none() {
            return false;
        }
        let members = self.to_inoculate(x, d, day);
        let call = self.sickness.inoculation_calls.remove(i);
        let (mut vaccinated, mut inoculated) = (0_usize, 0_usize);
        for m in members.into_iter().filter(|&m| m != who) {
            if let Some(v) = vaccine
                && vaccinated < doses
            {
                self.vaccinate(ctx, m, d, day, &v);
                vaccinated += 1;
            } else if inoc.is_some() {
                self.infect(ctx, m, d, day, Acquired::Inoculated { by: who });
                inoculated += 1;
            }
        }
        if let Some(v) = vaccine
            && vaccinated > 0
        {
            self.use_matter(ctx, own, v.good, vaccinated as f64 * v.dose_kg);
        }
        // The session's hours are practice of each, by the share of those it was given to.
        let hours = f64::from(minutes) / 60.0;
        let given = (vaccinated + inoculated).max(1) as f64;
        if let Some(v) = vaccine
            && vaccinated > 0
        {
            let share = vaccinated as f64 / given;
            self.practise(ctx, who, v.technique, def, Target::Home, hours * share);
        }
        if let Some(t) = inoc.map(|i| i.technique)
            && (inoculated > 0 || vaccinated == 0)
        {
            let share = if vaccinated == 0 {
                1.0
            } else {
                inoculated as f64 / given
            };
            self.practise(ctx, who, t, def, Target::Home, hours * share);
        }
        let fee = f64::from(call.fee_kcal);
        if fee > 0.0 {
            self.pay_food(ctx, household, own, fee);
        }
        true
    }

    /// Person `m` is vaccinated against `disease` on `day` by `v`: whole for a number of years
    /// drawn evenly between its two (M10c slice ED, third part).
    fn vaccinate(&mut self, ctx: &Ctx, m: PermanentId, disease: u16, day: i64, v: &VaccinationDef) {
        let u = sickness_rng(ctx.seed, m, day, disease, SickDraw::Vaccinated).next_f64();
        let [lo, hi] = v.full_years;
        let years = lo + (hi - lo).max(0.0) * u;
        self.sickness.vaccinated.push(Vaccinated {
            person: m,
            disease,
            day,
            full_until: day + (years * 365.25).round() as i64,
        });
    }

    /// Household `household` uses `kg` of good `good` from its stores.
    fn use_matter(&mut self, ctx: &Ctx, household: PermanentId, good: usize, kg: f64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        if let Some(x) = self.household_mut(household) {
            let members = x.members.len().max(1);
            x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            x.stores.resize(goods.len(), 0.0);
            let used = kg.min(x.stores[good].max(0.0));
            x.stores[good] -= used;
            x.flows.add(Flow::Used, good, used);
        }
        self.changed(household);
    }

    /// Calls to inoculate that are past their days, or whose household no longer wants it or is
    /// gone, lapse.
    pub(super) fn lapse_inoculation_calls(&mut self, ctx: &Ctx, day: i64) {
        if self.sickness.inoculation_calls.is_empty() {
            return;
        }
        let calls = std::mem::take(&mut self.sickness.inoculation_calls);
        let kept = calls
            .into_iter()
            .filter(|c| {
                day - c.day <= INOCULATION_CALL_DAYS
                    && self.household(c.carer).is_some()
                    && self
                        .household(c.caller)
                        .is_some_and(|x| self.inoculation_wanted(ctx, x, day).is_some())
            })
            .collect();
        self.sickness.inoculation_calls = kept;
    }

    /// Whom person `who` of household `h` would call to inoculate its people now: for tests.
    #[doc(hidden)]
    pub fn inoculation_call_for_tests(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        h: PermanentId,
    ) -> Option<PermanentId> {
        self.refresh_kin();
        let x = self.household(h)?;
        let key = x
            .settlement
            .filter(|s| self.homes.contains_key(s))
            .unwrap_or(x.id);
        self.inoculation_call_facts(ctx, who, x, key)
            .map(|c| c.carer)
    }
}
