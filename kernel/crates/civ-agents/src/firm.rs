//! Firms (slice J, ADR-0006 §5): an enterprise account with stores of its own, apart from the
//! household that owns it (research 08-08 §1.1: an early enterprise is an activity account
//! inside a household). In M3a every firm is a household workshop: one household owns it, its
//! members work for it unpaid and put in and draw out goods through the `owner` channel, and it
//! keeps books (08-08 §5.3: inventories, what came in, what went out, owner claims).

use std::collections::VecDeque;

use civ_core::{PermanentId, SimTime};

use crate::market::{Offer, month_of};
use crate::params::GoodDef;
use crate::person::Flows;

/// Why a firm closed (ADR-0006 §5; research 08-08 §1.8: closure is not necessarily failure).
/// Codes are part of saves and the boundary: never renumber.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Exit {
    /// Its owners gave it up: it sold nothing for a long time (a voluntary exit).
    Idle = 1,
    /// Its household is no more and nobody took it on, or it left the valley (the owner's line
    /// ended).
    OwnerGone = 2,
    /// It could not pay what it owed (financial failure).
    Failure = 3,
}

impl Exit {
    /// Every cause, in code order.
    pub const ALL: [Exit; 3] = [Exit::Idle, Exit::OwnerGone, Exit::Failure];

    /// The cause with this code.
    pub fn from_code(code: u8) -> Option<Exit> {
        Exit::ALL.into_iter().find(|e| *e as u8 == code)
    }

    /// Why, in running text: "it sold nothing for months".
    pub fn text(self) -> &'static str {
        match self {
            Exit::Idle => "it sold nothing for months",
            Exit::OwnerGone => "its household was no more",
            Exit::Failure => "it could not pay what it owed",
        }
    }
}

/// What a line of a firm's books records. Codes are part of saves and the boundary: never
/// renumber.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum BookKind {
    /// Its owners put goods in.
    PutIn = 1,
    /// Its owners drew goods out.
    Drawn = 2,
    /// Made by work done for it.
    Made = 3,
    /// Put into its recipes.
    Used = 4,
    /// Sold to a buyer.
    Sold = 5,
    /// Taken in payment.
    Paid = 6,
    /// Paid out as wages.
    Wages = 7,
    /// Spoiled in its store, or taken away when it closed.
    Lost = 8,
}

impl BookKind {
    /// Every kind, in code order.
    pub const ALL: [BookKind; 8] = [
        BookKind::PutIn,
        BookKind::Drawn,
        BookKind::Made,
        BookKind::Used,
        BookKind::Sold,
        BookKind::Paid,
        BookKind::Wages,
        BookKind::Lost,
    ];

    /// The kind with this code.
    pub fn from_code(code: u8) -> Option<BookKind> {
        BookKind::ALL.into_iter().find(|k| *k as u8 == code)
    }

    /// Plain-English label.
    pub fn label(self) -> &'static str {
        match self {
            BookKind::PutIn => "put in by its owners",
            BookKind::Drawn => "drawn by its owners",
            BookKind::Made => "made",
            BookKind::Used => "used",
            BookKind::Sold => "sold",
            BookKind::Paid => "taken in payment",
            BookKind::Wages => "paid in wages",
            BookKind::Lost => "lost",
        }
    }
}

/// A line of a firm's books: `amount` of `good` (in its unit) moved by `kind`, with `other`
/// the household on the other side, if any.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    /// When.
    pub at: SimTime,
    /// What.
    pub kind: BookKind,
    /// The good, by index in the catalog's goods.
    pub good: u16,
    /// How much, in the good's unit.
    pub amount: f32,
    /// The household on the other side: the buyer, the owner, the worker.
    pub other: Option<PermanentId>,
}

/// A month of a firm's books (ADR-0006 §5: monthly statements, kept for the firm's life).
/// Quantities by kind and good, and values in hours of the owners' own work (what the goods
/// cost them to get themselves), since a settlement need not have a money.
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    /// Months since the calendar's origin (see [`month_of`]).
    pub month: u32,
    /// Amounts by kind and good, in kind-then-good order.
    pub lines: Vec<(BookKind, u16, f32)>,
    /// Hours of work done for it by its owners' household, and by people it hired.
    pub owner_h: f32,
    /// See `owner_h`.
    pub hired_h: f32,
    /// What its payments were worth, hours of its owners' work.
    pub income_h: f32,
    /// What its inputs and wages were worth, hours of its owners' work.
    pub costs_h: f32,
    /// Its stock as the month ended (or as it stands, for the month under way), valued the same
    /// way.
    pub stock_h: f32,
}

impl Statement {
    fn new(month: u32) -> Statement {
        Statement {
            month,
            lines: Vec::new(),
            owner_h: 0.0,
            hired_h: 0.0,
            income_h: 0.0,
            costs_h: 0.0,
            stock_h: 0.0,
        }
    }

    /// The amount of `good` moved by `kind` this month.
    pub fn amount(&self, kind: BookKind, good: u16) -> f32 {
        self.lines
            .iter()
            .find(|&&(k, g, _)| k == kind && g == good)
            .map_or(0.0, |&(_, _, a)| a)
    }
}

/// A firm's books: the latest entries and every month's statement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Books {
    /// The latest entries, oldest first.
    pub entries: VecDeque<Entry>,
    /// Every month of its life, oldest first.
    pub months: Vec<Statement>,
}

impl Books {
    /// The statement of the month of `at`, begun if it is new.
    pub fn month_mut(&mut self, at: SimTime) -> &mut Statement {
        let month = month_of(at);
        if self.months.last().is_none_or(|s| s.month != month) {
            self.months.push(Statement::new(month));
        }
        let last = self.months.len() - 1;
        &mut self.months[last]
    }

    /// Records an entry, keeping the latest `keep`, and adds it to its month: payments to the
    /// income, inputs used and wages to the costs, at `worth_h` hours of the owners' work.
    pub fn record(&mut self, entry: Entry, worth_h: f64, keep: usize) {
        if !(entry.amount > 0.0 && entry.amount.is_finite()) {
            return;
        }
        let month = self.month_mut(entry.at);
        match month
            .lines
            .iter_mut()
            .find(|(k, g, _)| *k == entry.kind && *g == entry.good)
        {
            Some(line) => line.2 += entry.amount,
            None => {
                month.lines.push((entry.kind, entry.good, entry.amount));
                month.lines.sort_by_key(|&(k, g, _)| (k, g));
            }
        }
        let worth = worth_h.max(0.0) as f32;
        match entry.kind {
            BookKind::Paid => month.income_h += worth,
            BookKind::Used | BookKind::Wages => month.costs_h += worth,
            _ => {}
        }
        self.entries.push_back(entry);
        while self.entries.len() > keep {
            self.entries.pop_front();
        }
    }

    /// Adds `hours` of work done for the firm in the month of `at`, by its owners' household or
    /// hired.
    pub fn worked(&mut self, at: SimTime, hours: f64, hired: bool) {
        let month = self.month_mut(at);
        if hired {
            month.hired_h += hours.max(0.0) as f32;
        } else {
            month.owner_h += hours.max(0.0) as f32;
        }
    }
}

/// What a workshop pays for work, and how much of it it wants (ADR-0006 §5: wage offers;
/// research 08-10 §5.3: a posted offer workers take or leave).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WageOffer {
    /// The make activity worked, by index in the catalog's activities.
    pub activity: u16,
    /// What it pays in, by index in the catalog's goods.
    pub pay: u16,
    /// Units of that for an hour of work.
    pub per_hour: f32,
    /// What an hour of work is worth to its owners, hours of their own work: the wage.
    pub hour_h: f32,
    /// Hours of work it wants until its next review.
    pub hours: f32,
    /// Hours taken of those so far.
    pub taken: f32,
    /// The day its wage was last reviewed.
    pub reviewed: i64,
}

impl WageOffer {
    /// Hours of work it still wants.
    pub fn open_hours(&self) -> f64 {
        f64::from(self.hours - self.taken).max(0.0)
    }
}

/// A firm (ADR-0006 §5).
#[derive(Clone, Debug, PartialEq)]
pub struct Firm {
    /// Permanent id.
    pub id: PermanentId,
    /// The household that owns it.
    pub owner: PermanentId,
    /// Since when that household has owned it.
    pub owner_since: SimTime,
    /// Who founded it.
    pub founder: PermanentId,
    /// Its settlement.
    pub settlement: Option<PermanentId>,
    /// When it was founded.
    pub founded: SimTime,
    /// When it closed and why.
    pub closed: Option<(SimTime, Exit)>,
    /// The goods it makes to sell, by index in the catalog's goods.
    pub lines: Vec<u16>,
    /// What it holds, by good: its stock and what it was paid.
    pub stores: Vec<f64>,
    /// When its stores were last brought up to date.
    pub stores_at: SimTime,
    /// What it offers, and on what terms.
    pub offers: Vec<Offer>,
    /// When it last sold anything.
    pub last_sale: Option<SimTime>,
    /// What it pays for work, when it wants any.
    pub wage: Option<WageOffer>,
    /// Its books.
    pub books: Books,
    /// What became of its goods since the counters began (not saved).
    pub flows: Flows,
}

impl Firm {
    /// A new firm of household `owner`, founded by `founder` at `now` to make `line`.
    pub fn new(
        id: PermanentId,
        owner: PermanentId,
        founder: PermanentId,
        settlement: Option<PermanentId>,
        line: u16,
        goods: usize,
        now: SimTime,
    ) -> Firm {
        Firm {
            id,
            owner,
            owner_since: now,
            founder,
            settlement,
            founded: now,
            closed: None,
            lines: vec![line],
            stores: vec![0.0; goods],
            stores_at: now,
            offers: Vec::new(),
            last_sale: None,
            wage: None,
            books: Books::default(),
            flows: Flows::default(),
        }
    }

    /// Open, not closed.
    pub fn is_open(&self) -> bool {
        self.closed.is_none()
    }

    /// Brings its stores up to `t`: goods spoil by their half-lives, their sheltered ones under
    /// its owners' roof. A firm burns no firewood.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[GoodDef], sheltered: bool) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            let half_life = if sheltered && d.sheltered_half_life_days > 0.0 {
                d.sheltered_half_life_days
            } else {
                d.half_life_days
            };
            if half_life > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / half_life);
                self.flows.add(crate::person::Flow::Spoiled, g, *kg - after);
                *kg = after;
            }
        }
        self.stores_at = t;
    }

    /// Its name in words, from its founder's given name and what it makes: "Wren's sickle
    /// workshop".
    pub fn name(&self, founder: &str, goods: &[GoodDef]) -> String {
        let what = self
            .lines
            .first()
            .and_then(|&g| goods.get(usize::from(g)))
            .map_or_else(String::new, |d| format!("{} ", d.name.to_lowercase()));
        format!("{founder}'s {what}workshop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn entry(at: SimTime, kind: BookKind, good: u16, amount: f32) -> Entry {
        Entry {
            at,
            kind,
            good,
            amount,
            other: Some(id(2)),
        }
    }

    #[test]
    fn books_keep_the_latest_entries_and_every_month() {
        let mut books = Books::default();
        let march = SimTime::from_date(1, 3, 5, 9, 0).expect("a date");
        let april = SimTime::from_date(1, 4, 2, 9, 0).expect("a date");
        books.record(entry(march, BookKind::Made, 4, 1.0), 0.0, 3);
        books.record(entry(march, BookKind::Used, 1, 0.3), 0.6, 3);
        books.record(entry(march, BookKind::Made, 4, 1.5), 0.0, 3);
        books.record(entry(april, BookKind::Sold, 4, 1.0), 0.0, 3);
        books.record(entry(april, BookKind::Paid, 0, 12.0), 7.5, 3);
        books.worked(april, 3.0, false);
        // Nothing, or nothing real, is not recorded.
        books.record(entry(april, BookKind::Paid, 0, 0.0), 1.0, 3);
        books.record(entry(april, BookKind::Paid, 0, f32::NAN), 1.0, 3);
        assert_eq!(books.entries.len(), 3, "the latest only");
        assert_eq!(books.months.len(), 2);
        let (m, a) = (&books.months[0], &books.months[1]);
        assert_eq!(m.month, 2);
        assert_eq!(m.amount(BookKind::Made, 4), 2.5);
        assert_eq!((m.income_h, m.costs_h), (0.0, 0.6));
        assert_eq!(a.amount(BookKind::Sold, 4), 1.0);
        assert_eq!((a.income_h, a.owner_h, a.hired_h), (7.5, 3.0, 0.0));
        assert_eq!(
            a.lines.iter().map(|l| l.0).collect::<Vec<_>>(),
            vec![BookKind::Sold, BookKind::Paid],
            "in kind order"
        );
    }

    #[test]
    fn exit_and_book_codes_round_trip() {
        for e in Exit::ALL {
            assert_eq!(Exit::from_code(e as u8), Some(e));
        }
        for k in BookKind::ALL {
            assert_eq!(BookKind::from_code(k as u8), Some(k));
        }
        assert_eq!(Exit::from_code(0), None);
    }
}
