//! A village and what it makes of law, for the M7b demo (plan §7: "a court strikes down a curfew";
//! the M7 courts brief §3: the demo names its outcome, so it shows what its world reached, nudging
//! only priors). A village founded by a band of `PEOPLE` lives as the content has them. Nothing is
//! set: no store is emptied, no law proposed, no case brought, no forum named, no limit adopted, no
//! challenge brought, and nobody sits or decides by the observer. Each step a struck-down curfew
//! needs is people's: a case brought under a law against taking, a curfew passed, a forum that may
//! review (the elders, or a hearer bound by findings), a limit adopted through the custom's
//! amendment route, a challenge by one the curfew keeps home, and the forum's finding. If a step
//! has not come by its time box, the observer nudges a prior (`NUDGE`, logged as such), never an
//! event: what bringing a case costs (`crime.report_cost`), the share of takings a household
//! believes a curfew keeps off (`crime.curfew_guard`), what an evening at the gathering costs one
//! who comes (`elections.w_evening`, the evenings a forum saves), the weight a household puts on
//! the hours a curfew keeps it home (`crime.curfew_cost_days`) and what proposing costs a sponsor
//! (`polity.propose_cost`). The world lives at most `MAX_DAYS`, and ends early if its village
//! empties; it is saved when a law is first struck down ("A law is struck down") and at the end
//! ("After the rulings"), `AFTER_DAYS` after it if one was. Nudged priors live only in this run: a
//! loaded world takes its content's. Up to `TRIES` worlds are lived, each of the next seed and its
//! own name; the one that reached most steps is kept and the others' saves are removed.
//!
//! ```sh
//! cargo run --release -p civ-sim --example courts -- <saves folder> [seed]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::crime::CaseStage;
use civ_agents::polity::{LawStatus, PolicyKind};
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// People in the founding band: the content's default size (the M7a demo's 120 empty in their
/// second winter in most draws tried, before M7 as now; see plan §9).
const PEOPLE: u32 = 40;
/// Days lived at most.
const MAX_DAYS: i64 = 1000;
/// Days lived after the first law was struck down.
const AFTER_DAYS: i64 = 60;
const DAY: i64 = 24 * 60;
const NAME: &str = "Hallowford";
/// Worlds lived at most.
const TRIES: u64 = 4;

/// The steps a struck-down curfew needs, in the order the demo's time boxes ask for them.
const STEPS: [&str; 6] = [
    "a case brought under a law against taking",
    "a curfew in force",
    "a forum that may review in force",
    "a limit in force",
    "a law challenged before a forum",
    "a law struck down",
];

/// A nudge of a prior: by which day without which step, what it changes, and why.
struct Nudge {
    day: i64,
    /// The step whose absence it answers (an index into `STEPS`).
    step: usize,
    what: &'static str,
    prior: &'static str,
    to: f64,
    set: fn(&mut civ_agents::params::PeopleParams, f64) -> f64,
}

const NUDGES: [Nudge; 5] = [
    Nudge {
        day: 340,
        step: 0,
        what: "what bringing a case costs",
        prior: "crime.report_cost",
        to: 0.0,
        set: |p, v| std::mem::replace(&mut p.crime.report_cost, v),
    },
    Nudge {
        day: 460,
        step: 1,
        what: "the share of takings a household believes a curfew keeps off",
        prior: "crime.curfew_guard",
        to: 0.8,
        set: |p, v| std::mem::replace(&mut p.crime.curfew_guard, v),
    },
    Nudge {
        day: 600,
        step: 2,
        what: "what an evening at the gathering costs one who comes",
        prior: "elections.w_evening",
        to: 1.0,
        set: |p, v| std::mem::replace(&mut p.elections.w_evening, v),
    },
    Nudge {
        day: 700,
        step: 3,
        what: "the weight a household puts on the hours a curfew keeps it home",
        prior: "crime.curfew_cost_days",
        to: 2.0,
        set: |p, v| std::mem::replace(&mut p.crime.curfew_cost_days, v),
    },
    Nudge {
        day: 760,
        step: 3,
        what: "what proposing costs a sponsor",
        prior: "polity.propose_cost",
        to: 0.1,
        set: |p, v| std::mem::replace(&mut p.polity.propose_cost, v),
    },
];

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: courts <saves folder> [seed]");
        return ExitCode::FAILURE;
    };
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    match make_tries(&saves, seed) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("courts: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Which of `STEPS` the world has reached.
fn reached(sim: &Sim) -> [bool; 6] {
    let pop = sim.people();
    let policies = &sim.rules().catalog.policies;
    let laws = || pop.polities.iter().flat_map(|p| &p.laws);
    let in_force = |kind: PolicyKind| {
        laws().any(|l| {
            l.kind == kind && matches!(l.status, LawStatus::InForce | LawStatus::StruckDown)
        })
    };
    let review = laws().any(|l| {
        l.kind == PolicyKind::HearCases
            && l.status == LawStatus::InForce
            && policies[usize::from(l.policy)]
                .forum
                .as_ref()
                .is_some_and(|f| f.review)
    });
    [
        pop.order
            .cases
            .iter()
            .any(|c| c.challenge.is_none() && c.incident != 0),
        in_force(PolicyKind::Curfew),
        review,
        in_force(PolicyKind::Limit),
        pop.order.cases.iter().any(|c| c.challenge.is_some()),
        laws().any(|l| l.status == LawStatus::StruckDown),
    ]
}

/// Lives up to `TRIES` worlds from `seed` on and keeps the one that reached most steps.
fn make_tries(saves: &Path, seed: u64) -> Result<String, String> {
    let mut kept: Option<(usize, PathBuf, String)> = None;
    for k in 0..TRIES {
        let name = if k == 0 {
            NAME.to_owned()
        } else {
            format!("{NAME} {}", k + 1)
        };
        let (text, steps, dir) = make(saves, seed + k, &name)?;
        let text = format!("world {} (seed {}):\n{text}\nname: {name}", k + 1, seed + k);
        match &kept {
            Some((best, _, _)) if *best >= steps => {
                std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
            }
            _ => {
                if let Some((_, old, _)) = kept.take() {
                    std::fs::remove_dir_all(&old).map_err(|e| e.to_string())?;
                }
                kept = Some((steps, dir, text));
            }
        }
        if kept.as_ref().is_some_and(|k| k.0 == STEPS.len()) {
            break;
        }
    }
    let (steps, _, text) = kept.ok_or("no world was lived")?;
    Ok(format!(
        "{text}\nkept this world: {steps} of {} steps reached",
        STEPS.len()
    ))
}

/// Every law, in words.
fn laws_words(sim: &Sim) -> Vec<String> {
    let pop = sim.people();
    let policies = &sim.rules().catalog.policies;
    let mut out = Vec::new();
    for p in &pop.polities {
        for l in &p.laws {
            let (present, support, oppose) = l.counts();
            let hours = if l.kind == PolicyKind::Curfew {
                format!(", {:02}:00 to {:02}:00", l.hours.0, l.hours.1)
            } else {
                String::new()
            };
            out.push(format!(
                "{} proposed on day {} by {}{hours}: {:?} ({support} for, {oppose} against, \
                 {present} came)",
                policies[usize::from(l.policy)].name,
                l.proposed.day_index(),
                pop.name_of(l.sponsor),
                l.status,
            ));
        }
    }
    out
}

/// Cases by how they closed, and each challenge in words.
fn cases_words(sim: &Sim) -> Vec<String> {
    let pop = sim.people();
    let cases = &pop.order.cases;
    let count = |s: CaseStage| cases.iter().filter(|c| c.stage == s).count();
    let mut out = vec![format!(
        "{} cases brought ({} before a forum): {} found, {} not found, {} unheard, {} lapsed, \
         {} settled, {} withdrawn, {} open; {} departed from a finding",
        cases.len(),
        cases.iter().filter(|c| c.forum.is_some()).count(),
        count(CaseStage::Found),
        count(CaseStage::NotFound),
        count(CaseStage::Unheard),
        count(CaseStage::Lapsed),
        count(CaseStage::Settled),
        count(CaseStage::Withdrawn),
        count(CaseStage::Open),
        cases.iter().filter(|c| c.departed.is_some()).count(),
    )];
    for c in cases.iter().filter(|c| c.challenge.is_some()) {
        out.push(format!(
            "challenge {} brought on day {} by {}: {:?}, {} sat",
            c.id,
            c.opened,
            pop.name_of(c.by),
            c.stage,
            c.stances.len()
        ));
    }
    out
}

/// Lives one world named `name` from `seed`: what happened, in words; how many steps it reached;
/// and its saves folder.
fn make(saves: &Path, seed: u64, name: &str) -> Result<(String, usize, PathBuf), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut sim = Sim::create(
        &NewWorld {
            name: name.to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: PEOPLE,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        &content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| e.to_string())?;
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    let world_dir = saves.join(persist::world_dir_name(sim.meta()));
    let dir = SaveDir::create(world_dir.clone(), civ_schema::SAVE_EXTENSION)
        .map_err(|e| e.to_string())?;
    let mut log = Vec::new();
    let mut nudged = [false; NUDGES.len()];
    let mut first = [None::<i64>; STEPS.len()];
    let mut day = 0;
    while day < MAX_DAYS {
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        day += 1;
        let today = sim.now().day_index();
        let living = sim.people().people.len();
        if day % 90 == 0 {
            log.push(format!("day {today}: {living} people"));
        }
        if living == 0 {
            log.push(format!("day {today}: the village is empty"));
            break;
        }
        let now = reached(&sim);
        for (i, step) in STEPS.iter().enumerate() {
            if now[i] && first[i].is_none() {
                log.push(format!("day {today}: {step}"));
                first[i] = Some(day);
            }
        }
        if first[5] == Some(day) {
            let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "A law is struck down")
                .map_err(|e| e.to_string())?;
            log.push(format!("saved {}", saved.path.display()));
        }
        if first[5].is_some_and(|d| day - d >= AFTER_DAYS) {
            break;
        }
        for (n, done) in NUDGES.iter().zip(nudged.iter_mut()) {
            if *done || day < n.day || first[n.step].is_some() {
                continue;
            }
            let rules = sim
                .rules_mut_for_tests()
                .ok_or("the rules are shared: no nudge can be made")?;
            let was = (n.set)(&mut rules.people, n.to);
            log.push(format!(
                "day {today}: NUDGE ({}): {} days lived and not yet {}; {} {was} -> {}",
                n.what, n.day, STEPS[n.step], n.prior, n.to
            ));
            *done = true;
        }
    }
    log.push(format!(
        "lived {day} days, to day {}; {} people",
        sim.now().day_index(),
        sim.people().people.len()
    ));
    log.extend(laws_words(&sim).into_iter().map(|w| format!("  {w}")));
    log.extend(cases_words(&sim).into_iter().map(|w| format!("  {w}")));
    for p in &sim.people().polities {
        let label = civ_sim::labels::label_of(&sim, p);
        log.push(format!("  label: {} ({})", label.name, p.body.words()));
    }
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the rulings")
        .map_err(|e| e.to_string())?;
    log.push(format!("saved {}", saved.path.display()));
    let steps = first.iter().filter(|f| f.is_some()).count();
    Ok((log.join("\n"), steps, world_dir))
}
