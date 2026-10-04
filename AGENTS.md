# Agent Operating Contract

This file is authoritative for coding agents working in this repository. Explicit current
instructions from the project owner override it; record the change in `PROJECT_PLAN.md` §9.

## Reading order

Before changing anything:

1. `README.md`: what exists now, and what is planned.
2. `PROJECT_PLAN.md`: the plan of record. §1 (rules), §7 (the current milestone), §9 (decisions).
3. `decisions/`: the ADRs. They govern the boundary schema, saves and other choices that are
   expensive to reverse.
4. `research/README.md` and the research reports for the area you are touching (the index maps
   reports to milestones). Read them **before** designing, not after.
5. The code and tests you are about to change. Run the relevant tests first.

## Non-negotiable rules (plan §1)

1. **Author the vocabulary, never the plot.** Content defines primitives: presets, goods,
   offices, techniques. Nothing scripts events, eras, storylines or outcomes. "Monarchy" or
   "Gothic" are labels inferred after the fact; they are never engine states. If you can name the
   outcome a mechanism makes more likely, it is plot. Remove it or make it a primitive.
2. **Usable at every milestone.** A milestone ends in a build a non-developer can launch,
   create a world in, watch, save, load and quit without data loss. Every panel has empty,
   loading and error states.
3. **Emergence is time-boxed.** After about a week of tuning without the targeted behaviour,
   author a nudge, log it in §9 as `NUDGE:`, and move on.
4. **Realism means plausible, not proven.** Check against a short list of stylized facts.
   Do not run preregistered campaigns or statistical studies.
5. **The kernel owns truth.** The web observer, and later Unreal, draw and forward commands.
   They decide nothing.
6. **Determinism is not a goal.** Saves are snapshots. World *generation* is reproducible from a
   seed on one build; the simulation need not be.
7. **ADRs only for decisions that are expensive to reverse.** Write about three per
   milestone at most.

## Layout

| Path | What |
|---|---|
| `kernel/` | The Rust workspace (no Unreal dependency, ever). |
| `kernel/crates/civ-core` | Ids, handles, the 365-day calendar, the event-and-cadence scheduler, RNG. |
| `kernel/crates/civ-world` | World generation: terrain, lakes, rivers. Pure functions of their inputs. Also walking (`nav`) and terrain measures (`terrain`). |
| `kernel/crates/civ-grammar` | Building grammars: a pure expansion of a saved design into parts, outline and per-stage needs, with golden hashes (ADR-0004). M1 has the hut. |
| `kernel/crates/civ-land` | Land that changes: habitat patches, wild stocks, the climate year, settlements, fields, plots, buildings, and the ground worn by walking with the trails traced through it (`paths`) (ADR-0004). |
| `kernel/crates/civ-agents` | People: needs, decisions, activities, trips, households, founding bands, births, deaths and couples (`demography.rs`, `population/life.rs`), history (ADR-0003); recipes, tools and skills as arithmetic on stores (`make.rs`, ADR-0006); the ledger that moves goods between households by channel (`ledger.rs`, `population/transfer.rs`), what goods are worth in hours of a household's work (`value.rs`) and markets: posted terms, trades, acceptance, the inferred money (`market.rs`, `population/market.rs`). |
| `kernel/crates/civ-content` | The content compiler: TOML packs, stable diagnostics, fingerprints. |
| `kernel/crates/civ-schema` | FlatBuffers schemas and generated Rust (boundary and saves). |
| `kernel/crates/civ-sim` | The composition root: a world's state, save/load, boundary payloads (`frames/`, where the kernel puts fields, buildings, markets and trades in words). |
| `kernel/crates/civ-host` | The command line and the localhost observer server. |
| `kernel/crates/civ-ffi` | The kernel's C interface: the `tce_kernel` library Unreal loads, with its generated header `include/tce_kernel.h` (ADR-0005). |
| `commons/` | `engine-commons`, staged in-repo: `commons-wire` (frame envelope), `commons-persist` (snapshot container), and `cpp/` (the envelope and timed paths in header-only C++ for C++ hosts, tested by `crates/commons-cpp`). Engine-agnostic. |
| `content/` | Authored packs (`content/core`). See `content/README.md`. |
| `web/` | The web observer (TypeScript, Vite, PixiJS). See `web/README.md`. |
| `decisions/` | ADRs. |
| `research/` | Deep-research reports, indexed by milestone. |
| `tools/` | `run.sh` / `run.ps1` (launch), `gen-schema.sh` (FlatBuffers codegen). |

Dependency direction: `civ-core` ← `civ-world` ← `civ-grammar` ← `civ-land` ← `civ-agents` ←
`civ-content` ← `civ-sim` ← `civ-host` ← `civ-ffi`. `civ-grammar` depends on nothing, so Unreal can call the
same expansion over FFI (plan §3.4).
Domain crates (`civ-core`, `civ-world`, `civ-grammar`, `civ-land`, `civ-agents`) never use generated schema
types. Conversion happens only in `civ-sim` (world payloads, save sections) and `civ-host`
(session payloads); see ADR-0001.

## Commands

```sh
# Kernel (from kernel/)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --profile simcheck     # how CI runs tests
cargo run --release -p civ-host                        # serve the observer on 127.0.0.1:7420
cargo run --release -p civ-host -- content validate [--json]
cargo run --release -p civ-host -- smoke               # the smoke seeds; must pass
cargo run --release -p civ-host -- smoke --years 10    # and ten years of each (nightly; ~15 min on 4 cores)
cargo run --release -p civ-host -- run --seed 2 --years 5   # one world, reported each year (calibration)
cargo run --release -p civ-host -- new --seed 7 --size 1024
cargo run --release -p civ-host -- save info|verify <file>

# Commons (from commons/): the same fmt, clippy and test commands.

# Web (from web/)
npm ci && npm run build && npm test
npm run test:e2e                                       # needs dist/ and a built civ-host

# The kernel library for Unreal (ADR-0005): build, check the header, publish to dist/
tools/build-kernel-dll.ps1     # Windows, with the PDB; tools/build-kernel-dll.sh elsewhere

# Generated schema code: Rust, TypeScript and C++ (needs flatc 25.12.19)
tools/gen-schema.sh            # regenerate
tools/gen-schema.sh --check    # what CI runs
```

`.github/workflows/ci.yml` runs all of this. The Windows lane covers the kernel, commons,
content and smoke seeds; a second Windows job builds the kernel library and keeps it as an
artifact; Ubuntu lanes cover the web shell and schema freshness. Keep it green.
`.github/workflows/nightly.yml` runs the ten-year smoke seeds nightly on `main` and on pull
requests into it; run them yourself before a change to how people live, farm, build or die.

## Rules for changes

- **Boundary schema** (`civ-schema/schema/tce_wire.fbs`, ADR-0001): explicit field ids, append
  only. Never renumber, retype or remove a field; deprecate instead. Regenerate Rust and
  TypeScript with `tools/gen-schema.sh` and commit the result. Never edit generated files.
- **Saves** (ADR-0002): refusal, never repair. A change to a section's meaning bumps its
  version or `SAVE_SCHEMA_VERSION` and needs an explicit migration. The save → load → save
  test (identical raw digests) must stay exact. Saves store authoritative state only; anything
  derived is rebuilt on load.
- **World generation:** anything that changes what a seed produces bumps
  `civ_world::GENERATOR_VERSION`. Generation parameters are content (`content/core/worldgen`),
  not code constants.
- **Content:** new kinds of authored primitives go through `civ-content`. They get strict TOML
  (unknown fields are errors), stable diagnostic codes, and documentation in `content/README.md`.
- **Rust:** edition 2024, toolchain pinned by `rust-toolchain.toml`. `unsafe` is forbidden
  everywhere except the generated FlatBuffers module in `civ-schema` and the C interface in
  `civ-ffi` (`src/exports.rs`, and the tests that call it as a host would), where every `unsafe`
  block has a `SAFETY:` comment. `clippy -D warnings` must pass. Avoid `unwrap` outside tests.
  Doc comments are short, factual and in plain English. Never set `panic = "abort"`: the C
  interface catches panics.
- **C interface** (`civ-ffi`, ADR-0005): one export, `tce_get_api`, and a versioned table.
  Within ABI 1, only append functions to the table and fields to structures. After changing
  `src/exports.rs`, regenerate the header with `TCE_BLESS=1 cargo test -p civ-ffi --test header`
  and commit it; the C harness test checks that C and Rust agree on every layout.
- **Web:** the page shows state and forwards commands, and never decides. Generated TypeScript
  stays behind `src/net/messages.ts`. New panels need empty, loading and error states.
- **Claims:** do not call something done, fixed or fast without having run it. Keep the
  implemented-versus-planned table in `README.md` honest.
- **Git:** small commits with descriptive messages. Never commit `saves/`, `target/`,
  `node_modules/`, `web/dist/` or test output.

## Before you commit

1. The kernel and commons pass fmt, clippy and tests, and the smoke seeds pass if you touched
   generation, saves or the host.
2. The web shell builds, and its unit tests pass. Run the browser tests if you touched the
   protocol, the host or the shell.
3. `tools/gen-schema.sh --check` passes if you touched a schema.
4. Update `README.md` (status) and `PROJECT_PLAN.md` (status and decisions) when behaviour or
   decisions change.
