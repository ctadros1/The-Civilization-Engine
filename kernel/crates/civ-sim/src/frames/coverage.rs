//! The Water lens (wire 1.66, M6a slice BB, step three; the observer brief §1.3; research 12-07
//! §1.8 and 15-01 §3.1): for each home in scope, the water it *could reach* (walking minutes on
//! the walking grid, on the paths as last surveyed, to the nearest source of any kind and to the
//! nearest its people may draw at by right) and where its people *did come* for water (the
//! sources they drew at on each of the past 30 days, from their own record). Access is kept apart
//! from permission, and a home with no water within the search says so. Derived when asked,
//! never saved; nothing in the world reads it.

pub mod fire;

use std::collections::BTreeMap;

use civ_agents::person::Household;
use civ_agents::population::{cell_centre, cell_of};
use civ_agents::uses::Place;
use civ_core::PermanentId;
use civ_land::wells::Well;
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;
use civ_world::nav::TravelField;
use civ_world::{WATER_LAKE, WATER_RIVER, WorldMap};

use super::people::household_name;
use super::response;
use super::water::{spring_cell, standing, whose_well};
use crate::Sim;

/// The Water lens.
pub const WATER: u8 = 1;
/// Nearby adults and water now, beside recorded fire work.
pub const FIRE_HELP: u8 = 2;
/// The days *did come* looks back over, today one of them.
pub const PERIOD_DAYS: i64 = 30;
/// The walks the search for water tries in turn, seconds: most homes have water within two
/// minutes, so most searches stay small; none looks beyond an hour.
const SEARCH_S: [f32; 3] = [120.0, 600.0, 3_600.0];
/// The walks *could reach* is counted by, minutes: within each, and beyond the last.
pub const BANDS_MIN: [f32; 3] = [1.0, 5.0, 15.0];

/// How a home stands to a source (wire codes): its own well.
pub const OWN_WELL: u8 = 1;
/// A well its people may draw at by right: their polity's or their kin's.
pub const WELL_BY_RIGHT: u8 = 2;
/// Another's well, where they may draw only when short of water.
pub const OTHERS_WELL: u8 = 3;
/// A spring.
pub const SPRING: u8 = 4;
/// The edge of a river or lake.
pub const EDGE: u8 = 5;

/// A source as one home stands to it.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub kind: u8,
    pub name: String,
    pub at: (f32, f32),
    /// Walking minutes from the home; `None` when not within the search.
    pub minutes: Option<f32>,
    /// For *did come*: the days of the period its people drew there.
    pub days: u16,
}

/// One home and its water.
#[derive(Clone, Debug, PartialEq)]
pub struct Home {
    pub household: PermanentId,
    pub settlement: Option<PermanentId>,
    pub at: (f32, f32),
    /// Could reach: the nearest source of any kind.
    pub nearest: Option<Source>,
    /// The nearest its people may draw at by right, when that is not the nearest.
    pub by_right: Option<Source>,
    /// Did come: where its people drew water in the period, most days first.
    pub drew: Vec<Source>,
}

/// Whether `cell` is at the edge of a river or lake (or in a river's shallows), where the
/// kernel's people draw water.
fn edge(map: &WorldMap, cell: usize) -> bool {
    let (w, h) = (map.width as i64, map.height as i64);
    let (x, y) = ((cell as i64) % w, (cell as i64) / w);
    map.water[cell] == WATER_RIVER
        || civ_world::grid::D8.iter().any(|&(dx, dy)| {
            let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
            nx >= 0
                && ny >= 0
                && nx < w
                && ny < h
                && matches!(map.water[(ny * w + nx) as usize], WATER_RIVER | WATER_LAKE)
        })
}

/// How household `x` stands to well `w`.
fn well_kind(x: &Household, w: &Well, right: &dyn Fn(&Well) -> bool) -> u8 {
    if w.polity.is_none() && w.household == x.id {
        OWN_WELL
    } else if right(w) {
        WELL_BY_RIGHT
    } else {
        OTHERS_WELL
    }
}

fn well_name(sim: &Sim, w: &Well) -> String {
    format!("{} well", whose_well(sim, w))
}

/// The water household `x`'s home could reach: the nearest source of any kind and the nearest
/// its people may draw at by right, each with its walk, from the smallest search that finds the
/// one by right; and the walking field searched last.
fn could_reach(
    sim: &Sim,
    wells: &[&Well],
    springs: &[u32],
    x: &Household,
    right: &dyn Fn(&Well) -> bool,
) -> (Option<Source>, Option<Source>, TravelField) {
    let map = &sim.map;
    let from = cell_of(map, x.home);
    let mut field = TravelField::default();
    let mut found = (None, None);
    for &limit in &SEARCH_S {
        field = sim
            .nav
            .travel_field(&map.elevation, from, limit, &|c| sim.land.wear.factor(c));
        let (mut any, mut ours): (Option<Source>, Option<Source>) = (None, None);
        let mut offer = |s: Source| {
            let better = |b: &Option<Source>| {
                b.as_ref().is_none_or(|b| {
                    let (m, n) = (s.minutes.unwrap_or(f32::MAX), b.minutes.unwrap_or(f32::MAX));
                    m.total_cmp(&n).then(s.kind.cmp(&b.kind)).is_lt()
                })
            };
            if s.kind != OTHERS_WELL && better(&ours) {
                ours = Some(s.clone());
            }
            if better(&any) {
                any = Some(s);
            }
        };
        let bank = field
            .iter()
            .filter(|&(c, _)| edge(map, c))
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        if let Some((cell, s)) = bank {
            offer(Source {
                kind: EDGE,
                name: "the water's edge".to_owned(),
                at: cell_centre(map, cell),
                minutes: Some(s / 60.0),
                days: 0,
            });
        }
        for &cell in springs {
            if let Some(s) = field.seconds_to(cell as usize) {
                offer(Source {
                    kind: SPRING,
                    name: "a spring".to_owned(),
                    at: cell_centre(map, cell as usize),
                    minutes: Some(s / 60.0),
                    days: 0,
                });
            }
        }
        for w in wells {
            if let Some(s) = field.seconds_to(w.cell as usize) {
                offer(Source {
                    kind: well_kind(x, w, right),
                    name: well_name(sim, w),
                    at: w.rect.centre_m(),
                    minutes: Some(s / 60.0),
                    days: 0,
                });
            }
        }
        let done = ours.is_some();
        found = (any, ours);
        if done {
            break;
        }
    }
    let (any, ours) = found;
    let ours = ours.filter(|o| any.as_ref() != Some(o));
    (any, ours, field)
}

/// Where household `x`'s people drew water on the days of the period, from their own record:
/// each well and spring apart, the water's edge as one; most days first.
fn did_come(
    sim: &Sim,
    x: &Household,
    right: &dyn Fn(&Well) -> bool,
    field: &TravelField,
) -> Vec<Source> {
    let today = sim.now().day_index();
    let map = &sim.map;
    // By the source: its kind, name, place and walk, the days drawn there, and the most days at
    // any one cell of it.
    let mut by: BTreeMap<(u8, u64), (Source, u64, u32)> = BTreeMap::new();
    let mut add = |cell: u32, days: u64| {
        if days == 0 {
            return;
        }
        let well = sim
            .land
            .wells
            .list
            .iter()
            .filter(|w| w.cell == cell)
            .max_by_key(|w| (w.is_open(), w.id));
        let (kind, key, name, at) = match well {
            Some(w) => {
                let kind = well_kind(x, w, right);
                (kind, w.id.get(), well_name(sim, w), w.rect.centre_m())
            }
            None if spring_cell(sim, cell) => (
                SPRING,
                u64::from(cell),
                "a spring".to_owned(),
                cell_centre(map, cell as usize),
            ),
            None => (
                EDGE,
                0,
                "the water's edge".to_owned(),
                cell_centre(map, cell as usize),
            ),
        };
        let minutes = field.seconds_to(cell as usize).map(|s| s / 60.0);
        let e = by.entry((kind, key)).or_insert_with(|| {
            (
                Source {
                    kind,
                    name,
                    at,
                    minutes,
                    days: 0,
                },
                0,
                0,
            )
        });
        e.1 |= days;
        // The water's edge is placed at the stretch of it drawn at most.
        if kind == EDGE && days.count_ones() > e.2 {
            e.0.at = at;
            e.0.minutes = minutes;
            e.2 = days.count_ones();
        }
    };
    for u in sim.people.uses.households.get(&x.id).into_iter().flatten() {
        if let Place::Source(cell) = u.place {
            add(cell, u.worked_in(today, PERIOD_DAYS));
        }
    }
    // Draws not yet folded into the record: today's, and any of a day before.
    for w in &sim.people.uses.today {
        if let (true, Place::Source(cell)) = (w.household == x.id, w.place) {
            let k = today - w.day;
            if (0..PERIOD_DAYS).contains(&k) {
                add(cell, 1 << k);
            }
        }
    }
    let mut out: Vec<Source> = by
        .into_values()
        .map(|(mut s, days, _)| {
            s.days = days.count_ones() as u16;
            s
        })
        .collect();
    out.sort_by(|a, b| b.days.cmp(&a.days).then(a.kind.cmp(&b.kind)));
    out
}

/// The homes of settlement `settlement` (every settlement for `None`), each with the water it
/// could reach and did come for: households with people, in household order.
pub fn water_homes(sim: &Sim, settlement: Option<PermanentId>) -> Vec<Home> {
    let wells: Vec<&Well> = sim
        .land
        .wells
        .list
        .iter()
        .filter(|w| w.is_open() && standing(sim, w).0 > 0.0)
        .collect();
    let params = &sim.rules.land.water;
    let springs: Vec<u32> = (0..sim.land.water.aquifer.len())
        .filter_map(|p| sim.land.water.spring_at(params, p))
        .map(|s| s.cell)
        .collect();
    let mut homes: Vec<&Household> = sim
        .people
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty() && settlement.is_none_or(|s| x.settlement == Some(s)))
        .collect();
    homes.sort_by_key(|x| x.id);
    homes
        .into_iter()
        .map(|x| {
            let right = sim.people.by_right(x);
            let (nearest, by_right, field) = could_reach(sim, &wells, &springs, x, &right);
            let drew = did_come(sim, x, &right, &field);
            Home {
                household: x.id,
                settlement: x.settlement,
                at: x.home,
                nearest,
                by_right,
                drew,
            }
        })
        .collect()
}

/// "less than a minute's walk", "1 minute's walk", "12 minutes' walk".
fn walk_words(minutes: Option<f32>) -> String {
    match minutes {
        None => "beyond the search".to_owned(),
        Some(m) if m < 1.0 => "less than a minute's walk".to_owned(),
        Some(m) if m.round() as i64 == 1 => "1 minute's walk".to_owned(),
        Some(m) => format!("{:.0} minutes' walk", m),
    }
}

fn days_words(n: u16) -> String {
    if n == 1 {
        "1 day".to_owned()
    } else {
        format!("{n} days")
    }
}

/// A home's water in words: what it could reach, and where its people did come.
pub fn home_words(h: &Home) -> Vec<String> {
    let reach = match (&h.nearest, &h.by_right) {
        (None, _) => {
            "Could reach: no water within an hour's walk on the paths as they are".to_owned()
        }
        (Some(n), right) => {
            let mut s = format!("Could reach: {}, {}", n.name, walk_words(n.minutes));
            if n.kind == OTHERS_WELL {
                s.push_str(" (another's: its people may draw there only when short of water)");
                match right {
                    Some(r) => s.push_str(&format!(
                        "; by right, {}, {}",
                        r.name,
                        walk_words(r.minutes)
                    )),
                    None => s.push_str("; nothing it may draw at by right within an hour's walk"),
                }
            }
            s
        }
    };
    let came = if h.drew.is_empty() {
        format!("Did come: its people's record holds no water drawn in the past {PERIOD_DAYS} days")
    } else {
        let parts: Vec<String> = h
            .drew
            .iter()
            .map(|s| format!("at {} on {}", s.name, days_words(s.days)))
            .collect();
        format!(
            "Did come: in the past {PERIOD_DAYS} days its people drew {}",
            parts.join(", ")
        )
    };
    vec![reach, came]
}

/// The counts of each layer in words, over `homes`.
pub fn summary(homes: &[Home]) -> Vec<String> {
    let n = homes.len();
    let homes_words = |k: usize| {
        if k == 1 {
            "1 home".to_owned()
        } else {
            format!("{k} homes")
        }
    };
    let mut bands = [0usize; BANDS_MIN.len() + 2];
    for h in homes {
        let at = match h.nearest.as_ref().and_then(|s| s.minutes) {
            None => BANDS_MIN.len() + 1,
            Some(m) => BANDS_MIN
                .iter()
                .position(|&b| m < b)
                .unwrap_or(BANDS_MIN.len()),
        };
        bands[at] += 1;
    }
    let others = homes
        .iter()
        .filter(|h| h.nearest.as_ref().is_some_and(|s| s.kind == OTHERS_WELL))
        .count();
    let mut reach = format!(
        "Could reach: of {}, {} have water within a minute's walk, {} within 5 minutes, {} \
         within 15, {} farther within an hour, {} none within an hour",
        homes_words(n),
        bands[0],
        bands[1],
        bands[2],
        bands[3],
        bands[4]
    );
    if others > 0 {
        reach.push_str(&format!(
            "; for {}, the nearest is another's well",
            homes_words(others)
        ));
    }
    let came = |kind: u8| {
        homes
            .iter()
            .filter(|h| h.drew.iter().any(|s| s.kind == kind))
            .count()
    };
    let none = homes.iter().filter(|h| h.drew.is_empty()).count();
    let mut did = format!(
        "Did come: in the past {PERIOD_DAYS} days, of {}, {} drew at their own well, {} at a \
         well by right, {} at another's well, {} at a spring, {} at the water's edge",
        homes_words(n),
        came(OWN_WELL),
        came(WELL_BY_RIGHT),
        came(OTHERS_WELL),
        came(SPRING),
        came(EDGE)
    );
    if none > 0 {
        did.push_str(&format!("; {} have no draw recorded", homes_words(none)));
    }
    vec![reach, did]
}

fn put_source<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    s: &Source,
) -> WIPOffset<wire::CoverageSource<'a>> {
    let name = fbb.create_string(&s.name);
    wire::CoverageSource::create(
        fbb,
        &wire::CoverageSourceArgs {
            name: Some(name),
            at: Some(&wire::Vec2::new(s.at.0, s.at.1)),
            kind: s.kind,
            minutes: s.minutes.unwrap_or(-1.0),
            days: s.days,
        },
    )
}

/// A `Response` with lens `lens` over settlement `settlement`'s homes (every settlement for
/// `None`), or `None` for a lens there is not.
pub fn coverage_response(sim: &Sim, settlement: Option<PermanentId>, lens: u8) -> Option<Vec<u8>> {
    if lens == super::traffic::TRAFFIC {
        return Some(super::traffic::lens_response(sim, settlement));
    }
    if !matches!(lens, WATER | FIRE_HELP) {
        return None;
    }
    let homes = water_homes(sim, settlement);
    let help: BTreeMap<_, _> = if lens == FIRE_HELP {
        homes
            .iter()
            .map(|h| (h.household, fire::help(sim, h)))
            .collect()
    } else {
        BTreeMap::new()
    };
    let mut fbb = FlatBufferBuilder::new();
    let rows: Vec<_> = homes
        .iter()
        .map(|h| {
            let name = fbb.create_string(&household_name(sim, h.household));
            let nearest = h.nearest.as_ref().map(|s| put_source(&mut fbb, s));
            let by_right = h.by_right.as_ref().map(|s| put_source(&mut fbb, s));
            let drew: Vec<_> = h.drew.iter().map(|s| put_source(&mut fbb, s)).collect();
            let drew = fbb.create_vector(&drew);
            let measured = help.get(&h.household);
            let fire_help = measured.map(|p| {
                wire::FireHelp::create(
                    &mut fbb,
                    &wire::FireHelpArgs {
                        adults_now: p.available,
                        nearest_walk_min: p.nearest_min.unwrap_or(-1.0),
                        unknown_routes: p.unknown_routes,
                        water_available: p.water,
                        fire_episodes: p.fires,
                        helped_episodes: p.helped,
                        helpers: p.people,
                        water_l: p.litres,
                        first_work_min: p.first_min.unwrap_or(-1.0),
                        observer_episodes: p.observer,
                    },
                )
            });
            let lines = measured.map_or_else(|| home_words(h), fire::words);
            let words: Vec<_> = lines.iter().map(|l| fbb.create_string(l)).collect();
            let words = fbb.create_vector(&words);
            wire::CoverageHome::create(
                &mut fbb,
                &wire::CoverageHomeArgs {
                    household: h.household.get(),
                    name: Some(name),
                    at: Some(&wire::Vec2::new(h.at.0, h.at.1)),
                    settlement: h.settlement.map_or(0, PermanentId::get),
                    nearest,
                    by_right,
                    drew: Some(drew),
                    words: Some(words),
                    fire_help,
                },
            )
        })
        .collect();
    let rows = fbb.create_vector(&rows);
    let place = settlement
        .and_then(|s| sim.land.settlements.iter().find(|x| x.id == s))
        .map_or_else(|| "Every settlement".to_owned(), |s| s.name.clone());
    let scope = format!(
        "{place}: {}",
        if homes.len() == 1 {
            "1 home".to_owned()
        } else {
            format!("{} homes", homes.len())
        }
    );
    let title = fbb.create_string(if lens == WATER {
        "Water"
    } else {
        "Help at a fire"
    });
    let scope = fbb.create_string(&scope);
    let periods = [
        if lens == WATER {
            "now, on the paths as last surveyed".to_owned()
        } else {
            "one current probe; adults awake at home, within five minutes; no brigade or forecast"
                .to_owned()
        },
        format!("the past {PERIOD_DAYS} days, today included"),
    ];
    let periods: Vec<_> = periods.iter().map(|p| fbb.create_string(p)).collect();
    let periods = fbb.create_vector(&periods);
    let denominator = fbb.create_string("homes: households with people, each at its home");
    let summaries = if lens == WATER {
        summary(&homes)
    } else {
        let adults = help.values().filter(|h| h.available > 0).count();
        let water = help.values().filter(|h| h.water).count();
        let episodes: u32 = help.values().map(|h| h.fires).sum();
        let served: u32 = help.values().map(|h| h.helped).sum();
        let observer: u32 = help.values().map(|h| h.observer).sum();
        vec![
            format!(
                "Now: of {} homes, {adults} have an adult within the probe and {water} have water by right or a stored load; willingness is not forecast.",
                homes.len()
            ),
            format!(
                "Did come: {served} of {episodes} episode-household exposures received recorded useful work; unserved exposures remain in the denominator. {observer} exposures include an observer-set fire."
            ),
        ]
    };
    let summary: Vec<_> = summaries.iter().map(|l| fbb.create_string(l)).collect();
    let summary = fbb.create_vector(&summary);
    let body = wire::Coverage::create(
        &mut fbb,
        &wire::CoverageArgs {
            lens,
            title: Some(title),
            scope: Some(scope),
            periods: Some(periods),
            denominator: Some(denominator),
            homes: Some(rows),
            limit_min: SEARCH_S[SEARCH_S.len() - 1] / 60.0,
            days: PERIOD_DAYS as u16,
            summary: Some(summary),
            places: None,
        },
    );
    Some(response(fbb, wire::ResponseBody::Coverage, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(kind: u8, minutes: Option<f32>, days: u16) -> Source {
        Source {
            kind,
            name: match kind {
                OWN_WELL => "Bo's household's well",
                OTHERS_WELL => "Ada's household's well",
                SPRING => "a spring",
                _ => "the water's edge",
            }
            .to_owned(),
            at: (0.0, 0.0),
            minutes,
            days,
        }
    }

    fn home(nearest: Option<Source>, by_right: Option<Source>, drew: Vec<Source>) -> Home {
        Home {
            household: PermanentId::from_raw(1).expect("an id"),
            settlement: None,
            at: (0.0, 0.0),
            nearest,
            by_right,
            drew,
        }
    }

    #[test]
    fn a_home_s_water_reads_in_words_with_access_apart_from_permission() {
        let own = home(
            Some(source(OWN_WELL, Some(0.3), 0)),
            None,
            vec![source(OWN_WELL, Some(0.3), 28), source(EDGE, Some(2.4), 3)],
        );
        assert_eq!(
            home_words(&own),
            [
                "Could reach: Bo's household's well, less than a minute's walk",
                "Did come: in the past 30 days its people drew at Bo's household's well on 28 \
                 days, at the water's edge on 3 days"
            ]
        );
        let others = home(
            Some(source(OTHERS_WELL, Some(0.6), 0)),
            Some(source(EDGE, Some(12.2), 0)),
            Vec::new(),
        );
        assert_eq!(
            home_words(&others),
            [
                "Could reach: Ada's household's well, less than a minute's walk (another's: its \
                 people may draw there only when short of water); by right, the water's edge, 12 \
                 minutes' walk",
                "Did come: its people's record holds no water drawn in the past 30 days"
            ]
        );
        let dry = home(None, None, Vec::new());
        assert!(home_words(&dry)[0].contains("no water within an hour's walk"));
        assert_eq!(
            summary(&[own, others, dry]),
            [
                "Could reach: of 3 homes, 2 have water within a minute's walk, 0 within 5 \
                 minutes, 0 within 15, 0 farther within an hour, 1 none within an hour; for 1 \
                 home, the nearest is another's well",
                "Did come: in the past 30 days, of 3 homes, 1 drew at their own well, 0 at a well \
                 by right, 0 at another's well, 0 at a spring, 1 at the water's edge; 2 homes \
                 have no draw recorded"
            ]
        );
    }
}
