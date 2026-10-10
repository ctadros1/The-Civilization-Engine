//! How one polity stands toward another, worked out afterwards from what its people hold
//! (ADR-0020 §1): unknown, known, friendly or wary, with the reasons why. No relation is saved and
//! nothing in the world reads a label: the people live in `civ-agents`, which cannot see this
//! crate. Each side's label is worked out from its own people alone, so the two can differ.
//! Agreements and tribute (slices AU and AV) will add their own names.
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
}

/// The name a label gives a relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Nobody of it knows of the other.
    Unknown,
    /// Known, and nobody's view leans far enough either way.
    Known,
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
    let standing = if e.knowing == 0 && e.views == 0 {
        Standing::Unknown
    } else if share(e.harms) >= WARY && e.harms >= e.helps {
        Standing::Wary
    } else if share(e.helps) >= FRIENDLY {
        Standing::Friendly
    } else {
        Standing::Known
    };
    let mut why = Vec::new();
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
    RelationLabel { standing, why }
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
}
