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
//!
//! Version 3 (ADR-0031 §1) has room for twelve bays and four storeys and names its own
//! parameters (roof and post systems, wall ties, each storey's wall thickness and openings, a
//! stair). Its mass walls are laid storey by storey, each storey's at its own thickness, thinning
//! upward and grouped by storey. It may rise three or four storeys, each above the first floored
//! on joists like the second; three or more need a stair, a flight from each storey to the next
//! in place of the ladder, beside the well of the flight below, and each storey gives up the
//! flights' floor. Each storey may leave a share of its wall open, a window half the storey high
//! in every piece of wall the door does not split; an opening costs no wall and shortens the
//! length a mass wall bears on. Its roof may be trussed: two principal rafters and a king post on
//! the tie beam at each frame line, carrying a purlin along each slope halfway up on which the
//! rafters rest, so each spans half its slope. Its posts may be storeyed, each storey's standing
//! on the frame below rather than running through every storey; its mass walls may be tied, a
//! timber bond along every wall at each storey's head. With its own parameters all 0, a frame of
//! wattle walls builds what version 2 builds.

use std::f64::consts::TAU;

use crate::{
    BuildingSpec, Expansion, Footprint, Grammar, GrammarError, Group, GroupKind, Level, Part,
    PartKind, RECT_QUANTUM_CM, Space, SpaceUse, Stage, StageNeeds, TURN, cm, group_id, within,
};

/// The newest version of the frame grammar (ADR-0031 §1). Versions 1 and 2 are frozen: what they
/// expand to is pinned by their goldens.
pub const FRAME_VERSION: u16 = 3;
/// Most bays a frame may have (version 3; versions 1 and 2 have at most eight).
pub const MAX_BAYS: i32 = 12;
/// Most storeys a frame may have (version 3, ADR-0031 §1; versions 1 and 2 have at most two:
/// research 11-05 §2.2, one to three in timber).
pub const MAX_STOREYS: u8 = 4;

/// The frame version builders design under: version 2 until they weigh what version 3 adds,
/// storeys above two, ties, trusses and walls by storey (M8d slice CZ, ADR-0031 §2). Content's
/// programs are held to what it allows.
pub const FRAME_DESIGN_VERSION: u16 = 2;

/// The most bays and storeys a frame of `version` may have.
pub fn frame_limits(version: u16) -> (i32, u8) {
    if version >= 3 {
        (MAX_BAYS, MAX_STOREYS)
    } else {
        (8, 2)
    }
}

/// Most component groups an expansion may have (research 11-06 §5.1: 8 to 32).
pub const MAX_GROUPS: usize = 32;
/// Most component groups a version 3 expansion may have: a hall of twelve bays and four storeys
/// keeps a group for each floor's joists in each bay.
pub const MAX_GROUPS_V3: usize = 96;

/// The most component groups an expansion of frame `version` may have.
pub fn max_groups(version: u16) -> usize {
    if version >= 3 {
        MAX_GROUPS_V3
    } else {
        MAX_GROUPS
    }
}
/// Most rafters a slope has in a bay, or joists a floor has in a bay: indexes fit a byte.
const MAX_MEMBERS_PER_BAY: usize = 64;
/// Rise of a ladder from the horizontal, hundredths of a degree.
pub const LADDER_TILT_CENTIDEG: i32 = 7000;
/// Width of a ladder, centimetres.
pub const LADDER_WIDTH_CM: i32 = 50;
/// Thickness of a ladder, centimetres.
const LADDER_DEPTH_CM: f64 = 8.0;
/// Most of a storey's wall an opening may leave open, thousandths: a window half the storey high
/// and at most nine tenths of its panel wide.
pub const MAX_OPENING_PERMILLE: i32 = 450;
/// Rise of a stair from the horizontal, hundredths of a degree.
pub const STAIR_TILT_CENTIDEG: i32 = 3800;
/// Width of a stair, centimetres.
pub const STAIR_WIDTH_CM: i32 = 80;
/// Depth of a stair's strings and treads, centimetres.
const STAIR_DEPTH_CM: f64 = 20.0;
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

    // Version 3 (ADR-0031 §1). In versions 1 and 2 every one of these is 0.

    /// The roof system: 0, rafters from the eaves to the ridge; 1, a truss at each frame line
    /// carrying purlins.
    pub const ROOF_SYSTEM: usize = 16;
    /// The post system: 0, posts through every storey to the plates; 1, each storey's posts
    /// standing on the frame below.
    pub const POST_SYSTEM: usize = 17;
    /// Wall ties: 0, none; 1, floors and roof framed into the walls at each storey.
    pub const TIES: usize = 18;
    /// The first of four: each storey's mass-wall thickness, centimetres, the ground storey's
    /// first; 0 takes [`WALL_CM`].
    pub const STOREY_WALL_CM: usize = 19;
    /// The first of four: the share of each storey's wall left open, thousandths.
    pub const OPENINGS: usize = 23;
    /// A stair: 0, none; 1, a stair from each storey to the next.
    pub const STAIR: usize = 27;
    /// The first parameter no version has yet; it and those after it are 0.
    pub const SPARE: usize = 28;

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

/// Square metres in a square centimetre.
const M2: f64 = 1.0e-4;

/// A spec checked against its program's rules, and its dimensions: centimetres, with `u` along
/// the length, `v` across it and `z` up from the ground.
struct Layout<'r> {
    rules: &'r FrameRules,
    /// Centre, and the cosine and sine of the length's direction.
    centre: (f64, f64),
    turn: (f64, f64),
    /// The length's direction, turn units.
    angle: u16,
    /// Length, width, a bay's length.
    l: f64,
    w: f64,
    b: f64,
    /// Bays and storeys.
    n: usize,
    storeys: i32,
    /// Storey height, floor raise, overhang.
    h: f64,
    r: f64,
    o: f64,
    /// Post, joist and wall sizes; beam, rafter and decking sizes; posthole depth; door width.
    pd: f64,
    jst: f64,
    wt: f64,
    beam: f64,
    raf: f64,
    dk: f64,
    depth: f64,
    door_w: f64,
    /// Roof pitch, hundredths of a degree and radians.
    pitch_cd: i32,
    pitch: f64,
    /// Heights: the first floor, the plates, the apex, the eaves' edge.
    z_g: f64,
    z_p: f64,
    z_a: f64,
    z_e: f64,
    /// A raised floor's posts' height above the ground.
    floor_post_h: f64,
    /// A rafter's length.
    slope: f64,
    rafters_per_bay: usize,
    joists_per_bay: usize,
    joist_spacing: f64,
    /// The lofted bays, their mask, and the mask of every bay.
    lofts: Vec<usize>,
    loft_mask: i32,
    all_bays: i32,
    raised: bool,
    door_side: i32,
    door_bay: i32,
    /// Floors on joists: a raised floor (with its middle rail) and each upper storey's, with the
    /// height of their surface.
    floors: Vec<(Level, f64, bool)>,
    /// Whether a stair rises from each storey to the next (version 3) in place of a ladder.
    stair: bool,
    /// The share of each storey's wall left open (version 3), a fraction.
    openings: Vec<f64>,
    /// Whether a truss stands at each frame line carrying purlins (version 3).
    truss: bool,
    /// Whether each storey's posts stand on the frame below, not through every storey
    /// (version 3).
    storeyed: bool,
    /// Whether a timber tie binds the mass walls at each storey's head (version 3).
    ties: bool,
    wall_kind: i32,
    footing: i32,
    tile: bool,
    /// The grammar version it was designed under.
    version: u16,
    /// Each storey's wall thickness, centimetres: all `wt` but in version 3's mass walls, which
    /// may thin upward (ADR-0031 §1).
    storey_wt: Vec<f64>,
}

impl<'r> Layout<'r> {
    /// Checks `spec` against `rules` (ADR-0009 §2) and lays it out.
    fn new(spec: &BuildingSpec, rules: &'r FrameRules) -> Result<Layout<'r>, GrammarError> {
        use frame_params as fp;
        if !(1..=FRAME_VERSION).contains(&spec.version) {
            return Err(GrammarError::UnknownVersion(spec.version));
        }
        let (most_bays, most_storeys) = frame_limits(spec.version);
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
            ("posthole depth", rules.posthole_depth_cm),
            ("thatch thickness", rules.thatch_thickness_cm),
            ("hearth", rules.hearth_cm),
        ] {
            if v <= 0 {
                return refuse(format!("the rules' {name} must be positive, not {v}"));
            }
        }

        // What the spec chose, against what the program allows.
        let bays = p[fp::BAYS];
        within("bays", bays, rules.bays)?;
        within("bays", bays, (1, most_bays))?;
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
        within("storeys", storeys, (1, i32::from(most_storeys)))?;
        let eave = p[fp::EAVE_CM];
        within("storey height", eave, rules.eave_cm)?;
        let pitch_cd = p[fp::PITCH_CENTIDEG];
        within("roof pitch", pitch_cd, rules.pitch_centideg)?;
        within("roof pitch", pitch_cd, (1, 8_999))?;
        let post = p[fp::POST_CM];
        within("post diameter", post, rules.post_cm)?;
        if bay - post <= 0 || width - post <= 0 {
            return refuse(format!("posts {post} cm across leave no wall between them"));
        }
        let overhang = p[fp::OVERHANG_CM];
        within("overhang", overhang, rules.overhang_cm)?;
        let wall = p[fp::WALL_CM];
        within("wall thickness", wall, rules.wall_cm)?;
        let wall_kind = p[fp::WALL_KIND];
        let footing = p[fp::FOOTING];
        let tile = spec.materials[usize::from(COVERING)].contains("tile");
        if spec.version == 1 {
            if wall_kind != 0 {
                return refuse(format!(
                    "wall kind {wall_kind} is unknown to frame version 1"
                ));
            }
            if footing != 0 {
                return refuse(format!("footing {footing} is unknown to frame version 1"));
            }
            if tile {
                return refuse("tile covering is unknown to frame version 1");
            }
        } else {
            if !(0..=3).contains(&wall_kind) {
                return refuse(format!(
                    "wall kind {wall_kind} is unknown to frame version 2"
                ));
            }
            if !(0..=1).contains(&footing) {
                return refuse(format!("footing {footing} is unknown to frame version 2"));
            }
            if wall_kind > 0 && footing != 1 {
                return refuse(format!(
                    "mass wall kind {wall_kind} requires a stone footing (footing 1)"
                ));
            }
        }
        if p[fp::STYLE..fp::STYLE + fp::STYLE_TRAITS]
            .iter()
            .any(|&t| t != 0)
        {
            return refuse("no frame version lays out style traits yet");
        }
        // Version 3's parameters: none before it, and in it only those laid out so far.
        let laid_out = |i: usize| {
            spec.version >= 3
                && ((fp::STOREY_WALL_CM..fp::OPENINGS + 4).contains(&i)
                    || i == fp::STAIR
                    || i == fp::ROOF_SYSTEM
                    || i == fp::POST_SYSTEM
                    || i == fp::TIES)
        };
        if let Some(i) = (fp::ROOF_SYSTEM..crate::PARAMS).find(|&i| p[i] != 0 && !laid_out(i)) {
            return refuse(if spec.version < 3 {
                format!("parameter {i} is unknown to frame version {}", spec.version)
            } else {
                format!("parameter {i} is not laid out by frame version 3 yet")
            });
        }
        // Each storey's mass wall (version 3): its own thickness, never thicker than the one below.
        let mut storey_wt = vec![f64::from(wall); storeys as usize];
        for t in 0..4 {
            let v = p[fp::STOREY_WALL_CM + t];
            if v == 0 {
                continue;
            }
            if t >= storey_wt.len() {
                return refuse(format!(
                    "storey {} has a wall thickness and no storey",
                    t + 1
                ));
            }
            if wall_kind == 0 {
                return refuse(
                    "wattle walls have one thickness; storeys' thicknesses are for mass walls",
                );
            }
            within("storey wall thickness", v, rules.wall_cm)?;
            storey_wt[t] = f64::from(v);
        }
        if storey_wt.windows(2).any(|w| w[1] > w[0]) {
            return refuse("a mass wall may thin upward, not thicken");
        }
        // Each storey's openings (version 3): a share of its wall, on the bay rhythm.
        let mut openings = vec![0.0; storeys as usize];
        for t in 0..4 {
            let v = p[fp::OPENINGS + t];
            if v == 0 {
                continue;
            }
            if t >= openings.len() {
                return refuse(format!(
                    "storey {} has openings (parameter {}) and no storey",
                    t + 1,
                    fp::OPENINGS + t
                ));
            }
            within("openings", v, (0, MAX_OPENING_PERMILLE))?;
            openings[t] = f64::from(v) / 1000.0;
        }
        let roof_system = p[fp::ROOF_SYSTEM];
        if !(0..=1).contains(&roof_system) {
            return refuse(format!(
                "roof system {roof_system} (parameter {}) is unknown to frame version {}",
                fp::ROOF_SYSTEM,
                spec.version
            ));
        }
        let post_system = p[fp::POST_SYSTEM];
        if !(0..=1).contains(&post_system) {
            return refuse(format!(
                "post system {post_system} (parameter {}) is unknown to frame version {}",
                fp::POST_SYSTEM,
                spec.version
            ));
        }
        if post_system == 1 && wall_kind != 0 {
            return refuse(format!(
                "storeyed posts (parameter {}) stand in a timber frame; mass walls have none",
                fp::POST_SYSTEM
            ));
        }
        if post_system == 1 && storeys == 1 {
            return refuse(format!(
                "storeyed posts (parameter {}) need a storey above the first",
                fp::POST_SYSTEM
            ));
        }
        let ties = p[fp::TIES];
        if !(0..=1).contains(&ties) {
            return refuse(format!(
                "wall ties {ties} (parameter {}) are unknown to frame version {}",
                fp::TIES,
                spec.version
            ));
        }
        if ties == 1 && wall_kind == 0 {
            return refuse(format!(
                "wall ties (parameter {}) bind mass walls; a timber frame is tied by its joints",
                fp::TIES
            ));
        }
        let stair = p[fp::STAIR];
        if !(0..=1).contains(&stair) {
            return refuse(format!(
                "stair {stair} (parameter {}) is unknown to frame version {}",
                fp::STAIR,
                spec.version
            ));
        }
        if storeys > 2 && stair == 0 {
            return refuse(format!(
                "{storeys} storeys need a stair (parameter {})",
                fp::STAIR
            ));
        }
        if storeys == 1 && stair != 0 {
            return refuse(format!(
                "a stair (parameter {}) needs a storey above the first",
                fp::STAIR
            ));
        }
        // A program with no raised floor allows only 0; one whose floor may stand on the ground
        // or be raised allows 0 among the rest.
        let raise = p[fp::FLOOR_RAISE_CM];
        within("floor raise", raise, rules.floor_raise_cm)?;
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

        let (l, w, b) = (f64::from(length), f64::from(width), f64::from(bay));
        let (h, r, o) = (f64::from(eave), f64::from(raise), f64::from(overhang));
        let (jst, dk) = (f64::from(joist), f64::from(rules.decking_cm));
        let beam = f64::from(rules.beam_cm);
        let pitch = f64::from(pitch_cd) / 100.0 * TAU / 360.0;
        let z_p = r + h * f64::from(storeys);
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
        let rafters_per_bay = ((b / f64::from(rules.rafter_spacing_cm)).round() as usize).max(1);
        let joists_per_bay =
            (((b / f64::from(rules.joist_spacing_cm)).round() as usize).max(2) - 1).max(1);
        if rafters_per_bay > MAX_MEMBERS_PER_BAY / 2 || joists_per_bay > MAX_MEMBERS_PER_BAY {
            return refuse("the rules space rafters or joists too closely");
        }
        let a = f64::from(angle) / TURN * TAU;
        let layout = Layout {
            rules,
            centre: (f64::from(cx), f64::from(cy)),
            turn: (a.cos(), a.sin()),
            angle,
            l,
            w,
            b,
            n,
            storeys,
            h,
            r,
            o,
            pd: f64::from(post),
            jst,
            wt: f64::from(wall),
            beam,
            raf: f64::from(rules.rafter_cm),
            dk,
            depth: f64::from(rules.posthole_depth_cm),
            door_w: f64::from(rules.door_cm),
            pitch_cd,
            pitch,
            z_g: r,
            z_p,
            z_a: z_p + w / 2.0 * pitch.tan(),
            z_e,
            floor_post_h,
            slope: (w / 2.0 + o) / pitch.cos(),
            rafters_per_bay,
            joists_per_bay,
            joist_spacing: b / (joists_per_bay + 1) as f64,
            lofts,
            loft_mask,
            all_bays,
            raised,
            door_side,
            door_bay,
            floors: raised
                .then_some((Level::Ground, r, true))
                .into_iter()
                .chain((1..storeys).map(|t| (storey_level(t), r + h * f64::from(t), false)))
                .collect(),
            stair: stair != 0,
            openings,
            truss: roof_system == 1,
            storeyed: post_system == 1,
            ties: ties == 1,
            wall_kind,
            footing,
            tile,
            version: spec.version,
            storey_wt,
        };
        if raised && layout.wt / 2.0 + layout.ladder_run(r) > o {
            return refuse(format!(
                "the ladder up to a floor raised {raise} cm would stand beyond the roof's edge"
            ));
        }
        if layout.stair {
            let run = layout.stair_run();
            if run > l - layout.pd {
                return refuse(format!(
                    "a stair rising {eave} cm runs {run:.0} cm, more than the building's length"
                ));
            }
            if f64::from(2 * STAIR_WIDTH_CM) > w - layout.pd {
                return refuse(format!(
                    "two flights of stairs {STAIR_WIDTH_CM} cm wide do not fit a width of {width} cm"
                ));
            }
        }
        Ok(layout)
    }

    /// The point `(u, v)` on the map, centimetres.
    fn at(&self, u: f64, v: f64) -> (f64, f64) {
        let ((cx, cy), (ca, sa)) = (self.centre, self.turn);
        (cx + u * ca - v * sa, cy + u * sa + v * ca)
    }

    /// `u` of frame line `k`, the start of bay `k`.
    fn u_line(&self, k: usize) -> f64 {
        -self.l / 2.0 + self.b * k as f64
    }

    /// Directions: along the length, across it (`+v`), back along it, back across it (`-v`).
    fn along(&self) -> u16 {
        self.angle
    }

    fn across(&self) -> u16 {
        turned(self.angle, 1)
    }

    fn back(&self) -> u16 {
        turned(self.angle, 2)
    }

    fn back_across(&self) -> u16 {
        turned(self.angle, 3)
    }

    /// The long walls (side, `v`) and the ends (side, `u`).
    fn long_walls(&self) -> [(i32, f64); 2] {
        [(SIDE_RIGHT, self.w / 2.0), (SIDE_LEFT, -self.w / 2.0)]
    }

    fn end_walls(&self) -> [(i32, f64); 2] {
        [(SIDE_BACK, -self.l / 2.0), (SIDE_FRONT, self.l / 2.0)]
    }

    /// How far a ladder reaches out to rise `rise`, and its length.
    fn ladder_run(&self, rise: f64) -> f64 {
        rise / ladder_tilt().tan()
    }

    fn ladder_len(&self, rise: f64) -> f64 {
        rise / ladder_tilt().sin()
    }

    /// How far a flight of stairs reaches along the length to rise a storey, and its length.
    fn stair_run(&self) -> f64 {
        self.h / stair_tilt().tan()
    }

    fn stair_len(&self) -> f64 {
        self.h / stair_tilt().sin()
    }

    /// Floor a storey gives up to the stair, square metres: the flight rising from it and the
    /// well it is reached through, each a flight's plan.
    fn stair_m2(&self, t: i32) -> f64 {
        if !self.stair {
            return 0.0;
        }
        let flights = usize::from(t > 0) + usize::from(t + 1 < self.storeys);
        flights as f64 * self.stair_run() * f64::from(STAIR_WIDTH_CM) * M2
    }

    /// The width of the window in a wall piece `piece` wide on storey `t`, centimetres: none in
    /// a piece the door splits. A window is half the storey high, so it leaves its storey's share
    /// of the piece open.
    fn window(&self, t: i32, piece: f64, door_here: bool) -> f64 {
        if door_here {
            0.0
        } else {
            2.0 * self.openings[t as usize] * piece
        }
    }

    /// Whether the door splits the piece of wall `side` in bay `j` (0 on an end wall) on storey
    /// `t`.
    fn door_here(&self, t: i32, side: i32, j: usize) -> bool {
        t == 0 && side == self.door_side && (side >= SIDE_BACK || j as i32 == self.door_bay)
    }

    /// The windows' width along long wall `side` on storey `t`, centimetres, for panels
    /// `panel` wide.
    fn windows_along(&self, t: i32, side: i32, panel: f64) -> f64 {
        (0..self.n)
            .map(|j| self.window(t, panel, self.door_here(t, side, j)))
            .sum()
    }

    /// A truss's principal rafter, from a plate to the apex, and its king post, centimetres.
    fn principal_len(&self) -> f64 {
        self.w / 2.0 / self.pitch.cos()
    }

    fn king_post_h(&self) -> f64 {
        self.z_a - self.z_p
    }

    /// What storey `t` is for.
    fn use_of(&self, t: i32) -> SpaceUse {
        if t == 0 {
            self.rules.ground_use
        } else {
            self.rules.upper_use
        }
    }

    /// Every floor on joists by bay: (level, bay, floor surface height); raised and upper floors
    /// cover every bay, lofts their own.
    fn decks(&self) -> Vec<(Level, usize, f64)> {
        let mut decks: Vec<(Level, usize, f64)> = Vec::new();
        for &(level, z_floor, _) in &self.floors {
            decks.extend((0..self.n).map(|j| (level, j, z_floor)));
        }
        let loft_floor = self.z_p + self.jst + self.dk;
        decks.extend(self.lofts.iter().map(|&j| (Level::Loft, j, loft_floor)));
        decks
    }

    /// The door's middle `(u, v)` on its wall line, and the direction it faces.
    fn door(&self) -> (f64, f64, u16) {
        let mid = self.u_line(self.door_bay as usize) + self.b / 2.0;
        match self.door_side {
            SIDE_RIGHT => (mid, self.w / 2.0, self.across()),
            SIDE_LEFT => (mid, -self.w / 2.0, self.back_across()),
            SIDE_BACK => (-self.l / 2.0, 0.0, self.back()),
            _ => (self.l / 2.0, 0.0, self.along()),
        }
    }
}

/// A ladder's rise from the horizontal, radians.
fn ladder_tilt() -> f64 {
    f64::from(LADDER_TILT_CENTIDEG) / 100.0 * TAU / 360.0
}

/// A stair's rise from the horizontal, radians.
fn stair_tilt() -> f64 {
    f64::from(STAIR_TILT_CENTIDEG) / 100.0 * TAU / 360.0
}

/// The slopes: index and which side of the ridge (`+v`, `-v`).
const SLOPES: [(usize, f64); 2] = [(0, 1.0), (1, -1.0)];

/// The level of storey `t` (0 the first; a layout has at most [`MAX_STOREYS`]).
fn storey_level(t: i32) -> Level {
    Level::storey(t as usize).unwrap_or(Level::Fourth)
}

/// The parts of a building as they are laid out, with their semantic ids.
struct Parts<'l, 'r> {
    layout: &'l Layout<'r>,
    parts: Vec<Part>,
}

/// A part's place in its building: level, bay or frame line, kind and index.
type Slot = (Level, Option<usize>, PartKind, usize);

impl Parts<'_, '_> {
    /// Adds a part: its slot, stage, centre `(u, v, z)`, size along its own axes, direction and
    /// tilt, and material.
    fn push(
        &mut self,
        (level, slot, kind, index): Slot,
        stage: Stage,
        (u, v, z): (f64, f64, f64),
        size: [f64; 3],
        (angle, tilt): (u16, i32),
        material: Option<u8>,
    ) {
        let (x, y) = self.layout.at(u, v);
        self.parts.push(Part {
            id: part_id(level, slot, kind, index),
            kind,
            stage,
            at: [cm(x), cm(y), cm(z)],
            size: size.map(cm),
            angle,
            tilt,
            material,
        });
    }

    /// A window `width` wide in the wall piece at `slot` (level, bay, index), half its storey high
    /// and centred in it, through a wall `thickness` thick facing `angle`; none when `width` is 0.
    fn window(
        &mut self,
        (level, bay, index): (Level, Option<usize>, usize),
        width: f64,
        at: (f64, f64, f64),
        thickness: f64,
        angle: u16,
    ) {
        if width > 0.0 {
            let h = self.layout.h;
            self.push(
                (level, bay, PartKind::Window, index),
                Stage::Walls,
                at,
                [width, thickness, h / 2.0],
                (angle, 0),
                None,
            );
        }
    }

    /// A timber member lying flat.
    fn member(
        &mut self,
        slot: Slot,
        stage: Stage,
        at: (f64, f64, f64),
        size: [f64; 3],
        angle: u16,
    ) {
        self.push(slot, stage, at, size, (angle, 0), Some(TIMBER));
    }
}

/// Foundation: postholes or stone footings, and a raised floor's posts down the middle.
fn foundation(p: &mut Parts) {
    let l = p.layout;
    let hole = [l.pd + POSTHOLE_SLACK_CM, l.pd + POSTHOLE_SLACK_CM, l.depth];
    if l.footing == 1 {
        for j in 0..l.n {
            for (side, v) in l.long_walls() {
                let slot = (
                    Level::Ground,
                    Some(j),
                    PartKind::StoneFooting,
                    side as usize,
                );
                p.push(
                    slot,
                    Stage::Foundation,
                    (l.u_line(j) + l.b / 2.0, v, -l.depth / 2.0),
                    [l.b, l.wt, l.depth],
                    (l.along(), 0),
                    None,
                );
            }
        }
        for (side, u) in l.end_walls() {
            let slot = (Level::Ground, None, PartKind::StoneFooting, side as usize);
            p.push(
                slot,
                Stage::Foundation,
                (u, 0.0, -l.depth / 2.0),
                [l.w, l.wt, l.depth],
                (l.across(), 0),
                None,
            );
        }
    } else {
        for k in 0..=l.n {
            for (side, v) in l.long_walls() {
                let slot = (Level::Ground, Some(k), PartKind::Posthole, side as usize);
                p.push(
                    slot,
                    Stage::Foundation,
                    (l.u_line(k), v, -l.depth / 2.0),
                    hole,
                    (l.along(), 0),
                    None,
                );
            }
        }
    }
    if l.raised {
        for k in 0..=l.n {
            let slot = (Level::Ground, Some(k), PartKind::Posthole, 2);
            p.push(
                slot,
                Stage::Foundation,
                (l.u_line(k), 0.0, -l.depth / 2.0),
                hole,
                (l.along(), 0),
                None,
            );
            p.member(
                (Level::Ground, Some(k), PartKind::FloorPost, 0),
                Stage::Foundation,
                (l.u_line(k), 0.0, (l.floor_post_h - l.depth) / 2.0),
                [l.pd, l.pd, l.floor_post_h + l.depth],
                l.along(),
            );
        }
    }
}

/// Frame: posts through every storey, tie beams across their heads at each frame line, plates
/// along the wall heads a bay at a time; floors on joists; lofts; rafters and the ridge.
fn frame(p: &mut Parts) {
    let l = p.layout;
    for k in 0..=l.n {
        if l.wall_kind == 0 && l.storeyed {
            // Each storey's posts stand on the frame below: the first's in the ground to the head
            // of the first storey, each above it a storey high.
            for t in 0..l.storeys {
                let bottom = if t == 0 {
                    -l.depth
                } else {
                    l.z_g + l.h * f64::from(t)
                };
                let top = l.z_g + l.h * f64::from(t + 1);
                for (side, v) in l.long_walls() {
                    p.member(
                        (storey_level(t), Some(k), PartKind::Post, side as usize),
                        Stage::Frame,
                        (l.u_line(k), v, (bottom + top) / 2.0),
                        [l.pd, l.pd, top - bottom],
                        l.along(),
                    );
                }
            }
        } else if l.wall_kind == 0 {
            for (side, v) in l.long_walls() {
                p.member(
                    (Level::Ground, Some(k), PartKind::Post, side as usize),
                    Stage::Frame,
                    (l.u_line(k), v, (l.z_p - l.depth) / 2.0),
                    [l.pd, l.pd, l.z_p + l.depth],
                    l.along(),
                );
            }
        }
        p.member(
            (Level::Roof, Some(k), PartKind::TieBeam, 0),
            Stage::Frame,
            (l.u_line(k), 0.0, l.z_p + l.beam / 2.0),
            [l.w, l.beam, l.beam],
            l.across(),
        );
    }
    for j in 0..l.n {
        for (side, v) in l.long_walls() {
            p.member(
                (Level::Roof, Some(j), PartKind::WallPlate, side as usize),
                Stage::Frame,
                (l.u_line(j) + l.b / 2.0, v, l.z_p - l.beam / 2.0),
                [l.b, l.beam, l.beam],
                l.along(),
            );
        }
    }
    // Floors on joists: an upper storey's, and a raised floor (with its middle rail).
    for &(level, z_floor, middle) in &l.floors {
        let z_joist = z_floor - l.dk - l.jst / 2.0;
        let z_rail = z_floor - l.dk - l.jst - l.beam / 2.0;
        let rails: Vec<(usize, f64)> = l
            .long_walls()
            .iter()
            .map(|&(side, v)| (side as usize, v))
            .chain(middle.then_some((2, 0.0)))
            .collect();
        for j in 0..l.n {
            for &(index, v) in &rails {
                p.member(
                    (level, Some(j), PartKind::WallPlate, index),
                    Stage::Frame,
                    (l.u_line(j) + l.b / 2.0, v, z_rail),
                    [l.b, l.beam, l.beam],
                    l.along(),
                );
            }
            for i in 0..l.joists_per_bay {
                p.member(
                    (level, Some(j), PartKind::Joist, i),
                    Stage::Frame,
                    (l.u_line(j) + l.joist_spacing * (i + 1) as f64, 0.0, z_joist),
                    [l.w, l.jst, l.jst],
                    l.across(),
                );
            }
        }
        for k in 0..=l.n {
            p.member(
                (level, Some(k), PartKind::TieBeam, 0),
                Stage::Frame,
                (l.u_line(k), 0.0, z_floor - l.dk - l.beam / 2.0),
                [l.w, l.beam, l.beam],
                l.across(),
            );
        }
    }
    // Lofts: joists across the plates over the bays the spec names.
    for &j in &l.lofts {
        for i in 0..l.joists_per_bay {
            p.member(
                (Level::Loft, Some(j), PartKind::Joist, i),
                Stage::Frame,
                (
                    l.u_line(j) + l.joist_spacing * (i + 1) as f64,
                    0.0,
                    l.z_p + l.jst / 2.0,
                ),
                [l.w, l.jst, l.jst],
                l.across(),
            );
        }
    }
    // Rafters from the eaves to the ridge, in pairs along the length (the last pair on the end
    // frame line, which a further bay would begin with), and the ridge a bay at a time.
    for line in 0..=l.n {
        let count = if line < l.n { l.rafters_per_bay } else { 1 };
        for i in 0..count {
            let u = l.u_line(line) + l.b * i as f64 / l.rafters_per_bay as f64;
            for (sl, sign) in SLOPES {
                p.push(
                    (Level::Roof, Some(line), PartKind::Rafter, i * 2 + sl),
                    Stage::Frame,
                    (u, sign * (l.w / 2.0 + l.o) / 2.0, (l.z_e + l.z_a) / 2.0),
                    [l.slope, l.raf, l.raf],
                    (
                        if sign > 0.0 {
                            l.back_across()
                        } else {
                            l.across()
                        },
                        l.pitch_cd,
                    ),
                    Some(TIMBER),
                );
            }
        }
    }
    for j in 0..l.n {
        let u0 = l.u_line(j) - if j == 0 { l.o } else { 0.0 };
        let u1 = l.u_line(j + 1) + if j + 1 == l.n { l.o } else { 0.0 };
        p.member(
            (Level::Roof, Some(j), PartKind::Ridge, 0),
            Stage::Frame,
            ((u0 + u1) / 2.0, 0.0, l.z_a),
            [u1 - u0, l.beam, l.beam],
            l.along(),
        );
    }
    if l.truss {
        // A truss at each frame line: two principal rafters on the tie beam meeting a king post
        // at the apex; and a purlin along each slope halfway up, a bay at a time, on which the
        // rafters rest.
        let z_mid = (l.z_p + l.z_a) / 2.0;
        for k in 0..=l.n {
            for (sl, sign) in SLOPES {
                p.push(
                    (Level::Roof, Some(k), PartKind::Truss, sl),
                    Stage::Frame,
                    (l.u_line(k), sign * l.w / 4.0, z_mid),
                    [l.principal_len(), l.beam, l.beam],
                    (
                        if sign > 0.0 {
                            l.back_across()
                        } else {
                            l.across()
                        },
                        l.pitch_cd,
                    ),
                    Some(TIMBER),
                );
            }
            p.member(
                (Level::Roof, Some(k), PartKind::Truss, 2),
                Stage::Frame,
                (l.u_line(k), 0.0, z_mid),
                [l.beam, l.beam, l.king_post_h()],
                l.along(),
            );
        }
        for j in 0..l.n {
            for (sl, sign) in SLOPES {
                p.member(
                    (Level::Roof, Some(j), PartKind::Purlin, sl),
                    Stage::Frame,
                    (l.u_line(j) + l.b / 2.0, sign * l.w / 4.0, z_mid + l.beam),
                    [l.b, l.beam, l.beam],
                    l.along(),
                );
            }
        }
    }
}

/// The pieces of a wall panel `panel` wide: whole, or split either side of the door, as (index,
/// offset from its middle, width).
fn pieces(panel: f64, door_w: f64, door_here: bool) -> Vec<(usize, f64, f64)> {
    if door_here {
        let side = (panel - door_w) / 2.0;
        vec![
            (0, -(door_w + side) / 2.0, side),
            (1, (door_w + side) / 2.0, side),
        ]
    } else {
        vec![(0, 0.0, panel)]
    }
}

/// Walls: wattle and daub between the posts, or mass walls of rubble, mud brick or fired brick,
/// storey by storey, the door's panel split either side of it; and the gables. The walls' area,
/// square centimetres, and a mass wall's volume, cubic centimetres (each storey at its own
/// thickness).
fn walls(p: &mut Parts) -> (f64, f64) {
    let l = p.layout;
    let mut area = 0.0;
    // A mass wall's volume, cubic centimetres, storey by storey.
    let mut volume = 0.0;
    if l.wall_kind == 0 {
        for t in 0..l.storeys {
            let level = storey_level(t);
            let zc = l.z_g + l.h * f64::from(t) + l.h / 2.0;
            for kind in [PartKind::Wattle, PartKind::Daub] {
                let (thickness, material) = match kind {
                    PartKind::Wattle => (l.wt / 3.0, Some(WATTLE)),
                    _ => (l.wt, None),
                };
                for j in 0..l.n {
                    for (side, v) in l.long_walls() {
                        let door_here = l.door_here(t, side, j);
                        for (half, du, pw) in pieces(l.b - l.pd, l.door_w, door_here) {
                            let open = l.window(t, pw, door_here);
                            if kind == PartKind::Wattle {
                                area += pw * l.h - open * l.h / 2.0;
                                p.window(
                                    (level, Some(j), side as usize * 2 + half),
                                    open,
                                    (l.u_line(j) + l.b / 2.0 + du, v, zc),
                                    l.wt,
                                    l.along(),
                                );
                            }
                            p.push(
                                (level, Some(j), kind, side as usize * 2 + half),
                                Stage::Walls,
                                (l.u_line(j) + l.b / 2.0 + du, v, zc),
                                [pw, thickness, l.h],
                                (l.along(), 0),
                                material,
                            );
                        }
                    }
                }
                for (side, u) in l.end_walls() {
                    let door_here = l.door_here(t, side, 0);
                    for (half, dv, pw) in pieces(l.w - l.pd, l.door_w, door_here) {
                        let open = l.window(t, pw, door_here);
                        if kind == PartKind::Wattle {
                            area += pw * l.h - open * l.h / 2.0;
                            p.window(
                                (level, None, side as usize * 2 + half),
                                open,
                                (u, dv, zc),
                                l.wt,
                                l.across(),
                            );
                        }
                        p.push(
                            (level, None, kind, side as usize * 2 + half),
                            Stage::Walls,
                            (u, dv, zc),
                            [pw, thickness, l.h],
                            (l.across(), 0),
                            material,
                        );
                    }
                }
            }
        }
        let gable_h = l.z_a - l.z_p;
        for (side, u) in l.end_walls() {
            area += (l.w - l.pd) * gable_h / 2.0;
            p.push(
                (Level::Roof, None, PartKind::GableInfill, side as usize),
                Stage::Walls,
                (u, 0.0, l.z_p + gable_h / 3.0),
                [l.w - l.pd, l.wt, gable_h],
                (l.across(), 0),
                Some(WATTLE),
            );
        }
    } else {
        for t in 0..l.storeys {
            let level = storey_level(t);
            let zc = l.z_g + l.h * f64::from(t) + l.h / 2.0;
            let wt = l.storey_wt[t as usize];
            for j in 0..l.n {
                for (side, v) in l.long_walls() {
                    let door_here = l.door_here(t, side, j);
                    for (half, du, pw) in pieces(l.b, l.door_w, door_here) {
                        let open = l.window(t, pw, door_here);
                        let at = (l.u_line(j) + l.b / 2.0 + du, v, zc);
                        area += pw * l.h - open * l.h / 2.0;
                        volume += (pw * l.h - open * l.h / 2.0) * wt;
                        p.window(
                            (level, Some(j), side as usize * 2 + half),
                            open,
                            at,
                            wt,
                            l.along(),
                        );
                        p.push(
                            (level, Some(j), PartKind::MassWall, side as usize * 2 + half),
                            Stage::Walls,
                            (l.u_line(j) + l.b / 2.0 + du, v, zc),
                            [pw, wt, l.h],
                            (l.along(), 0),
                            Some(WATTLE),
                        );
                    }
                }
            }
            for (side, u) in l.end_walls() {
                let door_here = l.door_here(t, side, 0);
                for (half, dv, pw) in pieces(l.w, l.door_w, door_here) {
                    let open = l.window(t, pw, door_here);
                    area += pw * l.h - open * l.h / 2.0;
                    volume += (pw * l.h - open * l.h / 2.0) * wt;
                    p.window(
                        (level, None, side as usize * 2 + half),
                        open,
                        (u, dv, zc),
                        wt,
                        l.across(),
                    );
                    p.push(
                        (level, None, PartKind::MassWall, side as usize * 2 + half),
                        Stage::Walls,
                        (u, dv, zc),
                        [pw, wt, l.h],
                        (l.across(), 0),
                        Some(WATTLE),
                    );
                }
            }
        }
        let gable_h = l.z_a - l.z_p;
        let top_wt = l.storey_wt.last().copied().unwrap_or(l.wt);
        for (side, u) in l.end_walls() {
            area += l.w * gable_h / 2.0;
            volume += l.w * gable_h / 2.0 * top_wt;
            p.push(
                (Level::Roof, None, PartKind::MassWall, side as usize),
                Stage::Walls,
                (u, 0.0, l.z_p + gable_h / 3.0),
                [l.w, top_wt, gable_h],
                (l.across(), 0),
                Some(WATTLE),
            );
        }
    }
    (area, volume)
}

/// Wall ties: a timber bond along each mass wall at the head of every storey, a bay at a time
/// on the long walls and whole across the ends, into which that storey's floor or the roof is
/// framed.
fn ties(p: &mut Parts) {
    let l = p.layout;
    if !l.ties {
        return;
    }
    for t in 0..l.storeys {
        let level = storey_level(t);
        let z = l.z_g + l.h * f64::from(t + 1) - l.beam / 2.0;
        for j in 0..l.n {
            for (side, v) in l.long_walls() {
                p.member(
                    (level, Some(j), PartKind::WallTie, side as usize),
                    Stage::Walls,
                    (l.u_line(j) + l.b / 2.0, v, z),
                    [l.b, l.beam, l.beam],
                    l.along(),
                );
            }
        }
        for (side, u) in l.end_walls() {
            p.member(
                (level, None, PartKind::WallTie, side as usize),
                Stage::Walls,
                (u, 0.0, z),
                [l.w, l.beam, l.beam],
                l.across(),
            );
        }
    }
}

/// Roof: the covering, a slope at a time.
fn roof(p: &mut Parts) {
    let l = p.layout;
    let (kind, thick) = if l.tile {
        (PartKind::Tile, 4.0)
    } else {
        (PartKind::Thatch, f64::from(l.rules.thatch_thickness_cm))
    };
    for (sl, sign) in SLOPES {
        p.push(
            (Level::Roof, None, kind, sl),
            Stage::Roof,
            (0.0, sign * (l.w / 2.0 + l.o) / 2.0, (l.z_e + l.z_a) / 2.0),
            [l.slope, l.l + 2.0 * l.o, thick],
            (
                if sign > 0.0 {
                    l.back_across()
                } else {
                    l.across()
                },
                l.pitch_cd,
            ),
            Some(COVERING),
        );
    }
}

/// Finish: a floor beaten on the ground, or boards on the joists of every floor; the hearth on
/// the lowest living storey; ladders up to a raised floor, an upper storey and a loft, or a stair
/// from each storey to the next. What the ladders and flights take, in ladders: a flight takes a
/// ladder's labour and boards by its boards' area against a ladder's of the same rise.
fn finish(p: &mut Parts) -> f64 {
    let l = p.layout;
    if !l.raised {
        p.push(
            (Level::Ground, None, PartKind::Floor, 0),
            Stage::Finish,
            (0.0, 0.0, 0.0),
            [l.l, l.w, 0.0],
            (l.along(), 0),
            None,
        );
    }
    for (level, j, z_floor) in l.decks() {
        p.push(
            (level, Some(j), PartKind::Decking, 0),
            Stage::Finish,
            (l.u_line(j) + l.b / 2.0, 0.0, z_floor - l.dk / 2.0),
            [l.b, l.w, l.dk],
            (l.along(), 0),
            Some(BOARDS),
        );
    }
    if let Some(t) = (0..l.storeys).find(|&t| l.use_of(t) == SpaceUse::Living) {
        let hearth = f64::from(l.rules.hearth_cm);
        p.push(
            (storey_level(t), None, PartKind::Hearth, 0),
            Stage::Finish,
            (
                l.u_line(l.n / 2) + l.b / 2.0,
                0.0,
                l.z_g + l.h * f64::from(t),
            ),
            [hearth, hearth, HEARTH_HEIGHT_CM],
            (l.along(), 0),
            None,
        );
    }
    let size = |rise: f64| {
        [
            l.ladder_len(rise),
            f64::from(LADDER_WIDTH_CM),
            LADDER_DEPTH_CM,
        ]
    };
    let mut ladders = 0.0;
    if l.raised {
        // Outside the door, its top at the doorway, its foot under the eaves.
        let out = l.wt / 2.0 + l.ladder_run(l.r) / 2.0;
        let mid = l.u_line(l.door_bay as usize) + l.b / 2.0;
        let (u, v, up) = match l.door_side {
            SIDE_RIGHT => (mid, l.w / 2.0 + out, l.back_across()),
            SIDE_LEFT => (mid, -l.w / 2.0 - out, l.across()),
            SIDE_BACK => (-l.l / 2.0 - out, 0.0, l.along()),
            _ => (l.l / 2.0 + out, 0.0, l.back()),
        };
        ladders += 1.0;
        p.push(
            (Level::Ground, None, PartKind::Ladder, 0),
            Stage::Finish,
            (u, v, l.r / 2.0),
            size(l.r),
            (up, LADDER_TILT_CENTIDEG),
            Some(BOARDS),
        );
    }
    if l.stair {
        // Along the left long wall from the back, each flight beside the well of the one below.
        let run = l.stair_run();
        let width = f64::from(STAIR_WIDTH_CM);
        let flight = l.stair_len() * width / (l.ladder_len(l.h) * f64::from(LADDER_WIDTH_CM));
        for t in 0..l.storeys - 1 {
            let v = -l.w / 2.0 + l.pd / 2.0 + width * (f64::from(t % 2) + 0.5);
            ladders += flight;
            p.push(
                (storey_level(t), None, PartKind::Stair, 0),
                Stage::Finish,
                (
                    l.u_line(0) + l.pd / 2.0 + run / 2.0,
                    v,
                    l.z_g + l.h * f64::from(t) + l.h / 2.0,
                ),
                [l.stair_len(), width, STAIR_DEPTH_CM],
                (l.along(), STAIR_TILT_CENTIDEG),
                Some(BOARDS),
            );
        }
    } else if l.storeys > 1 {
        ladders += 1.0;
        p.push(
            (Level::Upper, None, PartKind::Ladder, 0),
            Stage::Finish,
            (l.u_line(0) + l.b / 2.0, -l.w / 4.0, l.z_g + l.h / 2.0),
            size(l.h),
            (l.along(), LADDER_TILT_CENTIDEG),
            Some(BOARDS),
        );
    }
    if let Some(&j) = l.lofts.first() {
        let from = l.z_g + l.h * f64::from(l.storeys - 1);
        let rise = l.z_p + l.jst + l.dk - from;
        ladders += 1.0;
        p.push(
            (Level::Loft, None, PartKind::Ladder, 0),
            Stage::Finish,
            (l.u_line(j) + l.b / 2.0, l.w / 4.0, from + rise / 2.0),
            size(rise),
            (l.along(), LADDER_TILT_CENTIDEG),
            Some(BOARDS),
        );
    }
    ladders
}

/// What the spaces give: the spaces, floor by use, storage by kind, places to work, floor area
/// and sleeping places.
struct Facts {
    spaces: Vec<Space>,
    floor_by_use: [f64; 3],
    storage_kg: [f64; 3],
    work_places: u32,
    floor_area_m2: f64,
    sleeping_places: u32,
}

fn facts(l: &Layout) -> Facts {
    let rules = l.rules;
    let storey_m2 = l.l * l.w * M2;
    let bay_m2 = l.b * l.w * M2;
    let mut spaces: Vec<Space> = (0..l.storeys)
        .map(|t| Space {
            level: storey_level(t),
            bays: l.all_bays as u16,
            area_m2: storey_m2 - l.stair_m2(t),
            use_: l.use_of(t),
        })
        .collect();
    if !l.lofts.is_empty() {
        spaces.push(Space {
            level: Level::Loft,
            bays: l.loft_mask as u16,
            area_m2: bay_m2 * l.lofts.len() as f64,
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
            (SpaceUse::Store, Level::Ground) if l.raised => {
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
    Facts {
        spaces,
        floor_by_use,
        storage_kg,
        work_places,
        floor_area_m2,
        sleeping_places,
    }
}

/// The labour and materials of each stage, from the members' counts and sizes, the walls'
/// area (square centimetres) and the ladders (a flight of stairs in ladders).
fn needs(l: &Layout, wall_area_cm2: f64, wall_volume_cm3: f64, ladders: f64) -> Vec<StageNeeds> {
    let rules = l.rules;
    let n = l.n;
    let storey_m2 = l.l * l.w * M2;
    let bay_m2 = l.b * l.w * M2;
    let wall_posts = if l.wall_kind == 0 { 2 * (n + 1) } else { 0 };
    // Storeyed posts: the first storey's set in the ground, and a post a storey above it.
    let posts_above = if l.storeyed {
        wall_posts * (l.storeys as usize - 1)
    } else {
        0
    };
    let floor_posts = if l.raised { n + 1 } else { 0 };
    let rafters = 2 * (n * l.rafters_per_bay + 1);
    let floored_bays = l.decks().len();
    let joists = l.joists_per_bay * floored_bays;
    let rails = l
        .floors
        .iter()
        .map(|&(_, _, middle)| if middle { 3 } else { 2 })
        .sum::<usize>()
        * n;
    let cross_beams = (1 + l.floors.len()) * (n + 1);
    let beams_m = (2 * n + rails) as f64 * l.b * 0.01 + cross_beams as f64 * l.w * 0.01;
    let ridge_m = (l.l + 2.0 * l.o) * 0.01;
    // A truss's principals and king post at each frame line, and two purlins a bay.
    // Wall ties round every storey's head.
    let ties_m = if l.ties {
        f64::from(l.storeys) * 2.0 * (l.l + l.w) * 0.01
    } else {
        0.0
    };
    let truss_m = if l.truss {
        ((n + 1) as f64 * (2.0 * l.principal_len() + l.king_post_h()) + 2.0 * n as f64 * l.b) * 0.01
    } else {
        0.0
    };
    let roof_m2 = (l.l + 2.0 * l.o) * (l.w + 2.0 * l.o) * M2 / l.pitch.cos();
    let wall_m2 = wall_area_cm2 * M2;
    let deck_m2 = floored_bays as f64 * bay_m2;
    let beaten_m2 = if l.raised { 0.0 } else { storey_m2 };
    let timber = |d: f64, count: usize, len_cm: f64| {
        round_section_m2(d) * count as f64 * len_cm * 0.01 * rules.timber_kg_per_m3
    };
    let floor_posts_kg = timber(l.pd, floor_posts, l.floor_post_h + l.depth);
    let frame_kg = if l.storeyed {
        timber(l.pd, wall_posts, l.z_g + l.h + l.depth) + timber(l.pd, posts_above, l.h)
    } else {
        timber(l.pd, wall_posts, l.z_p + l.depth)
    } + timber(
        l.beam,
        1,
        beams_m * 100.0 + ridge_m * 100.0 + truss_m * 100.0,
    ) + timber(l.raf, rafters, l.slope)
        + timber(l.jst, joists, l.w);
    let boards_kg = deck_m2 * l.dk * 0.01 * rules.timber_kg_per_m3 + rules.ladder_kg * ladders;
    let slots = |timber: f64, wattle: f64, covering: f64, boards: f64| {
        let mut kg = vec![0.0; frame_materials::COUNT];
        kg[usize::from(TIMBER)] = timber;
        kg[usize::from(WATTLE)] = wattle;
        kg[usize::from(COVERING)] = covering;
        kg[usize::from(BOARDS)] = boards;
        kg
    };

    let foundation_labour = if l.footing == 1 {
        let perimeter_m = 2.0 * (l.l + l.w) * 0.01;
        let trench_vol_m3 = perimeter_m * (l.wt * 0.01) * (l.depth * 0.01);
        rules.groundwork_h_per_m2 * storey_m2
            + trench_vol_m3 * (rules.daub_h_per_m3 + 12.0)
            + rules.posthole_h * floor_posts as f64
            + rules.post_h * floor_posts as f64
    } else {
        rules.groundwork_h_per_m2 * storey_m2
            + rules.posthole_h * (wall_posts + floor_posts) as f64
            + rules.post_h * floor_posts as f64
    };

    let (walls_labour, walls_kg) = if l.wall_kind == 0 {
        (
            (rules.wattle_h_per_m2 + rules.daub_h_per_m3 * l.wt * 0.01) * wall_m2,
            rules.wattle_kg_per_m2 * wall_m2,
        )
    } else {
        // Version 3's walls thin storey by storey; before it they are one thickness.
        let wall_vol_m3 = if l.version >= 3 {
            wall_volume_cm3 * 1.0e-6
        } else {
            wall_m2 * (l.wt * 0.01)
        };
        let (masonry_h_per_m3, masonry_density) = match l.wall_kind {
            1 => (32.0, 1900.0),
            2 => (16.0, 1600.0),
            3 => (20.0, 1800.0),
            _ => (20.0, 1800.0),
        };
        (
            wall_vol_m3 * masonry_h_per_m3,
            wall_vol_m3 * masonry_density,
        )
    };

    let (roof_labour, roof_kg) = if l.tile {
        (2.0 * roof_m2, 65.0 * roof_m2)
    } else {
        (
            rules.thatch_h_per_m2 * roof_m2,
            rules.thatch_kg_per_m2 * roof_m2,
        )
    };

    vec![
        StageNeeds {
            stage: Stage::Foundation,
            labour_h: foundation_labour,
            materials_kg: slots(floor_posts_kg, 0.0, 0.0, 0.0),
        },
        StageNeeds {
            stage: Stage::Frame,
            labour_h: rules.post_h * (wall_posts + posts_above) as f64
                + rules.beam_h_per_m * (beams_m + ridge_m + truss_m)
                + rules.rafter_h * rafters as f64
                + rules.joist_h_per_m * joists as f64 * l.w * 0.01,
            materials_kg: slots(frame_kg, 0.0, 0.0, 0.0),
        },
        StageNeeds {
            stage: Stage::Walls,
            labour_h: walls_labour + rules.beam_h_per_m * ties_m,
            materials_kg: slots(timber(l.beam, 1, ties_m * 100.0), walls_kg, 0.0, 0.0),
        },
        StageNeeds {
            stage: Stage::Roof,
            labour_h: roof_labour,
            materials_kg: slots(0.0, 0.0, roof_kg, 0.0),
        },
        StageNeeds {
            stage: Stage::Finish,
            labour_h: rules.finish_h_per_m2 * beaten_m2
                + rules.decking_h_per_m2 * deck_m2
                + rules.ladder_h * ladders,
            materials_kg: slots(0.0, 0.0, 0.0, boards_kg),
        },
    ]
}

/// A group of members, before its id is made: level, bay, kind and index; stage and material;
/// count; section, span and spacing (centimetres); the area it carries or covers.
struct GroupOf {
    level: Level,
    bay: Option<usize>,
    kind: GroupKind,
    index: u8,
    stage: Stage,
    material: Option<u8>,
    count: usize,
    section: [f64; 2],
    length: f64,
    spacing: f64,
    area_m2: f64,
}

impl GroupOf {
    fn group(self) -> Group {
        Group {
            id: group_id(self.level, self.bay.map(|b| b as u8), self.kind, self.index),
            kind: self.kind,
            stage: self.stage,
            material: self.material,
            count: self.count as u32,
            section_cm: self.section.map(cm),
            length_cm: cm(self.length),
            spacing_cm: cm(self.spacing),
            area_m2: self.area_m2,
        }
    }
}

/// The component groups (ADR-0009 §3), each with what it carries or covers, in stage order.
fn groups(l: &Layout) -> Result<Vec<Group>, GrammarError> {
    let n = l.n;
    let storey_m2 = l.l * l.w * M2;
    let bay_m2 = l.b * l.w * M2;
    let roof_m2 = (l.l + 2.0 * l.o) * (l.w + 2.0 * l.o) * M2 / l.pitch.cos();
    let decks = l.decks();
    let roof_plan_side_m2 = (l.l + 2.0 * l.o) * (l.w / 2.0 + l.o) * M2;
    let suspended_m2 = decks.len() as f64 * bay_m2;
    let raised_m2 = if l.raised { storey_m2 } else { 0.0 };
    let joist_share = l.joists_per_bay as f64 / (l.joists_per_bay + 1) as f64;
    let timber = Some(TIMBER);
    let storeys_h = l.h * f64::from(l.storeys);
    let mut out = Vec::new();
    if l.wall_kind == 0 && l.storeyed {
        // Each storey's posts carry the roof's slope, half of every floor above their storey and
        // the lofts, and the first storey's a raised floor's share too.
        let lofts_m2 = l.lofts.len() as f64 * bay_m2;
        for t in 0..l.storeys {
            let floors_above = f64::from(l.storeys - 1 - t) * storey_m2;
            for (side, _) in l.long_walls() {
                out.push(GroupOf {
                    level: storey_level(t),
                    bay: None,
                    kind: GroupKind::Posts,
                    index: side as u8,
                    stage: Stage::Frame,
                    material: timber,
                    count: n + 1,
                    section: [l.pd, l.pd],
                    length: if t == 0 { l.z_g + l.h } else { l.h },
                    spacing: l.b,
                    area_m2: roof_plan_side_m2
                        + (lofts_m2 + floors_above) / 2.0
                        + if t == 0 { raised_m2 / 4.0 } else { 0.0 },
                });
            }
        }
    } else if l.wall_kind == 0 {
        for (side, _) in l.long_walls() {
            // Each long wall carries its slope of the roof and half of every floor across the width
            // (a raised floor's middle line takes half of that one).
            out.push(GroupOf {
                level: Level::Ground,
                bay: None,
                kind: GroupKind::Posts,
                index: side as u8,
                stage: Stage::Frame,
                material: timber,
                count: n + 1,
                section: [l.pd, l.pd],
                length: l.z_p,
                spacing: l.b,
                area_m2: roof_plan_side_m2 + (suspended_m2 - raised_m2) / 2.0 + raised_m2 / 4.0,
            });
        }
    } else if l.version >= 3 {
        // Version 3: one group per long wall per storey, each at its own thickness and height,
        // each carrying what bears on it from above (ADR-0031 §1; its own weight and the walls
        // above it are slice CZ's).
        for t in 0..l.storeys {
            for (side, _) in l.long_walls() {
                out.push(GroupOf {
                    level: storey_level(t),
                    bay: None,
                    kind: GroupKind::MassWall,
                    index: side as u8,
                    stage: Stage::Walls,
                    material: Some(WATTLE),
                    count: 1,
                    // Its windows shorten the length it bears on.
                    section: [l.l - l.windows_along(t, side, l.b), l.storey_wt[t as usize]],
                    length: l.h,
                    spacing: l.wall_kind as f64,
                    area_m2: roof_plan_side_m2 + (suspended_m2 - raised_m2) / 2.0 + raised_m2 / 4.0,
                });
            }
        }
    } else {
        for (side, _) in l.long_walls() {
            out.push(GroupOf {
                level: Level::Ground,
                bay: None,
                kind: GroupKind::MassWall,
                index: side as u8,
                stage: Stage::Walls,
                material: Some(WATTLE),
                count: 1,
                section: [l.l, l.wt],
                length: storeys_h,
                spacing: l.wall_kind as f64,
                area_m2: roof_plan_side_m2 + (suspended_m2 - raised_m2) / 2.0 + raised_m2 / 4.0,
            });
        }
    }
    if l.ties {
        // Each storey's ties restrain its walls; by gravity they carry nothing.
        for t in 0..l.storeys {
            out.push(GroupOf {
                level: storey_level(t),
                bay: None,
                kind: GroupKind::Ties,
                index: 0,
                stage: Stage::Walls,
                material: timber,
                count: 2 * n + 2,
                section: [l.beam, l.beam],
                length: l.b,
                spacing: 0.0,
                area_m2: 0.0,
            });
        }
    }
    if l.raised {
        out.push(GroupOf {
            level: Level::Ground,
            bay: None,
            kind: GroupKind::Posts,
            index: 2,
            stage: Stage::Foundation,
            material: timber,
            count: n + 1,
            section: [l.pd, l.pd],
            length: l.floor_post_h,
            spacing: l.b,
            area_m2: raised_m2 / 2.0,
        });
    }
    // Tie beams at the wall heads carry the edges of the lofts beside them; cross beams carry
    // their floor's edges, the share its joists do not.
    out.push(GroupOf {
        level: Level::Roof,
        bay: None,
        kind: GroupKind::TieBeams,
        index: 0,
        stage: Stage::Frame,
        material: timber,
        count: n + 1,
        section: [l.beam, l.beam],
        length: l.w,
        spacing: l.b,
        area_m2: l.lofts.len() as f64 * bay_m2 * (1.0 - joist_share),
    });
    for &(level, _, middle) in &l.floors {
        out.push(GroupOf {
            level,
            bay: None,
            kind: GroupKind::TieBeams,
            index: 0,
            stage: Stage::Frame,
            material: timber,
            count: n + 1,
            section: [l.beam, l.beam],
            length: if middle { l.w / 2.0 } else { l.w },
            spacing: l.b,
            area_m2: storey_m2 * (1.0 - joist_share),
        });
    }
    for &(level, j, _) in &decks {
        let (kind, span) = match level {
            Level::Loft => (GroupKind::LoftJoists, l.w),
            Level::Upper | Level::Third | Level::Fourth => (GroupKind::FloorJoists, l.w),
            // A raised floor, with its middle rail.
            Level::Ground | Level::Roof => (GroupKind::RaisedFloor, l.w / 2.0),
        };
        out.push(GroupOf {
            level,
            bay: Some(j),
            kind,
            index: 0,
            stage: Stage::Frame,
            material: timber,
            count: l.joists_per_bay,
            section: [l.jst, l.jst],
            length: span,
            spacing: l.joist_spacing,
            area_m2: bay_m2 * joist_share,
        });
    }
    for j in 0..n {
        let ends = if j == 0 { l.o } else { 0.0 } + if j + 1 == n { l.o } else { 0.0 };
        let last = if j + 1 == n { 2 } else { 0 };
        out.push(GroupOf {
            level: Level::Roof,
            bay: Some(j),
            kind: GroupKind::RoofFrame,
            index: 0,
            stage: Stage::Frame,
            material: timber,
            count: 2 * l.rafters_per_bay + last,
            section: [l.raf, l.raf],
            // On a truss's purlin a rafter spans half its slope.
            length: if l.truss { l.slope / 2.0 } else { l.slope },
            spacing: l.b / l.rafters_per_bay as f64,
            area_m2: (l.b + ends) * (l.w + 2.0 * l.o) * M2 / l.pitch.cos(),
        });
    }
    if l.truss {
        // The trusses carry the whole roof, the purlins of each slope half of it.
        out.push(GroupOf {
            level: Level::Roof,
            bay: None,
            kind: GroupKind::Trusses,
            index: 0,
            stage: Stage::Frame,
            material: timber,
            count: n + 1,
            section: [l.beam, l.beam],
            length: l.principal_len(),
            spacing: l.b,
            area_m2: 2.0 * roof_plan_side_m2,
        });
        for (sl, _) in SLOPES {
            out.push(GroupOf {
                level: Level::Roof,
                bay: None,
                kind: GroupKind::Purlins,
                index: sl as u8,
                stage: Stage::Frame,
                material: timber,
                count: n,
                section: [l.beam, l.beam],
                length: l.b,
                spacing: 0.0,
                area_m2: roof_m2 / 4.0,
            });
        }
    }
    for (sl, _) in SLOPES {
        out.push(GroupOf {
            level: Level::Roof,
            bay: None,
            kind: GroupKind::Covering,
            index: sl as u8,
            stage: Stage::Roof,
            material: Some(COVERING),
            count: 1,
            section: [if l.tile {
                4.0
            } else {
                f64::from(l.rules.thatch_thickness_cm)
            }; 2],
            length: l.slope,
            spacing: 0.0,
            area_m2: roof_m2 / 2.0,
        });
    }
    let gable_h = l.z_a - l.z_p;
    let door_m2 = l.door_w * l.h * M2;
    if l.wall_kind == 0 {
        // What the windows leave open of a wall, square metres, over every storey.
        let open_m2 = |side: i32, panel: f64| -> f64 {
            (0..l.storeys)
                .map(|t| {
                    if side >= SIDE_BACK {
                        l.window(t, panel, l.door_here(t, side, 0))
                    } else {
                        l.windows_along(t, side, panel)
                    }
                })
                .sum::<f64>()
                * l.h
                / 2.0
                * M2
        };
        for (side, _) in l.long_walls() {
            let door = if side == l.door_side { door_m2 } else { 0.0 };
            out.push(GroupOf {
                level: Level::Ground,
                bay: None,
                kind: GroupKind::Infill,
                index: side as u8,
                stage: Stage::Walls,
                material: Some(WATTLE),
                count: n * l.storeys as usize + usize::from(side == l.door_side),
                section: [l.wt, l.wt],
                length: storeys_h,
                spacing: l.b,
                area_m2: (l.b - l.pd) * storeys_h * M2 * n as f64
                    - door
                    - open_m2(side, l.b - l.pd),
            });
        }
        for (side, _) in l.end_walls() {
            let door = if side == l.door_side { door_m2 } else { 0.0 };
            out.push(GroupOf {
                level: Level::Ground,
                bay: None,
                kind: GroupKind::Infill,
                index: side as u8,
                stage: Stage::Walls,
                material: Some(WATTLE),
                count: l.storeys as usize + 1 + usize::from(side == l.door_side),
                section: [l.wt, l.wt],
                length: storeys_h + gable_h,
                spacing: 0.0,
                area_m2: (l.w - l.pd) * (storeys_h + gable_h / 2.0) * M2
                    - door
                    - open_m2(side, l.w - l.pd),
            });
        }
    }
    if !l.raised {
        out.push(GroupOf {
            level: Level::Ground,
            bay: None,
            kind: GroupKind::Floor,
            index: 0,
            stage: Stage::Finish,
            material: None,
            count: 1,
            section: [0.0, 0.0],
            length: 0.0,
            spacing: 0.0,
            area_m2: storey_m2,
        });
    }
    let most = max_groups(l.version);
    if out.len() > most {
        return refuse(format!(
            "{} component groups are more than {most}",
            out.len()
        ));
    }
    let mut groups: Vec<Group> = out.into_iter().map(GroupOf::group).collect();
    groups.sort_by_key(|g| (g.stage, g.id));
    Ok(groups)
}

/// Expands a frame building (see the module's description).
pub fn expand_frame(spec: &BuildingSpec, rules: &FrameRules) -> Result<Expansion, GrammarError> {
    let l = Layout::new(spec, rules)?;
    let groups = groups(&l)?;
    let mut p = Parts {
        layout: &l,
        parts: Vec::new(),
    };
    foundation(&mut p);
    frame(&mut p);
    let (wall_area_cm2, wall_volume_cm3) = walls(&mut p);
    ties(&mut p);
    roof(&mut p);
    let ladders = finish(&mut p);
    let mut parts = p.parts;
    parts.sort_by_key(|p| (p.stage, p.id));

    // Outline, roof and door.
    let corners = |du: f64, dv: f64| -> Vec<(i32, i32)> {
        [(-du, -dv), (du, -dv), (du, dv), (-du, dv)]
            .into_iter()
            .map(|(u, v)| {
                let (x, y) = l.at(u, v);
                (cm(x), cm(y))
            })
            .collect()
    };
    let ridge = {
        let (x0, y0) = l.at(-l.l / 2.0 - l.o, 0.0);
        let (x1, y1) = l.at(l.l / 2.0 + l.o, 0.0);
        [(cm(x0), cm(y0)), (cm(x1), cm(y1))]
    };
    let (door_u, door_v, facing) = l.door();
    let (dx, dy) = l.at(door_u, door_v);
    let f = facts(&l);
    Ok(Expansion {
        grammar: Grammar::Frame,
        version: spec.version,
        parts,
        outline: corners(l.l / 2.0 + l.wt / 2.0, l.w / 2.0 + l.wt / 2.0),
        roof_radius_cm: cm((l.l / 2.0 + l.o).hypot(l.w / 2.0 + l.o)),
        apex_cm: cm(l.z_a),
        door: (cm(dx), cm(dy), facing),
        door_width_cm: rules.door_cm,
        stages: needs(&l, wall_area_cm2, wall_volume_cm3, ladders),
        floor_area_m2: f.floor_area_m2,
        sleeping_places: f.sleeping_places,
        roof_outline: corners(l.l / 2.0 + l.o, l.w / 2.0 + l.o),
        ridge: Some(ridge),
        spaces: f.spaces,
        groups,
        floor_by_use: f.floor_by_use,
        storage_kg: f.storage_kg,
        work_places: f.work_places,
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
            version: 1,
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
        let tall = [
            (tall(3), tall_house()),
            (tall(4), tall_house()),
            (windowed(), tall_house()),
            (trussed(), house()),
            (
                BuildingSpec {
                    params: {
                        let mut p = tall(3).params;
                        p[fp::POST_SYSTEM] = 1;
                        p
                    },
                    ..tall(3)
                },
                tall_house(),
            ),
        ];
        for (s, r) in goldens().map(|(s, r, _)| (s, r)).into_iter().chain(tall) {
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
        let tall = [(tall(3), tall_house()), (tall(4), tall_house())];
        for (s, r) in goldens().map(|(s, r, _)| (s, r)).into_iter().chain(tall) {
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
    fn version_three_with_nothing_of_its_own_expands_as_version_two() {
        // ADR-0031 §1: what version 3 adds is chosen by its own parameters; with them all 0 it
        // builds what version 2 builds, and only the version it records differs.
        for (spec, rules, _) in goldens() {
            let v2 = expand_frame(
                &BuildingSpec {
                    version: 2,
                    ..spec.clone()
                },
                &rules,
            )
            .expect("version 2 expands");
            let mut v3 = expand_frame(&BuildingSpec { version: 3, ..spec }, &rules)
                .expect("version 3 expands");
            assert_eq!(v3.version, 3);
            v3.version = v2.version;
            assert_eq!(v3, v2);
        }
    }

    #[test]
    fn version_three_has_room_for_more_bays_and_its_parameters_are_its_own() {
        let mut r = house();
        r.bays = (1, 12);
        let at = |version: u16, bays: i32| {
            let mut s = spec(bays, 250, 500);
            s.version = version;
            s
        };
        assert!(
            expand_frame(&at(2, 9), &r).is_err(),
            "version 2 has at most eight bays"
        );
        assert!(expand_frame(&at(3, 12), &r).is_ok());
        assert!(expand_frame(&at(3, 13), &r).is_err());
        // A version 3 parameter is unknown before it, and refused by name.
        let mut old = at(2, 3);
        old.params[fp::TIES] = 1;
        assert_eq!(
            expand_frame(&old, &r),
            Err(GrammarError::OutOfRange(format!(
                "parameter {} is unknown to frame version 2",
                fp::TIES
            )))
        );
        let mut spare = at(3, 3);
        spare.params[fp::SPARE] = 1;
        assert!(expand_frame(&spare, &r).is_err());
        // Three storeys need a stair.
        let mut r3 = r.clone();
        r3.storeys = (1, 4);
        let mut tall = at(3, 3);
        tall.storeys = 3;
        tall.params[fp::JOIST_CM] = 18;
        assert_eq!(
            expand_frame(&tall, &r3),
            Err(GrammarError::OutOfRange(format!(
                "3 storeys need a stair (parameter {})",
                fp::STAIR
            )))
        );
    }

    #[test]
    fn version_three_mass_walls_take_each_storeys_thickness_and_thin_upward() {
        let mut r = house();
        r.wall_cm = (10, 60);
        let mass = |walls: [i32; 2]| {
            let mut s = two_storeys();
            s.version = 3;
            s.params[fp::WALL_KIND] = 1;
            s.params[fp::FOOTING] = 1;
            s.params[fp::WALL_CM] = 50;
            s.params[fp::STOREY_WALL_CM] = walls[0];
            s.params[fp::STOREY_WALL_CM + 1] = walls[1];
            s
        };
        let walls_kg =
            |e: &Expansion| e.stages[Stage::Walls.index()].materials_kg[usize::from(WATTLE)];
        let tapered = expand_frame(&mass([50, 40]), &r).expect("a wall thinning upward");
        // Each storey's wall parts at its own thickness.
        let thickness = |level: Level| {
            tapered
                .parts
                .iter()
                .filter(|p| p.kind == PartKind::MassWall && p.id >> 24 == u32::from(level.code()))
                .map(|p| p.size[1])
                .collect::<Vec<_>>()
        };
        let (ground, upper) = (thickness(Level::Ground), thickness(Level::Upper));
        assert!(!ground.is_empty() && !upper.is_empty());
        assert!(ground.iter().all(|&t| t == 50), "{ground:?}");
        assert!(upper.iter().all(|&t| t == 40), "{upper:?}");
        // One group per long wall per storey, each its storey's height and thickness.
        let groups: Vec<_> = tapered
            .groups
            .iter()
            .filter(|g| g.kind == GroupKind::MassWall)
            .map(|g| (g.section_cm[1], g.length_cm))
            .collect();
        assert_eq!(groups.len(), 4, "{groups:?}");
        assert_eq!(groups.iter().filter(|g| g.0 == 50).count(), 2);
        assert_eq!(groups.iter().filter(|g| g.0 == 40).count(), 2);
        assert!(groups.iter().all(|g| g.1 == 200));
        // A wall of one thickness weighs as version 2's; thinning it upward saves stone.
        let even = expand_frame(&mass([0, 0]), &r).expect("one thickness");
        let mut v2 = mass([0, 0]);
        v2.version = 2;
        let v2 = expand_frame(&v2, &r).expect("version 2");
        assert!((walls_kg(&even) - walls_kg(&v2)).abs() < 1e-6 * walls_kg(&v2));
        assert!(walls_kg(&tapered) < walls_kg(&even));
        // Never thicker above; only storeys it has; only mass walls.
        assert!(expand_frame(&mass([40, 50]), &r).is_err(), "thicker above");
        let mut third = mass([50, 40]);
        third.params[fp::STOREY_WALL_CM + 2] = 30;
        assert!(
            expand_frame(&third, &r).is_err(),
            "a third storey's wall on two"
        );
        let mut wattle = mass([50, 40]);
        wattle.params[fp::WALL_KIND] = 0;
        wattle.params[fp::FOOTING] = 0;
        wattle.params[fp::WALL_CM] = 15;
        assert!(
            expand_frame(&wattle, &r).is_err(),
            "wattle has one thickness"
        );
    }

    /// Rules that allow a hall of up to twelve bays and four storeys.
    fn tall_house() -> FrameRules {
        FrameRules {
            bays: (1, 12),
            storeys: (1, 4),
            ..house()
        }
    }

    /// Four bays of 2.5 m by 5 m, `storeys` high, with a stair (version 3).
    fn tall(storeys: u8) -> BuildingSpec {
        let mut s = two_storeys();
        s.version = 3;
        s.storeys = storeys;
        s.params[fp::STAIR] = 1;
        s
    }

    #[test]
    fn version_three_rises_three_and_four_storeys_by_a_stair() {
        let r = tall_house();
        // A flight rises 2 m at 38°, so it runs 2 m / tan 38° along the length, 80 cm wide.
        let flight_m2 = 200.0 / (38.0_f64).to_radians().tan() * 80.0 * 1.0e-4;
        for storeys in [3_u8, 4] {
            let e = expand_frame(&tall(storeys), &r).expect("a tall building expands");
            let n = usize::from(storeys);
            let levels: Vec<Level> = e.spaces.iter().map(|s| s.level).collect();
            assert_eq!(
                levels,
                (0..n).filter_map(Level::storey).collect::<Vec<_>>(),
                "a space a storey"
            );
            // Each storey gives up the flight rising from it and the well it is reached through.
            for (t, s) in e.spaces.iter().enumerate() {
                let flights = usize::from(t > 0) + usize::from(t + 1 < n);
                assert!(
                    (s.area_m2 - (50.0 - flights as f64 * flight_m2)).abs() < 1e-9,
                    "storey {t}: {}",
                    s.area_m2
                );
            }
            let stairs: Vec<&Part> = e
                .parts
                .iter()
                .filter(|p| p.kind == PartKind::Stair)
                .collect();
            assert_eq!(stairs.len(), n - 1, "a flight from each storey to the next");
            assert!(stairs.iter().all(|p| p.tilt == STAIR_TILT_CENTIDEG));
            assert!(
                !e.parts.iter().any(|p| p.kind == PartKind::Ladder),
                "the stair takes the ladder's place"
            );
            let ids: HashSet<u32> = e.parts.iter().map(|p| p.id).collect();
            assert_eq!(ids.len(), e.parts.len(), "part ids are unique");
            let gids: HashSet<u32> = e.groups.iter().map(|g| g.id).collect();
            assert_eq!(gids.len(), e.groups.len(), "group ids are unique");
            // Every storey above the first floored on joists, a group a bay.
            let floors = e
                .groups
                .iter()
                .filter(|g| g.kind == GroupKind::FloorJoists)
                .count();
            assert_eq!(floors, 4 * (n - 1));
            let top = Level::storey(n - 1).expect("a storey");
            assert!(
                e.groups
                    .iter()
                    .any(|g| g.kind == GroupKind::TieBeams && g.id >> 24 == u32::from(top.code()))
            );
            // Posts run through every storey to the plates.
            let posts = e
                .groups
                .iter()
                .find(|g| g.kind == GroupKind::Posts)
                .expect("posts");
            assert_eq!(posts.length_cm, 200 * i32::from(storeys));
            // Walls on every storey; living floor from every storey.
            assert!(
                e.parts
                    .iter()
                    .any(|p| p.kind == PartKind::Daub && p.id >> 24 == u32::from(top.code()))
            );
            assert!((e.floor_area_m2 - e.floor_by_use[SpaceUse::Living.index()]).abs() < 1e-9);
        }
        // A third storey takes more of every stage but the roof's than two.
        let two = expand_frame(
            &BuildingSpec {
                storeys: 2,
                ..tall(3)
            },
            &r,
        )
        .expect("two storeys by a stair");
        let three = expand_frame(&tall(3), &r).expect("three storeys");
        for stage in [Stage::Frame, Stage::Walls, Stage::Finish] {
            assert!(three.stages[stage.index()].labour_h > two.stages[stage.index()].labour_h);
        }
        assert_eq!(
            three.stages[Stage::Roof.index()],
            two.stages[Stage::Roof.index()]
        );
        // Two storeys by a stair: one flight and no ladder; costlier than the ladder it replaces.
        assert_eq!(
            two.parts
                .iter()
                .filter(|p| p.kind == PartKind::Stair)
                .count(),
            1
        );
        let mut laddered = tall(3);
        laddered.storeys = 2;
        laddered.params[fp::STAIR] = 0;
        let laddered = expand_frame(&laddered, &r).expect("two storeys by a ladder");
        assert!(
            two.stages[Stage::Finish.index()].labour_h
                > laddered.stages[Stage::Finish.index()].labour_h
        );
        // Twelve bays of four storeys, lofted throughout, stay within version 3's groups.
        let mut hall = tall(4);
        hall.params[fp::BAYS] = 12;
        hall.params[fp::LOFT_BAYS] = (1 << 12) - 1;
        if let Footprint::Rect { length, .. } = &mut hall.footprint {
            *length = 12 * 250;
        }
        let e = expand_frame(&hall, &r).expect("a hall of twelve bays and four storeys");
        assert!(e.groups.len() > MAX_GROUPS && e.groups.len() <= MAX_GROUPS_V3);
        assert_eq!(max_groups(2), MAX_GROUPS);
    }

    /// Two storeys by a stair with a fifth of the first storey's wall open and a tenth of the
    /// second's.
    fn windowed() -> BuildingSpec {
        let mut s = tall(2);
        s.params[fp::OPENINGS] = 200;
        s.params[fp::OPENINGS + 1] = 100;
        s
    }

    #[test]
    fn version_three_openings_leave_each_storeys_share_of_wall_open() {
        let r = tall_house();
        let e = expand_frame(&windowed(), &r).expect("a building with windows");
        let shut = expand_frame(
            &BuildingSpec {
                params: {
                    let mut p = windowed().params;
                    p[fp::OPENINGS] = 0;
                    p[fp::OPENINGS + 1] = 0;
                    p
                },
                ..windowed()
            },
            &r,
        )
        .expect("the same without windows");
        let windows: Vec<&Part> = e
            .parts
            .iter()
            .filter(|p| p.kind == PartKind::Window)
            .collect();
        // A window in every piece of wall but the door's: seven long pieces and two ends below,
        // eight and two above.
        let on = |level: Level| {
            windows
                .iter()
                .filter(|p| p.id >> 24 == u32::from(level.code()))
                .count()
        };
        assert_eq!((on(Level::Ground), on(Level::Upper)), (9, 10));
        assert!(
            windows.iter().all(|p| p.size[2] == 100),
            "half a storey high"
        );
        assert!(!shut.parts.iter().any(|p| p.kind == PartKind::Window));
        // What they leave open: 2 × share × piece wide, half a storey high.
        let open_cm2 = 7.0 * (0.4 * 235.0) * 100.0
            + 2.0 * (0.4 * 485.0) * 100.0
            + 8.0 * (0.2 * 235.0) * 100.0
            + 2.0 * (0.2 * 485.0) * 100.0;
        let open_m2 = open_cm2 * 1.0e-4;
        let wattle =
            |e: &Expansion| e.stages[Stage::Walls.index()].materials_kg[usize::from(WATTLE)];
        assert!(
            (wattle(&shut) - wattle(&e) - open_m2 * r.wattle_kg_per_m2).abs() < 1e-6,
            "an opening costs no wall"
        );
        let infill = |e: &Expansion| -> f64 {
            e.groups
                .iter()
                .filter(|g| g.kind == GroupKind::Infill)
                .map(|g| g.area_m2)
                .sum()
        };
        assert!((infill(&shut) - infill(&e) - open_m2).abs() < 1e-9);
        // A mass wall's windows shorten the length it bears on, storey by storey.
        let mut rubble = windowed();
        rubble.params[fp::WALL_KIND] = 1;
        rubble.params[fp::FOOTING] = 1;
        rubble.params[fp::WALL_CM] = 50;
        let mut rr = r.clone();
        rr.wall_cm = (10, 60);
        let m = expand_frame(&rubble, &rr).expect("rubble with windows");
        let bearing = |level: Level, side: i32| {
            m.groups
                .iter()
                .find(|g| g.id == group_id(level, None, GroupKind::MassWall, side as u8))
                .map(|g| g.section_cm[0])
                .expect("the wall's group")
        };
        // The door is in bay 2 of the right wall: three windows of a metre below, four on the left.
        assert_eq!(bearing(Level::Ground, fp::SIDE_RIGHT), 1000 - 300);
        assert_eq!(bearing(Level::Ground, fp::SIDE_LEFT), 1000 - 400);
        assert_eq!(bearing(Level::Upper, fp::SIDE_RIGHT), 1000 - 200);
        rubble.params[fp::OPENINGS] = 0;
        rubble.params[fp::OPENINGS + 1] = 0;
        let solid = expand_frame(&rubble, &rr).expect("rubble without windows");
        assert!(
            m.stages[Stage::Walls.index()].labour_h < solid.stages[Stage::Walls.index()].labour_h
        );
        // Refused: more than the most, a storey it has not, and before version 3.
        let mut wide = windowed();
        wide.params[fp::OPENINGS] = MAX_OPENING_PERMILLE + 1;
        assert!(expand_frame(&wide, &r).is_err());
        let mut third = windowed();
        third.params[fp::OPENINGS + 2] = 100;
        assert_eq!(
            expand_frame(&third, &r),
            Err(GrammarError::OutOfRange(format!(
                "storey 3 has openings (parameter {}) and no storey",
                fp::OPENINGS + 2
            )))
        );
        let mut old = spec(3, 250, 500);
        old.version = 2;
        old.params[fp::OPENINGS] = 100;
        assert_eq!(
            expand_frame(&old, &r),
            Err(GrammarError::OutOfRange(format!(
                "parameter {} is unknown to frame version 2",
                fp::OPENINGS
            )))
        );
    }

    /// Three bays with a loft (version 3) under a trussed roof.
    fn trussed() -> BuildingSpec {
        let mut s = three_bays_one_loft();
        s.version = 3;
        s.params[fp::ROOF_SYSTEM] = 1;
        s
    }

    #[test]
    fn a_trussed_roof_has_a_truss_at_each_frame_line_and_purlins_halving_the_rafters_span() {
        let r = house();
        let e = expand_frame(&trussed(), &r).expect("a trussed roof");
        let poles = expand_frame(
            &BuildingSpec {
                version: 3,
                ..three_bays_one_loft()
            },
            &r,
        )
        .expect("pole rafters");
        let count = |k: PartKind| e.parts.iter().filter(|p| p.kind == k).count();
        // Four frame lines of two principals and a king post; two purlins a bay.
        assert_eq!(count(PartKind::Truss), 4 * 3);
        assert_eq!(count(PartKind::Purlin), 3 * 2);
        assert!(!poles.parts.iter().any(|p| p.kind == PartKind::Truss));
        let group = |e: &Expansion, k: GroupKind| e.groups.iter().filter(|g| g.kind == k).count();
        assert_eq!(group(&e, GroupKind::Trusses), 1);
        assert_eq!(group(&e, GroupKind::Purlins), 2);
        // A rafter on a purlin spans half its slope.
        let span = |e: &Expansion| {
            e.groups
                .iter()
                .find(|g| g.kind == GroupKind::RoofFrame)
                .map(|g| g.length_cm)
                .expect("a roof frame")
        };
        assert!((span(&e) * 2 - span(&poles)).abs() <= 1);
        // The trusses' and purlins' timber is costed as beams: 2.5 m / cos 45° principals and a
        // 2.5 m king post at four lines, and six purlins of 2.5 m.
        let principal = 250.0 / (45.0_f64).to_radians().cos();
        let truss_m = (4.0 * (2.0 * principal + 250.0) + 6.0 * 250.0) * 0.01;
        let frame_h = |e: &Expansion| e.stages[Stage::Frame.index()].labour_h;
        assert!((frame_h(&e) - frame_h(&poles) - r.beam_h_per_m * truss_m).abs() < 1e-6);
        let beam = f64::from(r.beam_cm) / 100.0;
        let kg = std::f64::consts::PI / 4.0 * beam * beam * truss_m * r.timber_kg_per_m3;
        let frame_kg =
            |e: &Expansion| e.stages[Stage::Frame.index()].materials_kg[usize::from(TIMBER)];
        assert!((frame_kg(&e) - frame_kg(&poles) - kg).abs() < 1e-6);
        // Refused: an unknown system, and a truss before version 3.
        let mut odd = trussed();
        odd.params[fp::ROOF_SYSTEM] = 2;
        assert_eq!(
            expand_frame(&odd, &r),
            Err(GrammarError::OutOfRange(format!(
                "roof system 2 (parameter {}) is unknown to frame version 3",
                fp::ROOF_SYSTEM
            )))
        );
        let mut old = trussed();
        old.version = 2;
        assert!(expand_frame(&old, &r).is_err());
    }

    #[test]
    fn storeyed_posts_stand_a_storey_high_on_the_frame_below() {
        let r = tall_house();
        let through = expand_frame(&tall(3), &r).expect("posts through three storeys");
        let mut s = tall(3);
        s.params[fp::POST_SYSTEM] = 1;
        let e = expand_frame(&s, &r).expect("storeyed posts");
        fn posts(e: &Expansion) -> Vec<&Part> {
            e.parts
                .iter()
                .filter(|p| p.kind == PartKind::Post)
                .collect()
        }
        // Five frame lines of two posts, through three storeys or one a storey.
        assert_eq!(posts(&through).len(), 10);
        assert_eq!(posts(&e).len(), 30);
        let ids: HashSet<u32> = e.parts.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), e.parts.len(), "part ids are unique");
        // The first storey's in the ground (70 cm) to its head; each above a storey high.
        for p in posts(&e) {
            let level = Level::from_code((p.id >> 24) as u8).expect("a level");
            let expect = if level == Level::Ground {
                200 + 70
            } else {
                200
            };
            assert_eq!(p.size[2], expect, "{level:?}");
        }
        // A posts group a wall a storey, each carrying the roof's slope and half the floors above
        // it: the first storey's what the through posts carry, the top storey's the roof alone.
        let group = |e: &Expansion, level: Level| {
            e.groups
                .iter()
                .find(|g| g.id == group_id(level, None, GroupKind::Posts, fp::SIDE_RIGHT as u8))
                .copied()
                .expect("a posts group")
        };
        let (g0, g2) = (group(&e, Level::Ground), group(&e, Level::Third));
        let whole = group(&through, Level::Ground);
        assert_eq!(
            e.groups
                .iter()
                .filter(|g| g.kind == GroupKind::Posts)
                .count(),
            6
        );
        assert_eq!(
            (g0.length_cm, g2.length_cm, whole.length_cm),
            (200, 200, 600)
        );
        assert!((g0.area_m2 - whole.area_m2).abs() < 1e-9);
        assert!(
            (whole.area_m2 - g2.area_m2 - 50.0).abs() < 1e-9,
            "two floors of 50 m², halved"
        );
        // As much post timber in all; more posts to joint and raise.
        let frame = |e: &Expansion| e.stages[Stage::Frame.index()].clone();
        assert!(
            (frame(&e).materials_kg[usize::from(TIMBER)]
                - frame(&through).materials_kg[usize::from(TIMBER)])
            .abs()
                < 1e-6
        );
        assert!((frame(&e).labour_h - frame(&through).labour_h - r.post_h * 20.0).abs() < 1e-9);
        // The same postholes.
        assert_eq!(
            e.stages[Stage::Foundation.index()],
            through.stages[Stage::Foundation.index()]
        );
        // Refused: an unknown system, mass walls, one storey, and before version 3.
        let mut odd = s.clone();
        odd.params[fp::POST_SYSTEM] = 2;
        assert!(expand_frame(&odd, &r).is_err());
        let mut rubble = s.clone();
        rubble.params[fp::WALL_KIND] = 1;
        rubble.params[fp::FOOTING] = 1;
        assert_eq!(
            expand_frame(&rubble, &r),
            Err(GrammarError::OutOfRange(format!(
                "storeyed posts (parameter {}) stand in a timber frame; mass walls have none",
                fp::POST_SYSTEM
            )))
        );
        let mut low = s.clone();
        low.storeys = 1;
        low.params[fp::STAIR] = 0;
        assert_eq!(
            expand_frame(&low, &r),
            Err(GrammarError::OutOfRange(format!(
                "storeyed posts (parameter {}) need a storey above the first",
                fp::POST_SYSTEM
            )))
        );
        let mut old = two_storeys();
        old.version = 2;
        old.params[fp::POST_SYSTEM] = 1;
        assert!(expand_frame(&old, &house()).is_err());
    }

    #[test]
    fn wall_ties_bind_each_storeys_mass_walls_at_its_head() {
        let mut r = house();
        r.wall_cm = (10, 60);
        let rubble = |ties: i32| {
            let mut s = two_storeys();
            s.version = 3;
            s.params[fp::WALL_KIND] = 1;
            s.params[fp::FOOTING] = 1;
            s.params[fp::WALL_CM] = 50;
            s.params[fp::TIES] = ties;
            s
        };
        let tied = expand_frame(&rubble(1), &r).expect("tied rubble");
        let loose = expand_frame(&rubble(0), &r).expect("untied rubble");
        // Four bays a long wall and the two ends, at the head of each of two storeys.
        let ties: Vec<&Part> = tied
            .parts
            .iter()
            .filter(|p| p.kind == PartKind::WallTie)
            .collect();
        assert_eq!(ties.len(), 2 * (2 * 4 + 2));
        assert!(!loose.parts.iter().any(|p| p.kind == PartKind::WallTie));
        let heads: HashSet<i32> = ties.iter().map(|p| p.at[2]).collect();
        assert_eq!(
            heads,
            HashSet::from([193, 393]),
            "half a beam under each storey's head"
        );
        let ids: HashSet<u32> = tied.parts.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), tied.parts.len(), "part ids are unique");
        // A group a storey, carrying nothing by gravity.
        let groups: Vec<&Group> = tied
            .groups
            .iter()
            .filter(|g| g.kind == GroupKind::Ties)
            .collect();
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|g| g.area_m2 == 0.0 && g.count == 10));
        // Round both storeys' heads: 2 × (10 + 5) m a storey, costed as beams in the walls' stage.
        let ties_m = 2.0 * 2.0 * (10.0 + 5.0);
        let walls = |e: &Expansion| e.stages[Stage::Walls.index()].clone();
        assert!(
            (walls(&tied).labour_h - walls(&loose).labour_h - r.beam_h_per_m * ties_m).abs() < 1e-9
        );
        let beam = f64::from(r.beam_cm) / 100.0;
        let kg = std::f64::consts::PI / 4.0 * beam * beam * ties_m * r.timber_kg_per_m3;
        assert!((walls(&tied).materials_kg[usize::from(TIMBER)] - kg).abs() < 1e-6);
        assert_eq!(walls(&loose).materials_kg[usize::from(TIMBER)], 0.0);
        // Refused: a timber frame, an unknown value, and before version 3.
        let mut wattle = tall(2);
        wattle.params[fp::TIES] = 1;
        assert_eq!(
            expand_frame(&wattle, &tall_house()),
            Err(GrammarError::OutOfRange(format!(
                "wall ties (parameter {}) bind mass walls; a timber frame is tied by its joints",
                fp::TIES
            )))
        );
        assert!(expand_frame(&rubble(2), &r).is_err());
        let mut old = rubble(1);
        old.version = 2;
        assert!(expand_frame(&old, &r).is_err());
    }

    #[test]
    fn a_stair_is_refused_by_name_where_it_has_no_place() {
        let r = tall_house();
        let mut odd = tall(3);
        odd.params[fp::STAIR] = 2;
        assert_eq!(
            expand_frame(&odd, &r),
            Err(GrammarError::OutOfRange(format!(
                "stair 2 (parameter {}) is unknown to frame version 3",
                fp::STAIR
            )))
        );
        let mut low = tall(3);
        low.storeys = 1;
        low.params[fp::JOIST_CM] = 0;
        assert_eq!(
            expand_frame(&low, &r),
            Err(GrammarError::OutOfRange(format!(
                "a stair (parameter {}) needs a storey above the first",
                fp::STAIR
            )))
        );
        let mut v2 = tall(2);
        v2.version = 2;
        assert_eq!(
            expand_frame(&v2, &r),
            Err(GrammarError::OutOfRange(format!(
                "parameter {} is unknown to frame version 2",
                fp::STAIR
            )))
        );
        // A stair runs further than one bay of 2.5 m is long.
        let mut short = tall(3);
        short.params[fp::BAYS] = 1;
        short.params[fp::DOOR] = fp::door(fp::SIDE_RIGHT, 0);
        if let Footprint::Rect { length, .. } = &mut short.footprint {
            *length = 250;
        }
        assert!(matches!(
            expand_frame(&short, &r),
            Err(GrammarError::OutOfRange(m)) if m.contains("more than the building's length")
        ));
    }

    #[test]
    fn version_three_golden_is_pinned() {
        // Three storeys by a stair: a change here is a change to what version 3 builds.
        let e = expand_frame(&tall(3), &tall_house()).expect("expands");
        assert_eq!(
            expansion_hash(&e),
            GOLDEN_V3_THREE_STOREYS,
            "{:#x}",
            expansion_hash(&e)
        );
    }

    #[test]
    fn a_spec_keeps_the_parameters_its_version_has() {
        let mut s = three_bays_one_loft();
        assert_eq!(s.params_in_use().len(), crate::PARAMS_BEFORE_V3);
        s.version = 2;
        assert_eq!(s.params_in_use().len(), crate::PARAMS_BEFORE_V3);
        s.version = 3;
        assert_eq!(s.params_in_use().len(), PARAMS);
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
        // A granary raised too low for its posts, or too high for its ladder to stay dry; nor on
        // the ground, when its program raises every floor.
        let g = granary();
        let mut low = raised_store();
        low.params[fp::FLOOR_RAISE_CM] = 30;
        assert!(expand_frame(&low, &g).is_err());
        let mut flat = raised_store();
        flat.params[fp::FLOOR_RAISE_CM] = 0;
        flat.params[fp::JOIST_CM] = 0;
        assert!(matches!(
            expand_frame(&flat, &g),
            Err(GrammarError::OutOfRange(m)) if m.contains("floor raise")
        ));
        let mut high = raised_store();
        high.params[fp::FLOOR_RAISE_CM] = 120;
        high.params[fp::OVERHANG_CM] = 40;
        assert!(expand_frame(&high, &g).is_err());
    }

    #[test]
    fn frame_version_2_mass_walls_and_tile_roof() {
        let mut s = spec(3, 250, 500);
        s.version = 2;
        s.params[fp::WALL_KIND] = 1;
        s.params[fp::FOOTING] = 1;
        s.materials[usize::from(COVERING)] = "core:good/roof_tile".into();
        let r = house();
        let e = expand_frame(&s, &r).expect("expands v2 mass wall");
        assert_eq!(e.version, 2);

        let count = |k: PartKind| e.parts.iter().filter(|p| p.kind == k).count();
        assert!(count(PartKind::StoneFooting) > 0);
        assert_eq!(count(PartKind::Posthole), 0);
        assert_eq!(count(PartKind::Post), 0);
        assert!(count(PartKind::MassWall) > 0);
        assert_eq!(count(PartKind::Wattle), 0);
        assert_eq!(count(PartKind::Daub), 0);
        assert_eq!(count(PartKind::Tile), 2);
        assert_eq!(count(PartKind::Thatch), 0);

        assert!(e.groups.iter().any(|g| g.kind == GroupKind::MassWall));
        assert!(!e.groups.iter().any(|g| g.kind == GroupKind::Posts));
        assert!(!e.groups.iter().any(|g| g.kind == GroupKind::Infill));

        let mw = e
            .groups
            .iter()
            .find(|g| g.kind == GroupKind::MassWall)
            .expect("mass wall group");
        assert_eq!(mw.spacing_cm, 1);

        let mut no_footing = s.clone();
        no_footing.params[fp::FOOTING] = 0;
        assert!(expand_frame(&no_footing, &r).is_err());

        let mut bad_wall = s.clone();
        bad_wall.params[fp::WALL_KIND] = 4;
        assert!(expand_frame(&bad_wall, &r).is_err());

        let mut bad_footing = s.clone();
        bad_footing.params[fp::FOOTING] = 2;
        assert!(expand_frame(&bad_footing, &r).is_err());
    }

    const GOLDEN_THREE_BAYS: u64 = 0xdb70_dcfc_e516_69b8;
    const GOLDEN_TWO_STOREYS: u64 = 0x7375_f0e6_7a98_0a0f;
    const GOLDEN_RAISED_STORE: u64 = 0xebfd_8b32_1aba_8021;
    const GOLDEN_WORKSHOP: u64 = 0xeadd_ad42_080a_e6ba;
    const GOLDEN_V3_THREE_STOREYS: u64 = 0xc325_5d0f_4a84_a224;
}
