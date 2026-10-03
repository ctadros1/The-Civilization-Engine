# Traffic flow and assignment models for TCE

**Engineering assessment, with sources checked through September 27, 2026**

## Recommendation

Build TCE’s traffic system around a **CPU-based, event-driven mesoscopic model with finite road storage, explicit intersection conflicts, and persistent traveler identities**. Add **platoon/cohort aggregation using the same congestion rules** for fast-forward. Let Unreal render interpolated movement rather than independently simulating every traveler’s physics.

The most useful combination of precedents is **A/B Street for event-driven Rust architecture, MATSim for activity-based demand and queue loading, and UXsim/Newell-style models for aggregation and congestion-wave behavior**. Their implementations differ substantially; this is an architectural recommendation, not a suggestion to combine their codebases wholesale. [A-B Street](https://a-b-street.github.io/docs/tech/trafficsim/discrete_event/index.html)

**Do not use BPR volume-delay functions as authoritative congestion physics. Do not start with city-wide microscopic car-following, GPU traffic simulation, or an equilibrium assignment solver.**

The published evidence supports the feasibility of a 50,000-person traffic system. It does **not** establish that TCE’s entire economy, population simulation, and UE rendering will meet 60 fps on the specified machine. That needs an integrated benchmark.

---

## 1. Establish the workload and the meaning of “real time”

Three quantities must remain separate:

| Quantity | Meaning for TCE |
| --- | --- |
| **Population** | Persistent people, including those sleeping, working, or remaining inside buildings. |
| **Active traffic** | People, animals, carts, and other vehicles currently traversing the transport network. |
| **Simulation acceleration** | Physical simulated seconds advanced per wall-clock second—not rendered frames per second. |

A settlement with 50,000 inhabitants is not necessarily a 50,000-concurrent-traveler workload. Nevertheless, **50,000 simultaneously active travelers should be a deliberate stress test**, particularly for evacuations, festivals, market days, and military movements.

Define acceleration explicitly:

\[
R=\frac{\text{simulated physical seconds}}{\text{wall-clock seconds}}.
\]

By arithmetic, one physical day in ten wall-clock minutes requires \(R=144\); one day per wall-clock second requires \(R=86{,}400\). Neither follows automatically from achieving 60 rendered frames per second.

For TCE, I would establish two separate acceptance criteria: **interactive frame performance**, including traffic visualization, and **simulation throughput at a specified \(R\)**. Otherwise, a smooth camera can conceal a simulation that is falling behind.

---

## 2. Model options and their trade-offs

The complexity assessments below are engineering judgments for a solo developer building a custom Rust kernel, not published benchmark results.

| Model | How it works | Congestion fidelity | Cost and implementation burden | Fit for TCE |
| --- | --- | --- | --- | --- |
| **Static volume-delay, including BPR** | Assign a travel time to a link from its traffic-to-capacity ratio. | Average delay; no intrinsic queue position, spillback, or queue history. | Cheap cost evaluation, but assignment can still require many shortest-path passes. Low implementation burden. | Strategic estimates and initial route costs—not live traffic. |
| **Point/vertical queues** | Travelers incur free-flow travel time, then wait for an exit server. Queue storage is unlimited. | Bottleneck delay and throughput, but not physical blocking of upstream streets. | Very cheap, especially event-driven. Low burden. | Useful prototype or non-spatial service queue. Insufficient as the final road model. |
| **Finite-storage spatial queues** | Each link or movement group has limited occupancy and discharge capacity. A full downstream link blocks entry. | Queue buildup, spillback, junction blocking, and gridlock. Wave propagation requires additional care. | Low-to-medium cost; intersection rules dominate complexity. | **Best starting point.** |
| **Newell-derived platoon models / link transmission models** | Enforce kinematic-wave constraints using vehicle/platoon trajectories or cumulative link flows. | Finite-speed congestion waves and spillback without detailed acceleration. | Medium burden; node allocation and boundary histories matter. | **Best direction for the production model and fast-forward.** |
| **Macroscopic cell transmission model, CTM** | Divide roads into cells; update density using sending and receiving flows. | Physically meaningful queue formation, propagation, and dissipation. | Cost scales with cells, time steps, and tracked destination classes. Medium burden. | Good aggregate model; less naturally identity-preserving. |
| **Microscopic car-following and lane-changing** | Update each vehicle’s acceleration, gap, speed, and lane decisions. | Detailed interactions and trajectories, subject to model calibration. | Frequent updates plus difficult junction and lane-changing edge cases. High burden. | Local validation or a later specialized subsystem—not the default world model. |

BPR, spatial queues, CTM, Newell-derived models, and microscopic following are established approaches. Importantly, **“agent-based” does not necessarily mean microscopic traffic dynamics**: individual people can retain activities and routes while their movement is governed by mesoscopic link queues. [Aequilibrae](https://www.aequilibrae.com/latest/python/traffic_assignment/volume_delay_functions.html)

### 2.1 Why BPR is insufficient

The usual form is

\[
t\_e=t\_e^0\left[1+\alpha\left(\frac{v\_e}{c\_e}\right)^\beta\right],
\]

where \(t\_e^0\) is free-flow travel time, \(v\_e\) is assigned traffic volume or flow, and \(c\_e\) is capacity in matching units. The familiar \(\alpha=0.15,\ \beta=4\) values are conventions requiring calibration, not universal physical constants. [Aequilibrae](https://www.aequilibrae.com/latest/python/traffic_assignment/volume_delay_functions.html)

With those values, the calculated travel time is \(1.15t\_e^0\) at \(v/c=1\), and \(3.4t\_e^0\) at \(v/c=2\).

The problem is not merely imperfect coefficients. BPR does not itself determine **where an excess queue occupies space**, whether it blocks a neighboring junction, or how much traffic actually exits. Its volume input is also not the number of travelers currently occupying the road. [Aequilibrae](https://www.aequilibrae.com/latest/python/traffic_assignment/volume_delay_functions.html)

**TCE use:** initialize expected route costs, estimate accessibility, or evaluate coarse infrastructure proposals. Do not add BPR congestion delay on top of an already congested queue model unless the terms represent different effects; otherwise congestion is counted twice.

### 2.2 The important distinction within queue models

A finite-storage queue can reproduce spillback, but **finite storage alone does not guarantee realistic backward-moving congestion waves**. A simplistic implementation can make space freed at a downstream exit instantly available at the upstream entrance.

More complete models constrain both downstream vehicle movement and upstream propagation of available space. SUMO’s mesoscopic implementation documents headway-based jam dynamics; its newer LTM option further changes how jammed space propagates. [Eclipse SUMO](https://sumo.dlr.de/docs/Simulation/Meso.html)

For TCE, first implement correct storage, discharge, and intersections. Then add a consistent Newell/LTM-style receiving rule where queue-wave timing matters. Avoid “fixing” this by adding an arbitrary extra delay to every exit, including uncongested traffic.

### 2.3 Macroscopic models and fast-forward

A simple kinematic-wave fundamental diagram is

\[
q(\rho)=\min\left(v\_f\rho,\ Q,\ w(\rho\_j-\rho)\right),
\]

where \(v\_f\) is free-flow speed, \(Q\) is capacity, \(\rho\_j\) is jam density, and \(w\) is the magnitude of backward-wave speed.

CTM updates each cell by conservation:

\[
n\_i(t+\Delta t)=n\_i(t)+\Delta t\left(f\_{i-1,i}-f\_{i,i+1}\right).
\]

For a simple connection, transmitted flow is constrained by upstream sending and downstream receiving. Junctions need an additional allocation rule. This family can reproduce the growth and dissipation of queues without simulating individual acceleration. [Institute of Transportation Studies](https://its.berkeley.edu/publications/cell-transmission-model-dynamic-representation-highway-traffic-consistent-hydrodynamic?utm_source=chatgpt.com)

However, explicit cell updates have a stability constraint approximately of the form

\[
\Delta t\leq\frac{\Delta x}{\max(v\_f,w)}.
\]

**Increasing the time step arbitrarily is not a valid fast-forward strategy.** Link-based formulations avoid many internal cells, while platoon models reduce the number of moving objects. Neither eliminates the need to preserve flow conservation and destination information. [Academia](https://www.academia.edu/24815545/The_cell_transmission_model_A_dynamic_representation_of_highway_traffic_consistent_with_the_hydrodynamic_theory?utm_source=chatgpt.com)

### 2.4 When microscopic simulation is justified

Models such as IDM calculate acceleration from speed, spacing, and relative speed; MOBIL adds lane-change incentive and safety criteria. These address questions that link queues deliberately omit. [Traffic Simulation](https://traffic-simulation.de/info/info_IDM.html)

For TCE, detailed following becomes worthwhile when the gameplay depends on overtaking maneuvers, collision avoidance, or particular local interactions—not simply because travelers are visible. Animated movement can look continuous while authoritative transfers remain event-driven.

Microscopic cellular automata offer another relatively inexpensive option by quantizing space and movement. They can produce jams, but their discrete geometry is a less convenient default for irregular streets, mixed-width carts, and pedestrians. The latter is an engineering suitability judgment; the ability of cellular models to generate bottleneck congestion is well established. [arXiv](https://arxiv.org/abs/cond-mat/9406089?utm_source=chatgpt.com)

---

## 3. Dynamic traffic assignment is a separate layer

**Traffic flow answers “what happens on the chosen routes?” Assignment answers “which routes and departure times do travelers choose?”**

Dynamic traffic assignment combines time-dependent route choice with network loading. Equilibrium approaches repeatedly adjust choices until travelers cannot materially improve their experienced costs by switching routes. They are not equivalent to sending everybody along the currently shortest path. [FHWA Operations](https://ops.fhwa.dot.gov/publications/fhwahop13015/sec2.htm)

MATSim’s repeated simulation/replanning framework is valuable for planning studies. TCE instead needs a world that advances continuously. I recommend **online, bounded adaptation**, not repeated equilibrium solves:

**Before departure:** choose among a small set of plausible routes using remembered travel times, access permissions, tolls, slope, safety, and traveler preferences.

**During travel:** reconsider after a closure, an unusually long delay, or newly acquired information—not every frame.

**Between trips:** update remembered costs gradually. Stagger updates and require a meaningful improvement before switching.

For genuinely time-dependent routing, costs should reflect when the traveler is expected to reach each link, rather than assuming that the entire route is traversed under conditions at departure. FHWA also notes the information assumptions embedded in instantaneous routing. [FHWA Operations](https://ops.fhwa.dot.gov/publications/fhwahop13015/sec2.htm)

For an early agrarian TCE society, this has an additional implication: **do not silently give every citizen a perfect live congestion map**. Familiar routes, incomplete information, and experience should remain meaningful.

Start with A\*/Dijkstra, reusable destination trees where appropriate, and versioned route caches. Batch requests. Add advanced routing acceleration only after profiling: graph mutation, heterogeneous preferences, and time-dependent costs complicate aggressive preprocessing.

---

## 4. What published performance numbers actually demonstrate

These results are **not an apples-to-apples ranking**. They differ in hardware, network size, active-agent count, simulation fidelity, routing workload, and timing scope.

| System and version | Published workload and hardware | Reported performance | Qualification for TCE |
| --- | --- | --- | --- |
| **CityFlow, WWW 2019 implementation** | A 30×30 intersection grid with tens of thousands of running vehicles; Xeon E5-2686 v4 at 2.30 GHz; eight threads. | Approximately **72 simulation steps per wall-clock second**, about **25×** the contemporary SUMO comparison. | Headless traffic throughput, not 72 rendered fps. Historical SUMO comparison; cannot be projected onto current SUMO or UE. [arXiv](https://arxiv.org/pdf/1905.05217) |
| **Rust MATSim prototype v0.2.0, 2025 paper** | **491,175 synthetic people**, 1,193,056 links; 36 simulated hours. Xeon Platinum 9242 cluster. | **560 simulated seconds/wall second with one process**; **24,284 with 1,024 processes across 43 nodes**. | Population is not simultaneous traffic. Cars/bicycles used the network; walking/ride legs were teleported, and public-transit users were excluded. The cluster result is not a desktop benchmark. [ResearchGate](https://www.researchgate.net/publication/388814961_High-Performance_Mobility_Simulation_Implementation_of_a_Parallel_Distributed_Message-Passing_Algorithm_for_MATSim) |
| **MATSim Hermes, October 2020** | Roughly one million agents and 1.5 million links; 48-core Xeon Platinum 8168 system. Hermes used 48 threads; QSim 18. | Reported average iteration: **3 min 33 s versus 8 min 45 s**. Total run time fell **40%**, with replanning becoming dominant. | Unequal thread counts; reported memory use was **130 GB**, above TCE’s machine. This demonstrates the value of event-driven movement, not direct suitability of that configuration. [MATSim](https://matsim.org/news/2020/introducing-hermes/) |
| **UXsim, December 2023 paper** | Sioux Falls: **34,690 vehicles over 7,200 simulated seconds**, 76 links; five-vehicle platoons, 5 s steps, route updates every 600 s. Windows 10, 3.79 GHz CPU, 32 GB RAM. | Approximately **16 wall seconds**—a calculated acceleration of **450×**. | CPU model unspecified; vehicles are total demand, not simultaneous occupancy. Small network, aggregated movement, infrequent routing updates. [arXiv](https://arxiv.org/pdf/2309.17114) |
| **SUMO mesoscopic mode, rolling documentation** | Scenario-dependent queue simulation. | Project documentation reports **up to 100× faster** than its microscopic model. | An upper-end project claim, not a universal speedup or controlled benchmark for TCE. [Eclipse SUMO](https://sumo.dlr.de/docs/Simulation/Meso.html) |

### Interpretation

The strongest conclusion is that **50,000 persistent people do not inherently require an expensive traffic core**. Published systems handle substantially larger demand populations or tens of thousands of active vehicles.

The weaker—and unjustified—conclusion would be that TCE therefore has guaranteed capacity for 50,000 visible, fully animated, continuously rerouting agents at 60 fps.

For your benchmark, measure **movement, routing, scheduling, event delivery, snapshot construction, and rendering separately**. Also record worst-case latency, not only average throughput: a major road closure can create a routing burst even when ordinary movement is cheap.

---

## 5. Shipped-game precedents and what they teach

### SimCity — specifically the 2013 GlassBox game

Andrew Willmott’s GDC 2012 *Inside GlassBox* talk is the primary architectural reference. The game’s developer material describes an agent-based simulation rather than a pure aggregate traffic layer. [Andrew Willmott](https://www.andrewwillmott.com/talks/inside-glassbox)

Its problems are more useful than an unsupported claim that it used a particular academic traffic model. Maxis’s August 2013 Update 7 notes explicitly addressed vehicles converging on the same destination, acceleration/deceleration around crossings and building exits, and freight trucks departing without valid delivery destinations. [Wayback Machine](https://web.archive.org/web/20130827223617/http%3A//forum.ea.com/eaforum/posts/list/0/9674690.page)

In its October 2013 retrospective, Maxis also reported abandoning larger city sizes after experiments involving routing and processing because of performance constraints. [Sims Community](https://simscommunity.info/2013/10/04/blog-post-state-of-simcity/)

**TCE lesson:** demand generation, dispatch, destination choice, and delivery validity are part of traffic correctness. A good movement solver cannot repair inconsistent logistics.

### Cities: Skylines 1

The official comparison with CS2 distinguishes two decisions: CS1 selected destinations or services using straight-line proximity, then followed a fastest route that generally remained fixed despite congestion unless network changes invalidated it. This does **not** mean vehicles traveled in straight lines. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai)

TM:PE’s documentation describes the vanilla stuck-vehicle despawning behavior and warns that disabling it increases simulation load. Its documentation also describes passengers/goods being effectively teleported when vehicles despawn. This is mod-author documentation of game behavior, not a disclosed engine specification. [TMPE Docs](https://doc.tmpe.me/toggle-despawn.html)

**TCE lesson:** never resolve a jam by silently converting a blocked delivery into a completed delivery. Cancellation, abandonment, spoilage, or emergency intervention can be legitimate outcomes—but must have explicit economic consequences.

### Cities: Skylines 2

The June 2023 developer diary describes generalized route costs incorporating time, comfort, money, and behavior; more responsive lane use; and multicore pathfinding. Its “no hard agent limit” claim is hardware-qualified, not a performance guarantee. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai)

Version matters. **Patch 1.5.9f1, May 27, 2026**, reduced unnecessary U-turns and made vehicles select turn lanes earlier. **The September 3, 2026 developer Q&A still identifies pathfinding as a major CPU challenge**, alongside GPU/rendering optimization needs. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/patch-notes-morning-dew)

**TCE lesson:** sophisticated preferences and multicore execution do not remove the difficult parts: route-request volume, advance lane selection, access geometry, and interactions with the rest of the economy.

### Transport Fever — documented here using Transport Fever 2

TF2’s simulation documentation represents residents and cargo individually and describes recurring home/work/shopping travel. It explicitly warns that nearby towns with many possible connections can impair performance through route-choice complexity. [Transport Fever 2 Wiki](https://wiki.transportfever2.com/doku.php?id=gamemanual%3Asimulationoverview)

Its 2024 Final Update-era changes included preferring routes with fewer intersections, improving distribution across lanes, and overtaking slow vehicles on multilane roads. These are concrete examples of continued work on local movement and route quality. [Transport Fever 2 Wiki](https://wiki.transportfever2.com/doku.php?id=releasenotes)

A particularly important distinction: the documentation says the default calendar advances **one day per two real seconds while vehicles move at realistic physical speeds**. That calendar rate is not evidence of simulating 24 hours of traffic every two seconds. [Transport Fever 2 Wiki](https://wiki.transportfever2.com/doku.php?id=gamemanual%3Asimulationoverview)

**TCE lesson:** decide whether the calendar represents physical elapsed time or a presentation/economic abstraction. Do not accidentally mix the two.

### What is not publicly established

I did not find reproducible, isolated traffic-core benchmarks for these shipped games that establish “50,000 active travelers cost X milliseconds” on hardware comparable to yours. Their observed game performance also includes rendering, services, economy, user interface, and other systems.

Their strongest value here is **architectural experience and documented failure modes**, not numerical capacity guarantees.

---

## 6. Open-source precedents worth studying

### A/B Street: the closest implementation reference

A/B Street’s Rust traffic simulator uses discrete events and explicitly discusses the trade-off between detailed acceleration and the more consequential contention around intersections and parking. Its documentation explains queue ordering, vehicle tails remaining on upstream lanes, and the complexity caused by short roads and long vehicles. [A-B Street](https://a-b-street.github.io/docs/tech/trafficsim/discrete_event/index.html)

It also exposes an important limitation for TCE: its documented pedestrian model largely permits pedestrians to pass through one another. Copying that behavior would miss congestion at markets, narrow gates, and shared passages. [A-B Street](https://a-b-street.github.io/docs/tech/trafficsim/discrete_event/index.html)

Study its event scheduling and occupancy bookkeeping, rather than adopting all its behavioral assumptions. The repository is Apache-2.0 licensed; the detailed simulator article is from 2021, while the project’s later work has broadened toward planning tools. [GitHub](https://github.com/a-b-street/abstreet)

### MATSim and Hermes: demand architecture and efficient loading

MATSim is valuable for separating people’s activity plans from network movement. Hermes demonstrates why processing relevant events instead of continually traversing the whole network can help. Its 2020 implementation, however, explicitly lacked some features—including within-day replanning and traffic signals—that TCE may need. [MATSim](https://matsim.org/news/2020/introducing-hermes/)

The current Rust MATSim repository is useful research material, but it is **not identical to the benchmarked 2025 prototype** and carries a **GPL-3.0 license**. Treat code reuse as a licensing review item, not an automatic DLL dependency. [GitHub](https://github.com/matsim-vsp/parallel_qsim_rust)

### UXsim and SUMO: aggregation reference and validation tool

UXsim is especially useful for studying compact, Newell-derived mesoscopic modeling and platoon aggregation; its repository is MIT licensed. [GitHub](https://github.com/toruseo/UXsim)

Use SUMO primarily as an external comparison tool for small, controlled scenarios. Match junction-control assumptions between micro and meso runs. Also pin versions: **SUMO 1.27.1 was released June 25, 2026**, while `--meso-ltm` appears in the subsequently retrieved **Git Main/nightly** changelog, not that stable release. [Eclipse SUMO](https://sumo.dlr.de/docs/ChangeLog.html)

---

## 7. Recommended TCE implementation

### 7.1 Authoritative state: people remain individuals; movement need not

Keep person, household, possessions, cargo ownership, destination, and trip purpose in persistent simulation state.

Represent transport separately:

| Component | Suggested responsibilities |
| --- | --- |
| **Transport graph** | Links, usable width, slope, surface, access rules, building entrances, crossings, and topology version. |
| **Movement group** | Occupancy, physical storage, discharge state, queue order, and compatible turns. |
| **Junction** | Conflict sets, priorities, reservations, and downstream receiving constraints. |
| **Trip** | Persistent traveler/cargo references, route, mode, departure time, and progress. |
| **Scheduler** | Next relevant arrivals, service opportunities, wake-ups, and topology-change events. |
| **Renderer snapshot** | Read-only movement descriptions for visible representatives. |

For a traveler entering link \(e\), the earliest uncongested exit is

\[
t\_{\mathrm{earliest}}=t\_{\mathrm{entry}}+\frac{L\_e}{v\_{\mathrm{free},e,m}}.
\]

An actual transfer additionally requires available discharge capacity, junction permission, and downstream receiving space.

Keep **storage** and **flow capacity** separate. A long road can hold many travelers while discharging slowly through a narrow gate. A short road may have high potential throughput yet quickly block an upstream intersection.

### 7.2 Spend fidelity on bottlenecks

For TCE’s initial technology level, I would prioritize mixed walking/cart corridors, gates, bridges, loading areas, market entrances, and narrow opposing passages—not modern freeway lane-changing.

Use mode-specific footprints and service behavior. A pedestrian and a laden cart should not differ only in maximum speed. Walking traffic may pass within available width; a single FIFO queue for an entire broad street would incorrectly force everyone behind the slowest cart.

Conversely, separate turn queues must not allow travelers to pass through a physically blocked shared approach. Movement groups should reflect actual passing opportunities.

For intersections, represent a small conflict matrix and reserve movements for finite durations. Keep a long cart’s tail occupying the approach until it has cleared. Define whether travelers can block the junction while waiting for downstream space; make that a behavioral rule, not an accidental consequence of update order.

### 7.3 Event-driven execution without event storms

Schedule the next possible exit rather than updating every traveler’s position continuously. A blocked queue should wake when the condition blocking it changes, not repeatedly poll every occupant.

Recommended safeguards:

* **No unlimited accumulation of unused discharge capacity.** Otherwise an empty or blocked road can later release an impossible burst.
* **Stable event ordering and bounded retries.** Saturated junctions must not generate endless immediate retry events.
* **Conservative transfers.** Moving a traveler releases and consumes the correct occupancy exactly once.

Use fixed or integer event times, stable tie-breaking, seeded random streams, and deterministic iteration order. Begin with a single-threaded reference implementation; parallelize only after its behavior is testable.

### 7.4 Route computation gets its own budget

Cache reusable route structures, not just individual paths. Invalidate them by network version when bridges disappear, streets are closed, or access laws change.

Separate ordinary trip requests from urgent requests. Distribute routine departures and replanning work across simulation time rather than causing every household to request a route at the same instant.

Do not synchronously reroute all travelers after a topology change. First identify affected routes; retain unaffected ones; process urgent broken routes before discretionary improvements.

---

## 8. Aggregate fast-forward without destroying the economy

I recommend three execution levels.

| Mode | What is simulated | What is deliberately approximated |
| --- | --- | --- |
| **Individual mesoscopic mode** | Every active traveler participates in link queues and junction decisions. | Continuous acceleration and detailed lateral motion. |
| **Cohort/platoon mode** | Compatible travelers share movement processing while retaining membership and cargo records. | Fine departure ordering and within-cohort spacing, within controlled tolerances. |
| **Strategic long-horizon mode** | Representative travel patterns and coarse transport constraints feed demographic/economic updates. | Exact realization of every daily commute. |

### Cohort mode should be the first fast-forward mechanism

Group travelers by compatible mode, route segment or next movement, and departure-time bucket. Split groups before routes diverge or constraints differ.

Preserve **count, membership, destination composition, queue position, travel age, and cargo**. An aggregate occupancy number alone is insufficient: it cannot tell the economy which worker arrived or which merchant’s goods remain trapped.

Test cohort sizes such as 1, 5, and 20 as engineering experiments—not historically justified constants. UXsim demonstrates that platoon size can be a useful computational control, but TCE must measure the error introduced into its own travel times and deliveries. [arXiv](https://arxiv.org/pdf/2309.17114)

Do not switch to independent BPR delays during fast-forward and then reconstruct an uncongested world. That would erase the causal consequences of the queues.

### Century-scale acceleration needs a separate contract

Even an excellent movement solver cannot cheaply execute arbitrarily many individual daily decisions. For very large jumps, use representative-day transport loading or another explicitly coarse transport/economy coupling.

Refresh it when relevant conditions change: infrastructure, settlement distribution, production, access laws, disasters, or demand patterns. Periodically compare against detailed sample days.

This mode necessarily sacrifices some exact daily history. **Preserving every person does not require preserving every footstep—but preserving the exact outcome of every trip is a much stronger requirement.**

Finally, keep authoritative fidelity independent of camera position. Looking away from a bridge must not increase its economic capacity.

---

## 9. Rust DLL and Unreal Engine 5.8 integration

Use Rust as the sole authority for transport state. Unreal should consume snapshots and present movement.

At the boundary, use an opaque simulation handle and a small C-compatible API with explicit ownership. Exchange plain data and stable identifiers; do not pass Rust `Vec`, `String`, references, or unwinding panics across the boundary. These recommendations follow the Rust FFI ownership and ABI constraints. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

A practical arrangement is a Rust simulation worker, bounded command queues, and double- or triple-buffered immutable snapshots. Unreal’s game thread performs the required engine-object updates. Avoid one FFI call per traveler per frame.

Unreal’s Mass representation system supports different visual representations and LODs. UE **5.8 was released June 23, 2026** and introduced additional experimental crowd-related workflows, including MetaHuman Collections. These are rendering/integration options, not evidence that 50,000 expensive characters will fit the target GPU. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-gameplay-in-unreal-engine)

My hardware-specific recommendation is to reserve the RTX 4070 Ti primarily for rendering. Start with CPU traffic, a small number of simulation workers, and measured thread allocation rather than competing with Unreal for every available core.

Visual interpolation must obey authoritative queue and junction constraints. A render-only traveler must not glide through a closed gate merely because its previous snapshot predicted forward movement.

### Initial performance targets—not measured results

| Metric | Proposed starting acceptance target |
| --- | --- |
| Interactive presentation | 60 fps, corresponding to a 16.67 ms frame interval. |
| Traffic snapshot ingestion | At most approximately **0.5–1 ms on the game thread** at the chosen update cadence. |
| Movement plus amortized routing | Initially target **2–4 ms CPU work per 100 ms of physical simulation** at \(R=1\), off the game thread. |
| Snapshot cadence | Start around **10 Hz**, with interpolation; increase only where visible error requires it. |
| Testing scale | Normal demand plus **50,000 simultaneous travelers** and a larger overload case. |

These are proposed budgets, not predictions. In particular, the simulation-work target must be multiplied by acceleration unless event skipping or aggregation reduces the work.

As a sizing example, 50,000 records at 32 bytes each are 1.6 MB per snapshot, or 16 MB/s at 10 Hz. That arithmetic suggests snapshot bandwidth need not be the dominant problem; engine-object updates, animation, routing bursts, and synchronization still need measurement.

---

## 10. Validation and implementation order

For a solo developer using AI coding agents, the safest sequence is **a small deterministic reference kernel, then congestion correctness, then aggregation, then optimization**.

First construct headless scenarios: a single bottleneck, two merging streams, a shared versus dedicated turn lane, a short link containing a long cart, opposing flow through a narrow passage, and a gridlocked loop. Add topology changes and trip cancellation before integrating elaborate visuals.

Measure both computation and simulated outcomes:

| Performance metrics | Behavioral and conservation metrics |
| --- | --- |
| Movement/routing/snapshot p50, p95, p99 times | Queue lengths and maximum spillback |
| Events processed per second | Bottleneck discharge and junction throughput |
| Simulated seconds per wall second | Travel-time distribution, not just the mean |
| Route requests and cache hit rates | Completed, delayed, aborted, and unreachable trips |
| Memory and allocations | No lost people, duplicated cargo, negative occupancy, or unexplained deliveries |

Compare small scenarios against analytical expectations and appropriately configured SUMO micro/meso runs. Then compare individual and cohort TCE modes using the same inputs. Passing conservation tests does not guarantee matching travel times, so both are necessary.

Assign coding agents bounded modules with invariant tests: queue accounting, intersection arbitration, route invalidation, serialization, and cohort splitting. Avoid asking them to produce a full multithreaded multimodal simulator before a correct single-threaded reference exists.

**The first milestone should be a headless Rust network where 50,000 travelers can congest a bridge, spill back through neighboring streets, reroute imperfectly, and eventually arrive—or fail to arrive—with every person and item accounted for.** Unreal presentation comes after that causal chain works.

---

## 11. Source guide and version applicability

| Area | Primary documentation, papers, talks, and code |
| --- | --- |
| **BPR and assignment** | [AequilibraE volume-delay functions](https://www.aequilibrae.com/latest/python/traffic_assignment/volume_delay_functions.html?utm_source=chatgpt.com), retrieved as 1.7.0 documentation; [FHWA DTA fundamentals](https://ops.fhwa.dot.gov/publications/fhwahop13015/sec2.htm?utm_source=chatgpt.com), 2013. |
| **Microscopic models** | Authors’ explanations of [IDM](https://traffic-simulation.de/info/info_IDM.html?utm_source=chatgpt.com) and [MOBIL](https://traffic-simulation.de/info/info_MOBIL.html). |
| **Macroscopic foundation** | Daganzo, 1994: [*The cell transmission model*](https://its.berkeley.edu/publications/cell-transmission-model-dynamic-representation-highway-traffic-consistent-hydrodynamic). |
| **Rust event-driven simulation** | [A/B Street technical article](https://a-b-street.github.io/docs/tech/trafficsim/discrete_event/index.html?utm_source=chatgpt.com), 2021; [source repository](https://github.com/a-b-street/abstreet?utm_source=chatgpt.com). |
| **MATSim performance** | [Rust prototype paper](https://doi.org/10.3390/info16020116), 2025, benchmarked v0.2.0; [current Rust repository](https://github.com/matsim-vsp/matsim-rust?utm_source=chatgpt.com); [Hermes production report](https://matsim.org/news/2020/introducing-hermes/), 2020. |
| **Platoon aggregation** | [UXsim paper](https://arxiv.org/abs/2309.17114), December 2023 version used for benchmark numbers; [2025 JOSS publication](https://joss.theoj.org/papers/10.21105/joss.07617); [source](https://github.com/toruseo/UXsim?utm_source=chatgpt.com). |
| **Microscopic benchmark** | [CityFlow WWW 2019 paper](https://arxiv.org/abs/1905.05217?utm_source=chatgpt.com); [source](https://github.com/cityflow-project/CityFlow?utm_source=chatgpt.com). |
| **Micro/meso comparison tool** | [SUMO mesoscopic documentation](https://sumo.dlr.de/docs/Simulation/Meso.html?utm_source=chatgpt.com); [changelog](https://sumo.dlr.de/docs/ChangeLog.html?utm_source=chatgpt.com). Distinguish stable 1.27.1 from later nightly features. |
| **SimCity** | Willmott’s [*Inside GlassBox*, GDC 2012](https://www.andrewwillmott.com/talks/inside-glassbox?utm_source=chatgpt.com); [archived Maxis Update 7 notes](https://web.archive.org/web/20130827223617/http://forum.ea.com/eaforum/posts/list/0/9674690.page), 2013. |
| **Cities: Skylines** | [Official CS1/CS2 traffic comparison](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai?utm_source=chatgpt.com), 2023; [TM:PE despawn documentation](https://doc.tmpe.me/toggle-despawn.html?utm_source=chatgpt.com); [CS2 1.5.9f1 notes](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/patch-notes-morning-dew?utm_source=chatgpt.com), May 2026; [developer performance Q&A](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/collected-questions), September 2026. |
| **Transport Fever 2** | [Simulation overview](https://wiki.transportfever2.com/doku.php?id=gamemanual:simulationoverview&utm_source=chatgpt.com), September 2024 documentation; [release notes](https://wiki.transportfever2.com/doku.php?id=releasenotes), including Final Update-era traffic changes. |
| **Engine boundary and rendering** | [Rustonomicon FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com); [Mass Gameplay overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-gameplay-in-unreal-engine); [UE 5.8 release announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available). |

**Bottom line:** TCE needs persistent travelers and physically consequential bottlenecks, not 50,000 independent steering simulations. A finite-storage, event-driven mesoscopic core with careful intersections, bounded routing, and identity-preserving aggregation offers the strongest balance of realism, speed, and implementation scope.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92939-456c-83ea-a240-ed7b6ebf665d)
