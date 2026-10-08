//! Takings for the observer (wire 1.30, ADR-0015 §7): what happened, and what people believe of it
//! and chose, in separate tables joined only by an incident's number. The kernel renders the
//! words; the observer only shows them.

use civ_agents::crime::{Incident, Outcome, Source, Standing};
use civ_core::PermanentId;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// Incidents the frame carries, newest first.
pub const MOST_INCIDENTS: usize = 60;

/// "the household of Rilla", for the household `household` as the observer names it now.
fn household_words(sim: &Sim, household: PermanentId) -> String {
    let pop = &sim.people;
    let elder = pop.household(household).and_then(|x| {
        let grown = sim.rules.people.family.independent_age;
        x.members
            .iter()
            .copied()
            .find(|&m| {
                pop.person(m)
                    .is_some_and(|p| p.age_years(sim.now()) >= grown)
            })
            .or_else(|| x.members.first().copied())
    });
    match elder {
        Some(e) => format!("the household of {}", pop.name_of(e)),
        None => "a household that is no more".to_owned(),
    }
}

/// What was carried off, or how the attempt ended, in words.
fn what_words(sim: &Sim, i: &Incident) -> String {
    match i.outcome {
        Outcome::Taken => {
            let goods = &sim.rules.catalog.goods;
            let parts: Vec<String> = i
                .goods
                .iter()
                .filter_map(|&(g, kg)| {
                    goods
                        .get(usize::from(g))
                        .map(|d| format!("{kg:.0} kg of {}", d.name.to_lowercase()))
                })
                .collect();
            if parts.is_empty() {
                "food".to_owned()
            } else {
                parts.join(" and ")
            }
        }
        other => other.words().to_owned(),
    }
}

/// A `Response` with the most recent takings, what is believed of each, and the totals.
pub fn order_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let pop = &sim.people;
    let order = &pop.order;
    let mut incidents = Vec::new();
    let mut known = Vec::new();
    for i in order.incidents.iter().rev().take(MOST_INCIDENTS) {
        let actor_name = fbb.create_string(&pop.name_of(i.actor));
        let target_name = fbb.create_string(&household_words(sim, i.target));
        let what = fbb.create_string(&what_words(sim, i));
        let seen: Vec<_> = i
            .seen_by
            .iter()
            .map(|&w| fbb.create_string(&pop.name_of(w)))
            .collect();
        let seen_by = fbb.create_vector(&seen);
        let settlement = pop
            .household(i.target)
            .and_then(|x| x.settlement)
            .map_or(0, PermanentId::get);
        incidents.push(wire::IncidentLine::create(
            &mut fbb,
            &wire::IncidentLineArgs {
                id: i.id,
                minute: i.at.minutes(),
                settlement,
                actor: i.actor.get(),
                actor_name: Some(actor_name),
                target: i.target.get(),
                target_name: Some(target_name),
                outcome: i.outcome.code(),
                what: Some(what),
                kcal: i.kcal,
                seen_by: Some(seen_by),
            },
        ));
        // What the living believe of it: knowledge, kept in its own table.
        let (mut know_taker, mut know_loss) = (0u32, 0u32);
        let mut victim_knows = false;
        for b in order.beliefs.iter().filter(|b| b.incident == i.id) {
            let Some(p) = pop.person(b.holder) else {
                continue;
            };
            if b.taker.is_some() {
                know_taker += 1;
                victim_knows |= p.household == i.target;
            } else if b.source == Source::Noticed {
                know_loss += 1;
            }
        }
        let response = order
            .responses
            .iter()
            .find(|r| r.incident == i.id)
            .map(|r| {
                if r.demand {
                    format!("{} demanded the food back", pop.name_of(r.by))
                } else {
                    format!("{} let it go", pop.name_of(r.by))
                }
            })
            .unwrap_or_default();
        let owed = order
            .obligations
            .iter()
            .find(|o| o.incident == i.id)
            .map(|o| {
                let given = 100.0 * f64::from(o.paid_kcal) / f64::from(o.kcal).max(1.0);
                match o.standing {
                    Standing::Open => format!("owed: {given:.0}\u{a0}% given back so far"),
                    Standing::Defaulted => {
                        format!("unpaid when due: {given:.0}\u{a0}% given back")
                    }
                    other => other.words().to_owned(),
                }
            })
            .unwrap_or_default();
        let response = fbb.create_string(&response);
        let owed = fbb.create_string(&owed);
        known.push(wire::KnownLine::create(
            &mut fbb,
            &wire::KnownLineArgs {
                incident: i.id,
                know_taker,
                know_loss,
                sources: order.sources(i.id) as u32,
                victim_knows,
                response: Some(response),
                owed: Some(owed),
            },
        ));
    }
    let incidents = fbb.create_vector(&incidents);
    let known = fbb.create_vector(&known);
    let count =
        |f: &dyn Fn(&Incident) -> bool| order.incidents.iter().filter(|i| f(i)).count() as u32;
    let known_to_victims = order
        .responses
        .iter()
        .map(|r| r.incident)
        .collect::<std::collections::BTreeSet<_>>()
        .len() as u32;
    let standing =
        |s: Standing| order.obligations.iter().filter(|o| o.standing == s).count() as u32;
    let body = wire::Order::create(
        &mut fbb,
        &wire::OrderArgs {
            minute: sim.now().minutes(),
            incidents: Some(incidents),
            known: Some(known),
            attempts: order.incidents.len() as u32,
            takings: count(&|i| i.outcome == Outcome::Taken),
            seen: count(&|i| !i.seen_by.is_empty()),
            known_to_victims,
            demands: order.responses.iter().filter(|r| r.demand).count() as u32,
            met: standing(Standing::Met),
            refused: standing(Standing::Refused),
            refusals: order.refusals,
        },
    );
    response(fbb, wire::ResponseBody::Order, body)
}
