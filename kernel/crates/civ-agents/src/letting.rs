//! Letting rooms (ADR-0040): a household with spare sleeping floor in a roofed, finished home lets
//! a share of it to a household with no roof of its own, for a rent in food a month paid on the
//! `Rent` channel. A let is a record, never a drawn room: its landlord, its lodger, the building,
//! the floor let, the rent, and its days.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// What a people profile asks of letting (ADR-0040 §2; the profile's `[letting]`, content API
/// 212). Design priors.
#[derive(Clone, Debug, PartialEq)]
pub struct LettingParams {
    /// The asking rent a household starts from: days of its lodgers' food a person a month.
    pub asking_rent_food_days_per_person_month: f64,
    /// How far the asking rent moves after an offer turned away for want of room (up) or a
    /// review with spare floor and no lodger (down), in the same unit.
    pub step: f64,
    /// The most homes a household seeking a room weighs at a review (08-19 §5.2: k candidates).
    pub candidates: u32,
}

impl Default for LettingParams {
    fn default() -> LettingParams {
        LettingParams {
            asking_rent_food_days_per_person_month: 1.0,
            step: 0.1,
            candidates: 8,
        }
    }
}

/// Days between a lodger's payments (ADR-0040 §3: monthly).
pub const RENT_DAYS: i64 = 30;
/// Months unpaid after which a let ends (ADR-0040 §3: after a second).
pub const ARREARS_MONTHS: u8 = 2;
/// Days a let begun is remembered for the settlement's recent rents (ADR-0040 §5: the last year).
pub const RECENT_DAYS: i64 = 365;

/// One household's lodging under another's roof (ADR-0040 §3).
#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    /// The household letting the floor, and the one lodging.
    pub landlord: PermanentId,
    pub lodger: PermanentId,
    /// The landlord's building the floor is in.
    pub building: PermanentId,
    /// Square metres of floor let.
    pub floor_m2: f64,
    /// The good the rent is reckoned in (a catalog index), and kilograms of it a month; a lodger
    /// short of it pays the same food energy in its other foods.
    pub good: usize,
    pub rent_kg: f64,
    /// The day it began, the day the next month's rent falls due, and the day of its next review.
    pub begun: i64,
    pub due: i64,
    pub next_review: i64,
    /// Months the lodger could not pay.
    pub unpaid: u8,
    /// Where the lodger's home was before it lodged, or where its own home is being built: where
    /// it goes back to when the let ends before it has a roof of its own, metres.
    pub own_home: (f32, f32),
}

/// A let begun, as the settlement's recent rents remember it (ADR-0040 §5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LetBegun {
    /// The day it began and the landlord's settlement.
    pub day: i64,
    pub settlement: PermanentId,
    /// Its rent, kcal of food a square metre a month.
    pub kcal_per_m2_month: f64,
}

/// Every let in force, the asking rents that moved, and the lets begun in the last year.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Letting {
    /// Lets in force, in the order begun.
    pub lets: Vec<Let>,
    /// Each household's asking rent where it moved from its profile's prior, food-days a person a
    /// month, by household.
    pub asking: BTreeMap<PermanentId, f64>,
    /// Lets begun within [`RECENT_DAYS`], oldest first.
    pub begun: Vec<LetBegun>,
}

impl Letting {
    /// The let household `lodger` lodges under, if any.
    pub fn lodging(&self, lodger: PermanentId) -> Option<&Let> {
        self.lets.iter().find(|l| l.lodger == lodger)
    }

    /// The lets household `landlord` gives.
    pub fn letting(&self, landlord: PermanentId) -> impl Iterator<Item = &Let> + '_ {
        self.lets.iter().filter(move |l| l.landlord == landlord)
    }

    /// Household `h`'s asking rent, food-days a person a month.
    pub fn asking_of(&self, h: PermanentId, params: &LettingParams) -> f64 {
        self.asking
            .get(&h)
            .copied()
            .unwrap_or(params.asking_rent_food_days_per_person_month)
            .max(0.0)
    }

    /// The mean rent of lets begun in settlement `s` within [`RECENT_DAYS`] of `day`, kcal a
    /// square metre a month; `None` when none were (ADR-0040 §5).
    pub fn recent_rent(&self, s: PermanentId, day: i64) -> Option<f64> {
        let (mut sum, mut n) = (0.0, 0usize);
        for b in &self.begun {
            if b.settlement == s && day - b.day < RECENT_DAYS {
                sum += b.kcal_per_m2_month;
                n += 1;
            }
        }
        (n > 0).then(|| sum / n as f64)
    }
}

/// Hours a year a home's spare floor would let for, to the household designing it (ADR-0040 §5):
/// `spare_m2` at the settlement's recent rent (`rent` kcal a square metre a month, `None` when no
/// let was seen), twelve months, times `roofless_share` (the chance a room is taken), in hours at
/// `hour_kcal` (what an hour of its work brings it in food). Nothing without a let seen.
pub fn rent_credit_h_year(
    rent: Option<f64>,
    spare_m2: f64,
    roofless_share: f64,
    hour_kcal: f64,
) -> f64 {
    match rent {
        Some(r) if r > 0.0 && spare_m2 > 0.0 && hour_kcal > 0.0 => {
            r * spare_m2 * 12.0 * roofless_share.clamp(0.0, 1.0) / hour_kcal
        }
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_credit_counts_only_where_lets_were_seen() {
        assert_eq!(rent_credit_h_year(None, 10.0, 0.5, 1000.0), 0.0);
        assert_eq!(rent_credit_h_year(Some(30.0), 0.0, 0.5, 1000.0), 0.0);
        assert_eq!(rent_credit_h_year(Some(30.0), 10.0, 0.0, 1000.0), 0.0);
        // 30 kcal a m² a month, 10 m², a year, half the households roofless, 1000 kcal an hour.
        let h = rent_credit_h_year(Some(30.0), 10.0, 0.5, 1000.0);
        assert!((h - 30.0 * 10.0 * 12.0 * 0.5 / 1000.0).abs() < 1e-12);
    }

    #[test]
    fn recent_rents_are_the_mean_of_the_last_year() {
        let s = PermanentId::from_raw(5).expect("id");
        let mut l = Letting::default();
        assert_eq!(l.recent_rent(s, 400), None);
        l.begun.push(LetBegun {
            day: 10,
            settlement: s,
            kcal_per_m2_month: 100.0,
        });
        l.begun.push(LetBegun {
            day: 300,
            settlement: s,
            kcal_per_m2_month: 50.0,
        });
        assert_eq!(l.recent_rent(s, 320), Some(75.0));
        assert_eq!(l.recent_rent(s, 400), Some(50.0));
        assert_eq!(l.recent_rent(s, 700), None);
    }
}
