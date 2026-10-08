//! Word of mouth and grievances (M4c slice AE, ADR-0016 §2–§3).
//!
//! **Claims** are shared, written once and never changed: that a gathering is called for a day, or
//! that someone holds a grievance against a party. **Hearing records** are per person: when they
//! first and last heard a claim, from whom, and the one it began with. A claim travels only by
//! contact (the household at midnight, companions at the hearth); absence of a record is not
//! having heard. **Grievances** are per person: a harm, the party blamed, the law whose terms it
//! broke, what is still unresolved, and an activation that fades and is raised only by reminders.
//! Nothing here reads a world total or another person's record.

use civ_core::PermanentId;

/// What a claim says. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimKind {
    /// A gathering is called at the settlement's hearth for a day.
    Gathering = 0,
    /// Someone holds a grievance against a party.
    Grievance = 1,
    /// A faction petitions the gathering at the settlement's hearth on a day (M4c slice AH).
    Petition = 2,
}

impl ClaimKind {
    /// Every kind, in code order.
    pub const ALL: [ClaimKind; 3] = [
        ClaimKind::Gathering,
        ClaimKind::Grievance,
        ClaimKind::Petition,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The kind numbered `code`.
    pub fn from_code(code: u8) -> Option<ClaimKind> {
        ClaimKind::ALL.get(usize::from(code)).copied()
    }
}

/// A shared claim (ADR-0016 §3): written once and never changed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Claim {
    /// Its number, from 1, in the order made.
    pub id: u32,
    /// What it says.
    pub kind: ClaimKind,
    /// The settlement it concerns.
    pub settlement: PermanentId,
    /// The day it concerns: the day a gathering meets; the day a grievance was first told.
    pub day: i64,
    /// For a gathering called on a law, the law; for a grievance, the one who holds it; for a
    /// petition, the petition.
    pub subject: Option<PermanentId>,
    /// For a grievance: the party blamed and the issue.
    pub grievance: Option<(Blamed, Grieved)>,
}

/// That `holder` heard claim `claim` (ADR-0016 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heard {
    /// Who heard it.
    pub holder: PermanentId,
    /// The claim.
    pub claim: u32,
    /// The day they first heard it.
    pub first: i64,
    /// The day they last heard it.
    pub last: i64,
    /// Who told them, if anyone (none for those who made it or saw it made).
    pub from: Option<PermanentId>,
    /// The one the account began with: several retellings of one origin are one source.
    pub origin: Option<PermanentId>,
}

/// What people have heard (ADR-0016 §3).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Word {
    /// The claims, by number.
    pub claims: Vec<Claim>,
    /// Who heard what, in (holder, claim) order.
    pub heard: Vec<Heard>,
    /// What each holds against whom, in (holder, made) order.
    pub grievances: Vec<Grievance>,
    /// The next claim's number.
    pub next: u32,
}

impl Word {
    /// Claim number `id`.
    pub fn claim(&self, id: u32) -> Option<&Claim> {
        self.claims
            .binary_search_by_key(&id, |c| c.id)
            .ok()
            .map(|i| &self.claims[i])
    }

    /// Makes `claim` (its number is set here) and returns its number.
    pub fn make(&mut self, mut claim: Claim) -> u32 {
        self.next = self.next.max(self.claims.last().map_or(0, |c| c.id)) + 1;
        claim.id = self.next;
        self.claims.push(claim);
        claim.id
    }

    /// The claim that a gathering meets at `settlement` on `day`, if one was made.
    pub fn gathering(&self, settlement: PermanentId, day: i64) -> Option<u32> {
        self.claims
            .iter()
            .rev()
            .find(|c| c.kind == ClaimKind::Gathering && c.settlement == settlement && c.day == day)
            .map(|c| c.id)
    }

    /// The claim that a petition sits at `settlement` on `day`, if one was made.
    pub fn petition(&self, settlement: PermanentId, day: i64) -> Option<u32> {
        self.claims
            .iter()
            .rev()
            .find(|c| c.kind == ClaimKind::Petition && c.settlement == settlement && c.day == day)
            .map(|c| c.id)
    }

    /// Whether `holder` has heard claim `claim`.
    pub fn has_heard(&self, holder: PermanentId, claim: u32) -> bool {
        self.heard
            .binary_search_by(|h| (h.holder, h.claim).cmp(&(holder, claim)))
            .is_ok()
    }

    /// `holder` hears claim `claim` on `day`, told by `from`, the account beginning with `origin`.
    /// Returns whether they had not heard it before.
    pub fn hear(
        &mut self,
        holder: PermanentId,
        claim: u32,
        day: i64,
        from: Option<PermanentId>,
        origin: Option<PermanentId>,
    ) -> bool {
        match self
            .heard
            .binary_search_by(|h| (h.holder, h.claim).cmp(&(holder, claim)))
        {
            Ok(i) => {
                self.heard[i].last = self.heard[i].last.max(day);
                false
            }
            Err(i) => {
                self.heard.insert(
                    i,
                    Heard {
                        holder,
                        claim,
                        first: day,
                        last: day,
                        from,
                        origin,
                    },
                );
                true
            }
        }
    }

    /// What `holder` has heard, in claim order.
    pub fn heard_by(&self, holder: PermanentId) -> &[Heard] {
        let from = self.heard.partition_point(|h| h.holder < holder);
        let to = self.heard.partition_point(|h| h.holder <= holder);
        &self.heard[from..to]
    }

    /// Lets go of what is no longer news on `today`: a gathering's call once its day has passed,
    /// and a grievance told more than `news_days` ago and not since; and of claims nobody holds.
    pub fn prune(&mut self, today: i64, news_days: i64) {
        let stale = |c: &Claim, last: i64| match c.kind {
            ClaimKind::Gathering | ClaimKind::Petition => c.day < today,
            ClaimKind::Grievance => last + news_days < today,
        };
        let claims = &self.claims;
        let find = |id: u32| {
            claims
                .binary_search_by_key(&id, |c| c.id)
                .ok()
                .map(|i| &claims[i])
        };
        self.heard
            .retain(|h| find(h.claim).is_some_and(|c| !stale(c, h.last)));
        self.drop_unheld_claims();
    }

    /// Lets go of what those no longer here held (`here` is false for them): their grievances
    /// and what they had heard go with them, and so do claims nobody holds now. A claim about
    /// them that others heard stays news as long as it would.
    pub fn let_go_of_gone(&mut self, here: impl Fn(PermanentId) -> bool) {
        self.grievances.retain(|g| here(g.holder));
        self.heard.retain(|h| here(h.holder));
        self.drop_unheld_claims();
    }

    fn drop_unheld_claims(&mut self) {
        let mut held: Vec<u32> = self.heard.iter().map(|h| h.claim).collect();
        held.sort_unstable();
        held.dedup();
        self.claims.retain(|c| held.binary_search(&c.id).is_ok());
    }

    /// The grievances `holder` holds, in the order made.
    pub fn grievances_of(&self, holder: PermanentId) -> impl Iterator<Item = &Grievance> {
        self.grievances.iter().filter(move |g| g.holder == holder)
    }
}

/// What a grievance is about (ADR-0016 §2). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grieved {
    /// Food when it was needed.
    Subsistence = 0,
    /// What was taken from the household by a levy or rent.
    Extraction = 1,
    /// How the household or its member was treated.
    Treatment = 2,
    /// A wrong left unanswered that the polity's law promised to answer.
    Collective = 3,
}

impl Grieved {
    /// Every issue, in code order.
    pub const ALL: [Grieved; 4] = [
        Grieved::Subsistence,
        Grieved::Extraction,
        Grieved::Treatment,
        Grieved::Collective,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The issue numbered `code`.
    pub fn from_code(code: u8) -> Option<Grieved> {
        Grieved::ALL.get(usize::from(code)).copied()
    }

    /// What it is over, in words, to follow "over": "food when their household was short".
    pub fn words(self) -> &'static str {
        match self {
            Grieved::Subsistence => "food when their household was short",
            Grieved::Extraction => "what was taken from their household",
            Grieved::Treatment => "how their household was treated",
            Grieved::Collective => "a wrong to their household left unanswered",
        }
    }
}

/// The party a grievance blames (ADR-0016 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Blamed {
    /// The polity's body, by the polity's id: the gathering.
    Body(PermanentId),
    /// An office, by the law that names its holder.
    Office(PermanentId),
    /// A household.
    Household(PermanentId),
    /// A person.
    Person(PermanentId),
}

impl Blamed {
    /// Its kind's number and its id, for saves.
    pub fn to_raw(self) -> (u8, u64) {
        match self {
            Blamed::Body(id) => (0, id.get()),
            Blamed::Office(id) => (1, id.get()),
            Blamed::Household(id) => (2, id.get()),
            Blamed::Person(id) => (3, id.get()),
        }
    }

    /// The party saved as `(kind, id)`.
    pub fn from_raw(kind: u8, id: u64) -> Option<Blamed> {
        let id = PermanentId::from_raw(id)?;
        match kind {
            0 => Some(Blamed::Body(id)),
            1 => Some(Blamed::Office(id)),
            2 => Some(Blamed::Household(id)),
            3 => Some(Blamed::Person(id)),
            _ => None,
        }
    }
}

/// The event that raised a grievance. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrong {
    /// The household was short of food and found the common store empty.
    StoreEmpty = 0,
    /// The gathering found against one of the household whom they do not believe took.
    FoundAgainst = 1,
    /// The gathering did not find against one they believe took from their household.
    NotFound = 2,
    /// Too few came to hear the household's case.
    Unheard = 3,
    /// What a finding imposed for the household was refused or left unpaid.
    Unpaid = 4,
    /// A levy was taken in a lean year that left the household short of a year's food.
    LeanLevy = 5,
    /// The gathering turned down, or was too thin to decide, what they came to petition for
    /// (M4c slice AH).
    Refused = 6,
}

impl Wrong {
    /// Every event, in code order.
    pub const ALL: [Wrong; 7] = [
        Wrong::StoreEmpty,
        Wrong::FoundAgainst,
        Wrong::NotFound,
        Wrong::Unheard,
        Wrong::Unpaid,
        Wrong::LeanLevy,
        Wrong::Refused,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The event numbered `code`.
    pub fn from_code(code: u8) -> Option<Wrong> {
        Wrong::ALL.get(usize::from(code)).copied()
    }

    /// What happened, in words: "the common store was empty when their household was short".
    pub fn words(self) -> &'static str {
        match self {
            Wrong::StoreEmpty => "the common store was empty when their household was short",
            Wrong::FoundAgainst => {
                "the gathering found against one of their household, who they do not believe took"
            }
            Wrong::NotFound => {
                "the gathering did not find against the one they believe took from their household"
            }
            Wrong::Unheard => "too few came to hear their household's case",
            Wrong::Unpaid => "what a finding imposed for their household was refused",
            Wrong::LeanLevy => {
                "the levy was taken in a lean year though it left their household short of a \
                 year's food"
            }
            Wrong::Refused => "the gathering did not grant what they came to petition for",
        }
    }
}

/// A grievance (ADR-0016 §2): a durable claim with a fading activation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grievance {
    /// Who holds it.
    pub holder: PermanentId,
    /// What it is about.
    pub issue: Grieved,
    /// Whom it blames.
    pub blamed: Blamed,
    /// The law whose terms it broke: the expectation the holder held.
    pub law: PermanentId,
    /// The harm, days of the household's food, all told.
    pub harm_days: f32,
    /// What is still unresolved, days of food.
    pub unresolved_days: f32,
    /// How keenly it was felt when last raised, 0 to 1.
    pub activation: f32,
    /// The day it was last raised.
    pub raised: i64,
    /// The day it was first held.
    pub made: i64,
    /// What last raised it.
    pub wrong: Wrong,
}

impl Grievance {
    /// How keenly it is felt on `day`, its activation having faded by `half_life_days` since it
    /// was last raised.
    pub fn activation_on(&self, day: i64, half_life_days: f64) -> f64 {
        let since = (day - self.raised).max(0) as f64;
        let fade = if half_life_days > 0.0 {
            (-since / half_life_days).exp2()
        } else {
            1.0
        };
        f64::from(self.activation) * fade
    }
}

/// How word travels and grievances are held (the people profile's `[word]` table; content API
/// 39). Tuning values unless the content says otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct WordParams {
    /// The chance a member of a household tells each other member, at midnight, of a gathering
    /// they heard is called (research 09-16 §2.2: 0.4–0.9 for urgent news).
    pub share_home: f64,
    /// The chance someone tells a companion at the hearth of a gathering called that they heard
    /// of (09-16 §2.2: 0.4–0.9).
    pub share_urgent: f64,
    /// The chance someone tells a companion at the hearth of a grievance they hold (09-16 §2.2:
    /// 0.05–0.25 for routine news).
    pub share_routine: f64,
    /// Days a grievance told stays news without being told again (09-16 §2.2: 30–180 for
    /// personally important claims).
    pub news_days: u32,
    /// Grievances a person holds at most; the least keenly felt gives way.
    pub max_grievances: u32,
    /// The half-life of a grievance's activation, days, by issue in code order (04-06 §2.2: 1–7
    /// for minor events, 7–90 for severe ones).
    pub half_life_days: [f64; 4],
    /// Days of the household's food a harm must reach to be felt fully (a tuning value).
    pub full_harm_days: f64,
    /// What hearing one's own grievance told raises its activation by.
    pub reminder: f64,
    /// Activation below which a grievance is not told.
    pub tell_floor: f64,
    /// Days before a harm that goes on (a store still empty) raises its grievance again: it is a
    /// reminder, never re-added each day (04-06 §5.3).
    pub remind_days: u32,
}

impl WordParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> WordParams {
        WordParams {
            share_home: 0.8,
            share_urgent: 0.6,
            share_routine: 0.15,
            news_days: 60,
            max_grievances: 8,
            half_life_days: [30.0, 30.0, 90.0, 60.0],
            full_harm_days: 10.0,
            reminder: 0.1,
            tell_floor: 0.2,
            remind_days: 7,
        }
    }

    /// The half-life of `issue`'s activation, days.
    pub fn half_life(&self, issue: Grieved) -> f64 {
        self.half_life_days[issue as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("nonzero")
    }

    #[test]
    fn hearing_is_once_per_claim_and_retellings_keep_their_origin() {
        let mut w = Word::default();
        let c = w.make(Claim {
            id: 0,
            kind: ClaimKind::Gathering,
            settlement: id(1),
            day: 10,
            subject: None,
            grievance: None,
        });
        assert_eq!(w.gathering(id(1), 10), Some(c));
        assert_eq!(w.gathering(id(1), 11), None);
        assert!(w.hear(id(5), c, 8, None, Some(id(5))));
        assert!(w.hear(id(3), c, 9, Some(id(5)), Some(id(5))));
        // Hearing it again moves only when it was last heard.
        assert!(!w.hear(id(3), c, 9, Some(id(7)), Some(id(7))));
        let h = w.heard_by(id(3));
        assert_eq!(h.len(), 1);
        assert_eq!((h[0].from, h[0].origin), (Some(id(5)), Some(id(5))));
        assert!(w.has_heard(id(5), c) && !w.has_heard(id(4), c));
        // A gathering's call is news until its day has passed.
        w.prune(10, 60);
        assert!(w.has_heard(id(3), c));
        w.prune(11, 60);
        assert!(!w.has_heard(id(3), c) && w.claim(c).is_none());
    }

    #[test]
    fn what_the_dead_held_goes_with_them() {
        let mut w = Word::default();
        let c = w.make(Claim {
            id: 0,
            kind: ClaimKind::Gathering,
            settlement: id(1),
            day: 10,
            subject: None,
            grievance: None,
        });
        w.hear(id(5), c, 8, None, Some(id(5)));
        w.hear(id(3), c, 9, Some(id(5)), Some(id(5)));
        for holder in [id(3), id(5)] {
            w.grievances.push(Grievance {
                holder,
                issue: Grieved::Collective,
                blamed: Blamed::Body(id(9)),
                law: id(9),
                wrong: Wrong::StoreEmpty,
                harm_days: 3.0,
                unresolved_days: 3.0,
                activation: 1.0,
                raised: 8,
                made: 8,
            });
        }
        // 5 died: what they held and had heard goes; 3 keeps what they heard from them.
        w.let_go_of_gone(|p| p != id(5));
        assert!(w.grievances.iter().all(|g| g.holder == id(3)));
        assert!(!w.has_heard(id(5), c) && w.has_heard(id(3), c));
        w.let_go_of_gone(|_| false);
        assert!(w.grievances.is_empty() && w.claim(c).is_none());
    }

    #[test]
    fn a_grievance_fades_by_its_half_life() {
        let g = Grievance {
            holder: id(1),
            issue: Grieved::Collective,
            blamed: Blamed::Body(id(9)),
            law: id(7),
            harm_days: 6.0,
            unresolved_days: 6.0,
            activation: 0.8,
            raised: 100,
            made: 100,
            wrong: Wrong::NotFound,
        };
        assert!((g.activation_on(100, 60.0) - 0.8).abs() < 1e-6);
        assert!((g.activation_on(160, 60.0) - 0.4).abs() < 1e-6);
        assert!(
            (g.activation_on(50, 60.0) - 0.8).abs() < 1e-6,
            "never before raised"
        );
        assert_eq!(
            Blamed::from_raw(Blamed::Office(id(4)).to_raw().0, 4),
            Some(Blamed::Office(id(4)))
        );
    }
}
