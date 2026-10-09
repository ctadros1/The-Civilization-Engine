//! Authored parameters of people and their activities. Filled by `civ-content`; nothing here has
//! a default, so every number is visible in a content file with its source.

use civ_core::time::DAYS_PER_YEAR;
use civ_land::{CropParams, FieldTask};
use civ_world::nav::NavParams;

/// What an authored activity does: the verb the engine knows how to carry out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Behavior {
    /// Sleep at home.
    Sleep,
    /// Eat a meal from the household's store, at home.
    Eat,
    /// Walk to water, fill containers, carry them home.
    FetchWater,
    /// Walk to a patch, gather a wild resource, carry it home.
    Gather,
    /// Sit with others at the hearth.
    Socialize,
    /// Rest at home.
    Rest,
    /// Children's play near home.
    Play,
    /// Work a household field: the activity names the task (prepare, sow, tend, reap, thresh).
    Farm,
    /// Ask a household of the settlement that can spare food for some, when short.
    Ask,
    /// Work on the household's home: the stage under way, with the materials at hand.
    Build,
    /// Work a recipe at home: grind grain, bake, make a tool (M3a).
    Make,
    /// Go to another household of the settlement and exchange goods at its posted terms (M3a).
    Trade,
    /// Work for wages at a workshop of another household of the settlement (M3a).
    Hire,
    /// Spend spare hours at home trying toward a technique that would answer a household problem
    /// (M3b, ADR-0008 §3): the activity's target names the technique.
    Try,
    /// Walk to a deposit the settlement knows, dig its good from a pit there and carry it home
    /// (M3b slice Q, ADR-0010 §2): the activity names the good.
    Dig,
    /// Go to the hearth for the gathering called there, and have a say (M4a slice Z, ADR-0013
    /// §1).
    Attend,
    /// Go to another household's home and take food from its store, when short (M4b slice AA,
    /// ADR-0015 §2).
    Take,
    /// Walk a round of the settlement's homes at night and stand watch at each (M4b slice AC,
    /// ADR-0015 §6): only for the one a law names to keep watch.
    Watch,
    /// Go to the hearth to join a petition a faction has called there (M4c slice AH, ADR-0017
    /// §3): only for those who heard of it.
    Petition,
    /// Walk to the hearth of another settlement the household knows, keep company there and
    /// walk home within the day (M5a slice AM, ADR-0018 §2: presence, never residence).
    Visit,
    /// Walk to a seller's door in another settlement the household holds a price report of, buy
    /// there if its terms still serve, and walk home within the day (M5b slice AP, ADR-0019 §2).
    Fetch,
}

impl Behavior {
    /// Every behavior, in a fixed order (part of the boundary: never reorder).
    pub const ALL: [Behavior; 21] = [
        Behavior::Sleep,
        Behavior::Eat,
        Behavior::FetchWater,
        Behavior::Gather,
        Behavior::Socialize,
        Behavior::Rest,
        Behavior::Play,
        Behavior::Farm,
        Behavior::Ask,
        Behavior::Build,
        Behavior::Make,
        Behavior::Trade,
        Behavior::Hire,
        Behavior::Try,
        Behavior::Dig,
        Behavior::Attend,
        Behavior::Take,
        Behavior::Watch,
        Behavior::Petition,
        Behavior::Visit,
        Behavior::Fetch,
    ];

    /// The authored name of a behavior.
    pub fn name(self) -> &'static str {
        match self {
            Behavior::Sleep => "sleep",
            Behavior::Eat => "eat",
            Behavior::FetchWater => "fetch_water",
            Behavior::Gather => "gather",
            Behavior::Socialize => "socialize",
            Behavior::Rest => "rest",
            Behavior::Play => "play",
            Behavior::Farm => "farm",
            Behavior::Ask => "ask",
            Behavior::Build => "build",
            Behavior::Make => "make",
            Behavior::Trade => "trade",
            Behavior::Hire => "hire",
            Behavior::Try => "try",
            Behavior::Dig => "dig",
            Behavior::Attend => "attend",
            Behavior::Take => "take",
            Behavior::Watch => "watch",
            Behavior::Petition => "petition",
            Behavior::Visit => "visit",
            Behavior::Fetch => "fetch",
        }
    }

    /// The behavior with an authored name.
    pub fn from_name(name: &str) -> Option<Behavior> {
        Behavior::ALL.into_iter().find(|b| b.name() == name)
    }
}

/// An authored activity: a behavior with its numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityDef {
    /// Content id, for example `core:activity/gather_plants`.
    pub id: String,
    /// Display name, for example "Gather plants".
    pub name: String,
    /// What the inspector says someone is doing, for example "gathering plants".
    pub doing: String,
    /// The behavior it uses.
    pub behavior: Behavior,
    /// For gathering: the land resource, by index in the land parameters.
    pub resource: Option<usize>,
    /// For farming: the field task.
    pub task: Option<FieldTask>,
    /// For making: the recipe, by index in the catalog's recipes.
    pub recipe: Option<usize>,
    /// For digging: the good dug, by index in the catalog's goods (M3b slice Q).
    pub digs: Option<usize>,
    /// Tools the work needs and wears, by index in the catalog's goods (a recipe's own tools are
    /// on the recipe).
    pub tools: Vec<usize>,
    /// Work done in an hour, as a share of what the task's authored rates assume (1 with the
    /// tools they assume; less by hand).
    pub rate: f64,
    /// Physical activity ratio of the work (energy use as a multiple of basal metabolism).
    pub par: f64,
    /// Youngest age that does it, years.
    pub min_age_years: f64,
    /// Oldest age that does it, years.
    pub max_age_years: f64,
    /// Shortest work time, minutes.
    pub min_minutes: u32,
    /// Longest work time, minutes.
    pub max_minutes: u32,
    /// Only done in daylight.
    pub daylight_only: bool,
    /// Longest one-way walk people will make for it, minutes.
    pub max_walk_minutes: u32,
    /// The technique the work needs, by index in the catalog's techniques (ADR-0008 §1); a
    /// `make` activity's is its recipe's.
    pub technique: Option<usize>,
}

/// What a good is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GoodUse {
    /// Eaten, as it is or once a recipe has made it ready.
    Food,
    /// Burned for cooking and warmth.
    Fuel,
    /// Built with, or made into something.
    Material,
    /// Worked with: counted in standard tools' worth of use (ADR-0006 §1).
    Tool,
    /// Keeps other goods in it (a pot): counted one by one, each giving room (M3b slice Q).
    Store,
}

impl GoodUse {
    /// Every use, in a fixed order.
    pub const ALL: [GoodUse; 5] = [
        GoodUse::Food,
        GoodUse::Fuel,
        GoodUse::Material,
        GoodUse::Tool,
        GoodUse::Store,
    ];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            GoodUse::Food => "food",
            GoodUse::Fuel => "fuel",
            GoodUse::Material => "material",
            GoodUse::Tool => "tool",
            GoodUse::Store => "store",
        }
    }

    /// The use with an authored name.
    pub fn from_name(name: &str) -> Option<GoodUse> {
        GoodUse::ALL.into_iter().find(|u| u.name() == name)
    }
}

/// How a good is eaten.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Eaten {
    /// As it is.
    Raw,
    /// Cooked over a fire: only while the household has firewood.
    Cooked,
    /// Not at all: a recipe must make it into food first (grain is ground or pounded).
    Never,
}

impl Eaten {
    /// Every way, in a fixed order.
    pub const ALL: [Eaten; 3] = [Eaten::Raw, Eaten::Cooked, Eaten::Never];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            Eaten::Raw => "raw",
            Eaten::Cooked => "cooked",
            Eaten::Never => "never",
        }
    }

    /// The way with an authored name.
    pub fn from_name(name: &str) -> Option<Eaten> {
        Eaten::ALL.into_iter().find(|e| e.name() == name)
    }
}

/// What a tool good adds to being a good (ADR-0006 §1).
#[derive(Clone, Debug, PartialEq)]
pub struct ToolDef {
    /// Hours of use a standard tool lasts: the good's unit. An hour's use takes `1 / life_h` of
    /// a unit.
    pub life_h: f64,
    /// Tools a household wants for each member old enough for the work that needs one.
    pub per_worker: f64,
    /// It stays where it was made (an oven): never carried off.
    pub fixed: bool,
}

/// What a store good adds to being a good (M3b slice Q): the room each one gives.
#[derive(Clone, Debug, PartialEq)]
pub struct StoreDef {
    /// Kilograms of goods one holds, keeping them as a raised store's floor does.
    pub keeps_kg: f64,
}

/// An authored good: something people carry home and keep.
#[derive(Clone, Debug, PartialEq)]
pub struct GoodDef {
    /// Content id, for example `core:good/meat`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What it is for.
    pub purpose: GoodUse,
    /// Food energy, kcal per kilogram (0 for goods that are not eaten).
    pub kcal_per_kg: f64,
    /// Days for half of a stored amount to spoil; 0 means it keeps.
    pub half_life_days: f64,
    /// The same under a roof; 0 means a roof makes no difference.
    pub sheltered_half_life_days: f64,
    /// How it is eaten.
    pub eaten: Eaten,
    /// When brought home it is shared among every household of the settlement.
    pub shared: bool,
    /// Kept back from another good (seed grain from grain), by index in the goods: used in its
    /// place only in hunger.
    pub reserve_for: Option<usize>,
    /// For a tool: its life and how many a household wants.
    pub tool: Option<ToolDef>,
    /// For a material built with as timber: what it carries (ADR-0009 §5).
    pub timber: Option<crate::structure::Timber>,
    /// For a store (a pot): the room each one gives.
    pub store: Option<StoreDef>,
}

impl GoodDef {
    /// It is eaten only cooked over a fire.
    pub fn cooked(&self) -> bool {
        self.eaten == Eaten::Cooked
    }

    /// It can be eaten as it is or cooked, without a recipe first.
    pub fn edible(&self) -> bool {
        self.purpose == GoodUse::Food && self.eaten != Eaten::Never && self.kcal_per_kg > 0.0
    }

    /// It is kept back from another good (seed).
    pub fn kept_back(&self) -> bool {
        self.reserve_for.is_some()
    }
}

/// A skill: a domain people get better at with practice (ADR-0006 §2).
#[derive(Clone, Debug, PartialEq)]
pub struct SkillDef {
    /// Content id, for example `core:skill/knapping`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Hours of practice that close 80 % of the gap to mastery (research 06-08 §5.2).
    pub t80_h: f64,
    /// Work speed by level: `(level, factor)`, ascending.
    pub speed: Vec<(f64, f64)>,
    /// Life of the tools made, by level: `(level, factor)`, ascending.
    pub quality: Vec<(f64, f64)>,
    /// Levels a grown founder brings: drawn evenly between the two.
    pub founder_level: [f64; 2],
}

impl SkillDef {
    /// The level after `hours` more practice from `level`: s′ = s + (1 − s)(1 − e^(−kE)) with
    /// k = ln 5 / T80 (research 06-08 §5.2), mastery being level 1.
    pub fn practised(&self, level: f64, hours: f64) -> f64 {
        let k = 5f64.ln() / self.t80_h.max(1e-6);
        let s = level.clamp(0.0, 1.0);
        (s + (1.0 - s) * (1.0 - (-k * hours.max(0.0)).exp())).clamp(0.0, 1.0)
    }
}

/// Who holds ground once it is broken (ADR-0007 §2; research 08-09 §1.2: claims form by first
/// cultivation, or by membership and allocation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LandHolder {
    /// The household that broke it.
    Breaker,
    /// The settlement it lies by.
    Settlement,
}

/// How the use of broken ground is given (ADR-0007 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LandUse {
    /// Its holder works it, or lets it where leasing is allowed.
    Holder,
    /// Its settlement gives each household fields to work by how many it feeds, at a yearly
    /// review and when households form or end (research 08-09 §5.2).
    Need,
}

/// What becomes of a household's holdings when it is no more (ADR-0007 §2; research 08-09 §1.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Succession {
    /// All of it goes to one heir: the household its people joined, or its nearest kin's.
    Heir,
    /// Its fields are shared out among its heirs' households, whole fields as near equal in
    /// area as they can be.
    Divided,
    /// It returns to the settlement.
    Settlement,
}

macro_rules! named {
    ($t:ty, $($v:path => $n:literal),+ $(,)?) => {
        impl $t {
            /// Every value, in a fixed order.
            pub const ALL: &'static [$t] = &[$($v),+];

            /// The authored name.
            pub fn name(self) -> &'static str {
                match self {
                    $($v => $n),+
                }
            }

            /// The value with an authored name.
            pub fn from_name(name: &str) -> Option<$t> {
                Self::ALL.iter().copied().find(|v| v.name() == name)
            }
        }
    };
}

named!(LandHolder, LandHolder::Breaker => "breaker", LandHolder::Settlement => "settlement");
named!(LandUse, LandUse::Holder => "holder", LandUse::Need => "need");
named!(
    Succession,
    Succession::Heir => "heir",
    Succession::Divided => "divided",
    Succession::Settlement => "settlement",
);

/// What a lease is, where a regime allows one (ADR-0007 §3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeaseRules {
    /// The share of the grain threshed from a let field that goes to its holder.
    pub holder_share: f64,
    /// Crop years a lease runs before it is renewed or ends.
    pub term_years: u32,
}

/// A property regime: who holds, works, lets and inherits land (ADR-0007 §2). Chosen for a
/// world when it is created; its name is content, not a state the engine switches between.
#[derive(Clone, Debug, PartialEq)]
pub struct RegimeDef {
    /// Content id, for example `core:regime/household`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What it means for the people who live under it, in a sentence or two.
    pub description: String,
    /// The regime a new world gets unless another is chosen.
    pub is_default: bool,
    /// Who holds ground once it is broken.
    pub holder: LandHolder,
    /// How its use is given.
    pub land_use: LandUse,
    /// Under [`LandUse::Need`], the day of the year (0 = 1 January) of the yearly review.
    pub review_day: u16,
    /// What becomes of a household's holdings when it is no more.
    pub succession: Succession,
    /// A new couple's household takes a share of its families' fields, as of their stores.
    pub union_share: bool,
    /// Whether use may be let, and on what terms.
    pub lease: Option<LeaseRules>,
}

impl RegimeDef {
    /// The rules every world lived by before regimes (M1 to M3a slice J): the household that
    /// breaks ground holds and works it, and when a household is no more everything goes to one
    /// heir. For content with no regimes, and worlds whose regime the content no longer has.
    pub fn legacy() -> RegimeDef {
        RegimeDef {
            id: String::new(),
            name: "Household fields".to_owned(),
            description: "A household holds the ground it breaks and leaves it to one heir."
                .to_owned(),
            is_default: true,
            holder: LandHolder::Breaker,
            land_use: LandUse::Holder,
            review_day: 45,
            succession: Succession::Heir,
            union_share: false,
            lease: None,
        }
    }
}

/// A recipe: what goes in, what comes out, the work and the tools (ADR-0006 §2).
#[derive(Clone, Debug, PartialEq)]
pub struct RecipeDef {
    /// Content id, for example `core:recipe/grind_grain`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Inputs per unit made: `(good, amount in its unit)`.
    pub inputs: Vec<(usize, f64)>,
    /// Outputs per unit made.
    pub outputs: Vec<(usize, f64)>,
    /// Inputs per session, whatever its size (the fuel to heat an oven).
    pub session_inputs: Vec<(usize, f64)>,
    /// Labour per unit, hours of a capable adult of middling skill.
    pub unit_h: f64,
    /// Labour per session, hours.
    pub session_h: f64,
    /// Most units one session makes; 0 for no limit beyond time and inputs.
    pub max_units: f64,
    /// Tools it needs and wears, by index in the goods.
    pub tools: Vec<usize>,
    /// The skill it uses and trains, by index in the skills.
    pub skill: Option<usize>,
    /// The technique working it needs, by index in the techniques (ADR-0008 §1).
    pub technique: Option<usize>,
}

/// A technique: a practical capability people know, learn and can lose (ADR-0008 §1). What it
/// gates is named by the work: recipes, activities and building programs name the technique
/// they need.
#[derive(Clone, Debug, PartialEq)]
pub struct TechniqueDef {
    /// Content id, for example `core:technique/emmer_growing`.
    pub id: String,
    /// Display name, for example "Growing emmer".
    pub name: String,
    /// What a competent person can do, completing "A competent person can …".
    pub can: String,
    /// The skill whose practice it is, by index in the skills.
    pub domain: Option<usize>,
    /// Prerequisites: alternative routes, each a set of techniques by index; empty for none.
    pub requires: Vec<Vec<usize>>,
    /// Activities whose practice counts toward finding it, by index in the activities.
    pub tried_in: Vec<usize>,
    /// Goods a household must hold to try it (the feasibility gate), by index in the goods.
    pub needs: Vec<usize>,
    /// Qualified hours of experiment to a median find (E50, research 07-01 §5.3).
    pub e50_h: f64,
    /// Hours of work beside someone who knows it that teach it.
    pub learn_h: f64,
    /// Children brought up in a household that knows it learn it at the work's age.
    pub upbringing: bool,
    /// The household problem it answers, which draws people to try toward it (ADR-0008 §3):
    /// goods lost to spoilage, by index in the goods.
    pub answers_spoilage: Vec<usize>,
}

/// The authored activities, goods and crops.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    /// Activities, in content id order. Their index is how people and saves refer to them.
    pub activities: Vec<ActivityDef>,
    /// Goods, in content id order. Their index is how stores refer to them.
    pub goods: Vec<GoodDef>,
    /// Crops, in content id order. Their index is how fields refer to them.
    pub crops: Vec<CropParams>,
    /// Building programs, in content id order.
    pub buildings: Vec<BuildingDef>,
    /// Recipes, in content id order.
    pub recipes: Vec<RecipeDef>,
    /// Skills, in content id order. Their index is how people's skills refer to them.
    pub skills: Vec<SkillDef>,
    /// Property regimes, in content id order (ADR-0007). A world keeps its regime's id.
    pub regimes: Vec<RegimeDef>,
    /// Techniques, in content id order (ADR-0008). Their index is how people's knowledge
    /// refers to them.
    pub techniques: Vec<TechniqueDef>,
    /// Policy templates, in content id order (ADR-0013 §3). Laws refer to them by index, saves
    /// by content id.
    pub policies: Vec<crate::polity::PolicyDef>,
    /// Norm templates, in content id order (ADR-0016 §4). People's states refer to them by
    /// index, saves by content id.
    pub norms: Vec<crate::norm::NormDef>,
    /// Values, in content id order (M4c slice AG, ADR-0016 §4). What people hold refers to them
    /// by index, saves by content id.
    pub values: Vec<crate::values::ValueDef>,
    /// Ideologies, in content id order (M4c slice AG, ADR-0016 §4). Holdings refer to them by
    /// index, saves by content id.
    pub ideologies: Vec<crate::ideology::IdeologyDef>,
}

impl Catalog {
    /// The regime with content id `id`.
    pub fn regime(&self, id: &str) -> Option<&RegimeDef> {
        self.regimes.iter().find(|r| r.id == id)
    }

    /// The default regime (content validation guarantees exactly one), if there are any.
    pub fn default_regime(&self) -> Option<&RegimeDef> {
        self.regimes
            .iter()
            .find(|r| r.is_default)
            .or(self.regimes.first())
    }
}

/// An authored building program with what people decide when they design one (ADR-0004 §2,
/// ADR-0009 §1): a hut, or a frame building in bays.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildingDef {
    /// Content id, for example `core:building/hut`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What its buildings are for: a dwelling, a store or a workshop.
    pub use_: civ_land::PlotUse,
    /// Its grammar's rules: every dimension, labour and material figure.
    pub rules: civ_grammar::ProgramRules,
    /// The good each material slot is made of, by index in the goods.
    pub materials: Vec<usize>,
    /// Wall height people build to, centimetres.
    pub eave_cm: i32,
    /// Roof pitch people build to, hundredths of a degree.
    pub pitch_centideg: i32,
    /// The day of the year a household wants to be under a roof by.
    pub roof_by_day: u16,
    /// The technique building it needs, by index in the techniques (ADR-0008 §1).
    pub technique: Option<usize>,
    /// The sizes people build a frame program's bays and members to (`None` for a hut).
    pub design: Option<FrameDesign>,
    /// Every shape people would build it in, with what each gives and costs, cheapest first
    /// ([`crate::build::shapes`]; derived from the rest when the content is compiled).
    pub shapes: Vec<crate::build::ShapeCost>,
    /// The skill building it uses and trains, by index in the skills (ADR-0009 §6).
    pub skill: Option<usize>,
    /// How its parts wear and decay (ADR-0009 §4).
    pub upkeep: Upkeep,
}

/// How one kind of a building's parts wears or decays (ADR-0009 §4).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wear {
    /// Share of its section or covering lost a year.
    pub per_year: f64,
    /// The share lost at which it shows.
    pub shows_at: f64,
}

/// How a building program's parts wear and decay (ADR-0009 §4), by kind of component group.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Upkeep {
    /// A covering in the weather: it leaks once it shows.
    pub covering: Wear,
    /// Posts set in the ground, at their foot, on ground of wetness 1.
    pub posts: Wear,
    /// Wattle and daub, at a wall's foot.
    pub infill: Wear,
    /// Timber under a covering, at a full leak (and not at all under a sound one).
    pub under_leak: Wear,
}

/// The sizes people build a frame program to (ADR-0009 §2): its bays' length and width, its
/// posts', walls', joists' and eaves' sizes, and how high its floor is raised. What varies between buildings of it is how many bays,
/// storeys and lofts they have.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameDesign {
    /// Length of a bay, centimetres.
    pub bay_cm: i32,
    /// Width between the long walls, centimetres.
    pub width_cm: i32,
    /// Diameter of a post, centimetres.
    pub post_cm: i32,
    /// Thickness of a wall, centimetres.
    pub wall_cm: i32,
    /// How far the roof reaches beyond the walls, centimetres.
    pub overhang_cm: i32,
    /// Diameter of a joist, centimetres.
    pub joist_cm: i32,
    /// Height of a raised floor above the ground, centimetres (0 for a floor on the ground).
    pub floor_raise_cm: i32,
}

impl BuildingDef {
    /// The grammar its buildings are expanded by.
    pub fn grammar(&self) -> civ_grammar::Grammar {
        self.rules.grammar()
    }

    /// Its rules, if it is a hut program.
    pub fn hut(&self) -> Option<&civ_grammar::HutRules> {
        match &self.rules {
            civ_grammar::ProgramRules::Hut(r) => Some(r),
            civ_grammar::ProgramRules::Frame(_) => None,
        }
    }
}

impl Catalog {
    /// The activity with this content id.
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.activities.iter().position(|a| a.id == id)
    }

    /// The good with this content id.
    pub fn good_index(&self, id: &str) -> Option<usize> {
        self.goods.iter().position(|g| g.id == id)
    }

    /// The crop with this content id.
    pub fn crop_index(&self, id: &str) -> Option<usize> {
        self.crops.iter().position(|c| c.id == id)
    }

    /// The building program with this content id.
    pub fn building_index(&self, id: &str) -> Option<usize> {
        self.buildings.iter().position(|b| b.id == id)
    }

    /// Whether program `id` is a home: a dwelling, not a store or a workshop.
    pub fn is_dwelling(&self, id: &str) -> bool {
        self.use_of(id) == Some(civ_land::PlotUse::Dwelling)
    }

    /// What program `id`'s buildings are for, if the content has it.
    pub fn use_of(&self, id: &str) -> Option<civ_land::PlotUse> {
        self.building_index(id).map(|i| self.buildings[i].use_)
    }

    /// The skill with this content id.
    pub fn skill_index(&self, id: &str) -> Option<usize> {
        self.skills.iter().position(|s| s.id == id)
    }

    /// The technique with this content id.
    pub fn technique_index(&self, id: &str) -> Option<usize> {
        self.techniques.iter().position(|t| t.id == id)
    }

    /// The technique an activity's work needs: its own, or its recipe's (ADR-0008 §1). Building
    /// work needs the technique of the building it works on, which the activity does not name.
    pub fn technique_of(&self, def: &ActivityDef) -> Option<usize> {
        def.technique.or_else(|| {
            def.recipe
                .and_then(|r| self.recipes.get(r))
                .and_then(|r| r.technique)
        })
    }

    /// The youngest age at which anyone does work that technique `t` gates, years: the age a
    /// child brought up with it learns it (ADR-0008 §4). Building programs are worked by the
    /// `build` activities.
    pub fn work_age(&self, t: usize) -> Option<f64> {
        let builds = self.buildings.iter().any(|b| b.technique == Some(t));
        self.activities
            .iter()
            .filter(|a| {
                self.technique_of(a) == Some(t) || (builds && a.behavior == Behavior::Build)
            })
            .map(|a| a.min_age_years)
            .reduce(f64::min)
    }

    /// The tools an activity needs: its own, or its recipe's, each once (its own first, in no
    /// particular order). Asked for every decision, so it allocates nothing.
    pub fn tools_of<'a>(&'a self, def: &'a ActivityDef) -> impl Iterator<Item = usize> + 'a {
        let own = def.tools.as_slice();
        let recipe = def
            .recipe
            .and_then(|r| self.recipes.get(r))
            .map_or(&[][..], |r| r.tools.as_slice());
        let first = |list: &'a [usize]| {
            list.iter()
                .enumerate()
                .filter(move |&(i, t)| !list[..i].contains(t))
                .map(|(_, &t)| t)
        };
        first(own).chain(first(recipe).filter(move |t| !own.contains(t)))
    }
}

/// Basal metabolism and body size (research 05-02 §2.1–2.2; FAO/WHO/UNU Schofield equations).
#[derive(Clone, Debug, PartialEq)]
pub struct EnergyParams {
    /// Basal metabolic rate per age band, `(slope kcal/kg/day, intercept kcal/day)`, for males.
    /// Bands start at [`EnergyParams::bmr_band_starts`].
    pub bmr_male: Vec<(f64, f64)>,
    /// The same for females.
    pub bmr_female: Vec<(f64, f64)>,
    /// First age of each BMR band, years.
    pub bmr_band_starts: Vec<f64>,
    /// Body mass by age: `(age years, male kg, female kg)`, ascending.
    pub mass_by_age: Vec<(f64, f64, f64)>,
    /// Physical activity ratio while walking.
    pub walk_par: f64,
    /// Physical activity ratio between activities (standing about).
    pub idle_par: f64,
    /// Hours a meal keeps someone full.
    pub satiety_hours: f64,
    /// Hours from the end of satiety to full hunger.
    pub hunger_ramp_hours: f64,
    /// Energy deficit that adds one unit of hunger, kcal.
    pub deficit_unit_kcal: f64,
    /// Largest energy surplus the body banks, kcal.
    pub max_surplus_kcal: f64,
    /// Energy the body can draw on in a shortage, kcal per kilogram of body mass: the floor of
    /// the energy balance.
    pub reserve_kcal_per_kg: f64,
    /// Food kept back (seed) is eaten only once a person has drawn this share of that reserve:
    /// people go hungry for days before they eat next year's sowing.
    pub eat_reserve_at_deficit: f64,
    /// Minutes a meal takes.
    pub meal_minutes: u32,
}

/// Sleep pressure, two-process model (research 04-02 §2.2, 01-09).
#[derive(Clone, Debug, PartialEq)]
pub struct SleepParams {
    /// Time constant of pressure building while awake, hours.
    pub tau_awake_h: f64,
    /// Time constant of pressure falling while asleep, hours.
    pub tau_asleep_h: f64,
    /// Pressure at which a sleeper wakes.
    pub wake_pressure: f64,
    /// Shortest night's sleep, hours.
    pub min_hours: f64,
    /// Longest night's sleep, hours.
    pub max_hours: f64,
    /// Shortest daytime nap, minutes.
    pub nap_min_minutes: f64,
    /// Longest daytime nap, minutes.
    pub nap_max_minutes: f64,
    /// Weight of sleep pressure in daylight, relative to night (circadian factor).
    pub day_factor: f64,
    /// Usual bedtime, hours after sunset. The weight keeps its daylight value until an hour
    /// before, then rises to its night value: alertness holds through the evening and the sleep
    /// gate opens near habitual bedtime (research 04-02: sleep onset about 3.3 h after sunset).
    pub bedtime_after_sunset_hours: f64,
}

/// Relatedness: closeness to others, eased toward the quality of present company
/// (research 04-01 §2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct SocialParams {
    /// Time constant, hours.
    pub tau_h: f64,
    /// Quality gained per companion present when sitting together.
    pub quality_per_companion: f64,
    /// Quality of time spent among one's household without sitting together.
    pub household_quality: f64,
}

/// What a household keeps and carries.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseholdParams {
    /// Water used per person per day, litres (research 10-01 §2.1).
    pub water_l_per_person_day: f64,
    /// Litres one person carries per trip.
    pub carry_water_l: f64,
    /// Days of water a household tries to keep.
    pub water_target_days: f64,
    /// Days of food a household tries to keep.
    pub food_target_days: f64,
    /// Days of food ready to eat (bread, porridge meal) a household tries to keep in hand.
    pub ready_food_days: f64,
    /// Days of food beyond its next harvest a household wants in store to see it through.
    pub harvest_margin_days: f64,
    /// How many times as long goods keep on a raised store's floor as elsewhere under a roof
    /// (ADR-0009 §5; research 08-02 §7.1).
    pub raised_store_factor: f64,
    /// Days of each food that is a step from ready (flour) a household tries to keep.
    pub processed_food_days: f64,
    /// What one person carries home, kilograms.
    pub carry_kg: f64,
    /// Firewood a household burns per member per day, by month, January first, kilograms.
    pub fuel_kg_per_person_day: [f64; 12],
    /// Days of firewood a household tries to keep.
    pub fuel_target_days: f64,
    /// Days of food in a settlement's stores below which the chronicle notes a shortage.
    pub short_food_days: f64,
    /// Days of food above which the chronicle notes that a shortage is over.
    pub recovered_food_days: f64,
    /// Food energy a person needs per day on average, kcal (for days-of-supply arithmetic).
    pub daily_kcal_per_person: f64,
    /// A household out of food whose members have drawn on average this share of their bodies'
    /// reserve may give up and leave the valley...
    pub leave_at_depletion: f64,
    /// ...with at most this chance a day...
    pub leave_per_day: f64,
    /// ...unless a crop of theirs ripens within this many days.
    pub leave_unless_ripe_within_days: f64,
    /// The day's chance is weighed (M4a slice Z): points toward going for the whole of the wait to
    /// its next harvest that its food, what others could spare it and any relief it may ask for
    /// would not cover...
    pub leave_w_gap: f64,
    /// ...points toward staying for a whole year's food its fields should bring, what leaving
    /// gives up...
    pub leave_w_stake: f64,
    /// ...and points toward staying before either; the day's chance is `leave_per_day` times the
    /// logistic of going's points less staying's.
    pub leave_stay: f64,
}

/// How choices are scored and sampled (research 01-09 §4.3, 04-07 §2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct DecisionParams {
    /// Softmax temperature as a fraction of the spread (standard deviation) of the candidates'
    /// utilities.
    pub temperature_sd_fraction: f64,
    /// Lowest temperature, utility points.
    pub min_temperature: f64,
    /// Points per unit of hunger.
    pub w_hunger: f64,
    /// Hunger beyond this adds nothing to the drive to find food: someone long hungry is not
    /// ever more driven, only weaker (a saturating response curve).
    pub max_hunger_drive: f64,
    /// Points per unit of sleep drive.
    pub w_sleep: f64,
    /// Points per unit of loneliness in the evening.
    pub w_social: f64,
    /// Points per unit of food shortage times the share of a day's food a trip brings.
    pub w_food: f64,
    /// Points per unit of the shortfall of stores against what will see the household through
    /// to its next harvest, times the worth of a trip for wild food.
    pub w_lean: f64,
    /// Points for useful work regardless of shortage (purpose).
    pub w_work: f64,
    /// Points per unit of firewood shortage times the worth of a trip.
    pub w_fuel: f64,
    /// Points per unit of the worth of field work: food for the year ahead.
    pub w_farm: f64,
    /// Points per unit of urgency: field work left over the work the household can still do
    /// before the season closes.
    pub w_deadline: f64,
    /// Points for building a household's roof, and for gathering what it is built of.
    pub w_shelter: f64,
    /// Points for making a tool the household lacks, and for gathering what it is made of.
    pub w_tools: f64,
    /// Days of household food a gathering trip must bring to be worth half as much as a very
    /// large haul.
    pub trip_half_worth_days: f64,
    /// Points per unit of water shortage.
    pub w_water: f64,
    /// Points lost per hour of walking.
    pub w_walk_hour: f64,
    /// Points lost per hour of hard work when tired, per unit of PAR above resting.
    pub w_effort: f64,
    /// Points lost by outdoor work that would run into darkness.
    pub w_dark: f64,
    /// Points for resting.
    pub w_rest: f64,
    /// Points for children's play.
    pub w_play: f64,
}

/// The first people of a new world (research 05-01 §4.5).
#[derive(Clone, Debug, PartialEq)]
pub struct BandParams {
    /// People in a founding band unless the player chooses otherwise.
    pub default_size: u32,
    /// Smallest band the new-world dialog allows.
    pub min_size: u32,
    /// Largest band the new-world dialog allows.
    pub max_size: u32,
    /// Fewest unrelated families in a band.
    pub min_families: u32,
    /// Candidate camp sites scored when a band arrives.
    pub camp_candidates: u32,
    /// Distance within which people judge a camp site's surroundings, metres.
    pub site_radius_m: f64,
    /// Days of food each household carries in.
    pub provisions_days: f64,
    /// The good they carry it as, by index in the catalog's goods.
    pub provisions_good: usize,
    /// Seed each person brings, kilograms of the crop's seed good.
    pub seed_kg_per_person: f64,
    /// Chance that a family brings an elder.
    pub elder_chance: f64,
    /// Chance that a family brings an unmarried young adult.
    pub young_adult_chance: f64,
    /// Months between births when building founding families.
    pub birth_spacing_months: f64,
    /// Steepest ground a camp is made on, rise over run.
    pub site_max_slope: f64,
    /// Points per natural-log unit of the wild food around a site, measured in years of the
    /// band's needs.
    pub site_w_food: f64,
    /// Points per natural-log unit of the arable land within a field walk of a site, measured
    /// in the area the band means to crop (`farm.max_walk_minutes` of off-trail walking).
    pub site_w_arable: f64,
    /// Points lost per 100 m from fresh water.
    pub site_w_water_per_100m: f64,
    /// Points lost per percent of slope.
    pub site_w_slope_per_pct: f64,
    /// Points lost by a site lower than [`BandParams::site_flood_hand_m`] above the nearest
    /// stream.
    pub site_w_flood: f64,
    /// Height above the nearest stream below which a site floods, metres.
    pub site_flood_hand_m: f64,
}

/// How households farm: the crop they know, how much of their food they plan to grow, and where
/// their fields go (research 08-02 §10, 10-01 §2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct FarmParams {
    /// The crop, by index in the catalog's crops.
    pub crop: usize,
    /// Share of a year's food a household plans to grow.
    pub grain_share: f64,
    /// Share of the crop's yield a household counts on when planning (a cautious harvest).
    pub plan_yield_share: f64,
    /// Share of the grain grown that is lost before it is eaten: in store, at the quern, and as
    /// food made ready goes off. A household grows its food over one less this.
    pub loss_share: f64,
    /// Days of grain a household aims to hold: beyond this a harvest is worth less.
    pub grain_target_days: f64,
    /// Hours of field work a capable adult gives a day, for planning what can be done in time.
    pub work_hours_per_day: f64,
    /// Hours of field work a capable adult gives at a peak, on a day the ground can be worked:
    /// what makes up the days the weather takes from a sowing window (ADR-0012 §5).
    pub peak_work_hours_per_day: f64,
    /// Side of a new field, metres.
    pub field_m: f64,
    /// Longest walk to a field, minutes.
    pub max_walk_minutes: f64,
    /// Sites sampled when marking out a new field.
    pub site_candidates: u32,
}

/// Siler mortality hazard per year at age `x` years: `A·e^(−Bx) + C + D·e^(Ex)`
/// (research 05-01 §1.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Siler {
    /// Infant hazard scale.
    pub a: f64,
    /// Infant hazard decay.
    pub b: f64,
    /// Constant adult hazard.
    pub c: f64,
    /// Senescent hazard scale.
    pub d: f64,
    /// Senescent hazard growth.
    pub e: f64,
}

impl Siler {
    /// Hazard per year at `age` years.
    pub fn hazard(&self, age: f64) -> f64 {
        self.a * (-self.b * age).exp() + self.c + self.d * (self.e * age).exp()
    }

    /// Probability of surviving from birth to `age` years.
    pub fn survival(&self, age: f64) -> f64 {
        // ∫ h = A/B (1 − e^(−Bx)) + C x + D/E (e^(Ex) − 1)
        let cum = self.a / self.b * (1.0 - (-self.b * age).exp())
            + self.c * age
            + self.d / self.e * ((self.e * age).exp() - 1.0);
        (-cum).exp()
    }
}

/// Mortality: an all-cause baseline by age, what hunger adds to it, and childbirth (research
/// 05-01 §1.2–1.3, 05-02 §1.6 and §2.7). Hunger modifies the hazard; it is not a second copy of
/// it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MortalityParams {
    /// The baseline hazard by age.
    pub siler: Siler,
    /// How many times as likely to die a person is who has drawn half their body's reserve; the
    /// ratio is `ratio^(4·d²)` for a share `d` drawn, up to `hunger_ratio_max`.
    pub hunger_ratio_at_half: f64,
    /// The most hunger multiplies the baseline hazard by.
    pub hunger_ratio_max: f64,
    /// Hazard per day of a body that has drawn all its reserve.
    pub exhaustion_per_day: f64,
    /// How steeply that hazard rises toward the end of the reserve: `exhaustion_per_day · d^power`.
    pub exhaustion_power: f64,
    /// Mothers' deaths per live birth.
    pub maternal_death_per_birth: f64,
}

impl MortalityParams {
    /// Hazards per year at `age` for a body that has drawn the share `depleted` (0–1) of its
    /// reserve: the baseline, and what hunger adds to it.
    pub fn hazards(&self, age: f64, depleted: f64) -> (f64, f64) {
        let base = self.siler.hazard(age.max(0.0));
        let d = depleted.clamp(0.0, 1.0);
        let ratio = self
            .hunger_ratio_at_half
            .max(1.0)
            .powf(4.0 * d * d)
            .min(self.hunger_ratio_max.max(1.0));
        let exhaustion =
            self.exhaustion_per_day * DAYS_PER_YEAR as f64 * d.powf(self.exhaustion_power);
        (base, base * (ratio - 1.0) + exhaustion)
    }
}

/// Conception, pregnancy and the months after a birth (research 05-01 §1.4–1.5 and §2.2, 04-08
/// §1.5 and §2.3): a state machine whose output, not its input, is a fertility schedule.
#[derive(Clone, Debug, PartialEq)]
pub struct FertilityParams {
    /// Chance a fecund woman living with her partner conceives in a month, at her most fecund
    /// ages.
    pub conception_per_month: f64,
    /// Fecundability relative to that, from each age (years) until the next: `(age, factor)`,
    /// ascending; nothing below the first age.
    pub age_factor: Vec<(f64, f64)>,
    /// Standard deviation of the logarithm of each woman's lasting fecundability factor (mean 1).
    pub fecundity_sd: f64,
    /// Fecundability halves for each this share of the body's reserve drawn.
    pub hunger_halving: f64,
    /// Days from conception to a birth at term: mean and standard deviation.
    pub pregnancy_days: f64,
    /// Standard deviation of `pregnancy_days`.
    pub pregnancy_sd_days: f64,
    /// Chance a pregnancy is lost, from each age of the mother at conception until the next.
    pub loss_by_age: Vec<(f64, f64)>,
    /// Days after conception a loss comes: from, to.
    pub loss_days: [f64; 2],
    /// Months after a live birth before the mother can conceive: mean.
    pub recovery_months: f64,
    /// Standard deviation of `recovery_months`.
    pub recovery_sd_months: f64,
    /// Fewest months of `recovery_months`.
    pub recovery_min_months: f64,
    /// Months after a loss before she can conceive.
    pub loss_recovery_months: f64,
    /// Months after a nursing child dies before its mother can conceive.
    pub weaned_recovery_months: f64,
    /// Boys born per 100 girls.
    pub boys_per_100_girls: f64,
    /// Extra energy a pregnancy costs its mother in each trimester, kcal a day.
    pub pregnancy_kcal_day: [f64; 3],
}

/// Where a new couple lives (research 06-01 §1.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Residence {
    /// A household of their own.
    NewHousehold,
    /// The man's household.
    HisHousehold,
    /// The woman's household.
    HerHousehold,
}

impl Residence {
    /// Every residence rule, in a fixed order.
    pub const ALL: [Residence; 3] = [
        Residence::NewHousehold,
        Residence::HisHousehold,
        Residence::HerHousehold,
    ];

    /// The authored name of a residence rule.
    pub fn name(self) -> &'static str {
        match self {
            Residence::NewHousehold => "new_household",
            Residence::HisHousehold => "his_household",
            Residence::HerHousehold => "her_household",
        }
    }

    /// The residence rule with an authored name.
    pub fn from_name(name: &str) -> Option<Residence> {
        Residence::ALL.into_iter().find(|r| r.name() == name)
    }
}

/// Couples and households (research 04-08 §1.1, §1.4 and §2.1; 06-01 §1.3–1.4 and §2.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FamilyParams {
    /// Ages from which women and men look for a partner: `[female, male]`.
    pub seek_min_age: [f64; 2],
    /// Ages after which they no longer look: `[female, male]`.
    pub seek_max_age: [f64; 2],
    /// Chance an unpartnered woman or man of those ages looks in a month: `[female, male]`.
    pub seek_per_month: [f64; 2],
    /// How much older than the woman the man of a couple may be, years: from (negative: younger)
    /// and to.
    pub age_gap_years: [f64; 2],
    /// The gap people look for, years (the man older).
    pub preferred_gap_years: f64,
    /// Points a candidate loses per year away from that gap.
    pub w_gap_per_year: f64,
    /// No couple shares an ancestor within this many generations, or has one partner descend from
    /// the other within them (2: parents, siblings, half-siblings, first cousins, aunts and
    /// uncles).
    pub kin_exclusion_generations: u32,
    /// Where a new couple lives.
    pub residence: Residence,
    /// Age from which someone can keep a household without an older member.
    pub independent_age: f64,
    /// How much of the mean of its parents' personality a child inherits (the regression of
    /// offspring on mid-parent).
    pub trait_heritability: f64,
}

impl FamilyParams {
    /// Index of a sex in the `[female, male]` pairs.
    pub fn of(sex: crate::needs::Sex) -> usize {
        match sex {
            crate::needs::Sex::Female => 0,
            crate::needs::Sex::Male => 1,
        }
    }

    /// Whether someone of `sex` and `age` looks for a partner (and can be one).
    pub fn seeks_at(&self, sex: crate::needs::Sex, age: f64) -> bool {
        let i = Self::of(sex);
        age >= self.seek_min_age[i] && age < self.seek_max_age[i]
    }
}

/// The value of a step table at `x`: the factor of the last age at or below it, 0 below the
/// first.
pub fn step_at(table: &[(f64, f64)], x: f64) -> f64 {
    table
        .iter()
        .take_while(|(age, _)| *age <= x)
        .last()
        .map_or(0.0, |(_, v)| *v)
}

/// Names (research 06-07): given names by sex, and parts for place names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NameParams {
    /// Given names for males.
    pub male: Vec<String>,
    /// Given names for females.
    pub female: Vec<String>,
    /// First parts of place names.
    pub place_first: Vec<String>,
    /// Second parts of place names.
    pub place_second: Vec<String>,
}

/// How households trade (slice I, ADR-0006 §4).
#[derive(Clone, Debug, PartialEq)]
pub struct MarketParams {
    /// Days between a household's reviews of what it offers and on what terms.
    pub review_days: u32,
    /// What a seller asks over its own cost, a share of it.
    pub margin: f64,
    /// The largest change of an ask in one review, a share of it.
    pub max_change: f64,
    /// How strongly a seller's ask for food answers what it holds beyond its needs: its anchor
    /// is its cost and margin times `exp(-this × s)`, `s` the years of its own need it can spare,
    /// at most one (research 08-04 §1.2: a cost anchor with modest inventory feedback).
    pub stock_response: f64,
    /// Half-life of what a market remembers of sales, payments and demand, days.
    pub memory_days: f64,
    /// The share of the payments' worth one good must settle for it to be the settlement's
    /// money.
    pub money_share: f64,
    /// Trades a market must remember before one good can be its money.
    pub money_min_trades: f64,
    /// A seller accepts a good in payment when it wants at least this much more of it (1 when
    /// it is short of it), or when the good is its settlement's money.
    pub accept_want: f64,
    /// Trades a market keeps in its list of the latest.
    pub recent_trades: usize,
}

/// Household workshops (slice J, ADR-0006 §5).
#[derive(Clone, Debug, PartialEq)]
pub struct FirmParams {
    /// Days without a sale after which a workshop's owners give it up.
    pub idle_close_days: f64,
    /// Entries a firm's books keep in full (its monthly statements are kept for its life).
    pub book_entries: usize,
    /// The share of what an hour's work adds that a workshop first offers for it.
    pub wage_share: f64,
    /// Days between reviews of a workshop's wage.
    pub wage_review_days: u32,
    /// The largest change of a wage in one review, a share of it.
    pub wage_max_change: f64,
    /// Most hours of work a workshop hires between two of its reviews.
    pub max_hire_hours: f64,
}

/// What households build (ADR-0009 §1, §7).
#[derive(Clone, Debug, PartialEq)]
pub struct BuildParams {
    /// The programs households may build, by index in the catalog's buildings, in the profile's
    /// order: homes, stores and workshops. A household builds one someone in it knows how to.
    pub programs: Vec<usize>,
    /// Days over which a household reckons what a storehouse would save of the goods it holds:
    /// what they would lose in the open over these days less what they would lose in it.
    pub store_horizon_days: f64,
    /// People who can work at a craft at once in a household's home, beside living there: a firm
    /// that has more working for it at once builds a workshop.
    pub home_work_places: u32,
    /// How unevenly the parts of a building are made, by a novice and by a master builder: a
    /// group's quality is one less this times the size of a normal draw (ADR-0009 §6).
    pub quality_spread: [f64; 2],
    /// How builders answer the failures their settlement has seen (ADR-0009 §6).
    pub caution: crate::caution::CautionParams,
    /// How households level a plot on sloping ground (ADR-0010 §2).
    pub levelling: Levelling,
}

/// How households level the plot of a building on sloping ground (ADR-0010 §2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Levelling {
    /// Ground that drops more than this across a plot, metres, is levelled before building.
    pub from_m: f64,
    /// Ground that drops more than this across a plot, metres, is not built on.
    pub most_m: f64,
    /// Hours of a capable adult to cut a cubic metre of earth and place it where it is wanted.
    pub h_per_m3: f64,
    /// A platform's sides' run, metres across for each metre up or down.
    pub side_run: f64,
}

/// How people dig at a deposit (M3b slice Q, ADR-0010 §2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Digging {
    /// Hours of a capable adult to dig a cubic metre of earth as it lay in the ground and lift it
    /// out.
    pub h_per_m3: f64,
    /// The side of a pit, and of the spoil heap beside it, metres.
    pub pit_side_m: f64,
}

/// A taste in building, or the traits of a building (M3b slice R): the roof pitch, the height to
/// the eaves and the roof's overhang beyond the walls a household would build to, each held to
/// what a program allows when it builds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Taste {
    /// Roof pitch, hundredths of a degree.
    pub pitch_centideg: f32,
    /// Height of the walls to the eaves, centimetres.
    pub eave_cm: f32,
    /// How far the roof reaches beyond the walls, centimetres.
    pub overhang_cm: f32,
}

impl Taste {
    /// Its traits in order: pitch, eaves, overhang.
    pub fn traits(&self) -> [f32; 3] {
        [self.pitch_centideg, self.eave_cm, self.overhang_cm]
    }

    /// The taste with traits `t` in that order.
    pub fn from_traits(t: [f32; 3]) -> Taste {
        Taste {
            pitch_centideg: t[0],
            eave_cm: t[1],
            overhang_cm: t[2],
        }
    }
}

/// How households' taste in building moves (M3b slice R; research 11-02 §1.1, §2.2): toward the
/// buildings their settlement admires, a little at a time, with now and then something new.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleParams {
    /// How far a household's taste moves toward a new building of its settlement's that it
    /// admires most, as a share of the way (11-02 §2.2: 0.02-0.20 a meaningful encounter).
    pub alpha: f64,
    /// How many times more the most admired new building of a settlement's weighs than the least
    /// admired (11-02 §2.2: 1-3 times a neutral exemplar).
    pub prestige_most: f64,
    /// The chance a building has one trait new to its builders, drawn within what its program
    /// allows (11-02 §2.2: 0.1-3 % a commission).
    pub innovation: f64,
    /// The way of building a founding band's is drawn around: the content's.
    pub tradition_mean: Taste,
    /// How far a founding band's shared way of building lies from the content's, each trait's
    /// standard deviation in its units.
    pub tradition_spread: Taste,
    /// How far each founding household's taste lies from its band's, likewise.
    pub personal_spread: Taste,
    /// Buildings of other settlements a person keeps in mind until their household's next
    /// review, newest first (M5b slice AR; research 11-02 §5.5: 5-20 salient exemplars).
    pub seen_most: usize,
    /// How far a person at another settlement's hearth or a seller's door there sees its new
    /// buildings, metres (a design prior).
    pub sight_m: f64,
}

/// Everything authored about people.
#[derive(Clone, Debug, PartialEq)]
pub struct PeopleParams {
    /// Walking (research 01-08).
    pub nav: NavParams,
    /// Walking speed multiplier by age: `(age years, factor)`, ascending.
    pub walk_speed_by_age: Vec<(f64, f64)>,
    /// Work efficiency by age: `(age years, factor)`, ascending.
    pub capacity_by_age: Vec<(f64, f64)>,
    /// Latitude, degrees north, for day length.
    pub latitude_deg: f64,
    /// Energy and body.
    pub energy: EnergyParams,
    /// Sleep.
    pub sleep: SleepParams,
    /// Company.
    pub social: SocialParams,
    /// Household stores.
    pub household: HouseholdParams,
    /// Choice.
    pub decision: DecisionParams,
    /// Founding bands.
    pub band: BandParams,
    /// Farming.
    pub farm: FarmParams,
    /// Building.
    pub build: BuildParams,
    /// Mortality.
    pub mortality: MortalityParams,
    /// Conception, pregnancy and birth.
    pub fertility: FertilityParams,
    /// Couples and households.
    pub family: FamilyParams,
    /// Trade.
    pub market: MarketParams,
    /// Workshops.
    pub firm: FirmParams,
    /// What founders know and how people learn.
    pub knowledge: KnowledgeParams,
    /// How people dig at a deposit (M3b slice Q).
    pub digging: Digging,
    /// Taste in building and how it moves (M3b slice R).
    pub style: StyleParams,
    /// The midden and carrying it to the fields (M3c slice V).
    pub midden: MiddenParams,
    /// Ties between people (M4a slice Y, ADR-0014).
    pub ties: crate::ties::TieParams,
    /// Standing and notables (M4a slice Y, ADR-0014 §3-4).
    pub standing: crate::standing::StandingParams,
    /// The polity: its gathering, forecasts and compliance (M4a slice Z, ADR-0013).
    pub polity: crate::polity::PolityParams,
    /// Taking, what is seen of it and what is owed for it (M4b slice AA, ADR-0015).
    pub crime: crate::crime::CrimeParams,
    /// How word travels and grievances are held (M4c slice AE, ADR-0016).
    pub word: crate::word::WordParams,
    /// How opinion moves (M4c slice AG, ADR-0016 §4).
    pub opinion: crate::opinion::OpinionParams,
    /// How factions are founded, joined and kept (M4c slice AH, ADR-0017 §2).
    pub faction: crate::faction::FactionParams,
    /// How households come to know other places (M5a slice AM, ADR-0018 §4).
    pub places: crate::places::PlacesParams,
    /// What moving to another settlement is worth to a household (M5a slice AN, ADR-0018 §5).
    pub moving: crate::places::MovingParams,
    /// What founding a settlement of its own is worth to a household, and what a coalition must
    /// hold to go (M5a slice AO).
    pub founding: crate::places::FoundingParams,
    /// How price reports of other settlements' markets are held and passed on (M5b slice AP,
    /// ADR-0019 §1).
    pub reports: crate::reports::ReportParams,
    /// Names.
    pub names: NameParams,
}

/// A household's midden, the heap of ash, food waste, sweepings and dung beside its home, and
/// carrying it to the fields (M3c slice V; the people profile's `[midden]`, content API 29).
#[derive(Clone, Debug, PartialEq)]
pub struct MiddenParams {
    /// Kilograms a member adds to the heap a day.
    pub kg_per_person_day: f64,
    /// Kilograms of nitrogen a member's share of the heap holds a year.
    pub n_kg_per_person_year: f64,
    /// Days the heap takes to lose half of itself, and of its nitrogen, to the air and the rain.
    pub half_life_days: f64,
    /// Kilograms carried to a field in one load.
    pub load_kg: f64,
    /// Hours a capable adult takes to dig a tonne out of the heap and spread it, besides carrying
    /// it.
    pub spread_h_per_t: f64,
}

impl MiddenParams {
    /// Kilograms of nitrogen in a kilogram of the heap.
    pub fn n_per_kg(&self) -> f64 {
        let kg_year = self.kg_per_person_day * 365.0;
        if kg_year > 0.0 {
            self.n_kg_per_person_year / kg_year
        } else {
            0.0
        }
    }

    /// Hours of a capable adult's work to dig out, carry and spread a kilogram on a field
    /// `walk_min` minutes' walk from home, a load at a time there and back.
    pub fn h_per_kg(&self, walk_min: f64) -> f64 {
        let trips = 1000.0 / self.load_kg.max(1.0);
        (self.spread_h_per_t.max(0.0) + trips * 2.0 * walk_min.max(0.0) / 60.0) / 1000.0
    }

    /// The heap of `members` people `days` after it held `kg`: what they add, less what it loses
    /// at its half-life (exact for a steady household).
    pub fn after(&self, kg: f64, members: usize, days: f64) -> f64 {
        let (kg, days) = (kg.max(0.0), days.max(0.0));
        let added = members as f64 * self.kg_per_person_day.max(0.0);
        if self.half_life_days <= 0.0 {
            return kg + added * days;
        }
        let rate = std::f64::consts::LN_2 / self.half_life_days;
        let keep = (-rate * days).exp();
        kg * keep + added / rate * (1.0 - keep)
    }
}

/// What founders know and how people learn from one another (ADR-0008).
#[derive(Clone, Debug, PartialEq)]
pub struct KnowledgeParams {
    /// The share of founders old enough for the work who know each technique:
    /// `(technique index, share 0-1)`. A band always brings at least one knower of a technique
    /// with a share above zero.
    pub founders: Vec<(usize, f64)>,
    /// Learners one person teaches at once (research 07-02 §2.3: 1-3).
    pub max_learners: u32,
    /// Utility points for working beside someone to learn what they know.
    pub w_learn: f64,
    /// The share of a session of routine work that counts as experiment toward the techniques
    /// its practice can find (research 07-01 §2.3: 1 %, tested 0-5 %).
    pub experiment_share: f64,
    /// How many times its hours a session spent trying counts for someone already aware of the
    /// technique: reconstruction is easier than invention (research 07-02 §1.7).
    pub aware_try_factor: f64,
    /// Utility points for trying toward a technique, times the share of the household's food
    /// that the problem it answers would cost.
    pub w_try: f64,
    /// Least days between one person's sessions of trying: what keeps trying to a small share of
    /// their time (research 07-01 §2.3; 07-11 §2.2).
    pub try_gap_days: f64,
    /// How far, metres, someone at another settlement's hearth or a seller's door there sees its
    /// people at work well enough to know of a technique the work needs (M5b slice AR; 0: never).
    pub watch_m: f64,
}

/// Linear interpolation in an ascending `(x, y)` table, clamped at its ends.
pub fn interpolate(table: &[(f64, f64)], x: f64) -> f64 {
    match table {
        [] => 0.0,
        [only] => only.1,
        _ => {
            if x <= table[0].0 {
                return table[0].1;
            }
            for w in table.windows(2) {
                let ((x0, y0), (x1, y1)) = (w[0], w[1]);
                if x <= x1 {
                    let t = if x1 > x0 { (x - x0) / (x1 - x0) } else { 1.0 };
                    return y0 + t * (y1 - y0);
                }
            }
            table[table.len() - 1].1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hadza_siler_gives_the_published_life_table() {
        // Research 05-01 §1.2: A .351, B .895, C .011, D 6.70e-6, E .125 gives 1q0 ≈ 216 ‰.
        let s = Siler {
            a: 0.351,
            b: 0.895,
            c: 0.011,
            d: 6.70e-6,
            e: 0.125,
        };
        let q0 = 1.0 - s.survival(1.0);
        assert!((q0 - 0.216).abs() < 0.03, "1q0 = {q0}");
        let q5 = 1.0 - s.survival(5.0);
        assert!((q5 - 0.358).abs() < 0.04, "5q0 = {q5}");
        assert!(s.survival(80.0) < s.survival(40.0));
    }

    #[test]
    fn interpolation_clamps_and_blends() {
        let t = [(0.0, 1.0), (10.0, 3.0)];
        assert_eq!(interpolate(&t, -5.0), 1.0);
        assert_eq!(interpolate(&t, 5.0), 2.0);
        assert_eq!(interpolate(&t, 50.0), 3.0);
        assert_eq!(interpolate(&[], 1.0), 0.0);
    }

    #[test]
    fn practice_closes_eighty_percent_of_the_gap_in_t80_hours() {
        let skill = SkillDef {
            id: "s".into(),
            name: "s".into(),
            t80_h: 100.0,
            speed: vec![(0.0, 0.5), (1.0, 1.5)],
            quality: vec![(0.0, 1.0), (1.0, 1.0)],
            founder_level: [0.0, 0.0],
        };
        assert!((skill.practised(0.0, 100.0) - 0.8).abs() < 1e-9);
        assert!((skill.practised(0.5, 100.0) - 0.9).abs() < 1e-9);
        // Practice in pieces comes to the same.
        let pieces = (0..10).fold(0.0, |s, _| skill.practised(s, 10.0));
        assert!((pieces - 0.8).abs() < 1e-9);
        assert_eq!(skill.practised(1.0, 50.0), 1.0);
        assert_eq!(skill.practised(0.3, 0.0), 0.3);
    }

    #[test]
    fn behaviors_round_trip_by_name() {
        for b in Behavior::ALL {
            assert_eq!(Behavior::from_name(b.name()), Some(b));
        }
        assert_eq!(Behavior::from_name("fly"), None);
    }
}
