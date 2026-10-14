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
    let (mut open, mut shut) = (village(), village());
    let ditch = dig(&mut open, true);
    assert_eq!(dig(&mut shut, false).field, ditch.field);
    assert!(problems(&open).is_empty(), "{:?}", problems(&open));
    assert!(ditch.capacity_m3s(&open.rules().land.ditches) > 0.0);
    // Ten days: the open ditch's field holds or has used more water than the shut one's.
    open.advance_minutes(10 * DAY).expect("advances");
    shut.advance_minutes(10 * DAY).expect("advances");
    let field = |sim: &Sim| {
        sim.land()
            .fields
            .iter()
            .find(|f| f.id == ditch.field)
            .cloned()
            .expect("the field")
    };
    let (wet, dry) = (field(&open), field(&shut));
    assert!(wet.stage == FieldStage::Sown && dry.stage == FieldStage::Sown);
    assert!(
        wet.got_mm + wet.water_mm > dry.got_mm + dry.water_mm,
        "watered: got {} water {}; shut: got {} water {}",
        wet.got_mm,
        wet.water_mm,
        dry.got_mm,
        dry.water_mm
    );
    let d = open.land().ditches[0];
    assert!(d.taken_m3 > 0.0 && d.kept_m3 > 0.0, "{d:?}");
    assert!((d.kept_m3 + d.seeped_m3 + d.returned_m3 - d.taken_m3).abs() < 1e-6 * d.taken_m3);
    assert_eq!(shut.land().ditches[0].taken_m3, 0.0);
    // A day with the field's root zone dry in both: the open ditch takes, and its taking is
    // debited from its reach and every reach below it; what returned comes back from the reach
    // below the intake on.
    for sim in [&mut open, &mut shut] {
        let f = sim
            .land_mut_for_tests()
            .fields
            .iter_mut()
            .find(|f| f.id == ditch.field)
            .expect("the field");
        f.water_mm = 0.0;
    }
    open.advance_minutes(DAY).expect("advances");
    shut.advance_minutes(DAY).expect("advances");
    let map = open.map();
    let flows = &open.land().ditch_flows;
    let &(r, take) = flows.first().expect("water taken today");
    assert_eq!(r, ditch.reach);
    assert!(take < 0.0);
    let back = flows.get(1).map_or(0.0, |f| f.1);
    // The same day's flows without the ditch's takings and returns. (Against the shut twin the
    // flows also differ by what the ditch's seepage has since given the river through the
    // ground.)
    let untaken = {
        let mut land = open.land().clone();
        land.ditch_flows.clear();
        land.river_flows(map, &open.rules().land, open.land().weather.today.day)
    };
    let mut at = Some(r as usize);
    let mut first = true;
    while let Some(i) = at {
        let gap = open.land().reach_discharges[i] - untaken[i];
        let want = if first { take } else { take + back };
        assert!(
            (gap - want).abs() < 1e-4 + 1e-3 * take.abs(),
            "reach {i}: {gap} against {want}"
        );
        first = false;
        at = map.reaches[i].downstream.map(|x| x as usize);
    }
    assert!(problems(&open).is_empty(), "{:?}", problems(&open));
    // Saved and loaded, it is as it was, and lives on the same.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
    let saved = persist::save(&mut open, &saves, SaveKind::Manual, "ditch").expect("save");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    for (a, b) in persist::encode_sections(&open)
        .iter()
        .zip(persist::encode_sections(&loaded))
    {
        assert!(a.bytes == b.bytes, "section `{}` differs", a.tag);
    }
    open.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    assert_eq!(open.land().ditches, loaded.land().ditches);
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
