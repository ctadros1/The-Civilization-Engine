//! Ties between people (ADR-0014): what each person remembers of others, written only by acts
//! the engine records, and fading toward what they would expect of a stranger.
//!
//! A tie is one person's view of another; the other may hold none back. It keeps familiarity and
//! warmth, good and bad evidence in a few domains (research 04-06 §5.2: counts that decay toward a
//! prior), fear (written from M4b), and a balance of help received less help given, in hours.
//! Nothing is updated for all pairs: a tie changes only when its holder sees an act, and fades
//! only when it is next read (04-04 §5.3). Kinship, the household, a hire and a lease stay in
//! their own records (04-04 §1.1); a tie to a relative or an employer is a personal view like any
//! other.

use civ_core::PermanentId;

/// The domains a tie keeps evidence in (ADR-0014 §1). Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Domain {
    /// Gives, pays, shares.
    Provision,
    /// Does good work.
    Craft,
    /// Keeps agreements: wages, rent, what was lent.
    Word,
    /// What they proposed did what they said it would (from M4a slice Z).
    Counsel,
}

impl Domain {
    /// Every domain, in save order.
    pub const ALL: [Domain; DOMAINS] = [
        Domain::Provision,
        Domain::Craft,
        Domain::Word,
        Domain::Counsel,
    ];

    /// Its index in a tie's evidence.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Its name in content.
    pub fn name(self) -> &'static str {
        match self {
            Domain::Provision => "provision",
            Domain::Craft => "craft",
            Domain::Word => "word",
            Domain::Counsel => "counsel",
        }
    }

    /// The domain content names `name`.
    pub fn from_name(name: &str) -> Option<Domain> {
        Domain::ALL.into_iter().find(|d| d.name() == name)
    }
}

/// Number of evidence domains.
pub const DOMAINS: usize = 4;

/// The acts that write ties (ADR-0014 §2), each seen by the tie's holder. The engine names the
/// acts; content says what each writes. Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Act {
    /// Food came from the other's household in answer to the holder's household's ask.
    GiftReceived,
    /// The holder's household gave food to the other's in answer to an ask.
    GiftGiven,
    /// The other's workshop paid the holder's wages as agreed.
    WagesPaid,
    /// The holder's workshop paid the other for work done.
    WorkSeen,
    /// The other's household paid the holder's its share of a let field's grain.
    RentPaid,
    /// The other's household let the holder's a field, and was paid its share.
    LandLent,
    /// The holder's household and the other's traded.
    Traded,
    /// The holder learnt a technique working beside the other, by the hour.
    LearnedFrom,
    /// The holder kept company with the other at the hearth, by the hour.
    Hearth,
}

impl Act {
    /// Every act, in save order.
    pub const ALL: [Act; ACTS] = [
        Act::GiftReceived,
        Act::GiftGiven,
        Act::WagesPaid,
        Act::WorkSeen,
        Act::RentPaid,
        Act::LandLent,
        Act::Traded,
        Act::LearnedFrom,
        Act::Hearth,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The act numbered `code` in saves.
    pub fn from_code(code: u8) -> Option<Act> {
        Act::ALL.get(usize::from(code)).copied()
    }

    /// Its name in content.
    pub fn name(self) -> &'static str {
        match self {
            Act::GiftReceived => "gift_received",
            Act::GiftGiven => "gift_given",
            Act::WagesPaid => "wages_paid",
            Act::WorkSeen => "work_seen",
            Act::RentPaid => "rent_paid",
            Act::LandLent => "land_lent",
            Act::Traded => "traded",
            Act::LearnedFrom => "learned_from",
            Act::Hearth => "hearth",
        }
    }

    /// The act content names `name`.
    pub fn from_name(name: &str) -> Option<Act> {
        Act::ALL.into_iter().find(|a| a.name() == name)
    }
}

/// Number of acts.
pub const ACTS: usize = 9;

/// What one act writes into its holder's tie, for each unit of it (an act, or an hour for
/// [`Act::LearnedFrom`] and [`Act::Hearth`]). Authored in content as tuning values.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActWeights {
    /// Share of the gap between familiarity and 1 a unit closes.
    pub familiarity: f64,
    /// Share of the gap between warmth and 1 a unit closes.
    pub warmth: f64,
    /// The domain the act is evidence in, if any.
    pub domain: Option<Domain>,
    /// Good evidence a unit adds.
    pub good: f64,
    /// Bad evidence a unit adds.
    pub bad: f64,
}

/// How ties are kept (the people profile's `[ties]`, content API 30). Tuning values within the
/// ranges research 04-04, 04-06 and 04-11 propose; see the content for each.
#[derive(Clone, Debug, PartialEq)]
pub struct TieParams {
    /// Most ties a person keeps.
    pub room: usize,
    /// People at the hearth a session's company touches, at most.
    pub companions: usize,
    /// Good and bad evidence a stranger starts from, each.
    pub prior: f64,
    /// Days evidence takes to fade halfway back to the prior.
    pub evidence_half_life_days: f64,
    /// Days familiarity takes to fade by half.
    pub familiarity_half_life_days: f64,
    /// Days warmth takes to fade by half.
    pub warmth_half_life_days: f64,
    /// Days fear takes to fade by half.
    pub fear_half_life_days: f64,
    /// Days the balance of help takes to fade by half.
    pub help_half_life_days: f64,
    /// A tie owing or owed more hours of help than this is let go only when no other can be.
    pub hold_help_h: f64,
    /// Salience a unit of evidence beyond the prior adds, beside familiarity and warmth.
    pub salience_per_evidence: f64,
    /// The share of a session's company drawn from those present the person does not yet know;
    /// the rest is drawn from those they know, the better known the likelier.
    pub new_share: f64,
    /// Minutes further a household short of food walks to ask someone its members regard (by
    /// [`Tie::regard_at`], up to 1) than a stranger, among those who could give it as much.
    pub ask_known_min: f64,
    /// What each act writes, by [`Act`] index.
    pub acts: [ActWeights; ACTS],
}

impl Default for TieParams {
    fn default() -> Self {
        TieParams {
            room: 48,
            companions: 3,
            prior: 1.0,
            evidence_half_life_days: 180.0,
            familiarity_half_life_days: 180.0,
            warmth_half_life_days: 730.0,
            fear_half_life_days: 365.0,
            help_half_life_days: 1095.0,
            hold_help_h: 4.0,
            salience_per_evidence: 0.1,
            new_share: 0.15,
            ask_known_min: 30.0,
            acts: [ActWeights::default(); ACTS],
        }
    }
}

impl TieParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> TieParams {
        let mut p = TieParams::default();
        let w = |familiarity, warmth, domain, good| ActWeights {
            familiarity,
            warmth,
            domain,
            good,
            bad: 0.0,
        };
        p.acts = [
            w(0.2, 0.1, Some(Domain::Provision), 1.0),
            w(0.1, 0.02, None, 0.0),
            w(0.05, 0.0, Some(Domain::Word), 1.0),
            w(0.05, 0.0, Some(Domain::Craft), 0.5),
            w(0.05, 0.0, Some(Domain::Word), 1.0),
            w(0.05, 0.0, Some(Domain::Word), 1.0),
            w(0.1, 0.0, Some(Domain::Word), 0.5),
            w(0.02, 0.005, Some(Domain::Craft), 0.1),
            w(0.1, 0.01, None, 0.0),
        ];
        p
    }
}

/// Why a tie stands as it does: the act that last moved it, when, and how many times running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reason {
    /// The act.
    pub act: Act,
    /// The day it last happened.
    pub day: i64,
    /// How many times it has happened since it became the reason.
    pub times: u16,
}

impl Reason {
    /// The reason in words, of the other as the holder's inspector tells it ("gave their
    /// household food 3 times, last in May of year 4").
    pub fn words(&self) -> String {
        let what = match self.act {
            Act::GiftReceived => "gave their household food",
            Act::GiftGiven => "had food from their household",
            Act::WagesPaid => "paid their wages",
            Act::WorkSeen => "worked for their household",
            Act::RentPaid => "paid their household rent",
            Act::LandLent => "let their household a field",
            Act::Traded => "traded with their household",
            Act::LearnedFrom => "taught them",
            Act::Hearth => "kept them company at the hearth",
        };
        let date = civ_core::time::SimTime::from_minutes(self.day * 1440).date();
        let month = civ_land::weather::MONTH_NAMES[usize::from(date.month.clamp(1, 12)) - 1];
        let times = match self.times {
            0 | 1 => String::new(),
            2 => " twice".to_owned(),
            n => format!(" {n} times"),
        };
        format!("{what}{times}, last in {month} of year {}", date.year)
    }
}

/// One person's view of another (ADR-0014 §1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tie {
    /// Who it is a view of.
    pub to: PermanentId,
    /// The day its values stand at.
    pub day: i64,
    /// How well the holder knows them, 0–1.
    pub familiarity: f32,
    /// How warmly the holder feels toward them, 0–1.
    pub warmth: f32,
    /// Good evidence by domain, the prior included.
    pub good: [f32; DOMAINS],
    /// Bad evidence by domain, the prior included.
    pub bad: [f32; DOMAINS],
    /// Expected harm from crossing them, 0–1 (written from M4b).
    pub fear: f32,
    /// Hours of help received from them less help given to them.
    pub help_h: f32,
    /// What last moved it most.
    pub reason: Option<Reason>,
}

/// `value` after `days` at a half-life of `half_life` days.
fn faded(value: f64, days: f64, half_life: f64) -> f64 {
    if days <= 0.0 {
        return value;
    }
    if half_life <= 0.0 {
        return 0.0;
    }
    value * (-std::f64::consts::LN_2 * days / half_life).exp()
}

/// `value` (0–1) after `units` of an act closing `share` of its gap to 1 each.
fn closer(value: f64, share: f64, units: f64) -> f64 {
    if share <= 0.0 || units <= 0.0 {
        return value;
    }
    let left = (1.0 - share.min(1.0)).powf(units);
    1.0 - (1.0 - value) * left
}

impl Tie {
    /// A view of a stranger `to` on `day`.
    pub fn new(to: PermanentId, day: i64, params: &TieParams) -> Tie {
        let prior = params.prior as f32;
        Tie {
            to,
            day,
            familiarity: 0.0,
            warmth: 0.0,
            good: [prior; DOMAINS],
            bad: [prior; DOMAINS],
            fear: 0.0,
            help_h: 0.0,
            reason: None,
        }
    }

    /// The tie as it stands on `day`: each quantity faded at its own half-life since the day it
    /// was last brought up to date, evidence toward the prior and the rest toward nothing.
    pub fn at(&self, day: i64, params: &TieParams) -> Tie {
        let days = (day - self.day) as f64;
        if days <= 0.0 {
            return *self;
        }
        let toward_prior = |x: f32| {
            (params.prior
                + faded(
                    f64::from(x) - params.prior,
                    days,
                    params.evidence_half_life_days,
                )) as f32
        };
        Tie {
            day,
            familiarity: faded(
                f64::from(self.familiarity),
                days,
                params.familiarity_half_life_days,
            ) as f32,
            warmth: faded(f64::from(self.warmth), days, params.warmth_half_life_days) as f32,
            good: self.good.map(toward_prior),
            bad: self.bad.map(toward_prior),
            fear: faded(f64::from(self.fear), days, params.fear_half_life_days) as f32,
            help_h: faded(f64::from(self.help_h), days, params.help_half_life_days) as f32,
            ..*self
        }
    }

    /// Writes `units` of `act` seen on `day`, with `help_h` hours of help received (negative:
    /// given). The tie must stand at `day` ([`Tie::at`]).
    pub fn record(&mut self, act: Act, units: f64, help_h: f64, day: i64, params: &TieParams) {
        let w = params.acts[act as usize];
        self.familiarity = closer(f64::from(self.familiarity), w.familiarity, units) as f32;
        self.warmth = closer(f64::from(self.warmth), w.warmth, units) as f32;
        let writes = |w: &ActWeights| w.domain.is_some() && (w.good > 0.0 || w.bad > 0.0);
        if let (true, Some(d)) = (writes(&w), w.domain) {
            self.good[d.index()] += (w.good * units) as f32;
            self.bad[d.index()] += (w.bad * units) as f32;
        }
        self.help_h += help_h as f32;
        // The reason is the act last seen that wrote evidence; an act that wrote none (company,
        // a gift given) is the reason only until one does.
        self.reason = match self.reason {
            Some(r) if r.act == act => Some(Reason {
                day,
                times: r.times.saturating_add(1),
                ..r
            }),
            Some(r) if writes(&params.acts[r.act as usize]) && !writes(&w) => Some(r),
            _ => Some(Reason {
                act,
                day,
                times: 1,
            }),
        };
        self.day = day;
    }

    /// Evidence beyond the prior in every domain together.
    pub fn evidence(&self, params: &TieParams) -> f64 {
        let prior = 2.0 * params.prior * DOMAINS as f64;
        let total: f64 = self
            .good
            .iter()
            .chain(&self.bad)
            .map(|&x| f64::from(x))
            .sum();
        (total - prior).max(0.0)
    }

    /// Net good evidence in `domain` beyond the prior: good acts remembered less bad ones.
    pub fn esteem(&self, domain: Domain, params: &TieParams) -> f64 {
        let i = domain.index();
        (f64::from(self.good[i]) - params.prior) - (f64::from(self.bad[i]) - params.prior)
    }

    /// How much the tie matters to its holder: what decides which ties are let go when there
    /// is no room.
    pub fn salience(&self, params: &TieParams) -> f64 {
        f64::from(self.familiarity)
            + f64::from(self.warmth)
            + params.salience_per_evidence * self.evidence(params)
    }

    /// [`Tie::salience`] on `day`, worked out without bringing the whole tie up to date: all
    /// evidence fades toward the prior at one rate, so what lies beyond it fades as one.
    pub fn salience_at(&self, day: i64, params: &TieParams) -> f64 {
        let days = (day - self.day).max(0) as f64;
        self.known_at(day, params)
            + params.salience_per_evidence
                * faded(self.evidence(params), days, params.evidence_half_life_days)
    }

    /// Familiarity and warmth together on `day`.
    pub fn known_at(&self, day: i64, params: &TieParams) -> f64 {
        let days = (day - self.day).max(0) as f64;
        faded(
            f64::from(self.familiarity),
            days,
            params.familiarity_half_life_days,
        ) + faded(f64::from(self.warmth), days, params.warmth_half_life_days)
    }

    /// How much the holder regards the other on `day`: their esteem in every domain together,
    /// with warmth to tell apart those esteemed alike (ADR-0014 §3).
    pub fn regard_at(&self, day: i64, params: &TieParams) -> f64 {
        let days = (day - self.day).max(0) as f64;
        // Good and bad evidence fade toward the prior at one rate, so their difference does too.
        let esteem: f64 = Domain::ALL.iter().map(|&d| self.esteem(d, params)).sum();
        faded(esteem, days, params.evidence_half_life_days)
            + faded(f64::from(self.warmth), days, params.warmth_half_life_days)
    }

    /// Hours of help owed or owing on `day`.
    pub fn help_at(&self, day: i64, params: &TieParams) -> f64 {
        let days = (day - self.day).max(0) as f64;
        faded(f64::from(self.help_h), days, params.help_half_life_days)
    }
}

/// Everyone's ties, by holder; each holder's sorted by whom they are of.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ties {
    held: civ_core::FastMap<PermanentId, Vec<Tie>>,
    /// Ties let go for want of room since the counters began (not saved).
    pub let_go: u64,
}

impl Ties {
    /// No ties.
    pub fn new() -> Ties {
        Ties::default()
    }

    /// The ties `holder` keeps, as they stood when last brought up to date, sorted by whom they
    /// are of.
    pub fn of(&self, holder: PermanentId) -> &[Tie] {
        self.held.get(&holder).map_or(&[], Vec::as_slice)
    }

    /// `holder`'s tie to `to`, as it stands on `day`.
    pub fn tie(
        &self,
        holder: PermanentId,
        to: PermanentId,
        day: i64,
        params: &TieParams,
    ) -> Option<Tie> {
        let ties = self.held.get(&holder)?;
        let i = ties.binary_search_by_key(&to, |t| t.to).ok()?;
        Some(ties[i].at(day, params))
    }

    /// How well `holder` knows `to` on `day`: familiarity and warmth together, 0 for a stranger.
    pub fn known(&self, holder: PermanentId, to: PermanentId, day: i64, params: &TieParams) -> f64 {
        self.held
            .get(&holder)
            .and_then(|ties| {
                let i = ties.binary_search_by_key(&to, |t| t.to).ok()?;
                Some(ties[i].known_at(day, params))
            })
            .unwrap_or(0.0)
    }

    /// How much `holder` regards `to` on `day` ([`Tie::regard_at`]), 0 for a stranger.
    pub fn regard(&self, holder: PermanentId, to: PermanentId, day: i64, params: &TieParams) -> f64 {
        self.held
            .get(&holder)
            .and_then(|ties| {
                let i = ties.binary_search_by_key(&to, |t| t.to).ok()?;
                Some(ties[i].regard_at(day, params))
            })
            .unwrap_or(0.0)
    }

    /// Every holder, in id order.
    pub fn holders(&self) -> Vec<PermanentId> {
        let mut out: Vec<PermanentId> = self.held.keys().copied().collect();
        out.sort_unstable();
        out
    }

    /// Number of ties kept.
    pub fn len(&self) -> usize {
        self.held.values().map(Vec::len).sum()
    }

    /// Whether nobody keeps a tie.
    pub fn is_empty(&self) -> bool {
        self.held.values().all(Vec::is_empty)
    }

    /// Writes `units` of `act` by `to`, seen by `holder` on `day`, with `help_h` hours of help
    /// received (negative: given). A stranger gets a tie; with no room, the least salient tie that
    /// holds no balance of help over [`TieParams::hold_help_h`] is let go first (ADR-0014 §2).
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        holder: PermanentId,
        to: PermanentId,
        act: Act,
        units: f64,
        help_h: f64,
        day: i64,
        params: &TieParams,
    ) {
        if holder == to || params.room == 0 {
            return;
        }
        let ties = self.held.entry(holder).or_default();
        let i = match ties.binary_search_by_key(&to, |t| t.to) {
            Ok(i) => {
                ties[i] = ties[i].at(day, params);
                i
            }
            Err(mut i) => {
                if ties.len() >= params.room {
                    let scored: Vec<(f64, bool)> = ties
                        .iter()
                        .map(|t| {
                            let held = t.help_at(day, params).abs() > params.hold_help_h;
                            (t.salience_at(day, params), held)
                        })
                        .collect();
                    let worst = |hold: bool| {
                        scored
                            .iter()
                            .enumerate()
                            .filter(|(_, (_, held))| !hold || !held)
                            .min_by(|a, b| a.1.0.total_cmp(&b.1.0).then(a.0.cmp(&b.0)))
                            .map(|(k, _)| k)
                    };
                    if let Some(k) = worst(true).or_else(|| worst(false)) {
                        ties.remove(k);
                        self.let_go += 1;
                        if k < i {
                            i -= 1;
                        }
                    }
                }
                ties.insert(i, Tie::new(to, day, params));
                i
            }
        };
        ties[i].record(act, units, help_h, day, params);
    }

    /// Forgets everything `holder` kept: they died or left.
    pub fn forget_holder(&mut self, holder: PermanentId) {
        self.held.remove(&holder);
    }

    /// Drops every tie to someone `keep` says is gone, and every holder with none left.
    pub fn prune(&mut self, keep: impl Fn(PermanentId) -> bool) {
        self.held.retain(|_, ties| {
            ties.retain(|t| keep(t.to));
            !ties.is_empty()
        });
    }

    /// Restores `holder`'s ties from a save, sorting them by whom they are of.
    pub fn restore(&mut self, holder: PermanentId, mut ties: Vec<Tie>) {
        ties.sort_by_key(|t| t.to);
        ties.dedup_by_key(|t| t.to);
        if !ties.is_empty() {
            self.held.insert(holder, ties);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn params() -> TieParams {
        let mut p = TieParams::default();
        p.acts[Act::GiftReceived as usize] = ActWeights {
            familiarity: 0.2,
            warmth: 0.1,
            domain: Some(Domain::Provision),
            good: 1.0,
            bad: 0.0,
        };
        p.acts[Act::Hearth as usize] = ActWeights {
            familiarity: 0.1,
            warmth: 0.01,
            ..ActWeights::default()
        };
        p
    }

    #[test]
    fn evidence_fades_toward_what_a_stranger_starts_from() {
        let p = params();
        let mut ties = Ties::new();
        ties.record(id(1), id(2), Act::GiftReceived, 1.0, 3.0, 0, &p);
        ties.record(id(1), id(2), Act::GiftReceived, 1.0, 3.0, 0, &p);
        let now = ties.tie(id(1), id(2), 0, &p).expect("kept");
        assert!((now.esteem(Domain::Provision, &p) - 2.0).abs() < 1e-6);
        assert!((now.help_h - 6.0).abs() < 1e-6);
        // A half-life on, half the evidence beyond the prior is left, and less help is owed.
        let later = ties.tie(id(1), id(2), 180, &p).expect("kept");
        assert!((later.esteem(Domain::Provision, &p) - 1.0).abs() < 1e-4);
        assert!(later.help_h < now.help_h);
        assert!(later.familiarity < now.familiarity);
        assert!(later.good[Domain::Craft.index()] == p.prior as f32);
        // Nobody else holds a tie back.
        assert!(ties.tie(id(2), id(1), 0, &p).is_none());
    }

    #[test]
    fn company_gives_familiarity_and_warmth_but_no_esteem() {
        let p = params();
        let mut ties = Ties::new();
        ties.record(id(1), id(2), Act::Hearth, 2.0, 0.0, 0, &p);
        let t = ties.tie(id(1), id(2), 0, &p).expect("kept");
        assert!((f64::from(t.familiarity) - 0.19).abs() < 1e-6);
        assert!(t.warmth > 0.0);
        for d in Domain::ALL {
            assert_eq!(t.esteem(d, &p), 0.0);
        }
        assert_eq!(t.reason.map(|r| r.act), Some(Act::Hearth));
        // A gift is the reason from then on; more company does not displace it.
        ties.record(id(1), id(2), Act::GiftReceived, 1.0, 0.0, 3, &p);
        ties.record(id(1), id(2), Act::Hearth, 1.0, 0.0, 4, &p);
        ties.record(id(1), id(2), Act::GiftReceived, 1.0, 0.0, 9, &p);
        let r = ties
            .tie(id(1), id(2), 9, &p)
            .and_then(|t| t.reason)
            .expect("a reason");
        assert_eq!((r.act, r.day, r.times), (Act::GiftReceived, 9, 2));
    }

    #[test]
    fn with_no_room_the_least_salient_tie_without_debts_is_let_go() {
        let mut p = params();
        p.room = 3;
        let mut ties = Ties::new();
        // 2 is owed and is the least salient; 3 is a little better known; 4 best.
        ties.record(id(1), id(2), Act::Hearth, 0.5, 10.0, 0, &p);
        ties.record(id(1), id(3), Act::Hearth, 1.0, 0.0, 0, &p);
        ties.record(id(1), id(4), Act::Hearth, 5.0, 0.0, 0, &p);
        ties.record(id(1), id(5), Act::Hearth, 1.0, 0.0, 1, &p);
        let kept: Vec<u64> = ties.of(id(1)).iter().map(|t| t.to.get()).collect();
        assert_eq!(kept, vec![2, 4, 5]);
        assert_eq!(ties.let_go, 1);
        assert_eq!(ties.len(), 3);
    }

    #[test]
    fn the_gone_are_pruned_and_their_own_ties_forgotten() {
        let p = params();
        let mut ties = Ties::new();
        ties.record(id(1), id(2), Act::Hearth, 1.0, 0.0, 0, &p);
        ties.record(id(1), id(3), Act::Hearth, 1.0, 0.0, 0, &p);
        ties.record(id(3), id(1), Act::Hearth, 1.0, 0.0, 0, &p);
        ties.forget_holder(id(3));
        ties.prune(|x| x != id(3));
        assert_eq!(ties.holders(), vec![id(1)]);
        assert_eq!(ties.of(id(1)).len(), 1);
    }

    #[test]
    fn a_reason_says_what_was_done_how_often_and_when() {
        let r = Reason {
            act: Act::GiftReceived,
            day: 365 * 3 + 130,
            times: 3,
        };
        assert_eq!(r.words(), "gave their household food 3 times, last in May of year 4");
        let once = Reason { times: 1, ..r };
        assert_eq!(once.words(), "gave their household food, last in May of year 4");
    }

    #[test]
    fn act_and_domain_names_and_codes_round_trip() {
        for a in Act::ALL {
            assert_eq!(Act::from_name(a.name()), Some(a));
            assert_eq!(Act::from_code(a.code()), Some(a));
        }
        for d in Domain::ALL {
            assert_eq!(Domain::from_name(d.name()), Some(d));
        }
    }
}
