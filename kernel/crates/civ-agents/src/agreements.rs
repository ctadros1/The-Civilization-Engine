//! Agreements between polities (M5c slice AU, ADR-0020 §6): one shared record and a law in each
//! polity, each decided by its own custom. Two people who know each other, one of each
//! settlement, meet and agree on a package both expect to pass at home, or part with none; each
//! sponsors it at home; it is in force once both gatherings have passed it and each side has
//! heard of the other's decision, and failure (turned down, too few came, never put to a
//! gathering, never answered) is an outcome, never repaired. The binding subject is the polity.

use civ_core::PermanentId;

/// A clause of an agreement. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Clause {
    /// The polity on side `from` (0 or 1) gives the other's people leave to use the places it
    /// claims.
    Leave { from: u8 },
}

impl Clause {
    /// Its kind and side in saves.
    pub fn code(self) -> (u8, u8) {
        match self {
            Clause::Leave { from } => (0, from),
        }
    }

    /// The clause saved as `kind` and `side`.
    pub fn from_code(kind: u8, side: u8) -> Option<Clause> {
        match (kind, side) {
            (0, 0 | 1) => Some(Clause::Leave { from: side }),
            _ => None,
        }
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
}

/// Every agreement, in the order made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Agreements {
    pub list: Vec<Agreement>,
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
        let all = Agreements { list: vec![a] };
        assert!(all.problems().is_empty());
        assert!(all.between(id(10), id(20)) && !all.between(id(10), id(30)));
    }

    #[test]
    fn codes_round_trip() {
        for c in [Clause::Leave { from: 0 }, Clause::Leave { from: 1 }] {
            let (k, s) = c.code();
            assert_eq!(Clause::from_code(k, s), Some(c));
        }
        assert_eq!(Clause::from_code(0, 2), None);
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
