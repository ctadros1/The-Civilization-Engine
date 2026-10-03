//! Building designs and what they expand into (ADR-0004 §2; research 11-03).
//!
//! A [`BuildingSpec`] records what a builder decided: the program, the grammar version it was
//! designed under, the footprint (authoritative: the grammar never moves it), integer parameters
//! and the materials. [`expand_hut`] is a pure function from a spec and its program's rules to the
//! building's parts, outline and door, the labour and materials each construction stage needs, and
//! derived facts (floor area, sleeping places). It has no clock, world access or shared
//! randomness, so the kernel and Unreal expand a saved spec to the same building. Golden-hash
//! tests pin what each grammar version produces; a change to what a rule produces bumps
//! [`HUT_VERSION`], and specs keep the version they were designed under.
//!
//! M1 has one program, the hut: a round post-built house with wattle-and-daub walls and a conical
//! thatched roof. Its rules are code; every dimension, labour and material figure comes from
//! content ([`HutRules`]). Geometry is in integer centimetres and angles in 1/65,536 of a turn
//! (ADR-0004 §1); floating point is used only on the way.

#![forbid(unsafe_code)]

use std::f64::consts::TAU;

/// Version of the hut grammar.
pub const HUT_VERSION: u16 = 1;

/// One full turn in angle units (ADR-0004 §1).
pub const TURN: f64 = 65_536.0;

/// Where a building stands, in integer centimetres from the map's north-west corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Footprint {
    /// A round building: the centre and the radius of its wall line.
    Round {
        /// Centre, east.
        x: i32,
        /// Centre, south.
        y: i32,
        /// Radius of the wall line.
        radius: i32,
    },
}

/// Indexes of a hut's parameters in [`BuildingSpec::params`].
pub mod hut_params {
    /// Wall height to the eaves, centimetres.
    pub const EAVE_CM: usize = 0;
    /// Roof pitch, hundredths of a degree.
    pub const PITCH_CENTIDEG: usize = 1;
    /// Direction the door faces, in 1/65,536 of a turn from east toward south.
    pub const DOOR_DIR: usize = 2;
}

/// A hut's material slots, indexes into [`BuildingSpec::materials`].
pub mod hut_materials {
    /// Posts and rafters.
    pub const TIMBER: u8 = 0;
    /// Rods woven between the posts.
    pub const WATTLE: u8 = 1;
    /// The roof covering.
    pub const THATCH: u8 = 2;
    /// Number of slots.
    pub const COUNT: usize = 3;
}

/// What a builder decided (ADR-0004 §2).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuildingSpec {
    /// The program's content id, for example `core:building/hut`.
    pub program: String,
    /// The grammar version it was designed under.
    pub version: u16,
    /// Where it stands.
    pub footprint: Footprint,
    /// Storeys above ground.
    pub storeys: u8,
    /// Up to eight integer parameters; their meaning is the program's ([`hut_params`]).
    pub params: [i32; 8],
    /// The good each material slot is made of, as content ids ([`hut_materials`]).
    pub materials: Vec<String>,
    /// Keys any variation the grammar draws (the hut draws none).
    pub style_seed: u64,
}

/// A construction stage (ADR-0004 §2), in the order the work goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stage {
    /// Ground marked out, postholes dug.
    Foundation,
    /// Posts set, rafters raised.
    Frame,
    /// Wattle woven and daubed.
    Walls,
    /// Thatched.
    Roof,
    /// Floor beaten and hearth laid.
    Finish,
}

impl Stage {
    /// Every stage, in order.
    pub const ALL: [Stage; 5] = [
        Stage::Foundation,
        Stage::Frame,
        Stage::Walls,
        Stage::Roof,
        Stage::Finish,
    ];

    /// Position in [`Stage::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }

    /// The stage at `index`, if any.
    pub fn from_index(index: usize) -> Option<Stage> {
        Stage::ALL.get(index).copied()
    }

    /// The stage's name.
    pub fn name(self) -> &'static str {
        match self {
            Stage::Foundation => "foundation",
            Stage::Frame => "frame",
            Stage::Walls => "walls",
            Stage::Roof => "roof",
            Stage::Finish => "finish",
        }
    }
}

/// What kind of piece a part is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartKind {
    /// A hole a post stands in.
    Posthole,
    /// An upright.
    Post,
    /// A roof timber from a post's head to the apex.
    Rafter,
    /// Rods woven between two posts.
    Wattle,
    /// Earth plastered on the wattle.
    Daub,
    /// The roof covering.
    Thatch,
    /// The beaten floor.
    Floor,
    /// The hearth.
    Hearth,
}

impl PartKind {
    fn code(self) -> u8 {
        match self {
            PartKind::Posthole => 0,
            PartKind::Post => 1,
            PartKind::Rafter => 2,
            PartKind::Wattle => 3,
            PartKind::Daub => 4,
            PartKind::Thatch => 5,
            PartKind::Floor => 6,
            PartKind::Hearth => 7,
        }
    }
}

/// One piece of a building, in centimetres from the map's north-west corner (z up from the
/// ground).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Part {
    /// Stable within the expansion: the stage times 1000 plus a running number.
    pub id: u32,
    /// What it is.
    pub kind: PartKind,
    /// The stage that puts it in place.
    pub stage: Stage,
    /// Centre: x east, y south, z up.
    pub at: [i32; 3],
    /// Extent along its own axes: length, width, height.
    pub size: [i32; 3],
    /// Rotation about the vertical, 1/65,536 of a turn from east toward south.
    pub angle: u16,
    /// Rise from the horizontal along its length, hundredths of a degree (rafters).
    pub tilt: i32,
    /// Material slot, or `None` for earth and work alone.
    pub material: Option<u8>,
}

/// The labour and materials a stage needs.
#[derive(Clone, Debug, PartialEq)]
pub struct StageNeeds {
    /// The stage.
    pub stage: Stage,
    /// Person-hours of a capable adult.
    pub labour_h: f64,
    /// Kilograms per material slot.
    pub materials_kg: Vec<f64>,
}

/// What a spec expands into.
#[derive(Clone, Debug, PartialEq)]
pub struct Expansion {
    /// The grammar version that produced it.
    pub version: u16,
    /// Every part, in stage order.
    pub parts: Vec<Part>,
    /// The outer face of the walls, centimetres, going round from the door.
    pub outline: Vec<(i32, i32)>,
    /// Radius of the roof's edge, centimetres (what is seen from above).
    pub roof_radius_cm: i32,
    /// Height of the roof's apex, centimetres.
    pub apex_cm: i32,
    /// The doorway's middle on the wall line, centimetres, and the direction it faces.
    pub door: (i32, i32, u16),
    /// Width of the doorway, centimetres.
    pub door_width_cm: i32,
    /// One entry per stage, in order.
    pub stages: Vec<StageNeeds>,
    /// Floor area inside the wall line, square metres.
    pub floor_area_m2: f64,
    /// People it sleeps.
    pub sleeping_places: u32,
}

impl Expansion {
    /// Labour of every stage together, person-hours.
    pub fn total_labour_h(&self) -> f64 {
        self.stages.iter().map(|s| s.labour_h).sum()
    }

    /// Kilograms of each material slot over every stage.
    pub fn total_materials_kg(&self) -> Vec<f64> {
        let mut out = vec![0.0; hut_materials::COUNT];
        for s in &self.stages {
            for (o, kg) in out.iter_mut().zip(&s.materials_kg) {
                *o += kg;
            }
        }
        out
    }
}

/// The hut program's rules: every dimension, labour and material figure, from content.
#[derive(Clone, Debug, PartialEq)]
pub struct HutRules {
    /// Allowed radius of the wall line, centimetres (least, most).
    pub radius_cm: (i32, i32),
    /// Allowed wall height to the eaves, centimetres.
    pub eave_cm: (i32, i32),
    /// Allowed roof pitch, hundredths of a degree.
    pub pitch_centideg: (i32, i32),
    /// Spacing of the wall posts along the wall line, centimetres.
    pub post_spacing_cm: i32,
    /// Diameter of a post, centimetres.
    pub post_diameter_cm: i32,
    /// Depth of a posthole, centimetres.
    pub posthole_depth_cm: i32,
    /// Thickness of a daubed wall, centimetres.
    pub wall_thickness_cm: i32,
    /// How far the roof reaches beyond the wall line, centimetres.
    pub roof_overhang_cm: i32,
    /// Thickness of the thatch, centimetres.
    pub thatch_thickness_cm: i32,
    /// Diameter of the hearth, centimetres.
    pub hearth_cm: i32,
    /// Floor area a household needs whatever its size (hearth, stores), square metres.
    pub floor_base_m2: f64,
    /// Floor area each resident adds, square metres.
    pub floor_m2_per_sleeper: f64,
    /// Person-hours to clear and level the floor and mark out the walls, per square metre.
    pub groundwork_h_per_m2: f64,
    /// Person-hours to dig a posthole.
    pub posthole_h: f64,
    /// Person-hours to fell, trim and set a post.
    pub post_h: f64,
    /// Person-hours to fell, trim and raise a rafter.
    pub rafter_h: f64,
    /// Person-hours to cut and weave wattle, per square metre of wall.
    pub wattle_h_per_m2: f64,
    /// Person-hours to dig, mix and apply daub, per square metre of wall.
    pub daub_h_per_m2: f64,
    /// Person-hours to thatch, per square metre of roof.
    pub thatch_h_per_m2: f64,
    /// Person-hours to beat the floor and lay the hearth, per square metre of floor.
    pub finish_h_per_m2: f64,
    /// Timber in a post, kilograms.
    pub post_kg: f64,
    /// Timber in a rafter, kilograms.
    pub rafter_kg: f64,
    /// Wattle rods per square metre of wall, kilograms.
    pub wattle_kg_per_m2: f64,
    /// Thatch per square metre of roof, kilograms.
    pub thatch_kg_per_m2: f64,
}

impl HutRules {
    /// The wall-line radius of a hut that sleeps `residents`, centimetres: the floor they need,
    /// rounded to whole decimetres and kept within the allowed range.
    pub fn radius_for(&self, residents: usize) -> i32 {
        let area = self.floor_base_m2 + self.floor_m2_per_sleeper * residents as f64;
        let r = (area.max(0.0) / std::f64::consts::PI).sqrt() * 100.0;
        let r = ((r / 10.0).ceil() * 10.0) as i32;
        r.clamp(self.radius_cm.0, self.radius_cm.1)
    }
}

/// Why a spec cannot be expanded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrammarError {
    /// A grammar version this build does not have.
    UnknownVersion(u16),
    /// A value outside what the program allows.
    OutOfRange(String),
}

impl std::fmt::Display for GrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarError::UnknownVersion(v) => write!(f, "hut grammar version {v} is unknown"),
            GrammarError::OutOfRange(what) => write!(f, "{what}"),
        }
    }
}

impl std::error::Error for GrammarError {}

fn within(name: &str, v: i32, (lo, hi): (i32, i32)) -> Result<(), GrammarError> {
    if (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(GrammarError::OutOfRange(format!(
            "{name} {v} is outside {lo}..={hi}"
        )))
    }
}

/// An angle in turn units, wrapped into `u16`.
fn turn_units(radians: f64) -> u16 {
    let t = (radians / TAU * TURN).round().rem_euclid(TURN);
    t as u16
}

fn cm(v: f64) -> i32 {
    v.round() as i32
}

/// Expands a hut: posts evenly round the wall line with the doorway in the gap the door faces,
/// a rafter from each post's head to the apex, wattle and daub between the posts, a conical
/// thatch, a beaten floor and a hearth in the middle.
pub fn expand_hut(spec: &BuildingSpec, rules: &HutRules) -> Result<Expansion, GrammarError> {
    if spec.version != HUT_VERSION {
        return Err(GrammarError::UnknownVersion(spec.version));
    }
    let Footprint::Round {
        x: cx,
        y: cy,
        radius,
    } = spec.footprint;
    within("radius", radius, rules.radius_cm)?;
    let eave = spec.params[hut_params::EAVE_CM];
    within("eave height", eave, rules.eave_cm)?;
    let pitch_cd = spec.params[hut_params::PITCH_CENTIDEG];
    within("roof pitch", pitch_cd, rules.pitch_centideg)?;
    if spec.storeys != 1 {
        return Err(GrammarError::OutOfRange(format!(
            "a hut has one storey, not {}",
            spec.storeys
        )));
    }
    if spec.materials.len() != hut_materials::COUNT {
        return Err(GrammarError::OutOfRange(format!(
            "a hut names {} materials, not {}",
            hut_materials::COUNT,
            spec.materials.len()
        )));
    }
    if rules.post_spacing_cm <= 0 || rules.post_diameter_cm <= 0 {
        return Err(GrammarError::OutOfRange(
            "post spacing and diameter must be positive".to_owned(),
        ));
    }
    let door_dir =
        f64::from(spec.params[hut_params::DOOR_DIR].rem_euclid(TURN as i32)) / TURN * TAU;
    let r = f64::from(radius);
    let pitch = f64::from(pitch_cd) / 100.0 * TAU / 360.0;
    let (cxf, cyf) = (f64::from(cx), f64::from(cy));
    let at = |angle: f64, dist: f64| (cxf + dist * angle.cos(), cyf + dist * angle.sin());

    // Posts evenly round the wall line, the doorway centred in the gap the door faces.
    let perimeter = TAU * r;
    let n = ((perimeter / f64::from(rules.post_spacing_cm)).round() as usize).max(6);
    let step = TAU / n as f64;
    let post_angle = |k: usize| door_dir + step * (k as f64 + 0.5);
    let chord = 2.0 * r * (step / 2.0).sin();
    let d = rules.post_diameter_cm;
    let depth = rules.posthole_depth_cm;
    let overhang = f64::from(rules.roof_overhang_cm);
    let apex = f64::from(eave) + r * pitch.tan();
    let roof_r = r + overhang;

    let mut parts: Vec<Part> = Vec::new();
    let mut push = |kind: PartKind,
                    stage: Stage,
                    at: [i32; 3],
                    size: [i32; 3],
                    angle: u16,
                    tilt: i32,
                    material: Option<u8>| {
        let id =
            stage.index() as u32 * 1000 + parts.iter().filter(|p| p.stage == stage).count() as u32;
        parts.push(Part {
            id,
            kind,
            stage,
            at,
            size,
            angle,
            tilt,
            material,
        });
    };
    for k in 0..n {
        let (x, y) = at(post_angle(k), r);
        push(
            PartKind::Posthole,
            Stage::Foundation,
            [cm(x), cm(y), -depth / 2],
            [d + 10, d + 10, depth],
            0,
            0,
            None,
        );
    }
    for k in 0..n {
        let (x, y) = at(post_angle(k), r);
        push(
            PartKind::Post,
            Stage::Frame,
            [cm(x), cm(y), (eave - depth) / 2],
            [d, d, eave + depth],
            0,
            0,
            Some(hut_materials::TIMBER),
        );
    }
    // Rafters run from the roof's edge over each post's head to the apex.
    let rafter_len = roof_r / pitch.cos();
    let edge_z = f64::from(eave) - overhang * pitch.tan();
    for k in 0..n {
        let a = post_angle(k);
        let (x, y) = at(a, roof_r / 2.0);
        push(
            PartKind::Rafter,
            Stage::Frame,
            [cm(x), cm(y), cm((edge_z + apex) / 2.0)],
            [cm(rafter_len), d * 2 / 3, d * 2 / 3],
            turn_units(a + TAU / 2.0),
            pitch_cd,
            Some(hut_materials::TIMBER),
        );
    }
    // Wattle and daub fill every gap between posts but the doorway's.
    let panel = (chord - f64::from(d)).max(0.0);
    for kind in [PartKind::Wattle, PartKind::Daub] {
        for k in 0..n - 1 {
            let mid = post_angle(k) + step / 2.0;
            let (x, y) = at(mid, r * (step / 2.0).cos());
            let (thickness, material) = match kind {
                PartKind::Wattle => (rules.wall_thickness_cm / 3, Some(hut_materials::WATTLE)),
                _ => (rules.wall_thickness_cm, None),
            };
            push(
                kind,
                Stage::Walls,
                [cm(x), cm(y), eave / 2],
                [cm(panel), thickness, eave],
                turn_units(mid + TAU / 4.0),
                0,
                material,
            );
        }
    }
    push(
        PartKind::Thatch,
        Stage::Roof,
        [cx, cy, cm((edge_z + apex) / 2.0)],
        [cm(2.0 * roof_r), cm(2.0 * roof_r), cm(apex - edge_z)],
        0,
        pitch_cd,
        Some(hut_materials::THATCH),
    );
    push(
        PartKind::Floor,
        Stage::Finish,
        [cx, cy, 0],
        [cm(2.0 * r), cm(2.0 * r), 0],
        0,
        0,
        None,
    );
    push(
        PartKind::Hearth,
        Stage::Finish,
        [cx, cy, 0],
        [rules.hearth_cm, rules.hearth_cm, 10],
        0,
        0,
        None,
    );

    // Outline: the walls' outer face, 32 points round from the door.
    let outer = r + f64::from(rules.wall_thickness_cm) / 2.0;
    let outline = (0..32)
        .map(|i| {
            let (x, y) = at(door_dir + TAU * f64::from(i) / 32.0, outer);
            (cm(x), cm(y))
        })
        .collect();
    let (dx, dy) = at(door_dir, r * (step / 2.0).cos());

    // What each stage needs.
    let m2 = 1.0e-4;
    let wall_area = (perimeter - chord) * f64::from(eave) * m2;
    let roof_area = std::f64::consts::PI * roof_r * (roof_r / pitch.cos()) * m2;
    let floor_area = std::f64::consts::PI * r * r * m2;
    let posts = n as f64;
    let stages = vec![
        StageNeeds {
            stage: Stage::Foundation,
            labour_h: rules.posthole_h * posts + rules.groundwork_h_per_m2 * floor_area,
            materials_kg: vec![0.0; hut_materials::COUNT],
        },
        StageNeeds {
            stage: Stage::Frame,
            labour_h: (rules.post_h + rules.rafter_h) * posts,
            materials_kg: vec![(rules.post_kg + rules.rafter_kg) * posts, 0.0, 0.0],
        },
        StageNeeds {
            stage: Stage::Walls,
            labour_h: (rules.wattle_h_per_m2 + rules.daub_h_per_m2) * wall_area,
            materials_kg: vec![0.0, rules.wattle_kg_per_m2 * wall_area, 0.0],
        },
        StageNeeds {
            stage: Stage::Roof,
            labour_h: rules.thatch_h_per_m2 * roof_area,
            materials_kg: vec![0.0, 0.0, rules.thatch_kg_per_m2 * roof_area],
        },
        StageNeeds {
            stage: Stage::Finish,
            labour_h: rules.finish_h_per_m2 * floor_area,
            materials_kg: vec![0.0; hut_materials::COUNT],
        },
    ];
    let sleeping_places = if rules.floor_m2_per_sleeper > 0.0 {
        ((floor_area - rules.floor_base_m2).max(0.0) / rules.floor_m2_per_sleeper).floor() as u32
    } else {
        0
    };
    Ok(Expansion {
        version: HUT_VERSION,
        parts,
        outline,
        roof_radius_cm: cm(roof_r),
        apex_cm: cm(apex),
        door: (cm(dx), cm(dy), turn_units(door_dir)),
        door_width_cm: cm(panel),
        stages,
        floor_area_m2: floor_area,
        sleeping_places,
    })
}

/// A stable 64-bit hash of an expansion, for golden tests: FNV-1a over its integers, with labour
/// in hundredths of an hour, materials in grams and area in hundredths of a square metre.
pub fn expansion_hash(e: &Expansion) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    eat(&e.version.to_le_bytes());
    for p in &e.parts {
        eat(&p.id.to_le_bytes());
        eat(&[p.kind.code(), p.stage.index() as u8]);
        for v in p.at.iter().chain(&p.size) {
            eat(&v.to_le_bytes());
        }
        eat(&p.angle.to_le_bytes());
        eat(&p.tilt.to_le_bytes());
        eat(&[p.material.unwrap_or(u8::MAX)]);
    }
    for (x, y) in &e.outline {
        eat(&x.to_le_bytes());
        eat(&y.to_le_bytes());
    }
    for v in [
        e.roof_radius_cm,
        e.apex_cm,
        e.door.0,
        e.door.1,
        e.door_width_cm,
    ] {
        eat(&v.to_le_bytes());
    }
    eat(&e.door.2.to_le_bytes());
    for s in &e.stages {
        eat(&[s.stage.index() as u8]);
        eat(&((s.labour_h * 100.0).round() as i64).to_le_bytes());
        for kg in &s.materials_kg {
            eat(&((kg * 1000.0).round() as i64).to_le_bytes());
        }
    }
    eat(&((e.floor_area_m2 * 100.0).round() as i64).to_le_bytes());
    eat(&e.sleeping_places.to_le_bytes());
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> HutRules {
        HutRules {
            radius_cm: (200, 450),
            eave_cm: (100, 220),
            pitch_centideg: (3500, 6000),
            post_spacing_cm: 80,
            post_diameter_cm: 15,
            posthole_depth_cm: 60,
            wall_thickness_cm: 15,
            roof_overhang_cm: 50,
            thatch_thickness_cm: 30,
            hearth_cm: 80,
            floor_base_m2: 4.0,
            floor_m2_per_sleeper: 3.0,
            groundwork_h_per_m2: 1.0,
            posthole_h: 1.0,
            post_h: 4.0,
            rafter_h: 3.0,
            wattle_h_per_m2: 2.0,
            daub_h_per_m2: 2.0,
            thatch_h_per_m2: 1.5,
            finish_h_per_m2: 0.5,
            post_kg: 30.0,
            rafter_kg: 20.0,
            wattle_kg_per_m2: 10.0,
            thatch_kg_per_m2: 30.0,
        }
    }

    fn spec(radius: i32) -> BuildingSpec {
        let mut params = [0; 8];
        params[hut_params::EAVE_CM] = 150;
        params[hut_params::PITCH_CENTIDEG] = 4500;
        params[hut_params::DOOR_DIR] = 8192; // south-east
        BuildingSpec {
            program: "core:building/hut".into(),
            version: HUT_VERSION,
            footprint: Footprint::Round {
                x: 100_000,
                y: 200_000,
                radius,
            },
            storeys: 1,
            params,
            materials: vec![
                "core:good/timber".into(),
                "core:good/timber".into(),
                "core:good/thatch".into(),
            ],
            style_seed: 0,
        }
    }

    #[test]
    fn a_hut_has_its_parts_in_stage_order() {
        let e = expand_hut(&spec(300), &rules()).expect("expands");
        let count = |k: PartKind| e.parts.iter().filter(|p| p.kind == k).count();
        // 2π·300 / 80 ≈ 23.6: 24 posts, with the doorway in one gap.
        assert_eq!(count(PartKind::Posthole), 24);
        assert_eq!(count(PartKind::Post), 24);
        assert_eq!(count(PartKind::Rafter), 24);
        assert_eq!(count(PartKind::Wattle), 23);
        assert_eq!(count(PartKind::Daub), 23);
        assert_eq!(count(PartKind::Thatch), 1);
        assert!(e.parts.windows(2).all(|w| w[0].stage <= w[1].stage));
        let ids: std::collections::HashSet<u32> = e.parts.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), e.parts.len(), "part ids are unique");
        // Posts stand on the wall line.
        for p in e.parts.iter().filter(|p| p.kind == PartKind::Post) {
            let r = f64::from(p.at[0] - 100_000).hypot(f64::from(p.at[1] - 200_000));
            assert!((r - 300.0).abs() <= 1.0, "{r}");
        }
        // The door faces south-east, between two posts.
        let (dx, dy, dir) = e.door;
        assert_eq!(dir, 8192);
        assert!(dx > 100_000 && dy > 200_000);
        assert!(e.door_width_cm > 50 && e.door_width_cm < 80);
        // 28.3 m² of floor, 4 m² of it shared, at 3 m² a sleeper; a roof 45° steep rises 3 m
        // above the eaves.
        assert!((e.floor_area_m2 - std::f64::consts::PI * 9.0).abs() < 1e-9);
        assert_eq!(e.sleeping_places, 8);
        assert_eq!(e.apex_cm, 450);
        assert_eq!(e.outline.len(), 32);
    }

    #[test]
    fn a_household_s_hut_is_sized_by_who_sleeps_in_it() {
        let r = rules();
        // 4 + 3·5 = 19 m²: radius 2.46 m, rounded up to 2.5 m.
        assert_eq!(r.radius_for(5), 250);
        assert!(r.radius_for(8) > r.radius_for(5));
        assert_eq!(r.radius_for(0), 200, "never below the least hut");
        assert_eq!(r.radius_for(60), 450, "nor above the largest");
        let e = expand_hut(&spec(r.radius_for(5)), &r).expect("expands");
        assert!(e.sleeping_places >= 5);
    }

    #[test]
    fn needs_grow_with_the_hut() {
        let r = rules();
        let small = expand_hut(&spec(250), &r).expect("expands");
        let large = expand_hut(&spec(400), &r).expect("expands");
        assert!(large.total_labour_h() > small.total_labour_h());
        let (s, l) = (small.total_materials_kg(), large.total_materials_kg());
        assert!(s.iter().zip(&l).all(|(a, b)| b >= a));
        assert!(l[usize::from(hut_materials::THATCH)] > s[usize::from(hut_materials::THATCH)]);
        // Every stage needs work; only the frame, walls and roof need material.
        assert!(small.stages.iter().all(|s| s.labour_h > 0.0));
        // 2π·250 / 80 ≈ 19.6: 20 posts and 20 rafters at 30 + 20 kg.
        assert_eq!(
            small.stages[Stage::Frame.index()].materials_kg[0],
            20.0 * 50.0
        );
        assert_eq!(
            small.stages[Stage::Foundation.index()].materials_kg,
            vec![0.0; 3]
        );
    }

    #[test]
    fn specs_out_of_range_or_of_an_unknown_version_are_refused() {
        let r = rules();
        let mut s = spec(300);
        s.version = 99;
        assert_eq!(expand_hut(&s, &r), Err(GrammarError::UnknownVersion(99)));
        assert!(matches!(
            expand_hut(&spec(1000), &r),
            Err(GrammarError::OutOfRange(_))
        ));
        let mut tall = spec(300);
        tall.params[hut_params::EAVE_CM] = 500;
        assert!(expand_hut(&tall, &r).is_err());
        let mut two = spec(300);
        two.storeys = 2;
        assert!(expand_hut(&two, &r).is_err());
    }

    #[test]
    fn expansion_is_pure_and_pinned() {
        let r = rules();
        let a = expand_hut(&spec(300), &r).expect("expands");
        let b = expand_hut(&spec(300), &r).expect("expands");
        assert_eq!(a, b);
        assert_eq!(expansion_hash(&a), expansion_hash(&b));
        // The golden hash of version 1: a change here is a change to what huts look like, which
        // must bump HUT_VERSION (ADR-0004 §2).
        assert_eq!(expansion_hash(&a), GOLDEN_300, "{:#x}", expansion_hash(&a));
        assert_ne!(
            expansion_hash(&expand_hut(&spec(310), &r).expect("expands")),
            GOLDEN_300
        );
    }

    const GOLDEN_300: u64 = 0xa9fc_49c5_c66b_50d7;
}
