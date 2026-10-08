//! Grievances and word of mouth for the observer (wire 1.34, ADR-0016 §2–§3): what one person
//! holds against whom and why, and what they have heard that is still news, with who told them.
//! Per person, in the inspector, never in the frame (ADR-0016 §7). The kernel renders the words;
//! the observer only shows them.

use civ_agents::Person;
use civ_agents::polity::{day_words, law_words};
use civ_agents::word::{Blamed, Claim, ClaimKind};
use civ_core::{PermanentId, SimTime};
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use crate::Sim;

/// The most claims heard the inspector is sent for one person.
pub const MAX_HEARD_SHOWN: usize = 12;

const DAY: i64 = 24 * 60;

/// The party a grievance blames, in words: "the gathering", "Ada as keeper of the common store",
/// "Bo's household", "Bo".
pub fn blamed_words(sim: &Sim, blamed: Blamed) -> String {
    let pop = &sim.people;
    let name_of = |id: PermanentId| pop.name_of(id);
    match blamed {
        Blamed::Body(_) => "the gathering".to_owned(),
        Blamed::Office(law) => pop
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .find(|l| l.id == law)
            .map_or_else(
                || "an office".to_owned(),
                |l| law_words(l, &sim.rules.catalog.policies, &name_of),
            ),
        Blamed::Household(h) => super::people::household_name(sim, h),
        Blamed::Person(p) => pop.name_of(p),
    }
}

/// What a claim says, in words: "a gathering meets on 3 May of year 2 to decide on a common
/// store, taking a tenth of each harvest"; "Bo holds a grievance against the gathering over food
/// when their household was short".
pub fn claim_words(sim: &Sim, c: &Claim) -> String {
    let pop = &sim.people;
    let name_of = |id: PermanentId| pop.name_of(id);
    match c.kind {
        ClaimKind::Gathering => {
            let day = day_words(SimTime::from_minutes(c.day * DAY));
            let law = c.subject.and_then(|id| {
                pop.polities
                    .iter()
                    .flat_map(|p| &p.laws)
                    .find(|l| l.id == id)
            });
            match law {
                Some(l) => format!(
                    "a gathering meets on {day} to decide on {}",
                    law_words(l, &sim.rules.catalog.policies, &name_of)
                ),
                None => format!("a gathering meets on {day} to hear a case"),
            }
        }
        ClaimKind::Grievance => {
            let who = c.subject.map_or_else(|| "someone".to_owned(), name_of);
            match c.grievance {
                Some((blamed, issue)) => format!(
                    "{who} holds a grievance against {} over {}",
                    blamed_words(sim, blamed),
                    issue.words()
                ),
                None => format!("{who} holds a grievance"),
            }
        }
    }
}

/// The grievances `p` holds, the most keenly felt now first.
pub fn grievance_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::GrievanceLine<'a>>>> {
    let (word, wp) = (&sim.people.word, &sim.rules.people.word);
    let day = sim.now().day_index();
    let mut held: Vec<_> = word
        .grievances_of(p.id)
        .map(|g| (g.activation_on(day, wp.half_life(g.issue)), g))
        .collect();
    held.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.made.cmp(&b.1.made)));
    let lines: Vec<_> = held
        .iter()
        .map(|&(now, g)| {
            let over = fbb.create_string(g.issue.words());
            let blamed = fbb.create_string(&blamed_words(sim, g.blamed));
            let reason = fbb.create_string(g.wrong.words());
            wire::GrievanceLine::create(
                fbb,
                &wire::GrievanceLineArgs {
                    issue: g.issue.code(),
                    over: Some(over),
                    blamed: Some(blamed),
                    law: g.law.get(),
                    harm_days: g.harm_days,
                    unresolved_days: g.unresolved_days,
                    activation: now as f32,
                    made_minute: g.made * DAY,
                    raised_minute: g.raised * DAY,
                    reason: Some(reason),
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// Where `p` stands on each question content names (wire 1.36, ADR-0016 §4).
pub fn position_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::PositionLine<'a>>>> {
    let policies = &sim.rules.catalog.policies;
    let lines: Vec<_> = sim
        .people
        .opinion
        .held_by(p.id)
        .iter()
        .filter_map(|pos| {
            let q = policies.get(usize::from(pos.policy))?.question.as_deref()?;
            Some((pos, q))
        })
        .map(|(pos, q)| {
            let question = fbb.create_string(q);
            let lean = fbb.create_string(civ_agents::opinion::lean_words(f64::from(pos.x)));
            wire::PositionLine::create(
                fbb,
                &wire::PositionLineArgs {
                    question: Some(question),
                    lean: Some(lean),
                    x: pos.x,
                    anchor: pos.anchor,
                    salience: pos.salience,
                    heard: pos.heard,
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// What `p` has heard that is still news, the latest first, at most [`MAX_HEARD_SHOWN`].
pub fn heard_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::HeardLine<'a>>>> {
    let word = &sim.people.word;
    let mut heard: Vec<_> = word.heard_by(p.id).iter().collect();
    heard.sort_by(|a, b| b.last.cmp(&a.last).then(b.claim.cmp(&a.claim)));
    let lines: Vec<_> = heard
        .iter()
        .take(MAX_HEARD_SHOWN)
        .filter_map(|h| word.claim(h.claim).map(|c| (h, c)))
        .map(|(h, c)| {
            let what = fbb.create_string(&claim_words(sim, c));
            let from_name = h.from.map(|f| fbb.create_string(&sim.people.name_of(f)));
            wire::HeardLine::create(
                fbb,
                &wire::HeardLineArgs {
                    kind: c.kind.code(),
                    what: Some(what),
                    from: h.from.map_or(0, PermanentId::get),
                    from_name,
                    origin: h.origin.map_or(0, PermanentId::get),
                    first_minute: h.first * DAY,
                    last_minute: h.last * DAY,
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}
