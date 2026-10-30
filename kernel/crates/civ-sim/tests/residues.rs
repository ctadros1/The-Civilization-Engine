//! The heap's ash and the works' residues (M11f, first part; ADR-0038 §2): what a hearth burns
//! leaves its fuel's ash on the household's heap, which never decays, dungs no field and carries
//! nothing to the water; what a recipe burns leaves `ash` in its maker's store; making gas leaves
//! tar and liquor and the iron recipes slag, so mass closes within each recipe's stated loss; and a
//! heap's ash saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::contagion::Node;
use civ_agents::make;
use civ_agents::params::RecipeDef;
use civ_agents::person::{Flow, Midden};
use civ_agents::population::fuel_per_day;
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
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

fn world(seed: u64, band: u32) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Ashpit".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: band,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [5; 16],
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

fn recipe(sim: &Sim, id: &str) -> RecipeDef {
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

/// Kilograms of `g` a unit of `r` gives.
fn gives(r: &RecipeDef, g: usize) -> f64 {
    r.outputs
        .iter()
        .filter(|&&(o, _)| o == g)
        .map(|&(_, kg)| kg)
        .sum()
}

/// Kilograms of `g` a unit of `r` takes.
fn takes(r: &RecipeDef, g: usize) -> f64 {
    r.inputs
        .iter()
        .filter(|&&(i, _)| i == g)
        .map(|&(_, kg)| kg)
        .sum()
}

/// The ash share of fuel `g`.
fn ash_share(sim: &Sim, g: usize) -> f64 {
    sim.rules().catalog.goods[g]
        .fuel
        .map_or(0.0, |f| f.ash_share)
}

/// The first household with people.
fn first_household(sim: &Sim) -> PermanentId {
    sim.people()
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty())
        .map(|(_, x)| x.id)
        .min()
        .expect("a household")
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

#[test]
fn the_fuels_carry_their_ash_shares() {
    let sim = world(3, 0);
    for (fuel, share) in [
        ("firewood", 0.005),
        ("charcoal", 0.03),
        ("coal", 0.11),
        ("coke", 0.10),
    ] {
        assert_eq!(ash_share(&sim, good(&sim, fuel)), share, "{fuel}");
    }
    // The residues keep and nobody eats them.
    for g in ["ash", "slag", "coal_tar", "ammoniacal_liquor"] {
        let d = &sim.rules().catalog.goods[good(&sim, g)];
        assert_eq!(d.half_life_days, 0.0, "{g} keeps");
        assert_eq!(d.purpose, civ_agents::params::GoodUse::Material, "{g}");
    }
}

#[test]
fn a_hearth_puts_its_firewood_s_ash_on_the_heap_which_never_decays() {
    let mut sim = world(3, 30);
    let firewood = good(&sim, "firewood");
    let h = first_household(&sim);
    let goods = sim.rules().catalog.goods.clone();
    let params = sim.rules().people.clone();
    let midden = params.midden.clone();
    let now = sim.now();
    // A day at the hearth: the firewood burned leaves 0.005 of itself as ash on the heap.
    let mut x = sim.people().household(h).expect("household").clone();
    x.stores.iter_mut().for_each(|kg| *kg = 0.0);
    x.stores[firewood] = 100.0;
    x.stores_at = now;
    x.midden = Midden::begun(now);
    let members = x.members.len();
    let later = SimTime::from_minutes(now.minutes() + DAY);
    x.settle_stores(later, &goods, &|d| fuel_per_day(&params, members, d));
    let burned = 100.0 - x.stores[firewood];
    assert!(burned > 0.0, "the hearth burned firewood");
    assert!(
        (x.midden.ash_kg - 0.005 * burned).abs() < 1e-12,
        "{} kg of ash for {burned} kg burned",
        x.midden.ash_kg
    );
    // Over a year with nobody adding to it, the decaying share halves and the ash stays.
    let mut heap = Midden {
        kg: 1000.0,
        ash_kg: 14.0,
        ..Midden::begun(now)
    };
    let year = SimTime::from_minutes(now.minutes() + 365 * DAY);
    heap.settle(year, 0, &midden);
    assert!((heap.kg - 500.0).abs() < 1e-9, "{} kg decaying", heap.kg);
    assert_eq!(heap.ash_kg, 14.0, "ash never decays");
    // What dungs a field is the decaying share alone: a heap of nothing but ash has none to give.
    let ash_only = Midden {
        ash_kg: 5000.0,
        ..Midden::begun(now)
    };
    assert_eq!(ash_only.at_time(year, 0, &midden), 0.0);

    // Lived: every household's heap gains the ash of the firewood its hearth burned, to the gram.
    sim.advance_minutes(10 * DAY).expect("lives");
    let coal = good(&sim, "coal");
    let (mut ash, mut burned) = (0.0, 0.0);
    for (_, x) in sim.people().households.iter() {
        assert_eq!(x.flows.get(Flow::Burned, coal), 0.0, "nobody burns coal");
        ash += x.midden.ash_kg;
        burned += x.flows.get(Flow::Burned, firewood);
    }
    assert!(burned > 0.0, "hearths burned");
    assert!(
        (ash - 0.005 * burned).abs() < 1e-9 * burned.max(1.0),
        "{ash} kg of ash for {burned} kg of firewood"
    );
}

#[test]
fn what_washes_from_a_heap_is_its_load_whatever_ash_it_holds() {
    // Contagion reads the heap's load, never its kilograms: a heap of tonnes of ash washes as one
    // of none (ADR-0038 §2: ash carries nothing to the water).
    let mut sim = world(3, 30);
    let h = first_household(&sim);
    let d = sim
        .rules()
        .catalog
        .disease_index("core:disease/cholera")
        .map(|i| u16::try_from(i).expect("few diseases"))
        .expect("cholera is content");
    let decay = sim.rules().catalog.diseases[usize::from(d)].decay_per_day;
    let params = sim.rules().land.contamination.clone();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            x.midden.ash_kg = 5000.0;
        }
    }
    sim.people_mut_for_tests()
        .contagion
        .add(Node::Midden(h), d, 1000.0);
    sim.land_mut_for_tests().water.last_surplus_mm = 20.0;
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let kept = 1000.0 * (1.0 - decay);
    let washed = kept * (params.wash_per_mm * 20.0).min(params.wash_max);
    let seeped = (kept - washed) * params.seep_per_day;
    let heap = sim.people().contagion.load(Node::Midden(h), d);
    assert!((heap - (kept - washed - seeped)).abs() < 1e-9, "{heap}");
}

#[test]
fn making_gas_leaves_tar_and_liquor_and_the_iron_recipes_slag_so_mass_closes() {
    let sim = world(3, 0);
    let goods = sim.rules().catalog.goods.clone();
    let [coal, gas, coke, tar, liquor, ash, slag] = [
        "coal",
        "coal_gas",
        "coke",
        "coal_tar",
        "ammoniacal_liquor",
        "ash",
        "slag",
    ]
    .map(|g| good(&sim, g));
    // Gas: 0.14 gas, 0.65 coke, 0.05 tar and 0.13 liquor a kilogram of coal (12-10 §5.7), and the
    // coal's ash beyond what its coke keeps (0.11 - 0.65 x 0.10).
    let r = recipe(&sim, "make_gas");
    assert_eq!(gives(&r, tar), 0.05);
    assert_eq!(gives(&r, liquor), 0.13);
    let mut stores = vec![0.0; goods.len()];
    stores[coal] = 100.0;
    let units = make::apply(&r, &mut stores, &goods, 100.0, 1.0, false, None);
    assert_eq!(units, 100.0);
    assert!((stores[gas] - 14.0).abs() < 1e-9 && (stores[coke] - 65.0).abs() < 1e-9);
    assert!(
        (stores[tar] - 5.0).abs() < 1e-9,
        "{} kg of tar",
        stores[tar]
    );
    assert!(
        (stores[liquor] - 13.0).abs() < 1e-9,
        "{} kg of liquor",
        stores[liquor]
    );
    let left = 100.0 * (0.11 - 0.65 * 0.10);
    assert!(
        (stores[ash] - left).abs() < 1e-9,
        "{} kg of ash",
        stores[ash]
    );
    let out: f64 = stores.iter().sum();
    assert!(
        (out - 100.0).abs() <= 0.05 * 100.0,
        "{out} kg out of 100 kg of coal"
    );
    // Steel: 1.15 kg of pig is 1 kg of steel and 0.15 of slag.
    let [pig, steel, iron, bloom, ore] =
        ["pig_iron", "steel", "iron", "iron_bloom", "iron_ore"].map(|g| good(&sim, g));
    let r = recipe(&sim, "make_steel");
    assert_eq!(gives(&r, slag), 0.15);
    assert!((takes(&r, pig) - gives(&r, steel) - gives(&r, slag)).abs() < 1e-9);
    // The finery: the quarter of the pig the bar does not keep is slag.
    let r = recipe(&sim, "fine_pig_iron");
    assert_eq!(gives(&r, slag), 0.35);
    assert!((takes(&r, pig) - gives(&r, iron) - gives(&r, slag)).abs() < 1e-9);
    // The blast furnace, on charcoal or coke: 0.6 kg of slag a kilogram of pig, within 12-10
    // §5.7's 0.5-1.0, and pig and slag no more than the ore.
    for id in ["smelt_pig_iron", "smelt_pig_iron_with_coke"] {
        let r = recipe(&sim, id);
        let s = gives(&r, slag) / gives(&r, pig);
        assert_eq!(gives(&r, slag), 0.6, "{id}");
        assert!((0.5..=1.0).contains(&s), "{id}: {s}");
        assert!(gives(&r, pig) + gives(&r, slag) <= takes(&r, ore), "{id}");
    }
    // The bloomery: the ore's earth and the iron the bloom does not take, within the ore.
    let r = recipe(&sim, "smelt_iron");
    assert_eq!(gives(&r, slag), 6.0);
    assert!(gives(&r, bloom) + gives(&r, slag) <= takes(&r, ore));
    // Each recipe still gives its metal or gas first: no recipe is for its residue.
    for (id, first) in [
        ("make_gas", gas),
        ("make_steel", steel),
        ("fine_pig_iron", iron),
        ("smelt_pig_iron", pig),
        ("smelt_pig_iron_with_coke", pig),
        ("smelt_iron", bloom),
    ] {
        assert_eq!(recipe(&sim, id).outputs[0].0, first, "{id}");
    }
    // Charring keeps its wood's ash in the charcoal: a burn leaves none.
    let r = recipe(&sim, "burn_charcoal");
    assert_eq!(make::ash_left(&r, &goods, 10.0), 0.0);
}

#[test]
fn a_works_smelting_pig_on_coke_keeps_its_ash_and_slag() {
    // M10a slice DT's blast furnace (tests/works.rs), on coke: its store gains ash at 0.10 of the
    // coke burned and 0.6 kg of slag a kilogram of pig.
    let mut sim = world(3, 0);
    sim.advance_minutes(DAY).expect("advances");
    let [pig, wheel, furnace, coke, ash, slag] = [
        "pig_iron",
        "water_wheel",
        "blast_furnace",
        "coke",
        "ash",
        "slag",
    ]
    .map(|g| good(&sim, g));
    let (owner, who) = grown(&sim)[0];
    let goods = sim.rules().catalog.goods.clone();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == owner {
            for (id, kg) in [
                ("core:good/water_wheel", 1.0),
                ("core:good/blast_furnace", 1.0),
                ("core:good/iron_ore", 400.0),
                ("core:good/coke", 400.0),
                ("core:good/grain", 5000.0),
            ] {
                let g = goods.iter().position(|d| d.id == id).expect(id);
                x.stores[g] = kg;
            }
        }
    }
    let firm = sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, who, pig))
        .expect("a works");
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.put_in_station(ctx, firm, wheel, 1.0)));
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.put_in_station(ctx, firm, furnace, 1.0)));
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
    let smelt = activity(&sim, "smelt_pig_iron_with_coke");
    let start = sim.now();
    let hands: Vec<PermanentId> = grown(&sim).into_iter().map(|(_, p)| p).take(4).collect();
    assert_eq!(hands.len(), 4, "four hands");
    for &h in &hands {
        sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(h, smelt, firm, start));
    }
    let held = |sim: &Sim, g: usize| sim.people().firm(firm).expect("works").stores[g];
    let owner_coke = |sim: &Sim| sim.people().household(owner).expect("owner").stores[coke];
    let (coke_before, ash_before, slag_before, pig_before) = (
        owner_coke(&sim),
        held(&sim, ash),
        held(&sim, slag),
        held(&sim, pig),
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, hands[0], smelt, 120, firm));
    let made = held(&sim, pig) - pig_before;
    let burned = coke_before - owner_coke(&sim);
    assert!(
        made > 0.0 && burned > 0.0,
        "{made} kg of pig on {burned} of coke"
    );
    let ash_made = held(&sim, ash) - ash_before;
    assert!(
        (ash_made - 0.10 * burned).abs() < 1e-6,
        "{ash_made} kg of ash for {burned} kg of coke"
    );
    let slag_made = held(&sim, slag) - slag_before;
    assert!(
        (slag_made - 0.6 * made).abs() < 1e-6,
        "{slag_made} kg of slag for {made} kg of pig"
    );
}

#[test]
fn a_heap_s_ash_saves_and_loads_to_the_byte() {
    let mut sim = world(3, 30);
    sim.advance_minutes(3 * DAY).expect("lives");
    let h = first_household(&sim);
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            x.midden.ash_kg = 14.25;
        }
    }
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "ash")
        .expect("saves");
    let loaded = civ_sim::persist::load(&saved.path, content()).expect("loads");
    let ash_of = |s: &Sim| s.people().household(h).expect("household").midden.ash_kg;
    assert_eq!(ash_of(&loaded), 14.25);
    let (a, b) = (
        civ_sim::persist::encode_sections(&sim),
        civ_sim::persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}
