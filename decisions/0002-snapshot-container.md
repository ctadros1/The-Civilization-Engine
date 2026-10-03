# ADR-0002: Snapshots: `commons-persist` chunked container with FlatBuffers sections

Status: Accepted
Date: 2026-10-03
Milestone: M0

## Context

Saves are snapshots (plan §1 rule 6, §3.5). Determinism is not a goal, so there is no input log and
no exact replay; a save is the only durable description of a world. Worlds are endless and will
reach gigabytes. The plan asks for `commons-persist` to provide magic, container and schema
versions, named sections, per-section and whole-state checksums, a content-pack fingerprint, and
refusal (never repair) on mismatch, matching the posture of Prometheus's `sim-persist`.

Reports read: 01-05 (persistence and schema evolution), 01-10 (content fingerprints), 01-11
(testing). Prometheus `engine/crates/sim-persist` was read at commit `54c50f3` for posture.

The persistence research's main points:

- Use a thin container plus an evolvable schema.
- Compress each chunk independently with zstd level 1, using 64-bit offsets.
- Publish immutable generations; never overwrite the only copy.
- Make migrations explicit.
- Avoid `bincode`, which RustSec marks unmaintained (RUSTSEC-2025-0141).

## Decision

1. **Container (`commons-persist`, engine-agnostic).** One file per generation, little-endian:
   - A **192-byte header**:
     - Magic `COMPSNAP` and container version.
     - Engine tag and engine schema version.
     - World id, snapshot id and parent snapshot id.
     - Generation number, creation time and simulation time.
     - A 32-byte content fingerprint and a 48-byte label.
     - An xxh3-64 digest of the header itself.
   - **Chunks**, each independently stored raw or zstd-compressed.
   - A **manifest** of 64-byte entries, one per chunk: section tag (8 ASCII bytes), chunk index,
     section version, codec, 64-bit offset, stored and raw lengths, and xxh3-64 digests of both
     the stored and the raw bytes.
   - A **48-byte footer**:
     - Manifest offset and length, and the total file length.
     - The manifest digest and a whole-body digest.
     - Closing magic `COMPEND1`.

   A file whose footer is missing or wrong is incomplete and is never loaded.

2. **Refusal, never repair.** The reader refuses wrong magic, an unsupported container version,
   header/manifest/body digest mismatches, out-of-bounds offsets, chunks whose declared sizes
   exceed limits, and decompression or digest failures. The engine refuses unknown schema
   versions unless an explicit migration exists. A **content-fingerprint** difference is *not*
   a refusal: the plan says such saves still load when the schema is compatible, and the save
   browser notes that the content changed.

3. **Publication.** Write to a uniquely named temp file in the destination directory, flush, then
   `sync_all`. Next, rename with no-clobber to a **new** generation file name, then sync the
   directory where the OS allows it. Existing generations are never overwritten. A "latest"
   pointer is only a convenience: recovery scans and validates generations. Autosaves rotate
   (keep the last N); manual saves and crash snapshots are never pruned automatically.

4. **Section payloads (TCE-specific): FlatBuffers tables**, from `civ-schema/schema/tce_save.fbs`,
   with the same evolution rules as ADR-0001. Large rasters are split into tiles so that every
   chunk stays within about 1–16 MiB. A save stores **authoritative state only**. Derived caches
   are rebuilt on load (flow accumulation from receivers, presentation shading, grammar geometry).
   The test for "derived" is that rebuilding preserves the model, not merely that code calls it a cache.

5. **Versions are independent.** There are separate versions for the container, the engine schema,
   each section, and the content fingerprint. A compression change needs no world migration.
   Migrations are explicit, ordered functions from old section versions to new ones. They run
   before runtime tables are built, and their output is always written as a **new** generation.

6. **Round-trip is a test, not a hope.** Every section encoder has a decode test. The world-level
   test is save → load → save, where the second save's raw section digests must equal the first's.

## Consequences

- Saves can be inspected and verified without the simulation (`civ-host save info|verify`).
  Corruption is reported per chunk.
- A save in progress never damages an existing save. A crash mid-write leaves an orphan temp
  file, which is ignored.
- Background autosave by copy-on-write is not needed in M0, because terrain is immutable after
  generation and shared by `Arc`. It becomes necessary once agents exist (M1+). The container
  already supports it: chunks are independent.
- zstd is a C dependency (`zstd-sys`). It builds on Windows MSVC and Linux; Miri cannot run it.

## Alternatives considered

- **bincode or another serde binary format.** Unmaintained, or bound to Rust struct layout, which
  would turn every refactor into a format change.
- **rkyv as the durable format.** Archive compatibility is coupled to type layout and feature
  flags. Good for rebuildable caches later, not for permanent saves.
- **SQLite chunk store.** A credible later option once saves need incremental history. Overkill
  for complete self-contained generations.
- **One compression stream for the whole file.** Lost because it rules out bounded-memory
  decoding, parallel work and precise corruption reports.

## Revisit when

A save exceeds roughly 1 GB and full rewrites cost too much (consider chunk reuse across
generations), or autosave latency becomes visible once agents exist.
