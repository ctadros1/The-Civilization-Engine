//! People live (ADR-0003; plan §7 M1): a founding band arrives and settles, its people sleep,
//! eat, fetch water and gather, they walk only where people can walk, and splitting an advance
//! into pieces changes nothing.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::{Behavior, ChronicleKind, Origin, Repro, Sex, population};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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
fn the_band_arrives_as_couples_some_expecting_and_some_nursing() {
    // Research 05-01 §5.4: a mixture of unions and reproductive states, never a crowd of newly
    // paired adults who all conceive at once.
    let (mut pregnant, mut nursing, mut open) = (0, 0, 0);
    for seed in [3, 5, 8] {
        let sim = new_world(seed, 0);
        let pop = sim.people();
        assert_eq!(
            pop.unions.len(),
            pop.households.len(),
            "a couple heads each family"
        );
        for (_, h) in pop.households.iter() {
            let partnered: Vec<_> = h
                .members
                .iter()
                .filter_map(|m| pop.person(*m))
                .filter(|p| p.partner.is_some())
                .collect();
            assert_eq!(partnered.len(), 2, "household {}", h.id);
            assert_ne!(partnered[0].sex, partnered[1].sex);
            assert_eq!(partnered[0].partner, Some(partnered[1].id));
        }
        for (_, p) in pop.people.iter() {
            match p.repro {
                Repro::Pregnant { conceived, due, .. } => {
                    assert!(conceived <= sim.now() && due > sim.now());
                    assert_eq!(p.sex, Sex::Female);
                    pregnant += 1;
                }
                Repro::Recovering { until } => {
                    assert!(until > sim.now() && p.nursing.is_some());
                    nursing += 1;
                }
                Repro::Open => open += 1,
            }
            if p.sex == Sex::Female {
                assert!(p.fecundity > 0.0);
            }
        }
        let problems = pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
        assert!(problems.is_empty(), "{problems:?}");
    }
    assert!(pregnant > 0 && nursing > 0 && open > pregnant + nursing);
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
fn households_break_ground_keep_firewood_and_gather() {
    let mut sim = new_world(3, 0);
    let rules = sim.rules().clone();
    let goods = &rules.catalog.goods;
    let days = 8;
    sim.advance_minutes(days * 24 * 60).expect("advances");

    // Arriving in March, they mark out fields and start breaking the ground.
    let fields = &sim.land().fields;
    assert!(!fields.is_empty(), "no field marked out");
    assert!(
        fields.iter().any(|f| f.work_h > 0.0 || f.broken),
        "no ground broken"
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

/// One world run through its first season, from 1 March to the end of September, shared by the
/// tests that only read it.
fn first_season() -> &'static Sim {
    static SEASON: OnceLock<Sim> = OnceLock::new();
    SEASON.get_or_init(|| {
        let mut sim = new_world(3, 0);
        sim.advance_minutes(212 * 24 * 60).expect("advances");
        sim
    })
}

#[test]
fn a_band_sows_reaps_and_threshes_its_first_harvest() {
    let sim = first_season();
    let rules = sim.rules().clone();
    let crop = &rules.catalog.crops[rules.people.farm.crop];
    let chronicle = &sim.people().chronicle;
    let sown = chronicle
        .iter()
        .position(|e| e.kind == ChronicleKind::FirstSowing);
    let harvest = chronicle
        .iter()
        .position(|e| e.kind == ChronicleKind::HarvestIn);
    assert!(sown.is_some(), "the first sowing is noted");
    assert!(harvest > sown, "and then the harvest");
    let kg = chronicle[harvest.expect("harvest")].number;
    let ha: f64 = sim.land().fields.iter().map(|f| f.area_ha()).sum();
    // Between a poor and a good year's yield on the ground they cropped.
    assert!(
        kg > 0.3 * crop.yield_kg_per_ha * ha && kg < 1.6 * crop.yield_kg_per_ha * ha,
        "{kg:.0} kg from {ha:.2} ha"
    );
    // Every household keeps the seed to sow its fields again, and has grain.
    let now = sim.now();
    for (_, h) in sim.people().households.iter() {
        let stores = population::stores_now(h, now, &rules.people, &rules.catalog.goods);
        let own: f64 = sim
            .land()
            .fields
            .iter()
            .filter(|f| f.household == h.id)
            .map(|f| f.area_ha())
            .sum();
        assert!(
            stores[crop.seed_good] + 1e-6 >= own * crop.seed_kg_per_ha,
            "household {} keeps {:.0} kg of seed for {own:.2} ha",
            h.id,
            stores[crop.seed_good]
        );
    }
    let grain: f64 = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| {
            population::stores_now(h, now, &rules.people, &rules.catalog.goods)[crop.good]
        })
        .sum();
    assert!(grain > 0.0, "the threshed grain is in store");
}

/// One world run from 1 March to 11 November, ten days past the day households want to be under
/// a roof by, and the households it began with.
fn until_winter() -> &'static (Sim, Vec<PermanentId>) {
    static WINTER: OnceLock<(Sim, Vec<PermanentId>)> = OnceLock::new();
    WINTER.get_or_init(|| {
        let mut sim = new_world(3, 0);
        let founding = sim.people().households.iter().map(|(_, h)| h.id).collect();
        sim.advance_minutes(255 * 24 * 60).expect("advances");
        (sim, founding)
    })
}

#[test]
fn households_raise_their_huts_and_are_under_a_roof_before_winter() {
    let (sim, founding) = until_winter();
    let rules = sim.rules().clone();
    let land = sim.land();
    let hut = &rules.catalog.buildings[rules.people.home_program];
    // The households the band arrived in (couples who set up their own households later in the
    // year may still be building).
    for (_, h) in sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| founding.contains(&h.id))
    {
        // One plot and one hut each, claimed for its home.
        let plots: Vec<_> = land.plots.iter().filter(|p| p.household == h.id).collect();
        let huts: Vec<_> = land
            .buildings
            .iter()
            .filter(|b| b.household == h.id)
            .collect();
        assert_eq!((plots.len(), huts.len()), (1, 1), "household {}", h.id);
        let b = huts[0];
        assert_eq!(b.plot, plots[0].id);
        assert_eq!(plots[0].rect, civ_agents::build::plot_rect(&b.spec, hut));
        // They live in it, it sleeps all who lived there when it was designed (children born
        // since crowd in), and its roof is on by winter.
        assert_eq!(h.home, civ_agents::build::centre_m(&b.spec));
        let e = civ_grammar::expand_hut(&b.spec, &hut.rules).expect("a valid design");
        let born = h
            .members
            .iter()
            .filter(|m| sim.people().records[m].origin == civ_agents::Origin::Born)
            .count();
        assert!(e.sleeping_places as usize + born >= h.members.len());
        assert!(b.roofed(), "household {}'s hut: stage {}", h.id, b.stage);
        assert!(h.sheltered, "a roofed household keeps its stores under it");
    }
    // Plots never overlap one another, or a field.
    for (i, p) in land.plots.iter().enumerate() {
        for q in &land.plots[i + 1..] {
            assert!(
                !p.rect.near(&q.rect, 0),
                "plots {} and {} overlap",
                p.id,
                q.id
            );
        }
        for f in &land.fields {
            assert!(
                !p.rect.near(&f.rect, 0),
                "plot {} overlaps field {}",
                p.id,
                f.id
            );
        }
    }
    // The first roof of the settlement is noted once.
    let roofs = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::FirstRoof)
        .count();
    assert_eq!(roofs, 1);
    let problems = land.problems(sim.map(), rules.land.habitats.len(), sim.ids().peek_next());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn walking_wears_trails_out_from_the_village() {
    let (sim, _) = until_winter();
    let land = sim.land();
    let wear = &land.wear;
    let hearth = land.settlements.first().expect("a settlement").hearth_m;
    let trails = wear.trails();
    let metres: f32 = trails.iter().map(|t| t.length_m()).sum();
    assert!(
        metres > 500.0,
        "{} trails, {metres:.0} m, worn in by the first winter",
        trails.len()
    );
    let from_village = trails.iter().any(|t| {
        t.points
            .iter()
            .any(|&(x, y)| ((x - hearth.0).powi(2) + (y - hearth.1).powi(2)).sqrt() < 60.0)
    });
    assert!(from_village, "trails start at the village");
    assert!(
        wear.max_factor() > 0.5,
        "the busiest ground is well worn: {}",
        wear.max_factor()
    );
    assert!(wear.problems().is_empty(), "{:?}", wear.problems());
    let noted: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::FirstTrail)
        .collect();
    assert_eq!(noted.len(), 1, "the first trail is noted once");
    assert!(noted[0].number >= f64::from(population::FIRST_TRAIL_M));
    let text: String = civ_agents::history::render(noted[0], &|_| String::new())
        .into_iter()
        .map(|s| match s {
            civ_agents::history::Span::Text(t)
            | civ_agents::history::Span::Person(_, t)
            | civ_agents::history::Span::Settlement(_, t) => t,
        })
        .collect();
    assert!(text.starts_with("The first trail out of "), "{text}");
}

#[test]
fn children_are_born_to_couples_and_the_chronicle_notes_every_birth_and_death() {
    let (sim, _) = until_winter();
    let pop = sim.people();
    let year = 365.0 * 24.0 * 60.0;
    let born: Vec<_> = pop
        .records
        .values()
        .filter(|r| r.origin == Origin::Born)
        .collect();
    assert!(
        !born.is_empty(),
        "mothers expecting when the band arrived have given birth by winter"
    );
    for r in &born {
        let mother = &pop.records[&r.mother.expect("a mother")];
        assert_eq!(mother.sex, Sex::Female);
        let age = (r.born.minutes() - mother.born.minutes()) as f64 / year;
        assert!((15.0..50.0).contains(&age), "a mother of {age:.1}");
        let noted = pop
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::Born && e.people.first() == Some(&r.id));
        assert!(noted, "{}'s birth is in the chronicle", r.given);
        // A child lives with its mother while she lives.
        if let (Some(child), Some(m)) = (pop.person(r.id), pop.person(mother.id)) {
            assert_eq!(child.household, m.household);
        }
    }
    for r in pop.records.values().filter(|r| r.died.is_some()) {
        assert!(pop.person(r.id).is_none());
        let noted = pop
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::Died && e.people.first() == Some(&r.id));
        assert!(noted, "{}'s death is in the chronicle", r.given);
    }
    // Every chronicle entry names people the records know.
    for e in &pop.chronicle {
        for p in &e.people {
            assert!(pop.records.contains_key(p), "entry {} names {p}", e.seq);
        }
    }
    let problems = pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn households_out_of_food_and_worn_down_give_up_and_leave() {
    // Research 05-06 §5.2: a founding can fail, and its households withdraw. A band arriving with
    // half a day's food, quick to give up, is soon gone; nobody it leaves behind is stranded.
    let mut hungry = content().clone();
    hungry.people.params.band.provisions_days = 0.5;
    hungry.people.params.household.leave_at_depletion = 0.02;
    hungry.people.params.household.leave_per_day = 1.0;
    let mut sim = world_with(&hungry, 3);
    sim.advance_minutes(20 * 24 * 60).expect("advances");
    let pop = sim.people();
    let left: Vec<_> = pop.records.values().filter(|r| r.left.is_some()).collect();
    assert!(!left.is_empty(), "hungry households left");
    let noted: Vec<_> = pop
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::Left)
        .collect();
    assert!(!noted.is_empty());
    for r in &left {
        assert!(pop.person(r.id).is_none() && r.died.is_none());
        assert!(
            noted.iter().any(|e| e.people.contains(&r.id)),
            "{} left in a noted household",
            r.given
        );
    }
    let problems = pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
    // The land they leave stands abandoned, and the world saves and loads with it.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "gone").expect("saves");
    let loaded = persist::load(&saved.path, &hungry).expect("a world with abandoned land loads");
    assert_eq!(loaded.people().records, sim.people().records);
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
