# Persistence and Interfaces

This describes the integrated repository at `main` commit [`eb00281`](https://github.com/ctadros1/The-Civilization-Engine/commit/eb00281). Current versions are TCE wire **1.53**, save schema **55**, and content API **55**. They are the versions in this source snapshot and can advance on later commits.

## Content loading

`civ-content` reads the `content/` TOML packs into strict typed definitions. IDs name a pack, kind and item. The compiler checks file identity, references, numeric ranges, activity and recipe constraints, policy vocabulary and dependencies before constructing an immutable `ContentRegistry`. Stable diagnostics include codes and source locations for command-line and agent tooling.

The registry retains an artifact fingerprint over the source files and a semantic fingerprint over compiled content. World metadata keeps content identity so a save can report the rules it used. Formatting-only source changes need not alter the semantic fingerprint.

## Wire envelope and payloads

`commons-wire` defines an engine-neutral binary envelope; `civ-schema` defines FlatBuffers payloads. The observer and host use the same versioned messages. The envelope records the payload schema, message kind, world epoch and ordering/correlation data needed for the frame's delivery class.

| Direction | Payloads | Semantics |
| --- | --- | --- |
| Host to observer | `Welcome`, `Snapshot`, `Events`, query `Response`, `Error` | Initial state, replaceable current view, ordered records and replies |
| Observer to host | `Hello`, `Command`, `Query`, `Heartbeat` | Connection, explicit state-changing intent, read request and liveness |

Snapshots can be superseded by newer snapshots. Events and command/query replies are ordered and correlated. A new world epoch separates messages for a loaded or newly created world from stale messages of the prior world. `civ-host` validates frames and requests before dispatch.

Wire changes are append-only within a major version: append fields and enum values, never renumber, retype or remove existing fields. `tools/gen-schema.sh` generates Rust, TypeScript and C++ readers from the pinned FlatBuffers compiler; CI checks generated files against the schemas. The web adapter keeps generated TypeScript behind `web/src/net/messages.ts`.

## Snapshots and migration

`commons-persist` stores named sections in a versioned, checksummed container. It records engine and schema identity, world and snapshot IDs, lengths and section checksums, and applies configured compression. Readers enforce limits and reject corrupt or incompatible data.

`civ-sim` maps authoritative state into sections for the clock and scheduler, terrain and land, households and people, buildings, economy, knowledge, institutions, social memory, settlements and residence histories. The sections expand as new facts become authoritative. Derived indexes, market summaries and annual settlement accounts are reconstructed from their source records rather than persisted as competing truth.

Supported older saves use explicit migrations. Save-schema meaning changes require a version bump and migration. The policy is **refusal, never repair**: if identity, checksum, version or invariants fail, the loader reports the problem rather than guessing at intent. Save → load → save tests compare raw section digests.

Snapshots preserve state, not a deterministic event replay. World generation is reproducible from its seed and inputs on one build; the simulation run is not promised to repeat. Continuing a saved world preserves the same future state as continuing without the save at the supported boundary, as checked by Gate A ([ADR-0011](../../decisions/0011-execution-modes.md) §5).

## C interface

`civ-ffi` exports one symbol, `tce_get_api`, which returns a versioned table of function pointers. A host creates a kernel, submits `commons-wire` frames, polls ordered output, copies the latest snapshot into caller-owned memory, optionally serves observer panels, and destroys the kernel. The interface avoids Rust-owned buffers and catches panics so they cannot unwind through C/C++.

The generated header and C harness check ABI layout and runtime framing. This boundary is implemented for future Unreal integration, but the Unreal plugin and packaged client are not. See [ADR-0005](../../decisions/0005-kernel-c-interface.md).

## Cross-boundary feature path

Features with persisted observer state usually cross these layers:

1. A domain crate owns the state and rules.
2. `civ-sim` composes the domain behavior and maps authoritative state to save sections.
3. `civ-schema` appends wire payload fields; `civ-sim/src/frames/` builds the observer representation.
4. `civ-host` validates commands and queries and dispatches them.
5. `web/src/net/messages.ts` decodes messages; panels render state and send commands.
6. Content additions go through `civ-content`; generated schema artifacts are regenerated from the source schema.

M4 illustrates the path: `civ-agents` owns polities, laws and claims, save sections preserve their durable histories, `frames/government.rs` and `frames/order.rs` expose kernel-authored views, and the observer's Government and Takings panels render them. M5a extends the same pattern with settlement identities, residence histories, places and movement; its annual population accounts are derived from those histories.

See [AGENTS.md](../../AGENTS.md) for the complete schema and save change checklist, and [architecture and data flow](architecture-and-data-flow.md) for the process boundaries.
