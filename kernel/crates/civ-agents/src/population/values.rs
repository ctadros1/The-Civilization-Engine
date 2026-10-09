//! Values at work (M4c slice AG, step three, ADR-0016 §4; research 06-04 §1.1, §1.7). Each person
//! takes what they hold of each value content names the first midnight they are here: drawn by a
//! key on the value's content id, pulled toward what their parents hold. What a law does to what
//! someone holds dear weighs, beside their household's lot, in their anchor on its question,
//! their stance at its gathering and how they weigh proposing it.

use super::*;
use crate::norm::id_key;
use crate::values::{Held, draw};

/// Purpose tag for a person's draw of a value.
pub const PURPOSE_VALUE: u64 = 0x7661_6c75_6573_3031; // "values01"

impl Population {
    /// What `id` holds of each value, taken now where they hold none yet.
    pub(super) fn ensure_values(&mut self, ctx: &Ctx, id: PermanentId) {
        let Some(p) = self.person(id) else {
            return;
        };
        let (mother, father) = (p.mother, p.father);
        for (k, def) in ctx.catalog.values.iter().enumerate() {
            let k = k as u16;
            if self.values.get(id, k).is_some() {
                continue;
            }
            let held = |q: Option<PermanentId>| q.and_then(|q| self.values.get(q, k));
            let mut rng = Rng64::from_key(&[ctx.seed, PURPOSE_VALUE, id.get(), id_key(&def.id)]);
            let v = draw(held(mother), held(father), def, &mut rng);
            self.values.insert(Held {
                holder: id,
                value: k,
                v,
            });
        }
    }

    /// The first of the month: what those no longer here held is let go, and everyone else holds
    /// each value, taken in id order so that parents hold theirs before their children draw.
    pub(super) fn values_month(&mut self, ctx: &Ctx) {
        if ctx.catalog.values.is_empty() {
            return;
        }
        let index = &self.index;
        self.values.held.retain(|h| index.contains_key(&h.holder));
        let mut ids: Vec<PermanentId> = self.people.iter().map(|(_, p)| p.id).collect();
        ids.sort_unstable();
        for id in ids {
            self.ensure_values(ctx, id);
        }
    }

    /// Midnight: anyone who holds no value or norm content names yet (the founders on the first
    /// night, a child the night after their birth, a newcomer the night they come) takes theirs,
    /// in id order so that parents hold theirs before their children draw.
    pub(super) fn take_new_holdings(&mut self, ctx: &Ctx) {
        let (values, norms) = (ctx.catalog.values.len(), ctx.catalog.norms.len());
        if values == 0 && norms == 0 {
            return;
        }
        let mut new: Vec<PermanentId> = self
            .people
            .iter()
            .map(|(_, p)| p.id)
            .filter(|&id| {
                (0..values as u16).any(|k| self.values.get(id, k).is_none())
                    || (0..norms as u16).any(|k| self.norms.state(id, k).is_none())
            })
            .collect();
        new.sort_unstable();
        for id in new {
            self.ensure_values(ctx, id);
            self.ensure_norm_state(ctx, id);
        }
    }

    /// The points what `person` holds dear adds to what they make of a law of template `policy`,
    /// with the commitments of the ideologies they hold (M4c slice AG, step four).
    pub(super) fn value_points(&self, ctx: &Ctx, person: PermanentId, policy: u16) -> f64 {
        ctx.catalog
            .policies
            .get(usize::from(policy))
            .map_or(0.0, |d| {
                self.values.points(person, &d.bears, &ctx.catalog.values)
            })
            + self.creed_points(ctx, person, policy)
    }
}
