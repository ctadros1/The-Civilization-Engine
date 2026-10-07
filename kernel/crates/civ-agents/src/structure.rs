//! Loads, margins and what shows of them (ADR-0009 §5; M3b slice P): what each component group of
//! a building carries, how near it is to giving way, and whether it sags or shows the strain.
//! Margins are derived from the expansion, the condition and the loads whenever they are needed,
//! and never saved.
//!
//! Beams, joists and rafters are simply supported members under a load spread along them:
//! bending stress from `M = W·L/8` over the section modulus, deflection `5·W·L³/(384·E·I)`, with
//! load carried for years bending them `(1 + φ)` times as far (research 11-05 §5.2, §2.4). Posts
//! carry their share of the building's weight against buckling (Euler's load, reduced for
//! crooked posts and loose joints) or crushing, whichever comes first. A group's quality scales
//! what it can carry; what it has lost scales its section's strength by `(1 − x)³` in bending and
//! `(1 − x)⁴` in buckling (11-05 §5.1).
//!
//! Nothing here decides who builds, loads or mends what.

use civ_grammar::{Expansion, Group, GroupKind, Level};
use civ_land::{GroupCondition, GroupState};

/// Standard gravity, m/s².
pub const G: f64 = 9.81;

/// A member's own weight: oak at 760 kg/m³ (research 11-08 §2.1).
const TIMBER_KG_M3: f64 = 760.0;

/// The margin below which a group shows the strain: it creaks, cracks or leans (a tuning value).
pub const STRAIN_SHOWS: f64 = 1.5;

/// Deflection over span past which a member visibly sags: 11-05 §2.4's screen of L/180, a
/// serviceability limit, never a failure. It is checked on members that carry lofts, floors and
/// beams, where a sag hinders their use. A roof frame's bow under its covering is not: 11-05 §2.4
/// gives the screen as a later engineering analogue, not as past practice, and thatched rafters
/// bow without harm, so a roof frame shows only its strain.
pub const SAG_SHOWS: f64 = 1.0 / 180.0;

/// How far short of Euler's load a real post buckles: crooked poles and loose joints (11-05 §5.2
/// asks for an imperfection model; a tuning value).
pub const POST_IMPERFECTION: f64 = 0.5;

/// A timber as the members of a building are made of it (research 11-05 §2.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timber {
    /// Bending strength (modulus of rupture), pascals.
    pub bending_pa: f64,
    /// Strength in compression along the grain, pascals.
    pub compression_pa: f64,
    /// Stiffness (modulus of elasticity), pascals.
    pub stiffness_pa: f64,
    /// How much further a member bends under load it carries for years than at first (φ).
    pub creep: f64,
    /// The share of its strength it keeps under load carried for years.
    pub sustained: f64,
}

/// What presses on a building now, pascals on the area each acts on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Loads {
    /// The roof covering's own weight, per square metre of roof slope (carried for years).
    pub covering_pa: f64,
    /// The weather on the roof, per square metre of plan: the month's storm on its day and the
    /// snow lying on it (passing; ADR-0012 §5).
    pub peak_pa: f64,
    /// Goods in its lofts, per square metre of loft (carried for years).
    pub loft_pa: f64,
    /// Goods on a raised floor, per square metre of it (carried for years).
    pub raised_pa: f64,
    /// Goods on an upper storey's floor, per square metre of it (carried for years).
    pub upper_pa: f64,
    /// People and what they carry on a floor off the ground, per square metre (passing).
    pub live_pa: f64,
}

/// How near a group is to giving way under its loads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Margin {
    /// What it can carry over what it must, in its mode: bending for beams, joists and rafters,
    /// buckling or crushing for posts. Infinite for a group nothing loads this way.
    pub strength: f64,
    /// Its members' long-term deflection over their span (0 for posts and roof frames, whose
    /// sag shows nothing: [`SAG_SHOWS`]).
    pub sag: f64,
}

impl Margin {
    /// A group that carries nothing this way.
    pub const NONE: Margin = Margin {
        strength: f64::INFINITY,
        sag: 0.0,
    };

    /// How the group stands under its loads alone: failed below a margin of 1, showing below
    /// [`STRAIN_SHOWS`] or once it sags past [`SAG_SHOWS`], otherwise sound.
    pub fn state(self) -> GroupState {
        if self.strength < 1.0 {
            GroupState::Failed
        } else if self.strength < STRAIN_SHOWS || self.sag > SAG_SHOWS {
            GroupState::Symptom
        } else {
            GroupState::Sound
        }
    }
}

/// A member's section: area (m²), section modulus (m³) and second moment of area (m⁴). Equal
/// sides are a round pole of that diameter; unequal, a sawn or split section of that width and
/// depth.
fn section(g: &Group) -> (f64, f64, f64) {
    let [w, d] = g.section_cm.map(|v| f64::from(v.max(0)) / 100.0);
    if (w - d).abs() < 1e-9 {
        let pi = std::f64::consts::PI;
        (
            pi * d * d / 4.0,
            pi * d.powi(3) / 32.0,
            pi * d.powi(4) / 64.0,
        )
    } else {
        (w * d, w * d * d / 6.0, w * d.powi(3) / 12.0)
    }
}

/// Square metres of the roof's plan: a hut's circle, or a frame's gabled roof's edge.
pub fn roof_plan_m2(e: &Expansion) -> f64 {
    let pts = &e.roof_outline;
    if pts.len() >= 3 {
        let mut twice = 0.0;
        for (i, &(x0, y0)) in pts.iter().enumerate() {
            let (x1, y1) = pts[(i + 1) % pts.len()];
            twice += f64::from(x0) * f64::from(y1) - f64::from(x1) * f64::from(y0);
        }
        (twice / 2.0).abs() * 1e-4
    } else {
        let r = f64::from(e.roof_radius_cm) / 100.0;
        std::f64::consts::PI * r * r
    }
}

/// Square metres of roof slope: its coverings' area.
fn roof_slope_m2(e: &Expansion) -> f64 {
    e.groups
        .iter()
        .filter(|g| g.kind == GroupKind::Covering)
        .map(|g| g.area_m2.max(0.0))
        .sum()
}

/// The cosine of the roof's pitch: its plan over its slopes' area (1 for a building without a
/// covering).
pub fn roof_cos(e: &Expansion) -> f64 {
    let slope = roof_slope_m2(e);
    if slope > 0.0 {
        (roof_plan_m2(e) / slope).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

/// A member's own weight, newtons.
fn own_weight(g: &Group) -> f64 {
    let (area, _, _) = section(g);
    area * f64::from(g.length_cm.max(0)) / 100.0 * TIMBER_KG_M3 * G
}

/// The level a group's id names.
fn level_of(g: &Group) -> Option<Level> {
    match (g.id >> 24) & 0xff {
        0 => Some(Level::Ground),
        1 => Some(Level::Upper),
        2 => Some(Level::Loft),
        3 => Some(Level::Roof),
        _ => None,
    }
}

/// What one member of bending group `g` of building `e` carries, newtons spread along it, as
/// load carried for years and load in passing. `None` for a group that does not bend.
fn bending_load(g: &Group, e: &Expansion, loads: &Loads) -> Option<(f64, f64)> {
    let per = g.area_m2.max(0.0) / f64::from(g.count.max(1));
    let own = own_weight(g);
    Some(match (g.kind, level_of(g)) {
        (GroupKind::RoofFrame, _) => {
            // Normal to the slope: the covering's weight, and the peak on the plan it covers.
            let c = roof_cos(e);
            (
                (loads.covering_pa * per + own) * c,
                loads.peak_pa * per * c * c,
            )
        }
        (GroupKind::LoftJoists, _) | (GroupKind::TieBeams, Some(Level::Roof)) => {
            (loads.loft_pa * per + own, 0.0)
        }
        (GroupKind::RaisedFloor, _) | (GroupKind::TieBeams, Some(Level::Ground)) => {
            (loads.raised_pa * per + own, loads.live_pa * per)
        }
        (GroupKind::FloorJoists, _) | (GroupKind::TieBeams, _) => {
            (loads.upper_pa * per + own, loads.live_pa * per)
        }
        _ => return None,
    })
}

/// The whole weight on building `e`'s posts, newtons, as load carried for years and load in
/// passing: its roof's covering and the month's peak, what its lofts and floors off the ground
/// hold, and its own timber above the ground.
fn weight_on_posts(e: &Expansion, loads: &Loads) -> (f64, f64) {
    let area = |level: Level| -> f64 {
        e.spaces
            .iter()
            .filter(|s| s.level == level)
            .map(|s| s.area_m2)
            .sum()
    };
    let raised = if e.groups.iter().any(|g| g.kind == GroupKind::RaisedFloor) {
        area(Level::Ground)
    } else {
        0.0
    };
    let upper = area(Level::Upper);
    let own: f64 = e
        .groups
        .iter()
        .filter(|g| {
            g.material.is_some()
                && !matches!(
                    g.kind,
                    GroupKind::Posts | GroupKind::Covering | GroupKind::Infill | GroupKind::Floor
                )
        })
        .map(|g| own_weight(g) * f64::from(g.count))
        .sum();
    (
        loads.covering_pa * roof_slope_m2(e)
            + loads.loft_pa * area(Level::Loft)
            + loads.raised_pa * raised
            + loads.upper_pa * upper
            + own,
        loads.peak_pa * roof_plan_m2(e) + loads.live_pa * (raised + upper),
    )
}

/// How near group `g` of building `e`, in condition `c`, is to giving way under `loads`, made of
/// `t` (ADR-0009 §5). Coverings, infill and floors on the ground carry nothing this way.
pub fn margin(g: &Group, c: &GroupCondition, e: &Expansion, loads: &Loads, t: &Timber) -> Margin {
    let q = f64::from(c.quality).max(0.0);
    let left = (1.0 - f64::from(c.loss)).clamp(0.0, 1.0);
    let (area, z, i) = section(g);
    let span = f64::from(g.length_cm.max(0)) / 100.0;
    if g.material.is_none() || span <= 0.0 || z <= 0.0 {
        return Margin::NONE;
    }
    let keep = t.sustained.max(1e-6);
    if g.kind == GroupKind::Posts {
        let total: f64 = e
            .groups
            .iter()
            .filter(|p| p.kind == GroupKind::Posts)
            .map(|p| p.area_m2.max(0.0))
            .sum();
        if total <= 0.0 {
            return Margin::NONE;
        }
        let share = g.area_m2.max(0.0) / total / f64::from(g.count.max(1));
        let (held, passing) = weight_on_posts(e, loads);
        let demand = held * share / keep + passing * share;
        if demand <= 0.0 {
            return Margin::NONE;
        }
        let pi = std::f64::consts::PI;
        let buckling =
            pi * pi * t.stiffness_pa * i * left.powi(4) / (span * span) * POST_IMPERFECTION;
        let crushing = t.compression_pa * area * left.powi(2);
        return Margin {
            strength: q * buckling.min(crushing) / demand,
            sag: 0.0,
        };
    }
    let Some((held, passing)) = bending_load(g, e, loads) else {
        return Margin::NONE;
    };
    let demand = (held / keep + passing) * span / 8.0;
    if demand <= 0.0 {
        return Margin::NONE;
    }
    let capacity = t.bending_pa * z * q * left.powi(3);
    let stiffness = t.stiffness_pa * i * left.powi(4);
    let sag = if g.kind == GroupKind::RoofFrame {
        0.0
    } else if stiffness > 0.0 {
        5.0 * span.powi(3) / (384.0 * stiffness) * ((1.0 + t.creep) * held + passing) / span
    } else {
        f64::INFINITY
    };
    Margin {
        strength: capacity / demand,
        sag,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_core::SimTime;
    use civ_grammar::{Stage, group_id};

    /// Green oak (11-05 §2.3), with creep of 1 (11-05 §2.4) and 0.6 of its strength kept under
    /// years of load.
    fn oak() -> Timber {
        Timber {
            bending_pa: 57e6,
            compression_pa: 24.5e6,
            stiffness_pa: 8.6e9,
            creep: 1.0,
            sustained: 0.6,
        }
    }

    fn sound(quality: f32) -> GroupCondition {
        GroupCondition {
            group: 0,
            quality,
            loss: 0.0,
            installed: SimTime::ZERO,
            repaired: SimTime::ZERO,
            state: GroupState::Sound,
        }
    }

    /// Joists of a loft over one bay: `count` round poles of diameter `d_cm` spanning `span_cm`,
    /// carrying `area_m2`.
    fn joists(d_cm: i32, span_cm: i32, count: u32, area_m2: f64) -> Group {
        Group {
            id: group_id(Level::Loft, Some(0), GroupKind::LoftJoists, 0),
            kind: GroupKind::LoftJoists,
            stage: Stage::Frame,
            material: Some(0),
            count,
            section_cm: [d_cm, d_cm],
            length_cm: span_cm,
            spacing_cm: 60,
            area_m2,
        }
    }

    /// A hut's expansion with `groups` in place of its own, and no spaces.
    fn expansion_with(groups: Vec<Group>) -> Expansion {
        let mut params = [0; civ_grammar::PARAMS];
        params[civ_grammar::hut_params::EAVE_CM] = 180;
        params[civ_grammar::hut_params::PITCH_CENTIDEG] = 4500;
        let spec = civ_grammar::BuildingSpec {
            program: "core:building/hut".into(),
            version: civ_grammar::HUT_VERSION,
            footprint: civ_grammar::Footprint::Round {
                x: 10_000,
                y: 10_000,
                radius: 300,
            },
            storeys: 1,
            params,
            materials: vec!["t".into(), "t".into(), "h".into()],
            style_seed: 0,
        };
        let rules = civ_grammar::ProgramRules::Hut(crate::build::tests::hut_rules());
        let mut e = civ_grammar::expand(&spec, &rules).expect("a hut");
        e.groups = groups;
        e.spaces.clear();
        e
    }

    #[test]
    fn a_loft_over_a_wide_bay_strains_where_a_narrow_one_holds() {
        // A full loft, 150 kg a square metre, on 15 cm poles 0.6 m apart.
        let loads = Loads {
            loft_pa: 150.0 * G,
            ..Loads::default()
        };
        let e = expansion_with(vec![joists(15, 500, 4, 4.0 * 0.6 * 5.0)]);
        let m = margin(&e.groups[0], &sound(0.76), &e, &loads, &oak());
        // M = W·L/8, W = 1471.5 Pa × 3 m² and the pole's own weight, at 0.6 of strength.
        let pi = std::f64::consts::PI;
        let z = pi * 0.15f64.powi(3) / 32.0;
        let own = pi * 0.15 * 0.15 / 4.0 * 5.0 * 760.0 * G;
        let w = 150.0 * G * 3.0 + own;
        let expected = 57e6 * z * f64::from(0.76f32) / ((w / 0.6) * 5.0 / 8.0);
        assert!(
            (m.strength - expected).abs() < 1e-9,
            "{} {expected}",
            m.strength
        );
        assert!(
            m.strength > STRAIN_SHOWS,
            "a 5 m loft holds: {}",
            m.strength
        );
        // Over 7 m the bending demand is about twice as much (L²): it shows the strain, and a
        // novice's poorest joists give way.
        let e = expansion_with(vec![joists(15, 700, 4, 4.0 * 0.6 * 7.0)]);
        let fair = margin(&e.groups[0], &sound(0.76), &e, &loads, &oak());
        assert!(
            fair.strength < STRAIN_SHOWS && fair.strength > 1.0,
            "{}",
            fair.strength
        );
        assert_eq!(fair.state(), GroupState::Symptom);
        let poor = margin(&e.groups[0], &sound(0.4), &e, &loads, &oak());
        assert_eq!(poor.state(), GroupState::Failed, "{}", poor.strength);
        // 15 cm poles over 7 m under a full loft bend far past L/180.
        assert!(fair.sag > SAG_SHOWS, "{}", fair.sag);
    }

    #[test]
    fn a_huts_rafters_bow_under_thatch_without_showing() {
        // A hut's roof at 45°: twenty 10 cm rafters from the eaves to the apex under thatch at
        // 30 kg/m² of slope.
        let group = |kind: GroupKind, count: u32, d_cm: i32, length_cm: i32| Group {
            id: group_id(Level::Roof, None, kind, 0),
            kind,
            stage: Stage::Frame,
            material: Some(0),
            count,
            section_cm: [d_cm, d_cm],
            length_cm,
            spacing_cm: 100,
            area_m2: 0.0,
        };
        let mut e = expansion_with(Vec::new());
        let plan = roof_plan_m2(&e);
        let slant = (plan / std::f64::consts::PI).sqrt() * std::f64::consts::SQRT_2;
        let slope = plan * std::f64::consts::SQRT_2;
        let mut rafters = group(GroupKind::RoofFrame, 20, 10, (slant * 100.0) as i32);
        rafters.area_m2 = slope;
        let mut covering = group(GroupKind::Covering, 1, 0, 0);
        covering.area_m2 = slope;
        e.groups = vec![rafters, covering];
        let r = &e.groups[0];
        let calm = Loads {
            covering_pa: 30.0 * G,
            ..Loads::default()
        };
        let m = margin(r, &sound(0.8), &e, &calm, &oak());
        // Over the years they would bend past L/180 under the thatch alone...
        let span = f64::from(r.length_cm) / 100.0;
        let (_, _, i) = section(r);
        let w = (calm.covering_pa * slope / 20.0 + own_weight(r)) * roof_cos(&e);
        let bow = 5.0 * w * span.powi(3) / (384.0 * 8.6e9 * i) * 2.0 / span;
        assert!(bow > SAG_SHOWS, "{bow}");
        // ...which shows nothing: the roof is sound on a calm day.
        assert_eq!(m.sag, 0.0);
        assert_eq!(m.state(), GroupState::Sound, "{}", m.strength);
        // What a storm does to them shows as their strain, and then as their breaking.
        let storm = |peak_pa: f64| {
            let loads = Loads { peak_pa, ..calm };
            margin(r, &sound(0.8), &e, &loads, &oak())
        };
        let mut seen = Vec::new();
        for kpa in 1..=20 {
            seen.push(storm(f64::from(kpa) * 1_000.0).state());
        }
        seen.dedup();
        assert_eq!(
            seen,
            vec![GroupState::Sound, GroupState::Symptom, GroupState::Failed]
        );
    }

    #[test]
    fn rot_takes_strength_by_the_cube_and_stiffness_by_the_fourth_power() {
        let loads = Loads {
            loft_pa: 150.0 * G,
            ..Loads::default()
        };
        let e = expansion_with(vec![joists(15, 500, 4, 6.0)]);
        let whole = margin(&e.groups[0], &sound(0.76), &e, &loads, &oak());
        let mut rotten = sound(0.76);
        rotten.loss = 0.2;
        let r = margin(&e.groups[0], &rotten, &e, &loads, &oak());
        let left = 1.0 - f64::from(0.2f32);
        assert!((r.strength / whole.strength - left.powi(3)).abs() < 1e-9);
        assert!((r.sag / whole.sag - 1.0 / left.powi(4)).abs() < 1e-9);
        // An empty loft carries only its own joists.
        let empty = margin(&e.groups[0], &sound(0.76), &e, &Loads::default(), &oak());
        assert!(empty.strength > 20.0, "{}", empty.strength);
    }

    #[test]
    fn posts_give_way_only_once_rot_has_eaten_most_of_them() {
        // A hut's twenty 12 cm posts, 1.8 m to the eaves, under 40 m² of roof.
        let posts = Group {
            id: group_id(Level::Ground, None, GroupKind::Posts, 0),
            kind: GroupKind::Posts,
            stage: Stage::Frame,
            material: Some(0),
            count: 20,
            section_cm: [12, 12],
            length_cm: 180,
            spacing_cm: 100,
            area_m2: 40.0,
        };
        let covering = Group {
            id: group_id(Level::Roof, None, GroupKind::Covering, 0),
            kind: GroupKind::Covering,
            stage: Stage::Roof,
            material: Some(2),
            count: 1,
            section_cm: [30, 30],
            length_cm: 500,
            spacing_cm: 0,
            area_m2: 56.6,
        };
        let mut e = expansion_with(vec![posts, covering]);
        e.roof_radius_cm = 357;
        let loads = Loads {
            covering_pa: 30.0 * G,
            peak_pa: 600.0,
            ..Loads::default()
        };
        let m = margin(&e.groups[0], &sound(0.76), &e, &loads, &oak());
        assert!(
            m.strength > 20.0,
            "sound posts carry a hut many times over: {}",
            m.strength
        );
        let mut rotten = sound(0.76);
        rotten.loss = 0.8;
        let r = margin(&e.groups[0], &rotten, &e, &loads, &oak());
        assert!(
            r.strength < 1.0,
            "four fifths rotted through: {}",
            r.strength
        );
        // The covering carries nothing this way.
        assert_eq!(
            margin(&e.groups[1], &sound(0.76), &e, &loads, &oak()),
            Margin::NONE
        );
    }
}
