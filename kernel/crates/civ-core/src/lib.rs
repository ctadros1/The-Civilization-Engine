//! Kernel foundations shared by every simulation crate.
//!
//! - [`ids`]: generational handles for live rows, permanent ids for history (research 01-02 §1.2).
//! - [`time`]: one tick is one in-game minute; a 365-day numbered calendar (plan §4.4).
//! - [`scheduler`]: event-scheduled work plus periodic cadence boundaries (plan §4.4, research
//!   01-02 §1.3).
//! - [`rng`]: small in-house generators, so a seed keeps meaning the same world across dependency
//!   upgrades (research 01-01 §4.2).
//!
//! Nothing here knows about people, settlements or terrain.

#![forbid(unsafe_code)]

pub mod ids;
pub mod rng;
pub mod scheduler;
pub mod time;

pub use ids::{GenTable, Handle, IdAllocator, PermanentId};
pub use rng::{Rng64, SplitMix64};
pub use scheduler::{Cadence, Due, Followups, ScheduleError, Scheduler};
pub use time::{Date, Season, SimTime};
