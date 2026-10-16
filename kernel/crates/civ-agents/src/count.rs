//! Counting an election exactly (M7a slice BN, ADR-0024 §3; research 09-08 §1.7: implement
//! "tie-breaking, exhausted ballots, rounding, thresholds" explicitly, and keep ballots, counted
//! totals and announced results apart). Pure functions over ballots of candidates' ids: no world
//! state, no weights. A ballot names candidates; who may stand and who may vote are the caller's.

use civ_core::PermanentId;
use civ_core::Rng64;

/// How a tie for the last seat is settled: the count fails, or a keyed lot draws among those
/// tied (ADR-0011 §3: a draw keyed by what it decides, so the same count draws the same).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TieRule {
    /// No one is chosen for a seat whose candidates tie.
    Fail,
    /// A lot keyed by this value draws among those tied.
    Lot(u64),
}

/// The totals of a count, each candidate's votes, most first and ties by id; ballots that
/// named nobody eligible are counted apart.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub votes: Vec<(PermanentId, u32)>,
    /// Ballots that named no candidate (blank or naming only those who do not stand).
    pub empty: u32,
}

/// What a count chose: the winners in seat order, and the seats it could not fill for a tie the
/// rule would not break.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Result {
    pub totals: Totals,
    pub winners: Vec<PermanentId>,
    pub unfilled: u32,
    /// Whether a lot decided any seat.
    pub by_lot: bool,
}

/// Tallies `ballots`, each naming candidates, counting each candidate once per ballot and only
/// candidates in `standing` (sorted). A ballot that names none of them is empty.
fn tally(ballots: &[Vec<PermanentId>], standing: &[PermanentId]) -> Totals {
    let mut votes: Vec<(PermanentId, u32)> = standing.iter().map(|&c| (c, 0)).collect();
    let mut empty = 0;
    for b in ballots {
        let mut named: Vec<PermanentId> = b
            .iter()
            .copied()
            .filter(|c| standing.binary_search(c).is_ok())
            .collect();
        named.sort_unstable();
        named.dedup();
        if named.is_empty() {
            empty += 1;
        }
        for c in named {
            if let Ok(i) = votes.binary_search_by_key(&c, |v| v.0) {
                votes[i].1 += 1;
            }
        }
    }
    votes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Totals { votes, empty }
}

/// The top `seats` of `totals`, with a tie at the edge settled by `tie`. Candidates with no
/// votes are never chosen.
fn choose(totals: Totals, seats: u32, tie: TieRule) -> Result {
    let seats = seats as usize;
    let voted: Vec<(PermanentId, u32)> = totals.votes.iter().copied().filter(|v| v.1 > 0).collect();
    if voted.len() <= seats {
        let winners: Vec<PermanentId> = voted.iter().map(|v| v.0).collect();
        let unfilled = (seats - winners.len()) as u32;
        return Result {
            totals,
            winners,
            unfilled,
            by_lot: false,
        };
    }
    // Those surely in: more votes than the last seat's count.
    let edge = voted[seats - 1].1;
    let sure: Vec<PermanentId> = voted.iter().filter(|v| v.1 > edge).map(|v| v.0).collect();
    let tied: Vec<PermanentId> = voted.iter().filter(|v| v.1 == edge).map(|v| v.0).collect();
    let open = seats - sure.len();
    let mut winners = sure;
    let mut by_lot = false;
    if tied.len() <= open {
        winners.extend(tied);
    } else {
        match tie {
            TieRule::Fail => {}
            TieRule::Lot(key) => {
                winners.extend(draw(&tied, open as u32, key));
                by_lot = true;
            }
        }
    }
    let unfilled = (seats - winners.len()) as u32;
    Result {
        totals,
        winners,
        unfilled,
        by_lot,
    }
}

/// The keyed draw's purpose for a lot.
pub const PURPOSE_LOT: u64 = 0x6c6f_7464_7261_7731; // "lotdraw1"

/// One seat by plurality: each ballot's candidate counts; the most votes wins.
pub fn plurality(ballots: &[PermanentId], standing: &[PermanentId], tie: TieRule) -> Result {
    single_vote(ballots, standing, 1, tie)
}

/// `seats` seats by the block vote: each ballot names up to `seats` different candidates (a name
/// repeated counts once; any beyond the first `seats` different names are not counted), and the
/// `seats` with the most votes win.
pub fn block_vote(
    ballots: &[Vec<PermanentId>],
    standing: &[PermanentId],
    seats: u32,
    tie: TieRule,
) -> Result {
    let ballots: Vec<Vec<PermanentId>> = ballots
        .iter()
        .map(|b| {
            let mut named: Vec<PermanentId> = Vec::new();
            for &c in b {
                if !named.contains(&c) {
                    named.push(c);
                }
            }
            named.truncate(seats as usize);
            named
        })
        .collect();
    choose(tally(&ballots, standing), seats, tie)
}

/// `seats` seats by the single vote: each ballot names one candidate, and the `seats` with the
/// most votes win.
pub fn single_vote(
    ballots: &[PermanentId],
    standing: &[PermanentId],
    seats: u32,
    tie: TieRule,
) -> Result {
    let ballots: Vec<Vec<PermanentId>> = ballots.iter().map(|&b| vec![b]).collect();
    choose(tally(&ballots, standing), seats, tie)
}

/// Draws `seats` of `pool` by lot, keyed by `key` (ADR-0024 §2: an office or seat filled by lot
/// from those who put their names in), in draw order. Fewer if the pool is smaller.
pub fn draw(pool: &[PermanentId], seats: u32, key: u64) -> Vec<PermanentId> {
    let mut pool: Vec<PermanentId> = pool.to_vec();
    pool.sort_unstable();
    pool.dedup();
    let mut rng = Rng64::from_key(&[key, PURPOSE_LOT]);
    let mut out = Vec::new();
    for _ in 0..seats {
        if pool.is_empty() {
            break;
        }
        let i = (rng.next_f64() * pool.len() as f64) as usize;
        out.push(pool.remove(i.min(pool.len() - 1)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("nonzero")
    }

    /// 09-08 §4's case: 42, 33 and 25 for A, B and C; plurality gives A one seat.
    #[test]
    fn plurality_gives_the_seat_to_the_most_votes() {
        let (a, b, c) = (pid(1), pid(2), pid(3));
        let mut ballots = Vec::new();
        ballots.extend(std::iter::repeat_n(a, 42));
        ballots.extend(std::iter::repeat_n(b, 33));
        ballots.extend(std::iter::repeat_n(c, 25));
        let r = plurality(&ballots, &[a, b, c], TieRule::Fail);
        assert_eq!(r.winners, vec![a]);
        assert_eq!(r.totals.votes, vec![(a, 42), (b, 33), (c, 25)]);
        assert_eq!((r.unfilled, r.by_lot, r.totals.empty), (0, false, 0));
    }

    /// The same ballots for two seats: the single vote seats A and B; the block vote, where each
    /// of A's voters also names C, seats C and A.
    #[test]
    fn two_seats_by_single_and_block_vote() {
        let (a, b, c) = (pid(1), pid(2), pid(3));
        let mut single = Vec::new();
        single.extend(std::iter::repeat_n(a, 42));
        single.extend(std::iter::repeat_n(b, 33));
        single.extend(std::iter::repeat_n(c, 25));
        let r = single_vote(&single, &[a, b, c], 2, TieRule::Fail);
        assert_eq!(r.winners, vec![a, b]);
        let mut block = Vec::new();
        block.extend(std::iter::repeat_n(vec![a, c], 42));
        block.extend(std::iter::repeat_n(vec![b], 33));
        block.extend(std::iter::repeat_n(vec![c], 25));
        let r = block_vote(&block, &[a, b, c], 2, TieRule::Fail);
        assert_eq!(r.totals.votes, vec![(c, 67), (a, 42), (b, 33)]);
        assert_eq!(r.winners, vec![c, a]);
    }

    /// A ballot names a candidate once however often it repeats them, ignores those who do not
    /// stand, and is empty if it names nobody standing; a block ballot counts only its first
    /// `seats` names.
    #[test]
    fn ballots_count_each_candidate_once_and_only_those_standing() {
        let (a, b, c, d) = (pid(1), pid(2), pid(3), pid(4));
        let ballots = vec![vec![a, a, b], vec![d], vec![], vec![c, b, a]];
        let r = block_vote(&ballots, &[a, b, c], 2, TieRule::Fail);
        assert_eq!(r.totals.votes, vec![(b, 2), (a, 1), (c, 1)]);
        assert_eq!(r.totals.empty, 2);
        assert_eq!(r.winners[0], b);
        // a and c tie for the last seat; the rule fails it.
        assert_eq!((r.winners.len(), r.unfilled), (1, 1));
    }

    /// A tie for the last seat fails under `Fail`, and under `Lot` the same key always draws
    /// the same one of those tied, while some key draws each.
    #[test]
    fn a_tie_fails_or_is_drawn_by_a_keyed_lot() {
        let (a, b) = (pid(1), pid(2));
        let ballots = vec![a, b];
        let failed = plurality(&ballots, &[a, b], TieRule::Fail);
        assert!(failed.winners.is_empty() && failed.unfilled == 1);
        let drawn = |k| plurality(&ballots, &[a, b], TieRule::Lot(k));
        assert_eq!(drawn(7), drawn(7));
        assert!(drawn(7).by_lot);
        let mut seen: Vec<PermanentId> = (0..32).map(|k| drawn(k).winners[0]).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen, vec![a, b], "the lot can fall on either");
    }

    /// Nobody with no votes is seated: seats beyond those voted for stay unfilled.
    #[test]
    fn seats_beyond_the_candidates_voted_for_stay_unfilled() {
        let (a, b, c) = (pid(1), pid(2), pid(3));
        let r = single_vote(&[a, a, b], &[a, b, c], 3, TieRule::Lot(1));
        assert_eq!(r.winners, vec![a, b]);
        assert_eq!((r.unfilled, r.by_lot), (1, false));
    }

    /// A lot draws from the pool without repeats, the same for the same key, and no more than
    /// the pool holds.
    #[test]
    fn a_draw_is_keyed_and_never_repeats() {
        let pool: Vec<PermanentId> = (1..=5).map(pid).collect();
        let d = draw(&pool, 3, 11);
        assert_eq!(d, draw(&pool, 3, 11));
        let mut s = d.clone();
        s.sort_unstable();
        s.dedup();
        assert_eq!(s.len(), 3);
        assert_eq!(draw(&pool, 9, 11).len(), 5);
        assert!(draw(&[], 2, 11).is_empty());
    }
}
