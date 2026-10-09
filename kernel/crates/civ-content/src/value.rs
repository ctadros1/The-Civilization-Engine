//! Values (`kind = "value"`, content API 43, ADR-0016 §4): a slow axis on which people weigh what
//! a law does beyond their household's own food (research 06-04 §1.1), what holding it more or
//! less than most means in words, and the priors each person's is drawn from. Policy templates
//! say in `[bears]` how a law of their kind bears on each. Who holds what, and what it moves, is
//! decided at run time.

use civ_agents::values::ValueDef;
use serde::Deserialize;

/// The `kind` value of a value.
pub const KIND: &str = "value";
/// The id segment: `pack:value/name`.
pub const ID_KIND: &str = "value";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ValueFile {
    pub kind: String,
    pub id: String,
    /// What it is, to follow "cares for": "safety from want and harm".
    pub name: String,
    pub description: String,
    /// Holding it more than most, and less, in words: "holds safety from want and harm dear".
    pub high: String,
    pub low: String,
    /// A founder's, before it is squashed into −1 to 1: the mean and spread, and how much of its
    /// parents' mean a child takes on.
    pub mean: f64,
    pub sd: f64,
    pub heritability: f64,
    /// Points a law that bears fully on it adds for one who holds it fully.
    pub weight: f64,
}

impl ValueFile {
    /// The compiled value (call after [`ValueFile::problems`] found none).
    pub fn def(&self) -> ValueDef {
        ValueDef {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            high: self.high.clone(),
            low: self.low.clone(),
            mean: self.mean,
            sd: self.sd,
            heritability: self.heritability,
            weight: self.weight,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for (key, text) in [
            ("name", &self.name),
            ("high", &self.high),
            ("low", &self.low),
        ] {
            if text.trim().is_empty() {
                p.push(format!("`{key}` must say something"));
            }
        }
        for (name, v, lo, hi) in [
            ("mean", self.mean, -5.0, 5.0),
            ("sd", self.sd, 0.0, 5.0),
            ("heritability", self.heritability, 0.0, 1.0),
            ("weight", self.weight, 0.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        p
    }
}
