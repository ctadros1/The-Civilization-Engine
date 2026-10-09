//! Where people live and how they came there (ADR-0018 §2–§3): each person's residence history,
//! written as they are born, arrive, move, are taken in, marry, leave or are sent away; each
//! settlement's accounts, derived from those histories and never kept beside them; and the day a
//! settlement's last resident died or left.

use std::collections::BTreeMap;

use civ_core::{PermanentId, SimTime};

use super::Population;
use crate::Ctx;
use crate::history::{ChronicleKind, Origin, PersonRecord, ResidenceWhy, Stay};

/// A settlement's accounts over a span of time (ADR-0018 §3), derived from the residence
/// histories and the records of births and deaths. Everything at a minute from the span's start
/// up to, not including, its end counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Accounts {
    /// Residents when it began.
    pub start: u32,
    /// Residents when it ended.
    pub end: u32,
    /// Children born to its households.
    pub births: u32,
    /// Residents who died.
    pub deaths: u32,
    /// People who came to live there, by where they came from (none: off the map).
    pub arrivals: BTreeMap<Option<PermanentId>, u32>,
    /// Residents who went to live elsewhere, by where they went (none: off the map).
    pub departures: BTreeMap<Option<PermanentId>, u32>,
}

impl Accounts {
    /// Everyone who came to live there.
    pub fn arrived(&self) -> u32 {
        self.arrivals.values().sum()
    }

    /// Everyone who went to live elsewhere.
    pub fn departed(&self) -> u32 {
        self.departures.values().sum()
    }

    /// Whether births less deaths plus arrivals less departures is the change in residents.
    pub fn balance(&self) -> bool {
        i64::from(self.start) + i64::from(self.births) + i64::from(self.arrived())
            == i64::from(self.end) + i64::from(self.deaths) + i64::from(self.departed())
    }
}

/// Whether `r` lived in `s` at `t` (after everything at an earlier minute).
fn resident(r: &PersonRecord, s: PermanentId, t: SimTime) -> bool {
    r.died.is_none_or(|(d, _)| d >= t) && r.residence_at(t) == Some(s)
}

/// Whether `t` is in the span from `from` up to `to`.
fn within(t: SimTime, from: SimTime, to: SimTime) -> bool {
    from <= t && t < to
}

impl Population {
    /// `id` lives in `settlement` from `now` (none: off the map), for `why` (ADR-0018 §2). Nothing
    /// is noted when that is where they already live.
    pub(crate) fn note_residence(
        &mut self,
        id: PermanentId,
        settlement: Option<PermanentId>,
        now: SimTime,
        why: ResidenceWhy,
    ) {
        let Some(r) = self.records.get_mut(&id) else {
            return;
        };
        if r.residence
            .last()
            .is_some_and(|x| x.settlement == settlement)
        {
            return;
        }
        r.residence.push(Stay {
            settlement,
            since: now,
            why,
        });
    }

    /// Settlement `s`'s accounts from `from` to `to` (ADR-0018 §3).
    pub fn accounts(&self, s: PermanentId, from: SimTime, to: SimTime) -> Accounts {
        let mut out = Accounts::default();
        for r in self.records.values() {
            out.start += u32::from(resident(r, s, from));
            out.end += u32::from(resident(r, s, to));
            for (i, stay) in r.residence.iter().enumerate() {
                if !within(stay.since, from, to) {
                    continue;
                }
                let before = i.checked_sub(1).and_then(|k| r.residence.get(k));
                let came_from = before.and_then(|b| b.settlement);
                if stay.settlement == Some(s) && before.is_none_or(|b| b.settlement != Some(s)) {
                    if stay.why == ResidenceWhy::Born {
                        out.births += 1;
                    } else {
                        *out.arrivals.entry(came_from).or_default() += 1;
                    }
                }
                if before.is_some_and(|b| b.settlement == Some(s)) && stay.settlement != Some(s) {
                    *out.departures.entry(stay.settlement).or_default() += 1;
                }
            }
            // A death counts against where they lived when they died, everything at that minute
            // done.
            if let Some((d, _)) = r.died
                && within(d, from, to)
                && r.residence
                    .iter()
                    .take_while(|x| x.since <= d)
                    .last()
                    .and_then(|x| x.settlement)
                    == Some(s)
            {
                out.deaths += 1;
            }
        }
        out
    }

    /// Living people whose household is in settlement `s`, as the world stands.
    pub fn residents(&self, s: PermanentId) -> u32 {
        self.people
            .iter()
            .filter(|(_, p)| self.household(p.household).and_then(|x| x.settlement) == Some(s))
            .count() as u32
    }

    /// Where the residence histories disagree with the world (ADR-0018 §3's hard check): someone
    /// living whose last stay is not their household's settlement, or someone who left the world
    /// whose last stay is not off the map.
    pub fn residence_problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (_, p) in self.people.iter() {
            let home = self.household(p.household).and_then(|x| x.settlement);
            let last = self
                .records
                .get(&p.id)
                .and_then(|r| r.residence.last())
                .map(|x| x.settlement);
            if last != Some(home) {
                out.push(format!(
                    "person {} lives in {home:?} by their household and {last:?} by their \
                     residence history",
                    p.id
                ));
            }
        }
        for r in self.records.values().filter(|r| r.left.is_some()) {
            if r.residence.last().is_none_or(|x| x.settlement.is_some()) {
                out.push(format!(
                    "person {} left the world and their residence history does not say so",
                    r.id
                ));
            }
        }
        out
    }

    /// Settlements whose last resident has died or left since yesterday are abandoned today
    /// (ADR-0018 §1). Their records stay.
    pub(crate) fn note_abandoned(&self, ctx: &mut Ctx) {
        let now = ctx.now;
        for s in ctx.land.settlements.iter_mut() {
            if s.abandoned.is_none() && self.residents(s.id) == 0 {
                s.abandoned = Some(now);
            }
        }
    }

    /// Residence histories for a world saved before they were kept (schema 50): for each person,
    /// where they came to live and, if they left, that they went. Their settlement is their
    /// household's if they are alive, else the one the chronicle named where it told of their
    /// death or leaving, else the world's first; a founder came at its founding, someone sent at
    /// their arrival, a child at birth.
    pub fn infer_residence(&mut self, settlements: &[civ_land::Settlement]) {
        let mut told: BTreeMap<PermanentId, Option<PermanentId>> = BTreeMap::new();
        let mut arrived: BTreeMap<PermanentId, SimTime> = BTreeMap::new();
        for e in &self.chronicle {
            match e.kind {
                ChronicleKind::Died | ChronicleKind::Left => {
                    for &p in &e.people {
                        told.entry(p).or_insert(e.settlement);
                    }
                }
                ChronicleKind::FamilyArrived => {
                    for &p in &e.people {
                        arrived.entry(p).or_insert(e.at);
                    }
                }
                _ => {}
            }
        }
        let first = settlements.first().map(|s| s.id);
        let homes: BTreeMap<PermanentId, Option<PermanentId>> = self
            .people
            .iter()
            .map(|(_, p)| (p.id, self.household(p.household).and_then(|x| x.settlement)))
            .collect();
        for r in self.records.values_mut() {
            if !r.residence.is_empty() {
                continue;
            }
            let settlement = match homes.get(&r.id) {
                Some(&home) => home,
                None => told.get(&r.id).copied().flatten().or(first),
            };
            let founded = settlement
                .and_then(|s| settlements.iter().find(|x| x.id == s))
                .map_or(r.born, |x| x.founded);
            let (since, why) = match r.origin {
                Origin::Born => (r.born, ResidenceWhy::Born),
                Origin::Founder => (founded.max(r.born), ResidenceWhy::Founder),
                Origin::Spawned => (
                    arrived.get(&r.id).copied().unwrap_or(founded).max(r.born),
                    ResidenceWhy::Arrived,
                ),
            };
            r.residence.push(Stay {
                settlement,
                since,
                why,
            });
            if let Some(left) = r.left {
                r.residence.push(Stay {
                    settlement: None,
                    since: left,
                    why: ResidenceWhy::LeftMap,
                });
            }
        }
    }
}
