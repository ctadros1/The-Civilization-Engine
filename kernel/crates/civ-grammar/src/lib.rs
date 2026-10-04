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
//! There are two grammars (ADR-0009 §1). The **hut** (M1, frozen at version 1) is a round
//! post-built house with wattle-and-daub walls and a conical thatched roof. The **frame**
//! ([`frame`], M3b) builds rectilinear post-framed buildings in bays: houses, storehouses and
//! workshops of one or two storeys, with lofts and raised floors. Their rules are code; every
//! dimension, labour and material figure comes from content ([`HutRules`], [`FrameRules`]).
//! [`expand`] dispatches on the program's grammar. Geometry is in integer centimetres and angles
//! in 1/65,536 of a turn (ADR-0004 §1); floating point is used only on the way.

#![forbid(unsafe_code)]

use std::f64::consts::TAU;

pub mod frame;

pub use frame::{
    FRAME_VERSION, FrameRules, MAX_BAYS, MAX_GROUPS, MAX_STOREYS, expand_frame, frame_materials,
    frame_params,
};

/// Version of the hut grammar.
pub const HUT_VERSION: u16 = 1;

/// Integer parameters a spec carries (ADR-0009 §2: eight in M1, sixteen since grammar v2).
pub const PARAMS: usize = 16;

/// The grammar a program's buildings are expanded by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Grammar {
    /// The round hut (frozen at version 1).
    Hut,
    /// Rectilinear post-framed buildings in bays.
    Frame,
}

impl Grammar {
    /// The authored name: `hut` or `frame`.
    pub fn name(self) -> &'static str {
        match self {
            Grammar::Hut => "hut",
            Grammar::Frame => "frame",
        }
    }

    /// The grammar with an authored name.
    pub fn from_name(name: &str) -> Option<Grammar> {
        [Grammar::Hut, Grammar::Frame]
            .into_iter()
            .find(|g| g.name() == name)
    }
}

/// A program's rules, for the grammar it uses.
#[derive(Clone, Debug, PartialEq)]
pub enum ProgramRules {
    /// A hut's.
    Hut(HutRules),
    /// A frame building's.
    Frame(FrameRules),
}

impl ProgramRules {
    /// The grammar these rules are for.
    pub fn grammar(&self) -> Grammar {
        match self {
            ProgramRules::Hut(_) => Grammar::Hut,
            ProgramRules::Frame(_) => Grammar::Frame,
        }
    }
}

/// Expands a spec by its program's grammar (ADR-0009 §1): the version must be one the grammar
/// has, and the footprint the grammar's kind.
pub fn expand(spec: &BuildingSpec, rules: &ProgramRules) -> Result<Expansion, GrammarError> {
    match rules {
        ProgramRules::Hut(r) => expand_hut(spec, r),
        ProgramRules::Frame(r) => expand_frame(spec, r),
    }
}

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
    /// A rectangular building (ADR-0009 §2): the centre, the length and width between its wall
    /// lines, on a 25 cm quantum, and the direction its length runs.
    Rect {
        /// Centre, east.
        x: i32,
        /// Centre, south.
        y: i32,
        /// Length between the end wall lines.
        length: i32,
        /// Width between the long wall lines.
        width: i32,
        /// Direction the length runs, 1/65,536 of a turn from east toward south.
        angle: u16,
    },
}

impl Footprint {
    /// The centre, centimetres.
    pub fn centre(&self) -> (i32, i32) {
        match *self {
            Footprint::Round { x, y, .. } | Footprint::Rect { x, y, .. } => (x, y),
        }
    }

    /// The box round its wall lines, `[west, north, east, south]` in centimetres; `None` when it
    /// has no size.
    pub fn bounds(&self) -> Option<[i32; 4]> {
        let (x, y, hx, hy) = match *self {
            Footprint::Round { x, y, radius } if radius > 0 => (x, y, radius, radius),
            Footprint::Rect {
                x,
                y,
                length,
                width,
                angle,
            } if length > 0 && width > 0 => {
                let a = f64::from(angle) / TURN * TAU;
                let (l, w) = (f64::from(length) / 2.0, f64::from(width) / 2.0);
                let hx = (l * a.cos()).abs() + (w * a.sin()).abs();
                let hy = (l * a.sin()).abs() + (w * a.cos()).abs();
                // Rounding noise in the sine of a quarter turn must not widen the box.
                let up = |h: f64| (h - 1e-6).ceil() as i32;
                (x, y, up(hx), up(hy))
            }
            _ => return None,
        };
        Some([x - hx, y - hy, x + hx, y + hy])
    }
}

/// The quantum of a rectangular footprint's length and width, centimetres (ADR-0009 §2).
pub const RECT_QUANTUM_CM: i32 = 25;

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
    /// Integer parameters; their meaning is the grammar's ([`hut_params`], [`frame_params`]).
    pub params: [i32; PARAMS],
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
    /// A beam along the head of a wall's posts (frame).
    WallPlate,
    /// A beam across the building from one wall's post to the other's (frame).
    TieBeam,
    /// A beam carrying a floor or loft (frame).
    Joist,
    /// Boards laid on joists (frame).
    Decking,
    /// The beam along the roof's apex (frame).
    Ridge,
    /// Wattle closing the triangle of a gable end (frame).
    GableInfill,
    /// A ladder up to a loft, a storey or a raised floor (frame).
    Ladder,
    /// A post carrying a raised floor (frame).
    FloorPost,
}

impl PartKind {
    /// A stable code (append only).
    pub fn code(self) -> u8 {
        match self {
            PartKind::Posthole => 0,
            PartKind::Post => 1,
            PartKind::Rafter => 2,
            PartKind::Wattle => 3,
            PartKind::Daub => 4,
            PartKind::Thatch => 5,
            PartKind::Floor => 6,
            PartKind::Hearth => 7,
            PartKind::WallPlate => 8,
            PartKind::TieBeam => 9,
            PartKind::Joist => 10,
            PartKind::Decking => 11,
            PartKind::Ridge => 12,
            PartKind::GableInfill => 13,
            PartKind::Ladder => 14,
            PartKind::FloorPost => 15,
        }
    }
}

/// What a space is used for (ADR-0009 §1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpaceUse {
    /// Living and sleeping, with the hearth.
    Living,
    /// Keeping goods.
    Store,
    /// Working at a craft.
    Work,
}

impl SpaceUse {
    /// Every use, in a fixed order (indexes of [`Expansion::floor_by_use`]).
    pub const ALL: [SpaceUse; 3] = [SpaceUse::Living, SpaceUse::Store, SpaceUse::Work];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            SpaceUse::Living => "living",
            SpaceUse::Store => "store",
            SpaceUse::Work => "work",
        }
    }

    /// The use with an authored name.
    pub fn from_name(name: &str) -> Option<SpaceUse> {
        SpaceUse::ALL.into_iter().find(|u| u.name() == name)
    }

    /// Position in [`SpaceUse::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }
}

/// A level of a building.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    /// The first floor, on the ground or raised on posts.
    Ground,
    /// An enclosed storey above it.
    Upper,
    /// A floor at tie-beam height inside the roof, holding goods only.
    Loft,
    /// The roof itself.
    Roof,
}

impl Level {
    /// A stable code (append only), the first part of a group's id.
    pub fn code(self) -> u8 {
        match self {
            Level::Ground => 0,
            Level::Upper => 1,
            Level::Loft => 2,
            Level::Roof => 3,
        }
    }
}

/// A space of a building (ADR-0009 §3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Space {
    /// Its level.
    pub level: Level,
    /// The bays it covers, one bit each (bit `b` for bay `b`; a hut has one bay).
    pub bays: u16,
    /// Net floor area, square metres.
    pub area_m2: f64,
    /// What it is for.
    pub use_: SpaceUse,
}

/// What kind of members a group gathers (ADR-0009 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GroupKind {
    /// The posts of one wall line.
    Posts,
    /// The tie beams across the building.
    TieBeams,
    /// The joists of a loft over one bay.
    LoftJoists,
    /// The joists of an upper storey's floor over one bay.
    FloorJoists,
    /// The joists and boards of a floor raised on posts over one bay.
    RaisedFloor,
    /// Rafters and ridge: the roof's frame over one bay (a hut's: all of it).
    RoofFrame,
    /// The roof's covering (one slope of a frame, all of a hut's).
    Covering,
    /// The wattle and daub of one wall, or a gable.
    Infill,
    /// A floor on the ground.
    Floor,
}

impl GroupKind {
    /// A stable code (append only), part of a group's id.
    pub fn code(self) -> u8 {
        match self {
            GroupKind::Posts => 1,
            GroupKind::TieBeams => 2,
            GroupKind::LoftJoists => 3,
            GroupKind::FloorJoists => 4,
            GroupKind::RaisedFloor => 5,
            GroupKind::RoofFrame => 6,
            GroupKind::Covering => 7,
            GroupKind::Infill => 8,
            GroupKind::Floor => 9,
        }
    }

    /// Its name in words: "posts", "roof frame".
    pub fn name(self) -> &'static str {
        match self {
            GroupKind::Posts => "posts",
            GroupKind::TieBeams => "tie beams",
            GroupKind::LoftJoists => "loft joists",
            GroupKind::FloorJoists => "floor joists",
            GroupKind::RaisedFloor => "raised floor",
            GroupKind::RoofFrame => "roof frame",
            GroupKind::Covering => "covering",
            GroupKind::Infill => "infill",
            GroupKind::Floor => "floor",
        }
    }

    /// Every kind, in code order.
    pub const ALL: [GroupKind; 9] = [
        GroupKind::Posts,
        GroupKind::TieBeams,
        GroupKind::LoftJoists,
        GroupKind::FloorJoists,
        GroupKind::RaisedFloor,
        GroupKind::RoofFrame,
        GroupKind::Covering,
        GroupKind::Infill,
        GroupKind::Floor,
    ];

    /// The kind with stable code `code`.
    pub fn from_code(code: u8) -> Option<GroupKind> {
        GroupKind::ALL.into_iter().find(|k| k.code() == code)
    }

    /// The kind of the group with semantic id `id` ([`group_id`]).
    pub fn of_group(id: u32) -> Option<GroupKind> {
        GroupKind::from_code(((id >> 8) & 0xff) as u8)
    }
}

/// A group of like members that stand or fall together (ADR-0009 §3; research 11-06 §5.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Group {
    /// Stable and semantic: level, bay (0 for none, else the bay plus one), kind and index, a
    /// byte each, so a later bay renumbers nothing (research 11-03 §4.8).
    pub id: u32,
    /// What its members are.
    pub kind: GroupKind,
    /// The stage that puts it in place.
    pub stage: Stage,
    /// Material slot, or `None` for earth.
    pub material: Option<u8>,
    /// Members in it.
    pub count: u32,
    /// A member's section, width and depth, centimetres (a covering or infill: its thickness).
    pub section_cm: [i32; 2],
    /// A member's span or height, centimetres (a covering: its length).
    pub length_cm: i32,
    /// Spacing between members, centimetres (0 for one).
    pub spacing_cm: i32,
    /// The area it carries or covers, square metres.
    pub area_m2: f64,
}

impl Group {
    /// The earth in it, cubic metres, if it is daubed infill: its area times its thickness
    /// (ADR-0010 §2: daub is dug from the ground beside the building); 0 for any other group.
    pub fn daub_m3(&self) -> f64 {
        if self.kind == GroupKind::Infill {
            self.area_m2 * f64::from(self.section_cm[0].max(0)) / 100.0
        } else {
            0.0
        }
    }
}

/// A group's semantic id.
pub fn group_id(level: Level, bay: Option<u8>, kind: GroupKind, index: u8) -> u32 {
    (u32::from(level.code()) << 24)
        | (u32::from(bay.map_or(0, |b| b.saturating_add(1))) << 16)
        | (u32::from(kind.code()) << 8)
        | u32::from(index)
}

/// One piece of a building, in centimetres from the map's north-west corner (z up from the
/// ground).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Part {
    /// Stable within the expansion. A hut's: the stage times 1000 plus a running number; a frame
    /// building's: what it is and where, `level << 24 | (bay or frame line + 1) << 16 | kind << 8
    /// | index` (0 in the second byte for a part of no one bay), the same for the same part of
    /// buildings that differ elsewhere.
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

/// Where goods are kept, the indexes of [`Expansion::storage_kg`] (ADR-0009 §3), best first.
pub mod storage {
    /// On a raised store's floor, off the ground and aired.
    pub const RAISED: usize = 0;
    /// In a loft.
    pub const LOFT: usize = 1;
    /// On a floor on the ground or a storey: a store room's, or what living and working floor
    /// holds besides its use.
    pub const FLOOR: usize = 2;
    /// Number of kinds.
    pub const KINDS: usize = 3;
}

/// What a spec expands into.
#[derive(Clone, Debug, PartialEq)]
pub struct Expansion {
    /// The grammar that produced it.
    pub grammar: Grammar,
    /// The grammar version that produced it.
    pub version: u16,
    /// Every part, in stage order.
    pub parts: Vec<Part>,
    /// The outer face of the walls, centimetres: going round from the door (a hut), or the four
    /// corners (a frame).
    pub outline: Vec<(i32, i32)>,
    /// Radius of the roof's edge (a hut), or of the circle round a frame's roof, centimetres.
    pub roof_radius_cm: i32,
    /// Height of the roof's apex or ridge, centimetres.
    pub apex_cm: i32,
    /// The doorway's middle on the wall line, centimetres, and the direction it faces.
    pub door: (i32, i32, u16),
    /// Width of the doorway, centimetres.
    pub door_width_cm: i32,
    /// One entry per stage, in order.
    pub stages: Vec<StageNeeds>,
    /// Floor area inside the wall lines over every level, square metres.
    pub floor_area_m2: f64,
    /// People it sleeps.
    pub sleeping_places: u32,
    /// The corners of a gabled roof's edge, centimetres (empty for a hut's cone, whose edge is a
    /// circle of `roof_radius_cm`).
    pub roof_outline: Vec<(i32, i32)>,
    /// The ends of a gabled roof's ridge, seen from above, centimetres (`None` for a hut).
    pub ridge: Option<[(i32, i32); 2]>,
    /// Its spaces: each level's bays, floor and use.
    pub spaces: Vec<Space>,
    /// Its component groups, in stage order.
    pub groups: Vec<Group>,
    /// Floor area by use ([`SpaceUse::index`]), square metres; they sum to `floor_area_m2`.
    pub floor_by_use: [f64; 3],
    /// Goods it can hold under its roof by where they are kept ([`storage`]), kilograms.
    pub storage_kg: [f64; storage::KINDS],
    /// Places to work at a craft.
    pub work_places: u32,
}

impl Expansion {
    /// The earth in the daub of the groups `stage` puts in place, or of every stage's when it is
    /// `None`, cubic metres ([`Group::daub_m3`]).
    pub fn daub_m3(&self, stage: Option<Stage>) -> f64 {
        self.groups
            .iter()
            .filter(|g| stage.is_none_or(|s| g.stage == s))
            .map(Group::daub_m3)
            .sum()
    }

    /// Labour of every stage together, person-hours.
    pub fn total_labour_h(&self) -> f64 {
        self.stages.iter().map(|s| s.labour_h).sum()
    }

    /// Kilograms of each material slot over every stage.
    pub fn total_materials_kg(&self) -> Vec<f64> {
        let slots = self
            .stages
            .iter()
            .map(|s| s.materials_kg.len())
            .max()
            .unwrap_or(0);
        let mut out = vec![0.0; slots];
        for s in &self.stages {
            for (o, kg) in out.iter_mut().zip(&s.materials_kg) {
                *o += kg;
            }
        }
        out
    }

    /// Goods it can hold under its roof, kilograms.
    pub fn storage_total_kg(&self) -> f64 {
        self.storage_kg.iter().sum()
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
    /// Goods its floor holds besides living on it, kilograms a square metre (ADR-0009 §3).
    pub store_kg_per_m2: f64,
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
            GrammarError::UnknownVersion(v) => write!(f, "grammar version {v} is unknown"),
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
    } = spec.footprint
    else {
        return Err(GrammarError::OutOfRange(
            "a hut stands on a round footprint".to_owned(),
        ));
    };
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
    let mut e = Expansion {
        grammar: Grammar::Hut,
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
        roof_outline: Vec::new(),
        ridge: None,
        spaces: vec![Space {
            level: Level::Ground,
            bays: 1,
            area_m2: floor_area,
            use_: SpaceUse::Living,
        }],
        groups: Vec::new(),
        floor_by_use: [floor_area, 0.0, 0.0],
        storage_kg: [0.0, 0.0, floor_area * rules.store_kg_per_m2],
        work_places: 0,
    };
    e.groups = hut_groups(&e, rules);
    Ok(e)
}

/// A hut's five component groups (ADR-0009 §3), derived from its version-1 expansion without
/// touching what that expansion is: its posts, its roof's frame (the rafters), its thatch, its
/// walls' wattle and daub and its floor. Each carries or covers the whole roof, wall or floor.
pub fn hut_groups(e: &Expansion, rules: &HutRules) -> Vec<Group> {
    let of = |k: PartKind| e.parts.iter().filter(move |p| p.kind == k);
    let first = |k: PartKind| of(k).next().copied();
    let m2 = 1.0e-4;
    let r = f64::from(e.roof_radius_cm);
    let roof_plan_m2 = std::f64::consts::PI * r * r * m2;
    let (posts, rafters, panels) = (
        of(PartKind::Post).count(),
        of(PartKind::Rafter).count(),
        of(PartKind::Wattle).count(),
    );
    let post = first(PartKind::Post);
    let rafter = first(PartKind::Rafter);
    let wattle = first(PartKind::Wattle);
    let thatch = first(PartKind::Thatch);
    let eave = post.map_or(0, |p| p.size[2] - rules.posthole_depth_cm);
    let rafter_len = rafter.map_or(0, |p| p.size[0]);
    let roof_m2 = thatch.map_or(0.0, |t| {
        let pitch = f64::from(t.tilt) / 100.0 * TAU / 360.0;
        roof_plan_m2 / pitch.cos()
    });
    let panel_cm = wattle.map_or(0, |p| p.size[0]);
    let spacing = panel_cm + rules.post_diameter_cm;
    let wall_m2 = f64::from(panel_cm) * f64::from(eave) * panels as f64 * m2;
    let group =
        |level, kind, stage, material, count: usize, section, length, spacing, area_m2| Group {
            id: group_id(level, None, kind, 0),
            kind,
            stage,
            material,
            count: count as u32,
            section_cm: section,
            length_cm: length,
            spacing_cm: spacing,
            area_m2,
        };
    let d = rules.post_diameter_cm;
    vec![
        group(
            Level::Ground,
            GroupKind::Posts,
            Stage::Frame,
            Some(hut_materials::TIMBER),
            posts,
            [d, d],
            eave,
            spacing,
            roof_plan_m2,
        ),
        group(
            Level::Roof,
            GroupKind::RoofFrame,
            Stage::Frame,
            Some(hut_materials::TIMBER),
            rafters,
            rafter.map_or([0, 0], |p| [p.size[1], p.size[2]]),
            rafter_len,
            spacing,
            roof_m2,
        ),
        group(
            Level::Ground,
            GroupKind::Infill,
            Stage::Walls,
            Some(hut_materials::WATTLE),
            panels,
            [rules.wall_thickness_cm, rules.wall_thickness_cm],
            eave,
            spacing,
            wall_m2,
        ),
        group(
            Level::Roof,
            GroupKind::Covering,
            Stage::Roof,
            Some(hut_materials::THATCH),
            1,
            [rules.thatch_thickness_cm, rules.thatch_thickness_cm],
            rafter_len,
            0,
            roof_m2,
        ),
        group(
            Level::Ground,
            GroupKind::Floor,
            Stage::Finish,
            None,
            1,
            [0, 0],
            0,
            0,
            e.floor_area_m2,
        ),
    ]
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
    // What grammar v2 added is covered for frames only, so the frozen hut's hash is unchanged
    // (ADR-0009 §1).
    if e.grammar != Grammar::Hut {
        let hundredths = |v: f64| ((v * 100.0).round() as i64).to_le_bytes();
        eat(e.grammar.name().as_bytes());
        for (x, y) in e.roof_outline.iter().chain(e.ridge.iter().flatten()) {
            eat(&x.to_le_bytes());
            eat(&y.to_le_bytes());
        }
        for s in &e.spaces {
            eat(&[s.level.code(), s.use_.index() as u8]);
            eat(&s.bays.to_le_bytes());
            eat(&hundredths(s.area_m2));
        }
        for g in &e.groups {
            eat(&g.id.to_le_bytes());
            eat(&[g.kind.code(), g.stage.index() as u8]);
            eat(&[g.material.unwrap_or(u8::MAX)]);
            eat(&g.count.to_le_bytes());
            for v in g.section_cm.iter().chain([&g.length_cm, &g.spacing_cm]) {
                eat(&v.to_le_bytes());
            }
            eat(&hundredths(g.area_m2));
        }
        for v in e.floor_by_use.iter().chain(&e.storage_kg) {
            eat(&hundredths(*v));
        }
        eat(&e.work_places.to_le_bytes());
    }
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
            store_kg_per_m2: 60.0,
        }
    }

    fn spec(radius: i32) -> BuildingSpec {
        let mut params = [0; PARAMS];
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

    #[test]
    fn a_hut_has_five_groups_a_living_space_and_room_for_goods() {
        let r = rules();
        let e = expand_hut(&spec(300), &r).expect("expands");
        assert_eq!(e.grammar, Grammar::Hut);
        let kinds: Vec<GroupKind> = e.groups.iter().map(|g| g.kind).collect();
        assert_eq!(
            kinds,
            [
                GroupKind::Posts,
                GroupKind::RoofFrame,
                GroupKind::Infill,
                GroupKind::Covering,
                GroupKind::Floor
            ]
        );
        let ids: std::collections::HashSet<u32> = e.groups.iter().map(|g| g.id).collect();
        assert_eq!(ids.len(), 5);
        let posts = &e.groups[0];
        assert_eq!(
            (posts.count, posts.length_cm, posts.section_cm),
            (24, 150, [15, 15])
        );
        assert_eq!(
            e.groups[2].count, 23,
            "a panel between each pair of posts but the door's"
        );
        // The thatch covers the cone: π (3.5 m)² over cos 45°.
        let cone = std::f64::consts::PI * 3.5 * 3.5 / (TAU / 8.0).cos();
        assert!(
            (e.groups[3].area_m2 - cone).abs() < 0.01,
            "{}",
            e.groups[3].area_m2
        );
        assert!((e.groups[4].area_m2 - e.floor_area_m2).abs() < 1e-9);
        // The daub is the infill's area at the wall's thickness, all put in place with the walls.
        let infill = &e.groups[2];
        let daub = infill.area_m2 * f64::from(r.wall_thickness_cm) / 100.0;
        assert!(daub > 1.0, "{daub}");
        assert!((infill.daub_m3() - daub).abs() < 1e-9);
        assert_eq!(e.groups[0].daub_m3(), 0.0);
        assert!((e.daub_m3(None) - daub).abs() < 1e-9);
        assert!((e.daub_m3(Some(Stage::Walls)) - daub).abs() < 1e-9);
        assert_eq!(e.daub_m3(Some(Stage::Roof)), 0.0);
        assert_eq!(e.spaces.len(), 1);
        assert_eq!(e.spaces[0].use_, SpaceUse::Living);
        assert_eq!(e.floor_by_use, [e.floor_area_m2, 0.0, 0.0]);
        assert!((e.storage_kg[storage::FLOOR] - e.floor_area_m2 * 60.0).abs() < 1e-9);
        assert_eq!(e.work_places, 0);
        assert!(e.roof_outline.is_empty() && e.ridge.is_none());
        // A hut's groups are derived after the fact and leave its golden hash alone.
        let mut bare = e.clone();
        bare.groups.clear();
        bare.spaces.clear();
        bare.storage_kg = [0.0; storage::KINDS];
        assert_eq!(expansion_hash(&bare), GOLDEN_300);
    }

    #[test]
    fn expand_dispatches_on_the_program_s_grammar() {
        let r = rules();
        let hut = ProgramRules::Hut(r.clone());
        assert_eq!(hut.grammar(), Grammar::Hut);
        assert_eq!(expand(&spec(300), &hut), expand_hut(&spec(300), &r));
        assert_eq!(Grammar::from_name("frame"), Some(Grammar::Frame));
        assert_eq!(Grammar::from_name("tent"), None);
        // A round spec under frame rules, or a hut of an unknown version, is refused.
        let mut v2 = spec(300);
        v2.version = 2;
        assert_eq!(expand(&v2, &hut), Err(GrammarError::UnknownVersion(2)));
    }
}
