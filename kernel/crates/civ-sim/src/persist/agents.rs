//! Save sections for land and people (ADR-0003 §4, ADR-0004 §6), new in schema version 2; goods
//! came in version 3.
//!
//! | Section | Holds |
//! |---|---|
//! | `land` | habitat patches, richness, wild stocks, the weather (schema 24; before it, the climate year) |
//! | `settle` | settlements |
//! | `people` | living people and their activities and trips |
//! | `houses` | households and their stores |
//! | `history` | everyone who ever lived here, and the chronicle |
//! | `receipts` | recent decision receipts per person |
//! | `events` | the scheduler's pending events |
//! | `fields` | fields and their crops (schema 4) |
//! | `plots` | ground households have claimed (schema 5) |
//! | `builds` | buildings: their designs and how far their construction has gone (schema 5) |
//! | `wear` | ground worn by walking (schema 7) |
//! | `market` | each settlement's market: what sold, for what, and what found no seller (schema 10) |
//! | `firms` | every firm there has been: its record, stores, terms and books (schema 11) |
//! | `wealth` | each settlement's wealth measures at the end of each year (schema 12) |
//!
//! Activities, habitats, resources and goods are saved by content id, so a world still loads after
//! the content adds, removes or reorders them. Where saved state names something the loaded
//! content no longer has, the loader adapts in the plainest way: a person whose activity is gone
//! decides again, a habitat that is gone becomes the catch-all habitat, a resource that is gone is
//! dropped and a new one starts at equilibrium, and a good that is gone is dropped from stores and
//! loads. A resource whose good or unit changed also starts at equilibrium: its saved numbers may
//! count something else.
//!
//! **Schema 2 → 3.** A schema-2 save (M1 slice A) counted food in kilocalories only. It loads as
//! follows: a household's food and food being carried become the band's provisions good, land
//! stocks start at equilibrium (their units are not recorded), and remembered patches are
//! forgotten (their returns were in kilocalories). Nothing else changes.
//!
//! **Schema 3 → 4.** Fields arrived with version 4; a version-2 or 3 save has none. A field whose
//! crop the loaded content no longer has is dropped.
//!
//! **Schema 4 → 5.** Plots and buildings arrived with version 5; an older save has none, and its
//! households are not yet under a roof. A building keeps its design whatever the loaded content
//! says; one whose program the content no longer has stands as it is, and work on it stops.
//! Whether a household's stores are under a roof is not saved: it follows from its buildings.
//!
//! **Schema 5 → 6.** Couples, pregnancies and unions arrived with version 6. In an older save the
//! couple at the head of each household (a woman and a man who share a child, or else the first
//! unrelated woman and man it lists) become partners, every woman draws her lasting
//! fecundability, and nobody is pregnant or nursing yet. A field, plot or building whose
//! household is no more stands abandoned.
//!
//! **Schema 9 → 10.** Markets arrived with version 10 (M3a slice I): households' offers and each
//! settlement's market. An older save has none: households post their offers at their next
//! review, and markets start empty. An offer, remembered terms or a remembered trade in a good
//! the loaded content no longer has is dropped.
//!
//! **Schema 10 → 11.** Firms arrived with version 11 (M3a slice J); an older save has none, and a
//! household sets up a workshop when it next makes something to sell. A firm's line, stock, book
//! entry or month line in a good the loaded content no longer has is dropped.
//!
//! **Schema 11 → 12.** Property regimes arrived with version 12 (M3a slice K, ADR-0007): the
//! world's regime in its metadata, and each field's holder and lease. An older save loads under the
//! content's default regime, every field held by the household that works it. The `wealth`
//! section came later in the same slice and is read when present: a save without it (older, or
//! made while the slice was under way) starts the yearly history afresh. The history is a measure,
//! never an input to behaviour, so nothing a world does depends on it.
//!
//! **Schema 23 → 24.** Weather arrived with version 24 (M3c slice U, ADR-0012): the `land`
//! section's weather (today's, what tomorrow depends on, snow lying by height, the soil under the
//! wild cover, each month's record) and each field's water. An older save's weather starts with
//! the old year's climate deviate as its slow anomaly, the soil at field capacity and no snow; a
//! crop growing then is at field capacity and unstressed so far (ADR-0012 §6).

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::{Read, Seek};

use civ_agents::caution::{FireTrust, Trust};
use civ_agents::condition;
use civ_agents::firm::{BookKind, Books, Entry as BookEntryOf, Exit, Firm, Statement, WageOffer};
use civ_agents::history::{
    ChronicleEvent, ChronicleKind, Origin, PersonRecord, ResidenceWhy, Stay,
};
use civ_agents::knowledge::{KnowledgeEvent, KnowledgeEventKind};
use civ_agents::ledger::{Channel, Trade};
use civ_agents::market::{Market, MonthOfTrade, Offer};
use civ_agents::person::{Know, KnowSource};
use civ_agents::population::DepositKnown;
use civ_agents::seats::Seats;
use civ_agents::wealth::{Spread, WealthYear};
use civ_agents::{
    Activity, AgentEvent, Cause, Household, KnownPatch, Load, Person, Population, Reason, Receipt,
    Repro, Scored, Sex, Step, Target, Term, Traits, Trip, Union,
};
use civ_core::scheduler::PendingEvent;
use civ_core::{PermanentId, SimTime};
use civ_grammar::{BuildingSpec, Footprint, PARAMS};
use civ_land::deposits::{Body, Deposit};
use civ_land::soil::RECORD_KEPT;
use civ_land::{
    Building, BuildingState, Climatology, Field, FieldSoil, FieldStage, Founding, GroupCondition,
    GroupState, HarvestRecord, Land, Lease, Limit, MonthRecord, Party, Patches, Plot, PlotUse,
    RectCm, Repair, Settlement, ViewTile, Wear, WearTile, Weather, WeatherDay,
};
use civ_schema::SAVE_SCHEMA_VERSION;
use civ_schema::flatbuffers::{self, FlatBufferBuilder, WIPOffset};
use civ_schema::save;
use civ_world::WorldMap;
use commons_persist::{SectionData, SectionTag, SnapshotReader};

use super::{
    LoadError, SCHEMA_V2, SCHEMA_V3, SCHEMA_V4, SCHEMA_V5, SCHEMA_V6, SCHEMA_V7, SCHEMA_V8,
    SCHEMA_V9, SCHEMA_V10, SCHEMA_V11, SCHEMA_V12, SCHEMA_V13, SCHEMA_V14, SCHEMA_V15, SCHEMA_V16,
    SCHEMA_V17, SCHEMA_V18, SCHEMA_V19, SCHEMA_V20, SCHEMA_V21, SCHEMA_V22, SCHEMA_V23, SCHEMA_V24,
    SCHEMA_V25, SCHEMA_V26, SCHEMA_V27, SCHEMA_V28, SCHEMA_V29, SCHEMA_V30, SCHEMA_V31, SCHEMA_V32,
    SCHEMA_V33, SCHEMA_V34, SCHEMA_V35, SCHEMA_V36, SCHEMA_V37, SCHEMA_V38, SCHEMA_V39, SCHEMA_V40,
    SCHEMA_V41, SCHEMA_V42, SCHEMA_V43, SCHEMA_V44, SCHEMA_V45, SCHEMA_V46, SCHEMA_V47, SCHEMA_V48,
    SCHEMA_V49, SCHEMA_V50, SCHEMA_V51, SCHEMA_V52, SCHEMA_V53, SCHEMA_V54, SCHEMA_V55, SCHEMA_V56,
    SCHEMA_V57, SCHEMA_V58, SCHEMA_V59, SCHEMA_V60, SCHEMA_V61, SCHEMA_V62, SCHEMA_V63, SCHEMA_V64,
    SCHEMA_V65, SCHEMA_V66, SCHEMA_V67, SCHEMA_V68, SCHEMA_V69, SCHEMA_V70, SCHEMA_V71, SCHEMA_V72,
    SCHEMA_V73, SCHEMA_V74, SCHEMA_V75, SCHEMA_V76, SCHEMA_V77, SCHEMA_V78, SCHEMA_V79, SCHEMA_V80,
    SCHEMA_V81, SCHEMA_V82, SCHEMA_V83, SCHEMA_V84, SCHEMA_V85, SCHEMA_V86, SCHEMA_V87, SCHEMA_V88,
    SCHEMA_V89, SCHEMA_V90, SCHEMA_V91, SCHEMA_V92, SCHEMA_V93, SCHEMA_V94, SCHEMA_V95, SCHEMA_V96,
    SCHEMA_V97, SCHEMA_V98, SCHEMA_V99, SCHEMA_V100, SCHEMA_V101, SCHEMA_V102, SCHEMA_V103,
    SCHEMA_V104, SCHEMA_V105, SCHEMA_V106, SCHEMA_V107, SCHEMA_V108, SCHEMA_V109, SCHEMA_V110,
    SCHEMA_V111, SCHEMA_V112, SCHEMA_V113, SCHEMA_V114, SCHEMA_V115, SCHEMA_V116, SCHEMA_V117,
    SCHEMA_V118, SCHEMA_V119, SCHEMA_V120, SCHEMA_V121, SCHEMA_V122, SCHEMA_V123, SCHEMA_V124,
    SCHEMA_V125, SCHEMA_V126, SCHEMA_V127, SCHEMA_V128, SCHEMA_V129, SCHEMA_V130, SCHEMA_V131,
    SCHEMA_V132, SCHEMA_V133, SCHEMA_V134, SCHEMA_V135, SCHEMA_V136, SCHEMA_V137, SCHEMA_V138,
    SCHEMA_V139, SCHEMA_V140, SCHEMA_V141, SCHEMA_V142, SCHEMA_V143, SCHEMA_V144, SCHEMA_V145,
    SCHEMA_V146, SCHEMA_V147, SCHEMA_V148, SCHEMA_V149, SCHEMA_V150, SCHEMA_V151, SCHEMA_V152,
    SCHEMA_V153, SCHEMA_V154, SCHEMA_V155, SCHEMA_V156, SCHEMA_V157, SCHEMA_V158, SCHEMA_V159,
    SCHEMA_V160, SCHEMA_V161, SCHEMA_V162, SCHEMA_V163, SCHEMA_V164, SCHEMA_V165, SCHEMA_V166,
    SCHEMA_V167, SCHEMA_V168, SCHEMA_V169, SCHEMA_V170, SCHEMA_V171, SCHEMA_V172, SCHEMA_V173,
    SCHEMA_V174, SCHEMA_V175, SCHEMA_V176, SCHEMA_V177, SCHEMA_V178, SCHEMA_V179, SCHEMA_V180,
    SCHEMA_V181, SCHEMA_V182, SCHEMA_V183, SCHEMA_V184, SCHEMA_V185, SCHEMA_V186, SCHEMA_V187,
    SCHEMA_V188, SCHEMA_V189, SCHEMA_V190, SCHEMA_V191, SCHEMA_V192, SCHEMA_V193, SCHEMA_V194,
    SCHEMA_V195, SCHEMA_V196, SCHEMA_V197, SCHEMA_V198, SCHEMA_V199, SCHEMA_V200, SCHEMA_V201,
    finish, section, section_root, single_chunk, unreadable,
};
use crate::{Rules, Sim, SimEvent};

/// Section: habitat patches and wild stocks.
pub const SECTION_LAND: SectionTag = SectionTag::new("land");
/// Section: settlements.
pub const SECTION_SETTLE: SectionTag = SectionTag::new("settle");
/// Section: living people.
pub const SECTION_PEOPLE: SectionTag = SectionTag::new("people");
/// Section: households.
pub const SECTION_HOUSES: SectionTag = SectionTag::new("houses");
/// Section: person records and the chronicle.
pub const SECTION_HISTORY: SectionTag = SectionTag::new("history");
/// Section: decision receipts.
pub const SECTION_RECEIPTS: SectionTag = SectionTag::new("receipts");
/// Section: pending scheduled events.
pub const SECTION_EVENTS: SectionTag = SectionTag::new("events");
/// Section: fields (schema 4).
pub const SECTION_FIELDS: SectionTag = SectionTag::new("fields");
/// Section: plots (schema 5).
pub const SECTION_PLOTS: SectionTag = SectionTag::new("plots");
/// Section: buildings (schema 5).
pub const SECTION_BUILDS: SectionTag = SectionTag::new("builds");
/// Section (schema 7): ground worn by walking.
pub const SECTION_WEAR: SectionTag = SectionTag::new("wear");
/// Section (schema 10): each settlement's market.
pub const SECTION_MARKET: SectionTag = SectionTag::new("market");
/// Section (schema 11): firms.
pub const SECTION_FIRMS: SectionTag = SectionTag::new("firms");
/// Section (schema 12): each settlement's yearly wealth measures.
pub const SECTION_WEALTH: SectionTag = SectionTag::new("wealth");
/// Section "know" (schema 13): each settlement's record of the techniques it came to know and
/// lost.
pub const SECTION_KNOW: SectionTag = SectionTag::new("know");
/// Section "deposits" (schema 19): bodies in the ground and what has been taken from each.
pub const SECTION_DEPOSITS: SectionTag = SectionTag::new("deposits");
/// Section "earth" (schema 20): earthworks and what they have done to the ground.
pub const SECTION_EARTH: SectionTag = SectionTag::new("earth");
/// Section "ties" (schema 28): what each person remembers of others (ADR-0014).
pub const SECTION_TIES: SectionTag = SectionTag::new("ties");
/// Section: each settlement's polity, its laws and its store (schema 29, ADR-0013).
pub const SECTION_POLITY: SectionTag = SectionTag::new("polity");
/// Section: takings, what people believe of them, and what households owe (schema 31, ADR-0015).
pub const SECTION_ORDER: SectionTag = SectionTag::new("order");
/// Section: what people have heard and the grievances they hold (M4c slice AE, ADR-0016).
pub const SECTION_WORD: SectionTag = SectionTag::new("word");
/// Section: where people stand on the questions content names (schema 37, M4c slice AG).
pub const SECTION_OPINION: SectionTag = SectionTag::new("opinion");
/// Section: what people hold of the norms content names (schema 38, M4c slice AG).
pub const SECTION_NORMS: SectionTag = SectionTag::new("norms");
/// Section: what people hold of the values content names (schema 39, M4c slice AG).
pub const SECTION_VALUES: SectionTag = SectionTag::new("values");
/// Section: who holds which ideology, and the creeds laws were proposed under (schema 40, M4c
/// slice AG).
pub const SECTION_IDEOLOGIES: SectionTag = SectionTag::new("creeds");
/// Section: factions, their stores and histories, and who belongs to each (schema 41, M4c slice
/// AH).
pub const SECTION_FACTIONS: SectionTag = SectionTag::new("factions");
/// Section: the observer's interventions (schema 48, M4c slice AJ, ADR-0016 §5).
pub const SECTION_INFLUENCE: SectionTag = SectionTag::new("influenc");
/// Section: the places households know (schema 51, M5a slice AM).
pub const SECTION_PLACES: SectionTag = SectionTag::new("places");
/// Section: what people believe of other polities (schema 63, M5c slice AT).
pub const SECTION_RELATIONS: SectionTag = SectionTag::new("relation");
/// Section: crossings over water (schema 68, M5c slice AW).
pub const SECTION_CROSSINGS: SectionTag = SectionTag::new("crossing");
/// Section: turn-taking at bottlenecks (schema 141, M8b slice CN).
pub const SECTION_TRAFFIC: SectionTag = SectionTag::new("traffic");
/// Section: ways people have worked on (schema 140, M8b slice CM).
pub const SECTION_WAYS: SectionTag = SectionTag::new("ways");
/// Section: boats (schema 142, M8b slice CO).
pub const SECTION_BOATS: SectionTag = SectionTag::new("boats");
/// Section: carts (schema 145, M8b slice CP), records of the boats' shape.
pub const SECTION_CARTS: SectionTag = SectionTag::new("carts");
/// Section: wagons (schema 181, M10b slice DV), records of the boats' shape.
pub const SECTION_WAGONS: SectionTag = SectionTag::new("wagons");
/// Section: lines' passing loops and train runs (schema 182, M10b slice DW).
pub const SECTION_RUNS: SectionTag = SectionTag::new("runs");
/// Section: gas mains and connections (schema 186, M10b slice DY).
pub const SECTION_GAS: SectionTag = SectionTag::new("gas");
/// Section: goods in transit by boat (schema 143, M8b slice CO).
pub const SECTION_CONSIGNMENTS: SectionTag = SectionTag::new("consigns");
/// Section: wells (schema 74, M6a slice AY, step three).
pub const SECTION_WELLS: SectionTag = SectionTag::new("wells");
/// Section: infections (schema 75, M6a slice AZ).
pub const SECTION_SICKNESS: SectionTag = SectionTag::new("sickness");
/// Section: what people shed, where it lies and where it went (schema 76, M6a slice AZ, step
/// two).
pub const SECTION_CONTAGION: SectionTag = SectionTag::new("pathogen");
/// Section: fires, conflagrations and the exposure near those burning (schema 88; M6b slice BC).
pub const SECTION_FIRES: SectionTag = SectionTag::new("fires");
/// Section: high waters, flood episodes, storm damages, and storm episodes (schema 93; M6b slice BF).
pub const SECTION_FLOODS: SectionTag = SectionTag::new("floods");
/// Section: each time the ground was shaken and what each building did (schema 162; M8d slice DA).
pub const SECTION_QUAKES: SectionTag = SectionTag::new("quakes");

/// Activity index meaning "an activity the loaded content no longer has" (receipts only).
pub const UNKNOWN_ACTIVITY: u16 = u16::MAX;

/// The people-and-land sections of a save of `sim`.
pub(super) fn encode(sim: &Sim) -> Vec<SectionData> {
    let rules = &sim.rules;
    let activities: Vec<&str> = rules
        .catalog
        .activities
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    let goods: Vec<&str> = rules.catalog.goods.iter().map(|g| g.id.as_str()).collect();
    let recipes: Vec<&str> = rules
        .catalog
        .recipes
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    let resources: Vec<&str> = rules.land.resources.iter().map(|r| r.id.as_str()).collect();
    let skills: Vec<&str> = rules.catalog.skills.iter().map(|k| k.id.as_str()).collect();
    let techniques: Vec<&str> = rules
        .catalog
        .techniques
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    vec![
        section(SECTION_LAND, 0, encode_land(&sim.land, rules)),
        section(SECTION_SETTLE, 0, encode_settlements(&sim.land.settlements)),
        section(
            SECTION_PEOPLE,
            0,
            encode_people(&sim.people, &activities, &goods, &skills, &techniques),
        ),
        section(
            SECTION_HOUSES,
            0,
            encode_households(&sim.people, &goods, &resources, &recipes),
        ),
        section(SECTION_HISTORY, 0, encode_history(&sim.people)),
        section(
            SECTION_RECEIPTS,
            0,
            encode_receipts(&sim.people, &activities),
        ),
        section(SECTION_EVENTS, 0, encode_events(sim)),
        section(SECTION_FIELDS, 0, encode_fields(&sim.land.fields, rules)),
        section(SECTION_PLOTS, 0, encode_plots(&sim.land.plots)),
        section(SECTION_BUILDS, 0, encode_buildings(&sim.land.buildings)),
        section(SECTION_WEAR, 0, encode_wear(&sim.land.wear)),
        section(SECTION_MARKET, 0, encode_markets(&sim.people, &goods)),
        section(
            SECTION_FIRMS,
            0,
            encode_firms(&sim.people, &goods, &activities, &recipes),
        ),
        section(SECTION_WEALTH, 0, encode_wealth(&sim.people)),
        section(SECTION_KNOW, 0, encode_knowledge(&sim.people, &techniques)),
        section(
            SECTION_DEPOSITS,
            0,
            encode_deposits(&sim.land, &sim.people, &goods),
        ),
        section(SECTION_EARTH, 0, encode_earth(&sim.land)),
        section(SECTION_TIES, 0, encode_ties(&sim.people)),
        section(
            SECTION_POLITY,
            0,
            encode_polities(&sim.people, rules, &goods),
        ),
        section(SECTION_ORDER, 0, encode_order(&sim.people.order, &goods)),
        section(SECTION_WORD, 0, encode_word(&sim.people.word)),
        section(
            SECTION_OPINION,
            0,
            encode_opinion(&sim.people.opinion, rules),
        ),
        section(SECTION_NORMS, 0, encode_norms(&sim.people.norms, rules)),
        section(SECTION_VALUES, 0, encode_values(&sim.people.values, rules)),
        section(
            SECTION_IDEOLOGIES,
            0,
            encode_ideologies(&sim.people.ideologies, rules),
        ),
        section(
            SECTION_FACTIONS,
            0,
            encode_factions(&sim.people.factions, &sim.rules, &goods),
        ),
        section(
            SECTION_INFLUENCE,
            0,
            encode_influences(&sim.people.influences, rules),
        ),
        section(SECTION_PLACES, 0, encode_places(&sim.people, &goods)),
        section(SECTION_RELATIONS, 0, encode_relations(&sim.people, &goods)),
        section(
            SECTION_CROSSINGS,
            0,
            encode_crossings(&sim.land, &sim.people.fords, rules),
        ),
        section(SECTION_WELLS, 0, encode_wells(&sim.land, rules)),
        section(SECTION_SICKNESS, 0, encode_sickness(&sim.people, rules)),
        section(SECTION_CONTAGION, 0, encode_contagion(&sim.people, rules)),
        section(SECTION_FIRES, 0, encode_fires(&sim.people)),
        section(SECTION_FLOODS, 0, encode_floods(&sim.land, &sim.people)),
        section(SECTION_QUAKES, 0, encode_quakes(&sim.people)),
        section(SECTION_WAYS, 0, encode_ways(&sim.land)),
        section(SECTION_BOATS, 0, encode_boats(&sim.land.boats, &sim.rules)),
        section(SECTION_CARTS, 0, encode_boats(&sim.land.carts, &sim.rules)),
        section(
            SECTION_WAGONS,
            0,
            encode_boats(&sim.land.wagons, &sim.rules),
        ),
        section(SECTION_RUNS, 0, encode_runs(&sim.land.runs)),
        section(SECTION_GAS, 0, encode_gas(&sim.land.gas)),
        section(
            SECTION_CONSIGNMENTS,
            0,
            encode_consignments(&sim.people, &sim.rules),
        ),
        section(
            SECTION_TRAFFIC,
            0,
            encode_traffic(&sim.people.traffic, &sim.people.travel),
        ),
    ]
}

/// What the people-and-land sections decode to.
pub(super) struct Decoded {
    pub land: Land,
    pub people: Population,
    pub events: Vec<PendingEvent<SimEvent>>,
    /// People whose activity the loaded content no longer has: they decide again on load.
    pub redecide: Vec<PermanentId>,
    /// The save is from before deposits were bodies: they are placed from the seed on load.
    pub place_deposits: bool,
}

/// What a schema version stores, for the decoders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Schema {
    /// M1 slice A: food in kilocalories.
    V2,
    /// Goods.
    V3,
    /// Fields.
    V4,
    /// Plots and buildings.
    V5,
    /// Partners, pregnancies and unions.
    V6,
    /// Worn ground.
    V7,
    /// Families the observer sends (a chronicle kind older builds do not know).
    V8,
    /// Tools and skills (ADR-0006).
    V9,
    /// Offers and markets (ADR-0006 §4).
    V10,
    /// Firms (ADR-0006 §5).
    V11,
    /// Property regimes: who holds each field (ADR-0007).
    V12,
    /// Knowledge carried by people (ADR-0008).
    V13,
    /// Trying toward techniques, and finds (ADR-0008 §3).
    V14,
    /// Frame buildings: rectangular footprints, sixteen parameters, plots for stores and
    /// workshops (ADR-0009 §2, §7).
    V15,
    /// Workshops that name their firms, firms' most at once, and roofed floor and storage in the
    /// wealth measures (ADR-0009 §7).
    V16,
    /// The condition of buildings: each group's quality, loss and state, the building's state,
    /// its builders' skill and upkeep under way (ADR-0009 §4, §6).
    V17,
    /// What each settlement has seen of each technique's buildings (ADR-0009 §6).
    V18,
    /// Deposits as bodies in the ground (ADR-0010 §1).
    V19,
    /// Earthworks and the ground they changed (ADR-0010 §2-3).
    V20,
    /// Pits and spoil heaps linked to their deposits (ADR-0010 §2).
    V21,
    /// Households' taste in building and the building each followed (M3b slice R).
    V22,
    /// Worn ground kept exactly, each tile at its own day, with the routing view (ADR-0011 §5).
    V23,
    /// Weather: a daily series with its record, and each growing field's water (ADR-0012).
    V24,
    /// What stood out in the weather, in the chronicle (ADR-0012).
    V25,
    /// Each field's soil and the record of its harvests (ADR-0012 §3).
    V26,
    /// Each household's midden (M3c slice V).
    V27,
    /// Ties between people (ADR-0014).
    V28,
    /// Each settlement's polity, its laws and its store (ADR-0013).
    V29,
    /// Laws that name someone, and laws that lapse (ADR-0013 §2).
    V30,
    /// Takings, beliefs and obligations; each person's objection and perceived risk (ADR-0015).
    V31,
    /// Laws against taking with their bundles; what households know of the food takings moved.
    V32,
    /// The watch: its record on its law (M4b slice AC).
    V33,
    /// Where a taker's household stood by food when they came (M4b slice AD).
    V34,
    /// What people have heard and the grievances they hold (M4c slice AE).
    V35,
    /// Amendments of the custom: a law's proposed body, the custom's versions (M4c slice AF).
    V36,
    /// Where people stand on questions; what talk added to stances (M4c slice AG).
    V37,
    /// What people hold of norms; what households last did at a levy (M4c slice AG).
    V38,
    /// What people hold dear; what it added to stances (M4c slice AG).
    V39,
    /// Who holds which ideology; the creeds laws were proposed under (M4c slice AG).
    V40,
    /// Factions and who belongs to each (M4c slice AH).
    V41,
    /// Petitions, and laws that replace another (M4c slice AH).
    V42,
    /// Refusals of a levy (M4c slice AH).
    V43,
    /// Revolts, and customs taken rather than amended (M4c slice AI).
    V44,
    /// Repeals carried, and laws put to a founding (M4c slice AI, step two).
    V45,
    /// Coups, and a body of those who keep the watch (M4c slice AI, step three).
    V46,
    /// Encounters, blows and deaths by violence (M4c slice AI, step four).
    V47,
    /// The observer's interventions (M4c slice AJ).
    V48,
    /// Agitators, blessings and curses (M4c slice AJ, step two).
    V49,
    /// Several settlements: how each was founded, residence histories, hearths that name their
    /// settlement (M5a slice AK, ADR-0018).
    V50,
    /// The places households know (M5a slice AM, ADR-0018 §4).
    V51,
    /// Visits: contacts between settlements and failed searches for a partner (M5a slice AM,
    /// step two).
    V52,
    /// Moving between settlements: leanings, reviews prompted, people moved (M5a slice AN).
    V53,
    /// Migration waves (M5a slice AN, step two).
    V54,
    /// Coalitions gathered to found settlements (M5a slice AO).
    V55,
    /// Price reports and purchases between settlements (M5b slice AP).
    V56,
    /// The convergence record (M5b slice AQ).
    V57,
    /// Errands to fetch goods to sell at home (M5b slice AQ, step two).
    V58,
    /// Buildings seen in other settlements (M5b slice AR).
    V59,
    /// Techniques seen in other settlements; the settlements an arrival or a loss concerns (M5b
    /// slice AR, step two).
    V60,
    /// Settlements' founding ways (M5b slice AR, step three).
    V61,
    /// The places people work, the outsiders seen there, and claims on them (M5c slice AT).
    V62,
    /// Who did the work, and people's views of other polities (M5c slice AT, step two).
    V63,
    /// The claims of other polities households heard of (M5c slice AU, step one).
    V64,
    /// Agreements between polities and the laws that decide them (M5c slice AU, step two).
    V65,
    /// What views of another polity added to stances on an agreement (M5c slice AU, step three).
    V66,
    /// Goods an agreement moves, and the payments owed under it (M5c slice AV).
    V67,
    /// Crossings over water (M5c slice AW).
    V68,
    /// The streams households wade, and the crossings they build (M5c slice AW, step two).
    V69,
    /// The crossings polities build, and the work each household gave them (M5c slice AW, step
    /// three).
    V70,
    /// The room to work on a crossing (M5c slice AX).
    V71,
    /// Water under the ground and the rivers' runoff store (M6a slice AY).
    V72,
    /// Sources people choose and the water they use (M6a slice AY, step two).
    V73,
    /// Wells (M6a slice AY, step three).
    V74,
    /// Infections (M6a slice AZ).
    V75,
    /// What people shed, moved through the ground and water (M6a slice AZ, step two).
    V76,
    /// Care given the ill (M6a slice AZ, step three).
    V77,
    /// Claims that a household had sickness (M6a slice BA): no new state, a claim kind appended.
    V78,
    /// Suspicions of a source, and the households seen drawing water (M6a slice BA, step two).
    V79,
    /// The chronicle's suspicions (M6a slice BA, step three): no new state, a chronicle kind
    /// appended.
    V80,
    /// Wells' lids and curbs (M6a slice BA, step three).
    V81,
    /// The issues of sickness and of a suspected source, and laws closing a source of water
    /// (M6a slice BA, step four).
    V82,
    /// Wells dug together by law, with the hours each household gave, and households' walks to
    /// water (M6a slice BA, step four).
    V83,
    /// The observer's drought, with the months it held marked (M6a slice BA, step five).
    V84,
    /// Each infection keeps the household it began in (M6a slice BB).
    V85,
    /// Each place a household uses remembers which of the last 64 days its people worked there
    /// (M6a slice BB, step three: the Water lens).
    V86,
    /// The weather keeps a daily wind and the fuel's dryness (M6b slice BC, step one).
    V87,
    /// Fires, conflagrations and the exposure near those burning (M6b slice BC, step two).
    V88,
    /// Ignitions of the fires people use, and the pending event that weighs one (M6b slice BD).
    V89,
    /// Escapes that grow before the covering, and who knew of each fire (M6b slice BD, step two).
    V90,
    /// Water carried to a fire: wet roofs, loads thrown and the douse target (M6b slice BD, step
    /// three).
    V91,
    /// Roofs stripped and goods carried out of a fire's way (M6b slice BD, step four).
    V92,
    /// Flood and storm incidents and episodes, flood memory (M6b slice BF).
    V93,
    /// Fire caution by covering and wall kind (BE persistence repair).
    V94,
    /// Active observer runoff (BF command repair).
    V95,
    /// Hazard provenance and storm pulse evaluations.
    V96,
    V97,
    V98,
    V99,
    V100,
    V101,
    /// M6b slice BG: households' flood losses, banks built by law.
    V102,
    /// M6b slice BG: bank piping damage and told failures.
    V103,
    /// M6b slice BG: banks mended (refill, repair hours asked).
    V104,
    /// M6b slice BG: irrigation ditches.
    V105,
    /// M6b slice BG: households dig ditches.
    V106,
    /// M6b slice BH: households remember fires.
    V107,
    /// M6b slice BH: households remember buildings that gave way.
    V108,
    /// M6c slice BI: an incident names the taker's settlement when it is another.
    V109,
    /// M6c slice BJ: bands and the encounters they met.
    V110,
    /// M6c slice BJ, step two: raids and bands seen as claims, who was asked to a band.
    V111,
    /// M6c slice BJ, step three: bands sent and raiding forbidden by law.
    V112,
    /// M6c slice BK: enclosures.
    V113,
    /// M6c slice BK, step two: palisades raised by law.
    V114,
    /// M6c slice BK, step three: gates kept, posts rotting and mended.
    V115,
    /// M6c slice BL, step one: bands camp by a shut line.
    V116,
    /// M6c slice BL, step two: words against raiding in agreements (clause kind 3).
    V117,
    /// M6c slice BM, step one: a polity holds several settlements, and one taken in ends.
    V118,
    /// M6c slice BM, step two: bands holding a settlement, the annex law and the holding issue.
    V119,
    /// M7a slice BN: office laws held for a term (`PolitySave.terms`).
    V120,
    /// M7a slice BO: councils and their elections (`PolitySave.seats`, `seated`, `elections`).
    V121,
    /// M7a slice BP: candidates' positions and faction slates (`ElectionSave.positions`,
    /// `endorsements`).
    V122,
    /// M7a slice BQ: seats shared among settlements (`ElectionSave.districts`,
    /// `candidate_districts`; the seats' packed number gains a bit).
    V123,
    /// M7a slice BQ: electoral alliances (`ElectionSave.alliances`).
    V124,
    /// M7b slice BR: forums (`CaseSave.forum`).
    V125,
    /// M7b slice BS: precedent (`CaseSave.tags`, `departed`, `stance_precedent`).
    V126,
    /// M7b slice BT: limits and review (`CaseSave.challenge`, law status 6 struck down).
    V127,
    /// Trade laws: the issue *goods fetched elsewhere* (M7c slice BV).
    V128,
    /// Organizations by stake (M7c slice BW: `FactionSave.has_stake` and its rules, events 3-4,
    /// `Blamed` kind 5, `Wrong` code 10).
    V129,
    /// Privileges (M7c slice BX: policy kind code 24, firm exit 4).
    V130,
    /// The keeper's diversion and relief by favour (M7c slice BY: `LawSave.diverted` and its
    /// fellows, channel 20, `Wrong` code 11).
    V131,
    /// Closed bodies (M7c slice BZ: membership codes 5 and 6).
    V132,
    /// Loans in goods (M7d slice CA: `Order.loans`, channels 21-22, tie acts 15-17).
    V133,
    /// Pledges and answers at the due day (M7d slice CB: `Order.loan_terms`, loan standings 3-4).
    V134,
    /// Debts heard (M7d slice CB: `CaseSave.loan`, obligation kind 4, issue code 21).
    V135,
    /// Coin (M7d slice CE: `Order.coin_issues`, `Order.purses`).
    V136,
    /// The mint (M7d slice CE2b: `Order.brought`, issue code 22).
    V137,
    /// Pay for common work (M7d slice CE3a: issue code 23).
    V138,
    /// Coin struck in the chronicle (M7d slice CE5: chronicle code 61).
    V139,
    /// Ways (M8b slice CM: the `ways` section).
    V140,
    /// Bottlenecks for walkers (M8b slice CN: the `traffic` section, a trip's next bottleneck,
    /// event kind 5).
    V141,
    /// Boats (M8b slice CO: the `boats` section).
    V142,
    /// Goods in transit by boat (M8b slice CO: paddle and handling steps, trips afloat, an
    /// errand's way by boat, the `consignments` section).
    V143,
    /// Boats being made (M8b slice CO: a boat's labour, work and crew, a boat as a target).
    V144,
    /// Carts (M8b slice CP: the `carts` section, the haul step, an errand's way by cart).
    V145,
    /// Carts being made (M8b slice CP, third part: the chronicle's cart made).
    V146,
    /// Carts meeting by clearance (M8b slice CP, fourth part: waits held at a narrows, facility
    /// kind 2).
    V147,
    /// Losses on ways (M8b slice CQ, first part: the traffic section's losses).
    V148,
    /// Passing places (M8b slice CQ, second part: a way as an activity's target).
    V149,
    V150,
    V151,
    V152,
    V153,
    V154,
    V155,
    V156,
    V157,
    V158,
    V159,
    V160,
    V161,
    V162,
    V163,
    V164,
    V165,
    V166,
    V167,
    V168,
    V169,
    /// Households keep batches that run unattended (M10a slice DP).
    V171,
    /// Wagons (M10b slice DV: the `wagons` section).
    V181,
    /// Train runs (M10b slice DW: the `runs` section).
    V182,
    /// Gas mains and connections (M10b slice DY: the `gas` section).
    V186,
}

/// Decodes and checks the people-and-land sections of a save of schema version `version` (2 or
/// later), taken at `now`.
#[allow(clippy::too_many_arguments)]
pub(super) fn decode<R: Read + Seek>(
    reader: &mut SnapshotReader<R>,
    rules: &Rules,
    map: &WorldMap,
    next_id: u64,
    version: u32,
    now: SimTime,
    seed: u64,
    climatology: Climatology,
) -> Result<Decoded, LoadError> {
    let schema = match version {
        SCHEMA_V2 => Schema::V2,
        SCHEMA_V3 => Schema::V3,
        SCHEMA_V4 => Schema::V4,
        SCHEMA_V5 => Schema::V5,
        SCHEMA_V6 => Schema::V6,
        SCHEMA_V7 => Schema::V7,
        SCHEMA_V8 => Schema::V8,
        SCHEMA_V9 => Schema::V9,
        SCHEMA_V10 => Schema::V10,
        SCHEMA_V11 => Schema::V11,
        SCHEMA_V12 => Schema::V12,
        SCHEMA_V13 => Schema::V13,
        SCHEMA_V14 => Schema::V14,
        SCHEMA_V15 => Schema::V15,
        SCHEMA_V16 => Schema::V16,
        SCHEMA_V17 => Schema::V17,
        SCHEMA_V18 => Schema::V18,
        SCHEMA_V19 => Schema::V19,
        SCHEMA_V20 => Schema::V20,
        SCHEMA_V21 => Schema::V21,
        SCHEMA_V22 => Schema::V22,
        SCHEMA_V23 => Schema::V23,
        SCHEMA_V24 => Schema::V24,
        SCHEMA_V25 => Schema::V25,
        SCHEMA_V26 => Schema::V26,
        SCHEMA_V27 => Schema::V27,
        SCHEMA_V28 => Schema::V28,
        SCHEMA_V29 => Schema::V29,
        SCHEMA_V30 => Schema::V30,
        SCHEMA_V31 => Schema::V31,
        SCHEMA_V32 => Schema::V32,
        SCHEMA_V33 => Schema::V33,
        SCHEMA_V34 => Schema::V34,
        SCHEMA_V35 => Schema::V35,
        SCHEMA_V36 => Schema::V36,
        SCHEMA_V37 => Schema::V37,
        SCHEMA_V38 => Schema::V38,
        SCHEMA_V39 => Schema::V39,
        SCHEMA_V40 => Schema::V40,
        SCHEMA_V41 => Schema::V41,
        SCHEMA_V42 => Schema::V42,
        SCHEMA_V43 => Schema::V43,
        SCHEMA_V44 => Schema::V44,
        SCHEMA_V45 => Schema::V45,
        SCHEMA_V46 => Schema::V46,
        SCHEMA_V47 => Schema::V47,
        SCHEMA_V48 => Schema::V48,
        SCHEMA_V49 => Schema::V49,
        SCHEMA_V50 => Schema::V50,
        SCHEMA_V51 => Schema::V51,
        SCHEMA_V52 => Schema::V52,
        SCHEMA_V53 => Schema::V53,
        SCHEMA_V54 => Schema::V54,
        SCHEMA_V55 => Schema::V55,
        SCHEMA_V56 => Schema::V56,
        SCHEMA_V57 => Schema::V57,
        SCHEMA_V58 => Schema::V58,
        SCHEMA_V59 => Schema::V59,
        SCHEMA_V60 => Schema::V60,
        SCHEMA_V61 => Schema::V61,
        SCHEMA_V62 => Schema::V62,
        SCHEMA_V63 => Schema::V63,
        SCHEMA_V64 => Schema::V64,
        SCHEMA_V65 => Schema::V65,
        SCHEMA_V66 => Schema::V66,
        SCHEMA_V67 => Schema::V67,
        SCHEMA_V68 => Schema::V68,
        SCHEMA_V69 => Schema::V69,
        SCHEMA_V70 => Schema::V70,
        SCHEMA_V71 => Schema::V71,
        SCHEMA_V72 => Schema::V72,
        SCHEMA_V73 => Schema::V73,
        SCHEMA_V74 => Schema::V74,
        SCHEMA_V75 => Schema::V75,
        SCHEMA_V76 => Schema::V76,
        SCHEMA_V77 => Schema::V77,
        SCHEMA_V78 => Schema::V78,
        SCHEMA_V79 => Schema::V79,
        SCHEMA_V80 => Schema::V80,
        SCHEMA_V81 => Schema::V81,
        SCHEMA_V82 => Schema::V82,
        SCHEMA_V83 => Schema::V83,
        SCHEMA_V84 => Schema::V84,
        SCHEMA_V85 => Schema::V85,
        SCHEMA_V86 => Schema::V86,
        SCHEMA_V87 => Schema::V87,
        SCHEMA_V88 => Schema::V88,
        SCHEMA_V89 => Schema::V89,
        SCHEMA_V90 => Schema::V90,
        SCHEMA_V91 => Schema::V91,
        SCHEMA_V92 => Schema::V92,
        SCHEMA_V93 => Schema::V93,
        SCHEMA_V94 => Schema::V94,
        SCHEMA_V95 => Schema::V95,
        SCHEMA_V96 => Schema::V96,
        SCHEMA_V97 => Schema::V97,
        SCHEMA_V98 => Schema::V98,
        SCHEMA_V99 => Schema::V99,
        SCHEMA_V100 => Schema::V100,
        SCHEMA_V101 => Schema::V101,
        SCHEMA_V102 => Schema::V102,
        SCHEMA_V103 => Schema::V103,
        SCHEMA_V104 => Schema::V104,
        SCHEMA_V105 => Schema::V105,
        SCHEMA_V106 => Schema::V106,
        SCHEMA_V107 => Schema::V107,
        SCHEMA_V108 => Schema::V108,
        SCHEMA_V109 => Schema::V109,
        SCHEMA_V110 => Schema::V110,
        SCHEMA_V111 => Schema::V111,
        SCHEMA_V112 => Schema::V112,
        SCHEMA_V113 => Schema::V113,
        SCHEMA_V114 => Schema::V114,
        SCHEMA_V115 => Schema::V115,
        SCHEMA_V116 => Schema::V116,
        SCHEMA_V117 => Schema::V117,
        SCHEMA_V118 => Schema::V118,
        SCHEMA_V119 => Schema::V119,
        SCHEMA_V120 => Schema::V120,
        SCHEMA_V121 => Schema::V121,
        SCHEMA_V122 => Schema::V122,
        SCHEMA_V123 => Schema::V123,
        SCHEMA_V124 => Schema::V124,
        SCHEMA_V125 => Schema::V125,
        SCHEMA_V126 => Schema::V126,
        SCHEMA_V127 => Schema::V127,
        SCHEMA_V128 => Schema::V128,
        SCHEMA_V129 => Schema::V129,
        SCHEMA_V130 => Schema::V130,
        SCHEMA_V131 => Schema::V131,
        SCHEMA_V132 => Schema::V132,
        SCHEMA_V133 => Schema::V133,
        SCHEMA_V134 => Schema::V134,
        SCHEMA_V135 => Schema::V135,
        SCHEMA_V136 => Schema::V136,
        SCHEMA_V137 => Schema::V137,
        SCHEMA_V138 => Schema::V138,
        SCHEMA_V139 => Schema::V139,
        SCHEMA_V140 => Schema::V140,
        SCHEMA_V141 => Schema::V141,
        SCHEMA_V142 => Schema::V142,
        SCHEMA_V143 => Schema::V143,
        SCHEMA_V144 => Schema::V144,
        SCHEMA_V145 => Schema::V145,
        SCHEMA_V146 => Schema::V146,
        SCHEMA_V147 => Schema::V147,
        SCHEMA_V148 => Schema::V148,
        SCHEMA_V149 => Schema::V149,
        SCHEMA_V150 => Schema::V150,
        SCHEMA_V151 => Schema::V151,
        SCHEMA_V152 => Schema::V152,
        SCHEMA_V153 => Schema::V153,
        SCHEMA_V154 => Schema::V154,
        SCHEMA_V155 => Schema::V155,
        SCHEMA_V156 => Schema::V156,
        SCHEMA_V157 => Schema::V157,
        SCHEMA_V158 => Schema::V158,
        SCHEMA_V159 => Schema::V159,
        SCHEMA_V160 => Schema::V160,
        SCHEMA_V161 => Schema::V161,
        SCHEMA_V162 => Schema::V162,
        SCHEMA_V163 => Schema::V163,
        SCHEMA_V164 => Schema::V164,
        SCHEMA_V165 => Schema::V165,
        SCHEMA_V166 => Schema::V166,
        SCHEMA_V167 => Schema::V167,
        SCHEMA_V168 => Schema::V168,
        // The observer's forcings (schema 170) changed no people or land.
        SCHEMA_V169 | SCHEMA_V170 => Schema::V169,
        // Workshops' batches (schema 172) changed no people or land; installations (schema 173),
        // households' mill counts (schema 174), pits' water (schema 175), engines (schema 176),
        // pits' bailing (schema 177), works' bells (schema 178), the first lamp lit (schema 179)
        // and ways' track (schema 180) are read wherever saved.
        SCHEMA_V171 | SCHEMA_V172 | SCHEMA_V173 | SCHEMA_V174 | SCHEMA_V175 | SCHEMA_V176
        | SCHEMA_V177 | SCHEMA_V178 | SCHEMA_V179 | SCHEMA_V180 => Schema::V171,
        SCHEMA_V181 => Schema::V181,
        // Runs' wagons (schema 183), lines' stations and timetables (schema 184) and rides and
        // fares (schema 185) are read wherever saved; gas mains (schema 186) have their section.
        SCHEMA_V182 | SCHEMA_V183 | SCHEMA_V184 | SCHEMA_V185 => Schema::V182,
        // Mains' lengths laid (schema 187), their worth (188) and street lamps (189) are read
        // wherever saved, as are calls for carers (190), who tended the ill (191), births
        // attended (192), enrolments to learn (193), households untaught (194), written records
        // (195), where each is kept (196), deaths in unknowing paid care (197), the day's
        // company kept (198) and calls to inoculate (199).
        SCHEMA_V186 | SCHEMA_V187 | SCHEMA_V188 | SCHEMA_V189 | SCHEMA_V190 | SCHEMA_V191
        | SCHEMA_V192 | SCHEMA_V193 | SCHEMA_V194 | SCHEMA_V195 | SCHEMA_V196 | SCHEMA_V197
        | SCHEMA_V198 | SCHEMA_V199 | SCHEMA_V200 | SCHEMA_V201 | SAVE_SCHEMA_VERSION => {
            Schema::V186
        }
        other => {
            return Err(LoadError::Incompatible(format!(
                "world schema version {other} has no people-and-land decoder"
            )));
        }
    };
    let bytes = single_chunk(reader, SECTION_LAND)?;
    let mut land = decode_land(&bytes, rules, map, climatology, seed)?;
    let bytes = single_chunk(reader, SECTION_SETTLE)?;
    land.settlements = decode_settlements(&bytes)?;

    let mut people = Population::new();
    let bytes = single_chunk(reader, SECTION_HOUSES)?;
    for mut h in decode_households(&bytes, rules, schema, now)? {
        // Before schema 22 nobody had a taste in building: each household gets the one its band
        // would have brought (M3b slice R).
        if schema < Schema::V22 {
            h.taste = civ_agents::style::founding_taste(
                &rules.people.style,
                seed,
                h.settlement.unwrap_or(h.id),
                h.id,
            );
        }
        people.insert_household(h);
    }
    let bytes = single_chunk(reader, SECTION_PEOPLE)?;
    let (persons, next_trip, redecide, new_skills, new_techniques) =
        decode_people(&bytes, rules, schema, &people)?;
    people.next_trip = next_trip;
    for p in persons {
        people.insert_person(p);
    }
    // Before schema 31 nobody had an objection to taking or a sense of the chance of being seen:
    // each gets a founder's, by a draw keyed to them (ADR-0015 §2).
    people.give_crime_state(&rules.people.crime, seed);
    let bytes = single_chunk(reader, SECTION_HISTORY)?;
    let (records, chronicle, unions) = decode_history(&bytes)?;
    people.records = records;
    people.chronicle = chronicle;
    people.unions = unions;
    // Before schema 50 nobody's residence was kept: it is inferred from where they live, the
    // chronicle and how they came (ADR-0018 §2).
    if schema < Schema::V50 {
        people.infer_residence(&land.settlements);
    }
    if schema < Schema::V6 {
        people.infer_couples(&rules.people, seed, now);
    }
    // Before schema 9 nobody had tools or skills: everyone gets what a founder brings. A skill
    // added to the content since the save was made, likewise.
    if schema < Schema::V9 {
        people.give_founders_kit(&rules.catalog, &rules.people, seed, now);
    }
    people.give_new_skills(&rules.catalog, &rules.people, seed, now, &new_skills);
    let bytes = single_chunk(reader, SECTION_RECEIPTS)?;
    decode_receipts(&bytes, rules, &mut people)?;
    let bytes = single_chunk(reader, SECTION_EVENTS)?;
    let events = decode_events(&bytes)?;
    if schema >= Schema::V4 {
        let bytes = single_chunk(reader, SECTION_FIELDS)?;
        land.fields = decode_fields(&bytes, rules, schema)?;
    }
    if schema >= Schema::V5 {
        let bytes = single_chunk(reader, SECTION_PLOTS)?;
        land.plots = decode_plots(&bytes)?;
        let bytes = single_chunk(reader, SECTION_BUILDS)?;
        land.buildings = decode_buildings(&bytes, schema)?;
        // Each building's condition in line with its design: before schema 17 its groups in
        // place are put there sound (ADR-0009 §8).
        condition::reconcile_all(
            &mut land.buildings,
            &rules.catalog,
            seed,
            rules.people.build.quality_spread,
            now,
        );
    }
    // Before schema 7 nobody had worn the ground: it starts untrodden.
    if schema >= Schema::V7 {
        let bytes = single_chunk(reader, SECTION_WEAR)?;
        land.wear = decode_wear(&bytes, map, now.day_index())?;
    }
    // Before schema 10 there were no markets: they start empty.
    if schema >= Schema::V10 {
        let bytes = single_chunk(reader, SECTION_MARKET)?;
        people.markets = decode_markets(&bytes, rules)?;
    }
    // Before schema 11 there were no firms.
    if schema >= Schema::V11 {
        let bytes = single_chunk(reader, SECTION_FIRMS)?;
        people.firms = decode_firms(&bytes, rules)?;
    }
    // The yearly wealth history, when the save has one.
    if schema >= Schema::V12 && reader.has_section(SECTION_WEALTH) {
        let bytes = single_chunk(reader, SECTION_WEALTH)?;
        people.wealth_years = decode_wealth(&bytes)?;
    }
    // Before schema 13 nobody's knowledge was kept: everyone gets what a founder of their age
    // brings, and the settlements' records begin now (ADR-0008 §7).
    if schema >= Schema::V13 {
        let bytes = single_chunk(reader, SECTION_KNOW)?;
        people.knowledge = decode_knowledge(&bytes, rules)?;
        // Before schema 18 no settlement remembered its buildings: it has seen nothing yet.
        if schema >= Schema::V18 {
            people.trust = decode_trust(&bytes, rules)?;
        }
        // Older saves did not keep fire caution; begin with no remembered losses.
        if schema >= Schema::V94 {
            people.fire_trust = decode_fire_trust(&bytes)?;
        }
        // Enrolments to learn and today's sessions (schema 193): none in older saves.
        decode_enrolments(&bytes, rules, &mut people)?;
        // A technique founders bring that the content has added since the save was made is
        // given as a founder of their age would bring it, like a new skill.
        people.give_new_founder_knowledge(
            &rules.catalog,
            &rules.people,
            seed,
            now,
            &new_techniques,
        );
    } else {
        people.give_founders_knowledge(&rules.catalog, &rules.people, seed, now);
    }
    // Deposits (schema 19); an older save places them from the seed once it is loaded.
    if schema >= Schema::V19 {
        let bytes = single_chunk(reader, SECTION_DEPOSITS)?;
        (land.deposits, people.deposits_known) = decode_deposits(&bytes, rules)?;
    }
    // Earthworks (schema 20); before them nobody had changed the ground.
    if schema >= Schema::V20 {
        let bytes = single_chunk(reader, SECTION_EARTH)?;
        decode_earth(&bytes, &mut land, schema)?;
    }
    // Ties (schema 28); before them nobody remembered anybody. Standing is kept beside them; a
    // save without it works it out afresh (ADR-0014 §3).
    let mut standing = None;
    if schema >= Schema::V28 {
        let bytes = single_chunk(reader, SECTION_TIES)?;
        (people.ties, standing) = decode_ties(&bytes)?;
    }
    // Polities (schema 29); before them no settlement had one, and each is founded at the next
    // midnight under the custom (ADR-0013 §1).
    if schema >= Schema::V29 {
        let bytes = single_chunk(reader, SECTION_POLITY)?;
        people.polities = decode_polities(&bytes, rules)?;
        // Explicit migration: schemas through 103 had asked no bank mending.
        if schema < Schema::V104
            && people.polities.iter().any(|p| {
                !p.bank_keeps.is_empty()
                    || p.banks.iter().any(|(_, s)| s.upkeep_h != 0.0 || s.mending)
            })
        {
            return Err(LoadError::Malformed(
                "bank mending asked before schema 104".into(),
            ));
        }
    }
    // Takings (schema 31); before them nobody had taken anything.
    if schema >= Schema::V31 {
        let bytes = single_chunk(reader, SECTION_ORDER)?;
        people.order = decode_order(&bytes, rules)?;
    }
    // Word and grievances (schema 35). Before them every adult heard of a gathering called, so a
    // gathering still to sit is heard of by every adult of its settlement.
    if schema >= Schema::V35 {
        let bytes = single_chunk(reader, SECTION_WORD)?;
        people.word = decode_word(&bytes)?;
    } else {
        people.hear_of_gatherings_called(now, &rules.people);
    }
    // Opinion (schema 37); before it nobody held a position, and each is anchored at the next
    // first of the month.
    if schema >= Schema::V37 {
        let bytes = single_chunk(reader, SECTION_OPINION)?;
        people.opinion = decode_opinion(&bytes, rules)?;
    }
    // Norms (schema 38); before them nobody held one, and each takes theirs at the next first of
    // the month, or when first asked to pay a levy.
    if schema >= Schema::V38 {
        let bytes = single_chunk(reader, SECTION_NORMS)?;
        people.norms = decode_norms(&bytes, rules)?;
    }
    // Values (schema 39); before them nobody held one, and each takes theirs the next midnight.
    if schema >= Schema::V39 {
        let bytes = single_chunk(reader, SECTION_VALUES)?;
        people.values = decode_values(&bytes, rules)?;
    }
    // Ideologies (schema 40); before them nobody held one, and everyone takes their start the
    // next midnight.
    if schema >= Schema::V40 {
        let bytes = single_chunk(reader, SECTION_IDEOLOGIES)?;
        people.ideologies = decode_ideologies(&bytes, rules)?;
    }
    // Factions (schema 41); before them there were none.
    if schema >= Schema::V41 {
        let bytes = single_chunk(reader, SECTION_FACTIONS)?;
        people.factions = decode_factions(&bytes, rules)?;
    }
    // The observer's interventions (schema 48); before them the observer had reached nobody so.
    if schema >= Schema::V48 {
        let bytes = single_chunk(reader, SECTION_INFLUENCE)?;
        people.influences = decode_influences(&bytes, rules)?;
    }
    // The places households know (schema 51); before them a household knew none but its own.
    if schema >= Schema::V51 {
        let bytes = single_chunk(reader, SECTION_PLACES)?;
        let d = decode_places(&bytes, rules)?;
        (
            people.known_places,
            people.contacts,
            people.unmatched,
            people.leanings,
            people.review_due,
            people.coalitions,
            people.reports,
            people.convergence,
            people.seen_away,
            people.founding_ways,
            people.uses,
            people.spring_draws,
        ) = (
            d.known,
            d.contacts,
            d.unmatched,
            d.leanings,
            d.due,
            d.coalitions,
            d.reports,
            d.convergence,
            d.seen,
            d.ways,
            d.uses,
            d.draws,
        );
    }
    // What people believe of other polities (schema 63); before, nobody held a view.
    if schema >= Schema::V63 {
        let bytes = single_chunk(reader, SECTION_RELATIONS)?;
        (people.polity_views, people.claims_heard, people.agreements) =
            decode_relations(&bytes, rules)?;
    }
    // Crossings over water (schema 68), and the streams households wade (schema 69); before,
    // none.
    if schema >= Schema::V68 {
        let bytes = single_chunk(reader, SECTION_CROSSINGS)?;
        (land.crossings, people.fords, land.enclosures) = decode_crossings(&bytes, rules, map)?;
    }
    // Turn-taking at bottlenecks (schema 141); before, nobody had taken a turn.
    if schema >= Schema::V141 {
        let bytes = single_chunk(reader, SECTION_TRAFFIC)?;
        (people.traffic, people.travel) = decode_traffic(&bytes)?;
    }
    // Boats (schema 142); before, none.
    if schema >= Schema::V142 {
        let bytes = single_chunk(reader, SECTION_BOATS)?;
        land.boats = decode_boats(&bytes, SECTION_BOATS, rules, map, schema)?;
    }
    // Carts (schema 145); before, none.
    if schema >= Schema::V145 {
        let bytes = single_chunk(reader, SECTION_CARTS)?;
        land.carts = decode_boats(&bytes, SECTION_CARTS, rules, map, schema)?;
    }
    // Wagons (schema 181); before, none.
    if schema >= Schema::V181 {
        let bytes = single_chunk(reader, SECTION_WAGONS)?;
        land.wagons = decode_boats(&bytes, SECTION_WAGONS, rules, map, schema)?;
    }
    // Lines' loops and train runs (schema 182); before, none.
    if schema >= Schema::V182 {
        let bytes = single_chunk(reader, SECTION_RUNS)?;
        land.runs = decode_runs(&bytes, SECTION_RUNS)?;
    }
    // Gas mains and connections (schema 186); before, none.
    if schema >= Schema::V186 {
        let bytes = single_chunk(reader, SECTION_GAS)?;
        land.gas = decode_gas(&bytes, SECTION_GAS)?;
    }
    // Goods in transit (schema 143); before, none.
    if schema >= Schema::V143 {
        let bytes = single_chunk(reader, SECTION_CONSIGNMENTS)?;
        people.consignments = decode_consignments(&bytes, rules)?;
    }
    // Ways (schema 140); before, none.
    if schema >= Schema::V140 {
        let bytes = single_chunk(reader, SECTION_WAYS)?;
        land.ways = decode_ways(&bytes, map)?;
    }
    // Wells (schema 74); before, none.
    if schema >= Schema::V74 {
        let bytes = single_chunk(reader, SECTION_WELLS)?;
        land.wells = decode_wells(&bytes, rules, map)?;
    }
    // Infections (schema 75); before, none.
    if schema >= Schema::V75 {
        let bytes = single_chunk(reader, SECTION_SICKNESS)?;
        people.sickness = decode_sickness(&bytes, rules)?;
        people.met = decode_met(&bytes)?;
    }
    // What people shed (schema 76); before, nothing lies anywhere.
    if schema >= Schema::V76 {
        let bytes = single_chunk(reader, SECTION_CONTAGION)?;
        people.contagion = decode_contagion(&bytes, rules)?;
    }
    // Fires (schema 88); before, none has burned.
    if schema >= Schema::V88 {
        let bytes = single_chunk(reader, SECTION_FIRES)?;
        people.fires = decode_fires(&bytes, schema)?;
    }
    // Floods (schema 93); before, none has flooded or blown.
    if schema >= Schema::V93 {
        let bytes = single_chunk(reader, SECTION_FLOODS)?;
        let (floods, flood_rev, reaches) = decode_floods(&bytes, schema)?;
        people.floods = floods;
        land.flood_revision = flood_rev;
        let r =
            reaches.ok_or_else(|| LoadError::Malformed("missing river discharges".to_owned()))?;
        if r.len() != map.reaches.len() || r.iter().any(|q| !q.is_finite() || *q < 0.0) {
            return Err(LoadError::Malformed(
                "invalid river discharge array".to_owned(),
            ));
        }
        land.reach_discharges = r;
        if schema >= Schema::V95 {
            let root = section_root::<save::FloodsSave>(&bytes)
                .map_err(|e| unreadable(SECTION_FLOODS, &e))?;
            for f in root.river_forcings().iter().flatten() {
                if f.reach() as usize >= map.reaches.len()
                    || !f.discharge_m3s().is_finite()
                    || f.discharge_m3s() <= 0.0
                    || f.until_day() <= now.day_index()
                    || !people.influences.list.iter().any(|i| {
                        i.id == f.influence()
                            && i.kind == civ_agents::influence::InfluenceKind::Flood
                    })
                    || land.river_forcings.iter().any(|old| old.reach == f.reach())
                {
                    return Err(LoadError::Malformed("invalid observer runoff".to_owned()));
                }
                land.river_forcings.push(civ_land::RiverForcing {
                    reach: f.reach(),
                    discharge_m3s: f.discharge_m3s(),
                    until_day: f.until_day(),
                    influence: f.influence(),
                });
            }
        }
        if schema >= Schema::V96 {
            let root = section_root::<save::FloodsSave>(&bytes)
                .map_err(|e| unreadable(SECTION_FLOODS, &e))?;
            for c in root.storm_checks().iter().flatten() {
                let building = required(c.building(), "a storm evaluation")?;
                if c.day() != now.day_index()
                    || civ_grammar::GroupKind::of_group(c.group())
                        != Some(civ_grammar::GroupKind::Covering)
                    || (c.observer() > 0
                        && !people.influences.list.iter().any(|i| {
                            i.id == c.observer()
                                && i.kind == civ_agents::influence::InfluenceKind::Storm
                        }))
                    || land.storm_checks.iter().any(|old| {
                        old.building == building
                            && old.group == c.group()
                            && old.day == c.day()
                            && old.observer == c.observer()
                    })
                {
                    return Err(LoadError::Malformed("invalid storm evaluation".to_owned()));
                }
                land.storm_checks.push(civ_land::StormCheck {
                    building,
                    group: c.group(),
                    day: c.day(),
                    observer: c.observer(),
                });
            }
        } else {
            // Old snapshots did not keep physical evaluations. Today's natural pulse has
            // passed; do not apply it again to the installed coverings on an old load.
            let day = now.day_index();
            for b in &land.buildings {
                for c in &b.condition {
                    if civ_grammar::GroupKind::of_group(c.group)
                        == Some(civ_grammar::GroupKind::Covering)
                    {
                        land.storm_checks.push(civ_land::StormCheck {
                            building: b.id,
                            group: c.group,
                            day,
                            observer: 0,
                        });
                    }
                }
            }
        }
        validate_flood_records(&people.floods, &people.influences, map.reaches.len(), now)?;
        let barriers: Vec<_> = land
            .banks
            .iter()
            .flat_map(|b| b.barriers(map, &land.earthworks, &land.floodplain.cell_hand))
            .collect();
        let changes =
            civ_land::banks::ground_changes(&land.banks, &land.earthworks, map, &land.ground);
        land.flood_depths = land.floodplain.flood_depths_with_ground(
            map,
            &land.reach_discharges,
            &barriers,
            &changes,
        );
    }
    // Shaking (schema 162); before, the ground never shook.
    if schema >= Schema::V162 {
        let bytes = single_chunk(reader, SECTION_QUAKES)?;
        people.quakes = decode_quakes(&bytes, &people.influences, now)?;
    }
    for c in &people.word.claims {
        if c.kind == civ_agents::word::ClaimKind::Fire {
            let f = c.fire.and_then(|id| people.fires.get(id)).ok_or_else(|| {
                LoadError::Malformed("fire account names no recorded fire".to_owned())
            })?;
            if schema < Schema::V99
                || c.subject != Some(f.building)
                || c.day < f.began.day_index()
                || c.day > now.day_index()
                || c.grievance.is_some()
                || c.suspected.is_some()
            {
                return Err(LoadError::Malformed(
                    "invalid found-burning account".to_owned(),
                ));
            }
        } else if c.fire.is_some() {
            return Err(LoadError::Malformed(
                "a non-fire claim names a fire".to_owned(),
            ));
        }
        // A raid names a recorded encounter, a band seen a recorded band (schema 111), a finding
        // a recorded case (schema 126, M7b slice BS).
        let named = match c.kind {
            civ_agents::word::ClaimKind::Raid => Some((
                c.event
                    .is_some_and(|id| people.order.clashes.iter().any(|x| x.id == id)),
                Schema::V111,
            )),
            civ_agents::word::ClaimKind::BandSeen => Some((
                c.event
                    .is_some_and(|id| people.order.bands.iter().any(|x| x.id == id)),
                Schema::V111,
            )),
            civ_agents::word::ClaimKind::Finding => Some((
                c.event
                    .is_some_and(|id| people.order.cases.iter().any(|x| x.id == id)),
                Schema::V126,
            )),
            _ => None,
        };
        match named {
            Some((true, since)) if schema >= since => {}
            None if c.event.is_none() => {}
            _ => {
                return Err(LoadError::Malformed(format!(
                    "claim {} names no recorded encounter, band or case",
                    c.id
                )));
            }
        }
    }
    for p in &people.fires.pulled {
        let bad = p.work_h <= 0.0
            || !p.work_h.is_finite()
            || p.people.len() < 2
            || p.people.windows(2).any(|w| w[0] >= w[1])
            || p.at > now
            || people.fires.get(p.fire).is_none_or(|f| p.at < f.began);
        if bad {
            return Err(LoadError::Malformed("invalid demolition record".to_owned()));
        }
    }
    for b in &land.buildings {
        if let Some(d) = b.demolition {
            let Some(f) = people.fires.get(d.fire) else {
                return Err(LoadError::Malformed(
                    "demolition names an unknown fire".to_owned(),
                ));
            };
            if d.last_minute > now.minutes()
                || (d.work_h > 0.0 && d.last_minute < 0)
                || (d.last_minute >= 0 && d.last_minute < f.began.minutes())
            {
                return Err(LoadError::Malformed("invalid demolition time".to_owned()));
            }
        }
    }
    // Settlements' founding ways (schema 61); before them a founding band's is drawn again from
    // its key, and any other settlement's taken as its households' now.
    if schema < Schema::V61 {
        people.derive_founding_ways(&land, &rules.people.style, seed);
    }
    people.derive_shelter(&land, &rules.catalog, &rules.people);
    match standing {
        Some(table) => people.standing = table,
        None => people.derive_standing(now, &rules.people),
    }

    let mut problems = land.problems(map, rules.land.habitats.len(), next_id);
    problems.extend(land.wear.problems());
    problems.extend(people.problems(next_id, rules.catalog.activities.len()));
    for (h, list) in &people.known_places.known {
        for k in list {
            if !land.settlements.iter().any(|s| s.id == k.settlement) {
                problems.push(format!(
                    "household {h} knows a missing settlement {}",
                    k.settlement
                ));
            }
        }
    }
    for b in &land.buildings {
        if let Some(f) = b.firm
            && people.firm(f).is_none()
        {
            problems.push(format!(
                "building {} is the workshop of a missing firm {f}",
                b.id
            ));
        }
    }
    let settlements: Vec<PermanentId> = land.settlements.iter().map(|s| s.id).collect();
    for (_, h) in people.households.iter() {
        if h.settlement.is_some_and(|s| !settlements.contains(&s)) {
            problems.push(format!("household {} names a missing settlement", h.id));
        }
    }
    for (i, p) in people.polities.iter().enumerate() {
        if !settlements.contains(&p.settlement) {
            problems.push(format!(
                "polity {} names missing settlement {}",
                p.id, p.settlement
            ));
        }
        // A polity taken in keeps its record, closed, and its settlement may be seat again of
        // one that broke away later (M6c slice BM): two still deciding is what is wrong.
        if p.ended.is_none()
            && people.polities[..i]
                .iter()
                .any(|o| o.holds(Some(p.settlement)))
        {
            problems.push(format!("settlement {} has two polities", p.settlement));
        }
        for (law, _) in &p.terms {
            if !p.laws.iter().any(|l| l.id == *law) {
                problems.push(format!(
                    "polity {} has a term of law {law}, not its own",
                    p.id
                ));
            }
        }
        for h in &p.held {
            if !settlements.contains(&h.settlement) {
                problems.push(format!(
                    "polity {} holds missing settlement {}",
                    p.id, h.settlement
                ));
            }
            let holders = people
                .polities
                .iter()
                .filter(|o| o.holds(Some(h.settlement)))
                .count();
            if p.ended.is_none() && holders != 1 {
                problems.push(format!(
                    "settlement {} is held by {holders} polities still deciding",
                    h.settlement
                ));
            }
        }
    }
    for (i, m) in people.markets.iter().enumerate() {
        if !settlements.contains(&m.settlement) {
            problems.push(format!(
                "a market names missing settlement {}",
                m.settlement
            ));
        }
        if people.markets[..i]
            .iter()
            .any(|o| o.settlement == m.settlement)
        {
            problems.push(format!("settlement {} has two markets", m.settlement));
        }
    }
    // A field, plot or building may belong to a household that is no more (it died out with no
    // kin to inherit): it stands abandoned. Its owner's id was allocated, which the land checks.
    if !problems.is_empty() {
        return Err(LoadError::Invalid(problems));
    }
    Ok(Decoded {
        land,
        people,
        events,
        redecide,
        place_deposits: schema < Schema::V19,
    })
}

// ---- polity ------------------------------------------------------------------------------------

fn encode_polities(pop: &Population, rules: &Rules, goods: &[&str]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let good_dictionary = strings(&mut fbb, goods);
    let policies: Vec<&str> = rules
        .catalog
        .policies
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    let policy_dictionary = strings(&mut fbb, &policies);
    let list: Vec<_> = pop
        .polities
        .iter()
        .map(|p| {
            let mut stores = p.stores.clone();
            stores.resize(goods.len(), 0.0);
            let stores = fbb.create_vector(&stores);
            let laws: Vec<_> = p
                .laws
                .iter()
                .map(|l| {
                    let stances: Vec<save::StanceSave> = l
                        .stances
                        .iter()
                        .map(|r| {
                            save::StanceSave::new(
                                r.person.get(),
                                r.household.get(),
                                r.gain,
                                r.regard,
                                stance_code(r.stance),
                            )
                        })
                        .collect();
                    let stances = fbb.create_vector(&stances);
                    let opinions: Vec<f32> = l.stances.iter().map(|r| r.opinion).collect();
                    let stance_opinions = fbb.create_vector(&opinions);
                    let values: Vec<f32> = l.stances.iter().map(|r| r.values).collect();
                    let stance_values = fbb.create_vector(&values);
                    // What views of another polity added, for an agreement (schema 66).
                    let views: Vec<f32> = l.stances.iter().map(|r| r.view).collect();
                    let stance_views =
                        (views.iter().any(|&v| v != 0.0)).then(|| fbb.create_vector(&views));
                    // What each household owed the sponsor (schema 131).
                    let owed: Vec<f32> = l.stances.iter().map(|r| r.owed).collect();
                    let stance_owed =
                        (owed.iter().any(|&v| v != 0.0)).then(|| fbb.create_vector(&owed));
                    let known: Vec<save::KnownSave> = l
                        .known
                        .iter()
                        .map(|&(person, day)| save::KnownSave::new(person.get(), day))
                        .collect();
                    let known = fbb.create_vector(&known);
                    let c = &l.compliance;
                    save::LawSave::create(
                        &mut fbb,
                        &save::LawSaveArgs {
                            id: l.id.get(),
                            policy: u32::from(l.policy),
                            levy_share: l.levy_share,
                            relief_days: l.relief_days,
                            compensation_days: l.sanction.compensation_days,
                            fine_days: l.sanction.fine_days,
                            exile: l.sanction.exile,
                            remove: l.sanction.remove,
                            yearly_days: l.sanction.yearly_days,
                            watch_rounds: l.watch.rounds,
                            watch_minutes: l.watch.minutes,
                            watch_cases: l.watch.cases,
                            watch_night: l.watch.night,
                            watch_tonight: l.watch.tonight,
                            watch_next: l.watch.next,
                            from_hour: l.hours.0,
                            to_hour: l.hours.1,
                            broken: c.broken,
                            broken_unaware: c.broken_unaware,
                            diverted: c.diverted,
                            diverted_kg: c.diverted_kg,
                            favoured: c.favoured,
                            passed_over: c.passed_over,
                            assessed: c.assessed,
                            asked: c.asked,
                            bribed: c.bribed,
                            bribe_kg: c.bribe_kg,
                            ends: l.ends.map_or(0, PermanentId::get),
                            agreement: l.agreement.map_or(0, PermanentId::get),
                            target: l.target.map_or(0, PermanentId::get),
                            status: law_status_code(l.status),
                            sponsor: l.sponsor.get(),
                            proposed: l.proposed.minutes(),
                            issue: issue_code(l.issue),
                            meets_day: l.meets_day,
                            decided: l.decided.is_some(),
                            decided_at: l.decided.map_or(0, SimTime::minutes),
                            outcome: l.outcome.map_or(u8::MAX, outcome_code),
                            eligible: l.eligible,
                            stances: Some(stances),
                            known: Some(known),
                            complied: c.complied,
                            could_not: c.could_not,
                            evaded: c.evaded,
                            unaware: c.unaware,
                            levied_kg: c.levied_kg,
                            withheld_kg: c.withheld_kg,
                            relieved: c.relieved,
                            relief_kg: c.relief_kg,
                            unanswered: c.unanswered,
                            refused: c.refused,
                            refused_kg: c.refused_kg,
                            holder: l.holder.map_or(0, PermanentId::get),
                            body_members: l.body.map_or(u8::MAX, |b| b.members.code()),
                            body_quorum: l.body.map_or(0.0, |b| b.quorum_share),
                            body_pass: l.body.map_or(0, |b| b.pass.code()),
                            body_seats: l.body.map_or(0, |b| Seats::pack(b.seats)),
                            stance_opinions: Some(stance_opinions),
                            stance_values: Some(stance_values),
                            stance_views,
                            stance_owed,
                        },
                    )
                })
                .collect();
            let laws = fbb.create_vector(&laws);
            let versions: Vec<save::CustomVersionSave> = p
                .versions
                .iter()
                .map(|v| {
                    save::CustomVersionSave::new(
                        v.body.members.code(),
                        v.body.pass.code(),
                        v.body.quorum_share,
                        v.since.minutes(),
                        v.law.map_or(0, PermanentId::get),
                    )
                })
                .collect();
            let versions = fbb.create_vector(&versions);
            let seized: Vec<u64> = p
                .versions
                .iter()
                .map(|v| v.seized_by.map_or(0, PermanentId::get))
                .collect();
            let seized_by = fbb.create_vector(&seized);
            // The places its laws claim (schema 62).
            let claimed: Vec<save::PlaceClaimSave> = p
                .claimed
                .iter()
                .map(|&(law, place)| {
                    let (kind, id) = place.code();
                    save::PlaceClaimSave::new(law.get(), kind, id)
                })
                .collect();
            let claimed = (!claimed.is_empty()).then(|| fbb.create_vector(&claimed));
            // The sources its laws close (schema 82).
            let closed: Vec<save::PlaceClaimSave> = p
                .closed
                .iter()
                .map(|&(law, place)| {
                    let (kind, id) = place.code();
                    save::PlaceClaimSave::new(law.get(), kind, id)
                })
                .collect();
            let closed = (!closed.is_empty()).then(|| fbb.create_vector(&closed));
            // Where its laws would build crossings (schema 70).
            let sites: Vec<_> = p
                .sites
                .iter()
                .map(|&(law, site)| {
                    let system = rules
                        .catalog
                        .bridges
                        .get(usize::from(site.system))
                        .map_or("", |d| d.id.as_str());
                    let system = fbb.create_string(system);
                    save::CrossingSiteSave::create(
                        &mut fbb,
                        &save::CrossingSiteSaveArgs {
                            law: law.get(),
                            cell: site.cell,
                            system: Some(system),
                            span_m: site.span_m,
                            labour_h: site.labour_h,
                        },
                    )
                })
                .collect();
            let sites = (!sites.is_empty()).then(|| fbb.create_vector(&sites));
            // Where its laws would dig wells (schema 83).
            let wells: Vec<_> = p
                .wells
                .iter()
                .map(|&(law, site)| {
                    let system = rules
                        .catalog
                        .wells
                        .get(usize::from(site.system))
                        .map_or("", |d| d.id.as_str());
                    let system = fbb.create_string(system);
                    save::PublicWellSave::create(
                        &mut fbb,
                        &save::PublicWellSaveArgs {
                            law: law.get(),
                            cell: site.cell,
                            system: Some(system),
                            x_cm: site.rect.x,
                            y_cm: site.rect.y,
                            w_cm: site.rect.w,
                            h_cm: site.rect.h,
                            target_m: site.target_m,
                            labour_h: site.labour_h,
                        },
                    )
                })
                .collect();
            let wells = (!wells.is_empty()).then(|| fbb.create_vector(&wells));
            // Where its laws would build flood banks, what each household reckoned they would
            // save it, and what each has given (schema 102).
            let banks: Vec<_> = p
                .banks
                .iter()
                .map(|&(law, s)| {
                    save::BankSiteSave::create(
                        &mut fbb,
                        &save::BankSiteSaveArgs {
                            law: law.get(),
                            reach: s.reach,
                            ax: s.a[0],
                            ay: s.a[1],
                            bx: s.b[0],
                            by: s.b[1],
                            height_m: s.height_m,
                            crest_width_m: s.crest_width_m,
                            side_run: s.side_run,
                            compacted_per_bank: s.compacted_per_bank,
                            env_x_cm: s.envelope.x,
                            env_y_cm: s.envelope.y,
                            env_w_cm: s.envelope.w,
                            env_h_cm: s.envelope.h,
                            pit_x_cm: s.pit.x,
                            pit_y_cm: s.pit.y,
                            pit_w_cm: s.pit.w,
                            pit_h_cm: s.pit.h,
                            labour_h: s.labour_h,
                            upkeep_h: s.upkeep_h,
                            mending: s.mending,
                            work: s.work.map_or(0, PermanentId::get),
                        },
                    )
                })
                .collect();
            let banks = (!banks.is_empty()).then(|| fbb.create_vector(&banks));
            let banks_failed: Vec<u64> = p.banks_failed.iter().map(|w| w.get()).collect();
            let banks_failed = (!banks_failed.is_empty()).then(|| fbb.create_vector(&banks_failed));
            let bank_keeps: Vec<_> = p
                .bank_keeps
                .iter()
                .map(|(k, b)| save::BankKeepRecord::new(k.get(), b.get()))
                .collect();
            let bank_keeps = (!bank_keeps.is_empty()).then(|| fbb.create_vector(&bank_keeps));
            let bank_saves = bank_share_rows(&mut fbb, &p.bank_saves);
            let bank_given = bank_share_rows(&mut fbb, &p.bank_given);
            // Where its laws would raise palisades, and what each household reckoned they
            // would keep it (schema 114).
            let enclosures: Vec<_> = p
                .enclosures
                .iter()
                .map(|&(law, s)| {
                    save::EnclosureSiteSave::create(
                        &mut fbb,
                        &save::EnclosureSiteSaveArgs {
                            law: law.get(),
                            x: s.centre.0,
                            y: s.centre.1,
                            radius_m: s.radius_m,
                            gates: s.gates,
                            inside: s.inside,
                            labour_h: s.labour_h,
                            work: s.work.map_or(0, PermanentId::get),
                        },
                    )
                })
                .collect();
            let enclosures = (!enclosures.is_empty()).then(|| fbb.create_vector(&enclosures));
            let enclosure_saves = bank_share_rows(&mut fbb, &p.enclosure_saves);
            // Where its laws would clear passing places, what each household reckoned they
            // would spare it, and the hours given (schema 150).
            let ways: Vec<_> = p
                .ways
                .iter()
                .map(|&(law, s)| {
                    save::WaySiteSave::create(
                        &mut fbb,
                        &save::WaySiteSaveArgs {
                            law: law.get(),
                            cell: s.cell,
                            labour_h: s.labour_h,
                            work: s.work.map_or(0, PermanentId::get),
                        },
                    )
                })
                .collect();
            let ways = (!ways.is_empty()).then(|| fbb.create_vector(&ways));
            let way_saves = bank_share_rows(&mut fbb, &p.way_saves);
            let way_given = bank_share_rows(&mut fbb, &p.way_given);
            // The plans its laws lay, as realised, and what it saw of its ground (schema 156).
            let plans: Vec<_> = p
                .plans
                .iter()
                .map(|(law, plan)| {
                    let l = &plan.layout;
                    let rect = |r: &civ_land::RectCm| save::PlanRectSave::new(r.x, r.y, r.w, r.h);
                    let streets: Vec<_> = l.streets.iter().map(rect).collect();
                    let streets = fbb.create_vector(&streets);
                    let sites: Vec<_> = l
                        .sites
                        .iter()
                        .map(|t| {
                            save::PlanSiteSave::new(
                                t.rect.x,
                                t.rect.y,
                                t.rect.w,
                                t.rect.h,
                                t.use_.code(),
                            )
                        })
                        .collect();
                    let sites = fbb.create_vector(&sites);
                    let parcels: Vec<_> = l
                        .parcels
                        .iter()
                        .map(|q| {
                            save::PlanParcelSave::new(
                                q.rect.x, q.rect.y, q.rect.w, q.rect.h, q.block,
                            )
                        })
                        .collect();
                    let parcels = fbb.create_vector(&parcels);
                    let refused: Vec<u64> = plan.refused_fields.iter().map(|f| f.get()).collect();
                    let refused = (!refused.is_empty()).then(|| fbb.create_vector(&refused));
                    let g = &plan.grid;
                    save::PlanSave::create(
                        &mut fbb,
                        &save::PlanSaveArgs {
                            law: law.get(),
                            version: plan.version,
                            origin_x: l.origin.0,
                            origin_y: l.origin.1,
                            axis: l.axis.map_or(0, civ_land::plans::Axis::code),
                            block_along: g.block_m.0,
                            block_across: g.block_m.1,
                            frontage: g.parcel_m.0,
                            depth: g.parcel_m.1,
                            street: g.street_m,
                            blocks_along: g.blocks.0,
                            blocks_across: g.blocks.1,
                            square: g.square,
                            streets: Some(streets),
                            sites: Some(sites),
                            parcels: Some(parcels),
                            refused_streets: l.refused.streets,
                            refused_sites: l.refused.sites,
                            refused_parcels: l.refused.parcels,
                            staked_h: plan.staked_h,
                            refused_fields: refused,
                        },
                    )
                })
                .collect();
            let plans = (!plans.is_empty()).then(|| fbb.create_vector(&plans));
            let ground_seen: Vec<_> = p
                .ground_seen
                .iter()
                .map(|g| {
                    save::GroundSeenSave::new(
                        g.day,
                        g.settlement.get(),
                        g.household.get(),
                        issue_code(g.issue),
                    )
                })
                .collect();
            let ground_seen = (!ground_seen.is_empty()).then(|| fbb.create_vector(&ground_seen));
            // The buildings begun on ground a plan reserves (schema 157).
            let encroachments: Vec<_> = p
                .encroachments
                .iter()
                .map(|e| {
                    save::EncroachmentSave::new(
                        e.day,
                        e.law.get(),
                        e.household.get(),
                        e.building.get(),
                    )
                })
                .collect();
            let encroachments =
                (!encroachments.is_empty()).then(|| fbb.create_vector(&encroachments));
            // Who saw each begun (schema 160).
            let encroachment_seen: Vec<_> = p
                .encroachments
                .iter()
                .enumerate()
                .flat_map(|(i, e)| {
                    e.seen_by
                        .iter()
                        .map(move |s| save::EncroachmentSeenSave::new(i as u32, s.get()))
                })
                .collect();
            let encroachment_seen =
                (!encroachment_seen.is_empty()).then(|| fbb.create_vector(&encroachment_seen));
            // What a finding ordered for each (schema 161).
            let encroachment_orders: Vec<_> = p
                .encroachments
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    let o = e.order?;
                    let (day, kcal) = o.day_kcal();
                    Some(save::EncroachmentOrderSave::new(
                        i as u32,
                        o.code(),
                        day,
                        kcal,
                    ))
                })
                .collect();
            let encroachment_orders =
                (!encroachment_orders.is_empty()).then(|| fbb.create_vector(&encroachment_orders));
            // The settlements it holds besides its seat, and its end (schema 118).
            let held: Vec<_> = p
                .held
                .iter()
                .map(|h| {
                    save::HeldSettlementSave::create(
                        &mut fbb,
                        &save::HeldSettlementSaveArgs {
                            settlement: h.settlement.get(),
                            since: h.since,
                            how: h.how.code(),
                            from: h.from.get(),
                        },
                    )
                })
                .collect();
            let held = (!held.is_empty()).then(|| fbb.create_vector(&held));
            // Its office laws held for a term (schema 120).
            let terms: Vec<save::TermSave> = p
                .terms
                .iter()
                .map(|&(law, t)| save::TermSave::new(law.get(), t.until, t.pool))
                .collect();
            let terms = (!terms.is_empty()).then(|| fbb.create_vector(&terms));
            // Its council: each version's seats, the seats held and its elections (schema 121).
            let version_seats: Vec<u64> = p
                .versions
                .iter()
                .map(|v| Seats::pack(v.body.seats))
                .collect();
            let version_seats = version_seats
                .iter()
                .any(|&v| v != 0)
                .then(|| fbb.create_vector(&version_seats));
            let seated: Vec<save::SeatSave> = p
                .seated
                .iter()
                .map(|x| save::SeatSave::new(x.seat, x.holder.get(), x.since, x.until, x.election))
                .collect();
            let seated = (!seated.is_empty()).then(|| fbb.create_vector(&seated));
            let elections: Vec<_> = p
                .elections
                .iter()
                .map(|e| {
                    let considered: Vec<u64> = e.considered.iter().map(|p| p.get()).collect();
                    let considered = fbb.create_vector(&considered);
                    let candidates: Vec<save::CandidateSave> = e
                        .candidates
                        .iter()
                        .map(|c| save::CandidateSave::new(c.person.get(), c.day))
                        .collect();
                    let candidates = fbb.create_vector(&candidates);
                    let ballots: Vec<_> = e
                        .ballots
                        .iter()
                        .map(|(voter, named)| {
                            let named: Vec<u64> = named.iter().map(|p| p.get()).collect();
                            let named = fbb.create_vector(&named);
                            save::BallotSave::create(
                                &mut fbb,
                                &save::BallotSaveArgs {
                                    voter: voter.get(),
                                    named: Some(named),
                                },
                            )
                        })
                        .collect();
                    let ballots = fbb.create_vector(&ballots);
                    let totals: Vec<save::VoteSave> = e
                        .totals
                        .iter()
                        .map(|&(c, v)| save::VoteSave::new(c.get(), v))
                        .collect();
                    let totals = fbb.create_vector(&totals);
                    let winners: Vec<u64> = e.winners.iter().map(|p| p.get()).collect();
                    let winners = fbb.create_vector(&winners);
                    let positions: Vec<save::StoodSave> = e
                        .candidates
                        .iter()
                        .flat_map(|c| {
                            c.positions
                                .iter()
                                .map(|&(q, x)| save::StoodSave::new(c.person.get(), q, x))
                        })
                        .collect();
                    let positions = (!positions.is_empty()).then(|| fbb.create_vector(&positions));
                    let endorsements: Vec<_> = e
                        .endorsements
                        .iter()
                        .map(|(faction, named)| {
                            let named: Vec<u64> = named.iter().map(|p| p.get()).collect();
                            let named = fbb.create_vector(&named);
                            save::SlateSave::create(
                                &mut fbb,
                                &save::SlateSaveArgs {
                                    faction: faction.get(),
                                    named: Some(named),
                                },
                            )
                        })
                        .collect();
                    let endorsements =
                        (!endorsements.is_empty()).then(|| fbb.create_vector(&endorsements));
                    let districts: Vec<save::DistrictSave> = e
                        .districts
                        .iter()
                        .map(|&(s, n)| save::DistrictSave::new(s.get(), n))
                        .collect();
                    let districts = (!districts.is_empty()).then(|| fbb.create_vector(&districts));
                    let alliances: Vec<save::AllianceSave> = e
                        .alliances
                        .iter()
                        .map(|a| {
                            save::AllianceSave::new(
                                a.factions[0].get(),
                                a.factions[1].get(),
                                a.package.code(),
                            )
                        })
                        .collect();
                    let alliances = (!alliances.is_empty()).then(|| fbb.create_vector(&alliances));
                    let candidate_districts =
                        e.candidates.iter().any(|c| c.district.is_some()).then(|| {
                            let ds: Vec<u64> = e
                                .candidates
                                .iter()
                                .map(|c| c.district.map_or(0, |d| d.get()))
                                .collect();
                            fbb.create_vector(&ds)
                        });
                    save::ElectionSave::create(
                        &mut fbb,
                        &save::ElectionSaveArgs {
                            id: e.id,
                            why: e.why.code(),
                            seats: Seats::pack(Some(e.seats)),
                            called: e.called,
                            stand_until: e.stand_until,
                            poll_day: e.poll_day,
                            stage: e.stage.code(),
                            considered: Some(considered),
                            candidates: Some(candidates),
                            ballots: Some(ballots),
                            totals: Some(totals),
                            winners: Some(winners),
                            by_lot: e.by_lot,
                            ended: e.ended.unwrap_or(-1),
                            until: e.until,
                            positions,
                            endorsements,
                            districts,
                            candidate_districts,
                            alliances,
                        },
                    )
                })
                .collect();
            let elections = (!elections.is_empty()).then(|| fbb.create_vector(&elections));
            // Patron gifts (schema 131, M7c slice BY).
            let gifts: Vec<save::PatronGiftSave> = p
                .patron_gifts
                .iter()
                .map(|g| {
                    save::PatronGiftSave::new(
                        g.patron.get(),
                        g.giver.get(),
                        g.client.get(),
                        g.law.get(),
                        g.day,
                        g.kcal,
                        g.days,
                    )
                })
                .collect();
            let patron_gifts = (!gifts.is_empty()).then(|| fbb.create_vector(&gifts));
            // Measured losses (schema 164, M8d slice DC).
            let measured: Vec<save::MeasuredSave> = p
                .measured
                .iter()
                .map(|m| {
                    save::MeasuredSave::new(
                        m.year,
                        m.spoiled_kcal,
                        m.gatherings,
                        m.wet_gatherings,
                        m.roof_fires,
                    )
                })
                .collect();
            let measured = (!measured.is_empty()).then(|| fbb.create_vector(&measured));
            // Losses measured beside them (schema 166, M8d slice DC).
            let losses: Vec<_> = p
                .measured
                .iter()
                .map(|m| {
                    save::MeasuredLossSave::create(
                        &mut fbb,
                        &save::MeasuredLossSaveArgs {
                            year: m.year,
                            wet_lost_h: m.wet_lost_h,
                            late_fire_h: m.late_fire_h,
                        },
                    )
                })
                .collect();
            let measured_losses = (!losses.is_empty()).then(|| fbb.create_vector(&losses));
            // Buildings raised in common (schema 165, M8d slice DC).
            let raises: Vec<_> = p
                .raises
                .iter()
                .map(|&(law, s)| {
                    save::RaiseSiteSave::create(
                        &mut fbb,
                        &save::RaiseSiteSaveArgs {
                            law: law.get(),
                            program: s.program,
                            labour_h: s.labour_h,
                            building: s.building.map_or(0, PermanentId::get),
                            begun: s.begun.map_or(-1, SimTime::minutes),
                            last_work: s.last_work.map_or(-1, SimTime::minutes),
                            first_use: s.first_use.map_or(-1, SimTime::minutes),
                            finished: s.finished.map_or(-1, SimTime::minutes),
                            stalled: s.stalled.map_or(-1, SimTime::minutes),
                        },
                    )
                })
                .collect();
            let raises = (!raises.is_empty()).then(|| fbb.create_vector(&raises));
            let raise_given = bank_share_rows(&mut fbb, &p.raise_given);
            let gathering = p.gathering.as_ref().map(|g| {
                let stakes: Vec<save::StakeSave> = g
                    .stakes
                    .iter()
                    .map(|&(h, points)| save::StakeSave::new(h.get(), points))
                    .collect();
                let stakes = fbb.create_vector(&stakes);
                let present: Vec<u64> = g.present.iter().map(|p| p.get()).collect();
                let present = fbb.create_vector(&present);
                let cases = fbb.create_vector(&g.cases);
                save::GatheringSave::create(
                    &mut fbb,
                    &save::GatheringSaveArgs {
                        law: g.law.map_or(0, PermanentId::get),
                        day: g.day,
                        stakes: Some(stakes),
                        present: Some(present),
                        cases: Some(cases),
                    },
                )
            });
            save::PolitySave::create(
                &mut fbb,
                &save::PolitySaveArgs {
                    id: p.id.get(),
                    settlement: p.settlement.get(),
                    founded: p.founded.minutes(),
                    members: p.body.members.code(),
                    quorum_share: p.body.quorum_share,
                    pass: p.body.pass.code(),
                    stores: Some(stores),
                    stores_at: p.stores_at.minutes(),
                    laws: Some(laws),
                    gathering,
                    reviewed: p.reviewed,
                    versions: Some(versions),
                    seized_by: Some(seized_by),
                    claimed,
                    sites,
                    closed,
                    wells,
                    banks,
                    bank_saves,
                    bank_given,
                    banks_failed,
                    bank_keeps,
                    enclosures,
                    enclosure_saves,
                    held,
                    ended_day: p.ended.map_or(-1, |e| e.day),
                    ended_by: p.ended.map_or(0, |e| e.by.get()),
                    ended_how: p.ended.map_or(255, |e| e.how.code()),
                    terms,
                    seats: Seats::pack(p.body.seats),
                    version_seats,
                    seated,
                    elections,
                    patron_gifts,
                    ways,
                    way_saves,
                    way_given,
                    plans,
                    ground_seen,
                    encroachments,
                    encroachment_seen,
                    encroachment_orders,
                    measured,
                    raises,
                    raise_given,
                    storehouse: p.storehouse.map_or(0, PermanentId::get),
                    hall: p.hall.map_or(0, PermanentId::get),
                    tower: p.tower.map_or(0, PermanentId::get),
                    measured_losses,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Polities::create(
        &mut fbb,
        &save::PolitiesArgs {
            polities: Some(list),
            goods: Some(good_dictionary),
            policies: Some(policy_dictionary),
        },
    );
    finish(fbb, root)
}

fn stance_code(s: civ_agents::polity::Stance) -> u8 {
    use civ_agents::polity::Stance;
    match s {
        Stance::Support => 0,
        Stance::Oppose => 1,
        Stance::Abstain => 2,
    }
}

fn stance_from(code: u8) -> Option<civ_agents::polity::Stance> {
    use civ_agents::polity::Stance;
    match code {
        0 => Some(Stance::Support),
        1 => Some(Stance::Oppose),
        2 => Some(Stance::Abstain),
        _ => None,
    }
}

fn law_status_code(s: civ_agents::polity::LawStatus) -> u8 {
    use civ_agents::polity::LawStatus;
    match s {
        LawStatus::Proposed => 0,
        LawStatus::InForce => 1,
        LawStatus::Rejected => 2,
        LawStatus::Lapsed => 3,
        LawStatus::Superseded => 4,
        LawStatus::Carried => 5,
        LawStatus::StruckDown => 6,
    }
}

/// A polity's bank shares (law, household, hours) as saved rows (schema 102), absent when none.
#[allow(clippy::type_complexity)]
fn bank_share_rows<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    list: &[(PermanentId, PermanentId, f32)],
) -> Option<
    flatbuffers::WIPOffset<
        flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<save::BankShareSave<'a>>>,
    >,
> {
    let rows: Vec<_> = list
        .iter()
        .map(|&(law, h, hours)| {
            save::BankShareSave::create(
                fbb,
                &save::BankShareSaveArgs {
                    law: law.get(),
                    household: h.get(),
                    hours,
                },
            )
        })
        .collect();
    (!rows.is_empty()).then(|| fbb.create_vector(&rows))
}

fn issue_code(i: civ_agents::polity::IssueKind) -> u8 {
    use civ_agents::polity::IssueKind;
    match i {
        IssueKind::FoodShort => 0,
        IssueKind::StoreUnkept => 1,
        IssueKind::Takings => 2,
        IssueKind::Overruled => 3,
        IssueKind::Petition => 4,
        IssueKind::Founding => 5,
        IssueKind::Outsiders => 6,
        IssueKind::ClaimedFromUs => 7,
        IssueKind::TermsSought => 8,
        IssueKind::Fords => 9,
        IssueKind::Sickness => 10,
        IssueKind::SourceSuspected => 11,
        IssueKind::WaterFar => 12,
        IssueKind::Flooded => 13,
        IssueKind::BankWorn => 14,
        IssueKind::Burned => 15,
        IssueKind::GaveWay => 16,
        IssueKind::Raided => 17,
        IssueKind::Holding => 18,
        IssueKind::CasesWaiting => 19,
        IssueKind::FetchedElsewhere => 20,
        IssueKind::DebtsUnpaid => 21,
        IssueKind::Weighed => 22,
        IssueKind::CoinIdle => 23,
        IssueKind::CartsWait => 24,
        IssueKind::NoGround => 25,
        IssueKind::WayBuiltOver => 26,
        IssueKind::NewGround => 27,
        IssueKind::StoreSpoils => 28,
        IssueKind::RainedOff => 29,
        IssueKind::FiresUnseen => 30,
        IssueKind::LoadsCarried => 31,
        IssueKind::DarkTakings => 32,
        IssueKind::Untaught => 33,
        IssueKind::UnknowingCare => 34,
        IssueKind::Contagion => 35,
    }
}

fn outcome_code(o: civ_agents::polity::Outcome) -> u8 {
    o as u8
}

/// The polities of a save. A law naming a policy template the loaded content lacks, or any code
/// this build does not know, refuses the save: a constitution is never rewritten to fit
/// (ADR-0013 §2).
fn decode_polities(
    bytes: &[u8],
    rules: &Rules,
) -> Result<Vec<civ_agents::polity::Polity>, LoadError> {
    use civ_agents::polity::{
        Body, Compliance, CustomVersion, Gathering, IssueKind, Law, LawStatus, Membership, Outcome,
        PassRule, Polity, Stance, StanceRecord,
    };
    let root = section_root::<save::Polities>(bytes).map_err(|e| unreadable(SECTION_POLITY, &e))?;
    let goods = good_map(&read_strings(root.goods()), rules);
    let policies: Vec<Option<u16>> = read_strings(root.policies())
        .iter()
        .map(|id| {
            rules
                .catalog
                .policies
                .iter()
                .position(|p| &p.id == id)
                .and_then(|i| u16::try_from(i).ok())
        })
        .collect();
    let saved_policies = read_strings(root.policies());
    let bad = |what: String| LoadError::Malformed(what);
    let mut out = Vec::new();
    for p in root.polities().iter().flatten() {
        let id = required(p.id(), "a polity")?;
        let settlement = required(p.settlement(), "a polity's settlement")?;
        // A body's seats come with a council's membership and only with it (schema 121).
        let body_of = |members: u8, quorum_share: f32, pass: u8, seats: u64, what: &str| {
            let members = Membership::from_code(members)
                .ok_or_else(|| bad(format!("{what} has membership code {members}")))?;
            let seats = Seats::unpack(seats).map_err(|e| bad(format!("{what}: {e}")))?;
            if (members == Membership::Seats) != seats.is_some() {
                return Err(bad(format!(
                    "{what} is a council without seats, or seats without one"
                )));
            }
            Ok::<Body, LoadError>(Body {
                members,
                quorum_share,
                pass: PassRule::from_code(pass)
                    .ok_or_else(|| bad(format!("{what} has decision rule code {pass}")))?,
                seats,
            })
        };
        let body = body_of(
            p.members(),
            p.quorum_share(),
            p.pass(),
            p.seats(),
            &format!("polity {id}"),
        )?;
        let saved: Vec<f64> = p.stores().map(|v| v.iter().collect()).unwrap_or_default();
        if saved.len() != goods.len() {
            return Err(bad(format!(
                "polity {id} stores {} goods of {}",
                saved.len(),
                goods.len()
            )));
        }
        let mut stores = vec![0.0; rules.catalog.goods.len()];
        for (kg, g) in saved.iter().zip(&goods) {
            if let Some(g) = g {
                stores[*g] += kg;
            }
        }
        let mut laws = Vec::new();
        for l in p.laws().iter().flatten() {
            let law = required(l.id(), "a law")?;
            let policy = match policies.get(l.policy() as usize) {
                Some(Some(k)) => *k,
                Some(None) => {
                    return Err(LoadError::Incompatible(format!(
                        "law {law} is a `{}`, which the loaded content does not define",
                        saved_policies[l.policy() as usize]
                    )));
                }
                None => {
                    return Err(bad(format!(
                        "law {law} names policy {} of {}",
                        l.policy(),
                        policies.len()
                    )));
                }
            };
            let status = match l.status() {
                0 => LawStatus::Proposed,
                1 => LawStatus::InForce,
                2 => LawStatus::Rejected,
                3 => LawStatus::Lapsed,
                4 => LawStatus::Superseded,
                5 => LawStatus::Carried,
                6 => LawStatus::StruckDown,
                c => return Err(bad(format!("law {law} has status code {c}"))),
            };
            let issue = match l.issue() {
                0 => IssueKind::FoodShort,
                1 => IssueKind::StoreUnkept,
                2 => IssueKind::Takings,
                3 => IssueKind::Overruled,
                4 => IssueKind::Petition,
                5 => IssueKind::Founding,
                6 => IssueKind::Outsiders,
                7 => IssueKind::ClaimedFromUs,
                8 => IssueKind::TermsSought,
                9 => IssueKind::Fords,
                10 => IssueKind::Sickness,
                11 => IssueKind::SourceSuspected,
                12 => IssueKind::WaterFar,
                13 => IssueKind::Flooded,
                14 => IssueKind::BankWorn,
                15 => IssueKind::Burned,
                16 => IssueKind::GaveWay,
                17 => IssueKind::Raided,
                18 => IssueKind::Holding,
                19 => IssueKind::CasesWaiting,
                20 => IssueKind::FetchedElsewhere,
                21 => IssueKind::DebtsUnpaid,
                22 => IssueKind::Weighed,
                23 => IssueKind::CoinIdle,
                24 => IssueKind::CartsWait,
                25 => IssueKind::NoGround,
                26 => IssueKind::WayBuiltOver,
                27 => IssueKind::NewGround,
                28 => IssueKind::StoreSpoils,
                29 => IssueKind::RainedOff,
                30 => IssueKind::FiresUnseen,
                31 => IssueKind::LoadsCarried,
                32 => IssueKind::DarkTakings,
                33 => IssueKind::Untaught,
                34 => IssueKind::UnknowingCare,
                35 => IssueKind::Contagion,
                c => return Err(bad(format!("law {law} has issue code {c}"))),
            };
            let outcome = match l.outcome() {
                u8::MAX => None,
                0 => Some(Outcome::Passed),
                1 => Some(Outcome::Failed),
                2 => Some(Outcome::Tied),
                3 => Some(Outcome::NoQuorum),
                c => return Err(bad(format!("law {law} has outcome code {c}"))),
            };
            let mut stances = Vec::new();
            let opinions: Vec<f32> = l
                .stance_opinions()
                .map(|v| v.iter().collect())
                .unwrap_or_default();
            let values: Vec<f32> = l
                .stance_values()
                .map(|v| v.iter().collect())
                .unwrap_or_default();
            let views: Vec<f32> = l
                .stance_views()
                .map(|v| v.iter().collect())
                .unwrap_or_default();
            let owed: Vec<f32> = l
                .stance_owed()
                .map(|v| v.iter().collect())
                .unwrap_or_default();
            for (k, r) in l.stances().iter().flatten().enumerate() {
                stances.push(StanceRecord {
                    person: required(r.person(), "a stance")?,
                    household: required(r.household(), "a stance's household")?,
                    stance: match r.stance() {
                        0 => Stance::Support,
                        1 => Stance::Oppose,
                        2 => Stance::Abstain,
                        c => return Err(bad(format!("law {law} has stance code {c}"))),
                    },
                    gain: r.gain(),
                    regard: r.regard(),
                    opinion: opinions.get(k).copied().unwrap_or(0.0),
                    values: values.get(k).copied().unwrap_or(0.0),
                    view: views.get(k).copied().unwrap_or(0.0),
                    owed: owed.get(k).copied().unwrap_or(0.0),
                });
            }
            let mut known = Vec::new();
            for k in l.known().iter().flatten() {
                known.push((required(k.person(), "a law's knower")?, k.day()));
            }
            if !known.windows(2).all(|w| w[0].0 < w[1].0) {
                return Err(bad(format!("law {law} lists who knows it out of order")));
            }
            laws.push(Law {
                id: law,
                policy,
                kind: rules.catalog.policies[usize::from(policy)].kind,
                levy_share: l.levy_share(),
                holder: PermanentId::from_raw(l.holder()),
                relief_days: l.relief_days(),
                sanction: civ_agents::polity::Sanction {
                    compensation_days: l.compensation_days(),
                    fine_days: l.fine_days(),
                    exile: l.exile(),
                    remove: l.remove(),
                    yearly_days: l.yearly_days(),
                },
                hours: (l.from_hour(), l.to_hour()),
                status,
                ends: PermanentId::from_raw(l.ends()),
                agreement: PermanentId::from_raw(l.agreement()),
                target: PermanentId::from_raw(l.target()),
                sponsor: required(l.sponsor(), "a law's sponsor")?,
                proposed: time(l.proposed()),
                issue,
                meets_day: l.meets_day(),
                decided: l.decided().then(|| time(l.decided_at())),
                outcome,
                eligible: l.eligible(),
                stances,
                known,
                compliance: Compliance {
                    complied: l.complied(),
                    could_not: l.could_not(),
                    evaded: l.evaded(),
                    unaware: l.unaware(),
                    levied_kg: l.levied_kg(),
                    withheld_kg: l.withheld_kg(),
                    relieved: l.relieved(),
                    relief_kg: l.relief_kg(),
                    unanswered: l.unanswered(),
                    broken: l.broken(),
                    broken_unaware: l.broken_unaware(),
                    diverted: l.diverted(),
                    diverted_kg: l.diverted_kg(),
                    favoured: l.favoured(),
                    passed_over: l.passed_over(),
                    assessed: l.assessed(),
                    asked: l.asked(),
                    bribed: l.bribed(),
                    bribe_kg: l.bribe_kg(),
                    refused: l.refused(),
                    refused_kg: l.refused_kg(),
                },
                watch: civ_agents::polity::WatchRecord {
                    rounds: l.watch_rounds(),
                    minutes: l.watch_minutes(),
                    cases: l.watch_cases(),
                    night: l.watch_night(),
                    tonight: l.watch_tonight(),
                    next: l.watch_next(),
                },
                body: match l.body_members() {
                    u8::MAX => None,
                    m => Some(body_of(
                        m,
                        l.body_quorum(),
                        l.body_pass(),
                        l.body_seats(),
                        &format!("law {law}"),
                    )?),
                },
            });
        }
        let gathering = match p.gathering() {
            None => None,
            Some(g) => {
                let mut stakes = Vec::new();
                for s in g.stakes().iter().flatten() {
                    stakes.push((required(s.household(), "a stake")?, s.points()));
                }
                let mut present = Vec::new();
                for raw in g.present().iter().flatten() {
                    present.push(required(raw, "someone at a gathering")?);
                }
                if !stakes.windows(2).all(|w| w[0].0 < w[1].0)
                    || !present.windows(2).all(|w| w[0] < w[1])
                {
                    return Err(bad(format!("polity {id}'s gathering is out of order")));
                }
                // Called on cases only (saves 32), or on a law that must be there.
                let law = id_of(g.law());
                if let Some(law) = law
                    && !laws.iter().any(|l: &Law| l.id == law)
                {
                    return Err(bad(format!(
                        "polity {id}'s gathering names missing law {law}"
                    )));
                }
                let cases: Vec<u32> = g.cases().map(|v| v.iter().collect()).unwrap_or_default();
                if law.is_none() && cases.is_empty() {
                    return Err(bad(format!(
                        "polity {id}'s gathering has neither a law nor a case"
                    )));
                }
                Some(Gathering {
                    law,
                    day: g.day(),
                    stakes,
                    present,
                    cases,
                })
            }
        };
        // The custom's versions (schema 36); before, the founding custom alone.
        let mut versions = Vec::new();
        let seized: Vec<u64> = p
            .seized_by()
            .map(|v| v.iter().collect())
            .unwrap_or_default();
        // Each version's council seats (schema 121); before, none.
        let version_seats: Vec<u64> = p
            .version_seats()
            .map(|v| v.iter().collect())
            .unwrap_or_default();
        for (k, v) in p.versions().iter().flatten().enumerate() {
            versions.push(CustomVersion {
                body: body_of(
                    v.members(),
                    v.quorum_share(),
                    v.pass(),
                    version_seats.get(k).copied().unwrap_or(0),
                    &format!("polity {id}'s custom"),
                )?,
                since: time(v.since()),
                law: id_of(v.law()),
                seized_by: seized.get(k).copied().and_then(PermanentId::from_raw),
            });
        }
        if versions.is_empty() {
            versions.push(CustomVersion {
                body,
                since: time(p.founded()),
                law: None,
                seized_by: None,
            });
        }
        if versions.last().map(|v| v.body) != Some(body) {
            return Err(bad(format!("polity {id}'s custom is not its last version")));
        }
        // The places its laws claim (schema 62); before, none.
        let mut claimed = Vec::new();
        for c in p.claimed().iter().flatten() {
            let (Some(law), Some(place)) = (
                PermanentId::from_raw(c.law()),
                civ_agents::uses::Place::from_code(c.kind(), c.place()),
            ) else {
                return Err(bad(format!(
                    "polity {id} claims place {} of kind {} by law {}",
                    c.place(),
                    c.kind(),
                    c.law()
                )));
            };
            if !laws.iter().any(|l| l.id == law) {
                return Err(bad(format!(
                    "polity {id} claims a place by a missing law {law}"
                )));
            }
            claimed.push((law, place));
        }
        // Where its laws would build crossings (schema 70); before, nowhere. One of a bridge
        // system the loaded content lacks is dropped: its law can build nothing.
        let mut sites = Vec::new();
        for c in p.sites().iter().flatten() {
            let law = PermanentId::from_raw(c.law())
                .filter(|law| laws.iter().any(|l| l.id == *law))
                .ok_or_else(|| {
                    bad(format!(
                        "polity {id} names a crossing site by a missing law {}",
                        c.law()
                    ))
                })?;
            let Some(system) = rules.catalog.bridge_index(c.system().unwrap_or("")) else {
                continue;
            };
            sites.push((
                law,
                civ_agents::polity::CrossingSite {
                    cell: c.cell(),
                    system: system as u16,
                    span_m: c.span_m(),
                    labour_h: c.labour_h(),
                },
            ));
        }
        // The sources its laws close (schema 82); before, none.
        let mut closed = Vec::new();
        for c in p.closed().iter().flatten() {
            let (Some(law), Some(place @ civ_agents::uses::Place::Source(_))) = (
                PermanentId::from_raw(c.law()),
                civ_agents::uses::Place::from_code(c.kind(), c.place()),
            ) else {
                return Err(bad(format!(
                    "polity {id} closes place {} of kind {} by law {}",
                    c.place(),
                    c.kind(),
                    c.law()
                )));
            };
            if !laws.iter().any(|l| l.id == law) {
                return Err(bad(format!(
                    "polity {id} closes a source by a missing law {law}"
                )));
            }
            closed.push((law, place));
        }
        // Where its laws would dig wells (schema 83); before, nowhere. One of a well system the
        // loaded content lacks is dropped: its law can dig nothing.
        let mut wells = Vec::new();
        for c in p.wells().iter().flatten() {
            let law = PermanentId::from_raw(c.law())
                .filter(|law| laws.iter().any(|l| l.id == *law))
                .ok_or_else(|| {
                    bad(format!(
                        "polity {id} names a well's site by a missing law {}",
                        c.law()
                    ))
                })?;
            let Some(system) = rules.catalog.well_index(c.system().unwrap_or("")) else {
                continue;
            };
            wells.push((
                law,
                civ_agents::polity::WellSite {
                    cell: c.cell(),
                    rect: civ_land::RectCm {
                        x: c.x_cm(),
                        y: c.y_cm(),
                        w: c.w_cm(),
                        h: c.h_cm(),
                    },
                    system: system as u16,
                    target_m: c.target_m(),
                    labour_h: c.labour_h(),
                },
            ));
        }
        // Where its laws would build flood banks, what each household reckoned and gave (schema
        // 102); before, none. A site of a missing law, or a share out of order, refuses the save.
        let law_of = |raw: u64, what: &str| {
            PermanentId::from_raw(raw)
                .filter(|law| laws.iter().any(|l| l.id == *law))
                .ok_or_else(|| bad(format!("polity {id} names {what} by a missing law {raw}")))
        };
        let mut banks = Vec::new();
        for c in p.banks().iter().flatten() {
            let law = law_of(c.law(), "a bank's site")?;
            let site = civ_agents::polity::BankSite {
                reach: c.reach(),
                a: [c.ax(), c.ay()],
                b: [c.bx(), c.by()],
                height_m: c.height_m(),
                crest_width_m: c.crest_width_m(),
                side_run: c.side_run(),
                compacted_per_bank: c.compacted_per_bank(),
                envelope: civ_land::RectCm {
                    x: c.env_x_cm(),
                    y: c.env_y_cm(),
                    w: c.env_w_cm(),
                    h: c.env_h_cm(),
                },
                pit: civ_land::RectCm {
                    x: c.pit_x_cm(),
                    y: c.pit_y_cm(),
                    w: c.pit_w_cm(),
                    h: c.pit_h_cm(),
                },
                labour_h: c.labour_h(),
                upkeep_h: c.upkeep_h(),
                mending: c.mending(),
                work: PermanentId::from_raw(c.work()),
            };
            let finite = [
                site.a[0],
                site.a[1],
                site.b[0],
                site.b[1],
                site.height_m,
                site.crest_width_m,
                site.side_run,
                site.compacted_per_bank,
                site.labour_h,
                site.upkeep_h,
            ]
            .iter()
            .all(|v| v.is_finite());
            if !finite || site.labour_h < 0.0 || site.upkeep_h < 0.0 || site.length_m() <= 0.0 {
                return Err(bad(format!("polity {id} has a bank site of no shape")));
            }
            banks.push((law, site));
        }
        let bank_shares = |list: Option<
            flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<save::BankShareSave<'_>>>,
        >,
                           what: &str|
         -> Result<Vec<(PermanentId, PermanentId, f32)>, LoadError> {
            let mut out: Vec<(PermanentId, PermanentId, f32)> = Vec::new();
            for r in list.iter().flatten() {
                let law = law_of(r.law(), what)?;
                let h = PermanentId::from_raw(r.household())
                    .ok_or_else(|| bad(format!("polity {id} names {what} of no household")))?;
                if !(r.hours().is_finite() && r.hours() >= 0.0)
                    || out.last().is_some_and(|p| (p.0, p.1) >= (law, h))
                {
                    return Err(bad(format!("polity {id} has {what} out of order")));
                }
                out.push((law, h, r.hours()));
            }
            Ok(out)
        };
        let bank_saves = bank_shares(p.bank_saves(), "a bank's reckoned saving")?;
        let bank_given = bank_shares(p.bank_given(), "hours given to a bank")?;
        // The failures of its banks its chronicle has told (schema 103); before, none.
        let mut banks_failed = Vec::new();
        for raw in p.banks_failed().iter().flatten() {
            let w = PermanentId::from_raw(raw)
                .filter(|w| banks.iter().any(|b| b.1.work == Some(*w)))
                .ok_or_else(|| bad(format!("polity {id} tells a failure of no bank of its own")))?;
            banks_failed.push(w);
        }
        // Which bank each law keeping one keeps (schema 104); before, none.
        let mut bank_keeps = Vec::new();
        for k in p.bank_keeps().iter().flatten() {
            let law = law_of(k.law(), "a bank's keeping")?;
            let built = law_of(k.built(), "a kept bank")?;
            if !banks.iter().any(|b| b.0 == built) || bank_keeps.iter().any(|x: &(_, _)| x.0 == law)
            {
                return Err(bad(format!("polity {id} keeps no bank of its own")));
            }
            bank_keeps.push((law, built));
        }
        // Where its laws would raise palisades (schema 114); before, nowhere.
        let mut enclosures = Vec::new();
        for e in p.enclosures().iter().flatten() {
            let law = law_of(e.law(), "a palisade")?;
            let ok = e.x().is_finite()
                && e.y().is_finite()
                && e.radius_m().is_finite()
                && e.radius_m() > 0.0
                && e.gates() >= 1
                && e.labour_h().is_finite()
                && e.labour_h() > 0.0;
            if !ok || enclosures.iter().any(|x: &(PermanentId, _)| x.0 == law) {
                return Err(bad(format!("polity {id} has a palisade's site malformed")));
            }
            enclosures.push((
                law,
                civ_agents::polity::EnclosureSite {
                    centre: (e.x(), e.y()),
                    radius_m: e.radius_m(),
                    gates: e.gates(),
                    inside: e.inside(),
                    labour_h: e.labour_h(),
                    work: PermanentId::from_raw(e.work()),
                },
            ));
        }
        let enclosure_saves = bank_shares(p.enclosure_saves(), "a palisade's reckoned saving")?;
        // Passing places (schema 150, M8b slice CQ, third part).
        let mut ways = Vec::new();
        for w in p.ways().iter().flatten() {
            let law = law_of(w.law(), "a passing place")?;
            let ok = w.labour_h().is_finite() && w.labour_h() > 0.0;
            if !ok || ways.iter().any(|x: &(PermanentId, _)| x.0 == law) {
                return Err(bad(format!(
                    "polity {id} has a passing place's site malformed"
                )));
            }
            ways.push((
                law,
                civ_agents::polity::WaySite {
                    cell: w.cell(),
                    labour_h: w.labour_h(),
                    work: PermanentId::from_raw(w.work()),
                },
            ));
        }
        let way_saves = bank_shares(p.way_saves(), "a passing place's reckoned saving")?;
        let way_given = bank_shares(p.way_given(), "hours given to a passing place")?;
        // Plans (schema 156, M8c slice CT, second part).
        let mut plans = Vec::new();
        for s in p.plans().iter().flatten() {
            let law = law_of(s.law(), "a plan")?;
            let malformed = || bad(format!("polity {id} has a plan malformed"));
            let axis = civ_land::plans::Axis::from_code(s.axis()).ok_or_else(malformed)?;
            let rect = |x: i32, y: i32, w: i32, h: i32| civ_land::RectCm { x, y, w, h };
            let mut sites = Vec::new();
            for t in s.sites().iter().flatten() {
                sites.push(civ_land::plans::Site {
                    rect: rect(t.x(), t.y(), t.w(), t.h()),
                    use_: civ_land::plans::SiteUse::from_code(t.kind()).ok_or_else(malformed)?,
                });
            }
            let grid = civ_land::plans::Grid {
                block_m: (s.block_along(), s.block_across()),
                parcel_m: (s.frontage(), s.depth()),
                street_m: s.street(),
                blocks: (s.blocks_along(), s.blocks_across()),
                square: s.square(),
            };
            let finite = [
                s.origin_x(),
                s.origin_y(),
                grid.block_m.0,
                grid.block_m.1,
                grid.parcel_m.0,
                grid.parcel_m.1,
                grid.street_m,
            ]
            .iter()
            .all(|v| v.is_finite());
            if !finite || s.version() == 0 || plans.iter().any(|x: &(PermanentId, _)| x.0 == law) {
                return Err(malformed());
            }
            plans.push((
                law,
                civ_agents::polity::PlanLaid {
                    version: s.version(),
                    grid,
                    layout: civ_land::plans::Layout {
                        origin: (s.origin_x(), s.origin_y()),
                        axis: Some(axis),
                        streets: s
                            .streets()
                            .iter()
                            .flatten()
                            .map(|r| rect(r.x(), r.y(), r.w(), r.h()))
                            .collect(),
                        sites,
                        parcels: s
                            .parcels()
                            .iter()
                            .flatten()
                            .map(|q| civ_land::plans::Parcel {
                                rect: rect(q.x(), q.y(), q.w(), q.h()),
                                block: q.block(),
                            })
                            .collect(),
                        refused: civ_land::plans::Refused {
                            streets: s.refused_streets(),
                            sites: s.refused_sites(),
                            parcels: s.refused_parcels(),
                        },
                    },
                    staked_h: s.staked_h(),
                    // Schema 159 (M8c slice CU, fifth part); none before.
                    refused_fields: s
                        .refused_fields()
                        .map(|v| v.iter().filter_map(id_of).collect())
                        .unwrap_or_default(),
                },
            ));
        }
        let mut ground_seen = Vec::new();
        for g in p.ground_seen().iter().flatten() {
            let (Some(settlement), Some(household), Some(&issue)) = (
                PermanentId::from_raw(g.settlement()),
                PermanentId::from_raw(g.household()),
                civ_agents::polity::IssueKind::ALL
                    .get(usize::from(g.issue()))
                    .filter(|&&k| issue_code(k) == g.issue()),
            ) else {
                return Err(bad(format!("polity {id} saw its ground malformed")));
            };
            ground_seen.push(civ_agents::polity::GroundSeen {
                day: g.day(),
                settlement,
                household,
                issue,
            });
        }
        let mut encroachments = Vec::new();
        for e in p.encroachments().iter().flatten() {
            let (Some(law), Some(household), Some(building)) = (
                PermanentId::from_raw(e.law()),
                PermanentId::from_raw(e.household()),
                PermanentId::from_raw(e.building()),
            ) else {
                return Err(bad(format!("polity {id} kept an encroachment malformed")));
            };
            encroachments.push(civ_agents::polity::Encroachment {
                day: e.day(),
                law,
                household,
                building,
                seen_by: Vec::new(),
                order: None,
            });
        }
        // Who saw each begun (schema 160); none before.
        for s in p.encroachment_seen().iter().flatten() {
            let (Some(e), Some(person)) = (
                encroachments.get_mut(s.encroachment() as usize),
                PermanentId::from_raw(s.person()),
            ) else {
                return Err(bad(format!("polity {id} kept a witness malformed")));
            };
            e.seen_by.push(person);
        }
        // What a finding ordered for each (schema 161); nothing before.
        for o in p.encroachment_orders().iter().flatten() {
            let (Some(e), Some(order)) = (
                encroachments.get_mut(o.encroachment() as usize),
                civ_agents::polity::PlanOrder::from_code(o.kind(), o.day(), o.kcal()),
            ) else {
                return Err(bad(format!("polity {id} kept an order malformed")));
            };
            e.order = Some(order);
        }
        // The settlements it holds besides its seat, and its end (schema 118); before, its seat
        // alone, still deciding.
        let mut held = Vec::new();
        for h in p.held().iter().flatten() {
            let (Some(settlement), Some(how), Some(from)) = (
                PermanentId::from_raw(h.settlement()),
                civ_agents::polity::HeldHow::from_code(h.how()),
                PermanentId::from_raw(h.from()),
            ) else {
                return Err(bad(format!("polity {id} holds a settlement malformed")));
            };
            held.push(civ_agents::polity::HeldSettlement {
                settlement,
                since: h.since(),
                how,
                from,
            });
        }
        // Its office laws held for a term (schema 120); before, none were.
        let mut terms = Vec::new();
        for t in p.terms().iter().flatten() {
            let Some(law) = PermanentId::from_raw(t.law()) else {
                return Err(bad(format!("polity {id} has a term of no law")));
            };
            terms.push((
                law,
                civ_agents::polity::Term {
                    until: t.until(),
                    pool: t.pool(),
                },
            ));
        }
        if !terms.windows(2).all(|w| w[0].0 < w[1].0) {
            return Err(bad(format!("polity {id}'s terms are out of order")));
        }
        // Its council's seats held and its elections (schema 121); before, none.
        let person = |raw: u64, what: &str| {
            PermanentId::from_raw(raw).ok_or_else(|| bad(format!("polity {id}: {what} is nobody")))
        };
        let mut seated = Vec::new();
        for x in p.seated().iter().flatten() {
            seated.push(civ_agents::seats::Seat {
                seat: x.seat(),
                holder: person(x.holder(), "a seat's holder")?,
                since: x.since(),
                until: x.until(),
                election: x.election(),
            });
        }
        // Measured losses (schema 164, M8d slice DC); before, nothing measured.
        let mut measured: Vec<civ_agents::polity::Measured> = Vec::new();
        for m in p.measured().iter().flatten() {
            let ok = m.spoiled_kcal().is_finite()
                && m.spoiled_kcal() >= 0.0
                && m.wet_gatherings() <= m.gatherings()
                && measured.last().is_none_or(|l| l.year < m.year());
            if !ok {
                return Err(bad(format!(
                    "polity {id} measures a year out of order, or losses out of range"
                )));
            }
            measured.push(civ_agents::polity::Measured {
                year: m.year(),
                spoiled_kcal: m.spoiled_kcal(),
                gatherings: m.gatherings(),
                wet_gatherings: m.wet_gatherings(),
                roof_fires: m.roof_fires(),
                wet_lost_h: 0.0,
                late_fire_h: 0.0,
            });
        }
        // The losses measured beside them (schema 166); before, none.
        if let Some(losses) = p.measured_losses() {
            if losses.len() != measured.len() {
                return Err(bad(format!(
                    "polity {id} measures its losses for {} years of {}",
                    losses.len(),
                    measured.len()
                )));
            }
            for (m, l) in measured.iter_mut().zip(losses.iter()) {
                let ok = l.year() == m.year
                    && l.wet_lost_h().is_finite()
                    && l.wet_lost_h() >= 0.0
                    && l.late_fire_h().is_finite()
                    && l.late_fire_h() >= 0.0;
                if !ok {
                    return Err(bad(format!(
                        "polity {id} measures its losses out of order, or out of range"
                    )));
                }
                m.wet_lost_h = l.wet_lost_h();
                m.late_fire_h = l.late_fire_h();
            }
        }
        if measured.len() > civ_agents::polity::MEASURED_YEARS {
            return Err(bad(format!(
                "polity {id} measures more years than it keeps"
            )));
        }
        // Buildings raised in common (schema 165, M8d slice DC); before, none. Their dates from
        // schema 168, -1 for none.
        let when = |m: i64| (m >= 0).then(|| SimTime::from_minutes(m));
        let mut raises = Vec::new();
        for r in p.raises().iter().flatten() {
            let law = law_of(r.law(), "a building raised in common")?;
            // The program is the law's template's, as the loaded content numbers its buildings:
            // the index saved is that of the content it was saved with.
            let program = laws
                .iter()
                .find(|l| l.id == law)
                .and_then(|l| rules.catalog.policies.get(usize::from(l.policy)))
                .and_then(|d| d.raise)
                .and_then(|r| u16::try_from(r.program).ok());
            let ok = r.labour_h().is_finite() && r.labour_h() > 0.0;
            let (true, Some(program)) = (ok, program) else {
                return Err(bad(format!(
                    "polity {id} has a building raised in common malformed"
                )));
            };
            if raises.iter().any(|x: &(PermanentId, _)| x.0 == law) {
                return Err(bad(format!(
                    "polity {id} has a building raised in common twice"
                )));
            }
            raises.push((
                law,
                civ_agents::polity::RaiseSite {
                    program,
                    labour_h: r.labour_h(),
                    building: PermanentId::from_raw(r.building()),
                    begun: when(r.begun()),
                    last_work: when(r.last_work()),
                    first_use: when(r.first_use()),
                    finished: when(r.finished()),
                    stalled: when(r.stalled()),
                },
            ));
        }
        let raise_given = bank_shares(p.raise_given(), "hours given to a building raised")?;
        let storehouse = PermanentId::from_raw(p.storehouse());
        let hall = PermanentId::from_raw(p.hall());
        let tower = PermanentId::from_raw(p.tower());
        // Patron gifts (schema 131, M7c slice BY).
        let mut patron_gifts = Vec::new();
        for g in p.patron_gifts().iter().flatten() {
            patron_gifts.push(civ_agents::polity::PatronGift {
                patron: required(g.patron(), "a patron gift's patron")?,
                giver: required(g.giver(), "a patron gift's giver")?,
                client: required(g.client(), "a patron gift's client")?,
                law: required(g.law(), "a patron gift's law")?,
                day: g.day(),
                kcal: g.kcal(),
                days: g.days(),
            });
        }
        let mut elections = Vec::new();
        for e in p.elections().iter().flatten() {
            let what = format!("polity {id}'s election {}", e.id());
            let seats = Seats::unpack(e.seats())
                .map_err(|m| bad(format!("{what}: {m}")))?
                .ok_or_else(|| bad(format!("{what} fills no seats")))?;
            let why = civ_agents::seats::ElectionWhy::from_code(e.why())
                .ok_or_else(|| bad(format!("{what} has reason code {}", e.why())))?;
            let stage = civ_agents::seats::ElectionStage::from_code(e.stage())
                .ok_or_else(|| bad(format!("{what} has stage code {}", e.stage())))?;
            let ids =
                |v: Option<flatbuffers::Vector<'_, u64>>| -> Result<Vec<PermanentId>, LoadError> {
                    v.iter()
                        .flatten()
                        .map(|raw| person(raw, "one in an election"))
                        .collect()
                };
            let mut candidates = Vec::new();
            for c in e.candidates().iter().flatten() {
                candidates.push(civ_agents::seats::Candidate {
                    person: person(c.person(), "a candidate")?,
                    day: c.day(),
                    positions: Vec::new(),
                    district: None,
                });
            }
            // Each candidate's district, and each district's seats (schema 123); before, none.
            if let Some(ds) = e.candidate_districts() {
                if ds.len() != candidates.len() {
                    return Err(bad(format!("{what}: districts for some who stood only")));
                }
                for (c, d) in candidates.iter_mut().zip(ds.iter()) {
                    c.district = PermanentId::from_raw(d);
                }
            }
            let districts: Vec<(PermanentId, u8)> = e
                .districts()
                .iter()
                .flatten()
                .map(|d| {
                    PermanentId::from_raw(d.settlement())
                        .map(|s| (s, d.seats()))
                        .ok_or_else(|| bad(format!("{what}: a district of nowhere")))
                })
                .collect::<Result<_, _>>()?;
            // Its alliances (schema 124); before, none.
            let mut alliances = Vec::new();
            for a in e.alliances().iter().flatten() {
                let (Some(first), Some(second), Some(package)) = (
                    PermanentId::from_raw(a.first()),
                    PermanentId::from_raw(a.second()),
                    civ_agents::seats::Package::from_code(a.package()),
                ) else {
                    return Err(bad(format!("{what}: an alliance is malformed")));
                };
                alliances.push(civ_agents::seats::Alliance {
                    factions: [first, second],
                    package,
                });
            }
            // Where each said they stand, and the factions' slates (schema 122); before, none.
            for x in e.positions().iter().flatten() {
                let Some(c) = candidates
                    .iter_mut()
                    .find(|c| c.person.get() == x.candidate())
                else {
                    return Err(bad(format!("{what}: a position of one who did not stand")));
                };
                c.positions.push((x.policy(), x.x()));
            }
            let mut endorsements = Vec::new();
            for s in e.endorsements().iter().flatten() {
                let faction = PermanentId::from_raw(s.faction())
                    .ok_or_else(|| bad(format!("{what}: a slate of no faction")))?;
                endorsements.push((faction, ids(s.named())?));
            }
            let mut ballots = Vec::new();
            for b in e.ballots().iter().flatten() {
                ballots.push((person(b.voter(), "a voter")?, ids(b.named())?));
            }
            let mut totals = Vec::new();
            for t in e.totals().iter().flatten() {
                totals.push((person(t.person(), "one counted")?, t.votes()));
            }
            elections.push(civ_agents::seats::Election {
                id: e.id(),
                why,
                seats,
                called: e.called(),
                stand_until: e.stand_until(),
                poll_day: e.poll_day(),
                stage,
                considered: ids(e.considered())?,
                candidates,
                ballots,
                totals,
                winners: ids(e.winners())?,
                by_lot: e.by_lot(),
                ended: (e.ended() >= 0).then_some(e.ended()),
                until: e.until(),
                endorsements,
                districts,
                alliances,
            });
        }
        let ended = match p.ended_how() {
            255 => None,
            code => {
                let (Some(how), Some(by)) = (
                    civ_agents::polity::HeldHow::from_code(code),
                    PermanentId::from_raw(p.ended_by()),
                ) else {
                    return Err(bad(format!("polity {id}'s end is malformed")));
                };
                Some(civ_agents::polity::PolityEnd {
                    day: p.ended_day(),
                    by,
                    how,
                })
            }
        };
        out.push(Polity {
            id,
            settlement,
            founded: time(p.founded()),
            body,
            versions,
            stores,
            stores_at: time(p.stores_at()),
            flows: Default::default(),
            laws,
            gathering,
            reviewed: p.reviewed(),
            claimed,
            sites,
            closed,
            wells,
            banks,
            bank_saves,
            bank_given,
            banks_failed,
            bank_keeps,
            enclosures,
            enclosure_saves,
            held,
            ended,
            terms,
            seated,
            elections,
            patron_gifts,
            ways,
            way_saves,
            way_given,
            plans,
            ground_seen,
            encroachments,
            measured,
            raises,
            raise_given,
            storehouse,
            hall,
            tower,
        });
    }
    Ok(out)
}

// ---- earth -------------------------------------------------------------------------------------

fn encode_earth(land: &Land) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let works: Vec<save::EarthworkRecord> = land
        .earthworks
        .iter()
        .map(|w| {
            save::EarthworkRecord::new(
                w.id.get(),
                w.household.get(),
                raw(w.plot),
                w.begun.minutes(),
                w.cut_m3,
                w.done,
                w.level_cm,
                w.rect.x,
                w.rect.y,
                w.rect.w,
                w.rect.h,
                w.side_run_cm,
                w.version,
                match w.kind {
                    civ_land::earth::EarthKind::Platform => 0,
                    civ_land::earth::EarthKind::Pit => 1,
                    civ_land::earth::EarthKind::Spoil => 2,
                    civ_land::earth::EarthKind::Bank => 3,
                    civ_land::earth::EarthKind::Ditch => 4,
                },
            )
        })
        .collect();
    let works = fbb.create_vector(&works);
    let tiles: Vec<_> = land
        .ground
        .tiles()
        .map(|(index, t)| {
            let cells = fbb.create_vector(&t.cells);
            save::GroundTile::create(
                &mut fbb,
                &save::GroundTileArgs {
                    index,
                    rev: t.rev,
                    cells: Some(cells),
                },
            )
        })
        .collect();
    let tiles = fbb.create_vector(&tiles);
    let links: Vec<save::EarthLink> = land
        .earthworks
        .iter()
        .filter(|w| w.deposit.is_some() || w.heap.is_some())
        .map(|w| save::EarthLink::new(w.id.get(), raw(w.deposit), raw(w.heap)))
        .collect();
    let links = fbb.create_vector(&links);
    let banks: Vec<_> = land
        .banks
        .iter()
        .map(|b| {
            save::BankSegmentRecord::new(
                b.work.get(),
                b.borrow.get(),
                b.a[0],
                b.a[1],
                b.b[0],
                b.b[1],
                b.height_m,
                b.crest_width_m,
                b.side_run,
                b.compacted_per_bank,
                b.settlement_m,
                b.erosion_m,
                b.breached,
            )
        })
        .collect();
    let banks = fbb.create_vector(&banks);
    let sediment: Vec<_> = land
        .banks
        .iter()
        .filter(|b| b.sediment_m3 != 0.0)
        .map(|b| save::BankSedimentRecord::new(b.work.get(), b.sediment_m3))
        .collect();
    let sediment = fbb.create_vector(&sediment);
    let piping: Vec<_> = land
        .banks
        .iter()
        .filter(|b| b.piping != 0.0)
        .map(|b| save::BankPipingRecord::new(b.work.get(), b.piping))
        .collect();
    let piping = fbb.create_vector(&piping);
    let refill: Vec<_> = land
        .banks
        .iter()
        .filter(|b| b.refill_m3 != 0.0)
        .map(|b| save::BankRefillRecord::new(b.work.get(), b.refill_m3))
        .collect();
    let refill = fbb.create_vector(&refill);
    let ditches: Vec<_> = land
        .ditches
        .iter()
        .map(|d| {
            save::DitchRecord::new(
                d.work.get(),
                d.reach,
                d.field.get(),
                d.a[0],
                d.a[1],
                d.b[0],
                d.b[1],
                d.bottom_m,
                d.depth_m,
                d.side_run,
                d.drop_m,
                d.weeds,
                d.silt_m,
                d.open,
                d.taken_m3,
                d.kept_m3,
                d.seeped_m3,
                d.returned_m3,
            )
        })
        .collect();
    let ditches = fbb.create_vector(&ditches);
    let ditch_flows: Vec<_> = land
        .ditch_flows
        .iter()
        .map(|&(r, q)| save::DitchFlowRecord::new(r, q))
        .collect();
    let ditch_flows = fbb.create_vector(&ditch_flows);
    // Water-power installations (schema 173), absent while there are none.
    let installations: Vec<_> = land
        .installations
        .iter()
        .map(|x| {
            save::InstallationRecord::new(
                x.id.get(),
                x.reach,
                x.at[0],
                x.at[1],
                x.head_m,
                x.race_m3s,
                x.efficiency,
                x.rated_kw,
                x.owner.get(),
                x.building.map_or(0, PermanentId::get),
            )
        })
        .collect();
    let installations = (!installations.is_empty()).then(|| fbb.create_vector(&installations));
    // Water standing in pits (schema 175), absent while no pit has taken any in.
    let pit_water: Vec<_> = land
        .pit_water
        .iter()
        .map(|p| save::PitWaterRecord::new(p.work.get(), p.water_m3, p.at.minutes()))
        .collect();
    let pit_water = (!pit_water.is_empty()).then(|| fbb.create_vector(&pit_water));
    // Engines set at pits (schema 176), absent while there are none.
    let engines: Vec<_> = land
        .engines
        .iter()
        .map(|e| {
            save::EngineRecord::new(
                e.id.get(),
                e.pit.get(),
                e.owner.get(),
                e.rated_kw,
                e.efficiency,
            )
        })
        .collect();
    let engines = (!engines.is_empty()).then(|| fbb.create_vector(&engines));
    // Hours bailed at pits (schema 177), absent while none.
    let pit_bailing: Vec<_> = land
        .pit_bailing
        .iter()
        .map(|p| save::PitBailingRecord::new(p.work.get(), p.hours, p.since.minutes()))
        .collect();
    let pit_bailing = (!pit_bailing.is_empty()).then(|| fbb.create_vector(&pit_bailing));
    let root = save::Earth::create(
        &mut fbb,
        &save::EarthArgs {
            works: Some(works),
            tiles: Some(tiles),
            links: Some(links),
            banks: Some(banks),
            bank_sediment: Some(sediment),
            bank_piping: Some(piping),
            bank_refill: Some(refill),
            ditches: Some(ditches),
            ditch_flows: Some(ditch_flows),
            installations,
            pit_water,
            engines,
            pit_bailing,
        },
    );
    finish(fbb, root)
}

fn decode_earth(bytes: &[u8], land: &mut Land, schema: Schema) -> Result<(), LoadError> {
    let root = section_root::<save::Earth>(bytes).map_err(|e| unreadable(SECTION_EARTH, &e))?;
    for w in root.works().iter().flatten() {
        let kind = match w.kind() {
            0 => civ_land::earth::EarthKind::Platform,
            1 => civ_land::earth::EarthKind::Pit,
            2 => civ_land::earth::EarthKind::Spoil,
            3 if schema >= Schema::V100 => civ_land::earth::EarthKind::Bank,
            4 if schema >= Schema::V105 => civ_land::earth::EarthKind::Ditch,
            other => {
                return Err(LoadError::Malformed(format!(
                    "an earthwork has kind {other}"
                )));
            }
        };
        land.earthworks.push(civ_land::earth::Earthwork {
            id: required(w.id(), "an earthwork")?,
            kind,
            rect: civ_land::RectCm {
                x: w.x_cm(),
                y: w.y_cm(),
                w: w.w_cm(),
                h: w.h_cm(),
            },
            level_cm: w.level_cm(),
            side_run_cm: w.side_run_cm(),
            plot: id(w.plot()),
            deposit: None,
            heap: None,
            household: required(w.household(), "an earthwork")?,
            cut_m3: w.cut_m3(),
            done: w.done(),
            version: w.version(),
            begun: time(w.begun()),
        });
    }
    if schema < Schema::V100 && root.banks().is_some_and(|b| !b.is_empty()) {
        return Err(LoadError::Malformed(
            "linear banks in a save before schema 100".into(),
        ));
    }
    // Explicit migration: older snapshots have no linear banks; none are inferred.
    if schema >= Schema::V100 {
        for b in root.banks().iter().flatten() {
            land.banks.push(civ_land::banks::BankSegment {
                work: required(b.work(), "a bank segment")?,
                borrow: required(b.borrow(), "a bank borrow source")?,
                a: [b.ax(), b.ay()],
                b: [b.bx(), b.by()],
                height_m: b.height_m(),
                crest_width_m: b.crest_width_m(),
                side_run: b.side_run(),
                compacted_per_bank: b.compacted_per_bank(),
                settlement_m: b.settlement_m(),
                erosion_m: b.erosion_m(),
                breached: b.breached(),
                sediment_m3: 0.0,
                piping: 0.0,
                refill_m3: 0.0,
            });
        }
    }
    // Explicit migration: schemas through 100 retain their ground and have no washed-soil stock.
    if schema < Schema::V101 && root.bank_sediment().is_some_and(|v| !v.is_empty()) {
        return Err(LoadError::Malformed(
            "bank sediment before schema 101".into(),
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for record in root.bank_sediment().iter().flatten() {
        let bank = land
            .banks
            .iter_mut()
            .find(|b| b.work.get() == record.work())
            .ok_or_else(|| LoadError::Malformed("sediment names missing bank".into()))?;
        if !seen.insert(record.work())
            || !record.compacted_m3().is_finite()
            || record.compacted_m3() <= 0.0
        {
            return Err(LoadError::Malformed(
                "invalid or duplicate bank sediment".into(),
            ));
        }
        bank.sediment_m3 = record.compacted_m3();
    }
    // Explicit migration: schemas through 102 have no seepage damage.
    if schema < Schema::V103 && root.bank_piping().is_some_and(|v| !v.is_empty()) {
        return Err(LoadError::Malformed("bank piping before schema 103".into()));
    }
    let mut seen = std::collections::HashSet::new();
    for record in root.bank_piping().iter().flatten() {
        let bank = land
            .banks
            .iter_mut()
            .find(|b| b.work.get() == record.work())
            .ok_or_else(|| LoadError::Malformed("piping names missing bank".into()))?;
        if !seen.insert(record.work())
            || !record.damage().is_finite()
            || !(record.damage() > 0.0 && record.damage() <= 1.0)
        {
            return Err(LoadError::Malformed(
                "invalid or duplicate bank piping".into(),
            ));
        }
        bank.piping = record.damage();
    }
    // Explicit migration: schemas through 103 have mended no bank.
    if schema < Schema::V104 && root.bank_refill().is_some_and(|v| !v.is_empty()) {
        return Err(LoadError::Malformed("bank refill before schema 104".into()));
    }
    let mut seen = std::collections::HashSet::new();
    for record in root.bank_refill().iter().flatten() {
        let bank = land
            .banks
            .iter_mut()
            .find(|b| b.work.get() == record.work())
            .ok_or_else(|| LoadError::Malformed("refill names missing bank".into()))?;
        if !seen.insert(record.work())
            || !record.compacted_m3().is_finite()
            || record.compacted_m3() <= 0.0
        {
            return Err(LoadError::Malformed(
                "invalid or duplicate bank refill".into(),
            ));
        }
        bank.refill_m3 = record.compacted_m3();
    }
    // Explicit migration: schemas through 104 have no irrigation ditches.
    if schema < Schema::V105
        && (root.ditches().is_some_and(|v| !v.is_empty())
            || root.ditch_flows().is_some_and(|v| !v.is_empty()))
    {
        return Err(LoadError::Malformed("ditches before schema 105".into()));
    }
    for d in root.ditches().iter().flatten() {
        land.ditches.push(civ_land::ditches::DitchSegment {
            work: required(d.work(), "a ditch")?,
            reach: d.reach(),
            field: required(d.field(), "a ditch's field")?,
            a: [d.ax(), d.ay()],
            b: [d.bx(), d.by()],
            bottom_m: d.bottom_m(),
            depth_m: d.depth_m(),
            side_run: d.side_run(),
            drop_m: d.drop_m(),
            weeds: d.weeds(),
            silt_m: d.silt_m(),
            open: d.open(),
            taken_m3: d.taken_m3(),
            kept_m3: d.kept_m3(),
            seeped_m3: d.seeped_m3(),
            returned_m3: d.returned_m3(),
        });
    }
    land.ditch_flows = root
        .ditch_flows()
        .iter()
        .flatten()
        .map(|f| (f.reach(), f.m3s()))
        .collect();
    // Water-power installations (schema 173); before, none.
    for x in root.installations().iter().flatten() {
        let record = civ_land::power::Installation {
            id: required(x.id(), "an installation")?,
            reach: x.reach(),
            at: [x.at_x(), x.at_y()],
            head_m: x.head_m(),
            race_m3s: x.race_m3s(),
            efficiency: x.efficiency(),
            rated_kw: x.rated_kw(),
            owner: required(x.owner(), "an installation's owner")?,
            building: PermanentId::from_raw(x.building()),
        };
        if !record.valid() {
            return Err(LoadError::Malformed(format!(
                "installation {} is not whole",
                record.id
            )));
        }
        land.installations.push(record);
    }
    // Engines set at pits (schema 176); before, none.
    for e in root.engines().iter().flatten() {
        let record = civ_land::power::Engine {
            id: required(e.id(), "an engine")?,
            pit: required(e.pit(), "an engine's pit")?,
            owner: required(e.owner(), "an engine's owner")?,
            rated_kw: e.rated_kw(),
            efficiency: e.efficiency(),
        };
        if !record.valid() {
            return Err(LoadError::Malformed(format!(
                "engine {} is not whole",
                record.id
            )));
        }
        land.engines.push(record);
    }
    // Hours bailed at pits (schema 177); before, none.
    for p in root.pit_bailing().iter().flatten() {
        let work = required(p.work(), "a pit's bailing")?;
        if !(p.hours().is_finite() && p.hours() >= 0.0) {
            return Err(LoadError::Malformed(format!(
                "pit {work} tallies invalid hours of bailing"
            )));
        }
        land.pit_bailing.push(civ_land::pits::Bailed {
            work,
            hours: p.hours(),
            since: SimTime::from_minutes(p.since()),
        });
    }
    // Water standing in pits (schema 175); before, none.
    for p in root.pit_water().iter().flatten() {
        let work = required(p.work(), "a pit's water")?;
        if !(p.water_m3().is_finite() && p.water_m3() >= 0.0) {
            return Err(LoadError::Malformed(format!(
                "pit {work} holds an invalid amount of water"
            )));
        }
        land.pit_water.push(civ_land::pits::PitWater {
            work,
            water_m3: p.water_m3(),
            at: SimTime::from_minutes(p.at()),
        });
    }
    // Schema 21: what each pit and heap is linked to.
    for l in root.links().iter().flatten() {
        let work = land
            .earthworks
            .iter_mut()
            .find(|w| w.id.get() == l.work())
            .ok_or_else(|| {
                LoadError::Malformed(format!("a link names earthwork {}, not saved", l.work()))
            })?;
        work.deposit = id(l.deposit());
        work.heap = id(l.heap());
    }
    for t in root.tiles().iter().flatten() {
        let tile = civ_land::earth::DeltaTile {
            rev: t.rev(),
            cells: t.cells().map(|c| c.iter().collect()).unwrap_or_default(),
        };
        if !land.ground.set_tile(t.index(), tile) {
            return Err(LoadError::Malformed(format!(
                "ground tile {} is off the map or the wrong size",
                t.index()
            )));
        }
    }
    Ok(())
}

// ---- deposits ----------------------------------------------------------------------------------

fn encode_ties(pop: &Population) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let mut records = Vec::with_capacity(pop.ties.len());
    for holder in pop.ties.holders() {
        for t in pop.ties.of(holder) {
            let (reason, reason_day, times) = t
                .reason
                .map_or((u8::MAX, 0, 0), |r| (r.act.code(), r.day, r.times));
            records.push(save::TieRecord::new(
                holder.get(),
                t.to.get(),
                t.day,
                reason_day,
                t.familiarity,
                t.warmth,
                t.good[0],
                t.good[1],
                t.good[2],
                t.good[3],
                t.bad[0],
                t.bad[1],
                t.bad[2],
                t.bad[3],
                t.fear,
                t.help_h,
                times,
                reason,
            ));
        }
    }
    let ties = fbb.create_vector(&records);
    let standing: Vec<save::StandingRecord> = pop
        .standing
        .rows
        .iter()
        .map(|r| {
            save::StandingRecord::new(
                r.person.get(),
                r.settlement.get(),
                r.esteem[0],
                r.esteem[1],
                r.esteem[2],
                r.esteem[3],
                r.influence,
                r.notable,
            )
        })
        .collect();
    let standing = fbb.create_vector(&standing);
    let root = save::Ties::create(
        &mut fbb,
        &save::TiesArgs {
            ties: Some(ties),
            standing: Some(standing),
            standing_day: pop.standing.day,
        },
    );
    finish(fbb, root)
}

/// The ties of a save, and its standing if it kept one.
fn decode_ties(
    bytes: &[u8],
) -> Result<
    (
        civ_agents::ties::Ties,
        Option<civ_agents::standing::StandingTable>,
    ),
    LoadError,
> {
    use civ_agents::ties::{Act, Reason, Tie, Ties};
    let root = section_root::<save::Ties>(bytes).map_err(|e| unreadable(SECTION_TIES, &e))?;
    let mut by_holder: std::collections::BTreeMap<PermanentId, Vec<Tie>> = Default::default();
    for r in root.ties().iter().flatten() {
        let (Some(holder), Some(to)) = (
            PermanentId::from_raw(r.holder()),
            PermanentId::from_raw(r.to()),
        ) else {
            return Err(LoadError::Malformed("a tie names nobody".to_owned()));
        };
        let reason = match r.reason() {
            u8::MAX => None,
            code => Some(Reason {
                act: Act::from_code(code).ok_or_else(|| {
                    LoadError::Malformed(format!("a tie's reason is act {code}, which is unknown"))
                })?,
                day: r.reason_day(),
                times: r.reason_times(),
            }),
        };
        by_holder.entry(holder).or_default().push(Tie {
            to,
            day: r.day(),
            familiarity: r.familiarity(),
            warmth: r.warmth(),
            good: [
                r.good_provision(),
                r.good_craft(),
                r.good_word(),
                r.good_counsel(),
            ],
            bad: [
                r.bad_provision(),
                r.bad_craft(),
                r.bad_word(),
                r.bad_counsel(),
            ],
            fear: r.fear(),
            help_h: r.help_h(),
            reason,
        });
    }
    let mut ties = Ties::new();
    for (holder, held) in by_holder {
        ties.restore(holder, held);
    }
    let standing = match root.standing() {
        None => None,
        Some(rows) => {
            let mut table = civ_agents::standing::StandingTable {
                day: root.standing_day(),
                rows: Vec::with_capacity(rows.len()),
            };
            for r in rows.iter() {
                let (Some(person), Some(settlement)) = (
                    PermanentId::from_raw(r.person()),
                    PermanentId::from_raw(r.settlement()),
                ) else {
                    return Err(LoadError::Malformed("a standing names nobody".to_owned()));
                };
                table.rows.push(civ_agents::standing::Standing {
                    person,
                    settlement,
                    esteem: [
                        r.esteem_provision(),
                        r.esteem_craft(),
                        r.esteem_word(),
                        r.esteem_counsel(),
                    ],
                    influence: r.influence(),
                    notable: r.notable(),
                });
            }
            table.rows.sort_by_key(|r| (r.settlement, r.person));
            Some(table)
        }
    };
    Ok((ties, standing))
}

fn encode_deposits(land: &Land, pop: &Population, goods: &[&str]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let dictionary = strings(&mut fbb, goods);
    let bodies: Vec<save::DepositBody> = land
        .deposits
        .iter()
        .map(|d| {
            let b = &d.body;
            save::DepositBody::new(
                b.at_cm.0,
                b.at_cm.1,
                d.id.get(),
                b.initial_kg,
                d.taken_kg,
                b.radius_cm,
                b.top_cm,
                b.thickness_cm,
                b.quality,
                b.good,
                b.exposed,
            )
        })
        .collect();
    let bodies = fbb.create_vector(&bodies);
    let known: Vec<save::DepositKnownEntry> = pop
        .deposits_known
        .iter()
        .map(|k| {
            save::DepositKnownEntry::new(
                k.at.minutes(),
                k.settlement.get(),
                k.deposit.get(),
                k.finder.get(),
            )
        })
        .collect();
    let known = fbb.create_vector(&known);
    let root = save::Deposits::create(
        &mut fbb,
        &save::DepositsArgs {
            goods: Some(dictionary),
            bodies: Some(bodies),
            known: Some(known),
        },
    );
    finish(fbb, root)
}

/// The bodies and what each settlement knows of them. Knowledge of a body that was dropped goes
/// with it.
fn decode_deposits(
    bytes: &[u8],
    rules: &Rules,
) -> Result<(Vec<Deposit>, Vec<DepositKnown>), LoadError> {
    let root =
        section_root::<save::Deposits>(bytes).map_err(|e| unreadable(SECTION_DEPOSITS, &e))?;
    let goods: Vec<Option<usize>> = read_strings(root.goods())
        .iter()
        .map(|id| rules.catalog.good_index(id))
        .collect();
    let mut out = Vec::new();
    for b in root.bodies().iter().flatten() {
        let good = goods.get(usize::from(b.good())).ok_or_else(|| {
            LoadError::Malformed(format!(
                "a deposit names good {} of {}",
                b.good(),
                goods.len()
            ))
        })?;
        // A good the loaded content no longer has: the body is dropped.
        let Some(good) = good else {
            continue;
        };
        out.push(Deposit {
            id: required(b.id(), "a deposit")?,
            body: Body {
                good: *good as u16,
                at_cm: (b.x_cm(), b.y_cm()),
                radius_cm: b.radius_cm(),
                top_cm: b.top_cm(),
                thickness_cm: b.thickness_cm(),
                quality: b.quality(),
                exposed: b.exposed(),
                initial_kg: b.initial_kg(),
            },
            taken_kg: b.taken_kg(),
        });
    }
    let mut known = Vec::new();
    for k in root.known().iter().flatten() {
        let deposit = required(k.deposit(), "a known deposit")?;
        if !out.iter().any(|d| d.id == deposit) {
            continue;
        }
        known.push(DepositKnown {
            settlement: required(k.settlement(), "a known deposit")?,
            deposit,
            finder: required(k.finder(), "a known deposit")?,
            at: time(k.at()),
        });
    }
    Ok((out, known))
}

// ---- Helpers -----------------------------------------------------------------------------------

fn id(raw: u64) -> Option<PermanentId> {
    PermanentId::from_raw(raw)
}

fn raw(id: Option<PermanentId>) -> u64 {
    id.map_or(0, PermanentId::get)
}

fn required(raw: u64, what: &str) -> Result<PermanentId, LoadError> {
    PermanentId::from_raw(raw).ok_or_else(|| LoadError::Malformed(format!("{what} has id 0")))
}

fn point((x, y): (f32, f32)) -> save::Point {
    save::Point::new(x, y)
}

fn xy(p: Option<&save::Point>) -> (f32, f32) {
    p.map_or((0.0, 0.0), |p| (p.x(), p.y()))
}

fn time(minutes: i64) -> SimTime {
    SimTime::from_minutes(minutes)
}

type StringVector<'a> = flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<&'a str>>;

fn strings<'a>(fbb: &mut FlatBufferBuilder<'a>, items: &[&str]) -> WIPOffset<StringVector<'a>> {
    let offsets: Vec<_> = items.iter().map(|s| fbb.create_string(s)).collect();
    fbb.create_vector(&offsets)
}

fn read_strings(v: Option<StringVector<'_>>) -> Vec<String> {
    v.map(|v| v.iter().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// Maps a saved activity dictionary to the loaded catalog.
fn activity_map(saved: &[String], rules: &Rules) -> Vec<Option<u16>> {
    saved
        .iter()
        .map(|id| rules.catalog.index_of(id).map(|i| i as u16))
        .collect()
}

/// Maps a saved good dictionary to the loaded catalog.
fn good_map(saved: &[String], rules: &Rules) -> Vec<Option<usize>> {
    saved
        .iter()
        .map(|id| rules.catalog.good_index(id))
        .collect()
}

/// Good `index` of a saved dictionary mapped by `map` (see `good_map`): its index in the loaded
/// catalog, or `None` when the content no longer has it. `what` names the record that refers to
/// it, for the error when the index is past the dictionary.
fn saved_good(
    map: &[Option<usize>],
    index: u32,
    what: impl Fn() -> String,
) -> Result<Option<u16>, LoadError> {
    match map.get(index as usize) {
        Some(g) => Ok(g.and_then(|g| u16::try_from(g).ok())),
        None => Err(LoadError::Malformed(format!(
            "{} names good {index} of {}",
            what(),
            map.len()
        ))),
    }
}

/// Kilograms of the provisions good holding `kcal` (schema-2 migration), or `None` when the
/// loaded content's provisions good has no food energy.
fn provisions_kg(rules: &Rules, kcal: f64) -> Option<(usize, f64)> {
    let g = rules.people.band.provisions_good;
    let good = rules.catalog.goods.get(g)?;
    (good.kcal_per_kg > 0.0).then(|| (g, kcal.max(0.0) / good.kcal_per_kg))
}

/// A target as its saved kind, index and id.
fn target_parts(t: Target) -> (save::TargetKind, u32, u64) {
    match t {
        Target::None => (save::TargetKind::None, 0, 0),
        Target::Home => (save::TargetKind::Home, 0, 0),
        Target::Hearth(s) => (save::TargetKind::Hearth, 0, s.get()),
        Target::Patch(p) => (save::TargetKind::Patch, p, 0),
        Target::Water(c) => (save::TargetKind::Water, c, 0),
        Target::Field(f) => (save::TargetKind::Field, 0, f.get()),
        Target::NewField => (save::TargetKind::NewField, 0, 0),
        Target::Household(h) => (save::TargetKind::Household, 0, h.get()),
        Target::Building(b) => (save::TargetKind::Building, 0, b.get()),
        Target::NewBuilding => (save::TargetKind::NewBuilding, 0, 0),
        Target::Firm(f) => (save::TargetKind::Firm, 0, f.get()),
        Target::NewFirm => (save::TargetKind::NewFirm, 0, 0),
        Target::Technique(t) => (save::TargetKind::Technique, u32::from(t), 0),
        Target::Deposit(d) => (save::TargetKind::Deposit, 0, d.get()),
        Target::Crossing(c) => (save::TargetKind::Crossing, 0, c.get()),
        Target::Well(w) => (save::TargetKind::Well, 0, w.get()),
        Target::Douse { building, from } => (save::TargetKind::Douse, from, building.get()),
        Target::Bank(b) => (save::TargetKind::Bank, 0, b.get()),
        Target::Ditch(d) => (save::TargetKind::Ditch, 0, d.get()),
        Target::Enclosure(e) => (save::TargetKind::Enclosure, 0, e.get()),
        Target::Boat(b) => (save::TargetKind::Boat, 0, b.get()),
        Target::Way(w) => (save::TargetKind::Way, 0, w.get()),
        Target::Mill(m) => (save::TargetKind::Mill, 0, m.get()),
    }
}

/// A target from its saved kind, index and id. `hearth` is the settlement whose hearth a target
/// saved before schema 50 meant (the person's own: the hearth named no settlement then); a
/// hearth target with neither is malformed.
fn target_of(
    kind: save::TargetKind,
    index: u32,
    id: u64,
    hearth: Option<PermanentId>,
) -> Result<Target, LoadError> {
    Ok(match kind {
        save::TargetKind::None => Target::None,
        save::TargetKind::Home => Target::Home,
        save::TargetKind::Hearth => Target::Hearth(
            PermanentId::from_raw(id)
                .or(hearth)
                .ok_or_else(|| LoadError::Malformed("a hearth of no settlement".to_owned()))?,
        ),
        save::TargetKind::Patch => Target::Patch(index),
        save::TargetKind::Water => Target::Water(index),
        save::TargetKind::Field => Target::Field(required(id, "a field target")?),
        save::TargetKind::NewField => Target::NewField,
        save::TargetKind::Household => Target::Household(required(id, "a household target")?),
        save::TargetKind::Building => Target::Building(required(id, "a building target")?),
        save::TargetKind::NewBuilding => Target::NewBuilding,
        save::TargetKind::Firm => Target::Firm(required(id, "a firm target")?),
        save::TargetKind::NewFirm => Target::NewFirm,
        save::TargetKind::Technique => Target::Technique(
            u16::try_from(index)
                .map_err(|_| LoadError::Malformed(format!("a target names technique {index}")))?,
        ),
        save::TargetKind::Deposit => Target::Deposit(required(id, "a deposit target")?),
        save::TargetKind::Crossing => Target::Crossing(required(id, "a crossing target")?),
        save::TargetKind::Well => Target::Well(required(id, "a well target")?),
        save::TargetKind::Douse => Target::Douse {
            building: required(id, "a roof to throw water on")?,
            from: index,
        },
        save::TargetKind::Bank => Target::Bank(required(id, "a bank target")?),
        save::TargetKind::Ditch => Target::Ditch(required(id, "a ditch target")?),
        save::TargetKind::Enclosure => Target::Enclosure(required(id, "a palisade target")?),
        save::TargetKind::Boat => Target::Boat(required(id, "a boat target")?),
        save::TargetKind::Way => Target::Way(required(id, "a way target")?),
        save::TargetKind::Mill => Target::Mill(required(id, "a mill target")?),
        other => {
            return Err(LoadError::Malformed(format!(
                "an activity has target kind {}",
                other.0
            )));
        }
    })
}

fn sex_parts(s: Sex) -> save::Sex {
    match s {
        Sex::Female => save::Sex::Female,
        Sex::Male => save::Sex::Male,
    }
}

fn sex_of(s: save::Sex) -> Result<Sex, LoadError> {
    match s {
        save::Sex::Female => Ok(Sex::Female),
        save::Sex::Male => Ok(Sex::Male),
        other => Err(LoadError::Malformed(format!(
            "a person has sex {}",
            other.0
        ))),
    }
}

// ---- land and settle ---------------------------------------------------------------------------

fn encode_land(land: &Land, rules: &Rules) -> Vec<u8> {
    let p = &land.patches;
    let mut fbb = FlatBufferBuilder::new();
    let habitat_ids: Vec<&str> = rules.land.habitats.iter().map(|h| h.id.as_str()).collect();
    let habitats = strings(&mut fbb, &habitat_ids);
    let class = fbb.create_vector(&p.class);
    let richness = fbb.create_vector(&p.richness);
    let resource_ids: Vec<&str> = rules.land.resources.iter().map(|r| r.id.as_str()).collect();
    let resources = strings(&mut fbb, &resource_ids);
    let good_ids: Vec<&str> = rules
        .land
        .resources
        .iter()
        .map(|r| {
            rules
                .catalog
                .goods
                .get(r.good)
                .map_or("", |g| g.id.as_str())
        })
        .collect();
    let resource_goods = strings(&mut fbb, &good_ids);
    let unit_kg: Vec<f64> = rules.land.resources.iter().map(|r| r.unit_kg).collect();
    let resource_unit_kg = fbb.create_vector(&unit_kg);
    let flat: Vec<f32> = land.stocks.iter().flatten().copied().collect();
    let stocks = fbb.create_vector(&flat);
    let weather = encode_weather(&mut fbb, &land.weather);
    let heads = fbb.create_vector(&land.water.heads);
    let water = save::Groundwater::create(
        &mut fbb,
        &save::GroundwaterArgs {
            heads: Some(heads),
            runoff_mm: land.water.runoff_mm,
        },
    );
    let root = save::Land::create(
        &mut fbb,
        &save::LandArgs {
            cols: p.cols,
            rows: p.rows,
            patch_cells: p.patch_cells,
            cell_size_m: p.cell_size_m,
            habitats: Some(habitats),
            class: Some(class),
            richness: Some(richness),
            resources: Some(resources),
            stocks: Some(stocks),
            stock_day: land.stock_day,
            resource_goods: Some(resource_goods),
            resource_unit_kg: Some(resource_unit_kg),
            weather: Some(weather),
            water: Some(water),
            ..Default::default()
        },
    );
    finish(fbb, root)
}

/// The weather's state (schema 24, ADR-0012 §1).
fn encode_weather<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    w: &Weather,
) -> flatbuffers::WIPOffset<save::Weather<'a>> {
    let t = &w.today;
    let today = save::WeatherDay::create(
        fbb,
        &save::WeatherDayArgs {
            day: t.day,
            wet: t.wet,
            precip_mm: t.precip_mm,
            mean_c: t.mean_c,
            min_c: t.min_c,
            max_c: t.max_c,
            wind_ms: t.wind_ms,
            wind_from_deg: t.wind_from_deg,
            gust_ms: t.gust_ms,
            fuel_moisture: t.fuel_moisture,
        },
    );
    let snow_mm = fbb.create_vector(&w.snow_mm);
    let months: Vec<_> = w
        .months
        .iter()
        .map(|m| {
            save::WeatherMonth::create(
                fbb,
                &save::WeatherMonthArgs {
                    year: m.year,
                    month: m.month,
                    days: m.days,
                    precip_mm: m.precip_mm,
                    wet_days: m.wet_days,
                    temp_sum_c: m.temp_sum_c,
                    min_c: m.min_c,
                    max_c: m.max_c,
                    frost_days: m.frost_days,
                    snow_days: m.snow_days,
                    soil_sum: m.soil_sum,
                    held: m.held,
                },
            )
        })
        .collect();
    let months = fbb.create_vector(&months);
    let forcings: Vec<_> = w
        .forcings
        .iter()
        .map(|f| {
            save::ClimateForcing::create(
                fbb,
                &save::ClimateForcingArgs {
                    from_day: f.from_day,
                    ramp_days: f.ramp_days,
                    offset_c: f.offset_c,
                    wet_factor: f.wet_factor,
                },
            )
        })
        .collect();
    let forcings = fbb.create_vector(&forcings);
    save::Weather::create(
        fbb,
        &save::WeatherArgs {
            today: Some(today),
            anomaly: w.anomaly,
            slow: w.slow,
            snow_base_m: w.snow_base_m,
            snow_mm: Some(snow_mm),
            soil_mm: w.soil_mm,
            cover_ease: w.cover_ease,
            months: Some(months),
            hold_slow: w.hold.map_or(0.0, |h| h.0),
            hold_until: w.hold.map_or(0, |h| h.1),
            wind_along: w.wind.0,
            wind_across: w.wind.1,
            forcings: Some(forcings),
        },
    )
}

/// The weather saved in schema 24 on, or (`None`) none.
fn decode_weather(w: save::Weather<'_>) -> Option<Weather> {
    let t = w.today()?;
    Some(Weather {
        today: WeatherDay {
            day: t.day(),
            wet: t.wet(),
            precip_mm: t.precip_mm(),
            mean_c: t.mean_c(),
            min_c: t.min_c(),
            max_c: t.max_c(),
            wind_ms: t.wind_ms(),
            wind_from_deg: t.wind_from_deg(),
            gust_ms: t.gust_ms(),
            fuel_moisture: t.fuel_moisture(),
        },
        anomaly: w.anomaly(),
        slow: w.slow(),
        wind: (w.wind_along(), w.wind_across()),
        snow_base_m: w.snow_base_m(),
        snow_mm: w.snow_mm().map(|v| v.iter().collect()).unwrap_or_default(),
        soil_mm: w.soil_mm(),
        cover_ease: w.cover_ease(),
        months: w
            .months()
            .iter()
            .flatten()
            .map(|m| MonthRecord {
                year: m.year(),
                month: m.month(),
                days: m.days(),
                precip_mm: m.precip_mm(),
                wet_days: m.wet_days(),
                temp_sum_c: m.temp_sum_c(),
                min_c: m.min_c(),
                max_c: m.max_c(),
                frost_days: m.frost_days(),
                snow_days: m.snow_days(),
                soil_sum: m.soil_sum(),
                held: m.held(),
            })
            .collect(),
        // The observer's drought (schema 84); none before.
        hold: (w.hold_until() > 0).then(|| (w.hold_slow(), w.hold_until())),
        // The observer's forcings on the climate (schema 170); none before.
        forcings: w
            .forcings()
            .iter()
            .flatten()
            .map(|f| civ_land::Forcing {
                from_day: f.from_day(),
                ramp_days: f.ramp_days(),
                offset_c: f.offset_c(),
                wet_factor: f.wet_factor(),
            })
            .collect(),
    })
}

fn decode_land(
    bytes: &[u8],
    rules: &Rules,
    map: &WorldMap,
    climatology: Climatology,
    seed: u64,
) -> Result<Land, LoadError> {
    let l = section_root::<save::Land>(bytes).map_err(|e| unreadable(SECTION_LAND, &e))?;
    let n = l.cols() as usize * l.rows() as usize;
    let habitats = read_strings(l.habitats());
    let catch_all = rules.land.habitats.len().saturating_sub(1) as u8;
    let remap: Vec<u8> = habitats
        .iter()
        .map(|h| {
            rules
                .land
                .habitats
                .iter()
                .position(|x| &x.id == h)
                .map_or(catch_all, |i| i as u8)
        })
        .collect();
    // A class beyond the saved habitat list stays out of range, for `Land::problems` to report.
    let class: Vec<u8> = l
        .class()
        .map(|c| c.bytes().to_vec())
        .unwrap_or_default()
        .into_iter()
        .map(|c| remap.get(usize::from(c)).copied().unwrap_or(u8::MAX))
        .collect();
    let richness: Vec<f32> = l.richness().map(|r| r.iter().collect()).unwrap_or_default();
    let saved_resources = read_strings(l.resources());
    // Absent before schema 3: then no saved resource is known to keep its units.
    let saved_goods = read_strings(l.resource_goods());
    let saved_units: Vec<f64> = l
        .resource_unit_kg()
        .map(|v| v.iter().collect())
        .unwrap_or_default();
    let flat: Vec<f32> = l.stocks().map(|s| s.iter().collect()).unwrap_or_default();
    if flat.len() != saved_resources.len() * n {
        return Err(LoadError::Malformed(format!(
            "section `land` has {} stock values for {} resources over {n} patches",
            flat.len(),
            saved_resources.len()
        )));
    }
    // Before schema 24 there was no weather: it starts from the old year's deviate as its slow
    // anomaly, with the soil at field capacity and no snow (ADR-0012 §6).
    let weather = match l.weather() {
        Some(w) => {
            let mut weather = decode_weather(w).ok_or_else(|| {
                LoadError::Malformed("section `land` has weather without a day".to_owned())
            })?;
            // Before schema 87 the day had no wind or dryness: they are drawn afresh, from no
            // anomaly, with nothing else changed (a day's fuel is never dry to nothing).
            if weather.today.fuel_moisture <= 0.0 {
                weather.draw_new_fields(&rules.land.weather, &climatology);
            }
            weather
        }
        None => {
            let (low, high) = civ_land::height_range(map);
            Weather::from_old(
                &rules.land.weather,
                &climatology,
                low,
                high,
                l.stock_day() + 1,
                l.climate_deviate(),
            )
        }
    };
    let floodplain = civ_world::WorldFloodplain::new(map, &civ_land::manning(&climatology));
    let mut land = Land {
        patches: Patches {
            cols: l.cols(),
            rows: l.rows(),
            patch_cells: l.patch_cells(),
            cell_size_m: l.cell_size_m(),
            class,
            richness,
            water: Vec::new(),
        },
        stocks: Vec::new(),
        stock_day: l.stock_day(),
        weather,
        climatology,
        settlements: Vec::new(),
        // Read from their own section (schema 4 on).
        fields: Vec::new(),
        plots: Vec::new(),
        buildings: Vec::new(),
        // Read from its own section (schema 7 on).
        wear: Wear::new(map.width, map.height, map.cell_size_m),
        // Read from its own section (schema 19 on), or placed from the seed on loading.
        deposits: Vec::new(),
        // Read from its own section (schema 20 on).
        earthworks: Vec::new(),
        banks: Vec::new(),
        ditches: Vec::new(),
        ditch_flows: Vec::new(),
        installations: Vec::new(),
        pit_water: Vec::new(),
        engines: Vec::new(),
        pit_bailing: Vec::new(),
        ground: civ_land::earth::GroundDelta::new(map.width, map.height, map.cell_size_m),
        // Read from its own section (schema 68 on).
        crossings: Default::default(),
        enclosures: Default::default(),
        // Read from its own section (schema 140 on).
        ways: Default::default(),
        // Read from its own section (schema 142 on).
        boats: Default::default(),
        carts: Default::default(),
        wagons: Default::default(),
        runs: Default::default(),
        gas: Default::default(),
        // The heads and the store below (schema 72 on); the aquifer derived.
        water: Default::default(),
        // Read from its own section (schema 74 on).
        wells: Default::default(),
        floodplain,
        rivers: civ_land::boating::RiverGraph::new(map),
        reach_discharges: map.reaches.iter().map(|r| r.discharge_m3s).collect(),
        flood_depths: vec![0.0; map.cell_count()],
        flood_revision: 0,
        river_forcings: Vec::new(),
        storm_checks: Vec::new(),
    };
    // A grid of the wrong size is reported by `Land::problems`; only fill stocks on a sound one.
    if land.patches.class.len() == n
        && land.patches.richness.len() == n
        && map.width.div_ceil(land.patches.patch_cells.max(1)) == land.patches.cols
        && map.height.div_ceil(land.patches.patch_cells.max(1)) == land.patches.rows
    {
        land.patches.measure_water(map);
        for (r, res) in rules.land.resources.iter().enumerate() {
            let good = rules.catalog.goods.get(res.good).map(|g| g.id.as_str());
            let same = saved_resources.iter().enumerate().position(|(i, id)| {
                *id == res.id
                    && saved_goods.get(i).map(String::as_str) == good
                    && saved_units.get(i) == Some(&res.unit_kg)
            });
            match same {
                Some(i) => land.stocks.push(flat[i * n..(i + 1) * n].to_vec()),
                None => {
                    land.stocks.push(vec![0.0; n]);
                    land.fill_equilibrium(&rules.land, r, land.stock_day);
                }
            }
        }
        // Before schema 72 there was no water under the ground: the heads settle where the
        // landscape's mean recharge holds them (ADR-0021 §2).
        land.derive_water(map, &rules.land, seed);
        match l.water() {
            Some(w) => {
                land.water.heads = w.heads().map(|v| v.iter().collect()).unwrap_or_default();
                land.water.runoff_mm = w.runoff_mm();
            }
            None => {
                let mean = land.climatology.surplus_mm_per_day;
                land.water.settle(&rules.land.water, mean);
            }
        }
        land.derive_floodplain(map, &rules.land);
    }
    Ok(land)
}

fn encode_settlements(settlements: &[Settlement]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = settlements
        .iter()
        .map(|s| {
            let name = fbb.create_string(&s.name);
            save::Settlement::create(
                &mut fbb,
                &save::SettlementArgs {
                    id: s.id.get(),
                    name: Some(name),
                    founded: s.founded.minutes(),
                    hearth: Some(&point(s.hearth_m)),
                    food_short: s.food_short,
                    harvest_kg: s.harvest_kg,
                    parent: raw(s.parent),
                    founding: s.founding.code(),
                    abandoned: s.abandoned.is_some(),
                    abandoned_at: s.abandoned.map_or(0, SimTime::minutes),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Settlements::create(
        &mut fbb,
        &save::SettlementsArgs {
            settlements: Some(list),
        },
    );
    finish(fbb, root)
}

fn decode_settlements(bytes: &[u8]) -> Result<Vec<Settlement>, LoadError> {
    let s = section_root::<save::Settlements>(bytes).map_err(|e| unreadable(SECTION_SETTLE, &e))?;
    s.settlements()
        .iter()
        .flatten()
        .enumerate()
        .map(|(i, s)| {
            // Before schema 50 the way of founding was not kept: the first settlement was the
            // founding band's and any later one a family the observer sent.
            let founding = match s.founding() {
                u8::MAX if i == 0 => Founding::Setup,
                u8::MAX => Founding::Sent,
                code => Founding::from_code(code).ok_or_else(|| {
                    LoadError::Malformed(format!("a settlement was founded in way {code}"))
                })?,
            };
            Ok(Settlement {
                id: required(s.id(), "a settlement")?,
                name: s.name().unwrap_or_default().to_owned(),
                founded: time(s.founded()),
                hearth_m: xy(s.hearth()),
                food_short: s.food_short(),
                harvest_kg: s.harvest_kg(),
                parent: PermanentId::from_raw(s.parent()),
                founding,
                abandoned: s.abandoned().then(|| time(s.abandoned_at())),
            })
        })
        .collect()
}

// ---- people ------------------------------------------------------------------------------------

fn sorted_people(pop: &Population) -> Vec<&Person> {
    let mut people: Vec<&Person> = pop.people.iter().map(|(_, p)| p).collect();
    people.sort_by_key(|p| p.id);
    people
}

fn encode_people(
    pop: &Population,
    activities: &[&str],
    goods: &[&str],
    skills: &[&str],
    techniques: &[&str],
) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let dictionary = strings(&mut fbb, activities);
    let good_dictionary = strings(&mut fbb, goods);
    let skill_dictionary = strings(&mut fbb, skills);
    let technique_dictionary = strings(&mut fbb, techniques);
    let list: Vec<_> = sorted_people(pop)
        .into_iter()
        .map(|p| encode_person(&mut fbb, p))
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::People::create(
        &mut fbb,
        &save::PeopleArgs {
            activities: Some(dictionary),
            people: Some(list),
            next_trip: pop.next_trip,
            goods: Some(good_dictionary),
            skills: Some(skill_dictionary),
            techniques: Some(technique_dictionary),
        },
    );
    finish(fbb, root)
}

fn encode_person<'a>(fbb: &mut FlatBufferBuilder<'a>, p: &Person) -> WIPOffset<save::Person<'a>> {
    let given = fbb.create_string(&p.given);
    let t = &p.traits;
    let traits = fbb.create_vector(&[
        t.openness,
        t.conscientiousness,
        t.extraversion,
        t.agreeableness,
        t.neuroticism,
        t.risk,
    ]);
    let steps: Vec<_> = p
        .act
        .steps
        .iter()
        .map(|s| {
            let none = (0.0, 0.0);
            let (kind, to, minutes, cell, unload) = match *s {
                Step::Walk { to } => (save::StepKind::Walk, to, 0, 0, false),
                Step::Work { minutes } => (save::StepKind::Work, none, minutes, 0, false),
                Step::Deposit => (save::StepKind::Deposit, none, 0, 0, false),
                Step::Wait { minutes } => (save::StepKind::Wait, none, minutes, 0, false),
                Step::Paddle { to } => (save::StepKind::Paddle, none, 0, to, false),
                Step::Haul { to } => (save::StepKind::Haul, to, 0, 0, false),
                Step::Ride { to } => (save::StepKind::Ride, none, 0, to, false),
                Step::Handle { minutes, unload } => {
                    (save::StepKind::Handle, none, minutes, 0, unload)
                }
            };
            save::Step::create(
                fbb,
                &save::StepArgs {
                    kind,
                    to: Some(&point(to)),
                    minutes,
                    cell,
                    unload,
                },
            )
        })
        .collect();
    let steps = fbb.create_vector(&steps);
    let (target_kind, target_index, target_id) = target_parts(p.act.target);
    let activity = save::Activity::create(
        fbb,
        &save::ActivityArgs {
            def: u32::from(p.act.def),
            target_kind,
            target_index,
            target_id,
            steps: Some(steps),
            step: p.act.step,
            started: p.act.started.minutes(),
            step_started: p.act.step_started.minutes(),
            step_ends: p.act.step_ends.minutes(),
            version: p.act.version,
        },
    );
    let trip = p.trip.as_ref().map(|t| {
        let points: Vec<save::Point> = t.points.iter().map(|&q| point(q)).collect();
        let points = fbb.create_vector(&points);
        let minutes = fbb.create_vector(&t.minutes);
        let ahead = t.ahead.map(|pd| {
            let (kind, id, sector) = pd.ahead.facility.code();
            save::TripAheadSave::create(
                fbb,
                &save::TripAheadSaveArgs {
                    kind,
                    id,
                    sector,
                    dir: pd.dir,
                    x: pd.ahead.at.0,
                    y: pd.ahead.at.1,
                    at_min: pd.ahead.at_min,
                    seg: pd.ahead.seg as u32,
                    ix: pd.ahead.inside.0,
                    iy: pd.ahead.inside.1,
                    headway_s: pd.headway_s,
                },
            )
        });
        save::Trip::create(
            fbb,
            &save::TripArgs {
                id: t.id,
                rev: t.rev,
                depart: t.depart.minutes(),
                points: Some(points),
                minutes: Some(minutes),
                ahead,
                afloat: t.afloat,
            },
        )
    });
    let skills: Vec<save::SkillLevel> = p
        .skills
        .iter()
        .map(|&(k, level)| save::SkillLevel::new(k, level))
        .collect();
    let skills = fbb.create_vector(&skills);
    let knows: Vec<save::Knowing> = p
        .knows
        .iter()
        .map(|k| {
            save::Knowing::new(
                k.since.minutes(),
                k.used.minutes(),
                raw(k.source.saved_id()),
                k.hours,
                k.technique,
                k.known,
                k.source.code(),
            )
        })
        .collect();
    let knows = fbb.create_vector(&knows);
    let (repro, conceived, repro_until, pregnancy_father, loss) = match p.repro {
        Repro::Open => (save::Repro::Open, 0, 0, 0, false),
        Repro::Pregnant {
            conceived,
            due,
            father,
            loss,
        } => (
            save::Repro::Pregnant,
            conceived.minutes(),
            due.minutes(),
            raw(father),
            loss,
        ),
        Repro::Recovering { until } => (save::Repro::Recovering, 0, until.minutes(), 0, false),
    };
    save::Person::create(
        fbb,
        &save::PersonArgs {
            id: p.id.get(),
            given: Some(given),
            sex: sex_parts(p.sex),
            born: p.born.minutes(),
            mother: raw(p.mother),
            father: raw(p.father),
            household: p.household.get(),
            traits: Some(traits),
            pos: Some(&point(p.pos)),
            energy_kcal: p.energy_kcal,
            satiety_until: p.satiety_until.minutes(),
            sleep_pressure: p.sleep_pressure,
            relatedness: p.relatedness,
            needs_at: p.needs_at.minutes(),
            burn_kcal_min: p.burn_kcal_min,
            asleep: p.asleep,
            company: p.company,
            activity: Some(activity),
            trip,
            carry_food_kcal: 0.0,
            carry_water_l: p.carrying.water_l,
            draws: p.draws,
            carry_good: p.carrying.good.map_or(-1, i32::from),
            carry_kg: p.carrying.kg,
            partner: raw(p.partner),
            repro,
            conceived,
            repro_until,
            pregnancy_father,
            loss,
            fecundity: p.fecundity,
            nursing: raw(p.nursing),
            skills: Some(skills),
            knows: Some(knows),
            tried: p.tried.map_or(-1, SimTime::minutes),
            objection: p.objection,
            risk_seen: p.risk_seen,
            guarded: raw(p.guarded.map(|(h, _)| h)),
            guarded_at: p.guarded.map_or(0, |(_, at)| at.minutes()),
        },
    )
}

/// The people, the next trip number, who decides again, and the skills of the content the save
/// knew nothing of.
type DecodedPeople = (Vec<Person>, u64, Vec<PermanentId>, Vec<usize>, Vec<usize>);

/// `homes` holds the households already loaded: a hearth target saved before schema 50 is their
/// settlement's.
fn decode_people(
    bytes: &[u8],
    rules: &Rules,
    schema: Schema,
    homes: &Population,
) -> Result<DecodedPeople, LoadError> {
    let root = section_root::<save::People>(bytes).map_err(|e| unreadable(SECTION_PEOPLE, &e))?;
    let map = activity_map(&read_strings(root.activities()), rules);
    let goods = good_map(&read_strings(root.goods()), rules);
    // Skills by saved index, in the loaded content; a skill it no longer has is forgotten.
    let skill_ids: Vec<Option<usize>> = read_strings(root.skills())
        .iter()
        .map(|id| rules.catalog.skill_index(id))
        .collect();
    // Skills the content has and the save never named (none before schema 9, when nobody had
    // any).
    let new_skills: Vec<usize> = if schema >= Schema::V9 {
        (0..rules.catalog.skills.len())
            .filter(|k| !skill_ids.contains(&Some(*k)))
            .collect()
    } else {
        Vec::new()
    };
    // Techniques by saved index, in the loaded content; one it no longer has is forgotten.
    let technique_ids: Vec<Option<usize>> = read_strings(root.techniques())
        .iter()
        .map(|id| rules.catalog.technique_index(id))
        .collect();
    // Techniques the content has and the save never named (none before schema 13, when nobody's
    // knowledge was kept).
    let new_techniques: Vec<usize> = if schema >= Schema::V13 {
        (0..rules.catalog.techniques.len())
            .filter(|t| !technique_ids.contains(&Some(*t)))
            .collect()
    } else {
        Vec::new()
    };
    let mut people = Vec::new();
    let mut redecide = Vec::new();
    for p in root.people().iter().flatten() {
        let person_id = required(p.id(), "a person")?;
        let a = p
            .activity()
            .ok_or_else(|| LoadError::Malformed(format!("person {person_id} has no activity")))?;
        let mut steps = Vec::new();
        for s in a.steps().iter().flatten() {
            steps.push(match s.kind() {
                save::StepKind::Walk => Step::Walk { to: xy(s.to()) },
                save::StepKind::Work => Step::Work {
                    minutes: s.minutes(),
                },
                save::StepKind::Deposit => Step::Deposit,
                save::StepKind::Wait => Step::Wait {
                    minutes: s.minutes(),
                },
                save::StepKind::Paddle => Step::Paddle { to: s.cell() },
                save::StepKind::Haul => Step::Haul { to: xy(s.to()) },
                save::StepKind::Ride => Step::Ride { to: s.cell() },
                save::StepKind::Handle => Step::Handle {
                    minutes: s.minutes(),
                    unload: s.unload(),
                },
                other => {
                    return Err(LoadError::Malformed(format!(
                        "person {person_id} has a step of kind {}",
                        other.0
                    )));
                }
            });
        }
        let def = match map.get(a.def() as usize) {
            Some(Some(def)) => *def,
            Some(None) => {
                redecide.push(person_id);
                0
            }
            None => {
                return Err(LoadError::Malformed(format!(
                    "person {person_id} does activity {} of {}",
                    a.def(),
                    map.len()
                )));
            }
        };
        let traits: Vec<f32> = p.traits().map(|t| t.iter().collect()).unwrap_or_default();
        let [
            openness,
            conscientiousness,
            extraversion,
            agreeableness,
            neuroticism,
            risk,
        ] = traits[..]
        else {
            return Err(LoadError::Malformed(format!(
                "person {person_id} has {} traits, expected 6",
                traits.len()
            )));
        };
        let trip = p.trip().map(|t| Trip {
            id: t.id(),
            rev: t.rev(),
            depart: time(t.depart()),
            points: t
                .points()
                .map(|v| v.iter().map(|q| (q.x(), q.y())).collect())
                .unwrap_or_default(),
            minutes: t.minutes().map(|v| v.iter().collect()).unwrap_or_default(),
            afloat: t.afloat(),
            ahead: t.ahead().and_then(|a| {
                Some(civ_agents::traffic::Pending {
                    ahead: civ_agents::traffic::Ahead {
                        facility: civ_agents::traffic::Facility::from_code(
                            a.kind(),
                            a.id(),
                            a.sector(),
                        )?,
                        at: (a.x(), a.y()),
                        at_min: a.at_min(),
                        seg: a.seg() as usize,
                        inside: (a.ix(), a.iy()),
                    },
                    dir: a.dir(),
                    headway_s: a.headway_s(),
                })
            }),
        });
        people.push(Person {
            id: person_id,
            given: p.given().unwrap_or_default().to_owned(),
            sex: sex_of(p.sex())?,
            born: time(p.born()),
            mother: id(p.mother()),
            father: id(p.father()),
            household: required(p.household(), "a person's household")?,
            traits: Traits {
                openness,
                conscientiousness,
                extraversion,
                agreeableness,
                neuroticism,
                risk,
            },
            pos: xy(p.pos()),
            energy_kcal: p.energy_kcal(),
            satiety_until: time(p.satiety_until()),
            sleep_pressure: p.sleep_pressure(),
            relatedness: p.relatedness(),
            needs_at: time(p.needs_at()),
            burn_kcal_min: p.burn_kcal_min(),
            asleep: p.asleep(),
            company: p.company(),
            act: Activity {
                def,
                target: match target_of(
                    a.target_kind(),
                    a.target_index(),
                    a.target_id(),
                    PermanentId::from_raw(p.household())
                        .and_then(|h| homes.household(h))
                        .and_then(|x| x.settlement),
                )? {
                    // A technique tried toward, by its place in the save's techniques: if the
                    // content no longer has it, the person decides again.
                    Target::Technique(t) => match technique_ids.get(usize::from(t)) {
                        Some(Some(now)) => Target::Technique(*now as u16),
                        Some(None) => {
                            if !redecide.contains(&person_id) {
                                redecide.push(person_id);
                            }
                            Target::None
                        }
                        None => {
                            return Err(LoadError::Malformed(format!(
                                "person {person_id} tries toward technique {t} of {}",
                                technique_ids.len()
                            )));
                        }
                    },
                    other => other,
                },
                steps,
                step: a.step(),
                started: time(a.started()),
                step_started: time(a.step_started()),
                step_ends: time(a.step_ends()),
                version: a.version(),
            },
            trip,
            carrying: carried(&p, &goods, rules, schema, person_id)?,
            draws: p.draws(),
            receipts: VecDeque::new(),
            partner: id(p.partner()),
            repro: match p.repro() {
                save::Repro::Open => Repro::Open,
                save::Repro::Pregnant => Repro::Pregnant {
                    conceived: time(p.conceived()),
                    due: time(p.repro_until()),
                    father: id(p.pregnancy_father()),
                    loss: p.loss(),
                },
                save::Repro::Recovering => Repro::Recovering {
                    until: time(p.repro_until()),
                },
                other => {
                    return Err(LoadError::Malformed(format!(
                        "person {person_id} is in reproductive state {}",
                        other.0
                    )));
                }
            },
            fecundity: p.fecundity(),
            nursing: id(p.nursing()),
            skills: {
                let mut skills: Vec<(u16, f32)> = Vec::new();
                for k in p.skills().iter().flatten() {
                    let known = skill_ids.get(usize::from(k.skill())).ok_or_else(|| {
                        LoadError::Malformed(format!(
                            "person {person_id} has skill {} of {}",
                            k.skill(),
                            skill_ids.len()
                        ))
                    })?;
                    if let Some(i) = known
                        && k.level().is_finite()
                    {
                        skills.push((*i as u16, k.level().clamp(0.0, 1.0)));
                    }
                }
                skills.sort_by_key(|(k, _)| *k);
                skills.dedup_by_key(|(k, _)| *k);
                skills
            },
            knows: {
                let mut knows: Vec<Know> = Vec::new();
                for k in p.knows().iter().flatten() {
                    let t = technique_ids
                        .get(usize::from(k.technique()))
                        .ok_or_else(|| {
                            LoadError::Malformed(format!(
                                "person {person_id} knows technique {} of {}",
                                k.technique(),
                                technique_ids.len()
                            ))
                        })?;
                    if let Some(t) = t {
                        knows.push(Know {
                            technique: *t as u16,
                            known: k.known(),
                            hours: if k.hours().is_finite() {
                                k.hours().max(0.0)
                            } else {
                                0.0
                            },
                            since: time(k.since()),
                            source: KnowSource::from_code(k.source(), id(k.source_person())),
                            used: time(k.used()),
                        });
                    }
                }
                knows.sort_by_key(|k| k.technique);
                knows.dedup_by_key(|k| k.technique);
                knows
            },
            tried: (p.tried() >= 0).then(|| time(p.tried())),
            // Not kept before schema 31 (-1): drawn once the people are in.
            objection: if p.objection().is_finite() {
                p.objection().min(1.0)
            } else {
                -1.0
            },
            risk_seen: if p.risk_seen().is_finite() {
                p.risk_seen().min(1.0)
            } else {
                -1.0
            },
            guarded: id(p.guarded()).map(|h| (h, time(p.guarded_at()))),
        });
    }
    Ok((
        people,
        root.next_trip(),
        redecide,
        new_skills,
        new_techniques,
    ))
}

/// What a saved person carries, in the loaded content's goods.
fn carried(
    p: &save::Person<'_>,
    goods: &[Option<usize>],
    rules: &Rules,
    schema: Schema,
    who: PermanentId,
) -> Result<Load, LoadError> {
    let water_l = p.carry_water_l();
    let (good, kg) = match schema {
        Schema::V2 => match provisions_kg(rules, f64::from(p.carry_food_kcal())) {
            Some((g, kg)) if kg > 0.0 => (Some(g), kg as f32),
            _ => (None, 0.0),
        },
        Schema::V3
        | Schema::V4
        | Schema::V5
        | Schema::V6
        | Schema::V7
        | Schema::V8
        | Schema::V9
        | Schema::V10
        | Schema::V11
        | Schema::V12
        | Schema::V13
        | Schema::V14
        | Schema::V15
        | Schema::V16
        | Schema::V17
        | Schema::V18
        | Schema::V19
        | Schema::V20
        | Schema::V21
        | Schema::V22
        | Schema::V23
        | Schema::V24
        | Schema::V25
        | Schema::V26
        | Schema::V27
        | Schema::V28
        | Schema::V29
        | Schema::V30
        | Schema::V31
        | Schema::V32
        | Schema::V33
        | Schema::V34
        | Schema::V35
        | Schema::V36
        | Schema::V37
        | Schema::V38
        | Schema::V39
        | Schema::V40
        | Schema::V41
        | Schema::V42
        | Schema::V43
        | Schema::V44
        | Schema::V45
        | Schema::V46
        | Schema::V47
        | Schema::V48
        | Schema::V49
        | Schema::V50
        | Schema::V51
        | Schema::V52
        | Schema::V53
        | Schema::V54
        | Schema::V55
        | Schema::V56
        | Schema::V57
        | Schema::V58
        | Schema::V59
        | Schema::V60
        | Schema::V61
        | Schema::V62
        | Schema::V63
        | Schema::V64
        | Schema::V65
        | Schema::V66
        | Schema::V67
        | Schema::V68
        | Schema::V69
        | Schema::V70
        | Schema::V71
        | Schema::V72
        | Schema::V73
        | Schema::V74
        | Schema::V75
        | Schema::V76
        | Schema::V77
        | Schema::V78
        | Schema::V79
        | Schema::V80
        | Schema::V81
        | Schema::V82
        | Schema::V83
        | Schema::V84
        | Schema::V85
        | Schema::V86
        | Schema::V87
        | Schema::V88
        | Schema::V89
        | Schema::V90
        | Schema::V91
        | Schema::V92
        | Schema::V93
        | Schema::V94
        | Schema::V95
        | Schema::V96
        | Schema::V97
        | Schema::V98
        | Schema::V99
        | Schema::V100
        | Schema::V101
        | Schema::V102
        | Schema::V103
        | Schema::V104
        | Schema::V105
        | Schema::V106
        | Schema::V107
        | Schema::V108
        | Schema::V109
        | Schema::V110
        | Schema::V111
        | Schema::V112
        | Schema::V113
        | Schema::V114
        | Schema::V115
        | Schema::V116
        | Schema::V117
        | Schema::V118
        | Schema::V119
        | Schema::V120
        | Schema::V121
        | Schema::V122
        | Schema::V123
        | Schema::V124
        | Schema::V125
        | Schema::V126
        | Schema::V127
        | Schema::V128
        | Schema::V129
        | Schema::V130
        | Schema::V131
        | Schema::V132
        | Schema::V133
        | Schema::V134
        | Schema::V135
        | Schema::V136
        | Schema::V137
        | Schema::V138
        | Schema::V139
        | Schema::V140
        | Schema::V141
        | Schema::V142
        | Schema::V143
        | Schema::V144
        | Schema::V145
        | Schema::V146
        | Schema::V147
        | Schema::V148
        | Schema::V149
        | Schema::V150
        | Schema::V151
        | Schema::V152
        | Schema::V153
        | Schema::V154
        | Schema::V155
        | Schema::V156
        | Schema::V157
        | Schema::V158
        | Schema::V159
        | Schema::V160
        | Schema::V161
        | Schema::V162
        | Schema::V163
        | Schema::V164
        | Schema::V165
        | Schema::V166
        | Schema::V167
        | Schema::V168
        | Schema::V169
        | Schema::V171
        | Schema::V181
        | Schema::V182
        | Schema::V186 => {
            match p.carry_good() {
                -1 => (None, 0.0),
                i => match usize::try_from(i).ok().and_then(|i| goods.get(i)) {
                    // A good the content no longer has is dropped.
                    Some(found) => (*found, p.carry_kg()),
                    None => {
                        return Err(LoadError::Malformed(format!(
                            "person {who} carries good {i} of {}",
                            goods.len()
                        )));
                    }
                },
            }
        }
    };
    let good = good.map(|g| u16::try_from(g).unwrap_or(u16::MAX));
    Ok(Load {
        good,
        kg: if good.is_some() { kg } else { 0.0 },
        water_l,
    })
}

// ---- houses ------------------------------------------------------------------------------------

fn encode_households(
    pop: &Population,
    goods: &[&str],
    resources: &[&str],
    recipes: &[&str],
) -> Vec<u8> {
    let mut households: Vec<&Household> = pop.households.iter().map(|(_, h)| h).collect();
    households.sort_by_key(|h| h.id);
    let mut fbb = FlatBufferBuilder::new();
    let good_dictionary = strings(&mut fbb, goods);
    let resource_dictionary = strings(&mut fbb, resources);
    let list: Vec<_> = households
        .into_iter()
        .map(|h| {
            let members: Vec<u64> = h.members.iter().map(|m| m.get()).collect();
            let members = fbb.create_vector(&members);
            let known: Vec<save::KnownResource> = h
                .known
                .iter()
                .map(|k| save::KnownResource::new(k.seen_day, k.patch, k.rate, k.hours, k.resource))
                .collect();
            let known = fbb.create_vector(&known);
            let mut stores = h.stores.clone();
            stores.resize(goods.len(), 0.0);
            let stores = fbb.create_vector(&stores);
            let offers: Vec<_> = h.offers.iter().map(|o| encode_offer(&mut fbb, o)).collect();
            let offers = fbb.create_vector(&offers);
            let taste = fbb.create_vector(&h.taste.traits());
            // What floods cost it (schema 102).
            let flood_losses: Vec<_> = h
                .flood_losses
                .iter()
                .map(|l| {
                    save::FloodLossSave::create(
                        &mut fbb,
                        &save::FloodLossSaveArgs {
                            day: l.day,
                            cell: l.cell,
                            depth_cm: l.depth_cm,
                            kcal: l.kcal,
                            hours: l.hours,
                        },
                    )
                })
                .collect();
            let flood_losses = (!flood_losses.is_empty()).then(|| fbb.create_vector(&flood_losses));
            // What fire took that it knew of (schema 107).
            let fire_losses: Vec<_> = h
                .fire_losses
                .iter()
                .map(|l| {
                    save::FireLossSave::create(
                        &mut fbb,
                        &save::FireLossSaveArgs {
                            day: l.day,
                            building: l.building.get(),
                            own: l.own,
                            burnable: l.burnable,
                            hours: l.hours,
                        },
                    )
                })
                .collect();
            let fire_losses = (!fire_losses.is_empty()).then(|| fbb.create_vector(&fire_losses));
            // What it heard gave way (schema 108).
            let gave_way: Vec<_> = h
                .gave_way
                .iter()
                .map(|l| {
                    save::GaveWaySave::create(
                        &mut fbb,
                        &save::GaveWaySaveArgs {
                            day: l.day,
                            building: l.building.get(),
                            own: l.own,
                            wall_kind: l.wall_kind,
                            hours: l.hours,
                            storeys: l.storeys,
                            tied: l.tied,
                        },
                    )
                })
                .collect();
            let gave_way = (!gave_way.is_empty()).then(|| fbb.create_vector(&gave_way));
            // Its batches under way (schema 171), each by its recipe's content id.
            let batches: Vec<_> = h
                .batches
                .iter()
                .filter_map(|b| {
                    let recipe = fbb.create_string(recipes.get(b.recipe)?);
                    Some(save::BatchSave::create(
                        &mut fbb,
                        &save::BatchSaveArgs {
                            recipe: Some(recipe),
                            units: b.units,
                            quality: b.quality,
                            ends_day: b.ends_day,
                        },
                    ))
                })
                .collect();
            let batches = (!batches.is_empty()).then(|| fbb.create_vector(&batches));
            // What it had made for a mill review (schema 174), absent before its first.
            let mill_seen = h.mill_seen.map(|m| {
                save::MillSeenSave::create(
                    &mut fbb,
                    &save::MillSeenSaveArgs {
                        day: m.day,
                        kg: m.kg,
                    },
                )
            });
            save::Household::create(
                &mut fbb,
                &save::HouseholdArgs {
                    id: h.id.get(),
                    members: Some(members),
                    home: Some(&point(h.home)),
                    settlement: raw(h.settlement),
                    food_kcal: 0.0,
                    water_l: h.water_l,
                    water_at: h.water_at.minutes(),
                    known: None,
                    stores: Some(stores),
                    stores_at: h.stores_at.minutes(),
                    known_resources: Some(known),
                    offers: Some(offers),
                    taste: Some(taste),
                    admired: raw(h.admired),
                    midden_kg: h.midden.kg,
                    midden_at: h.midden.at.minutes(),
                    water_use_l: h.water_use_l,
                    water_walk_min: h.water_walk_min,
                    flood_home_cm: h.flood_memory.home_depth_cm,
                    flood_field_cm: h.flood_memory.field_depth_cm,
                    flood_day: h.flood_memory.day,
                    flood_losses,
                    ditch_worth: h.ditch_worth,
                    fire_losses,
                    gave_way,
                    batches,
                    mill_seen,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Households::create(
        &mut fbb,
        &save::HouseholdsArgs {
            households: Some(list),
            goods: Some(good_dictionary),
            resources: Some(resource_dictionary),
        },
    );
    finish(fbb, root)
}

fn decode_households(
    bytes: &[u8],
    rules: &Rules,
    schema: Schema,
    now: SimTime,
) -> Result<Vec<Household>, LoadError> {
    let root =
        section_root::<save::Households>(bytes).map_err(|e| unreadable(SECTION_HOUSES, &e))?;
    let goods = good_map(&read_strings(root.goods()), rules);
    let resources: Vec<Option<u16>> = read_strings(root.resources())
        .iter()
        .map(|id| rules.land.resource(id).map(|r| r as u16))
        .collect();
    let catalog_goods = rules.catalog.goods.len();
    let mut out = Vec::new();
    for h in root.households().iter().flatten() {
        let hh_id = required(h.id(), "a household")?;
        let mut members = Vec::new();
        for m in h.members().iter().flatten() {
            members.push(required(m, "a household member")?);
        }
        let mut stores = vec![0.0; catalog_goods];
        let (stores_at, known) = match schema {
            Schema::V2 => {
                if let Some((g, kg)) = provisions_kg(rules, h.food_kcal()) {
                    stores[g] = kg;
                }
                (now, Vec::new())
            }
            Schema::V3
            | Schema::V4
            | Schema::V5
            | Schema::V6
            | Schema::V7
            | Schema::V8
            | Schema::V9
            | Schema::V10
            | Schema::V11
            | Schema::V12
            | Schema::V13
            | Schema::V14
            | Schema::V15
            | Schema::V16
            | Schema::V17
            | Schema::V18
            | Schema::V19
            | Schema::V20
            | Schema::V21
            | Schema::V22
            | Schema::V23
            | Schema::V24
            | Schema::V25
            | Schema::V26
            | Schema::V27
            | Schema::V28
            | Schema::V29
            | Schema::V30
            | Schema::V31
            | Schema::V32
            | Schema::V33
            | Schema::V34
            | Schema::V35
            | Schema::V36
            | Schema::V37
            | Schema::V38
            | Schema::V39
            | Schema::V40
            | Schema::V41
            | Schema::V42
            | Schema::V43
            | Schema::V44
            | Schema::V45
            | Schema::V46
            | Schema::V47
            | Schema::V48
            | Schema::V49
            | Schema::V50
            | Schema::V51
            | Schema::V52
            | Schema::V53
            | Schema::V54
            | Schema::V55
            | Schema::V56
            | Schema::V57
            | Schema::V58
            | Schema::V59
            | Schema::V60
            | Schema::V61
            | Schema::V62
            | Schema::V63
            | Schema::V64
            | Schema::V65
            | Schema::V66
            | Schema::V67
            | Schema::V68
            | Schema::V69
            | Schema::V70
            | Schema::V71
            | Schema::V72
            | Schema::V73
            | Schema::V74
            | Schema::V75
            | Schema::V76
            | Schema::V77
            | Schema::V78
            | Schema::V79
            | Schema::V80
            | Schema::V81
            | Schema::V82
            | Schema::V83
            | Schema::V84
            | Schema::V85
            | Schema::V86
            | Schema::V87
            | Schema::V88
            | Schema::V89
            | Schema::V90
            | Schema::V91
            | Schema::V92
            | Schema::V93
            | Schema::V94
            | Schema::V95
            | Schema::V96
            | Schema::V97
            | Schema::V98
            | Schema::V99
            | Schema::V100
            | Schema::V101
            | Schema::V102
            | Schema::V103
            | Schema::V104
            | Schema::V105
            | Schema::V106
            | Schema::V107
            | Schema::V108
            | Schema::V109
            | Schema::V110
            | Schema::V111
            | Schema::V112
            | Schema::V113
            | Schema::V114
            | Schema::V115
            | Schema::V116
            | Schema::V117
            | Schema::V118
            | Schema::V119
            | Schema::V120
            | Schema::V121
            | Schema::V122
            | Schema::V123
            | Schema::V124
            | Schema::V125
            | Schema::V126
            | Schema::V127
            | Schema::V128
            | Schema::V129
            | Schema::V130
            | Schema::V131
            | Schema::V132
            | Schema::V133
            | Schema::V134
            | Schema::V135
            | Schema::V136
            | Schema::V137
            | Schema::V138
            | Schema::V139
            | Schema::V140
            | Schema::V141
            | Schema::V142
            | Schema::V143
            | Schema::V144
            | Schema::V145
            | Schema::V146
            | Schema::V147
            | Schema::V148
            | Schema::V149
            | Schema::V150
            | Schema::V151
            | Schema::V152
            | Schema::V153
            | Schema::V154
            | Schema::V155
            | Schema::V156
            | Schema::V157
            | Schema::V158
            | Schema::V159
            | Schema::V160
            | Schema::V161
            | Schema::V162
            | Schema::V163
            | Schema::V164
            | Schema::V165
            | Schema::V166
            | Schema::V167
            | Schema::V168
            | Schema::V169
            | Schema::V171
            | Schema::V181
            | Schema::V182
            | Schema::V186 => {
                let saved: Vec<f64> = h.stores().map(|v| v.iter().collect()).unwrap_or_default();
                if saved.len() != goods.len() {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} stores {} goods of {}",
                        saved.len(),
                        goods.len()
                    )));
                }
                // Goods the content no longer has are dropped.
                for (kg, g) in saved.iter().zip(&goods) {
                    if let Some(g) = g {
                        stores[*g] = *kg;
                    }
                }
                let mut known = Vec::new();
                for k in h.known_resources().iter().flatten() {
                    match resources.get(usize::from(k.resource())) {
                        Some(Some(r)) => known.push(KnownPatch {
                            resource: *r,
                            patch: k.patch(),
                            rate: k.rate(),
                            hours: k.hours(),
                            seen_day: k.seen_day(),
                        }),
                        // A resource the content no longer has is forgotten.
                        Some(None) => {}
                        None => {
                            return Err(LoadError::Malformed(format!(
                                "household {hh_id} remembers resource {} of {}",
                                k.resource(),
                                resources.len()
                            )));
                        }
                    }
                }
                (time(h.stores_at()), known)
            }
        };
        // Offers arrived with schema 10: before, households post them at their next review.
        let saved_offers = if schema >= Schema::V10 {
            h.offers()
        } else {
            None
        };
        let offers = decode_offers(saved_offers, &goods, || {
            format!("an offer of household {hh_id}")
        })?;
        out.push(Household {
            id: hh_id,
            members,
            home: xy(h.home()),
            settlement: id(h.settlement()),
            stores,
            stores_at,
            water_l: h.water_l(),
            water_at: time(h.water_at()),
            known,
            // Derived from the buildings once the land is read.
            sheltered: false,
            keeping: civ_agents::person::Keeping::default(),
            // Counters start again on load.
            flows: Default::default(),
            offers,
            // Before schema 22 nobody had a taste: it is drawn on loading, as a band's would be.
            // Before schema 169 a taste had the roof's three traits and no front: one of three
            // traits loads with its front bare, unornamented, without openings or symmetry.
            taste: match h.taste() {
                Some(t) if schema >= Schema::V22 => {
                    let t: Vec<f32> = t.iter().collect();
                    if t.len() != civ_agents::params::Taste::TRAITS
                        && t.len() != civ_agents::params::Taste::ROOF_TRAITS
                    {
                        return Err(LoadError::Malformed(format!(
                            "household {hh_id} has a taste of {} traits",
                            t.len()
                        )));
                    }
                    if t.iter().any(|v| !v.is_finite()) {
                        return Err(LoadError::Malformed(format!(
                            "household {hh_id} has a taste that is not a number"
                        )));
                    }
                    civ_agents::params::Taste::from_traits(std::array::from_fn(|k| {
                        t.get(k).copied().unwrap_or(0.0)
                    }))
                }
                _ => civ_agents::params::Taste::default(),
            },
            admired: id(h.admired()),
            // Before schema 73 every household used the profile's water a day.
            water_use_l: {
                let l = if schema >= Schema::V73 {
                    h.water_use_l()
                } else {
                    0.0
                };
                if !(l.is_finite() && l >= 0.0) {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} uses {l} litres of water a day"
                    )));
                }
                l
            },
            // Before schema 83 no walk to water was recorded: none until its people fetch again.
            water_walk_min: {
                let m = if schema >= Schema::V83 {
                    h.water_walk_min()
                } else {
                    0.0
                };
                if !(m.is_finite() && m >= 0.0) {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} walks {m} minutes to water"
                    )));
                }
                m
            },
            // Before schema 27 no household kept a midden: one is begun as the save is loaded.
            midden: if schema < Schema::V27 {
                civ_agents::person::Midden::begun(now)
            } else {
                let kg = h.midden_kg();
                if !(kg.is_finite() && kg >= 0.0) {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} has a midden of {kg} kg"
                    )));
                }
                civ_agents::person::Midden {
                    kg,
                    at: time(h.midden_at()),
                }
            },
            flood_memory: if schema >= Schema::V93 {
                civ_agents::person::FloodMemory {
                    home_depth_cm: h.flood_home_cm(),
                    field_depth_cm: h.flood_field_cm(),
                    day: h.flood_day(),
                }
            } else {
                civ_agents::person::FloodMemory::default()
            },
            // What floods cost it (schema 102); before, nothing remembered.
            flood_losses: {
                let mut out: Vec<civ_agents::FloodLoss> = Vec::new();
                for l in h.flood_losses().iter().flatten() {
                    let ok = l.kcal().is_finite()
                        && l.kcal() >= 0.0
                        && l.hours().is_finite()
                        && l.hours() >= 0.0
                        && out.last().is_none_or(|p| p.day <= l.day());
                    if !ok {
                        return Err(LoadError::Malformed(format!(
                            "household {hh_id} remembers a flood loss out of order or of no \
                             amount"
                        )));
                    }
                    out.push(civ_agents::FloodLoss {
                        day: l.day(),
                        cell: l.cell(),
                        depth_cm: l.depth_cm(),
                        kcal: l.kcal(),
                        hours: l.hours(),
                    });
                }
                out
            },
            // What digging its ditch is worth to it (schema 106); before, nothing.
            ditch_worth: {
                let v = h.ditch_worth();
                if !(v.is_finite() && v >= 0.0) || (schema < Schema::V106 && v != 0.0) {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} reckons its ditch worth {v}"
                    )));
                }
                v
            },
            // What fire took that it knew of (schema 107); before, nothing remembered.
            fire_losses: {
                let mut out: Vec<civ_agents::person::FireLoss> = Vec::new();
                let losses = h.fire_losses();
                if schema < Schema::V107 && losses.is_some() {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} remembers fires in a save before schema 107"
                    )));
                }
                for l in losses.iter().flatten() {
                    let building = PermanentId::from_raw(l.building());
                    let ok = l.hours().is_finite()
                        && l.hours() >= 0.0
                        && building.is_some()
                        && out.last().is_none_or(|p| p.day <= l.day());
                    let (Some(building), true) = (building, ok) else {
                        return Err(LoadError::Malformed(format!(
                            "household {hh_id} remembers a fire out of order, of no building \
                             or of no amount"
                        )));
                    };
                    out.push(civ_agents::person::FireLoss {
                        day: l.day(),
                        building,
                        own: l.own(),
                        burnable: l.burnable(),
                        hours: l.hours(),
                    });
                }
                out
            },
            // What it heard gave way (schema 108); before, nothing remembered.
            gave_way: {
                let mut out: Vec<civ_agents::person::GaveWay> = Vec::new();
                let heard = h.gave_way();
                if schema < Schema::V108 && heard.is_some() {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} remembers buildings that gave way in a save before \
                         schema 108"
                    )));
                }
                for l in heard.iter().flatten() {
                    let building = PermanentId::from_raw(l.building());
                    let ok = l.hours().is_finite()
                        && l.hours() >= 0.0
                        && l.wall_kind() <= 3
                        && (1..=civ_grammar::frame::MAX_STOREYS).contains(&l.storeys())
                        && building.is_some()
                        && out.last().is_none_or(|p| p.day <= l.day());
                    let (Some(building), true) = (building, ok) else {
                        return Err(LoadError::Malformed(format!(
                            "household {hh_id} remembers a building that gave way out of order, \
                             of no building, wall kind, storeys or amount"
                        )));
                    };
                    out.push(civ_agents::person::GaveWay {
                        day: l.day(),
                        building,
                        own: l.own(),
                        wall_kind: l.wall_kind(),
                        hours: l.hours(),
                        // Its storeys and ties (schema 163); before, a storey untied.
                        storeys: l.storeys(),
                        tied: l.tied(),
                    });
                }
                out
            },
            // Its batches under way (schema 171); before, none. One of a recipe the content no
            // longer has is dropped, its inputs with it.
            batches: {
                let mut out: Vec<civ_agents::person::Batch> = Vec::new();
                let saved = h.batches();
                if schema < Schema::V171 && saved.is_some() {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} keeps batches in a save before schema 171"
                    )));
                }
                for b in saved.iter().flatten() {
                    let ok = b.units().is_finite()
                        && b.units() > 0.0
                        && b.quality().is_finite()
                        && b.quality() >= 0.0;
                    if !ok {
                        return Err(LoadError::Malformed(format!(
                            "household {hh_id} keeps a batch of no units or quality"
                        )));
                    }
                    let Some(recipe) = b
                        .recipe()
                        .and_then(|id| rules.catalog.recipes.iter().position(|r| r.id == id))
                    else {
                        continue;
                    };
                    out.push(civ_agents::person::Batch {
                        recipe,
                        units: b.units(),
                        quality: b.quality(),
                        ends_day: b.ends_day(),
                    });
                }
                out
            },
            // What it had made for a mill review (schema 174); before, none.
            mill_seen: match h.mill_seen() {
                Some(m) if m.kg().is_finite() && m.kg() >= 0.0 => {
                    Some(civ_agents::person::MillSeen {
                        day: m.day(),
                        kg: m.kg(),
                    })
                }
                Some(_) => {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} counts no kilograms for a mill"
                    )));
                }
                None => None,
            },
        });
    }
    Ok(out)
}

// ---- order -------------------------------------------------------------------------------------

fn encode_order(order: &civ_agents::crime::Order, goods: &[&str]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let good_dictionary = strings(&mut fbb, goods);
    let incidents: Vec<_> = order
        .incidents
        .iter()
        .map(|i| {
            let taken: Vec<save::TakenGood> = i
                .goods
                .iter()
                .map(|&(g, amount)| save::TakenGood::new(g, amount))
                .collect();
            let taken = fbb.create_vector(&taken);
            let seen: Vec<u64> = i.seen_by.iter().map(|p| p.get()).collect();
            let seen = fbb.create_vector(&seen);
            save::IncidentSave::create(
                &mut fbb,
                &save::IncidentSaveArgs {
                    id: i.id,
                    at: i.at.minutes(),
                    actor: i.actor.get(),
                    actor_household: i.actor_household.get(),
                    target: i.target.get(),
                    place: Some(&point(i.place)),
                    outcome: i.outcome.code(),
                    goods: Some(taken),
                    kcal: i.kcal,
                    seen_by: Some(seen),
                    noticed: i.noticed,
                    food_days: i.food_days,
                    richer: i.richer,
                    across: i.across.map_or(0, |s| s.get()),
                },
            )
        })
        .collect();
    let incidents = fbb.create_vector(&incidents);
    let beliefs: Vec<save::BeliefSave> = order
        .beliefs
        .iter()
        .map(|b| {
            save::BeliefSave::new(
                b.holder.get(),
                raw(b.taker),
                raw(b.from),
                raw(b.origin),
                b.day,
                b.incident,
                b.source.code(),
            )
        })
        .collect();
    let beliefs = fbb.create_vector(&beliefs);
    let responses: Vec<save::ResponseSave2> = order
        .responses
        .iter()
        .map(|r| {
            save::ResponseSave2::new(
                r.household.get(),
                r.by.get(),
                r.day,
                r.incident,
                r.points,
                r.report_points.unwrap_or(0.0),
                r.choice.code(),
                r.report_points.is_some(),
            )
        })
        .collect();
    let responses = fbb.create_vector(&responses);
    let obligations: Vec<save::ObligationSave2> = order
        .obligations
        .iter()
        .map(|o| {
            let (answer, points) = match o.answer {
                None => (0, 0.0),
                Some((true, p)) => (1, p),
                Some((false, p)) => (2, p),
            };
            save::ObligationSave2::new(
                o.debtor.get(),
                o.beneficiary.get(),
                o.made,
                o.due,
                o.id,
                o.incident,
                o.case.unwrap_or(0),
                o.kcal,
                o.paid_kcal,
                points,
                o.standing.code(),
                answer,
                o.kind.code(),
            )
        })
        .collect();
    let obligations = fbb.create_vector(&obligations);
    let cases: Vec<_> = order
        .cases
        .iter()
        .map(|c| {
            let leads: Vec<u64> = c.leads.iter().map(|p| p.get()).collect();
            let leads = fbb.create_vector(&leads);
            let stances: Vec<save::CaseStanceSave> = c
                .stances
                .iter()
                .map(|r| {
                    save::CaseStanceSave::new(
                        r.person.get(),
                        r.household.get(),
                        r.stake,
                        r.belief,
                        r.regard,
                        stance_code(r.stance),
                    )
                })
                .collect();
            let stances = fbb.create_vector(&stances);
            let precedent: Vec<f32> = c.stances.iter().map(|r| r.precedent).collect();
            let precedent = fbb.create_vector(&precedent);
            save::CaseSave::create(
                &mut fbb,
                &save::CaseSaveArgs {
                    id: c.id,
                    incident: c.incident,
                    settlement: c.settlement.get(),
                    law: c.law.get(),
                    accuser: c.accuser.get(),
                    by: c.by.get(),
                    accused: c.accused.get(),
                    accused_household: c.accused_household.get(),
                    kcal: c.kcal,
                    leads: Some(leads),
                    opened: c.opened,
                    stage: c.stage.code(),
                    heard: c.heard.is_some(),
                    heard_at: c.heard.map_or(0, |t| t.minutes()),
                    eligible: c.eligible,
                    stances: Some(stances),
                    exiled: c.exiled,
                    forum: c.forum.map_or(0, PermanentId::get),
                    challenge: c.challenge.map_or(0, PermanentId::get),
                    tags: c.tags,
                    departed: c.departed.unwrap_or(0),
                    stance_precedent: Some(precedent),
                    loan: c.loan.unwrap_or(0),
                    building: c.building.map_or(0, |b| b.get()),
                },
            )
        })
        .collect();
    let cases = fbb.create_vector(&cases);
    let sightings: Vec<save::SightingSave> = order
        .sightings
        .iter()
        .map(|s| {
            save::SightingSave::new(
                s.officer.get(),
                s.day,
                s.incident,
                s.kcal,
                s.points.0,
                s.points.1.unwrap_or(0.0),
                s.kept.code(),
                s.points.1.is_some(),
            )
        })
        .collect();
    let sightings = fbb.create_vector(&sightings);
    // Encounters (schema 47).
    let encounters: Vec<_> = order
        .encounters
        .iter()
        .map(|e| {
            let met: Vec<save::MetSave> = e
                .met
                .iter()
                .map(|&(p, m)| save::MetSave::new(p.get(), m.code()))
                .collect();
            let met = fbb.create_vector(&met);
            let harms: Vec<save::HarmSave> = e
                .harms
                .iter()
                .map(|h| save::HarmSave::new(h.by.get(), h.to.get(), h.days, h.killed))
                .collect();
            let harms = fbb.create_vector(&harms);
            save::EncounterSave::create(
                &mut fbb,
                &save::EncounterSaveArgs {
                    id: e.id,
                    day: e.day,
                    officer: e.officer.get(),
                    household: e.household.get(),
                    obligation: e.obligation,
                    owed_kcal: e.owed_kcal,
                    met: Some(met),
                    forced: e.forced,
                    taken_kcal: e.taken_kcal,
                    harms: Some(harms),
                },
            )
        })
        .collect();
    let encounters = fbb.create_vector(&encounters);
    // Bands and the encounters they met (schema 110).
    let ids = |fbb: &mut flatbuffers::FlatBufferBuilder<'static>, v: &[PermanentId]| {
        let raw: Vec<u64> = v.iter().map(|p| p.get()).collect();
        fbb.create_vector(&raw)
    };
    let bands: Vec<_> = order
        .bands
        .iter()
        .map(|b| {
            let members = ids(&mut fbb, &b.members);
            let asked = ids(&mut fbb, &b.asked);
            let mustered = ids(&mut fbb, &b.mustered);
            let arrived = ids(&mut fbb, &b.arrived);
            let home = ids(&mut fbb, &b.home);
            let taken: Vec<save::TakenGood> = b
                .taken
                .iter()
                .map(|&(g, amount)| save::TakenGood::new(g, amount))
                .collect();
            let taken = fbb.create_vector(&taken);
            save::BandSave::create(
                &mut fbb,
                &save::BandSaveArgs {
                    id: b.id,
                    organizer: b.organizer.get(),
                    settlement: b.settlement.get(),
                    purpose: b.purpose.code(),
                    target: b.target.get(),
                    place: Some(&point(b.place)),
                    called: b.called,
                    muster: b.muster,
                    members: Some(members),
                    asked: Some(asked),
                    mustered: Some(mustered),
                    arrived: Some(arrived),
                    home: Some(home),
                    stage: b.stage.code(),
                    since: b.since,
                    clash: b.clash.unwrap_or(0),
                    incident: b.incident.unwrap_or(0),
                    law: b.law.map_or(0, PermanentId::get),
                    approach: b.approach.map(point).as_ref(),
                    held: b.held.unwrap_or(-1),
                    taken: Some(taken),
                    taken_kcal: b.taken_kcal,
                },
            )
        })
        .collect();
    let bands = fbb.create_vector(&bands);
    let clashes: Vec<_> = order
        .clashes
        .iter()
        .map(|c| {
            let fighters: Vec<save::FighterSave> = c
                .fighters
                .iter()
                .map(|f| {
                    save::FighterSave::new(
                        f.person.get(),
                        f.side.code(),
                        f.holds.code(),
                        f.will,
                        f.regard_leader,
                        f.gave_way.map_or(-1, i32::from),
                        f.hurt_days,
                        f.killed,
                    )
                })
                .collect();
            let fighters = fbb.create_vector(&fighters);
            let blows: Vec<save::StruckSave> = c
                .blows
                .iter()
                .map(|s| {
                    save::StruckSave::new(
                        s.minute,
                        s.harm.by.get(),
                        s.harm.to.get(),
                        s.harm.days,
                        s.harm.killed,
                    )
                })
                .collect();
            let blows = fbb.create_vector(&blows);
            save::ClashSave::create(
                &mut fbb,
                &save::ClashSaveArgs {
                    id: c.id,
                    at: c.at,
                    place: Some(&point(c.place)),
                    band: c.band,
                    fighters: Some(fighters),
                    blows: Some(blows),
                    frontage: c.frontage,
                    minutes: c.minutes,
                    ended: c.ended.code(),
                },
            )
        })
        .collect();
    let clashes = fbb.create_vector(&clashes);
    let amounts: Vec<save::AmountSave> = order
        .amounts
        .iter()
        .map(|a| save::AmountSave::new(a.household.get(), a.day, a.incident, a.kcal, a.lost))
        .collect();
    let amounts = fbb.create_vector(&amounts);
    // Loans in goods (schema 133, M7d slice CA).
    let credit = &order.credit;
    let loans: Vec<save::LoanSave> = credit
        .claims
        .iter()
        .map(|c| {
            save::LoanSave::new(
                c.id,
                c.creditor.get(),
                c.debtor.get(),
                c.good,
                c.standing.code(),
                c.lent,
                c.owed,
                c.paid,
                c.made,
                c.due,
                c.witnesses[0].get(),
                c.witnesses[1].get(),
            )
        })
        .collect();
    let loans = (!loans.is_empty()).then(|| fbb.create_vector(&loans));
    // Their pledges, answers and remedies (schema 134, M7d slice CB).
    let terms: Vec<save::LoanTermsSave> = credit
        .claims
        .iter()
        .map(|c| {
            let answer = match c.answer {
                None => 0,
                Some((true, _)) => 1,
                Some((false, _)) => 2,
            };
            save::LoanTermsSave::new(
                c.pledge.map_or(0, civ_agents::credit::Pledge::code),
                answer,
                c.asks,
                c.extensions,
                c.answer.map_or(0.0, |a| a.1),
                c.pledge.map_or(0, |p| p.names().get()),
                c.taken.unwrap_or(i64::MIN),
                c.by_guarantor,
                c.recourse.unwrap_or(0),
            )
        })
        .collect();
    let loan_terms = (!terms.is_empty()).then(|| fbb.create_vector(&terms));
    // Coin issues and purses (schema 136, M7d slice CE).
    let coinage = &order.coinage;
    let issues: Vec<save::CoinIssueSave> = coinage
        .cohorts
        .iter()
        .map(|c| {
            save::CoinIssueSave::new(c.id, c.issuer.get(), c.day, c.mass_g, c.fineness, c.count)
        })
        .collect();
    let purses: Vec<save::PurseSave> = coinage
        .purses
        .iter()
        .flat_map(|p| {
            p.counts
                .iter()
                .map(move |&(issue, n)| save::PurseSave::new(p.holder.get(), issue, n))
        })
        .collect();
    let coin_issues = (!issues.is_empty()).then(|| fbb.create_vector(&issues));
    let purses = (!purses.is_empty()).then(|| fbb.create_vector(&purses));
    // Metal brought to mints (schema 137, M7d slice CE2b).
    let brought: Vec<save::BroughtSave> = coinage
        .brought
        .iter()
        .map(|b| save::BroughtSave::new(b.polity.get(), b.household.get(), b.metal_kg))
        .collect();
    let brought = (!brought.is_empty()).then(|| fbb.create_vector(&brought));
    let root = save::Order::create(
        &mut fbb,
        &save::OrderArgs {
            incidents: Some(incidents),
            beliefs: Some(beliefs),
            responses: None,
            obligations: None,
            goods: Some(good_dictionary),
            amounts: Some(amounts),
            responses2: Some(responses),
            obligations2: Some(obligations),
            cases: Some(cases),
            sightings: Some(sightings),
            encounters: Some(encounters),
            bands: Some(bands),
            clashes: Some(clashes),
            loans,
            loans_next: credit.next,
            loans_asked: credit.asked,
            loans_refused: credit.refused,
            loan_terms,
            coin_issues,
            purses,
            brought,
            lamp_first_lit: order.lamp_first_lit.map_or(-1, |t| t.minutes()),
        },
    );
    finish(fbb, root)
}

/// Takings and what follows them, from a save. A good the loaded content no longer has is dropped
/// from what an incident records as carried off.
fn decode_order(bytes: &[u8], rules: &Rules) -> Result<civ_agents::crime::Order, LoadError> {
    use civ_agents::crime::{
        Belief, Case, CaseStage, Choice, Incident, Obligation, Order, Outcome, Owed, Response,
        Source, Standing,
    };
    let root = section_root::<save::Order>(bytes).map_err(|e| unreadable(SECTION_ORDER, &e))?;
    let goods = good_map(&read_strings(root.goods()), rules);
    let bad = |what: String| LoadError::Malformed(what);
    let mut order = Order::default();
    for i in root.incidents().iter().flatten() {
        let id = i.id();
        let outcome = Outcome::from_code(i.outcome())
            .ok_or_else(|| bad(format!("incident {id} has outcome code {}", i.outcome())))?;
        let mut taken = Vec::new();
        for g in i.goods().iter().flatten() {
            if let Some(good) =
                saved_good(&goods, u32::from(g.good()), || format!("incident {id}"))?
            {
                taken.push((good, g.amount()));
            }
        }
        let place = i.place().map_or((0.0, 0.0), |p| (p.x(), p.y()));
        order.incidents.push(Incident {
            id,
            at: time(i.at()),
            actor: required(i.actor(), "an incident's actor")?,
            actor_household: required(i.actor_household(), "an incident's household")?,
            target: required(i.target(), "an incident's target")?,
            place,
            outcome,
            goods: taken,
            kcal: i.kcal(),
            seen_by: i
                .seen_by()
                .map(|v| v.iter().filter_map(id_of).collect())
                .unwrap_or_default(),
            noticed: i.noticed(),
            food_days: i.food_days(),
            richer: i.richer(),
            across: id_of(i.across()),
        });
    }
    for b in root.beliefs().iter().flatten() {
        order.beliefs.push(Belief {
            holder: required(b.holder(), "a belief's holder")?,
            incident: b.incident(),
            taker: id(b.taker()),
            source: Source::from_code(b.source())
                .ok_or_else(|| bad(format!("a belief has source code {}", b.source())))?,
            from: id(b.from()),
            origin: id(b.origin()),
            day: b.day(),
        });
    }
    // Saves 31 knew only letting go and demanding back.
    for r in root.responses().iter().flatten() {
        order.responses.push(Response {
            incident: r.incident(),
            household: required(r.household(), "a response's household")?,
            by: required(r.by(), "a response's chooser")?,
            day: r.day(),
            choice: if r.demand() {
                Choice::Demand
            } else {
                Choice::LetGo
            },
            points: r.points(),
            report_points: None,
        });
    }
    for r in root.responses2().iter().flatten() {
        order.responses.push(Response {
            incident: r.incident(),
            household: required(r.household(), "a response's household")?,
            by: required(r.by(), "a response's chooser")?,
            day: r.day(),
            choice: Choice::from_code(r.choice())
                .ok_or_else(|| bad(format!("a response has choice code {}", r.choice())))?,
            points: r.points(),
            report_points: r.report_open().then_some(r.report_points()),
        });
    }
    let answer = |oid: u32, code: u8, points: f32| match code {
        0 => Ok(None),
        1 => Ok(Some((true, points))),
        2 => Ok(Some((false, points))),
        c => Err(bad(format!("obligation {oid} has answer code {c}"))),
    };
    let standing = |oid: u32, code: u8| {
        Standing::from_code(code)
            .ok_or_else(|| bad(format!("obligation {oid} has standing code {code}")))
    };
    for o in root.obligations().iter().flatten() {
        let oid = o.id();
        order.obligations.push(Obligation {
            id: oid,
            incident: o.incident(),
            kind: Owed::Demanded,
            case: None,
            debtor: required(o.debtor(), "an obligation's debtor")?,
            beneficiary: required(o.beneficiary(), "an obligation's beneficiary")?,
            kcal: o.kcal(),
            paid_kcal: o.paid_kcal(),
            made: o.made(),
            due: o.due(),
            standing: standing(oid, o.standing())?,
            answer: answer(oid, o.answer(), o.answer_points())?,
        });
    }
    for o in root.obligations2().iter().flatten() {
        let oid = o.id();
        order.obligations.push(Obligation {
            id: oid,
            incident: o.incident(),
            kind: Owed::from_code(o.kind())
                .ok_or_else(|| bad(format!("obligation {oid} has kind code {}", o.kind())))?,
            case: (o.case_id() != 0).then_some(o.case_id()),
            debtor: required(o.debtor(), "an obligation's debtor")?,
            beneficiary: required(o.beneficiary(), "an obligation's beneficiary")?,
            kcal: o.kcal(),
            paid_kcal: o.paid_kcal(),
            made: o.made(),
            due: o.due(),
            standing: standing(oid, o.standing())?,
            answer: answer(oid, o.answer(), o.answer_points())?,
        });
    }
    for c in root.cases().iter().flatten() {
        let cid = c.id();
        let mut stances = Vec::new();
        let precedent: Vec<f32> = c
            .stance_precedent()
            .map(|v| v.iter().collect())
            .unwrap_or_default();
        for (i, r) in c.stances().iter().flatten().enumerate() {
            stances.push(civ_agents::crime::CaseStance {
                person: required(r.person(), "someone at a hearing")?,
                household: required(r.household(), "a household at a hearing")?,
                stance: stance_from(r.stance())
                    .ok_or_else(|| bad(format!("case {cid} has stance code {}", r.stance())))?,
                stake: r.stake(),
                belief: r.belief(),
                regard: r.regard(),
                precedent: precedent.get(i).copied().unwrap_or(0.0),
            });
        }
        order.cases.push(Case {
            id: cid,
            incident: c.incident(),
            settlement: required(c.settlement(), "a case's settlement")?,
            law: required(c.law(), "a case's law")?,
            accuser: required(c.accuser(), "a case's accuser")?,
            by: required(c.by(), "the one who brought a case")?,
            accused: required(c.accused(), "a case's accused")?,
            accused_household: required(c.accused_household(), "the accused's household")?,
            kcal: c.kcal(),
            leads: c
                .leads()
                .map(|v| v.iter().filter_map(id_of).collect())
                .unwrap_or_default(),
            opened: c.opened(),
            stage: CaseStage::from_code(c.stage())
                .ok_or_else(|| bad(format!("case {cid} has stage code {}", c.stage())))?,
            heard: c.heard().then(|| time(c.heard_at())),
            eligible: c.eligible(),
            stances,
            exiled: c.exiled(),
            forum: id_of(c.forum()),
            challenge: id_of(c.challenge()),
            loan: (c.loan() != 0).then_some(c.loan()),
            // Schema 160 (M8c slice CV); none before.
            building: PermanentId::from_raw(c.building()),
            tags: c.tags(),
            departed: (c.departed() != 0).then_some(c.departed()),
        });
    }
    for a in root.amounts().iter().flatten() {
        order.amounts.push(civ_agents::crime::KnownAmount {
            household: required(a.household(), "a known amount's household")?,
            incident: a.incident(),
            kcal: a.kcal(),
            lost: a.lost(),
            day: a.day(),
        });
    }
    for s in root.sightings().iter().flatten() {
        order.sightings.push(civ_agents::crime::Sighting {
            incident: s.incident(),
            officer: required(s.officer(), "a sighting's watcher")?,
            day: s.day(),
            kept: civ_agents::crime::Kept::from_code(s.kept())
                .ok_or_else(|| bad(format!("a sighting has choice code {}", s.kept())))?,
            kcal: s.kcal(),
            points: (s.report_points(), s.ask_open().then_some(s.ask_points())),
        });
    }
    for e in root.encounters().iter().flatten() {
        let mut met = Vec::new();
        for m in e.met().iter().flatten() {
            let how = civ_agents::crime::Met::from_code(m.met())
                .ok_or_else(|| bad(format!("encounter {} has answer code {}", e.id(), m.met())))?;
            met.push((required(m.person(), "one met in an encounter")?, how));
        }
        let mut harms = Vec::new();
        for h in e.harms().iter().flatten() {
            harms.push(civ_agents::crime::Harm {
                by: required(h.by(), "one who struck")?,
                to: required(h.to(), "one struck")?,
                days: h.days(),
                killed: h.killed(),
            });
        }
        order.encounters.push(civ_agents::crime::Encounter {
            id: e.id(),
            day: e.day(),
            officer: required(e.officer(), "an encounter's watcher")?,
            household: required(e.household(), "an encounter's household")?,
            obligation: e.obligation(),
            owed_kcal: e.owed_kcal(),
            met,
            forced: e.forced(),
            taken_kcal: e.taken_kcal(),
            harms,
        });
    }
    for b in root.bands().iter().flatten() {
        let id = b.id();
        let list = |v: Option<flatbuffers::Vector<'_, u64>>| -> Vec<PermanentId> {
            v.map(|v| v.iter().filter_map(id_of).collect())
                .unwrap_or_default()
        };
        let mut taken = Vec::new();
        for g in b.taken().iter().flatten() {
            if let Some(good) = saved_good(&goods, u32::from(g.good()), || format!("band {id}"))? {
                taken.push((good, g.amount()));
            }
        }
        order.bands.push(civ_agents::war::Band {
            id,
            organizer: required(b.organizer(), "a band's organizer")?,
            settlement: required(b.settlement(), "a band's settlement")?,
            purpose: civ_agents::war::BandPurpose::from_code(b.purpose())
                .ok_or_else(|| bad(format!("band {id} has purpose code {}", b.purpose())))?,
            target: required(b.target(), "a band's target")?,
            place: b.place().map_or((0.0, 0.0), |p| (p.x(), p.y())),
            called: b.called(),
            muster: b.muster(),
            members: list(b.members()),
            asked: list(b.asked()),
            mustered: list(b.mustered()),
            arrived: list(b.arrived()),
            home: list(b.home()),
            stage: civ_agents::war::BandStage::from_code(b.stage())
                .ok_or_else(|| bad(format!("band {id} has stage code {}", b.stage())))?,
            since: b.since(),
            clash: (b.clash() != 0).then_some(b.clash()),
            incident: (b.incident() != 0).then_some(b.incident()),
            law: PermanentId::from_raw(b.law()),
            taken,
            taken_kcal: b.taken_kcal(),
            approach: b.approach().map(|p| (p.x(), p.y())),
            held: (b.held() >= 0).then_some(b.held()),
        });
    }
    for c in root.clashes().iter().flatten() {
        let id = c.id();
        let mut fighters = Vec::new();
        for f in c.fighters().iter().flatten() {
            fighters.push(civ_agents::war::Fighter {
                person: required(f.person(), "one in an encounter")?,
                side: civ_agents::war::Side::from_code(f.side())
                    .ok_or_else(|| bad(format!("encounter {id} has side code {}", f.side())))?,
                holds: civ_agents::war::Holding::from_code(f.holds())
                    .ok_or_else(|| bad(format!("encounter {id} has holding code {}", f.holds())))?,
                will: f.will(),
                regard_leader: f.regard_leader(),
                gave_way: u16::try_from(f.gave_way()).ok(),
                hurt_days: f.hurt_days(),
                killed: f.killed(),
            });
        }
        let mut blows = Vec::new();
        for s in c.blows().iter().flatten() {
            blows.push(civ_agents::war::Struck {
                minute: s.minute(),
                harm: civ_agents::crime::Harm {
                    by: required(s.by(), "one who struck")?,
                    to: required(s.to(), "one struck")?,
                    days: s.days(),
                    killed: s.killed(),
                },
            });
        }
        order.clashes.push(civ_agents::war::Clash {
            id,
            at: c.at(),
            place: c.place().map_or((0.0, 0.0), |p| (p.x(), p.y())),
            band: c.band(),
            fighters,
            blows,
            frontage: c.frontage(),
            minutes: c.minutes(),
            ended: civ_agents::war::Ended::from_code(c.ended())
                .ok_or_else(|| bad(format!("encounter {id} has ending code {}", c.ended())))?,
        });
    }
    // Loans in goods (schema 133, M7d slice CA); a loan of a good the content no longer has is
    // dropped.
    let terms: Vec<&save::LoanTermsSave> = root.loan_terms().iter().flatten().collect();
    if !terms.is_empty() && terms.len() != root.loans().map_or(0, |l| l.len()) {
        return Err(bad(format!(
            "{} loans and {} loan terms",
            root.loans().map_or(0, |l| l.len()),
            terms.len()
        )));
    }
    for (k, l) in root.loans().iter().flatten().enumerate() {
        let id = l.id();
        let t = terms.get(k);
        let Some(good) = saved_good(&goods, u32::from(l.good()), || format!("loan {id}"))? else {
            continue;
        };
        order.credit.claims.push(civ_agents::credit::Claim {
            id,
            creditor: required(l.creditor(), "a loan's creditor")?,
            debtor: required(l.debtor(), "a loan's debtor")?,
            good,
            lent: l.lent(),
            owed: l.owed(),
            paid: l.paid(),
            made: l.made(),
            due: l.due(),
            standing: civ_agents::credit::Standing::from_code(l.standing())
                .ok_or_else(|| bad(format!("loan {id} has standing code {}", l.standing())))?,
            witnesses: [
                required(l.witness_a(), "a loan's witness")?,
                required(l.witness_b(), "a loan's witness")?,
            ],
            pledge: match t.map_or(0, |t| t.pledge_kind()) {
                0 => None,
                code => Some(
                    civ_agents::credit::Pledge::from_code(
                        code,
                        PermanentId::from_raw(t.map_or(0, |t| t.pledge())),
                    )
                    .ok_or_else(|| bad(format!("loan {id} has a pledge of code {code}")))?,
                ),
            },
            answer: match t.map_or(0, |t| t.answer()) {
                0 => None,
                1 => Some((true, t.map_or(0.0, |t| t.points()))),
                2 => Some((false, t.map_or(0.0, |t| t.points()))),
                code => return Err(bad(format!("loan {id} has answer code {code}"))),
            },
            asks: t.map_or(0, |t| t.asks()),
            extensions: t.map_or(0, |t| t.extensions()),
            taken: t.map(|t| t.taken()).filter(|&d| d != i64::MIN),
            by_guarantor: t.map_or(0.0, |t| t.by_guarantor()),
            recourse: t.map(|t| t.recourse()).filter(|&r| r != 0),
        });
    }
    order.credit.next = root.loans_next();
    order.credit.asked = root.loans_asked();
    order.credit.refused = root.loans_refused();
    // The world's first lamp lit (schema 179); none before.
    order.lamp_first_lit = Some(root.lamp_first_lit())
        .filter(|&m| m >= 0)
        .map(civ_core::time::SimTime::from_minutes);
    // Coin issues and purses (schema 136); none before.
    for c in root.coin_issues().iter().flatten() {
        order.coinage.cohorts.push(civ_agents::coin::Cohort {
            id: c.id(),
            issuer: required(c.issuer(), "a coin issue's issuer")?,
            day: c.day(),
            mass_g: c.mass_g(),
            fineness: c.fineness(),
            count: c.count(),
        });
    }
    for p in root.purses().iter().flatten() {
        let holder = required(p.holder(), "a purse's holder")?;
        match order.coinage.purses.last_mut() {
            Some(last) if last.holder == holder => last.counts.push((p.issue(), p.count())),
            _ => order.coinage.purses.push(civ_agents::coin::Purse {
                holder,
                counts: vec![(p.issue(), p.count())],
            }),
        }
    }
    // Metal brought to mints (schema 137); none before.
    for b in root.brought().iter().flatten() {
        order.coinage.brought.push(civ_agents::coin::Brought {
            polity: required(b.polity(), "metal brought's polity")?,
            household: required(b.household(), "metal brought's household")?,
            metal_kg: b.metal_kg(),
        });
    }
    let problems = order_problems(&order);
    if !problems.is_empty() {
        return Err(LoadError::Invalid(problems));
    }
    Ok(order)
}

fn id_of(raw: u64) -> Option<PermanentId> {
    PermanentId::from_raw(raw)
}

/// What is wrong with a save's takings: records out of order, or beliefs, responses and
/// obligations that name an incident there is no record of.
fn order_problems(order: &civ_agents::crime::Order) -> Vec<String> {
    let mut out = Vec::new();
    if order.incidents.windows(2).any(|w| w[0].id >= w[1].id) {
        out.push("incidents are out of order".to_owned());
    }
    if order
        .beliefs
        .windows(2)
        .any(|w| (w[0].holder, w[0].incident) >= (w[1].holder, w[1].incident))
    {
        out.push("beliefs are out of order".to_owned());
    }
    if order.obligations.windows(2).any(|w| w[0].id >= w[1].id) {
        out.push("obligations are out of order".to_owned());
    }
    let known = |i: u32| order.incident(i).is_some();
    if let Some(b) = order.beliefs.iter().find(|b| !known(b.incident)) {
        out.push(format!(
            "a belief of {} names missing incident {}",
            b.holder, b.incident
        ));
    }
    if let Some(r) = order.responses.iter().find(|r| !known(r.incident)) {
        out.push(format!("a response names missing incident {}", r.incident));
    }
    // A debt imposed by a finding names its loan, not an incident (M7d slice CB); ground rent
    // names none (M8c slice CU).
    if let Some(o) = order.obligations.iter().find(|o| match o.kind {
        civ_agents::crime::Owed::Debt => order.credit.claim(o.incident).is_none(),
        civ_agents::crime::Owed::GroundRent => o.incident != 0,
        // A breach's finding (M7c slice BX) names no incident, but its case.
        _ if o.incident == 0 => o.case.is_none(),
        _ => !known(o.incident),
    }) {
        out.push(format!(
            "obligation {} names missing incident {}",
            o.id, o.incident
        ));
    }
    if let Some(a) = order.amounts.iter().find(|a| !known(a.incident)) {
        out.push(format!(
            "an amount known to household {} names missing incident {}",
            a.household, a.incident
        ));
    }
    if order.amounts.windows(2).any(|w| w[0].day > w[1].day) {
        out.push("known amounts are out of day order".to_owned());
    }
    if order.cases.windows(2).any(|w| w[0].id >= w[1].id) {
        out.push("cases are out of order".to_owned());
    }
    if let Some(s) = order.sightings.iter().find(|s| !known(s.incident)) {
        out.push(format!(
            "a sighting by {} names missing incident {}",
            s.officer, s.incident
        ));
    }
    if let Some(c) = order
        .cases
        .iter()
        // A challenge, a breach of a privilege or a debt names no incident (0).
        .find(|c| c.incident != 0 && !known(c.incident))
    {
        out.push(format!(
            "case {} names missing incident {}",
            c.id, c.incident
        ));
    }
    if let Some(o) = order
        .obligations
        .iter()
        .find(|o| o.case.is_some_and(|c| order.case(c).is_none()))
    {
        out.push(format!("obligation {} names a missing case", o.id));
    }
    if let Some(c) = order
        .cases
        .iter()
        .find(|c| c.stances.windows(2).any(|w| w[0].person >= w[1].person))
    {
        out.push(format!("case {}'s stances are out of order", c.id));
    }
    out
}

// ---- history -----------------------------------------------------------------------------------

fn chronicle_code(kind: ChronicleKind) -> u16 {
    match kind {
        ChronicleKind::BandArrived => 1,
        ChronicleKind::SettlementFounded => 2,
        ChronicleKind::FoodRanShort => 3,
        ChronicleKind::FoodRecovered => 4,
        ChronicleKind::FirstSowing => 5,
        ChronicleKind::HarvestIn => 6,
        ChronicleKind::FirstRoof => 7,
        ChronicleKind::Born => 8,
        ChronicleKind::Died => 9,
        ChronicleKind::Paired => 10,
        ChronicleKind::TakenIn => 11,
        ChronicleKind::Left => 12,
        ChronicleKind::FirstTrail => 13,
        ChronicleKind::FamilyArrived => 14,
        ChronicleKind::WorkshopOpened => 15,
        ChronicleKind::WorkshopClosed => 16,
        ChronicleKind::TechniqueFound => 17,
        ChronicleKind::TechniqueLearned => 18,
        ChronicleKind::TechniqueLost => 19,
        ChronicleKind::TechniqueIntroduced => 20,
        ChronicleKind::BuildingFailed => 21,
        ChronicleKind::DepositFound => 22,
        ChronicleKind::DepositPlaced => 23,
        ChronicleKind::Weather => 24,
        ChronicleKind::LawProposed => 25,
        ChronicleKind::LawDecided => 26,
        ChronicleKind::LawLapsed => 27,
        ChronicleKind::Taking => 28,
        ChronicleKind::Restitution => 29,
        ChronicleKind::CaseBrought => 30,
        ChronicleKind::CaseHeard => 31,
        ChronicleKind::CustomAmended => 32,
        ChronicleKind::RevoltCalled => 33,
        ChronicleKind::CustomTaken => 34,
        ChronicleKind::RevoltFailed => 35,
        ChronicleKind::CoupCalled => 36,
        ChronicleKind::CoupFailed => 37,
        ChronicleKind::Encounter => 38,
        ChronicleKind::Influence => 39,
        ChronicleKind::InfluenceTurned => 40,
        ChronicleKind::Moved => 41,
        ChronicleKind::Coalition => 42,
        ChronicleKind::Agreement => 43,
        ChronicleKind::Crossing => 44,
        ChronicleKind::Well => 45,
        ChronicleKind::Outbreak => 46,
        ChronicleKind::Suspicion => 47,
        ChronicleKind::Fire => 48,
        ChronicleKind::Flood => 49,
        ChronicleKind::StormDamage => 50,
        ChronicleKind::Ditch => 51,
        ChronicleKind::Raid => 52,
        ChronicleKind::Fled => 53,
        ChronicleKind::Enclosure => 54,
        ChronicleKind::PolityJoined => 55,
        ChronicleKind::Seceded => 56,
        ChronicleKind::LotDrawn => 57,
        ChronicleKind::ElectionCalled => 58,
        ChronicleKind::ElectionHeld => 59,
        ChronicleKind::AllianceMade => 60,
        ChronicleKind::CoinStruck => 61,
        ChronicleKind::BoatMade => 62,
        ChronicleKind::CartMade => 63,
        ChronicleKind::TakenDown => 64,
        ChronicleKind::LineLaid => 65,
        ChronicleKind::StationsOpened => 66,
        ChronicleKind::RecordWritten => 67,
        ChronicleKind::RecordLost => 68,
    }
}

fn chronicle_kind(code: u16) -> Option<ChronicleKind> {
    match code {
        1 => Some(ChronicleKind::BandArrived),
        2 => Some(ChronicleKind::SettlementFounded),
        3 => Some(ChronicleKind::FoodRanShort),
        4 => Some(ChronicleKind::FoodRecovered),
        5 => Some(ChronicleKind::FirstSowing),
        6 => Some(ChronicleKind::HarvestIn),
        7 => Some(ChronicleKind::FirstRoof),
        8 => Some(ChronicleKind::Born),
        9 => Some(ChronicleKind::Died),
        10 => Some(ChronicleKind::Paired),
        11 => Some(ChronicleKind::TakenIn),
        12 => Some(ChronicleKind::Left),
        13 => Some(ChronicleKind::FirstTrail),
        14 => Some(ChronicleKind::FamilyArrived),
        15 => Some(ChronicleKind::WorkshopOpened),
        16 => Some(ChronicleKind::WorkshopClosed),
        17 => Some(ChronicleKind::TechniqueFound),
        18 => Some(ChronicleKind::TechniqueLearned),
        19 => Some(ChronicleKind::TechniqueLost),
        20 => Some(ChronicleKind::TechniqueIntroduced),
        21 => Some(ChronicleKind::BuildingFailed),
        22 => Some(ChronicleKind::DepositFound),
        23 => Some(ChronicleKind::DepositPlaced),
        24 => Some(ChronicleKind::Weather),
        25 => Some(ChronicleKind::LawProposed),
        26 => Some(ChronicleKind::LawDecided),
        27 => Some(ChronicleKind::LawLapsed),
        28 => Some(ChronicleKind::Taking),
        29 => Some(ChronicleKind::Restitution),
        30 => Some(ChronicleKind::CaseBrought),
        31 => Some(ChronicleKind::CaseHeard),
        32 => Some(ChronicleKind::CustomAmended),
        33 => Some(ChronicleKind::RevoltCalled),
        34 => Some(ChronicleKind::CustomTaken),
        35 => Some(ChronicleKind::RevoltFailed),
        36 => Some(ChronicleKind::CoupCalled),
        37 => Some(ChronicleKind::CoupFailed),
        38 => Some(ChronicleKind::Encounter),
        39 => Some(ChronicleKind::Influence),
        40 => Some(ChronicleKind::InfluenceTurned),
        41 => Some(ChronicleKind::Moved),
        42 => Some(ChronicleKind::Coalition),
        43 => Some(ChronicleKind::Agreement),
        44 => Some(ChronicleKind::Crossing),
        45 => Some(ChronicleKind::Well),
        46 => Some(ChronicleKind::Outbreak),
        47 => Some(ChronicleKind::Suspicion),
        48 => Some(ChronicleKind::Fire),
        49 => Some(ChronicleKind::Flood),
        50 => Some(ChronicleKind::StormDamage),
        51 => Some(ChronicleKind::Ditch),
        52 => Some(ChronicleKind::Raid),
        53 => Some(ChronicleKind::Fled),
        54 => Some(ChronicleKind::Enclosure),
        55 => Some(ChronicleKind::PolityJoined),
        56 => Some(ChronicleKind::Seceded),
        57 => Some(ChronicleKind::LotDrawn),
        58 => Some(ChronicleKind::ElectionCalled),
        59 => Some(ChronicleKind::ElectionHeld),
        60 => Some(ChronicleKind::AllianceMade),
        61 => Some(ChronicleKind::CoinStruck),
        62 => Some(ChronicleKind::BoatMade),
        63 => Some(ChronicleKind::CartMade),
        64 => Some(ChronicleKind::TakenDown),
        65 => Some(ChronicleKind::LineLaid),
        66 => Some(ChronicleKind::StationsOpened),
        67 => Some(ChronicleKind::RecordWritten),
        68 => Some(ChronicleKind::RecordLost),
        _ => None,
    }
}

fn encode_history(pop: &Population) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let records: Vec<_> = pop
        .records
        .values()
        .map(|r| {
            let given = fbb.create_string(&r.given);
            let (died, died_at, cause) = match r.died {
                Some((at, cause)) => (
                    true,
                    at.minutes(),
                    match cause {
                        Cause::Unspecified => save::Cause::Unspecified,
                        Cause::Starvation => save::Cause::Starvation,
                        Cause::Childbirth => save::Cause::Childbirth,
                        Cause::Collapse => save::Cause::Collapse,
                        Cause::Violence => save::Cause::Violence,
                        Cause::Fell => save::Cause::Fell,
                        Cause::Disease => save::Cause::Disease,
                        Cause::Burned => save::Cause::Burned,
                    },
                ),
                None => (false, 0, save::Cause::Unspecified),
            };
            let stays: Vec<_> = r
                .residence
                .iter()
                .map(|x| {
                    save::Stay::create(
                        &mut fbb,
                        &save::StayArgs {
                            settlement: raw(x.settlement),
                            since: x.since.minutes(),
                            why: x.why.code(),
                        },
                    )
                })
                .collect();
            let residence = fbb.create_vector(&stays);
            save::PersonRecord::create(
                &mut fbb,
                &save::PersonRecordArgs {
                    id: r.id.get(),
                    given: Some(given),
                    sex: sex_parts(r.sex),
                    born: r.born.minutes(),
                    died,
                    died_at,
                    cause,
                    mother: raw(r.mother),
                    father: raw(r.father),
                    left: r.left.is_some(),
                    left_at: r.left.map_or(0, SimTime::minutes),
                    origin: match r.origin {
                        Origin::Founder => save::Origin::Founder,
                        Origin::Born => save::Origin::Born,
                        Origin::Spawned => save::Origin::Spawned,
                    },
                    residence: Some(residence),
                },
            )
        })
        .collect();
    let records = fbb.create_vector(&records);
    let chronicle: Vec<_> = pop
        .chronicle
        .iter()
        .map(|e| {
            let people: Vec<u64> = e.people.iter().map(|p| p.get()).collect();
            let people = fbb.create_vector(&people);
            let name = fbb.create_string(&e.name);
            save::ChronicleEntry::create(
                &mut fbb,
                &save::ChronicleEntryArgs {
                    seq: e.seq,
                    at: e.at.minutes(),
                    kind: chronicle_code(e.kind),
                    people: Some(people),
                    settlement: raw(e.settlement),
                    has_place: e.place.is_some(),
                    place: Some(&point(e.place.unwrap_or((0.0, 0.0)))),
                    number: e.number,
                    name: Some(name),
                    firm: raw(e.firm),
                },
            )
        })
        .collect();
    let chronicle = fbb.create_vector(&chronicle);
    let unions: Vec<_> = pop
        .unions
        .iter()
        .map(|u| {
            save::Union::create(
                &mut fbb,
                &save::UnionArgs {
                    woman: u.woman.get(),
                    man: u.man.get(),
                    since: u.since.minutes(),
                    ended: u.ended.is_some(),
                    ended_at: u.ended.map_or(0, SimTime::minutes),
                },
            )
        })
        .collect();
    let unions = fbb.create_vector(&unions);
    let root = save::History::create(
        &mut fbb,
        &save::HistoryArgs {
            records: Some(records),
            chronicle: Some(chronicle),
            unions: Some(unions),
        },
    );
    finish(fbb, root)
}

type DecodedHistory = (
    BTreeMap<PermanentId, PersonRecord>,
    Vec<ChronicleEvent>,
    Vec<Union>,
);

fn decode_history(bytes: &[u8]) -> Result<DecodedHistory, LoadError> {
    let root = section_root::<save::History>(bytes).map_err(|e| unreadable(SECTION_HISTORY, &e))?;
    let mut records = BTreeMap::new();
    for r in root.records().iter().flatten() {
        let rid = required(r.id(), "a person record")?;
        let cause = match r.cause() {
            save::Cause::Unspecified => Cause::Unspecified,
            save::Cause::Starvation => Cause::Starvation,
            save::Cause::Childbirth => Cause::Childbirth,
            save::Cause::Collapse => Cause::Collapse,
            save::Cause::Violence => Cause::Violence,
            save::Cause::Fell => Cause::Fell,
            save::Cause::Disease => Cause::Disease,
            save::Cause::Burned => Cause::Burned,
            other => {
                return Err(LoadError::Incompatible(format!(
                    "record {rid} has cause of death {}, which this build does not know",
                    other.0
                )));
            }
        };
        let origin = match r.origin() {
            save::Origin::Founder => Origin::Founder,
            save::Origin::Born => Origin::Born,
            save::Origin::Spawned => Origin::Spawned,
            other => {
                return Err(LoadError::Incompatible(format!(
                    "record {rid} has origin {}, which this build does not know",
                    other.0
                )));
            }
        };
        let mut residence = Vec::new();
        for x in r.residence().iter().flatten() {
            residence.push(Stay {
                settlement: id(x.settlement()),
                since: time(x.since()),
                why: ResidenceWhy::from_code(x.why()).ok_or_else(|| {
                    LoadError::Incompatible(format!(
                        "record {rid} moved for reason {}, which this build does not know",
                        x.why()
                    ))
                })?,
            });
        }
        records.insert(
            rid,
            PersonRecord {
                id: rid,
                given: r.given().unwrap_or_default().to_owned(),
                sex: sex_of(r.sex())?,
                born: time(r.born()),
                died: r.died().then(|| (time(r.died_at()), cause)),
                left: r.left().then(|| time(r.left_at())),
                mother: id(r.mother()),
                father: id(r.father()),
                origin,
                residence,
            },
        );
    }
    let mut chronicle = Vec::new();
    for e in root.chronicle().iter().flatten() {
        let kind = chronicle_kind(e.kind()).ok_or_else(|| {
            LoadError::Incompatible(format!(
                "chronicle entry {} is of kind {}, which this build does not know",
                e.seq(),
                e.kind()
            ))
        })?;
        let mut people = Vec::new();
        for p in e.people().iter().flatten() {
            people.push(required(p, "a chronicle participant")?);
        }
        chronicle.push(ChronicleEvent {
            seq: e.seq(),
            at: time(e.at()),
            kind,
            people,
            settlement: id(e.settlement()),
            place: e.has_place().then(|| xy(e.place())),
            number: e.number(),
            name: e.name().unwrap_or_default().to_owned(),
            firm: id(e.firm()),
        });
    }
    let mut unions = Vec::new();
    for u in root.unions().iter().flatten() {
        unions.push(Union {
            woman: required(u.woman(), "a union's woman")?,
            man: required(u.man(), "a union's man")?,
            since: time(u.since()),
            ended: u.ended().then(|| time(u.ended_at())),
        });
    }
    Ok((records, chronicle, unions))
}

// ---- receipts ----------------------------------------------------------------------------------

fn encode_scored<'a>(fbb: &mut FlatBufferBuilder<'a>, s: &Scored) -> WIPOffset<save::Scored<'a>> {
    let terms: Vec<save::Term> = s
        .terms
        .iter()
        .map(|t| save::Term::new(t.reason as u16, t.points))
        .collect();
    let terms = fbb.create_vector(&terms);
    let (target_kind, target_index, target_id) = target_parts(s.target);
    save::Scored::create(
        fbb,
        &save::ScoredArgs {
            def: u32::from(s.def),
            target_kind,
            target_index,
            target_id,
            total: s.total,
            terms: Some(terms),
        },
    )
}

fn encode_receipt<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    r: &Receipt,
) -> WIPOffset<save::Receipt<'a>> {
    let chosen = encode_scored(fbb, &r.chosen);
    let runner_up = r.runner_up.as_ref().map(|s| encode_scored(fbb, s));
    let others: Vec<save::OtherOption> = r
        .others
        .iter()
        .map(|&(def, total)| save::OtherOption::new(u32::from(def), total))
        .collect();
    let others = fbb.create_vector(&others);
    let excluded: Vec<save::Exclusion> = r
        .excluded
        .iter()
        .map(|&(def, reason)| save::Exclusion::new(u32::from(def), reason as u16))
        .collect();
    let excluded = fbb.create_vector(&excluded);
    let needs = fbb.create_vector(&r.needs);
    save::Receipt::create(
        fbb,
        &save::ReceiptArgs {
            at: r.at.minutes(),
            chosen: Some(chosen),
            runner_up,
            others: Some(others),
            excluded: Some(excluded),
            probability: r.probability,
            temperature: r.temperature,
            needs: Some(needs),
        },
    )
}

fn encode_receipts(pop: &Population, activities: &[&str]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let dictionary = strings(&mut fbb, activities);
    let people: Vec<_> = sorted_people(pop)
        .into_iter()
        .map(|p| {
            let receipts: Vec<_> = p
                .receipts
                .iter()
                .map(|r| encode_receipt(&mut fbb, r))
                .collect();
            let receipts = fbb.create_vector(&receipts);
            save::PersonReceipts::create(
                &mut fbb,
                &save::PersonReceiptsArgs {
                    person: p.id.get(),
                    receipts: Some(receipts),
                },
            )
        })
        .collect();
    let people = fbb.create_vector(&people);
    let root = save::Receipts::create(
        &mut fbb,
        &save::ReceiptsArgs {
            activities: Some(dictionary),
            people: Some(people),
        },
    );
    finish(fbb, root)
}

fn reason(code: u16) -> Result<Reason, LoadError> {
    Reason::from_code(code).ok_or_else(|| {
        LoadError::Incompatible(format!(
            "a decision receipt gives reason {code}, which this build does not know"
        ))
    })
}

/// A receipt's activity in the loaded catalog, or [`UNKNOWN_ACTIVITY`].
fn def_of(map: &[Option<u16>], saved: u32) -> Result<u16, LoadError> {
    match map.get(saved as usize) {
        Some(def) => Ok(def.unwrap_or(UNKNOWN_ACTIVITY)),
        None => Err(LoadError::Malformed(format!(
            "a decision receipt names activity {saved} of {}",
            map.len()
        ))),
    }
}

fn decode_scored(
    s: &save::Scored<'_>,
    map: &[Option<u16>],
    hearth: Option<PermanentId>,
) -> Result<Scored, LoadError> {
    let mut terms = Vec::new();
    for t in s.terms().iter().flatten() {
        terms.push(Term {
            reason: reason(t.reason())?,
            points: t.points(),
        });
    }
    Ok(Scored {
        def: def_of(map, s.def())?,
        target: target_of(s.target_kind(), s.target_index(), s.target_id(), hearth)?,
        total: s.total(),
        terms,
    })
}

fn decode_receipt(
    r: &save::Receipt<'_>,
    map: &[Option<u16>],
    hearth: Option<PermanentId>,
) -> Result<Receipt, LoadError> {
    let chosen = r
        .chosen()
        .ok_or_else(|| LoadError::Malformed("a decision receipt has no choice".to_owned()))?;
    let needs: Vec<f32> = r.needs().map(|v| v.iter().collect()).unwrap_or_default();
    let needs: [f32; 5] = needs
        .try_into()
        .map_err(|_| LoadError::Malformed("a decision receipt lacks its five needs".to_owned()))?;
    let mut others = Vec::new();
    for o in r.others().iter().flatten() {
        others.push((def_of(map, o.def())?, o.total()));
    }
    let mut excluded = Vec::new();
    for x in r.excluded().iter().flatten() {
        excluded.push((def_of(map, x.def())?, reason(x.reason())?));
    }
    Ok(Receipt {
        at: time(r.at()),
        chosen: decode_scored(&chosen, map, hearth)?,
        runner_up: r
            .runner_up()
            .map(|s| decode_scored(&s, map, hearth))
            .transpose()?,
        others,
        excluded,
        probability: r.probability(),
        temperature: r.temperature(),
        needs,
    })
}

fn decode_receipts(bytes: &[u8], rules: &Rules, pop: &mut Population) -> Result<(), LoadError> {
    let root =
        section_root::<save::Receipts>(bytes).map_err(|e| unreadable(SECTION_RECEIPTS, &e))?;
    let map = activity_map(&read_strings(root.activities()), rules);
    let handles: HashMap<PermanentId, _> = pop.people.iter().map(|(h, p)| (p.id, h)).collect();
    for entry in root.people().iter().flatten() {
        let who = required(entry.person(), "a receipt's person")?;
        let Some(&h) = handles.get(&who) else {
            return Err(LoadError::Invalid(vec![format!(
                "receipts are kept for {who}, who is not alive"
            )]));
        };
        // A hearth weighed before schema 50 was their own settlement's.
        let hearth = pop
            .person(who)
            .and_then(|p| pop.household(p.household))
            .and_then(|x| x.settlement);
        let mut receipts = VecDeque::new();
        for r in entry.receipts().iter().flatten() {
            receipts.push_back(decode_receipt(&r, &map, hearth)?);
        }
        if receipts.len() > civ_agents::person::RECEIPT_RING {
            return Err(LoadError::Invalid(vec![format!(
                "{who} has {} receipts; at most {} are kept",
                receipts.len(),
                civ_agents::person::RECEIPT_RING
            )]));
        }
        if let Some(p) = pop.people.get_mut(h) {
            p.receipts = receipts;
        }
    }
    Ok(())
}

// ---- fields ------------------------------------------------------------------------------------

fn stage_parts(s: FieldStage) -> save::FieldStage {
    match s {
        FieldStage::Fallow => save::FieldStage::Fallow,
        FieldStage::Prepared => save::FieldStage::Prepared,
        FieldStage::Sown => save::FieldStage::Sown,
        FieldStage::Reaped => save::FieldStage::Reaped,
    }
}

fn stage_of(s: save::FieldStage) -> Result<FieldStage, LoadError> {
    match s {
        save::FieldStage::Fallow => Ok(FieldStage::Fallow),
        save::FieldStage::Prepared => Ok(FieldStage::Prepared),
        save::FieldStage::Sown => Ok(FieldStage::Sown),
        save::FieldStage::Reaped => Ok(FieldStage::Reaped),
        other => Err(LoadError::Malformed(format!(
            "a field has stage {}",
            other.0
        ))),
    }
}

fn encode_fields(fields: &[Field], rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let crop_ids: Vec<&str> = rules.catalog.crops.iter().map(|c| c.id.as_str()).collect();
    let crops = strings(&mut fbb, &crop_ids);
    let list: Vec<_> = fields
        .iter()
        .map(|f| {
            let record: Vec<save::HarvestRecord> = f
                .soil
                .record
                .iter()
                .map(|r| {
                    save::HarvestRecord::new(
                        r.year,
                        r.kg_per_ha,
                        match r.limit {
                            Limit::Season => save::HarvestLimit::Season,
                            Limit::Soil => save::HarvestLimit::Soil,
                        },
                    )
                })
                .collect();
            let record = fbb.create_vector(&record);
            save::Field::create(
                &mut fbb,
                &save::FieldArgs {
                    id: f.id.get(),
                    household: f.household.get(),
                    x_cm: f.rect.x,
                    y_cm: f.rect.y,
                    w_cm: f.rect.w,
                    h_cm: f.rect.h,
                    crop: f.crop,
                    stage: stage_parts(f.stage),
                    stage_since: f.stage_since.minutes(),
                    work_h: f.work_h,
                    tended_h: f.tended_h,
                    ground: f.ground,
                    sown_day: f.sown_day,
                    sheaves_kg: f.sheaves_kg,
                    harvests: f.harvests,
                    clear_h_per_ha: f.clear_h_per_ha,
                    broken: f.broken,
                    holder: match f.holder {
                        Party::Household(id) | Party::Settlement(id) => id.get(),
                    },
                    holder_settlement: matches!(f.holder, Party::Settlement(_)),
                    leased: f.lease.is_some(),
                    lease_since: f.lease.map_or(0, |l| l.since.minutes()),
                    lease_until: f.lease.map_or(0, |l| l.until.minutes()),
                    lease_share: f.lease.map_or(0.0, |l| l.holder_share),
                    water_mm: f.water_mm,
                    need_mm: f.need_mm,
                    got_mm: f.got_mm,
                    fast_n: f.soil.fast_n,
                    slow_n: f.soil.slow_n,
                    supply_n: f.soil.supply_n,
                    record: Some(record),
                    last_flood_depth_cm: f.last_flood_depth_cm,
                    submerged_days: f.submerged_days,
                    reserved: f.reserved.map_or(0, |l| l.get()),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Fields::create(
        &mut fbb,
        &save::FieldsArgs {
            crops: Some(crops),
            fields: Some(list),
        },
    );
    finish(fbb, root)
}

fn decode_fields(bytes: &[u8], rules: &Rules, schema: Schema) -> Result<Vec<Field>, LoadError> {
    let root = section_root::<save::Fields>(bytes).map_err(|e| unreadable(SECTION_FIELDS, &e))?;
    let crops: Vec<Option<u16>> = read_strings(root.crops())
        .iter()
        .map(|id| rules.catalog.crop_index(id).map(|i| i as u16))
        .collect();
    let mut out = Vec::new();
    for f in root.fields().iter().flatten() {
        let id = required(f.id(), "a field")?;
        let crop = match crops.get(usize::from(f.crop())) {
            Some(Some(c)) => *c,
            // A crop the content no longer has: the field goes.
            Some(None) => continue,
            None => {
                return Err(LoadError::Malformed(format!(
                    "field {id} grows crop {} of {}",
                    f.crop(),
                    crops.len()
                )));
            }
        };
        let household = required(f.household(), "a field's household")?;
        // Saves before schema 12 have no holder: the household that works a field held it.
        let holder = match (PermanentId::from_raw(f.holder()), f.holder_settlement()) {
            (Some(id), true) => Party::Settlement(id),
            (Some(id), false) => Party::Household(id),
            (None, _) => Party::Household(household),
        };
        let lease = f.leased().then(|| Lease {
            since: time(f.lease_since()),
            until: time(f.lease_until()),
            holder_share: f.lease_share(),
        });
        let stage = stage_of(f.stage())?;
        let soil = if schema < Schema::V26 {
            // Before schema 26 no field kept its soil: its harvests are replayed from native
            // ground (ADR-0012 §6).
            match rules.catalog.crops.get(usize::from(crop)) {
                Some(c) => {
                    FieldSoil::replayed(&rules.land.soil, c, f64::from(f.ground()), f.harvests())
                }
                None => FieldSoil::native(&rules.land.soil, f64::from(f.ground())),
            }
        } else {
            let mut record = Vec::new();
            for r in f.record().iter().flatten() {
                let limit = match r.limit() {
                    save::HarvestLimit::Season => Limit::Season,
                    save::HarvestLimit::Soil => Limit::Soil,
                    other => {
                        return Err(LoadError::Malformed(format!(
                            "field {id} has a harvest held back by {}",
                            other.0
                        )));
                    }
                };
                record.push(HarvestRecord {
                    year: r.year(),
                    kg_per_ha: r.kg_per_ha(),
                    limit,
                });
            }
            if record.len() > RECORD_KEPT {
                return Err(LoadError::Malformed(format!(
                    "field {id} remembers {} harvests",
                    record.len()
                )));
            }
            FieldSoil {
                fast_n: f.fast_n(),
                slow_n: f.slow_n(),
                supply_n: f.supply_n(),
                record,
            }
        };
        if !(soil.fast_n.is_finite()
            && soil.slow_n.is_finite()
            && soil.supply_n.is_finite()
            && soil.fast_n >= 0.0
            && soil.slow_n >= 0.0
            && soil.supply_n >= 0.0)
        {
            return Err(LoadError::Malformed(format!(
                "field {id} has an invalid soil"
            )));
        }
        out.push(Field {
            id,
            household,
            holder,
            lease,
            rect: RectCm {
                x: f.x_cm(),
                y: f.y_cm(),
                w: f.w_cm(),
                h: f.h_cm(),
            },
            crop,
            stage,
            stage_since: time(f.stage_since()),
            work_h: f.work_h(),
            tended_h: f.tended_h(),
            ground: f.ground(),
            clear_h_per_ha: f.clear_h_per_ha(),
            broken: f.broken(),
            sown_day: f.sown_day(),
            sheaves_kg: f.sheaves_kg(),
            harvests: f.harvests(),
            // Before schema 24 no field kept its water: a crop growing then is at field capacity
            // and unstressed so far (ADR-0012 §6).
            water_mm: if schema < Schema::V24 && stage == FieldStage::Sown {
                rules.land.weather.soil_water_mm as f32
            } else {
                f.water_mm()
            },
            need_mm: f.need_mm(),
            got_mm: f.got_mm(),
            soil,
            last_flood_depth_cm: if schema >= Schema::V93 {
                f.last_flood_depth_cm()
            } else {
                0
            },
            submerged_days: if schema >= Schema::V93 {
                f.submerged_days()
            } else {
                0
            },
            // Schema 159 (M8c slice CU, fifth part); none before.
            reserved: id_of(f.reserved()),
        });
    }
    Ok(out)
}

// ---- plots and builds --------------------------------------------------------------------------

fn encode_plots(plots: &[Plot]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = plots
        .iter()
        .map(|p| {
            save::Plot::create(
                &mut fbb,
                &save::PlotArgs {
                    id: p.id.get(),
                    household: p.household.get(),
                    x_cm: p.rect.x,
                    y_cm: p.rect.y,
                    w_cm: p.rect.w,
                    h_cm: p.rect.h,
                    purpose: match p.use_ {
                        PlotUse::Dwelling => save::PlotUse::Dwelling,
                        PlotUse::Store => save::PlotUse::Store,
                        PlotUse::Work => save::PlotUse::Work,
                        PlotUse::Hall => save::PlotUse::Hall,
                        PlotUse::Tower => save::PlotUse::Tower,
                    },
                    since: p.since.minutes(),
                    receipt_weighed: p.receipt.map_or(0, |r| r.weighed),
                    receipt_hours: p.receipt.map_or(0.0, |r| r.hours),
                    receipt_next: p.receipt.map_or(0.0, |r| r.next_hours),
                    allot_law: p.allotted.map_or(0, |a| a.law.get()),
                    allot_parcel: p.allotted.map_or(0, |a| a.parcel),
                    allot_rule: p.allotted.map_or(0, |a| a.rule.code()),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Plots::create(&mut fbb, &save::PlotsArgs { plots: Some(list) });
    finish(fbb, root)
}

fn decode_plots(bytes: &[u8]) -> Result<Vec<Plot>, LoadError> {
    let root = section_root::<save::Plots>(bytes).map_err(|e| unreadable(SECTION_PLOTS, &e))?;
    let mut out = Vec::new();
    for p in root.plots().iter().flatten() {
        let id = required(p.id(), "a plot")?;
        let use_ = match p.purpose() {
            save::PlotUse::Dwelling => PlotUse::Dwelling,
            save::PlotUse::Store => PlotUse::Store,
            save::PlotUse::Work => PlotUse::Work,
            save::PlotUse::Hall => PlotUse::Hall,
            save::PlotUse::Tower => PlotUse::Tower,
            other => {
                return Err(LoadError::Malformed(format!(
                    "plot {id} has use {}",
                    other.0
                )));
            }
        };
        out.push(Plot {
            id,
            household: required(p.household(), "a plot's household")?,
            rect: RectCm {
                x: p.x_cm(),
                y: p.y_cm(),
                w: p.w_cm(),
                h: p.h_cm(),
            },
            use_,
            since: time(p.since()),
            receipt: (p.receipt_weighed() > 0).then(|| civ_land::SiteReceipt {
                weighed: p.receipt_weighed(),
                hours: p.receipt_hours(),
                next_hours: p.receipt_next(),
            }),
            allotted: match PermanentId::from_raw(p.allot_law()) {
                None => None,
                Some(law) => Some(civ_land::Allotment {
                    law,
                    parcel: p.allot_parcel(),
                    rule: *civ_land::AllotRule::ALL
                        .get(usize::from(p.allot_rule()))
                        .ok_or_else(|| {
                            LoadError::Malformed(format!(
                                "plot {id} was given by rule {}",
                                p.allot_rule()
                            ))
                        })?,
                }),
            },
        });
    }
    Ok(out)
}

fn encode_buildings(buildings: &[Building]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = buildings
        .iter()
        .map(|b| {
            let s = &b.spec;
            let program = fbb.create_string(&s.program);
            // Only what its version has: a design before frame version 3 is saved as ever.
            let params = fbb.create_vector(s.params_in_use());
            let materials: Vec<&str> = s.materials.iter().map(String::as_str).collect();
            let materials = strings(&mut fbb, &materials);
            let (x_cm, y_cm) = s.footprint.centre();
            let (footprint, radius_cm, length_cm, width_cm, angle) = match s.footprint {
                Footprint::Round { radius, .. } => (save::FootprintKind::Round, radius, 0, 0, 0),
                Footprint::Rect {
                    length,
                    width,
                    angle,
                    ..
                } => (save::FootprintKind::Rect, 0, length, width, angle),
            };
            let spec = save::BuildingSpec::create(
                &mut fbb,
                &save::BuildingSpecArgs {
                    program: Some(program),
                    version: s.version,
                    footprint,
                    x_cm,
                    y_cm,
                    radius_cm,
                    storeys: s.storeys,
                    params: Some(params),
                    materials: Some(materials),
                    style_seed: s.style_seed,
                    length_cm,
                    width_cm,
                    angle,
                },
            );
            let mut condition = Vec::with_capacity(b.condition.len());
            for c in &b.condition {
                condition.push(save::GroupCondition::create(
                    &mut fbb,
                    &save::GroupConditionArgs {
                        group: c.group,
                        quality: c.quality,
                        loss: c.loss,
                        installed: c.installed.minutes(),
                        repaired: c.repaired.minutes(),
                        state: match c.state {
                            GroupState::Sound => save::GroupState::Sound,
                            GroupState::Symptom => save::GroupState::Symptom,
                            GroupState::Failed => save::GroupState::Failed,
                        },
                    },
                ));
            }
            let condition = fbb.create_vector(&condition);
            let repair = b.repair.map(|r| {
                save::RepairState::create(
                    &mut fbb,
                    &save::RepairStateArgs {
                        group: r.group,
                        share: r.share,
                        work_h: r.work_h,
                    },
                )
            });
            let demolition = b.demolition.map(|d| {
                save::DemolitionSave::create(
                    &mut fbb,
                    &save::DemolitionSaveArgs {
                        fire: d.fire,
                        needed_h: d.needed_h,
                        work_h: d.work_h,
                        last_minute: d.last_minute,
                    },
                )
            });
            save::Building::create(
                &mut fbb,
                &save::BuildingArgs {
                    id: b.id.get(),
                    household: b.household.get(),
                    plot: b.plot.get(),
                    spec: Some(spec),
                    stage: b.stage,
                    work_h: b.work_h,
                    started: b.started.minutes(),
                    stage_since: b.stage_since.minutes(),
                    firm: b.firm.map_or(0, |f| f.get()),
                    condition: Some(condition),
                    state: match b.state {
                        BuildingState::Standing => save::BuildingState::Standing,
                        BuildingState::Damaged => save::BuildingState::Damaged,
                        BuildingState::Ruin => save::BuildingState::Ruin,
                    },
                    skill_h: b.skill_h,
                    repair,
                    style_from: b.style_from.map_or(0, |s| s.get()),
                    last_flood_depth_cm: b.last_flood_depth_cm,
                    days_wet: b.days_wet,
                    demolition,
                    polity: b.polity.map_or(0, |p| p.get()),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Builds::create(
        &mut fbb,
        &save::BuildsArgs {
            buildings: Some(list),
        },
    );
    finish(fbb, root)
}

/// A saved float, refused if it is not finite.
fn finite(x: f32, what: &str) -> Result<f32, LoadError> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(LoadError::Malformed(format!(
            "{what} is not a number ({x})"
        )))
    }
}

/// The condition of building `id`'s groups as saved (none before schema 17).
fn decode_condition(
    list: Option<flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<save::GroupCondition<'_>>>>,
    id: PermanentId,
) -> Result<Vec<GroupCondition>, LoadError> {
    let mut out: Vec<GroupCondition> = Vec::new();
    for c in list.iter().flatten() {
        if out.iter().any(|o| o.group == c.group()) {
            return Err(LoadError::Malformed(format!(
                "building {id} has group {} twice",
                c.group()
            )));
        }
        out.push(GroupCondition {
            group: c.group(),
            quality: finite(c.quality(), "a group's quality")?.clamp(0.0, 1.0),
            loss: finite(c.loss(), "a group's loss")?.clamp(0.0, 1.0),
            installed: time(c.installed()),
            repaired: time(c.repaired()),
            state: match c.state() {
                save::GroupState::Symptom => GroupState::Symptom,
                save::GroupState::Failed => GroupState::Failed,
                _ => GroupState::Sound,
            },
        });
    }
    Ok(out)
}

fn decode_buildings(bytes: &[u8], schema: Schema) -> Result<Vec<Building>, LoadError> {
    let root = section_root::<save::Builds>(bytes).map_err(|e| unreadable(SECTION_BUILDS, &e))?;
    let mut out = Vec::new();
    for b in root.buildings().iter().flatten() {
        let id = required(b.id(), "a building")?;
        let Some(s) = b.spec() else {
            return Err(LoadError::Malformed(format!("building {id} has no design")));
        };
        let footprint = match s.footprint() {
            save::FootprintKind::Round => Footprint::Round {
                x: s.x_cm(),
                y: s.y_cm(),
                radius: s.radius_cm(),
            },
            save::FootprintKind::Rect => Footprint::Rect {
                x: s.x_cm(),
                y: s.y_cm(),
                length: s.length_cm(),
                width: s.width_cm(),
                angle: s.angle(),
            },
            other => {
                return Err(LoadError::Malformed(format!(
                    "building {id} has footprint kind {}",
                    other.0
                )));
            }
        };
        let saved: Vec<i32> = s.params().map(|v| v.iter().collect()).unwrap_or_default();
        if saved.len() > PARAMS {
            return Err(LoadError::Malformed(format!(
                "building {id} has {} parameters; a design has at most {PARAMS}",
                saved.len()
            )));
        }
        // Designs from before schema 15 have eight; the rest are 0 (ADR-0009 §2).
        let mut params = [0; PARAMS];
        params[..saved.len()].copy_from_slice(&saved);
        out.push(Building {
            id,
            household: required(b.household(), "a building's household")?,
            plot: required(b.plot(), "a building's plot")?,
            spec: BuildingSpec {
                program: s.program().unwrap_or_default().to_owned(),
                version: s.version(),
                footprint,
                storeys: s.storeys(),
                params,
                materials: read_strings(s.materials()),
                style_seed: s.style_seed(),
            },
            stage: b.stage(),
            work_h: b.work_h(),
            started: time(b.started()),
            stage_since: time(b.stage_since()),
            // Before schema 16 no building named a firm.
            firm: PermanentId::from_raw(b.firm()),
            // Before schema 17 no building had a condition: the stage under way was worked at
            // middling skill, and its groups are put in place on loading (ADR-0009 §8).
            skill_h: if schema >= Schema::V17 {
                finite(b.skill_h(), "skill hours")?.max(0.0)
            } else {
                b.work_h() * condition::MIDDLING_SKILL as f32
            },
            condition: if schema >= Schema::V17 {
                decode_condition(b.condition(), id)?
            } else {
                Vec::new()
            },
            state: match b.state() {
                save::BuildingState::Damaged if schema >= Schema::V17 => BuildingState::Damaged,
                save::BuildingState::Ruin if schema >= Schema::V17 => BuildingState::Ruin,
                _ => BuildingState::Standing,
            },
            repair: match b.repair() {
                Some(r) if schema >= Schema::V17 => Some(Repair {
                    group: r.group(),
                    share: finite(r.share(), "a repair's share")?.clamp(0.0, 1.0),
                    work_h: finite(r.work_h(), "a repair's hours")?.max(0.0),
                }),
                _ => None,
            },
            // Before schema 22 no building followed another's style.
            style_from: PermanentId::from_raw(b.style_from()),
            last_flood_depth_cm: if schema >= Schema::V93 {
                b.last_flood_depth_cm()
            } else {
                0
            },
            days_wet: if schema >= Schema::V93 {
                b.days_wet()
            } else {
                0
            },
            demolition: if schema >= Schema::V98 {
                match b.demolition() {
                    Some(d) => {
                        let needed_h = finite(d.needed_h(), "demolition labour")?;
                        let work_h = finite(d.work_h(), "demolition progress")?;
                        if d.fire() == 0
                            || needed_h <= 0.0
                            || work_h < 0.0
                            || work_h >= needed_h
                            || d.last_minute() < -1
                        {
                            return Err(LoadError::Malformed("invalid demolition work".to_owned()));
                        }
                        Some(civ_land::buildings::Demolition {
                            fire: d.fire(),
                            needed_h,
                            work_h,
                            last_minute: d.last_minute(),
                        })
                    }
                    None => None,
                }
            } else {
                None
            },
            // Before schema 165 no polity owned a building.
            polity: PermanentId::from_raw(b.polity()),
        });
    }
    Ok(out)
}

// ---- wear --------------------------------------------------------------------------------------

/// Each tile exactly as it stands, at its own day, in tile order, with the routing view people
/// plan on (schema 23, ADR-0011 §5): a world lived on from the save matches one never saved.
fn encode_wear(wear: &Wear) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let mut tiles: Vec<&WearTile> = wear.tiles().iter().collect();
    tiles.sort_by_key(|t| t.index);
    let tiles: Vec<_> = tiles
        .into_iter()
        .map(|t| {
            let wear_exact = fbb.create_vector(&t.wear);
            let trail = fbb.create_vector(&t.trail);
            save::WearTile::create(
                &mut fbb,
                &save::WearTileArgs {
                    index: t.index,
                    wear: None,
                    trail: Some(trail),
                    day: t.day,
                    wear_exact: Some(wear_exact),
                },
            )
        })
        .collect();
    let tiles = fbb.create_vector(&tiles);
    let view: Vec<_> = wear
        .surveyed_tiles()
        .map(|(index, cells, trail)| {
            let cells = fbb.create_vector(cells);
            let trail = fbb.create_vector(trail);
            save::WearViewTile::create(
                &mut fbb,
                &save::WearViewTileArgs {
                    index,
                    cells: Some(cells),
                    trail: Some(trail),
                },
            )
        })
        .collect();
    let view = fbb.create_vector(&view);
    let root = save::Wear::create(
        &mut fbb,
        &save::WearArgs {
            tiles: Some(tiles),
            surveyed: wear.surveyed(),
            view: Some(view),
        },
    );
    finish(fbb, root)
}

/// The worn ground of a save on `day`. A save from before schema 23 kept each tile faded to the
/// day saved and rounded, and no routing view: that is drawn again when the world is put together.
fn decode_wear(bytes: &[u8], map: &WorldMap, day: i64) -> Result<Wear, LoadError> {
    let w = section_root::<save::Wear>(bytes).map_err(|e| unreadable(SECTION_WEAR, &e))?;
    let (cells, words) = (
        civ_land::paths::TILE_CELLS,
        civ_land::paths::TILE_CELLS / 64,
    );
    let tile_count =
        map.width.div_ceil(civ_land::paths::TILE) * map.height.div_ceil(civ_land::paths::TILE);
    let malformed = |what: &str, index: u32| {
        LoadError::Malformed(format!("worn ground: {what} of tile {index}"))
    };
    let mut tiles = Vec::new();
    for t in w.tiles().iter().flatten() {
        let index = t.index();
        let trail: Vec<u64> = t.trail().map(|v| v.iter().collect()).unwrap_or_default();
        let tile = match t.wear_exact() {
            Some(exact) => WearTile {
                index,
                day: t.day(),
                wear: exact.iter().collect(),
                trail,
            },
            None => WearTile {
                index,
                day,
                wear: t
                    .wear()
                    .map(|v| v.iter().map(|q| f32::from(q) / 65535.0).collect())
                    .unwrap_or_default(),
                trail,
            },
        };
        if index >= tile_count {
            return Err(malformed("the index", index));
        }
        if tile.wear.len() != cells || tile.trail.len() != words {
            return Err(malformed("the size", index));
        }
        tiles.push(tile);
    }
    let mut wear = Wear::new(map.width, map.height, map.cell_size_m).with_tiles(tiles);
    if let Some(view) = w.view() {
        let mut out = Vec::with_capacity(view.len());
        for v in view {
            let tile = ViewTile {
                index: v.index(),
                cells: v.cells().map(|c| c.bytes().to_vec()).unwrap_or_default(),
                trail: v.trail().map(|c| c.iter().collect()).unwrap_or_default(),
            };
            if tile.index >= tile_count {
                return Err(malformed("the routing view's index", tile.index));
            }
            if tile.cells.len() != cells || tile.trail.len() != words {
                return Err(malformed("the routing view's size", tile.index));
            }
            out.push(tile);
        }
        wear = wear.with_view(w.surveyed(), out);
    }
    Ok(wear)
}

// ---- market ------------------------------------------------------------------------------------

/// A market's tally, one value per good of the dictionary.
fn tally<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    v: &[f64],
    goods: usize,
) -> WIPOffset<flatbuffers::Vector<'a, f64>> {
    let mut v = v.to_vec();
    v.resize(goods, 0.0);
    fbb.create_vector(&v)
}

fn encode_markets(pop: &Population, goods: &[&str]) -> Vec<u8> {
    let mut markets: Vec<&Market> = pop.markets.iter().collect();
    markets.sort_by_key(|m| m.settlement);
    let mut fbb = FlatBufferBuilder::new();
    let good_dictionary = strings(&mut fbb, goods);
    let list: Vec<_> = markets
        .into_iter()
        .map(|m| {
            let paid_h = tally(&mut fbb, &m.paid_h, goods.len());
            let sold = tally(&mut fbb, &m.sold, goods.len());
            let unmet = tally(&mut fbb, &m.unmet, goods.len());
            let unmet_h = tally(&mut fbb, &m.unmet_h, goods.len());
            // Terms are kept for the goods that have sold.
            let last: Vec<_> = m
                .last
                .iter()
                .enumerate()
                .filter_map(|(g, t)| t.map(|(payment, price)| (g, payment, price)))
                .map(|(g, payment, price)| {
                    save::LastTerms::create(
                        &mut fbb,
                        &save::LastTermsArgs {
                            good: g as u32,
                            payment: u32::from(payment),
                            price,
                        },
                    )
                })
                .collect();
            let last = fbb.create_vector(&last);
            let recent: Vec<_> = m
                .recent
                .iter()
                .map(|t| {
                    save::TradeRecord::create(
                        &mut fbb,
                        &save::TradeRecordArgs {
                            at: t.at.minutes(),
                            seller: t.seller.get(),
                            buyer: t.buyer.get(),
                            good: u32::from(t.good),
                            units: t.units,
                            payment: u32::from(t.payment),
                            paid: t.paid,
                            // The ledger's channel codes.
                            channel: save::TradeChannel(t.channel as u8),
                            from: t.from.map_or(0, PermanentId::get),
                        },
                    )
                })
                .collect();
            let recent = fbb.create_vector(&recent);
            let history: Vec<save::MonthOfTrade> = m
                .history
                .iter()
                .map(|h| {
                    save::MonthOfTrade::new(h.month, u32::from(h.good), h.trades, h.units, h.paid_h)
                })
                .collect();
            let history = fbb.create_vector(&history);
            save::MarketState::create(
                &mut fbb,
                &save::MarketStateArgs {
                    settlement: m.settlement.get(),
                    day: m.day,
                    paid_h: Some(paid_h),
                    sold: Some(sold),
                    unmet: Some(unmet),
                    unmet_h: Some(unmet_h),
                    trades: m.trades,
                    last: Some(last),
                    recent: Some(recent),
                    history: Some(history),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Markets::create(
        &mut fbb,
        &save::MarketsArgs {
            markets: Some(list),
            goods: Some(good_dictionary),
        },
    );
    finish(fbb, root)
}

fn decode_markets(bytes: &[u8], rules: &Rules) -> Result<Vec<Market>, LoadError> {
    let root = section_root::<save::Markets>(bytes).map_err(|e| unreadable(SECTION_MARKET, &e))?;
    let goods = good_map(&read_strings(root.goods()), rules);
    let catalog_goods = rules.catalog.goods.len();
    let mut out = Vec::new();
    for m in root.markets().iter().flatten() {
        let settlement = required(m.settlement(), "a market's settlement")?;
        let what = || format!("the market of settlement {settlement}");
        let mut market = Market::new(settlement, catalog_goods, m.day());
        market.trades = m.trades();
        for (saved, kept) in [
            (m.paid_h(), &mut market.paid_h),
            (m.sold(), &mut market.sold),
            (m.unmet(), &mut market.unmet),
            (m.unmet_h(), &mut market.unmet_h),
        ] {
            let saved: Vec<f64> = saved.map(|v| v.iter().collect()).unwrap_or_default();
            if saved.len() != goods.len() {
                return Err(LoadError::Malformed(format!(
                    "{} tallies {} goods of {}",
                    what(),
                    saved.len(),
                    goods.len()
                )));
            }
            // Goods the content no longer has are dropped.
            for (x, g) in saved.iter().zip(&goods) {
                if let Some(g) = g {
                    kept[*g] = *x;
                }
            }
        }
        for t in m.last().iter().flatten() {
            let good = saved_good(&goods, t.good(), what)?;
            let payment = saved_good(&goods, t.payment(), what)?;
            if let (Some(good), Some(payment)) = (good, payment) {
                market.last[usize::from(good)] = Some((payment, t.price()));
            }
        }
        for r in m.recent().iter().flatten() {
            let channel = match Channel::from_code(r.channel().0) {
                Some(c @ (Channel::Barter | Channel::Sale)) => c,
                _ => {
                    return Err(LoadError::Malformed(format!(
                        "{} remembers a trade on channel {}",
                        what(),
                        r.channel().0
                    )));
                }
            };
            let good = saved_good(&goods, r.good(), what)?;
            let payment = saved_good(&goods, r.payment(), what)?;
            // A trade in a good the content no longer has is forgotten.
            let (Some(good), Some(payment)) = (good, payment) else {
                continue;
            };
            market.recent.push_back(Trade {
                at: time(r.at()),
                seller: required(r.seller(), "a trade's seller")?,
                buyer: required(r.buyer(), "a trade's buyer")?,
                good,
                units: r.units(),
                payment,
                paid: r.paid(),
                channel,
                from: PermanentId::from_raw(r.from()),
            });
        }
        for h in m.history().iter().flatten() {
            // A good the content no longer has leaves the history.
            if let Some(good) = saved_good(&goods, h.good(), what)? {
                market.history.push(MonthOfTrade {
                    month: h.month(),
                    good,
                    trades: h.trades(),
                    units: h.units(),
                    paid_h: h.paid_h(),
                });
            }
        }
        // In the loaded content's order of goods.
        market.history.sort_by_key(|h| (h.month, h.good));
        out.push(market);
    }
    Ok(out)
}

// ---- firms -------------------------------------------------------------------------------------

fn encode_offer<'a>(fbb: &mut FlatBufferBuilder<'a>, o: &Offer) -> WIPOffset<save::Offer<'a>> {
    save::Offer::create(
        fbb,
        &save::OfferArgs {
            good: u32::from(o.good),
            payment: u32::from(o.payment),
            price: o.price,
            units: o.units,
            ask_h: o.ask_h,
        },
    )
}

fn encode_firms(
    pop: &Population,
    goods: &[&str],
    activities: &[&str],
    recipes: &[&str],
) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let good_dictionary = strings(&mut fbb, goods);
    let activity_dictionary = strings(&mut fbb, activities);
    let list: Vec<_> = pop
        .firms
        .iter()
        .map(|f| {
            let lines: Vec<u32> = f.lines.iter().map(|&g| u32::from(g)).collect();
            let lines = fbb.create_vector(&lines);
            let mut stores = f.stores.clone();
            stores.resize(goods.len(), 0.0);
            let stores = fbb.create_vector(&stores);
            let offers: Vec<_> = f.offers.iter().map(|o| encode_offer(&mut fbb, o)).collect();
            let offers = fbb.create_vector(&offers);
            let entries: Vec<_> = f
                .books
                .entries
                .iter()
                .map(|e| {
                    save::BookEntry::create(
                        &mut fbb,
                        &save::BookEntryArgs {
                            at: e.at.minutes(),
                            kind: e.kind as u8,
                            good: u32::from(e.good),
                            amount: e.amount,
                            other: raw(e.other),
                        },
                    )
                })
                .collect();
            let entries = fbb.create_vector(&entries);
            let months: Vec<_> = f
                .books
                .months
                .iter()
                .map(|m| {
                    let lines: Vec<save::BookLine> = m
                        .lines
                        .iter()
                        .map(|&(k, g, a)| save::BookLine::new(u32::from(g), a, k as u8))
                        .collect();
                    let lines = fbb.create_vector(&lines);
                    save::MonthStatement::create(
                        &mut fbb,
                        &save::MonthStatementArgs {
                            month: m.month,
                            lines: Some(lines),
                            owner_h: m.owner_h,
                            hired_h: m.hired_h,
                            income_h: m.income_h,
                            costs_h: m.costs_h,
                            stock_h: m.stock_h,
                        },
                    )
                })
                .collect();
            let months = fbb.create_vector(&months);
            let wage = f.wage.map(|w| {
                save::WageOfferState::create(
                    &mut fbb,
                    &save::WageOfferStateArgs {
                        activity: u32::from(w.activity),
                        pay: u32::from(w.pay),
                        per_hour: w.per_hour,
                        hour_h: w.hour_h,
                        hours: w.hours,
                        taken: w.taken,
                        reviewed: w.reviewed,
                        bell_min: w.bell_min,
                        shift_min: w.shift_min,
                    },
                )
            });
            // Its batches under way (schema 172), each by its recipe's content id.
            let batches: Vec<_> = f
                .batches
                .iter()
                .filter_map(|b| {
                    let recipe = fbb.create_string(recipes.get(b.recipe)?);
                    Some(save::BatchSave::create(
                        &mut fbb,
                        &save::BatchSaveArgs {
                            recipe: Some(recipe),
                            units: b.units,
                            quality: b.quality,
                            ends_day: b.ends_day,
                        },
                    ))
                })
                .collect();
            let batches = (!batches.is_empty()).then(|| fbb.create_vector(&batches));
            save::FirmState::create(
                &mut fbb,
                &save::FirmStateArgs {
                    id: f.id.get(),
                    owner: f.owner.get(),
                    owner_since: f.owner_since.minutes(),
                    founder: f.founder.get(),
                    settlement: raw(f.settlement),
                    founded: f.founded.minutes(),
                    closed: f.closed.is_some(),
                    closed_at: f.closed.map_or(0, |(t, _)| t.minutes()),
                    exit: f.closed.map_or(0, |(_, e)| e as u8),
                    lines: Some(lines),
                    stores: Some(stores),
                    stores_at: f.stores_at.minutes(),
                    offers: Some(offers),
                    sold: f.last_sale.is_some(),
                    last_sale: f.last_sale.map_or(0, SimTime::minutes),
                    entries: Some(entries),
                    months: Some(months),
                    wage,
                    most_at_once: f.most_at_once,
                    most_at_once_day: f.most_at_once_day,
                    batches,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Firms::create(
        &mut fbb,
        &save::FirmsArgs {
            firms: Some(list),
            goods: Some(good_dictionary),
            activities: Some(activity_dictionary),
        },
    );
    finish(fbb, root)
}

/// Offers saved against dictionary `goods`, in the loaded catalog; terms in a good it no longer
/// has are withdrawn.
fn decode_offers<'a>(
    saved: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<save::Offer<'a>>>>,
    goods: &[Option<usize>],
    what: impl Fn() -> String + Copy,
) -> Result<Vec<Offer>, LoadError> {
    let mut out = Vec::new();
    for o in saved.iter().flatten() {
        let good = saved_good(goods, o.good(), what)?;
        let payment = saved_good(goods, o.payment(), what)?;
        if let (Some(good), Some(payment)) = (good, payment) {
            out.push(Offer {
                good,
                payment,
                price: o.price(),
                units: o.units(),
                ask_h: o.ask_h(),
            });
        }
    }
    Ok(out)
}

fn decode_firms(bytes: &[u8], rules: &Rules) -> Result<Vec<Firm>, LoadError> {
    let root = section_root::<save::Firms>(bytes).map_err(|e| unreadable(SECTION_FIRMS, &e))?;
    let goods = good_map(&read_strings(root.goods()), rules);
    let activities = activity_map(&read_strings(root.activities()), rules);
    let catalog_goods = rules.catalog.goods.len();
    let mut out = Vec::new();
    for f in root.firms().iter().flatten() {
        let fid = required(f.id(), "a firm")?;
        let what = || format!("firm {fid}");
        let mut lines = Vec::new();
        for g in f.lines().iter().flatten() {
            if let Some(g) = saved_good(&goods, g, what)? {
                lines.push(g);
            }
        }
        let saved: Vec<f64> = f.stores().map(|v| v.iter().collect()).unwrap_or_default();
        if saved.len() != goods.len() {
            return Err(LoadError::Malformed(format!(
                "firm {fid} stores {} goods of {}",
                saved.len(),
                goods.len()
            )));
        }
        let mut stores = vec![0.0; catalog_goods];
        for (kg, g) in saved.iter().zip(&goods) {
            if let Some(g) = g {
                stores[*g] = *kg;
            }
        }
        let closed = if f.closed() {
            let why = Exit::from_code(f.exit()).ok_or_else(|| {
                LoadError::Malformed(format!("firm {fid} closed for reason {}", f.exit()))
            })?;
            Some((time(f.closed_at()), why))
        } else {
            None
        };
        let mut books = Books::default();
        for e in f.entries().iter().flatten() {
            let kind = BookKind::from_code(e.kind()).ok_or_else(|| {
                LoadError::Malformed(format!("{} has a book entry of kind {}", what(), e.kind()))
            })?;
            if let Some(good) = saved_good(&goods, e.good(), what)? {
                books.entries.push_back(BookEntryOf {
                    at: time(e.at()),
                    kind,
                    good,
                    amount: e.amount(),
                    other: id(e.other()),
                });
            }
        }
        for m in f.months().iter().flatten() {
            let mut month_lines = Vec::new();
            for l in m.lines().iter().flatten() {
                let kind = BookKind::from_code(l.kind()).ok_or_else(|| {
                    LoadError::Malformed(format!("{} has a book line of kind {}", what(), l.kind()))
                })?;
                if let Some(good) = saved_good(&goods, l.good(), what)? {
                    month_lines.push((kind, good, l.amount()));
                }
            }
            month_lines.sort_by_key(|&(k, g, _)| (k, g));
            books.months.push(Statement {
                month: m.month(),
                lines: month_lines,
                owner_h: m.owner_h(),
                hired_h: m.hired_h(),
                income_h: m.income_h(),
                costs_h: m.costs_h(),
                stock_h: m.stock_h(),
            });
        }
        out.push(Firm {
            id: fid,
            owner: required(f.owner(), "a firm's owner")?,
            owner_since: time(f.owner_since()),
            founder: required(f.founder(), "a firm's founder")?,
            settlement: id(f.settlement()),
            founded: time(f.founded()),
            closed,
            lines,
            stores,
            stores_at: time(f.stores_at()),
            offers: decode_offers(f.offers(), &goods, what)?,
            last_sale: f.sold().then(|| time(f.last_sale())),
            // A wage whose work or pay the content no longer has is withdrawn.
            wage: match f.wage() {
                Some(w) => {
                    let activity = activities.get(w.activity() as usize).copied().flatten();
                    let pay = saved_good(&goods, w.pay(), what)?;
                    match (activity, pay) {
                        (Some(activity), Some(pay)) => Some(WageOffer {
                            activity,
                            pay,
                            per_hour: w.per_hour(),
                            hour_h: w.hour_h(),
                            hours: w.hours(),
                            taken: w.taken(),
                            reviewed: w.reviewed(),
                            // Before schema 178 no works rang a bell.
                            bell_min: w.bell_min(),
                            shift_min: w.shift_min(),
                        }),
                        _ => None,
                    }
                }
                None => None,
            },
            books,
            // Counters start again on load.
            flows: Default::default(),
            // Before schema 16 nobody was counted: none at once, long ago.
            most_at_once: f.most_at_once(),
            most_at_once_day: f.most_at_once_day(),
            // Its batches under way (schema 172); before, none. One of a recipe the content no
            // longer has is dropped, its inputs with it.
            batches: {
                let mut out: Vec<civ_agents::person::Batch> = Vec::new();
                for b in f.batches().iter().flatten() {
                    let ok = b.units().is_finite()
                        && b.units() > 0.0
                        && b.quality().is_finite()
                        && b.quality() >= 0.0;
                    if !ok {
                        return Err(LoadError::Malformed(format!(
                            "{} keeps a batch of no units or quality",
                            what()
                        )));
                    }
                    let Some(recipe) = b
                        .recipe()
                        .and_then(|id| rules.catalog.recipes.iter().position(|r| r.id == id))
                    else {
                        continue;
                    };
                    out.push(civ_agents::person::Batch {
                        recipe,
                        units: b.units(),
                        quality: b.quality(),
                        ends_day: b.ends_day(),
                    });
                }
                out
            },
        });
    }
    Ok(out)
}

// ---- wealth ------------------------------------------------------------------------------------

fn encode_wealth(pop: &Population) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let years: Vec<_> = pop
        .wealth_years
        .iter()
        .map(|y| {
            let s = &y.spread;
            save::WealthYear::create(
                &mut fbb,
                &save::WealthYearArgs {
                    year: y.year,
                    settlement: s.settlement.get(),
                    households: s.households,
                    people: s.people,
                    gini_goods: s.gini_goods,
                    gini_held: s.gini_held,
                    gini_worked: s.gini_worked,
                    gini_floor: s.gini_floor,
                    top_tenth_goods: s.top_tenth_goods,
                    holding_none: s.holding_none,
                    working_none: s.working_none,
                    goods_h_per_head: s.goods_h_per_head,
                    worked_ha_per_head: s.worked_ha_per_head,
                    floor_m2_per_house: s.floor_m2_per_house,
                    common_ha: s.common_ha,
                    roofed_m2_per_house: s.roofed_m2_per_house,
                    storage_kg_per_house: s.storage_kg_per_house,
                },
            )
        })
        .collect();
    let years = fbb.create_vector(&years);
    let root = save::Wealth::create(&mut fbb, &save::WealthArgs { years: Some(years) });
    finish(fbb, root)
}

fn decode_wealth(bytes: &[u8]) -> Result<Vec<WealthYear>, LoadError> {
    let root = section_root::<save::Wealth>(bytes).map_err(|e| unreadable(SECTION_WEALTH, &e))?;
    let mut out = Vec::new();
    for y in root.years().iter().flatten() {
        out.push(WealthYear {
            year: y.year(),
            spread: Spread {
                settlement: required(y.settlement(), "a year's wealth measures")?,
                households: y.households(),
                people: y.people(),
                gini_goods: y.gini_goods(),
                gini_held: y.gini_held(),
                gini_worked: y.gini_worked(),
                gini_floor: y.gini_floor(),
                top_tenth_goods: y.top_tenth_goods(),
                holding_none: y.holding_none(),
                working_none: y.working_none(),
                goods_h_per_head: y.goods_h_per_head(),
                worked_ha_per_head: y.worked_ha_per_head(),
                floor_m2_per_house: y.floor_m2_per_house(),
                common_ha: y.common_ha(),
                // Before schema 16 they were not measured: 0.
                roofed_m2_per_house: y.roofed_m2_per_house(),
                storage_kg_per_house: y.storage_kg_per_house(),
            },
        });
    }
    Ok(out)
}

// ---- knowledge ---------------------------------------------------------------------------------

fn encode_knowledge(pop: &Population, techniques: &[&str]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let dictionary = strings(&mut fbb, techniques);
    let entries: Vec<save::KnowledgeEntry> = pop
        .knowledge
        .iter()
        .map(|e| {
            let (kind, source) = match e.kind {
                KnowledgeEventKind::Known(source) => (0, source),
                KnowledgeEventKind::Lost => (1, KnowSource::Founder),
            };
            save::KnowledgeEntry::new(
                e.at.minutes(),
                e.settlement.get(),
                e.person.get(),
                raw(source.saved_id()),
                e.technique,
                kind,
                source.code(),
            )
        })
        .collect();
    let entries = fbb.create_vector(&entries);
    // The other settlement each entry concerns (schema 60), only when one does.
    let elsewhere = pop
        .knowledge
        .iter()
        .any(|e| e.elsewhere.is_some())
        .then(|| {
            let ids: Vec<u64> = pop.knowledge.iter().map(|e| raw(e.elsewhere)).collect();
            fbb.create_vector(&ids)
        });
    let trust: Vec<save::TrustEntry> = pop
        .trust
        .iter()
        .map(|t| {
            save::TrustEntry::new(
                t.at.minutes(),
                t.settlement.get(),
                t.failures,
                t.years,
                t.technique,
            )
        })
        .collect();
    let trust = fbb.create_vector(&trust);
    let fire_trust: Vec<save::FireTrustEntry> = pop
        .fire_trust
        .iter()
        .map(|t| {
            save::FireTrustEntry::new(
                t.at.minutes(),
                t.settlement.get(),
                t.losses,
                t.years,
                t.tile,
                t.wall_kind,
            )
        })
        .collect();
    let fire_trust = fbb.create_vector(&fire_trust);
    // Enrolments and today's sessions (schema 193, M10c slice EB): techniques index the list.
    let enrolments: Vec<save::EnrolmentSave> = pop
        .enrolments
        .iter()
        .map(|e| {
            save::EnrolmentSave::new(
                e.teacher.get(),
                e.learner.get(),
                e.payer.get(),
                e.technique,
                e.fee_kcal,
                e.since,
                e.paid,
            )
        })
        .collect();
    let enrolments = fbb.create_vector(&enrolments);
    let sessions = |list: &[civ_agents::knowledge::Session]| -> Vec<save::SessionSave> {
        list.iter()
            .map(|s| save::SessionSave::new(s.person.get(), s.technique, s.minutes))
            .collect()
    };
    let taught = fbb.create_vector(&sessions(&pop.taught));
    let studied = fbb.create_vector(&sessions(&pop.studied));
    // Households untaught (schema 194, M10c slice EB, third part).
    let untaught: Vec<save::UntaughtSave> = pop
        .untaught
        .iter()
        .map(|u| save::UntaughtSave::new(u.household.get(), u.technique, u.day))
        .collect();
    let untaught = fbb.create_vector(&untaught);
    // Written records and the writing toward them (schema 195, M10c slice EC).
    let writings: Vec<save::RecordSave> = pop
        .writings
        .iter()
        .map(|r| {
            save::RecordSave::new(
                r.id.get(),
                r.holder.get(),
                r.maker.get(),
                r.technique,
                r.made,
            )
        })
        .collect();
    let writings = fbb.create_vector(&writings);
    let written = fbb.create_vector(&sessions(&pop.written));
    let kept: Vec<u64> = pop
        .writings
        .iter()
        .map(|r| r.building.map_or(0, PermanentId::get))
        .collect();
    let writings_kept = fbb.create_vector(&kept);
    let root = save::Knowledge::create(
        &mut fbb,
        &save::KnowledgeArgs {
            techniques: Some(dictionary),
            entries: Some(entries),
            trust: Some(trust),
            elsewhere,
            fire_trust: Some(fire_trust),
            enrolments: Some(enrolments),
            taught: Some(taught),
            studied: Some(studied),
            untaught: Some(untaught),
            writings: Some(writings),
            written: Some(written),
            writings_kept: Some(writings_kept),
        },
    );
    finish(fbb, root)
}

/// What each settlement has seen of each technique's buildings (schema 18), from the knowledge
/// section. A technique the loaded content no longer has drops out.
fn decode_trust(bytes: &[u8], rules: &Rules) -> Result<Vec<Trust>, LoadError> {
    let root = section_root::<save::Knowledge>(bytes).map_err(|e| unreadable(SECTION_KNOW, &e))?;
    let ids: Vec<Option<usize>> = read_strings(root.techniques())
        .iter()
        .map(|id| rules.catalog.technique_index(id))
        .collect();
    let mut out: Vec<Trust> = Vec::new();
    for t in root.trust().iter().flatten() {
        let technique = ids.get(usize::from(t.technique())).ok_or_else(|| {
            LoadError::Malformed(format!(
                "a trust entry names technique {} of {}",
                t.technique(),
                ids.len()
            ))
        })?;
        let Some(technique) = technique else {
            continue;
        };
        let (failures, years) = (t.failures(), t.years());
        if !(failures.is_finite() && failures >= 0.0 && years.is_finite() && years >= 0.0) {
            return Err(LoadError::Malformed(format!(
                "a trust entry has {failures} failures in {years} building-years"
            )));
        }
        let settlement = required(t.settlement(), "a trust entry")?;
        let technique = *technique as u16;
        if out
            .iter()
            .any(|x| x.settlement == settlement && x.technique == technique)
        {
            return Err(LoadError::Malformed(format!(
                "settlement {settlement:?} has two trust entries for technique {technique}"
            )));
        }
        out.push(Trust {
            settlement,
            technique,
            failures,
            years,
            at: time(t.at()),
        });
    }
    Ok(out)
}

/// Enrolments to learn from another household and today's sessions (schema 193, M10c slice EB),
/// households untaught (schema 194) and written records (schema 195), from the knowledge section: one naming a technique the loaded content no longer has, or
/// nobody, drops out.
fn decode_enrolments(
    bytes: &[u8],
    rules: &Rules,
    people: &mut Population,
) -> Result<(), LoadError> {
    let root = section_root::<save::Knowledge>(bytes).map_err(|e| unreadable(SECTION_KNOW, &e))?;
    let ids: Vec<Option<usize>> = read_strings(root.techniques())
        .iter()
        .map(|id| rules.catalog.technique_index(id))
        .collect();
    let technique = |t: u16| {
        ids.get(usize::from(t))
            .copied()
            .flatten()
            .and_then(|i| u16::try_from(i).ok())
    };
    for e in root.enrolments().into_iter().flatten() {
        let (Some(teacher), Some(learner), Some(payer), Some(t)) = (
            PermanentId::from_raw(e.teacher()),
            PermanentId::from_raw(e.learner()),
            PermanentId::from_raw(e.payer()),
            technique(e.technique()),
        ) else {
            continue;
        };
        people.enrolments.push(civ_agents::knowledge::Enrolment {
            teacher,
            learner,
            technique: t,
            fee_kcal: e.fee_kcal(),
            payer,
            since: e.since(),
            paid: e.paid(),
        });
    }
    let sessions = |list: Option<civ_schema::flatbuffers::Vector<'_, save::SessionSave>>| {
        list.into_iter()
            .flatten()
            .filter_map(|s| {
                Some(civ_agents::knowledge::Session {
                    person: PermanentId::from_raw(s.person())?,
                    technique: technique(s.technique())?,
                    minutes: s.minutes(),
                })
            })
            .collect::<Vec<_>>()
    };
    people.taught = sessions(root.taught());
    people.studied = sessions(root.studied());
    people.written = sessions(root.written());
    let kept: Vec<u64> = root.writings_kept().into_iter().flatten().collect();
    people.writings = root
        .writings()
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(i, r)| {
            Some(civ_agents::knowledge::Record {
                id: PermanentId::from_raw(r.id())?,
                technique: technique(r.technique())?,
                holder: PermanentId::from_raw(r.holder())?,
                maker: PermanentId::from_raw(r.maker())?,
                made: r.made(),
                building: kept.get(i).copied().and_then(PermanentId::from_raw),
            })
        })
        .collect();
    people.untaught = root
        .untaught()
        .into_iter()
        .flatten()
        .filter_map(|u| {
            Some(civ_agents::knowledge::Untaught {
                household: PermanentId::from_raw(u.household())?,
                technique: technique(u.technique())?,
                day: u.day(),
            })
        })
        .collect();
    Ok(())
}

/// Fire caution (schema 94); malformed or repeated records are refused.
fn decode_fire_trust(bytes: &[u8]) -> Result<Vec<FireTrust>, LoadError> {
    let root = section_root::<save::Knowledge>(bytes).map_err(|e| unreadable(SECTION_KNOW, &e))?;
    let mut out: Vec<FireTrust> = Vec::new();
    for t in root.fire_trust().iter().flatten() {
        let (losses, years) = (t.losses(), t.years());
        if !(losses.is_finite() && losses >= 0.0 && years.is_finite() && years >= 0.0)
            || t.wall_kind() > 3
        {
            return Err(LoadError::Malformed(
                "invalid fire caution record".to_owned(),
            ));
        }
        let settlement = required(t.settlement(), "a fire caution entry")?;
        if out.iter().any(|x| {
            x.settlement == settlement && x.tile == t.tile() && x.wall_kind == t.wall_kind()
        }) {
            return Err(LoadError::Malformed(
                "repeated fire caution record".to_owned(),
            ));
        }
        out.push(FireTrust {
            settlement,
            tile: t.tile(),
            wall_kind: t.wall_kind(),
            losses,
            years,
            at: time(t.at()),
        });
    }
    Ok(out)
}

fn decode_knowledge(bytes: &[u8], rules: &Rules) -> Result<Vec<KnowledgeEvent>, LoadError> {
    let root = section_root::<save::Knowledge>(bytes).map_err(|e| unreadable(SECTION_KNOW, &e))?;
    let ids: Vec<Option<usize>> = read_strings(root.techniques())
        .iter()
        .map(|id| rules.catalog.technique_index(id))
        .collect();
    let mut out = Vec::new();
    // The other settlement each entry concerns (schema 60), by entry.
    let elsewhere: Vec<u64> = root
        .elsewhere()
        .map(|v| v.iter().collect())
        .unwrap_or_default();
    let entries = root.entries().map_or(0, |v| v.len());
    if !elsewhere.is_empty() && elsewhere.len() != entries {
        return Err(LoadError::Malformed(format!(
            "{} knowledge entries with {} settlements elsewhere",
            entries,
            elsewhere.len()
        )));
    }
    for (i, e) in root.entries().iter().flatten().enumerate() {
        let t = ids.get(usize::from(e.technique())).ok_or_else(|| {
            LoadError::Malformed(format!(
                "a knowledge entry names technique {} of {}",
                e.technique(),
                ids.len()
            ))
        })?;
        // A technique the loaded content no longer has drops out of the record.
        let Some(t) = t else {
            continue;
        };
        let kind = match e.kind() {
            0 => {
                KnowledgeEventKind::Known(KnowSource::from_code(e.source(), id(e.source_person())))
            }
            1 => KnowledgeEventKind::Lost,
            other => {
                return Err(LoadError::Malformed(format!(
                    "a knowledge entry has kind {other}"
                )));
            }
        };
        out.push(KnowledgeEvent {
            at: time(e.at()),
            settlement: required(e.settlement(), "a knowledge entry")?,
            technique: *t as u16,
            person: required(e.person(), "a knowledge entry")?,
            kind,
            elsewhere: elsewhere.get(i).copied().and_then(PermanentId::from_raw),
        });
    }
    Ok(out)
}

// ---- events ------------------------------------------------------------------------------------

fn encode_events(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let events: Vec<_> = sim
        .scheduler
        .pending_events()
        .into_iter()
        .map(|e| {
            let (kind, person, version, trip) = match e.event {
                SimEvent::Agent(AgentEvent::Step { person, version }) => {
                    (save::EventKind::AgentStep, person.get(), version, 0)
                }
                SimEvent::Agent(AgentEvent::Fire) => (save::EventKind::FireStep, 0, 0, 0),
                SimEvent::Agent(AgentEvent::Ignite { building }) => {
                    (save::EventKind::Ignite, building.get(), 0, 0)
                }
                SimEvent::Agent(AgentEvent::BandLook) => (save::EventKind::BandLook, 0, 0, 0),
                SimEvent::Agent(AgentEvent::Gates) => (save::EventKind::Gates, 0, 0, 0),
                SimEvent::Agent(AgentEvent::Trains) => (save::EventKind::Trains, 0, 0, 0),
                SimEvent::Agent(AgentEvent::Gas) => (save::EventKind::Gas, 0, 0, 0),
                SimEvent::Agent(AgentEvent::Pass {
                    person,
                    version,
                    trip,
                }) => (save::EventKind::Pass, person.get(), version, trip),
            };
            save::PendingEvent::create(
                &mut fbb,
                &save::PendingEventArgs {
                    at: e.at.minutes(),
                    phase: e.phase,
                    tiebreak: e.tiebreak,
                    seq: e.seq,
                    kind,
                    person,
                    version,
                    trip,
                },
            )
        })
        .collect();
    let events = fbb.create_vector(&events);
    let root = save::PendingEvents::create(
        &mut fbb,
        &save::PendingEventsArgs {
            events: Some(events),
        },
    );
    finish(fbb, root)
}

fn decode_events(bytes: &[u8]) -> Result<Vec<PendingEvent<SimEvent>>, LoadError> {
    let root =
        section_root::<save::PendingEvents>(bytes).map_err(|e| unreadable(SECTION_EVENTS, &e))?;
    let mut out = Vec::new();
    for e in root.events().iter().flatten() {
        let event = match e.kind() {
            save::EventKind::AgentStep => SimEvent::Agent(AgentEvent::Step {
                person: required(e.person(), "an event's person")?,
                version: e.version(),
            }),
            save::EventKind::FireStep => SimEvent::Agent(AgentEvent::Fire),
            save::EventKind::Ignite => SimEvent::Agent(AgentEvent::Ignite {
                building: required(e.person(), "an ignition's building")?,
            }),
            save::EventKind::BandLook => SimEvent::Agent(AgentEvent::BandLook),
            save::EventKind::Gates => SimEvent::Agent(AgentEvent::Gates),
            save::EventKind::Trains => SimEvent::Agent(AgentEvent::Trains),
            save::EventKind::Gas => SimEvent::Agent(AgentEvent::Gas),
            save::EventKind::Pass => SimEvent::Agent(AgentEvent::Pass {
                person: required(e.person(), "a passage's walker")?,
                version: e.version(),
                trip: e.trip(),
            }),
            other => {
                return Err(LoadError::Incompatible(format!(
                    "a pending event is of kind {}, which this build does not know",
                    other.0
                )));
            }
        };
        out.push(PendingEvent {
            at: time(e.at()),
            phase: e.phase(),
            tiebreak: e.tiebreak(),
            seq: e.seq(),
            event,
        });
    }
    Ok(out)
}

// ---- opinion (M4c slice AG, ADR-0016 §4) ------------------------------------------------------

fn encode_opinion(opinion: &civ_agents::opinion::Opinion, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let positions: Vec<save::PositionSave> = opinion
        .positions
        .iter()
        .map(|p| {
            save::PositionSave::new(
                p.holder.get(),
                p.since,
                p.x,
                p.anchor,
                p.salience,
                p.heard,
                p.policy,
            )
        })
        .collect();
    let positions = fbb.create_vector(&positions);
    let ids: Vec<_> = rules
        .catalog
        .policies
        .iter()
        .map(|d| fbb.create_string(&d.id))
        .collect();
    let policies = fbb.create_vector(&ids);
    let root = save::OpinionSave::create(
        &mut fbb,
        &save::OpinionSaveArgs {
            positions: Some(positions),
            policies: Some(policies),
            told: opinion.told,
            taken: opinion.taken,
        },
    );
    finish(fbb, root)
}

/// Positions on a question the loaded content no longer asks are let go: they are re-anchored
/// from scratch if it asks one again.
fn decode_opinion(bytes: &[u8], rules: &Rules) -> Result<civ_agents::opinion::Opinion, LoadError> {
    use civ_agents::opinion::{Opinion, Position};
    let root =
        section_root::<save::OpinionSave>(bytes).map_err(|e| unreadable(SECTION_OPINION, &e))?;
    let map: Vec<Option<u16>> = read_strings(root.policies())
        .iter()
        .map(|id| {
            rules
                .catalog
                .policies
                .iter()
                .position(|p| &p.id == id && p.question.is_some())
                .and_then(|i| u16::try_from(i).ok())
        })
        .collect();
    let mut positions = Vec::new();
    for p in root.positions().iter().flatten() {
        let Some(Some(policy)) = map.get(usize::from(p.policy())) else {
            continue;
        };
        positions.push(Position {
            holder: required(p.holder(), "a position")?,
            policy: *policy,
            x: p.x(),
            anchor: p.anchor(),
            salience: p.salience(),
            since: p.since(),
            heard: p.heard(),
        });
    }
    positions.sort_by_key(|p| (p.holder, p.policy));
    if positions
        .windows(2)
        .any(|w| (w[0].holder, w[0].policy) == (w[1].holder, w[1].policy))
    {
        return Err(LoadError::Malformed(
            "a person holds two positions on one question".to_owned(),
        ));
    }
    Ok(Opinion {
        positions,
        told: root.told(),
        taken: root.taken(),
    })
}

// ---- word of mouth and grievances (M4c slice AE, ADR-0016) -------------------------------------

fn encode_word(word: &civ_agents::word::Word) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let claims: Vec<_> = word
        .claims
        .iter()
        .map(|c| {
            let (blamed_kind, blamed, issue) = match c.grievance {
                Some((b, i)) => {
                    let (k, id) = b.to_raw();
                    (k, id, i.code())
                }
                None => (255, 0, 0),
            };
            let (place_kind, place, counts) = match c.suspected {
                Some((p, n)) => {
                    let (k, id) = p.code();
                    let counts =
                        fbb.create_vector(&[n.sick_at, n.at, n.sick_elsewhere, n.elsewhere]);
                    (k, id, Some(counts))
                }
                None => (255, 0, None),
            };
            save::ClaimSave::create(
                &mut fbb,
                &save::ClaimSaveArgs {
                    id: c.id,
                    kind: c.kind.code(),
                    settlement: c.settlement.get(),
                    day: c.day,
                    subject: raw(c.subject),
                    blamed_kind,
                    blamed,
                    issue,
                    place_kind,
                    place,
                    counts,
                    fire: c.fire.unwrap_or(0),
                    event: c.event.unwrap_or(0),
                },
            )
        })
        .collect();
    let claims = fbb.create_vector(&claims);
    let heard: Vec<save::HeardSave> = word
        .heard
        .iter()
        .map(|h| {
            save::HeardSave::new(
                h.holder.get(),
                raw(h.from),
                raw(h.origin),
                h.first,
                h.last,
                h.claim,
            )
        })
        .collect();
    let heard = fbb.create_vector(&heard);
    let grievances: Vec<save::GrievanceSave> = word
        .grievances
        .iter()
        .map(|g| {
            let (kind, blamed) = g.blamed.to_raw();
            save::GrievanceSave::new(
                g.holder.get(),
                blamed,
                g.law.get(),
                g.raised,
                g.made,
                g.harm_days,
                g.unresolved_days,
                g.activation,
                g.issue.code(),
                kind,
                g.wrong.code(),
            )
        })
        .collect();
    let grievances = fbb.create_vector(&grievances);
    let suspicions: Vec<save::SuspicionSave> = word
        .suspicions
        .iter()
        .map(|s| {
            let (kind, place) = s.source.code();
            save::SuspicionSave::new(
                s.holder.get(),
                place,
                s.since,
                s.counts.sick_at,
                s.counts.at,
                s.counts.sick_elsewhere,
                s.counts.elsewhere,
                kind,
            )
        })
        .collect();
    let suspicions = fbb.create_vector(&suspicions);
    let root = save::WordSave::create(
        &mut fbb,
        &save::WordSaveArgs {
            claims: Some(claims),
            heard: Some(heard),
            grievances: Some(grievances),
            next: word.next,
            suspicions: Some(suspicions),
        },
    );
    finish(fbb, root)
}

fn decode_word(bytes: &[u8]) -> Result<civ_agents::word::Word, LoadError> {
    use civ_agents::uses::Place;
    use civ_agents::word::{
        Blamed, Claim, ClaimKind, Counts, Grievance, Grieved, Heard, Suspicion, Word, Wrong,
    };
    let root = section_root::<save::WordSave>(bytes).map_err(|e| unreadable(SECTION_WORD, &e))?;
    let bad = |what: String| LoadError::Malformed(what);
    let mut word = Word {
        next: root.next(),
        ..Word::default()
    };
    for c in root.claims().iter().flatten() {
        let kind = ClaimKind::from_code(c.kind())
            .ok_or_else(|| bad(format!("claim {} has kind code {}", c.id(), c.kind())))?;
        let grievance = if c.blamed_kind() == 255 {
            None
        } else {
            let blamed = Blamed::from_raw(c.blamed_kind(), c.blamed())
                .ok_or_else(|| bad(format!("claim {} blames nobody known", c.id())))?;
            let issue = Grieved::from_code(c.issue())
                .ok_or_else(|| bad(format!("claim {} has issue code {}", c.id(), c.issue())))?;
            Some((blamed, issue))
        };
        let suspected = if c.place_kind() == 255 {
            None
        } else {
            let place = Place::from_code(c.place_kind(), c.place())
                .ok_or_else(|| bad(format!("claim {} suspects no place known", c.id())))?;
            let n: Vec<u16> = c.counts().map(|v| v.iter().collect()).unwrap_or_default();
            let [sick_at, at, sick_elsewhere, elsewhere] = n[..] else {
                return Err(bad(format!("claim {} has {} counts of 4", c.id(), n.len())));
            };
            Some((
                place,
                Counts {
                    sick_at,
                    at,
                    sick_elsewhere,
                    elsewhere,
                },
            ))
        };
        word.claims.push(Claim {
            id: c.id(),
            kind,
            settlement: required(c.settlement(), "a claim's settlement")?,
            day: c.day(),
            subject: id(c.subject()),
            grievance,
            suspected,
            fire: (c.fire() != 0).then_some(c.fire()),
            event: (c.event() != 0).then_some(c.event()),
        });
    }
    if !word.claims.windows(2).all(|w| w[0].id < w[1].id) {
        return Err(bad("claims are out of order".to_owned()));
    }
    for h in root.heard().iter().flatten() {
        let heard = Heard {
            holder: required(h.holder(), "a hearer")?,
            claim: h.claim(),
            first: h.first(),
            last: h.last(),
            from: id(h.from()),
            origin: id(h.origin()),
        };
        if !word.heard.push(heard) {
            return Err(bad("what people heard is out of order".to_owned()));
        }
    }
    for g in root.grievances().iter().flatten() {
        word.grievances.push(Grievance {
            holder: required(g.holder(), "a grievance's holder")?,
            issue: Grieved::from_code(g.issue())
                .ok_or_else(|| bad(format!("a grievance has issue code {}", g.issue())))?,
            blamed: Blamed::from_raw(g.blamed_kind(), g.blamed())
                .ok_or_else(|| bad("a grievance blames nobody known".to_owned()))?,
            law: required(g.law(), "a grievance's law")?,
            harm_days: g.harm_days(),
            unresolved_days: g.unresolved_days(),
            activation: g.activation(),
            raised: g.raised(),
            made: g.made(),
            wrong: Wrong::from_code(g.wrong())
                .ok_or_else(|| bad(format!("a grievance has cause code {}", g.wrong())))?,
        });
    }
    for s in root.suspicions().iter().flatten() {
        word.suspicions.push(Suspicion {
            holder: required(s.holder(), "a suspicion's holder")?,
            source: Place::from_code(s.place_kind(), s.place())
                .ok_or_else(|| bad("a suspicion of no place known".to_owned()))?,
            since: s.since(),
            counts: Counts {
                sick_at: s.sick_at(),
                at: s.at(),
                sick_elsewhere: s.sick_elsewhere(),
                elsewhere: s.elsewhere(),
            },
        });
    }
    if !word
        .suspicions
        .windows(2)
        .all(|w| (w[0].holder, w[0].source) < (w[1].holder, w[1].source))
    {
        return Err(bad("suspicions are out of order".to_owned()));
    }
    Ok(word)
}

// ---- norms (M4c slice AG, ADR-0016 §4) ---------------------------------------------------------

fn encode_norms(norms: &civ_agents::norm::Norms, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let states: Vec<save::NormStateSave> = norms
        .states
        .iter()
        .map(|s| {
            save::NormStateSave::new(
                s.holder.get(),
                s.endorse,
                s.expect,
                s.threshold,
                s.heard,
                s.norm,
            )
        })
        .collect();
    let states = fbb.create_vector(&states);
    let ids: Vec<_> = rules
        .catalog
        .norms
        .iter()
        .map(|d| fbb.create_string(&d.id))
        .collect();
    let ids = fbb.create_vector(&ids);
    let acts: Vec<save::LevyActSave> = norms
        .acts
        .iter()
        .map(|a| save::LevyActSave::new(a.household.get(), a.day, a.paid, a.kept))
        .collect();
    let acts = fbb.create_vector(&acts);
    let root = save::NormsSave::create(
        &mut fbb,
        &save::NormsSaveArgs {
            states: Some(states),
            norms: Some(ids),
            acts: Some(acts),
            told: norms.told,
            taken: norms.taken,
        },
    );
    finish(fbb, root)
}

/// States of a norm the loaded content no longer names are let go: everyone takes one afresh if
/// it names it again.
fn decode_norms(bytes: &[u8], rules: &Rules) -> Result<civ_agents::norm::Norms, LoadError> {
    use civ_agents::norm::{LevyAct, NormState, Norms};
    let root = section_root::<save::NormsSave>(bytes).map_err(|e| unreadable(SECTION_NORMS, &e))?;
    let map: Vec<Option<u16>> = read_strings(root.norms())
        .iter()
        .map(|id| {
            rules
                .catalog
                .norms
                .iter()
                .position(|d| &d.id == id)
                .and_then(|i| u16::try_from(i).ok())
        })
        .collect();
    let mut states = Vec::new();
    for s in root.states().iter().flatten() {
        let Some(Some(norm)) = map.get(usize::from(s.norm())) else {
            continue;
        };
        states.push(NormState {
            holder: required(s.holder(), "a norm's holder")?,
            norm: *norm,
            endorse: s.endorse(),
            expect: s.expect(),
            threshold: s.threshold(),
            heard: s.heard(),
        });
    }
    states.sort_by_key(|s| (s.holder, s.norm));
    if states
        .windows(2)
        .any(|w| (w[0].holder, w[0].norm) == (w[1].holder, w[1].norm))
    {
        return Err(LoadError::Malformed(
            "a person holds one norm twice".to_owned(),
        ));
    }
    let mut acts = Vec::new();
    for a in root.acts().iter().flatten() {
        acts.push(LevyAct {
            household: required(a.household(), "a levy act's household")?,
            day: a.day(),
            paid: a.paid(),
            kept: a.kept(),
        });
    }
    acts.sort_by_key(|a| a.household);
    if acts.windows(2).any(|w| w[0].household == w[1].household) {
        return Err(LoadError::Malformed(
            "a household has two levy acts".to_owned(),
        ));
    }
    Ok(Norms {
        states,
        acts,
        told: root.told(),
        taken: root.taken(),
    })
}

// ---- values (M4c slice AG, ADR-0016 §4) --------------------------------------------------------

fn encode_values(values: &civ_agents::values::Values, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let held: Vec<save::HeldValueSave> = values
        .held
        .iter()
        .map(|h| save::HeldValueSave::new(h.holder.get(), h.v, h.value))
        .collect();
    let held = fbb.create_vector(&held);
    let ids: Vec<_> = rules
        .catalog
        .values
        .iter()
        .map(|d| fbb.create_string(&d.id))
        .collect();
    let ids = fbb.create_vector(&ids);
    let root = save::ValuesSave::create(
        &mut fbb,
        &save::ValuesSaveArgs {
            held: Some(held),
            values: Some(ids),
        },
    );
    finish(fbb, root)
}

/// What people held of a value the loaded content no longer names is let go: everyone draws it
/// afresh if it names it again.
fn decode_values(bytes: &[u8], rules: &Rules) -> Result<civ_agents::values::Values, LoadError> {
    use civ_agents::values::{Held, Values};
    let root =
        section_root::<save::ValuesSave>(bytes).map_err(|e| unreadable(SECTION_VALUES, &e))?;
    let map: Vec<Option<u16>> = read_strings(root.values())
        .iter()
        .map(|id| {
            rules
                .catalog
                .values
                .iter()
                .position(|d| &d.id == id)
                .and_then(|i| u16::try_from(i).ok())
        })
        .collect();
    let mut held = Vec::new();
    for h in root.held().iter().flatten() {
        let Some(Some(value)) = map.get(usize::from(h.value())) else {
            continue;
        };
        held.push(Held {
            holder: required(h.holder(), "a value's holder")?,
            value: *value,
            v: h.v(),
        });
    }
    held.sort_by_key(|h| (h.holder, h.value));
    if held
        .windows(2)
        .any(|w| (w[0].holder, w[0].value) == (w[1].holder, w[1].value))
    {
        return Err(LoadError::Malformed(
            "a person holds one value twice".to_owned(),
        ));
    }
    Ok(Values { held })
}

// ---- ideologies (M4c slice AG, ADR-0016 §4) ----------------------------------------------------

fn encode_ideologies(ideas: &civ_agents::ideology::Ideologies, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let held: Vec<save::HoldingSave> = ideas
        .held
        .iter()
        .map(|h| {
            save::HoldingSave::new(
                h.holder.get(),
                h.since,
                h.from.map_or(0, PermanentId::get),
                h.ideology,
            )
        })
        .collect();
    let held = fbb.create_vector(&held);
    let ids: Vec<_> = rules
        .catalog
        .ideologies
        .iter()
        .map(|d| fbb.create_string(&d.id))
        .collect();
    let ids = fbb.create_vector(&ids);
    let creeds: Vec<save::CreedSave> = ideas
        .creeds
        .iter()
        .map(|&(law, k)| save::CreedSave::new(law.get(), k))
        .collect();
    let creeds = fbb.create_vector(&creeds);
    let root = save::IdeologiesSave::create(
        &mut fbb,
        &save::IdeologiesSaveArgs {
            held: Some(held),
            ideologies: Some(ids),
            creeds: Some(creeds),
            told: ideas.told,
            taken: ideas.taken,
            seen: ideas.seen,
        },
    );
    finish(fbb, root)
}

/// Holdings and creeds of an ideology the loaded content no longer names are let go.
fn decode_ideologies(
    bytes: &[u8],
    rules: &Rules,
) -> Result<civ_agents::ideology::Ideologies, LoadError> {
    use civ_agents::ideology::{Holding, Ideologies};
    let root = section_root::<save::IdeologiesSave>(bytes)
        .map_err(|e| unreadable(SECTION_IDEOLOGIES, &e))?;
    let map: Vec<Option<u16>> = read_strings(root.ideologies())
        .iter()
        .map(|id| {
            rules
                .catalog
                .ideologies
                .iter()
                .position(|d| &d.id == id)
                .and_then(|i| u16::try_from(i).ok())
        })
        .collect();
    let mut held = Vec::new();
    for h in root.held().iter().flatten() {
        let Some(Some(ideology)) = map.get(usize::from(h.ideology())) else {
            continue;
        };
        held.push(Holding {
            holder: required(h.holder(), "an ideology's holder")?,
            ideology: *ideology,
            since: h.since(),
            from: PermanentId::from_raw(h.from()),
        });
    }
    held.sort_by_key(|h| (h.holder, h.ideology));
    if held
        .windows(2)
        .any(|w| (w[0].holder, w[0].ideology) == (w[1].holder, w[1].ideology))
    {
        return Err(LoadError::Malformed(
            "a person holds one ideology twice".to_owned(),
        ));
    }
    let mut creeds = Vec::new();
    for c in root.creeds().iter().flatten() {
        let Some(Some(ideology)) = map.get(usize::from(c.ideology())) else {
            continue;
        };
        creeds.push((required(c.law(), "a creed's law")?, *ideology));
    }
    creeds.sort_by_key(|c| c.0);
    if creeds.windows(2).any(|w| w[0].0 == w[1].0) {
        return Err(LoadError::Malformed("a law has two creeds".to_owned()));
    }
    Ok(Ideologies {
        held,
        told: root.told(),
        taken: root.taken(),
        seen: root.seen(),
        creeds,
    })
}

// ---- factions ----------------------------------------------------------------------------------

fn encode_factions(
    factions: &civ_agents::faction::Factions,
    rules: &Rules,
    goods: &[&str],
) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let good_dictionary = strings(&mut fbb, goods);
    let policies: Vec<&str> = rules
        .catalog
        .policies
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    let policy_dictionary = strings(&mut fbb, &policies);
    let petitions: Vec<_> = factions
        .petitions
        .iter()
        .map(|p| {
            let came: Vec<u64> = p.came.iter().map(|c| c.get()).collect();
            let came = fbb.create_vector(&came);
            save::PetitionSave::create(
                &mut fbb,
                &save::PetitionSaveArgs {
                    id: p.id.get(),
                    faction: p.faction.get(),
                    settlement: p.settlement.get(),
                    organizer: p.organizer.get(),
                    called: p.called.minutes(),
                    day: p.day,
                    policy: p.policy,
                    levy_share: p.levy_share,
                    nominee: p.nominee.map_or(0, PermanentId::get),
                    ends: p.ends.get(),
                    came: Some(came),
                    law: p.law.map_or(0, PermanentId::get),
                    answered: p.answered,
                },
            )
        })
        .collect();
    let petitions = fbb.create_vector(&petitions);
    let refusals: Vec<_> = factions
        .refusals
        .iter()
        .map(|r| {
            let kept: Vec<u64> = r.kept.iter().map(|k| k.get()).collect();
            let kept = fbb.create_vector(&kept);
            save::RefusalSave::create(
                &mut fbb,
                &save::RefusalSaveArgs {
                    id: r.id.get(),
                    faction: r.faction.get(),
                    settlement: r.settlement.get(),
                    organizer: r.organizer.get(),
                    called: r.called.minutes(),
                    until: r.until,
                    law: r.law.get(),
                    kept: Some(kept),
                    kept_kg: r.kept_kg,
                },
            )
        })
        .collect();
    let refusals = fbb.create_vector(&refusals);
    let revolts: Vec<_> = factions
        .revolts
        .iter()
        .map(|r| {
            let sides: Vec<save::SideSave> = r
                .sides
                .iter()
                .map(|&(p, s)| save::SideSave::new(p.get(), s.code()))
                .collect();
            let sides = fbb.create_vector(&sides);
            save::RevoltSave::create(
                &mut fbb,
                &save::RevoltSaveArgs {
                    id: r.id.get(),
                    faction: r.faction.get(),
                    settlement: r.settlement.get(),
                    organizer: r.organizer.get(),
                    called: r.called.minutes(),
                    until: r.until,
                    members: r.body.members.code(),
                    seats: Seats::pack(r.body.seats),
                    quorum_share: r.body.quorum_share,
                    pass: r.body.pass.code(),
                    sides: Some(sides),
                    has_held: r.held_since.is_some(),
                    held_since: r.held_since.unwrap_or(0),
                    ended: r.ended.map_or(u8::MAX, |e| e.0.code()),
                    ended_at: r.ended.map_or(0, |e| e.1.minutes()),
                },
            )
        })
        .collect();
    let revolts = fbb.create_vector(&revolts);
    // Coups (schema 46), as revolts with no faction.
    let coups: Vec<_> = factions
        .coups
        .iter()
        .map(|c| {
            let sides: Vec<save::SideSave> = c
                .sides
                .iter()
                .map(|&(p, s)| save::SideSave::new(p.get(), s.code()))
                .collect();
            let sides = fbb.create_vector(&sides);
            save::RevoltSave::create(
                &mut fbb,
                &save::RevoltSaveArgs {
                    id: c.id.get(),
                    faction: 0,
                    settlement: c.settlement.get(),
                    organizer: c.challenger.get(),
                    called: c.called.minutes(),
                    until: c.until,
                    members: c.body.members.code(),
                    seats: Seats::pack(c.body.seats),
                    quorum_share: c.body.quorum_share,
                    pass: c.body.pass.code(),
                    sides: Some(sides),
                    has_held: c.held_since.is_some(),
                    held_since: c.held_since.unwrap_or(0),
                    ended: c.ended.map_or(u8::MAX, |e| e.0.code()),
                    ended_at: c.ended.map_or(0, |e| e.1.minutes()),
                },
            )
        })
        .collect();
    let coups = fbb.create_vector(&coups);
    let list: Vec<_> = factions
        .list
        .iter()
        .map(|f| {
            let mut stores = f.stores.clone();
            stores.resize(goods.len(), 0.0);
            let stores = fbb.create_vector(&stores);
            let history: Vec<save::FactionEventSave> = f
                .history
                .iter()
                .map(|e| {
                    save::FactionEventSave::new(
                        e.at.minutes(),
                        e.who.map_or(0, PermanentId::get),
                        e.kind.code(),
                    )
                })
                .collect();
            let history = fbb.create_vector(&history);
            let (against_kind, against) = f.against.to_raw();
            save::FactionSave::create(
                &mut fbb,
                &save::FactionSaveArgs {
                    id: f.id.get(),
                    settlement: f.settlement.get(),
                    against_kind,
                    against,
                    founder: f.founder.get(),
                    organizer: f.organizer.get(),
                    founded: f.founded.minutes(),
                    has_ended: f.ended.is_some(),
                    ended: f.ended.map_or(0, SimTime::minutes),
                    stores: Some(stores),
                    stores_at: f.stores_at.minutes(),
                    history: Some(history),
                    has_stake: f.stake.is_some(),
                    stake_purpose: f.stake.map_or(0, |s| s.purpose as u8),
                    stake_admit: f.stake.map_or(0, |s| s.admit as u8),
                    stake_decides: f.stake.map_or(0, |s| s.decides as u8),
                    stake_dues: f.stake.map_or(0.0, |s| s.dues_share),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let members: Vec<save::MemberSave> = factions
        .members
        .iter()
        .map(|m| {
            save::MemberSave::new(
                m.person.get(),
                m.faction.get(),
                m.since,
                m.why.grievance,
                m.why.organizer,
                m.why.belong,
                m.why.dues,
                m.why.threshold,
                m.why.known,
            )
        })
        .collect();
    let members = fbb.create_vector(&members);
    let root = save::FactionsSave::create(
        &mut fbb,
        &save::FactionsSaveArgs {
            factions: Some(list),
            members: Some(members),
            goods: Some(good_dictionary),
            joined: factions.joined,
            left: factions.left,
            petitions: Some(petitions),
            policies: Some(policy_dictionary),
            refusals: Some(refusals),
            revolts: Some(revolts),
            coups: Some(coups),
        },
    );
    finish(fbb, root)
}

fn decode_factions(
    bytes: &[u8],
    rules: &Rules,
) -> Result<civ_agents::faction::Factions, LoadError> {
    use civ_agents::faction::{
        Coup, Faction, FactionEvent, FactionEventKind, Factions, Member, Petition, Refusal, Revolt,
        RevoltEnd, Side, Why,
    };
    use civ_agents::polity::{Body, Membership, PassRule};
    let root =
        section_root::<save::FactionsSave>(bytes).map_err(|e| unreadable(SECTION_FACTIONS, &e))?;
    let goods = good_map(&read_strings(root.goods()), rules);
    let bad = |m: String| LoadError::Malformed(m);
    let mut list = Vec::new();
    for f in root.factions().iter().flatten() {
        let id = required(f.id(), "a faction")?;
        let against = civ_agents::word::Blamed::from_raw(f.against_kind(), f.against())
            .ok_or_else(|| bad(format!("faction {id} holds against no party")))?;
        let saved: Vec<f64> = f.stores().map(|v| v.iter().collect()).unwrap_or_default();
        if saved.len() != goods.len() {
            return Err(bad(format!(
                "faction {id} stores {} goods of {}",
                saved.len(),
                goods.len()
            )));
        }
        let mut stores = vec![0.0; rules.catalog.goods.len()];
        for (kg, g) in saved.iter().zip(&goods) {
            if let Some(g) = g {
                stores[*g] += kg;
            }
        }
        let mut history = Vec::new();
        for e in f.history().iter().flatten() {
            let kind = FactionEventKind::from_code(e.kind())
                .ok_or_else(|| bad(format!("faction {id} has an event of kind {}", e.kind())))?;
            history.push(FactionEvent {
                at: SimTime::from_minutes(e.at()),
                kind,
                who: PermanentId::from_raw(e.who()),
            });
        }
        list.push(Faction {
            id,
            settlement: required(f.settlement(), "a faction's settlement")?,
            against,
            founder: required(f.founder(), "a faction's founder")?,
            organizer: required(f.organizer(), "a faction's organizer")?,
            founded: SimTime::from_minutes(f.founded()),
            ended: f.has_ended().then(|| SimTime::from_minutes(f.ended())),
            stores,
            stores_at: SimTime::from_minutes(f.stores_at()),
            flows: Default::default(),
            history,
            stake: if f.has_stake() {
                let purpose = civ_agents::params::GoodUse::ALL
                    .get(usize::from(f.stake_purpose()))
                    .copied()
                    .ok_or_else(|| bad(format!("organization {id} is of no purpose")))?;
                let admit = civ_agents::faction::Admit::from_code(f.stake_admit())
                    .ok_or_else(|| bad(format!("organization {id} admits by no rule")))?;
                let decides = civ_agents::faction::Decides::from_code(f.stake_decides())
                    .ok_or_else(|| bad(format!("organization {id} decides by no rule")))?;
                Some(civ_agents::faction::Stake {
                    purpose,
                    admit,
                    decides,
                    dues_share: f.stake_dues(),
                })
            } else {
                None
            },
        });
    }
    let mut members = Vec::new();
    for m in root.members().iter().flatten() {
        let faction = required(m.faction(), "a member's faction")?;
        if !list
            .iter()
            .any(|f: &Faction| f.id == faction && f.is_live())
        {
            return Err(bad(format!(
                "a member belongs to faction {faction}, which is not live"
            )));
        }
        members.push(Member {
            person: required(m.person(), "a faction's member")?,
            faction,
            since: m.since(),
            why: Why {
                grievance: m.grievance(),
                organizer: m.organizer(),
                belong: m.belong(),
                known: m.known(),
                dues: m.dues(),
                threshold: m.threshold(),
            },
        });
    }
    members.sort_by_key(|m| m.person);
    if members.windows(2).any(|w| w[0].person == w[1].person) {
        return Err(bad("a person belongs to two factions".to_owned()));
    }
    // A petition naming a policy template the loaded content lacks refuses the save, as a law
    // does (ADR-0013 §2).
    let saved_policies = read_strings(root.policies());
    let mut petitions = Vec::new();
    for p in root.petitions().iter().flatten() {
        let id = required(p.id(), "a petition")?;
        let Some(saved) = saved_policies.get(usize::from(p.policy())) else {
            return Err(bad(format!(
                "petition {id} names policy {} of {}",
                p.policy(),
                saved_policies.len()
            )));
        };
        let policy = rules
            .catalog
            .policies
            .iter()
            .position(|d| &d.id == saved)
            .and_then(|i| u16::try_from(i).ok())
            .ok_or_else(|| {
                LoadError::Incompatible(format!(
                    "petition {id} asks for a `{saved}`, which the loaded content does not define"
                ))
            })?;
        let faction = required(p.faction(), "a petition's faction")?;
        if !list.iter().any(|f: &Faction| f.id == faction) {
            return Err(bad(format!(
                "petition {id} was called by faction {faction}, which never was"
            )));
        }
        let mut came = Vec::new();
        for raw in p.came().iter().flatten() {
            came.push(required(raw, "one who came to a petition")?);
        }
        if !came.windows(2).all(|w| w[0] < w[1]) {
            return Err(bad(format!("petition {id} lists who came out of order")));
        }
        petitions.push(Petition {
            id,
            faction,
            settlement: required(p.settlement(), "a petition's settlement")?,
            organizer: required(p.organizer(), "a petition's organizer")?,
            called: SimTime::from_minutes(p.called()),
            day: p.day(),
            policy,
            levy_share: p.levy_share(),
            nominee: PermanentId::from_raw(p.nominee()),
            ends: required(p.ends(), "the law a petition would replace")?,
            came,
            law: PermanentId::from_raw(p.law()),
            answered: p.answered(),
        });
    }
    let mut refusals = Vec::new();
    for r in root.refusals().iter().flatten() {
        let id = required(r.id(), "a refusal")?;
        let faction = required(r.faction(), "a refusal's faction")?;
        if !list.iter().any(|f: &Faction| f.id == faction) {
            return Err(bad(format!(
                "refusal {id} was called by faction {faction}, which never was"
            )));
        }
        let mut kept = Vec::new();
        for raw in r.kept().iter().flatten() {
            kept.push(required(raw, "one who kept back in a refusal")?);
        }
        if !kept.windows(2).all(|w| w[0] < w[1]) {
            return Err(bad(format!(
                "refusal {id} lists who kept back out of order"
            )));
        }
        refusals.push(Refusal {
            id,
            faction,
            settlement: required(r.settlement(), "a refusal's settlement")?,
            organizer: required(r.organizer(), "a refusal's organizer")?,
            called: SimTime::from_minutes(r.called()),
            until: r.until(),
            law: required(r.law(), "the law a refusal keeps back")?,
            kept,
            kept_kg: r.kept_kg(),
        });
    }
    // A revolt's or a coup's body, sides and end.
    type Parts = (Body, Vec<(PermanentId, Side)>, Option<(RevoltEnd, SimTime)>);
    let parts = |r: &save::RevoltSave, id: PermanentId| -> Result<Parts, LoadError> {
        let body = Body {
            members: Membership::from_code(r.members())
                .ok_or_else(|| bad(format!("episode {id} has membership code {}", r.members())))?,
            quorum_share: r.quorum_share(),
            pass: PassRule::from_code(r.pass())
                .ok_or_else(|| bad(format!("episode {id} has decision rule code {}", r.pass())))?,
            // A council's seats (schema 121); before, none.
            seats: Seats::unpack(r.seats()).map_err(|e| bad(format!("episode {id}: {e}")))?,
        };
        let mut sides = Vec::new();
        for s in r.sides().iter().flatten() {
            let side = Side::from_code(s.side())
                .ok_or_else(|| bad(format!("episode {id} has side code {}", s.side())))?;
            sides.push((required(s.person(), "one who stood on an episode")?, side));
        }
        if !sides.windows(2).all(|w| w[0].0 < w[1].0) {
            return Err(bad(format!("episode {id} lists who stood out of order")));
        }
        let ended = match r.ended() {
            u8::MAX => None,
            c => Some((
                RevoltEnd::from_code(c)
                    .ok_or_else(|| bad(format!("episode {id} has end code {c}")))?,
                SimTime::from_minutes(r.ended_at()),
            )),
        };
        Ok((body, sides, ended))
    };
    let mut revolts = Vec::new();
    for r in root.revolts().iter().flatten() {
        let id = required(r.id(), "a revolt")?;
        let faction = required(r.faction(), "a revolt's faction")?;
        if !list.iter().any(|f: &Faction| f.id == faction) {
            return Err(bad(format!(
                "revolt {id} was called by faction {faction}, which never was"
            )));
        }
        let (body, sides, ended) = parts(&r, id)?;
        revolts.push(Revolt {
            id,
            faction,
            settlement: required(r.settlement(), "a revolt's settlement")?,
            organizer: required(r.organizer(), "a revolt's organizer")?,
            called: SimTime::from_minutes(r.called()),
            until: r.until(),
            body,
            sides,
            held_since: r.has_held().then(|| r.held_since()),
            ended,
        });
    }
    let mut coups = Vec::new();
    for c in root.coups().iter().flatten() {
        let id = required(c.id(), "a coup")?;
        let (body, sides, ended) = parts(&c, id)?;
        coups.push(Coup {
            id,
            settlement: required(c.settlement(), "a coup's settlement")?,
            challenger: required(c.organizer(), "a coup's challenger")?,
            called: SimTime::from_minutes(c.called()),
            until: c.until(),
            body,
            sides,
            held_since: c.has_held().then(|| c.held_since()),
            ended,
        });
    }
    Ok(Factions {
        list,
        petitions,
        refusals,
        revolts,
        coups,
        members,
        joined: root.joined(),
        left: root.left(),
    })
}

// ---- the observer's interventions (M4c slice AJ, ADR-0016 §5) -----------------------------------

fn encode_places(people: &Population, goods: &[&str]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    // The goods the price reports and the convergence record name, written only with those to
    // read them by (a world with none saves its places as before schema 56).
    let conv = &people.convergence;
    let good_dictionary = (!people.reports.held.is_empty()
        || !people.reports.errands.is_empty()
        || !conv.gaps.is_empty()
        || !conv.carried.is_empty())
    .then(|| strings(&mut fbb, goods));
    let list: Vec<_> = people
        .known_places
        .known
        .iter()
        .flat_map(|(&h, list)| list.iter().map(move |k| (h, k)))
        .map(|(h, k)| {
            save::KnownPlaceSave::create(
                &mut fbb,
                &save::KnownPlaceSaveArgs {
                    household: h.get(),
                    settlement: k.settlement.get(),
                    how: k.how.code(),
                    from: k.from.map_or(0, PermanentId::get),
                    first: k.first,
                    last: k.last,
                    food: k.food.unwrap_or(-1.0),
                    visited: k.visited.unwrap_or(-1),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let contacts: Vec<_> = people
        .contacts
        .years
        .iter()
        .map(|(&(year, from, to), c)| {
            let missed = c
                .missed
                .iter()
                .any(|&n| n > 0)
                .then(|| fbb.create_vector(&c.missed));
            save::ContactSave::create(
                &mut fbb,
                &save::ContactSaveArgs {
                    year,
                    from: from.get(),
                    to: to.get(),
                    visits: c.visits,
                    minutes: c.minutes,
                    marriages: c.marriages,
                    moved: c.moved,
                    bought: c.bought,
                    missed,
                },
            )
        })
        .collect();
    let contacts = fbb.create_vector(&contacts);
    let unmatched: Vec<_> = people
        .unmatched
        .iter()
        .map(|(&person, &day)| {
            save::UnmatchedSave::create(
                &mut fbb,
                &save::UnmatchedSaveArgs {
                    person: person.get(),
                    day,
                },
            )
        })
        .collect();
    let unmatched = fbb.create_vector(&unmatched);
    let leanings: Vec<_> = people
        .leanings
        .iter()
        .map(|(&household, l)| {
            save::LeaningSave::create(
                &mut fbb,
                &save::LeaningSaveArgs {
                    household: household.get(),
                    settlement: l.settlement.get(),
                    reviews: l.reviews,
                    day: l.day,
                },
            )
        })
        .collect();
    let leanings = fbb.create_vector(&leanings);
    let due: Vec<u64> = people.review_due.iter().map(|h| h.get()).collect();
    let due = fbb.create_vector(&due);
    let coalitions: Vec<_> = people
        .coalitions
        .iter()
        .map(|c| {
            let members: Vec<u64> = c.members.iter().map(|h| h.get()).collect();
            let members = fbb.create_vector(&members);
            save::CoalitionSave::create(
                &mut fbb,
                &save::CoalitionSaveArgs {
                    id: c.id,
                    organizer: c.organizer.get(),
                    from: c.from.get(),
                    site: Some(&point(c.site)),
                    formed: c.formed,
                    reviews: c.reviews,
                    members: Some(members),
                    people: c.people,
                    lacking: c.lacking as u8,
                    fate: c.fate as u8,
                    ended: c.ended.unwrap_or(-1),
                    settlement: c.settlement.map_or(0, |s| s.get()),
                    named: c.named.map_or(0, |p| p.get()),
                },
            )
        })
        .collect();
    let coalitions = fbb.create_vector(&coalitions);
    let reports: Vec<_> = people
        .reports
        .held
        .iter()
        .flat_map(|(&h, list)| list.iter().map(move |r| (h, r)))
        .map(|(h, r)| {
            save::PriceReportSave::create(
                &mut fbb,
                &save::PriceReportSaveArgs {
                    household: h.get(),
                    market: r.market.get(),
                    seller: r.seller.get(),
                    firm: r.firm,
                    good: u32::from(r.good),
                    payment: u32::from(r.payment),
                    price: r.price,
                    ask_h: r.ask_h,
                    units: r.units,
                    day: r.day,
                    how: r.how.code(),
                    from: r.from.map_or(0, PermanentId::get),
                },
            )
        })
        .collect();
    let reports = (!reports.is_empty()).then(|| fbb.create_vector(&reports));
    // The convergence record (schema 57).
    let gaps: Vec<_> = conv
        .gaps
        .iter()
        .flat_map(|(&(month, a, b), list)| list.iter().map(move |g| (month, a, b, g)))
        .map(|(month, a, b, g)| {
            save::GapSave::create(
                &mut fbb,
                &save::GapSaveArgs {
                    month,
                    a: a.get(),
                    b: b.get(),
                    good: u32::from(g.good),
                    ask_a: g.ask_h[0],
                    ask_b: g.ask_h[1],
                    paid_a: g.paid_h[0],
                    paid_b: g.paid_h[1],
                },
            )
        })
        .collect();
    let gaps = (!gaps.is_empty()).then(|| fbb.create_vector(&gaps));
    let carried: Vec<_> = conv
        .carried
        .iter()
        .map(|(&(month, from, to), c)| {
            let ids: Vec<u32> = c.goods.iter().map(|&(g, _)| u32::from(g)).collect();
            let units: Vec<f32> = c.goods.iter().map(|&(_, u)| u).collect();
            let ids = fbb.create_vector(&ids);
            let units = fbb.create_vector(&units);
            save::CarriedSave::create(
                &mut fbb,
                &save::CarriedSaveArgs {
                    month,
                    from: from.get(),
                    to: to.get(),
                    trips: c.trips,
                    walk_h: c.walk_h,
                    goods: Some(ids),
                    units: Some(units),
                },
            )
        })
        .collect();
    let carried = (!carried.is_empty()).then(|| fbb.create_vector(&carried));
    // Errands to fetch goods to sell at home (schema 58).
    let errands: Vec<_> = people
        .reports
        .errands
        .iter()
        .map(|(&h, e)| {
            save::ErrandSave::create(
                &mut fbb,
                &save::ErrandSaveArgs {
                    household: h.get(),
                    market: e.market.get(),
                    seller: e.seller.get(),
                    firm: e.firm,
                    good: u32::from(e.good),
                    payment: u32::from(e.payment),
                    units: e.units,
                    home_h: e.home_h,
                    share: e.share,
                    day: e.day,
                    by_boat: e.by_boat.is_some(),
                    boat: e.by_boat.map_or(0, |b| b.boat.get()),
                    boat_from: e.by_boat.map_or(0, |b| b.from),
                    boat_to: e.by_boat.map_or(0, |b| b.to),
                    travel_min: e
                        .by_boat
                        .map(|b| b.travel_min)
                        .or(e.by_cart.map(|c| c.travel_min))
                        .unwrap_or(0.0),
                    load_min: e
                        .by_boat
                        .map(|b| b.load_min)
                        .or(e.by_cart.map(|c| c.load_min))
                        .unwrap_or(0.0),
                    unload_min: e
                        .by_boat
                        .map(|b| b.unload_min)
                        .or(e.by_cart.map(|c| c.unload_min))
                        .unwrap_or(0.0),
                    by_cart: e.by_cart.is_some(),
                    cart: e.by_cart.map_or(0, |c| c.cart.get()),
                },
            )
        })
        .collect();
    let errands = (!errands.is_empty()).then(|| fbb.create_vector(&errands));
    // Buildings seen in other settlements (schema 59).
    let seen: Vec<_> = people
        .seen_away
        .iter()
        .map(|(&p, list)| {
            let buildings: Vec<u64> = list.iter().map(|b| b.get()).collect();
            let buildings = fbb.create_vector(&buildings);
            save::SeenSave::create(
                &mut fbb,
                &save::SeenSaveArgs {
                    person: p.get(),
                    buildings: Some(buildings),
                },
            )
        })
        .collect();
    let seen = (!seen.is_empty()).then(|| fbb.create_vector(&seen));
    // Settlements' founding ways (schema 61).
    let ways: Vec<_> = people
        .founding_ways
        .iter()
        .map(|(&id, t)| {
            save::WaySave::create(
                &mut fbb,
                &save::WaySaveArgs {
                    settlement: id.get(),
                    pitch_centideg: t.pitch_centideg,
                    eave_cm: t.eave_cm,
                    overhang_cm: t.overhang_cm,
                    openings_permille: t.openings_permille,
                    ornament_dh: t.ornament_dh,
                    finish: t.finish,
                    symmetry: t.symmetry,
                },
            )
        })
        .collect();
    let ways = (!ways.is_empty()).then(|| fbb.create_vector(&ways));
    // The places households' people work, and the work not yet folded in (schema 62).
    let uses: Vec<_> = people
        .uses
        .households
        .iter()
        .flat_map(|(&h, list)| list.iter().map(move |u| (h, u)))
        .map(|(h, u)| {
            let outsiders: Vec<save::OutsidersSave> = u
                .outsiders
                .iter()
                .map(|o| save::OutsidersSave::new(o.settlement.get(), o.days, o.last))
                .collect();
            let outsiders = (!outsiders.is_empty()).then(|| fbb.create_vector(&outsiders));
            // Households seen drawing water there (schema 79).
            let seen: Vec<save::SeenAtSave> = u
                .seen
                .iter()
                .map(|x| save::SeenAtSave::new(x.household.get(), x.days, x.last))
                .collect();
            let seen = (!seen.is_empty()).then(|| fbb.create_vector(&seen));
            let (kind, place) = u.place.code();
            save::PlaceUseSave::create(
                &mut fbb,
                &save::PlaceUseSaveArgs {
                    household: h.get(),
                    kind,
                    place,
                    days: u.days,
                    kcal: u.kcal,
                    day: u.day,
                    outsiders,
                    seen,
                    recent: u.recent,
                },
            )
        })
        .collect();
    let uses = (!uses.is_empty()).then(|| fbb.create_vector(&uses));
    let worked: Vec<save::WorkedSave> = people
        .uses
        .today
        .iter()
        .map(|w| {
            let (kind, place) = w.place.code();
            save::WorkedSave::new(
                kind,
                place,
                w.day,
                w.household.get(),
                w.settlement.get(),
                w.kcal,
            )
        })
        .collect();
    let worked = (!worked.is_empty()).then(|| fbb.create_vector(&worked));
    let worked_by: Vec<u64> = people
        .uses
        .today
        .iter()
        .map(|w| w.person.map_or(0, PermanentId::get))
        .collect();
    let worked_by = (!worked_by.is_empty()).then(|| fbb.create_vector(&worked_by));
    let draws = &people.spring_draws;
    let draw_patches: Vec<u32> = draws.drawn.iter().map(|d| d.0).collect();
    let draw_litres: Vec<f64> = draws.drawn.iter().map(|d| d.1).collect();
    let draw_patches = (!draw_patches.is_empty()).then(|| fbb.create_vector(&draw_patches));
    let draw_litres = (!draw_litres.is_empty()).then(|| fbb.create_vector(&draw_litres));
    let root = save::PlacesSave::create(
        &mut fbb,
        &save::PlacesSaveArgs {
            list: Some(list),
            contacts: Some(contacts),
            unmatched: Some(unmatched),
            leanings: Some(leanings),
            review_due: Some(due),
            coalitions: Some(coalitions),
            reports,
            goods: good_dictionary,
            gaps,
            carried,
            errands,
            seen,
            ways,
            uses,
            worked,
            worked_by,
            draws_day: draws.day,
            draw_patches,
            draw_litres,
        },
    );
    finish(fbb, root)
}

/// What the places section holds: the places households know, from schema 52 the contacts
/// between settlements and the failed searches for a partner, and from schema 53 the leanings
/// toward moving and the reviews prompted.
struct PlacesDecoded {
    known: civ_agents::places::Places,
    contacts: civ_agents::places::Contacts,
    unmatched: BTreeMap<PermanentId, i64>,
    leanings: BTreeMap<PermanentId, civ_agents::places::Leaning>,
    due: std::collections::BTreeSet<PermanentId>,
    coalitions: Vec<civ_agents::places::Coalition>,
    reports: civ_agents::reports::PriceReports,
    convergence: civ_agents::convergence::Convergence,
    seen: BTreeMap<PermanentId, Vec<PermanentId>>,
    ways: BTreeMap<PermanentId, civ_agents::params::Taste>,
    uses: civ_agents::uses::Uses,
    draws: civ_agents::uses::SpringDraws,
}

fn decode_places(bytes: &[u8], rules: &Rules) -> Result<PlacesDecoded, LoadError> {
    use civ_agents::places::{Contact, Contacts, KnownPlace, PlaceHow, Places};
    let root =
        section_root::<save::PlacesSave>(bytes).map_err(|e| unreadable(SECTION_PLACES, &e))?;
    let mut contacts = Contacts::default();
    for c in root.contacts().iter().flatten() {
        let (Some(from), Some(to)) = (
            PermanentId::from_raw(c.from()),
            PermanentId::from_raw(c.to()),
        ) else {
            return Err(LoadError::Malformed(
                "a contact with no settlement".to_owned(),
            ));
        };
        contacts.years.insert(
            (c.year(), from, to),
            Contact {
                visits: c.visits(),
                minutes: c.minutes(),
                marriages: c.marriages(),
                moved: c.moved(),
                bought: c.bought(),
                missed: {
                    use civ_agents::reports::Missed;
                    let mut m = [0u32; Missed::COUNT];
                    for (i, n) in c.missed().iter().flatten().take(Missed::COUNT).enumerate() {
                        m[i] = n;
                    }
                    m
                },
            },
        );
    }
    let mut unmatched = BTreeMap::new();
    for u in root.unmatched().iter().flatten() {
        let person = PermanentId::from_raw(u.person())
            .ok_or_else(|| LoadError::Malformed("a search for a partner by no one".to_owned()))?;
        unmatched.insert(person, u.day());
    }
    let mut leanings = BTreeMap::new();
    for l in root.leanings().iter().flatten() {
        let (Some(h), Some(s)) = (
            PermanentId::from_raw(l.household()),
            PermanentId::from_raw(l.settlement()),
        ) else {
            return Err(LoadError::Malformed(
                "a leaning with no household or place".to_owned(),
            ));
        };
        leanings.insert(
            h,
            civ_agents::places::Leaning {
                settlement: s,
                reviews: l.reviews(),
                day: l.day(),
            },
        );
    }
    let due = root
        .review_due()
        .iter()
        .flatten()
        .filter_map(PermanentId::from_raw)
        .collect();
    let mut coalitions = Vec::new();
    for c in root.coalitions().iter().flatten() {
        use civ_agents::places::{Coalition, CoalitionFate, Lacking};
        let site = c.site().map_or((0.0, 0.0), |p| (p.x(), p.y()));
        let mut members = Vec::new();
        for h in c.members().iter().flatten() {
            members.push(required(h, "a coalition's household")?);
        }
        coalitions.push(Coalition {
            id: c.id(),
            organizer: required(c.organizer(), "a coalition's organizer")?,
            named: PermanentId::from_raw(c.named()),
            from: required(c.from(), "a coalition's settlement")?,
            site,
            formed: c.formed(),
            reviews: c.reviews(),
            members,
            people: c.people(),
            lacking: Lacking::from_code(c.lacking())
                .ok_or_else(|| LoadError::Malformed(format!("unknown lack {}", c.lacking())))?,
            fate: CoalitionFate::from_code(c.fate()).ok_or_else(|| {
                LoadError::Malformed(format!("unknown fate of a coalition {}", c.fate()))
            })?,
            ended: (c.ended() >= 0).then_some(c.ended()),
            settlement: PermanentId::from_raw(c.settlement()),
        });
    }
    if coalitions.windows(2).any(|w| w[0].id >= w[1].id) {
        return Err(LoadError::Malformed("coalitions out of order".to_owned()));
    }
    // Price reports (schema 56): a report of a good the content no longer has is forgotten.
    let mut reports = civ_agents::reports::PriceReports::default();
    let report_goods = good_map(&read_strings(root.goods()), rules);
    for r in root.reports().iter().flatten() {
        use civ_agents::reports::{PriceReport, ReportHow};
        let household = required(r.household(), "a price report's household")?;
        let what = || format!("a price report of household {household}");
        let good = saved_good(&report_goods, r.good(), what)?;
        let payment = saved_good(&report_goods, r.payment(), what)?;
        let (Some(good), Some(payment)) = (good, payment) else {
            continue;
        };
        reports.note(
            household,
            PriceReport {
                market: required(r.market(), "a price report's market")?,
                seller: required(r.seller(), "a price report's seller")?,
                firm: r.firm(),
                good,
                payment,
                price: r.price(),
                ask_h: r.ask_h(),
                units: r.units(),
                day: r.day(),
                how: ReportHow::from_code(r.how()).ok_or_else(|| {
                    LoadError::Malformed(format!("unknown way of a price report {}", r.how()))
                })?,
                from: PermanentId::from_raw(r.from()),
            },
        );
    }
    // Errands (schema 58): one for a good the content no longer has is let go.
    for e in root.errands().iter().flatten() {
        let household = required(e.household(), "an errand's household")?;
        let what = || format!("an errand of household {household}");
        let good = saved_good(&report_goods, e.good(), what)?;
        let payment = saved_good(&report_goods, e.payment(), what)?;
        let (Some(good), Some(payment)) = (good, payment) else {
            continue;
        };
        reports.errands.insert(
            household,
            civ_agents::reports::Errand {
                market: required(e.market(), "an errand's market")?,
                seller: required(e.seller(), "an errand's seller")?,
                firm: e.firm(),
                good,
                payment,
                units: e.units(),
                home_h: e.home_h(),
                share: e.share(),
                day: e.day(),
                by_boat: if e.by_boat() {
                    Some(civ_agents::reports::ByBoat {
                        boat: required(e.boat(), "an errand's boat")?,
                        from: e.boat_from(),
                        to: e.boat_to(),
                        travel_min: e.travel_min(),
                        load_min: e.load_min(),
                        unload_min: e.unload_min(),
                    })
                } else {
                    None
                },
                by_cart: if e.by_cart() {
                    Some(civ_agents::reports::ByCart {
                        cart: required(e.cart(), "an errand's cart")?,
                        travel_min: e.travel_min(),
                        load_min: e.load_min(),
                        unload_min: e.unload_min(),
                    })
                } else {
                    None
                },
            },
        );
    }
    let mut out = Places::default();
    for k in root.list().iter().flatten() {
        let id = |raw: u64, what: &str| {
            PermanentId::from_raw(raw)
                .ok_or_else(|| LoadError::Malformed(format!("a known place with no {what}")))
        };
        let how = PlaceHow::from_code(k.how()).ok_or_else(|| {
            LoadError::Malformed(format!("unknown way of knowing a place {}", k.how()))
        })?;
        let list = out
            .known
            .entry(id(k.household(), "household")?)
            .or_default();
        let settlement = id(k.settlement(), "settlement")?;
        if list.last().is_some_and(|x| x.settlement >= settlement) {
            return Err(LoadError::Malformed("known places out of order".to_owned()));
        }
        list.push(KnownPlace {
            settlement,
            how,
            from: PermanentId::from_raw(k.from()),
            first: k.first(),
            last: k.last(),
            food: (k.food() >= 0.0).then_some(k.food()),
            visited: (k.visited() >= 0).then_some(k.visited()),
        });
    }
    // The convergence record (schema 57): what names a good the content no longer has is let go.
    let mut convergence = civ_agents::convergence::Convergence::default();
    for g in root.gaps().iter().flatten() {
        let what = || format!("a month {} gap", g.month());
        let Some(good) = saved_good(&report_goods, g.good(), what)? else {
            continue;
        };
        let pair = (
            g.month(),
            required(g.a(), "a gap's settlement")?,
            required(g.b(), "a gap's settlement")?,
        );
        convergence
            .gaps
            .entry(pair)
            .or_default()
            .push(civ_agents::convergence::GoodGap {
                good,
                ask_h: [g.ask_a(), g.ask_b()],
                paid_h: [g.paid_a(), g.paid_b()],
            });
    }
    for list in convergence.gaps.values_mut() {
        list.sort_by_key(|g| g.good);
    }
    for c in root.carried().iter().flatten() {
        let ids: Vec<u32> = c.goods().map(|v| v.iter().collect()).unwrap_or_default();
        let units: Vec<f32> = c.units().map(|v| v.iter().collect()).unwrap_or_default();
        if ids.len() != units.len() {
            return Err(LoadError::Malformed(
                "goods carried without their units".to_owned(),
            ));
        }
        let mut goods = Vec::new();
        for (i, u) in ids.into_iter().zip(units) {
            let what = || format!("month {} goods carried", c.month());
            if let Some(g) = saved_good(&report_goods, i, what)? {
                goods.push((g, u));
            }
        }
        goods.sort_by_key(|&(g, _)| g);
        convergence.carried.insert(
            (
                c.month(),
                required(c.from(), "a trip's settlement")?,
                required(c.to(), "a trip's settlement")?,
            ),
            civ_agents::convergence::Carried {
                trips: c.trips(),
                walk_h: c.walk_h(),
                goods,
            },
        );
    }
    // Buildings seen in other settlements (schema 59).
    let mut seen = BTreeMap::new();
    for x in root.seen().iter().flatten() {
        let person = required(x.person(), "a person who saw buildings elsewhere")?;
        let mut list = Vec::new();
        for b in x.buildings().iter().flatten() {
            list.push(required(b, "a building seen elsewhere")?);
        }
        let mut ids = list.clone();
        ids.sort_unstable();
        ids.dedup();
        if list.is_empty() || ids.len() != list.len() || seen.insert(person, list).is_some() {
            return Err(LoadError::Malformed(format!(
                "person {person} saw an empty or repeated list of buildings elsewhere"
            )));
        }
    }
    // Settlements' founding ways (schema 61).
    let mut ways = BTreeMap::new();
    for w in root.ways().iter().flatten() {
        let id = required(w.settlement(), "a founding way's settlement")?;
        ways.insert(
            id,
            civ_agents::params::Taste {
                pitch_centideg: w.pitch_centideg(),
                eave_cm: w.eave_cm(),
                overhang_cm: w.overhang_cm(),
                openings_permille: w.openings_permille(),
                ornament_dh: w.ornament_dh(),
                finish: w.finish(),
                symmetry: w.symmetry(),
            },
        );
    }
    // The places households' people work, and the work not yet folded in (schema 62).
    let mut uses = civ_agents::uses::Uses::default();
    let place_of = |kind: u8, id: u64| {
        civ_agents::uses::Place::from_code(kind, id).ok_or_else(|| {
            LoadError::Malformed(format!("a place worked has kind {kind} and number {id}"))
        })
    };
    for u in root.uses().iter().flatten() {
        let household = required(u.household(), "a household that works a place")?;
        let mut outsiders = Vec::new();
        for o in u.outsiders().iter().flatten() {
            outsiders.push(civ_agents::uses::Outsiders {
                settlement: required(o.settlement(), "a settlement seen at a place")?,
                days: o.days(),
                last: o.last(),
            });
        }
        let mut seen = Vec::new();
        for x in u.seen().iter().flatten() {
            seen.push(civ_agents::uses::SeenAt {
                household: required(x.household(), "a household seen drawing water")?,
                days: x.days(),
                last: x.last(),
            });
        }
        uses.households
            .entry(household)
            .or_default()
            .push(civ_agents::uses::PlaceUse {
                place: place_of(u.kind(), u.place())?,
                days: u.days(),
                kcal: u.kcal(),
                outsiders,
                seen,
                day: u.day(),
                // Which days were worked there, exactly (schema 86); before, none known.
                recent: u.recent(),
            });
    }
    // Who did each piece of work (schema 63); before, nobody known.
    let by: Vec<u64> = root
        .worked_by()
        .map(|v| v.iter().collect())
        .unwrap_or_default();
    for (k, w) in root.worked().iter().flatten().enumerate() {
        uses.today.push(civ_agents::uses::Worked {
            place: place_of(w.kind(), w.place())?,
            day: w.day(),
            household: required(w.household(), "a household that worked a place")?,
            settlement: required(w.settlement(), "a settlement whose people worked a place")?,
            kcal: w.kcal(),
            person: by.get(k).copied().and_then(PermanentId::from_raw),
        });
    }
    // What was drawn today at each spring (schema 73); before, nothing.
    let patches: Vec<u32> = root
        .draw_patches()
        .map(|v| v.iter().collect())
        .unwrap_or_default();
    let litres: Vec<f64> = root
        .draw_litres()
        .map(|v| v.iter().collect())
        .unwrap_or_default();
    if patches.len() != litres.len()
        || patches.windows(2).any(|w| w[0] >= w[1])
        || litres.iter().any(|l| !(l.is_finite() && *l >= 0.0))
    {
        return Err(LoadError::Malformed(
            "section `places` has malformed draws at springs".to_owned(),
        ));
    }
    let draws = civ_agents::uses::SpringDraws {
        day: root.draws_day(),
        drawn: patches.into_iter().zip(litres).collect(),
    };
    Ok(PlacesDecoded {
        draws,
        known: out,
        contacts,
        unmatched,
        leanings,
        due,
        coalitions,
        reports,
        convergence,
        seen,
        ways,
        uses,
    })
}

// ---- relations ---------------------------------------------------------------------------------

fn encode_relations(people: &Population, goods: &[&str]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let views: Vec<_> = people
        .polity_views
        .held
        .iter()
        .flat_map(|(&p, list)| list.iter().map(move |v| (p, v)))
        .map(|(p, v)| {
            let yes = fbb.create_vector(&v.yes);
            let no = fbb.create_vector(&v.no);
            save::ViewSave::create(
                &mut fbb,
                &save::ViewSaveArgs {
                    person: p.get(),
                    polity: v.polity.get(),
                    day: v.day,
                    yes: Some(yes),
                    no: Some(no),
                    reason_act: v.reason.map_or(u8::MAX, |r| r.act.code()),
                    reason_day: v.reason.map_or(0, |r| r.day),
                    reason_times: v.reason.map_or(0, |r| r.times),
                },
            )
        })
        .collect();
    let views = (!views.is_empty()).then(|| fbb.create_vector(&views));
    // The claims of other polities households heard of (schema 64).
    let heard: Vec<_> = people
        .claims_heard
        .households
        .iter()
        .flat_map(|(&h, list)| list.iter().map(move |c| (h, c)))
        .map(|(h, c)| {
            let (kind, place) = c.place.code();
            save::HeardClaimSave::create(
                &mut fbb,
                &save::HeardClaimSaveArgs {
                    household: h.get(),
                    kind,
                    place,
                    law: c.law.get(),
                    polity: c.polity.get(),
                    day: c.day,
                    from: c.from.get(),
                },
            )
        })
        .collect();
    let heard = (!heard.is_empty()).then(|| fbb.create_vector(&heard));
    // Agreements between polities (schema 65).
    let agreements: Vec<_> = people
        .agreements
        .list
        .iter()
        .map(|a| {
            use civ_agents::agreements::AgreementState;
            let (kinds, sides): (Vec<u8>, Vec<u8>) = a.clauses.iter().map(|c| c.code()).unzip();
            let clause_kind = fbb.create_vector(&kinds);
            let clause_side = fbb.create_vector(&sides);
            // What each clause gives (schema 67), only when one moves goods.
            let (clause_good, clause_kg, clause_days) = if a.clauses.iter().any(|c| c.moves_goods())
            {
                let amounts: Vec<(u16, u32, u32)> = a.clauses.iter().map(|c| c.amount()).collect();
                let good: Vec<u16> = amounts.iter().map(|x| x.0).collect();
                let kg: Vec<u32> = amounts.iter().map(|x| x.1).collect();
                let days: Vec<u32> = amounts.iter().map(|x| x.2).collect();
                (
                    Some(fbb.create_vector(&good)),
                    Some(fbb.create_vector(&kg)),
                    Some(fbb.create_vector(&days)),
                )
            } else {
                (None, None, None)
            };
            let (state, state_day, (why_kind, why_side), by) = match a.state {
                AgreementState::Offered => (0, 0, (0, 0), u8::MAX),
                AgreementState::InForce { since } => (1, since, (0, 0), u8::MAX),
                AgreementState::Failed { day, why } => (2, day, why.code(), u8::MAX),
                AgreementState::Ended { day, by } => (3, day, (0, 0), by.unwrap_or(u8::MAX)),
            };
            let law = |s: usize| a.laws[s].map_or(0, PermanentId::get);
            save::AgreementSave::create(
                &mut fbb,
                &save::AgreementSaveArgs {
                    id: a.id.get(),
                    polity_a: a.polities[0].get(),
                    polity_b: a.polities[1].get(),
                    negotiator_a: a.negotiators[0].get(),
                    negotiator_b: a.negotiators[1].get(),
                    clause_kind: Some(clause_kind),
                    clause_side: Some(clause_side),
                    term_days: a.term_days,
                    made: a.made,
                    law_a: law(0),
                    law_b: law(1),
                    passed_a: a.passed[0].unwrap_or(-1),
                    passed_b: a.passed[1].unwrap_or(-1),
                    heard_a: a.heard[0].unwrap_or(-1),
                    heard_b: a.heard[1].unwrap_or(-1),
                    state,
                    state_day,
                    why_kind,
                    why_side,
                    by,
                    clause_good,
                    clause_kg,
                    clause_days,
                },
            )
        })
        .collect();
    let agreements = (!agreements.is_empty()).then(|| fbb.create_vector(&agreements));
    // The payments agreements owe (schema 67).
    let dues: Vec<_> = people
        .agreements
        .dues
        .iter()
        .map(|d| {
            use civ_agents::agreements::DueState;
            let (state, state_day, why) = match d.state {
                DueState::Open => (0, 0, 0),
                DueState::Met { day } => (1, day, 0),
                DueState::Missed { day, why } => (2, day, why.code()),
            };
            save::DueSave::create(
                &mut fbb,
                &save::DueSaveArgs {
                    id: d.id.get(),
                    agreement: d.agreement.get(),
                    clause: d.clause,
                    from_polity: d.from.get(),
                    to_polity: d.to.get(),
                    good: d.good,
                    owed_kg: d.owed_kg,
                    made: d.made,
                    due: d.due,
                    set_aside_kg: d.set_aside_kg,
                    arrived_kg: d.arrived_kg,
                    carrier: d.carrier.map_or(0, PermanentId::get),
                    state,
                    state_day,
                    why,
                    held_kg: d.stores.get(usize::from(d.good)).copied().unwrap_or(0.0),
                    held_at: d.stores_at.minutes(),
                },
            )
        })
        .collect();
    let names_goods = !dues.is_empty()
        || people
            .agreements
            .list
            .iter()
            .any(|a| a.clauses.iter().any(|c| c.moves_goods()));
    let dues = (!dues.is_empty()).then(|| fbb.create_vector(&dues));
    let goods = names_goods.then(|| strings(&mut fbb, goods));
    let root = save::RelationsSave::create(
        &mut fbb,
        &save::RelationsSaveArgs {
            views,
            heard,
            agreements,
            dues,
            goods,
        },
    );
    finish(fbb, root)
}

type Relations = (
    civ_agents::views::Views,
    civ_agents::uses::ClaimsHeard,
    civ_agents::agreements::Agreements,
);

fn decode_relations(bytes: &[u8], rules: &Rules) -> Result<Relations, LoadError> {
    use civ_agents::views::{DOMAINS, View, ViewAct, ViewReason, Views};
    let root = section_root::<save::RelationsSave>(bytes)
        .map_err(|e| unreadable(SECTION_RELATIONS, &e))?;
    let prior = rules.people.relations.prior as f32;
    let mut out = Views::default();
    for v in root.views().iter().flatten() {
        let person = required(v.person(), "a person who holds a view of a polity")?;
        let polity = required(v.polity(), "a polity someone holds a view of")?;
        // Domains a later build appended are dropped; those it lacks start at the prior.
        let counts = |saved: Option<flatbuffers::Vector<'_, f32>>| {
            let mut out = [prior; DOMAINS];
            for (k, x) in saved.iter().flatten().take(DOMAINS).enumerate() {
                out[k] = x;
            }
            out
        };
        let reason = match v.reason_act() {
            u8::MAX => None,
            code => Some(ViewReason {
                act: ViewAct::from_code(code).ok_or_else(|| {
                    LoadError::Malformed(format!("person {person}'s view has act code {code}"))
                })?,
                day: v.reason_day(),
                times: v.reason_times(),
            }),
        };
        let list = out.held.entry(person).or_default();
        if list.last().is_some_and(|last: &View| last.polity >= polity) {
            return Err(LoadError::Malformed(format!(
                "person {person}'s views are out of order or repeated"
            )));
        }
        list.push(View {
            polity,
            day: v.day(),
            yes: counts(v.yes()),
            no: counts(v.no()),
            reason,
        });
    }
    // The claims of other polities households heard of (schema 64); before, none.
    let mut heard = civ_agents::uses::ClaimsHeard::default();
    for c in root.heard().iter().flatten() {
        let household = required(c.household(), "a household that heard of a claim")?;
        let place = civ_agents::uses::Place::from_code(c.kind(), c.place()).ok_or_else(|| {
            LoadError::Malformed(format!(
                "household {household} heard of a claim on place {} of kind {}",
                c.place(),
                c.kind()
            ))
        })?;
        let claim = civ_agents::uses::HeardClaim {
            place,
            law: required(c.law(), "the law of a claim heard of")?,
            polity: required(c.polity(), "the polity of a claim heard of")?,
            day: c.day(),
            from: required(c.from(), "who told of a claim")?,
        };
        if !heard.learn(household, claim) {
            return Err(LoadError::Malformed(format!(
                "household {household} heard of one claim twice"
            )));
        }
    }
    // Agreements between polities (schema 65); before, none. What a clause gives names a good of
    // the saved dictionary (schema 67); a clause or payment of a good the loaded content no longer
    // has is dropped, and the payments of an agreement's later clauses renumbered.
    let goods = good_map(&read_strings(root.goods()), rules);
    let good_of =
        |index: u16, what: &dyn Fn() -> String| saved_good(&goods, u32::from(index), what);
    let mut agreements = civ_agents::agreements::Agreements::default();
    let mut clause_maps: HashMap<PermanentId, Vec<Option<u8>>> = HashMap::new();
    for a in root.agreements().iter().flatten() {
        use civ_agents::agreements::{Agreement, AgreementState, Clause, Failure};
        let id = required(a.id(), "an agreement")?;
        let bad = |what: &str| LoadError::Malformed(format!("agreement {id} has {what}"));
        let kinds: Vec<u8> = a
            .clause_kind()
            .map(|v| v.iter().collect())
            .unwrap_or_default();
        let sides: Vec<u8> = a
            .clause_side()
            .map(|v| v.iter().collect())
            .unwrap_or_default();
        if kinds.len() != sides.len() {
            return Err(bad("clause kinds and sides of different lengths"));
        }
        let amounts = |v: Option<flatbuffers::Vector<'_, u32>>| -> Result<Vec<u32>, LoadError> {
            match v {
                None => Ok(vec![0; kinds.len()]),
                Some(v) if v.len() == kinds.len() => Ok(v.iter().collect()),
                Some(_) => Err(bad("clause amounts and kinds of different lengths")),
            }
        };
        let saved_goods: Vec<u16> = match a.clause_good() {
            None => vec![0; kinds.len()],
            Some(v) if v.len() == kinds.len() => v.iter().collect(),
            Some(_) => return Err(bad("clause goods and kinds of different lengths")),
        };
        let (kgs, days) = (amounts(a.clause_kg())?, amounts(a.clause_days())?);
        let mut clauses = Vec::with_capacity(kinds.len());
        let mut clause_map = Vec::with_capacity(kinds.len());
        for (k, (&kind, &side)) in kinds.iter().zip(&sides).enumerate() {
            // Leave, words against raiding (kind 3, schema 117) and one body (kind 4, schema
            // 119) name no good.
            let good = if kind == 0 || kind == 3 || kind == 4 {
                Some(0)
            } else {
                good_of(saved_goods[k], &|| format!("agreement {id}'s clause {k}"))?
            };
            let Some(good) = good else {
                clause_map.push(None);
                continue;
            };
            clause_map.push(Some(clauses.len() as u8));
            clauses.push(
                Clause::from_code(kind, side, good, kgs[k], days[k])
                    .ok_or_else(|| bad(&format!("a clause of kind {kind} and side {side}")))?,
            );
        }
        clause_maps.insert(id, clause_map);
        let day = |d: i64| (d >= 0).then_some(d);
        let state = match a.state() {
            0 => AgreementState::Offered,
            1 => AgreementState::InForce {
                since: a.state_day(),
            },
            2 => AgreementState::Failed {
                day: a.state_day(),
                why: Failure::from_code(a.why_kind(), a.why_side())
                    .ok_or_else(|| bad("an unknown failure"))?,
            },
            3 => AgreementState::Ended {
                day: a.state_day(),
                by: match a.by() {
                    u8::MAX => None,
                    s @ (0 | 1) => Some(s),
                    _ => return Err(bad("an ending by an unknown side")),
                },
            },
            c => return Err(bad(&format!("state code {c}"))),
        };
        agreements.list.push(Agreement {
            id,
            polities: [
                required(a.polity_a(), "an agreement's polity")?,
                required(a.polity_b(), "an agreement's polity")?,
            ],
            negotiators: [
                required(a.negotiator_a(), "an agreement's negotiator")?,
                required(a.negotiator_b(), "an agreement's negotiator")?,
            ],
            clauses,
            term_days: a.term_days(),
            made: a.made(),
            laws: [
                PermanentId::from_raw(a.law_a()),
                PermanentId::from_raw(a.law_b()),
            ],
            passed: [day(a.passed_a()), day(a.passed_b())],
            heard: [day(a.heard_a()), day(a.heard_b())],
            state,
        });
    }
    // The payments agreements owe (schema 67); before, none.
    for d in root.dues().iter().flatten() {
        use civ_agents::agreements::{Due, DueState, Miss};
        let id = required(d.id(), "a payment an agreement owes")?;
        let bad = |what: &str| LoadError::Malformed(format!("payment {id} has {what}"));
        let agreement = required(d.agreement(), "the agreement a payment is owed under")?;
        let clause = match clause_maps.get(&agreement) {
            None => return Err(bad(&format!("agreement {agreement}, which is not kept"))),
            Some(map) => match map.get(usize::from(d.clause())) {
                None => return Err(bad(&format!("clause {}, which it lacks", d.clause()))),
                Some(None) => continue,
                Some(&Some(c)) => c,
            },
        };
        let Some(good) = good_of(d.good(), &|| format!("payment {id}"))? else {
            continue;
        };
        let state = match d.state() {
            0 => DueState::Open,
            1 => DueState::Met { day: d.state_day() },
            2 => DueState::Missed {
                day: d.state_day(),
                why: Miss::from_code(d.why()).ok_or_else(|| bad("an unknown miss"))?,
            },
            c => return Err(bad(&format!("state code {c}"))),
        };
        let mut stores = vec![0.0; rules.catalog.goods.len()];
        stores[usize::from(good)] = d.held_kg();
        agreements.dues.push(Due {
            id,
            agreement,
            clause,
            from: required(d.from_polity(), "the polity a payment is owed by")?,
            to: required(d.to_polity(), "the polity a payment is owed to")?,
            good,
            owed_kg: d.owed_kg(),
            made: d.made(),
            due: d.due(),
            set_aside_kg: d.set_aside_kg(),
            arrived_kg: d.arrived_kg(),
            carrier: PermanentId::from_raw(d.carrier()),
            state,
            stores,
            stores_at: SimTime::from_minutes(d.held_at()),
            flows: Default::default(),
        });
    }
    Ok((out, heard, agreements))
}

// ---- crossings ---------------------------------------------------------------------------------

fn encode_crossings(land: &Land, fords: &civ_agents::fords::Fords, rules: &Rules) -> Vec<u8> {
    use civ_land::crossings::{CrossingOwner, CrossingState};
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = land
        .crossings
        .list
        .iter()
        .map(|c| {
            let system = rules
                .catalog
                .bridges
                .get(usize::from(c.system))
                .map_or("", |d| d.id.as_str());
            let system = fbb.create_string(system);
            let cells = fbb.create_vector(&c.cells);
            let (owner_kind, owner) = match c.owner {
                CrossingOwner::Household(h) => (0, h.get()),
                CrossingOwner::Polity(p) => (1, p.get()),
            };
            let shares: Vec<save::CrossingShareSave> = c
                .shares
                .iter()
                .map(|&(h, hours)| save::CrossingShareSave::new(h.get(), hours))
                .collect();
            let shares = (!shares.is_empty()).then(|| fbb.create_vector(&shares));
            let (state, work_h, state_day, why) = match c.state {
                CrossingState::Building { work_h } => (0, work_h, 0, 0),
                CrossingState::Open { since } => (1, 0.0, since, 0),
                CrossingState::Failed { day, why } => (2, 0.0, day, why.code()),
            };
            save::CrossingSave::create(
                &mut fbb,
                &save::CrossingSaveArgs {
                    id: c.id.get(),
                    system: Some(system),
                    bank_a: c.banks[0],
                    bank_b: c.banks[1],
                    cells: Some(cells),
                    span_m: c.span_m,
                    members: c.members,
                    diameter_cm: c.diameter_cm,
                    quality: c.quality,
                    loss: c.loss,
                    owner_kind,
                    owner,
                    labour_h: c.labour_h,
                    begun: c.begun.minutes(),
                    state,
                    work_h,
                    state_day,
                    why,
                    skill_h: c.skill_h,
                    shares,
                    crew_day: c.crew.0,
                    crew: c.crew.1,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let wades: Vec<save::WadeSave> = fords
        .households
        .iter()
        .flat_map(|(&h, list)| {
            list.iter()
                .map(move |w| save::WadeSave::new(h.get(), w.day, w.cell, w.wades))
        })
        .collect();
    let wades = (!wades.is_empty()).then(|| fbb.create_vector(&wades));
    let enclosures: Vec<_> = land
        .enclosures
        .list
        .iter()
        .map(|e| {
            let sectors: Vec<_> = e
                .sectors
                .iter()
                .map(|s| {
                    let cells = fbb.create_vector(&s.cells);
                    save::EnclosureSectorSave::create(
                        &mut fbb,
                        &save::EnclosureSectorSaveArgs {
                            ax: s.a.0,
                            ay: s.a.1,
                            bx: s.b.0,
                            by: s.b.1,
                            cells: Some(cells),
                            labour_h: s.labour_h,
                            gate: s.gate.map_or(0, |g| if g.open { 1 } else { 2 }),
                            loss: s.loss,
                        },
                    )
                })
                .collect();
            let sectors = fbb.create_vector(&sectors);
            let shares: Vec<save::CrossingShareSave> = e
                .shares
                .iter()
                .map(|&(h, hours)| save::CrossingShareSave::new(h.get(), hours))
                .collect();
            let shares = (!shares.is_empty()).then(|| fbb.create_vector(&shares));
            save::EnclosureSave::create(
                &mut fbb,
                &save::EnclosureSaveArgs {
                    id: e.id.get(),
                    owner: e.owner.get(),
                    law: e.law.get(),
                    settlement: e.settlement.get(),
                    centre: Some(&point(e.centre)),
                    radius_m: e.radius_m,
                    sectors: Some(sectors),
                    work_h: e.work_h,
                    shares,
                    crew_day: e.crew.0,
                    crew: e.crew.1,
                    begun: e.begun.minutes(),
                    finished: e.finished.unwrap_or(-1),
                    upkeep_h: e.upkeep_h,
                },
            )
        })
        .collect();
    let enclosures = (!enclosures.is_empty()).then(|| fbb.create_vector(&enclosures));
    let root = save::CrossingsSave::create(
        &mut fbb,
        &save::CrossingsSaveArgs {
            list: Some(list),
            revision: land.crossings.revision(),
            wades,
            enclosures,
            enclosures_revision: land.enclosures.revision(),
        },
    );
    finish(fbb, root)
}

/// The crossings saved in `bytes`, and the streams households wade. A crossing of a bridge
/// system the loaded content no longer has is dropped; one spanning cells off the map is
/// malformed, as is a wade of a cell off the map.
#[allow(clippy::type_complexity)]
fn decode_crossings(
    bytes: &[u8],
    rules: &Rules,
    map: &WorldMap,
) -> Result<
    (
        civ_land::crossings::Crossings,
        civ_agents::fords::Fords,
        civ_land::enclosures::Enclosures,
    ),
    LoadError,
> {
    use civ_land::crossings::{Collapse, Crossing, CrossingOwner, CrossingState, Crossings};
    let root = section_root::<save::CrossingsSave>(bytes)
        .map_err(|e| unreadable(SECTION_CROSSINGS, &e))?;
    let cells_on_map = u64::from(map.width) * u64::from(map.height);
    let mut list = Vec::new();
    for c in root.list().iter().flatten() {
        let id = required(c.id(), "a crossing")?;
        let bad = |what: &str| LoadError::Malformed(format!("crossing {id} has {what}"));
        let Some(system) = rules.catalog.bridge_index(c.system().unwrap_or("")) else {
            continue;
        };
        let cells: Vec<u32> = c.cells().map(|v| v.iter().collect()).unwrap_or_default();
        if cells.is_empty()
            || cells
                .iter()
                .chain([c.bank_a(), c.bank_b()].iter())
                .any(|&x| u64::from(x) >= cells_on_map)
        {
            return Err(bad("cells off the map, or none"));
        }
        let owner = required(c.owner(), "a crossing's owner")?;
        let owner = match c.owner_kind() {
            0 => CrossingOwner::Household(owner),
            1 => CrossingOwner::Polity(owner),
            k => return Err(bad(&format!("owner kind {k}"))),
        };
        let state = match c.state() {
            0 => CrossingState::Building { work_h: c.work_h() },
            1 => CrossingState::Open {
                since: c.state_day(),
            },
            2 => CrossingState::Failed {
                day: c.state_day(),
                why: Collapse::from_code(c.why()).ok_or_else(|| bad("an unknown collapse"))?,
            },
            s => return Err(bad(&format!("state code {s}"))),
        };
        // The work each household gave (schema 70), by household id; before, none.
        let mut shares = Vec::new();
        for s in c.shares().iter().flatten() {
            let household = required(s.household(), "a share's household")?;
            shares.push((household, s.hours()));
        }
        shares.sort_unstable_by_key(|s: &(PermanentId, f32)| s.0);
        shares.dedup_by_key(|s| s.0);
        list.push(Crossing {
            id,
            system: system as u16,
            banks: [c.bank_a(), c.bank_b()],
            cells,
            span_m: c.span_m(),
            members: c.members(),
            diameter_cm: c.diameter_cm(),
            quality: c.quality(),
            loss: c.loss(),
            owner,
            labour_h: c.labour_h(),
            skill_h: c.skill_h(),
            begun: SimTime::from_minutes(c.begun()),
            state,
            shares,
            crew: (c.crew_day(), c.crew()),
        });
    }
    let mut fords = civ_agents::fords::Fords::default();
    for w in root.wades().iter().flatten() {
        let household = required(w.household(), "a wade's household")?;
        if u64::from(w.cell()) >= cells_on_map {
            return Err(LoadError::Malformed(format!(
                "household {household} waded cell {} off the map",
                w.cell()
            )));
        }
        fords
            .households
            .entry(household)
            .or_default()
            .push(civ_agents::fords::Wade {
                cell: w.cell(),
                wades: w.wades(),
                day: w.day(),
            });
    }
    // Enclosures (schema 113); before, none.
    let mut enclosures = Vec::new();
    for e in root.enclosures().iter().flatten() {
        use civ_land::enclosures::{Enclosure, Gate, Sector};
        let id = required(e.id(), "an enclosure")?;
        let bad = |what: &str| LoadError::Malformed(format!("enclosure {id} has {what}"));
        let mut sectors = Vec::new();
        for s in e.sectors().iter().flatten() {
            let cells: Vec<u32> = s.cells().map(|v| v.iter().collect()).unwrap_or_default();
            if cells.iter().any(|&x| u64::from(x) >= cells_on_map) {
                return Err(bad("a sector off the map"));
            }
            let gate = match s.gate() {
                0 => None,
                1 => Some(Gate { open: true }),
                2 => Some(Gate { open: false }),
                g => return Err(bad(&format!("gate code {g}"))),
            };
            sectors.push(Sector {
                a: (s.ax(), s.ay()),
                b: (s.bx(), s.by()),
                cells,
                labour_h: s.labour_h(),
                gate,
                loss: s.loss(),
            });
        }
        let mut shares = Vec::new();
        for s in e.shares().iter().flatten() {
            shares.push((required(s.household(), "a share's household")?, s.hours()));
        }
        enclosures.push(Enclosure {
            id,
            owner: required(e.owner(), "an enclosure's polity")?,
            law: required(e.law(), "an enclosure's law")?,
            settlement: required(e.settlement(), "an enclosure's settlement")?,
            centre: e.centre().map_or((0.0, 0.0), |p| (p.x(), p.y())),
            radius_m: e.radius_m(),
            sectors,
            work_h: e.work_h(),
            shares,
            crew: (e.crew_day(), e.crew()),
            begun: SimTime::from_minutes(e.begun()),
            finished: (e.finished() >= 0).then_some(e.finished()),
            upkeep_h: e.upkeep_h(),
        });
    }
    let enclosures =
        civ_land::enclosures::Enclosures::with_revision(enclosures, root.enclosures_revision());
    let problems = enclosures.problems();
    if let Some(p) = problems.first() {
        return Err(LoadError::Malformed(p.clone()));
    }
    Ok((
        Crossings::with_revision(list, root.revision()),
        fords,
        enclosures,
    ))
}

fn encode_wells(land: &Land, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = land
        .wells
        .list
        .iter()
        .map(|w| {
            let system = rules
                .catalog
                .wells
                .get(w.system)
                .map_or("", |d| d.id.as_str());
            let system = fbb.create_string(system);
            let (state, state_at) = w.state.code();
            save::WellSave::create(
                &mut fbb,
                &save::WellSaveArgs {
                    id: w.id.get(),
                    system: Some(system),
                    cell: w.cell,
                    x_cm: w.rect.x,
                    y_cm: w.rect.y,
                    w_cm: w.rect.w,
                    h_cm: w.rect.h,
                    household: w.household.get(),
                    ground_m: w.ground_m,
                    depth_m: w.depth_m,
                    target_m: w.target_m,
                    work_h: w.work_h,
                    skill_h: w.skill_h,
                    quality: w.quality,
                    loss: w.loss,
                    mend_h: w.mend_h,
                    worth: w.worth,
                    level_m: w.level_m,
                    level_at: w.level_at.minutes(),
                    drawn_l: w.drawn_l,
                    begun: w.begun.minutes(),
                    state,
                    state_at: state_at.minutes(),
                    crew_day: w.crew.0,
                    crew: w.crew.1,
                    cover_h: w.cover_h,
                    covered: w.covered,
                    polity: w.polity.map_or(0, PermanentId::get),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let shares: Vec<save::WellShareSave> = land
        .wells
        .shares
        .iter()
        .map(|&(well, household, hours)| {
            save::WellShareSave::new(well.get(), household.get(), hours)
        })
        .collect();
    let shares = (!shares.is_empty()).then(|| fbb.create_vector(&shares));
    let root = save::WellsSave::create(
        &mut fbb,
        &save::WellsSaveArgs {
            list: Some(list),
            shares,
        },
    );
    finish(fbb, root)
}

/// The traffic section (schema 141).
fn encode_traffic(
    t: &civ_agents::traffic::Traffic,
    travel: &civ_agents::travel::Travel,
) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let servers: Vec<_> = t
        .servers
        .iter()
        .map(|s| {
            let (kind, id, sector) = s.facility.code();
            let slots = fbb.create_vector(&s.slots);
            save::TrafficServerSave::create(
                &mut fbb,
                &save::TrafficServerSaveArgs {
                    kind,
                    id,
                    sector,
                    dir: s.dir,
                    slots: Some(slots),
                },
            )
        })
        .collect();
    let servers = fbb.create_vector(&servers);
    let tallies: Vec<_> = t
        .tallies
        .iter()
        .map(|x| {
            let (kind, id, sector) = x.facility.code();
            let passages = fbb.create_vector(&x.passages);
            let waited = fbb.create_vector(&x.waited_min);
            save::TrafficTallySave::create(
                &mut fbb,
                &save::TrafficTallySaveArgs {
                    kind,
                    id,
                    sector,
                    day: x.day,
                    passages: Some(passages),
                    waited_min: Some(waited),
                    held_short: x.held_short,
                    longest_min: x.longest_min,
                    longest_day: x.longest_day,
                },
            )
        })
        .collect();
    let tallies = fbb.create_vector(&tallies);
    let waits: Vec<save::TrafficWaitSave> = t
        .waits
        .iter()
        .flat_map(|(h, list)| {
            list.iter().map(move |w| {
                let (kind, id, sector) = w.facility.code();
                save::TrafficWaitSave::new(h.get(), id, w.day, w.minutes, w.waits, sector, kind)
            })
        })
        .collect();
    let waits = fbb.create_vector(&waits);
    let long_waits: Vec<save::TrafficLongWaitSave> = t
        .long_waits
        .iter()
        .map(|w| {
            let (kind, id, sector) = w.facility.code();
            save::TrafficLongWaitSave::new(
                w.person.get(),
                w.household.get(),
                w.settlement.map_or(0, PermanentId::get),
                id,
                w.time.minutes(),
                w.id,
                w.at.0,
                w.at.1,
                w.minutes,
                w.ahead,
                sector,
                kind,
            )
        })
        .collect();
    let long_waits = (!long_waits.is_empty()).then(|| fbb.create_vector(&long_waits));
    let losses: Vec<save::TrafficLossSave> = t
        .losses
        .iter()
        .flat_map(|(h, list)| {
            list.iter().map(move |l| {
                save::TrafficLossSave::new(h.get(), l.day, l.cell, l.hours, l.times, l.cause.code())
            })
        })
        .collect();
    let losses = (!losses.is_empty()).then(|| fbb.create_vector(&losses));
    // Travel measured (schema 152).
    let rows = |map: &mut dyn Iterator<Item = (u8, civ_agents::travel::Tally)>| {
        map.map(|(code, t)| save::TravelTallySave::new(t.minutes, t.trips, code))
            .collect::<Vec<_>>()
    };
    let years: Vec<_> = travel
        .years
        .iter()
        .map(|(&(year, settlement), y)| {
            let purposes = rows(&mut y.purposes.iter().map(|(&c, &t)| (c, t)));
            let modes = rows(&mut y.modes.iter().enumerate().map(|(c, &t)| (c as u8, t)));
            let days: Vec<u32> = y.days.iter().flatten().copied().collect();
            let purposes = fbb.create_vector(&purposes);
            let modes = fbb.create_vector(&modes);
            let days = fbb.create_vector(&days);
            let minutes = fbb.create_vector(&y.minutes);
            save::TravelYearSave::create(
                &mut fbb,
                &save::TravelYearSaveArgs {
                    year,
                    settlement: settlement.get(),
                    purposes: Some(purposes),
                    modes: Some(modes),
                    waits_trips: y.waits.trips,
                    waits_minutes: y.waits.minutes,
                    days: Some(days),
                    minutes: Some(minutes),
                },
            )
        })
        .collect();
    let travel_years = (!years.is_empty()).then(|| fbb.create_vector(&years));
    let today: Vec<save::TravelTodaySave> = travel
        .today
        .iter()
        .map(|(p, &m)| save::TravelTodaySave::new(p.get(), m))
        .collect();
    let travel_today = (!today.is_empty()).then(|| fbb.create_vector(&today));
    let households: Vec<save::TravelHouseholdSave> = travel
        .households
        .iter()
        .map(|(h, t)| save::TravelHouseholdSave::new(t.minutes, t.days, h.get(), t.day))
        .collect();
    let travel_households = (!households.is_empty()).then(|| fbb.create_vector(&households));
    let goings: Vec<save::TravelGoingSave> = travel
        .goings
        .iter()
        .flat_map(|(p, g)| {
            g.to.iter()
                .map(|&(place, trips)| save::TravelGoingSave::new(p.get(), g.day, place, trips))
        })
        .collect();
    let travel_goings = (!goings.is_empty()).then(|| fbb.create_vector(&goings));
    let today: Vec<save::TravelGoingTodaySave> = travel
        .going_today
        .iter()
        .map(|&(p, place)| save::TravelGoingTodaySave::new(p.get(), place))
        .collect();
    let travel_going_today = (!today.is_empty()).then(|| fbb.create_vector(&today));
    let root = save::TrafficSave::create(
        &mut fbb,
        &save::TrafficSaveArgs {
            servers: Some(servers),
            tallies: Some(tallies),
            waits: Some(waits),
            long_waits,
            losses,
            travel_years,
            travel_today,
            travel_households,
            travel_goings,
            travel_going_today,
        },
    );
    finish(fbb, root)
}

/// The turn-taking saved in `bytes`. A facility of an unknown kind, a tally without its 24
/// hours, or a record out of order is malformed.
fn decode_traffic(
    bytes: &[u8],
) -> Result<(civ_agents::traffic::Traffic, civ_agents::travel::Travel), LoadError> {
    use civ_agents::traffic::{Facility, Server, Tally, Traffic, WaitHeld};
    let root =
        section_root::<save::TrafficSave>(bytes).map_err(|e| unreadable(SECTION_TRAFFIC, &e))?;
    let facility = |kind: u8, id: u64, sector: u16| {
        Facility::from_code(kind, id, sector).ok_or_else(|| {
            LoadError::Malformed(format!("a bottleneck of kind {kind} numbered {id}"))
        })
    };
    let hours = |v: Option<flatbuffers::Vector<'_, f32>>| -> Result<[f32; 24], LoadError> {
        let v: Vec<f32> = v.map(|v| v.iter().collect()).unwrap_or_default();
        v.try_into().map_err(|_| {
            LoadError::Malformed("a bottleneck's tally is not by hour of the day".to_owned())
        })
    };
    let mut t = Traffic::default();
    for s in root.servers().iter().flatten() {
        t.servers.push(Server {
            facility: facility(s.kind(), s.id(), s.sector())?,
            dir: s.dir(),
            slots: s.slots().map(|v| v.iter().collect()).unwrap_or_default(),
        });
    }
    for x in root.tallies().iter().flatten() {
        t.tallies.push(Tally {
            facility: facility(x.kind(), x.id(), x.sector())?,
            day: x.day(),
            passages: hours(x.passages())?,
            waited_min: hours(x.waited_min())?,
            held_short: x.held_short(),
            longest_min: x.longest_min(),
            longest_day: x.longest_day(),
        });
    }
    for w in root.waits().iter().flatten() {
        let household = required(w.household(), "a wait's household")?;
        t.waits.entry(household).or_default().push(WaitHeld {
            facility: facility(w.kind(), w.id(), w.sector())?,
            minutes: w.minutes(),
            waits: w.waits(),
            day: w.day(),
        });
    }
    for w in root.long_waits().iter().flatten() {
        t.long_waits.push(civ_agents::traffic::LongWait {
            id: w.wait_id(),
            person: required(w.person(), "a long wait's walker")?,
            household: required(w.household(), "a long wait's household")?,
            settlement: PermanentId::from_raw(w.settlement()),
            facility: facility(w.kind(), w.id(), w.sector())?,
            at: (w.x(), w.y()),
            time: time(w.time()),
            minutes: w.minutes(),
            ahead: w.ahead(),
        });
    }
    for l in root.losses().iter().flatten() {
        let household = required(l.household(), "a way loss's household")?;
        let cause = civ_agents::traffic::Cause::from_code(l.cause())
            .ok_or_else(|| LoadError::Malformed(format!("a way loss of cause {}", l.cause())))?;
        t.losses
            .entry(household)
            .or_default()
            .push(civ_agents::traffic::Loss {
                cell: l.cell(),
                cause,
                hours: l.hours(),
                times: l.times(),
                day: l.day(),
            });
    }
    if let Some(p) = t.problems().into_iter().next() {
        return Err(LoadError::Malformed(p));
    }
    // Travel measured (schema 152); before, none.
    let mut travel = civ_agents::travel::Travel::default();
    for y in root.travel_years().iter().flatten() {
        let settlement = required(y.settlement(), "a year of travel's settlement")?;
        let mut year = civ_agents::travel::TravelYear::default();
        for r in y.purposes().iter().flatten() {
            year.purposes.insert(
                r.code(),
                civ_agents::travel::Tally {
                    trips: r.trips(),
                    minutes: r.minutes(),
                },
            );
        }
        let modes: Vec<_> = y.modes().iter().flatten().collect();
        let days: Vec<u32> = y.days().map(|v| v.iter().collect()).unwrap_or_default();
        let minutes: Vec<f64> = y.minutes().map(|v| v.iter().collect()).unwrap_or_default();
        if modes.len() != year.modes.len()
            || days.len() != year.days.len() * civ_agents::travel::BANDS
            || minutes.len() != year.minutes.len()
        {
            return Err(LoadError::Malformed(format!(
                "settlement {settlement}'s year {} of travel is not by mode, occupation and band",
                y.year()
            )));
        }
        for (slot, r) in year.modes.iter_mut().zip(&modes) {
            *slot = civ_agents::travel::Tally {
                trips: r.trips(),
                minutes: r.minutes(),
            };
        }
        for (row, chunk) in year
            .days
            .iter_mut()
            .zip(days.chunks(civ_agents::travel::BANDS))
        {
            row.copy_from_slice(chunk);
        }
        year.minutes.copy_from_slice(&minutes);
        year.waits = civ_agents::travel::Tally {
            trips: y.waits_trips(),
            minutes: y.waits_minutes(),
        };
        travel.years.insert((y.year(), settlement), year);
    }
    for d in root.travel_today().iter().flatten() {
        let person = required(d.person(), "a day of travel's walker")?;
        travel.today.insert(person, d.minutes());
    }
    for h in root.travel_households().iter().flatten() {
        let household = required(h.household(), "a household's days of travel")?;
        travel.households.insert(
            household,
            civ_agents::travel::HouseholdTravel {
                minutes: h.minutes(),
                days: h.days(),
                day: h.day(),
            },
        );
    }
    for g in root.travel_goings().iter().flatten() {
        let person = required(g.person(), "where someone goes from home")?;
        let goings = travel
            .goings
            .entry(person)
            .or_insert(civ_agents::travel::Goings {
                day: g.day(),
                to: Vec::new(),
            });
        if goings.day != g.day() {
            return Err(LoadError::Malformed(format!(
                "where person {person} goes from home is held as of two days"
            )));
        }
        goings.to.push((g.place(), g.trips()));
    }
    for g in root.travel_going_today().iter().flatten() {
        let person = required(g.person(), "a trip from home today")?;
        travel.going_today.push((person, g.place()));
    }
    if let Some(p) = travel.problems().into_iter().next() {
        return Err(LoadError::Malformed(p));
    }
    Ok((t, travel))
}

/// The ways section (schema 140).
fn encode_ways(land: &Land) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = land
        .ways
        .list
        .iter()
        .map(|w| {
            let segments: Vec<_> = w
                .segments
                .iter()
                .map(|s| {
                    let cells = fbb.create_vector(&s.cells);
                    let (keeper, keeper_id) = s.keeper.code();
                    save::WaySegmentSave::create(
                        &mut fbb,
                        &save::WaySegmentSaveArgs {
                            cells: Some(cells),
                            corridor_m: s.corridor_m,
                            built_m: s.built_m,
                            usable_m: s.usable_m,
                            surface: s.surface.code(),
                            keeper,
                            keeper_id,
                            kept: s.kept,
                        },
                    )
                })
                .collect();
            let segments = fbb.create_vector(&segments);
            let track = w.track.as_ref().map(|t| {
                let bed = fbb.create_vector(&t.bed);
                save::WayTrackSave::create(
                    &mut fbb,
                    &save::WayTrackSaveArgs {
                        gauge_m: t.gauge_m,
                        bed: Some(bed),
                        cut_m3: t.cut_m3,
                        fill_m3: t.fill_m3,
                        good_kg: t.good_kg,
                        labour_h: t.labour_h,
                    },
                )
            });
            save::WayLineSave::create(
                &mut fbb,
                &save::WayLineSaveArgs {
                    id: w.id.get(),
                    begun: w.begun,
                    segments: Some(segments),
                    modes: w.modes,
                    trips: w.trips,
                    trips_day: w.trips_day,
                    rank: w.rank.code(),
                    track,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::WaysSave::create(
        &mut fbb,
        &save::WaysSaveArgs {
            list: Some(list),
            revision: land.ways.revision(),
        },
    );
    finish(fbb, root)
}

fn encode_consignments(people: &Population, rules: &Rules) -> Vec<u8> {
    let goods = &rules.catalog.goods;
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = people
        .consignments
        .iter()
        .map(|c| {
            let held: Vec<_> = c
                .stores
                .iter()
                .enumerate()
                .filter(|&(_, &x)| x != 0.0)
                .filter_map(|(g, &amount)| {
                    let good = fbb.create_string(&goods.get(g)?.id);
                    Some(save::ConsignedGoodSave::create(
                        &mut fbb,
                        &save::ConsignedGoodSaveArgs {
                            good: Some(good),
                            amount,
                        },
                    ))
                })
                .collect();
            let held = fbb.create_vector(&held);
            save::ConsignmentSave::create(
                &mut fbb,
                &save::ConsignmentSaveArgs {
                    id: c.id.get(),
                    owner: c.owner.get(),
                    carrier: c.carrier.get(),
                    boat: c.boat.get(),
                    from: c.from.get(),
                    to: c.to.get(),
                    made: c.made.minutes(),
                    stores_at: c.stores_at.minutes(),
                    goods: Some(held),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root =
        save::ConsignmentsSave::create(&mut fbb, &save::ConsignmentsSaveArgs { list: Some(list) });
    finish(fbb, root)
}

/// The loads in transit saved in `bytes`. A good the loaded content no longer has is let go.
fn decode_consignments(
    bytes: &[u8],
    rules: &Rules,
) -> Result<Vec<civ_agents::consignments::Consignment>, LoadError> {
    use civ_agents::consignments::Consignment;
    let goods = &rules.catalog.goods;
    let root = section_root::<save::ConsignmentsSave>(bytes)
        .map_err(|e| unreadable(SECTION_CONSIGNMENTS, &e))?;
    let mut out = Vec::new();
    for c in root.list().iter().flatten() {
        let id = required(c.id(), "a load in transit")?;
        let mut load = Consignment::new(
            id,
            (
                required(c.owner(), "a load's owner")?,
                required(c.carrier(), "a load's carrier")?,
                required(c.boat(), "a load's boat")?,
            ),
            (
                required(c.from(), "a load's origin")?,
                required(c.to(), "a load's destination")?,
            ),
            SimTime::from_minutes(c.made()),
            goods.len(),
        );
        load.stores_at = SimTime::from_minutes(c.stores_at());
        for g in c.goods().iter().flatten() {
            let name = g.good().unwrap_or("");
            if let Some(i) = goods.iter().position(|d| d.id == name) {
                load.stores[i] = g.amount();
            }
        }
        out.push(load);
    }
    Ok(out)
}

fn encode_boats(boats: &civ_land::boats::Boats, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = boats
        .list
        .iter()
        .map(|b| {
            let vehicle = rules
                .catalog
                .vehicles
                .get(usize::from(b.vehicle))
                .map_or("", |v| v.id.as_str());
            let vehicle = fbb.create_string(vehicle);
            save::BoatSave::create(
                &mut fbb,
                &save::BoatSaveArgs {
                    id: b.id.get(),
                    vehicle: Some(vehicle),
                    owner: b.owner.get(),
                    at: b.at,
                    made: b.made,
                    wear: b.wear,
                    labour_h: b.labour_h,
                    work_h: b.work_h,
                    saves_h_year: b.saves_h_year,
                    crew_day: b.crew.0,
                    crew: b.crew.1,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::BoatsSave::create(
        &mut fbb,
        &save::BoatsSaveArgs {
            list: Some(list),
            revision: boats.revision(),
        },
    );
    finish(fbb, root)
}

/// The boats, or carts, saved in `bytes` of section `tag`. One of a vehicle the loaded content
/// no longer has is dropped; one at a cell off the map, or out of order, is malformed.
/// Lines' passing loops and train runs (schema 182, M10b slice DW).
fn encode_runs(runs: &civ_land::trains::Runs) -> Vec<u8> {
    use civ_land::trains::RunState;
    let mut fbb = FlatBufferBuilder::new();
    let lines: Vec<_> = runs
        .lines
        .iter()
        .map(|l| {
            let loops = fbb.create_vector(&l.loops);
            save::RailLineSave::create(
                &mut fbb,
                &save::RailLineSaveArgs {
                    way: l.way.get(),
                    loops: Some(loops),
                    stations: l.stations,
                    runs_a_day: l.runs_a_day,
                    freight_kg: l.freight_kg,
                    freight_since: l.freight_since,
                    riders: l.riders,
                    fare_kg: l.fare_kg,
                },
            )
        })
        .collect();
    let list: Vec<_> = runs
        .list
        .iter()
        .map(|r| {
            let sections: Vec<u16> = r.legs.iter().map(|l| l.0).collect();
            let minutes: Vec<u32> = r.legs.iter().map(|l| l.1).collect();
            let sections = fbb.create_vector(&sections);
            let minutes = fbb.create_vector(&minutes);
            let cars: Vec<u64> = r.cars.iter().map(|c| c.get()).collect();
            let cars = fbb.create_vector(&cars);
            let (state, time) = match r.state {
                RunState::Waiting { since } => (0, since),
                RunState::Running { until } => (1, until),
                RunState::Arrived { at } => (2, at),
            };
            save::RunSave::create(
                &mut fbb,
                &save::RunSaveArgs {
                    id: r.id.get(),
                    way: r.way.get(),
                    back: r.back,
                    engine: r.engine.get(),
                    owner: r.owner.get(),
                    sections: Some(sections),
                    minutes: Some(minutes),
                    leg: r.leg,
                    state,
                    time,
                    load_kg: r.load_kg,
                    cars: Some(cars),
                },
            )
        })
        .collect();
    let lines = fbb.create_vector(&lines);
    let list = fbb.create_vector(&list);
    let root = save::RunsSave::create(
        &mut fbb,
        &save::RunsSaveArgs {
            lines: Some(lines),
            list: Some(list),
            revision: runs.revision(),
        },
    );
    finish(fbb, root)
}

/// Reads [`encode_runs`]'s section.
fn decode_runs(bytes: &[u8], tag: SectionTag) -> Result<civ_land::trains::Runs, LoadError> {
    use civ_land::trains::{Line, Run, RunState, Runs};
    let root = section_root::<save::RunsSave>(bytes).map_err(|e| unreadable(tag, &e))?;
    let mut lines = Vec::new();
    for l in root.lines().iter().flatten() {
        lines.push(Line {
            way: required(l.way(), "a line")?,
            loops: l.loops().map(|v| v.iter().collect()).unwrap_or_default(),
            // Schema 184 (M10b slice DX, second part): before, no stations and a run a day.
            stations: l.stations(),
            runs_a_day: l.runs_a_day().max(1),
            freight_kg: l.freight_kg(),
            freight_since: l.freight_since(),
            // Schema 185 (M10b slice DX, fourth part): before, no rides and no fare.
            riders: l.riders(),
            fare_kg: l.fare_kg(),
        });
    }
    let mut list = Vec::new();
    for r in root.list().iter().flatten() {
        let id = required(r.id(), "a run")?;
        let sections: Vec<u16> = r.sections().map(|v| v.iter().collect()).unwrap_or_default();
        let minutes: Vec<u32> = r.minutes().map(|v| v.iter().collect()).unwrap_or_default();
        if sections.len() != minutes.len() || usize::from(r.leg()) >= sections.len().max(1) {
            return Err(LoadError::Malformed(format!("run {id} has broken legs")));
        }
        let state = match r.state() {
            0 => RunState::Waiting { since: r.time() },
            1 => RunState::Running { until: r.time() },
            2 => RunState::Arrived { at: r.time() },
            other => {
                return Err(LoadError::Malformed(format!(
                    "run {id} is in an unknown state {other}"
                )));
            }
        };
        list.push(Run {
            id,
            way: required(r.way(), "a run's line")?,
            back: r.back(),
            engine: required(r.engine(), "a run's engine")?,
            owner: required(r.owner(), "a run's owner")?,
            legs: sections.into_iter().zip(minutes).collect(),
            leg: r.leg(),
            state,
            load_kg: r.load_kg(),
            // Schema 183 (M10b slice DW, third part): the wagons it pulls; before, none.
            cars: r
                .cars()
                .map(|v| v.iter().filter_map(PermanentId::from_raw).collect())
                .unwrap_or_default(),
        });
    }
    Ok(Runs::from_saved(lines, list, root.revision()))
}

/// Gas mains and connections (schema 186, M10b slice DY).
fn encode_gas(gas: &civ_land::gas::Gas) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let mains: Vec<_> = gas
        .mains
        .iter()
        .map(|m| {
            let cells = fbb.create_vector(&m.cells);
            save::GasMainSave::create(
                &mut fbb,
                &save::GasMainSaveArgs {
                    id: m.id.get(),
                    works: m.works.get(),
                    cells: Some(cells),
                    capacity_kg_h: m.capacity_kg_h,
                    broken: m.broken,
                    length_m: m.length_m,
                    laid_m: m.laid_m,
                    worth_h: m.worth_h,
                },
            )
        })
        .collect();
    let connections: Vec<_> = gas
        .connections
        .iter()
        .map(|c| {
            save::GasConnectionSave::create(
                &mut fbb,
                &save::GasConnectionSaveArgs {
                    building: c.building.get(),
                    household: c.household.get(),
                    main: c.main.get(),
                    burners: c.burners,
                    cut_off: c.cut_off,
                    evening: c.evening,
                    lit_from: c.lit_from,
                    lit_until: c.lit_until,
                    cause: c.cause.code(),
                },
            )
        })
        .collect();
    let lamps: Vec<_> = gas
        .lamps
        .iter()
        .map(|l| {
            save::StreetLampSave::create(
                &mut fbb,
                &save::StreetLampSaveArgs {
                    cell: l.cell,
                    polity: l.polity.get(),
                    law: l.law.get(),
                    lit_until: l.lit_until,
                },
            )
        })
        .collect();
    let mains = fbb.create_vector(&mains);
    let connections = fbb.create_vector(&connections);
    let lamps = fbb.create_vector(&lamps);
    let root = save::GasSave::create(
        &mut fbb,
        &save::GasSaveArgs {
            mains: Some(mains),
            connections: Some(connections),
            lamps: Some(lamps),
        },
    );
    finish(fbb, root)
}

/// Reads [`encode_gas`]'s section.
fn decode_gas(bytes: &[u8], tag: SectionTag) -> Result<civ_land::gas::Gas, LoadError> {
    use civ_land::gas::{Cause, Connection, Gas, Main, StreetLamp};
    let root = section_root::<save::GasSave>(bytes).map_err(|e| unreadable(tag, &e))?;
    let mut gas = Gas::default();
    for m in root.mains().iter().flatten() {
        gas.mains.push(Main {
            id: required(m.id(), "a gas main")?,
            works: required(m.works(), "a gas main's works")?,
            cells: m.cells().map(|v| v.iter().collect()).unwrap_or_default(),
            capacity_kg_h: m.capacity_kg_h(),
            broken: m.broken(),
            // Schema 187: before, a main had no length and was whole.
            length_m: m.length_m(),
            laid_m: m.laid_m(),
            // Schema 188: before, no worth was reckoned.
            worth_h: m.worth_h(),
        });
    }
    for c in root.connections().iter().flatten() {
        gas.connections.push(Connection {
            building: required(c.building(), "a gas connection's building")?,
            household: required(c.household(), "a gas connection's household")?,
            main: required(c.main(), "a gas connection's main")?,
            burners: c.burners(),
            cut_off: c.cut_off(),
            evening: c.evening(),
            lit_from: c.lit_from(),
            lit_until: c.lit_until(),
            cause: Cause::from_code(c.cause()),
        });
    }
    // Schema 189: before, no street lamps.
    for l in root.lamps().iter().flatten() {
        gas.lamps.push(StreetLamp {
            cell: l.cell(),
            polity: required(l.polity(), "a street lamp's polity")?,
            law: required(l.law(), "a street lamp's law")?,
            lit_until: l.lit_until(),
        });
    }
    Ok(gas)
}

fn decode_boats(
    bytes: &[u8],
    tag: SectionTag,
    rules: &Rules,
    map: &WorldMap,
    schema: Schema,
) -> Result<civ_land::boats::Boats, LoadError> {
    use civ_land::boats::{Boat, Boats};
    let root = section_root::<save::BoatsSave>(bytes).map_err(|e| unreadable(tag, &e))?;
    let cells_on_map = u64::from(map.width) * u64::from(map.height);
    let mut list = Vec::new();
    for b in root.list().iter().flatten() {
        let id = required(b.id(), "a boat")?;
        let Some(vehicle) = rules.catalog.vehicle_index(b.vehicle().unwrap_or("")) else {
            continue;
        };
        if u64::from(b.at()) >= cells_on_map {
            return Err(LoadError::Malformed(format!(
                "boat {id} stands off the map"
            )));
        }
        // Before schema 144 every boat was given whole: none was being made.
        let making = schema >= Schema::V144;
        list.push(Boat {
            id,
            vehicle: vehicle as u16,
            owner: required(b.owner(), "a boat's owner")?,
            at: b.at(),
            made: b.made(),
            wear: b.wear(),
            labour_h: if making { b.labour_h() } else { 0.0 },
            work_h: if making { b.work_h() } else { 0.0 },
            saves_h_year: if making { b.saves_h_year() } else { 0.0 },
            crew: if making {
                (b.crew_day(), b.crew())
            } else {
                (0, 0)
            },
        });
    }
    let boats = Boats::from_saved(list, root.revision());
    if let Some(p) = boats.problems().into_iter().next() {
        return Err(LoadError::Malformed(p));
    }
    Ok(boats)
}

/// The ways saved in `bytes`. One with an unknown surface, keeper or rank, a cell off the map,
/// widths out of order or a track that does not fit its cells is malformed.
fn decode_ways(bytes: &[u8], map: &WorldMap) -> Result<civ_land::ways::Ways, LoadError> {
    use civ_land::ways::{Keeper, Rank, Segment, Surface, Way, Ways};
    let root = section_root::<save::WaysSave>(bytes).map_err(|e| unreadable(SECTION_WAYS, &e))?;
    let mut list = Vec::new();
    for w in root.list().iter().flatten() {
        let id = required(w.id(), "a way")?;
        let bad = |what: &str| LoadError::Malformed(format!("way {id} has {what}"));
        let mut segments = Vec::new();
        for s in w.segments().iter().flatten() {
            segments.push(Segment {
                cells: s.cells().map(|c| c.iter().collect()).unwrap_or_default(),
                corridor_m: s.corridor_m(),
                built_m: s.built_m(),
                usable_m: s.usable_m(),
                surface: Surface::from_code(s.surface())
                    .ok_or_else(|| bad(&format!("surface code {}", s.surface())))?,
                keeper: Keeper::from_code(s.keeper(), s.keeper_id())
                    .ok_or_else(|| bad(&format!("keeper code {}", s.keeper())))?,
                kept: s.kept(),
            });
        }
        list.push(Way {
            id,
            begun: w.begun(),
            segments,
            modes: w.modes(),
            trips: w.trips(),
            trips_day: w.trips_day(),
            rank: Rank::from_code(w.rank())
                .ok_or_else(|| bad(&format!("rank code {}", w.rank())))?,
            track: w.track().map(|t| civ_land::rail::Track {
                gauge_m: t.gauge_m(),
                bed: t.bed().map(|b| b.iter().collect()).unwrap_or_default(),
                cut_m3: t.cut_m3(),
                fill_m3: t.fill_m3(),
                good_kg: t.good_kg(),
                labour_h: t.labour_h(),
            }),
        });
    }
    let ways = Ways::with_revision(list, root.revision());
    if let Some(p) = ways.problems(map.width * map.height).into_iter().next() {
        return Err(LoadError::Malformed(p));
    }
    Ok(ways)
}

/// The wells saved in `bytes`. A well of a system the loaded content no longer has is dropped;
/// one in a cell off the map, or in an unknown state, is malformed.
fn decode_wells(
    bytes: &[u8],
    rules: &Rules,
    map: &WorldMap,
) -> Result<civ_land::wells::Wells, LoadError> {
    use civ_land::wells::{Well, WellState, Wells};
    let root = section_root::<save::WellsSave>(bytes).map_err(|e| unreadable(SECTION_WELLS, &e))?;
    let cells_on_map = u64::from(map.width) * u64::from(map.height);
    let mut list = Vec::new();
    for w in root.list().iter().flatten() {
        let id = required(w.id(), "a well")?;
        let bad = |what: &str| LoadError::Malformed(format!("well {id} has {what}"));
        let Some(system) = rules.catalog.well_index(w.system().unwrap_or("")) else {
            continue;
        };
        if u64::from(w.cell()) >= cells_on_map {
            return Err(bad("a cell off the map"));
        }
        let state = WellState::from_code(w.state(), SimTime::from_minutes(w.state_at()))
            .ok_or_else(|| bad(&format!("state code {}", w.state())))?;
        list.push(Well {
            id,
            system,
            cell: w.cell(),
            rect: RectCm {
                x: w.x_cm(),
                y: w.y_cm(),
                w: w.w_cm(),
                h: w.h_cm(),
            },
            household: required(w.household(), "a well's household")?,
            polity: PermanentId::from_raw(w.polity()),
            ground_m: w.ground_m(),
            depth_m: w.depth_m(),
            target_m: w.target_m(),
            work_h: w.work_h(),
            skill_h: w.skill_h(),
            quality: w.quality(),
            loss: w.loss(),
            mend_h: w.mend_h(),
            cover_h: w.cover_h(),
            covered: w.covered(),
            worth: w.worth(),
            level_m: w.level_m(),
            level_at: SimTime::from_minutes(w.level_at()),
            drawn_l: w.drawn_l(),
            begun: SimTime::from_minutes(w.begun()),
            state,
            crew: (w.crew_day(), w.crew()),
        });
    }
    // The hours households gave to polities' wells (schema 83); a share of a well dropped with
    // its system goes with it.
    let mut shares = Vec::new();
    for s in root.shares().iter().flatten() {
        let (Some(well), Some(household)) = (
            PermanentId::from_raw(s.well()),
            PermanentId::from_raw(s.household()),
        ) else {
            return Err(LoadError::Malformed(
                "a share of a well names nothing".to_owned(),
            ));
        };
        if list.iter().any(|w: &Well| w.id == well) {
            shares.push((well, household, s.hours()));
        }
    }
    let wells = Wells { list, shares };
    let problems = wells.problems(rules.catalog.wells.len());
    if let Some(p) = problems.first() {
        return Err(LoadError::Malformed(p.clone()));
    }
    Ok(wells)
}

fn encode_sickness(people: &Population, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = people
        .sickness
        .episodes()
        .iter()
        .map(|e| {
            let disease = fbb.create_string(
                rules
                    .catalog
                    .diseases
                    .get(usize::from(e.disease))
                    .map_or("", |d| d.id.as_str()),
            );
            let (acquired, source) = e.acquired.code();
            let (ended, outcome) = e.ended.map_or((-1, 0), |(d, o)| (d, o.code()));
            save::EpisodeSave::create(
                &mut fbb,
                &save::EpisodeSaveArgs {
                    id: e.id,
                    person: e.person.get(),
                    disease: Some(disease),
                    infected: e.infected,
                    acquired,
                    source,
                    shed_from: e.course.shed_from,
                    shed_until: e.course.shed_until,
                    ill_from: e.course.ill_from,
                    ill_until: e.course.ill_until,
                    severe: e.course.severe,
                    immune_until: e.course.immune_until,
                    ended,
                    outcome,
                    care_first: e.care.first.map_or(-1, |(d, _)| d),
                    care_by: e.care.first.map_or(0, |(_, p)| p.get()),
                    care_day: e.care.day,
                    care_hours: e.care.hours,
                    care_rr: e.care.rr,
                    care_total_h: e.care.total_h,
                    outbreak: e.outbreak.unwrap_or(0),
                    household: e.household.map_or(0, PermanentId::get),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let outbreaks: Vec<_> = people
        .sickness
        .outbreaks()
        .iter()
        .map(|o| {
            let disease = fbb.create_string(
                rules
                    .catalog
                    .diseases
                    .get(usize::from(o.disease))
                    .map_or("", |d| d.id.as_str()),
            );
            save::OutbreakSave::create(
                &mut fbb,
                &save::OutbreakSaveArgs {
                    id: o.id,
                    disease: Some(disease),
                    settlement: o.settlement.map_or(0, |s| s.get()),
                    began: o.began,
                    first: o.first,
                    ended: o.ended.unwrap_or(-1),
                },
            )
        })
        .collect();
    let outbreaks = fbb.create_vector(&outbreaks);
    let calls_of = |birth: bool| -> Vec<_> {
        people
            .sickness
            .calls
            .iter()
            .filter(|c| c.birth == birth)
            .map(|c| {
                save::CareCallSave::new(c.caller.get(), c.carer.get(), c.fee_kcal, c.day, c.paid)
            })
            .collect()
    };
    let calls = fbb.create_vector(&calls_of(false));
    let birth_calls = fbb.create_vector(&calls_of(true));
    // Calls to inoculate (schema 199, M10c slice ED).
    let inoculation_calls: Vec<_> = people
        .sickness
        .inoculation_calls
        .iter()
        .map(|c| save::CareCallSave::new(c.caller.get(), c.carer.get(), c.fee_kcal, c.day, c.paid))
        .collect();
    let inoculation_calls = fbb.create_vector(&inoculation_calls);
    // Those vaccinated (schema 200, M10c slice ED, third part).
    let vaccinated: Vec<_> = people
        .sickness
        .vaccinated
        .iter()
        .map(|v| {
            let disease = fbb.create_string(
                rules
                    .catalog
                    .diseases
                    .get(usize::from(v.disease))
                    .map_or("", |d| d.id.as_str()),
            );
            save::VaccinatedSave::create(
                &mut fbb,
                &save::VaccinatedSaveArgs {
                    person: v.person.get(),
                    disease: Some(disease),
                    day: v.day,
                    full_until: v.full_until,
                    by: v.by.map_or(0, |b| b.get()),
                },
            )
        })
        .collect();
    let vaccinated = fbb.create_vector(&vaccinated);
    let attended: Vec<_> = people
        .sickness
        .attended
        .iter()
        .map(|a| save::AttendedSave::new(a.attendant.get(), a.household.get(), a.day, a.lived))
        .collect();
    let attended = fbb.create_vector(&attended);
    let tended: Vec<_> = people
        .sickness
        .tended
        .iter()
        .map(|(p, &d)| save::TendedSave::new(p.get(), d))
        .collect();
    let tended = fbb.create_vector(&tended);
    // Deaths in unknowing paid care (schema 197, M10c slice EC, third part).
    let unknowing: Vec<_> = people
        .sickness
        .unknowing
        .iter()
        .map(|u| {
            let disease = fbb.create_string(
                rules
                    .catalog
                    .diseases
                    .get(usize::from(u.disease))
                    .map_or("", |d| d.id.as_str()),
            );
            save::UnknowingSave::create(
                &mut fbb,
                &save::UnknowingSaveArgs {
                    household: u.household.get(),
                    disease: Some(disease),
                    day: u.day,
                },
            )
        })
        .collect();
    let unknowing = fbb.create_vector(&unknowing);
    // Today's company kept (schema 198, M10c slice ED).
    let met: Vec<_> = people
        .met
        .iter()
        .map(|&(a, b, h)| save::CompanySave::new(a.get(), b.get(), h))
        .collect();
    let met = fbb.create_vector(&met);
    let root = save::SicknessSave::create(
        &mut fbb,
        &save::SicknessSaveArgs {
            episodes: Some(list),
            outbreaks: Some(outbreaks),
            calls: Some(calls),
            tended: Some(tended),
            birth_calls: Some(birth_calls),
            attended: Some(attended),
            unknowing: Some(unknowing),
            met: Some(met),
            inoculation_calls: Some(inoculation_calls),
            vaccinated: Some(vaccinated),
        },
    );
    finish(fbb, root)
}

fn encode_fires(people: &Population) -> Vec<u8> {
    use civ_agents::fire::Kindled;
    let mut fbb = FlatBufferBuilder::new();
    let fires = &people.fires;
    let list: Vec<_> = fires
        .list
        .iter()
        .map(|f| {
            let deaths: Vec<u64> = f.deaths.iter().map(|d| d.get()).collect();
            let deaths = fbb.create_vector(&deaths);
            let (kindled, source, gap_m, wind_ms, toward, moisture) = match f.kindled {
                Kindled::Observer { influence } => (0, influence, 0.0, 0.0, 0.0, 0.0),
                Kindled::Neighbour {
                    fire,
                    gap_m,
                    wind_ms,
                    toward,
                    moisture,
                } => (1, fire, gap_m, wind_ms, toward, moisture),
                Kindled::Ignition { ignition } => (2, ignition, 0.0, 0.0, 0.0, 0.0),
            };
            save::FireSave::create(
                &mut fbb,
                &save::FireSaveArgs {
                    id: f.id,
                    building: f.building.get(),
                    household: f.household.get(),
                    settlement: f.settlement.map_or(0, PermanentId::get),
                    x: f.at.0,
                    y: f.at.1,
                    began: f.began.minutes(),
                    kindled,
                    source,
                    gap_m,
                    wind_ms,
                    toward,
                    moisture,
                    episode: f.episode.unwrap_or(0),
                    ended: f.ended.map_or(-1, |t| t.minutes()),
                    goods_kg: f.goods_kg,
                    deaths: Some(deaths),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let episodes: Vec<_> = fires
        .episodes
        .iter()
        .map(|e| {
            save::ConflagrationSave::create(
                &mut fbb,
                &save::ConflagrationSaveArgs {
                    id: e.id,
                    settlement: e.settlement.map_or(0, PermanentId::get),
                    first: e.first,
                    began: e.began.minutes(),
                    ended: e.ended.map_or(-1, |t| t.minutes()),
                },
            )
        })
        .collect();
    let episodes = fbb.create_vector(&episodes);
    let exposure: Vec<_> = fires
        .exposure
        .iter()
        .map(|&(b, kw)| {
            save::ExposureSave::create(
                &mut fbb,
                &save::ExposureSaveArgs {
                    building: b.get(),
                    kw,
                },
            )
        })
        .collect();
    let exposure = fbb.create_vector(&exposure);
    let weighed: Vec<_> = fires
        .weighed
        .iter()
        .map(|w| {
            save::WeighedSave::create(
                &mut fbb,
                &save::WeighedSaveArgs {
                    fire: w.fire,
                    building: w.building.get(),
                    peak_kw: w.peak_kw,
                    threshold_kw: w.threshold_kw,
                    caught: w.caught,
                },
            )
        })
        .collect();
    let weighed = fbb.create_vector(&weighed);
    let ignitions: Vec<_> = fires
        .ignitions
        .iter()
        .map(|g| {
            save::IgnitionSave::create(
                &mut fbb,
                &save::IgnitionSaveArgs {
                    id: g.id,
                    at: g.at.minutes(),
                    building: g.building.get(),
                    household: g.household.get(),
                    settlement: g.settlement.map_or(0, PermanentId::get),
                    hearth: g.hearth as u8,
                    moisture: g.moisture,
                    put_out_by: g.put_out_by.map_or(0, PermanentId::get),
                    fire: g.fire.unwrap_or(0),
                    grows_min: g.grows_min,
                    ended: g.ended.map_or(0, SimTime::minutes),
                    ended_set: g.ended.is_some(),
                },
            )
        })
        .collect();
    let ignitions = fbb.create_vector(&ignitions);
    let knew: Vec<_> = fires
        .knew
        .iter()
        .map(|k| {
            save::KnewSave::create(
                &mut fbb,
                &save::KnewSaveArgs {
                    fire: k.fire,
                    person: k.person.get(),
                    at: k.at.minutes(),
                    how: k.how as u8,
                    shouted: k.shouted,
                },
            )
        })
        .collect();
    let knew = fbb.create_vector(&knew);
    let wet: Vec<_> = fires
        .wet
        .iter()
        .map(|&(building, litres)| {
            save::WetSave::create(
                &mut fbb,
                &save::WetSaveArgs {
                    building: building.get(),
                    litres,
                },
            )
        })
        .collect();
    let wet = fbb.create_vector(&wet);
    let thrown: Vec<_> = fires
        .thrown
        .iter()
        .map(|t| {
            save::ThrownSave::create(
                &mut fbb,
                &save::ThrownSaveArgs {
                    fire: t.fire,
                    building: t.building.get(),
                    person: t.person.get(),
                    at: t.at.minutes(),
                    litres: t.litres,
                    from: t.from,
                },
            )
        })
        .collect();
    let thrown = fbb.create_vector(&thrown);
    let stripped: Vec<_> = fires
        .stripped
        .iter()
        .map(|t| {
            save::StrippedSave::create(
                &mut fbb,
                &save::StrippedSaveArgs {
                    fire: t.fire,
                    building: t.building.get(),
                    person: t.person.get(),
                    at: t.at.minutes(),
                    m2: t.m2,
                },
            )
        })
        .collect();
    let stripped = fbb.create_vector(&stripped);
    let salvaged: Vec<_> = fires
        .salvaged
        .iter()
        .map(|t| {
            save::SalvagedSave::create(
                &mut fbb,
                &save::SalvagedSaveArgs {
                    fire: t.fire,
                    building: t.building.get(),
                    person: t.person.get(),
                    at: t.at.minutes(),
                    good: t.good,
                    kg: t.kg,
                },
            )
        })
        .collect();
    let salvaged = fbb.create_vector(&salvaged);
    let pulled: Vec<_> = fires
        .pulled
        .iter()
        .map(|p| {
            let people: Vec<u64> = p.people.iter().map(|id| id.get()).collect();
            let people = fbb.create_vector(&people);
            save::PulledSave::create(
                &mut fbb,
                &save::PulledSaveArgs {
                    fire: p.fire,
                    building: p.building.get(),
                    at: p.at.minutes(),
                    people: Some(people),
                    work_h: p.work_h,
                },
            )
        })
        .collect();
    let pulled = fbb.create_vector(&pulled);
    let root = save::FiresSave::create(
        &mut fbb,
        &save::FiresSaveArgs {
            fires: Some(list),
            episodes: Some(episodes),
            exposure: Some(exposure),
            weighed: Some(weighed),
            ignitions: Some(ignitions),
            knew: Some(knew),
            wet: Some(wet),
            thrown: Some(thrown),
            stripped: Some(stripped),
            salvaged: Some(salvaged),
            pulled: Some(pulled),
        },
    );
    finish(fbb, root)
}

/// Before schema 90 an escape caught or was put out in the minute it happened: each has ended
/// then, and nobody's knowledge of a fire was kept.
fn decode_fires(bytes: &[u8], schema: Schema) -> Result<civ_agents::fire::Fires, LoadError> {
    use civ_agents::fire::{
        Conflagration, Fire, Fires, Hearth, Ignition, Kindled, Knew, Noticed, Salvaged, Stripped,
        Thrown, Weighed,
    };
    let root = section_root::<save::FiresSave>(bytes).map_err(|e| unreadable(SECTION_FIRES, &e))?;
    let at = |m: i64| (m >= 0).then(|| SimTime::from_minutes(m));
    let mut out = Fires::default();
    for f in root.fires().iter().flatten() {
        let kindled = match f.kindled() {
            0 => Kindled::Observer {
                influence: f.source(),
            },
            1 => Kindled::Neighbour {
                fire: f.source(),
                gap_m: f.gap_m(),
                wind_ms: f.wind_ms(),
                toward: f.toward(),
                moisture: f.moisture(),
            },
            2 => Kindled::Ignition {
                ignition: f.source(),
            },
            other => {
                return Err(LoadError::Malformed(format!(
                    "fire {} was kindled in an unknown way {other}",
                    f.id()
                )));
            }
        };
        out.list.push(Fire {
            id: f.id(),
            building: required(f.building(), "a fire's building")?,
            household: required(f.household(), "a fire's household")?,
            settlement: PermanentId::from_raw(f.settlement()),
            at: (f.x(), f.y()),
            began: SimTime::from_minutes(f.began()),
            kindled,
            episode: (f.episode() > 0).then_some(f.episode()),
            ended: at(f.ended()),
            goods_kg: f.goods_kg(),
            deaths: f
                .deaths()
                .iter()
                .flatten()
                .filter_map(PermanentId::from_raw)
                .collect(),
        });
    }
    for e in root.episodes().iter().flatten() {
        out.episodes.push(Conflagration {
            id: e.id(),
            settlement: PermanentId::from_raw(e.settlement()),
            first: e.first(),
            began: SimTime::from_minutes(e.began()),
            ended: at(e.ended()),
        });
    }
    for x in root.exposure().iter().flatten() {
        out.exposure
            .push((required(x.building(), "an exposed building")?, x.kw()));
    }
    for w in root.weighed().iter().flatten() {
        out.weighed.push(Weighed {
            fire: w.fire(),
            building: required(w.building(), "a weighed building")?,
            peak_kw: w.peak_kw(),
            threshold_kw: w.threshold_kw(),
            caught: w.caught(),
        });
    }
    for g in root.ignitions().iter().flatten() {
        out.ignitions.push(Ignition {
            id: g.id(),
            at: SimTime::from_minutes(g.at()),
            building: required(g.building(), "an ignition's building")?,
            household: required(g.household(), "an ignition's household")?,
            settlement: PermanentId::from_raw(g.settlement()),
            hearth: Hearth::from_code(g.hearth()).ok_or_else(|| {
                LoadError::Malformed(format!("ignition {} came from an unknown fire", g.id()))
            })?,
            moisture: g.moisture(),
            grows_min: g.grows_min(),
            ended: if schema >= Schema::V90 {
                g.ended_set().then(|| SimTime::from_minutes(g.ended()))
            } else {
                Some(SimTime::from_minutes(g.at()))
            },
            put_out_by: PermanentId::from_raw(g.put_out_by()),
            fire: (g.fire() > 0).then_some(g.fire()),
        });
    }
    for k in root.knew().iter().flatten() {
        out.knew.push(Knew {
            fire: k.fire(),
            person: required(k.person(), "someone who knew of a fire")?,
            at: SimTime::from_minutes(k.at()),
            how: Noticed::from_code(k.how()).ok_or_else(|| {
                LoadError::Malformed(format!(
                    "someone knew of fire {} by an unknown way",
                    k.fire()
                ))
            })?,
            shouted: k.shouted(),
        });
    }
    for w in root.wet().iter().flatten() {
        out.wet
            .push((required(w.building(), "a wet roof")?, w.litres()));
    }
    for t in root.thrown().iter().flatten() {
        out.thrown.push(Thrown {
            fire: t.fire(),
            building: required(t.building(), "a roof water was thrown on")?,
            person: required(t.person(), "someone who threw water")?,
            at: SimTime::from_minutes(t.at()),
            litres: t.litres(),
            from: t.from(),
        });
    }
    for t in root.stripped().iter().flatten() {
        out.stripped.push(Stripped {
            fire: t.fire(),
            building: required(t.building(), "a stripped roof")?,
            person: required(t.person(), "someone who stripped a roof")?,
            at: SimTime::from_minutes(t.at()),
            m2: t.m2(),
        });
    }
    for t in root.salvaged().iter().flatten() {
        out.salvaged.push(Salvaged {
            fire: t.fire(),
            building: required(t.building(), "a building goods were carried out of")?,
            person: required(t.person(), "someone who carried goods out")?,
            at: SimTime::from_minutes(t.at()),
            good: t.good(),
            kg: t.kg(),
        });
    }
    if schema >= Schema::V98 {
        for p in root.pulled().iter().flatten() {
            let people = p
                .people()
                .iter()
                .flatten()
                .map(|id| {
                    PermanentId::from_raw(id).ok_or_else(|| {
                        LoadError::Malformed("zero demolition crew member".to_owned())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            out.pulled.push(civ_agents::fire::Pulled {
                fire: p.fire(),
                building: PermanentId::from_raw(p.building())
                    .ok_or_else(|| LoadError::Malformed("zero demolished building".to_owned()))?,
                at: SimTime::from_minutes(p.at()),
                people,
                work_h: p.work_h(),
            });
        }
    }
    Ok(out)
}

fn encode_quakes(pop: &Population) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let quakes: Vec<_> = pop
        .quakes
        .list
        .iter()
        .map(|q| {
            let shaken: Vec<_> = q
                .shaken
                .iter()
                .map(|s| {
                    let cracked = fbb.create_vector(&s.cracked);
                    save::ShakenSave::create(
                        &mut fbb,
                        &save::ShakenSaveArgs {
                            building: s.building.get(),
                            household: s.household.get(),
                            settlement: raw(s.settlement),
                            z: s.z,
                            cs: s.cs,
                            outcome: s.outcome.code(),
                            cracked: Some(cracked),
                            deaths: s.deaths,
                        },
                    )
                })
                .collect();
            let shaken = fbb.create_vector(&shaken);
            let observers = fbb.create_vector(&q.origin.observers);
            save::QuakeSave::create(
                &mut fbb,
                &save::QuakeSaveArgs {
                    id: q.id,
                    influence: q.influence,
                    at: q.at.minutes(),
                    strength: q.strength,
                    settlement: raw(q.settlement),
                    origin_known: q.origin.known,
                    observers: Some(observers),
                    shaken: Some(shaken),
                },
            )
        })
        .collect();
    let quakes = fbb.create_vector(&quakes);
    let root = save::QuakesSave::create(
        &mut fbb,
        &save::QuakesSaveArgs {
            quakes: Some(quakes),
        },
    );
    finish(fbb, root)
}

/// Refuses an event that names no shaking the observer sent, a time to come, a strength outside
/// the tool's range or a building's outcome it does not know.
fn decode_quakes(
    bytes: &[u8],
    inputs: &civ_agents::influence::Influences,
    now: SimTime,
) -> Result<civ_agents::quake::Quakes, LoadError> {
    use civ_agents::influence::InfluenceKind;
    use civ_agents::quake::{Outcome, Quake, Quakes, STRENGTH_RANGE, Shaken};
    let root =
        section_root::<save::QuakesSave>(bytes).map_err(|e| unreadable(SECTION_QUAKES, &e))?;
    let bad = |what: &str| LoadError::Malformed(format!("invalid shaking: {what}"));
    let mut out = Quakes::default();
    for (index, q) in root.quakes().iter().flatten().enumerate() {
        let at = SimTime::from_minutes(q.at());
        let sent = |id: u32| {
            inputs
                .list
                .iter()
                .any(|i| i.id == id && i.kind == InfluenceKind::Earthquake)
        };
        // Kept to single precision: 0.6 reads back a hair above the tool's 0.6.
        let range = STRENGTH_RANGE.0 as f32..=STRENGTH_RANGE.1 as f32;
        if q.id() as usize != index + 1
            || q.at() < 0
            || at > now
            || out.list.last().is_some_and(|p| p.at > at)
            || !range.contains(&q.strength())
            || !sent(q.influence())
        {
            return Err(bad("its event"));
        }
        let observers: Vec<u32> = q.observers().iter().flatten().collect();
        if !observers.windows(2).all(|w| w[0] < w[1]) || !observers.iter().all(|&id| sent(id)) {
            return Err(bad("its origin"));
        }
        let mut shaken = Vec::new();
        for s in q.shaken().iter().flatten() {
            let outcome = Outcome::from_code(s.outcome()).ok_or_else(|| bad("an outcome"))?;
            if !s.z().is_finite() || !s.cs().is_finite() || s.cs() < 0.0 {
                return Err(bad("a response"));
            }
            shaken.push(Shaken {
                building: required(s.building(), "a shaken building")?,
                household: required(s.household(), "a shaken building's household")?,
                settlement: id(s.settlement()),
                z: s.z(),
                cs: s.cs(),
                outcome,
                cracked: s.cracked().iter().flatten().collect(),
                deaths: s.deaths(),
            });
        }
        out.list.push(Quake {
            id: q.id(),
            influence: q.influence(),
            at,
            strength: q.strength(),
            settlement: id(q.settlement()),
            origin: civ_agents::floods::Origin {
                known: q.origin_known(),
                observers,
            },
            shaken,
        });
    }
    Ok(out)
}

fn encode_floods(land: &Land, pop: &Population) -> Vec<u8> {
    let floods = &pop.floods;
    let mut fbb = FlatBufferBuilder::new();
    let high_waters: Vec<_> = floods
        .high_waters
        .iter()
        .map(|hw| {
            let observers = fbb.create_vector(&hw.origin.observers);
            save::HighWaterSave::create(
                &mut fbb,
                &save::HighWaterSaveArgs {
                    origin_known: hw.origin.known,
                    observers: Some(observers),
                    id: hw.id,
                    reach: hw.reach,
                    settlement: raw(hw.settlement),
                    x: hw.at.0,
                    y: hw.at.1,
                    began: hw.began.minutes(),
                    ended: hw.ended.map_or(-1, |t| t.minutes()),
                    peak_discharge: hw.peak_discharge,
                    peak_stage: hw.peak_stage,
                    episode: hw.episode.unwrap_or(0),
                    buildings_flooded: hw.buildings_flooded,
                    fields_flooded: hw.fields_flooded,
                    spoiled_kg: hw.spoiled_kg,
                    scoured_crossings: hw.scoured_crossings,
                },
            )
        })
        .collect();
    let high_waters = fbb.create_vector(&high_waters);
    let flood_episodes: Vec<_> = floods
        .flood_episodes
        .iter()
        .map(|ep| {
            let hw_vec = fbb.create_vector(&ep.high_waters);
            let observers = fbb.create_vector(&ep.origin.observers);
            save::FloodEpisodeSave::create(
                &mut fbb,
                &save::FloodEpisodeSaveArgs {
                    origin_known: ep.origin.known,
                    observers: Some(observers),
                    id: ep.id,
                    settlement: raw(ep.settlement),
                    first_reach: ep.first_reach,
                    began: ep.began.minutes(),
                    ended: ep.ended.map_or(-1, |t| t.minutes()),
                    peak_discharge: ep.peak_discharge,
                    high_waters: Some(hw_vec),
                    total_spoiled_kg: ep.total_spoiled_kg,
                    total_buildings: ep.total_buildings,
                    total_fields_lost: ep.total_fields_lost,
                },
            )
        })
        .collect();
    let flood_episodes = fbb.create_vector(&flood_episodes);
    let storm_damages: Vec<_> = floods
        .storm_damages
        .iter()
        .map(|sd| {
            let observers = fbb.create_vector(&sd.origin.observers);
            save::StormDamageSave::create(
                &mut fbb,
                &save::StormDamageSaveArgs {
                    origin_known: sd.origin.known,
                    observers: Some(observers),
                    id: sd.id,
                    building: sd.building.get(),
                    household: sd.household.get(),
                    settlement: raw(sd.settlement),
                    x: sd.at.0,
                    y: sd.at.1,
                    time: sd.time.minutes(),
                    wind_speed_ms: sd.wind_speed_ms,
                    uplift_pa: sd.uplift_pa,
                    episode: sd.episode.unwrap_or(0),
                },
            )
        })
        .collect();
    let storm_damages = fbb.create_vector(&storm_damages);
    let storm_episodes: Vec<_> = floods
        .storm_episodes
        .iter()
        .map(|ep| {
            let damages_vec = fbb.create_vector(&ep.damages);
            let observers = fbb.create_vector(&ep.origin.observers);
            save::StormEpisodeSave::create(
                &mut fbb,
                &save::StormEpisodeSaveArgs {
                    origin_known: ep.origin.known,
                    observers: Some(observers),
                    id: ep.id,
                    settlement: raw(ep.settlement),
                    began: ep.began.minutes(),
                    ended: ep.ended.map_or(-1, |t| t.minutes()),
                    peak_wind_speed_ms: ep.peak_wind_speed_ms,
                    damages: Some(damages_vec),
                },
            )
        })
        .collect();
    let storm_episodes = fbb.create_vector(&storm_episodes);
    let reach_discharges = fbb.create_vector(&land.reach_discharges);
    let forcings: Vec<_> = land
        .river_forcings
        .iter()
        .map(|f| save::RiverForcingSave::new(f.reach, f.discharge_m3s, f.until_day, f.influence))
        .collect();
    let river_forcings = fbb.create_vector(&forcings);
    let checks: Vec<_> = land
        .storm_checks
        .iter()
        .map(|c| save::StormCheckSave::new(c.building.get(), c.group, c.day, c.observer))
        .collect();
    let storm_checks = fbb.create_vector(&checks);
    let root = save::FloodsSave::create(
        &mut fbb,
        &save::FloodsSaveArgs {
            high_waters: Some(high_waters),
            flood_episodes: Some(flood_episodes),
            storm_damages: Some(storm_damages),
            storm_episodes: Some(storm_episodes),
            flood_revision: land.flood_revision,
            reach_discharges: Some(reach_discharges),
            river_forcings: Some(river_forcings),
            storm_checks: Some(storm_checks),
        },
    );
    finish(fbb, root)
}

fn decode_floods(
    bytes: &[u8],
    schema: Schema,
) -> Result<(civ_agents::floods::Floods, u32, Option<Vec<f32>>), LoadError> {
    use civ_agents::floods::{FloodEpisode, Floods, HighWater, StormDamage, StormEpisode};
    let root =
        section_root::<save::FloodsSave>(bytes).map_err(|e| unreadable(SECTION_FLOODS, &e))?;
    let at = |m: i64| (m >= 0).then(|| SimTime::from_minutes(m));
    let mut out = Floods::default();
    for hw in root.high_waters().iter().flatten() {
        if hw.ended() < -1 {
            return Err(LoadError::Malformed("invalid high water end".to_owned()));
        }
        out.high_waters.push(HighWater {
            origin: if schema >= Schema::V96 {
                civ_agents::floods::Origin {
                    known: hw.origin_known(),
                    observers: hw.observers().iter().flatten().collect(),
                }
            } else {
                Default::default()
            },
            id: hw.id(),
            reach: hw.reach(),
            settlement: PermanentId::from_raw(hw.settlement()),
            at: (hw.x(), hw.y()),
            began: SimTime::from_minutes(hw.began()),
            ended: at(hw.ended()),
            peak_discharge: hw.peak_discharge(),
            peak_stage: hw.peak_stage(),
            episode: (hw.episode() > 0).then_some(hw.episode()),
            buildings_flooded: hw.buildings_flooded(),
            fields_flooded: hw.fields_flooded(),
            spoiled_kg: hw.spoiled_kg(),
            scoured_crossings: hw.scoured_crossings(),
        });
    }
    for ep in root.flood_episodes().iter().flatten() {
        if ep.ended() < -1 {
            return Err(LoadError::Malformed("invalid flood episode end".to_owned()));
        }
        out.flood_episodes.push(FloodEpisode {
            origin: if schema >= Schema::V96 {
                civ_agents::floods::Origin {
                    known: ep.origin_known(),
                    observers: ep.observers().iter().flatten().collect(),
                }
            } else {
                Default::default()
            },
            id: ep.id(),
            settlement: PermanentId::from_raw(ep.settlement()),
            first_reach: ep.first_reach(),
            began: SimTime::from_minutes(ep.began()),
            ended: at(ep.ended()),
            peak_discharge: ep.peak_discharge(),
            high_waters: ep.high_waters().iter().flatten().collect(),
            total_spoiled_kg: ep.total_spoiled_kg(),
            total_buildings: ep.total_buildings(),
            total_fields_lost: ep.total_fields_lost(),
        });
    }
    for sd in root.storm_damages().iter().flatten() {
        out.storm_damages.push(StormDamage {
            origin: if schema >= Schema::V96 {
                civ_agents::floods::Origin {
                    known: sd.origin_known(),
                    observers: sd.observers().iter().flatten().collect(),
                }
            } else {
                Default::default()
            },
            id: sd.id(),
            building: required(sd.building(), "a storm damage building")?,
            household: required(sd.household(), "a storm damage household")?,
            settlement: PermanentId::from_raw(sd.settlement()),
            at: (sd.x(), sd.y()),
            time: SimTime::from_minutes(sd.time()),
            wind_speed_ms: sd.wind_speed_ms(),
            uplift_pa: sd.uplift_pa(),
            episode: (sd.episode() > 0).then_some(sd.episode()),
        });
    }
    for ep in root.storm_episodes().iter().flatten() {
        if ep.ended() < -1 {
            return Err(LoadError::Malformed("invalid storm episode end".to_owned()));
        }
        out.storm_episodes.push(StormEpisode {
            origin: if schema >= Schema::V96 {
                civ_agents::floods::Origin {
                    known: ep.origin_known(),
                    observers: ep.observers().iter().flatten().collect(),
                }
            } else {
                Default::default()
            },
            id: ep.id(),
            settlement: PermanentId::from_raw(ep.settlement()),
            began: SimTime::from_minutes(ep.began()),
            ended: at(ep.ended()),
            peak_wind_speed_ms: ep.peak_wind_speed_ms(),
            damages: ep.damages().iter().flatten().collect(),
        });
    }
    let flood_revision = root.flood_revision();
    let reach_discharges = root
        .reach_discharges()
        .map(|v| v.iter().collect::<Vec<f32>>());
    Ok((out, flood_revision, reach_discharges))
}

/// Refuses invalid historical links and numbers without guessing missing causes.
fn validate_flood_records(
    f: &civ_agents::floods::Floods,
    inputs: &civ_agents::influence::Influences,
    reaches: usize,
    now: SimTime,
) -> Result<(), LoadError> {
    use civ_agents::influence::InfluenceKind;
    let finite = |v: f32| v.is_finite() && v >= 0.0;
    let span = |began: SimTime, ended: Option<SimTime>| {
        began.minutes() >= 0 && began <= now && ended.is_none_or(|end| end >= began && end <= now)
    };
    let origin = |o: &civ_agents::floods::Origin, kind: InfluenceKind| {
        o.observers.windows(2).all(|w| w[0] < w[1])
            && o.observers
                .iter()
                .all(|id| inputs.list.iter().any(|i| i.id == *id && i.kind == kind))
    };
    let unique = |ids: &[u32]| {
        let mut seen = std::collections::BTreeSet::new();
        ids.iter().all(|id| *id > 0 && seen.insert(*id))
    };
    let bad = || LoadError::Malformed("invalid flood or storm incident/episode".to_owned());
    for (index, h) in f.high_waters.iter().enumerate() {
        if h.id as usize != index + 1
            || h.reach as usize >= reaches
            || !span(h.began, h.ended)
            || !h.at.0.is_finite()
            || !h.at.1.is_finite()
            || !finite(h.peak_discharge)
            || !finite(h.peak_stage)
            || !finite(h.spoiled_kg)
            || !origin(&h.origin, InfluenceKind::Flood)
            || h.episode.is_some_and(|id| {
                !f.flood_episodes
                    .iter()
                    .any(|e| e.id == id && e.high_waters.contains(&h.id))
            })
        {
            return Err(bad());
        }
    }
    for (index, e) in f.flood_episodes.iter().enumerate() {
        if e.id as usize != index + 1
            || e.first_reach as usize >= reaches
            || !span(e.began, e.ended)
            || !finite(e.peak_discharge)
            || !finite(e.total_spoiled_kg)
            || !origin(&e.origin, InfluenceKind::Flood)
            || !unique(&e.high_waters)
            || e.high_waters.iter().any(|id| {
                !f.high_waters
                    .iter()
                    .any(|h| h.id == *id && h.episode == Some(e.id))
            })
        {
            return Err(bad());
        }
    }
    for (index, d) in f.storm_damages.iter().enumerate() {
        if d.id as usize != index + 1
            || !span(d.time, None)
            || !d.at.0.is_finite()
            || !d.at.1.is_finite()
            || !finite(d.wind_speed_ms)
            || !finite(d.uplift_pa)
            || !origin(&d.origin, InfluenceKind::Storm)
            || d.episode.is_some_and(|id| {
                !f.storm_episodes
                    .iter()
                    .any(|e| e.id == id && e.damages.contains(&d.id))
            })
        {
            return Err(bad());
        }
    }
    for (index, e) in f.storm_episodes.iter().enumerate() {
        if e.id as usize != index + 1
            || !span(e.began, e.ended)
            || !finite(e.peak_wind_speed_ms)
            || !origin(&e.origin, InfluenceKind::Storm)
            || !unique(&e.damages)
            || e.damages.iter().any(|id| {
                !f.storm_damages
                    .iter()
                    .any(|d| d.id == *id && d.episode == Some(e.id))
            })
        {
            return Err(bad());
        }
    }
    Ok(())
}

/// An infection of a disease the loaded content no longer has is let go.
/// Today's company at the hearth kept for a disease passing by it (schema 198, M10c slice ED):
/// none in older saves; a pair naming nobody is let go.
fn decode_met(bytes: &[u8]) -> Result<Vec<(PermanentId, PermanentId, f32)>, LoadError> {
    let root =
        section_root::<save::SicknessSave>(bytes).map_err(|e| unreadable(SECTION_SICKNESS, &e))?;
    Ok(root
        .met()
        .into_iter()
        .flatten()
        .filter_map(|m| {
            Some((
                PermanentId::from_raw(m.a())?,
                PermanentId::from_raw(m.b())?,
                m.hours(),
            ))
        })
        .collect())
}

fn decode_sickness(
    bytes: &[u8],
    rules: &Rules,
) -> Result<civ_agents::sickness::Sickness, LoadError> {
    use civ_agents::sickness::{Acquired, Care, Course, Episode, Outbreak, Outcome, Sickness};
    let root =
        section_root::<save::SicknessSave>(bytes).map_err(|e| unreadable(SECTION_SICKNESS, &e))?;
    let mut list = Vec::new();
    for e in root.episodes().iter().flatten() {
        let Some(disease) = rules
            .catalog
            .disease_index(e.disease().unwrap_or_default())
            .and_then(|i| u16::try_from(i).ok())
        else {
            continue;
        };
        let acquired = Acquired::from_code(e.acquired(), e.source()).ok_or_else(|| {
            LoadError::Malformed(format!("episode {} came by an unknown way", e.id()))
        })?;
        let ended = if e.ended() < 0 {
            None
        } else {
            let outcome = Outcome::from_code(e.outcome()).ok_or_else(|| {
                LoadError::Malformed(format!("episode {} ended in an unknown way", e.id()))
            })?;
            Some((e.ended(), outcome))
        };
        list.push(Episode {
            id: e.id(),
            person: required(e.person(), "an episode's person")?,
            household: PermanentId::from_raw(e.household()),
            disease,
            infected: e.infected(),
            acquired,
            course: Course {
                shed_from: e.shed_from(),
                shed_until: e.shed_until(),
                ill_from: e.ill_from(),
                ill_until: e.ill_until(),
                severe: e.severe(),
                immune_until: e.immune_until(),
            },
            ended,
            care: Care {
                first: if e.care_first() < 0 {
                    None
                } else {
                    Some((e.care_first(), required(e.care_by(), "a carer")?))
                },
                day: e.care_day(),
                hours: e.care_hours(),
                rr: e.care_rr(),
                total_h: e.care_total_h(),
            },
            outbreak: (e.outbreak() > 0).then_some(e.outbreak()),
        });
    }
    // An outbreak of a disease the loaded content no longer has is let go with its cases.
    let mut outbreaks = Vec::new();
    for o in root.outbreaks().iter().flatten() {
        let Some(disease) = rules
            .catalog
            .disease_index(o.disease().unwrap_or_default())
            .and_then(|i| u16::try_from(i).ok())
        else {
            continue;
        };
        outbreaks.push(Outbreak {
            id: o.id(),
            disease,
            settlement: PermanentId::from_raw(o.settlement()),
            began: o.began(),
            first: o.first(),
            ended: (o.ended() >= 0).then_some(o.ended()),
        });
    }
    let mut sickness = Sickness::with(list, outbreaks);
    // Calls for carers (schema 190, M10c slice DZ), and to attend a birth (schema 192, M10c slice
    // EA): none in older saves; calls for carers first, as they were made apart.
    let calls = root.calls().into_iter().flatten().map(|c| (c, false));
    let birth_calls = root.birth_calls().into_iter().flatten().map(|c| (c, true));
    for (c, birth) in calls.chain(birth_calls) {
        let (Some(caller), Some(carer)) = (
            PermanentId::from_raw(c.caller()),
            PermanentId::from_raw(c.carer()),
        ) else {
            continue;
        };
        sickness.calls.push(civ_agents::sickness::CareCall {
            caller,
            carer,
            fee_kcal: c.fee_kcal(),
            day: c.day(),
            paid: c.paid(),
            birth,
        });
    }
    // Births attended (schema 192): none in older saves.
    for a in root.attended().iter().flatten() {
        let (Some(attendant), Some(household)) = (
            PermanentId::from_raw(a.attendant()),
            PermanentId::from_raw(a.household()),
        ) else {
            continue;
        };
        sickness.attended.push(civ_agents::sickness::BirthAttended {
            attendant,
            household,
            day: a.day(),
            lived: a.lived(),
        });
    }
    // Calls to inoculate (schema 199): none in older saves.
    for c in root.inoculation_calls().into_iter().flatten() {
        let (Some(caller), Some(carer)) = (
            PermanentId::from_raw(c.caller()),
            PermanentId::from_raw(c.carer()),
        ) else {
            continue;
        };
        sickness
            .inoculation_calls
            .push(civ_agents::sickness::CareCall {
                caller,
                carer,
                fee_kcal: c.fee_kcal(),
                day: c.day(),
                paid: c.paid(),
                birth: false,
            });
    }
    // Those vaccinated (schema 200): none in older saves; against a disease the loaded content no
    // longer has, let go. Who vaccinated them (schema 201): nobody known in older saves.
    for v in root.vaccinated().iter().flatten() {
        let (Some(person), Some(disease)) = (
            PermanentId::from_raw(v.person()),
            rules.catalog.disease_index(v.disease().unwrap_or_default()),
        ) else {
            continue;
        };
        sickness.vaccinated.push(civ_agents::sickness::Vaccinated {
            person,
            disease: disease as u16,
            day: v.day(),
            full_until: v.full_until(),
            by: PermanentId::from_raw(v.by()),
        });
    }
    // Deaths in unknowing paid care (schema 197): none in older saves; one of a disease the
    // loaded content no longer has is let go.
    for u in root.unknowing().iter().flatten() {
        let (Some(household), Some(disease)) = (
            PermanentId::from_raw(u.household()),
            rules.catalog.disease_index(u.disease().unwrap_or_default()),
        ) else {
            continue;
        };
        sickness
            .unknowing
            .push(civ_agents::sickness::UnknowingCare {
                household,
                disease: disease as u16,
                day: u.day(),
            });
    }
    // Who tended the ill lately (schema 191, M10c slice EA): none in older saves.
    for t in root.tended().iter().flatten() {
        if let Some(p) = PermanentId::from_raw(t.person()) {
            sickness.tended.insert(p, t.day());
        }
    }
    Ok(sickness)
}

fn node_save(n: civ_agents::contagion::Node) -> save::NodeSave {
    let (code, id) = n.code();
    save::NodeSave::new(code, id)
}

fn node_load(n: &save::NodeSave) -> Result<civ_agents::contagion::Node, LoadError> {
    civ_agents::contagion::Node::from_code(n.code(), n.id()).ok_or_else(|| {
        LoadError::Malformed(format!(
            "a load lies at an unknown place {}:{}",
            n.code(),
            n.id()
        ))
    })
}

fn encode_contagion(people: &Population, rules: &Rules) -> Vec<u8> {
    let c = &people.contagion;
    let mut fbb = FlatBufferBuilder::new();
    let ids: Vec<_> = rules
        .catalog
        .diseases
        .iter()
        .map(|d| fbb.create_string(&d.id))
        .collect();
    let diseases = fbb.create_vector(&ids);
    let loads: Vec<_> = c
        .loads
        .iter()
        .map(|(&(node, d), &amount)| save::LoadSave::new(&node_save(node), d, amount))
        .collect();
    let loads = fbb.create_vector(&loads);
    let stores: Vec<_> = c
        .stores
        .iter()
        .flat_map(|(&(household, d), s)| {
            s.iter().map(move |(&source, &amount)| {
                save::StoredSave::new(household.get(), &node_save(source), d, amount)
            })
        })
        .collect();
    let stores = fbb.create_vector(&stores);
    let transit: Vec<_> = c
        .transit
        .iter()
        .map(|t| {
            save::TransitSave::new(
                t.arrives,
                &node_save(t.from),
                &node_save(t.to),
                t.disease,
                t.amount,
            )
        })
        .collect();
    let transit = fbb.create_vector(&transit);
    let moves: Vec<_> = c
        .moves
        .iter()
        .map(|m| {
            save::MoveSave::new(
                m.day,
                &node_save(m.from),
                &node_save(m.to),
                m.disease,
                m.how.code(),
                m.amount,
            )
        })
        .collect();
    let moves = fbb.create_vector(&moves);
    let root = save::ContagionSave::create(
        &mut fbb,
        &save::ContagionSaveArgs {
            diseases: Some(diseases),
            loads: Some(loads),
            stores: Some(stores),
            transit: Some(transit),
            moves: Some(moves),
        },
    );
    finish(fbb, root)
}

/// Loads of a disease the loaded content no longer has are let go.
fn decode_contagion(
    bytes: &[u8],
    rules: &Rules,
) -> Result<civ_agents::contagion::Contagion, LoadError> {
    use civ_agents::contagion::{Contagion, How, Move, Transit};
    let root = section_root::<save::ContagionSave>(bytes)
        .map_err(|e| unreadable(SECTION_CONTAGION, &e))?;
    let map: Vec<Option<u16>> = read_strings(root.diseases())
        .iter()
        .map(|id| {
            rules
                .catalog
                .disease_index(id)
                .and_then(|i| u16::try_from(i).ok())
        })
        .collect();
    let disease = |d: u16| map.get(usize::from(d)).copied().flatten();
    let mut c = Contagion::default();
    for l in root.loads().iter().flatten() {
        if let Some(d) = disease(l.disease()) {
            c.add(node_load(l.at())?, d, l.amount());
        }
    }
    for s in root.stores().iter().flatten() {
        if let Some(d) = disease(s.disease()) {
            let household = required(s.household(), "a store's household")?;
            c.store_add(household, d, node_load(s.source())?, s.amount());
        }
    }
    for t in root.transit().iter().flatten() {
        if let Some(d) = disease(t.disease()) {
            c.transit.push(Transit {
                arrives: t.arrives(),
                disease: d,
                from: node_load(t.from())?,
                to: node_load(t.to())?,
                amount: t.amount(),
            });
        }
    }
    for m in root.moves().iter().flatten() {
        if let Some(d) = disease(m.disease()) {
            let how = How::from_code(m.how()).ok_or_else(|| {
                LoadError::Malformed(format!("a load moved in an unknown way {}", m.how()))
            })?;
            c.moves.push(Move {
                day: m.day(),
                disease: d,
                how,
                from: node_load(m.from())?,
                to: node_load(m.to())?,
                amount: m.amount(),
            });
        }
    }
    Ok(c)
}

fn encode_influences(inf: &civ_agents::influence::Influences, rules: &Rules) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let list: Vec<_> = inf
        .list
        .iter()
        .map(|i| {
            save::InfluenceSave::create(
                &mut fbb,
                &save::InfluenceSaveArgs {
                    id: i.id,
                    at: i.at.minutes(),
                    last: i.last.minutes(),
                    uses: i.uses,
                    kind: i.kind.code(),
                    target: i.target.get(),
                    subject: i.subject,
                    taken: i.taken.unwrap_or(-1),
                    weighed: i.weighed,
                    until: i.until,
                    share: i.share,
                    deaths: i.deaths,
                    finds: i.finds,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let ids: Vec<_> = rules
        .catalog
        .ideologies
        .iter()
        .map(|d| fbb.create_string(&d.id))
        .collect();
    let ids = fbb.create_vector(&ids);
    let diseases: Vec<_> = rules
        .catalog
        .diseases
        .iter()
        .map(|d| fbb.create_string(&d.id))
        .collect();
    let diseases = fbb.create_vector(&diseases);
    let point = |p: (f32, f32)| save::Point::new(p.0, p.1);
    let waves: Vec<_> = inf
        .waves
        .iter()
        .map(|w| {
            let parents: Vec<u64> = w
                .parents
                .iter()
                .flat_map(|(m, f)| [m.get(), f.get()])
                .collect();
            let parents = fbb.create_vector(&parents);
            let people: Vec<u64> = w.people.iter().map(|p| p.get()).collect();
            let people = fbb.create_vector(&people);
            save::WaveSave::create(
                &mut fbb,
                &save::WaveSaveArgs {
                    record: w.record,
                    at: Some(&point(w.at)),
                    edge: Some(&point(w.edge)),
                    day: w.day,
                    households: w.households,
                    days: w.days,
                    provisions_days: w.provisions_days,
                    arrived: w.arrived,
                    came: w.came,
                    next: w.next,
                    settlement: w.settlement.map_or(0, |s| s.get()),
                    band: w.band.map_or(0, |b| b.get()),
                    parents: Some(parents),
                    people: Some(people),
                },
            )
        })
        .collect();
    let waves = fbb.create_vector(&waves);
    let root = save::InfluencesSave::create(
        &mut fbb,
        &save::InfluencesSaveArgs {
            list: Some(list),
            ideologies: Some(ids),
            waves: Some(waves),
            diseases: Some(diseases),
        },
    );
    finish(fbb, root)
}

/// An ideology told of that the loaded content no longer names is let go, as its holdings are.
fn decode_influences(
    bytes: &[u8],
    rules: &Rules,
) -> Result<civ_agents::influence::Influences, LoadError> {
    use civ_agents::influence::{Influence, InfluenceKind, Influences, Wave};
    let root = section_root::<save::InfluencesSave>(bytes)
        .map_err(|e| unreadable(SECTION_INFLUENCE, &e))?;
    let map: Vec<Option<u32>> = read_strings(root.ideologies())
        .iter()
        .map(|id| {
            rules
                .catalog
                .ideologies
                .iter()
                .position(|d| &d.id == id)
                .and_then(|i| u32::try_from(i).ok())
        })
        .collect();
    let diseases: Vec<Option<u32>> = read_strings(root.diseases())
        .iter()
        .map(|id| {
            rules
                .catalog
                .disease_index(id)
                .and_then(|i| u32::try_from(i).ok())
        })
        .collect();
    let mut list = Vec::new();
    for i in root.list().iter().flatten() {
        let kind = InfluenceKind::from_code(i.kind()).ok_or_else(|| {
            LoadError::Malformed(format!("unknown kind of influence {}", i.kind()))
        })?;
        let subject = match kind {
            InfluenceKind::Whisper
            | InfluenceKind::Bless
            | InfluenceKind::Curse
            | InfluenceKind::Wave
            | InfluenceKind::Drought
            | InfluenceKind::Fire
            | InfluenceKind::Flood
            | InfluenceKind::Storm
            | InfluenceKind::Earthquake
            | InfluenceKind::Climate => i.subject(),
            InfluenceKind::Ideology | InfluenceKind::Agitator => {
                match map.get(i.subject() as usize) {
                    Some(Some(k)) => *k,
                    _ => continue,
                }
            }
            // A plague's disease, by its content id at save time: one the loaded content no longer
            // has is let go, as its infections are.
            InfluenceKind::Plague => match diseases.get(i.subject() as usize) {
                Some(Some(k)) => *k,
                _ => continue,
            },
        };
        list.push(Influence {
            id: i.id(),
            at: SimTime::from_minutes(i.at()),
            last: SimTime::from_minutes(i.last()),
            uses: i.uses(),
            kind,
            target: required(i.target(), "an influence's target")?,
            subject,
            taken: (i.taken() >= 0).then_some(i.taken()),
            weighed: i.weighed(),
            until: i.until(),
            share: i.share(),
            deaths: i.deaths(),
            finds: i.finds(),
        });
    }
    if list.windows(2).any(|w| w[0].id >= w[1].id) {
        return Err(LoadError::Malformed("influences out of order".to_owned()));
    }
    let mut waves = Vec::new();
    for w in root.waves().iter().flatten() {
        let point = |p: Option<&save::Point>| p.map_or((0.0, 0.0), |p| (p.x(), p.y()));
        let ids: Vec<u64> = w.parents().map(|v| v.iter().collect()).unwrap_or_default();
        if !ids.len().is_multiple_of(2) {
            return Err(LoadError::Malformed(
                "a wave's parents are not in pairs".to_owned(),
            ));
        }
        let mut parents = Vec::new();
        for pair in ids.chunks(2) {
            parents.push((
                required(pair[0], "a wave's parent")?,
                required(pair[1], "a wave's parent")?,
            ));
        }
        let mut people = Vec::new();
        for id in w.people().iter().flatten() {
            people.push(required(id, "one a wave brought")?);
        }
        waves.push(Wave {
            record: w.record(),
            at: point(w.at()),
            edge: point(w.edge()),
            day: w.day(),
            households: w.households(),
            days: w.days(),
            provisions_days: w.provisions_days(),
            arrived: w.arrived(),
            came: w.came(),
            next: w.next(),
            settlement: PermanentId::from_raw(w.settlement()),
            band: PermanentId::from_raw(w.band()),
            parents,
            people,
        });
    }
    if waves.iter().any(|w| {
        !list
            .iter()
            .any(|i| i.id == w.record && i.kind == InfluenceKind::Wave)
    }) {
        return Err(LoadError::Malformed(
            "a wave without its influence record".to_owned(),
        ));
    }
    Ok(Influences { list, waves })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn rules() -> Rules {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        let content = civ_content::load(&root)
            .registry
            .expect("the core content loads");
        Rules::of(&content)
    }

    fn person(args: &save::PersonArgs) -> Vec<u8> {
        let mut fbb = FlatBufferBuilder::new();
        let root = save::Person::create(&mut fbb, args);
        fbb.finish(root, None);
        fbb.finished_data().to_vec()
    }

    #[test]
    fn carried_food_of_a_schema_2_save_becomes_provisions() {
        let rules = rules();
        let who = PermanentId::from_raw(9).expect("non-zero");
        let bytes = person(&save::PersonArgs {
            carry_food_kcal: 6000.0,
            carry_water_l: 4.0,
            ..Default::default()
        });
        let p = section_root::<save::Person>(&bytes).expect("decodes");
        let load = carried(&p, &[], &rules, Schema::V2, who).expect("migrates");
        let g = rules.people.band.provisions_good;
        assert_eq!(load.good, Some(g as u16));
        let kcal = f64::from(load.kg) * rules.catalog.goods[g].kcal_per_kg;
        assert!((kcal - 6000.0).abs() < 1e-3, "{kcal}");
        assert_eq!(load.water_l, 4.0);

        // Nothing carried stays nothing.
        let bytes = person(&save::PersonArgs::default());
        let p = section_root::<save::Person>(&bytes).expect("decodes");
        let load = carried(&p, &[], &rules, Schema::V2, who).expect("migrates");
        assert_eq!(load, Load::default());
    }

    #[test]
    fn a_carried_good_the_content_dropped_is_dropped() {
        let rules = rules();
        let who = PermanentId::from_raw(9).expect("non-zero");
        let bytes = person(&save::PersonArgs {
            carry_good: 1,
            carry_kg: 12.0,
            ..Default::default()
        });
        let p = section_root::<save::Person>(&bytes).expect("decodes");
        let load = carried(&p, &[Some(0), None], &rules, Schema::V3, who).expect("decodes");
        assert_eq!(load, Load::default());
        let load = carried(&p, &[Some(0), Some(3)], &rules, Schema::V3, who).expect("decodes");
        assert_eq!((load.good, load.kg), (Some(3), 12.0));
        // An index past the saved dictionary is damage.
        assert!(carried(&p, &[Some(0)], &rules, Schema::V3, who).is_err());
    }

    /// A buildings section of one hut, as schema 14 wrote it: `params` of the given length, and no
    /// footprint kind, length, width or angle.
    fn schema_14_hut(params: &[i32]) -> Vec<u8> {
        let mut fbb = FlatBufferBuilder::new();
        let program = fbb.create_string("core:building/hut");
        let params = fbb.create_vector(params);
        let materials = strings(
            &mut fbb,
            &["core:good/timber", "core:good/timber", "core:good/thatch"],
        );
        let spec = save::BuildingSpec::create(
            &mut fbb,
            &save::BuildingSpecArgs {
                program: Some(program),
                version: civ_grammar::HUT_VERSION,
                x_cm: 120_000,
                y_cm: 80_000,
                radius_cm: 310,
                storeys: 1,
                params: Some(params),
                materials: Some(materials),
                style_seed: 7,
                ..Default::default()
            },
        );
        let building = save::Building::create(
            &mut fbb,
            &save::BuildingArgs {
                id: 11,
                household: 12,
                plot: 13,
                spec: Some(spec),
                stage: 2,
                work_h: 5.5,
                started: 60,
                stage_since: 120,
                ..Default::default()
            },
        );
        let list = fbb.create_vector(&[building]);
        let root = save::Builds::create(
            &mut fbb,
            &save::BuildsArgs {
                buildings: Some(list),
            },
        );
        finish(fbb, root)
    }

    #[test]
    fn a_schema_14_design_of_eight_parameters_loads_as_a_round_hut() {
        let eight = [240, 2500, 16_384, 0, 0, 0, 0, 0];
        let loaded = decode_buildings(&schema_14_hut(&eight), Schema::V14).expect("decodes");
        let [b] = loaded.as_slice() else {
            panic!("one building: {loaded:?}")
        };
        assert_eq!(
            b.spec.footprint,
            Footprint::Round {
                x: 120_000,
                y: 80_000,
                radius: 310
            }
        );
        assert_eq!(b.spec.params[..8], eight);
        assert!(
            b.spec.params[8..].iter().all(|&p| p == 0),
            "{:?}",
            b.spec.params
        );
        assert_eq!(
            (b.spec.version, b.spec.storeys, b.spec.style_seed),
            (1, 1, 7)
        );
        assert_eq!((b.stage, b.work_h), (2, 5.5));
        assert_eq!(b.firm, None, "no firm before schema 16");
        // No condition before schema 17: the stage under way was worked at middling skill, and
        // its finished stages' groups are put in place on loading.
        assert!(b.condition.is_empty() && b.repair.is_none());
        assert_eq!(b.state, BuildingState::Standing);
        assert_eq!(b.skill_h, 5.5 * condition::MIDDLING_SKILL as f32);
        // Saved again, it keeps the eight and the eight zeros after them.
        let again = decode_buildings(&encode_buildings(&loaded), Schema::V17).expect("decodes");
        assert_eq!(again, loaded);
        // More parameters than a design has is damage, not a design to cut short.
        assert!(matches!(
            decode_buildings(&schema_14_hut(&[1; PARAMS + 1]), Schema::V14),
            Err(LoadError::Malformed(_))
        ));
    }
}
