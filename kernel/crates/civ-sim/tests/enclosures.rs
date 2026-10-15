//! Enclosures (M6c slice BK, step one): a ring of palisade round a settlement's hearth is laid on
//! the walking grid. A standing sector is crossed only by climbing, slowly; an open gate is the
//! ground and a shut one is the palisade; a sector not yet built, or rotted through, bars nothing;
//! and the line is saved and loaded with the world.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_land::enclosures::{Enclosure, ring};
use civ_sim::{NewWorld, Sim, persist};
use civ_world::nav::RouteResult;

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
            name: "Enclosure".to_owned(),
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

/// The time to walk between two cells on the walking grid as it stands, with nobody's trails.
fn route(sim: &Sim, from: u32, to: u32) -> Option<(Vec<u32>, f32)> {
    match sim.nav().route(
        &sim.map().elevation,
        from as usize,
        to as usize,
        &|_| 0.0,
        2_000_000,
    ) {
        RouteResult::Found(r) => {
            let secs = r.seconds.last().copied().unwrap_or(0.0);
            Some((r.cells, secs))
        }
        _ => None,
    }
}

fn cell_of(sim: &Sim, p: (f32, f32)) -> u32 {
    civ_agents::population::cell_of(sim.map(), p) as u32
}

/// A ring of `radius_m` round the first settlement's hearth with one gate, facing away from a
/// walkable cell outside it that the hearth reaches: the enclosure (not yet worked), the hearth's
/// cell and the cell outside.
fn site(sim: &mut Sim, radius_m: f32) -> (Enclosure, u32, u32) {
    let s = &sim.land().settlements[0];
    let (settlement, hearth) = (s.id, s.hearth_m);
    let cell_m = sim.map().cell_size_m;
    let home = cell_of(sim, hearth);
    let far = radius_m + 5.0 * cell_m;
    let (bearing, outside) = (0..16)
        .map(|k| k as f32 * std::f32::consts::TAU / 16.0 + std::f32::consts::PI)
        .find_map(|t| {
            let c = cell_of(sim, (hearth.0 + far * t.cos(), hearth.1 + far * t.sin()));
            (sim.nav().walkable(c as usize) && route(sim, home, c).is_some()).then_some((t, c))
        })
        .expect("somewhere outside the line the hearth reaches");
    let sectors = ring(
        hearth,
        radius_m,
        20.0,
        1,
        bearing + std::f32::consts::PI,
        2.0,
        cell_m,
        sim.map().width,
        sim.map().height,
    );
    let id = sim.allocate_id_for_tests();
    let enclosure = Enclosure {
        id,
        owner: settlement,
        law: settlement,
        settlement,
        centre: hearth,
        radius_m,
        sectors,
        work_h: 0.0,
        shares: Vec::new(),
        crew: (0, 0),
        begun: sim.now(),
        finished: None,
    };
    (enclosure, home, outside)
}

/// Puts `enclosure` in the world in place of any there, and lays the walking grid again.
fn lay(sim: &mut Sim, enclosure: &Enclosure) {
    let land = sim.land_mut_for_tests();
    land.enclosures.list = vec![enclosure.clone()];
    land.enclosures.changed();
    sim.lay_decks_for_tests();
}

/// The cells the enclosures bar now.
fn barred(sim: &Sim) -> Vec<u32> {
    sim.land()
        .enclosures
        .barriers(0.05)
        .into_iter()
        .map(|b| b.0)
        .collect()
}

#[test]
fn a_standing_line_is_climbed_slowly_an_open_gate_is_walked_through_and_a_shut_one_is_not() {
    let mut sim = village(3);
    sim.advance_minutes(DAY).expect("lives");
    let radius = 8.0 * sim.map().cell_size_m;
    let (mut enclosure, home, outside) = site(&mut sim, radius);
    let (_, free) = route(&sim, home, outside).expect("the way out");

    // Not yet worked: nothing stands, nothing is barred.
    lay(&mut sim, &enclosure);
    assert!(barred(&sim).is_empty());
    let (_, unbuilt) = route(&sim, home, outside).expect("the way out");
    assert!(
        (unbuilt - free).abs() < 1e-3,
        "{unbuilt} s against {free} s"
    );

    // Whole, its gate open on the far side: the way out goes round through the gate.
    enclosure.work_h = enclosure.labour_h();
    assert!(enclosure.whole());
    lay(&mut sim, &enclosure);
    assert!(!barred(&sim).is_empty());
    let (_, open) = route(&sim, home, outside).expect("out through the gate");
    assert!(open > free, "round by the gate {open} s, straight {free} s");

    // Its gate shut: every way out is a climb, slower than the gate was.
    for s in &mut enclosure.sectors {
        if let Some(g) = &mut s.gate {
            g.open = false;
        }
    }
    let before = sim.nav().travel_revision();
    lay(&mut sim, &enclosure);
    assert_ne!(
        sim.nav().travel_revision(),
        before,
        "routes planned before are stale"
    );
    let (cells, shut) = route(&sim, home, outside).expect("over the line");
    assert!(
        shut > open,
        "over the palisade {shut} s, through the gate {open} s"
    );
    let bars = barred(&sim);
    assert!(cells.iter().any(|c| bars.contains(c)), "it climbs the line");

    // Rotted through, every sector is a gap again.
    let mut gapped = enclosure.clone();
    for s in &mut gapped.sectors {
        s.loss = 1.0;
    }
    lay(&mut sim, &gapped);
    let (_, through) = route(&sim, home, outside).expect("through the gaps");
    assert!(
        (through - free).abs() < 1e-3,
        "{through} s against {free} s"
    );

    lay(&mut sim, &enclosure);
    assert!(sim.land().enclosures.problems().is_empty());

    // The observer is sent each sector, with its ends, state and gate.
    let bytes = civ_sim::frames::earthworks::earthworks_response(&sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let sectors: Vec<_> = response
        .body_as_earthworks()
        .expect("earthworks")
        .works()
        .expect("works")
        .iter()
        .filter(|w| w.kind() == 5)
        .collect();
    assert_eq!(sectors.len(), enclosure.sectors.len());
    assert!(sectors.iter().all(|w| w.sector_state() == 1));
    let gate = sectors
        .iter()
        .find(|w| w.sector_gate() == 2)
        .expect("the shut gate");
    let i = gate.sector_index() as usize;
    assert_eq!(gate.sector_a().expect("a").x(), enclosure.sectors[i].a.0);
    assert!(gate.words().expect("words").contains("a gate, shut"));
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "ring").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(sim.land().enclosures, loaded.land().enclosures);
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    assert_eq!(sim.nav().travel_revision(), loaded.nav().travel_revision());
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}
