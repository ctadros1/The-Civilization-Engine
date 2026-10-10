//! The polity (ADR-0013): each settlement's body politic, with the founding custom it decides
//! by, its laws and their histories, and its common store. This module is pure: the values, and
//! the forecasts and choices made over copies of the facts. Where people meet, decide, pay and
//! ask is `population::polity`.
//!
//! No issue carries a weight toward a policy (ADR-0013 §5): an issue only makes a move
//! available, and every move, stance and payment is scored by its forecast effect on the
//! person's own household and on those who regard them.

use civ_core::{PermanentId, SimTime};

use crate::person::Flows;

/// What a policy template does, as the kernel carries it out (ADR-0013 §3). Content authors the
/// templates; a law names its template by content id. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyKind {
    /// A share of each household's threshed grain goes into the polity's store, and a household
    /// short of food may ask the store for some (a levy and relief: research 09-17 §1.5, 09-01
    /// §3.3).
    CommonStore,
    /// The one the gathering names keeps the common store under their roof (an office in its
    /// first form, ADR-0013 §2): it spoils as a roofed store does, and those who ask for relief
    /// ask at their home. The law lapses when they die or leave.
    KeepStore,
    /// Taking from another household's store is a wrong the gathering hears (M4b slice AB,
    /// ADR-0015 §4): whoever it finds took gives back what was taken, and the law's bundle
    /// besides (research 09-07 §6.2).
    AgainstTaking,
    /// The one the gathering names keeps watch over the households' stores at night (M4b slice
    /// AC, ADR-0015 §6: a watch is an office, as the storekeeper is): rounds they choose to walk,
    /// guarding and seeing only where they are. The law lapses when they die or leave.
    KeepWatch,
    /// Nobody may be away from home in the hours it sets, but the watch at its rounds and those
    /// at a gathering (M4b slice AD; research 12-04 §1.1, the brief's §1.6): a prohibition whose
    /// keeping is each person's choice (09-06 §1.5), with no sanction in v0.
    Curfew,
    /// The custom itself changes (M4c slice AF; ADR-0013 §2, ADR-0017 §1): who belongs to the
    /// body, how many must come and how it decides, by the custom's own procedure (research
    /// 09-02 §3.7, 09-05 §3.9: who occupies a gate is an amendable rule).
    AmendBody,
    /// A law in force ends, with nothing in its place (M4c slice AI, step two; ADR-0017 §5): the
    /// law it names is superseded, and it is carried rather than in force. A body that took the
    /// deciding weighs ending the laws the custom it replaced made (research 09-11 §1.6:
    /// provisional authority over what survives, unless actors dismantle it).
    Repeal,
    /// The polity claims the places where its people saw people of another settlement at work:
    /// outsiders may use them only by leave (M5c slice AT, ADR-0020 §5). Its own people's use is
    /// unchanged, and nobody is stopped: a claim is words until outsiders heed it.
    ClaimPlace,
    /// The polity's side of an agreement with another polity (M5c slice AU, ADR-0020 §6): its
    /// gathering ratifies what two who met agreed on; it is in force only once both have passed
    /// it and each has heard of the other's decision.
    Agreement,
}

impl PolicyKind {
    /// Every kind, in code order.
    pub const ALL: [PolicyKind; 9] = [
        PolicyKind::CommonStore,
        PolicyKind::KeepStore,
        PolicyKind::AgainstTaking,
        PolicyKind::KeepWatch,
        PolicyKind::Curfew,
        PolicyKind::AmendBody,
        PolicyKind::Repeal,
        PolicyKind::ClaimPlace,
        PolicyKind::Agreement,
    ];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            PolicyKind::CommonStore => "common_store",
            PolicyKind::KeepStore => "keep_store",
            PolicyKind::AgainstTaking => "against_taking",
            PolicyKind::KeepWatch => "keep_watch",
            PolicyKind::Curfew => "curfew",
            PolicyKind::AmendBody => "amend_body",
            PolicyKind::Repeal => "repeal",
            PolicyKind::ClaimPlace => "claim_place",
            PolicyKind::Agreement => "agreement",
        }
    }

    /// The kind with an authored name.
    pub fn from_name(name: &str) -> Option<PolicyKind> {
        PolicyKind::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// A recurring problem a polity can face (research 09-05 §1.1). An issue only makes moves
/// available; it weighs toward none. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueKind {
    /// The settlement's food ran short within the last year, or a household's food will not last
    /// until its next harvest: a shortfall against the outlook.
    FoodShort,
    /// A common store is in force and holds food, and nobody keeps it under a roof.
    StoreUnkept,
    /// A household of the settlement found food taken from its store within the last year (M4b
    /// slice AB): what its members know, not what happened.
    Takings,
    /// A gathering they came to within memory decided against where they stood, or they hold a
    /// grievance against the gathering (M4c slice AF): what one person has seen and holds, so
    /// it opens moves to them alone.
    Overruled,
    /// A faction petitioned the gathering for it (M4c slice AH): its members held grievances
    /// against the gathering. It answers no settlement's issue and opens no move to anyone.
    Petition,
    /// The custom was taken from the gathering lately (M4c slice AI, step two): the body that took
    /// it weighs which of the laws the old custom made stand (research 09-11 §1.6). It opens
    /// moves to that body's members, for those laws alone.
    Founding,
    /// People of another settlement worked places a household of the settlement works, within
    /// the last year, at places it has not claimed (M5c slice AT): what its members saw.
    Outsiders,
    /// A household of the settlement heard that another polity claims places its people work,
    /// which its own polity does not claim (M5c slice AU): what its members were told.
    ClaimedFromUs,
    /// One of the settlement's people met someone of another settlement who sought terms
    /// between their polities, and the two agreed on some to put to their gatherings (M5c slice
    /// AU). It answers no settlement's issue and opens no move to anyone.
    TermsSought,
}

impl IssueKind {
    /// Every kind, in code order.
    pub const ALL: [IssueKind; 9] = [
        IssueKind::FoodShort,
        IssueKind::StoreUnkept,
        IssueKind::Takings,
        IssueKind::Overruled,
        IssueKind::Petition,
        IssueKind::Founding,
        IssueKind::Outsiders,
        IssueKind::ClaimedFromUs,
        IssueKind::TermsSought,
    ];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            IssueKind::FoodShort => "food_short",
            IssueKind::StoreUnkept => "store_unkept",
            IssueKind::Takings => "takings",
            IssueKind::Overruled => "overruled",
            IssueKind::Petition => "petition",
            IssueKind::Founding => "founding",
            IssueKind::Outsiders => "outsiders",
            IssueKind::ClaimedFromUs => "claimed_from_us",
            IssueKind::TermsSought => "terms_sought",
        }
    }

    /// The kind with an authored name.
    pub fn from_name(name: &str) -> Option<IssueKind> {
        IssueKind::ALL.into_iter().find(|k| k.name() == name)
    }

    /// In words, after "because": "food would not last until the harvest".
    pub fn words(self) -> &'static str {
        match self {
            IssueKind::FoodShort => "food would not last until the harvest",
            IssueKind::StoreUnkept => "the common store lay in the open with nobody to keep it",
            IssueKind::Takings => "food was taken from households' stores",
            IssueKind::Overruled => "the gathering had decided against them",
            IssueKind::Petition => {
                "those who came to petition for it held it against the gathering"
            }
            IssueKind::Founding => {
                "the deciding had been taken from the gathering, and the laws it made stood only \
                 until the new body weighed them"
            }
            IssueKind::Outsiders => "people of another settlement worked the places theirs did",
            IssueKind::ClaimedFromUs => "another settlement claims places theirs work",
            IssueKind::TermsSought => {
                "one of theirs met someone of another settlement who sought terms with them"
            }
        }
    }
}

/// What a law against taking adds to giving back what was taken (research 09-07 §1.2, §6.2:
/// restitution, compensation and a fine are different transfers, and a punishment is a bundle),
/// in days of the taker's household's food (09-07 §2.3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sanction {
    /// Food to the household taken from, besides what was taken, days.
    pub compensation_days: f32,
    /// Food to the polity's store, days.
    pub fine_days: f32,
    /// The one found to have taken is sent from the valley (ADR-0015 §5: exile, recorded as a
    /// migration; 09-07 §6.2).
    pub exile: bool,
}

impl Sanction {
    /// In words, after "what was taken given back": ", with three days' food to the household
    /// taken from and three to the common store", or "" for nothing more.
    pub fn words(&self) -> String {
        let days = |d: f32| {
            let d = f64::from(d);
            let n = match d.round() as i64 {
                1 => "one",
                2 => "two",
                3 => "three",
                4 => "four",
                5 => "five",
                6 => "six",
                7 => "seven",
                10 => "ten",
                _ => "",
            };
            let unit = if (d - 1.0).abs() < 1e-6 {
                "day's"
            } else {
                "days'"
            };
            if n.is_empty() || (d - d.round()).abs() > 1e-6 {
                format!("{d:.1} {unit}")
            } else {
                format!("{n} {unit}")
            }
        };
        let mut parts = Vec::new();
        if self.compensation_days > 0.0 {
            parts.push(format!(
                "{} food to the household taken from",
                days(self.compensation_days)
            ));
        }
        if self.fine_days > 0.0 {
            parts.push(format!("{} to the common store", days(self.fine_days)));
        }
        let with = match parts.len() {
            0 => String::new(),
            1 => format!(", with {}", parts[0]),
            _ => format!(", with {} and {}", parts[0], parts[1]),
        };
        if self.exile {
            format!("{with}, and is sent from the valley")
        } else {
            with
        }
    }
}

/// A policy template (content kind `policy`, ADR-0013 §3): what it does, the issues whose
/// presence makes proposing it a move, and the parameters a sponsor may put forward.
#[derive(Clone, Debug, PartialEq)]
pub struct PolicyDef {
    /// Content id: what laws refer to.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What it is, in a sentence.
    pub description: String,
    /// What the kernel does under it.
    pub kind: PolicyKind,
    /// The issues whose presence makes proposing it a move.
    pub answers: Vec<IssueKind>,
    /// The shares of threshed grain a sponsor may propose for the levy, smallest first (research
    /// 09-05 §2.3: a sponsor weighs three to eight levels).
    pub levy_shares: Vec<f64>,
    /// The most food one ask brings a household from the store, in days of its need.
    pub relief_days: f64,
    /// For a law against taking: the bundles a sponsor may propose, lightest first.
    pub bundles: Vec<Sanction>,
    /// For a curfew: the hours a sponsor may propose, each `(from, to)` hours of the day, the
    /// curfew running from the first to the second, past midnight when it is the smaller.
    pub hours: Vec<Hours>,
    /// For an amendment of the custom: the bodies a sponsor may propose (M4c slice AF).
    pub bodies: Vec<Body>,
    /// The question people take positions on, in words ("whether to keep a common store"), or
    /// none (M4c slice AG, ADR-0016 §4).
    pub question: Option<String>,
    /// How a law of it bears on each value content names: (index in the catalog's values, −1
    /// against to 1 for), in value order (M4c slice AG, step three).
    pub bears: Vec<(u16, f32)>,
}

/// Hours of the day a curfew runs, `(from, to)`: from the start of hour `from` to the start of
/// hour `to`, past midnight when `to` is not after `from`. `(0, 0)` for a law that sets none.
pub type Hours = (u8, u8);

/// Whether minute `minute_of_day` (0 to 1439) falls within `hours`.
pub fn within_hours(hours: Hours, minute_of_day: i64) -> bool {
    let (from, to) = (i64::from(hours.0) * 60, i64::from(hours.1) * 60);
    if from == to {
        return false;
    }
    let m = minute_of_day.rem_euclid(24 * 60);
    if from < to {
        (from..to).contains(&m)
    } else {
        m >= from || m < to
    }
}

/// How long `hours` run, hours.
pub fn hours_long(hours: Hours) -> u32 {
    let (from, to) = (u32::from(hours.0), u32::from(hours.1));
    if from == to { 0 } else { (to + 24 - from) % 24 }
}

/// How a polity's members meet, decide, forecast and comply (the people profile's `[polity]`
/// table; content API 31). Tuning values unless the content says otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct PolityParams {
    /// Days between a polity's routine reviews, when those who may propose look at its issues
    /// (research 09-01 §3.3: 7-30 days).
    pub review_days: u32,
    /// Days from a proposal to the gathering that decides it: time for word to go round.
    pub notice_days: u32,
    /// Minutes a gathering sits, from the start of the evening.
    pub gathering_minutes: u32,
    /// The founding custom: the share of the settlement's adults who must come for a gathering
    /// to decide anything (ADR-0013 §1).
    pub quorum_share: f64,
    /// Points per unit of a forecast gain (a change in the log of a year's food above
    /// subsistence).
    pub w_gain: f64,
    /// Points for a member's full regard for a law's sponsor.
    pub w_regard: f64,
    /// Points either way within which a member abstains.
    pub stance_margin: f64,
    /// Points for attending a gathering, before the stake.
    pub attend_base: f64,
    /// Points per point at stake in what a gathering decides.
    pub w_attend: f64,
    /// Weight of those who regard a sponsor in what the sponsor forecasts, against their own
    /// household's 1.
    pub w_followers: f64,
    /// Points a proposal costs its sponsor: the outside option is worth this much more.
    pub propose_cost: f64,
    /// Temperature of the choice among moves, points.
    pub temperature: f64,
    /// Days a member remembers how a gathering they came to stood on a proposal: while they do,
    /// they expect no more support for the same proposal than they saw it get (research 09-05
    /// §1.2: a sponsor weighs expected success).
    pub vote_memory_days: u32,
    /// The lean years a settlement is believed to have before it has seen any, out of
    /// `prior_years`.
    pub prior_lean: f64,
    /// Years of that prior belief.
    pub prior_years: f64,
    /// A lean year's harvest, as a share of an ordinary one.
    pub lean_harvest: f64,
    /// The share of what outsiders take at places its people work that a household believes a
    /// claim on those places keeps for it (M5c slice AT, ADR-0020 §5): a design prior, since a
    /// claim is only words until outsiders heed it.
    pub claim_keeps: f64,
    /// The share of a year's food below which a household cannot live: food there is worth the
    /// most, and a household that would fall below it cannot pay a levy.
    pub subsistence_share: f64,
    /// Points for paying what a law asks, before its cost and apart from the norms each person
    /// holds (content API 42: the custom that the gathering binds is a norm, `core:norm/
    /// gathering_binds`, held by each person their own way; before it, this was 1.5 for all).
    pub comply_base: f64,
    /// Points for paying per unit of the stance a person took on the law (1 for, −1 against, 0
    /// for abstaining or absent).
    pub w_stance: f64,
    /// Days after the custom is taken from the gathering in which its new body's members weigh
    /// ending each law the old custom made (M4c slice AI, step two; research 09-11 §1.6). What
    /// they leave stands.
    pub founding_days: u32,
}

impl PolityParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> PolityParams {
        PolityParams {
            review_days: 7,
            notice_days: 1,
            gathering_minutes: 120,
            quorum_share: 0.25,
            w_gain: 10.0,
            w_regard: 1.0,
            stance_margin: 0.25,
            attend_base: 1.0,
            w_attend: 0.5,
            w_followers: 0.5,
            propose_cost: 0.5,
            temperature: 0.5,
            vote_memory_days: 365,
            prior_lean: 1.0,
            prior_years: 4.0,
            lean_harvest: 0.5,
            claim_keeps: 0.5,
            subsistence_share: 0.5,
            comply_base: 0.0,
            w_stance: 1.0,
            founding_days: 90,
        }
    }
}

/// Who a body's members are (ADR-0013 §2). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Membership {
    /// Every adult who lives in the settlement.
    Adults,
    /// The elder of each household: its eldest of an age to keep one (M4c slice AF; research
    /// 09-05 §1.4: aggregation by household).
    Elders,
    /// The adults of households that hold land (M4c slice AF): under a regime where the
    /// settlement holds the fields, nobody.
    Landholders,
    /// Those who keep the watch by a law in force (M4c slice AI, step three): the body a coup
    /// among them makes. When none keeps it, nobody.
    Watch,
}

impl Membership {
    /// Every membership, in code order.
    pub const ALL: [Membership; 4] = [
        Membership::Adults,
        Membership::Elders,
        Membership::Landholders,
        Membership::Watch,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The membership numbered `code`.
    pub fn from_code(code: u8) -> Option<Membership> {
        Membership::ALL.get(usize::from(code)).copied()
    }

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            Membership::Adults => "adults",
            Membership::Elders => "elders",
            Membership::Landholders => "landholders",
            Membership::Watch => "watch",
        }
    }

    /// The membership with an authored name.
    pub fn from_name(name: &str) -> Option<Membership> {
        Membership::ALL.into_iter().find(|m| m.name() == name)
    }

    /// Its members, in a count: "24 adults", "5 watchers".
    pub fn noun(self) -> &'static str {
        match self {
            Membership::Adults => "adults",
            Membership::Elders => "elders",
            Membership::Landholders => "landholders",
            Membership::Watch => "watchers",
        }
    }
}

/// How a body decides (ADR-0013 §2; research 09-05 §1.4). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassRule {
    /// Acclamation: more of those present for than against; a tie fails.
    MoreForThanAgainst,
    /// Two of every three who take a side must be for it; a tie fails (M4c slice AF; research
    /// 09-05 §1.4: a supermajority threshold).
    TwoThirds,
}

impl PassRule {
    /// Every rule, in code order.
    pub const ALL: [PassRule; 2] = [PassRule::MoreForThanAgainst, PassRule::TwoThirds];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The rule numbered `code`.
    pub fn from_code(code: u8) -> Option<PassRule> {
        PassRule::ALL.get(usize::from(code)).copied()
    }

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            PassRule::MoreForThanAgainst => "more_for",
            PassRule::TwoThirds => "two_thirds",
        }
    }

    /// The rule with an authored name.
    pub fn from_name(name: &str) -> Option<PassRule> {
        PassRule::ALL.into_iter().find(|r| r.name() == name)
    }
}

/// A deciding body, as values: who belongs, how many must come, and how it decides. Saved with
/// the polity, so an amendment (later) changes the polity's own body, never the content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// Who belongs.
    pub members: Membership,
    /// The share of members who must be present.
    pub quorum_share: f32,
    /// How those present decide.
    pub pass: PassRule,
}

impl Body {
    /// The founding custom every world starts from (ADR-0013 §1): a gathering of the adults,
    /// deciding by acclamation.
    pub fn gathering(params: &PolityParams) -> Body {
        Body {
            members: Membership::Adults,
            quorum_share: params.quorum_share as f32,
            pass: PassRule::MoreForThanAgainst,
        }
    }

    /// The body in words: "The adults who come to the hearth decide by acclamation: a quarter of
    /// them must come, more for than against carries it, and a tie fails."
    pub fn words(&self) -> String {
        let who = match self.members {
            Membership::Adults => "The adults who come to the hearth",
            Membership::Elders => "The elders of its households who come to the hearth",
            Membership::Landholders => {
                "The adults of households that hold land who come to the hearth"
            }
            Membership::Watch => "Those who keep the watch",
        };
        let how = match self.pass {
            PassRule::MoreForThanAgainst => {
                "decide by acclamation: more for than against carries it, and a tie fails"
            }
            PassRule::TwoThirds => {
                "decide by acclamation: two of every three who take a side must be for it"
            }
        };
        let share = f64::from(self.quorum_share);
        let quorum = match (share * 100.0).round() as i64 {
            0 => "anyone who comes may decide".to_owned(),
            25 => "a quarter of them must come".to_owned(),
            33 => "a third of them must come".to_owned(),
            50 => "half of them must come".to_owned(),
            pct => format!("{pct} in a hundred of them must come"),
        };
        format!("{who} {how}; {quorum}.")
    }

    /// The body as a clause to follow "so that" or a colon: "the elders of its households who
    /// come to the hearth decide by acclamation: ...; half of them must come".
    pub fn clause(&self) -> String {
        let w = self.words();
        let w = w.trim_end_matches('.');
        let mut c = w.chars();
        match c.next() {
            Some(f) => f.to_lowercase().chain(c).collect(),
            None => String::new(),
        }
    }

    /// How many must come of `eligible` members: at least one.
    pub fn quorum(&self, eligible: u32) -> u32 {
        ((f64::from(self.quorum_share) * f64::from(eligible)).ceil() as u32).max(1)
    }

    /// What the body decides with `present` of `eligible` members there, `support` for and
    /// `oppose` against. A failure is recorded as what it is, never repaired (ADR-0013 §2).
    pub fn decide(&self, eligible: u32, present: u32, support: u32, oppose: u32) -> Outcome {
        if present < self.quorum(eligible) {
            return Outcome::NoQuorum;
        }
        if support == oppose {
            return Outcome::Tied;
        }
        let passes = match self.pass {
            PassRule::MoreForThanAgainst => support > oppose,
            PassRule::TwoThirds => 3 * support >= 2 * (support + oppose),
        };
        if passes {
            Outcome::Passed
        } else {
            Outcome::Failed
        }
    }
}

/// Where a law stands. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LawStatus {
    /// Before the gathering that decides it.
    Proposed,
    /// Decided and in force.
    InForce,
    /// Not passed.
    Rejected,
    /// In force until the one it named died or left.
    Lapsed,
    /// Replaced by a later law: an amendment of the custom by a later one (M4c slice AF), or a
    /// law by one that ends or changes it (M4c slice AH).
    Superseded,
    /// A repeal that passed (M4c slice AI, step two): the law it named is superseded, and the
    /// repeal itself asks and gives nothing.
    Carried,
}

/// How a gathering's decision went. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// More were for than against.
    Passed,
    /// More were against.
    Failed,
    /// As many for as against.
    Tied,
    /// Too few came.
    NoQuorum,
}

impl Outcome {
    /// In words, after "the gathering": "passed it", "turned it down".
    pub fn words(self) -> &'static str {
        match self {
            Outcome::Passed => "passed it",
            Outcome::Failed => "turned it down",
            Outcome::Tied => "was evenly split, so it failed",
            Outcome::NoQuorum => "was too thin to decide",
        }
    }
}

/// A member's stance on a proposal (research 09-05 §1.4). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    /// For it.
    Support,
    /// Against it.
    Oppose,
    /// Neither.
    Abstain,
}

impl Stance {
    /// +1 for, −1 against, 0 otherwise.
    pub fn sign(self) -> f64 {
        match self {
            Stance::Support => 1.0,
            Stance::Oppose => -1.0,
            Stance::Abstain => 0.0,
        }
    }
}

/// One member's stance at the gathering, with what moved it (ADR-0013 §3, stage 2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StanceRecord {
    /// Who.
    pub person: PermanentId,
    /// Their household.
    pub household: PermanentId,
    /// Where they stood.
    pub stance: Stance,
    /// Their household's forecast, points ([`PolityParams::w_gain`] times the gain).
    pub gain: f32,
    /// What their regard for the sponsor added, points.
    pub regard: f32,
    /// What talk at the hearth had moved them from their household's own lot, points (M4c slice
    /// AG; 0 before).
    pub opinion: f32,
    /// What the law does to what they hold dear, points (M4c slice AG step three; 0 before).
    pub values: f32,
}

/// What became of what a law asks of people and gives them (ADR-0013 §3, stages 6-7).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Compliance {
    /// Levies paid.
    pub complied: u32,
    /// Levies a household could not pay without falling below subsistence.
    pub could_not: u32,
    /// Levies refused.
    pub evaded: u32,
    /// Grain threshed by someone who did not know the law.
    pub unaware: u32,
    /// Grain paid in, kilograms.
    pub levied_kg: f64,
    /// Grain owed and kept back (could not, evaded and refused), kilograms.
    pub withheld_kg: f64,
    /// Levies kept back openly in a faction's refusal (M4c slice AH), and the grain, kilograms
    /// (counted in `withheld_kg` too).
    pub refused: u32,
    pub refused_kg: f64,
    /// Asks the store answered.
    pub relieved: u32,
    /// Food the store gave, kilograms.
    pub relief_kg: f64,
    /// Asks the store could not answer: it held nothing to give.
    pub unanswered: u32,
    /// For a prohibition: times someone who knew of it did what it forbids (M4b slice AD).
    pub broken: u32,
    /// Times someone who did not know of it did what it forbids.
    pub broken_unaware: u32,
}

/// What a watch has done (M4b slice AC): what anyone could see of it, and where its next round
/// begins. Its hours on rounds are the polity's capacity to guard, a reported measure that nothing
/// reads (12-04 §2.4).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WatchRecord {
    /// Rounds walked.
    pub rounds: u32,
    /// Minutes on rounds, walking and standing watch.
    pub minutes: f64,
    /// Cases the watch brought before the gathering.
    pub cases: u32,
    /// The night (the day index of its evening) the latest round was walked.
    pub night: i64,
    /// Rounds walked that night.
    pub tonight: u8,
    /// Where the next round begins: an index into the settlement's households in id order.
    pub next: u32,
}

/// A law and its whole history (ADR-0013 §3): kept for ever, never pruned.
#[derive(Clone, Debug, PartialEq)]
pub struct Law {
    /// Permanent id.
    pub id: PermanentId,
    /// Its template, by index in the catalog's policies.
    pub policy: u16,
    /// What its template does: set from the catalog when it is proposed or loaded, not saved.
    pub kind: PolicyKind,
    /// The levy share proposed (0 for a law that levies nothing).
    pub levy_share: f32,
    /// The one it names, for a law that names someone (who keeps the store).
    pub holder: Option<PermanentId>,
    /// The most one ask brings, days of the asking household's food.
    pub relief_days: f32,
    /// For a law against taking: what it adds to giving back what was taken.
    pub sanction: Sanction,
    /// For a curfew: the hours it runs (`(0, 0)` for other laws).
    pub hours: Hours,
    /// Where it stands.
    pub status: LawStatus,
    /// Stage 1: who proposed it, when, and the issue it answered.
    pub sponsor: PermanentId,
    /// When it was proposed.
    pub proposed: SimTime,
    /// The issue it answered.
    pub issue: IssueKind,
    /// The day the gathering meets on it.
    pub meets_day: i64,
    /// Stage 3: when it was decided, and how.
    pub decided: Option<SimTime>,
    /// How the gathering went.
    pub outcome: Option<Outcome>,
    /// Members at the decision.
    pub eligible: u32,
    /// Stage 2: the stances of those present, in id order.
    pub stances: Vec<StanceRecord>,
    /// Stage 4: who knows it and the day each learnt it, by person id.
    pub known: Vec<(PermanentId, i64)>,
    /// Stages 6-7: what people did, and what moved.
    pub compliance: Compliance,
    /// For a watch: what it has done (M4b slice AC).
    pub watch: WatchRecord,
    /// For an amendment of the custom: the body it would make (M4c slice AF).
    pub body: Option<Body>,
    /// The law in force it would replace, if any (M4c slice AH): a common store at another
    /// share, or at none, in place of the one in force.
    pub ends: Option<PermanentId>,
    /// For the polity's side of an agreement: which (M5c slice AU).
    pub agreement: Option<PermanentId>,
}

impl Law {
    /// Whether `person` knows the law.
    pub fn knows(&self, person: PermanentId) -> bool {
        self.known.binary_search_by_key(&person, |k| k.0).is_ok()
    }

    /// `person` learns the law on `day`, if they did not know it. Returns whether they learnt it.
    pub fn learn(&mut self, person: PermanentId, day: i64) -> bool {
        match self.known.binary_search_by_key(&person, |k| k.0) {
            Ok(_) => false,
            Err(at) => {
                self.known.insert(at, (person, day));
                true
            }
        }
    }

    /// Those present, those for and those against.
    pub fn counts(&self) -> (u32, u32, u32) {
        let n = |s: Stance| self.stances.iter().filter(|r| r.stance == s).count() as u32;
        (
            self.stances.len() as u32,
            n(Stance::Support),
            n(Stance::Oppose),
        )
    }
}

/// A gathering called on a proposal or a case: the law before it, the day it meets, what each
/// household made of the law when word went round, the cases it is to hear, and who came.
#[derive(Clone, Debug, PartialEq)]
pub struct Gathering {
    /// The law before it, if it was called on one.
    pub law: Option<PermanentId>,
    /// The day it meets.
    pub day: i64,
    /// Each household's forecast of the law when it was proposed, points
    /// ([`PolityParams::w_gain`] times [`store_gain`]), by household id.
    pub stakes: Vec<(PermanentId, f32)>,
    /// Those who came, in id order.
    pub present: Vec<PermanentId>,
    /// The cases brought before it, by number, in the order brought (M4b slice AB).
    pub cases: Vec<u32>,
}

impl Gathering {
    /// What household `household` made of the law, points; 0 for one that formed since.
    pub fn stake(&self, household: PermanentId) -> f64 {
        self.stakes
            .binary_search_by_key(&household, |s| s.0)
            .map_or(0.0, |i| f64::from(self.stakes[i].1))
    }
}

/// Why a member stood where they did, in words: "their household stands to gain, and they think
/// well of the sponsor"; for `sponsor`, "they proposed it". `margin` is the stance margin, points.
pub fn stance_words(r: &StanceRecord, margin: f64, sponsor: PermanentId) -> String {
    if r.person == sponsor {
        return "they proposed it".to_owned();
    }
    let (gain, regard) = (f64::from(r.gain), f64::from(r.regard));
    let (talk, values) = (f64::from(r.opinion), f64::from(r.values));
    let held = if values > margin {
        "; what they hold dear drew them toward it"
    } else if values < -margin {
        "; what they hold dear turned them against it"
    } else {
        ""
    };
    let talked = if talk > margin {
        "; talk at the hearth had drawn them toward it"
    } else if talk < -margin {
        "; talk at the hearth had turned them against it"
    } else {
        ""
    };
    let household = if gain > margin {
        "their household stands to gain"
    } else if gain < -margin {
        "their household stands to lose"
    } else {
        "it makes little difference to their household"
    };
    let why = match r.stance {
        Stance::Support if gain <= margin && regard > 0.0 => {
            format!("{household}, but they think well of the sponsor")
        }
        Stance::Support if regard > 0.0 => {
            format!("{household}, and they think well of the sponsor")
        }
        Stance::Oppose | Stance::Abstain if regard > 0.0 && gain < -margin => {
            format!("{household}, though they think well of the sponsor")
        }
        Stance::Oppose | Stance::Abstain if regard < 0.0 => {
            format!("{household}, and they think better of the one it would replace")
        }
        _ => household.to_owned(),
    };
    format!("{why}{held}{talked}")
}

/// What a law is, in words, to follow "proposed", "agreed to" or "turned down": "a common store,
/// taking a tenth of each harvest", "Ada as keeper of the common store". `name_of` names people.
pub fn law_words(
    law: &Law,
    policies: &[PolicyDef],
    name_of: &dyn Fn(PermanentId) -> String,
) -> String {
    let def = policies.get(usize::from(law.policy));
    if let Some(body) = law
        .body
        .filter(|_| def.is_some_and(|d| d.kind == PolicyKind::AmendBody))
    {
        return format!("a change of the custom: {}", body.clause());
    }
    // A repeal names the law it ends; without that law at hand (see [`Polity::words_of`]), only
    // that it ends one.
    if def.is_some_and(|d| d.kind == PolicyKind::Repeal) {
        return "an end to a law in force".to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::ClaimPlace) {
        return "a claim on the places where people of another settlement worked: they may use \
                them only by leave"
            .to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::Agreement) {
        return "an agreement with a neighbouring polity".to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::AgainstTaking) {
        return format!(
            "a law against taking: whoever is found to have taken from another household's \
             store gives back what was taken{}",
            law.sanction.words()
        );
    }
    // An office's holder named in place of the one in it (M4c slice AH).
    let instead = if law.ends.is_some() {
        " in place of the one holding it"
    } else {
        ""
    };
    if def.is_some_and(|d| d.kind == PolicyKind::KeepStore) {
        return match law.holder {
            Some(h) => format!("{} as keeper of the common store{instead}", name_of(h)),
            None => "someone as keeper of the common store".to_owned(),
        };
    }
    if def.is_some_and(|d| d.kind == PolicyKind::KeepWatch) {
        return match law.holder {
            Some(h) => format!(
                "{} to keep watch over the stores at night{instead}",
                name_of(h)
            ),
            None => "someone to keep watch over the stores at night".to_owned(),
        };
    }
    if def.is_some_and(|d| d.kind == PolicyKind::Curfew) {
        return format!(
            "a curfew: nobody may be away from home from {}:00 to {}:00, but the watch at its \
             rounds and those at a gathering",
            law.hours.0, law.hours.1
        );
    }
    if law.ends.is_some() {
        return replacing_words(def, law.levy_share);
    }
    format!(
        "{}, taking {} of each harvest",
        policy_name(def),
        share_text(f64::from(law.levy_share))
    )
}

/// What ending law `ended` ends, in words, after "an end to" (M4c slice AI, step two): "Ada's
/// keeping of the common store", "the curfew from 21:00 to 5:00". `name_of` names people.
pub fn ending_words(
    ended: &Law,
    policies: &[PolicyDef],
    name_of: &dyn Fn(PermanentId) -> String,
) -> String {
    let who = |what: &str| match ended.holder {
        Some(h) => format!("{}'s {what}", name_of(h)),
        None => format!("the {what}"),
    };
    match policies.get(usize::from(ended.policy)).map(|d| d.kind) {
        Some(PolicyKind::CommonStore) => "the common store's levy".to_owned(),
        Some(PolicyKind::KeepStore) => who("keeping of the common store"),
        Some(PolicyKind::KeepWatch) => who("watch over the stores at night"),
        Some(PolicyKind::AgainstTaking) => "the law against taking".to_owned(),
        Some(PolicyKind::Curfew) => format!(
            "the curfew from {}:00 to {}:00",
            ended.hours.0, ended.hours.1
        ),
        Some(PolicyKind::AmendBody) => "a change of the custom".to_owned(),
        Some(PolicyKind::ClaimPlace) => "the claim on places outsiders worked".to_owned(),
        Some(PolicyKind::Agreement) => "the agreement with a neighbouring polity".to_owned(),
        Some(PolicyKind::Repeal) | None => "a law".to_owned(),
    }
}

/// A policy's name with its article: "a common store".
fn policy_name(def: Option<&PolicyDef>) -> String {
    def.map_or_else(
        || "a law".to_owned(),
        |d| {
            let name = d.name.to_lowercase();
            let article = match name.chars().next() {
                Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
                _ => "a",
            };
            format!("{article} {name}")
        },
    )
}

/// A store of template `def` that replaces the one in force (M4c slice AH: a petition's demand),
/// at `levy_share`, or at none: "an end to the common store's levy (the store gives what it
/// holds)"; "a common store, taking a twentieth of each harvest in place of the levy in force".
pub fn replacing_words(def: Option<&PolicyDef>, levy_share: f32) -> String {
    if levy_share <= 0.0 {
        return "an end to the common store's levy (the store gives what it holds)".to_owned();
    }
    format!(
        "{}, taking {} of each harvest in place of the levy in force",
        policy_name(def),
        share_text(f64::from(levy_share))
    )
}

/// A day in words: "3 May of year 2".
pub fn day_words(t: SimTime) -> String {
    let date = t.date();
    let month = civ_land::weather::MONTH_NAMES[usize::from(date.month.clamp(1, 12)) - 1];
    format!("{} {month} of year {}", date.day, date.year)
}

/// Every version of a polity's custom, oldest first, in words (ADR-0017 §1): "Since 3 May of
/// year 2, the founding custom: the adults who come to the hearth decide …", then "Since 9 June
/// of year 5, by the amendment Ada proposed: the elders of its households …". `name_of` names
/// people.
pub fn custom_history_words(
    polity: &Polity,
    name_of: &dyn Fn(PermanentId) -> String,
) -> Vec<String> {
    polity
        .versions
        .iter()
        .map(|v| {
            let since = day_words(v.since);
            if let Some(who) = v.seized_by {
                return format!(
                    "Since {since}, taken from the gathering by those who stood with {}, not by \
                     its procedure: {}.",
                    name_of(who),
                    v.body.clause()
                );
            }
            match v.law.and_then(|id| polity.laws.iter().find(|l| l.id == id)) {
                Some(l) => format!(
                    "Since {since}, by the amendment {} proposed: {}.",
                    name_of(l.sponsor),
                    v.body.clause()
                ),
                None => format!("Since {since}, the founding custom: {}.", v.body.clause()),
            }
        })
        .collect()
}

/// A polity's offices, in words: "Storekeeper: Ada, since 3 May of year 2", or once its holder is
/// gone and nobody named since, "Storekeeper: vacant since 9 June of year 5, when Ada died". A
/// watch adds what anyone could see of it: "Watch: Bram, since 3 May of year 2; 14 rounds, 30
/// hours on them, 2 cases brought". `gone` says how someone left ("died", "left the valley").
pub fn office_words(
    polity: &Polity,
    policies: &[PolicyDef],
    name_of: &dyn Fn(PermanentId) -> String,
    gone: &dyn Fn(PermanentId) -> String,
) -> Vec<String> {
    let mut out = Vec::new();
    for (k, def) in policies.iter().enumerate() {
        if !matches!(def.kind, PolicyKind::KeepStore | PolicyKind::KeepWatch) {
            continue;
        }
        // The last one named, still holding it or since gone.
        let last = polity.laws.iter().rfind(|l| {
            usize::from(l.policy) == k
                && l.holder.is_some()
                && matches!(l.status, LawStatus::InForce | LawStatus::Lapsed)
        });
        let Some(law) = last else {
            continue;
        };
        let Some(holder) = law.holder else {
            continue;
        };
        let since = law.decided.map(day_words).unwrap_or_default();
        let w = &law.watch;
        let record = if def.kind == PolicyKind::KeepWatch {
            let cases = match w.cases {
                1 => "1 case brought".to_owned(),
                n => format!("{n} cases brought"),
            };
            format!(
                "; {} round{}, {:.0} hours on them, {cases}",
                w.rounds,
                if w.rounds == 1 { "" } else { "s" },
                w.minutes / 60.0
            )
        } else {
            String::new()
        };
        out.push(match law.status {
            LawStatus::InForce => {
                format!("{}: {}, since {since}{record}", def.name, name_of(holder))
            }
            _ => format!(
                "{}: vacant, since {} {}",
                def.name,
                name_of(holder),
                gone(holder)
            ),
        });
    }
    out
}

/// How the gathering decided a law, in words: "agreed: 20 for, 3 against; 23 of 24 adults came,
/// 6 needed". `None` before it has.
pub fn decision_words(law: &Law, body: &Body) -> Option<String> {
    let outcome = law.outcome?;
    let (present, support, oppose) = law.counts();
    let came = format!(
        "{present} of {} {} came, {} needed",
        law.eligible,
        body.members.noun(),
        body.quorum(law.eligible)
    );
    Some(match outcome {
        Outcome::Passed => format!("agreed: {support} for, {oppose} against; {came}"),
        Outcome::Failed => format!("turned down: {support} for, {oppose} against; {came}"),
        Outcome::Tied => {
            format!("evenly split, so it failed: {support} for, {oppose} against; {came}")
        }
        Outcome::NoQuorum => format!("too few came to decide: {came}"),
    })
}

/// A levy share in words: "a tenth".
pub fn share_text(share: f64) -> String {
    match (share * 100.0).round() as i64 {
        5 => "a twentieth".to_owned(),
        10 => "a tenth".to_owned(),
        20 => "a fifth".to_owned(),
        25 => "a quarter".to_owned(),
        33 => "a third".to_owned(),
        50 => "half".to_owned(),
        pct => format!("{pct} parts in a hundred"),
    }
}

/// A settlement's polity (ADR-0013 §1): its own identity, its body, its laws and its store.
#[derive(Clone, Debug, PartialEq)]
pub struct Polity {
    /// Permanent id, apart from the settlement's.
    pub id: PermanentId,
    /// The settlement whose residents are its members.
    pub settlement: PermanentId,
    /// When it was founded.
    pub founded: SimTime,
    /// The body that decides.
    pub body: Body,
    /// The common store, by good (ADR-0013 §4): kept at the hearth, under no roof yet.
    pub stores: Vec<f64>,
    /// When the store was last brought up to date.
    pub stores_at: SimTime,
    /// What came into and went out of the store, and what it lost.
    pub flows: Flows,
    /// Every law proposed here, oldest first.
    pub laws: Vec<Law>,
    /// The gathering called, if any.
    pub gathering: Option<Gathering>,
    /// The day of its last routine review.
    pub reviewed: i64,
    /// Every version of its custom, oldest first: the founding custom, then each amendment
    /// (ADR-0017 §1). The last is `body`.
    pub versions: Vec<CustomVersion>,
    /// The places its claims name, each with the law that names it (M5c slice AT, ADR-0020 §5):
    /// recorded when the claim is proposed, and claimed while that law is in force.
    pub claimed: Vec<(PermanentId, crate::uses::Place)>,
}

/// A version of a polity's custom (ADR-0017 §1): its body, since when, and the law that made it
/// (none for the founding custom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomVersion {
    /// The body.
    pub body: Body,
    /// When it began.
    pub since: SimTime,
    /// The amendment that made it.
    pub law: Option<PermanentId>,
    /// For a custom taken from the gathering rather than amended (M4c slice AI): who called those
    /// who stood with it.
    pub seized_by: Option<PermanentId>,
}

impl Polity {
    /// The body that decided what it decided at `t`: the version of the custom in force before
    /// then (one that began at `t` was made by what was decided then, under the one before).
    pub fn body_at(&self, t: SimTime) -> &Body {
        self.versions
            .iter()
            .rev()
            .find(|v| v.since < t)
            .or_else(|| self.versions.first())
            .map_or(&self.body, |v| &v.body)
    }

    /// A new polity for `settlement` under the founding custom.
    pub fn found(
        id: PermanentId,
        settlement: PermanentId,
        now: SimTime,
        params: &PolityParams,
    ) -> Polity {
        Polity {
            id,
            settlement,
            founded: now,
            body: Body::gathering(params),
            stores: Vec::new(),
            stores_at: now,
            flows: Flows::default(),
            laws: Vec::new(),
            gathering: None,
            reviewed: now.day_index(),
            versions: vec![CustomVersion {
                body: Body::gathering(params),
                since: now,
                law: None,
                seized_by: None,
            }],
            claimed: Vec::new(),
        }
    }

    /// The law in force that claims `place`, if any (the first made; M5c slice AT).
    pub fn claim_on(&self, place: crate::uses::Place) -> Option<PermanentId> {
        self.claimed
            .iter()
            .filter(|&&(_, p)| p == place)
            .map(|&(law, _)| law)
            .find(|&law| {
                self.laws
                    .iter()
                    .any(|l| l.id == law && l.status == LawStatus::InForce)
            })
    }

    /// The places it claims now: those named by its claims in force (M5c slice AT), in place
    /// order.
    pub fn claims_now(&self) -> Vec<crate::uses::Place> {
        let mut out: Vec<crate::uses::Place> = self
            .claimed
            .iter()
            .filter(|(law, _)| {
                self.laws
                    .iter()
                    .any(|l| l.id == *law && l.status == LawStatus::InForce)
            })
            .map(|&(_, p)| p)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The law before the gathering, if any.
    pub fn agenda(&self) -> Option<&Law> {
        self.laws.iter().find(|l| l.status == LawStatus::Proposed)
    }

    /// The one who keeps the store under their roof by a law in force, and that law.
    pub fn keeper(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::KeepStore)
    }

    /// The one who keeps watch by a law in force, and that law (M4b slice AC): the first, when
    /// several do.
    pub fn watcher(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::KeepWatch)
    }

    /// Everyone who keeps watch by a law in force, with their laws, oldest law first (M4c slice
    /// AI: a watch of several).
    pub fn watchers(&self) -> impl Iterator<Item = (PermanentId, &Law)> {
        self.laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::KeepWatch)
            .filter_map(|l| Some((l.holder?, l)))
    }

    /// The holder of the office a law of kind `kind` in force names, and that law.
    fn office(&self, kind: PolicyKind) -> Option<(PermanentId, &Law)> {
        self.laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == kind)
            .find_map(|l| Some((l.holder?, l)))
    }

    /// The laws in force of kind `kind` (by the catalog's policies).
    pub fn in_force<'a>(
        &'a self,
        policies: &'a [PolicyDef],
        kind: PolicyKind,
    ) -> impl Iterator<Item = &'a Law> + 'a {
        self.laws.iter().filter(move |l| {
            l.status == LawStatus::InForce
                && policies
                    .get(usize::from(l.policy))
                    .is_some_and(|d| d.kind == kind)
        })
    }

    /// Law `law` in words, as [`law_words`] says it, but a repeal naming the law of this polity
    /// it ends (M4c slice AI, step two): "an end to Ada's keeping of the common store".
    pub fn words_of(
        &self,
        law: &Law,
        policies: &[PolicyDef],
        name_of: &dyn Fn(PermanentId) -> String,
    ) -> String {
        let kind = policies.get(usize::from(law.policy)).map(|d| d.kind);
        let repeal = kind == Some(PolicyKind::Repeal);
        // A claim says how many places it names (M5c slice AT).
        if kind == Some(PolicyKind::ClaimPlace) {
            let n = self.claimed.iter().filter(|c| c.0 == law.id).count();
            return format!(
                "a claim on {} where people of another settlement worked: they may use {} \
                 only by leave",
                if n == 1 {
                    "a place".to_owned()
                } else {
                    format!("{n} places")
                },
                if n == 1 { "it" } else { "them" }
            );
        }
        match law
            .ends
            .filter(|_| repeal)
            .and_then(|e| self.laws.iter().find(|l| l.id == e))
        {
            Some(ended) => format!("an end to {}", ending_words(ended, policies, name_of)),
            None => law_words(law, policies, name_of),
        }
    }

    /// The law with id `id`.
    pub fn law_mut(&mut self, id: PermanentId) -> Option<&mut Law> {
        self.laws.iter_mut().find(|l| l.id == id)
    }

    /// Spoils the store up to `t` as anyone's store spoils, under a roof when `sheltered`.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[crate::params::GoodDef], sheltered: bool) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            let half_life = if sheltered && d.sheltered_half_life_days > 0.0 {
                d.sheltered_half_life_days
            } else {
                d.half_life_days
            };
            if half_life > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / half_life);
                self.flows.add(crate::person::Flow::Spoiled, g, *kg - after);
                *kg = after;
            }
        }
        self.stores_at = t;
    }
}

/// What a household expects of its coming year, in kcal: what it holds now, what its next
/// harvest should bring in an ordinary year, and a year's need.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outlook {
    /// Food held now, kcal.
    pub held: f64,
    /// Its next harvest in an ordinary year, kcal.
    pub harvest: f64,
    /// A year of its members' food, kcal.
    pub year_need: f64,
}

/// What a settlement's people believe about their years (ADR-0013 §5: forecasts from what they
/// have seen).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Belief {
    /// The chance a year is lean: the lean years the settlement has seen, with the prior.
    pub p_lean: f64,
    /// An ordinary harvest of one of its households, on average, kcal.
    pub mean_harvest: f64,
}

/// Below this a year's food (as a share of the need, above subsistence) counts as nearly
/// nothing: it keeps the log finite and the order of worse outcomes.
const FLOOR_SMOOTH: f64 = 0.05;

/// The chance a year is lean, believed by people who have seen `lean` lean years in `years`.
pub fn p_lean(lean: f64, years: f64, params: &PolityParams) -> f64 {
    ((lean + params.prior_lean) / (years + params.prior_years).max(1e-9)).clamp(0.0, 1.0)
}

/// How much better (or worse) a household expects its year under a common store with levy share
/// `levy` than without one: the change in the expected log of its year's food above subsistence,
/// over an ordinary year and a lean one (one whose harvest is `lean_harvest` of an ordinary
/// one). It pays the levy on its harvest; the store holds the levy on an ordinary harvest of the
/// average household, and gives back what brings it up to a year's need, as far as that goes.
/// Food near subsistence is worth the most, so a household that would go hungry in a lean year
/// gains, and one whose harvest is large loses (research 09-17 §1.2: households insure one
/// another, though a year lean for all can overwhelm it).
pub fn store_gain(o: &Outlook, b: &Belief, levy: f64, params: &PolityParams) -> f64 {
    let need = o.year_need.max(1.0);
    let u = |kcal: f64| ((kcal / need - params.subsistence_share).max(0.0) + FLOOR_SMOOTH).ln();
    let pool = (levy * b.mean_harvest).max(0.0);
    let year = |harvest: f64| {
        let before = o.held + harvest;
        let after = before - levy * harvest;
        let relief = (need - after).clamp(0.0, pool);
        u(after + relief) - u(before)
    };
    let p = b.p_lean.clamp(0.0, 1.0);
    (1.0 - p) * year(o.harvest) + p * year(o.harvest * params.lean_harvest)
}

/// How much better a household expects its year when `kcal` more food is kept for it, in the same
/// units as [`store_gain`]: what a store kept under a roof saves from spoiling, as its share.
pub fn kept_gain(o: &Outlook, kcal: f64, params: &PolityParams) -> f64 {
    let need = o.year_need.max(1.0);
    let u = |kcal: f64| ((kcal / need - params.subsistence_share).max(0.0) + FLOOR_SMOOTH).ln();
    let before = o.held + o.harvest;
    u(before + kcal.max(0.0)) - u(before)
}

/// What a household expects of a law against taking (M4b slice AB), in the units of
/// [`store_gain`]: `recover` kcal a year back to it (what it lost to takers it knows of in the past
/// year, with the bundle's compensation), less `owe` kcal a year from it (what its own members
/// took, with the bundle, at the chance they believe a taker runs of being seen).
pub fn against_gain(o: &Outlook, recover: f64, owe: f64, params: &PolityParams) -> f64 {
    kept_gain(o, recover, params) - levy_cost(o, owe, params)
}

/// What paying a levy of `kcal` now costs a household with outlook `o`, in the same units as
/// [`store_gain`]: the fall in the log of its year's food above subsistence.
pub fn levy_cost(o: &Outlook, kcal: f64, params: &PolityParams) -> f64 {
    let need = o.year_need.max(1.0);
    let u = |kcal: f64| ((kcal / need - params.subsistence_share).max(0.0) + FLOOR_SMOOTH).ln();
    let before = o.held + o.harvest;
    u(before) - u(before - kcal.max(0.0))
}

/// Whether paying a levy of `kcal` now would leave a household holding `held` kcal below
/// subsistence for a year of `year_need`: then it cannot pay (research 09-01 §6.4: being unable
/// is not refusing).
pub fn cannot_pay(held: f64, kcal: f64, year_need: f64, params: &PolityParams) -> bool {
    held - kcal < params.subsistence_share * year_need
}

/// A member's stance: their household's forecast in points, and their regard for the sponsor
/// (0-1), against the margin (research 09-05 §1.4; ADR-0013 §3, stage 2).
pub fn stance(gain_points: f64, regard: f64, params: &PolityParams) -> (Stance, f64) {
    stance_between(gain_points, regard, 0.0, params)
}

/// As [`stance`], for a law that would put someone in the place of an office's holder (M4c slice
/// AH: a petition against an office): regard for the one it names, 0 to 1, less what holds them
/// to the one it would replace, -1 to 1 (regard for them, less a grievance against them). Its
/// regard points may be below nothing.
pub fn stance_between(
    gain_points: f64,
    regard: f64,
    replaced: f64,
    params: &PolityParams,
) -> (Stance, f64) {
    let regard_points = params.w_regard * (regard.clamp(0.0, 1.0) - replaced.clamp(-1.0, 1.0));
    let s = gain_points + regard_points;
    let stance = if s > params.stance_margin {
        Stance::Support
    } else if s < -params.stance_margin {
        Stance::Oppose
    } else {
        Stance::Abstain
    };
    (stance, regard_points)
}

/// The points for paying a levy: `norm`, what the norms the person holds add for abiding by what
/// the gathering decided (M4c slice AG), the stance the person took, their regard for the law's
/// sponsor, less what paying costs their household. There is no sanction: nobody is set to see
/// who pays.
pub fn comply_points(
    norm: f64,
    stance: f64,
    regard: f64,
    cost_points: f64,
    params: &PolityParams,
) -> f64 {
    params.comply_base + norm + params.w_stance * stance + params.w_regard * regard.clamp(0.0, 1.0)
        - cost_points
}

/// The chance someone pays given [`comply_points`]: a logistic choice between paying and
/// keeping it back.
pub fn comply_chance(points: f64) -> f64 {
    1.0 / (1.0 + (-points).exp())
}

/// A move a person could make (ADR-0013 §5): proposing a policy at one level against an issue,
/// with what they forecast of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveOption {
    /// The template, by index in the catalog's policies.
    pub policy: u16,
    /// The levy share.
    pub levy_share: f64,
    /// The issue it answers.
    pub issue: IssueKind,
    /// The one it would name, for a law that names someone.
    pub nominee: Option<PermanentId>,
    /// The bundle, for a law against taking.
    pub sanction: Sanction,
    /// The hours, for a curfew.
    pub hours: Hours,
    /// The body, for an amendment of the custom (M4c slice AF).
    pub body: Option<Body>,
    /// The law in force it would replace, for one a faction petitions for (M4c slice AH).
    pub ends: Option<PermanentId>,
    /// The forecast for the person's own household ([`store_gain`]).
    pub own_gain: f64,
    /// The forecast for the households of those who regard them, weighted by that regard.
    pub followers_gain: f64,
    /// The share of those who would take a stance whom they expect to back it, from those they
    /// know.
    pub support: f64,
}

/// A deliberator's choice and its receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    /// The move made, by index in the options; `None` for making none.
    pub chosen: Option<usize>,
    /// Each option's score, points, in order; then the outside option's.
    pub scores: Vec<f64>,
}

/// How a person who may make institutional moves chooses among them (ADR-0013 §5): from a copy
/// of the facts and the moves available, a choice and its receipt. A rule-based one now; another
/// (an LLM's, plan §8) can stand in its place.
pub trait Deliberator {
    /// Chooses among `moves` or none, with `u` a keyed draw in 0-1.
    fn choose(&self, moves: &[MoveOption], u: f64) -> Choice;
}

/// The rule-based deliberator (ADR-0013 §5): one level deep. A move is worth what the person
/// forecasts for their household and those who regard them, by the chance it passes, less what
/// proposing costs; making none is worth nothing. A softmax picks one.
#[derive(Clone, Copy, Debug)]
pub struct RuleDeliberator<'a> {
    /// The tuning.
    pub params: &'a PolityParams,
}

impl RuleDeliberator<'_> {
    /// A move's score, points.
    pub fn score(&self, m: &MoveOption) -> f64 {
        let p = self.params;
        let gain = m.own_gain + p.w_followers * m.followers_gain;
        m.support.clamp(0.0, 1.0) * p.w_gain * gain - p.propose_cost
    }
}

impl Deliberator for RuleDeliberator<'_> {
    fn choose(&self, moves: &[MoveOption], u: f64) -> Choice {
        let mut scores: Vec<f64> = moves.iter().map(|m| self.score(m)).collect();
        scores.push(0.0);
        let t = self.params.temperature.max(1e-6);
        let scaled: Vec<f64> = scores.iter().map(|s| s / t).collect();
        let picked = crate::demography::pick_softmax(&scaled, u);
        Choice {
            chosen: picked.filter(|&i| i < moves.len()),
            scores,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn params() -> PolityParams {
        PolityParams::core()
    }

    fn pid(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn what_was_decided_is_told_under_the_body_that_decided_it() {
        let t = |day: i64| SimTime::from_minutes(day * 24 * 60);
        let mut polity = Polity::found(pid(1), pid(2), t(0), &params());
        let elders = Body {
            members: Membership::Elders,
            ..polity.body
        };
        // An amendment decided on day 100 makes the elders the body from then.
        polity.versions.push(CustomVersion {
            body: elders,
            since: t(100),
            law: Some(pid(9)),
            seized_by: None,
        });
        polity.body = elders;
        assert_eq!(polity.body_at(t(50)).members, Membership::Adults);
        assert_eq!(
            polity.body_at(t(100)).members,
            Membership::Adults,
            "the amendment itself was decided by the adults"
        );
        assert_eq!(polity.body_at(t(101)).members, Membership::Elders);
    }

    #[test]
    fn the_founding_custom_decides_by_acclamation_and_records_its_failures() {
        let body = Body::gathering(&params());
        assert_eq!(body.quorum(40), 10);
        assert_eq!(body.quorum(1), 1);
        assert_eq!(body.decide(40, 12, 7, 5), Outcome::Passed);
        assert_eq!(body.decide(40, 12, 5, 7), Outcome::Failed);
        assert_eq!(body.decide(40, 12, 6, 6), Outcome::Tied, "a tie fails");
        assert_eq!(
            body.decide(40, 9, 9, 0),
            Outcome::NoQuorum,
            "too few came, however they stood"
        );
    }

    #[test]
    fn pooling_helps_the_household_that_would_go_hungry_and_costs_the_one_with_plenty() {
        let p = params();
        let need = 365.0 * 4.0 * 2100.0;
        let belief = Belief {
            p_lean: 0.25,
            mean_harvest: 1.2 * need,
        };
        let poor = Outlook {
            held: 0.05 * need,
            harvest: 0.8 * need,
            year_need: need,
        };
        let rich = Outlook {
            held: 0.1 * need,
            harvest: 2.0 * need,
            year_need: need,
        };
        let gain = |o: &Outlook| store_gain(o, &belief, 0.1, &p);
        assert!(gain(&poor) > 0.1, "{}", gain(&poor));
        assert!(gain(&rich) < -0.05, "{}", gain(&rich));
        // No levy, no change.
        assert!(store_gain(&poor, &belief, 0.0, &p).abs() < 1e-12);
        // The more lean years seen, the more a middling household gains.
        let middling = Outlook {
            held: 0.1 * need,
            harvest: 1.2 * need,
            year_need: need,
        };
        let at = |p_lean| store_gain(&middling, &Belief { p_lean, ..belief }, 0.1, &p);
        assert!(at(0.5) > at(0.25) && at(0.25) > at(0.1));
    }

    #[test]
    fn lean_years_seen_raise_the_belief_from_its_prior() {
        let p = params();
        assert!((p_lean(0.0, 0.0, &p) - 0.25).abs() < 1e-12);
        assert!(p_lean(2.0, 4.0, &p) > 0.25);
        assert!(p_lean(0.0, 20.0, &p) < 0.1);
    }

    #[test]
    fn a_stance_weighs_the_forecast_and_regard_for_the_sponsor() {
        let p = params();
        assert_eq!(stance(1.0, 0.0, &p).0, Stance::Support);
        assert_eq!(stance(-1.0, 0.0, &p).0, Stance::Oppose);
        assert_eq!(stance(0.1, 0.0, &p).0, Stance::Abstain);
        // Regard for the sponsor carries one who would lose a little.
        let (s, r) = stance(-0.5, 1.0, &p);
        assert_eq!((s, r), (Stance::Support, 1.0));
        assert_eq!(stance(-0.5, 5.0, &p).1, 1.0, "regard counts up to 1");
        // Someone named in place of an office's holder (M4c slice AH): regard for the holder
        // weighs against it, and a grievance against them lowers that regard below nothing.
        assert_eq!(
            stance_between(0.0, 0.5, 0.5, &p),
            (Stance::Abstain, 0.0),
            "liked alike"
        );
        assert_eq!(stance_between(0.0, 0.0, 1.0, &p).0, Stance::Oppose);
        let (s, r) = stance_between(0.0, 0.5, 0.5 - 1.0, &p);
        assert_eq!((s, r), (Stance::Support, p.w_regard));
    }

    #[test]
    fn paying_is_likelier_for_those_who_backed_the_law_and_dearer_for_the_hard_pressed() {
        let p = params();
        let backed = comply_chance(comply_points(1.5, 1.0, 0.0, 0.5, &p));
        let opposed = comply_chance(comply_points(1.5, -1.0, 0.0, 0.5, &p));
        let pressed = comply_chance(comply_points(1.5, 1.0, 0.0, 3.0, &p));
        let unbound = comply_chance(comply_points(0.0, 1.0, 0.0, 0.5, &p));
        assert!(backed > opposed && backed > pressed && backed > unbound);
        assert!(cannot_pay(100.0, 60.0, 200.0, &p));
        assert!(!cannot_pay(200.0, 60.0, 200.0, &p));
    }

    #[test]
    fn a_law_keeps_who_knows_it_in_order() {
        let mut law = Law {
            id: pid(1),
            policy: 0,
            kind: PolicyKind::CommonStore,
            levy_share: 0.1,
            holder: None,
            relief_days: 5.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::Proposed,
            sponsor: pid(7),
            proposed: SimTime::ZERO,
            issue: IssueKind::FoodShort,
            meets_day: 1,
            decided: None,
            outcome: None,
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord::default(),
            body: None,
            ends: None,
            agreement: None,
        };
        assert!(law.learn(pid(9), 3));
        assert!(law.learn(pid(4), 5));
        assert!(!law.learn(pid(9), 6), "known already");
        assert_eq!(law.known, vec![(pid(4), 5), (pid(9), 3)]);
        assert!(law.knows(pid(4)) && !law.knows(pid(7)));
    }

    #[test]
    fn a_law_in_words_follows_the_gatherings_verbs() {
        let def = |kind: PolicyKind, name: &str| PolicyDef {
            id: format!("core:policy/{name}"),
            name: name.to_owned(),
            description: String::new(),
            kind,
            answers: Vec::new(),
            levy_shares: Vec::new(),
            relief_days: 0.0,
            bundles: Vec::new(),
            hours: Vec::new(),
            bodies: Vec::new(),
            question: None,
            bears: Vec::new(),
        };
        let policies = [
            def(PolicyKind::CommonStore, "Common store"),
            def(PolicyKind::KeepStore, "Storekeeper"),
            def(PolicyKind::AgainstTaking, "Against taking"),
            def(PolicyKind::KeepWatch, "Watch"),
        ];
        let mut law = Law {
            id: pid(1),
            policy: 0,
            kind: PolicyKind::CommonStore,
            levy_share: 0.1,
            holder: None,
            relief_days: 5.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::Proposed,
            sponsor: pid(7),
            proposed: SimTime::ZERO,
            issue: IssueKind::FoodShort,
            meets_day: 1,
            decided: None,
            outcome: None,
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord::default(),
            body: None,
            ends: None,
            agreement: None,
        };
        let name_of = |_: PermanentId| "Ada".to_owned();
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a common store, taking a tenth of each harvest"
        );
        // A petition's demand replaces the store in force (M4c slice AH).
        law.ends = Some(pid(2));
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a common store, taking a tenth of each harvest in place of the levy in force"
        );
        law.levy_share = 0.0;
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "an end to the common store's levy (the store gives what it holds)"
        );
        law.ends = None;
        law.policy = 1;
        law.holder = Some(pid(9));
        // "proposed …", "agreed to …" and "turned down …" all read.
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "Ada as keeper of the common store"
        );
        law.policy = 2;
        law.holder = None;
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a law against taking: whoever is found to have taken from another household's store \
             gives back what was taken"
        );
        law.sanction = Sanction {
            compensation_days: 3.0,
            fine_days: 1.0,
            exile: false,
        };
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a law against taking: whoever is found to have taken from another household's store \
             gives back what was taken, with three days' food to the household taken from and one \
             day's to the common store"
        );
        law.sanction = Sanction {
            exile: true,
            ..Default::default()
        };
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a law against taking: whoever is found to have taken from another household's store \
             gives back what was taken, and is sent from the valley"
        );
        law.policy = 3;
        law.holder = Some(pid(9));
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "Ada to keep watch over the stores at night"
        );
    }

    #[test]
    fn a_watch_office_reads_with_what_anyone_could_see_of_it() {
        let policies = [PolicyDef {
            id: "core:policy/keep_watch".to_owned(),
            name: "Watch".to_owned(),
            description: String::new(),
            kind: PolicyKind::KeepWatch,
            answers: vec![IssueKind::Takings],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            bundles: Vec::new(),
            hours: Vec::new(),
            bodies: Vec::new(),
            question: None,
            bears: Vec::new(),
        }];
        let mut polity = Polity::found(pid(1), pid(2), SimTime::ZERO, &params());
        polity.laws.push(Law {
            id: pid(3),
            policy: 0,
            kind: PolicyKind::KeepWatch,
            levy_share: 0.0,
            holder: Some(pid(9)),
            relief_days: 0.0,
            sanction: Sanction::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: pid(9),
            proposed: SimTime::ZERO,
            issue: IssueKind::Takings,
            meets_day: 0,
            decided: Some(SimTime::ZERO),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord {
                rounds: 14,
                minutes: 1800.0,
                cases: 1,
                ..Default::default()
            },
            body: None,
            ends: None,
            agreement: None,
        });
        assert_eq!(polity.watcher().map(|w| w.0), Some(pid(9)));
        assert!(polity.keeper().is_none(), "a watch keeps no store");
        let words = office_words(&polity, &policies, &|_| "Bram".to_owned(), &|_| {
            String::new()
        });
        assert_eq!(words.len(), 1);
        assert!(
            words[0].starts_with("Watch: Bram, since ")
                && words[0].ends_with("; 14 rounds, 30 hours on them, 1 case brought"),
            "{}",
            words[0]
        );
    }

    #[test]
    fn a_law_against_taking_helps_those_taken_from_and_costs_those_who_took() {
        let p = params();
        let need_day = 4.0 * 2100.0;
        let o = Outlook {
            held: 30.0 * need_day,
            harvest: 300.0 * need_day,
            year_need: 365.0 * need_day,
        };
        let bundle = Sanction {
            compensation_days: 3.0,
            fine_days: 3.0,
            exile: false,
        };
        let robbed = crate::crime::TakingsKnown {
            lost_kcal: 10.0 * need_day,
            lost: 2,
            risk: 0.5,
            ..Default::default()
        };
        let took = crate::crime::TakingsKnown {
            took_kcal: 10.0 * need_day,
            took: 2,
            risk: 0.5,
            ..Default::default()
        };
        let gain = |k: &crate::crime::TakingsKnown, s: &Sanction| {
            let (recover, owe) = k.under(s, need_day, 60.0);
            against_gain(&o, recover, owe, &p)
        };
        assert!(gain(&robbed, &bundle) > 0.0);
        assert!(gain(&took, &bundle) < 0.0);
        // A harsher bundle is worth more to the one taken from and costs the taker more.
        assert!(gain(&robbed, &bundle) > gain(&robbed, &Sanction::default()));
        assert!(gain(&took, &bundle) < gain(&took, &Sanction::default()));
        // Exile weighs on a taker's household as the days of food a grown member is worth to it,
        // and brings the one taken from nothing more.
        let exiled = Sanction {
            exile: true,
            ..bundle
        };
        assert!(gain(&took, &exiled) < gain(&took, &bundle));
        assert!((gain(&robbed, &exiled) - gain(&robbed, &bundle)).abs() < 1e-12);
        // A taker sure of never being seen expects to owe nothing.
        let unseen = crate::crime::TakingsKnown { risk: 0.0, ..took };
        assert!(gain(&unseen, &bundle).abs() < 1e-12);
        // A household with no takings either way is untouched.
        let none = crate::crime::TakingsKnown {
            risk: 0.5,
            ..Default::default()
        };
        assert!(gain(&none, &bundle).abs() < 1e-12);
    }

    #[test]
    fn a_curfews_hours_run_past_midnight_when_they_end_before_they_begin() {
        let h = |hour: i64| hour * 60;
        assert!(within_hours((21, 5), h(21)));
        assert!(within_hours((21, 5), h(23) + 59));
        assert!(within_hours((21, 5), h(0)));
        assert!(within_hours((21, 5), h(4) + 59));
        assert!(!within_hours((21, 5), h(5)));
        assert!(!within_hours((21, 5), h(20) + 59));
        assert!(within_hours((9, 17), h(12)) && !within_hours((9, 17), h(17)));
        // No hours, no curfew.
        assert!(!within_hours((0, 0), h(3)));
        assert_eq!(hours_long((21, 5)), 8);
        assert_eq!(hours_long((23, 4)), 5);
        assert_eq!(hours_long((9, 17)), 8);
        assert_eq!(hours_long((0, 0)), 0);
    }

    #[test]
    fn a_two_thirds_rule_needs_two_of_every_three_who_take_a_side() {
        let body = Body {
            pass: PassRule::TwoThirds,
            ..Body::gathering(&params())
        };
        assert_eq!(body.decide(40, 12, 8, 4), Outcome::Passed);
        assert_eq!(
            body.decide(40, 12, 7, 5),
            Outcome::Failed,
            "a bare majority is not enough"
        );
        assert_eq!(body.decide(40, 12, 6, 6), Outcome::Tied);
        assert_eq!(body.decide(40, 9, 9, 0), Outcome::NoQuorum);
        for m in Membership::ALL {
            assert_eq!(Membership::from_code(m.code()), Some(m));
            assert_eq!(Membership::from_name(m.name()), Some(m));
        }
        for r in PassRule::ALL {
            assert_eq!(PassRule::from_code(r.code()), Some(r));
            assert_eq!(PassRule::from_name(r.name()), Some(r));
        }
    }

    #[test]
    fn the_custom_says_who_decides_and_keeps_its_versions() {
        let p = params();
        let mut polity = Polity::found(pid(1), pid(2), SimTime::from_minutes(0), &p);
        assert_eq!(polity.versions.len(), 1);
        let elders = Body {
            members: Membership::Elders,
            quorum_share: 0.5,
            ..polity.body
        };
        assert_eq!(
            elders.clause(),
            "the elders of its households who come to the hearth decide by acclamation: more \
             for than against carries it, and a tie fails; half of them must come"
        );
        let law = Law {
            id: pid(5),
            policy: 0,
            kind: PolicyKind::AmendBody,
            levy_share: 0.0,
            holder: None,
            relief_days: 0.0,
            sanction: Sanction::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: pid(9),
            proposed: SimTime::from_minutes(0),
            issue: IssueKind::Overruled,
            meets_day: 1,
            decided: Some(SimTime::from_minutes(1440)),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord::default(),
            body: Some(elders),
            ends: None,
            agreement: None,
        };
        let policies = [PolicyDef {
            id: "core:policy/amend_custom".to_owned(),
            name: "Amend the custom".to_owned(),
            description: String::new(),
            kind: PolicyKind::AmendBody,
            answers: vec![IssueKind::Overruled],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            bundles: Vec::new(),
            hours: Vec::new(),
            bodies: vec![elders],
            question: None,
            bears: Vec::new(),
        }];
        assert!(
            law_words(&law, &policies, &|_| "Ada".to_owned())
                .starts_with("a change of the custom: the elders of its households")
        );
        polity.laws.push(law);
        polity.body = elders;
        polity.versions.push(CustomVersion {
            body: elders,
            since: SimTime::from_minutes(1440),
            law: Some(pid(5)),
            seized_by: None,
        });
        let history = custom_history_words(&polity, &|_| "Ada".to_owned());
        assert_eq!(history.len(), 2);
        assert!(
            history[0].contains("the founding custom: the adults"),
            "{history:?}"
        );
        assert!(
            history[1].contains("by the amendment Ada proposed: the elders"),
            "{history:?}"
        );
    }

    #[test]
    fn the_rule_deliberator_proposes_what_it_expects_to_pass_and_gain_from() {
        let p = params();
        let d = RuleDeliberator { params: &p };
        let good = MoveOption {
            policy: 0,
            levy_share: 0.1,
            issue: IssueKind::FoodShort,
            nominee: None,
            sanction: Sanction::default(),
            hours: (0, 0),
            body: None,
            ends: None,
            own_gain: 0.3,
            followers_gain: 0.2,
            support: 0.8,
        };
        let hopeless = MoveOption {
            support: 0.0,
            ..good
        };
        // Worth 0.8 × 10 × 0.4 − 0.5 = 2.7 points against nothing's 0: nearly always chosen.
        let c = d.choose(&[hopeless, good], 0.5);
        assert_eq!(c.chosen, Some(1));
        assert!((c.scores[1] - 2.7).abs() < 1e-9 && c.scores[2] == 0.0);
        // A move nobody would back costs more than it could bring.
        let c = d.choose(&[hopeless], 0.5);
        assert_eq!(c.chosen, None);
    }
}
