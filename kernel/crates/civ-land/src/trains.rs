//! Train runs on a line (M10b slice DW, second part; ADR-0034 §4; the rail brief §1.2, §1.4): a
//! run is the line's own event chain, not a person's. An engine pulls its train step by step at
//! its power over the pull the step needs, `M·g·(C + s)`, no faster than its top speed, and stalls
//! where that pull is more than its driven wheels hold. A line is single track split into block
//! sections at its passing loops: one run holds a section at a time, the others wait at the loop
//! before it, first come first served, and a run that arrives stays where it arrived until its
//! owner takes it off. Nothing is despawned. Every run departed, moved or taken off bumps
//! [`Runs::revision`].

use civ_core::PermanentId;

use crate::carting::pull_n;
use crate::rail::step_grades;

/// A train as its line sees it: its engine's power, watts, and the most pull its driven wheels
/// hold, newtons; its top speed, metres a second; everything moving, engine included, kilograms;
/// and its rails' rolling resistance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Train {
    pub power_w: f64,
    pub pull_n: f64,
    pub top_m_s: f64,
    pub mass_kg: f64,
    pub c: f64,
}

impl Train {
    /// Its speed climbing `grade`, rise over run: its power over the pull needed, no faster than
    /// its top speed; at its top speed where the step needs no pull; `None` where the pull is
    /// more than its wheels hold.
    pub fn speed(&self, grade: f64) -> Option<f64> {
        let f = pull_n(self.mass_kg, self.c, grade);
        if f > self.pull_n {
            return None;
        }
        if f <= 0.0 {
            return Some(self.top_m_s);
        }
        Some((self.power_w / f).min(self.top_m_s))
    }

    /// The most mass, engine included, kilograms, that a pull of `pull_n_held` moves up a rise
    /// of `grade` on rails of rolling resistance `c`: `pull / (g·(C + s))`.
    pub fn most_mass(pull_n_held: f64, c: f64, grade: f64) -> f64 {
        let per_kg = pull_n(1.0, c, grade);
        if per_kg <= 0.0 {
            f64::INFINITY
        } else {
            pull_n_held / per_kg
        }
    }
}

/// The seconds `train` takes over each step of a bed (`runs` apart, as [`crate::rail::runs`]
/// gives), in the order it runs them: from the bed's first cell to its last, or from its last to
/// its first when `back`; `None` where it stalls.
pub fn step_seconds(bed: &[f32], runs: &[f64], train: &Train, back: bool) -> Option<Vec<f64>> {
    let mut steps: Vec<f64> = runs.get(1..).unwrap_or(&[]).to_vec();
    if back {
        steps.reverse();
    }
    step_grades(bed, runs, back)
        .into_iter()
        .zip(steps)
        .map(|(grade, run)| train.speed(grade).map(|v| run / v))
        .collect()
}

/// A line of `cells` cells split into block sections at its passing loops, `loops` being cell
/// indexes along it: each section's first and last cell index, in order. A loop at either end, or
/// beyond it, splits nothing.
pub fn sections(loops: &[u32], cells: usize) -> Vec<(usize, usize)> {
    let last = cells.saturating_sub(1);
    let mut bounds: Vec<usize> = loops
        .iter()
        .map(|&l| l as usize)
        .filter(|&l| l > 0 && l < last)
        .collect();
    bounds.sort_unstable();
    bounds.dedup();
    let mut out = Vec::with_capacity(bounds.len() + 1);
    let mut from = 0;
    for b in bounds {
        out.push((from, b));
        from = b;
    }
    out.push((from, last));
    out
}

/// The legs of a run over a line split as [`sections`] gives, in the order run: each section's
/// index and the whole minutes its steps take (`step_secs` in the order run, as
/// [`step_seconds`] gives), at least one.
pub fn legs(step_secs: &[f64], sections: &[(usize, usize)], back: bool) -> Vec<(u16, u32)> {
    let n = step_secs.len();
    let mut out: Vec<(u16, u32)> = sections
        .iter()
        .enumerate()
        .map(|(s, &(a, b))| {
            // Forward, step i runs from cell i to cell i + 1; back, the forward step i is the
            // (n - 1 - i)-th run.
            let secs: f64 = (a..b.min(n))
                .map(|i| {
                    let k = if back { n - 1 - i } else { i };
                    step_secs.get(k).copied().unwrap_or(0.0)
                })
                .sum();
            (s as u16, ((secs / 60.0).ceil() as u32).max(1))
        })
        .collect();
    if back {
        out.reverse();
    }
    out
}

/// Where a run is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    /// At the loop or end before its next leg since minute `since`, waiting for that section.
    Waiting { since: i64 },
    /// In its leg's section until minute `until`.
    Running { until: i64 },
    /// At the end of its last leg since minute `at`.
    Arrived { at: i64 },
}

/// One run.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub id: PermanentId,
    /// The line it runs on, and whether from its last cell to its first.
    pub way: PermanentId,
    pub back: bool,
    /// Its engine (a rail vehicle's record) and the household it runs for.
    pub engine: PermanentId,
    pub owner: PermanentId,
    /// The wagons it pulls (M10b slice DW, third part).
    pub cars: Vec<PermanentId>,
    /// Its legs in the order run: each section's index along the line and its minutes.
    pub legs: Vec<(u16, u32)>,
    /// The leg it is on or waiting for.
    pub leg: u16,
    pub state: RunState,
    /// The goods it carries, kilograms.
    pub load_kg: f32,
}

impl Run {
    /// The section it holds or waits for, if it has not arrived.
    pub fn section(&self) -> Option<u16> {
        match self.state {
            RunState::Arrived { .. } => None,
            _ => self.legs.get(usize::from(self.leg)).map(|l| l.0),
        }
    }

    /// Whether it holds a section now.
    pub fn running(&self) -> bool {
        matches!(self.state, RunState::Running { .. })
    }
}

/// A line's passing loops: cell indexes along it where two runs may pass.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub way: PermanentId,
    pub loops: Vec<u32>,
}

/// What happened to a run as it advanced, at a minute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEvent {
    Entered {
        run: PermanentId,
        section: u16,
        at: i64,
    },
    Left {
        run: PermanentId,
        section: u16,
        at: i64,
    },
    Arrived {
        run: PermanentId,
        at: i64,
    },
}

/// Every line's loops and every run, departed, running or arrived.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Runs {
    pub lines: Vec<Line>,
    pub list: Vec<Run>,
    revision: u32,
}

impl Runs {
    /// Runs as saved.
    pub fn from_saved(lines: Vec<Line>, list: Vec<Run>, revision: u32) -> Runs {
        Runs {
            lines,
            list,
            revision,
        }
    }

    /// Changes whenever a loop is laid or a run departs, moves or is taken off; 0 while there
    /// has never been one.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// Notes a change.
    pub fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    /// Line `way`'s passing loops, cell indexes along it, ascending.
    pub fn loops_of(&self, way: PermanentId) -> &[u32] {
        self.lines
            .iter()
            .find(|l| l.way == way)
            .map_or(&[], |l| l.loops.as_slice())
    }

    /// Lays a passing loop at cell index `at` along line `way`.
    pub fn add_loop(&mut self, way: PermanentId, at: u32) {
        let i = match self.lines.iter().position(|l| l.way == way) {
            Some(i) => i,
            None => {
                self.lines.push(Line {
                    way,
                    loops: Vec::new(),
                });
                self.lines.len() - 1
            }
        };
        let loops = &mut self.lines[i].loops;
        if let Err(p) = loops.binary_search(&at) {
            loops.insert(p, at);
            self.changed();
        }
    }

    /// Sets `run` off from its first leg's start at minute `at`: it waits there for its first
    /// section.
    pub fn depart(&mut self, mut run: Run, at: i64) {
        run.leg = 0;
        run.state = RunState::Waiting { since: at };
        self.list.push(run);
        self.changed();
    }

    /// Takes run `id` off its line once it has arrived (its owner unloading it there); `None`
    /// where there is no such run or it has not arrived.
    pub fn take_off(&mut self, id: PermanentId) -> Option<Run> {
        let i = self
            .list
            .iter()
            .position(|r| r.id == id && matches!(r.state, RunState::Arrived { .. }))?;
        self.changed();
        Some(self.list.remove(i))
    }

    /// Whether vehicle `id` is out on a run that has not been taken off: its engine or one of
    /// its wagons.
    pub fn aboard(&self, id: PermanentId) -> bool {
        self.list
            .iter()
            .any(|r| r.engine == id || r.cars.contains(&id))
    }

    /// The run holding section `section` of line `way` now, if any.
    pub fn held_by(&self, way: PermanentId, section: u16) -> Option<PermanentId> {
        self.list
            .iter()
            .find(|r| r.way == way && r.running() && r.section() == Some(section))
            .map(|r| r.id)
    }

    /// Moves every run on to minute `now`, in the order things happen: a run ends its leg at the
    /// minute it was due and waits at the loop for its next section, or arrives; a waiting run
    /// enters its section when no other run holds it, the one waiting longest first (then the
    /// lowest id), from the minute it was free. What happened, in order.
    pub fn advance(&mut self, now: i64) -> Vec<RunEvent> {
        let mut events = Vec::new();
        // When each section was last left, this advance.
        let mut free_at: Vec<((PermanentId, u16), i64)> = Vec::new();
        loop {
            // Waiting runs enter free sections, longest waiting first.
            let mut waiting: Vec<(i64, PermanentId, usize)> = self
                .list
                .iter()
                .enumerate()
                .filter_map(|(i, r)| match r.state {
                    RunState::Waiting { since } if since <= now => Some((since, r.id, i)),
                    _ => None,
                })
                .collect();
            waiting.sort_unstable();
            for (since, id, i) in waiting {
                let r = &self.list[i];
                let Some(&(section, minutes)) = r.legs.get(usize::from(r.leg)) else {
                    continue;
                };
                let key = (r.way, section);
                if self.held_by(r.way, section).is_some() {
                    continue;
                }
                let free = free_at
                    .iter()
                    .find(|f| f.0 == key)
                    .map_or(i64::MIN, |f| f.1);
                let start = since.max(free);
                self.list[i].state = RunState::Running {
                    until: start + i64::from(minutes),
                };
                events.push(RunEvent::Entered {
                    run: id,
                    section,
                    at: start,
                });
            }
            // The next leg to end by now.
            let Some(t) = self
                .list
                .iter()
                .filter_map(|r| match r.state {
                    RunState::Running { until } if until <= now => Some(until),
                    _ => None,
                })
                .min()
            else {
                break;
            };
            let mut ending: Vec<usize> = (0..self.list.len())
                .filter(|&i| self.list[i].state == RunState::Running { until: t })
                .collect();
            ending.sort_unstable_by_key(|&i| self.list[i].id);
            for i in ending {
                let r = &mut self.list[i];
                let section = r.legs[usize::from(r.leg)].0;
                events.push(RunEvent::Left {
                    run: r.id,
                    section,
                    at: t,
                });
                let key = (r.way, section);
                match free_at.iter_mut().find(|f| f.0 == key) {
                    Some(f) => f.1 = t,
                    None => free_at.push((key, t)),
                }
                if usize::from(r.leg) + 1 >= r.legs.len() {
                    r.state = RunState::Arrived { at: t };
                    events.push(RunEvent::Arrived { run: r.id, at: t });
                } else {
                    r.leg += 1;
                    r.state = RunState::Waiting { since: t };
                }
            }
        }
        if !events.is_empty() {
            self.changed();
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    /// The core's locomotive (5 t, 7.5 kW, 9,810 N, 2.2 m/s) with 20 t of wagons and load
    /// behind it on timber rails.
    fn train() -> Train {
        Train {
            power_w: 7_500.0,
            pull_n: 9_810.0,
            top_m_s: 2.2,
            mass_kg: 25_000.0,
            c: 0.005,
        }
    }

    #[test]
    fn a_train_runs_at_its_power_over_its_pull_no_faster_than_its_top_speed() {
        let t = train();
        // Level: 1,226 N needs 6.1 m/s of its power, so it runs at its top speed.
        assert_eq!(t.speed(0.0), Some(2.2));
        // Up 1 %: 3,679 N, so 7,500 / 3,679 = 2.04 m/s.
        let up = t.speed(0.01).expect("climbs");
        assert!(
            (up - 7_500.0 / (25_000.0 * 9.81 * 0.015)).abs() < 1e-9,
            "{up}"
        );
        // Up 4 %: 11,036 N, more than its wheels hold: it stalls.
        assert_eq!(t.speed(0.04), None);
        // Down a fall steeper than its rolling resistance, at its top speed.
        assert_eq!(t.speed(-0.02), Some(2.2));
        // What its wheels hold up 1 %: 9,810 / (9.81 × 0.015) = 66.7 t.
        assert!((Train::most_mass(9_810.0, 0.005, 0.01) - 66_666.7).abs() < 1.0);
        // Over a bed, step by step: 100 m level then 100 m up 1 %.
        let secs = step_seconds(&[0.0, 0.0, 1.0], &[0.0, 100.0, 100.0], &t, false).expect("runs");
        assert!((secs[0] - 100.0 / 2.2).abs() < 1e-9 && (secs[1] - 100.0 / up).abs() < 1e-9);
        // Back, down then level, both at its top speed.
        let back = step_seconds(&[0.0, 0.0, 1.0], &[0.0, 100.0, 100.0], &t, true).expect("runs");
        assert!(
            back.iter().all(|s| (s - 100.0 / 2.2).abs() < 1e-9),
            "{back:?}"
        );
        // Up a 5 % step it stalls.
        assert_eq!(step_seconds(&[0.0, 5.0], &[0.0, 100.0], &t, false), None);
    }

    #[test]
    fn a_line_is_split_at_its_loops_and_a_run_takes_its_legs_in_order() {
        assert_eq!(sections(&[], 10), vec![(0, 9)]);
        assert_eq!(sections(&[6, 3, 0, 9, 3], 10), vec![(0, 3), (3, 6), (6, 9)]);
        // Nine steps of 50 s: legs of 150 s, three minutes each; one of 10 s still a minute.
        let mut steps = vec![50.0; 9];
        steps[0] = 10.0;
        let forward = legs(&steps, &sections(&[3, 6], 10), false);
        assert_eq!(forward, vec![(0, 2), (1, 3), (2, 3)]);
        // Back, the last section first, its steps read from the end of the run.
        steps.reverse();
        assert_eq!(
            legs(&steps, &sections(&[3, 6], 10), true),
            vec![(2, 3), (1, 3), (0, 2)]
        );
    }

    fn run(n: u64, way: PermanentId, back: bool, legs: Vec<(u16, u32)>) -> Run {
        Run {
            id: id(n),
            way,
            back,
            engine: id(100 + n),
            owner: id(200),
            cars: Vec::new(),
            legs,
            leg: 0,
            state: RunState::Arrived { at: 0 },
            load_kg: 0.0,
        }
    }

    /// No two runs ever held one section of one line at once, by the events.
    fn never_shared(events: &[RunEvent], way_of: impl Fn(PermanentId) -> PermanentId) {
        let mut holding: Vec<(PermanentId, u16, PermanentId)> = Vec::new();
        for e in events {
            match *e {
                RunEvent::Entered { run, section, .. } => {
                    let key = (way_of(run), section);
                    assert!(
                        !holding.iter().any(|h| (h.0, h.1) == key),
                        "{run:?} entered section {section} while it was held: {events:?}"
                    );
                    holding.push((key.0, key.1, run));
                }
                RunEvent::Left { run, section, .. } => {
                    let before = holding.len();
                    holding.retain(|h| !(h.2 == run && h.1 == section));
                    assert_eq!(holding.len() + 1, before, "left what it held");
                }
                RunEvent::Arrived { .. } => {}
            }
        }
    }

    #[test]
    fn two_runs_meet_at_a_loop_on_single_track_and_neither_is_despawned() {
        let way = id(50);
        let mut runs = Runs::default();
        runs.add_loop(way, 3);
        runs.add_loop(way, 6);
        assert_eq!(runs.loops_of(way), &[3, 6]);
        // One each way over three sections of ten minutes, both setting off at minute 0.
        let ten = |s: [u16; 3]| s.iter().map(|&s| (s, 10)).collect::<Vec<_>>();
        runs.depart(run(1, way, false, ten([0, 1, 2])), 0);
        runs.depart(run(2, way, true, ten([2, 1, 0])), 0);
        let mut events = runs.advance(5);
        assert_eq!(runs.held_by(way, 0), Some(id(1)));
        assert_eq!(runs.held_by(way, 2), Some(id(2)));
        // At minute 10 both reach the loops either side of the middle section; the lower id
        // enters it and the other waits at its loop until it is clear.
        events.extend(runs.advance(15));
        assert_eq!(runs.held_by(way, 1), Some(id(1)));
        assert_eq!(runs.list[1].state, RunState::Waiting { since: 10 });
        events.extend(runs.advance(1_000));
        never_shared(&events, |_| way);
        assert!(events.contains(&RunEvent::Entered {
            run: id(2),
            section: 1,
            at: 20
        }));
        assert_eq!(runs.list[0].state, RunState::Arrived { at: 30 });
        assert_eq!(runs.list[1].state, RunState::Arrived { at: 40 });
        // Nothing despawned: both stay where they arrived until taken off.
        assert_eq!(runs.list.len(), 2);
        assert!(runs.take_off(id(1)).is_some());
        assert_eq!(runs.list.len(), 1);
        // Advancing again changes nothing.
        let rev = runs.revision();
        assert!(runs.advance(2_000).is_empty());
        assert_eq!(runs.revision(), rev);
    }

    #[test]
    fn a_run_waits_at_the_loop_while_its_section_is_held_and_enters_the_minute_it_clears() {
        let way = id(60);
        let mut runs = Runs::default();
        runs.add_loop(way, 4);
        // A slow run holds the second section for 300 minutes from minute 0.
        runs.depart(run(1, way, false, vec![(1, 300)]), 0);
        // Another sets off behind it at minute 2 and reaches the loop at 12.
        runs.depart(run(2, way, false, vec![(0, 10), (1, 10)]), 2);
        let mut events = runs.advance(200);
        assert_eq!(runs.list[1].state, RunState::Waiting { since: 12 });
        assert_eq!(runs.held_by(way, 1), Some(id(1)));
        events.extend(runs.advance(400));
        never_shared(&events, |_| way);
        assert!(events.contains(&RunEvent::Entered {
            run: id(2),
            section: 1,
            at: 300
        }));
        assert_eq!(runs.list[1].state, RunState::Arrived { at: 310 });
        // A run on another line is never held up by it.
        let other = id(61);
        runs.depart(run(3, other, false, vec![(1, 5)]), 400);
        runs.depart(run(4, way, false, vec![(1, 5)]), 400);
        let events = runs.advance(400);
        assert_eq!(events.len(), 2, "{events:?}");
    }
}
