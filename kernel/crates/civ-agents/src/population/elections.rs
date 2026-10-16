//! Seated bodies and their elections, stepped (M7a slice BO, ADR-0024 §1, §3). Each midnight a
//! polity whose custom is a council has its election called when nobody is yet seated under that
//! custom or the seats' term is ending; word of it goes round as word of a gathering does; those
//! of the electorate who hear of it each choose once whether to stand; when names close, a field
//! no larger than the seats is seated unopposed, a lot draws among the names, or the polling day
//! comes, when those who heard of it may walk to the hearth and name their choices; the next
//! midnight counts the names by the block vote, a lot settling a tie for the last seats, and
//! seats those chosen. Every failure is recorded and nothing repairs it: seats left empty stay
//! empty until the next election.

use civ_core::{PermanentId, Rng64, SimTime};

use super::*;
use crate::count::{self, TieRule};
use crate::decide::GatheringFacts;
use crate::history::ChronicleKind;
use crate::polity::{Membership, Outcome, Stance};
use crate::seats::{
    Candidate, Election, ElectionStage, ElectionWhy, Seat, SeatMethod, Seats, ranking,
};
use crate::word::{Claim, ClaimKind};

/// Keeps the choice to stand apart from every other draw by the same key.
const PURPOSE_STAND: u64 = 0x7374_616e_6430_3031; // "stand001"

/// Minutes a voter needs at least before the polling evening is out.
const LEAST_POLL_MIN: i64 = 10;

impl Population {
    /// Each polity's council and its election, stepped at midnight (ADR-0024 §1, §3).
    pub(super) fn elections_day(&mut self, ctx: &mut Ctx) {
        for pi in 0..self.polities.len() {
            if self.polities[pi].ended.is_some() {
                continue;
            }
            self.election_step(ctx, pi);
        }
    }

    /// Of `adults` (adults and their households, in id order), those holding a seat in any
    /// polity's council: a member sits where they live.
    pub(super) fn sitting(
        &self,
        adults: &[(PermanentId, PermanentId)],
    ) -> Vec<(PermanentId, PermanentId)> {
        adults
            .iter()
            .copied()
            .filter(|&(p, _)| self.sits(p))
            .collect()
    }

    /// Whether `person` holds a seat in a polity's council.
    pub(super) fn sits(&self, person: PermanentId) -> bool {
        self.polities
            .iter()
            .any(|p| p.seated.iter().any(|s| s.holder == person))
    }

    /// The electorate of polity `pi`'s council, with their households, in id order: those of its
    /// members the seats' electorate admits.
    pub(super) fn electorate(
        &self,
        ctx: &Ctx,
        pi: usize,
        seats: &Seats,
    ) -> Vec<(PermanentId, PermanentId)> {
        let adults = self.members_of_polity(pi, ctx.now, ctx.params);
        self.admitted(
            seats.electorate,
            &adults,
            &ctx.land.fields,
            ctx.now,
            ctx.params,
        )
    }

    /// How many of polity `pi`'s members the electorate of a council of `seats` admits now
    /// (M7a slice BO): for the label and the observer.
    pub fn electorate_count(
        &self,
        fields: &[civ_land::fields::Field],
        pi: usize,
        seats: &Seats,
        now: SimTime,
        params: &PeopleParams,
    ) -> u32 {
        let adults = self.members_of_polity(pi, now, params);
        self.admitted(seats.electorate, &adults, fields, now, params)
            .len() as u32
    }

    /// Polity `pi`'s council at midnight.
    fn election_step(&mut self, ctx: &mut Ctx, pi: usize) {
        let day = ctx.now.day_index();
        let Some(seats) = self.polities[pi].body.seats else {
            // No longer a council: its seats and any election under way end with it.
            let polity = &mut self.polities[pi];
            polity.seated.clear();
            for e in polity.elections.iter_mut().filter(|e| e.stage.open()) {
                e.stage = ElectionStage::Abandoned;
                e.ended = Some(day);
            }
            return;
        };
        // A council of other seats than an election under way was called for abandons it.
        for e in self.polities[pi]
            .elections
            .iter_mut()
            .filter(|e| e.stage.open() && e.seats != seats)
        {
            e.stage = ElectionStage::Abandoned;
            e.ended = Some(day);
        }
        // A seat whose holder died or no longer lives where the polity holds stays empty until
        // the next election; so does one whose term has run out.
        let members = self.members_of_polity(pi, ctx.now, ctx.params);
        self.polities[pi]
            .seated
            .retain(|s| s.until > day && members.binary_search_by_key(&s.holder, |m| m.0).is_ok());
        if !self.polities[pi].elections.iter().any(|e| e.stage.open()) {
            self.call_election(ctx, pi, seats);
        }
        let Some(ei) = self.polities[pi]
            .elections
            .iter()
            .position(|e| e.stage.open())
        else {
            return;
        };
        let stage = self.polities[pi].elections[ei].stage;
        if stage == ElectionStage::Standing {
            self.consider_standing(ctx, pi, ei);
            if day > self.polities[pi].elections[ei].stand_until {
                self.close_names(ctx, pi, ei);
            }
        } else if stage == ElectionStage::Polling && day > self.polities[pi].elections[ei].poll_day
        {
            self.count_election(ctx, pi, ei);
        }
    }

    /// Calls polity `pi`'s election for `seats` if one is due: nobody has been seated under the
    /// present custom yet (it was amended or taken to a council), or the seats' term ends by the
    /// time an election called today would seat its winners.
    fn call_election(&mut self, ctx: &mut Ctx, pi: usize, seats: Seats) {
        let ep = &ctx.params.elections;
        let day = ctx.now.day_index();
        let polity = &self.polities[pi];
        let since = polity
            .versions
            .last()
            .map_or(polity.founded, |v| v.since)
            .day_index();
        let last = polity
            .elections
            .iter()
            .rev()
            .find(|e| e.called >= since && e.seats == seats);
        let lead = i64::from(ep.stand_days) + 2;
        let why = match last {
            None => ElectionWhy::Amendment,
            Some(e) if day + lead >= e.until => ElectionWhy::TermEnding,
            Some(_) => return,
        };
        let id = polity.elections.last().map_or(1, |e| e.id + 1);
        let stand_until = day + i64::from(ep.stand_days);
        let poll_day = stand_until + 1;
        let until = poll_day + 1 + i64::from(seats.term_days);
        // Those who know first: under a new council, those who saw the custom change (the
        // amendment's gathering, or whoever took it); else those who sit now.
        let mut first: Vec<PermanentId> = match why {
            ElectionWhy::Amendment => polity.versions.last().map_or_else(Vec::new, |v| {
                v.law
                    .and_then(|l| polity.laws.iter().find(|x| x.id == l))
                    .map_or_else(Vec::new, |l| l.stances.iter().map(|r| r.person).collect())
            }),
            ElectionWhy::TermEnding => polity.seated.iter().map(|s| s.holder).collect(),
        };
        if let Some(by) = polity.versions.last().and_then(|v| v.seized_by) {
            first.push(by);
        }
        first.sort_unstable();
        first.dedup();
        let settlement = polity.settlement;
        self.polities[pi].elections.push(Election {
            id,
            why,
            seats,
            called: day,
            stand_until,
            poll_day,
            stage: ElectionStage::Standing,
            considered: Vec::new(),
            candidates: Vec::new(),
            ballots: Vec::new(),
            totals: Vec::new(),
            winners: Vec::new(),
            by_lot: false,
            ended: None,
            until,
            endorsements: Vec::new(),
        });
        self.election_word(settlement, poll_day, &first, day);
        let (place, name) = self.place_words(ctx, settlement);
        let words = format!(
            "An election was called at {name}, because {}: {}. Names may be put in until day {}; \
             {} on day {poll_day}.",
            why.words(),
            seats.words(),
            stand_until,
            match seats.method {
                SeatMethod::Election => "the polling is",
                SeatMethod::Lot => "the lot is drawn",
            }
        );
        self.chronicle_push(
            ctx.now,
            ChronicleKind::ElectionCalled,
            first,
            Some(settlement),
            place,
            f64::from(seats.n),
            words,
        );
    }

    /// Word that polity seated at `settlement` polls (or draws its lot) on `poll_day`: the claim,
    /// made if new, and each of `first` hears it on `today` at first hand.
    fn election_word(
        &mut self,
        settlement: PermanentId,
        poll_day: i64,
        first: &[PermanentId],
        today: i64,
    ) {
        let claim = match self.word.election(settlement, poll_day) {
            Some(c) => c,
            None => self.word.make(Claim {
                fire: None,
                event: None,
                id: 0,
                kind: ClaimKind::Election,
                settlement,
                day: poll_day,
                subject: None,
                grievance: None,
                suspected: None,
            }),
        };
        for &p in first {
            self.word.hear(p, claim, today, None, Some(p));
        }
    }

    /// Whether `person` has heard that an election polls at `settlement` on `poll_day`.
    pub(crate) fn heard_of_election(
        &self,
        person: PermanentId,
        settlement: PermanentId,
        poll_day: i64,
    ) -> bool {
        self.word
            .election(settlement, poll_day)
            .is_some_and(|c| self.word.has_heard(person, c))
    }

    /// The hearth and name of `settlement`, for the chronicle.
    fn place_words(&self, ctx: &Ctx, settlement: PermanentId) -> (Option<(f32, f32)>, String) {
        ctx.land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or((None, String::new()), |s| {
                (Some(s.hearth_m), s.name.clone())
            })
    }

    /// Each of the electorate who has heard of election `ei` and not yet chosen chooses whether
    /// to stand (ADR-0024 §3): what their sitting would turn toward their household (the
    /// decisions under this custom they saw go against them, each at their household's stake,
    /// a seat's share of it), and the regard a seat may bring (their standing in counsel),
    /// against the evenings in council and the walk; one keyed draw at the content's temperature.
    fn consider_standing(&mut self, ctx: &mut Ctx, pi: usize, ei: usize) {
        let (day, params) = (ctx.now.day_index(), ctx.params);
        let (ep, pp) = (&params.elections, &params.polity);
        let polity = &self.polities[pi];
        let (settlement, polity_id) = (polity.settlement, polity.id);
        let e = &polity.elections[ei];
        let (seats, poll_day, id) = (e.seats, e.poll_day, e.id);
        let deciding: Vec<(PermanentId, PermanentId)> = self
            .electorate(ctx, pi, &seats)
            .into_iter()
            .filter(|&(p, _)| e.considered.binary_search(&p).is_err())
            .filter(|&(p, _)| self.heard_of_election(p, settlement, poll_day))
            .collect();
        if deciding.is_empty() {
            return;
        }
        let decisions = self.decisions_seen(ctx, pi);
        let mut stood = Vec::new();
        for &(p, h) in &deciding {
            let turned: f64 = decisions
                .iter()
                .filter_map(|s| {
                    let r = s.stances.iter().find(|r| r.0 == p)?;
                    let lost = match r.2 {
                        Stance::Support => !s.passed,
                        Stance::Oppose => s.passed,
                        Stance::Abstain => false,
                    };
                    let stake = s.stances.iter().find(|r| r.1 == h).map_or(0.0, |r| r.3);
                    lost.then(|| stake.abs() / pp.w_gain.max(1e-9))
                })
                .sum();
            let counsel = self
                .standing
                .of(p)
                .map_or(0.0, |s| s.in_domain(crate::ties::Domain::Counsel))
                .max(0.0);
            let points = ep.w_sit * turned / f64::from(seats.n.max(1)) + ep.w_standing * counsel
                - ep.sit_cost;
            let chance = 1.0 / (1.0 + (-points / ep.stand_temperature.max(1e-9)).exp());
            let draw = Rng64::from_key(&[
                ctx.seed,
                polity_id.get(),
                u64::from(id),
                p.get(),
                PURPOSE_STAND,
            ])
            .next_f64();
            if draw < chance {
                stood.push(p);
            }
        }
        // Each who stands says where they stand on the questions that matter most to them
        // (ADR-0024 §4), and word of it goes round from them (M7a slice BP).
        let told = usize::from(ep.positions_told);
        let candidates: Vec<Candidate> = stood
            .iter()
            .map(|&p| {
                let mut held: Vec<&crate::opinion::Position> =
                    self.opinion.held_by(p).iter().collect();
                held.sort_by(|a, b| {
                    b.salience
                        .total_cmp(&a.salience)
                        .then(a.policy.cmp(&b.policy))
                });
                Candidate {
                    person: p,
                    day,
                    positions: held.iter().take(told).map(|x| (x.policy, x.x)).collect(),
                }
            })
            .collect();
        for c in &candidates {
            self.candidacy_word(settlement, poll_day, c.person, day);
        }
        let e = &mut self.polities[pi].elections[ei];
        for &(p, _) in &deciding {
            if let Err(i) = e.considered.binary_search(&p) {
                e.considered.insert(i, p);
            }
        }
        e.candidates.extend(candidates);
    }

    /// Word that `who` stands in the election polling at `settlement` on `poll_day`: the claim,
    /// made if new, which they hear first on `today`.
    fn candidacy_word(
        &mut self,
        settlement: PermanentId,
        poll_day: i64,
        who: PermanentId,
        today: i64,
    ) {
        let claim = match self.word.candidacy(settlement, poll_day, who) {
            Some(c) => c,
            None => self.word.make(Claim {
                fire: None,
                event: None,
                id: 0,
                kind: ClaimKind::Candidacy,
                settlement,
                day: poll_day,
                subject: Some(who),
                grievance: None,
                suspected: None,
            }),
        };
        self.word.hear(who, claim, today, None, Some(who));
    }

    /// Whether `voter` has heard that `who` stands in the election polling at `settlement` on
    /// `poll_day`, and so where they said they stand.
    fn heard_candidacy(
        &self,
        voter: PermanentId,
        settlement: PermanentId,
        poll_day: i64,
        who: PermanentId,
    ) -> bool {
        self.word
            .candidacy(settlement, poll_day, who)
            .is_some_and(|c| self.word.has_heard(voter, c))
    }

    /// Each faction of polity `pi`'s settlements whose members stood in election `ei` has its
    /// organizer endorse them, up to the seats (ADR-0024 §5, M7a slice BP; a design prior: a
    /// faction backs its own). Members weigh the slate when they poll.
    fn endorse_slates(&mut self, pi: usize, ei: usize) {
        let polity = &self.polities[pi];
        let e = &polity.elections[ei];
        let n = usize::from(e.seats.n);
        let mut slates: Vec<(PermanentId, Vec<PermanentId>)> = Vec::new();
        for f in self
            .factions
            .list
            .iter()
            .filter(|f| f.ended.is_none() && polity.holds(Some(f.settlement)))
        {
            let mut slate: Vec<PermanentId> = e
                .candidates
                .iter()
                .map(|c| c.person)
                .filter(|&p| {
                    self.factions
                        .membership(p)
                        .is_some_and(|m| m.faction == f.id)
                })
                .collect();
            slate.truncate(n);
            if !slate.is_empty() && self.person(f.organizer).is_some() {
                slates.push((f.id, slate));
            }
        }
        slates.sort_by_key(|s| s.0);
        self.polities[pi].elections[ei].endorsements = slates;
    }

    /// Names close on election `ei`: with nobody standing it fails; with no more standing than
    /// seats, those who stood sit unopposed; by lot, the lot waits for its day; by election, the
    /// polling day comes.
    fn close_names(&mut self, ctx: &mut Ctx, pi: usize, ei: usize) {
        let day = ctx.now.day_index();
        self.endorse_slates(pi, ei);
        let e = &mut self.polities[pi].elections[ei];
        let standing = e.standing();
        if standing.is_empty() {
            e.stage = ElectionStage::NoCandidates;
            e.ended = Some(day);
            self.tell_election(ctx, pi, ei);
            return;
        }
        if standing.len() <= usize::from(e.seats.n) {
            e.totals = standing.iter().map(|&c| (c, 0)).collect();
            let winners: Vec<PermanentId> = e.candidates.iter().map(|c| c.person).collect();
            self.install(ctx, pi, ei, winners, false);
            return;
        }
        e.stage = ElectionStage::Polling;
    }

    /// The polling day of election `ei` is past: the names are counted by the block vote (or
    /// the lot drawn), a lot settling a tie for the last seats, and those chosen sit.
    fn count_election(&mut self, ctx: &mut Ctx, pi: usize, ei: usize) {
        let day = ctx.now.day_index();
        let polity_id = self.polities[pi].id;
        let e = &mut self.polities[pi].elections[ei];
        let standing = e.standing();
        let key = Rng64::from_key(&[ctx.seed, polity_id.get(), u64::from(e.id)]).next_u64();
        let n = u32::from(e.seats.n);
        let result = match e.seats.method {
            SeatMethod::Lot => {
                let drawn = count::draw(&standing, n, key);
                e.totals = drawn.iter().map(|&c| (c, 1)).collect();
                Some((drawn, true))
            }
            SeatMethod::Election if e.ballots.is_empty() => None,
            SeatMethod::Election => {
                let ballots: Vec<Vec<PermanentId>> =
                    e.ballots.iter().map(|b| b.1.clone()).collect();
                let r = count::block_vote(&ballots, &standing, n, TieRule::Lot(key));
                e.totals = r.totals.votes.clone();
                Some((r.winners, r.by_lot))
            }
        };
        match result {
            Some((winners, by_lot)) if !winners.is_empty() => {
                self.install(ctx, pi, ei, winners, by_lot);
            }
            Some(_) => {
                e.stage = ElectionStage::Tied;
                e.ended = Some(day);
                self.tell_election(ctx, pi, ei);
            }
            None => {
                e.stage = ElectionStage::NoVoters;
                e.ended = Some(day);
                self.tell_election(ctx, pi, ei);
            }
        }
    }

    /// Election `ei` seats `winners`, in that order, for its term: the council's seats are
    /// theirs from today, those still seated under the last election giving theirs up.
    fn install(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        ei: usize,
        winners: Vec<PermanentId>,
        by_lot: bool,
    ) {
        let day = ctx.now.day_index();
        let polity = &mut self.polities[pi];
        let e = &mut polity.elections[ei];
        e.stage = ElectionStage::Installed;
        e.ended = Some(day);
        e.by_lot = by_lot;
        e.winners = winners.clone();
        let (until, id) = (e.until, e.id);
        polity.seated = winners
            .iter()
            .enumerate()
            .map(|(i, &holder)| Seat {
                seat: i as u8,
                holder,
                since: day,
                until,
                election: id,
            })
            .collect();
        self.tell_election(ctx, pi, ei);
    }

    /// The chronicle tells how election `ei` ended.
    fn tell_election(&mut self, ctx: &Ctx, pi: usize, ei: usize) {
        let settlement = self.polities[pi].settlement;
        let (place, name) = self.place_words(ctx, settlement);
        let e = &self.polities[pi].elections[ei];
        let n = e.seats.n;
        let stood = e.standing().len();
        let words = match e.stage {
            ElectionStage::Installed => {
                let names: Vec<String> = e.winners.iter().map(|&w| self.name_of(w)).collect();
                let how = if e.ballots.is_empty() && e.seats.method == SeatMethod::Election {
                    format!("unopposed, {stood} standing for {n} seats")
                } else if e.seats.method == SeatMethod::Lot {
                    format!("by lot from {stood} who put their names in")
                } else {
                    format!(
                        "by {} who came to name their choices, {stood} standing for {n} seats{}",
                        e.ballots.len(),
                        if e.by_lot {
                            "; a lot settled a tie"
                        } else {
                            ""
                        }
                    )
                };
                let empty = usize::from(n).saturating_sub(e.winners.len());
                format!(
                    "The council at {name} was chosen, {how}: {}{}.",
                    names.join(", "),
                    if empty > 0 {
                        format!("; {empty} seats stay empty")
                    } else {
                        String::new()
                    }
                )
            }
            stage => format!(
                "The election at {name} {}; its {n} seats stay empty until the next.",
                stage.words()
            ),
        };
        let who = e.winners.clone();
        let amount = e.winners.len() as f64;
        self.chronicle_push(
            ctx.now,
            ChronicleKind::ElectionHeld,
            who,
            Some(settlement),
            place,
            amount,
            words,
        );
    }

    /// What coming to poll in the settlement of household `hh` is worth to `person`, aged `age`,
    /// at `minute` of the day, with the evening from `evening_start` (ADR-0024 §3: polling is an
    /// activity like attending): the content's base and the stake, how much better their first
    /// choice is than the best left out. `None` when no election of theirs polls now, they are
    /// not of its electorate, have not heard of it or have polled.
    pub(crate) fn poll_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        age: f64,
        hh: &Household,
        minute: i64,
        evening_start: i64,
    ) -> Option<GatheringFacts> {
        let params = ctx.params;
        let ep = &params.elections;
        if age < params.family.independent_age {
            return None;
        }
        let pi = self.polity_of(hh.settlement?)?;
        let polity = &self.polities[pi];
        let e = polity
            .elections
            .iter()
            .find(|e| e.stage == ElectionStage::Polling)?;
        let end = evening_start + i64::from(params.polity.gathering_minutes);
        if e.poll_day != ctx.now.day_index()
            || minute < evening_start
            || minute + LEAST_POLL_MIN > end
            || e.polled(person)
            || !self.heard_of_election(person, polity.settlement, e.poll_day)
            || !self.admits_by(
                &ctx.land.fields,
                e.seats.electorate,
                person,
                hh.id,
                ctx.now,
                params,
            )
        {
            return None;
        }
        let mut utilities: Vec<f64> = e
            .standing()
            .into_iter()
            .map(|c| self.vote_utility(ctx, person, hh.id, c, e, polity.settlement))
            .collect();
        utilities.sort_by(|a, b| b.total_cmp(a));
        let left_out = utilities
            .get(usize::from(e.seats.n))
            .copied()
            .unwrap_or(f64::NEG_INFINITY);
        let stake = utilities
            .first()
            .map_or(0.0, |&u| {
                if left_out.is_finite() {
                    u - left_out
                } else {
                    0.0
                }
            })
            .max(0.0);
        Some(GatheringFacts {
            points: ep.poll_base + ep.w_poll_stake * stake,
            minutes: f64::from(ep.poll_minutes).min((end - minute) as f64),
        })
    }

    /// What candidate `c` in election `e` at `settlement` is worth to voter `v` of household
    /// `vh` (ADR-0024 §3, 09-08 §1.3's terms where state exists): regard (loyalty), their own
    /// household (group), the candidate's standing in counsel (competence), and themselves;
    /// distance from where the candidate said they stand, if the voter heard it (D); the
    /// candidate's record as the voter saw it (R); and the voter's own faction's slate. Unknown
    /// is no term, never neutral.
    pub(super) fn vote_utility(
        &self,
        ctx: &Ctx,
        v: PermanentId,
        vh: PermanentId,
        c: PermanentId,
        e: &Election,
        settlement: PermanentId,
    ) -> f64 {
        let (ep, day) = (&ctx.params.elections, ctx.now.day_index());
        if c == v {
            return ep.w_self;
        }
        let regard = self.ties.regard(v, c, day, &ctx.params.ties);
        let same = self.person(c).is_some_and(|q| q.household == vh);
        let counsel = self
            .standing
            .of(c)
            .map_or(0.0, |s| s.in_domain(crate::ties::Domain::Counsel))
            .max(0.0);
        let distance = self
            .heard_candidacy(v, settlement, e.poll_day, c)
            .then(|| e.positions_of(c))
            .flatten()
            .and_then(|told| self.distance(v, told));
        let record = self.record(ctx, v, vh, c, settlement);
        let slate = self
            .factions
            .membership(v)
            .is_some_and(|m| e.endorsed(m.faction, c));
        ep.w_regard * regard + if same { ep.w_household } else { 0.0 } + ep.w_counsel * counsel
            - distance.map_or(0.0, |d| ep.w_distance * d)
            + record.map_or(0.0, |r| ep.w_record * r)
            + if slate { ep.w_slate } else { 0.0 }
    }

    /// The mean distance, 0 to 1, between `v`'s positions and the positions `told`, on the
    /// questions `v` holds one on; `None` when they hold none of them.
    fn distance(&self, v: PermanentId, told: &[(u16, f32)]) -> Option<f64> {
        let ds: Vec<f64> = told
            .iter()
            .filter_map(|&(q, x)| {
                self.opinion
                    .position(v, q)
                    .map(|p| f64::from((p.x - x).abs()))
            })
            .collect();
        (!ds.is_empty()).then(|| ds.iter().sum::<f64>() / ds.len() as f64)
    }

    /// Candidate `c`'s record as voter `v` of household `vh` saw it (09-08 §1.3, R; ADR-0024 §3:
    /// the voter's household's stake in decisions they saw the candidate stand on): over the laws
    /// of the polity seated at `settlement` decided within memory at which both took a stance,
    /// how the candidate's stance lined up with what the law meant to `v`'s household, each
    /// weighed by it, −1 to 1. `None` when they shared no such decision.
    fn record(
        &self,
        ctx: &Ctx,
        v: PermanentId,
        vh: PermanentId,
        c: PermanentId,
        settlement: PermanentId,
    ) -> Option<f64> {
        let memory = i64::from(ctx.params.polity.vote_memory_days);
        let day = ctx.now.day_index();
        let polity = self.polities.iter().find(|p| p.settlement == settlement)?;
        let (mut along, mut weight) = (0.0, 0.0);
        for l in &polity.laws {
            if !l.decided.is_some_and(|t| day - t.day_index() <= memory) {
                continue;
            }
            let Some(mine) = l
                .stances
                .iter()
                .find(|r| r.person == v && r.household == vh)
            else {
                continue;
            };
            let Some(theirs) = l.stances.iter().find(|r| r.person == c) else {
                continue;
            };
            let stake = f64::from(mine.gain);
            along += theirs.stance.sign() * stake;
            weight += stake.abs();
        }
        (weight > 1e-9).then(|| (along / weight).clamp(-1.0, 1.0))
    }

    /// Points `person` finds in an evening at the hearth while an election they stand in runs,
    /// to tell that they stand and where (ADR-0024 §4); 0 when they stand in none.
    pub(crate) fn campaign_points(&self, ctx: &Ctx, person: PermanentId) -> f64 {
        let day = ctx.now.day_index();
        let standing = self.polities.iter().any(|p| {
            p.elections
                .iter()
                .any(|e| e.stage.open() && e.poll_day >= day && e.stood(person))
        });
        if standing {
            ctx.params.elections.w_campaign
        } else {
            0.0
        }
    }

    /// `person` has come to the hearth on the polling day and names their choices: the top n of
    /// one ranking of those standing by their utility, drawn at the content's scale (ADR-0024
    /// §3: one evaluation, one softmax; the block vote takes the top n of it).
    pub(crate) fn cast(&mut self, ctx: &Ctx, person: PermanentId) {
        let Some(hh) = self.person(person).map(|p| p.household) else {
            return;
        };
        let Some(pi) = self
            .household(hh)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        let day = ctx.now.day_index();
        let polity = &self.polities[pi];
        let polity_id = polity.id;
        let Some(ei) = polity
            .elections
            .iter()
            .position(|e| e.stage == ElectionStage::Polling && e.poll_day == day)
        else {
            return;
        };
        let e = &polity.elections[ei];
        if e.polled(person) {
            return;
        }
        let settlement = polity.settlement;
        let cands: Vec<(PermanentId, f64)> = e
            .standing()
            .into_iter()
            .map(|c| (c, self.vote_utility(ctx, person, hh, c, e, settlement)))
            .collect();
        let key =
            Rng64::from_key(&[ctx.seed, polity_id.get(), u64::from(e.id), person.get()]).next_u64();
        let mut choice = ranking(&cands, ctx.params.elections.vote_scale, key);
        choice.truncate(usize::from(e.seats.n));
        self.polities[pi].elections[ei]
            .ballots
            .push((person, choice));
    }

    /// Whether adult `person` of household `hh` belongs to membership `m` (an electorate).
    fn admits_by(
        &self,
        fields: &[civ_land::fields::Field],
        m: Membership,
        person: PermanentId,
        hh: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> bool {
        match m {
            Membership::Adults => true,
            Membership::Elders => self.elder_of(hh, now, params) == Some(person),
            Membership::Landholders => fields.iter().any(|f| f.holder.household() == Some(hh)),
            Membership::Watch => self.keeps_a_watch(person),
            Membership::Seats => self.sits(person),
        }
    }

    /// Who the sponsor of a council of `seats` expects would sit in it (ADR-0024 §6, a design
    /// prior): by election, the n of the electorate `sponsor` knows (a tie, or themselves) who
    /// stand highest in counsel; by lot, the whole electorate (its seen stances in proportion).
    pub(super) fn forecast_sitters(
        &self,
        ctx: &Ctx,
        seats: &Seats,
        electorate: &[PermanentId],
        sponsor: PermanentId,
    ) -> Vec<PermanentId> {
        if seats.method == SeatMethod::Lot {
            return electorate.to_vec();
        }
        let day = ctx.now.day_index();
        let tp = &ctx.params.ties;
        let mut known: Vec<(f64, PermanentId)> = electorate
            .iter()
            .copied()
            .filter(|&p| p == sponsor || self.ties.tie(sponsor, p, day, tp).is_some())
            .map(|p| {
                let counsel = self
                    .standing
                    .of(p)
                    .map_or(0.0, |s| s.in_domain(crate::ties::Domain::Counsel));
                (counsel, p)
            })
            .collect();
        known.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        known.truncate(usize::from(seats.n));
        let mut out: Vec<PermanentId> = known.into_iter().map(|k| k.1).collect();
        out.sort_unstable();
        out
    }
}

/// Whether a council of `seats` with rule `body`, sat by `sitters` (in id order), would have
/// passed a decision whose stances were `stances` (person, stance) (ADR-0024 §6): by election,
/// the sitters' recorded stances (absent ones abstain), its quorum counted as met; by lot, the
/// sitters' (the electorate's) seen stances in proportion, scaled to the seats.
pub(super) fn council_would_pass(
    body: &crate::polity::Body,
    seats: &Seats,
    sitters: &[PermanentId],
    stances: &[(PermanentId, Stance)],
) -> bool {
    let n = u32::from(seats.n);
    let (mut support, mut oppose) = (0u32, 0u32);
    for &(p, s) in stances {
        if sitters.binary_search(&p).is_err() {
            continue;
        }
        match s {
            Stance::Support => support += 1,
            Stance::Oppose => oppose += 1,
            Stance::Abstain => {}
        }
    }
    if seats.method == SeatMethod::Lot {
        let took = support + oppose;
        if took == 0 {
            return body.decide(n, n, 0, 0) == Outcome::Passed;
        }
        let s = (f64::from(support) * f64::from(n) / f64::from(took)).round() as u32;
        return body.decide(n, n, s, n - s.min(n)) == Outcome::Passed;
    }
    body.decide(n, n, support, oppose) == Outcome::Passed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polity::{Body, PassRule};

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn council(method: SeatMethod) -> (Body, Seats) {
        let seats = Seats {
            n: 5,
            electorate: Membership::Adults,
            method,
            term_days: 360,
        };
        let body = Body {
            members: Membership::Seats,
            quorum_share: 0.5,
            pass: PassRule::MoreForThanAgainst,
            seats: Some(seats),
        };
        (body, seats)
    }

    #[test]
    fn a_council_forecast_counts_its_sitters_stances_and_counts_its_quorum_as_met() {
        let (body, seats) = council(SeatMethod::Election);
        let sitters = [id(1), id(2), id(3), id(4), id(5)];
        // Seen: three sitters for, one against, and many others against: the council passes
        // what the gathering turned down.
        let mut stances = vec![
            (id(1), Stance::Support),
            (id(2), Stance::Support),
            (id(3), Stance::Support),
            (id(4), Stance::Oppose),
        ];
        stances.extend((10..30).map(|k| (id(k), Stance::Oppose)));
        assert!(council_would_pass(&body, &seats, &sitters, &stances));
        // A sitter who was not there abstains: two for, two against is a tie, which fails.
        let tied = [
            (id(1), Stance::Support),
            (id(2), Stance::Support),
            (id(3), Stance::Oppose),
            (id(4), Stance::Oppose),
        ];
        assert!(!council_would_pass(&body, &seats, &sitters, &tied));
    }

    #[test]
    fn a_council_by_lot_is_forecast_from_the_electorates_stances_in_proportion() {
        let (body, seats) = council(SeatMethod::Lot);
        let electorate: Vec<PermanentId> = (1..=10).map(id).collect();
        // Seven of ten for: about 4 of 5 seats for.
        let mostly: Vec<(PermanentId, Stance)> = (1..=10)
            .map(|k| {
                (
                    id(k),
                    if k <= 7 {
                        Stance::Support
                    } else {
                        Stance::Oppose
                    },
                )
            })
            .collect();
        assert!(council_would_pass(&body, &seats, &electorate, &mostly));
        // Three of ten for: about 2 of 5.
        let few: Vec<(PermanentId, Stance)> = (1..=10)
            .map(|k| {
                (
                    id(k),
                    if k <= 3 {
                        Stance::Support
                    } else {
                        Stance::Oppose
                    },
                )
            })
            .collect();
        assert!(!council_would_pass(&body, &seats, &electorate, &few));
    }

    #[test]
    fn a_council_of_the_gatherings_own_members_is_one_change_from_it_and_back() {
        let (body, seats) = council(SeatMethod::Election);
        let gathering = Body {
            members: Membership::Adults,
            quorum_share: 0.5,
            pass: PassRule::MoreForThanAgainst,
            seats: None,
        };
        assert_eq!(body.changes_from(&gathering), 1);
        assert_eq!(gathering.changes_from(&body), 1);
        let elders = Body {
            members: Membership::Elders,
            ..gathering
        };
        assert_eq!(
            body.changes_from(&elders),
            2,
            "another electorate is a second change"
        );
        let lot = Body {
            seats: Some(Seats {
                method: SeatMethod::Lot,
                ..seats
            }),
            ..body
        };
        assert_eq!(lot.changes_from(&body), 1);
        assert_eq!(
            body.eligible(2),
            5,
            "a council counts its seats, filled or not"
        );
        assert_eq!(gathering.eligible(40), 40);
    }
}
