//! Fires for the observer (wire 1.68, M6b slice BC, step three; ADR-0022 §13): each building's
//! fire as an incident in the one shape, its page with its stages and what it caught from, a page
//! for each conflagration with its fires in the order they caught and the buildings its heat
//! reached that held, how a building on the map stands to its latest fire, and the chronicle's
//! links to them. The knowledge layer reads only people's records of a fire (slice BD, step two):
//! who was in it, woke to it, saw it or heard the alarm, and whether anyone saw it start.

use civ_agents::fire::{Conflagration, Fire, Hearth, Kindled, Knew, Noticed, Stage};
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

/// The share of its strength building `building`'s bare steel keeps once burnt out, if any of its
/// members are of a material that softens rather than chars (ADR-0037 §5).
fn softens(sim: &Sim, building: PermanentId) -> Option<f64> {
    let catalog = &sim.rules.catalog;
    let b = sim.land.buildings.iter().find(|b| b.id == building)?;
    let def = catalog
        .building_index(&b.spec.program)
        .and_then(|d| catalog.buildings.get(d))?;
    let e = civ_grammar::expand(&b.spec, &def.rules).ok()?;
    e.groups
        .iter()
        .find_map(|g| civ_agents::structure::group_timber(&catalog.goods, def, g)?.burnt_keeps)
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
                Some(Hearth::Lamp) => "caught from a lamp lit at home",
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

/// People's records of fire `f`: who was in it, woke to it, saw it or heard the alarm of it,
/// oldest first.
fn knew_of<'a>(sim: &'a Sim, f: &Fire) -> Vec<&'a Knew> {
    let knew = &sim.people.fires.knew;
    let start = knew.partition_point(|k| k.at < f.began);
    knew[start..].iter().filter(|k| k.fire == f.id).collect()
}

/// What people know of a fire, in short: "Seen burning by 5; 2 were inside; 23 heard the alarm".
fn known_words(knew: &[&Knew]) -> String {
    let count = |how: Noticed| knew.iter().filter(|k| k.how == how).count();
    let (inside, woke, saw, heard) = (
        count(Noticed::Inside),
        count(Noticed::Woke),
        count(Noticed::Saw),
        count(Noticed::Heard),
    );
    let mut parts = Vec::new();
    if saw > 0 {
        parts.push(format!("Seen burning by {saw}"));
    }
    if inside > 0 {
        parts.push(format!(
            "{inside} {} inside",
            if inside == 1 { "was" } else { "were" }
        ));
    }
    if woke > 0 {
        parts.push(format!("{woke} woke to it inside"));
    }
    if heard > 0 {
        parts.push(format!("{heard} heard the alarm"));
    }
    if parts.is_empty() {
        return "Nobody saw it burn or heard of it".to_owned();
    }
    let mut s = parts.join("; ");
    if let Some(first) = s.get(..1) {
        s = format!("{}{}", first.to_uppercase(), &s[1..]);
    }
    s
}

/// The knowledge layer of fire `f`'s page: who knew first and how, whether anyone saw it start,
/// and how far the alarm went. Nobody knows its cause who did not see it start (ADR-0022 §7).
fn knowledge_lines(sim: &Sim, f: &Fire, summary: &str) -> Vec<String> {
    let knew = knew_of(sim, f);
    let mut lines = vec![summary.to_owned()];
    let first_hand: Vec<&&Knew> = knew.iter().filter(|k| k.how != Noticed::Heard).collect();
    if let Some(k) = first_hand.first() {
        let how = match k.how {
            Noticed::Inside => "was inside when it caught",
            Noticed::Woke => "woke to it inside",
            _ => "saw it burning",
        };
        lines.push(format!(
            "First to know: {}, who {how} at {}",
            sim.people.name_of(k.person),
            clock(k.at)
        ));
    }
    let at_start: Vec<String> = first_hand
        .iter()
        .filter(|k| k.at == f.began && k.how != Noticed::Woke)
        .map(|k| sim.people.name_of(k.person))
        .collect();
    if at_start.is_empty() {
        if !knew.is_empty() {
            lines.push(
                "No start observation is recorded: to those who know of it, it was found burning"
                    .to_owned(),
            );
        }
    } else {
        let what = "its roof catch";
        lines.push(format!("{} saw {what}", names_list(&at_start)));
    }
    lines.push(
        "No cause account is recorded; observing a roof catch does not identify its cause."
            .to_owned(),
    );
    if let Some(last) = knew.iter().rev().find(|k| k.how == Noticed::Heard) {
        let heard = knew.iter().filter(|k| k.how == Noticed::Heard).count();
        lines.push(format!(
            "The alarm reached {heard} {} by {}",
            if heard == 1 { "person" } else { "people" },
            clock(last.at)
        ));
    }
    let accounts: std::collections::BTreeSet<u32> = sim
        .people
        .word
        .claims
        .iter()
        .filter(|c| c.kind == civ_agents::word::ClaimKind::Fire && c.fire == Some(f.id))
        .map(|c| c.id)
        .collect();
    let holders: Vec<_> = sim
        .people
        .word
        .heard
        .iter()
        .filter(|h| accounts.contains(&h.claim))
        .collect();
    if !holders.is_empty() {
        let told = holders.iter().filter(|h| h.from.is_some()).count();
        lines.push(format!("{} people hold a dated found-burning account; {told} received it by contact. It names no cause.", holders.len()));
        if let Some(h) = holders
            .iter()
            .filter(|h| h.from.is_some())
            .min_by_key(|h| h.first)
        {
            lines.push(format!(
                "First recorded telling: {} heard it from {} on {}.",
                sim.people.name_of(h.holder),
                h.from
                    .map_or_else(|| "someone".to_owned(), |p| sim.people.name_of(p)),
                day_words(civ_core::SimTime::from_minutes(h.first * 1440))
            ));
        }
    }
    lines
}

/// "Ada", "Ada and Bo", "Ada, Bo and 3 others".
fn names_list(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [a] => a.clone(),
        [a, b] => format!("{a} and {b}"),
        [a, b, rest @ ..] => {
            let n = rest.len();
            format!(
                "{a}, {b} and {n} {}",
                if n == 1 { "other" } else { "others" }
            )
        }
    }
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
        known: known_words(&knew_of(sim, f)),
        polity: "No gathering was told of it".to_owned(),
        observer: observer_of(sim, f),
        outbreak: 0,
        conflagration: f.episode.unwrap_or(0),
        flood: 0,
        storm: 0,
        quake: 0,
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
            let litres: f32 = fires
                .thrown
                .iter()
                .filter(|t| t.fire == root && t.building == w.building)
                .map(|t| t.litres)
                .sum();
            let water = if litres > 0.0 {
                format!("; {litres:.0} L of water were thrown on it")
            } else {
                String::new()
            };
            if w.threshold_kw.is_finite() {
                format!(
                    "{name}: the heat on it rose to {:.1} kW/m² against the {:.1} its covering \
                     needed{water}",
                    w.peak_kw, w.threshold_kw
                )
            } else {
                format!(
                    "{name}: the heat on it rose to {:.1} kW/m², but its covering could not catch \
                     (too wet, or none that burns){water}",
                    w.peak_kw
                )
            }
        })
        .collect()
}

/// The water people carried against the fires begun with `root` (M6b slice BD, step three):
/// "Water was carried against it: 16 loads, 240 L, by 9 people, 6 of the loads from their
/// households' stores", or nothing when none was.
fn water_line(sim: &Sim, root: u32) -> Vec<String> {
    let fires = &sim.people.fires;
    let mut out: Vec<String> = Vec::new();
    out.extend(thrown_line(sim, root));
    // Roofs stripped and goods carried out (M6b slice BD, step four).
    let stripped: Vec<_> = fires.stripped.iter().filter(|t| t.fire == root).collect();
    if !stripped.is_empty() {
        let mut roofs: Vec<PermanentId> = stripped.iter().map(|t| t.building).collect();
        roofs.sort_unstable();
        roofs.dedup();
        let m2: f32 = stripped.iter().map(|t| t.m2).sum();
        out.push(format!(
            "Covering was torn from {} {} to save {}: {m2:.0} m²",
            roofs.len(),
            if roofs.len() == 1 { "roof" } else { "roofs" },
            if roofs.len() == 1 { "it" } else { "them" },
        ));
    }
    for p in fires.pulled.iter().filter(|p| p.fire == root) {
        let name = building_words(sim, p.building);
        let people: Vec<String> = p.people.iter().map(|&id| sim.people.name_of(id)).collect();
        out.push(format!(
            "{}: {name} was pulled down by {}; {:.2} person-hours of work, its parts lost",
            when(p.at),
            names_list(&people),
            p.work_h,
        ));
    }
    let salvaged: Vec<_> = fires.salvaged.iter().filter(|t| t.fire == root).collect();
    if !salvaged.is_empty() {
        let kg: f32 = salvaged.iter().map(|t| t.kg).sum();
        let mut people: Vec<PermanentId> = salvaged.iter().map(|t| t.person).collect();
        people.sort_unstable();
        people.dedup();
        out.push(format!(
            "Goods were carried out of its way: {kg:.0} kg, by {} {}",
            people.len(),
            if people.len() == 1 {
                "person"
            } else {
                "people"
            },
        ));
    }
    out
}

/// The water people carried against the fires begun with `root`, in a line, if any was.
fn thrown_line(sim: &Sim, root: u32) -> Option<String> {
    let thrown: Vec<_> = sim
        .people
        .fires
        .thrown
        .iter()
        .filter(|t| t.fire == root)
        .collect();
    if thrown.is_empty() {
        return None;
    }
    let litres: f32 = thrown.iter().map(|t| t.litres).sum();
    let mut people: Vec<PermanentId> = thrown.iter().map(|t| t.person).collect();
    people.sort_unstable();
    people.dedup();
    let stores = thrown
        .iter()
        .filter(|t| t.from == civ_agents::person::FROM_STORE)
        .count();
    let loads = thrown.len();
    Some(format!(
        "Water was carried against it: {loads} {}, {litres:.0} L, by {} {}, {stores} of the loads \
         from their households' stores",
        if loads == 1 { "load" } else { "loads" },
        people.len(),
        if people.len() == 1 {
            "person"
        } else {
            "people"
        },
    ))
}

/// A fire's page: its stages, what started it, the conflagration it is part of, and its three
/// layers.
fn response_steps(sim: &Sim, f: &Fire) -> Vec<Step> {
    let fires = &sim.people.fires;
    let root = fires.root(f.id);
    let relevant = |fire: u32, building: PermanentId| {
        fire == root && (building == f.building || (f.id == root && f.episode.is_none()))
    };
    let mut steps = Vec::new();
    for t in &fires.thrown {
        if relevant(t.fire, t.building) {
            steps.push(Step::at("Water delivered", t.at.minutes(), format!(
                "{} threw {:.1} L on {} to wet its covering against the fire; water was obtained before delivery",
                sim.people.name_of(t.person), t.litres, building_words(sim, t.building))));
        }
    }
    for t in &fires.stripped {
        if relevant(t.fire, t.building) {
            steps.push(Step::at("Covering stripped", t.at.minutes(), format!(
                "{} removed {:.1} m² from {} to remove fuel exposed to the fire; the covering was lost",
                sim.people.name_of(t.person), t.m2, building_words(sim, t.building))));
        }
    }
    for t in &fires.salvaged {
        if relevant(t.fire, t.building) {
            let good = sim
                .rules
                .catalog
                .goods
                .get(usize::from(t.good))
                .map_or("goods", |g| g.name.as_str());
            steps.push(Step::at(
                "Goods carried out",
                t.at.minutes(),
                format!(
                    "{} moved {:.1} kg of {good} out from {} and away from the fire",
                    sim.people.name_of(t.person),
                    t.kg,
                    building_words(sim, t.building)
                ),
            ));
        }
    }
    for t in &fires.pulled {
        if relevant(t.fire, t.building) {
            steps.push(Step::at("Building pulled down", t.at.minutes(), format!(
                "A crew of {} pulled down {} to break the fire's path; {:.2} person-hours, all parts lost",
                t.people.len(), building_words(sim, t.building), t.work_h)));
        }
    }
    steps.sort_by_key(|s| s.at);
    // Full quantities remain in the summary; a long bucket chain does not swamp the page.
    steps.truncate(30);
    steps
}

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
            Step::at("Fully involved", t.minutes(), format!(
                "The covering fire developed into the building's full burn after {:.0} minutes; {goods}", p.involve_minutes))
        }
        None => Step::not_reached("Fully involved", "not yet"),
    });
    steps.push(match f.ended {
        Some(t) => Step::at(
            "Burnt out",
            t.minutes(),
            "The building's burning stages finished; consumed parts and goods remain lost"
                .to_owned(),
        ),
        None => Step::not_reached("Burnt out", "it burns"),
    });
    if let Kindled::Ignition { ignition } = f.kindled
        && let Some(g) = ignition
            .checked_sub(1)
            .and_then(|i| fires.ignitions.get(i as usize))
    {
        steps.push(Step::at("Household fire escaped", g.at.minutes(), format!(
            "A fire in use escaped with nobody awake beside it to put it out; it grew for {:.0} minutes before reaching the roof", g.grows_min)));
    }
    if let Some(k) = fires
        .knew
        .iter()
        .find(|k| k.fire == f.id && k.how != Noticed::Heard)
    {
        steps.push(Step::at(
            "First recorded notice",
            k.at.minutes(),
            format!(
                "{} {}",
                sim.people.name_of(k.person),
                match k.how {
                    Noticed::Inside => "was awake inside",
                    Noticed::Woke => "woke to the fire inside",
                    Noticed::Saw => "saw it burning",
                    Noticed::Heard => "heard an alarm",
                }
            ),
        ));
    }
    if let Some(k) = fires
        .knew
        .iter()
        .find(|k| k.fire == f.id && k.how == Noticed::Heard)
    {
        steps.push(Step::at(
            "Alarm first heard",
            k.at.minutes(),
            format!(
                "{} heard a nearby shout; hearing the alarm alone names no cause",
                sim.people.name_of(k.person)
            ),
        ));
    }
    steps.extend(response_steps(sim, f));
    steps.sort_by_key(|s| s.at.map_or(i64::MAX, |(minute, _)| minute));
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
    // A steel frame softens where timber chars (ADR-0037 §5).
    let steel = softens(sim, f.building);
    if charred > 0.0 {
        let what = if steel.is_some() {
            "Its timber"
        } else {
            "Its posts and frame"
        };
        truth.push(format!(
            "{what} charred about {charred:.0} mm deep on each exposed face"
        ));
    }
    if f.ended.is_some() {
        truth.push(format!(
            "Its covering burned away, and its walls lost {:.0}% of their infill",
            p.infill_loss * 100.0
        ));
        if let Some(keeps) = steel {
            truth.push(format!(
                "Its steel softened in the fire to about {:.0}% of its strength; what stood \
                 regained it as it cooled",
                keeps * 100.0
            ));
        }
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
        truth.extend(water_line(sim, f.id));
    }
    let knowledge = knowledge_lines(sim, f, &row.known);
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
    let mut held = held_lines(sim, c.first);
    held.extend(water_line(sim, c.first));
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
            if step == FireStep::Pulled as u8 {
                fires
                    .pulled
                    .iter()
                    .rev()
                    .find(|p| {
                        p.at == e.at
                            && sim.land.buildings.iter().any(|b| {
                                b.id == p.building
                                    && Some(civ_agents::build::centre_m(&b.spec)) == e.place
                            })
                    })
                    .map(|p| (FIRE, p.fire))
            } else if step == FireStep::Burned as u8 {
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
            } else if step == FireStep::Ended as u8 {
                fires
                    .episodes
                    .iter()
                    .rev()
                    .find(|c| c.ended == Some(e.at) && first_at(c) == e.place)
                    .map(|c| (CONFLAGRATION, c.id))
            } else {
                None
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
