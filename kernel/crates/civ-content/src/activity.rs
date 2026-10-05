//! Activities (`kind = "activity"`): a behavior the engine can carry out, with its numbers.
//! Which activity anyone does, and where, is decided by people at run time (ADR-0003).

use civ_agents::params::{ActivityDef, Behavior};
use civ_land::FieldTask;
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
    /// For farming: the field task (`prepare`, `sow`, `tend`, `reap`, `thresh`).
    pub task: Option<String>,
    /// For making: the recipe id.
    pub recipe: Option<String>,
    /// For digging: the good dug, from the deposits of it the land profile lays down (content API
    /// 20).
    pub digs: Option<String>,
    /// Tools the work needs and wears: good ids (a recipe's own tools are on the recipe).
    pub tools: Vec<String>,
    /// Work done in an hour against the task's authored rates (1 with the tools they assume).
    pub rate: f64,
    pub par: f64,
    pub min_age_years: f64,
    pub max_age_years: f64,
    pub min_minutes: u32,
    pub max_minutes: u32,
    pub daylight_only: bool,
    pub max_walk_minutes: u32,
    /// The technique the work needs: a technique id, or "" for none (a `make` activity's is
    /// its recipe's).
    pub technique: String,
}

fn behavior_names() -> String {
    Behavior::ALL
        .iter()
        .map(|b| b.name())
        .collect::<Vec<_>>()
        .join(", ")
}

impl ActivityFile {
    /// The compiled activity, with its resource resolved to an index in the land profile, its
    /// recipe to an index in the recipes, its tools to indexes in the goods and its technique to
    /// an index in the techniques.
    pub fn def(
        &self,
        resource: Option<usize>,
        recipe: Option<usize>,
        digs: Option<usize>,
        tools: Vec<usize>,
        technique: Option<usize>,
    ) -> Option<ActivityDef> {
        Some(ActivityDef {
            id: self.id.clone(),
            name: self.name.clone(),
            doing: self.doing.clone(),
            behavior: Behavior::from_name(&self.behavior)?,
            resource,
            task: self.task.as_deref().and_then(FieldTask::from_name),
            recipe,
            digs,
            tools,
            rate: self.rate,
            par: self.par,
            min_age_years: self.min_age_years,
            max_age_years: self.max_age_years,
            min_minutes: self.min_minutes,
            max_minutes: self.max_minutes,
            daylight_only: self.daylight_only,
            max_walk_minutes: self.max_walk_minutes,
            technique,
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
        match (Behavior::from_name(&self.behavior), &self.task) {
            (Some(Behavior::Farm), None) => {
                p.push("a `farm` activity names the field `task` it does".to_owned());
            }
            (Some(Behavior::Farm), Some(t)) if FieldTask::from_name(t).is_none() => {
                p.push(format!(
                    "unknown task `{t}` (known: {})",
                    FieldTask::ALL.map(FieldTask::name).join(", ")
                ));
            }
            (Some(b), Some(_)) if b != Behavior::Farm => p.push(format!(
                "only `farm` activities take a `task` (this one is `{}`)",
                b.name()
            )),
            _ => {}
        }
        match (Behavior::from_name(&self.behavior), &self.digs) {
            (Some(Behavior::Dig), None) => {
                p.push("a `dig` activity names the good it `digs`".to_owned());
            }
            (Some(b), Some(_)) if b != Behavior::Dig => p.push(format!(
                "only `dig` activities take `digs` (this one is `{}`)",
                b.name()
            )),
            _ => {}
        }
        match (Behavior::from_name(&self.behavior), &self.recipe) {
            (Some(Behavior::Make), None) => {
                p.push("a `make` activity names the `recipe` it works".to_owned());
            }
            (Some(b), Some(_)) if b != Behavior::Make => p.push(format!(
                "only `make` activities take a `recipe` (this one is `{}`)",
                b.name()
            )),
            _ => {}
        }
        if Behavior::from_name(&self.behavior) == Some(Behavior::Make) && !self.technique.is_empty()
        {
            p.push(
                "a `make` activity's technique is its recipe's: `technique` must be empty"
                    .to_owned(),
            );
        }
        if Behavior::from_name(&self.behavior) == Some(Behavior::Build)
            && !self.technique.is_empty()
        {
            p.push(
                "building work needs its program's technique: a `build` activity's `technique` \
                 must be empty"
                    .to_owned(),
            );
        }
        if Behavior::from_name(&self.behavior) == Some(Behavior::Try) && !self.technique.is_empty()
        {
            p.push(
                "what a `try` activity works toward is chosen when it is done: `technique` must be \
                 empty"
                    .to_owned(),
            );
        }
        if Behavior::from_name(&self.behavior) == Some(Behavior::Make) && !self.tools.is_empty() {
            p.push("a `make` activity's tools are its recipe's: `tools` must be empty".to_owned());
        }
        if !(self.rate.is_finite() && self.rate > 0.0 && self.rate <= 4.0) {
            p.push(format!(
                "`rate` must be above 0 and at most 4 (got {})",
                self.rate
            ));
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
