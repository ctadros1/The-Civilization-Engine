//! Forums (M7b slice BR, ADR-0025 §1): a law naming who hears which cases, besides the
//! gathering. Pure records and the forum's decision: hearing lives in `population/cases.rs`.

use civ_core::PermanentId;

use crate::polity::{Body, Outcome, PassRule, PolicyKind, Stance};

/// Who sits in a forum (ADR-0025 §1). Codes are part of content and saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForumSeat {
    /// One holder, named by the body as an office is, lapsing as an office does.
    Holder = 0,
    /// A panel drawn by a keyed lot from the body's members for each sitting.
    Panel = 1,
    /// The elders of the polity's households.
    Elders = 2,
}

impl ForumSeat {
    /// Every seat, in code order.
    pub const ALL: [ForumSeat; 3] = [ForumSeat::Holder, ForumSeat::Panel, ForumSeat::Elders];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            ForumSeat::Holder => "holder",
            ForumSeat::Panel => "panel",
            ForumSeat::Elders => "elders",
        }
    }

    /// The seat with an authored name.
    pub fn from_name(name: &str) -> Option<ForumSeat> {
        ForumSeat::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// How far a forum's sitters follow the findings they remember (M7b slice BS, ADR-0025 §2).
/// Codes are part of content: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority {
    /// Nothing beyond what they believe: a finding remembered is only remembered.
    Informative = 0,
    /// Points toward the recalled findings' direction by the share of tags matched.
    Persuasive = 1,
    /// A large weight toward it, which stake and regard can still outweigh.
    Binding = 2,
}

impl Authority {
    /// Every authority, in code order.
    pub const ALL: [Authority; 3] = [
        Authority::Informative,
        Authority::Persuasive,
        Authority::Binding,
    ];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            Authority::Informative => "informative",
            Authority::Persuasive => "persuasive",
            Authority::Binding => "binding",
        }
    }

    /// The authority with an authored name.
    pub fn from_name(name: &str) -> Option<Authority> {
        Authority::ALL.into_iter().find(|a| a.name() == name)
    }
}

/// The points a sitter's recalled findings move them toward a finding under authority
/// `authority` (M7b slice BS, ADR-0025 §2): each recalled finding (+1 found, -1 not) by the share
/// of tags it matches, averaged, times the authority's weight; nothing when informative or with
/// nothing recalled.
pub fn precedent_points(
    authority: Authority,
    recalled: &[(f64, f64)],
    p: &crate::crime::CrimeParams,
) -> f64 {
    if recalled.is_empty() {
        return 0.0;
    }
    let w = match authority {
        Authority::Informative => return 0.0,
        Authority::Persuasive => p.w_persuasive,
        Authority::Binding => p.w_binding,
    };
    w * recalled.iter().map(|(dir, share)| dir * share).sum::<f64>() / recalled.len() as f64
}

/// A forum template's terms (content, ADR-0025 §1): its seat, how many a panel draws, the kinds
/// of law whose cases it hears, and, for a panel or the elders, the rule they decide by (a
/// quorum of those who sit and a pass rule).
#[derive(Clone, Debug, PartialEq)]
pub struct ForumDef {
    pub seat: ForumSeat,
    /// For a panel: how many are drawn for each sitting.
    pub panel: u8,
    /// The kinds of law whose cases it hears.
    pub hears: Vec<PolicyKind>,
    /// For a panel or the elders: their rule.
    pub rule: Body,
    /// How far its sitters follow remembered findings (M7b slice BS).
    pub authority: Authority,
    /// Whether it may hear challenges of laws under limits in force (M7b slice BT, ADR-0025 §4).
    pub review: bool,
}

/// What a limit forbids (M7b slice BT, ADR-0025 §3): a predicate over a law's own fields, from a
/// closed set the kernel knows, which content parameterizes. Codes are part of content: append
/// only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Predicate {
    /// No law keeps a household from its own work in daylight: a curfew may cover at most `bound`
    /// minutes of the longest day's light.
    DaylightWork = 0,
    /// No levy above a share: a law may levy at most `bound` of a harvest.
    LevyShare = 1,
}

impl Predicate {
    /// Every predicate, in code order.
    pub const ALL: [Predicate; 2] = [Predicate::DaylightWork, Predicate::LevyShare];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            Predicate::DaylightWork => "daylight_work",
            Predicate::LevyShare => "levy_share",
        }
    }

    /// The predicate with an authored name.
    pub fn from_name(name: &str) -> Option<Predicate> {
        Predicate::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// A limit template's terms (content, M7b slice BT).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LimitDef {
    pub predicate: Predicate,
    /// Minutes of light for daylight work; a share of a harvest for a levy.
    pub bound: f64,
    /// The rule a change of it is decided by (entrenchment, ADR-0025 §3).
    pub change: PassRule,
}

impl LimitDef {
    /// What it forbids, in words: "no law may keep a household from its own work for more than
    /// 30 minutes of the longest day's light".
    pub fn words(&self) -> String {
        match self.predicate {
            Predicate::DaylightWork => format!(
                "no law may keep a household from its own work for more than {:.0} minutes of the \
                 longest day's light",
                self.bound
            ),
            Predicate::LevyShare => format!(
                "no law may levy more than {:.0} in every hundred of a harvest",
                self.bound * 100.0
            ),
        }
    }

    /// How a law with `kind`, `hours` and `levy_share` reads against it at `latitude_deg`
    /// (ADR-0025 §4): -1 clearly no breach to 1 clearly one, by how far it passes the bound as a
    /// share of the bound; none when the predicate does not bind a law of its fields.
    pub fn reading(
        &self,
        kind: PolicyKind,
        hours: (u8, u8),
        levy_share: f64,
        latitude_deg: f64,
    ) -> Option<f64> {
        let bound = self.bound.max(1e-9);
        match self.predicate {
            Predicate::DaylightWork if kind == PolicyKind::Curfew => {
                let lit = daylight_covered(hours, latitude_deg);
                Some(((lit - bound) / bound).clamp(-1.0, 1.0))
            }
            Predicate::LevyShare if levy_share > 0.0 => {
                Some(((levy_share - bound) / bound).clamp(-1.0, 1.0))
            }
            _ => None,
        }
    }
}

/// The most minutes of a day's light, over the year, that a curfew from `hours.0`:00 to
/// `hours.1`:00 covers at `latitude_deg` (M7b slice BT): sampled each week.
pub fn daylight_covered(hours: (u8, u8), latitude_deg: f64) -> f64 {
    let (from, to) = (i64::from(hours.0) * 60, i64::from(hours.1) * 60);
    let spans: Vec<(i64, i64)> = if from <= to {
        vec![(from, to)]
    } else {
        vec![(from, 24 * 60), (0, to)]
    };
    (0..365)
        .step_by(7)
        .map(|doy| {
            let (rise, set) = crate::needs::daylight(latitude_deg, doy);
            spans
                .iter()
                .map(|&(a, b)| (b.min(set) - a.max(rise)).max(0))
                .sum::<i64>() as f64
        })
        .fold(0.0, f64::max)
}

impl ForumDef {
    /// Whether it hears cases brought under a law of kind `kind`.
    pub fn hears(&self, kind: PolicyKind) -> bool {
        self.hears.contains(&kind)
    }

    /// Who sits, in words: "a hearer the gathering named", "a panel of 5 drawn by lot", "the
    /// elders".
    pub fn words(&self) -> String {
        match self.seat {
            ForumSeat::Holder => "a hearer the gathering named".to_owned(),
            ForumSeat::Panel => format!("a panel of {} drawn by lot for each sitting", self.panel),
            ForumSeat::Elders => "the elders of its households".to_owned(),
        }
    }
}

/// Who heard a case, to begin a sentence: "The hearer Ada", "A panel of 5", "The elders".
/// `holder` is the holder's name, for a holder's seat.
pub fn hearers_words(def: &ForumDef, holder: Option<&str>) -> String {
    match (def.seat, holder) {
        (ForumSeat::Holder, Some(n)) => format!("The hearer {n}"),
        (ForumSeat::Holder, None) => "The hearer".to_owned(),
        (ForumSeat::Panel, _) => format!("A panel of {}", def.panel),
        (ForumSeat::Elders, _) => "The elders".to_owned(),
    }
}

/// The laws whose cases forum `def` hears, in words: "the law against taking", "the laws
/// against taking and against raiding".
pub fn heard_words(def: &ForumDef) -> String {
    let what: Vec<&str> = def
        .hears
        .iter()
        .filter_map(|k| match k {
            PolicyKind::AgainstTaking => Some("against taking"),
            PolicyKind::AgainstRaiding => Some("against raiding"),
            PolicyKind::Privilege => Some("granting privileges"),
            _ => None,
        })
        .collect();
    match what.as_slice() {
        [] => "no law".to_owned(),
        [one] => format!("the law {one}"),
        [a, b] => format!("the laws {a} and {b}"),
        [rest @ .., last] => format!("the laws {} and {last}", rest.join(", ")),
    }
}

/// How forum `def`, sat by `sitters` (all who sit come: a forum is called for the case), would
/// decide on stances `stances` (person, stance): one holder alone finds only by standing for a
/// finding; a panel or the elders by their rule, counting those who sit.
pub fn forum_decides(
    def: &ForumDef,
    sitters: &[PermanentId],
    stances: &[(PermanentId, Stance)],
) -> Outcome {
    let n = sitters.len() as u32;
    if n == 0 {
        return Outcome::NoQuorum;
    }
    let (mut support, mut oppose) = (0u32, 0u32);
    for &(p, s) in stances {
        if !sitters.contains(&p) {
            continue;
        }
        match s {
            Stance::Support => support += 1,
            Stance::Oppose => oppose += 1,
            Stance::Abstain => {}
        }
    }
    match def.seat {
        ForumSeat::Holder => {
            if support > 0 {
                Outcome::Passed
            } else {
                Outcome::Failed
            }
        }
        ForumSeat::Panel | ForumSeat::Elders => def.rule.decide(n, n, support, oppose),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polity::{Membership, PassRule};

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn def(seat: ForumSeat, pass: PassRule) -> ForumDef {
        ForumDef {
            seat,
            panel: 5,
            hears: vec![PolicyKind::AgainstTaking],
            authority: Authority::Informative,
            review: false,
            rule: Body {
                members: Membership::Adults,
                quorum_share: 0.5,
                pass,
                seats: None,
            },
        }
    }

    #[test]
    fn a_holder_finds_alone_and_a_panel_by_its_rule() {
        let holder = def(ForumSeat::Holder, PassRule::MoreForThanAgainst);
        assert!(holder.hears(PolicyKind::AgainstTaking));
        assert!(!holder.hears(PolicyKind::Curfew));
        let one = [id(1)];
        assert_eq!(
            forum_decides(&holder, &one, &[(id(1), Stance::Support)]),
            Outcome::Passed
        );
        assert_eq!(
            forum_decides(&holder, &one, &[(id(1), Stance::Abstain)]),
            Outcome::Failed,
            "a holder who will not stand for it finds nothing"
        );
        // Another's stance is not the holder's.
        assert_eq!(
            forum_decides(&holder, &one, &[(id(2), Stance::Support)]),
            Outcome::Failed
        );
        let panel = def(ForumSeat::Panel, PassRule::TwoThirds);
        let five = [id(1), id(2), id(3), id(4), id(5)];
        let three_two = [
            (id(1), Stance::Support),
            (id(2), Stance::Support),
            (id(3), Stance::Support),
            (id(4), Stance::Oppose),
            (id(5), Stance::Oppose),
        ];
        assert_eq!(forum_decides(&panel, &five, &three_two), Outcome::Failed);
        let more = def(ForumSeat::Panel, PassRule::MoreForThanAgainst);
        assert_eq!(forum_decides(&more, &five, &three_two), Outcome::Passed);
        assert_eq!(forum_decides(&more, &[], &three_two), Outcome::NoQuorum);
        assert_eq!(ForumSeat::from_name("elders"), Some(ForumSeat::Elders));
        assert_eq!(more.words(), "a panel of 5 drawn by lot for each sitting");
        assert_eq!(hearers_words(&holder, Some("Ada")), "The hearer Ada");
        assert_eq!(heard_words(&holder), "the law against taking");
        let both = ForumDef {
            hears: vec![PolicyKind::AgainstTaking, PolicyKind::AgainstRaiding],
            ..holder
        };
        assert_eq!(
            heard_words(&both),
            "the laws against taking and against raiding"
        );
        let p = crate::crime::CrimeParams::core();
        let recalled = [(1.0, 1.0), (-1.0, 0.5)];
        assert_eq!(precedent_points(Authority::Informative, &recalled, &p), 0.0);
        let persuasive = precedent_points(Authority::Persuasive, &recalled, &p);
        assert!((persuasive - p.w_persuasive * 0.25).abs() < 1e-12);
        assert!(precedent_points(Authority::Binding, &recalled, &p) > persuasive);
        assert_eq!(precedent_points(Authority::Binding, &[], &p), 0.0);
        // A curfew from nine at night to five in the morning covers the long days' early light
        // at 50 degrees; one from eleven to four, barely.
        let late = daylight_covered((21, 5), 50.0);
        let short = daylight_covered((23, 4), 50.0);
        assert!(late > 45.0 && short < late, "{late} {short}");
        let limit = LimitDef {
            predicate: Predicate::DaylightWork,
            bound: 30.0,
            change: PassRule::TwoThirds,
        };
        assert_eq!(
            limit.reading(PolicyKind::Curfew, (21, 5), 0.0, 50.0),
            Some(((late - 30.0) / 30.0).clamp(-1.0, 1.0))
        );
        assert_eq!(
            limit.reading(PolicyKind::CommonStore, (0, 0), 0.1, 50.0),
            None
        );
        let levy = LimitDef {
            predicate: Predicate::LevyShare,
            bound: 0.2,
            change: PassRule::TwoThirds,
        };
        let over = levy
            .reading(PolicyKind::CommonStore, (0, 0), 0.3, 50.0)
            .expect("a levy is read");
        assert!((over - 0.5).abs() < 1e-9, "{over}");
        assert!(
            levy.reading(PolicyKind::CommonStore, (0, 0), 0.1, 50.0)
                .expect("a levy is read")
                < 0.0
        );
        assert!(levy.words().contains("20 in every hundred"));
    }
}
