//! People live (ADR-0003; plan §7 M1): a founding band arrives and settles, its people sleep,
//! eat, fetch water and gather, they walk only where people can walk, and splitting an advance
//! into pieces changes nothing.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::person::{Flow, Flows};
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
    Sim::create_for_tests(
        &NewWorld {
            name: "People".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
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
    let refused = Sim::create_for_tests(
        &NewWorld {
            name: "Too many".to_owned(),
            seed: 3,
            preset_id: PRESET.to_owned(),
            size_cells: 256,
            band_size: band.max_size + 1,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
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
    Sim::create_for_tests(
        &NewWorld {
            name: "Goods".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content,
        [7; 16],
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
    // The village keeps the seed to sow again the ground it cropped (research 08-02 §10: seed is
    // set aside before anything is eaten), and has grain. Seed moves with those who leave a
    // household for another, and ground broken after the harvest for next year is planned against
    // the seed a household means to crop with (it may rest some), so the claim is the village's,
    // not each household's.
    let now = sim.now();
    let seed: f64 = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| {
            population::stores_now(h, now, &rules.people, &rules.catalog.goods)[crop.seed_good]
        })
        .sum();
    let cropped: f64 = sim
        .land()
        .fields
        .iter()
        .filter(|f| f.harvests > 0)
        .map(|f| f.area_ha())
        .sum();
    assert!(cropped > 0.0, "some ground was cropped");
    assert!(
        seed + 1e-6 >= cropped * crop.seed_kg_per_ha,
        "the village keeps {seed:.0} kg of seed for the {cropped:.2} ha it cropped"
    );
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

#[test]
fn every_good_held_is_what_came_in_less_what_went_out() {
    // ADR-0006 §3: from the founding on, what households hold is what was brought, gathered,
    // harvested and made, less what was eaten, spoiled, burned, worn out, built, sown and used.
    let sim = first_season();
    let pop = sim.people();
    let goods = &sim.rules().catalog.goods;
    let flows = pop.flows();
    let gaps = population::unaccounted(
        goods.len(),
        (&[], &Flows::default()),
        (&pop.goods_held(), &flows),
    );
    let named: Vec<String> = gaps
        .iter()
        .map(|&(g, kg)| format!("{} {kg:+.6}", goods[g].name))
        .collect();
    assert!(named.is_empty(), "unaccounted for: {named:?}");
    // The season moved goods every way that matters.
    let good = |id: &str| goods.iter().position(|g| g.id == id).expect(id);
    for (flow, id) in [
        (Flow::Brought, "core:good/provisions"),
        (Flow::Eaten, "core:good/provisions"),
        (Flow::Got, "core:good/grain"),
        (Flow::Sown, "core:good/seed_grain"),
        (Flow::Burned, "core:good/firewood"),
        (Flow::Worn, "core:good/sickle"),
    ] {
        assert!(flows.get(flow, good(id)) > 0.0, "{flow:?} {id}");
    }
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
    let hut = &rules.catalog.buildings[rules.people.build.programs[0]];
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
        let e = civ_grammar::expand(&b.spec, &hut.rules).expect("a valid design");
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
fn a_household_with_goods_to_spare_builds_a_larger_hut_than_it_needs() {
    let mut sim = new_world(3, 0);
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let rules = sim.rules().clone();
    let hut = &rules.catalog.buildings[rules.people.build.programs[0]];
    let grain = rules
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/grain")
        .expect("grain");
    // Two founding households of a size lose the huts they began, and are to plan them again:
    // one is given grain it does not need (in hours of its own work, some thousands), the other
    // nothing.
    let mut by_size: Vec<(usize, PermanentId)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.members.len(), h.id))
        .collect();
    by_size.sort_unstable();
    let (members, rich, plain) = by_size
        .windows(2)
        .find(|w| w[0].0 == w[1].0)
        .map(|w| (w[0].0, w[1].1, w[0].1))
        .expect("two households of a size");
    {
        let land = sim.land_mut_for_tests();
        let lost: Vec<PermanentId> = land
            .buildings
            .iter()
            .filter(|b| b.household == rich || b.household == plain)
            .map(|b| b.plot)
            .collect();
        assert_eq!(lost.len(), 2, "both have begun their huts");
        land.buildings
            .retain(|b| b.household != rich && b.household != plain);
        land.plots.retain(|p| !lost.contains(&p.id));
    }
    {
        let pop = sim.people_mut_for_tests();
        let h = pop
            .households
            .iter_mut()
            .map(|(_, h)| h)
            .find(|h| h.id == rich)
            .expect("household");
        h.stores[grain] += 4000.0;
    }
    let radius = |sim: &Sim, id: PermanentId| {
        sim.land()
            .buildings
            .iter()
            .find(|b| b.household == id)
            .and_then(|b| match b.spec.footprint {
                civ_grammar::Footprint::Round { radius, .. } => Some(radius),
                civ_grammar::Footprint::Rect { .. } => None,
            })
    };
    // Each begins again when it next chooses to build.
    for _ in 0..20 {
        if radius(&sim, rich).is_some() && radius(&sim, plain).is_some() {
            break;
        }
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let radius = |id: PermanentId| radius(&sim, id);
    let need = hut.hut().expect("a hut program").radius_for(members);
    let (rich_r, plain_r) = (
        radius(rich).expect("the household with grain has begun again"),
        radius(plain).expect("the other household has begun again"),
    );
    assert!(
        rich_r > need && rich_r > plain_r,
        "{members} people need {need} cm: {rich_r} cm with grain to spare, {plain_r} cm without"
    );
    assert!(plain_r >= need && rich_r <= hut.hut().expect("a hut program").radius_cm.1);
}

#[test]
fn a_household_grown_rich_builds_a_larger_home_beside_its_old_one_and_moves_in() {
    let mut sim = new_world(3, 0);
    let rules = sim.rules().clone();
    let hut = &rules.catalog.buildings[rules.people.build.programs[0]];
    let good = |id: &str| {
        rules
            .catalog
            .goods
            .iter()
            .position(|g| g.id == id)
            .expect(id)
    };
    let radius_of = |b: &civ_land::Building| match b.spec.footprint {
        civ_grammar::Footprint::Round { radius, .. } => radius,
        civ_grammar::Footprint::Rect { .. } => panic!("households build huts"),
    };
    // Live until a household's first hut is finished.
    let finished = |sim: &Sim| {
        sim.land()
            .buildings
            .iter()
            .find(|b| b.finished())
            .map(|b| (b.household, b.id, b.plot, radius_of(b)))
    };
    let mut found = None;
    for _ in 0..40 {
        found = finished(&sim);
        if found.is_some() {
            break;
        }
        sim.advance_minutes(10 * 24 * 60).expect("advances");
    }
    let (rich, old, old_plot, old_r) = found.expect("a hut finished within the year");
    let old_home = sim
        .people()
        .households
        .iter()
        .find(|(_, h)| h.id == rich)
        .map(|(_, h)| h.home)
        .expect("household");
    // It grows rich: grain it does not need, worth thousands of hours of its work, and the
    // timber and thatch a larger hut is built of.
    {
        let pop = sim.people_mut_for_tests();
        let h = pop
            .households
            .iter_mut()
            .map(|(_, h)| h)
            .find(|h| h.id == rich)
            .expect("household");
        h.stores[good("core:good/grain")] += 10_000.0;
        h.stores[good("core:good/timber")] += 3_000.0;
        h.stores[good("core:good/thatch")] += 3_000.0;
    }
    let new_building = |sim: &Sim| {
        sim.land()
            .buildings
            .iter()
            .find(|b| b.household == rich && b.id != old)
            .map(|b| (b.id, radius_of(b), b.roofed()))
    };
    // A larger hut it could not roof before winter waits until the roof deadline moves on to
    // next year (`build::roof_deadline`), so it may begin a few months later.
    let mut begun = None;
    for _ in 0..240 {
        begun = new_building(&sim);
        if begun.is_some() {
            break;
        }
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let (new, new_r, _) = begun.expect("it begins a new home");
    // At least a quarter more floor, within the rules; built beside the old one, which it lives
    // in meanwhile.
    assert!(
        f64::from(new_r).powi(2) >= 1.25 * f64::from(old_r).powi(2) - 1.0
            && new_r <= hut.hut().expect("a hut program").radius_cm.1,
        "{old_r} cm to {new_r} cm"
    );
    let lives_at = |sim: &Sim| {
        sim.people()
            .households
            .iter()
            .find(|(_, h)| h.id == rich)
            .map(|(_, h)| h.home)
    };
    assert_eq!(lives_at(&sim), Some(old_home), "it lives in the old home");
    assert!(sim.land().buildings.iter().any(|b| b.id == old));
    // Once the new roof is on it moves in, and the old home is taken down.
    for _ in 0..300 {
        if new_building(&sim).is_some_and(|(_, _, roofed)| roofed) {
            break;
        }
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == new)
        .expect("the new home stands");
    assert!(b.roofed(), "the new roof is on within the year");
    assert_eq!(lives_at(&sim), Some(civ_agents::build::centre_m(&b.spec)));
    assert!(!sim.land().buildings.iter().any(|b| b.id == old));
    assert!(!sim.land().plots.iter().any(|p| p.id == old_plot));
    assert_eq!(
        sim.land()
            .buildings
            .iter()
            .filter(|b| b.household == rich)
            .count(),
        1
    );
    let problems = sim
        .land()
        .problems(sim.map(), rules.land.habitats.len(), sim.ids().peek_next());
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
    let text =
        civ_agents::history::plain(&civ_agents::history::render(noted[0], &|_| String::new()));
    assert!(text.starts_with("The first trail out of "), "{text}");
}

/// A dry, walkable point about `metres` from `from`, if there is one.
fn land_near(sim: &Sim, from: (f32, f32), metres: f32) -> Option<(f32, f32)> {
    (0..64).find_map(|k| {
        let a = k as f32 / 64.0 * std::f32::consts::TAU;
        let p = (from.0 + metres * a.cos(), from.1 + metres * a.sin());
        let (w, h) = sim.map().extent_m();
        if p.0 < 0.0 || p.1 < 0.0 || f64::from(p.0) >= w || f64::from(p.1) >= h {
            return None;
        }
        let cell = population::cell_of(sim.map(), p);
        (sim.map().water[cell] == civ_world::WATER_LAND && sim.nav().walkable(cell)).then_some(p)
    })
}

#[test]
fn families_sent_together_settle_side_by_side_in_one_place() {
    let mut sim = new_world(3, 0);
    sim.advance_minutes(24 * 60).expect("advances");
    let village = sim.land().settlements[0].clone();
    let (households, living) = (sim.people().households.len(), sim.people().living());
    let far = land_near(&sim, village.hearth_m, 1500.0).expect("dry ground far away");
    let group = sim.spawn_families(far, 10).expect("the families arrive");
    assert_eq!(group.len(), 10, "dry ground around the first for all ten");
    assert!(group[0].founded, "the first makes camp");
    assert!(group[1..].iter().all(|s| !s.founded));
    assert!(group.iter().all(|s| s.settlement == group[0].settlement));
    assert_eq!(sim.land().settlements.len(), 2, "one camp, not ten");
    let people: usize = group.iter().map(|s| s.people.len()).sum();
    assert_eq!(sim.people().households.len(), households + 10);
    assert_eq!(sim.people().living(), living + people);
    let homes: Vec<(f32, f32)> = group
        .iter()
        .map(|s| {
            let hh = sim
                .people()
                .households
                .iter()
                .find(|(_, h)| h.id == s.household)
                .map(|(_, h)| h)
                .expect("the household");
            assert_eq!(hh.settlement, Some(group[0].settlement));
            hh.home
        })
        .collect();
    for (i, a) in homes.iter().enumerate() {
        for b in &homes[i + 1..] {
            let d = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
            assert!(
                d > 5.0,
                "homes side by side, not on top of each other: {d} m"
            );
        }
        let d = ((a.0 - far.0).powi(2) + (a.1 - far.1).powi(2)).sqrt();
        assert!(d < 150.0, "close to where they were sent: {d} m");
    }
    let arrived = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::FamilyArrived)
        .count();
    assert_eq!(arrived, 10, "each family noted as it arrives");
    // Each family is drawn on its own: not ten copies of the first.
    let shapes: std::collections::BTreeSet<Vec<(i64, bool)>> = group
        .iter()
        .map(|s| {
            s.people
                .iter()
                .map(|id| {
                    let p = &sim.people().records[id];
                    (p.born.minutes(), p.sex == Sex::Female)
                })
                .collect()
        })
        .collect();
    assert!(shapes.len() > 1, "families of their own");

    sim.advance_minutes(3 * 24 * 60).expect("the world goes on");
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn families_sent_to_the_hearth_come_in_groups_and_join_the_village() {
    let mut sim = new_world(3, 0);
    let (households, living) = (sim.people().households.len(), sim.people().living());
    // Twenty-five families: a group of twenty and a group of five, about their own points.
    let came = sim.send_families_to_hearth(25);
    assert_eq!(sim.land().settlements.len(), 1, "all join the village");
    assert_eq!(sim.people().living(), living + came);
    let arrived = sim.people().households.len() - households;
    assert!(
        (20..=25).contains(&arrived),
        "{arrived} of 25 families arrived"
    );
    assert_eq!(sim.send_families_to_hearth(0), 0);
}

#[test]
fn the_observer_sends_families_that_join_a_village_or_make_camp() {
    let mut sim = new_world(3, 0);
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let village = sim.land().settlements[0].clone();
    let households = sim.people().households.len();
    let living = sim.people().living();

    let near = land_near(&sim, village.hearth_m, 120.0).expect("dry ground by the village");
    let joined = sim.spawn_family(near).expect("a family arrives");
    assert!(!joined.founded, "close to the village it joins");
    assert_eq!(joined.settlement, village.id);
    assert!(joined.people.len() >= 2);
    assert_eq!(sim.people().households.len(), households + 1);
    assert_eq!(sim.people().living(), living + joined.people.len());
    for id in &joined.people {
        assert_eq!(sim.people().records[id].origin, Origin::Spawned);
    }
    let noted = sim
        .people()
        .chronicle
        .iter()
        .rev()
        .find(|e| e.kind == ChronicleKind::FamilyArrived)
        .expect("noted in the chronicle");
    assert_eq!(noted.people, joined.people);
    assert_eq!(noted.settlement, Some(village.id));

    let far = land_near(&sim, village.hearth_m, 1500.0).expect("dry ground far away");
    let camped = sim.spawn_family(far).expect("a family arrives");
    assert!(camped.founded, "far from any village it makes camp");
    assert_eq!(sim.land().settlements.len(), 2);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::SettlementFounded
                && e.settlement == Some(camped.settlement))
    );

    let water = (0..sim.map().cell_count())
        .find(|&c| sim.map().water[c] != civ_world::WATER_LAND)
        .map(|c| population::cell_centre(sim.map(), c));
    if let Some(wet) = water {
        assert!(sim.spawn_family(wet).is_err(), "not into water");
    }
    assert!(sim.spawn_family((-5.0, 10.0)).is_err(), "not off the map");

    sim.advance_minutes(3 * 24 * 60).expect("the world goes on");
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
    let alone = sim
        .people()
        .households
        .iter()
        .find(|(_, h)| h.id == camped.household)
        .map(|(_, h)| h.settlement);
    assert_eq!(
        alone,
        Some(Some(camped.settlement)),
        "the camp's household is its own"
    );
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

#[test]
fn people_who_keep_company_at_the_hearth_come_to_know_one_another() {
    let sim = first_season();
    let people = sim.people();
    let rules = sim.rules();
    let tp = &rules.people.ties;
    let day = sim.now().day_index();
    let holders = people.ties.holders();
    assert!(
        holders.len() * 2 > people.living(),
        "most people hold ties: {} of {}",
        holders.len(),
        people.living()
    );
    let mut from_hearth = 0;
    for h in &holders {
        let ties = people.ties.of(*h);
        assert!(ties.len() <= tp.room, "no more than the room for ties");
        assert!(ties.windows(2).all(|w| w[0].to < w[1].to), "kept in order");
        for t in ties {
            assert_ne!(t.to, *h, "nobody holds a tie to themselves");
            let now = t.at(day, tp);
            assert!((0.0..=1.0).contains(&now.familiarity));
            assert!((0.0..=1.0).contains(&now.warmth));
            if t.reason.map(|r| r.act) == Some(civ_agents::ties::Act::Hearth) {
                from_hearth += 1;
                // Company alone is evidence of nothing.
                for d in civ_agents::ties::Domain::ALL {
                    assert!(now.esteem(d, tp).abs() < 1e-6);
                }
            }
        }
    }
    assert!(from_hearth > 0, "company at the hearth made ties");
}

#[test]
fn company_at_the_hearth_is_kept_only_with_those_of_ones_own_settlement() {
    let mut sim = new_world(3, 0);
    sim.advance_minutes(24 * 60).expect("advances");
    let village = sim.land().settlements[0].clone();
    let far = land_near(&sim, village.hearth_m, 1500.0).expect("dry ground far away");
    let camp = sim.spawn_families(far, 10).expect("the families arrive");
    assert_eq!(
        sim.land().settlements.len(),
        2,
        "a camp apart from the village"
    );
    sim.advance_minutes(20 * 24 * 60)
        .expect("the world goes on");

    let people = sim.people();
    let settlement_of = |id: PermanentId| {
        let household = people.person(id)?.household;
        people.household(household)?.settlement
    };
    let (mut at_home, mut across) = (0, Vec::new());
    for h in people.ties.holders() {
        let Some(mine) = settlement_of(h) else {
            continue;
        };
        for t in people.ties.of(h) {
            if t.reason.map(|r| r.act) != Some(civ_agents::ties::Act::Hearth) {
                continue;
            }
            match settlement_of(t.to) {
                Some(theirs) if theirs == mine => at_home += 1,
                Some(theirs) => across.push((h, t.to, mine, theirs)),
                None => {}
            }
        }
    }
    assert!(at_home > 0, "company at each hearth made ties");
    assert!(
        across.is_empty(),
        "nobody keeps company at another settlement's hearth: {across:?} (camp {})",
        camp[0].settlement
    );
}

/// The household of the founding band with the most members, its oldest grown member left alone
/// in it (the others die), `age` years old now, holding `grain` kg of grain and a quern and
/// nothing else, and knowing how to grind, pound and bake.
fn lone_elder(sim: &mut Sim, age: f64, grain: f64) -> (PermanentId, PermanentId) {
    let now = sim.now();
    let (hh, members) = sim
        .people()
        .households
        .iter()
        .max_by_key(|(_, h)| (h.members.len(), std::cmp::Reverse(h.id)))
        .map(|(_, h)| (h.id, h.members.clone()))
        .expect("a household");
    let elder = members
        .iter()
        .copied()
        .max_by(|&a, &b| {
            let age = |m| sim.people().person(m).map_or(0.0, |p| p.age_years(now));
            age(a).total_cmp(&age(b)).then(b.cmp(&a))
        })
        .expect("a member");
    for &m in members.iter().filter(|&&m| m != elder) {
        sim.die_for_tests(m);
    }
    for t in [
        "core:technique/quern_grinding",
        "core:technique/pounding",
        "core:technique/hearth_baking",
    ] {
        let i = sim.rules().catalog.technique_index(t).expect(t);
        if !sim.people().person(elder).is_some_and(|p| p.knows(i)) {
            sim.introduce_technique(elder, t, false)
                .expect("introduced");
        }
    }
    let goods = sim.rules().catalog.goods.clone();
    let index = |id: &str| goods.iter().position(|g| g.id == id).expect(id);
    let (g, q) = (index("core:good/grain"), index("core:good/quern"));
    let pop = sim.people_mut_for_tests();
    let p = pop
        .people
        .iter_mut()
        .find(|(_, p)| p.id == elder)
        .map(|(_, p)| p)
        .expect("the elder");
    p.born = civ_core::SimTime::from_minutes(now.minutes() - (age * 365.0 * 1440.0) as i64);
    let x = pop
        .households
        .iter_mut()
        .find(|(_, x)| x.id == hh)
        .map(|(_, x)| x)
        .expect("the household");
    x.stores.iter_mut().for_each(|kg| *kg = 0.0);
    x.stores[g] = grain;
    x.stores[q] = 1.0;
    x.stores_at = now;
    (hh, elder)
}

#[test]
fn a_lone_elder_with_grain_and_a_quern_but_no_firewood_asks_for_fuel_and_eats() {
    // An elder of 72, alone, holds 198 kg of grain and a quern and knows how to grind, pound and
    // bake, but has no firewood and nothing to eat as it is; the wild food around has been
    // gathered bare, and they are 40 % into their body's reserve. Grain is eaten only ground and
    // baked or pounded and boiled, both over a fire, and at 72 they are past gathering firewood
    // (the content's 70). Their food waits on a fire, so they are in need however much grain
    // they hold: they ask a neighbour who can spare fuel for some (research 08-11 §1.1:
    // need-based help), grind, bake and eat. Before, the grain counted as food in hand: they were
    // never short enough to ask, nothing fetched fuel for hunger, and in this scene they drew
    // their whole reserve by day 26 and died of hunger on day 35 beside 193 kg of grain (the lone
    // elders of the dying worlds, plan §9).
    let mut sim = new_world(3, 0);
    sim.advance_minutes(5 * 24 * 60).expect("advances");
    let (hh, elder) = lone_elder(&mut sim, 72.3, 198.0);
    let params = sim.rules().people.clone();
    let goods = sim.rules().catalog.goods.clone();
    let index = |id: &str| goods.iter().position(|g| g.id == id).expect(id);
    let (grain, bread, wood) = (
        index("core:good/grain"),
        index("core:good/bread"),
        index("core:good/firewood"),
    );
    let wild: Vec<usize> = sim
        .rules()
        .land
        .resources
        .iter()
        .enumerate()
        .filter(|(_, r)| ["wild_plants", "game", "fish"].contains(&r.id.as_str()))
        .map(|(i, _)| i)
        .collect();
    let start = sim.now();
    let depleted_at = |sim: &Sim| {
        let p = sim.people().person(elder)?;
        let now = sim.now();
        Some(-population::energy_now(p, now, &params) / population::reserve_kcal(p, now, &params))
    };
    {
        let pop = sim.people_mut_for_tests();
        for (_, p) in pop.people.iter_mut().filter(|(_, p)| p.id == elder) {
            let reserve = population::reserve_kcal(p, start, &params);
            p.energy_kcal = (-0.4 * reserve) as f32;
            p.burn_kcal_min = 0.0;
            p.needs_at = start;
        }
    }
    let before = depleted_at(&sim).expect("alive");
    for _ in 0..30 {
        for &r in &wild {
            sim.land_mut_for_tests().stocks[r]
                .iter_mut()
                .for_each(|s| *s = 0.0);
        }
        sim.advance_minutes(24 * 60).expect("lives");
    }
    let record = &sim.people().records[&elder];
    assert_eq!(record.died, None, "they died beside their grain");
    let x = sim.people().household(hh).expect("their household");
    assert_eq!(x.members, vec![elder], "alone throughout");
    assert!(
        x.flows.get(Flow::Received, wood) > 0.0,
        "no firewood came to them"
    );
    assert!(x.flows.get(Flow::Made, bread) > 0.0, "they baked no bread");
    assert!(x.flows.get(Flow::Eaten, bread) > 0.0, "they ate no bread");
    let held = population::stores_now(x, sim.now(), &params, &goods);
    assert!(
        held[grain] < 198.0,
        "none of the grain was ground: {held:?}"
    );
    let after = depleted_at(&sim).expect("alive");
    assert!(
        after < before,
        "they ate less than they burned: {before:.2} of her reserve drawn, then {after:.2}"
    );
}

/// Gleaning (research 08-22): reaping leaves a share of the crop on the field, laid down as the
/// gleanings of the field's patch; nothing grows it, and people gather it as they gather any
/// food, by what it brings for the stooping. The band's first harvest leaves grain on the
/// stubble, and someone gleans it in the weeks after.
#[test]
fn reaping_leaves_grain_on_the_stubble_and_people_glean_it() {
    let mut sim = new_world(3, 0);
    let rules = sim.rules().clone();
    let crop = &rules.catalog.crops[rules.people.farm.crop];
    assert!(crop.left_share > 0.0, "reaping leaves some of the crop");
    let r = rules
        .land
        .resources
        .iter()
        .position(|x| x.id == "gleanings")
        .expect("the land has gleanings");
    assert!(rules.land.resources[r].reaped);
    assert_eq!(rules.land.resources[r].good, crop.good);
    let glean = rules
        .catalog
        .index_of("core:activity/glean")
        .expect("gleaning is an activity");
    let total = |sim: &Sim| {
        sim.land().stocks[r]
            .iter()
            .map(|&s| f64::from(s))
            .sum::<f64>()
    };
    assert_eq!(
        total(&sim),
        0.0,
        "nothing lies on the ground before a harvest"
    );
    // Live day by day until the first field is reaped.
    let mut reaped = None;
    for _ in 0..300 {
        sim.advance_minutes(24 * 60).expect("advances");
        if let Some(f) = sim.land().fields.iter().find(|f| f.harvests > 0) {
            reaped = Some(f.id);
            break;
        }
    }
    let field = reaped.expect("a field is reaped in the first season");
    let at = {
        let f = sim
            .land()
            .fields
            .iter()
            .find(|f| f.id == field)
            .expect("kept");
        let c = f.rect.centre_m();
        sim.land().patches.of_point(c.0, c.1)
    };
    assert!(
        sim.land().stocks[r][at] > 0.0,
        "the reaped field's patch holds gleanings"
    );
    // A household whose harvest is lost and whose food is gone (as the poor of a place, research
    // 08-22 §1) gleans in the days after; one with its harvest in has little reason to.
    let holder = sim
        .land()
        .fields
        .iter()
        .find(|f| f.id == field)
        .and_then(|f| f.holder.household())
        .expect("a household's field");
    let wild: Vec<usize> = rules
        .land
        .resources
        .iter()
        .enumerate()
        .filter(|(i, x)| *i != r && rules.catalog.goods[x.good].kcal_per_kg > 0.0)
        .map(|(i, _)| i)
        .collect();
    let food: Vec<usize> = rules
        .catalog
        .goods
        .iter()
        .enumerate()
        .filter(|(_, g)| g.kcal_per_kg > 0.0)
        .map(|(i, _)| i)
        .collect();
    // Its harvest failed outright: nothing in store, nothing in sheaves, each day of the window
    // (gifts from neighbours and its other fields do not refill it).
    let empty = |sim: &mut Sim| {
        let pop = sim.people_mut_for_tests();
        let x = pop
            .households
            .iter_mut()
            .find(|(_, x)| x.id == holder)
            .expect("the household")
            .1;
        for &g in &food {
            if let Some(kg) = x.stores.get_mut(g) {
                *kg = 0.0;
            }
        }
        let members = x.members.clone();
        let land = sim.land_mut_for_tests();
        for f in land.fields.iter_mut() {
            if f.holder.household() == Some(holder) {
                f.sheaves_kg = 0.0;
            }
        }
        // And nothing wild is to be had: the stubble is what is left to gather.
        for &w in &wild {
            land.stocks[w].iter_mut().for_each(|s| *s = 0.0);
        }
        members
    };
    let poor = empty(&mut sim);
    let mut gleaned = false;
    for _ in 0..20 {
        sim.advance_minutes(24 * 60).expect("advances");
        empty(&mut sim);
        gleaned = sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| poor.contains(&p.id))
            .any(|(_, p)| {
                p.receipts
                    .iter()
                    .any(|x| usize::from(x.chosen.def) == glean)
            });
        if gleaned {
            break;
        }
    }
    assert!(gleaned, "the household short of food gleaned the stubble");
    // The record keeps what was gleaned, and that the household short of food gleaned it
    // (research 08-22; the dashboard's crime and poverty row reads it), across a save and load.
    let g = sim.people().order.gleaned;
    assert!(g.kg > 0.0 && g.short_kg > 0.0, "{g:?}");
    assert!(g.short_kg <= g.kg + 1e-9, "{g:?}");
    // The dashboard's harvest margins count the same grain by whether its gleaner could earn the
    // harvest wage (§8 18, first): every kilogram once, in counters no decision reads.
    let m = sim.people().margins;
    assert!(
        (m.gleaned_able_kg + m.gleaned_other_kg - g.kg).abs() < 1e-6,
        "{m:?} {g:?}"
    );
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "glean").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().order.gleaned, g);
    // Gleaning is a use of a place like any gathering (research 08-22 §1, §4: the custom was the
    // place's own; outsiders were barred): the household's people worked the stubble's patch,
    // and the record of who uses which place, which outsiders' views and a polity's claims on
    // places read (M5c slices AT, AU: outsiders use a claimed place by leave), holds it.
    let side = 2 * rules.land.resources[r].range_patches + 1;
    let cols = sim.land().patches.cols;
    let rows = sim.land().patches.rows;
    let (px, py) = (at as u32 % cols, at as u32 / cols);
    let centre = ((py / side) * side + side / 2).min(rows - 1) * cols
        + ((px / side) * side + side / 2).min(cols - 1);
    let place = civ_agents::uses::Place::Patch(centre);
    assert!(place.claimable(), "a polity may claim the stubble's patch");
    let used = sim.people().uses.held(
        holder,
        place,
        sim.now().day_index(),
        rules.people.places.use_half_life_days,
    );
    assert!(
        used.as_ref().is_some_and(|u| u.kcal > 0.0),
        "the household's gleaning is a use of the patch: {used:?}"
    );
}

/// A crop left standing by a household that is no more, with no heir to take it with the field
/// (research 08-22 §1; 05-06 §1.4, §5.3: only surviving property rights bring a household back to
/// reap): once ripe, what it would give is left on the ground as gleanings for anyone in reach,
/// the settlement's households know it stands there, and the field lies between crops for the
/// next holder.
#[test]
fn a_crop_nobody_is_left_to_reap_is_left_to_be_gleaned() {
    let mut sim = new_world(3, 0);
    let rules = sim.rules().clone();
    let crop = rules.catalog.crops[rules.people.farm.crop].clone();
    let r = rules
        .land
        .resources
        .iter()
        .position(|x| x.id == "gleanings")
        .expect("the land has gleanings");
    // Live until a field is sown and not yet ripe.
    let mut sown = None;
    for _ in 0..300 {
        sim.advance_minutes(24 * 60).expect("advances");
        let day = sim.now().day_index();
        if let Some(f) = sim
            .land()
            .fields
            .iter()
            .find(|f| f.stage == civ_land::FieldStage::Sown && day + 5 < f.ripe_day(&crop))
        {
            sown = Some((f.id, f.household));
            break;
        }
    }
    let (field, household) = sown.expect("a field is sown in the first season");
    // Its household's people die, with no kin to take what it held.
    let members = sim
        .people()
        .household(household)
        .expect("the household")
        .members
        .clone();
    {
        let pop = sim.people_mut_for_tests();
        for rec in pop.records.values_mut() {
            if members.contains(&rec.id) {
                (rec.mother, rec.father) = (None, None);
            }
            if rec.mother.is_some_and(|m| members.contains(&m)) {
                rec.mother = None;
            }
            if rec.father.is_some_and(|f| members.contains(&f)) {
                rec.father = None;
            }
        }
    }
    // The youngest first, so no child is left to go to a neighbour's with the household's claim.
    let now = sim.now();
    let mut members = members;
    members.sort_by(|a, b| {
        let age = |p: &PermanentId| sim.people().person(*p).map_or(0.0, |q| q.age_years(now));
        age(a).total_cmp(&age(b))
    });
    for m in members {
        sim.with_ctx_for_tests(|pop, ctx| pop.die_for_tests(ctx, m));
    }
    assert!(sim.people().household(household).is_none(), "no more");
    let get = |sim: &Sim| {
        sim.land()
            .fields
            .iter()
            .find(|f| f.id == field)
            .cloned()
            .expect("the field")
    };
    let f = get(&sim);
    assert_eq!(f.stage, civ_land::FieldStage::Sown, "its crop still stands");
    let c = f.rect.centre_m();
    let at = sim.land().patches.of_point(c.0, c.1);
    let before = sim.land().stocks[r][at];
    // Past ripeness, the crop lies on the ground to be gleaned, and the field is between crops.
    let ripe = f.ripe_day(&crop);
    while sim.now().day_index() <= ripe + 1 {
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let f = get(&sim);
    assert_eq!(f.stage, civ_land::FieldStage::Fallow, "{:?}", f.stage);
    let after = sim.land().stocks[r][at];
    assert!(
        f64::from(after - before) > 0.2 * crop.yield_kg_per_ha * f.area_ha(),
        "the standing crop lies there to glean: {before} to {after}"
    );
    assert!(
        sim.people()
            .households
            .iter()
            .any(|(_, x)| x.known.iter().any(|k| usize::from(k.resource) == r)),
        "the settlement knows it stands there"
    );
}

/// The hired reapers' scene (§8 17, fourth): a household whose ripe crop has stood all but a day
/// and a half of its grace, holding `pay_kg` of its grain to pay with, among neighbours whose
/// harvest is in and whose food is gone. Gives the world, the household, its neighbours, its
/// fields and the activity of working for hire.
fn short_of_hands_at_the_harvest(
    pay_kg: f64,
) -> (Sim, PermanentId, Vec<PermanentId>, Vec<PermanentId>, usize) {
    let mut sim = new_world(3, 0);
    let rules = sim.rules().clone();
    let crop = rules.catalog.crops[rules.people.farm.crop].clone();
    let hire = rules
        .catalog
        .index_of("core:activity/hired_work")
        .expect("working for hire is an activity");
    let ripe = |sim: &Sim| {
        let day = sim.now().day_index();
        sim.land()
            .fields
            .iter()
            .find(|f| f.task(&crop, day) == Some(civ_land::FieldTask::Reap))
            .map(|f| f.household)
    };
    // Live day by day until a crop is ripe.
    let mut employer = None;
    for _ in 0..300 {
        sim.advance_minutes(24 * 60).expect("advances");
        employer = ripe(&sim);
        if employer.is_some() {
            break;
        }
    }
    let employer = employer.expect("a crop ripens in the first season");
    let settlement = sim
        .people()
        .household(employer)
        .and_then(|x| x.settlement)
        .expect("the household has a settlement");
    let day = sim.now().day_index();
    // Its crop has stood ripe all but a day and a half of the grace before a quarter is lost.
    let grace = 0.25 / crop.standing_loss_per_day;
    let sown = day - i64::from(crop.grow_days) - (grace - 1.5).floor() as i64;
    let food: Vec<usize> = rules
        .catalog
        .goods
        .iter()
        .enumerate()
        .filter(|(_, g)| g.kcal_per_kg > 0.0)
        .map(|(i, _)| i)
        .collect();
    let mut neighbours = Vec::new();
    {
        let pop = sim.people_mut_for_tests();
        for (_, x) in pop.households.iter_mut() {
            if x.id == employer {
                // The grain to pay with.
                if let Some(kg) = x.stores.get_mut(crop.good) {
                    *kg = pay_kg;
                }
            } else if x.settlement == Some(settlement) {
                // The neighbours' harvest is in and their food is gone.
                for &g in &food {
                    if let Some(kg) = x.stores.get_mut(g) {
                        *kg = 0.0;
                    }
                }
                neighbours.push(x.id);
            }
        }
        let land = sim.land_mut_for_tests();
        for f in land.fields.iter_mut() {
            if f.household == employer {
                if f.stage == civ_land::FieldStage::Sown {
                    f.sown_day = sown;
                    f.work_h = 0.0;
                }
            } else if neighbours.contains(&f.household) {
                f.stage = civ_land::FieldStage::Fallow;
                f.work_h = 0.0;
                f.sheaves_kg = 0.0;
            }
        }
    }
    assert!(!neighbours.is_empty(), "the household has neighbours");
    sim.people_mut_for_tests().forget_views();
    let fields: Vec<PermanentId> = sim
        .land()
        .fields
        .iter()
        .filter(|f| f.household == employer)
        .map(|f| f.id)
        .collect();
    (sim, employer, neighbours, fields, hire)
}

/// A household with no grain to pay reapers with hires none, however short of hands (the ninth
/// coverage pass, §8 18, third; slice 1: no more than the grain it holds pays for).
#[test]
fn a_household_with_no_grain_to_pay_hires_no_reapers() {
    let (mut sim, _, neighbours, fields, hire) = short_of_hands_at_the_harvest(0.0);
    for _ in 0..3 * 24 {
        sim.advance_minutes(60).expect("advances");
        let hired = sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| neighbours.contains(&p.household))
            .any(|(_, p)| {
                p.receipts.iter().any(|r| {
                    usize::from(r.chosen.def) == hire
                        && matches!(r.chosen.target, civ_agents::person::Target::Field(f)
                            if fields.contains(&f))
                })
            });
        assert!(
            !hired,
            "nobody sets out to reap for a household that cannot pay"
        );
    }
    assert_eq!(sim.people().margins.reapings, 0);
}

/// Hired field work at the harvest (§8 17, fourth; research 08-10 §1.3: a household allocates
/// its hands across its own land and wage work by season): a household whose ripe crop has stood
/// so long that its own hands cannot reap it before it loses a quarter of itself hires reapers, as
/// a workshop hires hands, and pays them in grain; a neighbour with its own harvest in and its food
/// gone, for whom the wage is worth more than what the time would do at home, reaps for it.
#[test]
fn a_household_short_of_hands_at_the_harvest_hires_reapers() {
    let (mut sim, employer, neighbours, fields, hire) = short_of_hands_at_the_harvest(400.0);
    let rules = sim.rules().clone();
    let crop = rules.catalog.crops[rules.people.farm.crop].clone();
    let worked = |sim: &Sim| -> f64 {
        sim.land()
            .fields
            .iter()
            .filter(|f| f.household == employer)
            .map(|f| f64::from(f.work_h) + f64::from(f.harvests) * 1e6)
            .sum()
    };
    let grain = crop.good;
    let wages = sim.people().transfers.get(civ_agents::Channel::Wage, grain);
    let margins = sim.people().margins;
    let mut hired = false;
    let mut told: Option<String> = None;
    for _ in 0..3 * 24 {
        sim.advance_minutes(60).expect("advances");
        // What the observer says of a reaper out for hire (§8 18, fourth).
        if told.is_none() {
            told = sim
                .people()
                .people
                .iter()
                .find(|(_, p)| {
                    usize::from(p.act.def) == hire
                        && matches!(p.act.target, civ_agents::person::Target::Field(f)
                            if fields.contains(&f))
                })
                .map(|(_, p)| civ_sim::frames::people::doing(&sim, p));
        }
        hired = sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| neighbours.contains(&p.household))
            .any(|(_, p)| {
                p.receipts.iter().any(|r| {
                    usize::from(r.chosen.def) == hire
                        && matches!(r.chosen.target, civ_agents::person::Target::Field(f)
                            if fields.contains(&f))
                })
            });
        if hired && sim.people().transfers.get(civ_agents::Channel::Wage, grain) > wages {
            break;
        }
    }
    assert!(hired, "a neighbour set out to reap the field for hire");
    // The observer says it is reaping, and whose field it is, not working for wages at their own.
    let told = told.expect("a reaper was seen out for hire");
    let whose = civ_sim::frames::people::household_name(&sim, employer);
    assert!(told.contains("reaping for wages"), "{told}");
    assert!(told.contains(&format!("the field of {whose}")), "{told}");
    assert!(worked(&sim) > 0.0, "the field was reaped");
    assert!(
        sim.people().transfers.get(civ_agents::Channel::Wage, grain) > wages,
        "the reapers were paid in grain"
    );
    // The dashboard's harvest margins count each session paid, its hours and its pay (§8 18,
    // first), a part of the wages paid in the crop's grain.
    let m = sim.people().margins;
    let paid = sim.people().transfers.get(civ_agents::Channel::Wage, grain) - wages;
    assert!(m.reapings > margins.reapings, "{m:?}");
    assert!(m.reaped_h > margins.reaped_h, "{m:?}");
    let reaped_paid = m.reaped_paid_kg - margins.reaped_paid_kg;
    assert!(
        reaped_paid > 0.0 && reaped_paid <= paid + 1e-6,
        "{m:?} {paid}"
    );
}

/// Fuel weighed with hunger (§8 18, sixth; research 08-23 §4, §6): a household short of both
/// fuel and food it can eat as it is, whose grain waits on a fire, weighs firewood by the meals it
/// makes possible, not only by the hours of gathering it saves. A grown woman alone, able to
/// gather her own wood and so saving little by buying it, holds grain beyond her year but no fuel
/// and nothing ready to eat; her neighbours hold firewood to spare. Before, the firewood was worth
/// only the gathering it saved, too little for what they ask; now it is worth the bread it bakes,
/// and she buys it.
#[test]
fn one_whose_grain_waits_on_a_fire_weighs_firewood_by_the_meals_it_makes() {
    let mut sim = new_world(3, 0);
    sim.advance_minutes(200 * 24 * 60).expect("advances");
    let (hh, woman) = lone_elder(&mut sim, 38.0, 450.0);
    let params = sim.rules().people.clone();
    let goods = sim.rules().catalog.goods.clone();
    let wood = goods
        .iter()
        .position(|g| g.id == "core:good/firewood")
        .expect("firewood");
    let now = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        for (_, p) in pop.people.iter_mut().filter(|(_, p)| p.id == woman) {
            let reserve = population::reserve_kcal(p, now, &params);
            p.energy_kcal = (-0.4 * reserve) as f32;
            p.needs_at = now;
        }
        for (_, x) in pop.households.iter_mut() {
            let held = population::stores_now(x, now, &params, &goods);
            x.stores = held;
            x.stores_at = now;
            for (g, d) in goods.iter().enumerate() {
                if d.purpose == civ_agents::params::GoodUse::Fuel {
                    x.stores[g] = 0.0;
                }
                // Nothing she can eat as it is: only the grain, which waits on a fire.
                if x.id == hh
                    && d.purpose == civ_agents::params::GoodUse::Food
                    && d.id != "core:good/grain"
                {
                    x.stores[g] = 0.0;
                }
            }
            if x.id != hh {
                let keep = params.household.fuel_target_days
                    * population::fuel_per_day(&params, x.members.len(), now.day_index());
                x.stores[wood] = 1.5 * keep;
            }
        }
    }
    // Her neighbours review their offers, and post the firewood they hold to spare.
    sim.advance_minutes(24 * 60).expect("lives");
    {
        let now = sim.now();
        let pop = sim.people_mut_for_tests();
        for (_, x) in pop.households.iter_mut() {
            let held = population::stores_now(x, now, &params, &goods);
            x.stores = held;
            x.stores_at = now;
            for (g, d) in goods.iter().enumerate() {
                if x.id == hh
                    && (d.purpose == civ_agents::params::GoodUse::Fuel
                        || (d.purpose == civ_agents::params::GoodUse::Food
                            && d.id != "core:good/grain"))
                {
                    x.stores[g] = 0.0;
                }
            }
        }
    }
    let offered = sim
        .people()
        .households
        .iter()
        .any(|(_, x)| x.id != hh && x.offers.iter().any(|o| usize::from(o.good) == wood));
    assert!(offered, "a neighbour offers firewood");
    let deal = sim.with_ctx_for_tests(|pop, ctx| pop.neighbours_deal_for_tests(ctx, hh));
    assert!(
        deal.is_some_and(|(g, units, _)| g == wood && units >= 1.0),
        "she would buy firewood for the bread it bakes: {deal:?}"
    );
}

/// Fuel offered at market (§8 17, fifth; research 08-16 §1.4: fuel is a household necessity and an
/// input; the lone-elders entry's "Not yet"): an elder of 72, alone, past gathering firewood, holds
/// more grain than their year needs and a quern, but no fuel; the neighbours hold firewood
/// beyond their hearths' days but not so much that they can give it (twice those days). A
/// household with wood to spare offers it as it offers food, and the elder buys firewood at a
/// neighbour's door, paying from what they hold beyond what they keep. Before, firewood was never offered, and with nobody able to
/// give it, none came to them.
#[test]
fn a_lone_elder_with_no_firewood_buys_it_from_a_neighbour_with_wood_to_spare() {
    let mut sim = new_world(3, 0);
    // Past the band's first harvest and its homes.
    sim.advance_minutes(200 * 24 * 60).expect("advances");
    let (hh, elder) = lone_elder(&mut sim, 72.3, 450.0);
    let params = sim.rules().people.clone();
    let goods = sim.rules().catalog.goods.clone();
    let index = |id: &str| goods.iter().position(|g| g.id == id).expect(id);
    let wood = index("core:good/firewood");
    let start = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        for (_, p) in pop.people.iter_mut().filter(|(_, p)| p.id == elder) {
            let reserve = population::reserve_kcal(p, start, &params);
            p.energy_kcal = (-0.4 * reserve) as f32;
            p.burn_kcal_min = 0.0;
            p.needs_at = start;
        }
    }
    // Each neighbour holds half as much firewood again as its hearth's days: some to spare, none
    // to give.
    let stock = |sim: &mut Sim| {
        let now = sim.now();
        let pop = sim.people_mut_for_tests();
        for (_, x) in pop.households.iter_mut().filter(|(_, x)| x.id != hh) {
            let keep = params.household.fuel_target_days
                * population::fuel_per_day(&params, x.members.len(), now.day_index());
            let held = population::stores_now(x, now, &params, &goods);
            x.stores = held;
            x.stores_at = now;
            for (g, d) in goods.iter().enumerate() {
                if d.purpose == civ_agents::params::GoodUse::Fuel {
                    x.stores[g] = 0.0;
                }
            }
            x.stores[wood] = 1.5 * keep;
        }
    };
    let settlement = sim
        .people()
        .household(hh)
        .and_then(|x| x.settlement)
        .expect("their settlement");
    // The elder bought firewood at a neighbour's door: a trade at the market with firewood the
    // good sold and their household the buyer (before, firewood only ever passed as payment).
    let mut bought = 0.0;
    for _ in 0..30 {
        stock(&mut sim);
        let gives = sim
            .people()
            .households
            .iter()
            .filter(|(_, x)| x.id != hh)
            .any(|(_, x)| population::spare_fuel_kg(x, sim.now(), &params, &goods) > 0.0);
        assert!(!gives, "no neighbour can give firewood");
        let since = sim.now();
        sim.advance_minutes(24 * 60).expect("lives");
        bought += sim.people().market(settlement).map_or(0.0, |m| {
            m.recent
                .iter()
                .filter(|t| t.at >= since && usize::from(t.good) == wood && t.buyer == hh)
                .map(|t| f64::from(t.units))
                .sum::<f64>()
        });
        if bought > 0.0 {
            break;
        }
    }
    assert!(
        bought > 0.0,
        "the elder bought firewood at a neighbour's door"
    );
    let x = sim.people().household(hh).expect("their household");
    assert!(
        x.flows.get(Flow::Received, wood) >= bought - 1e-6,
        "the firewood bought came to them"
    );
    // The dashboard's harvest margins read fuel traded from what moved by sale and barter (§8 18,
    // first): the firewood bought is among it.
    let t = &sim.people().transfers;
    let traded = t.get(civ_agents::Channel::Sale, wood) + t.get(civ_agents::Channel::Barter, wood);
    assert!(traded >= bought - 1e-6, "{traded} traded, {bought} bought");
}
