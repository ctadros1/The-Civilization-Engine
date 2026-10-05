//! Crops (`kind = "crop"`): a plant people sow, tend and reap, with its calendar and the work and
//! seed it takes by hand. Which fields are sown, by whom and when is decided by people at run
//! time.

use civ_land::CropParams;
use serde::Deserialize;

/// The `kind` value of a crop.
pub const KIND: &str = "crop";
/// The id segment: `pack:crop/name`.
pub const ID_KIND: &str = "crop";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CropFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// The good the harvest yields: a good id.
    pub good: String,
    /// The good seed is kept as: a good id.
    pub seed_good: String,
    pub seed_kg_per_ha: f64,
    pub yield_kg_per_ha: f64,
    pub prepare_from_day: u16,
    pub sow_from_day: u16,
    pub sow_until_day: u16,
    pub late_sowing_loss_per_day: f64,
    pub grow_days: u16,
    pub standing_loss_per_day: f64,
    pub untended_loss: f64,
    pub break_h_per_ha: f64,
    pub prepare_h_per_ha: f64,
    pub sow_h_per_ha: f64,
    pub tend_h_per_ha: f64,
    pub reap_h_per_ha: f64,
    pub thresh_h_per_kg: f64,
    /// The good the straw is kept as (a good id), and how much of it a kilogram of grain leaves:
    /// both or neither.
    pub straw_good: Option<String>,
    pub straw_kg_per_kg: Option<f64>,
    /// Crop coefficients at the start, in mid-season and at ripeness (ADR-0012 §2; content
    /// API 23).
    pub kc: [f64; 3],
    /// Days of the initial, development, mid-season and late stages: they sum to `grow_days`.
    pub kc_days: [u16; 4],
    /// Yield lost per share of the water need unmet.
    pub ky: f64,
}

impl CropFile {
    /// The parameters, with its goods resolved by `good_index` (`None` if one is unknown, which
    /// the cross-file check reports).
    pub fn params(&self, good_index: &dyn Fn(&str) -> Option<usize>) -> Option<CropParams> {
        let straw = match (&self.straw_good, self.straw_kg_per_kg) {
            (Some(good), Some(kg)) => Some((good_index(good)?, kg)),
            _ => None,
        };
        Some(CropParams {
            id: self.id.clone(),
            name: self.name.clone(),
            good: good_index(&self.good)?,
            seed_good: good_index(&self.seed_good)?,
            seed_kg_per_ha: self.seed_kg_per_ha,
            yield_kg_per_ha: self.yield_kg_per_ha,
            prepare_from_day: self.prepare_from_day,
            sow_from_day: self.sow_from_day,
            sow_until_day: self.sow_until_day,
            late_sowing_loss_per_day: self.late_sowing_loss_per_day,
            grow_days: self.grow_days,
            standing_loss_per_day: self.standing_loss_per_day,
            untended_loss: self.untended_loss,
            break_h_per_ha: self.break_h_per_ha,
            prepare_h_per_ha: self.prepare_h_per_ha,
            sow_h_per_ha: self.sow_h_per_ha,
            tend_h_per_ha: self.tend_h_per_ha,
            reap_h_per_ha: self.reap_h_per_ha,
            thresh_h_per_kg: self.thresh_h_per_kg,
            straw,
            kc: self.kc,
            kc_days: self.kc_days,
            ky: self.ky,
        })
    }

    /// Range problems, as messages. Goods are checked against the goods later.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        let positive = |name: &str, v: f64, p: &mut Vec<String>| {
            if !(v.is_finite() && v > 0.0) {
                p.push(format!("`{name}` must be positive (got {v})"));
            }
        };
        let share = |name: &str, v: f64, p: &mut Vec<String>| {
            if !(v.is_finite() && (0.0..=1.0).contains(&v)) {
                p.push(format!("`{name}` must be between 0 and 1 (got {v})"));
            }
        };
        positive("seed_kg_per_ha", self.seed_kg_per_ha, &mut p);
        positive("yield_kg_per_ha", self.yield_kg_per_ha, &mut p);
        for (name, v) in [
            ("break_h_per_ha", self.break_h_per_ha),
            ("prepare_h_per_ha", self.prepare_h_per_ha),
            ("sow_h_per_ha", self.sow_h_per_ha),
            ("reap_h_per_ha", self.reap_h_per_ha),
            ("thresh_h_per_kg", self.thresh_h_per_kg),
        ] {
            positive(name, v, &mut p);
        }
        if !(self.tend_h_per_ha.is_finite() && self.tend_h_per_ha >= 0.0) {
            p.push("`tend_h_per_ha` must be zero or more".to_owned());
        }
        for (name, v) in [
            ("late_sowing_loss_per_day", self.late_sowing_loss_per_day),
            ("standing_loss_per_day", self.standing_loss_per_day),
            ("untended_loss", self.untended_loss),
        ] {
            share(name, v, &mut p);
        }
        match (&self.straw_good, self.straw_kg_per_kg) {
            (Some(_), Some(kg)) if !(kg.is_finite() && kg >= 0.0) => {
                p.push(format!("`straw_kg_per_kg` must be zero or more (got {kg})"));
            }
            (Some(_), None) | (None, Some(_)) => {
                p.push("`straw_good` and `straw_kg_per_kg` go together".to_owned());
            }
            _ => {}
        }
        if self.standing_loss_per_day <= 0.0 {
            p.push("`standing_loss_per_day` must be above 0, so an unreaped crop ends".to_owned());
        }
        if !(self.prepare_from_day <= self.sow_from_day
            && self.sow_from_day <= self.sow_until_day
            && self.sow_until_day < 365)
        {
            p.push(
                "days must satisfy prepare_from_day <= sow_from_day <= sow_until_day < 365"
                    .to_owned(),
            );
        }
        if let Some(bad) = self
            .kc
            .iter()
            .find(|v| !(v.is_finite() && (0.05..=2.0).contains(*v)))
        {
            p.push(format!("every `kc` must be between 0.05 and 2 (got {bad})"));
        }
        if self.kc_days.iter().map(|&d| u32::from(d)).sum::<u32>() != u32::from(self.grow_days) {
            p.push(format!(
                "the stages of `kc_days` must sum to `grow_days` ({})",
                self.grow_days
            ));
        }
        if !(self.ky.is_finite() && (0.0..=3.0).contains(&self.ky)) {
            p.push(format!("`ky` must be between 0 and 3 (got {})", self.ky));
        }
        if self.grow_days == 0 || u32::from(self.sow_until_day) + u32::from(self.grow_days) >= 365 {
            p.push(
                "`grow_days` must be at least 1 and ripen the crop within the year it is sown"
                    .to_owned(),
            );
        }
        p
    }
}
