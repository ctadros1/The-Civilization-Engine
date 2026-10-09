//! Markets on the boundary (M3a slice I): the snapshot's market revision and the response to a
//! markets query (ADR-0006 §4, §6). A market and each trade it remembers are put in words here;
//! observers only show them.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::ledger::{Channel, Trade};
use civ_agents::market::{Market, Offer};
use civ_agents::params::GoodDef;
use civ_core::PermanentId;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::people::{firm_name, household_name, place_name};
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
    // The convergence record and who is on the road to buy (wire 1.55).
    let conv = &pop.convergence;
    (conv.gaps.len(), conv.gaps.keys().next_back()).hash(&mut hasher);
    for (k, c) in &conv.carried {
        (k, c.trips).hash(&mut hasher);
    }
    let activities = &sim.rules.catalog.activities;
    for (_, p) in pop.people.iter() {
        if activities
            .get(usize::from(p.act.def))
            .is_some_and(|a| a.behavior == civ_agents::Behavior::Fetch)
        {
            (p.id.get(), p.act.started.minutes()).hash(&mut hasher);
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
    // A buyer from another settlement is named with it (M5b slice AP).
    let from = t
        .from
        .map_or_else(String::new, |s| format!(" of {}", place_name(sim, Some(s))));
    format!(
        "{} sold {} to {}{from} for {}.",
        capitalized(&seller_name(sim, t.seller)),
        of(t.good, t.units),
        household_name(sim, t.buyer),
        of(t.payment, t.paid)
    )
}

/// What household `household` believes other settlements' sellers offer (ADR-0019 §1), a line a
/// seller and good, by settlement: "At Elmhollow, Bran's household: a sickle for 2.3 kg of grain
/// or 3.7 kg of provisions, 8.5 sickles to be had; seen 3 days ago", "…: none left of the sickle
/// it offered, told by Ada yesterday".
pub fn reports_words(sim: &Sim, household: PermanentId) -> Vec<String> {
    use civ_agents::reports::{PriceReport, ReportHow};
    let goods = &sim.rules.catalog.goods;
    let today = sim.now().day_index();
    let mut by: BTreeMap<(String, u16, PermanentId), Vec<&PriceReport>> = BTreeMap::new();
    for r in sim.people.reports.of(household) {
        by.entry((place_name(sim, Some(r.market)), r.good, r.seller))
            .or_default()
            .push(r);
    }
    by.into_iter()
        .filter_map(|((place, g, seller), rs)| {
            let good = goods.get(usize::from(g))?;
            let newest = rs
                .iter()
                .copied()
                .max_by_key(|r| (r.day, std::cmp::Reverse(r.how)))?;
            let offered: Vec<&PriceReport> = rs.iter().copied().filter(|r| r.units > 0.0).collect();
            let what = if offered.is_empty() {
                format!("none left of the {} it offered", good.name.to_lowercase())
            } else {
                let terms: Vec<String> = offered
                    .iter()
                    .filter_map(|r| {
                        goods
                            .get(usize::from(r.payment))
                            .map(|d| amount(d, f64::from(r.price)))
                    })
                    .collect();
                let terms = match terms.split_last() {
                    Some((last, [])) => last.clone(),
                    Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
                    None => String::new(),
                };
                let units = offered
                    .iter()
                    .map(|r| f64::from(r.units))
                    .fold(0.0, f64::max);
                if good.tool.is_some() {
                    format!(
                        "{} for {terms}, {} to be had",
                        amount(good, 1.0),
                        amount(good, units)
                    )
                } else {
                    format!(
                        "{} for {terms} a kg, {} to be had",
                        good.name.to_lowercase(),
                        amount(good, units)
                    )
                }
            };
            let how = match (newest.how, newest.from) {
                (ReportHow::Told, Some(p)) => format!("told by {}", sim.people.name_of(p)),
                (ReportHow::Told, None) => "told".to_owned(),
                (ReportHow::Seen, _) => "seen".to_owned(),
            };
            let when = match today - newest.day {
                i64::MIN..=0 => "today".to_owned(),
                1 => "yesterday".to_owned(),
                n => format!("{n} days ago"),
            };
            Some(format!(
                "At {place}, {}: {what}; {how} {when}",
                seller_name(sim, seller)
            ))
        })
        .collect()
}

/// What people of other settlements bought in settlement `s`'s market last year and this year so
/// far, in words (M5b slice AP): "in year 3 so far: 4 purchases by people of Oakholt"; empty when
/// none did.
pub fn outsiders_words(sim: &Sim, s: PermanentId) -> String {
    let this = sim.now().date().year - 1;
    let mut parts = Vec::new();
    for year in [this - 1, this] {
        let lines: Vec<String> = sim
            .people
            .contacts
            .of_year(year)
            .filter(|&(_, to, c)| to == s && c.bought > 0)
            .map(|(from, _, c)| {
                let n = if c.bought == 1 {
                    "1 purchase".to_owned()
                } else {
                    format!("{} purchases", c.bought)
                };
                format!("{n} by people of {}", place_name(sim, Some(from)))
            })
            .collect();
        if !lines.is_empty() {
            let when = if year == this {
                format!("in year {} so far", year + 1)
            } else {
                format!("in year {}", year + 1)
            };
            parts.push(format!("{when}: {}", lines.join("; ")));
        }
    }
    parts.join(". ")
}

/// Hours in running text: two places below one hour, one above.
fn hours(x: f32) -> String {
    if x < 1.0 {
        format!("{x:.2}")
    } else {
        format!("{x:.1}")
    }
}

/// A month on record in running text, against the present: "last month", "this month", or "in
/// month 4 of year 3".
fn month_words(sim: &Sim, month: u32) -> String {
    let now = civ_agents::market::month_of(sim.now());
    if month == now {
        "this month".to_owned()
    } else if month + 1 == now {
        "last month".to_owned()
    } else {
        format!("in month {} of year {}", month % 12 + 1, month / 12 + 1)
    }
}

/// Trade between settlement `s` and each other settlement it has any month on record with (M5b
/// slice AQ, ADR-0019 §7), a line each, from the latest such month: the asks of the goods offered
/// in both, here and there, and how far apart they stand; and what people of each carried home
/// from the other's market: "With Elmhollow last month: a sickle 4.2 hours here and 3.1 there (30
/// points apart); people of here carried home 3.0 sickles from there in 4 trips".
pub fn between_words(sim: &Sim, s: PermanentId) -> Vec<String> {
    let conv = &sim.people.convergence;
    let goods = &sim.rules.catalog.goods;
    let mut latest: BTreeMap<PermanentId, u32> = BTreeMap::new();
    let pairs = conv
        .gaps
        .keys()
        .copied()
        .chain(conv.carried.keys().copied());
    for (month, x, y) in pairs {
        let other = if x == s {
            y
        } else if y == s {
            x
        } else {
            continue;
        };
        let m = latest.entry(other).or_insert(month);
        *m = (*m).max(month);
    }
    latest
        .into_iter()
        .map(|(other, month)| {
            let first = s < other;
            let there = place_name(sim, Some(other));
            let mut parts = Vec::new();
            let pair = if first { (s, other) } else { (other, s) };
            if let Some(list) = conv.gaps.get(&(month, pair.0, pair.1)) {
                let asks: Vec<String> = list
                    .iter()
                    .filter_map(|g| {
                        let d = goods.get(usize::from(g.good))?;
                        let (here, away) = if first {
                            (g.ask_h[0], g.ask_h[1])
                        } else {
                            (g.ask_h[1], g.ask_h[0])
                        };
                        let gap = 100.0 * (f64::from(here) / f64::from(away)).ln().abs();
                        let (what, per) = if d.tool.is_some() {
                            (amount(d, 1.0), "")
                        } else {
                            (d.name.to_lowercase(), " a kg")
                        };
                        Some(format!(
                            "{what} {} hours{per} here and {} there ({gap:.0} points apart)",
                            hours(here),
                            hours(away)
                        ))
                    })
                    .collect();
                if !asks.is_empty() {
                    parts.push(asks.join(", "));
                }
            }
            for (from, to, who, place) in [
                (s, other, "here".to_owned(), "there"),
                (other, s, there.clone(), "here"),
            ] {
                let Some(c) = conv.carried.get(&(month, from, to)) else {
                    continue;
                };
                let trips = if c.trips == 1 {
                    "1 trip".to_owned()
                } else {
                    format!("{} trips", c.trips)
                };
                let what: Vec<String> = c
                    .goods
                    .iter()
                    .filter_map(|&(g, u)| {
                        goods.get(usize::from(g)).map(|d| amount(d, f64::from(u)))
                    })
                    .collect();
                parts.push(if what.is_empty() {
                    format!(
                        "people of {who} went to buy {place} in {trips} and carried nothing home"
                    )
                } else {
                    format!(
                        "people of {who} carried home {} from {place} in {trips}",
                        what.join(", ")
                    )
                });
            }
            format!(
                "With {there} {}: {}",
                month_words(sim, month),
                if parts.is_empty() {
                    "nothing offered in both".to_owned()
                } else {
                    parts.join("; ")
                }
            )
        })
        .collect()
}

/// Who is on the road to buy in settlement `s`'s market today (M5b slice AQ: caravans as a view,
/// never an actor), by the settlement they live in; those of one settlement on the same road on
/// the same day go together: "On the road to buy here today: 3 people of Oakholt, together; 1
/// person of Ashford". Empty when nobody is.
pub fn on_the_way_words(sim: &Sim, s: PermanentId) -> String {
    use civ_agents::Behavior;
    use civ_agents::person::Target;
    let pop = &sim.people;
    let activities = &sim.rules.catalog.activities;
    let today = sim.now().day_index();
    let mut by: BTreeMap<PermanentId, u32> = BTreeMap::new();
    for (_, p) in pop.people.iter() {
        let fetching = activities
            .get(usize::from(p.act.def))
            .is_some_and(|a| a.behavior == Behavior::Fetch);
        if !fetching || p.act.started.day_index() != today {
            continue;
        }
        let market = match p.act.target {
            Target::Household(h) => pop.household(h).and_then(|x| x.settlement),
            Target::Firm(f) => pop.firm(f).and_then(|f| f.settlement),
            _ => None,
        };
        if market != Some(s) {
            continue;
        }
        if let Some(home) = pop.household(p.household).and_then(|x| x.settlement) {
            *by.entry(home).or_default() += 1;
        }
    }
    if by.is_empty() {
        return String::new();
    }
    let groups: Vec<String> = by
        .into_iter()
        .map(|(home, n)| {
            let place = place_name(sim, Some(home));
            if n == 1 {
                format!("1 person of {place}")
            } else {
                format!("{n} people of {place}, together")
            }
        })
        .collect();
    format!("On the road to buy here today: {}", groups.join("; "))
}

/// The errand household `household` means to run, in words (M5b slice AQ, ADR-0019 §6): "Their
/// household means to fetch 3.6 sickles from Bran's household at Elmhollow to sell at home;
/// planned 2 days ago". Empty when it means none.
pub fn errand_words(sim: &Sim, household: PermanentId) -> String {
    let Some(e) = sim.people.reports.errands.get(&household) else {
        return String::new();
    };
    let Some(d) = sim.rules.catalog.goods.get(usize::from(e.good)) else {
        return String::new();
    };
    let when = match sim.now().day_index() - e.day {
        i64::MIN..=0 => "today".to_owned(),
        1 => "yesterday".to_owned(),
        n => format!("{n} days ago"),
    };
    format!(
        "Their household means to fetch {} from {} at {} to sell at home; planned {when}",
        amount(d, f64::from(e.units)),
        seller_name(sim, e.seller),
        place_name(sim, Some(e.market))
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
                            from: t.from.map_or(0, PermanentId::get),
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
            let outsiders = fbb.create_string(&outsiders_words(sim, settlement));
            let between: Vec<_> = between_words(sim, settlement)
                .iter()
                .map(|w| fbb.create_string(w))
                .collect();
            let between = fbb.create_vector(&between);
            let on_the_way = fbb.create_string(&on_the_way_words(sim, settlement));
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
                    outsiders: Some(outsiders),
                    between: Some(between),
                    on_the_way: Some(on_the_way),
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
