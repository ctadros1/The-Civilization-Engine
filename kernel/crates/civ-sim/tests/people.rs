//! People live (ADR-0003; plan §7 M1): a founding band arrives and settles, its people sleep,
//! eat, fetch water and gather, they walk only where people can walk, and splitting an advance
//! into pieces changes nothing.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

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
    Sim::create(
        &NewWorld {
            name: "People".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size,
            regime_id: String::new(),
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
            regime_id: String::new(),
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
            regime_id: String::new(),
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
