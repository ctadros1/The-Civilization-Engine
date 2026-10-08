//! Norms at work (M4c slice AG, step two, ADR-0016 §4; research 06-05). Each person takes a state
//! of each norm content names when first seen on the first of a month, or when first asked to
//! abide by one: an endorsement drawn by a key and pulled toward their parents', a threshold, and
//! what they believe others do, learned from their household's adults or else the founders'
//! custom (06-05 §5.4: children and newcomers learn through those they can reach). At the hearth,
//! companions now and then say whether their household paid its last levy, and a listener's
//! belief closes a share of the gap to what they heard. Abiding by a law the gathering passed
//! weighs each person's own state of it, never the settlement's true rate.

use super::*;
use crate::norm::{NormKind, NormState, draw_endorse, draw_threshold, id_key, learned, points};

/// Purpose tag for a person's draws of their state of a norm.
pub const PURPOSE_NORM: u64 = 0x6e6f_726d_7374_6531; // "normste1"
/// Purpose tag for a companion telling of their household's last levy.
pub const PURPOSE_NORM_TELL: u64 = 0x6e6f_726d_7465_6c31; // "normtel1"

impl Population {
    /// `id`'s state of each norm, taken now if they hold none: the endorsement drawn by a key and
    /// pulled toward their parents' (as they hold it), a threshold, and what they believe others
    /// do, the mean of what their household's others believe, or else the norm's prior.
    pub(super) fn ensure_norm_state(&mut self, ctx: &Ctx, id: PermanentId) {
        let Some(p) = self.person(id) else {
            return;
        };
        let (mother, father, household) = (p.mother, p.father, p.household);
        for (k, def) in ctx.catalog.norms.iter().enumerate() {
            let k = k as u16;
            if self.norms.state(id, k).is_some() {
                continue;
            }
            let held = |q: Option<PermanentId>| q.and_then(|q| self.norms.state(q, k));
            let (m, f) = (
                held(mother).map(|s| s.endorse),
                held(father).map(|s| s.endorse),
            );
            let others: Vec<f64> = self
                .household(household)
                .map(|h| h.members.as_slice())
                .unwrap_or_default()
                .iter()
                .filter(|&&q| q != id)
                .filter_map(|&q| self.norms.state(q, k))
                .map(|s| f64::from(s.expect))
                .collect();
            let expect = if others.is_empty() {
                def.expect_prior
            } else {
                others.iter().sum::<f64>() / others.len() as f64
            };
            let mut rng = Rng64::from_key(&[ctx.seed, PURPOSE_NORM, id.get(), id_key(&def.id)]);
            let endorse = draw_endorse(m, f, def, &mut rng);
            let threshold = draw_threshold(def, &mut rng);
            self.norms.insert(NormState {
                holder: id,
                norm: k,
                endorse,
                expect: expect as f32,
                threshold,
                heard: 0,
            });
        }
    }

    /// The first of the month: the states of those no longer here are let go, and everyone else
    /// holds one of each norm, taken in id order so that parents hold theirs before their
    /// children draw.
    pub(super) fn norm_month(&mut self, ctx: &Ctx) {
        if ctx.catalog.norms.is_empty() {
            return;
        }
        let kept: Vec<NormState> = self
            .norms
            .states
            .iter()
            .copied()
            .filter(|s| self.person(s.holder).is_some())
            .collect();
        self.norms.states = kept;
        let acts: Vec<_> = self
            .norms
            .acts
            .iter()
            .copied()
            .filter(|a| self.household(a.household).is_some())
            .collect();
        self.norms.acts = acts;
        let mut ids: Vec<PermanentId> = self.people.iter().map(|(_, p)| p.id).collect();
        ids.sort_unstable();
        for id in ids {
            self.ensure_norm_state(ctx, id);
        }
    }

    /// `a` and `b` keep company at the hearth (ADR-0016 §4): each may say whether their household
    /// paid its last levy, by the norm's chance, and the other's belief of what households do
    /// closes a share of the gap. Who says what is fixed first, so the order of the two changes
    /// nothing. They tell truly; nobody hides what their household kept back (not built).
    pub(crate) fn share_norms(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        let day = ctx.now.day_index();
        let mut said: Vec<(PermanentId, u16, f64)> = Vec::new();
        for (teller, listener) in [(a, b), (b, a)] {
            let Some(act) = self
                .person(teller)
                .and_then(|p| self.norms.act(p.household))
            else {
                continue;
            };
            let Some(x) = act.share_paid() else {
                continue;
            };
            for (k, def) in ctx.catalog.norms.iter().enumerate() {
                if def.kind != NormKind::AbideByLaws || day - act.day > def.tell_days {
                    continue;
                }
                let key = [
                    ctx.seed,
                    PURPOSE_NORM_TELL,
                    teller.get(),
                    listener.get(),
                    id_key(&def.id),
                    ctx.now.minutes() as u64,
                ];
                if Rng64::from_key(&key).next_f64() < def.share {
                    said.push((listener, k as u16, x));
                }
            }
        }
        for (listener, k, x) in said {
            self.norms.told += 1;
            let rate = ctx.catalog.norms[usize::from(k)].learn_rate;
            if let Some(s) = self.norms.state_mut(listener, k) {
                s.expect = learned(f64::from(s.expect), x, rate) as f32;
                s.heard = s.heard.saturating_add(1);
                self.norms.taken += 1;
            }
        }
    }

    /// The points `person` adds for abiding by a law the gathering passed: their own state of each
    /// norm that asks it (06-05 §5.2).
    pub(super) fn norm_points(&self, ctx: &Ctx, person: PermanentId) -> f64 {
        ctx.catalog
            .norms
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == NormKind::AbideByLaws)
            .filter_map(|(k, d)| Some(points(self.norms.state(person, k as u16)?, d)))
            .sum()
    }
}
