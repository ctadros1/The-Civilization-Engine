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

/// Reaping a household would hire: the field it is for, where it is, the hours wanted and the pay
/// for an hour of a capable adult's reaping, kilograms of the good `good`.
#[derive(Clone, Copy, Debug)]
pub(super) struct ReapOffer {
    /// The field to reap (the one ripe longest).
    pub field: PermanentId,
    /// The field's centre, metres.
    pub at: (f32, f32),
    /// Hours of a capable adult's reaping wanted.
    pub hours: f64,
    /// Pay for an hour, kilograms.
    pub kg_per_hour: f64,
    /// The good paid: the crop's grain.
    pub good: usize,
}

impl Population {
    /// The pay household `employer`'s field `fi` offers for an hour of a capable adult's reaping
    /// on day `day`: `firm.wage_share` of the grain the field is expected to give over the hours
    /// of work still needed to bring it in (0 when it is not ripe standing).
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
        ctx.params.firm.wage_share * expected / remaining
    }

    /// What household `employer` would hire of reaping now, if anything: its fields ripe standing
    /// need more hours than its members give at a peak's hours before the first of them has lost a
    /// quarter of its crop (the grace of [`crate::farm::FarmView::urgency`]), less the hours of
    /// hired reapers already set out for them, and no more than the grain it holds pays for.
    pub(super) fn reap_offer(&self, ctx: &Ctx, employer: PermanentId) -> Option<ReapOffer> {
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
                short -= f64::from(left) / 60.0;
            }
        }
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
            // The worker saw the wages paid; the field's household saw the work done (ADR-0014
            // §2).
            let (wages, work) = (crate::ties::Act::WagesPaid, crate::ties::Act::WorkSeen);
            self.note_between(ctx, Some(who), worker, employer, wages, work, 0.0);
        }
    }
}
