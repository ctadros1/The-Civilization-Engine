//! Seated bodies and elections (M7a slice BO, ADR-0024 §1, §3): a body of n seats filled from an
//! electorate by election or by lot for a term, and each election as a chain of dated steps,
//! every failure a recorded outcome. Pure records and the voter's ranking: the stepping lives in
//! `population/elections.rs`.

use civ_core::{PermanentId, Rng64};

use crate::polity::Membership;

/// How a seated body's seats are filled (ADR-0024 §1). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeatMethod {
    /// Those of the electorate who come to the hearth on the polling day choose among those who
    /// stood; the block vote counts it (each names up to n, the n named most sit).
    Election = 0,
    /// A keyed lot draws n among those who put their names in (ADR-0011 §3).
    Lot = 1,
}

impl SeatMethod {
    /// Every method, in code order.
    pub const ALL: [SeatMethod; 2] = [SeatMethod::Election, SeatMethod::Lot];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The method numbered `code`.
    pub fn from_code(code: u8) -> Option<SeatMethod> {
        SeatMethod::ALL.get(usize::from(code)).copied()
    }

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            SeatMethod::Election => "election",
            SeatMethod::Lot => "lot",
        }
    }

    /// The method with an authored name.
    pub fn from_name(name: &str) -> Option<SeatMethod> {
        SeatMethod::ALL.into_iter().find(|m| m.name() == name)
    }
}

/// A seated body's seats (ADR-0024 §1): how many, who chooses them and may stand (one membership
/// in v0: the candidates eligible are the electorate), how they are filled and for how long.
/// Stances in a seated body are open, as at the gathering; secret ballots wait (ADR-0024 "not
/// decided here").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seats {
    /// Seats (content offers 5, 9 and 15; research 09-05 §2.3).
    pub n: u8,
    /// Who chooses and who may stand.
    pub electorate: Membership,
    /// How the seats are filled.
    pub method: SeatMethod,
    /// Days a term runs from installation.
    pub term_days: u16,
}

impl Seats {
    /// Packed into one number for saves: 0 is no seats (a plenary body).
    pub fn pack(seats: Option<Seats>) -> u64 {
        seats.map_or(0, |s| {
            u64::from(s.n)
                | (u64::from(s.electorate.code()) << 8)
                | (u64::from(s.method.code()) << 16)
                | (u64::from(s.term_days) << 24)
        })
    }

    /// The seats packed as `v`: `Ok(None)` for 0, `Err` for a number no seats pack to.
    pub fn unpack(v: u64) -> Result<Option<Seats>, String> {
        if v == 0 {
            return Ok(None);
        }
        let n = (v & 0xff) as u8;
        let electorate = Membership::from_code(((v >> 8) & 0xff) as u8)
            .filter(|&m| m != Membership::Seats)
            .ok_or_else(|| format!("seats {v:#x}: no such electorate"))?;
        let method = SeatMethod::from_code(((v >> 16) & 0xff) as u8)
            .ok_or_else(|| format!("seats {v:#x}: no such method"))?;
        let term_days = ((v >> 24) & 0xffff) as u16;
        if n == 0 || term_days == 0 || v >> 40 != 0 {
            return Err(format!("seats {v:#x}: no seats or no term"));
        }
        Ok(Some(Seats {
            n,
            electorate,
            method,
            term_days,
        }))
    }

    /// In words: "a council of 9, elected by the adults for 360 days".
    pub fn words(&self) -> String {
        let how = match self.method {
            SeatMethod::Election => "elected by",
            SeatMethod::Lot => "drawn by lot from",
        };
        format!(
            "a council of {}, {how} the {} for {} days",
            self.n,
            self.electorate.noun(),
            self.term_days
        )
    }
}

/// A seat held (ADR-0024 §1: holders by seat, with their seat's since and until).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seat {
    /// The seat's number, from 0.
    pub seat: u8,
    /// Who sits in it.
    pub holder: PermanentId,
    /// The day they took it.
    pub since: i64,
    /// The day their term ends.
    pub until: i64,
    /// The election that seated them.
    pub election: u32,
}

/// Why an election was called (ADR-0024 §3). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElectionWhy {
    /// The custom was amended (or taken) to a seated body with nobody seated under it.
    Amendment = 0,
    /// The seats' term is ending.
    TermEnding = 1,
}

impl ElectionWhy {
    /// Every reason, in code order.
    pub const ALL: [ElectionWhy; 2] = [ElectionWhy::Amendment, ElectionWhy::TermEnding];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The reason numbered `code`.
    pub fn from_code(code: u8) -> Option<ElectionWhy> {
        ElectionWhy::ALL.get(usize::from(code)).copied()
    }

    /// In words, after "called because".
    pub fn words(self) -> &'static str {
        match self {
            ElectionWhy::Amendment => "the custom became a council",
            ElectionWhy::TermEnding => "the council's term is ending",
        }
    }
}

/// Where an election stands, or how it ended (ADR-0024 §3: each failure its record). Codes are
/// part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElectionStage {
    /// Called; those of the electorate who hear of it may stand until `stand_until`.
    Standing = 0,
    /// Standing is closed; the polling day is to come (by election), or the lot is to be drawn.
    Polling = 1,
    /// Counted and those chosen seated.
    Installed = 2,
    /// Nobody stood: the seats stay empty until the next election.
    NoCandidates = 3,
    /// Nobody came to poll: the seats stay empty until the next election.
    NoVoters = 4,
    /// A tie for the last seats the procedure could not break (never under a lot tie rule).
    Tied = 5,
    /// The custom changed before it was done: no longer a council, or another one.
    Abandoned = 6,
}

impl ElectionStage {
    /// Every stage, in code order.
    pub const ALL: [ElectionStage; 7] = [
        ElectionStage::Standing,
        ElectionStage::Polling,
        ElectionStage::Installed,
        ElectionStage::NoCandidates,
        ElectionStage::NoVoters,
        ElectionStage::Tied,
        ElectionStage::Abandoned,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The stage numbered `code`.
    pub fn from_code(code: u8) -> Option<ElectionStage> {
        ElectionStage::ALL.get(usize::from(code)).copied()
    }

    /// Whether it is still running.
    pub fn open(self) -> bool {
        matches!(self, ElectionStage::Standing | ElectionStage::Polling)
    }

    /// In words.
    pub fn words(self) -> &'static str {
        match self {
            ElectionStage::Standing => "open for names",
            ElectionStage::Polling => "to be polled",
            ElectionStage::Installed => "counted and seated",
            ElectionStage::NoCandidates => "failed: nobody stood",
            ElectionStage::NoVoters => "failed: nobody came to poll",
            ElectionStage::Tied => "failed: a tie for the last seats",
            ElectionStage::Abandoned => "abandoned: the custom changed",
        }
    }
}

/// One who stood (ADR-0024 §3: standing is a choice), and where they said they stand (ADR-0024
/// §4, M7a slice BP): their own positions, 0 against to 1 for, on the questions that mattered most
/// to them when they stood (at most `ElectionParams::positions_told`), told with their name.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub person: PermanentId,
    /// The day they put their name in.
    pub day: i64,
    /// (question, by the policy template that asks it; their position), most salient first.
    pub positions: Vec<(u16, f32)>,
}

/// An election (ADR-0024 §3): called, told, open for names, polled or drawn, counted, seated.
/// Ballots are open, as stances at the gathering are: who named whom is kept.
#[derive(Clone, Debug, PartialEq)]
pub struct Election {
    /// Its number within its polity, from 1.
    pub id: u32,
    /// Why it was called.
    pub why: ElectionWhy,
    /// The seats it fills, as the custom had them when it was called.
    pub seats: Seats,
    /// The day it was called.
    pub called: i64,
    /// The last day a name may be put in.
    pub stand_until: i64,
    /// The polling day (by election) or the day the lot is drawn.
    pub poll_day: i64,
    /// Where it stands.
    pub stage: ElectionStage,
    /// Those of the electorate who heard of it and chose whether to stand, in id order: each
    /// chooses once.
    pub considered: Vec<PermanentId>,
    /// Those who stood, in the order they put their names in.
    pub candidates: Vec<Candidate>,
    /// Each who came to poll and whom they named, first choice first, in the order they came.
    pub ballots: Vec<(PermanentId, Vec<PermanentId>)>,
    /// The count: each candidate's votes (or 1 for each drawn), most first.
    pub totals: Vec<(PermanentId, u32)>,
    /// Those seated, in seat order.
    pub winners: Vec<PermanentId>,
    /// Whether a lot settled a tie for the last seats.
    pub by_lot: bool,
    /// The day it was counted and those chosen seated, or failed.
    pub ended: Option<i64>,
    /// The day the seats it fills end: the next election is called before it.
    pub until: i64,
    /// Each faction's slate (ADR-0024 §5, M7a slice BP): the faction and those of its members
    /// who stood whom its organizer endorsed, in faction order.
    pub endorsements: Vec<(PermanentId, Vec<PermanentId>)>,
}

impl Election {
    /// Whether `person` stood.
    pub fn stood(&self, person: PermanentId) -> bool {
        self.candidates.iter().any(|c| c.person == person)
    }

    /// Whether faction `faction` endorsed `person`.
    pub fn endorsed(&self, faction: PermanentId, person: PermanentId) -> bool {
        self.endorsements
            .iter()
            .any(|(f, slate)| *f == faction && slate.contains(&person))
    }

    /// The positions `person` told when they stood, if they stood.
    pub fn positions_of(&self, person: PermanentId) -> Option<&[(u16, f32)]> {
        self.candidates
            .iter()
            .find(|c| c.person == person)
            .map(|c| c.positions.as_slice())
    }

    /// Whether `person` came to poll.
    pub fn polled(&self, person: PermanentId) -> bool {
        self.ballots.iter().any(|b| b.0 == person)
    }

    /// Those who stood, in id order.
    pub fn standing(&self) -> Vec<PermanentId> {
        let mut out: Vec<PermanentId> = self.candidates.iter().map(|c| c.person).collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// How elections run (the people profile's `[elections]`, content API 106; ADR-0024 §3). Design
/// priors unless cited.
#[derive(Clone, Debug, PartialEq)]
pub struct ElectionParams {
    /// Days names may be put in after an election is called.
    pub stand_days: u32,
    /// Minutes a voter spends at the hearth naming their choices.
    pub poll_minutes: u32,
    /// Points for each unit of household gain (stake over `polity.w_gain`) a seat would turn
    /// toward the candidate's household: the decisions they saw go against them.
    pub w_sit: f64,
    /// Points for standing in counsel the candidate holds, as the regard a seat may bring.
    pub w_standing: f64,
    /// Points against standing: the evenings in council and the walk, over a term.
    pub sit_cost: f64,
    /// Temperature of the choice to stand, points.
    pub stand_temperature: f64,
    /// A voter's utility for a candidate: regard for them (ADR-0024 §3, loyalty).
    pub w_regard: f64,
    /// For one of their own household (group).
    pub w_household: f64,
    /// For their standing in counsel (competence).
    pub w_counsel: f64,
    /// For themselves, when they stood.
    pub w_self: f64,
    /// The scale of the one softmax a voter's ranking is drawn at, utility points.
    pub vote_scale: f64,
    /// Points for coming to poll before the stake (a duty norm, where content authors one, adds
    /// to it).
    pub poll_base: f64,
    /// Points per point of the stake: how much better the first choice is than the one after the
    /// last seat.
    pub w_poll_stake: f64,
    /// The questions a candidate tells where they stand on, most salient first (ADR-0024 §4: up
    /// to three).
    pub positions_told: u8,
    /// A voter's utility against the mean distance between their positions and those a candidate
    /// told, on the questions both hold (09-08 §1.3, D): unknown is no term.
    pub w_distance: f64,
    /// For the candidate's record as the voter saw it: how their stances at decisions both came
    /// to lined up with the voter's household's stake, −1 to 1 (09-08 §1.3, R).
    pub w_record: f64,
    /// For a candidate their own faction endorsed (ADR-0024 §5).
    pub w_slate: f64,
    /// Points a candidate finds in an evening at the hearth while their election runs, where they
    /// tell that they stand and where (ADR-0024 §4: a campaign is hours and claims).
    pub w_campaign: f64,
}

impl ElectionParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> ElectionParams {
        ElectionParams {
            stand_days: 7,
            poll_minutes: 30,
            w_sit: 1.0,
            w_standing: 0.5,
            sit_cost: 1.0,
            stand_temperature: 0.5,
            w_regard: 1.0,
            w_household: 2.0,
            w_counsel: 0.5,
            w_self: 3.0,
            vote_scale: 1.0,
            poll_base: 0.5,
            w_poll_stake: 0.5,
            positions_told: 3,
            w_distance: 2.0,
            w_record: 1.0,
            w_slate: 1.0,
            w_campaign: 1.0,
        }
    }
}

/// A polity's council in words, for the Government panel's offices (M7a slice BO): each seat
/// held ("Seat 1 of 9: Bo, since 3 May of year 2, until 3 May of year 3"), the empty ones, and
/// its latest election and how it stands or ended. `name_of` names people and `day` renders a
/// day index. Nothing when it is no council and has held no election.
pub fn council_words(
    seats: Option<Seats>,
    seated: &[Seat],
    elections: &[Election],
    name_of: &dyn Fn(PermanentId) -> String,
    faction_of: &dyn Fn(PermanentId) -> String,
    day: &dyn Fn(i64) -> String,
) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(s) = seats {
        for k in 0..s.n {
            match seated.iter().find(|x| x.seat == k) {
                Some(x) => out.push(format!(
                    "Seat {} of {}: {}, since {}, until {}",
                    k + 1,
                    s.n,
                    name_of(x.holder),
                    day(x.since),
                    day(x.until)
                )),
                None => out.push(format!("Seat {} of {}: empty", k + 1, s.n)),
            }
        }
    }
    if let Some(e) = elections.last() {
        let stood = e.standing().len();
        let what = match e.stage {
            ElectionStage::Standing => format!("names may be put in until {}", day(e.stand_until)),
            ElectionStage::Polling => format!(
                "{stood} stand for {} seats; {} on {}",
                e.seats.n,
                match e.seats.method {
                    SeatMethod::Election => "the polling is",
                    SeatMethod::Lot => "the lot is drawn",
                },
                day(e.poll_day)
            ),
            ElectionStage::Installed => {
                let counted: Vec<String> = e
                    .totals
                    .iter()
                    .map(|&(c, v)| match e.seats.method {
                        SeatMethod::Election if !e.ballots.is_empty() => {
                            format!("{} {v}", name_of(c))
                        }
                        _ => name_of(c),
                    })
                    .collect();
                format!(
                    "{}, {stood} standing{}: {}",
                    e.stage.words(),
                    if e.ballots.is_empty() {
                        String::new()
                    } else {
                        format!(", {} came to poll", e.ballots.len())
                    },
                    counted.join(", ")
                )
            }
            stage => stage.words().to_owned(),
        };
        out.push(format!(
            "Election {} (called {} because {}): {what}",
            e.id,
            day(e.called),
            e.why.words()
        ));
        // The factions' slates in it (M7a slice BP).
        for (f, slate) in &e.endorsements {
            let named: Vec<String> = slate.iter().map(|&p| name_of(p)).collect();
            out.push(format!("{} backs {}", faction_of(*f), named.join(", ")));
        }
    }
    out
}

/// A voter's ranking of `candidates` by `utility` (ADR-0024 §3: one evaluation, one softmax at a
/// fixed scale; the block vote takes the top n of the same ranking): each utility perturbed by a
/// Gumbel draw at `scale`, keyed by `key`, highest first, so the first choice falls by the
/// softmax and the rest follow it without replacement.
pub fn ranking(candidates: &[(PermanentId, f64)], scale: f64, key: u64) -> Vec<PermanentId> {
    let mut rng = Rng64::from_key(&[key, PURPOSE_BALLOT]);
    let mut scored: Vec<(f64, PermanentId)> = candidates
        .iter()
        .map(|&(c, u)| {
            let x = rng.next_f64().clamp(1e-12, 1.0 - 1e-12);
            (u + scale.max(0.0) * -(-x.ln()).ln(), c)
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|s| s.1).collect()
}

/// Keeps a ballot's draws apart from every other draw by the same key.
pub const PURPOSE_BALLOT: u64 = 0x6261_6c6c_6f74_3031; // "ballot01"

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seats_pack_and_unpack_alike_and_bad_numbers_are_refused() {
        let s = Seats {
            n: 9,
            electorate: Membership::Elders,
            method: SeatMethod::Lot,
            term_days: 360,
        };
        assert_eq!(Seats::pack(None), 0);
        assert_eq!(Seats::unpack(0), Ok(None));
        assert_eq!(Seats::unpack(Seats::pack(Some(s))), Ok(Some(s)));
        assert!(Seats::unpack(1 << 50).is_err());
        assert!(Seats::unpack(9).is_err(), "no term");
        assert!(Seats::unpack(u64::from(Membership::Seats.code()) << 8 | 9 | 30 << 24).is_err());
        assert_eq!(
            s.words(),
            "a council of 9, drawn by lot from the elders for 360 days"
        );
    }

    #[test]
    fn a_ranking_follows_utility_at_no_scale_and_is_keyed() {
        let id = |n| PermanentId::from_raw(n).expect("non-zero");
        let (a, b, c) = (id(1), id(2), id(3));
        let cands = [(a, 0.0), (b, 2.0), (c, 1.0)];
        assert_eq!(ranking(&cands, 0.0, 7), vec![b, c, a]);
        assert_eq!(ranking(&cands, 1.0, 7), ranking(&cands, 1.0, 7));
        // At a broad scale the first choice is not always the best: about e^2 : e^1 : e^0.
        let firsts = (0..3000u64)
            .filter(|&k| ranking(&cands, 1.0, k)[0] == b)
            .count();
        let p = (2f64).exp() / ((2f64).exp() + 1f64.exp() + 1.0);
        assert!(
            (firsts as f64 / 3000.0 - p).abs() < 0.03,
            "{firsts} of 3000 against {p}"
        );
    }
}
