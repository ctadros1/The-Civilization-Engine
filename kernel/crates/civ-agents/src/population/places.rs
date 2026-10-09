//! How households come to know other settlements (ADR-0018 §4; M5a slice AM): by walking within
//! sight of them (in `start_walk`), by being told at the hearth, and by founding alongside them.

use super::*;
use crate::places::PlaceHow;

/// Purpose tag for the draw of whether a place is told at the hearth.
pub const PURPOSE_PLACE: u64 = 0x706c_6163_6530_3031; // "place001"

impl Population {
    /// `a` and `b` keep company at the hearth: each learns where the other is from, if it is
    /// another settlement, and each tells the other of the places their household knows that
    /// the other's does not, each by the content's chance (ADR-0018 §4).
    pub(crate) fn share_places(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        let home_of = |p: PermanentId| {
            let q = self.person(p)?;
            Some((q.household, self.household(q.household)?.settlement))
        };
        let (Some(ha), Some(hb)) = (home_of(a), home_of(b)) else {
            return;
        };
        let mut learnt: Vec<(PermanentId, PermanentId, PermanentId)> = Vec::new();
        for ((teller, (th, ts)), (listener, (lh, ls))) in [((a, ha), (b, hb)), ((b, hb), (a, ha))] {
            // Someone from elsewhere says where they are from.
            if let Some(s) = ts
                && Some(s) != ls
                && !self.known_places.knows(lh, s)
            {
                learnt.push((lh, s, teller));
            }
            for k in self.known_places.of(th) {
                if Some(k.settlement) == ls || self.known_places.knows(lh, k.settlement) {
                    continue;
                }
                let key = [
                    ctx.seed,
                    PURPOSE_PLACE,
                    teller.get(),
                    listener.get(),
                    k.settlement.get(),
                    ctx.now.minutes() as u64,
                ];
                if Rng64::from_key(&key).next_f64() < ctx.params.places.share_told {
                    learnt.push((lh, k.settlement, teller));
                }
            }
        }
        let day = ctx.now.day_index();
        for (household, settlement, from) in learnt {
            self.known_places
                .learn(household, settlement, day, PlaceHow::Told, Some(from));
        }
    }

    /// The founding groups `founded` came to the valley together and know where each other
    /// camped (the new-world choice of ADR-0018 §6).
    pub fn know_each_other(&mut self, founded: &[crate::found::Founded], day: i64) {
        for f in founded {
            for g in founded.iter().filter(|g| g.settlement != f.settlement) {
                for &h in &f.households {
                    self.known_places
                        .learn(h, g.settlement, day, PlaceHow::Founded, None);
                }
            }
        }
    }

    /// A walk through `points` by a member of `household` passes within sight of the homes of
    /// any other settlement still lived in: the household knows it (ADR-0018 §4).
    pub(crate) fn see_places(&mut self, ctx: &Ctx, household: PermanentId, points: &[(f32, f32)]) {
        if ctx.land.settlements.len() < 2 {
            return;
        }
        let home = self.household(household).and_then(|x| x.settlement);
        let day = ctx.now.day_index();
        for s in &ctx.land.settlements {
            if Some(s.id) != home
                && s.abandoned.is_none()
                && crate::places::distance_to_walk(points, s.hearth_m) <= ctx.params.places.sight_m
            {
                self.known_places
                    .learn(household, s.id, day, PlaceHow::Seen, None);
            }
        }
    }
}
