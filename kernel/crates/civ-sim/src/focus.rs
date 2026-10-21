//! Resolution by the observer's focus (ADR-0032 §6; M9b slice DK): which settlements are
//! household-resolved, chosen at each midnight from where the observer looks. A run setting the
//! observer turns on, never saved; off, every settlement is Detailed as it always was (ADR-0032
//! §5: household resolution is no default while Gate C fails).

use std::collections::{BTreeMap, BTreeSet};

use civ_core::PermanentId;
use civ_land::Settlement;

/// In a world of at most this many settlements lived in, every settlement is Detailed (ADR-0032
/// §6), so a world of the first versions lives as it always did.
pub const DETAILED_UP_TO: usize = 3;

/// Days a settlement stays Detailed before it may be household-resolved again (ADR-0032 §6,
/// research 01-04 §2's hysteresis). A settlement the observer turns to is Detailed from the next
/// midnight whatever its days, so the place looked at is never parked.
pub const DETAILED_AT_LEAST_DAYS: i64 = 30;

/// Where the observer looks, and what it holds Detailed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FocusPolicy {
    /// The settlement the observer follows or centres, if any.
    pub focus: Option<PermanentId>,
    /// Settlements the observer pinned Detailed.
    pub pins: BTreeSet<PermanentId>,
    /// The day each settlement last became Detailed under the policy.
    detailed_since: BTreeMap<PermanentId, i64>,
}

impl FocusPolicy {
    /// The settlements to household-resolve from the midnight beginning `day`, given those
    /// household-resolved now, or `None` if they stay as they are. With more than
    /// [`DETAILED_UP_TO`] settlements lived in, every one but the focus and the pins, each once
    /// it has been Detailed [`DETAILED_AT_LEAST_DAYS`].
    pub fn next(
        &mut self,
        day: i64,
        settlements: &[Settlement],
        now: &BTreeSet<PermanentId>,
    ) -> Option<BTreeSet<PermanentId>> {
        let lived: Vec<PermanentId> = settlements
            .iter()
            .filter(|s| s.abandoned.is_none())
            .map(|s| s.id)
            .collect();
        let kept = |s: &PermanentId| Some(*s) == self.focus || self.pins.contains(s);
        let mut next = BTreeSet::new();
        for s in lived.iter().filter(|_| lived.len() > DETAILED_UP_TO) {
            let wanted = !kept(s);
            let since = self.detailed_since.get(s).copied();
            let resolved = if now.contains(s) {
                wanted
            } else {
                wanted && since.is_none_or(|d| day - d >= DETAILED_AT_LEAST_DAYS)
            };
            if resolved {
                next.insert(*s);
            }
        }
        // A settlement brought back, or looked at, is Detailed as of today: being looked at keeps
        // its residence fresh, so it is held its days once the observer turns away.
        for s in &lived {
            if !next.contains(s) && (now.contains(s) || kept(s)) {
                self.detailed_since.insert(*s, day);
            }
        }
        (next != *now).then_some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn land_of(n: u64) -> Vec<Settlement> {
        let mut land = Vec::new();
        for i in 1..=n {
            land.push(Settlement {
                id: id(i),
                name: format!("S{i}"),
                founded: civ_core::SimTime::ZERO,
                hearth_m: (0.0, 0.0),
                food_short: false,
                harvest_kg: 0.0,
                parent: None,
                founding: civ_land::Founding::Setup,
                abandoned: None,
            });
        }
        land
    }

    #[test]
    fn three_settlements_or_fewer_stay_detailed() {
        let mut p = FocusPolicy::default();
        assert_eq!(p.next(10, &land_of(3), &BTreeSet::new()), None);
    }

    #[test]
    fn all_but_the_focus_and_the_pins_are_household_resolved() {
        let mut p = FocusPolicy {
            focus: Some(id(2)),
            pins: [id(4)].into(),
            ..FocusPolicy::default()
        };
        let next = p.next(10, &land_of(5), &BTreeSet::new()).expect("a change");
        assert_eq!(next, [id(1), id(3), id(5)].into());
        assert_eq!(p.next(11, &land_of(5), &next), None, "settled");
    }

    #[test]
    fn a_settlement_turned_to_is_detailed_at_once_and_stays_so_thirty_days() {
        let land = land_of(5);
        let mut p = FocusPolicy {
            focus: Some(id(1)),
            ..FocusPolicy::default()
        };
        let all_but_1: BTreeSet<_> = [id(2), id(3), id(4), id(5)].into();
        assert_eq!(p.next(0, &land, &BTreeSet::new()), Some(all_but_1.clone()));
        // The observer turns to settlement 2: Detailed from the next midnight; 1 holds 30 days.
        p.focus = Some(id(2));
        let now = p.next(1, &land, &all_but_1).expect("2 comes into focus");
        assert!(!now.contains(&id(2)) && !now.contains(&id(1)));
        assert_eq!(p.next(29, &land, &now), None, "1 is held Detailed");
        let later = p.next(31, &land, &now).expect("1 is let go");
        assert!(later.contains(&id(1)) && !later.contains(&id(2)));
    }
}
