//! Deposits found (ADR-0010 §1; M3b slice Q): a settlement comes to know a body that shows at
//! the surface when one of its members walks near it. A buried body waits for an earthwork to cut
//! into it. Nothing here decides where anyone goes; finding is what walking brings.

use super::*;

/// How far from a body's edge a walker sees it showing, metres: an exposure in a bank or a spread
/// on the surface, seen from the way (a tuning value; the research gives no figure, and looking
/// reveals indications, not a body's outline, 03-05 §1.3).
pub const FIND_M: f64 = 50.0;

/// A settlement's knowledge of a deposit: who found it, and when.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepositKnown {
    /// The settlement that knows it.
    pub settlement: PermanentId,
    /// The deposit.
    pub deposit: PermanentId,
    /// Who found it.
    pub finder: PermanentId,
    /// When.
    pub at: SimTime,
}

/// The distance from `p` to the segment from `a` to `b`, metres.
fn to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p.0 - (a.0 + t * dx)).hypot(p.1 - (a.1 + t * dy))
}

impl Population {
    /// Whether settlement `settlement` knows deposit `deposit`.
    pub fn knows_deposit(&self, settlement: PermanentId, deposit: PermanentId) -> bool {
        self.deposits_known
            .iter()
            .any(|k| k.settlement == settlement && k.deposit == deposit)
    }

    /// Person `who` of household `household` walked the route `points` (metres): their
    /// settlement finds every body showing within [`FIND_M`] of the way that it did not know, and
    /// the chronicle says so. A household of no settlement keeps no such record.
    pub(super) fn look_for_deposits(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        points: &[(f32, f32)],
    ) {
        let Some(settlement) = self.household(household).and_then(|h| h.settlement) else {
            return;
        };
        let route: Vec<(f64, f64)> = points
            .iter()
            .map(|&(x, y)| (f64::from(x), f64::from(y)))
            .collect();
        let near = |at: (f64, f64), reach: f64| match route.as_slice() {
            [] => false,
            [only] => (at.0 - only.0).hypot(at.1 - only.1) <= reach,
            r => r.windows(2).any(|w| to_segment(at, w[0], w[1]) <= reach),
        };
        let found: Vec<(PermanentId, u16, (f64, f64))> = ctx
            .land
            .deposits
            .iter()
            .filter(|d| d.body.exposed && d.left_kg() > 0.0)
            .filter_map(|d| {
                let at = (d.body.at_cm.0 as f64 / 100.0, d.body.at_cm.1 as f64 / 100.0);
                let reach = f64::from(d.body.radius_cm) / 100.0 + FIND_M;
                near(at, reach).then_some((d.id, d.body.good, at))
            })
            .filter(|&(id, _, _)| !self.knows_deposit(settlement, id))
            .collect();
        for (deposit, good, at) in found {
            self.deposits_known.push(DepositKnown {
                settlement,
                deposit,
                finder: who,
                at: ctx.now,
            });
            let what = ctx
                .catalog
                .goods
                .get(usize::from(good))
                .map_or_else(|| "a deposit".to_owned(), |g| g.name.to_lowercase());
            self.chronicle_push(
                ctx.now,
                ChronicleKind::DepositFound,
                vec![who],
                Some(settlement),
                Some((at.0 as f32, at.1 as f32)),
                0.0,
                format!("{what} showing at the surface"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::to_segment;

    #[test]
    fn the_distance_to_a_segment_is_to_its_nearest_point() {
        assert!((to_segment((5.0, 3.0), (0.0, 0.0), (10.0, 0.0)) - 3.0).abs() < 1e-12);
        assert!((to_segment((-4.0, 3.0), (0.0, 0.0), (10.0, 0.0)) - 5.0).abs() < 1e-12);
        assert!((to_segment((1.0, 1.0), (2.0, 2.0), (2.0, 2.0)) - 2f64.sqrt()).abs() < 1e-12);
    }
}
