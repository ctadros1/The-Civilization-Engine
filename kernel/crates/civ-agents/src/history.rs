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
    /// A roof over the household's sleepers and stores.
    Shelter = 15,
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
    /// Excluded: the materials the work needs are not at home.
    NoMaterials = 113,
    /// Excluded: the household's home is built.
    Built = 114,
    /// Excluded: nothing being built needs it.
    NotNeeded = 115,
}

impl Reason {
    /// Every reason, for the observer's label table.
    pub const ALL: [Reason; 31] = [
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
        Reason::Shelter,
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
        Reason::NoMaterials,
        Reason::Built,
        Reason::NotNeeded,
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
            Reason::Shelter => "a roof before winter",
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
            Reason::NoMaterials => "nothing to build with at home",
            Reason::Built => "their home is built",
            Reason::NotNeeded => "nothing being built needs it",
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

impl Cause {
    /// Every cause.
    pub const ALL: [Cause; 3] = [Cause::Unspecified, Cause::Starvation, Cause::Childbirth];

    /// The key a chronicle entry keeps it as.
    pub fn key(self) -> &'static str {
        match self {
            Cause::Unspecified => "unspecified",
            Cause::Starvation => "starvation",
            Cause::Childbirth => "childbirth",
        }
    }

    /// The cause with a key.
    pub fn from_key(key: &str) -> Option<Cause> {
        Cause::ALL.into_iter().find(|c| c.key() == key)
    }

    /// In words, as the inspector says it.
    pub fn label(self) -> &'static str {
        match self {
            Cause::Unspecified => "illness or accident",
            Cause::Starvation => "hunger",
            Cause::Childbirth => "childbirth",
        }
    }
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
    /// When they left the world alive (their household gave up and left the valley).
    pub left: Option<SimTime>,
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
    /// A settlement's first home was roofed: `people` names who finished it.
    FirstRoof,
    /// A child was born: `people` is the child, its mother and (if known) its father.
    Born,
    /// Someone died: `people` is them, `number` their age in years and `name` the cause's key
    /// ([`Cause::key`]).
    Died,
    /// Two people became partners: `people` is the woman and the man, `number` where they live
    /// ([`Moved`] as a number).
    Paired,
    /// Children left without an older member of their household went to live with kin or
    /// neighbours: `people` is the eldest of the household that took them in, then the children.
    TakenIn,
    /// A household gave up and left the valley: `people` is its members, eldest first, and
    /// `number` how many they were.
    Left,
    /// The first trail out of a settlement was worn in: `number` is its length, metres, and
    /// `place` its middle.
    FirstTrail,
    /// The observer sent a family (god tool): `people` is the family, the mother and father first,
    /// and `number` how many they are.
    FamilyArrived,
}

/// Where a new couple went to live, in a [`ChronicleKind::Paired`] entry. Numeric in saves: append
/// only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// They set up a household of their own.
    NewHousehold = 0,
    /// The woman moved into the man's household.
    HerToHis = 1,
    /// The man moved into the woman's household.
    HisToHers = 2,
    /// They already lived together.
    Stayed = 3,
}

impl Moved {
    /// The value with this number.
    pub fn from_number(n: f64) -> Moved {
        match n.round() as i64 {
            1 => Moved::HerToHis,
            2 => Moved::HisToHers,
            3 => Moved::Stayed,
            _ => Moved::NewHousehold,
        }
    }
}

/// A couple, kept forever (research 04-08 §5.1: a union is its own record, not a household).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Union {
    /// The woman.
    pub woman: PermanentId,
    /// The man.
    pub man: PermanentId,
    /// When they became partners.
    pub since: SimTime,
    /// When it ended (one of them died), if it has.
    pub ended: Option<SimTime>,
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
    let person = |i: usize| event.people.get(i).map(|&id| Span::Person(id, name_of(id)));
    match event.kind {
        ChronicleKind::Born => {
            let Some(child) = person(0) else {
                return vec![Span::Text("A child was born.".to_owned())];
            };
            let mut spans = vec![child, Span::Text(" was born".to_owned())];
            match (person(1), person(2)) {
                (Some(mother), Some(father)) => {
                    spans.push(Span::Text(" to ".to_owned()));
                    spans.push(mother);
                    spans.push(Span::Text(" and ".to_owned()));
                    spans.push(father);
                }
                (Some(mother), None) => {
                    spans.push(Span::Text(" to ".to_owned()));
                    spans.push(mother);
                }
                _ => {}
            }
            spans.push(Span::Text(".".to_owned()));
            spans
        }
        ChronicleKind::Died => {
            let Some(who) = person(0) else {
                return vec![Span::Text("Someone died.".to_owned())];
            };
            let age = age_text(event.number);
            let how = match Cause::from_key(&event.name) {
                Some(Cause::Starvation) => format!(" died of hunger, {age}."),
                Some(Cause::Childbirth) => format!(" died in childbirth, {age}."),
                _ => format!(" died, {age}."),
            };
            vec![who, Span::Text(how)]
        }
        ChronicleKind::Paired => {
            let (Some(woman), Some(man)) = (person(0), person(1)) else {
                return vec![Span::Text("Two people became partners.".to_owned())];
            };
            let mut spans = vec![
                woman.clone(),
                Span::Text(" and ".to_owned()),
                man.clone(),
                Span::Text(" became partners".to_owned()),
            ];
            match Moved::from_number(event.number) {
                Moved::NewHousehold => spans.push(Span::Text(
                    " and set up a household of their own.".to_owned(),
                )),
                Moved::HerToHis => {
                    spans.push(Span::Text("; ".to_owned()));
                    spans.push(woman);
                    spans.push(Span::Text(" moved into his household.".to_owned()));
                }
                Moved::HisToHers => {
                    spans.push(Span::Text("; ".to_owned()));
                    spans.push(man);
                    spans.push(Span::Text(" moved into her household.".to_owned()));
                }
                Moved::Stayed => spans.push(Span::Text(".".to_owned())),
            }
            spans
        }
        ChronicleKind::Left => {
            let Some(eldest) = person(0) else {
                return vec![Span::Text("A household left.".to_owned())];
            };
            let n = event.number.round() as i64;
            let mut spans = vec![Span::Text("The household of ".to_owned()), eldest];
            spans.push(Span::Text(if n > 1 {
                format!(" ({n} people) gave up and left ")
            } else {
                " gave up and left ".to_owned()
            }));
            spans.push(settlement(event));
            spans.push(Span::Text(".".to_owned()));
            spans
        }
        ChronicleKind::TakenIn => {
            let Some(taker) = person(0) else {
                return vec![Span::Text("Children were taken in.".to_owned())];
            };
            let children: Vec<Span> = (1..event.people.len()).filter_map(person).collect();
            let mut spans = Vec::new();
            for (i, c) in children.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::Text(
                        if i + 1 == children.len() {
                            " and "
                        } else {
                            ", "
                        }
                        .to_owned(),
                    ));
                }
                spans.push(c.clone());
            }
            spans.push(Span::Text(" went to live in the household of ".to_owned()));
            spans.push(taker);
            spans.push(Span::Text(".".to_owned()));
            spans
        }
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
        ChronicleKind::FirstRoof => vec![
            Span::Text("The first hut at ".to_owned()),
            settlement(event),
            Span::Text(" was roofed.".to_owned()),
        ],
        ChronicleKind::FamilyArrived => {
            let (Some(woman), Some(man)) = (person(0), person(1)) else {
                return vec![Span::Text("The observer sent a family.".to_owned())];
            };
            let n = event.number.round() as i64;
            let mut spans = vec![
                woman,
                Span::Text(" and ".to_owned()),
                man,
                Span::Text(" came to ".to_owned()),
                settlement(event),
            ];
            spans.push(Span::Text(if n > 2 {
                format!(" with their family, {n} in all, sent by the observer.")
            } else {
                ", sent by the observer.".to_owned()
            }));
            spans
        }
        ChronicleKind::FirstTrail => vec![
            Span::Text("The first trail out of ".to_owned()),
            settlement(event),
            Span::Text(format!(
                " was worn in, {} m long.",
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

/// "aged 34", "aged 5 months", "aged 3 days".
pub fn age_text(years: f64) -> String {
    let days = (years * 365.0).floor().max(0.0);
    if days < 1.0 {
        "on the day of their birth".to_owned()
    } else if days < 60.0 {
        format!(
            "aged {} day{}",
            days as i64,
            if days < 2.0 { "" } else { "s" }
        )
    } else if years < 2.0 {
        format!("aged {} months", (years * 12.0).floor() as i64)
    } else {
        format!("aged {}", years.floor() as i64)
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
