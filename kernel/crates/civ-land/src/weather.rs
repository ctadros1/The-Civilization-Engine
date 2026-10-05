//! Weather (ADR-0012 §1): one daily series per world, the same at every speed.
//!
//! - **Rain**: whether a day is wet follows a two-state Markov chain with persistence, by month;
//!   a wet day's amount is the threshold plus a gamma draw, scaled so that each month's mean is
//!   its share of the landscape's annual total (research 03-03 §1.3, §2.2). Months vary: their
//!   totals are expectations, never quotas.
//! - **Temperature**: the day's mean is its date's normal plus an AR(1) anomaly. In summer a wet
//!   day is cooler than a dry one, and every wet day's range is narrower, with each month's mean
//!   kept (03-03 §1.4: temperature conditioned on rain).
//! - **The slow anomaly**: a monthly AR(1) that makes rain heavier and more frequent and summers
//!   cooler, carrying droughts and wet spells across seasons without drifting the mean (§1.6).
//! - **Height**: temperature falls with height (6.5 °C a km, §2.1). Snow falls when a day is near
//!   freezing, and lies and melts by degree-days in bands of 100 m.
//! - **Soil water**: a bucket under the wild cover (the reference) and one under each growing
//!   crop, drawn down by evapotranspiration from temperature (Hargreaves, FAO-56) and refilled by
//!   rain and melt; what the soil cannot hold drains away (§1.6's minimal balance).
//!
//! Each day is drawn from `[seed, weather, landscape, day]`: both landscapes of a seed have their
//! own history, and nothing but the day and yesterday's state decides it, so a day lived by the
//! minute and a day lived at once see the same weather (ADR-0011 §3). People see weather as it
//! happens, never future draws (03-03 §5.2).
//!
//! [`Climatology`] holds what many years of the same generator say about a landscape: the
//! long-run means the harvest and wild-plant factors are divided by, so an average year gives the
//! content's yields and the weather's variance is not counted twice (08-02 §7.3). It is computed
//! when a world is made or loaded, never saved.

use civ_core::Rng64;
use civ_core::time::{DAYS_PER_YEAR, MONTH_LENGTHS, MONTH_STARTS};

use crate::fields::CropParams;
use crate::{day_of_year, season_weight};

/// Purpose tag for a world's daily weather.
const PURPOSE_WEATHER: u64 = 0x7765_6174_6865_7231; // "weather1"
/// Purpose tag for the years a [`Climatology`] is drawn from.
const PURPOSE_CLIMATOLOGY: u64 = 0x636c_696d_6174_6f6c; // "climatol"
/// Purpose tag for the state a world's weather starts from.
const PURPOSE_START: u64 = 0x7773_7461_7274_3031; // "wstart01"

/// Height of a snow band, metres.
pub const SNOW_BAND_M: f64 = 100.0;
/// Snow lying, mm of water, from which the ground counts as snow-covered (about 5 cm of new snow;
/// a tuning value).
pub const SNOW_COVER_MM: f64 = 5.0;
/// Days over which the wild cover's ease of drawing water is smoothed: about a month (a tuning
/// value; ADR-0012 §5 has wild plants follow the month's soil water).
pub const EASE_DAYS: f64 = 30.0;
/// Years of weather a [`Climatology`] is drawn from, after one year to settle.
pub const CLIMATOLOGY_YEARS: usize = 300;

/// How a landscape's weather is drawn: the land profile's `[weather]`. Monthly values are for
/// January first; temperatures are interpolated between mid-month values.
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherParams {
    /// Share of days with at least `wet_day_mm` of rain or snow water, by month.
    pub wet_days: [f64; 12],
    /// Share of the year's precipitation that falls in each month (they sum to 1).
    pub rain_share: [f64; 12],
    /// Persistence of wet and dry days: rain after a wet day is likelier than after a dry one
    /// by this much (`p11 − p01`).
    pub persistence: f64,
    /// Shape of the gamma distribution of a wet day's amount above the threshold.
    pub gamma_shape: f64,
    /// The least precipitation that makes a day wet, mm.
    pub wet_day_mm: f64,
    /// Mean daily temperature by month at `normals_at_m`, °C.
    pub mean_c: [f64; 12],
    /// Standard deviation of the day's mean about its normal, by month, °C.
    pub day_sd_c: [f64; 12],
    /// Lag-one autocorrelation of the day's temperature anomaly.
    pub day_persistence: f64,
    /// Mean difference between the day's highest and lowest temperature, by month, °C.
    pub day_range_c: [f64; 12],
    /// A wet day's range as a share of the month's mean range; dry days get the rest.
    pub wet_day_range: f64,
    /// How much cooler a wet day is than a dry one, by month, °C.
    pub wet_day_cooling_c: [f64; 12],
    /// The height the temperatures are for, metres.
    pub normals_at_m: f64,
    /// Fall of temperature with height, °C per km.
    pub lapse_c_per_km: f64,
    /// Persistence of the slow anomaly, months.
    pub slow_months: f64,
    /// The slow anomaly's effect on a wet day's amount: `exp(σs − σ²/2)`, of mean 1.
    pub slow_amount: f64,
    /// The slow anomaly's effect on the share of wet days: `q × (1 + this × s)`.
    pub slow_wet_days: f64,
    /// The slow anomaly's effect on temperature, by month, °C per unit.
    pub slow_warmth_c: [f64; 12],
    /// Precipitation falls as snow when the day's mean is below this, °C.
    pub snow_below_c: f64,
    /// Snow melted per degree of the day's mean above freezing, mm of water per °C.
    pub melt_mm_per_c: f64,
    /// Water the soil holds for roots between field capacity and wilting, mm.
    pub soil_water_mm: f64,
    /// The share of that water plants draw without stress (FAO-56's `p`).
    pub easy_water_share: f64,
    /// The wild cover's water use as a multiple of the reference evapotranspiration.
    pub cover_kc: f64,
}

/// One day's weather at the height the normals are for.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeatherDay {
    /// Its day index.
    pub day: i64,
    /// Whether it is a wet day (the chain's state).
    pub wet: bool,
    /// Rain and snow, mm of water.
    pub precip_mm: f32,
    /// Mean temperature, °C.
    pub mean_c: f32,
    /// Lowest temperature, °C.
    pub min_c: f32,
    /// Highest temperature, °C.
    pub max_c: f32,
}

/// One month's weather at the height the normals are for, as it was.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MonthRecord {
    /// Its year (the first year of a world is 1).
    pub year: i64,
    /// Its month, 0 for January.
    pub month: u8,
    /// Days recorded so far.
    pub days: u8,
    /// Rain and snow, mm of water.
    pub precip_mm: f32,
    /// Wet days.
    pub wet_days: u8,
    /// The days' mean temperatures summed, °C.
    pub temp_sum_c: f32,
    /// The lowest temperature, °C.
    pub min_c: f32,
    /// The highest temperature, °C.
    pub max_c: f32,
    /// Days whose lowest temperature was below freezing.
    pub frost_days: u8,
    /// Days that ended with snow lying.
    pub snow_days: u8,
    /// The reference soil's water as a share of what it holds, summed over the days.
    pub soil_sum: f32,
}

impl MonthRecord {
    /// Its mean temperature, °C.
    pub fn mean_c(&self) -> f64 {
        f64::from(self.temp_sum_c) / f64::from(self.days.max(1))
    }

    /// The reference soil's mean water, as a share of what it holds.
    pub fn soil(&self) -> f64 {
        f64::from(self.soil_sum) / f64::from(self.days.max(1))
    }
}

/// How a crop uses water: its coefficients by stage and its response (ADR-0012 §2).
#[derive(Clone, Debug, PartialEq)]
pub struct CropWater {
    /// Crop coefficients at the start, in mid-season and at ripeness.
    pub kc: [f64; 3],
    /// Days of its initial, development, mid-season and late stages.
    pub kc_days: [u16; 4],
    /// Yield lost per share of its water need unmet.
    pub ky: f64,
    /// Days from sowing to ripeness.
    pub grow_days: u16,
    /// The day of the year it is typically sown: a third of the way into its window.
    pub typical_sowing: i64,
}

impl CropWater {
    /// How a crop of `crop`'s kind uses water.
    pub fn of(crop: &CropParams) -> CropWater {
        let window = i64::from(crop.sow_until_day) - i64::from(crop.sow_from_day);
        CropWater {
            kc: crop.kc,
            kc_days: crop.kc_days,
            ky: crop.ky,
            grow_days: crop.grow_days,
            typical_sowing: i64::from(crop.sow_from_day) + window.max(0) / 3,
        }
    }

    /// The crop coefficient on day `t` of its growth (0 is the day it was sown): flat through its
    /// initial stage, rising through development, flat in mid-season and falling to ripeness
    /// (FAO-56's curve).
    pub fn kc_on(&self, t: i64) -> f64 {
        let [ini, mid, end] = self.kc;
        let [a, b, c, d] = self.kc_days.map(i64::from);
        if t < a {
            ini
        } else if t < a + b {
            ini + (mid - ini) * (t - a) as f64 / b.max(1) as f64
        } else if t < a + b + c {
            mid
        } else {
            mid + (end - mid) * ((t - a - b - c) as f64 / d.max(1) as f64).min(1.0)
        }
    }

    /// The share of the yield a season's water allows, `1 − Ky (1 − got / needed)`, before it is
    /// divided by its long-run mean.
    pub fn raw_factor(&self, need_mm: f64, got_mm: f64) -> f64 {
        let ratio = if need_mm > 0.0 {
            (got_mm / need_mm).clamp(0.0, 1.0)
        } else {
            1.0
        };
        (1.0 - self.ky * (1.0 - ratio)).max(0.0)
    }
}

/// What a day's normals are, worked out once for each day of the year.
#[derive(Clone, Debug, PartialEq)]
struct DayNormals {
    mean_c: f64,
    sd_c: f64,
    range_c: f64,
    cooling_c: f64,
    slow_c: f64,
    /// Extraterrestrial radiation, MJ per m² per day.
    ra: f64,
}

/// What many years of a landscape's weather say about it, and the tables the generator draws
/// from. Derived when a world is made or loaded, never saved.
#[derive(Clone, Debug, PartialEq)]
pub struct Climatology {
    /// The key of a world's daily series: `[seed, weather, landscape]`.
    stream: u64,
    /// The landscape's mean annual precipitation, mm.
    pub annual_mm: f64,
    /// Normals by day of the year.
    days: Vec<DayNormals>,
    /// A wet day's mean amount by month at the normal share of wet days, mm.
    wet_day_mean_mm: [f64; 12],
    /// How each crop uses water, by index in the content's crops.
    pub crops: Vec<CropWater>,
    /// The long-run mean of each crop's raw water factor ([`CropWater::raw_factor`]).
    pub crop_mean: Vec<f64>,
    /// The long-run mean of the wild cover's ease of drawing water, by month.
    pub cover_mean: [f64; 12],
    /// The long-run mean precipitation by month, mm.
    pub month_precip_mm: [f64; 12],
}

/// The key of a landscape: its preset's id, hashed (FNV-1a).
pub fn landscape_key(preset_id: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in preset_id.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The month, 0 for January, of day index `day`.
pub fn month_of(day: i64) -> usize {
    let d = day_of_year(day);
    MONTH_STARTS[1..].iter().position(|&s| d < s).unwrap_or(11)
}

/// The year, the first being 1, of day index `day`.
pub fn year_of(day: i64) -> i64 {
    day.div_euclid(DAYS_PER_YEAR) + 1
}

/// A standard normal draw: Box–Muller from two uniforms in (0, 1].
pub(crate) fn normal(rng: &mut Rng64) -> f64 {
    let u1 = 1.0 - rng.next_f64();
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// A gamma draw of shape `k` and scale 1 (Marsaglia and Tsang; for `k < 1`, a draw of shape
/// `k + 1` times `U^(1/k)`).
fn gamma(rng: &mut Rng64, k: f64) -> f64 {
    if k.is_nan() || k <= 0.0 {
        return 0.0;
    }
    if k < 1.0 {
        let u = 1.0 - rng.next_f64();
        return gamma(rng, k + 1.0) * u.powf(1.0 / k);
    }
    let d = k - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    // Accepts about 95 % of tries or more; the bound only guards against a broken stream.
    for _ in 0..1000 {
        let x = normal(rng);
        let v = (1.0 + c * x).powi(3);
        if v <= 0.0 {
            continue;
        }
        let u = 1.0 - rng.next_f64();
        if u.ln() < 0.5 * x * x + d - d * v + d * v.ln() {
            return d * v;
        }
    }
    k
}

/// Extraterrestrial radiation on day of the year `doy` at `latitude_deg`, MJ per m² per day
/// (FAO-56 equations 21-25). Polar day and night are clamped.
pub fn extraterrestrial_radiation(latitude_deg: f64, doy: i64) -> f64 {
    let j = (doy + 1) as f64;
    let phi = latitude_deg.to_radians();
    let year = std::f64::consts::TAU * j / DAYS_PER_YEAR as f64;
    let dr = 1.0 + 0.033 * year.cos();
    let delta = 0.409 * (year - 1.39).sin();
    let ws = (-phi.tan() * delta.tan()).clamp(-1.0, 1.0).acos();
    let ra = 24.0 * 60.0 / std::f64::consts::PI
        * 0.0820
        * dr
        * (ws * phi.sin() * delta.sin() + phi.cos() * delta.cos() * ws.sin());
    ra.max(0.0)
}

/// Reference evapotranspiration from temperature (Hargreaves, FAO-56 equation 52), mm a day.
pub fn hargreaves_mm(mean_c: f64, range_c: f64, ra: f64) -> f64 {
    (0.0023 * (mean_c + 17.8) * range_c.max(0.0).sqrt() * ra * 0.408).max(0.0)
}

/// How easily plants draw the water in a root zone holding `water_mm` of `total_mm`: 1 until a
/// share `easy` of it is used, then falling to 0 at wilting (FAO-56's `Ks`).
pub fn ease(water_mm: f64, total_mm: f64, easy: f64) -> f64 {
    let hard = (1.0 - easy) * total_mm;
    if total_mm <= 0.0 {
        0.0
    } else if total_mm - water_mm <= easy * total_mm || hard <= 0.0 {
        1.0
    } else {
        (water_mm / hard).clamp(0.0, 1.0)
    }
}

/// One day of a root zone's water balance: what reaches it (`input_mm`) and what the crop or
/// cover would use at full ease (`need_mm`). Returns the water it used; what the zone cannot hold
/// drains away.
pub fn root_zone_day(
    water_mm: &mut f64,
    input_mm: f64,
    need_mm: f64,
    params: &WeatherParams,
) -> f64 {
    let total = params.soil_water_mm;
    let used = (need_mm * ease(*water_mm, total, params.easy_water_share))
        .min(*water_mm + input_mm)
        .max(0.0);
    *water_mm = (*water_mm + input_mm - used).clamp(0.0, total);
    used
}

impl Climatology {
    /// The climatology of a landscape: `annual_mm` its mean annual precipitation, at
    /// `latitude_deg`, for a world of `seed`. Draws [`CLIMATOLOGY_YEARS`] years of its weather,
    /// the same for every world of the landscape, with each crop sown on its typical day.
    pub fn new(
        params: &WeatherParams,
        crops: &[CropParams],
        annual_mm: f64,
        latitude_deg: f64,
        landscape: u64,
        seed: u64,
    ) -> Climatology {
        let share_sum: f64 = params.rain_share.iter().sum();
        let mut wet_day_mean_mm = [0.0; 12];
        for (m, mean) in wet_day_mean_mm.iter_mut().enumerate() {
            let share = if share_sum > 0.0 {
                params.rain_share[m] / share_sum
            } else {
                1.0 / 12.0
            };
            let wet = params.wet_days[m].max(1e-6) * MONTH_LENGTHS[m] as f64;
            *mean = (annual_mm.max(0.0) * share / wet).max(params.wet_day_mm);
        }
        let days = (0..DAYS_PER_YEAR)
            .map(|d| DayNormals {
                mean_c: season_weight(&params.mean_c, d),
                sd_c: season_weight(&params.day_sd_c, d),
                range_c: season_weight(&params.day_range_c, d),
                cooling_c: season_weight(&params.wet_day_cooling_c, d),
                slow_c: season_weight(&params.slow_warmth_c, d),
                ra: extraterrestrial_radiation(latitude_deg, d),
            })
            .collect();
        let mut c = Climatology {
            stream: civ_core::rng::key(&[seed, PURPOSE_WEATHER, landscape]),
            annual_mm,
            days,
            wet_day_mean_mm,
            crops: crops.iter().map(CropWater::of).collect(),
            crop_mean: vec![1.0; crops.len()],
            cover_mean: [1.0; 12],
            month_precip_mm: [0.0; 12],
        };
        c.measure(params, landscape);
        c
    }

    /// Draws the climatology's years, after one to settle, and sets the long-run means. The years
    /// are the landscape's, from a stream of their own, the same for every world.
    fn measure(&mut self, params: &WeatherParams, landscape: u64) {
        let world = self.stream;
        self.stream = civ_core::rng::key(&[PURPOSE_CLIMATOLOGY, landscape]);
        let at = params.normals_at_m;
        let mut w = Weather::new(params, self, at, at, 0);
        let n = self.crops.len();
        let mut crop_sum = vec![0.0; n];
        let (mut cover_sum, mut cover_days) = ([0.0; 12], [0.0f64; 12]);
        // Each crop's root-zone water, water needed and water got this season.
        let mut seasons = vec![(0.0, 0.0, 0.0); n];
        let years = CLIMATOLOGY_YEARS as i64;
        for day in 0..(years + 1) * DAYS_PER_YEAR {
            let counted = day >= DAYS_PER_YEAR;
            let doy = day_of_year(day);
            let water = w.live(params, self);
            for (i, cw) in self.crops.iter().enumerate() {
                let t = doy - cw.typical_sowing;
                if t < 0 || t >= i64::from(cw.grow_days) {
                    continue;
                }
                let s = &mut seasons[i];
                if t == 0 {
                    *s = (water.soil_before_mm, 0.0, 0.0);
                }
                let need = cw.kc_on(t) * water.et0_mm[0];
                let got = root_zone_day(&mut s.0, water.input_mm[0], need, params);
                s.1 += need;
                s.2 += got;
                if counted && t + 1 == i64::from(cw.grow_days) {
                    crop_sum[i] += cw.raw_factor(s.1, s.2);
                }
            }
            if counted {
                let m = month_of(day);
                cover_sum[m] += w.cover_ease;
                cover_days[m] += 1.0;
            }
        }
        self.stream = world;
        for r in w.months.iter().filter(|r| r.year > 1) {
            self.month_precip_mm[usize::from(r.month)] += f64::from(r.precip_mm) / years as f64;
        }
        for (mean, sum) in self.crop_mean.iter_mut().zip(&crop_sum) {
            *mean = (sum / years as f64).max(0.05);
        }
        for m in 0..12 {
            if cover_days[m] > 0.0 {
                self.cover_mean[m] = (cover_sum[m] / cover_days[m]).max(0.05);
            }
        }
    }

    /// The share of its yield a crop of kind `crop` gives for a season that needed `need_mm` of
    /// water and got `got_mm`: 1 in a year of average water (ADR-0012 §2).
    pub fn crop_factor(&self, crop: usize, need_mm: f64, got_mm: f64) -> f64 {
        match (self.crops.get(crop), self.crop_mean.get(crop)) {
            (Some(cw), Some(&mean)) => cw.raw_factor(need_mm, got_mm) / mean,
            _ => 1.0,
        }
    }

    /// The share of their usual growth wild plants make with the cover's ease `ease` in month
    /// `month`: 1 in a month of average soil water.
    pub fn cover_factor(&self, ease: f64, month: usize) -> f64 {
        ease / self.cover_mean[month.min(11)]
    }

    fn normals(&self, day: i64) -> &DayNormals {
        &self.days[day_of_year(day) as usize]
    }
}

/// What reached the ground on a day, by snow band.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DayWater {
    /// Rain and melt reaching the ground, mm, by band.
    pub input_mm: Vec<f64>,
    /// Reference evapotranspiration, mm, by band.
    pub et0_mm: Vec<f64>,
    /// The reference soil's water at the start of the day, mm.
    pub soil_before_mm: f64,
}

/// A world's weather: today's, what tomorrow depends on, and the record. Saved.
#[derive(Clone, Debug, PartialEq)]
pub struct Weather {
    /// Today's weather: the day after the last one lived.
    pub today: WeatherDay,
    /// Today's temperature anomaly, in standard deviations.
    pub anomaly: f64,
    /// This month's slow anomaly, in standard deviations.
    pub slow: f64,
    /// Bottom of the lowest snow band, metres.
    pub snow_base_m: f32,
    /// Snow lying, mm of water, by band of [`SNOW_BAND_M`] from `snow_base_m`.
    pub snow_mm: Vec<f32>,
    /// Water in the reference soil under the wild cover, mm.
    pub soil_mm: f64,
    /// The wild cover's ease of drawing water, smoothed over about a month (0 to 1).
    pub cover_ease: f64,
    /// Each month's record, oldest first; the last is the month in progress.
    pub months: Vec<MonthRecord>,
}

impl Weather {
    /// A world's weather from day `first`: the slow and daily anomalies drawn from where they
    /// settle, the soil at field capacity and no snow, its bands covering heights `low_m` to
    /// `high_m`. Draws day `first`.
    pub fn new(
        params: &WeatherParams,
        climate: &Climatology,
        low_m: f64,
        high_m: f64,
        first: i64,
    ) -> Weather {
        let mut rng = Rng64::from_key(&[climate.stream, PURPOSE_START, first as u64]);
        let slow = normal(&mut rng);
        let anomaly = normal(&mut rng);
        let at = params.normals_at_m;
        let (low, high) = (low_m.min(at), high_m.max(at));
        // Bands are centred on the normals' height, so its band's temperature is the normal.
        let below = ((at - SNOW_BAND_M / 2.0 - low) / SNOW_BAND_M)
            .ceil()
            .max(0.0);
        let base = at - SNOW_BAND_M / 2.0 - below * SNOW_BAND_M;
        let bands = (((high - base) / SNOW_BAND_M).floor().max(0.0) as usize + 1).min(1000);
        let mut w = Weather {
            today: WeatherDay::default(),
            anomaly,
            slow,
            snow_base_m: base as f32,
            snow_mm: vec![0.0; bands],
            soil_mm: params.soil_water_mm,
            cover_ease: 1.0,
            months: Vec::new(),
        };
        w.draw(params, climate, first);
        w
    }

    /// A world's weather restored from a save older than weather (ADR-0012 §6): its slow anomaly
    /// the old year's deviate, the soil at field capacity and no snow. Draws day `today`.
    pub fn from_old(
        params: &WeatherParams,
        climate: &Climatology,
        low_m: f64,
        high_m: f64,
        today: i64,
        deviate: f64,
    ) -> Weather {
        let mut w = Weather::new(params, climate, low_m, high_m, today);
        w.slow = if deviate.is_finite() { deviate } else { 0.0 };
        w.anomaly = 0.0;
        w.today.wet = false;
        w.draw(params, climate, today);
        w
    }

    /// The snow band of height `z_m`.
    pub fn band_of(&self, z_m: f64) -> usize {
        let b = ((z_m - f64::from(self.snow_base_m)) / SNOW_BAND_M).floor();
        (b.max(0.0) as usize).min(self.snow_mm.len().saturating_sub(1))
    }

    /// The middle height of band `band`, metres.
    pub fn band_mid_m(&self, band: usize) -> f64 {
        f64::from(self.snow_base_m) + (band as f64 + 0.5) * SNOW_BAND_M
    }

    /// The band the normals are for.
    pub fn reference_band(&self, params: &WeatherParams) -> usize {
        self.band_of(params.normals_at_m)
    }

    /// Today's mean temperature at height `z_m`, °C.
    pub fn mean_c_at(&self, params: &WeatherParams, z_m: f64) -> f64 {
        f64::from(self.today.mean_c) - params.lapse_c_per_km * (z_m - params.normals_at_m) / 1000.0
    }

    /// Snow lying at height `z_m`, mm of water.
    pub fn snow_at(&self, z_m: f64) -> f64 {
        f64::from(self.snow_mm.get(self.band_of(z_m)).copied().unwrap_or(0.0))
    }

    /// Draws day `day` into `today` from yesterday's state.
    fn draw(&mut self, params: &WeatherParams, climate: &Climatology, day: i64) {
        let mut rng = Rng64::from_key(&[climate.stream, day as u64]);
        let month = month_of(day);
        if MONTH_STARTS[month] == day_of_year(day) {
            let a = (-1.0 / params.slow_months.max(1e-6)).exp();
            self.slow = a * self.slow + (1.0 - a * a).sqrt() * normal(&mut rng);
        }
        let q0 = params.wet_days[month];
        let q = (q0 * (1.0 + params.slow_wet_days * self.slow)).clamp(0.02, 0.95);
        let rho = params.persistence;
        let p = if self.today.wet && self.today.day == day - 1 {
            q + rho * (1.0 - q)
        } else {
            q * (1.0 - rho)
        };
        let wet = rng.next_f64() < p;
        let precip = if wet {
            let k = params.gamma_shape.max(0.05);
            let scale = (climate.wet_day_mean_mm[month] - params.wet_day_mm).max(0.0) / k;
            let sigma = params.slow_amount;
            (params.wet_day_mm + gamma(&mut rng, k) * scale)
                * (sigma * self.slow - sigma * sigma / 2.0).exp()
        } else {
            0.0
        };
        let phi = params.day_persistence.clamp(-0.99, 0.99);
        self.anomaly = phi * self.anomaly + (1.0 - phi * phi).sqrt() * normal(&mut rng);
        let n = climate.normals(day);
        // Wet days cooler and dry days warmer by their shares, so the month's mean is kept.
        let shift = if wet {
            -n.cooling_c * (1.0 - q0)
        } else {
            n.cooling_c * q0
        };
        let mean = n.mean_c + n.sd_c * self.anomaly + n.slow_c * self.slow + shift;
        let r_wet = params.wet_day_range.clamp(0.0, 1.0);
        let range = n.range_c
            * if wet {
                r_wet
            } else {
                (1.0 - q0 * r_wet) / (1.0 - q0).max(1e-6)
            };
        self.today = WeatherDay {
            day,
            wet,
            precip_mm: precip as f32,
            mean_c: mean as f32,
            min_c: (mean - range / 2.0) as f32,
            max_c: (mean + range / 2.0) as f32,
        };
    }

    /// Lives today: snow falls, lies and melts in each band, the reference soil gains and loses
    /// water, and the month's record grows; then tomorrow is drawn. Returns what reached the
    /// ground in each band, for the fields.
    pub fn live(&mut self, params: &WeatherParams, climate: &Climatology) -> DayWater {
        let day = self.today.day;
        let ra = climate.normals(day).ra;
        let precip = f64::from(self.today.precip_mm);
        let range = f64::from(self.today.max_c - self.today.min_c);
        let bands = self.snow_mm.len();
        let mut water = DayWater {
            input_mm: Vec::with_capacity(bands),
            et0_mm: Vec::with_capacity(bands),
            soil_before_mm: self.soil_mm,
        };
        for b in 0..bands {
            let t = self.mean_c_at(params, self.band_mid_m(b));
            let mut snow = f64::from(self.snow_mm[b]);
            let mut input = 0.0;
            if t < params.snow_below_c {
                snow += precip;
            } else {
                input += precip;
            }
            let melt = (params.melt_mm_per_c * t.max(0.0)).min(snow);
            snow -= melt;
            input += melt;
            self.snow_mm[b] = snow as f32;
            water.input_mm.push(input);
            water.et0_mm.push(hargreaves_mm(t, range, ra));
        }
        let r = self.reference_band(params);
        let need = params.cover_kc * water.et0_mm[r];
        root_zone_day(&mut self.soil_mm, water.input_mm[r], need, params);
        let now = ease(self.soil_mm, params.soil_water_mm, params.easy_water_share);
        self.cover_ease += (now - self.cover_ease) / EASE_DAYS;
        self.record(params, r);
        self.draw(params, climate, day + 1);
        water
    }

    /// Adds today to its month's record, beginning the month's record on its first day lived.
    fn record(&mut self, params: &WeatherParams, band: usize) {
        let t = self.today;
        let (year, month) = (year_of(t.day), month_of(t.day) as u8);
        if self
            .months
            .last()
            .is_none_or(|m| m.year != year || m.month != month)
        {
            self.months.push(MonthRecord {
                year,
                month,
                min_c: f32::INFINITY,
                max_c: f32::NEG_INFINITY,
                ..MonthRecord::default()
            });
        }
        let soil = (self.soil_mm / params.soil_water_mm.max(1e-9)) as f32;
        let snow = f64::from(self.snow_mm[band]) >= SNOW_COVER_MM;
        if let Some(m) = self.months.last_mut() {
            m.days = m.days.saturating_add(1);
            m.precip_mm += t.precip_mm;
            m.wet_days += u8::from(f64::from(t.precip_mm) >= params.wet_day_mm);
            m.temp_sum_c += t.mean_c;
            m.min_c = m.min_c.min(t.min_c);
            m.max_c = m.max_c.max(t.max_c);
            m.frost_days += u8::from(t.min_c < 0.0);
            m.snow_days += u8::from(snow);
            m.soil_sum += soil;
        }
    }

    /// What is wrong with the weather's state for land last lived to `stock_day`, if anything.
    pub fn problems(&self, stock_day: i64) -> Vec<String> {
        let mut out = Vec::new();
        let t = &self.today;
        let finite = [t.precip_mm, t.mean_c, t.min_c, t.max_c, self.snow_base_m]
            .iter()
            .all(|v| v.is_finite())
            && [self.anomaly, self.slow, self.soil_mm, self.cover_ease]
                .iter()
                .all(|v| v.is_finite());
        if !finite || t.precip_mm < 0.0 || t.min_c > t.max_c {
            out.push("the weather is not a number or out of order".to_owned());
        }
        if t.day != stock_day + 1 {
            out.push(format!(
                "the weather is for day {}, not the day after the land's last ({stock_day})",
                t.day
            ));
        }
        if self.snow_mm.is_empty() || self.snow_mm.iter().any(|s| !(s.is_finite() && *s >= 0.0)) {
            out.push("the snow lying is malformed".to_owned());
        }
        if self.soil_mm < 0.0 || !(0.0..=1.0).contains(&self.cover_ease) {
            out.push("the soil water is out of range".to_owned());
        }
        let bad_month = |m: &MonthRecord| {
            m.month > 11
                || m.days == 0
                || m.days > 31
                || !(m.precip_mm.is_finite() && m.precip_mm >= 0.0 && m.temp_sum_c.is_finite())
        };
        if self.months.iter().any(bad_month) {
            out.push("a month's weather record is malformed".to_owned());
        }
        out
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::fields::tests::crop;

    /// The temperate valley's `[weather]` (content/core/land/temperate_valley.toml).
    pub(crate) fn params() -> WeatherParams {
        WeatherParams {
            wet_days: [
                0.38, 0.36, 0.37, 0.38, 0.42, 0.43, 0.41, 0.39, 0.36, 0.37, 0.40, 0.40,
            ],
            rain_share: [
                0.066, 0.061, 0.066, 0.071, 0.101, 0.117, 0.112, 0.101, 0.081, 0.076, 0.071, 0.077,
            ],
            persistence: 0.3,
            gamma_shape: 0.8,
            wet_day_mm: 1.0,
            mean_c: [
                -1.0, 0.5, 4.5, 9.0, 13.5, 17.0, 19.0, 18.5, 14.5, 9.5, 4.0, 0.5,
            ],
            day_sd_c: [4.0, 4.0, 3.5, 3.0, 2.8, 2.5, 2.5, 2.5, 2.8, 3.0, 3.5, 4.0],
            day_persistence: 0.7,
            day_range_c: [
                6.0, 7.0, 9.0, 11.0, 11.5, 11.5, 12.0, 12.0, 11.0, 9.0, 6.5, 5.5,
            ],
            wet_day_range: 0.75,
            wet_day_cooling_c: [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
            normals_at_m: 150.0,
            lapse_c_per_km: 6.5,
            slow_months: 12.0,
            slow_amount: 0.1,
            slow_wet_days: 0.1,
            slow_warmth_c: [
                0.0, 0.0, 0.0, 0.0, -0.8, -0.8, -0.8, -0.8, -0.8, 0.0, 0.0, 0.0,
            ],
            snow_below_c: 0.5,
            melt_mm_per_c: 3.0,
            soil_water_mm: 100.0,
            easy_water_share: 0.55,
            cover_kc: 0.9,
        }
    }

    fn climate(seed: u64, landscape: u64) -> Climatology {
        Climatology::new(&params(), &[crop()], 800.0, 48.0, landscape, seed)
    }

    /// Lives `years` years of a world's weather from day 0, with a crop of the test kind sown
    /// on its typical day each year; returns the weather and each year's crop factor.
    fn live(c: &Climatology, years: i64) -> (Weather, Vec<f64>) {
        let p = params();
        let mut w = Weather::new(&p, c, 100.0, 1100.0, 0);
        let cw = &c.crops[0];
        let mut factors = Vec::new();
        let (mut water, mut need, mut got) = (0.0, 0.0, 0.0);
        for day in 0..years * DAYS_PER_YEAR {
            let dw = w.live(&p, c);
            let t = day_of_year(day) - cw.typical_sowing;
            if t == 0 {
                (water, need, got) = (dw.soil_before_mm, 0.0, 0.0);
            }
            if (0..i64::from(cw.grow_days)).contains(&t) {
                let r = w.reference_band(&p);
                let n = cw.kc_on(t) * dw.et0_mm[r];
                got += root_zone_day(&mut water, dw.input_mm[r], n, &p);
                need += n;
                if t + 1 == i64::from(cw.grow_days) {
                    factors.push(c.crop_factor(0, need, got));
                }
            }
        }
        (w, factors)
    }

    #[test]
    fn a_world_has_its_own_weather_and_the_same_weather_each_time() {
        let a = climate(1, landscape_key("core:worldgen/river_valley"));
        let b = climate(1, landscape_key("core:worldgen/river_valley"));
        let coast = climate(1, landscape_key("core:worldgen/ria_coast"));
        let other_seed = climate(2, landscape_key("core:worldgen/river_valley"));
        let (wa, fa) = live(&a, 3);
        let (wb, fb) = live(&b, 3);
        assert_eq!(wa, wb);
        assert_eq!(fa, fb);
        // Each landscape of a seed, and each seed, lives its own days.
        assert_ne!(live(&coast, 3).0.months, wa.months);
        assert_ne!(live(&other_seed, 3).0.months, wa.months);
        // The climatology is the landscape's, whatever the world.
        assert_eq!(a.crop_mean, other_seed.crop_mean);
    }

    #[test]
    fn five_hundred_years_keep_to_the_profile() {
        let p = params();
        let c = climate(7, landscape_key("core:worldgen/river_valley"));
        let (w, factors) = live(&c, 500);
        let years = w.months.len() as f64 / 12.0;
        let annual: f64 = w.months.iter().map(|m| f64::from(m.precip_mm)).sum::<f64>() / years;
        let wet: f64 = w.months.iter().map(|m| f64::from(m.wet_days)).sum::<f64>() / years;
        let target_wet: f64 = (0..12)
            .map(|m| p.wet_days[m] * MONTH_LENGTHS[m] as f64)
            .sum();
        assert!(
            (annual - 800.0).abs() < 800.0 * 0.04,
            "annual {annual:.0} mm"
        );
        assert!(
            (wet - target_wet).abs() < target_wet * 0.06,
            "wet days {wet:.0} of {target_wet:.0}"
        );
        for m in 0..12 {
            let months: Vec<&MonthRecord> = w
                .months
                .iter()
                .filter(|r| usize::from(r.month) == m)
                .collect();
            let mean = months.iter().map(|r| r.mean_c()).sum::<f64>() / months.len() as f64;
            assert!(
                (mean - p.mean_c[m]).abs() < 0.6,
                "month {m}: {mean:.2} °C against {}",
                p.mean_c[m]
            );
        }
        // Winters bring frost and some snow lying; summers none.
        let jan: Vec<&MonthRecord> = w.months.iter().filter(|r| r.month == 0).collect();
        let jul: Vec<&MonthRecord> = w.months.iter().filter(|r| r.month == 6).collect();
        let frost = jan.iter().map(|r| f64::from(r.frost_days)).sum::<f64>() / jan.len() as f64;
        let snow = jan.iter().map(|r| f64::from(r.snow_days)).sum::<f64>() / jan.len() as f64;
        assert!(frost > 15.0, "January frost days {frost:.1}");
        assert!(snow > 2.0 && snow < 25.0, "January snow days {snow:.1}");
        assert!(jul.iter().all(|r| r.snow_days == 0));
        // The harvest factor of a world's own years: mean 1, and a spread within rainfed farming's
        // (research 08-02 §7.3: 0.20-0.35 in favourable climates, all causes together).
        let n = factors.len() as f64;
        let mean = factors.iter().sum::<f64>() / n;
        let sd = (factors.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / n).sqrt();
        assert!((mean - 1.0).abs() < 0.03, "crop factor mean {mean:.3}");
        assert!(
            (0.15..=0.30).contains(&(sd / mean)),
            "crop factor CV {:.3}",
            sd / mean
        );
    }

    #[test]
    fn snow_lies_longer_higher_up_and_melts_by_degree_days() {
        let p = params();
        let c = climate(3, landscape_key("core:worldgen/river_valley"));
        let mut w = Weather::new(&p, &c, 100.0, 1100.0, 0);
        let (low, high) = (w.band_of(150.0), w.band_of(1050.0));
        assert!(
            (w.band_mid_m(low) - 150.0).abs() < 1e-9,
            "the normals' band is centred on them"
        );
        let (mut low_days, mut high_days) = (0, 0);
        for _ in 0..20 * DAYS_PER_YEAR {
            w.live(&p, &c);
            low_days += i32::from(f64::from(w.snow_mm[low]) >= SNOW_COVER_MM);
            high_days += i32::from(f64::from(w.snow_mm[high]) >= SNOW_COVER_MM);
        }
        assert!(
            high_days > 2 * low_days,
            "{high_days} snow days at 1,050 m, {low_days} at 150 m"
        );
        // A day 10 °C above freezing melts 30 mm of water.
        assert!((p.melt_mm_per_c * 10.0 - 30.0).abs() < 1e-9);
    }

    #[test]
    fn hargreaves_gives_summer_and_winter_demand_of_the_right_size() {
        // FAO-56: humid conditions at 15-25 °C, 3-4 mm a day (research 03-02 §2.1).
        let july = hargreaves_mm(19.0, 12.0, extraterrestrial_radiation(48.0, 196));
        let january = hargreaves_mm(-1.0, 6.0, extraterrestrial_radiation(48.0, 15));
        assert!((3.0..=5.0).contains(&july), "July {july:.2} mm");
        assert!(january < 0.6, "January {january:.2} mm");
        // Polar night has no sun.
        assert_eq!(extraterrestrial_radiation(80.0, 355), 0.0);
    }

    #[test]
    fn the_root_zone_holds_what_it_can_and_stress_begins_past_the_easy_water() {
        let p = params();
        let mut water = 100.0;
        // Rain beyond what it holds drains away.
        root_zone_day(&mut water, 30.0, 0.0, &p);
        assert_eq!(water, 100.0);
        // A crop draws freely until 55 mm are used, then less as it dries.
        assert_eq!(ease(50.0, 100.0, 0.55), 1.0);
        assert!((ease(30.0, 100.0, 0.55) - 30.0 / 45.0).abs() < 1e-12);
        let used = root_zone_day(&mut water, 0.0, 5.0, &p);
        assert_eq!(used, 5.0);
        let mut dry = 9.0;
        assert!((root_zone_day(&mut dry, 0.0, 5.0, &p) - 1.0).abs() < 1e-12);
        // The crop curve: flat, rising, flat, falling.
        let cw = CropWater {
            kc: [0.4, 1.15, 0.4],
            kc_days: [30, 30, 40, 20],
            ky: 1.15,
            grow_days: 120,
            typical_sowing: 95,
        };
        assert_eq!(cw.kc_on(0), 0.4);
        assert!((cw.kc_on(45) - 0.775).abs() < 1e-12);
        assert_eq!(cw.kc_on(80), 1.15);
        assert!((cw.kc_on(119) - (1.15 - 0.75 * 19.0 / 20.0)).abs() < 1e-12);
        // Ky: a fifth of the need unmet costs 23 % of the yield (08-02 §7.2).
        assert!((cw.raw_factor(100.0, 80.0) - 0.77).abs() < 1e-12);
    }
}
