//! Coal (M10a slice DS, first part; ADR-0033 §1, §7): seams lie in the ground with ids of their
//! own, so they move no other id; a household keeps coal for its hearth only if one of them knows
//! how to burn it; that is found by people gathering wood where their settlement knows coal
//! showing, and nowhere else; and those who know it dig coal and burn it by its heat.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::population::DepositKnown;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim};

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

fn world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Coal".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [8; 16],
    )
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

fn technique(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect(id)
}

/// The household with the lowest id, and its members.
fn first_household(sim: &Sim) -> (PermanentId, Vec<PermanentId>) {
    sim.people()
        .households
        .iter()
        .map(|(_, h)| h)
        .min_by_key(|h| h.id)
        .map(|h| (h.id, h.members.clone()))
        .expect("a household")
}

/// Everyone of `members` learns technique `t`.
fn teach(sim: &mut Sim, members: &[PermanentId], t: usize) {
    let learn_h = sim.rules().catalog.techniques[t].learn_h;
    let now = sim.now();
    let teacher = members[0];
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.learn(t, learn_h, teacher, learn_h, now);
        }
    }
}

/// Coal showing beside the first settlement's hearth, which it knows.
fn coal_by_the_hearth(sim: &mut Sim) -> PermanentId {
    let (settlement, hearth) = {
        let s = sim.land().settlements.first().expect("a settlement");
        (s.id, s.hearth_m)
    };
    let coal = sim
        .place_deposit((hearth.0 + 40.0, hearth.1), "core:good/coal", 12.0, true)
        .expect("laid down");
    let at = sim.now();
    let finder = first_household(sim).1[0];
    sim.people_mut_for_tests()
        .deposits_known
        .push(DepositKnown {
            settlement,
            deposit: coal,
            finder,
            at,
        });
    coal
}

#[test]
fn coal_is_reckoned_against_firewood_and_asks_its_technique() {
    let sim = world(3);
    let goods = &sim.rules().catalog.goods;
    let (coal, wood) = (
        good(&sim, "core:good/coal"),
        good(&sim, "core:good/firewood"),
    );
    // Coal comes first in the catalogue, yet the hearth's measure stays firewood, the first fuel
    // anyone burns.
    assert!(coal < wood);
    assert_eq!(civ_agents::params::fuel_ratio(goods, wood), 1.0);
    let ratio = civ_agents::params::fuel_ratio(goods, coal);
    assert!((ratio - 25.8 / 15.6).abs() < 1e-12, "{ratio}");
    let t = technique(&sim, "core:technique/coal_burning");
    assert_eq!(goods[coal].fuel.and_then(|f| f.technique), Some(t));
}

#[test]
fn seams_take_ids_of_their_own() {
    // Over a few seeds, every coal body has a keyed id and every other body one the world gave.
    let mut seams = 0;
    for seed in 1..=6 {
        let sim = world(seed);
        let coal = good(&sim, "core:good/coal");
        let next = sim.ids().peek_next();
        for d in &sim.land().deposits {
            if usize::from(d.body.good) == coal {
                seams += 1;
                assert!(d.id.get() >= civ_land::deposits::KEYED_ID_BASE, "{d:?}");
            } else {
                assert!(d.id.get() < next, "{d:?}");
            }
        }
    }
    assert!(seams > 0, "some seed has coal");
}

#[test]
fn coal_is_kept_for_the_hearth_only_where_someone_knows_how_to_burn_it() {
    let mut sim = world(3);
    let (coal, wood) = (
        good(&sim, "core:good/coal"),
        good(&sim, "core:good/firewood"),
    );
    let (household, members) = first_household(&sim);
    let keep = sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, household));
    assert_eq!(keep[coal], 0.0, "nobody knows how to burn it");
    assert!(keep[wood] > 0.0);
    let burning = technique(&sim, "core:technique/coal_burning");
    teach(&mut sim, &members, burning);
    let keep = sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, household));
    let ratio = civ_agents::params::fuel_ratio(&sim.rules().catalog.goods, coal);
    // The same heat as the firewood it keeps.
    assert!((keep[coal] * ratio - keep[wood]).abs() < 1e-9, "{keep:?}");
}

#[test]
fn burning_coal_is_found_only_where_coal_is_known() {
    let mut sim = world(3);
    let t = technique(&sim, "core:technique/coal_burning");
    // Found at once by anyone who may find it.
    if let Some(rules) = sim.rules_mut_for_tests() {
        rules.catalog.techniques[t].e50_h = 1e-6;
    }
    let knows = |sim: &Sim| {
        sim.people()
            .people
            .iter()
            .filter(|(_, p)| p.knows(t))
            .count()
    };
    sim.advance_minutes(10 * 24 * 60).expect("advances");
    assert_eq!(knows(&sim), 0, "no coal known, none found");
    coal_by_the_hearth(&mut sim);
    sim.advance_minutes(10 * 24 * 60).expect("advances");
    assert!(
        knows(&sim) > 0,
        "gathering wood where coal is known finds it"
    );
}

#[test]
fn those_who_know_dig_coal_and_burn_it() {
    let mut sim = world(3);
    let coal = good(&sim, "core:good/coal");
    let (household, members) = first_household(&sim);
    let burning = technique(&sim, "core:technique/coal_burning");
    teach(&mut sim, &members, burning);
    let seam = coal_by_the_hearth(&mut sim);
    let held = |sim: &Sim| {
        sim.people()
            .household(household)
            .map_or(0.0, |h| h.stores.get(coal).copied().unwrap_or(0.0))
    };
    let mut most = 0.0f64;
    for _ in 0..60 * 24 {
        sim.advance_minutes(60).expect("advances");
        most = most.max(held(&sim));
    }
    let taken = sim
        .land()
        .deposits
        .iter()
        .find(|d| d.id == seam)
        .map_or(0.0, |d| d.taken_kg);
    assert!(taken > 0.0, "coal was dug");
    assert!(most > 0.0, "coal was carried home");
    // The household never holds more than was dug from the seam.
    assert!(most <= taken + 1e-9, "{most} of {taken}");
}

#[test]
fn a_world_with_seams_saves_and_loads_to_the_byte() {
    // Keyed ids are not the world's next ones, and a save holding them loads as it was.
    let mut sim = (1..=6)
        .map(world)
        .find(|sim| {
            let coal = good(sim, "core:good/coal");
            sim.land()
                .deposits
                .iter()
                .any(|d| usize::from(d.body.good) == coal)
        })
        .expect("some seed has coal");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "coal")
        .expect("saves");
    let loaded = civ_sim::persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        civ_sim::persist::encode_sections(&sim),
        civ_sim::persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}
