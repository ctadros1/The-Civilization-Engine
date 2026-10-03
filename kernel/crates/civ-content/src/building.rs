//! Building programs (`kind = "building"`): what a kind of building is made of and how much work
//! and material each part takes, with the dimensions people build it to. The grammar that turns
//! them into a building is code (`civ-grammar`; M1 has the hut); who builds what, where and when
//! is decided by people at run time.

use civ_agents::params::BuildingDef;
use civ_grammar::HutRules;
use serde::Deserialize;

/// The `kind` value of a building program.
pub const KIND: &str = "building";
/// The id segment: `pack:building/name`.
pub const ID_KIND: &str = "building";
/// The grammars this build has.
pub const GRAMMARS: [&str; 1] = ["hut"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuildingFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// The grammar that expands it: `hut`.
    pub grammar: String,
    /// Wall height people build to, centimetres.
    pub eave_cm: i32,
    /// Roof pitch people build to, degrees.
    pub pitch_deg: f64,
    /// The day of the year (0 = 1 January) a household wants to be under its roof by.
    pub roof_by_day: u16,
    pub materials: Materials,
    pub rules: Rules,
}

/// The good each material slot is made of: good ids.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Materials {
    pub timber: String,
    pub wattle: String,
    pub thatch: String,
}

impl Materials {
    /// `(slot name, good id)` in slot order (`civ_grammar::hut_materials`).
    pub fn slots(&self) -> [(&'static str, &str); 3] {
        [
            ("timber", &self.timber),
            ("wattle", &self.wattle),
            ("thatch", &self.thatch),
        ]
    }
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

/// Degrees to hundredths of a degree.
fn centideg(deg: f64) -> i32 {
    (deg * 100.0).round() as i32
}

impl BuildingFile {
    /// The compiled program, with its materials resolved by `good_index` (`None` if one is
    /// unknown, which the cross-file check reports).
    pub fn def(&self, good_index: &dyn Fn(&str) -> Option<usize>) -> Option<BuildingDef> {
        let r = &self.rules;
        let materials = self
            .materials
            .slots()
            .iter()
            .map(|(_, id)| good_index(id))
            .collect::<Option<Vec<usize>>>()?;
        Some(BuildingDef {
            id: self.id.clone(),
            name: self.name.clone(),
            rules: HutRules {
                radius_cm: (r.radius_cm[0], r.radius_cm[1]),
                eave_cm: (r.eave_cm[0], r.eave_cm[1]),
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
            },
            materials,
            eave_cm: self.eave_cm,
            pitch_centideg: centideg(self.pitch_deg),
            roof_by_day: self.roof_by_day,
        })
    }

    /// Range problems, as messages. Materials are checked against the goods later.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        let r = &self.rules;
        if !GRAMMARS.contains(&self.grammar.as_str()) {
            p.push(format!(
                "unknown grammar `{}` (known: {})",
                self.grammar,
                GRAMMARS.join(", ")
            ));
        }
        for (name, [lo, hi]) in [("radius_cm", r.radius_cm), ("eave_cm", r.eave_cm)] {
            if !(lo > 0 && lo <= hi) {
                p.push(format!(
                    "`rules.{name}` must be [least, most] with 0 < least <= most (got [{lo}, {hi}])"
                ));
            }
        }
        let [lo, hi] = r.pitch_deg;
        if !(lo > 0.0 && lo <= hi && hi < 90.0) {
            p.push(format!(
                "`rules.pitch_deg` must be [least, most] with 0 < least <= most < 90 (got [{lo}, {hi}])"
            ));
        }
        if !(r.eave_cm[0]..=r.eave_cm[1]).contains(&self.eave_cm) {
            p.push(format!(
                "`eave_cm` ({}) must be within `rules.eave_cm`",
                self.eave_cm
            ));
        }
        if !(self.pitch_deg >= lo && self.pitch_deg <= hi) {
            p.push(format!(
                "`pitch_deg` ({}) must be within `rules.pitch_deg`",
                self.pitch_deg
            ));
        }
        if self.roof_by_day >= 365 {
            p.push(format!(
                "`roof_by_day` must be a day of the year, 0 to 364 (got {})",
                self.roof_by_day
            ));
        }
        for (name, v) in [
            ("post_spacing_cm", r.post_spacing_cm),
            ("post_diameter_cm", r.post_diameter_cm),
            ("posthole_depth_cm", r.posthole_depth_cm),
            ("wall_thickness_cm", r.wall_thickness_cm),
            ("thatch_thickness_cm", r.thatch_thickness_cm),
            ("hearth_cm", r.hearth_cm),
        ] {
            if v <= 0 {
                p.push(format!("`rules.{name}` must be positive (got {v})"));
            }
        }
        if r.roof_overhang_cm < 0 {
            p.push(format!(
                "`rules.roof_overhang_cm` must be zero or more (got {})",
                r.roof_overhang_cm
            ));
        }
        if !(r.floor_base_m2.is_finite() && r.floor_base_m2 >= 0.0) {
            p.push(format!(
                "`rules.floor_base_m2` must be zero or more (got {})",
                r.floor_base_m2
            ));
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
            if !(v.is_finite() && v > 0.0) {
                p.push(format!("`rules.{name}` must be positive (got {v})"));
            }
        }
        p
    }
}
