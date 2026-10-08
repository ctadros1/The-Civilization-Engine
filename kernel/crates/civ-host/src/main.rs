//! `civ-host`: the headless host of The Civilization Engine. Run `civ-host --help`.

#![forbid(unsafe_code)]

use std::io::Write as _;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Context;
use civ_host::engine::{self, EngineConfig};
use civ_host::session::SessionMarker;
use civ_host::{commands, paths, protocol, server, smoke};
use clap::{Args, Parser, Subcommand};

/// The default port of the observer on 127.0.0.1.
const DEFAULT_PORT: u16 = 7420;

#[derive(Parser)]
#[command(
    name = "civ-host",
    version,
    about = "The Civilization Engine's headless host: the web observer's server, worlds, saves, \
             content checks and smoke seeds."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the web observer on 127.0.0.1 (what runs when no command is given).
    Serve(ServeArgs),
    /// Generate a world from a seed and save it.
    New(NewArgs),
    /// Inspect or verify a save file.
    #[command(subcommand)]
    Save(SaveCommand),
    /// Check the content packs.
    #[command(subcommand)]
    Content(ContentCommand),
    /// Generate the smoke-seed worlds and check them against fixed thresholds.
    Smoke(SmokeArgs),
    /// Live the sanity dashboard's five river-valley worlds fifty years and grade plan §4.7.
    Dashboard(DashboardArgs),
    /// Gate B (ADR-0011 §5): a year from two fixture worlds, several times in each mode, and
    /// Accelerated mode's aggregates against Detailed mode's.
    Consistency(ConsistencyArgs),
    /// The notables' gate (ADR-0014 §4): two fixture worlds lived three years several times with
    /// the notables' tier and without it, and their polities' aggregates compared.
    Notables(NotablesArgs),
    /// Live one world for some years and report how its people live at each year's end.
    Run(RunArgs),
    /// Report a world's weather on the valley floor, year by year, as its seed draws it.
    Weather(WeatherArgs),
}

#[derive(Args, Clone)]
struct WeatherArgs {
    #[command(flatten)]
    folders: Folders,
    /// World seed.
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// World-generation preset id [default: the content's default].
    #[arg(long)]
    preset: Option<String>,
    /// Years to report, from the world's first.
    #[arg(long, default_value_t = 50)]
    years: u32,
}

#[derive(Args, Clone)]
struct RunArgs {
    #[command(flatten)]
    folders: Folders,
    /// World seed.
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// World-generation preset id [default: the content's default].
    #[arg(long)]
    preset: Option<String>,
    /// Map side, cells.
    #[arg(long, default_value_t = 768)]
    size: u32,
    /// Founding band size [default: the content's].
    #[arg(long, default_value_t = 0)]
    band: u32,
    /// Years to live.
    #[arg(long, default_value_t = 3)]
    years: u32,
    /// Property regime id [default: the content's default].
    #[arg(long)]
    regime: Option<String>,
    /// Families the observer sends to the village as it is founded, in groups of up to 20.
    #[arg(long, default_value_t = 0)]
    families: u32,
    /// A technique (by id) the observer introduces to the band's eldest grown founder as the
    /// world begins; repeat for more.
    #[arg(long)]
    introduce: Vec<String>,
}

#[derive(Args, Clone, Default)]
struct Folders {
    /// The content folder [default: content/ found above the current directory].
    #[arg(long)]
    content: Option<PathBuf>,
    /// The saves folder [default: saves/ next to the content folder].
    #[arg(long)]
    saves: Option<PathBuf>,
}

#[derive(Args, Clone)]
struct ServeArgs {
    #[command(flatten)]
    folders: Folders,
    /// Port on 127.0.0.1 (0 picks a free one).
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
    /// The built web shell [default: web/dist next to the content folder].
    #[arg(long)]
    web: Option<PathBuf>,
    /// Open the observer in the default browser.
    #[arg(long)]
    open: bool,
}

impl Default for ServeArgs {
    fn default() -> Self {
        ServeArgs {
            folders: Folders::default(),
            port: DEFAULT_PORT,
            web: None,
            open: false,
        }
    }
}

#[derive(Args)]
struct NewArgs {
    #[command(flatten)]
    folders: Folders,
    /// World-generation seed.
    #[arg(long)]
    seed: u64,
    /// World preset id [default: the content's default preset].
    #[arg(long)]
    preset: Option<String>,
    /// Cells per side: a multiple of 8 from 64 to 4096 (8 m cells).
    #[arg(long, default_value_t = civ_sim::DEFAULT_MAP_SIZE)]
    size: u32,
    /// World name.
    #[arg(long, default_value = "")]
    name: String,
    /// Days the founding band lives before the world is saved.
    #[arg(long, default_value_t = 0)]
    days: u32,
    /// Property regime id [default: the content's default].
    #[arg(long)]
    regime: Option<String>,
    /// Founding band size [default: the content's].
    #[arg(long, default_value_t = 0)]
    band: u32,
    /// Families the observer sends to the village as it is founded, in groups of up to 20.
    #[arg(long, default_value_t = 0)]
    families: u32,
    /// A technique (by id) the observer introduces to the band's eldest grown founder as the
    /// world begins; repeat for more.
    #[arg(long)]
    introduce: Vec<String>,
}

#[derive(Subcommand)]
enum SaveCommand {
    /// Print a save's header, sections and compatibility.
    Info(SaveArgs),
    /// Check every byte of a save and load its world.
    Verify(SaveArgs),
}

#[derive(Args)]
struct SaveArgs {
    #[command(flatten)]
    folders: Folders,
    /// The save file.
    file: PathBuf,
}

#[derive(Subcommand)]
enum ContentCommand {
    /// Validate the content packs.
    Validate {
        #[command(flatten)]
        folders: Folders,
        /// Print the report as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
struct ConsistencyArgs {
    #[command(flatten)]
    folders: Folders,
    /// Only Detailed runs, to see the spread the tolerances are set from (PROJECT_PLAN §9).
    #[arg(long)]
    calibrate: bool,
    /// Runs at a time [default: the cores, at most four].
    #[arg(long)]
    threads: Option<usize>,
    /// Accelerated runs without leisure blocks: to see what the approximation changes, not the
    /// gate.
    #[arg(long)]
    without_leisure_blocks: bool,
    /// Accelerated runs without the household view: to see what the approximation changes, not
    /// the gate.
    #[arg(long)]
    without_household_view: bool,
    /// Also write the report as JSON to this file.
    #[arg(long)]
    json: Option<PathBuf>,
}

#[derive(Args)]
struct NotablesArgs {
    #[command(flatten)]
    folders: Folders,
    /// Only runs with the tier, to see the spread the tolerances are set from (PROJECT_PLAN §9).
    #[arg(long)]
    calibrate: bool,
    /// Runs at a time [default: the cores, at most four].
    #[arg(long)]
    threads: Option<usize>,
    /// Also write the report as JSON to this file.
    #[arg(long)]
    json: Option<PathBuf>,
}

#[derive(Args)]
struct DashboardArgs {
    #[command(flatten)]
    folders: Folders,
    /// Cells per side of each world.
    #[arg(long, default_value_t = 1024)]
    size: u32,
    /// Years each world lives after its first month. The dashboard is fifty; fewer only for a
    /// quick look.
    #[arg(long, default_value_t = civ_host::dashboard::YEARS)]
    years: u32,
    /// Also write the report as JSON to this file.
    #[arg(long)]
    json: Option<PathBuf>,
    /// Keep each world's saves at every tenth year's end in this folder.
    #[arg(long)]
    keep_saves: Option<PathBuf>,
}

#[derive(Args)]
struct SmokeArgs {
    #[command(flatten)]
    folders: Folders,
    /// Cells per side of each world. Below 1024 (8 km), a coastal window can be mostly sea.
    #[arg(long, default_value_t = 1024)]
    size: u32,
    /// Seeds per preset.
    #[arg(long, default_value_t = 5)]
    seeds: u64,
    /// Years each world lives on after its first month, checked at every year's end (the M1
    /// sanity run: 10).
    #[arg(long, default_value_t = 0)]
    years: u32,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("civ_host=info,warn")),
        )
        .with_writer(std::io::stderr)
        // A closed terminal or a broken `| tee` must not turn every log line into a panic: the
        // fallback report of a failed write uses `eprintln!`, which panics when stderr is gone.
        .log_internal_errors(false)
        .init();
    let cli = Cli::parse();
    let result = match cli.command.unwrap_or(Command::Serve(ServeArgs::default())) {
        Command::Serve(args) => serve(args),
        Command::New(args) => new(args),
        Command::Save(SaveCommand::Info(args)) => save_info(args),
        Command::Save(SaveCommand::Verify(args)) => save_verify(args),
        Command::Content(ContentCommand::Validate { folders, json }) => {
            paths::locate(folders.content, None, None)
                .map(|layout| commands::content_validate(&layout.content, json))
        }
        Command::Smoke(args) => run_smoke(args),
        Command::Dashboard(args) => run_dashboard(args),
        Command::Consistency(args) => run_consistency(args),
        Command::Notables(args) => run_notables(args),
        Command::Run(args) => run_world(args),
        Command::Weather(args) => weather(args),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_world(args: RunArgs) -> anyhow::Result<ExitCode> {
    let layout = paths::locate(args.folders.content, args.folders.saves, None)?;
    let content = commands::load_content(&layout.content)?;
    let options = civ_host::report::RunOptions {
        preset: args.preset,
        seed: args.seed,
        size: args.size,
        band: args.band,
        years: args.years,
        regime: args.regime,
        families: args.families,
        introduce: args.introduce,
    };
    civ_host::report::run(&content, &options, &mut std::io::stdout().lock())?;
    Ok(ExitCode::SUCCESS)
}

fn weather(args: WeatherArgs) -> anyhow::Result<ExitCode> {
    let layout = paths::locate(args.folders.content, args.folders.saves, None)?;
    let content = commands::load_content(&layout.content)?;
    let options = civ_host::weather::WeatherOptions {
        preset: args.preset,
        seed: args.seed,
        years: args.years,
    };
    civ_host::weather::run(&content, &options, &mut std::io::stdout().lock())?;
    Ok(ExitCode::SUCCESS)
}

fn serve(args: ServeArgs) -> anyhow::Result<ExitCode> {
    let layout = paths::locate(args.folders.content, args.folders.saves, args.web)?;
    let content = Arc::new(commands::load_content(&layout.content)?);
    std::fs::create_dir_all(&layout.saves)
        .with_context(|| format!("could not create {}", layout.saves.display()))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, args.port))
            .await
            .with_context(|| {
                format!(
                    "port {} on 127.0.0.1 is not available (is another civ-host running? choose \
                     another with --port)",
                    args.port
                )
            })?;
        let addr = listener.local_addr()?;
        let previous = SessionMarker::left_behind(&layout.saves);
        let engine = engine::start(
            EngineConfig::new(Arc::clone(&content), layout.saves.clone()),
            previous,
        )
        .context("the engine could not start")?;
        let app = server::router(
            engine.channels.clone(),
            protocol::welcome_payload(&content),
            layout.web.clone(),
            server::Access::default(),
        );
        let url = format!("http://{addr}/");
        println!("The Civilization Engine is running at {url} (Ctrl+C to stop)");
        println!("  saves: {}", layout.saves.display());
        match &layout.web {
            Some(web) => println!("  web shell: {}", web.display()),
            None => println!("  web shell: not built (the page explains how to build it)"),
        }
        if args.open {
            open_browser(&url);
        }
        let served = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await;
        // The terminal may be gone by now (a closed window, a killed `| tee`), and `println!`
        // panics then. Nothing may stop the save.
        let _ = writeln!(
            std::io::stdout(),
            "Stopping: the world is saved if it changed."
        );
        tokio::task::spawn_blocking(move || engine.shutdown()).await?;
        served?;
        Ok(ExitCode::SUCCESS)
    })
}

/// Resolves when the host should stop: Ctrl+C; on Unix also SIGTERM, and SIGHUP from a closed
/// terminal; on Windows also Ctrl+Break, closing the console window, logging off or shutting
/// down. Windows gives a closing console a few seconds, which is enough to save the world and
/// remove the session marker.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match (
            signal(SignalKind::terminate()),
            signal(SignalKind::hangup()),
        ) {
            (Ok(mut term), Ok(mut hangup)) => {
                tokio::select! {
                    _ = term.recv() => {},
                    _ = hangup.recv() => {},
                }
            }
            _ => std::future::pending::<()>().await,
        }
    };
    #[cfg(windows)]
    let terminate = async {
        use tokio::signal::windows;
        match (
            windows::ctrl_break(),
            windows::ctrl_close(),
            windows::ctrl_logoff(),
            windows::ctrl_shutdown(),
        ) {
            (Ok(mut brk), Ok(mut close), Ok(mut logoff), Ok(mut shutdown)) => {
                tokio::select! {
                    _ = brk.recv() => {},
                    _ = close.recv() => {},
                    _ = logoff.recv() => {},
                    _ = shutdown.recv() => {},
                }
            }
            _ => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(any(unix, windows)))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let opened = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let opened = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let opened = std::process::Command::new("xdg-open").arg(url).spawn();
    if let Err(e) = opened {
        eprintln!("could not open a browser ({e}); open {url} yourself");
    }
}

fn new(args: NewArgs) -> anyhow::Result<ExitCode> {
    let layout = paths::locate(args.folders.content, args.folders.saves, None)?;
    let content = commands::load_content(&layout.content)?;
    commands::new_world(
        &content,
        &layout.saves,
        &commands::NewOptions {
            seed: args.seed,
            preset: args.preset,
            size: args.size,
            name: args.name,
            days: args.days,
            regime: args.regime,
            band: args.band,
            families: args.families,
            introduce: args.introduce,
        },
    )?;
    Ok(ExitCode::SUCCESS)
}

fn save_info(args: SaveArgs) -> anyhow::Result<ExitCode> {
    let layout = paths::locate(args.folders.content, args.folders.saves, None)?;
    let content = commands::load_content(&layout.content)?;
    commands::save_info(&args.file, &content)?;
    Ok(ExitCode::SUCCESS)
}

fn save_verify(args: SaveArgs) -> anyhow::Result<ExitCode> {
    let layout = paths::locate(args.folders.content, args.folders.saves, None)?;
    let content = commands::load_content(&layout.content)?;
    Ok(commands::save_verify(&args.file, &content))
}

fn run_notables(args: NotablesArgs) -> anyhow::Result<ExitCode> {
    use civ_host::{consistency, notables};
    let layout = paths::locate(args.folders.content, None, None)?;
    let content = commands::load_content(&layout.content)?;
    let threads = args.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map_or(1, std::num::NonZero::get)
            .min(4)
    });
    let runs = if args.calibrate {
        notables::CALIBRATION_RUNS
    } else {
        notables::RUNS
    };
    println!(
        "The notables' gate (ADR-0014 §4): {} fixture worlds at {}² cells, each lived its first \
         month by the minute and saved; from each, {runs} runs {} at Max to the end of year {}, \
         each with its own tie-break stream, on {threads} threads",
        consistency::FIXTURES.len(),
        consistency::SIZE,
        if args.calibrate {
            "with the tier only (calibration)"
        } else {
            "with the notables' tier and as many with every adult deliberating"
        },
        notables::LAST_YEAR,
    );
    let dir = tempfile::tempdir().map_err(|e| anyhow::anyhow!("no temporary folder: {e}"))?;
    let report = notables::run_gate(
        &content,
        notables::NotablesOptions {
            calibrate: args.calibrate,
        },
        dir.path(),
        threads,
    )
    .map_err(|e| anyhow::anyhow!(e))?;
    notables::print(&report);
    if let Some(path) = &args.json {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, json)
            .map_err(|e| anyhow::anyhow!("cannot write {}: {e}", path.display()))?;
    }
    Ok(if report.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn run_consistency(args: ConsistencyArgs) -> anyhow::Result<ExitCode> {
    use civ_host::consistency;
    let layout = paths::locate(args.folders.content, None, None)?;
    let content = commands::load_content(&layout.content)?;
    let threads = args.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map_or(1, std::num::NonZero::get)
            .min(4)
    });
    let runs = if args.calibrate {
        consistency::CALIBRATION_RUNS
    } else {
        consistency::RUNS
    };
    println!(
        "Gate B (ADR-0011 §5): {} fixture worlds at {}² cells, each lived by the minute to 1 \
         January of year {} and saved; from each, {runs} runs {} of that year, each with its own \
         tie-break stream, on {threads} threads",
        consistency::FIXTURES.len(),
        consistency::SIZE,
        consistency::RUN_YEAR,
        if args.calibrate {
            "in Detailed mode only (calibration)"
        } else {
            "in each mode"
        },
    );
    let dir = tempfile::tempdir().map_err(|e| anyhow::anyhow!("no temporary folder: {e}"))?;
    let report = consistency::run_gate(
        &content,
        consistency::ConsistencyOptions {
            calibrate: args.calibrate,
            approximations: civ_sim::Approximations {
                leisure_blocks: !args.without_leisure_blocks,
                household_view: !args.without_household_view,
            },
        },
        dir.path(),
        threads,
    )
    .map_err(|e| anyhow::anyhow!(e))?;
    consistency::print(&report);
    if let Some(path) = &args.json {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, json)
            .map_err(|e| anyhow::anyhow!("cannot write {}: {e}", path.display()))?;
    }
    Ok(if report.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn run_dashboard(args: DashboardArgs) -> anyhow::Result<ExitCode> {
    use civ_host::dashboard;
    let layout = paths::locate(args.folders.content, None, None)?;
    let content = commands::load_content(&layout.content)?;
    println!(
        "Sanity dashboard (plan §4.7): {} worlds of {} at {}² cells, each its first month by the \
         minute and then {} years a day at a time at Max, checked at every year's end",
        dashboard::WORLDS,
        dashboard::PRESET,
        args.size,
        args.years
    );
    if args.years < dashboard::YEARS {
        println!(
            "only {} of the dashboard's {} years: a quick look, not the dashboard",
            args.years,
            dashboard::YEARS
        );
    }
    println!("{}", dashboard::header());
    let report = dashboard::run(
        &content,
        dashboard::DashboardOptions {
            size: args.size,
            years: args.years,
        },
        args.keep_saves.as_deref(),
        &|world| println!("{}", dashboard::format_world(world)),
    );
    println!("{}", dashboard::format_rows(&report.rows));
    if let Some(path) = &args.json {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, json)
            .map_err(|e| anyhow::anyhow!("cannot write {}: {e}", path.display()))?;
    }
    let verdict = if report.passed() { "passed" } else { "FAILED" };
    println!(
        "{}; the dashboard {verdict} in {:.0} s",
        report.summary(),
        report.seconds
    );
    Ok(if report.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn run_smoke(args: SmokeArgs) -> anyhow::Result<ExitCode> {
    let layout = paths::locate(args.folders.content, None, None)?;
    let content = commands::load_content(&layout.content)?;
    println!(
        "Smoke seeds: {} preset(s) × {} seed(s) at {}² cells; thresholds: relief ≥ {} m, land ≥ \
         {:.0}%, gentle land ≥ {:.0}%, rivers ≥ {} km, exact save round trip, {} days of the \
         founding band",
        content.presets.len(),
        args.seeds,
        args.size,
        smoke::MIN_RELIEF_M,
        smoke::MIN_NOT_OCEAN * 100.0,
        smoke::MIN_GENTLE * 100.0,
        smoke::MIN_RIVER_KM,
        smoke::DAYS
    );
    if args.years > 0 {
        println!(
            "then {} years, checked at each year's end: nobody stuck, no population or land \
             problems, every good accounted for, at most {}× the founders, ≥ {:.0}% of households roofed from year 2, a \
             first trail in year 1, land claims that fit the regime, no field lost, wealth measures in range, the \
             techniques every founder brings known by ≥ {:.0}% of those old enough; at least half the bands keep {} \
             people. The worlds take the content's {} property regimes in turn. Each \
             economy is graded at the end (red fails): food stocks after the harvest, grain asked before it against \
             after, the Gini of goods from year {} (amber below {}), and workshop sizes (gray below {} workshops)",
            args.years,
            smoke::MAX_GROWTH,
            smoke::MIN_ROOFED * 100.0,
            smoke::MIN_UPBRINGING * 100.0,
            smoke::MIN_ALIVE,
            content.catalog.regimes.len(),
            civ_host::economy::GINI_FROM_YEAR,
            civ_host::economy::LOW_GINI,
            civ_host::economy::MIN_WORKSHOPS
        );
    }
    println!("{}", smoke::header());
    let results = smoke::run(
        &content,
        smoke::SmokeOptions {
            size: args.size,
            seeds: args.seeds,
            years: args.years,
        },
        &|result| println!("{}", smoke::format_result(result)),
    );
    let failed = results.iter().filter(|r| !r.failures.is_empty()).count();
    if let Some(summary) = smoke::economy_summary(&results) {
        println!("{summary}");
    }
    if !smoke::enough_alive(&results) {
        let long: Vec<_> = results.iter().filter_map(|r| r.living).collect();
        let alive = long.iter().filter(|&&n| n >= smoke::MIN_ALIVE).count();
        println!(
            "only {alive} of {} bands still live where they settled after {} years: FAILED",
            long.len(),
            args.years
        );
        return Ok(ExitCode::FAILURE);
    }
    if failed == 0 {
        println!("all {} smoke worlds passed", results.len());
        Ok(ExitCode::SUCCESS)
    } else {
        println!("{failed} of {} smoke worlds FAILED", results.len());
        Ok(ExitCode::FAILURE)
    }
}
