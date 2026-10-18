//! The river for boats and the dugout as a thing (M8b slice CO, first part; ADR-0029 §2, §5, §7):
//! a dugout goes down the river valley's trunk faster than up it, only where today's stage is
//! above its draft, by the quickest way the oracle also finds; a boat is a record that belongs to
//! a household, stands at a bank, ages by the day, is gone once its life is spent, and saves and
//! goes on alike.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_land::boating::{Hull, RiverGraph, going, velocity};
use civ_land::boats::Boat;
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
            name: "Dugouts".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [11; 16],
    )
    .expect("generates")
}

/// The core dugout's catalog index and hull.
fn dugout(sim: &Sim) -> (usize, Hull) {
    let i = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/dugout")
        .expect("the core has a dugout");
    let v = &sim.rules().catalog.vehicles[i];
    (i, v.hull())
}

#[test]
fn the_core_dugout_is_a_boat_made_with_an_axe_by_wood_shaping() {
    let sim = village(1);
    let (i, hull) = dugout(&sim);
    let catalog = &sim.rules().catalog;
    let v = &catalog.vehicles[i];
    assert_eq!(v.mode, civ_agents::params::VehicleMode::Boat);
    assert_eq!(v.payload_kg, 400.0);
    assert_eq!(hull.draft_m, 0.3);
    assert_eq!(
        v.technique.map(|t| catalog.techniques[t].id.as_str()),
        Some("core:technique/wood_shaping")
    );
    assert_eq!(
        v.tool.map(|g| catalog.goods[g].id.as_str()),
        Some("core:good/axe")
    );
    assert!((v.life_days() - 12.0 * 365.25).abs() < 1e-9);
}

#[test]
fn a_dugout_goes_down_the_trunk_faster_than_up_it_and_only_where_it_is_deep_enough() {
    let sim = village(1);
    let (_, hull) = dugout(&sim);
    let map = sim.map();
    let land = sim.land();
    let graph = RiverGraph::new(map);
    assert!(!graph.is_empty(), "the river valley has a river");
    let hyd = &land.floodplain.reaches;
    let q = &land.reach_discharges;
    // The trunk: the reach with the most water today.
    let trunk = (0..hyd.len())
        .max_by(|&a, &b| q[a].total_cmp(&q[b]))
        .expect("a reach");
    let cells: Vec<u32> = map.reaches[trunk]
        .cells
        .iter()
        .copied()
        .filter(|&c| graph.is_river(c))
        .collect();
    assert!(
        cells.len() > 10,
        "the trunk is long enough: {}",
        cells.len()
    );
    let g = going(&hyd[trunk], f64::from(q[trunk]), hull);
    assert!(
        g.stage_m > hull.draft_m,
        "the trunk floats a dugout today: stage {} m",
        g.stage_m
    );
    let v = velocity(&hyd[trunk], f64::from(q[trunk]));
    assert!(v > 0.0 && (g.down_m_s.expect("downstream") - (hull.paddle_m_s + v)).abs() < 1e-12);
    let (top, bottom) = (cells[0], *cells.last().expect("cells"));
    let down = graph
        .route(hyd, q, hull, &[top], &[bottom])
        .expect("down the trunk");
    assert_eq!(down.cells.first(), Some(&top));
    assert_eq!(down.cells.last(), Some(&bottom));
    let oracle = |from: u32, to: u32| graph.oracle_seconds(hyd, q, hull, &[from], &[to]);
    assert!((down.seconds - oracle(top, bottom).expect("oracle")).abs() < 1e-6);
    match graph.route(hyd, q, hull, &[bottom], &[top]) {
        Some(up) => {
            assert!(
                up.seconds > down.seconds,
                "against the current is slower: {} s up, {} s down",
                up.seconds,
                down.seconds
            );
            assert!((up.seconds - oracle(bottom, top).expect("oracle")).abs() < 1e-6);
            assert!((up.metres - down.metres).abs() < 1e-6);
        }
        None => assert!(g.up_m_s.is_none(), "upstream closed only by the current"),
    }
    // The quickest way between any two cells of reaches deep enough today is the oracle's.
    let all: Vec<u32> = map
        .reaches
        .iter()
        .filter(|r| {
            let i = r.id as usize;
            going(&hyd[i], f64::from(q[i]), hull).down_m_s.is_some()
        })
        .flat_map(|r| r.cells.iter().copied())
        .filter(|&c| graph.is_river(c))
        .collect();
    assert!(
        all.len() > cells.len(),
        "more than the trunk floats a dugout"
    );
    let mut checked = 0;
    for k in 0..40 {
        let (a, b) = (
            all[(k * 7919) % all.len()],
            all[(k * 104_729 + 13) % all.len()],
        );
        let route = graph.route(hyd, q, hull, &[a], &[b]).map(|r| r.seconds);
        let truth = oracle(a, b);
        match (route, truth) {
            (Some(r), Some(t)) => {
                assert!((r - t).abs() < 1e-6, "{a} to {b}: {r} s against {t} s");
                checked += 1;
            }
            (None, None) => {}
            other => panic!("{a} to {b}: route and oracle disagree: {other:?}"),
        }
    }
    assert!(checked >= 5, "enough pairs joined: {checked}");
    // Low water closes the river; a hull that needs more water than any reach has goes nowhere.
    let low: Vec<f32> = q.iter().map(|x| x * 1e-4).collect();
    assert!(graph.route(hyd, &low, hull, &[top], &[bottom]).is_none());
    let deep = Hull {
        draft_m: 50.0,
        ..hull
    };
    assert!(graph.route(hyd, q, deep, &[top], &[bottom]).is_none());
}

/// Saves `sim`, loads it, and checks every section is the same and that the two go on alike for
/// `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "boats").expect("saves");
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

#[test]
fn a_boat_belongs_to_a_household_stands_at_a_bank_ages_and_is_gone_when_spent() {
    let mut sim = village(1);
    let (vehicle, _) = dugout(&sim);
    let graph = RiverGraph::new(sim.map());
    // The bank nearest the first household's home.
    let (owner, home) = sim
        .people()
        .households
        .iter()
        .next()
        .map(|(_, h)| (h.id, h.home))
        .expect("a household");
    let map = sim.map();
    let w = map.width as usize;
    let cell_m = map.cell_size_m;
    let d = |c: u32| {
        let x = ((c as usize % w) as f32 + 0.5) * cell_m;
        let y = ((c as usize / w) as f32 + 0.5) * cell_m;
        (x - home.0).hypot(y - home.1)
    };
    let bank = (0..(map.width * map.height))
        .filter(|&c| !graph.boarding(map, c).is_empty())
        .min_by(|&a, &b| d(a).total_cmp(&d(b)).then(a.cmp(&b)))
        .expect("a bank");
    let id = sim.allocate_id_for_tests();
    let made = sim.now().day_index();
    let land = sim.land_mut_for_tests();
    land.boats.list.push(Boat {
        id,
        vehicle: vehicle as u16,
        owner,
        at: bank,
        made,
        wear: 0.0,
    });
    land.boats.changed();
    assert_eq!(sim.land().boats.of(owner).count(), 1);
    // A day spends a day of its twelve years.
    sim.advance_minutes(DAY).expect("lives");
    let wear = sim.land().boats.list[0].wear;
    let day = 1.0 / (12.0 * 365.25);
    assert!((f64::from(wear) - day).abs() < 1e-6, "worn {wear}");
    assert!(sim.land().boats.problems().is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
    // Near the end of its life, it is gone within two days.
    sim.land_mut_for_tests().boats.list[0].wear = 1.0 - day as f32;
    let revision = sim.land().boats.revision();
    sim.advance_minutes(2 * DAY).expect("lives");
    assert!(sim.land().boats.list.is_empty(), "past use, it is gone");
    assert_ne!(sim.land().boats.revision(), revision);
}
