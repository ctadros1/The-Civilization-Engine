//! Lamps (M10b slice DU; ADR-0034 §9): work at home after dark by lamplight burns the lamp's
//! fuel by the hour and wears the lamp; work in daylight burns none.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim};

fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content")
    })
}

fn world() -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Lampwick".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

fn index(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

/// An adult of some household, by household and person.
fn adult(sim: &Sim) -> (PermanentId, PermanentId) {
    let now = sim.now();
    sim.people()
        .households
        .iter()
        .find_map(|(_, h)| {
            let p = h.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 20.0)
            })?;
            Some((h.id, p))
        })
        .expect("an adult")
}

#[test]
fn work_at_home_after_dark_burns_the_lamps_tallow_and_none_in_daylight() {
    let worked = |end_minute: i64| {
        let mut sim = world();
        let minute = sim.now().minute_of_day();
        sim.advance_minutes((end_minute - minute).rem_euclid(1440) + 1440)
            .expect("advances");
        let (lamp, tallow, clay, wood) = (
            index(&sim, "core:good/fat_lamp"),
            index(&sim, "core:good/tallow"),
            index(&sim, "core:good/clay"),
            index(&sim, "core:good/firewood"),
        );
        let pots = sim
            .rules()
            .catalog
            .activities
            .iter()
            .position(|a| a.id == "core:activity/make_pots")
            .expect("pots");
        let (hh, who) = adult(&sim);
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == hh {
                x.stores[lamp] = 1.0;
                x.stores[tallow] = 1.0;
                x.stores[clay] += 100.0;
                x.stores[wood] += 100.0;
            }
        }
        let held = |sim: &Sim, g: usize| sim.people().household(hh).expect("household").stores[g];
        let before = (held(&sim, tallow), held(&sim, lamp));
        sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, who, pots, 120));
        (
            before.0 - held(&sim, tallow),
            before.1 - held(&sim, lamp),
            sim.rules().catalog.goods[lamp]
                .tool
                .as_ref()
                .expect("a tool")
                .life_h,
        )
    };
    // Two hours of making pots ending at two in the morning: all in the dark, by lamplight.
    let (burned, worn, life_h) = worked(2 * 60);
    assert!((burned - 2.0 * 0.008).abs() < 1e-9, "burned {burned} kg");
    assert!((worn - 2.0 / life_h).abs() < 1e-9, "wore {worn}");
    // The same ending at one in the afternoon: daylight, no lamp lit.
    let (burned, worn, _) = worked(13 * 60);
    assert_eq!((burned, worn), (0.0, 0.0));
}

/// In a household-resolved settlement (ADR-0032 §2) a day's making is lived at midnight, and a
/// lamp burns only for the minutes of it that would have fallen after dark.
#[test]
fn a_household_resolved_day_burns_tallow_only_for_making_after_dark() {
    let mut sim = world();
    sim.advance_minutes(2 * 1440).expect("advances");
    sim.advance_to_midnight().expect("advances");
    let settlements: Vec<_> = sim.land().settlements.iter().map(|s| s.id).collect();
    sim.set_household_resolved(settlements);
    let (lamp, tallow) = (
        index(&sim, "core:good/fat_lamp"),
        index(&sim, "core:good/tallow"),
    );
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        x.stores[lamp] = 1.0;
        x.stores[tallow] = 10.0;
    }
    let before: Vec<(PermanentId, f64)> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| (x.id, x.stores[tallow]))
        .collect();
    sim.advance_minutes(1440).expect("advances");
    let day = sim.now();
    let catalog = &sim.rules().catalog;
    let latitude = sim.rules().people.latitude_deg;
    let sun = civ_agents::needs::daylight(latitude, civ_land::day_of_year(day.day_index()));
    let by_lamp = |def: u16| {
        catalog.activities.get(usize::from(def)).is_some_and(|a| {
            a.daylight_only
                && a.behavior == civ_agents::params::Behavior::Make
                && a.max_walk_minutes == 0
        })
    };
    let (mut lit_day, mut checked) = (0i64, 0usize);
    for (id, had) in before {
        let Some(x) = sim.people().household(id) else {
            continue;
        };
        // The minutes its members' blocks put them making by lamp after sunset.
        let mut dark = 0i64;
        for &m in &x.members {
            for minute in 0..1440i64 {
                let Some(b) = sim.people().day_block(m, day.plus_minutes(minute)) else {
                    continue;
                };
                if by_lamp(b.def) {
                    if minute >= sun.1 {
                        dark += 1;
                    } else {
                        lit_day += 1;
                    }
                }
            }
        }
        let burned = had - x.stores[tallow];
        let expected = dark as f64 / 60.0 * 0.008;
        assert!(
            (burned - expected).abs() <= 0.002 + 0.1 * expected,
            "household {id:?} burned {burned} kg for {dark} minutes after dark"
        );
        checked += 1;
    }
    assert!(checked > 0, "no household lived the day");
    assert!(
        lit_day > 0,
        "nobody made anything by daylight to tell the two apart"
    );
}
