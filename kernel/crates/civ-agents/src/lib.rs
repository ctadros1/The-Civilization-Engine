//! People (plan §4.1; ADR-0003).
//!
//! - [`params`]: what is authored about people and their activities (filled by `civ-content`).
//! - [`needs`]: closed-form need and body arithmetic (lazy needs).
//! - [`person`]: people, households, activities, trips.
//! - [`decide`]: scoring candidate activities and sampling one.
//! - [`farm`]: what a household plans to grow, and what its field work is worth.
//! - [`build`]: the hut a household designs, the ground it claims, and the work left on it.
//! - [`caution`]: what a settlement has seen of a technique's buildings, and how cautiously it builds.
//! - [`make`]: recipes and tools as arithmetic on a household's stores (ADR-0006).
//! - [`ledger`]: the channels goods move between households by, and trades (ADR-0006 §3).
//! - [`value`]: what goods cost a household in hours of its own work.
//! - [`market`]: a settlement's offers, trades, money and recorded demand (ADR-0006 §4).
//! - [`population`]: the tables and the event-driven activity engine.
//! - [`found`]: a founding band arriving and choosing its camp.
//! - [`history`]: person records, unions, decision receipts and the chronicle.
//! - [`demography`]: births, deaths and couples as pure rules, applied by the population daily.
//! - [`wealth`]: what households have, measured several ways, and how it spreads (ADR-0007 §4).
//! - [`ties`]: what people remember of one another, written by the acts they see (ADR-0014).
//! - [`standing`]: what a settlement's adults think of one another, summed monthly, and notables.
//! - [`crime`]: takings as the kernel's truth, what people believe of them, and what is owed.
//!
//! The engine authors the vocabulary, never the plot (plan §1): activities, needs and their
//! weights are content; what people do, and where, follows from their circumstances.

#![forbid(unsafe_code)]

pub mod agreements;
pub mod bridge;
pub mod build;
pub mod caution;
pub mod condition;
pub mod convergence;
pub mod crime;
pub mod decide;
pub mod demography;
pub mod faction;
pub mod farm;
pub mod firm;
pub mod found;
pub mod history;
pub mod ideology;
pub mod influence;
pub mod knowledge;
pub mod ledger;
pub mod make;
pub mod market;
pub mod needs;
pub mod norm;
pub mod opinion;
pub mod params;
pub mod person;
pub mod places;
pub mod polity;
pub mod population;
pub mod reports;
pub mod standing;
pub mod structure;
pub mod style;
pub mod ties;
pub mod uses;
pub mod value;
pub mod values;
pub mod views;
pub mod wealth;
pub mod word;

pub use found::{
    Founded, MAX_FOUNDING_GROUPS, MAX_SPAWN_FAMILIES, Spawned, WAVE_KNOWN_M, found_band,
    found_bands, send_agitator, send_wave, spawn_families, spawn_family,
};
pub use history::{
    AgreementStep, Cause, ChronicleEvent, ChronicleKind, CoalitionStep, CrossingStep, Moved,
    Origin, PersonRecord, Reason, Receipt, ResidenceWhy, Scored, Span, Stay, Term, Union,
};
pub use ledger::Channel;
pub use needs::Sex;
pub use params::{ActivityDef, Behavior, Catalog, PeopleParams, RecipeDef, SkillDef};
pub use person::{
    Activity, Household, KnownPatch, Load, Person, Repro, Step, Target, Traits, Trip,
};
pub use population::{AgentEvent, Approximations, Ctx, Population, TimeUse};
