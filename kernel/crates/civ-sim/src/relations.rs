//! How one polity stands toward another, worked out afterwards from what its people hold
//! (ADR-0020 §1): unknown, known, under agreement, friendly or wary, with the reasons why. No
//! relation is saved and nothing in the world reads a label: the people live in `civ-agents`,
//! which cannot see this crate. Each side's label is worked out from its own people and its own
//! laws alone, so the two can differ. A polity paying the other a recurring transfer under an
//! agreement in force is *tributary* to it, its burden given both ways research 13-02 §2.1 asks
//! (M5c slice AV, step two).
//!
//! The thresholds are display conventions, uncalibrated, not findings.

use civ_agents::polity::Polity;
use civ_agents::views::Domain;
use civ_agents::word::{Blamed, Wrong};
use civ_core::PermanentId;

/// A view leans one way in a domain when the mean of its evidence is past this: with the core
/// content, a single account heard (half a seen one) is not enough, two are.
pub const LEAN: f64 = 0.6;
/// The share of a polity's adults whose view leans toward "harms us" for it to be called wary.
pub const WARY: f64 = 0.1;
/// The share whose view leans toward "helps us" for it to be called friendly.
pub const FRIENDLY: f64 = 0.1;

/// What a label is worked out from: what one polity's people hold of another, as plain counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RelationEvidence {
    /// Its households.
    pub households: u32,
    /// Of them, those that know the other's settlement.
    pub knowing: u32,
    /// Its adults.
    pub adults: u32,
    /// Of them, those who hold a view of the other polity.
    pub views: u32,
    /// Of those, the views that lean toward "harms us", and toward "helps us".
    pub harms: u32,
    pub helps: u32,
    /// Of its adults, those who hold a grievance against a household of the other settlement for
    /// working a place their polity claims.
    pub aggrieved: u32,
    /// The places it claims now, and of them those the other claims too.
    pub claims: u32,
    pub contested: u32,
    /// The terms of each agreement with the other in force by a law of its own in force, in
    /// words (M5c slice AU).
    pub in_force: Vec<String>,
    /// Agreements with the other still before a gathering or awaiting word, and those that
    /// failed or ended.
    pub pending: u32,
    pub over: u32,
    /// What it pays the other a year by transfers in force, in words ("100 kg of grain"), and
    /// that as a share of what its households' fields bring in an ordinary year (T/Y) and of
    /// what they bring beyond its people's need (T/(Y − C)), in tenths of a percent; `None`
    /// where its fields bring nothing, or no more than its people need (research 13-02 §2.1:
    /// a subsistence shortfall). M5c slice AV, step two.
    pub pays: Option<String>,
    pub of_output: Option<u32>,
    pub of_surplus: Option<u32>,
    /// What the other pays it a year by transfers in force, in words.
    pub paid: Option<String>,
}

/// The name a label gives a relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Nobody of it knows of the other.
    Unknown,
    /// Known, and nobody's view leans far enough either way.
    Known,
    /// An agreement with the other is in force by a law of its own (M5c slice AU).
    UnderAgreement,
    /// It pays the other a recurring transfer under an agreement in force (M5c slice AV).
    Tributary,
    /// Enough of its adults believe the other helps them.
    Friendly,
    /// Enough of its adults believe the other harms them.
    Wary,
}

impl Standing {
    /// In words: "wary".
    pub fn words(self) -> &'static str {
        match self {
            Standing::Unknown => "unknown",
            Standing::Known => "known",
            Standing::UnderAgreement => "under agreement",
            Standing::Tributary => "tributary",
            Standing::Friendly => "friendly",
            Standing::Wary => "wary",
        }
    }
}

/// A relation's label: its name and the reasons, in sentences.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationLabel {
    pub standing: Standing,
    pub why: Vec<String>,
}

/// What the people of `from` hold of `to` now.
pub fn evidence(sim: &crate::Sim, from: &Polity, to: &Polity) -> RelationEvidence {
    let (pop, rules, now) = (sim.people(), sim.rules(), sim.now());
    let (day, adult) = (now.day_index(), rules.people.family.independent_age);
    let rp = &rules.people.relations;
    let mut e = RelationEvidence::default();
    for (_, x) in pop.households.iter() {
        if x.members.is_empty() || x.settlement != Some(from.settlement) {
            continue;
        }
        e.households += 1;
        if pop
            .known_places
            .of(x.id)
            .iter()
            .any(|k| k.settlement == to.settlement)
        {
            e.knowing += 1;
        }
        for &m in &x.members {
            if !pop.person(m).is_some_and(|p| p.age_years(now) >= adult) {
                continue;
            }
            e.adults += 1;
            if let Some(v) = pop.polity_views.of(m, to.id, day, rp) {
                e.views += 1;
                e.harms += u32::from(v.lean(Domain::HarmsUs) > LEAN);
                e.helps += u32::from(v.lean(Domain::HelpsUs) > LEAN);
            }
            let theirs = |b: Blamed| match b {
                Blamed::Household(h) => pop
                    .household(h)
                    .is_some_and(|x| x.settlement == Some(to.settlement)),
                _ => false,
            };
            e.aggrieved += u32::from(
                pop.word
                    .grievances_of(m)
                    .any(|g| g.wrong == Wrong::Trespass && theirs(g.blamed)),
            );
        }
    }
    // Agreements between the two (M5c slice AU): in force by its own law, or not yet, or over.
    let name = |p: PermanentId| {
        pop.polities
            .iter()
            .find(|x| x.id == p)
            .and_then(|x| sim.land().settlements.iter().find(|s| s.id == x.settlement))
            .map_or_else(|| "another settlement".to_owned(), |s| s.name.clone())
    };
    let good = |g: u16| good_name(sim, g);
    for a in &pop.agreements.list {
        let Some(side) = a.side_of(from.id) else {
            continue;
        };
        if a.side_of(to.id).is_none() || from.id == to.id {
            continue;
        }
        let own_law_in_force = a.laws[usize::from(side)].is_some_and(|law| {
            from.laws
                .iter()
                .any(|l| l.id == law && l.status == civ_agents::polity::LawStatus::InForce)
        });
        if a.in_force() && own_law_in_force {
            e.in_force.push(a.words(&name, &good));
        } else if a.open() {
            e.pending += 1;
        } else if !a.in_force() {
            e.over += 1;
        }
    }
    // Transfers in force each way (M5c slice AV, step two).
    let burden = |payer: &Polity, to: &Polity| {
        pop.burden(
            sim.land(),
            &rules.catalog,
            &rules.people,
            now,
            payer.id,
            to.id,
        )
    };
    let goods_words = |b: &civ_agents::population::Burden| {
        b.goods
            .iter()
            .map(|&(g, kg)| format!("{kg:.0} kg of {}", good(g)))
            .collect::<Vec<_>>()
            .join(" and ")
    };
    let permille = |x: f64| (x * 1000.0).round().clamp(0.0, f64::from(u32::MAX)) as u32;
    if let Some(b) = burden(from, to) {
        e.pays = Some(goods_words(&b));
        e.of_output = b.of_output().map(permille);
        e.of_surplus = b.of_surplus().map(permille);
    }
    e.paid = burden(to, from).map(|b| goods_words(&b));
    let (ours, other) = (from.claims_now(), to.claims_now());
    e.claims = ours.len() as u32;
    e.contested = ours
        .iter()
        .filter(|p| other.binary_search(p).is_ok())
        .count() as u32;
    e
}

/// Names a relation from its evidence alone; `other` is the other polity's settlement's name.
pub fn classify(e: &RelationEvidence, other: &str) -> RelationLabel {
    let share = |n: u32| f64::from(n) / f64::from(e.adults.max(1));
    let standing = if e.knowing == 0 && e.views == 0 && e.in_force.is_empty() {
        Standing::Unknown
    } else if e.pays.is_some() {
        Standing::Tributary
    } else if !e.in_force.is_empty() {
        Standing::UnderAgreement
    } else if share(e.harms) >= WARY && e.harms >= e.helps {
        Standing::Wary
    } else if share(e.helps) >= FRIENDLY {
        Standing::Friendly
    } else {
        Standing::Known
    };
    let mut why = Vec::new();
    if let Some(goods) = &e.pays {
        let pct = |x: u32| format!("{:.1} %", f64::from(x) / 10.0);
        let burden = match (e.of_output, e.of_surplus) {
            (None, _) => "its households' fields bring nothing yet".to_owned(),
            (Some(y), None) => format!(
                "{} of what its households' fields bring in an ordinary year, which is no more \
                 than its people need",
                pct(y)
            ),
            (Some(y), Some(s)) => format!(
                "{} of what its households' fields bring in an ordinary year, {} of what they \
                 bring beyond its people's need",
                pct(y),
                pct(s)
            ),
        };
        why.push(format!("it pays {other} {goods} a year: {burden}"));
    }
    if let Some(goods) = &e.paid {
        why.push(format!("{other} pays it {goods} a year"));
    }
    for terms in &e.in_force {
        why.push(format!("an agreement with {other} is in force: {terms}"));
    }
    if e.knowing == 0 {
        why.push(format!("none of its households knows {other}"));
    } else {
        why.push(format!(
            "{} of its households know {other}",
            of(e.knowing, e.households)
        ));
    }
    if e.views > 0 {
        why.push(format!(
            "{} of its adults hold a view of {other}'s polity: {} believe its people harm \
             theirs, {} that they help them",
            of(e.views, e.adults),
            e.harms,
            e.helps
        ));
    }
    if e.aggrieved > 0 {
        why.push(format!(
            "{} hold a grievance against households of {other} for working places their polity \
             claims",
            of(e.aggrieved, e.adults)
        ));
    }
    if e.claims > 0 {
        why.push(format!(
            "it claims {} {}, {} of them claimed by {other} too",
            e.claims,
            if e.claims == 1 { "place" } else { "places" },
            e.contested
        ));
    }
    if e.pending > 0 {
        why.push(format!(
            "{} with {other} {} before a gathering or awaiting word",
            plural(e.pending, "agreement", "agreements"),
            if e.pending == 1 { "is" } else { "are" }
        ));
    }
    if e.over > 0 {
        why.push(format!(
            "{} sought with {other} failed or ended",
            plural(e.over, "agreement", "agreements")
        ));
    }
    RelationLabel { standing, why }
}

/// "1 agreement", "2 agreements".
fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// How `from` stands toward `to` now.
pub fn relation_of(sim: &crate::Sim, from: &Polity, to: &Polity) -> RelationLabel {
    let other = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == to.settlement)
        .map_or("another settlement", |s| s.name.as_str());
    classify(&evidence(sim, from, to), other)
}

/// The other polities `polity` stands toward: every other one whose settlement is lived in.
pub fn others(sim: &crate::Sim, polity: PermanentId) -> impl Iterator<Item = &Polity> {
    let land = sim.land();
    sim.people().polities.iter().filter(move |p| {
        p.id != polity
            && land
                .settlements
                .iter()
                .any(|s| s.id == p.settlement && s.abandoned.is_none())
    })
}

/// One agreement between two polities as one side's observer sees it (M5c slice AU, ADR-0020
/// §6): its terms, where it stands, and each side's law history, in words.
#[derive(Clone, Debug, PartialEq)]
pub struct AgreementView {
    pub id: PermanentId,
    /// "leave for Ashford's people to use the places Oakholt claims, for a year".
    pub terms: String,
    /// "in force since 3 May of year 12", "failed on 9 June of year 12: the gathering at Ashford
    /// turned it down".
    pub state: String,
    /// The law history of `from`'s side, and of the other's: "proposed by Pirra on 2 May of year
    /// 12; passed on 3 May of year 12, 9 for, 2 against, 14 of 30 adults came; heard on 20 May of
    /// year 12 that the other's gathering passed it".
    pub ours: String,
    pub theirs: String,
    /// Each payment it owed, oldest first (M5c slice AV): "400 kg of grain from Oakholt's store,
    /// owed from 3 May of year 12: 400 kg set aside, 398 kg arrived; met on 9 May of year 12".
    pub payments: Vec<String>,
}

/// The agreements between `from` and `to`, newest first, as `from`'s observer sees them.
pub fn agreements_between(sim: &crate::Sim, from: &Polity, to: &Polity) -> Vec<AgreementView> {
    use civ_agents::agreements::{AgreementState, Failure};
    use civ_agents::polity::{LawStatus, Outcome, day_words};
    let pop = sim.people();
    let name = |p: PermanentId| {
        pop.polities
            .iter()
            .find(|x| x.id == p)
            .and_then(|x| sim.land().settlements.iter().find(|s| s.id == x.settlement))
            .map_or_else(|| "another settlement".to_owned(), |s| s.name.clone())
    };
    let on = |day: i64| day_words(civ_core::SimTime::from_minutes(day * 24 * 60));
    let good = |g: u16| good_name(sim, g);
    let mut out = Vec::new();
    for a in pop.agreements.list.iter().rev() {
        let (Some(us), Some(them)) = (a.side_of(from.id), a.side_of(to.id)) else {
            continue;
        };
        if us == them {
            continue;
        }
        let side_name = |s: u8| name(a.polities[usize::from(s)]);
        let state = match a.state {
            AgreementState::Offered => format!("offered on {}, not yet in force", on(a.made)),
            AgreementState::InForce { since } => format!("in force since {}", on(since)),
            AgreementState::Failed { day, why } => {
                let why = match why {
                    Failure::NoTerms => "the two who met agreed on no terms".to_owned(),
                    Failure::TurnedDown(s) => {
                        format!("the gathering at {} turned it down", side_name(s))
                    }
                    Failure::TooFew(s) => {
                        format!("too few came to the gathering at {}", side_name(s))
                    }
                    Failure::NeverCalled(s) => {
                        format!("nobody put it to the gathering at {} in time", side_name(s))
                    }
                    Failure::Unanswered(s) => format!(
                        "{} never heard in time of the other's decision",
                        side_name(s)
                    ),
                };
                format!("failed on {}: {why}", on(day))
            }
            AgreementState::Ended { day, by } => match by {
                None => format!("ended on {}: its term ran out", on(day)),
                Some(s) => format!("ended on {}: a law at {} ended it", on(day), side_name(s)),
            },
        };
        let history = |s: u8| {
            let i = usize::from(s);
            let polity = pop.polities.iter().find(|p| p.id == a.polities[i]);
            let law = a.laws[i].and_then(|l| polity?.laws.iter().find(|x| x.id == l));
            let Some(l) = law else {
                return if a.open() {
                    format!(
                        "met by {}; not yet put to its gathering",
                        pop.name_of(a.negotiators[i])
                    )
                } else {
                    format!(
                        "met by {}; never put to its gathering",
                        pop.name_of(a.negotiators[i])
                    )
                };
            };
            let mut words = format!(
                "proposed by {} on {}",
                pop.name_of(l.sponsor),
                day_words(l.proposed)
            );
            let (present, support, oppose) = l.counts();
            let tally = format!(
                "{support} for, {oppose} against, {present} of {} came",
                l.eligible
            );
            match (l.outcome, l.decided) {
                (Some(Outcome::Passed), Some(t)) => {
                    words += &format!("; passed on {}, {tally}", day_words(t));
                }
                (Some(Outcome::Failed | Outcome::Tied), Some(t)) => {
                    words += &format!("; turned down on {}, {tally}", day_words(t));
                }
                (Some(Outcome::NoQuorum), Some(t)) => {
                    words += &format!("; too few came on {}, {tally}", day_words(t));
                }
                _ => words += "; before its gathering",
            }
            if let Some(d) = a.heard[i] {
                words += &format!("; heard on {} that the other's gathering passed it", on(d));
            }
            if l.status == LawStatus::Lapsed {
                words += "; lapsed";
            }
            words
        };
        let payments = pop
            .agreements
            .dues
            .iter()
            .filter(|d| d.agreement == a.id)
            .map(|d| {
                use civ_agents::agreements::DueState;
                let mut words = format!(
                    "{:.0} kg of {} from {}'s store, owed from {}: ",
                    d.owed_kg,
                    good(d.good),
                    name(d.from),
                    on(d.made)
                );
                if d.set_aside_kg <= 0.0 {
                    words += "nothing set aside";
                } else {
                    words += &format!(
                        "{:.0} kg set aside, {:.0} kg arrived",
                        d.set_aside_kg, d.arrived_kg
                    );
                }
                match d.state {
                    DueState::Open => {
                        words += &format!("; due by {}", on(d.due));
                        if let Some(c) = d.carrier {
                            words += &format!(", {} to carry it", pop.name_of(c));
                        }
                    }
                    DueState::Met { day } => words += &format!("; met on {}", on(day)),
                    DueState::Missed { day, why } => {
                        words += &format!("; missed on {}: {}", on(day), why.words());
                    }
                }
                words
            })
            .collect();
        out.push(AgreementView {
            id: a.id,
            terms: a.words(&name, &good),
            state,
            ours: history(us),
            theirs: history(them),
            payments,
        });
    }
    out
}

/// A good's name, in lower case: "grain".
fn good_name(sim: &crate::Sim, g: u16) -> String {
    sim.rules()
        .catalog
        .goods
        .get(usize::from(g))
        .map_or_else(|| "goods".to_owned(), |d| d.name.to_lowercase())
}

/// "3 of 4", "1 of 1".
fn of(a: u32, b: u32) -> String {
    format!("{a} of {b}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(adults: u32, views: u32, harms: u32, helps: u32) -> RelationEvidence {
        RelationEvidence {
            households: 5,
            knowing: 3,
            adults,
            views,
            harms,
            helps,
            ..RelationEvidence::default()
        }
    }

    #[test]
    fn a_relation_is_named_by_what_its_people_hold_and_says_why() {
        let none = RelationEvidence {
            households: 5,
            adults: 12,
            ..RelationEvidence::default()
        };
        assert_eq!(classify(&none, "Ashford").standing, Standing::Unknown);
        assert_eq!(
            classify(&known(12, 0, 0, 0), "Ashford").standing,
            Standing::Known
        );
        // One in twenty leaning either way is not enough.
        assert_eq!(
            classify(&known(20, 3, 1, 0), "Ashford").standing,
            Standing::Known
        );
        let wary = RelationEvidence {
            aggrieved: 2,
            claims: 3,
            contested: 1,
            ..known(20, 5, 3, 1)
        };
        let label = classify(&wary, "Ashford");
        assert_eq!(label.standing, Standing::Wary);
        assert_eq!(
            label.why,
            vec![
                "3 of 5 of its households know Ashford".to_owned(),
                "5 of 20 of its adults hold a view of Ashford's polity: 3 believe its people \
                 harm theirs, 1 that they help them"
                    .to_owned(),
                "2 of 20 hold a grievance against households of Ashford for working places their \
                 polity claims"
                    .to_owned(),
                "it claims 3 places, 1 of them claimed by Ashford too".to_owned(),
            ]
        );
        assert_eq!(
            classify(&known(20, 4, 0, 3), "Ashford").standing,
            Standing::Friendly
        );
        // Leaning both ways as much, harm is named.
        assert_eq!(
            classify(&known(20, 6, 3, 3), "Ashford").standing,
            Standing::Wary
        );
    }

    #[test]
    fn an_agreement_in_force_by_its_own_law_names_the_relation_and_says_its_terms() {
        let bound = RelationEvidence {
            in_force: vec![
                "leave for Ashford's people to use the places Oakholt claims, for a year"
                    .to_owned(),
            ],
            over: 2,
            ..known(20, 5, 3, 1)
        };
        let label = classify(&bound, "Ashford");
        assert_eq!(label.standing, Standing::UnderAgreement);
        assert_eq!(label.standing.words(), "under agreement");
        assert_eq!(
            label.why.first().map(String::as_str),
            Some(
                "an agreement with Ashford is in force: leave for Ashford's people to use the \
                 places Oakholt claims, for a year"
            )
        );
        assert_eq!(
            label.why.last().map(String::as_str),
            Some("2 agreements sought with Ashford failed or ended")
        );
        // One awaiting word names nothing yet.
        let awaiting = RelationEvidence {
            pending: 1,
            ..known(20, 0, 0, 0)
        };
        let label = classify(&awaiting, "Ashford");
        assert_eq!(label.standing, Standing::Known);
        assert!(label.why.contains(
            &"1 agreement with Ashford is before a gathering or awaiting word".to_owned()
        ));
    }

    #[test]
    fn a_polity_paying_a_transfer_in_force_is_tributary_and_its_burden_given_both_ways() {
        let terms = "leave for Oakholt's people to use the places Ashford claims, for five \
                     years, for 100 kg of grain from Oakholt's store to Ashford's each year";
        let paying = RelationEvidence {
            in_force: vec![terms.to_owned()],
            pays: Some("100 kg of grain".to_owned()),
            of_output: Some(25),
            of_surplus: Some(100),
            ..known(20, 0, 0, 0)
        };
        let label = classify(&paying, "Ashford");
        assert_eq!(label.standing, Standing::Tributary);
        assert_eq!(label.standing.words(), "tributary");
        assert_eq!(
            label.why[0],
            "it pays Ashford 100 kg of grain a year: 2.5 % of what its households' fields bring \
             in an ordinary year, 10.0 % of what they bring beyond its people's need"
        );
        // 13-02 §2.1: no surplus ratio where the fields bring no more than the need.
        let short = RelationEvidence {
            of_surplus: None,
            ..paying.clone()
        };
        assert!(
            classify(&short, "Ashford").why[0].ends_with("which is no more \
                 than its people need")
        );
        // The other side is under agreement, and says what it receives.
        let receiving = RelationEvidence {
            in_force: vec![terms.to_owned()],
            paid: Some("100 kg of grain".to_owned()),
            ..known(20, 0, 0, 0)
        };
        let label = classify(&receiving, "Oakholt");
        assert_eq!(label.standing, Standing::UnderAgreement);
        assert_eq!(label.why[0], "Oakholt pays it 100 kg of grain a year");
    }
}
