//! Property regimes and wealth on the boundary (M3a slice K; ADR-0007): a regime's rules in words,
//! the snapshot's wealth revision and the answer to a wealth query. The measures are the kernel's
//! ([`civ_agents::wealth`]); observers only show them.

use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::params::{LandHolder, LandUse, RegimeDef, Succession};
use civ_agents::wealth::{HouseholdWealth, Spread};
use civ_core::{PermanentId, SimTime};
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
            roofed_m2_per_house: s.roofed_m2_per_house as f32,
            storage_kg_per_house: s.storage_kg_per_house as f32,
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
            roofed_m2: w.roofed_m2 as f32,
            storage_kg: w.storage_kg as f32,
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
            let loans: Vec<_> = loan_words(sim, id)
                .iter()
                .map(|w| fbb.create_string(w))
                .collect();
            let loans = fbb.create_vector(&loans);
            let coin: Vec<_> = coin_words(sim, id)
                .iter()
                .map(|w| fbb.create_string(w))
                .collect();
            let coin = fbb.create_vector(&coin);
            wire::SettlementWealth::create(
                &mut fbb,
                &wire::SettlementWealthArgs {
                    settlement: id.get(),
                    name: Some(name),
                    now: spread,
                    households: Some(households),
                    history: Some(years),
                    loans: Some(loans),
                    coin: Some(coin),
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

/// Loans the households of `settlement` owe (M7d slice CA, ADR-0027 §1), in words: open ones
/// first, then those closed within a year, newest first, at most twelve. "Wren's household owes
/// Mira's household 40 kg of grain, lent on 3 May of year 2, due on 9 September of year 2";
/// "... repaid it"; "... did not repay 12 kg of it".
pub fn loan_words(sim: &Sim, settlement: PermanentId) -> Vec<String> {
    use civ_agents::credit::{Pledge, Standing};
    let pop = &sim.people;
    let today = sim.now().day_index();
    let year = civ_core::time::DAYS_PER_YEAR;
    // A household's settlement, or a workshop's (M7d slice CB: wages a workshop owes).
    let here = |h: PermanentId| {
        pop.household(h)
            .and_then(|x| x.settlement)
            .or_else(|| pop.firm(h).and_then(|f| f.settlement))
            == Some(settlement)
    };
    // A workshop, a polity's common store (M7d slice CC) or a household.
    let name = |h: PermanentId| {
        if pop.firm(h).is_some() {
            super::people::firm_name(sim, h)
        } else if pop.polities.iter().any(|p| p.id == h) {
            "the common store".to_owned()
        } else {
            super::people::household_name(sim, h)
        }
    };
    let mut loans: Vec<&civ_agents::credit::Claim> = pop
        .order
        .credit
        .claims
        .iter()
        .filter(|c| here(c.debtor))
        .filter(|c| c.standing == Standing::Open || today - c.due <= year)
        .collect();
    loans.sort_by(|a, b| {
        (a.standing != Standing::Open)
            .cmp(&(b.standing != Standing::Open))
            .then(b.made.cmp(&a.made))
            .then(b.id.cmp(&a.id))
    });
    loans.truncate(12);
    let day = |d: i64| civ_agents::polity::day_words(SimTime::from_minutes(d * 1440));
    loans
        .into_iter()
        .map(|c| {
            let good = sim
                .rules
                .catalog
                .goods
                .get(usize::from(c.good))
                .map_or("goods".to_owned(), |g| g.name.to_lowercase());
            let base = format!(
                "{} {} {} {:.0} kg of {good}, {} on {}",
                name(c.debtor),
                if c.standing == Standing::Open {
                    "owes"
                } else {
                    "owed"
                },
                name(c.creditor),
                c.owed,
                if pop.firm(c.debtor).is_some() {
                    "wages unpaid since"
                } else {
                    "lent"
                },
                day(c.made),
            );
            // What it was lent against, and on whose word (M7d slice CB).
            let pledge = match c.pledge {
                Some(Pledge::Field(_)) => ", against the use of a field".to_owned(),
                Some(Pledge::Guarantor(g)) => format!(
                    ", with {} standing guarantor",
                    super::people::household_name(sim, g)
                ),
                None if c.recourse.is_some() => ", for what it paid as guarantor".to_owned(),
                None => String::new(),
            };
            let base = format!("{base}{pledge}");
            let remedy = || {
                let mut out = String::new();
                if c.by_guarantor > 0.0 {
                    out.push_str(&format!("; its guarantor paid {:.0} kg", c.by_guarantor));
                }
                if matches!(c.pledge, Some(Pledge::Field(_))) && c.taken.is_some() {
                    out.push_str(if c.left() > 1e-3 {
                        "; the creditor works the field pledged until its crop pays the rest"
                    } else {
                        "; the crop of the field pledged paid it"
                    });
                }
                out
            };
            match c.standing {
                Standing::Open if c.extensions > 0 => {
                    format!("{base}, due on {} after time was granted", day(c.due))
                }
                Standing::Open => format!("{base}, due on {}", day(c.due)),
                Standing::Repaid => format!("{base}, and repaid it"),
                // A workshop that could not pay wages fails when they fall due (M7d slice CB).
                Standing::Defaulted if pop.firm(c.debtor).is_some() => format!(
                    "{base}, and failed when they fell due, {:.0} kg still owed",
                    c.left()
                ),
                Standing::Defaulted if c.left() > 1e-3 => format!(
                    "{base}, and had not repaid it when its days of grace ran out{}; \
                     {:.0} kg are still owed",
                    remedy(),
                    c.left()
                ),
                Standing::Defaulted => format!(
                    "{base}, and had not repaid it when its days of grace ran out{}",
                    remedy()
                ),
                Standing::Refused => format!("{base}, and refused to repay it{}", remedy()),
                Standing::Lapsed => format!("{base}, until a household of the two was no more"),
            }
        })
        .collect()
}

/// The coin of `settlement` (M7d slice CE, ADR-0027 §6), in words: where it is a polity's seat,
/// who keeps its mint and on what terms, each kind of piece the mint struck, metal waiting there to
/// be struck and what the common store holds; then what its households hold, by whose issue. None
/// where there is no coin. "Wren keeps the mint, striking 8 g pieces of copper at a 3 % fee";
/// "250 pieces of 8 g struck here on 3 May of year 2"; "Its households hold 243 pieces: 200 of
/// Ashford's issue, 43 of Bree's, taken by weight 10 % off".
pub fn coin_words(sim: &Sim, settlement: PermanentId) -> Vec<String> {
    let pop = &sim.people;
    let catalog = &sim.rules.catalog;
    let coinage = &pop.order.coinage;
    let day = |d: i64| civ_agents::polity::day_words(SimTime::from_minutes(d * 1440));
    let seat = |polity: PermanentId| {
        pop.polities
            .iter()
            .find(|p| p.id == polity)
            .and_then(|p| sim.land.settlements.iter().find(|s| s.id == p.settlement))
            .map_or_else(|| "another polity".to_owned(), |s| s.name.clone())
    };
    let mut out = Vec::new();
    let home = pop.polities.iter().find(|p| p.holds(Some(settlement)));
    if let Some(p) = home.filter(|p| p.settlement == settlement) {
        if let Some((keeper, law)) = p.minter()
            && let Some(m) = catalog
                .policies
                .get(usize::from(law.policy))
                .and_then(|d| d.mint)
        {
            let metal = catalog
                .recipes
                .get(m.recipe)
                .and_then(|r| r.inputs.first())
                .and_then(|&(g, _)| catalog.goods.get(g))
                .map_or_else(|| "metal".to_owned(), |g| g.name.to_lowercase());
            out.push(format!(
                "{} keeps the mint, striking {:.0} g pieces of {metal}{} at a {:.0} % fee",
                pop.name_of(keeper),
                m.mass_g,
                if m.fineness < 1.0 {
                    format!(", {:.0} % fine,", f64::from(m.fineness) * 100.0)
                } else {
                    String::new()
                },
                f64::from(m.fee) * 100.0
            ));
        }
        // Each kind of piece struck here, in the order first struck: pieces still in being, and
        // the first and last days struck.
        let mut kinds: Vec<((f32, f32), f64, i64, i64)> = Vec::new();
        for c in coinage.cohorts.iter().filter(|c| c.issuer == p.id) {
            match kinds.iter_mut().find(|k| k.0 == (c.mass_g, c.fineness)) {
                Some(k) => {
                    k.1 += c.count;
                    k.3 = k.3.max(c.day);
                }
                None => kinds.push(((c.mass_g, c.fineness), c.count, c.day, c.day)),
            }
        }
        for ((mass, fine), n, first, last) in kinds {
            let fine = if fine < 1.0 {
                format!(", {:.0} % fine,", f64::from(fine) * 100.0)
            } else {
                String::new()
            };
            out.push(if first == last {
                format!(
                    "{n:.0} pieces of {mass:.0} g{fine} struck here on {}",
                    day(first)
                )
            } else {
                format!(
                    "{n:.0} pieces of {mass:.0} g{fine} struck here from {} to {}",
                    day(first),
                    day(last)
                )
            });
        }
        let waiting: f64 = coinage
            .brought
            .iter()
            .filter(|b| b.polity == p.id)
            .map(|b| b.metal_kg)
            .sum();
        if waiting > 1e-3 {
            out.push(format!(
                "{waiting:.1} kg of metal waits at the mint to be struck"
            ));
        }
        let store = coinage.purse(p.id).map_or(0.0, |x| x.total());
        if store >= 0.5 {
            out.push(format!("The common store holds {store:.0} pieces"));
        }
    }
    // What its households hold, by whose issue, its own polity's first.
    let mut by: Vec<(PermanentId, f64)> = Vec::new();
    for (_, x) in pop.households.iter() {
        if x.settlement != Some(settlement) {
            continue;
        }
        let Some(purse) = coinage.purse(x.id) else {
            continue;
        };
        for &(id, n) in &purse.counts {
            let Some(c) = coinage.cohort(id) else {
                continue;
            };
            match by.iter_mut().find(|b| b.0 == c.issuer) {
                Some(b) => b.1 += n,
                None => by.push((c.issuer, n)),
            }
        }
    }
    let own = home.map(|p| p.id);
    by.sort_by_key(|b| (Some(b.0) != own, b.0));
    let total: f64 = by.iter().map(|b| b.1).sum();
    if total >= 0.5 {
        let discount = catalog
            .goods
            .iter()
            .filter_map(|g| g.money.as_ref())
            .find(|m| m.coin)
            .map_or(0.0, |m| m.foreign_discount);
        let parts: Vec<String> = by
            .iter()
            .filter(|b| b.1 >= 0.5)
            .map(|&(issuer, n)| {
                if Some(issuer) == own || discount <= 0.0 {
                    format!("{n:.0} of {}'s issue", seat(issuer))
                } else {
                    format!(
                        "{n:.0} of {}'s, taken by weight {:.0} % off",
                        seat(issuer),
                        discount * 100.0
                    )
                }
            })
            .collect();
        out.push(format!(
            "Its households hold {total:.0} pieces: {}",
            parts.join(", ")
        ));
    }
    out
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
