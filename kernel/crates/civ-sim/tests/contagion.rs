//! What people shed, moved through the ground and water (M6a slice AZ, step two; ADR-0021 §4;
//! research 12-02 §5.2-§5.4, 03-02 §1.6): a wet day washes a share of a heap to the well below
//! it, nothing lost or made on the way; those who draw there
//! carry its load home in their water, and nobody else has any of it; what soaks in arrives after
//! the ground's travel time, sooner and less decayed through faster ground; what a household's
//! water holds is drunk as doses, and the record names the well; and loads, stores, loads on
//! their way and moves save and load exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::contagion::{How, Node};
use civ_agents::population::{WellWeighed, cell_of};
use civ_agents::sickness::Acquired;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::wells::WellState;
use civ_sim::{NewWorld, Sim, persist};

const PRESET: &str = "core:worldgen/river_valley";
const CHOLERA: &str = "core:disease/cholera";
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

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Wellfield".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// Lives on to `minute` past the next midnight.
fn to_morning(sim: &mut Sim, minute: i64) {
    let rest = DAY - sim.now().minutes().rem_euclid(DAY) + minute;
    sim.advance_minutes(rest).expect("advances");
}

/// Households with people, by id.
fn households(sim: &Sim) -> Vec<PermanentId> {
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    out.sort_unstable();
    out
}

/// Household `h`'s home and members.
fn household(sim: &Sim, h: PermanentId) -> ((f32, f32), Vec<PermanentId>) {
    let x = sim
        .people()
        .households
        .iter()
        .find(|(_, x)| x.id == h)
        .expect("the household")
        .1;
    (x.home, x.members.clone())
}

/// The index of cholera in the catalog.
fn cholera(sim: &Sim) -> u16 {
    let i = sim
        .rules()
        .catalog
        .disease_index(CHOLERA)
        .expect("cholera is content");
    u16::try_from(i).expect("few diseases")
}

/// Lives on until homes stand, then has households weigh a well, by id, until one begins one:
/// that household and its well. At eight in the morning.
fn a_well_begun(sim: &mut Sim) -> (PermanentId, PermanentId) {
    for _ in 0..12 {
        to_morning(sim, 8 * 60);
        if let Some(w) = sim
            .land()
            .wells
            .list
            .iter()
            .find(|w| w.state == WellState::Digging)
        {
            return (w.household, w.id);
        }
        for h in households(sim) {
            let weighed = sim.with_ctx_for_tests(|pop, ctx| pop.review_well_for_tests(ctx, h));
            if let WellWeighed::Begun { .. } = weighed {
                let w = sim.land().wells.list.last().expect("begun");
                return (h, w.id);
            }
        }
        sim.advance_minutes(10 * DAY).expect("lives");
    }
    panic!("no household began a well");
}

/// A village of world 6 with one household's well dug and open, its water table standing two
/// metres above its floor: the household and its well, at about eight in the morning.
fn an_open_well() -> (Sim, PermanentId, PermanentId) {
    let mut sim = village(6);
    let (h, well) = a_well_begun(&mut sim);
    let w = *sim.land().wells.get(well).expect("the well");
    let p = sim.land().patches.of_cell(w.cell as usize, sim.map().width);
    let floor = f64::from(w.ground_m) - f64::from(w.target_m);
    sim.land_mut_for_tests().water.heads[p] = floor + 2.0;
    let def = &sim.rules().catalog.wells[w.system];
    let h_per_m = def.h_per_m(sim.rules().people.digging.h_per_m3);
    let hours = f64::from(w.target_m - w.depth_m) * h_per_m + 0.5;
    sim.with_ctx_for_tests(|pop, ctx| pop.well_work_for_tests(ctx, well, hours));
    assert!(sim.land().wells.get(well).expect("the well").is_open());
    (sim, h, well)
}

/// Sets household `h`'s water to `litres`, as of now.
fn set_water(sim: &mut Sim, h: PermanentId, litres: f64) {
    let now = sim.now();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            x.water_l = litres;
            x.water_at = now;
        }
    }
}

/// The household whose home, and the heap beside it, lies nearest well `well` within reach of
/// it and no lower: its distance and how far above the well's mouth it lies.
fn a_heap_above(sim: &Sim, well: PermanentId) -> (PermanentId, f64, f64) {
    let params = &sim.rules().land.contamination;
    let reach = params.runoff_radius_m.min(params.link_radius_m);
    let w = sim.land().wells.get(well).expect("the well");
    let (wx, wy) = w.rect.centre_m();
    households(sim)
        .into_iter()
        .filter_map(|x| {
            let (home, _) = household(sim, x);
            let cell = cell_of(sim.map(), home);
            let dist = (f64::from(wx) - f64::from(home.0))
                .hypot(f64::from(wy) - f64::from(home.1))
                .max(1.0);
            let drop = f64::from(sim.map().elevation[cell]) - f64::from(w.ground_m);
            (dist <= reach && drop >= 0.0).then_some((x, dist, drop))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("a heap within reach above the well")
}

#[test]
fn a_wet_day_washes_a_heap_into_the_well_below_it_and_its_drawers_carry_it_home() {
    let (mut sim, h, well) = an_open_well();
    let (above, _, _) = a_heap_above(&sim, well);
    let d = cholera(&sim);
    let decay = sim.rules().catalog.diseases[usize::from(d)].decay_per_day;
    let params = sim.rules().land.contamination.clone();
    sim.people_mut_for_tests()
        .contagion
        .add(Node::Midden(above), d, 1000.0);
    sim.land_mut_for_tests().water.last_surplus_mm = 20.0;
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let c = &sim.people().contagion;
    // It decays first; then a fifth washes off (0.01 a mm of 20 mm), split among the wellheads
    // and banks within reach and no higher, all of it delivered; a share of the rest soaks in.
    let kept = 1000.0 * (1.0 - decay);
    let washed = kept * (params.wash_per_mm * 20.0);
    let runoff: Vec<_> = c
        .moves
        .iter()
        .filter(|m| m.how == How::Runoff && m.day == day)
        .collect();
    assert!(runoff.iter().all(|m| m.from == Node::Midden(above)));
    let total: f64 = runoff.iter().map(|m| m.amount).sum();
    assert!((total - washed).abs() < 1e-9, "{total} washed of {washed}");
    let to_well = runoff
        .iter()
        .find(|m| m.to == Node::Well(well))
        .expect("the well below the heap took some");
    assert_eq!(c.load(Node::Well(well), d), to_well.amount);
    // The observer is told what fouls its water, in doses and how much water holds one (wire
    // 1.62); a clean well is told nothing.
    let words = civ_sim::frames::sickness::fouled_words(&sim, well, 1000.0);
    assert!(
        words.starts_with("Its water holds cholera: about ") && words.contains(" L"),
        "{words}"
    );
    assert_eq!(
        civ_sim::frames::sickness::fouled_words(
            &sim,
            PermanentId::from_raw(u64::MAX).expect("an id"),
            1000.0
        ),
        ""
    );
    let c = &sim.people().contagion;
    let seeped = (kept - washed) * params.seep_per_day;
    let heap = c.load(Node::Midden(above), d);
    assert!((heap - (kept - washed - seeped)).abs() < 1e-9, "{heap}");
    // What washed into a river passes every reach below it today.
    for m in runoff.iter().filter(|m| matches!(m.to, Node::Reach(_))) {
        let Node::Reach(mut r) = m.to else {
            unreachable!()
        };
        while let Some(below) = sim.map().reaches[r as usize].downstream {
            assert!(c.load(Node::Reach(below), d) >= m.amount - 1e-12);
            r = below;
        }
    }
    // With no water at home, the well's household fetches from it through the morning: each
    // load carries its share of the well's load home, and nothing is lost or made within the day.
    let in_well = c.load(Node::Well(well), d);
    set_water(&mut sim, h, 0.0);
    sim.advance_minutes(4 * 60).expect("lives");
    let c = &sim.people().contagion;
    let mut drawn = std::collections::BTreeMap::new();
    for m in &c.moves {
        if m.how == How::Drawn && m.from == Node::Well(well) {
            let Node::Store(x) = m.to else {
                panic!("drawn into {:?}", m.to)
            };
            *drawn.entry(x).or_insert(0.0) += m.amount;
        }
    }
    assert!(
        drawn.get(&h).is_some_and(|&a| a > 0.0),
        "its household carried none home"
    );
    let total: f64 = drawn.values().sum();
    assert!((c.load(Node::Well(well), d) + total - in_well).abs() < 1e-9);
    for (x, amount) in &drawn {
        let stored = c.stores[&(*x, d)][&Node::Well(well)];
        assert!((stored - amount).abs() < 1e-9, "{x}: {stored} of {amount}");
    }
    // Nobody holds any of it who did not draw it.
    for ((x, _), s) in &c.stores {
        for source in s.keys() {
            assert!(
                c.moves
                    .iter()
                    .any(|m| m.how == How::Drawn && m.from == *source && m.to == Node::Store(*x)),
                "{x} holds a load from {source:?} it never drew"
            );
        }
    }
}

#[test]
fn what_soaks_in_arrives_after_the_ground_s_travel_time_sooner_through_faster_ground() {
    let (mut sim, _, well) = an_open_well();
    let mut fast = reloaded(&mut sim);
    let d = cholera(&sim);
    let decay = sim.rules().catalog.diseases[usize::from(d)].decay_per_day;
    let params = sim.rules().land.contamination.clone();
    // The travel time through the ground from a heap above the well to it, by 03-02 §1.6, at
    // conductivity `k`.
    let (h, dist, drop) = a_heap_above(&sim, well);
    let (home, _) = household(&sim, h);
    let cell = cell_of(sim.map(), home);
    let i = (drop / dist).max(params.min_gradient);
    let n = 0.2;
    // Ground slow enough that it takes 20 days, and ten times faster.
    let slow_k = dist * n / (i * 20.0);
    let tau = |k: f64| dist * n / (k * i).max(1e-12);
    let mut sent = Vec::new();
    for (sim, k) in [(&mut sim, slow_k), (&mut fast, slow_k * 10.0)] {
        let p = sim.land().patches.of_cell(cell, sim.map().width);
        let unit = usize::from(sim.land().water.aquifer.unit[p]);
        let aq = &mut sim.land_mut_for_tests().water.aquifer;
        aq.unit_k_m_day[unit] = k;
        aq.unit_sy[unit] = n;
        sim.people_mut_for_tests()
            .contagion
            .add(Node::Midden(h), d, 1000.0);
        sim.land_mut_for_tests().water.last_surplus_mm = 0.0;
        sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
        let day = sim.now().day_index();
        let t = *sim
            .people()
            .contagion
            .transit
            .iter()
            .find(|t| t.to == Node::Well(well))
            .expect("some soaks toward the well");
        assert_eq!(t.from, Node::Midden(h));
        assert_eq!(t.arrives, day + (tau(k).ceil() as i64).max(1));
        sent.push((day, t));
    }
    let ((day, slow), (_, quick)) = (sent[0], sent[1]);
    assert!(quick.arrives < slow.arrives);
    // The same share sets out each way; what dies on the way is what the time underground takes.
    let ratio = quick.amount / slow.amount;
    let expect = (-decay * tau(slow_k * 10.0)).exp() / (-decay * tau(slow_k)).exp();
    assert!(
        (ratio - expect).abs() < 1e-9 * expect,
        "{ratio} against {expect}"
    );
    // It arrives on its day, whole; through the slow ground nothing has arrived by then.
    let arrived = |sim: &Sim| {
        sim.people()
            .contagion
            .moves
            .iter()
            .filter(|m| m.how == How::Arrived && m.to == Node::Well(well))
            .map(|m| (m.day, m.amount))
            .collect::<Vec<_>>()
    };
    let days = quick.arrives - day;
    to_morning(&mut fast, 1);
    fast.advance_minutes((days - 1) * DAY).expect("lives");
    assert_eq!(arrived(&fast), vec![(quick.arrives, quick.amount)]);
    to_morning(&mut sim, 1);
    sim.advance_minutes((days - 1) * DAY).expect("lives");
    assert!(
        arrived(&sim).is_empty(),
        "the slow ground's load came early"
    );
}

#[test]
fn a_household_drinks_what_its_water_holds_and_its_record_names_the_well() {
    let (mut sim, h, well) = an_open_well();
    let d = cholera(&sim);
    {
        let rules = sim.rules_mut_for_tests().expect("the rules are not shared");
        let def = &mut rules.catalog.diseases[usize::from(d)];
        def.household_hazard = 0.0;
        def.severe_death_per_day = 0.0;
    }
    // Just before midnight its water holds a heavy load drawn at the well.
    to_morning(&mut sim, 0);
    sim.advance_minutes(DAY - 2).expect("lives");
    set_water(&mut sim, h, 200.0);
    sim.people_mut_for_tests()
        .contagion
        .store_add(h, d, Node::Well(well), 1.0e6);
    sim.advance_minutes(4).expect("lives past midnight");
    let day = sim.now().day_index();
    let (_, members) = household(&sim, h);
    let episodes = sim.people().sickness.episodes();
    assert_eq!(episodes.len(), members.len(), "its people, and nobody else");
    for e in episodes {
        assert!(members.contains(&e.person));
        assert_eq!(e.infected, day);
        assert_eq!(
            e.acquired,
            Acquired::Water {
                source: Node::Well(well)
            }
        );
        // The inspector names the well it was drawn at (wire 1.62).
        let words = civ_sim::frames::sickness::sickness_words(&sim, e.person);
        assert!(
            words[0].contains("; took it in water drawn at ")
                && words[0].contains("'s household's well"),
            "{words:?}"
        );
    }
    // What it used of its water took its share of the load with it.
    let left = sim.people().contagion.store_load(h, d);
    assert!(left < 1.0e6, "{left}");
}

#[test]
fn a_world_where_nobody_sheds_has_no_load_anywhere() {
    let mut sim = village(6);
    to_morning(&mut sim, 0);
    sim.advance_minutes(10 * DAY).expect("lives");
    assert!(sim.people().contagion.is_empty());
    assert!(sim.people().contagion.moves.is_empty());
}

/// A save of `sim` loaded again, checked to be the same section for section.
fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "shed").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    same(sim, &loaded);
    loaded
}

fn same(a: &Sim, b: &Sim) {
    let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

#[test]
fn loads_stores_loads_underway_and_moves_save_load_and_go_on_alike() {
    let (mut sim, h, well) = an_open_well();
    let d = cholera(&sim);
    // Someone of the household sheds it, its heap holds some, and its water some from the well.
    let (_, members) = household(&sim, h);
    sim.plague(members[0], CHOLERA).expect("brought");
    sim.people_mut_for_tests()
        .contagion
        .store_add(h, d, Node::Well(well), 50.0);
    sim.people_mut_for_tests()
        .contagion
        .add(Node::Midden(h), d, 5000.0);
    sim.land_mut_for_tests().water.last_surplus_mm = 15.0;
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    sim.advance_minutes(3 * DAY + 7 * 60).expect("lives");
    let c = &sim.people().contagion;
    assert!(!c.loads.is_empty() && !c.moves.is_empty());
    let mut loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().contagion, sim.people().contagion);
    sim.advance_minutes(4 * DAY).expect("advances");
    loaded.advance_minutes(4 * DAY).expect("advances");
    same(&sim, &loaded);
}
