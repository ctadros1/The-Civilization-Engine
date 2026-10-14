//! Irrigation ditches (M6b slice BG; ADR-0022 §11): a finished, open ditch waters its field from
//! the river, the water it takes is debited from its reach and every reach below, what returns
//! comes back below the intake, every cubic metre keeps its destination, and saves keep it all.
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
use civ_land::RectCm;
use civ_land::ditches::DitchSegment;
use civ_land::earth::{EARTH_VERSION, EarthKind, Earthwork};
use civ_land::fields::FieldStage;
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};
use std::{path::Path, sync::OnceLock};

const DAY: i64 = 1440;

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

/// A lived village of seed 3, forty days on: its fields are sown.
fn village() -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Ditchside".to_owned(),
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
    .expect("generates");
    sim.advance_minutes(40 * DAY).expect("advances");
    sim
}

/// The first sown field, the reach nearest it and the point on that reach nearest the field.
fn sown_field(sim: &Sim) -> (PermanentId, u32, [f64; 2], [f64; 2]) {
    let map = sim.map();
    let cell = f64::from(map.cell_size_m);
    let field = sim
        .land()
        .fields
        .iter()
        .filter(|f| f.stage == FieldStage::Sown)
        .min_by_key(|f| f.id)
        .expect("a sown field");
    let (fx, fy) = field.rect.centre_m();
    let (fx, fy) = (f64::from(fx), f64::from(fy));
    let (reach, at) = map
        .reaches
        .iter()
        .enumerate()
        .flat_map(|(r, reach)| {
            reach.cells.iter().map(move |&c| {
                let x = (f64::from(c % map.width) + 0.5) * cell;
                let y = (f64::from(c / map.width) + 0.5) * cell;
                (r as u32, [x, y])
            })
        })
        .min_by(|a, b| {
            let d = |p: &[f64; 2]| (p[0] - fx).hypot(p[1] - fy);
            d(&a.1).total_cmp(&d(&b.1))
        })
        .expect("a reach");
    (field.id, reach, at, [fx, fy])
}

/// Lays a finished ditch, its intake open or shut, from beside the reach to the field.
fn dig(sim: &mut Sim, open: bool) -> DitchSegment {
    let (field, reach, intake, to) = sown_field(sim);
    let work = sim.with_ctx_for_tests(|_, ctx| ctx.ids.allocate());
    let household = sim
        .land()
        .fields
        .iter()
        .find(|f| f.id == field)
        .expect("field")
        .household;
    let length = (intake[0] - to[0]).hypot(intake[1] - to[1]);
    let ditch = DitchSegment {
        work,
        reach,
        field,
        a: intake,
        b: to,
        bottom_m: 0.3,
        depth_m: 0.4,
        side_run: 1.5,
        drop_m: 0.001 * length,
        weeds: 0.0,
        silt_m: 0.0,
        open,
        taken_m3: 0.0,
        kept_m3: 0.0,
        seeped_m3: 0.0,
        returned_m3: 0.0,
    };
    let (x0, y0) = (intake[0].min(to[0]), intake[1].min(to[1]));
    let (x1, y1) = (intake[0].max(to[0]), intake[1].max(to[1]));
    let land = sim.land_mut_for_tests();
    land.earthworks.push(Earthwork {
        id: work,
        kind: EarthKind::Ditch,
        rect: RectCm {
            x: (x0 * 100.0) as i32,
            y: (y0 * 100.0) as i32,
            w: (((x1 - x0) * 100.0) as i32).max(1),
            h: (((y1 - y0) * 100.0) as i32).max(1),
        },
        level_cm: 0,
        side_run_cm: 150,
        plot: None,
        deposit: None,
        heap: None,
        household,
        cut_m3: ditch.cut_m3() as f32,
        done: 1.0,
        version: EARTH_VERSION,
        begun: SimTime::ZERO,
    });
    land.ditches.push(ditch);
    ditch
}

fn problems(sim: &Sim) -> Vec<String> {
    sim.land()
        .problems(sim.map(), sim.rules().land.habitats.len(), u64::MAX)
}

#[test]
fn an_open_ditch_waters_its_field_from_the_river_and_every_cubic_metre_keeps_its_destination() {
    // The same village with a finished ditch to a sown field, and without.
    let (mut with, mut without) = (village(), village());
    let ditch = dig(&mut with, false);
    assert!(problems(&with).is_empty(), "{:?}", problems(&with));
    assert!(ditch.capacity_m3s(&with.rules().land.ditches) > 0.0);
    with.advance_minutes(10 * DAY).expect("advances");
    without.advance_minutes(10 * DAY).expect("advances");
    // Days with the field's root zone dry in both: at midnight its people open the ditch, and
    // its field gains what the ditch keeps there.
    for sim in [&mut with, &mut without] {
        let f = sim
            .land_mut_for_tests()
            .fields
            .iter_mut()
            .find(|f| f.id == ditch.field)
            .expect("the field");
        assert_eq!(f.stage, FieldStage::Sown);
        f.water_mm = 0.0;
    }
    // (Its people open it at midnight, after that day's water; it waters from the next.)
    with.advance_minutes(2 * DAY).expect("advances");
    without.advance_minutes(2 * DAY).expect("advances");
    let field = |sim: &Sim| {
        sim.land()
            .fields
            .iter()
            .find(|f| f.id == ditch.field)
            .cloned()
            .expect("the field")
    };
    let (wet, dry) = (field(&with), field(&without));
    assert!(
        wet.got_mm + wet.water_mm > dry.got_mm + dry.water_mm,
        "watered: got {} water {}; without: got {} water {}",
        wet.got_mm,
        wet.water_mm,
        dry.got_mm,
        dry.water_mm
    );
    let d = with.land().ditches[0];
    assert!(d.taken_m3 > 0.0 && d.kept_m3 > 0.0, "{d:?}");
    assert!((d.kept_m3 + d.seeped_m3 + d.returned_m3 - d.taken_m3).abs() < 1e-6 * d.taken_m3);
    // Today's taking is debited from its reach and every reach below it; what returned comes
    // back from the reach below the intake on.
    let map = with.map();
    let flows = &with.land().ditch_flows;
    let &(r, take) = flows.first().expect("water taken today");
    assert_eq!(r, ditch.reach);
    assert!(take < 0.0);
    let back = flows.get(1).map_or(0.0, |f| f.1);
    // The same day's flows without the ditch's takings and returns.
    let untaken = {
        let mut land = with.land().clone();
        land.ditch_flows.clear();
        land.river_flows(map, &with.rules().land, with.land().weather.today.day)
    };
    let mut at = Some(r as usize);
    let mut first = true;
    while let Some(i) = at {
        let gap = with.land().reach_discharges[i] - untaken[i];
        let want = if first { take } else { take + back };
        assert!(
            (gap - want).abs() < 1e-4 + 1e-3 * take.abs(),
            "reach {i}: {gap} against {want}"
        );
        first = false;
        at = map.reaches[i].downstream.map(|x| x as usize);
    }
    assert!(problems(&with).is_empty(), "{:?}", problems(&with));
    // Saved and loaded, it is as it was, and lives on the same.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
    let saved = persist::save(&mut with, &saves, SaveKind::Manual, "ditch").expect("save");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    for (a, b) in persist::encode_sections(&with)
        .iter()
        .zip(persist::encode_sections(&loaded))
    {
        assert!(a.bytes == b.bytes, "section `{}` differs", a.tag);
    }
    with.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    assert_eq!(with.land().ditches, loaded.land().ditches);
    // A save of schema 104 has no ditches; one that claims them is refused.
    let reader =
        commons_persist::SnapshotReader::open_file(&saved.path, commons_persist::Limits::default())
            .expect("reader");
    let mut info = reader.info().clone();
    info.schema_version = persist::SCHEMA_V104;
    let legacy = commons_persist::publish_file(
        dir.path(),
        "legacy.tcesave",
        &info,
        &persist::encode_sections(&loaded),
        commons_persist::DEFAULT_ZSTD_LEVEL,
    )
    .expect("publish");
    assert!(persist::load(&legacy.path, content()).is_err());
}

#[test]
fn a_household_whose_crop_thirsted_digs_a_ditch_to_its_field_and_opens_it_in_a_dry_spell() {
    let mut sim = village();
    // A field whose last crop got half the water it needed, as its people saw it thirst.
    let (field, household) = {
        let f = sim
            .land()
            .fields
            .iter()
            .filter(|f| f.stage == FieldStage::Sown)
            .min_by_key(|f| f.id)
            .expect("a field");
        (f.id, f.household)
    };
    let dry = |sim: &mut Sim| {
        let f = sim
            .land_mut_for_tests()
            .fields
            .iter_mut()
            .find(|f| f.id == field)
            .expect("the field");
        f.stage = FieldStage::Reaped;
        (f.need_mm, f.got_mm) = (300.0, 150.0);
    };
    dry(&mut sim);
    // A field that got all it needed is not weighed.
    {
        let f = sim
            .land_mut_for_tests()
            .fields
            .iter_mut()
            .find(|f| f.id == field)
            .expect("the field");
        f.got_mm = 300.0;
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.review_ditch_for_tests(ctx, household));
    assert!(sim.land().ditches.is_empty(), "no thirst, no ditch");
    dry(&mut sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.review_ditch_for_tests(ctx, household));
    let ditch = *sim.land().ditches.first().expect("a ditch laid out");
    assert_eq!(ditch.field, field);
    let rules = sim.rules().clone();
    let dp = &rules.people.ditches;
    assert!(ditch.length_m() <= dp.max_length_m);
    assert!(ditch.drop_m / ditch.length_m() >= rules.land.ditches.min_gradient);
    assert!(ditch.capacity_m3s(&rules.land.ditches) > 0.0);
    let worth = sim
        .people()
        .household(household)
        .expect("the household")
        .ditch_worth;
    assert!(worth > 0.0);
    // Its gain over its years exceeds its digging.
    assert!(f64::from(worth) * dp.life_years > 1.0, "{worth}");
    let work = |sim: &Sim| {
        *sim.land()
            .earthworks
            .iter()
            .find(|e| e.id == ditch.work)
            .expect("its earthwork")
    };
    assert_eq!((work(&sim).done, work(&sim).kind), (0.0, EarthKind::Ditch));
    assert!(problems(&sim).is_empty(), "{:?}", problems(&sim));
    // Weighed again while it is being dug, nothing more is laid out.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_ditch_for_tests(ctx, household));
    assert_eq!(sim.land().ditches.len(), 1);
    // Its people may dig it: someone of the household sees it, its work and its worth.
    sim.advance_minutes(60).expect("advances");
    let digger = sim
        .people()
        .household(household)
        .expect("the household")
        .members[0];
    let facts = sim
        .with_ctx_for_tests(|pop, ctx| pop.ditch_facts_for_tests(ctx, digger))
        .expect("a ditch to dig");
    assert_eq!(facts.ditch, ditch.work);
    let total = dp.dig_hours(ditch.length_m());
    assert!((facts.left_h - total).abs() < 1e-6 * total, "{facts:?}");
    assert!((facts.saves_per_hour - f64::from(worth)).abs() < 1e-6);
    // Eight hours dig a cubic metre; the rest finishes it, and that is told.
    sim.with_ctx_for_tests(|pop, ctx| pop.ditch_work_by_for_tests(ctx, digger, ditch.work, 8.0));
    assert!((f64::from(work(&sim).cut_m3) - 1.0).abs() < 1e-4);
    assert!(problems(&sim).is_empty(), "{:?}", problems(&sim));
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.ditch_work_by_for_tests(ctx, digger, ditch.work, total * 2.0)
    });
    assert_eq!(work(&sim).done, 1.0);
    assert!((f64::from(work(&sim).cut_m3) - ditch.cut_m3()).abs() < 1e-3);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.name.contains("finished digging a ditch"))
    );
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.ditch_facts_for_tests(ctx, digger))
            .is_none()
    );
    assert!(problems(&sim).is_empty(), "{:?}", problems(&sim));
    // Sown again with its root zone dry, the ditch is opened at midnight and waters it.
    {
        let f = sim
            .land_mut_for_tests()
            .fields
            .iter_mut()
            .find(|f| f.id == field)
            .expect("the field");
        f.stage = FieldStage::Sown;
        f.water_mm = 0.0;
    }
    sim.advance_minutes(2 * DAY).expect("advances");
    let d = sim.land().ditches[0];
    assert!(d.taken_m3 > 0.0 && d.kept_m3 > 0.0, "{d:?}");
    // It saves and loads exactly.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "dug").expect("save");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    for (a, b) in persist::encode_sections(&sim)
        .iter()
        .zip(persist::encode_sections(&loaded))
    {
        assert!(a.bytes == b.bytes, "section `{}` differs", a.tag);
    }
}
