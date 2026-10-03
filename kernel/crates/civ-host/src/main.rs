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
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
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
             problems, at most {}× the founders, ≥ {:.0}% of households roofed from year 2, a \
             first trail in year 1; at least half the bands keep {} people",
            args.years,
            smoke::MAX_GROWTH,
            smoke::MIN_ROOFED * 100.0,
            smoke::MIN_ALIVE
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
