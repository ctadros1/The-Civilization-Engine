//! Standing, influence and notables (ADR-0014 §3-4): what a settlement's adults think of one
//! another, summed from their ties once a month. Standing is always someone's view summed: there
//! is no global score. Notables are a compute tier: who considers institutional moves weekly. The
//! flag grants nothing and no choice reads it (research 04-11 §5.7).

use civ_core::PermanentId;

use crate::ties::{DOMAINS, Domain, TieParams, Ties};

/// How standing and notables are worked out (the people profile's `[ties]`, content API 30).
#[derive(Clone, Debug, PartialEq)]
pub struct StandingParams {
    /// How many people each adult counts among those they esteem most (research 04-11 §2.4:
    /// 4-12 candidates considered in an ordinary decision).
    pub candidates: usize,
    /// The share of a settlement's adults who are notables (plan §4.2: about 1 %).
    pub notable_share: f64,
    /// The fewest notables a settlement has, if it has that many adults.
    pub notable_floor: usize,
    /// A notable stays one while ranked within this many times the number of notables.
    pub notable_keep: f64,
}

impl Default for StandingParams {
    fn default() -> Self {
        StandingParams {
            candidates: 8,
            notable_share: 0.01,
            notable_floor: 3,
            notable_keep: 1.5,
        }
    }
}

/// One adult's standing in their settlement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standing {
    /// The adult.
    pub person: PermanentId,
    /// Their settlement.
    pub settlement: PermanentId,
    /// The esteem the settlement's other adults hold for them, by domain, summed (net good acts
    /// remembered).
    pub esteem: [f32; DOMAINS],
    /// How many of the settlement's adults count them among those they esteem most.
    pub influence: u32,
    /// Whether they are a notable (a compute tier only).
    pub notable: bool,
}

impl Standing {
    /// Esteem in every domain together.
    pub fn total(&self) -> f64 {
        self.esteem.iter().map(|&e| f64::from(e)).sum()
    }

    /// Esteem in `domain`.
    pub fn in_domain(&self, domain: Domain) -> f64 {
        f64::from(self.esteem[domain.index()])
    }
}

/// Every settlement's standing as worked out on `day`, by settlement and then person.
#[derive(Clone, Debug, PartialEq)]
pub struct StandingTable {
    /// The day it was worked out (`i64::MIN` before the first time).
    pub day: i64,
    /// Rows, sorted by settlement and then person.
    pub rows: Vec<Standing>,
}

impl Default for StandingTable {
    fn default() -> Self {
        StandingTable::new()
    }
}

impl StandingTable {
    /// Nothing worked out yet.
    pub fn new() -> StandingTable {
        StandingTable {
            day: i64::MIN,
            rows: Vec::new(),
        }
    }

    /// `person`'s row, if they are an adult of a settlement.
    pub fn of(&self, person: PermanentId) -> Option<&Standing> {
        self.rows.iter().find(|r| r.person == person)
    }

    /// The rows of `settlement`, in person order.
    pub fn in_settlement(&self, settlement: PermanentId) -> impl Iterator<Item = &Standing> {
        self.rows.iter().filter(move |r| r.settlement == settlement)
    }

    /// Whether `person` is a notable. For the deliberation scheduler only (ADR-0014 §4).
    pub fn is_notable(&self, person: PermanentId) -> bool {
        self.of(person).is_some_and(|r| r.notable)
    }
}

/// How much a tie's holder regards the person it is of: their esteem in every domain, with
/// warmth to tell apart those esteemed alike. What decides whom an adult counts among those they
/// esteem most.
pub fn regard(tie: &crate::ties::Tie, params: &TieParams) -> f64 {
    let esteem: f64 = Domain::ALL.iter().map(|&d| tie.esteem(d, params)).sum();
    esteem + f64::from(tie.warmth)
}

/// Works out every settlement's standing on `day` from `ties`. `adults` lists each settlement's
/// adults as (settlement, person), in any order; `before` is the table it replaces, whose
/// notables stay notables while they rank within [`StandingParams::notable_keep`] times their
/// number.
pub fn derive(
    ties: &Ties,
    adults: &[(PermanentId, PermanentId)],
    day: i64,
    tp: &TieParams,
    sp: &StandingParams,
    before: &StandingTable,
) -> StandingTable {
    let mut adults = adults.to_vec();
    adults.sort_unstable();
    adults.dedup();
    let mut rows: Vec<Standing> = adults
        .iter()
        .map(|&(settlement, person)| Standing {
            person,
            settlement,
            esteem: [0.0; DOMAINS],
            influence: 0,
            notable: false,
        })
        .collect();
    // Rows are sorted by (settlement, person), so a row is found by binary search.
    let row = |rows: &[Standing], settlement: PermanentId, person: PermanentId| {
        rows.binary_search_by_key(&(settlement, person), |r| (r.settlement, r.person))
            .ok()
    };
    for &(settlement, holder) in &adults {
        let mut regarded: Vec<(f64, PermanentId)> = Vec::new();
        for t in ties.of(holder) {
            let Some(k) = row(&rows, settlement, t.to) else {
                continue;
            };
            let now = t.at(day, tp);
            for d in Domain::ALL {
                rows[k].esteem[d.index()] += now.esteem(d, tp) as f32;
            }
            let r = regard(&now, tp);
            if r > 0.0 {
                regarded.push((r, t.to));
            }
        }
        regarded.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        for &(_, p) in regarded.iter().take(sp.candidates) {
            if let Some(k) = row(&rows, settlement, p) {
                rows[k].influence += 1;
            }
        }
    }
    // Notables, settlement by settlement: the most influential, those already notables keeping
    // their place while they rank within the keep factor (hysteresis, so the set does not churn).
    let mut start = 0;
    while start < rows.len() {
        let settlement = rows[start].settlement;
        let n = rows[start..]
            .iter()
            .take_while(|r| r.settlement == settlement)
            .count();
        let end = start + n;
        let target = ((sp.notable_share * n as f64).ceil() as usize)
            .max(sp.notable_floor)
            .min(n);
        let mut ranked: Vec<usize> = (start..end).collect();
        ranked.sort_by(|&a, &b| {
            rows[b]
                .influence
                .cmp(&rows[a].influence)
                .then(rows[b].total().total_cmp(&rows[a].total()))
                .then(rows[a].person.cmp(&rows[b].person))
        });
        let keep_within = ((target as f64 * sp.notable_keep).ceil() as usize).max(target);
        let mut chosen: Vec<usize> = ranked
            .iter()
            .take(keep_within)
            .copied()
            .filter(|&k| before.is_notable(rows[k].person))
            .collect();
        for &k in &ranked {
            if chosen.len() >= target {
                break;
            }
            if !chosen.contains(&k) {
                chosen.push(k);
            }
        }
        for k in chosen {
            rows[k].notable = true;
        }
        start = end;
    }
    StandingTable { day, rows }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ties::Act;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn standing_sums_what_others_think_and_influence_counts_who_esteems_most() {
        let tp = TieParams::core();
        let sp = StandingParams {
            candidates: 1,
            notable_share: 0.0,
            notable_floor: 1,
            notable_keep: 1.5,
        };
        let s = id(100);
        let mut ties = Ties::new();
        // 1 and 2 each received two gifts from 3; 2 traded with 4 too.
        for holder in [1, 2] {
            for _ in 0..2 {
                ties.record(id(holder), id(3), Act::GiftReceived, 1.0, 1.0, 0, &tp);
            }
        }
        ties.record(id(2), id(4), Act::Traded, 1.0, 0.0, 0, &tp);
        // Company alone makes no standing.
        ties.record(id(3), id(1), Act::Hearth, 3.0, 0.0, 0, &tp);
        let adults: Vec<_> = (1..=4).map(|p| (s, id(p))).collect();
        let t = derive(&ties, &adults, 0, &tp, &sp, &StandingTable::new());
        let r3 = t.of(id(3)).expect("3 is an adult here");
        assert!((r3.in_domain(Domain::Provision) - 4.0).abs() < 1e-5);
        assert_eq!(r3.influence, 2);
        assert!(r3.notable);
        let r4 = t.of(id(4)).expect("4");
        assert!(r4.in_domain(Domain::Word) > 0.0);
        assert_eq!(r4.influence, 0, "2 esteems 3 most");
        let r1 = t.of(id(1)).expect("1");
        assert_eq!(r1.total(), 0.0, "warmth from company is not esteem");
        // Someone of another settlement's view is not counted here.
        let mut other = ties.clone();
        other.record(id(9), id(4), Act::GiftReceived, 5.0, 0.0, 0, &tp);
        let mut more = adults.clone();
        more.push((id(200), id(9)));
        let t2 = derive(&other, &more, 0, &tp, &sp, &StandingTable::new());
        assert_eq!(
            t2.of(id(4)).map(|r| r.total()),
            t.of(id(4)).map(|r| r.total())
        );
    }

    #[test]
    fn notables_keep_their_place_until_they_fall_well_behind() {
        let tp = TieParams::core();
        let sp = StandingParams {
            candidates: 1,
            notable_share: 0.0,
            notable_floor: 1,
            notable_keep: 2.0,
        };
        let s = id(100);
        let adults: Vec<_> = (1..=6).map(|p| (s, id(p))).collect();
        // 5 is esteemed by two people, 6 by one.
        let mut ties = Ties::new();
        for h in [1, 2] {
            ties.record(id(h), id(5), Act::GiftReceived, 1.0, 0.0, 0, &tp);
        }
        ties.record(id(3), id(6), Act::GiftReceived, 1.0, 0.0, 0, &tp);
        let first = derive(&ties, &adults, 0, &tp, &sp, &StandingTable::new());
        assert!(first.is_notable(id(5)) && !first.is_notable(id(6)));
        // 6 draws ahead: 5, now second, keeps its place (within twice one).
        ties.record(id(4), id(6), Act::GiftReceived, 2.0, 0.0, 0, &tp);
        ties.record(id(1), id(6), Act::GiftReceived, 3.0, 0.0, 0, &tp);
        let second = derive(&ties, &adults, 0, &tp, &sp, &first);
        assert!(second.is_notable(id(5)), "kept within the keep factor");
        assert!(!second.is_notable(id(6)));
        // From nothing, the leader is chosen.
        let fresh = derive(&ties, &adults, 0, &tp, &sp, &StandingTable::new());
        assert!(fresh.is_notable(id(6)) && !fresh.is_notable(id(5)));
    }
}
