//! Each settlement's polity for the observer (wire 1.27, ADR-0013): its custom, its members, its
//! store, every law proposed there with its whole history, its offices (1.28) and its label
//! (1.29). The kernel renders the words; the observer only shows them.

use civ_agents::polity::{
    Law, LawStatus, Outcome, Polity, Stance, day_words, decision_words, office_words, stance_words,
};
use civ_core::{PermanentId, SimTime};
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
        LawStatus::StruckDown => 6,
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
    let what = fbb.create_string(&super::word::law_text(sim, polity, law));
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
pub fn faction_words(sim: &Sim, settlement: civ_core::PermanentId) -> Vec<String> {
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
            let since = civ_agents::polity::day_words(f.founded);
            if let Some(st) = f.stake {
                return organization_words(sim, f, &st, &name, &since);
            }
            let against = super::word::blamed_words(sim, f.against);
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

/// An organization by stake in words (M7c slice BW, ADR-0026 §2): what it is of and its rules,
/// its members and where those of another settlement live, its applications, and who recognizes
/// it: "Mira's organization, of those who sell tools (it admits one a member speaks for, its
/// organizer decides, and its dues are a twentieth of each threshing), since 3 May of year 2: 3
/// members, 1 of them of Sedgebrook, organized by Mira; 2 admitted and 1 turned away; its store
/// holds grain 4 kg; Oakholt's gathering recognizes it".
fn organization_words(
    sim: &Sim,
    f: &civ_agents::faction::Faction,
    st: &civ_agents::faction::Stake,
    name: &str,
    since: &str,
) -> String {
    use civ_agents::faction::FactionEventKind;
    let pop = &sim.people;
    let of = format!(
        "{name}, of those who sell {} (it admits {}, {} decides, and its dues are {} of each \
         threshing)",
        st.purpose.plural_words(),
        st.admit.words(),
        st.decides.words(),
        civ_agents::polity::share_text(f64::from(st.dues_share)),
    );
    let recognized = super::word::recognized_words(sim, f.id);
    if let Some(end) = f.ended {
        return format!(
            "{of}, from {since} until {}, when its last member left{recognized}",
            civ_agents::polity::day_words(end)
        );
    }
    let members: Vec<PermanentId> = pop.factions.members_of(f.id).map(|m| m.person).collect();
    let mut elsewhere: Vec<(PermanentId, usize)> = Vec::new();
    for &m in &members {
        let Some(s) = pop
            .person(m)
            .and_then(|p| pop.household(p.household))
            .and_then(|h| h.settlement)
            .filter(|&s| s != f.settlement)
        else {
            continue;
        };
        match elsewhere.iter_mut().find(|e| e.0 == s) {
            Some(e) => e.1 += 1,
            None => elsewhere.push((s, 1)),
        }
    }
    elsewhere.sort_by_key(|e| e.0);
    let away = elsewhere
        .iter()
        .map(|&(s, n)| format!(", {n} of them of {}", settlement_name(sim, s)))
        .collect::<String>();
    let count = |k: FactionEventKind| f.history.iter().filter(|e| e.kind == k).count();
    let (admitted, refused) = (
        count(FactionEventKind::Admitted),
        count(FactionEventKind::TurnedAway),
    );
    let applications = if admitted + refused == 0 {
        String::new()
    } else {
        format!("; {admitted} admitted and {refused} turned away")
    };
    format!(
        "{of}, since {since}: {} member{}{away}, organized by {}{applications}; its store holds \
         {}{recognized}",
        members.len(),
        if members.len() == 1 { "" } else { "s" },
        pop.name_of(f.organizer),
        store_words(sim, &f.stores)
    )
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

/// The name of settlement `s`, or "another settlement".
fn settlement_name(sim: &Sim, s: civ_core::PermanentId) -> String {
    sim.land
        .settlements
        .iter()
        .find(|x| x.id == s)
        .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
}

/// How a polity came to hold a settlement, in words.
fn held_how_words(how: civ_agents::polity::HeldHow) -> &'static str {
    match how {
        civ_agents::polity::HeldHow::Annexed => "by annexation",
        civ_agents::polity::HeldHow::Merged => "as one body, by agreement",
    }
}

/// An election in words for the observer (wire 1.77, M7a slice BQ).
#[derive(Clone, Debug)]
pub struct ElectionWords {
    pub id: u32,
    pub heading: String,
    pub stage: String,
    pub turnout: String,
    pub districts: Vec<String>,
    pub alliances: Vec<String>,
    pub candidates: Vec<CandidateWords>,
}

/// One who stood, in words (wire 1.77).
#[derive(Clone, Debug)]
pub struct CandidateWords {
    pub person: u64,
    pub name: String,
    pub district: String,
    pub told: String,
    pub backed: String,
    pub votes: u32,
    pub seated: bool,
}

/// Polity `pi`'s council's elections in words, newest first, at most six (wire 1.77, ADR-0024
/// §3): worked out when asked, read by nothing in the world.
pub fn election_words(sim: &Sim, pi: usize) -> Vec<ElectionWords> {
    use civ_agents::seats::{ElectionStage, Package, SeatMethod};
    let pop = &sim.people;
    let polity = &pop.polities[pi];
    let day = |d: i64| day_words(SimTime::from_minutes(d * civ_core::time::MINUTES_PER_DAY));
    let place = |s: civ_core::PermanentId| {
        sim.land
            .settlements
            .iter()
            .find(|x| x.id == s)
            .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
    };
    let faction = |f: civ_core::PermanentId| {
        pop.factions.get(f).map_or_else(
            || "a faction".to_owned(),
            |f| super::word::faction_name(sim, f),
        )
    };
    let policies = &sim.rules.catalog.policies;
    polity
        .elections
        .iter()
        .rev()
        .take(6)
        .map(|e| {
            let heading = format!(
                "Election {}, called on {} because {}: {}",
                e.id,
                day(e.called),
                e.why.words(),
                e.seats.words()
            );
            let stage = match e.stage {
                ElectionStage::Standing => format!(
                    "open for names until {}; {} have stood",
                    day(e.stand_until),
                    e.candidates.len()
                ),
                ElectionStage::Polling => match e.seats.method {
                    SeatMethod::Election => format!("to be polled on {}", day(e.poll_day)),
                    SeatMethod::Lot => format!("the lot is drawn on {}", day(e.poll_day)),
                },
                ElectionStage::Installed => format!(
                    "counted and seated on {}{}",
                    day(e.ended.unwrap_or(e.poll_day)),
                    if e.by_lot {
                        "; a lot settled a tie"
                    } else {
                        ""
                    }
                ),
                stage => format!(
                    "{} on {}",
                    stage.words(),
                    day(e.ended.unwrap_or(e.poll_day))
                ),
            };
            let turnout = if e.seats.method == SeatMethod::Lot {
                "drawn by lot from those who put their names in".to_owned()
            } else if e.ballots.is_empty() {
                if e.stage == ElectionStage::Installed {
                    "nobody polled: those who stood sat unopposed".to_owned()
                } else {
                    String::new()
                }
            } else {
                let now = pop.electorate_count(
                    &sim.land.fields,
                    pi,
                    &e.seats,
                    sim.now(),
                    &sim.rules.people,
                );
                format!(
                    "{} of the {now} now of the electorate came to poll",
                    e.ballots.len()
                )
            };
            let districts = e
                .districts
                .iter()
                .map(|&(d, n)| format!("{}: {n} seat{}", place(d), if n == 1 { "" } else { "s" }))
                .collect();
            let alliances = e
                .alliances
                .iter()
                .map(|a| {
                    let how = match a.package {
                        Package::Common => "stand on one slate",
                        Package::FirstStandsDown => "back the second's slate",
                        Package::SecondStandsDown => "back the first's slate",
                    };
                    format!(
                        "{} and {} {how}",
                        faction(a.factions[0]),
                        faction(a.factions[1])
                    )
                })
                .collect();
            let candidates = e
                .candidates
                .iter()
                .map(|c| {
                    let told: Vec<String> = c
                        .positions
                        .iter()
                        .filter_map(|&(q, x)| {
                            let q = policies.get(usize::from(q))?.question.as_deref()?;
                            Some(format!(
                                "{} {q}",
                                civ_agents::opinion::lean_words(f64::from(x))
                            ))
                        })
                        .collect();
                    let backers: Vec<String> = e
                        .endorsements
                        .iter()
                        .filter(|s| s.1.contains(&c.person))
                        .map(|s| faction(s.0))
                        .collect();
                    CandidateWords {
                        person: c.person.get(),
                        name: pop.name_of(c.person),
                        district: c.district.map_or_else(String::new, place),
                        told: told.join("; "),
                        backed: if backers.is_empty() {
                            String::new()
                        } else {
                            format!("backed by {}", backers.join(" and "))
                        },
                        votes: e.totals.iter().find(|t| t.0 == c.person).map_or(0, |t| t.1),
                        seated: e.winners.contains(&c.person),
                    }
                })
                .collect();
            ElectionWords {
                id: e.id,
                heading,
                stage,
                turnout,
                districts,
                alliances,
                candidates,
            }
        })
        .collect()
}

/// Polity `polity`'s settlements in words (wire 1.76, M6c slice BM), its seat first: "Oakholt,
/// its seat, since 3 May of year 1"; "Ashford, taken in on 9 June of year 12 from Ashford's
/// polity, by annexation".
pub fn polity_settlement_words(sim: &Sim, polity: &Polity) -> Vec<String> {
    let pop = &sim.people;
    let seat = format!(
        "{}, its seat, since {}",
        settlement_name(sim, polity.settlement),
        day_words(polity.founded)
    );
    std::iter::once(seat)
        .chain(polity.held.iter().map(|h| {
            let from = pop
                .polities
                .iter()
                .find(|p| p.id == h.from)
                .filter(|p| p.id != polity.id)
                .map_or_else(String::new, |p| {
                    format!(" from {}'s polity", settlement_name(sim, p.settlement))
                });
            format!(
                "{}, taken in on {}{from}, {}",
                settlement_name(sim, h.settlement),
                day_words(SimTime::from_minutes(h.since * 24 * 60)),
                held_how_words(h.how)
            )
        }))
        .collect()
}

/// How polity `polity`'s deciding closed, in words (wire 1.76): "taken in by Oakholt's polity on
/// 9 June of year 12, by annexation"; empty while it still decides.
pub fn polity_ended_words(sim: &Sim, polity: &Polity) -> String {
    polity.ended.map_or_else(String::new, |e| {
        let by = sim
            .people
            .polities
            .iter()
            .find(|p| p.id == e.by)
            .map_or_else(
                || "another polity".to_owned(),
                |p| format!("{}'s polity", settlement_name(sim, p.settlement)),
            );
        format!(
            "taken in by {by} on {}, {}",
            day_words(SimTime::from_minutes(e.day * 24 * 60)),
            held_how_words(e.how)
        )
    })
}

/// Food energy in the stores of households `households` now, kcal, and how many people they
/// keep.
fn households_food(sim: &Sim, households: &[civ_core::PermanentId]) -> (f64, usize) {
    let (pop, rules, now) = (&sim.people, &sim.rules, sim.now());
    let goods = &rules.catalog.goods;
    let (mut kcal, mut people) = (0.0, 0usize);
    for &h in households {
        if let Some(x) = pop.household(h) {
            let stores = civ_agents::population::stores_now(x, now, &rules.people, goods);
            kcal += civ_agents::person::stock_kcal(&stores, goods);
            people += x.members.len();
        }
    }
    (kcal, people)
}

/// Days of food `kcal` would keep `people` at what each eats in a day.
fn food_days(sim: &Sim, kcal: f64, people: usize) -> f64 {
    let daily = sim.rules.people.household.daily_kcal_per_person;
    kcal / (people as f64 * daily).max(1.0)
}

/// One siege's facts and words (wire 1.76, ADR-0023 §7 and §11): the band, the place it camps by
/// or holds, each side's food at current use, and what comes and goes by the line. Worked out
/// afterwards; nothing in the world reads it.
#[derive(Clone, Debug)]
pub struct Siege {
    /// The band, the settlement it is of, the settlement it camps by or holds, and its stage
    /// (4 camped, 5 holding).
    pub band: u32,
    pub from: civ_core::PermanentId,
    pub at: civ_core::PermanentId,
    pub stage: u8,
    /// The band in words, as the map's band brief has it.
    pub words: String,
    /// Inside: the food of the place's households and of a common store kept there, in words
    /// and in days at what its people eat.
    pub inside: String,
    pub inside_days: f64,
    /// Water and fields by the line, in words.
    pub comes_in: String,
    /// The band's own food, in words and in days for those there.
    pub band_food: String,
    pub band_days: f64,
}

/// Every band camped by a shut line or holding a settlement, as sieges, oldest band first.
pub fn sieges(sim: &Sim) -> Vec<Siege> {
    use civ_agents::war::BandStage;
    let (pop, land) = (&sim.people, &sim.land);
    let goods = &sim.rules.catalog.goods;
    let words = super::people::band_lines(sim);
    pop.order
        .bands
        .iter()
        .filter(|b| matches!(b.stage, BandStage::Camped | BandStage::Holding))
        .filter_map(|b| {
            let at = pop.household(b.target)?.settlement?;
            // Inside: the households of the place, and the common store if it is kept there.
            let mut theirs: Vec<civ_core::PermanentId> = pop
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(at) && !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            theirs.sort_unstable();
            let (kcal, people) = households_food(sim, &theirs);
            let mut inside_days = food_days(sim, kcal, people);
            let mut inside = format!(
                "its {} households hold about {:.0} days of food for {people} people at what \
                 they eat",
                theirs.len(),
                inside_days
            );
            if let Some(p) = pop
                .polities
                .iter()
                .find(|p| p.holds(Some(at)) && p.settlement == at)
            {
                let more = food_days(
                    sim,
                    civ_agents::person::stock_kcal(&p.stores, goods),
                    people,
                );
                if more >= 0.5 {
                    inside.push_str(&format!(
                        ", and the common store kept there {more:.0} days more"
                    ));
                    inside_days += more;
                }
            }
            // What comes and goes by the line: water and fields.
            let line = land
                .enclosures
                .list
                .iter()
                .find(|e| e.settlement == at && e.finished.is_some());
            let comes_in = match line {
                None => "no line stands round it: its people come and go as they would".to_owned(),
                Some(e) => {
                    let wells = land
                        .wells
                        .list
                        .iter()
                        .filter(|w| w.is_open() && e.inside(w.rect.centre_m()))
                        .count();
                    let water = match wells {
                        0 => "no open well inside the line: every walk to water goes out \
                              through a gate"
                            .to_owned(),
                        1 => "an open well inside the line".to_owned(),
                        n => format!("{n} open wells inside the line"),
                    };
                    let fields: Vec<_> = land
                        .fields
                        .iter()
                        .filter(|f| theirs.binary_search(&f.household).is_ok())
                        .collect();
                    let outside = fields
                        .iter()
                        .filter(|f| !e.inside(f.rect.centre_m()))
                        .count();
                    format!(
                        "{water}; {outside} of its {} fields lie outside it",
                        fields.len()
                    )
                }
            };
            // The band: what those there carry, and what their households at home hold.
            let there: Vec<_> = b
                .arrived
                .iter()
                .filter(|m| !b.home.contains(m))
                .filter_map(|&m| pop.person(m))
                .collect();
            let carried: f64 = there
                .iter()
                .filter_map(|p| {
                    let g = goods.get(usize::from(p.carrying.good?))?;
                    (g.purpose == civ_agents::params::GoodUse::Food)
                        .then(|| f64::from(p.carrying.kg) * g.kcal_per_kg)
                })
                .sum();
            let band_days = food_days(sim, carried, there.len());
            let mut homes: Vec<civ_core::PermanentId> = there.iter().map(|p| p.household).collect();
            homes.sort_unstable();
            homes.dedup();
            let (home_kcal, home_people) = households_food(sim, &homes);
            let took = if b.taken_kcal > 0.0 {
                format!(
                    " (it took {:.0} days' food for them)",
                    food_days(sim, f64::from(b.taken_kcal), there.len())
                )
            } else {
                String::new()
            };
            let band_food = format!(
                "{} there carry about {band_days:.0} days of food{took}; their households at \
                 home hold about {:.0} days for their {home_people} people",
                there.len(),
                food_days(sim, home_kcal, home_people)
            );
            let words = words
                .iter()
                .find(|w| w.0 == b.id)
                .map_or_else(String::new, |w| w.2.clone());
            Some(Siege {
                band: b.id,
                from: b.settlement,
                at,
                stage: b.stage as u8,
                words,
                inside,
                inside_days,
                comes_in,
                band_food,
                band_days,
            })
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
                    .is_some_and(|x| polity.holds(x.settlement))
            })
            .count() as u32;
        let laws: Vec<_> = polity
            .laws
            .iter()
            .rev()
            .map(|l| law_line(&mut fbb, sim, polity, l))
            .collect();
        let laws = fbb.create_vector(&laws);
        let mut offices = office_words(
            polity,
            &rules.catalog.policies,
            &|id| pop.name_of(id),
            &|id| gone_words(sim, id),
            now.day_index(),
        );
        // Its council's seats and latest election, if it has one (M7a slice BO).
        offices.extend(civ_agents::seats::council_words(
            polity.body.seats,
            &polity.seated,
            &polity.elections,
            &|id| pop.name_of(id),
            &|f| {
                pop.factions.get(f).map_or_else(
                    || "a faction".to_owned(),
                    |f| super::word::faction_name(sim, f),
                )
            },
            &|s| {
                sim.land
                    .settlements
                    .iter()
                    .find(|x| x.id == s)
                    .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
            },
            &|d| day_words(SimTime::from_minutes(d * civ_core::time::MINUTES_PER_DAY)),
        ));
        let offices: Vec<_> = offices.iter().map(|w| fbb.create_string(w)).collect();
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
        // Its settlements, and how its deciding closed (wire 1.76, M6c slice BM).
        let settlements: Vec<_> = polity_settlement_words(sim, polity)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        let settlements = fbb.create_vector(&settlements);
        let ended = fbb.create_string(&polity_ended_words(sim, polity));
        // Its council's elections (wire 1.77, M7a slice BQ).
        let elections: Vec<_> = election_words(sim, pi)
            .into_iter()
            .map(|e| {
                let candidates: Vec<_> = e
                    .candidates
                    .iter()
                    .map(|c| {
                        let name = fbb.create_string(&c.name);
                        let district = fbb.create_string(&c.district);
                        let told = fbb.create_string(&c.told);
                        let backed = fbb.create_string(&c.backed);
                        wire::CandidateLine::create(
                            &mut fbb,
                            &wire::CandidateLineArgs {
                                person: c.person,
                                name: Some(name),
                                district: Some(district),
                                told: Some(told),
                                backed: Some(backed),
                                votes: c.votes,
                                seated: c.seated,
                            },
                        )
                    })
                    .collect();
                let candidates = fbb.create_vector(&candidates);
                let heading = fbb.create_string(&e.heading);
                let stage = fbb.create_string(&e.stage);
                let turnout = fbb.create_string(&e.turnout);
                let districts: Vec<_> = e.districts.iter().map(|w| fbb.create_string(w)).collect();
                let districts = fbb.create_vector(&districts);
                let alliances: Vec<_> = e.alliances.iter().map(|w| fbb.create_string(w)).collect();
                let alliances = fbb.create_vector(&alliances);
                wire::ElectionLine::create(
                    &mut fbb,
                    &wire::ElectionLineArgs {
                        id: e.id,
                        heading: Some(heading),
                        stage: Some(stage),
                        turnout: Some(turnout),
                        districts: Some(districts),
                        alliances: Some(alliances),
                        candidates: Some(candidates),
                    },
                )
            })
            .collect();
        let elections = fbb.create_vector(&elections);
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
                settlements: Some(settlements),
                ended: Some(ended),
                elections: Some(elections),
            },
        ));
    }
    let polities = fbb.create_vector(&list);
    let sieges: Vec<_> = sieges(sim)
        .into_iter()
        .map(|x| {
            let words = fbb.create_string(&x.words);
            let inside = fbb.create_string(&x.inside);
            let comes_in = fbb.create_string(&x.comes_in);
            let band_food = fbb.create_string(&x.band_food);
            wire::SiegeLine::create(
                &mut fbb,
                &wire::SiegeLineArgs {
                    band: x.band,
                    from: x.from.get(),
                    at: x.at.get(),
                    stage: x.stage,
                    words: Some(words),
                    inside: Some(inside),
                    inside_days: x.inside_days as f32,
                    comes_in: Some(comes_in),
                    band_food: Some(band_food),
                    band_days: x.band_days as f32,
                },
            )
        })
        .collect();
    let sieges = fbb.create_vector(&sieges);
    let body = wire::Government::create(
        &mut fbb,
        &wire::GovernmentArgs {
            minute: now.minutes(),
            polities: Some(polities),
            sieges: Some(sieges),
        },
    );
    response(fbb, wire::ResponseBody::Government, body)
}
