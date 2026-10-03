# ADR-0001: Boundary schema: `commons-wire` envelope with FlatBuffers payloads

Status: Accepted
Date: 2026-10-03
Milestone: M0

## Context

Everything that crosses the kernel boundary (snapshot, delta and event frames going out; commands
and queries coming in) must use one versioned schema. That schema has to work over three
transports: in-process FFI (the Unreal plugin, M2), a localhost WebSocket (the web observer, M0),
and a recording file (later). It needs Rust, C++ and TypeScript readers (plan §3.1, §3.2).

The plan recommended FlatBuffers, but first asked M0 to check whether Genesis's `sim-protocol`
already covers this. Reports read: 01-07 (state streaming), 01-05 (persistence, for the FlatBuffers
evolution rules), 01-02 and 01-01 (boundary ownership).

What the Genesis check found (`crates/sim-protocol`, read at commit `fbce852`):

- It is a carefully validated, hand-written big-endian codec with a good envelope: magic,
  version, frame type, payload length, optional CRC, and bounds checks before allocation.
- Its frame bodies are Genesis-specific (organisms, pigment, energy, tiles of `(flags, food)`).
- It has no code generation. Every new message would need hand-written Rust, C++ and TypeScript
  decoders. Avoiding that is exactly the reason the plan gave for FlatBuffers.

So the envelope pattern generalizes; the bodies don't.

## Decision

1. **Envelope (`commons-wire`, engine-agnostic).** Every frame is a fixed 56-byte little-endian
   header followed by an opaque payload:

   | Offset | Size | Field |
   |---|---|---|
   | 0 | 4 | magic `CWIR` |
   | 4 | 1 | envelope version (1) |
   | 5 | 1 | frame kind |
   | 6 | 2 | flags (bit 0: payload CRC-32 present) |
   | 8 | 4 | schema tag (engine-chosen; TCE uses `TCE\0`) |
   | 12 | 2 | schema major |
   | 14 | 2 | schema minor |
   | 16 | 4 | world epoch |
   | 20 | 4 | payload length |
   | 24 | 8 | sequence |
   | 32 | 8 | correlation id |
   | 40 | 8 | simulation time (i64, engine-defined unit) |
   | 48 | 4 | payload CRC-32 (zero unless flag set) |
   | 52 | 4 | reserved, must be zero |

   Frame kinds and their delivery contracts:

   | Kind | Direction | Contract |
   |---|---|---|
   | Hello / Welcome | both | Handshake. A different schema **major** is refused; a different minor is accepted. |
   | Snapshot | out | Replaceable: a consumer may drop any snapshot older than the newest. |
   | Delta | out | Ordered within an epoch. A sequence gap means "request a snapshot". |
   | Events | out | Ordered within an epoch; gaps are detectable from the sequence. |
   | Command | in | Changes state. Carries a non-zero correlation id. |
   | Query | in | Reads state. Carries a non-zero correlation id. |
   | Response / Error | out | Answers the request with the same correlation id. An error with id 0 is unsolicited. |
   | Heartbeat | both | Liveness only. |

   **Epoch** increments whenever the authoritative world is replaced (new world, load). Consumers
   discard everything from an older epoch. **Sequence** is monotonic per (epoch, kind) on one
   stream. Decoders validate magic, version, kind, reserved bits and payload length against
   configured limits **before** allocating, and never partially apply a malformed frame.

2. **Payloads (TCE-specific): FlatBuffers.** Schemas live in `kernel/crates/civ-schema/schema/`.
   `flatc` generates Rust (`civ-schema`), TypeScript (`web/src/schema/generated/`) and, from M2,
   C++. The root table is determined by the envelope's frame kind. The `flatc` version is pinned
   (25.12.19, matching the Rust `flatbuffers` runtime). Generated code is committed, so a normal
   build never needs `flatc`; `tools/gen-schema.sh` regenerates it.

3. **Evolution rules** (enforced in review, and by `flatc --conform` against the last released
   schema when it runs):
   - Every table field has an explicit `(id: N)`. Never reuse or renumber an id.
   - Never change a field's type. Never remove a field: mark it `(deprecated)`.
   - Never reorder or renumber enum values or union members. Append only.
   - Structs are frozen once released: use a table if a shape might grow.
   - Additive changes bump the schema **minor**; anything else bumps the **major** and needs a new ADR.

## Consequences

- One schema change produces matching Rust, TypeScript and (later) C++ readers. The envelope is the
  only hand-written codec, and it is small enough to pin with golden-byte tests on both sides.
- Transports, recorders and viewers can route frames by kind and epoch without understanding TCE's
  payloads. That keeps multi-viewer, recording replay and an out-of-process move possible.
- FlatBuffers' Rust builder API is verbose; conversion code lives in `civ-schema` helpers so the
  kernel crates don't touch generated types.
- Generated files must never be hand-edited; CI checks that they match the schema.

## Alternatives considered

- **Lift Genesis `sim-protocol` into commons.** Lost because the bodies are engine-specific and
  hand-written per language. Its envelope design was adopted.
- **Cap'n Proto.** Comparable evolution story, weaker TypeScript tooling, and arena-list constraints.
  The research found no decisive advantage for TCE.
- **Protobuf.** No zero-copy reads, and protobuf's C++ runtime is notoriously awkward inside Unreal.
- **serde + MessagePack/JSON.** No schema-generated TypeScript/C++ readers, and JSON costs too much
  at 20–30 Hz for 50k agents.

## Revisit when

The M2 Unreal plugin finds FlatBuffers' C++ headers unworkable inside UE, or a second engine adopts
`commons-wire` and needs a different envelope field.
