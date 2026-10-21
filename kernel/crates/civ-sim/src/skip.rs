//! Skipping ahead (ADR-0032 §7; M9c slice DL): a fast run at Accelerated Max with every
//! settlement lived in household-resolved, stopping at a date, at the observer's word or at a
//! chronicle entry of a kind the observer marks, and putting the speed and the resolutions back
//! after. Nothing is generated after the fact: a skip is the world lived, and it saves and
//! reproduces as any run does.

use std::collections::BTreeSet;

use civ_agents::history::{self, ChronicleEvent, ChronicleKind, Span};
use civ_core::PermanentId;

use crate::{FocusPolicy, SPEED_MAX, Sim};

/// Kinds of chronicle entry a skip may stop at, as the observer marks them: bits of a mask.
pub mod stop {
    /// A settlement founded, taken into a polity or seceding.
    pub const SETTLEMENTS: u8 = 1;
    /// A settlement's food running short.
    pub const HUNGER: u8 = 2;
    /// A fire, a flood, a storm's damage, a building giving way, an outbreak or a raid.
    pub const DISASTER: u8 = 4;
    /// A law decided, a custom amended or taken, a revolt or coup called, an election held, an
    /// alliance or agreement made.
    pub const POLITICS: u8 = 8;
    /// A technique found, or a deposit.
    pub const DISCOVERY: u8 = 16;
    /// Every kind above.
    pub const ALL: u8 = SETTLEMENTS | HUNGER | DISASTER | POLITICS | DISCOVERY;
}

/// The mark a chronicle entry of `kind` falls under, or 0.
pub fn mark_of(kind: ChronicleKind) -> u8 {
    use ChronicleKind as K;
    match kind {
        K::SettlementFounded | K::PolityJoined | K::Seceded => stop::SETTLEMENTS,
        K::FoodRanShort => stop::HUNGER,
        K::Fire | K::Flood | K::StormDamage | K::BuildingFailed | K::Outbreak | K::Raid => {
            stop::DISASTER
        }
        K::LawDecided
        | K::CustomAmended
        | K::CustomTaken
        | K::RevoltCalled
        | K::CoupCalled
        | K::ElectionHeld
        | K::AllianceMade
        | K::Agreement => stop::POLITICS,
        K::TechniqueFound | K::DepositFound => stop::DISCOVERY,
        _ => 0,
    }
}

/// What a skip changed, to be put back when it ends.
#[derive(Debug)]
pub struct SkipRestore {
    speed: f32,
    focus: Option<FocusPolicy>,
    resolved: BTreeSet<PermanentId>,
}

impl Sim {
    /// Begins a skip: Accelerated Max, and every settlement lived in household-resolved from the
    /// next midnight (now, if the clock stands at one). Whatever holds a settlement Detailed (a
    /// fire, a band) still holds it.
    pub fn begin_skip(&mut self) -> SkipRestore {
        let restore = SkipRestore {
            speed: self.speed(),
            focus: self.focus_policy.take(),
            resolved: self.household_resolved().clone(),
        };
        // Max is always a speed the clock takes.
        let _ = self.set_speed(SPEED_MAX);
        self.focus_policy = Some(FocusPolicy::everything());
        restore
    }

    /// Ends a skip: the speed and the observer's choice of resolutions as they were before it,
    /// from the next midnight.
    pub fn end_skip(&mut self, restore: SkipRestore) {
        let _ = self.set_speed(restore.speed);
        self.focus_policy = restore.focus;
        if self.focus_policy.is_none() {
            self.set_household_resolved(restore.resolved);
        }
    }

    /// The first chronicle entry after `after_seq` of a kind marked in `marks`, if any.
    pub fn skip_stop(&self, after_seq: u64, marks: u8) -> Option<&ChronicleEvent> {
        if marks == 0 {
            return None;
        }
        self.people
            .chronicle
            .iter()
            .rev()
            .take_while(|e| e.seq > after_seq)
            .filter(|e| mark_of(e.kind) & marks != 0)
            .last()
    }

    /// A chronicle entry in plain words, as the chronicle tells it.
    pub fn chronicle_words(&self, e: &ChronicleEvent) -> String {
        let name_of = |id: PermanentId| self.people.name_of(id);
        history::render(e, &name_of)
            .iter()
            .map(|s| match s {
                Span::Text(t) | Span::Person(_, t) | Span::Settlement(_, t) | Span::Firm(_, t) => {
                    t.as_str()
                }
            })
            .collect()
    }

    /// The newest chronicle entry's sequence number (0: none).
    pub fn chronicle_head(&self) -> u64 {
        self.people.chronicle.last().map_or(0, |e| e.seq)
    }
}
