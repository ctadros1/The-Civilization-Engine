//! What a polity would be called, worked out afterwards from its saved history (ADR-0013 §6;
//! research 09-15 §2, §6). The classifier is pure: [`evidence`] summarises a polity's rules and
//! what happened under them over a window, and [`classify`] names it from that alone, with a
//! confidence and the reasons why. Nothing is saved, and nothing that decides what people do can
//! read a label: the people live in `civ-agents`, which cannot see this crate (09-01 §6.5: "the
//! label must never grant the capabilities used to classify it").
//!
//! The thresholds are display conventions, uncalibrated engineering defaults in 09-15 §4B's
//! ranges, not historical boundaries.

use civ_agents::polity::{Law, LawStatus, Membership, Outcome, PolicyDef, PolicyKind, Polity};
use civ_core::{PermanentId, SimTime};

/// The window of behaviour a label weighs, days (09-15 §4B: 24 months; test 12–36).
pub const WINDOW_DAYS: i64 = 730;
/// The share of the governed adults a body must admit to be called broad (09-15 §4B: 90 %, a UI
/// convention; test 80–95 %).
pub const BROAD: f64 = 0.9;
/// Below this share of the governed adults, a deciding body is a restricted group (09-15 §2:
/// oligarchy is a restricted group's decisive control; the half is a tuning value).
pub const RESTRICTED: f64 = 0.5;
/// The share of what passed that one sponsor must have put forward to set the agenda (09-15 §4B:
/// a dominant controller at about two thirds; test 0.60–0.80).
pub const DOMINANT: f64 = 2.0 / 3.0;
/// The fewest laws passed in the window before one sponsor can be called dominant (a tuning
/// value: two of two is not yet a pattern).
pub const DOMINANT_MIN: u32 = 3;
/// The share of a dominant sponsor's backing that regard for them carried, for a following (a
/// tuning value; 09-15 §2: big-man leadership rests on a following).
pub const FOLLOWING: f64 = 0.5;
/// Below this mean turnout, few come to the body's decisions (a tuning value; 09-15 §6.3 keeps
/// turnout apart from eligibility).
pub const FEW_COME: f64 = 0.5;
/// The fewest levies owed (paid or kept back) before compliance is judged; fewer are unobserved
/// (09-15 §6.5: an empty denominator is not perfect compliance).
pub const LEVIES_MIN: u32 = 10;

/// An office the polity's laws created, as its history shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct OfficeEvidence {
    /// Its title, from its policy template: "Storekeeper".
    pub title: String,
    /// Its holder now, if any.
    pub holder: Option<PermanentId>,
    /// Since when they hold it.
    pub since: Option<SimTime>,
    /// How many have held it, one law each.
    pub holders: u32,
}

/// What a label is worked out from: the polity's rules (de jure) and what happened under them in
/// the window (de facto), as plain values (09-15 §6.2).
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    /// The share of the governed adults the deciding body admits.
    pub body_share: f64,
    /// The governed adults now.
    pub adults: u32,
    /// Whether the polity is younger than the window, so the window runs from its founding.
    pub young: bool,
    /// Laws it passed before the window that are still in force: they still bind.
    pub standing_laws: u32,
    /// Matters the body decided in the window, whatever the outcome.
    pub decided: u32,
    /// Of them, decisions that failed for want of a quorum.
    pub no_quorum: u32,
    /// The mean share of the body that came to them.
    pub turnout: Option<f64>,
    /// Laws it passed in the window.
    pub passed: u32,
    /// The sponsor of the most laws passed in the window, and how many.
    pub top_sponsor: Option<(PermanentId, u32)>,
    /// How many put forward a law that passed in the window.
    pub sponsors: u32,
    /// Those who backed the top sponsor's laws, but for the sponsor themselves.
    pub backers: u32,
    /// Of them, those whose household would have gained too little to back it: regard for the
    /// one it put forward carried them.
    pub carried: u32,
    /// Levies paid, and kept back by choice, under the levies in force (those unable, or who did
    /// not know of it, did not refuse: 09-01 §6.4).
    pub levies_paid: u32,
    /// Levies kept back.
    pub levies_kept: u32,
    /// The offices its laws created.
    pub offices: Vec<OfficeEvidence>,
}

/// A polity's label: a name, what qualifies it, a confidence and the reasons, in words.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    /// The principal description: "Council community", "Council community — big-man leadership".
    pub name: String,
    /// What qualifies it: "a storekeeper's office", "its levy mostly paid".
    pub modifiers: Vec<String>,
    /// How much evidence stands behind it, 0–1 (uncalibrated).
    pub confidence: f64,
    /// Why, in sentences.
    pub why: Vec<String>,
}

impl Label {
    /// The name in a sentence, with its article: "a council community", "an oligarchy".
    pub fn in_prose(&self) -> String {
        let mut chars = self.name.chars();
        let first: String = chars
            .next()
            .map(char::to_lowercase)
            .into_iter()
            .flatten()
            .collect();
        let name = format!("{first}{}", chars.as_str());
        let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
            "an"
        } else {
            "a"
        };
        format!("{article} {name}")
    }
}

/// The share of the governed adults that a body of membership `m` admits.
fn admits(m: Membership) -> f64 {
    match m {
        Membership::Adults => 1.0,
    }
}

/// The evidence for polity `polity` at `now`, its body governing `adults` adults. `policies` is the
/// catalog's, and `margin` the polity's stance margin: a backer whose household's points stay
/// within it was carried by regard.
pub fn evidence(
    polity: &Polity,
    policies: &[PolicyDef],
    adults: u32,
    now: SimTime,
    margin: f64,
) -> Evidence {
    let from = now.day_index() - WINDOW_DAYS;
    let recent: Vec<&Law> = polity
        .laws
        .iter()
        .filter(|l| l.decided.is_some_and(|t| t.day_index() > from))
        .collect();
    let kind = |l: &Law| policies.get(usize::from(l.policy)).map(|d| d.kind);
    let turnouts: Vec<f64> = recent
        .iter()
        .filter(|l| l.eligible > 0)
        .map(|l| f64::from(l.counts().0) / f64::from(l.eligible))
        .collect();
    let passed: Vec<&&Law> = recent
        .iter()
        .filter(|l| l.outcome == Some(Outcome::Passed))
        .collect();
    let mut sponsors: Vec<(PermanentId, u32)> = Vec::new();
    for l in &passed {
        match sponsors.iter_mut().find(|s| s.0 == l.sponsor) {
            Some(s) => s.1 += 1,
            None => sponsors.push((l.sponsor, 1)),
        }
    }
    sponsors.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top_sponsor = sponsors.first().copied();
    let (mut backers, mut carried) = (0, 0);
    if let Some((top, _)) = top_sponsor {
        for l in passed.iter().filter(|l| l.sponsor == top) {
            for r in l.stances.iter().filter(|r| r.person != top) {
                if r.stance == civ_agents::polity::Stance::Support {
                    backers += 1;
                    if f64::from(r.gain) <= margin {
                        carried += 1;
                    }
                }
            }
        }
    }
    let (mut levies_paid, mut levies_kept) = (0, 0);
    for l in polity
        .laws
        .iter()
        .filter(|l| l.status == LawStatus::InForce && kind(l) == Some(PolicyKind::CommonStore))
    {
        levies_paid += l.compliance.complied;
        levies_kept += l.compliance.evaded;
    }
    let mut offices: Vec<OfficeEvidence> = Vec::new();
    for l in polity.laws.iter().filter(|l| {
        l.holder.is_some() && matches!(l.status, LawStatus::InForce | LawStatus::Lapsed)
    }) {
        let title = policies
            .get(usize::from(l.policy))
            .map_or_else(|| "Office".to_owned(), |d| d.name.clone());
        let o = match offices.iter_mut().position(|o| o.title == title) {
            Some(i) => &mut offices[i],
            None => {
                offices.push(OfficeEvidence {
                    title,
                    holder: None,
                    since: None,
                    holders: 0,
                });
                offices.last_mut().expect("just pushed")
            }
        };
        o.holders += 1;
        if l.status == LawStatus::InForce {
            o.holder = l.holder;
            o.since = l.decided;
        }
    }
    Evidence {
        body_share: admits(polity.body.members),
        adults,
        young: polity.founded.day_index() > from,
        standing_laws: polity
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce)
            .filter(|l| l.decided.is_some_and(|t| t.day_index() <= from))
            .count() as u32,
        decided: recent.len() as u32,
        no_quorum: recent
            .iter()
            .filter(|l| l.outcome == Some(Outcome::NoQuorum))
            .count() as u32,
        turnout: (!turnouts.is_empty())
            .then(|| turnouts.iter().sum::<f64>() / turnouts.len() as f64),
        passed: passed.len() as u32,
        top_sponsor,
        sponsors: sponsors.len() as u32,
        backers,
        carried,
        levies_paid,
        levies_kept,
        offices,
    }
}

/// Polity `polity`'s label in world `sim` now, its body governing the adults who live in its
/// settlement.
pub fn label_of(sim: &crate::Sim, polity: &Polity) -> Label {
    let (pop, rules, now) = (sim.people(), sim.rules(), sim.now());
    let adult = rules.people.family.independent_age;
    let adults = pop
        .people
        .iter()
        .filter(|(_, p)| p.age_years(now) >= adult)
        .filter(|(_, p)| {
            pop.household(p.household)
                .is_some_and(|x| x.settlement == Some(polity.settlement))
        })
        .count() as u32;
    let e = evidence(
        polity,
        &rules.catalog.policies,
        adults,
        now,
        rules.people.polity.stance_margin,
    );
    classify(&e, &|id| pop.name_of(id), &civ_agents::polity::day_words)
}

/// "3 of 4", "1 of 1".
fn of(a: u32, b: u32) -> String {
    format!("{a} of {b}")
}

/// A share in whole percent: "92 %".
fn percent(x: f64) -> String {
    format!("{:.0} %", 100.0 * x)
}

/// The label for evidence `e`. `name_of` names people and `day` renders a day, for the reasons.
pub fn classify(
    e: &Evidence,
    name_of: &dyn Fn(PermanentId) -> String,
    day: &dyn Fn(SimTime) -> String,
) -> Label {
    let mut why = Vec::new();
    let mut modifiers = Vec::new();
    // Who may decide (de jure): an assembly of nearly all, some, or a restricted group.
    let organization = if e.body_share >= BROAD {
        why.push(
            "All its adults may come and decide, and what they decide binds: leadership answers \
             to the gathering."
                .to_owned(),
        );
        "Council community"
    } else if e.body_share >= RESTRICTED {
        why.push(format!(
            "Its deciding body admits {} of its adults, and what it decides binds.",
            percent(e.body_share)
        ));
        modifiers.push(format!("{} of adults may decide", percent(e.body_share)));
        "Council community"
    } else {
        why.push(format!(
            "Only {} of its adults may decide, and what they decide binds the rest.",
            percent(e.body_share)
        ));
        "Oligarchy"
    };
    // Whether the procedure is exercised (de facto).
    let window = if e.young {
        "since it was founded".to_owned()
    } else {
        format!("in the last {} months", WINDOW_DAYS * 12 / 365)
    };
    if e.decided == 0 && e.standing_laws > 0 {
        why.push(format!(
            "It decided nothing {window}; {} it passed before still {}.",
            if e.standing_laws == 1 {
                "the law".to_owned()
            } else {
                format!("the {} laws", e.standing_laws)
            },
            if e.standing_laws == 1 {
                "binds"
            } else {
                "bind"
            }
        ));
    } else if e.decided == 0 {
        why.push(format!(
            "It decided nothing {window}: the custom stands, unexercised."
        ));
        modifiers.push("its custom not yet exercised".to_owned());
    } else {
        let came = e.turnout.map_or(String::new(), |t| {
            format!(", {} of its members coming on average", percent(t))
        });
        why.push(format!(
            "It decided {} {} {window}{came}; {} passed.",
            e.decided,
            if e.decided == 1 { "matter" } else { "matters" },
            e.passed
        ));
        if e.turnout.is_some_and(|t| t < FEW_COME) {
            modifiers.push("few come to decide".to_owned());
        }
    }
    // Who leads (headship): one sponsor setting the agenda, and whether a following carries it.
    let mut headship = None;
    match e.top_sponsor {
        Some((who, n))
            if e.passed >= DOMINANT_MIN && f64::from(n) >= DOMINANT * f64::from(e.passed) =>
        {
            let carried = if e.backers > 0 {
                f64::from(e.carried) / f64::from(e.backers)
            } else {
                0.0
            };
            if carried >= FOLLOWING {
                why.push(format!(
                    "{} put forward {} of what passed, and {} of those who backed it did so for \
                     regard of {} more than for their households: a following.",
                    name_of(who),
                    of(n, e.passed),
                    of(e.carried, e.backers),
                    name_of(who)
                ));
                headship = Some(format!("big-man leadership ({})", name_of(who)));
            } else {
                why.push(format!(
                    "{} put forward {} of what passed; most who backed it stood to gain.",
                    name_of(who),
                    of(n, e.passed)
                ));
                headship = Some(format!("led by its proposer ({})", name_of(who)));
            }
        }
        _ if e.passed >= DOMINANT_MIN => why.push(format!(
            "What passed was put forward by {} people: no one person sets its agenda.",
            e.sponsors
        )),
        _ if e.passed > 0 => why.push(format!(
            "{} passed, too few to tell whether one person sets its agenda.",
            if e.passed == 1 {
                "One law has".to_owned()
            } else {
                format!("{} laws have", e.passed)
            }
        )),
        _ => {}
    }
    // Offices, and whether one outlived its holder (ADR-0013 §6).
    for o in &e.offices {
        let title = o.title.to_lowercase();
        let passed_on = if o.holders > 1 {
            format!(
                "; it has passed from one holder to the next {} {}, by new laws",
                o.holders - 1,
                if o.holders == 2 { "time" } else { "times" }
            )
        } else {
            String::new()
        };
        match (o.holder, o.since) {
            (Some(h), Some(t)) => {
                why.push(format!(
                    "{} holds the office of {title}, named by the gathering on {}{passed_on}.",
                    name_of(h),
                    day(t)
                ));
                modifiers.push(if o.holders > 1 {
                    format!("a {title}'s office that outlived its holder")
                } else {
                    format!("a {title}'s office")
                });
            }
            _ => {
                why.push(format!("Its office of {title} stands empty{passed_on}."));
                modifiers.push(format!("its {title}'s office empty"));
            }
        }
    }
    // Whether its levy is paid (09-15 §6.5): too few cases are unobserved, not compliance.
    let owed = e.levies_paid + e.levies_kept;
    if owed >= LEVIES_MIN {
        let paid = f64::from(e.levies_paid) / f64::from(owed);
        why.push(format!(
            "Of those able to pay its levy and knowing of it, {} paid ({}).",
            percent(paid),
            of(e.levies_paid, owed)
        ));
        modifiers.push(
            if paid >= 0.8 {
                "its levy mostly paid"
            } else if paid >= 0.5 {
                "its levy often kept back"
            } else {
                "its levy mostly kept back"
            }
            .to_owned(),
        );
    } else if owed > 0 {
        why.push(format!(
            "Its levy has been owed {owed} {}, too few to judge whether it is paid.",
            if owed == 1 { "time" } else { "times" }
        ));
    }
    // The more it decided, and the more of what it decided before still binds, the more the label
    // rests on (uncalibrated): nothing decided rests on the custom alone.
    let n = f64::from(e.decided - e.no_quorum.min(e.decided) + e.standing_laws);
    let confidence = (n / (n + 2.0)).clamp(0.2, 0.95);
    let name = match headship {
        Some(h) => format!("{organization} — {h}"),
        None => organization.to_owned(),
    };
    Label {
        name,
        modifiers,
        confidence,
        why,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    /// A gathering of all its adults that has decided four matters, passed two by two sponsors,
    /// has a storekeeper and a levy mostly paid.
    fn council() -> Evidence {
        Evidence {
            body_share: 1.0,
            adults: 40,
            young: false,
            standing_laws: 0,
            decided: 4,
            no_quorum: 0,
            turnout: Some(0.9),
            passed: 2,
            top_sponsor: Some((pid(7), 1)),
            sponsors: 2,
            backers: 10,
            carried: 2,
            levies_paid: 90,
            levies_kept: 10,
            offices: vec![OfficeEvidence {
                title: "Storekeeper".to_owned(),
                holder: Some(pid(9)),
                since: Some(SimTime::from_minutes(400 * 1440)),
                holders: 1,
            }],
        }
    }

    fn label(e: &Evidence) -> Label {
        classify(e, &|p| format!("P{}", p.get()), &|t| {
            format!("day {}", t.day_index())
        })
    }

    #[test]
    fn a_gathering_of_all_its_adults_is_a_council_community() {
        let l = label(&council());
        assert_eq!(l.name, "Council community");
        assert_eq!(
            l.modifiers,
            vec!["a storekeeper's office", "its levy mostly paid"]
        );
        assert!((l.confidence - 4.0 / 6.0).abs() < 1e-9);
        assert!(
            l.why
                .iter()
                .any(|w| w.contains("2 laws have passed, too few"))
        );
        let three = label(&Evidence {
            passed: 3,
            sponsors: 3,
            ..council()
        });
        assert!(
            three
                .why
                .iter()
                .any(|w| w.contains("put forward by 3 people: no one person sets its agenda"))
        );
        assert!(l.why.iter().any(|w| w.contains("in the last 24 months")));
        let young = label(&Evidence {
            young: true,
            ..council()
        });
        assert!(young.why.iter().any(|w| w.contains("since it was founded")));
        assert!(
            l.why
                .iter()
                .any(|w| w.contains("P9 holds the office of storekeeper"))
        );
    }

    /// Table-driven: each change to the evidence, and the name and modifiers it should give.
    #[test]
    fn the_label_answers_the_evidence_and_nothing_else() {
        type Case = (
            &'static str,
            fn(&mut Evidence),
            &'static str,
            &'static [&'static str],
        );
        let cases: &[Case] = &[
            (
                "a custom never exercised",
                |e| {
                    e.decided = 0;
                    e.passed = 0;
                    e.turnout = None;
                    e.top_sponsor = None;
                    e.sponsors = 0;
                    e.offices.clear();
                    e.levies_paid = 0;
                    e.levies_kept = 0;
                },
                "Council community",
                &["its custom not yet exercised"],
            ),
            (
                "nothing decided lately, its laws standing",
                |e| {
                    e.decided = 0;
                    e.passed = 0;
                    e.turnout = None;
                    e.top_sponsor = None;
                    e.sponsors = 0;
                    e.standing_laws = 2;
                },
                "Council community",
                &["a storekeeper's office", "its levy mostly paid"],
            ),
            (
                // 09-15 §7: population doubling with institutions unchanged changes nothing.
                "twice the adults, the same institutions",
                |e| e.adults *= 2,
                "Council community",
                &["a storekeeper's office", "its levy mostly paid"],
            ),
            (
                // 09-15 §7: a council of privileged households excluding most adults is an
                // oligarchy, not a broad assembly.
                "a body admitting a third of the adults",
                |e| e.body_share = 1.0 / 3.0,
                "Oligarchy",
                &["a storekeeper's office", "its levy mostly paid"],
            ),
            (
                "a body admitting most but not nearly all",
                |e| e.body_share = 0.7,
                "Council community",
                &[
                    "70 % of adults may decide",
                    "a storekeeper's office",
                    "its levy mostly paid",
                ],
            ),
            (
                "one sponsor of most of what passed, backed on regard",
                |e| {
                    e.passed = 3;
                    e.top_sponsor = Some((pid(5), 3));
                    e.sponsors = 1;
                    e.backers = 10;
                    e.carried = 7;
                },
                "Council community — big-man leadership (P5)",
                &["a storekeeper's office", "its levy mostly paid"],
            ),
            (
                "one sponsor of most of what passed, backed for gain",
                |e| {
                    e.passed = 3;
                    e.top_sponsor = Some((pid(5), 2));
                    e.backers = 10;
                    e.carried = 1;
                },
                "Council community — led by its proposer (P5)",
                &["a storekeeper's office", "its levy mostly paid"],
            ),
            (
                "two of two passed is not yet a pattern",
                |e| {
                    e.top_sponsor = Some((pid(5), 2));
                    e.carried = 10;
                },
                "Council community",
                &["a storekeeper's office", "its levy mostly paid"],
            ),
            (
                "few come",
                |e| e.turnout = Some(0.3),
                "Council community",
                &[
                    "few come to decide",
                    "a storekeeper's office",
                    "its levy mostly paid",
                ],
            ),
            (
                "an office handed on by new laws",
                |e| e.offices[0].holders = 3,
                "Council community",
                &[
                    "a storekeeper's office that outlived its holder",
                    "its levy mostly paid",
                ],
            ),
            (
                "an office whose holder is gone",
                |e| {
                    e.offices[0].holder = None;
                    e.offices[0].since = None;
                },
                "Council community",
                &["its storekeeper's office empty", "its levy mostly paid"],
            ),
            (
                "a levy mostly kept back",
                |e| {
                    e.levies_paid = 4;
                    e.levies_kept = 8;
                },
                "Council community",
                &["a storekeeper's office", "its levy mostly kept back"],
            ),
            (
                // 09-15 §6.5: an empty or tiny denominator is unobserved, not compliance.
                "a levy owed too seldom to judge",
                |e| {
                    e.levies_paid = 3;
                    e.levies_kept = 0;
                },
                "Council community",
                &["a storekeeper's office"],
            ),
        ];
        for (what, change, name, modifiers) in cases {
            let mut e = council();
            change(&mut e);
            let l = label(&e);
            assert_eq!(l.name, *name, "{what}");
            assert_eq!(l.modifiers, *modifiers, "{what}");
            assert!(
                !l.why.is_empty() && (0.2..=0.95).contains(&l.confidence),
                "{what}"
            );
        }
    }

    #[test]
    fn a_label_reads_in_a_sentence() {
        let mut l = label(&council());
        assert_eq!(l.in_prose(), "a council community");
        l.name = "Oligarchy — led by its proposer (Ada)".to_owned();
        assert_eq!(l.in_prose(), "an oligarchy — led by its proposer (Ada)");
    }

    #[test]
    fn more_decisions_more_confidence_and_a_failed_quorum_adds_none() {
        let mut e = council();
        let at = |e: &Evidence| label(e).confidence;
        let four = at(&e);
        e.decided = 10;
        assert!(at(&e) > four);
        e.no_quorum = 6;
        assert!((at(&e) - four).abs() < 1e-9);
        e.decided = 0;
        assert!((at(&e) - 0.2).abs() < 1e-9);
        // Laws it passed before the window still bind, and count.
        e.standing_laws = 2;
        assert!((at(&e) - 0.5).abs() < 1e-9);
        assert!(
            label(&e)
                .why
                .iter()
                .any(|w| w.contains("the 2 laws it passed before still bind"))
        );
    }
}
