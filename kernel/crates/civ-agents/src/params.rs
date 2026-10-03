//! Authored parameters of people and their activities. Filled by `civ-content`; nothing here has
//! a default, so every number is visible in a content file with its source.

use civ_world::nav::NavParams;

/// What an authored activity does: the verb the engine knows how to carry out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Behavior {
    /// Sleep at home.
    Sleep,
    /// Eat a meal from the household's store, at home.
    Eat,
    /// Walk to water, fill containers, carry them home.
    FetchWater,
    /// Walk to a patch, gather a wild resource, carry it home.
    Gather,
    /// Sit with others at the hearth.
    Socialize,
    /// Rest at home.
    Rest,
    /// Children's play near home.
    Play,
}

impl Behavior {
    /// Every behavior, in a fixed order (part of the boundary: never reorder).
    pub const ALL: [Behavior; 7] = [
        Behavior::Sleep,
        Behavior::Eat,
        Behavior::FetchWater,
        Behavior::Gather,
        Behavior::Socialize,
        Behavior::Rest,
        Behavior::Play,
    ];

    /// The authored name of a behavior.
    pub fn name(self) -> &'static str {
        match self {
            Behavior::Sleep => "sleep",
            Behavior::Eat => "eat",
            Behavior::FetchWater => "fetch_water",
            Behavior::Gather => "gather",
            Behavior::Socialize => "socialize",
            Behavior::Rest => "rest",
            Behavior::Play => "play",
        }
    }

    /// The behavior with an authored name.
    pub fn from_name(name: &str) -> Option<Behavior> {
        Behavior::ALL.into_iter().find(|b| b.name() == name)
    }
}

/// An authored activity: a behavior with its numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityDef {
    /// Content id, for example `core:activity/gather_plants`.
    pub id: String,
    /// Display name, for example "Gather plants".
    pub name: String,
    /// What the inspector says someone is doing, for example "gathering plants".
    pub doing: String,
    /// The behavior it uses.
    pub behavior: Behavior,
    /// For gathering: the land resource, by index in the land parameters.
    pub resource: Option<usize>,
    /// Physical activity ratio of the work (energy use as a multiple of basal metabolism).
    pub par: f64,
    /// Youngest age that does it, years.
    pub min_age_years: f64,
    /// Oldest age that does it, years.
    pub max_age_years: f64,
    /// Shortest work time, minutes.
    pub min_minutes: u32,
    /// Longest work time, minutes.
    pub max_minutes: u32,
    /// Only done in daylight.
    pub daylight_only: bool,
    /// Longest one-way walk people will make for it, minutes.
    pub max_walk_minutes: u32,
}

/// What a good is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GoodUse {
    /// Eaten.
    Food,
    /// Burned for cooking and warmth.
    Fuel,
}

impl GoodUse {
    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            GoodUse::Food => "food",
            GoodUse::Fuel => "fuel",
        }
    }

    /// The use with an authored name.
    pub fn from_name(name: &str) -> Option<GoodUse> {
        [GoodUse::Food, GoodUse::Fuel]
            .into_iter()
            .find(|u| u.name() == name)
    }
}

/// An authored good: something people carry home and keep.
#[derive(Clone, Debug, PartialEq)]
pub struct GoodDef {
    /// Content id, for example `core:good/meat`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What it is for.
    pub purpose: GoodUse,
    /// Food energy, kcal per kilogram (0 for goods that are not eaten).
    pub kcal_per_kg: f64,
    /// Days for half of a stored amount to spoil; 0 means it keeps.
    pub half_life_days: f64,
    /// It must be cooked over a fire before it is eaten.
    pub cooked: bool,
    /// When brought home it is shared among every household of the settlement.
    pub shared: bool,
}

/// The authored activities and goods.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    /// Activities, in content id order. Their index is how people and saves refer to them.
    pub activities: Vec<ActivityDef>,
    /// Goods, in content id order. Their index is how stores refer to them.
    pub goods: Vec<GoodDef>,
}

impl Catalog {
    /// The activity with this content id.
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.activities.iter().position(|a| a.id == id)
    }

    /// The good with this content id.
    pub fn good_index(&self, id: &str) -> Option<usize> {
        self.goods.iter().position(|g| g.id == id)
    }
}

/// Basal metabolism and body size (research 05-02 §2.1–2.2; FAO/WHO/UNU Schofield equations).
#[derive(Clone, Debug, PartialEq)]
pub struct EnergyParams {
    /// Basal metabolic rate per age band, `(slope kcal/kg/day, intercept kcal/day)`, for males.
    /// Bands start at [`EnergyParams::bmr_band_starts`].
    pub bmr_male: Vec<(f64, f64)>,
    /// The same for females.
    pub bmr_female: Vec<(f64, f64)>,
    /// First age of each BMR band, years.
    pub bmr_band_starts: Vec<f64>,
    /// Body mass by age: `(age years, male kg, female kg)`, ascending.
    pub mass_by_age: Vec<(f64, f64, f64)>,
    /// Physical activity ratio while walking.
    pub walk_par: f64,
    /// Physical activity ratio between activities (standing about).
    pub idle_par: f64,
    /// Hours a meal keeps someone full.
    pub satiety_hours: f64,
    /// Hours from the end of satiety to full hunger.
    pub hunger_ramp_hours: f64,
    /// Energy deficit that adds one unit of hunger, kcal.
    pub deficit_unit_kcal: f64,
    /// Largest energy surplus the body banks, kcal.
    pub max_surplus_kcal: f64,
    /// Energy the body can draw on in a shortage, kcal per kilogram of body mass: the floor of
    /// the energy balance.
    pub reserve_kcal_per_kg: f64,
    /// Minutes a meal takes.
    pub meal_minutes: u32,
}

/// Sleep pressure, two-process model (research 04-02 §2.2, 01-09).
#[derive(Clone, Debug, PartialEq)]
pub struct SleepParams {
    /// Time constant of pressure building while awake, hours.
    pub tau_awake_h: f64,
    /// Time constant of pressure falling while asleep, hours.
    pub tau_asleep_h: f64,
    /// Pressure at which a sleeper wakes.
    pub wake_pressure: f64,
    /// Shortest night's sleep, hours.
    pub min_hours: f64,
    /// Longest night's sleep, hours.
    pub max_hours: f64,
    /// Shortest daytime nap, minutes.
    pub nap_min_minutes: f64,
    /// Longest daytime nap, minutes.
    pub nap_max_minutes: f64,
    /// Weight of sleep pressure in daylight, relative to night (circadian factor).
    pub day_factor: f64,
    /// Usual bedtime, hours after sunset. The weight keeps its daylight value until an hour
    /// before, then rises to its night value: alertness holds through the evening and the sleep
    /// gate opens near habitual bedtime (research 04-02: sleep onset about 3.3 h after sunset).
    pub bedtime_after_sunset_hours: f64,
}

/// Relatedness: closeness to others, eased toward the quality of present company
/// (research 04-01 §2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct SocialParams {
    /// Time constant, hours.
    pub tau_h: f64,
    /// Quality gained per companion present when sitting together.
    pub quality_per_companion: f64,
    /// Quality of time spent among one's household without sitting together.
    pub household_quality: f64,
}

/// What a household keeps and carries.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseholdParams {
    /// Water used per person per day, litres (research 10-01 §2.1).
    pub water_l_per_person_day: f64,
    /// Litres one person carries per trip.
    pub carry_water_l: f64,
    /// Days of water a household tries to keep.
    pub water_target_days: f64,
    /// Days of food a household tries to keep.
    pub food_target_days: f64,
    /// What one person carries home, kilograms.
    pub carry_kg: f64,
    /// Firewood a household burns per member per day, by month, January first, kilograms.
    pub fuel_kg_per_person_day: [f64; 12],
    /// Days of firewood a household tries to keep.
    pub fuel_target_days: f64,
    /// Days of food in a settlement's stores below which the chronicle notes a shortage.
    pub short_food_days: f64,
    /// Days of food above which the chronicle notes that a shortage is over.
    pub recovered_food_days: f64,
    /// Food energy a person needs per day on average, kcal (for days-of-supply arithmetic).
    pub daily_kcal_per_person: f64,
}

/// How choices are scored and sampled (research 01-09 §4.3, 04-07 §2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct DecisionParams {
    /// Softmax temperature as a fraction of the spread (standard deviation) of the candidates'
    /// utilities.
    pub temperature_sd_fraction: f64,
    /// Lowest temperature, utility points.
    pub min_temperature: f64,
    /// Points per unit of hunger.
    pub w_hunger: f64,
    /// Points per unit of sleep drive.
    pub w_sleep: f64,
    /// Points per unit of loneliness in the evening.
    pub w_social: f64,
    /// Points per unit of food shortage times the share of a day's food a trip brings.
    pub w_food: f64,
    /// Points for useful work regardless of shortage (purpose).
    pub w_work: f64,
    /// Points per unit of firewood shortage times the worth of a trip.
    pub w_fuel: f64,
    /// Days of household food a gathering trip must bring to be worth half as much as a very
    /// large haul.
    pub trip_half_worth_days: f64,
    /// Points per unit of water shortage.
    pub w_water: f64,
    /// Points lost per hour of walking.
    pub w_walk_hour: f64,
    /// Points lost per hour of hard work when tired, per unit of PAR above resting.
    pub w_effort: f64,
    /// Points lost by outdoor work that would run into darkness.
    pub w_dark: f64,
    /// Points for resting.
    pub w_rest: f64,
    /// Points for children's play.
    pub w_play: f64,
}

/// The first people of a new world (research 05-01 §4.5).
#[derive(Clone, Debug, PartialEq)]
pub struct BandParams {
    /// People in a founding band unless the player chooses otherwise.
    pub default_size: u32,
    /// Smallest band the new-world dialog allows.
    pub min_size: u32,
    /// Largest band the new-world dialog allows.
    pub max_size: u32,
    /// Fewest unrelated families in a band.
    pub min_families: u32,
    /// Candidate camp sites scored when a band arrives.
    pub camp_candidates: u32,
    /// Distance within which people judge a camp site's surroundings, metres.
    pub site_radius_m: f64,
    /// Days of food each household carries in.
    pub provisions_days: f64,
    /// The good they carry it as, by index in the catalog's goods.
    pub provisions_good: usize,
    /// Chance that a family brings an elder.
    pub elder_chance: f64,
    /// Chance that a family brings an unmarried young adult.
    pub young_adult_chance: f64,
    /// Months between births when building founding families.
    pub birth_spacing_months: f64,
    /// Steepest ground a camp is made on, rise over run.
    pub site_max_slope: f64,
    /// Points per natural-log unit of the wild food around a site, measured in years of the
    /// band's needs.
    pub site_w_food: f64,
    /// Points lost per 100 m from fresh water.
    pub site_w_water_per_100m: f64,
    /// Points lost per percent of slope.
    pub site_w_slope_per_pct: f64,
    /// Points lost by a site lower than [`BandParams::site_flood_hand_m`] above the nearest
    /// stream.
    pub site_w_flood: f64,
    /// Height above the nearest stream below which a site floods, metres.
    pub site_flood_hand_m: f64,
}

/// Siler mortality hazard per year at age `x` years: `A·e^(−Bx) + C + D·e^(Ex)`
/// (research 05-01 §1.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Siler {
    /// Infant hazard scale.
    pub a: f64,
    /// Infant hazard decay.
    pub b: f64,
    /// Constant adult hazard.
    pub c: f64,
    /// Senescent hazard scale.
    pub d: f64,
    /// Senescent hazard growth.
    pub e: f64,
}

impl Siler {
    /// Hazard per year at `age` years.
    pub fn hazard(&self, age: f64) -> f64 {
        self.a * (-self.b * age).exp() + self.c + self.d * (self.e * age).exp()
    }

    /// Probability of surviving from birth to `age` years.
    pub fn survival(&self, age: f64) -> f64 {
        // ∫ h = A/B (1 − e^(−Bx)) + C x + D/E (e^(Ex) − 1)
        let cum = self.a / self.b * (1.0 - (-self.b * age).exp())
            + self.c * age
            + self.d / self.e * ((self.e * age).exp() - 1.0);
        (-cum).exp()
    }
}

/// Names (research 06-07): given names by sex, and parts for place names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NameParams {
    /// Given names for males.
    pub male: Vec<String>,
    /// Given names for females.
    pub female: Vec<String>,
    /// First parts of place names.
    pub place_first: Vec<String>,
    /// Second parts of place names.
    pub place_second: Vec<String>,
}

/// Everything authored about people.
#[derive(Clone, Debug, PartialEq)]
pub struct PeopleParams {
    /// Walking (research 01-08).
    pub nav: NavParams,
    /// Walking speed multiplier by age: `(age years, factor)`, ascending.
    pub walk_speed_by_age: Vec<(f64, f64)>,
    /// Work efficiency by age: `(age years, factor)`, ascending.
    pub capacity_by_age: Vec<(f64, f64)>,
    /// Latitude, degrees north, for day length.
    pub latitude_deg: f64,
    /// Energy and body.
    pub energy: EnergyParams,
    /// Sleep.
    pub sleep: SleepParams,
    /// Company.
    pub social: SocialParams,
    /// Household stores.
    pub household: HouseholdParams,
    /// Choice.
    pub decision: DecisionParams,
    /// Founding bands.
    pub band: BandParams,
    /// Mortality (used for founding ages in M1's first slice; hazards follow).
    pub mortality: Siler,
    /// Names.
    pub names: NameParams,
}

/// Linear interpolation in an ascending `(x, y)` table, clamped at its ends.
pub fn interpolate(table: &[(f64, f64)], x: f64) -> f64 {
    match table {
        [] => 0.0,
        [only] => only.1,
        _ => {
            if x <= table[0].0 {
                return table[0].1;
            }
            for w in table.windows(2) {
                let ((x0, y0), (x1, y1)) = (w[0], w[1]);
                if x <= x1 {
                    let t = if x1 > x0 { (x - x0) / (x1 - x0) } else { 1.0 };
                    return y0 + t * (y1 - y0);
                }
            }
            table[table.len() - 1].1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hadza_siler_gives_the_published_life_table() {
        // Research 05-01 §1.2: A .351, B .895, C .011, D 6.70e-6, E .125 gives 1q0 ≈ 216 ‰.
        let s = Siler {
            a: 0.351,
            b: 0.895,
            c: 0.011,
            d: 6.70e-6,
            e: 0.125,
        };
        let q0 = 1.0 - s.survival(1.0);
        assert!((q0 - 0.216).abs() < 0.03, "1q0 = {q0}");
        let q5 = 1.0 - s.survival(5.0);
        assert!((q5 - 0.358).abs() < 0.04, "5q0 = {q5}");
        assert!(s.survival(80.0) < s.survival(40.0));
    }

    #[test]
    fn interpolation_clamps_and_blends() {
        let t = [(0.0, 1.0), (10.0, 3.0)];
        assert_eq!(interpolate(&t, -5.0), 1.0);
        assert_eq!(interpolate(&t, 5.0), 2.0);
        assert_eq!(interpolate(&t, 50.0), 3.0);
        assert_eq!(interpolate(&[], 1.0), 0.0);
    }

    #[test]
    fn behaviors_round_trip_by_name() {
        for b in Behavior::ALL {
            assert_eq!(Behavior::from_name(b.name()), Some(b));
        }
        assert_eq!(Behavior::from_name("fly"), None);
    }
}
