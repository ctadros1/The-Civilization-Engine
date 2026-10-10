//! Diseases (`kind = "disease"`, content API 72-73, M6a slice AZ; ADR-0021 §5; research 05-03 §1.1,
//! §2.1, §3.2, §7.3, §7.6): the routes one passes by, its own clocks, who has symptoms and who
//! is severely ill, the chance a day of dying while severely ill, how long an infection protects,
//! the household hazard, and (API 73) what one sheds a day and how fast it dies outside a person.
//! No rate of cases or deaths is authored: those come from who meets it.

use civ_agents::params::{Days, DiseaseDef, DiseaseRoute};
use serde::Deserialize;

/// The `kind` value of a disease.
pub const KIND: &str = "disease";
/// The id segment: `pack:disease/name`.
pub const ID_KIND: &str = "disease";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DiseaseFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// The routes it passes by: `water`, `household`.
    pub routes: Vec<String>,
    /// Durations as `[mean, sd]` days (lognormal; an sd of 0 is the mean exactly).
    pub incubation_days: [f64; 2],
    #[serde(default)]
    pub shed_lead_days: f64,
    pub shed_days: [f64; 2],
    #[serde(default)]
    pub shed_after_days: [f64; 2],
    pub symptomatic: f64,
    pub ill_days: [f64; 2],
    /// `[[age, share], ...]`, ages rising.
    pub severe_by_age: Vec<[f64; 2]>,
    pub severe_death_per_day: f64,
    /// `[least, most]` years.
    pub immunity_years: [f64; 2],
    #[serde(default)]
    pub household_hazard: f64,
    pub shed_ill_per_day: f64,
    pub shed_silent_per_day: f64,
    pub decay_per_day: f64,
}

fn days([mean, sd]: [f64; 2]) -> Days {
    Days { mean, sd }
}

impl DiseaseFile {
    /// The compiled disease; `None` when a route is unknown (reported by [`Self::problems`]).
    pub fn def(&self) -> Option<DiseaseDef> {
        let routes = self
            .routes
            .iter()
            .map(|r| DiseaseRoute::from_key(r))
            .collect::<Option<Vec<_>>>()?;
        Some(DiseaseDef {
            id: self.id.clone(),
            name: self.name.clone(),
            routes,
            incubation: days(self.incubation_days),
            shed_lead_days: self.shed_lead_days,
            shed: days(self.shed_days),
            shed_after: days(self.shed_after_days),
            symptomatic: self.symptomatic,
            ill: days(self.ill_days),
            severe_by_age: self.severe_by_age.iter().map(|[a, s]| (*a, *s)).collect(),
            severe_death_per_day: self.severe_death_per_day,
            immunity_years: self.immunity_years,
            household_hazard: self.household_hazard,
            shed_ill_per_day: self.shed_ill_per_day,
            shed_silent_per_day: self.shed_silent_per_day,
            decay_per_day: self.decay_per_day,
        })
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if self.routes.is_empty() {
            p.push("`routes` must name at least one route".to_owned());
        }
        for r in &self.routes {
            if DiseaseRoute::from_key(r).is_none() {
                p.push(format!("unknown route `{r}`: one of `water`, `household`"));
            }
        }
        for (name, [mean, sd], lo) in [
            ("incubation_days", self.incubation_days, 0.1),
            ("shed_days", self.shed_days, 0.1),
            ("shed_after_days", self.shed_after_days, 0.0),
            ("ill_days", self.ill_days, 0.1),
        ] {
            if !(mean.is_finite() && (lo..=365.0).contains(&mean)) {
                p.push(format!(
                    "`{name}`'s mean must be between {lo} and 365 (got {mean})"
                ));
            }
            if !(sd.is_finite() && (0.0..=365.0).contains(&sd)) {
                p.push(format!(
                    "`{name}`'s sd must be between 0 and 365 (got {sd})"
                ));
            }
        }
        for (name, v, lo, hi) in [
            ("shed_lead_days", self.shed_lead_days, 0.0, 60.0),
            ("symptomatic", self.symptomatic, 0.0, 1.0),
            ("severe_death_per_day", self.severe_death_per_day, 0.0, 1.0),
            ("household_hazard", self.household_hazard, 0.0, 10.0),
            ("shed_ill_per_day", self.shed_ill_per_day, 0.0, 1e12),
            ("shed_silent_per_day", self.shed_silent_per_day, 0.0, 1e12),
            ("decay_per_day", self.decay_per_day, 0.0, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        let [lo, hi] = self.immunity_years;
        if !(lo.is_finite() && hi.is_finite() && 0.0 <= lo && lo <= hi && hi <= 200.0) {
            p.push(format!(
                "`immunity_years` must be [least, most] within 0 to 200 (got [{lo}, {hi}])"
            ));
        }
        if self.severe_by_age.is_empty() {
            p.push("`severe_by_age` must give at least one [age, share]".to_owned());
        }
        let mut last = f64::NEG_INFINITY;
        for [age, share] in &self.severe_by_age {
            if !(age.is_finite() && *age > last && (0.0..=120.0).contains(age)) {
                p.push(format!(
                    "`severe_by_age`'s ages must rise within 0 to 120 (got {age})"
                ));
            }
            if !(share.is_finite() && (0.0..=1.0).contains(share)) {
                p.push(format!(
                    "`severe_by_age`'s shares must be between 0 and 1 (got {share})"
                ));
            }
            last = *age;
        }
        p
    }
}
