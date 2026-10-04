//! Goods (`kind = "good"`): things people carry home and keep, with how they keep. What anyone
//! gathers, stores or eats is decided by people at run time.

use civ_agents::params::{Eaten, GoodDef, GoodUse, ToolDef};
use serde::Deserialize;

/// The `kind` value of a good.
pub const KIND: &str = "good";
/// The id segment: `pack:good/name`.
pub const ID_KIND: &str = "good";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GoodFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// `food`, `fuel`, `material` or `tool`.
    pub purpose: String,
    pub kcal_per_kg: f64,
    pub half_life_days: f64,
    /// The half-life under a roof; 0 when a roof makes no difference.
    pub sheltered_half_life_days: f64,
    /// `raw`, `cooked` or `never` (a recipe must make it into food first).
    pub eaten: String,
    pub shared: bool,
    /// The good it is kept back from, like seed from grain (used in its place only in hunger),
    /// or "" for none.
    pub reserve_for: String,
    /// For a tool: its life and how many a household wants.
    pub tool: Option<Tool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Tool {
    pub life_h: f64,
    pub per_worker: f64,
    pub fixed: bool,
}

impl GoodFile {
    /// The compiled good (`None` if its purpose or way of eating is unknown, which
    /// [`GoodFile::problems`] reports). Its `reserve_for` is resolved later, once every good has
    /// its index.
    pub fn def(&self) -> Option<GoodDef> {
        Some(GoodDef {
            id: self.id.clone(),
            name: self.name.clone(),
            purpose: GoodUse::from_name(&self.purpose)?,
            kcal_per_kg: self.kcal_per_kg,
            half_life_days: self.half_life_days,
            sheltered_half_life_days: self.sheltered_half_life_days,
            eaten: Eaten::from_name(&self.eaten)?,
            shared: self.shared,
            reserve_for: None,
            tool: self.tool.as_ref().map(|t| ToolDef {
                life_h: t.life_h,
                per_worker: t.per_worker,
                fixed: t.fixed,
            }),
        })
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if !(self.half_life_days.is_finite() && self.half_life_days >= 0.0) {
            p.push(format!(
                "`half_life_days` must be zero (keeps) or more (got {})",
                self.half_life_days
            ));
        }
        let sheltered = self.sheltered_half_life_days;
        if !(sheltered.is_finite() && sheltered >= 0.0) {
            p.push(format!(
                "`sheltered_half_life_days` must be zero (no difference) or more (got {sheltered})"
            ));
        } else if sheltered > 0.0 && (self.half_life_days <= 0.0 || sheltered < self.half_life_days)
        {
            p.push(format!(
                "a roof never makes a good spoil faster: `sheltered_half_life_days` ({sheltered}) \
                 must be at least `half_life_days` ({}), or 0, and a good that keeps needs none",
                self.half_life_days
            ));
        }
        let eaten = Eaten::from_name(&self.eaten);
        if eaten.is_none() {
            p.push(format!(
                "unknown `eaten` `{}` (known: raw, cooked, never)",
                self.eaten
            ));
        }
        let purpose = GoodUse::from_name(&self.purpose);
        if purpose != Some(GoodUse::Food) {
            if eaten.is_some_and(|e| e != Eaten::Never) {
                p.push("only food is eaten: `eaten` must be \"never\"".to_owned());
            }
            if !self.reserve_for.is_empty() {
                p.push("only food is kept back: `reserve_for` must be \"\"".to_owned());
            }
        }
        match (purpose, &self.tool) {
            (Some(GoodUse::Tool), None) => {
                p.push("a tool needs its `[tool]` table".to_owned());
            }
            (Some(GoodUse::Tool), Some(t)) => {
                if !(t.life_h.is_finite() && t.life_h > 0.0) {
                    p.push(format!("`tool.life_h` must be positive (got {})", t.life_h));
                }
                if !(t.per_worker.is_finite() && (0.0..=10.0).contains(&t.per_worker)) {
                    p.push(format!(
                        "`tool.per_worker` must be between 0 and 10 (got {})",
                        t.per_worker
                    ));
                }
            }
            (Some(_), Some(_)) => p.push("only a tool takes a `[tool]` table".to_owned()),
            _ => {}
        }
        match purpose {
            None => p.push(format!(
                "unknown purpose `{}` (known: food, fuel, material, tool)",
                self.purpose
            )),
            Some(GoodUse::Food) => {
                if !(self.kcal_per_kg.is_finite() && self.kcal_per_kg > 0.0) {
                    p.push(format!(
                        "a food needs positive `kcal_per_kg` (got {})",
                        self.kcal_per_kg
                    ));
                }
            }
            Some(GoodUse::Fuel) => {
                if self.kcal_per_kg != 0.0 {
                    p.push("a fuel is not eaten: `kcal_per_kg` must be 0".to_owned());
                }
                // Burning is settled exactly for any split of time only if fuel does not also
                // decay (crate `civ-agents`, `Household::stores_at_time`).
                if self.half_life_days != 0.0 {
                    p.push("a fuel keeps: `half_life_days` must be 0".to_owned());
                }
                if sheltered != 0.0 {
                    p.push("a fuel keeps: `sheltered_half_life_days` must be 0".to_owned());
                }
            }
            Some(GoodUse::Material | GoodUse::Tool) => {
                if self.kcal_per_kg != 0.0 {
                    p.push(format!(
                        "a {} is not eaten: `kcal_per_kg` must be 0",
                        self.purpose
                    ));
                }
                // A household brings what it builds and works with for itself.
                if self.shared {
                    p.push(format!(
                        "a {} is the household's own: `shared` must be false",
                        self.purpose
                    ));
                }
            }
        }
        p
    }
}
