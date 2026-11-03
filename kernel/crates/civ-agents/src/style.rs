//! Style v0 (M3b slice R; research 11-02): a household's taste in building, the traits a building
//! is built to, and how taste moves toward the buildings a settlement admires. Traits are
//! realised decisions kept in a design's parameters (ADR-0009 §7): roof pitch, height to the eaves
//! and the roof's overhang, and from frame version 3 the front's openings, ornament, finish and
//! symmetry (M8d slice DD; ADR-0031 §7), each held to what a program allows. Taste is behaviour
//! state.

use civ_core::{PermanentId, Rng64};
use civ_grammar::frame::{MAX_OPENING_PERMILLE, MAX_ORNAMENT_DH};
use civ_grammar::{BuildingSpec, ProgramRules, frame_params, hut_params};

use crate::params::{BuildingDef, StyleParams, Taste};

/// Keyed-randomness purpose of taste: its own streams, so no other draw moves.
pub const PURPOSE_STYLE: u64 = 0x7374_796c_6530_3031; // "style001"
/// Keyed-randomness purpose of a commission's front (M8d slice DD): its own stream, so the roof's
/// draws are as before.
pub const PURPOSE_FRONT: u64 = 0x7374_796c_6566_7274; // "stylefrt"

/// A household's taste on arriving with band `band`: its band's way of building, drawn once for
/// the band around the content's, and its own a little apart from it. A founding band is keyed by
/// the settlement it founds; families the observer sends together by the first of them.
pub fn founding_taste(
    params: &StyleParams,
    seed: u64,
    band: PermanentId,
    household: PermanentId,
) -> Taste {
    let tradition = tradition(params, seed, band);
    let own_sd = params.personal_spread.traits();
    let mut apart = Rng64::from_key(&[seed, PURPOSE_STYLE, band.get(), household.get()]);
    Taste::from_traits(std::array::from_fn(|k| {
        (tradition[k] + f64::from(own_sd[k]) * crate::demography::normal(&mut apart)) as f32
    }))
}

/// Band `band`'s way of building: drawn once around the content's (M3b slice R), the founding way
/// of the settlement a founding band is keyed by (M5b slice AR).
pub fn band_way(params: &StyleParams, seed: u64, band: PermanentId) -> Taste {
    Taste::from_traits(tradition(params, seed, band).map(|x| x as f32))
}

fn tradition(params: &StyleParams, seed: u64, band: PermanentId) -> [f64; Taste::TRAITS] {
    let (mean, band_sd) = (
        params.tradition_mean.traits(),
        params.tradition_spread.traits(),
    );
    let mut shared = Rng64::from_key(&[seed, PURPOSE_STYLE, band.get()]);
    std::array::from_fn(|k| {
        f64::from(mean[k]) + f64::from(band_sd[k]) * crate::demography::normal(&mut shared)
    })
}

/// The mean and spread of the traits over some households or buildings (M5b slice AR): pitch in
/// hundredths of a degree, eaves and overhang in centimetres, then the front's four (M8d slice DD)
/// in their units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spread {
    /// How many.
    pub n: usize,
    /// Each trait's mean.
    pub mean: [f64; Taste::TRAITS],
    /// Each trait's standard deviation.
    pub sd: [f64; Taste::TRAITS],
}

impl Spread {
    /// The spread of `list`.
    pub fn of(list: impl IntoIterator<Item = Taste>) -> Spread {
        let list: Vec<[f64; Taste::TRAITS]> = list
            .into_iter()
            .map(|t| t.traits().map(f64::from))
            .collect();
        let n = list.len();
        if n == 0 {
            return Spread::default();
        }
        let mean: [f64; Taste::TRAITS] =
            std::array::from_fn(|k| list.iter().map(|x| x[k]).sum::<f64>() / n as f64);
        let sd = std::array::from_fn(|k| {
            (list.iter().map(|x| (x[k] - mean[k]).powi(2)).sum::<f64>() / n as f64).sqrt()
        });
        Spread { n, mean, sd }
    }

    /// Its mean as a taste (`None` for none).
    pub fn mean_taste(&self) -> Option<Taste> {
        (self.n > 0).then(|| Taste::from_traits(self.mean.map(|x| x as f32)))
    }
}

/// A settlement's style on the three clocks (M5b slice AR; research 11-02 §4, opening: exposure,
/// implementation and the built stock change at different speeds): the way it was founded with,
/// its households' taste now, the year's new buildings and its standing buildings; and how many of
/// the year's new buildings follow, through their chain of followed buildings, one in another
/// settlement.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clocks {
    /// Its founding way, if known.
    pub way: Option<Taste>,
    /// Its households' taste now.
    pub taste: Spread,
    /// Its buildings finished in the year past that stand whole.
    pub new: Spread,
    /// Its finished buildings standing.
    pub stock: Spread,
    /// Of the year's new buildings, those whose chain of followed buildings reaches one in
    /// another settlement.
    pub new_after_elsewhere: usize,
}

/// The mean of `tastes`, trait by trait (`None` for none).
pub fn mean_taste<'a>(tastes: impl IntoIterator<Item = &'a Taste>) -> Option<Taste> {
    let (mut sum, mut n) = ([0.0f64; Taste::TRAITS], 0usize);
    for t in tastes {
        for (s, x) in sum.iter_mut().zip(t.traits()) {
            *s += f64::from(x);
        }
        n += 1;
    }
    (n > 0).then(|| Taste::from_traits(sum.map(|s| (s / n as f64) as f32)))
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

/// What program `def` allows of each trait, least and most: pitch, eaves and overhang, then the
/// front's four, which only a frame designed under version 3 has (ADR-0031 §7), and a store no
/// openings.
pub fn allowed(def: &BuildingDef) -> [(f32, f32); Taste::TRAITS] {
    let pair = |(lo, hi): (i32, i32)| (lo as f32, hi as f32);
    let none = (0.0, 0.0);
    match &def.rules {
        ProgramRules::Hut(r) => [
            pair(r.pitch_centideg),
            pair(r.eave_cm),
            (r.roof_overhang_cm as f32, r.roof_overhang_cm as f32),
            none,
            none,
            none,
            none,
        ],
        ProgramRules::Frame(r) => {
            let front = civ_grammar::FRAME_DESIGN_VERSION >= 3;
            let openings = front && def.use_ != civ_land::PlotUse::Store;
            [
                pair(r.pitch_centideg),
                pair(r.eave_cm),
                pair(r.overhang_cm),
                if openings {
                    pair((0, MAX_OPENING_PERMILLE))
                } else {
                    none
                },
                if front {
                    pair((0, MAX_ORNAMENT_DH))
                } else {
                    none
                },
                if front { (0.0, 2.0) } else { none },
                if front { (0.0, 1.0) } else { none },
            ]
        }
    }
}

/// `taste` held to what program `def` allows, in whole units: the traits a building of it is
/// built to.
pub fn held_to(taste: &Taste, def: &BuildingDef) -> [i32; Taste::TRAITS] {
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

/// The traits building `spec` of program `def` was built to: a frame's front as its ground
/// storey's openings, ornament, finish and symmetry (all 0 before version 3).
pub fn traits_of(spec: &BuildingSpec, def: &BuildingDef) -> Taste {
    use frame_params as fp;
    match &def.rules {
        ProgramRules::Hut(r) => Taste {
            pitch_centideg: spec.params[hut_params::PITCH_CENTIDEG] as f32,
            eave_cm: spec.params[hut_params::EAVE_CM] as f32,
            overhang_cm: r.roof_overhang_cm as f32,
            ..Taste::default()
        },
        ProgramRules::Frame(_) => Taste {
            pitch_centideg: spec.params[fp::PITCH_CENTIDEG] as f32,
            eave_cm: spec.params[fp::EAVE_CM] as f32,
            overhang_cm: spec.params[fp::OVERHANG_CM] as f32,
            openings_permille: spec.params[fp::OPENINGS] as f32,
            ornament_dh: spec.params[fp::ORNAMENT] as f32,
            finish: spec.params[fp::FINISH] as f32,
            symmetry: spec.params[fp::SYMMETRY] as f32,
        },
    }
}

/// `taste` moved `share` of the way toward `toward`, trait by trait.
pub fn moved(taste: &Taste, toward: &Taste, share: f64) -> Taste {
    moved_where(taste, toward, share, [true; Taste::TRAITS])
}

/// `taste` moved `share` of the way toward `toward` in the traits `which` names, the rest kept.
pub fn moved_where(
    taste: &Taste,
    toward: &Taste,
    share: f64,
    which: [bool; Taste::TRAITS],
) -> Taste {
    let (a, b) = (taste.traits(), toward.traits());
    let share = share.clamp(0.0, 1.0) as f32;
    Taste::from_traits(std::array::from_fn(|k| {
        if which[k] {
            a[k] + share * (b[k] - a[k])
        } else {
            a[k]
        }
    }))
}

/// How much more than the least admired a building is admired that stands `rank` of the way from
/// the least admired of its settlement's (0) to the most (1): from 1 to the style's
/// `prestige_most` (research 11-02 §2.2: 1-3 times a neutral exemplar).
pub fn prestige(params: &StyleParams, rank: f64) -> f64 {
    1.0 + (params.prestige_most.max(1.0) - 1.0) * rank.clamp(0.0, 1.0)
}

/// The traits a household of taste `taste` builds program `def` to, at commission `key` (keyed
/// draws): its taste held to what the program allows, and now and then, at the style's
/// `innovation` chance, one of the roof's three traits new to it, anywhere the program allows;
/// and, at the same chance drawn apart, one of the front's four (M8d slice DD), so that a
/// program without a front draws as before.
pub fn commission(
    params: &StyleParams,
    taste: &Taste,
    def: &BuildingDef,
    key: &[u64],
) -> [i32; Taste::TRAITS] {
    let mut traits = held_to(taste, def);
    let a = allowed(def);
    let mut rng = Rng64::from_key(key);
    if rng.chance(params.innovation) {
        let k = rng.below(Taste::ROOF_TRAITS as u64) as usize;
        traits[k] = rng.range_f64(f64::from(a[k].0), f64::from(a[k].1)).round() as i32;
    }
    let front = Taste::ROOF_TRAITS..Taste::TRAITS;
    if front.clone().any(|k| a[k].1 > a[k].0) {
        let mut key = key.to_vec();
        key.push(PURPOSE_FRONT);
        let mut rng = Rng64::from_key(&key);
        if rng.chance(params.innovation) {
            let k = Taste::ROOF_TRAITS + rng.below(front.len() as u64) as usize;
            traits[k] = rng.range_f64(f64::from(a[k].0), f64::from(a[k].1)).round() as i32;
        }
    }
    traits
}

/// How far about a building its neighbours stand whose eaves it must rise over to be seen
/// farther (M8d slice DD; ADR-0031 §7; a design prior), metres.
pub const NEIGHBOURS_M: f64 = 30.0;
/// The least height of the eaves a building is seen over, metres: a person's line of sight
/// across open ground (a design prior).
pub const LEAST_EAVES_M: f64 = 2.0;
/// The most a building's height multiplies how far it is seen (a design prior).
pub const MOST_SIGHT_SCALE: f64 = 4.0;

/// The height of building `spec`'s eaves above the ground, metres: a hut's walls; a frame's
/// raised floor and every storey.
pub fn eaves_m(spec: &BuildingSpec) -> f64 {
    match spec.footprint {
        civ_grammar::Footprint::Round { .. } => f64::from(spec.params[hut_params::EAVE_CM]) / 100.0,
        civ_grammar::Footprint::Rect { .. } => {
            let p = &spec.params;
            f64::from(
                p[frame_params::FLOOR_RAISE_CM]
                    + p[frame_params::EAVE_CM] * i32::from(spec.storeys.max(1)),
            ) / 100.0
        }
    }
}

/// How far building `b` is seen, metres, by someone who sees an ordinary building `sight_m` off
/// (M8d slice DD; ADR-0031 §7): `sight_m` times its eaves' height over the highest eaves of the
/// standing buildings within [`NEIGHBOURS_M`] of it (never under [`LEAST_EAVES_M`]), from one
/// to [`MOST_SIGHT_SCALE`] times. A hall or a tower among huts is seen from farther away.
pub fn seen_from_m(land: &civ_land::Land, b: &civ_land::Building, sight_m: f64) -> f64 {
    let c = crate::build::centre_m(&b.spec);
    let around = land
        .buildings
        .iter()
        .filter(|o| o.id != b.id && o.finished() && o.state == civ_land::BuildingState::Standing)
        .filter(|o| {
            let d = crate::build::centre_m(&o.spec);
            f64::from((d.0 - c.0).hypot(d.1 - c.1)) <= NEIGHBOURS_M
        })
        .map(|o| eaves_m(&o.spec))
        .fold(LEAST_EAVES_M, f64::max);
    sight_m * (eaves_m(&b.spec) / around).clamp(1.0, MOST_SIGHT_SCALE)
}

/// What frame building `spec` was built with beyond what version 2 builds, in words (ADR-0031
/// §1): a trussed roof, posts a storey high or a steel skeleton (version 4, ADR-0037 §3), walls
/// tied at each storey, walls thinning storey by storey, each storey's windows and a stair. Empty
/// before version 3.
pub fn system_words(spec: &BuildingSpec) -> Vec<String> {
    use frame_params as fp;
    if spec.version < 3 {
        return Vec::new();
    }
    let p = &spec.params;
    let storeys = usize::from(spec.storeys).min(4);
    let ordinal = |t: usize| ["first", "second", "third", "fourth"][t];
    let mut words = Vec::new();
    if p[fp::ROOF_SYSTEM] == 1 {
        words.push("a trussed roof".to_owned());
    }
    if p[fp::POST_SYSTEM] == 1 {
        words.push("posts a storey high".to_owned());
    }
    if p[fp::POST_SYSTEM] == 2 {
        words.push("a steel skeleton".to_owned());
    }
    if p[fp::TIES] == 1 {
        words.push("walls tied at each storey".to_owned());
    }
    if (0..storeys).any(|t| p[fp::STOREY_WALL_CM + t] != 0) {
        let cm: Vec<String> = (0..storeys)
            .map(|t| match p[fp::STOREY_WALL_CM + t] {
                0 => p[fp::WALL_CM],
                v => v,
            })
            .map(|v| v.to_string())
            .collect();
        words.push(format!("walls {} cm thick storey by storey", cm.join(", ")));
    }
    let open: Vec<String> = (0..storeys)
        .filter(|&t| p[fp::OPENINGS + t] != 0)
        .map(|t| {
            format!(
                "{}% of the {} storey's wall",
                (f64::from(p[fp::OPENINGS + t]) / 10.0).round(),
                ordinal(t)
            )
        })
        .collect();
    if !open.is_empty() {
        words.push(format!("windows in {}", open.join(", ")));
    }
    if p[fp::STAIR] == 1 {
        words.push("a stair to every storey".to_owned());
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The draws as M3b made them, before the band's way was split out (M5b slice AR): a
    /// household's taste interleaves its band's draw and its own, trait by trait.
    fn founding_taste_as_before(
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

    #[test]
    fn founding_taste_draws_exactly_as_before_the_band_s_way_was_split_out() {
        let p = params();
        let id = |n: u64| PermanentId::from_raw(n).expect("an id");
        for seed in [1, 7, 42] {
            for band in [1, 51, 890] {
                for household in [2, 3, 99] {
                    assert_eq!(
                        founding_taste(&p, seed, id(band), id(household)),
                        founding_taste_as_before(&p, seed, id(band), id(household))
                    );
                }
                // The band's way is what its households' tastes are drawn around.
                let way = band_way(&p, seed, id(band)).traits();
                let mean = p.tradition_mean.traits();
                let sd = p.tradition_spread.traits();
                for k in 0..3 {
                    assert!((way[k] - mean[k]).abs() <= 6.0 * sd[k].max(1e-6), "{k}");
                }
            }
        }
    }

    /// A commission as M3b made it, before the front was laid out: the roof's three traits held
    /// to the program, and now and then one of them new.
    fn commission_as_before(
        params: &StyleParams,
        taste: &Taste,
        def: &BuildingDef,
        key: &[u64],
    ) -> [i32; 3] {
        let held = held_to(taste, def);
        let mut traits = [held[0], held[1], held[2]];
        let mut rng = Rng64::from_key(key);
        if rng.chance(params.innovation) {
            let a = allowed(def);
            let k = rng.below(3) as usize;
            traits[k] = rng.range_f64(f64::from(a[k].0), f64::from(a[k].1)).round() as i32;
        }
        traits
    }

    #[test]
    fn a_hut_is_commissioned_exactly_as_before_and_has_no_front() {
        let p = StyleParams {
            innovation: 0.5,
            ..params()
        };
        let def = crate::build::tests::def();
        let taste = Taste {
            openings_permille: 120.0,
            ornament_dh: 3.0,
            finish: 1.0,
            symmetry: 1.0,
            ..p.tradition_mean
        };
        assert!(
            allowed(&def)[Taste::ROOF_TRAITS..]
                .iter()
                .all(|a| *a == (0.0, 0.0))
        );
        let mut innovated = 0;
        for n in 0..400_u64 {
            let key = [7, n, 3];
            let now = commission(&p, &taste, &def, &key);
            let before = commission_as_before(&p, &taste, &def, &key);
            assert_eq!(now[..3], before, "{n}");
            assert_eq!(now[3..], [0; 4], "{n}");
            innovated += usize::from(
                before
                    != [
                        held_to(&taste, &def)[0],
                        held_to(&taste, &def)[1],
                        held_to(&taste, &def)[2],
                    ],
            );
        }
        assert!(innovated > 50, "{innovated}");
    }

    #[test]
    fn a_taste_moves_only_in_the_traits_named() {
        let p = params();
        let from = p.tradition_mean;
        let to = Taste {
            pitch_centideg: 5_000.0,
            openings_permille: 300.0,
            ornament_dh: 10.0,
            ..from
        };
        let mut which = [true; Taste::TRAITS];
        which[4] = false;
        let m = moved_where(&from, &to, 0.5, which);
        assert!((m.pitch_centideg - 4_750.0).abs() < 1e-3);
        assert!((m.openings_permille - 150.0).abs() < 1e-3);
        assert_eq!(m.ornament_dh, from.ornament_dh, "ornament kept");
        assert_eq!(moved(&from, &to, 0.5).ornament_dh, 5.0);
    }

    fn params() -> StyleParams {
        StyleParams {
            alpha: 0.1,
            prestige_most: 3.0,
            innovation: 0.0,
            tradition_mean: Taste {
                pitch_centideg: 4_500.0,
                eave_cm: 180.0,
                overhang_cm: 50.0,
                ..Default::default()
            },
            tradition_spread: Taste {
                pitch_centideg: 400.0,
                eave_cm: 15.0,
                overhang_cm: 10.0,
                ..Default::default()
            },
            personal_spread: Taste {
                pitch_centideg: 100.0,
                eave_cm: 4.0,
                overhang_cm: 3.0,
                ..Default::default()
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
            ..Default::default()
        };
        let (taste, admired) = couple_taste((&hers, Some(id(4))), (&his, Some(id(5))));
        assert_eq!(
            taste,
            Taste {
                pitch_centideg: 4_800.0,
                eave_cm: 190.0,
                overhang_cm: 55.0,
                ..Default::default()
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
            ..Default::default()
        };
        let m = moved(&from, &to, 0.1);
        assert!((m.pitch_centideg - 4_600.0).abs() < 1e-3);
        assert!((m.eave_cm - 184.0).abs() < 1e-3);
        assert_eq!(moved(&from, &to, 2.0), to, "never past it");
        assert_eq!(prestige(&p, 0.0), 1.0);
        assert_eq!(prestige(&p, 1.0), 3.0);
        assert_eq!(prestige(&p, 0.5), 2.0);
    }

    #[test]
    fn a_version_three_frame_says_what_it_was_built_with() {
        use civ_grammar::{Footprint, PARAMS, frame_params as fp};
        let mut params = [0; PARAMS];
        params[fp::WALL_CM] = 50;
        let mut spec = BuildingSpec {
            program: "core:building/longhouse".into(),
            version: 2,
            footprint: Footprint::Rect {
                x: 0,
                y: 0,
                length: 1000,
                width: 500,
                angle: 0,
            },
            storeys: 3,
            params,
            materials: Vec::new(),
            style_seed: 0,
        };
        assert!(
            system_words(&spec).is_empty(),
            "nothing of its own before version 3"
        );
        spec.version = 3;
        assert!(
            system_words(&spec).is_empty(),
            "version 3 with nothing of its own"
        );
        spec.params[fp::ROOF_SYSTEM] = 1;
        spec.params[fp::TIES] = 1;
        spec.params[fp::STOREY_WALL_CM + 1] = 40;
        spec.params[fp::STOREY_WALL_CM + 2] = 30;
        spec.params[fp::OPENINGS] = 200;
        spec.params[fp::OPENINGS + 2] = 100;
        spec.params[fp::STAIR] = 1;
        assert_eq!(
            system_words(&spec),
            [
                "a trussed roof",
                "walls tied at each storey",
                "walls 50, 40, 30 cm thick storey by storey",
                "windows in 20% of the first storey's wall, 10% of the third storey's wall",
                "a stair to every storey",
            ]
        );
        // Version 4's steel skeleton (ADR-0037 §3).
        let mut skeleton = spec.clone();
        skeleton.version = 4;
        skeleton.params = [0; civ_grammar::PARAMS];
        skeleton.params[fp::POST_SYSTEM] = 2;
        assert_eq!(system_words(&skeleton), ["a steel skeleton"]);
    }
}
