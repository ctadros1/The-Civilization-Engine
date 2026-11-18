//! The ground refuse tips lie on (ADR-0038's amendment of 2026-11-18): a tip a polity opens
//! keeps its footprint from building, plots, fields, pits and ditches, as a well's ground is kept.
//! The tip itself, what lies on it and who keeps it, is the polity's (`civ_agents::polity::Tip`);
//! only its ground is the land's.

use civ_core::PermanentId;

use crate::fields::RectCm;

/// A tip's ground: the tip, by its permanent id, and the rectangle its refuse lies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TipGround {
    pub id: PermanentId,
    pub rect: RectCm,
}

/// The ground of every tip, in the order opened. Saved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tips {
    pub list: Vec<TipGround>,
}

impl Tips {
    /// Whether any tip's ground lies within `gap_cm` of `rect`.
    pub fn near(&self, rect: &RectCm, gap_cm: i32) -> bool {
        self.list.iter().any(|t| t.rect.near(rect, gap_cm))
    }

    /// What is wrong with saved tip ground, if anything: ids repeated, or a footprint of no area.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut ids: Vec<PermanentId> = self.list.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        if ids.windows(2).any(|p| p[0] == p[1]) {
            out.push("two tips' ground share an id".to_owned());
        }
        if self.list.iter().any(|t| t.rect.w <= 0 || t.rect.h <= 0) {
            out.push("a tip's ground has no area".to_owned());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn a_tips_ground_is_near_what_overlaps_or_comes_within_the_gap() {
        let tips = Tips {
            list: vec![TipGround {
                id: id(1),
                rect: RectCm {
                    x: 1000,
                    y: 1000,
                    w: 500,
                    h: 500,
                },
            }],
        };
        let at = |x, y| RectCm {
            x,
            y,
            w: 100,
            h: 100,
        };
        assert!(tips.near(&at(1200, 1200), 0), "on it");
        assert!(!tips.near(&at(1600, 1200), 0), "beside it, clear");
        assert!(tips.near(&at(1600, 1200), 200), "within the gap");
        assert!(tips.problems().is_empty());
        let twice = Tips {
            list: vec![tips.list[0], tips.list[0]],
        };
        assert_eq!(twice.problems().len(), 1);
    }
}
