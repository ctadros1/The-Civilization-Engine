//! People on the boundary (ADR-0003 §2–3): the snapshot's per-person briefs and settlements, and
//! the responses to trip, person and chronicle queries.
//!
//! Every piece of text here is rendered by the kernel: what someone is doing, where a target is,
//! the chronicle. Observers show it; they never compose explanations of their own.

use std::collections::HashSet;

use civ_agents::history::{self, Span};
use civ_agents::params::GoodUse;
use civ_agents::person::{food_kcal, fuel_kg, stock_kcal};
use civ_agents::{Cause, Origin, Person, Receipt, Repro, Scored, Sex, Step, Target, population};
use civ_core::PermanentId;
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use super::{QueryError, response};
use crate::Sim;

/// Most trips one query may ask for.
pub const MAX_TRIPS_PER_QUERY: usize = 256;
/// Most chronicle entries one response carries.
pub const MAX_CHRONICLE_PER_QUERY: u32 = 256;
/// Most decision receipts one person response carries.
pub const MAX_DECISIONS_PER_QUERY: u32 = 64;

type Offsets<'a, T> = WIPOffset<Vector<'a, ForwardsUOffset<T>>>;

fn vec2((x, y): (f32, f32)) -> wire::Vec2 {
    wire::Vec2::new(x, y)
}

fn sex(s: Sex) -> wire::Sex {
    match s {
        Sex::Female => wire::Sex::Female,
        Sex::Male => wire::Sex::Male,
    }
}

/// The snapshot's entry for every living person, in permanent-id order, placed where they are at
/// the current minute.
pub fn person_briefs<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
) -> Offsets<'a, wire::PersonBrief<'a>> {
    let now = sim.now();
    let t = now.minutes() as f64;
    let mut people: Vec<&Person> = sim.people.people.iter().map(|(_, p)| p).collect();
    people.sort_by_key(|p| p.id);
    let briefs: Vec<_> = people
        .into_iter()
        .map(|p| {
            let (trip, trip_rev) = p.trip.as_ref().map_or((0, 0), |t| (t.id, t.rev));
            wire::PersonBrief::create(
                fbb,
                &wire::PersonBriefArgs {
                    id: p.id.get(),
                    pos: Some(&vec2(p.position_at(t))),
                    activity: p.act.def,
                    trip,
                    trip_rev,
                    sex: sex(p.sex),
                    age_years: p.age_years(now) as f32,
                    household: p.household.get(),
                    asleep: p.asleep,
                },
            )
        })
        .collect();
    fbb.create_vector(&briefs)
}

/// The snapshot's settlements, oldest first, with their living population.
pub fn settlement_briefs<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
) -> Offsets<'a, wire::SettlementBrief<'a>> {
    let briefs: Vec<_> = sim
        .land
        .settlements
        .iter()
        .map(|s| {
            let population: usize = sim
                .people
                .households
                .iter()
                .filter(|(_, h)| h.settlement == Some(s.id))
                .map(|(_, h)| h.members.len())
                .sum();
            let name = fbb.create_string(&s.name);
            let food_days = sim.people.settlement_food_days(
                s.id,
                sim.now(),
                &sim.rules.people,
                &sim.rules.catalog.goods,
            );
            wire::SettlementBrief::create(
                fbb,
                &wire::SettlementBriefArgs {
                    id: s.id.get(),
                    name: Some(name),
                    hearth: Some(&vec2(s.hearth_m)),
                    founded_minute: s.founded.minutes(),
                    population: population as u32,
                    food_days: food_days.unwrap_or(0.0) as f32,
                    food_short: s.food_short,
                    harvest_kg: s.harvest_kg as f32,
                },
            )
        })
        .collect();
    fbb.create_vector(&briefs)
}

/// The newest chronicle entry's sequence number (0 = none).
pub fn chronicle_head(sim: &Sim) -> u64 {
    sim.people.chronicle.last().map_or(0, |e| e.seq)
}

/// A `Response` with the geometry of the asked-for trips that are still under way.
pub fn trips_response(sim: &Sim, ids: &[u64]) -> Result<Vec<u8>, QueryError> {
    if ids.len() > MAX_TRIPS_PER_QUERY {
        return Err(QueryError(format!(
            "{} trips asked for; at most {MAX_TRIPS_PER_QUERY} per query",
            ids.len()
        )));
    }
    let wanted: HashSet<u64> = ids.iter().copied().collect();
    let mut fbb = FlatBufferBuilder::new();
    let mut found: Vec<&Person> = sim
        .people
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.trip.as_ref().is_some_and(|t| wanted.contains(&t.id)))
        .collect();
    found.sort_by_key(|p| p.id);
    let trips: Vec<_> = found
        .into_iter()
        .filter_map(|p| {
            let t = p.trip.as_ref()?;
            let points: Vec<wire::Vec2> = t.points.iter().map(|&q| vec2(q)).collect();
            let points = fbb.create_vector(&points);
            let minutes = fbb.create_vector(&t.minutes);
            Some(wire::TripInfo::create(
                &mut fbb,
                &wire::TripInfoArgs {
                    id: t.id,
                    rev: t.rev,
                    person: p.id.get(),
                    depart_minute: t.depart.minutes(),
                    points: Some(points),
                    minutes: Some(minutes),
                },
            ))
        })
        .collect();
    let trips = fbb.create_vector(&trips);
    let body = wire::Trips::create(&mut fbb, &wire::TripsArgs { trips: Some(trips) });
    Ok(response(fbb, wire::ResponseBody::Trips, body))
}

/// A `Response` with chronicle entries after `after_seq`, oldest first.
pub fn chronicle_response(sim: &Sim, after_seq: u64, limit: u32) -> Vec<u8> {
    let limit = limit.clamp(1, MAX_CHRONICLE_PER_QUERY) as usize;
    let pop = &sim.people;
    let name_of = |id: PermanentId| pop.name_of(id);
    let mut fbb = FlatBufferBuilder::new();
    let entries: Vec<_> = pop
        .chronicle
        .iter()
        .filter(|e| e.seq > after_seq)
        .take(limit)
        .map(|e| {
            let spans = history::render(e, &name_of);
            let spans: Vec<_> = spans
                .iter()
                .map(|s| {
                    let (kind, text, id) = match s {
                        Span::Text(t) => (wire::SpanKind::Text, t.as_str(), 0),
                        Span::Person(id, n) => (wire::SpanKind::Person, n.as_str(), id.get()),
                        Span::Settlement(id, n) => {
                            (wire::SpanKind::Settlement, n.as_str(), id.get())
                        }
                        Span::Firm(id, n) => (wire::SpanKind::Firm, n.as_str(), id.get()),
                    };
                    let text = fbb.create_string(text);
                    wire::Span::create(
                        &mut fbb,
                        &wire::SpanArgs {
                            kind,
                            text: Some(text),
                            id,
                        },
                    )
                })
                .collect();
            let spans = fbb.create_vector(&spans);
            wire::ChronicleEntry::create(
                &mut fbb,
                &wire::ChronicleEntryArgs {
                    seq: e.seq,
                    minute: e.at.minutes(),
                    spans: Some(spans),
                },
            )
        })
        .collect();
    let entries = fbb.create_vector(&entries);
    let body = wire::Chronicle::create(
        &mut fbb,
        &wire::ChronicleArgs {
            entries: Some(entries),
            head: chronicle_head(sim),
        },
    );
    response(fbb, wire::ResponseBody::Chronicle, body)
}

/// Distance and compass direction from `from` to `to`, in words: "600 m east".
pub fn bearing(from: (f32, f32), to: (f32, f32)) -> String {
    let (dx, dy) = (f64::from(to.0 - from.0), f64::from(to.1 - from.1));
    let d = dx.hypot(dy);
    if d < 5.0 {
        return "here".to_owned();
    }
    let distance = if d < 100.0 {
        format!("{} m", ((d / 10.0).round() * 10.0) as i64)
    } else if d < 1000.0 {
        format!("{} m", ((d / 50.0).round() * 50.0) as i64)
    } else {
        format!("{:.1} km", d / 1000.0)
    };
    // Map y runs south; compass angles run anticlockwise from east.
    let angle = (-dy).atan2(dx).to_degrees().rem_euclid(360.0);
    const WINDS: [&str; 8] = [
        "east",
        "north-east",
        "north",
        "north-west",
        "west",
        "south-west",
        "south",
        "south-east",
    ];
    let wind = WINDS[((angle + 22.5) / 45.0) as usize % 8];
    format!("{distance} {wind}")
}

/// What a target of a member of `household` is, in words, seen from `home`.
pub fn describe_target(
    sim: &Sim,
    home: (f32, f32),
    household: PermanentId,
    target: Target,
) -> String {
    match target {
        Target::None => String::new(),
        Target::Home => "home".to_owned(),
        Target::Hearth => "the hearth".to_owned(),
        Target::Patch(p) if (p as usize) < sim.land.patches.len() => {
            let class = sim.land.patches.class[p as usize] as usize;
            let habitat = sim
                .rules
                .land
                .habitats
                .get(class)
                .map_or("land", |h| h.name.as_str())
                .to_lowercase();
            let at = sim.land.patches.centre_m(p as usize);
            format!("{habitat} {} of home", bearing(home, at))
        }
        Target::Patch(_) => "a patch".to_owned(),
        Target::Water(cell) => {
            let at = population::cell_centre(&sim.map, cell as usize);
            format!("water {} of home", bearing(home, at))
        }
        Target::Field(id) => match sim.land.fields.iter().find(|f| f.id == id) {
            Some(f) => format!("the field {} of home", bearing(home, f.rect.centre_m())),
            None => "a field".to_owned(),
        },
        Target::NewField => "new ground".to_owned(),
        Target::Household(id) => household_name(sim, id),
        Target::Building(id) => match sim.land.buildings.iter().find(|b| b.id == id) {
            Some(b) => {
                let name = program_name(sim, &b.spec.program);
                let at = civ_agents::build::centre_m(&b.spec);
                if (at.0 - home.0).abs() < 1.0 && (at.1 - home.1).abs() < 1.0 {
                    format!("the {name} at home")
                } else {
                    format!("a {name} {} of home", bearing(home, at))
                }
            }
            None => "a building".to_owned(),
        },
        Target::NewBuilding => {
            // The home the household plans, or else the first it may build.
            let planned = sim
                .people
                .planned_home(household)
                .map(|s| s.program.as_str());
            let program = planned
                .or_else(|| {
                    sim.rules
                        .people
                        .build
                        .programs
                        .first()
                        .and_then(|&p| sim.rules.catalog.buildings.get(p))
                        .map(|b| b.id.as_str())
                })
                .unwrap_or("");
            format!("a new {}", program_name(sim, program))
        }
        Target::Firm(id) => firm_name(sim, id),
        Target::NewFirm => "a new workshop".to_owned(),
        // What trying works toward: the observer sees it, though the person cannot.
        Target::Technique(t) => sim
            .rules
            .catalog
            .techniques
            .get(usize::from(t))
            .map_or_else(
                || "something new".to_owned(),
                |d| format!("toward {}", d.name.to_lowercase()),
            ),
        // The deposit dug at: "the clay pit north-east of home".
        Target::Deposit(id) => match sim.land.deposits.iter().find(|d| d.id == id) {
            Some(d) => {
                let good = sim
                    .rules
                    .catalog
                    .goods
                    .get(usize::from(d.body.good))
                    .map_or_else(|| "a".to_owned(), |g| g.name.to_lowercase());
                let at = (
                    (d.body.at_cm.0 as f64 / 100.0) as f32,
                    (d.body.at_cm.1 as f64 / 100.0) as f32,
                );
                format!("the {good} pit {} of home", bearing(home, at))
            }
            None => "a pit".to_owned(),
        },
    }
}

/// A firm in words, by its founder and what it makes: "Wren's sickle workshop".
pub fn firm_name(sim: &Sim, id: PermanentId) -> String {
    match sim.people.firm(id) {
        Some(f) => f.name(&sim.people.name_of(f.founder), &sim.rules.catalog.goods),
        None => "a workshop".to_owned(),
    }
}

/// A building program's name in running text: "hut".
pub(crate) fn program_name(sim: &Sim, program: &str) -> String {
    sim.rules.catalog.building_index(program).map_or_else(
        || "building".to_owned(),
        |i| sim.rules.catalog.buildings[i].name.to_lowercase(),
    )
}

/// The given name of a household's eldest member; `None` when nobody lives in it.
pub fn eldest_name(sim: &Sim, id: PermanentId) -> Option<String> {
    let now = sim.now();
    sim.people
        .household(id)
        .and_then(|h| {
            h.members
                .iter()
                .filter_map(|m| sim.people.person(*m))
                .max_by(|a, b| a.age_years(now).total_cmp(&b.age_years(now)))
        })
        .map(|p| p.given.clone())
}

/// A household in words, by its eldest member: "Ada's household".
pub fn household_name(sim: &Sim, id: PermanentId) -> String {
    match eldest_name(sim, id) {
        Some(name) => format!("{name}'s household"),
        None => "another household".to_owned(),
    }
}

/// What a living person is doing now, in words.
pub fn doing(sim: &Sim, p: &Person) -> String {
    let def = sim.rules.catalog.activities.get(usize::from(p.act.def));
    let what = def.map_or("busy", |d| d.doing.as_str());
    let home = sim.people.household(p.household).map(|h| h.home);
    let place = home.map_or_else(String::new, |h| {
        describe_target(sim, h, p.household, p.act.target)
    });
    match p.act.steps.get(usize::from(p.act.step)) {
        Some(Step::Walk { to }) => {
            if home.is_some_and(|h| (h.0 - to.0).abs() < 1.0 && (h.1 - to.1).abs() < 1.0) {
                if p.act.target == Target::Home {
                    format!("walking home ({what})")
                } else {
                    "walking home".to_owned()
                }
            } else if place.is_empty() {
                format!("walking ({what})")
            } else {
                format!("walking to {place} ({what})")
            }
        }
        Some(Step::Work { .. }) => {
            let at_home =
                home.is_some_and(|h| (h.0 - p.pos.0).abs() < 1.0 && (h.1 - p.pos.1).abs() < 1.0);
            match p.act.target {
                _ if place.is_empty() => what.to_owned(),
                // Threshing is done at home, with sheaves carried from the field.
                Target::Field(_) if at_home => format!("{what} from {place}"),
                Target::Patch(_) | Target::Water(_) | Target::Field(_) => {
                    format!("{what}, {place}")
                }
                Target::Household(_) | Target::Deposit(_) => format!("{what} at {place}"),
                Target::Building(id) => {
                    match sim
                        .land
                        .buildings
                        .iter()
                        .find(|b| b.id == id)
                        .and_then(|b| b.stage())
                    {
                        Some(stage) => format!("{what}: the {}", stage.name()),
                        None => what.to_owned(),
                    }
                }
                _ => what.to_owned(),
            }
        }
        Some(Step::Deposit) => "putting away what they brought".to_owned(),
        Some(Step::Wait { .. }) => "waiting".to_owned(),
        None => "deciding what to do".to_owned(),
    }
}

fn scored<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    (home, household): ((f32, f32), PermanentId),
    s: &Scored,
) -> WIPOffset<wire::ScoredOption<'a>> {
    let target = fbb.create_string(&describe_target(sim, home, household, s.target));
    let terms: Vec<wire::Term> = s
        .terms
        .iter()
        .map(|t| wire::Term::new(t.reason as u16, t.points))
        .collect();
    let terms = fbb.create_vector(&terms);
    wire::ScoredOption::create(
        fbb,
        &wire::ScoredOptionArgs {
            activity: s.def,
            target: Some(target),
            total: s.total,
            terms: Some(terms),
        },
    )
}

fn decision<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    from: ((f32, f32), PermanentId),
    r: &Receipt,
) -> WIPOffset<wire::Decision<'a>> {
    let chosen = scored(fbb, sim, from, &r.chosen);
    let runner_up = r.runner_up.as_ref().map(|s| scored(fbb, sim, from, s));
    let others: Vec<_> = r
        .others
        .iter()
        .map(|&(def, total)| {
            wire::ScoredOption::create(
                fbb,
                &wire::ScoredOptionArgs {
                    activity: def,
                    total,
                    ..Default::default()
                },
            )
        })
        .collect();
    let others = fbb.create_vector(&others);
    let excluded: Vec<wire::Exclusion> = r
        .excluded
        .iter()
        .map(|&(def, reason)| wire::Exclusion::new(def, reason as u16))
        .collect();
    let excluded = fbb.create_vector(&excluded);
    let needs = fbb.create_vector(&r.needs);
    wire::Decision::create(
        fbb,
        &wire::DecisionArgs {
            minute: r.at.minutes(),
            chosen: Some(chosen),
            runner_up,
            others: Some(others),
            excluded: Some(excluded),
            probability: r.probability,
            temperature: r.temperature,
            needs: Some(needs),
        },
    )
}

/// A person's relatives, by permanent id, with how they are related.
fn kin(sim: &Sim, id: PermanentId) -> Vec<(PermanentId, &'static str)> {
    let records = &sim.people.records;
    let Some(me) = records.get(&id) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(m) = me.mother {
        out.push((m, "mother"));
    }
    if let Some(f) = me.father {
        out.push((f, "father"));
    }
    // Partners from the unions they were in; for a world older than unions, the other parent of
    // their children.
    let mut partners: Vec<PermanentId> = sim
        .people
        .unions
        .iter()
        .filter_map(|u| {
            if u.woman == id {
                Some(u.man)
            } else if u.man == id {
                Some(u.woman)
            } else {
                None
            }
        })
        .collect();
    for r in records.values() {
        if r.id == id {
            continue;
        }
        if r.mother == Some(id) || r.father == Some(id) {
            out.push((
                r.id,
                if r.sex == Sex::Male {
                    "son"
                } else {
                    "daughter"
                },
            ));
            let other = if r.mother == Some(id) {
                r.father
            } else {
                r.mother
            };
            if let Some(o) = other
                && !partners.contains(&o)
            {
                partners.push(o);
            }
        } else if (me.mother.is_some() && r.mother == me.mother)
            || (me.father.is_some() && r.father == me.father)
        {
            out.push((
                r.id,
                if r.sex == Sex::Male {
                    "brother"
                } else {
                    "sister"
                },
            ));
        }
    }
    for p in partners {
        out.push((p, "partner"));
    }
    out
}

/// "21 years", "8 months", "a few days".
fn how_long(minutes: i64) -> String {
    let days = minutes.max(0) as f64 / 1440.0;
    let years = days / 365.0;
    if years >= 2.0 {
        format!("{} years", years.floor() as i64)
    } else if years >= 1.0 {
        "a year".to_owned()
    } else if days >= 61.0 {
        format!("{} months", (days / 30.4).floor() as i64)
    } else if days >= 14.0 {
        format!("{} weeks", (days / 7.0).floor() as i64)
    } else {
        "a few days".to_owned()
    }
}

/// "in about 3 months", "in about 5 days".
fn in_about(minutes: i64) -> String {
    let days = minutes.max(0) as f64 / 1440.0;
    if days < 1.5 {
        "any day now".to_owned()
    } else if days < 14.0 {
        format!("in about {} days", days.round() as i64)
    } else if days < 60.0 {
        format!("in about {} weeks", (days / 7.0).round() as i64)
    } else {
        format!("in about {} months", (days / 30.4).round() as i64)
    }
}

/// A living person's family life in sentences, as the inspector shows it: their partner and since
/// when, or their widowhood; a pregnancy once it shows (the kernel knows more, such as whether it
/// will be lost, and does not say); the child they nurse.
pub fn family_notes(sim: &Sim, p: &Person) -> Vec<String> {
    let (now, pop) = (sim.now(), &sim.people);
    let mine = |u: &&civ_agents::Union| u.woman == p.id || u.man == p.id;
    let other = |u: &civ_agents::Union| if u.woman == p.id { u.man } else { u.woman };
    let mut out = Vec::new();
    if let Some(q) = p.partner {
        match pop
            .unions
            .iter()
            .rev()
            .filter(mine)
            .find(|u| u.ended.is_none())
        {
            Some(u) => out.push(format!(
                "Partner of {} for {}.",
                pop.name_of(q),
                how_long(now.minutes() - u.since.minutes())
            )),
            None => out.push(format!("Partner of {}.", pop.name_of(q))),
        }
    } else if let Some(u) = pop.unions.iter().rev().find(mine)
        && let Some(ended) = u.ended
    {
        out.push(format!(
            "Widowed when {} died in year {}.",
            pop.name_of(other(u)),
            ended.date().year
        ));
    }
    if let Repro::Pregnant { conceived, .. } = p.repro {
        let weeks = (now.minutes() - conceived.minutes()) / (7 * 1440);
        if weeks >= 8 {
            let days = sim.rules.people.fertility.pregnancy_days;
            let due = conceived.plus_minutes((days * 1440.0) as i64);
            out.push(format!(
                "Expecting a child, due {}.",
                in_about(due.minutes() - now.minutes())
            ));
        }
    }
    if let Some(child) = p.nursing.and_then(|c| pop.person(c)) {
        out.push(format!("Nursing {}.", child.given));
    }
    out
}

/// A `Response` describing one person, living or dead, with up to `decisions` receipts.
pub fn person_response(sim: &Sim, id: u64, decisions: u32) -> Result<Vec<u8>, QueryError> {
    let pid =
        PermanentId::from_raw(id).ok_or_else(|| QueryError("person 0 does not exist".into()))?;
    let pop = &sim.people;
    let record = pop
        .records
        .get(&pid)
        .ok_or_else(|| QueryError(format!("nobody with id {id} ever lived here")))?;
    let living = pop.person(pid);
    let now = sim.now();
    let mut fbb = FlatBufferBuilder::new();
    let name = fbb.create_string(&record.given);
    let (died_minute, cause) = match record.died {
        Some((at, cause)) => (
            at.minutes(),
            match cause {
                Cause::Unspecified => "illness or accident",
                Cause::Starvation => "hunger",
                Cause::Childbirth => "childbirth",
                Cause::Collapse => "a building's collapse",
            },
        ),
        None => (0, ""),
    };
    let cause = fbb.create_string(cause);
    let origin = fbb.create_string(match record.origin {
        Origin::Founder => "one of the founding band",
        Origin::Born => "born here",
        Origin::Spawned => "brought by the observer",
    });
    let age_at = record.died.map(|(at, _)| at).or(record.left).unwrap_or(now);
    let age =
        (age_at.minutes() - record.born.minutes()) as f64 / civ_core::time::MINUTES_PER_YEAR as f64;
    let kin: Vec<_> = kin(sim, pid)
        .into_iter()
        .map(|(k, relation)| {
            let name = fbb.create_string(&pop.name_of(k));
            let relation = fbb.create_string(relation);
            wire::KinLink::create(
                &mut fbb,
                &wire::KinLinkArgs {
                    id: k.get(),
                    name: Some(name),
                    relation: Some(relation),
                    alive: pop.person(k).is_some(),
                    left: pop.records.get(&k).is_some_and(|r| r.left.is_some()),
                },
            )
        })
        .collect();
    let kin = fbb.create_vector(&kin);

    let mut args = wire::PersonInfoArgs {
        id,
        name: Some(name),
        sex: sex(record.sex),
        born_minute: record.born.minutes(),
        age_years: age as f32,
        alive: living.is_some(),
        died_minute,
        cause: Some(cause),
        origin: Some(origin),
        kin: Some(kin),
        left_minute: record.left.map_or(0, |t| t.minutes()),
        ..Default::default()
    };
    let pos;
    if let Some(p) = living {
        let params = &sim.rules.people;
        let mut settled = p.clone();
        population::settle(&mut settled, now, params);
        let household = pop.household(p.household);
        let home = household.map_or(p.pos, |h| h.home);
        let settlement = household.and_then(|h| h.settlement);
        let settlement_name = settlement
            .and_then(|s| sim.land.settlements.iter().find(|x| x.id == s))
            .map(|s| fbb.create_string(&s.name));
        let doing = fbb.create_string(&doing(sim, p));
        let goods = &sim.rules.catalog.goods;
        let stores = household.map(|h| population::stores_now(h, now, params, goods));
        let (food_days, ready_days, water_days, fuel_days) = match (household, &stores) {
            (Some(h), Some(stores)) => {
                let members = h.members.len().max(1);
                let kcal_day = members as f64 * params.household.daily_kcal_per_person;
                let litres_day = members as f64 * params.household.water_l_per_person_day;
                let fuel_day = population::fuel_per_day(params, members, now.day_index());
                (
                    stock_kcal(stores, goods) / kcal_day.max(1.0),
                    food_kcal(stores, goods).0 / kcal_day.max(1.0),
                    h.water_at_time(now, litres_day) / litres_day.max(1e-6),
                    fuel_kg(stores, goods) / fuel_day.max(1e-6),
                )
            }
            _ => (0.0, 0.0, 0.0, 0.0),
        };
        let skills: Vec<wire::SkillLine> = p
            .skills
            .iter()
            .map(|&(k, level)| wire::SkillLine::new(k, level))
            .collect();
        let skills = fbb.create_vector(&skills);
        let knows = super::knowledge::know_lines(&mut fbb, sim, p);
        let lines: Vec<wire::StoreLine> = stores
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, kg)| **kg >= 0.05)
            .map(|(g, kg)| wire::StoreLine::new(g as u16, *kg as f32))
            .collect();
        let lines = fbb.create_vector(&lines);
        let take = decisions.min(MAX_DECISIONS_PER_QUERY) as usize;
        let receipts: Vec<_> = p
            .receipts
            .iter()
            .rev()
            .take(take)
            .map(|r| decision(&mut fbb, sim, (home, p.household), r))
            .collect();
        let receipts = fbb.create_vector(&receipts);
        let t = &p.traits;
        let traits = fbb.create_vector(&[
            t.openness,
            t.conscientiousness,
            t.extraversion,
            t.agreeableness,
            t.neuroticism,
            t.risk,
        ]);
        pos = vec2(p.position_at(now.minutes() as f64));
        args.household = p.household.get();
        args.settlement = settlement.map_or(0, PermanentId::get);
        args.settlement_name = settlement_name;
        args.activity = p.act.def;
        args.doing = Some(doing);
        args.since_minute = p.act.started.minutes();
        args.until_minute = p.act.step_ends.minutes();
        args.hunger = population::hunger(p, now, params) as f32;
        args.sleep_pressure = settled.sleep_pressure;
        args.loneliness = 1.0 - settled.relatedness;
        args.energy_kcal = settled.energy_kcal;
        let carried = p.carrying.good.and_then(|g| goods.get(usize::from(g)));
        args.carry_food_kcal = carried.map_or(0.0, |g| {
            if g.purpose == GoodUse::Food {
                (f64::from(p.carrying.kg) * g.kcal_per_kg) as f32
            } else {
                0.0
            }
        });
        args.carry_good = p.carrying.good.map_or(-1, i32::from);
        args.carry_kg = p.carrying.kg;
        args.carry_water_l = p.carrying.water_l;
        args.household_food_days = food_days as f32;
        args.household_ready_days = ready_days as f32;
        args.skills = Some(skills);
        args.knows = Some(knows);
        args.household_water_days = water_days as f32;
        args.household_fuel_days = fuel_days as f32;
        let notes: Vec<_> = family_notes(sim, p)
            .iter()
            .map(|n| fbb.create_string(n))
            .collect();
        let notes = fbb.create_vector(&notes);
        args.stores = Some(lines);
        args.decisions = Some(receipts);
        args.traits = Some(traits);
        args.pos = Some(&pos);
        args.partner = p.partner.map_or(0, PermanentId::get);
        args.family = Some(notes);
    }
    let body = wire::PersonInfo::create(&mut fbb, &args);
    Ok(response(fbb, wire::ResponseBody::PersonInfo, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_of_time_read_plainly() {
        let day = 1440;
        assert_eq!(how_long(21 * 365 * day + 100), "21 years");
        assert_eq!(how_long(400 * day), "a year");
        assert_eq!(how_long(100 * day), "3 months");
        assert_eq!(how_long(20 * day), "2 weeks");
        assert_eq!(how_long(3 * day), "a few days");
        assert_eq!(in_about(day), "any day now");
        assert_eq!(in_about(90 * day), "in about 3 months");
    }

    #[test]
    fn bearings_read_like_a_compass() {
        assert_eq!(bearing((0.0, 0.0), (600.0, 0.0)), "600 m east");
        assert_eq!(bearing((0.0, 0.0), (0.0, -42.0)), "40 m north");
        assert_eq!(bearing((0.0, 0.0), (-1500.0, 1500.0)), "2.1 km south-west");
        assert_eq!(bearing((10.0, 10.0), (11.0, 9.0)), "here");
    }
}
