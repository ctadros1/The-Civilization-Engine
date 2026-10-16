//! Grievances and word of mouth for the observer (wire 1.34, ADR-0016 §2–§3): what one person
//! holds against whom and why, and what they have heard that is still news, with who told them.
//! Per person, in the inspector, never in the frame (ADR-0016 §7). The kernel renders the words;
//! the observer only shows them.

use civ_agents::Person;
use civ_agents::influence::{Influence, InfluenceKind};
use civ_agents::params::Catalog;
use civ_agents::polity::{day_words, law_words};
use civ_agents::word::{Blamed, Claim, ClaimKind};
use civ_core::{PermanentId, SimTime};
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use crate::Sim;

/// The most claims heard the inspector is sent for one person.
pub const MAX_HEARD_SHOWN: usize = 12;
/// The most claims the inspector offers to whisper to one person (M4c slice AJ).
pub const MAX_NEWS_SHOWN: usize = 12;

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
        // A forum (M7b slice BU): who sits in it.
        Blamed::Forum(law) => pop
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .find(|l| l.id == law)
            .and_then(|l| {
                let def = sim
                    .rules
                    .catalog
                    .policies
                    .get(usize::from(l.policy))?
                    .forum
                    .as_ref()?;
                let holder = l.holder.map(|h| pop.name_of(h));
                let w = civ_agents::forum::hearers_words(def, holder.as_deref());
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_lowercase().collect::<String>() + c.as_str())
            })
            .unwrap_or_else(|| "a forum".to_owned()),
        // An organization by stake that turned them away (M7c slice BW).
        Blamed::Organization(id) => pop
            .factions
            .get(id)
            .map_or_else(|| "an organization".to_owned(), |f| faction_name(sim, f)),
    }
}

/// Law `law` of polity `polity` in words, as [`civ_agents::polity::Polity::words_of`] says it,
/// but a closing naming its source as its people know it (M6a slice BA, step four): "the closing
/// of Wren's household's well: nobody may draw there".
pub fn law_text(
    sim: &Sim,
    polity: &civ_agents::polity::Polity,
    law: &civ_agents::polity::Law,
) -> String {
    if law.kind == civ_agents::polity::PolicyKind::CloseSource
        && let Some(source) = polity.closure_of(law.id)
    {
        return format!(
            "the closing of {}: nobody may draw there",
            super::sickness::place_words(sim, source)
        );
    }
    // What names an organization or a settlement says which (M7c slices BV-BX).
    let org = || {
        law.target
            .and_then(|t| sim.people.factions.get(t))
            .map_or_else(|| "an organization".to_owned(), |f| faction_name(sim, f))
    };
    let def = sim.rules.catalog.policies.get(usize::from(law.policy));
    match law.kind {
        civ_agents::polity::PolicyKind::Recognize => {
            return format!(
                "recognition of {}: its members owe no duty on the goods it sells",
                org()
            );
        }
        civ_agents::polity::PolicyKind::Privilege => {
            if let Some(g) = def.and_then(|d| d.privilege) {
                return format!("a privilege: {} alone {}", org(), g.words());
            }
        }
        civ_agents::polity::PolicyKind::Embargo => {
            if let Some(t) = law.target {
                return format!(
                    "an embargo: nobody of theirs may buy at {}'s doors",
                    place_name(sim, t)
                );
            }
        }
        _ => {}
    }
    polity.words_of(law, &sim.rules.catalog.policies, &|id| {
        sim.people.name_of(id)
    })
}

/// What a claim says, in words: "a gathering meets on 3 May of year 2 to decide on a common
/// store, taking a tenth of each harvest"; "Bo holds a grievance against the gathering over food
/// when their household was short"; "someone of Bo's household lay sick from 3 May of year 2".
pub fn claim_words(sim: &Sim, c: &Claim) -> String {
    let pop = &sim.people;
    let name_of = |id: PermanentId| pop.name_of(id);
    match c.kind {
        // A finding (M7b slice BS).
        ClaimKind::Finding => {
            let day = day_words(SimTime::from_minutes(c.day * DAY));
            let place = place_name(sim, c.settlement);
            match c.event.and_then(|id| pop.order.case(id)) {
                Some(case) => format!(
                    "case {} at {place}, that {} took food from another household, was {} on \
                     {day}",
                    case.id,
                    name_of(case.accused),
                    if case.stage == civ_agents::crime::CaseStage::Found {
                        "found"
                    } else {
                        "not found"
                    }
                ),
                None => format!("a case was decided at {place} on {day}"),
            }
        }
        // A council's election (M7a slice BO).
        ClaimKind::Election => {
            let day = day_words(SimTime::from_minutes(c.day * DAY));
            let place = sim
                .land
                .settlements
                .iter()
                .find(|s| s.id == c.settlement)
                .map_or("the hearth", |s| s.name.as_str());
            let seats = pop
                .polities
                .iter()
                .find(|p| p.settlement == c.settlement)
                .and_then(|p| p.elections.iter().rev().find(|e| e.poll_day == c.day))
                .map(|e| e.seats);
            match seats {
                Some(s) if s.method == civ_agents::seats::SeatMethod::Lot => {
                    format!("the lot for {} is drawn at {place} on {day}", s.words())
                }
                Some(s) => format!("{} is polled at {place} on {day}", s.words()),
                None => format!("an election is held at {place} on {day}"),
            }
        }
        // One who stands in a council's election, and where they said they stand (M7a slice BP).
        ClaimKind::Candidacy => {
            let day = day_words(SimTime::from_minutes(c.day * DAY));
            let who = c.subject.map_or_else(|| "someone".to_owned(), name_of);
            let election = pop
                .polities
                .iter()
                .find(|p| p.settlement == c.settlement)
                .and_then(|p| p.elections.iter().rev().find(|e| e.poll_day == c.day));
            let mut seats = election.map_or_else(|| "a seat".to_owned(), |e| e.seats.words());
            // Where seats are shared among settlements, the district they stand in (M7a slice BQ).
            if let Some(d) = election
                .zip(c.subject)
                .and_then(|(e, who)| e.candidates.iter().find(|x| x.person == who))
                .and_then(|x| x.district)
            {
                seats = format!("{seats}, in {}", place_name(sim, d));
            }
            let policies = &sim.rules.catalog.policies;
            let told: Vec<String> = election
                .zip(c.subject)
                .and_then(|(e, who)| e.positions_of(who))
                .into_iter()
                .flatten()
                .filter_map(|&(q, x)| {
                    let q = policies.get(usize::from(q))?.question.as_deref()?;
                    Some(format!(
                        "{} on {q}",
                        civ_agents::opinion::lean_words(f64::from(x))
                    ))
                })
                .collect();
            if told.is_empty() {
                format!("{who} stands for {seats}, polled on {day}")
            } else {
                format!(
                    "{who} stands for {seats}, polled on {day}: {}",
                    told.join("; ")
                )
            }
        }
        ClaimKind::Gathering => {
            let day = day_words(SimTime::from_minutes(c.day * DAY));
            let law = c.subject.and_then(|id| {
                pop.polities
                    .iter()
                    .find_map(|p| Some((p, p.laws.iter().find(|l| l.id == id)?)))
            });
            match law {
                Some((p, l)) => format!(
                    "a gathering meets on {day} to decide on {}",
                    law_text(sim, p, l)
                ),
                None => format!("a gathering meets on {day} to hear a case"),
            }
        }
        ClaimKind::Petition => {
            let day = day_words(SimTime::from_minutes(c.day * DAY));
            match c
                .subject
                .and_then(|id| pop.factions.petitions.iter().find(|p| p.id == id))
            {
                Some(p) => format!(
                    "{} petitions the gathering at the hearth on the evening of {day} for {}",
                    pop.factions
                        .get(p.faction)
                        .map_or_else(|| "a faction".to_owned(), |f| faction_name(sim, f)),
                    petition_demand_words(sim, p)
                ),
                None => format!("a faction petitions the gathering on the evening of {day}"),
            }
        }
        ClaimKind::Refusal => {
            let until = day_words(SimTime::from_minutes(c.day * DAY));
            match c
                .subject
                .and_then(|id| pop.factions.refusals.iter().find(|r| r.id == id))
            {
                Some(r) => format!(
                    "{} keeps back the levy of {} until {until}",
                    pop.factions
                        .get(r.faction)
                        .map_or_else(|| "a faction".to_owned(), |f| faction_name(sim, f)),
                    refusal_law_words(sim, r)
                ),
                None => format!("a faction keeps back the levy until {until}"),
            }
        }
        ClaimKind::Revolt => {
            let until = day_words(SimTime::from_minutes(c.day * DAY));
            match c
                .subject
                .and_then(|id| pop.factions.revolts.iter().find(|r| r.id == id))
            {
                Some(r) => format!(
                    "{} calls on everyone to stand with it until {until}: from now on, {}, in place \
                     of the gathering's custom",
                    pop.factions
                        .get(r.faction)
                        .map_or_else(|| "a faction".to_owned(), |f| faction_name(sim, f)),
                    r.body.clause()
                ),
                None => format!("a faction calls on everyone to stand with it until {until}"),
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
        ClaimKind::Suspicion => {
            let who = c.subject.map_or_else(|| "someone".to_owned(), name_of);
            let households = |n: u16| match n {
                1 => "1 household".to_owned(),
                n => format!("{n} households"),
            };
            match c.suspected {
                Some((place, n)) => format!(
                    "{who} holds that the water at {} sickens: of {} they know that draw there, {} \
                     had sickness lately, against {} of {} that draw elsewhere",
                    super::sickness::place_words(sim, place),
                    households(n.at),
                    n.sick_at,
                    n.sick_elsewhere,
                    n.elsewhere
                ),
                None => format!("{who} holds that a source of water sickens"),
            }
        }
        ClaimKind::Fire => {
            let on = day_words(SimTime::from_minutes(c.day * DAY));
            let at = c.fire.and_then(|id| sim.people.fires.get(id)).map_or_else(
                || "a building".to_owned(),
                |f| {
                    format!(
                        "a building of {}",
                        super::people::household_name(sim, f.household)
                    )
                },
            );
            format!("{at} was found burning on {on}; this account names no cause")
        }
        ClaimKind::Raid => {
            let on = day_words(SimTime::from_minutes(c.day * DAY));
            let of = c.subject.map_or_else(
                || "a household".to_owned(),
                |h| super::people::household_name(sim, h),
            );
            let clash = c
                .event
                .and_then(|id| pop.order.clashes.iter().find(|x| x.id == id));
            let band = clash.and_then(|x| pop.order.bands.iter().find(|b| b.id == x.band));
            match (clash, band) {
                (Some(x), Some(b)) => {
                    let came = x
                        .fighters
                        .iter()
                        .filter(|f| f.side == civ_agents::war::Side::Band)
                        .count();
                    let struck = x.blows.len();
                    let died = x.blows.iter().filter(|s| s.harm.killed).count();
                    let mut said = format!(
                        "{came} of {} came in a band to {of} on {on}",
                        place_name(sim, b.settlement)
                    );
                    if struck > 0 {
                        said.push_str(&format!(
                            "; {struck} {} struck",
                            if struck == 1 {
                                "blow was"
                            } else {
                                "blows were"
                            }
                        ));
                    }
                    if died > 0 {
                        said.push_str(&format!(
                            ", and {died} died of {}",
                            if struck == 1 { "it" } else { "them" }
                        ));
                    }
                    if b.taken_kcal > 0.0 {
                        let kg: f32 = b.taken.iter().map(|t| t.1).sum();
                        said.push_str(&format!("; they took {kg:.0} kg"));
                    } else {
                        said.push_str("; they took nothing");
                    }
                    said
                }
                _ => format!("people of another settlement came in a band to {of} on {on}"),
            }
        }
        ClaimKind::BandSeen => {
            let on = day_words(SimTime::from_minutes(c.day * DAY));
            match c
                .event
                .and_then(|id| pop.order.bands.iter().find(|b| b.id == id))
            {
                Some(b) => {
                    let toward = pop
                        .household(b.target)
                        .and_then(|x| x.settlement)
                        .map_or_else(String::new, |s| format!(", toward {}", place_name(sim, s)));
                    format!(
                        "a band of {} of {} was seen walking on {on}{toward}",
                        b.mustered.len(),
                        place_name(sim, b.settlement)
                    )
                }
                None => format!("a band was seen walking on {on}"),
            }
        }
        ClaimKind::Sickness => {
            let from = day_words(SimTime::from_minutes(c.day * DAY));
            match c.subject {
                Some(h) => format!(
                    "someone of {} lay sick from {from}",
                    super::people::household_name(sim, h)
                ),
                None => format!("someone lay sick from {from}"),
            }
        }
    }
}

/// A settlement's name, or "another settlement".
fn place_name(sim: &Sim, id: PermanentId) -> String {
    sim.land
        .settlements
        .iter()
        .find(|s| s.id == id)
        .map_or_else(|| "another settlement".to_owned(), |s| s.name.clone())
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

/// What `p` holds of each norm content names (wire 1.37, ADR-0016 §4).
pub fn norm_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::NormLine<'a>>>> {
    use civ_agents::norm::{NormKind, activation, endorse_words, expect_words};
    let norms = &sim.rules.catalog.norms;
    let lines: Vec<_> = sim
        .people
        .norms
        .held_by(p.id)
        .iter()
        .filter_map(|s| Some((s, norms.get(usize::from(s.norm))?)))
        .map(|(s, def)| {
            let (e, x, t) = (
                f64::from(s.endorse),
                f64::from(s.expect),
                f64::from(s.threshold),
            );
            let does = match def.kind {
                NormKind::AbideByLaws => "pay what the gathering asks",
                NormKind::AbideByFindings => "meet what a finding imposes",
            };
            let statement = fbb.create_string(&def.statement);
            let holds = fbb.create_string(endorse_words(e));
            let believes = fbb.create_string(&format!("{} {does}", expect_words(x)));
            wire::NormLine::create(
                fbb,
                &wire::NormLineArgs {
                    statement: Some(statement),
                    holds: Some(holds),
                    believes: Some(believes),
                    endorse: s.endorse,
                    expect: s.expect,
                    threshold: s.threshold,
                    activation: activation(x, t, def.width) as f32,
                    heard: s.heard,
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// What `p` holds of each value content names (wire 1.38, ADR-0016 §4).
pub fn value_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::ValueLine<'a>>>> {
    let defs = &sim.rules.catalog.values;
    let lines: Vec<_> = sim
        .people
        .values
        .held_by(p.id)
        .iter()
        .filter_map(|h| Some((h, defs.get(usize::from(h.value))?)))
        .map(|(h, def)| {
            let name = fbb.create_string(&def.name);
            let words = fbb.create_string(&civ_agents::values::words(f64::from(h.v), def));
            wire::ValueLine::create(
                fbb,
                &wire::ValueLineArgs {
                    name: Some(name),
                    words: Some(words),
                    v: h.v,
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// The ideologies `p` holds (wire 1.39, ADR-0016 §4).
pub fn ideology_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::IdeologyLine<'a>>>> {
    let defs = &sim.rules.catalog.ideologies;
    let lines: Vec<_> = sim
        .people
        .ideologies
        .held_by(p.id)
        .iter()
        .filter_map(|h| Some((h, defs.get(usize::from(h.ideology))?)))
        .map(|(h, def)| {
            let name = fbb.create_string(&def.name);
            let legitimacy = fbb.create_string(&def.legitimacy);
            let from_name = h.from.map(|f| fbb.create_string(&sim.people.name_of(f)));
            wire::IdeologyLine::create(
                fbb,
                &wire::IdeologyLineArgs {
                    name: Some(name),
                    legitimacy: Some(legitimacy),
                    since_minute: h.since * DAY,
                    from: h.from.map_or(0, PermanentId::get),
                    from_name,
                    influence: sim
                        .people
                        .influences
                        .on(p.id)
                        .find(|i| {
                            i.kind == InfluenceKind::Ideology
                                && i.subject == u32::from(h.ideology)
                                && i.taken.is_some()
                        })
                        .map_or(0, |i| i.id),
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// A faction's name, after its founder: "Mira's faction"; an organization by stake's, "Mira's
/// organization".
pub fn faction_name(sim: &Sim, f: &civ_agents::faction::Faction) -> String {
    let word = if f.stake.is_some() {
        "organization"
    } else if is_party(sim, f) {
        "party"
    } else {
        "faction"
    };
    format!("{}'s {word}", sim.people.name_of(f.founder))
}

/// What an organization by stake is of and its rules, in words (M7c slice BW): "those who sell
/// tools; it admits one a member speaks for, its organizer decides, and its dues are a twentieth
/// of each threshing".
pub fn stake_words(st: &civ_agents::faction::Stake) -> String {
    format!(
        "those who sell {}; it admits {}, {} decides, and its dues are {} of each threshing",
        st.purpose.plural_words(),
        st.admit.words(),
        st.decides.words(),
        civ_agents::polity::share_text(f64::from(st.dues_share)),
    )
}

/// Which polities recognize organization `org` by a law in force (M7c slice BW), after its
/// stake: "; Oakholt's gathering recognizes it", or nothing.
pub fn recognized_words(sim: &Sim, org: PermanentId) -> String {
    let by: Vec<String> = sim
        .people
        .polities
        .iter()
        .filter(|p| {
            p.laws.iter().any(|l| {
                l.kind == civ_agents::polity::PolicyKind::Recognize
                    && l.status == civ_agents::polity::LawStatus::InForce
                    && l.target == Some(org)
            })
        })
        .map(|p| format!("{}'s gathering", place_name(sim, p.settlement)))
        .collect();
    if by.is_empty() {
        String::new()
    } else {
        format!("; {} recognizes it", by.join(" and "))
    }
}

/// Whether the observer calls faction `f` a party (ADR-0024 §5, M7a slice BQ): it has backed a
/// slate in two elections or more. Derived here, never saved, and nothing in the world reads it.
pub fn is_party(sim: &Sim, f: &civ_agents::faction::Faction) -> bool {
    sim.people
        .polities
        .iter()
        .flat_map(|p| &p.elections)
        .filter(|e| e.endorsements.iter().any(|x| x.0 == f.id))
        .count()
        >= 2
}

/// The law whose levy refusal `r` keeps back, in words (M4c slice AH): "a common store, taking a
/// tenth of each harvest".
pub fn refusal_law_words(sim: &Sim, r: &civ_agents::faction::Refusal) -> String {
    let pop = &sim.people;
    pop.polities
        .iter()
        .flat_map(|p| &p.laws)
        .find(|l| l.id == r.law)
        .map_or_else(
            || "the common store".to_owned(),
            |l| law_words(l, &sim.rules.catalog.policies, &|id| pop.name_of(id)),
        )
}

/// What petition `p` asks for, in words (M4c slice AH): "an end to the common store's levy (the
/// store gives what it holds)"; "Ada as keeper of the common store in place of Bo".
pub fn petition_demand_words(sim: &Sim, p: &civ_agents::faction::Petition) -> String {
    let pop = &sim.people;
    let def = sim.rules.catalog.policies.get(usize::from(p.policy));
    let Some(nominee) = p.nominee else {
        return civ_agents::polity::replacing_words(def, p.levy_share);
    };
    let held = pop
        .polities
        .iter()
        .flat_map(|q| &q.laws)
        .find(|l| l.id == p.ends)
        .and_then(|l| l.holder)
        .map_or_else(|| "the one holding it".to_owned(), |h| pop.name_of(h));
    let office = match def.map(|d| d.kind) {
        Some(civ_agents::polity::PolicyKind::KeepWatch) => "to keep watch over the stores at night",
        _ => "as keeper of the common store",
    };
    format!("{} {office} in place of {held}", pop.name_of(nominee))
}

/// The faction `p` belongs to, if any, and why (wire 1.40, ADR-0017 §2, §6).
pub fn faction_line<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> Option<WIPOffset<wire::FactionLine<'a>>> {
    let pop = &sim.people;
    let m = pop.factions.membership(p.id)?;
    let f = pop.factions.get(m.faction)?;
    let organizer = pop.name_of(f.organizer);
    let (party, why) = match f.stake {
        // An organization by stake is against nobody: it is of those who sell its goods.
        Some(st) => (
            format!("{}{}", stake_words(&st), recognized_words(sim, f.id)),
            civ_agents::faction::stake_why_words(
                &m.why,
                st.purpose,
                &organizer,
                f.organizer == p.id,
            ),
        ),
        None => {
            let party = blamed_words(sim, f.against);
            let why =
                civ_agents::faction::why_words(&m.why, &party, &organizer, f.organizer == p.id);
            (party, why)
        }
    };
    let name = fbb.create_string(&faction_name(sim, f));
    let against = fbb.create_string(&party);
    let organizer_name = fbb.create_string(&organizer);
    let why = fbb.create_string(&why);
    Some(wire::FactionLine::create(
        fbb,
        &wire::FactionLineArgs {
            faction: f.id.get(),
            name: Some(name),
            against: Some(against),
            organizer: f.organizer.get(),
            organizer_name: Some(organizer_name),
            since_minute: m.since * DAY,
            why: Some(why),
        },
    ))
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
                    influence: sim
                        .people
                        .influences
                        .find(InfluenceKind::Whisper, p.id, h.claim)
                        .map_or(0, |i| sim.people.influences.list[i].id),
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// The welcome's ideologies, which the observer may tell someone of (wire 1.47, M4c slice AJ).
pub fn ideology_infos<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    catalog: &Catalog,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::IdeologyInfo<'a>>>> {
    let list: Vec<_> = catalog
        .ideologies
        .iter()
        .map(|d| {
            let id = fbb.create_string(&d.id);
            let mut name = d.name.clone();
            if let Some(first) = name.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            let name = fbb.create_string(&name);
            let legitimacy = fbb.create_string(&d.legitimacy);
            wire::IdeologyInfo::create(
                fbb,
                &wire::IdeologyInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    legitimacy: Some(legitimacy),
                },
            )
        })
        .collect();
    fbb.create_vector(&list)
}

/// The true claims `p`'s settlement's word holds that they have not heard and the observer may
/// whisper to them (wire 1.47, M4c slice AJ, ADR-0016 §5), newest first: calls still to come, and
/// grievances someone else holds and has told.
pub fn news_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::NewsLine<'a>>>> {
    let pop = &sim.people;
    let today = sim.now().day_index();
    let settlement = pop.household(p.household).and_then(|x| x.settlement);
    let grown = p.age_years(sim.now()) >= sim.rules.people.family.independent_age;
    let lines: Vec<_> = pop
        .word
        .claims
        .iter()
        .rev()
        .filter(|_| grown)
        .filter(|c| Some(c.settlement) == settlement)
        .filter(|c| !pop.word.has_heard(p.id, c.id))
        .filter(|c| match c.kind {
            ClaimKind::Grievance => c.subject.is_some_and(|s| s != p.id),
            // Another household's sickness, or another's suspicion, while it is news (M6a slice BA).
            ClaimKind::Sickness => c.subject != Some(p.household),
            ClaimKind::Suspicion => c.subject != Some(p.id),
            ClaimKind::Fire | ClaimKind::Raid | ClaimKind::BandSeen => true,
            _ => c.day >= today,
        })
        .take(MAX_NEWS_SHOWN)
        .map(|c| {
            let what = fbb.create_string(&claim_words(sim, c));
            wire::NewsLine::create(
                fbb,
                &wire::NewsLineArgs {
                    claim: c.id,
                    what: Some(what),
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// "no one", "1 other", "3 others".
fn others(n: usize) -> String {
    match n {
        0 => "no one".to_owned(),
        1 => "1 other".to_owned(),
        n => format!("{n} others"),
    }
}

/// What came of whisper `i` to `p`, read from the records people keep (15-05 §6: perceived,
/// acted on, passed on; never what it was meant to do).
fn whisper_words(sim: &Sim, i: &Influence, p: PermanentId) -> String {
    let pop = &sim.people;
    let day = |d: i64| day_words(SimTime::from_minutes(d * DAY));
    let times = if i.uses > 1 {
        format!(" ({} times)", i.uses)
    } else {
        String::new()
    };
    let Some(c) = pop.word.claim(i.subject) else {
        return format!("Whispered news{times} that is no longer told: it was heard");
    };
    let mut said = vec![format!("Whispered that {}{times}", claim_words(sim, c))];
    match pop.word.heard_by(p).iter().find(|h| h.claim == c.id) {
        Some(h) => said.push(format!("heard on {}", day(h.first))),
        None => said.push("heard, and let go as news is".to_owned()),
    }
    let told = pop
        .word
        .heard
        .iter()
        .filter(|h| h.claim == c.id && h.from == Some(p))
        .count();
    said.push(format!("told {} since", others(told)));
    let since = i.at.day_index();
    let acted = match c.kind {
        // Whether they came to poll, once the polling day is past (M7a slice BO).
        ClaimKind::Election => pop
            .polities
            .iter()
            .find(|q| q.settlement == c.settlement)
            .and_then(|q| q.elections.iter().rev().find(|e| e.poll_day == c.day))
            .filter(|e| e.poll_day < sim.now().day_index())
            .map(|e| {
                if e.polled(p) {
                    "came to poll".to_owned()
                } else if e.stood(p) {
                    "stood".to_owned()
                } else {
                    "did not come to poll".to_owned()
                }
            }),
        // Whether they voted for the one who stood, once the polling day is past (M7a slice BP).
        ClaimKind::Candidacy => pop
            .polities
            .iter()
            .find(|q| q.settlement == c.settlement)
            .and_then(|q| q.elections.iter().rev().find(|e| e.poll_day == c.day))
            .filter(|e| e.poll_day < sim.now().day_index())
            .zip(c.subject)
            .map(|(e, who)| {
                if who == p {
                    "stood".to_owned()
                } else if e
                    .ballots
                    .iter()
                    .any(|(v, named)| *v == p && named.contains(&who))
                {
                    "named them".to_owned()
                } else if e.polled(p) {
                    "did not name them".to_owned()
                } else {
                    "did not come to poll".to_owned()
                }
            }),
        ClaimKind::Gathering => c.subject.and_then(|law| {
            let l = pop
                .polities
                .iter()
                .flat_map(|q| &q.laws)
                .find(|l| l.id == law)?;
            l.decided?;
            Some(if l.stances.iter().any(|s| s.person == p) {
                "came to the gathering".to_owned()
            } else {
                "did not come to the gathering".to_owned()
            })
        }),
        ClaimKind::Petition => c.subject.and_then(|id| {
            let q = pop.factions.petitions.iter().find(|q| q.id == id)?;
            (q.day < sim.now().day_index() || q.answered).then(|| {
                if q.came.contains(&p) {
                    "came to the petition".to_owned()
                } else {
                    "did not come to the petition".to_owned()
                }
            })
        }),
        ClaimKind::Refusal => c.subject.and_then(|id| {
            let r = pop.factions.refusals.iter().find(|r| r.id == id)?;
            let household = pop.person(p).map(|q| q.household)?;
            r.kept
                .contains(&household)
                .then(|| "their household kept back its levy with it".to_owned())
        }),
        ClaimKind::Revolt => c.subject.and_then(|id| {
            let r = pop.factions.revolts.iter().find(|r| r.id == id)?;
            r.sides.iter().find(|s| s.0 == p).map(|s| {
                match s.1 {
                    civ_agents::faction::Side::With => "stood with it",
                    civ_agents::faction::Side::Gathering => "stood with the gathering",
                    civ_agents::faction::Side::Neither => "stood with neither",
                }
                .to_owned()
            })
        }),
        // What people do of sickness they heard of is slice BA's later steps; a finding remembered
        // is recalled at a hearing (M7b slice BS).
        ClaimKind::Finding
        | ClaimKind::Sickness
        | ClaimKind::Suspicion
        | ClaimKind::Fire
        | ClaimKind::Raid
        | ClaimKind::BandSeen => None,
        ClaimKind::Grievance => {
            let against = c.grievance.map(|g| g.0);
            let m = pop.factions.membership(p).copied();
            m.and_then(|m| {
                let f = pop.factions.get(m.faction)?;
                (Some(f.against) == against && m.since >= since).then(|| {
                    if f.founder == p {
                        format!("founded {}", faction_name(sim, f))
                    } else {
                        format!("joined {}", faction_name(sim, f))
                    }
                })
            })
        }
    };
    if let Some(a) = acted {
        said.push(a);
    }
    said.join("; ")
}

/// What came of telling `p` of an ideology in `i`.
fn ideology_told_words(sim: &Sim, i: &Influence, p: PermanentId) -> String {
    let pop = &sim.people;
    let catalog = &sim.rules.catalog;
    let Some(k) = u16::try_from(i.subject).ok() else {
        return "Told of an ideology".to_owned();
    };
    let Some(def) = catalog.ideologies.get(usize::from(k)) else {
        return "Told of an ideology the content no longer names".to_owned();
    };
    let day = |d: i64| day_words(SimTime::from_minutes(d * DAY));
    let times = if i.uses > 1 {
        format!(" ({} times)", i.uses)
    } else {
        String::new()
    };
    let mut said = vec![format!("Told of {}{times}", def.name.to_lowercase())];
    said.push(match i.weighed {
        1 => "weighed it once".to_owned(),
        n => format!("weighed it {n} times"),
    });
    match i.taken {
        Some(d) => said.push(format!("took it up on {}", day(d))),
        None if pop.ideologies.holds(p, k) => said.push("holds it, from another".to_owned()),
        None => {
            let (fit, chance) = sim.people.ideology_chance(catalog, p, k);
            said.push(format!(
                "has not taken it up: it fits what they hold dear by {fit:+.2}, a {:.0}\u{a0}% \
                 chance",
                100.0 * chance
            ));
        }
    }
    let taught = pop
        .ideologies
        .held
        .iter()
        .filter(|h| h.ideology == k && h.from == Some(p))
        .count();
    if i.taken.is_some() || pop.ideologies.holds(p, k) {
        said.push(format!("{} took it up from them", others(taught)));
    }
    said.join("; ")
}

/// What came of sending `p` as an agitator in `i`: whom they reached with what they hold.
fn agitator_words(sim: &Sim, i: &Influence, p: PermanentId) -> String {
    let pop = &sim.people;
    let name = u16::try_from(i.subject).ok().and_then(|k| {
        sim.rules
            .catalog
            .ideologies
            .get(usize::from(k))
            .map(|d| (k, d))
    });
    let Some((k, def)) = name else {
        return "Sent here by the observer".to_owned();
    };
    let taught = pop
        .ideologies
        .held
        .iter()
        .filter(|h| h.ideology == k && h.from == Some(p))
        .count();
    let holds = if pop.ideologies.holds(p, k) {
        "holds to it still"
    } else {
        "no longer holds to it"
    };
    let known = pop
        .ties
        .of(p)
        .iter()
        .filter(|t| t.known_at(sim.now().day_index(), &sim.rules.people.ties) > 0.0)
        .count();
    format!(
        "Sent here by the observer holding to {}; {holds}; {} took it up from them; knows {} \
         {} here now",
        def.name.to_lowercase(),
        others(taught),
        known,
        if known == 1 { "person" } else { "people" }
    )
}

/// What the wave of `i` that brought them was, and what came of it so far: how many of its
/// households came, how many of the people it brought still live, and where (M5a slice AN).
fn wave_words(sim: &Sim, i: &Influence) -> String {
    let pop = &sim.people;
    let Some(w) = pop.influences.wave(i.id) else {
        return "Came with a wave the observer sent".to_owned();
    };
    let name = |s: PermanentId| {
        sim.land
            .settlements
            .iter()
            .find(|x| x.id == s)
            .map_or_else(|| "a settlement".to_owned(), |x| x.name.clone())
    };
    let came = if w.done() {
        format!(
            "{} of its {} households came over {} {}",
            w.came,
            w.households,
            w.days,
            if w.days == 1 { "day" } else { "days" }
        )
    } else {
        format!(
            "{} of its {} households have come; the rest come over {} {} in all",
            w.came,
            w.households,
            w.days,
            if w.days == 1 { "day" } else { "days" }
        )
    };
    let months = w.provisions_days as f64 * 12.0 / civ_core::time::DAYS_PER_YEAR as f64;
    let mut living: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for &q in &w.people {
        if let Some(s) = pop
            .person(q)
            .and_then(|p| pop.household(p.household))
            .and_then(|h| h.settlement)
        {
            *living.entry(name(s)).or_default() += 1;
        }
    }
    let now = if living.is_empty() {
        "none of its people lives on the map now".to_owned()
    } else {
        let at: Vec<String> = living.iter().map(|(s, n)| format!("{n} in {s}")).collect();
        format!(
            "of the {} people it brought, {} live now",
            w.people.len(),
            at.join(", ")
        )
    };
    format!("Came with a wave the observer sent, carrying {months:.0} months' food: {came}; {now}")
}

/// What a drought in `i` held back (M6a slice BA, step five).
fn drought_words(i: &Influence) -> String {
    format!(
        "Held back the rain for {} months, until {}",
        i.subject,
        day_words(SimTime::from_minutes(i.until * DAY))
    )
}

/// What the observer's fire in `i` came to (M6b slice BC): the kernel's record of it.
fn fire_words(sim: &Sim, i: &Influence) -> String {
    let fires = &sim.people.fires;
    let Some(f) = fires.get(i.subject) else {
        return "Set a building alight".to_owned();
    };
    let spread = f
        .episode
        .and_then(|e| fires.episodes.get(e as usize - 1))
        .map(|e| {
            fires
                .list
                .iter()
                .filter(|x| x.episode == Some(e.id))
                .count()
        });
    match (f.ended, spread) {
        (None, _) => "Set a building alight: it burns".to_owned(),
        (Some(_), Some(n)) => format!("Set a building alight: the fire spread to {} more", n - 1),
        (Some(_), None) => "Set a building alight: it burned out alone".to_owned(),
    }
}

/// What a plague in `i` brought and what became of it (M6a slice AZ): the observer's own record,
/// so it tells the truth of their infection.
fn plague_words(sim: &Sim, i: &Influence) -> String {
    use civ_agents::sickness::{Acquired, Outcome};
    let day = |d: i64| day_words(SimTime::from_minutes(d * DAY));
    let name = sim
        .rules
        .catalog
        .diseases
        .get(i.subject as usize)
        .map_or_else(|| "a disease".to_owned(), |d| d.name.to_lowercase());
    let lead = format!(
        "Brought {name} {}, as if they took it elsewhere",
        day_words(i.at)
    );
    let Some(e) = sim
        .people
        .sickness
        .episodes()
        .iter()
        .find(|e| e.acquired == (Acquired::Observer { influence: i.id }))
    else {
        return format!("{lead}.");
    };
    let course = if e.symptomatic() {
        let how = if e.course.severe {
            "severely ill"
        } else {
            "ill"
        };
        format!(
            "{how} from {} to {}",
            day(e.course.ill_from),
            day(e.course.ill_until)
        )
    } else {
        "it brought no symptoms".to_owned()
    };
    let end = match e.ended {
        None => "it runs".to_owned(),
        Some((_, Outcome::Recovered)) => format!(
            "they recovered, protected until {}",
            day(e.course.immune_until)
        ),
        Some((d, Outcome::Died)) => format!("it killed them {}", day(d)),
        Some((d, Outcome::Gone)) => format!("they were gone {} while it ran", day(d)),
    };
    format!("{lead}: {course}; {end}.")
}

/// What a blessing or a curse in `i` is and what it turned.
fn luck_words(sim: &Sim, i: &Influence) -> String {
    let day = |d: i64| day_words(SimTime::from_minutes(d * DAY));
    let bless = i.kind == InfluenceKind::Bless;
    let luck = civ_agents::influence::Luck {
        index: 0,
        bless,
        share: f64::from(i.share),
    };
    let what = if bless { "Blessed" } else { "Cursed" };
    let times = if i.uses > 1 {
        format!(" ({} times)", i.uses)
    } else {
        String::new()
    };
    let when = if sim.now().day_index() < i.until {
        format!("until {}", day(i.until))
    } else {
        format!("to {}", day(i.until))
    };
    let turned = |n: u32, one: &str, many: &str| match n {
        0 => None,
        1 => Some(one.to_owned()),
        n => Some(format!("{n} {many}")),
    };
    let mut said = vec![format!(
        "{what}{times} {when}: of their own chances, illness or accident at {:.2} of what it was, \
         finding things out at {:.2}",
        luck.harm(),
        luck.fortune()
    )];
    let deaths = if bless {
        turned(i.deaths, "a death it spared them", "deaths it spared them")
    } else {
        turned(i.deaths, "a death it brought", "deaths it brought")
    };
    let finds = if bless {
        turned(i.finds, "a find it brought", "finds it brought")
    } else {
        turned(i.finds, "a find it cost them", "finds it cost them")
    };
    match (deaths, finds) {
        (None, None) => said.push("no draw of theirs has gone otherwise for it".to_owned()),
        (d, f) => said.extend(d.into_iter().chain(f)),
    }
    said.join("; ")
}

/// The observer's interventions that reached `p`, newest first, with what came of each (wire
/// 1.47, M4c slice AJ, ADR-0016 §5).
pub fn influence_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::InfluenceLine<'a>>>> {
    let mut on: Vec<&Influence> = sim.people.influences.on(p.id).collect();
    on.reverse();
    let lines: Vec<_> = on
        .into_iter()
        .map(|i| {
            let words = match i.kind {
                InfluenceKind::Whisper => whisper_words(sim, i, p.id),
                InfluenceKind::Ideology => ideology_told_words(sim, i, p.id),
                InfluenceKind::Agitator => agitator_words(sim, i, p.id),
                InfluenceKind::Bless | InfluenceKind::Curse => luck_words(sim, i),
                InfluenceKind::Wave => wave_words(sim, i),
                InfluenceKind::Plague => plague_words(sim, i),
                InfluenceKind::Drought => drought_words(i),
                InfluenceKind::Fire => fire_words(sim, i),
                InfluenceKind::Flood => "the observer raised the river's runoff".to_owned(),
                InfluenceKind::Storm => "the observer sent a storm".to_owned(),
            };
            let what = fbb.create_string(&words);
            wire::InfluenceLine::create(
                fbb,
                &wire::InfluenceLineArgs {
                    id: i.id,
                    kind: i.kind.code(),
                    minute: i.at.minutes(),
                    what: Some(what),
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}
