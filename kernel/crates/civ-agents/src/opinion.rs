//! Opinion (M4c slice AG, ADR-0016 §4): each adult holds a position, 0 against to 1 for, on the
//! questions content names (one per policy template that asks one), with a salience. Its anchor
//! is what their household's own lot makes of the policy, recomputed on the first of each month;
//! the position is pulled toward it slowly, and moved by what companions at the hearth say, as
//! x′ = (1−a)x + a·m with a gated by salience, trust and disagreement (research 06-04 §1.4:
//! Friedkin–Johnsen persistence; without anchors opinions collapse to consensus, §2). There is no
//! negative influence in v0. Nothing here reads a world total or another person's record but
//! what they say.

use civ_core::PermanentId;

/// One person's position on one question (ADR-0016 §4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// Who holds it.
    pub holder: PermanentId,
    /// The question, by the policy template that asks it (index in the catalog's policies).
    pub policy: u16,
    /// Where they stand: 0 against, 1 for.
    pub x: f32,
    /// What their household's own lot makes of it, 0 to 1, as last worked out.
    pub anchor: f32,
    /// How much it matters to them now, 0 to 1: more while a law on it is in force or before the
    /// gathering.
    pub salience: f32,
    /// The day the anchor was last worked out.
    pub since: i64,
    /// What they have heard said of it and taken in, since they first held it.
    pub heard: u32,
}

/// Everyone's positions (ADR-0016 §4).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Opinion {
    /// Positions, in (holder, policy) order.
    pub positions: Vec<Position>,
    /// Positions told at the hearth, and those taken in (a measure, for the smoke).
    pub told: u64,
    pub taken: u64,
}

impl Opinion {
    fn find(&self, holder: PermanentId, policy: u16) -> Result<usize, usize> {
        self.positions
            .binary_search_by(|p| (p.holder, p.policy).cmp(&(holder, policy)))
    }

    /// `holder`'s position on the question of template `policy`, if they hold one.
    pub fn position(&self, holder: PermanentId, policy: u16) -> Option<&Position> {
        self.find(holder, policy).ok().map(|i| &self.positions[i])
    }

    /// The same, to change.
    pub fn position_mut(&mut self, holder: PermanentId, policy: u16) -> Option<&mut Position> {
        self.find(holder, policy)
            .ok()
            .map(|i| &mut self.positions[i])
    }

    /// Holds `p`, in its place.
    pub fn insert(&mut self, p: Position) {
        match self.find(p.holder, p.policy) {
            Ok(i) => self.positions[i] = p,
            Err(i) => self.positions.insert(i, p),
        }
    }

    /// What `holder` holds, in question order.
    pub fn held_by(&self, holder: PermanentId) -> &[Position] {
        let from = self.positions.partition_point(|p| p.holder < holder);
        let to = self.positions.partition_point(|p| p.holder <= holder);
        &self.positions[from..to]
    }
}

/// How opinion moves (the people profile's `[opinion]` table; content API 41). Every value is a
/// design prior (research 06-04 §3.2), not a historical estimate.
#[derive(Clone, Debug, PartialEq)]
pub struct OpinionParams {
    /// The chance someone tells a companion at the hearth where they stand on a question, at full
    /// salience (06-04 §3.2: 0.5 politically meaningful exposures a person a week, 0.05–5).
    pub share: f64,
    /// The share of the distance to what they heard a listener moves at full salience and trust
    /// (06-04 §3.2: η 0.03, 0.005–0.15).
    pub eta: f64,
    /// The disagreement over which taking it in falls away, smoothly (06-04 §3.2: ε 0.25,
    /// 0.10–0.60).
    pub epsilon: f64,
    /// Days to halve the distance to the household's own lot (06-04 §3.2: 2 years, 0.25–10).
    pub anchor_half_life_days: f64,
    /// Points of forecast gain at which the household's lot leans 0.73 for it (a logistic scale;
    /// a tuning value).
    pub anchor_points: f64,
    /// Points a full position's distance from the household's lot adds to a stance at the
    /// gathering (a tuning value).
    pub w_position: f64,
    /// Salience while a law on the question is in force or before the gathering, and otherwise.
    pub salience_live: f64,
    pub salience_idle: f64,
    /// Below this age, susceptibility is `youth_factor` times (06-04 §3.2: doubled between 12
    /// and 30).
    pub youth_until: f64,
    pub youth_factor: f64,
}

impl OpinionParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> OpinionParams {
        OpinionParams {
            share: 0.0015,
            eta: 0.03,
            epsilon: 0.25,
            anchor_half_life_days: 730.0,
            anchor_points: 1.0,
            w_position: 2.0,
            salience_live: 1.0,
            salience_idle: 0.3,
            youth_until: 30.0,
            youth_factor: 2.0,
        }
    }
}

/// What a household's lot makes of a policy, 0 to 1, from the points of its forecast gain.
pub fn anchor_of(gain_points: f64, params: &OpinionParams) -> f64 {
    let s = params.anchor_points.max(1e-9);
    1.0 / (1.0 + (-gain_points / s).exp())
}

/// Position `x` after `days` of being pulled toward anchor `z` (the anchor's half-life).
pub fn pulled(x: f64, z: f64, days: f64, params: &OpinionParams) -> f64 {
    let keep = 0.5f64.powf(days.max(0.0) / params.anchor_half_life_days.max(1e-9));
    z + (x - z) * keep
}

/// How far toward `m` one at `x` moves on hearing it (06-04 §1.4): η × salience × trust × a
/// smooth disagreement gate, doubled for the young.
pub fn taken_in(
    x: f64,
    m: f64,
    salience: f64,
    trust: f64,
    young: bool,
    params: &OpinionParams,
) -> f64 {
    let gate = (-(m - x).abs() / params.epsilon.max(1e-9)).exp();
    let youth = if young { params.youth_factor } else { 1.0 };
    (params.eta * salience.clamp(0.0, 1.0) * trust.clamp(0.0, 1.0) * gate * youth).clamp(0.0, 1.0)
}

/// Where they stand, in words: "for", "leaning for", "undecided", "leaning against", "against".
pub fn lean_words(x: f64) -> &'static str {
    match x {
        x if x >= 0.75 => "for",
        x if x >= 0.58 => "leaning for",
        x if x > 0.42 => "undecided",
        x if x > 0.25 => "leaning against",
        _ => "against",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_position_is_pulled_to_its_anchor_by_the_half_life() {
        let p = OpinionParams::core();
        assert!((pulled(1.0, 0.0, 730.0, &p) - 0.5).abs() < 1e-9);
        assert!((pulled(0.2, 0.2, 99.0, &p) - 0.2).abs() < 1e-9);
        assert!((anchor_of(0.0, &p) - 0.5).abs() < 1e-9);
        assert!(anchor_of(3.0, &p) > 0.9 && anchor_of(-3.0, &p) < 0.1);
    }

    #[test]
    fn talk_moves_a_little_and_less_the_further_apart() {
        let p = OpinionParams::core();
        let near = taken_in(0.5, 0.6, 1.0, 1.0, false, &p);
        let far = taken_in(0.1, 0.9, 1.0, 1.0, false, &p);
        assert!(near > far && near <= p.eta);
        assert!((taken_in(0.5, 0.6, 1.0, 1.0, true, &p) - 2.0 * near).abs() < 1e-12);
        assert_eq!(
            taken_in(0.5, 0.6, 1.0, 0.0, false, &p),
            0.0,
            "nobody trusted moves nobody"
        );
        // About 23 accepted exposures at full weight halve a gap (06-04 §3.2's arithmetic).
        let (mut x, m) = (0.0, 0.1);
        for _ in 0..23 {
            x += p.eta * (m - x);
        }
        assert!((x - 0.05).abs() < 0.002, "{x}");
    }

    #[test]
    fn positions_are_kept_in_holder_order() {
        let id = |n| PermanentId::from_raw(n).expect("nonzero");
        let mut o = Opinion::default();
        for (h, k) in [(3, 1), (1, 0), (3, 0)] {
            o.insert(Position {
                holder: id(h),
                policy: k,
                x: 0.5,
                anchor: 0.5,
                salience: 0.3,
                since: 0,
                heard: 0,
            });
        }
        assert_eq!(o.held_by(id(3)).len(), 2);
        assert!(o.position(id(1), 0).is_some() && o.position(id(1), 1).is_none());
        assert!(
            o.positions
                .windows(2)
                .all(|w| (w[0].holder, w[0].policy) < (w[1].holder, w[1].policy))
        );
        assert_eq!(lean_words(0.8), "for");
        assert_eq!(lean_words(0.5), "undecided");
        assert_eq!(lean_words(0.1), "against");
    }
}
