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
}

impl PolicyKind {
    /// Every kind, in code order.
    pub const ALL: [PolicyKind; 1] = [PolicyKind::CommonStore];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            PolicyKind::CommonStore => "common_store",
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
}

impl IssueKind {
    /// Every kind, in code order.
    pub const ALL: [IssueKind; 1] = [IssueKind::FoodShort];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            IssueKind::FoodShort => "food_short",
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
    /// The lean years a settlement is believed to have before it has seen any, out of
    /// `prior_years`.
    pub prior_lean: f64,
    /// Years of that prior belief.
    pub prior_years: f64,
    /// A lean year's harvest, as a share of an ordinary one.
    pub lean_harvest: f64,
    /// The share of a year's food below which a household cannot live: food there is worth the
    /// most, and a household that would fall below it cannot pay a levy.
    pub subsistence_share: f64,
    /// Points for paying what a law asks, before its cost: the custom that a gathering's
    /// decision binds.
    pub comply_base: f64,
    /// Points for paying per unit of the stance a person took on the law (1 for, −1 against, 0
    /// for abstaining or absent).
    pub w_stance: f64,
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
            prior_lean: 1.0,
            prior_years: 4.0,
            lean_harvest: 0.5,
            subsistence_share: 0.5,
            comply_base: 1.5,
            w_stance: 1.0,
        }
    }
}

/// Who a body's members are (ADR-0013 §2). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Membership {
    /// Every adult who lives in the settlement.
    Adults,
}

/// How a body decides (ADR-0013 §2; research 09-05 §1.4). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassRule {
    /// Acclamation: more of those present for than against; a tie fails.
    MoreForThanAgainst,
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
        };
        let how = match self.pass {
            PassRule::MoreForThanAgainst => {
                "decide by acclamation: more for than against carries it, and a tie fails"
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
        match self.pass {
            PassRule::MoreForThanAgainst if support > oppose => Outcome::Passed,
            PassRule::MoreForThanAgainst if support == oppose => Outcome::Tied,
            PassRule::MoreForThanAgainst => Outcome::Failed,
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
    /// Grain owed and kept back (could not and evaded), kilograms.
    pub withheld_kg: f64,
    /// Asks the store answered.
    pub relieved: u32,
    /// Food the store gave, kilograms.
    pub relief_kg: f64,
    /// Asks the store could not answer: it held nothing to give.
    pub unanswered: u32,
}

/// A law and its whole history (ADR-0013 §3): kept for ever, never pruned.
#[derive(Clone, Debug, PartialEq)]
pub struct Law {
    /// Permanent id.
    pub id: PermanentId,
    /// Its template, by index in the catalog's policies.
    pub policy: u16,
    /// The levy share proposed.
    pub levy_share: f32,
    /// The most one ask brings, days of the asking household's food.
    pub relief_days: f32,
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

/// A gathering called on a proposal: the law, the day it meets, what each household made of it
/// when word went round, and who came.
#[derive(Clone, Debug, PartialEq)]
pub struct Gathering {
    /// The law before it.
    pub law: PermanentId,
    /// The day it meets.
    pub day: i64,
    /// Each household's forecast of the law when it was proposed, points
    /// ([`PolityParams::w_gain`] times [`store_gain`]), by household id.
    pub stakes: Vec<(PermanentId, f32)>,
    /// Those who came, in id order.
    pub present: Vec<PermanentId>,
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
    let household = if gain > margin {
        "their household stands to gain"
    } else if gain < -margin {
        "their household stands to lose"
    } else {
        "it makes little difference to their household"
    };
    match r.stance {
        Stance::Support if gain <= margin && regard > 0.0 => {
            format!("{household}, but they think well of the sponsor")
        }
        Stance::Support if regard > 0.0 => {
            format!("{household}, and they think well of the sponsor")
        }
        Stance::Oppose | Stance::Abstain if regard > 0.0 && gain < -margin => {
            format!("{household}, though they think well of the sponsor")
        }
        _ => household.to_owned(),
    }
}

/// What a law is, in words: "a common store, taking a tenth of each harvest".
pub fn law_words(law: &Law, policies: &[PolicyDef]) -> String {
    let name = policies.get(usize::from(law.policy)).map_or_else(
        || "a law".to_owned(),
        |d| {
            let name = d.name.to_lowercase();
            let article = match name.chars().next() {
                Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
                _ => "a",
            };
            format!("{article} {name}")
        },
    );
    format!(
        "{name}, taking {} of each harvest",
        share_text(f64::from(law.levy_share))
    )
}

/// How the gathering decided a law, in words: "agreed: 20 for, 3 against; 23 of 24 adults came,
/// 6 needed". `None` before it has.
pub fn decision_words(law: &Law, body: &Body) -> Option<String> {
    let outcome = law.outcome?;
    let (present, support, oppose) = law.counts();
    let came = format!(
        "{present} of {} adults came, {} needed",
        law.eligible,
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
}

impl Polity {
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
        }
    }

    /// The law before the gathering, if any.
    pub fn agenda(&self) -> Option<&Law> {
        self.laws.iter().find(|l| l.status == LawStatus::Proposed)
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

    /// The law with id `id`.
    pub fn law_mut(&mut self, id: PermanentId) -> Option<&mut Law> {
        self.laws.iter_mut().find(|l| l.id == id)
    }

    /// Spoils the store up to `t` as anyone's unroofed store spoils.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[crate::params::GoodDef]) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            if d.half_life_days > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / d.half_life_days);
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
    let regard_points = params.w_regard * regard.clamp(0.0, 1.0);
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

/// The points for paying a levy: the custom that a gathering's decision binds, the stance the
/// person took, their regard for the law's sponsor, less what paying costs their household.
/// There is no sanction yet (M4b): nobody is set to see who pays.
pub fn comply_points(stance: f64, regard: f64, cost_points: f64, params: &PolityParams) -> f64 {
    params.comply_base + params.w_stance * stance + params.w_regard * regard.clamp(0.0, 1.0)
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
    }

    #[test]
    fn paying_is_likelier_for_those_who_backed_the_law_and_dearer_for_the_hard_pressed() {
        let p = params();
        let backed = comply_chance(comply_points(1.0, 0.0, 0.5, &p));
        let opposed = comply_chance(comply_points(-1.0, 0.0, 0.5, &p));
        let pressed = comply_chance(comply_points(1.0, 0.0, 3.0, &p));
        assert!(backed > opposed && backed > pressed);
        assert!(cannot_pay(100.0, 60.0, 200.0, &p));
        assert!(!cannot_pay(200.0, 60.0, 200.0, &p));
    }

    #[test]
    fn a_law_keeps_who_knows_it_in_order() {
        let mut law = Law {
            id: pid(1),
            policy: 0,
            levy_share: 0.1,
            relief_days: 5.0,
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
        };
        assert!(law.learn(pid(9), 3));
        assert!(law.learn(pid(4), 5));
        assert!(!law.learn(pid(9), 6), "known already");
        assert_eq!(law.known, vec![(pid(4), 5), (pid(9), 3)]);
        assert!(law.knows(pid(4)) && !law.knows(pid(7)));
    }

    #[test]
    fn the_rule_deliberator_proposes_what_it_expects_to_pass_and_gain_from() {
        let p = params();
        let d = RuleDeliberator { params: &p };
        let good = MoveOption {
            policy: 0,
            levy_share: 0.1,
            issue: IssueKind::FoodShort,
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
