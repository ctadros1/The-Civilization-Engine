//! Crossings over water (M5c slice AW; the settlements brief §1.7; research 11-07): a bridge is
//! a record of its own, not a building on a plot. It spans an ordered run of water cells between
//! two land cells, by a system the content authors (a log beam first), its members sized as
//! built. Its condition is its members' quality, drawn once, and the share of their section lost
//! to rot; its capacity is worked out from those whenever it is needed (11-07 §1.3: it follows
//! the bridge's geometry, members and condition, never a type's fixed limit) and never saved.
//! While it is open, its cells are walked at its deck's speed; once it has failed, they are as
//! the river made them. Every change of state bumps [`Crossings::revision`], which the walking
//! grid and every cached route are kept against (ADR-0004 amended).

use civ_core::{PermanentId, SimTime};

/// Who a crossing belongs to: who built it, and who would keep it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossingOwner {
    Household(PermanentId),
    Polity(PermanentId),
}

/// Why a crossing failed. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collapse {
    /// Its members gave way under their own weight.
    OwnWeight,
    /// They gave way under someone stepping onto it.
    UnderWalker,
}

impl Collapse {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The cause with code `code`.
    pub fn from_code(code: u8) -> Option<Collapse> {
        [Collapse::OwnWeight, Collapse::UnderWalker]
            .get(usize::from(code))
            .copied()
    }
}

/// Where a crossing stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrossingState {
    /// Being built: `work_h` of its labour done.
    Building { work_h: f32 },
    /// Open to walkers since `day`.
    Open { since: i64 },
    /// It gave way on `day`, for `why`.
    Failed { day: i64, why: Collapse },
}

/// One crossing.
#[derive(Clone, Debug, PartialEq)]
pub struct Crossing {
    pub id: PermanentId,
    /// Its system, by catalog index.
    pub system: u16,
    /// The land cells at either end, where its members bear.
    pub banks: [u32; 2],
    /// The water cells it spans, in order from `banks[0]`.
    pub cells: Vec<u32>,
    /// Clear span, metres: the channel's width where it crosses.
    pub span_m: f32,
    /// How many main members lie side by side, and their diameter, centimetres.
    pub members: u8,
    pub diameter_cm: f32,
    /// The members' quality (0–1), drawn once when they were laid from the builders' skill
    /// (ADR-0009 §6); 0 while it is being built.
    pub quality: f32,
    /// The share of their effective section lost to rot (0–1).
    pub loss: f32,
    pub owner: CrossingOwner,
    /// Labour it takes, hours.
    pub labour_h: f32,
    /// Its builders' work so far, each hour weighted by the building skill of whoever did it:
    /// over its labour, how skilled they were on average, from which its members' quality is
    /// drawn when it opens (M5c slice AW, step two; ADR-0009 §6).
    pub skill_h: f32,
    pub begun: SimTime,
    pub state: CrossingState,
}

impl Crossing {
    /// Whether people may walk across it.
    pub fn open(&self) -> bool {
        matches!(self.state, CrossingState::Open { .. })
    }
}

/// Every crossing in the world, in the order begun, and the revision of their states.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Crossings {
    pub list: Vec<Crossing>,
    revision: u32,
}

impl Crossings {
    /// Crossings `list` at revision `revision` (a loaded world's).
    pub fn with_revision(list: Vec<Crossing>, revision: u32) -> Crossings {
        Crossings { list, revision }
    }

    /// Changes whenever a crossing opens or fails: routes planned before are not to be trusted.
    /// 0 while there has never been one.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// Notes that a crossing opened or failed.
    pub fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    /// The open crossings' spans, each with the walking factor `deck(system)` gives it: what the
    /// walking grid lays over the water.
    pub fn decks(&self, deck: impl Fn(u16) -> f32) -> Vec<(u32, f32)> {
        self.list
            .iter()
            .filter(|c| c.open())
            .flat_map(|c| {
                let f = deck(c.system);
                c.cells.iter().map(move |&cell| (cell, f))
            })
            .collect()
    }

    /// The open crossing spanning cell `cell`, by position, if any.
    pub fn open_at(&self, cell: u32) -> Option<usize> {
        self.list
            .iter()
            .position(|c| c.open() && c.cells.contains(&cell))
    }

    /// What is wrong with the record, for the invariant checks.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.list.windows(2).any(|w| w[0].id >= w[1].id) {
            out.push("crossings are out of order or repeated".to_owned());
        }
        for c in &self.list {
            if c.cells.is_empty() {
                out.push(format!("crossing {} spans no water", c.id));
            }
            if !(0.0..=1.0).contains(&c.loss) || !(0.0..=1.0).contains(&c.quality) {
                out.push(format!("crossing {} has a condition out of range", c.id));
            }
            if !(c.skill_h.is_finite() && c.skill_h >= 0.0) {
                out.push(format!(
                    "crossing {} has its builders' skill malformed",
                    c.id
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

    fn crossing(n: u64, state: CrossingState) -> Crossing {
        Crossing {
            id: id(n),
            system: 0,
            banks: [10, 12],
            cells: vec![11],
            span_m: 6.0,
            members: 2,
            diameter_cm: 30.0,
            quality: 0.9,
            loss: 0.0,
            owner: CrossingOwner::Household(id(1)),
            labour_h: 120.0,
            skill_h: 0.0,
            begun: SimTime::from_minutes(0),
            state,
        }
    }

    #[test]
    fn only_open_crossings_lay_decks_and_each_change_is_a_new_revision() {
        let mut all = Crossings::default();
        assert_eq!(all.revision(), 0);
        all.list
            .push(crossing(5, CrossingState::Building { work_h: 3.0 }));
        all.list.push(crossing(6, CrossingState::Open { since: 2 }));
        assert_eq!(all.decks(|_| 0.7), vec![(11, 0.7)]);
        assert_eq!(all.open_at(11), Some(1));
        all.changed();
        assert_eq!(all.revision(), 1);
        all.list[1].state = CrossingState::Failed {
            day: 9,
            why: Collapse::UnderWalker,
        };
        assert!(all.decks(|_| 0.7).is_empty() && all.open_at(11).is_none());
        assert!(all.problems().is_empty());
        for why in [Collapse::OwnWeight, Collapse::UnderWalker] {
            assert_eq!(Collapse::from_code(why.code()), Some(why));
        }
    }
}
