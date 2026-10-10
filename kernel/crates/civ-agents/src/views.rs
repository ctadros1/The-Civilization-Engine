//! Views of another polity (M5c slice AT, ADR-0020 §3): what a person believes of a polity they
//! have heard of, shaped like a tie (ADR-0014): evidence for and against in three domains, the
//! prior included, fading back toward it. Only acts the person saw or heard of write it; a view is
//! never a cause by itself, and nothing reads it yet but the observer.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// How views of a polity are held (the people profile's `[relations]`, content API 60). The
/// half-life is ADR-0020 §3's five years (research 13-01 §3.3 tests 2, 5 and 20); the rest are
/// design priors.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationsParams {
    /// Evidence for and against in each domain before anything is seen (α = β).
    pub prior: f64,
    /// Days over which evidence beyond the prior halves.
    pub half_life_days: f64,
    /// Evidence that the other polity harms one's own from a day one saw its people work a place
    /// one's polity claims.
    pub seen_trespass: f64,
    /// The same from hearing of it told at the hearth.
    pub heard_trespass: f64,
    /// The chance someone tells a companion of a claim on a place their polity's law makes, or
    /// one of another polity's their household heard of, when the companion's household has not
    /// heard of it (content API 61).
    pub share_claims: f64,
    /// What a place a household has heard another polity claims is worth to it, as a share of
    /// what it would yield (content API 61).
    pub claimed_worth: f64,
}

impl RelationsParams {
    /// The core content's values, for tests.
    pub fn core() -> Self {
        RelationsParams {
            prior: 1.0,
            half_life_days: 1826.0,
            seen_trespass: 1.0,
            heard_trespass: 0.5,
            share_claims: 0.15,
            claimed_worth: 0.5,
        }
    }
}

/// A domain of a view (ADR-0020 §3). Its index is part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    /// It keeps its word: obligations due to one's polity or household, met or missed.
    KeepsWord = 0,
    /// It harms us: takings and breaches attributed to its people.
    HarmsUs = 1,
    /// It helps us: gifts and relief received.
    HelpsUs = 2,
}

/// How many domains a view has.
pub const DOMAINS: usize = 3;

impl Domain {
    /// Every domain, in index order.
    pub const ALL: [Domain; DOMAINS] = [Domain::KeepsWord, Domain::HarmsUs, Domain::HelpsUs];

    /// Its index in a view.
    pub fn index(self) -> usize {
        self as usize
    }

    /// What it asks, in words: "harms us".
    pub fn words(self) -> &'static str {
        match self {
            Domain::KeepsWord => "keeps its word",
            Domain::HarmsUs => "harms us",
            Domain::HelpsUs => "helps us",
        }
    }
}

/// What wrote a view. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewAct {
    /// One saw its people work a place one's polity claims, without leave.
    SawTrespass = 0,
    /// One heard of that told at the hearth.
    HeardTrespass = 1,
}

impl ViewAct {
    /// Every act, in code order.
    pub const ALL: [ViewAct; 2] = [ViewAct::SawTrespass, ViewAct::HeardTrespass];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The act numbered `code`.
    pub fn from_code(code: u8) -> Option<ViewAct> {
        ViewAct::ALL.get(usize::from(code)).copied()
    }

    /// The domain it writes, and whether for (true) or against.
    pub fn writes(self) -> (Domain, bool) {
        match self {
            ViewAct::SawTrespass | ViewAct::HeardTrespass => (Domain::HarmsUs, true),
        }
    }

    /// What happened, in words: "saw its people work a place our polity claims".
    pub fn words(self) -> &'static str {
        match self {
            ViewAct::SawTrespass => "saw its people work a place our polity claims",
            ViewAct::HeardTrespass => "heard its people worked a place our polity claims",
        }
    }
}

/// The act that last wrote a view, when, and how many times running.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewReason {
    pub act: ViewAct,
    pub day: i64,
    pub times: u16,
}

/// What a person believes of polity `polity`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// Whose view it is.
    pub polity: PermanentId,
    /// The day its evidence stands at.
    pub day: i64,
    /// Evidence for, by domain, the prior included.
    pub yes: [f32; DOMAINS],
    /// Evidence against, by domain, the prior included.
    pub no: [f32; DOMAINS],
    /// What last wrote it.
    pub reason: Option<ViewReason>,
}

impl View {
    /// No view yet of `polity` on `day`: the prior alone.
    pub fn new(polity: PermanentId, day: i64, params: &RelationsParams) -> View {
        let prior = params.prior as f32;
        View {
            polity,
            day,
            yes: [prior; DOMAINS],
            no: [prior; DOMAINS],
            reason: None,
        }
    }

    /// The view as it stands on `day`: its evidence faded toward the prior since it was last
    /// brought up to date.
    pub fn at(&self, day: i64, params: &RelationsParams) -> View {
        let days = (day - self.day) as f64;
        if days <= 0.0 {
            return *self;
        }
        let left = if params.half_life_days > 0.0 {
            (-std::f64::consts::LN_2 * days / params.half_life_days).exp()
        } else {
            0.0
        };
        let toward = |x: f32| (params.prior + (f64::from(x) - params.prior) * left) as f32;
        View {
            day,
            yes: self.yes.map(toward),
            no: self.no.map(toward),
            ..*self
        }
    }

    /// How far it leans toward "yes" in domain `d`: the mean of its beta, 0 to 1 (a half with no
    /// evidence).
    pub fn lean(&self, d: Domain) -> f64 {
        let (y, n) = (
            f64::from(self.yes[d.index()]),
            f64::from(self.no[d.index()]),
        );
        if y + n <= 0.0 { 0.5 } else { y / (y + n) }
    }

    /// Evidence beyond the prior in every domain together.
    pub fn evidence(&self, params: &RelationsParams) -> f64 {
        self.yes
            .iter()
            .chain(&self.no)
            .map(|&x| (f64::from(x) - params.prior).max(0.0))
            .sum()
    }
}

/// Evidence below which a view faded back to its prior is let go: no view.
const LET_GO: f64 = 0.01;

/// Everyone's views of polities, by person, each in polity order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Views {
    pub held: BTreeMap<PermanentId, Vec<View>>,
}

impl Views {
    /// `person` saw or heard of `units` of `act` by polity `polity` on `day`.
    pub fn record(
        &mut self,
        person: PermanentId,
        polity: PermanentId,
        act: ViewAct,
        units: f64,
        day: i64,
        params: &RelationsParams,
    ) {
        let list = self.held.entry(person).or_default();
        let at = match list.binary_search_by(|v| v.polity.cmp(&polity)) {
            Ok(at) => at,
            Err(at) => {
                list.insert(at, View::new(polity, day, params));
                at
            }
        };
        let v = &mut list[at];
        *v = v.at(day, params);
        let (domain, yes) = act.writes();
        let side = if yes { &mut v.yes } else { &mut v.no };
        side[domain.index()] += units as f32;
        v.reason = match v.reason {
            Some(r) if r.act == act => Some(ViewReason {
                day,
                times: r.times.saturating_add(1),
                ..r
            }),
            _ => Some(ViewReason { act, day, times: 1 }),
        };
    }

    /// What `person` believes of `polity` on `day`, if they hold a view of it.
    pub fn of(
        &self,
        person: PermanentId,
        polity: PermanentId,
        day: i64,
        params: &RelationsParams,
    ) -> Option<View> {
        let list = self.held.get(&person)?;
        let at = list.binary_search_by(|v| v.polity.cmp(&polity)).ok()?;
        Some(list[at].at(day, params))
    }

    /// Lets go of the views of those no longer here (`here` is false for them), and of views
    /// faded back to the prior by `day`.
    pub fn prune(
        &mut self,
        day: i64,
        params: &RelationsParams,
        here: impl Fn(PermanentId) -> bool,
    ) {
        self.held.retain(|&p, _| here(p));
        for list in self.held.values_mut() {
            list.retain(|v| v.at(day, params).evidence(params) >= LET_GO);
        }
        self.held.retain(|_, list| !list.is_empty());
    }

    /// What is wrong with the record, if anything: polities out of order or repeated, or evidence
    /// that is not finite or below nothing.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (p, list) in &self.held {
            if list.is_empty() || list.windows(2).any(|w| w[0].polity >= w[1].polity) {
                out.push(format!(
                    "person {p}'s views are empty, out of order or repeated"
                ));
            }
            for v in list {
                if v.yes
                    .iter()
                    .chain(&v.no)
                    .any(|x| !x.is_finite() || *x < 0.0)
                {
                    out.push(format!(
                        "person {p}'s view of {} holds a bad count",
                        v.polity
                    ));
                }
            }
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
    fn what_is_seen_leans_a_view_and_fades_back_to_none() {
        let params = RelationsParams::core();
        let mut views = Views::default();
        assert!(views.of(id(1), id(9), 0, &params).is_none());
        views.record(id(1), id(9), ViewAct::SawTrespass, 1.0, 12, &params);
        views.record(id(1), id(9), ViewAct::SawTrespass, 1.0, 12, &params);
        views.record(id(1), id(9), ViewAct::HeardTrespass, 0.5, 12, &params);
        let v = views.of(id(1), id(9), 12, &params).expect("a view");
        // Two seen and one heard on one day: 3.5 for against 1, the prior.
        assert!((v.lean(Domain::HarmsUs) - 3.5 / 4.5).abs() < 1e-6);
        assert!((v.lean(Domain::HelpsUs) - 0.5).abs() < 1e-9);
        assert_eq!(
            v.reason.map(|r| (r.act, r.times)),
            Some((ViewAct::HeardTrespass, 1))
        );
        // Five years on, half the evidence beyond the prior is left.
        let later = views.of(id(1), id(9), 12 + 1826, &params).expect("a view");
        assert!((later.evidence(&params) - 1.25).abs() < 1e-3);
        // Long after, it is let go; so are the views of one no longer here.
        views.prune(12 + 20 * 1826, &params, |_| true);
        assert!(views.held.is_empty());
        views.record(id(1), id(9), ViewAct::SawTrespass, 1.0, 10, &params);
        views.prune(10, &params, |p| p != id(1));
        assert!(views.held.is_empty());
        assert!(views.problems().is_empty());
    }

    #[test]
    fn acts_round_trip_their_codes() {
        for a in ViewAct::ALL {
            assert_eq!(ViewAct::from_code(a.code()), Some(a));
        }
        assert_eq!(ViewAct::from_code(2), None);
    }
}
