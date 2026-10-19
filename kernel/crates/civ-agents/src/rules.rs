//! Rules for building (M6b slice BH; ADR-0022 §12): how people weigh a rule a gathering might
//! make about the buildings begun under it, and whether a building keeps one. The rules
//! themselves are policy templates ([`crate::polity::RuleDef`]).

use civ_land::Building;

/// How people weigh a rule for building (the people profile's `[rules]`, content API 93). Every
/// value is a design prior unless cited.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuleParams {
    /// Years a household remembers a building it lost to fire or saw burn (research 11-09 §2.3:
    /// a personal disaster memory of 2-20 years).
    pub memory_years: f64,
    /// The share of what fire would take from its buildings a household believes a rule averts.
    pub avoided_share: f64,
    /// Years over which a household weighs a rule: what it believes averted a year, and what its
    /// next building would cost more under it, spread over these.
    pub horizon_years: f64,
}

impl Default for RuleParams {
    fn default() -> Self {
        Self {
            memory_years: 10.0,
            avoided_share: 0.5,
            horizon_years: 20.0,
        }
    }
}

/// Whether the covering of building `spec` does not burn: fired tile (ADR-0022 §10).
pub fn roof_unburnt(spec: &civ_grammar::BuildingSpec) -> bool {
    spec.materials
        .get(usize::from(civ_grammar::frame::frame_materials::COVERING))
        .is_some_and(|m| m.contains("tile"))
}

/// The kind of the walls of building `spec` of a program with rules `rules` (ADR-0022 §10): a
/// frame building's as designed, 0 (wattle and daub) for a hut's.
pub fn wall_kind(spec: &civ_grammar::BuildingSpec, rules: &civ_grammar::ProgramRules) -> u8 {
    match rules {
        civ_grammar::ProgramRules::Frame(_) => {
            spec.params[civ_grammar::frame::frame_params::WALL_KIND].clamp(0, 7) as u8
        }
        _ => 0,
    }
}

/// The strength in compression of a mass wall of kind `kind` (1 rubble, 2 mud brick, 3 fired
/// brick) in a program with rules `rules`, pascals: the program's own figure (ADR-0031 §1), or
/// rubble's for a kind it does not name.
pub fn masonry_strength_pa(rules: &civ_grammar::ProgramRules, kind: i32) -> f64 {
    match rules {
        civ_grammar::ProgramRules::Frame(r) => {
            r.masonry[(kind.clamp(1, 3) - 1) as usize].strength_pa
        }
        _ => 1.5e6,
    }
}

/// The least distance between two plots' rectangles, metres.
pub fn gap_m(a: &civ_land::RectCm, b: &civ_land::RectCm) -> f64 {
    let dx = (a.x - (b.x + b.w)).max(b.x - (a.x + a.w)).max(0);
    let dy = (a.y - (b.y + b.h)).max(b.y - (a.y + a.h)).max(0);
    f64::from(dx).hypot(f64::from(dy)) / 100.0
}

/// Whether a plot `rect` lies within `within_m` of the plot of any standing building but
/// `except` among `buildings`, plots found by `plot_of`.
pub fn near_another(
    rect: &civ_land::RectCm,
    within_m: f64,
    buildings: &[Building],
    except: Option<civ_core::PermanentId>,
    plot_of: &dyn Fn(&Building) -> Option<civ_land::RectCm>,
) -> bool {
    buildings
        .iter()
        .filter(|b| Some(b.id) != except && b.standing())
        .filter_map(plot_of)
        .any(|r| gap_m(rect, &r) < within_m)
}

/// Whether a plot `rect` lies within `within_m` of an open well among `wells` (M8c slice CW): a
/// household's midden lies at its home.
pub fn near_open_well(
    rect: &civ_land::RectCm,
    within_m: f64,
    wells: &civ_land::wells::Wells,
) -> bool {
    wells
        .list
        .iter()
        .filter(|w| w.is_open())
        .any(|w| gap_m(rect, &w.rect) < within_m)
}

/// Whether a plot `rect` lies within `within_m` of the plot of a standing home (`is_home`) of a
/// household other than `household` among `buildings`, plots found by `plot_of` (M8c slice CW).
pub fn near_others_homes(
    rect: &civ_land::RectCm,
    within_m: f64,
    buildings: &[Building],
    household: civ_core::PermanentId,
    plot_of: &dyn Fn(&Building) -> Option<civ_land::RectCm>,
    is_home: &dyn Fn(&Building) -> bool,
) -> bool {
    buildings
        .iter()
        .filter(|b| b.household != household && b.standing() && is_home(b))
        .filter_map(plot_of)
        .any(|r| gap_m(rect, &r) < within_m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plots_apart_by_their_nearest_edges() {
        let a = civ_land::RectCm {
            x: 0,
            y: 0,
            w: 400,
            h: 400,
        };
        let b = civ_land::RectCm {
            x: 700,
            y: 800,
            w: 100,
            h: 100,
        };
        // 3 m across and 4 m down from a's corner: 5 m.
        assert!((gap_m(&a, &b) - 5.0).abs() < 1e-9);
        assert_eq!(gap_m(&a, &a), 0.0);
    }
}
