//! The deposits panel (wire 1.19, M3b slice Q; ADR-0010 §1): every deposit in the ground, how
//! much is left in it, and which settlements know it and how they came to.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// The deposits' revision: changes whenever a deposit is laid down, found or dug from; 0 when the
/// world has none.
pub fn deposits_rev(sim: &Sim) -> u64 {
    let deposits = &sim.land.deposits;
    if deposits.is_empty() {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    for d in deposits {
        (d.id.get(), d.taken_kg.to_bits()).hash(&mut hasher);
    }
    for k in &sim.people.deposits_known {
        (k.settlement.get(), k.deposit.get()).hash(&mut hasher);
    }
    hasher.finish() | 1
}

/// Every deposit, in the order laid down, with who knows it: "found by Ash of Alderford in year
/// 3".
pub fn deposits_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let mut list = Vec::with_capacity(sim.land.deposits.len());
    for d in &sim.land.deposits {
        let b = &d.body;
        let known: Vec<_> = sim
            .people
            .deposits_known
            .iter()
            .filter(|k| k.deposit == d.id)
            .collect();
        let known_by: Vec<u64> = known.iter().map(|k| k.settlement.get()).collect();
        let finds: Vec<String> = known
            .iter()
            .map(|k| {
                let place = sim
                    .land
                    .settlements
                    .iter()
                    .find(|s| s.id == k.settlement)
                    .map_or_else(String::new, |s| format!(" of {}", s.name));
                format!(
                    "found by {}{place} in year {}",
                    sim.people.name_of(k.finder),
                    k.at.date().year
                )
            })
            .collect();
        let known_by = fbb.create_vector(&known_by);
        let finds: Vec<_> = finds.iter().map(|f| fbb.create_string(f)).collect();
        let finds = fbb.create_vector(&finds);
        list.push(wire::DepositInfo::create(
            &mut fbb,
            &wire::DepositInfoArgs {
                id: d.id.get(),
                good: u32::from(b.good),
                x: (b.at_cm.0 as f64 / 100.0) as f32,
                y: (b.at_cm.1 as f64 / 100.0) as f32,
                radius_m: b.radius_cm as f32 / 100.0,
                exposed: b.exposed,
                cover_m: b.top_cm as f32 / 100.0,
                thickness_m: b.thickness_cm as f32 / 100.0,
                quality: b.quality,
                left_kg: d.left_kg(),
                taken_kg: d.taken_kg,
                known_by: Some(known_by),
                finds: Some(finds),
            },
        ));
    }
    let list = fbb.create_vector(&list);
    let body = wire::Deposits::create(
        &mut fbb,
        &wire::DepositsArgs {
            rev: deposits_rev(sim),
            deposits: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Deposits, body)
}
