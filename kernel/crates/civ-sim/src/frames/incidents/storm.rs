//! Storms and wind damage incidents for the observer (wire 1.69, M6b slice BF; ADR-0022 §13).

use civ_agents::floods::{StormDamage, StormEpisode};
use civ_agents::history::ChronicleEvent;
use civ_agents::polity::day_words;
use civ_core::{PermanentId, SimTime};
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use super::{
    CAUSED_BY, Link, PART_OF, Page, Row, STORM, STORM_DAMAGE, Step, TO_INFLUENCE, TO_STORM,
    building_words, put_lines, put_link, put_row, settlement_place,
};
use crate::Sim;
use crate::frames::response;

fn when(t: SimTime) -> String {
    day_words(t)
}

pub fn observer_of(_sim: &Sim, e: &StormEpisode) -> u32 {
    e.origin.observers.first().copied().unwrap_or(0)
}

pub fn storm_row<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    e: &StormEpisode,
) -> WIPOffset<wire::StormRow<'a>> {
    let span = match e.ended {
        Some(end) => format!("from {} to {}", when(e.began), when(end)),
        None => format!("from {}", when(e.began)),
    };
    let tail = if e.ended.is_some() { "over" } else { "it runs" };
    let words = format!(
        "Storm {}, {span}: peak wind speed {:.1} m/s; {} buildings damaged; {tail}",
        settlement_place(sim, e.settlement),
        e.peak_wind_speed_ms,
        e.damages.len()
    );
    let words = fbb.create_string(&words);
    wire::StormRow::create(
        fbb,
        &wire::StormRowArgs {
            id: e.id,
            settlement: e.settlement.map_or(0, PermanentId::get),
            began: e.began.minutes(),
            ended: e.ended.map_or(-1, |t| t.minutes()),
            peak_wind_speed_ms: e.peak_wind_speed_ms,
            buildings_damaged: e.damages.len() as u32,
            observer: observer_of(sim, e),
            words: Some(words),
        },
    )
}

pub fn storm_damage_row(sim: &Sim, sd: &StormDamage) -> Row {
    let b_name = building_words(sim, sd.building);
    Row {
        kind: STORM_DAMAGE,
        id: sd.id,
        minute: sd.time.minutes(),
        settlement: sd.settlement,
        at: sd.at,
        who: sd.building,
        who_name: Some(b_name),
        happened: format!(
            "covering torn under wind speed {:.1} m/s ({:.0} Pa uplift)",
            sd.wind_speed_ms, sd.uplift_pa
        ),
        known: "No sightings recorded".to_owned(),
        polity: "No gathering was told of it".to_owned(),
        observer: sd.origin.observers.first().copied().unwrap_or(0),
        outbreak: 0,
        conflagration: 0,
        flood: 0,
        storm: sd.episode.unwrap_or(0),
        quake: 0,
        open: false,
    }
}

pub fn storm_damage_page(sim: &Sim, sd: &StormDamage) -> Page {
    let row = storm_damage_row(sim, sd);
    let steps = vec![Step::at(
        "Damaged",
        sd.time.minutes(),
        format!("Wind uplift torn covering at {:.1} m/s", sd.wind_speed_ms),
    )];
    let mut links = Vec::new();
    if let Some(ep) = sd.episode {
        links.push(Link(
            PART_OF,
            TO_STORM,
            u64::from(ep),
            format!("part of storm episode {ep}"),
        ));
    }
    let truth = vec![format!(
        "Wind speed {:.1} m/s generated {:.0} Pa uplift exceeding covering capacity.",
        sd.wind_speed_ms, sd.uplift_pa
    )];
    let knowledge = vec!["The household inspected and cleared debris.".to_owned()];
    let polity = vec!["No collective action taken.".to_owned()];
    (row, steps, links, [truth, knowledge, polity])
}

pub fn storm_response(sim: &Sim, id: u32) -> Option<Vec<u8>> {
    let e = sim
        .people
        .floods
        .storm_episodes
        .iter()
        .find(|e| e.id == id)?;
    let mut fbb = FlatBufferBuilder::new();
    let row = storm_row(&mut fbb, sim, e);
    let obs = observer_of(sim, e);
    let (cause, cause_link) = if obs > 0 {
        (
            format!(
                "Affected by {} recorded observer input(s): {:?}",
                e.origin.observers.len(),
                e.origin.observers
            ),
            Some(Link(
                CAUSED_BY,
                TO_INFLUENCE,
                u64::from(obs),
                "sent by the observer".to_owned(),
            )),
        )
    } else {
        (
            if e.origin.known {
                "Natural storm".to_owned()
            } else {
                "Origin was not recorded in this snapshot".to_owned()
            },
            None,
        )
    };
    let cause_str = fbb.create_string(&cause);
    let cause_link = cause_link.map(|l| put_link(&mut fbb, &l));
    let weather = fbb.create_string(&format!("peak wind speed {:.1} m/s", e.peak_wind_speed_ms));
    let incidents: Vec<_> = sim
        .people
        .floods
        .storm_damages
        .iter()
        .filter(|sd| sd.episode == Some(e.id))
        .map(|sd| {
            let r = storm_damage_row(sim, sd);
            put_row(&mut fbb, sim, &r)
        })
        .collect();
    let incidents = fbb.create_vector(&incidents);
    let summary = vec![
        format!("Peak wind speed: {:.1} m/s", e.peak_wind_speed_ms),
        format!("Buildings damaged: {}", e.damages.len()),
    ];
    let summary = put_lines(&mut fbb, &summary);
    let body = wire::StormInfo::create(
        &mut fbb,
        &wire::StormInfoArgs {
            row: Some(row),
            cause: Some(cause_str),
            cause_link,
            weather: Some(weather),
            incidents: Some(incidents),
            summary: Some(summary),
        },
    );
    Some(response(fbb, wire::ResponseBody::StormInfo, body))
}

pub fn chronicle_link(sim: &Sim, e: &ChronicleEvent) -> Option<(u8, u32)> {
    sim.people
        .floods
        .storm_episodes
        .iter()
        .rev()
        .find(|ep| ep.began <= e.at && ep.ended.is_none_or(|end| end >= e.at))
        .map(|ep| (STORM, ep.id))
}
