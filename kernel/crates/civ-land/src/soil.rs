//! The soil (ADR-0012 §3): what a field's ground holds of nitrogen, and how cropping and rest
//! change it. Research 03-04 sets the frame: fertility is stocks with their own turnover, not a
//! share that falls with each crop and returns with each fallow year (§1, §5.2); a harvest is the
//! least of what the season's water and the soil's nitrogen allow (§5.2); what the crop carries
//! off leaves the field and what it leaves behind returns (§1.2); nothing makes nitrogen out of
//! nothing (§5.5).
//!
//! Each field keeps two pools of organic nitrogen, kilograms per hectare: a fast one (roots,
//! stubble and fresh organic matter, turning over in a few years) and a slow one (humus, turning
//! over in decades). Once a year, on the first of January and with no random draws, each pool
//! releases its share as mineral nitrogen; that and what the air and free-living microbes add are
//! the year's supply for a crop (§2.1). A crop takes up a share of the supply. Its grain and the
//! straw carried home leave the field, while its roots and stubble return to the fast pool. What
//! no crop takes up is lost to leaching and the air. A field left unsown grows its wild cover,
//! whose litter brings each pool back toward the native ground's stock at the pool's own pace:
//! the fast pool within a few years, the slow one over decades (§2.5: fallow restores some
//! functions in 10-15 years, and others only partly after 50).

use crate::fields::CropParams;

/// Harvests a field remembers (ADR-0012 §3).
pub const RECORD_KEPT: usize = 8;

/// The day of the year the soil turns: the first of January, before any field is prepared.
pub const TURN_DAY: i64 = 0;

/// How a landscape's soils hold nitrogen (the land profile's `[soil]`; content API 27).
#[derive(Clone, Debug, PartialEq)]
pub struct SoilParams {
    /// Organic nitrogen in the humus of native ground of average richness, kg/ha.
    pub slow_n_kg_ha: f64,
    /// Organic nitrogen in the fast pool of native ground of average richness, kg/ha.
    pub fast_n_kg_ha: f64,
    /// The share of the slow pool that turns to mineral nitrogen in a year.
    pub slow_turnover: f64,
    /// The share of the fast pool that turns to mineral nitrogen in a year.
    pub fast_turnover: f64,
    /// Mineral nitrogen the air and free-living microbes add each year, kg/ha.
    pub free_n_kg_ha: f64,
    /// The share of a year's mineral nitrogen a crop can take up.
    pub uptake_share: f64,
}

impl SoilParams {
    /// The native stocks of ground of richness `ground` (1 for average ground): the fast and
    /// slow pools, kg/ha.
    pub fn native(&self, ground: f64) -> (f64, f64) {
        let g = ground.max(0.0);
        (self.fast_n_kg_ha * g, self.slow_n_kg_ha * g)
    }

    /// The grain a year's supply of `supply_kg_ha` lets a crop give, kg/ha.
    pub fn allows_kg_ha(&self, supply_kg_ha: f64, crop: &CropParams) -> f64 {
        if crop.crop_n <= 0.0 {
            return f64::INFINITY;
        }
        supply_kg_ha.max(0.0) * self.uptake_share / crop.crop_n
    }
}

/// What held a harvest back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Limit {
    /// The season: its water, and how well and when the work was done.
    Season,
    /// The soil's nitrogen.
    Soil,
}

impl Limit {
    /// Every limit, in order (part of the boundary: never reorder).
    pub const ALL: [Limit; 2] = [Limit::Season, Limit::Soil];
}

/// One harvest of a field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarvestRecord {
    /// The calendar year it was reaped.
    pub year: i32,
    /// The grain it gave, kilograms per hectare.
    pub kg_per_ha: f32,
    /// What held it back.
    pub limit: Limit,
}

/// What a field's soil holds, kilograms of nitrogen per hectare, and the record of its harvests.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldSoil {
    /// The fast pool of organic nitrogen.
    pub fast_n: f32,
    /// The slow pool of organic nitrogen.
    pub slow_n: f32,
    /// Mineral nitrogen this year's turn made available to a crop.
    pub supply_n: f32,
    /// Its last harvests, oldest first; at most [`RECORD_KEPT`].
    pub record: Vec<HarvestRecord>,
}

impl FieldSoil {
    /// The soil of native ground of richness `ground` just after its year's turn: what a field
    /// marked out on it starts with.
    pub fn native(params: &SoilParams, ground: f64) -> FieldSoil {
        let (fast, slow) = params.native(ground);
        let mut soil = FieldSoil {
            fast_n: fast as f32,
            slow_n: slow as f32,
            ..FieldSoil::default()
        };
        soil.release(params);
        soil
    }

    /// The year's turn at the start of calendar year `year`, for ground of richness `ground`: a
    /// field that gave no harvest in the year just ended had its wild cover, whose litter brings
    /// each pool back toward native ground's at the pool's own pace; then each pool releases its
    /// share as the new year's supply, and last year's unused supply is lost.
    pub fn turn(&mut self, params: &SoilParams, ground: f64, year: i32) {
        if self.last_harvest_year() != Some(year - 1) {
            let (fast, slow) = params.native(ground);
            self.fast_n += (params.fast_turnover * fast) as f32;
            self.slow_n += (params.slow_turnover * slow) as f32;
        }
        self.release(params);
    }

    /// Each pool releases its share as mineral nitrogen, which with what the air adds is the
    /// year's supply.
    fn release(&mut self, params: &SoilParams) {
        let fast = f64::from(self.fast_n).max(0.0);
        let slow = f64::from(self.slow_n).max(0.0);
        let (from_fast, from_slow) = (
            fast * params.fast_turnover.clamp(0.0, 1.0),
            slow * params.slow_turnover.clamp(0.0, 1.0),
        );
        self.fast_n = (fast - from_fast) as f32;
        self.slow_n = (slow - from_slow) as f32;
        self.supply_n = (from_fast + from_slow + params.free_n_kg_ha.max(0.0)) as f32;
    }

    /// The grain this year's supply lets `crop` give, kg/ha.
    pub fn allows_kg_ha(&self, params: &SoilParams, crop: &CropParams) -> f64 {
        params.allows_kg_ha(f64::from(self.supply_n), crop)
    }

    /// Accounts a harvest in calendar year `year`: the crop grew `grown_kg_ha` of grain (what the
    /// season and soil allowed, before what it lost standing ripe) and `taken_kg_ha` was carried
    /// home, held back by `limit`. Its grain and the straw carried home take their nitrogen from
    /// the field; the rest of what it took up, its roots and stubble and what it shed, returns to
    /// the fast pool. Returns the nitrogen carried off, kg/ha.
    pub fn harvested(
        &mut self,
        crop: &CropParams,
        grown_kg_ha: f64,
        taken_kg_ha: f64,
        limit: Limit,
        year: i32,
    ) -> f64 {
        let (grown, taken) = (grown_kg_ha.max(0.0), taken_kg_ha.max(0.0));
        let straw = crop.straw.map_or(0.0, |(_, kg)| kg.max(0.0));
        let up = grown * crop.crop_n.max(0.0);
        let off = (taken * (crop.grain_n.max(0.0) + straw * crop.straw_n.max(0.0))).min(up);
        self.fast_n += (up - off) as f32;
        if self.record.len() >= RECORD_KEPT {
            self.record.remove(0);
        }
        self.record.push(HarvestRecord {
            year,
            kg_per_ha: taken as f32,
            limit,
        });
        off
    }

    /// The calendar year of its last harvest, if it has one on record.
    pub fn last_harvest_year(&self) -> Option<i32> {
        self.record.last().map(|r| r.year)
    }

    /// Rebuilds the soil of a field saved before soils were kept (ADR-0012 §6): native ground of
    /// richness `ground`, cropped `harvests` years in a row at what an average season and the
    /// soil allowed `crop`.
    pub fn replayed(
        params: &SoilParams,
        crop: &CropParams,
        ground: f64,
        harvests: u16,
    ) -> FieldSoil {
        let mut soil = FieldSoil::native(params, ground);
        let season = crop.yield_kg_per_ha * ground.max(0.0);
        for year in 0..i32::from(harvests) {
            let soil_kg = soil.allows_kg_ha(params, crop);
            let (kg, limit) = if soil_kg < season {
                (soil_kg, Limit::Soil)
            } else {
                (season, Limit::Season)
            };
            soil.harvested(crop, kg, kg, limit, year);
            soil.turn(params, ground, year + 1);
        }
        // The years replayed are not the world's: no harvest is on record.
        soil.record.clear();
        soil
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::fields::tests::crop;

    pub(crate) fn params() -> SoilParams {
        SoilParams {
            slow_n_kg_ha: 2600.0,
            fast_n_kg_ha: 100.0,
            slow_turnover: 0.02,
            fast_turnover: 0.3,
            free_n_kg_ha: 15.0,
            uptake_share: 0.6,
        }
    }

    fn total(s: &FieldSoil) -> f64 {
        f64::from(s.fast_n) + f64::from(s.slow_n)
    }

    #[test]
    fn native_ground_releases_its_share_and_the_air_adds_its_own() {
        let p = params();
        let s = FieldSoil::native(&p, 1.0);
        // 0.3 of 100 and 0.02 of 2,600, and 15 from the air.
        assert!((f64::from(s.supply_n) - 97.0).abs() < 1e-3);
        assert!((f64::from(s.fast_n) - 70.0).abs() < 1e-3);
        assert!((f64::from(s.slow_n) - 2548.0).abs() < 1e-2);
        // Richer ground holds more.
        assert!(FieldSoil::native(&p, 1.4).supply_n > s.supply_n);
        // 97 kg at 0.6 taken up, 0.035 kg a kilogram of grain.
        let c = crop();
        assert!((s.allows_kg_ha(&p, &c) - 97.0 * 0.6 / c.crop_n).abs() < 0.1);
    }

    #[test]
    fn nitrogen_is_neither_made_nor_lost_unaccounted() {
        let (p, c) = (params(), crop());
        let mut s = FieldSoil::native(&p, 1.0);
        for year in 0..30 {
            let before = total(&s) + f64::from(s.supply_n);
            let grown = s.allows_kg_ha(&p, &c).min(1000.0);
            let taken = grown * 0.9;
            let off = s.harvested(&c, grown, taken, Limit::Season, year);
            let up = grown * c.crop_n;
            assert!(up <= f64::from(s.supply_n) * p.uptake_share + 1e-6);
            // What the crop took up and did not carry off is back in the soil; the rest of the
            // supply is lost.
            let lost = f64::from(s.supply_n) - up;
            let after = total(&s);
            assert!(
                (before - off - lost - after).abs() < 1e-2,
                "year {year}: {before} - {off} - {lost} != {after}"
            );
            // The turn moves nothing but what it releases and what the air adds.
            let pools = total(&s);
            s.turn(&p, 1.0, year + 1);
            let released = f64::from(s.supply_n) - p.free_n_kg_ha;
            assert!((pools - released - total(&s)).abs() < 1e-2);
            assert!(s.fast_n >= 0.0 && s.slow_n >= 0.0 && s.supply_n >= 0.0);
        }
    }

    #[test]
    fn cropping_every_year_falls_toward_a_low_level_that_is_not_nothing() {
        let (p, c) = (params(), crop());
        let mut s = FieldSoil::native(&p, 1.0);
        let mut allowed = Vec::new();
        for year in 0..200 {
            let soil = s.allows_kg_ha(&p, &c);
            allowed.push(soil);
            let grown = soil.min(c.yield_kg_per_ha);
            s.harvested(&c, grown, grown, Limit::Season, year);
            s.turn(&p, 1.0, year + 1);
        }
        // New ground allows more than an average season brings (research 03-04 §4.1: continuous
        // cropping need not run down to nothing).
        assert!(allowed[0] > c.yield_kg_per_ha);
        assert!(allowed[50] < allowed[0] * 0.8);
        assert!(allowed[199] < allowed[50]);
        // The air's nitrogen alone keeps a crop of about 0.6 * 15 / 0.035 = 257 kg/ha.
        assert!(allowed[199] > 250.0, "{}", allowed[199]);
    }

    #[test]
    fn rest_brings_the_fast_pool_back_in_years_and_the_slow_one_in_decades() {
        let (p, c) = (params(), crop());
        let mut s = FieldSoil::native(&p, 1.0);
        for year in 0..40 {
            let grown = s.allows_kg_ha(&p, &c).min(c.yield_kg_per_ha);
            s.harvested(&c, grown, grown, Limit::Season, year);
            s.turn(&p, 1.0, year + 1);
        }
        let (fast0, slow0) = (s.fast_n, s.slow_n);
        let worn = s.allows_kg_ha(&p, &c);
        for year in 41..46 {
            s.turn(&p, 1.0, year);
        }
        let (native_fast, native_slow) = p.native(1.0);
        // Five years of rest: the fast pool most of the way back, the slow one a little.
        let fast_back = (f64::from(s.fast_n) / (1.0 - p.fast_turnover) - f64::from(fast0))
            / (native_fast - f64::from(fast0));
        assert!(fast_back > 0.7, "{fast_back}");
        assert!(s.slow_n > slow0 && f64::from(s.slow_n) < 0.9 * native_slow);
        assert!(s.allows_kg_ha(&p, &c) > worn * 1.15);
        // Long rest brings both back toward native ground, never past it.
        for year in 46..400 {
            s.turn(&p, 1.0, year);
        }
        let again = FieldSoil::native(&p, 1.0);
        assert!((s.slow_n - again.slow_n).abs() / again.slow_n < 0.01);
        assert!((s.fast_n - again.fast_n).abs() / again.fast_n < 0.01);
    }

    #[test]
    fn a_field_remembers_its_last_harvests_only() {
        let (p, c) = (params(), crop());
        let mut s = FieldSoil::native(&p, 1.0);
        for year in 0..12 {
            s.harvested(&c, 500.0, 500.0, Limit::Season, year);
        }
        assert_eq!(s.record.len(), RECORD_KEPT);
        assert_eq!(s.record[0].year, 12 - RECORD_KEPT as i32);
        assert_eq!(s.last_harvest_year(), Some(11));
    }

    #[test]
    fn an_older_save_replays_its_harvests_from_native_ground() {
        let (p, c) = (params(), crop());
        let fresh = FieldSoil::replayed(&p, &c, 1.0, 0);
        assert_eq!(fresh, FieldSoil::native(&p, 1.0));
        let worked = FieldSoil::replayed(&p, &c, 1.0, 10);
        assert!(worked.fast_n < fresh.fast_n && worked.slow_n < fresh.slow_n);
        assert!(worked.record.is_empty());
    }
}
