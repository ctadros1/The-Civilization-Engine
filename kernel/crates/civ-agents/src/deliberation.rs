//! A model deliberator (ADR-0039 §2–§5): at the polity review a model may choose among a
//! notable's moves in the rule deliberator's place.
//!
//! The kernel lists the moves as plain data ([`DeliberationRequest`]) and asks the provider the
//! host set ([`ModelProvider`]); the kernel never touches a network. An answer naming no listed
//! move is refused; a refused or missing answer leaves the choice to [`RuleDeliberator`], on the
//! same keyed draw, so a world without a model lives exactly as before (ADR-0039 §3). Accepted
//! answers are saved ([`Answers`]) and a world in [`DeliberationMode::Recorded`] reads them back
//! (ADR-0039 §4).

use civ_core::PermanentId;

use crate::polity::{Choice, Deliberator, MoveOption, PolicyDef, RuleDeliberator};

/// Who made a notable's choice (ADR-0039 §4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChosenBy {
    /// The rule deliberator, with no model asked.
    #[default]
    Rule,
    /// The model.
    Model,
    /// The rule deliberator, because the model did not answer or its answer was refused.
    RuleFallback,
}

impl ChosenBy {
    /// Its code in saves: 0 the model, 1 rule after the model did not answer. `Rule` is never
    /// saved.
    pub fn code(self) -> u8 {
        match self {
            ChosenBy::Model => 0,
            ChosenBy::RuleFallback => 1,
            ChosenBy::Rule => u8::MAX,
        }
    }

    /// The one a saved code names.
    pub fn from_code(code: u8) -> Option<ChosenBy> {
        match code {
            0 => Some(ChosenBy::Model),
            1 => Some(ChosenBy::RuleFallback),
            _ => None,
        }
    }

    /// In words, after the sponsor's name: "chose by the model"; `None` for a rule-only choice.
    pub fn words(self) -> Option<&'static str> {
        match self {
            ChosenBy::Rule => None,
            ChosenBy::Model => Some("chose by the model"),
            ChosenBy::RuleFallback => Some("chose by rule (the model did not answer)"),
        }
    }
}

/// One move as the model is shown it (ADR-0039 §2): plain data, numbered from 0 here; a prompt
/// may number from 1.
#[derive(Clone, Debug, PartialEq)]
pub struct OfferedMove {
    /// Its index in the moves.
    pub index: usize,
    /// The policy's content id.
    pub policy: String,
    /// The policy's display name.
    pub name: String,
    /// The issue it answers, by its authored name.
    pub issue: &'static str,
    /// The issue in words.
    pub issue_words: &'static str,
    /// Its other terms in words: the levy share, whom it names, the body it sets.
    pub terms: Vec<String>,
    /// The forecast for the notable's own household.
    pub own_gain: f64,
    /// The forecast for those who regard them, weighted by that regard.
    pub followers_gain: f64,
    /// The share of those who would take a stance whom they expect to back it.
    pub support: f64,
}

/// What the kernel asks a provider (ADR-0039 §2): who chooses, when, and the moves.
#[derive(Clone, Debug, PartialEq)]
pub struct DeliberationRequest {
    /// The notable.
    pub notable: PermanentId,
    /// Their given name: in-world text, to be quoted as data (ADR-0039 §5).
    pub name: String,
    /// The day.
    pub day: i64,
    /// The moves, in order.
    pub moves: Vec<OfferedMove>,
    /// A digest of the moves as the kernel holds them ([`digest`]).
    pub digest: u64,
}

/// A provider's answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelAnswer {
    /// The move at this index.
    Move(usize),
    /// Making none.
    None,
    /// A reply that names no move (ADR-0039 §5): rules decide.
    Refused,
    /// No reply in time, or none asked for (ADR-0039 §3): rules decide.
    Unanswered,
}

/// Which provider answered (ADR-0039 §4): its kind and the model's name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderFingerprint {
    /// The kind of endpoint: "http", "stub".
    pub kind: String,
    /// The model's name as the host was told it.
    pub model: String,
}

/// Something that answers deliberation requests (ADR-0039 §3), set by the host. Never saved.
/// `Sync`, so a world stays shareable between threads.
pub trait ModelProvider: Send + Sync {
    /// Its fingerprint, saved with each answer it gives.
    fn fingerprint(&self) -> ProviderFingerprint;
    /// Answers one request. Blocks the simulation until it returns.
    fn answer(&mut self, request: &DeliberationRequest) -> ModelAnswer;
}

/// One saved answer (ADR-0039 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SavedAnswer {
    /// The notable.
    pub notable: PermanentId,
    /// The day.
    pub day: i64,
    /// The digest of the moves offered.
    pub digest: u64,
    /// The move made, by index; `None` for none.
    pub chosen: Option<u16>,
    /// `Model`, or `RuleFallback` for a move rules made when the model did not answer.
    pub by: ChosenBy,
    /// The provider, by index in [`Answers::providers`].
    pub provider: u16,
}

/// A world's saved answers (ADR-0039 §4): every accepted answer, and each move rules made when
/// the model did not answer, in the order given (so by day).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Answers {
    /// The providers that answered, in the order first seen.
    pub providers: Vec<ProviderFingerprint>,
    /// The answers, by day.
    pub answers: Vec<SavedAnswer>,
}

impl Answers {
    /// Those of `day`.
    fn of_day(&self, day: i64) -> &[SavedAnswer] {
        let from = self.answers.partition_point(|a| a.day < day);
        let to = self.answers.partition_point(|a| a.day <= day);
        &self.answers[from..to]
    }

    /// The answer saved for `notable` on `day` to moves of `digest`.
    pub fn find(&self, notable: PermanentId, day: i64, digest: u64) -> Option<&SavedAnswer> {
        self.of_day(day)
            .iter()
            .find(|a| a.notable == notable && a.digest == digest)
    }

    /// Who chose the move `sponsor` made on `day`, where a model was asked: for a law's receipt.
    pub fn chosen_by(&self, sponsor: PermanentId, day: i64) -> Option<ChosenBy> {
        self.of_day(day)
            .iter()
            .find(|a| a.notable == sponsor && a.chosen.is_some())
            .map(|a| a.by)
    }

    /// Keeps an answer from `provider`.
    fn push(
        &mut self,
        request: &DeliberationRequest,
        choice: &Choice,
        provider: &ProviderFingerprint,
    ) {
        let index = match self.providers.iter().position(|p| p == provider) {
            Some(i) => i,
            None => {
                self.providers.push(provider.clone());
                self.providers.len() - 1
            }
        };
        self.answers.push(SavedAnswer {
            notable: request.notable,
            day: request.day,
            digest: request.digest,
            chosen: choice.chosen.and_then(|i| u16::try_from(i).ok()),
            by: choice.by,
            provider: u16::try_from(index).unwrap_or(u16::MAX),
        });
    }
}

/// How a world deliberates (ADR-0039 §3): a session setting, never saved; a loaded world is
/// rule-only until the host says otherwise.
#[derive(Default)]
pub enum DeliberationMode {
    /// Rules alone (the default).
    #[default]
    Rule,
    /// Answers read back from these saved answers, matched by notable, day and digest; rules
    /// where none matches.
    Recorded(Answers),
    /// A provider asked at each choice.
    Model(Box<dyn ModelProvider>),
}

impl std::fmt::Debug for DeliberationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliberationMode::Rule => f.write_str("Rule"),
            DeliberationMode::Recorded(a) => write!(f, "Recorded({} answers)", a.answers.len()),
            DeliberationMode::Model(p) => write!(f, "Model({:?})", p.fingerprint()),
        }
    }
}

/// A world's deliberation: the mode (a session setting) and its saved answers.
#[derive(Debug, Default)]
pub struct Deliberation {
    /// The mode. Never saved.
    pub mode: DeliberationMode,
    /// The answers saved so far. Saved.
    pub answers: Answers,
}

impl Deliberation {
    /// Whether a choice now would consult anything beyond rules.
    pub fn consults(&self) -> bool {
        !matches!(self.mode, DeliberationMode::Rule)
    }

    /// Chooses among `moves` with draw `u`: the rule deliberator, unless the mode answers
    /// `request` with a move it lists or none (ADR-0039 §2). The receipt keeps the rule's scores
    /// and who chose. With no request, or rule-only, exactly the rule's choice.
    pub fn decide(
        &mut self,
        rule: &RuleDeliberator,
        moves: &[MoveOption],
        u: f64,
        request: Option<&DeliberationRequest>,
    ) -> Choice {
        let mut choice = rule.choose(moves, u);
        let Some(request) = request else {
            return choice;
        };
        let (answer, provider) = match &mut self.mode {
            DeliberationMode::Rule => return choice,
            DeliberationMode::Model(p) => (p.answer(request), p.fingerprint()),
            DeliberationMode::Recorded(source) => {
                let Some(a) = source.find(request.notable, request.day, request.digest) else {
                    return choice;
                };
                let provider = source
                    .providers
                    .get(usize::from(a.provider))
                    .cloned()
                    .unwrap_or_default();
                let answer = match (a.by, a.chosen) {
                    (ChosenBy::Model, Some(i)) => ModelAnswer::Move(usize::from(i)),
                    (ChosenBy::Model, None) => ModelAnswer::None,
                    _ => ModelAnswer::Unanswered,
                };
                (answer, provider)
            }
        };
        match answer {
            ModelAnswer::Move(i) if i < moves.len() => {
                choice.chosen = Some(i);
                choice.by = ChosenBy::Model;
            }
            ModelAnswer::None => {
                choice.chosen = None;
                choice.by = ChosenBy::Model;
            }
            // Out of range, refused or unanswered: rules decide (ADR-0039 §2).
            _ => choice.by = ChosenBy::RuleFallback,
        }
        if choice.by == ChosenBy::Model || choice.chosen.is_some() {
            self.answers.push(request, &choice, &provider);
        }
        choice
    }
}

/// A digest of `moves` as the kernel holds them, with each policy's content id: what a saved
/// answer is matched by (ADR-0039 §4). FNV-1a over each move's debug form, stable on one build.
pub fn digest(moves: &[MoveOption], policies: &[PolicyDef]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for m in moves {
        let id = policies
            .get(usize::from(m.policy))
            .map_or("", |d| d.id.as_str());
        eat(id.as_bytes());
        eat(&[0]);
        eat(format!("{m:?}").as_bytes());
        eat(&[0xff]);
    }
    h
}

/// The request for `notable` on `day` to choose among `moves` (ADR-0039 §2): each move as plain
/// data, with `name_of` naming those a move names.
pub fn request(
    notable: PermanentId,
    name: String,
    day: i64,
    moves: &[MoveOption],
    policies: &[PolicyDef],
    name_of: &dyn Fn(PermanentId) -> String,
) -> DeliberationRequest {
    let offered = moves
        .iter()
        .enumerate()
        .map(|(index, m)| {
            let def = policies.get(usize::from(m.policy));
            let mut terms = Vec::new();
            if m.levy_share > 0.0 {
                terms.push(format!(
                    "a levy of {:.0}% of threshed grain",
                    m.levy_share * 100.0
                ));
            }
            if let Some(p) = m.nominee {
                terms.push(format!("names {}", name_of(p)));
            }
            if let Some(b) = &m.body {
                terms.push(format!("the body becomes: {}", b.words()));
            }
            let sanction = m.sanction.words();
            if !sanction.is_empty() {
                terms.push(format!("what was taken given back{sanction}"));
            }
            if m.hours != (0, 0) {
                terms.push(format!("from hour {} to hour {}", m.hours.0, m.hours.1));
            }
            if let Some(l) = m.ends {
                terms.push(format!("replaces law {l}"));
            }
            if let Some(t) = m.target {
                terms.push(format!("against settlement {t}"));
            }
            if let Some(c) = m.site {
                terms.push(format!("at cell {c}"));
            }
            OfferedMove {
                index,
                policy: def.map_or_else(String::new, |d| d.id.clone()),
                name: def.map_or_else(String::new, |d| d.name.clone()),
                issue: m.issue.name(),
                issue_words: m.issue.words(),
                terms,
                own_gain: m.own_gain,
                followers_gain: m.followers_gain,
                support: m.support,
            }
        })
        .collect();
    DeliberationRequest {
        notable,
        name,
        day,
        moves: offered,
        digest: digest(moves, policies),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polity::{IssueKind, PolityParams, Sanction};

    fn pid(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn move_of(own_gain: f64) -> MoveOption {
        MoveOption {
            policy: 0,
            levy_share: 0.1,
            issue: IssueKind::FoodShort,
            nominee: None,
            sanction: Sanction::default(),
            hours: (0, 0),
            body: None,
            ends: None,
            site: None,
            target: None,
            own_gain,
            followers_gain: 0.0,
            support: 0.8,
        }
    }

    struct Fixed(ModelAnswer);

    impl ModelProvider for Fixed {
        fn fingerprint(&self) -> ProviderFingerprint {
            ProviderFingerprint {
                kind: "stub".into(),
                model: "fixed".into(),
            }
        }
        fn answer(&mut self, _: &DeliberationRequest) -> ModelAnswer {
            self.0
        }
    }

    fn ask(mode: DeliberationMode, moves: &[MoveOption], u: f64) -> (Choice, Deliberation) {
        let p = PolityParams::core();
        let rule = RuleDeliberator { params: &p };
        let mut d = Deliberation {
            mode,
            ..Default::default()
        };
        let req = request(pid(7), "Ann".into(), 40, moves, &[], &|_| String::new());
        let c = d.decide(&rule, moves, u, Some(&req));
        (c, d)
    }

    #[test]
    fn unanswered_and_refused_leave_the_rule_s_choice() {
        let p = PolityParams::core();
        let rule = RuleDeliberator { params: &p };
        let moves = [move_of(2.0), move_of(0.5)];
        for u in [0.05, 0.3, 0.6, 0.95] {
            let expected = rule.choose(&moves, u);
            for answer in [
                ModelAnswer::Unanswered,
                ModelAnswer::Refused,
                ModelAnswer::Move(2),
                ModelAnswer::Move(usize::MAX),
            ] {
                let (c, d) = ask(DeliberationMode::Model(Box::new(Fixed(answer))), &moves, u);
                assert_eq!(c.chosen, expected.chosen, "{answer:?} at {u}");
                assert_eq!(c.scores, expected.scores);
                assert_eq!(c.by, ChosenBy::RuleFallback);
                // Only a move rules made is kept, for the law's receipt.
                assert_eq!(d.answers.answers.len(), usize::from(c.chosen.is_some()));
            }
            let (c, d) = ask(DeliberationMode::Rule, &moves, u);
            assert_eq!(c, expected);
            assert!(d.answers.answers.is_empty());
        }
    }

    #[test]
    fn a_listed_answer_is_the_choice_and_is_saved() {
        let moves = [move_of(2.0), move_of(0.5)];
        let (c, d) = ask(
            DeliberationMode::Model(Box::new(Fixed(ModelAnswer::Move(1)))),
            &moves,
            0.01,
        );
        assert_eq!(c.chosen, Some(1));
        assert_eq!(c.by, ChosenBy::Model);
        assert_eq!(c.scores.len(), 3, "the rule's scores are kept");
        let a = d.answers.answers[0];
        assert_eq!(
            (a.notable, a.day, a.chosen, a.by),
            (pid(7), 40, Some(1), ChosenBy::Model)
        );
        assert_eq!(d.answers.providers[0].model, "fixed");
        assert_eq!(d.answers.chosen_by(pid(7), 40), Some(ChosenBy::Model));
        let (c, d) = ask(
            DeliberationMode::Model(Box::new(Fixed(ModelAnswer::None))),
            &moves,
            0.01,
        );
        assert_eq!((c.chosen, c.by), (None, ChosenBy::Model));
        assert_eq!(d.answers.answers[0].chosen, None);
        assert_eq!(d.answers.chosen_by(pid(7), 40), None, "no move, no law");
        // Read back, the same choice, without the provider.
        let (again, replayed) = ask(DeliberationMode::Recorded(d.answers.clone()), &moves, 0.01);
        assert_eq!((again.chosen, again.by), (None, ChosenBy::Model));
        assert_eq!(replayed.answers, d.answers);
        // Other moves match no saved answer: rules decide, and nothing is kept.
        let other = [move_of(3.0)];
        let (c, d2) = ask(DeliberationMode::Recorded(d.answers), &other, 0.01);
        assert_eq!(c.by, ChosenBy::Rule);
        assert!(d2.answers.answers.is_empty());
    }
}
