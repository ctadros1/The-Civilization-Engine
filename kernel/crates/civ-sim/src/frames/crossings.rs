//! The crossings panel (wire 1.60, M5c slice AW; ADR-0004 §7, ADR-0009 §9): every crossing over
//! water, where it stands, how far its building has gone, and its condition in words.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::bridge::crossing_margin;
use civ_core::SimTime;
use civ_land::crossings::{Collapse, Crossing, CrossingOwner, CrossingState};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::people::eldest_name;
use super::response;
use crate::Sim;

/// What the wire numbers a crossing's owner as: 0 a household, 1 a polity, and which.
fn owner_code(c: &Crossing) -> (u8, u64) {
    match c.owner {
        CrossingOwner::Household(h) => (0, h.get()),
        CrossingOwner::Polity(p) => (1, p.get()),
    }
}

/// The crossings' revision: changes whenever one is begun, worked on, opens, rots or gives way;
/// 0 when the world has none.
pub fn crossings_rev(sim: &Sim) -> u64 {
    let list = &sim.land.crossings.list;
    if list.is_empty() {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    for c in list {
        let (state, work, day) = match c.state {
            CrossingState::Building { work_h } => (0u8, work_h.to_bits(), 0),
            CrossingState::Open { since } => (1, 0, since),
            CrossingState::Failed { day, .. } => (2, 0, day),
        };
        (
            c.id.get(),
            state,
            work,
            day,
            c.loss.to_bits(),
            owner_code(c),
        )
            .hash(&mut hasher);
    }
    hasher.finish() | 1
}

/// Whose a crossing is, as a possessive: "Ada's household's", "Ashford's", "a".
fn whose(sim: &Sim, c: &Crossing) -> String {
    match c.owner {
        CrossingOwner::Household(h) => eldest_name(sim, h)
            .map_or_else(|| "a".to_owned(), |name| format!("{name}'s household's")),
        CrossingOwner::Polity(p) => sim
            .people
            .polities
            .iter()
            .find(|x| x.id == p)
            .and_then(|x| sim.land.settlements.iter().find(|s| s.id == x.settlement))
            .map_or_else(|| "a".to_owned(), |s| format!("{}'s", s.name)),
    }
}

/// The year of day `day`, the world's first being 1.
fn year_of(day: i64) -> i64 {
    SimTime::from_minutes(day * 24 * 60).date().year
}

/// A count in words, for the few members a crossing has.
fn count(n: u8) -> String {
    match n {
        1 => "one".to_owned(),
        2 => "two".to_owned(),
        3 => "three".to_owned(),
        4 => "four".to_owned(),
        n => n.to_string(),
    }
}

/// A crossing in words: "Ada's household's log footbridge, being built: 40% of 70 hours of
/// work", "..., open since year 2: two members 20 cm thick over 2.4 m, sound; it carries a walker
/// 19.0 times over", "..., gave way in year 9 under someone stepping onto it".
fn words(sim: &Sim, c: &Crossing, margin: Option<f64>) -> String {
    let name = sim
        .rules
        .catalog
        .bridges
        .get(usize::from(c.system))
        .map_or_else(|| "crossing".to_owned(), |d| d.name.to_lowercase());
    let what = format!("{} {name}", whose(sim, c));
    match c.state {
        CrossingState::Building { work_h } => {
            let share = if c.labour_h > 0.0 {
                (f64::from(work_h) / f64::from(c.labour_h) * 100.0).floor()
            } else {
                0.0
            };
            format!(
                "{what}, being built: {share}% of {:.0} hours of work",
                c.labour_h
            )
        }
        CrossingState::Open { since } => {
            let rot = if c.loss < 0.05 {
                "sound".to_owned()
            } else {
                format!(
                    "rot has taken {:.0}% of their section",
                    f64::from(c.loss) * 100.0
                )
            };
            let carries = match margin {
                Some(m) if m >= 1.0 => format!("; it carries a walker {m:.1} times over"),
                Some(_) => "; it would give way under the next to step onto it".to_owned(),
                None => String::new(),
            };
            format!(
                "{what}, open since year {}: {} members {:.0} cm thick over {:.1} m, {rot}{carries}",
                year_of(since),
                count(c.members),
                c.diameter_cm,
                c.span_m
            )
        }
        CrossingState::Failed { day, why } => {
            let how = match why {
                Collapse::OwnWeight => "under its own weight",
                Collapse::UnderWalker => "under someone stepping onto it",
            };
            format!("{what}, gave way in year {} {how}", year_of(day))
        }
    }
}

/// Every crossing, in the order begun.
pub fn crossings_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let map = &sim.map;
    let mut list = Vec::with_capacity(sim.land.crossings.list.len());
    for c in &sim.land.crossings.list {
        let def = sim.rules.catalog.bridges.get(usize::from(c.system));
        let margin = def.filter(|_| c.open()).and_then(|d| {
            let timber = sim.rules.catalog.goods.get(d.good)?.timber?;
            Some(crossing_margin(c, d, &timber, 1))
        });
        let text = fbb.create_string(&words(sim, c, margin));
        let system = fbb.create_string(def.map_or("", |d| d.name.as_str()));
        let a = civ_agents::population::cell_centre(map, c.banks[0] as usize);
        let b = civ_agents::population::cell_centre(map, c.banks[1] as usize);
        let (state, work_h) = match c.state {
            CrossingState::Building { work_h } => (0, work_h),
            CrossingState::Open { .. } => (1, c.labour_h),
            CrossingState::Failed { .. } => (2, c.labour_h),
        };
        let (owner_kind, owner) = owner_code(c);
        let length_m = def.map_or(c.span_m, |d| c.span_m + 2.0 * d.bearing_m as f32);
        list.push(wire::CrossingInfo::create(
            &mut fbb,
            &wire::CrossingInfoArgs {
                id: c.id.get(),
                system: Some(system),
                ax: a.0,
                ay: a.1,
                bx: b.0,
                by: b.1,
                span_m: c.span_m,
                members: c.members,
                diameter_cm: c.diameter_cm,
                state,
                labour_h: c.labour_h,
                work_h,
                quality: c.quality,
                loss: c.loss,
                margin: margin.unwrap_or(0.0) as f32,
                owner_kind,
                owner,
                words: Some(text),
                length_m,
            },
        ));
    }
    let list = fbb.create_vector(&list);
    let body = wire::Crossings::create(
        &mut fbb,
        &wire::CrossingsArgs {
            rev: crossings_rev(sim),
            crossings: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Crossings, body)
}
