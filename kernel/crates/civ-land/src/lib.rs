//! Land state that changes over time (ADR-0004).
//!
//! - **Habitat patches**: the map is divided into square patches (128 m at 8 m cells). Each gets a
//!   habitat class from authored rules (the first rule a patch meets wins) and a richness from
//!   seeded noise. Computed once when a world is created and saved, so content edits never remap
//!   a saved world's stocks.
//! - **Stocks**: every wild resource (M1: wild plant food) has a standing stock per patch. Each day
//!   it gains a seasonal production and loses a fixed fraction (rot, wildlife); gathering takes
//!   from it at a rate that falls as the patch empties. Nothing respawns (research 03-06 §1.1–1.3).
//! - **Climate**: one yearly factor shared by the whole map, an AR(1) series around 1, scales
//!   production (03-06 §5.2, 08-01 §2.3).
//! - **Settlements**: the places people found.
//!
//! This crate knows nothing about people: it is told what is gathered and where.

#![forbid(unsafe_code)]

use civ_core::time::{DAYS_PER_YEAR, MONTH_STARTS};
use civ_core::{PermanentId, Rng64, SimTime};
use civ_world::noise::Noise;
use civ_world::{WATER_LAND, WorldMap, terrain};

/// Purpose tag for the yearly climate draw.
const PURPOSE_CLIMATE: u64 = 0x636c_696d_6174_6531; // "climate1"
/// Purpose tag for patch richness noise.
const PURPOSE_RICHNESS: u64 = 0x7269_6368_6e65_7373; // "richness"

/// A habitat class rule. A patch belongs to the first rule whose conditions it meets; a rule
/// without conditions matches every patch.
#[derive(Clone, Debug, PartialEq)]
pub struct HabitatRule {
    /// Short id, for example `floodplain`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Matches if at least this share of the patch's cells is water.
    pub min_water_fraction: Option<f64>,
    /// Matches if the median height above drainage of the patch's land cells is at most this, m.
    pub max_median_hand_m: Option<f64>,
    /// Matches if the mean slope of the patch's land cells is at most this, rise over run.
    pub max_mean_slope: Option<f64>,
    /// Whether ground of this class can be cleared and cultivated.
    pub arable: bool,
}

/// A wild resource that grows in habitat patches.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceParams {
    /// Short id, for example `wild_plants`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Unit of the stock, for example `kcal`.
    pub unit: String,
    /// Production per hectare per year at richness 1 in an average year, by habitat class (in
    /// the order of [`LandParams::habitats`]).
    pub production_per_ha_yr: Vec<f64>,
    /// Share of the standing stock lost per day to rot and wildlife.
    pub loss_per_day: f64,
    /// Production weight by month, January first; the mean over the year should be 1.
    pub season: [f64; 12],
    /// Gathering rate on an untouched, saturated patch, units per person-hour.
    pub max_rate_per_hour: f64,
    /// Standing stock at which gathering runs at half its maximum rate, units per hectare.
    pub half_rate_stock_per_ha: f64,
}

/// How land is classified and how its resources grow. Authored in content.
#[derive(Clone, Debug, PartialEq)]
pub struct LandParams {
    /// Terrain cells per patch side.
    pub patch_cells: u32,
    /// Drainage area that counts as a channel when measuring height above drainage, km².
    pub channel_area_km2: f64,
    /// Habitat rules, in matching order. The last one should have no conditions.
    pub habitats: Vec<HabitatRule>,
    /// Lowest and highest patch richness.
    pub richness_min: f64,
    /// Highest patch richness.
    pub richness_max: f64,
    /// Typical size of rich and poor areas, metres.
    pub richness_feature_m: f64,
    /// Wild resources.
    pub resources: Vec<ResourceParams>,
    /// Coefficient of variation of the yearly climate factor.
    pub climate_cv: f64,
    /// Year-to-year autocorrelation of the climate factor.
    pub climate_autocorrelation: f64,
}

impl LandParams {
    /// The resource with this id.
    pub fn resource(&self, id: &str) -> Option<usize> {
        self.resources.iter().position(|r| r.id == id)
    }
}

/// The patch grid: habitat class and richness per patch.
#[derive(Clone, Debug, PartialEq)]
pub struct Patches {
    /// Patches west–east.
    pub cols: u32,
    /// Patches north–south.
    pub rows: u32,
    /// Terrain cells per patch side.
    pub patch_cells: u32,
    /// Terrain cell size, metres.
    pub cell_size_m: f32,
    /// Habitat class per patch: an index into [`LandParams::habitats`].
    pub class: Vec<u8>,
    /// Richness per patch, between the authored minimum and maximum.
    pub richness: Vec<f32>,
}

impl Patches {
    /// Number of patches.
    pub fn len(&self) -> usize {
        self.class.len()
    }

    /// Whether there are no patches.
    pub fn is_empty(&self) -> bool {
        self.class.is_empty()
    }

    /// Patch side, metres.
    pub fn side_m(&self) -> f32 {
        self.patch_cells as f32 * self.cell_size_m
    }

    /// Area of one patch, hectares.
    pub fn area_ha(&self) -> f64 {
        let side = f64::from(self.side_m());
        side * side / 10_000.0
    }

    /// The patch containing terrain cell `cell` of a map `map_width` cells wide.
    pub fn of_cell(&self, cell: usize, map_width: u32) -> usize {
        let (x, y) = (cell as u32 % map_width, cell as u32 / map_width);
        ((y / self.patch_cells) * self.cols + x / self.patch_cells) as usize
    }

    /// The patch containing the point `(x, y)` in metres, clamped to the grid.
    pub fn of_point(&self, x: f32, y: f32) -> usize {
        let side = self.side_m();
        let px = ((x / side).floor().max(0.0) as u32).min(self.cols - 1);
        let py = ((y / side).floor().max(0.0) as u32).min(self.rows - 1);
        (py * self.cols + px) as usize
    }

    /// Centre of patch `p`, metres.
    pub fn centre_m(&self, p: usize) -> (f32, f32) {
        let side = self.side_m();
        let (px, py) = (p as u32 % self.cols, p as u32 / self.cols);
        ((px as f32 + 0.5) * side, (py as f32 + 0.5) * side)
    }
}

/// The yearly climate factor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClimateYear {
    /// The year it applies to.
    pub year: i64,
    /// The standard-normal deviate behind it (carried for the next year's autocorrelation).
    pub deviate: f64,
    /// Production multiplier, around 1.
    pub factor: f64,
}

/// A place people founded.
#[derive(Clone, Debug, PartialEq)]
pub struct Settlement {
    /// Permanent id.
    pub id: PermanentId,
    /// Name, from the naming event that founded it.
    pub name: String,
    /// When it was founded.
    pub founded: SimTime,
    /// Where its hearth is, metres from the map's north-west corner.
    pub hearth_m: (f32, f32),
}

/// All land state of one world.
#[derive(Clone, Debug, PartialEq)]
pub struct Land {
    /// The patch grid. Saved.
    pub patches: Patches,
    /// Standing stock per resource per patch. Saved.
    pub stocks: Vec<Vec<f32>>,
    /// The last day whose growth has been applied. Saved.
    pub stock_day: i64,
    /// This year's climate. Saved.
    pub climate: ClimateYear,
    /// Settlements, oldest first. Saved.
    pub settlements: Vec<Settlement>,
}

/// Summary of a patch's terrain, for classification.
#[derive(Clone, Copy, Debug, Default)]
struct PatchTerrain {
    water_fraction: f64,
    median_hand_m: f64,
    mean_slope: f64,
}

/// Day of the year, 0–364, of a day index.
pub fn day_of_year(day: i64) -> i64 {
    day.rem_euclid(DAYS_PER_YEAR)
}

/// The seasonal weight of a resource on a day, interpolated between mid-month values.
pub fn season_weight(season: &[f64; 12], day: i64) -> f64 {
    let d = day_of_year(day) as f64 + 0.5;
    // Mid-points of each month, in days from January 1.
    let mid = |m: usize| (MONTH_STARTS[m] + MONTH_STARTS[m + 1]) as f64 / 2.0;
    let mut m = 0usize;
    while m < 12 && mid(m) <= d {
        m += 1;
    }
    let (a, b) = if m == 0 { (11, 0) } else { (m - 1, m % 12) };
    let (ma, mut mb) = (mid(a), mid(b));
    let mut x = d;
    if mb <= ma {
        mb += DAYS_PER_YEAR as f64;
        if x < ma {
            x += DAYS_PER_YEAR as f64;
        }
    }
    let t = ((x - ma) / (mb - ma)).clamp(0.0, 1.0);
    season[a] * (1.0 - t) + season[b] * t
}

fn classify(params: &LandParams, t: &PatchTerrain) -> u8 {
    for (i, rule) in params.habitats.iter().enumerate() {
        let ok = rule
            .min_water_fraction
            .is_none_or(|v| t.water_fraction >= v)
            && rule.max_median_hand_m.is_none_or(|v| t.median_hand_m <= v)
            && rule.max_mean_slope.is_none_or(|v| t.mean_slope <= v);
        if ok {
            return i as u8;
        }
    }
    params.habitats.len().saturating_sub(1) as u8
}

fn normal(rng: &mut Rng64) -> f64 {
    // Box–Muller from two uniforms in (0, 1].
    let u1 = 1.0 - rng.next_f64();
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

impl Land {
    /// Classifies a map's patches and grows a year of stocks so a new world starts in season.
    pub fn create(map: &WorldMap, params: &LandParams, seed: u64, today: i64) -> Land {
        let pc = params.patch_cells.max(1);
        let cols = map.width.div_ceil(pc);
        let rows = map.height.div_ceil(pc);
        let slope = terrain::slopes(map);
        let hand = terrain::height_above_drainage(map, (params.channel_area_km2 * 1e6) as f32);
        let noise = Noise::new(civ_core::rng::key(&[seed, PURPOSE_RICHNESS]));
        let feature_patches =
            (params.richness_feature_m / (f64::from(pc) * f64::from(map.cell_size_m))).max(1.0);
        let mut class = Vec::with_capacity((cols * rows) as usize);
        let mut richness = Vec::with_capacity((cols * rows) as usize);
        let w = map.width as usize;
        let mut hands = Vec::new();
        for py in 0..rows {
            for px in 0..cols {
                let (mut cells, mut water, mut slope_sum) = (0usize, 0usize, 0.0f64);
                hands.clear();
                for y in py * pc..((py + 1) * pc).min(map.height) {
                    for x in px * pc..((px + 1) * pc).min(map.width) {
                        let i = y as usize * w + x as usize;
                        cells += 1;
                        if map.water[i] == WATER_LAND {
                            slope_sum += f64::from(slope[i]);
                            hands.push(hand[i]);
                        } else {
                            water += 1;
                        }
                    }
                }
                let land = hands.len();
                hands.sort_by(f32::total_cmp);
                let t = PatchTerrain {
                    water_fraction: water as f64 / cells.max(1) as f64,
                    median_hand_m: hands.get(land / 2).map_or(0.0, |&h| f64::from(h)),
                    mean_slope: if land > 0 {
                        slope_sum / land as f64
                    } else {
                        0.0
                    },
                };
                class.push(classify(params, &t));
                let n = noise.fbm(
                    f64::from(px) / feature_patches,
                    f64::from(py) / feature_patches,
                    3,
                );
                let r = params.richness_min
                    + (params.richness_max - params.richness_min) * (0.5 + 0.5 * n).clamp(0.0, 1.0);
                richness.push(r as f32);
            }
        }
        let patches = Patches {
            cols,
            rows,
            patch_cells: pc,
            cell_size_m: map.cell_size_m,
            class,
            richness,
        };
        let year = today.div_euclid(DAYS_PER_YEAR) + 1;
        let mut land = Land {
            stocks: vec![vec![0.0; patches.len()]; params.resources.len()],
            patches,
            stock_day: today - DAYS_PER_YEAR - 1,
            climate: ClimateYear {
                year: year - 1,
                deviate: 0.0,
                factor: 1.0,
            },
            settlements: Vec::new(),
        };
        // Start each stock at its equilibrium for the season a year ago, then grow a year.
        for (r, res) in params.resources.iter().enumerate() {
            for p in 0..land.patches.len() {
                let daily = land.production(params, r, p, land.stock_day);
                let eq = if res.loss_per_day > 0.0 {
                    daily / res.loss_per_day
                } else {
                    0.0
                };
                land.stocks[r][p] = eq as f32;
            }
        }
        land.advance_to_day(params, seed, today - 1);
        land
    }

    /// What is wrong with saved land state for a map, if anything. A save holding any of these
    /// is refused (ADR-0002).
    pub fn problems(&self, map: &WorldMap, habitats: usize, next_id: u64) -> Vec<String> {
        let mut out = Vec::new();
        let p = &self.patches;
        let pc = p.patch_cells.max(1);
        if p.patch_cells == 0
            || p.cols != map.width.div_ceil(pc)
            || p.rows != map.height.div_ceil(pc)
            || p.cell_size_m != map.cell_size_m
        {
            out.push("the patch grid does not fit the map".to_owned());
        }
        let n = p.cols as usize * p.rows as usize;
        if p.class.len() != n || p.richness.len() != n {
            out.push("the patch grid has the wrong number of patches".to_owned());
        }
        if p.class.iter().any(|&c| usize::from(c) >= habitats.max(1)) {
            out.push("a patch has an unknown habitat".to_owned());
        }
        if p.richness.iter().any(|r| !(r.is_finite() && *r >= 0.0)) {
            out.push("a patch has an invalid richness".to_owned());
        }
        for s in &self.stocks {
            if s.len() != n || s.iter().any(|v| !v.is_finite()) {
                out.push("a stock layer is malformed".to_owned());
                break;
            }
        }
        if !(self.climate.factor.is_finite() && self.climate.deviate.is_finite()) {
            out.push("the climate year is not a number".to_owned());
        }
        let (w, h) = (
            map.width as f32 * map.cell_size_m,
            map.height as f32 * map.cell_size_m,
        );
        for s in &self.settlements {
            let (x, y) = s.hearth_m;
            if s.id.get() >= next_id || !(0.0..=w).contains(&x) || !(0.0..=h).contains(&y) {
                out.push(format!(
                    "settlement {} is outside the map or unallocated",
                    s.id
                ));
            }
        }
        out
    }

    /// Fills resource `r` at the equilibrium of production on `day` (for a resource added to the
    /// content after a world was saved).
    pub fn fill_equilibrium(&mut self, params: &LandParams, r: usize, day: i64) {
        let loss = params.resources[r].loss_per_day;
        for p in 0..self.patches.len() {
            let daily = self.production(params, r, p, day);
            self.stocks[r][p] = if loss > 0.0 {
                (daily / loss) as f32
            } else {
                0.0
            };
        }
    }

    /// Production of resource `r` in patch `p` on day `day`, units.
    pub fn production(&self, params: &LandParams, r: usize, p: usize, day: i64) -> f64 {
        let res = &params.resources[r];
        let class = self.patches.class[p] as usize;
        let per_ha = res.production_per_ha_yr.get(class).copied().unwrap_or(0.0);
        per_ha
            * self.patches.area_ha()
            * f64::from(self.patches.richness[p])
            * season_weight(&res.season, day)
            * self.climate.factor
            / DAYS_PER_YEAR as f64
    }

    fn climate_for(&self, params: &LandParams, seed: u64, year: i64) -> ClimateYear {
        let mut rng = Rng64::from_key(&[seed, PURPOSE_CLIMATE, year as u64]);
        let rho = params.climate_autocorrelation.clamp(-0.99, 0.99);
        let z = rho * self.climate.deviate + (1.0 - rho * rho).sqrt() * normal(&mut rng);
        ClimateYear {
            year,
            deviate: z,
            factor: (1.0 + params.climate_cv * z).max(0.05),
        }
    }

    /// Applies growth and loss for every day up to and including `day`. Each day's step is exact
    /// for constant production over the day, so splitting an advance changes nothing.
    pub fn advance_to_day(&mut self, params: &LandParams, seed: u64, day: i64) {
        while self.stock_day < day {
            let d = self.stock_day + 1;
            let year = d.div_euclid(DAYS_PER_YEAR) + 1;
            if year != self.climate.year {
                self.climate = self.climate_for(params, seed, year);
            }
            for (r, res) in params.resources.iter().enumerate() {
                let keep = (-res.loss_per_day).exp();
                for p in 0..self.patches.len() {
                    let prod = self.production(params, r, p, d);
                    let s = f64::from(self.stocks[r][p]);
                    let next = if res.loss_per_day > 0.0 {
                        s * keep + prod / res.loss_per_day * (1.0 - keep)
                    } else {
                        s + prod
                    };
                    self.stocks[r][p] = next as f32;
                }
            }
            self.stock_day = d;
        }
    }

    /// Gathering rate of resource `r` in patch `p` now, units per person-hour.
    pub fn gather_rate(&self, params: &LandParams, r: usize, p: usize) -> f64 {
        let res = &params.resources[r];
        let s = f64::from(self.stocks[r][p]).max(0.0);
        let half = res.half_rate_stock_per_ha * self.patches.area_ha();
        if s <= 0.0 {
            0.0
        } else {
            res.max_rate_per_hour * s / (s + half)
        }
    }

    /// The patch and its up-to-eight neighbours: the area one gathering trip ranges over.
    pub fn block(&self, p: usize) -> impl Iterator<Item = usize> + '_ {
        let (cols, rows) = (self.patches.cols as i64, self.patches.rows as i64);
        let (px, py) = (p as i64 % cols, p as i64 / cols);
        (-1..=1).flat_map(move |dy| {
            (-1..=1).filter_map(move |dx| {
                let (x, y) = (px + dx, py + dy);
                (x >= 0 && y >= 0 && x < cols && y < rows).then_some((y * cols + x) as usize)
            })
        })
    }

    /// Takes `hours` of one person's gathering of resource `r` around patch `p`: at each of a
    /// dozen small steps the gatherer works the richest patch of the block (the patch and its
    /// neighbours), as a forager moves to the best spot. Returns what was gathered.
    pub fn gather_around(
        &mut self,
        params: &LandParams,
        r: usize,
        p: usize,
        hours: f64,
        efficiency: f64,
    ) -> f64 {
        const STEPS: u32 = 12;
        let dt = hours / f64::from(STEPS);
        let block: Vec<usize> = self.block(p).collect();
        let mut total = 0.0;
        for _ in 0..STEPS {
            let Some(&best) = block
                .iter()
                .max_by(|&&a, &&b| self.stocks[r][a].total_cmp(&self.stocks[r][b]))
            else {
                break;
            };
            let take = (self.gather_rate(params, r, best) * efficiency * dt)
                .min(f64::from(self.stocks[r][best]).max(0.0));
            self.stocks[r][best] -= take as f32;
            total += take;
        }
        total
    }

    /// Takes `hours` of one person's gathering of resource `r` from patch `p`, scaled by
    /// `efficiency` (skill and capacity, 1 = a capable adult). Returns what was gathered. The
    /// rate falls as the stock falls, integrated in small steps.
    pub fn gather(
        &mut self,
        params: &LandParams,
        r: usize,
        p: usize,
        hours: f64,
        efficiency: f64,
    ) -> f64 {
        const STEPS: u32 = 12;
        let dt = hours / f64::from(STEPS);
        let mut total = 0.0;
        for _ in 0..STEPS {
            let take = (self.gather_rate(params, r, p) * efficiency * dt)
                .min(f64::from(self.stocks[r][p]).max(0.0));
            self.stocks[r][p] -= take as f32;
            total += take;
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_world::{Climate, WATER_RIVER};

    pub(crate) fn params() -> LandParams {
        let flat = [1.0; 12];
        LandParams {
            patch_cells: 4,
            channel_area_km2: 1e9,
            habitats: vec![
                HabitatRule {
                    id: "water".into(),
                    name: "Water".into(),
                    min_water_fraction: Some(0.5),
                    max_median_hand_m: None,
                    max_mean_slope: None,
                    arable: false,
                },
                HabitatRule {
                    id: "flat".into(),
                    name: "Flat".into(),
                    min_water_fraction: None,
                    max_median_hand_m: Some(3.0),
                    max_mean_slope: Some(0.05),
                    arable: true,
                },
                HabitatRule {
                    id: "rest".into(),
                    name: "Rest".into(),
                    min_water_fraction: None,
                    max_median_hand_m: None,
                    max_mean_slope: None,
                    arable: false,
                },
            ],
            richness_min: 1.0,
            richness_max: 1.0,
            richness_feature_m: 500.0,
            resources: vec![ResourceParams {
                id: "plants".into(),
                name: "Plants".into(),
                unit: "kcal".into(),
                production_per_ha_yr: vec![0.0, 36_500.0, 3_650.0],
                loss_per_day: 0.05,
                season: flat,
                max_rate_per_hour: 1000.0,
                half_rate_stock_per_ha: 100.0,
            }],
            climate_cv: 0.0,
            climate_autocorrelation: 0.3,
        }
    }

    fn map() -> WorldMap {
        // 8 x 8 cells of 8 m: a river along x = 0..2 (the west quarter), flat land elsewhere,
        // rising steeply in the south-east patch.
        let (w, h) = (8u32, 8u32);
        let n = (w * h) as usize;
        let mut elevation = vec![10.0; n];
        let mut water = vec![WATER_LAND; n];
        let mut receivers = vec![4u8; n];
        for i in 0..n {
            let (x, y) = (i % 8, i / 8);
            if x < 2 {
                water[i] = WATER_RIVER;
                receivers[i] = 8;
            }
            // Steep inside the south-east patch only, one cell in from its edges, so slopes
            // measured from the flat patches' edge cells stay flat.
            if x >= 5 && y >= 5 {
                elevation[i] = 10.0 + 4.0 * ((x - 5) + (y - 5)) as f32;
            }
        }
        WorldMap {
            width: w,
            height: h,
            cell_size_m: 8.0,
            sea_level_m: 0.0,
            elevation,
            receivers,
            water,
            lake_id: vec![0; n],
            lakes: Vec::new(),
            reaches: Vec::new(),
            inflows: Vec::new(),
            climate: Climate {
                precipitation_mm_per_yr: 800.0,
                evapotranspiration_mm_per_yr: 500.0,
                lake_evaporation_mm_per_yr: 900.0,
            },
            drainage_area_m2: vec![64.0; n],
        }
    }

    #[test]
    fn patches_are_classified_by_the_first_matching_rule() {
        let land = Land::create(&map(), &params(), 1, 400);
        assert_eq!((land.patches.cols, land.patches.rows), (2, 2));
        // West patches are half river: water. The north-east is flat and low: arable "flat".
        // The south-east is steep: "rest".
        assert_eq!(land.patches.class, vec![0, 1, 0, 2]);
    }

    #[test]
    fn stocks_settle_at_production_over_loss() {
        let p = params();
        let mut land = Land::create(&map(), &p, 1, 400);
        land.advance_to_day(&p, 1, 2000);
        let area = land.patches.area_ha();
        let daily = 36_500.0 * area / 365.0;
        let eq = daily / 0.05;
        assert!((f64::from(land.stocks[0][1]) - eq).abs() < 1e-3 * eq);
        assert_eq!(land.stocks[0][0], 0.0, "water grows no plants");
    }

    #[test]
    fn splitting_an_advance_changes_nothing() {
        let p = params();
        let mut a = Land::create(&map(), &p, 9, 400);
        let mut b = a.clone();
        a.advance_to_day(&p, 9, 900);
        for d in (401..=900).step_by(37) {
            b.advance_to_day(&p, 9, d);
        }
        b.advance_to_day(&p, 9, 900);
        assert_eq!(a, b);
    }

    #[test]
    fn gathering_slows_as_a_patch_empties_and_never_overdraws() {
        let p = params();
        let mut land = Land::create(&map(), &p, 1, 400);
        let before = f64::from(land.stocks[0][1]);
        let first = land.gather(&p, 0, 1, 1.0, 1.0);
        let second = land.gather(&p, 0, 1, 1.0, 1.0);
        assert!(first > 0.0 && second < first);
        let after = f64::from(land.stocks[0][1]);
        assert!((before - first - second - after).abs() < 1e-2);
        let all = land.gather(&p, 0, 1, 10_000.0, 1.0);
        assert!(all <= after + 1e-3);
        assert!(land.stocks[0][1] >= 0.0);
    }

    #[test]
    fn gathering_around_works_the_richest_neighbour() {
        let p = params();
        let mut land = Land::create(&map(), &p, 1, 400);
        assert_eq!(
            land.block(0).count(),
            4,
            "a corner patch has three neighbours"
        );
        let before: f32 = land.stocks[0].iter().sum();
        let got = land.gather_around(&p, 0, 0, 2.0, 1.0);
        let after: f32 = land.stocks[0].iter().sum();
        assert!(
            got > 0.0,
            "the water patch borrows from its land neighbours"
        );
        assert!((f64::from(before - after) - got).abs() < 1e-2);
    }

    #[test]
    fn the_climate_factor_varies_by_year_and_repeats_per_seed() {
        let mut p = params();
        p.climate_cv = 0.3;
        let mut a = Land::create(&map(), &p, 5, 400);
        let mut seen = Vec::new();
        for y in 2..8 {
            a.advance_to_day(&p, 5, y * 365);
            seen.push(a.climate.factor);
        }
        assert!(seen.windows(2).any(|w| w[0] != w[1]));
        assert!(seen.iter().all(|&f| f >= 0.05));
        let mut b = Land::create(&map(), &p, 5, 400);
        b.advance_to_day(&p, 5, 7 * 365);
        assert_eq!(a.climate, b.climate);
    }

    #[test]
    fn season_weights_interpolate_smoothly_across_the_year_end() {
        let mut season = [1.0; 12];
        season[0] = 0.0;
        season[11] = 2.0;
        // Mid-January is the January value; the new year falls between December and January.
        assert!((season_weight(&season, 15) - 0.0).abs() < 0.05);
        let new_year = season_weight(&season, 0);
        assert!(new_year > 0.5 && new_year < 1.5);
        let w: Vec<f64> = (0..365).map(|d| season_weight(&season, d)).collect();
        assert!(w.windows(2).all(|p| (p[1] - p[0]).abs() < 0.1));
    }
}
