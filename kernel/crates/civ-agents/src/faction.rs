//! Factions (M4c slice AH, step one; ADR-0017 §2; research 04-10 §1.1–§1.5, §5.1–§5.4, 09-12
//! §1.5, §2.2).
//!
//! A faction is an organization of those who hold grievances against the same party, the
//! gathering or an office: it has an organizer, members, a store of its own on the ledger and a
//! history. It forms from a grievance someone holds keenly and has heard that people they trust
//! hold too (04-10 §1.1, §1.5: never because enough people are angry); joining and leaving are
//! each person's choice, reviewed on a day of their own each month (09-12 §2.2), weighing their
//! grievance, their regard for its organizer, how many of those they know belong (04-10 §5.2: an
//! absolute count beside the share) and its dues, against a threshold of their own (04-10 §1.3).
//! Membership is kept apart from belief: a member's grievance may fade while they stay, and one
//! who holds it may never join (04-10 §5.1). Members' households give a share of each threshing
//! to its store, and a member's household short of food may ask it (04-10 §1.2, §1.5: mutual
//! assistance). Nothing here reads a world total or another person's record.

use civ_core::{PermanentId, SimTime};

use crate::person::Flows;
use crate::word::Blamed;

/// How factions are founded, joined and kept (the people profile's `[faction]` table, content
/// API 45). Every number is a design prior.
#[derive(Clone, Debug, PartialEq)]
pub struct FactionParams {
    /// Days between one person's reviews of where they belong; each reviews on a day of their own
    /// (research 09-12 §2.2: one to three months).
    pub review_days: u32,
    /// How keenly a grievance must be felt, 0 to 1, before one would gather others over it.
    pub found_floor: f64,
    /// Regard for someone, 0 to 1, before their grievance counts as one shared with someone they
    /// trust.
    pub trust_floor: f64,
    /// How many others they trust they must have heard hold one against the same party for it to
    /// count as fully shared.
    pub shared_full: f64,
    /// Points for a grievance felt fully, a grievance fully shared, full regard for the
    /// organizer and everyone they know belonging; and for dues of a whole threshing.
    pub w_grievance: f64,
    pub w_shared: f64,
    pub w_organizer: f64,
    pub w_belong: f64,
    pub w_dues: f64,
    /// What gathering others costs one who founds, points: the time it takes and the risk they
    /// believe it runs.
    pub found_cost: f64,
    /// Days after a faction they founded ended before they would found another (research 04-10
    /// §3: a failed attempt lowers what one expects of the next).
    pub retry_days: u32,
    /// Thresholds are drawn evenly between these, points (research 04-10 §1.3: their spread
    /// matters more than their mean).
    pub threshold: [f64; 2],
    /// How far below their threshold the worth of belonging must fall before a member leaves
    /// (04-10 §5.4: leaving has conditions of its own).
    pub leave_margin: f64,
    /// The share of each threshing a member's household gives the faction's store, while the
    /// store holds less than `reserve_days` of its members' households' food.
    pub dues_share: f64,
    pub reserve_days: f64,
    /// Days of a household's food one ask of the faction's store may bring.
    pub aid_days: f64,
    /// Petitions (M4c slice AH, step two): an organizer weighs calling one when the faction has
    /// at least `petition_members` members and none of its petitions is waiting, and not within
    /// `petition_days` of its last; calling costs `petition_cost` points.
    pub petition_members: u32,
    pub petition_days: u32,
    pub petition_cost: f64,
    /// Attending one (research 04-10 §5.3): a member's identification with it, points; the
    /// share of those one knows whom one expects to come, at full share, points; and the share of
    /// people, by a keyed draw, for whom more others coming makes their own coming matter less
    /// (04-10 §1.4: free-riding).
    pub w_member: f64,
    pub w_expect: f64,
    pub free_ride_share: f64,
    /// Days of a household's food a petition turned down is felt as, for those who came.
    pub refused_days: f64,
    /// Refusing a levy together (M4c slice AH, step three): calling on its members to keep back
    /// the levy costs an organizer `refusal_cost` points (as calling a petition does, by default:
    /// what sets them apart is the norm a refusal breaks) beside what the norms they hold weigh
    /// for abiding by what the gathering decided, and the call stands `refusal_days`.
    pub refusal_cost: f64,
    pub refusal_days: u32,
}

impl FactionParams {
    /// The core pack's values, for tests.
    pub fn core() -> FactionParams {
        FactionParams {
            review_days: 30,
            found_floor: 0.3,
            trust_floor: 0.25,
            shared_full: 3.0,
            w_grievance: 1.0,
            w_shared: 1.0,
            w_organizer: 0.5,
            w_belong: 1.0,
            w_dues: 5.0,
            found_cost: 0.5,
            retry_days: 365,
            threshold: [0.5, 2.0],
            leave_margin: 0.5,
            dues_share: 0.05,
            reserve_days: 20.0,
            aid_days: 5.0,
            petition_members: 3,
            petition_days: 180,
            petition_cost: 0.5,
            w_member: 1.0,
            w_expect: 1.0,
            free_ride_share: 0.2,
            refused_days: 5.0,
            refusal_cost: 0.5,
            refusal_days: 365,
        }
    }

    /// The threshold `u` (a keyed draw in 0-1) gives.
    pub fn threshold_at(&self, u: f64) -> f64 {
        let [lo, hi] = self.threshold;
        lo + (hi - lo).max(0.0) * u.clamp(0.0, 1.0)
    }
}

/// What happened to a faction. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactionEventKind {
    /// It was founded by the one named.
    Founded = 0,
    /// The one named took up its organizing, its organizer having died, gone or left it.
    Organizer = 1,
    /// Its last member left it, and it is no more.
    Ended = 2,
}

impl FactionEventKind {
    /// Every kind, in code order.
    pub const ALL: [FactionEventKind; 3] = [
        FactionEventKind::Founded,
        FactionEventKind::Organizer,
        FactionEventKind::Ended,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The kind numbered `code`.
    pub fn from_code(code: u8) -> Option<FactionEventKind> {
        FactionEventKind::ALL.get(usize::from(code)).copied()
    }
}

/// An event in a faction's history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FactionEvent {
    /// When it happened.
    pub at: SimTime,
    /// What happened.
    pub kind: FactionEventKind,
    /// Who it names, if anyone.
    pub who: Option<PermanentId>,
}

/// A faction (ADR-0017 §2): an organization, apart from what its members believe.
#[derive(Clone, Debug, PartialEq)]
pub struct Faction {
    /// Permanent id; it is also its store's holder id on the ledger.
    pub id: PermanentId,
    /// The settlement it gathers in.
    pub settlement: PermanentId,
    /// The party its founder held a grievance against.
    pub against: Blamed,
    /// Who founded it.
    pub founder: PermanentId,
    /// Who organizes it now.
    pub organizer: PermanentId,
    /// When it was founded.
    pub founded: SimTime,
    /// When its last member left, if it has ended.
    pub ended: Option<SimTime>,
    /// Its store, by good, kept at its organizer's home.
    pub stores: Vec<f64>,
    /// When its store was last brought up to date.
    pub stores_at: SimTime,
    /// What came into and went out of its store.
    pub flows: Flows,
    /// Its history, oldest first.
    pub history: Vec<FactionEvent>,
}

impl Faction {
    /// A faction founded by `founder` at `now` in `settlement` against `against`.
    pub fn found(
        id: PermanentId,
        settlement: PermanentId,
        against: Blamed,
        founder: PermanentId,
        now: SimTime,
    ) -> Faction {
        Faction {
            id,
            settlement,
            against,
            founder,
            organizer: founder,
            founded: now,
            ended: None,
            stores: Vec::new(),
            stores_at: now,
            flows: Flows::default(),
            history: vec![FactionEvent {
                at: now,
                kind: FactionEventKind::Founded,
                who: Some(founder),
            }],
        }
    }

    /// Whether it has members still.
    pub fn is_live(&self) -> bool {
        self.ended.is_none()
    }

    /// Brings its store up to `t`: food spoils as it would under its organizer's roof
    /// (`sheltered`) or in the open.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[crate::params::GoodDef], sheltered: bool) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            let half_life = if sheltered && d.sheltered_half_life_days > 0.0 {
                d.sheltered_half_life_days
            } else {
                d.half_life_days
            };
            if half_life > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / half_life);
                self.flows.add(crate::person::Flow::Spoiled, g, *kg - after);
                *kg = after;
            }
        }
        self.stores_at = t;
    }
}

/// What weighed in a person's choice to join or stay, as they last reviewed it: the record of
/// why (ADR-0017 §6), in points but for the count.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Why {
    /// Their grievance against its party, as keenly as they felt it, weighed.
    pub grievance: f32,
    /// Their regard for its organizer, weighed.
    pub organizer: f32,
    /// The share of those they know who belong, weighed.
    pub belong: f32,
    /// How many of those they know belong.
    pub known: u16,
    /// What its dues cost them, weighed (positive).
    pub dues: f32,
    /// Their threshold.
    pub threshold: f32,
}

impl Why {
    /// What belonging was worth to them, points, before their threshold.
    pub fn worth(&self) -> f64 {
        f64::from(self.grievance) + f64::from(self.organizer) + f64::from(self.belong)
            - f64::from(self.dues)
    }
}

/// One person's membership of a faction (ADR-0017 §2: kept on persons).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Member {
    /// Who belongs.
    pub person: PermanentId,
    /// The faction.
    pub faction: PermanentId,
    /// The day they joined.
    pub since: i64,
    /// Why, as they last reviewed it.
    pub why: Why,
}

/// A petition (M4c slice AH, step two; ADR-0017 §3; research 09-05 §1.2: petitioning is a right
/// apart from proposing): an organizer calls the faction to the hearth on an evening, and what it
/// asks then goes before the gathering, which decides it as any law. Its demand answers the party
/// its members blame: against the gathering, a common store at another share, or at none, in
/// place of the one in force; against an office, another holder in place of the one in it.
#[derive(Clone, Debug, PartialEq)]
pub struct Petition {
    /// Permanent id.
    pub id: PermanentId,
    /// The faction that called it, and its settlement.
    pub faction: PermanentId,
    pub settlement: PermanentId,
    /// Who called it.
    pub organizer: PermanentId,
    /// When it was called, and the day it sits at the hearth, in the evening.
    pub called: SimTime,
    pub day: i64,
    /// The demand: a law of template `policy` in place of law `ends`, taking `levy_share` (none
    /// at 0) for a store, or naming `nominee` for an office.
    pub policy: u16,
    pub levy_share: f32,
    pub nominee: Option<PermanentId>,
    pub ends: PermanentId,
    /// Those who came, in id order (04-10 §4.2: unique participants).
    pub came: Vec<PermanentId>,
    /// The proposal made of it, once it was put to the gathering.
    pub law: Option<PermanentId>,
    /// Whether its answer has been taken (those who came hold one turned down against the
    /// gathering), or it ended unattended.
    pub answered: bool,
}

/// A refusal of a levy (M4c slice AH, step three; ADR-0017 §3): an organizer calls on the
/// faction to keep back the levy of a store's law in force together until a day, openly, and
/// each who heard of it chooses at their threshing whether to join it (research 04-10 §1.9: a
/// withdrawal works through what it withholds).
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    /// Permanent id.
    pub id: PermanentId,
    /// The faction that called it, and its settlement.
    pub faction: PermanentId,
    pub settlement: PermanentId,
    /// Who called it, when, and the last day it stands.
    pub organizer: PermanentId,
    pub called: SimTime,
    pub until: i64,
    /// The law whose levy it keeps back.
    pub law: PermanentId,
    /// Those who kept back their household's levy under it, in id order, and what they kept,
    /// kilograms.
    pub kept: Vec<PermanentId>,
    pub kept_kg: f64,
}

/// Every faction and who belongs to each (ADR-0017 §2).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Factions {
    /// Every faction founded, oldest first; those that ended are kept for their history.
    pub list: Vec<Faction>,
    /// Every petition called, oldest first (M4c slice AH, step two).
    pub petitions: Vec<Petition>,
    /// Every refusal of a levy called, oldest first (M4c slice AH, step three).
    pub refusals: Vec<Refusal>,
    /// Memberships, in person order: one each at most in v0.
    pub members: Vec<Member>,
    /// Joinings and leavings since the world began (a measure, for the smoke).
    pub joined: u64,
    pub left: u64,
}

impl Factions {
    /// The faction `id`.
    pub fn get(&self, id: PermanentId) -> Option<&Faction> {
        self.list.iter().find(|f| f.id == id)
    }

    /// The same, to change.
    pub fn get_mut(&mut self, id: PermanentId) -> Option<&mut Faction> {
        self.list.iter_mut().find(|f| f.id == id)
    }

    /// `person`'s membership, if they belong to one.
    pub fn membership(&self, person: PermanentId) -> Option<&Member> {
        self.members
            .binary_search_by_key(&person, |m| m.person)
            .ok()
            .map(|i| &self.members[i])
    }

    /// `m` joins, in its place (replacing any membership of theirs).
    pub fn join(&mut self, m: Member) {
        match self.members.binary_search_by_key(&m.person, |x| x.person) {
            Ok(i) => self.members[i] = m,
            Err(i) => self.members.insert(i, m),
        }
    }

    /// `person` leaves whatever they belong to; returns the membership they held.
    pub fn leave(&mut self, person: PermanentId) -> Option<Member> {
        let i = self
            .members
            .binary_search_by_key(&person, |m| m.person)
            .ok()?;
        Some(self.members.remove(i))
    }

    /// Those who belong to faction `id`, in person order.
    pub fn members_of(&self, id: PermanentId) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(move |m| m.faction == id)
    }
}

/// How far a grievance is shared, 0 to 1: `heard` others one trusts holding one against the
/// same party, of the `full` that would make it fully shared.
pub fn shared(heard: usize, full: f64) -> f64 {
    if full <= 0.0 {
        return 1.0;
    }
    (heard as f64 / full).clamp(0.0, 1.0)
}

/// Why someone would belong to a faction, from what they hold and know: `grievance` how keenly
/// they feel theirs against its party (0-1), `regard` their regard for its organizer, and
/// `belong` the share of those they know who belong, of whom `known` they know.
pub fn why_belong(
    grievance: f64,
    regard: f64,
    belong: f64,
    known: usize,
    threshold: f64,
    p: &FactionParams,
) -> Why {
    Why {
        grievance: (p.w_grievance * grievance.clamp(0.0, 1.0)) as f32,
        organizer: (p.w_organizer * regard.clamp(0.0, 1.0)) as f32,
        belong: (p.w_belong * belong.clamp(0.0, 1.0)) as f32,
        known: known.min(usize::from(u16::MAX)) as u16,
        dues: (p.w_dues * p.dues_share) as f32,
        threshold: threshold as f32,
    }
}

/// What coming to a petition is worth to someone (research 04-10 §5.3), points: `grievance` how
/// keenly they feel theirs against the party it petitions (0-1); as a member, their
/// identification with it, or else their regard for its organizer; and the share of those they
/// know whom they expect to come (`expect`, 0-1), which draws most people but matters less to a
/// free-rider the more come (04-10 §1.4). What else they could do, and its walk, weigh against it
/// in their choice.
pub fn attend_points(
    grievance: f64,
    member: bool,
    regard: f64,
    expect: f64,
    free_rider: bool,
    p: &FactionParams,
) -> f64 {
    let s = expect.clamp(0.0, 1.0);
    let social = if free_rider { 4.0 * s * (1.0 - s) } else { s };
    let identity = if member {
        p.w_member
    } else {
        p.w_organizer * regard.clamp(0.0, 1.0)
    };
    p.w_grievance * grievance.clamp(0.0, 1.0) + identity + p.w_expect * social
}

/// What founding a faction is worth to one who feels their grievance at `grievance` (0-1) and
/// believes it `shared` (0-1), points, before their threshold.
pub fn found_worth(grievance: f64, shared: f64, p: &FactionParams) -> f64 {
    p.w_grievance * grievance.clamp(0.0, 1.0) + p.w_shared * shared.clamp(0.0, 1.0) - p.found_cost
}

/// Why someone belongs, in words to follow "they" (ADR-0017 §6): "belong as its organizer,
/// holding a grievance against the gathering, and 3 of those they know belong: worth 1.62 to them
/// against a threshold of 1.10, its dues counted". `party` names what it stands against,
/// `organizer` its organizer, and `organizes` says whether they are that organizer.
pub fn why_words(why: &Why, party: &str, organizer: &str, organizes: bool) -> String {
    let mut reasons: Vec<String> = Vec::new();
    if why.grievance >= 0.05 {
        reasons.push(format!("holding a grievance against {party}"));
    }
    if !organizes && why.organizer >= 0.05 {
        reasons.push(format!("regarding {organizer}"));
    }
    match why.known {
        0 => {}
        1 => reasons.push("1 of those they know belongs".to_owned()),
        n => reasons.push(format!("{n} of those they know belong")),
    }
    let lead = if organizes {
        "belong as its organizer"
    } else {
        "belong"
    };
    let because = match reasons.len() {
        0 => String::new(),
        1 => format!(", {}", reasons[0]),
        n => format!(", {} and {}", reasons[..n - 1].join(", "), reasons[n - 1]),
    };
    format!(
        "{lead}{because}: worth {:.2} to them against a threshold of {:.2}, its dues counted",
        why.worth(),
        why.threshold
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn a_grievance_shared_with_those_one_trusts_is_worth_organizing_and_one_held_alone_is_not() {
        let p = FactionParams::core();
        let lone = found_worth(0.8, shared(0, p.shared_full), &p);
        let shared_by_three = found_worth(0.8, shared(3, p.shared_full), &p);
        let threshold = p.threshold_at(0.5);
        assert!(lone < threshold, "{lone}");
        assert!(shared_by_three > threshold, "{shared_by_three}");
        // Hearing of more than enough adds nothing.
        assert_eq!(shared(9, p.shared_full), 1.0);
        assert!(p.threshold_at(0.0) >= p.threshold[0] && p.threshold_at(1.0) <= p.threshold[1]);
    }

    #[test]
    fn belonging_weighs_the_grievance_the_organizer_and_those_one_knows_against_the_dues() {
        let p = FactionParams::core();
        let keen = why_belong(1.0, 0.8, 0.5, 4, 1.0, &p);
        let cool = why_belong(0.0, 0.0, 0.0, 0, 1.0, &p);
        assert!(keen.worth() > f64::from(keen.threshold));
        assert!(cool.worth() < 0.0, "dues alone cost");
        assert_eq!(keen.known, 4);
        assert!((f64::from(keen.dues) - p.w_dues * p.dues_share).abs() < 1e-6);
    }

    #[test]
    fn why_someone_belongs_reads_as_a_sentence() {
        let p = FactionParams::core();
        let why = why_belong(0.8, 0.6, 0.5, 3, 1.1, &p);
        assert_eq!(
            why_words(&why, "the gathering", "Mira", false),
            "belong, holding a grievance against the gathering, regarding Mira and 3 of those \
             they know belong: worth 1.35 to them against a threshold of 1.10, its dues counted"
        );
        let organizer = why_belong(0.8, 1.0, 0.0, 0, 1.1, &p);
        assert!(
            why_words(&organizer, "the gathering", "Mira", true)
                .starts_with("belong as its organizer, holding a grievance against the gathering:")
        );
    }

    #[test]
    fn coming_to_a_petition_weighs_grievance_identity_and_who_else_comes() {
        let p = FactionParams::core();
        let member = attend_points(0.6, true, 0.0, 0.5, false, &p);
        let stranger = attend_points(0.6, false, 0.0, 0.5, false, &p);
        assert!(member > stranger);
        // Most are drawn by more coming; a free-rider less so once nearly everyone comes.
        let most = |s| attend_points(0.0, false, 0.0, s, false, &p);
        let rider = |s| attend_points(0.0, false, 0.0, s, true, &p);
        assert!(most(1.0) > most(0.5));
        assert!(rider(1.0) < rider(0.5), "{} {}", rider(1.0), rider(0.5));
    }

    #[test]
    fn memberships_are_kept_in_person_order_one_each() {
        let mut fs = Factions::default();
        let m = |person, faction| Member {
            person: pid(person),
            faction: pid(faction),
            since: 0,
            why: Why::default(),
        };
        fs.join(m(5, 100));
        fs.join(m(2, 100));
        fs.join(m(9, 101));
        fs.join(m(5, 101));
        let order: Vec<u64> = fs.members.iter().map(|m| m.person.get()).collect();
        assert_eq!(order, [2, 5, 9]);
        assert_eq!(fs.membership(pid(5)).map(|m| m.faction), Some(pid(101)));
        assert_eq!(fs.members_of(pid(101)).count(), 2);
        assert_eq!(fs.leave(pid(2)).map(|m| m.faction), Some(pid(100)));
        assert!(fs.membership(pid(2)).is_none());
        assert!(fs.leave(pid(2)).is_none());
    }
}
