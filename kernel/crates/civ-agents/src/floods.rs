//! High-water, flood and storm episodes (M6b slice BF): hydraulic inundation, displacement,
//! goods spoilage, scour collapse of crossings, and storm uplift damage.

use civ_core::{PermanentId, SimTime};

/// Recorded hazard provenance. Old snapshots have an unknown source; observations never
/// reveal this kernel-only attribution to people.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Origin {
    pub known: bool,
    /// Every observer input that changed the hazard, in id order.
    pub observers: Vec<u32>,
}
impl Origin {
    pub fn recorded(mut observers: Vec<u32>) -> Self {
        observers.sort_unstable();
        observers.dedup();
        Self {
            known: true,
            observers,
        }
    }
    pub fn join(&mut self, other: &Self) {
        self.known &= other.known;
        self.observers.extend_from_slice(&other.observers);
        self.observers.sort_unstable();
        self.observers.dedup();
    }
    pub fn natural(&self) -> bool {
        self.known && self.observers.is_empty()
    }
}

/// An incident of high water on a river reach (M6b slice BF).
#[derive(Clone, Debug, PartialEq)]
pub struct HighWater {
    pub origin: Origin,
    /// Its id, from 1.
    pub id: u32,
    /// Reach index on the river network.
    pub reach: u16,
    /// Settlement it affects, if any.
    pub settlement: Option<PermanentId>,
    /// Centroid or representative location of the reach, metres.
    pub at: (f32, f32),
    /// When flood stage began.
    pub began: SimTime,
    /// When floodwaters receded.
    pub ended: Option<SimTime>,
    /// Peak discharge during the event, m³/s.
    pub peak_discharge: f32,
    /// Peak stage above bed, metres.
    pub peak_stage: f32,
    /// Flood episode it belongs to, if part of a multi-reach or multi-day flood event.
    pub episode: Option<u32>,
    /// Number of buildings flooded.
    pub buildings_flooded: u32,
    /// Number of fields submerged.
    pub fields_flooded: u32,
    /// Kilograms of floor-stored goods spoiled.
    pub spoiled_kg: f32,
    /// Crossings scoured away by high water.
    pub scoured_crossings: u32,
}

/// A flood episode: high waters across reaches (M6b slice BF: an episode).
#[derive(Clone, Debug, PartialEq)]
pub struct FloodEpisode {
    pub origin: Origin,
    /// Its id, from 1.
    pub id: u32,
    /// Settlement where it began or primarily impacts.
    pub settlement: Option<PermanentId>,
    /// First reach where bankfull discharge was exceeded.
    pub first_reach: u16,
    /// When the flood episode began.
    pub began: SimTime,
    /// When all high water receded.
    pub ended: Option<SimTime>,
    /// Peak discharge observed, m³/s.
    pub peak_discharge: f32,
    /// High-water incident ids that are part of this episode.
    pub high_waters: Vec<u32>,
    /// Total kilograms of goods spoiled.
    pub total_spoiled_kg: f32,
    /// Total buildings inundated.
    pub total_buildings: u32,
    /// Total fields lost (submerged >= 5 days).
    pub total_fields_lost: u32,
}

/// An incident of storm damage to a building (M6b slice BF).
#[derive(Clone, Debug, PartialEq)]
pub struct StormDamage {
    pub origin: Origin,
    /// Its id, from 1.
    pub id: u32,
    /// The building damaged.
    pub building: PermanentId,
    /// The household owning the building.
    pub household: PermanentId,
    /// Settlement where it stands.
    pub settlement: Option<PermanentId>,
    /// Location of the building, metres.
    pub at: (f32, f32),
    /// When the damage occurred.
    pub time: SimTime,
    /// Wind speed in m/s at the time.
    pub wind_speed_ms: f32,
    /// Dynamic wind uplift pressure, Pa: q = 0.5 * rho * V^2.
    pub uplift_pa: f32,
    /// Storm episode it belongs to.
    pub episode: Option<u32>,
}

/// A storm episode: windstorm crossing the world (M6b slice BF).
#[derive(Clone, Debug, PartialEq)]
pub struct StormEpisode {
    pub origin: Origin,
    /// Its id, from 1.
    pub id: u32,
    /// Settlement it affects, if any.
    pub settlement: Option<PermanentId>,
    /// When the storm began.
    pub began: SimTime,
    /// When the storm passed.
    pub ended: Option<SimTime>,
    /// Peak wind speed, m/s.
    pub peak_wind_speed_ms: f32,
    /// Damage incident ids.
    pub damages: Vec<u32>,
}

/// Container for all flood and storm incidents and episodes (M6b slice BF).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Floods {
    pub high_waters: Vec<HighWater>,
    pub flood_episodes: Vec<FloodEpisode>,
    pub storm_damages: Vec<StormDamage>,
    pub storm_episodes: Vec<StormEpisode>,
}

impl Floods {
    /// Returns true if any high water incident is currently active.
    pub fn active_flood(&self) -> bool {
        self.high_waters.iter().any(|hw| hw.ended.is_none())
    }

    /// Returns true if any storm episode is currently active.
    pub fn active_storm(&self) -> bool {
        self.storm_episodes.iter().any(|st| st.ended.is_none())
    }

    /// The latest active flood episode, if any.
    pub fn current_flood_episode(&self) -> Option<&FloodEpisode> {
        self.flood_episodes.iter().find(|e| e.ended.is_none())
    }

    /// The latest active flood episode mutably, if any.
    pub fn current_flood_episode_mut(&mut self) -> Option<&mut FloodEpisode> {
        self.flood_episodes.iter_mut().find(|e| e.ended.is_none())
    }

    /// The latest active storm episode, if any.
    pub fn current_storm_episode(&self) -> Option<&StormEpisode> {
        self.storm_episodes.iter().find(|e| e.ended.is_none())
    }

    /// The latest active storm episode mutably, if any.
    pub fn current_storm_episode_mut(&mut self) -> Option<&mut StormEpisode> {
        self.storm_episodes.iter_mut().find(|e| e.ended.is_none())
    }
}
