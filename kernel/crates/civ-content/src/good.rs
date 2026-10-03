//! Goods (`kind = "good"`): things people carry home and keep, with how they keep. What anyone
//! gathers, stores or eats is decided by people at run time.

use civ_agents::params::{GoodDef, GoodUse};
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
    /// `food`, `fuel` or `material`.
    pub purpose: String,
    pub kcal_per_kg: f64,
    pub half_life_days: f64,
    /// The half-life under a roof; 0 when a roof makes no difference.
    pub sheltered_half_life_days: f64,
    pub cooked: bool,
    pub shared: bool,
    /// Kept back, like seed: eaten only when no other food is left.
    pub reserve: bool,
}

impl GoodFile {
    /// The compiled good (`None` if its purpose is unknown, which [`GoodFile::problems`] reports).
    pub fn def(&self) -> Option<GoodDef> {
        Some(GoodDef {
            id: self.id.clone(),
            name: self.name.clone(),
            purpose: GoodUse::from_name(&self.purpose)?,
            kcal_per_kg: self.kcal_per_kg,
            half_life_days: self.half_life_days,
            sheltered_half_life_days: self.sheltered_half_life_days,
            cooked: self.cooked,
            shared: self.shared,
            reserve: self.reserve,
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
        match GoodUse::from_name(&self.purpose) {
            None => p.push(format!(
                "unknown purpose `{}` (known: food, fuel, material)",
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
                if self.cooked {
                    p.push("only food can need cooking: `cooked` must be false".to_owned());
                }
                if self.reserve {
                    p.push("only food is kept back: `reserve` must be false".to_owned());
                }
                if sheltered != 0.0 {
                    p.push("a fuel keeps: `sheltered_half_life_days` must be 0".to_owned());
                }
            }
            Some(GoodUse::Material) => {
                if self.kcal_per_kg != 0.0 {
                    p.push("a material is not eaten: `kcal_per_kg` must be 0".to_owned());
                }
                if self.cooked {
                    p.push("only food can need cooking: `cooked` must be false".to_owned());
                }
                if self.reserve {
                    p.push("only food is kept back: `reserve` must be false".to_owned());
                }
                // A household brings what it builds with for its own home.
                if self.shared {
                    p.push("a material is the household's own: `shared` must be false".to_owned());
                }
            }
        }
        p
    }
}
