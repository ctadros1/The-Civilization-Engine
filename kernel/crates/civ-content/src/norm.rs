//! Norm templates (`kind = "norm"`, content API 42, ADR-0016 §4): a prescription the kernel knows
//! how to act on (`does`), what it says in words, and the priors from which each person's
//! endorsement, expectation and threshold of it are drawn and moved (research 06-05 §2.2). Who
//! holds it, and how far anyone abides by it, is decided by people at run time.

use civ_agents::norm::{NormDef, NormKind};
use serde::Deserialize;

/// The `kind` value of a norm.
pub const KIND: &str = "norm";
/// The id segment: `pack:norm/name`.
pub const ID_KIND: &str = "norm";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NormFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub description: String,
    /// What it says: "what the gathering decides binds everyone".
    pub statement: String,
    /// What the kernel does with it: `abide_by_laws`.
    pub does: String,
    pub endorse: Endorse,
    pub expect: Expect,
    pub threshold: Threshold,
    pub weights: Weights,
}

/// A founder's endorsement: the log-odds mean and spread, and how much of its parents' mean a
/// child takes on.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Endorse {
    pub mean: f64,
    pub sd: f64,
    pub heritability: f64,
}

/// What people believe others do: before anyone tells them anything, how far one account moves
/// it, the chance a companion tells of their household's last act at the hearth, and for how
/// many days an act is worth telling.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Expect {
    pub prior: f64,
    pub learn_rate: f64,
    pub share: f64,
    pub tell_days: u32,
}

/// Each person's threshold is drawn evenly from `low` to `high`; `width` smooths the step.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Threshold {
    pub low: f64,
    pub high: f64,
    pub width: f64,
}

/// Points for abiding, for a full endorsement and for a full activation by others' doing it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Weights {
    pub endorse: f64,
    pub expect: f64,
}

impl NormFile {
    /// The compiled template (call after [`NormFile::problems`] found none).
    pub fn def(&self) -> NormDef {
        NormDef {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            statement: self.statement.clone(),
            kind: NormKind::from_name(&self.does).unwrap_or(NormKind::AbideByLaws),
            endorse_mean: self.endorse.mean,
            endorse_sd: self.endorse.sd,
            heritability: self.endorse.heritability,
            expect_prior: self.expect.prior,
            learn_rate: self.expect.learn_rate,
            share: self.expect.share,
            tell_days: i64::from(self.expect.tell_days),
            threshold_low: self.threshold.low,
            threshold_high: self.threshold.high,
            width: self.threshold.width,
            w_endorse: self.weights.endorse,
            w_expect: self.weights.expect,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if NormKind::from_name(&self.does).is_none() {
            let all: Vec<&str> = NormKind::ALL.iter().map(|k| k.name()).collect();
            p.push(format!("`does` must be one of `{}`", all.join("`, `")));
        }
        if self.statement.trim().is_empty() {
            p.push("`statement` must say what the norm says".to_owned());
        }
        for (name, v, lo, hi) in [
            ("endorse.mean", self.endorse.mean, -10.0, 10.0),
            ("endorse.sd", self.endorse.sd, 0.0, 10.0),
            ("endorse.heritability", self.endorse.heritability, 0.0, 1.0),
            ("expect.prior", self.expect.prior, 0.0, 1.0),
            ("expect.learn_rate", self.expect.learn_rate, 0.0, 1.0),
            ("expect.share", self.expect.share, 0.0, 1.0),
            (
                "expect.tell_days",
                f64::from(self.expect.tell_days),
                1.0,
                3650.0,
            ),
            ("threshold.low", self.threshold.low, 0.0, 1.0),
            ("threshold.high", self.threshold.high, 0.0, 1.0),
            ("threshold.width", self.threshold.width, 0.001, 1.0),
            ("weights.endorse", self.weights.endorse, 0.0, 100.0),
            ("weights.expect", self.weights.expect, 0.0, 100.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.threshold.low > self.threshold.high {
            p.push(format!(
                "`threshold.low` ({}) must not be above `threshold.high` ({})",
                self.threshold.low, self.threshold.high
            ));
        }
        p
    }
}
