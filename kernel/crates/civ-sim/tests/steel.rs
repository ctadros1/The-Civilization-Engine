//! Cement, steel and rolling (M11e, first part; ADR-0037 §2): the chain's recipes sit in research
//! 11-14's envelopes, nobody starts knowing it, and a works blows pig to steel and rolls it into
//! sections only with each machine's crew at it.

use std::path::Path;
use std::sync::OnceLock;

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
            name: "Steel".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
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

fn technique(sim: &Sim, id: &str) -> usize {
    let id = format!("core:technique/{id}");
    sim.rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect("a technique")
}

/// Kilograms of `g` a unit of `r` takes.
fn takes(r: &[(usize, f64)], g: usize) -> f64 {
    r.iter().filter(|&&(i, _)| i == g).map(|&(_, kg)| kg).sum()
}

/// Grown people, with their households, oldest households first.
fn grown(sim: &Sim) -> Vec<(PermanentId, PermanentId)> {
    let now = sim.now();
    let mut out: Vec<(PermanentId, PermanentId)> = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| p.age_years(now) >= 20.0)
        .map(|(_, p)| (p.household, p.id))
        .collect();
    out.sort();
    out
}

/// Sets a household's stores of the goods given by content id.
fn set(sim: &mut Sim, household: PermanentId, kg: &[(&str, f64)]) {
    let goods = sim.rules().catalog.goods.clone();
    let h = sim
        .people_mut_for_tests()
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == household)
        .expect("household");
    for &(id, v) in kg {
        let g = goods
            .iter()
            .position(|d| d.id == format!("core:good/{id}"))
            .expect(id);
        h.stores[g] = v;
    }
}

#[test]
fn the_chain_from_limestone_and_pig_sits_in_the_research_envelopes() {
    let sim = world(3);
    let [stone, clay, coal, clinker, pig, steel] = [
        "stone",
        "clay",
        "coal",
        "cement_clinker",
        "pig_iron",
        "steel",
    ]
    .map(|g| good(&sim, g));
    let burn = recipe(&sim, "burn_clinker");
    // Research 11-14 §2.2: about 1.7 t of raw material a tonne of clinker.
    let raw = takes(&burn.inputs, stone) + takes(&burn.inputs, clay);
    assert!((1.5..=1.9).contains(&raw), "{raw} kg of raw a kilogram");
    // And 3.0-6.5 GJ a tonne of clinker, here coal at 28 MJ a kilogram over a full kiln.
    let gj_t = takes(&burn.session_inputs, coal) / burn.max_units * 28.0;
    assert!((3.0..=6.5).contains(&gj_t), "{gj_t} GJ a tonne");
    // Clinker is a burn that runs for days; grinding it is powered and at once.
    assert!(burn.elapsed_days >= 1.0);
    let grind = recipe(&sim, "grind_cement");
    assert!(takes(&grind.inputs, clinker) > 0.0 && grind.kwh_per_unit > 0.0);
    assert_eq!(grind.elapsed_days, 0.0);
    // Steel comes from pig, and sections and bar from steel, by a shaft's work.
    let blow = recipe(&sim, "make_steel");
    assert!(takes(&blow.inputs, pig) >= 1.0 && blow.kwh_per_unit > 0.0);
    for r in ["roll_sections", "roll_bar"] {
        let roll = recipe(&sim, r);
        assert!(
            takes(&roll.inputs, steel) >= 1.0 && roll.kwh_per_unit > 0.0,
            "{r}"
        );
    }
}

#[test]
fn nobody_starts_knowing_cement_steel_or_rolling_and_each_stands_on_the_works_before_it() {
    let sim = world(5);
    let [cement, steelmaking, rolling] =
        ["portland_cement", "steelmaking", "rolling"].map(|t| technique(&sim, t));
    for (_, p) in sim.people().people.iter() {
        for t in [cement, steelmaking, rolling] {
            assert!(!p.knows(t), "a founder knows technique {t}");
        }
    }
    let techniques = &sim.rules().catalog.techniques;
    let requires = |t: usize, on: &str| {
        let on = technique(&sim, on);
        techniques[t]
            .requires
            .iter()
            .all(|route| route.contains(&on))
    };
    assert!(requires(cement, "lime_burning"));
    assert!(requires(steelmaking, "blast_furnace"));
    assert!(requires(rolling, "steelmaking"));
}

#[test]
fn a_works_blows_pig_to_steel_and_rolls_sections_only_with_each_machines_crew() {
    let mut sim = world(3);
    sim.advance_minutes(24 * 60).expect("advances");
    let [steel, section, converter, mill, wheel, pig, coal] = [
        "steel",
        "steel_section",
        "converter",
        "rolling_mill",
        "water_wheel",
        "pig_iron",
        "coal",
    ]
    .map(|g| good(&sim, g));
    let (owner, who) = grown(&sim)[0];
    set(
        &mut sim,
        owner,
        &[
            ("pig_iron", 400.0),
            ("coal", 400.0),
            ("water_wheel", 1.0),
            ("converter", 1.0),
            ("rolling_mill", 1.0),
            ("grain", 5000.0),
        ],
    );
    let firm = sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, who, steel))
        .expect("a works");
    for tool in [wheel, converter, mill] {
        assert!(sim.with_ctx_for_tests(|pop, ctx| pop.put_in_station(ctx, firm, tool, 1.0)));
    }
    sim.land_mut_for_tests()
        .installations
        .push(civ_land::power::Installation {
            id: PermanentId::from_raw(9_000_003).expect("id"),
            reach: 0,
            at: [0.0, 0.0],
            head_m: 2.0,
            race_m3s: 0.5,
            efficiency: 0.3,
            rated_kw: 3.0,
            owner: firm,
            building: None,
        });
    sim.land_mut_for_tests().reach_discharges[0] = 5.0;
    sim.people_mut_for_tests()
        .firms
        .iter_mut()
        .find(|f| f.id == firm)
        .expect("works")
        .add_line(section as u16);
    let (blow, roll) = (
        activity(&sim, "make_steel"),
        activity(&sim, "roll_sections"),
    );
    let hands: Vec<PermanentId> = grown(&sim)
        .into_iter()
        .map(|(_, p)| p)
        .filter(|&p| p != who)
        .take(3)
        .collect();
    assert_eq!(hands.len(), 3, "three more hands");
    let start = sim.now();
    let held = |sim: &Sim, g: usize| sim.people().firm(firm).expect("works").stores[g];
    let owned = |sim: &Sim, g: usize| sim.people().household(owner).expect("owner").stores[g];
    let made = |sim: &mut Sim, act: usize, g: usize| {
        let before = held(sim, g);
        sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, who, act, 120, firm));
        held(sim, g) - before
    };
    // Two at the converter are one too few; the third blows it.
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(who, blow, firm, start));
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hands[0], blow, firm, start));
    assert_eq!(
        made(&mut sim, blow, steel),
        0.0,
        "two hands idle a converter of three"
    );
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hands[1], blow, firm, start));
    let pig_before = owned(&sim, pig);
    let blown = made(&mut sim, blow, steel);
    assert!(blown > 0.0, "three hands blow steel: {blown}");
    assert!(
        (pig_before - owned(&sim, pig) - 1.15 * blown).abs() < 1e-6,
        "1.15 kg of pig a kilogram of steel"
    );
    // Three at the mill are one too few; the fourth rolls the works' own steel.
    for &h in std::iter::once(&who).chain(&hands[..2]) {
        sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(h, roll, firm, start));
    }
    assert_eq!(
        made(&mut sim, roll, section),
        0.0,
        "three hands idle a mill of four"
    );
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hands[2], roll, firm, start));
    let (steel_before, coal_before) = (held(&sim, steel), owned(&sim, coal));
    let rolled = made(&mut sim, roll, section);
    assert!(rolled > 0.0, "four hands roll sections: {rolled}");
    assert!(
        (steel_before - held(&sim, steel) - 1.05 * rolled).abs() < 1e-6,
        "1.05 kg of steel a kilogram of section"
    );
    assert!(
        (coal_before - owned(&sim, coal) - 0.3 * rolled).abs() < 1e-6,
        "0.3 kg of coal to reheat a kilogram"
    );
}
