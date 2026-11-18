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
fn a_covered_wellhead_takes_its_share_of_what_an_open_one_would_and_the_rest_soaks_away() {
    // One wet day's runoff from a heap above the well, with the well open and then covered (M6a
    // slice BA): the covered mouth takes the content's share of what the open one took, every
    // other place below takes the same, and what it turned away is lost to the ground.
    let runoff = |covered: bool| {
        let (mut sim, _, well) = an_open_well();
        let (above, _, _) = a_heap_above(&sim, well);
        let d = cholera(&sim);
        if covered {
            let w = sim.land_mut_for_tests().wells.list.iter_mut();
            for w in w.filter(|w| w.id == well) {
                w.covered = true;
            }
        }
        sim.people_mut_for_tests()
            .contagion
            .add(Node::Midden(above), d, 1000.0);
        sim.land_mut_for_tests().water.last_surplus_mm = 20.0;
        let day = sim.now().day_index();
        sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
        let share = sim.rules().catalog.wells[0].covered_runoff;
        let moves: Vec<(Node, f64)> = sim
            .people()
            .contagion
            .moves
            .iter()
            .filter(|m| m.how == How::Runoff && m.day == day)
            .map(|m| (m.to, m.amount))
            .collect();
        (moves, Node::Well(well), share)
    };
    let (open, well, share) = runoff(false);
    let (covered, _, _) = runoff(true);
    let at = |moves: &[(Node, f64)], n: Node| moves.iter().find(|m| m.0 == n).map_or(0.0, |m| m.1);
    let open_well = at(&open, well);
    assert!(open_well > 0.0);
    assert!((at(&covered, well) - open_well * share).abs() < 1e-9);
    for &(n, a) in open.iter().filter(|m| m.0 != well) {
        assert!((at(&covered, n) - a).abs() < 1e-12, "{n:?}");
    }
    let total = |moves: &[(Node, f64)]| moves.iter().map(|m| m.1).sum::<f64>();
    assert!((total(&open) - total(&covered) - open_well * (1.0 - share)).abs() < 1e-9);
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

#[test]
fn the_incidents_list_every_case_and_an_outbreak_s_page_traces_the_well_its_water_came_from() {
    // Wire 1.64 (M6a slice BB, ADR-0015 §8).
    use civ_agents::contagion::Move;
    use civ_agents::uses::Place;
    use civ_agents::word::{Counts, Suspicion};
    use civ_schema::{flatbuffers, wire};
    use civ_sim::frames::incidents::{self, ILLNESS, TAKING};

    let (mut sim, h, well) = an_open_well();
    let d = cholera(&sim);
    {
        let rules = sim.rules_mut_for_tests().expect("the rules are not shared");
        let def = &mut rules.catalog.diseases[usize::from(d)];
        def.household_hazard = 0.0;
        def.severe_death_per_day = 0.0;
    }
    // The observer brings cholera to an asymptomatic member of the well's household: the outbreak's first case.
    let (_, members) = household(&sim, h);
    let index_person = members
        .iter()
        .copied()
        .find(|&m| {
            let p = sim.people().person(m).expect("person");
            let def = &sim.rules().catalog.diseases[usize::from(d)];
            let mut rng = civ_agents::sickness::sickness_rng(
                sim.meta().seed,
                m,
                sim.now().day_index(),
                d,
                civ_agents::sickness::SickDraw::Course,
            );
            let c = civ_agents::sickness::course(
                def,
                p.age_years(sim.now()),
                sim.now().day_index(),
                &mut rng,
            );
            c.ill_from == c.ill_until
        })
        .unwrap_or(members[0]);
    sim.plague(index_person, CHOLERA).expect("brought");
    // Another household's water, drawn at the well, holds a heavy load of it just before
    // midnight; a wet day had washed some into the well from the first household's heap.
    let other = households(&sim)
        .into_iter()
        .find(|&x| x != h)
        .expect("another household");
    to_morning(&mut sim, 0);
    sim.advance_minutes(DAY - 2).expect("lives");
    set_water(&mut sim, other, 200.0);
    let day = sim.now().day_index();
    {
        let pop = sim.people_mut_for_tests();
        pop.contagion.store_add(other, d, Node::Well(well), 1.0e6);
        pop.contagion.moves.push(Move {
            day,
            disease: d,
            how: How::Runoff,
            from: Node::Midden(h),
            to: Node::Well(well),
            amount: 120.0,
        });
        pop.contagion.moves.push(Move {
            day,
            disease: d,
            how: How::Drawn,
            from: Node::Well(well),
            to: Node::Store(other),
            amount: 1.0e6,
        });
    }
    sim.advance_minutes(4).expect("lives past midnight");
    let episodes = sim.people().sickness.episodes().to_vec();
    let (_, others) = household(&sim, other);
    assert_eq!(episodes.len(), 1 + others.len());
    assert!(episodes.iter().all(|e| e.outbreak == Some(1)));
    // Each case keeps the household it began in (saves 85).
    assert_eq!(episodes[0].household, Some(h));
    assert!(episodes[1..].iter().all(|e| e.household == Some(other)));

    // The list: every case, newest first, all of them the observer's, since it began the
    // outbreak.
    let bytes = incidents::incidents_response(&sim, None, ILLNESS, None, 0);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let list = response.body_as_incidents().expect("incidents");
    let rows = list.rows().expect("rows");
    assert_eq!(rows.len(), episodes.len());
    assert!(list.next().is_none());
    let observers = rows.get(rows.len() - 1);
    assert_eq!((observers.kind(), observers.id()), (ILLNESS, 1));
    let happened = observers.happened().unwrap_or_default();
    assert!(
        happened.starts_with("took cholera on ")
            && happened.contains(", brought by the observer as if they took it elsewhere"),
        "{happened}"
    );
    assert!(
        observers
            .known()
            .is_some_and(|k| k.starts_with("Nobody knows of it")),
        "{:?}",
        observers.known()
    );
    assert_eq!(observers.polity(), Some("No law names it"));
    assert!(observers.observer() > 0);
    assert_eq!(observers.outbreak(), 1);
    let drank = rows.get(0);
    let happened = drank.happened().unwrap_or_default();
    assert!(
        happened.contains(" in water drawn at ") && happened.contains("'s household's well"),
        "{happened}"
    );
    assert!(
        drank
            .polity()
            .is_some_and(|p| p.starts_with("No law names ") && p.ends_with("'s household's well")),
        "{:?}",
        drank.polity()
    );
    let total = list
        .totals()
        .expect("totals")
        .iter()
        .find(|t| t.kind() == ILLNESS)
        .expect("illness counted");
    assert_eq!(
        (total.all(), total.observer(), total.polity()),
        (episodes.len() as u32, episodes.len() as u32, 0)
    );
    let outbreaks = list.outbreaks().expect("outbreaks");
    assert_eq!(outbreaks.len(), 1);
    assert!(
        outbreaks
            .get(0)
            .words()
            .is_some_and(|w| w.starts_with("Cholera at ") && w.ends_with("; it runs")),
        "{:?}",
        outbreaks.get(0).words()
    );
    // A page at a time, each going on from where the last stopped.
    let mut paged = Vec::new();
    let mut before = None;
    loop {
        let bytes = incidents::incidents_response(&sim, None, ILLNESS, before, 1);
        let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
        let page = response.body_as_incidents().expect("incidents");
        paged.extend(page.rows().expect("rows").iter().map(|r| r.id()));
        match page.next() {
            Some(k) => before = Some((k.minute(), k.kind(), k.id())),
            None => break,
        }
    }
    assert_eq!(paged, rows.iter().map(|r| r.id()).collect::<Vec<_>>());
    // Another settlement's list has none of them.
    let bytes = incidents::incidents_response(&sim, PermanentId::from_raw(999_999), 0, None, 0);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    assert_eq!(
        response
            .body_as_incidents()
            .and_then(|l| l.rows())
            .map(|r| r.len()),
        Some(0)
    );

    // A case's page: its steps in order, each reached or not, and its links.
    let bytes = incidents::incident_response(&sim, ILLNESS, 1).expect("a page");
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let info = response.body_as_incident_info().expect("a page");
    let steps = info.steps().expect("steps");
    let labels: Vec<&str> = steps
        .iter()
        .map(|s| s.label().unwrap_or_default())
        .collect();
    assert_eq!(
        labels,
        ["Took it", "Shed it", "Fell ill", "Seen", "Tended", "Ended"]
    );
    assert_eq!((steps.get(0).state(), steps.get(0).by_day()), (0, true));
    assert_eq!(
        (steps.get(5).state(), steps.get(5).words()),
        (1, Some("it runs"))
    );
    let links: Vec<(u8, u8, u64)> = info
        .links()
        .expect("links")
        .iter()
        .map(|l| (l.relation(), l.target(), l.id()))
        .collect();
    assert!(links.contains(&(1, 4, 1)), "part of outbreak 1: {links:?}");
    assert!(
        links.iter().any(|&(r, t, _)| (r, t) == (0, 0)),
        "caused by the observer: {links:?}"
    );
    assert!(incidents::incident_response(&sim, ILLNESS, 999).is_none());
    assert!(incidents::incident_response(&sim, TAKING, 999).is_none());
    assert!(incidents::incident_response(&sim, 7, 1).is_none());

    // The outbreak's page traces its cases to the well, the truth beside what people hold.
    let page = |sim: &Sim| incidents::outbreak_response(sim, 1, 0).expect("a page");
    let bytes = page(&sim);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let info = response.body_as_outbreak_info().expect("an outbreak");
    let row = info.row().expect("its row");
    assert_eq!(row.took(), episodes.len() as u32);
    assert!(row.observer() > 0);
    let cause = info.cause().unwrap_or_default();
    assert!(
        cause.starts_with("Began with ") && cause.contains("brought by the observer"),
        "{cause}"
    );
    assert_eq!(info.cause_link().map(|l| l.target()), Some(0));
    let took: u32 = info.took().expect("a curve").iter().map(u32::from).sum();
    assert_eq!(took, episodes.len() as u32);
    let routes: Vec<&str> = info.routes().expect("routes").iter().collect();
    assert!(routes.contains(&"1 brought by the observer"), "{routes:?}");
    assert!(
        routes
            .iter()
            .any(|r| r.starts_with(&format!("{} in water drawn at ", others.len()))),
        "{routes:?}"
    );
    let sources = info.sources().expect("sources");
    let first = sources.get(0);
    assert!(
        first
            .name()
            .is_some_and(|n| n.ends_with("'s household's well")),
        "{:?}",
        first.name()
    );
    assert_eq!(first.cases(), others.len() as u32);
    let truth: Vec<&str> = first.truth().expect("truth").iter().collect();
    // Of the households, the one that drew it at the well had every case but the first; the rest
    // had only the observer's (the moves the kernel keeps, not the households' faded records).
    let rest = households(&sim).len() - 1;
    let n = others.len();
    assert_eq!(
        truth[0],
        format!(
            "Of 1 household that drew cholera in its water since it began, 1 had a case ({n} \
             case{}); of {rest} household{} that did not, 1 had a case (1 case)",
            if n == 1 { "" } else { "s" },
            if rest == 1 { "" } else { "s" },
        ),
        "{truth:?}"
    );
    assert!(
        truth[1].starts_with("Fouled from ")
            && truth[1].contains("120 doses washed from the heap of "),
        "{truth:?}"
    );
    let knowledge: Vec<&str> = first.knowledge().expect("knowledge").iter().collect();
    assert_eq!(knowledge, ["Nobody suspects it"]);
    let polity: Vec<&str> = first.polity().expect("polity").iter().collect();
    assert_eq!(polity, ["No law names it"]);
    assert_eq!(info.members_total(), episodes.len() as u32);
    assert_eq!(info.members().map(|m| m.len()), Some(episodes.len()));
    assert!(incidents::outbreak_response(&sim, 99, 0).is_none());
    // The dashboard's epidemics row reads the same split (plan §4.7).
    let c = incidents::clustering(&sim, 1).expect("the outbreak");
    assert_eq!((c.cases, c.water_cases), (episodes.len() as u32, n as u32));
    assert!(c.name.starts_with("Cholera at "), "{}", c.name);
    let s = c.source.expect("a source");
    assert!(s.name.ends_with("'s household's well"), "{}", s.name);
    assert_eq!(s.took, n as u32);
    assert_eq!(
        (s.drew.households, s.drew.with_case, s.drew.cases),
        (1, 1, n as u32)
    );
    assert_eq!(
        (s.rest.households, s.rest.with_case, s.rest.cases),
        (rest as u32, 1, 1)
    );
    assert_eq!(s.drew.attack(), Some(1.0));
    assert!(incidents::clustering(&sim, 99).is_none());
    // The chronicle links the observer's bringing it to the case, and the outbreak's beginning to
    // its page, each with where it happened (wire 1.65).
    let bytes = civ_sim::frames::people::chronicle_response(&sim, 0, 1000);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let links: Vec<(u8, u32, bool)> = response
        .body_as_chronicle()
        .and_then(|c| c.entries())
        .expect("entries")
        .iter()
        .filter(|e| e.incident_kind() != 0)
        .map(|e| (e.incident_kind(), e.incident(), e.at().is_some()))
        .collect();
    assert!(links.contains(&(ILLNESS, 1, false)), "{links:?}");
    assert!(
        links
            .iter()
            .any(|&(k, id, _)| k == incidents::OUTBREAK && id == 1),
        "{links:?}"
    );

    // Someone comes to suspect the well: what people hold changes, and only that.
    let cell = sim.land().wells.get(well).expect("the well").cell;
    let holder = others[0];
    {
        let pop = sim.people_mut_for_tests();
        pop.word.suspicions.push(Suspicion {
            holder,
            source: Place::Source(cell),
            since: day,
            counts: Counts::default(),
        });
        pop.word.suspicions.sort_by_key(|s| (s.holder, s.source));
    }
    let bytes = page(&sim);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let info = response.body_as_outbreak_info().expect("an outbreak");
    let first = info.sources().expect("sources").get(0);
    let knowledge: Vec<&str> = first.knowledge().expect("knowledge").iter().collect();
    assert!(
        knowledge[0].starts_with("1 person suspects it, the first since "),
        "{knowledge:?}"
    );
    assert_eq!(first.truth().map(|t| t.len()), Some(truth.len()));

    // A law to close water nobody drank it from, made since it began, lists that water too, with
    // what the polity did, whether or not anyone still suspects it.
    {
        use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind};
        // The river nearest the household's home, where no well is dug and no spring rises.
        let (home, _) = household(&sim, h);
        let (w, cs) = (sim.map().width, sim.map().cell_size_m);
        let elsewhere = (0..sim.map().water.len())
            .filter(|&c| sim.map().water[c] == civ_world::WATER_RIVER)
            .filter(|&c| sim.land().wells.list.iter().all(|x| x.cell as usize != c))
            .filter(|&c| {
                let p = sim.land().patches.of_cell(c, w);
                sim.land().water.aquifer.seep[p].is_none_or(|(s, _)| s as usize != c)
            })
            .min_by(|&a, &b| {
                let d = |c: usize| {
                    let (x, y) = (
                        ((c as u32 % w) as f32 + 0.5) * cs,
                        ((c as u32 / w) as f32 + 0.5) * cs,
                    );
                    (x - home.0).hypot(y - home.1)
                };
                d(a).total_cmp(&d(b))
            })
            .expect("a river") as u32;
        let policy = sim
            .rules()
            .catalog
            .policies
            .iter()
            .position(|d| d.kind == PolicyKind::CloseSource)
            .expect("the core content has the template");
        let id = sim.allocate_id_for_tests();
        let now = sim.now();
        let pop = sim.people_mut_for_tests();
        let p = &mut pop.polities[0];
        p.laws.push(Law {
            id,
            policy: policy as u16,
            kind: PolicyKind::CloseSource,
            levy_share: 0.0,
            holder: None,
            relief_days: 0.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: holder,
            proposed: now,
            issue: IssueKind::SourceSuspected,
            meets_day: now.day_index(),
            decided: Some(now),
            outcome: Some(Decided::Passed),
            eligible: 1,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
            target: None,
        });
        p.closed.push((id, Place::Source(elsewhere)));
    }
    let bytes = page(&sim);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let info = response.body_as_outbreak_info().expect("an outbreak");
    let closed = info
        .sources()
        .expect("sources")
        .iter()
        .find(|s| s.name() == Some("the river"))
        .expect("the water a law would close is listed");
    assert_eq!(closed.cases(), 0);
    let polity: Vec<&str> = closed.polity().expect("polity").iter().collect();
    assert!(
        polity.len() == 1 && polity[0] != "No law names it",
        "{polity:?}"
    );

    // Saved and loaded, the cases keep their households and every page reads the same.
    let loaded = reloaded(&mut sim);
    assert_eq!(
        incidents::incidents_response(&loaded, None, 0, None, 0),
        incidents::incidents_response(&sim, None, 0, None, 0)
    );
    assert_eq!(page(&loaded), page(&sim));
    assert_eq!(
        incidents::incident_response(&loaded, ILLNESS, 2),
        incidents::incident_response(&sim, ILLNESS, 2)
    );
    // While a case runs, the revision moves on with each day lived.
    let rev = incidents::incidents_rev(&sim);
    assert!(rev > 0);
    to_morning(&mut sim, 0);
    assert_ne!(incidents::incidents_rev(&sim), rev);
}

#[test]
fn the_water_lens_reads_each_home_s_nearest_water_and_the_days_its_people_drew_where() {
    // Wire 1.66 (M6a slice BB, step three): could reach, on the walking grid, and did come, from
    // the households' own record of the past 30 days.
    use civ_agents::uses::{Place, Worked};
    use civ_schema::{flatbuffers, wire};
    use civ_sim::frames::coverage::{self, OTHERS_WELL, OWN_WELL, WATER};

    let (mut sim, h, well) = an_open_well();
    sim.advance_minutes(3 * DAY).expect("lives");
    let homes = coverage::water_homes(&sim, None);
    assert_eq!(homes.len(), households(&sim).len());
    let metres_a_minute = sim.nav().params().top_speed_ms() * 60.0;
    let cell = f64::from(sim.map().cell_size_m);
    for x in &homes {
        // Every home has water within the search, never nearer by the grid than in a straight
        // line at the fastest walk (half a cell's slack each end for where in its cell each is).
        let n = x.nearest.as_ref().expect("water within reach");
        let walk = f64::from(n.minutes.expect("its walk"));
        let straight = (f64::from(n.at.0 - x.at.0)).hypot(f64::from(n.at.1 - x.at.1));
        assert!(
            walk * metres_a_minute + cell * 1.5 >= straight,
            "{walk} min for {straight} m"
        );
        // The nearest by right is given only when it is another, and is never another's well.
        if let Some(r) = &x.by_right {
            assert_ne!(r.kind, OTHERS_WELL);
            assert!(r.minutes >= n.minutes);
            assert_ne!(Some(r), x.nearest.as_ref());
        }
        // Everyone draws water every day: within the four days lived here, on one day at least,
        // and no source on more days than that.
        assert!(!x.drew.is_empty(), "{:?}", x.household);
        assert!(x.drew.iter().all(|s| (1..=30).contains(&s.days)));
    }
    // Their own record, exactly: the well's household drew at it on two days of the past 30 and
    // at the river on one long before, and nowhere else is held.
    let (today, half) = (
        sim.now().day_index(),
        sim.rules().people.places.use_half_life_days,
    );
    let w = *sim.land().wells.get(well).expect("the well");
    let settlement = sim
        .people()
        .households
        .iter()
        .find(|(_, x)| x.id == h)
        .and_then(|(_, x)| x.settlement)
        .expect("a settlement");
    {
        let uses = &mut sim.people_mut_for_tests().uses;
        uses.households.remove(&h);
        uses.today.retain(|x| x.household != h);
        for day in [today - 40, today - 5, today - 2] {
            let place = if day == today - 40 {
                Place::Source(0)
            } else {
                Place::Source(w.cell)
            };
            uses.worked(Worked {
                place,
                day,
                household: h,
                settlement,
                kcal: 0.0,
                person: None,
            });
            uses.fold(half);
        }
        uses.prune(today, half, 365, |_| true);
    }
    let homes = coverage::water_homes(&sim, Some(settlement));
    let mine = homes.iter().find(|x| x.household == h).expect("its home");
    assert_eq!(mine.drew.len(), 1, "{:?}", mine.drew);
    assert_eq!((mine.drew[0].kind, mine.drew[0].days), (OWN_WELL, 2));
    assert!(coverage::home_words(mine)[1].ends_with("'s household's well on 2 days"));
    // On the wire: every home, its words, the legend's parts and the counts.
    let bytes = coverage::coverage_response(&sim, Some(settlement), WATER).expect("the lens");
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let lens = response.body_as_coverage().expect("a lens");
    assert_eq!(lens.homes().map(|v| v.len()), Some(homes.len()));
    assert_eq!(lens.days(), 30);
    assert!(lens.scope().is_some_and(|s| s.ends_with(" homes")));
    let periods: Vec<&str> = lens.periods().expect("periods").iter().collect();
    assert_eq!(
        periods,
        [
            "now, on the paths as last surveyed",
            "the past 30 days, today included"
        ]
    );
    let summary: Vec<&str> = lens.summary().expect("counts").iter().collect();
    assert!(summary[0].starts_with(&format!("Could reach: of {} homes", homes.len())));
    assert!(summary[1].starts_with("Did come: in the past 30 days"));
    assert!(coverage::coverage_response(&sim, None, 9).is_none());
}

/// The well's household, under a rule keeping new homes thirty metres from open wells (M8c slice
/// CW, ADR-0030 §6), its people going nowhere from home, so it seeks a plot about its home alone,
/// beside its own well: it lays its home on clear ground farther afield that keeps the rule, as a
/// first home is laid when nothing is clear near, rather than on ground near that breaks it. A plot
/// breaking a rule its builders keep is laid only where none it seeks keeps it. Before, ground near
/// its old point breaking the rule was taken before ground farther afield keeping it was sought,
/// and a plot breaking it about one point was weighed against plots keeping it about others.
#[test]
fn a_home_breaks_a_siting_rule_only_where_no_ground_it_seeks_keeps_it() {
    let (mut sim, owner, well) = an_open_well();
    let k = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.id == "core:policy/works_wrought_wells")
        .expect("the rule") as u16;
    let settlement = sim
        .people()
        .household(owner)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let (_, members) = household(&sim, owner);
    sim.with_ctx_for_tests(|pop, ctx| pop.enact_rule_for_tests(ctx, pi, k, members[0]));
    let kept = sim.with_ctx_for_tests(|pop, ctx| pop.rules_kept_for_tests(ctx, owner));
    assert_eq!(kept.midden_from, Some(30.0), "{kept:?}");
    let rect = sim.land().wells.get(well).expect("the well").rect;
    // Its old point at its well's mouth, so no plot within the thirty metres a home is moved to
    // find clear ground keeps the rule.
    let home = rect.centre_m();
    let pop = sim.people_mut_for_tests();
    for (_, x) in pop.households.iter_mut() {
        if x.id == owner {
            x.home = home;
        }
    }
    for m in &members {
        pop.travel.goings.remove(m);
    }
    let spec = sim
        .with_ctx_for_tests(|pop, ctx| pop.first_home_for_tests(ctx, owner))
        .expect("ground for a home");
    let rules = sim.rules().clone();
    let def = rules
        .catalog
        .building_index(&spec.program)
        .map(|i| &rules.catalog.buildings[i])
        .expect("its program");
    let plot = civ_agents::build::plot_rect(&spec, def);
    let gap = civ_agents::rules::gap_m(&plot, &rect);
    let (x, y) = civ_agents::build::centre_m(&spec);
    assert!(
        gap >= 30.0,
        "it laid its home {gap:.1} m from the well, {:.0} m from its old point",
        (x - home.0).hypot(y - home.1)
    );
}

#[test]
fn a_rule_keeps_new_homes_and_their_middens_from_the_wells() {
    // M8c slice CW (ADR-0030 §6): a rule for building over a heap's seepage to a well. In force,
    // its builders lay a new home thirty metres from every open well; a home standing within that
    // reach of one breaks it, and one beyond keeps it. Its words say what it asks.
    let (mut sim, owner, well) = an_open_well();
    let k = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.id == "core:policy/works_wrought_wells")
        .expect("the rule") as u16;
    assert_eq!(
        sim.rules().catalog.policies[usize::from(k)].answers,
        vec![civ_agents::polity::IssueKind::Sickness]
    );
    let settlement = sim
        .people()
        .household(owner)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let sponsor = household(&sim, owner).1[0];
    let law = sim.with_ctx_for_tests(|pop, ctx| pop.enact_rule_for_tests(ctx, pi, k, sponsor));
    let kept = sim.with_ctx_for_tests(|pop, ctx| pop.rules_kept_for_tests(ctx, owner));
    assert_eq!(kept.midden_from, Some(30.0), "{kept:?}");
    // A new home is laid where it keeps the rule.
    let rect = sim.land().wells.get(well).expect("the well").rect;
    let rules = sim.rules().clone();
    let catalog = &rules.catalog;
    for h in households(&sim) {
        let Some(spec) = sim.with_ctx_for_tests(|pop, ctx| pop.first_home_for_tests(ctx, h)) else {
            continue;
        };
        let def = catalog
            .building_index(&spec.program)
            .map(|i| &catalog.buildings[i])
            .expect("its program");
        let plot = civ_agents::build::plot_rect(&spec, def);
        assert!(
            civ_agents::rules::gap_m(&plot, &rect) >= 30.0,
            "{h:?} laid its home {} m from the well",
            civ_agents::rules::gap_m(&plot, &rect)
        );
    }
    // Homes standing now are counted as if begun under it: near the well breaks it, far keeps.
    let homes: Vec<(PermanentId, PermanentId, f64)> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| b.standing() && catalog.is_dwelling(&b.spec.program))
        .filter_map(|b| {
            let p = sim.land().plots.iter().find(|p| p.id == b.plot)?;
            Some((b.household, b.id, civ_agents::rules::gap_m(&p.rect, &rect)))
        })
        .collect();
    for &(h, b, _) in &homes {
        sim.with_ctx_for_tests(|pop, ctx| pop.count_rule_keeping_for_tests(ctx, h, b));
    }
    let c = sim.people().polities[pi]
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("the law")
        .compliance;
    let near = homes.iter().filter(|h| h.2 < 30.0).count() as u32;
    assert_eq!(c.broken + c.broken_unaware, near, "{c:?} of {homes:?}");
    assert_eq!(c.complied, homes.len() as u32 - near, "{c:?}");
    assert!(
        near > 0,
        "the well's own household lives near it: {homes:?}"
    );
    assert_eq!(
        civ_sim::frames::word::law_text(
            &sim,
            &sim.people().polities[pi],
            &sim.people().polities[pi]
                .laws
                .iter()
                .find(|l| l.id == law)
                .cloned()
                .expect("the law")
        ),
        "a rule for building: a new home and its midden must stand 30 m from every open well, of \
         what is begun from its day"
    );
}

/// Shedding where people work (ADR-0021 §4: an infected person's shedding goes to the ground
/// where they work, otherwise to their heap): someone shedding by water who worked a field
/// yesterday sheds that share of the day there, by the hours worked over the day's 24, and the
/// rest to their household's heap, the day's shedding whole; the field then washes and seeps as a
/// heap does, from its own place. Someone not shedding is not kept, and the hours kept save and
/// load exactly.
#[test]
fn someone_shedding_who_worked_a_field_sheds_that_share_of_the_day_there() {
    let mut sim = village(6);
    let d = cholera(&sim);
    let def = sim.rules().catalog.diseases[usize::from(d)].clone();
    // A household and a field it works, once the band has broken one.
    for _ in 0..24 {
        if !sim.land().fields.is_empty() {
            break;
        }
        sim.advance_minutes(5 * DAY).expect("lives");
    }
    let (h, field) = households(&sim)
        .into_iter()
        .find_map(|h| {
            sim.land()
                .fields
                .iter()
                .find(|f| f.household == h)
                .map(|f| (h, f.id))
        })
        .expect("a household working a field");
    let (_, members) = household(&sim, h);
    let (who, other) = (members[0], members.get(1).copied());
    let note = |sim: &mut Sim, p: PermanentId, hours: f64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.note_worked_for_tests(ctx, p, field, hours));
    };
    // Not shedding, nobody is kept.
    note(&mut sim, who, 8.0);
    assert!(sim.people().worked.is_empty());
    // Shedding without symptoms from today, as a silent carrier who still works.
    let day = sim.now().day_index();
    sim.people_mut_for_tests().sickness.add(
        who,
        Some(h),
        d,
        day,
        Acquired::Observer { influence: 0 },
        civ_agents::sickness::Course {
            shed_from: day,
            shed_until: day + 30,
            ill_from: day,
            ill_until: day,
            severe: false,
            immune_until: day + 365,
        },
    );
    note(&mut sim, who, 6.0);
    note(&mut sim, who, 2.0);
    if let Some(o) = other {
        note(&mut sim, o, 8.0);
    }
    assert_eq!(
        sim.people().worked,
        vec![(who, field, 8.0)],
        "only the shedder, summed"
    );
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().worked, sim.people().worked);
    // Midnight: a third of the day's shedding to the field, the rest to the heap.
    sim.land_mut_for_tests().water.last_surplus_mm = 0.0;
    let today = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let c = &sim.people().contagion;
    let shed = |to: Node| -> f64 {
        c.moves
            .iter()
            .filter(|m| m.day == today && m.how == How::Shed && m.to == to)
            .map(|m| m.amount)
            .sum()
    };
    let daily = def.shed_silent_per_day;
    let (there, home) = (shed(Node::Field(field)), shed(Node::Midden(h)));
    assert!(
        (there - daily * 8.0 / 24.0).abs() < 1e-9,
        "{there} at the field of {daily}"
    );
    assert!(
        (there + home - daily).abs() < 1e-9,
        "the day's shedding whole"
    );
    // The field holds it, less what soaks in toward wells and rivers the same day.
    let seep = sim.rules().land.contamination.seep_per_day;
    let held = c.load(Node::Field(field), d);
    assert!(
        (held - there * (1.0 - seep)).abs() < 1e-9,
        "{held} held of {there}"
    );
    assert!(sim.people().worked.is_empty(), "drawn at midnight");
    // Lived on, the day's field work keeps their hours there by the evening.
    let mut kept = None;
    for _ in 0..20 {
        to_morning(&mut sim, 20 * 60);
        if let Some(w) = sim.people().worked.iter().find(|w| w.0 == who) {
            kept = Some(*w);
            break;
        }
    }
    let (_, worked_field, hours) = kept.expect("a day's field work kept");
    assert!(hours > 0.0 && sim.land().fields.iter().any(|f| f.id == worked_field));
}

/// Shedding where people work, on other ground (ADR-0021 §4, slice 2): someone shedding by water
/// who worked at a pit, a gathering block or a way sheds that share of the day into the ground of
/// the cell they worked on, by its hours over 24, the rest to their heap; that ground then washes
/// and seeps as a heap does, from the cell's middle. Hours worked off the land are not kept. The
/// hours kept save and load exactly, and a lived day's work keeps them.
#[test]
fn someone_shedding_who_worked_other_ground_sheds_that_share_of_the_day_there() {
    let mut sim = village(6);
    let d = cholera(&sim);
    let daily = sim.rules().catalog.diseases[usize::from(d)].shed_silent_per_day;
    let h = households(&sim)[0];
    let (home, members) = household(&sim, h);
    let who = members[0];
    let day = sim.now().day_index();
    sim.people_mut_for_tests().sickness.add(
        who,
        Some(h),
        d,
        day,
        Acquired::Observer { influence: 0 },
        civ_agents::sickness::Course {
            shed_from: day,
            shed_until: day + 60,
            ill_from: day,
            ill_until: day,
            severe: false,
            immune_until: day + 365,
        },
    );
    // A land cell 40 m from home, and the nearest cell off the land, if any.
    let map = sim.map();
    let land_at = (0..16)
        .map(|k| f32::from(k as u8) * std::f32::consts::PI / 8.0)
        .map(|a| (home.0 + 40.0 * a.cos(), home.1 + 40.0 * a.sin()))
        .find(|&at| map.water[cell_of(map, at)] == civ_world::WATER_LAND)
        .expect("land near home");
    let water_at = (0..map.water.len())
        .find(|&c| map.water[c] != civ_world::WATER_LAND)
        .map(|c| civ_agents::population::cell_centre(map, c));
    let cell = cell_of(map, land_at) as u32;
    let note = |sim: &mut Sim, at: (f32, f32), hours: f64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.note_worked_ground_for_tests(ctx, who, at, hours));
    };
    note(&mut sim, land_at, 4.0);
    note(&mut sim, land_at, 2.0);
    if let Some(w) = water_at {
        note(&mut sim, w, 5.0);
    }
    assert_eq!(
        sim.people().worked_ground,
        vec![(who, cell, 6.0)],
        "land only, summed"
    );
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().worked_ground, sim.people().worked_ground);
    sim.land_mut_for_tests().water.last_surplus_mm = 0.0;
    let today = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let c = &sim.people().contagion;
    let shed = |to: Node| -> f64 {
        c.moves
            .iter()
            .filter(|m| m.day == today && m.how == How::Shed && m.to == to)
            .map(|m| m.amount)
            .sum()
    };
    let (there, at_home) = (shed(Node::Ground(cell)), shed(Node::Midden(h)));
    assert!(
        (there - daily * 6.0 / 24.0).abs() < 1e-9,
        "{there} on the ground of {daily}"
    );
    assert!(
        (there + at_home - daily).abs() < 1e-9,
        "the day's shedding whole"
    );
    let seep = sim.rules().land.contamination.seep_per_day;
    let held = c.load(Node::Ground(cell), d);
    assert!(
        (held - there * (1.0 - seep)).abs() < 1e-9,
        "{held} held of {there}"
    );
    assert!(sim.people().worked_ground.is_empty(), "drawn at midnight");
    // It saves and loads with the ground's load.
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().contagion, sim.people().contagion);
    // Lived on, a day's gathering, digging or work on a way keeps their hours by the evening.
    let mut kept = None;
    for _ in 0..30 {
        to_morning(&mut sim, 20 * 60);
        if let Some(w) = sim.people().worked_ground.iter().find(|w| w.0 == who) {
            kept = Some(*w);
            break;
        }
    }
    let (_, worked_cell, hours) = kept.expect("a day's work on the ground kept");
    assert!(hours > 0.0);
    assert_eq!(sim.map().water[worked_cell as usize], civ_world::WATER_LAND);
}

/// Shedding where people work, slice 3 (ADR-0021 §4): the outdoor work that kept no hours now
/// keeps them, through the same call: here, a household digging its well, all its people shedding
/// silently, keeps hours on the well's ground by the evening of a day's digging. And on the cell of
/// their own home they are at home: nothing is kept there, so they shed into their heap.
#[test]
fn shedders_digging_a_well_keep_their_hours_there_and_none_at_home() {
    let mut sim = village(6);
    let (h, well) = a_well_begun(&mut sim);
    let d = cholera(&sim);
    let (home, members) = household(&sim, h);
    let day = sim.now().day_index();
    for &m in &members {
        sim.people_mut_for_tests().sickness.add(
            m,
            Some(h),
            d,
            day,
            Acquired::Observer { influence: 0 },
            civ_agents::sickness::Course {
                shed_from: day,
                shed_until: day + 60,
                ill_from: day,
                ill_until: day,
                severe: false,
                immune_until: day + 365,
            },
        );
    }
    // At home: nothing kept.
    sim.with_ctx_for_tests(|pop, ctx| pop.note_worked_ground_for_tests(ctx, members[0], home, 3.0));
    assert!(
        sim.people().worked_ground.is_empty(),
        "at home, nothing kept"
    );
    // The well begun beside the home lies on the home's own cell, where digging is at home; set
    // it on the land 24 m off, and a day's digging there keeps the diggers' hours on its ground.
    let home_cell = cell_of(sim.map(), home) as u32;
    let w = *sim.land().wells.get(well).expect("the well");
    let moved = (1..8i32)
        .map(|k| (k * 800, 0))
        .chain((1..8i32).map(|k| (0, k * 800)))
        .map(|(dx, dy)| {
            let mut r = w.rect;
            r.x += dx;
            r.y += dy;
            r
        })
        .find(|r| {
            let c = cell_of(sim.map(), r.centre_m());
            c as u32 != home_cell
                && sim.map().water[c] == civ_world::WATER_LAND
                && (r.centre_m().0 - home.0).hypot(r.centre_m().1 - home.1) <= 30.0
        })
        .expect("land beside the home");
    let at = cell_of(sim.map(), moved.centre_m()) as u32;
    for x in &mut sim.land_mut_for_tests().wells.list {
        if x.id == well {
            x.rect = moved;
            x.cell = at;
        }
    }
    let mut kept = None;
    for _ in 0..30 {
        to_morning(&mut sim, 20 * 60);
        if let Some(k) = sim
            .people()
            .worked_ground
            .iter()
            .find(|k| members.contains(&k.0) && k.1 == at)
        {
            kept = Some(*k);
            break;
        }
        if sim.land().wells.get(well).is_some_and(|w| w.is_open()) {
            break;
        }
    }
    let (who, _, hours) = kept.expect("hours kept at the well");
    assert!(members.contains(&who) && hours > 0.0);
}

/// A silent carrier from today in household `h`: its first member, shedding cholera for 60 days.
fn a_silent_carrier(sim: &mut Sim, h: PermanentId) -> PermanentId {
    let d = cholera(sim);
    let who = household(sim, h).1[0];
    let day = sim.now().day_index();
    sim.people_mut_for_tests().sickness.add(
        who,
        Some(h),
        d,
        day,
        Acquired::Observer { influence: 0 },
        civ_agents::sickness::Course {
            shed_from: day,
            shed_until: day + 60,
            ill_from: day,
            ill_until: day,
            severe: false,
            immune_until: day + 365,
        },
    );
    who
}

/// Today's shedding moves to node `to`, doses.
fn shed_to(sim: &Sim, day: i64, to: Node) -> f64 {
    sim.people()
        .contagion
        .moves
        .iter()
        .filter(|m| m.day == day && m.how == How::Shed && m.to == to)
        .map(|m| m.amount)
        .sum()
}

/// Shedding where people work, slice 4 (ADR-0021 §4): at another household's home (teaching,
/// study, writing or reading there) that home's heap takes the share, its heap being the node
/// the kernel keeps for a home's ground; the hours save and load exactly. Tending stays on its own
/// route: a carrier who tends another household keeps no hours, and all the day's shedding goes
/// to their own heap, nothing counted twice.
#[test]
fn at_another_s_home_its_heap_takes_the_share_and_tending_keeps_none() {
    let mut sim = village(6);
    let d = cholera(&sim);
    let daily = sim.rules().catalog.diseases[usize::from(d)].shed_silent_per_day;
    let hh = households(&sim);
    let (h, host) = (hh[0], hh[1]);
    let who = a_silent_carrier(&mut sim, h);
    let (host_home, _) = household(&sim, host);
    // Three hours at the host's home, as a learner or teacher there.
    sim.with_ctx_for_tests(|pop, ctx| pop.note_worked_ground_for_tests(ctx, who, host_home, 3.0));
    assert_eq!(sim.people().worked_homes, vec![(who, host, 3.0)]);
    assert!(sim.people().worked_ground.is_empty(), "not the ground");
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().worked_homes, sim.people().worked_homes);
    sim.land_mut_for_tests().water.last_surplus_mm = 0.0;
    let today = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let (there, home) = (
        shed_to(&sim, today, Node::Midden(host)),
        shed_to(&sim, today, Node::Midden(h)),
    );
    assert!(
        (there - daily * 3.0 / 24.0).abs() < 1e-9,
        "{there} at the host's heap"
    );
    assert!(
        (there + home - daily).abs() < 1e-9,
        "the day's shedding whole"
    );
    assert!(sim.people().worked_homes.is_empty(), "drawn at midnight");
    // Tending the host's people: no hours kept, and all of the next day's shedding at home.
    sim.advance_minutes(DAY).expect("lives");
    sim.people_mut_for_tests().worked_ground.clear();
    sim.people_mut_for_tests().worked_homes.clear();
    sim.people_mut_for_tests().worked.clear();
    sim.with_ctx_for_tests(|pop, ctx| pop.tended_for_tests(ctx, who, host, 120));
    let p = sim.people();
    assert!(
        p.worked.is_empty() && p.worked_ground.is_empty() && p.worked_homes.is_empty(),
        "tending keeps no hours"
    );
    // (The day lived has had its own midnight: what it shed is counted apart.)
    let today = sim.now().day_index();
    let anywhere = |sim: &Sim| -> f64 {
        sim.people()
            .contagion
            .moves
            .iter()
            .filter(|m| m.day == today && m.how == How::Shed && m.from == Node::Person(who))
            .map(|m| m.amount)
            .sum()
    };
    let (home0, all0) = (shed_to(&sim, today, Node::Midden(h)), anywhere(&sim));
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let home = shed_to(&sim, today, Node::Midden(h)) - home0;
    let all = anywhere(&sim) - all0;
    assert!((home - daily).abs() < 1e-9, "{home}: all at home");
    assert!((all - daily).abs() < 1e-9, "{all}: once");
}

/// Shedding where people work, slice 4: work in another settlement sheds into the ground there,
/// and while it lasts it counts toward what is left of the disease with the settlement whose
/// hearth lies nearest it, not the worker's own.
#[test]
fn work_in_another_settlement_sheds_there_and_counts_with_its_hearth() {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed: 3,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates");
    assert_eq!(sim.land().settlements.len(), 2);
    let d = cholera(&sim);
    let daily = sim.rules().catalog.diseases[usize::from(d)].shed_silent_per_day;
    let h = households(&sim)[0];
    let ours = sim
        .people()
        .household(h)
        .expect("home")
        .settlement
        .expect("ours");
    let theirs = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id != ours)
        .cloned()
        .expect("the other");
    let who = a_silent_carrier(&mut sim, h);
    // Four hours on land beside the other settlement's hearth, away from any home there.
    let map = sim.map();
    let at = (0..16)
        .map(|k| f32::from(k as u8) * std::f32::consts::PI / 8.0)
        .map(|a| {
            (
                theirs.hearth_m.0 + 24.0 * a.cos(),
                theirs.hearth_m.1 + 24.0 * a.sin(),
            )
        })
        .find(|&at| {
            let c = cell_of(map, at);
            map.water[c] == civ_world::WATER_LAND
                && !households(&sim)
                    .iter()
                    .any(|&x| cell_of(map, household(&sim, x).0) == c)
        })
        .expect("open land by their hearth");
    let cell = cell_of(map, at) as u32;
    sim.with_ctx_for_tests(|pop, ctx| pop.note_worked_ground_for_tests(ctx, who, at, 4.0));
    assert_eq!(sim.people().worked_ground, vec![(who, cell, 4.0)]);
    sim.land_mut_for_tests().water.last_surplus_mm = 0.0;
    let today = sim.now().day_index();
    let left = |sim: &mut Sim, s| {
        sim.with_ctx_for_tests(|pop, ctx| pop.disease_left_for_tests(ctx, d, Some(s)))
    };
    let (ours0, theirs0) = (left(&mut sim, ours), left(&mut sim, theirs.id));
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let there = shed_to(&sim, today, Node::Ground(cell));
    assert!(
        (there - daily * 4.0 / 24.0).abs() < 1e-9,
        "{there} shed there"
    );
    let held = sim.people().contagion.load(Node::Ground(cell), d);
    assert!(held > 0.0);
    let theirs1 = left(&mut sim, theirs.id);
    let ours1 = left(&mut sim, ours);
    eprintln!("theirs {theirs0:.2} -> {theirs1:.2}; ours {ours0:.2} -> {ours1:.2}; held {held:.2}");
    assert!(
        theirs1 - theirs0 >= held - 1e-9,
        "counted with the hearth nearest: {theirs0} -> {theirs1}"
    );
    // Ours holds the rest of the day at the heap, not the ground there.
    let at_home = sim.people().contagion.load(Node::Midden(h), d);
    assert!(
        (ours1 - ours0 - at_home).abs() < 1e-6 + 1e-9 * ours1,
        "{ours0} -> {ours1}, heap {at_home}"
    );
}

/// Household `h`'s stores set to hold only `kg` of each of `held` (good id, kg) of food, every
/// other food at none.
fn food_only(sim: &mut Sim, h: PermanentId, held: &[(usize, f64)]) {
    let goods = sim.rules().catalog.goods.clone();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            x.stores.resize(goods.len(), 0.0);
            for (g, d) in goods.iter().enumerate() {
                if d.edible() {
                    x.stores[g] = 0.0;
                }
            }
            for &(g, kg) in held {
                x.stores[g] = kg;
            }
        }
    }
}

/// The load of `d` on every food node, got or made here, doses.
fn food_load(sim: &Sim, d: u16) -> f64 {
    sim.people()
        .contagion
        .loads
        .iter()
        .filter(|((n, dd), _)| *dd == d && matches!(n, Node::Food { .. } | Node::MadeFood { .. }))
        .map(|(_, &l)| l)
        .sum()
}

/// Food that carries what its maker shed, slice 1 (ADR-0045): a silent carrier baking for a
/// workshop puts `food_share` of their hours' shedding onto its bread, taken ahead from the next
/// midnight's shedding; selling half the bread to another household moves half the load with it;
/// a member who eats there takes the share of the load they ate, a dose drawn at midnight, and
/// with enough of a dose takes the disease by food, named as that household's bread. A stock
/// lowered by a path the route does not handle loses its load in proportion; one raised is
/// diluted; nothing is ever made. Every state saves and loads exactly.
#[test]
fn a_carrier_s_bread_carries_its_load_to_those_who_buy_and_eat_it() {
    let mut sim = village(6);
    let d = cholera(&sim);
    let daily = sim.rules().catalog.diseases[usize::from(d)].shed_silent_per_day;
    let share = sim.rules().land.contamination.food_share;
    assert!(share > 0.0, "the core's food share");
    let rules = sim.rules().clone();
    let good = |id: &str| {
        rules
            .catalog
            .goods
            .iter()
            .position(|g| g.id == id)
            .expect(id)
    };
    let (bread, flour, firewood) = (
        good("core:good/bread"),
        good("core:good/flour"),
        good("core:good/firewood"),
    );
    let bake = rules
        .catalog
        .activities
        .iter()
        .position(|a| a.id == "core:activity/bake_bread")
        .expect("baking");
    let hh = households(&sim);
    let (owner, buyer) = (hh[0], hh[1]);
    let baker = a_silent_carrier(&mut sim, owner);
    let firm = sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, baker, bread))
        .expect("a bakery");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == owner {
            x.stores.resize(rules.catalog.goods.len(), 0.0);
            x.stores[flour] = 20.0;
            x.stores[firewood] = 20.0;
        }
    }
    assert!(food_load(&sim, d) == 0.0 && !sim.people().contagion.food_loaded());
    // An hour's baking: a tenth of an hour's shedding goes onto the bakery's bread.
    sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, baker, bake, 60, firm));
    let firm_bread = sim.people().firm(firm).expect("open").stores[bread];
    assert!(firm_bread > 0.0, "bread made");
    let made = daily / 24.0 * share;
    // Its baker is of the household whose workshop it is: the load is "made here" (slice 2).
    let on_firm = sim.people().contagion.load(
        Node::MadeFood {
            holder: firm,
            good: bread as u16,
        },
        d,
    );
    assert!(
        (on_firm - made).abs() < 1e-12,
        "{on_firm} on the bread of {made}"
    );
    assert_eq!(
        sim.people().contagion.owed.get(&(baker, d)).copied(),
        Some(made),
        "taken ahead from the next midnight's shedding"
    );
    // Half the bread sold to the buyer moves half the load.
    let sold = firm_bread / 2.0;
    food_only(&mut sim, buyer, &[]);
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.transfer_for_tests(ctx, firm, buyer, bread, sold))
    );
    let theirs = Node::Food {
        holder: buyer,
        good: bread as u16,
    };
    let at_buyer = sim.people().contagion.load(theirs, d);
    assert!((at_buyer - made / 2.0).abs() < 1e-12, "{at_buyer}");
    assert!((food_load(&sim, d) - made).abs() < 1e-12, "conserved");
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().contagion, sim.people().contagion);
    // A member eats a quarter of the buyer's bread: a quarter of its load is their dose.
    let eater = household(&sim, buyer).1[0];
    let stock = sim.people().household(buyer).expect("buyer").stores[bread];
    sim.with_ctx_for_tests(|pop, ctx| pop.eat_for_tests(ctx, eater));
    let left = sim.people().household(buyer).expect("buyer").stores[bread];
    let ate = stock - left;
    assert!(ate > 0.0, "ate bread");
    let eaten: Vec<_> = sim.people().contagion.eaten.clone();
    assert_eq!(eaten.len(), 1);
    let (who, dd, dose, source) = eaten[0];
    assert_eq!((who, dd, source), (eater, d, theirs));
    assert!((dose - at_buyer * ate / stock).abs() < 1e-12, "{dose}");
    assert!(
        (food_load(&sim, d) + dose - made).abs() < 1e-12,
        "what is on food and what was eaten is what was made"
    );
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().contagion, sim.people().contagion);
    // A stock lowered by a path the route does not handle loses its load in proportion; one
    // raised is diluted; neither makes any.
    let before = food_load(&sim, d);
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == buyer {
            x.stores[bread] /= 2.0;
        }
    }
    let other = household(&sim, buyer).1.get(1).copied().unwrap_or(eater);
    sim.with_ctx_for_tests(|pop, ctx| pop.eat_for_tests(ctx, other));
    assert!(
        food_load(&sim, d) < before,
        "lost with the food, never made"
    );
    let at_buyer = sim.people().contagion.load(theirs, d);
    let stock = sim.people().household(buyer).expect("buyer").stores[bread];
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == buyer {
            x.stores[bread] = stock * 4.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.eat_for_tests(ctx, eater));
    assert!(
        sim.people().contagion.load(theirs, d) <= at_buyer,
        "diluted, not made"
    );
    // The baker's next midnight: the day's shedding less what went onto the bread ahead.
    let today = sim.now().day_index();
    sim.land_mut_for_tests().water.last_surplus_mm = 0.0;
    sim.people_mut_for_tests().contagion.eaten.clear();
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let shed: f64 = sim
        .people()
        .contagion
        .moves
        .iter()
        .filter(|m| m.day == today && m.how == How::Shed && m.from == Node::Person(baker))
        .map(|m| m.amount)
        .sum();
    assert!(
        (shed - (daily - made)).abs() < 1e-9,
        "{shed} shed of {daily}"
    );
    assert!(sim.people().contagion.owed.is_empty());
    // With a heavy load on the buyer's bread, the one who eats it takes the disease by food.
    let stock = sim.people().household(buyer).expect("buyer").stores[bread];
    sim.people_mut_for_tests()
        .contagion
        .food_add(theirs, d, 1.0e6, stock);
    let victim = household(&sim, buyer).1.last().copied().expect("someone");
    sim.with_ctx_for_tests(|pop, ctx| pop.eat_for_tests(ctx, victim));
    assert!(sim.people().contagion.eaten.iter().any(|e| e.0 == victim));
    let reloaded_now = reloaded(&mut sim);
    assert_eq!(reloaded_now.people().contagion, sim.people().contagion);
    to_morning(&mut sim, 60);
    let took = sim
        .people()
        .sickness
        .episodes()
        .iter()
        .find(|e| e.person == victim)
        .map(|e| e.acquired);
    assert_eq!(
        took,
        Some(civ_agents::sickness::Acquired::Food { source: theirs }),
        "taken by food"
    );
    assert!(sim.people().contagion.eaten.is_empty(), "drawn at midnight");
}

/// Food that carries what its maker shed, slice 2 (ADR-0045): a household's own prepared food is
/// "made here": its own people who eat it take no dose, the load leaving with what they ate; sold
/// or given to a neighbour, the load goes with it as food got from elsewhere, and the neighbour's
/// people who eat it are dosed. A workshop's bread follows who made it: made by its owners'
/// people it stays theirs when they draw it, undosed; made by someone hired, it doses its owners
/// as any buyer. Every state saves and loads exactly.
#[test]
fn a_household_s_own_bread_doses_its_neighbours_and_not_its_own() {
    let mut sim = village(6);
    let d = cholera(&sim);
    let rules = sim.rules().clone();
    let good = |id: &str| {
        rules
            .catalog
            .goods
            .iter()
            .position(|g| g.id == id)
            .expect(id)
    };
    let (bread, flour, firewood) = (
        good("core:good/bread"),
        good("core:good/flour"),
        good("core:good/firewood"),
    );
    let bake = rules
        .catalog
        .activities
        .iter()
        .position(|a| a.id == "core:activity/bake_bread")
        .expect("baking");
    let hh = households(&sim);
    let (home, neighbour, other) = (hh[0], hh[1], hh[2]);
    let baker = a_silent_carrier(&mut sim, home);
    let stock = |sim: &mut Sim, h: PermanentId, flour_kg: f64| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == h {
                x.stores.resize(rules.catalog.goods.len(), 0.0);
                x.stores[flour] = flour_kg;
                x.stores[firewood] = 20.0;
            }
        }
    };
    stock(&mut sim, home, 20.0);
    food_only(&mut sim, home, &[(flour, 20.0)]);
    stock(&mut sim, home, 20.0);
    // An hour's baking at home: the load is the household's own.
    sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, baker, bake, 60));
    let made_here = Node::MadeFood {
        holder: home,
        good: bread as u16,
    };
    let on = sim.people().contagion.load(made_here, d);
    assert!(on > 0.0, "the home's bread carries its baker's load");
    assert_eq!(
        sim.people().contagion.load(
            Node::Food {
                holder: home,
                good: bread as u16
            },
            d
        ),
        0.0
    );
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().contagion, sim.people().contagion);
    // One of its own eats it: no dose, and the load leaves with what they ate.
    let own = household(&sim, home).1.last().copied().expect("someone");
    let kg = sim.people().household(home).expect("home").stores[bread];
    sim.with_ctx_for_tests(|pop, ctx| pop.eat_for_tests(ctx, own));
    let left = sim.people().household(home).expect("home").stores[bread];
    assert!(left < kg, "they ate bread");
    assert!(
        sim.people().contagion.eaten.is_empty(),
        "no dose to its own"
    );
    let on_left = sim.people().contagion.load(made_here, d);
    assert!((on_left - on * left / kg).abs() < 1e-12, "{on_left}");
    // Given to a neighbour: the load goes as food got from elsewhere, and doses who eats it.
    food_only(&mut sim, neighbour, &[]);
    let give = left / 2.0;
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.transfer_for_tests(ctx, home, neighbour, bread, give)));
    let got = Node::Food {
        holder: neighbour,
        good: bread as u16,
    };
    let at_neighbour = sim.people().contagion.load(got, d);
    assert!(
        (at_neighbour - on_left / 2.0).abs() < 1e-12,
        "{at_neighbour}"
    );
    assert_eq!(
        sim.people().contagion.load(
            Node::MadeFood {
                holder: neighbour,
                good: bread as u16
            },
            d
        ),
        0.0
    );
    let eater = household(&sim, neighbour).1[0];
    sim.with_ctx_for_tests(|pop, ctx| pop.eat_for_tests(ctx, eater));
    assert!(
        sim.people()
            .contagion
            .eaten
            .iter()
            .any(|e| e.0 == eater && e.3 == got),
        "the neighbour is dosed"
    );
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().contagion, sim.people().contagion);
    sim.people_mut_for_tests().contagion.eaten.clear();
    // A workshop's bread: made by its owners' own baker, drawn back undosed; made by a hired
    // baker of another household, it doses the owners as any buyer.
    let firm = sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, home, baker, bread))
        .expect("a bakery");
    stock(&mut sim, home, 20.0);
    for (by, own_made) in [(baker, true), (a_silent_carrier(&mut sim, other), false)] {
        stock(&mut sim, home, 20.0);
        sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, by, bake, 60, firm));
        let kg = sim.people().firm(firm).expect("open").stores[bread];
        assert!(kg > 0.0, "bread made");
        food_only(&mut sim, home, &[]);
        assert!(
            sim.with_ctx_for_tests(|pop, ctx| pop.transfer_for_tests(ctx, firm, home, bread, kg))
        );
        let node = |made: bool| {
            if made {
                Node::MadeFood {
                    holder: home,
                    good: bread as u16,
                }
            } else {
                Node::Food {
                    holder: home,
                    good: bread as u16,
                }
            }
        };
        assert!(
            sim.people().contagion.load(node(own_made), d) > 0.0,
            "{own_made}"
        );
        let eater = household(&sim, home).1[0];
        sim.with_ctx_for_tests(|pop, ctx| pop.eat_for_tests(ctx, eater));
        assert_eq!(
            sim.people().contagion.eaten.iter().any(|e| e.0 == eater),
            !own_made,
            "dosed only from bread someone else made"
        );
        sim.people_mut_for_tests().contagion.eaten.clear();
        food_only(&mut sim, home, &[]);
    }
}
