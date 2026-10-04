//! Recipes (`kind = "recipe"`): what goes in, what comes out, the work and the tools (ADR-0006
//! §2). Who works a recipe, and when, is decided by people at run time.

use civ_agents::params::RecipeDef;
use serde::Deserialize;

/// The `kind` value of a recipe.
pub const KIND: &str = "recipe";
/// The id segment: `pack:recipe/name`.
pub const ID_KIND: &str = "recipe";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// The skill it uses and trains: a skill id, or "" for none.
    pub skill: String,
    /// The technique working it needs: a technique id, or "" for none (ADR-0008 §1).
    pub technique: String,
    /// Labour per unit, hours of a capable adult of middling skill.
    pub unit_h: f64,
    /// Labour per session, hours.
    pub session_h: f64,
    /// Most units one session makes; 0 for no limit beyond time and inputs.
    pub max_units: f64,
    /// Tools it needs and wears: good ids.
    pub tools: Vec<String>,
    /// Per unit made.
    pub inputs: Vec<Amount>,
    pub outputs: Vec<Amount>,
    /// Per session, whatever its size.
    pub session_inputs: Vec<Amount>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Amount {
    /// A good id.
    pub good: String,
    /// In the good's unit: kilograms, or standard tools.
    pub amount: f64,
}

impl RecipeFile {
    /// Every good id the recipe names, with the field it is in.
    pub fn goods(&self) -> Vec<(&'static str, &str)> {
        let mut out = Vec::new();
        out.extend(self.tools.iter().map(|g| ("tools", g.as_str())));
        out.extend(self.inputs.iter().map(|a| ("inputs", a.good.as_str())));
        out.extend(self.outputs.iter().map(|a| ("outputs", a.good.as_str())));
        out.extend(
            self.session_inputs
                .iter()
                .map(|a| ("session_inputs", a.good.as_str())),
        );
        out
    }

    /// The compiled recipe, with goods, its skill and its technique resolved (`None` if one is
    /// unknown, which the cross-file check reports).
    pub fn def(
        &self,
        good_index: &dyn Fn(&str) -> Option<usize>,
        skill_index: &dyn Fn(&str) -> Option<usize>,
        technique: Option<usize>,
    ) -> Option<RecipeDef> {
        let amounts = |list: &[Amount]| -> Option<Vec<(usize, f64)>> {
            list.iter()
                .map(|a| good_index(&a.good).map(|g| (g, a.amount)))
                .collect()
        };
        let skill = if self.skill.is_empty() {
            None
        } else {
            Some(skill_index(&self.skill)?)
        };
        Some(RecipeDef {
            id: self.id.clone(),
            name: self.name.clone(),
            inputs: amounts(&self.inputs)?,
            outputs: amounts(&self.outputs)?,
            session_inputs: amounts(&self.session_inputs)?,
            unit_h: self.unit_h,
            session_h: self.session_h,
            max_units: self.max_units,
            tools: self
                .tools
                .iter()
                .map(|t| good_index(t))
                .collect::<Option<Vec<_>>>()?,
            skill,
            technique,
        })
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for (name, v) in [
            ("unit_h", self.unit_h),
            ("session_h", self.session_h),
            ("max_units", self.max_units),
        ] {
            if !(v.is_finite() && v >= 0.0) {
                p.push(format!("`{name}` must be zero or more (got {v})"));
            }
        }
        if self.unit_h <= 0.0 && self.session_h <= 0.0 {
            p.push("a recipe takes some work: `unit_h` or `session_h` must be positive".to_owned());
        }
        if self.outputs.is_empty() {
            p.push("a recipe makes something: `outputs` must not be empty".to_owned());
        }
        for (field, list) in [
            ("inputs", &self.inputs),
            ("outputs", &self.outputs),
            ("session_inputs", &self.session_inputs),
        ] {
            for a in list {
                if !(a.amount.is_finite() && a.amount > 0.0) {
                    p.push(format!(
                        "`{field}` amounts must be positive (`{}` has {})",
                        a.good, a.amount
                    ));
                }
            }
        }
        p
    }
}
