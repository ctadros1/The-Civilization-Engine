//! Goods (`kind = "good"`): things people carry home and keep, with how they keep. What anyone
//! gathers, stores or eats is decided by people at run time.

use civ_agents::params::{
    Eaten, GoodDef, GoodFire, GoodFuel, GoodHolder, GoodLift, GoodLight, GoodPower, GoodTelegraph,
    GoodUse, MoneyDef, PowerSource, StoreDef, ToolDef,
};
use civ_agents::structure::Timber;
use serde::Deserialize;

/// The `kind` value of a good.
pub const KIND: &str = "good";
/// The id segment: `pack:good/name`.
pub const ID_KIND: &str = "good";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GoodFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// `food`, `fuel`, `material`, `tool` or `store`.
    pub purpose: String,
    pub kcal_per_kg: f64,
    pub half_life_days: f64,
    /// The half-life under a roof; 0 when a roof makes no difference.
    pub sheltered_half_life_days: f64,
    /// `raw`, `cooked` or `never` (a recipe must make it into food first).
    pub eaten: String,
    pub shared: bool,
    /// The good it is kept back from, like seed from grain (used in its place only in hunger),
    /// or "" for none.
    pub reserve_for: String,
    /// For a tool: its life and how many a household wants.
    pub tool: Option<Tool>,
    /// For a material that is built with as timber: its strength (content API 16).
    pub timber: Option<TimberFile>,
    /// For a store: the room each one gives (content API 20).
    pub store: Option<StoreFile>,
    /// How it burns (content API 81; M6b slice BC): absent for a good fire does not touch.
    pub fire: Option<FireFile>,
    /// Content API 125 (M7d slice CD): it is paid by weight as money.
    #[serde(default)]
    pub money: Option<Money>,
    /// Content API 158 (M10a slice DP): what it gives burned; a fuel must have one.
    #[serde(default)]
    pub fuel: Option<FuelFile>,
    /// Content API 160 (M10a slice DR): the power it delivers as a station; only a fixed tool.
    #[serde(default)]
    pub power: Option<PowerFile>,
    /// Content API 165 (M10b slice DU): the light it gives as a lamp; only a tool.
    #[serde(default)]
    pub light: Option<LightFile>,
    /// Content API 172 (M10b slice DY): what it holds as a gas holder; only a fixed tool.
    #[serde(default)]
    pub holder: Option<HolderFile>,
    /// Content API 206 (M11c): what it sends as a telegraph instrument; only a fixed tool.
    #[serde(default)]
    pub telegraph: Option<TelegraphFile>,
    /// Content API 215 (ADR-0042 §2): what it does as a passenger lift; only a fixed tool.
    #[serde(default)]
    pub lift: Option<LiftFile>,
}

/// What a passenger lift does (ADR-0042 §2). See [`civ_agents::params::GoodLift`].
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LiftFile {
    pub rated_kg: f64,
    pub speed_ms: f64,
    pub drive_kw: f64,
    pub stop_s: f64,
    pub shaft_m2: f64,
}

/// What a telegraph instrument sends (M11c, ADR-0036 §8).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TelegraphFile {
    /// Messages its operator sends an hour, above 0 and at most 60.
    pub messages_h: f64,
}

/// What a holder keeps (M10b slice DY, ADR-0034 §8).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HolderFile {
    /// The good it holds, by content id.
    pub good: String,
    /// Kilograms of it held when full, above 0 and at most 100,000.
    pub kg: f64,
}

/// The light a lamp gives (M10b slice DU, ADR-0034 §9).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LightFile {
    /// The good it burns, by content id.
    pub fuel: String,
    /// Kilograms of it burned an hour lit, above 0 and at most 1.
    pub kg_per_hour: f64,
    /// Content API 172 (M10b slice DY): its fuel comes by a main, not from stores; false, the
    /// default, for a lamp its household fills.
    #[serde(default)]
    pub piped: bool,
}

/// The power a wheel or an engine delivers (M10a slice DR, ADR-0033 §5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PowerFile {
    /// `water` (a wheel at an installation) or `fuel` (an engine).
    pub source: String,
    /// The most it delivers, kilowatts.
    pub rated_kw: f64,
    /// The share of its source's power it turns into work, above 0 and at most 1 (research
    /// 07-06 §2.2).
    pub efficiency: f64,
    /// Content API 204 (M11b, first part): the one good an engine burns, by content id; left out
    /// for one that burns any fuel by its heat. Only an engine (`fuel`) names one.
    #[serde(default)]
    pub fuel: Option<String>,
}

/// What a good gives burned (M10a slice DP, ADR-0033 §1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FuelFile {
    /// Heat, megajoules a kilogram (research 07-06 §2.3: wood 15.6, charcoal 29.5, coal 25.8).
    pub mj_per_kg: f64,
    /// Smoke a kilogram makes, against firewood's 1.
    pub smoke: f64,
    /// Content API 162 (M10a slice DS): the technique a household must know to want it for its
    /// hearth; none, the default, for a fuel anyone burns.
    #[serde(default)]
    pub technique: Option<String>,
    /// Content API 209 (M11f, ADR-0038 §2): kilograms of ash a kilogram burned leaves; 0, the
    /// default, for none.
    #[serde(default)]
    pub ash_share: f64,
}

/// How a good burns (M6b slice BC; ADR-0022 §4, §5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FireFile {
    /// As a roof's covering, the exposure in kW/m² above which it catches at the fire's reference
    /// dryness (the people profile's `fire.reference_moisture`); 0 for one that does not catch
    /// from a neighbour's heat.
    pub catch_kw_m2: f64,
    /// The share of it under a roof that burns when the building is fully involved; only a good
    /// that takes room under a roof (a sheltered half-life) is ever there.
    #[serde(default)]
    pub burns: f64,
}

/// What a store holds (M3b slice Q).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoreFile {
    pub keeps_kg: f64,
}

/// What a timber carries (ADR-0009 §5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TimberFile {
    pub bending_mpa: f64,
    pub compression_mpa: f64,
    pub stiffness_gpa: f64,
    pub creep: f64,
    pub sustained: f64,
    /// Content API 208 (M11e, third part): the share of its strength a bare member keeps once
    /// its building has burnt out (ADR-0037 §5); absent, it chars as timber (ADR-0022 §5).
    #[serde(default)]
    pub burnt_keeps: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Money {
    pub verify_h: f64,
    /// Content API 126 (M7d slice CE): counted in pieces of issues a mint struck.
    #[serde(default)]
    pub coin: bool,
    /// Content API 193 (M10d slice EH): a note, counted in pieces of issues their issuers wrote.
    #[serde(default)]
    pub note: bool,
    /// Content API 130 (M7d slice CE4a): for coin, the metal it is struck of, by content id.
    #[serde(default)]
    pub metal: Option<String>,
    /// Content API 130 (M7d slice CE4a): for coin, the technique whose knowers assay a piece.
    #[serde(default)]
    pub assay: Option<String>,
    /// Content API 131 (M7d slice CE4c): for coin, the share off the metal's worth a foreign
    /// piece is taken at by one who cannot assay it.
    #[serde(default)]
    pub foreign_discount: f64,
    /// Content API 131 (M7d slice CE4c): hours to weigh and test a foreign piece.
    #[serde(default)]
    pub foreign_assay_h: f64,
    /// Content API 199 (M10d slice EK): for notes, hours a false piece takes one who knows the
    /// technique of `assay`.
    #[serde(default)]
    pub forge_h: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Tool {
    pub life_h: f64,
    pub per_worker: f64,
    pub fixed: bool,
    #[serde(default)]
    pub serves: String,
    /// Content API 164 (M10a slice DT): the fewest hands at it for it to run; only a fixed tool
    /// asks more than one.
    #[serde(default = "one_hand")]
    pub crew: u32,
}

fn one_hand() -> u32 {
    1
}

impl GoodFile {
    /// The compiled good (`None` if its purpose or way of eating is unknown, which
    /// [`GoodFile::problems`] reports). Its `reserve_for` is resolved later, once every good has
    /// its index.
    pub fn def(&self) -> Option<GoodDef> {
        Some(GoodDef {
            id: self.id.clone(),
            name: self.name.clone(),
            purpose: GoodUse::from_name(&self.purpose)?,
            kcal_per_kg: self.kcal_per_kg,
            half_life_days: self.half_life_days,
            sheltered_half_life_days: self.sheltered_half_life_days,
            eaten: Eaten::from_name(&self.eaten)?,
            shared: self.shared,
            reserve_for: None,
            tool: self.tool.as_ref().map(|t| ToolDef {
                life_h: t.life_h,
                per_worker: t.per_worker,
                fixed: t.fixed,
                serves: None,
                crew: t.crew.clamp(1, 20) as u8,
            }),
            timber: self.timber.as_ref().map(|t| Timber {
                bending_pa: t.bending_mpa * 1e6,
                compression_pa: t.compression_mpa * 1e6,
                stiffness_pa: t.stiffness_gpa * 1e9,
                creep: t.creep,
                sustained: t.sustained,
                // Timber's weight is the frame's (research 11-08 §2.1); only masonry reads it.
                density_kg_m3: 760.0,
                burnt_keeps: t.burnt_keeps,
            }),
            store: self.store.as_ref().map(|t| StoreDef {
                keeps_kg: t.keeps_kg,
            }),
            money: self.money.as_ref().map(|m| MoneyDef {
                verify_h: m.verify_h,
                coin: m.coin,
                note: m.note,
                // Resolved once the goods and techniques are known (`civ_content::compile`).
                metal: None,
                assay: None,
                foreign_discount: m.foreign_discount,
                foreign_assay_h: m.foreign_assay_h,
                forge_h: m.forge_h,
            }),
            fire: self.fire.as_ref().map(|f| GoodFire {
                catch_kw_m2: f.catch_kw_m2,
                burns: f.burns,
            }),
            fuel: self.fuel.as_ref().map(|f| GoodFuel {
                mj_per_kg: f.mj_per_kg,
                smoke: f.smoke,
                // Resolved once the techniques are known (`civ_content::compile`).
                technique: None,
                ash_share: f.ash_share,
            }),
            power: self.power.as_ref().map(|w| GoodPower {
                source: match w.source.as_str() {
                    "fuel" => PowerSource::Fuel,
                    "shaft" => PowerSource::Shaft,
                    _ => PowerSource::Water,
                },
                rated_kw: w.rated_kw,
                efficiency: w.efficiency,
                // Resolved once the goods are known (`civ_content::compile`).
                fuel: None,
            }),
            light: self.light.as_ref().map(|l| GoodLight {
                // Resolved once the goods are known (`civ_content::compile`).
                fuel: None,
                kg_per_hour: l.kg_per_hour,
                piped: l.piped,
            }),
            holder: self.holder.as_ref().map(|h| GoodHolder {
                // Resolved once the goods are known (`civ_content::compile`).
                good: None,
                kg: h.kg,
            }),
            telegraph: self.telegraph.as_ref().map(|t| GoodTelegraph {
                messages_h: t.messages_h,
            }),
            lift: self.lift.as_ref().map(|l| GoodLift {
                rated_kg: l.rated_kg,
                speed_ms: l.speed_ms,
                drive_kw: l.drive_kw,
                stop_s: l.stop_s,
                shaft_m2: l.shaft_m2,
            }),
        })
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if let Some(m) = &self.money {
            for (name, v) in [
                ("money.verify_h", m.verify_h),
                ("money.foreign_assay_h", m.foreign_assay_h),
                ("money.forge_h", m.forge_h),
            ] {
                if !(v.is_finite() && (0.0..=100.0).contains(&v)) {
                    p.push(format!("`{name}` must be between 0 and 100 (got {v})"));
                }
            }
            if m.coin && m.note {
                p.push("a money is coin or a note, not both".to_owned());
            }
            if !(m.foreign_discount.is_finite() && (0.0..1.0).contains(&m.foreign_discount)) {
                p.push(format!(
                    "`money.foreign_discount` must be at least 0 and below 1 (got {})",
                    m.foreign_discount
                ));
            }
        }
        if let Some(f) = &self.fuel {
            if !(f.mj_per_kg.is_finite() && (1.0..=60.0).contains(&f.mj_per_kg)) {
                p.push(format!(
                    "`fuel.mj_per_kg` must be between 1 and 60 (got {})",
                    f.mj_per_kg
                ));
            }
            if !(f.smoke.is_finite() && (0.0..=10.0).contains(&f.smoke)) {
                p.push(format!(
                    "`fuel.smoke` must be between 0 and 10 (got {})",
                    f.smoke
                ));
            }
            if !(f.ash_share.is_finite() && (0.0..=0.5).contains(&f.ash_share)) {
                p.push(format!(
                    "`fuel.ash_share` must be between 0 and 0.5 (got {})",
                    f.ash_share
                ));
            }
        }
        if let Some(w) = &self.power {
            if !matches!(w.source.as_str(), "water" | "fuel" | "shaft") {
                p.push(format!(
                    "unknown `power.source` `{}` (known: water, fuel, shaft)",
                    w.source
                ));
            }
            if !(w.rated_kw.is_finite() && w.rated_kw > 0.0 && w.rated_kw <= 1000.0) {
                p.push(format!(
                    "`power.rated_kw` must be above 0 and at most 1000 (got {})",
                    w.rated_kw
                ));
            }
            if !(w.efficiency.is_finite() && w.efficiency > 0.0 && w.efficiency <= 1.0) {
                p.push(format!(
                    "`power.efficiency` must be above 0 and at most 1 (got {})",
                    w.efficiency
                ));
            }
            if !self.tool.as_ref().is_some_and(|t| t.fixed) {
                p.push(
                    "only a fixed tool delivers power: `[power]` needs `[tool] fixed = true`"
                        .to_owned(),
                );
            }
            if w.fuel.is_some() && w.source != "fuel" {
                p.push("only an engine names a `power.fuel`: set `source = \"fuel\"`".to_owned());
            }
        }
        if let Some(l) = &self.light {
            if !(l.kg_per_hour.is_finite() && l.kg_per_hour > 0.0 && l.kg_per_hour <= 1.0) {
                p.push(format!(
                    "`light.kg_per_hour` must be above 0 and at most 1 (got {})",
                    l.kg_per_hour
                ));
            }
            if self.tool.is_none() {
                p.push("only a tool gives light: `[light]` needs a `[tool]` table".to_owned());
            }
        }
        if let Some(h) = &self.holder {
            if !(h.kg.is_finite() && h.kg > 0.0 && h.kg <= 100_000.0) {
                p.push(format!(
                    "`holder.kg` must be above 0 and at most 100000 (got {})",
                    h.kg
                ));
            }
            if !self.tool.as_ref().is_some_and(|t| t.fixed) {
                p.push(
                    "only a fixed tool holds apart: `[holder]` needs `[tool] fixed = true`"
                        .to_owned(),
                );
            }
        }
        if let Some(l) = &self.lift {
            for (name, v, lo, hi) in [
                ("lift.rated_kg", l.rated_kg, 1.0, 100_000.0),
                ("lift.speed_ms", l.speed_ms, 0.01, 20.0),
                ("lift.drive_kw", l.drive_kw, 0.01, 10_000.0),
                ("lift.stop_s", l.stop_s, 0.0, 600.0),
                ("lift.shaft_m2", l.shaft_m2, 0.0, 100.0),
            ] {
                if !(v.is_finite() && (lo..=hi).contains(&v)) {
                    p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
                }
            }
            if !self.tool.as_ref().is_some_and(|t| t.fixed) {
                p.push(
                    "only a fixed tool stands in a building: `[lift]` needs `[tool] fixed = true`"
                        .to_owned(),
                );
            }
        }
        if let Some(t) = &self.telegraph {
            if !(t.messages_h.is_finite() && t.messages_h > 0.0 && t.messages_h <= 60.0) {
                p.push(format!(
                    "`telegraph.messages_h` must be above 0 and at most 60 (got {})",
                    t.messages_h
                ));
            }
            if !self.tool.as_ref().is_some_and(|t| t.fixed) {
                p.push(
                    "only a fixed tool sends by wire: `[telegraph]` needs `[tool] fixed = true`"
                        .to_owned(),
                );
            }
        }
        if let Some(f) = &self.fire {
            if !(f.catch_kw_m2.is_finite() && (0.0..=500.0).contains(&f.catch_kw_m2)) {
                p.push(format!(
                    "`fire.catch_kw_m2` must be between 0 and 500 (got {})",
                    f.catch_kw_m2
                ));
            }
            if !(f.burns.is_finite() && (0.0..=1.0).contains(&f.burns)) {
                p.push(format!(
                    "`fire.burns` must be between 0 and 1 (got {})",
                    f.burns
                ));
            }
            if f.burns > 0.0 && self.sheltered_half_life_days <= 0.0 {
                p.push(
                    "`fire.burns` is for a good kept under a roof (a sheltered half-life); this \
                     one lies in the open"
                        .to_owned(),
                );
            }
        }
        if !(self.half_life_days.is_finite() && self.half_life_days >= 0.0) {
            p.push(format!(
                "`half_life_days` must be zero (keeps) or more (got {})",
                self.half_life_days
            ));
        }
        let sheltered = self.sheltered_half_life_days;
        if !(sheltered.is_finite() && sheltered >= 0.0) {
            p.push(format!(
                "`sheltered_half_life_days` must be zero (no difference) or more (got {sheltered})"
            ));
        } else if sheltered > 0.0 && (self.half_life_days <= 0.0 || sheltered < self.half_life_days)
        {
            p.push(format!(
                "a roof never makes a good spoil faster: `sheltered_half_life_days` ({sheltered}) \
                 must be at least `half_life_days` ({}), or 0, and a good that keeps needs none",
                self.half_life_days
            ));
        }
        let eaten = Eaten::from_name(&self.eaten);
        if eaten.is_none() {
            p.push(format!(
                "unknown `eaten` `{}` (known: raw, cooked, never)",
                self.eaten
            ));
        }
        let purpose = GoodUse::from_name(&self.purpose);
        if purpose != Some(GoodUse::Food) {
            if eaten.is_some_and(|e| e != Eaten::Never) {
                p.push("only food is eaten: `eaten` must be \"never\"".to_owned());
            }
            if !self.reserve_for.is_empty() {
                p.push("only food is kept back: `reserve_for` must be \"\"".to_owned());
            }
        }
        match (purpose, &self.tool) {
            (Some(GoodUse::Tool), None) => {
                p.push("a tool needs its `[tool]` table".to_owned());
            }
            (Some(GoodUse::Tool), Some(t)) => {
                if !(t.life_h.is_finite() && t.life_h > 0.0) {
                    p.push(format!("`tool.life_h` must be positive (got {})", t.life_h));
                }
                if !(t.per_worker.is_finite() && (0.0..=10.0).contains(&t.per_worker)) {
                    p.push(format!(
                        "`tool.per_worker` must be between 0 and 10 (got {})",
                        t.per_worker
                    ));
                }
                if !(1..=20).contains(&t.crew) {
                    p.push(format!(
                        "`tool.crew` must be between 1 and 20 (got {})",
                        t.crew
                    ));
                } else if t.crew > 1 && !t.fixed {
                    p.push(
                        "only a fixed tool needs a crew: `crew` above 1 needs `fixed = true`"
                            .to_owned(),
                    );
                }
            }
            (Some(_), Some(_)) => p.push("only a tool takes a `[tool]` table".to_owned()),
            _ => {}
        }
        match (purpose, &self.store) {
            (Some(GoodUse::Store), None) => {
                p.push("a store needs its `[store]` table".to_owned());
            }
            (Some(GoodUse::Store), Some(t)) => {
                if !(t.keeps_kg.is_finite() && t.keeps_kg > 0.0 && t.keeps_kg <= 1000.0) {
                    p.push(format!(
                        "`store.keeps_kg` must be above 0 and at most 1000 (got {})",
                        t.keeps_kg
                    ));
                }
            }
            (Some(_), Some(_)) => p.push("only a store takes a `[store]` table".to_owned()),
            _ => {}
        }
        if let Some(t) = &self.timber {
            if purpose != Some(GoodUse::Material) {
                p.push("only a material takes a `[timber]` table".to_owned());
            }
            for (name, v) in [
                ("bending_mpa", t.bending_mpa),
                ("compression_mpa", t.compression_mpa),
                ("stiffness_gpa", t.stiffness_gpa),
            ] {
                if !(v.is_finite() && v > 0.0) {
                    p.push(format!("`timber.{name}` must be positive (got {v})"));
                }
            }
            if !(t.creep.is_finite() && (0.0..=10.0).contains(&t.creep)) {
                p.push(format!(
                    "`timber.creep` must be between 0 and 10 (got {})",
                    t.creep
                ));
            }
            if !(t.sustained.is_finite() && t.sustained > 0.0 && t.sustained <= 1.0) {
                p.push(format!(
                    "`timber.sustained` must be above 0 and at most 1 (got {})",
                    t.sustained
                ));
            }
            if let Some(k) = t.burnt_keeps
                && !(k.is_finite() && (0.0..=1.0).contains(&k))
            {
                p.push(format!(
                    "`timber.burnt_keeps` must be between 0 and 1 (got {k})"
                ));
            }
        }
        match purpose {
            None => p.push(format!(
                "unknown purpose `{}` (known: food, fuel, material, tool, store)",
                self.purpose
            )),
            Some(GoodUse::Food) => {
                if !(self.kcal_per_kg.is_finite() && self.kcal_per_kg > 0.0) {
                    p.push(format!(
                        "a food needs positive `kcal_per_kg` (got {})",
                        self.kcal_per_kg
                    ));
                }
            }
            Some(GoodUse::Fuel) => {
                if self.fuel.is_none() {
                    p.push(
                        "a fuel says what it gives burned: a `[fuel]` table with `mj_per_kg` \
                         and `smoke`"
                            .to_owned(),
                    );
                }
                if self.kcal_per_kg != 0.0 {
                    p.push("a fuel is not eaten: `kcal_per_kg` must be 0".to_owned());
                }
                // Burning is settled exactly for any split of time only if fuel does not also
                // decay (crate `civ-agents`, `Household::stores_at_time`).
                if self.half_life_days != 0.0 {
                    p.push("a fuel keeps: `half_life_days` must be 0".to_owned());
                }
                if sheltered != 0.0 {
                    p.push("a fuel keeps: `sheltered_half_life_days` must be 0".to_owned());
                }
            }
            Some(GoodUse::Material | GoodUse::Tool | GoodUse::Store) => {
                if self.kcal_per_kg != 0.0 {
                    p.push(format!(
                        "a {} is not eaten: `kcal_per_kg` must be 0",
                        self.purpose
                    ));
                }
                // A household brings what it builds and works with for itself.
                if self.shared {
                    p.push(format!(
                        "a {} is the household's own: `shared` must be false",
                        self.purpose
                    ));
                }
            }
        }
        p
    }
}
