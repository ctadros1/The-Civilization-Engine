//! Theft and what was seen (M4b slice AA, ADR-0015): what happened, kept apart from what people
//! believe of it, and what a household owes another for it.
//!
//! - **Incidents are the kernel's truth** (ADR-0015 §1): who went to take from whose store, when,
//!   how it ended, what was carried off and who saw it. No choice reads an incident.
//! - **Beliefs are what people know**: each records how its holder came by it (they saw it, were
//!   told, or found the loss), and whose eyes it comes from, so several people repeating one
//!   witness are one source (research 12-04 §1.5).
//! - **Responses and obligations**: a household that knows who took from it chooses to let it go,
//!   to demand the food back, or, where a law against taking is in force and it knows of it, to
//!   bring a case before the gathering (M4b slice AB); a demand, or what a finding imposes, is an
//!   obligation the taker's household chooses to meet or refuse, and meets only from what it has
//!   (research 09-07 §1.2).
//! - **Cases are what the polity knows** (ADR-0015 §4): the accusation, the accounts behind it,
//!   and how the gathering decided, whole. A case never reads the incident it is about.
//!
//! Each person carries an objection to taking, drawn at birth and pulled toward their parents'
//! (research 04-09 §1.1, social learning), and the chance they believe a taker runs of being
//! seen, which moves with what they live and hear (04-09 §5.4). No number here is a crime rate:
//! how often anyone takes is an output.

use civ_core::PermanentId;
use civ_core::rng::Rng64;
use civ_core::time::SimTime;

use crate::demography::normal;

/// How a crime profile weighs taking and what follows it (the people profile's `[crime]`,
/// content API 35). Every value is a tuning value: research 04-09 §5.4 finds no established
/// figures for moral-cost weights, belief-update rates or the conversion of hunger into offending.
#[derive(Clone, Debug, PartialEq)]
pub struct CrimeParams {
    /// Mean of the log-odds of a person's objection to taking, among people with no parents
    /// here (founders)...
    pub objection_mean: f64,
    /// ...and its spread (standard deviation).
    pub objection_sd: f64,
    /// How much of the mean of its parents' objection a child takes on (regression of offspring
    /// on mid-parent, in log-odds standard units).
    pub objection_heritability: f64,
    /// An objection at or above this rules taking out before anything is weighed (research 04-09
    /// §5.3: a moral filter first).
    pub objection_filter: f64,
    /// Points against taking for a full objection.
    pub w_objection: f64,
    /// Points against taking for a certainty of being seen: what the taker believes being seen
    /// would cost them, with those who would hear.
    pub w_seen: f64,
    /// How a person's risk-taking (a trait, standard units) shrinks or swells that cost: it is
    /// multiplied by `exp(-w_risk_trait * risk)`.
    pub w_risk_trait: f64,
    /// Points against taking for full regard for the household's elder one would take from.
    pub w_regard: f64,
    /// The chance of being seen someone believes before they have taken or heard anything.
    pub risk_prior: f64,
    /// The share of the gap a taker's own attempt closes between the chance they believed and
    /// what happened (seen: 1, unseen: 0)...
    pub risk_alpha: f64,
    /// ...and the share a taking they are told of closes toward 1 (word comes only of takers who
    /// were seen: a selected sample, research 04-09 §5.4).
    pub risk_alpha_told: f64,
    /// Metres within which someone awake sees a taker at a household's store, and the taker them.
    pub sight_m: f64,
    /// Chance the taker notices each person awake in sight as they come to the store; one noticed
    /// and they turn back, and those they miss see them (research 12-04 §1.4: an offender may
    /// notice a guardian and abandon, or proceed regardless).
    pub notice_chance: f64,
    /// Hours someone who turned back or fled from a store waits, on average, before weighing
    /// taking again: between half and one and a half times this (12-04 §1.4: they change their
    /// timing).
    pub retry_hours: f64,
    /// Years from which someone at home or about stops a taker: a capable guardian. A younger
    /// child sees them and does not stop them.
    pub guardian_age: f64,
    /// Chance that a sleeper at home wakes as someone takes from the household's store.
    pub wake_chance: f64,
    /// Days a person keeps what they believe of a taking.
    pub remember_days: u32,
    /// A household refuses an ask from someone it believes took from a household whose elder one
    /// of its members regards at least this much (or from itself).
    pub refuse_regard: f64,
    /// Points toward demanding the food back over letting it go, before anything is weighed...
    pub demand_base: f64,
    /// ...for each day of the household's food that was taken...
    pub w_demand_loss: f64,
    /// ...and against it, for full regard for the taker.
    pub w_forgive: f64,
    /// Days the taker's household has to give the food back.
    pub due_days: u32,
    /// Points toward meeting a demand over refusing it, before anything is weighed...
    pub comply_base: f64,
    /// ...for every household of the settlement that believes the taker took, as a share of its
    /// households (research 12-04 §1.2: a sanction is credible when enough people will back it)...
    pub w_comply_known: f64,
    /// ...for full regard for the elder of the household owed...
    pub w_comply_regard: f64,
    /// ...and against it, for each day of its own food paying would cost the household.
    pub w_comply_cost: f64,
    /// A household meeting a demand pays only from food beyond this many days of its own need.
    pub keep_days: f64,
    /// Points against bringing a case before the gathering (M4b slice AB): an evening there and
    /// the dispute made public (research 09-07 §4.1: the cost C of seeking a forum).
    pub report_cost: f64,
    /// Points toward finding against the accused, at a hearing, for one who believes they took;
    /// one who does not weighs the accounts told there, and with none at all, this much against.
    pub w_case_belief: f64,
    /// Points toward meeting what a gathering's finding imposes over refusing it: the custom that
    /// the gathering binds.
    pub w_comply_found: f64,
    /// Days of its food a household reckons losing a grown member to exile costs it: what it
    /// weighs a law that exiles against, and what a hearing puts at stake for it.
    pub exile_days: f64,
}

impl CrimeParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> CrimeParams {
        CrimeParams {
            objection_mean: 2.5,
            objection_sd: 1.2,
            objection_heritability: 0.5,
            objection_filter: 0.8,
            w_objection: 20.0,
            w_seen: 10.0,
            w_risk_trait: 0.3,
            w_regard: 5.0,
            risk_prior: 0.5,
            risk_alpha: 0.3,
            risk_alpha_told: 0.1,
            sight_m: 30.0,
            notice_chance: 0.7,
            retry_hours: 12.0,
            guardian_age: 10.0,
            wake_chance: 0.25,
            remember_days: 730,
            refuse_regard: 0.5,
            demand_base: 0.0,
            w_demand_loss: 1.0,
            w_forgive: 3.0,
            due_days: 30,
            comply_base: 0.0,
            w_comply_known: 4.0,
            w_comply_regard: 2.0,
            w_comply_cost: 0.5,
            keep_days: 3.0,
            report_cost: 1.0,
            w_case_belief: 1.0,
            w_comply_found: 2.0,
            exile_days: 60.0,
        }
    }
}

/// The logistic function.
pub fn logistic(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Log-odds of `p`, clamped away from 0 and 1.
fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    (p / (1.0 - p)).ln()
}

/// A person's objection to taking, 0–1, drawn from `rng`: a founder's from the content's
/// distribution, a child's pulled toward the mean of its parents' as traits are (research 04-09
/// §1.1, §5.1: an offence-specific objection, kept apart from personality and self-control).
pub fn draw_objection(
    mother: Option<f32>,
    father: Option<f32>,
    params: &CrimeParams,
    rng: &mut Rng64,
) -> f32 {
    let sd = params.objection_sd.max(1e-6);
    let standard = |o: f32| (logit(f64::from(o)) - params.objection_mean) / sd;
    let parents: Vec<f64> = [mother, father]
        .into_iter()
        .flatten()
        .map(standard)
        .collect();
    let z = if parents.is_empty() {
        normal(rng)
    } else {
        let h = params.objection_heritability.clamp(0.0, 1.0);
        let mid = parents.iter().sum::<f64>() / parents.len() as f64;
        h * mid + (1.0 - h * h / 2.0).max(0.0).sqrt() * normal(rng)
    };
    logistic(params.objection_mean + sd * z) as f32
}

/// The chance of being seen someone believes after a signal `s` (1: a taker was seen, 0: one was
/// not), closing `alpha` of the gap (research 04-09 §5.4: p̂ ← (1 − α)p̂ + αs).
pub fn updated_risk(risk: f32, s: f64, alpha: f64) -> f32 {
    let a = alpha.clamp(0.0, 1.0);
    ((1.0 - a) * f64::from(risk) + a * s.clamp(0.0, 1.0)) as f32
}

/// How an attempt to take ended. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// They took food and carried it off.
    Taken = 0,
    /// Someone was about (of the household, at home and awake, or anyone they noticed in sight):
    /// they turned back before taking anything.
    TurnedBack = 1,
    /// A sleeper at home woke and saw them: they fled with nothing.
    Disturbed = 2,
}

impl Outcome {
    /// Every outcome, in code order.
    pub const ALL: [Outcome; 3] = [Outcome::Taken, Outcome::TurnedBack, Outcome::Disturbed];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The outcome numbered `code`.
    pub fn from_code(code: u8) -> Option<Outcome> {
        Outcome::ALL.get(usize::from(code)).copied()
    }

    /// In words, for the observer: "took food", "turned back", "fled when seen".
    pub fn words(self) -> &'static str {
        match self {
            Outcome::Taken => "took food",
            Outcome::TurnedBack => "turned back: someone was about",
            Outcome::Disturbed => "fled with nothing: a sleeper woke",
        }
    }
}

/// What happened when someone went to take from another household's store: the truth, which no
/// choice reads (ADR-0015 §1).
#[derive(Clone, Debug, PartialEq)]
pub struct Incident {
    /// Its number, from 1, in the order they happened.
    pub id: u32,
    /// When.
    pub at: SimTime,
    /// Who went to take.
    pub actor: PermanentId,
    /// Their household then.
    pub actor_household: PermanentId,
    /// The household whose store it was.
    pub target: PermanentId,
    /// Where, metres: the target's home.
    pub place: (f32, f32),
    /// How it ended.
    pub outcome: Outcome,
    /// What was carried off, `(good index, amount in its unit)`; empty unless taken.
    pub goods: Vec<(u16, f32)>,
    /// Its food energy, kcal.
    pub kcal: f32,
    /// Who saw the taker at it, in id order.
    pub seen_by: Vec<PermanentId>,
    /// The household has found the loss (a taking only).
    pub noticed: bool,
}

/// How someone came to believe what they believe of an incident. Codes are part of saves: append
/// only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// They saw it.
    Saw = 0,
    /// Someone who knew told them.
    Told = 1,
    /// Their household found food gone from its store, and they do not know who took it.
    Noticed = 2,
    /// They did it themselves (M4b slice AB): a taker knows what they took.
    Did = 3,
}

impl Source {
    /// Every source, in code order.
    pub const ALL: [Source; 4] = [Source::Saw, Source::Told, Source::Noticed, Source::Did];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The source numbered `code`.
    pub fn from_code(code: u8) -> Option<Source> {
        Source::ALL.get(usize::from(code)).copied()
    }
}

/// What one person believes of one incident (ADR-0015 §1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Belief {
    /// Whose belief it is.
    pub holder: PermanentId,
    /// The incident it is of.
    pub incident: u32,
    /// Who they believe went to take, if they know.
    pub taker: Option<PermanentId>,
    /// How they came by it.
    pub source: Source,
    /// Who told them, if they were told.
    pub from: Option<PermanentId>,
    /// Whose eyes the account comes from: the witness it began with (themselves if they saw it);
    /// none for a loss found.
    pub origin: Option<PermanentId>,
    /// The day they came to believe it.
    pub day: i64,
}

/// What a household knows of the food an incident moved (M4b slice AB): what it found missing
/// from its store, or what a member brought home from another's. Kept apart from the incident,
/// which no choice reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnownAmount {
    /// The household.
    pub household: PermanentId,
    /// The incident.
    pub incident: u32,
    /// Food energy, kcal.
    pub kcal: f32,
    /// It lost the food; otherwise a member took it.
    pub lost: bool,
    /// The day it came to know.
    pub day: i64,
}

/// What a household knows of takings over a year (M4b slice AB): what it lost to takers it knows
/// of, and what its own members took, with the chance its grown members believe a taker runs of
/// being seen.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TakingsKnown {
    /// Food lost to takings whose taker a member knows, kcal.
    pub lost_kcal: f64,
    /// How many such takings.
    pub lost: u32,
    /// Food its members took, kcal.
    pub took_kcal: f64,
    /// How many takings.
    pub took: u32,
    /// The mean chance its grown members believe a taker runs of being seen.
    pub risk: f64,
}

impl TakingsKnown {
    /// What a law against taking with `sanction` would bring the household a year, kcal: back to
    /// it, what it lost to known takers with compensation; from it, what its own takers took with
    /// compensation and the fine (and, for exile, `exile_days` of its food for the member it would
    /// lose), at the chance they believe of being seen. Compensation and the fine are in days of
    /// the taker's household's food, which it takes to be like its own, `need_day` kcal a day.
    pub fn under(
        &self,
        sanction: &crate::polity::Sanction,
        need_day: f64,
        exile_days: f64,
    ) -> (f64, f64) {
        let comp = f64::from(sanction.compensation_days) * need_day;
        let exile = if sanction.exile {
            exile_days * need_day
        } else {
            0.0
        };
        let fine = f64::from(sanction.fine_days) * need_day + exile;
        let recover = self.lost_kcal + f64::from(self.lost) * comp;
        let owe =
            self.risk.clamp(0.0, 1.0) * (self.took_kcal + f64::from(self.took) * (comp + fine));
        (recover, owe)
    }
}

/// What a household taken from chose to do about it. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// It let it go.
    LetGo = 0,
    /// It demanded the food back of the taker's household.
    Demand = 1,
    /// It brought a case before the gathering, under a law against taking (M4b slice AB).
    Report = 2,
}

impl Choice {
    /// Every choice, in code order.
    pub const ALL: [Choice; 3] = [Choice::LetGo, Choice::Demand, Choice::Report];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The choice numbered `code`.
    pub fn from_code(code: u8) -> Option<Choice> {
        Choice::ALL.get(usize::from(code)).copied()
    }
}

/// What a household chose when it learnt who took from it, or when the taker's household would
/// not give back what it demanded (ADR-0015 §3), with its receipt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Response {
    /// The incident.
    pub incident: u32,
    /// The household taken from.
    pub household: PermanentId,
    /// The member who chose for it.
    pub by: PermanentId,
    /// The day.
    pub day: i64,
    /// What it chose.
    pub choice: Choice,
    /// Points toward demanding over letting it go (the receipt); 0 when a demand was not open,
    /// as after one was refused.
    pub points: f32,
    /// Points toward bringing a case over letting it go, when a law against taking it knew of
    /// was in force.
    pub report_points: Option<f32>,
}

/// Where an obligation stands. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Owed, and not yet answered or not yet paid in full.
    Open = 0,
    /// Paid in full.
    Met = 1,
    /// The debtor refused it (unwilling).
    Refused = 2,
    /// Due, and not paid in full though the debtor meant to (unable): the rest is an arrear.
    Defaulted = 3,
    /// The debtor's or the beneficiary's household is no more.
    Lapsed = 4,
}

impl Standing {
    /// Every standing, in code order.
    pub const ALL: [Standing; 5] = [
        Standing::Open,
        Standing::Met,
        Standing::Refused,
        Standing::Defaulted,
        Standing::Lapsed,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The standing numbered `code`.
    pub fn from_code(code: u8) -> Option<Standing> {
        Standing::ALL.get(usize::from(code)).copied()
    }

    /// In words: "owed", "paid", "refused", "unpaid when due", "lapsed".
    pub fn words(self) -> &'static str {
        match self {
            Standing::Open => "owed",
            Standing::Met => "paid",
            Standing::Refused => "refused",
            Standing::Defaulted => "unpaid when due",
            Standing::Lapsed => "lapsed",
        }
    }
}

/// What an obligation is (ADR-0015 §5; research 09-07 §1.2: restitution, compensation and a fine
/// are different transfers). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owed {
    /// What was taken, demanded back by the household taken from.
    Demanded = 0,
    /// What was taken, given back as a gathering's finding imposes.
    Restitution = 1,
    /// More food to the household taken from, as the law's bundle says.
    Compensation = 2,
    /// Food to the polity's common store, as the law's bundle says.
    Fine = 3,
}

impl Owed {
    /// Every kind, in code order.
    pub const ALL: [Owed; 4] = [
        Owed::Demanded,
        Owed::Restitution,
        Owed::Compensation,
        Owed::Fine,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The kind numbered `code`.
    pub fn from_code(code: u8) -> Option<Owed> {
        Owed::ALL.get(usize::from(code)).copied()
    }

    /// In words: "the food back", "restitution", "compensation", "a fine".
    pub fn words(self) -> &'static str {
        match self {
            Owed::Demanded => "the food back",
            Owed::Restitution => "restitution",
            Owed::Compensation => "compensation",
            Owed::Fine => "a fine",
        }
    }
}

/// What one household owes another, or its polity (ADR-0015 §5): food demanded back, or what a
/// gathering's finding imposed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obligation {
    /// Its number, from 1.
    pub id: u32,
    /// The incident it answers.
    pub incident: u32,
    /// What it is.
    pub kind: Owed,
    /// The case whose finding imposed it, if one did.
    pub case: Option<u32>,
    /// The household that owes.
    pub debtor: PermanentId,
    /// The household owed, or for a fine the polity.
    pub beneficiary: PermanentId,
    /// Food energy owed, kcal.
    pub kcal: f32,
    /// Food energy paid so far, kcal.
    pub paid_kcal: f32,
    /// The day it was demanded.
    pub made: i64,
    /// The day it is due.
    pub due: i64,
    /// Where it stands.
    pub standing: Standing,
    /// The debtor's answer, once given: whether it meant to pay, and its points toward paying
    /// over refusing (the receipt).
    pub answer: Option<(bool, f32)>,
}

impl Obligation {
    /// Food energy still owed, kcal.
    pub fn left_kcal(&self) -> f64 {
        (f64::from(self.kcal) - f64::from(self.paid_kcal)).max(0.0)
    }
}

/// Everything slice AA keeps of takings: the truth, what people believe, and what households
/// chose and owe (saved).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Order {
    /// Every attempt to take, in the order they happened.
    pub incidents: Vec<Incident>,
    /// What people believe of them, by holder and then incident.
    pub beliefs: Vec<Belief>,
    /// What households taken from chose, in order.
    pub responses: Vec<Response>,
    /// What households owe one another, in the order demanded.
    pub obligations: Vec<Obligation>,
    /// What households know of the food incidents moved, in the order they came to know.
    pub amounts: Vec<KnownAmount>,
    /// Cases brought before the gathering, in the order brought (M4b slice AB).
    pub cases: Vec<Case>,
    /// Asks refused because the giver believed the asker took (a counter, not saved).
    pub refusals: u64,
}

/// Where a case stands. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseStage {
    /// Brought, and waiting for the gathering to hear it.
    Open = 0,
    /// The gathering found that the accused took.
    Found = 1,
    /// The gathering did not find it: it turned the accusation down, or split evenly.
    NotFound = 2,
    /// Too few came to the gathering to hear it.
    Unheard = 3,
    /// The household that brought it, or the accused, was no longer there to be heard.
    Lapsed = 4,
}

impl CaseStage {
    /// Every stage, in code order.
    pub const ALL: [CaseStage; 5] = [
        CaseStage::Open,
        CaseStage::Found,
        CaseStage::NotFound,
        CaseStage::Unheard,
        CaseStage::Lapsed,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The stage numbered `code`.
    pub fn from_code(code: u8) -> Option<CaseStage> {
        CaseStage::ALL.get(usize::from(code)).copied()
    }
}

/// An accusation brought before the gathering under a law against taking (ADR-0015 §4; research
/// 12-04 §1.5: a case holds its leads with their provenance). It is what the polity knows: the
/// household's account of its loss, who it says took, and the witnesses its account comes from.
/// The decision is kept whole, as a law's is.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    /// Its number, from 1.
    pub id: u32,
    /// The incident the accusation is of (a join key; the case never reads it).
    pub incident: u32,
    /// The settlement whose gathering hears it.
    pub settlement: PermanentId,
    /// The law it is brought under.
    pub law: PermanentId,
    /// The household that brought it: the one taken from.
    pub accuser: PermanentId,
    /// The member who brought it.
    pub by: PermanentId,
    /// Whom it accuses.
    pub accused: PermanentId,
    /// Their household when it was brought: the one that would owe.
    pub accused_household: PermanentId,
    /// What the household says it lost, kcal: what it found missing.
    pub kcal: f32,
    /// The witnesses the accusation's accounts come from, in id order: several people repeating
    /// one witness are one lead.
    pub leads: Vec<PermanentId>,
    /// The day it was brought.
    pub opened: i64,
    /// Where it stands.
    pub stage: CaseStage,
    /// When the gathering sat on it.
    pub heard: Option<SimTime>,
    /// The adults who could have come, when it sat.
    pub eligible: u32,
    /// Where each who came stood, in id order: support is for finding that the accused took.
    pub stances: Vec<CaseStance>,
    /// The finding sent the accused from the valley.
    pub exiled: bool,
}

/// One member's stance at a hearing, with what moved it (ADR-0015 §4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaseStance {
    /// Who.
    pub person: PermanentId,
    /// Their household.
    pub household: PermanentId,
    /// Where they stood: support is for finding that the accused took.
    pub stance: crate::polity::Stance,
    /// What their household stood to gain or lose by a finding, points: the days of its food at
    /// stake, at a demand's points a day.
    pub stake: f32,
    /// What they believed, or made of the accounts told, points.
    pub belief: f32,
    /// Their regard for the one who brought it less their regard for the accused, points.
    pub regard: f32,
}

impl Case {
    /// Who came, and how many supported and opposed a finding.
    pub fn counts(&self) -> (u32, u32, u32) {
        let n =
            |s: crate::polity::Stance| self.stances.iter().filter(|r| r.stance == s).count() as u32;
        (
            self.stances.len() as u32,
            n(crate::polity::Stance::Support),
            n(crate::polity::Stance::Oppose),
        )
    }
}

/// The chance someone believes a gathering will find against a taker from `accounts` distinct
/// accounts (research 12-04 §1.5: identifying information mostly comes with the report; several
/// people repeating one witness are one observation): 1 − 0.5^accounts, a tuning form.
pub fn case_strength(accounts: usize) -> f64 {
    1.0 - 0.5f64.powi(accounts.min(64) as i32)
}

/// Points toward bringing a case over letting it go (research 09-07 §4.1: U = p̂(success)·V − C):
/// `accounts` distinct accounts of who took, `lost_days` and `compensation_days` days of the
/// household's food to recover, and `regard` the chooser's for the taker.
pub fn report_points(
    accounts: usize,
    lost_days: f64,
    compensation_days: f64,
    regard: f64,
    p: &CrimeParams,
) -> f64 {
    let v = p.w_demand_loss * (lost_days.max(0.0) + compensation_days.max(0.0));
    case_strength(accounts) * v - p.report_cost - p.w_forgive * regard.clamp(0.0, 1.0)
}

/// Points toward finding against the accused for someone at a hearing (ADR-0015 §4): `believes`
/// whether they believe the accused took; otherwise `leads`, the accounts told there, weigh
/// [`case_strength`] either side of an even chance.
pub fn belief_points(believes: bool, leads: usize, p: &CrimeParams) -> f64 {
    let b = if believes { 1.0 } else { case_strength(leads) };
    p.w_case_belief * (2.0 * b - 1.0)
}

impl Order {
    /// The incident numbered `id`.
    pub fn incident(&self, id: u32) -> Option<&Incident> {
        self.incidents
            .binary_search_by_key(&id, |i| i.id)
            .ok()
            .map(|k| &self.incidents[k])
    }

    /// Where `holder`'s belief of incident `incident` is, or would go, in `beliefs`.
    fn slot(&self, holder: PermanentId, incident: u32) -> Result<usize, usize> {
        self.beliefs
            .binary_search_by_key(&(holder, incident), |b| (b.holder, b.incident))
    }

    /// Everything `holder` believes, by incident.
    pub fn held_by(&self, holder: PermanentId) -> &[Belief] {
        let lo = self.beliefs.partition_point(|b| b.holder < holder);
        let hi = self.beliefs.partition_point(|b| b.holder <= holder);
        &self.beliefs[lo..hi]
    }

    /// `holder`'s belief of incident `incident`, if they hold one.
    pub fn belief(&self, holder: PermanentId, incident: u32) -> Option<&Belief> {
        self.slot(holder, incident).ok().map(|i| &self.beliefs[i])
    }

    /// Whether `holder` believes `taker` went to take from a household `from` accepts, on or
    /// after day `since`.
    pub fn believes_took(
        &self,
        holder: PermanentId,
        taker: PermanentId,
        since: i64,
        from: &dyn Fn(PermanentId) -> bool,
    ) -> bool {
        self.held_by(holder).iter().any(|b| {
            b.taker == Some(taker)
                && b.day >= since
                && self.incident(b.incident).is_some_and(|i| from(i.target))
        })
    }

    /// Records that `holder` came to believe `belief` of an incident; a belief they hold already
    /// stays, unless the new one knows a taker the old one did not. Returns whether anything
    /// changed.
    pub fn learn(&mut self, belief: Belief) -> bool {
        match self.slot(belief.holder, belief.incident) {
            Ok(i) if self.beliefs[i].taker.is_none() && belief.taker.is_some() => {
                self.beliefs[i] = belief;
                true
            }
            Ok(_) => false,
            Err(i) => {
                self.beliefs.insert(i, belief);
                true
            }
        }
    }

    /// The case numbered `id`.
    pub fn case(&self, id: u32) -> Option<&Case> {
        self.cases
            .binary_search_by_key(&id, |c| c.id)
            .ok()
            .map(|k| &self.cases[k])
    }

    /// The witnesses behind what `holders` believe of who took in incident `incident`, in id
    /// order, leaving out `taker`'s own knowledge: the accounts they could bring.
    pub fn leads(
        &self,
        holders: &[PermanentId],
        incident: u32,
        taker: PermanentId,
    ) -> Vec<PermanentId> {
        let mut out: Vec<PermanentId> = holders
            .iter()
            .filter_map(|&h| self.belief(h, incident))
            .filter(|b| b.taker == Some(taker))
            .filter_map(|b| b.origin)
            .filter(|&o| o != taker)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Whether household `household` found food taken from its store on or after day `since`.
    pub fn lost_since(&self, household: PermanentId, since: i64) -> bool {
        self.amounts
            .iter()
            .rev()
            .take_while(|a| a.day >= since)
            .any(|a| a.lost && a.household == household)
    }

    /// The number of distinct first-hand accounts behind what people believe of incident
    /// `incident` (several people repeating one witness are one: research 12-04 §1.5). The taker's
    /// own knowledge is not an account of it.
    pub fn sources(&self, incident: u32) -> usize {
        let actor = self.incident(incident).map(|i| i.actor);
        let mut origins: Vec<PermanentId> = self
            .beliefs
            .iter()
            .filter(|b| b.incident == incident)
            .filter_map(|b| b.origin)
            .filter(|&o| Some(o) != actor)
            .collect();
        origins.sort_unstable();
        origins.dedup();
        origins.len()
    }
}

/// Points toward demanding the food back over letting it go (ADR-0015 §3; research 09-07 §4.1: a
/// victim weighs what it would recover against what the dispute costs): `lost_days` days of the
/// household's food were taken; `regard` is the chooser's for the taker.
pub fn demand_points(lost_days: f64, regard: f64, p: &CrimeParams) -> f64 {
    p.demand_base + p.w_demand_loss * lost_days.max(0.0) - p.w_forgive * regard.clamp(0.0, 1.0)
}

/// Points toward meeting a demand over refusing it: `known` is the share of the settlement's
/// households that believe the taker took, `regard` the chooser's for the elder owed, and
/// `cost_days` the days of its own food paying would cost the household.
pub fn comply_points(known: f64, regard: f64, cost_days: f64, p: &CrimeParams) -> f64 {
    p.comply_base
        + p.w_comply_known * known.clamp(0.0, 1.0)
        + p.w_comply_regard * regard.clamp(0.0, 1.0)
        - p.w_comply_cost * cost_days.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("id")
    }

    #[test]
    fn objections_are_mostly_above_the_filter_and_children_take_after_their_parents() {
        let p = CrimeParams::core();
        let mut rng = Rng64::from_key(&[1, 2, 3]);
        let n = 20_000;
        let founders: Vec<f32> = (0..n)
            .map(|_| draw_objection(None, None, &p, &mut rng))
            .collect();
        let below = founders
            .iter()
            .filter(|&&o| f64::from(o) < p.objection_filter)
            .count() as f64
            / n as f64;
        // About one in six could weigh taking at all (a tuning value; 04-09 §5.3's filter).
        assert!((0.1..0.3).contains(&below), "{below}");
        assert!(founders.iter().all(|o| (0.0..=1.0).contains(o)));
        // Children of two low-objection parents fall below the filter far more often.
        let low = 0.4f32;
        let children = (0..n)
            .filter(|_| {
                f64::from(draw_objection(Some(low), Some(low), &p, &mut rng)) < p.objection_filter
            })
            .count() as f64
            / n as f64;
        assert!(children > 2.0 * below, "{children} vs {below}");
    }

    #[test]
    fn perceived_risk_moves_toward_what_is_seen() {
        let r = updated_risk(0.5, 1.0, 0.3);
        assert!((r - 0.65).abs() < 1e-6);
        let r = updated_risk(0.5, 0.0, 0.3);
        assert!((r - 0.35).abs() < 1e-6);
        assert_eq!(updated_risk(0.5, 1.0, 0.0), 0.5);
    }

    #[test]
    fn a_belief_is_kept_once_and_one_witness_is_one_source() {
        let mut o = Order::default();
        let noticed = Belief {
            holder: id(1),
            incident: 1,
            taker: None,
            source: Source::Noticed,
            from: None,
            origin: None,
            day: 10,
        };
        assert!(o.learn(noticed));
        assert!(!o.learn(noticed));
        // Being told who took replaces knowing only of the loss.
        let told = Belief {
            taker: Some(id(9)),
            source: Source::Told,
            from: Some(id(3)),
            origin: Some(id(3)),
            day: 11,
            ..noticed
        };
        assert!(o.learn(told));
        assert_eq!(o.beliefs.len(), 1);
        assert_eq!(o.beliefs[0].taker, Some(id(9)));
        // Two more people repeating the same witness add no source.
        for h in [4, 5] {
            o.learn(Belief {
                holder: id(h),
                ..told
            });
        }
        o.learn(Belief {
            holder: id(3),
            source: Source::Saw,
            from: None,
            ..told
        });
        assert_eq!(o.sources(1), 1);
    }

    #[test]
    fn demands_weigh_the_loss_against_regard_and_payment_its_cost() {
        let p = CrimeParams::core();
        assert!(demand_points(5.0, 0.0, &p) > demand_points(5.0, 1.0, &p));
        assert!(demand_points(5.0, 0.0, &p) > demand_points(1.0, 0.0, &p));
        assert!(comply_points(0.8, 0.0, 1.0, &p) > comply_points(0.1, 0.0, 1.0, &p));
        assert!(comply_points(0.5, 0.0, 1.0, &p) > comply_points(0.5, 0.0, 10.0, &p));
    }
}
