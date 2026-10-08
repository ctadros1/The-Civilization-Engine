//! Opinion at work (M4c slice AG, ADR-0016 §4): on the first of each month every adult's anchor
//! on each question is worked out afresh from what their household's own lot makes of the policy
//! (the forecast a sponsor would weigh, of the law in force or else the template's middle level),
//! and their position is pulled toward it by the time since; at the hearth, companions say where
//! they stand, each by a keyed chance, and a listener takes in what a trusted companion says,
//! less the further apart they are (research 06-04 §1.4). At a gathering, how far talk has moved
//! a member from their household's lot weighs in their stance.

use super::*;
use crate::opinion::{Position, anchor_of, pulled, taken_in};
use crate::polity::{IssueKind, LawStatus, MoveOption, PolicyDef};

/// Purpose tag for a companion saying where they stand on a question.
pub const PURPOSE_OPINION: u64 = 0x6f70_696e_696f_6e31; // "opinion1"

/// The move whose forecast is a household's lot on template `def` (number `k`) in polity
/// `polity`: the law of it in force or before the gathering, or else the template's middle level.
fn level_of(def: &PolicyDef, k: u16, polity: &crate::polity::Polity) -> MoveOption {
    let live = polity
        .laws
        .iter()
        .rfind(|l| l.policy == k && matches!(l.status, LawStatus::InForce | LawStatus::Proposed));
    let middle = |n: usize| n / 2;
    MoveOption {
        policy: k,
        levy_share: live.map_or_else(
            || {
                def.levy_shares
                    .get(middle(def.levy_shares.len()))
                    .copied()
                    .unwrap_or(0.0)
            },
            |l| f64::from(l.levy_share),
        ),
        issue: def.answers.first().copied().unwrap_or(IssueKind::FoodShort),
        nominee: None,
        sanction: live.map_or_else(
            || {
                def.bundles
                    .get(middle(def.bundles.len()))
                    .copied()
                    .unwrap_or_default()
            },
            |l| l.sanction,
        ),
        hours: live.map_or_else(|| def.hours.first().copied().unwrap_or((0, 0)), |l| l.hours),
        body: None,
        ends: None,
        own_gain: 0.0,
        followers_gain: 0.0,
        support: 0.5,
    }
}

impl Population {
    /// The first of the month (ADR-0016 §4): each adult's anchor on each question content names
    /// is what their household's lot makes of it now, and their position is pulled toward it
    /// for the time since; someone new holds their household's lot. Positions of those no longer
    /// here are let go.
    pub(super) fn opinion_month(&mut self, ctx: &Ctx) {
        let (now, params) = (ctx.now, ctx.params);
        let (op, pp) = (&params.opinion, &params.polity);
        let day = now.day_index();
        let kept: Vec<Position> = self
            .opinion
            .positions
            .iter()
            .copied()
            .filter(|p| self.person(p.holder).is_some())
            .collect();
        self.opinion.positions = kept;
        let policies = &ctx.catalog.policies;
        let questions: Vec<u16> = policies
            .iter()
            .enumerate()
            .filter(|(_, d)| d.question.is_some())
            .map(|(k, _)| k as u16)
            .collect();
        if questions.is_empty() {
            return;
        }
        for pi in 0..self.polities.len() {
            let settlement = self.polities[pi].settlement;
            let mut households: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            households.sort_unstable();
            let adults = self.members_of(settlement, now, params);
            let fc = self.forecasts(ctx, pi, &households, &adults);
            let mut anchors: Vec<(PermanentId, u16, f64, f64)> = Vec::new();
            for &k in &questions {
                let polity = &self.polities[pi];
                let def = &policies[usize::from(k)];
                let m = level_of(def, k, polity);
                let live = polity.laws.iter().any(|l| {
                    l.policy == k && matches!(l.status, LawStatus::InForce | LawStatus::Proposed)
                });
                let salience = if live {
                    op.salience_live
                } else {
                    op.salience_idle
                };
                for &(a, h) in &adults {
                    // Their household's lot, and what it does to what they hold dear.
                    let points =
                        pp.w_gain * fc.gain(&m, h, policies, params) + self.value_points(ctx, a, k);
                    let z = anchor_of(points, op);
                    anchors.push((a, k, z, salience));
                }
            }
            for (a, k, z, salience) in anchors {
                match self.opinion.position_mut(a, k) {
                    Some(p) => {
                        p.x = pulled(f64::from(p.x), z, (day - p.since) as f64, op) as f32;
                        p.anchor = z as f32;
                        p.salience = salience as f32;
                        p.since = day;
                    }
                    None => self.opinion.insert(Position {
                        holder: a,
                        policy: k,
                        x: z as f32,
                        anchor: z as f32,
                        salience: salience as f32,
                        since: day,
                        heard: 0,
                    }),
                }
            }
        }
    }

    /// `a` and `b` keep company at the hearth (ADR-0016 §4): each says where they stand on a
    /// question they hold, by the content's chance at its salience, and the other takes in what
    /// they say by their trust in them (regard), less the further apart they are. Who says what
    /// is fixed first, so the order of the two changes nothing.
    pub(crate) fn share_opinion(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        let op = &ctx.params.opinion;
        let day = ctx.now.day_index();
        let mut said: Vec<(PermanentId, PermanentId, u16, f64)> = Vec::new();
        for (teller, listener) in [(a, b), (b, a)] {
            for p in self.opinion.held_by(teller) {
                let key = [
                    ctx.seed,
                    PURPOSE_OPINION,
                    teller.get(),
                    listener.get(),
                    u64::from(p.policy),
                    ctx.now.minutes() as u64,
                ];
                if Rng64::from_key(&key).next_f64() < op.share * f64::from(p.salience) {
                    said.push((listener, teller, p.policy, f64::from(p.x)));
                }
            }
        }
        for (listener, teller, k, m) in said {
            self.opinion.told += 1;
            let trust = self
                .ties
                .regard(listener, teller, day, &ctx.params.ties)
                .clamp(0.0, 1.0);
            let young = self
                .person(listener)
                .is_some_and(|p| p.age_years(ctx.now) < op.youth_until);
            let Some(pos) = self.opinion.position_mut(listener, k) else {
                continue;
            };
            let x = f64::from(pos.x);
            let a = taken_in(x, m, f64::from(pos.salience), trust, young, op);
            if a > 0.0 {
                pos.x = (x + a * (m - x)).clamp(0.0, 1.0) as f32;
                pos.heard = pos.heard.saturating_add(1);
                self.opinion.taken += 1;
            }
        }
    }

    /// What talk at the hearth adds to `person`'s stance on a law of template `policy`, points:
    /// how far their position has moved from their household's lot (ADR-0016 §4).
    pub(super) fn opinion_points(&self, person: PermanentId, policy: u16, w_position: f64) -> f64 {
        self.opinion
            .position(person, policy)
            .map_or(0.0, |p| w_position * f64::from(p.x - p.anchor))
    }
}
