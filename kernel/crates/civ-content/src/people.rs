//! People profiles (`kind = "people"`): how a world's people live, and their founding bands.
//! Every field is required: an omitted number is an error, never a silent engine default.

use civ_agents::params::{
    BandParams, DecisionParams, EnergyParams, HouseholdParams, NameParams, PeopleParams, Siler,
    SleepParams, SocialParams,
};
use civ_world::nav::NavParams;
use serde::Deserialize;

/// The `kind` value of a people profile.
pub const KIND: &str = "people";
/// The id segment: `pack:people/name`.
pub const ID_KIND: &str = "people";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PeopleFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub names: String,
    pub latitude_deg: f64,
    pub walk_speed_by_age: Vec<[f64; 2]>,
    pub capacity_by_age: Vec<[f64; 2]>,
    pub walking: Walking,
    pub energy: Energy,
    pub sleep: Sleep,
    pub social: Social,
    pub household: Household,
    pub decision: Decision,
    pub band: Band,
    pub mortality: Mortality,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Walking {
    pub top_speed_kmh: f64,
    pub slope_sensitivity: f64,
    pub best_slope_offset: f64,
    pub offtrail_factor: f64,
    pub wading_factor: f64,
    pub ford_max_discharge_m3s: f64,
    pub max_slope: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Energy {
    pub bmr_band_starts: Vec<f64>,
    pub bmr_male: Vec<[f64; 2]>,
    pub bmr_female: Vec<[f64; 2]>,
    pub mass_by_age: Vec<[f64; 3]>,
    pub walk_par: f64,
    pub idle_par: f64,
    pub satiety_hours: f64,
    pub hunger_ramp_hours: f64,
    pub deficit_unit_kcal: f64,
    pub max_surplus_kcal: f64,
    pub reserve_kcal_per_kg: f64,
    pub meal_minutes: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Sleep {
    pub tau_awake_h: f64,
    pub tau_asleep_h: f64,
    pub wake_pressure: f64,
    pub min_hours: f64,
    pub max_hours: f64,
    pub nap_min_minutes: f64,
    pub nap_max_minutes: f64,
    pub bedtime_after_sunset_hours: f64,
    pub day_factor: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Social {
    pub tau_h: f64,
    pub quality_per_companion: f64,
    pub household_quality: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Household {
    pub water_l_per_person_day: f64,
    pub carry_water_l: f64,
    pub water_target_days: f64,
    pub food_target_days: f64,
    pub carry_kg: f64,
    pub fuel_kg_per_person_day: [f64; 12],
    pub fuel_target_days: f64,
    pub short_food_days: f64,
    pub recovered_food_days: f64,
    pub daily_kcal_per_person: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Decision {
    pub temperature_sd_fraction: f64,
    pub min_temperature: f64,
    pub w_hunger: f64,
    pub w_sleep: f64,
    pub w_social: f64,
    pub w_food: f64,
    pub w_work: f64,
    pub w_fuel: f64,
    pub trip_half_worth_days: f64,
    pub w_water: f64,
    pub w_walk_hour: f64,
    pub w_effort: f64,
    pub w_dark: f64,
    pub w_rest: f64,
    pub w_play: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Band {
    pub default_size: u32,
    pub min_size: u32,
    pub max_size: u32,
    pub min_families: u32,
    pub camp_candidates: u32,
    pub site_radius_m: f64,
    pub provisions_days: f64,
    /// The good provisions are carried as: a good id.
    pub provisions_good: String,
    pub elder_chance: f64,
    pub young_adult_chance: f64,
    pub birth_spacing_months: f64,
    pub site_max_slope: f64,
    pub site_w_food: f64,
    pub site_w_water_per_100m: f64,
    pub site_w_slope_per_pct: f64,
    pub site_w_flood: f64,
    pub site_flood_hand_m: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Mortality {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
}

fn pairs(v: &[[f64; 2]]) -> Vec<(f64, f64)> {
    v.iter().map(|&[a, b]| (a, b)).collect()
}

fn ascending(name: &str, xs: impl Iterator<Item = f64>, problems: &mut Vec<String>) {
    let xs: Vec<f64> = xs.collect();
    if xs.is_empty() {
        problems.push(format!("`{name}` must not be empty"));
    } else if xs.windows(2).any(|w| w[1] <= w[0]) || xs.iter().any(|x| !x.is_finite()) {
        problems.push(format!(
            "`{name}` ages must be finite and strictly ascending"
        ));
    }
}

fn positive(name: &str, v: f64, problems: &mut Vec<String>) {
    if !(v.is_finite() && v > 0.0) {
        problems.push(format!("`{name}` must be positive (got {v})"));
    }
}

fn non_negative(name: &str, v: f64, problems: &mut Vec<String>) {
    if !(v.is_finite() && v >= 0.0) {
        problems.push(format!("`{name}` must be zero or more (got {v})"));
    }
}

fn unit(name: &str, v: f64, problems: &mut Vec<String>) {
    if !(v.is_finite() && (0.0..=1.0).contains(&v)) {
        problems.push(format!("`{name}` must be between 0 and 1 (got {v})"));
    }
}

impl PeopleFile {
    /// The parameters, with the given names and the index of the provisions good. Field by field
    /// on purpose: a new parameter fails to compile here until the authoring format carries it.
    pub fn params(&self, names: NameParams, provisions_good: usize) -> PeopleParams {
        let (w, e, s, so, h, d, b, m) = (
            &self.walking,
            &self.energy,
            &self.sleep,
            &self.social,
            &self.household,
            &self.decision,
            &self.band,
            &self.mortality,
        );
        PeopleParams {
            nav: NavParams {
                top_speed_kmh: w.top_speed_kmh,
                slope_sensitivity: w.slope_sensitivity,
                best_slope_offset: w.best_slope_offset,
                offtrail_factor: w.offtrail_factor,
                wading_factor: w.wading_factor,
                ford_max_discharge_m3s: w.ford_max_discharge_m3s,
                max_slope: w.max_slope,
            },
            walk_speed_by_age: pairs(&self.walk_speed_by_age),
            capacity_by_age: pairs(&self.capacity_by_age),
            latitude_deg: self.latitude_deg,
            energy: EnergyParams {
                bmr_male: pairs(&e.bmr_male),
                bmr_female: pairs(&e.bmr_female),
                bmr_band_starts: e.bmr_band_starts.clone(),
                mass_by_age: e.mass_by_age.iter().map(|&[a, x, y]| (a, x, y)).collect(),
                walk_par: e.walk_par,
                idle_par: e.idle_par,
                satiety_hours: e.satiety_hours,
                hunger_ramp_hours: e.hunger_ramp_hours,
                deficit_unit_kcal: e.deficit_unit_kcal,
                max_surplus_kcal: e.max_surplus_kcal,
                reserve_kcal_per_kg: e.reserve_kcal_per_kg,
                meal_minutes: e.meal_minutes,
            },
            sleep: SleepParams {
                tau_awake_h: s.tau_awake_h,
                tau_asleep_h: s.tau_asleep_h,
                wake_pressure: s.wake_pressure,
                min_hours: s.min_hours,
                max_hours: s.max_hours,
                nap_min_minutes: s.nap_min_minutes,
                nap_max_minutes: s.nap_max_minutes,
                bedtime_after_sunset_hours: s.bedtime_after_sunset_hours,
                day_factor: s.day_factor,
            },
            social: SocialParams {
                tau_h: so.tau_h,
                quality_per_companion: so.quality_per_companion,
                household_quality: so.household_quality,
            },
            household: HouseholdParams {
                water_l_per_person_day: h.water_l_per_person_day,
                carry_water_l: h.carry_water_l,
                water_target_days: h.water_target_days,
                food_target_days: h.food_target_days,
                carry_kg: h.carry_kg,
                fuel_kg_per_person_day: h.fuel_kg_per_person_day,
                fuel_target_days: h.fuel_target_days,
                short_food_days: h.short_food_days,
                recovered_food_days: h.recovered_food_days,
                daily_kcal_per_person: h.daily_kcal_per_person,
            },
            decision: DecisionParams {
                temperature_sd_fraction: d.temperature_sd_fraction,
                min_temperature: d.min_temperature,
                w_hunger: d.w_hunger,
                w_sleep: d.w_sleep,
                w_social: d.w_social,
                w_food: d.w_food,
                w_work: d.w_work,
                w_fuel: d.w_fuel,
                trip_half_worth_days: d.trip_half_worth_days,
                w_water: d.w_water,
                w_walk_hour: d.w_walk_hour,
                w_effort: d.w_effort,
                w_dark: d.w_dark,
                w_rest: d.w_rest,
                w_play: d.w_play,
            },
            band: BandParams {
                default_size: b.default_size,
                min_size: b.min_size,
                max_size: b.max_size,
                min_families: b.min_families,
                camp_candidates: b.camp_candidates,
                site_radius_m: b.site_radius_m,
                provisions_days: b.provisions_days,
                provisions_good,
                elder_chance: b.elder_chance,
                young_adult_chance: b.young_adult_chance,
                birth_spacing_months: b.birth_spacing_months,
                site_max_slope: b.site_max_slope,
                site_w_food: b.site_w_food,
                site_w_water_per_100m: b.site_w_water_per_100m,
                site_w_slope_per_pct: b.site_w_slope_per_pct,
                site_w_flood: b.site_w_flood,
                site_flood_hand_m: b.site_flood_hand_m,
            },
            mortality: Siler {
                a: m.a,
                b: m.b,
                c: m.c,
                d: m.d,
                e: m.e,
            },
            names,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if !(self.latitude_deg.is_finite() && self.latitude_deg.abs() <= 66.0) {
            p.push(format!(
                "`latitude_deg` must be between -66 and 66 (got {})",
                self.latitude_deg
            ));
        }
        ascending(
            "walk_speed_by_age",
            self.walk_speed_by_age.iter().map(|x| x[0]),
            &mut p,
        );
        ascending(
            "capacity_by_age",
            self.capacity_by_age.iter().map(|x| x[0]),
            &mut p,
        );
        for &[_, v] in self.walk_speed_by_age.iter().chain(&self.capacity_by_age) {
            if !(v.is_finite() && (0.0..=2.0).contains(&v)) {
                p.push(format!("age factors must be between 0 and 2 (got {v})"));
            }
        }
        let w = &self.walking;
        positive("walking.top_speed_kmh", w.top_speed_kmh, &mut p);
        non_negative("walking.slope_sensitivity", w.slope_sensitivity, &mut p);
        unit("walking.offtrail_factor", w.offtrail_factor, &mut p);
        unit("walking.wading_factor", w.wading_factor, &mut p);
        non_negative(
            "walking.ford_max_discharge_m3s",
            w.ford_max_discharge_m3s,
            &mut p,
        );
        positive("walking.max_slope", w.max_slope, &mut p);
        if w.offtrail_factor <= 0.0 {
            p.push("`walking.offtrail_factor` must be above 0".to_owned());
        }
        let e = &self.energy;
        let bands = e.bmr_band_starts.len();
        if bands == 0 || e.bmr_male.len() != bands || e.bmr_female.len() != bands {
            p.push(
                "`energy.bmr_male` and `energy.bmr_female` need one entry per band start"
                    .to_owned(),
            );
        }
        ascending(
            "energy.bmr_band_starts",
            e.bmr_band_starts.iter().copied(),
            &mut p,
        );
        ascending(
            "energy.mass_by_age",
            e.mass_by_age.iter().map(|x| x[0]),
            &mut p,
        );
        for &[_, m, f] in &e.mass_by_age {
            if !(m > 0.0 && f > 0.0 && m.is_finite() && f.is_finite()) {
                p.push("body masses must be positive".to_owned());
            }
        }
        positive("energy.walk_par", e.walk_par, &mut p);
        positive("energy.idle_par", e.idle_par, &mut p);
        positive("energy.satiety_hours", e.satiety_hours, &mut p);
        positive("energy.hunger_ramp_hours", e.hunger_ramp_hours, &mut p);
        positive("energy.deficit_unit_kcal", e.deficit_unit_kcal, &mut p);
        non_negative("energy.max_surplus_kcal", e.max_surplus_kcal, &mut p);
        positive("energy.reserve_kcal_per_kg", e.reserve_kcal_per_kg, &mut p);
        if e.meal_minutes == 0 {
            p.push("`energy.meal_minutes` must be at least 1".to_owned());
        }
        let s = &self.sleep;
        positive("sleep.tau_awake_h", s.tau_awake_h, &mut p);
        positive("sleep.tau_asleep_h", s.tau_asleep_h, &mut p);
        if !(s.wake_pressure > 0.0 && s.wake_pressure < 1.0) {
            p.push("`sleep.wake_pressure` must be between 0 and 1".to_owned());
        }
        positive("sleep.min_hours", s.min_hours, &mut p);
        if s.max_hours < s.min_hours {
            p.push("`sleep.max_hours` must be at least `sleep.min_hours`".to_owned());
        }
        if !(s.nap_min_minutes >= 1.0 && s.nap_min_minutes <= s.nap_max_minutes) {
            p.push("naps must satisfy 1 <= nap_min_minutes <= nap_max_minutes".to_owned());
        }
        unit("sleep.day_factor", s.day_factor, &mut p);
        if !(1.0..=12.0).contains(&s.bedtime_after_sunset_hours) {
            p.push("`sleep.bedtime_after_sunset_hours` must be between 1 and 12".to_owned());
        }
        positive("social.tau_h", self.social.tau_h, &mut p);
        unit(
            "social.quality_per_companion",
            self.social.quality_per_companion,
            &mut p,
        );
        unit(
            "social.household_quality",
            self.social.household_quality,
            &mut p,
        );
        let h = &self.household;
        positive(
            "household.water_l_per_person_day",
            h.water_l_per_person_day,
            &mut p,
        );
        positive("household.carry_water_l", h.carry_water_l, &mut p);
        positive("household.water_target_days", h.water_target_days, &mut p);
        positive("household.food_target_days", h.food_target_days, &mut p);
        positive("household.carry_kg", h.carry_kg, &mut p);
        for v in h.fuel_kg_per_person_day {
            non_negative("household.fuel_kg_per_person_day", v, &mut p);
        }
        positive("household.fuel_target_days", h.fuel_target_days, &mut p);
        non_negative("household.short_food_days", h.short_food_days, &mut p);
        if !(h.recovered_food_days.is_finite() && h.recovered_food_days > h.short_food_days) {
            p.push(
                "`household.recovered_food_days` must be more than `household.short_food_days`"
                    .to_owned(),
            );
        }
        positive(
            "household.daily_kcal_per_person",
            h.daily_kcal_per_person,
            &mut p,
        );
        let d = &self.decision;
        positive(
            "decision.temperature_sd_fraction",
            d.temperature_sd_fraction,
            &mut p,
        );
        positive("decision.min_temperature", d.min_temperature, &mut p);
        positive(
            "decision.trip_half_worth_days",
            d.trip_half_worth_days,
            &mut p,
        );
        for (name, v) in [
            ("w_hunger", d.w_hunger),
            ("w_sleep", d.w_sleep),
            ("w_social", d.w_social),
            ("w_food", d.w_food),
            ("w_work", d.w_work),
            ("w_fuel", d.w_fuel),
            ("w_water", d.w_water),
            ("w_walk_hour", d.w_walk_hour),
            ("w_effort", d.w_effort),
            ("w_dark", d.w_dark),
            ("w_rest", d.w_rest),
            ("w_play", d.w_play),
        ] {
            non_negative(&format!("decision.{name}"), v, &mut p);
        }
        let b = &self.band;
        if !(b.min_size >= 2 && b.min_size <= b.default_size && b.default_size <= b.max_size) {
            p.push("band sizes must satisfy 2 <= min_size <= default_size <= max_size".to_owned());
        }
        if b.min_families == 0 {
            p.push("`band.min_families` must be at least 1".to_owned());
        }
        if b.min_size < 2 * b.min_families {
            p.push(
                "`band.min_size` must be at least twice `band.min_families` (a couple per family)"
                    .to_owned(),
            );
        }
        if b.camp_candidates == 0 {
            p.push("`band.camp_candidates` must be at least 1".to_owned());
        }
        positive("band.site_radius_m", b.site_radius_m, &mut p);
        non_negative("band.provisions_days", b.provisions_days, &mut p);
        unit("band.elder_chance", b.elder_chance, &mut p);
        unit("band.young_adult_chance", b.young_adult_chance, &mut p);
        positive("band.birth_spacing_months", b.birth_spacing_months, &mut p);
        positive("band.site_max_slope", b.site_max_slope, &mut p);
        for (name, v) in [
            ("site_w_food", b.site_w_food),
            ("site_w_water_per_100m", b.site_w_water_per_100m),
            ("site_w_slope_per_pct", b.site_w_slope_per_pct),
            ("site_w_flood", b.site_w_flood),
            ("site_flood_hand_m", b.site_flood_hand_m),
        ] {
            non_negative(&format!("band.{name}"), v, &mut p);
        }
        let m = &self.mortality;
        for (name, v) in [("a", m.a), ("b", m.b), ("c", m.c), ("d", m.d), ("e", m.e)] {
            non_negative(&format!("mortality.{name}"), v, &mut p);
        }
        if m.b <= 0.0 || m.e <= 0.0 {
            p.push("`mortality.b` and `mortality.e` must be positive".to_owned());
        }
        p
    }
}
