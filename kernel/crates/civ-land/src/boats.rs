//! Boats (M8b slice CO; ADR-0029 §2, §5): a boat is a record of its own, not a good in a store.
//! It is of a vehicle the content authors (the dugout first), belongs to a household, and stands
//! drawn up at the bank cell where it was last left. One being made is worked on there until its
//! labour is done; once made it ages by the day and is past use when its life is spent. Every
//! boat begun, made, moved or gone bumps [`Boats::revision`].

use civ_core::PermanentId;

/// One boat.
#[derive(Clone, Debug, PartialEq)]
pub struct Boat {
    pub id: PermanentId,
    /// Its vehicle, by catalog index.
    pub vehicle: u16,
    /// The household it belongs to.
    pub owner: PermanentId,
    /// The land cell beside the river where it is drawn up.
    pub at: u32,
    /// The day it was made.
    pub made: i64,
    /// The share of its life spent, 0 to 1.
    pub wear: f32,
    /// Hours of work making it takes, and those done (M8b slice CO, third part): it is made, and
    /// can be used, once they are equal. A boat given whole has both at 0.
    pub labour_h: f32,
    pub work_h: f32,
    /// The hours a year its household expected it to save when it began it.
    pub saves_h_year: f32,
    /// The day the last session of work on it began, and how many began one that day.
    pub crew: (i64, u8),
}

impl Boat {
    /// Whether its making is done.
    pub fn made(&self) -> bool {
        self.work_h + 1e-3 >= self.labour_h
    }
}

/// Every boat in the world, oldest first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Boats {
    pub list: Vec<Boat>,
    revision: u32,
}

impl Boats {
    /// Boats as saved.
    pub fn from_saved(list: Vec<Boat>, revision: u32) -> Boats {
        Boats { list, revision }
    }

    /// Changes whenever a boat is made, moved or gone; 0 while there has never been one.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// Notes that a boat was made, moved or gone.
    pub fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    /// The boats household `owner` has, made or being made, oldest first.
    pub fn of(&self, owner: PermanentId) -> impl Iterator<Item = &Boat> {
        self.list.iter().filter(move |b| b.owner == owner)
    }

    /// The boats household `owner` has that are made, oldest first.
    pub fn made_of(&self, owner: PermanentId) -> impl Iterator<Item = &Boat> {
        self.of(owner).filter(|b| b.made())
    }

    /// A day's ageing: each boat spends a day of its vehicle's life (`life_days(vehicle)`), and
    /// those past use are taken from the list and returned.
    pub fn age(&mut self, life_days: impl Fn(u16) -> f64) -> Vec<Boat> {
        for b in self.list.iter_mut().filter(|b| b.made()) {
            let life = life_days(b.vehicle).max(1.0);
            b.wear = (f64::from(b.wear) + 1.0 / life).min(1.0) as f32;
        }
        let (gone, kept): (Vec<Boat>, Vec<Boat>) = std::mem::take(&mut self.list)
            .into_iter()
            .partition(|b| b.wear >= 1.0);
        self.list = kept;
        if !gone.is_empty() {
            self.changed();
        }
        gone
    }

    /// What is wrong with the record, for the invariant checks.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for w in self.list.windows(2) {
            if w[0].id >= w[1].id {
                p.push(format!("boats {} and {} out of order", w[0].id, w[1].id));
            }
        }
        for b in &self.list {
            if !(0.0..1.0).contains(&b.wear) {
                p.push(format!("boat {} worn {} yet kept", b.id, b.wear));
            }
            if !(b.labour_h >= 0.0 && (0.0..=b.labour_h + 1e-3).contains(&b.work_h)) {
                p.push(format!(
                    "boat {} has {} of {} hours of work",
                    b.id, b.work_h, b.labour_h
                ));
            }
        }
        p
    }
}
