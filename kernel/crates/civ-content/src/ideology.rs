//! Ideologies (`kind = "ideology"`, content API 44, ADR-0016 §4): a record of the problem it
//! explains, how it tilts what its holders hold dear, the laws it proposes, its legitimacy story
//! in words, and how it travels (research 06-04 §6.1: problem explanation → moral commitments →
//! institutional proposals → legitimacy narrative). Who holds one, and what they make of it, is
//! decided at run time.

use std::collections::BTreeMap;

use civ_agents::ideology::IdeologyDef;
use civ_agents::polity::IssueKind;
use serde::Deserialize;

/// The `kind` value of an ideology.
pub const KIND: &str = "ideology";
/// The id segment: `pack:ideology/name`.
pub const ID_KIND: &str = "ideology";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdeologyFile {
    pub kind: String,
    pub id: String,
    /// What it is called, to follow "holds to": "common provision".
    pub name: String,
    pub description: String,
    /// The problem it explains: an issue (`food_short`, `store_unkept`, `takings`, `overruled`).
    pub explains: String,
    /// Its legitimacy story, in words.
    pub legitimacy: String,
    /// The laws it proposes: policy template ids.
    #[serde(default)]
    pub program: Vec<String>,
    /// How it tilts what its holders hold dear: value id to −1 .. 1.
    #[serde(default)]
    pub commitments: BTreeMap<String, f64>,
    pub spread: Spread,
}

/// How it travels: the share of founders who bring it, the chance a holder speaks of it at the
/// hearth a session, the chance a listener who trusts them fully and holds just what it is
/// committed to takes it up, the chance a child takes up a parent's, and the points it adds to
/// proposing a law of its program.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spread {
    pub founders: f64,
    pub share: f64,
    pub adopt: f64,
    pub inherit: f64,
    pub w_program: f64,
}

impl IdeologyFile {
    /// The compiled ideology (call after [`IdeologyFile::problems`] found none, with every value
    /// and template it names defined: `value_index` and `policy_index` find them).
    pub fn def(
        &self,
        value_index: &dyn Fn(&str) -> Option<usize>,
        policy_index: &dyn Fn(&str) -> Option<usize>,
    ) -> IdeologyDef {
        let mut commitments: Vec<(u16, f32)> = self
            .commitments
            .iter()
            .filter_map(|(id, &c)| Some((u16::try_from(value_index(id)?).ok()?, c as f32)))
            .collect();
        commitments.sort_by_key(|c| c.0);
        IdeologyDef {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            explains: IssueKind::from_name(&self.explains).unwrap_or(IssueKind::FoodShort),
            legitimacy: self.legitimacy.clone(),
            commitments,
            program: self
                .program
                .iter()
                .filter_map(|id| u16::try_from(policy_index(id)?).ok())
                .collect(),
            founders: self.spread.founders,
            share: self.spread.share,
            adopt: self.spread.adopt,
            inherit: self.spread.inherit,
            w_program: self.spread.w_program,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if IssueKind::from_name(&self.explains).is_none() {
            let all: Vec<&str> = IssueKind::ALL.iter().map(|k| k.name()).collect();
            p.push(format!("`explains` must be one of `{}`", all.join("`, `")));
        }
        for (key, text) in [("name", &self.name), ("legitimacy", &self.legitimacy)] {
            if text.trim().is_empty() {
                p.push(format!("`{key}` must say something"));
            }
        }
        for (id, &c) in &self.commitments {
            if !(c.is_finite() && (-1.0..=1.0).contains(&c)) {
                p.push(format!(
                    "`commitments` gives `{id}` {c}; it must be between -1 and 1"
                ));
            }
        }
        let s = &self.spread;
        for (name, v, lo, hi) in [
            ("spread.founders", s.founders, 0.0, 1.0),
            ("spread.share", s.share, 0.0, 1.0),
            ("spread.adopt", s.adopt, 0.0, 1.0),
            ("spread.inherit", s.inherit, 0.0, 1.0),
            ("spread.w_program", s.w_program, 0.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        p
    }
}
