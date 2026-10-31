//! People on the boundary (ADR-0003 §2–3): the snapshot's per-person briefs and settlements, and
//! the responses to trip, person and chronicle queries.
//!
//! Every piece of text here is rendered by the kernel: what someone is doing, where a target is,
//! the chronicle. Observers show it; they never compose explanations of their own.

use civ_agents::params::Taste;
use std::collections::HashSet;

use civ_agents::history::{self, Span};
use civ_agents::params::GoodUse;
use civ_agents::person::{food_kcal, fuel_kg, stock_kcal};
use civ_agents::{Origin, Person, Receipt, Repro, Scored, Sex, Step, Target, population};
use civ_core::{PermanentId, SimTime};
use civ_land::Building;
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use super::{QueryError, response};
use crate::Sim;

/// Most trips one query may ask for.
pub const MAX_TRIPS_PER_QUERY: usize = 256;
/// Most chronicle entries one response carries.
pub const MAX_CHRONICLE_PER_QUERY: u32 = 256;
/// Most decision receipts one person response carries.
pub const MAX_DECISIONS_PER_QUERY: u32 = 64;

type Offsets<'a, T> = WIPOffset<Vector<'a, ForwardsUOffset<T>>>;

fn vec2((x, y): (f32, f32)) -> wire::Vec2 {
    wire::Vec2::new(x, y)
}

fn sex(s: Sex) -> wire::Sex {
    match s {
        Sex::Female => wire::Sex::Female,
        Sex::Male => wire::Sex::Male,
    }
}

/// The snapshot's entry for every living person, in permanent-id order, placed where they are at
/// the current minute. In Accelerated mode nobody is shown on a trip: the next frame is a day on,
/// so a walker is not to be carried along (ADR-0011 §6).
pub fn person_briefs<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
) -> Offsets<'a, wire::PersonBrief<'a>> {
    let now = sim.now();
    let t = now.minutes() as f64;
    let trips = sim.mode() == crate::Mode::Detailed;
    let mut people: Vec<&Person> = sim.people.people.iter().map(|(_, p)| p).collect();
    people.sort_by_key(|p| p.id);
    let briefs: Vec<_> = people
        .into_iter()
        .map(|p| {
            let (trip, trip_rev) = p
                .trip
                .as_ref()
                .filter(|_| trips)
                .map_or((0, 0), |t| (t.id, t.rev));
            // Someone of a household-resolved settlement is drawn where their day's block by the
            // clock puts them (ADR-0032 §6).
            let block = sim.people.day_block(p.id, now);
            let midnight = now.day_index() * civ_core::time::MINUTES_PER_DAY;
            wire::PersonBrief::create(
                fbb,
                &wire::PersonBriefArgs {
                    id: p.id.get(),
                    pos: Some(&vec2(block.map_or_else(|| p.position_at(t), |b| b.at))),
                    activity: block.map_or(p.act.def, |b| b.def),
                    trip,
                    trip_rev,
                    sex: sex(p.sex),
                    age_years: p.age_years(now) as f32,
                    household: p.household.get(),
                    asleep: block.map_or(p.asleep, |b| b.asleep),
                    block_until_minute: block.map_or(0, |b| midnight + i64::from(b.until)),
                },
            )
        })
        .collect();
    fbb.create_vector(&briefs)
}

/// The bands not yet home or dispersed (wire 1.74, M6c slice BJ), oldest first: each one's
/// number, who of it are going (before the muster those who said they would, after it those who
/// came and are not yet home), and in words ("Ada leads 5 of Oakdale to take food from Bo's
/// household at Thornmere").
pub fn band_lines(sim: &Sim) -> Vec<(u32, Vec<u64>, String)> {
    use civ_agents::war::BandStage;
    let pop = &sim.people;
    let place = |s: PermanentId| {
        sim.land
            .settlements
            .iter()
            .find(|x| x.id == s)
            .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
    };
    pop.order
        .bands
        .iter()
        .filter(|b| !b.stage.over())
        .map(|b| {
            let going: Vec<u64> = match b.stage {
                BandStage::Called => b.members.iter().map(|m| m.get()).collect(),
                _ => b
                    .mustered
                    .iter()
                    .filter(|m| !b.home.contains(m))
                    .filter(|&&m| pop.person(m).is_some())
                    .map(|m| m.get())
                    .collect(),
            };
            let to = pop
                .household(b.target)
                .and_then(|x| x.settlement)
                .map_or_else(String::new, |s| format!(" at {}", place(s)));
            let what = b.purpose.words().trim_start_matches("to ");
            let whose = household_name(sim, b.target);
            let law = if b.law.is_some() {
                ", sent by its gathering's law"
            } else {
                ""
            };
            let words = match b.stage {
                BandStage::Called => format!(
                    "{} has called {} of {} to {what} from {whose}{to}{law}; they muster at the \
                     hearth at {}:00",
                    pop.name_of(b.organizer),
                    going.len(),
                    place(b.settlement),
                    b.muster.rem_euclid(24 * 60) / 60
                ),
                // Camped by a shut line (M6c slice BL).
                BandStage::Camped => format!(
                    "{} and {} of {} are camped outside the palisade{to}, come {what} from \
                     {whose}{law}",
                    pop.name_of(b.organizer),
                    going.len().saturating_sub(1),
                    place(b.settlement)
                ),
                // Holding a settlement (M6c slice BM).
                BandStage::Holding => format!(
                    "{} and {} of {} hold the place{to}{law}, keeping it{held}",
                    pop.name_of(b.organizer),
                    going.len().saturating_sub(1),
                    place(b.settlement),
                    held = if b.held.is_some() {
                        " as long as a hold needs"
                    } else {
                        ""
                    }
                ),
                _ => format!(
                    "{} leads {} of {} to {what} from {whose}{to}{law}",
                    pop.name_of(b.organizer),
                    going.len(),
                    place(b.settlement)
                ),
            };
            (b.id, going, words)
        })
        .collect()
}

/// The snapshot's bands not yet home or dispersed (wire 1.74), as [`band_lines`] has them.
pub fn band_briefs<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::BandBrief<'a>>>> {
    let briefs: Vec<_> = band_lines(sim)
        .into_iter()
        .filter_map(|(id, going, words)| {
            let b = sim.people.order.bands.iter().find(|b| b.id == id)?;
            let members = fbb.create_vector(&going);
            let words = fbb.create_string(&words);
            Some(wire::BandBrief::create(
                fbb,
                &wire::BandBriefArgs {
                    id,
                    from: b.settlement.get(),
                    stage: u8::from(b.stage != civ_agents::war::BandStage::Called),
                    members: Some(members),
                    target: Some(&vec2(b.place)),
                    words: Some(words),
                },
            ))
        })
        .collect();
    fbb.create_vector(&briefs)
}

/// The snapshot's settlements, oldest first, with their living population.
pub fn settlement_briefs<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
) -> Offsets<'a, wire::SettlementBrief<'a>> {
    let briefs: Vec<_> = sim
        .land
        .settlements
        .iter()
        .map(|s| {
            let population: usize = sim
                .people
                .households
                .iter()
                .filter(|(_, h)| h.settlement == Some(s.id))
                .map(|(_, h)| h.members.len())
                .sum();
            let name = fbb.create_string(&s.name);
            let food_days = sim.people.settlement_food_days(
                s.id,
                sim.now(),
                &sim.rules.people,
                &sim.rules.catalog.goods,
            );
            let founding = fbb.create_string(&founding_words(sim, s));
            let year = fbb.create_string(&year_words(sim, s.id));
            let contacts = fbb.create_string(&contacts_words(sim, s.id));
            let style = fbb.create_string(&style_clock_words(sim, s.id));
            let coalitions: Vec<_> = coalition_words(sim, s.id)
                .iter()
                .map(|w| fbb.create_string(w))
                .collect();
            let coalitions = fbb.create_vector(&coalitions);
            wire::SettlementBrief::create(
                fbb,
                &wire::SettlementBriefArgs {
                    id: s.id.get(),
                    name: Some(name),
                    hearth: Some(&vec2(s.hearth_m)),
                    founded_minute: s.founded.minutes(),
                    population: population as u32,
                    food_days: food_days.unwrap_or(0.0) as f32,
                    food_short: s.food_short,
                    harvest_kg: s.harvest_kg as f32,
                    founding: Some(founding),
                    year: Some(year),
                    abandoned_minute: s.abandoned.map_or(-1, |t| t.minutes()),
                    contacts: Some(contacts),
                    coalitions: Some(coalitions),
                    style: Some(style),
                    household_resolved: sim.household_resolved().contains(&s.id)
                        && !sim.people().held_detailed().contains(&s.id),
                },
            )
        })
        .collect();
    fbb.create_vector(&briefs)
}

/// The coalitions of settlement `s`'s households gathered to found a settlement, still gathering
/// or ended in the past year, each in words (M5a slice AO): "Ada's household gathers 2 more
/// households, 14 people, to found a settlement 3.1 km north, since the spring of year 2; lacks
/// seed"; "Ada's household and 2 more founded Birchford in the summer of year 3".
pub fn coalition_words(sim: &Sim, s: PermanentId) -> Vec<String> {
    use civ_agents::places::{CoalitionFate, Lacking};
    let now = sim.now();
    let year_ago = now.day_index() - civ_core::time::DAYS_PER_YEAR;
    let day_words =
        |d: i64| season_words(SimTime::from_minutes(d * civ_core::time::MINUTES_PER_DAY));
    sim.people
        .coalitions
        .iter()
        .filter(|c| c.from == s && c.ended.is_none_or(|d| d >= year_ago))
        .map(|c| {
            let who = c.named.map_or_else(
                || "A household".to_owned(),
                |p| format!("{}'s household", sim.people.name_of(p)),
            );
            let more = c.members.len().saturating_sub(1);
            let them = match more {
                0 => who.clone(),
                1 => format!("{who} and one more household"),
                n => format!("{who} and {n} more households"),
            };
            match c.fate {
                CoalitionFate::Gathering => {
                    let home = sim.land.settlements.iter().find(|x| x.id == s);
                    let way = home.map_or_else(String::new, |h| {
                        let (dx, dy) = (c.site.0 - h.hearth_m.0, c.site.1 - h.hearth_m.1);
                        let way = match ((dy.atan2(dx).to_degrees() + 382.5) % 360.0 / 45.0) as u32
                        {
                            0 => "east",
                            1 => "south-east",
                            2 => "south",
                            3 => "south-west",
                            4 => "west",
                            5 => "north-west",
                            6 => "north",
                            _ => "north-east",
                        };
                        format!(" {:.1} km {way}", dx.hypot(dy) / 1000.0)
                    });
                    let lack = match c.lacking {
                        Lacking::Nothing => "",
                        Lacking::Food => "; too little food to go yet",
                        Lacking::Seed => "; too little seed to go yet",
                    };
                    format!(
                        "{them} {} to found a settlement{way}, {} people, since {}{lack}",
                        if more == 0 { "means" } else { "mean" },
                        c.people,
                        day_words(c.formed)
                    )
                }
                CoalitionFate::Founded => format!(
                    "{them} founded {} in {}",
                    place_name(sim, c.settlement),
                    day_words(c.ended.unwrap_or(c.formed))
                ),
                CoalitionFate::Dissolved => format!(
                    "{who} gave up its plan to found a settlement in {}",
                    day_words(c.ended.unwrap_or(c.formed))
                ),
            }
        })
        .collect()
}

/// A settlement's name, or "beyond the map" for none (ADR-0018 §3).
pub(crate) fn place_name(sim: &Sim, s: Option<PermanentId>) -> String {
    s.and_then(|s| sim.land.settlements.iter().find(|x| x.id == s))
        .map_or_else(|| "beyond the map".to_owned(), |x| x.name.clone())
}

/// When `t` was, in words: "the spring of year 3".
fn season_words(t: SimTime) -> String {
    let d = t.date();
    format!("the {} of year {}", d.season().name(), d.year)
}

/// How settlement `s` was founded, in words (ADR-0018 §1).
pub fn founding_words(sim: &Sim, s: &civ_land::Settlement) -> String {
    match s.founding {
        civ_land::Founding::Setup => "one of the groups the world began with".to_owned(),
        civ_land::Founding::Sent => "by a family the observer sent".to_owned(),
        civ_land::Founding::Wave => "by a migration wave".to_owned(),
        civ_land::Founding::Coalition => match s.parent {
            Some(p) => format!("by households from {}", place_name(sim, Some(p))),
            None => "by households already in the world".to_owned(),
        },
    }
}

/// Settlement `s`'s accounts over the past year, in words (ADR-0018 §3): "3 born, 1 died, 4 came
/// from beyond the map"; empty when nothing changed.
pub fn year_words(sim: &Sim, s: PermanentId) -> String {
    let now = sim.now();
    let from = now.plus_minutes(-civ_core::time::MINUTES_PER_YEAR);
    let a = sim.people.accounts(s, from, now.plus_minutes(1));
    let mut parts = Vec::new();
    if a.births > 0 {
        parts.push(format!("{} born", a.births));
    }
    if a.deaths > 0 {
        parts.push(format!("{} died", a.deaths));
    }
    for (&origin, &n) in &a.arrivals {
        parts.push(format!("{n} came from {}", place_name(sim, origin)));
    }
    for (&to, &n) in &a.departures {
        match to {
            None => parts.push(format!("{n} left the map")),
            Some(_) => parts.push(format!("{n} went to {}", place_name(sim, to))),
        }
    }
    parts.join(", ")
}

/// A taste's traits in words: "48° roofs, 1.9 m to the eaves, 0.5 m out", and its front where it
/// has one ([`front_words`]).
fn traits_words(mean: [f64; Taste::TRAITS]) -> String {
    format!(
        "{:.0}° roofs, {:.1} m to the eaves, {:.1} m out{}",
        mean[0] / 100.0,
        mean[1] / 100.0,
        mean[2] / 100.0,
        front_words(&mean)
    )
}

/// A front's four traits (M8d slice DD), as a mean over a taste or buildings, in words after the
/// roof's: ", a tenth of the walls open, 0.5 h of ornament a square metre, daubed, half set out
/// about the middle"; each left out where it is nothing.
pub fn front_words(mean: &[f64; Taste::TRAITS]) -> String {
    let mut parts = Vec::new();
    if mean[3] >= 1.0 {
        parts.push(format!("{:.0} % of the walls open", mean[3] / 10.0));
    }
    if mean[4] >= 0.5 {
        parts.push(format!(
            "{:.1} h of ornament a square metre",
            mean[4] / 10.0
        ));
    }
    if mean[5] >= 0.05 {
        parts.push(match mean[5] {
            f if f >= 1.95 => "lime-washed".to_owned(),
            f if (0.95..=1.05).contains(&f) => "daubed".to_owned(),
            f => format!("finished {f:.1} of the way from bare to lime-washed"),
        });
    }
    if mean[6] >= 0.05 {
        parts.push(if mean[6] >= 0.95 {
            "set out about the middle".to_owned()
        } else {
            format!("{:.0} % set out about the middle", mean[6] * 100.0)
        });
    }
    parts.iter().map(|p| format!(", {p}")).collect()
}

/// A spread's mean in words, with how far the pitch spreads: "47.6° (±1.1°) roofs, 1.9 m to the
/// eaves, 0.5 m out", and its front where it has one.
fn spread_words(s: &civ_agents::style::Spread) -> String {
    format!(
        "{:.1}° (±{:.1}°) roofs, {:.1} m to the eaves, {:.1} m out{}",
        s.mean[0] / 100.0,
        s.sd[0] / 100.0,
        s.mean[1] / 100.0,
        s.mean[2] / 100.0,
        front_words(&s.mean)
    )
}

/// Settlement `s`'s way of building on the three clocks, in words (M5b slice AR; research 11-02
/// §4): "founded to build 48° roofs, 1.9 m to the eaves, 0.5 m out; its 12 households would build
/// 47.6° (±1.1°) roofs, …; the year's 3 new buildings: 48.1° (±0.4°) roofs, …, 1 after a
/// building elsewhere; its 31 standing buildings: 47.9° (±1.0°) roofs, …"; empty with no people,
/// buildings or founding way.
pub fn style_clock_words(sim: &Sim, s: PermanentId) -> String {
    let c = sim
        .people
        .style_clocks(&sim.rules.catalog, &sim.land, sim.now(), s);
    let mut parts = Vec::new();
    if let Some(way) = c.way {
        parts.push(format!(
            "founded to build {}",
            traits_words(way.traits().map(f64::from))
        ));
    }
    if c.taste.n > 0 {
        let who = if c.taste.n == 1 {
            "its one household".to_owned()
        } else {
            format!("its {} households", c.taste.n)
        };
        parts.push(format!("{who} would build {}", spread_words(&c.taste)));
    }
    if c.new.n > 0 {
        let what = if c.new.n == 1 {
            "the year's one new building".to_owned()
        } else {
            format!("the year's {} new buildings", c.new.n)
        };
        let after = match c.new_after_elsewhere {
            0 => String::new(),
            n => format!(", {n} after a building elsewhere"),
        };
        parts.push(format!("{what}: {}{after}", spread_words(&c.new)));
    } else if c.stock.n > 0 {
        parts.push("no new building this year".to_owned());
    }
    if c.stock.n > 0 {
        let what = if c.stock.n == 1 {
            "its one standing building".to_owned()
        } else {
            format!("its {} standing buildings", c.stock.n)
        };
        parts.push(format!("{what}: {}", spread_words(&c.stock)));
    }
    parts.join("; ")
}

/// The buildings of other settlements `p` has seen since their household's last taste review, by
/// settlement, in words (M5b slice AR): "At Westford: Cal's hut, Bo's longhouse"; empty when none.
pub fn seen_away_words(sim: &Sim, p: PermanentId) -> String {
    let Some(list) = sim.people.seen_away.get(&p) else {
        return String::new();
    };
    let mut by: Vec<(String, Vec<String>)> = Vec::new();
    for &b in list {
        let Some(x) = sim.land.buildings.iter().find(|x| x.id == b) else {
            continue;
        };
        let at = sim
            .people
            .building_settlement(&sim.land, x)
            .and_then(|s| sim.land.settlements.iter().find(|y| y.id == s))
            .map_or_else(|| "another settlement".to_owned(), |y| y.name.clone());
        let name = whose_building(sim, x);
        match by.iter_mut().find(|(s, _)| *s == at) {
            Some((_, names)) => names.push(name),
            None => by.push((at, vec![name])),
        }
    }
    by.iter()
        .map(|(at, names)| format!("At {at}: {}", names.join(", ")))
        .collect::<Vec<_>>()
        .join("; ")
}

/// What passed between settlement `s` and others last year and this year so far, in words (M5a
/// slices AM and AN): "in year 2: 12 visits from Ashford, 7 hours at the hearth here; 3 visits to
/// Ashford; 5 people moved here from Ashford"; empty when nothing did.
pub fn contacts_words(sim: &Sim, s: PermanentId) -> String {
    let this = sim.now().date().year - 1;
    let count = |n: u32, one: &str, many: &str| {
        if n == 1 {
            format!("1 {one}")
        } else {
            format!("{n} {many}")
        }
    };
    let mut parts = Vec::new();
    for year in [this - 1, this] {
        let mut lines = Vec::new();
        for (from, to, c) in sim.people.contacts.of_year(year) {
            if to == s && c.visits > 0 {
                lines.push(format!(
                    "{} from {}, {:.0} hours at the hearth here",
                    count(c.visits, "visit", "visits"),
                    place_name(sim, Some(from)),
                    c.minutes as f64 / 60.0
                ));
            }
            if from == s && c.visits > 0 {
                lines.push(format!(
                    "{} to {}",
                    count(c.visits, "visit", "visits"),
                    place_name(sim, Some(to))
                ));
            }
            if to == s && c.marriages > 0 {
                lines.push(format!(
                    "{} from {}",
                    count(c.marriages, "marriage", "marriages"),
                    place_name(sim, Some(from))
                ));
            }
            if from == s && c.marriages > 0 {
                lines.push(format!(
                    "{} into {}",
                    count(c.marriages, "marriage", "marriages"),
                    place_name(sim, Some(to))
                ));
            }
            // Households that moved (M5a slice AN).
            if to == s && c.moved > 0 {
                lines.push(format!(
                    "{} moved here from {}",
                    count(c.moved, "person", "people"),
                    place_name(sim, Some(from))
                ));
            }
            if from == s && c.moved > 0 {
                lines.push(format!(
                    "{} moved to {}",
                    count(c.moved, "person", "people"),
                    place_name(sim, Some(to))
                ));
            }
            // Purchases at sellers' doors elsewhere, by report (M5b slice AP).
            if to == s && c.bought > 0 {
                lines.push(format!(
                    "{} here by people of {}",
                    count(c.bought, "purchase", "purchases"),
                    place_name(sim, Some(from))
                ));
            }
            if from == s && c.bought > 0 {
                lines.push(format!(
                    "{} at {}",
                    count(c.bought, "purchase", "purchases"),
                    place_name(sim, Some(to))
                ));
            }
            let missed: u32 = c.missed.iter().sum();
            if from == s && missed > 0 {
                let why: Vec<String> = civ_agents::reports::Missed::ALL
                    .iter()
                    .zip(c.missed)
                    .filter(|&(_, n)| n > 0)
                    .map(|(m, n)| format!("{n} where {}", m.words()))
                    .collect();
                lines.push(format!(
                    "{} to buy at {} that bought nothing ({})",
                    count(missed, "trip", "trips"),
                    place_name(sim, Some(to)),
                    why.join(", ")
                ));
            }
        }
        if !lines.is_empty() {
            let when = if year == this {
                format!("in year {} so far", year + 1)
            } else {
                format!("in year {}", year + 1)
            };
            parts.push(format!("{when}: {}", lines.join("; ")));
        }
    }
    parts.join(". ")
}

/// Where person `id` has lived, oldest first, each in words (ADR-0018 §2): "Ashford, from the
/// spring of year 1: came with a founding group".
pub fn residence_words(sim: &Sim, id: PermanentId) -> Vec<String> {
    sim.people
        .records
        .get(&id)
        .map(|r| {
            r.residence
                .iter()
                .map(|x| {
                    format!(
                        "{}, from {}: {}",
                        place_name(sim, x.settlement),
                        season_words(x.since),
                        x.why.words()
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The other settlements household `household` knows, each in words (ADR-0018 §4): "Elmhollow:
/// seen on a walk in the spring of year 1", with who told of it, if anyone.
pub fn places_words(sim: &Sim, household: PermanentId) -> Vec<String> {
    sim.people
        .known_places
        .of(household)
        .iter()
        .map(|k| {
            let since = SimTime::from_minutes(k.first * civ_core::time::MINUTES_PER_DAY);
            let by = k
                .from
                .filter(|_| k.how == civ_agents::places::PlaceHow::Told)
                .map(|f| format!(" by {}", sim.people.name_of(f)))
                .unwrap_or_default();
            // What a member saw of its food when last there (M5a slice AM, visits).
            let food = k.food.map_or_else(String::new, |fed| {
                let seen = if fed >= 0.9 {
                    "nobody they met going hungry"
                } else if fed >= 0.5 {
                    "some they met going hungry"
                } else {
                    "most they met going hungry"
                };
                format!("; when a member was last there, {seen}")
            });
            format!(
                "{}: {}{by} in {}{food}",
                place_name(sim, Some(k.settlement)),
                k.how.words(),
                season_words(since)
            )
        })
        .collect()
}

/// The newest chronicle entry's sequence number (0 = none).
pub fn chronicle_head(sim: &Sim) -> u64 {
    sim.people.chronicle.last().map_or(0, |e| e.seq)
}

/// A `Response` with the geometry of the asked-for trips that are still under way.
pub fn trips_response(sim: &Sim, ids: &[u64]) -> Result<Vec<u8>, QueryError> {
    if ids.len() > MAX_TRIPS_PER_QUERY {
        return Err(QueryError(format!(
            "{} trips asked for; at most {MAX_TRIPS_PER_QUERY} per query",
            ids.len()
        )));
    }
    let wanted: HashSet<u64> = ids.iter().copied().collect();
    let mut fbb = FlatBufferBuilder::new();
    let mut found: Vec<&Person> = sim
        .people
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.trip.as_ref().is_some_and(|t| wanted.contains(&t.id)))
        .collect();
    found.sort_by_key(|p| p.id);
    let trips: Vec<_> = found
        .into_iter()
        .filter_map(|p| {
            let t = p.trip.as_ref()?;
            let points: Vec<wire::Vec2> = t.points.iter().map(|&q| vec2(q)).collect();
            let points = fbb.create_vector(&points);
            let minutes = fbb.create_vector(&t.minutes);
            Some(wire::TripInfo::create(
                &mut fbb,
                &wire::TripInfoArgs {
                    id: t.id,
                    rev: t.rev,
                    person: p.id.get(),
                    depart_minute: t.depart.minutes(),
                    points: Some(points),
                    minutes: Some(minutes),
                },
            ))
        })
        .collect();
    let trips = fbb.create_vector(&trips);
    let body = wire::Trips::create(&mut fbb, &wire::TripsArgs { trips: Some(trips) });
    Ok(response(fbb, wire::ResponseBody::Trips, body))
}

/// A `Response` with chronicle entries after `after_seq`, oldest first.
pub fn chronicle_response(sim: &Sim, after_seq: u64, limit: u32) -> Vec<u8> {
    let limit = limit.clamp(1, MAX_CHRONICLE_PER_QUERY) as usize;
    let pop = &sim.people;
    let name_of = |id: PermanentId| pop.name_of(id);
    let mut fbb = FlatBufferBuilder::new();
    let entries: Vec<_> = pop
        .chronicle
        .iter()
        .filter(|e| e.seq > after_seq)
        .take(limit)
        .map(|e| {
            let spans = history::render(e, &name_of);
            let spans: Vec<_> = spans
                .iter()
                .map(|s| {
                    let (kind, text, id) = match s {
                        Span::Text(t) => (wire::SpanKind::Text, t.as_str(), 0),
                        Span::Person(id, n) => (wire::SpanKind::Person, n.as_str(), id.get()),
                        Span::Settlement(id, n) => {
                            (wire::SpanKind::Settlement, n.as_str(), id.get())
                        }
                        Span::Firm(id, n) => (wire::SpanKind::Firm, n.as_str(), id.get()),
                    };
                    let text = fbb.create_string(text);
                    wire::Span::create(
                        &mut fbb,
                        &wire::SpanArgs {
                            kind,
                            text: Some(text),
                            id,
                        },
                    )
                })
                .collect();
            let spans = fbb.create_vector(&spans);
            // The incident or outbreak it tells of (wire 1.65).
            let link = super::incidents::chronicle_link(sim, e);
            wire::ChronicleEntry::create(
                &mut fbb,
                &wire::ChronicleEntryArgs {
                    seq: e.seq,
                    minute: e.at.minutes(),
                    spans: Some(spans),
                    incident_kind: link.map_or(0, |l| l.0),
                    incident: link.map_or(0, |l| l.1),
                    at: e.place.map(|(x, y)| wire::Vec2::new(x, y)).as_ref(),
                },
            )
        })
        .collect();
    let entries = fbb.create_vector(&entries);
    let body = wire::Chronicle::create(
        &mut fbb,
        &wire::ChronicleArgs {
            entries: Some(entries),
            head: chronicle_head(sim),
        },
    );
    response(fbb, wire::ResponseBody::Chronicle, body)
}

/// Distance and compass direction from `from` to `to`, in words: "600 m east".
pub fn bearing(from: (f32, f32), to: (f32, f32)) -> String {
    let (dx, dy) = (f64::from(to.0 - from.0), f64::from(to.1 - from.1));
    let d = dx.hypot(dy);
    if d < 5.0 {
        return "here".to_owned();
    }
    let distance = if d < 100.0 {
        format!("{} m", ((d / 10.0).round() * 10.0) as i64)
    } else if d < 1000.0 {
        format!("{} m", ((d / 50.0).round() * 50.0) as i64)
    } else {
        format!("{:.1} km", d / 1000.0)
    };
    // Map y runs south; compass angles run anticlockwise from east.
    let angle = (-dy).atan2(dx).to_degrees().rem_euclid(360.0);
    const WINDS: [&str; 8] = [
        "east",
        "north-east",
        "north",
        "north-west",
        "west",
        "south-west",
        "south",
        "south-east",
    ];
    let wind = WINDS[((angle + 22.5) / 45.0) as usize % 8];
    format!("{distance} {wind}")
}

/// What a target of a member of `household` is, in words, seen from `home`.
pub fn describe_target(
    sim: &Sim,
    home: (f32, f32),
    household: PermanentId,
    target: Target,
) -> String {
    match target {
        Target::None => String::new(),
        Target::Home => "home".to_owned(),
        // One's own settlement's hearth is "the hearth"; another's is named (ADR-0018 §2).
        Target::Hearth(s) => {
            let own = sim.people.household(household).and_then(|x| x.settlement) == Some(s);
            match sim.land.settlements.iter().find(|x| x.id == s) {
                Some(x) if !own => format!("the hearth of {}", x.name),
                _ => "the hearth".to_owned(),
            }
        }
        Target::Patch(p) if (p as usize) < sim.land.patches.len() => {
            let class = sim.land.patches.class[p as usize] as usize;
            let habitat = sim
                .rules
                .land
                .habitats
                .get(class)
                .map_or("land", |h| h.name.as_str())
                .to_lowercase();
            let at = sim.land.patches.centre_m(p as usize);
            format!("{habitat} {} of home", bearing(home, at))
        }
        Target::Patch(_) => "a patch".to_owned(),
        Target::Water(cell) => {
            let at = population::cell_centre(&sim.map, cell as usize);
            format!("water {} of home", bearing(home, at))
        }
        Target::Field(id) => match sim.land.fields.iter().find(|f| f.id == id) {
            Some(f) => format!("the field {} of home", bearing(home, f.rect.centre_m())),
            None => "a field".to_owned(),
        },
        Target::NewField => "new ground".to_owned(),
        // A roof a fire threatens (M6b slice BD, step three).
        Target::Douse { building, .. } => super::incidents::building_words(sim, building),
        Target::Household(id) => household_name(sim, id),
        Target::Building(id) => match sim.land.buildings.iter().find(|b| b.id == id) {
            Some(b) => {
                let name = program_name(sim, &b.spec.program);
                let at = civ_agents::build::centre_m(&b.spec);
                if (at.0 - home.0).abs() < 1.0 && (at.1 - home.1).abs() < 1.0 {
                    format!("the {name} at home")
                } else {
                    format!("a {name} {} of home", bearing(home, at))
                }
            }
            None => "a building".to_owned(),
        },
        Target::NewBuilding => {
            // The home the household plans, or else the first it may build.
            let planned = sim
                .people
                .planned_home(household)
                .map(|s| s.program.as_str());
            let program = planned
                .or_else(|| {
                    sim.rules
                        .people
                        .build
                        .programs
                        .first()
                        .and_then(|&p| sim.rules.catalog.buildings.get(p))
                        .map(|b| b.id.as_str())
                })
                .unwrap_or("");
            format!("a new {}", program_name(sim, program))
        }
        Target::Firm(id) => firm_name(sim, id),
        Target::NewFirm => "a new workshop".to_owned(),
        // What trying works toward: the observer sees it, though the person cannot.
        Target::Technique(t) => sim
            .rules
            .catalog
            .techniques
            .get(usize::from(t))
            .map_or_else(
                || "something new".to_owned(),
                |d| format!("toward {}", d.name.to_lowercase()),
            ),
        // The deposit dug at: "the clay pit north-east of home", "the stone quarry west of home".
        Target::Deposit(id) => match sim.land.deposits.iter().find(|d| d.id == id) {
            Some(d) => {
                let g = usize::from(d.body.good);
                let good = sim
                    .rules
                    .catalog
                    .goods
                    .get(g)
                    .map_or_else(|| "a".to_owned(), |g| g.name.to_lowercase());
                let working = super::earthworks::working(sim, Some(g));
                let at = (
                    (d.body.at_cm.0 as f64 / 100.0) as f32,
                    (d.body.at_cm.1 as f64 / 100.0) as f32,
                );
                format!("the {good} {working} {} of home", bearing(home, at))
            }
            None => "a pit".to_owned(),
        },
        // A crossing worked on (M5c slice AW): "the log footbridge north-east of home".
        Target::Crossing(id) => match sim.land.crossings.list.iter().find(|c| c.id == id) {
            Some(c) => {
                let name = sim
                    .rules
                    .catalog
                    .bridges
                    .get(usize::from(c.system))
                    .map_or_else(|| "crossing".to_owned(), |d| d.name.to_lowercase());
                let at = c
                    .cells
                    .first()
                    .map_or(home, |&x| population::cell_centre(&sim.map, x as usize));
                format!("the {name} {} of home", bearing(home, at))
            }
            None => "a crossing".to_owned(),
        },
        // A well dug or drawn at (M6a slice AY, step three): "the timber-lined well beside home",
        // "the timber-lined well south of home".
        Target::Well(id) => match sim.land.wells.get(id) {
            Some(w) => {
                let name = sim
                    .rules
                    .catalog
                    .wells
                    .get(w.system)
                    .map_or_else(|| "well".to_owned(), |d| d.name.to_lowercase());
                let at = w.rect.centre_m();
                if (at.0 - home.0).hypot(at.1 - home.1) <= 15.0 {
                    format!("the {name} beside home")
                } else {
                    format!("the {name} {} of home", bearing(home, at))
                }
            }
            None => "a well".to_owned(),
        },
        // A flood bank built by law (M6b slice BG): "the flood bank east of home".
        Target::Bank(id) => match sim.land.banks.iter().find(|b| b.work == id) {
            Some(b) => {
                let at = (
                    ((b.a[0] + b.b[0]) / 2.0) as f32,
                    ((b.a[1] + b.b[1]) / 2.0) as f32,
                );
                if (at.0 - home.0).hypot(at.1 - home.1) <= 15.0 {
                    "the flood bank beside home".to_owned()
                } else {
                    format!("the flood bank {} of home", bearing(home, at))
                }
            }
            None => "a flood bank".to_owned(),
        },
        // A neighbour's water mill (M10a slice DR, fourth part): "the mill south of home".
        Target::Mill(id) => match sim.land.installations.iter().find(|m| m.id == id) {
            Some(m) => format!(
                "the mill {} of home",
                bearing(home, (m.at[0] as f32, m.at[1] as f32))
            ),
            None => "a mill".to_owned(),
        },
        // An irrigation ditch (M6b slice BG): "the ditch south of home".
        Target::Ditch(id) => match sim.land.ditches.iter().find(|d| d.work == id) {
            Some(d) => {
                let at = (
                    ((d.a[0] + d.b[0]) / 2.0) as f32,
                    ((d.a[1] + d.b[1]) / 2.0) as f32,
                );
                if (at.0 - home.0).hypot(at.1 - home.1) <= 15.0 {
                    "the ditch beside home".to_owned()
                } else {
                    format!("the ditch {} of home", bearing(home, at))
                }
            }
            None => "a ditch".to_owned(),
        },
        // A palisade raised by law (M6c slice BK): "the palisade round Ashford".
        Target::Enclosure(id) => match sim.land.enclosures.position(id) {
            Some(i) => {
                let e = &sim.land.enclosures.list[i];
                match sim.land.settlements.iter().find(|s| s.id == e.settlement) {
                    Some(s) => format!("the palisade round {}", s.name),
                    None => "a palisade".to_owned(),
                }
            }
            None => "a palisade".to_owned(),
        },
        // A boat being made (M8b slice CO): "the dugout on the bank north of home".
        Target::Boat(id) => match sim.land.boats.list.iter().find(|b| b.id == id) {
            Some(b) => {
                let name = sim
                    .rules
                    .catalog
                    .vehicles
                    .get(usize::from(b.vehicle))
                    .map_or_else(|| "boat".to_owned(), |v| v.name.to_lowercase());
                let at = sim.land.rivers.centre(b.at);
                format!("the {name} on the bank {} of home", bearing(home, at))
            }
            // A cart being made at the door (M8b slice CP, third part): "the household's
            // handcart".
            None => match sim.land.carts.list.iter().find(|c| c.id == id) {
                Some(c) => {
                    let name = sim
                        .rules
                        .catalog
                        .vehicles
                        .get(usize::from(c.vehicle))
                        .map_or_else(|| "cart".to_owned(), |v| v.name.to_lowercase());
                    format!("the household's {name}")
                }
                None => "a boat".to_owned(),
            },
        },
        // A passing place a household is clearing (M8b slice CQ, second part): "a passing place
        // east of home"; or a way it is laying on worn trail (the fourth part): "a way laid east
        // of home".
        Target::Way(id) => match sim.land.ways.list.iter().find(|w| w.id == id) {
            Some(w) => {
                let what =
                    if w.segments.first().is_some_and(|s| {
                        s.corridor_m <= civ_land::carting::TRAIL_TREAD_M as f32 + 1e-3
                    }) {
                        "a way laid"
                    } else {
                        "a passing place"
                    };
                match w.segments.first().and_then(|s| s.cells.first()) {
                    Some(&c) if (c as usize) < sim.map.water.len() => {
                        let at = civ_agents::population::cell_centre(&sim.map, c as usize);
                        format!("{what} {} of home", bearing(home, at))
                    }
                    _ => what.to_owned(),
                }
            }
            None => "a passing place".to_owned(),
        },
    }
}

/// A firm in words, by its founder and what it makes: "Wren's sickle workshop".
pub fn firm_name(sim: &Sim, id: PermanentId) -> String {
    match sim.people.firm(id) {
        Some(f) => f.name(&sim.people.name_of(f.founder), &sim.rules.catalog.goods),
        None => "a workshop".to_owned(),
    }
}

/// A building program's name in running text: "hut".
pub(crate) fn program_name(sim: &Sim, program: &str) -> String {
    sim.rules.catalog.building_index(program).map_or_else(
        || "building".to_owned(),
        |i| sim.rules.catalog.buildings[i].name.to_lowercase(),
    )
}

/// Building `b` as its owners call it: "Ada's hut", "a hut", "the granary raised together".
pub(crate) fn whose_building(sim: &Sim, b: &Building) -> String {
    let program = program_name(sim, &b.spec.program);
    // A building a polity raised is held in its name (M8d slice DC).
    if b.polity.is_some() {
        return format!("the {program} raised together");
    }
    match eldest_name(sim, b.household) {
        Some(name) => format!("{name}'s {program}"),
        None => format!("a {program}"),
    }
}

/// Building `id` as its owners call it, or "a building since gone".
pub(crate) fn building_name(sim: &Sim, id: PermanentId) -> String {
    sim.land.buildings.iter().find(|b| b.id == id).map_or_else(
        || "a building since gone".to_owned(),
        |b| whose_building(sim, b),
    )
}

/// [`building_name`], with the settlement it stands in when that is not `home` (M5b slice AR):
/// "Bo's hut at Westford".
pub(crate) fn building_name_from(sim: &Sim, id: PermanentId, home: Option<PermanentId>) -> String {
    let name = building_name(sim, id);
    let there = sim
        .land
        .buildings
        .iter()
        .find(|b| b.id == id)
        .and_then(|b| sim.people.household(b.household))
        .and_then(|h| h.settlement)
        .filter(|&s| Some(s) != home)
        .and_then(|s| sim.land.settlements.iter().find(|x| x.id == s));
    match there {
        Some(s) => format!("{name} at {}", s.name),
        None => name,
    }
}

/// How household `h` would build, in words, and the building that moved its taste most (M3b
/// slice R): "roofs pitched 48°, walls 1.9 m to the eaves, eaves 0.5 m out; admiring Bo's hut",
/// and where it stands if in another settlement (M5b slice AR).
pub fn taste_words(sim: &Sim, h: &civ_agents::person::Household) -> String {
    let t = h.taste;
    let mut words = format!(
        "roofs pitched {:.0}°, walls {:.1} m to the eaves, eaves {:.1} m out",
        t.pitch_centideg / 100.0,
        t.eave_cm / 100.0,
        t.overhang_cm / 100.0
    );
    if let Some(b) = h.admired {
        words.push_str(&format!(
            "; admiring {}",
            building_name_from(sim, b, h.settlement)
        ));
    }
    words
}

/// The given name of a household's eldest member; `None` when nobody lives in it.
pub fn eldest_name(sim: &Sim, id: PermanentId) -> Option<String> {
    let now = sim.now();
    sim.people
        .household(id)
        .and_then(|h| {
            h.members
                .iter()
                .filter_map(|m| sim.people.person(*m))
                .max_by(|a, b| a.age_years(now).total_cmp(&b.age_years(now)))
        })
        .map(|p| p.given.clone())
}

/// A household in words, by its eldest member: "Ada's household".
pub fn household_name(sim: &Sim, id: PermanentId) -> String {
    match eldest_name(sim, id) {
        Some(name) => format!("{name}'s household"),
        None => "another household".to_owned(),
    }
}

/// What a living person is doing now, in words.
pub fn doing(sim: &Sim, p: &Person) -> String {
    let def = sim.rules.catalog.activities.get(usize::from(p.act.def));
    let what = def.map_or("busy", |d| d.doing.as_str());
    let home = sim.people.household(p.household).map(|h| h.home);
    let place = home.map_or_else(String::new, |h| {
        describe_target(sim, h, p.household, p.act.target)
    });
    match p.act.steps.get(usize::from(p.act.step)) {
        Some(Step::Walk { to }) => {
            if home.is_some_and(|h| (h.0 - to.0).abs() < 1.0 && (h.1 - to.1).abs() < 1.0) {
                if p.act.target == Target::Home {
                    format!("walking home ({what})")
                } else {
                    "walking home".to_owned()
                }
            } else if place.is_empty() {
                format!("walking ({what})")
            } else {
                format!("walking to {place} ({what})")
            }
        }
        // By the household's boat (M8b slice CO) or cart (M8b slice CP).
        Some(Step::Paddle { .. }) => format!("paddling the household's boat ({what})"),
        Some(Step::Haul { .. }) => format!("pulling the household's handcart ({what})"),
        // On a line's train, or waiting at its station for one (M10b slice DX, fourth part).
        Some(Step::Ride { .. }) => match p.trip.as_ref() {
            Some(t)
                if f64::from(t.minutes.get(1).copied().unwrap_or(0.0))
                    > (sim.now().minutes() - t.depart.minutes()) as f64 =>
            {
                format!("waiting at the station for the train ({what})")
            }
            _ => format!("riding the train ({what})"),
        },
        Some(Step::Handle { unload, .. }) => {
            let cart = p.act.steps.iter().any(|s| matches!(s, Step::Haul { .. }));
            match (unload, cart) {
                (false, false) => format!("loading the boat ({what})"),
                (true, false) => format!("unloading the boat and carrying the load up ({what})"),
                (false, true) => format!("loading the handcart ({what})"),
                (true, true) => format!("unloading the handcart ({what})"),
            }
        }
        Some(Step::Work { .. }) => {
            let at_home =
                home.is_some_and(|h| (h.0 - p.pos.0).abs() < 1.0 && (h.1 - p.pos.1).abs() < 1.0);
            match p.act.target {
                _ if place.is_empty() => what.to_owned(),
                // Threshing is done at home, with sheaves carried from the field.
                Target::Field(_) if at_home => format!("{what} from {place}"),
                Target::Patch(_) | Target::Water(_) | Target::Field(_) => {
                    format!("{what}, {place}")
                }
                Target::Household(_) | Target::Deposit(_) => format!("{what} at {place}"),
                // "building the log footbridge north of home" (M5c slice AW).
                Target::Crossing(_) => format!("building {place}"),
                // "digging the timber-lined well beside home", "relining ...", or "fetching
                // water, the timber-lined well beside home" (M6a slice AY, step three).
                Target::Well(id)
                    if def.is_some_and(|d| d.behavior == civ_agents::Behavior::Well) =>
                {
                    if sim.land.wells.get(id).is_some_and(|w| w.is_open()) {
                        format!("relining {place}")
                    } else {
                        format!("digging {place}")
                    }
                }
                Target::Well(_) => format!("{what}, {place}"),
                // "building the flood bank east of home" (M6b slice BG).
                Target::Bank(_) => format!("building {place}"),
                // "digging the ditch south of home" (M6b slice BG).
                Target::Ditch(_) => format!("digging {place}"),
                // "grinding grain at the mill south of home" (M10a slice DR).
                Target::Mill(_) => format!("grinding grain at {place}"),
                // "setting posts on the palisade round Ashford" (M6c slice BK).
                Target::Enclosure(_) => format!("setting posts on {place}"),
                // Another settlement's hearth (M5a slice AM): "visiting the hearth of Ashford".
                Target::Hearth(_)
                    if def.is_some_and(|d| d.behavior == civ_agents::Behavior::Visit) =>
                {
                    format!("{what} {place}")
                }
                Target::Building(id) => {
                    match sim
                        .land
                        .buildings
                        .iter()
                        .find(|b| b.id == id)
                        .and_then(|b| b.stage())
                    {
                        Some(stage) => format!("{what}: the {}", stage.name()),
                        None => what.to_owned(),
                    }
                }
                _ => what.to_owned(),
            }
        }
        Some(Step::Deposit) => "putting away what they brought".to_owned(),
        Some(Step::Wait { .. }) => "waiting".to_owned(),
        None => "deciding what to do".to_owned(),
    }
}

fn scored<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    (home, household): ((f32, f32), PermanentId),
    s: &Scored,
) -> WIPOffset<wire::ScoredOption<'a>> {
    let target = fbb.create_string(&describe_target(sim, home, household, s.target));
    let terms: Vec<wire::Term> = s
        .terms
        .iter()
        .map(|t| wire::Term::new(t.reason as u16, t.points))
        .collect();
    let terms = fbb.create_vector(&terms);
    wire::ScoredOption::create(
        fbb,
        &wire::ScoredOptionArgs {
            activity: s.def,
            target: Some(target),
            total: s.total,
            terms: Some(terms),
        },
    )
}

fn decision<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    from: ((f32, f32), PermanentId),
    r: &Receipt,
) -> WIPOffset<wire::Decision<'a>> {
    let chosen = scored(fbb, sim, from, &r.chosen);
    let runner_up = r.runner_up.as_ref().map(|s| scored(fbb, sim, from, s));
    let others: Vec<_> = r
        .others
        .iter()
        .map(|&(def, total)| {
            wire::ScoredOption::create(
                fbb,
                &wire::ScoredOptionArgs {
                    activity: def,
                    total,
                    ..Default::default()
                },
            )
        })
        .collect();
    let others = fbb.create_vector(&others);
    let excluded: Vec<wire::Exclusion> = r
        .excluded
        .iter()
        .map(|&(def, reason)| wire::Exclusion::new(def, reason as u16))
        .collect();
    let excluded = fbb.create_vector(&excluded);
    let needs = fbb.create_vector(&r.needs);
    wire::Decision::create(
        fbb,
        &wire::DecisionArgs {
            minute: r.at.minutes(),
            chosen: Some(chosen),
            runner_up,
            others: Some(others),
            excluded: Some(excluded),
            probability: r.probability,
            temperature: r.temperature,
            needs: Some(needs),
        },
    )
}

/// A person's relatives, by permanent id, with how they are related.
fn kin(sim: &Sim, id: PermanentId) -> Vec<(PermanentId, &'static str)> {
    let records = &sim.people.records;
    let Some(me) = records.get(&id) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    // Parents who never lived on the map (a wave's, who stayed behind) have no record to show.
    if let Some(m) = me.mother.filter(|m| records.contains_key(m)) {
        out.push((m, "mother"));
    }
    if let Some(f) = me.father.filter(|f| records.contains_key(f)) {
        out.push((f, "father"));
    }
    // Partners from the unions they were in; for a world older than unions, the other parent of
    // their children.
    let mut partners: Vec<PermanentId> = sim
        .people
        .unions
        .iter()
        .filter_map(|u| {
            if u.woman == id {
                Some(u.man)
            } else if u.man == id {
                Some(u.woman)
            } else {
                None
            }
        })
        .collect();
    for r in records.values() {
        if r.id == id {
            continue;
        }
        if r.mother == Some(id) || r.father == Some(id) {
            out.push((
                r.id,
                if r.sex == Sex::Male {
                    "son"
                } else {
                    "daughter"
                },
            ));
            let other = if r.mother == Some(id) {
                r.father
            } else {
                r.mother
            };
            if let Some(o) = other
                && !partners.contains(&o)
            {
                partners.push(o);
            }
        } else if (me.mother.is_some() && r.mother == me.mother)
            || (me.father.is_some() && r.father == me.father)
        {
            out.push((
                r.id,
                if r.sex == Sex::Male {
                    "brother"
                } else {
                    "sister"
                },
            ));
        }
    }
    for p in partners {
        out.push((p, "partner"));
    }
    out
}

/// "21 years", "8 months", "a few days".
fn how_long(minutes: i64) -> String {
    let days = minutes.max(0) as f64 / 1440.0;
    let years = days / 365.0;
    if years >= 2.0 {
        format!("{} years", years.floor() as i64)
    } else if years >= 1.0 {
        "a year".to_owned()
    } else if days >= 61.0 {
        format!("{} months", (days / 30.4).floor() as i64)
    } else if days >= 14.0 {
        format!("{} weeks", (days / 7.0).floor() as i64)
    } else {
        "a few days".to_owned()
    }
}

/// "in about 3 months", "in about 5 days".
fn in_about(minutes: i64) -> String {
    let days = minutes.max(0) as f64 / 1440.0;
    if days < 1.5 {
        "any day now".to_owned()
    } else if days < 14.0 {
        format!("in about {} days", days.round() as i64)
    } else if days < 60.0 {
        format!("in about {} weeks", (days / 7.0).round() as i64)
    } else {
        format!("in about {} months", (days / 30.4).round() as i64)
    }
}

/// A living person's family life in sentences, as the inspector shows it: their partner and since
/// when, or their widowhood; a pregnancy once it shows (the kernel knows more, such as whether it
/// will be lost, and does not say); the child they nurse.
pub fn family_notes(sim: &Sim, p: &Person) -> Vec<String> {
    let (now, pop) = (sim.now(), &sim.people);
    let mine = |u: &&civ_agents::Union| u.woman == p.id || u.man == p.id;
    let other = |u: &civ_agents::Union| if u.woman == p.id { u.man } else { u.woman };
    let mut out = Vec::new();
    if let Some(q) = p.partner {
        match pop
            .unions
            .iter()
            .rev()
            .filter(mine)
            .find(|u| u.ended.is_none())
        {
            Some(u) => out.push(format!(
                "Partner of {} for {}.",
                pop.name_of(q),
                how_long(now.minutes() - u.since.minutes())
            )),
            None => out.push(format!("Partner of {}.", pop.name_of(q))),
        }
    } else if let Some(u) = pop.unions.iter().rev().find(mine)
        && let Some(ended) = u.ended
    {
        out.push(format!(
            "Widowed when {} died in year {}.",
            pop.name_of(other(u)),
            ended.date().year
        ));
    }
    if let Repro::Pregnant { conceived, .. } = p.repro {
        let weeks = (now.minutes() - conceived.minutes()) / (7 * 1440);
        if weeks >= 8 {
            let days = sim.rules.people.fertility.pregnancy_days;
            let due = conceived.plus_minutes((days * 1440.0) as i64);
            out.push(format!(
                "Expecting a child, due {}.",
                in_about(due.minutes() - now.minutes())
            ));
        }
    }
    if let Some(child) = p.nursing.and_then(|c| pop.person(c)) {
        out.push(format!("Nursing {}.", child.given));
    }
    // Where their household lodges, or whom it lets to (ADR-0040 §6).
    for w in lodging_words(sim, p.household) {
        out.push(format!("Household {w}."));
    }
    out
}

/// Household `household`'s lets in words (ADR-0040 §6): "lodges with Ada's household, 9 m², 4 kg
/// of food a month", or for each household it lets to, "lets 9 m² to Bo's household".
pub fn lodging_words(sim: &Sim, household: PermanentId) -> Vec<String> {
    let letting = &sim.people.letting;
    let mut out = Vec::new();
    if let Some(l) = letting.lodging(household) {
        out.push(format!(
            "lodges with {}, {:.0} m², {:.0} kg of food a month{}",
            household_name(sim, l.landlord),
            l.floor_m2,
            l.rent_kg,
            if l.unpaid > 0 { ", a month behind" } else { "" }
        ));
    }
    for l in letting.letting(household) {
        out.push(format!(
            "lets {:.0} m² to {}",
            l.floor_m2,
            household_name(sim, l.lodger)
        ));
    }
    out
}

/// A `Response` describing one person, living or dead, with up to `decisions` receipts.
pub fn person_response(sim: &Sim, id: u64, decisions: u32) -> Result<Vec<u8>, QueryError> {
    let pid =
        PermanentId::from_raw(id).ok_or_else(|| QueryError("person 0 does not exist".into()))?;
    let pop = &sim.people;
    let record = pop
        .records
        .get(&pid)
        .ok_or_else(|| QueryError(format!("nobody with id {id} ever lived here")))?;
    let living = pop.person(pid);
    let now = sim.now();
    let mut fbb = FlatBufferBuilder::new();
    let name = fbb.create_string(&record.given);
    let (died_minute, cause) = match record.died {
        // A disease's death names the disease (M6a slice AZ): the observer's truth.
        Some((at, civ_agents::Cause::Disease)) => (
            at.minutes(),
            pop.sickness
                .of(pid)
                .filter(|e| {
                    e.ended
                        .is_some_and(|(_, o)| o == civ_agents::sickness::Outcome::Died)
                })
                .filter_map(|e| sim.rules.catalog.diseases.get(usize::from(e.disease)))
                .last()
                .map_or_else(|| "disease".to_owned(), |d| d.name.to_lowercase()),
        ),
        Some((at, cause)) => (at.minutes(), cause.label().to_owned()),
        None => (0, String::new()),
    };
    let cause = fbb.create_string(&cause);
    let origin = fbb.create_string(match record.origin {
        Origin::Founder => "one of the founding band",
        Origin::Born => "born here",
        Origin::Spawned => "brought by the observer",
    });
    let age_at = record.died.map(|(at, _)| at).or(record.left).unwrap_or(now);
    let age =
        (age_at.minutes() - record.born.minutes()) as f64 / civ_core::time::MINUTES_PER_YEAR as f64;
    let kin: Vec<_> = kin(sim, pid)
        .into_iter()
        .map(|(k, relation)| {
            let name = fbb.create_string(&pop.name_of(k));
            let relation = fbb.create_string(relation);
            wire::KinLink::create(
                &mut fbb,
                &wire::KinLinkArgs {
                    id: k.get(),
                    name: Some(name),
                    relation: Some(relation),
                    alive: pop.person(k).is_some(),
                    left: pop.records.get(&k).is_some_and(|r| r.left.is_some()),
                },
            )
        })
        .collect();
    let kin = fbb.create_vector(&kin);

    let mut args = wire::PersonInfoArgs {
        id,
        name: Some(name),
        sex: sex(record.sex),
        born_minute: record.born.minutes(),
        age_years: age as f32,
        alive: living.is_some(),
        died_minute,
        cause: Some(cause),
        origin: Some(origin),
        kin: Some(kin),
        left_minute: record.left.map_or(0, |t| t.minutes()),
        ..Default::default()
    };
    let pos;
    if let Some(p) = living {
        let params = &sim.rules.people;
        let mut settled = p.clone();
        population::settle(&mut settled, now, params);
        let household = pop.household(p.household);
        let home = household.map_or(p.pos, |h| h.home);
        let settlement = household.and_then(|h| h.settlement);
        let settlement_name = settlement
            .and_then(|s| sim.land.settlements.iter().find(|x| x.id == s))
            .map(|s| fbb.create_string(&s.name));
        let doing = fbb.create_string(&doing(sim, p));
        let goods = &sim.rules.catalog.goods;
        let stores = household.map(|h| population::stores_now(h, now, params, goods));
        let (food_days, ready_days, water_days, fuel_days) = match (household, &stores) {
            (Some(h), Some(stores)) => {
                let members = h.members.len().max(1);
                let kcal_day = members as f64 * params.household.daily_kcal_per_person;
                let litres_day = members as f64 * h.water_use(&params.household);
                let fuel_day = population::fuel_per_day(params, members, now.day_index());
                (
                    stock_kcal(stores, goods) / kcal_day.max(1.0),
                    food_kcal(stores, goods).0 / kcal_day.max(1.0),
                    h.water_at_time(now, litres_day) / litres_day.max(1e-6),
                    fuel_kg(stores, goods) / fuel_day.max(1e-6),
                )
            }
            _ => (0.0, 0.0, 0.0, 0.0),
        };
        let skills: Vec<wire::SkillLine> = p
            .skills
            .iter()
            .map(|&(k, level)| wire::SkillLine::new(k, level))
            .collect();
        let skills = fbb.create_vector(&skills);
        let knows = super::knowledge::know_lines(&mut fbb, sim, p);
        let taste = household.map(|h| fbb.create_string(&taste_words(sim, h)));
        let lines: Vec<wire::StoreLine> = stores
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, kg)| **kg >= 0.05)
            .map(|(g, kg)| wire::StoreLine::new(g as u16, *kg as f32))
            .collect();
        let lines = fbb.create_vector(&lines);
        let take = decisions.min(MAX_DECISIONS_PER_QUERY) as usize;
        let receipts: Vec<_> = p
            .receipts
            .iter()
            .rev()
            .take(take)
            .map(|r| decision(&mut fbb, sim, (home, p.household), r))
            .collect();
        let receipts = fbb.create_vector(&receipts);
        let t = &p.traits;
        let traits = fbb.create_vector(&[
            t.openness,
            t.conscientiousness,
            t.extraversion,
            t.agreeableness,
            t.neuroticism,
            t.risk,
        ]);
        pos = vec2(p.position_at(now.minutes() as f64));
        args.household = p.household.get();
        args.settlement = settlement.map_or(0, PermanentId::get);
        args.settlement_name = settlement_name;
        args.activity = p.act.def;
        args.doing = Some(doing);
        args.since_minute = p.act.started.minutes();
        args.until_minute = p.act.step_ends.minutes();
        args.hunger = population::hunger(p, now, params) as f32;
        args.sleep_pressure = settled.sleep_pressure;
        args.loneliness = 1.0 - settled.relatedness;
        args.energy_kcal = settled.energy_kcal;
        let carried = p.carrying.good.and_then(|g| goods.get(usize::from(g)));
        args.carry_food_kcal = carried.map_or(0.0, |g| {
            if g.purpose == GoodUse::Food {
                (f64::from(p.carrying.kg) * g.kcal_per_kg) as f32
            } else {
                0.0
            }
        });
        args.carry_good = p.carrying.good.map_or(-1, i32::from);
        args.carry_kg = p.carrying.kg;
        args.carry_water_l = p.carrying.water_l;
        args.household_food_days = food_days as f32;
        args.household_ready_days = ready_days as f32;
        args.skills = Some(skills);
        args.knows = Some(knows);
        args.household_taste = taste;
        args.household_admired = household
            .and_then(|h| h.admired)
            .map_or(0, PermanentId::get);
        args.household_water_days = water_days as f32;
        args.household_fuel_days = fuel_days as f32;
        let notes: Vec<_> = family_notes(sim, p)
            .iter()
            .map(|n| fbb.create_string(n))
            .collect();
        let notes = fbb.create_vector(&notes);
        args.stores = Some(lines);
        args.decisions = Some(receipts);
        args.traits = Some(traits);
        args.pos = Some(&pos);
        args.partner = p.partner.map_or(0, PermanentId::get);
        args.family = Some(notes);
        args.ties = Some(super::standing::tie_lines(&mut fbb, sim, p));
        args.standing = pop
            .standing
            .of(p.id)
            .map(|r| super::standing::standing_line(&mut fbb, sim, r));
        args.grievances = Some(super::word::grievance_lines(&mut fbb, sim, p));
        args.heard = Some(super::word::heard_lines(&mut fbb, sim, p));
        args.positions = Some(super::word::position_lines(&mut fbb, sim, p));
        args.norms = Some(super::word::norm_lines(&mut fbb, sim, p));
        args.values = Some(super::word::value_lines(&mut fbb, sim, p));
        args.ideologies = Some(super::word::ideology_lines(&mut fbb, sim, p));
        args.faction = super::word::faction_line(&mut fbb, sim, p);
        args.news = Some(super::word::news_lines(&mut fbb, sim, p));
        args.influences = Some(super::word::influence_lines(&mut fbb, sim, p));
        let residence: Vec<_> = residence_words(sim, p.id)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        args.residence = Some(fbb.create_vector(&residence));
        let places: Vec<_> = places_words(sim, p.household)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        args.places = Some(fbb.create_vector(&places));
        let reports: Vec<_> = super::markets::reports_words(sim, p.household)
            .iter()
            .map(|w| fbb.create_string(w))
            .collect();
        args.reports = Some(fbb.create_vector(&reports));
        args.errand = Some(fbb.create_string(&super::markets::errand_words(sim, p.household)));
        args.seen_away = Some(fbb.create_string(&seen_away_words(sim, p.id)));
        args.water = Some(fbb.create_string(&super::water::water_words(sim, p.household)));
        args.light = Some(fbb.create_string(&light_words(sim, p.id)));
        args.care = Some(fbb.create_string(&super::sickness::care_words(sim, p.id)));
        args.teaching = Some(fbb.create_string(&teaching_words(sim, p.id)));
        args.letters = Some(fbb.create_string(&letters_words(sim, p.id)));
    }
    // A person's infections, living or not (wire 1.62).
    let sickness: Vec<_> = super::sickness::sickness_words(sim, pid)
        .iter()
        .map(|w| fbb.create_string(w))
        .collect();
    args.sickness = Some(fbb.create_vector(&sickness));
    // The sources they hold sicken (wire 1.63).
    let suspicions: Vec<_> = super::sickness::suspicion_words(sim, pid)
        .iter()
        .map(|w| fbb.create_string(w))
        .collect();
    args.suspicions = Some(fbb.create_vector(&suspicions));
    let body = wire::PersonInfo::create(&mut fbb, &args);
    Ok(response(fbb, wire::ResponseBody::PersonInfo, body))
}

/// Person `id`'s letters, in words (wire 1.106, M10c slice EC): each record they wrote and the
/// household keeping it; each receipt they wrote for goods kept for another (M10d slice EG), and
/// whether as notes (slice EH); a
/// record they are writing, and how far; and each technique they read toward from a record, with
/// its writer and the hours read of those it takes, or only heard of there. Empty for none.
pub fn letters_words(sim: &Sim, id: PermanentId) -> String {
    let people = sim.people();
    let rules = sim.rules();
    let techniques = &rules.catalog.techniques;
    let name = |t: u16| {
        techniques
            .get(usize::from(t))
            .map_or_else(|| "a craft".to_owned(), |d| d.name.to_lowercase())
    };
    let keeper = |h: PermanentId| {
        people
            .household(h)
            .and_then(|x| x.members.first().copied())
            .map_or_else(|| "nobody".to_owned(), |m| people.name_of(m))
    };
    let mut said: Vec<String> = people
        .writings
        .iter()
        .filter(|r| r.maker == id)
        .map(|r| {
            format!(
                "wrote a record of {}, kept by {}'s household",
                name(r.technique),
                keeper(r.holder)
            )
        })
        .collect();
    for k in &people.kept {
        if k.receipt.is_some_and(|(by, _)| by == id) {
            said.push(format!(
                "wrote a receipt for goods {}'s household left with {}'s household{}",
                keeper(k.depositor),
                keeper(k.keeper),
                if k.noted { ", as notes" } else { "" }
            ));
        }
    }
    let record_h = rules.people.knowledge.record_h;
    for w in people.written.iter().filter(|w| w.person == id) {
        said.push(format!(
            "writing a record of {}, {:.0} of {record_h:.0} hours",
            name(w.technique),
            f64::from(w.minutes) / 60.0
        ));
    }
    if let Some(p) = people.person(id) {
        for k in &p.knows {
            let civ_agents::person::KnowSource::Read(maker) = k.source else {
                continue;
            };
            let Some(d) = techniques.get(usize::from(k.technique)) else {
                continue;
            };
            let what = d.name.to_lowercase();
            let by = people.name_of(maker);
            said.push(if k.known {
                format!("learnt {what} from a record {by} wrote")
            } else if d.codified > 0.0 {
                format!(
                    "reading {what} from a record {by} wrote, {:.0} of {:.0} hours",
                    f64::from(k.hours) * d.codified,
                    d.learn_h * d.codified
                )
            } else {
                format!("heard of {what} in a record {by} wrote")
            });
        }
    }
    let mut s = said.join("; ");
    if let Some(c) = s.get(..1) {
        s = c.to_uppercase() + &s[1..];
    }
    s
}

/// Person `id`'s teaching and study, in words (wire 1.105, M10c slice EB): for each technique they
/// teach, whom and the fee, and today's hours taught; for what they are enrolled to learn, with
/// whom, the hours toward it of those it takes, and today's hours studied. Empty for neither.
pub fn teaching_words(sim: &Sim, id: PermanentId) -> String {
    let people = sim.people();
    let techniques = &sim.rules().catalog.techniques;
    let name = |t: u16| {
        techniques
            .get(usize::from(t))
            .map_or_else(|| "a craft".to_owned(), |d| d.name.to_lowercase())
    };
    let fee = |kcal: f32| {
        if kcal > 0.0 {
            format!("for {kcal:.0} kcal of food a day")
        } else {
            "asking nothing".to_owned()
        }
    };
    let hours_today = |list: &[civ_agents::knowledge::Session], t: u16| {
        let h = f64::from(civ_agents::knowledge::tallied(list, id, t)) / 60.0;
        if h > 0.0 {
            format!("; {h:.0} hours today")
        } else {
            String::new()
        }
    };
    let mut said = Vec::new();
    let mut taught: Vec<u16> = people
        .enrolments
        .iter()
        .filter(|e| e.teacher == id)
        .map(|e| e.technique)
        .collect();
    taught.sort_unstable();
    taught.dedup();
    for t in taught {
        let learners: Vec<String> = people
            .enrolments
            .iter()
            .filter(|e| e.teacher == id && e.technique == t)
            .map(|e| format!("{} ({})", people.name_of(e.learner), fee(e.fee_kcal)))
            .collect();
        said.push(format!(
            "Teaches {} to {}{}",
            name(t),
            learners.join(", "),
            hours_today(&people.taught, t)
        ));
    }
    for e in people.enrolments.iter().filter(|e| e.learner == id) {
        let toward = people
            .person(id)
            .and_then(|p| p.knows.iter().find(|k| k.technique == e.technique))
            .map_or(0.0, |k| f64::from(k.hours));
        let whole = techniques
            .get(usize::from(e.technique))
            .map_or(0.0, |d| d.learn_h);
        said.push(format!(
            "Studies {} with {}, {toward:.0} of {whole:.0} hours, {}{}",
            name(e.technique),
            people.name_of(e.teacher),
            fee(e.fee_kcal),
            hours_today(&people.studied, e.technique)
        ));
    }
    said.join("; ")
}

/// Person `id`'s light, in words (wire 1.99, M10b slice DU): at work by lamplight now, or the
/// lamp their household holds and the hours of light its fuel gives; empty when it holds none.
/// A home lit by gas from a main now (wire 1.103, M10b slice DY): at work by gaslight, or lit by
/// gas until its burners go out; and by the wire (wire 1.107, M11a), by electric light.
pub fn light_words(sim: &Sim, id: PermanentId) -> String {
    let people = sim.people();
    let (catalog, params, now) = (&sim.rules().catalog, &sim.rules().people, sim.now());
    let Some(p) = people.person(id) else {
        return String::new();
    };
    let Some(x) = people.household(p.household) else {
        return String::new();
    };
    if let Some(until) = sim.land.gas.lit_until(x.id, now.minutes()) {
        let working = people.lamp_lit_home(catalog, params, &sim.land, now, x.id)
            && !p.asleep
            && civ_agents::population::Population::works_by_lamp(catalog, p);
        return if working {
            "Working by gaslight at home".to_owned()
        } else {
            format!(
                "Their home is lit by gas until {}",
                super::ways::clock_at(until)
            )
        };
    }
    if let Some(until) = sim.land.wires.lit_until(x.id, now.minutes()) {
        let working = people.lamp_lit_home(catalog, params, &sim.land, now, x.id)
            && !p.asleep
            && civ_agents::population::Population::works_by_lamp(catalog, p);
        return if working {
            "Working by electric light at home".to_owned()
        } else {
            format!(
                "Their home is lit by the wire until {}",
                super::ways::clock_at(until)
            )
        };
    }
    let goods = &catalog.goods;
    // The lamp it holds and its fuel, or a whole lamp with none to burn.
    let (lamp, hours) = match civ_agents::population::lamp_of(goods, &x.stores) {
        Some((lamp, _, _, minutes)) => (lamp, minutes / 60.0),
        None => match goods.iter().enumerate().find(|&(g, d)| {
            d.light.is_some_and(|l| !l.piped)
                && civ_agents::make::in_use(goods, g, x.stores.get(g).copied().unwrap_or(0.0))
                    >= civ_agents::decide::MIN_TOOL
        }) {
            Some((lamp, _)) => (lamp, 0.0),
            None => return String::new(),
        },
    };
    let name = goods[lamp].name.to_lowercase();
    let fuel = goods[lamp]
        .light
        .as_ref()
        .and_then(|l| l.fuel)
        .and_then(|f| goods.get(f))
        .map_or_else(|| "fuel".to_owned(), |d| d.name.to_lowercase());
    let left = if hours >= 2.0 {
        format!("{fuel} for {} hours", hours.floor())
    } else if hours >= 1.0 {
        format!("{fuel} for an hour")
    } else if hours > 0.0 {
        format!("{fuel} for less than an hour")
    } else {
        format!("no {fuel}")
    };
    let working = people.lamp_lit_home(catalog, params, &sim.land, now, x.id)
        && !p.asleep
        && civ_agents::population::Population::works_by_lamp(catalog, p);
    if working {
        format!("Working by lamplight at home; their household's {name} has {left}")
    } else {
        let of_light = if hours > 0.0 { " of light" } else { "" };
        format!("Their household holds a {name}, with {left}{of_light}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_of_time_read_plainly() {
        let day = 1440;
        assert_eq!(how_long(21 * 365 * day + 100), "21 years");
        assert_eq!(how_long(400 * day), "a year");
        assert_eq!(how_long(100 * day), "3 months");
        assert_eq!(how_long(20 * day), "2 weeks");
        assert_eq!(how_long(3 * day), "a few days");
        assert_eq!(in_about(day), "any day now");
        assert_eq!(in_about(90 * day), "in about 3 months");
    }

    #[test]
    fn bearings_read_like_a_compass() {
        assert_eq!(bearing((0.0, 0.0), (600.0, 0.0)), "600 m east");
        assert_eq!(bearing((0.0, 0.0), (0.0, -42.0)), "40 m north");
        assert_eq!(bearing((0.0, 0.0), (-1500.0, 1500.0)), "2.1 km south-west");
        assert_eq!(bearing((10.0, 10.0), (11.0, 9.0)), "here");
    }
}
