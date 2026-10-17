//! Three villages and what trade between them makes of their gatherings, for the M7c demo (plan
//! §7: "a republic's assembly captured by a merchant guild passes protectionist tariffs"; ADR-0026
//! §6: the demo's outcome is one of several). Three founding bands of `BAND` that know where each
//! other camped live as the content has them. Nothing is set: no purchase is made for anyone, no
//! law proposed, no organization founded, recognized or granted a privilege, no gift given, and no
//! custom amended by the observer. Each step capture needs is people's: a purchase at another
//! settlement's door, a trade law (a duty or an embargo) in force, an organization by stake, its
//! recognition, a privilege granted it, a patron's gift before a gathering, and a custom closed to
//! an organization's members. If a step has not come by its time box, the observer nudges a prior
//! (`NUDGE`, logged as such), never an event, and only the two ADR-0026 §6 names: an
//! organization's fixed cost (`faction.organize_days`) and a report's half-life
//! (`reports.half_life_days`); `w_gain` and the stance margin are never touched. The world lives
//! `MAX_DAYS` at most, and ends early if its people are gone; it is saved at the end ("After the
//! years"). Nudged priors live only in this run: a loaded world takes its content's.
//!
//! With `TCE_SMELT_DAY=<day>` set, the observer introduces copper smelting on that day to the
//! grown members of the largest household of the first village (the knowledge tool; logged as
//! `INTRODUCED`), for the M7d demo's metal (ADR-0027 §6); the world's metal is tallied at the end.
//!
//! ```sh
//! cargo run --release -p civ-sim --example capture -- <saves folder> [seed]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::{LawStatus, Membership, PolicyKind};
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// People in each founding band.
const BAND: u32 = 40;
/// Days lived at most: four years.
const MAX_DAYS: i64 = 4 * 360;
const DAY: i64 = 24 * 60;
const NAME: &str = "Three Fords";

/// The steps capture needs, in the order the demo's time boxes ask for them.
const STEPS: [&str; 7] = [
    "a purchase at another settlement's door",
    "a trade law in force",
    "an organization by stake",
    "an organization recognized",
    "a privilege in force",
    "a patron's gift before a gathering",
    "a custom closed to an organization's members",
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

const NUDGES: [Nudge; 2] = [
    Nudge {
        day: 360,
        step: 0,
        what: "how long a price heard of elsewhere is believed",
        prior: "reports.half_life_days",
        to: 90.0,
        set: |p, v| std::mem::replace(&mut p.reports.half_life_days, v),
    },
    Nudge {
        day: 720,
        step: 2,
        what: "what organizing costs its founder, in days of their household's food",
        prior: "faction.organize_days",
        to: 10.0,
        set: |p, v| std::mem::replace(&mut p.faction.organize_days, v),
    },
];

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: capture <saves folder> [seed]");
        return ExitCode::FAILURE;
    };
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    match make(&saves, seed) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("capture: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Which of `STEPS` the world has reached.
fn reached(sim: &Sim) -> [bool; 7] {
    let pop = sim.people();
    let laws = || pop.polities.iter().flat_map(|p| &p.laws);
    let in_force =
        |kind: PolicyKind| laws().any(|l| l.kind == kind && l.status == LawStatus::InForce);
    [
        pop.markets
            .iter()
            .any(|m| m.recent.iter().any(|t| t.from.is_some())),
        in_force(PolicyKind::Duty) || in_force(PolicyKind::Embargo),
        pop.factions.list.iter().any(|f| f.stake.is_some()),
        in_force(PolicyKind::Recognize),
        in_force(PolicyKind::Privilege),
        pop.polities.iter().any(|p| !p.patron_gifts.is_empty()),
        pop.polities.iter().any(|p| {
            matches!(
                p.body.members,
                Membership::Organized | Membership::Privileged
            )
        }),
    ]
}

/// Every trade law, organization's law and amendment, in words.
fn laws_words(sim: &Sim) -> Vec<String> {
    let pop = sim.people();
    let policies = &sim.rules().catalog.policies;
    let mut out = Vec::new();
    for p in &pop.polities {
        for l in p.laws.iter().filter(|l| {
            matches!(
                l.kind,
                PolicyKind::Duty
                    | PolicyKind::Embargo
                    | PolicyKind::Recognize
                    | PolicyKind::Privilege
                    | PolicyKind::AmendBody
                    | PolicyKind::DebtsHeard
                    | PolicyKind::LendFromStore
            )
        }) {
            let (present, support, oppose) = l.counts();
            out.push(format!(
                "{} proposed on day {} by {}: {:?} ({support} for, {oppose} against, {present} \
                 came)",
                policies[usize::from(l.policy)].name,
                l.proposed.day_index(),
                pop.name_of(l.sponsor),
                l.status,
            ));
        }
    }
    out
}

/// Lives one world from `seed`: what happened, in words.
fn make(saves: &Path, seed: u64) -> Result<String, String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut sim = Sim::create(
        &NewWorld {
            name: NAME.to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 1024,
            band_size: BAND,
            neighbours: vec![BAND, BAND],
            neighbours_known: true,
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
    let dir = SaveDir::create(world_dir, civ_schema::SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let mut log = vec![format!("world (seed {seed}):")];
    let mut nudged = [false; NUDGES.len()];
    let mut first = [None::<i64>; STEPS.len()];
    let smelt_day: Option<i64> = std::env::var("TCE_SMELT_DAY")
        .ok()
        .and_then(|d| d.parse().ok());
    let mut day = 0;
    while day < MAX_DAYS {
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        day += 1;
        let today = sim.now().day_index();
        let living = sim.people().people.len();
        if day % 120 == 0 {
            log.push(format!("day {today}: {living} people"));
        }
        if living == 0 {
            log.push(format!("day {today}: the valley is empty"));
            break;
        }
        let now = reached(&sim);
        for (i, step) in STEPS.iter().enumerate() {
            if now[i] && first[i].is_none() {
                log.push(format!("day {today}: {step}"));
                first[i] = Some(day);
            }
        }
        if smelt_day == Some(day) {
            log.extend(introduce_smelting(&mut sim)?);
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
    let purchases: usize = sim
        .people()
        .markets
        .iter()
        .map(|m| m.recent.iter().filter(|t| t.from.is_some()).count())
        .sum();
    log.push(format!(
        "  purchases at another settlement's door its markets still remember: {purchases}"
    ));
    // Loans in goods (M7d slice CA): what the asking came to.
    let credit = &sim.people().order.credit;
    let count = |st: civ_agents::credit::Standing| {
        credit.claims.iter().filter(|c| c.standing == st).count()
    };
    log.push(format!(
        "  loans: {} asked, {} refused, {} made ({} open, {} repaid, {} defaulted, {} refused at \
         the due day, {} lapsed)",
        credit.asked,
        credit.refused,
        credit.claims.len(),
        count(civ_agents::credit::Standing::Open),
        count(civ_agents::credit::Standing::Repaid),
        count(civ_agents::credit::Standing::Defaulted),
        count(civ_agents::credit::Standing::Refused),
        count(civ_agents::credit::Standing::Lapsed),
    ));
    // Loans from a common store under a law of lending from the store (M7d slice CC).
    let from_store = credit
        .claims
        .iter()
        .filter(|c| sim.people().polities.iter().any(|p| p.id == c.creditor))
        .count();
    log.push(format!("  loans from the common store: {from_store}"));
    // Pledges and what came of them (M7d slice CB).
    let pledged = |code: u8| {
        credit
            .claims
            .iter()
            .filter(move |c| c.pledge.map(civ_agents::credit::Pledge::code) == Some(code))
    };
    log.push(format!(
        "  pledges: {} against a field's use ({} taken), {} with a guarantor ({} called on, {} \
         guarantor's claims); {} asked for time, {} granted",
        pledged(1).count(),
        pledged(1).filter(|c| c.taken.is_some()).count(),
        pledged(2).count(),
        credit
            .claims
            .iter()
            .filter(|c| c.by_guarantor > 0.0)
            .count(),
        credit
            .claims
            .iter()
            .filter(|c| c.recourse.is_some())
            .count(),
        credit.claims.iter().map(|c| u32::from(c.asks)).sum::<u32>(),
        credit
            .claims
            .iter()
            .map(|c| u32::from(c.extensions))
            .sum::<u32>(),
    ));
    // Wages workshops could not pay (M7d slice CB).
    let wages: Vec<_> = credit
        .claims
        .iter()
        .filter(|c| sim.people().firm(c.debtor).is_some())
        .collect();
    log.push(format!(
        "  wages owed by workshops: {} ({} paid, {} unpaid when due)",
        wages.len(),
        wages
            .iter()
            .filter(|c| c.standing == civ_agents::credit::Standing::Repaid)
            .count(),
        wages
            .iter()
            .filter(|c| c.standing == civ_agents::credit::Standing::Defaulted)
            .count(),
    ));
    // Debts brought under a law of debts heard (M7d slice CB).
    let debts: Vec<_> = sim
        .people()
        .order
        .cases
        .iter()
        .filter(|c| c.loan.is_some())
        .collect();
    log.push(format!(
        "  debts brought: {} ({} found, {} not found, {} unheard or lapsed)",
        debts.len(),
        debts
            .iter()
            .filter(|c| c.stage == civ_agents::crime::CaseStage::Found)
            .count(),
        debts
            .iter()
            .filter(|c| c.stage == civ_agents::crime::CaseStage::NotFound)
            .count(),
        debts
            .iter()
            .filter(|c| matches!(
                c.stage,
                civ_agents::crime::CaseStage::Unheard | civ_agents::crime::CaseStage::Lapsed
            ))
            .count(),
    ));
    log.extend(metal_words(&sim));
    log.extend(laws_words(&sim).into_iter().map(|w| format!("  {w}")));
    for p in &sim.people().polities {
        let label = civ_sim::labels::label_of(&sim, p);
        log.push(format!("  label: {} ({})", label.name, p.body.words()));
        for w in civ_sim::frames::government::capture_words(&sim, p) {
            log.push(format!("    {w}"));
        }
    }
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the years")
        .map_err(|e| e.to_string())?;
    log.push(format!("saved {}", saved.path.display()));
    let steps = first.iter().filter(|f| f.is_some()).count();
    log.push(format!("reached {steps} of {} steps", STEPS.len()));
    Ok(log.join("\n"))
}

/// The observer's introduction of copper smelting to the grown members of the largest household
/// of the first village (the knowledge tool), in words.
fn introduce_smelting(sim: &mut Sim) -> Result<Vec<String>, String> {
    let now = sim.now();
    let first = sim
        .people()
        .households
        .iter()
        .filter_map(|(_, x)| x.settlement)
        .min()
        .ok_or("nobody lives in a village")?;
    let hh = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| x.settlement == Some(first))
        .max_by_key(|x| (x.members.len(), std::cmp::Reverse(x.id)))
        .map(|x| x.id)
        .ok_or("the village has no household")?;
    let grown: Vec<_> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.household == hh && p.age_years(now) >= 16.0)
        .map(|p| p.id)
        .collect();
    let mut out = Vec::new();
    for p in grown {
        sim.introduce_technique(p, "core:technique/copper_smelting", false)?;
        out.push(format!(
            "day {}: INTRODUCED (observer): copper smelting to {} of household {hh}",
            now.day_index(),
            sim.people().name_of(p)
        ));
    }
    Ok(out)
}

/// The world's metal, by village: who knows how to smelt, and what is held of ore, copper and
/// axes of each kind.
fn metal_words(sim: &Sim) -> Vec<String> {
    let goods = &sim.rules().catalog.goods;
    let at = |id: &str| goods.iter().position(|g| g.id == id);
    let (Some(ore), Some(copper), Some(cast), Some(axe)) = (
        at("core:good/copper_ore"),
        at("core:good/copper"),
        at("core:good/copper_axe"),
        at("core:good/axe"),
    ) else {
        return Vec::new();
    };
    let Some(smelting) = sim
        .rules()
        .catalog
        .technique_index("core:technique/copper_smelting")
    else {
        return Vec::new();
    };
    let pop = sim.people();
    let mut villages: Vec<_> = pop
        .households
        .iter()
        .filter_map(|(_, x)| x.settlement)
        .collect();
    villages.sort_unstable();
    villages.dedup();
    let mut out = Vec::new();
    for v in villages {
        let held = |g: usize| -> f64 {
            pop.households
                .iter()
                .map(|(_, x)| x)
                .filter(|x| x.settlement == Some(v))
                .map(|x| x.stores.get(g).copied().unwrap_or(0.0))
                .sum()
        };
        let knowers = pop
            .people
            .iter()
            .filter(|(_, p)| p.knows(smelting))
            .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(v))
            .count();
        let bodies: Vec<_> = sim
            .land()
            .deposits
            .iter()
            .filter(|d| usize::from(d.body.good) == ore)
            .collect();
        let known = bodies.iter().filter(|d| pop.knows_deposit(v, d.id)).count();
        out.push(format!(
            "  metal at village {v}: {knowers} know smelting; knows {known} of {} ore bodies; {:.1} \
             kg ore, {:.2} kg copper, {:.2} copper axes, {:.2} stone axes",
            bodies.len(),
            held(ore),
            held(copper),
            held(cast),
            held(axe)
        ));
    }
    out
}
