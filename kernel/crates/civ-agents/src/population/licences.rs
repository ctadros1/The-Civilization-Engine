//! Licences to practise (M10c slice EC, third part; ADR-0035 §6: a licence to practise is a
//! privilege, ADR-0026 §3; brief m10-services §3.4; research 05-05 §2: licensed midwives): where
//! someone of a household of a polity died within the year in the care of a carer it paid, not
//! kin, who knew no treatment of their sickness, and someone grown of the settlement knows one,
//! anyone may propose that only those who know that treatment be paid to tend the sick. Under it,
//! a household calls to tend its ill, for a fee, only households where someone knows it; kin, who
//! ask none, are called as before.

use super::*;
use crate::polity::{LawStatus, PolicyKind};
use crate::sickness::{Episode, UnknowingCare};

impl Population {
    /// Episode `e`'s bearer died in the care of `carer`, of another household than `home` whose
    /// people are `members` (M10c slice EC, third part): where the carer is kin of none of them,
    /// and so was paid, and knew no treatment of the disease, the household remembers it a year.
    pub(super) fn note_unknowing(
        &mut self,
        ctx: &Ctx,
        e: &Episode,
        carer: PermanentId,
        home: PermanentId,
        members: &[PermanentId],
    ) {
        let day = ctx.now.day_index();
        self.sickness
            .unknowing
            .retain(|u| u.day > day - DAYS_PER_YEAR);
        let Some(def) = ctx.catalog.diseases.get(usize::from(e.disease)) else {
            return;
        };
        let Some(c) = self.person(carer) else {
            return;
        };
        if def.treatments.iter().any(|&(t, _)| c.knows(t)) {
            return;
        }
        self.refresh_kin();
        let kin = std::iter::once(e.person)
            .chain(members.iter().copied())
            .any(|m| self.close_kin(m).contains(&carer));
        if kin {
            return;
        }
        self.sickness.unknowing.push(UnknowingCare {
            household: home,
            disease: e.disease,
            day,
        });
    }

    /// The technique a licence in force where household `x` lives asks of those it pays to tend
    /// its ill, if any.
    fn licence_where(&self, ctx: &Ctx, x: &Household) -> Option<usize> {
        let pi = self.polity_of(x.settlement?)?;
        self.polities[pi]
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::Licence)
            .find_map(|l| {
                ctx.catalog
                    .policies
                    .get(usize::from(l.policy))?
                    .licence
                    .as_ref()
                    .map(|d| d.technique)
            })
    }

    /// Whether household `x` may pay household `y` to tend its ill: no licence where it lives,
    /// or someone of `y` knows the technique it asks.
    pub(super) fn licensed(&self, ctx: &Ctx, x: &Household, y: &Household) -> bool {
        let Some(t) = self.licence_where(ctx, x) else {
            return true;
        };
        y.members
            .iter()
            .any(|&m| self.person(m).is_some_and(|p| p.knows(t)))
    }

    /// The treatments someone of `adults` knows of the diseases someone of `households` died of
    /// within the year in the care of one paid who knew none (M10c slice EC, third part), in
    /// index order: what a licence could ask.
    pub(super) fn unknowing_cures(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
    ) -> Vec<usize> {
        if self.sickness.unknowing.is_empty() {
            return Vec::new();
        }
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let mut out: Vec<usize> = self
            .sickness
            .unknowing
            .iter()
            .filter(|u| u.day > since && households.contains(&u.household))
            .filter_map(|u| ctx.catalog.diseases.get(usize::from(u.disease)))
            .flat_map(|d| d.treatments.iter().map(|&(t, _)| t))
            .filter(|&t| {
                adults
                    .iter()
                    .any(|&(c, _)| self.person(c).is_some_and(|p| p.knows(t)))
            })
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// What each licence template would bring each of `households`, kcal, by template index and
    /// then household (M10c slice EC, third part): for each death of its people within the year
    /// in the care of one paid who knew no treatment, where the licence's technique treats that
    /// disease, the death as a household weighs one it believes avoided (`suspicion.w_death`
    /// days of work, at what an hour of its work brings in food).
    pub(super) fn licence_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(u16, Vec<(PermanentId, f64)>)> {
        if self.sickness.unknowing.is_empty() {
            return Vec::new();
        }
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let params = ctx.params;
        let day_h = params.farm.work_hours_per_day;
        ctx.catalog
            .policies
            .iter()
            .enumerate()
            .filter_map(|(k, def)| Some((k as u16, def.licence.as_ref()?.technique)))
            .map(|(k, t)| {
                let of = households
                    .iter()
                    .filter_map(|&h| {
                        let x = self.household(h)?;
                        let deaths = self
                            .sickness
                            .unknowing
                            .iter()
                            .filter(|u| u.household == h && u.day > since)
                            .filter(|u| {
                                ctx.catalog
                                    .diseases
                                    .get(usize::from(u.disease))
                                    .is_some_and(|d| d.treatments.iter().any(|&(c, _)| c == t))
                            })
                            .count();
                        (deaths > 0).then(|| {
                            let kcal = self.hour_kcal(ctx, x);
                            (h, deaths as f64 * params.suspicion.w_death * day_h * kcal)
                        })
                    })
                    .collect();
                (k, of)
            })
            .collect()
    }

    /// The treatments a licence in polity `pi` could ask now: for tests.
    #[doc(hidden)]
    pub fn unknowing_care_for_tests(&self, ctx: &Ctx, pi: usize) -> Vec<usize> {
        let settlement = self.polities[pi].settlement;
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement))
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let adults = self.members_of(settlement, ctx.now, ctx.params);
        self.unknowing_cures(ctx, &households, &adults)
    }

    /// Whether household `x` may pay household `y` to tend its ill now: for tests.
    #[doc(hidden)]
    pub fn licensed_for_tests(&self, ctx: &Ctx, x: PermanentId, y: PermanentId) -> bool {
        match (self.household(x), self.household(y)) {
            (Some(x), Some(y)) => self.licensed(ctx, x, y),
            _ => false,
        }
    }
}
