//! How people lay out and weigh an enclosure (M6c slice BK, step two; ADR-0023 §6; research
//! 11-11): a palisade round the hearth taking in the homes within a reach, cut into sectors with
//! gates, its work, the years its posts stand, and what a household believes it keeps out. Every
//! value is a design prior unless cited.

/// How people lay out and weigh an enclosure (the people profile's `[enclosures]`, content API
/// 100).
#[derive(Clone, Debug, PartialEq)]
pub struct EnclosureParams {
    /// The longest a sector is laid out, metres (research 11-11 §2.2: a simulation sector of
    /// 10-25 m).
    pub sector_m: f64,
    /// Gates spread round the line.
    pub gates: u32,
    /// The work a metre of palisade takes, hours: felling, trimming and carrying its posts,
    /// trenching and setting them (11-11 §2.3 gives about five posts a metre and no labour total:
    /// a design prior).
    pub palisade_h_per_m: f64,
    /// Homes this near the hearth, metres, are taken inside the line.
    pub reach_m: f64,
    /// How far outside the farthest home taken in the line runs, metres, and the least radius a
    /// line is laid out at.
    pub margin_m: f64,
    pub min_radius_m: f64,
    /// Years a household reckons its posts stand, over which its share of the work is spread
    /// (11-11 §2.2: 5, 15 and 30 years as sensitivity scenarios; the middle).
    pub life_years: f64,
    /// The share of what people of other settlements took from a household inside the line that
    /// it believes the line would keep out.
    pub cover: f64,
    /// Metres of line still to be built that give one worker room.
    pub front_m: f64,
    /// Times a day a household's people pass through the line, each going round to a gate.
    pub crossings_a_day: f64,
    /// The share of a post's section rot takes at the foot in a year, in ground of wetness one
    /// (step three; 11-11 §2.2: 5, 15 and 30 years as scenarios for timber's life).
    pub rot_per_year: f64,
    /// The share rotted through at which a stretch is asked mending.
    pub mend_loss: f64,
    /// Days a household that believes people of other settlements took from it or raided it
    /// keeps the gate nearest its home shut at night.
    pub alarm_days: u32,
}

impl EnclosureParams {
    /// The core profile's values.
    pub fn core() -> Self {
        EnclosureParams {
            sector_m: 20.0,
            gates: 2,
            palisade_h_per_m: 6.0,
            reach_m: 120.0,
            margin_m: 6.0,
            min_radius_m: 15.0,
            life_years: 15.0,
            cover: 0.5,
            front_m: 4.0,
            crossings_a_day: 4.0,
            rot_per_year: 1.0 / 15.0,
            mend_loss: 0.5,
            alarm_days: 60,
        }
    }
}

impl Default for EnclosureParams {
    fn default() -> Self {
        EnclosureParams::core()
    }
}

/// The hours a year a household's people would spend going round to a gate of a ring of
/// `radius_m` with `gates` gates spread round it, walking at `speed_m_s`: each crossing of the
/// line goes round, on average, a quarter of the arc between two gates.
pub fn detour_h_a_year(p: &EnclosureParams, radius_m: f64, gates: u32, speed_m_s: f64) -> f64 {
    let arc = std::f64::consts::TAU * radius_m / f64::from(gates.max(1));
    p.crossings_a_day * 365.0 * (arc / 4.0) / speed_m_s.max(0.05) / 3600.0
}

/// The radius of a line taking in homes at `distances` metres from its centre: as far out as the
/// farthest within `reach_m`, and `margin_m` beyond, at least `min_radius_m`. None when no home
/// is within reach.
pub fn ring_radius(p: &EnclosureParams, distances: &[f64]) -> Option<f64> {
    let far = distances
        .iter()
        .copied()
        .filter(|d| d.is_finite() && *d <= p.reach_m)
        .fold(None, |m: Option<f64>, d| Some(m.map_or(d, |m| m.max(d))))?;
    Some((far + p.margin_m).max(p.min_radius_m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ring_takes_in_the_homes_within_reach_and_its_detour_grows_with_it() {
        let p = EnclosureParams::core();
        assert_eq!(ring_radius(&p, &[30.0, 80.0, 400.0]), Some(86.0));
        assert_eq!(ring_radius(&p, &[2.0]), Some(15.0));
        assert_eq!(ring_radius(&p, &[400.0]), None);
        let small = detour_h_a_year(&p, 30.0, 2, 1.2);
        let large = detour_h_a_year(&p, 90.0, 2, 1.2);
        assert!(small > 0.0 && (large / small - 3.0).abs() < 1e-9);
        assert!(detour_h_a_year(&p, 90.0, 4, 1.2) < large);
    }
}
