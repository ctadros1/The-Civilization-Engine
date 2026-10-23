//! The works (M10a slice DT, first part; ADR-0033 §8): a machine names the hands it wants at it
//! together, and a missing hand idles it; a works whose machine wants a crew rings a bell at
//! sunrise and hires for its shift, so its hands are at it together; and a save keeps the bell.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::market::Market;
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
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

fn new_world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Works".to_owned(),
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
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|d| d.id == id)
        .expect(id)
}

fn activity(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .activities
        .iter()
        .position(|a| a.id == id)
        .expect(id)
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
        let g = goods.iter().position(|d| d.id == id).expect(id);
        h.stores[g] = v;
    }
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

/// A works forging bar under a tilt hammer driven by a wheel of its own, its owners holding blooms
/// and charcoal: the owning household, the person of it who works there, and the works.
fn a_hammer_works(sim: &mut Sim, with_hammer: bool) -> (PermanentId, PermanentId, PermanentId) {
    sim.advance_minutes(24 * 60).expect("advances");
    let (iron, hammer, wheel) = (
        good(sim, "core:good/iron"),
        good(sim, "core:good/tilt_hammer"),
        good(sim, "core:good/water_wheel"),
    );
    let (owner, who) = grown(sim)[0];
    set(
        sim,
        owner,
        &[
            ("core:good/iron_bloom", 200.0),
            ("core:good/charcoal", 2000.0),
            ("core:good/water_wheel", 1.0),
            ("core:good/tilt_hammer", if with_hammer { 1.0 } else { 0.0 }),
            ("core:good/grain", 5000.0),
        ],
    );
    let firm = sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, who, iron))
        .expect("a works");
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.put_in_station(ctx, firm, wheel, 1.0)));
    if with_hammer {
        assert!(sim.with_ctx_for_tests(|pop, ctx| pop.put_in_station(ctx, firm, hammer, 1.0)));
    }
    // Its wheel: at reach 0, falling two metres, its race carrying 0.5 m³/s of a full river.
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
    (owner, who, firm)
}

#[test]
fn a_two_hand_machine_idles_without_its_second_hand() {
    let mut sim = new_world(3);
    let (owner, who, firm) = a_hammer_works(&mut sim, true);
    let iron = good(&sim, "core:good/iron");
    let forge = activity(&sim, "core:activity/forge_bar_by_hammer");
    let hand = grown(&sim)
        .into_iter()
        .find(|&(h, _)| h != owner)
        .map(|(_, p)| p)
        .expect("a hand from another household");
    let start = sim.now();
    let forged = |sim: &mut Sim, by: PermanentId| {
        let before = sim.people().firm(firm).expect("works").stores[iron];
        sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, by, forge, 120, firm));
        sim.people().firm(firm).expect("works").stores[iron] - before
    };
    // Alone at it, the hammer idles.
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(who, forge, firm, start));
    assert_eq!(
        forged(&mut sim, who),
        0.0,
        "one hand idles a two-hand hammer"
    );
    // With a second hand there since the session began, it forges.
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hand, forge, firm, start));
    let two = forged(&mut sim, who);
    assert!(two > 0.0, "two hands work it: {two}");
    // A hand who came half an hour later does not make the earlier session's crew, but the
    // first hand, still there, makes the later one's.
    let later = SimTime::from_minutes(start.minutes() + 30);
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hand, forge, firm, later));
    assert_eq!(forged(&mut sim, who), 0.0);
    assert!(forged(&mut sim, hand) > 0.0);
}

/// The settlement of household `owner` wants `n` kilograms of iron it could not buy.
fn ask_for_iron(sim: &mut Sim, owner: PermanentId, n: f64) {
    let day = sim.now().day_index();
    let goods = sim.rules().catalog.goods.clone();
    let iron = good(sim, "core:good/iron");
    let settlement = sim
        .people()
        .household(owner)
        .and_then(|h| h.settlement)
        .expect("a settlement");
    let pop = sim.people_mut_for_tests();
    if !pop.markets.iter().any(|m| m.settlement == settlement) {
        pop.markets.push(Market::new(settlement, goods.len(), day));
    }
    let m = pop
        .markets
        .iter_mut()
        .find(|m| m.settlement == settlement)
        .expect("a market");
    m.record_unmet(iron, n, 40.0 * n);
}

/// Everyone of household `owner` knows the ways to iron: charcoal, the bloomery, forging and the
/// water hammer.
fn teach_iron(sim: &mut Sim, owner: PermanentId) {
    let ids = [
        "core:technique/charcoal_burning",
        "core:technique/bloomery",
        "core:technique/forging",
        "core:technique/water_hammer",
    ];
    let techniques: Vec<(usize, f64)> = ids
        .iter()
        .map(|id| {
            let t = sim
                .rules()
                .catalog
                .techniques
                .iter()
                .position(|t| t.id == *id)
                .expect(id);
            (t, sim.rules().catalog.techniques[t].learn_h)
        })
        .collect();
    let now = sim.now();
    let members = sim
        .people()
        .household(owner)
        .expect("owner")
        .members
        .clone();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            for &(t, learn_h) in &techniques {
                p.learn(t, learn_h, members[0], learn_h, now);
            }
        }
    }
}

#[test]
fn a_works_whose_machine_wants_a_crew_hires_for_a_shift_from_its_bell() {
    // With the hammer: the hammer's way, a bell at sunrise and a shift by daylight.
    let mut sim = new_world(3);
    let (owner, _, firm) = a_hammer_works(&mut sim, true);
    teach_iron(&mut sim, owner);
    ask_for_iron(&mut sim, owner, 50.0);
    let iron = good(&sim, "core:good/iron");
    let forge = activity(&sim, "core:activity/forge_bar_by_hammer");
    let wage = sim
        .with_ctx_for_tests(|pop, ctx| pop.plan_wage_for_tests(ctx, firm, iron))
        .expect("the works hires");
    assert_eq!(usize::from(wage.activity), forge);
    assert!(wage.shift_min > 0, "a shift");
    assert!(wage.shift_min <= 600, "no longer than ten hours");
    let doy = civ_land::day_of_year(sim.now().day_index());
    let sun = civ_agents::needs::daylight(sim.rules().people.latitude_deg, doy);
    assert_eq!(i64::from(wage.bell_min), sun.0, "the bell at sunrise");
    assert!(
        i64::from(wage.bell_min) + i64::from(wage.shift_min) <= sun.1,
        "by daylight"
    );
    // Without the hammer: the hand's way, and no bell.
    let mut sim = new_world(3);
    let (owner, _, firm) = a_hammer_works(&mut sim, false);
    teach_iron(&mut sim, owner);
    ask_for_iron(&mut sim, owner, 50.0);
    let wage = sim
        .with_ctx_for_tests(|pop, ctx| pop.plan_wage_for_tests(ctx, firm, iron))
        .expect("the workshop hires");
    assert_ne!(usize::from(wage.activity), forge);
    assert_eq!((wage.bell_min, wage.shift_min), (0, 0));
}

#[test]
fn a_bell_is_saved_and_loaded() {
    let mut sim = new_world(3);
    let (owner, _, firm) = a_hammer_works(&mut sim, true);
    teach_iron(&mut sim, owner);
    ask_for_iron(&mut sim, owner, 50.0);
    let iron = good(&sim, "core:good/iron");
    let wage = sim
        .with_ctx_for_tests(|pop, ctx| pop.plan_wage_for_tests(ctx, firm, iron))
        .expect("the works hires");
    sim.people_mut_for_tests()
        .firms
        .iter_mut()
        .find(|f| f.id == firm)
        .expect("works")
        .wage = Some(wage);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "bell")
        .expect("saves");
    let loaded = civ_sim::persist::load(&saved.path, content()).expect("loads");
    let bell = |s: &Sim| s.people().firm(firm).and_then(|f| f.wage);
    assert_eq!(bell(&loaded), bell(&sim));
    assert!(bell(&sim).is_some_and(|w| w.shift_min > 0));
    assert_eq!(
        civ_sim::persist::encode_sections(&sim),
        civ_sim::persist::encode_sections(&loaded)
    );
}

#[test]
fn in_a_household_resolved_day_a_crew_is_met_in_aggregate() {
    // ADR-0032 §2 (M10a slice DT): sessions at a machine wanting a crew are held while the day is
    // lived, then it runs once for their summed minutes over its crew if as many hands came.
    let detailed = {
        let mut sim = new_world(3);
        let (owner, who, firm) = a_hammer_works(&mut sim, true);
        let iron = good(&sim, "core:good/iron");
        let forge = activity(&sim, "core:activity/forge_bar_by_hammer");
        let hand = grown(&sim)
            .into_iter()
            .find(|&(h, _)| h != owner)
            .map(|(_, p)| p)
            .expect("a hand");
        let start = sim.now();
        sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(who, forge, firm, start));
        sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hand, forge, firm, start));
        sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, who, forge, 120, firm));
        sim.people().firm(firm).expect("works").stores[iron]
    };
    let resolved = |hands: usize| {
        let mut sim = new_world(3);
        let (owner, who, firm) = a_hammer_works(&mut sim, true);
        let iron = good(&sim, "core:good/iron");
        let forge = activity(&sim, "core:activity/forge_bar_by_hammer");
        let hand = grown(&sim)
            .into_iter()
            .find(|&(h, _)| h != owner)
            .map(|(_, p)| p)
            .expect("a hand");
        sim.with_ctx_for_tests(|pop, ctx| pop.resolving_for_tests(ctx, true));
        sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, who, forge, 120, firm));
        if hands > 1 {
            sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, hand, forge, 120, firm));
        }
        // Nothing is made while the day is lived.
        assert_eq!(sim.people().firm(firm).expect("works").stores[iron], 0.0);
        sim.with_ctx_for_tests(|pop, ctx| pop.resolving_for_tests(ctx, false));
        sim.people().firm(firm).expect("works").stores[iron]
    };
    assert!(detailed > 0.0);
    // One hand came: the hammer stood idle all day.
    assert_eq!(resolved(1), 0.0);
    // Two came for two hours each: two hours of the hammer, as a shift of them together.
    let two = resolved(2);
    assert!(
        (two - detailed).abs() < 1e-9,
        "{two} in aggregate against {detailed} in detail"
    );
}

#[test]
fn the_works_page_shows_its_bell_its_machines_and_its_hands_at_work() {
    // The observer's works page (wire 1.98, M10a slice DT): the bell and shift with its wage, each
    // machine and the crew it wants, and how many are at work for it now.
    use civ_schema::{flatbuffers, wire};
    let mut sim = new_world(3);
    let (owner, who, firm) = a_hammer_works(&mut sim, true);
    teach_iron(&mut sim, owner);
    ask_for_iron(&mut sim, owner, 50.0);
    let (iron, hammer, wheel) = (
        good(&sim, "core:good/iron"),
        good(&sim, "core:good/tilt_hammer"),
        good(&sim, "core:good/water_wheel"),
    );
    let forge = activity(&sim, "core:activity/forge_bar_by_hammer");
    let wage = sim
        .with_ctx_for_tests(|pop, ctx| pop.plan_wage_for_tests(ctx, firm, iron))
        .expect("the works hires");
    sim.people_mut_for_tests()
        .firms
        .iter_mut()
        .find(|f| f.id == firm)
        .expect("works")
        .wage = Some(wage);
    let start = sim.now();
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(who, forge, firm, start));
    let page =
        |sim: &Sim| civ_sim::frames::firms::firm_response(sim, firm.get()).expect("its page");
    let payload = page(&sim);
    let info = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_firm_info()
        .expect("a page");
    let w = info.wage().expect("its wage");
    assert_eq!(
        (w.bell_minute(), w.shift_minutes()),
        (wage.bell_min, wage.shift_min)
    );
    let text = w.text().expect("words");
    let bell = format!("{:02}:{:02}", wage.bell_min / 60, wage.bell_min % 60);
    assert!(
        text.contains(&format!("rings its bell at {bell}")),
        "{text}"
    );
    let machines: Vec<(u16, u8, f32)> = info
        .machines()
        .expect("its machines")
        .iter()
        .map(|m| (m.good(), m.crew(), m.units()))
        .collect();
    assert!(machines.contains(&(hammer as u16, 2, 1.0)), "{machines:?}");
    assert!(machines.contains(&(wheel as u16, 1, 1.0)), "{machines:?}");
    assert_eq!(info.at_work(), 1, "one hand at the hammer");
    // A workshop with no bell says nothing of one, and holds no machine.
    let mut plain = new_world(3);
    let (_, _, shop) = a_hammer_works(&mut plain, false);
    plain
        .people_mut_for_tests()
        .firms
        .iter_mut()
        .find(|f| f.id == shop)
        .expect("workshop")
        .wage = Some(civ_agents::firm::WageOffer {
        bell_min: 0,
        shift_min: 0,
        ..wage
    });
    let payload = civ_sim::frames::firms::firm_response(&plain, shop.get()).expect("its page");
    let info = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_firm_info()
        .expect("a page");
    let w = info.wage().expect("its wage");
    assert_eq!(w.shift_minutes(), 0);
    assert!(!w.text().unwrap_or_default().contains("bell"));
    assert_eq!(info.at_work(), 0);
}
