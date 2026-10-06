//! Fields on the boundary (M1 slice C): the snapshot's field revision and the response to a
//! fields query. Each field's state in words is rendered here; observers show it. Who holds and
//! who works each field, and its lease, came with M3a slice K (ADR-0007).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_land::{CropParams, Field, FieldStage, Party};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// A number that changes whenever a field is marked out, changes stage or changes hands (0 = no
/// fields): a hash of each field's stage start, user, holder and lease.
pub fn fields_rev(sim: &Sim) -> u64 {
    let fields = &sim.land.fields;
    if fields.is_empty() {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    for f in fields {
        (
            f.id.get(),
            f.stage_since.minutes(),
            f.household.get(),
            f.holder,
            f.lease.map(|l| l.until.minutes()),
        )
            .hash(&mut hasher);
    }
    hasher.finish() | 1
}

fn stage(s: FieldStage) -> wire::FieldStage {
    match s {
        FieldStage::Fallow => wire::FieldStage::Fallow,
        FieldStage::Prepared => wire::FieldStage::Prepared,
        FieldStage::Sown => wire::FieldStage::Sown,
        FieldStage::Reaped => wire::FieldStage::Reaped,
    }
}

/// Share of the work of the stage under way that is done: tending while a crop grows, the
/// stage's own work otherwise (threshing has no total to measure against).
fn progress(f: &Field, crop: &CropParams, day: i64) -> f64 {
    let share = |done: f64, all: f64| {
        if all > 0.0 {
            (done / all).clamp(0.0, 1.0)
        } else {
            1.0
        }
    };
    match f.stage {
        FieldStage::Sown if day < f.ripe_day(crop) => {
            share(f64::from(f.tended_h), crop.tend_h_per_ha * f.area_ha())
        }
        FieldStage::Reaped => 0.0,
        _ => share(f64::from(f.work_h), f.stage_work_h(crop)),
    }
}

fn percent(x: f64) -> String {
    format!("{:.0}%", (x * 100.0).clamp(0.0, 100.0))
}

fn days(n: i64) -> String {
    match n {
        1 => "1 day".to_owned(),
        n => format!("{n} days"),
    }
}

/// A field's state in words on `day`: "growing; ripe in about 20 days, weeded 60%; has had 82% of
/// the water it needed".
pub fn status(f: &Field, crop: &CropParams, day: i64) -> String {
    let mut s = stage_words(f, crop, day);
    // The water its crop has had so far this season (ADR-0012 §2), once it has needed some.
    if f.stage == FieldStage::Sown && f.need_mm > 0.0 {
        let had = percent(f64::from(f.got_mm) / f64::from(f.need_mm));
        if day < f.ripe_day(crop) {
            s.push_str(&format!("; has had {had} of the water it needed"));
        } else {
            s.push_str(&format!("; had {had} of the water it needed"));
        }
    }
    s
}

/// Where a field is in its year, in words.
fn stage_words(f: &Field, crop: &CropParams, day: i64) -> String {
    let done = progress(f, crop, day);
    let doy = day.rem_euclid(civ_core::time::DAYS_PER_YEAR);
    match f.stage {
        FieldStage::Fallow if !f.broken => {
            let woodland = f.clear_h_per_ha > 0.0;
            match (woodland, f.work_h > 0.0) {
                (true, true) => format!("woodland being cleared, {} done", percent(done)),
                (false, true) => format!("new ground being broken, {} done", percent(done)),
                (true, false) => "woodland marked out to clear".to_owned(),
                (false, false) => "new ground marked out".to_owned(),
            }
        }
        FieldStage::Fallow if f.work_h > 0.0 => {
            format!("being dug for sowing, {} done", percent(done))
        }
        FieldStage::Fallow if crop.can_prepare(day) => "fallow, to be dug for sowing".to_owned(),
        FieldStage::Fallow => "fallow".to_owned(),
        FieldStage::Prepared if f.work_h > 0.0 => format!("being sown, {} done", percent(done)),
        FieldStage::Prepared if crop.can_sow(day) => "dug and ready to sow".to_owned(),
        FieldStage::Prepared => {
            let wait = i64::from(crop.sow_from_day) - doy;
            format!("dug; sowing begins in {}", days(wait.max(1)))
        }
        FieldStage::Sown => {
            let ripe = f.ripe_day(crop);
            if day < ripe {
                let mut s = format!("growing; ripe in about {}", days(ripe - day));
                if done < 1.0 {
                    s.push_str(&format!(", weeded {}", percent(done)));
                }
                s
            } else if f.work_h > 0.0 {
                format!("ripe, being reaped, {} done", percent(done))
            } else if day == ripe {
                "ripe today".to_owned()
            } else {
                format!("ripe, standing {}", days(day - ripe))
            }
        }
        FieldStage::Reaped => format!("{:.0} kg of sheaves to thresh", f.sheaves_kg),
    }
}

/// A `Response` with every field, oldest first.
pub fn fields_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let day = sim.now().day_index();
    let crops = &sim.rules.catalog.crops;
    let soil_mm = sim.rules.land.soil.soil_water_mm.max(1e-9);
    let list: Vec<_> = sim
        .land
        .fields
        .iter()
        .map(|f| {
            let crop = crops.get(usize::from(f.crop));
            let text = crop.map_or_else(String::new, |c| status(f, c, day));
            let text = fbb.create_string(&text);
            // Its settlement: its user's, or else the settlement that holds it.
            let settlement = sim
                .people
                .household(f.household)
                .and_then(|h| h.settlement)
                .or(f.holder.settlement())
                .map_or(0, |s| s.get());
            let r = f.rect;
            wire::FieldInfo::create(
                &mut fbb,
                &wire::FieldInfoArgs {
                    id: f.id.get(),
                    household: f.household.get(),
                    settlement,
                    min: Some(&wire::Vec2::new(r.x as f32 / 100.0, r.y as f32 / 100.0)),
                    size: Some(&wire::Vec2::new(r.w as f32 / 100.0, r.h as f32 / 100.0)),
                    crop: f.crop,
                    stage: stage(f.stage),
                    stage_since_minute: f.stage_since.minutes(),
                    progress: crop.map_or(0.0, |c| progress(f, c, day)) as f32,
                    new_ground: !f.broken,
                    woodland: !f.broken && f.clear_h_per_ha > 0.0,
                    expected_kg: crop
                        .map_or(0.0, |c| f.expected_kg(c, day, sim.land.expected_water(f)))
                        as f32,
                    sheaves_kg: f.sheaves_kg,
                    harvests: u32::from(f.harvests),
                    status: Some(text),
                    ripe: f.stage == FieldStage::Sown && crop.is_some_and(|c| day >= f.ripe_day(c)),
                    holder: f.holder.household().map_or(0, |h| h.get()),
                    holder_settlement: match f.holder {
                        Party::Settlement(s) => s.get(),
                        Party::Household(_) => 0,
                    },
                    lease_until_minute: f.lease.map_or(-1, |l| l.until.minutes()),
                    lease_share: f.lease.map_or(0.0, |l| l.holder_share),
                    water_had: if f.stage == FieldStage::Sown && f.need_mm > 0.0 {
                        (f.got_mm / f.need_mm).clamp(0.0, 1.0)
                    } else {
                        -1.0
                    },
                    soil_water: (f64::from(f.water_mm) / soil_mm) as f32,
                    fast_n_kg: 0.0,
                    slow_n_kg: 0.0,
                    harvest_records: None,
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let body = wire::Fields::create(
        &mut fbb,
        &wire::FieldsArgs {
            rev: fields_rev(sim),
            fields: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Fields, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_core::{PermanentId, SimTime};
    use civ_land::RectCm;

    fn crop() -> CropParams {
        CropParams {
            id: "core:crop/emmer".into(),
            name: "Emmer".into(),
            good: 0,
            seed_good: 1,
            seed_kg_per_ha: 90.0,
            yield_kg_per_ha: 800.0,
            prepare_from_day: 59,
            sow_from_day: 80,
            sow_until_day: 125,
            late_sowing_loss_per_day: 0.005,
            grow_days: 120,
            standing_loss_per_day: 0.02,
            untended_loss: 0.4,
            break_h_per_ha: 1000.0,
            prepare_h_per_ha: 600.0,
            sow_h_per_ha: 120.0,
            tend_h_per_ha: 200.0,
            reap_h_per_ha: 280.0,
            thresh_h_per_kg: 0.1,
            straw: None,
            kc: [0.4, 1.15, 0.4],
            kc_days: [30, 30, 40, 20],
            ky: 1.15,
            n_kg_per_kg_grain: 0.02,
            n_kg_per_kg_straw: 0.005,
            roots_n_kg_per_ha: 15.0,
        }
    }

    fn field(stage: FieldStage) -> Field {
        Field {
            id: PermanentId::from_raw(5).expect("non-zero"),
            household: PermanentId::from_raw(2).expect("non-zero"),
            holder: civ_land::Party::Household(PermanentId::from_raw(2).expect("non-zero")),
            lease: None,
            rect: RectCm {
                x: 0,
                y: 0,
                w: 5000,
                h: 5000,
            },
            crop: 0,
            stage,
            stage_since: SimTime::ZERO,
            work_h: 0.0,
            tended_h: 0.0,
            ground: 1.0,
            clear_h_per_ha: 0.0,
            broken: true,
            sown_day: 90,
            sheaves_kg: 0.0,
            harvests: 1,
            water_mm: 0.0,
            need_mm: 0.0,
            got_mm: 0.0,
            fast_n_kg: 0.0,
            slow_n_kg: 0.0,
            harvest_records: Vec::new(),
        }
    }

    #[test]
    fn a_field_says_where_it_is_in_its_year() {
        let c = crop();
        let mut wood = field(FieldStage::Fallow);
        wood.broken = false;
        wood.clear_h_per_ha = 1000.0;
        assert_eq!(status(&wood, &c, 60), "woodland marked out to clear");
        wood.work_h = 250.0; // of 500 h for 0.25 ha
        assert_eq!(status(&wood, &c, 60), "woodland being cleared, 50% done");
        assert_eq!(status(&field(FieldStage::Fallow), &c, 200), "fallow");
        assert_eq!(
            status(&field(FieldStage::Prepared), &c, 70),
            "dug; sowing begins in 10 days"
        );
        let mut sown = field(FieldStage::Sown);
        sown.tended_h = 25.0; // of 50 h
        assert_eq!(
            status(&sown, &c, 150),
            "growing; ripe in about 60 days, weeded 50%"
        );
        assert_eq!(status(&sown, &c, 215), "ripe, standing 5 days");
        let mut reaped = field(FieldStage::Reaped);
        reaped.sheaves_kg = 180.4;
        assert_eq!(status(&reaped, &c, 220), "180 kg of sheaves to thresh");
        // A growing crop says how much of the water it needed it has had.
        sown.need_mm = 200.0;
        sown.got_mm = 164.0;
        assert_eq!(
            status(&sown, &c, 150),
            "growing; ripe in about 60 days, weeded 50%; has had 82% of the water it needed"
        );
        assert_eq!(
            status(&sown, &c, 215),
            "ripe, standing 5 days; had 82% of the water it needed"
        );
    }
}
