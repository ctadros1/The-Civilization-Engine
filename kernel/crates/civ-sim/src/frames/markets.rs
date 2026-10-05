//! Markets on the boundary (M3a slice I): the snapshot's market revision and the response to a
//! markets query (ADR-0006 §4, §6). A market and each trade it remembers are put in words here;
//! observers only show them.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::ledger::{Channel, Trade};
use civ_agents::market::{Market, Offer};
use civ_agents::params::GoodDef;
use civ_core::PermanentId;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::people::{firm_name, household_name};
use super::response;
use crate::Sim;

/// A household or workshop that offers goods, with its terms.
struct Seller<'a> {
    id: PermanentId,
    settlement: Option<PermanentId>,
    firm: bool,
    offers: &'a [Offer],
}

/// Every household and open workshop that offers anything, by id.
fn sellers(sim: &Sim) -> Vec<Seller<'_>> {
    let pop = &sim.people;
    let mut out: Vec<Seller<'_>> = pop
        .households
        .iter()
        .map(|(_, h)| Seller {
            id: h.id,
            settlement: h.settlement,
            firm: false,
            offers: &h.offers,
        })
        .chain(pop.firms.iter().filter(|f| f.is_open()).map(|f| Seller {
            id: f.id,
            settlement: f.settlement,
            firm: true,
            offers: &f.offers,
        }))
        .filter(|s| !s.offers.is_empty())
        .collect();
    out.sort_by_key(|s| s.id);
    out
}

/// A seller's name: "Ada's household", "Wren's sickle workshop".
fn seller_name(sim: &Sim, id: PermanentId) -> String {
    if sim.people.firm(id).is_some() {
        firm_name(sim, id)
    } else {
        household_name(sim, id)
    }
}

/// A number that changes whenever a household or workshop posts terms, a trade is made or a want
/// goes unmet (0 = nothing offered and no market yet): a hash of every offer and of each market's
/// tallies.
pub fn markets_rev(sim: &Sim) -> u64 {
    let pop = &sim.people;
    let offering = sellers(sim);
    if offering.is_empty() && pop.markets.is_empty() {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    for s in offering {
        s.id.get().hash(&mut hasher);
        for o in s.offers {
            (o.good, o.payment, o.price.to_bits(), o.units.to_bits()).hash(&mut hasher);
        }
    }
    for m in &pop.markets {
        (
            m.settlement.get(),
            m.day,
            m.trades.to_bits(),
            m.recent.len(),
        )
            .hash(&mut hasher);
        m.recent.back().map(|t| t.at.minutes()).hash(&mut hasher);
        for tally in [&m.sold, &m.unmet] {
            for x in tally {
                x.to_bits().hash(&mut hasher);
            }
        }
    }
    hasher.finish() | 1
}

/// An amount of a good in running text: "a sickle", "1.5 sickles", "12 kg of grain", "0.35 kg
/// of grain".
pub fn amount(good: &GoodDef, units: f64) -> String {
    let name = good.name.to_lowercase();
    if good.tool.is_some() {
        if (units - 1.0).abs() < 0.05 {
            let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
                "an"
            } else {
                "a"
            };
            format!("{article} {name}")
        } else {
            format!("{units:.1} {name}s")
        }
    } else if units < 1.0 {
        format!("{units:.2} kg of {name}")
    } else if units < 10.0 {
        format!("{units:.1} kg of {name}")
    } else {
        format!("{units:.0} kg of {name}")
    }
}

/// `s` with its first letter in capitals.
pub fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// A trade in words: "Ada's household sold a sickle to Bran's household for 12 kg of grain.",
/// "Wren's sickle workshop sold a sickle to …".
pub fn trade_text(sim: &Sim, t: &Trade) -> String {
    let goods = &sim.rules.catalog.goods;
    let of = |g: u16, units: f32| {
        goods
            .get(usize::from(g))
            .map_or_else(|| "goods".to_owned(), |d| amount(d, f64::from(units)))
    };
    format!(
        "{} sold {} to {} for {}.",
        capitalized(&seller_name(sim, t.seller)),
        of(t.good, t.units),
        household_name(sim, t.buyer),
        of(t.payment, t.paid)
    )
}

/// A market in words: its money if it has one, or how far it is from one.
pub fn summary(sim: &Sim, market: Option<&Market>) -> String {
    let mp = &sim.rules.people.market;
    let goods = &sim.rules.catalog.goods;
    let Some(m) = market.filter(|m| m.trades >= 0.5) else {
        return "Nothing has changed hands lately.".to_owned();
    };
    let name = |g: usize| {
        goods
            .get(g)
            .map_or("a good".to_owned(), |d| d.name.to_lowercase())
    };
    let shares = m.acceptance();
    let top = shares
        .iter()
        .copied()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
        .filter(|&(_, s)| s > 0.0);
    match (m.money(mp.money_share, mp.money_min_trades), top) {
        (Some(g), _) => format!(
            "{} is money here: it settles {:.0}% of what is paid.",
            capitalized(&name(g)),
            shares[g] * 100.0
        ),
        (None, Some((g, s))) if s >= mp.money_share => format!(
            "Barter: {} settles {:.0}% of what is paid, over too few trades yet to be money.",
            name(g),
            s * 100.0
        ),
        (None, Some((g, s))) => format!(
            "Barter: no good settles most of what is paid; {} settles {:.0}%.",
            name(g),
            s * 100.0
        ),
        (None, None) => "Barter.".to_owned(),
    }
}

/// The settlements with a market or a household or workshop offering anything, by id.
fn trading_settlements(sim: &Sim, sellers: &[Seller<'_>]) -> Vec<PermanentId> {
    let mut out: Vec<PermanentId> = sellers
        .iter()
        .filter_map(|s| s.settlement)
        .chain(sim.people.markets.iter().map(|m| m.settlement))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// A `Response` with every settlement's market.
pub fn markets_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let goods = &sim.rules.catalog.goods;
    let mp = &sim.rules.people.market;
    let everyone = sellers(sim);
    let list: Vec<_> = trading_settlements(sim, &everyone)
        .into_iter()
        .map(|settlement| {
            let market = sim.people.market(settlement);
            let sellers: Vec<&Seller<'_>> = everyone
                .iter()
                .filter(|s| s.settlement == Some(settlement))
                .collect();
            // Per good: units offered (a seller offers the same units for each payment it
            // takes), by how many sellers, and how many of those are workshops.
            let mut offered = vec![(0.0f64, 0u16, 0u16); goods.len()];
            for s in &sellers {
                let mut most = vec![0.0f64; goods.len()];
                for o in s.offers {
                    if let Some(x) = most.get_mut(usize::from(o.good)) {
                        *x = x.max(f64::from(o.units));
                    }
                }
                for (g, units) in most.into_iter().enumerate() {
                    if units > 0.0 {
                        offered[g].0 += units;
                        offered[g].1 += 1;
                        offered[g].2 += u16::from(s.firm);
                    }
                }
            }
            let shares = market.map(Market::acceptance).unwrap_or_default();
            let tally =
                |v: Option<&Vec<f64>>, g: usize| v.and_then(|v| v.get(g)).copied().unwrap_or(0.0);
            let lines: Vec<_> = (0..goods.len())
                .filter_map(|g| {
                    let sold = tally(market.map(|m| &m.sold), g);
                    let unmet = tally(market.map(|m| &m.unmet), g);
                    let acceptance = shares.get(g).copied().unwrap_or(0.0);
                    let last = market.and_then(|m| m.last.get(g).copied().flatten());
                    let shown = offered[g].1 > 0
                        || sold > 1e-3
                        || unmet > 1e-3
                        || acceptance > 1e-3
                        || last.is_some();
                    shown.then(|| {
                        wire::MarketGood::create(
                            &mut fbb,
                            &wire::MarketGoodArgs {
                                good: g as u16,
                                offered: offered[g].0 as f32,
                                sellers: offered[g].1,
                                sold: sold as f32,
                                unmet: unmet as f32,
                                unmet_worth_h: market.and_then(|m| m.unmet_worth(g)).unwrap_or(0.0)
                                    as f32,
                                acceptance: acceptance as f32,
                                last_payment: last.map_or(-1, |(p, _)| i32::from(p)),
                                last_price: last.map_or(0.0, |(_, price)| price),
                                workshops: offered[g].2,
                            },
                        )
                    })
                })
                .collect();
            let lines = fbb.create_vector(&lines);
            let mut rows: Vec<(&Seller<'_>, &Offer)> = sellers
                .iter()
                .flat_map(|s| s.offers.iter().map(move |o| (*s, o)))
                .collect();
            rows.sort_by(|a, b| {
                (a.1.good, a.1.payment)
                    .cmp(&(b.1.good, b.1.payment))
                    .then(a.1.price.total_cmp(&b.1.price))
                    .then(a.0.id.cmp(&b.0.id))
            });
            let offers: Vec<_> = rows
                .into_iter()
                .map(|(s, o)| {
                    let name = fbb.create_string(&seller_name(sim, s.id));
                    wire::OfferInfo::create(
                        &mut fbb,
                        &wire::OfferInfoArgs {
                            household: s.id.get(),
                            household_name: Some(name),
                            good: o.good,
                            payment: o.payment,
                            price: o.price,
                            units: o.units,
                            firm: s.firm,
                        },
                    )
                })
                .collect();
            let offers = fbb.create_vector(&offers);
            let recent: Vec<_> = market
                .map(|m| m.recent.iter().rev().collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .map(|t| {
                    let text = fbb.create_string(&trade_text(sim, t));
                    wire::TradeInfo::create(
                        &mut fbb,
                        &wire::TradeInfoArgs {
                            minute: t.at.minutes(),
                            seller: t.seller.get(),
                            buyer: t.buyer.get(),
                            good: t.good,
                            units: t.units,
                            payment: t.payment,
                            paid: t.paid,
                            sale: t.channel == Channel::Sale,
                            text: Some(text),
                            seller_firm: sim.people.firm(t.seller).is_some(),
                        },
                    )
                })
                .collect();
            let recent = fbb.create_vector(&recent);
            let history: Vec<wire::MonthOfTrade> = market
                .map(|m| {
                    m.history
                        .iter()
                        .map(|h| {
                            wire::MonthOfTrade::new(h.month, h.trades, h.units, h.paid_h, h.good)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let history = fbb.create_vector(&history);
            let name = sim
                .land
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map_or("", |s| s.name.as_str());
            let name = fbb.create_string(name);
            let summary = fbb.create_string(&summary(sim, market));
            let money = market
                .and_then(|m| m.money(mp.money_share, mp.money_min_trades))
                .map_or(-1, |g| g as i32);
            wire::MarketInfo::create(
                &mut fbb,
                &wire::MarketInfoArgs {
                    settlement: settlement.get(),
                    settlement_name: Some(name),
                    money,
                    summary: Some(summary),
                    trades: market.map_or(0.0, |m| m.trades) as f32,
                    memory_days: mp.memory_days as f32,
                    goods: Some(lines),
                    offers: Some(offers),
                    recent: Some(recent),
                    history: Some(history),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let body = wire::Markets::create(
        &mut fbb,
        &wire::MarketsArgs {
            rev: markets_rev(sim),
            markets: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Markets, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_agents::params::{Eaten, GoodUse, ToolDef};

    fn good(name: &str, tool: bool) -> GoodDef {
        GoodDef {
            id: format!("core:good/{}", name.to_lowercase()),
            name: name.into(),
            purpose: if tool { GoodUse::Tool } else { GoodUse::Food },
            kcal_per_kg: 0.0,
            half_life_days: 0.0,
            sheltered_half_life_days: 0.0,
            eaten: Eaten::Never,
            shared: false,
            reserve_for: None,
            tool: tool.then_some(ToolDef {
                life_h: 300.0,
                per_worker: 1.0,
                fixed: false,
            }),
            timber: None,
            store: None,
        }
    }

    #[test]
    fn amounts_read_as_people_would_say_them() {
        assert_eq!(amount(&good("Sickle", true), 1.0), "a sickle");
        assert_eq!(amount(&good("Axe", true), 0.98), "an axe");
        assert_eq!(amount(&good("Hoe", true), 1.5), "1.5 hoes");
        assert_eq!(amount(&good("Grain", false), 12.3), "12 kg of grain");
        assert_eq!(amount(&good("Flour", false), 2.25), "2.2 kg of flour");
        assert_eq!(amount(&good("Grain", false), 0.355), "0.35 kg of grain");
    }
}
