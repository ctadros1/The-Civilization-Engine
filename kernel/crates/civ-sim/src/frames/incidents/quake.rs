//! Shakings for the observer (wire 1.91, M8d slice DA; ADR-0031 §4): each building that fell or
//! cracked as an incident in the one shape, its page, a page for each shaking with what it
//! reached, and the chronicle's links to them. The kernel's record of a shaking never changes once
//! made; this only reads it.

use civ_agents::history::{ChronicleEvent, ChronicleKind};
use civ_agents::influence::InfluenceKind;
use civ_agents::polity::day_words;
use civ_agents::quake::{Outcome, Quake, Shaken};
use civ_core::PermanentId;
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use super::{
    CAUSED_BY, Link, PART_OF, Page, QUAKE, Row, SHAKEN, Step, TO_INFLUENCE, TO_QUAKE,
    building_words, capitalised, plural, put_lines, put_link, put_row, settlement_place,
};
use crate::Sim;
use crate::frames::response;

/// The words a building's failure carries in the chronicle when it gave way in a shaking.
pub(super) const SHOOK: &str = "when the ground shook";

/// Every building that fell or cracked in a shaking, with its incident number: a running count
/// from 1 over every shaking in the order they came, and within one in the order its buildings
/// were checked. A shaking's record is kept whole and never changes once made, so a number names
/// the same building after a save and a load.
pub(super) fn shaken(sim: &Sim) -> impl Iterator<Item = (u32, &Quake, &Shaken)> {
    sim.people
        .quakes
        .list
        .iter()
        .flat_map(|q| {
            q.shaken
                .iter()
                .filter(|s| s.outcome != Outcome::Stood)
                .map(move |s| (q, s))
        })
        .zip(1u32..)
        .map(|((q, s), id)| (id, q, s))
}

/// "the whole map", or "Hazelford".
fn where_words(sim: &Sim, q: &Quake) -> String {
    match q.settlement {
        Some(_) => settlement_place(sim, q.settlement),
        None => "over the whole map".to_owned(),
    }
}

/// How many of the buildings it reached did `outcome`, and how many died inside.
fn counts(q: &Quake) -> (u32, u32, u32) {
    let n = |o: Outcome| q.shaken.iter().filter(|s| s.outcome == o).count() as u32;
    let deaths = q.shaken.iter().map(|s| u32::from(s.deaths)).sum();
    (n(Outcome::Fell), n(Outcome::Cracked), deaths)
}

/// The observer's influence that sent it (0: none recorded).
fn observer_of(q: &Quake) -> u32 {
    q.origin.observers.first().copied().unwrap_or(q.influence)
}

pub(super) fn quake_row<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    q: &Quake,
) -> WIPOffset<wire::QuakeRow<'a>> {
    let (fell, cracked, deaths) = counts(q);
    let died = if deaths > 0 {
        format!("; {} inside", plural(deaths as usize, "died", "died"))
    } else {
        String::new()
    };
    let words = format!(
        "The ground shook {} on {} with {:.2} of a building's weight: {} reached; {fell} fell, \
         {cracked} cracked{died}",
        where_words(sim, q),
        day_words(q.at),
        q.strength,
        plural(q.shaken.len(), "building", "buildings"),
    );
    let words = fbb.create_string(&words);
    wire::QuakeRow::create(
        fbb,
        &wire::QuakeRowArgs {
            id: q.id,
            settlement: q.settlement.map_or(0, PermanentId::get),
            at: q.at.minutes(),
            strength: q.strength,
            reached: q.shaken.len() as u32,
            fell,
            cracked,
            deaths,
            observer: observer_of(q),
            words: Some(words),
        },
    )
}

/// Where building `building` stands, metres; (0, 0) when it is no longer recorded.
fn building_at(sim: &Sim, building: PermanentId) -> (f32, f32) {
    sim.land
        .buildings
        .iter()
        .find(|b| b.id == building)
        .map_or((0.0, 0.0), |b| civ_agents::build::centre_m(&b.spec))
}

/// "fell when the ground shook (it met 0.31 of its weight sideways); 1 died inside".
fn happened(s: &Shaken) -> String {
    let what = match s.outcome {
        Outcome::Fell => "fell",
        Outcome::Cracked => "cracked",
        Outcome::Stood => "stood",
    };
    let mut words = format!("{what} {SHOOK} (it met {:.2} of its weight sideways)", s.cs);
    if s.outcome == Outcome::Cracked && !s.cracked.is_empty() {
        words.push_str(&format!(
            "; {} cracked",
            plural(s.cracked.len(), "wall", "walls")
        ));
    }
    if s.deaths > 0 {
        words.push_str(&format!(
            "; {} inside",
            plural(usize::from(s.deaths), "died", "died")
        ));
    }
    words
}

/// Building `s`, shaken by `q`, as incident `id`.
pub(super) fn shaken_row(sim: &Sim, id: u32, q: &Quake, s: &Shaken) -> Row {
    Row {
        kind: SHAKEN,
        id,
        minute: q.at.minutes(),
        settlement: s.settlement,
        at: building_at(sim, s.building),
        who: s.building,
        who_name: Some(capitalised(&building_words(sim, s.building))),
        happened: happened(s),
        known: "No sightings recorded".to_owned(),
        polity: "No gathering was told of it".to_owned(),
        observer: observer_of(q),
        outbreak: 0,
        conflagration: 0,
        flood: 0,
        storm: 0,
        quake: q.id,
        open: false,
    }
}

pub(super) fn shaken_page(sim: &Sim, id: u32, q: &Quake, s: &Shaken) -> Page {
    let row = shaken_row(sim, id, q, s);
    let label = if s.outcome == Outcome::Fell {
        "Fell"
    } else {
        "Cracked"
    };
    let steps = vec![
        Step::at(
            "Shaken",
            q.at.minutes(),
            format!("it met {:.2} of its weight sideways", s.cs),
        ),
        Step::at(label, q.at.minutes(), String::new()),
    ];
    let links = vec![
        Link(
            PART_OF,
            TO_QUAKE,
            u64::from(q.id),
            format!("part of the shaking of {}", day_words(q.at)),
        ),
        Link(
            CAUSED_BY,
            TO_INFLUENCE,
            u64::from(observer_of(q)),
            "sent by the observer (one recorded influence)".to_owned(),
        ),
    ];
    let mut truth = vec![format!(
        "The shaking put {:.2} of a building's weight on it sideways; this building's own response \
         made it {:.2}",
        q.strength, s.cs
    )];
    match s.outcome {
        Outcome::Fell => truth.push(format!(
            "Something of it gave way; {} inside",
            if s.deaths == 0 {
                "nobody died".to_owned()
            } else {
                plural(usize::from(s.deaths), "died", "died")
            }
        )),
        _ => truth.push(format!(
            "{} cracked and nothing fell; a cracked wall holds no cohesion until it is mended",
            capitalised(&plural(s.cracked.len(), "wall", "walls"))
        )),
    }
    let knowledge = vec!["No sightings recorded".to_owned()];
    let polity = vec!["No gathering was told of it".to_owned()];
    (row, steps, links, [truth, knowledge, polity])
}

/// A `Response` with shaking `id`'s page, or `None` when there is no such shaking.
pub fn quake_response(sim: &Sim, id: u32) -> Option<Vec<u8>> {
    let q = sim.people.quakes.list.iter().find(|q| q.id == id)?;
    let mut fbb = FlatBufferBuilder::new();
    let row = quake_row(&mut fbb, sim, q);
    let obs = observer_of(q);
    let (cause, cause_link) = if obs > 0 {
        (
            "Sent by the observer (one recorded influence)".to_owned(),
            Some(Link(
                CAUSED_BY,
                TO_INFLUENCE,
                u64::from(obs),
                "sent by the observer".to_owned(),
            )),
        )
    } else {
        ("Its origin was not recorded".to_owned(), None)
    };
    let cause = fbb.create_string(&cause);
    let cause_link = cause_link.map(|l| put_link(&mut fbb, &l));
    let incidents: Vec<_> = shaken(sim)
        .filter(|&(_, x, _)| x.id == q.id)
        .map(|(n, x, s)| {
            let r = shaken_row(sim, n, x, s);
            put_row(&mut fbb, sim, &r)
        })
        .collect();
    let incidents = fbb.create_vector(&incidents);
    let (fell, cracked, deaths) = counts(q);
    let stood = q.shaken.len() as u32 - fell - cracked;
    let summary = vec![
        format!(
            "Strength: {:.2} of a building's weight, sideways, {}",
            q.strength,
            where_words(sim, q)
        ),
        format!(
            "Buildings reached: {}; {fell} fell, {cracked} cracked, {stood} stood",
            q.shaken.len()
        ),
        format!("Died inside buildings that fell: {deaths}"),
    ];
    let summary = put_lines(&mut fbb, &summary);
    let body = wire::QuakeInfo::create(
        &mut fbb,
        &wire::QuakeInfoArgs {
            row: Some(row),
            cause: Some(cause),
            cause_link,
            incidents: Some(incidents),
            summary: Some(summary),
        },
    );
    Some(response(fbb, wire::ResponseBody::QuakeInfo, body))
}

/// The shaking a chronicle entry tells of: the observer's shaking itself, or a building that gave
/// way in one. A shaking writes the failures of what fell, then its own entry, all at its minute,
/// so the entry belongs to the shaking whose own entry is the first at or after it: the shakings
/// at that minute counted by the shakings' own entries before it.
pub(super) fn chronicle_link(sim: &Sim, e: &ChronicleEvent) -> Option<(u8, u32)> {
    let pop = &sim.people;
    let at_minute: Vec<&Quake> = pop.quakes.list.iter().filter(|q| q.at == e.at).collect();
    let q = match at_minute.as_slice() {
        [] => return None,
        [one] => *one,
        many => {
            let earthquake = InfluenceKind::Earthquake.code();
            let before = usize::try_from(e.seq.saturating_sub(1)).unwrap_or(usize::MAX);
            let earlier = pop.chronicle[..before.min(pop.chronicle.len())]
                .iter()
                .rev()
                .take_while(|x| x.at == e.at)
                .filter(|x| {
                    x.kind == ChronicleKind::Influence && x.number.round() as u8 == earthquake
                })
                .count();
            *many.get(earlier).or(many.last())?
        }
    };
    Some((QUAKE, q.id))
}
