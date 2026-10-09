//! The observer's interventions (M4c slice AJ, ADR-0016 §5; research 15-05 §4.2, §4.5, §6).
//! A god tool submits only a perceptible event or a physical change: a true claim heard, an
//! ideology heard of, a newcomer sent, a person's own luck in material things shifted for a while.
//! None chooses an action, passes or blocks a law, sets an allegiance or writes
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
    /// A newcomer sent holding an ideology (step two): the target is the newcomer.
    Agitator = 2,
    /// A person's own draws for illness or accident and for finding things out moved their way
    /// for a while (step two).
    Bless = 3,
    /// The same moved against them.
    Curse = 4,
}

impl InfluenceKind {
    /// Every kind, in code order.
    pub const ALL: [InfluenceKind; 5] = [
        InfluenceKind::Whisper,
        InfluenceKind::Ideology,
        InfluenceKind::Agitator,
        InfluenceKind::Bless,
        InfluenceKind::Curse,
    ];

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
    /// What it submitted: for a whisper, the claim's number; for an ideology or an agitator, its
    /// index in the catalog's ideologies; for a blessing or a curse, nothing.
    pub subject: u32,
    /// For an ideology: the day they took it up, if they did; for an agitator, the day they came.
    pub taken: Option<i64>,
    /// For an ideology: how many times they weighed it.
    pub weighed: u32,
    /// For a blessing or a curse: the first day it no longer holds.
    pub until: i64,
    /// For a blessing or a curse: the share of the way each draw is moved (see [`Luck`]).
    pub share: f32,
    /// For a blessing or a curse: the deaths by illness or accident it turned (spared, or
    /// brought), and the finds (brought, or lost): draws that would have gone the other way
    /// without it.
    pub deaths: u32,
    pub finds: u32,
}

/// A blessing or a curse in force on someone: which record, and by how much it moves their own
/// draws. A chance of something bad (illness or accident) is scaled by `1 − share` for a blessing
/// and by `1 / (1 − share)` for a curse; a chance of something good (finding something out) the
/// other way about, at most 1. The draw itself is unchanged, so what it turned is exact: the same
/// draw against the chance without it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Luck {
    pub index: usize,
    pub bless: bool,
    pub share: f64,
}

impl Luck {
    /// What a chance of harm is multiplied by.
    pub fn harm(self) -> f64 {
        let keep = (1.0 - self.share).clamp(MIN_KEEP, 1.0);
        if self.bless { keep } else { 1.0 / keep }
    }

    /// What a chance of good fortune is multiplied by.
    pub fn fortune(self) -> f64 {
        1.0 / self.harm()
    }
}

/// The least share of a chance a blessing leaves, whatever share is asked for.
const MIN_KEEP: f64 = 0.5;

/// The shares a blessing or curse may move a draw by, and the days it may last.
pub const SHARE_RANGE: [f32; 2] = [0.05, 0.5];
pub const DAYS_RANGE: [u32; 2] = [1, 3650];

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

    /// The blessing or curse on `person` on `day`, if one holds.
    pub fn luck(&self, person: PermanentId, day: i64) -> Option<Luck> {
        if self.list.is_empty() {
            return None;
        }
        self.list
            .iter()
            .enumerate()
            .rev()
            .find(|(_, i)| {
                i.target == person
                    && matches!(i.kind, InfluenceKind::Bless | InfluenceKind::Curse)
                    && i.at.day_index() <= day
                    && day < i.until
            })
            .map(|(index, i)| Luck {
                index,
                bless: i.kind == InfluenceKind::Bless,
                share: f64::from(i.share),
            })
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
            until: 0,
            share: 0.0,
            deaths: 0,
            finds: 0,
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

    #[test]
    fn a_blessing_holds_for_its_days_and_moves_chances_both_ways() {
        let mut inf = Influences::default();
        let p = PermanentId::from_raw(7).expect("an id");
        let day = 10;
        let t = SimTime::from_minutes(day * 24 * 60);
        assert_eq!(inf.luck(p, day), None);
        inf.add(t, InfluenceKind::Bless, p, 0);
        inf.list[0].until = day + 30;
        inf.list[0].share = 0.25;
        let luck = inf.luck(p, day).expect("blessed");
        assert!(luck.bless);
        assert!((luck.harm() - 0.75).abs() < 1e-6);
        assert!((luck.fortune() - 1.0 / 0.75).abs() < 1e-6);
        assert!(inf.luck(p, day + 29).is_some());
        assert_eq!(inf.luck(p, day + 30), None, "it ends");
        assert_eq!(inf.luck(p, day - 1), None, "nor before");
        let curse = Luck {
            bless: false,
            ..luck
        };
        assert!((curse.harm() - 1.0 / 0.75).abs() < 1e-6);
        // However large a share is asked, a chance is at most halved or doubled.
        let big = Luck { share: 0.9, ..luck };
        assert!((big.harm() - 0.5).abs() < 1e-9);
    }
}
