# Routing 50,000 commuters in The Civilization Engine

**Engineering report — evidence reviewed through September 27, 2026**

## Executive recommendation

For TCE, I would build **a CPU routing service around cached habitual routes, A*/ALT, and a small hierarchical road graph*\*, with **Customizable Contraction Hierarchies (CCH) as the first advanced accelerator to evaluate**—not the initial implementation.

The most important optimization is architectural: **50,000 citizens should not imply 50,000 fresh shortest-path searches every simulation step.** Separate destination choice, route choice, and movement; reuse routes until something materially changes; share computations between commuters; and process requests asynchronously against immutable routing snapshots.

Among advanced algorithms, CCH is attractive for frequent **traffic-cost changes**, while a partitioned, portal-based hierarchy is easier to adapt to frequent **new roads and settlements**. Ordinary contraction hierarchies and hub labels are less attractive as TCE’s primary live router because their preprocessing is more closely tied to the current metric. Customizable alternatives specifically address that limitation. [arXiv](https://arxiv.org/abs/1402.0402)

The recommendation below distinguishes published measurements from proposed TCE design choices. **I have not benchmarked TCE’s graph or this architecture on your machine.**

---

## 1. Size the workload before choosing the algorithm

Population is only an indirect predictor of routing cost. The quantities that matter are:

\[
\text{routing work}
\approx
Q\,C\_q
+
F\_m\,C\_m
+
F\_t\,C\_t
+
C\_{\text{validation/reconstruction}}
\]

Here, \(Q\) is fresh queries per real second; \(C\_q\) is query cost; \(F\_m,C\_m\) describe metric updates; and \(F\_t,C\_t\) describe topology updates.

For illustration, assume 50,000 people make four trip legs per simulated day:

| Real time per simulated day | Trip starts per real second, averaged over the day |
| --- | --- |
| 20 minutes | 167 |
| 60 seconds | 3,333 |
| 10 seconds | 20,000 |

These are calculated examples, not assumptions about TCE’s eventual clock. With 90% reuse, fresh route searches would be one-tenth of those figures—but destination searches, invalidations, and exceptional trips still add work.

At an illustrative **100 μs per fresh search**, 3,333 searches consume approximately **0.33 CPU-core seconds per second**. Conversely, 50,000 simultaneous requests consume five core-seconds and create a substantial queue even when average demand is modest.

**Therefore, optimize rush-hour bursts and repeated work before optimizing the last microsecond of an individual query.**

Also distinguish three operations:

**Destination selection:** Which market, workplace, port, or supplier should this person use? A naive implementation can issue many route queries per decision.

**Route planning:** Which sequence of roads and transfers connects the chosen endpoints?

**Traffic progression:** When can this person enter the next road, pass an intersection, or board a vessel?

For TCE, I recommend separate interfaces and instrumentation for all three. A faster shortest-path algorithm cannot compensate for repeatedly evaluating hundreds of unnecessary destination candidates.

---

## 2. Main routing options

### A. Dijkstra, A\*, bidirectional search, and ALT

Dijkstra provides an uncomplicated exact baseline for nonnegative edge costs. A\* prioritizes vertices using an estimate of remaining cost. For travel time, straight-line distance divided by a valid maximum speed is a useful lower bound; a weak estimate makes A\* behave more like Dijkstra.

**ALT—A*, Landmarks, and Triangle inequalities—strengthens that estimate using precomputed distances to and from selected landmarks.*\* It supports directed road graphs and combines with bidirectional search. Unlike CH, it does not replace the road graph with metric-dependent shortcuts. [Microsoft](https://www.microsoft.com/en-us/research/publication/computing-the-shortest-path-a-search-meets-graph-theory/)

**TCE fit:** excellent initial production router. Roads can be inserted or removed without rebuilding a shortcut hierarchy. The main preprocessing obligation is keeping the heuristic valid.

An important implementation rule follows from admissibility: build landmark tables using an **optimistic lower-bound metric**, such as free-flow travel times. Increasing congestion costs preserves those lower bounds. Adding a faster connection, increasing maximum travel speed, or changing mode permissions can invalidate them. In those cases, rebuild the affected tables or temporarily use a weaker valid heuristic.

Bidirectional search needs a correct termination condition; “stop at the first meeting” is not generally sufficient.

Two additional variants deserve consideration:

**Weighted A*/ARA*.\*\* Increasing the heuristic’s influence trades exact optimality for bounded suboptimality under the algorithm’s assumptions. ARA\* can improve an initial solution while reusing search work. This is potentially useful for discretionary travel, but I would first exploit route reuse rather than introduce approximation everywhere. [CMU School of Computer Science](https://www.cs.cmu.edu/~maxim/files/ara_nips03.pdf)

**D* Lite/LPA*.\*\* These reuse previous search work after changes. They are particularly useful for sequences of related replanning problems, not automatically for 50,000 unrelated origin–destination pairs. Maintaining a large incremental search state for every citizen would be a poor default. [IDM Lab](https://idm-lab.org/bib/abstracts/papers/aaai02b.pdf)

### B. Ordinary Contraction Hierarchies: excellent queries, awkward live updates

CH contracts vertices in an importance order, adding shortcuts where necessary to preserve shortest paths. Queries search a restricted upward hierarchy; shortcut unpacking recovers the original road sequence.

The complication is that ordinary CH construction uses current weights to decide which shortcuts are necessary. **Changing an edge weight is not safely handled by merely changing that edge and adding up existing shortcut costs.** The Rust `fast_paths` implementation requires graph preparation again; reusing a node ordering can accelerate preparation, but it remains preparation rather than CCH-style customization. [CuriousCoding](https://curiouscoding.nl/posts/cch/)

**TCE fit:** static regional networks, infrequently changing free-flow routing, comparison benchmarks, or tools operating on frozen worlds. I would not use ordinary CH as the sole congestion-aware router.

### C. Customizable Contraction Hierarchies: separate connectivity from costs

CCH separates work into three phases:

1. **Topology preprocessing:** choose a metric-independent vertex order and construct the required hierarchy.
2. **Customization:** propagate current edge costs through that hierarchy.
3. **Query and unpacking:** compute a shortest route and reconstruct its original edges.

This is a strong match when the same topology receives many traffic-cost updates. Partial customization can propagate only affected changes, although its cost is highly uneven: changing a minor road may affect little; changing a major bottleneck may affect many shortcuts. [arXiv](https://arxiv.org/html/1402.0402v5)

**The limitation:** customizable does not mean arbitrarily dynamic connectivity.

A closure can often be represented as infinite cost on an existing arc. A genuinely new road or vertex requires structural work unless the relevant connection was already represented. An old ordering might remain usable, but the required shortcut structure still needs updating or rebuilding.

**TCE fit:** the leading advanced accelerator once query volume justifies it and topology edits can be batched. Keep A\* available for unsupported requests and while rebuilding.

### D. Customizable Route Planning / multilevel overlays

CRP partitions the graph into cells, then builds overlays representing shortest connections between cell boundaries. Large searches traverse those overlays rather than every interior road. Partitioning is largely metric-independent; customization computes the current boundary-to-boundary costs. The original work explicitly supports turn costs and multiple metrics. [Microsoft](https://www.microsoft.com/en-us/research/publication/customizable-route-planning/)

**TCE fit:** particularly attractive when settlements grow locally. A TCE-specific implementation can rebuild an edited cell’s internal connections and propagate changes to its ancestors.

This is not an unconditional constant-time update: a new boundary connection, poor partition, or strategic bridge can affect higher levels. Exactness also requires retaining the necessary boundary connections and computing their costs correctly.

I would favor a **small, understandable portal overlay** over implementing the full production CRP literature immediately.

### E. Hub labels: extremely fast distance queries, expensive indexing

Hub labeling stores, for each vertex, distances to selected hubs such that a shortest path can be recovered through a common hub. A query intersects the origin’s and destination’s labels rather than running a conventional graph search. The original practical implementation achieved submicrosecond distance queries on continental networks, at substantial memory and preprocessing cost. [Microsoft](https://www.microsoft.com/en-us/research/wp-content/uploads/2010/12/HL-TR.pdf)

Classical labels are strongly metric-dependent. **Customizable Hub Labeling exists**, so “labels cannot support updates” would be incorrect; however, this is a more specialized research direction than the readily usable Rust baseline options. [arXiv](https://arxiv.org/abs/2208.08709)

**TCE fit:** not my first choice. You need evolving topology, mode restrictions, several preferences, and actual paths—not merely the fastest possible static distance lookup.

### F. HPA\*: game-oriented hierarchy

HPA\* divides a map into clusters, precomputes crossings between selected entrances, searches the abstract graph, and refines the result. Its original experiments reported **up to 10× faster searches than optimized A**\* with paths **within 1% of optimal after smoothing** on the tested maps. That is an empirical result, not a universal guarantee. [WebDocs](https://webdocs.cs.ualberta.ca/~mmueller/ps/2004/hpastar.pdf)

For TCE, borrow the hierarchy rather than automatically adopting a fine grid. Roads already form a sparse graph. Walking across fields, through courtyards, and between building entrances may benefit from a separate terrain hierarchy.

**Important distinction:** sampled HPA\* entrances can sacrifice optimality; an overlay retaining all necessary boundary crossings can remain exact.

### G. Reverse trees and flow fields: share a destination’s work

A reverse shortest-path search can serve many origins going to one destination. A flow field extends the idea spatially: agents follow locally stored directions toward a goal or intermediate portal.

*Supreme Commander 2* used tiled flow fields, portal-level paths, cached field reuse, and separate steering. Tiles could be reused when units shared intermediate portals even when their final destinations differed. [GameAIPRO](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf)

**TCE fit:** markets, ports, settlement gates, pilgrimage sites, evacuation destinations, and shared route segments. They are less attractive when nearly every agent has a distinct destination: the number of fields then becomes the problem.

Do not confuse a flow field with congestion simulation or collision avoidance. It supplies guidance; another system must handle capacity and local movement.

---

## 3. Published performance: what the numbers actually establish

These results demonstrate feasibility, **not a controlled ranking across libraries**. The machines, networks, metrics, implementation dates, and inclusion of path reconstruction differ.

| Technique and source | Workload / environment | Published result | Interpretation for TCE |
| --- | --- | --- | --- |
| **Rust CCH, 2025 survey** | Stuttgart: approximately 110k vertices, 252k edges, production turn data; dual Xeon E5-2670, Linux, Rust 1.64 nightly | Perfect-customized query: **14.5 μs distance + 5.4 μs unpacking ≈ 19.9 μs** | Strong city-scale evidence, including actual path recovery. [arXiv](https://arxiv.org/pdf/2502.10519) |
| **Same CCH implementation** | Same Stuttgart graph | Topology preprocessing **0.9 s**; full customization **49.13 ms on one thread**, **11.67 ms on eight** | Updates and queries are very different budgets. Sixteen threads did not improve this small instance. [arXiv](https://arxiv.org/pdf/2502.10519) |
| **`fast_paths` CH** | New York: 264,347 vertices, 730,100 edges; M1 Max, one core, Rust 1.74.1; 100k random queries | Travel-time metric: **6 s preparation, 26 μs reported average query** | Fast queries do not imply cheap live metric changes. [GitHub](https://github.com/easbar/fast_paths) |
| **Original CRP, 2011** | Europe: about 18M vertices, 42M arcs; i7-920, Windows Server 2008 R2 | Four-level variant with 1-second U-turn cost: **5.8 s customization, 1.18 ms distance query** | Historical evidence for trading query speed for much cheaper customization; not a benchmark of current optimized CRP. [Microsoft](https://www.microsoft.com/en-us/research/wp-content/uploads/2011/05/crp-sea.pdf) |
| **Original hub-label implementation, 2010 report** | Western Europe; dual Xeon X5680 machine, Windows | One label-ordering variant: **769 ns distance query**, **30.6 GB label space** | The exceptionally fast lookup is purchased with a large index; this is not full-path timing. [Microsoft](https://www.microsoft.com/en-us/research/wp-content/uploads/2010/12/HL-TR.pdf) |

A useful decision rule is:

\[
Q(C\_{\text{A\*}}-C\_{\text{accelerated}})
>
F\_mC\_m+F\_tC\_t+\text{additional maintenance}
\]

Measure this with TCE workloads. If route reuse reduces fresh queries sufficiently, the sophisticated index may save less work than maintaining it costs.

Also distinguish distance-only experiments from usable routing implementations. A March 2026 CCH optimization write-up explicitly noted that its implementation did **not yet store path-recovery metadata**, making its comparison with a path-capable baseline incomplete. It also found correctness bugs only after adding Dijkstra comparisons. Both are valuable cautions for AI-assisted implementation. [CuriousCoding](https://curiouscoding.nl/posts/cch/)

---

## 4. Congestion-aware routing without constant replanning

### Start with stable estimates, not instantaneous perfect knowledge

For the first implementation, I recommend a static cost snapshot representing **expected** travel conditions:

\[
c\_e
=
\text{expected traversal time}
+
\text{turn penalty}
+
\text{profile-specific generalized penalties}
\]

Maintain actual elapsed travel time separately from generalized cost. A preference against danger or a monetary charge affects route choice, but it must not literally advance the simulation clock. SUMO makes the same distinction between travel time and optimization “effort,” especially when costs depend on arrival time. [Eclipse SUMO](https://sumo.dlr.de/docs/Simulation/Routing.html)

For TCE, estimate road delays from recent observations and the previous comparable period. Smooth them; avoid letting every small queue fluctuation cause a network-wide rebuild.

**Proposed route-choice policy:** retain a habitual route unless it becomes invalid, conditions deteriorate substantially, or an alternative offers a meaningful improvement. A 10–20% improvement threshold is a reasonable parameter to experiment with—not a literature-backed optimum for your world.

Refresh only a fraction of discretionary choices at once, and use persistent agent-specific preferences or seeded alternative selection. Otherwise, identical agents can all abandon one road for another, creating oscillating congestion.

### Time-dependent routing is more than departure-time buckets

In genuine time-dependent routing, an edge costs \(c\_e(t)\), where \(t\) is when the traveler reaches that edge. Choosing all edge costs from the trip’s departure bucket is only an approximation.

For standard earliest-arrival label-setting methods, the important condition is **FIFO**: leaving an edge later must not allow arrival earlier than someone who entered it first. Equivalently, \(t+c\_e(t)\) must be nondecreasing. Abrupt downward jumps between cost buckets can violate that property; use appropriately constructed arrival functions rather than arbitrary step changes.

Exact advanced implementations exist: **CATCHUp** addresses time-dependent road routing, and its reproducible ESA 2020 implementation is tied to `rust_road_router`. I would defer that complexity until TCE demonstrates that along-route prediction materially changes outcomes. [GitHub](https://raw.githubusercontent.com/kit-algo/catchup/master/README.md)

A practical progression is:

**First:** smoothed current/typical costs.

**Next:** morning, midday, evening, and exceptional-condition profiles.

**Later:** actual time-dependent traversal functions where journeys are long enough, and congestion predictable enough, to justify them.

### Route choice does not create believable traffic by itself

For TCE, the movement model should separately account for road capacity, finite storage, queues, intersection throughput, and—where relevant—spillback into upstream roads.

Do not turn every nearby moving person into a global pathfinding obstacle. Keep local passing, spacing, and intersection admission below the route planner. A/B Street demonstrates a discrete-event approach in which agents advance between meaningful lane/turn states rather than requiring universal fixed-frequency movement updates. [A-B Street](https://a-b-street.github.io/docs/tech/trafficsim/discrete_event/index.html)

At high time acceleration, preserve the route-demand and capacity model even when visible motion becomes statistical. Removing congestion from the fast mode would change which settlements, employers, and markets are economically accessible.

### Cache routes by more than an origin–destination pair

I recommend three layers:

| Cache | Contents | Refresh policy |
| --- | --- | --- |
| **Personal habitual routes** | Home–work, home–market, recurring social destinations | Reuse until invalid or materially unattractive |
| **Shared zone/portal corridors** | Several representative alternatives between origin and destination areas | Refresh by affected region, profile, and time bucket |
| **Popular-destination trees** | Reverse trees toward markets, ports, gates, major workplaces | Recompute when sufficient demand justifies it |

A shared cache key should account for **direction, mode, access permissions, cost profile, departure bucket, and relevant graph versions**. Origin/destination zones alone are insufficient: two buildings on opposite sides of a river may share a nominal zone but require different exits.

Cache a corridor or portal sequence, then calculate local access and egress. Do not force every commuter through zone centroids.

Keep two validity concepts separate:

**Feasibility:** does the route still exist and remain legal? Closures, destroyed bridges, changed permissions, or removed ferry services require hard invalidation.

**Quality:** is the route still reasonably attractive? Congestion changes usually justify gradual refresh. A newly built bridge can improve a route even when none of its existing edges changed, so dependency tracking alone cannot preserve optimality.

For failed searches, cache “unreachable” cautiously and invalidate it on reconnection. Use bounded, demand-driven caches rather than materializing every zone pair across every mode and time bucket.

Finally, destination search should exploit shared work: perform a bounded one-to-many search for suitable markets rather than running a separate full path query for every shop.

---

## 5. Multimodal routing: model the traveler’s state

Walking, carts, boats, and transit are not merely different edge-speed multipliers.

For TCE, use a routing state conceptually equivalent to:

\[
(\text{location},\ \text{mode},\ \text{vehicle/access state})
\]

A walking traveler can board a boat only at an appropriate location, with an available service or vessel. A cart cannot materialize at a remote destination. Loading, unloading, waiting, fares, ownership, capacity, and the return journey belong in the trip model.

I recommend **separate physical networks with explicit transfer links**:

**Walking:** roads, paths, building access, and limited off-road navigation. An early agrarian world must remain traversable before its road network is complete.

**Carts:** width, gradient, surface, bridge capacity, and legal-access restrictions.

**Boats:** directed river costs, navigability, landing places, draft constraints, loading and boarding.

**Transit:** passenger journeys across stops and schedules, distinct from the routing of the transit vehicles themselves.

A/B Street’s explicit trip-leg representation—walking, operating a vehicle, and riding transit—is a useful open-source precedent for making these transitions visible and testable. [A-B Street](https://a-b-street.github.io/docs/tech/trafficsim/trips.html)

For scheduled transit, **RAPTOR** is a strong later candidate. It scans routes in rounds corresponding to transfer counts and produces arrival-time/transfer tradeoffs without conventional road-graph preprocessing. Treat boarding capacity and missed departures as simulation events beyond the timetable query itself. [Microsoft](https://www.microsoft.com/en-us/research/publication/round-based-public-transit-routing/)

For preferences, begin with a small number of representative profiles, then let individuals choose among candidate routes. A separate fully customized network for every citizen’s unique utility function would undermine sharing. This is an approximation to individualized optimization, so measure its behavioral consequences.

---

## 6. What shipped games and open projects teach

### Cities: Skylines 1: destination choice and routing are different problems

The developer’s comparison states that CS1 selected destinations or services using straight-line proximity, then followed the fastest route and generally retained it through congestion unless network changes invalidated it.

The distinction matters: this does **not** mean vehicles drove in straight lines. It means a nearby fire station could be selected even when another had a shorter actual road journey. For TCE, route-aware accessibility must influence service, employment, and market selection—not merely movement afterward. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai)

### Cities: Skylines 2: costs and local behavior matter as much as search

The 2023 developer description specifies route costs based on **time, comfort, money, and behavior**, with traveler preferences and parking considerations. It also distinguishes route decisions from reactions to nearby traffic.

I did not find sufficient primary documentation to identify the stock games’ exact core search implementations as CH, CCH, or HPA\*. Assigning one of those algorithms to them would overstate the evidence. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai)

Version-specific examples are revealing:

**Patch 1.5.2f1, December 4, 2025:** fixed unnecessary pathfinding calculations that caused performance problems in some saves. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/asset-mods-patch-notes)

**Morning Dew, 1.5.9f1, May 2026:** increased U-turn costs and adjusted lane selection so vehicles merged earlier. The accompanying explanation acknowledged that legal U-turns previously had zero additional cost and that late lane changes created jams. These were modeling and execution problems, not simply a lack of a faster shortest-path algorithm. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/morning-dew)

### A/B Street: the closest Rust precedent

A/B Street rebuilds CHs for affected transportation modes after map edits and reuses previous node ordering through `fast_paths`. Its documentation identifies rebuilding those hierarchies as the slowest edit-processing step and batches the work until the user exits edit mode. [A-B Street](https://a-b-street.github.io/docs/tech/map/edits.html)

**Lesson for TCE:** read this code, but do not assume its edit workflow transfers unchanged to an autonomously growing world. TCE needs background rebuilds, batched construction changes, and a fallback while an accelerator is stale.

### SUMO: particularly relevant current engineering practice

SUMO’s routing documentation, edited **September 23, 2026**, describes CCH with partial recustomization, metrics keyed by vehicle type, thresholds for propagating cost changes, and stable vehicle assignment to multiple randomized metrics for route diversity. Unsupported per-request prohibitions fall back to embedded A\*. The page explicitly warns that feature support differs between its applications. [Eclipse SUMO](https://sumo.dlr.de/docs/Routing_Algorithms.html)

**Lesson:** an advanced hierarchy plus explicit fallbacks and limited metric families is a practical architecture—not an admission of failure. Pin a SUMO release before assuming the rolling documentation’s features are present in that release.

### OSRM and Supreme Commander 2

OSRM supports both CH and Multi-Level Dijkstra; its current README recommends MLD by default, except for special cases such as very large distance matrices. Its separate partition/customize workflow is a useful production reference. [GitHub](https://github.com/Project-OSRM/osrm-backend)

*Supreme Commander 2* provides the complementary game lesson: reuse local guidance through shared portals, and keep steering separate from global route construction. Its documented system is a precedent, not a published guarantee for 50,000 independently commuting citizens. [GameAIPRO](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf)

---

## 7. Rust tooling and the Unreal boundary

### Libraries worth evaluating

| Project / version reviewed | Role | Assessment for TCE |
| --- | --- | --- |
| [`pathfinding` 4.16.0](https://docs.rs/pathfinding/4.16.0/pathfinding/) | Generic A\*, Dijkstra, bidirectional search, Yen, and other algorithms | Good prototype and correctness reference. Accepts successor functions rather than requiring one graph representation. MSRV is Rust 1.88.0. [Docs.rs](https://docs.rs/pathfinding/latest/pathfinding/) |
| [`petgraph` 0.8.3](https://docs.rs/petgraph/0.8.3/petgraph/) | Graph structures, algorithms, diagnostics | Useful for graph construction, testing, and visualization. Compare `StableGraph` and CSR-style storage with your own packed runtime representation. [Docs.rs](https://docs.rs/petgraph/latest/petgraph/) |
| [`fast_paths` 1.0.0](https://docs.rs/fast_paths/1.0.0/fast_paths/?utm_source=chatgpt.com) | Ordinary CH | Convenient static-router benchmark. Use one reusable `PathCalculator` per worker. It is not CCH. [Docs.rs](https://docs.rs/fast_paths/1.0.0/fast_paths/) |
| [`rust_road_router`](https://github.com/kit-algo/rust_road_router?utm_source=chatgpt.com) — research repository | CCH, time-dependent algorithms, CH potentials, routing experiments | Strong algorithmic reference, but requires dependency, API, portability, and correctness evaluation rather than treating it as a turnkey game middleware crate. [GitHub](https://raw.githubusercontent.com/kit-algo/rust_road_router/master/engine/README.md) |
| [`cchpp`](https://github.com/kit-algo/cchpp?utm_source=chatgpt.com) — 2025 survey artifact | Reproduction of the CCH measurements above | Start here when reproducing the published benchmark; do not assume another implementation reproduces its numbers. [arXiv](https://arxiv.org/pdf/2502.10519) |

A `fast_paths` graph also requires care around modeling: its documented limitations include positive nonzero weights and collapsing duplicate directed edges to the lowest weight. Distinct parallel services or zero-cost transfers therefore need an appropriate state-expanded representation. [GitHub](https://github.com/easbar/fast_paths)

### Recommended Rust runtime layout

Keep authoritative routing data entirely inside Rust. I would use packed adjacency arrays, separate metric arrays, stable external road IDs, and per-worker reusable search scratch space.

Publish **graph, metric, and accelerator versions as a compatible bundle**. Do not query new weights through stale shortcuts.

Build the next metric/index snapshot in the background. When new topology becomes authoritative before its accelerator is ready, route against the new base graph with A\*, or use validated cached routes. Never let an old index authorize travel over a destroyed bridge.

Do not expose internal CSR indices as persistent citizen route identifiers without versioning or generation checks. A graph rebuild can renumber them.

Memory should be manageable with deliberate bounds. For example, 50,000 routes averaging 100 four-byte edge IDs require **20 MB of raw path storage**, before metadata. Four separate habitual paths each would make that 80 MB. Sixteen directed landmark pairs over 100,000 vertices require **12.8 MB** using four-byte distances. These are arithmetic examples; shortcut fill-in, geometry, queues, and allocator overhead are additional.

### Inside UE 5.8

Use a bounded Rust worker pool and batched requests/results. Do not assign an independent Unreal pathfinding controller to every commuter or make render-frame completion depend on the routing queue.

Start by measuring a few worker counts—such as two, four, and eight—while Unreal is rendering a representative city. Reserve headroom rather than automatically consuming every logical CPU. I would keep routing on the CPU initially and leave the GPU budget to rendering; GPU routing is a separate experiment, not a prerequisite.

Across the DLL boundary, use opaque handles and C-compatible data layouts, explicit ownership, and Rust-side release functions for Rust allocations. Prevent unwinding across an incompatible FFI boundary. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Epic’s **UE 5.8** documentation describes explicit DLL loading through `FPlatformProcess::GetDllHandle` and packaged-build staging through `RuntimeDependencies`. Verify the packaged Windows build early, including DLL shutdown after all routing workers have stopped. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

---

## 8. Recommended implementation sequence and acceptance tests

### Stage 1: ship the understandable system

Build exact Dijkstra as an oracle and A\* as the initial runtime router. Add habitual-route reuse, bounded request queues, immutable snapshots, mode restrictions, explicit transfers, and meaningful route-failure handling.

Stagger routine decisions over simulation time. Coalesce duplicate requests. Apply results at simulation boundaries rather than letting worker completion order unpredictably decide economic outcomes.

Keep agents on valid current plans while optional requests wait. When essential work falls behind, reduce simulation acceleration instead of silently teleporting people or dropping trips.

### Stage 2: exploit shared structure

Add ALT after measuring heuristic effectiveness. Introduce shared portal corridors and reverse trees for popular destinations.

Where whole-network searches remain costly, add a **two-level road overlay** with automatically derived cells and exact boundary connections. Keep local road edits localized when possible. Avoid simultaneously implementing a full CRP system, CCH, time-dependent shortcuts, and sophisticated flow fields.

This is my preferred solo-developer path: every step should remain testable against the same base graph.

### Stage 3: promote CCH only when it wins the whole workload

Evaluate CCH when fresh query volume remains high, many metric updates occur between topology changes, and maintaining a small set of cost profiles is sufficient.

Benchmark **query savings minus customization and rebuilding**, not query latency alone. Retain A\* permanently for unsupported profiles, stale indexes, unusual restrictions, and correctness checks.

### What to measure

Use generated TCE-like networks—not only random endpoints on modern highway maps. Include organic paths, regular street grids, rivers, bridges, disconnected villages, bottlenecks, and progressively more complex turn restrictions.

Measure **p50/p95/p99 request latency including queueing and path reconstruction**, CPU time per simulated day, metric and topology rebuild times, peak memory, cache hit rates, and route-quality loss from approximate reuse.

The most important regression scenarios are a mass departure wave, a bridge closure during travel, a newly opened shortcut, reconnection of an isolated district, changed access laws, a faster transport technology, and a cancelled ferry.

For every exact query, compare cost and legality against Dijkstra on the **same snapshot**. For cached or approximate routes, measure excess cost and behavioral effects rather than labeling every deviation a bug. Distinguish “no route exists” from “the search was deferred or exhausted its budget.”

**Bottom line:** for TCE, the winning combination is likely **fewer searches, shared routes, a compact road graph, stable congestion estimates, and asynchronous execution**. CCH is a credible accelerator when needed; it should strengthen that architecture, not substitute for it.

---

## 9. Source guide and version scope

The inline citations support the detailed claims. These are the most useful starting points for implementation:

**Road-routing foundations:** [Customizable Contraction Hierarchies](https://arxiv.org/abs/1402.0402?utm_source=chatgpt.com), 2014 preprint, revised 2015; [CCH survey and Rust experiments](https://arxiv.org/abs/2502.10519?utm_source=chatgpt.com), 2025 v1; [Customizable Route Planning](https://www.microsoft.com/en-us/research/publication/customizable-route-planning/?utm_source=chatgpt.com), SEA 2011; [ALT technical report](https://www.microsoft.com/en-us/research/publication/computing-the-shortest-path-a-search-meets-graph-theory/?utm_source=chatgpt.com), 2004.

**Specialized algorithms:** [Hub-label technical report](https://www.microsoft.com/en-us/research/publication/a-hub-based-labeling-algorithm-for-shortest-paths-on-road-networks/?utm_source=chatgpt.com), 2010; [Customizable Hub Labeling](https://arxiv.org/abs/2208.08709?utm_source=chatgpt.com), 2022; [CATCHUp reproducibility repository](https://github.com/kit-algo/catchup?utm_source=chatgpt.com), ESA 2020; [RAPTOR](https://www.microsoft.com/en-us/research/publication/round-based-public-transit-routing/?utm_source=chatgpt.com), ALENEX 2012.

**Game navigation:** [Original HPA\* paper](https://webdocs.cs.ualberta.ca/~mmueller/ps/2004/hpastar.pdf?utm_source=chatgpt.com), 2004; [Crowd Pathfinding and Steering Using Flow Field Tiles](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf?utm_source=chatgpt.com), *Game AI Pro*, 2013, describing *Supreme Commander 2*; [A/B Street technical documentation](https://a-b-street.github.io/docs/toc.html?utm_source=chatgpt.com) and [author presentation at State of the Map 2021](https://media.ccc.de/v/sotm2021-9963-a-b-street-using-osm-for-transportation-advocacy?utm_source=chatgpt.com).

**Current implementation references:** [SUMO routing feature matrix](https://sumo.dlr.de/docs/Routing_Algorithms.html?utm_source=chatgpt.com), rolling documentation edited September 23, 2026; [OSRM backend](https://github.com/Project-OSRM/osrm-backend?utm_source=chatgpt.com), repository documentation reviewed in September 2026; [Epic third-party library documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com), UE 5.8.

For experiments, pin crate versions and repository commits. Historical benchmark compiler versions are recorded above; they are not recommendations to use those old toolchains.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927bb-5710-83ea-8887-3bbca0fa7392)
