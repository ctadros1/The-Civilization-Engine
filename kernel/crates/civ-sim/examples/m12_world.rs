//! The M12 demo (plan §7, M12c; ADR-0039): one world lived three times from the same founding
//! save, to show the model deliberator's seam without a model.
//!
//! 1. **Rules alone**: the default; no request is built and nothing new is saved.
//! 2. **A stand-in**: an in-process [`StandIn`] provider answers every polity review that offers
//!    moves. It is **not a model** and no language model is run: it is a fixed policy written for
//!    this demo, so its choices can be explained. It proposes the listed move the notable expects
//!    the most support for, when that support is at least one half; otherwise it does not answer,
//!    and the rule deliberator chooses on its own draw, as it does when a real model is too slow.
//!    The world is saved at the end.
//! 3. **Recorded**: the founding save loaded again, with the answers read from the second run's
//!    save (as `civ-host serve --deliberator recorded --answers <save>` does), lived as long.
//!
//! It prints, for each run, how many reviews offered moves and who chose, the laws proposed and
//! how each stands in the government panel's words ("chose by the model", "chose by rule (the
//! model did not answer)"), and whether the third run's final sections are byte-equal to the
//! second's. What the people do with a proposal is theirs.
//!
//! ```sh
//! cargo run --release -p civ-sim --example m12_world -- <saves folder> [years] [seed] [groups] [people] [size]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use civ_agents::deliberation::{
    ChosenBy, DeliberationMode, DeliberationRequest, ModelAnswer, ModelProvider,
    ProviderFingerprint,
};
use civ_core::time::DAYS_PER_YEAR;
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Years lived, unless given.
const YEARS: i64 = 2;
/// The world's seed, unless given.
const SEED: u64 = 3;
/// Founding groups, unless given.
const GROUPS: u32 = 2;
/// People in each founding group, unless given.
const PEOPLE: u32 = 100;
/// The map's side in cells, unless given.
const SIZE: u32 = 1024;
const DAY: i64 = 24 * 60;
const NAME: &str = "Who Chooses";
/// The least support the stand-in will propose on.
const LEAST_SUPPORT: f64 = 0.5;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: m12_world <saves folder> [years] [seed] [groups] [people] [size]");
        return ExitCode::FAILURE;
    };
    let years: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(YEARS);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(SEED);
    let groups: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(GROUPS);
    let people: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(PEOPLE);
    let size: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(SIZE);
    match make(&saves, years, seed, groups, people, size) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("the demo failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Prints a line of the demo's record as it comes.
fn say(line: String) {
    use std::io::Write;
    println!("{line}");
    let _ = std::io::stdout().flush();
}

/// A stand-in for a model (ADR-0039 §3), **not a model**: a fixed policy. Of the moves listed it
/// proposes the one with the most expected support (the first among equals) when that support is
/// at least [`LEAST_SUPPORT`]; otherwise it does not answer, and rules choose.
struct StandIn {
    /// Requests seen.
    asked: Arc<AtomicUsize>,
    /// Requests it answered with a move.
    proposed: Arc<AtomicUsize>,
}

impl ModelProvider for StandIn {
    fn fingerprint(&self) -> ProviderFingerprint {
        ProviderFingerprint {
            kind: "stand-in".into(),
            model: "most-support-at-least-half (a fixed policy, not a model)".into(),
        }
    }

    fn answer(&mut self, request: &DeliberationRequest) -> ModelAnswer {
        self.asked.fetch_add(1, Ordering::Relaxed);
        let best = request
            .moves
            .iter()
            .max_by(|a, b| a.support.total_cmp(&b.support).then(b.index.cmp(&a.index)));
        match best {
            Some(m) if m.support >= LEAST_SUPPORT => {
                self.proposed.fetch_add(1, Ordering::Relaxed);
                ModelAnswer::Move(m.index)
            }
            _ => ModelAnswer::Unanswered,
        }
    }
}

/// One law as the government panel puts it.
struct LawWords {
    polity: String,
    what: String,
    sponsor: String,
    day: i64,
    issue: String,
    status: &'static str,
    decision: String,
}

/// Every law of every polity, in the government panel's words (the kernel's frame).
fn laws(sim: &Sim) -> Result<Vec<LawWords>, String> {
    let payload = civ_sim::frames::government::government_response(sim);
    let info = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .map_err(|e| e.to_string())?
        .body_as_government()
        .ok_or("no government in the response")?;
    let mut out = Vec::new();
    for p in info.polities().into_iter().flat_map(|v| v.iter()) {
        for l in p.laws().into_iter().flat_map(|v| v.iter()) {
            out.push(LawWords {
                polity: p.name().unwrap_or_default().to_owned(),
                what: l.what().unwrap_or_default().to_owned(),
                sponsor: l.sponsor_name().unwrap_or_default().to_owned(),
                day: l.proposed_minute() / DAY,
                issue: l.issue().unwrap_or_default().to_owned(),
                status: match l.status() {
                    0 => "before the gathering",
                    1 => "in force",
                    2 => "rejected",
                    3 => "lapsed",
                    4 => "superseded",
                    5 => "carried",
                    6 => "struck down",
                    _ => "unknown",
                },
                decision: l.decision().unwrap_or_default().to_owned(),
            });
        }
    }
    out.sort_by_key(|l| l.day);
    Ok(out)
}

/// What a run came to.
struct Lived {
    sim: Sim,
    asked: usize,
    proposed: usize,
    seconds: f64,
}

/// Loads `start` and lives it `days` with `mode`.
fn live(
    start: &Path,
    content: &civ_content::ContentRegistry,
    days: i64,
    mode: Option<DeliberationMode>,
    counters: Option<(Arc<AtomicUsize>, Arc<AtomicUsize>)>,
) -> Result<Lived, String> {
    let mut sim = persist::load(start, content).map_err(|e| e.to_string())?;
    if let Some(m) = mode {
        sim.set_deliberation(m);
    }
    let t = Instant::now();
    for day in 1..=days {
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        if day % DAYS_PER_YEAR == 0 {
            say(format!(
                "  year {}: {} people; {} laws proposed so far",
                day / DAYS_PER_YEAR,
                sim.people().living(),
                sim.people()
                    .polities
                    .iter()
                    .map(|p| p.laws.len())
                    .sum::<usize>()
            ));
        }
    }
    let (asked, proposed) = counters.map_or((0, 0), |(a, p)| {
        (a.load(Ordering::Relaxed), p.load(Ordering::Relaxed))
    });
    Ok(Lived {
        sim,
        asked,
        proposed,
        seconds: t.elapsed().as_secs_f64(),
    })
}

/// Prints a run's report: who chose, and the laws.
fn report(label: &str, lived: &Lived) -> Result<(), String> {
    let sim = &lived.sim;
    let saved = sim.saved_answers();
    let by = |who: ChosenBy, made: bool| {
        saved
            .answers
            .iter()
            .filter(|a| a.by == who && a.chosen.is_some() == made)
            .count()
    };
    say(format!(
        "{label}: lived to day {} in {:.0} s; {} people",
        sim.now().day_index(),
        lived.seconds,
        sim.people().living()
    ));
    match sim.deliberation() {
        DeliberationMode::Rule => say(
            "  rules alone: no request is built, so reviews offering moves are not counted; \
             every choice is by rule; no answers saved"
                .to_owned(),
        ),
        DeliberationMode::Model(_) => {
            let fallback_made = by(ChosenBy::RuleFallback, true);
            say(format!(
                "  reviews that offered moves: {}; the stand-in proposed a move at {} and \
                 left {} to rules (none had support of at least {LEAST_SUPPORT})",
                lived.asked,
                lived.proposed,
                lived.asked - lived.proposed
            ));
            say(format!(
                "  of those left to rules, rules proposed a move at {fallback_made} and none at {}",
                (lived.asked - lived.proposed).saturating_sub(fallback_made)
            ));
            say(format!(
                "  answers saved: {} ({} by the stand-in, {} moves by rule after it did not \
                 answer); provider {:?}",
                saved.answers.len(),
                by(ChosenBy::Model, true) + by(ChosenBy::Model, false),
                fallback_made,
                saved
                    .providers
                    .iter()
                    .map(|p| format!("{}: {}", p.kind, p.model))
                    .collect::<Vec<_>>()
            ));
        }
        DeliberationMode::Recorded(source) => {
            let model = by(ChosenBy::Model, true) + by(ChosenBy::Model, false);
            say(format!(
                "  recorded: {} answers read from the save; {} kept again by this run ({model} \
                 the stand-in's, {} rules' moves after it did not answer); no provider asked",
                source.answers.len(),
                saved.answers.len(),
                by(ChosenBy::RuleFallback, true)
            ));
        }
    }
    let all = laws(sim)?;
    say(format!("  laws proposed: {}", all.len()));
    for l in &all {
        say(format!(
            "    day {} at {}: {}, proposed by {} ({}); {}{}",
            l.day,
            l.polity,
            l.what,
            l.sponsor,
            l.issue,
            l.status,
            if l.decision.is_empty() {
                String::new()
            } else {
                format!(": {}", l.decision)
            }
        ));
    }
    let passed = all
        .iter()
        .filter(|l| !matches!(l.status, "before the gathering" | "rejected"))
        .count();
    say(format!(
        "  passed (in force or since ended): {passed}; rejected: {}; awaiting a gathering: {}",
        all.iter().filter(|l| l.status == "rejected").count(),
        all.iter()
            .filter(|l| l.status == "before the gathering")
            .count()
    ));
    Ok(())
}

fn make(
    saves: &Path,
    years: i64,
    seed: u64,
    groups: u32,
    people: u32,
    size: u32,
) -> Result<(), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let mut content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let band = &mut content.people.params.band;
    band.max_size = band.max_size.max(people);
    band.min_size = band.min_size.min(people);
    let made = Instant::now();
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: NAME.to_owned(),
            seed,
            preset_id: content.default_preset().id.clone(),
            size_cells: size,
            band_size: people,
            neighbours: vec![people; groups.saturating_sub(1) as usize],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        // A fixed identity, so the demo lives the same life every run.
        [12; 16],
    )
    .map_err(|e| e.to_string())?;
    if let Some(problem) = sim.founding_problem() {
        return Err(format!("the founding groups could not settle: {problem}"));
    }
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    say(format!(
        "world {NAME:?} (seed {seed}, {size} cells): {} people in {} settlements, founded in \
         {:.1} s, saved at the end of day {}; each run lives {years} years from that save",
        sim.people().living(),
        sim.land().settlements.len(),
        made.elapsed().as_secs_f64(),
        sim.now().day_index() - 1
    ));
    let world_dir = saves.join(persist::world_dir_name(sim.meta()));
    let dir = SaveDir::create(world_dir, civ_schema::SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let start = persist::save(&mut sim, &dir, SaveKind::Manual, "Founded")
        .map_err(|e| e.to_string())?
        .path;
    drop(sim);
    let days = years * DAYS_PER_YEAR;

    // 1. Rules alone.
    say("run 1, rules alone:".to_owned());
    let rule = live(&start, &content, days, None, None)?;
    report("run 1, rules alone", &rule)?;
    drop(rule);

    // 2. The stand-in, saved at the end.
    say("run 2, the stand-in (a fixed policy, not a model):".to_owned());
    let (asked, proposed) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let stand_in = DeliberationMode::Model(Box::new(StandIn {
        asked: Arc::clone(&asked),
        proposed: Arc::clone(&proposed),
    }));
    let mut model = live(
        &start,
        &content,
        days,
        Some(stand_in),
        Some((asked, proposed)),
    )?;
    report("run 2, the stand-in", &model)?;
    let end = persist::save(&mut model.sim, &dir, SaveKind::Manual, "Stand-in")
        .map_err(|e| e.to_string())?
        .path;
    let bytes = std::fs::metadata(&end).map_or(0, |m| m.len());
    let reloaded = persist::load(&end, &content).map_err(|e| e.to_string())?;
    say(format!(
        "  saved ({:.1} MB) and loaded: {} answers read back, the same as kept: {}",
        bytes as f64 / 1e6,
        reloaded.saved_answers().answers.len(),
        reloaded.saved_answers() == model.sim.saved_answers()
    ));

    // 3. The founding save again, answered from the second run's save.
    say("run 3, recorded (the founding save, answered from run 2's save):".to_owned());
    let answers = reloaded.saved_answers().clone();
    drop(reloaded);
    let replay = live(
        &start,
        &content,
        days,
        Some(DeliberationMode::Recorded(answers)),
        None,
    )?;
    report("run 3, recorded", &replay)?;
    let (a, b) = (
        persist::encode_sections(&model.sim),
        persist::encode_sections(&replay.sim),
    );
    let differ: Vec<String> = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| x.tag != y.tag || x.bytes != y.bytes)
        .map(|(x, _)| x.tag.to_string())
        .collect();
    say(format!(
        "replay: {} sections of run 3's world against {} of run 2's; byte-equal: {}{}",
        b.len(),
        a.len(),
        if a.len() == b.len() && differ.is_empty() {
            "yes"
        } else {
            "no"
        },
        if differ.is_empty() {
            String::new()
        } else {
            format!(" (differ: {})", differ.join(", "))
        }
    ));
    Ok(())
}
