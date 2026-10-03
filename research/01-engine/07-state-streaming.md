# Streaming TCE’s simulation into Unreal Engine 5.8

**Engineering report — research current to September 27, 2026**

## Executive recommendation

For TCE, build a **local, asynchronous presentation-state pipeline**, not a local multiplayer connection:

**Rust remains authoritative. It publishes compact, immutable render snapshots at a wall-clock-limited rate. Unreal maintains its own presentation state, interpolates movement, and renders people through batched, level-of-detail representations. Persistent city changes travel through a separate, versioned, recoverable stream.**

Start with **20 Hz publication and approximately 32 bytes per person**. For 50,000 people, that is **1.6 MB per snapshot and 32 MB/s of payload**. This is small enough that I would optimize ownership, batching, animation, and game-thread work before implementing sophisticated compression.

Then add **path-based movement descriptions with corrections** where they demonstrably reduce work. Do not make “trip start and trip end are sufficient” an architectural assumption: congestion, interruptions, rerouting, and time acceleration can invalidate it.

The largest uncertainty is not DLL-to-engine bandwidth. It is the cost of the chosen crowd assets, animation system, instance updates, shadows, and city reconstruction. Epic’s UE 5.8 crowd announcement describes scaling to thousands of characters using an Experimental MetaHuman Collections workflow; it does **not** establish that 50,000 detailed characters will run at 60 fps on a 4070 Ti. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

**All TCE-specific sizes, frequencies, and time allocations below are calculations or proposed starting budgets, not measurements on your hardware.**

---

## 1. Options: techniques and how they work

### 1.1 The principal transport choices

| Technique | How it works | Performance characteristics | Complexity and fit for TCE |
| --- | --- | --- | --- |
| **Full render snapshots** | Publish all relevant agents’ current presentation fields, independently of previous motion frames. | Predictable linear work; straightforward bulk copies; easy frame skipping. | Lowest implementation risk. Best starting point at 10k–50k people. |
| **Sparse absolute updates** | Publish only changed entities or fields, but values are absolute rather than relative to an older snapshot. | Excellent for stationary objects and infrequently changing attributes. Moving people still change continuously. | Moderate complexity. Requires a maintained receiver state and explicit lifecycle handling. |
| **Baseline-relative deltas** | Encode changes relative to a specific snapshot already available to the receiver. | Can substantially reduce bytes; adds comparison, encoding, decoding, and baseline retention. | Useful for remote transport or recording. Less compelling as the first in-process optimization. |
| **Movement descriptions plus corrections** | Publish a path reference, timing/progress information, and subsequent deviations. | Very economical when many rendered frames can be reconstructed from one description. | Strong eventual fit for road-bound travel; correctness becomes harder around interruptions and high-speed simulation. |
| **Replicated simulation or input replay** | Send inputs and run another simulation to reconstruct state. | Potentially tiny input bandwidth, but duplicates computation and requires reproducibility. | Poor fit for the Rust-to-Unreal boundary. Unreal should not become a second civilization kernel. |

Baseline-relative compression requires the receiver to possess the exact baseline used by the sender. Fiedler’s snapshot-compression implementation makes this explicit through snapshot identifiers and acknowledgements. Local transport removes packet-loss concerns, but **does not remove this dependency when your application intentionally skips frames**. [Gaffer On Games](https://gafferongames.com/post/snapshot_compression/)

### 1.2 Separate “current motion” from “persistent world changes”

This is the most important protocol distinction.

A motion frame answers:

> Where are the currently relevant people, and what are they visibly doing at simulation time \(t\)?

A persistent-state update answers:

> Which entities, buildings, paths, appearances, and construction stages exist, and which versions apply?

Treat them differently:

| Information | Appropriate delivery semantics |
| --- | --- |
| Current position, heading, movement speed | Replaceable by a newer independently usable state |
| Building creation, demolition, road revision | Ordered updates with gap detection and snapshot recovery |
| Appearance or animation-set assignment | Versioned persistent state |
| Dust puff, incidental sound, minor visual effect | Usually expendable during overload or fast-forward |
| Historically important simulation event | Separate history/archive policy, not merely a renderer queue |

**Never put incremental city mutations into a “latest frame wins” mailbox and assume correctness survives dropped frames.**

Likewise, dropping sparse updates is unsafe unless a consumer has already applied them to a complete materialized state. If frame 100 changes one person’s clothing and frame 101 only changes positions, discarding frame 100 does not become safe merely because frame 101 is newer.

For TCE, I recommend independently usable motion snapshots and recoverable, versioned city-state updates. They may share a publication mechanism, but their logical contracts must remain separate.

### 1.3 Quantization: useful, but not the first emergency

A plausible initial **32-byte motion record** is:

| Field | Bytes |
| --- | --- |
| Generational entity handle | 8 |
| Cell-local position, three `float32` values | 12 |
| Heading and speed | 4 |
| Animation clip and phase | 4 |
| Flags/activity information | 4 |
| **Total** | **32** |

Group records by spatial cell, with cell origin and simulation timestamp in shared headers. Appearance definitions and path geometry belong in separate catalogs.

A more compact **16-byte transform-only record** can contain an 8-byte handle, three 16-bit coordinates, and a 16-bit heading. It relies on other streams for activity and appearance.

For a 256 m coordinate range:

\[
\text{position step}=\frac{256}{65535}\approx 3.906\text{ mm}
\]

Rounding error is at most approximately 1.953 mm per axis. A 16-bit full-circle heading has approximately 0.00549° resolution.

Those numbers are calculations, not a recommendation that every coordinate—including elevation—must use the same range. Provide an escape representation for out-of-range values, and handle cell transitions atomically.

**Quantize absolute values before computing deltas.** Repeatedly accumulating rounded displacement increments introduces drift that absolute quantization avoids.

For an in-process feed, I would initially retain float positions and avoid general-purpose compression. Smaller data is not automatically faster after encoding, decoding, and bookkeeping are included.

### 1.4 Interest management: simulation relevance is not rendering relevance

TCE’s offscreen people must continue making decisions. Interest management should primarily reduce **presentation work**, not silently stop the authoritative simulation.

My proposed initial interest policy is a spatial grid, camera frustum plus a prefetch margin, distance/projected-size tiers, and explicit overrides for selected people. Add hysteresis so camera motion does not repeatedly create and destroy representations at a boundary.

At only 50,000 entities and one viewer, a simple linear relevance pass may be preferable to elaborate spatial machinery. Benchmark it before introducing per-person visibility rays or a complicated hierarchy.

Use separate messages for **leaving the renderer’s interest set** and **ceasing to exist**. On re-entry, bootstrap the current state, current path progress, and required asset revisions; do not replay every event missed while offscreen.

Research supports the potential benefit but also the workload dependence. Boulanger, Kienzle, and Verbrugge’s 2006 comparison found that accounting for obstacles could reduce update messages by up to a factor of six in their Mammoth experiments. Their results also changed substantially with indoor/outdoor movement and trace construction. That is evidence for testing realistic camera and settlement workloads—not a transferable sixfold TCE speedup. [Sable Research Group](https://www.sable.mcgill.ca/~clump/papers/boulanger-06-comparing.pdf)

### 1.5 Event-based movement versus streamed positions

**Use a hybrid, with different authority and reconstruction rules for different movement types.**

For ordinary travel on a stable road network, publish a movement description containing the entity and trip handles, path ID and revision, a time/progress anchor, speed or segment timing, lateral offset, and validity information. Unreal evaluates:

\[
\mathbf p(t)=P\_{\text{path}}\bigl(s(t)\bigr)
\]

Here \(s(t)\) is distance along the path—not an arbitrary spline parameter. Evaluating from an absolute time anchor avoids accumulating frame-rate-dependent integration error.

For crowded junctions, stopping, local avoidance, work interactions, combat, or other irregular motion, use more frequent authoritative samples or corrections.

**Trip start/end alone is not generally enough.** At departure, the eventual arrival time may not yet be known. A live simulator cannot truthfully provide the final uninterrupted trajectory if later interactions change it. Send committed segments, update timing, cancel or replace descriptions, and periodically refresh current progress.

For reconstruction, begin with linear interpolation between samples. Where needed, endpoint velocities permit smoother Hermite interpolation, but constrain it around corners and stops to avoid overshoot. When a path is known, interpolating progress along that path avoids cutting across buildings. Fiedler’s interpolation examples demonstrate the benefits of buffering and velocity-assisted interpolation, and also why unconstrained extrapolation behaves poorly around unpredictable physical interactions. [Gaffer On Games](https://gafferongames.com/post/snapshot_interpolation/)

Keep extrapolation bounded by **both wall-clock duration and simulation-space distance/time**. A proposed 50–100 ms wall-clock limit can be reasonable at low speed; it becomes dangerously permissive at extreme acceleration.

Finally, Unreal may synthesize cosmetic foot placement or animation blending. It should not independently decide authoritative arrival, collision outcomes, or resource access.

---

## 2. Trade-offs and numerical budgets

### 2.1 Transport bandwidth

The following are decimal MB and exclude headers, catalogs, corrections, and allocator overhead:

\[
B=N\times\text{record size}\times\text{publication frequency}
\]

| Configuration | Payload per publication | Payload rate |
| --- | --- | --- |
| 10,000 people × 32 bytes × 20 Hz | 0.32 MB | **6.4 MB/s** |
| 50,000 people × 32 bytes × 20 Hz | 1.60 MB | **32 MB/s** |
| 50,000 people × 32 bytes × 60 Hz | 1.60 MB | **96 MB/s** |
| 50,000 people × 16 bytes × 20 Hz | 0.80 MB | **16 MB/s** |
| Illustrative 64-byte GPU instance data × 50,000 × 60 Hz | 3.20 MB | **192 MB/s** |

The last row is an illustrative upload calculation, **not a claim about the complete UE 5.8 skinned-instance layout**.

A hypothetical distance-tiered feed could be:

* 5,000 nearby people at 32 bytes and 20 Hz;
* 15,000 medium-distance people at 24 bytes and 5 Hz;
* 30,000 distant people at 16 bytes and 1 Hz.

That totals **5.48 MB/s**, before other data. However, low-frequency updates only work visually when valid paths or adequate interpolation information bridge the gaps.

At an **assumed effective bulk-copy throughput of 10–20 GB/s**, copying a 1.6 MB frame takes approximately **0.16–0.08 ms**. This is an assumption-based estimate, not an i9 benchmark. It excludes extraction, cache misses, synchronization, decoding, and engine updates.

The implication is practical: **one intentional bulk copy can be a good trade for much simpler ownership**.

### 2.2 Path events do not remain cheap at arbitrary time acceleration

Suppose all 50,000 people continuously make trips lasting 300 simulated seconds, with two 64-byte records per trip.

That generates approximately:

\[
\frac{50{,}000}{300}\times2\times64
=21{,}333\text{ bytes per simulated second}
\]

At one simulated second per wall second, that is approximately **21.3 kB/s**. At 1,000 simulated seconds per wall second, it becomes **21.3 MB/s**, before path geometry and corrections.

This is why I would not couple renderer work directly to every movement event forever. At high speed, coalesce obsolete movement histories into current presentation state, while preserving persistent consequences.

Snapshot publication can stay at 20 Hz of **wall time** even when simulation throughput increases dramatically.

### 2.3 Memory budget

An illustrative bridge working set is:

| Allocation | Assumption | Size |
| --- | --- | --- |
| Rust handoff buffers | Three full 32-byte frames | 4.8 MB RAM |
| Unreal interpolation history | Four full frames | 6.4 MB RAM |
| CPU instance staging | Two 64-byte-per-person arrays | 6.4 MB RAM |
| Per-person path state | 64 bytes × 50,000 | 3.2 MB RAM |
| City/module descriptor catalog | 64 bytes × 100,000 | 6.4 MB RAM |
| **CPU subtotal** |  | **27.2 MB RAM** |
| Illustrative GPU instance-buffer ring | Three 3.2 MB buffers | **9.6 MB VRAM** |

This excludes indices, variable-length paths, meshes, textures, animation data, skeletal Actor objects, and render targets. Reserving **64–128 MB of RAM for the bridge’s ordinary working data** is a sensible initial envelope, not a bound on the whole renderer.

Do not confuse a small transform stream with a small visual asset footprint. Epic’s ISM documentation gives rough GPU costs of 64 bytes for a basic instance versus 672 bytes for a primitive; those figures are **not total character memory**. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Also distinguish live buffering from recording. Saving the uncompressed 32 MB/s feed continuously consumes **115.2 GB per hour**. A five-second debug ring consumes 160 MB. Endless worlds need bounded live history and a separate archival policy.

### 2.4 CPU and GPU acceptance targets

For the first performant implementation, I would use these **provisional acceptance targets**:

| Stage | Initial target | Measurement scope |
| --- | --- | --- |
| Rust render-state extraction and packing | ≤1 ms per 20 Hz publication | Worker execution |
| Unreal ingestion and indexing | ≤0.5 ms per publication | Worker execution |
| Interpolation, relevance, instance preparation | ≤2 ms per displayed frame | Elapsed worker phase |
| Game-thread bridge/batch commit | ≤1 ms per frame | Game-thread time |
| Amortized city structural updates | ≤0.5 ms per frame | Additional game-thread time |
| Crowd-specific render-thread submission | ≤1 ms per frame | Render-thread time |
| Crowd animation, drawing, and shadows | Initially allocate 3–4 ms | GPU time |

These do not add up as one serial pipeline: CPU workers, game thread, render thread, and GPU overlap. They are diagnostic limits within a **16.67 ms frame interval**, not proof that the complete scene meets it.

The per-entity arithmetic explains the danger of object-heavy code:

* 50,000 operations at 0.1 microseconds each consume **5 ms**.
* At 1 microsecond each, they consume **50 ms**.

Avoid per-person FFI calls, Blueprint updates, individual allocations, and unnecessary Actor/component work.

Do not maximize Rust CPU utilization independently of rendering. Sweep the kernel worker limit while measuring Unreal’s tail latency; leave scheduling headroom for the game thread, rendering, asset streaming, and driver work.

### 2.5 Published numbers: useful evidence, limited comparability

| Source | Published result | What it does—and does not—establish |
| --- | --- | --- |
| Fiedler, *Snapshot Compression*, 2015 | 901 cubes at 60 Hz start around **17.38 Mbit/s**; field reduction and quantization reduce each cube from **40.125 to 10 bytes**, before further delta techniques. | A worked bandwidth result, not a TCE CPU benchmark. [Gaffer On Games](https://gafferongames.com/post/snapshot_compression/) |
| `boxcars` Rocket League replay parser | README reports **5.1 ms / 290 MiB/s** with corruption checking, and **4.8 ms / 315 MiB/s** without it. | Parsing throughput for that benchmark; the table does not establish performance on your machine or include Unreal application/rendering costs. [GitHub](https://github.com/nickbabcock/boxcars) |
| Boulanger et al., 2006 | Obstacle-aware interest management reduced messages by **up to 6×** in the studied workloads. | Evidence that relevance policy matters, not a 50k-character rendering benchmark. [Sable Research Group](https://www.sable.mcgill.ca/~clump/papers/boulanger-06-comparing.pdf) |
| Epic Replication Graph documentation | Fortnite example: approximately **50,000 replicated Actors** at match start. | Replication scaling, not 50,000 simultaneously visible animated people. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/replication-graph-in-unreal-engine) |

None of these provides an apples-to-apples 50,000-person, UE 5.8, 1440p/60 benchmark for the specified PC. That final performance claim must come from your packaged-build prototype.

---

## 3. Precedents and their lessons

### Unreal replication and Iris: borrow the separation, not necessarily the subsystem

Iris maintains a quantized copy of replicated state, separates replication work from gameplay data, and supports filtering and prioritization. Those are directly useful architectural ideas for TCE’s presentation boundary. The introduction currently served as UE 5.8 documentation still labels Iris Experimental and opt-in. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/introduction-to-iris-in-unreal-engine)

My recommendation is **not** to create 50,000 replicated UObjects and run a local NetDriver just to move Rust data into Unreal. Build the smaller interface your single-process, single-viewer use case needs.

Replication Graph’s reusable relevance structures are another useful precedent: organize persistent spatial membership and avoid repeatedly rediscovering the same relationships. Its Fortnite scaling example should not be interpreted as a crowd-rendering guarantee. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/replication-graph-in-unreal-engine)

### Overwatch: presentation and replay benefit from a deliberate network model

Timothy Ford’s GDC 2017 talk describes Overwatch’s entity-component architecture and deterministic networked simulation. Philip Orwig’s companion replay talk describes replay technology built alongside the networking model, including interruptible killcams, highlights, and reproducible bugs. [GDC Vault](https://gdcvault.com/play/1024001/-Overwatch-Gameplay-Architecture-and)

The lesson for TCE is to make the presentation feed usable both live and from a recording. A renderer that can consume a captured stream is much easier to test independently of the kernel.

Do not extrapolate historical Overwatch implementation details into a claim that TCE needs shooter-style prediction or the same tick rates.

### Rocket League: inspect lifecycle and state changes, not just positions

Psyonix’s GDC 2018 talk is a useful first-party account of a game built around networked physics, rather than a directly comparable population simulator. [GDC Vault](https://www.gdcvault.com/play/1024972/It-IS-Rocket-Science-The)

For inspectable Rust code, `boxcars` is particularly relevant. Its replay-frame representation includes time, delta time, new Actors, deleted Actors, and updated Actors. That makes the distinction between lifecycle and changing state concrete. It is a community replay parser, not an official stable Psyonix protocol specification. [Docs.rs](https://docs.rs/boxcars/0.11.5/boxcars/struct.Frame.html)

For TCE, reuse that conceptual separation and version your own format explicitly.

### Factorio: input replication moves the burden into determinism

Factorio’s developer posts describe deterministic lockstep: machines exchange inputs and independently maintain matching simulations. Another post documents a map-generation desynchronization related to differing thread counts during Factorio 2.0 development. [Factorio](https://www.factorio.com/blog/post/fff-188)

That is a strong warning against treating input-only replication as “free compression.” It trades state bandwidth for duplicated computation and stringent reproducibility requirements.

For TCE, retain one authoritative Rust kernel. A render-only recording is useful, but **it is not a resumable simulation save**: resuming requires authoritative state, not just visible people and buildings.

### SUMO and MATSim/Via: closer analogues for a separate simulation and viewer

SUMO’s documentation explicitly motivates `libsumo` as an in-process alternative to TraCI’s protocol and socket overhead. The transferable lesson is to use a direct, coarse-grained library interface rather than thousands of fine-grained remote-style queries. [Eclipse SUMO](https://sumo.dlr.de/docs/Libsumo.html)

MATSim/Via demonstrates a complementary idea: visualization from a network plus simulation events. Via 22.2 added export of timestamped coordinate trajectories derived from event data. This supports the practicality of event-informed movement reconstruction, although it does not establish real-time UE crowd-rendering performance. [Simunto Documentation](https://docs.simunto.com/via/how-tos/modal-split.html)

### Unreal crowd systems: useful machinery, with version-specific risks

MassGameplay offers representation levels including high- and low-resolution Actors, instanced meshes, and no representation, with pooling and LOD controls. That is a useful presentation framework even when Rust owns all behavior. The overview also contains historical version notes and an Experimental warning for its ISM animation path; do not infer that every subsystem has the same maturity merely from the page’s UE 5.8 banner. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-gameplay-in-unreal-engine)

UE 5.8 additionally introduces Experimental MetaHuman Collections, transitioning between individual Actors and Instanced Skinned Meshes according to proximity. Treat that as a promising evaluation branch, not an essential dependency until it meets your packaged-build budget. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

---

## 4. Recommended TCE architecture

### 4.1 Establish a narrow, versioned presentation boundary

Use one render-extraction publisher after a **coherent simulation commit**. Do not scan partially updated kernel tables and assume that data-race freedom implies a logically consistent snapshot.

The pipeline should be:

| Stage | Responsibility |
| --- | --- |
| Rust simulation | Authoritative behavior, movement, construction, identities, and time |
| Rust render extraction | Build immutable, compact presentation records from committed state |
| Unreal ingestion worker | Acquire/copy frames, apply persistent updates, maintain indices and interpolation history |
| Unreal presentation preparation | Evaluate positions, choose representations, prepare contiguous instance batches |
| Game/render-thread integration | Commit bounded engine changes and submit owned render data |

Suggested frame metadata includes a schema version, world epoch, motion sequence, simulation timestamp, interest/scope epoch, required persistent-state sequence, and record counts.

Use generational handles and a separate mapping to live renderer slots. A renderer slot can be recycled; an old handle must never start referring to an unrelated newborn person.

Make world loads and discontinuous seeks explicit epoch changes. Reclaim obsolete renderer identities and catalogs: an endless simulation must not imply an endlessly growing live rendering index.

### 4.2 Use a latest-value mailbox for motion, a recoverable queue for changes

A single-producer/single-consumer triple buffer fits replaceable motion snapshots. The Rust `triple_buffer` crate documents this exact latest-value, nonblocking use case and supports in-place buffer reuse. [Docs.rs](https://docs.rs/triple_buffer/latest/triple_buffer/)

A bounded SPSC queue is appropriate for ordered changes. `rtrb` provides a fixed-capacity ring whose writes fail when full rather than silently overwriting unread entries. [Docs.rs](https://docs.rs/rtrb/latest/rtrb/)

Keep the concurrency machinery inside Rust and expose a small C ABI. Use fixed-width fields, `#[repr(C)]` layouts, explicit pointer/length pairs, and an acquire/copy/release lifetime. Do not export Rust `Vec`, `String`, or Rust-specific synchronization objects as cross-language data structures. Contain failures at the FFI boundary and make allocation ownership explicit. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For the first implementation, **copy each acquired frame into Unreal-owned history and release it promptly**. A triple buffer is not an interpolation-history buffer.

This matters because Unreal’s render thread can lag the game thread, and Epic’s threading guidance requires careful separation of game-thread objects and render-thread-owned proxy data. Never capture a temporarily leased Rust pointer in an asynchronous render command. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine)

Queue overflow needs a designed recovery path. With finite memory, arbitrary consumer stalls cannot coexist with indefinitely lossless delivery and a producer that never blocks. For the live renderer, mark affected chunks invalid and request current authoritative snapshots. A historical archive needs a separate durable-storage or backpressure policy.

### 4.3 Make motion and city revisions temporally consistent

A motion frame should identify the persistent-state revision it depends on.

For example, if a person starts walking on road revision 18, Unreal must not display that trajectory against revision 17 simply because the motion mailbox advanced faster than the city-update queue.

Apply city changes at their **effective presentation time**, not merely on arrival. Retain older resources while buffered motion still references them.

During overload, recover at chunk granularity. You can replace an affected chunk with a current snapshot or temporary proxy without forcing the entire city to wait for one expensive reconstruction.

For construction, send descriptors: stable IDs, authored module/template IDs, transforms, materials or variation seeds, construction stage, and revision. Generate or assemble the visual representation asynchronously where possible.

Avoid making each edit trigger a full-city mesh, collision, or navigation rebuild. Keep authoritative navigation in Rust; create Unreal collision only where the presentation and interaction design actually require it.

### 4.4 Decouple the three clocks

TCE needs three distinct clocks:

| Clock | Purpose |
| --- | --- |
| **Simulation time** | Authoritative ordering and outcomes |
| **Wall-clock publication schedule** | Limits extraction and delivery work |
| **Presentation time** | Determines the instant Unreal displays |

Start with a **20 Hz wall-clock publication cap** and a **50–100 ms wall-clock presentation delay**, then tune against measured jitter. Do not multiply publication frequency by the selected simulation speed.

Maintain a short mapping between committed simulation timestamps and monotonic wall time. Present slightly behind the latest committed timeline, using actual timestamps:

\[
\alpha=\frac{t\_{\text{present}}-t\_0}{t\_1-t\_0}
\]

Use 64-bit integer simulation timestamps across the boundary; convert only short local intervals for interpolation.

When speed changes, update a continuous clock mapping rather than repeatedly subtracting a newly scaled delay from the latest simulation time. That avoids accidental backward motion. Treat pause, load, seek, and large discontinuities explicitly.

If the kernel falls behind, follow **achieved simulation progress**, not requested speed. Preserve responsive camera/UI rendering, retain the last valid world state, and expose actual speed or lag rather than running a game-thread catch-up loop.

At extreme acceleration, change presentation policy. At 1,000 simulated seconds per wall second, 20 Hz snapshots are **50 simulated seconds apart**. A person may complete several meaningful movement segments between displayed states.

At that point, prefer correct sampled state, path-aware time-lapse, or aggregate visualizations. Do not interpolate a person straight through a neighborhood because their intermediate trip history was omitted. If a coarse simulation mode never produced detailed intraday interactions, the renderer cannot reconstruct those interactions as factual history.

### 4.5 Render people as representations, not 50,000 full Characters

I would use **Mass presentation entities with a custom ingestion processor**, while keeping simulation, pathfinding, and decision-making in Rust.

Begin with a small high-detail Actor pool—perhaps **100–300 as an initial tuning range**, not a promised capacity. Use instanced, baked-animation representations for the larger crowd and progressively cheaper distant representations.

Unreal should derive animation from activity, speed, clip, and phase anchors. Do not stream per-person bone matrices from Rust.

Use batched engine interfaces. `BatchUpdateInstancesTransforms` exposes bulk updates, including an overload with previous transforms; its documentation says to mark render state dirty only on the last update when updating many instances. A batch call still does work proportional to the data changed, but avoids unnecessary call and invalidation overhead. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/BatchUpdateInstancesTransforms?lang=en-US)

For the city, batch repeated authored modules by spatial region and mesh/material. Epic’s guidance distinguishes HISM’s usefulness for largely static instance sets from ISM’s per-instance GPU culling, and recommends ISM for Nanite-only usage. Do not blindly choose HISM for continuously moving people. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Keep visual costs conditional on projected importance: distant people should not inherit close-up hair, cloth, animation, collision, and shadow settings. Architectural variety should come primarily from combinations and per-instance variation of authored modules, rather than requiring every building to be a wholly unique rendering resource.

Evaluate UE 5.8’s newer skinned-instance crowd workflow separately. Preserve a simple backend interface so the simulation protocol does not depend on that experiment succeeding.

### 4.6 Implement in three measured stages

**Stage 1 — prove correctness and frame pacing.**  
Stream full 32-byte records at 20 Hz. Render 50,000 simple proxies, then representative low-detail animated people. Implement epochs, identities, lifecycle, chunk recovery, and the presentation clock before compression.

**Stage 2 — prove the actual visual budget.**  
Add realistic crowd assets, city modules, shadows, camera transitions, and the near-Actor pool. Profile packaged Windows builds with Unreal Insights, which supports CPU and GPU timing analysis. Optimize the bottleneck observed, not the bandwidth number that looks largest in isolation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

**Stage 3 — selectively reduce transport and preparation work.**  
Add interest tiers, path descriptors, sparse cold-state updates, and possibly quantized motion. Introduce GPU interpolation or a specialized rendering path only when CPU preparation or uploads remain limiting after ordinary batching.

For AI coding agents, make acceptance criteria executable: shared layout tests, golden byte fixtures, protocol-state tests, and captured-stream playback. A screenshot showing moving people does not test generation reuse, dropped deltas, or shutdown races.

The stress suite should include all 50,000 people moving, camera teleports, burst construction/demolition, road replacement during travel, pause/resume, rapid speed changes, 100–500 ms consumer stalls, queue overflow, world reload, and prolonged identity churn. Track p50/p95/p99 game-thread, render-thread, and GPU times alongside queue age, presentation lag, allocations, and bytes transferred.

---

## 5. Sources, code, and version applicability

The historical sources below document the systems of their time; they should not be read as statements about current game tick rates or current production configurations.

| Area | Direct sources | Version/date and use |
| --- | --- | --- |
| UE 5.8 crowd developments | [UE 5.8 announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available?utm_source=chatgpt.com) | June 2026; identifies Experimental MetaHuman Collections and the Actor-to-ISKM workflow. |
| Unreal presentation architecture | [MassGameplay overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-gameplay-in-unreal-engine?utm_source=chatgpt.com), [ISM documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com) | Current documentation consulted in September 2026; some Mass text retains historical version caveats. |
| Unreal replication | [Iris introduction](https://dev.epicgames.com/documentation/en-us/unreal-engine/introduction-to-iris-in-unreal-engine?utm_source=chatgpt.com), [Replication Graph](https://dev.epicgames.com/documentation/unreal-engine/replication-graph-in-unreal-engine?utm_source=chatgpt.com) | Current docs; useful design precedents, not a requirement to use networking locally. |
| Thread ownership and batching | [Threaded Rendering](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine?utm_source=chatgpt.com), [BatchUpdateInstancesTransforms](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/BatchUpdateInstancesTransforms?lang=en-US&utm_source=chatgpt.com) | Verify exact behavior against the chosen 5.8 patch. |
| Profiling | [Unreal Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine?utm_source=chatgpt.com) | Current Epic profiling documentation. |
| Snapshot techniques | [Snapshot Interpolation](https://gafferongames.com/post/snapshot_interpolation/?utm_source=chatgpt.com), [Snapshot Compression](https://gafferongames.com/post/snapshot_compression/?utm_source=chatgpt.com) | Glenn Fiedler, 2014–2015; worked examples and linked compression code. |
| Overwatch talks | [Gameplay Architecture and Netcode](https://gdcvault.com/play/1024001/-Overwatch-Gameplay-Architecture-and?utm_source=chatgpt.com), [Replay Technology](https://www.gdcvault.com/play/1024053/Replay-Technology-in-Overwatch-Kill?utm_source=chatgpt.com) | GDC 2017; historical original-Overwatch architecture. |
| Rocket League talk and code | [It IS Rocket Science!](https://www.gdcvault.com/play/1024972/It-IS-Rocket-Science-The?utm_source=chatgpt.com), [`boxcars` repository](https://github.com/nickbabcock/boxcars?utm_source=chatgpt.com), [`Frame` API](https://docs.rs/boxcars/0.11.5/boxcars/struct.Frame.html?utm_source=chatgpt.com) | Talk: 2018. Inspected Rust API: `boxcars` 0.11.5; community-maintained parser. |
| Factorio lessons | [FFF-188](https://www.factorio.com/blog/post/fff-188?utm_source=chatgpt.com), [FFF-415](https://www.factorio.com/blog/post/fff-415?utm_source=chatgpt.com) | 2017 and 2024; lockstep architecture and a 2.0-development desynchronization case. |
| Simulation/viewer coupling | [SUMO Libsumo](https://sumo.dlr.de/docs/Libsumo.html?utm_source=chatgpt.com), [Via 22.2 release](https://www.simunto.com/news/2022/via-222-released/?utm_source=chatgpt.com) | Libsumo page updated June 2026; Via example is version 22.2, 2022. |
| Interest-management research | [Boulanger, Kienzle, Verbrugge: Comparing Interest Management Algorithms for Massively Multiplayer Games](https://www.sable.mcgill.ca/~clump/papers/boulanger-06-comparing.pdf?utm_source=chatgpt.com) | 2006; primary experimental paper, with workload-specific results. |
| Historical motion-prediction research | [Singhal and Cheriton: Exploiting Position History for Efficient Remote Rendering in Networked Virtual Reality](https://direct.mit.edu/pvar/article/4/2/169/58896/Exploiting-Position-History-for-Efficient-Remote?utm_source=chatgpt.com) | 1995; foundational reading on position-history-based remote rendering. |
| Rust implementation building blocks | [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/?utm_source=chatgpt.com), [`rtrb`](https://docs.rs/rtrb/latest/rtrb/?utm_source=chatgpt.com), [Rustonomicon FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Retrieved crate docs show `triple_buffer` 9.0.0 and `rtrb` 0.4.0. Pin dependencies and test the C ABI. |

### Bottom line

**Begin with full 20 Hz render snapshots, a separate recoverable city-state stream, and batched crowd representations.** This gives TCE a transport budget of roughly **32 MB/s for 50,000 people**, understandable failure semantics, and a clear profiling baseline.

Add path-based reconstruction where it reduces measured work without inventing simulation outcomes. The architectural priority is not maximum compression: it is ensuring that **a slow renderer never stalls the kernel, skipped visual frames never corrupt the city, and neither side retains memory the other is allowed to overwrite**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927b8-f7cc-83ea-95ed-03bea0d270ef)
