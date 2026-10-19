//! The land map (wire 1.89, M8c slice CX; ADR-0030 §1, §8): every plot, drawn from what is
//! recorded of it and labelled so: what was paid for it (a parcel given for a fee) and what its
//! household reckoned it worth (its receipt: the hours a year it saved over the next best), or
//! nothing recorded. The kernel keeps no land value; this reads the world and changes nothing.

use civ_land::{AllotRule, Plot};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::plans::rect_m;
use super::response;
use crate::Sim;

/// What was paid for plot `p`, kcal: the fee of the plan that gave it by offer, at its good's
/// food energy; 0 for a plot given otherwise or taken freely.
pub fn paid_kcal(sim: &Sim, p: &Plot) -> f64 {
    let Some(given) = p.allotted.filter(|a| a.rule == AllotRule::Offer) else {
        return 0.0;
    };
    let catalog = &sim.rules.catalog;
    sim.people
        .polities
        .iter()
        .find_map(|polity| polity.laws.iter().find(|l| l.id == given.law))
        .and_then(|l| catalog.policies.get(usize::from(l.policy)))
        .and_then(|d| d.plan.as_ref())
        .and_then(|d| d.fee)
        .map_or(0.0, |(good, amount)| {
            f64::from(amount) * catalog.goods.get(good).map_or(0.0, |g| g.kcal_per_kg)
        })
}

/// What plot `p`'s household reckoned it worth, the hours a year it saved over the next best,
/// and how many it weighed; `None` for a plot laid without a receipt.
pub fn reckoned(p: &Plot) -> Option<(f64, u16)> {
    p.receipt
        .map(|r| (f64::from((r.next_hours - r.hours).max(0.0)), r.weighed))
}

/// The land map's revision: 0 while no plot is laid; else it changes whenever a plot is laid or
/// given up, or its receipt or parcel changes.
pub fn land_rev(sim: &Sim) -> u64 {
    if sim.land.plots.is_empty() {
        return 0;
    }
    // FNV-1a over what the map shows.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        for b in v.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for p in &sim.land.plots {
        mix(p.id.get());
        mix(p.receipt.map_or(0, |r| {
            u64::from(r.hours.to_bits()) << 32 | u64::from(r.weighed)
        }));
        mix(p.allotted.map_or(0, |a| a.law.get() ^ u64::from(a.parcel)));
    }
    h.max(1)
}

/// The land map (wire 1.89): every plot, what was paid for it and what it was reckoned worth.
pub fn land_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let plots: Vec<_> = sim
        .land
        .plots
        .iter()
        .map(|p| {
            let rect = rect_m(&p.rect);
            let (reckoned_h, weighed) = reckoned(p).unwrap_or((0.0, 0));
            wire::LandPlot::create(
                &mut fbb,
                &wire::LandPlotArgs {
                    rect: Some(&rect),
                    paid_kcal: paid_kcal(sim, p) as f32,
                    reckoned_h: reckoned_h as f32,
                    weighed,
                },
            )
        })
        .collect();
    let plots = fbb.create_vector(&plots);
    let body = wire::Land::create(
        &mut fbb,
        &wire::LandArgs {
            rev: land_rev(sim),
            plots: Some(plots),
        },
    );
    response(fbb, wire::ResponseBody::Land, body.as_union_value())
}
