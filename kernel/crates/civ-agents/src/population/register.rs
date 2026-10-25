//! The register of the sick (M10c slice EE, second part; ADR-0035 §7; brief m10-services §2.4;
//! research 05-04 §5.4, 05-05 §5.5): where its people had sickness at home lately and someone
//! grown of the settlement can write, anyone may propose that the one the gathering names keep a
//! register of the sick, an office whose holder must know how to write. Each midnight they enter
//! in it the households whose sickness they heard of, once a sickness spell, and the deaths of
//! disease of the polity's households that day, which all hear of. Their tallies of where sickness
//! comes from run over it: every household it holds counts as sick, and they know where each draws,
//! having asked when they wrote it down. So recognition can come earlier than anyone's own, or be
//! wrong. A household weighs the office by the share of its deaths of disease within the year it
//! believes a register would have spared.

use super::*;
use crate::sickness::{Outcome, RegisterEntry};
use crate::uses::Place;
use crate::word::ClaimKind;

/// An entry for sickness heard of.
const HEARD: u8 = 0;
/// An entry for a death of disease.
const DIED: u8 = 1;

impl Population {
    /// The day's entries in each register of the sick kept (see the module documentation), and
    /// the year's oldest let go.
    pub(super) fn register_day(&mut self, ctx: &Ctx, day: i64) {
        self.sickness
            .register
            .retain(|r| r.day > day - DAYS_PER_YEAR);
        let offices: Vec<(PermanentId, PermanentId, PermanentId)> = self
            .polities
            .iter()
            .filter_map(|p| Some((p.id, p.settlement, p.registrar()?.0)))
            .collect();
        if offices.is_empty() {
            return;
        }
        let spell = i64::from(ctx.params.word.sickness_spell_days);
        for (polity, settlement, holder) in offices {
            let of_it = |h: PermanentId| {
                self.household(h)
                    .is_some_and(|x| x.settlement == Some(settlement))
            };
            let mut heard: Vec<PermanentId> = self
                .word
                .heard_by(holder)
                .iter()
                .filter_map(|h| self.word.claim(h.claim))
                .filter(|c| c.kind == ClaimKind::Sickness)
                .filter_map(|c| c.subject)
                .filter(|&h| of_it(h))
                .filter(|&h| {
                    !self.sickness.register.iter().any(|r| {
                        r.polity == polity
                            && r.household == h
                            && r.kind == HEARD
                            && day - r.day <= spell
                    })
                })
                .collect();
            heard.sort_unstable();
            heard.dedup();
            // Deaths of disease since the last midnight, which all hear of.
            let mut died: Vec<(i64, PermanentId)> = self
                .sickness
                .episodes()
                .iter()
                .rev()
                .take_while(|e| e.infected >= day - DAYS_PER_YEAR)
                .filter_map(|e| match e.ended {
                    Some((d, Outcome::Died)) if d >= day - 1 => Some((d, e.household?)),
                    _ => None,
                })
                .filter(|&(_, h)| of_it(h))
                .filter(|&(d, h)| {
                    !self.sickness.register.iter().any(|r| {
                        r.polity == polity && r.household == h && r.kind == DIED && r.day == d
                    })
                })
                .collect();
            died.sort_unstable();
            for household in heard {
                self.sickness.register.push(RegisterEntry {
                    polity,
                    day,
                    household,
                    kind: HEARD,
                });
            }
            for (d, household) in died {
                self.sickness.register.push(RegisterEntry {
                    polity,
                    day: d,
                    household,
                    kind: DIED,
                });
            }
        }
    }

    /// The households the register `person` keeps holds within the sickness spell as of `day`,
    /// in id order: none where they keep none.
    pub(super) fn registered_sick(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        day: i64,
    ) -> Vec<PermanentId> {
        if self.sickness.register.is_empty() {
            return Vec::new();
        }
        let Some(polity) = self
            .polities
            .iter()
            .find(|p| p.registrar().is_some_and(|r| r.0 == person))
            .map(|p| p.id)
        else {
            return Vec::new();
        };
        let spell = i64::from(ctx.params.word.sickness_spell_days);
        let mut out: Vec<PermanentId> = self
            .sickness
            .register
            .iter()
            .filter(|r| r.polity == polity && day - r.day <= spell)
            .map(|r| r.household)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The households person `person`'s tally counts sick from the register they keep, and the
    /// sources it now knows them to draw at: for tests.
    #[doc(hidden)]
    pub fn register_tally_for_tests(
        &self,
        ctx: &Ctx,
        person: PermanentId,
    ) -> (Vec<PermanentId>, Vec<(Place, PermanentId)>) {
        let day = ctx.now.day_index();
        let sick = self.registered_sick(ctx, person, day);
        let draws = self.registered_draws(ctx, &sick, day);
        (sick, draws)
    }

    /// Where each of `households` draws most, as `day`'s tally reckons it, for a registrar who
    /// asked when they wrote it down.
    pub(super) fn registered_draws(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
        day: i64,
    ) -> Vec<(Place, PermanentId)> {
        let sp = &ctx.params.suspicion;
        let half = ctx.params.places.use_half_life_days;
        households
            .iter()
            .filter_map(|&h| Some((self.usual_source(h, day, half, sp.draws_min_days)?, h)))
            .collect()
    }

    /// What a register of the sick would spare each of `households`, kcal a year (M10c slice EE,
    /// second part): its people's deaths of disease within the year, at the template's share it
    /// believes a register would have avoided, each weighed as a licence's death avoided
    /// (`suspicion.w_death` days of work at what an hour of its work brings in food).
    pub(super) fn register_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64)> {
        let Some(def) = ctx.catalog.policies.iter().find_map(|d| d.register) else {
            return Vec::new();
        };
        if self.sickness.episodes().is_empty() {
            return Vec::new();
        }
        let params = ctx.params;
        let day = ctx.now.day_index();
        let since = day - DAYS_PER_YEAR;
        let day_h = params.farm.work_hours_per_day;
        let mut deaths: Vec<(PermanentId, usize)> = Vec::new();
        for e in self
            .sickness
            .episodes()
            .iter()
            .rev()
            .take_while(|e| e.infected >= since - 120)
        {
            let (Some((d, Outcome::Died)), Some(h)) = (e.ended, e.household) else {
                continue;
            };
            if d <= since || !households.contains(&h) {
                continue;
            }
            match deaths.iter_mut().find(|x| x.0 == h) {
                Some(x) => x.1 += 1,
                None => deaths.push((h, 1)),
            }
        }
        deaths.sort_unstable();
        deaths
            .into_iter()
            .filter_map(|(h, n)| {
                let x = self.household(h)?;
                let kcal = self.hour_kcal(ctx, x);
                Some((
                    h,
                    n as f64
                        * f64::from(def.avoided_share)
                        * params.suspicion.w_death
                        * day_h
                        * kcal,
                ))
            })
            .collect()
    }
}
