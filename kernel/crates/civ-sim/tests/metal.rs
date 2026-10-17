//! Metal as content (M7d slice CE, ADR-0027 §6): copper ore is picked and dug, smelted by those
//! who know how and cast into an axe that does an axe's work and lasts longer; a household makes
//! whichever axe costs it fewer of its hours for each hour of use; and where nobody knows how to
//! smelt, nobody holds copper.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
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
            name: "Copper".to_owned(),
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

/// The whole world's holdings of `g`.
fn held(sim: &Sim, g: usize) -> f64 {
    sim.people()
        .households
        .iter()
        .map(|(_, x)| x.stores.get(g).copied().unwrap_or(0.0))
        .sum()
}

#[test]
fn a_copper_axe_serves_for_an_axe_in_the_content() {
    let sim = world(3);
    let (axe, copper_axe) = (
        good(&sim, "core:good/axe"),
        good(&sim, "core:good/copper_axe"),
    );
    let goods = &sim.rules().catalog.goods;
    let tool = goods[copper_axe].tool.as_ref().expect("a tool");
    assert_eq!(tool.serves, Some(axe));
    assert!(tool.life_h > goods[axe].tool.as_ref().expect("a tool").life_h);
    assert!(goods[axe].tool.as_ref().is_some_and(|t| t.serves.is_none()));
}

#[test]
fn copper_axes_do_the_work_of_axes_and_wear_in_its_place() {
    let mut sim = world(3);
    let (axe, copper_axe) = (
        good(&sim, "core:good/axe"),
        good(&sim, "core:good/copper_axe"),
    );
    // Every axe a household holds becomes a copper axe.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        x.stores.resize(x.stores.len().max(copper_axe + 1), 0.0);
        x.stores[copper_axe] = x.stores[axe];
        x.stores[axe] = 0.0;
    }
    let before = held(&sim, copper_axe);
    assert!(before > 0.0, "the band brought axes");
    sim.advance_minutes(20 * DAY).expect("advances");
    let after = held(&sim, copper_axe);
    assert!(
        after < before - 1e-6,
        "copper axes were worn: {before} to {after}"
    );
    // Stone axes were made only as the spare kept in hand, never worn while a copper axe was:
    // none holds more than one.
    for (_, x) in sim.people().households.iter() {
        assert!(
            x.stores[axe] <= 1.0 + 1e-9,
            "{} holds {} axes",
            x.id,
            x.stores[axe]
        );
    }
}

#[test]
fn one_who_knows_smelting_smelts_ore_and_casts_a_copper_axe() {
    let mut sim = world(3);
    let goods = [
        "axe",
        "copper_axe",
        "copper",
        "copper_ore",
        "firewood",
        "timber",
    ]
    .map(|g| good(&sim, &format!("core:good/{g}")));
    let [axe, copper_axe, copper, ore, firewood, timber] = goods;
    // The largest household, its axes gone, holds ore, fuel and wood, and its grown members are
    // taught to smelt.
    let now = sim.now();
    let hh = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .max_by_key(|x| (x.members.len(), x.id))
        .map(|x| x.id)
        .expect("a household");
    let grown: Vec<PermanentId> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.household == hh && p.age_years(now) >= 18.0)
        .map(|p| p.id)
        .collect();
    assert!(!grown.is_empty());
    for p in &grown {
        sim.introduce_technique(*p, "core:technique/copper_smelting", false)
            .expect("introduced");
    }
    let n = sim.rules().catalog.goods.len();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == hh {
            x.stores.resize(x.stores.len().max(n), 0.0);
            x.stores[axe] = 0.0;
            x.stores[ore] = 25.0;
            x.stores[firewood] += 200.0;
            x.stores[timber] += 10.0;
        }
    }
    // With ore at home, the copper axe is the way it makes an axe: fewer of its hours for each
    // hour of use.
    let (way, _) = sim.with_ctx_for_tests(|pop, ctx| pop.tool_way_for_tests(ctx, hh, axe));
    assert_eq!(way.as_deref(), Some("core:recipe/cast_copper_axe"));
    let mut cast = false;
    for _ in 0..30 {
        sim.advance_minutes(DAY).expect("advances");
        let x = sim.people().household(hh).expect("household");
        if x.stores[copper_axe] > 0.0 {
            cast = true;
            break;
        }
    }
    let x = sim.people().household(hh).expect("household");
    assert!(
        cast,
        "a copper axe was cast: ore {:.1}, copper {:.2}, axes {:.2}",
        x.stores[ore], x.stores[copper], x.stores[axe]
    );
    assert!(x.stores[ore] < 25.0, "ore was smelted");
    assert!(x.stores[axe] < 1e-9, "no stone axe was made instead");
}

#[test]
fn one_who_knows_smelting_but_no_place_with_ore_still_makes_stone_axes() {
    let mut sim = world(3);
    let goods = ["axe", "copper_axe", "copper_ore", "stone", "timber"]
        .map(|g| good(&sim, &format!("core:good/{g}")));
    let [axe, copper_axe, ore, stone, timber] = goods;
    let now = sim.now();
    let hh = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .max_by_key(|x| (x.members.len(), x.id))
        .map(|x| x.id)
        .expect("a household");
    let grown: Vec<PermanentId> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.household == hh && p.age_years(now) >= 18.0)
        .map(|p| p.id)
        .collect();
    for p in &grown {
        sim.introduce_technique(*p, "core:technique/copper_smelting", false)
            .expect("introduced");
    }
    // Its axes gone, it holds stone and wood for stone ones, no ore, and knows no place with
    // ore (the band has worked no upland patch for it).
    let res = sim
        .rules()
        .land
        .resources
        .iter()
        .position(|r| r.id == "copper_ore")
        .expect("ore") as u16;
    let n = sim.rules().catalog.goods.len();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == hh {
            x.stores.resize(x.stores.len().max(n), 0.0);
            x.stores[axe] = 0.0;
            x.stores[ore] = 0.0;
            x.stores[stone] += 10.0;
            x.stores[timber] += 10.0;
            assert!(x.known.iter().all(|k| k.resource != res));
        }
    }
    let mut made = false;
    for _ in 0..20 {
        sim.advance_minutes(DAY).expect("advances");
        let x = sim.people().household(hh).expect("household");
        if x.stores[axe] > 0.0 {
            made = true;
            break;
        }
    }
    let x = sim.people().household(hh).expect("household");
    assert!(
        made,
        "a stone axe was made, not waited for on ore it cannot get"
    );
    assert!(x.stores[copper_axe] < 1e-9);
}

#[test]
fn nobody_holds_copper_where_nobody_knows_how_to_smelt() {
    let mut sim = world(3);
    let goods =
        ["copper_axe", "copper", "copper_ore"].map(|g| good(&sim, &format!("core:good/{g}")));
    sim.advance_minutes(60 * DAY).expect("advances");
    for g in goods {
        assert!(
            held(&sim, g) < 1e-9,
            "{} held",
            sim.rules().catalog.goods[g].id
        );
    }
}

#[test]
fn weighed_copper_is_worth_what_it_bought_and_less_beyond_a_balance() {
    let mut sim = world(3);
    let goods = ["copper", "grain", "firewood"].map(|g| good(&sim, &format!("core:good/{g}")));
    let [copper, grain, firewood] = goods;
    let (hh, settlement) = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find_map(|x| x.settlement.map(|s| (x.id, s)))
        .expect("a household");
    let seller = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x.id)
        .find(|&id| id != hh)
        .expect("another household");
    let worth = |sim: &mut Sim, p: usize, held: f64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.pay_worth_for_tests(ctx, hh, p, held))
    };
    // Nobody who cannot smelt has a use for copper, and none has bought with it.
    assert_eq!(worth(&mut sim, copper, 0.0), None);
    let grain_h = worth(&mut sim, grain, 0.0).expect("grain has a cost");
    let wood_before = worth(&mut sim, firewood, 0.0);
    // It paid 2 kg of copper for 100 kg of grain today.
    let n = sim.rules().catalog.goods.len();
    let now = sim.now();
    let pop = sim.people_mut_for_tests();
    if !pop.markets.iter().any(|m| m.settlement == settlement) {
        pop.markets.push(civ_agents::market::Market::new(
            settlement,
            n,
            now.day_index(),
        ));
    }
    let market = pop
        .markets
        .iter_mut()
        .find(|m| m.settlement == settlement)
        .expect("market");
    market.recent.push_back(civ_agents::ledger::Trade {
        at: now,
        seller,
        buyer: hh,
        good: grain as u16,
        units: 100.0,
        payment: copper as u16,
        paid: 2.0,
        channel: civ_agents::ledger::Channel::Sale,
        from: None,
    });
    // A kilogram bought 50 kg of grain, less the tenth of an hour of weighing it.
    let w = worth(&mut sim, copper, 0.0).expect("now worth what it bought");
    assert!(
        (w - (grain_h * 50.0 - 0.1)).abs() < 1e-6,
        "{w} against {grain_h}"
    );
    // Holding far more than its spending wants, a kilogram is worth less to it.
    let flush = worth(&mut sim, copper, 50.0).expect("still worth something");
    assert!(flush < w * 0.1, "{flush} against {w}");
    // A good not paid by weight is valued as before.
    assert_eq!(worth(&mut sim, firewood, 0.0), wood_before);
}
