//! Policy templates (`kind = "policy"`, ADR-0013 §3): what a law of this kind does, the issues
//! whose presence makes proposing it a move, and the levels a sponsor may put forward. Whether
//! anyone proposes it, and whether it passes, is decided by people at run time; a template
//! carries no weight toward being chosen.

use civ_agents::polity::{IssueKind, PolicyDef, PolicyKind};
use serde::Deserialize;

/// The `kind` value of a policy template.
pub const KIND: &str = "policy";
/// The id segment: `pack:policy/name`.
pub const ID_KIND: &str = "policy";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolicyFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub description: String,
    /// What the kernel does under it: `common_store`, `keep_store` or `against_taking`.
    pub does: String,
    /// The issues it answers: `food_short`, `store_unkept`, `takings`.
    pub answers: Vec<String>,
    /// For a common store: the shares of threshed grain a sponsor may propose for the levy.
    #[serde(default)]
    pub levy_shares: Vec<f64>,
    /// For a common store: the most food one ask brings a household from it, days of its need.
    #[serde(default)]
    pub relief_days: f64,
    /// For a law against taking (content API 36): the bundles a sponsor may propose, lightest
    /// first.
    #[serde(default)]
    pub bundles: Vec<BundleFile>,
}

/// One sanction bundle of a law against taking: what it adds to giving back what was taken, in
/// days of the taker's household's food.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleFile {
    pub compensation_days: f64,
    pub fine_days: f64,
    /// The one found to have taken is sent from the valley.
    #[serde(default)]
    pub exile: bool,
}

impl PolicyFile {
    /// The compiled template (call after [`PolicyFile::problems`] found none).
    pub fn def(&self) -> PolicyDef {
        let mut levy_shares = self.levy_shares.clone();
        levy_shares.sort_by(f64::total_cmp);
        PolicyDef {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            kind: PolicyKind::from_name(&self.does).unwrap_or(PolicyKind::CommonStore),
            answers: self
                .answers
                .iter()
                .filter_map(|a| IssueKind::from_name(a))
                .collect(),
            levy_shares,
            relief_days: self.relief_days,
            bundles: self
                .bundles
                .iter()
                .map(|b| civ_agents::polity::Sanction {
                    compensation_days: b.compensation_days as f32,
                    fine_days: b.fine_days as f32,
                    exile: b.exile,
                })
                .collect(),
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if PolicyKind::from_name(&self.does).is_none() {
            let all: Vec<&str> = PolicyKind::ALL.iter().map(|k| k.name()).collect();
            p.push(format!("`does` must be one of `{}`", all.join("`, `")));
        }
        if self.answers.is_empty() {
            p.push("`answers` must name at least one issue".to_owned());
        }
        for a in &self.answers {
            if IssueKind::from_name(a).is_none() {
                let all: Vec<&str> = IssueKind::ALL.iter().map(|k| k.name()).collect();
                p.push(format!(
                    "`answers` names `{a}`; issues are `{}`",
                    all.join("`, `")
                ));
            }
        }
        match PolicyKind::from_name(&self.does) {
            Some(PolicyKind::CommonStore) => {
                // Research 09-05 §2.3: a sponsor weighs three to eight levels; one is allowed.
                if !(1..=8).contains(&self.levy_shares.len()) {
                    p.push(format!(
                        "`levy_shares` must hold 1 to 8 shares (got {})",
                        self.levy_shares.len()
                    ));
                }
                for &s in &self.levy_shares {
                    if !(s.is_finite() && s > 0.0 && s < 1.0) {
                        p.push(format!(
                            "`levy_shares` must lie above 0 and below 1 (got {s})"
                        ));
                    }
                }
                let d = self.relief_days;
                if !(d.is_finite() && d > 0.0 && d <= 365.0) {
                    p.push(format!(
                        "`relief_days` must be above 0 and at most 365 (got {d})"
                    ));
                }
            }
            Some(k @ (PolicyKind::KeepStore | PolicyKind::KeepWatch))
                if !self.levy_shares.is_empty() || self.relief_days != 0.0 =>
            {
                p.push(format!(
                    "a `{}` policy levies nothing: leave out `levy_shares` and `relief_days`",
                    k.name()
                ));
            }
            Some(PolicyKind::AgainstTaking) => {
                if !self.levy_shares.is_empty() || self.relief_days != 0.0 {
                    p.push(
                        "an `against_taking` policy levies nothing: leave out `levy_shares` and \
                         `relief_days`"
                            .to_owned(),
                    );
                }
                // Research 09-05 §2.3: a sponsor weighs a few levels; one is allowed.
                if !(1..=8).contains(&self.bundles.len()) {
                    p.push(format!(
                        "`bundles` must hold 1 to 8 bundles (got {})",
                        self.bundles.len()
                    ));
                }
                for b in &self.bundles {
                    for (what, d) in [
                        ("compensation_days", b.compensation_days),
                        ("fine_days", b.fine_days),
                    ] {
                        if !(d.is_finite() && (0.0..=365.0).contains(&d)) {
                            p.push(format!(
                                "a bundle's `{what}` must be between 0 and 365 (got {d})"
                            ));
                        }
                    }
                }
            }
            Some(PolicyKind::KeepStore | PolicyKind::KeepWatch) | None => {}
        }
        if PolicyKind::from_name(&self.does) != Some(PolicyKind::AgainstTaking)
            && !self.bundles.is_empty()
        {
            p.push("only an `against_taking` policy has `bundles`".to_owned());
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file() -> PolicyFile {
        PolicyFile {
            kind: KIND.to_owned(),
            id: "core:policy/test".to_owned(),
            name: "Common store".to_owned(),
            description: "A test.".to_owned(),
            does: "common_store".to_owned(),
            answers: vec!["food_short".to_owned()],
            levy_shares: vec![0.2, 0.05, 0.1],
            relief_days: 5.0,
            bundles: Vec::new(),
        }
    }

    #[test]
    fn a_template_compiles_with_its_levels_in_order() {
        let f = file();
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def();
        assert_eq!(d.kind, PolicyKind::CommonStore);
        assert_eq!(d.answers, vec![IssueKind::FoodShort]);
        assert_eq!(d.levy_shares, vec![0.05, 0.1, 0.2]);
    }

    #[test]
    fn unknown_kinds_issues_and_shares_out_of_range_are_refused() {
        let mut f = file();
        f.does = "tithe".to_owned();
        f.answers = vec!["floods".to_owned()];
        assert_eq!(f.problems().len(), 2, "{:?}", f.problems());
        let mut f = file();
        f.levy_shares = vec![0.0, 1.5];
        f.relief_days = 0.0;
        assert_eq!(f.problems().len(), 3, "{:?}", f.problems());
        f.answers.clear();
        assert!(f.problems().iter().any(|m| m.contains("at least one")));
    }

    #[test]
    fn a_keeper_levies_nothing() {
        let mut f = file();
        f.does = "keep_store".to_owned();
        f.answers = vec!["store_unkept".to_owned()];
        assert_eq!(f.problems().len(), 1, "it must not carry levy shares");
        f.levy_shares.clear();
        f.relief_days = 0.0;
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        assert_eq!(f.def().kind, PolicyKind::KeepStore);
        // So does a watch.
        f.does = "keep_watch".to_owned();
        f.answers = vec!["takings".to_owned()];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        f.relief_days = 5.0;
        assert!(
            f.problems()
                .iter()
                .any(|m| m.contains("`keep_watch` policy levies nothing"))
        );
    }

    #[test]
    fn a_law_against_taking_carries_its_bundles_and_nothing_else_does() {
        let mut f = file();
        f.does = "against_taking".to_owned();
        f.answers = vec!["takings".to_owned()];
        f.levy_shares.clear();
        f.relief_days = 0.0;
        assert!(
            f.problems().iter().any(|m| m.contains("1 to 8 bundles")),
            "it needs a bundle: {:?}",
            f.problems()
        );
        f.bundles = vec![
            BundleFile {
                compensation_days: 0.0,
                fine_days: 0.0,
                exile: false,
            },
            BundleFile {
                compensation_days: 3.0,
                fine_days: 1.5,
                exile: true,
            },
        ];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def();
        assert_eq!(d.kind, PolicyKind::AgainstTaking);
        assert_eq!(d.answers, vec![IssueKind::Takings]);
        assert_eq!(d.bundles[1].compensation_days, 3.0);
        assert_eq!(d.bundles[1].fine_days, 1.5);
        assert!(d.bundles[1].exile && !d.bundles[0].exile);
        f.bundles[0].fine_days = -1.0;
        f.bundles[1].compensation_days = 400.0;
        assert_eq!(f.problems().len(), 2, "{:?}", f.problems());
        // A common store has no bundles.
        let mut g = file();
        g.bundles = f.bundles.clone();
        assert!(
            g.problems()
                .iter()
                .any(|m| m.contains("only an `against_taking`"))
        );
    }
}
