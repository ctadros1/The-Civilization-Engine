//! The dashboard's neighbours row (M5b slice AR; the M5 diffusion brief §3.1, test 1, and its
//! open question 9, answered in plan §9's M5b design): nothing crosses between settlements without
//! contact, and with contact something crosses within the run. It is graded by direction, never
//! by rate: a way of building or a technique reaching one settlement from another with no contact
//! on record that could carry it is the hearth leak the brief warns of, and red.
//!
//! What crosses from A to B: a standing building of B whose chain of followed buildings reaches
//! one of A; a household of B that admires a building of A most; a technique B's record says was
//! brought from A; and someone of B who knows of a technique by seeing it at A. What could carry
//! it: people of B who visited A or went to buy there, and people of A who came to live in B, on
//! marrying or with their household.

use std::collections::BTreeMap;

use civ_agents::knowledge::KnowledgeEventKind;
use civ_agents::person::KnowSource;
use civ_core::PermanentId;
use civ_sim::Sim;
use serde::Serialize;

use crate::economy::Grade;

/// What crossed one way between two settlements over a run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Crossed {
    /// Standing buildings whose chain of followed buildings reaches the other settlement.
    pub buildings: u32,
    /// Households that admire a building of the other settlement most.
    pub admired: u32,
    /// Techniques the record says were brought from the other settlement.
    pub brought: u32,
    /// People who know of a technique by seeing it in the other settlement.
    pub seen: u32,
}

impl Crossed {
    /// Everything that crossed.
    pub fn total(&self) -> u32 {
        self.buildings + self.admired + self.brought + self.seen
    }
}

/// One direction between two settlements: what could carry a way across, and what crossed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Way {
    /// Visits, trips to buy, moves and marriages that could carry it.
    pub contact: u32,
    /// What crossed.
    pub crossed: Crossed,
}

/// A pair of settlements (`a` before `b` by id), each way: `to_b` is what reached `b` from `a`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PairSeen {
    #[serde(serialize_with = "raw_id")]
    pub a: PermanentId,
    #[serde(serialize_with = "raw_id")]
    pub b: PermanentId,
    pub to_b: Way,
    pub to_a: Way,
}

fn raw_id<S: serde::Serializer>(id: &PermanentId, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u64(id.get())
}

impl PairSeen {
    /// The pair's grade and why: red for anything that crossed a way no contact could carry it;
    /// grey with no contact either way; amber with contact and nothing crossed; else green.
    pub fn grade(&self) -> (Grade, String) {
        let leaked: Vec<String> = [("to the second", &self.to_b), ("to the first", &self.to_a)]
            .iter()
            .filter(|(_, w)| w.contact == 0 && w.crossed.total() > 0)
            .map(|(dir, w)| format!("{} crossed {dir} with no contact", w.crossed.total()))
            .collect();
        if !leaked.is_empty() {
            return (Grade::Red, leaked.join("; "));
        }
        let contact = self.to_b.contact + self.to_a.contact;
        let crossed = self.to_b.crossed.total() + self.to_a.crossed.total();
        if contact == 0 {
            (Grade::Gray, "no contact".to_owned())
        } else if crossed == 0 {
            (Grade::Amber, "contact, but nothing crossed".to_owned())
        } else {
            (Grade::Green, "crossed only with contact".to_owned())
        }
    }
}

/// The way from `from` to `to` in `out`, the pair begun if new.
fn way_of(
    out: &mut BTreeMap<(PermanentId, PermanentId), PairSeen>,
    from: PermanentId,
    to: PermanentId,
) -> &mut Way {
    let (a, b) = if from < to { (from, to) } else { (to, from) };
    let p = out.entry((a, b)).or_insert(PairSeen {
        a,
        b,
        to_b: Way::default(),
        to_a: Way::default(),
    });
    if to == b { &mut p.to_b } else { &mut p.to_a }
}

/// Every pair of settlements a world has had, with what crossed each way and the contact that
/// could carry it, over its whole record.
pub fn pairs(sim: &Sim) -> Vec<PairSeen> {
    let pop = sim.people();
    let land = sim.land();
    let mut out: BTreeMap<(PermanentId, PermanentId), PairSeen> = BTreeMap::new();
    for (&(_, from, to), c) in &pop.contacts.years {
        // People of `from` who went to `to` carry what they saw there home to `from`.
        way_of(&mut out, to, from).contact += c.visits + c.bought + c.missed.iter().sum::<u32>();
        // People of `from` who came to live in `to` carry their ways there.
        way_of(&mut out, from, to).contact += c.marriages + c.moved;
    }
    let settlement_of = |p: PermanentId| {
        pop.person(p)
            .and_then(|q| pop.household(q.household))
            .and_then(|h| h.settlement)
    };
    for b in &land.buildings {
        if !(b.finished() && b.state == civ_land::BuildingState::Standing) {
            continue;
        }
        let (Some(here), Some(first)) =
            (pop.building_settlement(land, b), pop.crossing_of(land, b))
        else {
            continue;
        };
        let Some(there) = land
            .buildings
            .iter()
            .find(|x| x.id == first)
            .and_then(|x| pop.building_settlement(land, x))
        else {
            continue;
        };
        way_of(&mut out, there, here).crossed.buildings += 1;
    }
    for (_, h) in pop.households.iter() {
        let (Some(here), Some(admired)) = (h.settlement, h.admired) else {
            continue;
        };
        if h.members.is_empty() {
            continue;
        }
        let there = land
            .buildings
            .iter()
            .find(|x| x.id == admired)
            .and_then(|x| pop.building_settlement(land, x));
        if let Some(there) = there.filter(|&t| t != here) {
            way_of(&mut out, there, here).crossed.admired += 1;
        }
    }
    for e in &pop.knowledge {
        if let (KnowledgeEventKind::Known(_), Some(from)) = (e.kind, e.elsewhere) {
            way_of(&mut out, from, e.settlement).crossed.brought += 1;
        }
    }
    for (_, p) in pop.people.iter() {
        let Some(here) = settlement_of(p.id) else {
            continue;
        };
        for k in &p.knows {
            if let KnowSource::Seen(there) = k.source
                && there != here
                && !k.known
            {
                way_of(&mut out, there, here).crossed.seen += 1;
            }
        }
    }
    out.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("an id")
    }

    #[test]
    fn a_pair_is_red_for_a_crossing_without_contact_grey_without_contact_and_amber_without_a_crossing()
     {
        let crossed = Crossed {
            buildings: 1,
            ..Crossed::default()
        };
        let mut p = PairSeen {
            a: id(1),
            b: id(2),
            to_b: Way::default(),
            to_a: Way::default(),
        };
        assert_eq!(p.grade().0, Grade::Gray);
        p.to_a.contact = 3;
        assert_eq!(p.grade().0, Grade::Amber);
        p.to_a.crossed = crossed;
        assert_eq!(p.grade().0, Grade::Green);
        // Something reached the second with no contact that way: a leak, whatever else passed.
        p.to_b.crossed = crossed;
        let (g, why) = p.grade();
        assert_eq!(g, Grade::Red);
        assert!(why.contains("to the second with no contact"), "{why}");
        p.to_b.contact = 1;
        assert_eq!(p.grade().0, Grade::Green);
    }
}
