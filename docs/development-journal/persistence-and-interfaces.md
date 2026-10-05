# Persistence and Interfaces

TCE treats boundary messages and durable saves as long-lived contracts. Their versioning, ownership and error behavior are deliberate parts of the architecture, not implementation details to change casually.

## Content loading

`civ-content` reads the `content/` TOML pack into strict typed definitions. A content ID names its pack, kind and item; file paths, identity fields, references, numeric ranges, techniques and recipe/bootstrap constraints are checked before an immutable `ContentRegistry` is produced. Diagnostics have stable codes and file locations for command-line and agent tooling.

The registry keeps an artifact fingerprint over source bytes and a semantic BLAKE3 fingerprint over effective compiled content. World metadata records the pack identities and content stamp. A save can therefore explain which rules it used, while formatting-only edits need not count as semantic changes.

## Network and host boundary

`commons-wire` defines an engine-neutral binary frame envelope. At this baseline its envelope major is 1; TCE payload schemas are FlatBuffers generated from `kernel/crates/civ-schema/schema/`. Each frame identifies the schema, frame kind, world epoch and ordering/correlation information needed by its delivery class.

The main payload flow is:

| Direction | Frame/payload purpose | Delivery behavior |
| --- | --- | --- |
| Host → observer | `Welcome`, world `Snapshot`, `Events`, query `Response`, `Error` | `Welcome` initializes the client; ordered frames are not silently replaced |
| Observer → host | `Hello`, `Command`, `Query`, `Heartbeat` | Commands change state; queries read state; correlation IDs match replies |

Snapshots are replaceable state: a slow client can skip an older snapshot and copy a newer one. Events and replies are ordered records, so they are not treated as replaceable state. World epochs separate a newly created/loaded world from messages belonging to a previous world. The host validates frame structure, schema identity and requests before dispatch.

`civ-host` serves the browser files and WebSocket on localhost. It rejects pages from other origins and can require a token. The web network layer centralizes frame handling in `web/src/net/messages.ts` and `web/src/net/client.ts`; generated TypeScript is kept behind the messages module. Display components operate on plain typed data.

The schema is append-only within a major version. New fields and enum values are added without renumbering old fields. `tools/gen-schema.sh` regenerates Rust, TypeScript and C++ readers from the pinned FlatBuffers compiler, and CI checks that generated files are current. See [ADR-0001](../../decisions/0001-boundary-schema.md).

## Save format and lifecycle

`commons-persist` stores named sections in a versioned snapshot container. The format carries an engine identity, container and state-schema versions, world/snapshot identities, section descriptors, lengths and checksums. Section payloads are compressed where configured. Readers enforce size limits and verify data; a damaged or incompatible file is refused rather than silently repaired.

`civ-sim` maps authoritative world state into sections such as metadata, clock/scheduler, terrain and water, land, agents, buildings, economy and knowledge. State that can be derived from saved records (for example terrain indexes and certain presentation summaries) is rebuilt rather than independently persisted. Save code owns migration from supported older schemas; a meaning change requires a schema version change and an explicit migration.

Each manual save or autosave publishes a new immutable generation in `saves/<world>/`. A save is not an event-log replay: it is a snapshot of current state. TCE does not promise deterministic simulation replay from a seed. World generation is reproducible for a given build and input; the subsequent simulation is not required to reproduce bit for bit.

The host autosaves changed worlds at the configured cadence and before replacement/exit. It prunes old autosave generations according to policy and keeps a session marker pointing at the last good save. On restart, an interrupted session can offer recovery from that save. Crash snapshots used for diagnosis are not offered as normal recovery saves.

The central persistence rule is **refusal, never repair**. Save → load → save checks compare raw section digests for supported states. If a format or semantic invariant is incompatible, the system should report it rather than guessing how to reconstruct intent. See [ADR-0002](../../decisions/0002-snapshot-container.md).

## C interface for external hosts

`civ-ffi` exports one C symbol, `tce_get_api`, which returns a versioned table of function pointers plus version information. The host creates one kernel, submits `commons-wire` frames, polls ordered frames, copies the latest snapshot into a caller-owned buffer, optionally serves the observer panels, and destroys the kernel.

The boundary avoids Rust-owned memory, callbacks into the host and engine-specific types. Structs include sizes for append-only extension. Functions catch panics at the exported boundary and report status codes; a panic faults that kernel instance rather than unwinding into C/C++. Polling and submission are non-blocking interfaces to the engine worker. The C harness exercises the same ABI that a future plugin will load.

The interface and Unreal integration are separate milestones: the kernel library and C contract exist, but this baseline has no Unreal `EngineBridge` plugin or packaged UE client. See [ADR-0005](../../decisions/0005-kernel-c-interface.md).

## Schema-change checklist

When a feature changes durable state or a message:

1. Decide which crate owns the new authoritative fact.
2. Update the FlatBuffers schema, never its generated output by hand.
3. Add conversion in the composition/host boundary, not in domain crates.
4. Append wire fields and bump the minor schema version; never renumber/remove/retype an existing field.
5. For saves, bump the state schema when section meaning changes and implement a migration for supported older versions.
6. Update save round-trip, schema freshness, C layout or browser protocol checks as applicable.
7. Update the root README, the plan's status/decision log and the relevant journal page.

The project's [AGENTS.md](../../AGENTS.md) contains the full change contract and required commands.
