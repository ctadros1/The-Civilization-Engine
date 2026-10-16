//! Claims in goods (M7d slice CA, ADR-0027 §1-2): a household short of food before its harvest
//! asks a household with food to spare for a loan, repaid in the same good, with an increment, at
//! the harvest. Loans are a book apart from the help path (gifts keep no debt; research 08-11
//! §1.1) and from ADR-0015's obligations (what a finding imposes). The lender decides from what it
//! believes: research 08-07 §1.3's reservation rate, i_min = (cT + p(1 − R)) / ((1 − p)T), with p
//! the chance it believes the loan will not be repaid (from its regard for the borrower and the
//! harvest it expects the borrower to bring in), R what it would recover (nothing, without a
//! pledge; pledges wait for slice CB), c what keeping the food at home would earn it a year, and T
//! the years to the due day. It asks the larger of that rate over T and the content's customary
//! increment, and refuses above a believed p. Interest is measured, never set.

use civ_core::PermanentId;

/// Where a claim stands. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Lent and not yet repaid in full.
    Open = 0,
    /// Repaid in full.
    Repaid = 1,
    /// Not repaid in full by its due day and the days of grace after it.
    Defaulted = 2,
}

impl Standing {
    /// Every standing, in code order.
    pub const ALL: [Standing; 3] = [Standing::Open, Standing::Repaid, Standing::Defaulted];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The standing numbered `code`.
    pub fn from_code(code: u8) -> Option<Standing> {
        Standing::ALL.get(usize::from(code)).copied()
    }
}

/// A loan in goods between two households (ADR-0027 §1).
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    /// Its number, from 1.
    pub id: u32,
    /// The household that lent.
    pub creditor: PermanentId,
    /// The household that owes.
    pub debtor: PermanentId,
    /// The good lent and owed.
    pub good: u16,
    /// Kilograms lent.
    pub lent: f32,
    /// Kilograms owed: what was lent and its increment.
    pub owed: f32,
    /// Kilograms paid so far.
    pub paid: f32,
    /// The day it was made.
    pub made: i64,
    /// The day it falls due.
    pub due: i64,
    /// Where it stands.
    pub standing: Standing,
    /// Who saw it made: the two households' elders.
    pub witnesses: [PermanentId; 2],
}

impl Claim {
    /// Kilograms still owed.
    pub fn left(&self) -> f64 {
        (f64::from(self.owed) - f64::from(self.paid)).max(0.0)
    }
}

/// The loans of a world, and what the asking came to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Credit {
    /// Every loan made, oldest first.
    pub claims: Vec<Claim>,
    /// The last loan number handed out.
    pub next: u32,
    /// Asks made of a lender.
    pub asked: u64,
    /// Asks a lender refused.
    pub refused: u64,
}

impl Credit {
    /// Whether household `household` owes on a loan still open.
    pub fn owes(&self, household: PermanentId) -> bool {
        self.claims
            .iter()
            .any(|c| c.debtor == household && c.standing == Standing::Open)
    }
}

/// Credit's priors (ADR-0027 §2; research 08-07 §1.3, §2.3). See content `[credit]`.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditParams {
    /// Days between a household's reviews of whether it must borrow.
    pub review_days: u32,
    /// The least days of its food a household borrows; below it, it asks no one.
    pub least_days: f64,
    /// The increment a lender asks at the least, a share of what it lent (a convention).
    pub customary_increment: f64,
    /// The most increment a borrower accepts, a share of what it is lent.
    pub max_increment: f64,
    /// The chance a lender believes a loan to a stranger will not be repaid, before what it
    /// expects of the borrower's harvest.
    pub p_prior: f64,
    /// The believed chance above which a lender refuses.
    pub p_max: f64,
    /// What keeping the food at home earns a lender a year, a share (c).
    pub keep_return: f64,
    /// The most of the food it can spare a lender lends at once.
    pub lend_share: f64,
    /// Lenders asked at most, at one review.
    pub max_asked: u32,
    /// Days after its due day a debtor has to pay before the loan is defaulted.
    pub grace_days: u32,
}

impl Default for CreditParams {
    fn default() -> Self {
        CreditParams {
            review_days: 7,
            least_days: 3.0,
            customary_increment: 0.2,
            max_increment: 0.5,
            p_prior: 0.2,
            p_max: 0.5,
            keep_return: 0.0,
            lend_share: 0.5,
            max_asked: 3,
            grace_days: 30,
        }
    }
}

/// A lender's reservation rate a year (research 08-07 §1.3, with no fixed cost):
/// i_min = (cT + p(1 − R)) / ((1 − p)T), for a loan of `t_years` it believes will not be repaid
/// with chance `p`, recovering `r` of it then, keeping the food earning `c` a year.
pub fn reservation_rate(c: f64, t_years: f64, p: f64, r: f64) -> f64 {
    let t = t_years.max(1.0 / 360.0);
    let p = p.clamp(0.0, 0.999);
    (c * t + p * (1.0 - r.clamp(0.0, 1.0))) / ((1.0 - p) * t)
}

/// The chance a lender believes a loan will not be repaid: of a stranger, `p_prior`, less as it
/// regards the borrower (`regard`, -1 to 1: none at full regard, more for one it thinks ill of),
/// and more as the harvest it expects the borrower to bring in falls short of what it would owe
/// (`cover`, the harvest over what is owed).
pub fn default_chance(p_prior: f64, regard: f64, cover: f64) -> f64 {
    let trust = (p_prior * (1.0 - regard.clamp(-1.0, 1.0))).clamp(0.0, 1.0);
    let short = (1.0 - cover.max(0.0)).clamp(0.0, 1.0);
    1.0 - (1.0 - trust) * (1.0 - short)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reservation_rate_answers_the_research_s_illustration() {
        // 08-07 §1.3, the money brief's illustration: T half a year, p 0.1, R 0.5, c 0.05.
        let i = reservation_rate(0.05, 0.5, 0.1, 0.5);
        assert!((i - 0.075 / 0.45).abs() < 1e-9, "{i}");
        // Nothing believed lost and nothing earned at home: nothing asked.
        assert_eq!(reservation_rate(0.0, 0.5, 0.0, 0.0), 0.0);
        // Over T it is a share of what is lent: p/(1-p) when c is 0 and nothing is recovered.
        assert!((reservation_rate(0.0, 0.5, 0.2, 0.0) * 0.5 - 0.25).abs() < 1e-9);
    }

    #[test]
    fn regard_and_a_harvest_that_covers_the_debt_lower_the_believed_chance_of_default() {
        let stranger = default_chance(0.2, 0.0, 2.0);
        assert!((stranger - 0.2).abs() < 1e-9);
        assert!(
            default_chance(0.2, 1.0, 2.0) < 1e-9,
            "full regard trusts fully"
        );
        assert!(default_chance(0.2, -1.0, 2.0) > stranger);
        // A harvest of half what is owed: half believed lost besides.
        assert!((default_chance(0.2, 1.0, 0.5) - 0.5).abs() < 1e-9);
        for s in Standing::ALL {
            assert_eq!(Standing::from_code(s.code()), Some(s));
        }
    }
}
