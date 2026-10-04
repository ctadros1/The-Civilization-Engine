//! Building homes (plan §7 M1; ADR-0004 §2–3): the hut a household designs for itself, the ground
//! it claims for it, the work and materials still needed, and how pressing that work is. Whether
//! anyone builds, and when, is decided like every other activity ([`crate::decide`]); nothing here
//! makes anyone build.

use std::f64::consts::TAU;

use civ_core::time::DAYS_PER_YEAR;
use civ_grammar::{
    BuildingSpec, FRAME_VERSION, Footprint, HUT_VERSION, PARAMS, ProgramRules, Stage, StageNeeds,
    TURN, expand, frame_params, hut_params,
};
use civ_land::{Building, Land, RectCm};
use civ_world::nav::NavGrid;
use civ_world::{WATER_LAND, WorldMap};

use crate::params::{BuildingDef, GoodDef};

/// Least distance between a plot and another plot or a field, centimetres: room to walk between
/// them (a tuning value; the research gives no spacing).
pub const PLOT_GAP_CM: i32 = 100;
/// Least distance between a plot and the settlement's hearth, metres (a tuning value).
const HEARTH_GAP_M: f64 = 4.0;
/// Hours a household reckons each load of building material takes to cut and carry home: about
/// an hour's work and the walk there and back (a tuning value).
pub const HAUL_H_PER_LOAD: f64 = 1.5;
/// Share of its working hours a farming household reckons it can spare for building when it
/// weighs how pressing its roof is (a tuning value: fields, food, water and firewood take the
/// rest).
pub const BUILD_LABOUR_SHARE: f64 = 0.5;
/// Share of what a household could spare, in hours of its own work, that it puts into a larger
/// home than its members need: with goods in store it need not work for, it can give more time to
/// building (research 08-14 §2.3: households meet consumption and keep reserves before they
/// invest; §3.7 proposes 0.10, 0.30 and 0.60 as the share of discretionary resources offered to
/// feasible investments, to be varied. The middle one; a tuning value).
pub const HOUSE_INVESTMENT_SHARE: f64 = 0.3;
/// Share of the building time a household can give before its roof deadline
/// ([`BUILD_LABOUR_SHARE`]) that a larger home may take in all: nobody designs a hut it would have
/// to work at every spare hour to roof in time (a tuning value).
pub const HOUSE_TIME_SHARE: f64 = 0.5;
/// Floor area a new home must add to the home a household has before it builds again, a share
/// of the floor it has: nobody pulls down a sound home for a little more room (a tuning value;
/// research 10-06 gives none).
pub const REBUILD_GAIN: f64 = 0.25;
/// Step between the rings of points tried when a home's own ground is not clear, metres.
const SEARCH_STEP_M: f64 = 2.0;
/// Points tried on each ring.
const SEARCH_DIRECTIONS: usize = 16;

fn cm(m: f64) -> i32 {
    (m * 100.0).round() as i32
}

/// The direction from `from` to `to` in 1/65,536 of a turn from east toward south; east when they
/// are the same point.
fn direction(from: (f32, f32), to: (f32, f32)) -> i32 {
    let (dx, dy) = (f64::from(to.0 - from.0), f64::from(to.1 - from.1));
    if dx == 0.0 && dy == 0.0 {
        return 0;
    }
    ((dy.atan2(dx) / TAU * TURN).round() as i32).rem_euclid(TURN as i32)
}

/// The hut a household designs standing at `at` (metres), its wall line `radius` centimetres from
/// the centre ([`HutRules::radius_for`] for the floor its members need, or
/// [`radius_within`]), built to the hut program `def`'s usual wall height and pitch, its door
/// toward `toward` (the settlement's hearth), or facing east when there is none.
///
/// [`HutRules::radius_for`]: civ_grammar::HutRules::radius_for
pub fn design(
    def: &BuildingDef,
    goods: &[GoodDef],
    radius: i32,
    at: (f32, f32),
    toward: Option<(f32, f32)>,
) -> BuildingSpec {
    let mut params = [0; PARAMS];
    params[hut_params::EAVE_CM] = def.eave_cm;
    params[hut_params::PITCH_CENTIDEG] = def.pitch_centideg;
    params[hut_params::DOOR_DIR] = toward.map_or(0, |t| direction(at, t));
    BuildingSpec {
        program: def.id.clone(),
        version: HUT_VERSION,
        footprint: Footprint::Round {
            x: cm(f64::from(at.0)),
            y: cm(f64::from(at.1)),
            radius,
        },
        storeys: 1,
        params,
        materials: def
            .materials
            .iter()
            .map(|&g| goods.get(g).map_or_else(String::new, |g| g.id.clone()))
            .collect(),
        style_seed: 0,
    }
}

/// The centre of a building, metres.
pub fn centre_m(spec: &BuildingSpec) -> (f32, f32) {
    let (x, y) = spec.footprint.centre();
    ((f64::from(x) / 100.0) as f32, (f64::from(y) / 100.0) as f32)
}

/// Floor area of a building over every level, square metres, from its design alone (what its
/// expansion gives: a hut's circle, a frame's storeys and lofts, each to its wall lines).
pub fn floor_m2(spec: &BuildingSpec) -> f64 {
    match spec.footprint {
        Footprint::Round { radius, .. } => {
            std::f64::consts::PI * f64::from(radius) * f64::from(radius) * 1e-4
        }
        Footprint::Rect { length, width, .. } => {
            let bays = spec.params[frame_params::BAYS].max(1);
            let lofts = spec.params[frame_params::LOFT_BAYS].count_ones() as f64;
            let storey = f64::from(length) * f64::from(width);
            let bay = storey / f64::from(bays);
            (storey * f64::from(spec.storeys) + bay * lofts) * 1e-4
        }
    }
}

/// The ground a building's roof covers, which its household claims as its plot: the box round
/// the roof's edge (a hut's overhang is its program's; a frame's, its spec's).
pub fn plot_rect(spec: &BuildingSpec, def: &BuildingDef) -> RectCm {
    let roof = match spec.footprint {
        Footprint::Round { x, y, radius } => Footprint::Round {
            x,
            y,
            radius: radius + def.hut().map_or(0, |h| h.roof_overhang_cm.max(0)),
        },
        Footprint::Rect {
            x,
            y,
            length,
            width,
            angle,
        } => {
            let o = spec.params[frame_params::OVERHANG_CM].max(0);
            Footprint::Rect {
                x,
                y,
                length: length + 2 * o,
                width: width + 2 * o,
                angle,
            }
        }
    };
    let (x, y) = spec.footprint.centre();
    let [x0, y0, x1, y1] = roof.bounds().unwrap_or([x, y, x, y]);
    RectCm {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// Whether ground can be claimed as a plot: inside the map, every cell under it dry walkable
/// land, and clear of every field and plot.
pub fn plot_clear(land: &Land, map: &WorldMap, nav: &NavGrid, rect: &RectCm) -> bool {
    let cell_cm = cm(f64::from(map.cell_size_m)).max(1);
    let (map_w, map_h) = (map.width as i32 * cell_cm, map.height as i32 * cell_cm);
    if rect.w <= 0
        || rect.h <= 0
        || rect.x < 0
        || rect.y < 0
        || rect.x + rect.w > map_w
        || rect.y + rect.h > map_h
    {
        return false;
    }
    if land.fields.iter().any(|f| f.rect.near(rect, PLOT_GAP_CM))
        || land.plots.iter().any(|p| p.rect.near(rect, PLOT_GAP_CM))
    {
        return false;
    }
    let w = map.width as usize;
    let (x0, y0) = (rect.x / cell_cm, rect.y / cell_cm);
    let (x1, y1) = (
        (rect.x + rect.w - 1) / cell_cm,
        (rect.y + rect.h - 1) / cell_cm,
    );
    (y0..=y1).all(|y| {
        (x0..=x1).all(|x| {
            let c = y as usize * w + x as usize;
            map.water[c] == WATER_LAND && nav.walkable(c)
        })
    })
}

/// Where a household builds: the hut `design_at` designs at its home if the ground it needs there
/// is clear, otherwise at the nearest point found stepping outward from home (away from the
/// hearth first) no farther than `max_m`. Never on the hearth. `design_at` returns `None` for a
/// point the household would not build on (out of reach). `None` if no ground is clear.
#[allow(clippy::too_many_arguments)]
pub fn home_site(
    land: &Land,
    map: &WorldMap,
    nav: &NavGrid,
    def: &BuildingDef,
    design_at: &dyn Fn((f32, f32)) -> Option<BuildingSpec>,
    home: (f32, f32),
    hearth: Option<(f32, f32)>,
    max_m: f64,
) -> Option<BuildingSpec> {
    let away = hearth.map_or(0.0, |h| {
        f64::from(home.1 - h.1).atan2(f64::from(home.0 - h.0))
    });
    let clear_of_hearth = |rect: &RectCm| {
        hearth.is_none_or(|(hx, hy)| {
            let (x, y) = (cm(f64::from(hx)), cm(f64::from(hy)));
            let dx = (rect.x - x).max(x - (rect.x + rect.w)).max(0);
            let dy = (rect.y - y).max(y - (rect.y + rect.h)).max(0);
            f64::from(dx).hypot(f64::from(dy)) >= HEARTH_GAP_M * 100.0
        })
    };
    let rings = (max_m / SEARCH_STEP_M).floor().max(0.0) as usize;
    let around = (1..=rings).flat_map(|ring| {
        let d = ring as f64 * SEARCH_STEP_M;
        (0..SEARCH_DIRECTIONS).map(move |k| {
            // Either side of the direction away from the hearth in turn: 0, +1, -1, +2, ...
            let side = k.div_ceil(2) as f64 * if k % 2 == 1 { 1.0 } else { -1.0 };
            let a = away + side * TAU / SEARCH_DIRECTIONS as f64;
            (home.0 + (d * a.cos()) as f32, home.1 + (d * a.sin()) as f32)
        })
    });
    std::iter::once(home).chain(around).find_map(|at| {
        let spec = design_at(at)?;
        let rect = plot_rect(&spec, def);
        (clear_of_hearth(&rect) && plot_clear(land, map, nav, &rect)).then_some(spec)
    })
}

/// What each construction stage of the building `spec` designs needs; `None` if the design does not
/// expand under `def`'s rules.
pub fn stage_needs(spec: &BuildingSpec, def: &BuildingDef) -> Option<Vec<StageNeeds>> {
    expand(spec, &def.rules).ok().map(|e| e.stages)
}

/// Hours of work the building `spec` designs takes in all: building it through every stage and
/// bringing what it is built of, `carry_kg` a load ([`HAUL_H_PER_LOAD`]). `None` if the design
/// does not expand under `def`'s rules.
pub fn building_hours(spec: &BuildingSpec, def: &BuildingDef, carry_kg: f64) -> Option<f64> {
    let e = expand(spec, &def.rules).ok()?;
    let loads = e.total_materials_kg().iter().sum::<f64>() / carry_kg.max(1e-6);
    Some(e.total_labour_h() + loads * HAUL_H_PER_LOAD)
}

/// The wall-line radius of the hut a household of `residents` designs, centimetres: the floor its
/// members need ([`HutRules::radius_for`]), made larger a decimetre at a time, up to the largest
/// the rules allow, while the extra work stays within `budget_h` hours (what the household puts
/// into its home of the goods it could spare, [`HOUSE_INVESTMENT_SHARE`]) and the whole hut
/// within `time_h` hours (the share of the building time it can give before its roof deadline
/// that a home may take, [`HOUSE_TIME_SHARE`]). Never smaller than its members need; `None` if
/// `def` is not a hut program.
///
/// [`HutRules::radius_for`]: civ_grammar::HutRules::radius_for
pub fn radius_within(
    def: &BuildingDef,
    goods: &[GoodDef],
    residents: usize,
    carry_kg: f64,
    budget_h: f64,
    time_h: f64,
) -> Option<i32> {
    let hut = def.hut()?;
    let need = hut.radius_for(residents);
    let mut spec = design(def, goods, need, (0.0, 0.0), None);
    let hours = |spec: &BuildingSpec| building_hours(spec, def, carry_kg);
    let Some(base) = hours(&spec) else {
        return Some(need);
    };
    let mut best = need;
    while best + 10 <= hut.radius_cm.1 {
        spec.footprint = Footprint::Round {
            x: 0,
            y: 0,
            radius: best + 10,
        };
        match hours(&spec) {
            Some(h) if h - base <= budget_h && h <= time_h => best += 10,
            _ => break,
        }
    }
    Some(best)
}

/// The wall-line radius of the new home a household of `residents` living in a hut of radius
/// `current` would build, centimetres, if any: the largest whose work its means pay for in all
/// (`budget_h` hours: while it has a home, all of a new one is beyond its needs) and that takes
/// no more than `time_h` hours, among those at least [`REBUILD_GAIN`] larger in floor area than
/// the home it has and as large as its members need. `None` when it cannot afford the least of
/// them, or `def` is not a hut program.
pub fn rebuild_radius(
    def: &BuildingDef,
    goods: &[GoodDef],
    residents: usize,
    current: i32,
    carry_kg: f64,
    budget_h: f64,
    time_h: f64,
) -> Option<i32> {
    let hut = def.hut()?;
    let larger =
        (f64::from(current.max(0)) * (1.0 + REBUILD_GAIN).sqrt() / 10.0).ceil() as i32 * 10;
    let mut r = larger.max(hut.radius_for(residents));
    let mut spec = design(def, goods, r, (0.0, 0.0), None);
    let mut best = None;
    while r <= hut.radius_cm.1 {
        spec.footprint = Footprint::Round {
            x: 0,
            y: 0,
            radius: r,
        };
        match building_hours(&spec, def, carry_kg) {
            Some(h) if h <= budget_h && h <= time_h => best = Some(r),
            _ => break,
        }
        r += 10;
    }
    best
}

/// The shape a building of a program takes (ADR-0009 §1): how large a hut is, or how many bays,
/// storeys and lofts a frame building has (its other sizes are the program's
/// [`FrameDesign`](crate::params::FrameDesign)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    /// A hut whose wall line is `radius` centimetres from its centre.
    Round {
        /// Radius of the wall line, centimetres.
        radius: i32,
    },
    /// A frame building of `bays` bays and `storeys` storeys, with lofts over its first `lofts`
    /// bays from the end its length runs from.
    Bays {
        /// Bays.
        bays: i32,
        /// Storeys.
        storeys: u8,
        /// Bays floored as lofts.
        lofts: i32,
    },
}

/// What a shape of a program gives and costs. Neither depends on where it stands or which way it
/// faces, so each program's shapes are expanded once, when the content is compiled.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeCost {
    /// The shape.
    pub shape: Shape,
    /// Person-hours of a capable adult to build it, every stage.
    pub labour_h: f64,
    /// Kilograms of every material it is built of.
    pub materials_kg: f64,
    /// Floor area over every level, square metres.
    pub floor_m2: f64,
    /// Living floor, square metres.
    pub living_m2: f64,
    /// Goods it holds under its roof, kilograms.
    pub storage_kg: f64,
}

impl ShapeCost {
    /// Hours of work it takes in all: building it and bringing what it is built of, `carry_kg` a
    /// load ([`HAUL_H_PER_LOAD`]).
    pub fn hours(&self, carry_kg: f64) -> f64 {
        self.labour_h + self.materials_kg / carry_kg.max(1e-6) * HAUL_H_PER_LOAD
    }
}

/// The design of a building of program `def` in shape `shape` standing at `at` (metres), its
/// door toward `toward` (the settlement's hearth), or facing east when there is none. A hut is
/// built to the program's usual wall height and pitch; a frame building to its usual sizes, its
/// door in the middle of the long wall that faces `toward` and its lofts from the back. `None`
/// when the shape is not the program's kind.
pub fn design_shape(
    def: &BuildingDef,
    goods: &[GoodDef],
    shape: Shape,
    at: (f32, f32),
    toward: Option<(f32, f32)>,
) -> Option<BuildingSpec> {
    match (shape, &def.rules) {
        (Shape::Round { radius }, ProgramRules::Hut(_)) => {
            Some(design(def, goods, radius, at, toward))
        }
        (
            Shape::Bays {
                bays,
                storeys,
                lofts,
            },
            ProgramRules::Frame(_),
        ) => {
            use frame_params as fp;
            let d = def.design?;
            let mut params = [0; PARAMS];
            params[fp::EAVE_CM] = def.eave_cm;
            params[fp::PITCH_CENTIDEG] = def.pitch_centideg;
            params[fp::BAYS] = bays;
            params[fp::DOOR] = fp::door(fp::SIDE_RIGHT, bays / 2);
            params[fp::LOFT_BAYS] = (1 << lofts.clamp(0, bays)) - 1;
            if lofts > 0 || storeys > 1 {
                params[fp::JOIST_CM] = d.joist_cm;
            }
            params[fp::OVERHANG_CM] = d.overhang_cm;
            params[fp::POST_CM] = d.post_cm;
            params[fp::WALL_CM] = d.wall_cm;
            // The door's wall faces `toward`: it faces a quarter turn on from the length.
            let facing = toward.map_or(0, |t| direction(at, t));
            let angle = (facing - TURN as i32 / 4).rem_euclid(TURN as i32) as u16;
            Some(BuildingSpec {
                program: def.id.clone(),
                version: FRAME_VERSION,
                footprint: Footprint::Rect {
                    x: cm(f64::from(at.0)),
                    y: cm(f64::from(at.1)),
                    length: bays * d.bay_cm,
                    width: d.width_cm,
                    angle,
                },
                storeys,
                params,
                materials: def
                    .materials
                    .iter()
                    .map(|&g| goods.get(g).map_or_else(String::new, |g| g.id.clone()))
                    .collect(),
                style_seed: 0,
            })
        }
        _ => None,
    }
}

/// Every shape people would build program `def` in, with what each gives and costs: a hut of
/// each radius its rules allow, a decimetre apart; a frame building of each number of bays and
/// storeys its rules allow, with lofts over none to all of its bays where it may have them.
/// Shapes that do not expand are left out.
pub fn shapes(def: &BuildingDef, goods: &[GoodDef]) -> Vec<ShapeCost> {
    let mut candidates = Vec::new();
    match &def.rules {
        ProgramRules::Hut(h) => {
            let mut radius = h.radius_cm.0;
            while radius <= h.radius_cm.1 {
                candidates.push(Shape::Round { radius });
                radius += 10;
            }
        }
        ProgramRules::Frame(f) => {
            for storeys in f.storeys.0..=f.storeys.1 {
                for bays in f.bays.0..=f.bays.1 {
                    for lofts in 0..=(if f.lofts { bays } else { 0 }) {
                        candidates.push(Shape::Bays {
                            bays,
                            storeys,
                            lofts,
                        });
                    }
                }
            }
        }
    }
    candidates
        .into_iter()
        .filter_map(|shape| {
            let spec = design_shape(def, goods, shape, (0.0, 0.0), None)?;
            let e = expand(&spec, &def.rules).ok()?;
            Some(ShapeCost {
                shape,
                labour_h: e.total_labour_h(),
                materials_kg: e.total_materials_kg().iter().sum(),
                floor_m2: e.floor_area_m2,
                living_m2: e.floor_by_use[civ_grammar::SpaceUse::Living.index()],
                storage_kg: e.storage_total_kg(),
            })
        })
        .collect()
}

/// What a household needs of a home: room for its members to live, and for its goods under the
/// roof (ADR-0009 §1; research 10-06 §2.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HomeNeed {
    /// Its members.
    pub residents: usize,
    /// The goods it keeps under a roof, kilograms.
    pub storage_kg: f64,
}

/// The living floor program `def` gives `residents` people, square metres: what a household
/// needs whatever its size and what each resident adds (research 10-06 §2.3).
fn living_need_m2(def: &BuildingDef, residents: usize) -> f64 {
    let (base, per) = match &def.rules {
        ProgramRules::Hut(h) => (h.floor_base_m2, h.floor_m2_per_sleeper),
        ProgramRules::Frame(f) => (f.floor_base_m2, f.floor_m2_per_sleeper),
    };
    base + per * residents as f64
}

/// How well shape `c` of program `def` covers `need`: 2 for its members and its goods, 1 for its
/// members only, 0 for neither.
fn coverage(def: &BuildingDef, c: &ShapeCost, need: &HomeNeed) -> u8 {
    let living = c.living_m2 + 1e-9 >= living_need_m2(def, need.residents);
    let goods = c.storage_kg + 1e-9 >= need.storage_kg;
    match (living, goods) {
        (true, true) => 2,
        (true, false) => 1,
        _ => 0,
    }
}

/// Whether shape `c` (`hours` of work, covering `need` this well) is a better first home than
/// `b`: one that covers more of the need; among those covering it all, the cheaper; among those
/// covering only its members, the one holding more goods, then the cheaper; otherwise the one
/// with more living floor, then the cheaper.
fn better_base(c: (&ShapeCost, u8, f64), b: (&ShapeCost, u8, f64)) -> bool {
    let ((c, c_cov, c_h), (b, b_cov, b_h)) = (c, b);
    if c_cov != b_cov {
        return c_cov > b_cov;
    }
    let near = |x: f64, y: f64| (x - y).abs() <= 1e-9;
    match c_cov {
        2 => c_h < b_h,
        1 => c.storage_kg > b.storage_kg + 1e-9 || (near(c.storage_kg, b.storage_kg) && c_h < b_h),
        _ => c.living_m2 > b.living_m2 + 1e-9 || (near(c.living_m2, b.living_m2) && c_h < b_h),
    }
}

/// The first home a household designs, as a program (an index into `buildings`) and a shape.
/// Among the programs `programs` it may build, the shape that covers `need` for the fewest hours
/// of work (`carry_kg` a load); if none can hold all its goods, the one covering its members that
/// holds the most of them, and if none covers its members, the one with the most living floor
/// (see `better_base`). Then, within
/// that program, the shape with the most floor whose extra hours over that one stay within
/// `budget_h` (what it puts into its home of the goods it could spare,
/// [`HOUSE_INVESTMENT_SHARE`]) and whose whole stays within `time_h` (the share of the building
/// time it can give before its roof deadline that a home may take, [`HOUSE_TIME_SHARE`]): never
/// less than the first, and more means never give less floor. `None` if it may build none.
pub fn first_home(
    buildings: &[BuildingDef],
    programs: &[usize],
    need: HomeNeed,
    carry_kg: f64,
    budget_h: f64,
    time_h: f64,
) -> Option<(usize, Shape)> {
    let mut base: Option<(usize, &ShapeCost, u8, f64)> = None;
    for &p in programs {
        let Some(def) = buildings.get(p) else {
            continue;
        };
        for c in &def.shapes {
            let (cov, h) = (coverage(def, c, &need), c.hours(carry_kg));
            if base.is_none_or(|(_, b, b_cov, b_h)| better_base((c, cov, h), (b, b_cov, b_h))) {
                base = Some((p, c, cov, h));
            }
        }
    }
    let (p, b, cov, b_h) = base?;
    let def = &buildings[p];
    let mut best = (b, b_h);
    for c in &def.shapes {
        let h = c.hours(carry_kg);
        let affordable = h - b_h <= budget_h && h <= time_h;
        if affordable
            && coverage(def, c, &need) >= cov
            && (c.floor_m2 > best.0.floor_m2 + 1e-9
                || ((c.floor_m2 - best.0.floor_m2).abs() <= 1e-9 && h < best.1))
            && c.floor_m2 + 1e-9 >= b.floor_m2
        {
            best = (c, h);
        }
    }
    Some((p, best.0.shape))
}

/// The new home a household living in a home of `current_m2` square metres of floor would build,
/// if any: among the programs `programs` it may build, the shape with the most floor whose work
/// its means pay for in all (`budget_h` hours: while it has a home, all of a new one is beyond its
/// needs) and that takes no more than `time_h` hours, among those at least [`REBUILD_GAIN`]
/// larger in floor than the home it has and with living floor for its members (room for its
/// goods comes with the floor). `None` when it can afford none.
pub fn new_home(
    buildings: &[BuildingDef],
    programs: &[usize],
    need: HomeNeed,
    current_m2: f64,
    carry_kg: f64,
    budget_h: f64,
    time_h: f64,
) -> Option<(usize, Shape)> {
    let least = current_m2.max(0.0) * (1.0 + REBUILD_GAIN);
    let mut best: Option<(usize, &ShapeCost, f64)> = None;
    for &p in programs {
        let Some(def) = buildings.get(p) else {
            continue;
        };
        for c in &def.shapes {
            let h = c.hours(carry_kg);
            if c.floor_m2 + 1e-9 < least
                || coverage(def, c, &need) == 0
                || !(h <= budget_h && h <= time_h)
            {
                continue;
            }
            let better = best.is_none_or(|(_, b, b_h)| {
                c.floor_m2 > b.floor_m2 + 1e-9
                    || ((c.floor_m2 - b.floor_m2).abs() <= 1e-9 && h < b_h)
            });
            if better {
                best = Some((p, c, h));
            }
        }
    }
    best.map(|(p, c, _)| (p, c.shape))
}

/// Work on a building: what each of its stages needs, the stage under way and the hours done on
/// it.
#[derive(Clone, Debug, PartialEq)]
pub struct HomeWork {
    /// What each stage needs, in order ([`stage_needs`]).
    pub stages: Vec<StageNeeds>,
    /// The stage under way, an index into `stages` (their number once finished).
    pub stage: usize,
    /// Hours of a capable adult's work done on the stage under way.
    pub done_h: f64,
}

impl HomeWork {
    /// The work on `building`, whose stages need `stages`.
    pub fn of(building: &Building, stages: Vec<StageNeeds>) -> HomeWork {
        HomeWork {
            stages,
            stage: usize::from(building.stage),
            done_h: f64::from(building.work_h),
        }
    }

    /// The work on a building not yet begun, whose stages need `stages`.
    pub fn begin(stages: Vec<StageNeeds>) -> HomeWork {
        HomeWork {
            stages,
            stage: 0,
            done_h: 0.0,
        }
    }

    /// The labour and materials of the stage under way; `None` once every stage is done.
    pub fn needs(&self) -> Option<&StageNeeds> {
        self.stages.get(self.stage)
    }

    /// The share of stage `i` still to do. A stage that needs no labour is done at once and uses
    /// nothing (crate `civ-land`, [`Building::work`]).
    fn left(&self, i: usize, s: &StageNeeds) -> f64 {
        if i < self.stage || s.labour_h <= 0.0 {
            0.0
        } else if i > self.stage {
            1.0
        } else {
            (1.0 - self.done_h / s.labour_h).clamp(0.0, 1.0)
        }
    }

    /// Hours of a capable adult's work left through stage `last`.
    pub fn hours_left(&self, last: Stage) -> f64 {
        self.stages
            .iter()
            .enumerate()
            .take(last.index() + 1)
            .map(|(i, s)| s.labour_h * self.left(i, s))
            .sum()
    }

    /// Kilograms of each material slot still to be built in through stage `last`: a stage uses its
    /// materials in proportion to its work.
    pub fn materials_left(&self, last: Stage) -> Vec<f64> {
        let slots = self
            .stages
            .iter()
            .map(|s| s.materials_kg.len())
            .max()
            .unwrap_or(0);
        let mut out = vec![0.0; slots];
        for (i, s) in self.stages.iter().enumerate().take(last.index() + 1) {
            let share = self.left(i, s);
            for (o, kg) in out.iter_mut().zip(&s.materials_kg) {
                *o += kg.max(0.0) * share;
            }
        }
        out
    }

    /// Kilograms of each material slot at hand for the stage under way, from `stores` (by good). A
    /// good that fills more than one slot the stage needs is shared between them by need.
    pub fn held_by_slot(&self, def: &BuildingDef, stores: &[f64]) -> Vec<f64> {
        let Some(needs) = self.needs() else {
            return Vec::new();
        };
        let slots = &needs.materials_kg;
        slots
            .iter()
            .enumerate()
            .map(|(slot, need)| {
                let Some(&g) = def.materials.get(slot) else {
                    return 0.0;
                };
                let have = stores.get(g).copied().unwrap_or(0.0).max(0.0);
                let total: f64 = slots
                    .iter()
                    .enumerate()
                    .filter(|(t, _)| def.materials.get(*t) == Some(&g))
                    .map(|(_, kg)| kg.max(0.0))
                    .sum();
                if total > 0.0 {
                    have * need.max(0.0) / total
                } else {
                    have
                }
            })
            .collect()
    }

    /// Hours of the stage under way that `held_by_slot` kilograms of each slot's material allow.
    pub fn workable_h(&self, held_by_slot: &[f64]) -> f64 {
        self.needs().map_or(0.0, |s| {
            civ_land::workable_h(s.labour_h, self.done_h, &s.materials_kg, held_by_slot)
        })
    }

    /// Kilograms of each good (by index; `goods` of them) the household still has to bring for
    /// the work through stage `last`, beyond what `stores` holds.
    pub fn need_by_good(
        &self,
        def: &BuildingDef,
        last: Stage,
        stores: &[f64],
        goods: usize,
    ) -> Vec<f64> {
        let mut need = vec![0.0; goods];
        for (slot, kg) in self.materials_left(last).into_iter().enumerate() {
            if let Some(n) = def.materials.get(slot).and_then(|&g| need.get_mut(g)) {
                *n += kg;
            }
        }
        for (n, held) in need.iter_mut().zip(stores) {
            *n = (*n - held.max(0.0)).max(0.0);
        }
        // What the scraps about the site make up is not worth a trip.
        for n in &mut need {
            if *n <= civ_land::MATERIAL_SLACK_KG {
                *n = 0.0;
            }
        }
        need
    }
}

/// The day (a day index) a household that began building on day `since` wants to be under its
/// roof by: the first `roof_by_day` of a year on or after `since`.
pub fn roof_deadline(since: i64, roof_by_day: u16) -> i64 {
    let this_year = since - civ_land::day_of_year(since) + i64::from(roof_by_day);
    if this_year >= since {
        this_year
    } else {
        this_year + DAYS_PER_YEAR
    }
}

/// How pressing building is on day `day`: the hours of work left until the roof is on (building,
/// and bringing what it is built of) over the hours a household giving `labour_per_day` can spare
/// for it ([`BUILD_LABOUR_SHARE`]) before `deadline`. Once the deadline has passed, what is left
/// over one day's work.
pub fn urgency(hours_left: f64, deadline: i64, labour_per_day: f64, day: i64) -> f64 {
    let days = (deadline - day).max(1) as f64;
    hours_left.max(0.0) / ((labour_per_day * BUILD_LABOUR_SHARE).max(1e-6) * days)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::GoodUse;
    use civ_core::{PermanentId, SimTime};
    use civ_grammar::{FRAME_VERSION, HutRules, ProgramRules, hut_materials};
    use civ_land::PlotUse;

    /// The core hut's figures (content `core:building/hut`).
    fn hut_rules() -> HutRules {
        HutRules {
            radius_cm: (150, 400),
            eave_cm: (150, 250),
            pitch_centideg: (4500, 5500),
            post_spacing_cm: 100,
            post_diameter_cm: 12,
            posthole_depth_cm: 60,
            wall_thickness_cm: 15,
            roof_overhang_cm: 50,
            thatch_thickness_cm: 30,
            hearth_cm: 80,
            floor_base_m2: 6.0,
            floor_m2_per_sleeper: 4.8,
            groundwork_h_per_m2: 3.0,
            posthole_h: 0.5,
            post_h: 2.0,
            rafter_h: 1.6,
            wattle_h_per_m2: 1.0,
            daub_h_per_m2: 2.0,
            thatch_h_per_m2: 2.4,
            finish_h_per_m2: 3.0,
            post_kg: 22.0,
            rafter_kg: 21.0,
            wattle_kg_per_m2: 6.0,
            thatch_kg_per_m2: 30.0,
            store_kg_per_m2: 60.0,
        }
    }

    /// The core hut program.
    fn def() -> BuildingDef {
        BuildingDef {
            id: "core:building/hut".into(),
            name: "Hut".into(),
            use_: PlotUse::Dwelling,
            rules: ProgramRules::Hut(hut_rules()),
            // Posts, rafters and wattle are all timber; the roof is thatch.
            materials: vec![0, 0, 1],
            eave_cm: 180,
            pitch_centideg: 4500,
            roof_by_day: 304,
            technique: None,
            design: None,
            shapes: Vec::new(),
        }
    }

    fn goods() -> Vec<GoodDef> {
        let good = |id: &str| GoodDef {
            id: id.into(),
            name: id.into(),
            purpose: GoodUse::Material,
            kcal_per_kg: 0.0,
            half_life_days: 0.0,
            sheltered_half_life_days: 0.0,
            eaten: crate::params::Eaten::Never,
            shared: false,
            reserve_for: None,
            tool: None,
        };
        vec![good("core:good/timber"), good("core:good/thatch")]
    }

    #[test]
    fn a_hut_for_five_faces_the_hearth_and_claims_the_ground_its_roof_covers() {
        let (def, goods) = (def(), goods());
        // The hearth lies 20 m south of home.
        let spec = design(
            &def,
            &goods,
            hut_rules().radius_for(5),
            (100.0, 100.0),
            Some((100.0, 120.0)),
        );
        // 6 + 4.8 · 5 = 30 m² of floor: a radius of 3.09 m, rounded up to 3.1 m.
        assert_eq!(
            spec.footprint,
            Footprint::Round {
                x: 10_000,
                y: 10_000,
                radius: 310
            }
        );
        // A quarter turn from east toward south.
        assert_eq!(spec.params[hut_params::DOOR_DIR], 16_384);
        assert_eq!(spec.params[hut_params::EAVE_CM], 180);
        assert_eq!(
            spec.materials,
            vec!["core:good/timber", "core:good/timber", "core:good/thatch"]
        );
        assert_eq!(centre_m(&spec), (100.0, 100.0));
        // The roof reaches 50 cm beyond the wall line: a plot 7.2 m square.
        assert_eq!(
            plot_rect(&spec, &def),
            RectCm {
                x: 9_640,
                y: 9_640,
                w: 720,
                h: 720
            }
        );
        // Without a hearth the door faces east; the design expands under the hut's rules.
        let lone = design(
            &def,
            &goods,
            hut_rules().radius_for(5),
            (100.0, 100.0),
            None,
        );
        assert_eq!(lone.params[hut_params::DOOR_DIR], 0);
        let e = expand(&spec, &def.rules).expect("a valid hut");
        assert!(e.sleeping_places >= 5, "{}", e.sleeping_places);
    }

    #[test]
    fn a_frame_building_claims_the_box_round_its_roof() {
        let def = def();
        let mut params = [0; PARAMS];
        params[frame_params::OVERHANG_CM] = 50;
        let at = |angle: u16| BuildingSpec {
            program: "core:building/longhouse".into(),
            version: FRAME_VERSION,
            footprint: Footprint::Rect {
                x: 10_000,
                y: 20_000,
                length: 750,
                width: 500,
                angle,
            },
            storeys: 1,
            params,
            materials: Vec::new(),
            style_seed: 0,
        };
        // Along the east-west axis: the roof is 8.5 m by 6 m.
        assert_eq!(
            plot_rect(&at(0), &def),
            RectCm {
                x: 10_000 - 425,
                y: 20_000 - 300,
                w: 850,
                h: 600
            }
        );
        // Turned a quarter, north to south.
        assert_eq!(
            plot_rect(&at(16_384), &def),
            RectCm {
                x: 10_000 - 300,
                y: 20_000 - 425,
                w: 600,
                h: 850
            }
        );
        // At an eighth of a turn the box is wider than the roof: (4.25 + 3) / √2 each way.
        let r = plot_rect(&at(8_192), &def);
        assert_eq!((r.w, r.h), (1_026, 1_026));
        assert_eq!(centre_m(&at(8_192)), (100.0, 200.0));
    }

    #[test]
    fn goods_to_spare_buy_a_larger_hut_within_the_time_before_winter() {
        let (def, goods) = (def(), goods());
        let (need, most) = (hut_rules().radius_for(5), hut_rules().radius_cm.1);
        let carry = 20.0;
        let hours = |r: i32| {
            building_hours(&design(&def, &goods, r, (0.0, 0.0), None), &def, carry)
                .expect("expands")
        };
        let within = |budget: f64, time: f64| {
            radius_within(&def, &goods, 5, carry, budget, time).expect("a hut program")
        };
        // Nothing to spare: the hut its members need.
        assert_eq!(within(0.0, f64::INFINITY), need);
        // Ample means and time: the largest the rules allow.
        assert_eq!(within(1e9, 1e9), most);
        // In between, the largest whose extra work its means cover.
        let budget = 150.0;
        let r = within(budget, 1e9);
        assert!(r > need && r < most, "{r}");
        assert!(hours(r) - hours(need) <= budget);
        assert!(hours(r + 10) - hours(need) > budget);
        // More to spare never makes a hut smaller.
        let mut last = need;
        for b in [0.0, 50.0, 100.0, 200.0, 400.0, 800.0] {
            let r = within(b, 1e9);
            assert!(r >= last, "{b} h: {r} after {last}");
            last = r;
        }
        // Short of time, the hut its members need whatever it could spare, and never smaller.
        assert_eq!(within(1e9, hours(need) - 1.0), need);
        let tight = within(1e9, hours(need) + 100.0);
        assert!(
            tight > need && hours(tight) <= hours(need) + 100.0,
            "{tight}"
        );
        // A household that needs the largest hut builds it already.
        assert_eq!(radius_within(&def, &goods, 12, carry, 0.0, 1e9), Some(most));
        // Nonsense means change nothing.
        assert_eq!(within(f64::NAN, 1e9), need);
        assert_eq!(within(-5.0, 1e9), need);
    }

    #[test]
    fn a_household_with_a_home_builds_again_only_for_a_markedly_larger_one_it_can_pay_for() {
        let (def, goods) = (def(), goods());
        let carry = 20.0;
        let hours = |r: i32| {
            building_hours(&design(&def, &goods, r, (0.0, 0.0), None), &def, carry)
                .expect("expands")
        };
        let rebuild = |current: i32, budget: f64, time: f64| {
            rebuild_radius(&def, &goods, 5, current, carry, budget, time)
        };
        let need = hut_rules().radius_for(5);
        // At least a quarter more floor: 310 cm grows to 350 (√1.25 · 310 = 346.6, rounded up).
        let least = 350;
        assert_eq!(rebuild(need, hours(least), 1e9), Some(least));
        // All of the new hut must be paid for, not just what it adds.
        assert_eq!(rebuild(need, hours(least) - 1.0, 1e9), None);
        assert_eq!(rebuild(need, hours(least) - hours(need), 1e9), None);
        // More means build larger, up to the largest the rules allow.
        let r = rebuild(need, hours(least + 30), 1e9).expect("affordable");
        assert_eq!(r, least + 30);
        assert_eq!(rebuild(need, 1e9, 1e9), Some(hut_rules().radius_cm.1));
        // Not without the time, and never from the largest.
        assert_eq!(rebuild(need, 1e9, hours(least) - 1.0), None);
        assert_eq!(rebuild(hut_rules().radius_cm.1, 1e9, 1e9), None);
        // A crowded household builds at least what its members need now.
        let small = 200;
        let crowded = rebuild(small, 1e9, 1e9).expect("affordable");
        assert!(crowded >= need, "{crowded}");
        assert_eq!(
            rebuild_radius(&def, &goods, 5, small, carry, hours(need), 1e9),
            Some(need),
            "the least it would build is what its members need"
        );
    }

    #[test]
    fn the_work_left_counts_down_through_the_stages() {
        let (def, goods) = (def(), goods());
        let spec = design(
            &def,
            &goods,
            hut_rules().radius_for(5),
            (100.0, 100.0),
            None,
        );
        let e = expand(&spec, &def.rules).expect("expands");
        let mut work = HomeWork::begin(stage_needs(&spec, &def).expect("expands"));
        let roof: f64 = e.stages[..=Stage::Roof.index()]
            .iter()
            .map(|s| s.labour_h)
            .sum();
        assert!((work.hours_left(Stage::Roof) - roof).abs() < 1e-9);
        assert!((work.hours_left(Stage::Finish) - e.total_labour_h()).abs() < 1e-9);
        assert_eq!(work.materials_left(Stage::Finish), e.total_materials_kg());
        // The foundation needs no material, so all of it can be done with nothing at hand.
        let held = work.held_by_slot(&def, &[0.0, 0.0]);
        assert_eq!(work.workable_h(&held), e.stages[0].labour_h);
        // Halfway through the frame, half its timber is still to come; the rest of the timber
        // is wattle.
        work.stage = Stage::Frame.index();
        let frame = &e.stages[Stage::Frame.index()];
        work.done_h = frame.labour_h / 2.0;
        let left = work.materials_left(Stage::Roof);
        let wattle =
            e.stages[Stage::Walls.index()].materials_kg[usize::from(hut_materials::WATTLE)];
        let timber = frame.materials_kg[usize::from(hut_materials::TIMBER)];
        assert!((left[0] - timber / 2.0).abs() < 1e-9);
        assert!((left[1] - wattle).abs() < 1e-9);
        // By good: timber for the frame's other half and the walls, less the 100 kg in store.
        let need = work.need_by_good(&def, Stage::Roof, &[100.0, 0.0], 2);
        assert!((need[0] - (timber / 2.0 + wattle - 100.0)).abs() < 1e-9);
        // The frame needs only the timber slot: all the timber at hand is there for it, and it
        // allows as many hours as it has wood for.
        let held = work.held_by_slot(&def, &[100.0, 0.0]);
        assert_eq!(held, vec![100.0, 0.0, 0.0]);
        let per_hour = timber / frame.labour_h;
        assert!((work.workable_h(&held) - 100.0 / per_hour).abs() < 1e-9);
        // The household's own building gives the same figures.
        let b = Building {
            id: PermanentId::from_raw(3).expect("non-zero"),
            household: PermanentId::from_raw(2).expect("non-zero"),
            plot: PermanentId::from_raw(4).expect("non-zero"),
            spec,
            stage: Stage::Frame.index() as u8,
            work_h: work.done_h as f32,
            started: SimTime::ZERO,
            stage_since: SimTime::ZERO,
        };
        let of = HomeWork::of(&b, e.stages.clone());
        assert!((of.hours_left(Stage::Roof) - work.hours_left(Stage::Roof)).abs() < 1e-3);
        // Finished, nothing is left and nothing can be done.
        work.stage = Stage::ALL.len();
        assert_eq!(work.hours_left(Stage::Finish), 0.0);
        assert_eq!(work.workable_h(&[1e9; 3]), 0.0);
    }

    #[test]
    fn the_roof_is_wanted_before_winter_and_presses_harder_as_it_nears() {
        // Begun on 1 March of the first year (day 59): wanted by day 304 of that year.
        assert_eq!(roof_deadline(59, 304), 304);
        // Begun in December: by day 304 of the next year.
        assert_eq!(roof_deadline(340, 304), 365 + 304);
        // 400 hours, with half of 15 hours a day to spare: plenty of time in March, pressing by
        // October, and overdue after the deadline stays overdue.
        let march = urgency(400.0, 304, 15.0, 59);
        let october = urgency(400.0, 304, 15.0, 290);
        assert!(march < 0.3 && october > 3.0, "{march} {october}");
        assert_eq!(urgency(400.0, 304, 15.0, 400), 400.0 / 7.5);
        assert_eq!(urgency(0.0, 304, 15.0, 100), 0.0);
    }
}
