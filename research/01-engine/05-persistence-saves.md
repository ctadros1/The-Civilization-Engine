# Save/load and schema evolution for TCE

**Engineering report — sources checked through September 27, 2026**

## Executive recommendation

For TCE, I recommend **versioned, immutable, chunked snapshots**, with **FlatBuffers as the durable schema format**, **independent Zstandard compression per chunk**, **application-level copy-on-write for background autosaves**, and **explicit migrations into a storage-independent world model**.

Keep persistence inside the Rust kernel. Unreal should request saves, display progress, and reconstruct presentation objects after loading—not serialize the authoritative civilization.

The central design principle is:

> **A save is a durable description of a world, not a dump of the current program’s memory.**

Separate four responsibilities: obtaining a consistent snapshot, encoding it, publishing it safely, and interpreting it after years of changes. No serialization library solves all four.

Start with complete, self-contained save generations rather than a custom incremental database. Design chunk identities and history partitions now so unchanged-data reuse can be added later. Retain several validated generations and never migrate the only copy of a world in place.

Two current findings materially affect the choice. RustSec marks **bincode unmaintained**, and its published 3.0.0 is a discontinuation notice rather than a usable successor codec. Separately, **UE 5.8’s `AsyncSaveGameToSlot` still serializes on the game thread**; only its platform write runs on a worker. Neither is a suitable default foundation for TCE’s long-lived, multi-gigabyte saves. [RustSec](https://rustsec.org/advisories/RUSTSEC-2025-0141.html)

---

# 1. Options: the main techniques

## 1.1 Binary formats

A **container** and a **serialization format** are different layers. The container handles chunk boundaries, compression, checksums, indexing, and commit metadata. FlatBuffers, Cap’n Proto, or another codec describes the contents of those chunks.

### FlatBuffers

FlatBuffers stores offset-addressed data that generated accessors can inspect without first constructing a native object graph. Its critical distinction is between **evolvable tables** and **fixed-layout structs**. Tables support missing fields and schema extension; structs should be reserved for genuinely stable layouts. [FlatBuffers](https://flatbuffers.dev/internals/)

The evolution rules are strict: append table fields unless all fields have explicit IDs; do not remove or reuse old fields; preserve defaults and existing enum/union identities. Renaming a field generally does not change its binary representation. The compiler’s `--conform` check can catch incompatible schema edits. [FlatBuffers](https://flatbuffers.dev/evolution/)

**Fit for TCE:** strong. Use tables around columnar vectors—for example, a population chunk containing parallel vectors of persistent IDs, birth dates, household IDs, and attributes. This avoids forcing every person into a deeply nested table hierarchy.

**Main limitations:** building buffers costs CPU and temporary memory; table-heavy layouts can have substantial overhead. The Rust `FlatBufferBuilder` documents a **2 GiB buffer limit**, so a 5 GB world must not be one FlatBuffer. Its Rust documentation also retains experimental API/verification caveats: pin the compiler, generated code, and runtime together rather than assuming every language binding has identical maturity. [Docs.rs](https://docs.rs/flatbuffers/latest/flatbuffers/)

### Cap’n Proto

Cap’n Proto also supports direct access, but uses a word-aligned, segmented representation. Its schema ordinals identify fields independently of source-code ordering, and its documented evolution rules permit compatible additions while requiring existing ordinals and identities to remain stable. [Cap'n Proto](https://capnproto.org/encoding.html)

It is a credible alternative to FlatBuffers, especially for structured messages. However, arena-backed lists are not general-purpose resizable runtime collections: their sizes must be known when allocated. “Packing” improves size but adds an unpacking step before ordinary direct access. Its reference implementation has substantial production use in Cloudflare’s infrastructure. [Cap'n Proto](https://capnproto.org/faq.html)

**Fit for TCE:** good, but not decisively better. I would choose it over FlatBuffers only after a representative prototype demonstrated substantially simpler code or better end-to-end performance. Its RPC machinery is unnecessary for a local save system.

For large inputs, retain traversal and nesting limits rather than disabling validation globally. Rust’s reader options explicitly guard against repeated-reference amplification and excessive nesting. [Docs.rs](https://docs.rs/capnp/latest/capnp/message/struct.ReaderOptions.html)

### rkyv

rkyv creates archived Rust representations that can be accessed without conventional deserialization. This is not simply writing native Rust memory to disk.

Its documented compatibility conditions are narrower than a tagged, evolvable schema: the underlying schema must remain unchanged, format-control features must remain unchanged, and the producing version must be semver-compatible. Endianness, alignment, and pointer-width settings can affect the archive format. Validation is available and enabled through the default `bytecheck` feature. [Docs.rs](https://docs.rs/rkyv/latest/rkyv/)

**Fit for TCE:** excellent candidate for **rebuildable caches**, frozen data products, or tightly controlled checkpoints. Less attractive as the sole permanent world format.

It can support long-lived saves by keeping old archived types and conversion code. The objection is not impossibility; it is that ordinary Rust refactoring becomes coupled to archive compatibility unless persistence types are rigorously separated.

### bincode

Bincode is straightforward: encode typed Rust values into a compact binary stream, then decode them into owned or borrowed values. Struct fields are represented in order rather than by self-describing field names, so layout changes require version-aware decoding. Integer encoding and endianness are also part of the format configuration. [Docs.rs](https://docs.rs/bincode/2.0.1/bincode/spec/index.html)

**Fit for TCE:** technically workable with frozen `SaveV1`, `SaveV2`, and migration code, but not my recommendation for a new multi-year persistence foundation.

As of this review, RustSec lists the project as unmaintained. The usable 2.0.1 documentation remains available; 3.0.0 is a terminal notice containing a compile error. This is a maintenance-risk finding, not evidence that every existing bincode save is insecure. [RustSec](https://rustsec.org/advisories/RUSTSEC-2025-0141.html)

### Custom encoding and database-backed storage

A fully custom binary encoding offers exact control over layout, packing, and streaming. It also makes TCE responsible for every compatibility rule, malformed-input case, and inspection tool.

I recommend a **small custom container**, not a custom serialization framework.

SQLite is the alternative worth considering when persistence becomes dominated by incremental updates, archival queries, and transactional metadata. Store bounded chunks as BLOBs rather than modeling every per-tick agent mutation as SQL. WAL permits readers to coexist with a writer, but introduces checkpointing and additional persistent files; copying only the main database can omit committed state. [SQLite](https://sqlite.org/wal.html)

**2026 caveat:** use a release containing the WAL-reset corruption fix—SQLite identifies **3.51.3 and later**, or specified backports, as fixed. The affected pattern involves concurrent connections writing/checkpointing in WAL mode. [SQLite](https://sqlite.org/wal.html)

### Overall comparison

The suitability judgments below are recommendations, not universal benchmark rankings.

| Approach | Principal advantage | Principal cost | Recommended TCE role |
| --- | --- | --- | --- |
| FlatBuffers | Explicit evolvable schema; direct inspection | Builder cost, generated-code workflow | **Authoritative chunk payloads** |
| Cap’n Proto | Evolvable schema; efficient structured access | Arena/list constraints; another representation to maintain | Strong alternative |
| rkyv | Rust ergonomics; efficient archived access | Archive-schema and configuration coupling | Rebuildable caches |
| bincode | Simple integration | Explicit old-layout readers; maintenance risk | Existing formats, not new foundation |
| Fully custom codec | Maximum layout control | Maximum correctness and tooling burden | Only for measured special cases |
| SQLite chunk store | Mature transactions and incremental storage | Checkpointing, file lifecycle, operational complexity | Later alternative to custom incremental storage |

## 1.2 Snapshot techniques

**Stop-and-save** is the simplest correctness baseline: stop mutation, serialize, then resume. It is valuable for initial implementation and tests, but incompatible with a low-latency experience at multi-gigabyte scale.

**Deep-copy, then save asynchronously** moves encoding and disk I/O off the simulation thread, but copying remains on the critical path. Pointer-rich cloning can allocate heavily. For illustration, even a hypothetical sustained copy rate of 20 GB/s takes 250 ms to copy 5 GB, before object traversal or allocation.

**Application-level copy-on-write** captures a shared immutable version; subsequent simulation writes copy only shared pages being modified. Rust’s `Arc::make_mut` provides the basic clone-on-write mechanism, but the application chooses the granularity and must protect every relevant mutation path. [Rust Documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)

**Operating-system snapshots** can provide page-level copy-on-write automatically. Factorio documents using Unix `fork` for non-blocking saves on macOS/Linux, with memory costs and remaining freeze problems. That is a useful precedent, not a directly transferable Windows/UE solution. [Factorio](https://factorio.com/blog/post/fff-408)

**Incremental checkpoints** write changed data relative to a previous checkpoint. They reduce repeated work but introduce dependency tracking, garbage collection, and recovery complexity. They still require a consistent logical snapshot.

**Event sourcing** records changes or commands for replay. It is useful for debugging and historical records, but is a poor sole persistence strategy for TCE: replay across years of changed simulation code would require preserving old behavior or migrating the event semantics. Prefer snapshots plus optional bounded logs.

---

# 2. Trade-offs and performance evidence

## 2.1 Zero-copy access is not zero-time loading

For TCE, distinguish:

\[
T\_{\text{load}} =
T\_{\text{read}}+
T\_{\text{decompress}}+
T\_{\text{validate}}+
T\_{\text{migrate}}+
T\_{\text{materialize}}+
T\_{\text{relink}}+
T\_{\text{rebuild}}
\]

These stages may overlap, but each represents real work.

A zero-copy codec can avoid **materializing an intermediate object graph**. It does not remove disk reads, decompression, validation, schema migration, or the construction of mutable simulation tables. FlatBuffers, rkyv, and Cap’n Proto provide different access and validation mechanisms; none automatically restores TCE’s runtime indexes. [Docs.rs](https://docs.rs/flatbuffers/latest/flatbuffers/)

For TCE, direct access is most valuable for **read-only historical chunks**. Active people, markets, and construction queues will usually benefit from conversion into the kernel’s preferred mutable layout.

## 2.2 Published serialization benchmark

The Rust serialization benchmark’s September 10, 2026 results use Linux, an AMD EPYC 9V74 virtualized environment, and Rust 1.100 nightly—not your Windows i9. Its `minecraft_savedata` dataset contains structured player-save data, not a multi-gigabyte Minecraft world. Selected buffer-only results are: [GitHub](https://github.com/djkoloski/rust_serialization_benchmark)

| Codec | Serialize | Encoded bytes | Bytes after benchmark’s Zstd compression |
| --- | --- | --- | --- |
| FlatBuffers 25.12.19 | 2.8732 ms | 849,472 | 294,871 |
| Cap’n Proto Rust 0.27.0, unpacked | 0.39246 ms | 803,896 | 280,744 |
| rkyv 0.8.18 | 0.21250 ms | 603,776 | 219,421 |

These results justify testing rkyv and Cap’n Proto for throughput-sensitive uses. They do **not** establish their complete-world load times, nor make nanosecond buffer-access measurements comparable to validated loading and reconstruction. [GitHub](https://github.com/djkoloski/rust_serialization_benchmark)

My choice of FlatBuffers prioritizes the permanent schema contract over winning this microbenchmark.

## 2.3 What can be said about 1–5 GB states?

**I did not find a controlled, directly comparable end-to-end benchmark covering all these formats on 1–5 GB simulation states on your target hardware.** The following is therefore capacity modeling, not measured TCE performance.

Zstandard’s reference benchmark reports, on an i7-9700K/Linux/Silesia workload:

| Codec | Compression ratio | Compression throughput | Decompression throughput |
| --- | --- | --- | --- |
| Zstd 1.5.7, level 1 | 2.896:1 | 510 MB/s | 1,550 MB/s |
| LZ4 1.10.0 | 2.101:1 | 675 MB/s | 3,850 MB/s |

These are in-memory compression measurements. [GitHub](https://github.com/facebook/zstd)

Applying those ratios and rates arithmetically gives:

| Codec | Uncompressed serialized payload | Estimated compressed payload | Compression time | Decompression time |
| --- | --- | --- | --- | --- |
| Zstd level 1 | 1 GB | 0.345 GB | 1.96 s | 0.65 s |
| Zstd level 1 | 5 GB | 1.727 GB | 9.80 s | 3.23 s |
| LZ4 | 1 GB | 0.476 GB | 1.48 s | 0.26 s |
| LZ4 | 5 GB | 2.380 GB | 7.41 s | 1.30 s |

**These are extrapolations, not multi-gigabyte benchmark results.** GB is decimal; input means serialized bytes, not the Rust heap. TCE’s data distribution, chunking, concurrent simulation, and storage will change the results.

For example, assuming—not measuring—2 GB/s effective storage reads, the 5 GB Zstd case requires about **0.86 seconds of reading plus 3.23 seconds of decompression** if performed sequentially, before validation and reconstruction. Independent chunks permit overlap and parallelism.

For a well-pipelined implementation, a useful optimistic model is:

\[
T\_{\text{bulk load}}
\gtrsim
\max\left(
\frac{D}{R\_{\text{I/O}}},
\frac{U}{R\_{\text{decompression}}},
\frac{U}{R\_{\text{validation}}},
\frac{U}{R\_{\text{materialization}}}
\right)
\]

where \(D\) is compressed size and \(U\) is uncompressed size. Dependencies, startup, allocation, and index construction add overhead.

**Practical conclusion:** optimize the whole pipeline before changing codecs. Saving unnecessary state or rebuilding expensive indexes can outweigh serialization differences.

## 2.4 The benchmark TCE actually needs

Build a standalone Rust benchmark using real persistence structures, with **1, 3, and 5 GB payloads** spanning dense arrays, strings, relationships, terrain, and old history.

Measure complete save latency through successful flush/publication; cold and warm loads; migration from the oldest supported schema; peak committed memory; and simulation/frame-time percentiles during autosave. Separate “world data loaded,” “first simulation tick possible,” and “UE presentation ready.”

Repeat under the actual 1440p rendering workload. Your SSD model, free space, and sustained performance are unspecified, so the hardware description alone cannot support a trustworthy load-time promise.

---

# 3. Precedents and what they teach

## Factorio: explicit migrations plus consistent snapshots

Factorio’s current **2.1.20** migration documentation separates prototype changes expressed in JSON from scripted Lua migrations. Saves track applied migration filenames. Ordering is defined, and the documentation warns that changing prototype type can invalidate references even when a simple rename would preserve them. [Factorio Lua API](https://lua-api.factorio.com/latest/auxiliary/migrations.html)

**Lesson for TCE:** separate mechanical identifier remapping from semantic transformations. Make migrations identifiable, ordered, and recorded. Reference preservation needs explicit tests.

Its April 2024 discussion of Unix `fork` saving illustrates the snapshot problem: background serialization is useful only after the saver has a stable world version. The platform-specific implementation and memory caveats matter. [Factorio](https://factorio.com/blog/post/fff-408)

## Paradox: compatibility is sometimes deliberately bounded

Stellaris **4.5 “Cygnus,” released September 22, 2026**, explicitly breaks compatibility with 4.4-and-earlier saves because its population-group redesign changes storage. The official announcement tells players to back up and use the 4.4 rollback branch before opening an old campaign in 4.5. [Steam Store](https://store.steampowered.com/news/posts/?appids=281990&feed=steam_community_announcements)

An older but concrete EU4 example is **1.30.6 in March 2021**: the developer added save-version checks to the Continue path after players had damaged campaigns by continuing across incompatible versions, and advised remaining on the prior patch. [Steam Store](https://store.steampowered.com/news/posts/?appids=236850&enddate=1619445544&feed=all)

**Lesson for TCE:** an old executable is a useful preservation fallback, but it is not migration. Because your product promise is stronger, budget for conversions that other games sometimes choose not to implement. Every entry point—including “Continue”—must enforce compatibility.

## Dwarf Fortress: loadability and new-world content are different

Bay 12’s **53.15 release in June 2026** distinguishes new animals that require a newly generated world from graphical changes that work in old saves. **53.13** also documents a targeted repair of corrupted musical instruments during loading. [Bay 12 Games](https://www.bay12games.com/dwarves/)

The official **50.01+ modding guide** says mods no longer live inside save files and must be installed on each machine loading a dependent save. [Bay 12 Games](https://www.bay12games.com/dwarves/modding_guide.html)

**Lesson for TCE:** preserving a world does not mean retroactively generating every new feature. Dependency availability is a separate preservation problem: a perfectly readable save can still lack the definitions needed to interpret it.

## RimWorld: compatibility includes targeted retrofits

RimWorld’s **1.6.4630 update, October 2025**, includes a fix for a gravship launch ritual not being retroactively added to existing saves. Later 1.6 updates explicitly state their intended save/mod compatibility. These are concrete examples of supporting existing worlds through targeted additions and repairs. [Steam Store](https://store.steampowered.com/oldnews/?appgroupname=RimWorld+Name+in+Game+Pack&appids=294100&feed=steam_community_announcements&l=swedish)

**Lesson for TCE:** introducing a system often requires a deliberate old-world initialization rule, not merely a new optional field.

The public first-party material reviewed does not establish RimWorld’s complete internal migration architecture, so it should not be treated as evidence for a particular binary codec or snapshot implementation.

## OpenTTD and Minecraft: useful code-level models

OpenTTD’s `afterload.cpp` contains explicit conversions for old representations. Particularly relevant is its comment explaining why window/cache initialization was moved after all save conversions: adding conversions after initialization had caused bugs. [GitHub](https://raw.githubusercontent.com/OpenTTD/OpenTTD/master/src/saveload/afterload.cpp)

Mojang’s **DataFixerUpper** provides utilities for building, composing, and optimizing data transformations, and is used for Minecraft Java’s versioned data conversion. [GitHub](https://github.com/Mojang/DataFixerUpper)

**Lesson for TCE:** decode historical representations, perform explicit transformations, and only then construct derived runtime state. Borrow this organization; do not reproduce an entire general-purpose transformation framework for a solo project.

## Crash-consistency research

Pillai and colleagues’ **OSDI 2014** study found that application crash protocols depended on subtle filesystem persistence behavior; it examined six Linux filesystems and found numerous application vulnerabilities. It is evidence against casually transferring filesystem recipes between platforms, not a Windows durability benchmark. [USENIX](https://www.usenix.org/conference/osdi14/technical-sessions/presentation/pillai)

SQLite’s testing infrastructure is an especially practical precedent: it simulates failures, reorders or damages unsynchronized writes, and checks that a transaction either completed or rolled back. Simply killing a process is not equivalent to testing power failure. [SQLite](https://sqlite.org/testing.html)

---

# 4. Recommended architecture for TCE

The following sizes and policies are **proposed starting points to benchmark**, not established performance guarantees.

## 4.1 Define the persistence contract independently of the runtime

Use three representations:

```
Historical save schemas
        ↓ decode and migrate
Current durable world model
        ↓ construct and index
Runtime simulation tables / ECS / UE presentation
```

The durable model should express domain facts: a person’s identity, relationships, property, commitments, institutional memberships, and current activities. It should not expose allocator layout, pointers, hash-bucket order, ECS archetype ordering, or Unreal object addresses.

Use **world-stable IDs** for people, institutions, parcels, structures, and historical entities. Reconstruct runtime handles on load. A persistent identity should not change because an entity moved between tables or was promoted from a historical record into an active agent.

Save authoritative state that is easy to overlook: simulation clock and resolution mode, RNG state and algorithm identity, scheduled events, outstanding reservations, partially completed production, and ID-allocation counters.

For multi-rate operation, define a legal checkpoint boundary. Either finish the current atomic daily/sub-daily step or explicitly persist its intermediate state. Never save half a market-clearing operation while presenting it as a complete tick.

Classify caches carefully. A value is safely rebuildable only when rebuilding it preserves the intended model—not merely because the code calls it a cache.

## 4.2 Use a small, explicit chunk container

A self-contained generation might contain:

```
Fixed header
  magic, container version, world ID, snapshot ID, parent ID, tick

Independently compressed chunks
  population / households / institutions
  land / buildings / inventories / production
  scheduler / random state / pending commitments
  content definitions and provenance
  historical partitions

Manifest
  chunk identity, schema version, codec
  64-bit offset and lengths, integrity digest, dependencies

Completion footer
  manifest location, expected file length, integrity information
```

Use **64-bit outer offsets and lengths**, even though individual FlatBuffers are small.

Start with roughly **1–16 MiB uncompressed disk chunks**, testing several sizes. Partition by domain and locality, not by arbitrary traversal order. A settlement may occupy many chunks; it need not become one enormous buffer.

Use FlatBuffers tables with explicit field IDs and scalar vectors where appropriate. Validate vector lengths, ID uniqueness, and cross-column consistency separately from binary verification.

Compress each chunk independently with **Zstd level 1** initially. Benchmark LZ4 and no compression for latency-sensitive or poorly compressible chunks. Record the codec per chunk so future changes do not require reinterpreting older files.

Do not make the entire file one compression stream. Independent chunks are what make bounded-memory decoding, parallel work, selective history access, and precise corruption reports possible.

## 4.3 Capture snapshots with bounded copy-on-write

Use a short simulation barrier after all jobs for a logical checkpoint have completed. Capture an immutable root containing the complete authoritative state.

The background saver traverses that version while the simulation mutates a newer version.

For Rust, prefer copy-on-write **pages or blocks**, not `Arc<WholeWorld>` with a deep-cloning world underneath. An `Arc<Vec<Person>>` can accidentally turn one small mutation into cloning the entire population vector; the clone boundary is an architectural choice. [Rust Documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)

Test memory-page sizes around **64 KiB–1 MiB**. These need not match disk chunks. Frequently modified columns may warrant smaller pages or a separate storage strategy.

A useful memory model is:

\[
M\_{\text{peak}}
\approx
M\_{\text{live}}+
M\_{\text{retained old pages}}+
M\_{\text{encoding buffers}}+
M\_{\text{UE and OS}}
\]

If every page changes while saving, retained old data can approach another full state. Copy-on-write shifts copying to writers; it does not make copying disappear.

For the first implementation, allow **one autosave in flight**, use bounded queues and a memory budget, and coalesce subsequent requests. Start with a small dedicated worker pool—perhaps two to four threads—and measure rather than letting compression occupy every available core.

Move substantial buffer destruction and snapshot cleanup off latency-sensitive threads. Track both save completion time and the cost of mutations while a snapshot is pinned.

With 64 GB RAM, this design is plausible, but available headroom must be measured alongside UE. The GPU should not be part of the initial persistence pipeline; adding a GPU codec would introduce another synchronization and resource-contention problem before CPU persistence is characterized.

## 4.4 Publish saves safely on Windows

Treat these as separate properties:

**Snapshot consistency** means every saved relationship belongs to one coherent world version. **Integrity checking** detects damaged bytes. **Publication** determines which completed generation is visible. **Durability** concerns what survives an OS crash or power loss.

For TCE’s initial implementation, use this protocol:

1. **Create a uniquely named temporary file in the destination directory**, exclusively. Never open the previous generation for truncation.
2. **Write the complete container**, checking all writes, compression finalization, manifest, and footer operations.
3. **Flush application buffers, then synchronize the file.** With Rust buffering, call `BufWriter::flush()` before `File::sync_all()`. Dropping a writer/file is not a checked durability operation. [Rust Documentation](https://doc.rust-lang.org/std/io/struct.BufWriter.html)
4. **Close and publish under a new generation filename on the same volume.** A Windows implementation can explicitly use `MoveFileExW` with `MOVEFILE_WRITE_THROUGH`; do not allow a cross-volume copy fallback. Preserve the old generations.
5. **Update any “latest” selector only as a convenience.** Recovery must be able to discover valid generations without trusting that selector.
6. **Retire old generations only after successful publication and validation**, retaining several predecessors and independent manual checkpoints.

The Windows details matter. Microsoft documents `FlushFileBuffers` as writing buffered file information to the device. `MoveFileExW` documents write-through behavior, but `MOVEFILE_COPY_ALLOWED` can implement movement using copy/delete across volumes. `ReplaceFileW` has its own failure outcomes, and its `REPLACEFILE_WRITE_THROUGH` flag is explicitly **unsupported**. These are not interchangeable magic “atomic save” APIs. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)

The recommended design deliberately avoids making correctness depend on replacing the only valid file. Same-volume rename is the publication operation; **retained immutable generations plus validation provide recovery** when publication is interrupted or ambiguous.

At startup, identify candidate generations using world/branch identity and sequence metadata, validate framing and required chunks, and select the newest valid candidate. Do not select by modification time alone. Report fallback to an older save visibly.

Ignore incomplete temporary files during normal loading; offer separately verified complete temporaries as recovery candidates rather than silently declaring them committed.

Checksums are not structural validation or proof of correct world semantics. Reject oversized decompression claims, invalid offsets, malformed references, and unreasonable allocation requests before constructing the world.

Finally, state the guarantee honestly: this is designed to tolerate interrupted saves and preserve prior generations under the tested Windows/storage configuration. It cannot protect against arbitrary device failure, a filesystem losing unrelated files, or a storage stack violating flush expectations. Keep external backups.

## 4.5 Make migrations a permanent subsystem

### Version independent concerns independently

Record separate versions for the container, each domain schema, the simulation model/rules, content definitions, and optional cache formats.

A new compression option should not require a population migration. A population migration should not force every immutable historical partition to be rewritten.

### Preserve old readers and apply explicit transformations

Use a pipeline such as:

```
Read and bound-check container
→ Identify schema and dependencies
→ Decode historical representation
→ Apply ordered migrations
→ Validate world invariants
→ Build runtime tables
→ Resolve references
→ Rebuild derived indexes
→ Enable simulation
```

Keep historical schema definitions and regression fixtures under source control. Pin released migrations: correcting a migration later should be an explicit, tested change, not incidental refactoring.

For local changes, stream chunks through the necessary transformations. For global changes—such as introducing household property ownership—use staged passes: create identities first, then transform holdings, then resolve and validate references. Avoid repeatedly materializing several complete 5 GB intermediate worlds.

### Distinguish missing data from legitimate zero values

Suppose an old save stores only:

```
person.food_stock
```

and the new model stores:

```
household.food_inventory
person.food_entitlement
```

No codec can decide ownership, pooling rules, or entitlement semantics.

Define the transformation explicitly, preserve total food except for documented adjustments, assign stable household identities, and record the migration. A newly defaulted zero must not silently mean “this person has no rights” when it really means “the old model did not represent rights.”

Do not initialize new systems by running normal simulation ticks during migration. That can consume resources, trigger events, or advance history unexpectedly.

### Preserve content meaning, not merely content names

Store stable identifiers and exact versions/hashes for authored building blocks. For essential definitions, consider embedding the normalized data required to interpret the world, or maintaining a versioned content package alongside it.

A content hash identifies missing data; it does not recreate it.

Generated architecture needs either preserved results or enough versioned generator inputs and code compatibility to reproduce them. The same seed is insufficient when the generator changes.

Define a missing-content policy: hard failure for essential semantics; explicit placeholders only where safe; no silent substitution of an unrelated current definition.

### Promise preservation of facts, not identical future history

Separate the following compatibility targets:

| Target | Meaning |
| --- | --- |
| Structural | The file can be decoded |
| Referential | Identities and relationships remain valid |
| Semantic | Facts such as ownership and inventory retain their meaning |
| Behavioral | Continuing produces equivalent future behavior |

TCE should strongly target the first three. Behavioral equivalence across changed economic, disease, or political algorithms requires preserving those algorithms or accepting and documenting changed continuation behavior.

Reject newer unsupported **critical** schemas before allowing the old executable to save. Ignoring unknown fields while reading does not establish that an older writer can safely preserve them.

Always write a migrated world as a **new generation**, leaving the original untouched.

## 4.6 Keep endless history separate from active state

Partition history into immutable, independently indexed blocks. Load only the history needed for current simulation dependencies and requested inspection.

Later, unchanged blocks can be reused through a full manifest referencing immutable chunk packs. Prefer a manifest that directly names all required blocks over a long chain of “apply these 400 deltas.”

That optimization introduces garbage collection, compaction, and reference accounting. Add it only after complete snapshots and recovery tests are solid. SQLite becomes attractive when those storage-management demands exceed the simplicity of the original container.

Also define what “endless” preserves. Keeping every sub-daily observation forever implies unbounded storage growth. Distinguish authoritative state and meaningful history from disposable telemetry, with explicit retention rather than accidental loss.

## 4.7 Integrate through a narrow DLL boundary

Let Unreal submit a save request and poll or receive completion through a small C-compatible interface. Rust should own snapshot lifetimes, serialization buffers, workers, and errors.

Do not marshal the whole world into a `USaveGame` object merely to call the async API: Epic’s UE 5.8 documentation says serialization occurs on the game thread. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UGameplayStatics/AsyncSaveGameToSlot)

On load, build the new Rust world separately and validate it before replacing the active instance. Recreate UE presentation incrementally afterward.

Define shutdown behavior explicitly: outstanding work must finish or cancel safely before DLL unload; buffers must be released by their owning allocation boundary; callbacks must not target destroyed UE objects.

## 4.8 Testing is part of the format

Maintain golden saves from **every released schema version**, not only the previous release. Include sparse worlds, dense cities, long histories, missing dependencies, and unusual institutional relationships.

Test three things independently:

**State correctness:** same-schema round trips preserve authoritative state; migrations preserve declared invariants; references resolve; ownership and inventory totals remain valid.

**Failure recovery:** inject write failures, disk-full conditions, truncation, corrupted chunks, interrupted migration, and failures around flush/publication. Verify that the previous valid generation remains recoverable. SQLite’s failure-injection approach is a better model than a handful of manual process kills. [SQLite](https://sqlite.org/testing.html)

**Latency and memory:** record snapshot-barrier duration, copy-on-write costs, peak memory, save age, and frame/tick percentiles under sustained play.

For AI-assisted development, make schema discipline executable: compatibility checks, immutable field-ID rules, migration fixtures, and code review gates. A perfectly reasonable-looking automated refactor must not renumber persisted enums or delete historical readers.

Build a headless Rust utility early:

```
tce-save inspect
tce-save verify
tce-save migrate
tce-save compare
```

That tool will make persistence failures diagnosable without launching Unreal.

---

# 5. Sources and version scope

The principal references are linked below. Game examples are tied to the stated releases; they are not claims of universal compatibility across every version.

| Area | Documentation, code, or research | Scope |
| --- | --- | --- |
| FlatBuffers | [Evolution rules](https://flatbuffers.dev/evolution/?utm_source=chatgpt.com), [internals](https://flatbuffers.dev/internals/?utm_source=chatgpt.com), [Rust API](https://docs.rs/flatbuffers/25.12.19/flatbuffers/) | Rust 25.12.19; current format documentation |
| Cap’n Proto | [Schema language/evolution](https://capnproto.org/language.html?utm_source=chatgpt.com), [encoding](https://capnproto.org/encoding.html?utm_source=chatgpt.com), [Rust reader limits](https://docs.rs/capnp/0.27.2/capnp/message/struct.ReaderOptions.html) | Rust 0.27.2 documentation; benchmark uses 0.27.0 |
| Rust-native codecs | [rkyv compatibility](https://docs.rs/rkyv/0.8.18/rkyv/), [bincode 2.0.1 specification](https://docs.rs/bincode/2.0.1/bincode/spec/index.html?utm_source=chatgpt.com), [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2025-0141.html?utm_source=chatgpt.com) | rkyv 0.8.18; bincode maintenance status checked in 2026 |
| Performance | [Rust serialization benchmark and code](https://github.com/djkoloski/rust_serialization_benchmark?utm_source=chatgpt.com), [Zstd reference benchmark](https://github.com/facebook/zstd#benchmarks) | September 10, 2026 serialization results; Zstd 1.5.7/LZ4 1.10.0 compression results |
| Windows | [FlushFileBuffers](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers?utm_source=chatgpt.com), [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw?utm_source=chatgpt.com), [ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew?utm_source=chatgpt.com) | Current documented Win32 contracts |
| Engine and snapshot precedent | [UE async save API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UGameplayStatics/AsyncSaveGameToSlot?utm_source=chatgpt.com), [Factorio FFF-408](https://factorio.com/blog/post/fff-408?utm_source=chatgpt.com) | UE 5.8; Factorio April 2024 implementation discussion |
| Migration precedents | [Factorio migrations](https://lua-api.factorio.com/latest/auxiliary/migrations.html?utm_source=chatgpt.com), [OpenTTD after-load code](https://github.com/OpenTTD/OpenTTD/blob/master/src/saveload/afterload.cpp), [Mojang DataFixerUpper](https://github.com/Mojang/DataFixerUpper?utm_source=chatgpt.com) | Factorio 2.1.20; public repository revisions reviewed September 2026 |
| Game compatibility policies | [Stellaris official announcements](https://store.steampowered.com/news/posts/?appids=281990&feed=steam_community_announcements&utm_source=chatgpt.com), [Bay 12 development log](https://www.bay12games.com/dwarves/?utm_source=chatgpt.com), [DF mod guide](https://www.bay12games.com/dwarves/modding_guide.html?utm_source=chatgpt.com), [RimWorld official announcements](https://store.steampowered.com/oldnews/?appgroupname=RimWorld+Name+in+Game+Pack&appids=294100&feed=steam_community_announcements&l=swedish&utm_source=chatgpt.com) | Stellaris 4.5; DF 53.x and 50.01+ mod policy; RimWorld 1.6 |
| Crash-consistency research | [Pillai et al., OSDI 2014—paper, slides, and presentation](https://www.usenix.org/conference/osdi14/technical-sessions/presentation/pillai?utm_source=chatgpt.com), [Atomic Commit in SQLite](https://sqlite.org/atomiccommit.html?utm_source=chatgpt.com), [SQLite testing](https://sqlite.org/testing.html?utm_source=chatgpt.com) | Foundational research and maintained implementation documentation |
| Database alternative | [SQLite WAL documentation](https://sqlite.org/wal.html?utm_source=chatgpt.com) | Includes 2026 WAL-reset fix information |

**Bottom line:** the most robust fit is not the fastest single serializer. It is **a stable domain schema, a consistent snapshot boundary, immutable generations, checked Windows publication, and a migration/test suite maintained as carefully as the simulation itself**. For TCE, a thin chunk container plus FlatBuffers and background copy-on-write offers the best starting balance of longevity, performance control, and solo-developer complexity.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92785-9d28-83ea-8347-bf9dfcc288e1)
