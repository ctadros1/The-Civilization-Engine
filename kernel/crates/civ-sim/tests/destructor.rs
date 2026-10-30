//! The refuse destructor (M11f, fourth part; ADR-0038 §5): an engine whose one fuel is refuse.
//! Held beside a dynamo it turns it at refuse's energy, as a gas engine does at coal gas's; what it
//! burns leaves 0.3 of itself as ash, and the disease the refuse held ends with it. Its keeper
//! draws refuse from its polity's open tip, mass and load conserved. Every engine's fuel now leaves
//! its ash. Nobody wants a destructor without its technique, and a world with one saves and loads
//! exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::contagion::{How, Node};
use civ_agents::person::Flow;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind, Tip};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::wire::Cause;
use civ_sim::{NewWorld, Sim, persist};

const CHOLERA: &str = "core:disease/cholera";
const DAY: i64 = 24 * 60;
/// Refuse's heat, MJ a kilogram: 0.15 of coal's 25.8 (research 12-10 §2.2).
const REFUSE_MJ: f64 = 3.87;
/// The destructor's rating and efficiency (content design priors).
const RATED_KW: f64 = 6.0;
const EFFICIENCY: f64 = 0.05;

fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content")
    })
}

fn create(seed: u64, size_cells: u32, band_size: u32) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Destructor".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells,
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

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

fn cholera(sim: &Sim) -> u16 {
    let i = sim
        .rules()
        .catalog
        .disease_index(CHOLERA)
        .expect("cholera is content");
    u16::try_from(i).expect("few diseases")
}

fn held(sim: &Sim, household: PermanentId, g: usize) -> f64 {
    sim.people()
        .household(household)
        .map_or(0.0, |h| h.stores.get(g).copied().unwrap_or(0.0))
}

fn set(sim: &mut Sim, household: PermanentId, g: usize, kg: f64) {
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == household {
            x.stores[g] = kg;
        }
    }
}

fn teach(sim: &mut Sim, household: PermanentId, id: &str) {
    let t = sim
        .rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect("a technique");
    let learn_h = sim.rules().catalog.techniques[t].learn_h;
    let members = sim
        .people()
        .household(household)
        .expect("household")
        .members
        .clone();
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.learn(t, learn_h, members[0], learn_h, now);
        }
    }
}

/// A world of seed 3 after ten days, with the first home finished: the world, its household and
/// the home, as the wires target's gas engine is set up.
fn a_station_world() -> (Sim, PermanentId, PermanentId) {
    let mut sim = create(3, 768, 30);
    sim.advance_minutes(10 * DAY).expect("lives");
    let (station, home) = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| sim.rules().catalog.is_dwelling(&b.spec.program))
            .expect("a home");
        (b.household, b.id)
    };
    for b in &mut sim.land_mut_for_tests().buildings {
        if b.id == home {
            b.stage = civ_grammar::Stage::ALL.len() as _;
        }
    }
    assert!(
        sim.land().installations.iter().all(|s| s.owner != station),
        "no water of its own"
    );
    (sim, station, home)
}

/// Building `id`'s current in words as the observer reads them.
fn wire_words_seen(sim: &Sim, id: PermanentId) -> String {
    let bytes = civ_sim::frames::buildings::buildings_response(sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    response
        .body_as_buildings()
        .expect("buildings")
        .buildings()
        .expect("a list")
        .iter()
        .find(|b| b.id() == id.get())
        .expect("the building")
        .wire()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn a_destructor_fed_refuse_turns_a_dynamo_at_refuse_s_energy_and_burns_its_load_to_ash() {
    let (mut sim, station, home) = a_station_world();
    let (dynamo, destructor, refuse, ash) = (
        good(&sim, "core:good/dynamo"),
        good(&sim, "core:good/destructor"),
        good(&sim, "core:good/refuse"),
        good(&sim, "core:good/ash"),
    );
    set(&mut sim, station, dynamo, 1.0);
    set(&mut sim, station, destructor, 1.0);
    set(&mut sim, station, refuse, 0.0);
    // Without refuse it turns nothing, so nothing is hooked up.
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    assert!(sim.land().wires.hookups.is_empty());
    // With plenty, its full 6 kW of shaft at the dynamo's 80 %.
    set(&mut sim, station, refuse, 1000.0);
    let current = sim.with_ctx_for_tests(|pop, ctx| pop.electric_kw(ctx, station));
    assert!((current - RATED_KW * 0.8).abs() < 1e-9, "{current}");
    // With 100 kg, what 100 kg gives over an evening's 4 hours at 5 % of 3.87 MJ.
    set(&mut sim, station, refuse, 100.0);
    let shaft = 100.0 * REFUSE_MJ * EFFICIENCY / 3.6 / 4.0;
    let current = sim.with_ctx_for_tests(|pop, ctx| pop.electric_kw(ctx, station));
    assert!((current - shaft * 0.8).abs() < 1e-9, "{current}");
    // Its refuse holds a disease, drawn with it from a tip.
    let d = cholera(&sim);
    sim.people_mut_for_tests()
        .contagion
        .add(Node::Refuse(station), d, 1000.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    assert_eq!(sim.land().wires.hookups.len(), 1, "its home is hooked up");
    let (ash_before, made_before) = (
        held(&sim, station, ash),
        sim.people()
            .household(station)
            .map_or(0.0, |h| h.flows.get(Flow::Made, ash)),
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    assert_eq!(sim.land().wires.hookups[0].cause, Cause::Served);
    // Its home's 0.2 kW took 0.25 kW of shaft for 4 hours: 1 kWh, 3.6 / (3.87 x 5 %) kg.
    let burned = 3.6 / (REFUSE_MJ * EFFICIENCY);
    let left = held(&sim, station, refuse);
    assert!((left - (100.0 - burned)).abs() < 1e-9, "{left}");
    // It leaves 0.3 of what it burned as ash, counted as made.
    let ash_now = held(&sim, station, ash) - ash_before;
    assert!((ash_now - 0.3 * burned).abs() < 1e-9, "{ash_now}");
    let made = sim
        .people()
        .household(station)
        .map_or(0.0, |h| h.flows.get(Flow::Made, ash))
        - made_before;
    assert!((made - ash_now).abs() < 1e-9);
    // And the disease of what it burned is gone, the rest held with the rest of its refuse.
    let c = &sim.people().contagion;
    let load = c.load(Node::Refuse(station), d);
    assert!(
        (load - 1000.0 * (1.0 - burned / 100.0)).abs() < 1e-6,
        "{load}"
    );
    let total: f64 = c
        .loads
        .iter()
        .filter(|((_, x), _)| *x == d)
        .map(|(_, l)| l)
        .sum();
    assert!((total - load).abs() < 1e-9, "burned, not moved: {total}");
    // The station's building says so.
    let words = wire_words_seen(&sim, home);
    assert!(
        words.contains(
            "Station: a dynamo beside its refuse destructor, 6 kW, burning up to 447 kg of \
             refuse an evening"
        ),
        "{words}"
    );
}

#[test]
fn a_gas_engine_on_coal_gas_leaves_no_ash() {
    // As a destructor does at refuse's energy, a gas engine turns a dynamo at coal gas's; coal
    // gas's ash share is none, so it leaves none.
    let (mut sim, station, home) = a_station_world();
    let (dynamo, engine, gas, ash) = (
        good(&sim, "core:good/dynamo"),
        good(&sim, "core:good/gas_engine"),
        good(&sim, "core:good/coal_gas"),
        good(&sim, "core:good/ash"),
    );
    set(&mut sim, station, dynamo, 1.0);
    set(&mut sim, station, engine, 1.0);
    set(&mut sim, station, gas, 10.0);
    let current = sim.with_ctx_for_tests(|pop, ctx| pop.electric_kw(ctx, station));
    assert!((current - 2.4).abs() < 1e-9, "{current}");
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    let ash_before = held(&sim, station, ash);
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    let left = held(&sim, station, gas);
    assert!((left - 9.25).abs() < 1e-9, "{left}");
    assert_eq!(held(&sim, station, ash), ash_before);
    let words = wire_words_seen(&sim, home);
    assert!(
        words.contains(
            "Station: a dynamo beside its gas engine, 3 kW, burning up to 9 kg of coal gas an \
             evening"
        ),
        "{words}"
    );
}

#[test]
fn an_atmospheric_engine_at_a_pit_leaves_its_coal_s_ash() {
    let mut sim = create(3, 512, 30);
    let (coal, engine, ash) = (
        good(&sim, "core:good/coal"),
        good(&sim, "core:good/atmospheric_engine"),
        good(&sim, "core:good/ash"),
    );
    let (work, owner) = a_coal_pit(&mut sim);
    let now = sim.now();
    sim.land_mut_for_tests().set_pit_water(work, 30.0, now);
    for g in [
        good(&sim, "core:good/charcoal"),
        good(&sim, "core:good/coke"),
    ] {
        set(&mut sim, owner, g, 0.0);
    }
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == owner {
            h.stores[engine] += 1.0;
            h.stores[coal] += 500.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.set_engine_for_tests(ctx, owner, work, engine))
        .expect("set");
    let used = |sim: &Sim| {
        sim.people()
            .household(owner)
            .map_or(0.0, |h| h.flows.get(Flow::Used, coal))
    };
    let (used_before, ash_before) = (used(&sim), held(&sim, owner, ash));
    sim.with_ctx_for_tests(|pop, ctx| pop.engines_day_for_tests(ctx));
    let burned = used(&sim) - used_before;
    assert!(burned > 0.0, "it pumped");
    let ash_now = held(&sim, owner, ash) - ash_before;
    assert!(
        (ash_now - 0.11 * burned).abs() < 1e-9,
        "{ash_now} kg of ash from {burned} kg of coal"
    );
}

/// A coal pit dug by the first household on a seam beside the hearth, as the coal target's: the
/// pit and its household.
fn a_coal_pit(sim: &mut Sim) -> (PermanentId, PermanentId) {
    let (first, members) = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .min_by_key(|h| h.id)
        .map(|h| (h.id, h.members.clone()))
        .expect("a household");
    teach(sim, first, "core:technique/coal_burning");
    let (settlement, hearth) = {
        let s = sim.land().settlements.first().expect("a settlement");
        (s.id, s.hearth_m)
    };
    let seam = sim
        .place_deposit((hearth.0 + 40.0, hearth.1), "core:good/coal", 12.0, true)
        .expect("laid down");
    let at = sim.now();
    sim.people_mut_for_tests()
        .deposits_known
        .push(civ_agents::population::DepositKnown {
            settlement,
            deposit: seam,
            finder: members[0],
            at,
        });
    let pit = |sim: &Sim| {
        sim.land()
            .earthworks
            .iter()
            .find(|w| w.kind == civ_land::earth::EarthKind::Pit && w.deposit == Some(seam))
            .map(|w| (w.id, w.household))
    };
    for _ in 0..60 * 24 {
        if pit(sim).is_some() {
            break;
        }
        sim.advance_minutes(60).expect("advances");
    }
    pit(sim).expect("a pit on the seam")
}

/// A law of the first template of `kind` in force in the first polity, every adult of its
/// settlement knowing it, as the rounds target's: its id.
fn a_law(sim: &mut Sim, kind: PolicyKind) -> PermanentId {
    let policy = sim
        .rules()
        .catalog
        .policies
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

/// A world of seed 5 two days on with the law carrying refuse to a tip passed in the first polity,
/// its tip holding 1,000 kg of refuse, 50 of ash and 500 doses of cholera: the world, a household
/// of its settlement and the tip.
fn a_village_with_a_tip() -> (Sim, PermanentId, Tip) {
    let mut sim = create(5, 512, 0);
    sim.advance_minutes(2 * DAY).expect("advances");
    let law = a_law(&mut sim, PolicyKind::RefuseToTip);
    sim.with_ctx_for_tests(|pop, ctx| pop.open_tip_for_tests(ctx, 0, law));
    let d = cholera(&sim);
    let pop = sim.people_mut_for_tests();
    let tip = pop.polities[0]
        .tips
        .iter_mut()
        .find(|t| t.law == law)
        .expect("a place qualifies in this valley, or this test shows nothing");
    tip.kg = 1000.0;
    tip.ash_kg = 50.0;
    let tip = tip.clone();
    pop.contagion.add(Node::Tip(tip.id), d, 500.0);
    let settlement = pop.polities[0].settlement;
    let keeper = pop
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .min()
        .expect("a household");
    (sim, keeper, tip)
}

fn the_tip(sim: &Sim, id: PermanentId) -> Tip {
    sim.people().polities[0]
        .tips
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .expect("the tip")
}

#[test]
fn refuse_drawn_from_a_tip_conserves_its_mass_and_its_load() {
    let (mut sim, keeper, tip) = a_village_with_a_tip();
    let (destructor, refuse) = (
        good(&sim, "core:good/destructor"),
        good(&sim, "core:good/refuse"),
    );
    let d = cholera(&sim);
    // Without a destructor, nobody draws.
    sim.with_ctx_for_tests(|pop, ctx| pop.refuse_draws_for_tests(ctx));
    assert_eq!(the_tip(&sim, tip.id).kg, 1000.0);
    // Its keeper draws what its destructor burns at its rating over an evening.
    set(&mut sim, keeper, destructor, 1.0);
    set(&mut sim, keeper, refuse, 0.0);
    let evening = 3.6 * RATED_KW * 4.0 / (REFUSE_MJ * EFFICIENCY);
    sim.with_ctx_for_tests(|pop, ctx| pop.refuse_draws_for_tests(ctx));
    let (got, t) = (held(&sim, keeper, refuse), the_tip(&sim, tip.id));
    assert!((got - evening).abs() < 1e-9, "{got}");
    assert!((t.kg + got - 1000.0).abs() < 1e-9, "mass conserved");
    assert_eq!(t.ash_kg, 50.0, "the ash stays at the tip");
    let c = &sim.people().contagion;
    let (at_tip, drawn) = (
        c.load(Node::Tip(tip.id), d),
        c.load(Node::Refuse(keeper), d),
    );
    assert!((drawn - 500.0 * evening / 1000.0).abs() < 1e-9, "{drawn}");
    assert!((at_tip + drawn - 500.0).abs() < 1e-9, "load conserved");
    assert!(c.moves.iter().any(|m| m.how == How::Carried
        && m.from == Node::Tip(tip.id)
        && m.to == Node::Refuse(keeper)
        && (m.amount - drawn).abs() < 1e-9));
    // Holding an evening's, it draws no more.
    sim.with_ctx_for_tests(|pop, ctx| pop.refuse_draws_for_tests(ctx));
    assert_eq!(held(&sim, keeper, refuse), got);
    // Emptied, it draws another evening's; and then the rest of the tip, all its load with it.
    set(&mut sim, keeper, refuse, 0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.refuse_draws_for_tests(ctx));
    assert!((held(&sim, keeper, refuse) - evening).abs() < 1e-9);
    set(&mut sim, keeper, refuse, 0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.refuse_draws_for_tests(ctx));
    let rest = held(&sim, keeper, refuse);
    assert!((rest - (1000.0 - 2.0 * evening)).abs() < 1e-9, "{rest}");
    assert_eq!(the_tip(&sim, tip.id).kg, 0.0);
    let c = &sim.people().contagion;
    assert_eq!(c.load(Node::Tip(tip.id), d), 0.0);
    assert!((c.load(Node::Refuse(keeper), d) - 500.0).abs() < 1e-9);
    // The polity page tells the destructor and the tip it draws from.
    let polity = sim.people().polities[0].clone();
    let words = civ_sim::frames::government::services_words(&sim, &polity).join(". ");
    assert!(
        words.contains("'s refuse destructor, 6 kW, burns refuse drawn from the tip"),
        "{words}"
    );
}

#[test]
fn nobody_wants_a_destructor_without_its_technique_and_a_world_with_one_saves_and_loads() {
    let (mut sim, keeper, tip) = a_village_with_a_tip();
    let (destructor, dynamo, refuse) = (
        good(&sim, "core:good/destructor"),
        good(&sim, "core:good/dynamo"),
        good(&sim, "core:good/refuse"),
    );
    let keep = |sim: &mut Sim, g: usize| {
        sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, keeper)[g])
    };
    // Refuse at the tip, but nobody knows how: nobody wants one, and none is made.
    assert_eq!(keep(&mut sim, destructor), 0.0);
    sim.advance_minutes(DAY).expect("lives");
    assert!(
        sim.people().households.iter().all(|(_, x)| x
            .stores
            .get(destructor)
            .copied()
            .unwrap_or(0.0)
            < 1e-9),
        "nobody built one"
    );
    // Knowing the destructor alone is not enough: nothing it could turn.
    teach(&mut sim, keeper, "core:technique/refuse_destructor");
    assert_eq!(keep(&mut sim, destructor), 0.0);
    // Knowing the dynamo too, with refuse at the tip, it wants one.
    teach(&mut sim, keeper, "core:technique/dynamo");
    assert!(
        sim.land().installations.iter().all(|s| s.owner != keeper),
        "no water of its own"
    );
    assert!(keep(&mut sim, destructor) >= 1.0);
    // With the tip empty, it does not.
    let tip_kg = |sim: &mut Sim, kg: f64| {
        for t in &mut sim.people_mut_for_tests().polities[0].tips {
            if t.id == tip.id {
                t.kg = kg;
            }
        }
    };
    tip_kg(&mut sim, 0.0);
    assert_eq!(keep(&mut sim, destructor), 0.0);
    tip_kg(&mut sim, 1000.0);
    // A world with a destructor, refuse drawn and its load saves and loads exactly, and goes on
    // alike.
    set(&mut sim, keeper, destructor, 1.0);
    set(&mut sim, keeper, dynamo, 1.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.refuse_draws_for_tests(ctx));
    assert!(held(&sim, keeper, refuse) > 0.0);
    assert!(
        sim.people()
            .contagion
            .load(Node::Refuse(keeper), cholera(&sim))
            > 0.0
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "destructor").expect("saves");
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
