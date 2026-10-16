//! Force between polities (M6c slice BJ, ADR-0023 §3-4): bands and the encounters they meet.
//!
//! A band is people who chose to go, after someone chose to call it (13-05 §1.2); this module
//! keeps its record and the encounter it meets. An encounter has two sides. Every minute each
//! person on it still standing either stays or gives way, by blows they saw near them in the last
//! minutes, others of their side giving way, their own hurts and their regard for the one who
//! leads them (13-07 §1.6: no casualty breakpoint). Pairs form up to the place's frontage (13-07
//! §1.5: E = min(N_available, K_usable)); each pair exchanges blows at a hazard set by what each
//! holds, and both strike from the same pre-step state. It ends when a side has nobody standing.
//! Nothing here is drawn but whether a blow lands and how hard it falls; there is no battle
//! score, and Lanchester's laws are tests, not the model (13-07 §1.2).

use civ_core::{PermanentId, Rng64};

use crate::crime::Harm;

/// Purpose tag for whether a blow lands in a minute.
pub const PURPOSE_LAND: u64 = 0x6c61_6e64_626c_6f77; // "landblow"
/// Purpose tag for how hard a landed blow falls.
pub const PURPOSE_FALL: u64 = 0x6661_6c6c_626c_6f77; // "fallblow"
/// Purpose tag for whether a landed blow kills.
pub const PURPOSE_SLAY: u64 = 0x736c_6179_626c_6f77; // "slayblow"

/// What a band set out to do (ADR-0023 §3). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BandPurpose {
    /// Food from stores its people have seen.
    Take = 0,
    /// What a household believes it is owed by a household of another settlement that took
    /// from it.
    TakeBack = 1,
}

impl BandPurpose {
    /// Every purpose, in code order.
    pub const ALL: [BandPurpose; 2] = [BandPurpose::Take, BandPurpose::TakeBack];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The purpose numbered `code`.
    pub fn from_code(code: u8) -> Option<BandPurpose> {
        BandPurpose::ALL.get(usize::from(code)).copied()
    }

    /// In words: "to take food".
    pub fn words(self) -> &'static str {
        match self {
            BandPurpose::Take => "to take food",
            BandPurpose::TakeBack => "to take back what was taken",
        }
    }
}

/// Where a band is (ADR-0023 §3: stages saved and dated). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BandStage {
    /// Called: those who joined are to come to the hearth at the muster.
    Called = 0,
    /// Out: those who came at the muster have set off.
    Out = 1,
    /// Home: it met what it met and its people have come home.
    Home = 2,
    /// Dispersed before setting off: too few came to the muster.
    Dispersed = 3,
    /// Camped by a shut line round the place it went to (M6c slice BL, ADR-0023 §7): those who
    /// came wait outside it, each staying or going home as they choose.
    Camped = 4,
    /// Holding the settlement it went to (M6c slice BM, ADR-0023 §9): sent by law, it held the
    /// place at its encounter and its people stay there, sleeping where they keep it, each
    /// choosing to stay or go home.
    Holding = 5,
}

impl BandStage {
    /// Every stage, in code order.
    pub const ALL: [BandStage; 6] = [
        BandStage::Called,
        BandStage::Out,
        BandStage::Home,
        BandStage::Dispersed,
        BandStage::Camped,
        BandStage::Holding,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The stage numbered `code`.
    pub fn from_code(code: u8) -> Option<BandStage> {
        BandStage::ALL.get(usize::from(code)).copied()
    }

    /// Whether it is over.
    pub fn over(self) -> bool {
        matches!(self, BandStage::Home | BandStage::Dispersed)
    }
}

/// A band (ADR-0023 §3): who called it and why, who joined and who came, where it is, and what
/// it met, took and lost.
#[derive(Clone, Debug, PartialEq)]
pub struct Band {
    /// Its number, from 1.
    pub id: u32,
    /// Who called it.
    pub organizer: PermanentId,
    /// Their settlement, where it musters.
    pub settlement: PermanentId,
    /// What for.
    pub purpose: BandPurpose,
    /// The household it goes to, and where its home stood when the band was called.
    pub target: PermanentId,
    pub place: (f32, f32),
    /// The minute it was called, and the minute it musters.
    pub called: i64,
    pub muster: i64,
    /// Who joined, in the order they chose to.
    pub members: Vec<PermanentId>,
    /// Who were asked and chose not to, in id order (M6c slice BJ, step two).
    pub asked: Vec<PermanentId>,
    /// Who were at the hearth at the muster, in id order.
    pub mustered: Vec<PermanentId>,
    /// Who of those came to the target, in the order they came.
    pub arrived: Vec<PermanentId>,
    /// Who of those are home again.
    pub home: Vec<PermanentId>,
    /// Where it is, and the minute it got there.
    pub stage: BandStage,
    pub since: i64,
    /// The encounter it met, if it met one, and the incident it was recorded as.
    pub clash: Option<u32>,
    pub incident: Option<u32>,
    /// The law that sent it, if a law did (M6c slice BJ, step three).
    pub law: Option<PermanentId>,
    /// What it took, `(good, amount)`, and its food energy, kcal.
    pub taken: Vec<(u16, f32)>,
    pub taken_kcal: f32,
    /// Where it comes up to the line round the place it goes to, if that place lay inside one
    /// when it was called (M6c slice BL): it walks there first, and camps there if the line
    /// bars it.
    pub approach: Option<(f32, f32)>,
    /// The day its people had held the settlement it went to `bands.hold_days`, if they did (M6c
    /// slice BM, ADR-0023 §9: ADR-0017 §4's test applied to a band). A hold changes nothing by
    /// itself.
    pub held: Option<i64>,
}

impl Band {
    /// Whether `p` joined it.
    pub fn joined(&self, p: PermanentId) -> bool {
        self.members.contains(&p)
    }
}

/// Which side someone stood on (ADR-0023 §4). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    /// The band.
    Band = 0,
    /// Those who stood against it: the household, neighbours who came, the watch.
    Defence = 1,
}

impl Side {
    /// Both sides, in code order.
    pub const ALL: [Side; 2] = [Side::Band, Side::Defence];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The side numbered `code`.
    pub fn from_code(code: u8) -> Option<Side> {
        Side::ALL.get(usize::from(code)).copied()
    }
}

/// What someone holds into a fight (ADR-0023 §4: equipment is goods; it sets the hazard, never a
/// damage multiplier). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Holding {
    /// Their hands, or whatever comes to them.
    Hands = 0,
    /// An axe.
    Axe = 1,
    /// A spear.
    Spear = 2,
}

impl Holding {
    /// Every holding, in code order.
    pub const ALL: [Holding; 3] = [Holding::Hands, Holding::Axe, Holding::Spear];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The holding numbered `code`.
    pub fn from_code(code: u8) -> Option<Holding> {
        Holding::ALL.get(usize::from(code)).copied()
    }

    /// In words: "a spear".
    pub fn words(self) -> &'static str {
        match self {
            Holding::Hands => "bare hands",
            Holding::Axe => "an axe",
            Holding::Spear => "a spear",
        }
    }
}

/// How an encounter ended (ADR-0023 §4). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ended {
    /// Nobody stood against the band.
    NobodyStood = 0,
    /// Those against it gave way, or fell.
    DefenceGaveWay = 1,
    /// The band gave way, or fell.
    BandGaveWay = 2,
    /// Both still stood at the longest an encounter runs, and each went its way.
    Parted = 3,
}

impl Ended {
    /// Every ending, in code order.
    pub const ALL: [Ended; 4] = [
        Ended::NobodyStood,
        Ended::DefenceGaveWay,
        Ended::BandGaveWay,
        Ended::Parted,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The ending numbered `code`.
    pub fn from_code(code: u8) -> Option<Ended> {
        Ended::ALL.get(usize::from(code)).copied()
    }

    /// Whether the band may take what it came for.
    pub fn band_holds(self) -> bool {
        matches!(self, Ended::NobodyStood | Ended::DefenceGaveWay)
    }
}

/// Someone in an encounter: who, on which side, holding what, how much they wanted to stand
/// there when it began (points), and what they regard the one who leads their side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
    pub person: PermanentId,
    pub side: Side,
    pub holds: Holding,
    pub will: f32,
    pub regard_leader: f32,
    /// The minute they gave way, if they did.
    pub gave_way: Option<u16>,
    /// Days of hurt the encounter left them, and whether it killed them.
    pub hurt_days: u16,
    pub killed: bool,
}

/// A blow in an encounter, with the minute it fell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Struck {
    pub minute: u16,
    pub harm: Harm,
}

/// An encounter between a band and those who stood against it (ADR-0023 §4).
#[derive(Clone, Debug, PartialEq)]
pub struct Clash {
    /// Its number, from 1.
    pub id: u32,
    /// The minute it began, and where.
    pub at: i64,
    pub place: (f32, f32),
    /// The band.
    pub band: u32,
    /// Those in it, the band first, each side in id order.
    pub fighters: Vec<Fighter>,
    /// Every blow, in the order they fell.
    pub blows: Vec<Struck>,
    /// The pairs the place allowed at once.
    pub frontage: u32,
    /// The minutes it ran, and how it ended.
    pub minutes: u16,
    pub ended: Ended,
}

/// Who calls a band, who joins it and how it runs (the people profile's `[bands]`, content API
/// 96; ADR-0023 §3). Design priors, each named in the content; 13-05 §1.2 gives the terms, not
/// their weights.
#[derive(Clone, Debug, PartialEq)]
pub struct BandParams {
    /// Days between someone's reviews of calling a band (13-05 §2.4 tests 7-30); 0 calls none.
    pub review_days: u32,
    /// Ages that call, join and go.
    pub min_age: f64,
    pub max_age: f64,
    /// The longest walk a band goes, minutes one way.
    pub max_walk_minutes: f64,
    /// The share of a load someone hopes to find in a store of a settlement they have seen whose
    /// food they never saw.
    pub food_prior: f64,
    /// Points for each day of their household's food a load would bring home.
    pub w_spoils: f64,
    /// Points for each unit of grievance against the household it goes to.
    pub w_grievance: f64,
    /// Points against for each defender believed, shared among those who go.
    pub w_danger: f64,
    /// Points against for the objection to taking (0-1).
    pub w_objection: f64,
    /// Points for regard for the organizer, and for those who have joined.
    pub w_regard: f64,
    pub w_follow: f64,
    /// The range a person's reluctance to call, and to join, is drawn in.
    pub call_threshold: [f64; 2],
    pub join_threshold: [f64; 2],
    /// The least regard the organizer holds for someone to tell them, and the most they tell.
    pub join_regard: f64,
    pub told_most: u32,
    /// The fewest at the muster for a band to set off (the organizer counts).
    pub least: u32,
    /// The hour it musters, the day after it is called.
    pub muster_hour: u32,
    /// Points going is worth to one who said they would.
    pub w_promise: f64,
    /// Minutes it stays at the place it went to.
    pub stay_minutes: u32,
    /// Minutes between looks by those about at a band walking (a band seen, ADR-0023 §8).
    pub look_minutes: u32,
    /// Units of the act `stayed_home` written by each who went toward one who said they would
    /// come and did not, and toward one who was asked and said no.
    pub shirk_promised: f64,
    pub shirk_asked: f64,
    /// Laws (M6c slice BJ, step three): points against calling or joining a band for one who
    /// knows a law against raiding in force; days of a household's food the one it sends with a
    /// band a law sent is reckoned away and at risk, which it weighs as a levy; and the share of
    /// what outsiders took from a household it believes its own people's raids bring on it.
    pub w_law: f64,
    pub service_days: f64,
    pub retaliation_share: f64,
    /// Will to stay (points): of one who went with the band; of one standing at their own home;
    /// of a neighbour, at full regard for the household come to.
    pub will_band: f64,
    pub will_home: f64,
    pub will_neighbour: f64,
    /// Camps (M6c slice BL; content API 102): the minutes between each who camps choosing to
    /// stay or go home, and what staying costs them: points a day camped, and points at a full
    /// day's want of sleep.
    pub camp_minutes: u32,
    pub w_camp_day: f64,
    pub w_camp_tired: f64,
    /// Holds (M6c slice BM; content API 104): the days a band sent by law must keep a settlement
    /// it took the field at for the hold to stand; the want of sleep at which one staying there
    /// lies down where they are; and how long they sleep, minutes.
    pub hold_days: u32,
    pub sleep_out_at: f64,
    pub sleep_out_minutes: u32,
    /// How encounters run.
    pub fight: FightParams,
}

impl BandParams {
    /// Whether someone of `age` years may go with a band (`min_age` to `max_age`): who may lead
    /// one, join one, or be sent by a law (M6c slice BJ).
    pub fn can_go(&self, age: f64) -> bool {
        age >= self.min_age && age <= self.max_age
    }

    /// The core content's values, for tests.
    pub fn core() -> Self {
        BandParams {
            review_days: 30,
            min_age: 15.0,
            max_age: 60.0,
            max_walk_minutes: 120.0,
            food_prior: 0.5,
            w_spoils: 0.5,
            w_grievance: 2.0,
            w_danger: 1.5,
            w_objection: 4.0,
            w_regard: 2.0,
            w_follow: 1.0,
            call_threshold: [1.0, 4.0],
            join_threshold: [0.5, 3.0],
            join_regard: 0.3,
            told_most: 12,
            least: 2,
            muster_hour: 7,
            w_promise: 20.0,
            stay_minutes: 30,
            look_minutes: 10,
            shirk_promised: 2.0,
            shirk_asked: 1.0,
            w_law: 3.0,
            service_days: 2.0,
            retaliation_share: 0.5,
            will_band: 2.0,
            will_home: 3.0,
            will_neighbour: 2.0,
            camp_minutes: 120,
            w_camp_day: 1.0,
            w_camp_tired: 2.0,
            hold_days: 7,
            sleep_out_at: 0.6,
            sleep_out_minutes: 360,
            fight: FightParams::core(),
        }
    }
}

/// How encounters run (the people profile's `[bands.fight]`, content API 96). Design priors: the
/// research gives a structure (13-07 §1.5-1.7, §2.3) and checks (13-07 §4), not these values.
#[derive(Clone, Debug, PartialEq)]
pub struct FightParams {
    /// The longest an encounter runs, minutes.
    pub max_minutes: u16,
    /// Pairs that can fight at once at a household's home (its door and yard).
    pub frontage_home: u32,
    /// The chance a paired fighter lands a blow in a minute, by what they hold.
    pub land_hands: f64,
    pub land_axe: f64,
    pub land_spear: f64,
    /// The range of days a landed blow keeps someone from work, and the share that kill.
    pub hurt_days: [u16; 2],
    pub kill_share: f64,
    /// Will to stay lost for each blow someone saw their own side take in the last
    /// `seen_minutes` minutes.
    pub will_seen: f64,
    pub seen_minutes: u16,
    /// Will lost by the share of one's own side who have given way.
    pub will_gave: f64,
    /// Will lost for each day of hurt one has taken.
    pub will_hurt: f64,
    /// Will kept for regard for the one who leads one's side, while they stand.
    pub will_leader: f64,
}

impl FightParams {
    /// The core content's values, for tests.
    pub fn core() -> Self {
        FightParams {
            max_minutes: 60,
            frontage_home: 3,
            land_hands: 0.03,
            land_axe: 0.06,
            land_spear: 0.08,
            hurt_days: [3, 21],
            kill_share: 0.02,
            will_seen: 0.5,
            seen_minutes: 5,
            will_gave: 2.0,
            will_hurt: 0.2,
            will_leader: 1.0,
        }
    }

    /// The chance someone holding `h` lands a blow in a minute.
    pub fn land(&self, h: Holding) -> f64 {
        match h {
            Holding::Hands => self.land_hands,
            Holding::Axe => self.land_axe,
            Holding::Spear => self.land_spear,
        }
    }
}

/// Runs an encounter among `fighters` (each side's leader the first of it, if `leads`), at a place
/// allowing `frontage` pairs, beginning at minute `start` with draws keyed by `seed`. Fills in who
/// gave way and when, hurts and deaths, and returns every blow, the minutes it ran and how it
/// ended. Pairs form in order of who most wants to stand there, then by id; unpaired people
/// strike nobody. Both sides strike from the state before the minute.
pub fn fight(
    fighters: &mut [Fighter],
    frontage: u32,
    leads: [Option<PermanentId>; 2],
    start: i64,
    seed: u64,
    params: &FightParams,
) -> (Vec<Struck>, u16, Ended) {
    let mut blows: Vec<Struck> = Vec::new();
    let standing = |f: &Fighter| f.gave_way.is_none() && !f.killed;
    let side_count = |fs: &[Fighter], s: Side| fs.iter().filter(|f| f.side == s).count();
    if side_count(fighters, Side::Defence) == 0 {
        return (blows, 0, Ended::NobodyStood);
    }
    if side_count(fighters, Side::Band) == 0 {
        return (blows, 0, Ended::BandGaveWay);
    }
    for minute in 0..params.max_minutes {
        let mut lines: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
        for (i, f) in fighters.iter().enumerate() {
            if standing(f) {
                lines[f.side as usize].push(i);
            }
        }
        if lines[Side::Band as usize].is_empty() {
            return (blows, minute, Ended::BandGaveWay);
        }
        if lines[Side::Defence as usize].is_empty() {
            return (blows, minute, Ended::DefenceGaveWay);
        }
        for line in &mut lines {
            line.sort_by(|&a, &b| {
                fighters[b]
                    .will
                    .total_cmp(&fighters[a].will)
                    .then(fighters[a].person.cmp(&fighters[b].person))
            });
        }
        let pairs = (frontage.max(1) as usize)
            .min(lines[0].len())
            .min(lines[1].len());
        let at = start + i64::from(minute);
        let mut landed: Vec<Harm> = Vec::new();
        for (&a, &d) in lines[0].iter().zip(&lines[1]).take(pairs) {
            for (by, to) in [(a, d), (d, a)] {
                if let Some(h) = strike(seed, &fighters[by], &fighters[to], at, params) {
                    landed.push(h);
                }
            }
        }
        // Every blow of the minute falls together: both struck from the state before it.
        for h in &landed {
            if let Some(f) = fighters.iter_mut().find(|f| f.person == h.to) {
                f.hurt_days = f.hurt_days.saturating_add(h.days);
                f.killed |= h.killed;
            }
            blows.push(Struck { minute, harm: *h });
        }
        // Each still standing stays or gives way.
        let since = minute.saturating_sub(params.seen_minutes.saturating_sub(1));
        let mut seen = [0usize; 2];
        for b in blows.iter().filter(|b| b.minute >= since) {
            if let Some(f) = fighters.iter().find(|f| f.person == b.harm.to) {
                seen[f.side as usize] += 1;
            }
        }
        let mut gave = [0.0f64; 2];
        for s in Side::ALL {
            let all = fighters.iter().filter(|f| f.side == s).count().max(1);
            let out = fighters
                .iter()
                .filter(|f| f.side == s && !standing(f))
                .count();
            gave[s as usize] = out as f64 / all as f64;
        }
        let leader_stands: [bool; 2] = [0, 1].map(|s| {
            leads[s].is_some_and(|l| fighters.iter().any(|f| f.person == l && standing(f)))
        });
        for f in fighters.iter_mut() {
            if !standing(f) {
                continue;
            }
            let s = f.side as usize;
            let mut stay = f64::from(f.will)
                - params.will_seen * seen[s] as f64
                - params.will_gave * gave[s]
                - params.will_hurt * f64::from(f.hurt_days);
            if leader_stands[s] && leads[s] != Some(f.person) {
                stay += params.will_leader * f64::from(f.regard_leader);
            }
            if stay < 0.0 {
                f.gave_way = Some(minute);
            }
        }
    }
    let band = fighters.iter().any(|f| f.side == Side::Band && standing(f));
    let defence = fighters
        .iter()
        .any(|f| f.side == Side::Defence && standing(f));
    let ended = match (band, defence) {
        (true, true) => Ended::Parted,
        (true, false) => Ended::DefenceGaveWay,
        _ => Ended::BandGaveWay,
    };
    (blows, params.max_minutes, ended)
}

/// Whether `by` lands a blow on `to` in minute `at`, and how hard it falls.
fn strike(seed: u64, by: &Fighter, to: &Fighter, at: i64, params: &FightParams) -> Option<Harm> {
    let key = |purpose: u64| [seed, purpose, by.person.get(), to.person.get(), at as u64];
    if Rng64::from_key(&key(PURPOSE_LAND)).next_f64() >= params.land(by.holds) {
        return None;
    }
    let [lo, hi] = params.hurt_days;
    let span = f64::from(hi.saturating_sub(lo));
    let u = Rng64::from_key(&key(PURPOSE_FALL)).next_f64();
    let days = lo + (u * (span + 1.0)).floor().min(span) as u16;
    let killed = Rng64::from_key(&key(PURPOSE_SLAY)).next_f64() < params.kill_share;
    Some(Harm {
        by: by.person,
        to: to.person,
        days,
        killed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn side(first: u64, n: u64, side: Side, holds: Holding, will: f32) -> Vec<Fighter> {
        (first..first + n)
            .map(|k| Fighter {
                person: id(k),
                side,
                holds,
                will,
                regard_leader: 0.5,
                gave_way: None,
                hurt_days: 0,
                killed: false,
            })
            .collect()
    }

    #[test]
    fn nobody_standing_against_a_band_is_no_fight() {
        let mut f = side(1, 5, Side::Band, Holding::Spear, 3.0);
        let (blows, minutes, ended) = fight(&mut f, 3, [None; 2], 0, 7, &FightParams::core());
        assert!(blows.is_empty());
        assert_eq!(minutes, 0);
        assert_eq!(ended, Ended::NobodyStood);
        assert!(ended.band_holds());
    }

    #[test]
    fn people_behind_the_frontage_add_no_blows() {
        // 13-07 §4: people added behind a gate add no blows at once. In the first minute, before
        // anyone gives way, two blows fall per pair however many stand behind.
        let p = FightParams {
            land_spear: 1.0,
            kill_share: 0.0,
            ..FightParams::core()
        };
        for (n, k) in [(2u64, 1u32), (40, 1), (40, 2)] {
            let mut f = side(1, n, Side::Band, Holding::Spear, 50.0);
            f.extend(side(1000, n, Side::Defence, Holding::Spear, 50.0));
            let (blows, _, _) = fight(&mut f, k, [None; 2], 0, 11, &p);
            let first = blows.iter().filter(|b| b.minute == 0).count();
            assert_eq!(first, 2 * k as usize, "{n} a side, frontage {k}");
        }
    }

    #[test]
    fn both_strike_from_the_state_before_the_minute() {
        // Two who are sure to land and to kill both die in the first minute.
        let p = FightParams {
            land_hands: 1.0,
            kill_share: 1.0,
            ..FightParams::core()
        };
        let mut f = side(1, 1, Side::Band, Holding::Hands, 10.0);
        f.extend(side(2, 1, Side::Defence, Holding::Hands, 10.0));
        let (blows, _, ended) = fight(&mut f, 1, [None; 2], 0, 3, &p);
        assert_eq!(blows.len(), 2);
        assert!(f.iter().all(|x| x.killed));
        assert_eq!(ended, Ended::BandGaveWay, "nobody of the band stands");
    }

    #[test]
    fn those_with_little_will_give_way_when_their_side_is_struck_and_the_fight_ends() {
        let mut f = side(1, 6, Side::Band, Holding::Spear, 4.0);
        f.extend(side(100, 3, Side::Defence, Holding::Hands, 0.6));
        let (_, minutes, ended) = fight(&mut f, 3, [Some(id(1)), None], 0, 5, &FightParams::core());
        assert!(
            minutes < 60,
            "it ended before the longest: {minutes} minutes"
        );
        assert_eq!(ended, Ended::DefenceGaveWay);
        assert!(
            f.iter()
                .filter(|x| x.side == Side::Defence)
                .all(|x| x.gave_way.is_some() || x.killed)
        );
    }

    #[test]
    fn small_encounters_are_lumpy_and_mostly_bloodless() {
        // 13-07 §4: most village encounters end with few or no deaths.
        let mut deathless = 0;
        let runs = 200u64;
        for seed in 0..runs {
            let mut f = side(1, 8, Side::Band, Holding::Spear, 2.0);
            f.extend(side(100, 4, Side::Defence, Holding::Axe, 1.5));
            fight(
                &mut f,
                3,
                [Some(id(1)), Some(id(100))],
                0,
                seed,
                &FightParams::core(),
            );
            if !f.iter().any(|x| x.killed) {
                deathless += 1;
            }
        }
        assert!(
            deathless * 10 >= runs * 8,
            "{deathless} of {runs} ended with nobody dead"
        );
    }

    #[test]
    fn only_those_of_band_age_may_go_and_so_lead_a_band_a_law_sends() {
        // The M6c demo's probe: every law to send a band that lapsed the midnight it passed named
        // a proposer of 67 to 69 as its leader; the move is now offered by the same rule.
        let bp = BandParams::core();
        assert!(bp.can_go(bp.min_age) && bp.can_go(bp.max_age));
        assert!(bp.can_go(34.0));
        assert!(!bp.can_go(bp.min_age - 0.5));
        assert!(!bp.can_go(68.0) && bp.max_age < 68.0);
    }
}
