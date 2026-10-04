//! What people know (ADR-0008): what founders bring, and each settlement's record of when it
//! came to know a technique and when it lost one. Knowledge itself is kept by each person
//! ([`crate::person::Know`]); what a settlement knows is derived from its people, never stored.

use civ_core::rng::Rng64;
use civ_core::{PermanentId, SimTime};

use crate::params::{Catalog, KnowledgeParams};
use crate::person::{Know, KnowSource};

/// What happened to a technique in a settlement. Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnowledgeEventKind {
    /// Someone there came to know it while nobody there did: the first time, or again after it
    /// was lost.
    Known(KnowSource),
    /// The last person there who knew it died or left.
    Lost,
}

/// One entry in a settlement's record of a technique (ADR-0008 §2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnowledgeEvent {
    /// When.
    pub at: SimTime,
    /// Where.
    pub settlement: PermanentId,
    /// The technique, by index in the catalog's techniques.
    pub technique: u16,
    /// Who came to know it, or who knew it last.
    pub person: PermanentId,
    /// What happened.
    pub kind: KnowledgeEventKind,
}

/// The age from which a founder can know technique `t`: the youngest age of the work it gates,
/// or for a craft not learnt in childhood, adulthood (`grown_at`) if that is later.
pub fn knowing_age(catalog: &Catalog, t: usize, grown_at: f64) -> Option<f64> {
    let work = catalog.work_age(t)?;
    let upbringing = catalog.techniques.get(t).is_some_and(|d| d.upbringing);
    Some(if upbringing { work } else { work.max(grown_at) })
}

/// Years from the age of a technique's work during which someone learns it at home, when the
/// work is first done at or after growing up (a tuning value).
pub const UPBRINGING_GRACE_YEARS: f64 = 1.0;

/// The ages at which someone learns technique `t` at home (ADR-0008 §4), if it is learnt in
/// upbringing: from the youngest age of its work until they are grown (`grown_at`), or for work
/// first done at or after growing up, for [`UPBRINGING_GRACE_YEARS`] from its age.
pub fn upbringing_ages(catalog: &Catalog, t: usize, grown_at: f64) -> Option<(f64, f64)> {
    catalog.techniques.get(t).filter(|d| d.upbringing)?;
    let work = catalog.work_age(t)?;
    Some((work, grown_at.max(work + UPBRINGING_GRACE_YEARS)))
}

/// The techniques a founder of `age` brings (ADR-0008 §1): for each technique in the people
/// profile's founders' list, they know it if they are old enough for it and a draw falls within
/// its share. One draw is made per technique whatever the age, so the draws stay aligned.
pub fn founder_knowledge(
    catalog: &Catalog,
    params: &KnowledgeParams,
    grown_at: f64,
    age: f64,
    rng: &mut Rng64,
    now: SimTime,
) -> Vec<Know> {
    let mut out: Vec<Know> = Vec::new();
    for &(t, share) in &params.founders {
        let u = rng.next_f64();
        let old_enough = knowing_age(catalog, t, grown_at).is_some_and(|a| age >= a);
        if old_enough && u < share && !out.iter().any(|k| usize::from(k.technique) == t) {
            out.push(Know {
                technique: t as u16,
                known: true,
                hours: catalog.techniques.get(t).map_or(0.0, |d| d.learn_h) as f32,
                since: now,
                source: KnowSource::Founder,
                used: now,
            });
        }
    }
    out.sort_by_key(|k| k.technique);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{ActivityDef, Behavior, TechniqueDef};

    fn catalog() -> Catalog {
        let activity = |id: &str, min_age: f64, technique: usize| ActivityDef {
            id: id.into(),
            name: id.into(),
            doing: id.into(),
            behavior: Behavior::Farm,
            resource: None,
            task: Some(civ_land::FieldTask::Sow),
            recipe: None,
            tools: Vec::new(),
            rate: 1.0,
            par: 3.0,
            min_age_years: min_age,
            max_age_years: 70.0,
            min_minutes: 30,
            max_minutes: 120,
            daylight_only: true,
            max_walk_minutes: 30,
            technique: Some(technique),
        };
        let technique = |id: &str, upbringing: bool| TechniqueDef {
            id: id.into(),
            name: id.into(),
            can: "do it".into(),
            domain: None,
            requires: Vec::new(),
            tried_in: Vec::new(),
            needs: Vec::new(),
            e50_h: 1000.0,
            learn_h: 50.0,
            upbringing,
        };
        Catalog {
            activities: vec![activity("sow", 10.0, 0), activity("weave", 12.0, 1)],
            techniques: vec![technique("sowing", true), technique("weaving", false)],
            ..Catalog::default()
        }
    }

    #[test]
    fn founders_know_what_their_age_and_share_allow() {
        let c = catalog();
        let all = KnowledgeParams {
            founders: vec![(0, 1.0), (1, 1.0)],
            max_learners: 2,
            w_learn: 2.0,
        };
        let now = SimTime::from_minutes(0);
        let mut rng = Rng64::from_key(&[1]);
        let adult = founder_knowledge(&c, &all, 16.0, 30.0, &mut rng, now);
        assert_eq!(adult.len(), 2);
        assert!(
            adult
                .iter()
                .all(|k| k.known && k.source == KnowSource::Founder)
        );
        // A child knows what it was brought up with from the work's age, and a craft only once
        // grown.
        let child = founder_knowledge(&c, &all, 16.0, 11.0, &mut rng, now);
        assert_eq!(child.iter().map(|k| k.technique).collect::<Vec<_>>(), [0]);
        assert!(founder_knowledge(&c, &all, 16.0, 8.0, &mut rng, now).is_empty());
        // A share of nothing gives nobody the technique.
        let none = KnowledgeParams {
            founders: vec![(0, 0.0), (1, 0.0)],
            ..all
        };
        assert!(founder_knowledge(&c, &none, 16.0, 30.0, &mut rng, now).is_empty());
        assert_eq!(knowing_age(&c, 1, 16.0), Some(16.0));
        assert_eq!(knowing_age(&c, 0, 16.0), Some(10.0));
    }

    #[test]
    fn work_learnt_at_home_is_learnt_by_growing_up_or_within_a_year_of_its_age() {
        let c = catalog();
        assert_eq!(upbringing_ages(&c, 0, 16.0), Some((10.0, 16.0)));
        // Work first done once grown is still learnt at home, for a year.
        assert_eq!(upbringing_ages(&c, 0, 10.0), Some((10.0, 11.0)));
        assert_eq!(upbringing_ages(&c, 0, 8.0), Some((10.0, 11.0)));
        // A craft is not learnt at home at all.
        assert_eq!(upbringing_ages(&c, 1, 16.0), None);
    }

    #[test]
    fn a_share_is_kept_on_average() {
        let c = catalog();
        let half = KnowledgeParams {
            founders: vec![(1, 0.5)],
            max_learners: 2,
            w_learn: 2.0,
        };
        let now = SimTime::from_minutes(0);
        let knowers = (0..2000u64)
            .filter(|&i| {
                let mut rng = Rng64::from_key(&[7, i]);
                !founder_knowledge(&c, &half, 16.0, 30.0, &mut rng, now).is_empty()
            })
            .count();
        assert!((900..1100).contains(&knowers), "{knowers}");
    }
}
