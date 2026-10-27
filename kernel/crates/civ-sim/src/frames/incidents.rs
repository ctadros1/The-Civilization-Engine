//! Incidents of every kind for the observer (wire 1.64, M6a slice BB; ADR-0015 §8): takings and
//! cases of illness in one shape, each with its three layers kept apart and in the kernel's words
//! (what happened, which nobody in the world reads; what people know, from their beliefs and word;
//! what the polity knows and did, from its cases and laws), a page for each with its stamped steps
//! and typed links, and a page for each outbreak with its sources, the truth beside what people
//! hold and what their polity did. Each kind keeps its own store; this only reads them, and no
//! choice reads what it builds. Wire 1.68 adds a building's fire, and a page for each
//! conflagration (M6b slice BC, step three; `fire`). Wire 1.91 adds each building that fell or
//! cracked when the ground shook, and a page for each shaking (M8d slice DA; `quake`).

mod fire;
mod flood;
mod quake;
mod storm;
mod waits;

pub use fire::{building_fire, building_words, conflagration_response};
pub use flood::flood_response;
pub use quake::quake_response;
pub use storm::storm_response;

use std::collections::{BTreeMap, BTreeSet};
use std::hash::{DefaultHasher, Hash, Hasher};

use civ_agents::OutbreakStep;
use civ_agents::contagion::{How, Node};
use civ_agents::crime::{CaseStage, Incident, Outcome as Taken, Source};
use civ_agents::history::{ChronicleEvent, ChronicleKind};
use civ_agents::polity::{Law, LawStatus, Outcome as Decided, PolicyKind, day_words};
use civ_agents::sickness::{Acquired, Episode, Outbreak, Outcome};
use civ_agents::uses::Place;
use civ_agents::word::ClaimKind;
use civ_core::{PermanentId, SimTime};
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use super::order::{
    case_words, household_words, owed_words, response_words, watch_words, what_words,
};
use super::response;
use super::sickness::{episode_words, source_words};
use crate::Sim;

/// A taking (ADR-0015 §1).
pub const TAKING: u8 = 1;
/// A case of illness: an infection (ADR-0021 §5).
pub const ILLNESS: u8 = 2;
/// An outbreak, as a chronicle entry's link names it (wire 1.65).
pub const OUTBREAK: u8 = 3;
/// A building's fire (wire 1.68, ADR-0022 §13).
pub const FIRE: u8 = 4;
/// A conflagration, as a chronicle entry's link names it (wire 1.68).
pub const CONFLAGRATION: u8 = 5;
/// High water at a river reach (wire 1.69, ADR-0022 §13).
pub const HIGH_WATER: u8 = 6;
/// A flood episode (wire 1.69, ADR-0022 §13).
pub const FLOOD: u8 = 7;
/// Storm damage to a building (wire 1.69, ADR-0022 §13).
pub const STORM_DAMAGE: u8 = 8;
/// A storm episode (wire 1.69, ADR-0022 §13).
pub const STORM: u8 = 9;
/// A wait worth telling at a crossing or a gate (wire 1.85, ADR-0029 §9).
pub const WAIT: u8 = 10;
/// A building that fell or cracked when the ground shook (wire 1.91, ADR-0031 §4). Its number is
/// a running count from 1 over every shaking's buildings that fell or cracked (`quake::shaken`).
pub const SHAKEN: u8 = 11;
/// A shaking, as a chronicle entry's link names it (wire 1.91, ADR-0031 §4).
pub const QUAKE: u8 = 12;

/// Rows a page carries when none is asked, and at most.
const DEFAULT_ROWS: usize = 50;
const MOST_ROWS: usize = 200;
/// Cases an outbreak's page lists when none is asked, and at most.
const DEFAULT_MEMBERS: usize = 100;
const MOST_MEMBERS: usize = 500;
/// Days an outbreak's curve runs at most.
const CURVE_DAYS: i64 = 400;
/// Days before an outbreak began that a load reaching a source still counts as fouling it.
const FOULED_BEFORE_DAYS: i64 = 60;

const MINUTES_PER_DAY: i64 = 1440;

/// Where a list of incidents stands: newest first, by when each began, then kind, then number.
pub type Key = (i64, u8, u32);

/// One incident, in words, before it is put on the wire.
struct Row {
    kind: u8,
    id: u32,
    minute: i64,
    settlement: Option<PermanentId>,
    at: (f32, f32),
    who: PermanentId,
    /// Whom the words begin with, when not a person's name: a fire's building.
    who_name: Option<String>,
    happened: String,
    known: String,
    polity: String,
    observer: u32,
    outbreak: u32,
    conflagration: u32,
    flood: u32,
    storm: u32,
    /// The shaking a building that fell or cracked is part of (wire 1.91).
    quake: u32,
    open: bool,
}

/// Day `day` (an index) in words: "3 May of year 2".
fn day_text(day: i64) -> String {
    day_words(SimTime::from_minutes(day * MINUTES_PER_DAY))
}

/// "1 account", "3 accounts".
fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// A list in words: "A", "A and B", "A, B and 2 others".
fn names_text(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [a] => a.clone(),
        [a, b] => format!("{a} and {b}"),
        [a, b, _] => format!("{a}, {b} and 1 other"),
        [a, b, rest @ ..] => format!("{a}, {b} and {} others", rest.len()),
    }
}

/// "1 person suspects", "3 people suspect".
fn suspect_words(n: usize) -> String {
    if n == 1 {
        "1 person suspects".to_owned()
    } else {
        format!("{n} people suspect")
    }
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

/// A source of water, kept apart for an outbreak's page: a well, the river (every stretch of its
/// edge), or a spring.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SourceKey {
    Well(PermanentId),
    River,
    Spring(u32),
}

/// What people know and what the polities did, gathered once for a query.
struct Facts<'a> {
    sim: &'a Sim,
    /// Sickness claims by the household they name, oldest first: (day, claim).
    sick_claims: BTreeMap<PermanentId, Vec<(i64, u32)>>,
    /// For each sickness claim: those who hold it, and those who hold it at first hand.
    heard: BTreeMap<u32, (u32, u32)>,
    /// Each law to close a source, with the settlement of the polity that made it and the source
    /// it names.
    closures: Vec<(PermanentId, &'a Law, Place)>,
    /// The observer's influence that began each outbreak, by the outbreak's number.
    begun_by: BTreeMap<u32, u32>,
    /// How long a household's sickness stays news (the content's spell, days).
    spell: i64,
}

impl<'a> Facts<'a> {
    fn of(sim: &'a Sim) -> Facts<'a> {
        let pop = &sim.people;
        let mut sick_claims: BTreeMap<PermanentId, Vec<(i64, u32)>> = BTreeMap::new();
        let mut sick_ids = BTreeSet::new();
        for c in pop
            .word
            .claims
            .iter()
            .filter(|c| c.kind == ClaimKind::Sickness)
        {
            if let Some(h) = c.subject {
                sick_claims.entry(h).or_default().push((c.day, c.id));
                sick_ids.insert(c.id);
            }
        }
        for list in sick_claims.values_mut() {
            list.sort_unstable();
        }
        let mut heard: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
        for h in pop
            .word
            .heard
            .iter()
            .filter(|h| sick_ids.contains(&h.claim))
        {
            let e = heard.entry(h.claim).or_default();
            e.0 += 1;
            e.1 += u32::from(h.from.is_none());
        }
        let mut closures = Vec::new();
        for polity in &pop.polities {
            for law in polity
                .laws
                .iter()
                .filter(|l| l.kind == PolicyKind::CloseSource)
            {
                if let Some(place) = polity.closure_of(law.id) {
                    closures.push((polity.settlement, law, place));
                }
            }
        }
        let sickness = &pop.sickness;
        let begun_by = sickness
            .outbreaks()
            .iter()
            .filter_map(|o| {
                sickness
                    .episodes()
                    .iter()
                    .find(|e| e.id == o.first)
                    .and_then(|e| match e.acquired {
                        Acquired::Observer { influence } => Some((o.id, influence)),
                        _ => None,
                    })
            })
            .collect();
        Facts {
            sim,
            sick_claims,
            heard,
            closures,
            begun_by,
            spell: i64::from(sim.rules.people.word.sickness_spell_days),
        }
    }

    /// The household an infection began in: as recorded, or (from an older save) where its bearer
    /// lives now.
    fn household_of(&self, e: &Episode) -> Option<PermanentId> {
        e.household
            .or_else(|| self.sim.people.person(e.person).map(|p| p.household))
    }

    /// The claim that someone of the case's household lay sick, made while they were ill.
    fn claim_of(&self, e: &Episode) -> Option<(i64, u32)> {
        if !e.symptomatic() {
            return None;
        }
        let h = self.household_of(e)?;
        self.sick_claims
            .get(&h)?
            .iter()
            .rev()
            .find(|&&(day, _)| day <= e.course.ill_until && day >= e.course.ill_from - self.spell)
            .copied()
    }

    /// The source an infection's water came from, as a source's key.
    fn source_of(&self, e: &Episode) -> Option<SourceKey> {
        match e.acquired {
            Acquired::Water { source } => node_key(source),
            _ => None,
        }
    }

    /// The observer's influence that started an infection, if any: its own, or its outbreak's.
    fn observer_of(&self, e: &Episode) -> u32 {
        match e.acquired {
            Acquired::Observer { influence } => influence,
            _ => e
                .outbreak
                .and_then(|o| self.begun_by.get(&o).copied())
                .unwrap_or(0),
        }
    }

    /// The laws to close a source, made by `settlement`'s polity (any polity when `None`).
    fn closures_of(&self, key: SourceKey, settlement: Option<PermanentId>) -> Vec<&'a Law> {
        self.closures
            .iter()
            .filter(|&&(s, _, place)| {
                settlement.is_none_or(|want| s == want) && key_of(self.sim, place) == Some(key)
            })
            .map(|&(_, law, _)| law)
            .collect()
    }
}

/// The key of a node water is drawn from: a well, or a stretch of the river.
fn node_key(node: Node) -> Option<SourceKey> {
    match node {
        Node::Well(w) => Some(SourceKey::Well(w)),
        Node::Reach(_) => Some(SourceKey::River),
        _ => None,
    }
}

/// The key of a place people draw water at.
fn key_of(sim: &Sim, place: Place) -> Option<SourceKey> {
    let Place::Source(cell) = place else {
        return None;
    };
    if let Some(w) = sim.land.wells.list.iter().find(|w| w.cell == cell) {
        return Some(SourceKey::Well(w.id));
    }
    let patch = sim.land.patches.of_cell(cell as usize, sim.map.width);
    let spring = sim
        .land
        .water
        .aquifer
        .seep
        .get(patch)
        .copied()
        .flatten()
        .is_some_and(|(c, _)| c == cell);
    Some(if spring {
        SourceKey::Spring(cell)
    } else {
        SourceKey::River
    })
}

/// A source's name: "Bo's household's well", "the river", "the spring near Ada's household's
/// home".
fn key_words(sim: &Sim, key: SourceKey) -> String {
    match key {
        SourceKey::Well(w) => source_words(sim, Node::Well(w)),
        SourceKey::River => "the river".to_owned(),
        SourceKey::Spring(cell) => super::sickness::place_words(sim, Place::Source(cell)),
    }
}

/// How a law to close a source was decided, in words.
fn closure_words(sim: &Sim, law: &Law) -> String {
    let pop = &sim.people;
    let (present, support, oppose) = law.counts();
    let proposed = format!(
        "{} proposed closing it on {}",
        pop.name_of(law.sponsor),
        day_words(law.proposed)
    );
    let on = law
        .decided
        .map_or_else(String::new, |t| format!(" on {}", day_words(t)));
    let tally = format!("{support} for, {oppose} against");
    match law.status {
        LawStatus::Proposed => format!(
            "{proposed}; the gathering meets on {}",
            day_text(law.meets_day)
        ),
        LawStatus::InForce => format!("{proposed}; the gathering closed it{on}: {tally}"),
        LawStatus::Rejected => match law.outcome {
            Some(Decided::NoQuorum) => format!(
                "{proposed}; too few came to decide it{on}: {present} of {}",
                law.eligible
            ),
            _ => format!("{proposed}; the gathering would not close it{on}: {tally}"),
        },
        LawStatus::Lapsed | LawStatus::Superseded | LawStatus::Carried | LawStatus::StruckDown => {
            format!(
                "{proposed}; the gathering closed it{on} ({tally}), and the closing has since ended"
            )
        }
    }
}

/// What a polity did about a case's source, in words, and whether any law names it.
fn polity_words(f: &Facts, e: &Episode, settlement: Option<PermanentId>) -> (String, bool) {
    let Some(key) = f.source_of(e) else {
        return ("No law names it".to_owned(), false);
    };
    match f.closures_of(key, settlement).last() {
        None => (format!("No law names {}", key_words(f.sim, key)), false),
        Some(law) => (
            format!(
                "Of {}: {}",
                key_words(f.sim, key),
                closure_words(f.sim, law)
            ),
            true,
        ),
    }
}

/// What people know of a case, in words, and whether anyone does.
fn illness_known(f: &Facts, e: &Episode) -> (String, bool) {
    let today = f.sim.now().day_index();
    if !e.symptomatic() {
        return ("Nobody knows of it: they were never ill".to_owned(), false);
    }
    if e.course.ill_from > today {
        return ("Nobody knows of it: they are not ill yet".to_owned(), false);
    }
    let Some((_, claim)) = f.claim_of(e) else {
        return ("Nobody knows of it".to_owned(), false);
    };
    let (holders, first) = f.heard.get(&claim).copied().unwrap_or_default();
    let household = f.household_of(e).map_or_else(
        || "their household".to_owned(),
        |h| super::people::household_name(f.sim, h),
    );
    (
        format!(
            "Known to {} as sickness in {household}, from {}",
            plural(holders as usize, "person", "people"),
            plural(first as usize, "first-hand account", "first-hand accounts")
        ),
        holders > 0,
    )
}

/// What happened in a case, after its bearer's name.
fn illness_happened(sim: &Sim, e: &Episode) -> String {
    let today = sim.now().day_index();
    let name = sim
        .rules
        .catalog
        .diseases
        .get(usize::from(e.disease))
        .map_or_else(|| "a disease".to_owned(), |d| d.name.to_lowercase());
    let how = match e.acquired {
        Acquired::Observer { .. } => {
            ", brought by the observer as if they took it elsewhere".to_owned()
        }
        Acquired::Household { .. } => " from those who shed it at home".to_owned(),
        Acquired::Water { source } => {
            format!(" in water drawn at {}", source_words(sim, source))
        }
        Acquired::Inoculated { by } => {
            format!(", inoculated by {}", sim.people().name_of(by))
        }
        Acquired::Sickhouse { person } => {
            format!(" at the sickhouse, from {}", sim.people().name_of(person))
        }
        Acquired::Company { person } => {
            format!(
                " in company at the hearth with {}",
                sim.people().name_of(person)
            )
        }
    };
    let c = &e.course;
    let course = if !e.symptomatic() {
        "without falling ill".to_owned()
    } else if c.ill_from > today {
        "not ill yet".to_owned()
    } else {
        let severe = if c.severe { ", severely" } else { "" };
        format!("ill {} days{severe}", c.ill_until - c.ill_from)
    };
    let end = match e.ended {
        Some((day, Outcome::Died)) => format!("; died of it on {}", day_text(day)),
        Some((_, Outcome::Recovered)) => "; over it".to_owned(),
        Some((day, Outcome::Gone)) => {
            format!("; died or left while it ran, on {}", day_text(day))
        }
        None => "; it runs".to_owned(),
    };
    format!(
        "took {name} on {}{how}; {course}{end}",
        day_text(e.infected)
    )
}

/// Where someone lived, as metres: their household's home, else their settlement's hearth.
fn home_at(
    sim: &Sim,
    household: Option<PermanentId>,
    settlement: Option<PermanentId>,
) -> (f32, f32) {
    if let Some(x) = household.and_then(|h| sim.people.household(h)) {
        return x.home;
    }
    settlement
        .and_then(|s| sim.land.settlements.iter().find(|x| x.id == s))
        .map_or((0.0, 0.0), |s| s.hearth_m)
}

/// The settlement a case belongs to: its outbreak's, its household's, or where its bearer lived.
fn illness_settlement(f: &Facts, e: &Episode) -> Option<PermanentId> {
    let pop = &f.sim.people;
    e.outbreak
        .and_then(|o| pop.sickness.outbreaks().iter().find(|x| x.id == o))
        .and_then(|o| o.settlement)
        .or_else(|| {
            f.household_of(e)
                .and_then(|h| pop.household(h))
                .and_then(|x| x.settlement)
        })
        .or_else(|| {
            pop.records
                .get(&e.person)
                .and_then(|r| r.residence_at(SimTime::from_minutes(e.infected * MINUTES_PER_DAY)))
        })
}

/// A case's row, and whether anyone knows of it and whether a law names its source.
fn illness_row(f: &Facts, e: &Episode) -> (Row, bool, bool) {
    let settlement = illness_settlement(f, e);
    let (known, knows) = illness_known(f, e);
    let (polity, before) = polity_words(f, e, settlement);
    (
        Row {
            kind: ILLNESS,
            id: e.id,
            minute: e.infected * MINUTES_PER_DAY,
            settlement,
            at: home_at(f.sim, f.household_of(e), settlement),
            who: e.person,
            who_name: None,
            happened: illness_happened(f.sim, e),
            known,
            polity,
            observer: f.observer_of(e),
            outbreak: e.outbreak.unwrap_or(0),
            conflagration: 0,
            flood: 0,
            storm: 0,
            quake: 0,
            open: e.ended.is_none(),
        },
        knows,
        before,
    )
}

/// What happened in a taking, after the taker's name.
fn taking_happened(sim: &Sim, i: &Incident) -> String {
    let pop = &sim.people;
    let target = household_words(sim, i.target);
    let when = day_words(i.at);
    let mut s = match i.outcome {
        Taken::Taken => format!("took {} from {target} on {when}", what_words(sim, i)),
        Taken::TurnedBack => {
            format!("went to take from {target} on {when} and turned back: someone was home")
        }
        Taken::Disturbed => format!(
            "went to take from {target} on {when} and fled with nothing when a sleeper woke"
        ),
    };
    // One from another settlement is told as of it (M6c slice BI).
    if let Some(from) = i.across {
        let name = sim
            .land
            .settlements
            .iter()
            .find(|x| x.id == from)
            .map_or("another settlement", |x| x.name.as_str());
        s = format!("of {name} {s}");
    }
    let seen: Vec<String> = i.seen_by.iter().map(|&w| pop.name_of(w)).collect();
    if seen.is_empty() {
        s.push_str("; nobody saw it");
    } else {
        s.push_str(&format!("; seen by {}", names_text(&seen)));
    }
    let watch = watch_words(sim, i);
    if !watch.is_empty() {
        s.push_str(&format!("; {watch}"));
    }
    s
}

/// Whether anyone in the world knows of a taking (ADR-0015 §7: what the taker knows of their own
/// doing is no account of it), as believers of the taker and those who know only of a loss.
fn taking_believers(sim: &Sim, i: &Incident) -> (usize, usize) {
    let (mut taker, mut loss) = (0usize, 0usize);
    for b in sim
        .people
        .order
        .beliefs
        .iter()
        .filter(|b| b.incident == i.id)
    {
        if sim.people.person(b.holder).is_none() {
            continue;
        }
        if b.taker.is_some() && b.origin == Some(i.actor) {
            continue;
        }
        if b.taker.is_some() {
            taker += 1;
        } else if b.source == Source::Noticed {
            loss += 1;
        }
    }
    (taker, loss)
}

/// What people believe of a taking and what the household taken from chose, in words.
fn taking_known(sim: &Sim, i: &Incident) -> String {
    let (taker, loss) = taking_believers(sim, i);
    let mut parts = Vec::new();
    if taker > 0 {
        parts.push(format!(
            "{taker} {} who took, from {}",
            if taker == 1 {
                "believes they know"
            } else {
                "believe they know"
            },
            plural(sim.people.order.sources(i.id), "account", "accounts")
        ));
    }
    if loss > 0 {
        parts.push(format!(
            "{loss} {} only of a loss",
            if loss == 1 { "knows" } else { "know" }
        ));
    }
    let mut s = if parts.is_empty() {
        "Nobody knows of it".to_owned()
    } else {
        parts.join("; ")
    };
    let answered = response_words(sim, i.id);
    if !answered.is_empty() {
        s.push_str(&format!(". {answered}"));
        let owed = owed_words(sim, i.id);
        if !owed.is_empty() {
            s.push_str(&format!(": {owed}"));
        }
    }
    s
}

/// The settlement a taking belongs to: the household taken from's.
fn taking_settlement(sim: &Sim, i: &Incident) -> Option<PermanentId> {
    sim.people.household(i.target).and_then(|x| x.settlement)
}

fn taking_row(sim: &Sim, i: &Incident) -> Row {
    let case = case_words(sim, i.id);
    Row {
        kind: TAKING,
        id: i.id,
        minute: i.at.minutes(),
        settlement: taking_settlement(sim, i),
        at: i.place,
        who: i.actor,
        who_name: None,
        happened: taking_happened(sim, i),
        known: taking_known(sim, i),
        polity: if case.is_empty() {
            "No gathering was told of it".to_owned()
        } else {
            capitalised(&case)
        },
        observer: 0,
        outbreak: 0,
        conflagration: 0,
        flood: 0,
        storm: 0,
        quake: 0,
        open: false,
    }
}

fn put_row<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    r: &Row,
) -> WIPOffset<wire::IncidentRow<'a>> {
    let who_name = match &r.who_name {
        Some(name) => fbb.create_string(name),
        None => fbb.create_string(&sim.people.name_of(r.who)),
    };
    let happened = fbb.create_string(&r.happened);
    let known = fbb.create_string(&r.known);
    let polity = fbb.create_string(&r.polity);
    wire::IncidentRow::create(
        fbb,
        &wire::IncidentRowArgs {
            kind: r.kind,
            id: r.id,
            minute: r.minute,
            settlement: r.settlement.map_or(0, PermanentId::get),
            at: Some(&wire::Vec2::new(r.at.0, r.at.1)),
            who: r.who.get(),
            who_name: Some(who_name),
            happened: Some(happened),
            known: Some(known),
            polity: Some(polity),
            observer: r.observer,
            outbreak: r.outbreak,
            open: r.open,
            conflagration: r.conflagration,
            flood: r.flood,
            storm: r.storm,
            quake: r.quake,
        },
    )
}

fn put_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    lines: &[String],
) -> WIPOffset<Vector<'a, ForwardsUOffset<&'a str>>> {
    let list: Vec<_> = lines.iter().map(|l| fbb.create_string(l)).collect();
    fbb.create_vector(&list)
}

/// Changes whenever an incident or an outbreak opens, moves on or ends, or what people know of
/// one or what a polity did about one changes (0 = no incident yet).
pub fn incidents_rev(sim: &Sim) -> u64 {
    let pop = &sim.people;
    let order = &pop.order;
    let sickness = &pop.sickness;
    let fires = &pop.fires;
    let floods = &pop.floods;
    if order.incidents.is_empty()
        && sickness.episodes().is_empty()
        && fires.list.is_empty()
        && floods.high_waters.is_empty()
        && floods.storm_damages.is_empty()
        && pop.traffic.long_waits.is_empty()
        && pop.quakes.list.is_empty()
    {
        return 0;
    }
    let mut h = DefaultHasher::new();
    (
        order.incidents.len(),
        order.beliefs.len(),
        order.responses.len(),
        order.cases.len(),
        order.sightings.len(),
    )
        .hash(&mut h);
    for c in &order.cases {
        (c.stage.code(), c.heard.map(SimTime::minutes)).hash(&mut h);
    }
    for o in &order.obligations {
        (o.standing.code(), o.paid_kcal.to_bits()).hash(&mut h);
    }
    (
        sickness.episodes().len(),
        sickness.running().len(),
        sickness.outbreaks().len(),
        sickness
            .outbreaks()
            .iter()
            .filter(|o| o.ended.is_some())
            .count(),
    )
        .hash(&mut h);
    (
        pop.word.claims.len(),
        pop.word.heard.len(),
        pop.word.suspicions.len(),
    )
        .hash(&mut h);
    for p in &pop.polities {
        for l in p.laws.iter().filter(|l| l.kind == PolicyKind::CloseSource) {
            (l.id, l.status as u8).hash(&mut h);
        }
    }
    // A running case's words change with each day lived.
    if !sickness.running().is_empty() {
        sim.now().day_index().hash(&mut h);
    }
    // A fire's words change with each minute it burns.
    (
        fires.list.len(),
        fires.episodes.len(),
        fires.list.iter().filter(|f| f.ended.is_some()).count(),
        fires.weighed.len(),
    )
        .hash(&mut h);
    if fires.any_burning() {
        sim.now().minutes().hash(&mut h);
    }
    (
        floods.high_waters.len(),
        floods.flood_episodes.len(),
        floods
            .high_waters
            .iter()
            .filter(|hw| hw.ended.is_some())
            .count(),
        floods.storm_damages.len(),
        floods.storm_episodes.len(),
    )
        .hash(&mut h);
    pop.traffic.long_waits.last().map(|w| w.id).hash(&mut h);
    // A shaking never changes once recorded.
    pop.quakes.list.len().hash(&mut h);
    h.finish().max(1)
}

/// A `Response` with a page of incidents of the kind asked (every kind for 0), newest first, from
/// before `before`, with the totals by kind and the outbreaks, in `settlement` (every one when
/// `None`).
pub fn incidents_response(
    sim: &Sim,
    settlement: Option<PermanentId>,
    kind: u8,
    before: Option<Key>,
    limit: usize,
) -> Vec<u8> {
    let pop = &sim.people;
    let f = Facts::of(sim);
    let limit = if limit == 0 {
        DEFAULT_ROWS
    } else {
        limit.min(MOST_ROWS)
    };
    let in_scope = |s: Option<PermanentId>| settlement.is_none_or(|want| s == Some(want));
    // Every incident's key in scope; words only for the page.
    let mut keys: Vec<(Key, usize)> = Vec::new();
    let mut totals: BTreeMap<u8, [u32; 4]> = BTreeMap::new();
    totals.insert(TAKING, [0; 4]);
    totals.insert(ILLNESS, [0; 4]);
    totals.insert(FIRE, [0; 4]);
    totals.insert(HIGH_WATER, [0; 4]);
    totals.insert(STORM_DAMAGE, [0; 4]);
    totals.insert(WAIT, [0; 4]);
    totals.insert(SHAKEN, [0; 4]);
    for (at, i) in pop.order.incidents.iter().enumerate() {
        if !in_scope(taking_settlement(sim, i)) {
            continue;
        }
        let (taker, loss) = taking_believers(sim, i);
        let t = totals.entry(TAKING).or_default();
        t[0] += 1;
        t[1] += u32::from(taker + loss > 0);
        t[2] += u32::from(pop.order.cases.iter().any(|c| c.incident == i.id));
        if kind == 0 || kind == TAKING {
            keys.push(((i.at.minutes(), TAKING, i.id), at));
        }
    }
    for (at, e) in pop.sickness.episodes().iter().enumerate() {
        let s = illness_settlement(&f, e);
        if !in_scope(s) {
            continue;
        }
        let t = totals.entry(ILLNESS).or_default();
        t[0] += 1;
        t[1] += u32::from(illness_known(&f, e).1);
        t[2] += u32::from(polity_words(&f, e, s).1);
        t[3] += u32::from(f.observer_of(e) > 0);
        if kind == 0 || kind == ILLNESS {
            keys.push(((e.infected * MINUTES_PER_DAY, ILLNESS, e.id), at));
        }
    }
    for (at, x) in pop.fires.list.iter().enumerate() {
        if !in_scope(x.settlement) {
            continue;
        }
        let t = totals.entry(FIRE).or_default();
        t[0] += 1;
        t[3] += u32::from(fire::observer_of(sim, x) > 0);
        if kind == 0 || kind == FIRE {
            keys.push(((x.began.minutes(), FIRE, x.id), at));
        }
    }
    for (at, x) in pop.floods.high_waters.iter().enumerate() {
        if !in_scope(x.settlement) {
            continue;
        }
        let t = totals.entry(HIGH_WATER).or_default();
        t[0] += 1;
        t[1] += u32::from(x.buildings_flooded > 0);
        let obs = x
            .episode
            .and_then(|ep| pop.floods.flood_episodes.iter().find(|e| e.id == ep))
            .map_or(0, |e| flood::observer_of(sim, e));
        t[3] += u32::from(obs > 0);
        if kind == 0 || kind == HIGH_WATER {
            keys.push(((x.began.minutes(), HIGH_WATER, x.id), at));
        }
    }
    for (at, x) in pop.floods.storm_damages.iter().enumerate() {
        if !in_scope(x.settlement) {
            continue;
        }
        let t = totals.entry(STORM_DAMAGE).or_default();
        t[0] += 1;
        t[1] += 1;
        let obs = x
            .episode
            .and_then(|ep| pop.floods.storm_episodes.iter().find(|e| e.id == ep))
            .map_or(0, |e| storm::observer_of(sim, e));
        t[3] += u32::from(obs > 0);
        if kind == 0 || kind == STORM_DAMAGE {
            keys.push(((x.time.minutes(), STORM_DAMAGE, x.id), at));
        }
    }
    for (at, w) in pop.traffic.long_waits.iter().enumerate() {
        if !in_scope(w.settlement) {
            continue;
        }
        let t = totals.entry(WAIT).or_default();
        t[0] += 1;
        t[1] += 1;
        if kind == 0 || kind == WAIT {
            keys.push(((w.time.minutes(), WAIT, w.id), at));
        }
    }
    // Buildings shaken: nobody in the world tells of them as such yet, so only the truth and
    // the observer's share are counted.
    let shakens: Vec<_> = quake::shaken(sim).collect();
    for (at, &(id, q, x)) in shakens.iter().enumerate() {
        if !in_scope(x.settlement) {
            continue;
        }
        let t = totals.entry(SHAKEN).or_default();
        t[0] += 1;
        t[3] += u32::from(q.influence > 0);
        if kind == 0 || kind == SHAKEN {
            keys.push(((q.at.minutes(), SHAKEN, id), at));
        }
    }
    keys.sort_unstable_by_key(|k| std::cmp::Reverse(k.0));
    let start = before.map_or(0, |k| keys.partition_point(|&(key, _)| key >= k));
    let end = (start + limit).min(keys.len());
    let page = &keys[start.min(end)..end];
    let next = if end < keys.len() {
        page.last().map(|p| p.0)
    } else {
        None
    };
    let mut fbb = FlatBufferBuilder::new();
    let rows: Vec<_> = page
        .iter()
        .map(|&((_, k, _), at)| {
            let row = match k {
                TAKING => taking_row(sim, &pop.order.incidents[at]),
                FIRE => fire::fire_row(sim, &pop.fires.list[at]),
                HIGH_WATER => flood::high_water_row(sim, &pop.floods.high_waters[at]),
                STORM_DAMAGE => storm::storm_damage_row(sim, &pop.floods.storm_damages[at]),
                WAIT => waits::wait_row(sim, &pop.traffic.long_waits[at]),
                SHAKEN => {
                    let (id, q, x) = shakens[at];
                    quake::shaken_row(sim, id, q, x)
                }
                _ => illness_row(&f, &pop.sickness.episodes()[at]).0,
            };
            put_row(&mut fbb, sim, &row)
        })
        .collect();
    let rows = fbb.create_vector(&rows);
    let totals: Vec<_> = totals
        .iter()
        .map(|(&kind, t)| {
            wire::IncidentTotal::create(
                &mut fbb,
                &wire::IncidentTotalArgs {
                    kind,
                    all: t[0],
                    known: t[1],
                    polity: t[2],
                    observer: t[3],
                },
            )
        })
        .collect();
    let totals = fbb.create_vector(&totals);
    let outbreaks: Vec<_> = pop
        .sickness
        .outbreaks()
        .iter()
        .rev()
        .filter(|o| in_scope(o.settlement))
        .map(|o| outbreak_row(&mut fbb, &f, o))
        .collect();
    let outbreaks = fbb.create_vector(&outbreaks);
    let conflagrations: Vec<_> = pop
        .fires
        .episodes
        .iter()
        .rev()
        .filter(|e| in_scope(e.settlement))
        .map(|e| fire::conflagration_row(&mut fbb, sim, e))
        .collect();
    let conflagrations = fbb.create_vector(&conflagrations);
    let floods: Vec<_> = pop
        .floods
        .flood_episodes
        .iter()
        .rev()
        .filter(|e| in_scope(e.settlement))
        .map(|e| flood::flood_row(&mut fbb, sim, e))
        .collect();
    let floods = fbb.create_vector(&floods);
    let storms: Vec<_> = pop
        .floods
        .storm_episodes
        .iter()
        .rev()
        .filter(|e| in_scope(e.settlement))
        .map(|e| storm::storm_row(&mut fbb, sim, e))
        .collect();
    let storms = fbb.create_vector(&storms);
    let quakes: Vec<_> = pop
        .quakes
        .list
        .iter()
        .rev()
        .filter(|q| in_scope(q.settlement) || q.shaken.iter().any(|x| in_scope(x.settlement)))
        .map(|q| quake::quake_row(&mut fbb, sim, q))
        .collect();
    let quakes = fbb.create_vector(&quakes);
    let next = next.map(|(minute, kind, id)| wire::IncidentKey::new(minute, kind, id));
    let body = wire::Incidents::create(
        &mut fbb,
        &wire::IncidentsArgs {
            minute: sim.now().minutes(),
            rows: Some(rows),
            next: next.as_ref(),
            totals: Some(totals),
            outbreaks: Some(outbreaks),
            conflagrations: Some(conflagrations),
            floods: Some(floods),
            storms: Some(storms),
            quakes: Some(quakes),
        },
    );
    response(fbb, wire::ResponseBody::Incidents, body)
}

/// An outbreak's counts: those who took it, fell ill and died of it.
fn outbreak_counts(sim: &Sim, o: &Outbreak) -> (u32, u32, u32) {
    let today = sim.now().day_index();
    let (mut took, mut ill, mut died) = (0, 0, 0);
    for e in sim.people.sickness.cases(o.id) {
        took += 1;
        ill += u32::from(e.symptomatic() && e.course.ill_from <= today);
        died += u32::from(matches!(e.ended, Some((_, Outcome::Died))));
    }
    (took, ill, died)
}

/// A disease's name: "Cholera".
fn disease_name(sim: &Sim, disease: u16) -> String {
    sim.rules
        .catalog
        .diseases
        .get(usize::from(disease))
        .map_or_else(|| "A disease".to_owned(), |d| d.name.clone())
}

/// "at Hazelford", or "among people of no settlement".
fn settlement_place(sim: &Sim, settlement: Option<PermanentId>) -> String {
    settlement
        .and_then(|s| sim.land.settlements.iter().find(|x| x.id == s))
        .map_or_else(
            || "among people of no settlement".to_owned(),
            |s| format!("at {}", s.name),
        )
}

fn outbreak_row<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    f: &Facts,
    o: &Outbreak,
) -> WIPOffset<wire::OutbreakRow<'a>> {
    let sim = f.sim;
    let disease = disease_name(sim, o.disease);
    let (took, ill, died) = outbreak_counts(sim, o);
    let span = match o.ended {
        Some(end) => format!("from {} to {}", day_text(o.began), day_text(end)),
        None => format!("from {}", day_text(o.began)),
    };
    let tail = if o.ended.is_some() { "over" } else { "it runs" };
    let words = format!(
        "{disease} {}, {span}: {took} took it, {ill} fell ill, {died} died; {tail}",
        settlement_place(sim, o.settlement)
    );
    let disease = fbb.create_string(&disease);
    let words = fbb.create_string(&words);
    wire::OutbreakRow::create(
        fbb,
        &wire::OutbreakRowArgs {
            id: o.id,
            disease: Some(disease),
            settlement: o.settlement.map_or(0, PermanentId::get),
            began: o.began * MINUTES_PER_DAY,
            ended: o.ended.map_or(-1, |d| d * MINUTES_PER_DAY),
            took,
            ill,
            died,
            observer: f.begun_by.get(&o.id).copied().unwrap_or(0),
            words: Some(words),
        },
    )
}

/// One step of an incident's chain, before it is put on the wire.
struct Step {
    label: &'static str,
    /// When, if reached: a minute, and whether only its day is known.
    at: Option<(i64, bool)>,
    words: String,
}

impl Step {
    fn at(label: &'static str, minute: i64, words: String) -> Step {
        Step {
            label,
            at: Some((minute, false)),
            words,
        }
    }

    fn day(label: &'static str, day: i64, words: String) -> Step {
        Step {
            label,
            at: Some((day * MINUTES_PER_DAY, true)),
            words,
        }
    }

    fn not_reached(label: &'static str, words: &str) -> Step {
        Step {
            label,
            at: None,
            words: words.to_owned(),
        }
    }
}

/// A link, before it is put on the wire: its relation, what it points at, and its words.
pub(crate) struct Link(
    pub(crate) u8,
    pub(crate) u8,
    pub(crate) u64,
    pub(crate) String,
);

pub(crate) const CAUSED_BY: u8 = 0;
pub(crate) const PART_OF: u8 = 1;
pub(crate) const TO_INFLUENCE: u8 = 0;
pub(crate) const TO_PERSON: u8 = 1;
pub(crate) const TO_HOUSEHOLD: u8 = 2;
pub(crate) const TO_SOURCE: u8 = 3;
pub(crate) const TO_OUTBREAK: u8 = 4;
pub(crate) const TO_FIRE: u8 = 7;
pub(crate) const TO_CONFLAGRATION: u8 = 8;
pub(crate) const TO_FLOOD: u8 = 10;
pub(crate) const TO_STORM: u8 = 11;
pub(crate) const TO_QUAKE: u8 = 12;

/// How a case was caught, as a link to what it was caught from.
fn caught_link(sim: &Sim, e: &Episode) -> Link {
    match e.acquired {
        Acquired::Observer { influence } => Link(
            CAUSED_BY,
            TO_INFLUENCE,
            u64::from(influence),
            "brought by the observer, as if they took it elsewhere (one recorded influence)"
                .to_owned(),
        ),
        Acquired::Household { household } => Link(
            CAUSED_BY,
            TO_HOUSEHOLD,
            household.get(),
            format!(
                "caught at home, from those of {} who shed it",
                super::people::household_name(sim, household)
            ),
        ),
        Acquired::Inoculated { by } => Link(
            CAUSED_BY,
            TO_PERSON,
            by.get(),
            format!("inoculated by {}", sim.people().name_of(by)),
        ),
        Acquired::Sickhouse { person } => Link(
            CAUSED_BY,
            TO_PERSON,
            person.get(),
            format!(
                "caught at the sickhouse from {}, who shed it",
                sim.people().name_of(person)
            ),
        ),
        Acquired::Company { person } => Link(
            CAUSED_BY,
            TO_PERSON,
            person.get(),
            format!(
                "caught in company at the hearth with {}, who shed it",
                sim.people().name_of(person)
            ),
        ),
        Acquired::Water { source } => Link(
            CAUSED_BY,
            TO_SOURCE,
            match source {
                Node::Well(w) => w.get(),
                _ => 0,
            },
            format!("caught in water drawn at {}", source_words(sim, source)),
        ),
    }
}

/// An incident's page before it is put on the wire: its row, steps, links, and its three layers
/// in lines.
type Page = (Row, Vec<Step>, Vec<Link>, [Vec<String>; 3]);

fn illness_page(f: &Facts, e: &Episode) -> Page {
    let sim = f.sim;
    let pop = &sim.people;
    let today = sim.now().day_index();
    let row = illness_row(f, e).0;
    let c = &e.course;
    let caught = caught_link(sim, e);
    let mut steps = vec![Step::day("Took it", e.infected, caught.3.clone())];
    steps.push(if c.shed_until > c.shed_from && c.shed_from <= today {
        let until = if c.shed_until > today {
            "and sheds it still".to_owned()
        } else {
            format!("until {}", day_text(c.shed_until))
        };
        Step::day("Shed it", c.shed_from, until)
    } else {
        Step::not_reached("Shed it", "not yet")
    });
    steps.push(if !e.symptomatic() {
        Step::not_reached("Fell ill", "never: they took it without falling ill")
    } else if c.ill_from > today {
        Step::not_reached("Fell ill", "not yet")
    } else {
        let severe = if c.severe { ", severely" } else { "" };
        Step::day(
            "Fell ill",
            c.ill_from,
            format!("for {} days{severe}", c.ill_until - c.ill_from),
        )
    });
    steps.push(match f.claim_of(e) {
        Some((day, _)) => Step::day("Seen", day, "abed, by their household".to_owned()),
        None => Step::not_reached("Seen", "nobody saw them abed"),
    });
    steps.push(match e.care.first {
        Some((day, by)) => Step::day(
            "Tended",
            day,
            format!(
                "first by {}; {:.0} hours in all",
                pop.name_of(by),
                e.care.total_h
            ),
        ),
        None => Step::not_reached("Tended", "nobody tended them"),
    });
    steps.push(match e.ended {
        Some((day, Outcome::Died)) => Step::day("Ended", day, "they died of it".to_owned()),
        Some((day, Outcome::Recovered)) => Step::day("Ended", day, "they were over it".to_owned()),
        Some((day, Outcome::Gone)) => {
            Step::day("Ended", day, "they died or left while it ran".to_owned())
        }
        None => Step::not_reached("Ended", "it runs"),
    });
    let mut links = vec![caught];
    if let Some(o) = e
        .outbreak
        .and_then(|o| pop.sickness.outbreaks().iter().find(|x| x.id == o))
    {
        let name = sim
            .rules
            .catalog
            .diseases
            .get(usize::from(o.disease))
            .map_or_else(|| "a disease".to_owned(), |d| d.name.to_lowercase());
        let first = if o.first == e.id {
            ", its first case"
        } else {
            ""
        };
        links.push(Link(
            PART_OF,
            TO_OUTBREAK,
            u64::from(o.id),
            format!(
                "part of the outbreak of {name} {}{first}",
                settlement_place(sim, o.settlement)
            ),
        ));
    }
    let truth = vec![
        format!("{} {}", pop.name_of(e.person), illness_happened(sim, e)),
        format!("Now: {}", episode_words(sim, e)),
    ];
    let mut knowledge = vec![row.known.clone()];
    if let Some(key) = f.source_of(e) {
        let holders = suspecting(sim, key);
        knowledge.push(if holders.is_empty() {
            format!("Nobody suspects {}", key_words(sim, key))
        } else {
            format!("{} {}", suspect_words(holders.len()), key_words(sim, key))
        });
    }
    let polity = vec![row.polity.clone()];
    (row, steps, links, [truth, knowledge, polity])
}

/// The living who suspect a source, with the day each came to, oldest first.
fn suspecting(sim: &Sim, key: SourceKey) -> Vec<(PermanentId, i64)> {
    let mut out: Vec<(PermanentId, i64)> = sim
        .people
        .word
        .suspicions
        .iter()
        .filter(|s| sim.people.person(s.holder).is_some() && key_of(sim, s.source) == Some(key))
        .map(|s| (s.holder, s.since))
        .collect();
    out.sort_unstable_by_key(|&(p, d)| (p, d));
    out.dedup_by_key(|x| x.0);
    out.sort_unstable_by_key(|&(p, d)| (d, p));
    out
}

fn taking_page(sim: &Sim, i: &Incident) -> Page {
    let pop = &sim.people;
    let order = &pop.order;
    let row = taking_row(sim, i);
    let mut steps = vec![Step::at(
        "Went to take",
        i.at.minutes(),
        match i.outcome {
            Taken::Taken => format!("took {}", what_words(sim, i)),
            other => other.words().to_owned(),
        },
    )];
    let seen: Vec<String> = i.seen_by.iter().map(|&w| pop.name_of(w)).collect();
    steps.push(if seen.is_empty() {
        Step::not_reached("Seen", "nobody saw it")
    } else {
        Step::at("Seen", i.at.minutes(), format!("by {}", names_text(&seen)))
    });
    let first = |source: Source| {
        order
            .beliefs
            .iter()
            .filter(|b| b.incident == i.id && b.source == source)
            .map(|b| b.day)
            .min()
    };
    steps.push(match first(Source::Noticed) {
        Some(day) => Step::day(
            "Loss noticed",
            day,
            "by the household taken from".to_owned(),
        ),
        None => Step::not_reached("Loss noticed", "nobody noticed a loss"),
    });
    steps.push(match first(Source::Told) {
        Some(day) => Step::day("Told", day, "word of it first passed on".to_owned()),
        None => Step::not_reached("Told", "nobody was told of it"),
    });
    steps.push(
        match order
            .responses
            .iter()
            .filter(|r| r.incident == i.id)
            .map(|r| r.day)
            .min()
        {
            Some(day) => Step::day("Answered", day, response_words(sim, i.id)),
            None => Step::not_reached("Answered", "the household taken from did nothing"),
        },
    );
    let case = order.cases.iter().rev().find(|c| c.incident == i.id);
    steps.push(match case {
        Some(c) => Step::day(
            "Before the gathering",
            c.opened,
            format!("brought by {}", pop.name_of(c.by)),
        ),
        None => Step::not_reached("Before the gathering", "nobody brought it"),
    });
    steps.push(match case {
        Some(c) => match (c.heard, c.stage) {
            (Some(t), _) => Step::at("Decided", t.minutes(), capitalised(&case_words(sim, i.id))),
            (None, CaseStage::Open) => Step::not_reached("Decided", "it waits to be heard"),
            (None, _) => Step::not_reached("Decided", "it was never heard"),
        },
        None => Step::not_reached("Decided", "nothing to decide"),
    });
    let links = vec![Link(
        CAUSED_BY,
        TO_PERSON,
        i.actor.get(),
        format!("{}'s choice", pop.name_of(i.actor)),
    )];
    let truth = vec![format!("{} {}", pop.name_of(i.actor), row.happened)];
    let knowledge = vec![row.known.clone()];
    let polity = vec![row.polity.clone()];
    (row, steps, links, [truth, knowledge, polity])
}

fn put_link<'a>(fbb: &mut FlatBufferBuilder<'a>, l: &Link) -> WIPOffset<wire::IncidentLink<'a>> {
    let words = fbb.create_string(&l.3);
    wire::IncidentLink::create(
        fbb,
        &wire::IncidentLinkArgs {
            relation: l.0,
            target: l.1,
            id: l.2,
            words: Some(words),
        },
    )
}

/// A `Response` with incident `id` of `kind`'s page, or `None` when there is no such incident.
pub fn incident_response(sim: &Sim, kind: u8, id: u32) -> Option<Vec<u8>> {
    let pop = &sim.people;
    let f = Facts::of(sim);
    let (row, steps, links, [truth, knowledge, polity]) = match kind {
        TAKING => taking_page(sim, pop.order.incident(id)?),
        ILLNESS => illness_page(&f, pop.sickness.episodes().iter().find(|e| e.id == id)?),
        FIRE => fire::fire_page(sim, pop.fires.get(id)?),
        HIGH_WATER => {
            flood::high_water_page(sim, pop.floods.high_waters.iter().find(|hw| hw.id == id)?)
        }
        STORM_DAMAGE => {
            storm::storm_damage_page(sim, pop.floods.storm_damages.iter().find(|sd| sd.id == id)?)
        }
        WAIT => waits::wait_page(sim, pop.traffic.long_waits.iter().find(|w| w.id == id)?),
        SHAKEN => {
            let (id, q, x) = quake::shaken(sim).find(|&(n, _, _)| n == id)?;
            quake::shaken_page(sim, id, q, x)
        }
        _ => return None,
    };
    let mut fbb = FlatBufferBuilder::new();
    let row = put_row(&mut fbb, sim, &row);
    let steps: Vec<_> = steps
        .iter()
        .map(|s| {
            let label = fbb.create_string(s.label);
            let words = fbb.create_string(&s.words);
            let (minute, by_day, state) = match s.at {
                Some((m, d)) => (m, d, 0),
                None => (-1, false, 1),
            };
            wire::IncidentStep::create(
                &mut fbb,
                &wire::IncidentStepArgs {
                    label: Some(label),
                    minute,
                    state,
                    by_day,
                    words: Some(words),
                },
            )
        })
        .collect();
    let steps = fbb.create_vector(&steps);
    let links: Vec<_> = links.iter().map(|l| put_link(&mut fbb, l)).collect();
    let links = fbb.create_vector(&links);
    let truth = put_lines(&mut fbb, &truth);
    let knowledge = put_lines(&mut fbb, &knowledge);
    let polity = put_lines(&mut fbb, &polity);
    let body = wire::IncidentInfo::create(
        &mut fbb,
        &wire::IncidentInfoArgs {
            row: Some(row),
            steps: Some(steps),
            links: Some(links),
            truth: Some(truth),
            knowledge: Some(knowledge),
            polity: Some(polity),
        },
    );
    Some(response(fbb, wire::ResponseBody::IncidentInfo, body))
}

/// A node a load came from, in words: "the heap of Bo's household", "Bo's household's well".
fn node_words(sim: &Sim, node: Node) -> String {
    match node {
        Node::Midden(h) => format!("the heap of {}", super::people::household_name(sim, h)),
        Node::Person(p) => sim.people.name_of(p),
        Node::Store(h) => format!(
            "the stored water of {}",
            super::people::household_name(sim, h)
        ),
        other => source_words(sim, other),
    }
}

/// A load in words: "less than a dose", "1 dose", "120 doses".
fn dose_words(doses: f64) -> String {
    match doses.round() as i64 {
        ..=0 => "less than a dose".to_owned(),
        1 => "1 dose".to_owned(),
        n => format!("{n} doses"),
    }
}

/// What fouled a source with an outbreak's disease, as the kernel recorded it: the loads that
/// reached it, from where and how, from shortly before the outbreak began.
fn fouled_lines(sim: &Sim, o: &Outbreak, key: SourceKey) -> Vec<String> {
    if matches!(key, SourceKey::Spring(_)) {
        return Vec::new();
    }
    let into = |n: Node| match (key, n) {
        (SourceKey::Well(w), Node::Well(x)) => w == x,
        (SourceKey::River, Node::Reach(_)) => true,
        _ => false,
    };
    let since = o.began - FOULED_BEFORE_DAYS;
    let until = o.ended.unwrap_or(i64::MAX);
    let kept_from = sim.now().day_index() - civ_agents::contagion::MOVES_KEPT_DAYS;
    let mut by_from: BTreeMap<(Node, u8), (i64, f64)> = BTreeMap::new();
    for m in sim.people.contagion.moves.iter().filter(|m| {
        m.disease == o.disease
            && into(m.to)
            && (m.how == How::Runoff || m.how == How::Arrived)
            && m.day >= since
            && m.day <= until
    }) {
        let e = by_from
            .entry((m.from, m.how.code()))
            .or_insert((m.day, 0.0));
        e.0 = e.0.min(m.day);
        e.1 += m.amount;
    }
    if by_from.is_empty() {
        return vec![if since < kept_from {
            "What reached it then is no longer recorded".to_owned()
        } else {
            "No load of it reached this water from shortly before it began".to_owned()
        }];
    }
    let first = by_from.values().map(|v| v.0).min().unwrap_or(o.began);
    let mut parts: Vec<(f64, String)> = by_from
        .iter()
        .map(|(&(from, how), &(_, doses))| {
            let way = if how == How::Runoff.code() {
                "washed from"
            } else {
                "seeped through the ground from"
            };
            (
                doses,
                format!("{} {way} {}", dose_words(doses), node_words(sim, from)),
            )
        })
        .collect();
    parts.sort_by(|a, b| b.0.total_cmp(&a.0));
    vec![format!(
        "Fouled from {}: {}",
        day_text(first),
        parts
            .into_iter()
            .map(|p| p.1)
            .collect::<Vec<_>>()
            .join("; ")
    )]
}

/// One side of an outbreak's households: how many, how many had a case of it, and its cases.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Side {
    pub households: u32,
    pub with_case: u32,
    pub cases: u32,
}

impl Side {
    /// The share of its households that had a case, if it has any households.
    pub fn attack(&self) -> Option<f64> {
        (self.households > 0).then(|| f64::from(self.with_case) / f64::from(self.households))
    }

    fn add(&mut self, cases: u32) {
        self.households += 1;
        self.with_case += u32::from(cases > 0);
        self.cases += cases;
    }
}

/// An outbreak's households, each with its cases of it: its settlement's households with people
/// now, and every household one of its cases began in, so a household it emptied still counts.
fn outbreak_households(f: &Facts, o: &Outbreak, cases: &[&Episode]) -> BTreeMap<PermanentId, u32> {
    let mut out: BTreeMap<PermanentId, u32> = f
        .sim
        .people
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty() && o.settlement.is_none_or(|s| x.settlement == Some(s)))
        .map(|x| (x.id, 0))
        .collect();
    for e in cases {
        if let Some(h) = f.household_of(e) {
            *out.entry(h).or_default() += 1;
        }
    }
    out
}

/// The households that drew outbreak `o`'s disease in water at `key` from the day it began to
/// its end, as the moves the kernel keeps say (a draw too slight to count is not recorded).
fn drew_from(sim: &Sim, o: &Outbreak, key: SourceKey) -> BTreeSet<PermanentId> {
    let until = o.ended.unwrap_or(i64::MAX);
    sim.people
        .contagion
        .moves
        .iter()
        .filter(|m| {
            m.how == How::Drawn
                && m.disease == o.disease
                && m.day >= o.began
                && m.day <= until
                && node_key(m.from) == Some(key)
        })
        .filter_map(|m| match m.to {
            Node::Store(h) => Some(h),
            _ => None,
        })
        .collect()
}

/// An outbreak's households split into those that drew its disease at a source and the rest.
fn split(households: &BTreeMap<PermanentId, u32>, drew: &BTreeSet<PermanentId>) -> (Side, Side) {
    let (mut a, mut b) = (Side::default(), Side::default());
    for (h, &n) in households {
        if drew.contains(h) {
            a.add(n);
        } else {
            b.add(n);
        }
    }
    (a, b)
}

/// Who drew an outbreak's disease at a source, against the rest, in words: "Of 12 households
/// that drew cholera in its water since it began, 7 had a case (9 cases); of 30 that did not, 2
/// had a case (2 cases)".
fn exposure_line(sim: &Sim, o: &Outbreak, drew: Side, rest: Side) -> String {
    let disease = disease_name(sim, o.disease).to_lowercase();
    let kept_from = sim.now().day_index() - civ_agents::contagion::MOVES_KEPT_DAYS;
    let since = if o.began < kept_from {
        format!("from {}, the earliest still recorded", day_text(kept_from))
    } else {
        "since it began".to_owned()
    };
    let n = |x: u32, one: &str, many: &str| plural(x as usize, one, many);
    if drew.households == 0 {
        return format!(
            "No household drew {disease} in its water {since}; of {}, {} had a case ({})",
            n(rest.households, "household", "households"),
            rest.with_case,
            n(rest.cases, "case", "cases"),
        );
    }
    format!(
        "Of {} that drew {disease} in its water {since}, {} had a case ({}); of {} that did not, \
         {} had a case ({})",
        n(drew.households, "household", "households"),
        drew.with_case,
        n(drew.cases, "case", "cases"),
        n(rest.households, "household", "households"),
        rest.with_case,
        n(rest.cases, "case", "cases"),
    )
}

/// The source most of an outbreak's water cases took it at, with how many did and the outbreak's
/// households that drew its disease there against the rest.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceSplit {
    pub name: String,
    pub took: u32,
    pub drew: Side,
    pub rest: Side,
}

/// How an outbreak's cases sat against its sources (the dashboard's epidemics row, plan §4.7:
/// waterborne epidemics cluster on contaminated sources).
#[derive(Clone, Debug, PartialEq)]
pub struct Clustering {
    /// "Cholera at Hazelford, from 3 May of year 2 to 9 July of year 2".
    pub name: String,
    pub cases: u32,
    /// Those of its cases who took it in water.
    pub water_cases: u32,
    /// `None` when none of its cases took it in water.
    pub source: Option<SourceSplit>,
}

/// How outbreak `id`'s cases sat against the source most of its water cases took it at, or
/// `None` when there is no such outbreak. Reads the moves the kernel keeps for 400 days, so an
/// outbreak is best judged soon after it ends.
pub fn clustering(sim: &Sim, id: u32) -> Option<Clustering> {
    let o = sim
        .people
        .sickness
        .outbreaks()
        .iter()
        .find(|o| o.id == id)?;
    let f = Facts::of(sim);
    let cases: Vec<&Episode> = sim.people.sickness.cases(o.id).collect();
    let mut by_key: BTreeMap<SourceKey, u32> = BTreeMap::new();
    for e in &cases {
        if let Some(k) = f.source_of(e) {
            *by_key.entry(k).or_default() += 1;
        }
    }
    let water_cases = by_key.values().sum();
    // The commonest source; of two as common, the first in key order.
    let most = by_key
        .iter()
        .fold(None::<(SourceKey, u32)>, |best, (&k, &n)| match best {
            Some((_, m)) if m >= n => best,
            _ => Some((k, n)),
        });
    let source = most.map(|(key, took)| {
        let households = outbreak_households(&f, o, &cases);
        let (drew, rest) = split(&households, &drew_from(sim, o, key));
        SourceSplit {
            name: key_words(sim, key),
            took,
            drew,
            rest,
        }
    });
    let span = match o.ended {
        Some(end) => format!("from {} to {}", day_text(o.began), day_text(end)),
        None => format!("from {}", day_text(o.began)),
    };
    Some(Clustering {
        name: format!(
            "{} {}, {span}",
            disease_name(sim, o.disease),
            settlement_place(sim, o.settlement)
        ),
        cases: cases.len() as u32,
        water_cases,
        source,
    })
}

/// A `Response` with outbreak `id`'s page, its newest `limit` cases listed (the default for 0),
/// or `None` when there is no such outbreak.
pub fn outbreak_response(sim: &Sim, id: u32, limit: usize) -> Option<Vec<u8>> {
    let pop = &sim.people;
    let o = pop.sickness.outbreaks().iter().find(|o| o.id == id)?;
    let f = Facts::of(sim);
    let today = sim.now().day_index();
    let limit = if limit == 0 {
        DEFAULT_MEMBERS
    } else {
        limit.min(MOST_MEMBERS)
    };
    let cases: Vec<&Episode> = pop.sickness.cases(o.id).collect();
    // The curve: by the day they took it, and by the day they fell ill.
    let last = o.ended.unwrap_or(today).max(o.began);
    let days = (last - o.began + 1).clamp(1, CURVE_DAYS) as usize;
    let (mut took, mut onset) = (vec![0u16; days], vec![0u16; days]);
    let slot = |day: i64| usize::try_from(day - o.began).ok().filter(|&d| d < days);
    for e in &cases {
        if let Some(d) = slot(e.infected) {
            took[d] = took[d].saturating_add(1);
        }
        if let Some(d) =
            slot(e.course.ill_from).filter(|_| e.symptomatic() && e.course.ill_from <= today)
        {
            onset[d] = onset[d].saturating_add(1);
        }
    }
    // How its cases took it, the commonest way first.
    let mut ways: BTreeMap<(u8, String), u32> = BTreeMap::new();
    for e in &cases {
        let way = match e.acquired {
            Acquired::Observer { .. } => (0, "brought by the observer".to_owned()),
            Acquired::Household { .. } => (1, "at home, from those who shed it".to_owned()),
            Acquired::Company { .. } => (3, "in company at the hearth".to_owned()),
            Acquired::Inoculated { .. } => (4, "inoculated".to_owned()),
            Acquired::Sickhouse { .. } => (5, "at the sickhouse".to_owned()),
            Acquired::Water { source } => (
                2,
                format!("in water drawn at {}", source_words(sim, source)),
            ),
        };
        *ways.entry(way).or_default() += 1;
    }
    let mut ways: Vec<(u32, String)> = ways.into_iter().map(|((_, w), n)| (n, w)).collect();
    ways.sort_by_key(|w| std::cmp::Reverse(w.0));
    let routes: Vec<String> = ways.into_iter().map(|(n, w)| format!("{n} {w}")).collect();
    // The sources its cases drank from, those its settlement's people suspect, and those its
    // settlement's polity was asked to close since it began (a suspicion may fade, a law stays).
    let mut keys: BTreeSet<SourceKey> = cases.iter().filter_map(|e| f.source_of(e)).collect();
    for &(s, law, place) in &f.closures {
        if o.settlement.is_none_or(|st| st == s) && law.proposed.day_index() >= o.began {
            keys.extend(key_of(sim, place));
        }
    }
    for s in &pop.word.suspicions {
        let lives_here = pop
            .person(s.holder)
            .and_then(|p| pop.household(p.household))
            .is_some_and(|x| o.settlement.is_none_or(|st| x.settlement == Some(st)));
        if let Some(k) = key_of(sim, s.source).filter(|_| lives_here) {
            keys.insert(k);
        }
    }
    // Its households and the cases each had, split at each source by who drew the disease there.
    let households = outbreak_households(&f, o, &cases);
    let mut fbb = FlatBufferBuilder::new();
    let mut sources = Vec::new();
    for key in keys {
        let (drew, rest) = split(&households, &drew_from(sim, o, key));
        let took_here = cases.iter().filter(|e| f.source_of(e) == Some(key)).count() as u32;
        let mut truth = vec![exposure_line(sim, o, drew, rest)];
        truth.extend(fouled_lines(sim, o, key));
        let holders = suspecting(sim, key);
        let knowledge = vec![match holders.first() {
            None => "Nobody suspects it".to_owned(),
            Some(&(_, since)) => format!(
                "{} it, the first since {}",
                suspect_words(holders.len()),
                day_text(since)
            ),
        }];
        let laws = f.closures_of(key, o.settlement);
        let polity: Vec<String> = if laws.is_empty() {
            vec!["No law names it".to_owned()]
        } else {
            laws.iter()
                .map(|law| capitalised(&closure_words(sim, law)))
                .collect()
        };
        let name = fbb.create_string(&key_words(sim, key));
        let truth = put_lines(&mut fbb, &truth);
        let knowledge = put_lines(&mut fbb, &knowledge);
        let polity = put_lines(&mut fbb, &polity);
        sources.push((
            took_here,
            wire::OutbreakSource::create(
                &mut fbb,
                &wire::OutbreakSourceArgs {
                    name: Some(name),
                    cases: took_here,
                    truth: Some(truth),
                    knowledge: Some(knowledge),
                    polity: Some(polity),
                },
            ),
        ));
    }
    // The source most of its cases drank from first.
    sources.sort_by_key(|s| std::cmp::Reverse(s.0));
    let sources: Vec<_> = sources.into_iter().map(|s| s.1).collect();
    let sources = fbb.create_vector(&sources);
    let (cause, cause_link) = match cases.iter().find(|e| e.id == o.first) {
        Some(e) => {
            let link = caught_link(sim, e);
            (
                format!("Began with {}, {}", pop.name_of(e.person), link.3),
                Some(link),
            )
        }
        None => ("Its first case is not recorded".to_owned(), None),
    };
    let cause = fbb.create_string(&cause);
    let cause_link = cause_link.map(|l| put_link(&mut fbb, &l));
    let mut members: Vec<&Episode> = cases.clone();
    members.sort_by_key(|e| std::cmp::Reverse((e.infected, e.id)));
    let members: Vec<_> = members
        .into_iter()
        .take(limit)
        .map(|e| {
            let row = illness_row(&f, e).0;
            put_row(&mut fbb, sim, &row)
        })
        .collect();
    let members = fbb.create_vector(&members);
    let took = fbb.create_vector(&took);
    let onset = fbb.create_vector(&onset);
    let routes = put_lines(&mut fbb, &routes);
    let row = outbreak_row(&mut fbb, &f, o);
    let body = wire::OutbreakInfo::create(
        &mut fbb,
        &wire::OutbreakInfoArgs {
            row: Some(row),
            cause: Some(cause),
            cause_link,
            start: o.began * MINUTES_PER_DAY,
            took: Some(took),
            onset: Some(onset),
            routes: Some(routes),
            sources: Some(sources),
            members: Some(members),
            members_total: cases.len() as u32,
        },
    );
    Some(response(fbb, wire::ResponseBody::OutbreakInfo, body))
}

/// The incident or outbreak a chronicle entry tells of, as its kind and number, found in the
/// records the entry was written from, never by nearness in time alone (wire 1.65; 15-01 §3.5): a
/// taking by its taker and minute; an outbreak's beginning by its first case, its end by its
/// settlement and day; a death of a disease, and the observer's bringing one, by the case; a
/// fire, a conflagration, a death by fire and the observer's setting one (wire 1.68) as `fire`
/// finds them; a shaking, and a building that fell when the ground shook (wire 1.91), as `quake`
/// finds them.
pub fn chronicle_link(sim: &Sim, e: &ChronicleEvent) -> Option<(u8, u32)> {
    let pop = &sim.people;
    let who = e.people.first().copied();
    let day = e.at.day_index();
    match e.kind {
        ChronicleKind::Taking => pop
            .order
            .incidents
            .iter()
            .rev()
            .find(|i| i.at == e.at && Some(i.actor) == who)
            .map(|i| (TAKING, i.id)),
        ChronicleKind::Outbreak => {
            let outbreaks = pop.sickness.outbreaks();
            let found = if e.number.round() as u8 == OutbreakStep::Began as u8 {
                outbreaks.iter().find(|o| {
                    o.began == day
                        && pop
                            .sickness
                            .episodes()
                            .iter()
                            .any(|c| c.id == o.first && Some(c.person) == who)
                })
            } else {
                outbreaks
                    .iter()
                    .find(|o| o.ended == Some(day) && o.settlement == e.settlement)
            };
            found.map(|o| (OUTBREAK, o.id))
        }
        ChronicleKind::Died if e.name.starts_with("disease:") => pop
            .sickness
            .episodes()
            .iter()
            .rev()
            .find(|c| Some(c.person) == who && c.ended == Some((day, Outcome::Died)))
            .map(|c| (ILLNESS, c.id)),
        ChronicleKind::Influence
            if e.number.round() as u8 == civ_agents::influence::InfluenceKind::Plague.code() =>
        {
            pop.sickness
                .episodes()
                .iter()
                .rev()
                .find(|c| {
                    Some(c.person) == who
                        && c.infected == day
                        && matches!(c.acquired, Acquired::Observer { .. })
                })
                .map(|c| (ILLNESS, c.id))
        }
        ChronicleKind::Flood => flood::chronicle_link(sim, e),
        ChronicleKind::StormDamage => storm::chronicle_link(sim, e),
        ChronicleKind::Influence
            if e.number.round() as u8 == civ_agents::influence::InfluenceKind::Flood.code() =>
        {
            flood::chronicle_link(sim, e)
        }
        ChronicleKind::Influence
            if e.number.round() as u8 == civ_agents::influence::InfluenceKind::Storm.code() =>
        {
            storm::chronicle_link(sim, e)
        }
        ChronicleKind::Influence
            if e.number.round() as u8
                == civ_agents::influence::InfluenceKind::Earthquake.code() =>
        {
            quake::chronicle_link(sim, e)
        }
        ChronicleKind::BuildingFailed if e.name.contains(quake::SHOOK) => {
            quake::chronicle_link(sim, e)
        }
        ChronicleKind::Fire | ChronicleKind::Died | ChronicleKind::Influence => {
            fire::chronicle_link(sim, e)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_load_reads_in_doses_and_less_than_one_says_so() {
        assert_eq!(dose_words(0.3), "less than a dose");
        assert_eq!(dose_words(1.2), "1 dose");
        assert_eq!(dose_words(119.6), "120 doses");
    }

    #[test]
    fn lists_and_counts_read_as_words() {
        assert_eq!(names_text(&[]), "");
        assert_eq!(names_text(&["Ada".to_owned()]), "Ada");
        assert_eq!(
            names_text(&["Ada".to_owned(), "Bo".to_owned()]),
            "Ada and Bo"
        );
        assert_eq!(
            names_text(&["Ada".to_owned(), "Bo".to_owned(), "Cy".to_owned()]),
            "Ada, Bo and 1 other"
        );
        assert_eq!(
            names_text(&["A".into(), "B".into(), "C".into(), "D".into()]),
            "A, B and 2 others"
        );
        assert_eq!(plural(1, "case", "cases"), "1 case");
        assert_eq!(plural(3, "case", "cases"), "3 cases");
        assert_eq!(capitalised("the gathering"), "The gathering");
    }
}
