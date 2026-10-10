//! Farming (plan §7 M1): what a household plans to grow, which of its fields needs which work and
//! what that work is worth, where it marks out new ground, and what field work does to its
//! stores. The research's planning order (08-02 §10): keep the seed, estimate needs, and check
//! each labour window; the area sown is the least of the land, the seed and the labour
//! (08-20 §1.1).

use civ_core::{PermanentId, Rng64};
use civ_land::{CropParams, Field, FieldStage, FieldTask, Land, LandParams, RectCm};
use civ_world::nav::{NavGrid, TravelField};
use civ_world::{WATER_LAND, WorldMap};

use crate::decide::FieldOption;
use crate::history::Reason;
use crate::params::PeopleParams;

/// Purpose tag for choosing new ground (ADR-0003 keyed randomness).
pub const PURPOSE_SITE: u64 = 0x7369_7465_3030_3031; // "site0001"
/// Least distance between a new field and another field, centimetres.
const FIELD_GAP_CM: i32 = 200;
/// Least distance between a new field and a home or the hearth, metres.
const HOME_GAP_M: f32 = 15.0;
/// Years over which new ground is weighed: clearing and breaking it is paid once, its crops come
/// every year (research 10-01 §2.3: households plan two to five years ahead).
const PLAN_YEARS: f64 = 3.0;
/// Days from a crop ripening until reaping and threshing bring its first grain in (a tuning
/// value).
const HARVEST_IN_DAYS: f64 = 7.0;

/// What a household knows of its farming on one day.
#[derive(Clone, Debug)]
pub struct FarmView<'a> {
    /// The crop it grows.
    pub crop: &'a CropParams,
    /// Food energy of the crop's grain, kcal per kilogram.
    pub kcal_per_kg: f64,
    /// Its fields.
    pub fields: Vec<&'a Field>,
    /// The day.
    pub day: i64,
    /// Hours of a capable adult's field work the household can give a day.
    pub labour_per_day: f64,
    /// Seed in store, kilograms.
    pub seed_kg: f64,
    /// How much more grain is worth to it, 0–1.
    pub room: f64,
    /// The grain it means to grow for its needs, kilograms ([`need_grain_kg`]).
    pub need_kg: f64,
    /// The share of what it expects of a field it counts on when it plans (research 10-01 §2.3:
    /// plan on a poor harvest).
    pub plan_yield_share: f64,
    /// The share of the days of the sowing window the ground can usually be worked (ADR-0012
    /// §5): how pressing the work left is, of the days left.
    pub workable_share: f64,
    /// Field work a capable adult gives on a day at a peak, against an ordinary day's: what makes
    /// up the days the weather takes when it plans spring work.
    pub peak_ratio: f64,
    /// What it knows of the weather's years, to judge a growing crop by the water its season so
    /// far has brought (ADR-0012 §4); `None` judges every crop by an average year.
    pub climatology: Option<&'a civ_land::Climatology>,
    /// Its midden and what dung is known to bring; `None` when it has none.
    pub manuring: Option<Manuring<'a>>,
}

/// What a household knows of dunging its fields from its midden (M3c slice V).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Manuring<'a> {
    /// Kilograms in its midden.
    pub midden_kg: f64,
    /// Grain a kilogram of dung is known to add to the next harvest of ground that gives less
    /// than new ground of its kind, kilograms: the nitrogen in it the year's crop can draw on,
    /// over what the crop takes up for a kilogram of grain. What people know of manuring, not of
    /// any field's soil (ADR-0012 §4).
    pub grain_per_kg: f64,
    /// How the midden is carried.
    pub midden: &'a crate::params::MiddenParams,
}

/// The grain a household of `members` means to grow in a year for its needs, kilograms: the
/// share of a year's food it grows, with what is lost before it is eaten, of grain of `kcal` a
/// kilogram.
pub fn need_grain_kg(members: usize, params: &PeopleParams, kcal: f64) -> f64 {
    let year = members as f64 * params.household.daily_kcal_per_person * 365.0;
    year * params.farm.grain_share / (1.0 - params.farm.loss_share).max(0.05) / kcal.max(1.0)
}

/// The area a household of `members` plans to crop for its needs, hectares: its grain
/// ([`need_grain_kg`]) at a cautious share of the yield it expects, `yield_kg_ha`, less the seed
/// (research 10-01 §2.3: plan on a poor harvest, keep seed apart).
pub fn need_area_ha(
    members: usize,
    params: &PeopleParams,
    crop: &CropParams,
    kcal: f64,
    yield_kg_ha: f64,
) -> f64 {
    let net = yield_kg_ha * params.farm.plan_yield_share - crop.seed_kg_per_ha;
    need_grain_kg(members, params, kcal) / net.max(1.0)
}

/// The yield a household expects of its fields on `day`, kilograms a hectare: what each has
/// given ([`Field::expected_kg_per_ha`]) weighted by its area, or with no fields what the crop
/// gives on average ground (ADR-0012 §4).
pub fn expected_yield_kg_ha<'a>(
    fields: impl Iterator<Item = &'a Field>,
    crop: &CropParams,
    day: i64,
) -> f64 {
    let year = civ_land::calendar_year(day);
    let (mut kg, mut ha) = (0.0, 0.0);
    for f in fields {
        kg += f.expected_kg_per_ha(crop, year) * f.area_ha();
        ha += f.area_ha();
    }
    if ha > 0.0 {
        kg / ha
    } else {
        crop.yield_kg_per_ha
    }
}

/// Seed a household never eats, `(good, kg)`: what it takes to sow again the ground it already
/// crops (research 08-02 §10: protect a minimum viable seed reserve first). Seed kept beyond it,
/// for new ground, can be eaten in a crisis (§2.3). `None` when it crops no ground yet.
pub fn protected_seed<'a>(
    fields: impl Iterator<Item = &'a Field>,
    crops: &[CropParams],
) -> Option<(usize, f64)> {
    let mut out: Option<(usize, f64)> = None;
    for f in fields {
        let Some(crop) = crops.get(usize::from(f.crop)) else {
            continue;
        };
        let kg = f.area_ha() * crop.seed_kg_per_ha;
        match &mut out {
            Some((good, total)) if *good == crop.seed_good => *total += kg,
            Some(_) => {}
            None => out = Some((crop.seed_good, kg)),
        }
    }
    out
}

/// The area a household plans to sow in the coming season, hectares: what it needs, as far as
/// its labour can prepare and sow in one sowing season, and never less than the fields it already
/// crops (research 08-20 §1.1: the area sown is the least of the land, the seed and the labour).
/// `plan_share` is the share of the season's days of ordinary work it counts on ([`plan_share`]).
pub fn plan_area_ha(
    need_ha: f64,
    held_ha: f64,
    labour_per_day: f64,
    plan_share: f64,
    crop: &CropParams,
) -> f64 {
    let season = f64::from(crop.sow_until_day.saturating_sub(crop.prepare_from_day)) + 1.0;
    let per_ha = (crop.prepare_h_per_ha + crop.sow_h_per_ha).max(1e-6);
    need_ha
        .min(labour_per_day * season * plan_share.clamp(0.0, 1.0) / per_ha)
        .max(held_ha)
}

/// The share of a sowing season's days of ordinary field work a household counts on when it
/// plans: the days the ground can usually be worked (`workable_share`), each worked at a peak's
/// longer hours (`peak_ratio` of an ordinary day's), and never more than every day's ordinary
/// work (ADR-0012 §5; research 04-02 §2.4: 6 hours ordinary, 10 at a peak; 08-02: the area is
/// what can be done within workable windows).
pub fn plan_share(workable_share: f64, peak_ratio: f64) -> f64 {
    (workable_share.clamp(0.0, 1.0) * peak_ratio.max(1.0)).min(1.0)
}

/// Days from `day` until a household's next harvest begins to come in: its first sown field
/// ripening, or with none sown the crop's calendar (sown at the start of the next sowing window it
/// can still use, ripe `grow_days` later), and then the days reaping and threshing take to bring
/// the first grain in. Sheaves waiting to be threshed are grain within days.
pub fn days_to_harvest<'a>(
    crop: &CropParams,
    fields: impl Iterator<Item = &'a Field>,
    day: i64,
) -> f64 {
    let mut next: Option<f64> = None;
    for f in fields {
        let days = match f.stage {
            FieldStage::Sown => (f.ripe_day(crop) - day).max(0) as f64 + HARVEST_IN_DAYS,
            FieldStage::Reaped => HARVEST_IN_DAYS,
            FieldStage::Fallow | FieldStage::Prepared => continue,
        };
        next = Some(next.map_or(days, |n: f64| n.min(days)));
    }
    next.unwrap_or_else(|| {
        let year = civ_core::time::DAYS_PER_YEAR;
        let doy = day.rem_euclid(year);
        let sow = if doy <= i64::from(crop.sow_until_day) {
            doy.max(i64::from(crop.sow_from_day))
        } else {
            i64::from(crop.sow_from_day) + year
        };
        (sow - doy) as f64 + f64::from(crop.grow_days) + HARVEST_IN_DAYS
    })
}

/// Seed a household keeps on `day` to sow `plan_ha` hectares: the sowing rate, and what the seed
/// loses in store until sowing begins (research 08-01 §1.5, 08-02 §10: seed is set aside by the
/// planned area before anything is eaten). `half_life_days` is the seed good's (0 = keeps).
pub fn seed_to_keep(plan_ha: f64, crop: &CropParams, half_life_days: f64, day: i64) -> f64 {
    let doy = day.rem_euclid(civ_core::time::DAYS_PER_YEAR);
    let wait = (i64::from(crop.sow_from_day) - doy).rem_euclid(civ_core::time::DAYS_PER_YEAR);
    let loss = if half_life_days > 0.0 {
        (wait as f64 / half_life_days).exp2()
    } else {
        1.0
    };
    plan_ha * crop.seed_kg_per_ha * loss
}

/// Days from `day` to the last day of the crop's sowing window, counting both (0 once past).
fn sowing_days_left(crop: &CropParams, day: i64) -> f64 {
    let doy = day.rem_euclid(civ_core::time::DAYS_PER_YEAR);
    (i64::from(crop.sow_until_day) - doy + 1).max(0) as f64
}

/// Days from `day` until the crop's next window for preparing ground opens (0 within it).
fn days_to_window(crop: &CropParams, day: i64) -> f64 {
    if crop.can_prepare(day) {
        return 0.0;
    }
    let doy = day.rem_euclid(civ_core::time::DAYS_PER_YEAR);
    (i64::from(crop.prepare_from_day) - doy).rem_euclid(civ_core::time::DAYS_PER_YEAR) as f64
}

/// Days in the crop's window for preparing and sowing ground.
fn window_days(crop: &CropParams) -> f64 {
    f64::from(crop.sow_until_day.saturating_sub(crop.prepare_from_day)) + 1.0
}

/// Hours a field still needs before it is sown (0 once sown).
fn to_sow_h(f: &Field, crop: &CropParams) -> f64 {
    let left = (f.stage_work_h(crop) - f64::from(f.work_h)).max(0.0);
    match f.stage {
        FieldStage::Fallow => left + crop.sow_h_per_ha * f.area_ha(),
        FieldStage::Prepared => left,
        FieldStage::Sown | FieldStage::Reaped => 0.0,
    }
}

impl<'a> FarmView<'a> {
    /// The grain field `f` is expected to give a hectare beyond its seed, as the household plans
    /// it: a cautious share of what it has given ([`Field::expected_kg_per_ha`]).
    fn net_kg_ha(&self, f: &Field) -> f64 {
        let expected = f.expected_kg_per_ha(self.crop, civ_land::calendar_year(self.day));
        (expected * self.plan_yield_share - self.crop.seed_kg_per_ha).max(0.0)
    }

    /// The fields the household means to crop in the coming season (ADR-0012 §4): those
    /// prepared, sown or begun, then the others, best first by the grain each is expected to
    /// give for the work it needs before it is sown, until they are expected to grow what the
    /// household needs. The fields it can spare rest (research 08-02 §6: the land a household
    /// holds is not the land it sows).
    pub fn cropping(&self) -> Vec<&'a Field> {
        let crop = self.crop;
        let mut chosen: Vec<&'a Field> = Vec::new();
        let mut spare: Vec<(f64, &'a Field)> = Vec::new();
        for &f in &self.fields {
            let begun = matches!(f.stage, FieldStage::Prepared | FieldStage::Sown)
                || (f.stage == FieldStage::Fallow && f.work_h > 0.0);
            if begun {
                chosen.push(f);
            } else {
                let kg = self.net_kg_ha(f) * f.area_ha();
                spare.push((kg / to_sow_h(f, crop).max(1e-6), f));
            }
        }
        let mut grown: f64 = chosen.iter().map(|f| self.net_kg_ha(f) * f.area_ha()).sum();
        spare.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.id.cmp(&b.1.id)));
        for (_, f) in spare {
            if grown + 1e-9 >= self.need_kg {
                break;
            }
            grown += self.net_kg_ha(f) * f.area_ha();
            chosen.push(f);
        }
        chosen
    }

    /// Whether the household would mark out a new field of `ha` hectares, needing
    /// `clear_h_per_ha` of clearing, today: all its fields cropped would not grow what it
    /// needs, it has the seed for the new ground and every field it means to crop, and it can
    /// work them all in time. In the sowing season that is clearing, breaking and sowing the new
    /// ground and the rest before the window closes. Out of it (ADR-0012 §4: break ground outside
    /// the sowing season) it is clearing and breaking the new ground before the next window
    /// opens, and preparing and sowing everything in that window.
    pub fn wants_new_field(&self, ha: f64, clear_h_per_ha: f64) -> bool {
        let crop = self.crop;
        let all: f64 = self
            .fields
            .iter()
            .map(|f| self.net_kg_ha(f) * f.area_ha())
            .sum();
        if all + 1e-9 >= self.need_kg {
            return false;
        }
        let cropping = self.cropping();
        let share = plan_share(self.workable_share, self.peak_ratio);
        if crop.can_prepare(self.day) {
            let unsown: f64 = cropping
                .iter()
                .filter(|f| matches!(f.stage, FieldStage::Fallow | FieldStage::Prepared))
                .map(|f| f.area_ha())
                .sum();
            if self.seed_kg + 1e-9 < (unsown + ha) * crop.seed_kg_per_ha {
                return false;
            }
            let left: f64 = cropping.iter().map(|f| to_sow_h(f, crop)).sum();
            let new = ha * (crop.break_h_per_ha + clear_h_per_ha + crop.sow_h_per_ha);
            return left + new <= self.labour_per_day * sowing_days_left(crop, self.day) * share;
        }
        let next: f64 = cropping.iter().map(|f| f.area_ha()).sum::<f64>() + ha;
        if self.seed_kg + 1e-9 < next * crop.seed_kg_per_ha {
            return false;
        }
        let breaking: f64 = cropping
            .iter()
            .filter(|f| !f.broken)
            .map(|f| (f.stage_work_h(crop) - f64::from(f.work_h)).max(0.0))
            .sum::<f64>()
            + ha * (crop.break_h_per_ha + clear_h_per_ha);
        let spring = next * (crop.prepare_h_per_ha + crop.sow_h_per_ha);
        breaking <= self.labour_per_day * days_to_window(crop, self.day) * share
            && spring <= self.labour_per_day * window_days(crop) * share
    }

    /// Of `days` of the sowing window, those the ground can usually be worked.
    fn workable_days(&self, days: f64) -> f64 {
        days * self.workable_share.clamp(0.0, 1.0)
    }

    /// Field work left for `task` over the work the household can still give it before its
    /// season closes (research 04-02 §1.3: remaining person-hours over available hours before
    /// the deadline). Threshing has no deadline.
    pub fn urgency(&self, task: FieldTask) -> f64 {
        let crop = self.crop;
        let day = self.day;
        let (work, days): (f64, f64) = match task {
            // Ground broken out of the season has no deadline yet.
            FieldTask::Prepare | FieldTask::Sow if !crop.can_prepare(day) => (0.0, f64::INFINITY),
            FieldTask::Prepare | FieldTask::Sow => {
                let work = self
                    .cropping()
                    .iter()
                    .filter(|f| f.task(crop, day).is_some())
                    .map(|f| to_sow_h(f, crop))
                    .sum();
                (work, self.workable_days(sowing_days_left(crop, day)))
            }
            FieldTask::Tend => {
                let mut work = 0.0;
                let mut days = f64::INFINITY;
                for f in &self.fields {
                    if f.task(crop, day) == Some(FieldTask::Tend) {
                        let needed = crop.tend_h_per_ha * f.area_ha();
                        work += (needed - f64::from(f.tended_h)).max(0.0);
                        days = days.min((f.ripe_day(crop) - day).max(1) as f64);
                    }
                }
                (work, days)
            }
            FieldTask::Reap => {
                // Reaping is due before a quarter of the ripe crop is lost standing.
                let grace = 0.25 / crop.standing_loss_per_day.max(1e-6);
                let mut work = 0.0;
                let mut days = f64::INFINITY;
                for f in &self.fields {
                    if f.task(crop, day) == Some(FieldTask::Reap) {
                        work += (f.stage_work_h(crop) - f64::from(f.work_h)).max(0.0);
                        let standing = (day - f.ripe_day(crop)) as f64;
                        days = days.min((grace - standing).max(1.0));
                    }
                }
                (work, days)
            }
            FieldTask::Thresh | FieldTask::Manure => (0.0, f64::INFINITY),
        };
        if work <= 0.0 || !days.is_finite() {
            return 0.0;
        }
        work / (self.labour_per_day * days).max(1e-6)
    }

    /// What `task` on field `f` is worth: the food its expected harvest brings per hour of the
    /// work it still needs, and the hours this task still needs.
    fn worth(&self, f: &Field, task: FieldTask) -> (f64, f64) {
        let crop = self.crop;
        let water = match (self.climatology, f.stage) {
            (Some(c), FieldStage::Sown) => c.expected_crop_factor(
                usize::from(f.crop),
                f64::from(f.need_mm),
                f64::from(f.got_mm),
            ),
            _ => 1.0,
        };
        let expected = f.expected_kg(crop, self.day, water);
        let remaining = f.remaining_work_h(crop, self.day, expected);
        let per_hour = expected * self.kcal_per_kg / remaining.max(1e-6);
        let left = match task {
            FieldTask::Tend => (crop.tend_h_per_ha * f.area_ha() - f64::from(f.tended_h)).max(0.0),
            _ => (f.stage_work_h(crop) - f64::from(f.work_h)).max(0.0),
        };
        (per_hour, left)
    }

    /// The best field to dung from the midden today (M3c slice V): among the fields it means to
    /// crop that lie broken or dug for sowing and have given less than new ground of their kind
    /// ([`Field::expected_kg_per_ha`]), the most grain an hour of the work brings, the walk there
    /// and back with each load counted. Dung is carried while it can make up the shortfall, and
    /// once the heap holds an hour's carrying at least.
    fn best_to_manure(
        &self,
        walk_min: &dyn Fn(&Field) -> Option<f64>,
    ) -> Result<FieldOption, Reason> {
        let crop = self.crop;
        let Some(m) = self.manuring else {
            return Err(Reason::NoFieldWork);
        };
        if m.grain_per_kg <= 0.0 || m.midden_kg < m.midden.load_kg {
            return Err(Reason::NoFieldWork);
        }
        let year = civ_land::calendar_year(self.day);
        let mut best: Option<FieldOption> = None;
        for f in self.cropping() {
            let ready =
                (f.stage == FieldStage::Fallow && f.broken) || f.stage == FieldStage::Prepared;
            if !ready {
                continue;
            }
            let new_ground = crop.yield_kg_per_ha * f64::from(f.ground);
            let short_kg = (new_ground - f.expected_kg_per_ha(crop, year)).max(0.0) * f.area_ha();
            let usable = m.midden_kg.min(short_kg / m.grain_per_kg);
            let Some(walk) = walk_min(f) else {
                continue;
            };
            let h_per_kg = m.midden.h_per_kg(walk).max(1e-9);
            // A session's work is an hour at least: the heap waits until it holds that much.
            if usable < m.midden.load_kg.max(1.0 / h_per_kg) {
                continue;
            }
            let option = FieldOption {
                field: Some(f.id),
                walk_min: walk,
                at: f.rect.centre_m(),
                kcal_per_hour: m.grain_per_kg * self.kcal_per_kg / h_per_kg,
                hours_left: usable * h_per_kg,
                room: self.room,
                urgency: 0.0,
                at_home: false,
                soon: false,
            };
            if best
                .as_ref()
                .is_none_or(|b| option.kcal_per_hour > b.kcal_per_hour)
            {
                best = Some(option);
            }
        }
        best.ok_or(Reason::NoFieldWork)
    }

    /// The best field for `task` today, among the household's fields and, for preparing, new
    /// ground at `site`: the most food per hour once the walk there and back is counted for a
    /// session of `session_h` hours. `walk_min` gives the walk to a field from home, and
    /// `weather` why the ground at a place cannot be worked today, if it cannot (ADR-0012 §5):
    /// only work that turns the soil waits for it ([`FieldTask::turns_soil`]).
    pub fn best(
        &self,
        task: FieldTask,
        session_h: f64,
        site: Option<&Site>,
        walk_min: &dyn Fn(&Field) -> Option<f64>,
        weather: &dyn Fn(&RectCm) -> Option<Reason>,
    ) -> Result<FieldOption, Reason> {
        let crop = self.crop;
        let mut any = false;
        let mut held_back: Option<Reason> = None;
        let mut best: Option<(f64, FieldOption)> = None;
        if task == FieldTask::Manure {
            return self.best_to_manure(walk_min);
        }
        // Ground is prepared and sown only where the household means to crop; the rest rests.
        let cropping: Vec<PermanentId> = match task {
            FieldTask::Prepare | FieldTask::Sow => self.cropping().iter().map(|f| f.id).collect(),
            _ => Vec::new(),
        };
        let consider = |option: FieldOption, best: &mut Option<(f64, FieldOption)>| {
            let walk_h = if option.at_home {
                0.0
            } else {
                2.0 * option.walk_min / 60.0
            };
            let value = option.kcal_per_hour * session_h / (session_h + walk_h);
            if best.is_none_or(|(v, _)| value > v) {
                *best = Some((value, option));
            }
        };
        for f in &self.fields {
            if f.task(crop, self.day) != Some(task) {
                continue;
            }
            if matches!(task, FieldTask::Prepare | FieldTask::Sow) && !cropping.contains(&f.id) {
                continue;
            }
            any = true;
            if task == FieldTask::Sow && self.seed_kg <= 1e-6 {
                continue;
            }
            let at_home = task == FieldTask::Thresh;
            if task.turns_soil()
                && let Some(why) = weather(&f.rect)
            {
                held_back.get_or_insert(why);
                continue;
            }
            let walk = if at_home { Some(0.0) } else { walk_min(f) };
            let Some(walk) = walk else {
                continue;
            };
            let (per_hour, left) = self.worth(f, task);
            consider(
                FieldOption {
                    field: Some(f.id),
                    walk_min: walk,
                    at: f.rect.centre_m(),
                    kcal_per_hour: per_hour,
                    hours_left: left,
                    room: self.room,
                    urgency: self.urgency(task),
                    at_home,
                    soon: matches!(task, FieldTask::Reap | FieldTask::Thresh),
                },
                &mut best,
            );
        }
        let site = site.filter(|s| match weather(&s.rect) {
            Some(why) if task == FieldTask::Prepare => {
                held_back.get_or_insert(why);
                false
            }
            _ => true,
        });
        if task == FieldTask::Prepare
            && let Some(site) = site
            && self.wants_new_field(site.rect.area_ha(), site.clear_h_per_ha)
        {
            any = true;
            let ground = Field {
                id: site.placeholder,
                household: site.placeholder,
                holder: civ_land::Party::Household(site.placeholder),
                lease: None,
                rect: site.rect,
                crop: 0,
                stage: FieldStage::Fallow,
                stage_since: civ_core::SimTime::ZERO,
                work_h: 0.0,
                tended_h: 0.0,
                ground: site.ground as f32,
                clear_h_per_ha: site.clear_h_per_ha as f32,
                broken: false,
                sown_day: 0,
                sheaves_kg: 0.0,
                harvests: 0,
                water_mm: 0.0,
                need_mm: 0.0,
                got_mm: 0.0,
                soil: civ_land::FieldSoil::default(),
            };
            let (per_hour, left) = self.worth(&ground, task);
            consider(
                FieldOption {
                    field: None,
                    walk_min: site.walk_min,
                    at: site.rect.centre_m(),
                    kcal_per_hour: per_hour,
                    hours_left: left,
                    room: self.room,
                    urgency: self.urgency(task),
                    at_home: false,
                    soon: false,
                },
                &mut best,
            );
        }
        match best {
            Some((_, option)) => Ok(option),
            None if let Some(why) = held_back => Err(why),
            None if task == FieldTask::Sow && any => Err(Reason::NoSeed),
            None if task == FieldTask::Prepare && self.fields.is_empty() => Err(Reason::NoPlace),
            None => Err(Reason::NoFieldWork),
        }
    }
}

/// New ground a household could mark out as a field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Site {
    /// Where.
    pub rect: RectCm,
    /// Walk there from home, minutes.
    pub walk_min: f64,
    /// How good its ground is (the mean richness of its patches).
    pub ground: f64,
    /// Work to clear it before it is first broken, person-hours per hectare (the mean over its
    /// ground).
    pub clear_h_per_ha: f64,
    /// Any id, for the placeholder field the option is valued with.
    pub placeholder: PermanentId,
}

/// What new ground of quality `ground` that needs `clear_h_per_ha` of clearing is worth, a walk
/// of `walk_min` from home: grain over the work it takes across a few years (clearing and
/// breaking once, then the yearly round), less for the walk to a working session.
pub fn site_value(crop: &CropParams, ground: f64, clear_h_per_ha: f64, walk_min: f64) -> f64 {
    let grain = crop.yield_kg_per_ha * ground;
    let yearly = crop.prepare_h_per_ha
        + crop.sow_h_per_ha
        + crop.tend_h_per_ha
        + crop.reap_h_per_ha
        + crop.thresh_h_per_kg * grain;
    let work = crop.break_h_per_ha + clear_h_per_ha - crop.prepare_h_per_ha + PLAN_YEARS * yearly;
    let session_h = 4.0;
    PLAN_YEARS * grain / work.max(1e-6) * session_h / (session_h + 2.0 * walk_min / 60.0)
}

/// Whether new ground could be broken on `cell`: dry, walkable and arable.
fn breakable(
    map: &WorldMap,
    nav: &NavGrid,
    land: &Land,
    land_params: &LandParams,
    cell: usize,
) -> bool {
    let p = land.patches.of_cell(cell, map.width);
    map.water[cell] == WATER_LAND
        && nav.walkable(cell)
        && land
            .patches
            .class
            .get(p)
            .and_then(|&c| land_params.habitats.get(usize::from(c)))
            .is_some_and(|h| h.arable)
}

/// The cells within a field walk of a home (`reach`, its walking times) where new ground could be
/// broken, with those times, in cell order: what [`find_site`] picks from. The ground does not
/// change, so this is worked out once for each home's walking times.
pub fn breakable_in_reach(
    map: &WorldMap,
    nav: &NavGrid,
    land: &Land,
    land_params: &LandParams,
    reach: &TravelField,
    params: &PeopleParams,
) -> Vec<(u32, f32)> {
    let max_s = (params.farm.max_walk_minutes * 60.0) as f32;
    // Cells in reach, in a fixed order (the field's own order is not stable).
    let mut cells: Vec<(u32, f32)> = reach
        .iter()
        .filter(|&(c, s)| s <= max_s && breakable(map, nav, land, land_params, c))
        .map(|(c, s)| (c as u32, s))
        .collect();
    cells.sort_unstable_by_key(|c| c.0);
    cells
}

/// The ground [`find_site`] looks at for a field on one of `cells`: the box around every field it
/// could mark out there, widened by the gap kept between fields. Fields, plots and earthworks
/// outside it cannot change what it finds (M5a slice AL). `None` when there are no cells.
pub fn site_ground(map: &WorldMap, cells: &[(u32, f32)], params: &PeopleParams) -> Option<RectCm> {
    let w = map.width as usize;
    let side_cm = (params.farm.field_m * 100.0).round() as i32;
    let cell_cm = (map.cell_size_m * 100.0).round() as i32;
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for &(cell, _) in cells {
        let (cx, cy) = ((cell as usize % w) as i32, (cell as usize / w) as i32);
        (x0, x1) = (x0.min(cx), x1.max(cx));
        (y0, y1) = (y0.min(cy), y1.max(cy));
    }
    (x0 <= x1).then(|| {
        let corner = |c: i32| c * cell_cm + cell_cm / 2 - side_cm / 2 - FIELD_GAP_CM;
        RectCm {
            x: corner(x0),
            y: corner(y0),
            w: (x1 - x0) * cell_cm + side_cm + 2 * FIELD_GAP_CM,
            h: (y1 - y0) * cell_cm + side_cm + 2 * FIELD_GAP_CM,
        }
    })
}

/// Looks for new ground for a field of side `params.farm.field_m` among `cells`, the breakable
/// cells within the farthest field walk of a home ([`breakable_in_reach`]), clear of other fields
/// and of `homes`: samples `params.farm.site_candidates` of them with a draw keyed by `key`, and
/// keeps the one worth most for `crop` ([`site_value`]).
#[allow(clippy::too_many_arguments)]
pub fn find_site(
    map: &WorldMap,
    nav: &NavGrid,
    land: &Land,
    land_params: &LandParams,
    cells: &[(u32, f32)],
    homes: &[(f32, f32)],
    params: &PeopleParams,
    crop: &CropParams,
    key: &[u64],
    placeholder: PermanentId,
) -> Option<Site> {
    let w = map.width as usize;
    let habitat = |cell: usize| {
        let p = land.patches.of_cell(cell, map.width);
        land.patches
            .class
            .get(p)
            .and_then(|&c| land_params.habitats.get(usize::from(c)))
    };
    let ok = |cell: usize| breakable(map, nav, land, land_params, cell);
    if cells.is_empty() {
        return None;
    }
    let side_cm = (params.farm.field_m * 100.0).round() as i32;
    let cell_cm = (map.cell_size_m * 100.0).round() as i32;
    let (map_w_cm, map_h_cm) = (map.width as i32 * cell_cm, map.height as i32 * cell_cm);
    // Every candidate is drawn first, one draw each as ever; then only the fields, plots,
    // earthworks and homes that could lie near one of them are kept to check against. Whatever
    // is near a candidate is near the box around them all, so the checks find what they would
    // against the whole world, at the cost of the neighbourhood (M5a slice AL).
    let mut rng = Rng64::from_key(key);
    let picks: Vec<(RectCm, f32)> = (0..params.farm.site_candidates)
        .filter_map(|_| {
            let (cell, secs) = cells[(rng.next_u64() % cells.len() as u64) as usize];
            let (cx, cy) = ((cell as usize % w) as i32, (cell as usize / w) as i32);
            let rect = RectCm {
                x: cx * cell_cm + cell_cm / 2 - side_cm / 2,
                y: cy * cell_cm + cell_cm / 2 - side_cm / 2,
                w: side_cm,
                h: side_cm,
            };
            let inside = rect.x >= 0
                && rect.y >= 0
                && rect.x + rect.w <= map_w_cm
                && rect.y + rect.h <= map_h_cm;
            inside.then_some((rect, secs))
        })
        .collect();
    let around = picks.iter().map(|&(r, _)| r).reduce(|a, b| {
        let (x, y) = (a.x.min(b.x), a.y.min(b.y));
        RectCm {
            x,
            y,
            w: (a.x + a.w).max(b.x + b.w) - x,
            h: (a.y + a.h).max(b.y + b.h) - y,
        }
    })?;
    let home_gap = |rect: &RectCm, (hx, hy): (f32, f32)| {
        let x = (hx * 100.0) as i32;
        let y = (hy * 100.0) as i32;
        let dx = (rect.x - x).max(x - (rect.x + rect.w)).max(0) as f32 / 100.0;
        let dy = (rect.y - y).max(y - (rect.y + rect.h)).max(0) as f32 / 100.0;
        dx.hypot(dy)
    };
    let taken: Vec<RectCm> = land
        .fields
        .iter()
        .map(|f| f.rect)
        .chain(land.plots.iter().map(|p| p.rect))
        .chain(
            land.earthworks
                .iter()
                .filter(|e| e.kind != civ_land::earth::EarthKind::Platform)
                .map(|e| e.rect),
        )
        .chain(land.wells.list.iter().map(|w| w.rect))
        .filter(|r| r.near(&around, FIELD_GAP_CM))
        .collect();
    let near_homes: Vec<(f32, f32)> = homes
        .iter()
        .copied()
        .filter(|&h| home_gap(&around, h) < HOME_GAP_M)
        .collect();
    let mut best: Option<(f64, Site)> = None;
    for (rect, secs) in picks {
        if taken.iter().any(|r| r.near(&rect, FIELD_GAP_CM)) {
            continue;
        }
        if !near_homes.iter().all(|&h| home_gap(&rect, h) >= HOME_GAP_M) {
            continue;
        }
        // Every cell under it must be arable dry ground; its ground is their patches' richness.
        let (x0, y0) = (rect.x / cell_cm, rect.y / cell_cm);
        let (x1, y1) = (
            (rect.x + rect.w - 1) / cell_cm,
            (rect.y + rect.h - 1) / cell_cm,
        );
        let mut sum = 0.0;
        let mut clear = 0.0;
        let mut n = 0usize;
        let mut fits = true;
        'cells: for y in y0..=y1 {
            for x in x0..=x1 {
                let c = y as usize * w + x as usize;
                if !ok(c) {
                    fits = false;
                    break 'cells;
                }
                let p = land.patches.of_cell(c, map.width);
                sum += f64::from(land.patches.richness.get(p).copied().unwrap_or(1.0));
                clear += habitat(c).map_or(0.0, |h| h.clear_h_per_ha);
                n += 1;
            }
        }
        if !fits || n == 0 {
            continue;
        }
        let (ground, clear_h_per_ha) = (sum / n as f64, clear / n as f64);
        let walk_min = f64::from(secs) / 60.0;
        let value = site_value(crop, ground, clear_h_per_ha, walk_min);
        if best.is_none_or(|(v, _)| value > v) {
            best = Some((
                value,
                Site {
                    rect,
                    walk_min,
                    ground,
                    clear_h_per_ha,
                    placeholder,
                },
            ));
        }
    }
    best.map(|(_, s)| s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_core::SimTime;

    fn crop() -> CropParams {
        CropParams {
            id: "core:crop/emmer".into(),
            name: "Emmer".into(),
            good: 0,
            seed_good: 1,
            seed_kg_per_ha: 90.0,
            yield_kg_per_ha: 800.0,
            prepare_from_day: 59,
            sow_from_day: 80,
            sow_until_day: 125,
            late_sowing_loss_per_day: 0.005,
            grow_days: 120,
            standing_loss_per_day: 0.02,
            untended_loss: 0.4,
            break_h_per_ha: 1000.0,
            prepare_h_per_ha: 600.0,
            sow_h_per_ha: 120.0,
            tend_h_per_ha: 200.0,
            reap_h_per_ha: 280.0,
            thresh_h_per_kg: 0.1,
            straw: None,
            kc: [0.4, 1.15, 0.4],
            kc_days: [30, 30, 40, 20],
            ky: 1.15,
            grain_n: 0.020,
            straw_n: 0.006,
            crop_n: 0.035,
        }
    }

    fn field(id: u64, stage: FieldStage) -> Field {
        Field {
            id: PermanentId::from_raw(id).expect("non-zero"),
            household: PermanentId::from_raw(1).expect("non-zero"),
            holder: civ_land::Party::Household(PermanentId::from_raw(1).expect("non-zero")),
            lease: None,
            rect: RectCm {
                x: 0,
                y: 0,
                w: 5000,
                h: 5000,
            },
            crop: 0,
            stage,
            stage_since: SimTime::ZERO,
            work_h: 0.0,
            tended_h: 0.0,
            ground: 1.0,
            clear_h_per_ha: 0.0,
            broken: true,
            sown_day: 0,
            sheaves_kg: 0.0,
            harvests: 1,
            water_mm: 0.0,
            need_mm: 0.0,
            got_mm: 0.0,
            soil: civ_land::FieldSoil::default(),
        }
    }

    fn view<'a>(crop: &'a CropParams, fields: Vec<&'a Field>, day: i64) -> FarmView<'a> {
        FarmView {
            crop,
            kcal_per_kg: 3340.0,
            fields,
            day,
            labour_per_day: 20.0,
            seed_kg: 100.0,
            room: 1.0,
            // A hectare's worth at full share: 800 kg less 90 of seed.
            need_kg: 710.0,
            plan_yield_share: 1.0,
            workable_share: 1.0,
            peak_ratio: 1.0,
            climatology: None,
            manuring: None,
        }
    }

    #[test]
    fn more_land_is_marked_out_only_with_need_seed_and_time_for_it() {
        let c = crop();
        let f = field(2, FieldStage::Fallow);
        // 0.25 ha held, 1 ha needed, 100 kg of seed: enough for 0.25 + 0.25 ha.
        let v = view(&c, vec![&f], 60);
        assert!(v.wants_new_field(0.25, 0.0));
        assert!(!v.wants_new_field(1.0, 0.0), "not enough seed for 1.25 ha");
        let mut late = v.clone();
        late.day = 120;
        assert!(
            !late.wants_new_field(0.25, 0.0),
            "no time left to break new ground"
        );
        // 0.25 ha of woodland on day 100: 180 h on the field held, 530 h to clear, break and
        // sow; 26 days at 20 h.
        let mut wood = v.clone();
        wood.day = 100;
        assert!(wood.wants_new_field(0.25, 0.0));
        assert!(
            !wood.wants_new_field(0.25, 1000.0),
            "no time left to clear woodland"
        );
        let mut fed = v.clone();
        fed.need_kg = 0.25 * 710.0;
        assert!(!fed.wants_new_field(0.25, 0.0), "enough land already");
        // Out of the season it breaks new ground for the next (ADR-0012 §4) with the seed for
        // it in store, if it can break it before the window opens...
        let mut autumn = v.clone();
        autumn.day = 260;
        assert!(autumn.wants_new_field(0.25, 0.0));
        let mut spent = autumn.clone();
        spent.seed_kg = 30.0;
        assert!(!spent.wants_new_field(0.25, 0.0), "no seed for it");
        // ...nine days before the window, 180 h of work against the 250 breaking needs...
        let mut eve = autumn.clone();
        eve.day = 50;
        assert!(!eve.wants_new_field(0.25, 0.0), "no time to break it first");
        // ...and prepare and sow it all in the window: 720 h a hectare against 1,340 h.
        let mut far = autumn.clone();
        far.need_kg = 5.0 * 710.0;
        far.seed_kg = 1000.0;
        assert!(far.wants_new_field(1.0, 0.0));
        assert!(
            !far.wants_new_field(2.0, 0.0),
            "too much to sow in one window"
        );
    }

    fn harvest(year: i32, kg_per_ha: f32) -> civ_land::HarvestRecord {
        civ_land::HarvestRecord {
            year,
            kg_per_ha,
            limit: civ_land::Limit::Season,
        }
    }

    fn midden() -> crate::params::MiddenParams {
        crate::params::MiddenParams {
            kg_per_person_day: 0.5,
            n_kg_per_person_year: 1.0,
            half_life_days: 365.0,
            load_kg: 25.0,
            spread_h_per_t: 2.0,
        }
    }

    #[test]
    fn a_midden_grows_with_its_household_and_wastes() {
        let m = midden();
        // A person's year adds 182.5 kg, less what wastes on the heap meanwhile.
        let year = m.after(0.0, 1, 365.0);
        assert!(year > 0.6 * 182.5 && year < 182.5, "{year}");
        // A steady household's heap settles where what it adds matches what wastes.
        let steady = m.after(0.0, 4, 36_500.0);
        assert!((steady - 4.0 * 0.5 * 365.0 / std::f64::consts::LN_2).abs() < 1e-6);
        // However the time is split.
        let split = m.after(m.after(100.0, 3, 40.0), 3, 60.0);
        assert!((split - m.after(100.0, 3, 100.0)).abs() < 1e-9);
        // A kilogram of nitrogen in each person's 182.5 kg a year.
        assert!((m.n_per_kg() - 1.0 / 182.5).abs() < 1e-12);
        // 2 h a tonne to dig out and spread, and 40 loads, each 6 minutes there and 6 back.
        assert!((m.h_per_kg(6.0) - (2.0 + 40.0 * 2.0 * 0.1) / 1000.0).abs() < 1e-12);
    }

    #[test]
    fn a_household_dungs_the_nearest_field_that_has_given_less_than_new_ground() {
        let c = crop();
        let m = midden();
        // Two fields that gave 500 kg/ha against new ground's 800, one near and one far, and one
        // that gave what new ground does.
        let mut near = field(2, FieldStage::Fallow);
        near.soil.record = vec![harvest(1, 500.0)];
        let mut far = field(3, FieldStage::Fallow);
        far.soil.record = vec![harvest(1, 500.0)];
        let mut good = field(4, FieldStage::Fallow);
        good.soil.record = vec![harvest(1, 800.0)];
        let mut v = view(&c, vec![&near, &far, &good], 300);
        v.need_kg = 10_000.0;
        v.manuring = Some(Manuring {
            midden_kg: 500.0,
            grain_per_kg: 0.01,
            midden: &m,
        });
        let walk = |f: &Field| Some(if f.id.get() == 3 { 20.0 } else { 3.0 });
        let none = |_: &RectCm| None;
        let dung = v
            .best(FieldTask::Manure, 4.0, None, &walk, &none)
            .expect("a field to dung");
        assert_eq!(dung.field, Some(near.id));
        // It may make up 300 kg/ha over 0.25 ha, 75 kg of grain: 7,500 kg of dung, more than
        // the 500 kg in the heap, all of which goes.
        assert!((dung.hours_left - 500.0 * m.h_per_kg(3.0)).abs() < 1e-9);
        assert_eq!(dung.urgency, 0.0, "nothing presses on it");
        // Not with less than a load in the heap...
        let mut empty = v.clone();
        empty.manuring = Some(Manuring {
            midden_kg: 10.0,
            grain_per_kg: 0.01,
            midden: &m,
        });
        assert!(
            empty
                .best(FieldTask::Manure, 4.0, None, &walk, &none)
                .is_err()
        );
        // ...nor on ground that gives what new ground does, nor on ground under a crop.
        let mut sown = near.clone();
        sown.stage = FieldStage::Sown;
        v.fields = vec![&good, &sown];
        assert!(v.best(FieldTask::Manure, 4.0, None, &walk, &none).is_err());
    }

    #[test]
    fn a_household_crops_its_best_fields_for_its_need_and_rests_the_rest() {
        let c = crop();
        let mut good = field(2, FieldStage::Fallow);
        good.soil.record = vec![harvest(1, 800.0)];
        let mut worn = field(3, FieldStage::Fallow);
        worn.soil.record = vec![harvest(1, 300.0)];
        // No record: what the crop gives on its ground, 800 kg/ha.
        let fresh = field(4, FieldStage::Fallow);
        // 300 kg to grow: two of the three fields' worth (177.5 kg each at full share).
        let mut v = view(&c, vec![&good, &worn, &fresh], 300);
        v.need_kg = 300.0;
        let ids = |v: &FarmView| -> Vec<u64> { v.cropping().iter().map(|f| f.id.get()).collect() };
        assert_eq!(ids(&v), vec![2, 4], "the worn field rests");
        // A field already begun is kept, whatever it gave.
        let mut begun = worn.clone();
        begun.stage = FieldStage::Prepared;
        v.fields = vec![&good, &begun, &fresh];
        assert_eq!(ids(&v), vec![3, 2, 4]);
        // In the window, with the good fields prepared, the worn one is not dug: it rests.
        let (mut a, mut b) = (good.clone(), fresh.clone());
        a.stage = FieldStage::Prepared;
        b.stage = FieldStage::Prepared;
        let mut spring = view(&c, vec![&a, &worn, &b], 70);
        spring.need_kg = 300.0;
        assert_eq!(spring.urgency(FieldTask::Prepare), 0.0);
        let dig = spring.best(FieldTask::Prepare, 4.0, None, &|_| Some(5.0), &|_| None);
        assert_eq!(dig.err(), Some(Reason::NoFieldWork));
        // Short of grain, it digs the worn field too.
        spring.need_kg = 400.0;
        let dig = spring.best(FieldTask::Prepare, 4.0, None, &|_| Some(5.0), &|_| None);
        assert_eq!(dig.expect("the worn field").field, Some(worn.id));
        // Out of the season nothing presses.
        assert_eq!(v.urgency(FieldTask::Prepare), 0.0);
    }

    #[test]
    fn the_seed_to_sow_the_ground_already_cropped_is_protected() {
        let c = crop();
        let a = field(2, FieldStage::Fallow);
        let b = field(3, FieldStage::Sown);
        // Two fields of 0.25 ha at 90 kg/ha.
        let (good, kg) = protected_seed([&a, &b].into_iter(), &[c]).expect("crops ground");
        assert_eq!(good, 1);
        assert!((kg - 45.0).abs() < 1e-9);
        assert_eq!(protected_seed(std::iter::empty(), &[crop()]), None);
    }

    #[test]
    fn seed_is_kept_for_the_area_planned_and_its_loss_in_store() {
        let c = crop();
        // Labour for 20 h a day over the 67-day season prepares and sows 1340 / 720 ha.
        let workable = 20.0 * 67.0 / 720.0;
        assert!((plan_area_ha(5.0, 0.5, 20.0, 1.0, &c) - workable).abs() < 1e-9);
        // When a fifth of the days are usually too wet or cold, a fifth less, unless longer
        // hours on the others make them up (ADR-0012 §5).
        assert!(
            (plan_area_ha(5.0, 0.5, 20.0, plan_share(0.8, 1.0), &c) - 0.8 * workable).abs() < 1e-9
        );
        assert_eq!(plan_share(0.8, 10.0 / 6.0), 1.0);
        assert!((plan_share(0.5, 10.0 / 6.0) - 0.5 * 10.0 / 6.0).abs() < 1e-12);
        assert_eq!(plan_area_ha(1.0, 0.5, 20.0, 1.0, &c), 1.0, "what it needs");
        assert_eq!(
            plan_area_ha(0.25, 0.5, 20.0, 1.0, &c),
            0.5,
            "the fields it has"
        );
        // Threshed on day 236, sowing starts on day 80 of the next year: 209 days in store.
        let kept = seed_to_keep(1.0, &c, 1560.0, 236);
        assert!((kept - 90.0 * (209.0f64 / 1560.0).exp2()).abs() < 1e-9);
        assert_eq!(seed_to_keep(1.0, &c, 0.0, 236), 90.0, "seed that keeps");
        assert_eq!(seed_to_keep(1.0, &c, 1560.0, 80), 90.0, "sowing today");
    }

    #[test]
    fn the_next_harvest_is_the_first_sown_field_ripening_or_the_next_season() {
        let c = crop();
        let mut sown = field(2, FieldStage::Sown);
        sown.sown_day = 90;
        let ripe = sown.ripe_day(&c);
        let fallow = field(3, FieldStage::Fallow);
        // A sown field ripens on its day, and the first grain is in a week later.
        let d = days_to_harvest(&c, [&fallow, &sown].into_iter(), 100);
        assert!((d - ((ripe - 100) as f64 + HARVEST_IN_DAYS)).abs() < 1e-9);
        // With nothing sown, in the sowing window: sown today, ripe the crop's growing days on.
        let d = days_to_harvest(&c, [&fallow].into_iter(), 100);
        assert!((d - (f64::from(c.grow_days) + HARVEST_IN_DAYS)).abs() < 1e-9);
        // Before the window, from its first day; after it, from next year's.
        let d = days_to_harvest(&c, std::iter::empty(), 30);
        let first = f64::from(c.sow_from_day) - 30.0;
        assert!((d - (first + f64::from(c.grow_days) + HARVEST_IN_DAYS)).abs() < 1e-9);
        let d = days_to_harvest(&c, std::iter::empty(), 300);
        let next = f64::from(c.sow_from_day) + 365.0 - 300.0;
        assert!((d - (next + f64::from(c.grow_days) + HARVEST_IN_DAYS)).abs() < 1e-9);
        // Sheaves in the field are grain within days.
        let reaped = field(4, FieldStage::Reaped);
        assert_eq!(
            days_to_harvest(&c, [&reaped].into_iter(), 200),
            HARVEST_IN_DAYS
        );
    }

    #[test]
    fn new_ground_is_worth_its_grain_over_the_work_and_the_walk() {
        let c = crop();
        let open = site_value(&c, 1.0, 0.0, 5.0);
        assert!(
            site_value(&c, 1.0, 1000.0, 5.0) < open,
            "woodland must be cleared first"
        );
        assert!(site_value(&c, 1.0, 0.0, 30.0) < open, "farther");
        assert!(
            site_value(&c, 1.3, 1000.0, 5.0) > open,
            "much better ground repays clearing"
        );
    }

    #[test]
    fn urgency_is_work_left_over_work_that_can_still_be_done() {
        let c = crop();
        let f = field(2, FieldStage::Fallow);
        // 0.25 ha: 150 h to prepare cropped ground, 30 h to sow; 20 h a day.
        let v = view(&c, vec![&f], 116); // 10 days left in the window
        assert!((v.urgency(FieldTask::Prepare) - 180.0 / 200.0).abs() < 1e-9);
        assert_eq!(v.urgency(FieldTask::Thresh), 0.0);
        // Out of season there is nothing to do, so nothing presses.
        let v = view(&c, vec![&f], 200);
        assert_eq!(v.urgency(FieldTask::Prepare), 0.0);
    }

    #[test]
    fn the_best_field_pays_most_food_per_hour_and_sowing_needs_seed() {
        let c = crop();
        let near = field(2, FieldStage::Prepared);
        let mut far = field(3, FieldStage::Prepared);
        far.rect.x = 100_000;
        let walk = |f: &Field| Some(if f.id.get() == 2 { 5.0 } else { 40.0 });
        let fair = |_: &RectCm| None;
        let v = view(&c, vec![&near, &far], 85);
        let best = v
            .best(FieldTask::Sow, 4.0, None, &walk, &fair)
            .expect("a field to sow");
        assert_eq!(best.field, Some(near.id));
        assert!(best.kcal_per_hour > 0.0 && best.hours_left > 0.0);
        let mut no_seed = v.clone();
        no_seed.seed_kg = 0.0;
        assert_eq!(
            no_seed
                .best(FieldTask::Sow, 4.0, None, &walk, &fair)
                .map(|o| o.field),
            Err(Reason::NoSeed)
        );
        assert_eq!(
            v.best(FieldTask::Reap, 4.0, None, &walk, &fair)
                .map(|o| o.field),
            Err(Reason::NoFieldWork)
        );
        // Where snow still lies on the near field, higher up, and not on the far one, the far
        // one is worked (ADR-0012 §5). Where neither can be, the weather is the reason.
        let near_rect = near.rect;
        let snow_near = |r: &RectCm| (*r == near_rect).then_some(Reason::SnowCover);
        let best = v
            .best(FieldTask::Sow, 4.0, None, &walk, &snow_near)
            .expect("the far field");
        assert_eq!(best.field, Some(far.id));
        let rain = |_: &RectCm| Some(Reason::WetGround);
        assert_eq!(
            v.best(FieldTask::Sow, 4.0, None, &walk, &rain)
                .map(|o| o.field),
            Err(Reason::WetGround)
        );
        // Only work that turns the soil waits for the weather: a growing crop is weeded in it.
        let mut growing = field(4, FieldStage::Sown);
        growing.sown_day = 85;
        let v = view(&c, vec![&growing], 100);
        assert_eq!(
            v.best(FieldTask::Tend, 4.0, None, &walk, &rain)
                .map(|o| o.field),
            Ok(Some(growing.id))
        );
        assert!(FieldTask::Prepare.turns_soil() && FieldTask::Sow.turns_soil());
        assert!(!FieldTask::Tend.turns_soil() && !FieldTask::Reap.turns_soil());
    }
}
