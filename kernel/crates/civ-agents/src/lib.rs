//! People (plan §4.1; ADR-0003).
//!
//! - [`params`]: what is authored about people and their activities (filled by `civ-content`).
//! - [`needs`]: closed-form need and body arithmetic (lazy needs).
//! - [`person`]: people, households, activities, trips.
//! - [`decide`]: scoring candidate activities and sampling one.
//! - [`farm`]: what a household plans to grow, and what its field work is worth.
//! - [`build`]: the hut a household designs, the ground it claims, and the work left on it.
//! - [`population`]: the tables and the event-driven activity engine.
//! - [`found`]: a founding band arriving and choosing its camp.
//! - [`history`]: person records, unions, decision receipts and the chronicle.
//! - [`demography`]: births, deaths and couples as pure rules, applied by the population daily.
//!
//! The engine authors the vocabulary, never the plot (plan §1): activities, needs and their
//! weights are content; what people do, and where, follows from their circumstances.

#![forbid(unsafe_code)]

pub mod build;
pub mod decide;
pub mod demography;
pub mod farm;
pub mod found;
pub mod history;
pub mod needs;
pub mod params;
pub mod person;
pub mod population;

pub use found::{Founded, found_band};
pub use history::{
    Cause, ChronicleEvent, ChronicleKind, Moved, Origin, PersonRecord, Reason, Receipt, Scored,
    Span, Term, Union,
};
pub use needs::Sex;
pub use params::{ActivityDef, Behavior, Catalog, PeopleParams};
pub use person::{
    Activity, Household, KnownPatch, Load, Person, Repro, Step, Target, Traits, Trip,
};
pub use population::{AgentEvent, Ctx, Population};
