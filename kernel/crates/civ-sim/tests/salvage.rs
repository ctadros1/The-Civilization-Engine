//! What is thrown away (backlog C's first slice; ADR-0044 §1): a worn iron tool leaves half the
//! bar it took as scrap iron on its holder's heap, in proportion to the share worn, and a tool
//! that names no scrap leaves nothing; the heap's discards never decay, are not dung and lie with
//! the heap; carried to a tip they arrive whole and are recorded there; the destructor draws only
//! a tip's decaying share; and a world with discards on heaps and a tip saves and loads exactly.
//!
//! Picking (backlog C's second slice; ADR-0044 §2-§3): a member sorting a tip or its heap takes
//! out exactly what it carries, at `sort_kg_h` times the good's share, and leaves the disease
//! where it lies; picking's cost a kilogram falls as the good's share of the tip rises; of two
//! households, the one whose hour brings less food picks where the other buys; a household fires
//! bricks with ash only where picking ash beats digging clay; and a posted round's fee falls below
//! zero when a buyer of what its keeper sorts out appears, the keeper then paying the homes on
//! `Refuse`, and the world saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::contagion::Node;
use civ_agents::ledger::Channel;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind, Tip};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};
use civ_world::WATER_LAND;

const PRESET: &str = "core:worldgen/river_valley";
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

/// A world of `seed` whose band has just settled, as the crime target's.
fn world_with(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Tips".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
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

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "tips").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

/// A law of `kind` in force in the first polity, every adult of its settlement knowing it, as the
/// crime target's: its id.
fn a_law(sim: &mut Sim, kind: PolicyKind) -> PermanentId {
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.kind == kind)
        .expect("the core content has the template");
    let id = sim.allocate_id_for_tests();
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people_mut_for_tests();
    let settlement = pop.polities[0].settlement;
    let mut adults: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(settlement))
        .flat_map(|(_, h)| h.members.clone())
        .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
        .collect();
    adults.sort_unstable();
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: 0.0,
        holder: None,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: adults[0],
        proposed: now,
        issue: IssueKind::HeapsNear,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: adults.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    };
    for &a in &adults {
        law.learn(a, now.day_index());
    }
    pop.polities[0].laws.push(law);
    id
}

/// The refuse law passed in the first polity, its tip opened as passing it opens one: the law and
/// the tip, if one opened.
fn the_law_passed(sim: &mut Sim) -> (PermanentId, Option<Tip>) {
    let law = a_law(sim, PolicyKind::RefuseToTip);
    sim.with_ctx_for_tests(|pop, ctx| pop.open_tip_for_tests(ctx, 0, law));
    let tip = sim.people().polities[0]
        .tips
        .iter()
        .find(|t| t.law == law)
        .cloned();
    (law, tip)
}

/// Households of the first polity's settlement with people, by id.
fn our_households(sim: &Sim) -> Vec<PermanentId> {
    let settlement = sim.people().polities[0].settlement;
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    out.sort_unstable();
    out
}

/// The catalog index of good `id`.
fn good(sim: &Sim, id: &str) -> usize {
    sim.rules().catalog.good_index(id).expect("a core good")
}

/// Household `h`'s midden discards.
fn discards(sim: &Sim, h: PermanentId) -> Vec<(u16, f64)> {
    sim.people()
        .household(h)
        .expect("there")
        .midden
        .discards
        .clone()
}

/// Gives household `h` `n` of good `g`.
fn give(sim: &mut Sim, h: PermanentId, g: usize, n: f64) {
    let pop = sim.people_mut_for_tests();
    let (_, x) = pop
        .households
        .iter_mut()
        .find(|(_, x)| x.id == h)
        .expect("the household");
    x.stores.resize(x.stores.len().max(g + 1), 0.0);
    x.stores[g] = n;
}

#[test]
fn a_worn_iron_tool_leaves_its_scrap_on_the_heap_in_proportion_to_its_wear() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let (axe, stone_axe, scrap) = (
        good(&sim, "core:good/iron_axe"),
        good(&sim, "core:good/axe"),
        good(&sim, "core:good/scrap_iron"),
    );
    let life = sim.rules().catalog.goods[axe]
        .tool
        .as_ref()
        .expect("a tool")
        .life_h;
    let h = our_households(&sim)[0];
    assert!(discards(&sim, h).is_empty(), "nothing thrown on it yet");
    give(&mut sim, h, axe, 1.0);
    give(&mut sim, h, stone_axe, 0.0);
    // Half its life worn: half a tool gone, and half its scrap on the heap.
    sim.with_ctx_for_tests(|pop, ctx| pop.wear_tools_for_tests(ctx, h, &[axe], life / 2.0));
    let x = sim.people().household(h).expect("there");
    assert!((x.stores[axe] - 0.5).abs() < 1e-12, "{}", x.stores[axe]);
    let d = discards(&sim, h);
    assert_eq!(d.len(), 1);
    assert_eq!(usize::from(d[0].0), scrap);
    assert!((d[0].1 - 0.3).abs() < 1e-12, "{}", d[0].1);
    // Worn through: the tool is gone and 0.6 kg lies on the heap in all; more wear leaves no more.
    sim.with_ctx_for_tests(|pop, ctx| pop.wear_tools_for_tests(ctx, h, &[axe], life));
    let x = sim.people().household(h).expect("there");
    assert_eq!(x.stores[axe], 0.0);
    let d = discards(&sim, h);
    assert!((d[0].1 - 0.6).abs() < 1e-12, "{}", d[0].1);
    // A stone axe names no scrap: worn through, it leaves nothing.
    let other = our_households(&sim)[1];
    give(&mut sim, other, stone_axe, 1.0);
    give(&mut sim, other, axe, 0.0);
    let stone_life = sim.rules().catalog.goods[stone_axe]
        .tool
        .as_ref()
        .expect("a tool")
        .life_h;
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.wear_tools_for_tests(ctx, other, &[stone_axe], 2.0 * stone_life)
    });
    assert_eq!(
        sim.people().household(other).expect("there").stores[stone_axe],
        0.0
    );
    assert!(discards(&sim, other).is_empty());
    // The heap's discards are inert: a year of the heap decaying leaves them as they were.
    sim.advance_minutes(30 * DAY).expect("advances");
    let d = discards(&sim, h);
    assert!((d[0].1 - 0.6).abs() < 1e-12, "{}", d[0].1);
}

#[test]
fn discards_carried_to_a_tip_arrive_whole_and_the_world_saves_and_loads_exactly() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let (_, tip) = the_law_passed(&mut sim);
    let tip = tip.expect("a place qualifies in this valley, or this test shows nothing");
    let (axe, sickle, scrap) = (
        good(&sim, "core:good/iron_axe"),
        good(&sim, "core:good/iron_sickle"),
        good(&sim, "core:good/scrap_iron"),
    );
    let households = our_households(&sim);
    let (h, other) = (households[0], households[1]);
    // Two worn tools on one heap, one on another.
    give(&mut sim, h, axe, 1.0);
    give(&mut sim, h, sickle, 1.0);
    give(&mut sim, other, axe, 1.0);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.wear_tools_for_tests(ctx, h, &[axe, sickle], 1.0e6);
        pop.wear_tools_for_tests(ctx, other, &[axe], 1.0e6);
    });
    let before = discards(&sim, h);
    assert_eq!(before, vec![(scrap as u16, 0.6 + 0.2)]);
    let carrier = sim.people().household(h).expect("there").members[0];
    sim.with_ctx_for_tests(|pop, ctx| pop.carry_midden_for_tests(ctx, carrier, tip.place));
    // The heap is empty of them; the tip holds them whole.
    assert!(discards(&sim, h).is_empty());
    let t = &sim.people().polities[0].tips[0];
    assert_eq!(t.discards, before, "what left the heap lies at the tip");
    // The destructor's draw takes only a tip's decaying share, never its discards (ADR-0044 §1):
    // its draw reads `kg` alone, so a tip of discards and nothing else has nothing to burn.
    // Saved with discards on a heap and at a tip, the world loads exactly and goes on alike.
    assert_eq!(discards(&sim, other).len(), 1);
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// The index of cholera, a disease the core has.
fn cholera(sim: &Sim) -> u16 {
    let i = sim
        .rules()
        .catalog
        .disease_index("core:disease/cholera")
        .expect("cholera is content");
    u16::try_from(i).expect("few diseases")
}

/// Sets the first polity's first tip to hold `kg` of decaying share, `ash_kg` of ash and
/// `discards`.
fn set_tip(sim: &mut Sim, kg: f64, ash_kg: f64, discards: Vec<(u16, f64)>) {
    let t = &mut sim.people_mut_for_tests().polities[0].tips[0];
    t.kg = kg;
    t.ash_kg = ash_kg;
    t.discards = discards;
}

/// Sets household `h`'s heap to `kg` of decaying share, `ash_kg` of ash and `discards`, as of now.
fn set_heap(sim: &mut Sim, h: PermanentId, kg: f64, ash_kg: f64, discards: Vec<(u16, f64)>) {
    let now = sim.now();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            x.midden.kg = kg;
            x.midden.ash_kg = ash_kg;
            x.midden.discards = discards.clone();
            x.midden.at = now;
        }
    }
}

/// The first grown member of household `h`, and what a capable adult's hour is to them: their
/// share of it at their age.
fn a_grown_member(sim: &Sim, h: PermanentId) -> (PermanentId, f64) {
    let now = sim.now();
    let people = &sim.rules().people;
    let pop = sim.people();
    pop.household(h)
        .expect("there")
        .members
        .iter()
        .filter_map(|&m| pop.person(m))
        .find(|p| p.age_years(now) >= 16.0)
        .map(|p| {
            (
                p.id,
                civ_agents::params::interpolate(&people.capacity_by_age, p.age_years(now)),
            )
        })
        .expect("a grown member")
}

/// A world of seed 5 two days on, its first polity's tip law passed and the tip open: the world
/// and the tip.
fn a_village_with_a_tip() -> (Sim, Tip) {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let (_, tip) = the_law_passed(&mut sim);
    let tip = tip.expect("a place qualifies in this valley, or this test shows nothing");
    (sim, tip)
}

#[test]
fn picking_takes_exactly_what_it_carries_from_a_tip_and_a_heap_and_leaves_disease_there() {
    let (mut sim, tip) = a_village_with_a_tip();
    let (scrap, ash) = (
        good(&sim, "core:good/scrap_iron"),
        good(&sim, "core:good/ash"),
    );
    let sort = sim.rules().people.midden.sort_kg_h;
    // A tonne at the tip: 900 kg of decaying refuse, 50 of ash and 50 of scrap iron, with cholera
    // in it.
    set_tip(&mut sim, 900.0, 50.0, vec![(scrap as u16, 50.0)]);
    let d = cholera(&sim);
    sim.people_mut_for_tests()
        .contagion
        .add(Node::Tip(tip.id), d, 500.0);
    let h = our_households(&sim)[0];
    let (who, eff) = a_grown_member(&sim, h);
    // An hour's sorting there brings out the scrap's share of what was sorted: a twentieth.
    let kg = sim.with_ctx_for_tests(|pop, ctx| pop.pick_for_tests(ctx, who, tip.place, scrap, 60));
    eprintln!("an hour at the tip: {kg} kg of scrap");
    assert!((kg - sort * eff * 0.05).abs() < 1e-9, "{kg}");
    let t = sim.people().polities[0].tips[0].clone();
    assert_eq!(
        t.discards,
        vec![(scrap as u16, 50.0 - kg)],
        "out of the tip exactly"
    );
    assert_eq!((t.kg, t.ash_kg), (900.0, 50.0), "the rest lies as it was");
    let p = sim.people().person(who).expect("there");
    assert_eq!(p.carrying.good, Some(scrap as u16));
    assert_eq!(p.carrying.kg, kg as f32, "carried home whole");
    assert_eq!(
        sim.people().contagion.load(Node::Tip(tip.id), d),
        500.0,
        "the disease stays where it lies"
    );
    // Half an hour sifting it for ash: the ash's share of what is left.
    let total = 900.0 + 50.0 + (50.0 - kg);
    let ash_kg =
        sim.with_ctx_for_tests(|pop, ctx| pop.pick_for_tests(ctx, who, tip.place, ash, 30));
    assert!((ash_kg - sort * eff * 0.5 * 50.0 / total).abs() < 1e-9);
    let t = sim.people().polities[0].tips[0].clone();
    assert_eq!(t.ash_kg, 50.0 - ash_kg);
    assert_eq!(t.kg, 900.0);
    // At its own heap at home: 95 kg of decaying share and 5 of ash.
    set_heap(&mut sim, h, 95.0, 5.0, Vec::new());
    sim.people_mut_for_tests()
        .contagion
        .add(Node::Midden(h), d, 200.0);
    let home = sim.people().household(h).expect("there").home;
    let from_heap = sim.with_ctx_for_tests(|pop, ctx| pop.pick_for_tests(ctx, who, home, ash, 60));
    assert!(
        (from_heap - (sort * eff * 0.05).min(5.0)).abs() < 1e-9,
        "{from_heap}"
    );
    let x = sim.people().household(h).expect("there");
    assert_eq!(x.midden.ash_kg, 5.0 - from_heap);
    assert!(
        (x.midden
            .at_time(sim.now(), x.members.len(), &sim.rules().people.midden)
            - 95.0)
            .abs()
            < 1e-9
    );
    assert_eq!(sim.people().contagion.load(Node::Midden(h), d), 200.0);
    // Nothing of a good the place does not hold, and nothing standing at no place to pick over.
    let none = sim.with_ctx_for_tests(|pop, ctx| pop.pick_for_tests(ctx, who, home, scrap, 60));
    assert_eq!(none, 0.0);
    let far = (tip.place.0 + 500.0, tip.place.1 + 500.0);
    let none = sim.with_ctx_for_tests(|pop, ctx| pop.pick_for_tests(ctx, who, far, scrap, 60));
    assert_eq!(none, 0.0);
}

#[test]
fn picking_costs_fewer_hours_a_kilogram_the_more_of_the_tip_is_that_good() {
    let (mut sim, tip) = a_village_with_a_tip();
    let scrap = good(&sim, "core:good/scrap_iron");
    let h = our_households(&sim)[0];
    set_heap(&mut sim, h, 50.0, 0.0, Vec::new());
    let cost =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.own_costs_for_tests(ctx, h))[scrap];
    // With no scrap anywhere it may pick over, the household has no way to it.
    set_tip(&mut sim, 1000.0, 0.0, Vec::new());
    assert_eq!(cost(&mut sim), None);
    let people = &sim.rules().people;
    let (sort, carry, walk_ms) = (
        people.midden.sort_kg_h,
        people.household.carry_kg,
        people.nav.tobler_ms(0.0),
    );
    let home = sim.people().household(h).expect("there").home;
    let walk_h =
        f64::from(home.0 - tip.place.0).hypot(f64::from(home.1 - tip.place.1)) / walk_ms / 3600.0;
    let mut last = f64::INFINITY;
    for share in [0.01, 0.05, 0.2] {
        let kg = 1000.0 * share;
        set_tip(&mut sim, 1000.0 - kg, 0.0, vec![(scrap as u16, kg)]);
        let c = cost(&mut sim).expect("picking is a way to it");
        let expected = 1.0 / (sort * share) + 2.0 * walk_h / carry.min(kg);
        eprintln!("scrap at a share of {share}: {c:.4} h a kg");
        assert!((c - expected).abs() < 1e-9, "{c} against {expected}");
        assert!(c < last, "dearer as the share fell");
        last = c;
    }
}

#[test]
fn the_household_whose_hour_brings_less_food_picks_where_the_other_buys() {
    let (mut sim, tip) = a_village_with_a_tip();
    let scrap = good(&sim, "core:good/scrap_iron");
    set_tip(&mut sim, 950.0, 0.0, vec![(scrap as u16, 50.0)]);
    let households = our_households(&sim);
    let (poor, rich, seller) = (households[0], households[1], households[2]);
    for &h in &[poor, rich, seller] {
        set_heap(&mut sim, h, 20.0, 0.0, Vec::new());
    }
    // The richer household knows a place where wild food comes ten times as fast as any the
    // land promises: an hour of its work brings it more food.
    let (resource, food) = {
        let land = &sim.rules().land;
        let goods = &sim.rules().catalog.goods;
        land.resources
            .iter()
            .enumerate()
            .find(|(_, r)| goods[r.good].kcal_per_kg > 0.0 && r.max_rate_per_hour > 0.0)
            .map(|(i, r)| (i, r.good))
            .expect("a wild food")
    };
    let rate = sim.rules().land.resources[resource].max_rate_per_hour * 10.0;
    let today = sim.now().day_index();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == rich {
            x.known.push(civ_agents::person::KnownPatch {
                resource: resource as u16,
                patch: 0,
                rate: rate as f32,
                hours: 1000.0,
                seen_day: today,
            });
        }
    }
    let costs = |sim: &mut Sim, h: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.own_costs_for_tests(ctx, h))
    };
    let (c_poor, c_rich) = (
        costs(&mut sim, poor)[food].expect("the poorer can get food"),
        costs(&mut sim, rich)[food].expect("the richer can get food"),
    );
    let kcal_h = |sim: &mut Sim, h: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.hour_kcal_for_tests(ctx, h))
    };
    let (k_poor, k_rich) = (kcal_h(&mut sim, poor), kcal_h(&mut sim, rich));
    eprintln!(
        "food: {c_poor:.3} and {c_rich:.3} h a kg; an hour brings {k_poor:.0} and {k_rich:.0} kcal"
    );
    assert!(c_rich < c_poor && k_poor < k_rich);
    // Before anyone sells scrap, both would pick it at the tip, at the same hours a kilogram.
    let place = |sim: &mut Sim, h: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.pick_place_for_tests(ctx, h, scrap))
    };
    let (p_poor, walk, kg_h) = place(&mut sim, poor).expect("the poorer would pick");
    assert_eq!(p_poor.tip, Some(tip.id));
    assert!(place(&mut sim, rich).is_some(), "so would the richer");
    let carry = sim.rules().people.household.carry_kg;
    let picked_h = 1.0 / kg_h + 2.0 * walk / 60.0 / carry.min(50.0);
    // A neighbour offers scrap for that food at a price between what picking costs each in food.
    let price = picked_h / (c_poor * c_rich).sqrt();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == seller {
            x.offers = vec![civ_agents::market::Offer {
                good: scrap as u16,
                payment: food as u16,
                price: price as f32,
                units: 5.0,
                ask_h: 1.0,
            }];
        }
    }
    let bought = |sim: &mut Sim, h: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.bought_h_for_tests(ctx, h, scrap))
            .expect("offered")
    };
    let (b_poor, b_rich) = (bought(&mut sim, poor), bought(&mut sim, rich));
    eprintln!("picking {picked_h:.3} h a kg; buying {b_poor:.3} and {b_rich:.3}");
    assert!(b_rich < picked_h && picked_h < b_poor);
    // The household whose hour brings less food still picks; the other would rather buy.
    assert!(place(&mut sim, poor).is_some(), "the poorer picks");
    assert!(place(&mut sim, rich).is_none(), "the richer buys");
}

#[test]
fn a_brickmaker_takes_ash_only_where_picking_it_beats_digging_clay() {
    let (mut sim, tip) = a_village_with_a_tip();
    let (ash, clay, brick) = (
        good(&sim, "core:good/ash"),
        good(&sim, "core:good/clay"),
        good(&sim, "core:good/brick"),
    );
    let h = our_households(&sim)[0];
    set_heap(&mut sim, h, 20.0, 0.0, Vec::new());
    // Someone of the household fires bricks.
    let firing = sim
        .rules()
        .catalog
        .technique_index("core:technique/firing")
        .expect("firing");
    let (who, _) = a_grown_member(&sim, h);
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == who {
            p.come_to_know(firing, civ_agents::person::KnowSource::Observer, now);
        }
    }
    // Clay showing a long walk from home, which the settlement knows.
    let (settlement, hearth) = {
        let s = sim.land().settlements.first().expect("a settlement");
        (s.id, s.hearth_m)
    };
    let deposit = [(1500.0, 0.0), (-1500.0, 0.0), (0.0, 1500.0), (0.0, -1500.0)]
        .iter()
        .find_map(|&(dx, dy)| {
            sim.place_deposit((hearth.0 + dx, hearth.1 + dy), "core:good/clay", 8.0, true)
                .ok()
        })
        .expect("dry land a long walk off");
    sim.people_mut_for_tests()
        .deposits_known
        .push(civ_agents::population::DepositKnown {
            settlement,
            deposit,
            finder: who,
            at: now,
        });
    let dug = sim
        .with_ctx_for_tests(|pop, ctx| pop.dug_h_for_tests(ctx, h, clay))
        .expect("it knows the clay");
    let way = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.material_way_for_tests(ctx, h, brick))
            .expect("a way to bricks")
    };
    let ash_h = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.own_costs_for_tests(ctx, h))[ash]
            .expect("ash lies to pick")
    };
    let _ = tip;
    let mut seen = Vec::new();
    for share in [0.01, 0.95] {
        set_tip(&mut sim, 1000.0 * (1.0 - share), 1000.0 * share, Vec::new());
        let (w, a) = (way(&mut sim), ash_h(&mut sim));
        eprintln!("ash {share}: {a:.4} h a kg against clay dug at {dug:.4}: {w}");
        // The brick body takes ash in place of clay exactly where picking it costs less than
        // digging the clay it replaces.
        assert_eq!(w == "core:recipe/fire_bricks_with_ash", a < dug, "{w}");
        seen.push(w);
    }
    assert_eq!(
        seen,
        vec![
            "core:recipe/fire_bricks".to_owned(),
            "core:recipe/fire_bricks_with_ash".to_owned()
        ],
        "dear ash is left for clay; cheap ash goes in the bricks"
    );
}

/// A handcart made and kept by household `owner`, as the rounds target's: its id.
fn a_handcart(sim: &mut Sim, owner: PermanentId) -> PermanentId {
    let vehicle = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart");
    let (id, today) = (sim.allocate_id_for_tests(), sim.now().day_index());
    let at = sim
        .people()
        .household(owner)
        .map(|x| civ_agents::population::cell_of(sim.map(), x.home) as u32)
        .expect("the keeper");
    let land = sim.land_mut_for_tests();
    land.carts.list.push(civ_land::boats::Boat {
        id,
        vehicle: vehicle as u16,
        owner,
        at,
        made: today,
        wear: 0.0,
        labour_h: 0.0,
        work_h: 0.0,
        saves_h_year: 0.0,
        crew: (0, 0),
    });
    land.carts.changed();
    id
}

/// A cart way over every dry cell about the settlement's homes and `also`, a few cells beyond, as
/// the rounds target's.
fn cart_ground(sim: &mut Sim, also: (f32, f32)) {
    use civ_land::ways::{CART, Keeper, Rank, Segment, Surface, WALK, Way};
    let map = sim.map().clone();
    let size = map.cell_size_m;
    let places: Vec<(f32, f32)> = our_households(sim)
        .iter()
        .filter_map(|&h| sim.people().household(h).map(|x| x.home))
        .chain([also])
        .collect();
    let (x0, x1, y0, y1) = places.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |(a, b, c, d), &(x, y)| (a.min(x), b.max(x), c.min(y), d.max(y)),
    );
    let cell = |v: f32, n: u32| ((v / size).floor() as i64).clamp(0, i64::from(n) - 1);
    let pad = 3.0 * size;
    let (cx0, cx1) = (cell(x0 - pad, map.width), cell(x1 + pad, map.width));
    let (cy0, cy1) = (cell(y0 - pad, map.height), cell(y1 + pad, map.height));
    let width = i64::from(map.width);
    let cells: Vec<u32> = (cy0..=cy1)
        .flat_map(|y| (cx0..=cx1).map(move |x| (y * width + x) as u32))
        .filter(|&c| map.water[c as usize] == WATER_LAND)
        .collect();
    let (way, today) = (sim.allocate_id_for_tests(), sim.now().day_index());
    let land = sim.land_mut_for_tests();
    land.ways.add(Way {
        id: way,
        begun: today,
        segments: vec![Segment {
            cells,
            corridor_m: 4.0,
            built_m: 4.0,
            usable_m: 4.0,
            surface: Surface::Cleared,
            keeper: Keeper::Nobody,
            kept: today,
        }],
        modes: WALK | CART,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
    land.ways.changed();
}

/// Food moved by `channel` since the counters began, kcal.
fn food_kcal(sim: &Sim, channel: Channel) -> f64 {
    let goods = &sim.rules().catalog.goods;
    (0..goods.len())
        .map(|g| sim.people().transfers.get(channel, g) * goods[g].kcal_per_kg)
        .sum()
}

#[test]
fn a_posted_rounds_fee_falls_below_zero_when_a_buyer_of_what_it_sorts_out_appears() {
    let (mut sim, tip) = a_village_with_a_tip();
    let households = our_households(&sim);
    let (keeper, buyer) = (households[0], households[1]);
    a_handcart(&mut sim, keeper);
    cart_ground(&mut sim, tip.place);
    let (ash, brick) = (good(&sim, "core:good/ash"), good(&sim, "core:good/brick"));
    // The tip is two parts in five ash; each home's heap half ash, as a coal town's refuse was
    // mostly ash (research 12-10 §2.1).
    set_tip(&mut sim, 600.0, 400.0, Vec::new());
    for &h in &households {
        set_heap(&mut sim, h, 50.0, 50.0, Vec::new());
    }
    let review = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_refuse_round_for_tests(ctx, keeper));
        sim.people().polities[0]
            .rounds
            .iter()
            .find(|r| r.operator == keeper)
            .cloned()
    };
    // Nobody wants the ash: the homes the tip law asks to carry pay the keeper's hours.
    let round = review(&mut sim).expect("homes that carry under the law join a round");
    eprintln!(
        "no buyer: {:.0} kcal a call, {} homes",
        round.fee_kcal,
        round.homes.len()
    );
    assert!(round.fee_kcal > 0.0);
    // A household that would build a refuse destructor to drive a dynamo appears, refuse to
    // burn in it, its bar and timber at hand, a long way from any clay it knows, and its hour
    // bringing much food: it wants the bricks' ash.
    let now = sim.now();
    let catalog = &sim.rules().catalog;
    let techniques: Vec<usize> = [
        "core:technique/refuse_destructor",
        "core:technique/firing",
        "core:technique/dynamo",
    ]
    .iter()
    .map(|t| catalog.technique_index(t).expect(t))
    .collect();
    let (iron, timber) = (good(&sim, "core:good/iron"), good(&sim, "core:good/timber"));
    let members = sim
        .people()
        .household(buyer)
        .expect("there")
        .members
        .clone();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            for &t in &techniques {
                p.come_to_know(t, civ_agents::person::KnowSource::Observer, now);
            }
        }
    }
    give(&mut sim, buyer, iron, 1200.0);
    give(&mut sim, buyer, timber, 200.0);
    let refuse = good(&sim, "core:good/refuse");
    give(&mut sim, buyer, refuse, 100.0);
    let (settlement, hearth) = {
        let s = sim.land().settlements.first().expect("a settlement");
        (s.id, s.hearth_m)
    };
    let deposit = [(1500.0, 0.0), (-1500.0, 0.0), (0.0, 1500.0), (0.0, -1500.0)]
        .iter()
        .find_map(|&(dx, dy)| {
            sim.place_deposit((hearth.0 + dx, hearth.1 + dy), "core:good/clay", 8.0, true)
                .ok()
        })
        .expect("dry land a long walk off");
    sim.people_mut_for_tests()
        .deposits_known
        .push(civ_agents::population::DepositKnown {
            settlement,
            deposit,
            finder: members[0],
            at: now,
        });
    let (resource, rate) = {
        let land = &sim.rules().land;
        let goods = &sim.rules().catalog.goods;
        land.resources
            .iter()
            .enumerate()
            .find(|(_, r)| goods[r.good].kcal_per_kg > 0.0 && r.max_rate_per_hour > 0.0)
            .map(|(i, r)| (i, r.max_rate_per_hour * 10.0))
            .expect("a wild food")
    };
    let today = sim.now().day_index();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == buyer {
            x.known.push(civ_agents::person::KnownPatch {
                resource: resource as u16,
                patch: 0,
                rate: rate as f32,
                hours: 1000.0,
                seen_day: today,
            });
        }
    }
    let way = sim
        .with_ctx_for_tests(|pop, ctx| pop.material_way_for_tests(ctx, buyer, brick))
        .expect("a way to bricks");
    let keep = sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, buyer));
    eprintln!(
        "the buyer makes bricks by {way} and keeps {:.0} kg of ash",
        keep[ash]
    );
    assert_eq!(way, "core:recipe/fire_bricks_with_ash");
    assert!(keep[ash] > 1000.0, "short of ash");
    // Weighed again, the round's fee falls below zero: what the keeper would sort out of a load
    // is worth more than its hours.
    let round = review(&mut sim).expect("still posted");
    eprintln!(
        "a buyer: {:.0} kcal a call, {} homes",
        round.fee_kcal,
        round.homes.len()
    );
    assert!(round.fee_kcal < 0.0, "{}", round.fee_kcal);
    // On its run the keeper sorts the load before tipping, keeps the ash and pays each home on
    // `Refuse`: what it keeps comes out of what went to the tip, kilogram for kilogram.
    let tip_before = sim.people().polities[0].tips[0].clone();
    let keeper_ash = sim.people().household(keeper).expect("there").stores[ash];
    let paid_before = food_kcal(&sim, Channel::Refuse);
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    // Food each home has received from others, kcal.
    let received = |sim: &Sim, h: PermanentId| {
        let goods = &sim.rules().catalog.goods;
        let x = sim.people().household(h).expect("there");
        (0..goods.len())
            .map(|g| x.flows.get(civ_agents::person::Flow::Received, g) * goods[g].kcal_per_kg)
            .sum::<f64>()
    };
    let homes_food: Vec<f64> = round.homes.iter().map(|&h| received(&sim, h)).collect();
    let ran = sim.with_ctx_for_tests(|pop, ctx| pop.refuse_run_for_tests(ctx, round.id));
    assert!(ran, "the round ran");
    let run = sim.people().polities[0]
        .rounds
        .iter()
        .find(|r| r.operator == keeper)
        .and_then(|r| r.runs.last().cloned())
        .expect("its run recorded");
    eprintln!(
        "kept {:?} of {} calls in {:.2} h",
        run.kept,
        run.calls.len(),
        run.hours
    );
    let kept = run
        .kept
        .iter()
        .find(|k| usize::from(k.0) == ash)
        .map_or(0.0, |k| k.1);
    let loaded: f64 = run.calls.iter().map(|c| c.ash_kg).sum();
    assert!(kept > 0.0 && kept <= loaded + 1e-9, "{kept} of {loaded}");
    let t = &sim.people().polities[0].tips[0];
    assert!((t.ash_kg - (tip_before.ash_kg + loaded - kept)).abs() < 1e-9);
    let x = sim.people().household(keeper).expect("there");
    assert!((x.stores[ash] - keeper_ash - kept).abs() < 1e-9);
    let paid = food_kcal(&sim, Channel::Refuse) - paid_before;
    let owed = -f64::from(round.fee_kcal) * run.calls.len() as f64;
    eprintln!("the keeper paid {paid:.0} kcal of {owed:.0} owed");
    assert!(paid > 0.0 && paid <= owed + 1e-3 * owed);
    // Each home called at received food: it was paid, not paying.
    for c in &run.calls {
        let i = round
            .homes
            .iter()
            .position(|&h| h == c.household)
            .expect("on it");
        assert!(
            received(&sim, c.household) > homes_food[i],
            "{} was paid",
            c.household
        );
    }
    let gaps = civ_agents::population::unaccounted(
        sim.rules().catalog.goods.len(),
        (&held_before, &flows_before),
        (&sim.people().goods_held(), &sim.people().flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    // A world whose round's fee is below zero and whose run kept a load saves and loads exactly.
    saves_and_goes_on_alike(&mut sim, DAY);
}
