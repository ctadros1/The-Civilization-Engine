# Architecture of large-scale agent-based simulation engines

**Engineering report for The Civilization Engine — evidence reviewed through September 27, 2026**

## Executive recommendation

For TCE, I recommend a **CPU-first, data-oriented, activity-driven hybrid simulation**, with Unreal acting as a presentation client rather than the simulation’s authoritative object model.

The central design decision is not “which ECS handles 50,000 entities?” It is:

> **Which situations require computation, at what simulated times, and at what level of physical detail?**

My proposed foundation is:

| Area | Recommended starting point |
| --- | --- |
| People and economic state | Dense, typed domain tables; hot/cold separation; selective struct-of-arrays |
| Identity | Generational handles for live objects, separate permanent IDs for history |
| Scheduling | Discrete events for activities and decisions; periodic batches for suitable systems |
| Movement | Individual routes and timed progress, with explicit capacity constraints; high-frequency interaction only where necessary |
| Queries | Uniform grids for nearby moving agents; hierarchical navigation and service indexes; R-trees for suitable static geometry |
| Parallelism | Region-owned work, immutable read phases, buffered intentions, explicit conflict resolution |
| Unreal integration | Versioned C ABI, batched commands, immutable render snapshots |
| GPU use | Rendering first; optional acceleration of isolated, regular numerical workloads later |

This is a strong architectural candidate for 10,000–100,000 persistent people. It is **not a demonstrated guarantee** of 50,000 TCE-complexity citizens, 1440p/60 FPS, and century-scale acceleration on the specified PC. None of the reviewed benchmarks establishes that combined workload.

The most important constraint is **strong acceleration**. At 50,000 people, a modest average of 48 meaningful decisions per person per simulated day becomes **14.6 million decisions per wall-clock second** when advancing one simulated year per minute—before movement, pathfinding, markets, construction, or history storage. That arithmetic should shape the engine from the beginning.

---

## 1. Options: how the main architectures work

### 1.1 Storage: data layout matters more than the ECS label

An ECS and a struct-of-arrays layout are not competing concepts. An archetype ECS is one way to organize component arrays; plain tables can achieve similar locality with less general machinery.

| Storage approach | How it works | Benefits | Costs and TCE fit |
| --- | --- | --- | --- |
| **Object graph** | Each person owns or references separate objects for needs, inventory, relationships, tasks, and so on | Natural domain modeling; flexible individual access | Pointer-heavy traversal and many allocations can become expensive. Useful for cold, irregular information—not the main update loop |
| **Dense array-of-structs** | A contiguous array of compact person records | Simple; good when an event touches most fields of one person | Bulk systems load irrelevant fields when records become large |
| **Struct-of-arrays / grouped columns** | Positions, needs, activities, affiliations, and other fields live in separate arrays | Bulk systems read only required data; straightforward partitioning | More bookkeeping; individual events may touch several arrays |
| **Archetype ECS** | Entities sharing component types occupy dense component tables/chunks | Efficient component queries; established borrowing and scheduling infrastructure | Structural changes move entities; excessive component combinations fragment work |
| **Sparse-set / hybrid ECS** | Components maintain dense values plus entity-to-position mappings | Convenient for optional, frequently added/removed components | Multiple-component access and iteration have different trade-offs from archetype tables |

Unity Entities 1.0.16 documents its archetype implementation as **16 KiB chunks**, each containing arrays of components and entity IDs. Adding or removing component types moves entities between archetypes. Bevy ECS likewise distinguishes table storage, optimized for iteration, from sparse-set storage, optimized for component insertion/removal. These are concrete examples of why “use ECS” is insufficiently specific. [Unity Documentation](https://docs.unity3d.com/Packages/com.unity.entities%401.0/manual/concepts-archetypes.html)

**For TCE, start with grouped columns rather than maximum fragmentation.** For example:

* A compact activity record: current activity, target, start time, expected completion, interruption version.
* Movement columns: route reference, segment, progress parameters, region.
* Economic columns: household, occupation, workplace, account.
* Cold records: names, biographies, detailed memories, authored descriptions.

A cooking-completion event may benefit from one compact record. A movement batch benefits from position and route columns. There is no requirement to split every scalar into its own allocation.

**Cache arithmetic, not a benchmark:** at 50,000 people, 64 bytes of frequently accessed state per person is 3.2 MB; 128 bytes is 6.4 MB; a 1 KiB person record is 51.2 MB. The difference matters even with 64 GB of RAM: fitting in memory is not the same as repeatedly accessing it cheaply.

Do not encode every occupation, belief, disease stage, and temporary activity as a different structural component combination. Prefer stable person components with enums, flags, and references where appropriate. Use component composition for genuine structural differences.

A small ECS remains reasonable when its tooling saves substantial development effort. `hecs` is explicitly a minimalist library rather than a full framework; standalone `bevy_ecs` provides a richer dependency-aware system scheduler. Neither requires adopting the corresponding renderer. [Docs.rs](https://docs.rs/hecs/latest/hecs/)

### 1.2 Generational handles are not historical identities

A reusable array index is unsafe as a long-lived reference: after a death and slot reuse, an old reference could accidentally identify a newborn.

Use a live handle conceptually containing:

```
LivePersonHandle = slot + generation
HistoricalPersonId = permanent, world-scoped identity
```

Validate the generation when dereferencing live handles. Keep historical IDs in genealogies, ownership histories, institutional records, and saved events.

Rust’s `slotmap` implements versioned keys and offers several storage choices. Its documentation explicitly notes that `DenseSlotMap` improves iteration at the cost of additional lookup indirection, and that generations eventually wrap. It is therefore useful machinery, not an eternal-identity scheme or an automatic SoA implementation. [Docs.rs](https://docs.rs/slotmap/latest/slotmap/)

For custom dense columns, maintain a handle-to-row map and reverse row-to-handle map. A swap-remove must update the moved person’s mapping and every aligned column. Keep that operation centralized and heavily tested.

### 1.3 Fixed-step, discrete-event, and hybrid scheduling

| Scheduler | Mechanism | Best use | Main limitation |
| --- | --- | --- | --- |
| **Fixed-step** | Advance by a constant simulated interval; run scheduled systems | Dense interactions, bounded-step numerical processes, simple debugging | Wastes work on unchanged agents; acceleration multiplies all recurring work |
| **Discrete-event** | Process the earliest scheduled change; jump over uneventful intervals | Sleeping, work completion, arrivals, deadlines, physiological thresholds | Event ordering, cancellation, dependencies, and queue overhead require care |
| **Hybrid** | Combine event-driven activities with periodic or locally time-stepped systems | TCE’s mixture of daily routines, transport, economics, and occasional intense interactions | Requires explicit rules governing interactions between time scales |

For an activity-based person, “working until noon” should usually be a state with a completion or interruption event—not thousands of calls to ask whether noon has arrived.

Similarly, an analytically evolving need might be represented as:

\[
h(t)=h(t\_0)+r(t-t\_0)
\]

Store its last evaluated value, time, and rate. Schedule the next threshold crossing. Recompute when work intensity, health, food intake, or another relevant input changes.

This is exact **within that chosen model** when its assumptions hold. It is not valid to skip over an unmodeled interruption.

A practical initial queue is a binary heap with integer timestamps and explicit tie-breaking. Rust’s `BinaryHeap` can form a min-priority queue using `Reverse`; its documentation specifies the operation costs. More elaborate bucketed queues should follow profiling, not precede a working model. [Rust Documentation](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html)

A useful event representation is:

```
(simulation_time, phase, tie_breaker,
 actor_handle, activity_version, event_kind, payload_reference)
```

The **entity generation** rejects events for a deleted person. The **activity version** rejects an obsolete completion event after that person abandoned the task. These solve different problems.

Avoid scheduling an entire lifetime. Keep the next relevant event or small set of events, then schedule successors. Bound canceled-event accumulation and detect zero-time event loops.

### 1.4 The movement problem determines achievable acceleration

Activity scheduling alone does not solve fast-forwarding if every walking person still requires ten authoritative steering updates per simulated second.

For TCE, I would distinguish:

**Individual transport simulation.** Each person owns a route, departure time, progress, destination, and reservations or capacity claims. Roads, doorways, bridges, and workplaces have meaningful constraints. Progress along an uncongested segment can be evaluated from elapsed time.

**Visual locomotion.** Unreal turns that transport state into footsteps, animation, turning, and permissible local offsets.

**Causally significant physical interaction.** Combat, a crush at a doorway, or movement that changes access to food requires an authoritative interaction model—not merely animation.

The recommended simplification is to use a tractable transport model **consistently**, including when observed. Do not secretly change economic or movement outcomes because the camera moved away.

MATSim provides a useful precedent: individual travelers can interact through link queues, earliest exit times, and capacity constraints rather than continuous vehicle physics. That is a modeling choice, not the elimination of individual identity. [ResearchGate](https://www.researchgate.net/publication/388814961_High-Performance_Mobility_Simulation_Implementation_of_a_Parallel_Distributed_Message-Passing_Algorithm_for_MATSim)

### 1.5 Spatial indexing: use different indexes for different questions

| Index | Mechanism and strengths | Weaknesses | Recommended role |
| --- | --- | --- | --- |
| **Uniform grid / spatial hash** | Map positions to cells; inspect intersecting cells and their occupants | Dense cells still produce many candidates; one scale does not suit every query | Nearby people, collision candidates, local perception |
| **Quadtree** | Recursively subdivide 2D space, concentrating subdivisions where needed | Traversal and moving-object maintenance add complexity | Highly uneven distributions or variable-scale regional queries |
| **R-tree / R\*-tree** | Group spatial objects by bounding envelopes | Overlapping envelopes can weaken pruning; moving objects require maintenance | Buildings, parcels, resource zones, other extent-bearing objects |
| **Navigation/service hierarchy** | Store connectivity and summaries such as reachable food, jobs, beds, or materials | Summaries must be maintained and invalidated | Long-range decisions and pathfinding |

Uniform-grid subdivision is a standard broad-phase technique; it eliminates many irrelevant pair tests but does not remove the quadratic worst case in a dense cluster. Epic’s Mass Avoidance also uses a 2D obstacle hash grid for neighboring entities. [NVIDIA Developer](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-32-broad-phase-collision-detection-cuda)

Quadtrees support adaptive spatial subdivision; R-trees support spatial objects with extent. Rust’s `rstar` supplies an R\*-tree implementation for points, lines, rectangles, and related objects. [D3.js](https://d3js.org/d3-quadtree)

**My TCE choice:** a chunked uniform grid for moving people, a separate connectivity hierarchy for travel, and service/resource indexes for decisions. Add an R-tree where irregular static geometry makes it useful.

“Find food” should not mean “scan all buildings and pathfind to each.” Query eligible sources, filter by the person’s knowledge and permissions, then evaluate a bounded set of reachable candidates. Engine indexes must not accidentally give people omniscient knowledge.

---

## 2. Trade-offs and benchmarks: what the measurements actually establish

### 2.1 Published results worth using

The measurements below are informative, but they are **not interchangeable units of intelligence**.

| System and tested version | Workload and hardware | Reported measurement | Interpretation |
| --- | --- | --- | --- |
| **MATSim Hermes, 2020/MATSim 13-era report** | Roughly 1 million travelers and 1.5 million links; 48-core Xeon server; approximately 130 GB used | Average reported iteration: **3 min 33 sec**, versus **8 min 45 sec** for QSim; total run reduced by 40% | Strong production evidence for changing algorithms and data layout; not a desktop benchmark or century simulation. [MATSim](https://matsim.org/news/2020/introducing-hermes/) |
| **Rust MATSim prototype v0.2.0, 2025 paper** | 491,175 travelers; Xeon Platinum 9242 cluster | **560 simulated seconds per wall second** with one process; **24,284** with 1,024 processes across 43 nodes | Strong ownership/partitioning precedent. The fastest result is emphatically not an i9 result. [ResearchGate](https://www.researchgate.net/publication/388814961_High-Performance_Mobility_Simulation_Implementation_of_a_Parallel_Distributed_Message-Passing_Algorithm_for_MATSim) |
| **FLAME GPU 2.0.0-rc0, 2023** | 80,000 Boids, 100 iterations; A100 80 GB GPU | Approximately **51 ms** for the simulation loop, excluding initialization | Derived throughput: approximately **157 million agent-iterations/sec**. These are flocking updates, not rich-person decisions, and not an RTX 4070 Ti result. [NVIDIA Developer](https://developer.nvidia.com/blog/fast-large-scale-agent-based-simulations-on-nvidia-gpus-with-flame-gpu/) |
| **Maintainer ABM comparison: Agents.jl 6.2.10, MASON 22, Mesa 3.2** | Published CI benchmark implementations | For “flocking-large,” relative runtimes include **1.0**, **0.61**, and **59.5**, respectively | Useful evidence that workload and implementation matter; ratios do not establish absolute i9 throughput. [GitHub](https://github.com/JuliaDynamics/ABM_Framework_Comparisons) |

For the MATSim Rust result, the “24 hours in about 3.5 seconds” headline is a **24-hour equivalent derived from its real-time ratio**; the scenario execution described in the paper extends to 36 simulated hours. More importantly, its mobility benchmark does not include centuries of evolving institutions, births, inventories, and technological change. [ResearchGate](https://www.researchgate.net/publication/388814961_High-Performance_Mobility_Simulation_Implementation_of_a_Parallel_Distributed_Message-Passing_Algorithm_for_MATSim)

**There is no defensible universal “agent decisions per second on a modern desktop CPU” number in these sources.** A decision might be a few comparisons, a neighbor search, a route search, or a multi-stage resource negotiation. I found no reproducible benchmark matching TCE’s rich-person workload on a 13th-generation i9.

Consequently, I would not convert an ECS iteration benchmark—or the FLAME GPU result—into a promised citizen count.

### 2.2 Translate the product goal into required work

Define acceleration explicitly:

\[
A=\frac{\text{simulated seconds}}{\text{wall-clock seconds}}
\]

For constant population \(N\), with \(d\) meaningful decisions per person per simulated day:

\[
D=\frac{N d A}{86{,}400}
\]

The following is **planning arithmetic, not measured performance**, assuming 50,000 people and 48 decisions per person per day:

| Desired advancement | Required meaningful decisions/sec |
| --- | --- |
| One simulated day per wall-clock second | 2.4 million |
| One simulated year per wall-clock minute | 14.6 million |
| Ten simulated years per wall-clock minute | 146 million |
| One hundred simulated years per wall-clock minute | 1.46 billion |

These figures exclude route transitions, market operations, environment updates, and bookkeeping.

Another way to expose the constraint: 100 years at that decision density contains **87.6 billion meaningful decisions**. At a hypothetical measured throughput of one million such decisions/sec, those decisions alone take approximately **24.3 hours**.

Likewise, using 365-day years:

| Acceleration | Wall time for 100 simulated years |
| --- | --- |
| 1,000× | 36.5 days |
| 10,000× | 3.65 days |
| 100,000× | 8.76 hours |
| 1,000,000× | 52.6 minutes |

This does not imply that TCE needs 48 expensive decisions per person per day. It shows why the distinction between **activity continuation, cheap state advancement, and genuine reconsideration** is fundamental.

A person can remain individually represented while executing an established routine. It is unnecessary to perform a fresh job search, route search, and utility evaluation every time that routine advances.

### 2.3 Performance versus complexity

For a solo developer, I would rank the interventions in this order:

1. **Stop unnecessary work:** persistent activities, threshold events, cached routes, dirty-region updates.
2. **Bound queries and interactions:** local neighborhoods, service indexes, explicit ownership.
3. **Improve layout and batch execution:** compact hot state, allocation-free common paths.
4. **Parallelize independent work.**
5. **Specialize queues, vectorization, or GPU kernels where measurements justify them.**

That ordering is consistent with Hermes’ emphasis on event-driven execution, contiguous structures, precomputation, and compact hot paths—not simply adding more threads. [MATSim](https://matsim.org/news/2020/introducing-hermes/)

Pure GPU execution offers impressive throughput for suitable models, but implementing resource conflicts and heterogeneous behavior can require substantial additional machinery. FLAME GPU 2 explicitly supports iterative submodels for situations such as resource-conflict resolution. For TCE, that complexity must also compete with the GPU’s rendering duties. [FLAME GPU](https://www.flamegpu.com/about/)

---

## 3. Precedents: how the requested systems are structured

### MATSim: individual plans, queue-based movement, repeated daily simulation

MATSim separates mobility execution from scoring and replanning. Its conventional workflow repeatedly evaluates variants of a simulated day; this is not equivalent to simulating successive historical days.

The Rust prototype combines a priority queue of activity completions with time-stepped network processing and message exchange between owned partitions. It is therefore a **hybrid**, not a pure event-only engine. [ResearchGate](https://www.researchgate.net/publication/388814961_High-Performance_Mobility_Simulation_Implementation_of_a_Parallel_Distributed_Message-Passing_Algorithm_for_MATSim)

**Lesson for TCE:** copy the separation between activity plans, transport execution, and reconsideration. Copy partition ownership. Do not assume that a transport plan contains the same work as a person’s complete social and economic life.

### FLAME GPU: dense state populations, message passing, staged functions

FLAME GPU 2 expresses agent behavior as functions operating over populations and states, with message-based interactions and GPU execution. State grouping and staged execution make otherwise enormous populations tractable when their computations fit this structure. Its APIs support CUDA C++ and a Python-facing workflow; the performance comes from GPU execution, not ordinary Python object loops. [GitHub](https://github.com/FLAMEGPU/FLAMEGPU2)

**Lesson for TCE:** adopt its explicit communication and homogeneous batches. Consider GPU processing for sufficiently large, regular fields or local-interaction kernels. Do not begin by placing all law, ownership, relationships, planning, and transactions on the GPU.

### Repast HPC: explicit ownership and synchronized nonlocal copies

Repast HPC is a C++/MPI framework. Agents are held in contexts and exposed through projections such as spaces, grids, and networks. A process owns local agents; nonlocal agents can exist as synchronized copies for interaction. Scheduling, migration, and synchronization are explicit concerns. The official site lists release **2.3.1**, dated October 2021. [Repast Suite](https://repast.github.io/docs/repast_hpc.pdf)

**Lesson for TCE:** an agent’s state needs a clear authority. Read-only neighboring copies and boundary messages are useful concepts even inside a single desktop process. MPI deployment and distributed infrastructure are unnecessary starting costs for TCE.

### Mesa: accessible model construction, now with stable event scheduling

Mesa provides Python-based agents, `AgentSet` activation, spatial environments, and numerical property layers. Its **3.5** series stabilizes event scheduling through `mesa.time` and model scheduling methods; descriptions that treat all Mesa event scheduling as experimental are outdated. The 3.5.1 release also addresses canceled-event compaction and related scheduling behavior. [Mesa](https://mesa.readthedocs.io/v3.5.1/overview.html)

**Lesson for TCE:** Mesa is useful for small reference models and experiments that clarify behavior. It is not my preferred production Rust-DLL foundation. Also, the Mesa 1.0 result in the 2023 FLAME comparison should not be represented as a benchmark of Mesa 3.5.

### Dwarf Fortress: multi-rate updates and active-region filtering

A particularly useful primary source is Tarn Adams’ **2019 technical interview**, which describes the actual fortress loop: immediate actions update frequently, while job assignment, institutional information, item processes, and other systems run at different intervals. Flags allow unchanged map sections to be skipped. He also describes world-history generation as a separate strategy-like simulation whose events become historical records. [Game Developer](https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams)

That account predates newer work. Bay 12’s June 2023 developer log explicitly discusses adding a multithreading option, so “Dwarf Fortress is wholly single-threaded” is not an adequate current architectural summary. [Bay 12 Games](https://www.bay12games.com/dwarves/)

**Lesson for TCE:** multi-rate execution, inactive-region skipping, and history as recorded consequences are valuable. Its world-history scale does not demonstrate the same number of fully detailed local creatures. Also watch non-person objects: Adams explicitly identifies large accumulated item populations as a performance concern. [Game Developer](https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams)

### RimWorld: persistent jobs rather than constant full replanning

RimWorld’s documented modding model distinguishes job selection, job drivers, and sequential task steps called *toils*. An older primary mod-author tutorial demonstrates this structure and its interruption conditions; it should be read as an architectural illustration, not current API documentation. [Ludeon Studios](https://ludeon.com/forums/index.php?topic=16405.0)

For current threading, the official **1.6 announcement in June 2025** states that pathfinding is fully multithreaded and batched, with lighting also multithreaded and work spread more evenly. Older advice saying its pathfinding is necessarily single-threaded is obsolete. [Ludeon Studios](https://ludeon.com/blog/2025/06/announcing-odyssey-and-update-1-6/)

**Lesson for TCE:** choose a job relatively infrequently, execute it efficiently, and interrupt for explicit reasons. Rich per-pawn simulation remains a different workload from a 50,000-person city.

### Songs of Syx: hierarchical searches and shared summaries

A 2019 developer explanation describes a custom pathfinding hierarchy: individual tiles plus **16×16-tile chunks**, with chunk-level information about jobs, resources, corpses, and other destinations. Routes use coarse search followed by local refinement. The developer also describes a decaying route-use cost that reduces crowding. [Reddit](https://www.reddit.com/r/songsofsyx/comments/dirkmw/crowd_control/)

The claim of thousands of paths per second in that discussion lacks a controlled hardware/workload specification, so it should not be treated as a portable benchmark.

**Lesson for TCE:** this is one of the most relevant precedents. The important optimization is not merely faster A\*: it is supplying agents with efficient shared search structures so they do less searching.

### Cities: Skylines II: data-oriented execution does not remove algorithmic bottlenecks

The official 2024 code-modding documentation exposes the game’s ECS-oriented toolchain and Burst processing. This is a relevant commercial precedent for data-oriented systems rather than a conventional object-and-update-callback architecture. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/modding/dev-diary-3-code-modding)

However, the developer’s **September 3, 2026** discussion still identifies CPU pathfinding as a major challenge and explains that some performance improvements may require changes to simulation behavior. It also illustrates how logistics debugging crosses business decisions, availability, and route selection. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/collected-questions)

**Lesson for TCE:** ECS improves execution infrastructure. It does not automatically control route demand, economic feedback, invalidation storms, or semantic complexity.

---

## 4. Recommended implementation for TCE

### 4.1 Separate the authoritative model from its execution and presentation

I would structure the kernel around five responsibilities:

| Responsibility | Owns |
| --- | --- |
| **World state** | People, households, institutions, buildings, resources, topology, historical identities |
| **Activity and event scheduler** | Logical time, activity continuations, interruptions, periodic deadlines |
| **Query infrastructure** | Spatial cells, navigation hierarchy, service directories, cached derived information |
| **Execution and commit** | Independent work batches, intentions, resource conflicts, state transitions |
| **Presentation interface** | Read-only snapshots, inspection records, commands, diagnostics |

Keep authored definitions separate from mutable state. Governments, technologies, production rules, and building components should refer to validated definitions or compiled rule representations rather than duplicating large descriptions across people.

Crucially, **a person remains an individual record even while receiving no CPU time**. Sleeping, continuing a routine, or traveling along an uncomplicated segment does not require replacing that person with a population cohort.

### 4.2 Establish scheduling semantics before parallelism

At each relevant logical time, process an explicit sequence such as:

```
complete due activities
→ update affected availability and derived state
→ evaluate eligible decisions
→ collect requests and intentions
→ resolve shared-resource conflicts
→ commit changes and schedule successors
```

The exact phases are model choices. Document them.

When two hungry people request the last meal, an immutable snapshot prevents half-updated reads, but **does not resolve the conflict**. The food owner must authorize at most one transfer. Decide whether simultaneous demand uses priority, price, a seeded lottery, or another modeled rule. A fixed ascending person ID should not accidentally become a lifelong advantage.

Start with one correct event coordinator. Parallelize dense phases and independent same-time work first. Do not execute later events early merely because a worker is available.

For regional queues, advancing a region ahead of others requires a proven boundary rule: it must not miss an earlier incoming effect. Until that is established, use explicit synchronization boundaries rather than implementing optimistic rollback.

### 4.3 Use a private worker pool, not one task per citizen

A private Rayon pool is a sensible starting point. Its configuration permits an explicit worker count rather than forcing TCE to consume every logical processor through the global pool. [Docs.rs](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html)

For the specified i9, benchmark several worker counts **with Unreal running**. Do not assume that the maximum thread count maximizes joint performance, or that all cores have identical execution characteristics.

Work units should be batches: a region’s due decisions, a set of independent paths, a movement column range, or one market’s requests. Avoid an operating-system task, mutex, or asynchronous mailbox per person.

Partition by measured work, not equal map area. One dense capital may be more expensive than many rural regions. Keep authority for hot resources explicit; unrestricted shared mutation around a central market can negate otherwise good parallelism.

### 4.4 Make pathfinding demand a first-class budget

Use hierarchical routes and reuse valid route structure. Recompute when topology, accessibility, destination, or relevant costs change—not simply because a new frame or minute arrived.

Maintain versioned caches. A bridge demolition should invalidate affected connectivity and routes; it should not blindly trigger a complete world-wide search storm.

Destination search and route search should be distinct stages. Failed job searches also need a policy: retry on meaningful availability changes or bounded deadlines rather than continuously rediscovering that no work exists.

This recommendation directly follows the shared-index approach described by Songs of Syx and the continuing pathfinding pressure acknowledged by Cities: Skylines II. [Reddit](https://www.reddit.com/r/songsofsyx/comments/dirkmw/crowd_control/)

### 4.5 Keep Unreal’s frame rate independent of simulation progress

Use a versioned C interface around an opaque kernel instance. Exchange fixed-layout records and batched buffers; do not pass Rust `Vec`, `String`, references, or C++ standard-library containers across the boundary.

Define allocation ownership, snapshot lifetimes, error handling, and panic behavior. Prevent unwinding across an incompatible ABI. Stop and join kernel workers before unloading its DLL. Rust’s FFI guidance and Epic’s third-party-library documentation are the relevant implementation references. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Unreal should consume immutable snapshots rather than lock the simulation’s live state during rendering. Intermediate **presentation snapshots** may be dropped when the renderer falls behind; authoritative simulation events and accepted commands may not.

An initial experiment might publish snapshots at 10–20 wall-clock updates/sec, with interpolation or route evaluation between them. As a payload calculation, 50,000 records × 32 bytes × 20 updates/sec is **32 MB/sec**, excluding copies and other overhead. Batched transfer is therefore worth testing before inventing a complicated zero-copy interface.

Do not represent every person as a fully ticking Actor with an independent authoritative behavior tree. Use representation tiers and batched rendering. Mass can assist presentation, but it should not become a second simulation authority. Epic still labels Mass Avoidance experimental in its 5.8 documentation, so validate the exact subsystem and engine revision used. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/mass-avoidance-overview-in-unreal-engine)

At extreme acceleration, use a timelapse or strategic presentation. There is no requirement to visibly animate every intermediate footstep across years of simulated history.

### 4.6 Preserve model fidelity independently of the camera

There are two different optimizations:

**Presentation LOD:** fewer bones, simpler meshes, less frequent animation, hidden off-screen representations.

**Simulation approximation:** fewer contacts, altered movement constraints, batched economic decisions, aggregated conflict resolution.

The former need not change history. The latter can.

For TCE, keep presentation LOD camera-dependent, but keep authoritative modeling rules camera-independent. Where a simplified transport or contact model is necessary, apply it consistently and validate its consequences.

If detailed combat or a crowded emergency requires more authoritative work, let maximum acceleration temporarily decrease. Do not silently skip causal events to maintain a displayed speed multiplier.

### 4.7 Treat “endless” as a storage and lifecycle requirement

The active population is not the lifetime population.

Archive deceased people’s compact historical records instead of retaining their full active simulation components. Keep relationships sparse. Give histories and diagnostics explicit retention policies.

Likewise, avoid one active entity per fungible grain of wheat or every routine action ever performed. Individual people can remain individually simulated while inventories use quantities and history records meaningful transitions.

Save/load must preserve—or correctly reconstruct—activity continuations, pending events, reservations, random state, and cache validity. A saved person should not resume with a completion event belonging to an earlier abandoned task.

### 4.8 Build the benchmark before the full civilization

The first vertical slice should contain housing, work, food, travel, reservations, and at least one disruption. Run it at **10k, 50k, and 100k people**, with both dispersed settlements and one dense city.

Measure four distinct things:

| Measurement | Why it matters |
| --- | --- |
| **Handler-only decisions/sec** | Identifies expensive decision logic |
| **End-to-end events/sec** | Includes scheduling, queries, cancellation, and commit |
| **Simulated days per wall second** | Measures the actual product capability |
| **Joint Unreal frame times and simulation throughput** | Tests the real hardware-sharing constraint |

Also record path searches and expanded nodes, query candidate counts, allocations, cache misses where available, canceled-event ratios, queue size, memory growth, and long-tail stalls.

Use a single-thread reference run for debugging even though exact reproducibility is not a product requirement. Test conservation of goods and money, unique ownership, valid references, reservation cleanup, and event-time monotonicity. Compare accelerated and unaccelerated execution under the same logical rules.

For AI-assisted development, require each new subsystem to declare its read set, write authority, event triggers, invariants, and performance counter. That is more valuable than letting coding agents independently add unrestricted callbacks to a global world object.

---

## 5. Sources, code, talks, and version scope

These are the most useful starting points; dates and versions refer to the material used here, not a claim that an old benchmark measures the latest release.

| Topic | Primary references | Applicable scope |
| --- | --- | --- |
| Data-oriented storage | [Unity archetype/chunk documentation](https://docs.unity3d.com/Packages/com.unity.entities@1.0/manual/concepts-archetypes.html); [Bevy ECS](https://docs.rs/bevy_ecs/latest/bevy_ecs/?utm_source=chatgpt.com); [hecs](https://docs.rs/hecs/latest/hecs/?utm_source=chatgpt.com) | Entities **1.0.16**; retrieved Rust documentation **bevy\_ecs 0.19.1**, **hecs 0.11.1** |
| Identity and queues | [slotmap](https://docs.rs/slotmap/latest/slotmap/?utm_source=chatgpt.com); [Rust BinaryHeap](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html?utm_source=chatgpt.com) | **slotmap 1.1.1**; current standard-library documentation |
| Spatial indexing | [NVIDIA grid broad-phase chapter](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-32-broad-phase-collision-detection-cuda?utm_source=chatgpt.com); [rstar](https://docs.rs/rstar/latest/rstar/?utm_source=chatgpt.com) | Classic algorithm reference; **rstar 0.13.0** |
| MATSim | [2025 distributed-Rust paper](https://www.mdpi.com/2078-2489/16/2/116?utm_source=chatgpt.com); [Rust implementation](https://github.com/matsim-vsp/matsim-rust); [Hermes developer report](https://matsim.org/news/2020/introducing-hermes/?utm_source=chatgpt.com) | Paper compares **MATSim 15.0** and prototype **0.2.0**; Hermes report is **2020** |
| FLAME GPU | [Architecture and benchmarks](https://developer.nvidia.com/blog/fast-large-scale-agent-based-simulations-on-nvidia-gpus-with-flame-gpu/?utm_source=chatgpt.com); [code](https://github.com/FLAMEGPU/FLAMEGPU2?utm_source=chatgpt.com); [GTC talk and publications](https://flamegpu.com/about/) | Benchmark **2.0.0-rc0**, **2023**; Paul Richmond’s introductory talk **GTC 2021** |
| Repast HPC | [Official overview](https://repast.github.io/repast_hpc.html?utm_source=chatgpt.com); [technical manual](https://repast.github.io/docs/repast_hpc.pdf?utm_source=chatgpt.com); [code](https://github.com/Repast/repast.hpc?utm_source=chatgpt.com) | Listed release **2.3.1**; foundational manual **2013** |
| Mesa | [Versioned overview](https://mesa.readthedocs.io/v3.5.1/overview.html?utm_source=chatgpt.com); [release notes](https://github.com/mesa/mesa/releases) | Stable **3.5.x** event scheduling; older benchmark versions identified separately |
| Dwarf Fortress | [Creator’s technical interview](https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams?utm_source=chatgpt.com); [official development log](https://www.bay12games.com/dwarves/?utm_source=chatgpt.com) | Detailed loop description **2019**; later threading changes documented separately |
| RimWorld | [Official 1.6 announcement](https://ludeon.com/blog/2025/06/announcing-odyssey-and-update-1-6/?utm_source=chatgpt.com); [job/toil implementation tutorial](https://ludeon.com/forums/index.php?topic=16405.0&utm_source=chatgpt.com) | Threading changes **1.6 / 2025**; tutorial **Alpha 12-era**, not current API guidance |
| Songs of Syx | [Developer’s pathfinding explanation](https://www.reddit.com/r/songsofsyx/comments/dirkmw/crowd_control/?utm_source=chatgpt.com) | **2019** architecture explanation; not a controlled current-build benchmark |
| Cities: Skylines II | [Code-modding diary](https://www.paradoxinteractive.com/games/cities-skylines-ii/modding/dev-diary-3-code-modding?utm_source=chatgpt.com); [developer performance discussion](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/collected-questions?utm_source=chatgpt.com) | **March 2024** toolchain; **September 2026** performance discussion |
| Rust–Unreal boundary | [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com); [Epic DLL/library integration](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com) | Current Rust guidance; retrieved **Unreal 5.8** documentation |
| Additional Rust ABM precedent | [krABMaga documentation](https://krabmaga.github.io/?utm_source=chatgpt.com); [code](https://github.com/krABMaga/krABMaga?utm_source=chatgpt.com) | Discrete-event Rust framework; **0.6.2** surfaced in June 2026; inspect its experimental parallel components before adopting |

**Bottom line:** build TCE so that cost follows **meaningful changes, interactions, and searches**, not population multiplied by rendered frames. Dense storage and parallel execution then amplify a sound model; they cannot rescue a model that repeatedly asks tens of thousands of unchanged people the same questions.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92680-0034-83e9-b4c9-c8598fa287fd)
