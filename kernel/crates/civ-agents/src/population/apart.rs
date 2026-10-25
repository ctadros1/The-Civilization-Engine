//! Keeping the sick apart (M10c slice EE, first part; ADR-0035 §7; brief m10-services §2.4;
//! research 05-05 §1.6-1.7): where someone of a household of a polity died within the year of a
//! sickness that passes from the sick to those near them, at home or in company, anyone may
//! propose that a household with someone ill keeps to its home, and others keep from it, until
//! some days after the last of it is well. It is kept by choice, as a curfew is: one who knows the
//! law weighs going out of a home kept apart, or tending at one, by what keeping the law weighs
//! with them. Against a sickness carried by water it costs work and does little: a legible
//! mistake.

use super::*;
use crate::params::DiseaseRoute;
use crate::polity::{KeepApartDef, LawStatus, PolicyKind};
use crate::sickness::Outcome;

/// A law keeping the sick apart where someone lives, as weighed by them now.
#[derive(Clone, Copy, Debug)]
pub(super) struct ApartNow {
    /// The law, by polity and index.
    pub pi: usize,
    pub li: usize,
    /// What keeping it weighs with them, where they know of it.
    pub points: Option<f64>,
    /// Its terms.
    pub def: KeepApartDef,
    /// Whether their own household is kept apart today.
    pub home: bool,
}

impl Population {
    /// The deaths within the year of people of `households`, of a sickness passing at home or in
    /// company, by household (M10c slice EE, first part), in household order.
    pub(super) fn contagion_deaths(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, usize)> {
        let day = ctx.now.day_index();
        let since = day - DAYS_PER_YEAR;
        let contact = |d: u16| {
            ctx.catalog.diseases.get(usize::from(d)).is_some_and(|x| {
                x.passes_by(DiseaseRoute::Household) || x.passes_by(DiseaseRoute::Company)
            })
        };
        let mut out: Vec<(PermanentId, usize)> = Vec::new();
        // An infection that ended within the year began within some months before it.
        for e in self
            .sickness
            .episodes()
            .iter()
            .rev()
            .take_while(|e| e.infected >= since - 120)
        {
            let Some((d, Outcome::Died)) = e.ended else {
                continue;
            };
            let Some(h) = e.household.filter(|h| households.contains(h)) else {
                continue;
            };
            if d <= since || !contact(e.disease) {
                continue;
            }
            match out.iter_mut().find(|o| o.0 == h) {
                Some(o) => o.1 += 1,
                None => out.push((h, 1)),
            }
        }
        out.sort_unstable();
        out
    }

    /// What keeping the sick apart would bring each of `households` a year and cost it, kcal
    /// (M10c slice EE, first part): each death of its people within the year of a sickness
    /// passing at home or in company, weighed as one it believes avoided (`suspicion.w_death`
    /// days of work, at what an hour of its work brings in food, as a licence's); and, for the
    /// days its people lay ill within the year with `days_after` more, the share `cost_share` of
    /// a day's work of each grown member (of `adults`, by person and household), by the law's
    /// first template's terms.
    pub(super) fn keep_apart_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
    ) -> Vec<(PermanentId, f64, f64)> {
        let Some(def) = ctx.catalog.policies.iter().find_map(|d| d.keep_apart) else {
            return Vec::new();
        };
        if self.sickness.episodes().is_empty() {
            return Vec::new();
        }
        let params = ctx.params;
        let day = ctx.now.day_index();
        let since = day - DAYS_PER_YEAR;
        let day_h = params.farm.work_hours_per_day;
        let deaths = self.contagion_deaths(ctx, households);
        households
            .iter()
            .filter_map(|&h| {
                let x = self.household(h)?;
                let died = deaths.iter().find(|d| d.0 == h).map_or(0, |d| d.1);
                // The days within the year it had someone ill.
                let mut ill: Vec<i64> = Vec::new();
                for &m in &x.members {
                    for e in self.sickness.of(m) {
                        if !e.symptomatic() {
                            continue;
                        }
                        let (from, until) = (e.course.ill_from.max(since), e.course.ill_until);
                        ill.extend((from..until.min(day)).filter(|&d| d > since));
                    }
                }
                ill.sort_unstable();
                ill.dedup();
                if died == 0 && ill.is_empty() {
                    return None;
                }
                let kcal = self.hour_kcal(ctx, x);
                let apart_days = if ill.is_empty() {
                    0.0
                } else {
                    ill.len() as f64 + f64::from(def.days_after)
                };
                let grown = adults.iter().filter(|a| a.1 == h).count() as f64;
                let gain = died as f64 * params.suspicion.w_death * day_h * kcal;
                let cost = apart_days * f64::from(def.cost_share) * grown * day_h * kcal;
                Some((h, gain, cost))
            })
            .collect()
    }

    /// Whether household `x` is kept apart on `day` under terms `def`: someone of it is ill of
    /// anything, or was within `days_after` days.
    pub(super) fn kept_apart(&self, x: &Household, def: &KeepApartDef, day: i64) -> bool {
        if self.sickness.quiet() && self.sickness.episodes().is_empty() {
            return false;
        }
        let after = i64::from(def.days_after);
        x.members.iter().any(|&m| {
            self.sickness.of(m).any(|e| {
                e.symptomatic() && e.course.ill_from <= day && day < e.course.ill_until + after
            })
        })
    }

    /// The law keeping the sick apart where household `hh` lives, as person `person` weighs it
    /// now, if one is in force (or struck down and still kept by them).
    pub(super) fn keep_apart_now(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
    ) -> Option<ApartNow> {
        let pi = self.polity_of(hh.settlement?)?;
        let laws = &self.polities[pi].laws;
        let li = laws.iter().position(|l| {
            l.kind == PolicyKind::KeepApart
                && (l.status == LawStatus::InForce || self.keeps_struck(ctx, person, l))
        })?;
        let law = &laws[li];
        let def = ctx
            .catalog
            .policies
            .get(usize::from(law.policy))
            .and_then(|d| d.keep_apart)?;
        let points = law.knows(person).then(|| {
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == person)
                .map_or(0.0, |r| r.stance.sign());
            let regard =
                self.ties
                    .regard(person, law.sponsor, ctx.now.day_index(), &ctx.params.ties);
            let norm = self.norm_points(ctx, person);
            crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity).max(0.0)
        });
        let home = self.kept_apart(hh, &def, ctx.now.day_index());
        Some(ApartNow {
            pi,
            li,
            points,
            def,
            home,
        })
    }

    /// Whether household `household` is kept apart today where a law keeping the sick apart
    /// is in force where it lives: for tests.
    #[doc(hidden)]
    pub fn kept_apart_for_tests(&self, ctx: &Ctx, household: PermanentId) -> bool {
        let Some(x) = self.household(household) else {
            return false;
        };
        let Some(def) = ctx.catalog.policies.iter().find_map(|d| d.keep_apart) else {
            return false;
        };
        self.kept_apart(x, &def, ctx.now.day_index())
    }
}
