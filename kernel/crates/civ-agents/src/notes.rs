//! Notes (M10d slice EH; ADR-0035 §9, §12; brief m10-money §3, §4, §9): a written claim on its
//! issuer for a good, held in pieces as coin is. An issue is one keeper's notes in one good, a
//! piece redeemable at its door for `kg` of it (one issue per issuer and terms, never per day:
//! brief §1); purses count, for each holder, how many pieces of each issue it holds, as coin's do
//! ([`crate::coin`]). A piece comes into being only when its issuer writes it, and leaves only when
//! it is redeemed and retired; every piece outside its issuer's hands is its issuer's liability.

use civ_core::PermanentId;

use crate::coin::{Purse, give_in, purse_in, purse_mut_in, reconcile_in};

/// Least count of pieces that matters (rounding).
const EPS: f64 = 1e-6;

/// One issuer's notes in one good.
#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    /// Its number, from 1 in the order first written.
    pub id: u32,
    /// The household that wrote them and redeems them.
    pub issuer: PermanentId,
    /// The good a piece is redeemed in (catalog index), and kilograms of it a piece pays.
    pub good: u16,
    pub kg: f32,
    /// Pieces written and not yet retired: what its issuer owes.
    pub count: f64,
    /// Pieces presented for redemption, and of those refused for want of the good, since it was
    /// first written; and the last day one was refused (-1 for none).
    pub presented: f64,
    pub refused: f64,
    pub refused_day: i64,
    /// Pieces its issuer wrote as loans, beyond the goods left with it (M10d slice EI).
    pub lent: f64,
    /// The last day a debtor of a loan in these notes defaulted (-1 for none), the pieces
    /// presented in the week begun on `seen_day` (-1 for none), and the day before which its
    /// issuer pays none (-1 for none: it does not suspend) (M10d slice EI, second part).
    pub defaulted_day: i64,
    pub seen: f64,
    pub seen_day: i64,
    pub suspended_until: i64,
}

/// Days over which pieces presented at an issuer's door are seen together (M10d slice EI).
pub const SEEN_DAYS: i64 = 7;

/// False issues are numbered from here, apart from real ones (M10d slice EK).
pub const FORGED_BASE: u32 = 1 << 31;

/// False pieces of one real issue that one household made (M10d slice EK; brief m10-money §7):
/// held in purses under their own number, counted by holders as the real issue's, owed by
/// nobody.
#[derive(Clone, Debug, PartialEq)]
pub struct Forgery {
    /// Its number, from [`FORGED_BASE`] in the order first made.
    pub id: u32,
    /// The real issue it passes for.
    pub of: u32,
    /// The household that made them.
    pub maker: PermanentId,
    /// Pieces made and not yet taken out of being.
    pub count: f64,
}

/// Every issue of notes and every purse holding any (saved).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Notes {
    /// Issues, in the order first written.
    pub issues: Vec<Issue>,
    /// Purses, by holder.
    pub purses: Vec<Purse>,
    /// Loans in the credit book that are notes refused under a law of bearer heard and booked as
    /// what their keeper owes their holder (M10d slice EI, third part), by claim number.
    pub dishonoured: Vec<u32>,
    /// False pieces made, in the order first made (M10d slice EK).
    pub forgeries: Vec<Forgery>,
}

impl Notes {
    /// The issue numbered `id`.
    pub fn issue(&self, id: u32) -> Option<&Issue> {
        self.issues
            .binary_search_by_key(&id, |c| c.id)
            .ok()
            .map(|i| &self.issues[i])
    }

    /// The issue numbered `id`, to change.
    pub fn issue_mut(&mut self, id: u32) -> Option<&mut Issue> {
        self.issues
            .binary_search_by_key(&id, |c| c.id)
            .ok()
            .map(|i| &mut self.issues[i])
    }

    /// The false issue numbered `id` (M10d slice EK).
    pub fn forgery(&self, id: u32) -> Option<&Forgery> {
        self.forgeries
            .binary_search_by_key(&id, |f| f.id)
            .ok()
            .map(|i| &self.forgeries[i])
    }

    /// The issue a holder takes pieces numbered `id` for: the issue itself, or the real one a
    /// false issue passes for (M10d slice EK).
    pub fn seen(&self, id: u32) -> Option<&Issue> {
        self.issue(id).or_else(|| self.issue(self.forgery(id)?.of))
    }

    /// Pieces `holder` holds that it takes for issue `issue`'s: the real ones and any false ones
    /// passing for them (M10d slice EK).
    pub fn held_seen(&self, holder: PermanentId, issue: u32) -> f64 {
        self.held(holder, issue)
            + self
                .forgeries
                .iter()
                .filter(|f| f.of == issue)
                .map(|f| self.held(holder, f.id))
                .sum::<f64>()
    }

    /// Household `maker` makes `count` false pieces passing for issue `of` into its own purse:
    /// its false issue of it, or a new one (M10d slice EK). The pieces are the caller's to add to
    /// its stores. Returns the false issue's number.
    pub fn forge(&mut self, maker: PermanentId, of: u32, count: f64) -> u32 {
        let id = match self
            .forgeries
            .iter_mut()
            .find(|f| f.maker == maker && f.of == of)
        {
            Some(f) => {
                f.count += count;
                f.id
            }
            None => {
                let id = self.forgeries.last().map_or(FORGED_BASE, |f| f.id + 1);
                self.forgeries.push(Forgery {
                    id,
                    of,
                    maker,
                    count,
                });
                id
            }
        };
        purse_mut_in(&mut self.purses, maker).add(id, count);
        id
    }

    /// `issuer`'s issue in good `good`, if it has written one.
    pub fn issue_of(&self, issuer: PermanentId, good: usize) -> Option<&Issue> {
        self.issues
            .iter()
            .find(|c| c.issuer == issuer && usize::from(c.good) == good)
    }

    /// Whether claim `id` of the credit book is notes refused, booked as a debt.
    pub fn is_dishonoured(&self, id: u32) -> bool {
        self.dishonoured.binary_search(&id).is_ok()
    }

    /// `holder`'s purse, if it holds any notes.
    pub fn purse(&self, holder: PermanentId) -> Option<&Purse> {
        purse_in(&self.purses, holder)
    }

    /// Pieces of issue `issue` `holder` holds.
    pub fn held(&self, holder: PermanentId, issue: u32) -> f64 {
        self.purse(holder).map_or(0.0, |p| {
            p.counts
                .binary_search_by_key(&issue, |&(i, _)| i)
                .map_or(0.0, |i| p.counts[i].1)
        })
    }

    /// Writes `count` pieces of `issuer`'s issue in good `good`, `kg` of it a piece, into `to`'s
    /// purse: its issue, or a new one. The pieces are the caller's to add to `to`'s stores.
    /// Returns the issue's number.
    pub fn write(
        &mut self,
        issuer: PermanentId,
        (good, kg): (usize, f32),
        count: f64,
        to: PermanentId,
    ) -> u32 {
        let id = match self
            .issues
            .iter_mut()
            .find(|c| c.issuer == issuer && usize::from(c.good) == good && c.kg == kg)
        {
            Some(c) => {
                c.count += count;
                c.id
            }
            None => {
                let id = self.issues.last().map_or(1, |c| c.id + 1);
                self.issues.push(Issue {
                    id,
                    issuer,
                    good: u16::try_from(good).unwrap_or(u16::MAX),
                    kg,
                    count,
                    presented: 0.0,
                    refused: 0.0,
                    refused_day: -1,
                    lent: 0.0,
                    defaulted_day: -1,
                    seen: 0.0,
                    seen_day: -1,
                    suspended_until: -1,
                });
                id
            }
        };
        purse_mut_in(&mut self.purses, to).add(id, count);
        id
    }

    /// Moves `n` pieces from `from`'s purse to `to`'s, those worth least to `from` first, as
    /// coin moves ([`crate::coin::Coinage::give`]).
    pub fn give(&mut self, from: PermanentId, to: PermanentId, n: f64, worth: &dyn Fn(u32) -> f64) {
        give_in(&mut self.purses, from, to, n, worth);
    }

    /// Moves `n` pieces of issue `issue` from `from`'s purse to `to`'s, as far as `from`'s holds
    /// them: presented at the issuer's door.
    pub fn give_issue(&mut self, from: PermanentId, to: PermanentId, issue: u32, n: f64) {
        let n = n.min(self.held(from, issue));
        if from == to || n <= EPS {
            return;
        }
        let giver = purse_mut_in(&mut self.purses, from);
        if let Ok(i) = giver.counts.binary_search_by_key(&issue, |&(c, _)| c) {
            giver.counts[i].1 -= n;
        }
        giver.counts.retain(|&(_, k)| k > EPS);
        purse_mut_in(&mut self.purses, to).add(issue, n);
        self.purses.retain(|p| !p.counts.is_empty());
    }

    /// Takes `n` pieces of issue `issue` out of `holder`'s purse and out of being: redeemed, they
    /// are no longer owed. The pieces are the caller's to take out of `holder`'s stores.
    pub fn retire(&mut self, holder: PermanentId, issue: u32, n: f64) {
        if n <= 0.0 {
            return;
        }
        let p = purse_mut_in(&mut self.purses, holder);
        if let Ok(i) = p.counts.binary_search_by_key(&issue, |&(c, _)| c) {
            p.counts[i].1 -= n;
        }
        p.counts.retain(|&(_, k)| k > EPS);
        self.purses.retain(|p| !p.counts.is_empty());
        if let Some(c) = self.issue_mut(issue) {
            c.count = (c.count - n).max(0.0);
        }
    }

    /// Counts `n` pieces of issue `issue` presented on `day`, `refused` of them refused, and seen
    /// presented that week.
    pub fn note_presented(&mut self, issue: u32, n: f64, refused: f64, day: i64) {
        if let Some(c) = self.issue_mut(issue) {
            c.presented += n;
            c.refused += refused;
            if refused > EPS {
                c.refused_day = day;
            }
            if c.seen_day < 0 || day - c.seen_day >= SEEN_DAYS {
                c.seen = 0.0;
                c.seen_day = day;
            }
            c.seen += n;
        }
    }

    /// Puts every purse right by what its holder holds of notes (`held`), each issue's count
    /// conserved, as coin's are ([`crate::coin::Coinage::reconcile`]).
    pub fn reconcile(&mut self, holders: &[PermanentId], held: &dyn Fn(PermanentId) -> f64) {
        // False pieces are found again as real ones are (M10d slice EK).
        let issues: Vec<(u32, f64)> = self
            .issues
            .iter()
            .map(|c| (c.id, c.count))
            .chain(self.forgeries.iter().map(|f| (f.id, f.count)))
            .collect();
        reconcile_in(&mut self.purses, &issues, holders, held);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn notes_are_written_one_issue_per_issuer_and_good_and_retired_when_redeemed() {
        let mut n = Notes::default();
        let a = n.write(id(9), (3, 1.0), 100.0, id(1));
        assert_eq!(n.write(id(9), (3, 1.0), 50.0, id(2)), a, "the same issue");
        let b = n.write(id(9), (4, 1.0), 10.0, id(1));
        assert_ne!(a, b, "another good, another issue");
        assert_eq!(n.issue(a).expect("issue").count, 150.0);
        n.give(id(1), id(3), 30.0, &|i| if i == b { 0.0 } else { 1.0 });
        assert_eq!(n.held(id(3), b), 10.0, "the least worth first");
        assert_eq!(n.held(id(3), a), 20.0);
        n.retire(id(3), a, 20.0);
        assert_eq!(n.held(id(3), a), 0.0);
        assert_eq!(n.issue(a).expect("issue").count, 130.0);
        n.note_presented(a, 20.0, 5.0, 7);
        let c = n.issue(a).expect("issue");
        assert_eq!((c.presented, c.refused, c.refused_day), (20.0, 5.0, 7));
    }
}
