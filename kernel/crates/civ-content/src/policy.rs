//! Policy templates (`kind = "policy"`, ADR-0013 §3): what a law of this kind does, the issues
//! whose presence makes proposing it a move, and the levels a sponsor may put forward. Whether
//! anyone proposes it, and whether it passes, is decided by people at run time; a template
//! carries no weight toward being chosen.

use civ_agents::polity::{
    Body, IssueKind, Membership, PassRule, PolicyDef, PolicyKind, RuleDef, RuleTemplate, WALL_KINDS,
};
use civ_agents::seats::{SeatMethod, Seats};
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
    /// `keep_watch`, `curfew`, `amend_body` or `repeal` (content API 49), `claim_place` (59),
    /// `agreement` (62), `build_crossing` (67), `close_source` or `dig_well` (M6a), or
    /// `build_bank` (88) or `keep_bank` (90), or `building_rule` (93).
    pub does: String,
    /// The issues it answers: `food_short`, `store_unkept`, `takings`, `overruled`, `petition`,
    /// `founding` (content API 49), `outsiders` (59), `claimed_from_us`, `terms_sought` (61) or
    /// `fords` (67), `sickness`, `source_suspected` or `water_far` (M6a), `flooded` (88),
    /// `bank_worn` (90) or `burned` (93).
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
    /// `landholders`; content API 118, M7c slice BZ: `organized`, the members of organizations the
    /// polity recognizes, and `privileged`, those of organizations it grants a privilege), the
    /// shares of members who must come, and the rules (`more_for`,
    /// `two_thirds`) a sponsor may combine into the body they propose.
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub quorum_shares: Vec<f64>,
    #[serde(default)]
    pub pass: Vec<String>,
    /// For an amendment to a council (content API 106, M7a slice BO, ADR-0024 §1): the seat
    /// counts, the ways seats are filled (`election`, `lot`) and the terms in days a sponsor may
    /// combine; `members` then names the councils' electorates. Left out, the template's bodies
    /// are bodies of all their members.
    #[serde(default)]
    pub seats: Vec<u8>,
    #[serde(default)]
    pub seat_methods: Vec<String>,
    #[serde(default)]
    pub seat_terms: Vec<u16>,
    /// How a council's seats may be held among a polity's settlements (content API 108, M7a
    /// slice BQ): `at_large` (the whole polity elects them all) and `by_settlement` (each
    /// settlement elects its share). Left out, `at_large` alone.
    #[serde(default)]
    pub seat_districts: Vec<String>,
    /// The question people take positions on (content API 41, M4c slice AG): "whether to keep a
    /// common store". Left out, nobody holds a position on it.
    #[serde(default)]
    pub question: Option<String>,
    /// How a law of it bears on each value (content API 43, M4c slice AG): value id to −1
    /// (against it) .. 1 (for it). Left out, it bears on none.
    #[serde(default)]
    pub bears: std::collections::BTreeMap<String, f64>,
    /// For a rule for building (content API 93, M6b slice BH): what it requires.
    #[serde(default)]
    pub rule: Option<RuleFile>,
    /// For an office (`keep_store`, `keep_watch`): a term and how it is filled (content API 105,
    /// M7a slice BN, ADR-0024 §2). Absent, the office is held until its holder dies or leaves,
    /// and the body names its holder.
    #[serde(default)]
    pub tenure: Option<TenureFile>,
    /// For a forum (`hear_cases`, content API 110, M7b slice BR, ADR-0025 §1): who sits and what
    /// it hears.
    #[serde(default)]
    pub forum: Option<ForumFile>,
    /// For a limit (`limit`, content API 112, M7b slice BT): what it forbids.
    #[serde(default)]
    pub limit: Option<LimitFile>,
    /// For a duty (`duty`, content API 114, M7c slice BV): what it is laid on. Its rates are its
    /// `levy_shares`.
    #[serde(default)]
    pub duty: Option<DutyFile>,
    /// For a privilege (`privilege`, content API 116, M7c slice BX): what it grants. Its
    /// `bundles` are what a finding of a breach imposes.
    #[serde(default)]
    pub privilege: Option<PrivilegeFile>,
}

/// A privilege's terms in a policy file (content API 116, M7c slice BX).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PrivilegeFile {
    /// `licence` (the sole right to offer the goods at doors in the settlement) or `staple` (the
    /// sole right to sell there such goods fetched from another settlement).
    pub grant: String,
    /// The purpose of the goods: `food`, `fuel`, `material`, `tool` or `store`.
    pub purpose: String,
}

/// A duty's terms in a policy file (content API 114, M7c slice BV).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DutyFile {
    /// The purpose of the goods it is laid on: `food`, `fuel`, `material`, `tool` or `store`.
    pub purpose: String,
    /// Whether households that offer such goods at home are spared it.
    #[serde(default)]
    pub exempt_sellers: bool,
    /// Whether the law names one who collects it (content API 117, M7c slice BY).
    #[serde(default)]
    pub collector: bool,
}

/// A forum's terms in a policy file (content API 110).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ForumFile {
    /// `holder` (one the body names), `panel` (drawn by lot for each sitting) or `elders`.
    pub seat: String,
    /// For a panel: how many are drawn (3 to 15).
    #[serde(default)]
    pub panel: u8,
    /// The kinds of law whose cases it hears (`does` names).
    pub hears: Vec<String>,
    /// For a panel or the elders: the share of those who sit who must come (all who sit are
    /// called) and the rule, as for a body.
    #[serde(default = "half")]
    pub quorum_share: f64,
    #[serde(default = "more_for")]
    pub pass: String,
    /// How far its sitters follow remembered findings (content API 111, M7b slice BS):
    /// `informative` (the default), `persuasive` or `binding`.
    #[serde(default = "informative")]
    pub authority: String,
    /// Whether it may hear challenges of laws under limits in force (content API 112, M7b slice
    /// BT).
    #[serde(default)]
    pub review: bool,
}

/// A limit's terms in a policy file (content API 112, M7b slice BT).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LimitFile {
    /// `daylight_work` or `levy_share`.
    pub predicate: String,
    /// For daylight work: minutes of the longest day's light a curfew may cover.
    #[serde(default)]
    pub minutes: f64,
    /// For a levy: the largest share of a harvest a law may levy.
    #[serde(default)]
    pub share: f64,
    /// The rule its change is decided by (`more_for` or `two_thirds`).
    pub change: String,
}

fn informative() -> String {
    "informative".to_owned()
}

fn half() -> f64 {
    0.5
}

fn more_for() -> String {
    "more_for".to_owned()
}

/// What a rule for building requires (M6b slice BH; ADR-0022 §12).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuleFile {
    /// `roofs_unburnt`: a new roof within `within_m` of another building must not burn;
    /// `walls_of_kinds`: new walls must be of one of `walls` (content API 94); `least_gap`: a
    /// new building must stand `gap_m` from every other (94).
    pub template: String,
    /// For `roofs_unburnt`: how near another building a roof is bound, metres.
    #[serde(default)]
    pub within_m: f64,
    /// For `walls_of_kinds`: the wall kinds allowed, of `wattle`, `rubble`, `mud_brick` and
    /// `fired_brick`.
    #[serde(default)]
    pub walls: Vec<String>,
    /// For `least_gap`: the least gap between a new building's plot and every other's, metres.
    #[serde(default)]
    pub gap_m: f64,
    /// The technique a sponsor must know to put it forward (research 11-09 §1.4).
    #[serde(default)]
    pub needs: Option<String>,
}

impl RuleFile {
    /// Its template, if the name is one.
    fn template(&self) -> Option<RuleTemplate> {
        match self.template.as_str() {
            "roofs_unburnt" => Some(RuleTemplate::RoofsUnburnt {
                within_m: self.within_m as f32,
            }),
            "walls_of_kinds" => Some(RuleTemplate::WallsOfKinds {
                kinds: self.walls.iter().fold(0u8, |m, w| {
                    WALL_KINDS
                        .iter()
                        .position(|k| k == w)
                        .map_or(m, |k| m | (1 << k))
                }),
            }),
            "least_gap" => Some(RuleTemplate::LeastGap {
                gap_m: self.gap_m as f32,
            }),
            _ => None,
        }
    }
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

/// An office's tenure in a policy file (content API 105).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TenureFile {
    /// Days the office is held from the day the body passes it.
    pub term_days: u32,
    /// Whether the one who held it may be named or drawn again for the next term.
    #[serde(default = "yes")]
    pub renewable: bool,
    /// `named`: the body names its holder; `lot`: a lot draws among those who put their names
    /// in.
    #[serde(default = "named")]
    pub fill: String,
}

fn yes() -> bool {
    true
}

fn named() -> String {
    "named".to_owned()
}

impl PolicyFile {
    /// The compiled template (call after [`PolicyFile::problems`] found none, and with every
    /// value `bears` names defined: `value_index` finds it).
    pub fn def(
        &self,
        value_index: &dyn Fn(&str) -> Option<usize>,
        technique_index: &dyn Fn(&str) -> Option<usize>,
    ) -> PolicyDef {
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
            rule: self.rule.as_ref().and_then(|r| {
                Some(RuleDef {
                    template: r.template()?,
                    needs: r.needs.as_deref().and_then(technique_index),
                })
            }),
            tenure: self.tenure.as_ref().map(|t| civ_agents::polity::Tenure {
                term_days: t.term_days,
                renewable: t.renewable,
                by_lot: t.fill == "lot",
            }),
            forum: self.forum.as_ref().and_then(|f| {
                Some(civ_agents::forum::ForumDef {
                    seat: civ_agents::forum::ForumSeat::from_name(&f.seat)?,
                    panel: f.panel,
                    hears: f
                        .hears
                        .iter()
                        .filter_map(|h| PolicyKind::from_name(h))
                        .collect(),
                    rule: civ_agents::polity::Body {
                        members: civ_agents::polity::Membership::Adults,
                        quorum_share: f.quorum_share as f32,
                        pass: civ_agents::polity::PassRule::from_name(&f.pass)?,
                        seats: None,
                    },
                    authority: civ_agents::forum::Authority::from_name(&f.authority)?,
                    review: f.review,
                })
            }),
            privilege: self.privilege.as_ref().and_then(|g| {
                Some(civ_agents::polity::PrivilegeDef {
                    grant: civ_agents::polity::Grant::from_name(&g.grant)?,
                    purpose: civ_agents::params::GoodUse::from_name(&g.purpose)?,
                })
            }),
            duty: self.duty.as_ref().and_then(|d| {
                Some(civ_agents::polity::DutyDef {
                    purpose: civ_agents::params::GoodUse::from_name(&d.purpose)?,
                    exempt_sellers: d.exempt_sellers,
                    collector: d.collector,
                })
            }),
            limit: self.limit.as_ref().and_then(|l| {
                let predicate = civ_agents::forum::Predicate::from_name(&l.predicate)?;
                Some(civ_agents::forum::LimitDef {
                    predicate,
                    bound: match predicate {
                        civ_agents::forum::Predicate::DaylightWork => l.minutes,
                        civ_agents::forum::Predicate::LevyShare => l.share,
                    },
                    change: civ_agents::polity::PassRule::from_name(&l.change)?,
                })
            }),
        }
    }

    /// For an amendment: every body its levels combine, in the order written (memberships, then
    /// quorum shares, then rules).
    fn bodies(&self) -> Vec<Body> {
        if PolicyKind::from_name(&self.does) != Some(PolicyKind::AmendBody) {
            return Vec::new();
        }
        // A council's seats, each way they may be combined (M7a slice BO): none for a body
        // of all its members.
        let mut councils: Vec<Option<(u8, SeatMethod, u16, bool)>> = Vec::new();
        let districts: Vec<bool> = if self.seat_districts.is_empty() {
            vec![false]
        } else {
            self.seat_districts
                .iter()
                .filter_map(|d| match d.as_str() {
                    "at_large" => Some(false),
                    "by_settlement" => Some(true),
                    _ => None,
                })
                .collect()
        };
        for &n in &self.seats {
            for method in self
                .seat_methods
                .iter()
                .filter_map(|m| SeatMethod::from_name(m))
            {
                for &t in &self.seat_terms {
                    for &d in &districts {
                        councils.push(Some((n, method, t, d)));
                    }
                }
            }
        }
        if councils.is_empty() {
            councils.push(None);
        }
        let mut out = Vec::new();
        for m in self.members.iter().filter_map(|m| Membership::from_name(m)) {
            for &q in &self.quorum_shares {
                for r in self.pass.iter().filter_map(|r| PassRule::from_name(r)) {
                    for c in &councils {
                        let seats = c.map(|(n, method, term_days, by_settlement)| Seats {
                            n,
                            electorate: m,
                            method,
                            term_days,
                            by_settlement,
                        });
                        out.push(Body {
                            members: if seats.is_some() {
                                Membership::Seats
                            } else {
                                m
                            },
                            quorum_share: q as f32,
                            pass: r,
                            seats,
                        });
                    }
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
        // A forum's terms (M7b slice BR): only for `hear_cases`, which must have them.
        match (&self.forum, PolicyKind::from_name(&self.does)) {
            (Some(f), Some(PolicyKind::HearCases)) => {
                if civ_agents::forum::ForumSeat::from_name(&f.seat).is_none() {
                    p.push(format!(
                        "`forum.seat` must be `holder`, `panel` or `elders` (got `{}`)",
                        f.seat
                    ));
                }
                if f.seat == "panel" && !(3..=15).contains(&f.panel) {
                    p.push(format!("`forum.panel` must be 3 to 15 (got {})", f.panel));
                }
                if f.hears.is_empty() {
                    p.push("`forum.hears` must name at least one kind of law".to_owned());
                }
                for h in &f.hears {
                    if !matches!(
                        PolicyKind::from_name(h),
                        Some(
                            PolicyKind::AgainstTaking
                                | PolicyKind::AgainstRaiding
                                | PolicyKind::Privilege
                        )
                    ) {
                        p.push(format!(
                            "`forum.hears` names `{h}`; cases are brought under `against_taking`, `against_raiding` and `privilege`"
                        ));
                    }
                }
                if !(0.0..=1.0).contains(&f.quorum_share) {
                    p.push("`forum.quorum_share` must be 0 to 1".to_owned());
                }
                if civ_agents::polity::PassRule::from_name(&f.pass).is_none() {
                    p.push(format!("`forum.pass` names `{}`", f.pass));
                }
                if civ_agents::forum::Authority::from_name(&f.authority).is_none() {
                    p.push(format!(
                        "`forum.authority` must be `informative`, `persuasive` or `binding` (got \
                         `{}`)",
                        f.authority
                    ));
                }
            }
            (None, Some(PolicyKind::HearCases)) => {
                p.push("a `hear_cases` template needs a `forum` table".to_owned());
            }
            (Some(_), _) => p.push("`forum` is for `does = \"hear_cases\"`".to_owned()),
            (None, _) => {}
        }
        // A privilege's terms (M7c slice BX): only for `privilege`, which must have them.
        match (&self.privilege, PolicyKind::from_name(&self.does)) {
            (Some(g), Some(PolicyKind::Privilege)) => {
                if civ_agents::polity::Grant::from_name(&g.grant).is_none() {
                    p.push(format!(
                        "`privilege.grant` must be `licence` or `staple` (got `{}`)",
                        g.grant
                    ));
                }
                if civ_agents::params::GoodUse::from_name(&g.purpose).is_none() {
                    p.push(format!(
                        "`privilege.purpose` must be `food`, `fuel`, `material`, `tool` or \
                         `store` (got `{}`)",
                        g.purpose
                    ));
                }
            }
            (None, Some(PolicyKind::Privilege)) => {
                p.push("a `privilege` template needs a `privilege` table".to_owned());
            }
            (Some(_), _) => p.push("`privilege` is for `does = \"privilege\"`".to_owned()),
            (None, _) => {}
        }
        // A duty's terms (M7c slice BV): only for `duty`, which must have them.
        match (&self.duty, PolicyKind::from_name(&self.does)) {
            (Some(d), Some(PolicyKind::Duty)) => {
                if civ_agents::params::GoodUse::from_name(&d.purpose).is_none() {
                    p.push(format!(
                        "`duty.purpose` must be `food`, `fuel`, `material`, `tool` or `store` \
                         (got `{}`)",
                        d.purpose
                    ));
                }
            }
            (None, Some(PolicyKind::Duty)) => {
                p.push("a `duty` template needs a `duty` table".to_owned());
            }
            (Some(_), _) => p.push("`duty` is for `does = \"duty\"`".to_owned()),
            (None, _) => {}
        }
        // A limit's terms (M7b slice BT): only for `limit`, which must have them.
        match (&self.limit, PolicyKind::from_name(&self.does)) {
            (Some(l), Some(PolicyKind::Limit)) => {
                match civ_agents::forum::Predicate::from_name(&l.predicate) {
                    Some(civ_agents::forum::Predicate::DaylightWork) => {
                        if !(l.minutes.is_finite() && (1.0..=720.0).contains(&l.minutes)) {
                            p.push(format!(
                                "`limit.minutes` must be 1 to 720 (got {})",
                                l.minutes
                            ));
                        }
                    }
                    Some(civ_agents::forum::Predicate::LevyShare) => {
                        if !(l.share.is_finite() && l.share > 0.0 && l.share < 1.0) {
                            p.push(format!(
                                "`limit.share` must lie above 0 and below 1 (got {})",
                                l.share
                            ));
                        }
                    }
                    None => p.push(format!(
                        "`limit.predicate` must be `daylight_work` or `levy_share` (got `{}`)",
                        l.predicate
                    )),
                }
                if civ_agents::polity::PassRule::from_name(&l.change).is_none() {
                    p.push(format!("`limit.change` names `{}`", l.change));
                }
            }
            (None, Some(PolicyKind::Limit)) => {
                p.push("a `limit` template needs a `limit` table".to_owned());
            }
            (Some(_), _) => p.push("`limit` is for `does = \"limit\"`".to_owned()),
            (None, _) => {}
        }
        if let Some(t) = &self.tenure {
            if !matches!(
                PolicyKind::from_name(&self.does),
                Some(PolicyKind::KeepStore | PolicyKind::KeepWatch)
            ) {
                p.push(
                    "`tenure` is for an office: `does` must be `keep_store` or `keep_watch`"
                        .to_owned(),
                );
            }
            // A day to ten years (research 09-14 §2.1: a presiding officer for a day; 09-02
            // §5.1: life subject to removal is the absent tenure).
            if !(1..=3650).contains(&t.term_days) {
                p.push(format!(
                    "`tenure.term_days` must be 1 to 3650 (got {})",
                    t.term_days
                ));
            }
            if !matches!(t.fill.as_str(), "named" | "lot") {
                p.push(format!(
                    "`tenure.fill` must be `named` or `lot` (got `{}`); election and appointment come with seated bodies",
                    t.fill
                ));
            }
        }
        match PolicyKind::from_name(&self.does) {
            // A duty's rates (M7c slice BV): shares of the goods bought, as a store's levy.
            Some(PolicyKind::Duty) => {
                if !(1..=8).contains(&self.levy_shares.len()) {
                    p.push(format!(
                        "`levy_shares` must hold 1 to 8 rates (got {})",
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
                if self.relief_days != 0.0 {
                    p.push("a `duty` policy gives no relief: leave out `relief_days`".to_owned());
                }
            }
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
                | PolicyKind::Repeal
                | PolicyKind::ClaimPlace
                | PolicyKind::Agreement
                | PolicyKind::BuildCrossing
                | PolicyKind::CloseSource
                | PolicyKind::DigWell
                | PolicyKind::BuildBank
                | PolicyKind::KeepBank
                | PolicyKind::BuildingRule
                | PolicyKind::SendBand
                | PolicyKind::Enclose
                | PolicyKind::Annex
                | PolicyKind::HearCases
                | PolicyKind::Limit
                | PolicyKind::Embargo
                | PolicyKind::Recognize),
            ) if !self.levy_shares.is_empty() || self.relief_days != 0.0 => {
                p.push(format!(
                    "a `{}` policy levies nothing: leave out `levy_shares` and `relief_days`",
                    k.name()
                ));
            }
            Some(
                k
                @ (PolicyKind::AgainstTaking | PolicyKind::AgainstRaiding | PolicyKind::Privilege),
            ) => {
                if !self.levy_shares.is_empty() || self.relief_days != 0.0 {
                    p.push(format!(
                        "an `{}` policy levies nothing: leave out `levy_shares` and \
                         `relief_days`",
                        k.name()
                    ));
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
                | PolicyKind::Repeal
                | PolicyKind::ClaimPlace
                | PolicyKind::Agreement
                | PolicyKind::BuildCrossing
                | PolicyKind::CloseSource
                | PolicyKind::DigWell
                | PolicyKind::BuildBank
                | PolicyKind::KeepBank
                | PolicyKind::BuildingRule
                | PolicyKind::SendBand
                | PolicyKind::Enclose
                | PolicyKind::Annex
                | PolicyKind::HearCases
                | PolicyKind::Limit
                | PolicyKind::Embargo
                | PolicyKind::Recognize,
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
            // A council's levels come together or not at all (M7a slice BO).
            let council = [
                !self.seats.is_empty(),
                !self.seat_methods.is_empty(),
                !self.seat_terms.is_empty(),
            ];
            if council.iter().any(|&c| c) && !council.iter().all(|&c| c) {
                p.push(
                    "`seats`, `seat_methods` and `seat_terms` must be given together".to_owned(),
                );
            }
            for &n in &self.seats {
                // Research 09-05 §2.3: councils of 5, 9 and 15.
                if !(3..=31).contains(&n) {
                    p.push(format!("`seats` must each be 3 to 31 (got {n})"));
                }
            }
            for m in &self.seat_methods {
                if SeatMethod::from_name(m).is_none() {
                    p.push(format!(
                        "`seat_methods` names `{m}`; methods are `election`, `lot`"
                    ));
                }
            }
            for d in &self.seat_districts {
                if d != "at_large" && d != "by_settlement" {
                    p.push(format!(
                        "`seat_districts` names `{d}`; they are `at_large`, `by_settlement`"
                    ));
                }
            }
            if !self.seat_districts.is_empty() && !council[0] {
                p.push("`seat_districts` needs `seats`".to_owned());
            }
            for &t in &self.seat_terms {
                if !(30..=3650).contains(&t) {
                    p.push(format!(
                        "`seat_terms` must each be 30 to 3650 days (got {t})"
                    ));
                }
            }
            if self.members.iter().any(|m| m == "seats" || m == "watch") && council[0] {
                p.push(
                    "a council's electorate (`members`) is `adults`, `elders` or `landholders`"
                        .to_owned(),
                );
            }
        } else if !self.members.is_empty()
            || !self.quorum_shares.is_empty()
            || !self.pass.is_empty()
            || !self.seats.is_empty()
            || !self.seat_methods.is_empty()
            || !self.seat_terms.is_empty()
        {
            p.push(
                "only an `amend_body` policy has `members`, `quorum_shares`, `pass` and seats"
                    .to_owned(),
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
        if !matches!(
            PolicyKind::from_name(&self.does),
            Some(PolicyKind::AgainstTaking | PolicyKind::AgainstRaiding | PolicyKind::Privilege)
        ) && !self.bundles.is_empty()
        {
            p.push(
                "only an `against_taking`, `against_raiding` or `privilege` policy has `bundles`"
                    .to_owned(),
            );
        }
        match (PolicyKind::from_name(&self.does), &self.rule) {
            (Some(PolicyKind::BuildingRule), None) => {
                p.push("a `building_rule` policy needs a `[rule]`".to_owned());
            }
            (Some(PolicyKind::BuildingRule), Some(r)) => match r.template() {
                None => p.push(format!(
                    "`rule.template` is `{}`; templates are `roofs_unburnt`, `walls_of_kinds` \
                     and `least_gap`",
                    r.template
                )),
                // Research 12-03 §2.2: radiation from a burning dwelling falls from about 36
                // kW/m² at 1 m to 5 at 4 m; a rule's reach is authored within a street's width.
                Some(RuleTemplate::RoofsUnburnt { .. }) => {
                    if !(r.within_m.is_finite() && r.within_m > 0.0 && r.within_m <= 50.0) {
                        p.push(format!(
                            "`rule.within_m` must be above 0 and at most 50 (got {})",
                            r.within_m
                        ));
                    }
                }
                Some(RuleTemplate::WallsOfKinds { .. }) => {
                    if r.walls.is_empty() {
                        p.push("`rule.walls` must name at least one wall kind".to_owned());
                    }
                    for w in &r.walls {
                        if !WALL_KINDS.contains(&w.as_str()) {
                            p.push(format!(
                                "`rule.walls` names `{w}`; wall kinds are `{}`",
                                WALL_KINDS.join("`, `")
                            ));
                        }
                    }
                }
                // Research 12-03 §2.2: separation thresholds of about 2.14 and 3.14 m for its
                // configurations; a gap is authored within a street's width.
                Some(RuleTemplate::LeastGap { .. }) => {
                    if !(r.gap_m.is_finite() && r.gap_m > 0.0 && r.gap_m <= 50.0) {
                        p.push(format!(
                            "`rule.gap_m` must be above 0 and at most 50 (got {})",
                            r.gap_m
                        ));
                    }
                }
            },
            (_, Some(_)) => p.push("only a `building_rule` policy has a `[rule]`".to_owned()),
            (_, None) => {}
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
            seats: Vec::new(),
            seat_methods: Vec::new(),
            seat_terms: Vec::new(),
            seat_districts: Vec::new(),
            question: None,
            bears: Default::default(),
            rule: None,
            tenure: None,
            forum: None,
            limit: None,
            duty: None,
            privilege: None,
        }
    }

    fn no_values(_: &str) -> Option<usize> {
        None
    }

    #[test]
    fn a_building_rule_needs_its_rule_and_nothing_else_has_one() {
        let rule = |template: &str, within_m: f64| RuleFile {
            template: template.to_owned(),
            within_m,
            walls: Vec::new(),
            gap_m: 0.0,
            needs: Some("core:technique/firing".to_owned()),
        };
        let roofs_with = |r: Option<RuleFile>| PolicyFile {
            does: "building_rule".to_owned(),
            answers: vec!["burned".to_owned()],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            rule: r,
            tenure: None,
            forum: None,
            limit: None,
            duty: None,
            privilege: None,
            ..file()
        };
        let roofs = roofs_with(Some(rule("roofs_unburnt", 4.0)));
        assert!(roofs.problems().is_empty(), "{:?}", roofs.problems());
        let firing = |id: &str| (id == "core:technique/firing").then_some(3);
        let d = roofs.def(&no_values, &firing);
        assert_eq!(d.kind, PolicyKind::BuildingRule);
        assert_eq!(
            d.rule,
            Some(RuleDef {
                template: RuleTemplate::RoofsUnburnt { within_m: 4.0 },
                needs: Some(3),
            })
        );
        let missing = roofs_with(None);
        assert!(
            missing
                .problems()
                .iter()
                .any(|p| p.contains("needs a `[rule]`"))
        );
        for bad in [
            rule("walls", 4.0),
            rule("roofs_unburnt", 0.0),
            rule("roofs_unburnt", 99.0),
        ] {
            let f = roofs_with(Some(bad));
            assert!(!f.problems().is_empty());
        }
        let stray = PolicyFile {
            rule: Some(rule("roofs_unburnt", 4.0)),
            tenure: None,
            forum: None,
            limit: None,
            duty: None,
            privilege: None,
            ..file()
        };
        assert!(
            stray
                .problems()
                .iter()
                .any(|p| p.contains("only a `building_rule`"))
        );
        // Walls of named kinds, and a least gap (content API 94).
        let other = |template: &str, walls: &[&str], gap_m: f64| {
            roofs_with(Some(RuleFile {
                template: template.to_owned(),
                within_m: 0.0,
                walls: walls.iter().map(|w| (*w).to_owned()).collect(),
                gap_m,
                needs: None,
            }))
        };
        let walls = other("walls_of_kinds", &["rubble", "mud_brick"], 0.0);
        assert!(walls.problems().is_empty(), "{:?}", walls.problems());
        assert_eq!(
            walls.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::WallsOfKinds { kinds: 0b0110 })
        );
        assert!(
            !other("walls_of_kinds", &["straw"], 0.0)
                .problems()
                .is_empty()
        );
        assert!(!other("walls_of_kinds", &[], 0.0).problems().is_empty());
        let gap = other("least_gap", &[], 3.0);
        assert!(gap.problems().is_empty(), "{:?}", gap.problems());
        assert_eq!(
            gap.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::LeastGap { gap_m: 3.0 })
        );
        assert!(!other("least_gap", &[], 0.0).problems().is_empty());
    }

    #[test]
    fn a_template_compiles_with_its_levels_in_order() {
        let f = file();
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values, &no_values);
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
        assert_eq!(f.def(&no_values, &no_values).kind, PolicyKind::KeepStore);
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
        let d = f.def(&no_values, &no_values);
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
        let d = f.def(&no_values, &no_values);
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
        let d = f.def(&no_values, &no_values);
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
