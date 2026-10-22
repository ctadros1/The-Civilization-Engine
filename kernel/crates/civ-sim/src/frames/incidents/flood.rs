//! Floods and high-water incidents for the observer (wire 1.69, M6b slice BF; ADR-0022 §13).

use civ_agents::floods::{FloodEpisode, HighWater};
use civ_agents::history::ChronicleEvent;
use civ_agents::polity::day_words;
use civ_core::{PermanentId, SimTime};
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use super::{
    CAUSED_BY, FLOOD, HIGH_WATER, Link, PART_OF, Page, Row, Step, TO_FLOOD, TO_INFLUENCE,
    put_lines, put_link, put_row, settlement_place,
};
use crate::Sim;
use crate::frames::response;

fn when(t: SimTime) -> String {
    day_words(t)
}

pub fn observer_of(_sim: &Sim, e: &FloodEpisode) -> u32 {
    e.origin.observers.first().copied().unwrap_or(0)
}

pub fn flood_row<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    e: &FloodEpisode,
) -> WIPOffset<wire::FloodRow<'a>> {
    let span = match e.ended {
        Some(end) => format!("from {} to {}", when(e.began), when(end)),
        None => format!("from {}", when(e.began)),
    };
    let tail = if e.ended.is_some() { "over" } else { "it runs" };
    let words = format!(
        "Flood on reach {} {}, {span}: peak discharge {:.1} m³/s; {} buildings flooded, {} fields lost, {:.1} kg goods spoiled; {tail}",
        e.first_reach,
        settlement_place(sim, e.settlement),
        e.peak_discharge,
        e.total_buildings,
        e.total_fields_lost,
        e.total_spoiled_kg
    );
    let words = fbb.create_string(&words);
    wire::FloodRow::create(
        fbb,
        &wire::FloodRowArgs {
            id: e.id,
            reach: u32::from(e.first_reach),
            settlement: e.settlement.map_or(0, PermanentId::get),
            began: e.began.minutes(),
            ended: e.ended.map_or(-1, |t| t.minutes()),
            peak_discharge_m3s: e.peak_discharge,
            peak_depth_m: 0.0,
            buildings_flooded: e.total_buildings,
            fields_submerged: e.total_fields_lost,
            spoiled_kg: e.total_spoiled_kg,
            observer: observer_of(sim, e),
            words: Some(words),
        },
    )
}

pub fn high_water_row(_sim: &Sim, hw: &HighWater) -> Row {
    Row {
        kind: HIGH_WATER,
        id: hw.id,
        minute: hw.began.minutes(),
        settlement: hw.settlement,
        at: hw.at,
        who: hw
            .settlement
            .unwrap_or_else(|| PermanentId::from_raw(1).expect("id")),
        who_name: Some(format!("Reach {}", hw.reach)),
        happened: format!(
            "rose to peak discharge {:.1} m³/s (stage {:.2} m); {} buildings flooded, {:.1} kg goods spoiled",
            hw.peak_discharge, hw.peak_stage, hw.buildings_flooded, hw.spoiled_kg
        ),
        known: "No sightings recorded".to_owned(),
        polity: "No gathering was told of it".to_owned(),
        observer: hw.origin.observers.first().copied().unwrap_or(0),
        outbreak: 0,
        conflagration: 0,
        flood: hw.episode.unwrap_or(0),
        storm: 0,
        quake: 0,
        open: hw.ended.is_none(),
    }
}

pub fn high_water_page(sim: &Sim, hw: &HighWater) -> Page {
    let row = high_water_row(sim, hw);
    let mut steps = vec![Step::at(
        "Began",
        hw.began.minutes(),
        format!("Flow exceeded bankfull capacity on reach {}", hw.reach),
    )];
    if let Some(ended) = hw.ended {
        steps.push(Step::at(
            "Ended",
            ended.minutes(),
            "Water receded below bankfull".to_owned(),
        ));
    }
    let mut links = Vec::new();
    if let Some(ep) = hw.episode {
        links.push(Link(
            PART_OF,
            TO_FLOOD,
            u64::from(ep),
            format!("part of flood episode {ep}"),
        ));
    }
    let truth = vec![
        format!(
            "Peak discharge reached {:.1} m³/s with stage {:.2} m above channel bed.",
            hw.peak_discharge, hw.peak_stage
        ),
        format!(
            "{} buildings flooded; {:.1} kg stored goods spoiled on floors.",
            hw.buildings_flooded, hw.spoiled_kg
        ),
    ];
    let knowledge = vec!["No sightings or reports are linked to this incident.".to_owned()];
    let polity = vec!["No collective decision recorded.".to_owned()];
    (row, steps, links, [truth, knowledge, polity])
}

pub fn flood_response(sim: &Sim, id: u32) -> Option<Vec<u8>> {
    let e = sim
        .people
        .floods
        .flood_episodes
        .iter()
        .find(|e| e.id == id)?;
    let mut fbb = FlatBufferBuilder::new();
    let row = flood_row(&mut fbb, sim, e);
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
                "brought by the observer".to_owned(),
            )),
        )
    } else {
        (
            if e.origin.known {
                "Unforced river runoff".to_owned()
            } else {
                "Origin was not recorded in this snapshot".to_owned()
            },
            None,
        )
    };
    let cause_str = fbb.create_string(&cause);
    let cause_link = cause_link.map(|l| put_link(&mut fbb, &l));
    let weather = fbb.create_string("Weather at peak flow was not recorded");
    let incidents: Vec<_> = sim
        .people
        .floods
        .high_waters
        .iter()
        .filter(|hw| hw.episode == Some(e.id))
        .map(|hw| {
            let r = high_water_row(sim, hw);
            put_row(&mut fbb, sim, &r)
        })
        .collect();
    let incidents = fbb.create_vector(&incidents);
    let summary = vec![
        format!("Total buildings flooded: {}", e.total_buildings),
        format!("Total fields lost: {}", e.total_fields_lost),
        format!("Total goods spoiled: {:.1} kg", e.total_spoiled_kg),
    ];
    let summary = put_lines(&mut fbb, &summary);
    let body = wire::FloodInfo::create(
        &mut fbb,
        &wire::FloodInfoArgs {
            row: Some(row),
            cause: Some(cause_str),
            cause_link,
            weather: Some(weather),
            incidents: Some(incidents),
            summary: Some(summary),
        },
    );
    Some(response(fbb, wire::ResponseBody::FloodInfo, body))
}

pub fn chronicle_link(sim: &Sim, e: &ChronicleEvent) -> Option<(u8, u32)> {
    sim.people
        .floods
        .flood_episodes
        .iter()
        .rev()
        .find(|ep| ep.began <= e.at && ep.ended.is_none_or(|end| end >= e.at))
        .map(|ep| (FLOOD, ep.id))
}
