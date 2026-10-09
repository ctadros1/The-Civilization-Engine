//! The observer's interventions (M4c slice AJ, ADR-0016 §5; research 15-05 §4.2, §4.5, §6).
//! A god tool submits only a perceptible event or a physical change: a true claim heard, an
//! ideology heard of. None chooses an action, passes or blocks a law, sets an allegiance or writes
//! a grievance, a position or a vote. Each use is one record, kept with its number, its target and
//! what it submitted; a repeat of the same thing to the same person refreshes that record and never
//! stacks (15-05 §4.5: a hundred identical whispers equal one). What came of it is read from the
//! records people keep, never from here.

use civ_core::{PermanentId, SimTime};

/// What an intervention submitted. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfluenceKind {
    /// A true claim the settlement's word holds, placed in one person's hearing.
    Whisper = 0,
    /// A content ideology, heard of by one person from no one they know.
    Ideology = 1,
}

impl InfluenceKind {
    /// Every kind, in code order.
    pub const ALL: [InfluenceKind; 2] = [InfluenceKind::Whisper, InfluenceKind::Ideology];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The kind numbered `code`.
    pub fn from_code(code: u8) -> Option<InfluenceKind> {
        InfluenceKind::ALL.get(usize::from(code)).copied()
    }
}

/// One use of a god tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Influence {
    /// Its number, from 1, in the order used.
    pub id: u32,
    /// When it was first used.
    pub at: SimTime,
    /// When it was last used: a repeat refreshes the record and never makes another.
    pub last: SimTime,
    /// How many times it was used.
    pub uses: u32,
    pub kind: InfluenceKind,
    /// Whom it reached.
    pub target: PermanentId,
    /// What it submitted: for a whisper, the claim's number; for an ideology, its index in the
    /// catalog's ideologies.
    pub subject: u32,
    /// For an ideology: the day they took it up, if they did.
    pub taken: Option<i64>,
    /// For an ideology: how many times they weighed it.
    pub weighed: u32,
}

/// Every intervention, in the order used.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Influences {
    pub list: Vec<Influence>,
}

impl Influences {
    /// The record of `kind` that reached `target` with `subject`, if any.
    pub fn find(&self, kind: InfluenceKind, target: PermanentId, subject: u32) -> Option<usize> {
        self.list
            .iter()
            .position(|i| i.kind == kind && i.target == target && i.subject == subject)
    }

    /// The interventions that reached `target`, in the order used.
    pub fn on(&self, target: PermanentId) -> impl Iterator<Item = &Influence> {
        self.list.iter().filter(move |i| i.target == target)
    }

    /// Records a new use, numbered here, and returns its number.
    pub fn add(
        &mut self,
        at: SimTime,
        kind: InfluenceKind,
        target: PermanentId,
        subject: u32,
    ) -> u32 {
        let id = self.list.last().map_or(0, |i| i.id) + 1;
        self.list.push(Influence {
            id,
            at,
            last: at,
            uses: 1,
            kind,
            target,
            subject,
            taken: None,
            weighed: 0,
        });
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repeat_is_found_and_numbers_only_grow() {
        let mut inf = Influences::default();
        let p = PermanentId::from_raw(7).expect("an id");
        let t = SimTime::from_minutes(0);
        assert_eq!(inf.add(t, InfluenceKind::Whisper, p, 3), 1);
        assert_eq!(inf.add(t, InfluenceKind::Ideology, p, 3), 2);
        assert_eq!(inf.find(InfluenceKind::Whisper, p, 3), Some(0));
        assert_eq!(inf.find(InfluenceKind::Whisper, p, 4), None);
        assert_eq!(inf.on(p).count(), 2);
        for k in InfluenceKind::ALL {
            assert_eq!(InfluenceKind::from_code(k.code()), Some(k));
        }
    }
}
