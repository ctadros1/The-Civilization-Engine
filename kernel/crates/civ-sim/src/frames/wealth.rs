//! Property regimes and wealth on the boundary (M3a slice K; ADR-0007): a regime's rules in words,
//! the snapshot's wealth revision and the answer to a wealth query. The measures are the kernel's
//! ([`civ_agents::wealth`]); observers only show them.

use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::params::{LandHolder, LandUse, RegimeDef, Succession};
use civ_agents::wealth::{HouseholdWealth, Spread};
use civ_core::PermanentId;
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use super::people::household_name;
use super::response;
use crate::Sim;

/// A regime's rules in words, one sentence each.
pub fn regime_rules(r: &RegimeDef) -> Vec<String> {
    let mut out = Vec::new();
    out.push(
        match r.holder {
            LandHolder::Breaker => "A household holds the ground it breaks.",
            LandHolder::Settlement => "The settlement holds the ground its households break.",
        }
        .to_owned(),
    );
    out.push(
        match r.land_use {
            LandUse::Holder => {
                "A household works the ground it holds; ground whose holder is no more is taken \
                 up by a household short of land, which then holds it."
            }
            LandUse::Need => {
                "Each year, and when households form or end, the settlement gives its households \
                 fields to work by how many each feeds."
            }
        }
        .to_owned(),
    );
    out.push(match r.lease {
        Some(l) => format!(
            "A holder may let ground it does not need to a household short of land for {:.0}% of \
             the grain threshed from it, {} at a time.",
            l.holder_share * 100.0,
            if l.term_years == 1 {
                "a crop year".to_owned()
            } else {
                format!("{} crop years", l.term_years)
            }
        ),
        None => "Fields are not let.".to_owned(),
    });
    if r.union_share {
        out.push("A new couple's household takes a share of its families' fields.".to_owned());
    }
    out.push(
        match r.succession {
            Succession::Heir => "When a household is no more, its fields go to one heir.",
            Succession::Divided => {
                "When a household is no more, its fields are divided among its heirs' households."
            }
            Succession::Settlement => {
                "When a household is no more, its fields go back to the settlement."
            }
        }
        .to_owned(),
    );
    out
}

/// A number that changes at the start of each month, when households form or end and when a
/// year's wealth measures are recorded (0 = no households).
pub fn wealth_rev(sim: &Sim) -> u64 {
    let households: Vec<u64> = sim
        .people
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !h.members.is_empty())
        .map(|h| h.id.get())
        .collect();
    if households.is_empty() {
        return 0;
    }
    let date = sim.date();
    let mut hasher = DefaultHasher::new();
    (
        date.year,
        date.month,
        households,
        sim.people.wealth_years.len(),
    )
        .hash(&mut hasher);
    hasher.finish() | 1
}

fn spread_info<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    year: i64,
    s: &Spread,
) -> WIPOffset<wire::WealthSpread<'a>> {
    wire::WealthSpread::create(
        fbb,
        &wire::WealthSpreadArgs {
            year,
            households: s.households,
            people: s.people,
            gini_goods: s.gini_goods as f32,
            gini_held: s.gini_held as f32,
            gini_worked: s.gini_worked as f32,
            gini_floor: s.gini_floor as f32,
            top_tenth_goods: s.top_tenth_goods as f32,
            holding_none: s.holding_none as f32,
            working_none: s.working_none as f32,
            goods_h_per_head: s.goods_h_per_head as f32,
            worked_ha_per_head: s.worked_ha_per_head as f32,
            floor_m2_per_house: s.floor_m2_per_house as f32,
            common_ha: s.common_ha as f32,
        },
    )
}

fn household_info<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    w: &HouseholdWealth,
) -> WIPOffset<wire::HouseholdWealth<'a>> {
    let name = fbb.create_string(&household_name(sim, w.household));
    wire::HouseholdWealth::create(
        fbb,
        &wire::HouseholdWealthArgs {
            household: w.household.get(),
            name: Some(name),
            members: w.members,
            held_ha: w.held_ha as f32,
            worked_ha: w.worked_ha as f32,
            let_ha: w.let_ha as f32,
            rented_ha: w.rented_ha as f32,
            goods_h: w.goods_h as f32,
            floor_m2: w.floor_m2 as f32,
        },
    )
}

/// A `Response` with every settlement's wealth measures: as they stand, each household's (the
/// most goods per head first), and at the end of each year. A settlement with nobody left keeps
/// its history.
pub fn wealth_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let rules = &sim.rules;
    let year = sim.date().year;
    let now = sim.people.wealth_by_settlement(
        &rules.catalog,
        &rules.people,
        &rules.land,
        &sim.land,
        sim.now(),
    );
    let history = &sim.people.wealth_years;
    let ids: BTreeSet<PermanentId> = now
        .iter()
        .map(|(s, _)| s.settlement)
        .chain(history.iter().map(|y| y.spread.settlement))
        .collect();
    let list: Vec<_> = ids
        .into_iter()
        .map(|id| {
            let name = sim
                .land
                .settlements
                .iter()
                .find(|s| s.id == id)
                .map_or("", |s| s.name.as_str());
            let name = fbb.create_string(name);
            let current = now.iter().find(|(s, _)| s.settlement == id);
            let spread = current.map(|(s, _)| spread_info(&mut fbb, year, s));
            let mut households: Vec<&HouseholdWealth> =
                current.map(|(_, h)| h.iter().collect()).unwrap_or_default();
            let per_head = |w: &HouseholdWealth| w.goods_h / f64::from(w.members.max(1));
            households.sort_by(|a, b| {
                per_head(b)
                    .total_cmp(&per_head(a))
                    .then(a.household.cmp(&b.household))
            });
            let households: Vec<_> = households
                .into_iter()
                .map(|w| household_info(&mut fbb, sim, w))
                .collect();
            let households = fbb.create_vector(&households);
            let years: Vec<_> = history
                .iter()
                .filter(|y| y.spread.settlement == id)
                .map(|y| spread_info(&mut fbb, y.year, &y.spread))
                .collect();
            let years = fbb.create_vector(&years);
            wire::SettlementWealth::create(
                &mut fbb,
                &wire::SettlementWealthArgs {
                    settlement: id.get(),
                    name: Some(name),
                    now: spread,
                    households: Some(households),
                    history: Some(years),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let regime_name = fbb.create_string(&sim.regime().name);
    let body = wire::Wealth::create(
        &mut fbb,
        &wire::WealthArgs {
            rev: wealth_rev(sim),
            regime_name: Some(regime_name),
            settlements: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Wealth, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_read_as_sentences_for_each_regime() {
        let mut r = RegimeDef::legacy();
        let text = regime_rules(&r).join(" ");
        assert!(text.contains("holds the ground it breaks"), "{text}");
        assert!(text.contains("go to one heir"), "{text}");
        r.holder = LandHolder::Settlement;
        r.land_use = LandUse::Need;
        r.succession = Succession::Settlement;
        r.lease = None;
        r.union_share = false;
        let text = regime_rules(&r);
        assert_eq!(text.len(), 4, "{text:?}");
        assert!(text.iter().all(|s| s.ends_with('.')), "{text:?}");
        assert!(text[1].contains("by how many each feeds"), "{text:?}");
        assert_eq!(text[2], "Fields are not let.");
    }
}
