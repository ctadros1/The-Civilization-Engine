//! Water people choose (M6a slice AY, step two; ADR-0021 §1): a household's people fetch from
//! the nearest spring that flows and has a load left today, if it is nearer than the river or
//! lake, and from the river or lake otherwise; a spring gives what flows to it in a day; every
//! draw is logged as a use of the source; and what a household uses a day follows the walk to
//! the water it last fetched.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::uses::Place;
use civ_content::ContentRegistry;
use civ_sim::{NewWorld, Sim, persist};

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

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Spring".to_owned(),
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

/// Lives on to a minute past the next midnight: the heads are lived at midnight, so what a test
/// sets them to holds until the one after.
fn to_morning(sim: &mut Sim) {
    let rest = DAY - sim.now().minutes().rem_euclid(DAY) + 1;
    sim.advance_minutes(rest).expect("advances");
}

/// The patch off the water whose lowest ground is nearest the hearth, and that cell.
fn spring_site(sim: &Sim) -> (usize, u32) {
    let (map, land) = (sim.map(), sim.land());
    let hearth = land.settlements[0].hearth_m;
    let aq = &land.water.aquifer;
    (0..aq.len())
        .filter(|&p| aq.stage[p].is_none())
        .filter_map(|p| aq.seep[p].map(|(cell, _)| (p, cell)))
        .min_by(|a, b| {
            let d = |c: u32| {
                let (x, y) = civ_agents::population::cell_centre(map, c as usize);
                (x - hearth.0).hypot(y - hearth.1)
            };
            d(a.1).total_cmp(&d(b.1)).then(a.0.cmp(&b.0))
        })
        .expect("a patch off the water")
}

/// Sets the heads round patch `p` so a spring rises there giving `m3` a day, or, with `None`, so
/// its head stands a metre below its lowest ground and nothing rises: its head at its lowest
/// ground, every neighbour's no higher, and one neighbour's high enough to give it `m3`.
fn set_spring(sim: &mut Sim, p: usize, m3: Option<f64>) {
    let water = &mut sim.land_mut_for_tests().water;
    let aq = &water.aquifer;
    let cols = aq.cols as usize;
    let z = f64::from(aq.seep[p].expect("lowest ground").1);
    let (x, y) = (p % cols, p / cols);
    let mut round = Vec::new();
    if x + 1 < cols {
        round.push((p + 1, aq.c_east[p]));
    }
    if x > 0 {
        round.push((p - 1, aq.c_east[p - 1]));
    }
    if y + 1 < aq.rows as usize {
        round.push((p + cols, aq.c_south[p]));
    }
    if y > 0 {
        round.push((p - cols, aq.c_south[p - cols]));
    }
    let (q, c) = round
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("a neighbour");
    assert!(c > 0.0, "the ground passes water");
    for &(n, _) in &round {
        water.heads[n] = water.heads[n].min(z);
    }
    match m3 {
        Some(m3) => {
            water.heads[p] = z;
            water.heads[q] = z + m3 / c;
        }
        None => water.heads[p] = z - 1.0,
    }
}

/// Loads of water drawn today, logged at the source `cell` and at any other.
fn draws(sim: &Sim, cell: u32) -> (usize, usize) {
    let today = &sim.people().uses.today;
    let at = |w: &&civ_agents::uses::Worked| w.place == Place::Source(cell);
    let here = today.iter().filter(at).count();
    let elsewhere = today
        .iter()
        .filter(|w| matches!(w.place, Place::Source(c) if c != cell))
        .count();
    (here, elsewhere)
}

#[test]
fn a_spring_nearer_than_the_river_gives_its_days_water_and_then_people_go_to_the_river() {
    let mut sim = village(6);
    to_morning(&mut sim);
    let (p, cell) = spring_site(&sim);
    // The least a spring flows to be a source: 500 L, a little over 33 loads of 15 L.
    let flow = sim.rules().land.water.spring_min_m3_day;
    set_spring(&mut sim, p, Some(flow));
    let day = sim.now().day_index();
    let spring = sim
        .land()
        .water
        .spring_at(&sim.rules().land.water, p)
        .expect("a spring rises");
    assert!((spring.flow_m3_day - flow).abs() < 1e-9);
    sim.advance_minutes(DAY - 60).expect("advances");
    let (here, elsewhere) = draws(&sim, cell);
    let drawn = sim.people().spring_draws.drawn(day, p as u32);
    let load = sim.rules().people.household.carry_water_l;
    assert!(here >= 30, "only {here} loads were drawn at the spring");
    assert!((drawn - here as f64 * load).abs() < 1e-6);
    // Nobody sets off for it once a load is not left, though some on their way may find less.
    assert!(
        drawn <= flow * 1000.0 + 5.0 * load,
        "{drawn} L drawn of {flow} m³"
    );
    assert!(
        elsewhere > 0,
        "once the spring was drawn nobody went to the river"
    );
    // What households use follows the walk to the water they fetched: the profile's 20 L, the
    // spring and the river both being near.
    let households = &sim.people().households;
    assert!(households.iter().any(|(_, h)| h.water_use_l == 20.0));
    assert!(
        households
            .iter()
            .all(|(_, h)| h.water_use_l == 0.0 || h.water_use_l == 20.0)
    );
    // The next day the spring gives again.
    to_morning(&mut sim);
    assert_eq!(sim.people().spring_draws.drawn(day + 1, p as u32), 0.0);
}

#[test]
fn a_spring_that_does_not_flow_is_left_for_the_river() {
    let mut sim = village(6);
    to_morning(&mut sim);
    let (p, cell) = spring_site(&sim);
    set_spring(&mut sim, p, None);
    assert!(
        sim.land()
            .water
            .spring_at(&sim.rules().land.water, p)
            .is_none()
    );
    sim.advance_minutes(DAY - 60).expect("advances");
    let (here, elsewhere) = draws(&sim, cell);
    assert_eq!(here, 0);
    assert!(elsewhere > 0, "nobody fetched water");
}

#[test]
fn draws_at_a_spring_save_load_and_go_on_alike() {
    let mut sim = village(6);
    to_morning(&mut sim);
    let (p, _) = spring_site(&sim);
    let flow = sim.rules().land.water.spring_min_m3_day;
    set_spring(&mut sim, p, Some(flow));
    sim.advance_minutes(10 * 60).expect("advances");
    let day = sim.now().day_index();
    assert!(sim.people().spring_draws.drawn(day, p as u32) > 0.0);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "spring",
    )
    .expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(&sim, &loaded);
    sim.advance_minutes(DAY).expect("advances");
    loaded.advance_minutes(DAY).expect("advances");
    same(&sim, &loaded);
}
