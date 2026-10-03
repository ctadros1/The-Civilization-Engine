# Runtime PCG driven by external simulation data

**Engineering report for The Civilization Engine — UE 5.8, Windows; assessed September 27, 2026**

## Executive recommendation

**Use PCG as a deterministic, disposable presentation layer over Rust-owned simulation state—not as the simulation’s database.** Rust should decide where farmland exists, which trees are resources, what fences have been built, and how crops develop. PCG should turn those decisions into meshes, variation, and nearby detail.

For TCE, the best fit is a **hybrid architecture**:

* **Native C++ data bridge and CPU PCG** for bounded, event-driven dressing from Rust snapshots.
* **`GenerateAtRuntime` with hierarchical generation** for replaceable detail around the camera.
* **GPU PCG as an optional optimization** for dense, noninteractive vegetation—not the initial foundation for fences, resource trees, or anything requiring reliable collision and navigation.

This distinction matters in UE 5.8. The core PCG framework became **Production-Ready in 5.7**, but the current documentation still labels **GPU processing Beta**, and its GPU procedural-instancing path and FastGeo integration remain **Experimental**. Core framework maturity does not imply identical maturity or functionality across all spawning paths. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-7-is-now-available)

There is credible production precedent for runtime procedural dressing, including LEGO Fortnite and Horizon Zero Dawn. However, **I found no reproducible public benchmark matching UE 5.8, a Rust simulation, TCE’s update workload, and an RTX 4070 Ti**. The budgets below are proposed engineering targets, not measured performance claims.

---

## 1. Options: how to drive PCG at runtime

### 1.1 Separate three decisions

There are three independent choices:

| Decision | Alternatives | Recommended starting point |
| --- | --- | --- |
| **Data transport** | Native point/spatial data; actor properties; textures; imported assets | Native C++ bridge over immutable Rust snapshots |
| **Generation lifetime** | On load; explicit on demand; proximity-driven runtime generation | On-demand bounded tiles, plus proximity-driven detail |
| **Output representation** | Ordinary instanced meshes; actors; GPU procedural instances | Ordinary instances initially; GPU instances only for suitable decoration |

A texture does not require GPU spawning, and a CPU graph can run under the runtime-generation scheduler. Conversely, calling a graph during gameplay does not require selecting `GenerateAtRuntime`: `GenerateOnDemand` is specifically available when the application manages generation itself. Epic’s explanation distinguishes the trigger from how the graph executes. [Epic Developer Community Forums](https://forums.unrealengine.com/t/pcg-high-memory-usage-of-runtime-pcg-metadata-assets/2551052)

### 1.2 Native point and spatial data: the strongest baseline

Implement a small runtime plugin containing a custom PCG source node, conceptually:

```
TCE tile snapshot
    → bounded candidate points / fence paths / exclusion shapes
    → land-use and suitability filtering
    → explicit density thinning
    → species or kit selection
    → transforms and instance custom data
    → Static Mesh Spawner
```

The source node should retrieve **one immutable snapshot for the requested region**, rather than calling the DLL separately for each point.

The relevant extension interfaces are `UPCGSettings` and `IPCGElement`. Current point-data APIs include `UPCGBasePointData` and `UPCGPointArrayData`; the latter uses the newer point-array representation. Do not build a new 5.8 integration around assumptions copied from early tutorials about directly modifying the old point container. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/IPCGElement)

For TCE, expose only rendering-relevant information:

| Input | Suitable representation |
| --- | --- |
| Land use, fertility, moisture, biomass | Bounded raster/grid snapshot |
| Resource trees or other individually meaningful objects | Points with stable simulation IDs |
| Field boundaries, fences, hedges | Polylines/splines or explicit segment transforms |
| Buildings, roads, cleared ground | Exclusion polygons, footprints, or masks |
| Species and architectural style choices | Small recipe/palette tables |
| Crop growth and local seasonal response | A few scalar attributes |

**Advantages:** no unnecessary GPU round trip; explicit ownership; easy deterministic testing; direct access to the simulation’s spatial index.

**Costs:** custom C++, thread/lifetime management, and responsibility for cache invalidation. This is nevertheless a much smaller integration problem than exposing the entire Rust entity system through Unreal objects.

### 1.3 Actor properties and graph parameters: good for prototypes and small control inputs

A tile-provider actor can expose parameters or PCG data, which graphs retrieve using actor-data/property nodes. This is useful for debugging, designer overrides, and a first end-to-end prototype.

`UPCGDataFromActorSettings` supports property-driven data retrieval and offers `bAlwaysRequeryActors`, which prevents caching that element and rereads the selected actors when executed. **It does not itself constitute an external-data change notification system.** [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGDataFromActorSettings)

Recommended uses include a biome ID, crop type, density multiplier, snapshot revision, or reference to a tile’s data object. Avoid one actor per raster sample, crop plant, or simulation agent. Likewise, avoid repeatedly searching every actor in the world to discover a small tile’s inputs.

Keep overrides on the relevant component’s graph instance rather than treating one shared graph asset as mutable per-tile state. The component API exposes graph-instance access and local generation operations. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGComponent?lang=en-US)

### 1.4 Textures: valuable for raster fields, especially GPU decoration

Textures are a natural representation for fertility, moisture, snow coverage, disturbance, and vegetation suitability. A native source can construct `UPCGTextureData` around a runtime texture; alternatively, graphs can use texture-retrieval and sampling nodes.

The 5.8 texture API explicitly distinguishes initialization, CPU readback, and an **editor-only CPU-visible duplicate**. The node reference also documents texture-to-world mapping and point versus bilinear filtering. A texture workflow that succeeds through an editor-only duplicate is not sufficient evidence that the same path works in a cooked game. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGTextureData)

For TCE, use these rules:

**Keep CPU data on the CPU when the graph is CPU-based.** Encoding an existing Rust grid as a GPU texture only to read it back adds an avoidable transport stage.

**Use tiled textures for GPU graphs.** Update changed regions or replace a tile’s texture generation; avoid uploading a world-sized atlas after each local change.

**Treat categorical and continuous channels differently.** My recommendation is nearest/point sampling for land-use and species IDs, but interpolated sampling for continuous fertility or moisture. Do not interpolate a “forest” ID with a “wheat field” ID and interpret the result as a valid category.

**Make coordinates explicit.** Each texture needs a documented tile origin, extent, resolution, axis convention, and revision. Keep simulation coordinates separate from camera-relative rendering coordinates.

**Keep resources alive until consumers finish.** `UPCGTextureData` stores its texture as a weak object reference; the bridge therefore needs an appropriate owning reference and a safe replacement policy. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGTextureData)

### 1.5 GPU PCG: dense decoration with important restrictions

GPU PCG is attractive when there are enough points to amortize setup costs. Connected GPU nodes form compute graphs; repeated CPU/GPU transitions undermine the benefit. A good candidate is “sample a few masks, generate grass or crop clumps, select meshes, and spawn,” with the dense data remaining GPU-resident. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-with-gpu-processing-in-unreal-engine?lang=en-US)

However, the documented GPU Static Mesh Spawner uses procedural instances that lack **collision/physics, navigation, ray tracing participation, and distance-field lighting contribution**. Its instances are runtime-only, without saved instance information, baked lighting, or HLOD support. This is not feature-equivalent to ordinary CPU-managed instances. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-with-gpu-processing-in-unreal-engine?lang=en-US)

Consequently:

| Good initial GPU candidates | Keep on another path initially |
| --- | --- |
| Grass, flowers, stubble, small stones | Fence collision and gates |
| Decorative crop clumps | Individually harvested resource objects |
| Noninteractive understory | Trees whose geometry must participate fully in the chosen lighting/collision setup |
| Cosmetic density variation | Anything whose disappearance changes simulation truth |

GPU decoration can still follow authoritative crop and land-use state. It simply must not become the only representation of that state.

### 1.6 Imported assets and data tables: recipes, not a live simulation bus

PCG supports data tables, PCG data assets, and Alembic point imports. These are useful for authored palettes, reusable assemblies, and precomputed distributions. Epic specifically advises converting Alembic into PCG assets rather than repeatedly importing it inside an iterating graph. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/procedural-content-generation-framework-node-reference-in-unreal-engine)

For TCE, reserve this route for authored building blocks. Do not serialize the live kernel to CSV or Alembic whenever a field changes.

---

## 2. Runtime generation, performance, and trade-offs

### 2.1 What `GenerateAtRuntime` actually does

`GenerateAtRuntime` delegates proximity-driven generation and cleanup to PCG’s scheduler. It searches for eligible execution sources near active generation sources, prioritizes work, and creates partition actors as needed. Epic explicitly says that the runtime scheduler should exclusively manage the execution sources and partition actors it creates. **Do not simultaneously manage those same local objects through a second custom lifecycle.** [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/FPCGRuntimeGenScheduler)

Generation sources can include player controllers, World Partition sources, or a dedicated PCG generation-source component. Per-grid generation radii determine when cells are scheduled; a cleanup multiplier provides a larger removal boundary. The default scheduling policy considers distance and viewing direction. Runtime generation is supported in editor, PIE, and standalone builds. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-generation-modes-in-unreal-engine?lang=en-US)

**It is not a continuously reevaluated simulation graph.** A fertility change inside Rust is not automatically visible to Unreal’s tracking systems. TCE must publish the changed data and explicitly invalidate or refresh the relevant generation work.

### 2.2 Partitioning and hierarchical generation

Partitioning bounds the work and output spatially. Hierarchical generation allows different parts of a graph to execute at different grid sizes. PCG’s `Grid Size` node controls downstream execution scale; the Biome documentation describes separate levels for locally generated detail and broader data. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/procedural-content-generation-framework-node-reference-in-unreal-engine)

For TCE, a sensible **initial experiment**, not a universal optimum, is:

| Layer | Candidate cell width | Initial generation reach | Policy |
| --- | --- | --- | --- |
| Fine ground detail | 32 m / 3,200 cm | 96–128 m | Camera-proximity generation |
| Medium decorative vegetation and field detail | 128 m / 12,800 cm | 384–512 m | Proximity or explicit tile generation |
| Coarse visual vegetation structure | 256–512 m | Determined by visible distance | Persistent tile representation or coarse generation |
| Authoritative trees, fences, gates | Simulation-defined | Gameplay/render residency | Independent of disposable detail cleanup |

Use the smallest practical number of levels. More levels also create more dependencies and lifecycle states to diagnose.

**Keep spatial queries bounded from their first expensive operation.** Fetching all world points and filtering them after the fact defeats much of the value of partitioning.

An unbounded stage is appropriate for small shared recipe data, not scanning every settlement. Also watch coarse-to-fine data replication: Epic warns that points from a larger grid can be duplicated into smaller cells and recommends `Cull Points Outside Actor Bounds` where appropriate. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-generation-modes-in-unreal-engine?lang=en-US)

World Partition and PCG partitioning are related but distinct responsibilities. My recommendation is to let World Partition handle applicable authored world content while TCE explicitly manages simulation snapshots and runtime-generated presentation residency. Do not treat runtime PCG output as an automatic persistent-world storage system.

### 2.3 Scheduler settings are controls, not hard frame-time guarantees

Relevant documented defaults include:

| Setting | Documented default | TCE starting experiment |
| --- | --- | --- |
| `pcg.FrameTime` | 16.667 ms | Start around **1–2 ms**, then measure |
| `pcg.RuntimeGeneration.NumGeneratingComponents` | 16 | Start with **2–4** |
| `pcg.RuntimeGeneration.FramesBetweenGraphSchedules` | 0 | Initially 0; throttle only when justified |
| `pcg.RuntimeGeneration.BasePoolSize` | 100 | Size from measured peak cell demand |

These defaults come from the generation-mode documentation; the suggested values are my proposed starting points. The concurrency setting limits generating components, not every worker thread used by the engine. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-generation-modes-in-unreal-engine?lang=en-US)

A time budget cannot be assumed to interrupt every expensive component-registration, physics, upload, or cleanup operation. Keep work units small enough that their unavoidable commits are acceptable.

The partition-actor pool is particularly important: **it doubles when exhausted**, which Epic identifies as an expensive gameplay-time operation. Pre-size or warm it using representative camera movement, rather than discovering its capacity limit during normal play. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/pcg-runtime-generation-debugging-in-unreal-engine)

### 2.4 What the available performance evidence establishes

| Evidence | Reported observation | What it means for TCE |
| --- | --- | --- |
| Runtime GPU-spawner report, November–December 2025; engine version unspecified | Good GPU performance, but **several-millisecond game-thread hitches** in `PrepareForExecute_GameThread()` while creating PISM components. The author reported greatly reduced spikes after enabling the componentless path. | A cheap GPU kernel can still have an expensive CPU commit. Measure mesh-type/component churn, not only point count. [Epic Developer Community Forums](https://forums.unrealengine.com/t/upcgstaticmeshspawnerdataprovider-prepareforexecute-gamethread-hitches/2688886) |
| PCG metadata investigation, April–June 2025 | A memory report showed **763 MB** of metadata. Correctly marking an editor-only graph removed **over 700 MB** of unnecessary runtime data. | Audit serialized graph output and unused attributes. **Do not mark TCE’s genuinely runtime-required graphs editor-only.** [Epic Developer Community Forums](https://forums.unrealengine.com/t/pcg-high-memory-usage-of-runtime-pcg-metadata-assets/2551052) |
| UE 5.7 release announcement | Epic reports significantly faster GPU PCG, without a transferable benchmark configuration or numerical speedup on the page. | Do not turn this into a claimed “2× faster” or a 4070 Ti capacity estimate. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-7-is-now-available) |

The FastGeo route uses `PCGFastGeoInterop` and `pcg.RuntimeGeneration.ISM.ComponentlessPrimitives`; it is documented as Experimental. Treat it as an A/B-tested optimization after establishing a working baseline, not a mandatory dependency for the first implementation. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-with-gpu-processing-in-unreal-engine?lang=en-US)

### 2.5 Memory and rendering costs

An illustrative field-data calculation:

\[
128 \times 128 \times 4\ \text{bytes} = 64\ \text{KiB per RGBA8 tile}.
\]

Thus, 64 active tiles require **4 MiB for one texture set**, or **8 MiB with double buffering**, before CPU copies, allocation overhead, mips, and other resources. This is a calculation, not an engine memory measurement.

The more consequential cost may be expanded points, metadata, generated components, and retained outputs. Epic recommends stripping unused attributes and replacing repeated asset properties with table indices; its newer point format can allocate only necessary properties. [Epic Developer Community Forums](https://forums.unrealengine.com/t/pcg-high-memory-usage-of-runtime-pcg-metadata-assets/2551052)

For TCE, carry a recipe index and a few changing values—not the entire crop, household, or ecological record on every decorative point.

Also separate **generation performance from rendering performance**. A graph that finishes quickly can still produce an unaffordable scene. Profile the resulting vegetation alongside TCE’s buildings, citizens, materials, shadows, and lighting; do not benchmark the PCG kernel in isolation and interpret that as a 60-fps result.

---

## 3. Precedents and what to learn from them

| Precedent | What is documented | Lesson and limitation |
| --- | --- | --- |
| **[LEGO Fortnite](https://www.epicgames.com/site/news/the-adventure-is-building-lego-fortnite-is-live?utm_source=chatgpt.com)** — shipped, 2023 | Epic states that World Partition streams its 95 km² playable space and PCG dynamically creates detailed environments. | Strong evidence that PCG can serve a shipped large-world runtime architecture. The announcement does not disclose a Rust-like simulation bridge, generation budgets, or the current 5.8 scheduler configuration. [Epic Games Store](https://www.epicgames.com/site/news/the-adventure-is-building-lego-fortnite-is-live) |
| **[Horizon Zero Dawn / Decima](https://www.guerrilla-games.com/read/gpu-based-procedural-placement-in-horizon-zero-dawn?utm_source=chatgpt.com)** — shipped; GDC 2017 talk by Jaap van Muijden | A GPU procedural-placement system assembles environments around the player from authored rules, including more than vegetation alone. | The closest architectural analogue: retain compact world descriptions and reconstruct nearby detail. It is a custom engine implementation, not evidence of UE GPU-spawner feature parity. [Guerrilla Games](https://www.guerrilla-games.com/read/gpu-based-procedural-placement-in-horizon-zero-dawn) |
| **[PCG Biome Core and Sample](https://dev.epicgames.com/documentation/unreal-engine/procedural-content-generation-pcg-biome-core-and-sample-plugins-in-unreal-engine)** — current 5.8 documentation | Data-driven biome definitions, texture/spline/volume inputs, exclusions, a crop-field generator, and a separate camera-centered GPU runtime graph. | Probably the most useful sample for TCE. Borrow its palette, exclusion, and layered-generation patterns. The plugin remains Experimental; Epic recommends copying it when necessary to protect production content from future changes. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/procedural-content-generation-pcg-biome-core-and-sample-plugins-overview-guide-in-unreal-engine) |
| **[Updated City Sample](https://www.unrealengine.com/learning/city-sample-gets-a-major-update-with-pcg-and-unreal-mcp-workflows?utm_source=chatgpt.com)** — UE 5.8, August 2026 | New PCG-built city levels, assembly assets, building/road grammars, and `CitySamplePCG_demo`. | Useful current-version references for authored kits and AI-assisted authoring. This is **not** proof that a continually changing simulation can regenerate a city within TCE’s frame budget. [Unreal Engine](https://www.unrealengine.com/learning/city-sample-gets-a-major-update-with-pcg-and-unreal-mcp-workflows) |
| **[PCG Extended Toolkit](https://github.com/PCGEx/PCGExtendedToolkit)** — open-source, MIT | Native PCG extensions for paths, spatial operations, graph structures, filtering, and asset management. The repository distinguishes unstable `main` from engine-version branches. | Useful code to study, especially for fence/path processing. Adopt selectively and pin an engine-matched revision; it need not become a foundational dependency. [GitHub](https://github.com/Nebukam/PCGExtendedToolkit) |
| **[Deussen et al., “Realistic Modeling and Rendering of Plant Ecosystems”](https://algorithmicbotany.org/papers/ecosys.sig98.html?utm_source=chatgpt.com)** — SIGGRAPH 1998 | Separates plant distribution, including ecosystem simulation, from geometric realization and approximate instancing. | Supports TCE’s proposed separation between ecological state and representative rendered plants. It is foundational graphics research, not a modern real-time UE benchmark. [Algorithmic Botany](https://algorithmicbotany.org/papers/ecosys.sig98.html) |

The common architectural lesson is **compact causes, derived geometry**. None of these examples justifies storing simulation truth only in generated mesh instances.

---

## 4. Recommended implementation for TCE

### 4.1 Establish a strict ownership boundary

Use the following division:

```
Rust kernel
    Owns simulation state, persistent IDs, land use, growth and construction.
        ↓ versioned immutable tile snapshots
C++ runtime bridge
    Owns snapshot lifetime, dirty-region queue and Unreal-facing data.
        ↓ bounded generation requests
PCG
    Owns disposable procedural dressing.
        ↓
Rendering
    Displays instances; significant objects retain simulation-ID mappings.
```

A citizen harvesting a tree should change a Rust object first. That change then removes or alters its representation. A camera leaving a PCG generation radius must never delete the underlying resource.

For fences, Rust should supply the authoritative boundary and gate state. PCG can choose posts, rails, variation, and adjoining clutter. Keep traversability and meaningful collision ownership independent of whether decorative PCG cells are currently resident.

### 4.2 Keep the DLL interface small and boring

Use a versioned C ABI with fixed-width types, explicit lengths, status codes, and documented ownership. A conceptual snapshot—not a proposed Unreal API—might contain:

```
TileKey:           signed integer tile coordinates
SnapshotRevision:  monotonically increasing per tile/layer
TerrainRevision:   changes when placement height or normals change
RecipeVersion:     identifies the rendering rules
Fields:            land use, fertility, moisture, biomass
Objects:           stable ID, local transform, recipe ID, visual state
Boundaries:        polylines and exclusion footprints
```

Prefer bulk copies into C++-owned staging storage for the first implementation. Add reference-counted immutable shared buffers only after measuring copy cost.

Do not pass Rust `Vec`, `String`, or ordinary C++ containers directly across the ABI. Prevent exceptions and Rust panics from escaping the boundary; Rust’s documentation explains the distinct failure behavior of unwinding across FFI and the limitations of `catch_unwind`. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Stage the Windows DLL through Unreal Build Tool’s `RuntimeDependencies` mechanism and verify it in the packaged executable, not only the editor. Epic documents the DLL staging and loading requirements explicitly. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

Keep UObject-dependent work and scene mutations on supported Unreal execution paths. Do bulk field sampling and filtering against immutable data without repeatedly locking the live simulation.

### 4.3 Use two PCG lifetimes, with one owner for each

**First implementation: application-managed tiles.** Use a bounded set of tile components with `GenerateOnDemand`. TCE decides which tiles are resident and when their decoration changes. `GenerateLocal` is the documented nonreplicated generation operation; generation is delayed rather than an immediate synchronous completion. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGComponent?lang=en-US)

This is easier to validate than beginning with a single large hierarchical graph whose residency, dirtying, and dependencies all interact.

**Second implementation: scheduler-managed near detail.** Add a separate `GenerateAtRuntime` layer for grass, stubble, flowers, and similar replaceable content. Let PCG manage its local execution sources and partition actors. Do not manually clean up those objects behind the scheduler’s back. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/FPCGRuntimeGenScheduler)

For TCE’s elevated camera, test a dedicated generation source projected onto the relevant ground region. At overview zoom, display field and vegetation coverage through coarser representations rather than expanding blade-level generation to cover everything visible.

### 4.4 Make change notification and cache identity explicit

A robust update sequence is:

1. Rust publishes a new immutable tile/layer snapshot.
2. The bridge records its revision and dirty bounds.
3. It merges overlapping changes and retains only the latest pending revision.
4. It invalidates the relevant PCG dependencies.
5. It requests generation through the owner of that layer’s lifecycle.

UE 5.8 exposes `UPCGSubsystem::DirtyGraph(Component, Bounds, Flag)`, intersecting-local-component queries, and runtime execution-source refresh APIs. These are useful integration points, but **bounded dirtying does not promise per-point differential mesh updates**. Validate the actual refresh scope with your graph and chosen component layout. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGSubsystem)

For a native source node, include the relevant snapshot and recipe revisions in dependency identity. `IPCGElement` exposes dependency CRC and cacheability hooks; cache lookup can occur before execution-context creation. Reading a revision only inside the execution body is therefore too late to govern an earlier cache hit. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/IPCGElement)

One important 5.8 default: **the CPU graph cache is disabled in game worlds and enabled in editor worlds**. Test both settings deliberately. Cache disabling does not remove the need to tell the scheduler that already-generated content is obsolete. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/pcg-runtime-generation-debugging-in-unreal-engine)

### 4.5 Separate appearance changes from population changes

This is likely TCE’s largest avoidable source of unnecessary generation.

| Simulation change | Recommended rendering response |
| --- | --- |
| Season tint, snow amount, wetness, wind | Shared material parameters |
| Crop maturity within a compatible mesh representation | Instance custom data or a field-state texture |
| Harvested area or a new building footprint | Immediate exclusion/direct representation change, followed by bounded dressing regeneration |
| Species distribution, vegetation density, field topology | Regenerate affected tiles |
| One fence segment or resource tree changes | Direct stable-ID instance update |
| Offscreen region changes repeatedly | Update authoritative data; generate only the latest state when needed |

These are proposed policies. The key is to avoid turning “one simulated day elapsed” into “regenerate every visible graph.”

For crop fields, keep row orientation and spacing stable while varying growth, coverage, and appearance. When the representation genuinely changes—for example, standing crop to stubble—perform a controlled tile or subfield update.

### 4.6 Preserve identity across regeneration

Use stable candidate identities:

\[
\text{candidate key}
=
H(\text{world seed},\text{absolute tile},\text{layer},\text{candidate index},\text{recipe version}).
\]

Do **not** include the current snapshot revision in the random seed. Revision controls freshness; it should not reroll every plant after a fertility update.

For decorative vegetation, a simple proposed thinning rule is:

\[
\text{keep candidate }i
\quad\text{when}\quad
u\_i < p\_i,
\]

where \(u\_i\) is a stable hashed value and \(p\_i\) is the suitability/coverage probability derived from current fields. Increasing fertility then adds candidates without relocating all existing ones.

Use a consistent tile-boundary ownership rule, with a neighbor halo for spacing and exclusion queries. Persist authoritative removals or object state so cleanup and regeneration cannot resurrect harvested resources.

### 4.7 Handle overlapping generations without stale visual commits

Allow at most one active generation per application-managed tile/layer initially. Coalesce subsequent updates into a latest-only pending request.

There is an important distinction between **discarding a stale calculation** and **undoing a stale spawn**. A completion callback may arrive after a standard spawner has already modified the scene. Do not advertise that callback as an atomic commit barrier.

For the baseline, accept controlled visual latency and serialize generations. Where atomic replacement is necessary, use a custom managed output or staging representation that checks the revision before publishing. This is additional engineering, not an assumed built-in PCG transaction.

During fast-forward, skip intermediate visual revisions. There is no benefit in visibly generating every historical harvest that occurred while the player advanced a century.

### 4.8 Avoid tying the bridge unnecessarily to Landscape

TCE’s land-use and fertility data do not need to become painted Landscape layers to drive PCG.

Provide height, normals, and suitability from the same terrain source used by the renderer. When using Unreal Landscape, GPU PCG offers collision-height upload, RVT sampling, and generated Landscape textures. RVT sampling requires adequate residency/priming; Landscape-specific nodes should not be assumed to support a custom terrain implementation unchanged. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-with-gpu-processing-in-unreal-engine?lang=en-US)

My recommendation is to start with direct height/normal sampling from TCE’s tile snapshot. Add RVTs only when they solve a measured rendering or data-sharing problem.

### 4.9 Performance targets and acceptance tests

At 60 fps, the frame interval is approximately **16.67 ms**. The following are proposed starting targets:

| Measurement | Initial target |
| --- | --- |
| Incremental PCG/bridge game-thread work during routine streaming | Roughly 0.5–1 ms average; investigate bursts above 2 ms |
| Optional GPU-generation work | Start with a 0.5–1 ms allowance during generation bursts |
| External update coalescing | Approximately 100–250 ms wall-clock for nonurgent dressing |
| Active application-managed work | One generation per tile/layer; bounded global concurrency |
| Pending changes | Latest revision only; no unbounded historical backlog |
| Memory behavior | Stable working set after repeated travel and regeneration |

These targets are not additive guarantees: game, render, GPU, and Rust worker work interact. In particular, avoid configuring Rust and Unreal to independently saturate all CPU resources.

Measure **time-to-visible-result**, not just graph execution. A useful derived condition is:

\[
R\_{\text{generation}}
\ge
R\_{\text{visible}}
+
v\_{\text{camera}}\,t\_{\text{generation latency}}
+
\text{safety margin}.
\]

Use measured high-percentile latency and the fastest supported camera movement. Teleports require a separate preloading or temporary fallback policy.

The acceptance suite should include continuous camera travel, rapid turns, teleportation, stationary-camera simulation changes, mass land-use conversion, accelerated seasons, save/load, terrain edits, and repeated unload/reload. Run both isolated PCG tests and the integrated 50,000-agent workload.

For instrumentation, use:

```
pcg.RuntimeGeneration.EnableDebugOverlay 1
pcg.GraphExecution.DebugDrawGeneratedCells 1
```

The runtime overlay reports generation activity, pool state, and scheduler game-thread time; it is available in Editor and Development, not Shipping. Epic also identifies Unreal Insights for CPU investigation and `ComputeFrame_ExecuteBatches` for GPU PCG work. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/pcg-runtime-generation-debugging-in-unreal-engine)

### 4.10 Pitfalls to put directly into the regression suite

**Editor success mistaken for packaged success.** A 5.6.1 forum thread documents a partitioned-generation setup that failed in PIE until its logged configuration problem was resolved; subsequent users reported unresolved packaged-build failures involving GPU grass maps. These are historical, setup-specific reports—not proof that 5.8 is generally broken. They justify testing a cooked build immediately. [Epic Developer Community Forums](https://forums.unrealengine.com/t/pcg-runtime-generation/2648015)

**GPU spawn missing instances.** The documented weighted GPU selector depends on properly varied point seeds; identical seeds can exceed a mesh’s estimated allocation. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-with-gpu-processing-in-unreal-engine?lang=en-US)

**Excessive serialized or retained metadata.** Strip unused attributes, avoid unnecessary top-level outputs, and use recipe-table indices. Distinguish runtime-required data from editor-only generation leftovers. [Epic Developer Community Forums](https://forums.unrealengine.com/t/pcg-high-memory-usage-of-runtime-pcg-metadata-assets/2551052)

**AI-generated code using old PCG APIs.** Pin UE, plugin revisions, and the bridge schema. Require a packaged-build test, deterministic snapshot test, and dirty-region test for each integration change. Keep custom nodes small enough that their cache keys, thread behavior, and ownership can be reviewed independently.

---

## 5. Sources and version applicability

The linked precedents above provide the shipped examples, sample projects, open-source code, paper, and GDC presentation. The core implementation references are:

| Reference | Applicability |
| --- | --- |
| [PCG generation modes](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-generation-modes-in-unreal-engine) and [runtime scheduler API](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/FPCGRuntimeGenScheduler?utm_source=chatgpt.com) | UE 5.8 documentation: triggers, partitioning, hierarchy, scheduling, ownership |
| [GPU processing guide](https://dev.epicgames.com/documentation/unreal-engine/using-pcg-with-gpu-processing-in-unreal-engine) | UE 5.8: compute graphs, supported nodes, procedural-instancing restrictions, FastGeo, Landscape and RVT paths |
| [Runtime debugging and profiling](https://dev.epicgames.com/documentation/unreal-engine/pcg-runtime-generation-debugging-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8: runtime/editor cache defaults, overlays, profiling commands |
| [PCG node reference](https://dev.epicgames.com/documentation/unreal-engine/procedural-content-generation-framework-node-reference-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8: texture sampling, tables, splines, filtering, grid control |
| [IPCGElement](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/IPCGElement?utm_source=chatgpt.com), [UPCGPointArrayData](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGPointArrayData?utm_source=chatgpt.com), [UPCGTextureData](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGTextureData?utm_source=chatgpt.com) | Native node integration, dependency identity, current point and texture representations |
| [UPCGComponent](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGComponent) and [UPCGSubsystem](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/PCG/UPCGSubsystem?utm_source=chatgpt.com) | UE 5.8 generation, dirtying, refresh, and lifecycle APIs |
| [Unreal third-party library integration](https://dev.epicgames.com/documentation/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com) and [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | DLL staging and ABI/lifetime/error-boundary design |
| [UE 5.7 announcement](https://www.unrealengine.com/news/unreal-engine-5-7-is-now-available?utm_source=chatgpt.com) and [UE 5.8 announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available) | Maturity and release context; do not substitute these announcements for per-feature documentation |

**Bottom line:** build the bounded CPU snapshot-to-PCG path first, with explicit revisions and a tested packaged build. Add scheduler-managed nearby detail next. Adopt GPU spawning or FastGeo only where profiling shows a worthwhile gain and their feature restrictions are acceptable. That gives TCE a controllable rendering subsystem without making its simulation depend on transient procedural geometry.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92973-4c30-83e9-b793-098cf2054c3a)
