//! Read-only fire help: today's nearby adults and water, beside recorded work.

use std::collections::BTreeSet;

use civ_agents::params::{Behavior, interpolate};
use civ_agents::population::cell_of;
use civ_core::PermanentId;
use civ_world::nav::RouteResult;

use super::{Home, OTHERS_WELL, PERIOD_DAYS};
use crate::Sim;

/// One current probe and the household's realized help over the displayed period.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Help {
    pub available: u32,
    pub nearest_min: Option<f32>,
    pub unknown_routes: u32,
    pub water: bool,
    pub fires: u32,
    pub observer: u32,
    pub helped: u32,
    pub people: u32,
    pub litres: f32,
    pub first_min: Option<f32>,
}

/// Adults awake at home who could walk here now; nobody is reserved or made willing.
pub fn help(sim: &Sim, home: &Home) -> Help {
    let mut out = Help::default();
    let now = sim.now();
    let day = now.day_index();
    let params = &sim.rules.people;
    let carrier = sim
        .rules
        .catalog
        .activities
        .iter()
        .find(|a| a.behavior == Behavior::Douse);
    let water = home
        .by_right
        .as_ref()
        .or_else(|| home.nearest.as_ref().filter(|s| s.kind != OTHERS_WELL));
    out.water = water.is_some()
        || sim.people.household(home.household).is_some_and(|h| {
            h.water_at_time(now, h.water_use(&params.household)) >= params.household.carry_water_l
        });
    for (_, p) in sim.people.people.iter() {
        let age = p.age_years(now);
        if age < 18.0
            || carrier.is_none_or(|a| age < a.min_age_years || age > a.max_age_years)
            || p.asleep
            || p.trip.is_some()
            || sim.people.sickness.ill(p.id, day)
            || sim
                .people
                .order
                .hurt_until(p.id)
                .is_some_and(|until| day <= until)
            || sim
                .rules
                .catalog
                .activities
                .get(usize::from(p.act.def))
                .is_some_and(|d| {
                    matches!(
                        d.behavior,
                        Behavior::Douse | Behavior::Strip | Behavior::Salvage | Behavior::Pull
                    )
                })
        {
            continue;
        }
        let Some(hh) = sim.people.household(p.household) else {
            continue;
        };
        if (p.pos.0 - hh.home.0).hypot(p.pos.1 - hh.home.1) > 1.0 {
            continue;
        }
        let speed = interpolate(&params.walk_speed_by_age, age).max(0.05);
        // Bound obvious distant candidates before the directed walking search.
        if f64::from((p.pos.0 - home.at.0).hypot(p.pos.1 - home.at.1))
            > params.nav.top_speed_ms() * speed * 300.0
        {
            continue;
        }
        let from = cell_of(&sim.map, p.pos);
        let to = cell_of(&sim.map, home.at);
        if !sim.nav.walkable(from) || !sim.nav.walkable(to) {
            continue;
        }
        let seconds = if from == to {
            let surface = params.nav.offtrail_factor
                + (1.0 - params.nav.offtrail_factor) * f64::from(sim.land.wear.factor(from));
            Some(
                f64::from((p.pos.0 - home.at.0).hypot(p.pos.1 - home.at.1))
                    / (params.nav.tobler_ms(0.0) * surface * speed),
            )
        } else {
            match sim.nav.route(
                &sim.map.elevation,
                from,
                to,
                &|c| sim.land.wear.factor(c),
                50_000,
            ) {
                RouteResult::Found(route) => Some(f64::from(route.total_seconds()) / speed),
                RouteResult::Unreachable => None,
                RouteResult::BudgetExhausted => {
                    out.unknown_routes += 1;
                    None
                }
            }
        };
        if let Some(seconds) = seconds.filter(|&s| s <= 300.0) {
            out.available += 1;
            let minutes = (seconds / 60.0) as f32;
            out.nearest_min = Some(out.nearest_min.map_or(minutes, |m| m.min(minutes)));
        }
    }
    let since = day - PERIOD_DAYS + 1;
    let fires = &sim.people.fires;
    let in_period = |root: u32| {
        fires.get(root).is_some_and(|f| {
            f.began.day_index() >= since
                || f.ended.is_none()
                || f.episode
                    .and_then(|id| fires.episodes.iter().find(|c| c.id == id))
                    .is_some_and(|c| c.ended.is_none_or(|at| at.day_index() >= since))
        })
    };
    let ours: BTreeSet<PermanentId> = sim
        .land
        .buildings
        .iter()
        .filter(|b| b.household == home.household)
        .map(|b| b.id)
        .collect();
    let mut reached: BTreeSet<u32> = fires
        .weighed
        .iter()
        .filter(|w| ours.contains(&w.building) && in_period(w.fire))
        .map(|w| w.fire)
        .collect();
    reached.extend(
        fires
            .list
            .iter()
            .filter(|f| f.household == home.household && in_period(fires.root(f.id)))
            .map(|f| fires.root(f.id)),
    );
    let mut helped = BTreeSet::new();
    let mut people = BTreeSet::new();
    let mut add = |fire: u32, building: PermanentId, who: PermanentId, at: civ_core::SimTime| {
        if !ours.contains(&building) || at.day_index() < since || at > now {
            return false;
        }
        let root = fires.root(fire);
        reached.insert(root);
        helped.insert(root);
        people.insert(who);
        if let Some(f) = fires.get(root) {
            let minutes = (at.minutes() - f.began.minutes()).max(0) as f32;
            out.first_min = Some(out.first_min.map_or(minutes, |m| m.min(minutes)));
        }
        true
    };
    for t in &fires.thrown {
        if add(t.fire, t.building, t.person, t.at) {
            out.litres += t.litres;
        }
    }
    for t in &fires.stripped {
        add(t.fire, t.building, t.person, t.at);
    }
    for t in &fires.salvaged {
        add(t.fire, t.building, t.person, t.at);
    }
    for t in &fires.pulled {
        for &who in &t.people {
            add(t.fire, t.building, who, t.at);
        }
    }
    out.fires = reached.len() as u32;
    out.observer = reached
        .iter()
        .filter(|&&root| {
            fires.list.iter().any(|f| {
                fires.root(f.id) == root
                    && matches!(f.kindled, civ_agents::fire::Kindled::Observer { .. })
            })
        })
        .count() as u32;
    out.helped = helped.len() as u32;
    out.people = people.len() as u32;
    out
}

pub fn words(h: &Help) -> Vec<String> {
    let water = if h.water {
        "water within reach by right or a stored load"
    } else {
        "no water within reach by right and no stored load"
    };
    let walk = h.nearest_min.map_or_else(
        || "no adult within the probe".to_owned(),
        |m| format!("nearest walk {m:.1} min"),
    );
    let mut rows = vec![format!(
        "Now: {} adults awake at home within five minutes' walk; {walk}; {water}. Willingness and useful work are not forecast.",
        h.available
    )];
    if h.unknown_routes > 0 {
        rows.push(format!(
            "{} route searches were inconclusive; those adults are not counted as unreachable.",
            h.unknown_routes
        ));
    }
    rows.push(if h.fires == 0 {
        format!("Did come: no fire episode reached this household's roofs in the past {PERIOD_DAYS} days.")
    } else {
        let first = h.first_min.map_or_else(|| "no useful work recorded".to_owned(), |m| format!("earliest useful work {m:.1} min after an episode began"));
        format!("Did come: {} of {} fire episodes received recorded work at this household's roofs; {} people, {:.1} L applied; {first}.", h.helped, h.fires, h.people, h.litres)
    });
    if h.observer > 0 {
        rows.push(format!("{} of those episode exposures include an observer-set fire; this does not attribute every loss to that input.", h.observer));
    }
    rows
}
