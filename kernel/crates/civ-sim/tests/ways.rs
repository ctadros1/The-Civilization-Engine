//! Ways as records (M8b slice CM; ADR-0029 §1): a way someone worked on holds its ground's wear
//! at its condition before each month's survey, so its cells walk as worn trail; it keeps its
//! identity through surveys and loads; its walkers are counted; and its rank is derived at the
//! survey from what it joins, its build and its use.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
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

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Ways".to_owned(),
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

/// The dry cells in a straight run from the hearth's cell, east or west (whichever runs longer
/// on dry land), at most `n` of them.
fn line_from_hearth(sim: &Sim, n: u32) -> Vec<u32> {
    let map = sim.map();
    let (x, y) = sim.land().settlements[0].hearth_m;
    let (cx, cy) = ((x / map.cell_size_m) as u32, (y / map.cell_size_m) as u32);
    let run = |step: i64| -> Vec<u32> {
        (0..i64::from(n))
            .map(|k| i64::from(cx) + step * k)
            .take_while(|&c| c >= 0 && c < i64::from(map.width))
            .map(|c| cy * map.width + c as u32)
            .take_while(|&cell| map.water[cell as usize] == WATER_LAND)
            .collect()
    };
    let (east, west) = (run(1), run(-1));
    if east.len() >= west.len() { east } else { west }
}

/// Lays a way along `cells`, kept by the first household since `kept`.
fn lay(sim: &mut Sim, cells: Vec<u32>, usable_m: f32, kept: i64) -> civ_core::PermanentId {
    let household = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h.id)
        .min()
        .expect("a household");
    let id = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    sim.land_mut_for_tests().ways.add(Way {
        id,
        begun: today,
        segments: vec![Segment {
            cells,
            corridor_m: 6.0,
            built_m: usable_m,
            usable_m,
            surface: Surface::Cleared,
            keeper: Keeper::Household(household),
            kept,
        }],
        modes: WALK,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
    id
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "ways").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    assert_eq!(sim.land().ways, loaded.land().ways);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

#[test]
fn a_kept_way_holds_its_ground_counts_its_walkers_and_is_ranked_at_the_survey() {
    let mut sim = village(1);
    let cells = line_from_hearth(&sim, 40);
    assert!(cells.len() >= 20, "a run of dry land from the hearth");
    let far = *cells.last().expect("cells");
    let before = sim.land().wear.factor(far as usize);
    let today = sim.now().day_index();
    let kept = lay(&mut sim, cells.clone(), 1.5, today);
    // Another, untended for two half-lives, along the first stretch only.
    let untended = lay(&mut sim, cells[..5].to_vec(), 1.0, today - 240);
    let revision = sim.land().ways.revision();
    // Past the next month's survey.
    sim.advance_minutes(35 * DAY).expect("advances");
    let land = sim.land();
    let surveyed = land.wear.surveyed();
    assert!(surveyed > today, "a survey since the way was laid");
    let held = 0.5f32.powf((surveyed - today) as f32 / 120.0);
    for &c in &cells {
        assert!(
            land.wear.factor(c as usize) >= held - 0.01,
            "cell {c} walks at its way's condition: {} < {held}",
            land.wear.factor(c as usize)
        );
    }
    assert!(
        before < held - 0.1,
        "the far end was not worn so far before: {before}"
    );
    let ways = &land.ways;
    assert!(ways.revision() > revision, "the survey is a new revision");
    let (a, b) = (&ways.list[0], &ways.list[1]);
    assert_eq!((a.id, b.id), (kept, untended), "identities kept");
    // Both ends lie by the one settlement: a lane or street by its build and use, never a way
    // between settlements.
    assert_ne!(a.rank, Rank::Between);
    assert!(a.trips > 0.0, "people walked along the way by the hearth");
    assert!(b.segments[0].condition(sim.now().day_index(), &sim.rules().land.paths) < 0.25);
    saves_and_goes_on_alike(&mut sim, 2 * DAY);
}

#[test]
fn a_passing_place_left_unkept_grows_over_at_the_turn_of_the_year_and_a_kept_one_does_not() {
    // Backlog D, fourteenth item: a cleared margin left unkept the soil's `regrown_years` whole
    // years has grown over at the turn of the year, its usable width back to a trail's tread.
    let mut sim = village(1);
    let years = sim.rules().land.soil.regrown_years;
    assert!(years > 0, "the core's wild cover grows back");
    let cells = line_from_hearth(&sim, 3);
    assert!(cells.len() >= 3, "a run of dry land from the hearth");
    let today = sim.now().day_index();
    let year = civ_land::calendar_year(today);
    // One last kept in the year `years` before this one, so that its `years` whole years unkept
    // end as this year does; another this year.
    let unkept = (i64::from(year) - i64::from(years) - 1) * 365 + 100;
    let place = |sim: &mut Sim, cell: u32, kept: i64| {
        let id = sim.allocate_id_for_tests();
        sim.land_mut_for_tests().ways.add(Way {
            id,
            begun: kept,
            segments: vec![Segment {
                cells: vec![cell],
                corridor_m: 2.0,
                built_m: 2.0,
                usable_m: 2.0,
                surface: Surface::Cleared,
                keeper: Keeper::Nobody,
                kept,
            }],
            modes: WALK | civ_land::ways::CART,
            trips: 0.0,
            trips_day: today,
            rank: Rank::Lane,
            track: None,
        });
        id
    };
    let old = place(&mut sim, cells[1], unkept);
    let kept = place(&mut sim, cells[2], today);
    let usable = |sim: &Sim, id| {
        let w = sim
            .land()
            .ways
            .list
            .iter()
            .find(|w| w.id == id)
            .expect("the passing place");
        (w.segments[0].usable_m, w.segments[0].built_m)
    };
    // Until the year turns, both stand cleared.
    let to_turn = 365 - today.rem_euclid(365);
    sim.advance_minutes((to_turn - 1) * DAY).expect("advances");
    assert_eq!(usable(&sim, old), (2.0, 2.0));
    // Past the turn, the one left unkept has grown over to the tread; its built width stands.
    sim.advance_minutes(2 * DAY).expect("advances");
    assert_eq!(civ_land::calendar_year(sim.now().day_index()), year + 1);
    assert_eq!(usable(&sim, old), (1.2, 2.0));
    assert_eq!(usable(&sim, kept), (2.0, 2.0));
    saves_and_goes_on_alike(&mut sim, 60);
}
