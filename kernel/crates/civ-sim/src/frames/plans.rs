//! The plans layer (wire 1.88, M8c slice CT, fourth part; ADR-0030 §3, §8): each plan put
//! forward or in force, as laid, in three layers kept apart (research 10-04 §6.1): the ground it
//! reserves (its streets and square) and the parcels it lays out; the reserved ground a way
//! already runs over; and the buildings standing on reserved ground. It reads the world and
//! changes nothing.

use civ_agents::polity::{LawStatus, Polity};
use civ_core::PermanentId;
use civ_land::RectCm;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// The plans shown of polity `p`, each put forward and not yet decided or in force: (law, its
/// status), in the order laid.
fn plans_of(p: &Polity) -> impl Iterator<Item = (PermanentId, LawStatus)> + '_ {
    p.plans.iter().filter_map(|(law, _)| {
        p.laws
            .iter()
            .find(|l| l.id == *law)
            .map(|l| (*law, l.status))
            .filter(|(_, s)| matches!(s, LawStatus::Proposed | LawStatus::InForce))
    })
}

/// The plots of the buildings standing on ground law `law`'s plan reserves, in the order begun.
fn occupied(sim: &Sim, p: &Polity, law: PermanentId) -> Vec<RectCm> {
    p.encroachments
        .iter()
        .filter(|e| e.law == law)
        .filter_map(|e| {
            let b = sim
                .land
                .buildings
                .iter()
                .find(|b| b.id == e.building && b.standing())?;
            sim.land
                .plots
                .iter()
                .find(|q| q.id == b.plot)
                .map(|q| q.rect)
        })
        .collect()
}

/// Whether a way runs over `rect`: the centre of a cell of one of its stretches lies within it.
fn way_over(sim: &Sim, rect: &RectCm) -> bool {
    let map = &sim.map;
    let (x0, y0) = (f64::from(rect.x) / 100.0, f64::from(rect.y) / 100.0);
    let (x1, y1) = (
        x0 + f64::from(rect.w) / 100.0,
        y0 + f64::from(rect.h) / 100.0,
    );
    sim.land.ways.list.iter().any(|w| {
        w.segments.iter().any(|s| {
            s.cells.iter().any(|&c| {
                let (x, y) = civ_agents::population::cell_centre(map, c as usize);
                let (x, y) = (f64::from(x), f64::from(y));
                x >= x0 && x < x1 && y >= y0 && y < y1
            })
        })
    })
}

/// The plans' revision: 0 while no polity has laid a plan; else it changes whenever a plan is
/// put forward, passed or ended, a building is begun on its reserved ground or one there falls,
/// and with the ways.
pub fn plans_rev(sim: &Sim) -> u64 {
    if sim.people.polities.iter().all(|p| p.plans.is_empty()) {
        return 0;
    }
    // FNV-1a over what the layer shows.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        for b in v.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    mix(u64::from(sim.land.ways.revision()));
    for p in &sim.people.polities {
        for (law, status) in plans_of(p) {
            mix(law.get());
            mix(u64::from(status == LawStatus::InForce));
            mix(occupied(sim, p, law).len() as u64);
        }
    }
    h.max(1)
}

/// A rectangle in metres.
fn rect_m(r: &RectCm) -> wire::PlanRect {
    wire::PlanRect::new(
        r.x as f32 / 100.0,
        r.y as f32 / 100.0,
        r.w as f32 / 100.0,
        r.h as f32 / 100.0,
    )
}

/// A plan in words: "Ashford: a plan (version 1) of ..., in force; a way already runs over 2 of
/// the stretches it reserves; 1 building stands on its reserved ground".
fn words(sim: &Sim, p: &Polity, law: PermanentId, built: usize, occupied: usize) -> String {
    let place = sim
        .land
        .settlements
        .iter()
        .find(|s| s.id == p.settlement)
        .map_or_else(|| "A polity".to_owned(), |s| s.name.clone());
    let Some(l) = p.laws.iter().find(|l| l.id == law) else {
        return place;
    };
    let status = if l.status == LawStatus::InForce {
        "in force"
    } else {
        "put forward"
    };
    let mut out = format!("{place}: {}, {status}", super::word::law_text(sim, p, l));
    if built > 0 {
        out.push_str(&format!(
            "; a way already runs over {built} of the stretches it reserves"
        ));
    }
    match occupied {
        0 => {}
        1 => out.push_str("; 1 building stands on its reserved ground"),
        n => out.push_str(&format!("; {n} buildings stand on its reserved ground")),
    }
    out
}

/// Every plan put forward or in force, by polity, each in the order laid.
pub fn plans_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let mut list = Vec::new();
    for p in &sim.people.polities {
        for (law, status) in plans_of(p) {
            let Some(laid) = p.plan_of(law) else {
                continue;
            };
            let layout = &laid.layout;
            let streets: Vec<_> = layout.streets.iter().map(rect_m).collect();
            let sites: Vec<_> = layout.sites.iter().map(|s| rect_m(&s.rect)).collect();
            let parcels: Vec<_> = layout.parcels.iter().map(|q| rect_m(&q.rect)).collect();
            let built: Vec<_> = layout
                .streets
                .iter()
                .chain(layout.sites.iter().map(|s| &s.rect))
                .filter(|r| way_over(sim, r))
                .map(rect_m)
                .collect();
            let on: Vec<_> = occupied(sim, p, law).iter().map(rect_m).collect();
            let text = fbb.create_string(&words(sim, p, law, built.len(), on.len()));
            let streets = fbb.create_vector(&streets);
            let sites = fbb.create_vector(&sites);
            let parcels = fbb.create_vector(&parcels);
            let built = fbb.create_vector(&built);
            let on = fbb.create_vector(&on);
            list.push(wire::PlanInfo::create(
                &mut fbb,
                &wire::PlanInfoArgs {
                    law: law.get(),
                    version: laid.version,
                    in_force: status == LawStatus::InForce,
                    streets: Some(streets),
                    sites: Some(sites),
                    parcels: Some(parcels),
                    built: Some(built),
                    occupied: Some(on),
                    words: Some(text),
                },
            ));
        }
    }
    let list = fbb.create_vector(&list);
    let body = wire::Plans::create(
        &mut fbb,
        &wire::PlansArgs {
            rev: plans_rev(sim),
            plans: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Plans, body)
}
