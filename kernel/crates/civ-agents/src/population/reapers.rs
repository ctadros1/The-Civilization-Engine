//! Hired field work at the harvest, slice 1 (§8 17, fourth; research 08-10 §1.3: a household
//! allocates its hands across its own land and wage work by season): a household whose reaping
//! outruns its own hands before the standing crop loses a quarter of itself hires reapers as a
//! workshop hires hands. The offer is read from the field and the household as they are when
//! someone weighs it, so nothing is kept for it: the hours its own people cannot give before the
//! standing loss, less those hired reapers already on their way have taken, and a wage in the
//! crop's grain at the share of what an hour of the harvest's work brings that a workshop pays of
//! what an hour adds (`firm.wage_share`, the same custom), as far as the household holds the grain
//! to pay. Whoever weighs it sets the pay against what the time would do at home, as any wage.

use civ_core::{Handle, PermanentId};
use civ_land::{FieldStage, FieldTask};

use super::{Ctx, Population, peak_ratio, stores_now};
use crate::decide::MIN_TOOL;
use crate::ledger::{Channel, Leg};
use crate::make;
use crate::params::{Behavior, interpolate};
use crate::person::{Person, Step, Target};

/// Field work a household would hire, reaping or threshing: the field it is for, where it is
/// done, the hours wanted and the pay for an hour of a capable adult's work, kilograms of the good
/// `good`.
#[derive(Clone, Copy, Debug)]
pub(super) struct ReapOffer {
    /// The work: reaping, or threshing (§8 21, twelfth).
    pub task: FieldTask,
    /// The field to reap (the one ripe longest), or whose sheaves to thresh.
    pub field: PermanentId,
    /// Where it is done, metres: the field's centre to reap, the household's home to thresh.
    pub at: (f32, f32),
    /// Hours of a capable adult's reaping wanted.
    pub hours: f64,
    /// Pay for an hour, kilograms.
    pub kg_per_hour: f64,
    /// The good paid: the crop's grain.
    pub good: usize,
}

/// Field work a household would hire now, as the observer is shown it (wire 1.115): what
/// [`Population::field_hire`] reads of the offer someone weighing it would weigh.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldHire {
    /// The work: reaping, threshing, or preparing and sowing (`Sow`).
    pub task: FieldTask,
    /// The field it is for.
    pub field: PermanentId,
    /// Hours of a capable adult's work wanted.
    pub hours: f64,
    /// Pay for an hour, kilograms of `good`.
    pub kg_per_hour: f64,
    /// The good paid: the crop's grain.
    pub good: usize,
}

/// Whether someone `age` years old could earn the harvest wage (§8 17, fourth, slice 2; research
/// 08-22 §1: the bylaws barred from gleaning anyone who could earn a penny a day): of an age to
/// work for hire, as the content's work for hire allows.
pub(super) fn could_earn(catalog: &crate::params::Catalog, age: f64) -> bool {
    catalog
        .activities
        .iter()
        .any(|a| a.behavior == Behavior::Hire && (a.min_age_years..=a.max_age_years).contains(&age))
}

/// How much a household short of hands bids up the harvest wage's share for each day its crop has
/// stood ripe while hours go untaken (§8 21, twelfth; research 08-10 §5.6: an ordinary review
/// moves a wage 1-5 %, every 14-30 days, faster for urgent seasonal tasks): the top of that range,
/// reviewed daily at the harvest. A design prior.
pub const HARVEST_BID_PER_DAY: f64 = 0.05;

impl Population {
    /// The pay household `employer`'s field `fi` offers for an hour of a capable adult's reaping
    /// on day `day`: `firm.wage_share` of the grain the field is expected to give over the hours
    /// of work still needed to bring it in, bid up by [`HARVEST_BID_PER_DAY`] for each day the
    /// crop has stood ripe, while hours go untaken (§8 21, twelfth; research 08-10 §5.6: faster
    /// wage review for urgent seasonal tasks, harvest bidding), never past all of what the hour
    /// brings (0 when it is not ripe standing).
    fn reap_wage(&self, ctx: &Ctx, fi: usize, day: i64) -> f64 {
        let f = &ctx.land.fields[fi];
        let Some(crop) = ctx.catalog.crops.get(usize::from(f.crop)) else {
            return 0.0;
        };
        if f.task(crop, day) != Some(FieldTask::Reap) {
            return 0.0;
        }
        let water = match f.stage {
            FieldStage::Sown => ctx.land.climatology.expected_crop_factor(
                usize::from(f.crop),
                f64::from(f.need_mm),
                f64::from(f.got_mm),
            ),
            _ => 1.0,
        };
        let expected = f.expected_kg(crop, day, water);
        let remaining = f.remaining_work_h(crop, day, expected);
        if remaining <= 0.0 {
            return 0.0;
        }
        let stood = (day - f.ripe_day(crop)).max(0) as f64;
        let share = (ctx.params.firm.wage_share * (1.0 + HARVEST_BID_PER_DAY * stood)).min(1.0);
        share * expected / remaining
    }

    /// What household `employer` would hire of reaping now, its hours and its pay for an hour:
    /// for tests.
    #[doc(hidden)]
    pub fn reap_offer_for_tests(&self, ctx: &Ctx, employer: PermanentId) -> Option<(f64, f64)> {
        self.reap_offer(ctx, employer)
            .map(|o| (o.hours, o.kg_per_hour))
    }

    /// What household `employer` would hire of field work now, if anything: the offer someone
    /// weighing work for hire would weigh ([`Self::reap_offer`]), read only, for the observer.
    pub fn field_hire(&self, ctx: &Ctx, employer: PermanentId) -> Option<FieldHire> {
        self.reap_offer(ctx, employer).map(|o| FieldHire {
            task: o.task,
            field: o.field,
            hours: o.hours,
            kg_per_hour: o.kg_per_hour,
            good: o.good,
        })
    }

    /// Person `person` works `minutes` for hire on `field`, as a session out for hire ends:
    /// reaping, threshing, preparing or sowing, by what the field needs: for tests.
    #[doc(hidden)]
    pub fn hired_field_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        person: PermanentId,
        field: PermanentId,
        minutes: u32,
    ) {
        if let Some(&h) = self.index.get(&person) {
            self.hired_reaping(ctx, h, field, minutes);
        }
    }

    /// What household `employer` would hire of field work now, whether reaping or threshing, its
    /// hours and its pay for an hour: for tests.
    #[doc(hidden)]
    pub fn field_hire_for_tests(
        &self,
        ctx: &Ctx,
        employer: PermanentId,
    ) -> Option<(FieldTask, f64, f64)> {
        self.reap_offer(ctx, employer)
            .map(|o| (o.task, o.hours, o.kg_per_hour))
    }

    /// What household `employer` would hire of reaping now, if anything: its fields ripe standing
    /// need more hours than its members give at a peak's hours before the first of them has lost a
    /// quarter of its crop (the grace of [`crate::farm::FarmView::urgency`]), less the hours of
    /// hired reapers already set out for them, and no more than the grain it holds pays for.
    pub(super) fn reap_offer(&self, ctx: &Ctx, employer: PermanentId) -> Option<ReapOffer> {
        self.reaping_offer(ctx, employer)
            .or_else(|| self.thresh_offer(ctx, employer))
            .or_else(|| self.sowing_offer(ctx, employer))
    }

    /// Hired threshing (§8 21, twelfth; research 08-10 §1.3: a household allocates its hands
    /// across its own land and wage work by season; 08-02 §3.2: threshing's hours): a field whose
    /// sheaves wait cannot be prepared or sown until they are threshed, so once the window for
    /// preparing ground is open, a household whose fields need more threshing, preparing and
    /// sowing than its members give at a peak's hours over the workable days left of the window
    /// offers the threshing short, less what hired threshers already set out to give. The wage is
    /// the custom's share (`firm.wage_share`) of the grain an hour threshes, in that grain, paid
    /// from the grain once it is threshed.
    fn thresh_offer(&self, ctx: &Ctx, employer: PermanentId) -> Option<ReapOffer> {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let x = self.household(employer)?;
        let crop = ctx.catalog.crops.get(params.farm.crop)?;
        if !crop.can_prepare(day) {
            return None;
        }
        let (mut thresh, mut sow) = (0.0, 0.0);
        let mut first: Option<(PermanentId, usize)> = None;
        let mut ids: Vec<PermanentId> = Vec::new();
        for f in self.fields_of(ctx.land, employer) {
            if usize::from(f.crop) != params.farm.crop {
                continue;
            }
            if f.stage == FieldStage::Reaped && f.sheaves_kg > 0.0 {
                thresh += crop.thresh_h_per_kg * f64::from(f.sheaves_kg);
                // Once threshed it lies fallow, to be prepared and sown.
                let mut fallow = f.clone();
                fallow.stage = FieldStage::Fallow;
                fallow.work_h = 0.0;
                sow += crate::farm::to_sow_h(&fallow, crop);
                ids.push(f.id);
                if first.is_none_or(|(id, _)| f.id < id) {
                    let fi = ctx.land.fields.iter().position(|g| g.id == f.id)?;
                    first = Some((f.id, fi));
                }
            } else {
                sow += crate::farm::to_sow_h(f, crop);
            }
        }
        let (field, _) = first?;
        let days = (crate::farm::sowing_days_left(crop, day)
            * super::workable_share(ctx.land, params.farm.crop))
        .max(1.0);
        let own = self.labour_per_day(&x.members, now, params) * peak_ratio(params) * days;
        let mut short = (thresh + sow - own).min(thresh);
        if short < 0.5 {
            return None;
        }
        short -= self.hired_hours_set_out(ctx, &ids);
        let kg_per_hour = params.firm.wage_share / crop.thresh_h_per_kg.max(1e-9);
        (short >= 0.5).then_some(ReapOffer {
            task: FieldTask::Thresh,
            field,
            at: x.home,
            hours: short,
            kg_per_hour,
            good: crop.good,
        })
    }

    /// Hired preparing and sowing (§8 21, twelfth, third slice; research 08-10 §1.3; 04-02 §1.3:
    /// field work left over the hours available before the season closes): in the window for
    /// preparing ground, a household whose fields to prepare and sow, with the sheaves it must
    /// thresh first, need more hours than its members give at a peak's hours over the workable
    /// days left of the window offers the preparing and sowing short, less what hired hands
    /// already set out to give, as far as the grain it holds pays for. The wage is the custom's
    /// share of what an hour of the field's work brings (its expected crop over the hours still
    /// needed to bring it in), as reaping's is before it is bid up.
    fn sowing_offer(&self, ctx: &Ctx, employer: PermanentId) -> Option<ReapOffer> {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let x = self.household(employer)?;
        let crop = ctx.catalog.crops.get(params.farm.crop)?;
        if !crop.can_prepare(day) {
            return None;
        }
        let (mut thresh, mut sow) = (0.0, 0.0);
        let mut first: Option<(PermanentId, usize)> = None;
        let mut ids: Vec<PermanentId> = Vec::new();
        for f in self.fields_of(ctx.land, employer) {
            if usize::from(f.crop) != params.farm.crop {
                continue;
            }
            match f.stage {
                FieldStage::Reaped if f.sheaves_kg > 0.0 => {
                    thresh += crop.thresh_h_per_kg * f64::from(f.sheaves_kg);
                }
                FieldStage::Fallow | FieldStage::Prepared
                    if matches!(f.task(crop, day), Some(FieldTask::Prepare | FieldTask::Sow)) =>
                {
                    sow += crate::farm::to_sow_h(f, crop);
                    ids.push(f.id);
                    if first.is_none_or(|(id, _)| f.id < id) {
                        let fi = ctx.land.fields.iter().position(|g| g.id == f.id)?;
                        first = Some((f.id, fi));
                    }
                }
                _ => {}
            }
        }
        let (field, fi) = first?;
        let days = (crate::farm::sowing_days_left(crop, day)
            * super::workable_share(ctx.land, params.farm.crop))
        .max(1.0);
        let own = self.labour_per_day(&x.members, now, params) * peak_ratio(params) * days;
        let mut short = (sow + thresh - own).min(sow);
        if short < 0.5 {
            return None;
        }
        short -= self.hired_hours_set_out(ctx, &ids);
        let kg_per_hour = self.sow_wage(ctx, fi, day);
        if kg_per_hour <= 0.0 {
            return None;
        }
        // It pays only from the grain it can spare: what it holds beyond the food that sees it to
        // its next harvest, as it would offer it for sale (the market's keep; the harvest is
        // months off at the sowing, where reaping's is days).
        let spare = self.spare_of(ctx, x, crop.good);
        let hours = short.min(spare / kg_per_hour);
        (hours >= 0.5).then_some(ReapOffer {
            task: FieldTask::Sow,
            field,
            at: ctx.land.fields[fi].rect.centre_m(),
            hours,
            kg_per_hour,
            good: crop.good,
        })
    }

    /// What household `x` holds of good `g` beyond what it keeps ([`Self::holding_of`]): for food,
    /// beyond what sees it to its next harvest and its margin, as it would spare it for sale.
    fn spare_of(&self, ctx: &Ctx, x: &crate::person::Household, g: usize) -> f64 {
        let stores = stores_now(x, ctx.now, ctx.params, &ctx.catalog.goods);
        let held = stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let keep = self
            .holding_of(ctx, x, &stores)
            .keep
            .get(g)
            .copied()
            .unwrap_or(0.0);
        (held - keep).max(0.0)
    }

    /// The pay field `fi` offers for an hour of a capable adult's preparing or sowing on day
    /// `day`: the custom's share of the grain it is expected to give over the hours still needed
    /// to bring it in (0 when nothing is to be prepared or sown).
    fn sow_wage(&self, ctx: &Ctx, fi: usize, day: i64) -> f64 {
        let f = &ctx.land.fields[fi];
        let Some(crop) = ctx.catalog.crops.get(usize::from(f.crop)) else {
            return 0.0;
        };
        if !matches!(f.task(crop, day), Some(FieldTask::Prepare | FieldTask::Sow)) {
            return 0.0;
        }
        let expected = f.expected_kg(crop, day, 1.0);
        let remaining = f.remaining_work_h(crop, day, expected);
        if remaining <= 0.0 || expected <= 0.0 {
            return 0.0;
        }
        ctx.params.firm.wage_share * expected / remaining
    }

    /// Hours of hired field work already set out for fields `ids`: what is left of the work steps
    /// of those out for hire on them.
    fn hired_hours_set_out(&self, ctx: &Ctx, ids: &[PermanentId]) -> f64 {
        let mut hours = 0.0;
        for (_, p) in self.people.iter() {
            let Target::Field(f) = p.act.target else {
                continue;
            };
            let hired = ctx
                .catalog
                .activities
                .get(usize::from(p.act.def))
                .is_some_and(|a| a.behavior == Behavior::Hire);
            if hired && ids.contains(&f) {
                let left: u32 = p
                    .act
                    .steps
                    .iter()
                    .skip(usize::from(p.act.step))
                    .map(|s| match s {
                        Step::Work { minutes } => *minutes,
                        _ => 0,
                    })
                    .sum();
                hours += f64::from(left) / 60.0;
            }
        }
        hours
    }

    /// The reaping household `employer` would hire now ([`Self::reap_offer`]'s reaping).
    fn reaping_offer(&self, ctx: &Ctx, employer: PermanentId) -> Option<ReapOffer> {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let x = self.household(employer)?;
        let mut work = 0.0;
        let mut days = f64::INFINITY;
        // The field ripe longest is reaped first: (ripe day, field id, index).
        let mut first: Option<(i64, PermanentId, usize)> = None;
        let mut ids: Vec<PermanentId> = Vec::new();
        for f in self.fields_of(ctx.land, employer) {
            let Some(crop) = ctx.catalog.crops.get(usize::from(f.crop)) else {
                continue;
            };
            if f.task(crop, day) != Some(FieldTask::Reap) {
                continue;
            }
            let grace = 0.25 / crop.standing_loss_per_day.max(1e-6);
            work += (f.stage_work_h(crop) - f64::from(f.work_h)).max(0.0);
            let ripe = f.ripe_day(crop);
            days = days.min((grace - (day - ripe) as f64).max(1.0));
            ids.push(f.id);
            if first.is_none_or(|(r, id, _)| (ripe, f.id) < (r, id)) {
                let fi = ctx.land.fields.iter().position(|g| g.id == f.id)?;
                first = Some((ripe, f.id, fi));
            }
        }
        let (_, field, fi) = first?;
        let own = self.labour_per_day(&x.members, now, params) * peak_ratio(params) * days;
        let mut short = work - own;
        if short < 0.5 {
            return None;
        }
        // Hours hired reapers have already set out to give its fields.
        short -= self.hired_hours_set_out(ctx, &ids);
        let kg_per_hour = self.reap_wage(ctx, fi, day);
        let crop = ctx
            .catalog
            .crops
            .get(usize::from(ctx.land.fields[fi].crop))?;
        let good = crop.good;
        let held = stores_now(x, now, params, &ctx.catalog.goods)
            .get(good)
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        if kg_per_hour <= 0.0 {
            return None;
        }
        let hours = short.min(held / kg_per_hour);
        (hours >= 0.5).then_some(ReapOffer {
            task: FieldTask::Reap,
            field,
            at: ctx.land.fields[fi].rect.centre_m(),
            hours,
            kg_per_hour,
            good,
        })
    }

    /// Person `h` has reaped `minutes` for hire on `field`: the work is done as the field's
    /// household's own would be (its grain goes to that household's store), at the rate of the
    /// best reaping the worker's household holds the tools for, wearing them; and the field's
    /// household pays the worker's household the wage for the hours through the ledger (the
    /// `wage` channel), working no longer than it can pay for.
    pub(super) fn hired_reaping(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        field: PermanentId,
        minutes: u32,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let Some((who, worker, age)) = self
            .people
            .get(h)
            .map(|p| (p.id, p.household, p.age_years(now)))
        else {
            return;
        };
        let Some(fi) = ctx.land.fields.iter().position(|f| f.id == field) else {
            return;
        };
        let employer = ctx.land.fields[fi].household;
        if employer == worker {
            return;
        }
        let Some(crop) = ctx.catalog.crops.get(usize::from(ctx.land.fields[fi].crop)) else {
            return;
        };
        let (here, there) = (
            self.household(worker).and_then(|x| x.settlement),
            self.household(employer).and_then(|x| x.settlement),
        );
        if here != there {
            crate::d21(
                day,
                format_args!(
                    "hire_session far true task {:?} worker {worker} employer {employer}",
                    ctx.land.fields[fi].task(crop, day)
                ),
            );
        }
        // Sheaves waiting are threshed for hire at the household's home (§8 21, twelfth).
        if ctx.land.fields[fi].stage == FieldStage::Reaped && ctx.land.fields[fi].sheaves_kg > 0.0 {
            self.hired_threshing(ctx, h, field, minutes);
            return;
        }
        // Ground to prepare or sow is worked for hire (§8 21, twelfth, third slice).
        if let Some(task @ (FieldTask::Prepare | FieldTask::Sow)) =
            ctx.land.fields[fi].task(crop, day)
        {
            self.hired_sowing(ctx, h, field, task, minutes);
            return;
        }
        let good = crop.good;
        let left = (ctx.land.fields[fi].stage_work_h(crop) - f64::from(ctx.land.fields[fi].work_h))
            .max(0.0);
        let kg_per_hour = self.reap_wage(ctx, fi, day);
        if kg_per_hour <= 0.0 || left <= 0.0 {
            return;
        }
        // The best reaping the worker's household has the tools for.
        let Some(held) = self
            .household(worker)
            .map(|x| stores_now(x, now, params, &ctx.catalog.goods))
        else {
            return;
        };
        let goods = &ctx.catalog.goods;
        let tool = |t: &usize| {
            make::tools_for(goods, *t, &|g| held.get(g).copied().unwrap_or(0.0)) >= MIN_TOOL
        };
        let Some(def) = ctx
            .catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Farm && a.task == Some(FieldTask::Reap))
            .filter(|a| a.tools.iter().all(tool))
            .max_by(|a, b| a.rate.total_cmp(&b.rate))
            .cloned()
        else {
            return;
        };
        let can = self
            .household(employer)
            .map(|x| stores_now(x, now, params, goods))
            .and_then(|s| s.get(good).copied())
            .unwrap_or(0.0)
            .max(0.0);
        let eff = interpolate(&params.capacity_by_age, age) * def.rate;
        let full = f64::from(minutes) / 60.0 * eff;
        let hours = full.min(left).min(can / kg_per_hour);
        if hours <= 1e-6 {
            return;
        }
        self.field_work(ctx, who, employer, field, FieldTask::Reap, hours);
        // The tools are worn for the time they were used.
        let used = f64::from(minutes) / 60.0 * (hours / full.max(1e-9));
        self.wear_tools(ctx, worker, &def.tools, used);
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let leg = [Leg {
            from: employer,
            to: worker,
            good,
            amount: kg_per_hour * hours,
        }];
        if self.transfer(now, params, goods, &leg, Channel::Wage) {
            // For the record (§8 18, first): it changes nothing.
            self.margins.reapings += 1;
            self.margins.reaped_h += hours;
            self.margins.reaped_paid_kg += kg_per_hour * hours;
            // The worker saw the wages paid; the field's household saw the work done (ADR-0014
            // §2).
            let (wages, work) = (crate::ties::Act::WagesPaid, crate::ties::Act::WorkSeen);
            self.note_between(ctx, Some(who), worker, employer, wages, work, 0.0);
        }
    }

    /// Person `h` has threshed `minutes` for hire of `field`'s sheaves (§8 21, twelfth): as the
    /// field's household's own would, at the best threshing the worker's household has the tools
    /// for, wearing them; the grain goes into that household's store, which then pays the worker's
    /// household the threshing wage out of it through the ledger (the `wage` channel), no more than
    /// it holds.
    fn hired_threshing(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        field: PermanentId,
        minutes: u32,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let Some((who, worker, age)) = self
            .people
            .get(h)
            .map(|p| (p.id, p.household, p.age_years(now)))
        else {
            return;
        };
        let Some(fi) = ctx.land.fields.iter().position(|f| f.id == field) else {
            return;
        };
        let employer = ctx.land.fields[fi].household;
        let Some(crop) = ctx.catalog.crops.get(usize::from(ctx.land.fields[fi].crop)) else {
            return;
        };
        let (good, per_kg) = (crop.good, crop.thresh_h_per_kg.max(1e-9));
        let left = per_kg * f64::from(ctx.land.fields[fi].sheaves_kg);
        let Some(held) = self
            .household(worker)
            .map(|x| stores_now(x, now, params, &ctx.catalog.goods))
        else {
            return;
        };
        let goods = &ctx.catalog.goods;
        let tool = |t: &usize| {
            make::tools_for(goods, *t, &|g| held.get(g).copied().unwrap_or(0.0)) >= MIN_TOOL
        };
        let Some(def) = ctx
            .catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Farm && a.task == Some(FieldTask::Thresh))
            .filter(|a| a.tools.iter().all(tool))
            .max_by(|a, b| a.rate.total_cmp(&b.rate))
            .cloned()
        else {
            return;
        };
        let eff = interpolate(&params.capacity_by_age, age) * def.rate;
        let full = f64::from(minutes) / 60.0 * eff;
        let hours = full.min(left);
        if hours <= 1e-6 {
            return;
        }
        self.field_work(ctx, who, employer, field, FieldTask::Thresh, hours);
        let used = f64::from(minutes) / 60.0 * (hours / full.max(1e-9));
        self.wear_tools(ctx, worker, &def.tools, used);
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let can = self
            .household(employer)
            .map(|x| stores_now(x, now, params, goods))
            .and_then(|s| s.get(good).copied())
            .unwrap_or(0.0)
            .max(0.0);
        let pay = (params.firm.wage_share / per_kg * hours).min(can);
        if pay <= 1e-9 {
            return;
        }
        let leg = [Leg {
            from: employer,
            to: worker,
            good,
            amount: pay,
        }];
        if self.transfer(now, params, goods, &leg, Channel::Wage) {
            let (wages, work) = (crate::ties::Act::WagesPaid, crate::ties::Act::WorkSeen);
            self.note_between(ctx, Some(who), worker, employer, wages, work, 0.0);
        }
    }

    /// Person `h` has prepared or sown (`task`) `field` for hire for `minutes` (§8 21, twelfth,
    /// third slice): as the field's household's own would, its seed from that household's store,
    /// at the best of that work the worker's household has the tools for, wearing them; paid for
    /// the hours the field took, the custom's share of what an hour of it brings, out of the grain
    /// the field's household holds.
    fn hired_sowing(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        field: PermanentId,
        task: FieldTask,
        minutes: u32,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let Some((who, worker, age)) = self
            .people
            .get(h)
            .map(|p| (p.id, p.household, p.age_years(now)))
        else {
            return;
        };
        let Some(fi) = ctx.land.fields.iter().position(|f| f.id == field) else {
            return;
        };
        let employer = ctx.land.fields[fi].household;
        let Some(crop) = ctx.catalog.crops.get(usize::from(ctx.land.fields[fi].crop)) else {
            return;
        };
        let good = crop.good;
        let kg_per_hour = self.sow_wage(ctx, fi, day);
        let Some(held) = self
            .household(worker)
            .map(|x| stores_now(x, now, params, &ctx.catalog.goods))
        else {
            return;
        };
        let goods = &ctx.catalog.goods;
        let tool = |t: &usize| {
            make::tools_for(goods, *t, &|g| held.get(g).copied().unwrap_or(0.0)) >= MIN_TOOL
        };
        let Some(def) = ctx
            .catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Farm && a.task == Some(task))
            .filter(|a| a.tools.iter().all(tool))
            .max_by(|a, b| a.rate.total_cmp(&b.rate))
            .cloned()
        else {
            return;
        };
        // Paid from what it can spare beyond the food that sees it to its next harvest.
        let can = self
            .household(employer)
            .map_or(0.0, |x| self.spare_of(ctx, x, good));
        if kg_per_hour <= 0.0 {
            return;
        }
        let eff = interpolate(&params.capacity_by_age, age) * def.rate;
        let full = (f64::from(minutes) / 60.0 * eff).min(can / kg_per_hour);
        if full <= 1e-6 {
            return;
        }
        // The hours the field took: its stage's work done, as far as its seed allowed.
        let (stage, before) = {
            let f = &ctx.land.fields[fi];
            (f.stage, f64::from(f.work_h))
        };
        let left = (ctx.land.fields[fi].stage_work_h(crop) - before).max(0.0);
        self.field_work(ctx, who, employer, field, task, full);
        let f = &ctx.land.fields[fi];
        let done = if f.stage != stage {
            left
        } else {
            (f64::from(f.work_h) - before).max(0.0)
        }
        .min(full);
        if done <= 1e-6 {
            return;
        }
        let used = f64::from(minutes) / 60.0 * (done / (f64::from(minutes) / 60.0 * eff).max(1e-9));
        self.wear_tools(ctx, worker, &def.tools, used);
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let leg = [Leg {
            from: employer,
            to: worker,
            good,
            amount: kg_per_hour * done,
        }];
        if self.transfer(now, params, goods, &leg, Channel::Wage) {
            let (wages, work) = (crate::ties::Act::WagesPaid, crate::ties::Act::WorkSeen);
            self.note_between(ctx, Some(who), worker, employer, wages, work, 0.0);
        }
    }
}
