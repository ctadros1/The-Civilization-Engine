//! Each settlement's polity for the observer (wire 1.27, ADR-0013): its custom, its members, its
//! store, every law proposed there with its whole history, its offices (1.28) and its label
//! (1.29). The kernel renders the words; the observer only shows them.

use civ_agents::polity::{
    Law, LawStatus, Outcome, Polity, Stance, day_words, decision_words, law_words, office_words,
    stance_words,
};
use civ_core::SimTime;
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use super::response;
use crate::Sim;

fn status_code(s: LawStatus) -> u8 {
    match s {
        LawStatus::Proposed => 0,
        LawStatus::InForce => 1,
        LawStatus::Rejected => 2,
        LawStatus::Lapsed => 3,
    }
}

fn outcome_code(o: Option<Outcome>) -> u8 {
    o.map_or(u8::MAX, |o| o as u8)
}

fn stance_code(s: Stance) -> u8 {
    match s {
        Stance::Support => 0,
        Stance::Oppose => 1,
        Stance::Abstain => 2,
    }
}

/// One law with its history.
fn law_line<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    polity: &Polity,
    law: &Law,
) -> WIPOffset<wire::LawLine<'a>> {
    let (pop, rules) = (&sim.people, &sim.rules);
    let margin = rules.people.polity.stance_margin;
    let stances: Vec<_> = law
        .stances
        .iter()
        .map(|r| {
            let name = fbb.create_string(&pop.name_of(r.person));
            let why = fbb.create_string(&stance_words(r, margin, law.sponsor));
            wire::StanceLine::create(
                fbb,
                &wire::StanceLineArgs {
                    person: r.person.get(),
                    name: Some(name),
                    stance: stance_code(r.stance),
                    gain: r.gain,
                    regard: r.regard,
                    why: Some(why),
                },
            )
        })
        .collect();
    let stances = fbb.create_vector(&stances);
    let what = fbb.create_string(&law_words(law, &rules.catalog.policies, &|id| {
        pop.name_of(id)
    }));
    let holder_name = law.holder.map(|h| fbb.create_string(&pop.name_of(h)));
    let policy = fbb.create_string(
        rules
            .catalog
            .policies
            .get(usize::from(law.policy))
            .map_or("", |d| d.id.as_str()),
    );
    let sponsor_name = fbb.create_string(&pop.name_of(law.sponsor));
    let issue = fbb.create_string(law.issue.words());
    let decision = decision_words(law, &polity.body).map(|w| fbb.create_string(&w));
    let known = law
        .known
        .iter()
        .filter(|(p, _)| pop.person(*p).is_some())
        .count() as u32;
    let c = &law.compliance;
    wire::LawLine::create(
        fbb,
        &wire::LawLineArgs {
            id: law.id.get(),
            what: Some(what),
            policy: Some(policy),
            levy_share: law.levy_share,
            relief_days: law.relief_days,
            status: status_code(law.status),
            sponsor: law.sponsor.get(),
            sponsor_name: Some(sponsor_name),
            proposed_minute: law.proposed.minutes(),
            issue: Some(issue),
            meets_minute: law.meets_day * 1440,
            decided_minute: law.decided.map_or(0, SimTime::minutes),
            outcome: outcome_code(law.outcome),
            decision,
            eligible: law.eligible,
            quorum: polity.body.quorum(law.eligible),
            stances: Some(stances),
            known,
            complied: c.complied,
            could_not: c.could_not,
            evaded: c.evaded,
            unaware: c.unaware,
            levied_kg: c.levied_kg as f32,
            withheld_kg: c.withheld_kg as f32,
            relieved: c.relieved,
            relief_kg: c.relief_kg as f32,
            unanswered: c.unanswered,
            holder: law.holder.map_or(0, |h| h.get()),
            holder_name,
        },
    )
}

/// How someone came to be gone, in words: "when they died on 9 June of year 5".
fn gone_words(sim: &Sim, id: civ_core::PermanentId) -> String {
    match sim.people.records.get(&id) {
        Some(r) => match (r.died, r.left) {
            (Some((t, _)), _) => format!("died on {}", day_words(t)),
            (_, Some(t)) => format!("left the valley on {}", day_words(t)),
            _ => "moved away".to_owned(),
        },
        None => "is gone".to_owned(),
    }
}

/// What a store holds, in words: "grain 322 kg", or "nothing".
fn store_words(sim: &Sim, polity: &Polity) -> String {
    let goods = &sim.rules.catalog.goods;
    let held: Vec<String> = goods
        .iter()
        .zip(&polity.stores)
        .filter(|(_, kg)| **kg >= 0.5)
        .map(|(g, kg)| format!("{} {kg:.0} kg", g.name.to_lowercase()))
        .collect();
    if held.is_empty() {
        "nothing".to_owned()
    } else {
        held.join(", ")
    }
}

/// A `Response` with every settlement's polity.
pub fn government_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let (pop, rules) = (&sim.people, &sim.rules);
    let now = sim.now();
    let goods = &rules.catalog.goods;
    let adult = rules.people.family.independent_age;
    let mut list = Vec::new();
    for polity in &pop.polities {
        let settlement = sim
            .land
            .settlements
            .iter()
            .find(|s| s.id == polity.settlement);
        let name = fbb.create_string(settlement.map_or("", |s| s.name.as_str()));
        let custom = fbb.create_string(&polity.body.words());
        let store = fbb.create_string(&store_words(sim, polity));
        let store_kg: f64 = goods
            .iter()
            .zip(&polity.stores)
            .filter(|(g, _)| g.purpose == civ_agents::params::GoodUse::Food)
            .map(|(_, kg)| kg.max(0.0))
            .sum();
        let members = pop
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= adult)
            .filter(|(_, p)| {
                pop.household(p.household)
                    .is_some_and(|x| x.settlement == Some(polity.settlement))
            })
            .count() as u32;
        let laws: Vec<_> = polity
            .laws
            .iter()
            .rev()
            .map(|l| law_line(&mut fbb, sim, polity, l))
            .collect();
        let laws = fbb.create_vector(&laws);
        let offices: Vec<_> = office_words(
            polity,
            &rules.catalog.policies,
            &|id| pop.name_of(id),
            &|id| gone_words(sim, id),
        )
        .iter()
        .map(|w| fbb.create_string(w))
        .collect();
        let offices = fbb.create_vector(&offices);
        let label = crate::labels::label_of(sim, polity);
        let label_name = fbb.create_string(&label.name);
        let modifiers: Vec<_> = label
            .modifiers
            .iter()
            .map(|m| fbb.create_string(m))
            .collect();
        let label_modifiers = fbb.create_vector(&modifiers);
        let why: Vec<_> = label.why.iter().map(|w| fbb.create_string(w)).collect();
        let label_why = fbb.create_vector(&why);
        let (gathering_law, gathering_minute, gathering_present) =
            polity.gathering.as_ref().map_or((0, 0, 0), |g| {
                (
                    g.law.map_or(0, civ_core::PermanentId::get),
                    g.day * 1440,
                    g.present.len() as u32,
                )
            });
        // The cases it is to hear (wire 1.31).
        let pop = &sim.people;
        let cases: Vec<String> = polity
            .gathering
            .as_ref()
            .map(|g| {
                g.cases
                    .iter()
                    .filter_map(|&c| pop.order.case(c))
                    .map(|c| {
                        format!(
                            "{}'s case against {}",
                            pop.name_of(c.by),
                            pop.name_of(c.accused)
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let cases: Vec<_> = cases.iter().map(|c| fbb.create_string(c)).collect();
        let gathering_cases = fbb.create_vector(&cases);
        list.push(wire::PolityLine::create(
            &mut fbb,
            &wire::PolityLineArgs {
                polity: polity.id.get(),
                settlement: polity.settlement.get(),
                name: Some(name),
                founded_minute: polity.founded.minutes(),
                custom: Some(custom),
                members,
                store: Some(store),
                store_kg: store_kg as f32,
                laws: Some(laws),
                gathering_law,
                gathering_minute,
                gathering_present,
                gathering_cases: Some(gathering_cases),
                offices: Some(offices),
                label: Some(label_name),
                label_modifiers: Some(label_modifiers),
                label_why: Some(label_why),
                label_confidence: label.confidence as f32,
            },
        ));
    }
    let polities = fbb.create_vector(&list);
    let body = wire::Government::create(
        &mut fbb,
        &wire::GovernmentArgs {
            minute: now.minutes(),
            polities: Some(polities),
        },
    );
    response(fbb, wire::ResponseBody::Government, body)
}
