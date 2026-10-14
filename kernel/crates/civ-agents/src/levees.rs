//! Flood banks people choose to build together (M6b slice BG; ADR-0022 §11): what a household
//! remembers a flood cost it, how a bank is laid out to keep that water out, and what its work
//! takes. The physical bank and its borrow pit are `civ_land::banks`'.

/// How people lay out and build a flood bank, and how long they remember what a flood cost
/// them (M6b slice BG; content API 88). Every value is a design prior unless cited.
#[derive(Clone, Debug, PartialEq)]
pub struct LeveeParams {
    /// Years a household remembers what a flood cost it.
    pub memory_years: f64,
    /// Years a household reckons a bank keeps water out, over which its work is spread.
    pub life_years: f64,
    /// Metres a bank's crest stands above the highest water remembered at the places it keeps.
    pub freeboard_m: f64,
    /// The least and the most height a bank is laid out to, metres above its ground.
    pub min_height_m: f64,
    pub max_height_m: f64,
    /// Crest width, metres, and side slope, horizontal per vertical (research 11-12 §2.5 C:
    /// 1 m and 2:1 for a village levee).
    pub crest_width_m: f64,
    pub side_run: f64,
    /// Compacted volume per cubic metre dug in place (research 11-12 §2.4: 0.90, 0.85-1.00).
    pub compacted_per_bank: f64,
    /// Loose volume per cubic metre dug in place (research 11-12 §2.4: 1.20, 1.10-1.40).
    pub loose_per_bank: f64,
    /// Mass of a cubic metre of loose earth, kg (research 11-12 §2.3: 1,300-1,500 dry).
    pub loose_kg_per_m3: f64,
    /// A basket's load, kg (research 11-12 §2.4: 20, 15-25).
    pub carry_kg: f64,
    /// Minutes to fill and tip one basket, besides walking it there and back.
    pub load_min: f64,
    /// Hours to spread and tamp a compacted cubic metre (research 11-12 §2.4: 3 m³ an
    /// 8-hour day, 2-5).
    pub place_h_per_m3: f64,
    /// How deep the borrow pit beside a bank is dug, metres.
    pub pit_depth_m: f64,
    /// Metres of bank each worker needs to work at once: a narrow front holds few (research
    /// 11-12 §1.2).
    pub front_m: f64,
    /// A finished bank is mended once its crest has sunk or worn this many metres below its
    /// design, or water has done this much seepage damage to it (0 to 1), or it has given way
    /// (research 11-12 §4.1: inspect, refill, compact). Design priors.
    pub mend_crest_m: f64,
    pub mend_piping: f64,
}

impl Default for LeveeParams {
    fn default() -> Self {
        Self {
            memory_years: 10.0,
            life_years: 20.0,
            freeboard_m: 0.3,
            min_height_m: 0.3,
            max_height_m: 3.0,
            crest_width_m: 1.0,
            side_run: 2.0,
            compacted_per_bank: 0.9,
            loose_per_bank: 1.2,
            loose_kg_per_m3: 1400.0,
            carry_kg: 20.0,
            load_min: 0.5,
            place_h_per_m3: 8.0 / 3.0,
            pit_depth_m: 1.0,
            front_m: 5.0,
            mend_crest_m: 0.1,
            mend_piping: 0.1,
        }
    }
}

impl LeveeParams {
    /// The hours a cubic metre of bank, dug in place, takes to dig at `dig_h_per_m3`, carry
    /// `haul_m` each way a basket at a time at `speed_m_per_min`, and spread and tamp:
    /// each charged apart (research 11-12 §1.2: digging, carrying and placing are distinct
    /// operations, not one digging rate).
    pub fn hours_per_m3(&self, dig_h_per_m3: f64, haul_m: f64, speed_m_per_min: f64) -> f64 {
        let loads = self.loose_per_bank * self.loose_kg_per_m3 / self.carry_kg.max(1e-6);
        let haul_min = loads * (self.load_min + 2.0 * haul_m.max(0.0) / speed_m_per_min.max(1e-6));
        dig_h_per_m3 + haul_min / 60.0 + self.compacted_per_bank * self.place_h_per_m3
    }

    /// The hours a compacted cubic metre of bank takes to dig out where water has seeped
    /// through it and spread and tamp again in place: no carrying.
    pub fn rework_h_per_m3(&self, dig_h_per_m3: f64) -> f64 {
        dig_h_per_m3 + self.place_h_per_m3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cubic_metre_of_bank_charges_digging_carrying_and_placing_apart() {
        let p = LeveeParams::default();
        // 1.2 × 1,400 / 20 = 84 baskets; 15 m each way at 60 m/min is 0.5 min, plus 0.5 to fill.
        let h = p.hours_per_m3(8.0, 15.0, 60.0);
        assert!((h - (8.0 + 84.0 / 60.0 + 0.9 * 8.0 / 3.0)).abs() < 1e-9);
        // Carrying further costs more; digging and placing are the same.
        assert!(p.hours_per_m3(8.0, 60.0, 60.0) > h);
    }
}
