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
    Alliance, Candidate, Election, ElectionStage, ElectionWhy, Package, Seat, SeatMethod, Seats,
    apportion, bloc_seats, ranking,
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

    /// Where polity `pi`'s `seats` are shared among its settlements and it holds several (M7a
    /// slice BQ, ADR-0024 §1): each settlement, the seat's first, and its share of the seats by
    /// its electorate now; else none.
    fn districts(&self, ctx: &Ctx, pi: usize, seats: &Seats) -> Vec<(PermanentId, u8)> {
        let polity = &self.polities[pi];
        if !seats.by_settlement || polity.held.is_empty() {
            return Vec::new();
        }
        let electorate = self.electorate(ctx, pi, seats);
        let counts: Vec<(PermanentId, u32)> = polity
            .settlements()
            .into_iter()
            .map(|s| {
                let n = electorate
                    .iter()
                    .filter(|e| self.household(e.1).and_then(|h| h.settlement) == Some(s))
                    .count();
                (s, n as u32)
            })
            .collect();
        apportion(seats.n, &counts)
    }

    /// The settlement `person` lives in, if any.
    fn residence(&self, person: PermanentId) -> Option<PermanentId> {
        self.person(person)
            .and_then(|p| self.household(p.household))
            .and_then(|h| h.settlement)
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
        // the next election; so does one whose term has run out. Seats the custom no longer has
        // lapse.
        let members = self.members_of_polity(pi, ctx.now, ctx.params);
        self.polities[pi]
            .seated
            .retain(|s| s.until > day && members.binary_search_by_key(&s.holder, |m| m.0).is_ok());
        self.polities[pi].lapse_seats_beyond_custom();
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
        // Too few seated for its quorum: the seats are called again, `stand_days` after the last
        // election closed, and those seated give theirs up to the winners as at a term's end.
        let too_few = (polity.seated.len() as u32) < polity.body.quorum(u32::from(seats.n));
        let why = match last {
            None => ElectionWhy::Amendment,
            Some(e) if day + lead >= e.until => ElectionWhy::TermEnding,
            Some(e) if too_few && e.ended.is_some_and(|d| day >= d + i64::from(ep.stand_days)) => {
                ElectionWhy::TooFewSat
            }
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
            // Those seated, and those who stood last time.
            ElectionWhy::TooFewSat => polity
                .seated
                .iter()
                .map(|s| s.holder)
                .chain(
                    last.into_iter()
                        .flat_map(|e| e.candidates.iter().map(|c| c.person)),
                )
                .collect(),
        };
        if let Some(by) = polity.versions.last().and_then(|v| v.seized_by) {
            first.push(by);
        }
        first.sort_unstable();
        first.dedup();
        let settlement = polity.settlement;
        let districts = self.districts(ctx, pi, &seats);
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
            districts,
            alliances: Vec::new(),
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
        // Where seats are shared among settlements, each stands in the district they live in,
        // for its seats (M7a slice BQ).
        let district_of =
            |h: PermanentId| e.district_for(self.household(h).and_then(|x| x.settlement));
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
            let d = district_of(h);
            let n = e.seats_in(d);
            if n == 0 {
                continue;
            }
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
            let points = ep.w_sit * turned / f64::from(n) + ep.w_standing * counsel - ep.sit_cost;
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
                stood.push((p, d));
            }
        }
        // Each who stands says where they stand on the questions that matter most to them
        // (ADR-0024 §4), and word of it goes round from them (M7a slice BP).
        let told = usize::from(ep.positions_told);
        let candidates: Vec<Candidate> = stood
            .iter()
            .map(|&(p, district)| {
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
                    district,
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

    /// Before names close on election `ei` of polity `pi`, each pair of factions with slates
    /// whose organizers know each other may meet (ADR-0024 §5, research 09-08 §1.9, M7a slice
    /// BQ): each reckons, from those of the electorate they know to belong to each faction (all
    /// of their own), the seats each faction's slate would take if every bloc voted its own slate
    /// whole (`bloc_seats`, a design prior); and weighs three packages against that outside
    /// option: one common slate, shared by following, or either standing down for the other.
    /// They agree on the package that leaves neither worse off and one better, the best for the
    /// one who gains least first, then for both; else they part. Both then back its slate. Seats
    /// shared among settlements wait (each district would need its own meeting).
    fn seek_alliances(&mut self, ctx: &Ctx, pi: usize, ei: usize) {
        let day = ctx.now.day_index();
        let tp = &ctx.params.ties;
        let e = &self.polities[pi].elections[ei];
        if !e.districts.is_empty() || e.endorsements.len() < 3 {
            return;
        }
        let n = e.seats.n;
        let seats = e.seats;
        let mut slates = e.endorsements.clone();
        let electorate: Vec<PermanentId> = self
            .electorate(ctx, pi, &seats)
            .into_iter()
            .map(|x| x.0)
            .collect();
        let factions: Vec<PermanentId> = slates.iter().map(|s| s.0).collect();
        let organizer: Vec<Option<PermanentId>> = factions
            .iter()
            .map(|&f| self.factions.get(f).map(|x| x.organizer))
            .collect();
        // How many of the electorate organizer `o` (of faction `own`) reckons follow faction `f`.
        let following = |o: PermanentId, own: PermanentId, f: PermanentId| -> u32 {
            electorate
                .iter()
                .filter(|&&p| {
                    self.factions.membership(p).is_some_and(|m| m.faction == f)
                        && (f == own || p == o || self.ties.tie(o, p, day, tp).is_some())
                })
                .count() as u32
        };
        let mut allied: Vec<bool> = vec![false; slates.len()];
        let mut made: Vec<(Alliance, Vec<PermanentId>, [PermanentId; 2])> = Vec::new();
        for i in 0..slates.len() {
            for j in (i + 1)..slates.len() {
                if allied[i] || allied[j] {
                    continue;
                }
                let (Some(oi), Some(oj)) = (organizer[i], organizer[j]) else {
                    continue;
                };
                if self.ties.tie(oi, oj, day, tp).is_none() {
                    continue;
                }
                // Each organizer's reckoning: followings of every faction, as they see them.
                let seen: [Vec<u32>; 2] = [(oi, factions[i]), (oj, factions[j])]
                    .map(|(o, own)| factions.iter().map(|&f| following(o, own, f)).collect());
                let lens: Vec<usize> = slates.iter().map(|s| s.1.len()).collect();
                let own_of = |slate: &[PermanentId], f: PermanentId, k: u32| -> u32 {
                    slate
                        .iter()
                        .take(k as usize)
                        .filter(|&&p| self.factions.membership(p).is_some_and(|m| m.faction == f))
                        .count() as u32
                };
                // Seats each side expects for its own members, alone.
                let alone: [u32; 2] = [0, 1].map(|w| {
                    let blocs: Vec<(u32, usize)> =
                        seen[w].iter().copied().zip(lens.iter().copied()).collect();
                    bloc_seats(n, &blocs)[if w == 0 { i } else { j }]
                });
                let mut best: Option<(i64, i64, Package, Vec<PermanentId>)> = None;
                for package in Package::ALL {
                    let slate = match package {
                        Package::Common => {
                            common_slate(&slates[i].1, &slates[j].1, seen[0][i], seen[1][j], n)
                        }
                        Package::FirstStandsDown => slates[j].1.clone(),
                        Package::SecondStandsDown => slates[i].1.clone(),
                    };
                    let gains: [i64; 2] = [0, 1].map(|w| {
                        let mut blocs: Vec<(u32, usize)> =
                            seen[w].iter().copied().zip(lens.iter().copied()).collect();
                        blocs[i] = (seen[w][i] + seen[w][j], slate.len());
                        blocs[j] = (0, 0);
                        let k = bloc_seats(n, &blocs)[i];
                        let own = own_of(&slate, factions[if w == 0 { i } else { j }], k);
                        i64::from(own) - i64::from(alone[w])
                    });
                    if gains[0] < 0 || gains[1] < 0 || gains[0] + gains[1] <= 0 {
                        continue;
                    }
                    let key = (gains[0].min(gains[1]), gains[0] + gains[1]);
                    if best.as_ref().is_none_or(|b| key > (b.0, b.1)) {
                        best = Some((key.0, key.1, package, slate));
                    }
                }
                if let Some((_, _, package, slate)) = best {
                    allied[i] = true;
                    allied[j] = true;
                    made.push((
                        Alliance {
                            factions: [factions[i], factions[j]],
                            package,
                        },
                        slate,
                        [oi, oj],
                    ));
                }
            }
        }
        if made.is_empty() {
            return;
        }
        let settlement = self.polities[pi].settlement;
        let (place, name) = self.place_words(ctx, settlement);
        for (a, slate, organizers) in made {
            for s in slates.iter_mut().filter(|s| a.factions.contains(&s.0)) {
                s.1 = slate.clone();
            }
            let names: Vec<String> = slate.iter().map(|&p| self.name_of(p)).collect();
            let words = format!(
                "{} and {} agreed at {name} to {} for the council: {}.",
                self.name_of(organizers[0]),
                self.name_of(organizers[1]),
                match a.package {
                    Package::Common => "stand on one slate",
                    Package::FirstStandsDown | Package::SecondStandsDown => "back one slate",
                },
                names.join(", ")
            );
            self.chronicle_push(
                ctx.now,
                ChronicleKind::AllianceMade,
                organizers.to_vec(),
                Some(settlement),
                place,
                slate.len() as f64,
                words,
            );
            self.polities[pi].elections[ei].alliances.push(a);
        }
        self.polities[pi].elections[ei].endorsements = slates;
    }

    /// Names close on election `ei`: with nobody standing it fails; with no more standing than
    /// seats, those who stood sit unopposed; by lot, the lot waits for its day; by election, the
    /// polling day comes.
    fn close_names(&mut self, ctx: &mut Ctx, pi: usize, ei: usize) {
        let day = ctx.now.day_index();
        self.endorse_slates(pi, ei);
        self.seek_alliances(ctx, pi, ei);
        let e = &mut self.polities[pi].elections[ei];
        let standing = e.standing();
        if standing.is_empty() {
            e.stage = ElectionStage::NoCandidates;
            e.ended = Some(day);
            self.tell_election(ctx, pi, ei);
            return;
        }
        // Every district's field no larger than its seats: those who stood sit unopposed.
        let contested = e
            .district_seats()
            .into_iter()
            .any(|(d, n)| e.standing_in(d).len() > usize::from(n));
        if !contested {
            e.totals = standing.iter().map(|&c| (c, 0)).collect();
            let winners: Vec<PermanentId> = e
                .district_seats()
                .into_iter()
                .flat_map(|(d, _)| {
                    e.candidates
                        .iter()
                        .filter(move |c| c.district == d)
                        .map(|c| c.person)
                })
                .collect();
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
        // Each district counted apart (M7a slice BQ): one of the whole polity where seats are not
        // shared, whose count is as it was.
        let (mut winners, mut totals, mut by_lot) = (Vec::new(), Vec::new(), false);
        let (mut any_counted, mut any_unpolled) = (false, false);
        for (d, k) in e.district_seats() {
            let standing = e.standing_in(d);
            let n = u32::from(k);
            let key = match d {
                None => Rng64::from_key(&[ctx.seed, polity_id.get(), u64::from(e.id)]),
                Some(d) => Rng64::from_key(&[ctx.seed, polity_id.get(), u64::from(e.id), d.get()]),
            }
            .next_u64();
            if standing.len() <= usize::from(k) {
                // Unopposed in its district.
                totals.extend(standing.iter().map(|&c| (c, 0)));
                winners.extend(standing);
                any_counted = true;
                continue;
            }
            match e.seats.method {
                SeatMethod::Lot => {
                    let drawn = count::draw(&standing, n, key);
                    totals.extend(drawn.iter().map(|&c| (c, 1)));
                    winners.extend(drawn);
                    by_lot = true;
                    any_counted = true;
                }
                SeatMethod::Election => {
                    let ballots: Vec<Vec<PermanentId>> = e
                        .ballots
                        .iter()
                        .filter(|b| {
                            b.1.first()
                                .is_some_and(|c| standing.binary_search(c).is_ok())
                        })
                        .map(|b| b.1.clone())
                        .collect();
                    if ballots.is_empty() {
                        any_unpolled = true;
                        continue;
                    }
                    let r = count::block_vote(&ballots, &standing, n, TieRule::Lot(key));
                    totals.extend(r.totals.votes.iter().copied());
                    by_lot |= r.by_lot;
                    winners.extend(r.winners);
                    any_counted = true;
                }
            }
        }
        e.totals = totals;
        let result = if any_counted {
            Some((winners, by_lot))
        } else if any_unpolled {
            None
        } else {
            Some((Vec::new(), false))
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
        // A lot is drawn, not polled.
        if e.seats.method == SeatMethod::Lot {
            return None;
        }
        // Each polls for their own district's seats, where those are contested (M7a slice BQ).
        let d = e.district_for(hh.settlement);
        let n = e.seats_in(d);
        let standing = e.standing_in(d);
        if standing.len() <= usize::from(n) {
            return None;
        }
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
        let mut utilities: Vec<f64> = standing
            .into_iter()
            .map(|c| self.vote_utility(ctx, person, hh.id, c, e, polity.settlement))
            .collect();
        utilities.sort_by(|a, b| b.total_cmp(a));
        let left_out = utilities
            .get(usize::from(n))
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
        if e.polled(person) || e.seats.method == SeatMethod::Lot {
            return;
        }
        let settlement = polity.settlement;
        let d = e.district_for(self.household(hh).and_then(|x| x.settlement));
        let n = e.seats_in(d);
        let standing = e.standing_in(d);
        if standing.len() <= usize::from(n) {
            return;
        }
        let cands: Vec<(PermanentId, f64)> = standing
            .into_iter()
            .map(|c| (c, self.vote_utility(ctx, person, hh, c, e, settlement)))
            .collect();
        let key =
            Rng64::from_key(&[ctx.seed, polity_id.get(), u64::from(e.id), person.get()]).next_u64();
        let mut choice = ranking(&cands, ctx.params.elections.vote_scale, key);
        choice.truncate(usize::from(n));
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
            m @ (Membership::Organized | Membership::Privileged) => self.closed_member(m, hh),
        }
    }

    /// Who the sponsor of a council of `seats` for polity `pi` expects would sit in it (ADR-0024
    /// §6, a design prior): by election, the n of the electorate `sponsor` knows (a tie, or
    /// themselves) who stand highest in counsel, each district's share from those living there
    /// where seats are shared among settlements; by lot, the whole electorate (its seen stances in
    /// proportion). A change of franchise alone, under a council by election whose last count
    /// they saw, is forecast by counting that election's open ballots again with only the new
    /// electorate's (research 09-08 §1.8; M7a slice BQ).
    pub(super) fn forecast_sitters(
        &self,
        ctx: &Ctx,
        pi: usize,
        seats: &Seats,
        electorate: &[PermanentId],
        sponsor: PermanentId,
    ) -> Vec<PermanentId> {
        if seats.method == SeatMethod::Lot {
            return electorate.to_vec();
        }
        if let Some(out) = self.recount_for_franchise(ctx, pi, seats, electorate) {
            return out;
        }
        let day = ctx.now.day_index();
        let tp = &ctx.params.ties;
        let known: Vec<(f64, PermanentId)> = electorate
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
        let top = |mut pool: Vec<(f64, PermanentId)>, k: usize| -> Vec<PermanentId> {
            pool.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            pool.truncate(k);
            pool.into_iter().map(|x| x.1).collect()
        };
        let polity = &self.polities[pi];
        let mut out = if seats.by_settlement && !polity.held.is_empty() {
            let counts: Vec<(PermanentId, u32)> = polity
                .settlements()
                .into_iter()
                .map(|s| {
                    let n = electorate
                        .iter()
                        .filter(|&&p| self.residence(p) == Some(s))
                        .count();
                    (s, n as u32)
                })
                .collect();
            apportion(seats.n, &counts)
                .into_iter()
                .flat_map(|(s, k)| {
                    let pool: Vec<(f64, PermanentId)> = known
                        .iter()
                        .copied()
                        .filter(|&(_, p)| self.residence(p) == Some(s))
                        .collect();
                    top(pool, usize::from(k))
                })
                .collect()
        } else {
            top(known, usize::from(seats.n))
        };
        out.sort_unstable();
        out
    }

    /// Who the sponsor of a council of `seats` for polity `pi` expects would sit, and that
    /// council's electorate now, in id order: for tests.
    pub fn forecast_sitters_for_tests(
        &self,
        ctx: &Ctx,
        pi: usize,
        seats: &Seats,
        sponsor: PermanentId,
    ) -> (Vec<PermanentId>, Vec<PermanentId>) {
        let electorate: Vec<PermanentId> = self
            .electorate(ctx, pi, seats)
            .into_iter()
            .map(|e| e.0)
            .collect();
        (
            self.forecast_sitters(ctx, pi, seats, &electorate, sponsor),
            electorate,
        )
    }

    /// Those who would have sat had only `electorate` (in id order) polled in polity `pi`'s last
    /// counted election, where `seats` differ from its council's only in who chooses: its open
    /// ballots counted again by the block vote, the same lot settling a tie. `None` where that
    /// does not apply, or nobody of the new electorate polled.
    fn recount_for_franchise(
        &self,
        ctx: &Ctx,
        pi: usize,
        seats: &Seats,
        electorate: &[PermanentId],
    ) -> Option<Vec<PermanentId>> {
        let polity = &self.polities[pi];
        let current = polity.body.seats?;
        let same_but_franchise = current.method == SeatMethod::Election
            && current.n == seats.n
            && current.term_days == seats.term_days
            && !current.by_settlement
            && !seats.by_settlement
            && current.electorate != seats.electorate;
        if !same_but_franchise {
            return None;
        }
        let e = polity
            .elections
            .iter()
            .rev()
            .find(|e| e.stage == ElectionStage::Installed && e.seats == current)?;
        let ballots: Vec<Vec<PermanentId>> = e
            .ballots
            .iter()
            .filter(|b| electorate.binary_search(&b.0).is_ok())
            .map(|b| b.1.clone())
            .collect();
        if ballots.is_empty() {
            return None;
        }
        let key = Rng64::from_key(&[ctx.seed, polity.id.get(), u64::from(e.id)]).next_u64();
        let r = count::block_vote(
            &ballots,
            &e.standing(),
            u32::from(seats.n),
            TieRule::Lot(key),
        );
        let mut out = r.winners;
        out.sort_unstable();
        Some(out)
    }
}

/// One slate of two factions' (`a`, `b`, each in its own order), followed by `fa` and `fb`: up to
/// `n` names, each next one from the side with the more following for each name it has already
/// (D'Hondt's rule between the two), the first side on a tie, until both are out of names.
fn common_slate(a: &[PermanentId], b: &[PermanentId], fa: u32, fb: u32, n: u8) -> Vec<PermanentId> {
    let (mut ia, mut ib) = (0usize, 0usize);
    let mut out = Vec::new();
    while out.len() < usize::from(n) && (ia < a.len() || ib < b.len()) {
        let qa = f64::from(fa) / (ia as f64 + 1.0);
        let qb = f64::from(fb) / (ib as f64 + 1.0);
        if ib >= b.len() || (ia < a.len() && qa >= qb) {
            out.push(a[ia]);
            ia += 1;
        } else {
            out.push(b[ib]);
            ib += 1;
        }
    }
    out
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
            by_settlement: false,
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
    fn a_common_slate_shares_its_names_by_following() {
        let a = [id(1), id(2), id(3), id(4)];
        let b = [id(11), id(12), id(13)];
        // Followings 6 and 3, quotients 6, 3, 2, 1.5 against 3, 1.5: names go 1, then 2 (a tie
        // at 3 to the first side), 11, 3, then 4 (a tie at 1.5 to the first side).
        assert_eq!(
            common_slate(&a, &b, 6, 3, 5),
            vec![id(1), id(2), id(11), id(3), id(4)]
        );
        // One side out of names: the other fills the rest.
        assert_eq!(
            common_slate(&[id(1)], &b, 9, 1, 3),
            vec![id(1), id(11), id(12)]
        );
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
        // The founding custom's quorum is a quarter of its adults; a council's is of its seats:
        // the two shares are not compared, and taking seats stays one change (M7a slice BQ).
        let founding = Body {
            quorum_share: 0.25,
            ..gathering
        };
        assert_eq!(body.changes_from(&founding), 1);
        assert_eq!(founding.changes_from(&body), 1);
        let half = Body {
            quorum_share: 0.25,
            ..body
        };
        assert_eq!(
            half.changes_from(&body),
            1,
            "between councils, quorum is a change"
        );
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
