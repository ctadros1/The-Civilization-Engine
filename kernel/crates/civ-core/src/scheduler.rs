//! The event-and-cadence scheduler.
//!
//! Agents are **event-scheduled** (plan §4.4): a tick only touches work due in that minute, and
//! most agents cost nothing in most ticks. Periodic systems (hourly dispatch, daily economy,
//! yearly demography) subscribe to **cadence** boundaries instead of polling the clock.
//!
//! Ordering at one instant, as documented in research 01-02 §1.3 and §4.2:
//!
//! 1. Cadence boundaries, finest first (minute, hour, day, week, month, season, year). A boundary
//!    at time `t` means "the period ending at `t` is complete", so a coarser summary sees the
//!    results of the finer periods that close with it.
//! 2. Events, by phase, then by a random tie-break drawn when the event was scheduled, then by
//!    scheduling order. The random tie-break exists so that equal claims are not won by whoever
//!    was scheduled first or has the lowest id; scheduling order only breaks exact ties of the
//!    64-bit draw.
//!
//! Handlers schedule follow-ups through [`Followups`]. A follow-up may land at the current
//! instant; a run of more than the configured number of events at one instant is reported as a
//! zero-time loop rather than spinning forever.
//!
//! The scheduler does not know about entities. Stale events (an actor that died, an activity that
//! was abandoned) are recognised by the handler from the generation and activity version the
//! event carries (research 01-02 §1.3).

use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::fmt;

use crate::rng::Rng64;
use crate::time::{
    DAYS_PER_WEEK, MINUTES_PER_DAY, MINUTES_PER_HOUR, MINUTES_PER_YEAR, MONTH_STARTS, SimTime,
};

/// A periodic boundary a system can subscribe to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Cadence {
    /// Every minute.
    Minute,
    /// On the hour.
    Hour,
    /// At midnight.
    Day,
    /// Every seventh midnight, counted from the calendar origin.
    Week,
    /// At the first midnight of each month.
    Month,
    /// At the first midnight of months 3, 6, 9 and 12.
    Season,
    /// At the first midnight of each year.
    Year,
}

impl Cadence {
    /// Every cadence, finest first: the order boundaries at one instant are delivered in.
    pub const ALL: [Cadence; 7] = [
        Cadence::Minute,
        Cadence::Hour,
        Cadence::Day,
        Cadence::Week,
        Cadence::Month,
        Cadence::Season,
        Cadence::Year,
    ];

    /// The first boundary strictly after `t`.
    pub fn next_boundary(self, t: SimTime) -> SimTime {
        let m = t.minutes();
        let next_multiple = |period: i64| (m.div_euclid(period) + 1) * period;
        let minutes = match self {
            Cadence::Minute => m + 1,
            Cadence::Hour => next_multiple(MINUTES_PER_HOUR),
            Cadence::Day => next_multiple(MINUTES_PER_DAY),
            Cadence::Week => next_multiple(MINUTES_PER_DAY * DAYS_PER_WEEK),
            Cadence::Year => next_multiple(MINUTES_PER_YEAR),
            Cadence::Month => next_month_start(m, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
            Cadence::Season => next_month_start(m, &[2, 5, 8, 11]),
        };
        SimTime::from_minutes(minutes)
    }

    /// Whether `t` is a boundary of this cadence.
    pub fn is_boundary(self, t: SimTime) -> bool {
        self.next_boundary(t.plus_minutes(-1)) == t
    }
}

/// The first start of one of `months` (0-based) strictly after minute `m`.
fn next_month_start(m: i64, months: &[usize]) -> i64 {
    let year_start = m.div_euclid(MINUTES_PER_YEAR) * MINUTES_PER_YEAR;
    for &month in months {
        let start = year_start + MONTH_STARTS[month] * MINUTES_PER_DAY;
        if start > m {
            return start;
        }
    }
    year_start + MINUTES_PER_YEAR + MONTH_STARTS[months[0]] * MINUTES_PER_DAY
}

/// Something due at the current instant.
#[derive(Debug, PartialEq, Eq)]
pub enum Due<E> {
    /// A cadence boundary was reached.
    Cadence {
        /// Which cadence.
        cadence: Cadence,
        /// The boundary time.
        at: SimTime,
    },
    /// A scheduled event came due.
    Event {
        /// When it was due.
        at: SimTime,
        /// Its phase.
        phase: u8,
        /// The event.
        event: E,
    },
}

/// Why the scheduler refused or stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    /// An event was scheduled before the current time.
    InThePast {
        /// Requested time.
        at: SimTime,
        /// Current time.
        now: SimTime,
    },
    /// Too many events ran at one instant: almost certainly handlers rescheduling each other at
    /// the same time forever.
    ZeroTimeLoop {
        /// The instant.
        at: SimTime,
        /// The limit that was hit.
        limit: usize,
    },
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleError::InThePast { at, now } => write!(
                f,
                "event scheduled at minute {} but the clock is at minute {}",
                at.minutes(),
                now.minutes()
            ),
            ScheduleError::ZeroTimeLoop { at, limit } => write!(
                f,
                "more than {limit} events at minute {}: handlers are rescheduling at the same instant",
                at.minutes()
            ),
        }
    }
}

impl std::error::Error for ScheduleError {}

/// Counters from one [`Scheduler::advance_to`] call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdvanceStats {
    /// Events delivered.
    pub events: u64,
    /// Cadence boundaries delivered.
    pub cadences: u64,
}

/// A pending event as stored, for saving and restoring the queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingEvent<E> {
    /// When it is due.
    pub at: SimTime,
    /// Its phase.
    pub phase: u8,
    /// The random tie-break drawn when it was scheduled.
    pub tiebreak: u64,
    /// Scheduling order.
    pub seq: u64,
    /// The event.
    pub event: E,
}

struct Entry<E>(PendingEvent<E>);

impl<E> Entry<E> {
    fn key(&self) -> (SimTime, u8, u64, u64) {
        (self.0.at, self.0.phase, self.0.tiebreak, self.0.seq)
    }
}

impl<E> PartialEq for Entry<E> {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl<E> Eq for Entry<E> {}

impl<E> PartialOrd for Entry<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for Entry<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

/// Follow-up events scheduled by a handler while it runs.
#[derive(Debug)]
pub struct Followups<E> {
    now: SimTime,
    items: Vec<(SimTime, u8, E)>,
}

impl<E> Followups<E> {
    /// The instant being processed.
    pub fn now(&self) -> SimTime {
        self.now
    }

    /// Schedules an event at `at` (not before the current instant) in `phase`.
    pub fn schedule(&mut self, at: SimTime, phase: u8, event: E) -> Result<(), ScheduleError> {
        if at < self.now {
            return Err(ScheduleError::InThePast { at, now: self.now });
        }
        self.items.push((at, phase, event));
        Ok(())
    }
}

/// Delivers cadence boundaries and scheduled events in time order.
pub struct Scheduler<E> {
    now: SimTime,
    queue: BinaryHeap<Reverse<Entry<E>>>,
    seq: u64,
    tiebreak: Rng64,
    cadences: Vec<(Cadence, SimTime)>,
    max_events_per_instant: usize,
}

impl<E> fmt::Debug for Scheduler<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scheduler")
            .field("now", &self.now)
            .field("pending", &self.queue.len())
            .field("cadences", &self.cadences)
            .finish_non_exhaustive()
    }
}

impl<E> Scheduler<E> {
    /// A scheduler whose clock reads `now`. `tiebreak_seed` seeds the tie-break stream.
    pub fn new(now: SimTime, tiebreak_seed: u64) -> Self {
        Scheduler {
            now,
            queue: BinaryHeap::new(),
            seq: 0,
            tiebreak: Rng64::seed_from_u64(tiebreak_seed),
            cadences: Vec::new(),
            max_events_per_instant: 1_000_000,
        }
    }

    /// The current time.
    pub fn now(&self) -> SimTime {
        self.now
    }

    /// Number of pending events.
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Changes the zero-time-loop limit.
    pub fn set_max_events_per_instant(&mut self, limit: usize) {
        self.max_events_per_instant = limit.max(1);
    }

    /// Starts delivering boundaries of `cadence`. Subscribing twice has no extra effect.
    pub fn subscribe(&mut self, cadence: Cadence) {
        if !self.cadences.iter().any(|(c, _)| *c == cadence) {
            self.cadences
                .push((cadence, cadence.next_boundary(self.now)));
            self.cadences.sort_by_key(|(c, _)| *c);
        }
    }

    /// Schedules `event` at `at` in `phase`. Refuses times before the current time.
    pub fn schedule(&mut self, at: SimTime, phase: u8, event: E) -> Result<(), ScheduleError> {
        if at < self.now {
            return Err(ScheduleError::InThePast { at, now: self.now });
        }
        let pending = PendingEvent {
            at,
            phase,
            tiebreak: self.tiebreak.next_u64(),
            seq: self.seq,
            event,
        };
        self.seq += 1;
        self.queue.push(Reverse(Entry(pending)));
        Ok(())
    }

    /// The time of the next boundary or event, if any.
    pub fn next_due(&self) -> Option<SimTime> {
        let event = self.queue.peek().map(|Reverse(e)| e.0.at);
        let cadence = self.cadences.iter().map(|(_, at)| *at).min();
        match (event, cadence) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    /// Delivers everything due up to and including `target`, in order, then sets the clock to
    /// `target` (if it is later than the current time).
    pub fn advance_to<F>(
        &mut self,
        target: SimTime,
        handle: F,
    ) -> Result<AdvanceStats, ScheduleError>
    where
        F: FnMut(Due<E>, &mut Followups<E>),
    {
        self.advance(target, true, handle)
    }

    /// Delivers everything due before `target` and the cadence boundaries at `target`, in order,
    /// then sets the clock to `target`. The events due at `target` stay queued, delivered first by
    /// the next advance, in the order they would have been: cut so, an advance gives the same as
    /// one uncut, and a clock stopped at a boundary stands after its cadences and before its
    /// events.
    pub fn advance_before_events<F>(
        &mut self,
        target: SimTime,
        handle: F,
    ) -> Result<AdvanceStats, ScheduleError>
    where
        F: FnMut(Due<E>, &mut Followups<E>),
    {
        self.advance(target, false, handle)
    }

    fn advance<F>(
        &mut self,
        target: SimTime,
        events_at_target: bool,
        mut handle: F,
    ) -> Result<AdvanceStats, ScheduleError>
    where
        F: FnMut(Due<E>, &mut Followups<E>),
    {
        let mut stats = AdvanceStats::default();
        while let Some(at) = self.next_due() {
            if at > target {
                break;
            }
            self.now = at;
            let mut followups = Followups {
                now: at,
                items: Vec::new(),
            };

            for i in 0..self.cadences.len() {
                let (cadence, due) = self.cadences[i];
                if due == at {
                    handle(Due::Cadence { cadence, at }, &mut followups);
                    self.cadences[i].1 = cadence.next_boundary(at);
                    stats.cadences += 1;
                }
            }
            self.enqueue(&mut followups);
            if at == target && !events_at_target {
                break;
            }

            let mut at_this_instant = 0usize;
            while self.queue.peek().is_some_and(|Reverse(e)| e.0.at == at) {
                let Some(Reverse(Entry(pending))) = self.queue.pop() else {
                    break;
                };
                at_this_instant += 1;
                if at_this_instant > self.max_events_per_instant {
                    return Err(ScheduleError::ZeroTimeLoop {
                        at,
                        limit: self.max_events_per_instant,
                    });
                }
                handle(
                    Due::Event {
                        at,
                        phase: pending.phase,
                        event: pending.event,
                    },
                    &mut followups,
                );
                stats.events += 1;
                self.enqueue(&mut followups);
            }
        }
        if target > self.now {
            self.now = target;
        }
        Ok(stats)
    }

    fn enqueue(&mut self, followups: &mut Followups<E>) {
        for (at, phase, event) in followups.items.drain(..) {
            // Followups::schedule already refused times in the past.
            let _ = self.schedule(at, phase, event);
        }
    }

    /// The pending events in delivery order, for saving.
    pub fn pending_events(&self) -> Vec<&PendingEvent<E>> {
        let mut events: Vec<&PendingEvent<E>> = self.queue.iter().map(|Reverse(e)| &e.0).collect();
        events.sort_by_key(|e| (e.at, e.phase, e.tiebreak, e.seq));
        events
    }

    /// Everything needed to restore this scheduler, apart from the events and subscriptions:
    /// the clock, the scheduling counter and the tie-break stream state.
    pub fn state(&self) -> (SimTime, u64, [u64; 4]) {
        (self.now, self.seq, self.tiebreak.state())
    }

    /// Rebuilds a scheduler from saved state. Subscriptions are code, not state: subscribe again
    /// after restoring.
    pub fn restore(
        now: SimTime,
        seq: u64,
        tiebreak_state: [u64; 4],
        events: Vec<PendingEvent<E>>,
    ) -> Option<Self> {
        let tiebreak = Rng64::from_state(tiebreak_state)?;
        let max_seq = events.iter().map(|e| e.seq + 1).max().unwrap_or(0);
        if events.iter().any(|e| e.at < now) {
            return None;
        }
        Some(Scheduler {
            now,
            queue: events.into_iter().map(|e| Reverse(Entry(e))).collect(),
            seq: seq.max(max_seq),
            tiebreak,
            cadences: Vec::new(),
            max_events_per_instant: 1_000_000,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::{DAYS_PER_YEAR, MINUTES_PER_YEAR};

    fn collect(s: &mut Scheduler<u32>, target: SimTime) -> Vec<(i64, String)> {
        let mut seen = Vec::new();
        s.advance_to(target, |due, _| match due {
            Due::Cadence { cadence, at } => seen.push((at.minutes(), format!("{cadence:?}"))),
            Due::Event { at, phase, event } => {
                seen.push((at.minutes(), format!("e{event}p{phase}")));
            }
        })
        .expect("no loop");
        seen
    }

    #[test]
    fn events_come_in_time_then_phase_order() {
        let mut s = Scheduler::new(SimTime::ZERO, 1);
        s.schedule(SimTime::from_minutes(10), 2, 1).expect("ok");
        s.schedule(SimTime::from_minutes(5), 0, 2).expect("ok");
        s.schedule(SimTime::from_minutes(10), 0, 3).expect("ok");
        let seen = collect(&mut s, SimTime::from_minutes(20));
        assert_eq!(
            seen,
            vec![
                (5, "e2p0".to_owned()),
                (10, "e3p0".to_owned()),
                (10, "e1p2".to_owned())
            ]
        );
        assert_eq!(s.now(), SimTime::from_minutes(20));
    }

    #[test]
    fn scheduling_in_the_past_is_refused() {
        let mut s: Scheduler<u32> = Scheduler::new(SimTime::from_minutes(100), 1);
        assert!(matches!(
            s.schedule(SimTime::from_minutes(99), 0, 1),
            Err(ScheduleError::InThePast { .. })
        ));
    }

    #[test]
    fn a_year_of_boundaries_has_the_right_counts() {
        let mut s: Scheduler<u32> = Scheduler::new(SimTime::ZERO, 1);
        for c in [
            Cadence::Day,
            Cadence::Week,
            Cadence::Month,
            Cadence::Season,
            Cadence::Year,
        ] {
            s.subscribe(c);
        }
        let seen = collect(&mut s, SimTime::from_minutes(MINUTES_PER_YEAR));
        let count = |name: &str| seen.iter().filter(|(_, n)| n == name).count();
        assert_eq!(count("Day"), DAYS_PER_YEAR as usize);
        assert_eq!(count("Week"), 52);
        assert_eq!(count("Month"), 12);
        assert_eq!(count("Season"), 4);
        assert_eq!(count("Year"), 1);
    }

    #[test]
    fn coincident_boundaries_arrive_finest_first() {
        let mut s: Scheduler<u32> = Scheduler::new(SimTime::from_minutes(MINUTES_PER_YEAR - 1), 1);
        for c in [Cadence::Year, Cadence::Day, Cadence::Month, Cadence::Hour] {
            s.subscribe(c);
        }
        let seen = collect(&mut s, SimTime::from_minutes(MINUTES_PER_YEAR));
        let names: Vec<&str> = seen.iter().map(|(_, n)| n.as_str()).collect();
        assert_eq!(names, vec!["Hour", "Day", "Month", "Year"]);
    }

    #[test]
    fn month_and_season_boundaries_land_on_month_starts() {
        let feb1 = SimTime::from_date(1, 2, 1, 0, 0).expect("valid");
        assert_eq!(Cadence::Month.next_boundary(SimTime::ZERO), feb1);
        let mar1 = SimTime::from_date(1, 3, 1, 0, 0).expect("valid");
        assert_eq!(Cadence::Season.next_boundary(SimTime::ZERO), mar1);
        let dec1 = SimTime::from_date(1, 12, 1, 0, 0).expect("valid");
        let next_mar1 = SimTime::from_date(2, 3, 1, 0, 0).expect("valid");
        assert_eq!(Cadence::Season.next_boundary(dec1), next_mar1);
        assert!(Cadence::Month.is_boundary(feb1));
        assert!(!Cadence::Month.is_boundary(feb1.plus_minutes(1)));
    }

    #[test]
    fn advancing_in_two_steps_equals_one_step() {
        // Metamorphic check (research 01-11 §1.4): 0→100 equals 0→40→100.
        let build = || {
            let mut s = Scheduler::new(SimTime::ZERO, 7);
            s.subscribe(Cadence::Hour);
            for (i, t) in [3, 40, 40, 41, 77, 99].into_iter().enumerate() {
                s.schedule(SimTime::from_minutes(t), (i % 2) as u8, i as u32)
                    .expect("ok");
            }
            s
        };
        let mut one = build();
        let all = collect(&mut one, SimTime::from_minutes(100));
        let mut two = build();
        let mut split = collect(&mut two, SimTime::from_minutes(40));
        split.extend(collect(&mut two, SimTime::from_minutes(100)));
        assert_eq!(all, split);
    }

    #[test]
    fn stopping_before_a_boundarys_events_delivers_them_next_in_the_same_order() {
        // A stop at 60 delivers the hour's boundary and leaves the two events of minute 60 for
        // the next advance, which delivers them first: the whole is as one uncut advance.
        let build = || {
            let mut s = Scheduler::new(SimTime::ZERO, 7);
            s.subscribe(Cadence::Hour);
            for (i, t) in [30, 60, 60, 61, 90].into_iter().enumerate() {
                s.schedule(SimTime::from_minutes(t), (i % 2) as u8, i as u32)
                    .expect("ok");
            }
            s
        };
        let mut one = build();
        let all = collect(&mut one, SimTime::from_minutes(120));
        let mut two = build();
        let mut split = Vec::new();
        two.advance_before_events(SimTime::from_minutes(60), |due, _| match due {
            Due::Cadence { cadence, at } => split.push((at.minutes(), format!("{cadence:?}"))),
            Due::Event { at, phase, event } => {
                split.push((at.minutes(), format!("e{event}p{phase}")));
            }
        })
        .expect("no loop");
        assert_eq!(two.now(), SimTime::from_minutes(60));
        assert_eq!(
            split.last().map(|(t, n)| (*t, n.as_str())),
            Some((60, "Hour")),
            "the boundary, not its events: {split:?}"
        );
        assert_eq!(two.next_due(), Some(SimTime::from_minutes(60)));
        split.extend(collect(&mut two, SimTime::from_minutes(120)));
        assert_eq!(all, split);
    }

    #[test]
    fn followups_at_the_same_instant_run_in_the_same_pass() {
        let mut s = Scheduler::new(SimTime::ZERO, 1);
        s.schedule(SimTime::from_minutes(5), 0, 0u32).expect("ok");
        let mut seen = Vec::new();
        s.advance_to(SimTime::from_minutes(5), |due, f| {
            if let Due::Event { event, .. } = due {
                seen.push(event);
                if event < 3 {
                    f.schedule(f.now(), 1, event + 1).expect("ok");
                }
            }
        })
        .expect("no loop");
        assert_eq!(seen, vec![0, 1, 2, 3]);
    }

    #[test]
    fn zero_time_loops_are_reported() {
        let mut s = Scheduler::new(SimTime::ZERO, 1);
        s.set_max_events_per_instant(50);
        s.schedule(SimTime::from_minutes(1), 0, 0u32).expect("ok");
        let result = s.advance_to(SimTime::from_minutes(2), |_, f| {
            f.schedule(f.now(), 0, 0).expect("ok");
        });
        assert!(matches!(result, Err(ScheduleError::ZeroTimeLoop { .. })));
    }

    #[test]
    fn equal_events_are_not_ordered_by_scheduling_order() {
        // Across many seeds, the first-scheduled of two identical-time events must sometimes
        // lose: the tie-break is random, not first-come.
        let mut first_wins = 0;
        for seed in 0..64 {
            let mut s = Scheduler::new(SimTime::ZERO, seed);
            s.schedule(SimTime::from_minutes(1), 0, 1u32).expect("ok");
            s.schedule(SimTime::from_minutes(1), 0, 2u32).expect("ok");
            let seen = collect(&mut s, SimTime::from_minutes(1));
            if seen[0].1 == "e1p0" {
                first_wins += 1;
            }
        }
        assert!((10..=54).contains(&first_wins), "first won {first_wins}/64");
    }

    #[test]
    fn state_round_trips_through_restore() {
        let mut s = Scheduler::new(SimTime::ZERO, 3);
        for t in [5, 9, 9, 30] {
            s.schedule(SimTime::from_minutes(t), 0, t as u32)
                .expect("ok");
        }
        s.advance_to(SimTime::from_minutes(6), |_, _| {})
            .expect("ok");
        let (now, seq, rng) = s.state();
        let events: Vec<PendingEvent<u32>> = s.pending_events().into_iter().cloned().collect();
        let mut restored = Scheduler::restore(now, seq, rng, events).expect("valid");
        assert_eq!(
            collect(&mut s, SimTime::from_minutes(50)),
            collect(&mut restored, SimTime::from_minutes(50))
        );
    }
}
