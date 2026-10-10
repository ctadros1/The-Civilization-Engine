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

use civ_agents::caution::Trust;
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
    SCHEMA_V57, SCHEMA_V58, SCHEMA_V59, SCHEMA_V60, SCHEMA_V61, SCHEMA_V62, SCHEMA_V63, finish,
    section, single_chunk, unreadable,
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
            encode_households(&sim.people, &goods, &resources),
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
            encode_firms(&sim.people, &goods, &activities),
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
        section(SECTION_RELATIONS, 0, encode_relations(&sim.people)),
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
        SAVE_SCHEMA_VERSION => Schema::V64,
        other => {
            return Err(LoadError::Incompatible(format!(
                "world schema version {other} has no people-and-land decoder"
            )));
        }
    };
    let bytes = single_chunk(reader, SECTION_LAND)?;
    let mut land = decode_land(&bytes, rules, map, climatology)?;
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
        decode_earth(&bytes, &mut land)?;
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
        );
    }
    // What people believe of other polities (schema 63); before, nobody held a view.
    if schema >= Schema::V63 {
        let bytes = single_chunk(reader, SECTION_RELATIONS)?;
        (people.polity_views, people.claims_heard) = decode_relations(&bytes, rules)?;
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
        if people.polities[..i]
            .iter()
            .any(|o| o.settlement == p.settlement)
        {
            problems.push(format!("settlement {} has two polities", p.settlement));
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
                            ends: l.ends.map_or(0, PermanentId::get),
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
                            stance_opinions: Some(stance_opinions),
                            stance_values: Some(stance_values),
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
    }
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
    let root =
        flatbuffers::root::<save::Polities>(bytes).map_err(|e| unreadable(SECTION_POLITY, &e))?;
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
        let body_of = |members: u8, quorum_share: f32, pass: u8, what: &str| {
            Ok::<Body, LoadError>(Body {
                members: Membership::from_code(members)
                    .ok_or_else(|| bad(format!("{what} has membership code {members}")))?,
                quorum_share,
                pass: PassRule::from_code(pass)
                    .ok_or_else(|| bad(format!("{what} has decision rule code {pass}")))?,
            })
        };
        let body = body_of(
            p.members(),
            p.quorum_share(),
            p.pass(),
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
                },
                hours: (l.from_hour(), l.to_hour()),
                status,
                ends: PermanentId::from_raw(l.ends()),
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
        for (k, v) in p.versions().iter().flatten().enumerate() {
            versions.push(CustomVersion {
                body: body_of(
                    v.members(),
                    v.quorum_share(),
                    v.pass(),
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
    let root = save::Earth::create(
        &mut fbb,
        &save::EarthArgs {
            works: Some(works),
            tiles: Some(tiles),
            links: Some(links),
        },
    );
    finish(fbb, root)
}

fn decode_earth(bytes: &[u8], land: &mut Land) -> Result<(), LoadError> {
    let root =
        flatbuffers::root::<save::Earth>(bytes).map_err(|e| unreadable(SECTION_EARTH, &e))?;
    for w in root.works().iter().flatten() {
        let kind = match w.kind() {
            0 => civ_land::earth::EarthKind::Platform,
            1 => civ_land::earth::EarthKind::Pit,
            2 => civ_land::earth::EarthKind::Spoil,
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
    let root = flatbuffers::root::<save::Ties>(bytes).map_err(|e| unreadable(SECTION_TIES, &e))?;
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
        flatbuffers::root::<save::Deposits>(bytes).map_err(|e| unreadable(SECTION_DEPOSITS, &e))?;
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
                },
            )
        })
        .collect();
    let months = fbb.create_vector(&months);
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
        },
        anomaly: w.anomaly(),
        slow: w.slow(),
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
            })
            .collect(),
    })
}

fn decode_land(
    bytes: &[u8],
    rules: &Rules,
    map: &WorldMap,
    climatology: Climatology,
) -> Result<Land, LoadError> {
    let l = flatbuffers::root::<save::Land>(bytes).map_err(|e| unreadable(SECTION_LAND, &e))?;
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
        Some(w) => decode_weather(w).ok_or_else(|| {
            LoadError::Malformed("section `land` has weather without a day".to_owned())
        })?,
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
        ground: civ_land::earth::GroundDelta::new(map.width, map.height, map.cell_size_m),
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
    let s = flatbuffers::root::<save::Settlements>(bytes)
        .map_err(|e| unreadable(SECTION_SETTLE, &e))?;
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
            let (kind, to, minutes) = match *s {
                Step::Walk { to } => (save::StepKind::Walk, to, 0),
                Step::Work { minutes } => (save::StepKind::Work, (0.0, 0.0), minutes),
                Step::Deposit => (save::StepKind::Deposit, (0.0, 0.0), 0),
                Step::Wait { minutes } => (save::StepKind::Wait, (0.0, 0.0), minutes),
            };
            save::Step::create(
                fbb,
                &save::StepArgs {
                    kind,
                    to: Some(&point(to)),
                    minutes,
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
        save::Trip::create(
            fbb,
            &save::TripArgs {
                id: t.id,
                rev: t.rev,
                depart: t.depart.minutes(),
                points: Some(points),
                minutes: Some(minutes),
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
    let root =
        flatbuffers::root::<save::People>(bytes).map_err(|e| unreadable(SECTION_PEOPLE, &e))?;
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
        | Schema::V64 => {
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

fn encode_households(pop: &Population, goods: &[&str], resources: &[&str]) -> Vec<u8> {
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
        flatbuffers::root::<save::Households>(bytes).map_err(|e| unreadable(SECTION_HOUSES, &e))?;
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
            | Schema::V64 => {
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
            taste: match h.taste() {
                Some(t) if schema >= Schema::V22 && t.len() == 3 => {
                    let t: Vec<f32> = t.iter().collect();
                    if t.iter().any(|v| !v.is_finite()) {
                        return Err(LoadError::Malformed(format!(
                            "household {hh_id} has a taste that is not a number"
                        )));
                    }
                    civ_agents::params::Taste::from_traits([t[0], t[1], t[2]])
                }
                Some(_) if schema >= Schema::V22 => {
                    return Err(LoadError::Malformed(format!(
                        "household {hh_id} has a taste of other than three traits"
                    )));
                }
                _ => civ_agents::params::Taste::default(),
            },
            admired: id(h.admired()),
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
    let amounts: Vec<save::AmountSave> = order
        .amounts
        .iter()
        .map(|a| save::AmountSave::new(a.household.get(), a.day, a.incident, a.kcal, a.lost))
        .collect();
    let amounts = fbb.create_vector(&amounts);
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
    let root =
        flatbuffers::root::<save::Order>(bytes).map_err(|e| unreadable(SECTION_ORDER, &e))?;
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
        for r in c.stances().iter().flatten() {
            stances.push(civ_agents::crime::CaseStance {
                person: required(r.person(), "someone at a hearing")?,
                household: required(r.household(), "a household at a hearing")?,
                stance: stance_from(r.stance())
                    .ok_or_else(|| bad(format!("case {cid} has stance code {}", r.stance())))?,
                stake: r.stake(),
                belief: r.belief(),
                regard: r.regard(),
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
    if let Some(o) = order.obligations.iter().find(|o| !known(o.incident)) {
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
    if let Some(c) = order.cases.iter().find(|c| !known(c.incident)) {
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
    let root =
        flatbuffers::root::<save::History>(bytes).map_err(|e| unreadable(SECTION_HISTORY, &e))?;
    let mut records = BTreeMap::new();
    for r in root.records().iter().flatten() {
        let rid = required(r.id(), "a person record")?;
        let cause = match r.cause() {
            save::Cause::Unspecified => Cause::Unspecified,
            save::Cause::Starvation => Cause::Starvation,
            save::Cause::Childbirth => Cause::Childbirth,
            save::Cause::Collapse => Cause::Collapse,
            save::Cause::Violence => Cause::Violence,
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
        flatbuffers::root::<save::Receipts>(bytes).map_err(|e| unreadable(SECTION_RECEIPTS, &e))?;
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
    let root =
        flatbuffers::root::<save::Fields>(bytes).map_err(|e| unreadable(SECTION_FIELDS, &e))?;
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
                    },
                    since: p.since.minutes(),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let root = save::Plots::create(&mut fbb, &save::PlotsArgs { plots: Some(list) });
    finish(fbb, root)
}

fn decode_plots(bytes: &[u8]) -> Result<Vec<Plot>, LoadError> {
    let root =
        flatbuffers::root::<save::Plots>(bytes).map_err(|e| unreadable(SECTION_PLOTS, &e))?;
    let mut out = Vec::new();
    for p in root.plots().iter().flatten() {
        let id = required(p.id(), "a plot")?;
        let use_ = match p.purpose() {
            save::PlotUse::Dwelling => PlotUse::Dwelling,
            save::PlotUse::Store => PlotUse::Store,
            save::PlotUse::Work => PlotUse::Work,
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
            let params = fbb.create_vector(&s.params);
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
    let root =
        flatbuffers::root::<save::Builds>(bytes).map_err(|e| unreadable(SECTION_BUILDS, &e))?;
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
    let w = flatbuffers::root::<save::Wear>(bytes).map_err(|e| unreadable(SECTION_WEAR, &e))?;
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
    let root =
        flatbuffers::root::<save::Markets>(bytes).map_err(|e| unreadable(SECTION_MARKET, &e))?;
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

fn encode_firms(pop: &Population, goods: &[&str], activities: &[&str]) -> Vec<u8> {
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
                    },
                )
            });
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
    let root =
        flatbuffers::root::<save::Firms>(bytes).map_err(|e| unreadable(SECTION_FIRMS, &e))?;
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
    let root =
        flatbuffers::root::<save::Wealth>(bytes).map_err(|e| unreadable(SECTION_WEALTH, &e))?;
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
    let root = save::Knowledge::create(
        &mut fbb,
        &save::KnowledgeArgs {
            techniques: Some(dictionary),
            entries: Some(entries),
            trust: Some(trust),
            elsewhere,
        },
    );
    finish(fbb, root)
}

/// What each settlement has seen of each technique's buildings (schema 18), from the knowledge
/// section. A technique the loaded content no longer has drops out.
fn decode_trust(bytes: &[u8], rules: &Rules) -> Result<Vec<Trust>, LoadError> {
    let root =
        flatbuffers::root::<save::Knowledge>(bytes).map_err(|e| unreadable(SECTION_KNOW, &e))?;
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

fn decode_knowledge(bytes: &[u8], rules: &Rules) -> Result<Vec<KnowledgeEvent>, LoadError> {
    let root =
        flatbuffers::root::<save::Knowledge>(bytes).map_err(|e| unreadable(SECTION_KNOW, &e))?;
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
            let SimEvent::Agent(AgentEvent::Step { person, version }) = e.event;
            save::PendingEvent::create(
                &mut fbb,
                &save::PendingEventArgs {
                    at: e.at.minutes(),
                    phase: e.phase,
                    tiebreak: e.tiebreak,
                    seq: e.seq,
                    kind: save::EventKind::AgentStep,
                    person: person.get(),
                    version,
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
    let root = flatbuffers::root::<save::PendingEvents>(bytes)
        .map_err(|e| unreadable(SECTION_EVENTS, &e))?;
    let mut out = Vec::new();
    for e in root.events().iter().flatten() {
        let event = match e.kind() {
            save::EventKind::AgentStep => SimEvent::Agent(AgentEvent::Step {
                person: required(e.person(), "an event's person")?,
                version: e.version(),
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
    let root = flatbuffers::root::<save::OpinionSave>(bytes)
        .map_err(|e| unreadable(SECTION_OPINION, &e))?;
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
    let root = save::WordSave::create(
        &mut fbb,
        &save::WordSaveArgs {
            claims: Some(claims),
            heard: Some(heard),
            grievances: Some(grievances),
            next: word.next,
        },
    );
    finish(fbb, root)
}

fn decode_word(bytes: &[u8]) -> Result<civ_agents::word::Word, LoadError> {
    use civ_agents::word::{Blamed, Claim, ClaimKind, Grievance, Grieved, Heard, Word, Wrong};
    let root =
        flatbuffers::root::<save::WordSave>(bytes).map_err(|e| unreadable(SECTION_WORD, &e))?;
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
        word.claims.push(Claim {
            id: c.id(),
            kind,
            settlement: required(c.settlement(), "a claim's settlement")?,
            day: c.day(),
            subject: id(c.subject()),
            grievance,
        });
    }
    if !word.claims.windows(2).all(|w| w[0].id < w[1].id) {
        return Err(bad("claims are out of order".to_owned()));
    }
    for h in root.heard().iter().flatten() {
        word.heard.push(Heard {
            holder: required(h.holder(), "a hearer")?,
            claim: h.claim(),
            first: h.first(),
            last: h.last(),
            from: id(h.from()),
            origin: id(h.origin()),
        });
    }
    if !word
        .heard
        .windows(2)
        .all(|w| (w[0].holder, w[0].claim) < (w[1].holder, w[1].claim))
    {
        return Err(bad("what people heard is out of order".to_owned()));
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
    let root =
        flatbuffers::root::<save::NormsSave>(bytes).map_err(|e| unreadable(SECTION_NORMS, &e))?;
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
        flatbuffers::root::<save::ValuesSave>(bytes).map_err(|e| unreadable(SECTION_VALUES, &e))?;
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
    let root = flatbuffers::root::<save::IdeologiesSave>(bytes)
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
    let root = flatbuffers::root::<save::FactionsSave>(bytes)
        .map_err(|e| unreadable(SECTION_FACTIONS, &e))?;
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
}

fn decode_places(bytes: &[u8], rules: &Rules) -> Result<PlacesDecoded, LoadError> {
    use civ_agents::places::{Contact, Contacts, KnownPlace, PlaceHow, Places};
    let root =
        flatbuffers::root::<save::PlacesSave>(bytes).map_err(|e| unreadable(SECTION_PLACES, &e))?;
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
        uses.households
            .entry(household)
            .or_default()
            .push(civ_agents::uses::PlaceUse {
                place: place_of(u.kind(), u.place())?,
                days: u.days(),
                kcal: u.kcal(),
                outsiders,
                day: u.day(),
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
    Ok(PlacesDecoded {
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

fn encode_relations(people: &Population) -> Vec<u8> {
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
    let root = save::RelationsSave::create(&mut fbb, &save::RelationsSaveArgs { views, heard });
    finish(fbb, root)
}

fn decode_relations(
    bytes: &[u8],
    rules: &Rules,
) -> Result<(civ_agents::views::Views, civ_agents::uses::ClaimsHeard), LoadError> {
    use civ_agents::views::{DOMAINS, View, ViewAct, ViewReason, Views};
    let root = flatbuffers::root::<save::RelationsSave>(bytes)
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
    Ok((out, heard))
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
    let root = flatbuffers::root::<save::InfluencesSave>(bytes)
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
    let mut list = Vec::new();
    for i in root.list().iter().flatten() {
        let kind = InfluenceKind::from_code(i.kind()).ok_or_else(|| {
            LoadError::Malformed(format!("unknown kind of influence {}", i.kind()))
        })?;
        let subject = match kind {
            InfluenceKind::Whisper
            | InfluenceKind::Bless
            | InfluenceKind::Curse
            | InfluenceKind::Wave => i.subject(),
            InfluenceKind::Ideology | InfluenceKind::Agitator => {
                match map.get(i.subject() as usize) {
                    Some(Some(k)) => *k,
                    _ => continue,
                }
            }
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
        let p = flatbuffers::root::<save::Person>(&bytes).expect("decodes");
        let load = carried(&p, &[], &rules, Schema::V2, who).expect("migrates");
        let g = rules.people.band.provisions_good;
        assert_eq!(load.good, Some(g as u16));
        let kcal = f64::from(load.kg) * rules.catalog.goods[g].kcal_per_kg;
        assert!((kcal - 6000.0).abs() < 1e-3, "{kcal}");
        assert_eq!(load.water_l, 4.0);

        // Nothing carried stays nothing.
        let bytes = person(&save::PersonArgs::default());
        let p = flatbuffers::root::<save::Person>(&bytes).expect("decodes");
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
        let p = flatbuffers::root::<save::Person>(&bytes).expect("decodes");
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
