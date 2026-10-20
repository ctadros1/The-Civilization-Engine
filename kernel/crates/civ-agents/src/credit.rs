//! Claims in goods (M7d slices CA–CB, ADR-0027 §1-3): a household short of food before its
//! harvest asks a household with food to spare for a loan, repaid in the same good, with an
//! increment, at the harvest. Loans are a book apart from the help path (gifts keep no debt;
//! research 08-11 §1.1) and from ADR-0015's obligations (what a finding imposes). The lender
//! decides from what it believes: research 08-07 §1.3's reservation rate, i_min = (cT + p(1 − R))
//! / ((1 − p)T), with p the chance it believes the loan will not be repaid (from its regard for
//! the borrower and the harvest it expects the borrower to bring in), R what it would recover (a
//! field's use pledged recovers a content share of the debt), c what keeping the food at home
//! would earn it a year, and T the years to the due day. It asks the larger of that rate over T
//! and the content's customary increment, and refuses above a believed loss. Interest is
//! measured, never set.
//!
//! At the due day the debtor answers once, as it answers a finding (slice CB, ADR-0027 §3): it
//! means to pay, and pays what it can spare of the good, asking for time when that is not all; or
//! it refuses. Refused, or unpaid when the days of grace run out, the creditor takes the pledge: a
//! guarantor is called on, or the field pledged is worked by the creditor until its crop has paid
//! the debt.

use civ_core::PermanentId;

/// Where a claim stands. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Lent and not yet repaid in full.
    Open = 0,
    /// Repaid in full.
    Repaid = 1,
    /// Not repaid in full by its due day and the days of grace after it, though the debtor meant
    /// to pay (unable).
    Defaulted = 2,
    /// At its due day the debtor refused to pay (unwilling; slice CB).
    Refused = 3,
    /// The debtor's or the creditor's household is no more (slice CB).
    Lapsed = 4,
}

impl Standing {
    /// Every standing, in code order.
    pub const ALL: [Standing; 5] = [
        Standing::Open,
        Standing::Repaid,
        Standing::Defaulted,
        Standing::Refused,
        Standing::Lapsed,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The standing numbered `code`.
    pub fn from_code(code: u8) -> Option<Standing> {
        Standing::ALL.get(usize::from(code)).copied()
    }

    /// Whether the debtor failed it: defaulted or refused.
    pub fn failed(self) -> bool {
        matches!(self, Standing::Defaulted | Standing::Refused)
    }
}

/// What a debtor pledged for a loan (slice CB, ADR-0027 §3). Pledges at village scale are things
/// people already hold: a field's use (land is not sold, so no mortgage), or a guarantor's word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pledge {
    /// The use of the field it names, which the debtor holds and works: on a failed loan the
    /// creditor works it, on the regime's lease, until the crop it keeps has paid what is left.
    Field(PermanentId),
    /// The household it names promised to pay what the debtor does not: on a failed loan the
    /// creditor calls on it, and what it pays the debtor owes it in turn.
    Guarantor(PermanentId),
}

impl Pledge {
    /// Its kind's number in saves (0 is none): append only.
    pub fn code(self) -> u8 {
        match self {
            Pledge::Field(_) => 1,
            Pledge::Guarantor(_) => 2,
        }
    }

    /// The field or household it names.
    pub fn names(self) -> PermanentId {
        match self {
            Pledge::Field(id) | Pledge::Guarantor(id) => id,
        }
    }

    /// The pledge of kind `code` naming `id`; none for 0 or a code it does not know.
    pub fn from_code(code: u8, id: Option<PermanentId>) -> Option<Pledge> {
        match (code, id) {
            (1, Some(id)) => Some(Pledge::Field(id)),
            (2, Some(id)) => Some(Pledge::Guarantor(id)),
            _ => None,
        }
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
    /// Kilograms paid so far, by the debtor, its guarantor or its pledged field's crop.
    pub paid: f32,
    /// The day it was made.
    pub made: i64,
    /// The day it falls due: moved on by each extension granted.
    pub due: i64,
    /// Where it stands.
    pub standing: Standing,
    /// Who saw it made: the two households' elders.
    pub witnesses: [PermanentId; 2],
    /// What the debtor pledged, if anything (slice CB).
    pub pledge: Option<Pledge>,
    /// The debtor's answer at the due day, once given: whether it meant to pay, and its points
    /// toward paying over refusing (the receipt; slice CB).
    pub answer: Option<(bool, f32)>,
    /// Times the debtor asked for time (slice CB).
    pub asks: u8,
    /// Times the creditor granted it (slice CB).
    pub extensions: u8,
    /// The day the creditor took the pledge: called on the guarantor, or began to work the field
    /// (slice CB).
    pub taken: Option<i64>,
    /// Kilograms a guarantor paid of it (slice CB).
    pub by_guarantor: f32,
    /// For a claim a guarantor holds for what it paid: the claim it paid on (slice CB).
    pub recourse: Option<u32>,
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

    /// Whether household `household` stands guarantor for a loan still open.
    pub fn guarantees(&self, household: PermanentId) -> bool {
        self.claims
            .iter()
            .any(|c| c.standing == Standing::Open && c.pledge == Some(Pledge::Guarantor(household)))
    }

    /// Whether field `field` is pledged for a loan whose debt is not yet paid.
    pub fn pledged(&self, field: PermanentId) -> bool {
        self.claims.iter().any(|c| {
            c.pledge == Some(Pledge::Field(field))
                && c.standing != Standing::Lapsed
                && c.left() > 1e-3
        })
    }

    /// The claim numbered `id`.
    pub fn claim(&self, id: u32) -> Option<&Claim> {
        self.claims.iter().find(|c| c.id == id)
    }
}

/// Credit's priors (ADR-0027 §2-3; research 08-07 §1.3, §2.3). See content `[credit]`.
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
    /// The believed loss above which a lender refuses: the chance of no repayment, less what the
    /// pledge would recover of it.
    pub p_max: f64,
    /// What keeping the food at home earns a lender a year, a share (c).
    pub keep_return: f64,
    /// The most of the food it can spare a lender lends at once.
    pub lend_share: f64,
    /// Lenders asked at most, at one review.
    pub max_asked: u32,
    /// Days after its due day a debtor has to pay before the loan is defaulted.
    pub grace_days: u32,
    /// The share of what is owed a lender believes a field's use recovers (R; slice CB).
    pub field_recovery: f64,
    /// The least regard a household's elder holds for a borrower's elder to stand its guarantor
    /// (slice CB).
    pub guarantor_regard: f64,
    /// Days of its food a debtor keeps back when it pays (slice CB).
    pub keep_days: f64,
    /// Days an extension moves the due day on (slice CB).
    pub extension_days: u32,
    /// Extensions a creditor grants at the most (slice CB).
    pub max_extensions: u32,
    /// The least regard a creditor's elder holds for the debtor's elder to grant time (slice CB).
    pub extend_regard: f64,
    /// Points toward paying at the due day, before regard and burden (slice CB).
    pub pay_base: f64,
    /// Points toward paying per unit of the debtor elder's regard for the creditor's (0 to 1).
    pub w_pay_regard: f64,
    /// Points against paying per share of the food the debtor household holds that what it owes
    /// would take.
    pub w_pay_cost: f64,
    /// Days a workshop has to pay wages it could not pay before it fails (slice CB).
    pub wage_days: u32,
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
            field_recovery: 0.5,
            guarantor_regard: 0.3,
            keep_days: 3.0,
            extension_days: 30,
            max_extensions: 1,
            extend_regard: 0.0,
            pay_base: 3.0,
            w_pay_regard: 2.0,
            w_pay_cost: 2.0,
            wage_days: 30,
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

/// The terms a lender would lend on with what it believes (slice CB): the chance `p` it believes
/// the loan will not be repaid and `r` it would recover then, over `t_years`. The increment it
/// asks, a share of what it lends, or `None` when it refuses: when the loss it believes,
/// p(1 − R), is above `p_max`, or the increment above what a borrower accepts.
pub fn terms(p: f64, r: f64, t_years: f64, cp: &CreditParams) -> Option<f64> {
    let rate = reservation_rate(cp.keep_return, t_years, p, r);
    let increment = (rate * t_years).max(cp.customary_increment);
    let loss = p * (1.0 - r.clamp(0.0, 1.0));
    (loss <= cp.p_max && increment <= cp.max_increment).then_some(increment)
}

/// Points toward a debtor paying at the due day over refusing (slice CB; as a finding is met,
/// ADR-0015 §5): its elder's `regard` for the creditor's (0 to 1 counts) for paying, and the
/// `cost_share` against: the share of the food its household holds that what it owes would take
/// (0 to 1: a debt beyond all it holds weighs as all of it).
pub fn pay_points(regard: f64, cost_share: f64, cp: &CreditParams) -> f64 {
    cp.pay_base + cp.w_pay_regard * regard.clamp(0.0, 1.0)
        - cp.w_pay_cost * cost_share.clamp(0.0, 1.0)
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

    #[test]
    fn a_pledge_turns_a_refusal_into_terms_and_lowers_what_is_asked() {
        let cp = CreditParams::default();
        // Believed 0.6 unrepaid over half a year: refused bare, lent against a field.
        assert_eq!(terms(0.6, 0.0, 0.5, &cp), None);
        let loose = CreditParams {
            max_increment: 2.0,
            ..cp.clone()
        };
        assert!(terms(0.6, 0.5, 0.5, &loose).is_some());
        // A pledge lowers the increment the rate asks.
        let bare = terms(0.3, 0.0, 0.5, &cp).expect("lent");
        let pledged = terms(0.3, 0.5, 0.5, &cp).expect("lent");
        assert!(pledged < bare, "{pledged} {bare}");
        // Never below the custom.
        assert!(terms(0.0, 0.0, 0.5, &cp).is_some_and(|i| (i - 0.2).abs() < 1e-12));
        let id = PermanentId::from_raw(7).expect("id");
        for p in [Pledge::Field(id), Pledge::Guarantor(id)] {
            assert_eq!(Pledge::from_code(p.code(), Some(p.names())), Some(p));
        }
        assert_eq!(Pledge::from_code(0, Some(id)), None);
    }

    #[test]
    fn a_debtor_that_regards_its_creditor_and_owes_little_pays() {
        let cp = CreditParams::default();
        assert!(pay_points(1.0, 1.0, &cp) > pay_points(0.0, 1.0, &cp));
        assert!(pay_points(0.0, 1.0, &cp) < pay_points(0.0, 0.5, &cp));
        assert_eq!(pay_points(0.0, 5.0, &cp), pay_points(0.0, 1.0, &cp));
        // Negative regard counts as none.
        assert_eq!(pay_points(-1.0, 0.0, &cp), pay_points(0.0, 0.0, &cp));
    }
}
