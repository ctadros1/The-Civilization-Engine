//! Price reports (M5b slice AP, ADR-0019 §1): what each household believes another settlement's
//! sellers offer, and since when. A household reads its own settlement's offers as they stand; it
//! knows another's only by what a member saw there or was told at a hearth, dated, and buys there
//! only by a report.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// How price reports are held and passed on (the people profile's `[reports]`, content API 56).
/// Design priors: the research gives no half-life for a price heard of, only that merchants know
/// dated observations (08-12 §1.6) and that routine news is passed at 0.05–0.25 an opportunity
/// (09-16 §2.2).
#[derive(Clone, Debug, PartialEq)]
pub struct ReportParams {
    /// Days over which how much a household believes a report still holds halves.
    pub half_life_days: f64,
    /// Days after which a report is let go.
    pub max_age_days: i64,
    /// The chance someone tells a companion at the hearth of an offer elsewhere their household
    /// holds a newer report of than the companion's does.
    pub share_told: f64,
}

impl ReportParams {
    /// The core content's values, for tests.
    pub fn core() -> Self {
        ReportParams {
            half_life_days: 30.0,
            max_age_days: 120,
            share_told: 0.15,
        }
    }
}

/// How a household came by a report. Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReportHow {
    /// A member saw the offer at the seller's door.
    Seen,
    /// Someone told of it at a hearth: the seller's own people, or another who held the report.
    Told,
}

impl ReportHow {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The way with a code.
    pub fn from_code(code: u8) -> Option<ReportHow> {
        match code {
            0 => Some(ReportHow::Seen),
            1 => Some(ReportHow::Told),
            _ => None,
        }
    }
}

/// Why a trip to buy at a seller's door in another settlement bought nothing (ADR-0019 §2): the
/// report it went by no longer held, or the household no longer needed what it went for. Numeric
/// in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Missed {
    /// The seller no longer offered what it was reported to, or less than a whole tool of it.
    SoldOut,
    /// It offered it, on terms that no longer served the buyer.
    Terms,
    /// It offered it, for nothing the buyer held to spare.
    Payment,
    /// The household no longer needed what it was reported to offer.
    NoLonger,
}

impl Missed {
    /// How many reasons there are.
    pub const COUNT: usize = 4;

    /// Every reason, in code order.
    pub const ALL: [Missed; Missed::COUNT] = [
        Missed::SoldOut,
        Missed::Terms,
        Missed::Payment,
        Missed::NoLonger,
    ];

    /// In words: "the seller had sold out".
    pub fn words(self) -> &'static str {
        match self {
            Missed::SoldOut => "the seller had sold out",
            Missed::Terms => "the seller asked more than it was worth to them",
            Missed::Payment => "the seller took nothing they could spare",
            Missed::NoLonger => "they no longer needed what the seller had",
        }
    }
}

/// What a household believes one seller elsewhere offers of one good, as last seen or told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PriceReport {
    /// The settlement whose market it is.
    pub market: PermanentId,
    /// The selling household, or workshop.
    pub seller: PermanentId,
    pub firm: bool,
    /// The good offered and what it was asked in, by index in the catalog's goods.
    pub good: u16,
    pub payment: u16,
    /// Units of the payment asked for a unit of the good.
    pub price: f32,
    /// What a unit was worth to the seller behind its terms, hours of its own work.
    pub ask_h: f32,
    /// Units of the good offered: none once a member saw the seller had none left.
    pub units: f32,
    /// The day it was seen or told.
    pub day: i64,
    pub how: ReportHow,
    /// Who told of it, if it was told.
    pub from: Option<PermanentId>,
}

impl PriceReport {
    /// What it is held by: one report a market, good and payment.
    pub fn key(&self) -> (PermanentId, u16, u16) {
        (self.market, self.good, self.payment)
    }

    /// How much it is still to be believed on `day`: 1 when fresh, halving every `half_life_days`.
    pub fn weight(&self, day: i64, half_life_days: f64) -> f64 {
        let age = (day - self.day).max(0) as f64;
        0.5f64.powf(age / half_life_days.max(1e-9))
    }
}

/// Every household's price reports, each household's by market, good and payment: at most one
/// each, the latest seen or told. A seller may take several goods for one; a buyer goes by the
/// terms in what it holds to pay with.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PriceReports {
    /// By household.
    pub held: BTreeMap<PermanentId, Vec<PriceReport>>,
}

impl PriceReports {
    /// The reports `household` holds, by market, good and payment.
    pub fn of(&self, household: PermanentId) -> &[PriceReport] {
        self.held.get(&household).map_or(&[], Vec::as_slice)
    }

    /// `household` comes by `report`: it replaces the one held of its market, good and payment if
    /// it is newer, or as new and seen where the held one was told, or as new, come by alike and
    /// cheaper. Whether it was kept.
    pub fn note(&mut self, household: PermanentId, report: PriceReport) -> bool {
        let list = self.held.entry(household).or_default();
        match list.binary_search_by_key(&report.key(), PriceReport::key) {
            Ok(i) => {
                let held = &list[i];
                let first_hand = (report.how, report.price) < (held.how, held.price);
                if report.day > held.day || (report.day == held.day && first_hand) {
                    list[i] = report;
                    true
                } else {
                    false
                }
            }
            Err(i) => {
                list.insert(i, report);
                true
            }
        }
    }

    /// `household` saw on `day` that `seller` of `market` no longer offers `good` for `payment`:
    /// the report that said it did now says none is left, as seen then, so that word of it told
    /// later from before is not believed.
    pub fn sold_out(
        &mut self,
        household: PermanentId,
        market: PermanentId,
        seller: PermanentId,
        (good, payment): (u16, u16),
        day: i64,
    ) {
        if let Some(r) = self.held.get_mut(&household).and_then(|list| {
            list.iter_mut()
                .find(|r| r.key() == (market, good, payment) && r.seller == seller)
        }) {
            r.units = 0.0;
            r.day = r.day.max(day);
            r.how = ReportHow::Seen;
            r.from = None;
        }
    }

    /// People of `from` came to live in `to`, in settlement `home`: `to` holds what they held of
    /// other markets than its own, the newer of the two where both held one.
    pub fn bring(&mut self, from: PermanentId, to: PermanentId, home: Option<PermanentId>) {
        let brought: Vec<PriceReport> = self
            .of(from)
            .iter()
            .filter(|r| Some(r.market) != home)
            .copied()
            .collect();
        for r in brought {
            self.note(to, r);
        }
    }

    /// `household` is no more.
    pub fn forget(&mut self, household: PermanentId) {
        self.held.remove(&household);
    }

    /// Reports older than `max_age_days` on `day` are let go, and so are those of markets that
    /// are now a household's own (`home`; none for a household that is no more, whose reports
    /// all go).
    pub fn prune(
        &mut self,
        day: i64,
        max_age_days: i64,
        home: impl Fn(PermanentId) -> Option<Option<PermanentId>>,
    ) {
        self.held.retain(|&h, list| {
            let Some(home) = home(h) else {
                return false;
            };
            list.retain(|r| day - r.day <= max_age_days && Some(r.market) != home);
            !list.is_empty()
        });
    }

    /// What is wrong with these records, given the settlement a household lives in (none for a
    /// missing household).
    pub fn problems(
        &self,
        household: impl Fn(PermanentId) -> Option<Option<PermanentId>>,
    ) -> Vec<String> {
        let mut out = Vec::new();
        for (&h, list) in &self.held {
            if household(h).is_none() {
                out.push(format!("household {h} holds price reports but is no more"));
            }
            if list.is_empty() {
                out.push(format!(
                    "household {h} holds an empty list of price reports"
                ));
            }
            if list.windows(2).any(|w| w[0].key() >= w[1].key()) {
                out.push(format!("household {h}'s price reports are out of order"));
            }
            if list
                .iter()
                .any(|r| !(r.price.is_finite() && r.price > 0.0 && r.units >= 0.0))
            {
                out.push(format!("household {h} holds a price report without terms"));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn report(market: u64, good: u16, day: i64, price: f32) -> PriceReport {
        PriceReport {
            market: id(market),
            seller: id(9),
            firm: false,
            good,
            payment: 0,
            price,
            ask_h: 1.0,
            units: 5.0,
            day,
            how: ReportHow::Seen,
            from: None,
        }
    }

    #[test]
    fn a_newer_report_replaces_an_older_one_and_an_older_one_is_not_kept() {
        let mut r = PriceReports::default();
        assert!(r.note(id(1), report(5, 3, 10, 2.0)));
        assert!(!r.note(id(1), report(5, 3, 9, 1.0)), "older");
        assert!(r.note(id(1), report(5, 3, 10, 1.5)), "as new and cheaper");
        assert!(!r.note(id(1), report(5, 3, 10, 1.8)), "as new and dearer");
        let told = PriceReport {
            how: ReportHow::Told,
            ..report(5, 3, 10, 1.0)
        };
        assert!(!r.note(id(1), told), "told, as new as one seen");
        assert!(r.note(id(3), told));
        assert!(
            r.note(id(3), report(5, 3, 10, 9.0)),
            "seen, as new as one told"
        );
        assert!(r.note(id(3), report(5, 3, 10, 1.5)));
        r.forget(id(3));
        assert!(r.note(id(1), report(5, 1, 4, 1.0)));
        assert!(r.note(id(1), report(4, 7, 4, 1.0)));
        // The same good for another payment is another report.
        assert!(r.note(
            id(1),
            PriceReport {
                payment: 2,
                ..report(5, 3, 1, 9.0)
            }
        ));
        let held: Vec<_> = r
            .of(id(1))
            .iter()
            .map(|x| (x.market.get(), x.good, x.payment))
            .collect();
        assert_eq!(held, vec![(4, 7, 0), (5, 1, 0), (5, 3, 0), (5, 3, 2)]);
        assert!((r.of(id(1))[2].price - 1.5).abs() < 1e-6);
        assert!(r.problems(|_| Some(Some(id(4)))).is_empty());
    }

    #[test]
    fn reports_fade_by_age_and_go_when_old_or_home() {
        let mut r = PriceReports::default();
        r.note(id(1), report(5, 3, 0, 2.0));
        r.note(id(1), report(6, 3, 90, 2.0));
        let w = r.of(id(1))[0].weight(30, 30.0);
        assert!((w - 0.5).abs() < 1e-12);
        r.prune(100, 60, |_| Some(Some(id(6))));
        assert!(r.of(id(1)).is_empty(), "one too old, the other now home");
        r.note(id(2), report(5, 3, 0, 2.0));
        r.prune(10, 60, |_| None);
        assert!(
            r.held.is_empty(),
            "a household that is no more holds nothing"
        );
    }

    #[test]
    fn people_who_move_bring_what_they_knew_of_other_markets() {
        let mut r = PriceReports::default();
        r.note(id(1), report(5, 3, 10, 2.0));
        r.note(id(1), report(6, 3, 10, 2.0));
        r.bring(id(1), id(2), Some(id(6)));
        let held: Vec<_> = r.of(id(2)).iter().map(|x| x.market.get()).collect();
        assert_eq!(held, vec![5]);
        r.sold_out(id(2), id(5), id(9), (3, 1), 20);
        assert_eq!(r.of(id(2))[0].units, 5.0, "for another payment");
        r.sold_out(id(2), id(5), id(9), (3, 0), 20);
        let gone = r.of(id(2))[0];
        assert_eq!((gone.units, gone.day, gone.how), (0.0, 20, ReportHow::Seen));
        assert!(
            !r.note(id(2), report(5, 3, 15, 1.0)),
            "told later of before"
        );
        assert!(r.note(id(2), report(5, 3, 21, 1.0)), "of after");
    }
}
