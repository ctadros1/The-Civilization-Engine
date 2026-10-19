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
pub(crate) fn household_words(sim: &Sim, household: PermanentId) -> String {
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
pub(crate) fn what_words(sim: &Sim, i: &Incident) -> String {
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
pub(crate) fn response_words(sim: &Sim, incident: u32) -> String {
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
pub(crate) fn owed_words(sim: &Sim, incident: u32) -> String {
    let owed: Vec<&Obligation> = sim
        .people
        .order
        .obligations
        .iter()
        // Ground rent answers no incident (M8c slice CU).
        .filter(|o| o.incident == incident && o.kind != Owed::GroundRent)
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
pub(crate) fn watch_words(sim: &Sim, i: &Incident) -> String {
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
pub(crate) fn case_words(sim: &Sim, incident: u32) -> String {
    sim.people
        .order
        .cases
        .iter()
        .rev()
        .find(|c| c.incident == incident && c.challenge.is_none())
        .map_or_else(String::new, |c| case_text(sim, c))
}

/// A challenge of a law under a limit, in words (M7b slice BU): who challenged what before whom,
/// and how it closed.
pub(crate) fn challenge_text(sim: &Sim, c: &civ_agents::crime::Case) -> String {
    let pop = &sim.people;
    let policies = &sim.rules.catalog.policies;
    let laws = || pop.polities.iter().flat_map(|p| &p.laws);
    let words_of = |id: civ_core::PermanentId| {
        laws().find(|l| l.id == id).map_or_else(
            || "a law".to_owned(),
            |l| civ_agents::polity::law_words(l, policies, &|p| pop.name_of(p)),
        )
    };
    let limit = c
        .challenge
        .and_then(|lid| laws().find(|l| l.id == lid))
        .and_then(|l| policies.get(usize::from(l.policy))?.limit)
        .map_or_else(|| "a limit".to_owned(), |l| l.words());
    let forum = c.forum.and_then(|f| {
        let law = laws().find(|l| l.id == f)?;
        let def = policies.get(usize::from(law.policy))?.forum.as_ref()?;
        let holder = law.holder.map(|h| pop.name_of(h));
        let w = civ_agents::forum::hearers_words(def, holder.as_deref());
        let mut ch = w.chars();
        ch.next()
            .map(|f| f.to_lowercase().collect::<String>() + ch.as_str())
    });
    let before = forum.unwrap_or_else(|| "a forum".to_owned());
    let (_, support, oppose) = c.counts();
    let closed = c
        .closure_days()
        .filter(|_| c.stage != CaseStage::Open)
        .map_or_else(String::new, |d| {
            format!("; closed {d} day{} after", if d == 1 { "" } else { "s" })
        });
    let how = match c.stage {
        CaseStage::Open => "it waits to be heard".to_owned(),
        CaseStage::Found => format!(
            "{before} found that it breaks the limit, {support} for and {oppose} against: it \
             binds no more"
        ),
        CaseStage::NotFound => format!(
            "{before} did not find that it breaks the limit, {support} for and {oppose} \
             against: it stands"
        ),
        _ => "it was never heard".to_owned(),
    };
    format!(
        "{} challenged {} before {before}, under the limit that {limit}; {how}{closed}",
        pop.name_of(c.by),
        words_of(c.law)
    )
}

/// A case of a taking, in words (what the polity knows).
pub(crate) fn case_text(sim: &Sim, c: &civ_agents::crime::Case) -> String {
    let pop = &sim.people;
    let witnesses = match c.leads.len() {
        1 => "1 witness".to_owned(),
        n => format!("{n} witnesses"),
    };
    // Who was to hear it: the gathering, or a forum (M7b slice BR).
    let forum = c.forum.and_then(|f| {
        let law = pop
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .find(|l| l.id == f)?;
        let def = sim
            .rules
            .catalog
            .policies
            .get(usize::from(law.policy))?
            .forum
            .as_ref()?;
        let holder = law.holder.map(|h| pop.name_of(h));
        Some((
            def,
            civ_agents::forum::hearers_words(def, holder.as_deref()),
        ))
    });
    let hearers = forum.as_ref().map_or_else(
        || "the gathering".to_owned(),
        |(_, w)| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_lowercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        },
    );
    let brought = format!(
        "{} brought it before {hearers} on the word of {witnesses}",
        pop.name_of(c.by)
    );
    let (present, support, oppose) = c.counts();
    let tally = match &forum {
        Some((def, _)) if def.seat == civ_agents::forum::ForumSeat::Holder => {
            "heard alone".to_owned()
        }
        Some(_) => format!("{support} for, {oppose} against, of {} who sat", c.eligible),
        None => format!(
            "{support} for, {oppose} against; {present} of {} adults came",
            c.eligible
        ),
    };
    let accused = pop.name_of(c.accused);
    let decided = match c.stage {
        CaseStage::Open => "it waits to be heard".to_owned(),
        CaseStage::Found if c.exiled => {
            format!("{hearers} found that {accused} took: {tally}; they were sent from the valley")
        }
        CaseStage::Found => format!("{hearers} found that {accused} took: {tally}"),
        CaseStage::NotFound => {
            format!("{hearers} did not find that {accused} took: {tally}")
        }
        CaseStage::Unheard if forum.is_some() => "nobody sat to hear it".to_owned(),
        CaseStage::Unheard => format!("too few came to hear it: {present} of {}", c.eligible),
        CaseStage::Lapsed => "it was never heard: a party was no longer there".to_owned(),
        CaseStage::Settled => format!(
            "{accused}'s household offered back what was taken before it was heard, and it was \
             accepted"
        ),
        CaseStage::Withdrawn => {
            "it was withdrawn before it was heard: what was taken had been given back".to_owned()
        }
    };
    // A finding against a remembered one of the same facts (M7b slice BS).
    let decided = match c.departed {
        Some(n) => format!("{decided}; it departed from case {n}, of the same facts"),
        None => decided,
    };
    // Time to closure (M7b slice BR).
    let decided = match c.closure_days() {
        Some(d) if c.stage != CaseStage::Open => format!(
            "{decided}; closed {d} day{} after it was brought",
            if d == 1 { "" } else { "s" }
        ),
        _ => decided,
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
    // Encounters (wire 1.46), newest first, as the chronicle told them.
    let encounters: Vec<_> = pop
        .chronicle
        .iter()
        .rev()
        .filter(|e| e.kind == civ_agents::ChronicleKind::Encounter)
        .map(|e| fbb.create_string(&e.name))
        .collect();
    let encounters = fbb.create_vector(&encounters);
    // The latest cases and challenges in words (wire 1.79).
    let case_lines: Vec<_> = order
        .cases
        .iter()
        .rev()
        .take(20)
        .map(|c| {
            let text = if c.challenge.is_some() {
                challenge_text(sim, c)
            } else {
                case_text(sim, c)
            };
            let mut ch = text.chars();
            let text = ch
                .next()
                .map(|f| f.to_uppercase().collect::<String>() + ch.as_str())
                .unwrap_or_default();
            fbb.create_string(&format!("Case {}: {text}.", c.id))
        })
        .collect();
    let case_lines = fbb.create_vector(&case_lines);
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
            encounters: Some(encounters),
            settled: stage(CaseStage::Settled),
            withdrawn: stage(CaseStage::Withdrawn),
            challenges: order.cases.iter().filter(|c| c.challenge.is_some()).count() as u32,
            struck_down: order
                .cases
                .iter()
                .filter(|c| c.challenge.is_some() && c.stage == CaseStage::Found)
                .count() as u32,
            case_lines: Some(case_lines),
        },
    );
    response(fbb, wire::ResponseBody::Order, body)
}
