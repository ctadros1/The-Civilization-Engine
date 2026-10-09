//! Norms (M4c slice AG, step two, ADR-0016 §4; research 06-05): a norm is a content template
//! naming a prescription the kernel knows how to act on, and each person holds their own state of
//! it: a private endorsement, drawn at birth and pulled toward their parents', and an empirical
//! expectation, what they believe others do, moved only by what companions tell them of their own
//! households' acts (06-05 §1.1, §5.4: Bicchieri's endorsement and empirical expectation kept
//! apart, beliefs updated from evidence, never from the settlement's true rate). Their threshold
//! sets how much others' doing it moves them (§5.2: heterogeneous conditional compliance), so a
//! principled few can hold to a norm others have let go, and others follow only the crowd.
//!
//! The one norm built is "what the gathering decides binds": it adds points for abiding by a law
//! the gathering passed (paying a levy, keeping a curfew), and companions tell one another whether
//! their household paid its last levy. The normative expectation (what others think one ought to
//! do) is not built: nothing yet says what anyone approves of (06-05 §5.4: a statement is not a
//! deed). Nothing here reads a world total.

use civ_core::PermanentId;
use civ_core::rng::Rng64;

use crate::crime::logistic;
use crate::demography::normal;

/// What a norm prescribes, as far as the kernel knows how to act on it: the code a norm file's
/// `does` names (content API 42).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormKind {
    /// Abide by what the gathering decided: pay its levy, keep its curfew.
    AbideByLaws,
}

impl NormKind {
    pub const ALL: [NormKind; 1] = [NormKind::AbideByLaws];

    /// The name a norm file uses in `does`.
    pub fn name(self) -> &'static str {
        match self {
            NormKind::AbideByLaws => "abide_by_laws",
        }
    }

    pub fn from_name(s: &str) -> Option<NormKind> {
        NormKind::ALL.into_iter().find(|k| k.name() == s)
    }
}

/// A norm template (content kind `norm`). Every number is a design prior (research 06-05 §2.2:
/// engineering priors for sensitivity analysis, not estimates).
#[derive(Clone, Debug, PartialEq)]
pub struct NormDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// What it says, in words: "what the gathering decides binds everyone".
    pub statement: String,
    pub kind: NormKind,
    /// The log-odds mean and spread of a founder's endorsement, and how much of its parents' mean
    /// a child takes on (as the objection to taking is drawn, ADR-0015 §2).
    pub endorse_mean: f64,
    pub endorse_sd: f64,
    pub heritability: f64,
    /// What a founder believes of others before anyone has told them anything, 0 to 1: the
    /// custom the founders brought with them.
    pub expect_prior: f64,
    /// The share of the gap to what they were told a listener closes on one account (06-05 §2.2:
    /// 0.02–0.30 for one credible observation).
    pub learn_rate: f64,
    /// The chance a companion at the hearth tells of their household's last act, a session.
    pub share: f64,
    /// Days after an act it is still worth telling.
    pub tell_days: i64,
    /// Each person's threshold on what they believe others do is drawn evenly between these
    /// (06-05 §2.2: 0.2–0.9), with the width of the smooth step at it.
    pub threshold_low: f64,
    pub threshold_high: f64,
    pub width: f64,
    /// Points for abiding, for a full endorsement and for a full activation by others' doing it.
    pub w_endorse: f64,
    pub w_expect: f64,
}

/// One person's state of one norm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormState {
    /// Who holds it.
    pub holder: PermanentId,
    /// The norm, by index in the catalog's norms.
    pub norm: u16,
    /// How far they hold it themselves, 0 to 1.
    pub endorse: f32,
    /// The share of households they believe abide by it, 0 to 1.
    pub expect: f32,
    /// Where others' doing it starts to move them, 0 to 1.
    pub threshold: f32,
    /// Accounts they have taken in since they first held it.
    pub heard: u32,
}

/// What a household did at its last levy, the season's paying and keeping back counted by thresh:
/// what its members can tell of it. A household that could not pay is not counted: the custom
/// does not ask what a household has not got (the norm's exception, 06-05 §5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevyAct {
    pub household: PermanentId,
    /// The day of the last levy counted.
    pub day: i64,
    pub paid: u16,
    pub kept: u16,
}

impl LevyAct {
    /// The share paid, 0 to 1, if anything was decided.
    pub fn share_paid(&self) -> Option<f64> {
        let all = u32::from(self.paid) + u32::from(self.kept);
        (all > 0).then(|| f64::from(self.paid) / f64::from(all))
    }
}

/// Everyone's norm states, and what each household last did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Norms {
    /// States, in (holder, norm) order.
    pub states: Vec<NormState>,
    /// Acts, in household order.
    pub acts: Vec<LevyAct>,
    /// Accounts told at the hearth, and those taken in (a measure, for the smoke).
    pub told: u64,
    pub taken: u64,
}

/// Days after which a household's next levy starts a new season's count.
pub const SEASON_DAYS: i64 = 120;

impl Norms {
    fn find(&self, holder: PermanentId, norm: u16) -> Result<usize, usize> {
        self.states
            .binary_search_by(|s| (s.holder, s.norm).cmp(&(holder, norm)))
    }

    /// `holder`'s state of norm `norm`, if they hold one.
    pub fn state(&self, holder: PermanentId, norm: u16) -> Option<&NormState> {
        self.find(holder, norm).ok().map(|i| &self.states[i])
    }

    /// The same, to change.
    pub fn state_mut(&mut self, holder: PermanentId, norm: u16) -> Option<&mut NormState> {
        self.find(holder, norm).ok().map(|i| &mut self.states[i])
    }

    /// Holds `s`, in its place.
    pub fn insert(&mut self, s: NormState) {
        match self.find(s.holder, s.norm) {
            Ok(i) => self.states[i] = s,
            Err(i) => self.states.insert(i, s),
        }
    }

    /// What `holder` holds, in norm order.
    pub fn held_by(&self, holder: PermanentId) -> &[NormState] {
        let from = self.states.partition_point(|s| s.holder < holder);
        let to = self.states.partition_point(|s| s.holder <= holder);
        &self.states[from..to]
    }

    /// What `household` last did at a levy.
    pub fn act(&self, household: PermanentId) -> Option<&LevyAct> {
        self.acts
            .binary_search_by_key(&household, |a| a.household)
            .ok()
            .map(|i| &self.acts[i])
    }

    /// `household` paid (or kept back) a levy on `day`: counted in this season's act, or a new
    /// season's if the last was long ago.
    pub fn note_levy(&mut self, household: PermanentId, day: i64, paid: bool) {
        let i = match self.acts.binary_search_by_key(&household, |a| a.household) {
            Ok(i) => i,
            Err(i) => {
                self.acts.insert(
                    i,
                    LevyAct {
                        household,
                        day,
                        paid: 0,
                        kept: 0,
                    },
                );
                i
            }
        };
        let a = &mut self.acts[i];
        if day - a.day > SEASON_DAYS {
            (a.paid, a.kept) = (0, 0);
        }
        a.day = day;
        if paid {
            a.paid = a.paid.saturating_add(1);
        } else {
            a.kept = a.kept.saturating_add(1);
        }
    }
}

/// A stable key for content id `id`, so a norm's draws do not move when other content is added
/// (ADR-0016: draws keyed by content id). FNV-1a.
pub fn id_key(id: &str) -> u64 {
    id.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// How far others' doing it moves someone with `threshold`, who believes `expect` of them: a
/// smooth step at the threshold (06-05 §5.2).
pub fn activation(expect: f64, threshold: f64, width: f64) -> f64 {
    logistic((expect - threshold) / width.max(1e-6))
}

/// The points `s` adds for abiding by norm `def`: their own endorsement, and others' doing it as
/// far as it moves them (06-05 §5.2: μV + γA).
pub fn points(s: &NormState, def: &NormDef) -> f64 {
    def.w_endorse * f64::from(s.endorse)
        + def.w_expect * activation(f64::from(s.expect), f64::from(s.threshold), def.width)
}

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    (p / (1.0 - p)).ln()
}

/// An endorsement of `def`, 0 to 1, drawn from `rng`: a founder's from the content's spread, a
/// child's pulled toward the mean of its parents' (06-05 §1.3: transmitted by learning).
pub fn draw_endorse(
    mother: Option<f32>,
    father: Option<f32>,
    def: &NormDef,
    rng: &mut Rng64,
) -> f32 {
    let sd = def.endorse_sd.max(1e-6);
    let standard = |e: f32| (logit(f64::from(e)) - def.endorse_mean) / sd;
    let parents: Vec<f64> = [mother, father]
        .into_iter()
        .flatten()
        .map(standard)
        .collect();
    let z = if parents.is_empty() {
        normal(rng)
    } else {
        let h = def.heritability.clamp(0.0, 1.0);
        let mid = parents.iter().sum::<f64>() / parents.len() as f64;
        h * mid + (1.0 - h * h / 2.0).max(0.0).sqrt() * normal(rng)
    };
    logistic(def.endorse_mean + sd * z) as f32
}

/// A threshold for `def`, drawn evenly over its range.
pub fn draw_threshold(def: &NormDef, rng: &mut Rng64) -> f32 {
    let (lo, hi) = (def.threshold_low, def.threshold_high.max(def.threshold_low));
    (lo + (hi - lo) * rng.next_f64()) as f32
}

/// What someone believes of others after an account `x` (the share a household paid), closing
/// `rate` of the gap (06-05 §5.4: b ← b + α·w·(x − b)).
pub fn learned(expect: f64, x: f64, rate: f64) -> f64 {
    (expect + rate.clamp(0.0, 1.0) * (x - expect)).clamp(0.0, 1.0)
}

/// How firmly someone holds a norm, in words.
pub fn endorse_words(e: f64) -> &'static str {
    match e {
        e if e >= 0.75 => "holds firmly",
        e if e >= 0.5 => "holds",
        e if e >= 0.25 => "doubts",
        _ => "does not hold",
    }
}

/// What someone believes others do, in words.
pub fn expect_words(x: f64) -> &'static str {
    match x {
        x if x >= 0.85 => "nearly every household",
        x if x >= 0.6 => "most households",
        x if x >= 0.4 => "about half the households",
        x if x >= 0.15 => "few households",
        _ => "hardly any household",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn binds() -> NormDef {
        NormDef {
            id: "core:norm/gathering_binds".to_owned(),
            name: "the gathering binds".to_owned(),
            description: String::new(),
            statement: "what the gathering decides binds everyone".to_owned(),
            kind: NormKind::AbideByLaws,
            endorse_mean: 0.4,
            endorse_sd: 1.0,
            heritability: 0.5,
            expect_prior: 0.85,
            learn_rate: 0.1,
            share: 0.001,
            tell_days: 365,
            threshold_low: 0.2,
            threshold_high: 0.9,
            width: 0.1,
            w_endorse: 1.0,
            w_expect: 1.0,
        }
    }

    #[test]
    fn others_doing_it_moves_those_whose_threshold_it_passes() {
        let d = binds();
        let s = |endorse, expect, threshold| NormState {
            holder: PermanentId::from_raw(1).expect("nonzero"),
            norm: 0,
            endorse,
            expect,
            threshold,
            heard: 0,
        };
        // The crowd moves a follower; a principled one abides whatever they believe.
        let follower_now = points(&s(0.1, 0.9, 0.5), &d);
        let follower_later = points(&s(0.1, 0.2, 0.5), &d);
        assert!(
            follower_now - follower_later > 0.9,
            "{follower_now} {follower_later}"
        );
        let principled = points(&s(0.95, 0.2, 0.5), &d);
        assert!(principled > follower_later + 0.8);
        assert!((activation(0.5, 0.5, 0.1) - 0.5).abs() < 1e-12);
        // One account closes a tenth of the gap.
        assert!((learned(0.8, 0.0, 0.1) - 0.72).abs() < 1e-12);
    }

    #[test]
    fn endorsement_takes_after_the_parents_and_thresholds_span_their_range() {
        let d = binds();
        let mut rng = Rng64::seed_from_u64(7);
        let n = 4000;
        let founders: f64 = (0..n)
            .map(|_| f64::from(draw_endorse(None, None, &d, &mut rng)))
            .sum::<f64>()
            / f64::from(n);
        assert!((founders - 0.58).abs() < 0.05, "{founders}");
        let low: f64 = (0..n)
            .map(|_| f64::from(draw_endorse(Some(0.1), Some(0.1), &d, &mut rng)))
            .sum::<f64>()
            / f64::from(n);
        assert!(low < founders - 0.15, "{low} {founders}");
        let t: Vec<f32> = (0..n).map(|_| draw_threshold(&d, &mut rng)).collect();
        assert!(t.iter().all(|&t| (0.2..=0.9).contains(&t)));
    }

    #[test]
    fn a_household_s_act_is_counted_by_season() {
        let id = |n| PermanentId::from_raw(n).expect("nonzero");
        let mut n = Norms::default();
        n.note_levy(id(4), 200, true);
        n.note_levy(id(4), 203, false);
        n.note_levy(id(2), 201, true);
        assert_eq!(n.act(id(4)).and_then(LevyAct::share_paid), Some(0.5));
        assert_eq!(n.acts[0].household, id(2), "kept in household order");
        n.note_levy(id(4), 200 + 365, true);
        assert_eq!(n.act(id(4)).map(|a| (a.paid, a.kept)), Some((1, 0)));
        assert_ne!(id_key("core:norm/a"), id_key("core:norm/b"));
        assert_eq!(endorse_words(0.8), "holds firmly");
        assert_eq!(expect_words(0.5), "about half the households");
    }
}
