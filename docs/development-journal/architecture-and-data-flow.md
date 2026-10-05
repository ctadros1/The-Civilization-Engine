# Architecture and Data Flow

TCE keeps one authoritative simulation kernel and multiple possible observers. At the documented baseline, the runnable observer is a local browser app. The C interface is implemented for a future Unreal host; the Unreal client itself is not.

## Runtime shape

```text
content/core/*.toml ──► civ-content ──► ContentRegistry
                                            │
NewWorld command ──► civ-host ──► civ-sim::Sim
                                     │       ├─ civ-core: ids, time, scheduler, RNG
                                     │       ├─ civ-world: seeded terrain and water
                                     │       ├─ civ-land: mutable land and buildings
                                     │       └─ civ-agents: people, decisions, work, economy
                                     │
                                     ├─ save sections ──► commons-persist ──► saves/<world>/
                                     └─ frames ──► commons-wire + FlatBuffers
                                                       ├─ WebSocket ──► TypeScript/PixiJS observer
                                                       └─ C ABI ──► future Unreal plugin
```

The web and future Unreal surfaces are clients. They can ask questions and submit commands, but they do not decide what people do or own a second copy of the world rules. `civ-sim::Sim` is the composition root: it owns the clock, scheduler, metadata, terrain reference, mutable land, population and content stamp.

## Crate responsibilities

| Crate or package | Responsibility |
| --- | --- |
| `civ-core` | Simulation time, calendar, entity handles, permanent IDs, keyed RNG and event/cadence scheduler |
| `civ-world` | Pure seeded terrain, hydrology, lakes, rivers, terrain metrics and navigation grids |
| `civ-grammar` | Pure expansion from a saved building design to geometry marks, components, spaces and staged material/labor needs |
| `civ-land` | Mutable land: habitat stocks, settlements, fields, plots, buildings, worn ground and the started deposit placement primitive |
| `civ-agents` | People, households, needs, choices, activities, trips, births, deaths, work, goods, markets, firms, knowledge and building behavior |
| `civ-content` | Strict TOML parsing, cross-reference and range validation, diagnostics, and immutable compiled registry |
| `civ-schema` | FlatBuffers boundary/save definitions and generated Rust types |
| `civ-sim` | World composition, operations, boundary payload construction and save/load migrations |
| `civ-host` | Command line, worker thread, localhost WebSocket server, autosave/recovery and smoke checks |
| `civ-ffi` | One exported entry point returning the versioned C function table for an external host |
| `web/` | Browser observer: map, panels, dialogs, state and network client |
| `commons/` | Engine-neutral frame envelope, checksummed snapshot container, and header-only C++ readers |

The dependency structure is layered rather than one strict chain. `civ-core`, `civ-world` and `civ-grammar` provide foundations used by `civ-land`; `civ-agents` builds on the living world; `civ-content` compiles authored definitions; and `civ-sim` composes the domain model. `civ-host` and `civ-ffi` expose it to clients. Domain crates avoid generated FlatBuffers types. Translation into wire/save formats happens at the composition and host boundaries.

## Starting and creating a world

1. `tools/run.sh` (or `tools/run.ps1`) locates the repository, builds the web shell if needed, builds `civ-host`, and serves both the static observer and `/ws` from localhost.
2. The observer sends `NewWorld` with a seed, preset, size, name and optional founding-band/regime choices.
3. The host loads and validates the TOML content pack into a `ContentRegistry`. Invalid content yields diagnostics and cannot produce a registry.
4. `civ-sim` invokes `civ-world` to create terrain and hydrology from the seed and preset. World generation is pure for a given build and input. The resulting map is retained; ongoing simulation does not regenerate it.
5. `civ-land` and `civ-agents` initialize mutable stocks, the settlement and its people. The kernel publishes initial metadata, snapshot and event frames for observers.

New worlds use 8 m simulation cells. The default is 2,048 cells per side (16 km); smaller sizes support rapid iteration. The clock begins on 1 March, year 1, at 06:00. Generation parameters and content fingerprints are kept with world metadata; the generated terrain itself is saved.

## Running and exchanging data

The host owns the simulation worker. While running, `civ-host::Engine` receives commands and queries, advances `Sim`, schedules autosaves, and publishes updates. A user command changes kernel state only after validation and execution in the host/kernel path.

The observer uses one TypeScript network module to encode commands and queries, decode responses, and maintain a client state store. The map and panels display received facts. They do not invent prices, household decisions, field states or building dimensions. Pure display helpers turn kernel facts into labels and shapes.

Frame delivery has two paths:

- **Web:** `civ-host` accepts a local WebSocket connection from the same origin. A handshake establishes schema identity and a world epoch. Snapshot frames are replaceable; ordered events, replies and errors use the frame stream.
- **C ABI:** `civ-ffi` exposes `tce_get_api`, which returns a versioned table. Callers submit byte frames, poll ordered frames, and copy the newest snapshot into their own buffers. The interface does not lend Rust memory or call back into the host.

The C ABI lets a future Unreal plugin load and drive the kernel. It does not mean the plugin, terrain renderer, building assembler, HUD or packaged Unreal build exists yet. See [ADR-0005](../../decisions/0005-kernel-c-interface.md).

## Content-to-runtime boundary

Authored content is divided into files by kind and stable IDs of the form `pack:kind/name`. The compiler rejects unknown/missing fields, duplicate IDs, invalid references, out-of-range parameters and invalid cross-file combinations. Compiled content is immutable during a world run.

The registry records two BLAKE3 fingerprints: a byte-level pack fingerprint and a semantic fingerprint over compiled behavior. The latter is stamped into saves, so changing whitespace does not count as changing the rules. World parameters such as terrain shape, people profiles, goods, recipes, property regimes, building programs and techniques remain reviewable in TOML rather than hidden in a viewer.

## Ownership rule

When a feature is added, place its authoritative rules in the domain crate that owns them. Let `civ-sim` compose the model and handle persistence/boundary conversion. Let `civ-host` expose the model, and let `web/` render and forward it. This rule keeps the browser and any future Unreal client as views of one simulation, not separate implementations.
