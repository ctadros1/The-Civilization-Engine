//! Activities (`kind = "activity"`): a behavior the engine can carry out, with its numbers.
//! Which activity anyone does, and where, is decided by people at run time (ADR-0003).

use civ_agents::params::{ActivityDef, Behavior};
use serde::Deserialize;

/// The `kind` value of an activity.
pub const KIND: &str = "activity";
/// The id segment: `pack:activity/name`.
pub const ID_KIND: &str = "activity";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActivityFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub doing: String,
    pub behavior: String,
    /// For gathering: the land profile's resource id.
    pub resource: Option<String>,
    pub par: f64,
    pub min_age_years: f64,
    pub max_age_years: f64,
    pub min_minutes: u32,
    pub max_minutes: u32,
    pub daylight_only: bool,
    pub max_walk_minutes: u32,
}

fn behavior_names() -> String {
    Behavior::ALL
        .iter()
        .map(|b| b.name())
        .collect::<Vec<_>>()
        .join(", ")
}

impl ActivityFile {
    /// The compiled activity, with its resource resolved to an index in the land profile.
    pub fn def(&self, resource: Option<usize>) -> Option<ActivityDef> {
        Some(ActivityDef {
            id: self.id.clone(),
            name: self.name.clone(),
            doing: self.doing.clone(),
            behavior: Behavior::from_name(&self.behavior)?,
            resource,
            par: self.par,
            min_age_years: self.min_age_years,
            max_age_years: self.max_age_years,
            min_minutes: self.min_minutes,
            max_minutes: self.max_minutes,
            daylight_only: self.daylight_only,
            max_walk_minutes: self.max_walk_minutes,
        })
    }

    /// Range problems, as messages. Resource ids are checked against the land profile later.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        match Behavior::from_name(&self.behavior) {
            None => p.push(format!(
                "unknown behavior `{}` (known: {})",
                self.behavior,
                behavior_names()
            )),
            Some(Behavior::Gather) if self.resource.is_none() => {
                p.push("a `gather` activity names the `resource` it gathers".to_owned());
            }
            Some(b) if b != Behavior::Gather && self.resource.is_some() => p.push(format!(
                "only `gather` activities take a `resource` (this one is `{}`)",
                b.name()
            )),
            Some(_) => {}
        }
        if !(self.par.is_finite() && (1.0..=10.0).contains(&self.par)) {
            p.push(format!("`par` must be between 1 and 10 (got {})", self.par));
        }
        if !(self.min_age_years.is_finite()
            && self.max_age_years.is_finite()
            && 0.0 <= self.min_age_years
            && self.min_age_years <= self.max_age_years)
        {
            p.push("ages must satisfy 0 <= min_age_years <= max_age_years".to_owned());
        }
        if !(1..=self.max_minutes).contains(&self.min_minutes) || self.max_minutes > 24 * 60 {
            p.push("minutes must satisfy 1 <= min_minutes <= max_minutes <= 1440".to_owned());
        }
        if self.max_walk_minutes > 24 * 60 {
            p.push("`max_walk_minutes` must be at most 1440".to_owned());
        }
        p
    }
}
