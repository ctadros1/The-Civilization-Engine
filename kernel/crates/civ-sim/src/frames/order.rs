//! Takings for the observer (wire 1.30, ADR-0015 §7): what happened, and what people believe of it
//! and chose, in separate tables joined only by an incident's number; from wire 1.31 also the case
//! brought for it, which is what the polity knows. The kernel renders the words; the observer
//! only shows them.

use civ_agents::crime::{
    CaseStage, Choice, Incident, Kept, Obligation, Outcome, Owed, Source, Standing,
};
use civ_core::PermanentId;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// Incidents the frame carries, newest first: every taking and every attempt someone saw before
/// any other, then the newest attempts that turned back unseen, which are the commonest and tell
/// least (a lean spell can bring dozens).
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
                    goods.get(usize::from(g)).map(|d| {
                        let name = d.name.to_lowercase();
                        // Not "0 kg of wild plant food" for a handful.
                        if kg < 0.5 {
                            format!("a little {name}")
                        } else {
                            format!("{kg:.0} kg of {name}")
                        }
                    })
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

/// Where obligation `o` stands, in words: "paid", "owed: 40 % given so far".
fn standing_words(o: &Obligation) -> String {
    let given = 100.0 * f64::from(o.paid_kcal) / f64::from(o.kcal).max(1.0);
    match o.standing {
        Standing::Open => format!("owed: {given:.0}\u{a0}% given so far"),
        Standing::Defaulted => format!("unpaid when due: {given:.0}\u{a0}% given"),
        other => other.words().to_owned(),
    }
}

/// What the household taken from chose, in order: "Rilla demanded the food back; then brought it
/// before the gathering".
fn response_words(sim: &Sim, incident: u32) -> String {
    let pop = &sim.people;
    let mut parts = Vec::new();
    for r in pop
        .order
        .responses
        .iter()
        .filter(|r| r.incident == incident)
    {
        let what = match r.choice {
            Choice::LetGo => "let it go",
            Choice::Demand => "demanded the food back",
            Choice::Report => "brought it before the gathering",
        };
        if parts.is_empty() {
            parts.push(format!("{} {what}", pop.name_of(r.by)));
        } else {
            parts.push(format!("then {what}"));
        }
    }
    parts.join("; ")
}

/// Where what is owed for an incident stands: "paid" for a demand alone, or each kind in turn
/// ("restitution: paid; compensation: owed: 40 % given so far; a fine: refused").
fn owed_words(sim: &Sim, incident: u32) -> String {
    let owed: Vec<&Obligation> = sim
        .people
        .order
        .obligations
        .iter()
        .filter(|o| o.incident == incident)
        .collect();
    match owed.as_slice() {
        [] => String::new(),
        [o] if o.kind == Owed::Demanded => standing_words(o),
        many => many
            .iter()
            .map(|o| format!("{}: {}", o.kind.words(), standing_words(o)))
            .collect::<Vec<_>>()
            .join("; "),
    }
}

/// What the one who keeps the watch did with a taking they saw, in words: the truth (M4b slice
/// AC). "Bram, keeping watch, saw it and said nothing for 9 kg of grain."
fn watch_words(sim: &Sim, i: &Incident) -> String {
    let pop = &sim.people;
    let Some(s) = pop.order.sightings.iter().find(|s| s.incident == i.id) else {
        return String::new();
    };
    let did = match s.kept {
        Kept::Reported => "brought it before the gathering".to_owned(),
        Kept::Told => "told the household taken from".to_owned(),
        Kept::LookedAway => "said nothing".to_owned(),
        Kept::Paid => {
            // In days of the watcher's household's food, the scale every sanction is weighed in.
            let need = pop
                .person(s.officer)
                .and_then(|p| pop.household(p.household))
                .map_or(1, |x| x.members.len().max(1)) as f64
                * sim.rules.people.household.daily_kcal_per_person;
            format!(
                "said nothing, for {:.1} days of their household's food from the taker's",
                f64::from(s.kcal) / need.max(1.0)
            )
        }
        Kept::Refused => {
            "asked the taker's household to pay to say nothing, was refused, and brought it before \
             the gathering"
                .to_owned()
        }
    };
    format!(
        "{}, keeping watch, saw it and {did}",
        pop.name_of(s.officer)
    )
}

/// The case brought for an incident and how the gathering decided, in words: what the polity
/// knows.
fn case_words(sim: &Sim, incident: u32) -> String {
    let pop = &sim.people;
    let Some(c) = pop
        .order
        .cases
        .iter()
        .rev()
        .find(|c| c.incident == incident)
    else {
        return String::new();
    };
    let witnesses = match c.leads.len() {
        1 => "1 witness".to_owned(),
        n => format!("{n} witnesses"),
    };
    let brought = format!(
        "{} brought it before the gathering on the word of {witnesses}",
        pop.name_of(c.by)
    );
    let (present, support, oppose) = c.counts();
    let tally = format!(
        "{support} for, {oppose} against; {present} of {} adults came",
        c.eligible
    );
    let accused = pop.name_of(c.accused);
    let decided = match c.stage {
        CaseStage::Open => "it waits to be heard".to_owned(),
        CaseStage::Found if c.exiled => format!(
            "the gathering found that {accused} took: {tally}; they were sent from the valley"
        ),
        CaseStage::Found => format!("the gathering found that {accused} took: {tally}"),
        CaseStage::NotFound => {
            format!("the gathering did not find that {accused} took: {tally}")
        }
        CaseStage::Unheard => format!("too few came to hear it: {present} of {}", c.eligible),
        CaseStage::Lapsed => "it was never heard: a party was no longer there".to_owned(),
    };
    format!("{brought}; {decided}")
}

/// A `Response` with the most recent takings, what is believed of each, and the totals.
pub fn order_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let pop = &sim.people;
    let order = &pop.order;
    let mut incidents = Vec::new();
    let mut known = Vec::new();
    let telling = |i: &&civ_agents::crime::Incident| {
        i.outcome == civ_agents::crime::Outcome::Taken || !i.seen_by.is_empty()
    };
    let mut shown: Vec<&civ_agents::crime::Incident> = order
        .incidents
        .iter()
        .rev()
        .filter(telling)
        .take(MOST_INCIDENTS)
        .collect();
    let room = MOST_INCIDENTS - shown.len();
    shown.extend(
        order
            .incidents
            .iter()
            .rev()
            .filter(|i| !telling(i))
            .take(room),
    );
    shown.sort_unstable_by_key(|i| std::cmp::Reverse(i.id));
    for i in shown {
        let actor_name = fbb.create_string(&pop.name_of(i.actor));
        let target_name = fbb.create_string(&household_words(sim, i.target));
        let what = fbb.create_string(&what_words(sim, i));
        let seen: Vec<_> = i
            .seen_by
            .iter()
            .map(|&w| fbb.create_string(&pop.name_of(w)))
            .collect();
        let seen_by = fbb.create_vector(&seen);
        let watch = fbb.create_string(&watch_words(sim, i));
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
                watch: Some(watch),
            },
        ));
        // What the living believe of it: knowledge, kept in its own table. What the taker knows of
        // their own doing, and told their household, is no account of it (M4b slice AB).
        let (mut know_taker, mut know_loss) = (0u32, 0u32);
        let mut victim_knows = false;
        for b in order.beliefs.iter().filter(|b| b.incident == i.id) {
            let Some(p) = pop.person(b.holder) else {
                continue;
            };
            if b.taker.is_some() && b.origin == Some(i.actor) {
                continue;
            }
            if b.taker.is_some() {
                know_taker += 1;
                victim_knows |= p.household == i.target;
            } else if b.source == Source::Noticed {
                know_loss += 1;
            }
        }
        let response = response_words(sim, i.id);
        let owed = owed_words(sim, i.id);
        let case = fbb.create_string(&case_words(sim, i.id));
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
                case: Some(case),
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
    // Demands to give back, as AA counted them; what findings impose is told by case.
    let standing = |s: Standing| {
        order
            .obligations
            .iter()
            .filter(|o| o.kind == Owed::Demanded && o.standing == s)
            .count() as u32
    };
    let stage = |s: CaseStage| order.cases.iter().filter(|c| c.stage == s).count() as u32;
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
            demands: order
                .responses
                .iter()
                .filter(|r| r.choice == Choice::Demand)
                .count() as u32,
            met: standing(Standing::Met),
            refused: standing(Standing::Refused),
            refusals: order.refusals,
            cases: order.cases.len() as u32,
            found: stage(CaseStage::Found),
            not_found: stage(CaseStage::NotFound),
            unheard: stage(CaseStage::Unheard),
        },
    );
    response(fbb, wire::ResponseBody::Order, body)
}
