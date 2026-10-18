//! The Traffic lens (wire 1.85, M8b slice CN; ADR-0029 §9; research 12-05 §4.1: congestion's
//! signature is delay concentrated at particular places and times): each crossing and gate that
//! passed anyone in about the last 30 days, its passages and waits by hour of the day, read from
//! its tallies. Derived when asked, never saved; nothing in the world reads it.

use civ_agents::traffic::{Facility, Tally};
use civ_core::PermanentId;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// The Traffic lens.
pub const TRAFFIC: u8 = 3;

/// A facility in words: "Ada's household's log footbridge", "a gate of Ashford's palisade".
pub fn facility_name(sim: &Sim, f: Facility) -> String {
    match f {
        Facility::Crossing(id) => sim
            .land
            .crossings
            .list
            .iter()
            .find(|c| c.id == id)
            .map_or_else(
                || "a crossing gone".to_owned(),
                |c| {
                    let name = sim
                        .rules
                        .catalog
                        .bridges
                        .get(usize::from(c.system))
                        .map_or_else(|| "crossing".to_owned(), |d| d.name.to_lowercase());
                    format!("{} {name}", super::crossings::whose(sim, c))
                },
            ),
        Facility::Gate { enclosure, .. } => sim
            .land
            .enclosures
            .list
            .iter()
            .find(|e| e.id == enclosure)
            .and_then(|e| sim.land.settlements.iter().find(|s| s.id == e.settlement))
            .map_or_else(
                || "a gate in a palisade".to_owned(),
                |s| format!("a gate of {}'s palisade", s.name),
            ),
    }
}

/// Where a facility is, metres: a crossing's middle, a gate's sector's.
pub fn facility_at(sim: &Sim, f: Facility) -> Option<(f32, f32)> {
    match f {
        Facility::Crossing(id) => {
            let c = sim.land.crossings.list.iter().find(|c| c.id == id)?;
            let a = civ_agents::population::cell_centre(&sim.map, c.banks[0] as usize);
            let b = civ_agents::population::cell_centre(&sim.map, c.banks[1] as usize);
            Some(((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0))
        }
        Facility::Gate { enclosure, sector } => {
            let e = sim
                .land
                .enclosures
                .list
                .iter()
                .find(|e| e.id == enclosure)?;
            let s = e.sectors.get(usize::from(sector))?;
            Some(((s.a.0 + s.b.0) / 2.0, (s.a.1 + s.b.1) / 2.0))
        }
    }
}

/// The settlement a facility is by: a gate's enclosure's, a crossing's nearest hearth.
fn facility_settlement(sim: &Sim, f: Facility, at: (f32, f32)) -> Option<PermanentId> {
    match f {
        Facility::Gate { enclosure, .. } => sim
            .land
            .enclosures
            .list
            .iter()
            .find(|e| e.id == enclosure)
            .map(|e| e.settlement),
        Facility::Crossing(_) => {
            let d = |s: &civ_land::Settlement| (s.hearth_m.0 - at.0).hypot(s.hearth_m.1 - at.1);
            sim.land
                .settlements
                .iter()
                .filter(|s| s.abandoned.is_none())
                .min_by(|a, b| d(a).total_cmp(&d(b)).then(a.id.cmp(&b.id)))
                .map(|s| s.id)
        }
    }
}

/// A place on the lens: its faded tally, the facility, where it is and the settlement it serves.
type Place = (Tally, Facility, (f32, f32), Option<PermanentId>);

/// Whether the tally's longest wait is still within its memory on `day`.
fn longest_kept(t: &Tally, day: i64) -> bool {
    day - t.longest_day <= civ_agents::traffic::TALLY_MEMORY_DAYS as i64
}

/// A place's lines: its passages and waits, its busiest hour, its longest wait and those held
/// short.
fn place_words(t: &Tally, day: i64) -> Vec<String> {
    let (passages, waited) = t.totals();
    let per_s = if passages > 0.0 {
        waited * 60.0 / passages
    } else {
        0.0
    };
    let mut out = vec![format!(
        "{passages:.0} passages over about the last 30 days, {waited:.1} minutes waited in all \
         ({per_s:.1} s a passage)"
    )];
    if let Some((hour, n)) = t
        .passages
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
        .filter(|(_, n)| **n >= 0.5)
    {
        out.push(format!("busiest from {hour:02}:00, {n:.0} passages"));
    }
    if t.longest_min >= 1.0 && longest_kept(t, day) {
        out.push(format!(
            "the longest wait in memory: {:.0} minutes",
            t.longest_min
        ));
    } else {
        out.push("nobody waited a minute".to_owned());
    }
    if t.held_short >= 0.5 {
        out.push(format!(
            "{:.0} stood short of its approach, its waiting space full",
            t.held_short
        ));
    }
    out
}

/// The Traffic lens over settlement `settlement` (every one when `None`): each facility that
/// passed anyone lately, busiest first.
pub fn lens_response(sim: &Sim, settlement: Option<PermanentId>) -> Vec<u8> {
    let day = sim.now().day_index();
    let mut places: Vec<Place> = sim
        .people
        .traffic
        .tallies
        .iter()
        .filter_map(|t| {
            let at = facility_at(sim, t.facility)?;
            let s = facility_settlement(sim, t.facility, at);
            settlement
                .is_none_or(|want| s == Some(want))
                .then(|| (t.faded(day), t.facility, at, s))
        })
        .filter(|p| p.0.totals().0 >= 0.5)
        .collect();
    places.sort_by(|a, b| {
        b.0.totals()
            .0
            .total_cmp(&a.0.totals().0)
            .then(a.1.cmp(&b.1))
    });
    let mut fbb = FlatBufferBuilder::new();
    let rows: Vec<_> = places
        .iter()
        .map(|(t, f, at, s)| {
            let name = fbb.create_string(&facility_name(sim, *f));
            let hour_passages = fbb.create_vector(&t.passages);
            let hour_wait_s: Vec<f32> = t
                .passages
                .iter()
                .zip(&t.waited_min)
                .map(|(&n, &w)| if n > 0.0 { w * 60.0 / n } else { 0.0 })
                .collect();
            let hour_wait_s = fbb.create_vector(&hour_wait_s);
            let lines: Vec<_> = place_words(t, day)
                .iter()
                .map(|l| fbb.create_string(l))
                .collect();
            let words = fbb.create_vector(&lines);
            let (passages, waited) = t.totals();
            wire::CoveragePlace::create(
                &mut fbb,
                &wire::CoveragePlaceArgs {
                    name: Some(name),
                    at: Some(&wire::Vec2::new(at.0, at.1)),
                    settlement: s.map_or(0, PermanentId::get),
                    passages,
                    waited_min: waited,
                    held_short: t.held_short,
                    longest_min: if longest_kept(t, day) {
                        t.longest_min
                    } else {
                        0.0
                    },
                    hour_passages: Some(hour_passages),
                    hour_wait_s: Some(hour_wait_s),
                    words: Some(words),
                },
            )
        })
        .collect();
    let rows = fbb.create_vector(&rows);
    let homes: Vec<civ_schema::flatbuffers::WIPOffset<wire::CoverageHome>> = Vec::new();
    let homes = fbb.create_vector(&homes);
    let place = settlement
        .and_then(|s| sim.land.settlements.iter().find(|x| x.id == s))
        .map_or_else(|| "Every settlement".to_owned(), |s| s.name.clone());
    let scope = fbb.create_string(&format!(
        "{place}: {} crossings and gates that passed anyone lately",
        places.len()
    ));
    let title = fbb.create_string("Traffic");
    let periods = [
        "about the last 30 days, each passage fading".to_owned(),
        "by the hour of the day it was reached".to_owned(),
    ];
    let periods: Vec<_> = periods.iter().map(|p| fbb.create_string(p)).collect();
    let periods = fbb.create_vector(&periods);
    let denominator = fbb.create_string("passages: each walker through a crossing or gate once");
    let (passages, waited) = places.iter().fold((0.0f32, 0.0f32), |a, p| {
        let t = p.0.totals();
        (a.0 + t.0, a.1 + t.1)
    });
    let longest = places
        .iter()
        .filter(|p| longest_kept(&p.0, day) && p.0.longest_min >= 1.0)
        .max_by(|a, b| a.0.longest_min.total_cmp(&b.0.longest_min));
    let summary = if places.is_empty() {
        vec![
            "No crossing or gate has passed anyone in about the last 30 days.".to_owned(),
            String::new(),
        ]
    } else {
        vec![
            format!(
                "{} crossings and gates passed {passages:.0} walkers over about the last 30 days \
                 and kept them {waited:.1} minutes in all, {:.1} s a passage.",
                places.len(),
                waited * 60.0 / passages.max(1.0)
            ),
            match longest {
                Some(p) => format!(
                    "The longest wait in memory: {:.0} minutes, at {}.",
                    p.0.longest_min,
                    facility_name(sim, p.1)
                ),
                None => "Nobody waited a minute at any.".to_owned(),
            },
        ]
    };
    let summary: Vec<_> = summary.iter().map(|l| fbb.create_string(l)).collect();
    let summary = fbb.create_vector(&summary);
    let body = wire::Coverage::create(
        &mut fbb,
        &wire::CoverageArgs {
            lens: TRAFFIC,
            title: Some(title),
            scope: Some(scope),
            periods: Some(periods),
            denominator: Some(denominator),
            homes: Some(homes),
            limit_min: 0.0,
            days: 30,
            summary: Some(summary),
            places: Some(rows),
        },
    );
    response(fbb, wire::ResponseBody::Coverage, body)
}
