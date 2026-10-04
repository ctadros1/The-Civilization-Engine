//! Plots and buildings (ADR-0004 §3): ground a household has claimed for its home, and the
//! building it raises there. A building keeps its design (a [`BuildingSpec`], whose footprint is
//! authoritative) and how far its construction has gone: the stage under way and the work done on
//! it. Work on a stage uses its materials in proportion, and stops where they run out (research
//! 11-04 §5.2: progress is the least of labour, material and capacity). Nothing here decides who
//! builds what, or where.

use civ_core::{PermanentId, SimTime};
use civ_grammar::{BuildingSpec, Stage};

use crate::fields::RectCm;

/// What a plot is claimed for, and what the building on it is for (ADR-0009 §7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlotUse {
    /// A household's home.
    Dwelling,
    /// A store for goods.
    Store,
    /// A workshop.
    Work,
}

impl PlotUse {
    /// Every use, in a fixed order.
    pub const ALL: [PlotUse; 3] = [PlotUse::Dwelling, PlotUse::Store, PlotUse::Work];

    /// The authored name: `dwelling`, `store` or `work`.
    pub fn name(self) -> &'static str {
        match self {
            PlotUse::Dwelling => "dwelling",
            PlotUse::Store => "store",
            PlotUse::Work => "work",
        }
    }

    /// The use with an authored name.
    pub fn from_name(name: &str) -> Option<PlotUse> {
        PlotUse::ALL.into_iter().find(|u| u.name() == name)
    }
}

/// Ground a household has claimed (ADR-0004 §3). A plot outlives its building.
#[derive(Clone, Debug, PartialEq)]
pub struct Plot {
    /// Permanent id.
    pub id: PermanentId,
    /// The household that claimed it.
    pub household: PermanentId,
    /// Where it is.
    pub rect: RectCm,
    /// What it is for.
    pub use_: PlotUse,
    /// When it was claimed.
    pub since: SimTime,
}

/// A building and how far its construction has gone.
#[derive(Clone, Debug, PartialEq)]
pub struct Building {
    /// Permanent id.
    pub id: PermanentId,
    /// The household it belongs to.
    pub household: PermanentId,
    /// The plot it stands on.
    pub plot: PermanentId,
    /// Its design.
    pub spec: BuildingSpec,
    /// The stage under way, an index into [`Stage::ALL`]; the number of stages once finished.
    pub stage: u8,
    /// Person-hours of a capable adult done on the stage under way.
    pub work_h: f32,
    /// When work began.
    pub started: SimTime,
    /// When the stage under way began (when it was finished, once finished).
    pub stage_since: SimTime,
}

/// Work left on a stage below which it counts as done, hours (about four seconds): work is kept
/// as `f32`, which cannot tell a stage of a few hundred hours from one a few millionths of an
/// hour short of it.
pub const STAGE_DONE_SLACK_H: f64 = 1e-3;

/// What a piece of building work did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuildWork {
    /// Hours of work counted.
    pub hours: f64,
    /// Kilograms of each material slot used.
    pub used_kg: Vec<f64>,
    /// The stage it finished, if it did.
    pub finished: Option<Stage>,
}

/// Material a stage can be short of without its work being held up, kilograms per slot: the last
/// scraps are found about the site (a tuning value). Without it, stores that spoil a little
/// between cutting and use could leave a stage forever a few hundred grams short.
pub const MATERIAL_SLACK_KG: f64 = 0.5;

/// Hours of a stage needing `labour_h` and `materials_kg` (per slot) that can still be done,
/// `done_h` already done, with `held_kg` of each slot's material at hand.
pub fn workable_h(labour_h: f64, done_h: f64, materials_kg: &[f64], held_kg: &[f64]) -> f64 {
    let left = (labour_h - done_h).max(0.0);
    let mut hours = left;
    if labour_h <= 0.0 {
        return hours;
    }
    for (need, held) in materials_kg.iter().zip(held_kg) {
        if *need > 0.0 {
            let per_h = need / labour_h;
            let held = held.max(0.0);
            if per_h * left > held + MATERIAL_SLACK_KG {
                hours = hours.min(held / per_h);
            }
        }
    }
    hours
}

impl Building {
    /// The stage under way, `None` once finished.
    pub fn stage(&self) -> Option<Stage> {
        Stage::from_index(usize::from(self.stage))
    }

    /// The roof is on: its sleepers and stores are sheltered.
    pub fn roofed(&self) -> bool {
        usize::from(self.stage) > Stage::Roof.index()
    }

    /// Every stage is done.
    pub fn finished(&self) -> bool {
        usize::from(self.stage) >= Stage::ALL.len()
    }

    /// Does up to `hours` of a capable adult's work on the stage under way, which needs
    /// `labour_h` and `materials_kg` per slot, with `held_kg` of each at hand. Work beyond what
    /// the stage needs or the materials allow is not counted.
    pub fn work(
        &mut self,
        hours: f64,
        labour_h: f64,
        materials_kg: &[f64],
        held_kg: &[f64],
        now: SimTime,
    ) -> BuildWork {
        let mut done = BuildWork {
            used_kg: vec![0.0; materials_kg.len()],
            ..BuildWork::default()
        };
        let Some(stage) = self.stage() else {
            return done;
        };
        let before = f64::from(self.work_h);
        let h = hours
            .min(workable_h(labour_h, before, materials_kg, held_kg))
            .max(0.0);
        if labour_h > 0.0 {
            for (used, need) in done.used_kg.iter_mut().zip(materials_kg) {
                *used = need * h / labour_h;
            }
        }
        done.hours = h;
        let after = before + h;
        // Work is kept as `f32`: the stage is done once what is left is below what that can tell
        // apart at these sizes (a few seconds), not only when it is exactly nothing.
        if after + STAGE_DONE_SLACK_H >= labour_h {
            self.stage += 1;
            self.work_h = 0.0;
            self.stage_since = now;
            done.finished = Some(stage);
        } else {
            self.work_h = after as f32;
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_grammar::{Footprint, HUT_VERSION};

    fn building() -> Building {
        Building {
            id: PermanentId::from_raw(9).expect("non-zero"),
            household: PermanentId::from_raw(2).expect("non-zero"),
            plot: PermanentId::from_raw(8).expect("non-zero"),
            spec: BuildingSpec {
                program: "core:building/hut".into(),
                version: HUT_VERSION,
                footprint: Footprint::Round {
                    x: 1000,
                    y: 1000,
                    radius: 300,
                },
                storeys: 1,
                params: [0; civ_grammar::PARAMS],
                materials: Vec::new(),
                style_seed: 0,
            },
            stage: 0,
            work_h: 0.0,
            started: SimTime::ZERO,
            stage_since: SimTime::ZERO,
        }
    }

    #[test]
    fn work_uses_materials_in_proportion_and_stops_where_they_run_out() {
        let mut b = building();
        b.stage = Stage::Frame.index() as u8;
        // 40 h of frame work needing 800 kg of timber: 20 kg an hour. 100 kg at hand: 5 h.
        assert_eq!(workable_h(40.0, 0.0, &[800.0, 0.0], &[100.0, 0.0]), 5.0);
        let done = b.work(8.0, 40.0, &[800.0, 0.0], &[100.0, 0.0], SimTime::ZERO);
        assert_eq!(done.hours, 5.0);
        assert_eq!(done.used_kg, vec![100.0, 0.0]);
        assert_eq!(done.finished, None);
        // With timber enough, the stage finishes and the next begins.
        let done = b.work(50.0, 40.0, &[800.0, 0.0], &[1000.0, 0.0], SimTime::ZERO);
        assert_eq!(done.hours, 35.0);
        assert_eq!(done.finished, Some(Stage::Frame));
        assert_eq!(b.stage(), Some(Stage::Walls));
        assert!(!b.roofed());
    }

    #[test]
    fn a_building_is_roofed_before_it_is_finished() {
        let mut b = building();
        b.stage = Stage::Roof.index() as u8;
        assert!(!b.roofed());
        b.work(10.0, 10.0, &[], &[], SimTime::ZERO);
        assert!(b.roofed() && !b.finished());
        b.work(10.0, 10.0, &[], &[], SimTime::ZERO);
        assert!(b.finished() && b.stage().is_none());
        // Finished, it takes no more work.
        assert_eq!(b.work(10.0, 10.0, &[], &[], SimTime::ZERO).hours, 0.0);
    }

    #[test]
    fn a_stage_is_not_held_up_by_its_last_scraps_of_material() {
        let mut b = building();
        b.stage = Stage::Walls.index() as u8;
        // 93 h of walls needing 186 kg of rods: 2 kg an hour. With 92.9 h done, 0.2 kg is left
        // to put in and none is at hand: the scraps about the site do.
        b.work_h = 92.9;
        assert!((workable_h(93.0, 92.9, &[0.0, 186.0], &[0.0, 0.0]) - 0.1).abs() < 1e-4);
        let done = b.work(1.0, 93.0, &[0.0, 186.0], &[0.0, 0.0], SimTime::ZERO);
        assert_eq!(done.finished, Some(Stage::Walls));
        // More than the slack short, the work waits for the material.
        assert_eq!(workable_h(93.0, 90.0, &[0.0, 186.0], &[0.0, 0.0]), 0.0);
    }

    #[test]
    fn a_stage_finishes_although_its_hours_do_not_fit_an_f32() {
        // 100.1 h is 100.09999847 h as an f32: work done in pieces up to it still ends the stage.
        let mut b = building();
        b.work(66.6, 100.1, &[], &[], SimTime::ZERO);
        let done = b.work(50.0, 100.1, &[], &[], SimTime::ZERO);
        assert_eq!(done.finished, Some(Stage::Foundation));
        assert_eq!(b.stage(), Some(Stage::Frame));
        // And a stage saved a hair short of done finishes with the next minute of work.
        b.work_h = (100.1f64 - 2e-6) as f32;
        b.stage = 0;
        assert!(workable_h(100.1, f64::from(b.work_h), &[], &[]) < STAGE_DONE_SLACK_H);
        assert_eq!(
            b.work(1.0 / 60.0, 100.1, &[], &[], SimTime::ZERO).finished,
            Some(Stage::Foundation)
        );
    }
}
