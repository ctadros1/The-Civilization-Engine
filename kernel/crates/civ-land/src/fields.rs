//! Fields (ADR-0004 §3): ground a household works for a crop. A field is a rectangle in integer
//! centimetres. Its crop goes through the year as people do the work, preparing, sowing,
//! tending, reaping and threshing, and as the season turns. What a harvest brings depends on the
//! field's ground, the water its crop had through the season (ADR-0012 §2: the land keeps each
//! growing field's water balance), and whether the work was done in time (research 08-02
//! §2.1–2.2: spare labour later cannot make up for a crop left unweeded or standing). Nothing
//! here decides who works which field or when.
//!
//! Soil fertility does not fall by a share per crop and return by a share per fallow year
//! (research 03-04 warns against exactly that shortcut; unmanured continuous wheat holds about
//! 1 t/ha): a field keeps the quality of its ground. A nutrient budget arrives with soils.

use crate::SoilParams;
use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, SimTime};

/// A crop and how it is grown by hand (content kind `crop`).
#[derive(Clone, Debug, PartialEq)]
pub struct CropParams {
    /// Content id, for example `core:crop/barley`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The good the harvest yields, by index in the content's goods.
    pub good: usize,
    /// The good seed is kept as, by index in the content's goods.
    pub seed_good: usize,
    /// Seed sown, kilograms per hectare.
    pub seed_kg_per_ha: f64,
    /// Clean grain harvested from ground of average quality in an average year with timely work,
    /// kilograms per hectare.
    pub yield_kg_per_ha: f64,
    /// First day of the year preparing ground for this crop makes sense.
    pub prepare_from_day: u16,
    /// First day of the year it can be sown.
    pub sow_from_day: u16,
    /// Last day of the year it can be sown.
    pub sow_until_day: u16,
    /// Share of the yield lost for each day sowing finishes after `sow_from_day`.
    pub late_sowing_loss_per_day: f64,
    /// Days from sowing to ripeness.
    pub grow_days: u16,
    /// Share of the ripe crop lost for each day it stands unreaped.
    pub standing_loss_per_day: f64,
    /// Share of the yield lost when the crop gets no tending at all (weeds).
    pub untended_loss: f64,
    /// Work to break new ground for its first crop, person-hours per hectare.
    pub break_h_per_ha: f64,
    /// Work to prepare cropped ground again, person-hours per hectare.
    pub prepare_h_per_ha: f64,
    /// Work to sow, person-hours per hectare.
    pub sow_h_per_ha: f64,
    /// Tending a sown crop needs, person-hours per hectare over the season.
    pub tend_h_per_ha: f64,
    /// Work to reap and carry the crop home, person-hours per hectare.
    pub reap_h_per_ha: f64,
    /// Work to thresh and clean the grain, person-hours per kilogram.
    pub thresh_h_per_kg: f64,
    /// The straw threshing leaves: the good it is kept as (by index in the content's goods) and
    /// kilograms of it per kilogram of grain.
    pub straw: Option<(usize, f64)>,
    /// Its water use as a multiple of the reference evapotranspiration at the start, in
    /// mid-season and at ripeness (FAO-56's crop coefficients; ADR-0012 §2).
    pub kc: [f64; 3],
    /// Days of its initial, development, mid-season and late stages; they sum to `grow_days`.
    pub kc_days: [u16; 4],
    /// The share of its yield lost per share of its water need unmet (FAO's `Ky`).
    pub ky: f64,
    pub n_kg_per_kg_grain: f64,
    pub n_kg_per_kg_straw: f64,
    pub roots_n_kg_per_ha: f64,
}

impl CropParams {
    /// The day of the year of day index `day`.
    fn day_of_year(day: i64) -> i64 {
        day.rem_euclid(DAYS_PER_YEAR)
    }

    /// Whether `day` falls in the sowing window.
    pub fn can_sow(&self, day: i64) -> bool {
        let d = Self::day_of_year(day);
        (i64::from(self.sow_from_day)..=i64::from(self.sow_until_day)).contains(&d)
    }

    /// Whether ground can usefully be prepared on `day`: from `prepare_from_day` to the end of
    /// the sowing window.
    pub fn can_prepare(&self, day: i64) -> bool {
        let d = Self::day_of_year(day);
        (i64::from(self.prepare_from_day)..=i64::from(self.sow_until_day)).contains(&d)
    }

    /// The share of the yield that sowing finished on `day` keeps.
    pub fn timeliness(&self, day: i64) -> f64 {
        let late = (Self::day_of_year(day) - i64::from(self.sow_from_day)).max(0) as f64;
        (1.0 - self.late_sowing_loss_per_day * late).clamp(0.0, 1.0)
    }
}

/// Field work, in the order a crop needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FieldTask {
    /// Break or prepare the ground.
    Prepare,
    /// Sow.
    Sow,
    /// Weed and watch the growing crop.
    Tend,
    /// Reap and carry the crop home.
    Reap,
    /// Thresh and clean the grain at home.
    Thresh,
    /// Manure the field.
    Manure,
}

impl FieldTask {
    /// Whether the task turns the soil, and so waits for a day the ground can be worked
    /// (ADR-0012 §5): breaking or preparing ground, and sowing. Weeding, reaping and threshing go
    /// on whatever the ground is like.
    pub fn turns_soil(self) -> bool {
        matches!(self, FieldTask::Prepare | FieldTask::Sow | FieldTask::Manure)
    }

    /// Every task, in order (part of the boundary: never reorder).
    pub const ALL: [FieldTask; 6] = [
        FieldTask::Prepare,
        FieldTask::Sow,
        FieldTask::Tend,
        FieldTask::Reap,
        FieldTask::Thresh,
        FieldTask::Manure,
    ];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            FieldTask::Prepare => "prepare",
            FieldTask::Sow => "sow",
            FieldTask::Tend => "tend",
            FieldTask::Reap => "reap",
            FieldTask::Thresh => "thresh",
            FieldTask::Manure => "manure",
        }
    }

    /// The task with an authored name.
    pub fn from_name(name: &str) -> Option<FieldTask> {
        FieldTask::ALL.into_iter().find(|t| t.name() == name)
    }
}

/// What limited a field's harvest (ADR-0012 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarvestLimit {
    None,
    Water,
    Nitrogen,
}

/// One past harvest of a field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarvestRecord {
    pub year: u32,
    pub yield_kg_per_ha: f32,
    pub limit: HarvestLimit,
}

/// Where a field is in its year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FieldStage {
    /// Not worked this year: new ground, or fallow since its last harvest.
    Fallow,
    /// Prepared for sowing.
    Prepared,
    /// Sown; growing, or ripe once its days are up.
    Sown,
    /// Reaped: the crop is at home in sheaves, waiting to be threshed.
    Reaped,
}

/// An axis-aligned rectangle in integer centimetres from the map's north-west corner
/// (ADR-0004 §1): its north-west corner and its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RectCm {
    /// West edge.
    pub x: i32,
    /// North edge.
    pub y: i32,
    /// Width (west to east).
    pub w: i32,
    /// Height (north to south).
    pub h: i32,
}

impl RectCm {
    /// Area, hectares.
    pub fn area_ha(&self) -> f64 {
        f64::from(self.w.max(0)) * f64::from(self.h.max(0)) / 1.0e8
    }

    /// Centre, metres.
    pub fn centre_m(&self) -> (f32, f32) {
        (
            ((f64::from(self.x) + f64::from(self.w) / 2.0) / 100.0) as f32,
            ((f64::from(self.y) + f64::from(self.h) / 2.0) / 100.0) as f32,
        )
    }

    /// Whether the two rectangles are closer than `gap` centimetres (or overlap).
    pub fn near(&self, other: &RectCm, gap: i32) -> bool {
        self.x - gap < other.x + other.w
            && other.x - gap < self.x + self.w
            && self.y - gap < other.y + other.h
            && other.y - gap < self.y + self.h
    }
}

/// Who holds a claim to land (ADR-0007 §1): a household, or a settlement for its people.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Party {
    /// A household.
    Household(PermanentId),
    /// A settlement.
    Settlement(PermanentId),
}

impl Party {
    /// The household, when the party is one.
    pub fn household(self) -> Option<PermanentId> {
        match self {
            Party::Household(id) => Some(id),
            Party::Settlement(_) => None,
        }
    }

    /// The settlement, when the party is one.
    pub fn settlement(self) -> Option<PermanentId> {
        match self {
            Party::Household(_) => None,
            Party::Settlement(id) => Some(id),
        }
    }
}

/// A household's lease of a field from the household that holds it (ADR-0007 §3): it works the
/// field until `until`, and pays its holder `holder_share` of the grain it threshes from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lease {
    /// When it began.
    pub since: SimTime,
    /// When its term ends, to be renewed or not at the holder's next review.
    pub until: SimTime,
    /// The holder's share of the grain threshed from the field.
    pub holder_share: f32,
}

/// Ground a household works for a crop.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// Permanent id.
    pub id: PermanentId,
    /// The household that works it and keeps its crop: its user.
    pub household: PermanentId,
    /// Who holds it (ADR-0007): the household that broke it or its settlement, by the world's
    /// regime. Holding it changes nothing about how it yields.
    pub holder: Party,
    /// The lease its user works it under, when its holder let it to another household.
    pub lease: Option<Lease>,
    /// Where it is.
    pub rect: RectCm,
    /// The crop, by index in the content's crops.
    pub crop: u16,
    /// Where it is in its year.
    pub stage: FieldStage,
    /// When the stage began.
    pub stage_since: SimTime,
    /// Person-hours of a capable adult's work done toward the current stage.
    pub work_h: f32,
    /// Hours of tending the growing crop has had.
    pub tended_h: f32,
    /// How good its ground is: 1 for average ground (the mean richness of its patches).
    pub ground: f32,
    /// Work to clear the ground of what grows on it before it is first broken (woodland),
    /// person-hours per hectare; 0 for open ground.
    pub clear_h_per_ha: f32,
    /// The ground has been broken (cleared and dug for a first crop).
    pub broken: bool,
    /// The day sowing finished (a day index); meaningful once sown.
    pub sown_day: i64,
    /// Grain reaped and not yet threshed, kilograms.
    pub sheaves_kg: f32,
    /// Crops harvested from it in all.
    pub harvests: u16,
    /// Water in the root zone of its growing crop, mm (ADR-0012 §2; meaningful once sown).
    pub water_mm: f32,
    /// The water its crop has needed so far this season (its evapotranspiration unstressed), mm.
    pub need_mm: f32,
    /// The water its crop has had of that, mm.
    pub got_mm: f32,
    /// The fast pool of organic nitrogen, kg (ADR-0012 §3).
    pub fast_n_kg: f32,
    /// The slow pool of organic nitrogen, kg.
    pub slow_n_kg: f32,
    /// Records of the last harvests.
    pub harvest_records: Vec<HarvestRecord>,
}

/// What a piece of field work did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorkDone {
    /// Seed sown, kilograms (taken from the household's seed).
    pub seed_kg: f64,
    /// Manure applied or other materials used, kilograms.
    pub used_kg: f64,
    /// Grain threshed, kilograms (into the household's store).
    pub grain_kg: f64,
    /// The stage was finished.
    pub finished: bool,
}

impl Field {
    /// Area, hectares.
    pub fn area_ha(&self) -> f64 {
        self.rect.area_ha()
    }

    /// The yield expected from this field, kilograms per hectare.
    pub fn expected_yield_kg_per_ha(&self, crop: &CropParams, current_year: u32) -> f64 {
        if let Some(last) = self.harvest_records.last() {
            if current_year.saturating_sub(last.year) <= 5 {
                return f64::from(last.yield_kg_per_ha);
            }
        }
        crop.yield_kg_per_ha * f64::from(self.ground)
    }

    /// The day the crop is ripe (a day index), once sown.
    pub fn turn_nitrogen(&mut self, soil: &SoilParams) {
        if self.stage == FieldStage::Fallow {
            let native_fast = (soil.fast_n_kg_per_ha * self.area_ha()) as f32;
            let native_slow = (soil.slow_n_kg_per_ha * self.area_ha()) as f32;
            self.fast_n_kg +=
                (native_fast - self.fast_n_kg).max(0.0_f32) * (soil.decay_fast as f32);
            self.slow_n_kg +=
                (native_slow - self.slow_n_kg).max(0.0_f32) * (soil.decay_slow as f32);
        }

        let decay_f = self.fast_n_kg * (soil.decay_fast as f32);
        let decay_s = self.slow_n_kg * (soil.decay_slow as f32);

        self.fast_n_kg = (self.fast_n_kg - decay_f).max(0.0_f32);
        self.slow_n_kg = (self.slow_n_kg - decay_s).max(0.0_f32);

        // Return some decayed slow N back into fast pool
        self.fast_n_kg += decay_s;
    }

    pub fn ripe_day(&self, crop: &CropParams) -> i64 {
        self.sown_day + i64::from(crop.grow_days)
    }

    /// The task the field needs on `day`, if any.
    pub fn task(&self, crop: &CropParams, day: i64) -> Option<FieldTask> {
        match self.stage {
            FieldStage::Fallow => crop.can_prepare(day).then_some(FieldTask::Prepare),
            FieldStage::Prepared => crop.can_sow(day).then_some(FieldTask::Sow),
            FieldStage::Sown if day >= self.ripe_day(crop) => Some(FieldTask::Reap),
            FieldStage::Sown => (f64::from(self.tended_h) < crop.tend_h_per_ha * self.area_ha())
                .then_some(FieldTask::Tend),
            FieldStage::Reaped => (self.sheaves_kg > 0.0).then_some(FieldTask::Thresh),
        }
    }

    /// Person-hours the current stage needs in all (for threshing, what is left).
    pub fn stage_work_h(&self, crop: &CropParams) -> f64 {
        let ha = self.area_ha();
        match self.stage {
            FieldStage::Fallow if !self.broken => {
                (crop.break_h_per_ha + f64::from(self.clear_h_per_ha)) * ha
            }
            FieldStage::Fallow => crop.prepare_h_per_ha * ha,
            FieldStage::Prepared => crop.sow_h_per_ha * ha,
            FieldStage::Sown => crop.reap_h_per_ha * ha,
            FieldStage::Reaped => crop.thresh_h_per_kg * f64::from(self.sheaves_kg),
        }
    }

    /// The share of the yield the crop's tending so far keeps (weeds take the rest).
    fn tending(&self, crop: &CropParams) -> f64 {
        let needed = crop.tend_h_per_ha * self.area_ha();
        let done = if needed > 0.0 {
            (f64::from(self.tended_h) / needed).clamp(0.0, 1.0)
        } else {
            1.0
        };
        1.0 - crop.untended_loss * (1.0 - done)
    }

    /// The grain the field would give if reaped on `day`, kilograms, when its season's water
    /// allows a share `water` of the yield (1 in a year of average water): its area and ground,
    /// the timeliness of its sowing, its tending, and what the ripe crop has lost standing.
    pub fn yield_kg(&self, crop: &CropParams, day: i64, water: f64) -> f64 {
        let standing = (day - self.ripe_day(crop)).max(0) as f64;
        let kept = (1.0 - crop.standing_loss_per_day * standing).clamp(0.0, 1.0);
        self.area_ha()
            * crop.yield_kg_per_ha
            * f64::from(self.ground)
            * water.max(0.0)
            * crop.timeliness(self.sown_day)
            * self.tending(crop)
            * kept
    }

    /// What people expect the field to give in all if the remaining work is done in time,
    /// kilograms of grain, from what they know on `day`: sowing finishing today (or when the
    /// window opens), the tending still to do done, reaping at ripeness, and a growing crop's
    /// water expected to allow a share `water` of its yield (an average year's, 1, before it is
    /// sown; ADR-0012 §4).
    pub fn expected_kg(&self, crop: &CropParams, day: i64, water: f64) -> f64 {
        let ha = self.area_ha();
        let base = ha * crop.yield_kg_per_ha * f64::from(self.ground);
        match self.stage {
            FieldStage::Fallow | FieldStage::Prepared => {
                if crop.can_prepare(day) {
                    base * crop.timeliness(self.first_sowing_day(crop, day))
                } else {
                    0.0
                }
            }
            FieldStage::Sown => {
                let reap_day = day.max(self.ripe_day(crop));
                let mut f = self.clone();
                f.tended_h = (crop.tend_h_per_ha * ha) as f32;
                f.yield_kg(crop, reap_day, water)
            }
            FieldStage::Reaped => f64::from(self.sheaves_kg),
        }
    }

    /// The first day of this year's sowing window not before `day`.
    fn first_sowing_day(&self, crop: &CropParams, day: i64) -> i64 {
        let doy = CropParams::day_of_year(day);
        day + (i64::from(crop.sow_from_day) - doy).max(0)
    }

    /// Person-hours still needed to bring the field's crop into store, for a harvest of
    /// `expected_kg`.
    pub fn remaining_work_h(&self, crop: &CropParams, expected_kg: f64) -> f64 {
        let ha = self.area_ha();
        let this = (self.stage_work_h(crop) - f64::from(self.work_h)).max(0.0);
        let thresh = crop.thresh_h_per_kg * expected_kg;
        let tend = (crop.tend_h_per_ha * ha - f64::from(self.tended_h)).max(0.0);
        match self.stage {
            FieldStage::Fallow => {
                this + crop.sow_h_per_ha * ha + tend + crop.reap_h_per_ha * ha + thresh
            }
            FieldStage::Prepared => this + tend + crop.reap_h_per_ha * ha + thresh,
            FieldStage::Sown => this + tend + thresh,
            FieldStage::Reaped => this,
        }
    }

    /// Does `hours` of a capable adult's `task` on `day`, its season's water allowing a share
    /// `water` of the yield, with `seed_kg` of seed at hand for sowing. Work beyond what the
    /// stage needs is not counted; sowing stops when the seed runs out.
    #[allow(clippy::too_many_arguments)]
    pub fn work(
        &mut self,
        soil: &SoilParams,
        crop: &CropParams,
        task: FieldTask,
        hours: f64,
        day: i64,
        water: f64,
        seed_kg: f64,
        supplied_kg: f64,
        now: SimTime,
    ) -> WorkDone {
        let mut done = WorkDone::default();
        if task == FieldTask::Manure {
            self.fast_n_kg += supplied_kg as f32;
            done.used_kg = supplied_kg;
            return done;
        }
        if self.task(crop, day) != Some(task) || hours <= 0.0 {
            return done;
        }
        if task == FieldTask::Tend {
            let needed = crop.tend_h_per_ha * self.area_ha();
            self.tended_h = (f64::from(self.tended_h) + hours).min(needed) as f32;
            return done;
        }
        if task == FieldTask::Thresh {
            // The work left is the sheaves left: each hour threshes a fixed amount.
            let kg = (hours / crop.thresh_h_per_kg.max(1e-9)).min(f64::from(self.sheaves_kg));
            done.grain_kg = kg;
            let left = f64::from(self.sheaves_kg) - kg;
            self.sheaves_kg = left.max(0.0) as f32;
            if left <= 1e-6 {
                done.finished = true;
                self.finish_stage(soil, crop, day, water, now);
            }
            return done;
        }
        let needed = self.stage_work_h(crop);
        let before = f64::from(self.work_h);
        let mut after = (before + hours).min(needed);
        if task == FieldTask::Sow {
            // Seed goes in with the work, in proportion.
            let per_hour = crop.seed_kg_per_ha / crop.sow_h_per_ha.max(1e-9);
            let affordable = before + seed_kg.max(0.0) / per_hour.max(1e-12);
            after = after.min(affordable);
            done.seed_kg = (after - before).max(0.0) * per_hour;
        }
        self.work_h = after as f32;
        if after + 1e-6 >= needed {
            done.finished = true;
            self.finish_stage(soil, crop, day, water, now);
        }
        done
    }

    fn finish_stage(
        &mut self,
        soil: &SoilParams,
        crop: &CropParams,
        day: i64,
        water: f64,
        now: SimTime,
    ) {
        self.work_h = 0.0;
        self.stage_since = now;
        self.stage = match self.stage {
            FieldStage::Fallow => {
                self.broken = true;
                FieldStage::Prepared
            }
            FieldStage::Prepared => {
                self.sown_day = day;
                self.tended_h = 0.0;
                (self.need_mm, self.got_mm) = (0.0, 0.0);
                FieldStage::Sown
            }
            FieldStage::Sown => {
                let water_yield = self.yield_kg(crop, day, water);

                let n_supply = (self.fast_n_kg * soil.decay_fast as f32
                    + self.slow_n_kg * soil.decay_slow as f32)
                    as f64;
                let n_yield = if crop.n_kg_per_kg_grain > 0.0 {
                    n_supply / crop.n_kg_per_kg_grain
                } else {
                    f64::INFINITY
                };

                let actual_yield = water_yield.min(n_yield);

                let limit = if actual_yield == 0.0 {
                    HarvestLimit::None
                } else if actual_yield < water_yield {
                    HarvestLimit::Nitrogen
                } else if water_yield < crop.yield_kg_per_ha * self.area_ha() * 0.99 {
                    HarvestLimit::Water
                } else {
                    HarvestLimit::None
                };

                self.sheaves_kg = actual_yield as f32;
                self.harvests = self.harvests.saturating_add(1);

                self.harvest_records.push(HarvestRecord {
                    year: (day / 365) as u32,
                    yield_kg_per_ha: (actual_yield / self.area_ha()) as f32,
                    limit,
                });

                // take away nitrogen
                let n_taken = (actual_yield * crop.n_kg_per_kg_grain
                    + actual_yield * crop.n_kg_per_kg_straw) as f32;
                self.fast_n_kg = (self.fast_n_kg - n_taken).max(0.0_f32);

                // roots and stubble return some
                self.slow_n_kg += crop.roots_n_kg_per_ha as f32 * self.area_ha() as f32;

                FieldStage::Reaped
            }
            FieldStage::Reaped => {
                self.sheaves_kg = 0.0;
                FieldStage::Fallow
            }
        };
    }

    /// The turn of the day: a prepared field whose sowing window has passed, or a ripe crop
    /// left standing until it is all lost, falls back to fallow, its work lost. Returns whether
    /// the field changed stage.
    pub fn new_day(
        &mut self,
        _soil: &SoilParams,
        crop: &CropParams,
        day: i64,
        now: SimTime,
    ) -> bool {
        let lapsed = match self.stage {
            FieldStage::Prepared => !crop.can_prepare(day),
            FieldStage::Sown => {
                let standing = (day - self.ripe_day(crop)) as f64;
                standing * crop.standing_loss_per_day >= 1.0
            }
            FieldStage::Fallow | FieldStage::Reaped => false,
        };
        if lapsed {
            self.stage = FieldStage::Fallow;
            self.work_h = 0.0;
            self.tended_h = 0.0;
            self.stage_since = now;
        }
        lapsed
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn soil() -> SoilParams {
        SoilParams {
            fast_n_kg_per_ha: 100.0,
            slow_n_kg_per_ha: 2000.0,
            decay_fast: 0.1,
            decay_slow: 0.02,
            soil_water_mm: 100.0,
            easy_water_share: 0.5,
        }
    }

    pub(crate) fn crop() -> CropParams {
        CropParams {
            id: "core:crop/barley".into(),
            name: "Barley".into(),
            good: 0,
            seed_good: 1,
            seed_kg_per_ha: 100.0,
            yield_kg_per_ha: 1000.0,
            prepare_from_day: 50,
            sow_from_day: 80,
            sow_until_day: 125,
            late_sowing_loss_per_day: 0.01,
            grow_days: 120,
            standing_loss_per_day: 0.02,
            untended_loss: 0.4,
            break_h_per_ha: 800.0,
            prepare_h_per_ha: 400.0,
            sow_h_per_ha: 50.0,
            tend_h_per_ha: 200.0,
            reap_h_per_ha: 250.0,
            thresh_h_per_kg: 0.5,
            straw: None,
            kc: [0.4, 1.15, 0.4],
            kc_days: [30, 30, 40, 20],
            ky: 1.15,
            n_kg_per_kg_grain: 0.02,
            n_kg_per_kg_straw: 0.005,
            roots_n_kg_per_ha: 15.0,
        }
    }

    pub(crate) fn field() -> Field {
        Field {
            id: PermanentId::from_raw(7).expect("non-zero"),
            fast_n_kg: 10.0,
            slow_n_kg: 200.0,
            harvest_records: Vec::new(),
            household: PermanentId::from_raw(3).expect("non-zero"),
            holder: Party::Household(PermanentId::from_raw(3).expect("non-zero")),
            lease: None,
            // 50 m x 20 m = 0.1 ha.
            rect: RectCm {
                x: 0,
                y: 0,
                w: 5000,
                h: 2000,
            },
            crop: 0,
            stage: FieldStage::Fallow,
            stage_since: SimTime::ZERO,
            work_h: 0.0,
            tended_h: 0.0,
            ground: 1.0,
            clear_h_per_ha: 0.0,
            broken: false,
            sown_day: 0,
            sheaves_kg: 0.0,
            harvests: 0,
            water_mm: 0.0,
            need_mm: 0.0,
            got_mm: 0.0,
        }
    }

    #[test]
    fn woodland_is_cleared_before_it_is_first_broken_and_broken_ground_stays_broken() {
        let c = crop();
        let mut f = field();
        f.clear_h_per_ha = 1200.0;
        // 0.1 ha: 80 h to break and 120 h to clear.
        assert!((f.stage_work_h(&c) - 200.0).abs() < 1e-9);
        assert!(
            f.work(
                &soil(),
                &c,
                FieldTask::Prepare,
                200.0,
                60,
                1.0,
                0.0,
                0.0,
                SimTime::ZERO
            )
            .finished
        );
        assert!(f.broken);
        // Prepared but never sown, it lies fallow again: still broken ground.
        assert!(f.new_day(&soil(), &c, 126, SimTime::ZERO));
        assert!((f.stage_work_h(&c) - 40.0).abs() < 1e-9);
    }

    #[test]
    fn a_crop_goes_from_new_ground_to_grain_in_store() {
        let c = crop();
        let mut f = field();
        let t = SimTime::ZERO;
        assert_eq!(f.task(&c, 40), None, "too early to prepare");
        assert_eq!(f.task(&c, 60), Some(FieldTask::Prepare));
        // Breaking new ground: 80 h for 0.1 ha.
        let done = f.work(&soil(), &c, FieldTask::Prepare, 50.0, 60, 1.0, 0.0, 0.0, t);
        assert!(!done.finished && f.stage == FieldStage::Fallow);
        assert!(
            f.work(&soil(), &c, FieldTask::Prepare, 50.0, 61, 1.0, 0.0, 0.0, t)
                .finished
        );
        assert_eq!(f.stage, FieldStage::Prepared);
        assert_eq!(f.task(&c, 70), None, "not yet time to sow");
        // Sowing needs 10 kg of seed; with 4 kg at hand it goes 40 % of the way.
        let done = f.work(&soil(), &c, FieldTask::Sow, 10.0, 80, 1.0, 4.0, 0.0, t);
        assert!((done.seed_kg - 4.0).abs() < 1e-9 && !done.finished);
        let done = f.work(&soil(), &c, FieldTask::Sow, 10.0, 80, 1.0, 50.0, 0.0, t);
        assert!((done.seed_kg - 6.0).abs() < 1e-9 && done.finished);
        assert_eq!((f.stage, f.sown_day), (FieldStage::Sown, 80));
        // Tend fully, then reap at ripeness: the full yield of 100 kg.
        assert_eq!(f.task(&c, 100), Some(FieldTask::Tend));
        f.work(&soil(), &c, FieldTask::Tend, 100.0, 100, 1.0, 0.0, 0.0, t);
        assert_eq!(f.task(&c, 150), None, "tended; not ripe yet");
        assert_eq!(f.task(&c, 200), Some(FieldTask::Reap));
        assert!((f.expected_kg(&c, 200, 1.0) - 100.0).abs() < 1e-6);
        // A season short of water is expected to give less.
        assert!((f.expected_kg(&c, 200, 0.8) - 80.0).abs() < 1e-6);
        assert!(
            f.work(&soil(), &c, FieldTask::Reap, 25.0, 200, 1.0, 0.0, 0.0, t)
                .finished
        );
        assert!((f64::from(f.sheaves_kg) - 100.0).abs() < 1e-4);
        // Threshing: half an hour a kilogram.
        let done = f.work(&soil(), &c, FieldTask::Thresh, 20.0, 205, 1.0, 0.0, 0.0, t);
        assert!((done.grain_kg - 40.0).abs() < 1e-4);
        let done = f.work(&soil(), &c, FieldTask::Thresh, 100.0, 206, 1.0, 0.0, 0.0, t);
        assert!((done.grain_kg - 60.0).abs() < 1e-3 && done.finished);
        assert_eq!(f.stage, FieldStage::Fallow);
        // Next year it is prepared again, at the lighter work of cropped ground.
        assert!((f.stage_work_h(&c) - 40.0).abs() < 1e-9);
    }

    #[test]
    fn late_sowing_neglect_and_a_crop_left_standing_cost_grain() {
        let c = crop();
        let mut f = field();
        f.stage = FieldStage::Sown;
        f.sown_day = 100; // 20 days late: 20 % lost
        let ripe = f.ripe_day(&c);
        // Untended: 40 % lost on top.
        let y = f.yield_kg(&c, ripe, 1.0);
        assert!((y - 100.0 * 0.8 * 0.6).abs() < 1e-6, "{y}");
        // Ten days standing: 20 % more.
        let late = f.yield_kg(&c, ripe + 10, 1.0);
        assert!((late - y * 0.8).abs() < 1e-6);
        // A season with half the water an average one allows halves it.
        assert!((f.yield_kg(&c, ripe, 0.5) - y / 2.0).abs() < 1e-6);
        // Poorer ground gives less.
        let mut poor = f.clone();
        poor.ground = 0.75;
        assert!((poor.yield_kg(&c, ripe, 1.0) - 0.75 * y).abs() < 1e-6);
        // Left standing until it is all gone, the field lies fallow.
        assert!(!f.new_day(&soil(), &c, ripe + 10, SimTime::ZERO));
        assert!(f.new_day(&soil(), &c, ripe + 50, SimTime::ZERO));
        assert_eq!(f.stage, FieldStage::Fallow);
    }

    #[test]
    fn a_prepared_field_not_sown_in_its_window_lies_fallow_again() {
        let c = crop();
        let mut p = field();
        p.stage = FieldStage::Prepared;
        assert!(!p.new_day(&soil(), &c, 120, SimTime::ZERO));
        assert!(p.new_day(&soil(), &c, 126, SimTime::ZERO));
        assert_eq!(p.stage, FieldStage::Fallow);
    }

    #[test]
    fn rectangles_measure_and_keep_their_distance() {
        let a = RectCm {
            x: 0,
            y: 0,
            w: 10_000,
            h: 5_000,
        };
        assert!((a.area_ha() - 0.5).abs() < 1e-12);
        assert_eq!(a.centre_m(), (50.0, 25.0));
        let b = RectCm {
            x: 10_300,
            y: 0,
            w: 1_000,
            h: 1_000,
        };
        assert!(!a.near(&b, 200), "3 m apart");
        assert!(a.near(&b, 400), "within 4 m");
    }
}
