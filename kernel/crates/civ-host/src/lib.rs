//! The headless host of The Civilization Engine (plan §3): a command line for worlds, saves,
//! content and smoke seeds, and the localhost server the web observer talks to.
//!
//! - [`engine`]: the thread that owns the current world.
//! - [`server`]: HTTP and the observer WebSocket (ADR-0001), on 127.0.0.1 only.
//! - [`protocol`]: requests and the host's payloads.
//! - [`session`]: the saves directory, the session marker and crash recovery.
//! - [`deliberator`]: rules, recorded answers or a model's, for notables' moves (ADR-0039 §3).
//! - [`commands`], [`smoke`], [`economy`], [`report`], [`weather`] and [`paths`]: the command
//!   line.

#![forbid(unsafe_code)]

pub mod commands;
pub mod consistency;
pub mod crossings;
pub mod dashboard;
pub mod deliberator;
pub mod economy;
pub mod engine;
pub mod neighbours;
pub mod notables;
pub mod paths;
pub mod protocol;
pub mod report;
pub mod server;
pub mod session;
pub mod smoke;
pub mod trade;
pub mod twin;
pub mod weather;
