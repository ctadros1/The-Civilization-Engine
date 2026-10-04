//! Techniques (`kind = "technique"`): practical capabilities people know, learn and can lose
//! (ADR-0008 §1). What a technique gates is named by the work: recipes, activities and building
//! programs name the technique they need. Who knows what, and how it spreads, is decided by
//! people at run time.

use civ_agents::params::TechniqueDef;
use serde::Deserialize;

/// The `kind` value of a technique.
pub const KIND: &str = "technique";
/// The id segment: `pack:technique/name`.
pub const ID_KIND: &str = "technique";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TechniqueFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// What a competent person can do, completing "A competent person can …".
    pub can: String,
    /// The skill whose practice it is: a skill id, or "" for none.
    pub domain: String,
    /// Prerequisites: alternative routes, each a list of technique ids; empty for none.
    pub requires: Vec<Vec<String>>,
    /// Activities whose practice counts toward finding it: activity ids.
    pub tried_in: Vec<String>,
    /// Goods a household must hold to try it: good ids.
    pub needs: Vec<String>,
    /// Qualified hours of experiment to a median find.
    pub e50_h: f64,
    /// Hours of work beside someone who knows it that teach it.
    pub learn_h: f64,
    /// Children brought up in a household that knows it learn it at the work's age.
    pub upbringing: bool,
}

impl TechniqueFile {
    /// The compiled technique, with its references resolved (`None` if one is unknown, which
    /// the cross-file check reports).
    pub fn def(
        &self,
        skill_index: &dyn Fn(&str) -> Option<usize>,
        technique_index: &dyn Fn(&str) -> Option<usize>,
        activity_index: &dyn Fn(&str) -> Option<usize>,
        good_index: &dyn Fn(&str) -> Option<usize>,
    ) -> Option<TechniqueDef> {
        let domain = if self.domain.is_empty() {
            None
        } else {
            Some(skill_index(&self.domain)?)
        };
        Some(TechniqueDef {
            id: self.id.clone(),
            name: self.name.clone(),
            can: self.can.clone(),
            domain,
            requires: self
                .requires
                .iter()
                .map(|route| route.iter().map(|t| technique_index(t)).collect())
                .collect::<Option<Vec<Vec<usize>>>>()?,
            tried_in: self
                .tried_in
                .iter()
                .map(|a| activity_index(a))
                .collect::<Option<Vec<_>>>()?,
            needs: self
                .needs
                .iter()
                .map(|g| good_index(g))
                .collect::<Option<Vec<_>>>()?,
            e50_h: self.e50_h,
            learn_h: self.learn_h,
            upbringing: self.upbringing,
        })
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if self.can.trim().is_empty() {
            p.push("`can` says what a competent person can do; it must not be empty".to_owned());
        }
        if !(self.e50_h.is_finite() && self.e50_h > 0.0) {
            p.push(format!("`e50_h` must be positive (got {})", self.e50_h));
        }
        if !(self.learn_h.is_finite() && self.learn_h > 0.0) {
            p.push(format!("`learn_h` must be positive (got {})", self.learn_h));
        }
        if self.requires.iter().any(Vec::is_empty) {
            p.push(
                "every route in `requires` names at least one technique (leave `requires` empty \
                 for none)"
                    .to_owned(),
            );
        }
        if self.requires.iter().flatten().any(|t| *t == self.id) {
            p.push("a technique cannot require itself".to_owned());
        }
        p
    }
}
