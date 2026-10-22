//! Irrigation ditches a household digs to its own field (M6b slice BG; ADR-0022 §11; the M6 fire
//! brief §1.11: "A household ... digs a ditch from an intake to its fields at a gradient of at
//! least 0.05 %. In a dry spell its people choose to water a field"). A household that saw its
//! last crop on a field go short of water weighs a ditch to it from the river: the grain that
//! water would have grown, at what it costs the household to grow, over the ditch's years, against
//! the digging. Its people dig it as they choose; once dug, its intake is opened while the crop
//! grows and its root zone runs short, and shut otherwise. The water is `civ_land::ditches`'.

use super::*;
use crate::decide::DitchFacts;
use civ_land::RectCm;
use civ_land::ditches::DitchSegment;
use civ_land::earth::{EARTH_VERSION, EarthKind, Earthwork};

/// Purpose tag for the day of the year a household weighs a ditch.
const PURPOSE_DITCH_REVIEW: u64 = 0x6469_7463_6872_7631; // "ditchrv1"
/// Metres between the points along a ditch's line checked for what lies there.
const LINE_STEP_M: f64 = 2.0;

/// A ditch a household would dig: to which field, from which reach and point, its fall, and what
/// it would bring the household a year, hours of its work.
struct DitchPlan {
    field: PermanentId,
    reach: u32,
    intake: [f64; 2],
    to: [f64; 2],
    drop_m: f64,
    gain_h: f64,
    net_h: f64,
}

impl Population {
    /// The households whose day of the year it is weigh a ditch to their fields, and every ditch
    /// dug has its intake opened or shut for the day (M6b slice BG).
    pub(super) fn ditches_day(&mut self, ctx: &mut Ctx, day: i64) {
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| !x.members.is_empty())
            .map(|(_, x)| x.id)
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_DITCH_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_ditch(ctx, h);
        }
        self.open_ditches(ctx, day);
    }

    /// Household `household` weighs a ditch now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_ditch_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        self.review_ditch(ctx, household);
    }

    /// Each finished ditch's intake is opened while its field's crop grows and its root zone holds
    /// less than the water a crop draws at ease, and shut otherwise (the M6 fire brief §1.11:
    /// "in a dry spell its people choose to water a field"; a design prior that it costs no
    /// labour to open or shut).
    fn open_ditches(&self, ctx: &mut Ctx, day: i64) {
        let total = ctx.land_params.weather.soil_water_mm;
        let easy = ctx.land_params.weather.easy_water_share;
        for i in 0..ctx.land.ditches.len() {
            let d = ctx.land.ditches[i];
            let kept = self
                .household_of_work(ctx, d.work)
                .is_some_and(|h| self.household(h).is_some_and(|x| !x.members.is_empty()));
            let thirsty = ctx
                .land
                .fields
                .iter()
                .find(|f| f.id == d.field)
                .is_some_and(|f| {
                    let growing = f.stage == civ_land::FieldStage::Sown
                        && ctx
                            .land
                            .climatology
                            .crops
                            .get(usize::from(f.crop))
                            .is_some_and(|cw| {
                                (0..i64::from(cw.grow_days)).contains(&(day - f.sown_day))
                            });
                    growing && f64::from(f.water_mm) < (1.0 - easy) * total
                });
            ctx.land.ditches[i].open = kept && thirsty;
        }
    }

    /// The household whose ditch earthwork `work` is.
    fn household_of_work(&self, ctx: &Ctx, work: PermanentId) -> Option<PermanentId> {
        ctx.land
            .earthworks
            .iter()
            .find(|e| e.id == work)
            .map(|e| e.household)
    }

    /// Household `household` weighs a ditch to each of its fields whose last crop got less than
    /// `ditches.short_share` of the water it needed, and digs the one whose gain over its years
    /// most exceeds its digging, if any does: one at a time, and none to a field with one.
    fn review_ditch(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let Some(x) = self.household(household) else {
            return;
        };
        if x.members.is_empty()
            || ctx.land.ditches.iter().any(|d| {
                self.household_of_work(ctx, d.work) == Some(household)
                    && ctx
                        .land
                        .earthworks
                        .iter()
                        .any(|e| e.id == d.work && e.done < 1.0)
            })
        {
            return;
        }
        let costs = self.own_costs_of(ctx, x);
        let dp = &ctx.params.ditches;
        let mut best: Option<DitchPlan> = None;
        for f in ctx.land.fields.iter().filter(|f| f.household == household) {
            if f.stage == civ_land::FieldStage::Sown
                || f.need_mm <= 0.0
                || ctx.land.ditches.iter().any(|d| d.field == f.id)
            {
                continue;
            }
            let (need, got) = (f64::from(f.need_mm), f64::from(f.got_mm));
            if got / need >= dp.short_share {
                continue;
            }
            let (Some(crop), Some(cw)) = (
                ctx.catalog.crops.get(usize::from(f.crop)),
                ctx.land.climatology.crops.get(usize::from(f.crop)),
            ) else {
                continue;
            };
            let Some(hours_per_kg) = costs.get(crop.good).copied().flatten() else {
                continue;
            };
            // The grain the water it lacked would have grown on it in an average year, at what it
            // costs the household to grow.
            let lost = 1.0 - cw.raw_factor(need, got);
            let gain_h =
                crop.yield_kg_per_ha * f64::from(f.ground) * f.area_ha() * lost * hours_per_kg;
            let Some(plan) = self.ditch_site(ctx, f, gain_h) else {
                continue;
            };
            if plan.net_h > 0.0 && best.as_ref().is_none_or(|b| plan.net_h > b.net_h) {
                best = Some(plan);
            }
        }
        let Some(p) = best else {
            return;
        };
        let length = (p.intake[0] - p.to[0]).hypot(p.intake[1] - p.to[1]);
        let work = ctx.ids.allocate();
        let ditch = DitchSegment {
            work,
            reach: p.reach,
            field: p.field,
            a: p.intake,
            b: p.to,
            bottom_m: dp.bottom_m,
            depth_m: dp.depth_m,
            side_run: dp.side_run,
            drop_m: p.drop_m,
            weeds: 0.0,
            silt_m: 0.0,
            open: false,
            taken_m3: 0.0,
            kept_m3: 0.0,
            seeped_m3: 0.0,
            returned_m3: 0.0,
        };
        // Its earthwork's ground is a small square at its intake: the line itself lies in its
        // segment, below the grid.
        let rect = RectCm {
            x: ((p.intake[0] - 1.0) * 100.0).max(0.0) as i32,
            y: ((p.intake[1] - 1.0) * 100.0).max(0.0) as i32,
            w: 200,
            h: 200,
        };
        ctx.land.earthworks.push(Earthwork {
            id: work,
            kind: EarthKind::Ditch,
            rect,
            level_cm: 0,
            side_run_cm: (dp.side_run * 100.0).round() as u16,
            plot: None,
            deposit: None,
            heap: None,
            household,
            cut_m3: 0.0,
            done: 0.0,
            version: EARTH_VERSION,
            begun: ctx.now,
        });
        ctx.land.ditches.push(ditch);
        let worth = (p.gain_h / dp.dig_hours(length).max(1e-6)) as f32;
        if let Some(x) = self.household_mut(household) {
            x.ditch_worth = worth;
        }
    }

    /// The best ditch to field `f` worth `gain_h` hours a year (M6b slice BG): from a point on any
    /// reach within `ditches.max_length_m` of the field's middle whose water stands high enough
    /// over the field that a bed taken in half a ditch's depth below it and cut a ditch's depth
    /// into the field falls at least the least gradient, along a line that crosses no other field,
    /// plot or well. Of those, the one whose gain over its years most exceeds its digging.
    fn ditch_site(&self, ctx: &Ctx, f: &civ_land::Field, gain_h: f64) -> Option<DitchPlan> {
        let dp = &ctx.params.ditches;
        let map = ctx.map;
        let cell = f64::from(map.cell_size_m);
        let (fx, fy) = f.rect.centre_m();
        let to = [f64::from(fx), f64::from(fy)];
        let field_ground = civ_land::height_at(map, fx, fy);
        let mut best: Option<DitchPlan> = None;
        for (r, reach) in map.reaches.iter().enumerate() {
            for &c in &reach.cells {
                let intake = [
                    (f64::from(c % map.width) + 0.5) * cell,
                    (f64::from(c / map.width) + 0.5) * cell,
                ];
                let length = (intake[0] - to[0]).hypot(intake[1] - to[1]);
                if length <= 0.0 || length > dp.max_length_m {
                    continue;
                }
                let water = civ_land::height_at(map, intake[0] as f32, intake[1] as f32);
                let drop = water - field_ground + dp.depth_m / 2.0;
                if drop / length < ctx.land_params.ditches.min_gradient {
                    continue;
                }
                let net_h = gain_h * dp.life_years - dp.dig_hours(length);
                if best.as_ref().is_some_and(|b| b.net_h >= net_h)
                    || !line_clear(ctx, f.id, intake, to)
                {
                    continue;
                }
                best = Some(DitchPlan {
                    field: f.id,
                    reach: r as u32,
                    intake,
                    to,
                    drop_m: drop,
                    gain_h,
                    net_h,
                });
            }
        }
        best
    }

    /// The ditch household `x`'s people may dig, if any (M6b slice BG): the household's ditch not
    /// yet dug; where its work is done (its middle), the walk there from home over the ground as
    /// its people walk it, the digging left, and what each hour of it is worth to the household.
    pub(super) fn ditch_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<DitchFacts> {
        if dark || ctx.land.ditches.is_empty() {
            return None;
        }
        let (d, e) = ctx.land.ditches.iter().find_map(|d| {
            let e = ctx.land.earthworks.iter().find(|e| e.id == d.work)?;
            (e.household == x.id && e.kind == EarthKind::Ditch && e.done < 1.0).then_some((d, e))
        })?;
        // A ditch is dug from its middle; a headrace from its wheel, on the bank (M10a slice DR).
        let at = if ctx.land.installations.iter().any(|x| x.id == d.work) {
            (d.b[0] as f32, d.b[1] as f32)
        } else {
            (
                ((d.a[0] + d.b[0]) / 2.0) as f32,
                ((d.a[1] + d.b[1]) / 2.0) as f32,
            )
        };
        let walk_s = self
            .homes
            .get(&field_key)?
            .reach
            .seconds_to(cell_of(ctx.map, at))?;
        let total = ctx.params.ditches.dig_hours(d.length_m());
        Some(DitchFacts {
            ditch: d.work,
            at,
            walk_min: f64::from(walk_s) / 60.0,
            left_h: total * f64::from(1.0 - e.done).max(0.0),
            saves_per_hour: f64::from(x.ditch_worth),
        })
    }

    /// `hours` of a capable adult's work by `who` of `household` on ditch `ditch` (M6b slice BG):
    /// as much channel as those hours dig and form (research 11-12 §2.2), its earth thrown along
    /// its side. Once all of it is dug, the ditch is finished.
    pub(super) fn ditch_work(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        ditch: PermanentId,
        hours: f64,
    ) {
        let Some(d) = ctx.land.ditches.iter().find(|d| d.work == ditch).copied() else {
            return;
        };
        let Some(e) = ctx
            .land
            .earthworks
            .iter_mut()
            .find(|e| e.id == ditch && e.household == household && e.done < 1.0)
        else {
            return;
        };
        let total = d.cut_m3();
        let dug = (f64::from(e.cut_m3) + hours.max(0.0) / ctx.params.ditches.h_per_m3.max(1e-9))
            .min(total);
        e.cut_m3 = dug as f32;
        e.done = if dug >= total - 1e-6 {
            1.0
        } else {
            (dug / total) as f32
        };
        if e.done < 1.0 {
            return;
        }
        e.cut_m3 = total as f32;
        // A headrace is a ditch whose work an installation shares (M10a slice DR).
        let sentence = if ctx.land.installations.iter().any(|x| x.id == ditch) {
            format!(
                "{} finished digging a headrace {:.0} m long from the river to their household's \
                 mill.",
                self.name_of(who),
                d.length_m()
            )
        } else {
            format!(
                "{} finished digging a ditch {:.0} m long from the river to their household's \
                 field.",
                self.name_of(who),
                d.length_m()
            )
        };
        let at = (
            ((d.a[0] + d.b[0]) / 2.0) as f32,
            ((d.a[1] + d.b[1]) / 2.0) as f32,
        );
        let settlement = self.household(household).and_then(|x| x.settlement);
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Ditch,
            vec![who],
            settlement,
            Some(at),
            0.0,
            sentence,
        );
    }

    /// The ditch person `who`'s household is digging, as their decision sees it: for tests.
    #[doc(hidden)]
    pub fn ditch_facts_for_tests(&self, ctx: &Ctx, who: PermanentId) -> Option<DitchFacts> {
        let x = self.household(self.person(who)?.household)?;
        let key = x.settlement.unwrap_or(x.id);
        self.ditch_facts(ctx, x, key, false)
    }

    /// `hours` of `who`'s work on ditch `ditch`, as their session would put in: for tests.
    #[doc(hidden)]
    pub fn ditch_work_by_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        ditch: PermanentId,
        hours: f64,
    ) {
        let Some(household) = self.person(who).map(|p| p.household) else {
            return;
        };
        self.ditch_work(ctx, who, household, ditch, hours);
    }
}

/// Whether a ditch's line from `a` to `b` crosses no field but `field`, no plot and no well.
pub(super) fn line_clear(ctx: &Ctx, field: PermanentId, a: [f64; 2], b: [f64; 2]) -> bool {
    let length = (a[0] - b[0]).hypot(a[1] - b[1]);
    let steps = (length / LINE_STEP_M).ceil().max(1.0) as u32;
    (0..=steps).all(|i| {
        let t = f64::from(i) / f64::from(steps);
        let x = ((a[0] + (b[0] - a[0]) * t) * 100.0) as i32;
        let y = ((a[1] + (b[1] - a[1]) * t) * 100.0) as i32;
        let point = RectCm { x, y, w: 1, h: 1 };
        !ctx.land
            .fields
            .iter()
            .any(|f| f.id != field && f.rect.near(&point, 0))
            && !ctx.land.plots.iter().any(|p| p.rect.near(&point, 0))
            && !ctx.land.wells.near(&point, 0)
    })
}
