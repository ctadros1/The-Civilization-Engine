# Architecture and Data Flow

This guide follows the integrated repository at `main` commit [`eb00281`](https://github.com/ctadros1/The-Civilization-Engine/commit/eb00281). The kernel owns simulation state and decisions. The web observer renders that state and forwards user commands; it does not determine outcomes.

## Runtime shape

```text
Authored TOML packs ──► civ-content ──► ContentRegistry
                                           │
CLI / WebSocket / C ABI ─► civ-host ─► civ-sim ─► domain crates
                                  │        │
                                  │        ├─ save sections ─► commons-persist
                                  │        └─ state frames ──► civ-schema + commons-wire
                                  │                                  │
                                  └──── WebSocket ◄─────────────────┘
                                               │
                                       web observer
```

The shell starts `civ-host`, which loads and validates content, creates or restores a simulation, and exposes commands, queries, saves and observer frames. The same simulation composition can be driven headlessly by CLI commands or through the C interface. The current visual client is the local web observer; Unreal integration has not been built.

## Kernel ownership

| Area | Responsibility |
| --- | --- |
| `civ-core` | IDs, calendar, event/cadence scheduler and random streams |
| `civ-world` | Seeded terrain, drainage, lakes, rivers, navigation and terrain measures |
| `civ-grammar` | Pure expansion of a saved building design into geometry, parts, construction stages and needs |
| `civ-land` | Mutable landscape: habitat, weather, settlements, fields, plots, buildings and worn paths |
| `civ-agents` | People, households, goods, markets, firms, knowledge, ties, polities, law, crime, factions and movement |
| `civ-content` | Strict TOML parsing, cross-reference validation, stable diagnostics and content fingerprints |
| `civ-sim` | Composition root: world state, step order, save/load, and conversion to boundary payloads |
| `civ-host` | CLI, local observer server, commands, queries, smoke runs, dashboard and consistency gates |
| `civ-ffi` | Versioned C interface for embedding the same kernel in another host |

`commons-wire` defines the host-neutral frame envelope. `commons-persist` defines the checksummed snapshot container. Domain crates do not depend on generated FlatBuffers types: conversion happens at `civ-sim` and `civ-host` boundaries. This keeps the Rust model usable without the web client or Unreal.

The module organization follows state ownership. For example, `civ-agents/src/polity.rs` and `population/polity.rs` own institutional state and its update path; `crime.rs`, `population/crime.rs`, `population/cases.rs` and `population/watch.rs` own taking, evidence, cases, obligations and enforcement; `population/places.rs`, `moving.rs` and `founding.rs` own settlement knowledge and movement. `civ-sim/src/frames/` translates these facts into observer words and tables.

## Content-to-runtime boundary

Content packs define vocabulary and parameters: goods, recipes, activities, techniques, policies, offices, issues, norms, values and ideologies. References are resolved and constraints checked before a world starts. Unknown fields and invalid references are errors; runtime code receives a compiled registry rather than repeatedly parsing TOML.

The compiler keeps both a source-artifact fingerprint and a semantic fingerprint of compiled content. World metadata records the content identity that governed its simulation. New authored primitives pass through `civ-content` and are documented in [the content guide](../../content/README.md).

Authored rules do not script the order of history. A policy template can offer a levy, for example, but people weigh proposals and attendance, the polity's current rules decide them, and each household decides whether to pay. A regime label is a pure description of recorded institutions; simulation decisions do not read it.

## Observer and host flow

The web app starts from `web/src/main.ts`, keeps connection and message handling in `web/src/net/`, and renders typed state through panels and map layers. `civ-host` serves the static app and a same-origin localhost WebSocket. The handshake establishes protocol identity and the current world epoch. Commands request changes; queries request views. The host validates and dispatches them into the kernel.

The stream distinguishes replaceable snapshots from ordered records. A slow client may skip an obsolete snapshot and take the latest one. Events, command replies and errors retain order and correlation. A world epoch prevents late messages from an old world being applied after a create or load. Generated TypeScript is isolated behind the message adapter.

The frame is a view, not the authority. It may include derived settlement labels, standing summaries, observed market prices or weather panels; the kernel rebuilds derived facts as needed and accepts only explicit commands back from the observer.

## Save and resume flow

`civ-sim` serializes authoritative state into named sections. `commons-persist` wraps them in immutable, checksummed generations. A manual save or autosave creates a generation; on load, the reader verifies identity, version, bounds, checksums and invariants before restoring. Supported older versions use explicit migrations. Damaged or incompatible state is refused instead of repaired by guesswork.

Derived indexes and presentation tables are rebuilt from saved authoritative records. This matters for multiple settlements: residence events and settlement records are the source for population accounts; the account table itself is derived and checked against yearly changes. A snapshot is not an event-log replay, and the simulation does not promise that a seed alone recreates the same life.

## C interface and future hosts

`civ-ffi` exports `tce_get_api`, which returns a versioned table. A host creates a kernel instance, submits frames, polls ordered output, copies snapshots into caller-owned buffers, and destroys the instance. The interface avoids Rust-owned memory and catches panics at the boundary. The generated C header and harness test the layout expected by a future Unreal plugin.

The C interface is implemented, but the Unreal `EngineBridge`, terrain and building rendering, crowd, HUD and packaged client remain unimplemented. See [ADR-0005](../../decisions/0005-kernel-c-interface.md).

## Source map

- [Project layout and change rules](../../AGENTS.md)
- [Project plan and implementation status](../../PROJECT_PLAN.md)
- [Boundary schema decision](../../decisions/0001-boundary-schema.md)
- [Snapshot container decision](../../decisions/0002-snapshot-container.md)
- [C interface decision](../../decisions/0005-kernel-c-interface.md)
