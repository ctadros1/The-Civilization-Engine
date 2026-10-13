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
        let (home, _) = household(&sim, h);
        let elsewhere = cell_of(sim.map(), home) as u32;
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
