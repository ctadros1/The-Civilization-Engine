//! Building programs (`kind = "building"`): what a kind of building is for and made of, how much
//! work and material each part takes, and the dimensions people build it to. The grammar that
//! turns them into a building is code (`civ-grammar`: the hut since M1, the frame since M3b
//! slice O); who builds what, where and when is decided by people at run time.

use std::collections::BTreeMap;

use civ_agents::params::{BuildingDef, FrameDesign};
use civ_grammar::{FrameRules, Grammar, HutRules, MAX_BAYS, MAX_STOREYS, ProgramRules, SpaceUse};
use civ_land::PlotUse;
use serde::Deserialize;

/// The `kind` value of a building program.
pub const KIND: &str = "building";
/// The id segment: `pack:building/name`.
pub const ID_KIND: &str = "building";
/// The grammars this build has.
pub const GRAMMARS: [&str; 2] = ["hut", "frame"];
/// A hut's material slots, in slot order (`civ_grammar::hut_materials`).
pub const HUT_SLOTS: [&str; 3] = ["timber", "wattle", "thatch"];
/// A frame's material slots, in slot order (`civ_grammar::frame_materials`).
pub const FRAME_SLOTS: [&str; 4] = ["timber", "wattle", "covering", "boards"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuildingFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// The grammar that expands it: `hut` or `frame`.
    pub grammar: String,
    /// What its buildings are for: `dwelling`, `store` or `work` (ADR-0009 §1).
    #[serde(rename = "use")]
    pub use_: String,
    /// Wall height people build to, centimetres.
    pub eave_cm: i32,
    /// Roof pitch people build to, degrees.
    pub pitch_deg: f64,
    /// The day of the year (0 = 1 January) a household wants to be under its roof by.
    pub roof_by_day: u16,
    /// The technique building it needs: a technique id, or "" for none (ADR-0008 §1).
    pub technique: String,
    /// The skill building it uses and trains: a skill id, or "" for none (ADR-0009 §6).
    pub skill: String,
    /// How its parts wear and decay (ADR-0009 §4).
    pub upkeep: UpkeepFile,
    /// The good each material slot is made of: slot name to good id ([`HUT_SLOTS`],
    /// [`FRAME_SLOTS`]).
    pub materials: BTreeMap<String, String>,
    /// A hut's rules.
    pub rules: Option<Rules>,
    /// A frame building's rules.
    pub frame: Option<Frame>,
    /// The sizes people build a frame program to.
    pub design: Option<Design>,
}

/// How a program's parts wear and decay (`civ_agents::params::Upkeep`), as authored: for each
/// kind, `[share lost a year, share at which it shows]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpkeepFile {
    pub covering: [f64; 2],
    pub posts: [f64; 2],
    pub infill: [f64; 2],
    pub under_leak: [f64; 2],
}

impl UpkeepFile {
    fn kinds(&self) -> [(&'static str, [f64; 2]); 4] {
        [
            ("covering", self.covering),
            ("posts", self.posts),
            ("infill", self.infill),
            ("under_leak", self.under_leak),
        ]
    }

    fn upkeep(&self) -> civ_agents::params::Upkeep {
        let wear = |[per_year, shows_at]: [f64; 2]| civ_agents::params::Wear { per_year, shows_at };
        civ_agents::params::Upkeep {
            covering: wear(self.covering),
            posts: wear(self.posts),
            infill: wear(self.infill),
            under_leak: wear(self.under_leak),
        }
    }
}

/// The sizes people build a frame program to (`civ_agents::params::FrameDesign`), as authored.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Design {
    pub bay_cm: i32,
    pub width_cm: i32,
    pub post_cm: i32,
    pub wall_cm: i32,
    pub overhang_cm: i32,
    pub joist_cm: i32,
    /// A raised floor's height; none unless authored.
    #[serde(default)]
    pub floor_raise_cm: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rules {
    pub radius_cm: [i32; 2],
    pub eave_cm: [i32; 2],
    pub pitch_deg: [f64; 2],
    pub post_spacing_cm: i32,
    pub post_diameter_cm: i32,
    pub posthole_depth_cm: i32,
    pub wall_thickness_cm: i32,
    pub roof_overhang_cm: i32,
    pub thatch_thickness_cm: i32,
    pub hearth_cm: i32,
    pub floor_base_m2: f64,
    pub floor_m2_per_sleeper: f64,
    pub store_kg_per_m2: f64,
    pub groundwork_h_per_m2: f64,
    pub posthole_h: f64,
    pub post_h: f64,
    pub rafter_h: f64,
    pub wattle_h_per_m2: f64,
    pub daub_h_per_m2: f64,
    pub thatch_h_per_m2: f64,
    pub finish_h_per_m2: f64,
    pub post_kg: f64,
    pub rafter_kg: f64,
    pub wattle_kg_per_m2: f64,
    pub thatch_kg_per_m2: f64,
}

/// A frame program's rules (`civ_grammar::FrameRules`), as authored.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Frame {
    pub bays: [i32; 2],
    pub bay_cm: [i32; 2],
    pub width_cm: [i32; 2],
    pub storeys: [u8; 2],
    pub eave_cm: [i32; 2],
    pub pitch_deg: [f64; 2],
    pub joist_cm: [i32; 2],
    pub post_cm: [i32; 2],
    pub overhang_cm: [i32; 2],
    pub floor_raise_cm: [i32; 2],
    pub wall_cm: [i32; 2],
    pub lofts: bool,
    /// What the first storey is for: `living`, `store` or `work`.
    pub ground: String,
    /// What an upper storey is for.
    pub upper: String,
    pub posthole_depth_cm: i32,
    pub beam_cm: i32,
    pub rafter_cm: i32,
    pub rafter_spacing_cm: i32,
    pub joist_spacing_cm: i32,
    pub decking_cm: i32,
    pub thatch_thickness_cm: i32,
    pub hearth_cm: i32,
    pub door_cm: i32,
    pub floor_base_m2: f64,
    pub floor_m2_per_sleeper: f64,
    pub work_m2_per_worker: f64,
    pub store_kg_per_m2: f64,
    pub loft_kg_per_m2: f64,
    pub living_kg_per_m2: f64,
    pub timber_kg_per_m3: f64,
    pub groundwork_h_per_m2: f64,
    pub posthole_h: f64,
    pub post_h: f64,
    pub beam_h_per_m: f64,
    pub rafter_h: f64,
    pub joist_h_per_m: f64,
    pub wattle_h_per_m2: f64,
    pub daub_h_per_m3: f64,
    pub thatch_h_per_m2: f64,
    pub finish_h_per_m2: f64,
    pub decking_h_per_m2: f64,
    pub ladder_h: f64,
    pub wattle_kg_per_m2: f64,
    pub thatch_kg_per_m2: f64,
    pub ladder_kg: f64,
}

/// Degrees to hundredths of a degree.
fn centideg(deg: f64) -> i32 {
    (deg * 100.0).round() as i32
}

fn pair<T: Copy>(v: [T; 2]) -> (T, T) {
    (v[0], v[1])
}

impl BuildingFile {
    /// The grammar it names, if this build has it.
    fn grammar(&self) -> Option<Grammar> {
        Grammar::from_name(&self.grammar)
    }

    /// Its material slots in slot order, with the good each names (`""` when missing).
    pub fn slots(&self) -> Vec<(&'static str, &str)> {
        let names: &[&'static str] = match self.grammar() {
            Some(Grammar::Frame) => &FRAME_SLOTS,
            _ => &HUT_SLOTS,
        };
        names
            .iter()
            .map(|&n| (n, self.materials.get(n).map_or("", String::as_str)))
            .collect()
    }

    /// The grammar's rules, if they are there and every name in them is known.
    fn program_rules(&self) -> Option<ProgramRules> {
        match self.grammar()? {
            Grammar::Hut => {
                let r = self.rules.as_ref()?;
                Some(ProgramRules::Hut(HutRules {
                    radius_cm: pair(r.radius_cm),
                    eave_cm: pair(r.eave_cm),
                    pitch_centideg: (centideg(r.pitch_deg[0]), centideg(r.pitch_deg[1])),
                    post_spacing_cm: r.post_spacing_cm,
                    post_diameter_cm: r.post_diameter_cm,
                    posthole_depth_cm: r.posthole_depth_cm,
                    wall_thickness_cm: r.wall_thickness_cm,
                    roof_overhang_cm: r.roof_overhang_cm,
                    thatch_thickness_cm: r.thatch_thickness_cm,
                    hearth_cm: r.hearth_cm,
                    floor_base_m2: r.floor_base_m2,
                    floor_m2_per_sleeper: r.floor_m2_per_sleeper,
                    groundwork_h_per_m2: r.groundwork_h_per_m2,
                    posthole_h: r.posthole_h,
                    post_h: r.post_h,
                    rafter_h: r.rafter_h,
                    wattle_h_per_m2: r.wattle_h_per_m2,
                    daub_h_per_m2: r.daub_h_per_m2,
                    thatch_h_per_m2: r.thatch_h_per_m2,
                    finish_h_per_m2: r.finish_h_per_m2,
                    post_kg: r.post_kg,
                    rafter_kg: r.rafter_kg,
                    wattle_kg_per_m2: r.wattle_kg_per_m2,
                    thatch_kg_per_m2: r.thatch_kg_per_m2,
                    store_kg_per_m2: r.store_kg_per_m2,
                }))
            }
            Grammar::Frame => {
                let f = self.frame.as_ref()?;
                Some(ProgramRules::Frame(FrameRules {
                    bays: pair(f.bays),
                    bay_cm: pair(f.bay_cm),
                    width_cm: pair(f.width_cm),
                    storeys: pair(f.storeys),
                    eave_cm: pair(f.eave_cm),
                    pitch_centideg: (centideg(f.pitch_deg[0]), centideg(f.pitch_deg[1])),
                    joist_cm: pair(f.joist_cm),
                    post_cm: pair(f.post_cm),
                    overhang_cm: pair(f.overhang_cm),
                    floor_raise_cm: pair(f.floor_raise_cm),
                    wall_cm: pair(f.wall_cm),
                    lofts: f.lofts,
                    ground_use: SpaceUse::from_name(&f.ground)?,
                    upper_use: SpaceUse::from_name(&f.upper)?,
                    posthole_depth_cm: f.posthole_depth_cm,
                    beam_cm: f.beam_cm,
                    rafter_cm: f.rafter_cm,
                    rafter_spacing_cm: f.rafter_spacing_cm,
                    joist_spacing_cm: f.joist_spacing_cm,
                    decking_cm: f.decking_cm,
                    thatch_thickness_cm: f.thatch_thickness_cm,
                    hearth_cm: f.hearth_cm,
                    door_cm: f.door_cm,
                    floor_base_m2: f.floor_base_m2,
                    floor_m2_per_sleeper: f.floor_m2_per_sleeper,
                    work_m2_per_worker: f.work_m2_per_worker,
                    store_kg_per_m2: f.store_kg_per_m2,
                    loft_kg_per_m2: f.loft_kg_per_m2,
                    living_kg_per_m2: f.living_kg_per_m2,
                    timber_kg_per_m3: f.timber_kg_per_m3,
                    groundwork_h_per_m2: f.groundwork_h_per_m2,
                    posthole_h: f.posthole_h,
                    post_h: f.post_h,
                    beam_h_per_m: f.beam_h_per_m,
                    rafter_h: f.rafter_h,
                    joist_h_per_m: f.joist_h_per_m,
                    wattle_h_per_m2: f.wattle_h_per_m2,
                    daub_h_per_m3: f.daub_h_per_m3,
                    thatch_h_per_m2: f.thatch_h_per_m2,
                    finish_h_per_m2: f.finish_h_per_m2,
                    decking_h_per_m2: f.decking_h_per_m2,
                    ladder_h: f.ladder_h,
                    wattle_kg_per_m2: f.wattle_kg_per_m2,
                    thatch_kg_per_m2: f.thatch_kg_per_m2,
                    ladder_kg: f.ladder_kg,
                }))
            }
        }
    }

    /// The compiled program, with its materials resolved by `good_index` (in `goods`) and its
    /// technique and skill given, and the shapes people would build it in (`None` if a material
    /// is unknown, which the cross-file check reports, or its rules do not compile, which
    /// [`BuildingFile::problems`] reports).
    pub fn def(
        &self,
        good_index: &dyn Fn(&str) -> Option<usize>,
        goods: &[civ_agents::params::GoodDef],
        technique: Option<usize>,
        skill: Option<usize>,
    ) -> Option<BuildingDef> {
        let materials = self
            .slots()
            .iter()
            .map(|(_, id)| good_index(id))
            .collect::<Option<Vec<usize>>>()?;
        let mut def = BuildingDef {
            id: self.id.clone(),
            name: self.name.clone(),
            use_: PlotUse::from_name(&self.use_)?,
            rules: self.program_rules()?,
            materials,
            eave_cm: self.eave_cm,
            pitch_centideg: centideg(self.pitch_deg),
            roof_by_day: self.roof_by_day,
            technique,
            design: self.design.as_ref().map(|d| FrameDesign {
                bay_cm: d.bay_cm,
                width_cm: d.width_cm,
                post_cm: d.post_cm,
                wall_cm: d.wall_cm,
                overhang_cm: d.overhang_cm,
                joist_cm: d.joist_cm,
                floor_raise_cm: d.floor_raise_cm,
            }),
            shapes: Vec::new(),
            skill,
            upkeep: self.upkeep.upkeep(),
        };
        def.shapes = civ_agents::build::shapes(&def, goods);
        Some(def)
    }

    /// Range problems, as messages. Materials are checked against the goods later.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        let Some(grammar) = self.grammar() else {
            p.push(format!(
                "unknown grammar `{}` (known: {})",
                self.grammar,
                GRAMMARS.join(", ")
            ));
            return p;
        };
        if PlotUse::from_name(&self.use_).is_none() {
            let known: Vec<&str> = PlotUse::ALL.iter().map(|u| u.name()).collect();
            p.push(format!(
                "unknown use `{}` (known: {})",
                self.use_,
                known.join(", ")
            ));
        }
        let slots: Vec<&str> = self.slots().iter().map(|(n, _)| *n).collect();
        for (slot, _) in self.slots().iter().filter(|(_, id)| id.is_empty()) {
            p.push(format!("`materials.{slot}` is missing"));
        }
        for slot in self
            .materials
            .keys()
            .filter(|k| !slots.contains(&k.as_str()))
        {
            p.push(format!(
                "`materials.{slot}` is not a slot of the {} grammar (slots: {})",
                self.grammar,
                slots.join(", ")
            ));
        }
        for (name, [per_year, shows_at]) in self.upkeep.kinds() {
            if !(per_year.is_finite() && per_year >= 0.0) {
                p.push(format!(
                    "`upkeep.{name}`: the share lost a year must be zero or more (got {per_year})"
                ));
            }
            if !(shows_at > 0.0 && shows_at <= 1.0) {
                p.push(format!(
                    "`upkeep.{name}`: the share at which it shows must be in (0, 1] (got {shows_at})"
                ));
            }
        }
        if self.roof_by_day >= 365 {
            p.push(format!(
                "`roof_by_day` must be a day of the year, 0 to 364 (got {})",
                self.roof_by_day
            ));
        }
        match grammar {
            Grammar::Hut => {
                if self.frame.is_some() || self.design.is_some() {
                    p.push("a hut program has `[rules]`, not `[frame]` or `[design]`".to_owned());
                }
                match &self.rules {
                    Some(r) => self.hut_problems(r, &mut p),
                    None => p.push("a hut program needs `[rules]`".to_owned()),
                }
            }
            Grammar::Frame => {
                if self.rules.is_some() {
                    p.push("a frame program has `[frame]`, not `[rules]`".to_owned());
                }
                match &self.frame {
                    Some(f) => self.frame_problems(f, &mut p),
                    None => p.push("a frame program needs `[frame]`".to_owned()),
                }
                match (&self.design, &self.frame) {
                    (Some(d), Some(f)) => design_problems(d, f, &mut p),
                    (None, _) => p.push("a frame program needs `[design]`".to_owned()),
                    (Some(_), None) => {}
                }
            }
        }
        p
    }

    /// The top-level wall height and pitch must lie within the program's ranges.
    fn usual_within(&self, table: &str, eave: [i32; 2], pitch: [f64; 2], p: &mut Vec<String>) {
        if !(eave[0]..=eave[1]).contains(&self.eave_cm) {
            p.push(format!(
                "`eave_cm` ({}) must be within `{table}.eave_cm`",
                self.eave_cm
            ));
        }
        if !(self.pitch_deg >= pitch[0] && self.pitch_deg <= pitch[1]) {
            p.push(format!(
                "`pitch_deg` ({}) must be within `{table}.pitch_deg`",
                self.pitch_deg
            ));
        }
    }

    fn hut_problems(&self, r: &Rules, p: &mut Vec<String>) {
        for (name, [lo, hi]) in [("radius_cm", r.radius_cm), ("eave_cm", r.eave_cm)] {
            ranged(p, "rules", name, lo, hi, 1);
        }
        pitch_range(p, "rules", r.pitch_deg);
        self.usual_within("rules", r.eave_cm, r.pitch_deg, p);
        for (name, v) in [
            ("post_spacing_cm", r.post_spacing_cm),
            ("post_diameter_cm", r.post_diameter_cm),
            ("posthole_depth_cm", r.posthole_depth_cm),
            ("wall_thickness_cm", r.wall_thickness_cm),
            ("thatch_thickness_cm", r.thatch_thickness_cm),
            ("hearth_cm", r.hearth_cm),
        ] {
            positive_int(p, "rules", name, v);
        }
        if r.roof_overhang_cm < 0 {
            p.push(format!(
                "`rules.roof_overhang_cm` must be zero or more (got {})",
                r.roof_overhang_cm
            ));
        }
        for (name, v) in [
            ("floor_base_m2", r.floor_base_m2),
            ("store_kg_per_m2", r.store_kg_per_m2),
        ] {
            not_negative(p, "rules", name, v);
        }
        // Every stage takes work, and every piece of material has weight: a stage with no work
        // would be done the moment it began.
        for (name, v) in [
            ("floor_m2_per_sleeper", r.floor_m2_per_sleeper),
            ("groundwork_h_per_m2", r.groundwork_h_per_m2),
            ("posthole_h", r.posthole_h),
            ("post_h", r.post_h),
            ("rafter_h", r.rafter_h),
            ("wattle_h_per_m2", r.wattle_h_per_m2),
            ("daub_h_per_m2", r.daub_h_per_m2),
            ("thatch_h_per_m2", r.thatch_h_per_m2),
            ("finish_h_per_m2", r.finish_h_per_m2),
            ("post_kg", r.post_kg),
            ("rafter_kg", r.rafter_kg),
            ("wattle_kg_per_m2", r.wattle_kg_per_m2),
            ("thatch_kg_per_m2", r.thatch_kg_per_m2),
        ] {
            positive(p, "rules", name, v);
        }
    }

    fn frame_problems(&self, f: &Frame, p: &mut Vec<String>) {
        let t = "frame";
        ranged(p, t, "bays", f.bays[0], f.bays[1], 1);
        if f.bays[1] > MAX_BAYS {
            p.push(format!(
                "`frame.bays` may reach at most {MAX_BAYS} (got {})",
                f.bays[1]
            ));
        }
        let [s0, s1] = f.storeys;
        if !(s0 >= 1 && s0 <= s1 && s1 <= MAX_STOREYS) {
            p.push(format!(
                "`frame.storeys` must be [least, most] within 1..={MAX_STOREYS} (got [{s0}, {s1}])"
            ));
        }
        for (name, [lo, hi]) in [
            ("bay_cm", f.bay_cm),
            ("width_cm", f.width_cm),
            ("eave_cm", f.eave_cm),
            ("joist_cm", f.joist_cm),
            ("post_cm", f.post_cm),
            ("wall_cm", f.wall_cm),
        ] {
            ranged(p, t, name, lo, hi, 1);
        }
        for (name, [lo, hi]) in [
            ("overhang_cm", f.overhang_cm),
            ("floor_raise_cm", f.floor_raise_cm),
        ] {
            ranged(p, t, name, lo, hi, 0);
        }
        pitch_range(p, t, f.pitch_deg);
        self.usual_within(t, f.eave_cm, f.pitch_deg, p);
        // The grammar refuses a bay or a width no longer than its posts are thick.
        for (name, [lo, _]) in [("bay_cm", f.bay_cm), ("width_cm", f.width_cm)] {
            if lo <= f.post_cm[1] {
                p.push(format!(
                    "`frame.{name}` must start above `frame.post_cm`'s most ({lo} <= {})",
                    f.post_cm[1]
                ));
            }
        }
        for (name, v) in [("ground", &f.ground), ("upper", &f.upper)] {
            if SpaceUse::from_name(v).is_none() {
                let known: Vec<&str> = SpaceUse::ALL.iter().map(|u| u.name()).collect();
                p.push(format!(
                    "`frame.{name}` must be one of {} (got `{v}`)",
                    known.join(", ")
                ));
            }
        }
        for (name, v) in [
            ("posthole_depth_cm", f.posthole_depth_cm),
            ("beam_cm", f.beam_cm),
            ("rafter_cm", f.rafter_cm),
            ("rafter_spacing_cm", f.rafter_spacing_cm),
            ("joist_spacing_cm", f.joist_spacing_cm),
            ("decking_cm", f.decking_cm),
            ("thatch_thickness_cm", f.thatch_thickness_cm),
            ("hearth_cm", f.hearth_cm),
            ("door_cm", f.door_cm),
        ] {
            positive_int(p, t, name, v);
        }
        for (name, v) in [
            ("floor_base_m2", f.floor_base_m2),
            ("work_m2_per_worker", f.work_m2_per_worker),
            ("store_kg_per_m2", f.store_kg_per_m2),
            ("loft_kg_per_m2", f.loft_kg_per_m2),
            ("living_kg_per_m2", f.living_kg_per_m2),
        ] {
            not_negative(p, t, name, v);
        }
        for (name, v) in [
            ("floor_m2_per_sleeper", f.floor_m2_per_sleeper),
            ("timber_kg_per_m3", f.timber_kg_per_m3),
            ("groundwork_h_per_m2", f.groundwork_h_per_m2),
            ("posthole_h", f.posthole_h),
            ("post_h", f.post_h),
            ("beam_h_per_m", f.beam_h_per_m),
            ("rafter_h", f.rafter_h),
            ("joist_h_per_m", f.joist_h_per_m),
            ("wattle_h_per_m2", f.wattle_h_per_m2),
            ("daub_h_per_m3", f.daub_h_per_m3),
            ("thatch_h_per_m2", f.thatch_h_per_m2),
            ("finish_h_per_m2", f.finish_h_per_m2),
            ("decking_h_per_m2", f.decking_h_per_m2),
            ("ladder_h", f.ladder_h),
            ("wattle_kg_per_m2", f.wattle_kg_per_m2),
            ("thatch_kg_per_m2", f.thatch_kg_per_m2),
            ("ladder_kg", f.ladder_kg),
        ] {
            positive(p, t, name, v);
        }
    }
}

/// The sizes people build to must lie within the frame's ranges, its bay and width on the
/// footprint's quantum.
fn design_problems(d: &Design, f: &Frame, p: &mut Vec<String>) {
    for (name, v, [lo, hi]) in [
        ("bay_cm", d.bay_cm, f.bay_cm),
        ("width_cm", d.width_cm, f.width_cm),
        ("post_cm", d.post_cm, f.post_cm),
        ("wall_cm", d.wall_cm, f.wall_cm),
        ("overhang_cm", d.overhang_cm, f.overhang_cm),
        ("joist_cm", d.joist_cm, f.joist_cm),
        ("floor_raise_cm", d.floor_raise_cm, f.floor_raise_cm),
    ] {
        if !(lo..=hi).contains(&v) {
            p.push(format!(
                "`design.{name}` ({v}) must be within `frame.{name}`"
            ));
        }
    }
    for (name, v) in [("bay_cm", d.bay_cm), ("width_cm", d.width_cm)] {
        if v % civ_grammar::RECT_QUANTUM_CM != 0 {
            p.push(format!(
                "`design.{name}` ({v}) must be a multiple of {} cm",
                civ_grammar::RECT_QUANTUM_CM
            ));
        }
    }
}

/// `[least, most]` with `min <= least <= most`.
fn ranged(p: &mut Vec<String>, table: &str, name: &str, lo: i32, hi: i32, min: i32) {
    if !(lo >= min && lo <= hi) {
        p.push(format!(
            "`{table}.{name}` must be [least, most] with {min} <= least <= most (got [{lo}, {hi}])"
        ));
    }
}

fn pitch_range(p: &mut Vec<String>, table: &str, [lo, hi]: [f64; 2]) {
    if !(lo > 0.0 && lo <= hi && hi < 90.0) {
        p.push(format!(
            "`{table}.pitch_deg` must be [least, most] with 0 < least <= most < 90 (got [{lo}, {hi}])"
        ));
    }
}

fn positive_int(p: &mut Vec<String>, table: &str, name: &str, v: i32) {
    if v <= 0 {
        p.push(format!("`{table}.{name}` must be positive (got {v})"));
    }
}

fn positive(p: &mut Vec<String>, table: &str, name: &str, v: f64) {
    if !(v.is_finite() && v > 0.0) {
        p.push(format!("`{table}.{name}` must be positive (got {v})"));
    }
}

fn not_negative(p: &mut Vec<String>, table: &str, name: &str, v: f64) {
    if !(v.is_finite() && v >= 0.0) {
        p.push(format!("`{table}.{name}` must be zero or more (got {v})"));
    }
}
