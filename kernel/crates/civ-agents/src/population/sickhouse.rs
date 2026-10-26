//! The sickhouse at work (M10c slice EE, fourth part; ADR-0035 §7; brief m10-services §2.4;
//! research 05-05 §1.6): once a polity's sickhouse stands finished, each midnight a severe case of
//! its households, ill today, whom nobody tended yesterday, is admitted while beds are left: one
//! for each [`BED_M2`] of its floor. Admitted, they lie there, not at home: those who shed a
//! sickness passing at home put its household hazard on the others lying there and on yesterday's
//! carers there, not on their household, and their care comes from those who come to tend there,
//! not from home. Anyone of the polity may tend there, close kin of someone lying there as they
//! would at home, others at half the weight (a design prior: a duty to the sick of the polity,
//! less than to one's own), the store paying their household for the hours where a law pays common
//! work. Those no longer ill, or gone, leave at midnight.

use super::*;
use crate::decide::TendFacts;
use crate::sickness::Admitted;

/// The floor a bed takes, with room beside it to tend, square metres (a design prior).
pub const BED_M2: f64 = 4.5;

/// The weight one not close kin of anyone lying in a sickhouse gives tending there, of what they
/// would give their own (a design prior).
const STRANGER_SHARE: f64 = 0.5;

impl Population {
    /// The beds of the sickhouse `building` stands for, by its program's floor.
    fn beds_of(ctx: &Ctx, building: PermanentId) -> usize {
        let Some(b) = ctx.land.buildings.iter().find(|b| b.id == building) else {
            return 0;
        };
        ctx.catalog
            .building_index(&b.spec.program)
            .and_then(|p| ctx.catalog.buildings.get(p))
            .and_then(|d| d.shapes.first())
            .map_or(0, |c| (c.floor_m2 / BED_M2).floor() as usize)
    }

    /// The midnight's admissions and discharges (see the module documentation).
    pub(super) fn admit_day(&mut self, ctx: &Ctx, day: i64) {
        // Those no longer ill, or gone, leave.
        if !self.sickness.admitted.is_empty() {
            let admitted = std::mem::take(&mut self.sickness.admitted);
            self.sickness.admitted = admitted
                .into_iter()
                .filter(|a| {
                    self.person(a.person).is_some()
                        && !self.sickness.ill_with(a.person, day).is_empty()
                })
                .collect();
        }
        let houses: Vec<(PermanentId, PermanentId)> = self
            .polities
            .iter()
            .filter_map(|p| Some((p.settlement, p.sickhouse?)))
            .collect();
        if houses.is_empty() {
            return;
        }
        for (settlement, building) in houses {
            let beds = Self::beds_of(ctx, building);
            let mut lying = self
                .sickness
                .admitted
                .iter()
                .filter(|a| a.building == building)
                .count();
            if lying >= beds {
                continue;
            }
            let mut admit: Vec<PermanentId> = Vec::new();
            for &i in self.sickness.running() {
                if lying >= beds {
                    break;
                }
                let Some(e) = self.sickness.get(i) else {
                    continue;
                };
                if !(e.course.severe && e.ill_on(day) && e.care.day != day - 1) {
                    continue;
                }
                let of_it = self
                    .person(e.person)
                    .and_then(|p| self.household(p.household))
                    .is_some_and(|x| x.settlement == Some(settlement));
                if !of_it
                    || admit.contains(&e.person)
                    || self.sickness.admitted_to(e.person).is_some()
                {
                    continue;
                }
                admit.push(e.person);
                lying += 1;
            }
            for person in admit {
                self.sickness.admitted.push(Admitted {
                    person,
                    building,
                    day,
                });
            }
        }
    }

    /// The hazards of the day at each sickhouse (see the module documentation): for each
    /// disease passing at home that someone lying there sheds, each other lying there and each of
    /// yesterday's carers there takes the household hazard for each who sheds it, from the one
    /// who does (the first, by episode order).
    pub(super) fn sickhouse_hazards(
        &self,
        ctx: &Ctx,
        day: i64,
        staffed: &[(PermanentId, PermanentId)],
    ) -> Vec<(PermanentId, u16, f64, PermanentId)> {
        if self.sickness.admitted.is_empty() {
            return Vec::new();
        }
        let mut shed: BTreeMap<(PermanentId, u16), (u32, PermanentId)> = BTreeMap::new();
        for &i in self.sickness.running() {
            let Some(e) = self.sickness.get(i) else {
                continue;
            };
            let Some(building) = self.sickness.admitted_to(e.person) else {
                continue;
            };
            let household_route = ctx
                .catalog
                .diseases
                .get(usize::from(e.disease))
                .is_some_and(|d| d.passes_by(crate::params::DiseaseRoute::Household));
            if household_route && e.shedding(day) {
                let s = shed.entry((building, e.disease)).or_insert((0, e.person));
                s.0 += 1;
            }
        }
        let mut out = Vec::new();
        for ((building, disease), (n, from)) in shed {
            let Some(def) = ctx.catalog.diseases.get(usize::from(disease)) else {
                continue;
            };
            let hazard = def.household_hazard.max(0.0) * f64::from(n);
            let lying = self
                .sickness
                .admitted
                .iter()
                .filter(|a| a.building == building)
                .map(|a| a.person);
            let carers = staffed.iter().filter(|s| s.1 == building).map(|s| s.0);
            let mut near: Vec<PermanentId> = lying.chain(carers).collect();
            near.sort_unstable();
            near.dedup();
            for m in near {
                out.push((m, disease, hazard, from));
            }
        }
        out
    }

    /// The sickhouse person `who` of household `x` could tend at (M10c slice EE, fourth part):
    /// their polity's, where care is owed those lying there; close kin of one of them at its whole
    /// weight, others at [`STRANGER_SHARE`] of it.
    pub(super) fn sickhouse_tend(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<TendFacts> {
        if self.sickness.admitted.is_empty() {
            return None;
        }
        let building = self.polities[self.polity_of(x.settlement?)?].sickhouse?;
        let lying: Vec<PermanentId> = self
            .sickness
            .admitted
            .iter()
            .filter(|a| a.building == building && a.person != who)
            .map(|a| a.person)
            .collect();
        if lying.is_empty() || self.sickness.admitted_to(who).is_some() {
            return None;
        }
        let owed = self.care_owed_at(ctx, &lying);
        if owed <= 1e-9 {
            return None;
        }
        let kin = self.close_kin(who);
        let share = if lying.iter().any(|m| kin.contains(m)) {
            1.0
        } else {
            STRANGER_SHARE
        };
        let at = ctx
            .land
            .buildings
            .iter()
            .find(|b| b.id == building)
            .map(|b| build::centre_m(&b.spec))?;
        let reach = &self.homes.get(&field_key)?.reach;
        let secs = reach.seconds_to(cell_of(ctx.map, at))?;
        Some(TendFacts {
            owed: owed * share,
            household: None,
            at,
            walk_min: f64::from(secs) / 60.0,
            sickhouse: Some(building),
        })
    }

    /// Person `who` tended those lying in sickhouse `building` for `minutes` at activity `def`
    /// (M10c slice EE, fourth part): the hours are shared among them as at home, the carer
    /// practises each treatment they know, is among the day's carers there, and the store pays
    /// their household where a law pays common work.
    pub(super) fn tend_at_sickhouse(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        building: PermanentId,
        def: u16,
        minutes: u32,
    ) {
        let day = ctx.now.day_index();
        let Some(carer) = self.person(who).cloned() else {
            return;
        };
        let ill: Vec<usize> = self
            .sickness
            .admitted
            .iter()
            .filter(|a| a.building == building && a.person != who)
            .flat_map(|a| self.sickness.ill_with(a.person, day))
            .collect();
        if ill.is_empty() {
            return;
        }
        let hours = f64::from(minutes) / 60.0;
        let mut given = Vec::new();
        let mut treatments: Vec<usize> = Vec::new();
        for &i in &ill {
            let Some(d) = self
                .sickness
                .get(i)
                .and_then(|e| ctx.catalog.diseases.get(usize::from(e.disease)))
            else {
                continue;
            };
            let rr = d
                .treatments
                .iter()
                .filter(|(t, _)| carer.knows(*t))
                .map(|(_, rr)| *rr)
                .fold(1.0, f64::min);
            given.push((i, rr));
            treatments.extend(d.treatments.iter().map(|(t, _)| *t));
        }
        for (i, rr) in given {
            self.sickness
                .tend(i, day, who, hours / ill.len() as f64, rr);
        }
        self.sickness.tended.insert(who, day);
        if !self.staffed.contains(&(who, building)) {
            self.staffed.push((who, building));
        }
        treatments.sort_unstable();
        treatments.dedup();
        for t in treatments {
            self.practise(ctx, who, t, def, Target::Building(building), hours);
        }
        let polity = self
            .household(carer.household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
            .map(|pi| self.polities[pi].id);
        if let Some(polity) = polity {
            self.pay_for_work(ctx, polity, carer.household, hours);
        }
    }

    /// Who lies in sickhouses now, and the beds of the sickhouse of polity `pi`: for tests.
    #[doc(hidden)]
    pub fn sickhouse_for_tests(&self, ctx: &Ctx, pi: usize) -> (Vec<PermanentId>, usize) {
        let lying = self.sickness.admitted.iter().map(|a| a.person).collect();
        let beds = self.polities[pi]
            .sickhouse
            .map_or(0, |b| Self::beds_of(ctx, b));
        (lying, beds)
    }
}
