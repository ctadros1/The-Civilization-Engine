//! Knowledge on the boundary (M3b slice M; ADR-0008): the techniques in the welcome, what a
//! person knows, the snapshot's knowledge revision and the answer to a knowledge query. What a
//! settlement knows is derived from its people here, as the kernel derives it; every word is the
//! kernel's.

use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::knowledge::KnowledgeEventKind;
use civ_agents::params::Catalog;
use civ_agents::person::{KnowSource, Person};
use civ_core::{PermanentId, SimTime};
use civ_grammar::ProgramRules;
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use super::response;
use crate::Sim;

type Offsets<'a, T> = WIPOffset<Vector<'a, ForwardsUOffset<T>>>;

/// A technique's prerequisites in words: "Shaping wood and Knapping, or Quarrying" (empty for
/// none).
pub fn requires_words(catalog: &Catalog, t: usize) -> String {
    let Some(def) = catalog.techniques.get(t) else {
        return String::new();
    };
    def.requires
        .iter()
        .map(|route| {
            route
                .iter()
                .filter_map(|&u| catalog.techniques.get(u).map(|d| d.name.as_str()))
                .collect::<Vec<_>>()
                .join(" and ")
        })
        .collect::<Vec<_>>()
        .join(", or ")
}

/// The welcome's technique catalogue.
pub fn technique_infos<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    catalog: &Catalog,
) -> Offsets<'a, wire::TechniqueInfo<'a>> {
    let list: Vec<_> = catalog
        .techniques
        .iter()
        .enumerate()
        .map(|(t, d)| {
            let id = fbb.create_string(&d.id);
            let name = fbb.create_string(&d.name);
            let can = fbb.create_string(&d.can);
            let requires = fbb.create_string(&requires_words(catalog, t));
            wire::TechniqueInfo::create(
                fbb,
                &wire::TechniqueInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    can: Some(can),
                    domain: d.domain.map_or(-1, |k| k as i32),
                    requires: Some(requires),
                    learn_h: d.learn_h as f32,
                    upbringing: d.upbringing,
                },
            )
        })
        .collect();
    fbb.create_vector(&list)
}

fn name_of(sim: &Sim, id: PermanentId) -> String {
    sim.people
        .records
        .get(&id)
        .map_or_else(|| "someone".to_owned(), |r| r.given.clone())
}

fn settlement_name(sim: &Sim, id: PermanentId) -> String {
    sim.land
        .settlements
        .iter()
        .find(|x| x.id == id)
        .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
}

/// How a person came to know of a technique, in words.
pub fn source_words(sim: &Sim, source: KnowSource) -> String {
    match source {
        KnowSource::Founder => "brought it".to_owned(),
        KnowSource::Upbringing(p) => format!("brought up with it by {}", name_of(sim, p)),
        KnowSource::Taught(p) => format!("taught by {}", name_of(sim, p)),
        KnowSource::Found => "found it".to_owned(),
        KnowSource::Observer => "introduced by the observer".to_owned(),
        KnowSource::Seen(s) => format!("saw it at {}", settlement_name(sim, s)),
    }
}

/// What a person knows, is learning or has heard of.
pub fn know_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> Offsets<'a, wire::KnowLine<'a>> {
    let catalog = &sim.rules.catalog;
    let list: Vec<_> = p
        .knows
        .iter()
        .map(|k| {
            let learn_h = catalog
                .techniques
                .get(usize::from(k.technique))
                .map_or(0.0, |d| d.learn_h);
            let state = if k.known {
                2
            } else if k.hours > 0.0 {
                1
            } else {
                0
            };
            let source = fbb.create_string(&source_words(sim, k.source));
            wire::KnowLine::create(
                fbb,
                &wire::KnowLineArgs {
                    technique: k.technique,
                    state,
                    hours: k.hours,
                    learn_h: learn_h as f32,
                    since_minute: k.since.minutes(),
                    source: Some(source),
                    source_person: k.source.person().map_or(0, PermanentId::get),
                    used_minute: k.used.minutes(),
                },
            )
        })
        .collect();
    fbb.create_vector(&list)
}

/// A number that changes whenever someone comes to know, learns toward, hears of or loses a
/// technique, and when people arrive, leave or die (0 = no people).
pub fn knowledge_rev(sim: &Sim) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut ids: Vec<(u64, u32, u32, u64)> = sim
        .people
        .people
        .iter()
        .map(|(_, p)| {
            let known = p.knows.iter().filter(|k| k.known).count() as u32;
            let hours: f32 = p.knows.iter().map(|k| k.hours).sum();
            (
                p.id.get(),
                known,
                p.knows.len() as u32,
                f64::from(hours).round() as u64,
            )
        })
        .collect();
    if ids.is_empty() {
        return 0;
    }
    ids.sort_unstable();
    (ids, sim.people.knowledge.len()).hash(&mut hasher);
    // What settlements have seen of their buildings, as the panel words it.
    let trust: Vec<(u64, u16, u64, u64)> = sim
        .people
        .trust
        .iter()
        .map(|t| {
            (
                t.settlement.get(),
                t.technique,
                (t.failures * 10.0).round() as u64,
                t.years.round() as u64,
            )
        })
        .collect();
    trust.hash(&mut hasher);
    hasher.finish() | 1
}

/// How many times their usual strength builders of settlement `s` make the members of
/// technique `t`'s frame buildings now, and what they have seen in words: "built as usual; 31
/// building-years without a failure", "built 1.4 times as strong: failures weigh 2.0 against 31
/// building-years". A technique with no frame programs has no members to size (ADR-0009 §6), so
/// its buildings are built as usual whatever its builders have seen: "built as usual, as a hut
/// has nothing to size: failures weigh 0.9 against 28 building-years". `(1, "")` when nothing has
/// been built by it there.
pub fn trust_words(sim: &Sim, s: PermanentId, t: usize) -> (f64, String) {
    let Some(trust) = sim
        .people
        .trust
        .iter()
        .find(|x| x.settlement == s && usize::from(x.technique) == t)
    else {
        return (1.0, String::new());
    };
    let p = &sim.rules.people.build.caution;
    let now = sim.now();
    let caution = trust.caution(now, p);
    let (failures, years) = trust.faded(now, p.half_life_years);
    let stood = match years.round() as u64 {
        0 => "under a building-year".to_owned(),
        1 => "one building-year".to_owned(),
        n => format!("{n} building-years"),
    };
    let sized = sim
        .rules
        .catalog
        .buildings
        .iter()
        .any(|b| b.technique == Some(t) && matches!(b.rules, ProgramRules::Frame(_)));
    let words = if failures < 0.05 {
        format!("built as usual; {stood} without a failure")
    } else if !sized {
        return (
            1.0,
            format!(
                "built as usual, as a hut has nothing to size: failures weigh {failures:.1} \
                 against {stood}"
            ),
        );
    } else if caution < 1.05 {
        format!("built as usual: failures weigh {failures:.1} against {stood}")
    } else {
        format!("built {caution:.1} times as strong: failures weigh {failures:.1} against {stood}")
    };
    (caution, words)
}

fn person_ref<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    p: &Person,
    now: SimTime,
) -> WIPOffset<wire::PersonRef<'a>> {
    let name = fbb.create_string(&p.given);
    wire::PersonRef::create(
        fbb,
        &wire::PersonRefArgs {
            id: p.id.get(),
            name: Some(name),
            age_years: p.age_years(now) as f32,
        },
    )
}

/// A settlement's record of a technique, oldest first, in sentences.
fn history_words(sim: &Sim, s: PermanentId, t: usize) -> Vec<String> {
    sim.people
        .knowledge
        .iter()
        .filter(|e| e.settlement == s && usize::from(e.technique) == t)
        .map(|e| {
            let year = e.at.date().year;
            let who = name_of(sim, e.person);
            // A knower who came from another settlement brought it from there; a loss says where
            // it is still known among those people here have kin or friends in (M5b slice AR).
            match (e.kind, e.elsewhere) {
                (KnowledgeEventKind::Known(_), Some(from)) => {
                    return format!(
                        "Year {year}: {who} brought it from {}.",
                        settlement_name(sim, from)
                    );
                }
                (KnowledgeEventKind::Lost, Some(at)) => {
                    return format!(
                        "Year {year}: lost with {who}; still known at {}, where people here have \
                         kin or friends.",
                        settlement_name(sim, at)
                    );
                }
                _ => {}
            }
            match e.kind {
                KnowledgeEventKind::Known(KnowSource::Founder) => {
                    format!("Year {year}: brought by {who}.")
                }
                KnowledgeEventKind::Known(KnowSource::Upbringing(from)) => format!(
                    "Year {year}: {who} was brought up with it by {}.",
                    name_of(sim, from)
                ),
                KnowledgeEventKind::Known(KnowSource::Taught(from)) => {
                    format!("Year {year}: {who} learnt it from {}.", name_of(sim, from))
                }
                KnowledgeEventKind::Known(KnowSource::Found) => {
                    format!("Year {year}: {who} worked it out.")
                }
                KnowledgeEventKind::Known(KnowSource::Observer) => {
                    format!("Year {year}: the observer taught {who}.")
                }
                KnowledgeEventKind::Known(KnowSource::Seen(at)) => {
                    format!("Year {year}: {who} saw it at {}.", settlement_name(sim, at))
                }
                KnowledgeEventKind::Lost => format!("Year {year}: lost with {who}."),
            }
        })
        .collect()
}

/// Who among `residents` knows technique `t`, is learning it, and has only heard of it.
fn split<'p>(
    residents: &[&'p Person],
    t: usize,
) -> (Vec<&'p Person>, Vec<&'p Person>, Vec<&'p Person>) {
    let mut knowers = Vec::new();
    let mut learners = Vec::new();
    let mut heard = Vec::new();
    for p in residents {
        match p.know(t) {
            Some(k) if k.known => knowers.push(*p),
            Some(k) if k.hours > 0.0 => learners.push(*p),
            Some(_) => heard.push(*p),
            None => {}
        }
    }
    (knowers, learners, heard)
}

fn refs<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    people: &[&Person],
    now: SimTime,
) -> Offsets<'a, wire::PersonRef<'a>> {
    let list: Vec<_> = people.iter().map(|p| person_ref(fbb, p, now)).collect();
    fbb.create_vector(&list)
}

/// A `Response` with every settlement's knowledge: each technique known there, ever known there
/// or known of there, with its knowers (eldest first), learners and those who have only heard of
/// it, how many practised it in the last year, its state in words and its history.
pub fn knowledge_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let catalog = &sim.rules.catalog;
    let now = sim.now();
    let year_ago = now.plus_minutes(-civ_core::time::MINUTES_PER_YEAR);
    let settlements: BTreeSet<PermanentId> = sim
        .people
        .households
        .iter()
        .filter_map(|(_, x)| x.settlement)
        .chain(sim.people.knowledge.iter().map(|e| e.settlement))
        .chain(sim.people.trust.iter().map(|t| t.settlement))
        .collect();
    let mut list = Vec::new();
    for s in settlements {
        let mut residents: Vec<&Person> = sim
            .people
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(s))
            .flat_map(|(_, x)| x.members.iter())
            .filter_map(|m| sim.people.person(*m))
            .collect();
        residents.sort_by_key(|p| (p.born, p.id));
        let mut techniques = Vec::new();
        for (t, def) in catalog.techniques.iter().enumerate() {
            let recorded = sim
                .people
                .knowledge
                .iter()
                .any(|e| e.settlement == s && usize::from(e.technique) == t);
            // What its builders there have seen keeps it on the list once nobody knows it.
            let trusted = sim
                .people
                .trust
                .iter()
                .any(|x| x.settlement == s && usize::from(x.technique) == t);
            let (knowers, learners, heard) = split(&residents, t);
            if !recorded
                && !trusted
                && knowers.is_empty()
                && learners.is_empty()
                && heard.is_empty()
            {
                continue;
            }
            let practised = knowers
                .iter()
                .filter(|p| p.know(t).is_some_and(|k| k.used > year_ago))
                .count() as u32;
            let status = status_words(sim, s, t, (&knowers, &learners, &heard), def.upbringing);
            let history: Vec<_> = history_words(sim, s, t)
                .iter()
                .map(|h| fbb.create_string(h))
                .collect();
            let history = fbb.create_vector(&history);
            let knowers_v = refs(&mut fbb, &knowers, now);
            let learners_v = refs(&mut fbb, &learners, now);
            let heard_v = refs(&mut fbb, &heard, now);
            let status = fbb.create_string(&status);
            let (caution, trust) = trust_words(sim, s, t);
            let trust = fbb.create_string(&trust);
            techniques.push(wire::TechniqueHere::create(
                &mut fbb,
                &wire::TechniqueHereArgs {
                    technique: t as u16,
                    known: !knowers.is_empty(),
                    knowers: Some(knowers_v),
                    learners: Some(learners_v),
                    heard: Some(heard_v),
                    practised_last_year: practised,
                    status: Some(status),
                    history: Some(history),
                    caution: caution as f32,
                    trust: Some(trust),
                },
            ));
        }
        let techniques = fbb.create_vector(&techniques);
        let name = sim
            .land
            .settlements
            .iter()
            .find(|x| x.id == s)
            .map_or("", |x| x.name.as_str());
        let name = fbb.create_string(name);
        list.push(wire::SettlementKnowledge::create(
            &mut fbb,
            &wire::SettlementKnowledgeArgs {
                settlement: s.get(),
                name: Some(name),
                techniques: Some(techniques),
            },
        ));
    }
    let list = fbb.create_vector(&list);
    let body = wire::Knowledge::create(
        &mut fbb,
        &wire::KnowledgeArgs {
            rev: knowledge_rev(sim),
            settlements: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Knowledge, body)
}

/// A technique's state in a settlement, in words: "known by 12", "known by one, Ash, aged 61;
/// nobody learning", "lost in year 9 with Wren; two still know of it".
fn status_words(
    sim: &Sim,
    s: PermanentId,
    t: usize,
    (knowers, learners, heard): (&[&Person], &[&Person], &[&Person]),
    upbringing: bool,
) -> String {
    let now = sim.now();
    let count = |n: usize| match n {
        1 => "one".to_owned(),
        2 => "two".to_owned(),
        3 => "three".to_owned(),
        n => n.to_string(),
    };
    let learning = match learners.len() {
        0 if upbringing => String::new(),
        0 => "; nobody learning".to_owned(),
        n => format!("; {} learning", count(n)),
    };
    match knowers {
        [] => {
            let lost = sim
                .people
                .knowledge
                .iter()
                .rev()
                .find(|e| e.settlement == s && usize::from(e.technique) == t)
                .filter(|e| e.kind == KnowledgeEventKind::Lost)
                .map(|e| {
                    format!(
                        "lost in year {} with {}",
                        e.at.date().year,
                        name_of(sim, e.person)
                    )
                });
            let aware = learners.len() + heard.len();
            let tail = if aware > 0 {
                format!("; {} still know of it", count(aware))
            } else {
                String::new()
            };
            match lost {
                Some(l) => format!("{l}{tail}"),
                None => format!("known of, not known{tail}"),
            }
        }
        [one] => format!(
            "known by one, {}, aged {:.0}{learning}",
            one.given,
            one.age_years(now).floor()
        ),
        many => format!("known by {}{learning}", count(many.len())),
    }
}
