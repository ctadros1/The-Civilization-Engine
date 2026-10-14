//! Irrigation ditches a household digs to water its own field (M6b slice BG; ADR-0022 §11; the
//! M6 fire brief §1.11): how it lays one out, what digging it takes, and when it weighs one. The
//! channel and its water are `civ_land::ditches`'.

/// How a household lays out and digs an irrigation ditch (M6b slice BG; content API 92). Every
/// value is a design prior unless cited.
#[derive(Clone, Debug, PartialEq)]
pub struct DitchPlanParams {
    /// Bottom width and depth, metres, and side slope, horizontal per vertical (research 11-12
    /// §2.3: 1.5:1 in ordinary earth).
    pub bottom_m: f64,
    pub depth_m: f64,
    pub side_run: f64,
    /// Hours to dig and form a cubic metre of channel (research 11-12 §2.2: 0.8-1.2 m³ a
    /// worker-day for excavating and accurately forming canals, here an eight-hour day).
    pub h_per_m3: f64,
    /// The longest ditch a household lays out, metres.
    pub max_length_m: f64,
    /// Years a household reckons a ditch serves its field, over which its digging is weighed.
    pub life_years: f64,
    /// A household weighs a ditch for a field whose last crop got less than this share of the
    /// water it needed, as its people saw the crop thirst.
    pub short_share: f64,
}

impl Default for DitchPlanParams {
    fn default() -> Self {
        Self {
            bottom_m: 0.3,
            depth_m: 0.4,
            side_run: 1.5,
            h_per_m3: 8.0,
            max_length_m: 400.0,
            life_years: 20.0,
            short_share: 0.9,
        }
    }
}

impl DitchPlanParams {
    /// The hours digging a ditch `length_m` long takes.
    pub fn dig_hours(&self, length_m: f64) -> f64 {
        length_m.max(0.0)
            * self.depth_m
            * (self.bottom_m + self.side_run * self.depth_m)
            * self.h_per_m3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hundred_metre_ditch_takes_its_channel_at_a_cubic_metre_a_day() {
        let p = DitchPlanParams::default();
        // 100 m of 0.3 m bottom, 0.4 m deep at 1.5:1 is 36 m³: 36 eight-hour days (research
        // 11-12 §2.5 A: a 100 m ditch of 40 m³ takes 33-50 worker-days).
        assert!((p.dig_hours(100.0) - 36.0 * 8.0).abs() < 1e-9);
    }
}
