//! Style v0 (M3b slice R; research 11-02): a household's taste in building, the traits a building
//! is built to, and how taste moves toward the buildings a settlement admires. Traits are
//! realised decisions kept in a design's parameters (ADR-0009 §7): roof pitch, height to the eaves
//! and the roof's overhang, each held to what a program allows. Taste is behaviour state.

use civ_core::{PermanentId, Rng64};
use civ_grammar::{BuildingSpec, ProgramRules, frame_params, hut_params};

use crate::params::{BuildingDef, StyleParams, Taste};

/// Keyed-randomness purpose of taste: its own streams, so no other draw moves.
pub const PURPOSE_STYLE: u64 = 0x7374_796c_6530_3031; // "style001"

/// A household's taste on arriving with band `band`: its band's way of building, drawn once for
/// the band around the content's, and its own a little apart from it. A founding band is keyed by
/// the settlement it founds; families the observer sends together by the first of them.
pub fn founding_taste(
    params: &StyleParams,
    seed: u64,
    band: PermanentId,
    household: PermanentId,
) -> Taste {
    let mean = params.tradition_mean.traits();
    let (band_sd, own_sd) = (
        params.tradition_spread.traits(),
        params.personal_spread.traits(),
    );
    let mut shared = Rng64::from_key(&[seed, PURPOSE_STYLE, band.get()]);
    let mut apart = Rng64::from_key(&[seed, PURPOSE_STYLE, band.get(), household.get()]);
    Taste::from_traits(std::array::from_fn(|k| {
        let tradition =
            f64::from(mean[k]) + f64::from(band_sd[k]) * crate::demography::normal(&mut shared);
        (tradition + f64::from(own_sd[k]) * crate::demography::normal(&mut apart)) as f32
    }))
}

/// The taste of a couple's new household from those of the households they grew up in, hers and
/// his, each with the building it admired: halfway between the two, admiring what hers admired,
/// else what his did.
pub fn couple_taste(
    hers: (&Taste, Option<PermanentId>),
    his: (&Taste, Option<PermanentId>),
) -> (Taste, Option<PermanentId>) {
    let (a, b) = (hers.0.traits(), his.0.traits());
    (
        Taste::from_traits(std::array::from_fn(|k| (a[k] + b[k]) / 2.0)),
        hers.1.or(his.1),
    )
}

/// What program `def` allows of each trait, least and most: pitch, eaves and overhang.
pub fn allowed(def: &BuildingDef) -> [(f32, f32); 3] {
    let pair = |(lo, hi): (i32, i32)| (lo as f32, hi as f32);
    match &def.rules {
        ProgramRules::Hut(r) => [
            pair(r.pitch_centideg),
            pair(r.eave_cm),
            (r.roof_overhang_cm as f32, r.roof_overhang_cm as f32),
        ],
        ProgramRules::Frame(r) => [pair(r.pitch_centideg), pair(r.eave_cm), pair(r.overhang_cm)],
    }
}

/// `taste` held to what program `def` allows, in whole units: the traits a building of it is
/// built to.
pub fn held_to(taste: &Taste, def: &BuildingDef) -> [i32; 3] {
    let t = taste.traits();
    let a = allowed(def);
    std::array::from_fn(|k| {
        if t[k].is_finite() {
            t[k].clamp(a[k].0, a[k].1).round() as i32
        } else {
            a[k].0.round() as i32
        }
    })
}

/// The traits building `spec` of program `def` was built to.
pub fn traits_of(spec: &BuildingSpec, def: &BuildingDef) -> Taste {
    match &def.rules {
        ProgramRules::Hut(r) => Taste {
            pitch_centideg: spec.params[hut_params::PITCH_CENTIDEG] as f32,
            eave_cm: spec.params[hut_params::EAVE_CM] as f32,
            overhang_cm: r.roof_overhang_cm as f32,
        },
        ProgramRules::Frame(_) => Taste {
            pitch_centideg: spec.params[frame_params::PITCH_CENTIDEG] as f32,
            eave_cm: spec.params[frame_params::EAVE_CM] as f32,
            overhang_cm: spec.params[frame_params::OVERHANG_CM] as f32,
        },
    }
}

/// `taste` moved `share` of the way toward `toward`, trait by trait.
pub fn moved(taste: &Taste, toward: &Taste, share: f64) -> Taste {
    let (a, b) = (taste.traits(), toward.traits());
    let share = share.clamp(0.0, 1.0) as f32;
    Taste::from_traits(std::array::from_fn(|k| a[k] + share * (b[k] - a[k])))
}

/// How much more than the least admired a building is admired that stands `rank` of the way from
/// the least admired of its settlement's (0) to the most (1): from 1 to the style's
/// `prestige_most` (research 11-02 §2.2: 1-3 times a neutral exemplar).
pub fn prestige(params: &StyleParams, rank: f64) -> f64 {
    1.0 + (params.prestige_most.max(1.0) - 1.0) * rank.clamp(0.0, 1.0)
}

/// The traits a household of taste `taste` builds program `def` to, at commission `key` (keyed
/// draws): its taste held to what the program allows, and now and then, at the style's
/// `innovation` chance, one trait new to it, anywhere the program allows.
pub fn commission(params: &StyleParams, taste: &Taste, def: &BuildingDef, key: &[u64]) -> [i32; 3] {
    let mut traits = held_to(taste, def);
    let mut rng = Rng64::from_key(key);
    if rng.chance(params.innovation) {
        let a = allowed(def);
        let k = rng.below(3) as usize;
        traits[k] = rng.range_f64(f64::from(a[k].0), f64::from(a[k].1)).round() as i32;
    }
    traits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> StyleParams {
        StyleParams {
            alpha: 0.1,
            prestige_most: 3.0,
            innovation: 0.0,
            tradition_mean: Taste {
                pitch_centideg: 4_500.0,
                eave_cm: 180.0,
                overhang_cm: 50.0,
            },
            tradition_spread: Taste {
                pitch_centideg: 400.0,
                eave_cm: 15.0,
                overhang_cm: 10.0,
            },
            personal_spread: Taste {
                pitch_centideg: 100.0,
                eave_cm: 4.0,
                overhang_cm: 3.0,
            },
            seen_most: 8,
            sight_m: 200.0,
        }
    }

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("nonzero")
    }

    #[test]
    fn a_band_shares_a_way_of_building_and_each_household_strays_a_little_from_it() {
        let p = params();
        let (a, b) = (
            founding_taste(&p, 7, id(1), id(2)),
            founding_taste(&p, 7, id(1), id(3)),
        );
        let other = founding_taste(&p, 7, id(9), id(2));
        assert_ne!(a, b);
        assert_eq!(a, founding_taste(&p, 7, id(1), id(2)), "keyed");
        // Two households of a band differ by their own spread, not their band's.
        let gap = |x: &Taste, y: &Taste| (x.pitch_centideg - y.pitch_centideg).abs();
        assert!(
            gap(&a, &b) < 6.0 * 100.0 * std::f32::consts::SQRT_2,
            "{a:?} {b:?}"
        );
        assert_ne!(gap(&a, &other), 0.0);
    }

    #[test]
    fn a_couple_builds_halfway_between_their_families_and_after_what_hers_admired() {
        let p = params();
        let hers = p.tradition_mean;
        let his = Taste {
            pitch_centideg: 5_100.0,
            eave_cm: 200.0,
            overhang_cm: 60.0,
        };
        let (taste, admired) = couple_taste((&hers, Some(id(4))), (&his, Some(id(5))));
        assert_eq!(
            taste,
            Taste {
                pitch_centideg: 4_800.0,
                eave_cm: 190.0,
                overhang_cm: 55.0,
            }
        );
        assert_eq!(admired, Some(id(4)), "what her household admired");
        assert_eq!(
            couple_taste((&hers, None), (&his, Some(id(5)))).1,
            Some(id(5))
        );
        assert_eq!(couple_taste((&hers, None), (&his, None)).1, None);
    }

    #[test]
    fn taste_moves_part_of_the_way_and_prestige_runs_from_one_to_the_most() {
        let p = params();
        let from = p.tradition_mean;
        let to = Taste {
            pitch_centideg: 5_500.0,
            eave_cm: 220.0,
            overhang_cm: 80.0,
        };
        let m = moved(&from, &to, 0.1);
        assert!((m.pitch_centideg - 4_600.0).abs() < 1e-3);
        assert!((m.eave_cm - 184.0).abs() < 1e-3);
        assert_eq!(moved(&from, &to, 2.0), to, "never past it");
        assert_eq!(prestige(&p, 0.0), 1.0);
        assert_eq!(prestige(&p, 1.0), 3.0);
        assert_eq!(prestige(&p, 0.5), 2.0);
    }
}
