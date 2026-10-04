//! Workshops on the boundary (M3a slice J): the snapshot's workshop revision, the list of
//! workshops and a workshop's page (ADR-0006 §5). A workshop's record, wage and books are put in
//! words here; observers only show them.

use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::firm::{BookKind, Entry, Firm, WageOffer};
use civ_core::PermanentId;
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use super::markets::{amount, capitalized};
use super::people::{firm_name, household_name};
use super::{QueryError, response};
use crate::Sim;

/// A number that changes whenever a workshop opens, closes, makes, sells, pays or is reviewed
/// (0 = no workshop yet): a hash of each workshop's state and the latest line of its books.
pub fn firms_rev(sim: &Sim) -> u64 {
    let firms = &sim.people.firms;
    if firms.is_empty() {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    for f in firms {
        (f.id.get(), f.owner.get()).hash(&mut hasher);
        f.closed
            .map(|(t, why)| (t.minutes(), why as u8))
            .hash(&mut hasher);
        f.books.entries.len().hash(&mut hasher);
        f.books
            .entries
            .back()
            .map(|e| (e.at.minutes(), e.kind as u8, e.good, e.amount.to_bits()))
            .hash(&mut hasher);
        f.books.months.len().hash(&mut hasher);
        if let Some(m) = f.books.months.last() {
            (
                m.owner_h.to_bits(),
                m.hired_h.to_bits(),
                m.stock_h.to_bits(),
            )
                .hash(&mut hasher);
        }
        for o in &f.offers {
            (o.good, o.payment, o.price.to_bits(), o.units.to_bits()).hash(&mut hasher);
        }
        f.wage
            .map(|w| {
                (
                    w.pay,
                    w.per_hour.to_bits(),
                    w.hours.to_bits(),
                    w.taken.to_bits(),
                )
            })
            .hash(&mut hasher);
    }
    hasher.finish() | 1
}

/// How much of `good` there is, in words, or "goods" for a good the content no longer has.
fn of(sim: &Sim, good: u16, units: f64) -> String {
    sim.rules
        .catalog
        .goods
        .get(usize::from(good))
        .map_or_else(|| "goods".to_owned(), |d| amount(d, units))
}

/// What workshop `f` made and sold in its life, in words: "Made 3.0 sickles and sold a sickle.",
/// "Made nothing yet."
pub fn record_text(sim: &Sim, f: &Firm) -> String {
    let total = |kind: BookKind, good: u16| -> f64 {
        f.books
            .months
            .iter()
            .map(|m| f64::from(m.amount(kind, good)))
            .sum()
    };
    let parts: Vec<String> = f
        .lines
        .iter()
        .filter_map(|&g| {
            let (made, sold) = (total(BookKind::Made, g), total(BookKind::Sold, g));
            if made < 0.05 && sold < 0.05 {
                return None;
            }
            let sold = if sold < 0.05 {
                "none".to_owned()
            } else {
                of(sim, g, sold)
            };
            Some(format!("made {} and sold {sold}", of(sim, g, made)))
        })
        .collect();
    if parts.is_empty() {
        "Made nothing yet.".to_owned()
    } else {
        format!("{}.", capitalized(&parts.join("; ")))
    }
}

/// A workshop's wage in words: "Pays 0.90 kg of grain an hour; wants 6 more hours of work before
/// its next review."
pub fn wage_text(sim: &Sim, w: &WageOffer) -> String {
    let pay = of(sim, w.pay, f64::from(w.per_hour));
    let open = w.open_hours();
    if open >= 0.5 {
        format!("Pays {pay} an hour; wants {open:.0} more hours of work before its next review.")
    } else if w.hours > 0.0 {
        format!("Pays {pay} an hour; has hired all the work it wants until its next review.")
    } else {
        format!("Pays {pay} an hour when it hires; it wants no hired work now.")
    }
}

/// A line of a workshop's books in words: "Sold a sickle to Bran's household."
pub fn entry_text(sim: &Sim, e: &Entry) -> String {
    let what = of(sim, e.good, f64::from(e.amount));
    let other = e.other.map_or_else(
        || "another household".to_owned(),
        |id| household_name(sim, id),
    );
    match e.kind {
        BookKind::PutIn => format!("{} put in {what}.", capitalized(&other)),
        BookKind::Drawn => format!("{} drew out {what}.", capitalized(&other)),
        BookKind::Made => format!("Made {what}."),
        BookKind::Used => format!("Used {what}."),
        BookKind::Sold => format!("Sold {what} to {other}."),
        BookKind::Paid => format!("Was paid {what} by {other}."),
        BookKind::Wages => format!("Paid {other} {what} in wages."),
        BookKind::Lost => format!("Lost {what}."),
    }
}

fn settlement_name(sim: &Sim, id: Option<PermanentId>) -> &str {
    id.and_then(|id| sim.land.settlements.iter().find(|s| s.id == id))
        .map_or("", |s| s.name.as_str())
}

fn brief<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    f: &Firm,
) -> WIPOffset<wire::FirmBrief<'a>> {
    let name = fbb.create_string(&firm_name(sim, f.id));
    let owner_name = fbb.create_string(&household_name(sim, f.owner));
    let place = fbb.create_string(settlement_name(sim, f.settlement));
    let closed_why = f.closed.map(|(_, why)| fbb.create_string(why.text()));
    let lines = fbb.create_vector(&f.lines);
    let record = fbb.create_string(&record_text(sim, f));
    let hiring_h = f
        .wage
        .filter(|_| f.is_open())
        .map_or(0.0, |w| w.open_hours() as f32);
    wire::FirmBrief::create(
        fbb,
        &wire::FirmBriefArgs {
            id: f.id.get(),
            name: Some(name),
            owner: f.owner.get(),
            owner_name: Some(owner_name),
            settlement: f.settlement.map_or(0, PermanentId::get),
            settlement_name: Some(place),
            founded_minute: f.founded.minutes(),
            open: f.is_open(),
            closed_minute: f.closed.map_or(0, |(t, _)| t.minutes()),
            closed_why,
            lines: Some(lines),
            hiring_h,
            record: Some(record),
        },
    )
}

/// A `Response` with every workshop in brief: the open ones first, the newest first; then the
/// closed ones, the last to close first.
pub fn firms_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let mut firms: Vec<&Firm> = sim.people.firms.iter().collect();
    firms.sort_by_key(|f| {
        let since = f.closed.map_or(f.founded, |(t, _)| t);
        (f.closed.is_some(), Reverse(since), f.id)
    });
    let list: Vec<_> = firms.into_iter().map(|f| brief(&mut fbb, sim, f)).collect();
    let list = fbb.create_vector(&list);
    let body = wire::Firms::create(
        &mut fbb,
        &wire::FirmsArgs {
            rev: firms_rev(sim),
            firms: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Firms, body)
}

/// A `Response` with workshop `id`'s page: what it holds, its terms, its wage and its books.
pub fn firm_response(sim: &Sim, id: u64) -> Result<Vec<u8>, QueryError> {
    let f = PermanentId::from_raw(id)
        .and_then(|id| sim.people.firm(id))
        .ok_or_else(|| QueryError(format!("there is no workshop with id {id}")))?;
    let mut fbb = FlatBufferBuilder::new();
    let b = brief(&mut fbb, sim, f);
    let founder_name = fbb.create_string(&sim.people.name_of(f.founder));
    // As of its last settling: a workshop's stores are brought up to date whenever it makes,
    // sells, pays or is reviewed, at least weekly while it is open.
    let stores: Vec<wire::StoreLine> = f
        .stores
        .iter()
        .enumerate()
        .filter(|&(_, &units)| units > 1e-3)
        .map(|(g, &units)| wire::StoreLine::new(g as u16, units as f32))
        .collect();
    let stores = fbb.create_vector(&stores);
    let name = firm_name(sim, f.id);
    let offers: Vec<_> = f
        .offers
        .iter()
        .map(|o| {
            let name = fbb.create_string(&name);
            wire::OfferInfo::create(
                &mut fbb,
                &wire::OfferInfoArgs {
                    household: f.id.get(),
                    household_name: Some(name),
                    good: o.good,
                    payment: o.payment,
                    price: o.price,
                    units: o.units,
                    firm: true,
                },
            )
        })
        .collect();
    let offers = fbb.create_vector(&offers);
    let wage = f.wage.map(|w| {
        let text = fbb.create_string(&wage_text(sim, &w));
        wire::WageInfo::create(
            &mut fbb,
            &wire::WageInfoArgs {
                activity: w.activity,
                pay: w.pay,
                per_hour: w.per_hour,
                hour_h: w.hour_h,
                hours: w.hours,
                taken: w.taken,
                text: Some(text),
            },
        )
    });
    let entries: Vec<_> = f
        .books
        .entries
        .iter()
        .rev()
        .map(|e| {
            let text = fbb.create_string(&entry_text(sim, e));
            wire::BookEntryInfo::create(
                &mut fbb,
                &wire::BookEntryInfoArgs {
                    minute: e.at.minutes(),
                    kind: wire::BookKind(e.kind as u8),
                    good: e.good,
                    amount: e.amount,
                    other: e.other.map_or(0, PermanentId::get),
                    text: Some(text),
                },
            )
        })
        .collect();
    let entries = fbb.create_vector(&entries);
    let months: Vec<_> = f
        .books
        .months
        .iter()
        .map(|m| {
            let lines: Vec<wire::BookLine> = m
                .lines
                .iter()
                .map(|&(kind, good, amount)| {
                    wire::BookLine::new(amount, good, wire::BookKind(kind as u8))
                })
                .collect();
            let lines = fbb.create_vector(&lines);
            wire::MonthStatement::create(
                &mut fbb,
                &wire::MonthStatementArgs {
                    month: m.month,
                    lines: Some(lines),
                    owner_h: m.owner_h,
                    hired_h: m.hired_h,
                    income_h: m.income_h,
                    costs_h: m.costs_h,
                    stock_h: m.stock_h,
                },
            )
        })
        .collect();
    let months = fbb.create_vector(&months);
    let body = wire::FirmInfo::create(
        &mut fbb,
        &wire::FirmInfoArgs {
            brief: Some(b),
            founder: f.founder.get(),
            founder_name: Some(founder_name),
            owner_since_minute: f.owner_since.minutes(),
            last_sale_minute: f.last_sale.map_or(-1, |t| t.minutes()),
            stores: Some(stores),
            offers: Some(offers),
            wage,
            entries: Some(entries),
            months: Some(months),
        },
    );
    Ok(response(fbb, wire::ResponseBody::FirmInfo, body))
}
