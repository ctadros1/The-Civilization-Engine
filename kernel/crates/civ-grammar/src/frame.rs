//! The frame grammar (ADR-0009 §1, version 1): rectilinear post-framed buildings of equal bays at a
//! constant width under a gabled roof, of one or two storeys, with lofts over chosen bays and
//! optionally a floor raised on posts (research 10-06 §1.2, §2.1, §3.1; 11-01 §2.4, §3.4).
//!
//! A frame stands on a [`Footprint::Rect`]: its length runs along the footprint's angle (`u`) and
//! its width across it (`v`, a quarter turn on, to the right of the length's direction). At each
//! frame line (the ends of every bay) a post stands on each long wall and runs through every
//! storey to the plates along the wall heads; a tie beam joins the two posts' heads, and pairs of
//! rafters rise from the eaves to a ridge. A loft is a floor of joists across the width resting on
//! the plates over the bays the spec names, and holds goods only (11-01 §2.4: a storage platform
//! is not a storey). An upper storey's floor and a raised floor are joists across the width on
//! rails along the walls, with cross beams at the frame lines; a raised floor also rests on a
//! middle rail carried by floor posts. The walls are wattle and daub between the posts, storey by
//! storey, and the gables above the end tie beams are woven and daubed too.
//!
//! Every dimension, labour and material figure comes from the program's [`FrameRules`]; timber is
//! costed from the members' sizes (research 11-04 §2.2), so a deeper joist costs real wood, and
//! the roof's area is its projection over the cosine of its pitch (11-04 §2.3). Part and group ids
//! are semantic (research 11-03 §4.8): level, bay or frame line, kind and index, a byte each, so a
//! building with one more bay keeps every id it had.

use std::f64::consts::TAU;

use crate::{
    BuildingSpec, Expansion, Footprint, Grammar, GrammarError, Group, GroupKind, Level, Part,
    PartKind, RECT_QUANTUM_CM, Space, SpaceUse, Stage, StageNeeds, TURN, cm, group_id, within,
};

/// Version of the frame grammar.
pub const FRAME_VERSION: u16 = 1;
/// Most bays a frame may have.
pub const MAX_BAYS: i32 = 8;
/// Most storeys a frame may have (research 11-05 §2.2: one to three in timber; two here).
pub const MAX_STOREYS: u8 = 2;
/// Most component groups an expansion may have (research 11-06 §5.1: 8 to 32).
pub const MAX_GROUPS: usize = 32;
/// Most rafters a slope has in a bay, or joists a floor has in a bay: indexes fit a byte.
const MAX_MEMBERS_PER_BAY: usize = 64;
/// Rise of a ladder from the horizontal, hundredths of a degree.
pub const LADDER_TILT_CENTIDEG: i32 = 7000;
/// Width of a ladder, centimetres.
pub const LADDER_WIDTH_CM: i32 = 50;
/// Thickness of a ladder, centimetres.
const LADDER_DEPTH_CM: f64 = 8.0;
/// Height of a hearth's clay, centimetres.
const HEARTH_HEIGHT_CM: f64 = 10.0;
/// How much wider than its post a posthole is dug, centimetres (as the hut's).
const POSTHOLE_SLACK_CM: f64 = 10.0;

/// Indexes of a frame's parameters in [`BuildingSpec::params`] (ADR-0009 §2).
pub mod frame_params {
    /// Wall height of each storey, from its floor to the plates or the floor above, centimetres.
    pub const EAVE_CM: usize = 0;
    /// Roof pitch, hundredths of a degree.
    pub const PITCH_CENTIDEG: usize = 1;
    /// Bays, 1 to [`super::MAX_BAYS`]: each is the length over the bays.
    pub const BAYS: usize = 2;
    /// The door: its side times 16 plus its bay (an end wall's door has bay 0). It opens on the
    /// first storey.
    pub const DOOR: usize = 3;
    /// The bays floored as a loft, a bit each (bit `b` for bay `b`).
    pub const LOFT_BAYS: usize = 4;
    /// Diameter of a joist, centimetres: the over-build decision; 0 for a building without joists.
    pub const JOIST_CM: usize = 5;
    /// How far the roof reaches beyond the wall lines, at the eaves and the gables, centimetres.
    pub const OVERHANG_CM: usize = 6;
    /// Height of a raised floor above the ground, centimetres; 0 for a floor on the ground.
    pub const FLOOR_RAISE_CM: usize = 7;
    /// Diameter of a post, centimetres.
    pub const POST_CM: usize = 8;
    /// The walls' kind: 0, wattle and daub (the only kind in version 1).
    pub const WALL_KIND: usize = 9;
    /// Thickness of a daubed wall, centimetres.
    pub const WALL_CM: usize = 10;
    /// The footing: 0, posts set in the ground (the only kind in version 1).
    pub const FOOTING: usize = 11;
    /// The first of four style traits (ADR-0009 §7). Version 1 has none, so they are 0.
    pub const STYLE: usize = 12;
    /// Number of style traits.
    pub const STYLE_TRAITS: usize = 4;

    /// The long wall to the right of the length's direction (`+v`).
    pub const SIDE_RIGHT: i32 = 0;
    /// The long wall to the left of it (`-v`).
    pub const SIDE_LEFT: i32 = 1;
    /// The end wall the length runs from (`-u`).
    pub const SIDE_BACK: i32 = 2;
    /// The end wall the length runs to (`+u`).
    pub const SIDE_FRONT: i32 = 3;

    /// A door's parameter: on `side`, in `bay` (0 on an end wall).
    pub fn door(side: i32, bay: i32) -> i32 {
        side * 16 + bay
    }
}

/// A frame's material slots, indexes into [`BuildingSpec::materials`].
pub mod frame_materials {
    /// Posts, plates, beams, rafters and joists.
    pub const TIMBER: u8 = 0;
    /// Rods woven between the posts.
    pub const WATTLE: u8 = 1;
    /// The roof covering.
    pub const COVERING: u8 = 2;
    /// Boards for floors and ladders.
    pub const BOARDS: u8 = 3;
    /// Number of slots.
    pub const COUNT: usize = 4;
}

use frame_materials::{BOARDS, COVERING, TIMBER, WATTLE};
use frame_params::{SIDE_BACK, SIDE_FRONT, SIDE_LEFT, SIDE_RIGHT};

/// A frame program's rules: what a spec may choose, and every dimension, labour and material
/// figure (from content).
#[derive(Clone, Debug, PartialEq)]
pub struct FrameRules {
    /// Allowed bays (least, most), within 1 to [`MAX_BAYS`].
    pub bays: (i32, i32),
    /// Allowed length of a bay, centimetres.
    pub bay_cm: (i32, i32),
    /// Allowed width between the long wall lines, centimetres.
    pub width_cm: (i32, i32),
    /// Allowed storeys, within 1 to [`MAX_STOREYS`].
    pub storeys: (u8, u8),
    /// Allowed storey height, centimetres.
    pub eave_cm: (i32, i32),
    /// Allowed roof pitch, hundredths of a degree.
    pub pitch_centideg: (i32, i32),
    /// Allowed joist diameter, centimetres.
    pub joist_cm: (i32, i32),
    /// Allowed post diameter, centimetres.
    pub post_cm: (i32, i32),
    /// Allowed overhang of the roof, centimetres.
    pub overhang_cm: (i32, i32),
    /// Allowed height of a raised floor, centimetres; `(0, 0)` when the program has none.
    pub floor_raise_cm: (i32, i32),
    /// Allowed thickness of a daubed wall, centimetres.
    pub wall_cm: (i32, i32),
    /// Whether bays may be floored as lofts.
    pub lofts: bool,
    /// What the first storey is for.
    pub ground_use: SpaceUse,
    /// What an upper storey is for.
    pub upper_use: SpaceUse,
    /// Depth of a posthole, centimetres.
    pub posthole_depth_cm: i32,
    /// Diameter of a plate, rail, tie or cross beam and the ridge, centimetres.
    pub beam_cm: i32,
    /// Diameter of a rafter, centimetres.
    pub rafter_cm: i32,
    /// Spacing of rafters along the length, centimetres.
    pub rafter_spacing_cm: i32,
    /// Spacing of joists along the length, centimetres.
    pub joist_spacing_cm: i32,
    /// Thickness of a floor's boards, centimetres.
    pub decking_cm: i32,
    /// Thickness of the roof covering, centimetres.
    pub thatch_thickness_cm: i32,
    /// Diameter of the hearth, centimetres.
    pub hearth_cm: i32,
    /// Width of the doorway, centimetres.
    pub door_cm: i32,
    /// Living floor a household needs whatever its size (hearth, stores), square metres.
    pub floor_base_m2: f64,
    /// Living floor each resident adds, square metres.
    pub floor_m2_per_sleeper: f64,
    /// Floor a place to work takes, square metres.
    pub work_m2_per_worker: f64,
    /// Goods a store's floor holds, kilograms a square metre.
    pub store_kg_per_m2: f64,
    /// Goods a loft holds, kilograms a square metre.
    pub loft_kg_per_m2: f64,
    /// Goods living or working floor holds besides its use, kilograms a square metre.
    pub living_kg_per_m2: f64,
    /// Density of the timber, kilograms a cubic metre.
    pub timber_kg_per_m3: f64,
    /// Person-hours to clear and level the ground and mark out the building, per square metre.
    pub groundwork_h_per_m2: f64,
    /// Person-hours to dig a posthole.
    pub posthole_h: f64,
    /// Person-hours to fell, trim, joint and set a post.
    pub post_h: f64,
    /// Person-hours to fell, trim and joint a plate, rail, beam or ridge, per metre.
    pub beam_h_per_m: f64,
    /// Person-hours to fell, trim and raise a rafter.
    pub rafter_h: f64,
    /// Person-hours to fell, trim and lay a joist, per metre.
    pub joist_h_per_m: f64,
    /// Person-hours to cut and weave wattle, per square metre of wall.
    pub wattle_h_per_m2: f64,
    /// Person-hours to dig, mix and apply daub, per cubic metre.
    pub daub_h_per_m3: f64,
    /// Person-hours to lay the covering, per square metre of roof.
    pub thatch_h_per_m2: f64,
    /// Person-hours to beat a floor on the ground and lay the hearth, per square metre.
    pub finish_h_per_m2: f64,
    /// Person-hours to split and lay boards, per square metre of floor.
    pub decking_h_per_m2: f64,
    /// Person-hours to make a ladder.
    pub ladder_h: f64,
    /// Wattle rods per square metre of wall, kilograms.
    pub wattle_kg_per_m2: f64,
    /// Covering per square metre of roof, kilograms.
    pub thatch_kg_per_m2: f64,
    /// Boards in a ladder, kilograms.
    pub ladder_kg: f64,
}

fn refuse<T>(what: impl Into<String>) -> Result<T, GrammarError> {
    Err(GrammarError::OutOfRange(what.into()))
}

/// A part's semantic id: level, bay or frame line (0 for none, else it plus one), kind and index,
/// a byte each.
fn part_id(level: Level, slot: Option<usize>, kind: PartKind, index: usize) -> u32 {
    (u32::from(level.code()) << 24)
        | ((slot.map_or(0, |s| s + 1) as u32 & 0xff) << 16)
        | (u32::from(kind.code()) << 8)
        | (index as u32 & 0xff)
}

/// `a` turned on by `quarters` quarter turns.
fn turned(a: u16, quarters: u16) -> u16 {
    a.wrapping_add(quarters.wrapping_mul(16_384))
}

/// Area of a circle of diameter `d` centimetres, square metres.
fn round_section_m2(d: f64) -> f64 {
    std::f64::consts::PI * d * d / 4.0 * 1.0e-4
}

/// Expands a frame building (see the module's description).
pub fn expand_frame(spec: &BuildingSpec, rules: &FrameRules) -> Result<Expansion, GrammarError> {
    use frame_params as fp;
    if spec.version != FRAME_VERSION {
        return Err(GrammarError::UnknownVersion(spec.version));
    }
    let Footprint::Rect {
        x: cx,
        y: cy,
        length,
        width,
        angle,
    } = spec.footprint
    else {
        return refuse("a frame building stands on a rectangular footprint");
    };
    let p = &spec.params;
    if spec.materials.len() != frame_materials::COUNT {
        return refuse(format!(
            "a frame names {} materials, not {}",
            frame_materials::COUNT,
            spec.materials.len()
        ));
    }
    for (name, v) in [
        ("beam diameter", rules.beam_cm),
        ("rafter diameter", rules.rafter_cm),
        ("rafter spacing", rules.rafter_spacing_cm),
        ("joist spacing", rules.joist_spacing_cm),
        ("decking", rules.decking_cm),
        ("door width", rules.door_cm),
    ] {
        if v <= 0 {
            return refuse(format!("the rules' {name} must be positive, not {v}"));
        }
    }

    // What the spec chose, against what the program allows.
    let bays = p[fp::BAYS];
    within("bays", bays, rules.bays)?;
    within("bays", bays, (1, MAX_BAYS))?;
    for (name, v) in [("length", length), ("width", width)] {
        if v <= 0 || v % RECT_QUANTUM_CM != 0 {
            return refuse(format!(
                "{name} {v} is not a positive multiple of {RECT_QUANTUM_CM} cm"
            ));
        }
    }
    if length % bays != 0 {
        return refuse(format!("length {length} does not divide into {bays} bays"));
    }
    let bay = length / bays;
    within("bay length", bay, rules.bay_cm)?;
    within("width", width, rules.width_cm)?;
    let storeys = i32::from(spec.storeys);
    within(
        "storeys",
        storeys,
        (i32::from(rules.storeys.0), i32::from(rules.storeys.1)),
    )?;
    within("storeys", storeys, (1, i32::from(MAX_STOREYS)))?;
    let eave = p[fp::EAVE_CM];
    within("storey height", eave, rules.eave_cm)?;
    let pitch_cd = p[fp::PITCH_CENTIDEG];
    within("roof pitch", pitch_cd, rules.pitch_centideg)?;
    within("roof pitch", pitch_cd, (1, 8_999))?;
    let post = p[fp::POST_CM];
    within("post diameter", post, rules.post_cm)?;
    let overhang = p[fp::OVERHANG_CM];
    within("overhang", overhang, rules.overhang_cm)?;
    let wall = p[fp::WALL_CM];
    within("wall thickness", wall, rules.wall_cm)?;
    if p[fp::WALL_KIND] != 0 {
        return refuse(format!(
            "wall kind {} is unknown to frame version 1",
            p[fp::WALL_KIND]
        ));
    }
    if p[fp::FOOTING] != 0 {
        return refuse(format!(
            "footing {} is unknown to frame version 1",
            p[fp::FOOTING]
        ));
    }
    if p[fp::STYLE..fp::STYLE + fp::STYLE_TRAITS]
        .iter()
        .any(|&t| t != 0)
    {
        return refuse("frame version 1 has no style traits");
    }
    let raise = p[fp::FLOOR_RAISE_CM];
    if raise != 0 {
        within("floor raise", raise, rules.floor_raise_cm)?;
    }
    let loft_mask = p[fp::LOFT_BAYS];
    let all_bays = (1_i32 << bays) - 1;
    if loft_mask < 0 || loft_mask & !all_bays != 0 {
        return refuse(format!(
            "loft bays {loft_mask:#b} are not among the {bays} bays"
        ));
    }
    if loft_mask != 0 && !rules.lofts {
        return refuse("this program has no lofts");
    }
    let n = bays as usize;
    let lofts: Vec<usize> = (0..n).filter(|b| loft_mask & (1 << b) != 0).collect();
    let raised = raise > 0;
    let joisted = !lofts.is_empty() || storeys > 1 || raised;
    let joist = p[fp::JOIST_CM];
    if joisted {
        within("joist diameter", joist, rules.joist_cm)?;
    } else if joist != 0 {
        return refuse("a building without joists has a joist diameter of 0");
    }
    let door = p[fp::DOOR];
    let (door_side, door_bay) = (door.div_euclid(16), door.rem_euclid(16));
    let door_fits_side = match door_side {
        SIDE_RIGHT | SIDE_LEFT => door_bay < bays,
        SIDE_BACK | SIDE_FRONT => door_bay == 0,
        _ => false,
    };
    if door < 0 || !door_fits_side {
        return refuse(format!("door {door} is on no wall of {bays} bays"));
    }
    let door_panel = if door_side < SIDE_BACK {
        bay - post
    } else {
        width - post
    };
    if rules.door_cm >= door_panel {
        return refuse(format!(
            "a door {} cm wide does not fit a panel {door_panel} cm wide",
            rules.door_cm
        ));
    }

    // Dimensions, centimetres; u along the length, v across it, z up from the ground.
    let (l, w, b) = (f64::from(length), f64::from(width), f64::from(bay));
    let (h, r, o) = (f64::from(eave), f64::from(raise), f64::from(overhang));
    let (pd, jst, wt) = (f64::from(post), f64::from(joist), f64::from(wall));
    let beam = f64::from(rules.beam_cm);
    let raf = f64::from(rules.rafter_cm);
    let dk = f64::from(rules.decking_cm);
    let depth = f64::from(rules.posthole_depth_cm);
    let door_w = f64::from(rules.door_cm);
    let pitch = f64::from(pitch_cd) / 100.0 * TAU / 360.0;
    let z_g = r;
    let z_p = r + h * f64::from(storeys);
    let z_a = z_p + w / 2.0 * pitch.tan();
    let z_e = z_p - o * pitch.tan();
    if z_e <= 0.0 {
        return refuse("the eaves would reach the ground");
    }
    let floor_post_h = r - dk - jst - beam;
    if raised && floor_post_h <= 0.0 {
        return refuse(format!(
            "a floor raised {raise} cm leaves no room for its posts under the joists"
        ));
    }
    let slope = (w / 2.0 + o) / pitch.cos();
    let rafters_per_bay = ((b / f64::from(rules.rafter_spacing_cm)).round() as usize).max(1);
    let joists_per_bay =
        (((b / f64::from(rules.joist_spacing_cm)).round() as usize).max(2) - 1).max(1);
    if rafters_per_bay > MAX_MEMBERS_PER_BAY / 2 || joists_per_bay > MAX_MEMBERS_PER_BAY {
        return refuse("the rules space rafters or joists too closely");
    }
    let joist_spacing = b / (joists_per_bay + 1) as f64;
    let ladder_tilt = f64::from(LADDER_TILT_CENTIDEG) / 100.0 * TAU / 360.0;
    let ladder_run = |rise: f64| rise / ladder_tilt.tan();
    let ladder_len = |rise: f64| rise / ladder_tilt.sin();
    if raised && wt / 2.0 + ladder_run(r) > o {
        return refuse(format!(
            "the ladder up to a floor raised {raise} cm would stand beyond the roof's edge"
        ));
    }

    let a = f64::from(angle) / TURN * TAU;
    let (ca, sa) = (a.cos(), a.sin());
    let (cxf, cyf) = (f64::from(cx), f64::from(cy));
    let at = |u: f64, v: f64| (cxf + u * ca - v * sa, cyf + u * sa + v * ca);
    let along = angle;
    let across = turned(angle, 1);
    let back = turned(angle, 2);
    let back_across = turned(angle, 3);
    let u_line = |k: usize| -l / 2.0 + b * k as f64;
    let long_walls = [(SIDE_RIGHT, w / 2.0), (SIDE_LEFT, -w / 2.0)];
    let end_walls = [(SIDE_BACK, -l / 2.0), (SIDE_FRONT, l / 2.0)];
    let slopes = [(0_usize, 1.0_f64), (1, -1.0)];
    let storey_level = |t: i32| if t == 0 { Level::Ground } else { Level::Upper };

    let mut parts: Vec<Part> = Vec::new();
    let mut push = |level: Level,
                    slot: Option<usize>,
                    kind: PartKind,
                    index: usize,
                    stage: Stage,
                    (u, v, z): (f64, f64, f64),
                    size: [f64; 3],
                    angle: u16,
                    tilt: i32,
                    material: Option<u8>| {
        let (x, y) = at(u, v);
        parts.push(Part {
            id: part_id(level, slot, kind, index),
            kind,
            stage,
            at: [cm(x), cm(y), cm(z)],
            size: size.map(cm),
            angle,
            tilt,
            material,
        });
    };

    // Foundation: postholes, and a raised floor's posts down the middle.
    for k in 0..=n {
        for (side, v) in long_walls {
            push(
                Level::Ground,
                Some(k),
                PartKind::Posthole,
                side as usize,
                Stage::Foundation,
                (u_line(k), v, -depth / 2.0),
                [pd + POSTHOLE_SLACK_CM, pd + POSTHOLE_SLACK_CM, depth],
                along,
                0,
                None,
            );
        }
        if raised {
            push(
                Level::Ground,
                Some(k),
                PartKind::Posthole,
                2,
                Stage::Foundation,
                (u_line(k), 0.0, -depth / 2.0),
                [pd + POSTHOLE_SLACK_CM, pd + POSTHOLE_SLACK_CM, depth],
                along,
                0,
                None,
            );
            push(
                Level::Ground,
                Some(k),
                PartKind::FloorPost,
                0,
                Stage::Foundation,
                (u_line(k), 0.0, (floor_post_h - depth) / 2.0),
                [pd, pd, floor_post_h + depth],
                along,
                0,
                Some(TIMBER),
            );
        }
    }

    // Frame: posts through every storey, plates along the wall heads a bay at a time, tie beams
    // across them at each frame line.
    for k in 0..=n {
        for (side, v) in long_walls {
            push(
                Level::Ground,
                Some(k),
                PartKind::Post,
                side as usize,
                Stage::Frame,
                (u_line(k), v, (z_p - depth) / 2.0),
                [pd, pd, z_p + depth],
                along,
                0,
                Some(TIMBER),
            );
        }
        push(
            Level::Roof,
            Some(k),
            PartKind::TieBeam,
            0,
            Stage::Frame,
            (u_line(k), 0.0, z_p + beam / 2.0),
            [w, beam, beam],
            across,
            0,
            Some(TIMBER),
        );
    }
    for j in 0..n {
        for (side, v) in long_walls {
            push(
                Level::Roof,
                Some(j),
                PartKind::WallPlate,
                side as usize,
                Stage::Frame,
                (u_line(j) + b / 2.0, v, z_p - beam / 2.0),
                [b, beam, beam],
                along,
                0,
                Some(TIMBER),
            );
        }
    }
    // Floors on joists: an upper storey's, and a raised floor (with its middle rail).
    let floors: Vec<(Level, f64, bool)> = [
        (raised).then_some((Level::Ground, z_g, true)),
        (storeys > 1).then_some((Level::Upper, z_g + h, false)),
    ]
    .into_iter()
    .flatten()
    .collect();
    for &(level, z_floor, middle) in &floors {
        let z_joist = z_floor - dk - jst / 2.0;
        let z_rail = z_floor - dk - jst - beam / 2.0;
        let rails: Vec<(usize, f64)> = long_walls
            .iter()
            .map(|&(side, v)| (side as usize, v))
            .chain(middle.then_some((2, 0.0)))
            .collect();
        for j in 0..n {
            for &(index, v) in &rails {
                push(
                    level,
                    Some(j),
                    PartKind::WallPlate,
                    index,
                    Stage::Frame,
                    (u_line(j) + b / 2.0, v, z_rail),
                    [b, beam, beam],
                    along,
                    0,
                    Some(TIMBER),
                );
            }
            for i in 0..joists_per_bay {
                push(
                    level,
                    Some(j),
                    PartKind::Joist,
                    i,
                    Stage::Frame,
                    (u_line(j) + joist_spacing * (i + 1) as f64, 0.0, z_joist),
                    [w, jst, jst],
                    across,
                    0,
                    Some(TIMBER),
                );
            }
        }
        for k in 0..=n {
            push(
                level,
                Some(k),
                PartKind::TieBeam,
                0,
                Stage::Frame,
                (u_line(k), 0.0, z_floor - dk - beam / 2.0),
                [w, beam, beam],
                across,
                0,
                Some(TIMBER),
            );
        }
    }
    // Lofts: joists across the plates over the bays the spec names.
    for &j in &lofts {
        for i in 0..joists_per_bay {
            push(
                Level::Loft,
                Some(j),
                PartKind::Joist,
                i,
                Stage::Frame,
                (
                    u_line(j) + joist_spacing * (i + 1) as f64,
                    0.0,
                    z_p + jst / 2.0,
                ),
                [w, jst, jst],
                across,
                0,
                Some(TIMBER),
            );
        }
    }
    // Rafters from the eaves to the ridge, in pairs along the length (the last pair on the end
    // frame line, which a further bay would begin with), and the ridge a bay at a time.
    for line in 0..=n {
        let count = if line < n { rafters_per_bay } else { 1 };
        for i in 0..count {
            let u = u_line(line) + b * i as f64 / rafters_per_bay as f64;
            for (sl, sign) in slopes {
                push(
                    Level::Roof,
                    Some(line),
                    PartKind::Rafter,
                    i * 2 + sl,
                    Stage::Frame,
                    (u, sign * (w / 2.0 + o) / 2.0, (z_e + z_a) / 2.0),
                    [slope, raf, raf],
                    if sign > 0.0 { back_across } else { across },
                    pitch_cd,
                    Some(TIMBER),
                );
            }
        }
    }
    for j in 0..n {
        let u0 = u_line(j) - if j == 0 { o } else { 0.0 };
        let u1 = u_line(j + 1) + if j + 1 == n { o } else { 0.0 };
        push(
            Level::Roof,
            Some(j),
            PartKind::Ridge,
            0,
            Stage::Frame,
            ((u0 + u1) / 2.0, 0.0, z_a),
            [u1 - u0, beam, beam],
            along,
            0,
            Some(TIMBER),
        );
    }

    // Walls: wattle and daub between the posts, storey by storey, the door's panel split either
    // side of it; and the gables.
    let pieces = |panel: f64, door_here: bool| -> Vec<(usize, f64, f64)> {
        if door_here {
            let side = (panel - door_w) / 2.0;
            vec![
                (0, -(door_w + side) / 2.0, side),
                (1, (door_w + side) / 2.0, side),
            ]
        } else {
            vec![(0, 0.0, panel)]
        }
    };
    let mut wall_area_cm2 = 0.0;
    for t in 0..storeys {
        let level = storey_level(t);
        let zc = z_g + h * f64::from(t) + h / 2.0;
        for kind in [PartKind::Wattle, PartKind::Daub] {
            let (thickness, material) = match kind {
                PartKind::Wattle => (wt / 3.0, Some(WATTLE)),
                _ => (wt, None),
            };
            for j in 0..n {
                for (side, v) in long_walls {
                    let door_here = t == 0 && side == door_side && j as i32 == door_bay;
                    for (half, du, pw) in pieces(b - pd, door_here) {
                        if kind == PartKind::Wattle {
                            wall_area_cm2 += pw * h;
                        }
                        push(
                            level,
                            Some(j),
                            kind,
                            side as usize * 2 + half,
                            Stage::Walls,
                            (u_line(j) + b / 2.0 + du, v, zc),
                            [pw, thickness, h],
                            along,
                            0,
                            material,
                        );
                    }
                }
            }
            for (side, u) in end_walls {
                let door_here = t == 0 && side == door_side;
                for (half, dv, pw) in pieces(w - pd, door_here) {
                    if kind == PartKind::Wattle {
                        wall_area_cm2 += pw * h;
                    }
                    push(
                        level,
                        None,
                        kind,
                        side as usize * 2 + half,
                        Stage::Walls,
                        (u, dv, zc),
                        [pw, thickness, h],
                        across,
                        0,
                        material,
                    );
                }
            }
        }
    }
    let gable_h = z_a - z_p;
    for (side, u) in end_walls {
        wall_area_cm2 += (w - pd) * gable_h / 2.0;
        push(
            Level::Roof,
            None,
            PartKind::GableInfill,
            side as usize,
            Stage::Walls,
            (u, 0.0, z_p + gable_h / 3.0),
            [w - pd, wt, gable_h],
            across,
            0,
            Some(WATTLE),
        );
    }

    // Roof: the covering, a slope at a time.
    for (sl, sign) in slopes {
        push(
            Level::Roof,
            None,
            PartKind::Thatch,
            sl,
            Stage::Roof,
            (0.0, sign * (w / 2.0 + o) / 2.0, (z_e + z_a) / 2.0),
            [slope, l + 2.0 * o, f64::from(rules.thatch_thickness_cm)],
            if sign > 0.0 { back_across } else { across },
            pitch_cd,
            Some(COVERING),
        );
    }

    // Finish: a floor beaten on the ground, or boards on the joists of every floor; the hearth on
    // the lowest living storey; ladders up to a raised floor, an upper storey and a loft.
    if !raised {
        push(
            Level::Ground,
            None,
            PartKind::Floor,
            0,
            Stage::Finish,
            (0.0, 0.0, 0.0),
            [l, w, 0.0],
            along,
            0,
            None,
        );
    }
    let mut decks: Vec<(Level, usize, f64)> = Vec::new();
    for &(level, z_floor, _) in &floors {
        decks.extend((0..n).map(|j| (level, j, z_floor)));
    }
    decks.extend(lofts.iter().map(|&j| (Level::Loft, j, z_p + jst + dk)));
    for &(level, j, z_floor) in &decks {
        push(
            level,
            Some(j),
            PartKind::Decking,
            0,
            Stage::Finish,
            (u_line(j) + b / 2.0, 0.0, z_floor - dk / 2.0),
            [b, w, dk],
            along,
            0,
            Some(BOARDS),
        );
    }
    let uses = [rules.ground_use, rules.upper_use];
    if let Some(t) = (0..storeys).find(|&t| uses[t as usize] == SpaceUse::Living) {
        let hearth = f64::from(rules.hearth_cm);
        push(
            storey_level(t),
            None,
            PartKind::Hearth,
            0,
            Stage::Finish,
            (u_line(n / 2) + b / 2.0, 0.0, z_g + h * f64::from(t)),
            [hearth, hearth, HEARTH_HEIGHT_CM],
            along,
            0,
            None,
        );
    }
    let mut ladders = 0_usize;
    let ladder_size = |rise: f64| {
        [
            ladder_len(rise),
            f64::from(LADDER_WIDTH_CM),
            LADDER_DEPTH_CM,
        ]
    };
    if raised {
        // Outside the door, its top at the doorway, its foot under the eaves.
        let out = wt / 2.0 + ladder_run(r) / 2.0;
        let (u, v, up) = match door_side {
            SIDE_RIGHT => (
                u_line(door_bay as usize) + b / 2.0,
                w / 2.0 + out,
                back_across,
            ),
            SIDE_LEFT => (u_line(door_bay as usize) + b / 2.0, -w / 2.0 - out, across),
            SIDE_BACK => (-l / 2.0 - out, 0.0, along),
            _ => (l / 2.0 + out, 0.0, back),
        };
        ladders += 1;
        push(
            Level::Ground,
            None,
            PartKind::Ladder,
            0,
            Stage::Finish,
            (u, v, r / 2.0),
            ladder_size(r),
            up,
            LADDER_TILT_CENTIDEG,
            Some(BOARDS),
        );
    }
    if storeys > 1 {
        ladders += 1;
        push(
            Level::Upper,
            None,
            PartKind::Ladder,
            0,
            Stage::Finish,
            (u_line(0) + b / 2.0, -w / 4.0, z_g + h / 2.0),
            ladder_size(h),
            along,
            LADDER_TILT_CENTIDEG,
            Some(BOARDS),
        );
    }
    if let Some(&j) = lofts.first() {
        let from = z_g + h * f64::from(storeys - 1);
        let rise = z_p + jst + dk - from;
        ladders += 1;
        push(
            Level::Loft,
            None,
            PartKind::Ladder,
            0,
            Stage::Finish,
            (u_line(j) + b / 2.0, w / 4.0, from + rise / 2.0),
            ladder_size(rise),
            along,
            LADDER_TILT_CENTIDEG,
            Some(BOARDS),
        );
    }
    parts.sort_by_key(|p| (p.stage, p.id));

    // Outline, roof and door.
    let corners = |du: f64, dv: f64| -> Vec<(i32, i32)> {
        [(-du, -dv), (du, -dv), (du, dv), (-du, dv)]
            .into_iter()
            .map(|(u, v)| {
                let (x, y) = at(u, v);
                (cm(x), cm(y))
            })
            .collect()
    };
    let outline = corners(l / 2.0 + wt / 2.0, w / 2.0 + wt / 2.0);
    let roof_outline = corners(l / 2.0 + o, w / 2.0 + o);
    let ridge = {
        let (x0, y0) = at(-l / 2.0 - o, 0.0);
        let (x1, y1) = at(l / 2.0 + o, 0.0);
        [(cm(x0), cm(y0)), (cm(x1), cm(y1))]
    };
    let (door_u, door_v, facing) = match door_side {
        SIDE_RIGHT => (u_line(door_bay as usize) + b / 2.0, w / 2.0, across),
        SIDE_LEFT => (u_line(door_bay as usize) + b / 2.0, -w / 2.0, back_across),
        SIDE_BACK => (-l / 2.0, 0.0, back),
        _ => (l / 2.0, 0.0, along),
    };
    let (dx, dy) = at(door_u, door_v);

    // Spaces and what they give.
    let m2 = 1.0e-4;
    let storey_m2 = l * w * m2;
    let bay_m2 = b * w * m2;
    let mut spaces = vec![Space {
        level: Level::Ground,
        bays: all_bays as u16,
        area_m2: storey_m2,
        use_: rules.ground_use,
    }];
    if storeys > 1 {
        spaces.push(Space {
            level: Level::Upper,
            bays: all_bays as u16,
            area_m2: storey_m2,
            use_: rules.upper_use,
        });
    }
    if !lofts.is_empty() {
        spaces.push(Space {
            level: Level::Loft,
            bays: loft_mask as u16,
            area_m2: bay_m2 * lofts.len() as f64,
            use_: SpaceUse::Store,
        });
    }
    let mut floor_by_use = [0.0; 3];
    let mut storage_kg = [0.0; 3];
    let mut work_places = 0;
    for s in &spaces {
        floor_by_use[s.use_.index()] += s.area_m2;
        match (s.use_, s.level) {
            (SpaceUse::Store, Level::Loft) => storage_kg[1] += s.area_m2 * rules.loft_kg_per_m2,
            (SpaceUse::Store, Level::Ground) if raised => {
                storage_kg[0] += s.area_m2 * rules.store_kg_per_m2;
            }
            (SpaceUse::Store, _) => storage_kg[2] += s.area_m2 * rules.store_kg_per_m2,
            _ => storage_kg[2] += s.area_m2 * rules.living_kg_per_m2,
        }
        if s.use_ == SpaceUse::Work && rules.work_m2_per_worker > 0.0 {
            work_places += (s.area_m2 / rules.work_m2_per_worker).floor() as u32;
        }
    }
    let floor_area_m2: f64 = spaces.iter().map(|s| s.area_m2).sum();
    let living = floor_by_use[SpaceUse::Living.index()];
    let sleeping_places = if living > 0.0 && rules.floor_m2_per_sleeper > 0.0 {
        ((living - rules.floor_base_m2).max(0.0) / rules.floor_m2_per_sleeper).floor() as u32
    } else {
        0
    };

    // Members, by kind.
    let wall_posts = 2 * (n + 1);
    let floor_posts = if raised { n + 1 } else { 0 };
    let rafters = 2 * (n * rafters_per_bay + 1);
    let floored_bays = decks.len();
    let joists = joists_per_bay * floored_bays;
    let rails = floors
        .iter()
        .map(|&(_, _, middle)| if middle { 3 } else { 2 })
        .sum::<usize>()
        * n;
    let cross_beams = (1 + floors.len()) * (n + 1);
    let beams_m = (2 * n + rails) as f64 * b * 0.01 + cross_beams as f64 * w * 0.01;
    let ridge_m = (l + 2.0 * o) * 0.01;
    let roof_m2 = (l + 2.0 * o) * (w + 2.0 * o) * m2 / pitch.cos();
    let wall_m2 = wall_area_cm2 * m2;
    let deck_m2 = floored_bays as f64 * bay_m2;
    let beaten_m2 = if raised { 0.0 } else { storey_m2 };
    let timber = |d: f64, count: usize, len_cm: f64| {
        round_section_m2(d) * count as f64 * len_cm * 0.01 * rules.timber_kg_per_m3
    };
    let floor_posts_kg = timber(pd, floor_posts, floor_post_h + depth);
    let frame_kg = timber(pd, wall_posts, z_p + depth)
        + timber(beam, 1, beams_m * 100.0 + ridge_m * 100.0)
        + timber(raf, rafters, slope)
        + timber(jst, joists, w);
    let boards_kg = deck_m2 * dk * 0.01 * rules.timber_kg_per_m3 + rules.ladder_kg * ladders as f64;
    let slots = |timber: f64, wattle: f64, covering: f64, boards: f64| {
        let mut kg = vec![0.0; frame_materials::COUNT];
        kg[usize::from(TIMBER)] = timber;
        kg[usize::from(WATTLE)] = wattle;
        kg[usize::from(COVERING)] = covering;
        kg[usize::from(BOARDS)] = boards;
        kg
    };
    let stages = vec![
        StageNeeds {
            stage: Stage::Foundation,
            labour_h: rules.groundwork_h_per_m2 * storey_m2
                + rules.posthole_h * (wall_posts + floor_posts) as f64
                + rules.post_h * floor_posts as f64,
            materials_kg: slots(floor_posts_kg, 0.0, 0.0, 0.0),
        },
        StageNeeds {
            stage: Stage::Frame,
            labour_h: rules.post_h * wall_posts as f64
                + rules.beam_h_per_m * (beams_m + ridge_m)
                + rules.rafter_h * rafters as f64
                + rules.joist_h_per_m * joists as f64 * w * 0.01,
            materials_kg: slots(frame_kg, 0.0, 0.0, 0.0),
        },
        StageNeeds {
            stage: Stage::Walls,
            labour_h: (rules.wattle_h_per_m2 + rules.daub_h_per_m3 * wt * 0.01) * wall_m2,
            materials_kg: slots(0.0, rules.wattle_kg_per_m2 * wall_m2, 0.0, 0.0),
        },
        StageNeeds {
            stage: Stage::Roof,
            labour_h: rules.thatch_h_per_m2 * roof_m2,
            materials_kg: slots(0.0, 0.0, rules.thatch_kg_per_m2 * roof_m2, 0.0),
        },
        StageNeeds {
            stage: Stage::Finish,
            labour_h: rules.finish_h_per_m2 * beaten_m2
                + rules.decking_h_per_m2 * deck_m2
                + rules.ladder_h * ladders as f64,
            materials_kg: slots(0.0, 0.0, 0.0, boards_kg),
        },
    ];

    // Component groups (ADR-0009 §3), each with what it carries or covers.
    let roof_plan_side_m2 = (l + 2.0 * o) * (w / 2.0 + o) * m2;
    let suspended_m2 = deck_m2;
    let raised_m2 = if raised { storey_m2 } else { 0.0 };
    let joist_share = joists_per_bay as f64 / (joists_per_bay + 1) as f64;
    let mut groups = Vec::new();
    let mut group = |level: Level,
                     bay: Option<usize>,
                     kind: GroupKind,
                     index: u8,
                     stage: Stage,
                     material: Option<u8>,
                     count: usize,
                     section: [f64; 2],
                     length: f64,
                     spacing: f64,
                     area_m2: f64| {
        groups.push(Group {
            id: group_id(level, bay.map(|b| b as u8), kind, index),
            kind,
            stage,
            material,
            count: count as u32,
            section_cm: section.map(cm),
            length_cm: cm(length),
            spacing_cm: cm(spacing),
            area_m2,
        });
    };
    for (side, _) in long_walls {
        // Each long wall carries its slope of the roof and half of every floor across the width
        // (a raised floor's middle line takes half of that one).
        group(
            Level::Ground,
            None,
            GroupKind::Posts,
            side as u8,
            Stage::Frame,
            Some(TIMBER),
            n + 1,
            [pd, pd],
            z_p,
            b,
            roof_plan_side_m2 + (suspended_m2 - raised_m2) / 2.0 + raised_m2 / 4.0,
        );
    }
    if raised {
        group(
            Level::Ground,
            None,
            GroupKind::Posts,
            2,
            Stage::Foundation,
            Some(TIMBER),
            n + 1,
            [pd, pd],
            floor_post_h,
            b,
            raised_m2 / 2.0,
        );
    }
    // Tie beams at the wall heads carry the edges of the lofts beside them; cross beams carry
    // their floor's edges, the share its joists do not.
    group(
        Level::Roof,
        None,
        GroupKind::TieBeams,
        0,
        Stage::Frame,
        Some(TIMBER),
        n + 1,
        [beam, beam],
        w,
        b,
        lofts.len() as f64 * bay_m2 * (1.0 - joist_share),
    );
    for &(level, _, middle) in &floors {
        group(
            level,
            None,
            GroupKind::TieBeams,
            0,
            Stage::Frame,
            Some(TIMBER),
            n + 1,
            [beam, beam],
            if middle { w / 2.0 } else { w },
            b,
            storey_m2 * (1.0 - joist_share),
        );
    }
    for &(level, j, _) in &decks {
        let (kind, span) = match level {
            Level::Loft => (GroupKind::LoftJoists, w),
            Level::Upper => (GroupKind::FloorJoists, w),
            _ => (GroupKind::RaisedFloor, w / 2.0),
        };
        group(
            level,
            Some(j),
            kind,
            0,
            Stage::Frame,
            Some(TIMBER),
            joists_per_bay,
            [jst, jst],
            span,
            joist_spacing,
            bay_m2 * joist_share,
        );
    }
    for j in 0..n {
        let ends = if j == 0 { o } else { 0.0 } + if j + 1 == n { o } else { 0.0 };
        let last = if j + 1 == n { 2 } else { 0 };
        group(
            Level::Roof,
            Some(j),
            GroupKind::RoofFrame,
            0,
            Stage::Frame,
            Some(TIMBER),
            2 * rafters_per_bay + last,
            [raf, raf],
            slope,
            b / rafters_per_bay as f64,
            (b + ends) * (w + 2.0 * o) * m2 / pitch.cos(),
        );
    }
    for (sl, _) in slopes {
        group(
            Level::Roof,
            None,
            GroupKind::Covering,
            sl as u8,
            Stage::Roof,
            Some(COVERING),
            1,
            [f64::from(rules.thatch_thickness_cm); 2],
            slope,
            0.0,
            roof_m2 / 2.0,
        );
    }
    let storeys_h = h * f64::from(storeys);
    let door_m2 = door_w * h * m2;
    for (side, _) in long_walls {
        let door = if side == door_side { door_m2 } else { 0.0 };
        group(
            Level::Ground,
            None,
            GroupKind::Infill,
            side as u8,
            Stage::Walls,
            Some(WATTLE),
            n * storeys as usize + usize::from(side == door_side),
            [wt, wt],
            storeys_h,
            b,
            (b - pd) * storeys_h * m2 * n as f64 - door,
        );
    }
    for (side, _) in end_walls {
        let door = if side == door_side { door_m2 } else { 0.0 };
        group(
            Level::Ground,
            None,
            GroupKind::Infill,
            side as u8,
            Stage::Walls,
            Some(WATTLE),
            storeys as usize + 1 + usize::from(side == door_side),
            [wt, wt],
            storeys_h + gable_h,
            0.0,
            (w - pd) * (storeys_h + gable_h / 2.0) * m2 - door,
        );
    }
    if !raised {
        group(
            Level::Ground,
            None,
            GroupKind::Floor,
            0,
            Stage::Finish,
            None,
            1,
            [0.0, 0.0],
            0.0,
            0.0,
            storey_m2,
        );
    }
    if groups.len() > MAX_GROUPS {
        return refuse(format!(
            "{} component groups are more than {MAX_GROUPS}",
            groups.len()
        ));
    }
    groups.sort_by_key(|g| (g.stage, g.id));

    Ok(Expansion {
        grammar: Grammar::Frame,
        version: FRAME_VERSION,
        parts,
        outline,
        roof_radius_cm: cm((l / 2.0 + o).hypot(w / 2.0 + o)),
        apex_cm: cm(z_a),
        door: (cm(dx), cm(dy), facing),
        door_width_cm: rules.door_cm,
        stages,
        floor_area_m2,
        sleeping_places,
        roof_outline,
        ridge: Some(ridge),
        spaces,
        groups,
        floor_by_use,
        storage_kg,
        work_places,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PARAMS, expansion_hash};
    use frame_params as fp;
    use std::collections::HashSet;

    /// A house's rules, close to the core longhouse's.
    fn house() -> FrameRules {
        FrameRules {
            bays: (1, 8),
            bay_cm: (200, 400),
            width_cm: (300, 700),
            storeys: (1, 2),
            eave_cm: (180, 250),
            pitch_centideg: (4500, 5500),
            joist_cm: (8, 25),
            post_cm: (12, 30),
            overhang_cm: (30, 80),
            floor_raise_cm: (0, 0),
            wall_cm: (10, 25),
            lofts: true,
            ground_use: SpaceUse::Living,
            upper_use: SpaceUse::Living,
            posthole_depth_cm: 70,
            beam_cm: 15,
            rafter_cm: 10,
            rafter_spacing_cm: 60,
            joist_spacing_cm: 60,
            decking_cm: 4,
            thatch_thickness_cm: 30,
            hearth_cm: 90,
            door_cm: 90,
            floor_base_m2: 6.0,
            floor_m2_per_sleeper: 4.8,
            work_m2_per_worker: 6.0,
            store_kg_per_m2: 500.0,
            loft_kg_per_m2: 150.0,
            living_kg_per_m2: 60.0,
            timber_kg_per_m3: 760.0,
            groundwork_h_per_m2: 3.0,
            posthole_h: 0.8,
            post_h: 12.0,
            beam_h_per_m: 4.0,
            rafter_h: 2.0,
            joist_h_per_m: 1.0,
            wattle_h_per_m2: 1.0,
            daub_h_per_m3: 13.3,
            thatch_h_per_m2: 2.4,
            finish_h_per_m2: 3.0,
            decking_h_per_m2: 2.0,
            ladder_h: 4.0,
            wattle_kg_per_m2: 6.0,
            thatch_kg_per_m2: 30.0,
            ladder_kg: 15.0,
        }
    }

    /// A raised granary's rules.
    fn granary() -> FrameRules {
        FrameRules {
            bays: (1, 2),
            bay_cm: (200, 350),
            width_cm: (200, 400),
            storeys: (1, 1),
            floor_raise_cm: (50, 120),
            overhang_cm: (40, 80),
            lofts: false,
            ground_use: SpaceUse::Store,
            upper_use: SpaceUse::Store,
            ..house()
        }
    }

    /// A workshop's rules: a working floor with a store loft.
    fn workshop() -> FrameRules {
        FrameRules {
            ground_use: SpaceUse::Work,
            upper_use: SpaceUse::Store,
            ..house()
        }
    }

    fn spec(bays: i32, bay_cm: i32, width: i32) -> BuildingSpec {
        let mut params = [0; PARAMS];
        params[fp::EAVE_CM] = 200;
        params[fp::PITCH_CENTIDEG] = 4500;
        params[fp::BAYS] = bays;
        params[fp::DOOR] = fp::door(fp::SIDE_RIGHT, bays / 2);
        params[fp::OVERHANG_CM] = 50;
        params[fp::POST_CM] = 15;
        params[fp::WALL_CM] = 15;
        BuildingSpec {
            program: "core:building/longhouse".into(),
            version: FRAME_VERSION,
            footprint: Footprint::Rect {
                x: 100_000,
                y: 200_000,
                length: bays * bay_cm,
                width,
                angle: 6_000,
            },
            storeys: 1,
            params,
            materials: vec![
                "core:good/timber".into(),
                "core:good/timber".into(),
                "core:good/thatch".into(),
                "core:good/timber".into(),
            ],
            style_seed: 0,
        }
    }

    /// Three bays of 2.5 m by 5 m with a loft over the first.
    fn three_bays_one_loft() -> BuildingSpec {
        let mut s = spec(3, 250, 500);
        s.params[fp::LOFT_BAYS] = 0b001;
        s.params[fp::JOIST_CM] = 15;
        s
    }

    fn two_storeys() -> BuildingSpec {
        let mut s = spec(4, 250, 500);
        s.storeys = 2;
        s.params[fp::JOIST_CM] = 18;
        s
    }

    fn raised_store() -> BuildingSpec {
        let mut s = spec(1, 300, 300);
        s.program = "core:building/granary".into();
        s.params[fp::FLOOR_RAISE_CM] = 80;
        s.params[fp::JOIST_CM] = 15;
        s.params[fp::DOOR] = fp::door(fp::SIDE_FRONT, 0);
        s
    }

    fn workshop_spec() -> BuildingSpec {
        let mut s = spec(2, 300, 500);
        s.program = "core:building/workshop".into();
        s.params[fp::LOFT_BAYS] = 0b10;
        s.params[fp::JOIST_CM] = 12;
        s.params[fp::DOOR] = fp::door(fp::SIDE_BACK, 0);
        s
    }

    fn goldens() -> [(BuildingSpec, FrameRules, u64); 4] {
        [
            (three_bays_one_loft(), house(), GOLDEN_THREE_BAYS),
            (two_storeys(), house(), GOLDEN_TWO_STOREYS),
            (raised_store(), granary(), GOLDEN_RAISED_STORE),
            (workshop_spec(), workshop(), GOLDEN_WORKSHOP),
        ]
    }

    /// The point `(x, y)` in the building's own axes, centimetres from its centre.
    fn local(spec: &BuildingSpec, x: f64, y: f64) -> (f64, f64) {
        let Footprint::Rect {
            x: cx,
            y: cy,
            angle,
            ..
        } = spec.footprint
        else {
            unreachable!()
        };
        let a = f64::from(angle) / TURN * TAU;
        let (dx, dy) = (x - f64::from(cx), y - f64::from(cy));
        (dx * a.cos() + dy * a.sin(), -dx * a.sin() + dy * a.cos())
    }

    #[test]
    fn three_bays_with_a_loft_have_their_parts_spaces_and_groups() {
        let s = three_bays_one_loft();
        let e = expand_frame(&s, &house()).expect("expands");
        let count = |k: PartKind| e.parts.iter().filter(|p| p.kind == k).count();
        // Four frame lines of two posts; plates two a bay; a tie beam on each line.
        assert_eq!(count(PartKind::Posthole), 8);
        assert_eq!(count(PartKind::Post), 8);
        assert_eq!(count(PartKind::WallPlate), 6);
        assert_eq!(count(PartKind::TieBeam), 4);
        // 250 / 60 ≈ 4 rafters a bay on each slope, and the pair on the last line.
        assert_eq!(count(PartKind::Rafter), 2 * (3 * 4 + 1));
        assert_eq!(count(PartKind::Ridge), 3);
        // 250 / 60 ≈ 4 spaces: 3 joists inside the loft's bay.
        assert_eq!(count(PartKind::Joist), 3);
        assert_eq!(count(PartKind::Decking), 1);
        assert_eq!(count(PartKind::Ladder), 1);
        // Six long-wall panels, one split by the door, and two end walls.
        assert_eq!(count(PartKind::Wattle), 9);
        assert_eq!(count(PartKind::Daub), 9);
        assert_eq!(count(PartKind::GableInfill), 2);
        assert_eq!(count(PartKind::Thatch), 2);
        assert_eq!(count(PartKind::Floor), 1);
        assert_eq!(count(PartKind::Hearth), 1);
        assert!(e.parts.windows(2).all(|w| w[0].stage <= w[1].stage));
        let ids: HashSet<u32> = e.parts.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), e.parts.len(), "part ids are unique");
        // Posts stand on the long wall lines, 2.5 m apart.
        for p in e.parts.iter().filter(|p| p.kind == PartKind::Post) {
            let (u, v) = local(&s, f64::from(p.at[0]), f64::from(p.at[1]));
            assert!((v.abs() - 250.0).abs() <= 1.0, "{v}");
            assert!(((u + 375.0) / 250.0 - ((u + 375.0) / 250.0).round()).abs() < 0.01);
        }
        // 37.5 m² of living floor and 12.5 m² of loft; the ridge 2.5 m above the plates at 45°.
        assert_eq!(e.grammar, Grammar::Frame);
        assert!((e.floor_by_use[SpaceUse::Living.index()] - 37.5).abs() < 1e-9);
        assert!((e.floor_by_use[SpaceUse::Store.index()] - 12.5).abs() < 1e-9);
        assert_eq!(e.floor_by_use[SpaceUse::Work.index()], 0.0);
        assert_eq!(e.sleeping_places, 6, "(37.5 - 6) / 4.8");
        assert_eq!(e.apex_cm, 450);
        assert_eq!(e.spaces.len(), 2);
        assert_eq!(e.spaces[1].level, Level::Loft);
        assert_eq!(e.spaces[1].bays, 0b001);
        // Stores: the loft at 150 kg/m², the living floor at 60.
        assert!((e.storage_kg[1] - 12.5 * 150.0).abs() < 1e-6);
        assert!((e.storage_kg[2] - 37.5 * 60.0).abs() < 1e-6);
        assert_eq!(e.storage_kg[0], 0.0);
        assert_eq!(e.outline.len(), 4);
        assert_eq!(e.roof_outline.len(), 4);
        // Groups: two post lines, tie beams, the loft's joists, three roof frames, two coverings,
        // four walls and the floor.
        assert_eq!(e.groups.len(), 14);
        let gids: HashSet<u32> = e.groups.iter().map(|g| g.id).collect();
        assert_eq!(gids.len(), e.groups.len(), "group ids are unique");
        let loft = e
            .groups
            .iter()
            .find(|g| g.kind == GroupKind::LoftJoists)
            .expect("the loft's joists");
        assert_eq!(
            loft.id,
            group_id(Level::Loft, Some(0), GroupKind::LoftJoists, 0)
        );
        assert_eq!(
            (loft.count, loft.length_cm, loft.section_cm),
            (3, 500, [15, 15])
        );
        assert_eq!(loft.spacing_cm, 63, "2.5 m in four spaces");
        // The door faces out of the right-hand long wall, in the middle bay.
        let (dx, dy, facing) = e.door;
        let (du, dv) = local(&s, f64::from(dx), f64::from(dy));
        assert!(du.abs() < 1.0 && (dv - 250.0).abs() < 1.0, "{du} {dv}");
        assert_eq!(facing, 6_000 + 16_384);
        assert_eq!(e.door_width_cm, 90);
    }

    #[test]
    fn the_four_goldens_are_pinned() {
        for (s, r, golden) in goldens() {
            let e = expand_frame(&s, &r).expect("expands");
            let again = expand_frame(&s, &r).expect("expands");
            assert_eq!(e, again, "expansion is pure");
            assert!(
                (8..=MAX_GROUPS).contains(&e.groups.len()),
                "{} groups",
                e.groups.len()
            );
            // A change here is a change to what frame buildings are: it must bump FRAME_VERSION
            // (ADR-0009 §1).
            assert_eq!(
                expansion_hash(&e),
                golden,
                "{}: {:#x}",
                s.program,
                expansion_hash(&e)
            );
        }
    }

    #[test]
    fn two_storeys_have_an_upper_floor_and_a_ladder() {
        let e = expand_frame(&two_storeys(), &house()).expect("expands");
        assert_eq!(e.spaces.len(), 2);
        assert_eq!(e.spaces[1].level, Level::Upper);
        assert!((e.floor_area_m2 - 2.0 * 50.0).abs() < 1e-9);
        let floors = e
            .groups
            .iter()
            .filter(|g| g.kind == GroupKind::FloorJoists)
            .count();
        assert_eq!(floors, 4, "a joist group a bay");
        // Posts run through both storeys to the plates.
        let posts = e
            .groups
            .iter()
            .find(|g| g.kind == GroupKind::Posts)
            .expect("posts");
        assert_eq!(posts.length_cm, 400);
        let post = e
            .parts
            .iter()
            .find(|p| p.kind == PartKind::Post)
            .expect("a post");
        assert_eq!(post.size[2], 400 + 70);
        assert!(
            e.parts
                .iter()
                .any(|p| p.kind == PartKind::Ladder && p.id >> 24 == u32::from(Level::Upper.code()))
        );
        // Living on both storeys: sleeping places from 100 m².
        assert_eq!(e.sleeping_places, ((100.0 - 6.0) / 4.8_f64).floor() as u32);
    }

    #[test]
    fn a_raised_store_stands_on_posts_and_holds_goods_off_the_ground() {
        let s = raised_store();
        let e = expand_frame(&s, &granary()).expect("expands");
        let count = |k: PartKind| e.parts.iter().filter(|p| p.kind == k).count();
        assert_eq!(
            count(PartKind::FloorPost),
            2,
            "one down the middle of each frame line"
        );
        assert_eq!(count(PartKind::Floor), 0, "no floor on the ground");
        assert_eq!(count(PartKind::Decking), 1);
        assert_eq!(count(PartKind::Hearth), 0, "a store has no hearth");
        assert_eq!(count(PartKind::Ladder), 1);
        assert!(e.storage_kg[0] > 0.0, "a raised store");
        assert_eq!((e.storage_kg[1], e.storage_kg[2]), (0.0, 0.0));
        assert_eq!(e.sleeping_places, 0);
        assert!(e.groups.iter().any(|g| g.kind == GroupKind::RaisedFloor));
        assert!(!e.groups.iter().any(|g| g.kind == GroupKind::Floor));
        // The walls start at the raised floor.
        let wall = e
            .parts
            .iter()
            .find(|p| p.kind == PartKind::Daub)
            .expect("a wall");
        assert_eq!(wall.at[2], 80 + 100);
    }

    #[test]
    fn a_workshop_has_places_to_work() {
        let e = expand_frame(&workshop_spec(), &workshop()).expect("expands");
        assert!((e.floor_by_use[SpaceUse::Work.index()] - 30.0).abs() < 1e-9);
        assert_eq!(e.work_places, 5, "30 m² at 6 m² a place");
        assert_eq!(e.sleeping_places, 0);
        assert!(!e.parts.iter().any(|p| p.kind == PartKind::Hearth));
        assert!((e.storage_kg[1] - 15.0 * 150.0).abs() < 1e-6, "the loft");
    }

    #[test]
    fn a_new_bay_keeps_every_id() {
        let r = house();
        for (bays, loft) in [(2, 0b01), (3, 0b101), (5, 0b10)] {
            let mut a = spec(bays, 250, 500);
            a.params[fp::LOFT_BAYS] = loft;
            a.params[fp::JOIST_CM] = 15;
            a.params[fp::DOOR] = fp::door(fp::SIDE_LEFT, 0);
            let mut b = a.clone();
            b.params[fp::BAYS] = bays + 1;
            if let Footprint::Rect { length, .. } = &mut b.footprint {
                *length += 250;
            }
            let (ea, eb) = (
                expand_frame(&a, &r).expect("expands"),
                expand_frame(&b, &r).expect("expands"),
            );
            let parts: HashSet<u32> = eb.parts.iter().map(|p| p.id).collect();
            let groups: HashSet<u32> = eb.groups.iter().map(|g| g.id).collect();
            for p in &ea.parts {
                assert!(parts.contains(&p.id), "part {:#x} ({:?})", p.id, p.kind);
            }
            for g in &ea.groups {
                assert!(groups.contains(&g.id), "group {:#x} ({:?})", g.id, g.kind);
            }
        }
    }

    #[test]
    fn every_part_stands_within_the_plot() {
        // The plot is the box round the roof (crate civ-agents); every part's plan lies under it.
        for (s, r, _) in goldens() {
            let e = expand_frame(&s, &r).expect("expands");
            let (mut x0, mut x1, mut y0, mut y1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
            for &(x, y) in &e.roof_outline {
                (x0, x1, y0, y1) = (x0.min(x), x1.max(x), y0.min(y), y1.max(y));
            }
            for p in &e.parts {
                let a = f64::from(p.angle) / TURN * TAU;
                let tilt = f64::from(p.tilt) / 100.0 * TAU / 360.0;
                let len = f64::from(p.size[0]) * tilt.cos();
                let wid = f64::from(p.size[1]);
                for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                    let (du, dv) = (su * len / 2.0, sv * wid / 2.0);
                    let x = f64::from(p.at[0]) + du * a.cos() - dv * a.sin();
                    let y = f64::from(p.at[1]) + du * a.sin() + dv * a.cos();
                    assert!(
                        x >= f64::from(x0) - 2.0
                            && x <= f64::from(x1) + 2.0
                            && y >= f64::from(y0) - 2.0
                            && y <= f64::from(y1) + 2.0,
                        "{}: {:?} {:#x} at ({x}, {y}) is beyond {x0}..{x1}, {y0}..{y1}",
                        s.program,
                        p.kind,
                        p.id
                    );
                }
            }
        }
    }

    #[test]
    fn areas_by_use_sum_to_the_floor() {
        for (s, r, _) in goldens() {
            let e = expand_frame(&s, &r).expect("expands");
            let by_use: f64 = e.floor_by_use.iter().sum();
            let spaces: f64 = e.spaces.iter().map(|s| s.area_m2).sum();
            assert!((by_use - e.floor_area_m2).abs() < 1e-9);
            assert!((spaces - e.floor_area_m2).abs() < 1e-9);
        }
    }

    #[test]
    fn the_roof_is_its_projection_over_the_cosine_of_its_pitch() {
        let r = house();
        for pitch in [4500, 5000, 5500] {
            let mut s = three_bays_one_loft();
            s.params[fp::PITCH_CENTIDEG] = pitch;
            let e = expand_frame(&s, &r).expect("expands");
            let roof = e.stages[Stage::Roof.index()].materials_kg[usize::from(COVERING)]
                / r.thatch_kg_per_m2;
            // 7.5 + 2 × 0.5 by 5 + 2 × 0.5 metres.
            let projection = 8.5 * 6.0;
            let theta = f64::from(pitch) / 100.0 * TAU / 360.0;
            assert!((roof - projection / theta.cos()).abs() < 1e-9, "{roof}");
            let covering: f64 = e
                .groups
                .iter()
                .filter(|g| g.kind == GroupKind::Covering)
                .map(|g| g.area_m2)
                .sum();
            assert!((covering - roof).abs() < 1e-9);
        }
    }

    #[test]
    fn deeper_joists_take_more_timber() {
        let r = house();
        let timber = |joist: i32| {
            let mut s = three_bays_one_loft();
            s.params[fp::JOIST_CM] = joist;
            let e = expand_frame(&s, &r).expect("expands");
            e.stages[Stage::Frame.index()].materials_kg[usize::from(TIMBER)]
        };
        assert!(timber(15) > timber(10));
        assert!(timber(20) > timber(15));
        // Three joists 5 m long: π/4 (0.20² − 0.10²) × 15 m × 760 kg/m³ more for 20 cm than 10.
        let extra = std::f64::consts::PI / 4.0 * (0.04 - 0.01) * 15.0 * 760.0;
        assert!((timber(20) - timber(10) - extra).abs() < 1e-6);
    }

    #[test]
    fn needs_cover_every_stage_and_slot() {
        let e = expand_frame(&three_bays_one_loft(), &house()).expect("expands");
        assert_eq!(e.stages.len(), Stage::ALL.len());
        assert!(e.stages.iter().all(|s| s.labour_h > 0.0));
        assert!(
            e.stages
                .iter()
                .all(|s| s.materials_kg.len() == frame_materials::COUNT)
        );
        let total = e.total_materials_kg();
        assert_eq!(total.len(), frame_materials::COUNT);
        assert!(total.iter().all(|&kg| kg > 0.0), "{total:?}");
        // 72 m² of roof at 30 kg/m² (research brief: 2.2 t for this house).
        assert!(
            (total[usize::from(COVERING)] - 2_163.7).abs() < 1.0,
            "{total:?}"
        );
        // A jointed frame takes 0.5 to 2 pd8 a square metre of footprint (11-04 §2.2).
        let frame_pd8_m2 = e.stages[Stage::Frame.index()].labour_h / 8.0 / 37.5;
        assert!((0.5..=2.0).contains(&frame_pd8_m2), "{frame_pd8_m2}");
    }

    #[test]
    fn bad_specs_are_refused() {
        let r = house();
        let refused = |f: &dyn Fn(&mut BuildingSpec)| {
            let mut s = three_bays_one_loft();
            f(&mut s);
            expand_frame(&s, &r).is_err()
        };
        assert!(!refused(&|_| ()));
        assert_eq!(
            expand_frame(
                &BuildingSpec {
                    version: 9,
                    ..three_bays_one_loft()
                },
                &r
            ),
            Err(GrammarError::UnknownVersion(9))
        );
        assert!(refused(&|s| s.footprint = Footprint::Round {
            x: 0,
            y: 0,
            radius: 300
        }));
        let set_length = |s: &mut BuildingSpec, v: i32| {
            if let Footprint::Rect { length, .. } = &mut s.footprint {
                *length = v;
            }
        };
        assert!(refused(&|s| set_length(s, 760)), "off the quantum");
        assert!(refused(&|s| set_length(s, 775)), "not three equal bays");
        assert!(refused(&|s| set_length(s, 1500)), "bays of 5 m");
        assert!(refused(&|s| s.params[fp::BAYS] = 9));
        assert!(
            refused(&|s| s.params[fp::LOFT_BAYS] = 0b1000),
            "no fourth bay"
        );
        assert!(
            refused(&|s| s.params[fp::JOIST_CM] = 0),
            "a loft needs joists"
        );
        assert!(
            refused(&|s| {
                s.params[fp::LOFT_BAYS] = 0;
            }),
            "no joists to size"
        );
        assert!(refused(
            &|s| s.params[fp::DOOR] = fp::door(fp::SIDE_RIGHT, 3)
        ));
        assert!(refused(&|s| s.params[fp::DOOR] = fp::door(fp::SIDE_BACK, 1)));
        assert!(refused(&|s| s.params[fp::DOOR] = fp::door(4, 0)));
        assert!(refused(&|s| s.params[fp::WALL_KIND] = 1));
        assert!(refused(&|s| s.params[fp::FOOTING] = 1));
        assert!(refused(&|s| s.params[fp::STYLE + 3] = 1));
        assert!(
            refused(&|s| s.params[fp::FLOOR_RAISE_CM] = 80),
            "a house is not raised"
        );
        assert!(refused(&|s| s.params[fp::PITCH_CENTIDEG] = 3000));
        assert!(refused(&|s| s.storeys = 3));
        assert!(refused(&|s| s.materials.pop().map_or((), drop)));
        // Two storeys of eight bays, each lofted, are more groups than a building may have.
        let mut s = spec(8, 250, 500);
        s.storeys = 2;
        s.params[fp::JOIST_CM] = 15;
        s.params[fp::LOFT_BAYS] = 0xff;
        assert!(matches!(
            expand_frame(&s, &r),
            Err(GrammarError::OutOfRange(m)) if m.contains("groups")
        ));
        // A granary raised too low for its posts, or too high for its ladder to stay dry.
        let g = granary();
        let mut low = raised_store();
        low.params[fp::FLOOR_RAISE_CM] = 30;
        assert!(expand_frame(&low, &g).is_err());
        let mut high = raised_store();
        high.params[fp::FLOOR_RAISE_CM] = 120;
        high.params[fp::OVERHANG_CM] = 40;
        assert!(expand_frame(&high, &g).is_err());
    }

    const GOLDEN_THREE_BAYS: u64 = 0xdb70_dcfc_e516_69b8;
    const GOLDEN_TWO_STOREYS: u64 = 0x7375_f0e6_7a98_0a0f;
    const GOLDEN_RAISED_STORE: u64 = 0xebfd_8b32_1aba_8021;
    const GOLDEN_WORKSHOP: u64 = 0xeadd_ad42_080a_e6ba;
}
