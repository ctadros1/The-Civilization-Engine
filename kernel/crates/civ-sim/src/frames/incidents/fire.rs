//! Fires for the observer (wire 1.68, M6b slice BC, step three; ADR-0022 §13): each building's
//! fire as an incident in the one shape, its page with its stages and what it caught from, a page
//! for each conflagration with its fires in the order they caught and the buildings its heat
//! reached that held, how a building on the map stands to its latest fire, and the chronicle's
//! links to them. Only the kernel's records are read: nobody in the world sees or tells of a fire
//! until slice BD, and the knowledge layer says so.

use civ_agents::fire::{Conflagration, Fire, Hearth, Kindled, Stage};
use civ_agents::history::{Cause, ChronicleEvent, ChronicleKind, FireStep};
use civ_agents::influence::InfluenceKind;
use civ_agents::polity::day_words;
use civ_core::{PermanentId, SimTime};
use civ_grammar::GroupKind;
use civ_land::{Building, GroupState};
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use super::{
    CAUSED_BY, CONFLAGRATION, FIRE, Link, PART_OF, Page, Row, Step, TO_CONFLAGRATION, TO_FIRE,
    TO_HOUSEHOLD, TO_INFLUENCE, capitalised, put_lines, put_link, put_row, settlement_place,
};
use crate::Sim;
use crate::frames::response;
use crate::frames::weather::wind_strength;

/// Buildings a page lists as reached and held, at most.
const MOST_HELD: usize = 30;
/// Heat below which a building is not counted as reached, kW/m².
const REACHED_KW: f32 = 0.1;

/// "02:14", a moment's time of day.
fn clock(t: SimTime) -> String {
    let m = t.minute_of_day();
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// "02:14 on 3 May of year 2".
fn when(t: SimTime) -> String {
    format!("{} on {}", clock(t), day_words(t))
}

/// Building `building` of household `household` in words: "the hut of Wren's household", or "a
/// building of Wren's household" once it is gone.
fn building_name(sim: &Sim, building: PermanentId, household: PermanentId) -> String {
    let catalog = &sim.rules.catalog;
    let what = sim
        .land
        .buildings
        .iter()
        .find(|b| b.id == building)
        .and_then(|b| catalog.building_index(&b.spec.program))
        .and_then(|d| catalog.buildings.get(d))
        .map(|d| d.name.to_lowercase());
    let of = crate::frames::people::household_name(sim, household);
    match what {
        Some(what) => format!("the {what} of {of}"),
        None => format!("a building of {of}"),
    }
}

/// Building `building` in words, as the observer's tools name it: "the hut of Wren's household";
/// "a building" when there is none.
pub fn building_words(sim: &Sim, building: PermanentId) -> String {
    sim.land
        .buildings
        .iter()
        .find(|b| b.id == building)
        .map_or_else(
            || "a building".to_owned(),
            |b| building_name(sim, b.id, b.household),
        )
}

/// How fire `f` began, in words: "set alight by the observer (one recorded influence)", "caught
/// from the hut of Bo's household 2.0 m away, downwind of it in a gentle breeze, its covering
/// holding water 12% of its dry weight".
fn cause_words(sim: &Sim, f: &Fire) -> String {
    match f.kindled {
        Kindled::Observer { .. } => {
            "set alight by the observer (one recorded influence)".to_owned()
        }
        Kindled::Neighbour {
            fire,
            gap_m,
            wind_ms,
            toward,
            moisture,
        } => {
            let source = sim.people.fires.get(fire).map_or_else(
                || "another fire".to_owned(),
                |s| building_name(sim, s.building, s.household),
            );
            let wind = if wind_ms < 0.5 {
                "in still air".to_owned()
            } else {
                let lie = match toward {
                    t if t > 0.5 => "downwind of it",
                    t if t < -0.5 => "upwind of it",
                    _ => "across the wind from it",
                };
                format!("{lie} in {}", wind_strength(f64::from(wind_ms)))
            };
            format!(
                "caught from {source} {gap_m:.1} m away, {wind}, its covering holding water {:.0}% \
                 of its dry weight",
                f64::from(moisture) * 100.0
            )
        }
        Kindled::Ignition { ignition } => {
            let g = ignition
                .checked_sub(1)
                .and_then(|i| sim.people.fires.ignitions.get(i as usize));
            let from = match g.map(|g| g.hearth) {
                Some(Hearth::Banked) => {
                    "caught from its banked hearth, with none of its people awake at home"
                }
                Some(Hearth::Lit) => "caught from its hearth",
                Some(Hearth::Worked) => "caught from a fire worked at home",
                None => "caught from a fire in use",
            };
            match g {
                Some(g) => format!(
                    "{from}, its covering holding water {:.0}% of its dry weight",
                    f64::from(g.moisture) * 100.0
                ),
                None => from.to_owned(),
            }
        }
    }
}

/// How fire `f` began, as a link to what started it.
fn cause_link(sim: &Sim, f: &Fire) -> Link {
    match f.kindled {
        Kindled::Observer { influence } => Link(
            CAUSED_BY,
            TO_INFLUENCE,
            u64::from(influence),
            cause_words(sim, f),
        ),
        Kindled::Neighbour { fire, .. } => {
            Link(CAUSED_BY, TO_FIRE, u64::from(fire), cause_words(sim, f))
        }
        Kindled::Ignition { .. } => Link(
            CAUSED_BY,
            TO_HOUSEHOLD,
            f.household.get(),
            cause_words(sim, f),
        ),
    }
}

/// The observer's influence that set fire `f`, or the first fire of its conflagration (0: none).
pub(super) fn observer_of(sim: &Sim, f: &Fire) -> u32 {
    let fires = &sim.people.fires;
    let first = fires.get(fires.root(f.id)).unwrap_or(f);
    match first.kindled {
        Kindled::Observer { influence } => influence,
        Kindled::Neighbour { .. } | Kindled::Ignition { .. } => 0,
    }
}

/// What a fire took, in words: "; Ada and Bram died; 120 kg of goods burned"; empty for nothing.
fn losses(sim: &Sim, deaths: &[PermanentId], goods_kg: f32) -> String {
    let mut s = String::new();
    if !deaths.is_empty() {
        let names: Vec<String> = deaths.iter().map(|&d| sim.people.name_of(d)).collect();
        let list = match names.len() {
            1 => names[0].clone(),
            n => format!("{} and {}", names[..n - 1].join(", "), names[n - 1]),
        };
        s.push_str(&format!("; {list} died"));
    }
    if goods_kg >= 1.0 {
        s.push_str(&format!("; {goods_kg:.0} kg of goods burned"));
    }
    s
}

/// When fire `f` was fully involved, if it was by now.
fn involved_at(sim: &Sim, f: &Fire) -> Option<SimTime> {
    let p = &sim.rules.people.fire;
    let at = f
        .began
        .plus_minutes(p.involve_minutes.max(0.0).ceil() as i64);
    (at <= f.ended.unwrap_or_else(|| sim.now())).then_some(at)
}

/// A fire as an incident row: its building's name, then what happened.
pub(super) fn fire_row(sim: &Sim, f: &Fire) -> Row {
    let name = building_name(sim, f.building, f.household);
    let end = match f.ended {
        Some(t) => format!("; it burnt out at {}", clock(t)),
        None => "; it burns".to_owned(),
    };
    Row {
        kind: FIRE,
        id: f.id,
        minute: f.began.minutes(),
        settlement: f.settlement,
        at: f.at,
        who: f.building,
        who_name: Some(capitalised(&name)),
        happened: format!(
            "burned from {}, {}{}{end}",
            when(f.began),
            cause_words(sim, f),
            losses(sim, &f.deaths, f.goods_kg)
        ),
        known: "Who saw it or heard of it is not yet recorded".to_owned(),
        polity: "No gathering was told of it".to_owned(),
        observer: observer_of(sim, f),
        outbreak: 0,
        conflagration: f.episode.unwrap_or(0),
        open: f.burning(),
    }
}

/// The buildings the heat of the fires begun with `root` reached and that held, nearest (most
/// heated) first: "The hut of Bo's household: the heat on it rose to 7.1 kW/m² against the 10.9
/// its covering needed".
fn held_lines(sim: &Sim, root: u32) -> Vec<String> {
    let fires = &sim.people.fires;
    let mut held: Vec<_> = fires
        .weighed
        .iter()
        .filter(|w| w.fire == root && !w.caught && w.peak_kw >= REACHED_KW)
        .collect();
    held.sort_by(|a, b| b.peak_kw.total_cmp(&a.peak_kw));
    held.iter()
        .take(MOST_HELD)
        .map(|w| {
            let household = sim
                .land
                .buildings
                .iter()
                .find(|b| b.id == w.building)
                .map_or(w.building, |b| b.household);
            let name = capitalised(&building_name(sim, w.building, household));
            if w.threshold_kw.is_finite() {
                format!(
                    "{name}: the heat on it rose to {:.1} kW/m² against the {:.1} its covering \
                     needed",
                    w.peak_kw, w.threshold_kw
                )
            } else {
                format!(
                    "{name}: the heat on it rose to {:.1} kW/m², but its covering could not catch \
                     (too wet, or none that burns)",
                    w.peak_kw
                )
            }
        })
        .collect()
}

/// A fire's page: its stages, what started it, the conflagration it is part of, and its three
/// layers.
pub(super) fn fire_page(sim: &Sim, f: &Fire) -> Page {
    let fires = &sim.people.fires;
    let p = &sim.rules.people.fire;
    let row = fire_row(sim, f);
    let cause = cause_link(sim, f);
    let mut steps = vec![Step::at("Caught", f.began.minutes(), cause.3.clone())];
    steps.push(match involved_at(sim, f) {
        Some(t) => {
            let goods = if f.goods_kg >= 1.0 {
                format!("{:.0} kg of goods under its roof burned", f.goods_kg)
            } else {
                "nothing that burns lay under its roof".to_owned()
            };
            Step::at("Fully involved", t.minutes(), goods)
        }
        None => Step::not_reached("Fully involved", "not yet"),
    });
    steps.push(match f.ended {
        Some(t) => Step::at("Burnt out", t.minutes(), String::new()),
        None => Step::not_reached("Burnt out", "it burns"),
    });
    let mut links = vec![cause];
    if let Some(c) = f.episode.and_then(|e| fires.episodes.get(e as usize - 1)) {
        let first = if c.first == f.id {
            ", its first fire"
        } else {
            ""
        };
        links.push(Link(
            PART_OF,
            TO_CONFLAGRATION,
            u64::from(c.id),
            format!(
                "part of the fire {} that burned {}{first}",
                settlement_place(sim, c.settlement),
                plural_buildings(
                    fires
                        .list
                        .iter()
                        .filter(|x| x.episode == Some(c.id))
                        .count()
                )
            ),
        ));
    }
    // What it did to the building, from the stages it reached.
    let involved_min = |end: SimTime| {
        ((end.minutes() - f.began.minutes()) as f64 - p.involve_minutes)
            .clamp(0.0, p.involved_minutes)
    };
    let charred = p.char_mm(involved_min(f.ended.unwrap_or_else(|| sim.now())));
    let mut truth = vec![format!(
        "{} {}",
        row.who_name.clone().unwrap_or_default(),
        row.happened
    )];
    if charred > 0.0 {
        truth.push(format!(
            "Its posts and frame charred about {charred:.0} mm deep on each exposed face"
        ));
    }
    if f.ended.is_some() {
        truth.push(format!(
            "Its covering burned away, and its walls lost {:.0}% of their infill",
            p.infill_loss * 100.0
        ));
    }
    if !f.deaths.is_empty() {
        let names: Vec<String> = f.deaths.iter().map(|&d| sim.people.name_of(d)).collect();
        truth.push(format!(
            "Caught asleep inside and did not get out: {}",
            names.join(", ")
        ));
    }
    // A lone fire's page tells what its heat reached; a conflagration's page tells it for all.
    if f.episode.is_none() {
        let held = held_lines(sim, f.id);
        if held.is_empty() {
            truth.push("Its heat caught no other building, and reached none".to_owned());
        } else {
            truth.push("Its heat reached these, and they held:".to_owned());
            truth.extend(held);
        }
    }
    let knowledge = vec![row.known.clone()];
    let polity = vec![row.polity.clone()];
    (row, steps, links, [truth, knowledge, polity])
}

/// "1 building", "7 buildings".
fn plural_buildings(n: usize) -> String {
    if n == 1 {
        "1 building".to_owned()
    } else {
        format!("{n} buildings")
    }
}

/// A conflagration's fires, the first first.
fn fires_of<'a>(sim: &'a Sim, c: &Conflagration) -> Vec<&'a Fire> {
    let mut fires: Vec<&Fire> = sim
        .people
        .fires
        .list
        .iter()
        .filter(|f| f.episode == Some(c.id))
        .collect();
    fires.sort_by_key(|f| (f.began, f.id));
    fires
}

/// A conflagration as a row on the wire.
pub(super) fn conflagration_row<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    c: &Conflagration,
) -> WIPOffset<wire::ConflagrationRow<'a>> {
    let fires = fires_of(sim, c);
    let deaths: Vec<PermanentId> = fires
        .iter()
        .flat_map(|f| f.deaths.iter().copied())
        .collect();
    let goods: f32 = fires.iter().map(|f| f.goods_kg).sum();
    let place = settlement_place(sim, c.settlement);
    let words = match c.ended {
        Some(end) => format!(
            "A fire {place} from {} to {}: it burned {}{}",
            when(c.began),
            when(end),
            plural_buildings(fires.len()),
            losses(sim, &deaths, goods)
        ),
        None => format!(
            "A fire {place} since {}: {} caught so far{}; it burns",
            when(c.began),
            plural_buildings(fires.len()),
            losses(sim, &deaths, goods)
        ),
    };
    let observer = sim
        .people
        .fires
        .get(c.first)
        .map_or(0, |f| observer_of(sim, f));
    let words = fbb.create_string(&words);
    wire::ConflagrationRow::create(
        fbb,
        &wire::ConflagrationRowArgs {
            id: c.id,
            settlement: c.settlement.map_or(0, PermanentId::get),
            began: c.began.minutes(),
            ended: c.ended.map_or(-1, SimTime::minutes),
            burned: fires.len() as u32,
            died: deaths.len() as u32,
            goods_kg: goods,
            observer,
            words: Some(words),
        },
    )
}

/// A `Response` with conflagration `id`'s page, or `None` when there is no such conflagration.
pub fn conflagration_response(sim: &Sim, id: u32) -> Option<Vec<u8>> {
    let fires = &sim.people.fires;
    let c = fires.episodes.iter().find(|c| c.id == id)?;
    let list = fires_of(sim, c);
    let first = fires.get(c.first);
    let (cause, link) = match first {
        Some(f) => (
            format!(
                "Began at {}, {}",
                building_name(sim, f.building, f.household),
                cause_words(sim, f)
            ),
            Some(cause_link(sim, f)),
        ),
        None => ("Its first fire is not recorded".to_owned(), None),
    };
    // The day's wind and dryness when it first spread, as the catch recorded them.
    let weather = list
        .iter()
        .find_map(|f| match f.kindled {
            Kindled::Neighbour {
                wind_ms, moisture, ..
            } => Some(format!(
                "When it first spread: {}, coverings holding water {:.0}% of their dry weight",
                if wind_ms < 0.5 {
                    "still air".to_owned()
                } else {
                    format!("{} at {wind_ms:.1} m/s", wind_strength(f64::from(wind_ms)))
                },
                f64::from(moisture) * 100.0
            )),
            Kindled::Observer { .. } | Kindled::Ignition { .. } => None,
        })
        .unwrap_or_default();
    let held = held_lines(sim, c.first);
    let mut fbb = FlatBufferBuilder::new();
    let row = conflagration_row(&mut fbb, sim, c);
    let cause = fbb.create_string(&cause);
    let cause_link = link.map(|l| put_link(&mut fbb, &l));
    let weather = fbb.create_string(&weather);
    let rows: Vec<_> = list
        .iter()
        .map(|f| {
            let row = fire_row(sim, f);
            put_row(&mut fbb, sim, &row)
        })
        .collect();
    let rows = fbb.create_vector(&rows);
    let held = put_lines(&mut fbb, &held);
    let body = wire::ConflagrationInfo::create(
        &mut fbb,
        &wire::ConflagrationInfoArgs {
            row: Some(row),
            cause: Some(cause),
            cause_link,
            weather: Some(weather),
            fires: Some(rows),
            held: Some(held),
        },
    );
    Some(response(fbb, wire::ResponseBody::ConflagrationInfo, body))
}

/// How building `b` stands to its latest fire (wire 1.68): 0 none (or its covering since mended),
/// 1 its covering alight, 2 fully involved, 3 burnt out; that fire's number; and in words.
pub fn building_fire(sim: &Sim, b: &Building) -> (u8, u32, String) {
    let fires = &sim.people.fires;
    let Some(f) = fires.list.iter().rev().find(|f| f.building == b.id) else {
        return (0, 0, String::new());
    };
    if f.burning() {
        let p = &sim.rules.people.fire;
        let stage = p.stage_at(f.minutes_at(sim.now()));
        let cause = cause_words(sim, f);
        return match (stage, involved_at(sim, f)) {
            (Stage::Covering, _) | (_, None) => (
                1,
                f.id,
                format!("its roof alight since {}, {cause}", clock(f.began)),
            ),
            (_, Some(t)) => (
                2,
                f.id,
                format!("fully involved since {}, {cause}", clock(t)),
            ),
        };
    }
    let bare = b.condition.iter().any(|c| {
        GroupKind::of_group(c.group) == Some(GroupKind::Covering)
            && (c.loss >= 1.0 || c.state == GroupState::Failed)
    });
    match f.ended {
        Some(end) if bare => (3, f.id, format!("burnt out at {}", when(end))),
        _ => (0, 0, String::new()),
    }
}

/// The fire or conflagration a chronicle entry tells of (wire 1.68), found in the records it was
/// written from: a lone fire burnt out by its building's place and minute; a conflagration's
/// spread by its first fire's place while it burned, and its end by that place and minute; a
/// death by fire by the fire that killed them; the observer's setting one by its place and minute.
pub(super) fn chronicle_link(sim: &Sim, e: &ChronicleEvent) -> Option<(u8, u32)> {
    let fires = &sim.people.fires;
    let first_at = |c: &Conflagration| fires.get(c.first).map(|f| f.at);
    match e.kind {
        ChronicleKind::Fire => {
            let step = e.number.round() as u8;
            if step == FireStep::Burned as u8 {
                fires
                    .list
                    .iter()
                    .rev()
                    .find(|f| f.ended == Some(e.at) && f.episode.is_none() && Some(f.at) == e.place)
                    .map(|f| (FIRE, f.id))
            } else if step == FireStep::Spread as u8 {
                fires
                    .episodes
                    .iter()
                    .rev()
                    .find(|c| {
                        c.began <= e.at
                            && c.ended.is_none_or(|end| end >= e.at)
                            && first_at(c) == e.place
                    })
                    .map(|c| (CONFLAGRATION, c.id))
            } else {
                fires
                    .episodes
                    .iter()
                    .rev()
                    .find(|c| c.ended == Some(e.at) && first_at(c) == e.place)
                    .map(|c| (CONFLAGRATION, c.id))
            }
        }
        ChronicleKind::Died if e.name == Cause::Burned.key() => {
            let who = e.people.first()?;
            fires
                .list
                .iter()
                .rev()
                .find(|f| f.deaths.contains(who))
                .map(|f| (FIRE, f.id))
        }
        ChronicleKind::Influence if e.number.round() as u8 == InfluenceKind::Fire.code() => fires
            .list
            .iter()
            .rev()
            .find(|f| {
                matches!(f.kindled, Kindled::Observer { .. })
                    && f.began == e.at
                    && Some(f.at) == e.place
            })
            .map(|f| (FIRE, f.id)),
        _ => None,
    }
}
