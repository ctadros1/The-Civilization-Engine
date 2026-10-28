//! The wire (M11a, third part; ADR-0036 §3–§4): lines of current and hookups are saved with the
//! land and load exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::wire::{Hookup, Line};
use civ_sim::{NewWorld, Sim, persist};

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

fn world() -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Wires".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
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

fn id(n: u64) -> PermanentId {
    PermanentId::from_raw(n).expect("id")
}

#[test]
fn lines_and_hookups_save_and_load_exactly() {
    let mut sim = world();
    let (station, home) = {
        let mut hh = sim.people().households.iter().map(|(_, h)| h.id);
        (hh.next().expect("a household"), hh.next().expect("another"))
    };
    let wires = &mut sim.land_mut_for_tests().wires;
    assert!(
        wires.lines.is_empty() && wires.hookups.is_empty(),
        "nobody lays any"
    );
    wires.lines.push(Line {
        id: id(9_100_001),
        station,
        parent: None,
        rating_kw: 6.0,
        length_m: 180.5,
        laid_m: 180.5,
        broken: false,
    });
    wires.lines.push(Line {
        id: id(9_100_002),
        station,
        parent: Some(id(9_100_001)),
        rating_kw: 2.0,
        length_m: 75.0,
        laid_m: 12.25,
        broken: true,
    });
    wires.hookups.push(Hookup {
        building: id(9_100_010),
        household: home,
        line: id(9_100_002),
        ask_kw: 0.3,
        cut_off: true,
    });
    let before = sim.land().wires.clone();
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "wires").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().wires, before);
    let (a, b) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    assert!(
        a.iter().any(|s| s.tag.to_string() == "wires"),
        "a wires section"
    );
}
