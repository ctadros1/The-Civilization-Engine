//! Wells (M6a slice AY, step three; ADR-0021 §1, §3): a household weighs one beside its home by
//! the walking it would save over its lining's life against the hours to dig and line it to the
//! depth it expects water at; its people dig it; it meets water where the water table stands
//! above its floor, or is dug deeper, or given up, and its depth becomes what others expect; its
//! household and its kin draw from it, and anyone short of water; what is drawn comes out of the
//! water table at the day's end; its lining rots until it is relined or falls in.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ChronicleKind;
use civ_agents::WellStep;
use civ_agents::population::WellWeighed;
use civ_agents::uses::Place;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::buildings::PlotUse;
use civ_land::wells::WellState;
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

/// The home of household `h`, metres.
fn home_of(sim: &Sim, h: PermanentId) -> (f32, f32) {
    sim.people()
        .households
        .iter()
        .find(|(_, x)| x.id == h)
        .expect("the household")
        .1
        .home
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
            if let WellWeighed::Begun { saved_h, labour_h } = weighed {
                assert!(saved_h > labour_h);
                let w = sim.land().wells.list.last().expect("begun");
                return (h, w.id);
            }
        }
        sim.advance_minutes(10 * DAY).expect("lives");
    }
    panic!("no household began a well");
}

/// The water table's head under well `well`, metres, and its patch.
fn head_under(sim: &Sim, well: PermanentId) -> (f64, usize) {
    let w = sim.land().wells.get(well).expect("the well");
    let p = sim.land().patches.of_cell(w.cell as usize, sim.map().width);
    (sim.land().water.heads[p], p)
}

/// Sets the water table under well `well` to stand `above` metres above its target floor.
fn set_water(sim: &mut Sim, well: PermanentId, above: f64) {
    let w = *sim.land().wells.get(well).expect("the well");
    let (_, p) = head_under(sim, well);
    let floor = f64::from(w.ground_m) - f64::from(w.target_m);
    sim.land_mut_for_tests().water.heads[p] = floor + above;
}

/// Hours to dig and line well `well` down to its target from where it is now.
fn hours_left(sim: &Sim, well: PermanentId) -> f64 {
    let w = sim.land().wells.get(well).expect("the well");
    let def = &sim.rules().catalog.wells[w.system];
    let h_per_m = def.h_per_m(sim.rules().people.digging.h_per_m3);
    f64::from(w.target_m - w.depth_m) * h_per_m
}

/// Digs and lines well `well` to its target now.
fn dig_out(sim: &mut Sim, well: PermanentId) {
    let hours = hours_left(sim, well) + 0.5;
    sim.with_ctx_for_tests(|pop, ctx| pop.well_work_for_tests(ctx, well, hours));
}

/// Chronicle entries about wells: step and sentence.
fn well_entries(sim: &Sim) -> Vec<(f64, String)> {
    sim.people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::Well)
        .map(|e| (e.number, e.name.clone()))
        .collect()
}

/// A village of world 6 with one household's well dug and open, its water table standing two
/// metres above its floor: the household and its well.
fn an_open_well() -> (Sim, PermanentId, PermanentId) {
    let mut sim = village(6);
    let (h, well) = a_well_begun(&mut sim);
    set_water(&mut sim, well, 2.0);
    dig_out(&mut sim, well);
    assert!(sim.land().wells.get(well).expect("the well").is_open());
    (sim, h, well)
}

#[test]
fn a_household_weighs_a_well_by_the_walk_it_would_save_and_digs_it_beside_its_home() {
    let mut sim = village(6);
    let (h, well) = a_well_begun(&mut sim);
    let w = *sim.land().wells.get(well).expect("the well");
    let def = sim.rules().catalog.wells[w.system].clone();
    assert_eq!(def.id, "core:well/timber_lined");
    // Beside its home: within eight metres of its home plot, on ground nobody else holds.
    let plot = sim
        .land()
        .plots
        .iter()
        .find(|p| p.household == h && p.use_ == PlotUse::Dwelling)
        .expect("a home plot")
        .rect;
    assert!(w.rect.near(&plot, 800 + 150), "{:?} {:?}", w.rect, plot);
    assert!(!w.rect.near(&plot, 0));
    assert!(sim.land().plots.iter().all(|p| !p.rect.near(&w.rect, 0)));
    assert!(sim.land().fields.iter().all(|f| !f.rect.near(&w.rect, 0)));
    // A metre below where its people expect the water, no deeper than its diggers go.
    assert!(
        w.target_m >= def.water_m as f32 && w.target_m <= def.max_depth_m as f32,
        "{}",
        w.target_m
    );
    assert_eq!(w.depth_m, 0.0);
    assert!(w.worth > 0.0, "{}", w.worth);
    // A second weighing begins nothing more while it is being dug.
    let again = sim.with_ctx_for_tests(|pop, ctx| pop.review_well_for_tests(ctx, h));
    assert_eq!(again, WellWeighed::Busy);
    assert_eq!(
        sim.land()
            .wells
            .list
            .iter()
            .filter(|w| w.household == h)
            .count(),
        1
    );
    // Its people choose to dig it, a session at a time, no more of them a day than its shaft
    // has room for.
    for _ in 0..60 {
        sim.advance_minutes(DAY).expect("lives");
        let w = sim.land().wells.get(well).expect("the well");
        assert!(u32::from(w.crew.1) <= def.crew, "{:?}", w.crew);
        if w.work_h > 0.0 {
            break;
        }
    }
    let w = sim.land().wells.get(well).expect("the well");
    assert!(w.work_h > 0.0 && w.depth_m > 0.0, "{w:?}");
    assert!(w.skill_h > 0.0 || w.is_open());
}

#[test]
fn a_shaft_meets_water_where_the_water_table_stands_above_its_floor() {
    let mut sim = village(6);
    let (_, well) = a_well_begun(&mut sim);
    set_water(&mut sim, well, 0.5);
    // Half way down, nothing yet.
    let half = hours_left(&sim, well) / 2.0;
    sim.with_ctx_for_tests(|pop, ctx| pop.well_work_for_tests(ctx, well, half));
    let w = *sim.land().wells.get(well).expect("the well");
    assert_eq!(w.state, WellState::Digging);
    assert!((w.depth_m - w.target_m / 2.0).abs() < 0.01, "{w:?}");
    // At its target it meets the water and opens, its lining's quality drawn from its diggers'
    // skill, the water standing in it at the water table.
    dig_out(&mut sim, well);
    let w = *sim.land().wells.get(well).expect("the well");
    assert!(matches!(w.state, WellState::Open { .. }), "{:?}", w.state);
    assert_eq!(w.depth_m, w.target_m);
    assert!(w.quality > 0.0 && w.quality <= 1.0);
    let (head, _) = head_under(&sim, well);
    assert!((w.level_m - head).abs() < 1e-9);
    assert!(w.floor_m() < head);
    let entries = well_entries(&sim);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0].0, f64::from(WellStep::Opened as u8));
    assert!(entries[0].1.contains("met water"), "{}", entries[0].1);
    assert!(sim.land().wells.problems(1).is_empty());
}

#[test]
fn a_dry_shaft_is_dug_deeper_and_at_the_deepest_given_up_and_others_expect_water_below_it() {
    let mut sim = village(6);
    let (h, well) = a_well_begun(&mut sim);
    let max = sim.rules().catalog.wells[0].max_depth_m as f32;
    // The water table far below anything its diggers would reach.
    let (_, p) = head_under(&sim, well);
    let ground = sim.land().wells.get(well).expect("the well").ground_m;
    sim.land_mut_for_tests().water.heads[p] = f64::from(ground) - 40.0;
    // At its target it meets nothing and is dug a metre deeper, and so on to the deepest.
    let first = sim.land().wells.get(well).expect("the well").target_m;
    dig_out(&mut sim, well);
    let w = *sim.land().wells.get(well).expect("the well");
    if first + 1.0 <= max {
        assert_eq!(w.state, WellState::Digging);
        assert!((w.target_m - (first + 1.0)).abs() < 1e-4, "{w:?}");
    }
    for _ in 0..12 {
        if sim.land().wells.get(well).expect("the well").state != WellState::Digging {
            break;
        }
        dig_out(&mut sim, well);
    }
    let w = *sim.land().wells.get(well).expect("the well");
    assert!(matches!(w.state, WellState::GivenUp { .. }), "{w:?}");
    assert!(w.depth_m <= max && w.depth_m > max - 1.0, "{}", w.depth_m);
    let entries = well_entries(&sim);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0].0, f64::from(WellStep::GivenUp as u8));
    assert!(entries[0].1.contains("no water"), "{}", entries[0].1);
    // A given-up shaft is no source.
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, h, true))
            .is_none_or(|o| o.well != Some(well))
    );
    // Its depth is what the village now knows: anyone weighing a well expects water below its
    // floor, as deep again as the column its diggers wanted.
    let dry_floor = w.floor_m();
    let water_m = sim.rules().catalog.wells[0].water_m;
    let before = sim.land().wells.list.len();
    for other in households(&sim) {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_well_for_tests(ctx, other));
    }
    for w in &sim.land().wells.list[before..] {
        assert!(
            w.floor_m() <= dry_floor - 2.0 * water_m + 1e-3,
            "a well to {} below a dry floor at {dry_floor}",
            w.floor_m()
        );
    }
}

#[test]
fn people_draw_from_their_own_well_and_the_water_table_gives_what_they_drew() {
    let (mut sim, h, well) = an_open_well();
    let o = sim
        .with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, h, false))
        .expect("a source");
    assert_eq!(o.well, Some(well));
    // With no water at home, its people fetch from it through the morning.
    {
        let pop = sim.people_mut_for_tests();
        for (_, x) in pop.households.iter_mut() {
            if x.id == h {
                x.water_l = 0.0;
            }
        }
    }
    sim.advance_minutes(4 * 60).expect("lives");
    let cell = sim.land().wells.get(well).expect("the well").cell;
    let draws = sim
        .people()
        .uses
        .today
        .iter()
        .filter(|u| u.place == Place::Source(cell) && u.household == h)
        .count();
    assert!(draws > 0, "nobody drew at the well");
    let w = *sim.land().wells.get(well).expect("the well");
    let carry = sim.rules().people.household.carry_water_l;
    assert!(
        (w.drawn_l - draws as f64 * carry).abs() < 1e-6,
        "{} L for {draws} loads",
        w.drawn_l
    );
    // What it uses follows the walk to it: a few steps.
    let x = sim
        .people()
        .households
        .iter()
        .find(|(_, x)| x.id == h)
        .expect("the household")
        .1;
    assert_eq!(x.water_use_l, 20.0);
    // At the day's end what was drawn comes out of the water table under it: a twin in which
    // nothing was drawn has the water table that much higher there, and the same elsewhere.
    let mut twin = reloaded(&mut sim);
    if let Some(w) = twin.land_mut_for_tests().wells.get_mut(well) {
        w.drawn_l = 0.0;
    }
    to_morning(&mut sim, 1);
    to_morning(&mut twin, 1);
    let (head, p) = head_under(&sim, well);
    let (twin_head, _) = head_under(&twin, well);
    let aq = &sim.land().water.aquifer;
    let expect = if aq.stage[p].is_some() {
        0.0
    } else {
        w.drawn_l / 1000.0 / aq.storage_m2[p]
    };
    assert!(expect >= 0.0);
    assert!(
        ((twin_head - head) - expect).abs() < 1e-9,
        "{} against {expect}",
        twin_head - head
    );
    for (q, (a, b)) in sim
        .land()
        .water
        .heads
        .iter()
        .zip(&twin.land().water.heads)
        .enumerate()
    {
        if q != p {
            assert_eq!(a, b, "patch {q}");
        }
    }
    assert_eq!(sim.land().wells.get(well).expect("the well").drawn_l, 0.0);
}

#[test]
fn a_well_serves_its_household_its_kin_and_anyone_short_of_water() {
    let (mut sim, h, well) = an_open_well();
    let at = sim
        .land()
        .wells
        .get(well)
        .expect("the well")
        .rect
        .centre_m();
    let others: Vec<PermanentId> = households(&sim).into_iter().filter(|&o| o != h).collect();
    assert!(others.len() >= 2, "{others:?}");
    // Nobody else's people draw there unless short of water; short, they draw at the nearer of
    // it and their own water.
    for &o in &others {
        let not_short =
            sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, o, false));
        assert!(not_short.is_none_or(|w| w.well != Some(well)), "{o}");
        let short = sim
            .with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, o, true))
            .expect("a source");
        if short.well != Some(well) {
            let home = home_of(&sim, o);
            let wear = &sim.land().wear;
            if let Some(s) = sim
                .nav()
                .segment_seconds(&sim.map().elevation, home, at, &|c| wear.factor(c))
            {
                assert!(short.walk_min * 60.0 <= f64::from(s) + 1e-6, "{o}");
            }
        }
    }
    // A household for whom the well is nearer than its own water, one of whose people is made a
    // child of the well's household's, is kin: its people draw there now, not short of water.
    let kin = others
        .iter()
        .copied()
        .find(|&o| {
            sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, o, true))
                .is_some_and(|w| w.well == Some(well))
        })
        .expect("a neighbour nearer the well than its own water");
    let first_member = |sim: &Sim, of: PermanentId| {
        sim.people()
            .households
            .iter()
            .find(|(_, x)| x.id == of)
            .expect("the household")
            .1
            .members[0]
    };
    let (parent, child) = (first_member(&sim, h), first_member(&sim, kin));
    sim.people_mut_for_tests()
        .records
        .get_mut(&child)
        .expect("a record")
        .mother = Some(parent);
    let own = sim
        .with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, kin, false))
        .expect("a source");
    assert_eq!(own.well, Some(well), "kin draw at the well");
}

#[test]
fn rot_takes_a_lining_until_its_household_relines_it_or_it_falls_in() {
    let (mut sim, h, well) = an_open_well();
    // Sound while rot has taken less than half of what its lining had.
    {
        let w = sim
            .land_mut_for_tests()
            .wells
            .get_mut(well)
            .expect("the well");
        w.quality = 0.6;
        w.loss = 0.29;
    }
    let weighed = sim.with_ctx_for_tests(|pop, ctx| pop.review_well_for_tests(ctx, h));
    assert_eq!(weighed, WellWeighed::Sound);
    // Past half, its household weighs relining it, and does: the lining's hours, metre by metre.
    sim.land_mut_for_tests()
        .wells
        .get_mut(well)
        .expect("the well")
        .loss = 0.31;
    let weighed = sim.with_ctx_for_tests(|pop, ctx| pop.review_well_for_tests(ctx, h));
    assert!(matches!(weighed, WellWeighed::Reline { .. }), "{weighed:?}");
    let w = *sim.land().wells.get(well).expect("the well");
    let lining = sim.rules().catalog.wells[w.system].lining_h_per_m;
    assert!((f64::from(w.mend_h) - f64::from(w.depth_m) * lining).abs() < 1e-3);
    assert!(w.is_open(), "drawn from while it is relined");
    let mend = f64::from(w.mend_h);
    sim.with_ctx_for_tests(|pop, ctx| pop.well_work_for_tests(ctx, well, mend + 0.1));
    let w = *sim.land().wells.get(well).expect("the well");
    assert_eq!((w.mend_h, w.loss), (0.0, 0.0));
    assert!(w.quality > 0.0 && w.quality <= 1.0);
    // Left to rot through, it falls in on the first of a month.
    sim.land_mut_for_tests()
        .wells
        .get_mut(well)
        .expect("the well")
        .loss = w.quality - 1e-4;
    for _ in 0..32 {
        sim.advance_minutes(DAY).expect("lives");
        if sim.now().date().day == 1 {
            break;
        }
    }
    let w = *sim.land().wells.get(well).expect("the well");
    assert!(matches!(w.state, WellState::FellIn { .. }), "{:?}", w.state);
    let entries = well_entries(&sim);
    assert_eq!(
        entries.last().map(|e| e.0),
        Some(f64::from(WellStep::FellIn as u8))
    );
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, h, true))
            .is_none_or(|o| o.well != Some(well))
    );
}

/// A save of `sim` loaded again, checked to be the same section for section.
fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "well").expect("saves");
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
fn wells_dug_and_drawn_from_save_load_and_go_on_alike() {
    let (mut sim, h, _) = an_open_well();
    // Another household's well being dug, and the open one drawn from.
    for o in households(&sim) {
        if o != h
            && matches!(
                sim.with_ctx_for_tests(|pop, ctx| pop.review_well_for_tests(ctx, o)),
                WellWeighed::Begun { .. }
            )
        {
            break;
        }
    }
    sim.advance_minutes(3 * 60).expect("lives");
    assert!(
        sim.land()
            .wells
            .list
            .iter()
            .any(|w| w.drawn_l > 0.0 || w.work_h > 0.0)
    );
    let mut loaded = reloaded(&mut sim);
    sim.advance_minutes(DAY).expect("advances");
    loaded.advance_minutes(DAY).expect("advances");
    same(&sim, &loaded);
}
