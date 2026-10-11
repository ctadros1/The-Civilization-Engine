//! Suspicion of a source of water (M6a slice BA, step two; ADR-0021 §6; research 12-02 §4, 05-04
//! §3.2). Each person tallies their own records: of the households they know to draw at a source,
//! how many had sickness lately, against those they know to draw elsewhere. They hold that the
//! source sickens when its share is a multiple of the rest's, over a minimum count (Snow's
//! comparison of houses by the company that supplied them orients this; it never targets it).
//! Nothing here reads where anyone took sickness, a source's load, or anyone else's tally.

use std::collections::{BTreeMap, BTreeSet};

use civ_core::PermanentId;

use crate::uses::Place;
use crate::word::Counts;

/// How people tally a source (the people profile's `[suspicion]` table; content API 76). Design
/// priors: research 12-02 §4's 8.5-fold contrast between water companies is an orientation, not a
/// target.
#[derive(Clone, Debug, PartialEq)]
pub struct SuspicionParams {
    /// Households known to draw at a source, at least, before its share means anything.
    pub min_households: u32,
    /// Households known to draw there that had sickness lately, at least.
    pub min_sick: u32,
    /// How many times the share elsewhere the source's share must be.
    pub ratio: f64,
    /// Days of drawing at a source, fading at the places' half-life, from which a household is
    /// taken to draw there (its own, and those seen there).
    pub draws_min_days: f64,
    /// How well someone must know a person to know their household's usual source (a tie's
    /// familiarity, 0-1).
    pub known_min: f64,
    /// Days between someone's tallies while they hold news of sickness or a suspicion; they also
    /// tally on first hearing of a household's sickness.
    pub review_days: u32,
}

impl SuspicionParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> SuspicionParams {
        SuspicionParams {
            min_households: 3,
            min_sick: 2,
            ratio: 3.0,
            draws_min_days: 2.0,
            known_min: 0.3,
            review_days: 7,
        }
    }
}

/// The sources a tally finds suspect, each with its counts, in source order: `draws` are the
/// households someone knows to draw at each source (a household may draw at several), `sick` the
/// households they know had sickness lately. Of the households known to draw anywhere, those at a
/// source are compared with the rest: the source is suspect when at least `min_households` draw
/// there, at least `min_sick` of them had sickness, and their share is at least `ratio` times the
/// rest's (counted with half a sick household in one more, so that a source everyone draws at,
/// with nothing to compare, is never suspect).
pub fn tally(
    draws: &BTreeMap<Place, BTreeSet<PermanentId>>,
    sick: &BTreeSet<PermanentId>,
    params: &SuspicionParams,
) -> Vec<(Place, Counts)> {
    let known: BTreeSet<PermanentId> = draws.values().flatten().copied().collect();
    let count = |n: usize| u16::try_from(n).unwrap_or(u16::MAX);
    let mut out = Vec::new();
    for (&source, at) in draws {
        let sick_at = at.iter().filter(|h| sick.contains(h)).count();
        let elsewhere: Vec<_> = known.iter().filter(|h| !at.contains(h)).collect();
        let sick_elsewhere = elsewhere.iter().filter(|h| sick.contains(h)).count();
        if at.len() < params.min_households as usize || sick_at < params.min_sick as usize {
            continue;
        }
        let share = sick_at as f64 / at.len() as f64;
        let rest = (sick_elsewhere as f64 + 0.5) / (elsewhere.len() as f64 + 1.0);
        if share >= params.ratio * rest {
            out.push((
                source,
                Counts {
                    sick_at: count(sick_at),
                    at: count(at.len()),
                    sick_elsewhere: count(sick_elsewhere),
                    elsewhere: count(elsewhere.len()),
                },
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("nonzero")
    }

    fn households(ns: impl IntoIterator<Item = u64>) -> BTreeSet<PermanentId> {
        ns.into_iter().map(id).collect()
    }

    #[test]
    fn a_source_whose_drawers_fell_sick_against_those_elsewhere_is_suspect() {
        let p = SuspicionParams::core();
        let well = Place::Source(10);
        let river = Place::Source(20);
        let mut draws = BTreeMap::new();
        draws.insert(well, households(1..=5));
        draws.insert(river, households(6..=15));
        // Three of five at the well, one of ten at the river.
        let sick = households([1, 2, 3, 9]);
        let found = tally(&draws, &sick, &p);
        assert_eq!(
            found,
            vec![(
                well,
                Counts {
                    sick_at: 3,
                    at: 5,
                    sick_elsewhere: 1,
                    elsewhere: 10
                }
            )]
        );
        // As many sick at the river, in proportion: nothing to tell them apart.
        let sick = households([1, 2, 6, 7, 8, 9]);
        assert!(tally(&draws, &sick, &p).is_empty());
    }

    #[test]
    fn too_few_to_count_or_nothing_to_compare_with_is_never_suspect() {
        let p = SuspicionParams::core();
        let well = Place::Source(10);
        // Everyone they know draws at one place, and all of them are sick.
        let mut draws = BTreeMap::new();
        draws.insert(well, households(1..=6));
        assert!(tally(&draws, &households(1..=6), &p).is_empty());
        // Two households at the well, both sick, against none sick of ten elsewhere: too few.
        let mut draws = BTreeMap::new();
        draws.insert(well, households([1, 2]));
        draws.insert(Place::Source(20), households(3..=12));
        assert!(tally(&draws, &households([1, 2]), &p).is_empty());
        // One sick of four at the well: below the minimum of sick households.
        let mut draws = BTreeMap::new();
        draws.insert(well, households(1..=4));
        draws.insert(Place::Source(20), households(5..=14));
        assert!(tally(&draws, &households([1]), &p).is_empty());
    }

    #[test]
    fn a_household_drawing_at_two_sources_counts_at_each() {
        let p = SuspicionParams::core();
        let (well, river) = (Place::Source(10), Place::Source(20));
        let mut draws = BTreeMap::new();
        draws.insert(well, households([1, 2, 3, 4]));
        draws.insert(river, households([4, 5, 6, 7, 8, 9, 10, 11]));
        // 4 draws at both: at the well it counts there, and is not among those elsewhere.
        let sick = households([1, 2, 4]);
        let found = tally(&draws, &sick, &p);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, well);
        assert_eq!(
            found[0].1,
            Counts {
                sick_at: 3,
                at: 4,
                sick_elsewhere: 0,
                elsewhere: 7
            }
        );
    }
}
