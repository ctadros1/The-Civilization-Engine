//! Coin (M7d slice CE, ADR-0027 §6): issues a polity's mint strikes (cohorts: issuer, day, mass
//! and fineness) and purses that count, for each holder of coin, how many of each issue it holds.
//! A holder's coin is a good in its stores like any other; its purse says which issues those
//! pieces are. Payments move pieces issue by issue (the payer gives those worth least to it,
//! research 08-06 §1.7); anything that moves coin by another path is put right at midnight
//! ([`Coinage::reconcile`]), each issue's count conserved.

use civ_core::PermanentId;

/// Least count of pieces that matters (rounding).
const EPS: f64 = 1e-6;

/// An issue of coin: pieces struck alike by one mint on one day.
#[derive(Clone, Debug, PartialEq)]
pub struct Cohort {
    /// Its number, from 1 in the order struck.
    pub id: u32,
    /// The polity whose mint struck it.
    pub issuer: PermanentId,
    /// The day it was struck.
    pub day: i64,
    /// Grams a piece weighs.
    pub mass_g: f32,
    /// The share of a piece that is the metal it is reckoned in.
    pub fineness: f32,
    /// Pieces struck and still in being.
    pub count: f64,
}

impl Cohort {
    /// Kilograms of fine metal in a piece.
    pub fn metal_kg(&self) -> f64 {
        f64::from(self.mass_g) * f64::from(self.fineness) / 1000.0
    }
}

/// What one holder (a household, a workshop, a polity's store, a faction or a payment owed) holds
/// of each issue, by issue number.
#[derive(Clone, Debug, PartialEq)]
pub struct Purse {
    pub holder: PermanentId,
    /// `(issue, pieces)`, in issue order, none empty.
    pub counts: Vec<(u32, f64)>,
}

impl Purse {
    /// Pieces it holds, of every issue.
    pub fn total(&self) -> f64 {
        self.counts.iter().map(|&(_, n)| n).sum()
    }

    fn add(&mut self, issue: u32, n: f64) {
        match self.counts.binary_search_by_key(&issue, |&(i, _)| i) {
            Ok(i) => self.counts[i].1 += n,
            Err(i) => self.counts.insert(i, (issue, n)),
        }
    }
}

/// Every issue struck and every purse (saved).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Coinage {
    /// Issues, in the order struck.
    pub cohorts: Vec<Cohort>,
    /// Purses, by holder.
    pub purses: Vec<Purse>,
}

impl Coinage {
    /// The issue numbered `id`.
    pub fn cohort(&self, id: u32) -> Option<&Cohort> {
        self.cohorts
            .binary_search_by_key(&id, |c| c.id)
            .ok()
            .map(|i| &self.cohorts[i])
    }

    /// `holder`'s purse, if it holds any coin.
    pub fn purse(&self, holder: PermanentId) -> Option<&Purse> {
        self.purses
            .binary_search_by_key(&holder, |p| p.holder)
            .ok()
            .map(|i| &self.purses[i])
    }

    fn purse_mut(&mut self, holder: PermanentId) -> &mut Purse {
        let i = match self.purses.binary_search_by_key(&holder, |p| p.holder) {
            Ok(i) => i,
            Err(i) => {
                self.purses.insert(
                    i,
                    Purse {
                        holder,
                        counts: Vec::new(),
                    },
                );
                i
            }
        };
        &mut self.purses[i]
    }

    /// Strikes `count` pieces for `issuer` on `day`, each of `mass_g` grams at `fineness`, into
    /// `to`'s purse: an issue of their own, or the day's issue already struck to the same terms.
    /// The pieces are the caller's to add to `to`'s stores. Returns the issue's number.
    pub fn strike(
        &mut self,
        issuer: PermanentId,
        day: i64,
        mass_g: f32,
        fineness: f32,
        count: f64,
        to: PermanentId,
    ) -> u32 {
        let same = self.cohorts.iter_mut().find(|c| {
            c.issuer == issuer && c.day == day && c.mass_g == mass_g && c.fineness == fineness
        });
        let id = match same {
            Some(c) => {
                c.count += count;
                c.id
            }
            None => {
                let id = self.cohorts.last().map_or(1, |c| c.id + 1);
                self.cohorts.push(Cohort {
                    id,
                    issuer,
                    day,
                    mass_g,
                    fineness,
                    count,
                });
                id
            }
        };
        self.purse_mut(to).add(id, count);
        id
    }

    /// Moves `n` pieces from `from`'s purse to `to`'s, those `from` holds worth least to it
    /// first (`worth` of an issue, ties to the older), as far as its purse has them; any it does
    /// not is left for [`Coinage::reconcile`].
    pub fn give(&mut self, from: PermanentId, to: PermanentId, n: f64, worth: &dyn Fn(u32) -> f64) {
        if from == to || n <= EPS {
            return;
        }
        let Some(purse) = self.purse(from) else {
            return;
        };
        let mut order: Vec<(u32, f64)> = purse.counts.clone();
        order.sort_by(|a, b| worth(a.0).total_cmp(&worth(b.0)).then(a.0.cmp(&b.0)));
        let mut left = n;
        let mut moved: Vec<(u32, f64)> = Vec::new();
        for (issue, held) in order {
            if left <= EPS {
                break;
            }
            let take = held.min(left);
            moved.push((issue, take));
            left -= take;
        }
        let giver = self.purse_mut(from);
        for &(issue, take) in &moved {
            if let Ok(i) = giver.counts.binary_search_by_key(&issue, |&(c, _)| c) {
                giver.counts[i].1 -= take;
            }
        }
        giver.counts.retain(|&(_, k)| k > EPS);
        let taker = self.purse_mut(to);
        for (issue, take) in moved {
            taker.add(issue, take);
        }
        self.purses.retain(|p| !p.counts.is_empty());
    }

    /// Puts every purse right by what its holder holds of coin (`held`): one that counts more
    /// gives the excess up, every issue alike; then each of `holders` that holds more than its
    /// purse counts takes the pieces nobody's purse counts, oldest issue first. Issues are never
    /// struck or lost here.
    pub fn reconcile(&mut self, holders: &[PermanentId], held: &dyn Fn(PermanentId) -> f64) {
        for p in &mut self.purses {
            let has = held(p.holder).max(0.0);
            let counted = p.total();
            if counted > has + EPS {
                let keep = if counted > 0.0 { has / counted } else { 0.0 };
                for c in &mut p.counts {
                    c.1 *= keep;
                }
                p.counts.retain(|&(_, k)| k > EPS);
            }
        }
        self.purses.retain(|p| !p.counts.is_empty());
        let mut loose: Vec<(u32, f64)> = self
            .cohorts
            .iter()
            .map(|c| {
                let counted: f64 = self
                    .purses
                    .iter()
                    .flat_map(|p| p.counts.iter())
                    .filter(|&&(i, _)| i == c.id)
                    .map(|&(_, n)| n)
                    .sum();
                (c.id, (c.count - counted).max(0.0))
            })
            .filter(|&(_, n)| n > EPS)
            .collect();
        let mut hs = holders.to_vec();
        hs.sort_unstable();
        hs.dedup();
        for h in hs {
            let mut want = held(h) - self.purse(h).map_or(0.0, Purse::total);
            if want <= EPS {
                continue;
            }
            for (issue, n) in &mut loose {
                if want <= EPS {
                    break;
                }
                let take = n.min(want);
                if take > EPS {
                    self.purse_mut(h).add(*issue, take);
                    *n -= take;
                    want -= take;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn struck_pieces_move_issue_by_issue_least_worth_first() {
        let mut c = Coinage::default();
        let mint = id(9);
        let old = c.strike(mint, 1, 8.0, 0.95, 10.0, id(1));
        let new = c.strike(mint, 5, 8.0, 0.75, 10.0, id(1));
        assert_ne!(old, new);
        assert_eq!(
            c.strike(mint, 5, 8.0, 0.75, 2.0, id(2)),
            new,
            "same day, same terms"
        );
        // Paying 12, a payer who counts metal gives the debased issue first.
        let worth = |i: u32| if i == new { 0.75 } else { 0.95 };
        c.give(id(1), id(3), 12.0, &worth);
        let got = c.purse(id(3)).expect("paid");
        assert_eq!(got.counts, vec![(old, 2.0), (new, 10.0)]);
        assert!((c.purse(id(1)).expect("left").total() - 8.0).abs() < 1e-9);
        assert!((c.cohort(new).expect("issue").metal_kg() - 0.006).abs() < 1e-12);
    }

    #[test]
    fn reconciling_conserves_each_issue() {
        let mut c = Coinage::default();
        let a = c.strike(id(9), 1, 8.0, 0.95, 10.0, id(1));
        // Five pieces went from 1 to 2 by a path the purses did not see.
        let held = |h: PermanentId| match h.get() {
            1 => 5.0,
            2 => 5.0,
            _ => 0.0,
        };
        c.reconcile(&[id(1), id(2)], &held);
        assert_eq!(c.purse(id(1)).expect("kept").counts, vec![(a, 5.0)]);
        assert_eq!(c.purse(id(2)).expect("found").counts, vec![(a, 5.0)]);
        // A holder whose coin is gone loses its purse; the pieces stay struck, counted nowhere.
        c.reconcile(&[id(2)], &|h| if h.get() == 2 { 5.0 } else { 0.0 });
        assert!(c.purse(id(1)).is_none());
        assert_eq!(c.cohort(a).expect("issue").count, 10.0);
    }
}
