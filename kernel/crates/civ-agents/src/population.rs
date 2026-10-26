//! The population: people and households, and how their activities run (ADR-0003).
//!
//! An activity is a list of steps (walk, work, deposit). Each step that takes time schedules one
//! event at its end carrying the person's permanent id and the activity version; when it fires,
//! the step's effects are applied and the next step starts, or, after the last, the person
//! decides what to do next. Needs are brought up to date only at step boundaries.

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use civ_core::time::{DAYS_PER_YEAR, MINUTES_PER_DAY};
use civ_core::{FastMap, GenTable, Handle, IdAllocator, PermanentId, Rng64, SimTime};
use civ_grammar::{BuildingSpec, Expansion, Stage, StageNeeds};
use civ_land::paths::{CellBox, cells_along};
use civ_land::{
    Building, CropParams, Field, FieldSoil, FieldStage, FieldTask, Land, LandParams, Party, Plot,
};
use civ_world::nav::{NavGrid, RouteResult, TravelField};
use civ_world::{WATER_LAKE, WATER_LAND, WATER_RIVER, WorldMap};

use crate::build::{self, HomeWork};
use crate::condition;
use crate::decide::{
    self, BuildOption, Facts, FieldOption, GiverOption, Limits, MakeOption, MakeWorth, PatchOption,
    WaterOption,
};
use crate::farm::{self, FarmView, Site};
use crate::history::{ChronicleEvent, ChronicleKind, PersonRecord, Reason, Receipt, Union};
use crate::ledger::{Channel, Leg};
use crate::make;
use crate::needs;
use crate::params::{
    ActivityDef, Behavior, Catalog, GoodDef, GoodUse, HouseholdParams, LandHolder, PeopleParams,
    RegimeDef, interpolate,
};
use crate::person::{
    Activity, Batch, Flow, Flows, Household, KnownPatch, Load, Person, Step, Target, Trip,
    food_kcal, fuel_kg, reserve_food_kcal, stock_kcal,
};

mod apart;
mod boating;
mod bottlenecks;
mod cases;
mod contagion;
mod crime;
mod crossings;
mod deposits;
mod digging;
mod ditches;
mod duties;
mod elections;
mod enclosures;
mod engines;
mod faction;
mod fire;
mod firm;
mod floods;
mod force;
mod founding;
mod gas;
mod holds;
mod ideology;
mod influence;
mod inoculation;
mod knowledge;
mod lamps;
mod land;
mod letters;
mod levees;
mod licences;
mod life;
mod loads;
mod market;
mod middens;
mod mills;
mod mint;
mod money;
mod moving;
mod norm;
mod opinion;
mod payments;
mod trains;
mod wagons;
mod war;
mod way_works;
pub use crime::TakeProbe;
pub use payments::Burden;
pub use resolution::{DayBlock, NOT_TAKEN, SwitchInvariants};
pub use sickhouse::sickhouse_bed_m2;
mod credit;
mod custody;
mod patronage;
mod places;
mod polity;
mod privileges;
mod register;
mod relations;
mod residence;
mod resolution;
mod rules;
mod school;
mod sickness;
mod stewardship;
mod suspicion;
mod terms;
mod values;
mod watch;
mod wells;
pub use wells::WellWeighed;
mod word;

pub use deposits::{DepositKnown, FIND_M};
pub use influence::{PURPOSE_IDEOLOGY_HEARD, Reached};
pub use loads::{
    DIES_IN_RUIN, DIES_UNDER_FLOOR, DIES_UNDER_ROOF, LIVE_PA, SNOW_TOLD_PA, SPILLED,
    snow_on_roof_pa, storm_pa,
};
mod plans;
mod quakes;
pub use plans::ParcelOffer;
pub use way_works::LAID_FROM_M;
mod raising;
pub use raising::{
    AT_TOWER_M, LATE_NOTICE_MIN, RAISE_WITHIN_M, SEEN_GIVERS, STALL_DAYS, TOWER_SIGHT_M,
};
mod sickhouse;
mod siting;
mod taste;
mod ties;
mod transfer;
mod travelled;
mod trust;

use knowledge::Gate;
pub use knowledge::{LOST_AWARE, LOST_MADE_REMAIN};

pub use life::{depleted, extra_kcal_day};
pub use residence::Accounts;

/// Keeps a route (or that there is none) in the newer generation of the route cache, retiring
/// that generation to the older when it is full. The cache is exact: a route kept is the one a
/// search would find again.
fn keep_route(
    routes: &mut FastMap<(u32, u32), Option<CachedRoute>>,
    old: &mut FastMap<(u32, u32), Option<CachedRoute>>,
    key: (u32, u32),
    route: Option<CachedRoute>,
    max: usize,
) {
    if routes.len() >= max {
        *old = std::mem::take(routes);
    }
    routes.insert(key, route);
}

/// Purpose tag for decision draws (ADR-0003 keyed randomness).
pub const PURPOSE_DECIDE: u64 = 0x6465_6369_6465_3031; // "decide01"
/// Most cells one route search may expand.
const ROUTE_BUDGET: usize = 600_000;
/// Further stretches of the water's edge a home knows beside the nearest (M6a slice BA, step
/// four): where its people go along the water when a nearer one is passed over. A few, as a
/// sponsor weighs a few sites (research 09-05 §3.3); an engineering choice.
const BANKS_KNOWN: usize = 3;
/// How far apart, metres, two stretches of the water's edge a home knows are at least: far
/// enough to be another place to stand and draw, not the next step along (an engineering choice).
const BANK_APART_M: f32 = 100.0;
/// Minutes a person waits before deciding again when a walk cannot be routed.
const WAIT_AFTER_FAILURE_MIN: u32 = 10;
/// Cells beyond the ground people have walked that route searches are bounded by landmarks
/// over ([`civ_world::nav::NavGrid::landmarks`]): half a wear tile, 256 m on 8 m cells.
const LANDMARK_MARGIN_CELLS: usize = 32;

/// Landmarks by region of walked ground: the regions as last surveyed (each with its margin; one,
/// the walked area, since M10a slice DO), and each region's landmarks once a search has asked for
/// them, with the box of cells they read ([`NavGrid::landmark_reach`]). Derived.
#[derive(Debug, Default)]
struct RegionLandmarks {
    /// The paths survey the regions were drawn from ([`civ_land::paths::Wear::rev`]).
    wear_rev: u32,
    regions: Vec<CellBox>,
    built: Vec<Option<(std::sync::Arc<civ_world::nav::Landmarks>, CellBox)>>,
}
/// Fewest routes the route cache keeps in each of its two generations: when the newer is full it
/// becomes the older, and the older is let go. A route found again is the route let go (the same
/// paths, the same search), so its size changes how fast the world lives, never what happens.
const ROUTE_CACHE_MAX: usize = 50_000;
/// ...and routes kept a generation for each person living, above that (M8a slice CJ: three
/// settlements of 3,334 search about 5,400 new routes a day).
const ROUTE_CACHE_PER_PERSON: usize = 8;
/// Farthest a household moves its home from where it stands to find clear ground to build on,
/// metres (a tuning value).
const HOME_SHIFT_M: f64 = 30.0;
/// Farthest a household with no home looks for clear ground when none lies within
/// [`HOME_SHIFT_M`], metres: one that came to a crowded hearth builds at the settlement's edge
/// rather than never (M5b slice AS; a tuning value, a short walk, within its fields' reach).
const FIRST_HOME_SHIFT_M: f64 = 120.0;
/// Least food a household makes ready in one go, in days of its needs: below this it waits until
/// it needs more (a tuning value).
const MIN_BATCH_DAYS: f64 = 0.25;
/// Hours of light a household holding a lamp keeps fuel for (M10b slice DU; a design prior: a
/// week of evenings).
const LAMP_FUEL_HOURS: f64 = 30.0;
/// Metres from home within which someone counts as at home for a lamp lit there.
const LAMP_AT_HOME_M: f32 = 2.0;
/// A settlement's first trail, for the chronicle: at least this long, metres...
pub const FIRST_TRAIL_M: f32 = 150.0;
/// ...and passing within this of the settlement's hearth, metres.
pub const FIRST_TRAIL_NEAR_M: f32 = 80.0;

/// Who may be keeping company at a settlement's hearth, in id order (`Population::at_hearth`).
type AtHearth = Vec<(PermanentId, Handle<Person>)>;

/// A route at standard walking speed, pulled straight across open ground: vertices (cell centres)
/// and seconds to each.
type CachedRoute = std::sync::Arc<(Vec<(f32, f32)>, Vec<f32>)>;

/// Something a person scheduled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AgentEvent {
    /// The current step of the person's activity ends.
    Step {
        /// Who.
        person: PermanentId,
        /// The activity version it belongs to.
        version: u32,
    },
    /// A minute of every fire burning (M6b slice BC, ADR-0022 §4): one is pending whenever a
    /// fire burns.
    Fire,
    /// A home's fires in use may escape now (M6b slice BD, ADR-0022 §3): drawn at midnight.
    Ignite {
        /// The home.
        building: PermanentId,
    },
    /// Those about look at the bands walking (M6c slice BJ, ADR-0023 §8): one is pending
    /// whenever a band is out.
    BandLook,
    /// Gates are kept (M6c slice BK, step three): at first light each is opened, and at dusk the
    /// household nearest each chooses whether to shut it for the night.
    Gates,
    /// Trains are due (M10b slice DX, second part): each household with an engine brings its
    /// arrived trains to account and sets off where its line's timetable has a run now. Pending
    /// at each of the day's departures while any engine is made.
    Trains,
    /// Gas is lit at dusk (M10b slice DY, third part): each main's evening is checked and its
    /// connections lit or not. Pending each day while any main is laid.
    Gas,
    /// A walker reaches a bottleneck on their walk (M8b slice CN, ADR-0029 §4).
    Pass {
        /// Who.
        person: PermanentId,
        /// The activity version and the trip it belongs to.
        version: u32,
        trip: u64,
    },
}

/// The approximations the kernel makes as it advances (ADR-0011 §4), each on or off on its own.
/// Accelerated mode makes them; Detailed mode makes none, and a test can switch them off to show
/// that Accelerated mode is otherwise Detailed mode a day at a time. Never world state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Approximations {
    /// Rest, the hearth and play last until the next consequential boundary (a meal, the sleep
    /// threshold, sunrise or sunset, the evening, the last light for water), as long as the
    /// activity's longest session, instead of a session at a time.
    pub leisure_blocks: bool,
    /// A household's view of its options (what it would buy, whom it would ask, where it would
    /// work, gather, dig and farm) is kept from its first decision after midnight until midnight
    /// or its next consequential step, instead of being worked out at every decision.
    pub household_view: bool,
    /// When goods move between households (a gift, a trade, paid work, a fetch, a taking), only
    /// the views of the two parties and of households whose kept choice names one of them are
    /// let go, not every household's (M8a slice CL, ADR-0028 §6). Only with `household_view`.
    pub views_by_party: bool,
}

impl Approximations {
    /// None: every event as Detailed mode lives it.
    pub const NONE: Approximations = Approximations {
        leisure_blocks: false,
        household_view: false,
        views_by_party: false,
    };
    /// Every approximation Accelerated mode declares.
    pub const ACCELERATED: Approximations = Approximations {
        leisure_blocks: true,
        household_view: true,
        views_by_party: true,
    };
}

/// The world a person acts in, borrowed for one event.
#[derive(Debug)]
pub struct Ctx<'a> {
    /// The current time.
    pub now: SimTime,
    /// The world seed (for keyed draws).
    pub seed: u64,
    /// Terrain and water.
    pub map: &'a WorldMap,
    /// Walkability.
    pub nav: &'a NavGrid,
    /// Land state.
    pub land: &'a mut Land,
    /// Land parameters.
    pub land_params: &'a LandParams,
    /// People parameters.
    pub params: &'a PeopleParams,
    /// Activities.
    pub catalog: &'a Catalog,
    /// The world's property regime (ADR-0007).
    pub regime: &'a RegimeDef,
    /// Permanent ids.
    pub ids: &'a mut IdAllocator,
    /// Events to schedule.
    pub schedule: &'a mut Vec<(SimTime, AgentEvent)>,
    /// The approximations made now (none in Detailed mode).
    pub approx: Approximations,
}

impl Ctx<'_> {
    fn schedule_step(&mut self, at: SimTime, person: PermanentId, version: u32) {
        let at = at.max(self.now.plus_minutes(1));
        self.schedule
            .push((at, AgentEvent::Step { person, version }));
    }
}

/// Where gathering trips go for resources of one range and medium: aligned blocks of patches of
/// side `2·range + 1`, the area one trip works. Blocks do not overlap, so land a trip has worked
/// is never mistaken for untouched land by the next. Derived.
#[derive(Debug)]
struct Places {
    range: u32,
    in_water: bool,
    /// Per block, row by row: its centre patch, and the cell people walk to (dry walkable land
    /// nearest the centre; at the water's edge for resources that live in water), if any.
    blocks: Vec<Option<(u32, u32)>>,
}

/// Travel times from a settlement's hearth (or a lone household's home), and the water and places
/// within reach. Shared by every household of the settlement: homes stand a few metres from the
/// hearth. Derived.
#[derive(Debug)]
struct HomeField {
    cell: usize,
    day: i64,
    /// The paths survey and crossings it was walked out on ([`civ_land::Land::route_revision`]).
    rev: u64,
    water: Option<(u32, f32)>,
    /// Further edges of the water in reach, each at least [`BANK_APART_M`] from `water` and from
    /// each other, nearest first, at most [`BANKS_KNOWN`]: where people go along the water when a
    /// nearer edge is passed over (M6a slice BA, step four), as walking seconds.
    banks: Vec<(u32, f32)>,
    /// Springs in reach that would be nearer than `water` if they flowed (M6a slice AY):
    /// walking seconds, patch and the cell it rises at, nearest first. Whether one flows, and
    /// has water left today, is asked when water is fetched.
    springs: Vec<(f32, u32, u32)>,
    /// Per kind of place (the index in `Population::places`), blocks by walking time to their
    /// cell, seconds.
    places: Vec<Vec<(u32, f32)>>,
    /// Walking times to every cell in reach, for fields.
    reach: TravelField,
    /// The cells in reach where new ground could be broken, in cell order ([`farm::find_site`]).
    breakable: Vec<(u32, f32)>,
    /// The ground a search for a field among them looks at ([`farm::site_ground`]).
    site_ground: Option<civ_land::RectCm>,
}

/// What a household's members weigh at a decision whatever the person: the purchase it would
/// make, the household it would ask, the paid work it would take, and the best place for each
/// gathering, digging and field activity, each worked out when first needed. In Accelerated mode
/// (ADR-0011 §4) it is kept from the household's first decision after midnight until midnight or
/// its next consequential step (see [`Population::changed`]); otherwise it serves one decision.
/// Derived, never saved.
#[derive(Debug, Default)]
struct HouseholdView {
    trade: Option<Option<decide::TradeOption>>,
    fetch: Option<Option<decide::TradeOption>>,
    giver: Option<Option<GiverOption>>,
    job: Option<Option<decide::JobOption>>,
    take: Option<Option<crime::TakeTarget>>,
    /// Per activity: the best place to gather or dig, or none.
    patches: Vec<Option<Option<PatchOption>>>,
    /// Per activity: the best field to work, or why there is none.
    fields: Vec<Option<Result<FieldOption, Reason>>>,
}

/// A [`HouseholdView`] during one decision, filled as its options are first needed.
struct ViewCells {
    trade: OnceCell<Option<decide::TradeOption>>,
    fetch: OnceCell<Option<decide::TradeOption>>,
    giver: OnceCell<Option<GiverOption>>,
    job: OnceCell<Option<decide::JobOption>>,
    take: OnceCell<Option<crime::TakeTarget>>,
    patches: Vec<OnceCell<Option<PatchOption>>>,
    fields: Vec<OnceCell<Result<FieldOption, Reason>>>,
}

impl HouseholdView {
    /// The view to fill during a decision among `activities` activities.
    fn cells(self, activities: usize) -> ViewCells {
        fn cell<T>(v: Option<T>) -> OnceCell<T> {
            v.map_or_else(OnceCell::new, OnceCell::from)
        }
        let (patches, fields) = if self.patches.len() == activities {
            (
                self.patches.into_iter().map(cell).collect(),
                self.fields.into_iter().map(cell).collect(),
            )
        } else {
            (
                (0..activities).map(|_| OnceCell::new()).collect(),
                (0..activities).map(|_| OnceCell::new()).collect(),
            )
        };
        ViewCells {
            trade: cell(self.trade),
            fetch: cell(self.fetch),
            giver: cell(self.giver),
            job: cell(self.job),
            take: cell(self.take),
            patches,
            fields,
        }
    }
}

impl ViewCells {
    /// The view as filled, to keep.
    fn kept(self) -> HouseholdView {
        HouseholdView {
            trade: self.trade.into_inner(),
            fetch: self.fetch.into_inner(),
            giver: self.giver.into_inner(),
            job: self.job.into_inner(),
            take: self.take.into_inner(),
            patches: self.patches.into_iter().map(OnceCell::into_inner).collect(),
            fields: self.fields.into_iter().map(OnceCell::into_inner).collect(),
        }
    }
}

/// Minutes people spent, by what they did: the length of each step as it finished (counters, not
/// saved; for the consistency test, ADR-0011 §5). Every minute of a life falls in some step, so
/// they add up to the person-minutes lived.
#[derive(Clone, Debug)]
pub struct TimeUse {
    /// Minutes of work, rest, sleep and the rest, by behaviour in [`Behavior::ALL`] order.
    pub work: [f64; Behavior::ALL.len()],
    /// Minutes walking.
    pub walking: f64,
    /// Minutes waiting, where a step could not begin.
    pub waiting: f64,
}

impl Default for TimeUse {
    fn default() -> Self {
        TimeUse {
            work: [0.0; Behavior::ALL.len()],
            walking: 0.0,
            waiting: 0.0,
        }
    }
}

/// Loads of water drawn, by the kind of place they were drawn at, and the litres they came to
/// (M6a slice AY; counters, not saved: for the dashboard's water row).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterDraws {
    /// Loads drawn from wells, and of those, loads the well held less than a full load of.
    pub wells: u64,
    pub short: u64,
    /// Loads drawn at springs.
    pub springs: u64,
    /// Loads drawn at a river's or lake's edge.
    pub banks: u64,
    /// Litres drawn at all of them.
    pub litres: f64,
}

/// What a gathering activity is expected to bring from each patch on one day, at equilibrium
/// (see [`Land::typical_yields`]). Derived.
#[derive(Debug, Default)]
struct Prior {
    day: i64,
    per_patch: Vec<f32>,
    /// The most of `per_patch`: no patch is expected to bring more.
    most: f32,
}

/// Every person and household of a world.
#[derive(Debug, Default)]
pub struct Population {
    /// Living people.
    pub people: GenTable<Person>,
    /// Households.
    pub households: GenTable<Household>,
    /// Every person who ever lived here, by permanent id.
    pub records: BTreeMap<PermanentId, PersonRecord>,
    /// The chronicle, oldest first.
    pub chronicle: Vec<ChronicleEvent>,
    /// Every couple there has been, oldest first (research 04-08 §5.1).
    pub unions: Vec<Union>,
    /// The last trip id handed out.
    pub next_trip: u64,
    index: FastMap<PermanentId, Handle<Person>>,
    hh_index: FastMap<PermanentId, Handle<Household>>,
    /// Each settlement's households, in the table's order (M8a slice CG, ADR-0028 §2): a search
    /// walks its own settlement's, in the order a scan of every household would find them.
    /// Derived, kept in step where households are added and taken away.
    settled: BTreeMap<PermanentId, BTreeSet<Handle<Household>>>,
    /// Whether households may have been changed by hand since `settled` was last built
    /// ([`Population::forget_derived`]): it is built again before the world next advances.
    settled_stale: bool,
    homes: FastMap<PermanentId, HomeField>,
    /// The kinds of place gathering trips go to.
    places: Vec<Places>,
    /// Routes by (from cell, to cell); `None` when there is none. Derived: kept until the paths
    /// are next surveyed, when walking costs change.
    routes: FastMap<(u32, u32), Option<CachedRoute>>,
    /// The generation of routes before `routes`; one walked again moves back into `routes`.
    routes_old: FastMap<(u32, u32), Option<CachedRoute>>,
    /// The paths survey and crossings the routes were planned on
    /// ([`civ_land::Land::route_revision`]).
    routes_rev: u64,
    /// Times the route cache was cleared for a new revision of the paths, and times it was full
    /// and began a new generation (M8a slice CJ's measurement). Not saved.
    pub route_counts: (u64, u64),
    /// Decisions in Accelerated mode that found their household's view kept and that found
    /// none, and the times every view was let go after goods moved between households (M8a slice
    /// CL's measurement). Not saved.
    pub view_counts: (u64, u64, u64),
    /// Lower bounds for route searches on the paths as surveyed ([`NavGrid::landmarks`]), by
    /// region of walked ground, each built at the first search after a survey to a cell in it.
    /// Derived.
    landmarks: Option<RegionLandmarks>,
    /// What walking reads in each wear tile at `routes_rev` ([`NavGrid::tile_digests`]): where
    /// a new revision leaves a tile's digest as it was, fields and landmarks that read only such
    /// tiles are kept (M10a slice DO). Derived.
    route_tiles: Vec<u64>,
    /// Per activity, what a trip is expected to bring from each patch today.
    priors: Vec<Prior>,
    /// Per household, the new ground it would mark out for a field, as found on a day against the
    /// land as it then stood ([`site_inputs`]), with what that was read against ([`SiteStamp`]).
    sites: FastMap<PermanentId, (i64, u64, Option<Site>, SiteStamp)>,
    /// Counts every change to where households' homes stand: one founded or gone, or a home
    /// moved ([`SiteStamp`]). Derived.
    homes_moved: u64,
    /// Per household with nothing under way, the building it would begin and where (a home, or a
    /// store or a workshop beside it), as found on a day.
    home_sites: FastMap<PermanentId, (i64, Option<NewHome>)>,
    /// Per building, what each of its stages needs (its design never changes). Derived.
    stage_needs: FastMap<PermanentId, Vec<StageNeeds>>,
    /// Per building, its expansion (its design never changes). Derived.
    expansions: FastMap<PermanentId, Arc<Expansion>>,
    /// Households whose last member died today, with that member (within a day's step only).
    emptied: Vec<(PermanentId, PermanentId)>,
    /// Per household, its view of its options while Accelerated mode keeps it (ADR-0011 §4).
    views: FastMap<PermanentId, HouseholdView>,
    /// People in a leisure block that ends by their choosing something else, with the version of
    /// that activity and its activity (ADR-0011 §4). Kept only until the next midnight.
    switched: FastMap<PermanentId, (u32, u16)>,
    /// The settlements household-resolved (M9a slice DF, ADR-0032 §1): a run setting, never
    /// saved; their people are parked ([`Self::reconcile_resolutions`]).
    resolved: BTreeSet<PermanentId>,
    /// Of those, the ones held Detailed for now (ADR-0032 §1): a fire burns there, or a band is
    /// called from it, out from it, or come to it. Derived from saved state at each reconcile.
    held: BTreeSet<PermanentId>,
    /// Whether a decision is a household-resolved day's planning, and what it found
    /// (ADR-0032 §2): set and taken within [`Self::live_household_days`] only.
    planning: bool,
    planned: Option<resolution::Planned>,
    /// When the making a household-resolved day enacts at midnight would have ended in its day
    /// (M10b slice DU): its minutes after dark are counted from there, not from midnight.
    enact_end: Option<SimTime>,
    /// Sessions at a works' machine that wants a crew, lived in a household-resolved day (M10a
    /// slice DT; ADR-0032 §2, crews met in aggregate): held while the settlement's day is lived,
    /// then the machine runs for their summed minutes over its crew if as many hands as it
    /// wants worked it ([`Self::run_held_crews`]). Filled and emptied within
    /// [`Self::live_household_days`] only, never saved.
    crew_held: Vec<firm::HeldSession>,
    /// Hours worked beside a household's knowers in a household-resolved day (M10c slice EB,
    /// first part; ADR-0035 §5): by those learning a technique, and by those knowing it, at each
    /// household, credited when the day is lived ([`Self::credit_alongside`]). Filled and emptied
    /// within [`Self::live_household_days`] only, never saved.
    alongside: knowledge::Alongside,
    /// Whether a household-resolved day is being lived, and whether a held crew's run is under
    /// way (its crew already counted).
    resolving: bool,
    crew_run: bool,
    /// Per parked person, the blocks of the household-resolved day last lived and that day's
    /// index (ADR-0032 §6): what the observer is shown, never read by the simulation, never saved.
    day_blocks: FastMap<PermanentId, Vec<resolution::DayBlock>>,
    day_blocks_day: i64,
    /// Per settlement, who has set to keeping company at its hearth since it was last looked at,
    /// and who was there then, in id order: everyone there now, and some who have left
    /// ([`Self::hearth_company`]). Derived: built at the first look after a load.
    at_hearth: Option<FastMap<PermanentId, AtHearth>>,
    /// Per household and resource, the day its known places of that resource were last cleared
    /// of what has faded ([`Self::remember_patch`]): cleared again that day, they would lose
    /// nothing. Derived; forgotten when the household's places come from elsewhere.
    known_pruned: FastMap<(PermanentId, u16), i64>,
    /// The fields each household works, by index into the land's fields, in order, for the
    /// first `fields_indexed` of them ([`Self::fields_of`]). Brought up to date at midnight;
    /// emptied when a field changes hands. Derived.
    field_index: FastMap<PermanentId, Vec<u32>>,
    fields_indexed: usize,
    /// Everyone's children on record, in id order, as of the first `children_indexed` records
    /// ([`Self::refresh_kin`]). Derived.
    children: FastMap<PermanentId, Vec<PermanentId>>,
    children_indexed: usize,
    /// What became of the goods of households that are no more (counters, not saved).
    pub flows_gone: Flows,
    /// What moved between households, by channel (counters, not saved).
    pub transfers: crate::ledger::Transfers,
    /// How people spent their time (counters, not saved).
    pub time_use: TimeUse,
    /// Every infection there has been (M6a slice AZ; ADR-0021 §5). Saved.
    pub sickness: crate::sickness::Sickness,
    /// What people shed, where it lies and where it went (M6a slice AZ, step two; ADR-0021
    /// §4). Saved.
    pub contagion: crate::contagion::Contagion,
    /// Every fire there has been, the conflagrations they made, and the exposure of buildings
    /// near those burning now (M6b slice BC; ADR-0022 §4, §13). Saved.
    pub fires: crate::fire::Fires,
    /// Every high-water, flood and storm incident and episode (M6b slice BF). Saved.
    pub floods: crate::floods::Floods,
    /// Every time the ground shook and what each building did (M8d slice DA). Saved.
    pub quakes: crate::quake::Quakes,
    /// River cells by the reach they belong to, for loads washed or drawn there. Derived from the
    /// map when first needed; not saved.
    reach_cells: Option<BTreeMap<u32, u32>>,
    /// Water drawn, by the kind of place (counters, not saved; M6a slice AY).
    pub water_draws: WaterDraws,
    /// Each settlement's market (slice I).
    pub markets: Vec<crate::market::Market>,
    /// Every firm there has been, open and closed, in the order they were founded (slice J).
    pub firms: Vec<crate::firm::Firm>,
    /// Each settlement's wealth measures at the end of each year, oldest first (ADR-0007 §4).
    pub wealth_years: Vec<crate::wealth::WealthYear>,
    /// Each settlement's record of the techniques it came to know and lost, oldest first
    /// (ADR-0008 §2).
    pub knowledge: Vec<crate::knowledge::KnowledgeEvent>,
    /// Learners enrolled to learn a technique from someone of another household, in the order
    /// enrolled, and today's minutes of teaching and of study (M10c slice EB, second part;
    /// ADR-0035 §5).
    pub enrolments: Vec<crate::knowledge::Enrolment>,
    pub taught: Vec<crate::knowledge::Session>,
    pub studied: Vec<crate::knowledge::Session>,
    /// Households that could not hold the fee to send a child to learn, within a year, oldest
    /// first (M10c slice EB, third part).
    pub untaught: Vec<crate::knowledge::Untaught>,
    /// Written records of techniques, in the order written, and the minutes each writer has
    /// written toward the record they are writing (M10c slice EC; ADR-0035 §6).
    pub writings: Vec<crate::knowledge::Record>,
    /// Who met whom in company at the hearth today, and for how many hours, kept only while a
    /// disease passing by company runs (M10c slice ED): the day's company exposure, drawn at
    /// midnight.
    pub met: Vec<(PermanentId, PermanentId, f32)>,
    /// Who tended at a sickhouse today, and which (M10c slice EE, fourth part): the day's
    /// exposure of its carers, drawn at midnight.
    pub staffed: Vec<(PermanentId, PermanentId)>,
    /// What each money is worth to each household from its use as money, by household and money
    /// (M10d slice EF, first part; ADR-0035 §12): built at midnight.
    pub money: Vec<crate::market::MoneyBelief>,
    pub written: Vec<crate::knowledge::Session>,
    /// What each settlement has seen of each technique's buildings, in the order first seen
    /// (ADR-0009 §6).
    pub trust: Vec<crate::caution::Trust>,
    /// What each settlement has seen of buildings lost to fire by covering and wall kind
    /// (ADR-0022 §7).
    pub fire_trust: Vec<crate::caution::FireTrust>,
    /// The deposits each settlement knows, in the order they were found (ADR-0010 §1).
    pub deposits_known: Vec<DepositKnown>,
    /// What each person remembers of others (ADR-0014).
    pub ties: crate::ties::Ties,
    /// Each settlement's standing as worked out on the first of the month (ADR-0014 §3).
    pub standing: crate::standing::StandingTable,
    /// Each settlement's polity, in the order they were founded (ADR-0013 §1).
    pub polities: Vec<crate::polity::Polity>,
    /// The notables' tier switched off (ADR-0014 §4): every adult weighs moves at the weekly
    /// review, not only the notables and those an issue reaches. A run setting for the gate that
    /// checks the tier, never saved; off unless set.
    pub every_adult_deliberates: bool,
    /// Takings, what people believe of them, and what households owe for them (ADR-0015).
    pub order: crate::crime::Order,
    /// What people have heard and the grievances they hold (M4c slice AE, ADR-0016).
    pub word: crate::word::Word,
    /// Where people stand on the questions content names (M4c slice AG, ADR-0016 §4).
    pub opinion: crate::opinion::Opinion,
    /// What each holds of the norms content names, and what each household last did (M4c slice
    /// AG, ADR-0016 §4).
    pub norms: crate::norm::Norms,
    /// What each holds dear (M4c slice AG, ADR-0016 §4).
    pub values: crate::values::Values,
    /// Who holds which ideology, and the creeds laws were proposed under (M4c slice AG,
    /// ADR-0016 §4).
    pub ideologies: crate::ideology::Ideologies,
    /// Factions and who belongs to each (M4c slice AH, ADR-0017 §2).
    pub factions: crate::faction::Factions,
    /// The observer's interventions (M4c slice AJ, ADR-0016 §5).
    pub influences: crate::influence::Influences,
    /// The other settlements each household knows (M5a slice AM, ADR-0018 §4).
    pub known_places: crate::places::Places,
    /// Visits and marriages between settlements, by year (M5a slice AM).
    pub contacts: crate::places::Contacts,
    /// The unpartnered who looked for a partner at home and found nobody, and the day they last
    /// did (M5a slice AM; research 04-08 §1.1).
    pub unmatched: BTreeMap<PermanentId, i64>,
    /// Each household's leaning toward moving: the place that won its last reviews (M5a slice
    /// AN, ADR-0018 §5).
    pub leanings: BTreeMap<PermanentId, crate::places::Leaning>,
    /// Households that review where to live at the next midnight, an event having prompted it.
    pub review_due: BTreeSet<PermanentId>,
    /// Every coalition gathered to found a settlement, in the order gathered (M5a slice AO).
    pub coalitions: Vec<crate::places::Coalition>,
    /// What each household believes other settlements' sellers offer (M5b slice AP, ADR-0019
    /// §1).
    pub reports: crate::reports::PriceReports,
    /// Monthly, per pair of settlements, the asks of goods offered in both and what each
    /// carried home from the other (M5b slice AQ, ADR-0019 §7).
    pub convergence: crate::convergence::Convergence,
    /// The demo's twin harness (ADR-0019 §8): purchases between settlements are stopped. Set in
    /// memory by a host command or a test, never by content or a save.
    pub stop_trade_between: bool,
    /// Hours households gave to polities' common works since the last event was handled
    /// (polity, household, hours): paid for under a law of pay for common work as each event
    /// ends (M7d slice CE3a), so never held across a save.
    pub(crate) work_given: Vec<(PermanentId, PermanentId, f64)>,
    /// The households whose review today found a boat worth making (household, vehicle, bank
    /// cell, hours of walking a year it would save): begun as the day's reviews end (M8b slice
    /// CO, third part), so never held across a save.
    pub(crate) boat_wishes: Vec<(PermanentId, usize, u32, f32)>,
    /// The buildings of other settlements each person has seen, newest first, until their
    /// household's next taste review (M5b slice AR).
    pub seen_away: BTreeMap<PermanentId, Vec<PermanentId>>,
    /// Each settlement's way of building when it was founded (M5b slice AR; the first of the
    /// three clocks, research 11-02 §4): a founding band's drawn way, else the mean taste of the
    /// households that founded it.
    pub founding_ways: BTreeMap<PermanentId, crate::params::Taste>,
    /// The places each household's people work, and the outsiders they saw there (M5c slice AT,
    /// ADR-0020 §5).
    pub uses: crate::uses::Uses,
    /// What has been drawn today at each spring (M6a slice AY, ADR-0021 §1).
    pub spring_draws: crate::uses::SpringDraws,
    /// What people believe of other polities they have heard of (M5c slice AT, ADR-0020 §3).
    pub polity_views: crate::views::Views,
    /// The claims of other polities each household has heard of (M5c slice AU, ADR-0020 §2).
    pub claims_heard: crate::uses::ClaimsHeard,
    /// Agreements between polities, in the order made (M5c slice AU, ADR-0020 §6).
    pub agreements: crate::agreements::Agreements,
    /// The river cells each household's people wade (M5c slice AW, step two).
    pub fords: crate::fords::Fords,
    /// Turn-taking at bottlenecks, their tallies and the waits households hold (M8b slice CN,
    /// ADR-0029 §4).
    pub traffic: crate::traffic::Traffic,
    /// Travel measured (M8b slice CR, first part): trips by purpose and mode, waits, and
    /// residents' days of travel, by settlement and year. Nothing reads it to choose.
    pub travel: crate::travel::Travel,
    /// Goods in transit by boat (M8b slice CO, ADR-0029 §3), in the order loaded.
    pub consignments: Vec<crate::consignments::Consignment>,
    /// What the counters of consignments since unloaded saw: counters, never saved.
    pub consigned_flows: crate::person::Flows,
    /// Goods households keep for others (M10d slice EG, second part; ADR-0027 §4), in the order
    /// left, and what the counters of those since emptied saw (counters, never saved).
    pub kept: Vec<crate::custody::Kept>,
    pub kept_flows: crate::person::Flows,
    /// Each cell a facility lies on, with it and its passage width, in cell order, and the
    /// crossings' and enclosures' revisions it stands for. Derived.
    facilities: Vec<(u32, crate::traffic::Facility, f32)>,
    facilities_rev: Option<(u32, u32)>,
}

/// A building a household would begin: its design (which says where it stands), what each stage
/// needs, and for a workshop the firm it is for.
#[derive(Clone, Debug)]
struct NewHome {
    spec: BuildingSpec,
    stages: Vec<StageNeeds>,
    firm: Option<PermanentId>,
    /// Why a home's plot was chosen among others (M8c slice CS).
    receipt: Option<civ_land::SiteReceipt>,
    /// The parcel of a plan it is to stand on, if a keeper gives one (M8c slice CU), and the fee
    /// paid the store for it (the good, by index, and the amount), if one.
    parcel: Option<civ_land::Allotment>,
    fee: Option<(usize, f64)>,
}

/// Why a household begins no building (M8c slice CT): nothing it would build, or a home it would
/// build and no clear ground for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unbuilt {
    Nothing,
    NoGround,
}

/// What a household builds: the building under way (or the one it would begin), the work left on
/// it, the program it follows and the day it wants its roof by.
#[derive(Clone, Debug)]
struct HomePlan {
    building: Option<PermanentId>,
    spec: BuildingSpec,
    work: HomeWork,
    def: usize,
    deadline: i64,
}

/// Whether what [`site_inputs`] reads may have changed, cheaply: fields, plots and earthworks are
/// only ever added, and none is moved, so their counts say whether any has been; and the count
/// of moves of households' homes; and the ground the household looks over, which its home's view
/// can redraw within a day (floodwater changes what ground it can break, M6b slice BF). While
/// these stand, a site found earlier in the day stands.
type SiteStamp = (usize, usize, usize, u64, Option<civ_land::RectCm>);

/// What finding new ground for a field reads that can change within a day, as one number: the
/// fields, plots and earthworks laid out on `ground` (the ground the search looks at,
/// [`farm::site_ground`]; elsewhere they cannot change what it finds) and the homes and hearth it
/// keeps clear of. A site found earlier in the day is kept only while this is the same, so a world
/// saved and loaded within a day finds the same ground as one lived straight on (ADR-0011 §5; a
/// site kept from before a neighbour's field was marked out made the two differ about one world
/// in a hundred).
fn site_inputs(
    land: &civ_land::Land,
    ground: Option<civ_land::RectCm>,
    homes: &[(f32, f32)],
) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    let pair = |a: i32, b: i32| u64::from(a as u32) | (u64::from(b as u32) << 32);
    let on = |r: &civ_land::RectCm| ground.is_some_and(|g| r.near(&g, 0));
    let fields = land.fields.iter().map(|f| f.rect).filter(on);
    let plots = land.plots.iter().map(|p| p.rect).filter(on);
    let works = land.earthworks.iter().map(|w| w.rect).filter(on);
    for (list, rects) in [
        (1u64, fields.collect::<Vec<_>>()),
        (2, plots.collect()),
        (3, works.collect()),
    ] {
        mix(list);
        mix(rects.len() as u64);
        for r in rects {
            mix(pair(r.x, r.y));
            mix(pair(r.w, r.h));
        }
    }
    mix(homes.len() as u64);
    for &(x, y) in homes {
        mix(u64::from(x.to_bits()) | (u64::from(y.to_bits()) << 32));
    }
    h
}

/// The share of the days of crop `crop`'s window for preparing and sowing the ground can usually
/// be worked, from the landscape's climatology (ADR-0012 §5).
fn workable_share(land: &civ_land::Land, crop: usize) -> f64 {
    land.climatology
        .workable_share
        .get(crop)
        .copied()
        .unwrap_or(1.0)
}

/// Grain a kilogram of midden is known to add to the next harvest of `crop` on ground that has
/// lost heart, kilograms (M3c slice V): its nitrogen, the share of it the year's crop can draw on
/// (research 03-04 §2.3: 5-15 %) and takes up, over what the crop takes up for a kilogram of
/// grain.
fn manure_grain_per_kg(params: &PeopleParams, land: &LandParams, crop: &CropParams) -> f64 {
    if crop.crop_n <= 0.0 {
        return 0.0;
    }
    params.midden.n_per_kg() * land.soil.manure_first_year() * land.soil.uptake_share / crop.crop_n
}

/// Field work a capable adult gives on a day at a peak, against an ordinary day's.
fn peak_ratio(params: &PeopleParams) -> f64 {
    params.farm.peak_work_hours_per_day / params.farm.work_hours_per_day.max(1e-9)
}

/// Why the weather keeps people off the ground today, as a reason.
/// Whether `b` is done in company at the hearth (ADR-0014 §2).
fn is_company(b: Behavior) -> bool {
    matches!(
        b,
        Behavior::Socialize
            | Behavior::Attend
            | Behavior::Petition
            | Behavior::Poll
            | Behavior::Visit
    )
}

fn unworkable_reason(why: civ_land::Unworkable) -> Reason {
    match why {
        civ_land::Unworkable::Wet => Reason::WetGround,
        civ_land::Unworkable::Snow => Reason::SnowCover,
        civ_land::Unworkable::Frozen => Reason::FrozenGround,
    }
}

/// Hours of levelling plot `plot` needs, all told: its platforms' earth at the profile's rate
/// (ADR-0010 §2). 0 for a plot on level ground.
fn levelling_h(land: &civ_land::Land, plot: PermanentId, params: &PeopleParams) -> f64 {
    land.earthworks
        .iter()
        .filter(|w| w.plot == Some(plot))
        .map(|w| f64::from(w.cut_m3) * params.build.levelling.h_per_m3)
        .sum()
}

/// Takes what building work used, `used_kg` of each of `def`'s material slots, from household
/// `x`'s stores.
fn use_materials(x: &mut Household, def: &crate::params::BuildingDef, used_kg: &[f64]) {
    for (slot, kg) in used_kg.iter().enumerate() {
        if let Some(&g) = def.materials.get(slot)
            && let Some(s) = x.stores.get_mut(g)
        {
            let used = s.max(0.0).min(*kg);
            *s = (*s - kg).max(0.0);
            x.flows.add(Flow::Built, g, used);
        }
    }
}

/// Whether household `household` keeps its stores under a roof, and how much room it has for them
/// there (ADR-0009 §5): every building of its with its roof on, home or store, shelters them, and
/// gives the room its floors on the ground or a storey hold; a finished one's lofts and raised
/// floors hold goods too.
/// A building whose program has left the content keeps room for everything, as before storage
/// had a capacity.
pub(crate) fn shelter_of(
    land: &Land,
    catalog: &Catalog,
    params: &PeopleParams,
    household: PermanentId,
) -> (bool, crate::person::Keeping) {
    use civ_grammar::storage;
    let mut keeping = crate::person::Keeping {
        raised_kg: 0.0,
        roofed_kg: 0.0,
        raised_factor: params.household.raised_store_factor,
    };
    let mut sheltered = false;
    for b in land
        .buildings
        .iter()
        .filter(|b| b.household == household && b.roofed())
    {
        sheltered = true;
        let room = room_of(b, catalog);
        keeping.raised_kg += room[storage::RAISED];
        keeping.roofed_kg += room[storage::LOFT] + room[storage::FLOOR];
    }
    (sheltered, keeping)
}

/// The room for goods building `b` gives, kilograms by kind ([`civ_grammar::storage`]): none
/// before its roof is on, or in a ruin; then its floors on the ground or a storey; once finished,
/// its lofts and raised floors too; of that, what its condition leaves dry and standing
/// ([`condition::room_left`]). A building whose program has left the content has room for
/// everything.
pub fn room_of(b: &Building, catalog: &Catalog) -> [f64; 3] {
    use civ_grammar::storage;
    if !b.roofed() {
        return [0.0; 3];
    }
    let def = catalog
        .building_index(&b.spec.program)
        .and_then(|i| catalog.buildings.get(i));
    let e = def.and_then(|d| civ_grammar::expand(&b.spec, &d.rules).ok().map(|e| (d, e)));
    match e {
        Some((d, e)) => {
            let room = if b.finished() {
                e.storage_kg
            } else {
                let mut room = [0.0; 3];
                room[storage::FLOOR] = e.storage_kg[storage::FLOOR];
                room
            };
            // As its condition leaves it (ADR-0009 §4).
            condition::room_left(b, &d.upkeep, &e, room)
        }
        None => {
            let mut room = [0.0; 3];
            room[storage::FLOOR] = f64::INFINITY;
            room
        }
    }
}

/// Distance from `p` to the segment `a`–`b`, metres.
fn segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((p.0 - a.0 - t * dx).powi(2) + (p.1 - a.1 - t * dy).powi(2)).sqrt()
}

/// The terrain cell under a point.
pub fn cell_of(map: &WorldMap, (x, y): (f32, f32)) -> usize {
    let c = map.cell_size_m;
    let cx = ((x / c).floor().max(0.0) as u32).min(map.width - 1);
    let cy = ((y / c).floor().max(0.0) as u32).min(map.height - 1);
    (cy * map.width + cx) as usize
}

/// The centre of a terrain cell, metres.
pub fn cell_centre(map: &WorldMap, cell: usize) -> (f32, f32) {
    let w = map.width as usize;
    let c = map.cell_size_m;
    (((cell % w) as f32 + 0.5) * c, ((cell / w) as f32 + 0.5) * c)
}

fn draw(seed: u64, person: &mut Person) -> f64 {
    let mut rng = Rng64::from_key(&[seed, PURPOSE_DECIDE, person.id.get(), person.draws]);
    person.draws += 1;
    rng.next_f64()
}

/// The energy a person's body can draw on in a shortage, kcal: the floor of their balance.
pub fn reserve_kcal(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let mass = needs::mass_kg(&params.energy, p.sex, p.age_years(now));
    params.energy.reserve_kcal_per_kg * mass
}

/// Firewood a household of `members` burns on day `day`, kilograms.
pub fn fuel_per_day(params: &PeopleParams, members: usize, day: i64) -> f64 {
    let month = SimTime::from_minutes(day * MINUTES_PER_DAY).date().month;
    let m = usize::from(month.clamp(1, 12)) - 1;
    params.household.fuel_kg_per_person_day[m] * members as f64
}

/// Least food, in days of one person's need, that must be about to spoil for a household to keep
/// it by a recipe that preserves (a tuning value).
pub const MIN_PRESERVE_DAYS: f64 = 0.5;

/// A household's stores at `now`: spoiled by each good's half-life, less the firewood burned.
pub fn stores_now(
    h: &Household,
    now: SimTime,
    params: &PeopleParams,
    goods: &[GoodDef],
) -> Vec<f64> {
    let members = h.members.len();
    h.stores_at_time(now, goods, &|d| fuel_per_day(params, members, d))
}

/// Goods not accounted for between two readings of the books, each `(held, flows)`: `(good,
/// amount)` wherever what households hold changed by other than what entered less what left
/// (ADR-0006 §3: goods are conserved; moving between households is neither). Empty when the
/// books balance, to rounding. `goods` is the number of goods.
pub fn unaccounted(
    goods: usize,
    before: (&[f64], &Flows),
    after: (&[f64], &Flows),
) -> Vec<(usize, f64)> {
    let held = |v: &[f64], g: usize| v.get(g).copied().unwrap_or(0.0);
    (0..goods)
        .filter_map(|g| {
            let change = held(after.0, g) - held(before.0, g);
            let gap = change - (after.1.net(g) - before.1.net(g));
            let scale = after.1.turnover(g)
                + before.1.turnover(g)
                + held(after.0, g).abs()
                + held(before.0, g).abs();
            (gap.abs() > 1e-9 * scale + 1e-6).then_some((g, gap))
        })
        .collect()
}

/// Brings a person's needs up to `now` at the rates in force since they were last settled.
pub fn settle(p: &mut Person, now: SimTime, params: &PeopleParams) {
    let dt = (now.minutes() - p.needs_at.minutes()) as f64;
    if dt <= 0.0 {
        return;
    }
    let energy = f64::from(p.energy_kcal) - f64::from(p.burn_kcal_min) * dt;
    let floor = -reserve_kcal(p, now, params);
    p.energy_kcal = energy.clamp(floor, params.energy.max_surplus_kcal) as f32;
    p.sleep_pressure =
        needs::sleep_pressure(&params.sleep, f64::from(p.sleep_pressure), dt, p.asleep) as f32;
    p.relatedness = needs::relatedness(
        f64::from(p.relatedness),
        f64::from(p.company),
        dt,
        params.social.tau_h,
    ) as f32;
    p.needs_at = now;
}

/// Basal metabolism of a person now, kcal per day.
pub fn bmr(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let age = p.age_years(now);
    let mass = needs::mass_kg(&params.energy, p.sex, age);
    needs::bmr_kcal_day(&params.energy, p.sex, age, mass)
}

/// Energy a person spends a minute at physical activity level `par`, kcal: their basal metabolism
/// times the level, and what a pregnancy costs (research 05-02 §2.4).
pub fn burn_rate(p: &Person, now: SimTime, params: &PeopleParams, par: f64) -> f32 {
    ((bmr(p, now, params) * par + extra_kcal_day(p, now, params)) / 1440.0) as f32
}

/// Hunger now: 0 while full, rising to 1 over the hunger ramp, plus deficit units.
pub fn hunger(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let after = (now.minutes() - p.satiety_until.minutes()) as f64 / 60.0;
    let ramp = (after / params.energy.hunger_ramp_hours).clamp(0.0, 1.0);
    ramp + (-energy_now(p, now, params)).max(0.0) / params.energy.deficit_unit_kcal
}

/// Minutes of a leisure block between the times it looks at for the sleep threshold.
const LEISURE_SLEEP_STEP_MIN: f64 = 10.0;

/// Minutes from `now` until the next consequential boundary for someone at leisure (ADR-0011 §4):
/// the next turn of the day (`next_turn_min` on), when hunger begins, or when the sleep drive
/// reaches the threshold at which sleep becomes an option (found to the nearest
/// [`LEISURE_SLEEP_STEP_MIN`], looking no further than `longest_min`). Zero when hunger or sleep is
/// already upon them.
pub fn leisure_until(
    p: &Person,
    now: SimTime,
    params: &PeopleParams,
    sun: (i64, i64),
    next_turn_min: f64,
    longest_min: f64,
) -> f64 {
    if hunger(p, now, params) > 0.0 {
        return 0.0;
    }
    let minute = now.minute_of_day();
    let wake = params.sleep.wake_pressure;
    let p0 = f64::from(p.sleep_pressure);
    if p0 * needs::circadian(&params.sleep, minute, sun) >= wake {
        return 0.0;
    }
    let fed = (p.satiety_until.minutes() - now.minutes()).max(0) as f64;
    let until = next_turn_min.min(fed).max(0.0);
    let mut t = LEISURE_SLEEP_STEP_MIN;
    while t < until.min(longest_min) {
        let at = (minute + t as i64).rem_euclid(MINUTES_PER_DAY);
        let pressure = needs::sleep_pressure(&params.sleep, p0, t, false);
        if pressure * needs::circadian(&params.sleep, at, sun) >= wake {
            return t;
        }
        t += LEISURE_SLEEP_STEP_MIN;
    }
    until
}

/// A person's energy balance at `now`, kcal: negative in deficit, no lower than their body's
/// reserve allows.
pub fn energy_now(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let elapsed = (now.minutes() - p.needs_at.minutes()).max(0) as f64;
    (f64::from(p.energy_kcal) - f64::from(p.burn_kcal_min) * elapsed)
        .max(-reserve_kcal(p, now, params))
}

/// Whether a person is hungry enough to eat food kept back (seed): they have drawn
/// `energy.eat_reserve_at_deficit` of their body's reserve (research 08-02 §2.3: households can
/// eat seed in a crisis, at the cost of the next sowing).
pub fn may_eat_reserve(p: &Person, now: SimTime, params: &PeopleParams) -> bool {
    let reserve = reserve_kcal(p, now, params);
    -energy_now(p, now, params) >= params.energy.eat_reserve_at_deficit * reserve
}

/// Food energy a household holds beyond what it keeps for itself, kcal: what it can spare a
/// household in need (food, ready or not, above twice the days of food it tries to keep).
pub fn spare_food_kcal(
    h: &Household,
    now: SimTime,
    params: &PeopleParams,
    goods: &[GoodDef],
) -> f64 {
    let stores = stores_now(h, now, params, goods);
    let food = stock_kcal(&stores, goods);
    let need = h.members.len() as f64 * params.household.daily_kcal_per_person;
    (food - 2.0 * params.household.food_target_days * need).max(0.0)
}

impl Population {
    /// An empty population.
    pub fn new() -> Population {
        Population::default()
    }

    /// Adds a household.
    pub fn insert_household(&mut self, h: Household) -> Handle<Household> {
        self.homes_moved += 1;
        let (id, settlement) = (h.id, h.settlement);
        let handle = self.households.insert(h);
        self.hh_index.insert(id, handle);
        if let Some(s) = settlement {
            self.settled.entry(s).or_default().insert(handle);
        }
        handle
    }

    /// Builds the settlement index from the households as they are.
    fn build_settled(&mut self) {
        self.settled.clear();
        for (h, x) in self.households.iter() {
            if let Some(s) = x.settlement {
                self.settled.entry(s).or_default().insert(h);
            }
        }
        self.settled_stale = false;
    }

    /// Builds the settlement index again if households may have been changed by hand since it
    /// was built (as tests do, after [`Population::forget_derived`]). The world calls it before
    /// each advance.
    pub fn refresh_settled(&mut self) {
        if self.settled_stale {
            self.build_settled();
        }
    }

    /// Takes household `id` away, with its place in the indexes.
    pub(crate) fn take_household(&mut self, id: PermanentId) -> Option<Household> {
        let hd = self.hh_index.remove(&id)?;
        let gone = self.households.remove(hd)?;
        if let Some(s) = gone.settlement
            && let Some(set) = self.settled.get_mut(&s)
        {
            set.remove(&hd);
            if set.is_empty() {
                self.settled.remove(&s);
            }
        }
        Some(gone)
    }

    /// The households of settlement `s`, in the order a scan of every household finds them
    /// (M8a slice CG, ADR-0028 §2). Checked against that scan whenever debug checks are on.
    pub(crate) fn households_in(&self, s: PermanentId) -> impl Iterator<Item = &Household> + '_ {
        #[cfg(debug_assertions)]
        self.check_settled(&[s]);
        self.settled
            .get(&s)
            .into_iter()
            .flatten()
            .filter_map(|&h| self.households.get(h))
    }

    /// The households of any of `settlements`, in the order a scan of every household finds
    /// them.
    pub(crate) fn households_in_any(&self, settlements: &[PermanentId]) -> Vec<&Household> {
        #[cfg(debug_assertions)]
        self.check_settled(settlements);
        let mut handles: Vec<Handle<Household>> = settlements
            .iter()
            .filter_map(|s| self.settled.get(s))
            .flatten()
            .copied()
            .collect();
        handles.sort_unstable();
        handles.dedup();
        handles
            .into_iter()
            .filter_map(|h| self.households.get(h))
            .collect()
    }

    /// Panics unless the settlement index lists, for each of `settlements`, exactly the
    /// households a scan finds, in its order.
    #[cfg(debug_assertions)]
    fn check_settled(&self, settlements: &[PermanentId]) {
        for &s in settlements {
            let scan: Vec<Handle<Household>> = self
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(s))
                .map(|(h, _)| h)
                .collect();
            let index: Vec<Handle<Household>> = self
                .settled
                .get(&s)
                .map(|set| set.iter().copied().collect())
                .unwrap_or_default();
            assert_eq!(
                index, scan,
                "the households of settlement {s} are out of step"
            );
        }
    }

    /// Adds a living person (their record must be added separately).
    pub fn insert_person(&mut self, p: Person) -> Handle<Person> {
        let id = p.id;
        let handle = self.people.insert(p);
        self.index.insert(id, handle);
        handle
    }

    /// Rebuilds the id indexes and drops derived caches (after loading).
    pub fn rebuild_indexes(&mut self) {
        self.index = self.people.iter().map(|(h, p)| (p.id, h)).collect();
        self.hh_index = self.households.iter().map(|(h, x)| (x.id, h)).collect();
        self.build_settled();
        self.homes.clear();
        self.places.clear();
        self.routes.clear();
        self.routes_old.clear();
        self.priors.clear();
        self.sites.clear();
        self.home_sites.clear();
        self.stage_needs.clear();
        self.expansions.clear();
        self.at_hearth = None;
        self.known_pruned.clear();
        self.landmarks = None;
        self.route_tiles.clear();
        self.fields_moved();
    }

    /// Gives people and households of a world saved before tools and skills (save schema 8 and
    /// earlier, ADR-0006 §6) what a founder brings: every household the tools its adults want,
    /// and every person skills drawn as a founder's are, keyed by the world seed and their id.
    pub fn give_founders_kit(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        seed: u64,
        now: SimTime,
    ) {
        const PURPOSE_KIT: u64 = 0x6b69_7430_3030_3031; // "kit00001"
        let ages: FastMap<PermanentId, f64> = self
            .people
            .iter()
            .map(|(_, p)| (p.id, p.age_years(now)))
            .collect();
        for (_, h) in self.households.iter_mut() {
            let members: Vec<f64> = h
                .members
                .iter()
                .filter_map(|m| ages.get(m).copied())
                .collect();
            h.stores.resize(catalog.goods.len(), 0.0);
            // The tools for the work founders know (ADR-0008 §1), but for those that stay where
            // they are made (an oven).
            let founders_know = |t: usize| {
                params
                    .knowledge
                    .founders
                    .iter()
                    .any(|&(x, s)| x == t && s > 0.0)
            };
            let wants = make::tool_wants_for(
                catalog,
                &members,
                params.family.independent_age,
                &founders_know,
            );
            for (g, want) in wants.into_iter().enumerate() {
                if catalog.goods[g].tool.as_ref().is_some_and(|t| t.fixed) {
                    continue;
                }
                h.stores[g] += want;
                h.flows.add(Flow::Brought, g, want);
            }
        }
        for (_, p) in self.people.iter_mut() {
            let mut rng = Rng64::from_key(&[seed, PURPOSE_KIT, p.id.get()]);
            p.skills = crate::found::founder_skills(
                &catalog.skills,
                p.age_years(now),
                params.family.independent_age,
                &mut rng,
            );
        }
    }

    /// Gives everyone the skills in `new` (indexes into `catalog.skills`; skills a save made
    /// before they were in the content knows nothing of) as a founder of their age would bring
    /// them, drawn from their founder's skill stream ([`crate::found::PURPOSE_SKILLS`]).
    pub fn give_new_skills(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        seed: u64,
        now: SimTime,
        new: &[usize],
    ) {
        if new.is_empty() {
            return;
        }
        for (_, p) in self.people.iter_mut() {
            let mut rng = Rng64::from_key(&[seed, crate::found::PURPOSE_SKILLS, p.id.get()]);
            let drawn = crate::found::founder_skills(
                &catalog.skills,
                p.age_years(now),
                params.family.independent_age,
                &mut rng,
            );
            for (k, level) in drawn {
                if new.contains(&usize::from(k)) {
                    p.set_skill(usize::from(k), f64::from(level));
                }
            }
        }
    }

    /// What became of every household's, firm's and polity's goods since the counters began,
    /// those that are no more included.
    pub fn flows(&self) -> Flows {
        let mut all = self.flows_gone.clone();
        for (_, h) in self.households.iter() {
            all.absorb(&h.flows);
        }
        for f in &self.firms {
            all.absorb(&f.flows);
        }
        for p in &self.polities {
            all.absorb(&p.flows);
        }
        // And what payments agreements owe hold, set aside and on the road (M5c slice AV).
        for d in &self.agreements.dues {
            all.absorb(&d.flows);
        }
        // And loads in transit by boat (M8b slice CO).
        for c in &self.consignments {
            all.absorb(&c.flows);
        }
        all.absorb(&self.consigned_flows);
        // And goods kept for others (M10d slice EG, second part).
        for k in &self.kept {
            all.absorb(&k.flows);
        }
        all.absorb(&self.kept_flows);
        all
    }

    /// What all households, firms and polities hold, by good, as each one's stores were last
    /// brought up to date: the balance that [`Population::flows`] accounts for (ADR-0006 §3).
    pub fn goods_held(&self) -> Vec<f64> {
        let mut out: Vec<f64> = Vec::new();
        let stores = self
            .households
            .iter()
            .map(|(_, h)| &h.stores)
            .chain(self.firms.iter().map(|f| &f.stores))
            .chain(self.polities.iter().map(|p| &p.stores))
            .chain(self.agreements.dues.iter().map(|d| &d.stores))
            .chain(self.consignments.iter().map(|c| &c.stores))
            .chain(self.kept.iter().map(|k| &k.stores));
        for s in stores {
            if out.len() < s.len() {
                out.resize(s.len(), 0.0);
            }
            for (o, kg) in out.iter_mut().zip(s) {
                *o += kg;
            }
        }
        out
    }

    /// Household `household`'s buildings changed other than by its own work (a test's): its
    /// stores are settled to `now` under the roofs it had, and its shelter derived again.
    pub fn buildings_changed(
        &mut self,
        now: SimTime,
        land: &Land,
        catalog: &Catalog,
        params: &PeopleParams,
        household: PermanentId,
    ) {
        if let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            let members = x.members.len().max(1);
            x.settle_stores(now, &catalog.goods, &|d| fuel_per_day(params, members, d));
            (x.sheltered, x.keeping) = shelter_of(land, catalog, params, household);
        }
    }

    /// Brings every household's shelter up to date with the land's buildings (after loading,
    /// where it is not saved; [`shelter_of`]).
    pub fn derive_shelter(&mut self, land: &Land, catalog: &Catalog, params: &PeopleParams) {
        for (_, h) in self.households.iter_mut() {
            (h.sheltered, h.keeping) = shelter_of(land, catalog, params, h.id);
        }
    }

    /// The new building household `household` plans to begin, as it planned it today, if any: a
    /// home, or a store beside it.
    pub fn planned_home(&self, household: PermanentId) -> Option<&BuildingSpec> {
        self.home_sites
            .get(&household)
            .and_then(|(_, site)| site.as_ref())
            .map(|s| &s.spec)
    }

    /// A living person by permanent id.
    pub fn person(&self, id: PermanentId) -> Option<&Person> {
        self.index.get(&id).and_then(|&h| self.people.get(h))
    }

    /// A household by permanent id.
    pub fn household(&self, id: PermanentId) -> Option<&Household> {
        self.hh_index.get(&id).and_then(|&h| self.households.get(h))
    }

    /// The name of anyone who ever lived here.
    pub fn name_of(&self, id: PermanentId) -> String {
        self.records
            .get(&id)
            .map_or_else(|| format!("person {id}"), |r| r.given.clone())
    }

    /// Number of living people.
    pub fn living(&self) -> usize {
        self.people.len()
    }

    /// What is wrong with the population, if anything: references that do not resolve, steps
    /// out of range, malformed trips. A save holding any of these is refused (ADR-0002).
    /// `next_id` is the permanent-id counter; `catalog_len` the number of activities.
    pub fn problems(&self, next_id: u64, catalog_len: usize) -> Vec<String> {
        let mut out = self
            .known_places
            .problems(|h| self.household(h).map(|x| x.settlement), |_| true);
        out.extend(
            self.reports
                .problems(|h| self.household(h).map(|x| x.settlement)),
        );
        out.extend(self.convergence.problems());
        out.extend(self.fires.problems());
        out.extend(self.uses.problems());
        out.extend(self.polity_views.problems());
        out.extend(self.claims_heard.problems());
        out.extend(self.fords.problems());
        out.extend(self.traffic.problems());
        out.extend(self.travel.problems());
        out.extend(self.agreements.problems());
        for (&p, seen) in &self.seen_away {
            if self.person(p).is_none() {
                out.push(format!(
                    "person {p} keeps buildings seen elsewhere but is no more"
                ));
            }
            let mut ids = seen.clone();
            ids.sort_unstable();
            ids.dedup();
            if seen.is_empty() || ids.len() != seen.len() {
                out.push(format!(
                    "person {p} keeps an empty or repeated list of buildings seen"
                ));
            }
        }
        for &(year, from, to) in self.contacts.years.keys() {
            if from == to || year < 0 {
                out.push(format!(
                    "contacts of {from} with itself or before the first year"
                ));
            }
        }
        for id in self.unmatched.keys() {
            if !self.records.contains_key(id) {
                out.push(format!(
                    "person {id} looked for a partner and is on no record"
                ));
            }
        }
        let mut ids = civ_core::FastSet::default();
        for (_, p) in self.people.iter() {
            if !ids.insert(p.id) || p.id.get() >= next_id {
                out.push(format!("person {} has a duplicate or unallocated id", p.id));
            }
            match self.household(p.household) {
                Some(h) if h.members.contains(&p.id) => {}
                _ => out.push(format!(
                    "person {} belongs to household {}, which does not list them",
                    p.id, p.household
                )),
            }
            match self.records.get(&p.id) {
                Some(r) if r.died.is_none() && r.left.is_none() => {}
                _ => out.push(format!("person {} has no living record", p.id)),
            }
            if usize::from(p.act.def) >= catalog_len.max(1) {
                out.push(format!("person {} does an unknown activity", p.id));
            }
            // A parked activity (ADR-0032 §3) has no steps and stands at its own mark.
            let parked = p.act.steps.is_empty() && p.act.step == resolution::PARKED_STEP;
            if usize::from(p.act.step) > p.act.steps.len() && !parked {
                out.push(format!(
                    "person {} is past the last step of their activity",
                    p.id
                ));
            }
            if let Some(t) = &p.trip
                && (t.points.len() < 2
                    || t.points.len() != t.minutes.len()
                    || t.minutes.windows(2).any(|w| w[1].is_nan() || w[1] < w[0])
                    || t.id > self.next_trip)
            {
                out.push(format!("person {} has a malformed trip", p.id));
            }
            let finite = [
                p.pos.0,
                p.pos.1,
                p.energy_kcal,
                p.sleep_pressure,
                p.relatedness,
                p.burn_kcal_min,
                p.company,
            ];
            if finite.iter().any(|v| !v.is_finite()) {
                out.push(format!("person {} has a value that is not a number", p.id));
            }
        }
        let mut hh_ids = civ_core::FastSet::default();
        for (_, h) in self.households.iter() {
            if !hh_ids.insert(h.id) || h.id.get() >= next_id {
                out.push(format!(
                    "household {} has a duplicate or unallocated id",
                    h.id
                ));
            }
            for m in &h.members {
                if self.person(*m).is_none_or(|p| p.household != h.id) {
                    out.push(format!(
                        "household {} lists {m}, who does not live there",
                        h.id
                    ));
                }
            }
            if !(h.stores.iter().all(|v| v.is_finite()) && h.water_l.is_finite()) {
                out.push(format!(
                    "household {} has a store that is not a number",
                    h.id
                ));
            }
        }
        let mut firm_ids = civ_core::FastSet::default();
        for f in &self.firms {
            if !firm_ids.insert(f.id) || hh_ids.contains(&f.id) || f.id.get() >= next_id {
                out.push(format!("firm {} has a duplicate or unallocated id", f.id));
            }
            // A firm closed when its household was no more may name it; an open one may not.
            if f.is_open() && self.household(f.owner).is_none() {
                out.push(format!(
                    "firm {} is open but its owner {} is not a household",
                    f.id, f.owner
                ));
            }
            if !f.stores.iter().all(|v| v.is_finite())
                || (!f.is_open() && f.stores.iter().any(|&v| v != 0.0))
            {
                out.push(format!(
                    "firm {} has a store that is not a number or outlived it",
                    f.id
                ));
            }
        }
        // Factions (M4c slice AH): ids of their own, members alive and in live ones.
        let mut faction_ids = civ_core::FastSet::default();
        for f in &self.factions.list {
            if !faction_ids.insert(f.id)
                || hh_ids.contains(&f.id)
                || firm_ids.contains(&f.id)
                || f.id.get() >= next_id
            {
                out.push(format!(
                    "faction {} has a duplicate or unallocated id",
                    f.id
                ));
            }
            if !f.stores.iter().all(|v| v.is_finite() && *v >= 0.0) {
                out.push(format!("faction {} has a store that is not a number", f.id));
            }
        }
        for m in &self.factions.members {
            if self.person(m.person).is_none() {
                out.push(format!(
                    "{} belongs to a faction but is not alive",
                    m.person
                ));
            }
            if !self
                .factions
                .get(m.faction)
                .is_some_and(crate::faction::Faction::is_live)
            {
                out.push(format!(
                    "{} belongs to faction {}, which is not live",
                    m.person, m.faction
                ));
            }
        }
        for (id, r) in &self.records {
            if *id != r.id || id.get() >= next_id {
                out.push(format!("record {id} is filed under the wrong id"));
            }
            if r.died.is_none() && r.left.is_none() && self.person(*id).is_none() {
                out.push(format!("record {id} is alive but the person is missing"));
            }
        }
        for (i, e) in self.chronicle.iter().enumerate() {
            if e.seq != i as u64 + 1 {
                out.push(format!("chronicle entry {} is out of sequence", e.seq));
                break;
            }
        }
        for (_, p) in self.people.iter() {
            if let Some(q) = p.partner
                && self
                    .person(q)
                    .is_none_or(|x| x.partner != Some(p.id) || x.sex == p.sex)
            {
                out.push(format!(
                    "person {}'s partner {q} is not partnered with them",
                    p.id
                ));
            }
            if p.sex == crate::needs::Sex::Male && p.repro != crate::person::Repro::Open {
                out.push(format!("person {} is a man with a pregnancy", p.id));
            }
            if let crate::person::Repro::Pregnant { conceived, due, .. } = p.repro
                && due <= conceived
            {
                out.push(format!("person {}'s pregnancy ends before it began", p.id));
            }
            if !(p.fecundity.is_finite() && p.fecundity >= 0.0) {
                out.push(format!(
                    "person {} has a fecundity that is not a number",
                    p.id
                ));
            }
            if p.nursing.is_some_and(|c| !self.records.contains_key(&c)) {
                out.push(format!("person {} nurses a child nobody knows", p.id));
            }
        }
        for u in &self.unions {
            if !(self.records.contains_key(&u.woman) && self.records.contains_key(&u.man)) {
                out.push(format!(
                    "a union of {} and {} names nobody known",
                    u.woman, u.man
                ));
            } else if u.ended.is_none()
                && (self.person(u.woman).and_then(|p| p.partner) != Some(u.man)
                    || self.person(u.man).and_then(|p| p.partner) != Some(u.woman))
            {
                out.push(format!(
                    "the union of {} and {} has not ended but they are not partners",
                    u.woman, u.man
                ));
            }
        }
        out.extend(self.residence_problems());
        out
    }

    /// Adds a chronicle entry about firm `firm`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn chronicle_push_firm(
        &mut self,
        at: SimTime,
        kind: ChronicleKind,
        people: Vec<PermanentId>,
        settlement: Option<PermanentId>,
        place: Option<(f32, f32)>,
        number: f64,
        name: String,
        firm: PermanentId,
    ) {
        self.chronicle_push(at, kind, people, settlement, place, number, name);
        if let Some(e) = self.chronicle.last_mut() {
            e.firm = Some(firm);
        }
    }

    /// Adds a chronicle entry.
    #[allow(clippy::too_many_arguments)]
    pub fn chronicle_push(
        &mut self,
        at: SimTime,
        kind: ChronicleKind,
        people: Vec<PermanentId>,
        settlement: Option<PermanentId>,
        place: Option<(f32, f32)>,
        number: f64,
        name: String,
    ) {
        let seq = self.chronicle.len() as u64 + 1;
        self.chronicle.push(ChronicleEvent {
            seq,
            at,
            kind,
            people,
            settlement,
            place,
            number,
            name,
            firm: None,
        });
    }

    /// Handles a scheduled event. Events of dead people or replaced activities do nothing.
    pub fn on_event(&mut self, ctx: &mut Ctx, event: AgentEvent) {
        match event {
            AgentEvent::Step { person, version } => {
                let Some(&h) = self.index.get(&person) else {
                    return;
                };
                if self.people.get(h).map(|p| p.act.version) != Some(version) {
                    return;
                }
                self.end_step(ctx, h);
                if let Some(p) = self.people.get_mut(h) {
                    p.act.step += 1;
                }
                self.run_steps(ctx, h, 0);
            }
            AgentEvent::Fire => self.fire_minute(ctx),
            AgentEvent::Ignite { building } => self.ignite(ctx, building),
            AgentEvent::BandLook => self.band_look(ctx),
            AgentEvent::Gates => self.keep_gates(ctx),
            AgentEvent::Trains => self.trains_due(ctx),
            AgentEvent::Gas => self.gas_evening(ctx),
            AgentEvent::Pass {
                person,
                version,
                trip,
            } => {
                let Some(&h) = self.index.get(&person) else {
                    return;
                };
                let live = self.people.get(h).is_some_and(|p| {
                    p.act.version == version && p.trip.as_ref().map(|t| t.id) == Some(trip)
                });
                if live {
                    self.reach_bottleneck(ctx, h);
                }
            }
        }
        self.pay_for_work_given(ctx);
    }

    /// Sends `id` walking to `to` at once, as a step of their activity would (tests): `false`
    /// when they cannot set off.
    #[doc(hidden)]
    pub fn walk_for_tests(&mut self, ctx: &mut Ctx, id: PermanentId, to: (f32, f32)) -> bool {
        let Some(&h) = self.index.get(&id) else {
            return false;
        };
        self.start_walk(ctx, h, to)
    }

    /// Sends `id` pulling their household's cart that stands where they are to `to` at once, as a
    /// step of their activity would (tests): `false` when they cannot set off, or turn back.
    #[doc(hidden)]
    pub fn haul_for_tests(&mut self, ctx: &mut Ctx, id: PermanentId, to: (f32, f32)) -> bool {
        let Some(&h) = self.index.get(&id) else {
            return false;
        };
        if let Some(p) = self.people.get_mut(h) {
            p.act.steps = vec![Step::Haul { to }];
            p.act.step = 0;
        }
        self.start_haul(ctx, h, to)
    }

    /// Sends `id` riding a line's train from the station where they stand to its end at cell `to`
    /// at once, as a step of their activity would (tests): `false` when no train would take them.
    #[doc(hidden)]
    pub fn ride_for_tests(&mut self, ctx: &mut Ctx, id: PermanentId, to: u32) -> bool {
        let Some(&h) = self.index.get(&id) else {
            return false;
        };
        if let Some(p) = self.people.get_mut(h) {
            p.act.steps = vec![Step::Ride { to }];
            p.act.step = 0;
            p.act.version += 1;
        }
        self.start_ride(ctx, h, to)
    }

    /// A ride for `id` from the station where they stand to the line's end at cell `to`, as a
    /// household-resolved day lives it (tests): `false` when no train runs that day or their
    /// household cannot pay.
    #[doc(hidden)]
    pub fn ride_in_block_for_tests(&mut self, ctx: &mut Ctx, id: PermanentId, to: u32) -> bool {
        let Some(&h) = self.index.get(&id) else {
            return false;
        };
        self.ride_in_block(ctx, h, to)
    }

    /// The ride household `household` could take out to a pit at `pit`, its walk to the station
    /// reckoned in a straight line (tests): the line's end it rides to, the minutes one way, and
    /// the fare.
    #[doc(hidden)]
    pub fn ride_legs_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        pit: (f32, f32),
    ) -> Option<(u32, f64, f64)> {
        let x = self.household(household)?;
        let hearth = Self::hearth_of(ctx, x);
        let pace = ctx.params.nav.tobler_ms(0.0) * ctx.params.nav.offtrail_factor;
        let home = x.home;
        let walk = |at: (f32, f32)| {
            Some(f64::from((at.0 - home.0).hypot(at.1 - home.1)) / pace.max(1e-6) / 60.0)
        };
        let (legs, minutes, _, fare) = Self::ride_legs(ctx, household, home, hearth, pit, &walk)?;
        Some((legs.to, minutes, fare))
    }

    /// Drops a person's activity so they decide again when an event of the returned version
    /// fires (used when a loaded world's content no longer has their activity). The caller
    /// schedules that event.
    pub fn restart(&mut self, id: PermanentId) -> Option<u32> {
        let &h = self.index.get(&id)?;
        let p = self.people.get_mut(h)?;
        let now = p.needs_at;
        p.act = Activity {
            def: 0,
            target: Target::None,
            steps: Vec::new(),
            step: 0,
            started: now,
            step_started: now,
            step_ends: now,
            version: p.act.version.wrapping_add(1),
        };
        p.trip = None;
        Some(p.act.version)
    }

    /// Household `household` has taken a consequential step (a deposit, work on a field or a
    /// building, making): it sees its options afresh at its next decision (ADR-0011 §4).
    fn changed(&mut self, household: PermanentId) {
        self.views.remove(&household);
    }

    /// Goods moved between household `household` and the party its step was with (`with`): the
    /// two of them, and every household whose kept view names either (as the seller it would buy
    /// from, the household it would ask or take from, the workshop it would work for), see their
    /// options afresh; the rest keep theirs (M8a slice CL, ADR-0028 §6).
    fn views_moved(&mut self, household: PermanentId, with: Target) {
        let (other, firm) = match with {
            Target::Household(x) => (Some(x), None),
            Target::Firm(f) => (self.firm(f).map(|x| x.owner), Some(f)),
            _ => (None, None),
        };
        let party = |id: PermanentId| id == household || Some(id) == other || Some(id) == firm;
        self.views.retain(|&hh, view| {
            fn kept<T>(cell: &Option<Option<T>>) -> Option<&T> {
                cell.as_ref().and_then(Option::as_ref)
            }
            !(party(hh)
                || kept(&view.trade).is_some_and(|o| party(o.seller))
                || kept(&view.fetch).is_some_and(|o| party(o.seller))
                || kept(&view.giver).is_some_and(|g| party(g.household))
                || kept(&view.job).is_some_and(|j| party(j.firm))
                || kept(&view.take).is_some_and(|t| party(t.household())))
        });
    }

    /// Every household sees its options afresh at its next decision: after the observer has
    /// changed the world, or the approximations made have changed.
    pub fn forget_views(&mut self) {
        self.views.clear();
        self.switched.clear();
    }

    /// Forgets what is kept to find things faster (field sites, who is at each hearth, the
    /// fields by household, faded places let go): after people, households or land were changed
    /// by hand, as tests do. Each is worked out again when next needed, the same as it would
    /// have been, so nothing that happens changes.
    pub fn forget_derived(&mut self) {
        self.sites.clear();
        self.homes_moved += 1;
        self.at_hearth = None;
        self.known_pruned.clear();
        self.fields_moved();
        self.children_indexed = usize::MAX;
        self.settled_stale = true;
    }

    /// Starts a person's life in the simulation: they decide what to do first.
    pub fn begin(&mut self, ctx: &mut Ctx, id: PermanentId) {
        if let Some(&h) = self.index.get(&id) {
            self.decide(ctx, h, 0);
        }
    }

    /// Lays out the places gathering trips go, once per world: one kind of place per range and
    /// medium among the land's resources.
    fn ensure_places(&mut self, ctx: &Ctx) {
        let mut kinds: Vec<(u32, bool)> = ctx
            .land_params
            .resources
            .iter()
            .map(|r| (r.range_patches, r.in_water))
            .collect();
        kinds.sort_unstable();
        kinds.dedup();
        if self.places.len() == kinds.len()
            && self
                .places
                .iter()
                .zip(&kinds)
                .all(|(p, k)| (p.range, p.in_water) == *k)
        {
            return;
        }
        self.places = kinds
            .into_iter()
            .map(|(range, in_water)| lay_out_places(ctx, range, in_water))
            .collect();
        self.homes.clear();
    }

    /// The kind of place resource `r` is gathered in.
    fn places_for(&self, params: &LandParams, r: usize) -> Option<usize> {
        let res = params.resources.get(r)?;
        self.places
            .iter()
            .position(|p| p.range == res.range_patches && p.in_water == res.in_water)
    }

    /// Brings the expected yields of every gathering activity up to `day`.
    fn ensure_priors(&mut self, ctx: &Ctx, day: i64) {
        let defs = &ctx.catalog.activities;
        if self.priors.len() != defs.len() {
            self.priors = defs.iter().map(|_| Prior::default()).collect();
        }
        for (a, prior) in defs.iter().zip(&mut self.priors) {
            let Some(r) = a.resource.filter(|&r| r < ctx.land_params.resources.len()) else {
                continue;
            };
            if prior.day == day && prior.per_patch.len() == ctx.land.patches.len() {
                continue;
            }
            let hours = f64::from(a.max_minutes) / 60.0;
            prior.per_patch = ctx.land.typical_yields(ctx.land_params, r, hours, day);
            prior.most = prior.per_patch.iter().copied().fold(0.0, f32::max);
            prior.day = day;
        }
    }

    fn field_reach_seconds(catalog: &Catalog) -> f32 {
        let max_min = catalog
            .activities
            .iter()
            .map(|a| a.max_walk_minutes)
            .max()
            .unwrap_or(60);
        // Off-trail walking is slower than the walk the decision allows for; keep some margin.
        (f64::from(max_min) * 60.0 * 1.2) as f32
    }

    /// Brings route caches to the paths survey and walking ground as they stand
    /// ([`civ_land::Land::route_revision`]). On a new revision the routes found are let go, as
    /// before; walking fields and landmarks are kept where every wear tile they read has the
    /// digest it had (M10a slice DO, ADR-0034 §1): walking costs there are as they were, so
    /// they are what working them out again would give. A kept field's day's facts are drawn
    /// again at its next use, as for a new one.
    fn sync_routes(&mut self, ctx: &Ctx) {
        let revision = ctx.land.route_revision(ctx.nav.travel_revision());
        if revision == self.routes_rev && !self.route_tiles.is_empty() {
            return;
        }
        if revision != self.routes_rev {
            self.route_counts.0 += 1;
        }
        self.routes.clear();
        self.routes_old.clear();
        self.routes_rev = revision;
        let wear = &ctx.land.wear;
        let tile = civ_land::paths::TILE as usize;
        let digests = ctx
            .nav
            .tile_digests(tile, &ctx.map.elevation, &|c| wear.factor(c));
        let tiles_x = (ctx.map.width as usize).div_ceil(tile);
        let changed: Vec<(usize, usize)> = if digests.len() == self.route_tiles.len() {
            digests
                .iter()
                .zip(&self.route_tiles)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(k, _)| (k % tiles_x, k / tiles_x))
                .collect()
        } else {
            (0..digests.len())
                .map(|k| (k % tiles_x, k / tiles_x))
                .collect()
        };
        let untouched = |((x0, y0), (x1, y1)): CellBox| {
            !changed.iter().any(|&(tx, ty)| {
                tx * tile <= x1 && x0 < (tx + 1) * tile && ty * tile <= y1 && y0 < (ty + 1) * tile
            })
        };
        self.homes
            .retain(|_, f| f.reach.reach_box().is_some_and(untouched));
        for f in self.homes.values_mut() {
            f.rev = revision;
            f.day = i64::MIN;
        }
        match self.landmarks.as_mut() {
            Some(by_region) if by_region.wear_rev == wear.rev() => {
                for built in &mut by_region.built {
                    if built.as_ref().is_some_and(|(_, reach)| !untouched(*reach)) {
                        *built = None;
                    }
                }
            }
            _ => self.landmarks = None,
        }
        self.route_tiles = digests;
    }

    fn home_field(&mut self, ctx: &Ctx, key: PermanentId, origin: (f32, f32)) {
        self.sync_routes(ctx);
        let cell = cell_of(ctx.map, origin);
        let day = ctx.now.day_index();
        let rev = ctx.land.route_revision(ctx.nav.travel_revision());
        if let Some(f) = self.homes.get(&key)
            && f.cell == cell
            && f.rev == rev
            && f.day == day
        {
            return;
        }
        self.ensure_places(ctx);
        let places_in = |field: &TravelField| -> Vec<Vec<(u32, f32)>> {
            self.places
                .iter()
                .map(|kind| {
                    let mut out: Vec<(u32, f32)> = kind
                        .blocks
                        .iter()
                        .enumerate()
                        .filter_map(|(b, place)| {
                            let (_, cell) = (*place)?;
                            field.seconds_to(cell as usize).map(|s| (b as u32, s))
                        })
                        .collect();
                    out.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
                    out
                })
                .collect()
        };
        // Reuse walking times while refreshing the day's land and source facts.
        if let Some(f) = self.homes.get_mut(&key)
            && f.cell == cell
            && f.rev == rev
        {
            f.breakable = farm::breakable_in_reach(
                ctx.map,
                ctx.nav,
                ctx.land,
                ctx.land_params,
                &f.reach,
                ctx.params,
            );
            f.springs =
                springs_in_reach(ctx.land, &f.reach, f.water.map_or(f32::INFINITY, |w| w.1));
            f.places = places_in(&f.reach);
            f.site_ground = farm::site_ground(ctx.map, &f.breakable, ctx.params);
            f.day = day;
            return;
        }
        let reach = Self::field_reach_seconds(ctx.catalog);
        let wear = &ctx.land.wear;
        let field = ctx
            .nav
            .travel_field(&ctx.map.elevation, cell, reach, &|c| wear.factor(c));
        let mut edges = water_edges(ctx.map, &field).into_iter();
        let water = edges.next();
        let banks: Vec<(u32, f32)> = edges.collect();
        let springs = springs_in_reach(ctx.land, &field, water.map_or(f32::INFINITY, |w| w.1));
        let places = places_in(&field);
        let breakable = farm::breakable_in_reach(
            ctx.map,
            ctx.nav,
            ctx.land,
            ctx.land_params,
            &field,
            ctx.params,
        );
        self.homes.insert(
            key,
            HomeField {
                cell,
                day,
                rev,
                water,
                banks,
                springs,
                places,
                reach: field,
                site_ground: farm::site_ground(ctx.map, &breakable, ctx.params),
                breakable,
            },
        );
    }

    /// The new ground household `hh` would mark out for a field today (found once a day).
    fn site_for(
        &mut self,
        ctx: &Ctx,
        hh: &Household,
        field_key: PermanentId,
        crop: &CropParams,
    ) -> Option<Site> {
        let day = ctx.now.day_index();
        let home = self.homes.get(&field_key)?;
        let (breakable, ground) = (&home.breakable, home.site_ground);
        let land: &civ_land::Land = ctx.land;
        let stamp = (
            land.fields.len(),
            land.plots.len(),
            land.earthworks.len(),
            self.homes_moved,
            ground,
        );
        let kept = self
            .sites
            .get(&hh.id)
            .filter(|&&(seen, ..)| seen == day)
            .copied();
        if let Some((_, read, site, at)) = kept
            && at == stamp
        {
            debug_assert_eq!(
                read,
                site_inputs(land, ground, &self.site_homes(land, hh)),
                "what a field site is found against changed without its stamp"
            );
            return site;
        }
        let homes = self.site_homes(land, hh);
        let inputs = site_inputs(ctx.land, ground, &homes);
        if let Some((_, at, site, _)) = kept
            && at == inputs
        {
            self.sites.insert(hh.id, (day, inputs, site, stamp));
            return site;
        }
        let site = farm::find_site(
            ctx.map,
            ctx.nav,
            ctx.land,
            ctx.land_params,
            breakable,
            &homes,
            ctx.params,
            crop,
            &[ctx.seed, farm::PURPOSE_SITE, hh.id.get(), day as u64],
            hh.id,
        );
        self.sites.insert(hh.id, (day, inputs, site, stamp));
        site
    }

    /// The fields household `household` works, in the land's order: those indexed, then any
    /// marked out since, found by looking at those alone (M5a slice AL).
    pub(crate) fn fields_of<'s, 'l>(
        &'s self,
        land: &'l civ_land::Land,
        household: PermanentId,
    ) -> impl Iterator<Item = &'l Field> + Clone + use<'s, 'l> {
        let indexed = self.fields_indexed.min(land.fields.len());
        let head = if indexed == 0 {
            &[][..]
        } else {
            self.field_index
                .get(&household)
                .map_or(&[][..], Vec::as_slice)
        };
        debug_assert!(
            head.iter()
                .all(|&i| land.fields[i as usize].household == household)
                && head.len()
                    == land.fields[..indexed]
                        .iter()
                        .filter(|f| f.household == household)
                        .count(),
            "a field changed hands without the index knowing"
        );
        head.iter().map(|&i| &land.fields[i as usize]).chain(
            land.fields[indexed..]
                .iter()
                .filter(move |f| f.household == household),
        )
    }

    /// Brings [`Self::fields_of`]'s index up to the fields marked out so far.
    fn index_fields(&mut self, land: &civ_land::Land) {
        for (i, f) in land.fields.iter().enumerate().skip(self.fields_indexed) {
            self.field_index
                .entry(f.household)
                .or_default()
                .push(i as u32);
        }
        self.fields_indexed = land.fields.len();
    }

    /// A field has changed hands: [`Self::fields_of`] looks at every field until midnight.
    pub(crate) fn fields_moved(&mut self) {
        if self.fields_indexed > 0 {
            self.field_index.clear();
            self.fields_indexed = 0;
        }
    }

    /// Where a new field for household `hh` keeps clear of: every home of its settlement (its
    /// own if it has none) and the settlement's hearth.
    fn site_homes(&self, land: &civ_land::Land, hh: &Household) -> Vec<(f32, f32)> {
        // Its settlement's households from the index, which lists them in the order a scan finds
        // them (M9b slice DJ); a household the index does not hold among them is looked for by
        // the scan, as before.
        let indexed = hh
            .settlement
            .filter(|&s| self.households_in(s).any(|x| x.id == hh.id));
        let mut homes: Vec<(f32, f32)> = match indexed {
            Some(s) => self.households_in(s).map(|x| x.home).collect(),
            None => self
                .households
                .iter()
                .filter(|(_, x)| {
                    x.id == hh.id || (hh.settlement.is_some() && x.settlement == hh.settlement)
                })
                .map(|(_, x)| x.home)
                .collect(),
        };
        homes.extend(
            land.settlements
                .iter()
                .filter(|s| Some(s.id) == hh.settlement)
                .map(|s| s.hearth_m),
        );
        homes
    }

    /// What each stage of `building` needs (expanded once per building), its first stage also
    /// `levelling_h` of levelling its plot (ADR-0010 §2).
    fn needs_of(
        &mut self,
        building: &Building,
        def: &crate::params::BuildingDef,
        levelling_h: f64,
    ) -> Option<Vec<StageNeeds>> {
        if let Some(n) = self.stage_needs.get(&building.id) {
            return Some(n.clone());
        }
        let mut needs = build::stage_needs(&building.spec, def)?;
        if let Some(first) = needs.first_mut() {
            first.labour_h += levelling_h;
        }
        self.stage_needs.insert(building.id, needs.clone());
        Some(needs)
    }

    /// The expansion of `building` under `def`'s rules (expanded once per building).
    fn expansion_of(
        &mut self,
        building: &Building,
        def: &crate::params::BuildingDef,
    ) -> Option<Arc<Expansion>> {
        if let Some(e) = self.expansions.get(&building.id) {
            return Some(Arc::clone(e));
        }
        let e = Arc::new(civ_grammar::expand(&building.spec, &def.rules).ok()?);
        self.expansions.insert(building.id, Arc::clone(&e));
        Some(e)
    }

    /// The upkeep household `hh` would do (ADR-0009 §4): the repair under way on one of its
    /// buildings, or else renewing all that is lost of the group gone furthest beyond showing its
    /// condition, of all its standing buildings someone in it can work on
    /// ([`condition::repair_of`]). Wanted, like any building, by the roof deadline. `None` when
    /// nothing of its shows.
    fn upkeep_plan(&mut self, ctx: &Ctx, hh: &Household) -> Option<HomePlan> {
        let catalog = ctx.catalog;
        // How far gone, the building's index and its program's.
        let mut best: Option<(f64, usize, usize)> = None;
        for (i, b) in ctx.land.buildings.iter().enumerate() {
            if b.household != hh.id
                || !b.finished()
                || !b.standing()
                || !self.can_build(catalog, &hh.members, b)
            {
                continue;
            }
            let Some(d) = catalog.building_index(&b.spec.program) else {
                continue;
            };
            // A repair begun is finished first.
            let over = match b.repair {
                Some(_) => f64::INFINITY,
                None => match condition::worst(b, &catalog.buildings[d].upkeep) {
                    Some((_, over)) => over,
                    None => continue,
                },
            };
            if best.is_none_or(|(o, _, _)| over > o) {
                best = Some((over, i, d));
            }
        }
        let (_, i, d) = best?;
        let (b, def) = (&ctx.land.buildings[i], &catalog.buildings[d]);
        let r = condition::repair_of(b, &def.upkeep)?;
        let e = self.expansion_of(b, def)?;
        let needs = condition::mend_needs(&e, r.group, f64::from(r.share))?;
        Some(HomePlan {
            building: Some(b.id),
            spec: b.spec.clone(),
            work: HomeWork::mend(needs, f64::from(r.work_h)),
            def: d,
            deadline: build::roof_deadline(ctx.now.day_index(), def.roof_by_day),
        })
    }

    /// What household `hh` builds: one building at a time (ADR-0009 §7). Its building under way
    /// that someone in it can work on, whatever it is for, or else the one it would begin and
    /// where (found once a day: for a home, its home if the ground there is clear, else the
    /// nearest clear ground within reach of the hearth; for a store or a workshop, clear ground
    /// near its home); between the two, the upkeep of its buildings ([`Population::upkeep_plan`],
    /// ADR-0009 §4). It builds to one of the programs it may build, those someone in it knows
    /// how to (ADR-0009 §1). A first home is the cheapest that covers its members and its goods,
    /// as large as what it puts in of its means makes it ([`build::first_home`]). A household
    /// that has a home builds a storehouse beside it when the goods its roofs have no room for
    /// would lose more in the open than one costs ([`Population::storehouse_plan`]); else a
    /// workshop for a firm that has had more at work at once than a home has room for
    /// ([`Population::workshop_plans`]); else a new home only when its means pay for all of one
    /// markedly larger ([`build::new_home`]), beside the old: the first of these it can pay for
    /// and find ground for. Its means are [`Population::home_means`], for a household giving
    /// `labour_per_day`. `Built` while it has a finished home and builds no other; `NoPlace` when
    /// there is no program it can build to or no clear ground.
    fn home_plan(
        &mut self,
        ctx: &Ctx,
        hh: &Household,
        hearth: Option<(f32, f32)>,
        field_key: PermanentId,
        labour_per_day: f64,
    ) -> Result<HomePlan, Reason> {
        let catalog = ctx.catalog;
        let dwelling = |b: &Building| catalog.is_dwelling(&b.spec.program);
        // The floor of the finished home it lives in, if it has one (a ruin is none).
        let mut home: Option<f64> = None;
        for b in ctx
            .land
            .buildings
            .iter()
            .filter(|b| b.household == hh.id && b.finished() && b.standing() && dwelling(b))
        {
            let floor = build::floor_m2(&b.spec);
            home = Some(home.map_or(floor, |f| f.max(floor)));
        }
        // Its building under way that it can work on, a home first; one nobody in it can work on
        // waits, and holds nothing else up.
        let under_way = ctx
            .land
            .buildings
            .iter()
            .filter(|b| {
                b.household == hh.id && !b.finished() && self.can_build(catalog, &hh.members, b)
            })
            .min_by_key(|b| !dwelling(b));
        if let Some(b) = under_way {
            let def = catalog
                .building_index(&b.spec.program)
                .ok_or(Reason::NoPlace)?;
            let levelling = levelling_h(ctx.land, b.plot, ctx.params);
            let needs = self
                .needs_of(b, &catalog.buildings[def], levelling)
                .ok_or(Reason::NoPlace)?;
            return Ok(HomePlan {
                building: Some(b.id),
                spec: b.spec.clone(),
                work: HomeWork::of(b, needs),
                def,
                deadline: build::roof_deadline(
                    b.started.day_index(),
                    catalog.buildings[def].roof_by_day,
                ),
            });
        }
        // Then upkeep, before anything new.
        if let Some(plan) = self.upkeep_plan(ctx, hh) {
            return Ok(plan);
        }
        let unbuilt = if home.is_some() {
            Reason::Built
        } else {
            Reason::NoPlace
        };
        let day = ctx.now.day_index();
        let found = match self.home_sites.get(&hh.id) {
            Some((seen, site)) if *seen == day => site.clone(),
            _ => {
                let site = self.new_building(ctx, hh, hearth, field_key, labour_per_day, home);
                // A home it meant to build and found no clear ground for (M8c slice CT).
                if site.as_ref().err() == Some(&Unbuilt::NoGround) {
                    self.see_ground(
                        ctx,
                        hh.settlement,
                        hh.id,
                        crate::polity::IssueKind::NoGround,
                    );
                }
                let site = site.ok();
                self.home_sites.insert(hh.id, (day, site.clone()));
                site
            }
        };
        let Some(site) = found else {
            return Err(unbuilt);
        };
        let def = catalog
            .building_index(&site.spec.program)
            .ok_or(Reason::NoPlace)?;
        Ok(HomePlan {
            building: None,
            spec: site.spec,
            work: HomeWork::begin(site.stages),
            def,
            deadline: build::roof_deadline(day, catalog.buildings[def].roof_by_day),
        })
    }

    /// The building household `hh` would begin today and where, if any: its first home; or, when
    /// it has a home of `home` square metres, a storehouse beside it if one is worth it, a
    /// workshop for a busy firm, or a new home if its means pay for one markedly larger
    /// ([`Population::home_plan`]): the first of these that it can pay for and find ground for.
    fn new_building(
        &self,
        ctx: &Ctx,
        hh: &Household,
        hearth: Option<(f32, f32)>,
        field_key: PermanentId,
        labour_per_day: f64,
        home: Option<f64>,
    ) -> Result<NewHome, Unbuilt> {
        let catalog = ctx.catalog;
        let day = ctx.now.day_index();
        let programs = self.home_programs(ctx, &hh.members);
        let carry = ctx.params.household.carry_kg;
        let home_choice = |current: Option<f64>| {
            let first = programs.first().and_then(|&p| catalog.buildings.get(p))?;
            let (budget_h, time_h) = self.home_means(ctx, hh, first, labour_per_day, day);
            let need = build::HomeNeed {
                residents: hh.members.len().max(1),
                storage_kg: self.storage_need(ctx, hh),
            };
            let chosen = match current {
                None => {
                    build::first_home(&catalog.buildings, &programs, need, carry, budget_h, time_h)
                }
                Some(current) => build::new_home(
                    &catalog.buildings,
                    &programs,
                    need,
                    current,
                    carry,
                    budget_h,
                    time_h,
                ),
            }?;
            // And the shapes on less ground its means would also pay for, to be weighed with the
            // ground each takes (M8d slice CZ): a first home's extra over the one chosen within
            // what it would put into a home, a new home's whole within its means.
            let program = &catalog.buildings[chosen.0];
            let chosen_h = program
                .shapes
                .iter()
                .find(|c| c.shape == chosen.1)
                .map_or(0.0, |c| c.hours(carry));
            let affordable = |h: f64| {
                h <= time_h
                    && match current {
                        None => h <= chosen_h + budget_h,
                        Some(_) => h <= budget_h,
                    }
            };
            let compact = build::compact_alternatives(program, chosen.1, &need, carry, &affordable);
            Some((chosen, compact))
        };
        let goods = &catalog.goods;
        let reach = self.homes.get(&field_key).map(|f| &f.reach);
        // The rules for building its builders keep (M6b slice BH), and the plots of the buildings
        // that stand, which a new one keeps its distance from.
        let kept_rules = self.rules_kept(ctx, hh);
        let rule_rects: Vec<civ_land::RectCm> = if kept_rules.any() {
            ctx.land
                .buildings
                .iter()
                .filter(|b| b.standing())
                .filter_map(|b| ctx.land.plots.iter().find(|p| p.id == b.plot))
                .map(|p| p.rect)
                .collect()
        } else {
            Vec::new()
        };
        // Other households' homes, which a home where fire is worked keeps its distance from
        // (M8c slice CW).
        let others_homes: Vec<civ_land::RectCm> = if kept_rules.fire_from.is_some() {
            ctx.land
                .buildings
                .iter()
                .filter(|b| b.household != hh.id && b.standing())
                .filter(|b| catalog.is_dwelling(&b.spec.program))
                .filter_map(|b| ctx.land.plots.iter().find(|p| p.id == b.plot))
                .map(|p| p.rect)
                .collect()
        } else {
            Vec::new()
        };
        // The plans in force its builders keep (M8c slice CT, third part), whose streets and
        // public sites a new building keeps clear of.
        let kept_plans = self.plans_kept(ctx, hh);
        // And the parcels whose district allows only some uses (M8c slice CW).
        let zoned = self.zoned_kept(ctx, hh);
        // Where `choice` would stand, its door toward `toward`, if there is clear ground for it
        // within `max_m` of `from`: keeping the rules and plans its builders keep, if there is
        // ground for that, else as they would without them.
        let site_kept = |(p, shape): (usize, build::Shape),
                         toward: Option<(f32, f32)>,
                         firm: Option<PermanentId>,
                         from: (f32, f32),
                         max_m: f64,
                         obey_rules: bool| {
            let program = &catalog.buildings[p];
            // Members as strong as what the settlement has seen of the techniques it would build
            // with calls for: its program's, and the walls and systems its people know (ADR-0031
            // §5: a fallen rubble wall thickens the next rubble walls).
            let caution = self.building_caution(ctx, hh, program);
            // Built to the household's taste, held to what the program allows, and now and then
            // with something new to it (M3b slice R).
            let style = crate::style::commission(
                &ctx.params.style,
                &hh.taste,
                program,
                &[
                    ctx.seed,
                    crate::style::PURPOSE_STYLE,
                    hh.id.get(),
                    day as u64,
                    p as u64,
                ],
            );
            let knows_technique = |name: &str| {
                catalog.technique_index(name).is_some_and(|t| {
                    hh.members
                        .iter()
                        .filter_map(|&m| self.person(m))
                        .any(|p| p.knows(t))
                })
            };
            // The rules in force that its builders keep (M6b slice BH): a roof that does not burn
            // where they can make one, else a plot apart from others; walls of a kind a rule
            // allows where they can build them; a plot the least gap from every other.
            let rules = if obey_rules {
                kept_rules
            } else {
                Default::default()
            };
            let plans: &[&civ_land::plans::Layout] = if obey_rules { &kept_plans } else { &[] };
            let caution_thatch =
                self.fire_caution_thatch(hh.settlement, ctx.now, &ctx.params.build.caution);
            let caution_wattle =
                self.fire_caution_wattle(hh.settlement, ctx.now, &ctx.params.build.caution);
            let fire_design = build::FireDesign {
                caution_thatch: if rules.roof_within.is_some() {
                    caution_thatch.max(2.0)
                } else {
                    caution_thatch
                },
                caution_wattle: if rules.walls.is_some() {
                    caution_wattle.max(2.0)
                } else {
                    caution_wattle
                },
                knows_rubble: knows_technique("core:technique/rubble_walling"),
                knows_mud_brick: knows_technique("core:technique/mud_brick"),
                knows_firing: knows_technique("core:technique/firing"),
                knows_lime: knows_technique("core:technique/lime_burning"),
                // The version 3 systems its program opens to a technique someone of it knows.
                knows_systems: program.systems.map(|t| {
                    t.is_some_and(|t| {
                        hh.members
                            .iter()
                            .filter_map(|&m| self.person(m))
                            .any(|p| p.knows(t))
                    })
                }),
            };
            let design_at = |at: (f32, f32)| {
                let reachable = reach.is_none_or(|r| r.seconds_to(cell_of(ctx.map, at)).is_some());
                let design = |shape: build::Shape| {
                    build::design_cautious(
                        program,
                        goods,
                        shape,
                        at,
                        toward,
                        caution,
                        Some(style),
                        Some(fire_design),
                    )
                };
                reachable
                    .then(|| {
                        let spec = design(shape)?;
                        // Built lower where a height or ties rule its builders keep allows no
                        // more storeys on its walls (M8d slice DB).
                        let most = rules.most_storeys(
                            crate::rules::wall_kind(&spec, &program.rules),
                            crate::rules::tied(&spec),
                        );
                        match shape {
                            build::Shape::Bays {
                                bays,
                                storeys,
                                lofts,
                            } if storeys > most && most >= 1 => design(build::Shape::Bays {
                                bays,
                                storeys: most,
                                lofts,
                            }),
                            _ => Some(spec),
                        }
                    })
                    .flatten()
                    .filter(|spec| {
                        !rules.too_tall(spec, crate::rules::wall_kind(spec, &program.rules))
                    })
                    // A roof that burns is not put within a roof rule's reach of another building,
                    // walls are of a kind the rules allow, and a plot keeps the least gap.
                    .filter(|spec| {
                        let nearest = || {
                            let rect = build::plot_rect(spec, program);
                            rule_rects
                                .iter()
                                .map(|r| crate::rules::gap_m(&rect, r))
                                .fold(f64::INFINITY, f64::min)
                        };
                        rules.roof_within.is_none_or(|within| {
                            crate::rules::roof_unburnt(spec) || nearest() >= within
                        }) && rules.walls.is_none_or(|kinds| {
                            kinds & (1 << crate::rules::wall_kind(spec, &program.rules)) != 0
                        }) && rules.gap.is_none_or(|gap| nearest() >= gap)
                    })
                    // A home keeps its midden from open wells, and one where fire is worked keeps
                    // from others' homes (M8c slice CW).
                    .filter(|spec| {
                        program.use_ != civ_land::PlotUse::Dwelling || {
                            let rect = build::plot_rect(spec, program);
                            rules.midden_from.is_none_or(|m| {
                                !crate::rules::near_open_well(&rect, m, &ctx.land.wells)
                            }) && rules.fire_from.is_none_or(|m| {
                                others_homes
                                    .iter()
                                    .all(|r| crate::rules::gap_m(&rect, r) >= m)
                            })
                        }
                    })
                    // Nor on ground a plan its builders keep reserves.
                    .filter(|spec| {
                        plans.is_empty() || {
                            let rect = build::plot_rect(spec, program);
                            plans.iter().all(|l| !l.reserves(&rect, 0))
                        }
                    })
                    // Nor on a parcel whose district allows another use (M8c slice CW).
                    .filter(|spec| {
                        !obey_rules || zoned.is_empty() || {
                            let rect = build::plot_rect(spec, program);
                            let bit = crate::polity::use_bit(program.use_);
                            zoned
                                .iter()
                                .all(|(r, uses)| uses & bit != 0 || !r.near(&rect, 0))
                        }
                    })
            };
            // Ground that drops too far across a plot is levelled, and ground too steep to level
            // by hand, or that would be levelled beside water, is not built on; level ground is
            // sought first (ADR-0010 §2-3).
            let lev = &ctx.params.build.levelling;
            let bed = |x: f64, y: f64| civ_land::earth::bed_height(ctx.map, (x, y));
            let levelling = |rect: &civ_land::RectCm| {
                let drop = civ_land::earth::drop_across(rect, &bed);
                if drop <= lev.from_m {
                    Some(0.0)
                } else {
                    (drop <= lev.most_m
                        && civ_land::earth::clear_of_water(ctx.map, rect, drop, lev.side_run))
                    .then_some(drop)
                }
            };
            let spec = build::home_site(
                ctx.land, ctx.map, ctx.nav, program, &design_at, from, hearth, max_m, &levelling,
            )?;
            let stages = build::stage_needs(&spec, program)?;
            // The levelling it needs, hours: a slope's cut, half the plot to a quarter of its
            // drop on the mean, as the platform takes it (ADR-0010 §2).
            let rect = build::plot_rect(&spec, program);
            let area_m2 = f64::from(rect.w) * f64::from(rect.h) / 1e4;
            let level_h = levelling(&rect).unwrap_or(0.0) * area_m2 / 8.0 * lev.h_per_m3;
            Some((
                NewHome {
                    spec,
                    stages,
                    firm,
                    receipt: None,
                    parcel: None,
                    fee: None,
                },
                level_h,
            ))
        };
        let site_levelled = |choice: (usize, build::Shape),
                             toward: Option<(f32, f32)>,
                             firm: Option<PermanentId>,
                             from: (f32, f32),
                             max_m: f64| {
            site_kept(choice, toward, firm, from, max_m, true).or_else(|| {
                (kept_rules.any() || !kept_plans.is_empty())
                    .then(|| site_kept(choice, toward, firm, from, max_m, false))
                    .flatten()
            })
        };
        let site = |choice: (usize, build::Shape),
                    toward: Option<(f32, f32)>,
                    firm: Option<PermanentId>,
                    max_m: f64| {
            site_levelled(choice, toward, firm, hh.home, max_m).map(|s| s.0)
        };
        // A home's plot is a choice (M8c slice CS, ADR-0030 §1): sought about the points its
        // people's walks suggest, each found weighed by its walks over the content's years and
        // its levelling, the cheapest taken with its receipt. About its old point first, farther
        // afield for a first home when nothing is clear near.
        let places = self.goings_of(ctx, hh);
        let speed = ctx.nav.params().top_speed_ms().max(0.1);
        let years = ctx.params.build.site_years;
        let offer = self.parcel_offer(ctx, hh);
        let choose_home = |choice: (usize, build::Shape), first: bool| {
            let anchors = siting::site_anchors(hh.home, &places);
            let mut found: Vec<(f64, NewHome)> = Vec::new();
            for (i, &from) in anchors.iter().enumerate() {
                let Some((home, level_h)) = site_levelled(choice, hearth, None, from, HOME_SHIFT_M)
                    .or_else(|| {
                        (first && i == 0)
                            .then(|| site_levelled(choice, hearth, None, from, FIRST_HOME_SHIFT_M))
                            .flatten()
                    })
                else {
                    continue;
                };
                let at = build::centre_m(&home.spec);
                if found.iter().any(|(_, f)| build::centre_m(&f.spec) == at) {
                    continue;
                }
                // Beyond its old point, no farther from the hearth than the walk it accepts to a
                // field: a home elsewhere is a move between settlements, weighed apart.
                let near_hearth = reach.is_none_or(|r| {
                    r.seconds_to(cell_of(ctx.map, at))
                        .is_some_and(|s| f64::from(s) <= ctx.params.farm.max_walk_minutes * 60.0)
                });
                if i > 0 && !near_hearth {
                    continue;
                }
                let hours = years * siting::walk_hours_year(at, &places, speed) + level_h;
                found.push((hours, home));
            }
            // The parcel a plan's keeper would give it, weighed as any plot is (M8c slice CU):
            // a plot found whose middle lies on it takes it, and its home is also laid out at the
            // parcel's middle or a few metres along it, wholly within it.
            // A fee counts at what the household's own hours would make of its good, and a
            // ground rent the same each year over the years a site is weighed; one it cannot
            // reckon is not paid.
            let costs = self.own_costs_of(ctx, hh);
            let cost_of = |g: usize| costs.get(g).copied().flatten();
            let offer = offer.filter(|o| {
                o.fee.is_none_or(|(g, _)| cost_of(g).is_some())
                    && o.rent.is_none_or(|(g, _)| cost_of(g).is_some())
            });
            if let Some(o) = offer {
                let (law, parcel, ground) = (o.law, o.parcel, o.ground);
                let fee_h = o.fee.map_or(0.0, |(g, a)| a * cost_of(g).unwrap_or(0.0))
                    + years * o.rent.map_or(0.0, |(g, a)| a * cost_of(g).unwrap_or(0.0));
                let program = &catalog.buildings[choice.0];
                let within = |spec: &BuildingSpec| {
                    let rect = build::plot_rect(spec, program);
                    rect.x >= ground.x
                        && rect.y >= ground.y
                        && rect.x + rect.w <= ground.x + ground.w
                        && rect.y + rect.h <= ground.y + ground.h
                };
                let given = civ_land::Allotment {
                    law,
                    parcel,
                    rule: o.rule,
                };
                let on = |spec: &BuildingSpec| {
                    let (x, y) = build::centre_m(spec);
                    let (x, y) = ((f64::from(x) * 100.0) as i32, (f64::from(y) * 100.0) as i32);
                    x >= ground.x
                        && y >= ground.y
                        && x < ground.x + ground.w
                        && y < ground.y + ground.h
                };
                for (h, f) in found.iter_mut().filter(|(_, f)| on(&f.spec)) {
                    f.parcel = Some(given);
                    f.fee = o.fee;
                    *h += fee_h;
                }
                let (cx, cy) = ground.centre_m();
                let along_x = ground.w >= ground.h;
                let laid = [0.0f32, 2.0, -2.0, 4.0, -4.0].into_iter().find_map(|d| {
                    let from = if along_x { (cx + d, cy) } else { (cx, cy + d) };
                    site_levelled(choice, hearth, None, from, 0.0).filter(|(h, _)| within(&h.spec))
                });
                if let Some((mut home, level_h)) = laid {
                    let at = build::centre_m(&home.spec);
                    if !found.iter().any(|(_, f)| build::centre_m(&f.spec) == at) {
                        let hours =
                            years * siting::walk_hours_year(at, &places, speed) + level_h + fee_h;
                        home.parcel = Some(given);
                        home.fee = o.fee;
                        found.push((hours, home));
                    }
                }
            }
            // The cheapest, the first found among equals.
            let best = found
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.0.total_cmp(&b.1.0).then(a.0.cmp(&b.0)))
                .map(|(i, _)| i)?;
            let hours = found[best].0;
            let next = found
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != best)
                .map(|(_, f)| f.0)
                .fold(None, |m: Option<f64>, h| Some(m.map_or(h, |m| m.min(h))))
                .unwrap_or(hours);
            let weighed = found.len() as u16;
            let (_, mut home) = found.swap_remove(best);
            home.receipt = Some(civ_land::SiteReceipt {
                weighed,
                hours: hours as f32,
                next_hours: next as f32,
            });
            Some(home)
        };
        // A first home before anything, farther afield if there is no clear ground near; then a
        // store, while goods are lost for want of room, and a workshop for each firm that has more
        // at work than its home has room for, before more floor. A home's door faces the hearth;
        // a store's or a workshop's, its household's home.
        // A home it means to build and finds no ground for is told apart (M8c slice CT).
        // The shape it would build and those on less ground are each weighed with the plot it
        // would take, in hours: building it and its ground's walks, levelling and fees, the least
        // in all taken, the one first chosen among equals (M8d slice CZ, ADR-0031 §2).
        let home_on_ground = |c: Option<((usize, build::Shape), Vec<build::Shape>)>,
                              first: bool| {
            let ((p, chosen), compact) = c.ok_or(Unbuilt::Nothing)?;
            let program = &catalog.buildings[p];
            let shape_h = |shape: build::Shape| {
                program
                    .shapes
                    .iter()
                    .find(|c| c.shape == shape)
                    .map_or(0.0, |c| c.hours(carry))
            };
            let mut best: Option<(f64, NewHome)> = None;
            for shape in std::iter::once(chosen).chain(compact) {
                let Some(home) = choose_home((p, shape), first) else {
                    continue;
                };
                let ground_h = home.receipt.map_or(0.0, |r| f64::from(r.hours));
                let total = shape_h(shape) + ground_h;
                if best.as_ref().is_none_or(|(b, _)| total < *b) {
                    best = Some((total, home));
                }
            }
            best.map(|(_, home)| home).ok_or(Unbuilt::NoGround)
        };
        match home {
            None => home_on_ground(home_choice(None), true),
            Some(current) => self
                .storehouse_plan(ctx, hh, labour_per_day, day)
                .and_then(|c| site(c, Some(hh.home), None, HOME_SHIFT_M))
                .or_else(|| {
                    self.workshop_plans(ctx, hh, labour_per_day, day)
                        .into_iter()
                        .find_map(|(c, firm)| site(c, Some(hh.home), Some(firm), HOME_SHIFT_M))
                })
                .ok_or(Unbuilt::Nothing)
                .or_else(|_| home_on_ground(home_choice(Some(current)), false)),
        }
    }

    /// The workshops household `hh` would build beside its home on day `day`, each with the firm
    /// it is for, the busiest firm first: for each of its open firms that have lately had more
    /// people working for them at once than a home has places for
    /// ([`crate::params::BuildParams::home_work_places`]) and have no workshop, the cheapest
    /// building with places for them all that its means and time pay for in full
    /// ([`build::workshop`]). None while the household has a workshop no firm works in: that
    /// goes to its firm without one ([`Population::hand_on_workshops`]). Never because a firm
    /// exists: only for the work it has.
    fn workshop_plans(
        &self,
        ctx: &Ctx,
        hh: &Household,
        labour_per_day: f64,
        day: i64,
    ) -> Vec<((usize, build::Shape), PermanentId)> {
        let catalog = ctx.catalog;
        let programs = self.programs_for(ctx, &hh.members, civ_land::PlotUse::Work);
        let Some(first) = programs.first().and_then(|&p| catalog.buildings.get(p)) else {
            return Vec::new();
        };
        let mine = || ctx.land.buildings.iter().filter(|b| b.household == hh.id);
        if mine().any(|b| self.free_workshop(catalog, b)) {
            return Vec::new();
        }
        let places = ctx.params.build.home_work_places;
        let mut firms: Vec<&crate::firm::Firm> = self
            .firms
            .iter()
            .filter(|f| {
                f.is_open()
                    && f.owner == hh.id
                    && u32::from(f.at_once(day)) > places
                    && !mine().any(|b| b.firm == Some(f.id) && b.standing())
            })
            .collect();
        if firms.is_empty() {
            return Vec::new();
        }
        firms.sort_by_key(|f| (std::cmp::Reverse(f.at_once(day)), f.id));
        let (budget_h, time_h) = self.home_means(ctx, hh, first, labour_per_day, day);
        firms
            .into_iter()
            .filter_map(|f| {
                build::workshop(
                    &catalog.buildings,
                    &programs,
                    u32::from(f.at_once(day)),
                    ctx.params.household.carry_kg,
                    budget_h,
                    time_h,
                )
                .map(|shop| (shop, f.id))
            })
            .collect()
    }

    /// The storehouse household `hh` would build beside its home on day `day`, if one is worth
    /// it ([`build::storehouse`]): for the goods its roofs have no room for now, at what they
    /// are worth to it, with its means and time reckoned to the store's own roof deadline.
    fn storehouse_plan(
        &self,
        ctx: &Ctx,
        hh: &Household,
        labour_per_day: f64,
        day: i64,
    ) -> Option<(usize, build::Shape)> {
        let catalog = ctx.catalog;
        let programs = self.programs_for(ctx, &hh.members, civ_land::PlotUse::Store);
        let first = programs.first().and_then(|&p| catalog.buildings.get(p))?;
        let goods = &catalog.goods;
        let stores = stores_now(hh, ctx.now, ctx.params, goods);
        let keeping = hh.keeping.with_stores(&stores, goods);
        let room = [keeping.raised_kg, keeping.roofed_kg, 0.0];
        let placed = crate::person::Keeping::fill(&stores, goods, room);
        let overflow: Vec<f64> = stores
            .iter()
            .zip(goods)
            .enumerate()
            .map(|(g, (kg, d))| {
                if d.sheltered_half_life_days > 0.0 {
                    (kg - placed.iter().map(|k| k[g]).sum::<f64>()).max(0.0)
                } else {
                    0.0
                }
            })
            .collect();
        if overflow.iter().all(|&kg| kg <= 0.0) {
            return None;
        }
        let cost_h: Vec<f64> = self
            .own_costs_of(ctx, hh)
            .into_iter()
            .map(|c| c.unwrap_or(0.0))
            .collect();
        let (budget_h, time_h) = self.home_means(ctx, hh, first, labour_per_day, day);
        build::storehouse(
            &catalog.buildings,
            &programs,
            &build::StoreNeed {
                overflow: &overflow,
                cost_h: &cost_h,
                raised_factor: ctx.params.household.raised_store_factor,
                horizon_days: ctx.params.build.store_horizon_days,
            },
            goods,
            ctx.params.household.carry_kg,
            budget_h,
            time_h,
        )
    }

    /// Whether a household of `members` can work on building `b`: its program is still in the
    /// content, and needs no technique or one someone among them knows (ADR-0009 §1).
    pub(crate) fn can_build(
        &self,
        catalog: &Catalog,
        members: &[PermanentId],
        b: &Building,
    ) -> bool {
        catalog
            .building_index(&b.spec.program)
            .and_then(|i| catalog.buildings.get(i))
            .is_some_and(|d| d.technique.is_none_or(|t| self.household_knows(members, t)))
    }

    /// The home programs a household of `members` may build: the people profile's dwellings, in
    /// its order, that someone among them knows how to build (or that need no technique).
    pub(crate) fn home_programs(&self, ctx: &Ctx, members: &[PermanentId]) -> Vec<usize> {
        self.programs_for(ctx, members, civ_land::PlotUse::Dwelling)
    }

    /// The programs of use `use_` a household of `members` may build: the people profile's, in
    /// its order, that someone among them knows how to build (or that need no technique).
    fn programs_for(
        &self,
        ctx: &Ctx,
        members: &[PermanentId],
        use_: civ_land::PlotUse,
    ) -> Vec<usize> {
        ctx.params
            .build
            .programs
            .iter()
            .copied()
            .filter(|&p| {
                ctx.catalog.buildings.get(p).is_some_and(|d| {
                    d.use_ == use_ && d.technique.is_none_or(|t| self.household_knows(members, t))
                })
            })
            .collect()
    }

    /// The goods household `hh` wants room for under a roof, kilograms: those that keep better
    /// there (ADR-0009 §5), what it holds now or a year's food for its members as its crop's
    /// grain, whichever is more.
    fn storage_need(&self, ctx: &Ctx, hh: &Household) -> f64 {
        let goods = &ctx.catalog.goods;
        let held: f64 = hh
            .stores
            .iter()
            .zip(goods)
            .filter(|(_, g)| g.sheltered_half_life_days > 0.0)
            .map(|(kg, _)| kg.max(0.0))
            .sum();
        let grain_kcal = ctx
            .catalog
            .crops
            .get(ctx.params.farm.crop)
            .and_then(|c| goods.get(c.good))
            .map_or(0.0, |g| g.kcal_per_kg);
        let year = if grain_kcal > 0.0 {
            hh.members.len() as f64
                * ctx.params.household.daily_kcal_per_person
                * DAYS_PER_YEAR as f64
                / grain_kcal
        } else {
            0.0
        };
        held.max(year)
    }

    /// What household `hh` can put into a home on day `day`, hours of work: what it puts in
    /// ([`build::HOUSE_INVESTMENT_SHARE`]) of the goods it could spare, as it would offer them
    /// for sale, beyond what it keeps (food to see it to its next harvest, its tools, and until
    /// its home is finished what it builds with; never its seed or firewood), valued in hours of
    /// its own work; and the time a home may take in all, [`build::HOUSE_TIME_SHARE`] of what a
    /// household giving `labour_per_day` can spare for building before its roof deadline. It
    /// reckons without founders' provisions, what carries a band to dependable harvests (research
    /// 05-06 §5.4) and stands for the herds real colonists drove in: they are not to spend, and
    /// what it keeps must come from the rest.
    fn home_means(
        &self,
        ctx: &Ctx,
        hh: &Household,
        program: &crate::params::BuildingDef,
        labour_per_day: f64,
        day: i64,
    ) -> (f64, f64) {
        let goods = &ctx.catalog.goods;
        let mut stores = stores_now(hh, ctx.now, ctx.params, goods);
        // Reckoned without its provisions, which neither count nor cover what it keeps.
        if let Some(p) = stores.get_mut(ctx.params.band.provisions_good) {
            *p = 0.0;
        }
        let costs = self.own_costs_of(ctx, hh);
        let keep = self.holding_of(ctx, hh, &stores).keep;
        let spare_h: f64 = stores
            .iter()
            .enumerate()
            .filter(|&(g, _)| goods.get(g).is_some_and(market::can_offer))
            .map(|(g, &held)| {
                let spare = held.max(0.0) - keep.get(g).copied().unwrap_or(0.0);
                let cost = costs.get(g).copied().flatten().unwrap_or(0.0);
                if spare > 0.0 && cost > 0.0 {
                    spare * cost
                } else {
                    0.0
                }
            })
            .sum();
        let days = (build::roof_deadline(day, program.roof_by_day) - day).max(1) as f64;
        let time_h = labour_per_day * build::BUILD_LABOUR_SHARE * days * build::HOUSE_TIME_SHARE;
        (build::HOUSE_INVESTMENT_SHARE * spare_h, time_h)
    }

    /// Household `household` claims the ground for the hut it planned today and begins it: the
    /// plot is marked out, the building is begun, and the household's home moves to it (a
    /// household that has a home moves when the new one's roof is on). `None` if the ground is no
    /// longer clear.
    /// Household `household` begins the first home it would lay out now beside its settlement's
    /// hearth, as if it had none: for tests.
    #[doc(hidden)]
    pub fn begin_home_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
    ) -> Option<PermanentId> {
        let hh = self.household(household)?.clone();
        let hearth = hh.settlement.and_then(|s| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        });
        let labour = self.labour_per_day(&hh.members, ctx.now, ctx.params);
        let site = self
            .new_building(ctx, &hh, hearth, hh.id, labour, None)
            .ok()?;
        self.home_sites
            .insert(household, (ctx.now.day_index(), Some(site)));
        self.begin_home(ctx, household)
    }

    fn begin_home(&mut self, ctx: &mut Ctx, household: PermanentId) -> Option<PermanentId> {
        let (_, site) = self.home_sites.remove(&household)?;
        let site = site?;
        // A workshop is begun only for a firm still open, of this household, with none standing.
        if let Some(f) = site.firm {
            let wanted = self
                .firm(f)
                .is_some_and(|f| f.is_open() && f.owner == household)
                && !ctx
                    .land
                    .buildings
                    .iter()
                    .any(|b| b.firm == Some(f) && b.standing());
            if !wanted {
                return None;
            }
        }
        let def = ctx
            .catalog
            .building_index(&site.spec.program)
            .and_then(|i| ctx.catalog.buildings.get(i))?;
        let rect = build::plot_rect(&site.spec, def);
        if !build::plot_clear(ctx.land, ctx.map, ctx.nav, &rect) {
            return None;
        }
        // A household building a new home lives in the old until the new one's roof is on; one
        // building a store or a workshop lives where it did.
        let housed = def.use_ != civ_land::PlotUse::Dwelling
            || ctx.land.buildings.iter().any(|b| {
                b.household == household
                    && b.finished()
                    && b.standing()
                    && ctx.catalog.is_dwelling(&b.spec.program)
            });
        // A parcel given for a fee is paid for first, to the store (M8c slice CU): unpaid, nothing
        // is begun.
        if let (Some(_), Some((good, amount))) = (site.parcel, site.fee) {
            let store = self
                .household(household)
                .and_then(|x| x.settlement)
                .and_then(|s| self.polity_of(s))
                .map(|pi| self.polities[pi].id)?;
            let legs = [crate::ledger::Leg {
                from: household,
                to: store,
                good,
                amount,
            }];
            if !self.transfer(
                ctx.now,
                ctx.params,
                &ctx.catalog.goods,
                &legs,
                crate::ledger::Channel::GroundFee,
            ) {
                return None;
            }
        }
        // What its polity sees of the ground it takes (M8c slice CT).
        self.ground_taken(
            ctx,
            household,
            &rect,
            def.use_ == civ_land::PlotUse::Dwelling,
        );
        let (plot, id) = (ctx.ids.allocate(), ctx.ids.allocate());
        ctx.land.plots.push(Plot {
            id: plot,
            household,
            rect,
            use_: def.use_,
            since: ctx.now,
            receipt: site.receipt,
            allotted: site.parcel,
        });
        // A parcel given is staked by the plan's keeper (M8c slice CU).
        if let Some(a) = site.parcel {
            let settlement = self.household(household).and_then(|x| x.settlement);
            self.stake_parcel(ctx, settlement, a.law);
        }
        // Its builders' taste followed the building that moved it most (M3b slice R).
        let style_from = self.household(household).and_then(|h| h.admired);
        ctx.land.buildings.push(Building {
            firm: site.firm,
            style_from,
            ..Building::new(id, household, plot, site.spec.clone(), ctx.now)
        });
        // Ground that drops too far across the plot is levelled first (ADR-0010 §2).
        let mut stages = site.stages;
        let levelling = build::level_plot(
            ctx.land,
            ctx.map,
            ctx.ids,
            &ctx.params.build.levelling,
            (plot, rect),
            household,
            ctx.now,
        );
        if let Some(first) = stages.first_mut() {
            first.labour_h += levelling;
        }
        self.stage_needs.insert(id, stages);
        // Whether it keeps the rules for building in force that bind it (M6b slice BH).
        self.count_rule_keeping(ctx, household, id);
        let at = build::centre_m(&site.spec);
        if !housed
            && let Some(&hd) = self.hh_index.get(&household)
            && let Some(x) = self.households.get_mut(hd)
        {
            x.home = at;
            self.homes_moved += 1;
        }
        Some(id)
    }

    /// The household of `hh`'s settlement that could give it the most food now, when it holds
    /// `held` kcal of a need of `kcal_day` a day and is short; the walk to its home is counted
    /// from the settlement's hearth (from `field_key`'s travel field).
    fn best_giver(
        &self,
        ctx: &Ctx,
        hh: &Household,
        field_key: PermanentId,
        kcal_day: f64,
        held: f64,
    ) -> Option<GiverOption> {
        let settlement = hh.settlement?;
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let want = (params.household.food_target_days * kcal_day - held).max(0.0);
        if want <= 0.0 {
            return None;
        }
        let reach = &self.homes.get(&field_key)?.reach;
        let mut found: Vec<GiverOption> = Vec::new();
        for x in self.households_in(settlement) {
            if x.id == hh.id {
                continue;
            }
            let kcal = spare_food_kcal(x, now, params, goods).min(want);
            if kcal <= 0.0 {
                continue;
            }
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, x.home)) else {
                continue;
            };
            found.push(GiverOption {
                household: x.id,
                walk_min: f64::from(secs) / 60.0,
                at: x.home,
                kcal,
            });
        }
        // The common store, at its keeper's or the hearth, when a law its members know keeps one
        // (ADR-0013 §4).
        if let Some(s) = ctx.land.settlements.iter().find(|s| s.id == settlement)
            && let Some(store) = self.relief_option(ctx, hh, kcal_day, want, reach, s.hearth_m)
        {
            found.push(store);
        }
        // The store of a faction one of its members belongs to, at its organizer's (M4c slice
        // AH).
        if let Some(store) = self.aid_option(ctx, hh, kcal_day, want, reach) {
            found.push(store);
        }
        // Those who could give the most; among them, the nearest once a walk is weighed against
        // how well the household's members regard the one who stands for each (ADR-0014 §3):
        // they would walk `ask_known_min` further to ask someone they regard fully than a
        // stranger.
        let most = found.iter().map(|g| g.kcal).fold(0.0, f64::max);
        let tp = &params.ties;
        let day = now.day_index();
        let cost = |g: &GiverOption| {
            // A faction's store is asked of its organizer.
            let elder = self
                .factions
                .get(g.household)
                .map(|f| f.organizer)
                .or_else(|| self.elder_of(g.household, now, params));
            let regard = elder.map_or(0.0, |e| {
                hh.members
                    .iter()
                    .map(|&m| self.ties.regard(m, e, day, tp))
                    .fold(0.0, f64::max)
            });
            g.walk_min - tp.ask_known_min * regard.clamp(0.0, 1.0)
        };
        found
            .into_iter()
            .filter(|g| g.kcal >= most - 1e-6)
            .map(|g| (cost(&g), g))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, g)| g)
    }

    /// Where this household receives supplies while sheltering from its wet home.
    fn supply_at(ctx: &Ctx, hh: &Household) -> (f32, f32) {
        let wet = ctx
            .land
            .flood_depths
            .get(cell_of(ctx.map, hh.home))
            .copied()
            .unwrap_or(0.0)
            > 0.0;
        if wet {
            hh.settlement
                .and_then(|id| ctx.land.settlements.iter().find(|s| s.id == id))
                .map_or(hh.home, |s| s.hearth_m)
        } else {
            hh.home
        }
    }

    /// Reconsiders an interrupted person's ordinary options (tests).
    #[doc(hidden)]
    pub fn reconsider_for_tests(&mut self, ctx: &mut Ctx, who: PermanentId) {
        let h = self
            .people
            .iter()
            .find(|(_, p)| p.id == who)
            .map(|(h, _)| h);
        if let Some(h) = h {
            self.decide(ctx, h, 0);
        }
    }

    fn decide(&mut self, ctx: &mut Ctx, h: Handle<Person>, depth: u32) {
        // Someone of a household-resolved settlement decides nothing: their day is lived at
        // midnight (ADR-0032 §2).
        if self.resolved_person(h) && !self.planning {
            self.park(ctx, h);
            return;
        }
        let now = ctx.now;
        let params = ctx.params;
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, now, params);
        let (hh_id, age, pos, who, laden) = (
            p.household,
            p.age_years(now),
            p.pos,
            p.id,
            p.carrying.kg > 0.0 || p.carrying.water_l > 0.0,
        );
        let Some(hh) = self.household(hh_id).cloned() else {
            return;
        };
        let supply_at = Self::supply_at(ctx, &hh);
        // An interrupted delivery keeps its load. Complete it before starting another task;
        // waiting for a safe route neither deposits remotely nor overwrites carried water.
        if laden {
            if self.knows_home_afire(who, hh_id, now, params) {
                self.wait(ctx, h, WAIT_AFTER_FAILURE_MIN);
                return;
            }
            if let Some(p) = self.people.get_mut(h) {
                p.act = Activity {
                    def: p.act.def,
                    target: Target::Home,
                    steps: vec![Step::Walk { to: supply_at }, Step::Deposit],
                    step: 0,
                    started: now,
                    step_started: now,
                    step_ends: now,
                    version: p.act.version.wrapping_add(1),
                };
                p.trip = None;
            }
            self.run_steps(ctx, h, depth + 1);
            return;
        }
        // The household's view of its options: kept in Accelerated mode, worked out afresh at
        // each decision otherwise (ADR-0011 §4).
        let hh_view = if ctx.approx.household_view {
            let kept = self.views.remove(&hh_id);
            if kept.is_some() {
                self.view_counts.0 += 1;
            } else {
                self.view_counts.1 += 1;
            }
            kept.unwrap_or_default()
        } else {
            HouseholdView::default()
        }
        .cells(ctx.catalog.activities.len());
        let hearth = hh.settlement.and_then(|s| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        });
        // One travel field per settlement, from its hearth.
        let mut field_key = match (hh.settlement, hearth) {
            (Some(s), Some(_)) => s,
            _ => hh_id,
        };
        self.home_field(ctx, field_key, hearth.unwrap_or(hh.home));
        // Sharing a hearth's travel field assumes homes remain in its walking catchment.
        // A closed ford can separate a nearby home: plan from that home's own bank instead.
        if field_key != hh_id
            && self
                .homes
                .get(&field_key)
                .is_some_and(|f| f.reach.seconds_to(cell_of(ctx.map, supply_at)).is_none())
        {
            field_key = hh_id;
            self.home_field(ctx, field_key, supply_at);
        }
        self.refresh_kin();
        let goods = &ctx.catalog.goods;
        let member_count = hh.members.len().max(1);
        let members = member_count as f64;
        let kcal_day = members * params.household.daily_kcal_per_person;
        let fuel_day = |d: i64| fuel_per_day(params, member_count, d);
        let stores = hh.stores_at_time(now, goods, &fuel_day);
        // Food ready to eat (bread, meat, berries), and all the household has to live on (grain
        // and flour count, made ready by a recipe).
        let (food_all, food_raw) = food_kcal(&stores, goods);
        // Seed is food only to someone in real hunger, once ground, and never the seed to sow
        // the ground already cropped.
        let reserve_ok = self
            .people
            .get(h)
            .is_some_and(|p| may_eat_reserve(p, now, params));
        let protected = farm::protected_seed(self.fields_of(ctx.land, hh_id), &ctx.catalog.crops);
        let seed_kcal = if reserve_ok {
            let mut spare = stores.clone();
            if let Some((g, kg)) = protected
                && let Some(s) = spare.get_mut(g)
            {
                *s = (*s - kg).max(0.0);
            }
            reserve_food_kcal(&spare, goods)
        } else {
            0.0
        };
        let stock = stock_kcal(&stores, goods) + seed_kcal;
        let fuel = fuel_kg(&stores, goods);
        let edible = if fuel > 0.0 { food_all } else { food_raw };
        let household_fuel_day = fuel_per_day(params, member_count, now.day_index());
        let water_day = members * hh.water_use(&params.household);
        let water_l = hh.water_at_time(now, water_day);
        let doy = civ_land::day_of_year(now.day_index());
        let sun = needs::daylight(params.latitude_deg, doy);
        let minute = now.minute_of_day();
        let dark = minute < sun.0 || minute >= sun.1;
        let until_sunrise = if minute < sun.0 {
            sun.0 - minute
        } else {
            MINUTES_PER_DAY - minute + needs::daylight(params.latitude_deg, doy + 1).0
        };
        let evening_end = (sun.1 + 240).min(MINUTES_PER_DAY);
        let evening_start = sun.1 - 60;
        let evening = if minute >= evening_start && minute < evening_end {
            1.0
        } else {
            0.0
        };
        // The food that would see the household through to its next harvest, with a margin
        // (research 08-01 §2.3: a seasonal reserve until the next reliable food, plus 0-90 days).
        let food_outlook_days = ctx.catalog.crops.get(params.farm.crop).map_or(0.0, |c| {
            farm::days_to_harvest(c, self.fields_of(ctx.land, hh_id), now.day_index())
                + params.household.harvest_margin_days
        });
        // A band they joined that musters within the day (M6c slice BJ).
        let band_facts = match self.people.get(h).map(|p| (p.id, p.pos)) {
            Some((who, pos)) => self.band_facts(ctx, who, pos),
            None => None,
        };
        // Roofs a fire they know of threatens (M6b slice BD, step three).
        let fire_facts = self.fire_facts(
            ctx,
            h,
            &hh,
            water_l,
            Self::short_of_water(water_l, water_day),
            field_key,
        );
        let Some(p) = self.people.get(h) else {
            return;
        };
        let circ = needs::circadian(&params.sleep, minute, sun);
        // In a leisure block (Accelerated mode, ADR-0011 §4), leisure lasts no further than the
        // next turn of the person's day: when hunger begins, when sleep becomes an option, or the
        // day's own turns.
        let leisure_until_min = ctx.approx.leisure_blocks.then(|| {
            let turns = [
                sun.0,
                sun.1 - decide::LAST_WATER_BEFORE_DARK_MIN as i64,
                evening_start,
                sun.1,
                evening_end,
            ];
            let next_turn = turns
                .iter()
                .filter(|&&t| t > minute)
                .min()
                .map_or(until_sunrise, |&t| t - minute);
            let longest = ctx
                .catalog
                .activities
                .iter()
                .filter(|a| {
                    matches!(
                        a.behavior,
                        Behavior::Rest | Behavior::Play | Behavior::Socialize
                    )
                })
                .map(|a| f64::from(a.max_minutes))
                .fold(0.0, f64::max);
            leisure_until(p, now, params, sun, next_turn as f64, longest)
        });
        let facts = Facts {
            age,
            capacity: interpolate(&params.capacity_by_age, age),
            hunger: hunger(p, now, params).min(params.decision.max_hunger_drive),
            sleep_drive: f64::from(p.sleep_pressure) * circ,
            sleep_pressure: f64::from(p.sleep_pressure),
            loneliness: 1.0 - f64::from(p.relatedness),
            dark,
            daylight_left_min: if dark { 0.0 } else { (sun.1 - minute) as f64 },
            clock_min: minute as f64,
            evening,
            until_sunrise_min: until_sunrise as f64,
            // Light for the evening: the lamp's fuel, or the burners' gas lit until they go out
            // (M10b slice DY).
            lamp_min: lamp_of(goods, &stores).map_or(0.0, |l| l.3).max(
                ctx.land
                    .gas
                    .lit_until(hh_id, now.minutes())
                    .map_or(0.0, |u| (u - now.minutes()) as f64),
            ),
            food_days: stock / kcal_day.max(1.0),
            food_target_days: params.household.food_target_days,
            food_outlook_days,
            water_days: water_l / water_day.max(1e-6),
            water_target_days: params.household.water_target_days,
            household_kcal_day: kcal_day,
            has_food: edible > 1.0,
            food_needs_fire: food_all > 1.0 && edible <= 1.0,
            fuel_days: fuel / household_fuel_day.max(1e-6),
            fuel_target_days: params.household.fuel_target_days,
            household_fuel_day,
            at_home: (pos.0 - hh.home.0).abs() < 1.0 && (pos.1 - hh.home.1).abs() < 1.0,
            home: supply_at,
            hearth: hh.settlement.zip(hearth),
            gathering: self.gathering_facts(ctx, p.id, age, &hh, minute, evening_start),
            petition: self.petition_facts(ctx, p.id, age, &hh, minute, evening_start),
            poll: self.poll_facts(ctx, p.id, age, &hh, minute, evening_start),
            campaign: self.campaign_points(ctx, p.id),
            watch: self.watch_facts(ctx, p.id, &hh, dark),
            light: self.light_facts(ctx, p.id, &hh, dark),
            rounds: self.rounds_facts(ctx, p.id, &hh, dark),
            midden: self.midden_facts(ctx, p.id, &hh, dark),
            carry: self.carry_facts(ctx, p.id, field_key, dark),
            crossing: self.crossing_facts(ctx, &hh, p.id, field_key, dark),
            well: self.well_facts(ctx, &hh, p.id, dark),
            levee: self.levee_facts(ctx, &hh, p.id, field_key, dark),
            boat: self.boat_facts(ctx, &hh, field_key, dark),
            cart: self.cart_facts(ctx, &hh, dark),
            way: self.way_facts(ctx, &hh, p.id, dark),
            raise: self.raise_facts(ctx, &hh, p.id, dark),
            enclosure: self.enclosure_facts(ctx, &hh, p.id, field_key, dark),
            ditch: self.ditch_facts(ctx, &hh, field_key, dark),
            mill: self.mill_facts(ctx, &hh, field_key, dark),
            band: band_facts,
            hurt: self
                .order
                .hurt_until(p.id)
                .is_some_and(|d| d >= now.day_index()),
            ill: self.sickness.ill(p.id, now.day_index()),
            home_afire: self.knows_home_afire(p.id, p.household, now, params),
            fire: fire_facts,
            tend: self.tend_facts(ctx, p.id, &hh, field_key),
            call: self.call_facts(ctx, p.id, &hh, field_key),
            teach: self.teach_facts(ctx, p.id),
            study: self.study_facts(ctx, p.id, field_key),
            write: self.write_facts(ctx, p.id),
            read: self.read_facts(ctx, p.id),
            home_flooded: ctx
                .land
                .flood_depths
                .get(cell_of(ctx.map, hh.home))
                .copied()
                .unwrap_or(0.0)
                > 0.0,
            shelter: hearth,
        };
        let limits = Limits {
            sleep_needed_min: needs::minutes_to_rest(&params.sleep, f64::from(p.sleep_pressure)),
            sleep_min: params.sleep.min_hours * 60.0,
            sleep_max: params.sleep.max_hours * 60.0,
            nap: (params.sleep.nap_min_minutes, params.sleep.nap_max_minutes),
            sleep_threshold: params.sleep.wake_pressure,
            meal_min: params.energy.meal_minutes,
            carry_kg: params.household.carry_kg,
        };
        self.ensure_priors(ctx, now.day_index());
        // Farming: what the household can put into its fields, what it holds, what it needs.
        let today = now.day_index();
        let crop = ctx.catalog.crops.get(params.farm.crop);
        let grain_kcal = crop
            .and_then(|c| goods.get(c.good))
            .map_or(0.0, |g| g.kcal_per_kg);
        let held = |g: Option<usize>| {
            g.and_then(|g| stores.get(g))
                .copied()
                .unwrap_or(0.0)
                .max(0.0)
        };
        let seed_kg = held(crop.map(|c| c.seed_good));
        let grain_days = held(crop.map(|c| c.good)) * grain_kcal / kcal_day.max(1.0);
        let target_days = params.farm.grain_target_days.max(1e-6);
        let labour_per_day = self.labour_per_day(&hh.members, now, params);
        let farm_view = crop.map(|c| FarmView {
            crop: c,
            kcal_per_kg: grain_kcal,
            // A field a plan reserved begins no crop again; one standing comes in (M8c slice CU).
            fields: self
                .fields_of(ctx.land, hh_id)
                .filter(|f| {
                    f.reserved.is_none()
                        || matches!(
                            f.stage,
                            civ_land::FieldStage::Sown | civ_land::FieldStage::Reaped
                        )
                })
                .collect(),
            day: today,
            labour_per_day,
            seed_kg,
            room: target_days / (target_days + grain_days),
            need_kg: farm::need_grain_kg(member_count, params, grain_kcal),
            plan_yield_share: params.farm.plan_yield_share,
            workable_share: workable_share(ctx.land, params.farm.crop),
            peak_ratio: peak_ratio(params),
            climatology: Some(&ctx.land.climatology),
            manuring: Some(farm::Manuring {
                midden_kg: hh.midden.at_time(now, member_count, &params.midden),
                grain_per_kg: manure_grain_per_kg(params, ctx.land_params, c),
                midden: &params.midden,
            }),
        });
        let field_ha = params.farm.field_m * params.farm.field_m / 10_000.0;
        let wants_land = farm_view
            .as_ref()
            .is_some_and(|v| v.wants_new_field(field_ha, 0.0));
        let site = match crop {
            Some(c) if wants_land => self.site_for(ctx, &hh, field_key, c),
            _ => None,
        };
        // Building: what is under way or what the household would begin, what it still has
        // to bring for it, and how pressing it is.
        let plan = self.home_plan(ctx, &hh, hearth, field_key, labour_per_day);
        let (build_need, build_urgency) = match &plan {
            Ok(p) => {
                let def = &ctx.catalog.buildings[p.def];
                let need = p.work.need_by_good(def, Stage::Roof, &stores, goods.len());
                // The work left counts the loads still to be cut and carried home.
                let loads = need.iter().sum::<f64>() / params.household.carry_kg.max(1e-6);
                let hours = p.work.hours_left(Stage::Roof) + loads * build::HAUL_H_PER_LOAD;
                (
                    need,
                    build::urgency(hours, p.deadline, labour_per_day, today),
                )
            }
            Err(_) => (Vec::new(), 0.0),
        };
        // Tools: what the household wants for its working members, what of it is free (not in
        // the hands of members at work), and what the tools it lacks are made of.
        let catalog = ctx.catalog;
        let ages: Vec<f64> = hh
            .members
            .iter()
            .filter_map(|m| self.person(*m))
            .map(|q| q.age_years(now))
            .collect();
        let knows = |t: usize| self.household_knows(&hh.members, t);
        let mut wants = make::tool_wants_for(catalog, &ages, params.family.independent_age, &knows);
        cap_power_wants(ctx, hh.id, &mut wants);
        let costs_once = OnceCell::new();
        let costs = || {
            costs_once
                .get_or_init(|| self.own_costs_of(ctx, &hh))
                .clone()
        };
        // A light it knows how to make is wanted where an hour lit costs it less of its own
        // hours, in fuel and in the lamp's wear, than the hour of making at home it gives after
        // dark (M10b slice DU; brief §2.3).
        for (g, d) in goods.iter().enumerate() {
            let Some(light) = d.light.as_ref() else {
                continue;
            };
            let known = catalog
                .recipes
                .iter()
                .any(|r| r.outputs.iter().any(|&(o, _)| o == g) && r.technique.is_none_or(&knows));
            if !known || ages.iter().all(|&a| a < params.family.independent_age) {
                continue;
            }
            // A piped burner's light comes by a main, not from what the household makes: the works
            // of a main laid whole keeps one by it to fit the next home that connects (M10b slice
            // DY, fourth part).
            if light.piped {
                if ctx
                    .land
                    .gas
                    .mains
                    .iter()
                    .any(|m| m.works == hh.id && m.whole() && !m.broken)
                    && let Some(w) = wants.get_mut(g)
                {
                    *w = w.max(1.0);
                }
                continue;
            }
            // Its fuel decides: the lamp's own hours, spread over its life, count where the
            // household can put a figure on them (its clay may come from a pit it has not found).
            let Some(per_hour) = light_hour_cost(goods, &costs(), g) else {
                continue;
            };
            if per_hour < 1.0
                && let Some(w) = wants.get_mut(g)
            {
                *w = w.max(1.0);
            }
        }
        let me = self.people.get(h).map(|p| p.id);
        let mut free_tools: Vec<f64> = stores
            .iter()
            .zip(goods)
            .enumerate()
            .map(|(i, (s, g))| {
                if g.tool.is_some() {
                    make::in_use(goods, i, *s)
                } else {
                    0.0
                }
            })
            .collect();
        // A tool that serves for another counts toward the work that needs that one.
        for (g, d) in goods.iter().enumerate() {
            if let Some(t) = d.tool.as_ref().and_then(|x| x.serves)
                && t != g
            {
                let more = free_tools.get(g).copied().unwrap_or(0.0);
                if let Some(f) = free_tools.get_mut(t) {
                    *f += more;
                }
            }
        }
        let held_tools = free_tools.clone();
        for m in &hh.members {
            let Some(q) = self.person(*m).filter(|q| Some(q.id) != me) else {
                continue;
            };
            let at_work = matches!(
                q.act.steps.get(q.act.step as usize),
                Some(Step::Walk { .. } | Step::Work { .. })
            );
            if let (true, Some(def)) = (at_work, catalog.activities.get(usize::from(q.act.def))) {
                for t in catalog.tools_of(def) {
                    if let Some(f) = free_tools.get_mut(t) {
                        *f -= 1.0;
                    }
                }
            }
        }
        let held_of = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let tool_need: Vec<f64> = (0..goods.len())
            .map(|g| make::tool_need(wants[g], make::serving(goods, g).map(held_of).sum()))
            .collect();
        // Pots: as many more as would keep the food that lies anywhere but a raised floor or a
        // pot, and the food the next would keep over the store horizon (M3b slice Q).
        let keeping_now = hh.keeping.with_stores(&stores, goods);
        let pots: Vec<(f64, f64)> = goods
            .iter()
            .map(|d| {
                d.store.as_ref().map_or((0.0, 0.0), |s| {
                    build::pots_wanted(
                        &stores,
                        goods,
                        &keeping_now,
                        hh.sheltered,
                        s.keeps_kg,
                        params.build.store_horizon_days,
                    )
                })
            })
            .collect();
        // Tools others want and nobody offers, which the household makes for fewer hours than
        // they would give: worth making to sell (slice I).
        let for_sale = self.for_sale(ctx, &hh, &stores, &tool_need);
        let mut tool_material = vec![0.0; goods.len()];
        let mut tool_material_blocked = vec![false; goods.len()];
        // How each tool it needs is made: its own recipe, or a tool that serves for it that costs
        // fewer of its hours for each hour of use (M7d slice CE). And the materials those ways
        // need that it makes itself (copper for a copper axe): the tool each is for, and how
        // much of it.
        let can_work = |r: &crate::params::RecipeDef| self.can_work_way(ctx, &hh, &stores, r);
        let ways: Vec<Option<usize>> = (0..goods.len())
            .map(|t| {
                if goods[t].tool.is_none() {
                    None
                } else if goods[t].tool.as_ref().is_some_and(|x| x.serves.is_some()) {
                    catalog
                        .recipes
                        .iter()
                        .position(|r| r.outputs.iter().any(|&(g, _)| g == t))
                } else {
                    make::way_to_tool(catalog, t, &can_work, &costs)
                }
            })
            .collect();
        let mut made_for: Vec<Option<(usize, f64)>> = vec![None; goods.len()];
        for (t, need) in tool_need.iter().enumerate() {
            if *need <= 0.0 && for_sale.get(t).copied().unwrap_or(0.0) <= 0.0 {
                continue;
            }
            let Some(r) = ways[t].and_then(|r| catalog.recipes.get(r)) else {
                continue;
            };
            let blocked = make::tools_for(goods, t, &held_of) < decide::MIN_TOOL;
            for &(g, amount) in &r.inputs {
                let short = (amount - held_of(g)).max(0.0);
                if let Some(m) = tool_material.get_mut(g) {
                    *m += short;
                }
                if blocked && let Some(b) = tool_material_blocked.get_mut(g) {
                    *b = true;
                }
                // A material it makes itself, and what that is made of.
                let Some((made, per_unit)) = catalog.recipes.iter().find_map(|m| {
                    m.outputs
                        .iter()
                        .find(|&&(o, _)| o == g)
                        .filter(|_| short > 0.0 && can_work(m))
                        .map(|&(_, a)| (m, a))
                }) else {
                    continue;
                };
                if per_unit <= 0.0 {
                    continue;
                }
                made_for[g] = Some((t, *need));
                let units = short / per_unit;
                for &(i, a) in &made.inputs {
                    if let Some(m) = tool_material.get_mut(i) {
                        *m += (a * units - held_of(i)).max(0.0);
                    }
                    if blocked && let Some(b) = tool_material_blocked.get_mut(i) {
                        *b = true;
                    }
                }
            }
        }
        // A whole lamp it holds wants its fuel, to `LAMP_FUEL_HOURS` of light: made as a tool's
        // material is, for the lamp (M10b slice DU).
        if let Some((lamp, fuel, kg_h)) = goods.iter().enumerate().find_map(|(g, d)| {
            let l = d.light.as_ref().filter(|l| !l.piped)?;
            (make::in_use(goods, g, held_of(g)) >= decide::MIN_TOOL).then_some((
                g,
                l.fuel?,
                l.kg_per_hour,
            ))
        }) {
            let reserve = kg_h * LAMP_FUEL_HOURS;
            let short = (reserve - held_of(fuel)).max(0.0);
            if short > 0.0 && reserve > 0.0 {
                made_for[fuel] = Some((lamp, short / reserve));
                if let Some(m) = tool_material.get_mut(fuel) {
                    *m += short;
                }
            }
        }
        // A works laying a gas main wants the pipe the rest of it takes, as a tool's material is
        // wanted (M10b slice DY, fourth part).
        if let Some(p) = ctx.land_params.gas_main
            && let Some(m) = ctx
                .land
                .gas
                .mains
                .iter()
                .find(|m| m.works == hh.id && !m.whole())
        {
            let need = f64::from(m.length_m - m.laid_m).max(0.0) * p.good_kg_m;
            let short = (need - held_of(p.good)).max(0.0);
            if short > 0.0
                && let Some(t) = tool_material.get_mut(p.good)
            {
                *t += short;
            }
        }
        // And what the pots it would gain from are made of, a session's worth at most.
        for (t, &(want, _)) in pots.iter().enumerate() {
            let Some((r, per_unit)) = catalog.recipes.iter().find_map(|r| {
                r.outputs
                    .iter()
                    .find(|&&(g, _)| g == t)
                    .map(|&(_, amount)| (r, amount))
            }) else {
                continue;
            };
            if want <= 0.0 || per_unit <= 0.0 {
                continue;
            }
            let units = (want / per_unit).min(r.max_units.max(1.0));
            for &(g, amount) in &r.inputs {
                if let Some(m) = tool_material.get_mut(g) {
                    *m += (amount * units - held_of(g)).max(0.0);
                }
            }
        }
        let home = self.homes.get(&field_key);
        let short = Self::short_of_water(water_l, water_day);
        let water = self.water_source(ctx, home, Some((&hh, short)), me);
        let land_params = ctx.land_params;
        let map = ctx.map;
        let (places, priors) = (&self.places, &self.priors);
        // Each activity's kind of place; what the settlement knows of each resource, by place
        // (the last of any repeats wins), and the best return it has seen of each anywhere:
        // worked out only if a place to gather is.
        let gathering_once = OnceCell::new();
        let gathering = || {
            gathering_once.get_or_init(|| {
                let place_kinds: Vec<Option<usize>> = catalog
                    .activities
                    .iter()
                    .map(|a| a.resource.and_then(|r| self.places_for(land_params, r)))
                    .collect();
                let mut seen_most = vec![0.0f64; land_params.resources.len()];
                let memory: civ_core::FastMap<(u16, u32), &KnownPatch> = hh
                    .known
                    .iter()
                    .map(|k| {
                        if let Some(m) = seen_most.get_mut(usize::from(k.resource)) {
                            *m = m.max(f64::from(k.rate));
                        }
                        ((k.resource, k.patch), k)
                    })
                    .collect();
                (place_kinds, seen_most, memory)
            })
        };
        // Where each dig activity would dig (M3b slice Q): the deposit of its good the settlement
        // knows that brings most for the walk, while the household needs the good; otherwise the
        // nearest it knows, to be weighed and found not needed.
        let toward = hearth.unwrap_or(hh.home);
        let toward = (f64::from(toward.0), f64::from(toward.1));
        // A place the household has heard another polity's law claims is worth the content's
        // share of what it would yield to it (M5c slice AU, ADR-0020 §5: outsiders use a claimed
        // place by leave); it may still go there when nothing else is as good. One its own
        // polity claims too it holds as its own, and so one an agreement in force gives its people
        // leave to use, when one of them knows their own polity's law deciding it. A household
        // some of whose people are going hungry weighs no claim: need, not another's word,
        // decides where it gathers (M5c slice AX; research 04-09 §5.3: what a breach is worth
        // turns on the agent's needs and alternatives).
        let own = hh.settlement.and_then(|s| self.polity_of(s));
        let own_polity = own.map(|i| self.polities[i].id);
        let claimed_worth = ctx.params.relations.claimed_worth;
        let hungry = hh.members.iter().any(|&m| {
            self.person(m)
                .is_some_and(|q| may_eat_reserve(q, now, params))
        });
        let worth = |place: crate::uses::Place| {
            if hungry {
                return 1.0;
            }
            let leave = |from: PermanentId| {
                own.is_some_and(|i| self.leave_known(from, self.polities[i].id, i, &hh.members))
            };
            let theirs = self
                .claims_heard
                .claimed_without_leave(hh.id, place, own_polity, leave)
                .is_some();
            if theirs && !own.is_some_and(|i| self.polities[i].claim_on(place).is_some()) {
                claimed_worth
            } else {
                1.0
            }
        };
        // What a kilogram of a fuel stands for at the hearth (M10a slice DS); 1 for anything else.
        let heat_of = |g: usize| match catalog.goods.get(g) {
            Some(d) if d.purpose == GoodUse::Fuel => crate::params::fuel_ratio(&catalog.goods, g),
            _ => 1.0,
        };
        let dig = |def: usize| -> Option<PatchOption> {
            {
                let a = catalog.activities.get(def)?;
                let g = a.digs?;
                let settlement = hh.settlement?;
                let reach = &home?.reach;
                let good = catalog.goods.get(g)?;
                let need_kg = build_need.get(g).copied().unwrap_or(0.0);
                let tool_need_kg = tool_material.get(g).copied().unwrap_or(0.0);
                // A fuel is dug for the hearth, so where it would be dug is wanted too (M10a slice
                // DS).
                let needed = need_kg > 0.0 || tool_need_kg > 0.0 || good.purpose == GoodUse::Fuel;
                let hours = f64::from(a.max_minutes) / 60.0;
                let mut best: Option<(f64, PatchOption)> = None;
                for d in ctx.land.deposits.iter().filter(|d| {
                    usize::from(d.body.good) == g
                        && d.left_kg() > 0.0
                        && self.knows_deposit(settlement, d.id)
                }) {
                    let centre = (
                        (d.body.at_cm.0 as f64 / 100.0) as f32,
                        (d.body.at_cm.1 as f64 / 100.0) as f32,
                    );
                    let at = if needed {
                        let side = params.digging.pit_side_m;
                        match digging::dig_place(ctx.land, map, d, toward, side) {
                            Some(at) => at,
                            None => continue,
                        }
                    } else {
                        centre
                    };
                    let Some(secs) = reach.seconds_to(cell_of(map, at)) else {
                        continue;
                    };
                    let walk = f64::from(secs) / 60.0;
                    let rates = digging::rates(ctx.land_params, &d.body, params.digging.h_per_m3);
                    // Less where water standing in the pit must be lifted out first (M10a
                    // slice DS).
                    let kg_per_hour = digging::dig_kg_per_hour(&d.body, rates)
                        * digging::dry_share(
                            (ctx.land, map, ctx.land_params),
                            &params.digging,
                            d,
                            now,
                            hours,
                        );
                    if walk > f64::from(a.max_walk_minutes) || kg_per_hour <= 0.0 {
                        continue;
                    }
                    let value = kg_per_hour * hours / (hours + 2.0 * walk / 60.0)
                        * worth(crate::uses::Place::Deposit(d.id));
                    if best.is_none_or(|(v, _)| value > v) {
                        best = Some((
                            value,
                            PatchOption {
                                patch: 0,
                                walk_min: walk,
                                kg_per_hour,
                                purpose: good.purpose,
                                kcal_per_kg: good.kcal_per_kg,
                                fuel_ratio: heat_of(g),
                                stored_days: 0.0,
                                need_kg,
                                urgency: build_urgency,
                                tool_need_kg,
                                tool_blocked: tool_material_blocked
                                    .get(g)
                                    .copied()
                                    .unwrap_or(false),
                                at,
                                deposit: Some(d.id),
                                by_ride: None,
                            },
                        ));
                    }
                }
                let mut o = best.map(|(_, o)| o)?;
                // Out to the pit on a line's train (M10b slice DX, fourth part), where the ride's
                // minutes and its fare, at the household's own cost of the fuel it is paid in,
                // come to less than the walk; home on foot.
                let walk_to = |at: (f32, f32)| {
                    reach
                        .seconds_to(cell_of(map, at))
                        .map(|s| f64::from(s) / 60.0)
                };
                if let Some((legs, ride_min, fuel, fare_kg)) =
                    Self::ride_legs(ctx, hh.id, hh.home, hearth, o.at, &walk_to)
                {
                    let fare_min = if fare_kg > 0.0 {
                        self.own_costs_of(ctx, &hh)
                            .get(fuel)
                            .copied()
                            .flatten()
                            .map_or(f64::INFINITY, |h| h * fare_kg * 60.0)
                    } else {
                        0.0
                    };
                    if ride_min + fare_min < o.walk_min {
                        o.walk_min = (ride_min + o.walk_min) / 2.0;
                        o.by_ride = Some(legs);
                    }
                }
                Some(o)
            }
        };
        let find_patch = |def: usize| -> Option<PatchOption> {
            let a = &catalog.activities[def];
            if a.digs.is_some() {
                return dig(def);
            }
            let r = a.resource?;
            let res = land_params.resources.get(r)?;
            let good = catalog.goods.get(res.good)?;
            let home = home?;
            let hours = f64::from(a.max_minutes) / 60.0;
            let prior = priors.get(def)?;
            let prior_yield = &prior.per_patch;
            let (place_kinds, seen_most, memory) = gathering();
            // No place can be expected to give more than the best expected anywhere or the best
            // seen anywhere, as a belief lies between the two (with a margin for rounding). The
            // places come nearest first, so once even that, so far off, would not beat the best
            // found, nothing further can, and the search stops: the same place is found sooner.
            let most = (f64::from(prior.most) / hours.max(1e-6))
                .max(seen_most.get(r).copied().unwrap_or(0.0))
                .max(1e-6)
                * (1.0 + 1e-9);
            let most_kg_per_hour = most * res.unit_kg;
            // Days of the household's need it already holds of this good.
            let held = stores.get(res.good).copied().unwrap_or(0.0).max(0.0);
            let stored_days = match good.purpose {
                GoodUse::Food => held * good.kcal_per_kg / kcal_day.max(1.0),
                GoodUse::Fuel => {
                    held * crate::params::fuel_ratio(goods, res.good) / household_fuel_day.max(1e-6)
                }
                GoodUse::Material | GoodUse::Tool | GoodUse::Store => 0.0,
            };
            // A material is worth bringing only toward what the household is building, or the
            // tools it lacks.
            let (need_kg, urgency) = match good.purpose {
                GoodUse::Material | GoodUse::Tool | GoodUse::Store => (
                    build_need.get(res.good).copied().unwrap_or(0.0),
                    build_urgency,
                ),
                GoodUse::Food | GoodUse::Fuel => (0.0, 0.0),
            };
            let tool_need_kg = tool_material.get(res.good).copied().unwrap_or(0.0);
            let tool_blocked = tool_material_blocked
                .get(res.good)
                .copied()
                .unwrap_or(false);
            let kind = place_kinds.get(def).copied().flatten()?;
            let mut best: Option<(f64, PatchOption)> = None;
            for &(block, secs) in home.places.get(kind)? {
                let walk = f64::from(secs) / 60.0;
                if walk > f64::from(a.max_walk_minutes) {
                    break;
                }
                if best.as_ref().is_some_and(|(bv, _)| {
                    most_kg_per_hour * hours / (hours + 2.0 * walk / 60.0) <= *bv
                }) {
                    break;
                }
                let Some((patch, cell)) =
                    places[kind].blocks.get(block as usize).copied().flatten()
                else {
                    continue;
                };
                let prior = prior_yield
                    .get(patch as usize)
                    .map_or(0.0, |&y| f64::from(y) / hours.max(1e-6));
                // What they have seen there, weighed against what they would expect.
                let rate = match memory.get(&(r as u16, patch)) {
                    Some(k) => k.belief(prior, today, res.renewal_days()),
                    None => prior,
                };
                if rate <= 0.0 {
                    continue;
                }
                let kg_per_hour = rate * res.unit_kg;
                let value = kg_per_hour * hours / (hours + 2.0 * walk / 60.0)
                    * worth(crate::uses::Place::Patch(patch));
                if best.is_none_or(|(bv, _)| value > bv) {
                    best = Some((
                        value,
                        PatchOption {
                            patch,
                            walk_min: walk,
                            kg_per_hour,
                            purpose: good.purpose,
                            kcal_per_kg: good.kcal_per_kg,
                            fuel_ratio: heat_of(res.good),
                            stored_days,
                            need_kg,
                            urgency,
                            tool_need_kg,
                            tool_blocked,
                            at: cell_centre(map, cell as usize),
                            deposit: None,
                            by_ride: None,
                        },
                    ));
                }
            }
            best.map(|(_, o)| o)
        };
        let best_patch = |def: usize| -> Option<PatchOption> {
            match hh_view.patches.get(def) {
                Some(cell) => *cell.get_or_init(|| find_patch(def)),
                None => find_patch(def),
            }
        };
        let reach = home.map(|f| &f.reach);
        let find_field = |def: usize| -> Result<FieldOption, Reason> {
            let a = &catalog.activities[def];
            let (Some(task), Some(view)) = (a.task, farm_view.as_ref()) else {
                return Err(Reason::NoPlace);
            };
            let walk = |f: &Field| {
                let cell = cell_of(map, f.rect.centre_m());
                reach?.seconds_to(cell).map(|s| f64::from(s) / 60.0)
            };
            // Rain, snow or frost keep people off the ground today (ADR-0012 §5).
            let weather = |r: &civ_land::RectCm| {
                ctx.land
                    .unworkable_at(ctx.land_params, map, r.centre_m())
                    .map(unworkable_reason)
            };
            view.best(
                task,
                f64::from(a.max_minutes) / 60.0,
                site.as_ref(),
                &walk,
                &weather,
            )
        };
        let best_field = |def: usize| -> Result<FieldOption, Reason> {
            match hh_view.fields.get(def) {
                Some(cell) => *cell.get_or_init(|| find_field(def)),
                None => find_field(def),
            }
        };
        // Whom to ask, what to buy and where to work are each worked out only if an option
        // needs them (they cost a search of the settlement), and then once.
        let giver = || {
            *hh_view
                .giver
                .get_or_init(|| self.best_giver(ctx, &hh, field_key, kcal_day, stock))
        };
        // A purchase that costs the household fewer hours than getting the good itself.
        let trade = || {
            *hh_view
                .trade
                .get_or_init(|| self.best_purchase(ctx, &hh, &stores, reach))
        };
        // One in another settlement, by a report (M5b slice AP, ADR-0019 §2). One of a household
        // goes at a time: what another is on the way to buy is not home yet, and two would walk
        // for the same thing.
        let fetch = || {
            let another = hh
                .members
                .iter()
                .filter(|&&m| Some(m) != me)
                .filter_map(|&m| self.person(m))
                .any(|q| {
                    catalog
                        .activities
                        .get(q.act.def as usize)
                        .is_some_and(|a| a.behavior == Behavior::Fetch)
                });
            if self.stop_trade_between {
                return Err(Reason::Stopped);
            }
            if another {
                return Err(Reason::AnotherGoes);
            }
            // For its own need first; else an errand, to sell at home (M5b slice AQ).
            let trip = hh_view
                .fetch
                .get_or_init(|| {
                    self.best_fetch(ctx, &hh, &stores, reach)
                        .or_else(|| self.errand_trip(ctx, &hh, reach))
                })
                .ok_or(Reason::NoReport)?;
            // Not to doors an embargo they keep forbids (M7c slice BV).
            if me.is_some_and(|m| self.keeps_embargo(ctx, m, hh.id, trip.seller)) {
                return Err(Reason::Embargo);
            }
            Ok(trip)
        };
        // Paid work at a workshop of another household (slice J).
        let session_min = catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Hire)
            .map(|a| f64::from(a.max_minutes))
            .fold(0.0, f64::max);
        let job = || {
            *hh_view.job.get_or_init(|| {
                if session_min > 0.0 {
                    self.best_job(ctx, &hh, &stores, reach, session_min)
                } else {
                    None
                }
            })
        };
        let build = plan.as_ref().map_err(|why| *why).and_then(|p| {
            let def = &catalog.buildings[p.def];
            let held = p.work.held_by_slot(def, &stores);
            let at = build::centre_m(&p.spec);
            let at_home = (at.0 - hh.home.0).abs() < 1.0 && (at.1 - hh.home.1).abs() < 1.0;
            let walk_min = if at_home {
                0.0
            } else {
                let secs = reach.and_then(|r| r.seconds_to(cell_of(map, at)));
                f64::from(secs.ok_or(Reason::Unreachable)?) / 60.0
            };
            Ok(BuildOption {
                building: p.building,
                at,
                walk_min,
                workable_h: p.work.workable_h(&held),
                left_h: p
                    .work
                    .needs()
                    .map_or(0.0, |s| (s.labour_h - p.work.done_h).max(0.0)),
                urgency: build_urgency,
            })
        });
        // Recipes: how much the person could make at home this session, and what it is worth.
        let person = self.people.get(h);
        let capacity = facts.capacity;
        let ready_target = params.household.ready_food_days.max(1e-6);
        let processed_target = params.household.processed_food_days.max(1e-6);
        let best_make = |def: usize,
                         _blocked: &[usize]|
         -> Result<MakeOption, (Reason, Option<usize>)> {
            let a = &catalog.activities[def];
            let r = a
                .recipe
                .and_then(|r| catalog.recipes.get(r))
                .ok_or((Reason::NoPlace, None))?;
            let &(out, _) = r.outputs.first().ok_or((Reason::NotNeeded, None))?;
            let out_good = goods.get(out).ok_or((Reason::NotNeeded, None))?;
            if trade().is_some_and(|t| t.good == out) {
                return Err((Reason::Cheaper, None));
            }
            let level = match (r.skill, person) {
                (Some(k), Some(p)) => p.skill(k),
                _ => 0.0,
            };
            let speed = r
                .skill
                .and_then(|k| catalog.skills.get(k))
                .map_or(1.0, |k| interpolate(&k.speed, level))
                * capacity
                * a.rate;
            if speed <= 0.0 {
                return Err((Reason::TooYoung, None));
            }
            // Coin is struck only by a mint's keeper, from the metal brought to it (M7d slice
            // CE2b): the polity's store holds the metal and fuel, the duty is worth the office's
            // points.
            if out_good.money.as_ref().is_some_and(|m| m.coin) {
                let Some((_, _, kg)) = person
                    .and_then(|p| self.mint_work(ctx, p.id))
                    .filter(|w| a.recipe == Some(w.1.recipe))
                else {
                    return Err((Reason::NotNeeded, None));
                };
                if let Some(&t) = r
                    .tools
                    .iter()
                    .find(|&&t| free_tools.get(t).copied().unwrap_or(0.0) < decide::MIN_TOOL)
                {
                    return Err((Reason::NoTool, Some(t)));
                }
                let per = r.inputs.first().map_or(1.0, |i| i.1).max(1e-9);
                let units = (kg / per).min(make::units_in(r, f64::from(a.max_minutes), speed));
                if units.is_nan() || units <= 1e-6 {
                    return Err((Reason::NoInputs, None));
                }
                return Ok(MakeOption {
                    units,
                    minutes: make::minutes_for(r, units, speed),
                    worth: MakeWorth::Office {
                        points: params.market.strike_points,
                    },
                    firm: None,
                    site: None,
                });
            }
            let have = |g: usize| make::available(&stores, goods, g, reserve_ok, protected);
            // Matter to vaccinate with is made only for the calls to give it that stand at the
            // household's door (M10c slice ED, third part): as much as they are short of, owed as
            // the care they ask is.
            if let Some((dose_kg, needed)) = self.vaccine_wanted(ctx, hh_id, out) {
                let short_kg = needed as f64 * dose_kg - held_of(out);
                let per = r.outputs.first().map_or(0.0, |o| o.1);
                if short_kg <= 1e-9 || per <= 0.0 {
                    return Err((Reason::NotNeeded, None));
                }
                if let Some(&t) = r
                    .tools
                    .iter()
                    .find(|&&t| free_tools.get(t).copied().unwrap_or(0.0) < decide::MIN_TOOL)
                {
                    return Err((Reason::NoTool, Some(t)));
                }
                let units = (short_kg / per)
                    .min(make::units_from_inputs(r, &have))
                    .min(make::units_in(r, f64::from(a.max_minutes), speed));
                if units.is_nan() || units <= 1e-6 {
                    return Err((Reason::NoInputs, None));
                }
                return Ok(MakeOption {
                    units,
                    minutes: make::minutes_for(r, units, speed),
                    worth: MakeWorth::Called {
                        owed: short_kg / dose_kg.max(1e-12),
                    },
                    firm: None,
                    site: None,
                });
            }
            // A batch runs unattended (M10a slice DP, ADR-0033 §2): while one of this recipe is
            // under way there is nothing to do; once its days are past, a session opens it, worth
            // what making its units would be.
            let mut waiting = 0.0;
            if r.elapsed_days > 0.0 {
                let today = ctx.now.day_index();
                // Its workshop's batches of it as well as its own (M10a slice DQ).
                let shop = self
                    .workshop_of(hh_id, out)
                    .and_then(|f| self.firm(f))
                    .map(|f| f.batches.as_slice())
                    .unwrap_or_default();
                for b in hh
                    .batches
                    .iter()
                    .chain(shop)
                    .filter(|b| a.recipe == Some(b.recipe))
                {
                    if b.ends_day > today {
                        return Err((Reason::Burning, None));
                    }
                    waiting += b.units;
                }
            }
            let mut from_inputs = make::units_from_inputs(r, &have) + waiting;
            // What making it is worth, and how much of it is wanted.
            let (worth, wanted) = if let Some(_tool) = &out_good.tool {
                // Its need is the need of the tool it serves for, made the way the household
                // makes that one (M7d slice CE).
                let served = make::served(goods, out);
                let need = tool_need.get(served).copied().unwrap_or(0.0);
                let sale = for_sale.get(out).copied().unwrap_or(0.0);
                let way = ways
                    .get(served)
                    .copied()
                    .flatten()
                    .and_then(|w| catalog.recipes.get(w));
                if need > 0.0 && way.is_some_and(|w| std::ptr::eq(w, r)) {
                    (MakeWorth::Tool { tool: served, need }, 1.0)
                } else if need > 0.0 && sale <= 0.0 {
                    return Err((Reason::BetterWay, None));
                } else if sale > 0.0 {
                    (MakeWorth::Sale { share: sale }, 1.0)
                } else {
                    return Err((Reason::NotNeeded, None));
                }
            } else if let Some(input) = make::preserves(r, goods) {
                // Keeping food that would spoil before it is eaten (drying, smoking): as much of
                // it as would spoil, worth the food kept.
                let spoil = crate::knowledge::spoiling_kcal(
                    goods,
                    &stores,
                    hh.sheltered,
                    kcal_day,
                    &[input],
                );
                let per_unit = r
                    .inputs
                    .iter()
                    .find(|&&(g, _)| g == input)
                    .map_or(0.0, |&(_, amount)| amount * goods[input].kcal_per_kg);
                if per_unit <= 0.0
                    || spoil < MIN_PRESERVE_DAYS * params.household.daily_kcal_per_person
                {
                    return Err((Reason::NotNeeded, None));
                }
                (MakeWorth::Preserve { kcal: per_unit }, spoil / per_unit)
            } else if out_good.store.is_some() {
                // Pots: the food the next would keep, as long as more would keep some (M3b
                // slice Q).
                let (want, kcal) = pots.get(out).copied().unwrap_or((0.0, 0.0));
                let made = r
                    .outputs
                    .iter()
                    .find(|&&(g, _)| g == out)
                    .map_or(0.0, |&(_, amount)| amount);
                if want <= 0.0
                    || made <= 0.0
                    || kcal < MIN_PRESERVE_DAYS * params.household.daily_kcal_per_person
                {
                    return Err((Reason::NotNeeded, None));
                }
                (MakeWorth::Preserve { kcal: kcal * made }, want / made)
            } else if let Some((tool, need)) = made_for.get(out).copied().flatten() {
                // A material it makes for a tool it needs (copper for a copper axe): as much as
                // that way is short of.
                let per_unit = r
                    .outputs
                    .iter()
                    .find(|&&(g, _)| g == out)
                    .map_or(0.0, |&(_, amount)| amount);
                let short = tool_material.get(out).copied().unwrap_or(0.0);
                if per_unit <= 0.0 || short <= 0.0 {
                    return Err((Reason::NotNeeded, None));
                }
                (MakeWorth::Tool { tool, need }, short / per_unit)
            } else if out_good.purpose == GoodUse::Food {
                let per_unit: f64 = r
                    .outputs
                    .iter()
                    .filter_map(|&(g, amount)| goods.get(g).map(|d| amount * d.kcal_per_kg))
                    .sum();
                if per_unit <= 0.0 {
                    return Err((Reason::NotNeeded, None));
                }
                // Ready food answers for itself; food a step from ready (flour) is wanted only
                // as far as ready food and what is already a step from it fall short.
                let ready_days = food_all / kcal_day.max(1.0);
                let (pool_days, target) = if out_good.edible() {
                    (ready_days, ready_target)
                } else {
                    (
                        ready_days + held_of(out) * out_good.kcal_per_kg / kcal_day.max(1.0),
                        processed_target.max(ready_target),
                    )
                };
                let short = (1.0 - pool_days / target).clamp(0.0, 1.0);
                let room = target / (target + pool_days.max(0.0));
                // Enough to bring what it makes up to its days and no more: what is made ahead
                // of need spoils (bread in days).
                let want_days = target - pool_days;
                if want_days < MIN_BATCH_DAYS {
                    return Err((Reason::NotNeeded, None));
                }
                let want_kcal = want_days * kcal_day.max(1.0);
                let toward_meal = out_good.edible() || held_of(out) * out_good.kcal_per_kg < 1.0;
                (
                    MakeWorth::Food {
                        kcal: 0.0,
                        short,
                        room,
                        toward_meal,
                    },
                    want_kcal / per_unit,
                )
            } else {
                return Err((Reason::NotNeeded, None));
            };
            // Made for sale, its workshop's goods in progress (what another of its lines made) go
            // in before the household's, and its stations do the work they can (M10a slice DQ).
            let shop = match worth {
                MakeWorth::Sale { .. } => self.workshop_of(hh_id, out),
                _ => None,
            };
            if let Some(f) = shop
                .and_then(|f| self.firm(f))
                .filter(|f| f.lines.len() > 1)
            {
                let with = |g: usize| have(g) + f.in_progress(g, out);
                from_inputs = make::units_from_inputs(r, &with) + waiting;
            }
            if from_inputs <= 1e-6 {
                return Err((Reason::NoInputs, None));
            }
            // A machine that wants a crew is worked only at a works, whose bell brings its hands
            // together (M10a slice DT; ADR-0033 §8).
            if shop.is_none() && crew_of(goods, r) > 1 {
                return Err((Reason::NoCrew, None));
            }
            if let Some(&t) = r.tools.iter().find(|&&t| {
                free_tools.get(t).copied().unwrap_or(0.0) < decide::MIN_TOOL
                    && shop.is_none_or(|f| self.station_for(goods, f, t) < decide::MIN_TOOL)
            }) {
                return Err((Reason::NoTool, Some(t)));
            }
            let by_time = make::units_in(r, f64::from(a.max_minutes), speed);
            let units = from_inputs.min(wanted).min(by_time);
            if units.is_nan() || units <= 1e-6 {
                return Err((Reason::NoInputs, None));
            }
            let worth = match worth {
                MakeWorth::Food {
                    short,
                    room,
                    toward_meal,
                    ..
                } => MakeWorth::Food {
                    kcal: units
                        * r.outputs
                            .iter()
                            .filter_map(|&(g, amount)| goods.get(g).map(|d| amount * d.kcal_per_kg))
                            .sum::<f64>(),
                    short,
                    room,
                    toward_meal,
                },
                // The food kept: the units made of what would have spoiled.
                MakeWorth::Preserve { kcal: per_unit } => MakeWorth::Preserve {
                    kcal: units * per_unit,
                },
                tool => tool,
            };
            let firm = match worth {
                MakeWorth::Sale { .. } => self.workshop_of(hh_id, out),
                _ => None,
            };
            // Its workshop works in its own building once that has its roof on.
            let site = match firm.and_then(|f| self.firm_site(ctx.land, ctx.catalog, f)) {
                Some(at) if at != hh.home => {
                    let secs = reach
                        .and_then(|r| r.seconds_to(cell_of(map, at)))
                        .ok_or((Reason::Unreachable, None))?;
                    Some((at, f64::from(secs) / 60.0))
                }
                _ => None,
            };
            Ok(MakeOption {
                units,
                minutes: make::minutes_for(r, units, speed),
                worth,
                firm,
                site,
            })
        };
        let makes_tool = |def: usize| {
            catalog.activities[def]
                .recipe
                .and_then(|r| catalog.recipes.get(r))
                .and_then(|r| r.outputs.first())
                .and_then(|&(g, _)| goods.get(g))
                .is_some_and(|g| g.tool.is_some())
        };
        let shop = decide::Workshop {
            free_tools: &free_tools,
            held_tools: &held_tools,
            best_make: &best_make,
            makes_tool: &makes_tool,
            knows: &|t| self.people.get(h).is_some_and(|p| p.knows(t)),
        };
        // Trying toward a technique that would answer a problem at home (ADR-0008 §3): the one
        // whose problem would cost the household the most food, among those the person could
        // find.
        let try_gap_min = (params.knowledge.try_gap_days * MINUTES_PER_DAY as f64) as i64;
        let trying = || -> Result<decide::TryOption, Reason> {
            let p = self.people.get(h).ok_or(Reason::NoProblem)?;
            if p.tried
                .is_some_and(|t| now.minutes() - t.minutes() < try_gap_min)
            {
                return Err(Reason::TriedLately);
            }
            let half = (params.decision.trip_half_worth_days * kcal_day).max(1.0);
            let knows = |t: usize| p.knows(t);
            let mut best: Option<(f64, usize)> = None;
            for (t, def) in catalog.techniques.iter().enumerate() {
                if def.answers_spoilage.is_empty()
                    || p.knows(t)
                    || !crate::knowledge::could_find(catalog, t, &knows, &stores)
                {
                    continue;
                }
                let kcal = crate::knowledge::spoiling_kcal(
                    goods,
                    &stores,
                    hh.sheltered,
                    kcal_day,
                    &def.answers_spoilage,
                );
                let share = kcal / (kcal + half);
                if best.is_none_or(|(b, _)| share > b) {
                    best = Some((share, t));
                }
            }
            let (share, t) = best
                .filter(|&(share, _)| share >= crate::knowledge::MIN_PROBLEM_SHARE)
                .ok_or(Reason::NoProblem)?;
            Ok(decide::TryOption {
                technique: t as u16,
                points: params.knowledge.w_try * share,
            })
        };
        // Taking from another household's store (M4b slice AA, ADR-0015 §2): the person's moral
        // filter first, then the household's target.
        let take = || -> Result<decide::TakeOption, Reason> {
            let p = self.people.get(h).ok_or(Reason::WouldNotTake)?;
            if f64::from(p.objection) >= params.crime.objection_filter {
                return Err(Reason::WouldNotTake);
            }
            if Population::turned_back_lately(p, now) {
                return Err(Reason::TurnedBackLately);
            }
            let target = *hh_view
                .take
                .get_or_init(|| self.take_target(ctx, &hh, field_key, kcal_day, stock));
            Population::take_option(p, target, params)
        };
        // Another settlement's hearth (M5a slice AM): those the household knows within a day's
        // walk there and back.
        let visit_max_min = catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Visit)
            .map(|a| f64::from(a.max_walk_minutes))
            .fold(0.0, f64::max);
        let visits = || match me {
            Some(me) => self.visit_options(ctx, me, hh_id, field_key, visit_max_min, dark),
            None => Vec::new(),
        };
        let (cands, excluded) = decide::candidates(
            &catalog.activities,
            &params.decision,
            &facts,
            &limits,
            &best_patch,
            &best_field,
            water,
            &giver,
            &trade,
            &fetch,
            &job,
            build,
            &shop,
            &trying,
            &take,
            &visits,
        );
        if ctx.approx.household_view {
            self.views.insert(hh_id, hh_view.kept());
        }
        // Work that needs a technique the person does not know is left out, unless a member of
        // their household who knows it is at that work there now: then they may work beside them
        // and learn it (ADR-0008 §4).
        let (mut cands, mut excluded) = (cands, excluded);
        if let Some(me) = self.people.get(h) {
            let mut i = 0;
            while i < cands.len() {
                let (def, target) = (cands[i].scored.def, cands[i].scored.target);
                let gate = catalog
                    .activities
                    .get(usize::from(def))
                    .and_then(|a| self.technique_for(ctx, a, target, me.household))
                    .map_or(Gate::Open, |t| self.gate(ctx, me, t, def, target));
                match gate {
                    Gate::Open => i += 1,
                    Gate::Learner(_) => {
                        decide::add_term(&mut cands[i], Reason::Learning, params.knowledge.w_learn);
                        i += 1;
                    }
                    Gate::Closed => {
                        cands.remove(i);
                        excluded.push((def, Reason::DoesNotKnow));
                    }
                }
            }
        }
        // A curfew in force where they live (M4b slice AD): being away from home in its hours
        // costs one who knows of it what keeping it weighs with them. The watch at its rounds,
        // those at a gathering and those carrying water to a fire are exempt.
        let curfew = self
            .people
            .get(h)
            .map(|p| p.id)
            .and_then(|id| self.curfew_now(ctx, id, &hh, minute));
        let covered = |c: &decide::Candidate| {
            let exempt = catalog
                .activities
                .get(usize::from(c.scored.def))
                .is_some_and(|a| {
                    matches!(
                        a.behavior,
                        Behavior::Watch
                            | Behavior::Light
                            | Behavior::Attend
                            | Behavior::Douse
                            | Behavior::Strip
                            | Behavior::Pull
                            | Behavior::Salvage
                    )
                });
            !exempt && self::polity::away_from_home(&c.steps, pos, hh.home)
        };
        if let Some((_, _, Some(points))) = curfew {
            for c in &mut cands {
                if covered(c) {
                    decide::add_term(c, Reason::Curfew, -points);
                }
            }
        }
        // A law keeping the sick apart where they live (M10c slice EE, first part): going out of
        // a home kept apart, or tending at another one, costs one who knows of it what keeping it
        // weighs with them. The watch, the lamplighter and those at a fire are exempt.
        let apart = self
            .people
            .get(h)
            .map(|p| p.id)
            .and_then(|id| self.keep_apart_now(ctx, id, &hh));
        let today = ctx.now.day_index();
        // The households their tending options would take them to that are kept apart.
        let apart_targets: Vec<PermanentId> = match apart {
            Some(k) if !k.home => cands
                .iter()
                .filter_map(|c| match c.scored.target {
                    Target::Household(x) => Some(x),
                    _ => None,
                })
                .filter(|&x| {
                    self.household(x)
                        .is_some_and(|y| self.kept_apart(y, &k.def, today))
                })
                .collect(),
            _ => Vec::new(),
        };
        let kept_from = |c: &decide::Candidate| {
            let Some(k) = apart else {
                return false;
            };
            let behavior = catalog
                .activities
                .get(usize::from(c.scored.def))
                .map(|a| a.behavior);
            if k.home {
                let exempt = behavior.is_some_and(|b| {
                    matches!(
                        b,
                        Behavior::Watch
                            | Behavior::Light
                            | Behavior::Douse
                            | Behavior::Strip
                            | Behavior::Pull
                            | Behavior::Salvage
                    )
                });
                return !exempt && self::polity::away_from_home(&c.steps, pos, hh.home);
            }
            match c.scored.target {
                Target::Household(x) => {
                    behavior == Some(Behavior::Tend) && apart_targets.contains(&x)
                }
                _ => false,
            }
        };
        if let Some(points) = apart.and_then(|k| k.points) {
            for c in &mut cands {
                if kept_from(c) {
                    decide::add_term(c, Reason::KeepApart, -points);
                }
            }
        }
        // A household-resolved day's planning takes the options as scored and stops here
        // (ADR-0032 §2): nothing is chosen, drawn or begun.
        if self.planning {
            self.planned = Some(resolution::Planned {
                cands,
                site,
                meal_min: limits.meal_min,
            });
            return;
        }
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        if cands.is_empty() {
            // Nothing can be done: wait and look again.
            self.wait(ctx, h, WAIT_AFTER_FAILURE_MIN);
            return;
        }
        // A leisure block that ended by the person choosing something else (ADR-0011 §4): what
        // they choose now is something else, as in Detailed mode.
        if let Some((version, def)) = self.switched.remove(&p.id)
            && version == p.act.version
            && cands.iter().any(|c| c.scored.def != def)
        {
            cands.retain(|c| c.scored.def != def);
        }
        let totals: Vec<f32> = cands.iter().map(|c| c.scored.total).collect();
        let u = draw(ctx.seed, p);
        let (choice, probability, temperature) = decide::choose(&totals, &params.decision, u);
        // Leisure chosen in Accelerated mode lasts the run of sessions Detailed mode would live
        // before choosing something else, up to the next turn of the day (ADR-0011 §4).
        let block = match (
            leisure_until_min,
            catalog
                .activities
                .get(usize::from(cands[choice].scored.def)),
        ) {
            (Some(until), Some(a))
                if matches!(
                    a.behavior,
                    Behavior::Rest | Behavior::Play | Behavior::Socialize
                ) =>
            {
                let u = draw(ctx.seed, p);
                Some(decide::leisure_block(
                    a.min_minutes,
                    a.max_minutes,
                    until,
                    probability,
                    u,
                ))
            }
            _ => None,
        };
        // A choice that lays claim to something (new ground, a building or a workshop begun,
        // goods to buy or ask for, paid work) is the household's consequential step: its other
        // members see their options afresh (ADR-0011 §4).
        let claims = matches!(
            cands[choice].scored.target,
            Target::NewField | Target::NewBuilding | Target::NewFirm
        ) || catalog
            .activities
            .get(usize::from(cands[choice].scored.def))
            .is_some_and(|a| {
                matches!(
                    a.behavior,
                    Behavior::Trade
                        | Behavior::Fetch
                        | Behavior::Ask
                        | Behavior::Hire
                        | Behavior::Take
                )
            });
        if claims {
            self.changed(hh_id);
        }
        let mut order: Vec<usize> = (0..cands.len()).filter(|&i| i != choice).collect();
        order.sort_by(|&a, &b| cands[b].scored.total.total_cmp(&cands[a].scored.total));
        let receipt = Receipt {
            at: now,
            chosen: cands[choice].scored.clone(),
            runner_up: order.first().map(|&i| cands[i].scored.clone()),
            others: order
                .iter()
                .skip(1)
                .take(2)
                .map(|&i| (cands[i].scored.def, cands[i].scored.total))
                .collect(),
            excluded: excluded.into_iter().take(4).collect(),
            probability: probability as f32,
            temperature: temperature as f32,
            needs: [
                facts.hunger as f32,
                facts.sleep_drive as f32,
                facts.loneliness as f32,
                facts.food_days as f32,
                facts.water_days as f32,
            ],
        };
        // One who goes out of a home kept apart, or tends at one, breaks the law that keeps the
        // sick apart, knowing it or not (M10c slice EE, first part).
        if let Some(k) = apart
            && kept_from(&cands[choice])
            && let Some(law) = self
                .polities
                .get_mut(k.pi)
                .and_then(|x| x.laws.get_mut(k.li))
        {
            if k.points.is_some() {
                law.compliance.broken += 1;
            } else {
                law.compliance.broken_unaware += 1;
            }
        }
        // One who goes out in a curfew's hours breaks it, knowing it or not.
        if let Some((pi, li, points)) = curfew
            && covered(&cands[choice])
            && let Some(law) = self.polities.get_mut(pi).and_then(|x| x.laws.get_mut(li))
        {
            if points.is_some() {
                law.compliance.broken += 1;
            } else {
                law.compliance.broken_unaware += 1;
            }
        }
        self.take_up(ctx, h, &cands[choice], site, &hh, block, Some(receipt));
        self.run_steps(ctx, h, depth + 1);
    }

    /// Takes up `chosen` (shared by a decision and a household-resolved day, ADR-0032 §2): new
    /// ground marked out, a workshop set up, paid work held, a home begun, the activity set with
    /// its receipt, a round of the watch counted and the crews joined.
    #[allow(clippy::too_many_arguments)]
    fn take_up(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        chosen: &decide::Candidate,
        site: Option<Site>,
        hh: &Household,
        block: Option<(u32, bool)>,
        mut receipt: Option<Receipt>,
    ) {
        let (now, params, catalog, hh_id) = (ctx.now, ctx.params, ctx.catalog, hh.id);
        let mut target = chosen.scored.target;
        if target == Target::NewField {
            // New ground is marked out when someone sets off to break it.
            match site {
                Some(site) => {
                    let id = ctx.ids.allocate();
                    // Who holds new ground is the regime's rule (ADR-0007 §2).
                    let holder = match (ctx.regime.holder, hh.settlement) {
                        (LandHolder::Settlement, Some(s)) => Party::Settlement(s),
                        _ => Party::Household(hh_id),
                    };
                    ctx.land.fields.push(Field {
                        id,
                        household: hh_id,
                        holder,
                        lease: None,
                        rect: site.rect,
                        crop: params.farm.crop as u16,
                        stage: FieldStage::Fallow,
                        stage_since: now,
                        work_h: 0.0,
                        tended_h: 0.0,
                        ground: site.ground as f32,
                        clear_h_per_ha: site.clear_h_per_ha as f32,
                        broken: false,
                        sown_day: 0,
                        sheaves_kg: 0.0,
                        harvests: 0,
                        water_mm: 0.0,
                        need_mm: 0.0,
                        got_mm: 0.0,
                        last_flood_depth_cm: 0,
                        submerged_days: 0,
                        reserved: None,
                        // Native ground's soil (ADR-0012 §3), of the ground the field keeps.
                        soil: FieldSoil::native(
                            &ctx.land_params.soil,
                            f64::from(site.ground as f32),
                        ),
                    });
                    self.sites.remove(&hh_id);
                    target = Target::Field(id);
                }
                None => target = Target::None,
            }
            if let Some(r) = receipt.as_mut() {
                r.chosen.target = target;
            }
        }
        if target == Target::NewFirm {
            // A workshop is set up when someone sets to work making what it sells.
            let out = catalog
                .activities
                .get(usize::from(chosen.scored.def))
                .and_then(|a| a.recipe)
                .and_then(|r| catalog.recipes.get(r))
                .and_then(|r| r.outputs.first())
                .map(|&(g, _)| g);
            let founder = self.people.get(h).map(|p| p.id);
            target = match (out, founder) {
                (Some(g), Some(founder)) => self
                    .found_firm(ctx, hh_id, founder, g)
                    .map_or(Target::Home, Target::Firm),
                _ => Target::Home,
            };
            if let Some(r) = receipt.as_mut() {
                r.chosen.target = target;
            }
        }
        // Paid work taken is held for the person (slice J).
        if let (Target::Firm(firm), Some(Behavior::Hire)) = (
            target,
            catalog
                .activities
                .get(usize::from(chosen.scored.def))
                .map(|a| a.behavior),
        ) {
            let minutes: u32 = chosen
                .steps
                .iter()
                .map(|s| match s {
                    Step::Work { minutes } => *minutes,
                    _ => 0,
                })
                .sum();
            self.take_work(firm, f64::from(minutes));
        }
        let mut steps = chosen.steps.clone();
        if let Some((minutes, _)) = block
            && let Some(Step::Work { minutes: m }) = steps
                .iter_mut()
                .rev()
                .find(|s| matches!(s, Step::Work { .. }))
        {
            *m = minutes;
        }
        if target == Target::NewBuilding {
            // Ground for a home is claimed when someone sets to work on it; the household's home
            // moves there.
            target = self
                .begin_home(ctx, hh_id)
                .map_or(Target::None, Target::Building);
            if let Some(r) = receipt.as_mut() {
                r.chosen.target = target;
            }
            if let Some(new_home) = self.household(hh_id).map(|x| x.home) {
                for s in &mut steps {
                    if let Step::Walk { to } = s
                        && *to == hh.home
                    {
                        *to = new_home;
                    }
                }
            }
        }
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        p.act = Activity {
            def: chosen.scored.def,
            target,
            steps,
            step: 0,
            started: now,
            step_started: now,
            step_ends: now,
            version: p.act.version.wrapping_add(1),
        };
        if let Some(r) = receipt {
            p.push_receipt(r);
        }
        if let Some((_, true)) = block {
            let (id, version, def) = (p.id, p.act.version, p.act.def);
            self.switched.insert(id, (version, def));
        }
        // A round of the watch is counted as it begins (M4b slice AC).
        let (who, def) = (p.id, p.act.def);
        if catalog
            .activities
            .get(usize::from(def))
            .is_some_and(|a| a.behavior == Behavior::Watch)
        {
            self.start_round(ctx, who);
        }
        // And the health officer's (M10c slice EE, second part).
        if catalog
            .activities
            .get(usize::from(def))
            .is_some_and(|a| a.behavior == Behavior::Rounds)
        {
            self.start_rounds(ctx, who);
        }
        // Someone who sets off to work on a crossing is one of its crew that day, counted as
        // they choose it, so that those choosing after them see the room taken (M5c slice AW;
        // research 11-11 §1.3: progress is limited by the usable work front).
        if let Target::Crossing(c) = target
            && let Some(x) = ctx.land.crossings.list.iter_mut().find(|x| x.id == c)
        {
            let day = now.day_index();
            if x.crew.0 != day {
                x.crew = (day, 0);
            }
            x.crew.1 = x.crew.1.saturating_add(1);
        }
        // And so is someone who sets off to work on a boat (M8b slice CO, third part).
        if let Target::Boat(b) = target {
            Self::joined_boat_crew(ctx, b);
        }
        // And so is someone who sets off to work on a well (M6a slice AY, step three).
        if let Target::Well(w) = target
            && catalog
                .activities
                .get(usize::from(def))
                .is_some_and(|a| a.behavior == Behavior::Well)
        {
            Self::joined_well_crew(ctx, w);
        }
    }

    /// Replaces the activity with waiting where the person stands.
    fn wait(&mut self, ctx: &mut Ctx, h: Handle<Person>, minutes: u32) {
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let def = p.act.def;
        p.act = Activity {
            def,
            target: Target::None,
            steps: vec![Step::Wait { minutes }],
            step: 0,
            started: ctx.now,
            step_started: ctx.now,
            step_ends: ctx.now,
            version: p.act.version.wrapping_add(1),
        };
        p.trip = None;
        self.start_work(ctx, h, minutes, None);
    }

    /// Starts the current step; runs instant steps; decides again after the last.
    fn run_steps(&mut self, ctx: &mut Ctx, h: Handle<Person>, depth: u32) {
        for _ in 0..8 {
            let Some(p) = self.people.get(h) else {
                return;
            };
            let Some(step) = p.act.steps.get(p.act.step as usize).copied() else {
                if depth > 2 {
                    // An activity of instant steps only: never loop at one instant.
                    self.wait(ctx, h, 1);
                } else {
                    self.decide(ctx, h, depth);
                }
                return;
            };
            match step {
                Step::Deposit => {
                    self.deposit(ctx, h);
                    if let Some(p) = self.people.get_mut(h) {
                        p.act.step += 1;
                    }
                }
                Step::Walk { mut to } => {
                    // A saved delivery aimed at a home that has since flooded returns to the
                    // same refuge as meals and sleep. The load is handed over only on arrival.
                    if (p.carrying.kg > 0.0 || p.carrying.water_l > 0.0)
                        && let Some(hh) = self.household(p.household)
                        && to == hh.home
                    {
                        to = Self::supply_at(ctx, hh);
                        if let Some(p) = self.people.get_mut(h) {
                            p.act.steps[p.act.step as usize] = Step::Walk { to };
                        }
                    }
                    // Remove a completed retry wait before retrying the same walk. Repeated
                    // blocked trips keep a bounded activity and the original deposit target.
                    if let Some(p) = self.people.get_mut(h)
                        && (p.carrying.kg > 0.0 || p.carrying.water_l > 0.0)
                        && p.act.step > 0
                        && matches!(p.act.steps.get(p.act.step as usize - 1), Some(Step::Wait { minutes }) if *minutes == WAIT_AFTER_FAILURE_MIN)
                    {
                        p.act.steps.remove(p.act.step as usize - 1);
                        p.act.step -= 1;
                    }
                    if !self.start_walk(ctx, h, to) {
                        let laden = self
                            .people
                            .get(h)
                            .is_some_and(|p| p.carrying.kg > 0.0 || p.carrying.water_l > 0.0);
                        if laden {
                            if let Some(p) = self.people.get_mut(h) {
                                p.act.steps.insert(
                                    p.act.step as usize,
                                    Step::Wait {
                                        minutes: WAIT_AFTER_FAILURE_MIN,
                                    },
                                );
                                p.trip = None;
                            }
                            self.start_work(ctx, h, WAIT_AFTER_FAILURE_MIN, None);
                        } else {
                            self.wait(ctx, h, WAIT_AFTER_FAILURE_MIN);
                        }
                    }
                    return;
                }
                Step::Work { minutes } => {
                    let behavior = ctx
                        .catalog
                        .activities
                        .get(p.act.def as usize)
                        .map(|a| a.behavior);
                    self.start_work(ctx, h, minutes, behavior);
                    return;
                }
                Step::Paddle { to } => {
                    if self.start_paddle(ctx, h, to) {
                        return;
                    }
                    // No boat here, or no way by water today: home on foot instead.
                    self.cannot_paddle(h);
                }
                Step::Ride { to } => {
                    if self.start_ride(ctx, h, to) {
                        return;
                    }
                    // No train to take soon: on foot instead, the rest as planned.
                    if let Some(p) = self.people.get_mut(h) {
                        let at = usize::from(p.act.step);
                        if at < p.act.steps.len() {
                            p.act.steps.remove(at);
                        }
                    }
                }
                Step::Haul { to } => {
                    if self.start_haul(ctx, h, to) {
                        return;
                    }
                    // No cart here, or no way for its wheels today: home on foot instead, the
                    // cart and what is in it left where they are.
                    self.cannot_paddle(h);
                }
                Step::Handle { minutes, .. } => {
                    self.start_work(ctx, h, minutes, None);
                    return;
                }
                Step::Wait { minutes } => {
                    // One with a band holding a settlement waits there asleep (M6c slice BM).
                    let band = self.people.get(h).is_some_and(|p| {
                        ctx.catalog
                            .activities
                            .get(p.act.def as usize)
                            .is_some_and(|a| a.behavior == Behavior::Band)
                    });
                    let behavior = band.then_some(Behavior::Sleep);
                    self.start_work(ctx, h, minutes, behavior);
                    return;
                }
            }
        }
        self.wait(ctx, h, 1);
    }

    fn start_walk(&mut self, ctx: &mut Ctx, h: Handle<Person>, to: (f32, f32)) -> bool {
        self.sync_routes(ctx);
        let now = ctx.now;
        let params = ctx.params;
        let cache_max = ROUTE_CACHE_MAX.max(ROUTE_CACHE_PER_PERSON * self.people.len());
        let Some(p) = self.people.get_mut(h) else {
            return false;
        };
        let from_cell = cell_of(ctx.map, p.pos);
        let to_cell = cell_of(ctx.map, to);
        let speed = interpolate(&params.walk_speed_by_age, p.age_years(now)).max(0.05);
        let wear = &ctx.land.wear;
        let (points, minutes): (Vec<(f32, f32)>, Vec<f32>) = if from_cell == to_cell {
            let d = ((to.0 - p.pos.0).powi(2) + (to.1 - p.pos.1).powi(2)).sqrt();
            let surface = params.nav.offtrail_factor
                + (1.0 - params.nav.offtrail_factor) * f64::from(wear.factor(from_cell));
            let v = params.nav.tobler_ms(0.0) * surface * speed;
            let m = (f64::from(d) / v / 60.0) as f32;
            (vec![p.pos, to], vec![0.0, m])
        } else {
            let key = (from_cell as u32, to_cell as u32);
            if let Some(known) = self.routes_old.remove(&key) {
                keep_route(
                    &mut self.routes,
                    &mut self.routes_old,
                    key,
                    known,
                    cache_max,
                );
            }
            let cached = match self.routes.get(&key) {
                Some(known) => known.clone(),
                None => {
                    // Planned on the paths as last surveyed; with no worn ground the search can
                    // assume off-trail walking everywhere and stays tight. Where people walk,
                    // landmarks bound it more tightly still (M5a slice AL).
                    // One region round all walked ground: landmarks per settlement's ground
                    // (ADR-0028 §4) left walks between settlements unbounded and were slower
                    // (M10a slice DO, measured), so the regions are the one walked area.
                    let by_region = self.landmarks.get_or_insert_with(|| {
                        let m = LANDMARK_MARGIN_CELLS;
                        let regions: Vec<CellBox> = wear
                            .walked_area()
                            .into_iter()
                            .map(|((x0, y0), (x1, y1))| {
                                (
                                    (x0.saturating_sub(m), y0.saturating_sub(m)),
                                    (x1 + m, y1 + m),
                                )
                            })
                            .collect();
                        RegionLandmarks {
                            wear_rev: wear.rev(),
                            built: vec![None; regions.len()],
                            regions,
                        }
                    });
                    let width = ctx.map.width as usize;
                    let (tx, ty) = (to_cell % width, to_cell / width);
                    let region = by_region.regions.iter().position(|&((x0, y0), (x1, y1))| {
                        (x0..=x1).contains(&tx) && (y0..=y1).contains(&ty)
                    });
                    let landmarks = region.map(|k| {
                        let area = by_region.regions[k];
                        &*by_region.built[k]
                            .get_or_insert_with(|| {
                                let trail = |c| wear.factor(c);
                                let built = ctx.nav.landmarks(&ctx.map.elevation, &trail, area);
                                (std::sync::Arc::new(built), ctx.nav.landmark_reach(area))
                            })
                            .0
                    });
                    let routed = ctx.nav.route_with(
                        &ctx.map.elevation,
                        (from_cell, to_cell),
                        &|c| wear.factor(c),
                        wear.max_factor(),
                        ROUTE_BUDGET,
                        landmarks,
                    );
                    let found = match routed {
                        RouteResult::Found(route) => Some(std::sync::Arc::new(ctx.nav.straighten(
                            &ctx.map.elevation,
                            &route,
                            &|c| wear.factor(c),
                        ))),
                        RouteResult::Unreachable | RouteResult::BudgetExhausted => None,
                    };
                    self.route_counts.1 += u64::from(self.routes.len() >= cache_max);
                    keep_route(
                        &mut self.routes,
                        &mut self.routes_old,
                        key,
                        found.clone(),
                        cache_max,
                    );
                    found
                }
            };
            let Some(route) = cached else {
                return false;
            };
            let (pts, secs) = &*route;
            let mut pts = pts.clone();
            if let Some(first) = pts.first_mut() {
                *first = p.pos;
            }
            if let Some(last) = pts.last_mut() {
                *last = to;
            }
            let mins = secs
                .iter()
                .map(|s| (f64::from(*s) / 60.0 / speed) as f32)
                .collect();
            (pts, mins)
        };
        let duration = minutes.last().copied().unwrap_or(0.0).ceil().max(1.0);
        // A crossing on the way is stepped onto, and one that gives way under them stops the
        // walk; the streams it wades are their household's to remember (M5c slice AW).
        let (who, household) = (p.id, p.household);
        let nav = ctx.nav;
        let along = civ_land::paths::cells_along_by(
            &points,
            ctx.map.cell_size_m,
            ctx.map.width,
            ctx.map.height,
            |a, b| crossings::better_corner(nav, a, b),
        );
        if !ctx.land.crossings.list.is_empty() && !self.step_on(ctx, who, &along) {
            return false;
        }
        self.wade(ctx, household, &along);
        let Some(p) = self.people.get_mut(h) else {
            return false;
        };
        // What the walk passes within sight of (ADR-0018 §4).
        let seen = (ctx.land.settlements.len() > 1).then(|| (p.household, points.clone()));
        self.next_trip += 1;
        let trip = Trip {
            id: self.next_trip,
            rev: 0,
            depart: now,
            points,
            minutes,
            ahead: None,
            afloat: false,
        };
        settle(p, now, params);
        p.burn_kcal_min = burn_rate(p, now, params, params.energy.walk_par);
        p.asleep = false;
        p.company = 0.0;
        p.trip = Some(trip);
        p.act.step_started = now;
        p.act.step_ends = now.plus_minutes(duration as i64);
        let (id, version, ends) = (p.id, p.act.version, p.act.step_ends);
        ctx.schedule_step(ends, id, version);
        self.count_trip(ctx, h, crate::travel::Mode::Walk, duration);
        self.count_going(ctx, h);
        // Its first bottleneck, if it passes one (M8b slice CN).
        if !ctx.land.crossings.list.is_empty() || !ctx.land.enclosures.list.is_empty() {
            self.facilities_now(ctx);
            self.schedule_pass(ctx, h, 0.0, None);
        }
        if let Some((household, points)) = seen {
            self.see_places(ctx, household, &points);
        }
        true
    }

    fn start_work(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        minutes: u32,
        behavior: Option<Behavior>,
    ) {
        let now = ctx.now;
        let params = ctx.params;
        let present = match (behavior, self.people.get(h)) {
            (
                Some(
                    Behavior::Socialize
                    | Behavior::Attend
                    | Behavior::Petition
                    | Behavior::Poll
                    | Behavior::Visit,
                ),
                Some(p),
            ) => self.hearth_company(ctx, p.id, p.act.target),
            _ => Vec::new(),
        };
        // Someone come to another settlement's hearth sees it and is counted (M5a slice AM).
        if let (Some(Behavior::Visit), Some(p)) = (behavior, self.people.get(h))
            && let Target::Hearth(s) = p.act.target
        {
            let me = p.id;
            self.visited(ctx, me, s, &present, minutes);
        }
        if let (Some(b), Some(p)) = (behavior, self.people.get(h))
            && is_company(b)
            && let Target::Hearth(s) = p.act.target
            && let Some(at) = &mut self.at_hearth
        {
            let there = at.entry(s).or_default();
            if let Err(i) = there.binary_search_by_key(&p.id, |&(q, _)| q) {
                there.insert(i, (p.id, h));
            }
        }
        // Someone come to the gathering takes their place there (ADR-0013 §1).
        if let (Some(Behavior::Attend), Some(p)) = (behavior, self.people.get(h)) {
            let me = p.id;
            self.attend(ctx, me);
        }
        // Someone come to poll names their choices (M7a slice BO).
        if let (Some(Behavior::Poll), Some(p)) = (behavior, self.people.get(h)) {
            let me = p.id;
            self.cast(ctx, me);
        }
        // Someone come to a petition is counted among those who came (M4c slice AH).
        if let (Some(Behavior::Petition), Some(p)) = (behavior, self.people.get(h)) {
            let me = p.id;
            self.join_petition(ctx, me);
        }
        let companions = present.len();
        // Company at the hearth: a few of those there become ties (ADR-0014 §2), a session's
        // worth each, a block's for each of its sessions.
        if let (false, Some(p)) = (present.is_empty(), self.people.get(h)) {
            let session = ctx
                .catalog
                .activities
                .get(p.act.def as usize)
                .map_or(minutes, |a| a.min_minutes.max(1));
            let sessions = (minutes + session / 2) / session.max(1);
            let me = p.id;
            self.keep_company(ctx, me, &present, minutes, sessions.max(1));
        }
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, now, params);
        let def_par = ctx
            .catalog
            .activities
            .get(p.act.def as usize)
            .map_or(params.energy.idle_par, |a| a.par);
        let (par, asleep, company) = match behavior {
            Some(Behavior::Sleep) => (def_par, true, params.social.household_quality),
            Some(
                Behavior::Socialize
                | Behavior::Attend
                | Behavior::Petition
                | Behavior::Poll
                | Behavior::Visit,
            ) => (
                def_par,
                false,
                (params.social.quality_per_companion * companions as f64).min(1.0),
            ),
            // Work at home is done among the household.
            Some(
                Behavior::Eat
                | Behavior::Rest
                | Behavior::Play
                | Behavior::Make
                | Behavior::Try
                | Behavior::Tend
                | Behavior::Teach
                | Behavior::Write
                | Behavior::Read,
            ) => (def_par, false, params.social.household_quality),
            Some(
                Behavior::Gather
                | Behavior::Dig
                | Behavior::FetchWater
                | Behavior::Farm
                | Behavior::Ask
                | Behavior::Trade
                | Behavior::Fetch
                | Behavior::Hire
                | Behavior::Build
                | Behavior::Take
                | Behavior::Watch
                | Behavior::Light
                | Behavior::Rounds
                | Behavior::Midden
                | Behavior::Call
                | Behavior::Study
                | Behavior::Carry
                | Behavior::Bridge
                | Behavior::Boat
                | Behavior::Cart
                | Behavior::Way
                | Behavior::Raise
                | Behavior::Well
                | Behavior::Levee
                | Behavior::Palisade
                | Behavior::Ditch
                | Behavior::Mill
                | Behavior::Douse
                | Behavior::Strip
                | Behavior::Pull
                | Behavior::Salvage
                | Behavior::Band,
            ) => (def_par, false, 0.0),
            None => (params.energy.idle_par, false, 0.0),
        };
        p.burn_kcal_min = burn_rate(p, now, params, par);
        p.asleep = asleep;
        p.company = company as f32;
        p.trip = None;
        p.act.step_started = now;
        p.act.step_ends = now.plus_minutes(i64::from(minutes.max(1)));
        let (id, version, ends) = (p.id, p.act.version, p.act.step_ends);
        ctx.schedule_step(ends, id, version);
        // At the muster, or arrived where the band went (M6c slice BJ): last, so that an
        // encounter may cut short the stay just scheduled.
        if behavior == Some(Behavior::Band) {
            self.band_work(ctx, h);
        }
    }

    fn end_step(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        // The muster, and coming home, for those with a band (M6c slice BJ).
        let band = self.people.get(h).is_some_and(|p| {
            ctx.catalog
                .activities
                .get(p.act.def as usize)
                .is_some_and(|a| a.behavior == Behavior::Band)
        });
        if band {
            self.band_step_ended(ctx, h);
        }
        let now = ctx.now;
        let params = ctx.params;
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, now, params);
        let step = p.act.steps.get(p.act.step as usize).copied();
        let def = ctx.catalog.activities.get(p.act.def as usize).cloned();
        let (who, act_def, act_target, started, household) = (
            p.id,
            p.act.def,
            p.act.target,
            p.act.step_started,
            p.household,
        );
        // The technique the work needs, found before the work changes what it is aimed at.
        let technique = match (step, def.as_ref()) {
            (Some(Step::Work { .. }), Some(d)) => self.technique_for(ctx, d, act_target, household),
            _ => None,
        };
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        match step {
            Some(Step::Walk { to }) => {
                p.pos = to;
                if let Some(trip) = p.trip.take() {
                    let map = ctx.map;
                    let mut cells =
                        cells_along(&trip.points, map.cell_size_m, map.width, map.height);
                    cells.retain(|&c| map.water[c as usize] == WATER_LAND);
                    ctx.land
                        .wear
                        .walk(&cells, now.day_index(), &ctx.land_params.paths);
                    ctx.land
                        .ways
                        .walked(&cells, now.day_index(), civ_land::ways::WALK);
                    self.look_for_deposits(ctx, who, household, &trip.points);
                }
            }
            Some(Step::Work { minutes }) => match def.as_ref().map(|d| d.behavior) {
                Some(Behavior::Sleep) => p.asleep = false,
                Some(Behavior::Eat) => {
                    let hh = p.household;
                    self.eat(ctx, h, hh);
                }
                Some(Behavior::Gather) => {
                    if let (Some(d), Target::Patch(patch)) = (def.as_ref(), p.act.target)
                        && let Some(r) = d.resource
                        && let Some(res) = ctx.land_params.resources.get(r)
                    {
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * d.rate;
                        let hours = f64::from(minutes) / 60.0;
                        let u = draw(ctx.seed, p);
                        let got =
                            ctx.land
                                .harvest(ctx.land_params, r, patch as usize, hours, eff, u);
                        // Carry a load at most: plants and fish beyond it stay where they were;
                        // meat beyond it is left at the kill.
                        let can = params.household.carry_kg / res.unit_kg.max(1e-9);
                        let kept = got.min(can);
                        if got > kept && !res.discrete {
                            ctx.land.stocks[r][patch as usize] += (got - kept) as f32;
                        }
                        p.carrying.good = Some(res.good as u16);
                        p.carrying.kg = (kept * res.unit_kg) as f32;
                        let (hh_id, kcal) = (p.household, kept * res.unit_kg);
                        let seen = (r as u16, patch, got, hours, now.day_index());
                        self.remember_patch(hh_id, seen, res.renewal_days());
                        // Who worked this place today (M5c slice AT): food got, kcal.
                        let kcal = ctx
                            .catalog
                            .goods
                            .get(res.good)
                            .map_or(0.0, |g| kcal * g.kcal_per_kg);
                        self.log_work(crate::uses::Place::Patch(patch), hh_id, who, now, kcal);
                    }
                }
                // Digging at a deposit's pit (M3b slice Q, ADR-0010 §2).
                Some(Behavior::Dig) => {
                    if let (Some(d), Target::Deposit(deposit)) = (def.as_ref(), p.act.target)
                        && let Some(good) = d.digs
                    {
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * d.rate;
                        let hh = self
                            .hh_index
                            .get(&household)
                            .and_then(|&x| self.households.get(x));
                        let (settlement, home) =
                            (hh.and_then(|x| x.settlement), hh.map_or(p.pos, |x| x.home));
                        let hearth = settlement
                            .and_then(|s| ctx.land.settlements.iter().find(|x| x.id == s))
                            .map(|x| x.hearth_m);
                        let toward = hearth.unwrap_or(p.pos);
                        let toward = (f64::from(toward.0), f64::from(toward.1));
                        // Onto the household's train, where its engine works the line from the pit
                        // (M10b slice DW, third part); by its wagon, where its line runs from the
                        // pit home (M10b slice DV, third part); else what one person carries.
                        let (kg, loaded) = Self::dig_and_bring(
                            ctx,
                            &self.consignments,
                            household,
                            home,
                            hearth,
                            toward,
                            deposit,
                            f64::from(minutes),
                            eff,
                        );
                        if kg > 0.0 {
                            p.carrying.good = Some(good as u16);
                            p.carrying.kg = kg as f32;
                        }
                        if let Some((wagon, kg)) = loaded {
                            self.load_wagon(ctx, household, wagon, good, kg);
                        }
                        let place = crate::uses::Place::Deposit(deposit);
                        self.log_work(place, household, who, now, 0.0);
                    }
                }
                Some(Behavior::FetchWater) => {
                    let (who, household) = (p.id, p.household);
                    match p.act.target {
                        // At a well, what its column holds of a load (M6a slice AY, step three).
                        Target::Well(well) => {
                            let got = self.drew_from_well(ctx, well, household, who, now);
                            if let Some(p) = self.people.get_mut(h) {
                                p.carrying.water_l = got as f32;
                            }
                            if let Some(cell) = ctx.land.wells.get(well).map(|w| w.cell) {
                                self.drew_at(who, household, cell);
                            }
                        }
                        Target::Water(cell) => {
                            p.carrying.water_l = params.household.carry_water_l as f32;
                            self.drew_water(ctx, cell, household, who, now);
                            self.drew_at(who, household, cell);
                        }
                        _ => p.carrying.water_l = params.household.carry_water_l as f32,
                    }
                }
                Some(Behavior::Pull) => {
                    let (who, target) = (p.id, p.act.target);
                    if let (Target::Building(building), Some(Step::Work { minutes })) =
                        (target, step)
                    {
                        self.pull_down(ctx, who, building, minutes);
                    }
                }
                Some(Behavior::Strip) => {
                    // Covering torn from the roof (M6b slice BD, step four).
                    let (who, target) = (p.id, p.act.target);
                    if let (Target::Building(building), Some(Step::Work { minutes })) =
                        (target, step)
                    {
                        self.strip_roof(ctx, who, building, minutes);
                    }
                }
                Some(Behavior::Salvage) => {
                    // A load of goods taken up from under the roof; set down out of the way.
                    let (who, household, target, step_at) =
                        (p.id, p.household, p.act.target, p.act.step);
                    if let Target::Building(building) = target
                        && step_at <= 1
                    {
                        self.salvage(ctx, who, household, building);
                    }
                }
                Some(Behavior::Douse) => {
                    // A load drawn, or thrown on the roof (M6b slice BD, step three).
                    let (who, household, target, step_at) =
                        (p.id, p.household, p.act.target, p.act.step);
                    if let Target::Douse { building, from } = target {
                        if step_at <= 1 {
                            self.draw_for_fire(ctx, h, who, household, from);
                        } else {
                            self.throw_water(ctx, h, who, building);
                        }
                    }
                }
                Some(Behavior::Farm) => {
                    if let (Some(d), Target::Field(field)) = (def.as_ref(), p.act.target)
                        && let Some(task) = d.task
                    {
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * d.rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.field_work(ctx, who, household, field, task, hours);
                    }
                }
                // A session taught or studied, tallied for midnight (M10c slice EB, second part).
                Some(Behavior::Teach) => {
                    if let Target::Technique(t) = p.act.target {
                        let who = p.id;
                        crate::knowledge::tally(&mut self.taught, who, t, minutes as f32);
                    }
                }
                Some(Behavior::Study) => {
                    if let Target::Technique(t) = p.act.target {
                        let who = p.id;
                        crate::knowledge::tally(&mut self.studied, who, t, minutes as f32);
                        self.pay_study_fee(ctx, who, t);
                    }
                }
                // A record written toward, or read (M10c slice EC).
                Some(Behavior::Write) => {
                    if let Target::Technique(t) = p.act.target {
                        let who = p.id;
                        self.finish_write(ctx, who, t, f64::from(minutes));
                    }
                }
                Some(Behavior::Read) => {
                    if let Target::Technique(t) = p.act.target {
                        let who = p.id;
                        self.finish_read(ctx, who, t, f64::from(minutes));
                    }
                }
                // A call left at a carer's door (M10c slice DZ, second part).
                Some(Behavior::Call) => {
                    if let Target::Household(carer) = p.act.target {
                        let household = p.household;
                        self.call_carer(ctx, household, carer);
                    }
                }
                Some(Behavior::Ask) => {
                    if let Target::Household(giver) = p.act.target {
                        let (asker, household) = (p.id, p.household);
                        self.give_food(ctx, giver, household, asker);
                    }
                }
                Some(Behavior::Take) => {
                    if let Target::Household(victim) = p.act.target {
                        self.take_from(ctx, h, victim);
                    }
                }
                Some(Behavior::Watch) => {
                    let who = p.id;
                    self.stood_watch(who, minutes);
                }
                // A home on the health officer's round, seen and entered (M10c slice EE, second
                // part).
                Some(Behavior::Rounds) => {
                    let (who, pos) = (p.id, p.pos);
                    self.round_stop(ctx, who, pos, minutes);
                }
                // A midden carried from the water (M10c slice EE, third part).
                Some(Behavior::Midden) => {
                    let (who, pos) = (p.id, p.pos);
                    self.carry_midden(ctx, who, pos);
                }
                // A street lamp lit where the lamplighter stands (M10b slice DY, fourth part).
                Some(Behavior::Light) => {
                    let (who, pos) = (p.id, p.pos);
                    self.light_lamp(ctx, who, pos);
                }
                Some(Behavior::Carry) => {
                    // Handed over at the other hearth; at the store, taking it up moves nothing
                    // (M5c slice AV).
                    if let Target::Hearth(s) = p.act.target {
                        let (who, pos) = (p.id, p.pos);
                        self.hand_over(ctx, who, s, pos);
                    }
                }
                Some(Behavior::Trade | Behavior::Fetch) => {
                    if let Target::Household(seller) | Target::Firm(seller) = p.act.target {
                        let (who, household) = (p.id, p.household);
                        self.settle_trade(ctx, household, seller, who);
                    }
                }
                Some(Behavior::Build) => {
                    if let Target::Building(building) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.build_work(ctx, (h, who), household, building, hours);
                    }
                }
                Some(Behavior::Bridge) => {
                    if let Target::Crossing(crossing) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.crossing_work(ctx, (h, who), household, crossing, hours);
                    }
                }
                Some(Behavior::Raise) => {
                    if let Target::Building(building) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let household = p.household;
                        self.raise_work(ctx, household, building, hours);
                    }
                }
                Some(Behavior::Way) => {
                    if let Target::Way(way) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let household = p.household;
                        self.way_work(ctx, household, way, hours);
                    }
                }
                Some(Behavior::Boat | Behavior::Cart) => {
                    if let Target::Boat(boat) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.boat_work(ctx, (h, who), household, boat, hours);
                    }
                }
                Some(Behavior::Well) => {
                    if let Target::Well(well) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.well_work(ctx, (h, who), household, well, hours);
                    }
                }
                Some(Behavior::Levee) => {
                    if let Target::Bank(bank) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.levee_work(ctx, (h, who), household, bank, hours);
                    }
                }
                Some(Behavior::Palisade) => {
                    if let Target::Enclosure(enclosure) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.enclosure_work(ctx, who, household, enclosure, hours);
                    }
                }
                Some(Behavior::Ditch) => {
                    if let Target::Ditch(ditch) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.ditch_work(ctx, who, household, ditch, hours);
                    }
                }
                Some(Behavior::Mill) => {
                    if let Target::Mill(mill) = p.act.target {
                        self.mill_grind(ctx, h, mill, minutes);
                    }
                }
                Some(Behavior::Hire) => {
                    if let Target::Firm(firm) = p.act.target {
                        self.hired_work(ctx, h, firm, minutes);
                    }
                }
                Some(Behavior::Tend) => {
                    // At home, or at close kin's home (M10c slice DZ), or at the sickhouse (M10c
                    // slice EE, fourth part).
                    let (who, def) = (p.id, p.act.def);
                    match p.act.target {
                        Target::Building(b) => self.tend_at_sickhouse(ctx, who, b, def, minutes),
                        Target::Household(h) => self.tended(ctx, who, h, def, minutes),
                        _ => {
                            let household = p.household;
                            self.tended(ctx, who, household, def, minutes);
                        }
                    }
                }
                Some(Behavior::Make) => {
                    if let Some(d) = def.as_ref() {
                        match p.act.target {
                            Target::Firm(firm) => self.firm_make(ctx, h, d, minutes, firm),
                            _ => self.make_work(ctx, h, d, minutes),
                        }
                    }
                }
                _ => {}
            },
            Some(Step::Paddle { to }) => {
                p.pos = civ_land::boating::RiverGraph::centre(&ctx.land.rivers, to);
                if let Some(trip) = p.trip.take() {
                    let from = trip.points.first().map(|&at| cell_of(ctx.map, at) as u32);
                    if let Some(from) = from {
                        Self::paddled(ctx, household, (from, to));
                    }
                }
            }
            Some(Step::Haul { to }) => {
                p.pos = to;
                if let Some(trip) = p.trip.take() {
                    let map = ctx.map;
                    let mut cells =
                        cells_along(&trip.points, map.cell_size_m, map.width, map.height);
                    cells.retain(|&c| map.water[c as usize] == WATER_LAND);
                    ctx.land
                        .wear
                        .walk(&cells, now.day_index(), &ctx.land_params.paths);
                    // The hauler walks beside the cart: a trip on footpath and cart track alike.
                    ctx.land.ways.walked(
                        &cells,
                        now.day_index(),
                        civ_land::ways::WALK | civ_land::ways::CART,
                    );
                    if let Some(&from) = trip.points.first() {
                        let from = cell_of(map, from) as u32;
                        Self::hauled(ctx, household, (from, cell_of(map, to) as u32));
                    }
                }
            }
            // Off the train at the line's end (M10b slice DX, fourth part): aboard, no ground worn.
            Some(Step::Ride { to }) => {
                p.pos = cell_centre(ctx.map, to as usize);
                p.trip = None;
            }
            Some(Step::Handle { unload: true, .. }) => self.unloaded(ctx, who),
            Some(Step::Wait { .. } | Step::Deposit | Step::Handle { .. }) | None => {}
        }
        // The time the step took.
        let took = (now.minutes() - started.minutes()).max(0) as f64;
        match step {
            Some(
                Step::Walk { .. } | Step::Paddle { .. } | Step::Haul { .. } | Step::Ride { .. },
            ) => self.time_use.walking += took,
            Some(Step::Wait { .. } | Step::Handle { .. }) => self.time_use.waiting += took,
            Some(Step::Work { .. }) => {
                let b = def.as_ref().map(|d| d.behavior);
                if let Some(i) = b.and_then(|b| Behavior::ALL.iter().position(|&x| x == b))
                    && let Some(m) = self.time_use.work.get_mut(i)
                {
                    *m += took;
                }
            }
            Some(Step::Deposit) | None => {}
        }
        // A household's view of its options is out of date after its consequential steps
        // (ADR-0011 §4): work on a field or a building, making, trying; and every household's
        // after a gift, a trade or paid work, which move goods between households.
        if let (Some(Step::Work { .. }), Some(d)) = (step, def.as_ref()) {
            match d.behavior {
                Behavior::Farm | Behavior::Build | Behavior::Make | Behavior::Try => {
                    self.changed(household);
                }
                Behavior::Ask
                | Behavior::Trade
                | Behavior::Fetch
                | Behavior::Hire
                | Behavior::Take => {
                    self.view_counts.2 += 1;
                    if ctx.approx.views_by_party {
                        self.views_moved(household, act_target);
                    } else {
                        self.views.clear();
                    }
                }
                _ => {}
            }
        }
        // A knower has practised the work's technique; a learner learns by it (ADR-0008 §4).
        if let (Some(Step::Work { minutes }), Some(t)) = (step, technique) {
            self.practise(ctx, who, t, act_def, act_target, f64::from(minutes) / 60.0);
        }
        // The work may have found something new (ADR-0008 §3).
        if let Some(Step::Work { minutes }) = step {
            if def.as_ref().is_some_and(|d| d.behavior == Behavior::Try)
                && let Some(p) = self.people.get_mut(h)
            {
                p.tried = Some(now);
            }
            let hours = f64::from(minutes) / 60.0;
            self.discover(ctx, who, (act_def, act_target), hours, started);
        }
        // Work wears the tools it needs (a recipe's are worn where it is worked).
        if let (Some(Step::Work { minutes }), Some(d)) = (step, def.as_ref())
            && d.behavior != Behavior::Make
            && !d.tools.is_empty()
            && let Some(household) = self.people.get(h).map(|p| p.household)
        {
            self.wear_tools(ctx, household, &d.tools, f64::from(minutes) / 60.0);
        }
        if let Some(p) = self.people.get_mut(h) {
            p.burn_kcal_min = burn_rate(p, now, params, params.energy.idle_par);
            p.asleep = false;
        }
    }

    /// Wears `tools` of `household` by `hours` of use.
    fn wear_tools(&mut self, ctx: &Ctx, household: PermanentId, tools: &[usize], hours: f64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let before = x.stores.clone();
        make::wear(&mut x.stores, goods, tools, hours);
        x.flows
            .add_changes(&before, &x.stores, Flow::Made, Flow::Worn);
    }

    /// `minutes` of `who`'s work at activity `def` (a making), as their session would do: for
    /// tests.
    #[doc(hidden)]
    pub fn make_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        def: usize,
        minutes: u32,
    ) {
        let (Some(&h), Some(d)) = (self.index.get(&who), ctx.catalog.activities.get(def)) else {
            return;
        };
        let d = d.clone();
        self.make_work(ctx, h, &d, minutes);
    }

    /// Applies `minutes` of person `h` working the recipe of make activity `def` at home: what
    /// their skill and age let them make in that time, from what the household holds (seed only
    /// in hunger, and never the seed for the ground already cropped), the recipe's tools worn by
    /// the time worked, and their skill practised.
    /// Whether a lamp burns for work at household `household`'s home at `now` (M10b slice DU):
    /// it holds a whole lamp and its fuel, or its burners are lit by gas (`gas`, M10b slice DY),
    /// it is dark, and one of its members awake at home is at work kept to daylight.
    pub fn lamp_lit_home(
        &self,
        catalog: &crate::params::Catalog,
        params: &PeopleParams,
        gas: &civ_land::gas::Gas,
        now: SimTime,
        household: PermanentId,
    ) -> bool {
        let Some(x) = self.household(household) else {
            return false;
        };
        let t = now.minutes() as f64;
        dark_at(params, now)
            && (lamp_of(&catalog.goods, &x.stores).is_some() || gas.lit(household, now.minutes()))
            && x.members.iter().any(|&m| {
                self.person(m).is_some_and(|q| {
                    let at = q.position_at(t);
                    !q.asleep
                        && (at.0 - x.home.0).hypot(at.1 - x.home.1) <= LAMP_AT_HOME_M
                        && Self::works_by_lamp(catalog, q)
                })
            })
    }

    /// Homes where a lamp burns for work now (M10b slice DU), none until the world's first lamp
    /// has been lit.
    pub fn lit_homes(
        &self,
        catalog: &crate::params::Catalog,
        params: &PeopleParams,
        gas: &civ_land::gas::Gas,
        now: SimTime,
    ) -> usize {
        if self.order.lamp_first_lit.is_none() || !dark_at(params, now) {
            return 0;
        }
        self.households
            .iter()
            .filter(|(_, x)| self.lamp_lit_home(catalog, params, gas, now, x.id))
            .count()
    }

    fn make_work(&mut self, ctx: &mut Ctx, h: Handle<Person>, def: &ActivityDef, minutes: u32) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(recipe) = def.recipe.and_then(|r| ctx.catalog.recipes.get(r)) else {
            return;
        };
        let Some(p) = self.people.get(h) else {
            return;
        };
        let household = p.household;
        let skill = recipe
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let level = skill.map_or(0.0, |(k, _)| p.skill(k));
        let speed = skill.map_or(1.0, |(_, s)| interpolate(&s.speed, level))
            * interpolate(&params.capacity_by_age, p.age_years(now))
            * def.rate;
        let quality = skill.map_or(1.0, |(_, s)| interpolate(&s.quality, level));
        let reserve_ok = may_eat_reserve(p, now, params);
        let protected =
            farm::protected_seed(self.fields_of(ctx.land, household), &ctx.catalog.crops);
        let hours = f64::from(minutes) / 60.0;
        let mut units = make::units_in(recipe, f64::from(minutes), speed);
        // A powered recipe makes no more than its household's installations deliver over the
        // session (M10a slice DR; ADR-0033 §5).
        if recipe.kwh_per_unit > 0.0 {
            units = units.min(power_kw_of(ctx, household) * hours / recipe.kwh_per_unit);
        }
        // A machine its crew left idles (M10a slice DT).
        if !self.crewed(ctx, h, crew_of(goods, recipe)) {
            units = 0.0;
        }
        // Coin is struck from the metal brought to the mint the worker keeps, in the polity's
        // store, and nobody else strikes it (M7d slice CE2b).
        let coin = recipe
            .outputs
            .first()
            .and_then(|&(g, _)| goods.get(g))
            .and_then(|g| g.money.as_ref())
            .is_some_and(|m| m.coin);
        if coin {
            let Some((pi, mint, kg)) = self
                .mint_work(ctx, p.id)
                .filter(|w| def.recipe == Some(w.1.recipe))
            else {
                return;
            };
            let per = recipe.inputs.first().map_or(1.0, |i| i.1).max(1e-9);
            units = units.min(kg / per);
            self.strike(ctx, pi, &mint, units);
            if let Some(x) = self
                .hh_index
                .get(&household)
                .and_then(|&hd| self.households.get_mut(hd))
            {
                let members = x.members.len().max(1);
                x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                x.stores.resize(goods.len(), 0.0);
                let before = x.stores.clone();
                make::wear(&mut x.stores, goods, &recipe.tools, hours);
                x.flows
                    .add_changes(&before, &x.stores, Flow::Made, Flow::Worn);
            }
            if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
                let next = s.practised(p.skill(k), hours);
                p.set_skill(k, next);
            }
            return;
        }
        let mut lit = None;
        if let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            let members = x.members.len().max(1);
            x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            x.stores.resize(goods.len(), 0.0);
            let before = x.stores.clone();
            match def.recipe.filter(|_| recipe.elapsed_days > 0.0) {
                // A batch (M10a slice DP, ADR-0033 §2): open those of this recipe whose days are
                // past, and start another unless one is still under way.
                Some(ri) => {
                    let today = now.day_index();
                    let mut running = false;
                    let mut opened = Vec::new();
                    x.batches.retain(|b| {
                        if b.recipe != ri {
                            true
                        } else if b.ends_day <= today {
                            opened.push((b.units, b.quality));
                            false
                        } else {
                            running = true;
                            true
                        }
                    });
                    for (u, q) in opened {
                        make::give(recipe, &mut x.stores, goods, u, q);
                    }
                    if !running {
                        let started =
                            make::start(recipe, &mut x.stores, goods, units, reserve_ok, protected);
                        if started > 0.0 {
                            x.batches.push(Batch {
                                recipe: ri,
                                units: started,
                                quality,
                                ends_day: today + recipe.elapsed_days.ceil() as i64,
                            });
                        }
                    }
                }
                None => {
                    make::apply(
                        recipe,
                        &mut x.stores,
                        goods,
                        units,
                        quality,
                        reserve_ok,
                        protected,
                    );
                }
            }
            x.flows
                .add_changes(&before, &x.stores, Flow::Made, Flow::Used);
            let before = x.stores.clone();
            make::wear(&mut x.stores, goods, &recipe.tools, hours);
            x.flows
                .add_changes(&before, &x.stores, Flow::Made, Flow::Worn);
            // Gas made beyond its holders' room goes to the air (M10b slice DY, ADR-0034 §8).
            for (g, kg) in make::keep_to_holders(goods, &mut x.stores) {
                x.flows.add(Flow::Spoiled, g, kg);
            }
            // Work after dark by lamplight burns the lamp's fuel and wears the lamp (M10b slice
            // DU; research 11-13 §2.2: light's cost is its fuel and fixture by the hour lit).
            if def.daylight_only {
                let end = self.enact_end.unwrap_or(now);
                let dark_h = dark_minutes(params, end, minutes) / 60.0;
                if dark_h > 0.0 && ctx.land.gas.lit(household, end.minutes() - 1) {
                    // Burners lit by gas burn none of the household's fuel: it came by the main
                    // (M10b slice DY).
                    lit = Some(end);
                } else if dark_h > 0.0
                    && let Some((lamp, fuel, kg_h, _)) = lamp_of(goods, &x.stores)
                {
                    lit = Some(end);
                    let before = x.stores.clone();
                    x.stores[fuel] = (x.stores[fuel] - kg_h * dark_h).max(0.0);
                    x.flows
                        .add_changes(&before, &x.stores, Flow::Made, Flow::Used);
                    let before = x.stores.clone();
                    make::wear(&mut x.stores, goods, &[lamp], dark_h);
                    x.flows
                        .add_changes(&before, &x.stores, Flow::Made, Flow::Worn);
                }
            }
        }
        // The world's first lamp lit for work is remembered: from then on the dark shortens
        // sight where no lamp's light falls (ADR-0034 §9).
        if self.order.lamp_first_lit.is_none() {
            self.order.lamp_first_lit = lit;
        }
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            let next = s.practised(p.skill(k), hours);
            p.set_skill(k, next);
        }
    }

    /// `person` of `household` worked `place` now and got `kcal` of food there: logged for the
    /// day's end, when who else worked it that day is known (M5c slice AT, ADR-0020 §5).
    /// Someone of `household` drew a load of water at `cell` (M6a slice AY, ADR-0021 §1): at a
    /// spring it counts against what flows to it today, and every draw is logged as a use.
    fn drew_water(
        &mut self,
        ctx: &Ctx,
        cell: u32,
        household: PermanentId,
        who: PermanentId,
        now: SimTime,
    ) {
        let patch = ctx.land.patches.of_cell(cell as usize, ctx.map.width);
        let aq = &ctx.land.water.aquifer;
        let spring = aq.stage.get(patch).is_some_and(Option::is_none)
            && aq
                .seep
                .get(patch)
                .copied()
                .flatten()
                .is_some_and(|(c, _)| c == cell);
        let litres = ctx.params.household.carry_water_l;
        if spring {
            self.spring_draws
                .draw(now.day_index(), patch as u32, litres);
            self.water_draws.springs += 1;
        } else {
            self.water_draws.banks += 1;
        }
        self.water_draws.litres += litres;
        if !spring {
            self.drew_load_from_reach(ctx, cell, household, litres);
        }
        self.log_work(crate::uses::Place::Source(cell), household, who, now, 0.0);
    }

    /// Where people of a household with `home` fetch water now (M6a slice AY, ADR-0021 §1): the
    /// nearest spring that flows and has a load left today, if one is nearer than the river or
    /// lake, else the river or lake; or, for household `x` (and whether it is short of water),
    /// the nearest well it may draw at that holds a load, if that is nearer still (step three).
    /// A source its people suspect is passed over while it knows another (M6a slice BA, ADR-0021
    /// §7), and so is one a closing in force names that `who`, the one going, keeps (step four);
    /// when every source it knows is passed over, it draws where it would have.
    fn water_source(
        &self,
        ctx: &Ctx,
        home: Option<&HomeField>,
        x: Option<(&Household, bool)>,
        who: Option<PermanentId>,
    ) -> Option<WaterOption> {
        let mut suspect = x.map_or_else(Vec::new, |(x, _)| self.suspected_sources(x));
        if let (Some(who), Some((x, _))) = (who, x) {
            let kept = self.closures_kept(ctx, who, x);
            if !kept.is_empty() {
                suspect.extend(kept);
                suspect.sort_unstable();
                suspect.dedup();
            }
        }
        let shunned = |cell: u32| {
            suspect
                .binary_search(&crate::uses::Place::Source(cell))
                .is_ok()
        };
        let chosen = self.water_source_among(ctx, home, x, &shunned);
        if chosen.is_none() && !suspect.is_empty() {
            return self.water_source_among(ctx, home, x, &|_| false);
        }
        chosen
    }

    /// [`Self::water_source`] among the sources `shunned` does not pass over.
    fn water_source_among(
        &self,
        ctx: &Ctx,
        home: Option<&HomeField>,
        x: Option<(&Household, bool)>,
        shunned: &dyn Fn(u32) -> bool,
    ) -> Option<WaterOption> {
        let other = home
            .and_then(|home| self.spring_or_bank(ctx, home, shunned))
            .filter(|o| !shunned(o.cell));
        let bound = other.map_or(f64::INFINITY, |o| 2.0 * o.walk_min);
        let well = x.and_then(|(x, short)| self.well_option(ctx, x, short, bound, shunned));
        // The well is chosen when its walk there and back and its lift take no longer.
        match (well, other) {
            (Some(w), Some(o)) if 2.0 * o.walk_min < 2.0 * w.walk_min + w.lift_min => Some(o),
            (Some(w), _) => Some(w),
            (None, o) => o,
        }
    }

    /// The nearest spring that flows, has a load left today and is not `shunned`, if one is
    /// nearer than the river or lake, else the river or lake, from `home`.
    fn spring_or_bank(
        &self,
        ctx: &Ctx,
        home: &HomeField,
        shunned: &dyn Fn(u32) -> bool,
    ) -> Option<WaterOption> {
        let (water, params) = (&ctx.land.water, &ctx.land_params.water);
        let carry = ctx.params.household.carry_water_l;
        let day = ctx.now.day_index();
        let option = |cell: u32, s: f32| WaterOption {
            cell,
            walk_min: f64::from(s) / 60.0,
            at: cell_centre(ctx.map, cell as usize),
            well: None,
            lift_min: 0.0,
        };
        for &(s, patch, cell) in &home.springs {
            if shunned(cell) {
                continue;
            }
            let Some(spring) = water.spring_at(params, patch as usize) else {
                continue;
            };
            if spring.flow_m3_day * 1000.0 - self.spring_draws.drawn(day, patch) >= carry {
                return Some(option(cell, s));
            }
        }
        // The nearest edge of the water, or the nearest further along it not passed over.
        home.water
            .into_iter()
            .chain(home.banks.iter().copied())
            .find(|&(cell, _)| !shunned(cell))
            .or(home.water)
            .map(|(cell, s)| option(cell, s))
    }

    fn log_work(
        &mut self,
        place: crate::uses::Place,
        household: PermanentId,
        person: PermanentId,
        now: SimTime,
        kcal: f64,
    ) {
        let Some(settlement) = self.household(household).and_then(|x| x.settlement) else {
            return;
        };
        self.uses.worked(crate::uses::Worked {
            place,
            day: now.day_index(),
            household,
            settlement,
            kcal: kcal as f32,
            person: Some(person),
        });
    }

    /// Records what a trip found at a place: `(resource, place, units, hours, day)`. Households of a
    /// settlement share what they learn (they talk at the hearth), so every one of them remembers
    /// it; what has faded to almost nothing is dropped.
    fn remember_patch(
        &mut self,
        household: PermanentId,
        (resource, patch, units, hours, day): (u16, u32, f64, f64, i64),
        renewal_days: f64,
    ) {
        let settlement = self.household(household).and_then(|h| h.settlement);
        for (_, hh) in self.households.iter_mut() {
            if hh.id != household && (settlement.is_none() || hh.settlement != settlement) {
                continue;
            }
            // What has faded is let go, once a day: a place seen since was seen that day, and
            // has not faded (M5a slice AL).
            if self.known_pruned.insert((hh.id, resource), day) != Some(day) {
                hh.known.retain(|k| {
                    k.resource != resource
                        || k.weight(day, renewal_days) >= 0.05 * f64::from(k.hours)
                });
            }
            match hh
                .known
                .iter_mut()
                .find(|k| k.resource == resource && k.patch == patch)
            {
                Some(k) => k.observe(units, hours, day, renewal_days),
                None => {
                    let mut k = KnownPatch {
                        resource,
                        patch,
                        rate: 0.0,
                        hours: 0.0,
                        seen_day: day,
                    };
                    k.observe(units, hours, day, renewal_days);
                    hh.known.push(k);
                }
            }
        }
    }

    /// Household `giver` gives household `to`, which asked, food it can spare: up to what brings
    /// `to` to the days of food it tries to keep, and no more than one person carries home. The
    /// most perishable food goes first. No debt is kept (research 08-11 §5.4: need-based help).
    fn give_food(&mut self, ctx: &Ctx, giver: PermanentId, to: PermanentId, asker: PermanentId) {
        // The common store gives by its law (ADR-0013 §4).
        if let Some(pi) = self.polities.iter().position(|p| p.id == giver) {
            self.give_relief(ctx, pi, to);
            return;
        }
        // A faction's store gives its members' households (M4c slice AH).
        if let Some(fi) = self.factions.list.iter().position(|f| f.id == giver) {
            self.give_aid(ctx, fi, to);
            return;
        }
        // A household that believes the asker took from it, or from those it regards, refuses
        // them (ADR-0015 §3).
        if self.refuses(ctx, giver, asker) {
            self.order.refusals += 1;
            return;
        }
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let (Some(&gh), Some(&th)) = (self.hh_index.get(&giver), self.hh_index.get(&to)) else {
            return;
        };
        if gh == th {
            return;
        }
        let (Some(g), Some(t)) = (self.households.get(gh), self.households.get(th)) else {
            return;
        };
        let spare = spare_food_kcal(g, now, params, goods);
        let need = t.members.len() as f64 * params.household.daily_kcal_per_person;
        let held = stock_kcal(&stores_now(t, now, params, goods), goods);
        let mut want = spare.min((params.household.food_target_days * need - held).max(0.0));
        if want <= 0.0 {
            return;
        }
        let stores = stores_now(g, now, params, goods);
        let mut order: Vec<usize> = (0..goods.len())
            .filter(|&i| {
                goods[i].purpose == GoodUse::Food
                    && goods[i].kcal_per_kg > 0.0
                    && !goods[i].kept_back()
            })
            .collect();
        let keeps = |i: usize| match goods[i].half_life_days {
            h if h > 0.0 => h,
            _ => f64::INFINITY,
        };
        order.sort_by(|&a, &b| keeps(a).total_cmp(&keeps(b)).then(a.cmp(&b)));
        let mut carry = params.household.carry_kg;
        let mut legs = Vec::new();
        for i in order {
            if want <= 0.0 || carry <= 0.0 {
                break;
            }
            let kg = (want / goods[i].kcal_per_kg)
                .min(stores.get(i).copied().unwrap_or(0.0).max(0.0))
                .min(carry);
            if kg <= 0.0 {
                continue;
            }
            legs.push(Leg {
                from: giver,
                to,
                good: i,
                amount: kg,
            });
            carry -= kg;
            want -= kg * goods[i].kcal_per_kg;
        }
        if legs.is_empty() {
            return;
        }
        // The help, in hours of the receivers' own work (what getting it themselves would cost).
        let help_h = self.household(to).map_or(0.0, |t| {
            let costs = self.own_costs_of(ctx, t);
            legs.iter()
                .map(|l| costs.get(l.good).copied().flatten().unwrap_or(0.0) * l.amount)
                .sum::<f64>()
        });
        if self.transfer(now, params, goods, &legs, Channel::Gift) {
            self.note_between(
                ctx,
                Some(asker),
                to,
                giver,
                crate::ties::Act::GiftReceived,
                crate::ties::Act::GiftGiven,
                help_h,
            );
        }
    }

    /// Hours of a capable adult's field work `members` can give a day.
    fn labour_per_day(&self, members: &[PermanentId], now: SimTime, params: &PeopleParams) -> f64 {
        members
            .iter()
            .filter_map(|m| self.person(*m))
            .map(|q| interpolate(&params.capacity_by_age, q.age_years(now)))
            .sum::<f64>()
            * params.farm.work_hours_per_day
    }

    /// Carries `hours` of a capable adult's work of dung from household `hh`'s midden to field
    /// `fi` and spreads it (M3c slice V): as much as the hours carry at the walk from home, a load
    /// at a time, never more than the heap holds. Its nitrogen enters the field's soil.
    fn manure_field(&mut self, ctx: &mut Ctx, hh: Handle<Household>, fi: usize, hours: f64) {
        let params = ctx.params;
        let Some(x) = self.households.get(hh) else {
            return;
        };
        // Walking times are kept from the settlement's hearth, or a lone household's home.
        let key = x
            .settlement
            .filter(|s| ctx.land.settlements.iter().any(|t| t.id == *s))
            .unwrap_or(x.id);
        let cell = cell_of(ctx.map, ctx.land.fields[fi].rect.centre_m());
        let Some(walk) = self
            .homes
            .get(&key)
            .and_then(|h| h.reach.seconds_to(cell))
            .map(|s| f64::from(s) / 60.0)
        else {
            return;
        };
        let members = x.members.len();
        let Some(x) = self.households.get_mut(hh) else {
            return;
        };
        x.midden.settle(ctx.now, members, &params.midden);
        let kg = (hours / params.midden.h_per_kg(walk).max(1e-9)).clamp(0.0, x.midden.kg);
        if kg <= 0.0 {
            return;
        }
        x.midden.kg -= kg;
        let f = &mut ctx.land.fields[fi];
        let ha = f.area_ha();
        if ha > 0.0 {
            f.soil
                .manured(&ctx.land_params.soil, kg * params.midden.n_per_kg() / ha);
        }
    }

    /// Applies `hours` of a capable adult's `task` on `field` by person `who` of `household`:
    /// seed comes out of the household's store, and threshed grain goes in, next season's seed
    /// first (the area it plans to sow at the crop's seed rate, with what the seed loses in store
    /// until sowing: research 08-01 §1.5, seed is kept by planned area, never as a share of the
    /// harvest).
    fn field_work(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        field: PermanentId,
        task: FieldTask,
        hours: f64,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(fi) = ctx.land.fields.iter().position(|f| f.id == field) else {
            return;
        };
        let Some(crop) = ctx.catalog.crops.get(usize::from(ctx.land.fields[fi].crop)) else {
            return;
        };
        let Some(&hh) = self.hh_index.get(&household) else {
            return;
        };
        if task == FieldTask::Manure {
            self.manure_field(ctx, hh, fi, hours);
            return;
        }
        let area: f64 = self
            .fields_of(ctx.land, household)
            .map(Field::area_ha)
            .sum();
        // Seed for the coming season is set aside before any grain is eaten.
        let seed_wanted = {
            let Some(x) = self.households.get(hh) else {
                return;
            };
            let kcal = goods.get(crop.good).map_or(0.0, |g| g.kcal_per_kg);
            let expected = farm::expected_yield_kg_ha(
                self.fields_of(ctx.land, household),
                crop,
                now.day_index(),
            );
            let need = farm::need_area_ha(x.members.len(), params, crop, kcal, expected);
            let labour = self.labour_per_day(&x.members, now, params);
            let share = workable_share(ctx.land, usize::from(ctx.land.fields[fi].crop));
            let share = farm::plan_share(share, peak_ratio(params));
            let plan = farm::plan_area_ha(need, area, labour, share, crop);
            let half_life = goods.get(crop.seed_good).map_or(0.0, |g| g.half_life_days);
            farm::seed_to_keep(plan, crop, half_life, now.day_index())
        };
        let Some(x) = self.households.get_mut(hh) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let seed = x
            .stores
            .get(crop.seed_good)
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        // What its season's water and its soil allow, fixed once the crop is ripe (ADR-0012
        // §2-3).
        let allow = ctx
            .land
            .allowance(ctx.land_params, &ctx.land.fields[fi], crop);
        let done = ctx.land.fields[fi].work(crop, task, hours, now.day_index(), allow, seed, now);
        if let Some(s) = x.stores.get_mut(crop.seed_good) {
            *s = (seed - done.seed_kg).max(0.0);
            x.flows
                .add(Flow::Sown, crop.seed_good, seed.min(done.seed_kg));
        }
        if done.grain_kg > 0.0 {
            x.flows.add(Flow::Got, crop.good, done.grain_kg);
            let have = x.stores.get(crop.seed_good).copied().unwrap_or(0.0);
            let to_seed = (seed_wanted - have).clamp(0.0, done.grain_kg);
            if let Some(s) = x.stores.get_mut(crop.seed_good) {
                *s += to_seed;
            }
            if let Some(g) = x.stores.get_mut(crop.good) {
                *g += done.grain_kg - to_seed;
            }
            // Grain set aside as seed is counted as got as seed, not grain.
            x.flows.add(Flow::Got, crop.good, -to_seed);
            x.flows.add(Flow::Got, crop.seed_good, to_seed);
            // Threshing leaves the straw at home too.
            if let Some((good, kg_per_kg)) = crop.straw
                && let Some(s) = x.stores.get_mut(good)
            {
                *s += done.grain_kg * kg_per_kg;
                x.flows.add(Flow::Got, good, done.grain_kg * kg_per_kg);
            }
        }
        let settlement = x.settlement;
        // A let field's holder takes its share of the grain threshed from it, through the ledger
        // (ADR-0007 §3): in grain, and in seed grain for what the grain does not cover.
        if done.grain_kg > 0.0
            && let (Some(lease), Party::Household(holder)) =
                (ctx.land.fields[fi].lease, ctx.land.fields[fi].holder)
            && holder != household
            && self.household(holder).is_some()
        {
            let rent = done.grain_kg * f64::from(lease.holder_share);
            let held = |pop: &Population, g: usize| {
                pop.household(household)
                    .and_then(|x| x.stores.get(g))
                    .copied()
                    .unwrap_or(0.0)
                    .max(0.0)
            };
            let in_grain = rent.min(held(self, crop.good));
            let in_seed = (rent - in_grain).min(held(self, crop.seed_good));
            let legs: Vec<Leg> = [(crop.good, in_grain), (crop.seed_good, in_seed)]
                .into_iter()
                .filter(|&(_, kg)| kg > 1e-9)
                .map(|(good, amount)| Leg {
                    from: household,
                    to: holder,
                    good,
                    amount,
                })
                .collect();
            if !legs.is_empty() && self.transfer(now, params, goods, &legs, Channel::Rent) {
                // The tenant saw the land lent; the holder saw its share paid (ADR-0014 §2).
                let (lent, paid) = (crate::ties::Act::LandLent, crate::ties::Act::RentPaid);
                self.note_between(ctx, Some(who), household, holder, lent, paid, 0.0);
            }
        }
        // A common store in force takes its share of what is left to the household (ADR-0013 §4).
        if done.grain_kg > 0.0 {
            let rent = match (ctx.land.fields[fi].lease, ctx.land.fields[fi].holder) {
                (Some(lease), Party::Household(holder)) if holder != household => {
                    f64::from(lease.holder_share)
                }
                _ => 0.0,
            };
            let kept = done.grain_kg * (1.0 - rent);
            // A creditor working a field pledged to it counts what it keeps against the debt
            // (M7d slice CB).
            let field = ctx.land.fields[fi].id;
            self.pledge_crop(ctx, field, household, crop.good, kept);
            self.levy(ctx, who, household, kept, crop.good);
            // And a faction the thresher belongs to its dues (M4c slice AH).
            self.pay_dues(ctx, who, household, kept, crop.good);
        }
        let Some(si) = settlement.and_then(|s| ctx.land.settlements.iter().position(|x| x.id == s))
        else {
            return;
        };
        ctx.land.settlements[si].harvest_kg += done.grain_kg;
        let first = task == FieldTask::Sow
            && done.finished
            && !self
                .chronicle
                .iter()
                .any(|e| e.kind == ChronicleKind::FirstSowing && e.settlement == settlement);
        if first {
            let s = &ctx.land.settlements[si];
            let (name, place) = (s.name.clone(), ctx.land.fields[fi].rect.centre_m());
            self.chronicle_push(
                now,
                ChronicleKind::FirstSowing,
                vec![who],
                settlement,
                Some(place),
                0.0,
                name,
            );
        }
    }

    /// Applies `hours` of a capable adult's work by person `h` of `household` to the upkeep of
    /// its finished building at index `bi` (of program `def`): the repair under way, or else one
    /// begun on the worst of its groups that shows ([`condition::repair_of`]), advances as far as
    /// the work and the materials in the household's store allow. The materials it uses leave the
    /// store, and the work counts toward the builder's building skill. Once it is done the group
    /// is mended, and the household's shelter derived again.
    fn mend_work(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        household: PermanentId,
        bi: usize,
        def: &crate::params::BuildingDef,
        hours: f64,
    ) {
        let (now, params, catalog) = (ctx.now, ctx.params, ctx.catalog);
        let goods = &catalog.goods;
        let Some(repair) = condition::repair_of(&ctx.land.buildings[bi], &def.upkeep) else {
            return;
        };
        let Some(e) = self.expansion_of(&ctx.land.buildings[bi], def) else {
            return;
        };
        let Some(needs) = condition::mend_needs(&e, repair.group, f64::from(repair.share)) else {
            return;
        };
        let skill = def
            .skill
            .and_then(|k| catalog.skills.get(k).map(|s| (k, s)));
        let toward = self.hearth_toward(ctx.land, household);
        let Some(&hd) = self.hh_index.get(&household) else {
            return;
        };
        let Some(x) = self.households.get_mut(hd) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let work = HomeWork::mend(needs, f64::from(repair.work_h));
        let Some(stage) = work.needs() else {
            return;
        };
        let held = work.held_by_slot(def, &x.stores);
        let mender_skill = skill
            .and_then(|(k, _)| self.people.get(h).map(|p| p.skill(k)))
            .unwrap_or(condition::MIDDLING_SKILL);
        let b = &mut ctx.land.buildings[bi];
        b.repair = Some(repair);
        let done = b.mend_work(hours, stage.labour_h, &stage.materials_kg, &held);
        use_materials(x, def, &done.used_kg);
        if done.done {
            // A part that gave way is rebuilt whole of new members, as well as its mender can.
            let failed = b
                .group(repair.group)
                .is_some_and(|c| c.state == civ_land::GroupState::Failed);
            let rebuilt = failed.then(|| {
                condition::draw_rebuilt_quality(
                    ctx.seed,
                    b.id,
                    repair.group,
                    now,
                    mender_skill,
                    params.build.quality_spread,
                )
            });
            condition::mend(b, repair.group, repair.share, &def.upkeep, now, rebuilt);
            let plot = b.plot;
            (x.sheltered, x.keeping) = shelter_of(ctx.land, catalog, params, household);
            // Daub renewed is dug beside the building (ADR-0010 §2).
            let mended = e
                .groups
                .iter()
                .find(|g| g.id == repair.group)
                .map_or(0.0, |g| g.daub_m3() * f64::from(repair.share));
            if let Some(toward) = toward {
                digging::dig_daub(ctx, plot, toward, e.daub_m3(None), mended);
            }
        }
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            p.set_skill(k, s.practised(p.skill(k), done.hours));
        }
    }

    /// Applies `hours` of a capable adult's work by person `who` (handle and id) of `household`
    /// to its building `building`: the stage under way advances as far as the work and the
    /// materials in the household's store allow, and the materials it uses leave the store. The
    /// work counts toward the builder's building skill, and toward the quality of the groups the
    /// stage puts in place, drawn when it is finished from how skilled its builders were on
    /// average (ADR-0009 §6). When the roof goes on, the household's stores are under it from
    /// then on.
    fn build_work(
        &mut self,
        ctx: &mut Ctx,
        (h, who): (Handle<Person>, PermanentId),
        household: PermanentId,
        building: PermanentId,
        hours: f64,
    ) {
        let (now, params, catalog) = (ctx.now, ctx.params, ctx.catalog);
        let goods = &catalog.goods;
        let Some(bi) = ctx
            .land
            .buildings
            .iter()
            .position(|b| b.id == building && b.household == household)
        else {
            return;
        };
        let Some(def) = catalog
            .building_index(&ctx.land.buildings[bi].spec.program)
            .and_then(|i| catalog.buildings.get(i))
        else {
            return;
        };
        if ctx.land.buildings[bi].finished() {
            self.mend_work(ctx, h, household, bi, def, hours);
            return;
        }
        let levelling = levelling_h(ctx.land, ctx.land.buildings[bi].plot, params);
        let Some(needs) = self.needs_of(&ctx.land.buildings[bi], def, levelling) else {
            return;
        };
        let expansion = self.expansion_of(&ctx.land.buildings[bi], def);
        let toward = self.hearth_toward(ctx.land, household);
        let skill = def
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let level = match skill {
            Some((k, _)) => self.people.get(h).map_or(0.0, |p| p.skill(k)),
            None => condition::MIDDLING_SKILL,
        };
        let Some(&hd) = self.hh_index.get(&household) else {
            return;
        };
        let Some(x) = self.households.get_mut(hd) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let work = HomeWork::of(&ctx.land.buildings[bi], needs);
        let Some(stage) = work.needs() else {
            return;
        };
        let held = work.held_by_slot(def, &x.stores);
        let b = &mut ctx.land.buildings[bi];
        let done = b.work(hours, stage.labour_h, &stage.materials_kg, &held, now);
        b.skill_h += (done.hours * level) as f32;
        // The earth its walls are daubed with is dug beside it as they go up (ADR-0010 §2).
        let daub = expansion.as_ref().map_or((0.0, 0.0), |e| {
            let share = if stage.labour_h > 0.0 {
                (done.hours / stage.labour_h).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (e.daub_m3(None), e.daub_m3(Some(stage.stage)) * share)
        });
        // Its plot is levelled first, with the first stage's first hours; cutting into a buried
        // deposit finds it (ADR-0010 §1).
        if levelling > 0.0 {
            let (plot, stage, work_h) = (b.plot, b.stage, f64::from(b.work_h));
            let share = if stage == 0 {
                (work_h / levelling).min(1.0)
            } else {
                1.0
            };
            let land = &mut *ctx.land;
            for work in land.earthworks.iter_mut().filter(|w| w.plot == Some(plot)) {
                civ_land::earth::advance(work, ctx.map, &mut land.ground, share as f32);
            }
            let cut: Vec<civ_land::earth::Earthwork> = ctx
                .land
                .earthworks
                .iter()
                .filter(|w| w.plot == Some(plot))
                .copied()
                .collect();
            for work in &cut {
                self.find_by_cutting(ctx, who, household, work);
            }
        }
        if let (Some(toward), true) = (toward, daub.1 > 0.0) {
            let plot = ctx.land.buildings[bi].plot;
            digging::dig_daub(ctx, plot, toward, daub.0, daub.1);
        }
        let Some(x) = self.households.get_mut(hd) else {
            return;
        };
        let b = &mut ctx.land.buildings[bi];
        if let Some(finished) = done.finished {
            // How skilled its builders were on average, over the stage's work.
            let skill = if stage.labour_h > 0.0 {
                (f64::from(b.skill_h) / stage.labour_h).clamp(0.0, 1.0)
            } else {
                level
            };
            if let Ok(e) = civ_grammar::expand(&b.spec, &def.rules) {
                let spread = params.build.quality_spread;
                condition::install(b, &e, finished, skill, ctx.seed, spread, now);
            }
            b.skill_h = 0.0;
        }
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            p.set_skill(k, s.practised(p.skill(k), done.hours));
        }
        use_materials(x, def, &done.used_kg);
        if done.finished == Some(Stage::Finish) {
            // Its lofts and raised floors take goods from now on; stores were settled above.
            (x.sheltered, x.keeping) = shelter_of(ctx.land, ctx.catalog, params, household);
        }
        if done.finished != Some(Stage::Roof) {
            return;
        }
        let settlement = x.settlement;
        // A household that built itself a new home moves in once its roof is on, and the home it
        // leaves is taken down and its ground given up; so is a ruin of the household's that the
        // new building stands in for.
        let is_home = def.use_ == civ_land::PlotUse::Dwelling;
        let dwelling = |b: &Building| catalog.is_dwelling(&b.spec.program);
        let old: Vec<(PermanentId, PermanentId, bool)> = ctx
            .land
            .buildings
            .iter()
            .filter(|b| {
                b.household == household
                    && b.id != building
                    && b.finished()
                    && ((is_home && dwelling(b))
                        || (!b.standing() && catalog.use_of(&b.spec.program) == Some(def.use_)))
            })
            .map(|b| (b.id, b.plot, is_home && dwelling(b)))
            .collect();
        if old.iter().any(|&(_, _, home)| home) {
            x.home = build::centre_m(&ctx.land.buildings[bi].spec);
            self.homes_moved += 1;
        }
        if !old.is_empty() {
            ctx.land
                .buildings
                .retain(|b| !old.iter().any(|&(id, _, _)| id == b.id));
            ctx.land
                .plots
                .retain(|p| !old.iter().any(|&(_, plot, _)| plot == p.id));
            for (id, _, _) in &old {
                self.stage_needs.remove(id);
                self.expansions.remove(id);
            }
        }
        // Stores were settled to now above, so they keep under the new roof from here on.
        if let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            (x.sheltered, x.keeping) = shelter_of(ctx.land, ctx.catalog, params, household);
        }
        let Some(bi) = ctx.land.buildings.iter().position(|b| b.id == building) else {
            return;
        };
        let first = !self
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::FirstRoof && e.settlement == settlement);
        let name = settlement
            .and_then(|s| ctx.land.settlements.iter().find(|x| x.id == s))
            .map(|s| s.name.clone());
        if let (true, Some(name)) = (first, name) {
            let place = build::centre_m(&ctx.land.buildings[bi].spec);
            self.chronicle_push(
                now,
                ChronicleKind::FirstRoof,
                vec![who],
                settlement,
                Some(place),
                0.0,
                name,
            );
        }
    }

    /// After the paths were surveyed: notes in the chronicle each settlement whose first trail out
    /// has been worn in, a trail at least [`FIRST_TRAIL_M`] long that passes within
    /// [`FIRST_TRAIL_NEAR_M`] of its hearth.
    pub fn note_trails(&mut self, land: &Land, now: SimTime) {
        for s in &land.settlements {
            let noted = self
                .chronicle
                .iter()
                .any(|e| e.kind == ChronicleKind::FirstTrail && e.settlement == Some(s.id));
            if noted {
                continue;
            }
            let best = land
                .wear
                .trails()
                .iter()
                .filter(|t| {
                    t.length_m() >= FIRST_TRAIL_M
                        && t.points
                            .windows(2)
                            .any(|w| segment_distance(s.hearth_m, w[0], w[1]) <= FIRST_TRAIL_NEAR_M)
                })
                .max_by(|a, b| a.length_m().total_cmp(&b.length_m()));
            if let Some(t) = best {
                let middle = t.points[t.points.len() / 2];
                self.chronicle_push(
                    now,
                    ChronicleKind::FirstTrail,
                    Vec::new(),
                    Some(s.id),
                    Some(middle),
                    f64::from(t.length_m()),
                    s.name.clone(),
                );
            }
        }
    }

    fn eat(&mut self, ctx: &mut Ctx, h: Handle<Person>, household: PermanentId) {
        let now = ctx.now;
        let params = ctx.params;
        let Some(p) = self.people.get(h) else {
            return;
        };
        let day_kcal = bmr(p, now, params) * 1.6;
        let want =
            (-f64::from(p.energy_kcal)).max(0.0) + day_kcal / 24.0 * params.energy.satiety_hours;
        let reserve_ok = may_eat_reserve(p, now, params);
        let protected =
            farm::protected_seed(self.fields_of(ctx.land, household), &ctx.catalog.crops);
        let Some(&hh) = self.hh_index.get(&household) else {
            return;
        };
        let Some(hh) = self.households.get_mut(hh) else {
            return;
        };
        let goods = &ctx.catalog.goods;
        let members = hh.members.len().max(1);
        hh.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        hh.stores.resize(goods.len(), 0.0);
        // The seed to sow the ground already cropped is set aside before the meal.
        let set_aside =
            protected.map(|(g, kg)| (g, hh.stores.get(g).copied().unwrap_or(0.0).clamp(0.0, kg)));
        if let Some((g, kg)) = set_aside {
            hh.stores[g] -= kg;
        }
        let before = hh.stores.clone();
        let take = eat_from(&mut hh.stores, goods, want, reserve_ok);
        hh.flows
            .add_changes(&before, &hh.stores, Flow::Eaten, Flow::Eaten);
        if let Some((g, kg)) = set_aside {
            hh.stores[g] += kg;
        }
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        p.energy_kcal =
            (f64::from(p.energy_kcal) + take).min(params.energy.max_surplus_kcal) as f32;
        let full = if want > 0.0 {
            (take / want).clamp(0.0, 1.0)
        } else {
            1.0
        };
        p.satiety_until = now.plus_minutes((params.energy.satiety_hours * 60.0 * full) as i64);
    }

    fn deposit(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let load: Load = std::mem::take(&mut p.carrying);
        let target = p.act.target;
        let hh_id = p.household;
        self.changed(hh_id);
        let Some(&own) = self.hh_index.get(&hh_id) else {
            return;
        };
        let settlement = self.households.get(own).and_then(|x| x.settlement);
        if let Some(x) = self.households.get_mut(own) {
            let members = x.members.len().max(1) as f64;
            let litres_day = members * x.water_use(&params.household);
            x.settle_water(now, litres_day);
            x.water_l += f64::from(load.water_l);
        }
        // What its people use from now follows the walk to the water just fetched (ADR-0021 §1):
        // to a well, from home (M6a slice AY, step three).
        if load.water_l > 0.0 {
            let walk = match target {
                Target::Water(cell) => settlement
                    .and_then(|s| self.homes.get(&s))
                    .or_else(|| self.homes.get(&hh_id))
                    .and_then(|f| f.reach.seconds_to(cell as usize))
                    .map(f64::from),
                Target::Well(well) => {
                    let home = self.households.get(own).map(|x| x.home);
                    ctx.land
                        .wells
                        .get(well)
                        .zip(home)
                        .and_then(|(w, home)| Self::well_walk_s(ctx, home, w.rect.centre_m()))
                }
                _ => None,
            };
            if let (Some(s), Some(x)) = (walk, self.households.get_mut(own)) {
                x.water_use_l = params.household.water_use_l(s / 60.0);
                x.water_walk_min = (s / 60.0) as f32;
            }
        }
        let Some(g) = load.good.map(usize::from) else {
            return;
        };
        let Some(good) = goods.get(g) else {
            return;
        };
        if load.kg <= 0.0 {
            return;
        }
        let kg = f64::from(load.kg);
        // The load comes into the household's store.
        let Some(x) = self.households.get_mut(own) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let target = params.household.food_target_days
            * members as f64
            * params.household.daily_kcal_per_person;
        let short = (target - stock_kcal(&x.stores, goods)).max(0.0);
        x.stores[g] += kg;
        x.flows.add(Flow::Got, g, kg);
        // A shared good is shared out of what the household can spare: it first keeps what
        // brings its food to the days it tries to keep, and the rest goes to every household of
        // the settlement, by members (research 06-01 §2.3 and §3.2: help comes from disposable
        // surplus, after a household's own subsistence). In plenty, all of a kill is shared; in
        // hunger, a family keeps what it catches.
        let Some(s) = settlement.filter(|_| good.shared && good.kcal_per_kg > 0.0) else {
            return;
        };
        let spare = kg - kg.min(short / good.kcal_per_kg);
        if spare <= 0.0 {
            return;
        }
        let mut recipients: Vec<(PermanentId, usize)> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(s) && !x.members.is_empty())
            .map(|(_, x)| (x.id, x.members.len()))
            .collect();
        recipients.sort_unstable();
        let total = recipients.iter().map(|r| r.1).sum::<usize>().max(1) as f64;
        let legs: Vec<Leg> = recipients
            .into_iter()
            .filter(|&(id, _)| id != hh_id)
            .map(|(id, n)| Leg {
                from: hh_id,
                to: id,
                good: g,
                amount: spare * n as f64 / total,
            })
            .collect();
        self.transfer(now, params, goods, &legs, Channel::Share);
    }

    /// Days of food in the stores of a settlement's households at its people's needs, at `now`;
    /// `None` if nobody lives there.
    pub fn settlement_food_days(
        &self,
        settlement: PermanentId,
        now: SimTime,
        params: &PeopleParams,
        goods: &[GoodDef],
    ) -> Option<f64> {
        let mut households: Vec<&Household> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
            .collect();
        // A fixed order, so the sum is the same however the table is laid out.
        households.sort_by_key(|x| x.id);
        let (mut kcal, mut members) = (0.0, 0usize);
        for x in households {
            kcal += stock_kcal(&stores_now(x, now, params, goods), goods);
            members += x.members.len();
        }
        (members > 0)
            .then(|| kcal / (members as f64 * params.household.daily_kcal_per_person).max(1.0))
    }

    /// The end of a day: the season turns for every field, harvests and food shortages are noted
    /// in the chronicle (with a gap between running short and recovering, so a store hovering
    /// near the line is not noted every day), and then a day of life: births, deaths, couples and
    /// the households they make (see `life`). Newborns' first decisions go to `ctx.schedule`.
    /// A month of wear on every building with groups in place (ADR-0009 §4), its posts set in
    /// the ground of the habitat it stands in (its `wetness`), as wet as the month just lived was
    /// against its usual (ADR-0012 §5). The households whose roofs leak more, or whose
    /// buildings' state changed, have their stores settled under the roofs they had and their
    /// shelter derived again.
    fn wear_buildings(&mut self, ctx: &mut Ctx) {
        let (map, catalog, land_params) = (ctx.map, ctx.catalog, ctx.land_params);
        let rain = ctx
            .land
            .weather
            .last_month_wetness(&ctx.land.climatology)
            .unwrap_or(1.0);
        let patches = &ctx.land.patches;
        let wetness: Vec<f64> = ctx
            .land
            .buildings
            .iter()
            .map(|b| {
                let cell = cell_of(map, build::centre_m(&b.spec));
                patches
                    .class
                    .get(patches.of_cell(cell, map.width))
                    .and_then(|&c| land_params.habitats.get(usize::from(c)))
                    .map_or(1.0, |h| h.wetness)
            })
            .collect();
        let mut changed: Vec<PermanentId> = Vec::new();
        for (b, wet) in ctx.land.buildings.iter_mut().zip(wetness) {
            let Some(def) = catalog
                .building_index(&b.spec.program)
                .and_then(|i| catalog.buildings.get(i))
            else {
                continue;
            };
            if condition::wear_month(b, &def.upkeep, wet, rain) && !changed.contains(&b.household) {
                changed.push(b.household);
            }
        }
        for household in changed {
            self.buildings_changed(ctx.now, ctx.land, catalog, ctx.params, household);
        }
        self.remember_standing(ctx);
        self.look_over_storehouses(ctx);
    }

    pub fn on_day(&mut self, ctx: &mut Ctx) {
        self.pay_for_work_given(ctx);
        self.close_travel_day(ctx);
        // What each money is worth to those who used it, afresh for the day (M10d slice EF).
        self.money_day(ctx);
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        // Gates are opened at first light and kept at dusk (M6c slice BK).
        self.schedule_gates(ctx);
        // Gas is lit at dusk where mains are laid (M10b slice DY).
        self.schedule_gas(ctx);
        // Households see their options afresh each day (ADR-0011 §4), and a leisure block that
        // runs past midnight ends in a decision made afresh: nobody decides between midnight's
        // day of work and the day's first event, so a save there loses nothing.
        self.views.clear();
        self.switched.clear();
        self.index_fields(ctx.land);
        // On the first of each month, ties to those no longer here are let go and standing is
        // worked out afresh (ADR-0014 §1, §3).
        if now.date().day == 1 {
            self.prune_ties();
            self.prune_beliefs(now, params);
            self.derive_standing(now, params);
            // And where each stands on the questions content names is anchored afresh in their
            // household's lot (M4c slice AG, ADR-0016 §4).
            // Everyone holds each value content names, before their anchors are worked out from
            // them (M4c slice AG).
            self.values_month(ctx);
            self.opinion_month(ctx);
            // Everyone holds a state of each norm content names (M4c slice AG).
            self.norm_month(ctx);
        }
        // The season turns for every field: a sowing window passes, a crop left standing is lost.
        // A sown crop drowned under water is a loss its household saw (M6b slice BG): the seed
        // it was sown with and the work put into it.
        let day = now.day_index();
        let mut drowned: Vec<(PermanentId, crate::person::FloodLoss)> = Vec::new();
        for f in &mut ctx.land.fields {
            if let Some(crop) = ctx.catalog.crops.get(usize::from(f.crop)) {
                let under = f.stage == FieldStage::Sown && f.submerged_days >= 5;
                let loss = crate::person::FloodLoss {
                    day,
                    cell: cell_of(ctx.map, f.rect.centre_m()) as u32,
                    depth_cm: f.last_flood_depth_cm,
                    kcal: (crop.seed_kg_per_ha
                        * f.area_ha()
                        * ctx
                            .catalog
                            .goods
                            .get(crop.seed_good)
                            .map_or(0.0, |g| g.kcal_per_kg)) as f32,
                    hours: f.work_h + f.tended_h,
                };
                if f.new_day(crop, day, now) && under {
                    drowned.push((f.household, loss));
                }
            }
        }
        for (h, loss) in drowned {
            if let Some(&hd) = self.hh_index.get(&h)
                && let Some(x) = self.households.get_mut(hd)
            {
                x.remember_flood_loss(loss);
            }
        }
        // A settlement's harvest is in once none of its fields has a crop growing or unthreshed.
        for i in 0..ctx.land.settlements.len() {
            let s = &ctx.land.settlements[i];
            if s.harvest_kg <= 0.0 {
                continue;
            }
            let id = s.id;
            let busy = ctx.land.fields.iter().any(|f| {
                matches!(f.stage, FieldStage::Sown | FieldStage::Reaped)
                    && self.household(f.household).and_then(|h| h.settlement) == Some(id)
            });
            if busy {
                continue;
            }
            let s = &mut ctx.land.settlements[i];
            let (kg, name, place) = (s.harvest_kg, s.name.clone(), s.hearth_m);
            s.harvest_kg = 0.0;
            self.chronicle_push(
                now,
                ChronicleKind::HarvestIn,
                Vec::new(),
                Some(id),
                Some(place),
                kg,
                name,
            );
        }
        for i in 0..ctx.land.settlements.len() {
            let id = ctx.land.settlements[i].id;
            let Some(days) = self.settlement_food_days(id, now, params, goods) else {
                continue;
            };
            let s = &mut ctx.land.settlements[i];
            let Some(kind) = shortage_change(s.food_short, days, &params.household) else {
                continue;
            };
            s.food_short = kind == ChronicleKind::FoodRanShort;
            let (name, place) = (s.name.clone(), s.hearth_m);
            self.chronicle_push(now, kind, Vec::new(), Some(id), Some(place), days, name);
        }
        // Each settlement's polity: founded under the custom, its gathering decided, word of its
        // laws gone round, its review held (ADR-0013).
        self.polity_day(ctx);
        // The fields under plans in force that take or pay for them (M8c slice CU, fifth part).
        self.claim_plan_fields(ctx);
        // Takings found, word of them gone round, and what households taken from choose and are
        // owed (ADR-0015).
        self.crime_day(ctx);
        self.band_day(ctx);
        // The yearly land review (ADR-0007 §2): fields given out by need, or ground nobody holds
        // any more taken up.
        if day.rem_euclid(365) == i64::from(ctx.regime.review_day) {
            let ids: Vec<PermanentId> = ctx.land.settlements.iter().map(|s| s.id).collect();
            for s in ids {
                self.review_land(ctx, s);
            }
        }
        // A month of weather on every building, on the first of the month (ADR-0009 §4), and
        // each day every building weighed against what it carries (§5). What stood out in the
        // weather just lived goes into the chronicle (ADR-0012).
        if now.date().day == 1 {
            self.wear_buildings(ctx);
            // A month's rot at the foot of every palisade's posts (M6c slice BK).
            self.wear_enclosures(ctx);
            // Each household's midden grows with its members and wastes (M3c slice V).
            for (_, h) in self.households.iter_mut() {
                let members = h.members.len();
                h.midden.settle(now, members, &params.midden);
            }
            let notes = ctx
                .land
                .weather
                .notes(&ctx.land_params.weather, &ctx.land.climatology);
            for note in notes {
                self.chronicle_push(
                    now,
                    ChronicleKind::Weather,
                    Vec::new(),
                    None,
                    None,
                    f64::from(note.flags),
                    note.words,
                );
            }
        }
        self.floods_day(ctx);
        self.check_buildings(ctx);
        // The minutes at which each home's fires in use may escape today (M6b slice BD).
        self.plan_ignitions(ctx, day);
        self.fire_month(ctx);
        // And the crossings over water (M5c slice AW): rot and failure, then the households
        // whose day it is weigh building one where their people wade.
        if !ctx.land.crossings.list.is_empty() {
            self.crossings_day(ctx);
        }
        if !self.fords.households.is_empty() {
            self.crossing_reviews(ctx, day);
        }
        // Lines of track weighed and trains timetabled (M10b slices DV and DW), every day
        // whether or not anyone has waded (M10b slice DX, second part: they had been weighed only
        // where someone had).
        self.line_reviews(ctx, day);
        self.engine_runs(ctx, day);
        // Gas mains weighed by the works whose day it is (M10b slice DY, fourth part).
        self.main_reviews(ctx, day);
        // A child sent to learn a craft the household buys (M10c slice EB, second part).
        self.enrol_reviews(ctx, day);
        // And the wells (M6a slice AY, step three): what was drawn taken from the water table, a
        // month's rot on their linings, then the households whose day it is weigh one.
        if !ctx.land.wells.list.is_empty() {
            self.wells_day(ctx);
            if now.date().day == 1 {
                self.wells_month(ctx);
            }
        }
        self.well_reviews(ctx, day);
        // And the ditches (M6b slice BG): the households whose day it is weigh one, and every
        // ditch's intake is opened or shut for the day.
        self.ditches_day(ctx, day);
        // And the mills (M10a slice DR): the households whose day it is weigh one.
        self.mills_day(ctx, day);
        // And the engines at pits pump them (M10a slice DS, third part); the households whose
        // day it is weigh one (its fourth part).
        self.engine_reviews(ctx, day);
        // Water wheels pump their holders' pits near them first (M10a slice DT, fifth part).
        if !ctx.land.installations.is_empty() && !ctx.land.pit_water.is_empty() {
            self.wheels_day(ctx);
        }
        if !ctx.land.engines.is_empty() {
            self.engines_day(ctx);
        }
        // A month ended: its asks and prices per pair of settlements are recorded (M5b slice AQ).
        self.record_convergence(ctx);
        // Households whose day it is review what they offer and on what terms.
        self.review_offers(ctx, day);
        // Goods kept for households with no roof of their own (M10d slice EG, second part).
        self.custody_day(ctx, day);
        // Those that found a boat would save them more walking over its life than making it
        // takes begin one (M8b slice CO, third part).
        self.begin_boats(ctx);
        // Households whose day it is weigh borrowing, and loans due are paid (M7d slice CA).
        self.credit_day(ctx, day);
        // Households bring metal to their polity's mint (M7d slice CE2b).
        self.bring_to_mints(ctx);
        self.reconcile_coin(ctx);
        // A workshop whose firm closed goes to another firm of its household that has none.
        self.hand_on_workshops(ctx.land, ctx.catalog);
        // Births, deaths, couples and the households they make.
        self.live_day(ctx);
        // Children who have reached the age of their household's work learn it (ADR-0008 §4).
        self.bring_up(ctx);
        // The households of a migration wave whose day it is come (M5a slice AN).
        if !self.influences.waves.is_empty() {
            crate::found::wave_arrivals(self, ctx);
        }
        // Hours given to common works today are paid for (M7d slice CE3a).
        self.pay_for_work_given(ctx);
    }
}

/// Whether a settlement's food has just run short, or just recovered, given whether it was short
/// and its days of food now. The two lines are apart, so a store hovering near one of them is not
/// noted every day.
pub fn shortage_change(short: bool, days: f64, h: &HouseholdParams) -> Option<ChronicleKind> {
    if !short && days < h.short_food_days {
        Some(ChronicleKind::FoodRanShort)
    } else if short && days > h.recovered_food_days {
        Some(ChronicleKind::FoodRecovered)
    } else {
        None
    }
}

/// Takes up to `want` kcal from the food in store that can be eaten as it is, the most perishable
/// first and goods kept back last of all and only when `reserve` allows, and only what needs no
/// fire when there is no firewood. Food a recipe must make ready first (grain, flour) is never
/// eaten. Returns the kcal taken.
pub fn eat_from(stores: &mut [f64], goods: &[GoodDef], want: f64, reserve: bool) -> f64 {
    let fire = fuel_kg(stores, goods) > 0.0;
    let keeps = |g: &GoodDef| {
        if g.half_life_days > 0.0 {
            g.half_life_days
        } else {
            f64::INFINITY
        }
    };
    let mut order: Vec<usize> = (0..goods.len().min(stores.len()))
        .filter(|&g| {
            let good = &goods[g];
            good.edible() && (fire || !good.cooked()) && (reserve || !good.kept_back())
        })
        .collect();
    order.sort_by(|&a, &b| {
        let (ga, gb) = (&goods[a], &goods[b]);
        ga.kept_back()
            .cmp(&gb.kept_back())
            .then(keeps(ga).total_cmp(&keeps(gb)))
            .then(a.cmp(&b))
    });
    let mut taken = 0.0;
    for g in order {
        let need = want - taken;
        if need <= 0.0 {
            break;
        }
        let have = stores[g].max(0.0) * goods[g].kcal_per_kg;
        let take = have.min(need);
        stores[g] -= take / goods[g].kcal_per_kg;
        taken += take;
    }
    taken
}

/// Lays out the aligned blocks of side `2·range + 1` patches and the cell people walk to in each.
fn lay_out_places(ctx: &Ctx, range: u32, in_water: bool) -> Places {
    let map = ctx.map;
    let patches = &ctx.land.patches;
    let (pc, side) = (patches.patch_cells, 2 * range + 1);
    let (w, h) = (i64::from(map.width), i64::from(map.height));
    let by_water = |i: usize| {
        let (x, y) = ((i as i64) % w, (i as i64) / w);
        civ_world::grid::D8.iter().any(|&(dx, dy)| {
            let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
            nx >= 0
                && ny >= 0
                && nx < w
                && ny < h
                && map.water[(ny * w + nx) as usize] != civ_world::WATER_LAND
        })
    };
    let (cols, rows) = (patches.cols.div_ceil(side), patches.rows.div_ceil(side));
    let mut blocks = Vec::with_capacity((cols * rows) as usize);
    for by in 0..rows {
        for bx in 0..cols {
            let cpx = (bx * side + range).min(patches.cols - 1);
            let cpy = (by * side + range).min(patches.rows - 1);
            let centre = cpy * patches.cols + cpx;
            let (cx, cy) = (cpx * pc + pc / 2, cpy * pc + pc / 2);
            let mut best: Option<(u32, u32)> = None;
            for y in by * side * pc..((by + 1) * side * pc).min(map.height) {
                for x in bx * side * pc..((bx + 1) * side * pc).min(map.width) {
                    let i = y as usize * map.width as usize + x as usize;
                    if map.water[i] != civ_world::WATER_LAND {
                        continue;
                    }
                    let d = x.abs_diff(cx).pow(2) + y.abs_diff(cy).pow(2);
                    if best.is_none_or(|(bd, _)| d < bd) && (!in_water || by_water(i)) {
                        best = Some((d, i as u32));
                    }
                }
            }
            blocks.push(best.map(|(_, cell)| (centre, cell)));
        }
    }
    Places {
        range,
        in_water,
        blocks,
    }
}

/// The springs that could rise within `field`'s reach nearer than `bank_s` seconds (M6a slice
/// AY): each patch off the water whose lowest ground is reachable, with the walk to it, nearest
/// first. Which of them flow changes with the heads, so it is asked when water is fetched.
fn springs_in_reach(land: &Land, field: &TravelField, bank_s: f32) -> Vec<(f32, u32, u32)> {
    let aq = &land.water.aquifer;
    let mut out: Vec<(f32, u32, u32)> = (0..aq.len())
        .filter_map(|p| {
            let (cell, _) = aq.seep[p]?;
            if aq.stage[p].is_some() {
                return None;
            }
            let s = field.seconds_to(cell as usize)?;
            (s < bank_s).then_some((s, p as u32, cell))
        })
        .collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    out
}

/// The cells in `field` beside a river or lake (not the sea) that people may draw at, nearest
/// in walking time first (then by cell): the nearest, and up to [`BANKS_KNOWN`] further ones,
/// each at least [`BANK_APART_M`] from every one before it, so that each is its own stretch of
/// the water's edge (M6a slice BA, step four).
fn water_edges(map: &WorldMap, field: &TravelField) -> Vec<(u32, f32)> {
    let (w, h) = (map.width as i64, map.height as i64);
    let mut edges: Vec<(u32, f32)> = field
        .iter()
        .filter(|&(i, _)| {
            let (x, y) = ((i as i64) % w, (i as i64) / w);
            map.water[i] == WATER_RIVER
                || civ_world::grid::D8.iter().any(|&(dx, dy)| {
                    let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
                    nx >= 0
                        && ny >= 0
                        && nx < w
                        && ny < h
                        && matches!(map.water[(ny * w + nx) as usize], WATER_RIVER | WATER_LAKE)
                })
        })
        .map(|(i, s)| (i as u32, s))
        .collect();
    edges.sort_unstable_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    let apart = BANK_APART_M / map.cell_size_m.max(1e-3);
    let mut out: Vec<(u32, f32)> = Vec::new();
    for e in edges {
        if out.len() > BANKS_KNOWN {
            break;
        }
        let (x, y) = ((i64::from(e.0) % w) as f32, (i64::from(e.0) / w) as f32);
        let far = out.iter().all(|o| {
            let (ox, oy) = ((i64::from(o.0) % w) as f32, (i64::from(o.0) / w) as f32);
            (x - ox).hypot(y - oy) >= apart
        });
        if far {
            out.push(e);
        }
    }
    out
}

/// The kilowatts the installations `holder` owns deliver today (M10a slice DR; ADR-0033 §5):
/// the sum of their powers from the day's river. Nothing for a holder that owns none.
pub(crate) fn power_kw_of(ctx: &Ctx, holder: PermanentId) -> f64 {
    ctx.land
        .installations
        .iter()
        .enumerate()
        .filter(|(_, x)| x.owner == holder)
        .map(|(i, _)| ctx.land.installation_kw(ctx.land_params, i))
        .sum()
}

/// The lamp a household holding `stores` lights for work after dark (M10b slice DU; ADR-0034
/// §9): of the whole lamps it holds whose fuel it has (a piped burner is none: its gas comes by a
/// main, M10b slice DY), the one whose fuel lasts longest, as the
/// lamp and its fuel by good index, the kilograms it burns an hour and the minutes the fuel
/// held lasts.
pub fn lamp_of(goods: &[GoodDef], stores: &[f64]) -> Option<(usize, usize, f64, f64)> {
    goods
        .iter()
        .enumerate()
        .filter_map(|(g, d)| {
            let light = d.light.filter(|l| !l.piped)?;
            let fuel = light.fuel?;
            let held = stores.get(g).copied().unwrap_or(0.0);
            if light.kg_per_hour <= 0.0 || make::in_use(goods, g, held) < decide::MIN_TOOL {
                return None;
            }
            let kg = stores.get(fuel).copied().unwrap_or(0.0).max(0.0);
            (kg > 0.0).then_some((g, fuel, light.kg_per_hour, kg / light.kg_per_hour * 60.0))
        })
        .max_by(|a, b| a.3.total_cmp(&b.3).then(b.0.cmp(&a.0)))
}

/// What an hour of light good `g` costs a household whose own costs, hours a unit, are `costs`
/// (M10b slice DU, fourth part; brief §2.3): the fuel it burns at the household's cost of it, and
/// the light's own cost spread over its life where the household can put a figure on it. `None`
/// where it is no light, or the household can put no figure on its fuel.
pub(crate) fn light_hour_cost(goods: &[GoodDef], costs: &[Option<f64>], g: usize) -> Option<f64> {
    let light = goods.get(g)?.light?;
    let fuel = light.fuel.and_then(|f| costs.get(f).copied().flatten())?;
    let life = make::life_of(goods, g);
    let own = costs.get(g).copied().flatten().unwrap_or(0.0);
    Some(light.kg_per_hour * fuel + if life > 0.0 { own / life } else { 0.0 })
}

/// Minutes of the `minutes` before `end` that fell in the dark, sunrise and sunset taken day by
/// day at the profile's latitude (M10b slice DU).
pub(crate) fn dark_minutes(params: &PeopleParams, end: SimTime, minutes: u32) -> f64 {
    let end = end.minutes();
    let mut t = end - i64::from(minutes);
    let mut dark = 0i64;
    let overlap = |a: i64, b: i64, lo: i64, hi: i64| (b.min(hi) - a.max(lo)).max(0);
    while t < end {
        let day = t.div_euclid(MINUTES_PER_DAY);
        let start = day * MINUTES_PER_DAY;
        let next = (start + MINUTES_PER_DAY).min(end);
        let sun = needs::daylight(params.latitude_deg, civ_land::day_of_year(day));
        dark += overlap(t, next, start, start + sun.0)
            + overlap(t, next, start + sun.1, start + MINUTES_PER_DAY);
        t = next;
    }
    dark as f64
}

/// Whether it is dark at `now`, between sunset and sunrise at the profile's latitude.
pub fn dark_at(params: &PeopleParams, now: SimTime) -> bool {
    let sun = needs::daylight(params.latitude_deg, civ_land::day_of_year(now.day_index()));
    let minute = now.minute_of_day();
    minute < sun.0 || minute >= sun.1
}

/// The crew recipe `recipe`'s tools need: the most hands any of them wants at it together (M10a
/// slice DT; ADR-0033 §8), 1 for a recipe of hand tools.
pub(crate) fn crew_of(goods: &[GoodDef], recipe: &crate::params::RecipeDef) -> u8 {
    recipe
        .tools
        .iter()
        .filter_map(|&t| goods.get(t)?.tool.as_ref().map(|x| x.crew))
        .max()
        .unwrap_or(1)
        .max(1)
}

/// Caps `holder`'s want of each powered tool at the places it has to set one: a wheel at the
/// installations it owns whose headraces are dug (M10a slice DR), an engine at the pits it has
/// chosen to set one at (M10a slice DS, fourth part). A wheel or an engine is wanted only where
/// one can be set.
pub(crate) fn cap_power_wants(ctx: &Ctx, holder: PermanentId, wants: &mut [f64]) {
    use crate::params::PowerSource;
    let (mut sites, mut pits) = (None, None);
    for (g, want) in wants.iter_mut().enumerate() {
        let Some(source) = ctx
            .catalog
            .goods
            .get(g)
            .and_then(|x| x.power.as_ref())
            .map(|p| p.source)
        else {
            continue;
        };
        let n = match source {
            PowerSource::Water => *sites.get_or_insert_with(|| {
                ctx.land
                    .installations
                    .iter()
                    .enumerate()
                    .filter(|&(i, x)| x.owner == holder && ctx.land.installation_ready(i))
                    .count()
            }),
            PowerSource::Fuel => *pits.get_or_insert_with(|| {
                ctx.land
                    .engines
                    .iter()
                    .filter(|e| e.owner == holder)
                    .count()
            }),
        };
        *want = want.min(n as f64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Eaten, GoodUse};

    #[test]
    fn water_use_follows_the_walk_within_the_curve_and_holds_beyond_it() {
        let mut h = household();
        assert_eq!(h.water_use_l(40.0), h.water_l_per_person_day);
        h.water_use_by_walk = vec![(15.0, 20.0), (30.0, 10.0)];
        assert_eq!(h.water_use_l(0.0), 20.0);
        assert_eq!(h.water_use_l(15.0), 20.0);
        assert!((h.water_use_l(22.5) - 15.0).abs() < 1e-12);
        assert_eq!(h.water_use_l(30.0), 10.0);
        assert_eq!(h.water_use_l(90.0), 10.0);
    }

    fn household() -> HouseholdParams {
        HouseholdParams {
            water_l_per_person_day: 20.0,
            carry_water_l: 15.0,
            water_use_by_walk: Vec::new(),
            water_target_days: 1.5,
            food_target_days: 5.0,
            carry_kg: 20.0,
            fuel_kg_per_person_day: [1.5; 12],
            fuel_target_days: 3.0,
            short_food_days: 2.0,
            recovered_food_days: 5.0,
            daily_kcal_per_person: 2100.0,
            leave_at_depletion: 0.3,
            leave_per_day: 0.1,
            leave_unless_ripe_within_days: 30.0,
            leave_w_gap: 6.0,
            leave_w_stake: 3.0,
            leave_stay: 2.0,
            ready_food_days: 2.0,
            harvest_margin_days: 30.0,
            raised_store_factor: 2.0,
            processed_food_days: 5.0,
        }
    }

    #[test]
    fn shortages_are_noted_once_until_food_has_clearly_recovered() {
        let h = household();
        let mut short = false;
        let mut notes = Vec::new();
        for days in [9.0, 3.0, 1.9, 1.0, 3.0, 4.9, 1.5, 5.1, 6.0, 1.9] {
            if let Some(kind) = shortage_change(short, days, &h) {
                short = kind == ChronicleKind::FoodRanShort;
                notes.push((days, kind));
            }
        }
        assert_eq!(
            notes,
            vec![
                (1.9, ChronicleKind::FoodRanShort),
                (5.1, ChronicleKind::FoodRecovered),
                (1.9, ChronicleKind::FoodRanShort),
            ]
        );
    }

    fn goods() -> Vec<GoodDef> {
        let good = |id: &str, purpose, kcal, half, eaten| GoodDef {
            fuel: None,
            power: None,
            light: None,
            holder: None,
            money: None,
            id: id.into(),
            name: id.into(),
            purpose,
            kcal_per_kg: kcal,
            half_life_days: half,
            eaten,
            shared: false,
            reserve_for: None,
            tool: None,
            sheltered_half_life_days: 0.0,
            timber: None,
            store: None,
            fire: None,
        };
        vec![
            good("bread", GoodUse::Food, 2500.0, 1000.0, Eaten::Raw),
            good("meat", GoodUse::Food, 1500.0, 3.0, Eaten::Cooked),
            good("berries", GoodUse::Food, 600.0, 5.0, Eaten::Raw),
            good("wood", GoodUse::Fuel, 0.0, 0.0, Eaten::Never),
            good("grain", GoodUse::Food, 3300.0, 1500.0, Eaten::Never),
        ]
    }

    #[test]
    fn meals_come_from_the_most_perishable_food_and_cooked_food_needs_firewood() {
        let goods = goods();
        // Meat spoils fastest, then berries; the bread keeps longest of what is ready, and the
        // grain must be ground or pounded first.
        let mut stores = vec![10.0, 2.0, 1.0, 5.0, 100.0];
        let taken = eat_from(&mut stores, &goods, 3600.0, false);
        assert_eq!(taken, 3600.0);
        assert!((stores[1] - 0.0).abs() < 1e-12, "meat first: {stores:?}");
        assert!((stores[2] - 0.0).abs() < 1e-12, "then berries: {stores:?}");
        assert!((stores[0] - (10.0 - 0.0)).abs() < 1e-12, "bread untouched");
        // Without firewood the meat cannot be eaten, nor ever the grain.
        let mut cold = vec![1.0, 2.0, 0.0, 0.0, 100.0];
        let taken = eat_from(&mut cold, &goods, 100_000.0, false);
        assert_eq!(taken, 2500.0, "only the bread");
        assert_eq!(cold[1], 2.0);
        assert_eq!(cold[4], 100.0);
    }
}
