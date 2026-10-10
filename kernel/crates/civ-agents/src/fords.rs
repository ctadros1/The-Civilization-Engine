//! The streams a household's people wade (M5c slice AW, step two; the settlements brief §1.7):
//! each walk that wades a river cell counts one wade there for the walker's household, so what a
//! crossing would save the household is the walking its own recorded trips would save. What a
//! household holds fades by the same half-life as what it holds of the places its people work,
//! so a stream it no longer crosses weighs little.

use std::collections::BTreeMap;

use civ_core::PermanentId;

use crate::uses::fading;

/// What a household holds of a river cell its people wade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wade {
    pub cell: u32,
    /// Times its people waded it, fading.
    pub wades: f32,
    /// The day the fading was reckoned to.
    pub day: i64,
}

/// The river cells households' people wade, by household, each in cell order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fords {
    pub households: BTreeMap<PermanentId, Vec<Wade>>,
}

impl Fords {
    /// Someone of `household` waded `cell` on `day`.
    pub fn waded(&mut self, household: PermanentId, cell: u32, day: i64, half_life_days: f64) {
        let list = self.households.entry(household).or_default();
        match list.binary_search_by_key(&cell, |w| w.cell) {
            Ok(i) => {
                let w = &mut list[i];
                if day > w.day {
                    w.wades *= fading(day - w.day, half_life_days);
                    w.day = day;
                }
                w.wades += 1.0;
            }
            Err(i) => list.insert(
                i,
                Wade {
                    cell,
                    wades: 1.0,
                    day,
                },
            ),
        }
    }

    /// What `household` holds of the cells its people wade, faded to `day`, in cell order.
    pub fn held(
        &self,
        household: PermanentId,
        day: i64,
        half_life_days: f64,
    ) -> impl Iterator<Item = Wade> + '_ {
        self.households
            .get(&household)
            .into_iter()
            .flatten()
            .map(move |w| Wade {
                cell: w.cell,
                wades: w.wades * fading(day - w.day, half_life_days),
                day: day.max(w.day),
            })
    }

    /// Lets go of what households hold below `least` wades once faded to `day`, and of the
    /// households `keep` says are no more.
    pub fn forget(
        &mut self,
        day: i64,
        half_life_days: f64,
        least: f32,
        keep: impl Fn(PermanentId) -> bool,
    ) {
        self.households.retain(|&h, list| {
            list.retain(|w| w.wades * fading(day - w.day, half_life_days) >= least);
            keep(h) && !list.is_empty()
        });
    }

    /// What is wrong with the record, if anything: cells out of order or repeated, or a count
    /// that is not a count.
    pub fn problems(&self) -> Vec<String> {
        self.households
            .iter()
            .filter(|(_, list)| {
                list.is_empty()
                    || list.windows(2).any(|w| w[0].cell >= w[1].cell)
                    || list
                        .iter()
                        .any(|w| !(w.wades.is_finite() && w.wades >= 0.0))
            })
            .map(|(h, _)| format!("household {h}'s wades are out of order or malformed"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn wades_count_by_household_and_cell_and_fade_with_time() {
        let mut f = Fords::default();
        f.waded(id(2), 40, 10, 180.0);
        f.waded(id(2), 30, 10, 180.0);
        f.waded(id(2), 40, 10, 180.0);
        f.waded(id(3), 40, 10, 180.0);
        let held: Vec<Wade> = f.held(id(2), 10, 180.0).collect();
        assert_eq!(
            held.iter().map(|w| w.cell).collect::<Vec<_>>(),
            vec![30, 40]
        );
        assert_eq!(held[1].wades, 2.0);
        // A half-life on, the two count as one; a wade then adds to what is left.
        let later: Vec<Wade> = f.held(id(2), 190, 180.0).collect();
        assert!((later[1].wades - 1.0).abs() < 1e-6);
        f.waded(id(2), 40, 190, 180.0);
        assert!((f.held(id(2), 190, 180.0).nth(1).expect("waded").wades - 2.0).abs() < 1e-6);
        assert!(f.problems().is_empty());
        // What has faded below the least is let go of, and so is a household that is no more.
        f.forget(190, 180.0, 1.0, |h| h != id(3));
        assert_eq!(f.households.len(), 1);
        assert_eq!(f.held(id(2), 190, 180.0).count(), 1);
    }
}
