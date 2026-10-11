//! What the observer is shown of sickness (wire 1.62, M6a slice AZ, step four; ADR-0021 §5, §8):
//! the diseases they may bring to someone, a person's infections in words, and what fouls a
//! well's water. All of it is the kernel's truth, which no one in the world knows.

use civ_agents::contagion::Node;
use civ_agents::params::{Catalog, DiseaseRoute};
use civ_agents::sickness::{Acquired, Episode, Outcome};
use civ_core::PermanentId;
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use crate::Sim;

/// The diseases the observer may bring to someone, in catalog order (`Welcome.diseases`).
pub fn disease_infos<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    catalog: &Catalog,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::DiseaseInfo<'a>>>> {
    let list: Vec<_> = catalog
        .diseases
        .iter()
        .map(|d| {
            let routes: Vec<&str> = d
                .routes
                .iter()
                .map(|r| match r {
                    DiseaseRoute::Water => "by water",
                    DiseaseRoute::Household => "between those who share a home",
                })
                .collect();
            let id = fbb.create_string(&d.id);
            let name = fbb.create_string(&d.name);
            let routes = fbb.create_string(&routes.join(" and "));
            wire::DiseaseInfo::create(
                fbb,
                &wire::DiseaseInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    routes: Some(routes),
                },
            )
        })
        .collect();
    fbb.create_vector(&list)
}

/// Day `day` against today, in words: "today", "yesterday", "3 days ago", "tomorrow", "in 3
/// days".
fn when(today: i64, day: i64) -> String {
    match today - day {
        0 => "today".to_owned(),
        1 => "yesterday".to_owned(),
        -1 => "tomorrow".to_owned(),
        n if n > 0 => format!("{n} days ago"),
        n => format!("in {} days", -n),
    }
}

/// Where water was drawn, in words: "Bo's household's well", "the river".
fn source_words(sim: &Sim, source: Node) -> String {
    match source {
        Node::Well(w) => sim
            .land
            .wells
            .get(w)
            .and_then(|w| {
                sim.people
                    .household(w.household)
                    .and_then(|x| x.members.first().copied())
            })
            .map_or_else(
                || "a well".to_owned(),
                |m| format!("{}'s household's well", sim.people.name_of(m)),
            ),
        Node::Reach(_) => "the river".to_owned(),
        _ => "their store".to_owned(),
    }
}

/// One infection in words (see `PersonInfo.sickness`).
fn episode_words(sim: &Sim, e: &Episode) -> String {
    let today = sim.now().day_index();
    let name = sim
        .rules
        .catalog
        .diseases
        .get(usize::from(e.disease))
        .map_or_else(|| "a disease".to_owned(), |d| d.name.to_lowercase());
    let c = &e.course;
    let mut parts = Vec::new();
    parts.push(match e.ended {
        Some((day, Outcome::Died)) => format!("Died of {name} {}", when(today, day)),
        Some((_, Outcome::Gone)) => format!("Had {name} when they died or left"),
        Some((_, Outcome::Recovered)) => {
            let ill = if e.symptomatic() {
                format!("ill {} days", c.ill_until - c.ill_from)
            } else {
                "without falling ill".to_owned()
            };
            let protected = if c.immune_until > today {
                format!(", protected for {} days more", c.immune_until - today)
            } else {
                ", protected no longer".to_owned()
            };
            format!("Had {name} {}, {ill}{protected}", when(today, e.infected))
        }
        None if e.ill_on(today) => {
            let severe = if c.severe { ", severely" } else { "" };
            format!(
                "Ill with {name} since {}{severe}, day {} of {}",
                when(today, c.ill_from),
                today - c.ill_from + 1,
                c.ill_until - c.ill_from
            )
        }
        None if e.symptomatic() && today < c.ill_from => format!(
            "Took {name} {}; ill from {}",
            when(today, e.infected),
            when(today, c.ill_from)
        ),
        None => {
            let shed = if e.shedding(today) {
                ", shedding it"
            } else {
                ""
            };
            if e.symptomatic() {
                format!("Over {name}'s illness{shed}")
            } else {
                format!(
                    "Took {name} {} and is not ill{shed}",
                    when(today, e.infected)
                )
            }
        }
    });
    parts.push(match e.acquired {
        Acquired::Observer { .. } => {
            "brought by the observer, as if they took it elsewhere".to_owned()
        }
        Acquired::Household { .. } => "took it from those who shed it at home".to_owned(),
        Acquired::Water { source } => {
            format!("took it in water drawn at {}", source_words(sim, source))
        }
    });
    if let Some((first, by)) = e.care.first {
        let mut care = format!(
            "tended {:.0} hours in all, first by {} {}",
            e.care.total_h,
            sim.people.name_of(by),
            when(today, first)
        );
        if e.care.day == today && e.care.hours > 0.0 {
            care.push_str(&format!(", {:.0} today", e.care.hours));
        }
        if e.care.rr < 1.0 {
            care.push_str(", by one who knows a treatment");
        }
        parts.push(care);
    }
    if let Some(o) = e
        .outbreak
        .and_then(|id| sim.people.sickness.outbreaks().iter().find(|o| o.id == id))
    {
        let cases = sim.people.sickness.cases(o.id).count();
        let place = o.settlement.map_or_else(
            || "among people of no settlement".to_owned(),
            |s| {
                sim.land
                    .settlements
                    .iter()
                    .find(|x| x.id == s)
                    .map_or_else(|| "a settlement".to_owned(), |x| format!("at {}", x.name))
            },
        );
        parts.push(match (cases, o.ended.is_some()) {
            (1, false) => format!("the only case so far of an outbreak {place}"),
            (1, true) => format!("the only case of an outbreak {place}, now over"),
            (n, false) => format!("one of {n} cases so far of the outbreak {place}"),
            (n, true) => format!("one of {n} cases of the outbreak {place}, now over"),
        });
    }
    parts.join("; ")
}

/// `person`'s infections in words, the latest first, at most five.
pub fn sickness_words(sim: &Sim, person: PermanentId) -> Vec<String> {
    let mut list: Vec<&Episode> = sim.people.sickness.of(person).collect();
    list.reverse();
    list.into_iter()
        .take(5)
        .map(|e| episode_words(sim, e))
        .collect()
}

/// What fouls the water of well `well`, holding `litres`, in words (`WellInfo.fouled`): "Its
/// water holds cholera: about 120 doses, a dose in every 14 L"; empty when nothing does.
pub fn fouled_words(sim: &Sim, well: PermanentId, litres: f64) -> String {
    let held: Vec<String> = sim
        .people
        .contagion
        .loads
        .range((Node::Well(well), 0)..=(Node::Well(well), u16::MAX))
        .filter(|&(_, &l)| l >= 0.5)
        .map(|(&(_, d), &l)| {
            let name = sim
                .rules
                .catalog
                .diseases
                .get(usize::from(d))
                .map_or_else(|| "a disease".to_owned(), |x| x.name.to_lowercase());
            if litres > 0.0 {
                format!(
                    "{name}: about {l:.0} doses, a dose in every {:.0} L",
                    (litres / l).max(1.0)
                )
            } else {
                format!("{name}: about {l:.0} doses")
            }
        })
        .collect();
    if held.is_empty() {
        String::new()
    } else {
        format!("Its water holds {}", held.join("; "))
    }
}
