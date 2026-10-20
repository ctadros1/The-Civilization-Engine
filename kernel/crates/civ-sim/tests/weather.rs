//! The weather in the chronicle (M3c slice U, ADR-0012): what stood out in a month, a winter or a
//! year against what it usually brings is noted on the first of the next month, as the weather's
//! record says, whoever lives in the world, and kept across a save.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::history::{ChronicleKind, Span};
use civ_content::ContentRegistry;
use civ_core::time::MONTH_LENGTHS;
use civ_land::weather::{WeatherNote, stood_out};
use civ_sim::persist;
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind};

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

fn world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Weather".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

#[test]
fn what_stood_out_in_the_weather_is_noted_on_the_first_of_the_next_month() {
    let mut sim = world(2);
    // Nobody lives here, so the years pass quickly: the weather is the world's, whoever lives in
    // it.
    let living: Vec<_> = sim.people().people.iter().map(|(_, p)| p.id).collect();
    for id in living {
        sim.die_for_tests(id);
    }
    // The first note is of the month the world was made in, once it is over.
    let begun = {
        let months = &sim.land().weather.months;
        let last = months.last().expect("a new world has lived a year");
        let whole = i64::from(last.days) == MONTH_LENGTHS[usize::from(last.month)];
        months.len() + usize::from(whole)
    };
    sim.advance_minutes((3 * 365 + 10) * 24 * 60)
        .expect("advances");
    // What the record said on each first of a month since.
    let expected: Vec<WeatherNote> = {
        let (params, land) = (&sim.rules().land.weather, sim.land());
        (begun..=land.weather.months.len())
            .flat_map(|k| {
                let mut then = land.weather.clone();
                then.months.truncate(k);
                then.notes(params, &land.climatology)
            })
            .collect()
    };
    let noted: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::Weather)
        .collect();
    assert!(!noted.is_empty(), "three years with nothing that stood out");
    assert_eq!(noted.len(), expected.len());
    for (e, n) in noted.iter().zip(&expected) {
        assert_eq!(e.name, n.words);
        assert_eq!(e.number, f64::from(n.flags));
        assert!(e.people.is_empty() && e.settlement.is_none() && e.place.is_none());
        let date = e.at.date();
        assert_eq!((date.day, date.hour, date.minute), (1, 0, 0), "{}", e.name);
        // A year is noted on the first of January, a winter on the first of May.
        if n.flags & stood_out::YEAR != 0 {
            assert_eq!(date.month, 1, "{}", e.name);
        }
        if n.flags & (stood_out::LONG_WINTER | stood_out::MILD_WINTER) != 0 {
            assert_eq!(date.month, 5, "{}", e.name);
        }
    }
    // Told in the kernel's words.
    assert_eq!(
        civ_agents::history::render(noted[0], &|_| String::new()),
        vec![Span::Text(noted[0].name.clone())]
    );
    // Kept across a save (saves 25).
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "weather").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().chronicle, sim.people().chronicle);
}

/// The world of seed `seed` with nobody living in it, so its years pass quickly.
fn empty(seed: u64) -> Sim {
    let mut sim = world(seed);
    let living: Vec<_> = sim.people().people.iter().map(|(_, p)| p.id).collect();
    for id in living {
        sim.die_for_tests(id);
    }
    sim
}

#[test]
fn the_observer_holds_back_the_rain_and_the_months_it_held_are_drier_and_marked() {
    use civ_agents::influence::InfluenceKind;
    let mut dry = empty(4);
    let mut free = empty(4);
    assert_eq!(dry.land().weather, free.land().weather);
    // Out of bounds, in length or depth, is refused and holds nothing.
    for (months, depth) in [(2, 1.0), (25, 1.0), (6, 0.0), (6, 3.5), (6, f64::NAN)] {
        assert!(dry.drought(months, depth).is_err(), "{months} {depth}");
    }
    assert_eq!(dry.land().weather.hold, None);
    let day = dry.now().day_index();
    let reached = dry.drought(9, 2.0).expect("held");
    assert!(!reached.repeat);
    let until = day + (9.0_f64 * 365.0 / 12.0).round() as i64;
    assert_eq!(dry.land().weather.hold, Some((-2.0, until)));
    let begun = dry.land().weather.months.len() - 1;
    assert!(dry.land().weather.months[begun].held);
    // A record of the observer's act, and a line in the chronicle; a second use refreshes it.
    let records = |sim: &Sim| -> Vec<(u32, u32, i64)> {
        sim.people()
            .influences
            .list
            .iter()
            .filter(|i| i.kind == InfluenceKind::Drought)
            .map(|i| (i.uses, i.subject, i.until))
            .collect()
    };
    assert_eq!(records(&dry), vec![(1, 9, until)]);
    assert!(dry.drought(9, 2.0).expect("held again").repeat);
    assert_eq!(records(&dry), vec![(2, 9, until)]);
    let lines: Vec<_> = dry
        .people()
        .chronicle
        .iter()
        .filter(|e| {
            e.kind == ChronicleKind::Influence
                && e.number == f64::from(InfluenceKind::Drought.code())
        })
        .collect();
    assert_eq!(lines.len(), 2);
    assert!(
        lines[0]
            .name
            .starts_with(": for 9 months the weather runs dry, until "),
        "{}",
        lines[0].name
    );
    assert_eq!(lines[0].settlement, Some(dry.land().settlements[0].id));
    // Kept across a save, and the world goes on alike (saves 84).
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut dry, &saves, SaveKind::Manual, "drought").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().weather, dry.land().weather);
    assert_eq!(records(&loaded), records(&dry));
    // Through the drought and a month past it.
    let minutes = (until - day + 31) * 24 * 60;
    for sim in [&mut dry, &mut free, &mut loaded] {
        sim.advance_minutes(minutes).expect("advances");
    }
    assert_eq!(loaded.land().weather, dry.land().weather);
    assert_eq!(loaded.land().water, dry.land().water);
    let (wd, wf) = (&dry.land().weather, &free.land().weather);
    assert_eq!(wd.hold, None, "let go on its day");
    let held: Vec<usize> = (0..wd.months.len())
        .filter(|&k| wd.months[k].held)
        .collect();
    let last = held.last().copied().expect("months held");
    assert_eq!(held, (begun..=last).collect::<Vec<_>>());
    assert!(
        last >= begun + 9 && last + 1 < wd.months.len(),
        "{begun}..={last}"
    );
    assert!(wf.months.iter().all(|m| !m.held));
    let rain = |w: &civ_land::Weather| -> f64 {
        w.months[begun + 1..last]
            .iter()
            .map(|m| f64::from(m.precip_mm))
            .sum()
    };
    assert!(
        rain(wd) < 0.85 * rain(wf),
        "{:.0} mm against {:.0} mm",
        rain(wd),
        rain(wf)
    );
    // The ground holds less water for it.
    let heads = |sim: &Sim| -> f64 { sim.land().water.heads.iter().sum() };
    assert!(heads(&dry) < heads(&free), "the water table falls");
}

#[test]
fn the_wind_and_the_thatch_s_dryness_are_kept_across_a_save_and_drawn_afresh_from_an_older_one() {
    let mut sim = empty(5);
    sim.advance_minutes(40 * 24 * 60).expect("advances");
    let t = sim.land().weather.today;
    assert!(t.wind_ms >= 0.0 && t.fuel_moisture > 0.0, "{t:?}");
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "wind").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().weather, sim.land().weather);
    // Lived on, the two days' winds and dryness stay alike.
    for s in [&mut sim, &mut loaded] {
        s.advance_minutes(10 * 24 * 60).expect("advances");
    }
    assert_eq!(loaded.land().weather, sim.land().weather);
    // A save from before schema 87 holds no wind and no dryness (they read as nothing): they
    // are drawn afresh on loading, from no anomaly, and nothing else of the weather changes.
    {
        let w = &mut sim.land_mut_for_tests().weather;
        w.wind = (0.0, 0.0);
        w.today.wind_ms = 0.0;
        w.today.wind_from_deg = 0.0;
        w.today.gust_ms = 0.0;
        w.today.fuel_moisture = 0.0;
    }
    let before = sim.land().weather.clone();
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "old").expect("saves");
    let old = persist::load(&saved.path, content()).expect("loads");
    let w = &old.land().weather;
    assert!(
        w.today.fuel_moisture > 0.0 && w.today.wind_ms > 0.0,
        "{:?}",
        w.today
    );
    assert_eq!(
        (
            w.today.day,
            w.today.precip_mm,
            w.today.mean_c,
            w.months.clone(),
            w.soil_mm
        ),
        (
            before.today.day,
            before.today.precip_mm,
            before.today.mean_c,
            before.months.clone(),
            before.soil_mm
        )
    );
    let mut redrawn = before.clone();
    redrawn.draw_new_fields(&old.rules().land.weather, &old.land().climatology);
    assert_eq!(w, &redrawn);
}
