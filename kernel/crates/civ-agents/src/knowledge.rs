//! What people know (ADR-0008): what founders bring, and each settlement's record of when it
//! came to know a technique and when it lost one. Knowledge itself is kept by each person
//! ([`crate::person::Know`]); what a settlement knows is derived from its people, never stored.

use civ_core::rng::Rng64;
use civ_core::{PermanentId, SimTime};

use crate::params::{Catalog, GoodDef, KnowledgeParams};
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
    /// Another settlement it concerns (M5b slice AR): for its coming to be known, the one its
    /// knower came from; for its loss, one where it is still known that someone left here has
    /// kin or a tie in.
    pub elsewhere: Option<PermanentId>,
}

/// The age from which a founder can know technique `t`: the age at which a child learns it at
/// home ([`upbringing_ages`]), or for a craft not learnt in childhood, the youngest age of the
/// work it gates or adulthood (`grown_at`), whichever is later.
pub fn knowing_age(catalog: &Catalog, t: usize, grown_at: f64) -> Option<f64> {
    if let Some((from, _)) = upbringing_ages(catalog, t, grown_at) {
        return Some(from);
    }
    Some(catalog.work_age(t)?.max(grown_at))
}

/// Years before growing up in which a child learns at home the techniques of work first done
/// later (a tuning value): they have watched it all their childhood.
pub const UPBRINGING_LEAD_YEARS: f64 = 1.0;

/// The ages at which someone learns technique `t` at home (ADR-0008 §4), if it is learnt in
/// upbringing: from the youngest age of its work, or for work first done later than a year
/// before growing up ([`UPBRINGING_LEAD_YEARS`]), from then, until they are grown (`grown_at`).
/// So all of a household's work is learnt at home before anyone leaves it to keep their own.
pub fn upbringing_ages(catalog: &Catalog, t: usize, grown_at: f64) -> Option<(f64, f64)> {
    catalog.techniques.get(t).filter(|d| d.upbringing)?;
    let work = catalog.work_age(t)?;
    Some((
        work.min(grown_at - UPBRINGING_LEAD_YEARS).max(0.0),
        grown_at,
    ))
}

/// The chance that `qualified_h` hours of experiment find a technique whose median find takes
/// `e50_h` qualified hours (ADR-0008 §3; research 07-01 §5.3): 1 − exp(−ln 2 · hours / E50).
/// The hazard is per hour, so splitting the hours changes nothing.
pub fn find_chance(qualified_h: f64, e50_h: f64) -> f64 {
    if !(qualified_h > 0.0 && e50_h > 0.0) {
        return 0.0;
    }
    1.0 - (-std::f64::consts::LN_2 * qualified_h / e50_h).exp()
}

/// Food energy, kcal, expected to spoil from a stock of `kcal` that keeps with a half-life of
/// `half_life_days` (0: it keeps), in a household that eats `eaten_per_day` kcal a day and first
/// eats `before_kcal` of foods that spoil sooner (people eat what spoils first). The stock decays
/// while the others are eaten, then is eaten as it decays: with λ = ln 2 / half-life and c the
/// rate, a stock S eaten from at once is gone after ln(1 + λS/c)/λ days, having lost
/// S − (c/λ)·ln(1 + λS/c) (dS/dt = −λS − c).
pub fn expected_spoilage_kcal(
    kcal: f64,
    half_life_days: f64,
    eaten_per_day: f64,
    before_kcal: f64,
) -> f64 {
    if kcal <= 0.0 || half_life_days <= 0.0 {
        return 0.0;
    }
    if eaten_per_day <= 0.0 {
        return kcal;
    }
    let lambda = std::f64::consts::LN_2 / half_life_days;
    let left = kcal * (-lambda * before_kcal.max(0.0) / eaten_per_day).exp();
    let k = eaten_per_day / lambda;
    let eaten = k * (left / k).ln_1p();
    (kcal - eaten).clamp(0.0, kcal)
}

/// Least of a good, kilograms or tools, that counts as holding it for a technique's `needs`.
pub const NEEDS_MIN: f64 = 0.01;

/// Least share of a household's food that a problem must threaten for anyone to try at it: a
/// problem nobody has gets no attention (a tuning value).
pub const MIN_PROBLEM_SHARE: f64 = 0.05;

/// Whether someone who knows what `knows` says could find technique `t` with the household's
/// `stores` (ADR-0008 §3): one of its routes is wholly known to them (or it needs none), and the
/// goods it needs are at home. Whether they know it already is for the caller.
pub fn could_find(
    catalog: &Catalog,
    t: usize,
    knows: &dyn Fn(usize) -> bool,
    stores: &[f64],
) -> bool {
    let Some(def) = catalog.techniques.get(t) else {
        return false;
    };
    let route = def.requires.is_empty() || def.requires.iter().any(|r| r.iter().all(|&u| knows(u)));
    route
        && def
            .needs
            .iter()
            .all(|&g| stores.get(g).copied().unwrap_or(0.0) >= NEEDS_MIN)
}

/// Food energy, kcal, expected to spoil from a household's `stores` of the `answered` goods
/// before it is eaten ([`expected_spoilage_kcal`]): each is eaten at the household's daily need,
/// `kcal_day`, after the foods ready to eat that keep less long. Under a roof (`sheltered`) goods
/// keep by their sheltered half-lives.
pub fn spoiling_kcal(
    goods: &[GoodDef],
    stores: &[f64],
    sheltered: bool,
    kcal_day: f64,
    answered: &[usize],
) -> f64 {
    let keeps = |g: &GoodDef| {
        if sheltered && g.sheltered_half_life_days > 0.0 {
            g.sheltered_half_life_days
        } else {
            g.half_life_days
        }
    };
    answered
        .iter()
        .filter_map(|&g| goods.get(g).map(|d| (g, d)))
        .map(|(g, d)| {
            let life = keeps(d);
            let kcal = stores.get(g).copied().unwrap_or(0.0).max(0.0) * d.kcal_per_kg;
            if kcal <= 0.0 || life <= 0.0 {
                return 0.0;
            }
            let before: f64 = goods
                .iter()
                .zip(stores)
                .filter(|(o, _)| o.edible() && !o.kept_back())
                .filter(|(o, _)| keeps(o) > 0.0 && keeps(o) < life)
                .map(|(o, kg)| kg.max(0.0) * o.kcal_per_kg)
                .sum();
            expected_spoilage_kcal(kcal, life, kcal_day, before)
        })
        .sum()
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
            digs: None,
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
            answers_spoilage: Vec::new(),
        };
        Catalog {
            activities: vec![activity("sow", 10.0, 0), activity("weave", 12.0, 1)],
            techniques: vec![technique("sowing", true), technique("weaving", false)],
            ..Catalog::default()
        }
    }

    fn params() -> KnowledgeParams {
        KnowledgeParams {
            founders: Vec::new(),
            max_learners: 2,
            w_learn: 2.0,
            experiment_share: 0.01,
            aware_try_factor: 3.0,
            w_try: 2.0,
            try_gap_days: 7.0,
            watch_m: 100.0,
        }
    }

    #[test]
    fn a_find_is_even_odds_at_the_median_hours_however_the_hours_are_split() {
        assert!((find_chance(1000.0, 1000.0) - 0.5).abs() < 1e-12);
        assert_eq!(find_chance(0.0, 1000.0), 0.0);
        let (a, b) = (find_chance(300.0, 1000.0), find_chance(700.0, 1000.0));
        assert!((1.0 - (1.0 - a) * (1.0 - b) - 0.5).abs() < 1e-12);
        // Keyed draws at the median find half the time.
        let finds = (0..10_000u64)
            .filter(|&i| Rng64::from_key(&[3, i]).next_f64() < find_chance(500.0, 500.0))
            .count();
        assert!((4850..=5150).contains(&finds), "{finds}");
    }

    #[test]
    fn a_find_needs_a_known_route_and_its_goods_at_home() {
        let mut c = catalog();
        // Weaving needs sowing known, and good 0 at home.
        c.techniques[1].requires = vec![vec![0]];
        c.techniques[1].needs = vec![0];
        let knows_sowing = |t: usize| t == 0;
        assert!(could_find(&c, 1, &knows_sowing, &[1.0]));
        assert!(
            !could_find(&c, 1, &knows_sowing, &[0.0]),
            "nothing to try with"
        );
        assert!(!could_find(&c, 1, &|_| false, &[1.0]), "no route known");
        // Alternative routes: either will do.
        c.techniques[1].requires = vec![vec![0, 1], vec![0]];
        assert!(could_find(&c, 1, &knows_sowing, &[1.0]));
        assert!(
            !could_find(&c, 9, &knows_sowing, &[1.0]),
            "no such technique"
        );
    }

    #[test]
    fn a_household_counts_what_would_spoil_of_the_goods_a_problem_names() {
        let food = |id: &str, kcal: f64, half_life: f64, sheltered: f64| GoodDef {
            id: id.into(),
            name: id.into(),
            purpose: crate::params::GoodUse::Food,
            kcal_per_kg: kcal,
            half_life_days: half_life,
            sheltered_half_life_days: sheltered,
            eaten: crate::params::Eaten::Cooked,
            shared: false,
            reserve_for: None,
            tool: None,
            timber: None,
            store: None,
        };
        let goods = vec![
            food("fish", 600.0, 2.0, 0.0),
            food("meat", 1500.0, 3.0, 6.0),
        ];
        let stores = [10.0, 30.0];
        // The meat is eaten after the fish, which spoils sooner.
        let meat = spoiling_kcal(&goods, &stores, false, 10_000.0, &[1]);
        assert!((meat - expected_spoilage_kcal(45_000.0, 3.0, 10_000.0, 6_000.0)).abs() < 1e-6);
        // Under a roof it keeps longer, and less of it spoils.
        assert!(spoiling_kcal(&goods, &stores, true, 10_000.0, &[1]) < meat);
        // Both goods count together.
        let both = spoiling_kcal(&goods, &stores, false, 10_000.0, &[0, 1]);
        assert!(both > meat);
        assert_eq!(spoiling_kcal(&goods, &stores, false, 10_000.0, &[]), 0.0);
    }

    #[test]
    fn what_spoils_is_what_is_not_eaten_before_it_goes_off() {
        // Nothing eaten: all of it spoils; a good that keeps never does.
        assert_eq!(expected_spoilage_kcal(1000.0, 3.0, 0.0, 0.0), 1000.0);
        assert_eq!(expected_spoilage_kcal(1000.0, 0.0, 10.0, 0.0), 0.0);
        // A deer's meat (30 kg at 1,500 kcal) for five at 2,000 kcal a day, half-life 3 days:
        // S − (c/λ)·ln(1 + λS/c), about a third of it.
        let (s, c, l) = (45_000.0, 10_000.0, std::f64::consts::LN_2 / 3.0);
        let want = s - c / l * (1.0 + l * s / c).ln();
        assert!((expected_spoilage_kcal(s, 3.0, c, 0.0) - want).abs() < 1e-6);
        assert!((13_000.0..15_000.0).contains(&want), "{want}");
        // Eating others first leaves it longer to spoil.
        assert!(expected_spoilage_kcal(s, 3.0, c, 20_000.0) > want);
        // Plenty of mouths: little spoils.
        assert!(expected_spoilage_kcal(s, 3.0, 1e9, 0.0) < 1.0);
    }

    #[test]
    fn founders_know_what_their_age_and_share_allow() {
        let c = catalog();
        let all = KnowledgeParams {
            founders: vec![(0, 1.0), (1, 1.0)],
            ..params()
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
    fn work_learnt_at_home_is_learnt_before_growing_up() {
        let c = catalog();
        assert_eq!(upbringing_ages(&c, 0, 16.0), Some((10.0, 16.0)));
        // Work first done once grown is learnt at home in the last year before growing up.
        assert_eq!(upbringing_ages(&c, 0, 10.0), Some((9.0, 10.0)));
        assert_eq!(upbringing_ages(&c, 0, 8.0), Some((7.0, 8.0)));
        assert_eq!(knowing_age(&c, 0, 8.0), Some(7.0));
        // A craft is not learnt at home at all.
        assert_eq!(upbringing_ages(&c, 1, 16.0), None);
    }

    #[test]
    fn a_share_is_kept_on_average() {
        let c = catalog();
        let half = KnowledgeParams {
            founders: vec![(1, 0.5)],
            ..params()
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
