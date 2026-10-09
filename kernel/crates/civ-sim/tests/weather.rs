//! The weather in the chronicle (M3c slice U, ADR-0012): what stood out in a month, a winter or a
//! year against what it usually brings is noted on the first of the next month, as the weather's
//! record says, whoever lives in the world, and kept across a save.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

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
    Sim::create(
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
        &mut |_| {},
        &AtomicBool::new(false),
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
