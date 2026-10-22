//! Iron as content (M10a slice DP; ADR-0033 §3): bog ore smelted in a bloomery batch, the bloom
//! forged into bar and bar into tools that serve for stone ones, all under smithing; and where
//! nobody knows the bloomery, nobody holds iron.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_sim::{NewWorld, Sim};

const DAY: i64 = 24 * 60;

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
            name: "Bloomery".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    let id = format!("core:good/{id}");
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect("a good")
}

fn recipe(sim: &Sim, id: &str) -> civ_agents::params::RecipeDef {
    let id = format!("core:recipe/{id}");
    sim.rules()
        .catalog
        .recipes
        .iter()
        .find(|r| r.id == id)
        .cloned()
        .expect("a recipe")
}

fn activity(sim: &Sim, id: &str) -> usize {
    let id = format!("core:activity/{id}");
    sim.rules()
        .catalog
        .activities
        .iter()
        .position(|a| a.id == id)
        .expect("an activity")
}

/// Kilograms of `g` a unit of `r` takes.
fn takes(r: &civ_agents::params::RecipeDef, g: usize) -> f64 {
    r.inputs
        .iter()
        .filter(|&&(i, _)| i == g)
        .map(|&(_, kg)| kg)
        .sum()
}

/// The whole world's holdings of `g`.
fn held(sim: &Sim, g: usize) -> f64 {
    sim.people()
        .households
        .iter()
        .map(|(_, x)| x.stores.get(g).copied().unwrap_or(0.0))
        .sum()
}

#[test]
fn the_chain_from_ore_to_bar_sits_in_the_research_envelopes_and_a_tonne_of_bar_takes_its_wood() {
    let sim = world(3);
    let [bloom, charcoal, firewood] = ["iron_bloom", "charcoal", "firewood"].map(|g| good(&sim, g));
    let (smelt, bar, burn) = (
        recipe(&sim, "smelt_iron"),
        recipe(&sim, "forge_bar"),
        recipe(&sim, "burn_charcoal"),
    );
    // A kilogram of bar takes its bloom's smelting and its own forging.
    let blooms = takes(&bar, bloom);
    let hours = blooms * smelt.unit_h + bar.unit_h;
    let coal = blooms * takes(&smelt, charcoal) + takes(&bar, charcoal);
    // Research 07-05 §2.3: 6-20 person-hours and 5-30 kg of charcoal a kilogram of usable bar.
    assert!((6.0..=20.0).contains(&hours), "{hours} h a kilogram");
    assert!(
        (5.0..=30.0).contains(&coal),
        "{coal} kg of charcoal a kilogram"
    );
    // The brief's arithmetic (docs/briefs/m10-power.md §1.2): a tonne of bar is 25-150 t of wood.
    let wood_t = coal * takes(&burn, firewood);
    assert!((25.0..=150.0).contains(&wood_t), "{wood_t} t of wood");
    // A smelt is a batch that cools overnight; forging is worked at once.
    assert!(smelt.elapsed_days >= 1.0);
    assert_eq!(bar.elapsed_days, 0.0);
}

#[test]
fn iron_tools_serve_for_the_tools_they_replace_and_outlast_them() {
    let sim = world(3);
    let goods = &sim.rules().catalog.goods;
    let life = |g: usize| goods[g].tool.as_ref().expect("a tool").life_h;
    for (iron, plain) in [
        ("iron_axe", "axe"),
        ("iron_sickle", "sickle"),
        ("iron_hoe", "hoe"),
    ] {
        let (i, p) = (good(&sim, iron), good(&sim, plain));
        assert_eq!(
            goods[i].tool.as_ref().expect("a tool").serves,
            Some(p),
            "{iron}"
        );
        assert!(life(i) > life(p), "{iron}");
    }
    assert!(life(good(&sim, "iron_axe")) > life(good(&sim, "copper_axe")));
}

#[test]
fn metal_is_worked_under_smithing_which_founders_do_not_bring() {
    let sim = world(3);
    let smithing = sim
        .rules()
        .catalog
        .skills
        .iter()
        .position(|s| s.id == "core:skill/smithing")
        .expect("smithing");
    for r in [
        "smelt_copper",
        "cast_copper_axe",
        "smelt_iron",
        "forge_bar",
        "forge_iron_axe",
        "forge_iron_sickle",
        "forge_iron_hoe",
    ] {
        assert_eq!(recipe(&sim, r).skill, Some(smithing), "{r}");
    }
    for (_, p) in sim.people().people.iter() {
        assert!(p.skill(smithing) == 0.0, "{} smiths", p.id);
    }
}

#[test]
fn a_smith_smelts_a_bloom_that_cools_overnight_then_forges_bar_and_an_iron_axe() {
    let mut sim = world(3);
    let goods = [
        "iron_ore",
        "charcoal",
        "iron_bloom",
        "iron",
        "iron_axe",
        "timber",
    ]
    .map(|g| good(&sim, g));
    let [ore, charcoal, bloom, iron, axe, timber] = goods;
    let now = sim.now();
    let (hh, who) = sim
        .people()
        .households
        .iter()
        .find_map(|(_, h)| {
            let grown = h.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 20.0)
            })?;
            Some((h.id, grown))
        })
        .expect("a household with a grown member");
    for t in ["core:technique/bloomery", "core:technique/forging"] {
        sim.introduce_technique(who, t, false).expect("introduced");
    }
    let n = sim.rules().catalog.goods.len();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == hh {
            x.stores.resize(x.stores.len().max(n), 0.0);
            x.stores[ore] = 100.0;
            x.stores[charcoal] = 200.0;
            x.stores[timber] += 5.0;
        }
    }
    let stores = |sim: &Sim| {
        let x = sim.people().household(hh).expect("household");
        civ_agents::population::stores_now(
            x,
            sim.now(),
            &sim.rules().people,
            &sim.rules().catalog.goods,
        )
    };
    let work = |sim: &mut Sim, act: &str, minutes: u32| {
        let a = activity(sim, act);
        sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, who, a, minutes));
    };
    // Charging the furnace takes its ore and charcoal and gives no bloom yet.
    let before = stores(&sim);
    work(&mut sim, "smelt_iron", 480);
    let after = stores(&sim);
    let batches = sim
        .people()
        .household(hh)
        .expect("household")
        .batches
        .clone();
    assert_eq!(batches.len(), 1, "one smelt under way");
    let units = batches[0].units;
    assert!(units > 0.0);
    assert!((before[ore] - after[ore] - 8.0 * units).abs() < 1e-6);
    assert!((before[charcoal] - after[charcoal] - 12.0 * units).abs() < 1e-6);
    assert_eq!(after[bloom], 0.0, "the bloom is drawn once it has cooled");
    // The next day a session draws the bloom, exactly, and charges the furnace again.
    sim.advance_minutes(DAY).expect("advances");
    work(&mut sim, "smelt_iron", 480);
    let drawn = stores(&sim);
    assert!(
        (drawn[bloom] - units).abs() < 1e-9,
        "{} of a bloom of {units}",
        drawn[bloom]
    );
    // Forging the bloom into bar loses a fifth of it.
    work(&mut sim, "forge_bar", 480);
    let barred = stores(&sim);
    let made = barred[iron];
    assert!(made > 0.0, "bar was forged");
    assert!((drawn[bloom] - barred[bloom] - 1.25 * made).abs() < 1e-6);
    // With bar enough, an axe is forged and hafted.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == hh {
            x.stores[iron] += 2.0;
        }
    }
    let ready = stores(&sim);
    work(&mut sim, "forge_iron_axe", 300);
    let done = stores(&sim);
    assert!(done[axe] > ready[axe], "an iron axe was forged");
    // A tool made is worth its maker's quality of a standard one (`make::give`): the bar taken
    // is 1.2 kg a unit worked, the axe credited that unit at a quality between nothing and the
    // smithing skill's best, 1.5.
    let units = (ready[iron] - done[iron]) / 1.2;
    assert!(units > 0.0 && units <= 1.0 + 1e-9, "{units} units");
    let worth = (done[axe] - ready[axe]) / units;
    assert!(worth > 0.0 && worth <= 1.5, "worth {worth}");
}

#[test]
fn nobody_holds_iron_where_nobody_knows_the_bloomery() {
    let mut sim = world(3);
    let goods = [
        "iron_ore",
        "iron_bloom",
        "iron",
        "iron_axe",
        "iron_sickle",
        "iron_hoe",
    ]
    .map(|g| good(&sim, g));
    sim.advance_minutes(60 * DAY).expect("advances");
    for g in goods {
        assert!(
            held(&sim, g) < 1e-9,
            "{} held",
            sim.rules().catalog.goods[g].id
        );
    }
    assert!(
        sim.people()
            .households
            .iter()
            .all(|(_, x)| x.batches.is_empty()),
        "no batch was started"
    );
}
