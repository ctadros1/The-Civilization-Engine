//! People live (ADR-0003; plan §7 M1): a founding band arrives and settles, its people sleep,
//! eat, fetch water and gather, they walk only where people can walk, and splitting an advance
//! into pieces changes nothing.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::{Behavior, ChronicleKind, population};
use civ_content::ContentRegistry;
use civ_sim::persist;
use civ_sim::{NewWorld, Sim};

const PRESET: &str = "core:worldgen/river_valley";

fn content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        civ_content::load(&root)
            .registry
            .expect("the core content loads")
    })
}

fn new_world(seed: u64, band_size: u32) -> Sim {
    Sim::create(
        &NewWorld {
            name: "People".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size,
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

/// One world run for three days, shared by the tests that only read it.
fn lived() -> &'static Sim {
    static LIVED: OnceLock<Sim> = OnceLock::new();
    LIVED.get_or_init(|| {
        let mut sim = new_world(3, 0);
        sim.advance_minutes(3 * 24 * 60).expect("advances");
        sim
    })
}

/// Minutes from now until the next `hour`:00.
fn until_hour(sim: &Sim, hour: i64) -> i64 {
    (hour * 60 - sim.now().minute_of_day()).rem_euclid(24 * 60)
}

#[test]
fn a_new_world_begins_with_a_founding_band() {
    let sim = new_world(3, 0);
    let band = &content().people.params.band;
    assert_eq!(sim.founding_problem(), None);
    assert_eq!(sim.people().living(), band.default_size as usize);
    assert_eq!(sim.people().records.len(), band.default_size as usize);
    assert!(sim.people().households.len() >= band.min_families as usize);
    assert_eq!(sim.land().settlements.len(), 1);
    let kinds: Vec<ChronicleKind> = sim.people().chronicle.iter().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        vec![ChronicleKind::BandArrived, ChronicleKind::SettlementFounded]
    );
    // Everyone has decided what to do first.
    assert!(sim.pending_events() >= sim.people().living());
    for (_, p) in sim.people().people.iter() {
        assert_eq!(p.receipts.len(), 1, "{} decided once", p.given);
    }
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_band_of_any_allowed_size_can_be_asked_for() {
    let band = &content().people.params.band;
    let sim = new_world(3, band.min_size);
    assert_eq!(sim.people().living(), band.min_size as usize);
    let refused = Sim::create(
        &NewWorld {
            name: "Too many".to_owned(),
            seed: 3,
            preset_id: PRESET.to_owned(),
            size_cells: 256,
            band_size: band.max_size + 1,
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    );
    assert!(refused.is_err());
}

#[test]
fn people_sleep_eat_fetch_water_and_gather() {
    let sim = lived();
    let catalog = &sim.rules().catalog;
    let mut seen = std::collections::HashSet::new();
    for (_, p) in sim.people().people.iter() {
        for r in &p.receipts {
            if let Some(a) = catalog.activities.get(usize::from(r.chosen.def)) {
                seen.insert(a.behavior);
            }
        }
    }
    for behavior in [
        Behavior::Sleep,
        Behavior::Eat,
        Behavior::FetchWater,
        Behavior::Gather,
        Behavior::Socialize,
    ] {
        assert!(seen.contains(&behavior), "nobody chose {behavior:?}");
    }
    let start = civ_core::time::DEFAULT_WORLD_START;
    for (_, p) in sim.people().people.iter() {
        assert!(p.satiety_until > start, "{} has eaten", p.given);
        assert!(p.energy_kcal > -2000.0, "{} is not starving", p.given);
    }
    let per_person = content().people.params.household.water_l_per_person_day;
    for (_, h) in sim.people().households.iter() {
        let litres = h.water_at_time(sim.now(), h.members.len() as f64 * per_person);
        assert!(litres > 0.0, "household {} has water", h.id);
    }
}

#[test]
fn people_keep_a_daily_rhythm() {
    let mut sim = new_world(3, 0);
    let living = sim.people().living();
    // Asleep in the small hours of the second night, mostly awake at midday.
    let wait = 24 * 60 + until_hour(&sim, 2);
    sim.advance_minutes(wait).expect("advances");
    let asleep = sim.people().people.iter().filter(|(_, p)| p.asleep).count();
    assert!(
        asleep * 10 >= living * 9,
        "at 02:00 {asleep} of {living} are asleep"
    );
    let wait = until_hour(&sim, 12);
    sim.advance_minutes(wait).expect("advances");
    let asleep = sim.people().people.iter().filter(|(_, p)| p.asleep).count();
    assert!(
        asleep * 4 <= living,
        "at noon {asleep} of {living} are asleep"
    );
}

#[test]
fn people_stand_and_walk_only_where_people_can_walk() {
    let sim = lived();
    let map = sim.map();
    let t = sim.now().minutes() as f64;
    let (w, h) = (
        map.width as f32 * map.cell_size_m,
        map.height as f32 * map.cell_size_m,
    );
    for (_, p) in sim.people().people.iter() {
        let (x, y) = p.position_at(t);
        assert!(
            (0.0..w).contains(&x) && (0.0..h).contains(&y),
            "{} is on the map",
            p.given
        );
        let cell = population::cell_of(map, (x, y));
        assert!(
            sim.nav().walkable(cell),
            "{} stands on walkable ground",
            p.given
        );
        if let Some(trip) = &p.trip {
            assert_eq!(trip.points.len(), trip.minutes.len());
            assert!(trip.minutes.windows(2).all(|w| w[0] <= w[1]));
            for &q in &trip.points {
                let cell = population::cell_of(map, q);
                assert!(sim.nav().walkable(cell), "{}'s route is walkable", p.given);
            }
        }
        // Nobody's current step ended without its event being handled.
        assert!(p.act.step_ends >= sim.now(), "{} is not stuck", p.given);
    }
}

#[test]
fn splitting_an_advance_changes_nothing() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let mut sim = new_world(4, 0);
    sim.advance_minutes(6 * 60).expect("advances");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "split").expect("saves");
    let mut whole = persist::load(&saved.path, content()).expect("loads");
    let mut pieces = persist::load(&saved.path, content()).expect("loads");
    // Across midnight, so a day boundary falls inside the pieces.
    let minutes = 20 * 60 + 17;
    whole.advance_minutes(minutes).expect("advances");
    let mut left = minutes;
    let mut step = 1;
    while left > 0 {
        let n = step.min(left);
        pieces.advance_minutes(n).expect("advances");
        left -= n;
        step = step % 97 + 1;
    }
    assert_eq!(whole.now(), pieces.now());
    let a = persist::encode_sections(&whole);
    let b = persist::encode_sections(&pieces);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.tag, y.tag);
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

fn world_with(content: &ContentRegistry, seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Goods".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size: 0,
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

#[test]
fn households_keep_firewood_and_forage_to_spare_their_provisions() {
    let mut sim = new_world(3, 0);
    let rules = sim.rules().clone();
    let goods = &rules.catalog.goods;
    let provisions = rules.people.band.provisions_good;
    let held = |sim: &Sim| -> f64 {
        sim.people()
            .households
            .iter()
            .map(|(_, h)| population::stores_now(h, sim.now(), &rules.people, goods)[provisions])
            .sum()
    };
    let before = held(&sim);
    let days = 8;
    sim.advance_minutes(days * 24 * 60).expect("advances");

    // Foraging fed people too: less of the provisions went than a band eats in that time.
    let eaten_kcal = (before - held(&sim)) * goods[provisions].kcal_per_kg;
    let need_kcal =
        sim.people().living() as f64 * rules.people.household.daily_kcal_per_person * days as f64;
    assert!(
        eaten_kcal < 0.97 * need_kcal,
        "provisions eaten {eaten_kcal:.0} kcal of {need_kcal:.0} needed"
    );

    // Most households keep firewood at home.
    let with_wood = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| {
            civ_agents::person::fuel_kg(
                &population::stores_now(h, sim.now(), &rules.people, goods),
                goods,
            ) > 0.0
        })
        .count();
    assert!(
        with_wood * 4 >= sim.people().households.len() * 3,
        "{with_wood} of {} households have firewood",
        sim.people().households.len()
    );

    // People went for more than one wild resource.
    let mut resources = std::collections::BTreeSet::new();
    for (_, p) in sim.people().people.iter() {
        for r in &p.receipts {
            if let Some(res) = rules
                .catalog
                .activities
                .get(usize::from(r.chosen.def))
                .and_then(|a| a.resource)
            {
                resources.insert(res);
            }
        }
    }
    assert!(resources.len() >= 2, "gathered {resources:?}");

    // What households remember of places is shared within the settlement.
    let mut known = sim.people().households.iter().map(|(_, h)| h.known.len());
    let first = known.next().expect("households");
    assert!(first > 0, "places were worked");
    assert!(known.all(|n| n == first));
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), rules.catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_settlement_running_short_of_food_is_noted_in_the_chronicle() {
    let mut scarce = content().clone();
    scarce.people.params.band.provisions_days = 0.5;
    let mut sim = world_with(&scarce, 3);
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let short: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::FoodRanShort)
        .collect();
    assert_eq!(short.len(), 1, "noted once, not every day");
    let settlement = &sim.land().settlements[0];
    assert!(settlement.food_short);
    assert_eq!(short[0].settlement, Some(settlement.id));
    assert!(short[0].number < scarce.people.params.household.short_food_days);
}
