//! Agreements between polities (M5c slice AU, ADR-0020 §6): one shared record and a law in each
//! polity, each decided by its own custom. Two people who know each other, one of each
//! settlement, meet and agree on a package both expect to pass at home, or part with none; each
//! sponsors it at home; it is in force once both gatherings have passed it and each side has
//! heard of the other's decision, and failure (turned down, too few came, never put to a
//! gathering, never answered) is an outcome, never repaired. The binding subject is the polity.

use civ_core::{PermanentId, SimTime};

use crate::person::Flows;

/// A clause of an agreement. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Clause {
    /// The polity on side `from` (0 or 1) gives the other's people leave to use the places it
    /// claims.
    Leave { from: u8 },
    /// The polity on side `from` gives the other's store `kg` of good `good` (by catalog index)
    /// once, from its own common store (M5c slice AV, ADR-0020 §7).
    Gift { from: u8, good: u16, kg: u32 },
    /// The polity on side `from` gives the other's store `kg` of good `good` every `every_days`
    /// while it is in force, the first when it comes into force, from its own common store.
    Transfer {
        from: u8,
        good: u16,
        kg: u32,
        every_days: u32,
    },
}

impl Clause {
    /// Its kind and side in saves.
    pub fn code(self) -> (u8, u8) {
        match self {
            Clause::Leave { from } => (0, from),
            Clause::Gift { from, .. } => (1, from),
            Clause::Transfer { from, .. } => (2, from),
        }
    }

    /// What it gives, in saves: the good, the kilograms and the days between payments (0 for
    /// leave, and for a gift's days).
    pub fn amount(self) -> (u16, u32, u32) {
        match self {
            Clause::Leave { .. } => (0, 0, 0),
            Clause::Gift { good, kg, .. } => (good, kg, 0),
            Clause::Transfer {
                good,
                kg,
                every_days,
                ..
            } => (good, kg, every_days),
        }
    }

    /// The clause saved as `kind` and `side`, giving `good`, `kg` and `days` (see
    /// [`Clause::amount`]).
    pub fn from_code(kind: u8, side: u8, good: u16, kg: u32, days: u32) -> Option<Clause> {
        if side > 1 {
            return None;
        }
        match kind {
            0 => Some(Clause::Leave { from: side }),
            1 => Some(Clause::Gift {
                from: side,
                good,
                kg,
            }),
            2 if days > 0 => Some(Clause::Transfer {
                from: side,
                good,
                kg,
                every_days: days,
            }),
            _ => None,
        }
    }

    /// The side that gives by it.
    pub fn from(self) -> u8 {
        match self {
            Clause::Leave { from } | Clause::Gift { from, .. } | Clause::Transfer { from, .. } => {
                from
            }
        }
    }

    /// Whether it moves goods: a gift or a transfer.
    pub fn moves_goods(self) -> bool {
        !matches!(self, Clause::Leave { .. })
    }
}

/// Why a payment an agreement owes was missed (M5c slice AV, ADR-0020 §7: a miss keeps its cause
/// in the truth layer). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Miss {
    /// Nothing was set aside by the day it was due: the store held none of the good.
    EmptyStore,
    /// It was set aside, and nobody of the paying settlement could carry it.
    NoCarrier,
    /// It was set aside and someone could carry it, and nobody did by the day it was due.
    NotCarried,
}

impl Miss {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The miss with code `code`.
    pub fn from_code(code: u8) -> Option<Miss> {
        [Miss::EmptyStore, Miss::NoCarrier, Miss::NotCarried]
            .get(usize::from(code))
            .copied()
    }

    /// In words, after "missed: ".
    pub fn words(self) -> &'static str {
        match self {
            Miss::EmptyStore => "the store held none of it",
            Miss::NoCarrier => "nobody was there to carry it",
            Miss::NotCarried => "nobody carried it in time",
        }
    }
}

/// Where a payment stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DueState {
    /// Owed, and not yet handed over.
    Open,
    /// Handed over at the other's hearth on `day`.
    Met { day: i64 },
    /// Not handed over by the day it was due; `day` the day it was given up.
    Missed { day: i64, why: Miss },
}

/// One payment an agreement in force owes (M5c slice AV, ADR-0020 §7), with its four accounts:
/// what was owed, what the paying store set aside, and what arrived (what the paying store's
/// households paid in is its levy's own account). What is set aside it holds, a ledger holder of
/// its own, spoiling in the open until it is handed over; what is left of it when a payment is
/// given up goes back to the paying store.
#[derive(Clone, Debug, PartialEq)]
pub struct Due {
    /// Its id, as a ledger holder.
    pub id: PermanentId,
    pub agreement: PermanentId,
    /// The clause of the agreement it pays, by index.
    pub clause: u8,
    /// The paying and receiving polities.
    pub from: PermanentId,
    pub to: PermanentId,
    pub good: u16,
    /// Owed, kilograms.
    pub owed_kg: f64,
    /// The day it fell due, and the day by which it must be handed over.
    pub made: i64,
    pub due: i64,
    /// Set aside from the paying store so far, kilograms, and handed over at the other hearth.
    pub set_aside_kg: f64,
    pub arrived_kg: f64,
    /// Who is to carry it, once it is set aside.
    pub carrier: Option<PermanentId>,
    pub state: DueState,
    /// What it holds now, by good (catalog index), last brought up to date at `stores_at`.
    pub stores: Vec<f64>,
    pub stores_at: SimTime,
    pub flows: Flows,
}

impl Due {
    /// Whether it is still owed.
    pub fn open(&self) -> bool {
        self.state == DueState::Open
    }

    /// Brings what it holds up to `t`: it spoils in the open.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[crate::params::GoodDef]) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            if d.half_life_days > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / d.half_life_days);
                self.flows.add(crate::person::Flow::Spoiled, g, *kg - after);
                *kg = after;
            }
        }
        self.stores_at = t;
    }
}

/// Why an agreement failed. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The two who met found no package both expected to pass at home.
    NoTerms,
    /// Side `.0`'s gathering turned it down, or was evenly split.
    TurnedDown(u8),
    /// Too few came to side `.0`'s gathering.
    TooFew(u8),
    /// Nobody put it to side `.0`'s gathering in time.
    NeverCalled(u8),
    /// Side `.0` never heard of the other's decision in time.
    Unanswered(u8),
}

impl Failure {
    /// Its kind and side in saves.
    pub fn code(self) -> (u8, u8) {
        match self {
            Failure::NoTerms => (0, 0),
            Failure::TurnedDown(s) => (1, s),
            Failure::TooFew(s) => (2, s),
            Failure::NeverCalled(s) => (3, s),
            Failure::Unanswered(s) => (4, s),
        }
    }

    /// The failure saved as `kind` and `side`.
    pub fn from_code(kind: u8, side: u8) -> Option<Failure> {
        if side > 1 {
            return None;
        }
        Some(match kind {
            0 => Failure::NoTerms,
            1 => Failure::TurnedDown(side),
            2 => Failure::TooFew(side),
            3 => Failure::NeverCalled(side),
            4 => Failure::Unanswered(side),
            _ => return None,
        })
    }
}

/// Where an agreement stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgreementState {
    /// Agreed by the two who met, and not yet in force on both sides.
    Offered,
    /// In force since `since` (a day).
    InForce { since: i64 },
    /// Failed on `day`.
    Failed { day: i64, why: Failure },
    /// Ended on `day`: its term ran out, or a law of side `by` ended it.
    Ended { day: i64, by: Option<u8> },
}

/// One agreement between two polities.
#[derive(Clone, Debug, PartialEq)]
pub struct Agreement {
    pub id: PermanentId,
    /// The two polities: side 0 is the one whose member sought terms.
    pub polities: [PermanentId; 2],
    /// Who met: side 0's and side 1's.
    pub negotiators: [PermanentId; 2],
    /// What it does, in order.
    pub clauses: Vec<Clause>,
    /// Days it runs once in force; 0 until withdrawn.
    pub term_days: u32,
    /// The day the two agreed on it (or parted with none).
    pub made: i64,
    /// The law each side's gathering decides it as, once put to it.
    pub laws: [Option<PermanentId>; 2],
    /// The day each side's gathering passed it.
    pub passed: [Option<i64>; 2],
    /// The day each side heard that the other's gathering had passed it.
    pub heard: [Option<i64>; 2],
    pub state: AgreementState,
}

impl Agreement {
    /// The side `polity` is on, if either.
    pub fn side_of(&self, polity: PermanentId) -> Option<u8> {
        self.polities
            .iter()
            .position(|&p| p == polity)
            .map(|i| i as u8)
    }

    /// Whether it is in force.
    pub fn in_force(&self) -> bool {
        matches!(self.state, AgreementState::InForce { .. })
    }

    /// Whether, in force, it gives the people of `to` leave to use the places `from` claims.
    pub fn leave(&self, from: PermanentId, to: PermanentId) -> bool {
        self.in_force()
            && match (self.side_of(from), self.side_of(to)) {
                (Some(f), Some(t)) if f != t => self.clauses.contains(&Clause::Leave { from: f }),
                _ => false,
            }
    }

    /// Whether it is still before the gatherings or awaiting word: neither in force nor ended.
    pub fn open(&self) -> bool {
        self.state == AgreementState::Offered
    }

    /// Its terms in words, `name` naming a polity by its settlement and `good` a good by its
    /// catalog index: "leave for Ashford's people to use the places Oakholt claims, and 100 kg of
    /// grain from Ashford's store to Oakholt's each year, for 5 years".
    pub fn words(
        &self,
        name: &dyn Fn(PermanentId) -> String,
        good: &dyn Fn(u16) -> String,
    ) -> String {
        let side = |s: u8| name(self.polities[usize::from(s)]);
        let every = |d: u32| match d {
            365 => "each year".to_owned(),
            d if d % 365 == 0 => format!("every {} years", d / 365),
            d => format!("every {d} days"),
        };
        let clauses: Vec<String> = self
            .clauses
            .iter()
            .map(|c| match *c {
                Clause::Leave { from } => format!(
                    "leave for {}'s people to use the places {} claims",
                    side(1 - from),
                    side(from)
                ),
                Clause::Gift { from, good: g, kg } => format!(
                    "a gift of {kg} kg of {} from {}'s store to {}'s",
                    good(g),
                    side(from),
                    side(1 - from)
                ),
                Clause::Transfer {
                    from,
                    good: g,
                    kg,
                    every_days,
                } => format!(
                    "{kg} kg of {} from {}'s store to {}'s {}",
                    good(g),
                    side(from),
                    side(1 - from),
                    every(every_days)
                ),
            })
            .collect();
        let term = match self.term_days {
            0 => "until a law ends it".to_owned(),
            365 => "for a year".to_owned(),
            d if d % 365 == 0 => format!("for {} years", d / 365),
            d => format!("for {d} days"),
        };
        if clauses.is_empty() {
            return "no terms".to_owned();
        }
        format!("{}, {term}", clauses.join(", and "))
    }
}

/// Every agreement, in the order made, and every payment they owe (M5c slice AV), in the order
/// they fell due.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Agreements {
    pub list: Vec<Agreement>,
    pub dues: Vec<Due>,
}

impl Agreements {
    /// The agreement with id `id`.
    pub fn get(&self, id: PermanentId) -> Option<&Agreement> {
        self.list.iter().find(|a| a.id == id)
    }

    /// The agreement with id `id`, to change.
    pub fn get_mut(&mut self, id: PermanentId) -> Option<&mut Agreement> {
        self.list.iter_mut().find(|a| a.id == id)
    }

    /// The agreement law `law` decides, and the side it is on.
    pub fn of_law(&self, law: PermanentId) -> Option<(&Agreement, u8)> {
        self.list.iter().find_map(|a| {
            a.laws
                .iter()
                .position(|&l| l == Some(law))
                .map(|s| (a, s as u8))
        })
    }

    /// Whether an agreement in force gives the people of `to` leave to use `from`'s claims.
    pub fn leave(&self, from: PermanentId, to: PermanentId) -> bool {
        self.list.iter().any(|a| a.leave(from, to))
    }

    /// Whether an agreement between `a` and `b` is open or in force.
    pub fn between(&self, a: PermanentId, b: PermanentId) -> bool {
        self.list.iter().any(|x| {
            (x.open() || x.in_force()) && x.side_of(a).is_some() && x.side_of(b).is_some() && a != b
        })
    }

    /// What is wrong with the record, if anything.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.list.windows(2).any(|w| w[0].id >= w[1].id) {
            out.push("agreements are out of order or repeated".to_owned());
        }
        if self.dues.windows(2).any(|w| w[0].id >= w[1].id) {
            out.push("payments owed are out of order or repeated".to_owned());
        }
        for d in &self.dues {
            if !self.list.iter().any(|a| a.id == d.agreement) {
                out.push(format!("payment {} names a missing agreement", d.id));
            }
            if d.arrived_kg > d.set_aside_kg + 1e-6 || d.stores.iter().any(|&kg| kg < -1e-9) {
                out.push(format!("payment {} arrived more than was set aside", d.id));
            }
        }
        for a in &self.list {
            if a.polities[0] == a.polities[1] {
                out.push(format!("agreement {} is between a polity and itself", a.id));
            }
            if a.in_force()
                && (a.passed.iter().any(Option::is_none) || a.heard.iter().any(Option::is_none))
            {
                out.push(format!(
                    "agreement {} is in force without both passing it and hearing of it",
                    a.id
                ));
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

    fn offered(clauses: Vec<Clause>) -> Agreement {
        Agreement {
            id: id(1),
            polities: [id(10), id(20)],
            negotiators: [id(100), id(200)],
            clauses,
            term_days: 365,
            made: 5,
            laws: [None, None],
            passed: [None, None],
            heard: [None, None],
            state: AgreementState::Offered,
        }
    }

    #[test]
    fn leave_is_given_only_in_force_and_only_the_way_a_clause_runs() {
        let mut a = offered(vec![Clause::Leave { from: 1 }]);
        assert!(!a.leave(id(20), id(10)), "not before it is in force");
        a.passed = [Some(9), Some(12)];
        a.heard = [Some(14), Some(13)];
        a.state = AgreementState::InForce { since: 14 };
        assert!(a.leave(id(20), id(10)));
        assert!(!a.leave(id(10), id(20)), "the clause runs one way");
        assert!(!a.leave(id(20), id(30)), "nobody else's");
        let all = Agreements {
            list: vec![a],
            dues: Vec::new(),
        };
        assert!(all.problems().is_empty());
        assert!(all.between(id(10), id(20)) && !all.between(id(10), id(30)));
    }

    #[test]
    fn terms_are_told_in_words() {
        let mut a = offered(vec![Clause::Leave { from: 0 }, Clause::Leave { from: 1 }]);
        let name = |p: PermanentId| if p == id(10) { "Oakholt" } else { "Ashford" }.to_owned();
        let good = |_: u16| "grain".to_owned();
        assert_eq!(
            a.words(&name, &good),
            "leave for Ashford's people to use the places Oakholt claims, and leave for \
             Oakholt's people to use the places Ashford claims, for a year"
        );
        a.term_days = 1825;
        a.clauses.truncate(1);
        assert_eq!(
            a.words(&name, &good),
            "leave for Ashford's people to use the places Oakholt claims, for 5 years"
        );
        a.clauses.push(Clause::Transfer {
            from: 1,
            good: 3,
            kg: 100,
            every_days: 365,
        });
        a.clauses.push(Clause::Gift {
            from: 1,
            good: 3,
            kg: 400,
        });
        assert_eq!(
            a.words(&name, &good),
            "leave for Ashford's people to use the places Oakholt claims, and 100 kg of grain \
             from Ashford's store to Oakholt's each year, and a gift of 400 kg of grain from \
             Ashford's store to Oakholt's, for 5 years"
        );
        a.clauses.clear();
        assert_eq!(a.words(&name, &good), "no terms");
    }

    #[test]
    fn codes_round_trip() {
        for c in [
            Clause::Leave { from: 0 },
            Clause::Leave { from: 1 },
            Clause::Gift {
                from: 1,
                good: 4,
                kg: 200,
            },
            Clause::Transfer {
                from: 0,
                good: 4,
                kg: 100,
                every_days: 365,
            },
        ] {
            let ((k, s), (g, kg, d)) = (c.code(), c.amount());
            assert_eq!(Clause::from_code(k, s, g, kg, d), Some(c));
        }
        assert_eq!(Clause::from_code(0, 2, 0, 0, 0), None);
        assert_eq!(
            Clause::from_code(2, 0, 4, 100, 0),
            None,
            "a transfer needs its days"
        );
        for m in [Miss::EmptyStore, Miss::NoCarrier, Miss::NotCarried] {
            assert_eq!(Miss::from_code(m.code()), Some(m));
        }
        for f in [
            Failure::NoTerms,
            Failure::TurnedDown(1),
            Failure::TooFew(0),
            Failure::NeverCalled(1),
            Failure::Unanswered(0),
        ] {
            let (k, s) = f.code();
            assert_eq!(Failure::from_code(k, s), Some(f));
        }
    }
}
