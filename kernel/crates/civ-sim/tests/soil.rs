//! The soil (M3c slice V, ADR-0012 §3): each field a world's people crop keeps its soil's
//! nitrogen, turned on the first of each January, and the record of its harvests, and both are
//! kept across a save.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_content::ContentRegistry;
use civ_core::SimTime;
use civ_land::FieldSoil;
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

#[test]
fn cropped_fields_keep_their_soil_and_harvests_across_the_years_and_a_save() {
    let mut sim = Sim::create(
        &NewWorld {
            name: "Soil".to_owned(),
            seed: 2,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            regime_id: String::new(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates");
    // Two harvests, and the turns of two Januaries after them.
    let until = SimTime::from_date(3, 1, 2, 0, 0).expect("a date");
    sim.advance_minutes(until.minutes() - sim.now().minutes())
        .expect("lives");
    let soil = &sim.rules().land.soil;
    let fields = &sim.land().fields;
    let cropped: Vec<_> = fields.iter().filter(|f| f.harvests > 0).collect();
    assert!(!cropped.is_empty(), "the band crops some ground");
    for f in &cropped {
        // Every harvest of a field broken in the world is on record, oldest first.
        assert_eq!(
            f.soil.record.len(),
            usize::from(f.harvests),
            "field {}",
            f.id
        );
        assert!(f.soil.record.windows(2).all(|w| w[0].year < w[1].year));
        assert!(
            f.soil
                .record
                .iter()
                .all(|r| r.year <= 2 && r.kg_per_ha > 0.0)
        );
        // Cropping draws the soil down from native ground's.
        let native = FieldSoil::native(soil, f64::from(f.ground));
        assert!(
            f.soil.supply_n < native.supply_n,
            "field {}: {} after {} harvests, {} native",
            f.id,
            f.soil.supply_n,
            f.harvests,
            native.supply_n
        );
        assert!(f.soil.fast_n >= 0.0 && f.soil.slow_n > 0.0);
    }
    // Kept across a save (saves 26).
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "soil").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().fields, sim.land().fields);
}
