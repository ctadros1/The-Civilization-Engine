//! Ideologies (M4c slice AG, step four, ADR-0016 §4; research 06-04 §1.1, §4.2, §6.1): a content
//! record of the problem it explains, its moral commitments (how it tilts what its holders hold
//! dear), the laws it proposes (the law pipeline's own templates), and its legitimacy story in
//! words. Who holds one is a per-person record with whom they had it from: a founder who brought
//! it, a parent, or a companion at the hearth who spoke of it and was believed (06-04 §1.4:
//! exposure is not acceptance; acceptance is gated by trust and by how well it fits what the
//! listener holds dear). Someone who holds one weighs the laws its program names among their
//! moves when the problem it explains is before the village (06-04 §1.2: an explanation shapes
//! the response to a problem when it is felt), and weighs every law with its commitments beside
//! their own values.
//!
//! An ideology is not news: holding one is kept apart from the claims of word of mouth (ADR-0016
//! §3), which are let go once stale. Nobody yet gives one up, no new one is made (06-04 §4.2's
//! recombination is not built), and none reaches anyone but by birth, a founder or the hearth;
//! the god tool that introduces one is slice AJ's.

use civ_core::PermanentId;

use crate::polity::IssueKind;

/// An ideology (content kind `ideology`, content API 44). Every number is a design prior.
#[derive(Clone, Debug, PartialEq)]
pub struct IdeologyDef {
    pub id: String,
    /// What it is called: "common provision".
    pub name: String,
    pub description: String,
    /// The problem it explains.
    pub explains: IssueKind,
    /// Its legitimacy story, in words: "what the village gathers, the village keeps against a
    /// lean year".
    pub legitimacy: String,
    /// How it tilts what its holders hold dear: (index in the catalog's values, −1 to 1), in
    /// value order.
    pub commitments: Vec<(u16, f32)>,
    /// The laws it proposes: indices in the catalog's policies, in order.
    pub program: Vec<u16>,
    /// The share of founders (and of newcomers with no parents here) who bring it.
    pub founders: f64,
    /// The chance a holder speaks of it to a companion at the hearth, a session.
    pub share: f64,
    /// The chance one who hears it, trusting the teller fully and holding dear just what it is
    /// committed to, takes it up.
    pub adopt: f64,
    /// The chance a child takes up what a parent holds.
    pub inherit: f64,
    /// Points it adds, for a holder, to proposing a law its program names.
    pub w_program: f64,
}

/// One person's holding of an ideology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Holding {
    pub holder: PermanentId,
    /// The ideology, by index in the catalog's ideologies.
    pub ideology: u16,
    /// The day they took it up.
    pub since: i64,
    /// Who they had it from: a parent or a companion; none for one who brought it.
    pub from: Option<PermanentId>,
}

/// Who holds what, in (holder, ideology) order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ideologies {
    pub held: Vec<Holding>,
    /// Times one was spoken of at the hearth, and taken up (a measure, for the smoke).
    pub told: u64,
    pub taken: u64,
    /// The highest person id given their start (what a founder brought or a child took from a
    /// parent): ids only grow, so anyone above it is new.
    pub seen: u64,
    /// The laws proposed as an ideology's program, by law id, with the ideology (the creed it was
    /// proposed under), in law order.
    pub creeds: Vec<(PermanentId, u16)>,
}

impl Ideologies {
    fn find(&self, holder: PermanentId, ideology: u16) -> Result<usize, usize> {
        self.held
            .binary_search_by(|h| (h.holder, h.ideology).cmp(&(holder, ideology)))
    }

    /// Whether `holder` holds ideology `ideology`.
    pub fn holds(&self, holder: PermanentId, ideology: u16) -> bool {
        self.find(holder, ideology).is_ok()
    }

    /// Takes up `h`, unless they already hold it. Returns whether it is new to them.
    pub fn take_up(&mut self, h: Holding) -> bool {
        match self.find(h.holder, h.ideology) {
            Ok(_) => false,
            Err(i) => {
                self.held.insert(i, h);
                true
            }
        }
    }

    /// The creed law `law` was proposed under, if any.
    pub fn creed_of(&self, law: PermanentId) -> Option<u16> {
        self.creeds
            .binary_search_by_key(&law, |c| c.0)
            .ok()
            .map(|i| self.creeds[i].1)
    }

    /// Law `law` was proposed under ideology `ideology`.
    pub fn note_creed(&mut self, law: PermanentId, ideology: u16) {
        match self.creeds.binary_search_by_key(&law, |c| c.0) {
            Ok(i) => self.creeds[i].1 = ideology,
            Err(i) => self.creeds.insert(i, (law, ideology)),
        }
    }

    /// What `holder` holds, in ideology order.
    pub fn held_by(&self, holder: PermanentId) -> &[Holding] {
        let from = self.held.partition_point(|h| h.holder < holder);
        let to = self.held.partition_point(|h| h.holder <= holder);
        &self.held[from..to]
    }
}

/// How well an ideology's `commitments` fit what someone holds dear, −1 (against all of it) to 1
/// (just what they hold), by `held` (their value of each, by index).
pub fn fit(commitments: &[(u16, f32)], held: &dyn Fn(u16) -> f64) -> f64 {
    let total: f64 = commitments.iter().map(|&(_, c)| f64::from(c).abs()).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let along: f64 = commitments
        .iter()
        .map(|&(k, c)| f64::from(c) * held(k))
        .sum();
    (along / total).clamp(-1.0, 1.0)
}

/// The chance someone takes up `def` on hearing it from one they trust `trust` (0 to 1), when it
/// fits what they hold dear by `fit` (−1 to 1): none unless it fits at all, the content's whole
/// chance for a perfect fit (06-04 §1.4: exposure is not acceptance; acceptance is gated by trust
/// and by agreement, and those who hold its commitments less than most do not take it up from
/// talk).
pub fn adopt_chance(def: &IdeologyDef, trust: f64, fit: f64) -> f64 {
    (def.adopt * trust.clamp(0.0, 1.0) * fit.clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provision() -> IdeologyDef {
        IdeologyDef {
            id: "core:ideology/common_provision".to_owned(),
            name: "common provision".to_owned(),
            description: String::new(),
            explains: IssueKind::FoodShort,
            legitimacy: "what the village gathers, the village keeps".to_owned(),
            commitments: vec![(0, 0.5), (2, 0.3)],
            program: vec![0],
            founders: 0.1,
            share: 0.001,
            adopt: 0.3,
            inherit: 0.5,
            w_program: 1.0,
        }
    }

    #[test]
    fn it_takes_with_those_it_fits_and_who_trust_the_teller() {
        let d = provision();
        let fond = |k: u16| if k == 0 || k == 2 { 1.0 } else { 0.0 };
        let averse = |k: u16| if k == 0 || k == 2 { -1.0 } else { 0.0 };
        assert!((fit(&d.commitments, &fond) - 1.0).abs() < 1e-9);
        assert!((fit(&d.commitments, &averse) + 1.0).abs() < 1e-9);
        assert!((adopt_chance(&d, 1.0, 1.0) - 0.3).abs() < 1e-9);
        assert!((adopt_chance(&d, 1.0, 0.5) - 0.15).abs() < 1e-9);
        assert_eq!(
            adopt_chance(&d, 1.0, 0.0),
            0.0,
            "nor when it does not fit at all"
        );
        assert_eq!(
            adopt_chance(&d, 1.0, -1.0),
            0.0,
            "never against all they hold"
        );
        assert_eq!(
            adopt_chance(&d, 0.0, 1.0),
            0.0,
            "never from one they do not trust"
        );
        assert_eq!(fit(&[], &fond), 0.0);
    }

    #[test]
    fn holdings_are_kept_in_holder_order_once_each() {
        let id = |n| PermanentId::from_raw(n).expect("nonzero");
        let mut ideas = Ideologies::default();
        let h = |holder, ideology| Holding {
            holder: id(holder),
            ideology,
            since: 3,
            from: None,
        };
        assert!(ideas.take_up(h(3, 1)));
        assert!(ideas.take_up(h(1, 0)));
        assert!(ideas.take_up(h(3, 0)));
        assert!(!ideas.take_up(h(3, 0)), "once");
        assert_eq!(ideas.held_by(id(3)).len(), 2);
        assert!(ideas.holds(id(1), 0) && !ideas.holds(id(1), 1));
    }
}
