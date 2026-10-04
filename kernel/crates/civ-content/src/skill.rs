//! Skills (`kind = "skill"`): domains people get better at with practice (ADR-0006 §2). Who
//! practises what is decided by people at run time.

use civ_agents::params::SkillDef;
use serde::Deserialize;

/// The `kind` value of a skill.
pub const KIND: &str = "skill";
/// The id segment: `pack:skill/name`.
pub const ID_KIND: &str = "skill";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// Hours of practice that close 80 % of the gap to mastery.
    pub t80_h: f64,
    /// Work speed by level: `[level, factor]`, ascending.
    pub speed: Vec<[f64; 2]>,
    /// Life of the tools made, by level: `[level, factor]`, ascending.
    pub quality: Vec<[f64; 2]>,
    /// Levels a grown founder brings: from, to.
    pub founder_level: [f64; 2],
}

impl SkillFile {
    /// The compiled skill.
    pub fn def(&self) -> SkillDef {
        let pairs = |v: &[[f64; 2]]| v.iter().map(|&[a, b]| (a, b)).collect();
        SkillDef {
            id: self.id.clone(),
            name: self.name.clone(),
            t80_h: self.t80_h,
            speed: pairs(&self.speed),
            quality: pairs(&self.quality),
            founder_level: self.founder_level,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if !(self.t80_h.is_finite() && self.t80_h > 0.0) {
            p.push(format!("`t80_h` must be positive (got {})", self.t80_h));
        }
        for (name, table) in [("speed", &self.speed), ("quality", &self.quality)] {
            if table.is_empty()
                || table.windows(2).any(|w| w[1][0] <= w[0][0])
                || table.iter().any(|&[l, f]| {
                    !(l.is_finite() && (0.0..=1.0).contains(&l) && f.is_finite() && f > 0.0)
                })
            {
                p.push(format!(
                    "`{name}` needs [level, factor] pairs with levels 0-1, strictly ascending, and \
                     positive factors"
                ));
            }
        }
        let [lo, hi] = self.founder_level;
        if !(lo.is_finite() && hi.is_finite() && 0.0 <= lo && lo <= hi && hi <= 1.0) {
            p.push("`founder_level` must satisfy 0 <= from <= to <= 1".to_owned());
        }
        p
    }
}
