//! People profiles (`kind = "people"`): how a world's people live, and their founding bands.
//! Every field is required: an omitted number is an error, never a silent engine default.

use civ_agents::params::{
    BandParams, BuildParams, DecisionParams, EnergyParams, FamilyParams, FarmParams,
    FertilityParams, FirmParams, HouseholdParams, KnowledgeParams, MarketParams, MortalityParams,
    NameParams, PeopleParams, Residence, Siler, SleepParams, SocialParams,
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
    pub farm: Farm,
    pub build: Build,
    pub mortality: Mortality,
    pub fertility: Fertility,
    pub family: Family,
    pub market: Market,
    pub firm: Firm,
    pub knowledge: Knowledge,
    pub digging: DiggingFile,
    pub style: StyleFile,
    /// The midden and carrying it to the fields (M3c slice V; content API 29).
    pub midden: MiddenFile,
    /// Ties between people (M4a slice Y; content API 30).
    pub ties: TiesFile,
    /// Standing and notables (M4a slice Y; content API 30).
    pub standing: StandingFile,
    /// The polity: its gathering, forecasts and compliance (M4a slice Z; content API 31).
    pub polity: PolityFile,
    /// Taking, what is seen of it and what is owed for it (M4b slice AA; content API 35).
    pub crime: CrimeFile,
    /// How word travels and grievances are held (M4c slice AE; content API 39).
    pub word: WordFile,
    /// How opinion moves (M4c slice AG; content API 41).
    pub opinion: OpinionFile,
    /// How factions are founded, joined and kept (M4c slice AH; content API 45).
    pub faction: FactionFile,
    /// How households come to know other places (M5a slice AM; content API 52).
    pub places: PlacesFile,
    pub moving: MovingFile,
    /// What founding a settlement is worth, and what a coalition must hold (M5a slice AO; content
    /// API 55).
    pub founding: FoundingFile,
    /// How price reports of other markets are held and passed on (M5b slice AP; content API 56).
    pub reports: ReportsFile,
    /// How views of other polities are held (M5c slice AT; content API 60).
    pub relations: RelationsFile,
    /// How people tally a source of water (M6a slice BA, step two; content API 76).
    pub suspicion: SuspicionFile,
    /// How buildings burn and fire passes between them (M6b slice BC; content API 81).
    pub fire: FireFile,
    /// How people lay out and build flood banks, and remember what floods cost them (M6b slice
    /// BG; content API 88).
    pub levees: LeveesFile,
    /// How a household lays out and digs a ditch to its field (content API 92).
    pub ditches: DitchesFile,
    /// How people weigh a rule for building (M6b slice BH; content API 93).
    pub rules: RulesFile,
    /// Who calls a band, who joins it and how encounters run (M6c slice BJ; content API 96).
    pub bands: BandsFile,
    /// How people lay out and weigh an enclosure (M6c slice BK; content API 100).
    pub enclosures: EnclosuresFile,
    /// How a council's elections run (M7a slice BO; content API 106).
    pub elections: ElectionsFile,
    /// How households borrow and lend food (M7d slice CA; content API 119).
    pub credit: CreditFile,
    /// How households let spare floor and seek a room (ADR-0040; content API 212).
    pub letting: LettingFile,
    /// How long people take to climb to the storeys they live on (ADR-0042 §1; content API 214).
    pub climbing: ClimbingFile,
}

/// How long people take to climb a storey by a stair (ADR-0042 §1; content API 214). See
/// [`civ_agents::climb::ClimbParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClimbingFile {
    pub up_s_per_storey: f64,
    pub down_s_per_storey: f64,
}

impl ClimbingFile {
    fn params(&self) -> civ_agents::climb::ClimbParams {
        civ_agents::climb::ClimbParams {
            up_s_per_storey: self.up_s_per_storey,
            down_s_per_storey: self.down_s_per_storey,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v) in [
            ("climbing.up_s_per_storey", self.up_s_per_storey),
            ("climbing.down_s_per_storey", self.down_s_per_storey),
        ] {
            if !(v.is_finite() && (0.0..=600.0).contains(&v)) {
                p.push(format!("`{name}` must be between 0 and 600 (got {v})"));
            }
        }
    }
}

/// How households let spare floor and seek a room (ADR-0040 §2; content API 212). See
/// [`civ_agents::letting::LettingParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LettingFile {
    pub asking_rent_food_days_per_person_month: f64,
    pub step: f64,
    pub candidates: u32,
}

impl LettingFile {
    fn params(&self) -> civ_agents::letting::LettingParams {
        civ_agents::letting::LettingParams {
            asking_rent_food_days_per_person_month: self.asking_rent_food_days_per_person_month,
            step: self.step,
            candidates: self.candidates,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            (
                "letting.asking_rent_food_days_per_person_month",
                self.asking_rent_food_days_per_person_month,
                0.0,
                365.0,
            ),
            ("letting.step", self.step, 0.0, 365.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if !(1..=64).contains(&self.candidates) {
            p.push(format!(
                "`letting.candidates` must be between 1 and 64 (got {})",
                self.candidates
            ));
        }
    }
}

/// How households borrow and lend food (M7d slice CA, ADR-0027 §2; content API 119). See
/// [`civ_agents::credit::CreditParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreditFile {
    pub review_days: u32,
    pub least_days: f64,
    pub customary_increment: f64,
    pub max_increment: f64,
    pub p_prior: f64,
    pub p_max: f64,
    pub keep_return: f64,
    pub lend_share: f64,
    pub max_asked: u32,
    pub grace_days: u32,
    // Pledges and answers at the due day (M7d slice CB; content API 120).
    pub field_recovery: f64,
    pub guarantor_regard: f64,
    pub keep_days: f64,
    pub extension_days: u32,
    pub max_extensions: u32,
    pub extend_regard: f64,
    pub pay_base: f64,
    pub w_pay_regard: f64,
    pub w_pay_cost: f64,
    // Wage arrears (M7d slice CB; content API 121).
    pub wage_days: u32,
}

impl CreditFile {
    fn params(&self) -> civ_agents::credit::CreditParams {
        civ_agents::credit::CreditParams {
            review_days: self.review_days,
            least_days: self.least_days,
            customary_increment: self.customary_increment,
            max_increment: self.max_increment,
            p_prior: self.p_prior,
            p_max: self.p_max,
            keep_return: self.keep_return,
            lend_share: self.lend_share,
            max_asked: self.max_asked,
            grace_days: self.grace_days,
            field_recovery: self.field_recovery,
            guarantor_regard: self.guarantor_regard,
            keep_days: self.keep_days,
            extension_days: self.extension_days,
            max_extensions: self.max_extensions,
            extend_regard: self.extend_regard,
            pay_base: self.pay_base,
            w_pay_regard: self.w_pay_regard,
            w_pay_cost: self.w_pay_cost,
            wage_days: self.wage_days,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("credit.review_days", self.review_days, 1, 365),
            ("credit.max_asked", self.max_asked, 1, 20),
            ("credit.grace_days", self.grace_days, 0, 365),
            ("credit.extension_days", self.extension_days, 1, 365),
            ("credit.max_extensions", self.max_extensions, 0, 10),
            ("credit.wage_days", self.wage_days, 0, 365),
        ] {
            if !(lo..=hi).contains(&v) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        for (name, v, lo, hi) in [
            ("credit.least_days", self.least_days, 0.0, 365.0),
            (
                "credit.customary_increment",
                self.customary_increment,
                0.0,
                10.0,
            ),
            ("credit.max_increment", self.max_increment, 0.0, 10.0),
            ("credit.p_prior", self.p_prior, 0.0, 1.0),
            ("credit.p_max", self.p_max, 0.0, 1.0),
            ("credit.keep_return", self.keep_return, -1.0, 10.0),
            ("credit.lend_share", self.lend_share, 0.0, 1.0),
            ("credit.field_recovery", self.field_recovery, 0.0, 1.0),
            ("credit.guarantor_regard", self.guarantor_regard, -1.0, 1.0),
            ("credit.keep_days", self.keep_days, 0.0, 365.0),
            ("credit.extend_regard", self.extend_regard, -1.0, 1.0),
            ("credit.pay_base", self.pay_base, -20.0, 20.0),
            ("credit.w_pay_regard", self.w_pay_regard, 0.0, 20.0),
            ("credit.w_pay_cost", self.w_pay_cost, 0.0, 20.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
    }
}

/// How a council's elections run (M7a slice BO; content API 106). See
/// [`civ_agents::seats::ElectionParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ElectionsFile {
    pub stand_days: u32,
    pub poll_minutes: u32,
    pub w_sit: f64,
    pub w_standing: f64,
    pub sit_cost: f64,
    pub stand_temperature: f64,
    pub w_regard: f64,
    pub w_household: f64,
    pub w_counsel: f64,
    pub w_self: f64,
    pub vote_scale: f64,
    pub poll_base: f64,
    pub w_poll_stake: f64,
    pub positions_told: u8,
    pub w_distance: f64,
    pub w_record: f64,
    pub w_slate: f64,
    pub w_campaign: f64,
    pub w_evening: f64,
}

impl ElectionsFile {
    fn params(&self) -> civ_agents::seats::ElectionParams {
        civ_agents::seats::ElectionParams {
            stand_days: self.stand_days,
            poll_minutes: self.poll_minutes,
            w_sit: self.w_sit,
            w_standing: self.w_standing,
            sit_cost: self.sit_cost,
            stand_temperature: self.stand_temperature,
            w_regard: self.w_regard,
            w_household: self.w_household,
            w_counsel: self.w_counsel,
            w_self: self.w_self,
            vote_scale: self.vote_scale,
            poll_base: self.poll_base,
            w_poll_stake: self.w_poll_stake,
            positions_told: self.positions_told,
            w_distance: self.w_distance,
            w_record: self.w_record,
            w_slate: self.w_slate,
            w_campaign: self.w_campaign,
            w_evening: self.w_evening,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        if !(1..=60).contains(&self.stand_days) {
            p.push(format!(
                "`elections.stand_days` must be 1 to 60 (got {})",
                self.stand_days
            ));
        }
        // ADR-0024 §4: up to three questions.
        if self.positions_told > 3 {
            p.push(format!(
                "`elections.positions_told` must be 0 to 3 (got {})",
                self.positions_told
            ));
        }
        if !(1..=240).contains(&self.poll_minutes) {
            p.push(format!(
                "`elections.poll_minutes` must be 1 to 240 (got {})",
                self.poll_minutes
            ));
        }
        for (name, v, lo, hi) in [
            ("elections.w_sit", self.w_sit, 0.0, 100.0),
            ("elections.w_standing", self.w_standing, 0.0, 100.0),
            ("elections.sit_cost", self.sit_cost, 0.0, 100.0),
            (
                "elections.stand_temperature",
                self.stand_temperature,
                0.01,
                100.0,
            ),
            ("elections.w_regard", self.w_regard, 0.0, 100.0),
            ("elections.w_household", self.w_household, 0.0, 100.0),
            ("elections.w_counsel", self.w_counsel, 0.0, 100.0),
            ("elections.w_self", self.w_self, 0.0, 100.0),
            ("elections.vote_scale", self.vote_scale, 0.0, 100.0),
            ("elections.poll_base", self.poll_base, 0.0, 100.0),
            ("elections.w_poll_stake", self.w_poll_stake, 0.0, 100.0),
            ("elections.w_distance", self.w_distance, 0.0, 100.0),
            ("elections.w_record", self.w_record, 0.0, 100.0),
            ("elections.w_slate", self.w_slate, 0.0, 100.0),
            ("elections.w_campaign", self.w_campaign, 0.0, 100.0),
            ("elections.w_evening", self.w_evening, 0.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
    }
}

/// How people lay out and weigh an enclosure (M6c slice BK; content API 100). See
/// [`civ_agents::enclose::EnclosureParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EnclosuresFile {
    pub sector_m: f64,
    pub gates: u32,
    pub palisade_h_per_m: f64,
    pub reach_m: f64,
    pub margin_m: f64,
    pub min_radius_m: f64,
    pub life_years: f64,
    pub cover: f64,
    pub front_m: f64,
    pub crossings_a_day: f64,
    pub rot_per_year: f64,
    pub mend_loss: f64,
    pub alarm_days: u32,
}

impl EnclosuresFile {
    fn params(&self) -> civ_agents::enclose::EnclosureParams {
        civ_agents::enclose::EnclosureParams {
            sector_m: self.sector_m,
            gates: self.gates,
            palisade_h_per_m: self.palisade_h_per_m,
            reach_m: self.reach_m,
            margin_m: self.margin_m,
            min_radius_m: self.min_radius_m,
            life_years: self.life_years,
            cover: self.cover,
            front_m: self.front_m,
            crossings_a_day: self.crossings_a_day,
            rot_per_year: self.rot_per_year,
            mend_loss: self.mend_loss,
            alarm_days: self.alarm_days,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("enclosures.sector_m", self.sector_m, 5.0, 50.0),
            (
                "enclosures.palisade_h_per_m",
                self.palisade_h_per_m,
                0.1,
                200.0,
            ),
            ("enclosures.reach_m", self.reach_m, 1.0, 2000.0),
            ("enclosures.margin_m", self.margin_m, 0.0, 100.0),
            ("enclosures.min_radius_m", self.min_radius_m, 3.0, 2000.0),
            ("enclosures.life_years", self.life_years, 1.0, 500.0),
            ("enclosures.cover", self.cover, 0.0, 1.0),
            ("enclosures.front_m", self.front_m, 0.5, 100.0),
            (
                "enclosures.crossings_a_day",
                self.crossings_a_day,
                0.0,
                100.0,
            ),
            ("enclosures.rot_per_year", self.rot_per_year, 0.0, 1.0),
            ("enclosures.mend_loss", self.mend_loss, 0.05, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.alarm_days > 3650 {
            p.push(format!(
                "`enclosures.alarm_days` must be at most 3650 (got {})",
                self.alarm_days
            ));
        }
        if !(1..=12).contains(&self.gates) {
            p.push(format!(
                "`enclosures.gates` must be between 1 and 12 (got {})",
                self.gates
            ));
        }
    }
}

/// Who calls a band, who joins it and how it runs (M6c slice BJ; content API 96). See
/// [`civ_agents::war::BandParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BandsFile {
    pub review_days: u32,
    pub min_age: f64,
    pub max_age: f64,
    pub max_walk_minutes: f64,
    pub food_prior: f64,
    pub w_spoils: f64,
    pub w_grievance: f64,
    pub w_danger: f64,
    pub w_objection: f64,
    pub w_regard: f64,
    pub w_follow: f64,
    pub call_threshold: [f64; 2],
    pub join_threshold: [f64; 2],
    pub join_regard: f64,
    pub told_most: u32,
    pub least: u32,
    pub muster_hour: u32,
    pub w_promise: f64,
    pub stay_minutes: u32,
    /// Content API 97 (M6c slice BJ, step two).
    pub look_minutes: u32,
    pub shirk_promised: f64,
    pub shirk_asked: f64,
    /// Content API 98 (M6c slice BJ, step three).
    pub w_law: f64,
    pub service_days: f64,
    pub retaliation_share: f64,
    pub will_band: f64,
    pub will_home: f64,
    pub will_neighbour: f64,
    /// Content API 102 (M6c slice BL).
    pub camp_minutes: u32,
    pub w_camp_day: f64,
    pub w_camp_tired: f64,
    /// Content API 104 (M6c slice BM).
    pub hold_days: u32,
    pub sleep_out_at: f64,
    pub sleep_out_minutes: u32,
    pub fight: FightFile,
}

/// How an encounter runs (M6c slice BJ; content API 96). See [`civ_agents::war::FightParams`].
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FightFile {
    pub max_minutes: u16,
    pub frontage_home: u32,
    pub land_hands: f64,
    pub land_axe: f64,
    pub land_spear: f64,
    pub hurt_days: [u16; 2],
    pub kill_share: f64,
    pub will_seen: f64,
    pub seen_minutes: u16,
    pub will_gave: f64,
    pub will_hurt: f64,
    pub will_leader: f64,
}

impl BandsFile {
    fn params(&self) -> civ_agents::war::BandParams {
        let f = &self.fight;
        civ_agents::war::BandParams {
            review_days: self.review_days,
            min_age: self.min_age,
            max_age: self.max_age,
            max_walk_minutes: self.max_walk_minutes,
            food_prior: self.food_prior,
            w_spoils: self.w_spoils,
            w_grievance: self.w_grievance,
            w_danger: self.w_danger,
            w_objection: self.w_objection,
            w_regard: self.w_regard,
            w_follow: self.w_follow,
            call_threshold: self.call_threshold,
            join_threshold: self.join_threshold,
            join_regard: self.join_regard,
            told_most: self.told_most,
            least: self.least,
            muster_hour: self.muster_hour,
            w_promise: self.w_promise,
            stay_minutes: self.stay_minutes,
            look_minutes: self.look_minutes,
            shirk_promised: self.shirk_promised,
            shirk_asked: self.shirk_asked,
            w_law: self.w_law,
            service_days: self.service_days,
            retaliation_share: self.retaliation_share,
            will_band: self.will_band,
            will_home: self.will_home,
            will_neighbour: self.will_neighbour,
            camp_minutes: self.camp_minutes,
            w_camp_day: self.w_camp_day,
            w_camp_tired: self.w_camp_tired,
            hold_days: self.hold_days,
            sleep_out_at: self.sleep_out_at,
            sleep_out_minutes: self.sleep_out_minutes,
            fight: civ_agents::war::FightParams {
                max_minutes: f.max_minutes,
                frontage_home: f.frontage_home,
                land_hands: f.land_hands,
                land_axe: f.land_axe,
                land_spear: f.land_spear,
                hurt_days: f.hurt_days,
                kill_share: f.kill_share,
                will_seen: f.will_seen,
                seen_minutes: f.seen_minutes,
                will_gave: f.will_gave,
                will_hurt: f.will_hurt,
                will_leader: f.will_leader,
            },
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        let f = &self.fight;
        for (name, v, lo, hi) in [
            ("bands.min_age", self.min_age, 0.0, 130.0),
            ("bands.max_age", self.max_age, 0.0, 130.0),
            ("bands.max_walk_minutes", self.max_walk_minutes, 1.0, 1440.0),
            ("bands.food_prior", self.food_prior, 0.0, 1.0),
            ("bands.w_spoils", self.w_spoils, 0.0, 100.0),
            ("bands.w_grievance", self.w_grievance, 0.0, 100.0),
            ("bands.w_danger", self.w_danger, 0.0, 100.0),
            ("bands.w_objection", self.w_objection, 0.0, 100.0),
            ("bands.w_regard", self.w_regard, 0.0, 100.0),
            ("bands.w_follow", self.w_follow, 0.0, 100.0),
            ("bands.join_regard", self.join_regard, 0.0, 1.0),
            ("bands.w_promise", self.w_promise, 0.0, 1000.0),
            ("bands.shirk_promised", self.shirk_promised, 0.0, 100.0),
            ("bands.shirk_asked", self.shirk_asked, 0.0, 100.0),
            ("bands.w_law", self.w_law, 0.0, 100.0),
            ("bands.service_days", self.service_days, 0.0, 365.0),
            ("bands.retaliation_share", self.retaliation_share, 0.0, 1.0),
            ("bands.will_band", self.will_band, 0.0, 100.0),
            ("bands.will_home", self.will_home, 0.0, 100.0),
            ("bands.will_neighbour", self.will_neighbour, 0.0, 100.0),
            ("bands.fight.land_hands", f.land_hands, 0.0, 1.0),
            ("bands.fight.land_axe", f.land_axe, 0.0, 1.0),
            ("bands.fight.land_spear", f.land_spear, 0.0, 1.0),
            ("bands.fight.kill_share", f.kill_share, 0.0, 1.0),
            ("bands.fight.will_seen", f.will_seen, 0.0, 100.0),
            ("bands.fight.will_gave", f.will_gave, 0.0, 100.0),
            ("bands.fight.will_hurt", f.will_hurt, 0.0, 100.0),
            ("bands.fight.will_leader", f.will_leader, 0.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.min_age > self.max_age {
            p.push("`bands.min_age` must not be above `bands.max_age`".to_owned());
        }
        for (name, [lo, hi]) in [
            ("bands.call_threshold", self.call_threshold),
            ("bands.join_threshold", self.join_threshold),
        ] {
            if !(lo.is_finite() && hi.is_finite() && lo <= hi) {
                p.push(format!(
                    "`{name}` must be two finite numbers, low then high"
                ));
            }
        }
        if self.muster_hour > 23 {
            p.push("`bands.muster_hour` must be an hour of the day, 0-23".to_owned());
        }
        if self.least == 0 {
            p.push("`bands.least` must be at least 1".to_owned());
        }
        if self.look_minutes == 0 || self.look_minutes > 1440 {
            p.push("`bands.look_minutes` must be 1-1440".to_owned());
        }
        if self.camp_minutes == 0 || self.camp_minutes > 1440 {
            p.push("`bands.camp_minutes` must be 1-1440".to_owned());
        }
        if !(1..=365).contains(&self.hold_days) {
            p.push("`bands.hold_days` must be 1-365".to_owned());
        }
        if !(self.sleep_out_at.is_finite() && (0.0..=1.0).contains(&self.sleep_out_at)) {
            p.push("`bands.sleep_out_at` must be between 0 and 1".to_owned());
        }
        if self.sleep_out_minutes == 0 || self.sleep_out_minutes > 1440 {
            p.push("`bands.sleep_out_minutes` must be 1-1440".to_owned());
        }
        for (name, v) in [
            ("bands.w_camp_day", self.w_camp_day),
            ("bands.w_camp_tired", self.w_camp_tired),
        ] {
            if !(v.is_finite() && (0.0..=100.0).contains(&v)) {
                p.push(format!("`{name}` must be between 0 and 100 (got {v})"));
            }
        }
        if f.max_minutes == 0 || f.frontage_home == 0 || f.seen_minutes == 0 {
            p.push(
                "`bands.fight` minutes, frontage and seen minutes must be at least 1".to_owned(),
            );
        }
        if f.hurt_days[0] > f.hurt_days[1] {
            p.push("`bands.fight.hurt_days` must be low then high".to_owned());
        }
    }
}

/// How people weigh a rule for building (M6b slice BH; content API 93). See
/// [`civ_agents::rules::RuleParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RulesFile {
    pub memory_years: f64,
    pub avoided_share: f64,
    pub horizon_years: f64,
}

impl RulesFile {
    fn params(&self) -> civ_agents::rules::RuleParams {
        civ_agents::rules::RuleParams {
            memory_years: self.memory_years,
            avoided_share: self.avoided_share,
            horizon_years: self.horizon_years,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("rules.memory_years", self.memory_years, 0.1, 100.0),
            ("rules.avoided_share", self.avoided_share, 0.0, 1.0),
            ("rules.horizon_years", self.horizon_years, 1.0, 500.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
    }
}

/// How a household lays out and digs an irrigation ditch to its field (M6b slice BG; content API
/// 92). See [`civ_agents::irrigation::DitchPlanParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DitchesFile {
    pub bottom_m: f64,
    pub depth_m: f64,
    pub side_run: f64,
    pub h_per_m3: f64,
    pub max_length_m: f64,
    pub life_years: f64,
    pub short_share: f64,
}

impl DitchesFile {
    fn params(&self) -> civ_agents::irrigation::DitchPlanParams {
        civ_agents::irrigation::DitchPlanParams {
            bottom_m: self.bottom_m,
            depth_m: self.depth_m,
            side_run: self.side_run,
            h_per_m3: self.h_per_m3,
            max_length_m: self.max_length_m,
            life_years: self.life_years,
            short_share: self.short_share,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("ditches.bottom_m", self.bottom_m, 0.05, 5.0),
            ("ditches.depth_m", self.depth_m, 0.05, 3.0),
            ("ditches.side_run", self.side_run, 0.0, 10.0),
            ("ditches.h_per_m3", self.h_per_m3, 0.1, 100.0),
            ("ditches.max_length_m", self.max_length_m, 1.0, 5000.0),
            ("ditches.life_years", self.life_years, 1.0, 500.0),
            ("ditches.short_share", self.short_share, 0.0, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
    }
}

/// How people lay out and build flood banks (M6b slice BG, ADR-0022 §11; content API 88). See
/// [`civ_agents::levees::LeveeParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LeveesFile {
    pub memory_years: f64,
    pub life_years: f64,
    pub freeboard_m: f64,
    pub min_height_m: f64,
    pub max_height_m: f64,
    pub crest_width_m: f64,
    pub side_run: f64,
    pub compacted_per_bank: f64,
    pub loose_per_bank: f64,
    pub loose_kg_per_m3: f64,
    pub carry_kg: f64,
    pub load_min: f64,
    pub place_h_per_m3: f64,
    pub pit_depth_m: f64,
    pub front_m: f64,
    pub mend_crest_m: f64,
    pub mend_piping: f64,
}

impl LeveesFile {
    fn params(&self) -> civ_agents::levees::LeveeParams {
        civ_agents::levees::LeveeParams {
            memory_years: self.memory_years,
            life_years: self.life_years,
            freeboard_m: self.freeboard_m,
            min_height_m: self.min_height_m,
            max_height_m: self.max_height_m,
            crest_width_m: self.crest_width_m,
            side_run: self.side_run,
            compacted_per_bank: self.compacted_per_bank,
            loose_per_bank: self.loose_per_bank,
            loose_kg_per_m3: self.loose_kg_per_m3,
            carry_kg: self.carry_kg,
            load_min: self.load_min,
            place_h_per_m3: self.place_h_per_m3,
            pit_depth_m: self.pit_depth_m,
            front_m: self.front_m,
            mend_crest_m: self.mend_crest_m,
            mend_piping: self.mend_piping,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("levees.memory_years", self.memory_years, 0.1, 200.0),
            ("levees.life_years", self.life_years, 1.0, 500.0),
            ("levees.freeboard_m", self.freeboard_m, 0.0, 5.0),
            ("levees.min_height_m", self.min_height_m, 0.05, 10.0),
            ("levees.max_height_m", self.max_height_m, 0.05, 20.0),
            ("levees.crest_width_m", self.crest_width_m, 0.1, 20.0),
            ("levees.side_run", self.side_run, 0.5, 10.0),
            (
                "levees.compacted_per_bank",
                self.compacted_per_bank,
                0.5,
                1.0,
            ),
            ("levees.loose_per_bank", self.loose_per_bank, 1.0, 2.0),
            (
                "levees.loose_kg_per_m3",
                self.loose_kg_per_m3,
                500.0,
                3000.0,
            ),
            ("levees.carry_kg", self.carry_kg, 1.0, 100.0),
            ("levees.load_min", self.load_min, 0.0, 30.0),
            ("levees.place_h_per_m3", self.place_h_per_m3, 0.1, 100.0),
            ("levees.pit_depth_m", self.pit_depth_m, 0.1, 5.0),
            ("levees.front_m", self.front_m, 0.5, 100.0),
            ("levees.mend_crest_m", self.mend_crest_m, 0.01, 2.0),
            ("levees.mend_piping", self.mend_piping, 0.01, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.max_height_m < self.min_height_m {
            p.push("`levees.max_height_m` must not be below `levees.min_height_m`".to_owned());
        }
    }
}

/// How buildings burn and fire passes between them (M6b slice BC, ADR-0022 §4-§5; content API
/// 81), how fires in use escape (slice BD; content API 82), and how an escape grows and people
/// notice and shout (content API 83). See [`civ_agents::fire::FireParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FireFile {
    pub reach_m: f64,
    pub flux_kw: [f64; 2],
    pub covering_flux_share: f64,
    pub wind_stretch_per_ms: f64,
    pub exposure_minutes: f64,
    pub catch_per_minute: f64,
    pub extinction_moisture: f64,
    pub reference_moisture: f64,
    pub involve_minutes: f64,
    pub involved_minutes: f64,
    pub covering_minutes: f64,
    pub char_mm_first_hour: f64,
    pub char_exponent: f64,
    pub infill_loss: f64,
    pub hearth_lit_per_hour: f64,
    pub hearth_banked_per_hour: f64,
    pub worked_per_hour: f64,
    /// Content API 166 (M10b slice DU): ignitions per hour of a lamp lit at home.
    #[serde(default)]
    pub lamp_lit_per_hour: f64,
    pub beside_m: f64,
    pub grow_minutes: [f64; 2],
    pub wake_minutes: [f64; 2],
    pub sight_m: f64,
    pub earshot_m: f64,
    pub water_stays: f64,
    pub wet_drains_per_minute: f64,
    pub w_save: f64,
    pub kin_share: f64,
    pub neighbour_share: f64,
    pub strip_m2_per_minute: f64,
    pub strip_cost: f64,
    pub salvage_share: f64,
    pub pull_crew: u16,
    pub pull_h_per_m2: f64,
    pub pull_cost: f64,
}

impl FireFile {
    fn params(&self) -> civ_agents::fire::FireParams {
        civ_agents::fire::FireParams {
            reach_m: self.reach_m,
            flux_kw: self.flux_kw,
            covering_flux_share: self.covering_flux_share,
            wind_stretch_per_ms: self.wind_stretch_per_ms,
            exposure_minutes: self.exposure_minutes,
            catch_per_minute: self.catch_per_minute,
            extinction_moisture: self.extinction_moisture,
            reference_moisture: self.reference_moisture,
            involve_minutes: self.involve_minutes,
            involved_minutes: self.involved_minutes,
            covering_minutes: self.covering_minutes,
            char_mm_first_hour: self.char_mm_first_hour,
            char_exponent: self.char_exponent,
            infill_loss: self.infill_loss,
            hearth_lit_per_hour: self.hearth_lit_per_hour,
            hearth_banked_per_hour: self.hearth_banked_per_hour,
            worked_per_hour: self.worked_per_hour,
            lamp_lit_per_hour: self.lamp_lit_per_hour,
            beside_m: self.beside_m,
            grow_minutes: self.grow_minutes,
            wake_minutes: self.wake_minutes,
            sight_m: self.sight_m,
            earshot_m: self.earshot_m,
            water_stays: self.water_stays,
            wet_drains_per_minute: self.wet_drains_per_minute,
            w_save: self.w_save,
            kin_share: self.kin_share,
            neighbour_share: self.neighbour_share,
            strip_m2_per_minute: self.strip_m2_per_minute,
            strip_cost: self.strip_cost,
            salvage_share: self.salvage_share,
            pull_crew: self.pull_crew,
            pull_h_per_m2: self.pull_h_per_m2,
            pull_cost: self.pull_cost,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        if !(2..=32).contains(&self.pull_crew) {
            p.push("fire.pull_crew must be between 2 and 32".to_owned());
        }

        for (name, v, lo, hi) in [
            ("fire.reach_m", self.reach_m, 0.0, 100.0),
            ("fire.flux_kw[0]", self.flux_kw[0], 0.1, 500.0),
            ("fire.flux_kw[1]", self.flux_kw[1], 0.1, 500.0),
            (
                "fire.covering_flux_share",
                self.covering_flux_share,
                0.0,
                1.0,
            ),
            (
                "fire.wind_stretch_per_ms",
                self.wind_stretch_per_ms,
                0.0,
                1.0,
            ),
            ("fire.exposure_minutes", self.exposure_minutes, 0.01, 120.0),
            ("fire.catch_per_minute", self.catch_per_minute, 0.0, 60.0),
            (
                "fire.extinction_moisture",
                self.extinction_moisture,
                0.01,
                3.0,
            ),
            ("fire.reference_moisture", self.reference_moisture, 0.0, 3.0),
            ("fire.involve_minutes", self.involve_minutes, 0.0, 600.0),
            ("fire.involved_minutes", self.involved_minutes, 1.0, 6000.0),
            ("fire.covering_minutes", self.covering_minutes, 0.1, 600.0),
            (
                "fire.char_mm_first_hour",
                self.char_mm_first_hour,
                0.0,
                200.0,
            ),
            ("fire.char_exponent", self.char_exponent, 0.1, 2.0),
            ("fire.infill_loss", self.infill_loss, 0.0, 1.0),
            (
                "fire.hearth_lit_per_hour",
                self.hearth_lit_per_hour,
                0.0,
                0.1,
            ),
            (
                "fire.hearth_banked_per_hour",
                self.hearth_banked_per_hour,
                0.0,
                0.1,
            ),
            ("fire.worked_per_hour", self.worked_per_hour, 0.0, 0.1),
            ("fire.lamp_lit_per_hour", self.lamp_lit_per_hour, 0.0, 0.1),
            ("fire.beside_m", self.beside_m, 0.0, 50.0),
            ("fire.grow_minutes[0]", self.grow_minutes[0], 0.0, 600.0),
            ("fire.grow_minutes[1]", self.grow_minutes[1], 0.0, 600.0),
            ("fire.wake_minutes[0]", self.wake_minutes[0], 0.0, 600.0),
            ("fire.wake_minutes[1]", self.wake_minutes[1], 0.0, 600.0),
            ("fire.sight_m", self.sight_m, 0.0, 2000.0),
            ("fire.earshot_m", self.earshot_m, 0.0, 1000.0),
            ("fire.water_stays", self.water_stays, 0.0, 1.0),
            (
                "fire.wet_drains_per_minute",
                self.wet_drains_per_minute,
                0.0,
                1.0,
            ),
            ("fire.w_save", self.w_save, 0.0, 1000.0),
            ("fire.kin_share", self.kin_share, 0.0, 1.0),
            ("fire.neighbour_share", self.neighbour_share, 0.0, 1.0),
            (
                "fire.strip_m2_per_minute",
                self.strip_m2_per_minute,
                0.0,
                100.0,
            ),
            ("fire.strip_cost", self.strip_cost, 0.0, 1.0),
            ("fire.salvage_share", self.salvage_share, 0.0, 10.0),
            ("fire.pull_h_per_m2", self.pull_h_per_m2, 0.001, 100.0),
            ("fire.pull_cost", self.pull_cost, 0.0, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.flux_kw[1] >= self.flux_kw[0] {
            p.push("`fire.flux_kw` must fall from 1 m to 4 m".to_owned());
        }
        if self.reference_moisture >= self.extinction_moisture {
            p.push("`fire.reference_moisture` must be below `fire.extinction_moisture`".to_owned());
        }
        for (name, [lo, hi]) in [
            ("fire.grow_minutes", self.grow_minutes),
            ("fire.wake_minutes", self.wake_minutes),
        ] {
            if lo > hi {
                p.push(format!("`{name}` must run from the least to the most"));
            }
        }
    }
}

/// How people tally a source of water (M6a slice BA, ADR-0021 §6; content API 76), and what they
/// believe its sickness costs (content API 78). See
/// [`civ_agents::suspicion::SuspicionParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SuspicionFile {
    pub min_households: u32,
    pub min_sick: u32,
    pub ratio: f64,
    pub draws_min_days: f64,
    pub known_min: f64,
    pub review_days: u32,
    pub case_days: f64,
    pub case_death_share: f64,
    pub w_death: f64,
}

impl SuspicionFile {
    fn params(&self) -> civ_agents::suspicion::SuspicionParams {
        civ_agents::suspicion::SuspicionParams {
            min_households: self.min_households,
            min_sick: self.min_sick,
            ratio: self.ratio,
            draws_min_days: self.draws_min_days,
            known_min: self.known_min,
            review_days: self.review_days,
            case_days: self.case_days,
            case_death_share: self.case_death_share,
            w_death: self.w_death,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("suspicion.min_households", self.min_households, 1, 1000),
            ("suspicion.min_sick", self.min_sick, 1, 1000),
            ("suspicion.review_days", self.review_days, 1, 365),
        ] {
            if !(lo..=hi).contains(&v) {
                p.push(format!("`{name}` must be {lo} to {hi} (got {v})"));
            }
        }
        for (name, v, lo, hi) in [
            ("suspicion.ratio", self.ratio, 1.0, 100.0),
            ("suspicion.draws_min_days", self.draws_min_days, 0.0, 365.0),
            ("suspicion.known_min", self.known_min, 0.0, 1.0),
            ("suspicion.case_days", self.case_days, 0.0, 365.0),
            (
                "suspicion.case_death_share",
                self.case_death_share,
                0.0,
                1.0,
            ),
            ("suspicion.w_death", self.w_death, 0.0, 100_000.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
    }
}

/// Factions, their petitions, refusals and revolts (M4c slices AH-AI, ADR-0017 §2-4; content API
/// 45-48). See
/// [`civ_agents::faction::FactionParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactionFile {
    pub review_days: u32,
    pub found_floor: f64,
    pub trust_floor: f64,
    pub shared_full: f64,
    pub w_grievance: f64,
    pub w_shared: f64,
    pub w_organizer: f64,
    pub w_belong: f64,
    pub w_dues: f64,
    pub found_cost: f64,
    pub retry_days: u32,
    pub threshold: [f64; 2],
    pub leave_margin: f64,
    pub dues_share: f64,
    pub reserve_days: f64,
    pub aid_days: f64,
    pub petition_members: u32,
    pub petition_days: u32,
    pub petition_cost: f64,
    pub w_member: f64,
    pub w_expect: f64,
    pub free_ride_share: f64,
    pub refused_days: f64,
    pub refusal_cost: f64,
    pub refusal_days: u32,
    pub revolt_cost: f64,
    pub revolt_days: u32,
    pub hold_days: u32,
    pub w_exclusion: f64,
    /// Content API 50 (M4c slice AI, step three): what calling a coup costs a watcher, points.
    pub coup_cost: f64,
    /// Content API 115 (M7c slice BW): organizations by stake.
    pub organize_days: f64,
    pub w_stake: f64,
    pub stake_admit: String,
    pub stake_decides: String,
    pub stake_dues_share: f64,
}

impl FactionFile {
    fn params(&self) -> civ_agents::faction::FactionParams {
        civ_agents::faction::FactionParams {
            review_days: self.review_days,
            found_floor: self.found_floor,
            trust_floor: self.trust_floor,
            shared_full: self.shared_full,
            w_grievance: self.w_grievance,
            w_shared: self.w_shared,
            w_organizer: self.w_organizer,
            w_belong: self.w_belong,
            w_dues: self.w_dues,
            found_cost: self.found_cost,
            retry_days: self.retry_days,
            threshold: self.threshold,
            leave_margin: self.leave_margin,
            dues_share: self.dues_share,
            reserve_days: self.reserve_days,
            aid_days: self.aid_days,
            petition_members: self.petition_members,
            petition_days: self.petition_days,
            petition_cost: self.petition_cost,
            w_member: self.w_member,
            w_expect: self.w_expect,
            free_ride_share: self.free_ride_share,
            refused_days: self.refused_days,
            refusal_cost: self.refusal_cost,
            refusal_days: self.refusal_days,
            revolt_cost: self.revolt_cost,
            revolt_days: self.revolt_days,
            hold_days: self.hold_days,
            w_exclusion: self.w_exclusion,
            coup_cost: self.coup_cost,
            organize_days: self.organize_days,
            w_stake: self.w_stake,
            stake_admit: civ_agents::faction::Admit::from_name(&self.stake_admit)
                .unwrap_or(civ_agents::faction::Admit::Sponsor),
            stake_decides: civ_agents::faction::Decides::from_name(&self.stake_decides)
                .unwrap_or(civ_agents::faction::Decides::Organizer),
            stake_dues_share: self.stake_dues_share,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        if self.retry_days > 36500 {
            p.push(format!(
                "`faction.retry_days` must be at most 36500 (got {})",
                self.retry_days
            ));
        }
        if !(1..=365).contains(&self.review_days) {
            p.push(format!(
                "`faction.review_days` must be between 1 and 365 (got {})",
                self.review_days
            ));
        }
        if !(2..=1000).contains(&self.petition_members) {
            p.push(format!(
                "`faction.petition_members` must be between 2 and 1000 (got {})",
                self.petition_members
            ));
        }
        for (name, v) in [
            ("faction.revolt_days", self.revolt_days),
            ("faction.hold_days", self.hold_days),
        ] {
            if !(1..=3650).contains(&v) {
                p.push(format!("`{name}` must be between 1 and 3650 (got {v})"));
            }
        }
        if !(1..=3650).contains(&self.refusal_days) {
            p.push(format!(
                "`faction.refusal_days` must be between 1 and 3650 (got {})",
                self.refusal_days
            ));
        }
        if self.petition_days > 36500 {
            p.push(format!(
                "`faction.petition_days` must be at most 36500 (got {})",
                self.petition_days
            ));
        }
        for (name, v, lo, hi) in [
            ("faction.found_floor", self.found_floor, 0.0, 1.0),
            ("faction.trust_floor", self.trust_floor, 0.0, 1.0),
            ("faction.shared_full", self.shared_full, 0.0, 100.0),
            ("faction.w_grievance", self.w_grievance, 0.0, 100.0),
            ("faction.w_shared", self.w_shared, 0.0, 100.0),
            ("faction.w_organizer", self.w_organizer, 0.0, 100.0),
            ("faction.w_belong", self.w_belong, 0.0, 100.0),
            ("faction.w_dues", self.w_dues, 0.0, 1000.0),
            ("faction.found_cost", self.found_cost, 0.0, 100.0),
            ("faction.threshold[0]", self.threshold[0], 0.0, 100.0),
            ("faction.threshold[1]", self.threshold[1], 0.0, 100.0),
            ("faction.leave_margin", self.leave_margin, 0.0, 100.0),
            ("faction.dues_share", self.dues_share, 0.0, 1.0),
            ("faction.reserve_days", self.reserve_days, 0.0, 3650.0),
            ("faction.aid_days", self.aid_days, 0.0, 365.0),
            ("faction.petition_cost", self.petition_cost, 0.0, 100.0),
            ("faction.w_member", self.w_member, 0.0, 100.0),
            ("faction.w_expect", self.w_expect, 0.0, 100.0),
            ("faction.free_ride_share", self.free_ride_share, 0.0, 1.0),
            ("faction.refused_days", self.refused_days, 0.0, 365.0),
            ("faction.refusal_cost", self.refusal_cost, 0.0, 100.0),
            ("faction.revolt_cost", self.revolt_cost, 0.0, 100.0),
            ("faction.w_exclusion", self.w_exclusion, 0.0, 100.0),
            ("faction.coup_cost", self.coup_cost, 0.0, 100.0),
            ("faction.organize_days", self.organize_days, 0.1, 3650.0),
            ("faction.w_stake", self.w_stake, 0.0, 100.0),
            ("faction.stake_dues_share", self.stake_dues_share, 0.0, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.threshold[0] > self.threshold[1] {
            p.push("`faction.threshold` must run from low to high".to_owned());
        }
        if civ_agents::faction::Admit::from_name(&self.stake_admit).is_none() {
            p.push(format!(
                "`faction.stake_admit` must be \"open\" or \"sponsor\" (got {:?})",
                self.stake_admit
            ));
        }
        if civ_agents::faction::Decides::from_name(&self.stake_decides).is_none() {
            p.push(format!(
                "`faction.stake_decides` must be \"organizer\" or \"members\" (got {:?})",
                self.stake_decides
            ));
        }
    }
}

/// Opinion (M4c slice AG, ADR-0016 §4; content API 41). See
/// [`civ_agents::opinion::OpinionParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpinionFile {
    pub share: f64,
    pub eta: f64,
    pub epsilon: f64,
    pub anchor_half_life_days: f64,
    pub anchor_points: f64,
    pub w_position: f64,
    pub salience_live: f64,
    pub salience_idle: f64,
    pub youth_until: f64,
    pub youth_factor: f64,
}

impl OpinionFile {
    fn params(&self) -> civ_agents::opinion::OpinionParams {
        civ_agents::opinion::OpinionParams {
            share: self.share,
            eta: self.eta,
            epsilon: self.epsilon,
            anchor_half_life_days: self.anchor_half_life_days,
            anchor_points: self.anchor_points,
            w_position: self.w_position,
            salience_live: self.salience_live,
            salience_idle: self.salience_idle,
            youth_until: self.youth_until,
            youth_factor: self.youth_factor,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("opinion.share", self.share, 0.0, 1.0),
            ("opinion.eta", self.eta, 0.0, 1.0),
            ("opinion.epsilon", self.epsilon, 0.01, 10.0),
            (
                "opinion.anchor_half_life_days",
                self.anchor_half_life_days,
                1.0,
                36500.0,
            ),
            ("opinion.anchor_points", self.anchor_points, 0.01, 100.0),
            ("opinion.w_position", self.w_position, 0.0, 100.0),
            ("opinion.salience_live", self.salience_live, 0.0, 1.0),
            ("opinion.salience_idle", self.salience_idle, 0.0, 1.0),
            ("opinion.youth_until", self.youth_until, 0.0, 120.0),
            ("opinion.youth_factor", self.youth_factor, 0.0, 10.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
    }
}

/// How households come to know other places (M5a slice AM, ADR-0018 §4; content API 52), and
/// what a visit to one is worth (content API 53). See [`civ_agents::places::PlacesParams`] for
/// what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlacesFile {
    pub sight_m: f32,
    pub share_told: f64,
    pub w_kin: f64,
    pub w_ties: f64,
    pub w_seek: f64,
    pub seek_days: i64,
    pub revisit_days: f64,
    /// Content API 59 (M5c slice AT): days over which what a household holds of the places its
    /// people work halves.
    pub use_half_life_days: f64,
}

/// What moving to another settlement is worth to a household (M5a slice AN, ADR-0018 §5; content
/// API 54). See [`civ_agents::places::MovingParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MovingFile {
    pub w_kin: f64,
    pub w_ties: f64,
    pub w_fed: f64,
    pub w_grievance: f64,
    pub w_stake: f64,
    pub cost: f64,
    pub reviews: u32,
    /// Content API 77 (M6a slice BA, step three).
    pub w_sickness: f64,
    /// Content API 97 (M6c slice BJ, step two).
    pub w_raids: f64,
    /// Content API 139 (M8b slice CR, second part).
    pub w_walk: f64,
}

/// What founding a settlement of its own is worth to a household, and what a coalition must hold
/// to go (M5a slice AO; content API 55). See [`civ_agents::places::FoundingParams`] for what each
/// means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FoundingFile {
    pub cost: f64,
    pub yield_share: f64,
    pub walk_hours: f64,
    pub candidates: u32,
    pub buffer_months: f64,
    pub work_h_per_day: f64,
}

/// How price reports of other settlements' markets are held and passed on (M5b slice AP; content
/// API 56). See [`civ_agents::reports::ReportParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReportsFile {
    pub half_life_days: f64,
    pub max_age_days: i64,
    pub share_told: f64,
}

/// How views of other polities are held (M5c slice AT, ADR-0020 §3; content API 60). See
/// [`civ_agents::views::RelationsParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationsFile {
    pub prior: f64,
    pub half_life_days: f64,
    pub seen_trespass: f64,
    pub heard_trespass: f64,
    /// Content API 95 (M6c slice BI): the same for takings from a store of one's settlement.
    pub seen_taking: f64,
    pub heard_taking: f64,
    /// Content API 61 (M5c slice AU): word of claims at the hearth, and what a claimed place is
    /// worth to a household that heard of the claim.
    pub share_claims: f64,
    pub claimed_worth: f64,
    /// Content API 62 (M5c slice AU, step two): the packages two who meet weigh, the days an
    /// agreement waits for the other gathering and for word of it, and the terms it may run.
    pub packages: u32,
    pub answer_days: i64,
    pub terms_days: Vec<u32>,
    /// Content API 63 (M5c slice AV): the gifts and transfers a store may give for leave, the days
    /// between transfers, the days a payment may take to be handed over, and what carrying one
    /// is worth to its carrier.
    pub gifts_kg: Vec<u32>,
    pub transfers_kg: Vec<u32>,
    pub transfer_days: u32,
    pub deliver_days: i64,
    pub carry_points: f64,
    /// Content API 64 (M5c slice AV, step two): the evidence a payment handed over or missed gives
    /// those of the receiving polity who know its law deciding the agreement.
    pub performance: f64,
    /// Content API 103 (M6c slice BL): the share of what people of another settlement took from
    /// a household that it believes their polity's word against raiding keeps from it.
    pub raids_kept: f64,
}

/// Word of mouth and grievances (M4c slice AE, ADR-0016; content API 39). See
/// [`civ_agents::word::WordParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WordFile {
    pub share_home: f64,
    pub share_urgent: f64,
    pub share_routine: f64,
    pub news_days: u32,
    pub max_grievances: u32,
    pub half_life_days: [f64; 4],
    pub full_harm_days: f64,
    pub reminder: f64,
    pub tell_floor: f64,
    pub remind_days: u32,
    /// Content API 75 (M6a slice BA, step one).
    pub share_sickness: f64,
    pub sickness_days: u32,
    pub sickness_spell_days: u32,
}

impl WordFile {
    fn params(&self) -> civ_agents::word::WordParams {
        civ_agents::word::WordParams {
            share_home: self.share_home,
            share_urgent: self.share_urgent,
            share_routine: self.share_routine,
            news_days: self.news_days,
            max_grievances: self.max_grievances,
            half_life_days: self.half_life_days,
            full_harm_days: self.full_harm_days,
            reminder: self.reminder,
            tell_floor: self.tell_floor,
            remind_days: self.remind_days,
            share_sickness: self.share_sickness,
            sickness_days: self.sickness_days,
            sickness_spell_days: self.sickness_spell_days,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("word.share_home", self.share_home, 0.0, 1.0),
            ("word.share_urgent", self.share_urgent, 0.0, 1.0),
            ("word.share_routine", self.share_routine, 0.0, 1.0),
            ("word.full_harm_days", self.full_harm_days, 0.01, 3650.0),
            ("word.reminder", self.reminder, 0.0, 1.0),
            ("word.tell_floor", self.tell_floor, 0.0, 1.0),
            ("word.share_sickness", self.share_sickness, 0.0, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        for &h in &self.half_life_days {
            if !(h.is_finite() && h > 0.0 && h <= 3650.0) {
                p.push(format!(
                    "`word.half_life_days` must each be above 0 and at most 3650 (got {h})"
                ));
            }
        }
        if !(1..=3650).contains(&self.news_days) {
            p.push(format!(
                "`word.news_days` must be 1 to 3650 (got {})",
                self.news_days
            ));
        }
        if !(1..=365).contains(&self.remind_days) {
            p.push(format!(
                "`word.remind_days` must be 1 to 365 (got {})",
                self.remind_days
            ));
        }
        if !(1..=64).contains(&self.max_grievances) {
            p.push(format!(
                "`word.max_grievances` must be 1 to 64 (got {})",
                self.max_grievances
            ));
        }
        for (name, v) in [
            ("word.sickness_days", self.sickness_days),
            ("word.sickness_spell_days", self.sickness_spell_days),
        ] {
            if !(1..=365).contains(&v) {
                p.push(format!("`{name}` must be 1 to 365 (got {v})"));
            }
        }
    }
}

/// Taking and what follows it (M4b slice AA, ADR-0015; content API 35; cases from content API 36). See
/// [`civ_agents::crime::CrimeParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CrimeFile {
    pub objection_mean: f64,
    pub objection_sd: f64,
    pub objection_heritability: f64,
    pub objection_filter: f64,
    pub w_objection: f64,
    pub w_seen: f64,
    pub w_risk_trait: f64,
    pub w_regard: f64,
    pub risk_prior: f64,
    pub risk_alpha: f64,
    pub risk_alpha_told: f64,
    pub sight_m: f64,
    /// Sight in the dark once a lamp has been lit (content API 167; absent, day sight).
    #[serde(default)]
    pub dark_sight_m: Option<f64>,
    /// Metres about a building where a lamp burns within which its light restores day sight.
    #[serde(default)]
    pub lit_m: f64,
    pub notice_chance: f64,
    pub retry_hours: f64,
    pub guardian_age: f64,
    pub wake_chance: f64,
    pub remember_days: u32,
    pub refuse_regard: f64,
    pub demand_base: f64,
    pub w_demand_loss: f64,
    pub w_forgive: f64,
    pub due_days: u32,
    pub comply_base: f64,
    pub w_comply_known: f64,
    pub w_comply_regard: f64,
    pub w_comply_cost: f64,
    pub keep_days: f64,
    pub report_cost: f64,
    pub w_case_belief: f64,
    pub w_comply_found: f64,
    pub exile_days: f64,
    pub watch_guard: f64,
    pub lit_guard: f64,
    pub w_light: f64,
    pub w_watch: f64,
    pub rounds_per_night: u32,
    pub round_stops: u32,
    pub w_watch_report: f64,
    pub ask_days: f64,
    /// Content API 117 (M7c slice BY): the keeper's diversion and relief by favour.
    pub divert_share: f64,
    pub favour_reserve_days: f64,
    pub favour_floor: f64,
    pub collector_sight: f64,
    pub collector_ask_share: f64,
    pub curfew_guard: f64,
    pub curfew_cost_days: f64,
    /// Content API 51 (M4c slice AI, step four): force.
    pub w_collect: f64,
    pub w_harm: f64,
    pub strike_threshold: [f64; 2],
    pub hurt_days: [u16; 2],
    pub kill_share: f64,
    /// Content API 111 (M7b slice BS).
    pub recall_k: u32,
    pub w_persuasive: f64,
    pub w_binding: f64,
    /// Content API 112 (M7b slice BT).
    pub w_reading: f64,
}

impl CrimeFile {
    fn params(&self) -> civ_agents::crime::CrimeParams {
        civ_agents::crime::CrimeParams {
            objection_mean: self.objection_mean,
            objection_sd: self.objection_sd,
            objection_heritability: self.objection_heritability,
            objection_filter: self.objection_filter,
            w_objection: self.w_objection,
            w_seen: self.w_seen,
            w_risk_trait: self.w_risk_trait,
            w_regard: self.w_regard,
            risk_prior: self.risk_prior,
            risk_alpha: self.risk_alpha,
            risk_alpha_told: self.risk_alpha_told,
            sight_m: self.sight_m,
            dark_sight_m: self.dark_sight_m.unwrap_or(self.sight_m),
            lit_m: self.lit_m,
            notice_chance: self.notice_chance,
            retry_hours: self.retry_hours,
            guardian_age: self.guardian_age,
            wake_chance: self.wake_chance,
            remember_days: self.remember_days,
            refuse_regard: self.refuse_regard,
            demand_base: self.demand_base,
            w_demand_loss: self.w_demand_loss,
            w_forgive: self.w_forgive,
            due_days: self.due_days,
            comply_base: self.comply_base,
            w_comply_known: self.w_comply_known,
            w_comply_regard: self.w_comply_regard,
            w_comply_cost: self.w_comply_cost,
            keep_days: self.keep_days,
            report_cost: self.report_cost,
            w_case_belief: self.w_case_belief,
            w_comply_found: self.w_comply_found,
            exile_days: self.exile_days,
            watch_guard: self.watch_guard,
            lit_guard: self.lit_guard,
            w_light: self.w_light,
            w_watch: self.w_watch,
            rounds_per_night: self.rounds_per_night,
            round_stops: self.round_stops,
            w_watch_report: self.w_watch_report,
            ask_days: self.ask_days,
            divert_share: self.divert_share,
            favour_reserve_days: self.favour_reserve_days,
            favour_floor: self.favour_floor,
            collector_sight: self.collector_sight,
            collector_ask_share: self.collector_ask_share,
            curfew_guard: self.curfew_guard,
            curfew_cost_days: self.curfew_cost_days,
            w_collect: self.w_collect,
            w_harm: self.w_harm,
            strike_threshold: self.strike_threshold,
            hurt_days: self.hurt_days,
            kill_share: self.kill_share,
            recall_k: self.recall_k,
            w_persuasive: self.w_persuasive,
            w_binding: self.w_binding,
            w_reading: self.w_reading,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("crime.remember_days", self.remember_days, 0, 36_500),
            ("crime.due_days", self.due_days, 1, 3650),
            ("crime.rounds_per_night", self.rounds_per_night, 1, 24),
            ("crime.round_stops", self.round_stops, 1, 1000),
            ("crime.recall_k", self.recall_k, 3, 8),
        ] {
            if !(lo..=hi).contains(&v) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        for (name, v, lo, hi) in [
            ("crime.objection_mean", self.objection_mean, -20.0, 20.0),
            ("crime.objection_sd", self.objection_sd, 0.0, 20.0),
            (
                "crime.objection_heritability",
                self.objection_heritability,
                0.0,
                1.0,
            ),
            ("crime.objection_filter", self.objection_filter, 0.0, 1.0),
            ("crime.w_objection", self.w_objection, 0.0, 1000.0),
            ("crime.w_seen", self.w_seen, 0.0, 1000.0),
            ("crime.w_risk_trait", self.w_risk_trait, 0.0, 10.0),
            ("crime.w_regard", self.w_regard, 0.0, 1000.0),
            ("crime.risk_prior", self.risk_prior, 0.0, 1.0),
            ("crime.risk_alpha", self.risk_alpha, 0.0, 1.0),
            ("crime.risk_alpha_told", self.risk_alpha_told, 0.0, 1.0),
            ("crime.sight_m", self.sight_m, 0.0, 10_000.0),
            (
                "crime.dark_sight_m",
                self.dark_sight_m.unwrap_or(self.sight_m),
                0.0,
                self.sight_m,
            ),
            ("crime.lit_m", self.lit_m, 0.0, 100.0),
            ("crime.notice_chance", self.notice_chance, 0.0, 1.0),
            ("crime.retry_hours", self.retry_hours, 0.0, 87_600.0),
            ("crime.guardian_age", self.guardian_age, 0.0, 130.0),
            ("crime.wake_chance", self.wake_chance, 0.0, 1.0),
            ("crime.refuse_regard", self.refuse_regard, 0.0, 100.0),
            ("crime.demand_base", self.demand_base, -100.0, 100.0),
            ("crime.w_demand_loss", self.w_demand_loss, 0.0, 100.0),
            ("crime.w_forgive", self.w_forgive, 0.0, 100.0),
            ("crime.comply_base", self.comply_base, -100.0, 100.0),
            ("crime.w_comply_known", self.w_comply_known, 0.0, 100.0),
            ("crime.w_comply_regard", self.w_comply_regard, 0.0, 100.0),
            ("crime.w_comply_cost", self.w_comply_cost, 0.0, 100.0),
            ("crime.keep_days", self.keep_days, 0.0, 3650.0),
            ("crime.report_cost", self.report_cost, 0.0, 100.0),
            ("crime.w_case_belief", self.w_case_belief, 0.0, 100.0),
            ("crime.w_comply_found", self.w_comply_found, -100.0, 100.0),
            ("crime.exile_days", self.exile_days, 0.0, 3650.0),
            ("crime.watch_guard", self.watch_guard, 0.0, 1.0),
            ("crime.lit_guard", self.lit_guard, 0.0, 1.0),
            ("crime.w_light", self.w_light, 0.0, 100.0),
            ("crime.w_watch", self.w_watch, 0.0, 100.0),
            ("crime.w_watch_report", self.w_watch_report, -100.0, 100.0),
            ("crime.ask_days", self.ask_days, 0.0, 365.0),
            ("crime.divert_share", self.divert_share, 0.0, 1.0),
            (
                "crime.favour_reserve_days",
                self.favour_reserve_days,
                0.0,
                3650.0,
            ),
            ("crime.favour_floor", self.favour_floor, -1.0, 1.0),
            ("crime.collector_sight", self.collector_sight, 0.0, 1.0),
            (
                "crime.collector_ask_share",
                self.collector_ask_share,
                0.0,
                1.0,
            ),
            ("crime.curfew_guard", self.curfew_guard, 0.0, 1.0),
            ("crime.curfew_cost_days", self.curfew_cost_days, 0.0, 365.0),
            ("crime.w_collect", self.w_collect, -100.0, 100.0),
            ("crime.w_harm", self.w_harm, 0.0, 100.0),
            ("crime.kill_share", self.kill_share, 0.0, 1.0),
            ("crime.w_persuasive", self.w_persuasive, 0.0, 100.0),
            ("crime.w_binding", self.w_binding, 0.0, 100.0),
            ("crime.w_reading", self.w_reading, 0.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        let [lo, hi] = self.strike_threshold;
        if !(lo.is_finite() && hi.is_finite() && 0.0 <= lo && lo <= hi && hi <= 100.0) {
            p.push(format!(
                "`crime.strike_threshold` must be two points from 0 to 100, the first at most \
                 the second (got [{lo}, {hi}])"
            ));
        }
        let [lo, hi] = self.hurt_days;
        if !(1 <= lo && lo <= hi && hi <= 365) {
            p.push(format!(
                "`crime.hurt_days` must be two days from 1 to 365, the first at most the second \
                 (got [{lo}, {hi}])"
            ));
        }
    }
}

/// The polity (M4a slice Z, ADR-0013; content API 31). See
/// [`civ_agents::polity::PolityParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolityFile {
    pub review_days: u32,
    pub notice_days: u32,
    pub gathering_minutes: u32,
    pub quorum_share: f64,
    pub w_gain: f64,
    pub w_regard: f64,
    pub stance_margin: f64,
    pub attend_base: f64,
    pub w_attend: f64,
    /// Content API 154 (M8d slice DC): what rain takes from a gathering in the open.
    pub wet_attend: f64,
    pub w_followers: f64,
    pub propose_cost: f64,
    pub temperature: f64,
    pub vote_memory_days: u32,
    pub prior_lean: f64,
    pub prior_years: f64,
    pub lean_harvest: f64,
    /// Content API 59 (M5c slice AT): the share of what outsiders take that a claim is believed to
    /// keep.
    pub claim_keeps: f64,
    pub subsistence_share: f64,
    pub comply_base: f64,
    pub w_stance: f64,
    /// Content API 49 (M4c slice AI): days after the custom is taken in which its new body weighs
    /// ending the laws the old one made.
    pub founding_days: u32,
    /// Content API 117 (M7c slice BY): what a client owes a patron, and the most one gift is.
    pub w_owed: f64,
    pub owed_memory_days: u32,
    pub patron_gift_days: f64,
}

impl PolityFile {
    fn params(&self) -> civ_agents::polity::PolityParams {
        civ_agents::polity::PolityParams {
            review_days: self.review_days,
            notice_days: self.notice_days,
            gathering_minutes: self.gathering_minutes,
            quorum_share: self.quorum_share,
            w_gain: self.w_gain,
            w_regard: self.w_regard,
            stance_margin: self.stance_margin,
            attend_base: self.attend_base,
            w_attend: self.w_attend,
            wet_attend: self.wet_attend,
            w_followers: self.w_followers,
            propose_cost: self.propose_cost,
            temperature: self.temperature,
            vote_memory_days: self.vote_memory_days,
            prior_lean: self.prior_lean,
            prior_years: self.prior_years,
            lean_harvest: self.lean_harvest,
            claim_keeps: self.claim_keeps,
            subsistence_share: self.subsistence_share,
            comply_base: self.comply_base,
            w_stance: self.w_stance,
            founding_days: self.founding_days,
            w_owed: self.w_owed,
            owed_memory_days: self.owed_memory_days,
            patron_gift_days: self.patron_gift_days,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        for (name, v, lo, hi) in [
            ("polity.review_days", self.review_days, 1, 365),
            ("polity.notice_days", self.notice_days, 1, 30),
            ("polity.gathering_minutes", self.gathering_minutes, 15, 600),
            ("polity.vote_memory_days", self.vote_memory_days, 0, 3650),
            ("polity.founding_days", self.founding_days, 1, 3650),
            ("polity.owed_memory_days", self.owed_memory_days, 1, 3650),
        ] {
            if !(lo..=hi).contains(&v) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        for (name, v, lo, hi) in [
            ("polity.quorum_share", self.quorum_share, 0.0, 1.0),
            ("polity.w_gain", self.w_gain, 0.0, 100.0),
            ("polity.w_regard", self.w_regard, 0.0, 100.0),
            ("polity.stance_margin", self.stance_margin, 0.0, 100.0),
            ("polity.attend_base", self.attend_base, -100.0, 100.0),
            ("polity.w_attend", self.w_attend, 0.0, 100.0),
            ("polity.wet_attend", self.wet_attend, 0.0, 100.0),
            ("polity.w_followers", self.w_followers, 0.0, 10.0),
            ("polity.propose_cost", self.propose_cost, 0.0, 100.0),
            ("polity.temperature", self.temperature, 0.01, 100.0),
            ("polity.prior_lean", self.prior_lean, 0.0, 100.0),
            ("polity.prior_years", self.prior_years, 0.01, 100.0),
            ("polity.lean_harvest", self.lean_harvest, 0.0, 1.0),
            ("polity.claim_keeps", self.claim_keeps, 0.0, 1.0),
            ("polity.subsistence_share", self.subsistence_share, 0.0, 1.0),
            ("polity.comply_base", self.comply_base, -100.0, 100.0),
            ("polity.w_stance", self.w_stance, 0.0, 100.0),
            ("polity.w_owed", self.w_owed, 0.0, 100.0),
            ("polity.patron_gift_days", self.patron_gift_days, 0.0, 365.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.prior_lean > self.prior_years {
            p.push(format!(
                "`polity.prior_lean` must be at most `polity.prior_years` (got {} of {})",
                self.prior_lean, self.prior_years
            ));
        }
    }
}

/// Standing and notables (M4a slice Y, ADR-0014 §3-4; content API 30). See
/// [`civ_agents::standing::StandingParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StandingFile {
    pub candidates: u32,
    pub notable_share: f64,
    pub notable_floor: u32,
    pub notable_keep: f64,
}

/// Ties between people (M4a slice Y, ADR-0014; content API 30). See
/// [`civ_agents::ties::TieParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TiesFile {
    pub room: u32,
    pub companions: u32,
    pub prior: f64,
    pub evidence_half_life_days: f64,
    pub familiarity_half_life_days: f64,
    pub warmth_half_life_days: f64,
    pub fear_half_life_days: f64,
    pub help_half_life_days: f64,
    pub hold_help_h: f64,
    pub salience_per_evidence: f64,
    pub new_share: f64,
    pub ask_known_min: f64,
    /// What each act the engine records writes into its holder's tie.
    pub acts: Vec<TieActFile>,
}

/// What one act writes, for each unit of it ([`civ_agents::ties::ActWeights`]). `domain` is
/// `"none"` for an act that is evidence of nothing.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TieActFile {
    pub act: String,
    pub familiarity: f64,
    pub warmth: f64,
    pub domain: String,
    pub good: f64,
    pub bad: f64,
}

impl TiesFile {
    fn params(&self) -> civ_agents::ties::TieParams {
        use civ_agents::ties::{Act, ActWeights, Domain, TieParams};
        let mut acts = [ActWeights::default(); civ_agents::ties::ACTS];
        for a in &self.acts {
            if let Some(act) = Act::from_name(&a.act) {
                acts[act as usize] = ActWeights {
                    familiarity: a.familiarity,
                    warmth: a.warmth,
                    domain: Domain::from_name(&a.domain),
                    good: a.good,
                    bad: a.bad,
                };
            }
        }
        TieParams {
            room: self.room as usize,
            companions: self.companions as usize,
            prior: self.prior,
            evidence_half_life_days: self.evidence_half_life_days,
            familiarity_half_life_days: self.familiarity_half_life_days,
            warmth_half_life_days: self.warmth_half_life_days,
            fear_half_life_days: self.fear_half_life_days,
            help_half_life_days: self.help_half_life_days,
            hold_help_h: self.hold_help_h,
            salience_per_evidence: self.salience_per_evidence,
            new_share: self.new_share,
            ask_known_min: self.ask_known_min,
            acts,
        }
    }

    fn problems(&self, p: &mut Vec<String>) {
        use civ_agents::ties::{Act, Domain};
        if !(1..=1000).contains(&self.room) {
            p.push(format!(
                "`ties.room` must be between 1 and 1000 (got {})",
                self.room
            ));
        }
        if self.companions > 100 {
            p.push(format!(
                "`ties.companions` must be at most 100 (got {})",
                self.companions
            ));
        }
        for (name, v, lo, hi) in [
            ("ties.prior", self.prior, 0.01, 100.0),
            (
                "ties.evidence_half_life_days",
                self.evidence_half_life_days,
                1.0,
                36_500.0,
            ),
            (
                "ties.familiarity_half_life_days",
                self.familiarity_half_life_days,
                1.0,
                36_500.0,
            ),
            (
                "ties.warmth_half_life_days",
                self.warmth_half_life_days,
                1.0,
                36_500.0,
            ),
            (
                "ties.fear_half_life_days",
                self.fear_half_life_days,
                1.0,
                36_500.0,
            ),
            (
                "ties.help_half_life_days",
                self.help_half_life_days,
                1.0,
                36_500.0,
            ),
            ("ties.hold_help_h", self.hold_help_h, 0.0, 10_000.0),
            (
                "ties.salience_per_evidence",
                self.salience_per_evidence,
                0.0,
                10.0,
            ),
            ("ties.new_share", self.new_share, 0.0, 1.0),
            ("ties.ask_known_min", self.ask_known_min, 0.0, 1440.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        let mut seen = std::collections::HashSet::new();
        for a in &self.acts {
            if Act::from_name(&a.act).is_none() {
                p.push(format!(
                    "`ties.acts` names an act the engine does not record: `{}`",
                    a.act
                ));
            } else if !seen.insert(a.act.as_str()) {
                p.push(format!("`ties.acts` gives act `{}` twice", a.act));
            }
            if a.domain != "none" && Domain::from_name(&a.domain).is_none() {
                p.push(format!(
                    "`ties.acts` act `{}` names an unknown domain `{}` (provision, craft, word, \
                     counsel or none)",
                    a.act, a.domain
                ));
            }
            for (what, v) in [("familiarity", a.familiarity), ("warmth", a.warmth)] {
                if !(v.is_finite() && (0.0..=1.0).contains(&v)) {
                    p.push(format!(
                        "`ties.acts` act `{}`: `{what}` must be between 0 and 1 (got {v})",
                        a.act
                    ));
                }
            }
            for (what, v) in [("good", a.good), ("bad", a.bad)] {
                if !(v.is_finite() && (0.0..=100.0).contains(&v)) {
                    p.push(format!(
                        "`ties.acts` act `{}`: `{what}` must be between 0 and 100 (got {v})",
                        a.act
                    ));
                }
            }
        }
        for act in Act::ALL {
            if !seen.contains(act.name()) {
                p.push(format!(
                    "`ties.acts` must say what act `{}` writes",
                    act.name()
                ));
            }
        }
    }
}

/// A household's midden and carrying it to the fields (M3c slice V; content API 29). See
/// [`civ_agents::params::MiddenParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MiddenFile {
    pub kg_per_person_day: f64,
    pub n_kg_per_person_year: f64,
    pub half_life_days: f64,
    pub load_kg: f64,
    pub spread_h_per_t: f64,
}

/// Taste in building, as authored: roof pitch in degrees, eaves and overhang in metres; and the
/// front of a frame of version 3 (M8d slice DD; content API 157), each 0 where not given: the
/// share of its walls left open, ornament in hours of craft a square metre, finish (0 bare, 1
/// daubed, 2 lime-washed) and symmetry (0 none to 1 a front set out about its middle).
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TasteFile {
    pub pitch_deg: f64,
    pub eave_m: f64,
    pub overhang_m: f64,
    #[serde(default)]
    pub openings: f64,
    #[serde(default)]
    pub ornament_h_per_m2: f64,
    #[serde(default)]
    pub finish: f64,
    #[serde(default)]
    pub symmetry: f64,
}

impl TasteFile {
    fn taste(self) -> civ_agents::params::Taste {
        civ_agents::params::Taste {
            pitch_centideg: (self.pitch_deg * 100.0) as f32,
            eave_cm: (self.eave_m * 100.0) as f32,
            overhang_cm: (self.overhang_m * 100.0) as f32,
            openings_permille: (self.openings * 1000.0) as f32,
            ornament_dh: (self.ornament_h_per_m2 * 10.0) as f32,
            finish: self.finish as f32,
            symmetry: self.symmetry as f32,
        }
    }
}

/// How households' taste in building moves (M3b slice R; content API 22).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StyleFile {
    /// The share of the way a taste moves toward the most admired new building.
    pub alpha: f64,
    /// How many times the least admired the most admired building weighs.
    pub prestige_most: f64,
    /// The chance a building has one trait new to its builders.
    pub innovation: f64,
    /// The way of building founding bands' are drawn around.
    pub tradition: TasteFile,
    /// How far a band's lies from it.
    pub tradition_spread: TasteFile,
    /// How far a household's lies from its band's.
    pub personal_spread: TasteFile,
    /// Buildings of other settlements a person keeps in mind until the household's review
    /// (content API 57).
    pub seen_most: u32,
    /// How far a person in another settlement sees its new buildings, metres (content API 57).
    pub sight_m: f64,
}

/// How people dig at a deposit (M3b slice Q, ADR-0010 §2; content API 20).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DiggingFile {
    /// Hours to dig a cubic metre of earth as it lay in the ground and lift it out.
    pub h_per_m3: f64,
    /// The side of a pit, and of the spoil heap beside it, metres.
    pub pit_side_m: f64,
    /// Content API 163 (M10a slice DS): a person's sustained power bailing a pit by hand, watts.
    #[serde(default = "bail_w")]
    pub bail_w: f64,
    /// Content API 163: the share of it that lifts water (bucket and rope).
    #[serde(default = "bail_efficiency")]
    pub bail_efficiency: f64,
    /// Content API 163: a pit's radius of influence on the water table, metres.
    #[serde(default = "pit_influence_m")]
    pub influence_m: f64,
}

/// A person's sustained power, watts, when a profile gives none (research 07-06 §1.3: 75 W).
fn bail_w() -> f64 {
    75.0
}

/// Bucket and rope's share of it that lifts water, when a profile gives none (07-06 §2.2).
fn bail_efficiency() -> f64 {
    0.5
}

/// A pit's radius of influence, metres, when a profile gives none (as the core's wells').
fn pit_influence_m() -> f64 {
    50.0
}

/// What founders know and how people learn (ADR-0008).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Knowledge {
    /// The share of founders old enough for the work who know each technique.
    pub founders: Vec<Founders>,
    /// Learners one person teaches at once.
    pub max_learners: u32,
    /// Utility points for working beside someone to learn what they know.
    pub w_learn: f64,
    /// The share of routine work's hours that counts as experiment (research 07-01 §2.3).
    pub experiment_share: f64,
    /// How many times its hours trying counts for someone already aware of the technique.
    pub aware_try_factor: f64,
    /// Utility points for trying at a problem, times the share of food it would cost.
    pub w_try: f64,
    /// Least days between one person's sessions of trying.
    pub try_gap_days: f64,
    /// How far, metres, someone in another settlement sees work done there (content API 58).
    pub watch_m: f64,
    /// Hours of writing that make a record of a technique (content API 182).
    pub record_h: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Founders {
    /// A technique id.
    pub technique: String,
    /// The share of founders old enough for its work who know it, 0-1.
    pub share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Firm {
    pub idle_close_days: f64,
    pub book_entries: u32,
    pub wage_share: f64,
    pub wage_review_days: u32,
    pub wage_max_change: f64,
    pub max_hire_hours: f64,
    /// Content API 164 (M10a slice DT): hours of a works' shift from its bell, by daylight.
    #[serde(default = "shift_h")]
    pub shift_h: f64,
}

fn shift_h() -> f64 {
    10.0
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Build {
    /// The programs a household may build: building ids of homes, stores and workshops.
    pub programs: Vec<String>,
    /// Days over which a household reckons what a storehouse would save of its goods.
    pub store_horizon_days: f64,
    /// People who can work at a craft at once in a home, beside living there.
    pub home_work_places: u32,
    /// Years over which a household reckons a home plot's walking (content API 140).
    pub site_years: f64,
    /// How unevenly a novice and a master make the parts of a building.
    pub quality_spread: [f64; 2],
    /// How builders answer the failures their settlement has seen.
    pub caution: Caution,
    /// How households level a plot on sloping ground (content API 19).
    pub levelling: Levelling,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Levelling {
    /// Ground dropping more than this across a plot, metres, is levelled before building.
    pub from_m: f64,
    /// Ground dropping more than this across a plot, metres, is not built on.
    pub most_m: f64,
    /// Hours to cut a cubic metre of earth and place it where it is wanted.
    pub h_per_m3: f64,
    /// A platform's sides' run, metres across for each metre up or down.
    pub side_run: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Caution {
    /// Years over which a settlement's memory of failures and of years without them fades by half.
    pub half_life_years: f64,
    /// How many times its usual strength a member is made at most.
    pub most: f64,
    /// Failures a building-year at which caution is half way to its most.
    pub half_rate: f64,
    /// How many failures more each death in one counts as.
    pub death_weight: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Farm {
    /// The crop: a crop id.
    pub crop: String,
    pub grain_share: f64,
    pub plan_yield_share: f64,
    pub loss_share: f64,
    pub grain_target_days: f64,
    pub work_hours_per_day: f64,
    /// Content API 24: field work a capable adult gives on a workable day at a peak (ADR-0012 §5).
    pub peak_work_hours_per_day: f64,
    pub field_m: f64,
    pub max_walk_minutes: f64,
    pub site_candidates: u32,
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
    /// Content API 99 (M6c slice BK).
    pub climb_factor: f64,
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
    pub eat_reserve_at_deficit: f64,
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
    /// Content API 70: what a person uses a day by the one-way walk to the water, pairs of
    /// minutes and litres; none for `water_l_per_person_day` whatever the walk.
    #[serde(default)]
    pub water_use_by_walk_min: Vec<[f64; 2]>,
    pub water_target_days: f64,
    pub food_target_days: f64,
    pub ready_food_days: f64,
    pub harvest_margin_days: f64,
    pub raised_store_factor: f64,
    pub processed_food_days: f64,
    pub carry_kg: f64,
    pub fuel_kg_per_person_day: [f64; 12],
    pub fuel_target_days: f64,
    pub short_food_days: f64,
    pub recovered_food_days: f64,
    pub daily_kcal_per_person: f64,
    pub leave_at_depletion: f64,
    pub leave_per_day: f64,
    pub leave_unless_ripe_within_days: f64,
    /// Leaving weighed (M4a slice Z; content API 32).
    pub leave_w_gap: f64,
    pub leave_w_stake: f64,
    pub leave_stay: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Decision {
    pub temperature_sd_fraction: f64,
    pub min_temperature: f64,
    pub w_hunger: f64,
    pub max_hunger_drive: f64,
    pub w_sleep: f64,
    pub w_social: f64,
    pub w_food: f64,
    pub w_lean: f64,
    pub w_work: f64,
    pub w_fuel: f64,
    pub w_farm: f64,
    pub w_deadline: f64,
    pub w_shelter: f64,
    pub w_tools: f64,
    pub trip_half_worth_days: f64,
    pub w_water: f64,
    pub w_walk_hour: f64,
    pub w_effort: f64,
    pub w_dark: f64,
    pub w_rest: f64,
    pub w_play: f64,
    /// Content API 74 (M6a slice AZ, step three).
    pub w_tend: f64,
    /// Content API 180 (M10c slice EB, second part).
    pub w_teach: f64,
    pub w_study: f64,
    /// Content API 182 (M10c slice EC).
    pub w_write: f64,
    pub w_read: f64,
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
    pub seed_kg_per_person: f64,
    pub elder_chance: f64,
    pub young_adult_chance: f64,
    pub birth_spacing_months: f64,
    pub site_max_slope: f64,
    pub site_w_food: f64,
    pub site_w_arable: f64,
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
    pub hunger_ratio_at_half: f64,
    pub hunger_ratio_max: f64,
    pub exhaustion_per_day: f64,
    pub exhaustion_power: f64,
    pub maternal_death_per_birth: f64,
    /// Content API 179 (M10c slice EA): how mothers' deaths in childbirth divide among the
    /// complications, and an attendant's harm fresh from the fevered.
    pub birth: BirthFile,
}

/// How mothers' deaths in childbirth divide among the complications (M10c slice EA).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BirthFile {
    pub bleeding: f64,
    pub infection: f64,
    pub obstructed: f64,
    pub fever_harm: f64,
    pub fever_harm_days: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Fertility {
    pub conception_per_month: f64,
    pub age_factor: Vec<[f64; 2]>,
    pub fecundity_sd: f64,
    pub hunger_halving: f64,
    pub pregnancy_days: f64,
    pub pregnancy_sd_days: f64,
    pub loss_by_age: Vec<[f64; 2]>,
    pub loss_days: [f64; 2],
    pub recovery_months: f64,
    pub recovery_sd_months: f64,
    pub recovery_min_months: f64,
    pub loss_recovery_months: f64,
    pub weaned_recovery_months: f64,
    pub boys_per_100_girls: f64,
    pub pregnancy_kcal_day: [f64; 3],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Market {
    pub review_days: u32,
    pub margin: f64,
    pub max_change: f64,
    /// Content API 26: how strongly a seller's ask for food answers what it can spare (ADR-0006
    /// §4's stock term).
    pub stock_response: f64,
    pub memory_days: f64,
    pub money_share: f64,
    pub money_min_trades: f64,
    pub accept_want: f64,
    pub recent_trades: u32,
    /// Content API 125 (M7d slice CD): a holder's desired balance of a weighed money, days of
    /// its spending in it.
    pub money_days: f64,
    /// Content API 127 (M7d slice CE2b): what striking the metal brought to a mint is worth to
    /// its minter as a duty, decision points.
    pub strike_points: f64,
    /// Content API 192 (M10d slice EG, second part): what a keeper takes each 30 days of what it
    /// keeps for another, a share of it.
    pub custody_fee_share: f64,
    /// Content API 194 (M10d slice EI): what a keeper keeps behind its notes, at least, of the
    /// pieces out, when it lends in them.
    pub note_coverage: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Family {
    pub seek_min_age_female: f64,
    pub seek_min_age_male: f64,
    pub seek_max_age_female: f64,
    pub seek_max_age_male: f64,
    pub seek_per_month_female: f64,
    pub seek_per_month_male: f64,
    pub age_gap_years: [f64; 2],
    pub preferred_gap_years: f64,
    pub w_gap_per_year: f64,
    pub kin_exclusion_generations: u32,
    /// `new_household`, `his_household` or `her_household`.
    pub residence: String,
    pub independent_age: f64,
    pub trait_heritability: f64,
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
    /// The parameters, with the given names and the indexes of the provisions good, the crop and
    /// the programs households may build. Field by field on purpose: a new parameter fails to compile here until
    /// the authoring format carries it.
    pub fn params(
        &self,
        names: NameParams,
        provisions_good: usize,
        crop: usize,
        programs: Vec<usize>,
        founders: Vec<(usize, f64)>,
    ) -> PeopleParams {
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
                climb_factor: w.climb_factor,
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
                eat_reserve_at_deficit: e.eat_reserve_at_deficit,
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
                water_use_by_walk: h
                    .water_use_by_walk_min
                    .iter()
                    .map(|&[m, l]| (m, l))
                    .collect(),
                water_target_days: h.water_target_days,
                food_target_days: h.food_target_days,
                ready_food_days: h.ready_food_days,
                harvest_margin_days: h.harvest_margin_days,
                raised_store_factor: h.raised_store_factor,
                processed_food_days: h.processed_food_days,
                carry_kg: h.carry_kg,
                fuel_kg_per_person_day: h.fuel_kg_per_person_day,
                fuel_target_days: h.fuel_target_days,
                short_food_days: h.short_food_days,
                recovered_food_days: h.recovered_food_days,
                daily_kcal_per_person: h.daily_kcal_per_person,
                leave_at_depletion: h.leave_at_depletion,
                leave_per_day: h.leave_per_day,
                leave_unless_ripe_within_days: h.leave_unless_ripe_within_days,
                leave_w_gap: h.leave_w_gap,
                leave_w_stake: h.leave_w_stake,
                leave_stay: h.leave_stay,
            },
            decision: DecisionParams {
                temperature_sd_fraction: d.temperature_sd_fraction,
                min_temperature: d.min_temperature,
                w_hunger: d.w_hunger,
                max_hunger_drive: d.max_hunger_drive,
                w_sleep: d.w_sleep,
                w_social: d.w_social,
                w_food: d.w_food,
                w_lean: d.w_lean,
                w_work: d.w_work,
                w_fuel: d.w_fuel,
                w_farm: d.w_farm,
                w_deadline: d.w_deadline,
                w_shelter: d.w_shelter,
                w_tools: d.w_tools,
                trip_half_worth_days: d.trip_half_worth_days,
                w_water: d.w_water,
                w_walk_hour: d.w_walk_hour,
                w_effort: d.w_effort,
                w_dark: d.w_dark,
                w_rest: d.w_rest,
                w_play: d.w_play,
                w_tend: d.w_tend,
                w_teach: d.w_teach,
                w_study: d.w_study,
                w_write: d.w_write,
                w_read: d.w_read,
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
                seed_kg_per_person: b.seed_kg_per_person,
                elder_chance: b.elder_chance,
                young_adult_chance: b.young_adult_chance,
                birth_spacing_months: b.birth_spacing_months,
                site_max_slope: b.site_max_slope,
                site_w_food: b.site_w_food,
                site_w_arable: b.site_w_arable,
                site_w_water_per_100m: b.site_w_water_per_100m,
                site_w_slope_per_pct: b.site_w_slope_per_pct,
                site_w_flood: b.site_w_flood,
                site_flood_hand_m: b.site_flood_hand_m,
            },
            farm: FarmParams {
                crop,
                grain_share: self.farm.grain_share,
                plan_yield_share: self.farm.plan_yield_share,
                loss_share: self.farm.loss_share,
                grain_target_days: self.farm.grain_target_days,
                work_hours_per_day: self.farm.work_hours_per_day,
                peak_work_hours_per_day: self.farm.peak_work_hours_per_day,
                field_m: self.farm.field_m,
                max_walk_minutes: self.farm.max_walk_minutes,
                site_candidates: self.farm.site_candidates,
            },
            build: BuildParams {
                programs,
                store_horizon_days: self.build.store_horizon_days,
                home_work_places: self.build.home_work_places,
                site_years: self.build.site_years,
                quality_spread: self.build.quality_spread,
                levelling: civ_agents::params::Levelling {
                    from_m: self.build.levelling.from_m,
                    most_m: self.build.levelling.most_m,
                    h_per_m3: self.build.levelling.h_per_m3,
                    side_run: self.build.levelling.side_run,
                },
                caution: civ_agents::caution::CautionParams {
                    half_life_years: self.build.caution.half_life_years,
                    most: self.build.caution.most,
                    half_rate: self.build.caution.half_rate,
                    death_weight: self.build.caution.death_weight,
                },
            },
            mortality: MortalityParams {
                siler: Siler {
                    a: m.a,
                    b: m.b,
                    c: m.c,
                    d: m.d,
                    e: m.e,
                },
                hunger_ratio_at_half: m.hunger_ratio_at_half,
                hunger_ratio_max: m.hunger_ratio_max,
                exhaustion_per_day: m.exhaustion_per_day,
                exhaustion_power: m.exhaustion_power,
                maternal_death_per_birth: m.maternal_death_per_birth,
                birth: civ_agents::params::BirthParams {
                    shares: [m.birth.bleeding, m.birth.infection, m.birth.obstructed],
                    fever_harm: m.birth.fever_harm,
                    fever_harm_days: m.birth.fever_harm_days,
                },
            },
            fertility: {
                let f = &self.fertility;
                FertilityParams {
                    conception_per_month: f.conception_per_month,
                    age_factor: pairs(&f.age_factor),
                    fecundity_sd: f.fecundity_sd,
                    hunger_halving: f.hunger_halving,
                    pregnancy_days: f.pregnancy_days,
                    pregnancy_sd_days: f.pregnancy_sd_days,
                    loss_by_age: pairs(&f.loss_by_age),
                    loss_days: f.loss_days,
                    recovery_months: f.recovery_months,
                    recovery_sd_months: f.recovery_sd_months,
                    recovery_min_months: f.recovery_min_months,
                    loss_recovery_months: f.loss_recovery_months,
                    weaned_recovery_months: f.weaned_recovery_months,
                    boys_per_100_girls: f.boys_per_100_girls,
                    pregnancy_kcal_day: f.pregnancy_kcal_day,
                }
            },
            family: {
                let y = &self.family;
                FamilyParams {
                    seek_min_age: [y.seek_min_age_female, y.seek_min_age_male],
                    seek_max_age: [y.seek_max_age_female, y.seek_max_age_male],
                    seek_per_month: [y.seek_per_month_female, y.seek_per_month_male],
                    age_gap_years: y.age_gap_years,
                    preferred_gap_years: y.preferred_gap_years,
                    w_gap_per_year: y.w_gap_per_year,
                    kin_exclusion_generations: y.kin_exclusion_generations,
                    residence: Residence::from_name(&y.residence)
                        .unwrap_or(Residence::NewHousehold),
                    independent_age: y.independent_age,
                    trait_heritability: y.trait_heritability,
                }
            },
            market: {
                let k = &self.market;
                MarketParams {
                    review_days: k.review_days,
                    margin: k.margin,
                    max_change: k.max_change,
                    stock_response: k.stock_response,
                    memory_days: k.memory_days,
                    money_share: k.money_share,
                    money_min_trades: k.money_min_trades,
                    accept_want: k.accept_want,
                    recent_trades: k.recent_trades as usize,
                    money_days: k.money_days,
                    strike_points: k.strike_points,
                    custody_fee_share: k.custody_fee_share,
                    note_coverage: k.note_coverage,
                }
            },
            firm: FirmParams {
                idle_close_days: self.firm.idle_close_days,
                book_entries: self.firm.book_entries as usize,
                wage_share: self.firm.wage_share,
                wage_review_days: self.firm.wage_review_days,
                wage_max_change: self.firm.wage_max_change,
                max_hire_hours: self.firm.max_hire_hours,
                shift_h: self.firm.shift_h,
            },
            knowledge: KnowledgeParams {
                founders,
                max_learners: self.knowledge.max_learners,
                w_learn: self.knowledge.w_learn,
                experiment_share: self.knowledge.experiment_share,
                aware_try_factor: self.knowledge.aware_try_factor,
                w_try: self.knowledge.w_try,
                try_gap_days: self.knowledge.try_gap_days,
                watch_m: self.knowledge.watch_m,
                record_h: self.knowledge.record_h,
            },
            digging: civ_agents::params::Digging {
                h_per_m3: self.digging.h_per_m3,
                pit_side_m: self.digging.pit_side_m,
                bail_w: self.digging.bail_w,
                bail_efficiency: self.digging.bail_efficiency,
                influence_m: self.digging.influence_m,
            },
            style: civ_agents::params::StyleParams {
                alpha: self.style.alpha,
                prestige_most: self.style.prestige_most,
                innovation: self.style.innovation,
                tradition_mean: self.style.tradition.taste(),
                tradition_spread: self.style.tradition_spread.taste(),
                personal_spread: self.style.personal_spread.taste(),
                seen_most: self.style.seen_most as usize,
                sight_m: self.style.sight_m,
            },
            midden: civ_agents::params::MiddenParams {
                kg_per_person_day: self.midden.kg_per_person_day,
                n_kg_per_person_year: self.midden.n_kg_per_person_year,
                half_life_days: self.midden.half_life_days,
                load_kg: self.midden.load_kg,
                spread_h_per_t: self.midden.spread_h_per_t,
            },
            ties: self.ties.params(),
            standing: civ_agents::standing::StandingParams {
                candidates: self.standing.candidates as usize,
                notable_share: self.standing.notable_share,
                notable_floor: self.standing.notable_floor as usize,
                notable_keep: self.standing.notable_keep,
            },
            polity: self.polity.params(),
            crime: self.crime.params(),
            word: self.word.params(),
            opinion: self.opinion.params(),
            faction: self.faction.params(),
            places: civ_agents::places::PlacesParams {
                sight_m: self.places.sight_m,
                share_told: self.places.share_told,
                w_kin: self.places.w_kin,
                w_ties: self.places.w_ties,
                w_seek: self.places.w_seek,
                seek_days: self.places.seek_days,
                revisit_days: self.places.revisit_days,
                use_half_life_days: self.places.use_half_life_days,
            },
            moving: civ_agents::places::MovingParams {
                w_kin: self.moving.w_kin,
                w_ties: self.moving.w_ties,
                w_fed: self.moving.w_fed,
                w_grievance: self.moving.w_grievance,
                w_stake: self.moving.w_stake,
                cost: self.moving.cost,
                reviews: self.moving.reviews,
                w_sickness: self.moving.w_sickness,
                w_raids: self.moving.w_raids,
                w_walk: self.moving.w_walk,
            },
            founding: civ_agents::places::FoundingParams {
                cost: self.founding.cost,
                yield_share: self.founding.yield_share,
                walk_hours: self.founding.walk_hours,
                candidates: self.founding.candidates,
                buffer_months: self.founding.buffer_months,
                work_h_per_day: self.founding.work_h_per_day,
            },
            reports: civ_agents::reports::ReportParams {
                half_life_days: self.reports.half_life_days,
                max_age_days: self.reports.max_age_days,
                share_told: self.reports.share_told,
            },
            suspicion: self.suspicion.params(),
            fire: self.fire.params(),
            levees: self.levees.params(),
            ditches: self.ditches.params(),
            rules: self.rules.params(),
            bands: self.bands.params(),
            enclosures: self.enclosures.params(),
            elections: self.elections.params(),
            credit: self.credit.params(),
            letting: self.letting.params(),
            climbing: self.climbing.params(),
            relations: civ_agents::views::RelationsParams {
                prior: self.relations.prior,
                half_life_days: self.relations.half_life_days,
                seen_trespass: self.relations.seen_trespass,
                heard_trespass: self.relations.heard_trespass,
                seen_taking: self.relations.seen_taking,
                heard_taking: self.relations.heard_taking,
                share_claims: self.relations.share_claims,
                claimed_worth: self.relations.claimed_worth,
                packages: self.relations.packages,
                answer_days: self.relations.answer_days,
                terms_days: self.relations.terms_days.clone(),
                gifts_kg: self.relations.gifts_kg.clone(),
                transfers_kg: self.relations.transfers_kg.clone(),
                transfer_days: self.relations.transfer_days,
                deliver_days: self.relations.deliver_days,
                carry_points: self.relations.carry_points,
                performance: self.relations.performance,
                raids_kept: self.relations.raids_kept,
            },
            names,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        positive("digging.h_per_m3", self.digging.h_per_m3, &mut p);
        positive("digging.bail_w", self.digging.bail_w, &mut p);
        positive("digging.influence_m", self.digging.influence_m, &mut p);
        if !(self.digging.bail_efficiency > 0.0 && self.digging.bail_efficiency <= 1.0) {
            p.push(format!(
                "`digging.bail_efficiency` must be above 0 and at most 1 (got {})",
                self.digging.bail_efficiency
            ));
        }
        self.ties.problems(&mut p);
        self.polity.problems(&mut p);
        self.crime.problems(&mut p);
        self.word.problems(&mut p);
        self.suspicion.problems(&mut p);
        self.fire.problems(&mut p);
        self.levees.problems(&mut p);
        self.ditches.problems(&mut p);
        self.rules.problems(&mut p);
        self.bands.problems(&mut p);
        self.enclosures.problems(&mut p);
        self.elections.problems(&mut p);
        self.credit.problems(&mut p);
        self.letting.problems(&mut p);
        self.climbing.problems(&mut p);
        if !(self.places.sight_m.is_finite() && (0.0..=10_000.0).contains(&self.places.sight_m)) {
            p.push(format!(
                "`places.sight_m` must be between 0 and 10000 (got {})",
                self.places.sight_m
            ));
        }
        if !(self.places.share_told.is_finite() && (0.0..=1.0).contains(&self.places.share_told)) {
            p.push(format!(
                "`places.share_told` must be between 0 and 1 (got {})",
                self.places.share_told
            ));
        }
        for (key, v) in [
            ("places.w_kin", self.places.w_kin),
            ("places.w_ties", self.places.w_ties),
            ("places.w_seek", self.places.w_seek),
        ] {
            if !(v.is_finite() && (0.0..=100.0).contains(&v)) {
                p.push(format!("`{key}` must be between 0 and 100 (got {v})"));
            }
        }
        if !(self.places.use_half_life_days.is_finite()
            && (1.0..=3650.0).contains(&self.places.use_half_life_days))
        {
            p.push(format!(
                "`places.use_half_life_days` must be between 1 and 3650 (got {})",
                self.places.use_half_life_days
            ));
        }
        if !(self.places.revisit_days.is_finite()
            && (1.0..=3650.0).contains(&self.places.revisit_days))
        {
            p.push(format!(
                "`places.revisit_days` must be between 1 and 3650 (got {})",
                self.places.revisit_days
            ));
        }
        for (key, v) in [
            ("moving.w_kin", self.moving.w_kin),
            ("moving.w_ties", self.moving.w_ties),
            ("moving.w_fed", self.moving.w_fed),
            ("moving.w_grievance", self.moving.w_grievance),
            ("moving.w_stake", self.moving.w_stake),
            ("moving.cost", self.moving.cost),
            ("moving.w_sickness", self.moving.w_sickness),
            ("moving.w_raids", self.moving.w_raids),
            ("moving.w_walk", self.moving.w_walk),
        ] {
            if !(v.is_finite() && (0.0..=100.0).contains(&v)) {
                p.push(format!("`{key}` must be between 0 and 100 (got {v})"));
            }
        }
        let f = &self.founding;
        for (key, v, lo, hi) in [
            ("founding.cost", f.cost, 0.0, 100.0),
            ("founding.yield_share", f.yield_share, 0.05, 1.0),
            ("founding.walk_hours", f.walk_hours, 0.25, 24.0),
            ("founding.buffer_months", f.buffer_months, 0.0, 24.0),
            ("founding.work_h_per_day", f.work_h_per_day, 0.5, 16.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{key}` must be between {lo} and {hi} (got {v})"));
            }
        }
        let r = &self.reports;
        if !(r.half_life_days.is_finite() && (1.0..=3650.0).contains(&r.half_life_days)) {
            p.push(format!(
                "`reports.half_life_days` must be between 1 and 3650 (got {})",
                r.half_life_days
            ));
        }
        if !(1..=3650).contains(&r.max_age_days) {
            p.push(format!(
                "`reports.max_age_days` must be between 1 and 3650 (got {})",
                r.max_age_days
            ));
        }
        if !(r.share_told.is_finite() && (0.0..=1.0).contains(&r.share_told)) {
            p.push(format!(
                "`reports.share_told` must be between 0 and 1 (got {})",
                r.share_told
            ));
        }
        let v = &self.relations;
        if !(v.prior.is_finite() && (0.01..=100.0).contains(&v.prior)) {
            p.push(format!(
                "`relations.prior` must be between 0.01 and 100 (got {})",
                v.prior
            ));
        }
        if !(v.half_life_days.is_finite() && (1.0..=36500.0).contains(&v.half_life_days)) {
            p.push(format!(
                "`relations.half_life_days` must be between 1 and 36500 (got {})",
                v.half_life_days
            ));
        }
        for (name, x) in [
            ("seen_trespass", v.seen_trespass),
            ("heard_trespass", v.heard_trespass),
            ("seen_taking", v.seen_taking),
            ("heard_taking", v.heard_taking),
        ] {
            if !(x.is_finite() && (0.0..=100.0).contains(&x)) {
                p.push(format!(
                    "`relations.{name}` must be between 0 and 100 (got {x})"
                ));
            }
        }
        for (name, x) in [
            ("share_claims", v.share_claims),
            ("claimed_worth", v.claimed_worth),
        ] {
            if !(x.is_finite() && (0.0..=1.0).contains(&x)) {
                p.push(format!(
                    "`relations.{name}` must be between 0 and 1 (got {x})"
                ));
            }
        }
        if !(1..=32).contains(&v.packages) {
            p.push(format!(
                "`relations.packages` must be between 1 and 32 (got {})",
                v.packages
            ));
        }
        if !(1..=3650).contains(&v.answer_days) {
            p.push(format!(
                "`relations.answer_days` must be between 1 and 3650 (got {})",
                v.answer_days
            ));
        }
        if v.terms_days.is_empty() || v.terms_days.iter().any(|&t| t > 36_500) {
            p.push(format!(
                "`relations.terms_days` must name at least one term of at most 36500 days (got \
                 {:?})",
                v.terms_days
            ));
        }
        for (key, list) in [("gifts_kg", &v.gifts_kg), ("transfers_kg", &v.transfers_kg)] {
            if list.len() > 8 || list.iter().any(|&kg| kg == 0 || kg > 100_000) {
                p.push(format!(
                    "`relations.{key}` must name at most 8 amounts of 1 to 100000 kg (got \
                     {list:?})"
                ));
            }
        }
        if !(1..=3650).contains(&v.transfer_days) {
            p.push(format!(
                "`relations.transfer_days` must be between 1 and 3650 (got {})",
                v.transfer_days
            ));
        }
        if !(1..=365).contains(&v.deliver_days) {
            p.push(format!(
                "`relations.deliver_days` must be between 1 and 365 (got {})",
                v.deliver_days
            ));
        }
        if !(v.carry_points.is_finite() && (0.0..=100.0).contains(&v.carry_points)) {
            p.push(format!(
                "`relations.carry_points` must be between 0 and 100 (got {})",
                v.carry_points
            ));
        }
        if !(v.performance.is_finite() && (0.0..=10.0).contains(&v.performance)) {
            p.push(format!(
                "`relations.performance` must be between 0 and 10 (got {})",
                v.performance
            ));
        }
        if !(v.raids_kept.is_finite() && (0.0..=1.0).contains(&v.raids_kept)) {
            p.push(format!(
                "`relations.raids_kept` must be between 0 and 1 (got {})",
                v.raids_kept
            ));
        }
        if !(1..=256).contains(&f.candidates) {
            p.push(format!(
                "`founding.candidates` must be between 1 and 256 (got {})",
                f.candidates
            ));
        }
        if !(1..=10).contains(&self.moving.reviews) {
            p.push(format!(
                "`moving.reviews` must be between 1 and 10 (got {})",
                self.moving.reviews
            ));
        }
        if !(0..=3650).contains(&self.places.seek_days) {
            p.push(format!(
                "`places.seek_days` must be between 0 and 3650 (got {})",
                self.places.seek_days
            ));
        }
        self.opinion.problems(&mut p);
        self.faction.problems(&mut p);
        let st = &self.standing;
        if !(1..=100).contains(&st.candidates) {
            p.push(format!(
                "`standing.candidates` must be between 1 and 100 (got {})",
                st.candidates
            ));
        }
        if st.notable_floor > 1000 {
            p.push(format!(
                "`standing.notable_floor` must be at most 1000 (got {})",
                st.notable_floor
            ));
        }
        for (name, v, lo, hi) in [
            ("standing.notable_share", st.notable_share, 0.0, 1.0),
            ("standing.notable_keep", st.notable_keep, 1.0, 10.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        let m = &self.midden;
        for (name, v, lo, hi) in [
            ("midden.kg_per_person_day", m.kg_per_person_day, 0.0, 10.0),
            (
                "midden.n_kg_per_person_year",
                m.n_kg_per_person_year,
                0.0,
                20.0,
            ),
            ("midden.half_life_days", m.half_life_days, 0.0, 36_500.0),
            ("midden.load_kg", m.load_kg, 1.0, 200.0),
            ("midden.spread_h_per_t", m.spread_h_per_t, 0.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        let st = &self.style;
        for (name, v, lo, hi) in [
            ("style.alpha", st.alpha, 0.0, 1.0),
            ("style.prestige_most", st.prestige_most, 1.0, 10.0),
            ("style.innovation", st.innovation, 0.0, 1.0),
            ("style.seen_most", f64::from(st.seen_most), 0.0, 64.0),
            ("style.sight_m", st.sight_m, 0.0, 2000.0),
            (
                "style.tradition.pitch_deg",
                st.tradition.pitch_deg,
                0.0,
                80.0,
            ),
            ("style.tradition.eave_m", st.tradition.eave_m, 0.5, 6.0),
            (
                "style.tradition.overhang_m",
                st.tradition.overhang_m,
                0.0,
                3.0,
            ),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        for (name, t) in [
            ("style.tradition_spread", st.tradition_spread),
            ("style.personal_spread", st.personal_spread),
        ] {
            for (field, v, most) in [
                ("pitch_deg", t.pitch_deg, 20.0),
                ("eave_m", t.eave_m, 1.0),
                ("overhang_m", t.overhang_m, 1.0),
            ] {
                if !(v.is_finite() && (0.0..=most).contains(&v)) {
                    p.push(format!(
                        "`{name}.{field}` must be between 0 and {most} (got {v})"
                    ));
                }
            }
        }
        if !(self.digging.pit_side_m.is_finite() && (1.0..=20.0).contains(&self.digging.pit_side_m))
        {
            p.push(format!(
                "`digging.pit_side_m` must be between 1 and 20 (got {})",
                self.digging.pit_side_m
            ));
        }
        for f in &self.knowledge.founders {
            if !(f.share.is_finite() && (0.0..=1.0).contains(&f.share)) {
                p.push(format!(
                    "`knowledge.founders` share of `{}` must be between 0 and 1 (got {})",
                    f.technique, f.share
                ));
            }
        }
        if !(1..=10).contains(&self.knowledge.max_learners) {
            p.push(format!(
                "`knowledge.max_learners` must be 1-10 (got {})",
                self.knowledge.max_learners
            ));
        }
        if !(self.knowledge.w_learn.is_finite() && self.knowledge.w_learn >= 0.0) {
            p.push(format!(
                "`knowledge.w_learn` must be zero or more (got {})",
                self.knowledge.w_learn
            ));
        }
        let k = &self.knowledge;
        if !(k.experiment_share.is_finite() && (0.0..=1.0).contains(&k.experiment_share)) {
            p.push(format!(
                "`knowledge.experiment_share` must be between 0 and 1 (got {})",
                k.experiment_share
            ));
        }
        if !(k.record_h.is_finite() && k.record_h > 0.0 && k.record_h <= 1_000.0) {
            p.push(format!(
                "`knowledge.record_h` must be above 0 and at most 1000 (got {})",
                k.record_h
            ));
        }
        if !(k.watch_m.is_finite() && (0.0..=1_000.0).contains(&k.watch_m)) {
            p.push(format!(
                "`knowledge.watch_m` must be between 0 and 1000 (got {})",
                k.watch_m
            ));
        }
        if !(k.aware_try_factor.is_finite() && k.aware_try_factor >= 1.0) {
            p.push(format!(
                "`knowledge.aware_try_factor` must be at least 1 (got {})",
                k.aware_try_factor
            ));
        }
        if !(k.w_try.is_finite() && k.w_try >= 0.0) {
            p.push(format!(
                "`knowledge.w_try` must be zero or more (got {})",
                k.w_try
            ));
        }
        if !(k.try_gap_days.is_finite() && k.try_gap_days >= 0.0) {
            p.push(format!(
                "`knowledge.try_gap_days` must be zero or more (got {})",
                k.try_gap_days
            ));
        }
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
        unit("walking.climb_factor", w.climb_factor, &mut p);
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
        unit(
            "energy.eat_reserve_at_deficit",
            e.eat_reserve_at_deficit,
            &mut p,
        );
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
        let mut last = f64::NEG_INFINITY;
        for &[m, l] in &h.water_use_by_walk_min {
            if !(m.is_finite() && m >= 0.0 && m > last) {
                p.push(
                    "`household.water_use_by_walk_min` needs minutes of 0 or more, rising"
                        .to_owned(),
                );
            }
            if !(l.is_finite() && (1.0..=100.0).contains(&l)) {
                p.push(format!(
                    "`household.water_use_by_walk_min` litres must be between 1 and 100 (got {l})"
                ));
            }
            last = m;
        }
        positive("household.water_target_days", h.water_target_days, &mut p);
        positive("household.food_target_days", h.food_target_days, &mut p);
        positive("household.ready_food_days", h.ready_food_days, &mut p);
        positive(
            "build.store_horizon_days",
            self.build.store_horizon_days,
            &mut p,
        );
        positive("build.site_years", self.build.site_years, &mut p);
        let [novice, master] = self.build.quality_spread;
        if !(master >= 0.0 && master <= novice && novice < 1.0) {
            p.push(format!(
                "`build.quality_spread` must be [novice, master] with 0 <= master <= novice < 1 \
                 (got [{novice}, {master}])"
            ));
        }
        let l = &self.build.levelling;
        if !(l.from_m.is_finite()
            && l.most_m.is_finite()
            && 0.0 < l.from_m
            && l.from_m <= l.most_m
            && l.most_m <= 20.0)
        {
            p.push(format!(
                "`build.levelling` needs 0 < from_m <= most_m <= 20 (got {} and {})",
                l.from_m, l.most_m
            ));
        }
        positive("build.levelling.h_per_m3", l.h_per_m3, &mut p);
        if !(l.side_run.is_finite() && (0.5..=10.0).contains(&l.side_run)) {
            p.push(format!(
                "`build.levelling.side_run` must be between 0.5 and 10 (got {})",
                l.side_run
            ));
        }
        let c = &self.build.caution;
        if !(c.half_life_years.is_finite() && (0.5..=100.0).contains(&c.half_life_years)) {
            p.push(format!(
                "`build.caution.half_life_years` must be between 0.5 and 100 (got {})",
                c.half_life_years
            ));
        }
        if !(c.most.is_finite() && (1.0..=10.0).contains(&c.most)) {
            p.push(format!(
                "`build.caution.most` must be between 1 and 10: builders never build weaker for \
                 what they have seen (got {})",
                c.most
            ));
        }
        positive("build.caution.half_rate", c.half_rate, &mut p);
        if !(c.death_weight.is_finite() && (0.0..=100.0).contains(&c.death_weight)) {
            p.push(format!(
                "`build.caution.death_weight` must be between 0 and 100 (got {})",
                c.death_weight
            ));
        }
        if !(h.raised_store_factor.is_finite() && h.raised_store_factor >= 1.0) {
            p.push("`household.raised_store_factor` must be 1 or more: a raised floor never keeps worse".to_owned());
        }
        if !(h.harvest_margin_days.is_finite() && (0.0..=365.0).contains(&h.harvest_margin_days)) {
            p.push("`household.harvest_margin_days` must be between 0 and 365".to_owned());
        }
        positive(
            "household.processed_food_days",
            h.processed_food_days,
            &mut p,
        );
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
        unit("household.leave_at_depletion", h.leave_at_depletion, &mut p);
        unit("household.leave_per_day", h.leave_per_day, &mut p);
        non_negative(
            "household.leave_unless_ripe_within_days",
            h.leave_unless_ripe_within_days,
            &mut p,
        );
        for (name, v, lo, hi) in [
            ("household.leave_w_gap", h.leave_w_gap, 0.0, 100.0),
            ("household.leave_w_stake", h.leave_w_stake, 0.0, 100.0),
            ("household.leave_stay", h.leave_stay, -100.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        let d = &self.decision;
        positive(
            "decision.temperature_sd_fraction",
            d.temperature_sd_fraction,
            &mut p,
        );
        positive("decision.min_temperature", d.min_temperature, &mut p);
        if !(d.max_hunger_drive.is_finite() && d.max_hunger_drive >= 1.0) {
            p.push(format!(
                "`decision.max_hunger_drive` must be at least 1, full hunger (got {})",
                d.max_hunger_drive
            ));
        }
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
            ("w_lean", d.w_lean),
            ("w_work", d.w_work),
            ("w_fuel", d.w_fuel),
            ("w_farm", d.w_farm),
            ("w_deadline", d.w_deadline),
            ("w_shelter", d.w_shelter),
            ("w_tools", d.w_tools),
            ("w_water", d.w_water),
            ("w_walk_hour", d.w_walk_hour),
            ("w_effort", d.w_effort),
            ("w_dark", d.w_dark),
            ("w_rest", d.w_rest),
            ("w_play", d.w_play),
            ("w_tend", d.w_tend),
            ("w_teach", d.w_teach),
            ("w_study", d.w_study),
            ("w_write", d.w_write),
            ("w_read", d.w_read),
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
        non_negative("band.seed_kg_per_person", b.seed_kg_per_person, &mut p);
        let f = &self.farm;
        unit("farm.grain_share", f.grain_share, &mut p);
        if !(f.plan_yield_share.is_finite()
            && f.plan_yield_share > 0.0
            && f.plan_yield_share <= 2.0)
        {
            p.push("`farm.plan_yield_share` must be above 0 and at most 2".to_owned());
        }
        if !(f.loss_share.is_finite() && (0.0..=0.5).contains(&f.loss_share)) {
            p.push("`farm.loss_share` must be between 0 and 0.5".to_owned());
        }
        positive("farm.grain_target_days", f.grain_target_days, &mut p);
        if !(f.work_hours_per_day.is_finite() && (0.5..=16.0).contains(&f.work_hours_per_day)) {
            p.push("`farm.work_hours_per_day` must be between 0.5 and 16".to_owned());
        }
        if !(f.peak_work_hours_per_day.is_finite()
            && (f.work_hours_per_day..=16.0).contains(&f.peak_work_hours_per_day))
        {
            p.push(
                "`farm.peak_work_hours_per_day` must be between `work_hours_per_day` and 16"
                    .to_owned(),
            );
        }
        if !(f.field_m.is_finite() && (10.0..=500.0).contains(&f.field_m)) {
            p.push("`farm.field_m` must be between 10 and 500 metres".to_owned());
        }
        positive("farm.max_walk_minutes", f.max_walk_minutes, &mut p);
        if f.site_candidates == 0 {
            p.push("`farm.site_candidates` must be at least 1".to_owned());
        }
        unit("band.elder_chance", b.elder_chance, &mut p);
        unit("band.young_adult_chance", b.young_adult_chance, &mut p);
        positive("band.birth_spacing_months", b.birth_spacing_months, &mut p);
        positive("band.site_max_slope", b.site_max_slope, &mut p);
        for (name, v) in [
            ("site_w_food", b.site_w_food),
            ("site_w_arable", b.site_w_arable),
            ("site_w_water_per_100m", b.site_w_water_per_100m),
            ("site_w_slope_per_pct", b.site_w_slope_per_pct),
            ("site_w_flood", b.site_w_flood),
            ("site_flood_hand_m", b.site_flood_hand_m),
        ] {
            non_negative(&format!("band.{name}"), v, &mut p);
        }
        let k = &self.market;
        if !(1..=90).contains(&k.review_days) {
            p.push("`market.review_days` must be between 1 and 90".to_owned());
        }
        non_negative("market.margin", k.margin, &mut p);
        unit("market.max_change", k.max_change, &mut p);
        if !(k.stock_response.is_finite() && (0.0..=3.0).contains(&k.stock_response)) {
            p.push(format!(
                "`market.stock_response` must be between 0 and 3 (got {})",
                k.stock_response
            ));
        }
        positive("market.memory_days", k.memory_days, &mut p);
        unit("market.money_share", k.money_share, &mut p);
        non_negative("market.money_min_trades", k.money_min_trades, &mut p);
        unit("market.accept_want", k.accept_want, &mut p);
        positive("market.money_days", k.money_days, &mut p);
        unit("market.custody_fee_share", k.custody_fee_share, &mut p);
        unit("market.note_coverage", k.note_coverage, &mut p);
        if !(k.strike_points.is_finite() && (0.0..=100.0).contains(&k.strike_points)) {
            p.push(format!(
                "`market.strike_points` must be between 0 and 100 (got {})",
                k.strike_points
            ));
        }
        if k.recent_trades == 0 || k.recent_trades > 1000 {
            p.push("`market.recent_trades` must be between 1 and 1000".to_owned());
        }
        positive("firm.idle_close_days", self.firm.idle_close_days, &mut p);
        if self.firm.book_entries == 0 || self.firm.book_entries > 10_000 {
            p.push("`firm.book_entries` must be between 1 and 10000".to_owned());
        }
        if !(1..=365).contains(&self.firm.wage_review_days) {
            p.push("`firm.wage_review_days` must be between 1 and 365".to_owned());
        }
        unit("firm.wage_share", self.firm.wage_share, &mut p);
        unit("firm.wage_max_change", self.firm.wage_max_change, &mut p);
        non_negative("firm.max_hire_hours", self.firm.max_hire_hours, &mut p);
        if !(self.firm.shift_h.is_finite() && (1.0..=16.0).contains(&self.firm.shift_h)) {
            p.push(format!(
                "`firm.shift_h` must be between 1 and 16 (got {})",
                self.firm.shift_h
            ));
        }
        let m = &self.mortality;
        for (name, v) in [("a", m.a), ("b", m.b), ("c", m.c), ("d", m.d), ("e", m.e)] {
            non_negative(&format!("mortality.{name}"), v, &mut p);
        }
        if m.b <= 0.0 || m.e <= 0.0 {
            p.push("`mortality.b` and `mortality.e` must be positive".to_owned());
        }
        if !(m.hunger_ratio_at_half.is_finite() && m.hunger_ratio_at_half >= 1.0) {
            p.push("`mortality.hunger_ratio_at_half` must be at least 1".to_owned());
        }
        if !(m.hunger_ratio_max.is_finite() && m.hunger_ratio_max >= m.hunger_ratio_at_half) {
            p.push(
                "`mortality.hunger_ratio_max` must be at least `mortality.hunger_ratio_at_half`"
                    .to_owned(),
            );
        }
        non_negative("mortality.exhaustion_per_day", m.exhaustion_per_day, &mut p);
        positive("mortality.exhaustion_power", m.exhaustion_power, &mut p);
        unit(
            "mortality.maternal_death_per_birth",
            m.maternal_death_per_birth,
            &mut p,
        );
        let b = &m.birth;
        for (name, v) in [
            ("bleeding", b.bleeding),
            ("infection", b.infection),
            ("obstructed", b.obstructed),
        ] {
            unit(&format!("mortality.birth.{name}"), v, &mut p);
        }
        if ((b.bleeding + b.infection + b.obstructed) - 1.0).abs() > 1e-6 {
            p.push("`mortality.birth`'s shares must sum to 1".to_owned());
        }
        if !(b.fever_harm.is_finite() && b.fever_harm >= 1.0) {
            p.push(format!(
                "`mortality.birth.fever_harm` must be at least 1 (got {})",
                b.fever_harm
            ));
        }
        if b.fever_harm_days < 0 {
            p.push("`mortality.birth.fever_harm_days` must not be negative".to_owned());
        }
        let f = &self.fertility;
        unit(
            "fertility.conception_per_month",
            f.conception_per_month,
            &mut p,
        );
        for (name, table) in [
            ("fertility.age_factor", &f.age_factor),
            ("fertility.loss_by_age", &f.loss_by_age),
        ] {
            ascending(name, table.iter().map(|x| x[0]), &mut p);
        }
        for &[_, v] in &f.age_factor {
            if !(v.is_finite() && (0.0..=2.0).contains(&v)) {
                p.push(format!(
                    "fertility age factors must be between 0 and 2 (got {v})"
                ));
            }
        }
        for &[_, v] in &f.loss_by_age {
            unit("fertility.loss_by_age", v, &mut p);
        }
        non_negative("fertility.fecundity_sd", f.fecundity_sd, &mut p);
        positive("fertility.hunger_halving", f.hunger_halving, &mut p);
        positive("fertility.pregnancy_days", f.pregnancy_days, &mut p);
        non_negative("fertility.pregnancy_sd_days", f.pregnancy_sd_days, &mut p);
        if !(f.loss_days[0].is_finite()
            && f.loss_days[0] >= 1.0
            && f.loss_days[1] >= f.loss_days[0]
            && f.loss_days[1] < f.pregnancy_days)
        {
            p.push(
                "`fertility.loss_days` must be [from, to] with 1 <= from <= to < pregnancy_days"
                    .to_owned(),
            );
        }
        positive("fertility.recovery_months", f.recovery_months, &mut p);
        non_negative("fertility.recovery_sd_months", f.recovery_sd_months, &mut p);
        non_negative(
            "fertility.recovery_min_months",
            f.recovery_min_months,
            &mut p,
        );
        non_negative(
            "fertility.loss_recovery_months",
            f.loss_recovery_months,
            &mut p,
        );
        non_negative(
            "fertility.weaned_recovery_months",
            f.weaned_recovery_months,
            &mut p,
        );
        non_negative("fertility.boys_per_100_girls", f.boys_per_100_girls, &mut p);
        for v in f.pregnancy_kcal_day {
            non_negative("fertility.pregnancy_kcal_day", v, &mut p);
        }
        let y = &self.family;
        for (name, lo, hi) in [
            ("female", y.seek_min_age_female, y.seek_max_age_female),
            ("male", y.seek_min_age_male, y.seek_max_age_male),
        ] {
            if !(lo.is_finite() && hi.is_finite() && lo >= 10.0 && hi > lo) {
                p.push(format!(
                    "`family.seek_min_age_{name}` must be at least 10 and below `family.seek_max_age_{name}`"
                ));
            }
        }
        unit(
            "family.seek_per_month_female",
            y.seek_per_month_female,
            &mut p,
        );
        unit("family.seek_per_month_male", y.seek_per_month_male, &mut p);
        if !(y.age_gap_years[0].is_finite() && y.age_gap_years[1] >= y.age_gap_years[0]) {
            p.push("`family.age_gap_years` must be [from, to] with from <= to".to_owned());
        }
        if !(y.age_gap_years[0]..=y.age_gap_years[1]).contains(&y.preferred_gap_years) {
            p.push(
                "`family.preferred_gap_years` must lie within `family.age_gap_years`".to_owned(),
            );
        }
        non_negative("family.w_gap_per_year", y.w_gap_per_year, &mut p);
        if y.kin_exclusion_generations > 6 {
            p.push("`family.kin_exclusion_generations` must be at most 6".to_owned());
        }
        if Residence::from_name(&y.residence).is_none() {
            let names: Vec<&str> = Residence::ALL.iter().map(|r| r.name()).collect();
            p.push(format!(
                "`family.residence` must be one of {} (got `{}`)",
                names.join(", "),
                y.residence
            ));
        }
        if !(y.independent_age.is_finite() && (10.0..=25.0).contains(&y.independent_age)) {
            p.push("`family.independent_age` must be between 10 and 25".to_owned());
        }
        unit("family.trait_heritability", y.trait_heritability, &mut p);
        p
    }
}
