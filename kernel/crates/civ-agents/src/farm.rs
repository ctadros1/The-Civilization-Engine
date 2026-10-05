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
    /// The area it plans to crop for its needs, hectares.
    pub need_ha: f64,
    /// The share of the days of the sowing window the ground can usually be worked (ADR-0012
    /// §5): how pressing the work left is, of the days left.
    pub workable_share: f64,
    /// Field work a capable adult gives on a day at a peak, against an ordinary day's: what makes
    /// up the days the weather takes when it plans spring work.
    pub peak_ratio: f64,
    /// What it knows of the weather's years, to judge a growing crop by the water its season so
    /// far has brought (ADR-0012 §4); `None` judges every crop by an average year.
    pub climatology: Option<&'a civ_land::Climatology>,
}

/// The area a household of `members` plans to crop for its needs, hectares: the share of a
/// year's food it means to grow, with what is lost before it is eaten, at a cautious yield less
/// the seed (research 10-01 §2.3: plan on a poor harvest, keep seed apart).
pub fn need_area_ha(members: usize, params: &PeopleParams, crop: &CropParams, kcal: f64) -> f64 {
    let year = members as f64 * params.household.daily_kcal_per_person * 365.0;
    let grown = year * params.farm.grain_share / (1.0 - params.farm.loss_share).max(0.05);
    let net = crop.yield_kg_per_ha * params.farm.plan_yield_share - crop.seed_kg_per_ha;
    grown / (net.max(1.0) * kcal.max(1.0))
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

/// Hours a field still needs before it is sown (0 once sown).
fn to_sow_h(f: &Field, crop: &CropParams) -> f64 {
    let left = (f.stage_work_h(crop) - f64::from(f.work_h)).max(0.0);
    match f.stage {
        FieldStage::Fallow => left + crop.sow_h_per_ha * f.area_ha(),
        FieldStage::Prepared => left,
        FieldStage::Sown | FieldStage::Reaped => 0.0,
    }
}

impl FarmView<'_> {
    /// Whether the household would mark out a new field of `ha` hectares, needing
    /// `clear_h_per_ha` of clearing, today: it needs more land, has the seed for it and every
    /// field not yet sown, and can still clear, prepare and sow them all before the sowing
    /// window closes.
    pub fn wants_new_field(&self, ha: f64, clear_h_per_ha: f64) -> bool {
        let crop = self.crop;
        if !crop.can_prepare(self.day) {
            return false;
        }
        let total: f64 = self.fields.iter().map(|f| f.area_ha()).sum();
        if total + 1e-9 >= self.need_ha {
            return false;
        }
        let unsown: f64 = self
            .fields
            .iter()
            .filter(|f| matches!(f.stage, FieldStage::Fallow | FieldStage::Prepared))
            .map(|f| f.area_ha())
            .sum();
        if self.seed_kg + 1e-9 < (unsown + ha) * crop.seed_kg_per_ha {
            return false;
        }
        let left: f64 = self.fields.iter().map(|f| to_sow_h(f, crop)).sum();
        let new = ha * (crop.break_h_per_ha + clear_h_per_ha + crop.sow_h_per_ha);
        let days =
            sowing_days_left(crop, self.day) * plan_share(self.workable_share, self.peak_ratio);
        left + new <= self.labour_per_day * days
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
            FieldTask::Prepare | FieldTask::Sow => {
                let work = self
                    .fields
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
            FieldTask::Thresh => (0.0, f64::INFINITY),
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
        let remaining = f.remaining_work_h(crop, expected);
        let per_hour = expected * self.kcal_per_kg / remaining.max(1e-6);
        let left = match task {
            FieldTask::Tend => (crop.tend_h_per_ha * f.area_ha() - f64::from(f.tended_h)).max(0.0),
            _ => (f.stage_work_h(crop) - f64::from(f.work_h)).max(0.0),
        };
        (per_hour, left)
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
    let mut rng = Rng64::from_key(key);
    let mut best: Option<(f64, Site)> = None;
    for _ in 0..params.farm.site_candidates {
        let (cell, secs) = cells[(rng.next_u64() % cells.len() as u64) as usize];
        let (cx, cy) = ((cell as usize % w) as i32, (cell as usize / w) as i32);
        let rect = RectCm {
            x: cx * cell_cm + cell_cm / 2 - side_cm / 2,
            y: cy * cell_cm + cell_cm / 2 - side_cm / 2,
            w: side_cm,
            h: side_cm,
        };
        if rect.x < 0 || rect.y < 0 || rect.x + rect.w > map_w_cm || rect.y + rect.h > map_h_cm {
            continue;
        }
        if land.fields.iter().any(|f| f.rect.near(&rect, FIELD_GAP_CM))
            || land.plots.iter().any(|p| p.rect.near(&rect, FIELD_GAP_CM))
            || civ_land::earth::dug_near(&land.earthworks, &rect, FIELD_GAP_CM)
        {
            continue;
        }
        let clear_of_homes = homes.iter().all(|&(hx, hy)| {
            let x = (hx * 100.0) as i32;
            let y = (hy * 100.0) as i32;
            let dx = (rect.x - x).max(x - (rect.x + rect.w)).max(0) as f32 / 100.0;
            let dy = (rect.y - y).max(y - (rect.y + rect.h)).max(0) as f32 / 100.0;
            dx.hypot(dy) >= HOME_GAP_M
        });
        if !clear_of_homes {
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
            need_ha: 1.0,
            workable_share: 1.0,
            peak_ratio: 1.0,
            climatology: None,
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
        fed.need_ha = 0.25;
        assert!(!fed.wants_new_field(0.25, 0.0), "enough land already");
        let mut winter = v.clone();
        winter.day = 200;
        assert!(!winter.wants_new_field(0.25, 0.0), "not the season");
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
