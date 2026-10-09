//! Policy templates (`kind = "policy"`, ADR-0013 §3): what a law of this kind does, the issues
//! whose presence makes proposing it a move, and the levels a sponsor may put forward. Whether
//! anyone proposes it, and whether it passes, is decided by people at run time; a template
//! carries no weight toward being chosen.

use civ_agents::polity::{Body, IssueKind, Membership, PassRule, PolicyDef, PolicyKind};
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
    /// What the kernel does under it: `common_store`, `keep_store`, `against_taking`,
    /// `keep_watch`, `curfew`, `amend_body` or `repeal` (content API 49).
    pub does: String,
    /// The issues it answers: `food_short`, `store_unkept`, `takings`, `overruled`, `petition`,
    /// `founding` (content API 49).
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
    /// For a curfew (content API 38): the hours a sponsor may propose, each `[from, to]` hours of
    /// the day, past midnight when `to` is the smaller.
    #[serde(default)]
    pub hours: Vec<[u8; 2]>,
    /// For an amendment of the custom (content API 40): who may belong (`adults`, `elders`,
    /// `landholders`), the shares of members who must come, and the rules (`more_for`,
    /// `two_thirds`) a sponsor may combine into the body they propose.
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub quorum_shares: Vec<f64>,
    #[serde(default)]
    pub pass: Vec<String>,
    /// The question people take positions on (content API 41, M4c slice AG): "whether to keep a
    /// common store". Left out, nobody holds a position on it.
    #[serde(default)]
    pub question: Option<String>,
    /// How a law of it bears on each value (content API 43, M4c slice AG): value id to −1
    /// (against it) .. 1 (for it). Left out, it bears on none.
    #[serde(default)]
    pub bears: std::collections::BTreeMap<String, f64>,
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
    /// The compiled template (call after [`PolicyFile::problems`] found none, and with every
    /// value `bears` names defined: `value_index` finds it).
    pub fn def(&self, value_index: &dyn Fn(&str) -> Option<usize>) -> PolicyDef {
        let mut bears: Vec<(u16, f32)> = self
            .bears
            .iter()
            .filter_map(|(id, &b)| Some((u16::try_from(value_index(id)?).ok()?, b as f32)))
            .collect();
        bears.sort_by_key(|b| b.0);
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
            hours: self.hours.iter().map(|h| (h[0], h[1])).collect(),
            bodies: self.bodies(),
            question: self.question.clone(),
            bears,
        }
    }

    /// For an amendment: every body its levels combine, in the order written (memberships, then
    /// quorum shares, then rules).
    fn bodies(&self) -> Vec<Body> {
        if PolicyKind::from_name(&self.does) != Some(PolicyKind::AmendBody) {
            return Vec::new();
        }
        let mut out = Vec::new();
        for m in self.members.iter().filter_map(|m| Membership::from_name(m)) {
            for &q in &self.quorum_shares {
                for r in self.pass.iter().filter_map(|r| PassRule::from_name(r)) {
                    out.push(Body {
                        members: m,
                        quorum_share: q as f32,
                        pass: r,
                    });
                }
            }
        }
        out
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for (id, &b) in &self.bears {
            if !(b.is_finite() && (-1.0..=1.0).contains(&b)) {
                p.push(format!(
                    "`bears` gives `{id}` {b}; it must be between -1 and 1"
                ));
            }
        }
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
            Some(
                k @ (PolicyKind::KeepStore
                | PolicyKind::KeepWatch
                | PolicyKind::Curfew
                | PolicyKind::AmendBody
                | PolicyKind::Repeal),
            ) if !self.levy_shares.is_empty() || self.relief_days != 0.0 => {
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
            Some(
                PolicyKind::KeepStore
                | PolicyKind::KeepWatch
                | PolicyKind::Curfew
                | PolicyKind::AmendBody
                | PolicyKind::Repeal,
            )
            | None => {}
        }
        if PolicyKind::from_name(&self.does) == Some(PolicyKind::AmendBody) {
            if !(1..=Membership::ALL.len()).contains(&self.members.len()) {
                p.push(format!(
                    "`members` must hold 1 to {} memberships (got {})",
                    Membership::ALL.len(),
                    self.members.len()
                ));
            }
            for m in &self.members {
                if Membership::from_name(m).is_none() {
                    let all: Vec<&str> = Membership::ALL.iter().map(|m| m.name()).collect();
                    p.push(format!(
                        "`members` names `{m}`; memberships are `{}`",
                        all.join("`, `")
                    ));
                }
            }
            if !(1..=8).contains(&self.quorum_shares.len()) {
                p.push(format!(
                    "`quorum_shares` must hold 1 to 8 shares (got {})",
                    self.quorum_shares.len()
                ));
            }
            for &q in &self.quorum_shares {
                if !(q.is_finite() && (0.0..=1.0).contains(&q)) {
                    p.push(format!(
                        "`quorum_shares` must lie between 0 and 1 (got {q})"
                    ));
                }
            }
            if !(1..=2).contains(&self.pass.len()) {
                p.push(format!(
                    "`pass` must hold 1 or 2 rules (got {})",
                    self.pass.len()
                ));
            }
            for r in &self.pass {
                if PassRule::from_name(r).is_none() {
                    let all: Vec<&str> = PassRule::ALL.iter().map(|r| r.name()).collect();
                    p.push(format!(
                        "`pass` names `{r}`; rules are `{}`",
                        all.join("`, `")
                    ));
                }
            }
        } else if !self.members.is_empty()
            || !self.quorum_shares.is_empty()
            || !self.pass.is_empty()
        {
            p.push(
                "only an `amend_body` policy has `members`, `quorum_shares` and `pass`".to_owned(),
            );
        }
        if PolicyKind::from_name(&self.does) == Some(PolicyKind::Curfew) {
            if !(1..=8).contains(&self.hours.len()) {
                p.push(format!(
                    "`hours` must hold 1 to 8 pairs (got {})",
                    self.hours.len()
                ));
            }
            for h in &self.hours {
                if h[0] > 23 || h[1] > 23 || h[0] == h[1] {
                    p.push(format!(
                        "each of `hours` must be two different hours of the day, 0 to 23 (got \
                         {h:?})"
                    ));
                }
            }
        } else if !self.hours.is_empty() {
            p.push("only a `curfew` policy has `hours`".to_owned());
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
            hours: Vec::new(),
            members: Vec::new(),
            quorum_shares: Vec::new(),
            pass: Vec::new(),
            question: None,
            bears: Default::default(),
        }
    }

    fn no_values(_: &str) -> Option<usize> {
        None
    }

    #[test]
    fn a_template_compiles_with_its_levels_in_order() {
        let f = file();
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values);
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
        assert_eq!(f.def(&no_values).kind, PolicyKind::KeepStore);
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
        let d = f.def(&no_values);
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

    #[test]
    fn a_curfew_carries_its_hours_and_nothing_else_does() {
        let mut f = file();
        f.does = "curfew".to_owned();
        f.answers = vec!["takings".to_owned()];
        f.levy_shares.clear();
        f.relief_days = 0.0;
        assert!(
            f.problems().iter().any(|m| m.contains("1 to 8 pairs")),
            "{:?}",
            f.problems()
        );
        f.hours = vec![[21, 5]];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values);
        assert_eq!(d.kind, PolicyKind::Curfew);
        assert_eq!(d.hours, vec![(21, 5)]);
        f.hours = vec![[24, 5], [6, 6]];
        assert_eq!(f.problems().len(), 2, "{:?}", f.problems());
        // A common store has no hours.
        let mut g = file();
        g.hours = vec![[21, 5]];
        assert!(g.problems().iter().any(|m| m.contains("only a `curfew`")));
    }

    #[test]
    fn an_amendment_combines_its_levels_into_bodies_and_nothing_else_has_them() {
        let mut f = file();
        f.does = "amend_body".to_owned();
        f.answers = vec!["overruled".to_owned()];
        f.levy_shares = Vec::new();
        f.relief_days = 0.0;
        f.members = vec!["adults".to_owned(), "elders".to_owned()];
        f.quorum_shares = vec![0.25, 0.5];
        f.pass = vec!["more_for".to_owned()];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values);
        assert_eq!(d.kind, PolicyKind::AmendBody);
        assert_eq!(d.answers, vec![IssueKind::Overruled]);
        assert_eq!(d.bodies.len(), 4);
        assert_eq!(d.bodies[0].members, Membership::Adults);
        assert_eq!(d.bodies[3].members, Membership::Elders);
        assert!((d.bodies[3].quorum_share - 0.5).abs() < 1e-6);
        f.members = vec!["chiefs".to_owned()];
        f.quorum_shares = vec![1.5];
        f.pass = vec!["unanimity".to_owned()];
        assert_eq!(f.problems().len(), 3, "{:?}", f.problems());
        // A common store has none of them.
        let mut g = file();
        g.pass = vec!["more_for".to_owned()];
        assert_eq!(g.problems().len(), 1);
    }
}
