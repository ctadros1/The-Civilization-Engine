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
    /// The tools the household's work needs.
    Tools = 16,
    /// Food ready to eat, or a step nearer it, running short.
    ReadyFood = 17,
    /// The household's stores will not last until its next harvest is in.
    LeanSeason = 18,
    /// Others want it and nobody offers it: made to sell.
    ForSale = 19,
    /// What the work is paid, for the household (slice J).
    Wages = 20,
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
    /// Excluded: the household has no free tool for the work.
    NoTool = 116,
    /// Excluded: what the recipe takes is not at home.
    NoInputs = 117,
    /// Excluded: a better way to do the same work is at hand (a tool for what is otherwise done
    /// by hand).
    BetterWay = 118,
    /// Excluded: it costs less to get it from another household than to make it.
    Cheaper = 119,
    /// Excluded: nobody nearby offers what the household needs on terms it can meet.
    NoOffer = 120,
    /// Excluded: no workshop nearby is hiring (slice J).
    NoWork = 121,
    /// To learn the work from someone who knows it, by working beside them (ADR-0008 §4).
    Learning = 21,
    /// Excluded: they do not know how, and nobody they could learn from is at the work
    /// (ADR-0008 §1).
    DoesNotKnow = 122,
    /// To find a way to answer a problem at home: food lost to spoiling (ADR-0008 §3).
    Problem = 22,
    /// To keep food that would otherwise spoil before it is eaten (ADR-0008 §3's crafts).
    Spoiling = 23,
    /// Excluded: nothing at home calls for trying something new.
    NoProblem = 123,
    /// Excluded: they tried something new lately.
    TriedLately = 124,
    /// Excluded: the ground is too wet to work today (ADR-0012 §5).
    WetGround = 125,
    /// Excluded: snow lies on the ground.
    SnowCover = 126,
    /// Excluded: the ground is frozen.
    FrozenGround = 127,
    /// To have a say at the gathering called at the hearth: what it decides is worth to the
    /// household (M4a slice Z, ADR-0013 §1).
    Gathering = 24,
    /// Excluded: no gathering is sitting that they belong to.
    NoGathering = 128,
    /// Their objection to taking what is not theirs (M4b slice AA, ADR-0015 §2).
    Objection = 25,
    /// The chance they believe they run of being seen, and what being seen would cost them.
    Risk = 26,
    /// Their regard for those they would take from.
    Regard = 27,
    /// Excluded: they would not take what is not theirs (a moral filter, research 04-09 §5.3).
    WouldNotTake = 129,
    /// Excluded: no household within reach has food to take.
    NothingToTake = 130,
    /// Excluded: they turned back or fled from a store lately, and wait to try again.
    TurnedBackLately = 131,
    /// The watch a law names them to keep (M4b slice AC, ADR-0015 §6).
    Duty = 28,
    /// Excluded: they keep no watch, it is not dark, or they have walked tonight's rounds.
    NoWatch = 132,
    /// A curfew they know of forbids being away from home now (M4b slice AD): what keeping it
    /// weighs with them.
    Curfew = 29,
    /// To join a petition a faction called at the hearth (M4c slice AH): their grievance, their
    /// identification with it and those they expect to come (research 04-10 §5.3).
    Petition = 30,
    /// Excluded: no petition they heard of sits now.
    NoPetition = 133,
    /// Excluded: a blow keeps them from work (M4c slice AI, step four).
    Hurt = 134,
    /// Those they would see at another settlement's hearth (M5a slice AM): kin living there,
    /// those they know there, and, for one who found no partner at home, the hope of meeting
    /// someone.
    Company = 31,
    /// Excluded: no other settlement their household knows lies within a day's walk there and
    /// back, in daylight.
    NoPlaceToVisit = 135,
    NoReport = 136,
    /// Excluded: someone of their household is already on the way to buy elsewhere (M5b slice
    /// AP).
    AnotherGoes = 137,
    /// Excluded: the observer has stopped purchases between settlements (the demo's twin, M5b
    /// slice AQ).
    Stopped = 138,
}

impl Reason {
    /// Every reason, for the observer's label table.
    pub const ALL: [Reason; 70] = [
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
        Reason::Tools,
        Reason::ReadyFood,
        Reason::LeanSeason,
        Reason::ForSale,
        Reason::Wages,
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
        Reason::NoTool,
        Reason::NoInputs,
        Reason::BetterWay,
        Reason::Cheaper,
        Reason::NoOffer,
        Reason::NoWork,
        Reason::Learning,
        Reason::DoesNotKnow,
        Reason::Problem,
        Reason::Spoiling,
        Reason::NoProblem,
        Reason::TriedLately,
        Reason::WetGround,
        Reason::SnowCover,
        Reason::FrozenGround,
        Reason::Gathering,
        Reason::NoGathering,
        Reason::Objection,
        Reason::Risk,
        Reason::Regard,
        Reason::WouldNotTake,
        Reason::NothingToTake,
        Reason::TurnedBackLately,
        Reason::Duty,
        Reason::NoWatch,
        Reason::Curfew,
        Reason::Petition,
        Reason::NoPetition,
        Reason::Hurt,
        Reason::Company,
        Reason::NoPlaceToVisit,
        Reason::NoReport,
        Reason::AnotherGoes,
        Reason::Stopped,
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
            Reason::Tools => "tools for the work",
            Reason::ReadyFood => "food to make ready",
            Reason::LeanSeason => "stores will not last to the harvest",
            Reason::ForSale => "others want it",
            Reason::Wages => "what the work is paid",
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
            Reason::NoTool => "no tool for it at home",
            Reason::NoInputs => "nothing to make it from at home",
            Reason::BetterWay => "a better way to do it is at hand",
            Reason::Cheaper => "it costs less to get it from a neighbour",
            Reason::NoOffer => "nobody nearby offers what is needed",
            Reason::NoWork => "nobody nearby is hiring",
            Reason::Learning => "learning it from someone who knows",
            Reason::DoesNotKnow => "does not know how, and nobody to learn from is at it",
            Reason::Problem => "a problem at home it might answer",
            Reason::Spoiling => "before it spoils",
            Reason::NoProblem => "nothing at home calls for it",
            Reason::TriedLately => "tried something new lately",
            Reason::WetGround => "the ground too wet to work",
            Reason::SnowCover => "snow on the ground",
            Reason::FrozenGround => "the ground frozen",
            Reason::Gathering => "a say at the gathering",
            Reason::NoGathering => "no gathering is sitting",
            Reason::Objection => "taking what is not theirs",
            Reason::Risk => "the chance of being seen",
            Reason::Regard => "regard for those taken from",
            Reason::WouldNotTake => "would not take what is not theirs",
            Reason::NothingToTake => "no store within reach to take from",
            Reason::TurnedBackLately => "turned back from a store lately",
            Reason::Duty => "the watch they keep",
            Reason::NoWatch => "no watch of theirs to keep now",
            Reason::Curfew => "a curfew forbids being away from home now",
            Reason::Petition => "the petition at the hearth",
            Reason::NoPetition => "no petition they heard of sits now",
            Reason::Hurt => "a blow keeps them from work",
            Reason::Company => "those they would see there",
            Reason::NoPlaceToVisit => "no settlement they know within a day's walk",
            Reason::NoReport => "no offer elsewhere they know of is worth the walk",
            Reason::AnotherGoes => {
                "someone of their household is already on the way to buy elsewhere"
            }
            Reason::Stopped => "the observer has stopped purchases between settlements",
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
    /// When a building gave way around them (ADR-0009 §5).
    Collapse,
    /// Of a blow another struck (M4c slice AI, step four; ADR-0017 §3): the encounter that
    /// records who struck it.
    Violence,
}

impl Cause {
    /// Every cause.
    pub const ALL: [Cause; 5] = [
        Cause::Unspecified,
        Cause::Starvation,
        Cause::Childbirth,
        Cause::Collapse,
        Cause::Violence,
    ];

    /// The key a chronicle entry keeps it as.
    pub fn key(self) -> &'static str {
        match self {
            Cause::Unspecified => "unspecified",
            Cause::Starvation => "starvation",
            Cause::Childbirth => "childbirth",
            Cause::Collapse => "collapse",
            Cause::Violence => "violence",
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
            Cause::Collapse => "a building's collapse",
            Cause::Violence => "a blow struck by another",
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
    /// Where they have lived, oldest first: each settlement (none: off the map), since when and
    /// why (ADR-0018 §2). The last is where they live now, or where they last lived.
    pub residence: Vec<Stay>,
}

impl PersonRecord {
    /// The settlement they lived in at `t` (after everything at an earlier minute), if any.
    pub fn residence_at(&self, t: SimTime) -> Option<PermanentId> {
        self.residence
            .iter()
            .take_while(|r| r.since < t)
            .last()
            .and_then(|r| r.settlement)
    }
}

/// Why someone came to live where they did, or went (ADR-0018 §2). Numeric in saves: append
/// only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidenceWhy {
    /// Born to a household there.
    Born,
    /// One of a founding group the world was made with.
    Founder,
    /// Came from off the map: a family or an agitator the observer sent, or a wave.
    Arrived,
    /// Their household moved there from another settlement.
    Moved,
    /// Joined a household there on marrying.
    Married,
    /// Taken in by kin there when nobody older was left in their own household.
    TakenIn,
    /// Sent away by a finding.
    Exiled,
    /// Their household gave up and left the map.
    LeftMap,
    /// Their household went with others to found it (M5a slice AO).
    Founded,
}

impl ResidenceWhy {
    /// Every reason, in code order.
    pub const ALL: [ResidenceWhy; 9] = [
        ResidenceWhy::Born,
        ResidenceWhy::Founder,
        ResidenceWhy::Arrived,
        ResidenceWhy::Moved,
        ResidenceWhy::Married,
        ResidenceWhy::TakenIn,
        ResidenceWhy::Exiled,
        ResidenceWhy::LeftMap,
        ResidenceWhy::Founded,
    ];

    /// Its code in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The reason with a code.
    pub fn from_code(code: u8) -> Option<ResidenceWhy> {
        ResidenceWhy::ALL.get(usize::from(code)).copied()
    }

    /// In words, as the inspector says it.
    pub fn words(self) -> &'static str {
        match self {
            ResidenceWhy::Born => "born there",
            ResidenceWhy::Founder => "came with a founding group",
            ResidenceWhy::Arrived => "came from beyond the map",
            ResidenceWhy::Moved => "moved there with their household",
            ResidenceWhy::Married => "married into a household there",
            ResidenceWhy::TakenIn => "taken in by kin there",
            ResidenceWhy::Exiled => "sent away by a finding",
            ResidenceWhy::LeftMap => "left with their household",
            ResidenceWhy::Founded => "went with others to found it",
        }
    }
}

/// One stay in someone's residence history (ADR-0018 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stay {
    /// The settlement, or none: off the map.
    pub settlement: Option<PermanentId>,
    /// From when.
    pub since: SimTime,
    /// Why.
    pub why: ResidenceWhy,
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
    /// A household gave up and left the valley: `people` is its members, eldest first,
    /// `number` how many they were, and `name` the settlement they left (empty before
    /// 2026-10-07, and when they had none).
    Left,
    /// The first trail out of a settlement was worn in: `number` is its length, metres, and
    /// `place` its middle.
    FirstTrail,
    /// The observer sent a family (god tool): `people` is the family, the mother and father first,
    /// and `number` how many they are.
    FamilyArrived,
    /// A household set up a workshop (slice J): `people` is who founded it, `firm` the workshop
    /// and `name` the good it makes.
    WorkshopOpened,
    /// A workshop closed: `people` is who founded it, `firm` the workshop, `name` the good it
    /// made and `number` why ([`crate::firm::Exit`] as a number).
    WorkshopClosed,
    /// Someone found a technique (ADR-0008 §3): `people` is the finder, `name` the technique and
    /// `number` what kind of find it was: [`FOUND_FIRST_ANYWHERE`], [`FOUND_FIRST_HERE`],
    /// [`FOUND_AGAIN`] or [`FOUND_KNOWN_HERE`].
    TechniqueFound,
    /// Someone learnt a craft by working beside someone who knew it (ADR-0008 §4): `people` is
    /// the learner and the teacher, `name` the technique.
    TechniqueLearned,
    /// The last person in a settlement who knew a technique died or left (ADR-0008 §5): `people`
    /// is them, `name` the technique, and `number` flags: 1 if someone there still knows of it or
    /// is learning it, 2 if goods or buildings made with it remain.
    TechniqueLost,
    /// The observer introduced a technique (god tool, ADR-0008 §6): `people` is the person,
    /// `name` the technique, `number` 1 if they only heard of it.
    TechniqueIntroduced,
    /// A building, or a part of it, gave way (ADR-0009 §5): `people` is the eldest of its
    /// household then those it killed, `place` the building, `number` how many it killed, and
    /// `name` what gave way, under what and why, in words ("longhouse lost its loft: the joists
    /// broke under 1.9 t of grain; they were poorly made").
    BuildingFailed,
    /// A settlement found a deposit (ADR-0010 §1): `people` is who found it, `place` the body,
    /// and `name` what was found in words ("clay showing at the surface").
    DepositFound,
    /// The observer laid down a deposit (the god tool, ADR-0010 §1): `place` the body, `settlement`
    /// the nearest settlement if any, and `name` what it is in words ("clay under the ground").
    DepositPlaced,
    /// A month, a winter or a year of weather on the valley floor stood out against what it
    /// usually brings (ADR-0012): `number` says what stood out ([`civ_land::weather::stood_out`]
    /// flags) and `name` says it in words ("October was wet and cold on the valley floor: 168 mm
    /// fell, 2.1 times what October usually brings; a mean of 3.5 °C, 6.0 °C below its usual.").
    Weather,
    /// Someone put a law to the gathering (ADR-0013 §3, stage 1): `people` is the sponsor,
    /// `number` the levy share, and `name` what they proposed and why, in words ("a common
    /// store, a tenth of each harvest, because food ran short").
    LawProposed,
    /// A gathering decided on a law (ADR-0013 §3, stage 3): `people` is the sponsor, `number`
    /// how it went ([`crate::polity::Outcome`] as a number) and `name` the whole of it in words
    /// ("The gathering at Ashford passed a common store, a tenth of each harvest: 14 for, 3
    /// against; 19 of 40 adults came.").
    LawDecided,
    /// A law naming someone lapsed because they died or left: `people` is them, `name` it in
    /// words ("Ada no longer keeps the common store at Ashford: they died.").
    LawLapsed,
    /// Someone was seen taking food from another household's store (M4b slice AA): `people` is
    /// the taker and then those who saw, `number` the kilograms taken, and `name` the rest in
    /// words ("took 12 kg of grain from the household of Rilla; Bram saw it.").
    Taking,
    /// A demand to give back what was taken ended (ADR-0015 §5): `people` is the taker, `number`
    /// the obligation's standing ([`crate::crime::Standing`] as a number), and `name` it in words
    /// ("The household of Tam gave back the food taken from the household of Rilla.").
    Restitution,
    /// Someone brought a case before the gathering (M4b slice AB, ADR-0015 §4): `people` is the
    /// one who brought it and then the accused, `number` how many witnesses its accounts come
    /// from, and `name` the rest in words ("brought a case before the gathering at Ashford: that
    /// Tam took food from their household, on the word of Bram.").
    CaseBrought,
    /// The gathering heard a case, or it lapsed: `people` is the one who brought it and the
    /// accused, `number` the case's stage ([`crate::crime::CaseStage`] as a number), and `name`
    /// the whole of it in words.
    CaseHeard,
    /// The custom changed by its own procedure (M4c slice AF, ADR-0017 §1: an amendment, not a
    /// replacement): `people` is the sponsor, `number` the custom's version now, and `name` the
    /// whole of it in words ("The custom at Ashford changed by its own procedure, on Ada's
    /// proposal: from now on, the elders of its households ...").
    CustomAmended,
    /// A faction called on everyone to stand with its body in place of the gathering's (M4c slice
    /// AI): `people` is its organizer, `name` the whole of it in words.
    RevoltCalled,
    /// The custom was taken from the gathering, not amended (M4c slice AI, ADR-0017 §1: a
    /// replacement): `people` is the one who called it, `number` the custom's version now, and
    /// `name` the whole of it in words.
    CustomTaken,
    /// A faction's call to stand with its body came to nothing (M4c slice AI): `people` is the
    /// one who called it, `name` the whole of it in words.
    RevoltFailed,
    /// One who keeps the watch called on the others to take the deciding for it (M4c slice AI,
    /// step three): `people` is the one who called it, `name` the whole of it in words.
    CoupCalled,
    /// A call for the watch to take the deciding came to nothing (M4c slice AI, step three):
    /// `people` is the one who called it, `name` the whole of it in words.
    CoupFailed,
    /// One who keeps the watch came to take what a refused finding owed (M4c slice AI, step four;
    /// research 06-10 §4.C): `people` is the watcher, then the household's elder; `number` the
    /// food taken, kcal; `name` the whole of it in words, who met them how and every blow.
    Encounter,
    /// One recorded influence (M4c slice AJ, ADR-0016 §5): the observer reached someone by a god
    /// tool. `people` is whom it reached; `number` the tool, by [`crate::influence::InfluenceKind`]
    /// code; `name` what followed their name, in words (" that a gathering meets …", " of common
    /// provision.").
    Influence,
    /// A draw a blessing or a curse turned (M4c slice AJ): `people` is whose draw it was;
    /// `number` the influence's number; `name` what followed their name, in words (" was spared
    /// a death by illness or accident: the observer's blessing turned the draw.").
    InfluenceTurned,
    /// A household moved to another settlement (M5a slice AN, ADR-0018 §5): `people` are its
    /// members, eldest first as they were; `settlement` and `name` where it went; `number` how
    /// many.
    Moved,
    /// Households gathered to found a settlement (M5a slice AO): `people` are the organizing
    /// household's members; `settlement` the one they would leave, or for a founding the one
    /// founded; `pos` the site; `number` what happened ([`CoalitionStep`]); `name` the settlement
    /// left, for a founding, else empty.
    Coalition,
}

/// What happened to a coalition, in a [`ChronicleKind::Coalition`] entry. Numeric in saves:
/// append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoalitionStep {
    /// It began to gather households.
    Began = 0,
    /// Its time came and it lacked food to go.
    LackedFood = 1,
    /// Its time came and it lacked seed.
    LackedSeed = 2,
    /// Its organizer gave the plan up.
    GaveUp = 3,
    /// It went and founded its settlement.
    Founded = 4,
}

impl CoalitionStep {
    /// The step numbered `n`.
    pub fn from_number(n: f64) -> Option<CoalitionStep> {
        [
            CoalitionStep::Began,
            CoalitionStep::LackedFood,
            CoalitionStep::LackedSeed,
            CoalitionStep::GaveUp,
            CoalitionStep::Founded,
        ]
        .get(n.round().max(0.0) as usize)
        .copied()
    }
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
    /// The firm the entry is about (slice J).
    pub firm: Option<PermanentId>,
}

/// A find (ADR-0008 §3) that is the first anywhere: the `number` of a `TechniqueFound` entry.
pub const FOUND_FIRST_ANYWHERE: i64 = 2;
/// A find that is the first in the finder's settlement, of a technique known elsewhere.
pub const FOUND_FIRST_HERE: i64 = 1;
/// A find of a technique lost in the finder's settlement before.
pub const FOUND_AGAIN: i64 = 0;
/// A find of a technique others in the finder's settlement already knew.
pub const FOUND_KNOWN_HERE: i64 = 3;

/// A piece of rendered chronicle text.
#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    /// Plain text.
    Text(String),
    /// A link to a person.
    Person(PermanentId, String),
    /// A link to a settlement.
    Settlement(PermanentId, String),
    /// A link to a firm (slice J).
    Firm(PermanentId, String),
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
                Some(Cause::Collapse) => format!(" died when a building gave way, {age}."),
                Some(Cause::Violence) => format!(" died of a blow, {age}."),
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
                    " and set up a household of their own".to_owned(),
                )),
                Moved::HerToHis => {
                    spans.push(Span::Text("; ".to_owned()));
                    spans.push(woman);
                    spans.push(Span::Text(" moved into his household".to_owned()));
                }
                Moved::HisToHers => {
                    spans.push(Span::Text("; ".to_owned()));
                    spans.push(man);
                    spans.push(Span::Text(" moved into her household".to_owned()));
                }
                Moved::Stayed => {}
            }
            // Partners of two settlements (M5a slice AM): where they settled is named.
            if !event.name.is_empty() {
                spans.push(Span::Text(" in ".to_owned()));
                spans.push(settlement(event));
            }
            spans.push(Span::Text(".".to_owned()));
            spans
        }
        ChronicleKind::Coalition => {
            let who = person(0).unwrap_or_else(|| Span::Text("a household".to_owned()));
            match CoalitionStep::from_number(event.number) {
                Some(CoalitionStep::Began) => vec![
                    Span::Text("The household of ".to_owned()),
                    who,
                    Span::Text(format!(
                        " began gathering households to found a settlement{}",
                        event.name
                    )),
                    settlement(event),
                    Span::Text(".".to_owned()),
                ],
                Some(step @ (CoalitionStep::LackedFood | CoalitionStep::LackedSeed)) => vec![
                    Span::Text("The households gathered by the household of ".to_owned()),
                    who,
                    Span::Text(format!(
                        " held too little {} to go and found their settlement yet.",
                        if step == CoalitionStep::LackedFood {
                            "food"
                        } else {
                            "seed"
                        }
                    )),
                ],
                Some(CoalitionStep::GaveUp) => vec![
                    Span::Text("The household of ".to_owned()),
                    who,
                    Span::Text(" gave up gathering households to found a settlement.".to_owned()),
                ],
                Some(CoalitionStep::Founded) => vec![
                    Span::Text("The household of ".to_owned()),
                    who,
                    Span::Text(event.name.clone()),
                    settlement(event),
                    Span::Text(".".to_owned()),
                ],
                None => vec![Span::Text(
                    "Households gathered to found a settlement.".to_owned(),
                )],
            }
        }
        ChronicleKind::Moved => {
            let Some(eldest) = person(0) else {
                return vec![Span::Text("A household moved away.".to_owned())];
            };
            let n = event.number.round() as i64;
            let mut spans = vec![Span::Text("The household of ".to_owned()), eldest];
            spans.push(Span::Text(if n > 1 {
                format!(" ({n} people) moved to ")
            } else {
                " moved to ".to_owned()
            }));
            spans.push(settlement(event));
            spans.push(Span::Text(".".to_owned()));
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
            // Entries made before 2026-10-07 kept no name: they left the valley.
            if event.name.is_empty() {
                spans.push(Span::Text("the valley".to_owned()));
            } else {
                spans.push(settlement(event));
            }
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
        ChronicleKind::WorkshopOpened => {
            let what = workshop(&event.name);
            let link = match event.firm {
                Some(id) => Span::Firm(id, format!("a {what}")),
                None => Span::Text(format!("a {what}")),
            };
            match person(0) {
                Some(who) => vec![
                    who,
                    Span::Text(" set up ".to_owned()),
                    link,
                    Span::Text(".".to_owned()),
                ],
                None => vec![Span::Text(format!("A {what} was set up."))],
            }
        }
        ChronicleKind::WorkshopClosed => {
            let what = workshop(&event.name);
            let founder = event.people.first().map(|&id| name_of(id));
            let title = match founder {
                Some(n) => format!("{n}'s {what}"),
                None => format!("A {what}"),
            };
            let why = crate::firm::Exit::from_code(event.number.round() as u8)
                .map_or_else(String::new, |e| format!(": {}", e.text()));
            let link = match event.firm {
                Some(id) => Span::Firm(id, title),
                None => Span::Text(title),
            };
            vec![link, Span::Text(format!(" closed{why}."))]
        }
        ChronicleKind::TechniqueFound => {
            let what = event.name.to_lowercase();
            let first = match event.number.round() as i64 {
                FOUND_FIRST_ANYWHERE => ", the first anyone has",
                FOUND_FIRST_HERE => ", the first here",
                FOUND_KNOWN_HERE => ", which others here already knew",
                _ => ", lost here before",
            };
            match person(0) {
                Some(who) => vec![who, Span::Text(format!(" worked out {what}{first}."))],
                None => vec![Span::Text(format!("Someone worked out {what}{first}."))],
            }
        }
        ChronicleKind::TechniqueLearned => {
            let what = event.name.to_lowercase();
            match (person(0), person(1)) {
                (Some(learner), Some(teacher)) => vec![
                    learner,
                    Span::Text(format!(" learnt {what} from ")),
                    teacher,
                    Span::Text(".".to_owned()),
                ],
                (Some(learner), None) => vec![learner, Span::Text(format!(" learnt {what}."))],
                _ => vec![Span::Text(format!("Someone learnt {what}."))],
            }
        }
        ChronicleKind::TechniqueLost => {
            let flags = event.number.round().max(0.0) as u32;
            let mut tail = String::from(": nobody else here knew it.");
            if flags & 1 != 0 {
                tail.push_str(" Some here still know of it.");
            }
            if flags & 2 != 0 {
                tail.push_str(" What was made with it remains.");
            }
            if flags & 4 != 0 {
                tail.push_str(" It is still known where some here have kin or friends.");
            }
            match person(0) {
                Some(last) => vec![
                    Span::Text(format!("{} was lost with ", event.name)),
                    last,
                    Span::Text(tail),
                ],
                None => vec![Span::Text(format!("{} was lost here{tail}", event.name))],
            }
        }
        ChronicleKind::TechniqueIntroduced => {
            let what = event.name.to_lowercase();
            let (before, after) = if event.number.round() as i64 == 1 {
                ("The observer told ", format!(" of {what}."))
            } else {
                ("The observer taught ", format!(" {what}."))
            };
            match person(0) {
                Some(who) => vec![Span::Text(before.to_owned()), who, Span::Text(after)],
                None => vec![Span::Text(format!("The observer introduced {what}."))],
            }
        }
        ChronicleKind::BuildingFailed => {
            let mut spans = match person(0) {
                Some(owner) => vec![owner, Span::Text(format!("'s {}.", event.name))],
                None => vec![Span::Text(format!("A {}.", event.name))],
            };
            let dead = event.number.round().max(0.0) as usize;
            if dead > 0 {
                let killed: Vec<Span> = (1..=dead).filter_map(person).collect();
                spans.push(Span::Text(" It killed ".to_owned()));
                let n = killed.len();
                for (k, p) in killed.into_iter().enumerate() {
                    if k > 0 {
                        spans.push(Span::Text(
                            if k + 1 == n { " and " } else { ", " }.to_owned(),
                        ));
                    }
                    spans.push(p);
                }
                spans.push(Span::Text(".".to_owned()));
            }
            spans
        }
        ChronicleKind::DepositFound => match person(0) {
            Some(finder) => vec![finder, Span::Text(format!(" found {}.", event.name))],
            None => vec![Span::Text(format!("Found {}.", event.name))],
        },
        ChronicleKind::DepositPlaced => {
            vec![Span::Text(format!(
                "The observer laid down {}.",
                event.name
            ))]
        }
        ChronicleKind::Influence => {
            let verb = match crate::influence::InfluenceKind::from_code(event.number.round() as u8)
            {
                Some(crate::influence::InfluenceKind::Whisper) => "whispered to ",
                Some(crate::influence::InfluenceKind::Ideology) => "told ",
                Some(
                    crate::influence::InfluenceKind::Agitator
                    | crate::influence::InfluenceKind::Wave,
                ) => "sent ",
                Some(crate::influence::InfluenceKind::Bless) => "blessed ",
                Some(crate::influence::InfluenceKind::Curse) => "cursed ",
                None => "reached ",
            };
            let lead = format!("One recorded influence: the observer {verb}");
            match person(0) {
                Some(who) => vec![Span::Text(lead), who, Span::Text(event.name.clone())],
                None => vec![Span::Text(format!("{lead}someone{}", event.name))],
            }
        }
        ChronicleKind::InfluenceTurned => {
            let lead = "One recorded influence: ".to_owned();
            match person(0) {
                Some(who) => vec![Span::Text(lead), who, Span::Text(event.name.clone())],
                None => vec![Span::Text(format!("{lead}someone{}", event.name))],
            }
        }
        ChronicleKind::Weather => vec![Span::Text(event.name.clone())],
        ChronicleKind::LawProposed => match person(0) {
            Some(who) => vec![who, Span::Text(format!(" proposed {}.", event.name))],
            None => vec![Span::Text(format!("Someone proposed {}.", event.name))],
        },
        ChronicleKind::LawDecided
        | ChronicleKind::LawLapsed
        | ChronicleKind::Restitution
        | ChronicleKind::CaseHeard
        | ChronicleKind::CustomAmended
        | ChronicleKind::CustomTaken
        | ChronicleKind::RevoltFailed
        | ChronicleKind::CoupFailed
        | ChronicleKind::Encounter => {
            vec![Span::Text(event.name.clone())]
        }
        ChronicleKind::CaseBrought | ChronicleKind::RevoltCalled | ChronicleKind::CoupCalled => {
            match person(0) {
                Some(who) => vec![who, Span::Text(format!(" {}", event.name))],
                None => vec![Span::Text(format!("Someone {}", event.name))],
            }
        }
        ChronicleKind::Taking => match person(0) {
            Some(who) => vec![who, Span::Text(format!(" {}", event.name))],
            None => vec![Span::Text(format!("Someone {}", event.name))],
        },
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

/// "sickle workshop", or "workshop" when the good is not known.
fn workshop(good: &str) -> String {
    if good.is_empty() {
        "workshop".to_owned()
    } else {
        format!("{} workshop", good.to_lowercase())
    }
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
            Span::Person(_, n) | Span::Settlement(_, n) | Span::Firm(_, n) => n.as_str(),
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
            firm: None,
        };
        let spans = render(&e, &|_| String::new());
        assert!(spans.contains(&Span::Settlement(id, "Alder Ford".to_owned())));
        assert_eq!(plain(&spans), "They made camp at Alder Ford.");
    }

    #[test]
    fn a_household_that_left_names_where_it_left() {
        let (village, eldest) = (
            PermanentId::from_raw(7).expect("non-zero"),
            PermanentId::from_raw(9).expect("non-zero"),
        );
        let left = |name: &str| ChronicleEvent {
            seq: 3,
            at: SimTime::ZERO,
            kind: ChronicleKind::Left,
            people: vec![eldest, PermanentId::from_raw(10).expect("non-zero")],
            settlement: Some(village),
            place: None,
            number: 2.0,
            name: name.to_owned(),
            firm: None,
        };
        let name_of = |_| "Iver".to_owned();
        let spans = render(&left("Hazelstead"), &name_of);
        assert!(spans.contains(&Span::Settlement(village, "Hazelstead".to_owned())));
        assert_eq!(
            plain(&spans),
            "The household of Iver (2 people) gave up and left Hazelstead."
        );
        // An entry saved before the name was kept.
        assert_eq!(
            plain(&render(&left(""), &name_of)),
            "The household of Iver (2 people) gave up and left the valley."
        );
    }
}
