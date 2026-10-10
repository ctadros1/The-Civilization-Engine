//! Each settlement's polity for the observer (wire 1.27, ADR-0013): its custom, its members, its
//! store, every law proposed there with its whole history, its offices (1.28) and its label
//! (1.29). The kernel renders the words; the observer only shows them.

use civ_agents::polity::{
    Law, LawStatus, Outcome, Polity, Stance, day_words, decision_words, office_words, stance_words,
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
        LawStatus::Superseded => 4,
        LawStatus::Carried => 5,
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
                    opinion: r.opinion,
                    values: r.values,
                },
            )
        })
        .collect();
    let stances = fbb.create_vector(&stances);
    let what =
        fbb.create_string(&polity.words_of(law, &rules.catalog.policies, &|id| pop.name_of(id)));
    let holder_name = law.holder.map(|h| fbb.create_string(&pop.name_of(h)));
    let policy = fbb.create_string(
        rules
            .catalog
            .policies
            .get(usize::from(law.policy))
            .map_or("", |d| d.id.as_str()),
    );
    let sponsor_name = fbb.create_string(&pop.name_of(law.sponsor));
    // The issue it answered, and the creed its sponsor proposed it under (M4c slice AG).
    let creed = pop
        .ideologies
        .creed_of(law.id)
        .and_then(|k| sim.rules.catalog.ideologies.get(usize::from(k)))
        .map(|d| format!(", as one who holds to {}", d.name))
        .unwrap_or_default();
    let issue = fbb.create_string(&format!("{}{creed}", law.issue.words()));
    // Under the body that decided it, which an amendment since may have changed.
    let body = law.decided.map_or(&polity.body, |t| polity.body_at(t));
    let decision = decision_words(law, body).map(|w| fbb.create_string(&w));
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
            broken: c.broken,
            broken_unaware: c.broken_unaware,
            refused: c.refused,
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
fn store_words(sim: &Sim, stores: &[f64]) -> String {
    let goods = &sim.rules.catalog.goods;
    let held: Vec<String> = goods
        .iter()
        .zip(stores)
        .filter(|(_, kg)| **kg >= 0.5)
        .map(|(g, kg)| format!("{} {kg:.0} kg", g.name.to_lowercase()))
        .collect();
    if held.is_empty() {
        "nothing".to_owned()
    } else {
        held.join(", ")
    }
}

/// The factions of `settlement` in words (wire 1.40, ADR-0017 §2), those with members first, the
/// newest first among each: "Mira's faction, against the gathering, since 3 May of year 2: 5
/// members, organized by Mira; its store holds grain 40 kg"; "Bo's faction, against the
/// storekeeper, from 3 May of year 2 until 9 June of year 3, when its last member left".
fn faction_words(sim: &Sim, settlement: civ_core::PermanentId) -> Vec<String> {
    let pop = &sim.people;
    let mut here: Vec<&civ_agents::faction::Faction> = pop
        .factions
        .list
        .iter()
        .filter(|f| f.settlement == settlement)
        .collect();
    here.sort_by(|a, b| {
        b.is_live()
            .cmp(&a.is_live())
            .then(b.founded.cmp(&a.founded))
            .then(b.id.cmp(&a.id))
    });
    here.into_iter()
        .map(|f| {
            let name = super::word::faction_name(sim, f);
            let against = super::word::blamed_words(sim, f.against);
            let since = civ_agents::polity::day_words(f.founded);
            match f.ended {
                None => {
                    let members = pop.factions.members_of(f.id).count();
                    format!(
                        "{name}, against {against}, since {since}: {members} member{}, organized \
                         by {}; its store holds {}",
                        if members == 1 { "" } else { "s" },
                        pop.name_of(f.organizer),
                        store_words(sim, &f.stores)
                    )
                }
                Some(end) => format!(
                    "{name}, against {against}, from {since} until {}, when its last member left",
                    civ_agents::polity::day_words(end)
                ),
            }
        })
        .collect()
}

/// The petitions of `settlement` in words (wire 1.41, ADR-0017 §3), newest first: "Mira's
/// faction petitioned the gathering on 3 May of year 2 for an end to the common store's levy (the
/// store gives what it holds): 9 came; the gathering turned it down".
pub fn petition_words(sim: &Sim, settlement: civ_core::PermanentId) -> Vec<String> {
    let pop = &sim.people;
    let today = sim.now().day_index();
    let laws: Vec<&Law> = pop.polities.iter().flat_map(|p| &p.laws).collect();
    let mut here: Vec<&civ_agents::faction::Petition> = pop
        .factions
        .petitions
        .iter()
        .filter(|p| p.settlement == settlement)
        .collect();
    here.sort_by(|a, b| b.day.cmp(&a.day).then(b.id.cmp(&a.id)));
    here.into_iter()
        .map(|p| {
            let name = pop.factions.get(p.faction).map_or_else(
                || "A faction".to_owned(),
                |f| super::word::faction_name(sim, f),
            );
            let demand = super::word::petition_demand_words(sim, p);
            let on = day_words(SimTime::from_minutes(p.day * 24 * 60));
            if p.day >= today && p.law.is_none() && !p.answered {
                let so_far = if p.day == today && !p.came.is_empty() {
                    format!("; {} have come so far", p.came.len())
                } else {
                    String::new()
                };
                return format!(
                    "{name} petitions the gathering on the evening of {on} for {demand}{so_far}"
                );
            }
            let came = match p.came.len() {
                0 => "nobody came".to_owned(),
                n => format!("{n} came"),
            };
            let answer = match p.law.and_then(|id| laws.iter().find(|l| l.id == id)) {
                Some(l) => match l.outcome {
                    Some(Outcome::Passed) => "the gathering granted it".to_owned(),
                    Some(Outcome::Failed) => "the gathering turned it down".to_owned(),
                    Some(Outcome::Tied) => "the gathering was split on it".to_owned(),
                    Some(Outcome::NoQuorum) => {
                        "too few came to the gathering to decide it".to_owned()
                    }
                    None => format!(
                        "it goes before the gathering on {}",
                        day_words(SimTime::from_minutes(l.meets_day * 24 * 60))
                    ),
                },
                None if p.came.is_empty() => "so it ended there".to_owned(),
                None if p.answered => "it never came before the gathering".to_owned(),
                None => "it waits for the gathering to be free".to_owned(),
            };
            format!("{name} petitioned the gathering on {on} for {demand}: {came}; {answer}")
        })
        .collect()
}

/// The refusals of a levy at `settlement` in words (wire 1.42, ADR-0017 §3), newest first:
/// "Mira's faction called on its members on 3 May of year 2 to keep back the levy of a common
/// store, taking a tenth of each harvest, until 3 May of year 3: 6 kept back 240 kg".
pub fn refusal_words(sim: &Sim, settlement: civ_core::PermanentId) -> Vec<String> {
    let pop = &sim.people;
    let today = sim.now().day_index();
    let mut here: Vec<&civ_agents::faction::Refusal> = pop
        .factions
        .refusals
        .iter()
        .filter(|r| r.settlement == settlement)
        .collect();
    here.sort_by(|a, b| b.called.cmp(&a.called).then(b.id.cmp(&a.id)));
    here.into_iter()
        .map(|r| {
            let name = pop.factions.get(r.faction).map_or_else(
                || "A faction".to_owned(),
                |f| super::word::faction_name(sim, f),
            );
            let law = super::word::refusal_law_words(sim, r);
            let (called, until) = (
                day_words(r.called),
                day_words(SimTime::from_minutes(r.until * 24 * 60)),
            );
            let kept = match r.kept.len() {
                0 => "nobody has kept any back".to_owned(),
                n => format!("{n} kept back {:.0} kg", r.kept_kg),
            };
            if r.until >= today {
                format!(
                    "{name} calls on its members, since {called}, to keep back the levy of {law}, \
                     until {until}: {kept} so far"
                )
            } else {
                format!(
                    "{name} called on its members on {called} to keep back the levy of {law}, \
                     until {until}: {kept}"
                )
            }
        })
        .collect()
}

/// The revolts at `settlement` in words (wire 1.43, ADR-0017 §4), newest first: "Mira's faction
/// called on everyone on 3 May of year 2 to stand with it: from now on, the elders of its
/// households decide ...; it held on 20 May of year 2, 14 standing with it, 9 with the gathering".
pub fn revolt_words(sim: &Sim, settlement: civ_core::PermanentId) -> Vec<String> {
    use civ_agents::faction::{RevoltEnd, Side};
    let pop = &sim.people;
    let mut here: Vec<&civ_agents::faction::Revolt> = pop
        .factions
        .revolts
        .iter()
        .filter(|r| r.settlement == settlement)
        .collect();
    here.sort_by(|a, b| b.called.cmp(&a.called).then(b.id.cmp(&a.id)));
    here.into_iter()
        .map(|r| {
            let name = pop.factions.get(r.faction).map_or_else(
                || "A faction".to_owned(),
                |f| super::word::faction_name(sim, f),
            );
            let called = day_words(r.called);
            let counts = format!(
                "{} standing with it, {} with the gathering, {} with neither",
                r.count(Side::With),
                r.count(Side::Gathering),
                r.count(Side::Neither)
            );
            let how = match r.ended {
                None => format!(
                    "it stands until {}: {counts} now",
                    day_words(SimTime::from_minutes(r.until * 24 * 60))
                ),
                Some((RevoltEnd::Held, at)) => {
                    format!("it held on {}, {counts}", day_words(at))
                }
                Some((RevoltEnd::Failed, at)) => {
                    format!("it came to nothing on {}, {counts}", day_words(at))
                }
            };
            format!(
                "{name} called on everyone on {called} to stand with it: from now on, {}; {how}",
                r.body.clause()
            )
        })
        .collect()
}

/// The coups at `settlement` in words (wire 1.45, ADR-0017 §4), newest first: "Bo called on those
/// who keep the watch on 3 May of year 2 to take the deciding with them; it held on 11 May of year
/// 2, 2 of 3 watchers with it, none with the gathering".
pub fn coup_words(sim: &Sim, settlement: civ_core::PermanentId) -> Vec<String> {
    use civ_agents::faction::{RevoltEnd, Side};
    let pop = &sim.people;
    let mut here: Vec<&civ_agents::faction::Coup> = pop
        .factions
        .coups
        .iter()
        .filter(|c| c.settlement == settlement)
        .collect();
    here.sort_by(|a, b| b.called.cmp(&a.called).then(b.id.cmp(&a.id)));
    here.into_iter()
        .map(|c| {
            let count = |n: usize| match n {
                0 => "none".to_owned(),
                n => n.to_string(),
            };
            let counts = format!(
                "{} of {} watchers with it, {} with the gathering",
                count(c.count(Side::With)),
                c.sides.len(),
                count(c.count(Side::Gathering))
            );
            let how = match c.ended {
                None => format!(
                    "it stands until {}: {counts} now",
                    day_words(SimTime::from_minutes(c.until * 24 * 60))
                ),
                Some((RevoltEnd::Held, at)) => format!("it held on {}, {counts}", day_words(at)),
                Some((RevoltEnd::Failed, at)) => {
                    format!("it came to nothing on {}, {counts}", day_words(at))
                }
            };
            format!(
                "{} called on those who keep the watch on {} to take the deciding with them; {how}",
                pop.name_of(c.challenger),
                day_words(c.called)
            )
        })
        .collect()
}

/// A `Response` with every settlement's polity.
pub fn government_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let (pop, rules) = (&sim.people, &sim.rules);
    let now = sim.now();
    let goods = &rules.catalog.goods;
    let adult = rules.people.family.independent_age;
    let mut list = Vec::new();
    for (pi, polity) in pop.polities.iter().enumerate() {
        let settlement = sim
            .land
            .settlements
            .iter()
            .find(|s| s.id == polity.settlement);
        let name = fbb.create_string(settlement.map_or("", |s| s.name.as_str()));
        let custom = fbb.create_string(&polity.body.words());
        let store = fbb.create_string(&store_words(sim, &polity.stores));
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
        // The custom's versions, and how many its body admits now (wire 1.35, M4c slice AF).
        let history: Vec<_> =
            civ_agents::polity::custom_history_words(polity, &|id| pop.name_of(id))
                .iter()
                .map(|w| fbb.create_string(w))
                .collect();
        let custom_history = fbb.create_vector(&history);
        // Its factions (wire 1.40, M4c slice AH).
        let factions: Vec<_> = faction_words(sim, polity.settlement)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        let factions = fbb.create_vector(&factions);
        // Its petitions (wire 1.41, M4c slice AH).
        let petitions: Vec<_> = petition_words(sim, polity.settlement)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        let petitions = fbb.create_vector(&petitions);
        // Its refusals of a levy (wire 1.42, M4c slice AH).
        let refusals: Vec<_> = refusal_words(sim, polity.settlement)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        let refusals = fbb.create_vector(&refusals);
        // Its revolts (wire 1.43, M4c slice AI).
        let revolts: Vec<_> = revolt_words(sim, polity.settlement)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        let revolts = fbb.create_vector(&revolts);
        // Its coups (wire 1.45, M4c slice AI, step three).
        let coups: Vec<_> = coup_words(sim, polity.settlement)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        let coups = fbb.create_vector(&coups);
        let body_members = pop
            .body_members(&sim.land.fields, pi, now, &rules.people)
            .len() as u32;
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
        // How it stands toward each other lived-in polity, from its own side (wire 1.57).
        let relations: Vec<_> = crate::relations::others(sim, polity.id)
            .map(|other| {
                let label = crate::relations::relation_of(sim, polity, other);
                let name = sim
                    .land
                    .settlements
                    .iter()
                    .find(|s| s.id == other.settlement)
                    .map_or("", |s| s.name.as_str());
                let name = fbb.create_string(name);
                let standing = fbb.create_string(label.standing.words());
                let why: Vec<_> = label.why.iter().map(|w| fbb.create_string(w)).collect();
                let why = fbb.create_vector(&why);
                // The agreements between the two, both law histories side by side (wire 1.58).
                let agreements: Vec<_> = crate::relations::agreements_between(sim, polity, other)
                    .iter()
                    .map(|a| {
                        let terms = fbb.create_string(&a.terms);
                        let state = fbb.create_string(&a.state);
                        let ours = fbb.create_string(&a.ours);
                        let theirs = fbb.create_string(&a.theirs);
                        // Its payments (wire 1.59).
                        let payments: Vec<_> =
                            a.payments.iter().map(|w| fbb.create_string(w)).collect();
                        let payments = fbb.create_vector(&payments);
                        wire::AgreementLine::create(
                            &mut fbb,
                            &wire::AgreementLineArgs {
                                id: a.id.get(),
                                terms: Some(terms),
                                state: Some(state),
                                ours: Some(ours),
                                theirs: Some(theirs),
                                payments: Some(payments),
                            },
                        )
                    })
                    .collect();
                let agreements = fbb.create_vector(&agreements);
                wire::RelationLine::create(
                    &mut fbb,
                    &wire::RelationLineArgs {
                        polity: other.id.get(),
                        name: Some(name),
                        label: Some(standing),
                        why: Some(why),
                        agreements: Some(agreements),
                    },
                )
            })
            .collect();
        let relations = fbb.create_vector(&relations);
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
                custom_history: Some(custom_history),
                factions: Some(factions),
                petitions: Some(petitions),
                refusals: Some(refusals),
                revolts: Some(revolts),
                coups: Some(coups),
                body_members,
                relations: Some(relations),
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
