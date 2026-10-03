# Parallel simulation in Rust: throughput and correctness for TCE

**Engineering report — September 2026**

## Executive recommendation

Build TCE around **explicit simulation phases, immutable read snapshots, disjoint writes, and resource-owner arbitration**, executed on a **private, bounded Rayon thread pool**.

The essential separation is:

> **Agents decide in parallel. Shared resources resolve conflicts under explicit rules. Only completed, validated state becomes visible.**

Do not begin with a lock around every person, a thread per settlement, or a custom fiber scheduler. Those approaches either leave important simulation semantics implicit or create substantial infrastructure work before TCE’s actual bottlenecks are known.

The most important distinction is that **race-free does not mean order-independent**. Safe Rust prevents data races when its underlying unsafe code and foreign interfaces are sound; it does not prevent logical races, deadlocks, or simulation outcomes accidentally determined by execution order. [doc.rust-lang.org](https://doc.rust-lang.org/nomicon/races.html)

For your hardware, also separate **maximum headless simulation throughput** from **maximum throughput while Unreal maintains 60 fps**. The best thread count need not be the same.

---

# 1. Options: what the main techniques actually solve

These techniques are complementary. Double buffering controls **visibility**; partitioning controls **ownership and locality**; a job system controls **execution**; an entity-component system controls **storage and declared access**.

| Technique | How it works | Main benefit | Main limitation |
| --- | --- | --- | --- |
| **Double-buffered state** | Read state for tick *t*; write separate state for *t+1*; publish at a boundary. | Agents cannot accidentally observe partially completed updates. | Does not itself resolve competing claims on money, food, space, or jobs. |
| **Intent buffers and resource owners** | Agents emit proposed actions; designated owners resolve conflicts and produce accepted transactions. | Makes allocation and transaction rules explicit. | A popular resource can become a serial bottleneck. |
| **Spatial partitioning** | Divide space into cells or regions; process local agents together; read neighboring cells through an immutable index. | Better locality and fewer candidate interactions. | Dense cities, border interactions, and long-range relationships complicate ownership. |
| **Phase/job graph** | Run independent jobs concurrently; dependencies establish visibility boundaries. | Parallelism without unnecessary global serialization. | Missing dependencies create semantic bugs even when accesses are memory-safe. |
| **ECS scheduling** | Systems declare component/resource access; compatible systems run concurrently. | Useful organization and automatic access-conflict scheduling. | Access conflicts do not specify the *correct* gameplay order. |
| **Shared mutable state with locks/atomics** | Synchronize access to live objects. | Convenient for small, genuinely shared services. | Contention, lock ordering, and scheduler-dependent outcomes become pervasive. |

There are direct precedents for these distinctions: krABMaga describes double-buffered fields; Unreal’s Tasks system provides dependency graphs; and Bevy explicitly distinguishes conflicting access from an indeterminate system order. [JASSS](https://www.jasss.org/27/2/4.html)

## 1.1 Double buffering: necessary visibility discipline, not a complete transaction system

Consider two people trying to purchase the last unit of grain.

With live mutable state, one person may see the grain before another removes it. A mutex can prevent simultaneous modification, but the winner may simply be whichever worker acquires it first.

With a snapshot, both people correctly see one available unit. However, **both can still propose buying it**. Applying both proposals would oversell the grain.

The solution is to separate observation, decision, and resolution:

```
Committed world S[t]
    |
    +--> Parallel decisions: read S[t], produce private intents
    |
    +--> Resolve conflicting intents under simulation rules
    |
    +--> Apply accepted results to candidate S[t+1]
    |
    +--> Validate, complete structural changes, publish
```

For TCE, define this contract explicitly:

\[
I\_t = \operatorname{decide}(S\_t,\text{inputs}\_t)
\]\[
R\_t = \operatorname{resolve}(S\_t,I\_t)
\]\[
S\_{t+1} = \operatorname{commit}(S\_t,R\_t)
\]

These equations are a proposed architectural contract, not a requirement to copy every world object each tick.

Some operations can run directly into disjoint next-state arrays: hunger progression, private timers, or movement proposals. Others must pass through conflict resolution: purchases, harvesting, employment, construction reservations, and combat involving multiple participants.

**“Everybody reads the previous tick” is itself a modeling choice.** A movement phase followed by perception of the completed movement state is also valid. What is invalid is allowing perception to see an arbitrary mixture because some movement jobs happened to finish first.

Synchronous updates can also introduce artificial lockstep behavior. Huberman and Glance’s classic evolutionary-game study demonstrated substantially different outcomes when changing temporal updating assumptions. Therefore, removing accidental update order does not remove the need to validate timestep and scheduling semantics. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC47213/)

## 1.2 Spatial ownership versus logical ownership

Spatial partitioning suits walking, local observation, harvesting, and neighborhood interactions. It is not a complete ownership model for an economy.

A household can have members in several cells. A merchant can trade with multiple settlements. One wallet can fund transactions in several markets. A government can govern disconnected territory.

My recommendation is therefore **two overlapping partition schemes**:

* **Spatial partitions** for proximity calculations and movement.
* **Logical owners** for accounts, inventories, markets, institutions, and other shared constraints.

Ownership should mean “the only job allowed to resolve this object’s changes during this phase,” not necessarily “a permanent operating-system thread.”

For interactions crossing spatial boundaries, use read-only neighboring data from the same state version. Perform migration and membership changes at a boundary. Each person must have exactly one update owner for that epoch.

## 1.3 Rayon versus custom thread pools

Rayon uses work stealing: workers prefer local tasks and steal from others when they run out. A private `ThreadPool` lets you set the worker count; operations invoked inside `pool.install(...)` use that pool. [Docs.rs](https://docs.rs/rayon/latest/rayon/struct.ThreadPool.html)

| Execution approach | Fit for TCE | Assessment |
| --- | --- | --- |
| **Private Rayon pool** | Chunked agent updates, spatial queries, independent resource groups, reductions. | **Best starting point:** established library, relatively small integration surface. |
| **Fixed region-to-thread assignment** | Uniform, stable workloads with strong locality. | Simple, but an expanding capital can overload one thread while others idle. |
| **Custom work-stealing/fiber pool** | Proven need for affinity, priorities, specialized waits, or tight host scheduling integration. | High development and debugging cost; not automatically faster. |
| **Unreal-owned jobs calling Rust** | A later alternative when competing engine/kernel pools demonstrably hurt frame time. | Shares host scheduling, but requires a carefully designed cross-language task interface. |

Use **chunk jobs**, not one heavyweight task per person. Start by testing chunk sizes such as 64, 256, and 1,024 agents. Choose from measurements rather than fixing a universal size.

Prefer chunk-owned scratch buffers to assumptions about “one buffer per worker.” Rayon documents that calling into a pool changes thread-local context, and nested scheduling can have surprising execution-order behavior. [Docs.rs](https://docs.rs/rayon/latest/rayon/struct.ThreadPool.html)

A custom scheduler is justified only after profiling identifies a problem that cannot reasonably be solved through task granularity, workload partitioning, or a bounded Rayon pool.

---

# 2. Trade-offs and benchmark evidence

## 2.1 Closely relevant desktop measurements

A 2025 krABMaga/Bevy study tested an **i9-14900KF, 64 GB RAM, Ubuntu 24.04.1**, averaging five runs. The following are its reported **whole-run speedups for 2,048,000 agents**, relative to the same implementation using one thread. [CEUR-WS](https://ceur-ws.org/Vol-4124/paper43.pdf)

| Threads | Wolf–Sheep–Grass | Boids |
| --- | --- | --- |
| 1 | 1.00× | 1.00× |
| 2 | 1.78× | 1.99× |
| 4 | 2.91× | 3.86× |
| 8 | 4.45× | 6.76× |
| 16 | 5.09× | 10.04× |
| 20 | 5.27× | 11.38× |
| 24 | Not reported | Not reported |

The implementation used storage recycling to reduce entity-lifecycle overhead. These are **research-prototype results, not TCE predictions**: different CPU generation, operating system, population, and behavior. The experiment does not establish a universal eight-thread ECS limit or isolate P/E-core scheduling effects. [ceur-ws.org](https://ceur-ws.org/Vol-4124/paper43.pdf)

**I did not find a directly comparable published 1–24-thread benchmark for a 10k–50k-person Rust economy running inside Unreal on your target configuration.** The available desktop evidence reaches 20 threads; the remaining points need measurement, not extrapolation.

## 2.2 Why thread count alone is a poor predictor

A useful decomposition is:

\[
T\_{\text{tick}} =
T\_{\text{serial}}
+T\_{\text{parallel work}}
+T\_{\text{coordination}}
+T\_{\text{memory stalls}}
+T\_{\text{load imbalance}}
\]

The crucial quantity is the longest dependency path through the tick—not the sum of how busy all cores appear.

As an **idealized calculation**, with identical workers, no additional parallel overhead, and 10% unavoidable serial work:

\[
S(24)=\frac{1}{0.10+0.90/24}\approx 7.27
\]

That is not a forecast for the i9. It illustrates why parallelizing agent decisions while leaving a large serial commit stage can disappoint.

The main trade-offs for TCE are:

**More buffering versus more memory traffic.** Double buffering makes ownership much easier to reason about, but copying large object graphs every tick can consume the benefit. Buffer only hot, frequently updated state; share or version cold data.

**More partitions versus more coordination.** Smaller jobs improve load balancing, but eventually scheduling, event merging, and index construction dominate.

**More concurrency versus a longer conflict-resolution stage.** Parallel proposals are easy; allocating scarce resources across interacting transactions is the difficult part.

**More threads versus worse frame latency.** Unreal and the simulation share execution resources and caches. A faster isolated kernel can still produce worse presentation latency once embedded.

## 2.3 What “24 threads” means on your i9

Assuming a desktop **13900K-class** processor, Intel specifies **8 performance cores plus 16 efficiency cores: 24 physical cores and 32 hardware threads**. The exact SKU should be recorded rather than inferred from “13th-gen i9.” [Intel](https://www.intel.com/content/www/us/en/products/sku/230496/intel-core-i913900k-processor-36m-cache-up-to-5-80-ghz/specifications.html)

Consequently:

* Twenty-four Rayon workers do not necessarily mean one worker on each physical core.
* Performance cores, efficiency cores, and SMT siblings are not interchangeable.
* An apparent scaling knee may combine algorithmic limits, memory pressure, and placement effects.

Start with normal OS scheduling. Test explicit placement only as a controlled experiment. Windows CPU Sets provide a softer affinity mechanism; use topology information rather than hard-coded processor numbers. Avoid changing process-wide defaults casually inside a DLL, because the process also contains Unreal. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/procthread/cpu-sets)

---

# 3. Precedents: what games, engines, and research projects learned

## Factorio: parallelize independent work, not just expensive work

Factorio’s 2017 engineering discussion described how cache effects and false sharing complicated multithreaded entity updates, while read-oriented rendering preparation was more amenable to parallelization. This is historical evidence, not a claim about the entire current engine. [Factorio](https://www.factorio.com/blog/post/fff-215)

For its 2.0 development, the team reported approximately **9.5% improvement on a playtesting save** after parallelizing suitable control behaviors, versus a reported **14.9× improvement on a synthetic combinator-heavy save**. A separate electric-network experiment reduced that subsystem from roughly **0.5 ms to 0.39 ms**, but did not improve the overall save’s performance; memory throughput was the limiting factor. [Factorio](https://www.factorio.com/blog/post/fff-421)

**TCE lesson:** independence and memory behavior matter more than whether a subsystem looks expensive in isolation. A synthetic “all agents think” benchmark is insufficient.

## BioDynaMo: locality and data structures determine scalability

The PPoPP 2023 BioDynaMo paper reports whole-simulation speedups of **60.7–74.0×**, median **64.7×**, on a **72-physical-core server with hyperthreading enabled**. Its optimizations included spatial indexing, memory layout, and parallel handling of locally accumulated results. These are server-scale biological workloads, not desktop citizen simulation benchmarks. [arXiv](https://arxiv.org/pdf/2301.06984)

**TCE lesson:** make the data traversal and spatial-query design parallel-friendly before spending time on a more elaborate scheduler.

## krABMaga: useful Rust precedent, but separate the versions and experiments

The 2024 krABMaga paper describes read/write field buffering to prevent agents from observing current-step changes accidentally. Its current **0.6.2 documentation still labels single-simulation parallel scheduling experimental**; parallel parameter exploration is a separate capability. [JASSS](https://www.jasss.org/27/2/4.html)

Its framework-comparison benchmark page measures speedups **against other frameworks**, not speedup from adding threads. Those numbers should not be presented as multicore scaling evidence. [Krabmaga](https://krabmaga.github.io/benchmarks/)

**TCE lesson:** borrow architectural ideas and benchmark models; do not assume an ABM framework is automatically the lowest-risk foundation for a production game kernel.

## Unreal Mass, Unity Entities, and Bevy: access safety does not supply model semantics

Unreal Mass stores similarly composed entities in chunks and defers structural changes through command buffers. Unity Entities documents that parallel command recording order depends on scheduling and that independent sort keys can control playback order. Bevy’s **0.19.1** schedule settings explicitly expose ambiguities—conflicting accesses without a specified order—and default ambiguity detection to `Ignore`. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine)

**TCE lesson:** an ECS can prevent incompatible accesses from running concurrently, yet still execute two systems in the wrong *semantic* order. Make phase dependencies explicit and enable ambiguity errors during development.

## Shipped game job systems

Naughty Dog’s GDC 2015 talk, *Parallelizing the Naughty Dog Engine Using Fibers*, describes its job-system work in the context of *The Last of Us Remastered* and a 60-fps target. Cities: Skylines II’s developer explanation identifies multicore pathfinding and simulation as important to its traffic architecture, but provides no transferable 1–24-thread scaling curve. [GDC Vault](https://www.gdcvault.com/play/1022186/Parallelizing-the-Naughty-Dog-Engine)

**TCE lesson:** job systems and parallel pathfinding have shipped successfully. That supports the overall approach—not the conclusion that a solo developer should reproduce a proprietary engine’s scheduler.

---

# 4. Recommended TCE architecture and implementation rules

The following is a design recommendation for TCE, rather than a claim that this exact architecture has already been benchmarked.

## 4.1 Organize the kernel around explicit ownership

Use three layers:

```
tce_core
    Safe Rust simulation, state, rules, validation, serialization
    Runs independently in a headless executable

tce_ffi
    Small C ABI, opaque handles, snapshot ownership, shutdown protocol

Unreal plugin
    Input submission, presentation snapshots, rendering integration
```

Within `tce_core`, I would initially choose **typed, data-oriented storage plus Rayon**, rather than adopting a second complete game engine. A standalone ECS remains reasonable where dynamic composition genuinely simplifies authored behavior, but it should not own an additional uncontrolled worker pool.

For every phase, define:

| Contract | Example |
| --- | --- |
| Read version | Positions, inventories, and prices from epoch 417. |
| Writable objects | This job’s output range, or this resolver’s resource set. |
| Outputs | Agent proposals, reservations, accepted transactions. |
| Visibility boundary | Results become available only after phase completion. |
| Failure policy | Reject intent, fault the tick, or retry under a declared rule. |

This is particularly valuable with AI coding agents: require this contract in changes that introduce shared state or new phase dependencies.

Do not accept `unsafe impl Send/Sync`, raw-pointer aliasing, or a blanket `Arc<Mutex<World>>` merely to make the borrow checker stop complaining.

### Hot and cold state

Keep frequently touched fields compact: position, velocity, current activity, short-term needs, IDs, and small counters. Store genealogies, histories, laws, authored templates, and long-lived relationship data separately.

As an illustrative calculation, 50,000 people × 256 bytes of hot state is **12.8 MB per buffer**, or **25.6 MB for two buffers**. The issue is usually not whether this fits in 64 GB; it is how often it moves through the memory hierarchy.

When reusing the next-state buffer, every published field must be either written or explicitly preserved from the current state. Otherwise, stale values from an older tick can survive in scratch storage.

## 4.2 Use a small, explicit phase sequence

A practical initial sequence is:

**Input boundary → spatial/index preparation → parallel decisions → reservation and conflict resolution → parallel application → structural changes and validation → publication.**

Within that sequence, parallelize independent work. Do not introduce a barrier after every tiny operation, but do not remove a boundary simply because the code can technically execute concurrently.

Use `par_chunks_mut`-style disjoint output ranges and immutable input views. Reuse chunk-local event vectors. Avoid a shared `Mutex<Vec<Intent>>` or per-agent atomic append index in the hot loop.

Births, deaths, membership changes, and storage relocation should occur at declared structural boundaries. Use generational handles or equivalent stale-reference protection. A person crossing a region boundary must not be updated twice—or missed entirely—because two regions disagree about ownership.

### Keep random choices independent of worker scheduling

Use separate random streams keyed by stable simulation identity, or a vetted counter-based generator keyed by world seed, event identity, tick, and draw index. Counter-based generators such as those described by Random123 support obtaining a random value without advancing one shared mutable generator. [Random123](https://random123.com/)

This is not a demand for perfect reproducibility. It prevents adding threads from silently changing *which person receives which random draw*.

For equal-priority resource claims, use explicit model rules and an event-specific randomized tie-break. Do not permanently favor low agent IDs, container iteration order, or whichever task finishes first.

## 4.3 Resolve contention at the model level

### Shared markets and wallets

Use a quoted market snapshot for planning. Collect orders privately, then clear them in batches under a market’s explicit rules.

However, **market-level sharding is insufficient when transactions share wallets or inventory**.

Suppose a person has 10 coins and submits an 8-coin order to each of two markets. Both markets can be individually correct and still approve 16 coins of spending.

Choose one of these designs:

| Design | Appropriate use |
| --- | --- |
| **Pre-reserved budgets and stock** | Assign bounded reservations before independent clearing; their sum cannot exceed available resources. |
| **Single authoritative ledger stage** | A simple starting point when settlement/accounting costs are still small. |
| **Conflict-connected transaction groups** | Parallelize only groups whose accounts and inventories are disjoint. |

For a solo developer, I would start with **batched market processing plus a straightforward authoritative ledger**, then introduce reservations where profiling proves necessary.

Apply a trade as one logical operation: debit, credit, inventory movement, and reservation release. Track transaction IDs so an event cannot be applied twice.

Use integer or explicitly rounded fixed-unit accounting where appropriate. Negative balances should be allowed only through modeled credit rules—not as an accidental transient later hidden by clamping.

### Shared finite resources

Forests, wells, workplaces, construction sites, and housing should receive batches of claims.

The owner checks availability and selects winners. Reducing requests into a total is not enough when demand exceeds capacity; the model also needs an allocation rule.

Prefer allocation by simulated arrival time, queue policy, status, contract, or randomized tie-break as appropriate to the institution. Operating-system scheduling should not become an undocumented social institution.

### Spatial queries and pathfinding

Start with an immutable uniform-grid index: cell offsets plus packed agent IDs are a reasonable initial representation. Build it from one consistent position version.

Use per-chunk temporary results and a merge/build phase instead of concurrent insertion into one general-purpose spatial map. Keep neighborhood query data separate from cold person records.

Dense neighborhoods still require care. A grid cannot remove genuine all-to-all interactions inside one crowded cell. Split overloaded work, improve the query representation, and distinguish “unnecessary candidate checks” from interactions the model truly requires.

For pathfinding, batch requests, share immutable navigation data, cache reusable results, and replan on relevant changes rather than every frame. Version results against the navigation/world state they used.

Admit asynchronous results at declared logical boundaries. A person should not receive a scarce destination merely because their pathfinding task completed first.

## 4.4 Budget threads for the entire Unreal process

Create one long-lived private Rayon pool with an explicit size. Do not build a pool every frame.

Unreal’s Tasks system and TaskGraph share their backend scheduler and workers. Rayon is separate, so unconstrained pools can compete with Unreal’s work. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/tasks-systems-in-unreal-engine)

As **initial configurations to test**, not asserted optima:

* **Interactive mode:** compare 8, 12, and 16 simulation workers first, then higher counts.
* **Headless or fast-forward mode:** include 20, 24, and optionally all 32 logical processors on a matching desktop i9.

Leave capacity for Unreal’s game/render work and other latency-sensitive tasks. A reduced worker count is not failure to exploit the CPU when it produces more useful simulation progress while preserving frame time.

Keep blocking file operations and waits for Unreal callbacks out of Rayon jobs. Avoid holding locks across nested parallel calls.

Only move to Unreal-owned execution if integrated profiling shows that competing schedulers are a material bottleneck. In that design, Unreal launches coarse Rust range jobs; the Rust job must not launch another full nested pool.

### Simulation time is not presentation time

At 60 fps, presentation has approximately **16.67 ms per frame**. That does not imply every economic or institutional system must execute 60 times per second.

A sensible prototype is a fixed **20-Hz movement/interaction tick**, with rendering interpolation, plus explicitly scheduled lower-frequency decisions. The appropriate rate must be validated against movement, congestion, collisions, and behavioral responsiveness.

Needs can advance analytically between events. Governments, technological decisions, construction planning, and market clearing can have their own simulated-time schedules.

Do not use camera visibility to decide whose economic behavior is skipped. Presentation LOD and authoritative simulation approximation are different concerns.

If the simulation falls behind, reduce simulation speed or bound catch-up work. Do not silently skip causal steps or let a wall-clock deadline decide which citizens get updated.

I would reserve the RTX 4070 Ti primarily for rendering initially. Consider GPU simulation later for isolated, regular workloads—not as the first solution to irregular economic and institutional interactions.

## 4.5 Make the DLL boundary deliberately small

Use a C ABI with fixed-width fields, lengths, versioned structures, and opaque handles. Keep Rust-owned `Vec`, `String`, references, and `Arc` values behind that boundary. Define allocation and release ownership explicitly. Rust’s FFI documentation also emphasizes synchronization and lifetime responsibilities for asynchronous callbacks. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

**Simulation double buffering does not make asynchronous rendering safe.**

If Unreal retains a pointer to `S[t]`, the simulation must not recycle that storage while Unreal still reads it.

Publish a separate compact, immutable render snapshot. Suitable initial implementations include a reference-counted snapshot behind a briefly held mutex, or a bounded buffer system with explicit acquire/release ownership. A raw pointer plus an atomic “current index” does not solve reclamation.

Drop obsolete **complete presentation snapshots** when necessary. Do not lose authoritative inputs or essential structural deltas merely because a render queue is full.

Keep one clearly defined coordinator as the mutable owner of the simulation. Two foreign callers creating mutable views of the same raw context can violate Rust’s guarantees even when the Rust implementation looks locally safe.

### Panics and unloading

Treat a worker panic as a failed tick, not permission to continue with partially applied state. With unwinding enabled, contain panics inside Rust before a non-unwinding C boundary; ordinary failures should return explicit error status. `panic=abort` cannot be recovered with `catch_unwind`. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For unloading:

**Stop new work → finish or cancel at safe boundaries → wait for workers → release snapshots/callbacks → unload.**

Rayon’s documentation distinguishes dropping the pool from waiting for worker termination; when termination must be guaranteed, the caller must arrange that wait. Avoid the global pool in an unloadable DLL. Perform shutdown outside `DllMain`, whose loader-lock restrictions make waiting and complex cleanup hazardous. [Docs.rs](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html)

Disable live reloading of an active Rust DLL until this lifecycle is demonstrably safe.

## 4.6 Debug without requiring centuries of deterministic replay

### Validate invariants at phase boundaries

Check more than memory safety. Useful TCE invariants include:

| Area | Checks |
| --- | --- |
| Accounting | Transfers conserve quantities except explicit sources/sinks; reservations respect available balances. |
| Identity | Live handles are valid; transaction IDs apply once; each person updates once per epoch. |
| Space | Each person has one owner; spatial membership matches the published position version. |
| State | No unexpected NaNs; capacities and modeled credit limits hold. |
| Scheduling | Events are not lost, duplicated, or applied before their prerequisites. |

Run cheap checks routinely and expensive whole-world checks in development, stress testing, and sampled validation modes.

Do not “repair” negative stock by clamping it to zero. That destroys the evidence needed to locate the original faulty transaction.

### Maintain a rolling failure recorder

Keep periodic authoritative checkpoints plus a bounded journal of recent inputs and decisions.

A useful failure bundle contains the last valid state, tick and phase, build/ruleset versions, configuration, scheduled events, RNG state or keys, reservations, relevant intents, and accepted/rejected transactions.

On an invariant failure, stop **before publishing** the candidate state and retain the preceding committed snapshot.

Full snapshots alone are not sufficient for exact local replay when important decisions depend on unrecorded randomness or external input. Capture enough of those inputs—or the resolved decision journal—to replay the failing boundary.

This turns “something broke after 180 simulated years” into “these 17 intents produced an invalid ledger transition.”

### Trace causality, not every function call

Use structured tracing fields such as:

```
tick, phase, chunk_id, agent_id, resource_id,
transaction_id, source_epoch, outcome
```

Trace phase/chunk spans routinely; enable detailed entity traces selectively. `tracing` provides structured spans and events, while Unreal Insights supplies host-side timing analysis. Correlate them with shared tick or job identifiers. [Docs.rs](https://docs.rs/tracing/latest/tracing/)

Look for the critical path, maximum chunk duration, waits, queue age, allocation spikes, and snapshot-publication delay—not only aggregate CPU use.

### Use complementary correctness tools

| Tool | Best use | Important limitation |
| --- | --- | --- |
| **ThreadSanitizer** | Executed memory races in instrumented native tests. | Current Rust target support does **not include Windows**; use the headless core on a supported Linux target. |
| **AddressSanitizer** | Out-of-bounds and lifetime-related memory errors. | Not a detector of economic or update-order mistakes. |
| **Miri** | Small tests around unsafe Rust and ownership assumptions. | Most foreign calls are unsupported; not a way to run Unreal. |
| **Loom** | Exploring interleavings in small custom synchronization protocols. | Model the protocol, not the entire simulation or Rayon runtime. |

Rust’s sanitizers remain nightly/toolchain-sensitive; instrument the relevant dependencies and standard library as required. Mixed-language sanitizer runtimes need special care. Linux testing does not replace Windows DLL stress and shutdown testing. [Rust Documentation](https://doc.rust-lang.org/unstable-book/compiler-flags/sanitizer.html)

Finally, test schedule sensitivity directly. Vary thread counts, chunk sizes, partition boundaries, and harmless yields. Compare against a serial implementation of the **same phased semantics**, not an in-place sequential model.

Independent work reordered should preserve discrete accounting invariants. Floating-point calculations may require tolerances; long stochastic runs require distributional comparisons rather than demanding identical histories.

## 4.7 Benchmark TCE from 1 to 24 threads

Build the benchmark harness before writing a custom scheduler.

**Workloads.** Use saved worlds at 10k and 50k people: dispersed farms, a dense capital, scarce-food market contention, mass path replanning, migration/birth/death churn, and an old world with extensive institutions and relationship graphs.

**Thread sweep.** Measure **1, 2, 4, 8, 12, 16, 20, and 24 workers**. Add 32 where applicable. Separately test P-core-only placement, mixed cores, and SMT effects.

**Baselines.** Compare an optimized serial implementation with Rayon at one worker to expose framework overhead. For isolated phase benchmarks, reuse identical snapshots and inputs so diverging histories do not change the work being compared.

**Measurement.** Record whole-tick throughput and p50/p95/p99 latency; per-phase time; longest job; merge/resolution cost; allocation traffic; memory/cache behavior; and validation/publication cost. Warm up first and run repeated sustained trials.

**Integrated test.** Repeat inside packaged Unreal at 1440p with representative camera positions, animation, and rendering settings. Record frame-time percentiles and simulation lag together.

Record CPU SKU, memory configuration, OS build, compiler, dependency lockfile, power settings, and thermal behavior. A sustained desktop run is more informative than a brief turbo-frequency peak.

Choose the configuration that maximizes **simulation progress subject to acceptable frame latency and bounded lag**. Do not choose it by CPU utilization alone.

For implementation order, build the serial phased model and invariants first; parallelize disjoint agent work second; add spatial and resource partitioning third; optimize the remaining measured critical path last.

---

# 5. Sources and version applicability

These are the principal implementation and research references. **Documentation versions below are those inspected for this September 2026 report; historical measurements apply to their original implementations, not automatically to current releases.**

| Source | Version/date and relevance |
| --- | --- |
| [Rayon `ThreadPool`](https://docs.rs/rayon/latest/rayon/struct.ThreadPool.html?utm_source=chatgpt.com) and [pool builder](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html?utm_source=chatgpt.com) | **Rayon 1.12.0**. Work stealing, private pools, nested execution, thread lifecycle. |
| [Rustonomicon: races](https://doc.rust-lang.org/nomicon/races.html?utm_source=chatgpt.com) and [FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Current documentation inspected in 2026. Distinguishes data races from general race conditions; covers foreign-boundary safety. |
| [Rust sanitizer documentation](https://doc.rust-lang.org/unstable-book/compiler-flags/sanitizer.html?utm_source=chatgpt.com) | Nightly/unstable tooling; verify supported targets against the pinned CI toolchain. |
| [Unreal Tasks](https://dev.epicgames.com/documentation/en-us/unreal-engine/tasks-systems-in-unreal-engine?utm_source=chatgpt.com), [Mass overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine?utm_source=chatgpt.com), [Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine?utm_source=chatgpt.com) | **UE 5.8 documentation** inspected. Scheduling, chunked entity storage, deferred changes, host profiling. |
| [Bevy schedule build settings](https://docs.rs/bevy_ecs/latest/bevy_ecs/schedule/struct.ScheduleBuildSettings.html?utm_source=chatgpt.com) | **bevy\_ecs 0.19.1**. Explicit ambiguity detection and deferred-command synchronization. |
| [Unity command-buffer playback](https://docs.unity3d.com/Packages/com.unity.entities@1.0/manual/systems-entity-command-buffer-playback.html) | **Entities 1.0**, intentionally versioned historical reference for parallel recording and ordered playback. |
| [krABMaga paper](https://www.jasss.org/27/2/4.html?utm_source=chatgpt.com), [documentation](https://docs.rs/krabmaga/latest/krabmaga/?utm_source=chatgpt.com), [code](https://github.com/krABMaga/krABMaga?utm_source=chatgpt.com) | **2024 paper; crate 0.6.2** inspected. Double buffering, simulation architecture, feature maturity. |
| [The impact of ECS logic on parallel performance in agent-based model simulations](https://ceur-ws.org/Vol-4124/paper43.pdf?utm_source=chatgpt.com) | **BigHPC 2025**, published December 2025. Desktop scaling experiment. |
| [BioDynaMo PPoPP paper](https://arxiv.org/abs/2301.06984?utm_source=chatgpt.com) and [code](https://github.com/biodynamo/biodynamo?utm_source=chatgpt.com) | **2023**. Spatial indexing, locality, parallel scaling, reproducible research implementation. |
| [Factorio FFF 215](https://www.factorio.com/blog/post/fff-215?utm_source=chatgpt.com) and [FFF 421](https://www.factorio.com/blog/post/fff-421?utm_source=chatgpt.com) | **2017 and 2024** engineering accounts. Cache behavior, read-heavy parallelism, and unsuccessful optimization attempts. |
| [Cities: Skylines II traffic AI](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai?utm_source=chatgpt.com) | **2023 developer explanation**. Multicore traffic/pathfinding precedent, not a scaling benchmark. |
| [Parallelizing the Naughty Dog Engine Using Fibers](https://www.gdcvault.com/play/1022186/Parallelizing-the-Naughty-Dog-Engine?utm_source=chatgpt.com) | **GDC 2015 talk**. Shipped-engine job-system precedent. |
| [Huberman and Glance](https://pmc.ncbi.nlm.nih.gov/articles/PMC47213/?utm_source=chatgpt.com) and [Random123](https://random123.com/?utm_source=chatgpt.com) | **1993 and 2011 foundations**. Temporal update semantics and parallel random-number generation. |
| [Loom](https://github.com/tokio-rs/loom?utm_source=chatgpt.com), [Miri](https://github.com/rust-lang/miri?utm_source=chatgpt.com), [tracing](https://docs.rs/tracing/latest/tracing/?utm_source=chatgpt.com) | Current project documentation; `tracing` **0.1.44** inspected. Protocol testing, unsafe-code checking, structured diagnostics. |
| [Windows CPU Sets](https://learn.microsoft.com/en-us/windows/win32/procthread/cpu-sets?utm_source=chatgpt.com), [DLL best practices](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices?utm_source=chatgpt.com), [i9-13900K specifications](https://www.intel.com/content/www/us/en/products/sku/230496/intel-core-i913900k-processor-36m-cache-up-to-5-80-ghz/specifications.html?utm_source=chatgpt.com) | Platform constraints for placement, unloading, and interpreting core/thread counts. |

## Bottom line

For TCE, **keep simulation meaning independent of execution machinery**. Define what each agent may observe, who can authorize changes to shared resources, and when those changes become visible. Then let Rayon execute the independent work.

That gives you a kernel that can scale across cores without making thread timing part of civilization’s rules—and whose failures can be investigated from a recent state transition rather than reproduced from the beginning of history.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9248d-dd88-83ea-8e10-57ac43a6e849)
