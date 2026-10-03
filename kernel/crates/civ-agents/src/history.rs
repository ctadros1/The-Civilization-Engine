//! History (ADR-0003): person records kept forever, decision receipts kept in a ring per person,
//! and the chronicle. Explanations are recorded when a decision is made, never reconstructed, and
//! the observer only ever shows what the kernel rendered here.

use civ_core::{PermanentId, SimTime};

use crate::needs::Sex;
use crate::person::Target;

/// A consideration in a decision, or a reason an option was left out. Numeric on the wire and in
/// saves: append only, never renumber.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Reason {
    /// How hungry they are.
    Hunger = 1,
    /// How much they need sleep, given the time of day.
    Sleep = 2,
    /// How much they miss company, in the evening.
    Loneliness = 3,
    /// How short the household is of food, times what the trip would bring.
    FoodShortage = 4,
    /// The worth of useful work in itself.
    UsefulWork = 5,
    /// How short the household is of water.
    WaterShortage = 6,
    /// The time spent walking.
    Walking = 7,
    /// The effort of hard work when tired.
    Effort = 8,
    /// Work that would run into darkness.
    Darkness = 9,
    /// Rest.
    Rest = 10,
    /// Play.
    Play = 11,
    /// How short the household is of firewood, times what the trip would bring.
    FuelShortage = 12,
    /// The food field work brings for the year ahead.
    Harvest = 13,
    /// Field work left against the work the household can still do before the season closes.
    Deadline = 14,
    /// Excluded: there is no food at home.
    NoFood = 100,
    /// Excluded: too young.
    TooYoung = 101,
    /// Excluded: too old.
    TooOld = 102,
    /// Excluded: only done in daylight, and it is dark.
    NotInDark = 103,
    /// Excluded: nowhere known to do it.
    NoPlace = 104,
    /// Excluded: cannot be reached on foot.
    Unreachable = 105,
    /// Excluded: the household has no settlement hearth.
    NoHearth = 106,
    /// Excluded: not tired enough to sleep.
    NotTired = 107,
    /// Excluded: the food at home must be cooked, and there is no firewood.
    NoFire = 108,
    /// Excluded: no seed to sow.
    NoSeed = 109,
    /// Excluded: no field needs this work now.
    NoFieldWork = 110,
    /// Excluded: the household is not short of food.
    NotShort = 111,
    /// Excluded: no household nearby can spare food.
    NoOneToAsk = 112,
}

impl Reason {
    /// Every reason, for the observer's label table.
    pub const ALL: [Reason; 27] = [
        Reason::Hunger,
        Reason::Sleep,
        Reason::Loneliness,
        Reason::FoodShortage,
        Reason::UsefulWork,
        Reason::WaterShortage,
        Reason::Walking,
        Reason::Effort,
        Reason::Darkness,
        Reason::Rest,
        Reason::Play,
        Reason::FuelShortage,
        Reason::Harvest,
        Reason::Deadline,
        Reason::NoFood,
        Reason::TooYoung,
        Reason::TooOld,
        Reason::NotInDark,
        Reason::NoPlace,
        Reason::Unreachable,
        Reason::NoHearth,
        Reason::NotTired,
        Reason::NoFire,
        Reason::NoSeed,
        Reason::NoFieldWork,
        Reason::NotShort,
        Reason::NoOneToAsk,
    ];

    /// The reason with this code.
    pub fn from_code(code: u16) -> Option<Reason> {
        Reason::ALL.into_iter().find(|r| *r as u16 == code)
    }

    /// Plain-English label.
    pub fn label(self) -> &'static str {
        match self {
            Reason::Hunger => "hunger",
            Reason::Sleep => "tiredness",
            Reason::Loneliness => "wanting company",
            Reason::FoodShortage => "food running short",
            Reason::UsefulWork => "useful work",
            Reason::WaterShortage => "water running short",
            Reason::Walking => "the walk",
            Reason::Effort => "effort when tired",
            Reason::Darkness => "darkness falling",
            Reason::Rest => "rest",
            Reason::Play => "play",
            Reason::FuelShortage => "firewood running short",
            Reason::Harvest => "food for the year ahead",
            Reason::Deadline => "the season will not wait",
            Reason::NoFood => "no food at home",
            Reason::TooYoung => "too young",
            Reason::TooOld => "too old",
            Reason::NotInDark => "not done in the dark",
            Reason::NoPlace => "nowhere known to do it",
            Reason::Unreachable => "cannot be reached on foot",
            Reason::NoHearth => "no hearth to sit at",
            Reason::NotTired => "not tired",
            Reason::NoFire => "no fire to cook on",
            Reason::NoSeed => "no seed to sow",
            Reason::NoFieldWork => "no field needs it now",
            Reason::NotShort => "not short of food",
            Reason::NoOneToAsk => "no one nearby can spare food",
        }
    }
}

/// One consideration's contribution to an option's utility, in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Term {
    /// What it is.
    pub reason: Reason,
    /// Points it added (negative for costs).
    pub points: f32,
}

/// An option as it was scored.
#[derive(Clone, Debug, PartialEq)]
pub struct Scored {
    /// The activity, by catalog index.
    pub def: u16,
    /// Its target.
    pub target: Target,
    /// Total utility, points: the sum of the terms.
    pub total: f32,
    /// The considerations, largest first.
    pub terms: Vec<Term>,
}

/// Why a person chose what they did, recorded at the moment of choice (ADR-0003).
#[derive(Clone, Debug, PartialEq)]
pub struct Receipt {
    /// When the choice was made.
    pub at: SimTime,
    /// What was chosen.
    pub chosen: Scored,
    /// The strongest alternative.
    pub runner_up: Option<Scored>,
    /// The next alternatives' totals: `(activity, total)`.
    pub others: Vec<(u16, f32)>,
    /// Options left out and why: `(activity, reason)`.
    pub excluded: Vec<(u16, Reason)>,
    /// Probability the chosen option had.
    pub probability: f32,
    /// Softmax temperature used, points.
    pub temperature: f32,
    /// Needs at the time: hunger, sleep drive, loneliness (0–1 scales), and the household's days
    /// of food and water.
    pub needs: [f32; 5],
}

/// How a person came to be in the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// One of the founding band.
    Founder,
    /// Born here.
    Born,
    /// Brought by the observer's god tool.
    Spawned,
}

/// Why someone died. Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// Not specified (the all-cause life table).
    Unspecified,
    /// Starvation.
    Starvation,
    /// In childbirth.
    Childbirth,
}

/// A person, kept forever (ADR-0003).
#[derive(Clone, Debug, PartialEq)]
pub struct PersonRecord {
    /// Permanent id.
    pub id: PermanentId,
    /// Given name.
    pub given: String,
    /// Sex.
    pub sex: Sex,
    /// Birth time.
    pub born: SimTime,
    /// Death time and cause.
    pub died: Option<(SimTime, Cause)>,
    /// Mother.
    pub mother: Option<PermanentId>,
    /// Father.
    pub father: Option<PermanentId>,
    /// How they came to be here.
    pub origin: Origin,
}

/// What happened, in the chronicle. Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChronicleKind {
    /// A founding band arrived: `people` lists them, `number` is their count.
    BandArrived,
    /// A settlement was founded: `settlement` and its `place`.
    SettlementFounded,
    /// A settlement's food ran short: `number` is the days of food left.
    FoodRanShort,
    /// A settlement had enough food again: `number` is the days of food in store.
    FoodRecovered,
    /// A settlement sowed its first field: `people` names who finished it.
    FirstSowing,
    /// A settlement's harvest was in: `number` is the grain threshed, kilograms.
    HarvestIn,
}

/// A chronicle entry: structured facts, rendered to text when read (ADR-0003).
#[derive(Clone, Debug, PartialEq)]
pub struct ChronicleEvent {
    /// Sequence number, from 1.
    pub seq: u64,
    /// When it happened.
    pub at: SimTime,
    /// What happened.
    pub kind: ChronicleKind,
    /// The people involved.
    pub people: Vec<PermanentId>,
    /// The settlement involved.
    pub settlement: Option<PermanentId>,
    /// Where, metres.
    pub place: Option<(f32, f32)>,
    /// A number the entry reports.
    pub number: f64,
    /// A name the entry reports (for example a settlement's).
    pub name: String,
}

/// A piece of rendered chronicle text.
#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    /// Plain text.
    Text(String),
    /// A link to a person.
    Person(PermanentId, String),
    /// A link to a settlement.
    Settlement(PermanentId, String),
}

/// Renders an entry. `name_of` gives a person's name (alive or dead).
pub fn render(event: &ChronicleEvent, name_of: &dyn Fn(PermanentId) -> String) -> Vec<Span> {
    let _ = name_of;
    match event.kind {
        ChronicleKind::BandArrived => vec![Span::Text(format!(
            "A band of {} people arrived.",
            event.number as u64
        ))],
        ChronicleKind::SettlementFounded => {
            let mut spans = vec![Span::Text("They made camp at ".to_owned())];
            spans.push(settlement(event));
            spans.push(Span::Text(".".to_owned()));
            spans
        }
        ChronicleKind::FoodRanShort => vec![
            Span::Text("Food ran short at ".to_owned()),
            settlement(event),
            Span::Text(format!(": {} left.", days_text(event.number))),
        ],
        ChronicleKind::FoodRecovered => vec![
            Span::Text("There was enough food again at ".to_owned()),
            settlement(event),
            Span::Text(format!(": {} in store.", days_text(event.number))),
        ],
        ChronicleKind::FirstSowing => vec![
            Span::Text("The first field was sown at ".to_owned()),
            settlement(event),
            Span::Text(".".to_owned()),
        ],
        ChronicleKind::HarvestIn => vec![
            Span::Text("The harvest at ".to_owned()),
            settlement(event),
            Span::Text(format!(
                " was in: {} kg of grain.",
                thousands(event.number.round().max(0.0) as u64)
            )),
        ],
    }
}

/// "12,345".
fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn settlement(event: &ChronicleEvent) -> Span {
    match event.settlement {
        Some(id) => Span::Settlement(id, event.name.clone()),
        None => Span::Text(event.name.clone()),
    }
}

fn days_text(days: f64) -> String {
    if days < 1.0 {
        "less than a day's food".to_owned()
    } else if days < 1.5 {
        "about a day's food".to_owned()
    } else {
        format!("about {} days' food", days.round() as i64)
    }
}

/// Plain text of rendered spans.
pub fn plain(spans: &[Span]) -> String {
    spans
        .iter()
        .map(|s| match s {
            Span::Text(t) => t.as_str(),
            Span::Person(_, n) | Span::Settlement(_, n) => n.as_str(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_codes_round_trip_and_are_unique() {
        let mut codes: Vec<u16> = Reason::ALL.iter().map(|r| *r as u16).collect();
        for r in Reason::ALL {
            assert_eq!(Reason::from_code(r as u16), Some(r));
            assert!(!r.label().is_empty());
        }
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), Reason::ALL.len());
    }

    #[test]
    fn chronicle_entries_render_with_links() {
        let id = PermanentId::from_raw(7).expect("non-zero");
        let e = ChronicleEvent {
            seq: 2,
            at: SimTime::ZERO,
            kind: ChronicleKind::SettlementFounded,
            people: Vec::new(),
            settlement: Some(id),
            place: Some((1.0, 2.0)),
            number: 0.0,
            name: "Alder Ford".to_owned(),
        };
        let spans = render(&e, &|_| String::new());
        assert!(spans.contains(&Span::Settlement(id, "Alder Ford".to_owned())));
        assert_eq!(plain(&spans), "They made camp at Alder Ford.");
    }
}
