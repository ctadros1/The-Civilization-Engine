# Runtime assembly of Nanite modular buildings in UE 5.8

**Recommendation:** Build TCE’s renderer around **spatially partitioned `UInstancedStaticMeshComponent` pools containing pre-cooked Nanite kit meshes**, with a separate system that changes each building’s *representation* with distance and importance. Keep building identity, construction state, persistence, and simulation in Rust.

Use HISM selectively for non-Nanite or fallback content, not as the default for Nanite buildings. Epic’s current guidance explicitly favors ISM when Nanite supplies the geometry culling and LOD system. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

The central distinction is:

> **Nanite reduces the geometry needed to render an instance. TCE must also control how many instances exist, change, cast shadows, participate in lighting, and require Unreal objects.**

A city containing tens of thousands of logical buildings is a reasonable architectural target. Whether it meets **1440p/60 fps on your RTX 4070 Ti** depends on the resident working set, materials, lighting, camera position, and mutation workload—not simply the total building count.

**Version scope:** This report uses the UE **5.8** documentation available on **September 27, 2026**, supplemented by explicitly dated production examples. UE 5.8 is released; however, several relevant subsystems remain experimental. Proposed TCE budgets below are engineering starting points, **not measurements from your hardware**. [Unreal Engine](https://www.unrealengine.com/en-US/news/unreal-engine-5-8-is-now-available)

---

## 1. Options: what each technique actually provides

| Technique | How it works | Strengths and costs | Fit for TCE |
| --- | --- | --- | --- |
| **Nanite ISM** | A component holds transforms and instance data for one mesh/material configuration. Nanite handles geometry detail and visibility. | Avoids a component for every piece; supports runtime instance changes. Component count and update work still matter. | **Default rendering path.** |
| **Nanite HISM** | Adds an instance hierarchy and associated build/update machinery to ISM. | Useful for large, mostly unchanged non-Nanite populations; hierarchy maintenance adds complexity during construction and demolition. | Optional fallback path, not the default. |
| **Ordinary Actors / Static Mesh Components** | Each object retains its own Unreal object/component identity. | Convenient independent behavior, but unnecessary object management when Rust already owns the building. | Interactive exceptions; avoid an actor per kit piece. |
| **Instanced Actors plugin** | Regional management combines instance representations with Mass/Actor-oriented infrastructure. | Useful when gameplay objects must switch representations; introduces another entity-management system. **Experimental in 5.8.** | Usually redundant beside Rust’s authoritative simulation. |
| **Runtime PCG** | Graphs generate and clean up content around generation sources, with spatial grids and scheduling. | Useful procedural orchestration; does not eliminate rendering, persistence, or update costs. | Optional tool for decoration or authoring, rather than TCE’s core building database. |
| **Nanite Assemblies** | Micro-instances are combined within an asset’s Nanite hierarchy, allowing the complete assembly to simplify more effectively. | Attractive for repeated fine detail; creation tools are editor-oriented and experimental. Parts lack ordinary per-part material instance data. | Investigate for **prebuilt macro-parts**, not arbitrary runtime building assembly. |
| **FastGeo** | Lightweight geometry representations and streaming reduce reliance on normal Actor/component machinery. | Relevant to dense worlds; **experimental in 5.8**. PCG interop explicitly supports runtime spawning. | A later backend experiment behind a stable TCE rendering interface. |
| **Prebuilt merged meshes / macro-kits** | Several details become one reusable asset or larger kit element. | Fewer instances, but reduced independent editability and potentially worse lighting representation if merged indiscriminately. | Strong complement to ISM, especially for distant buildings. |

The ISM/HISM distinction above follows Epic’s current guide; modern ISM also supports per-instance conventional LOD, so older advice that only HISM can do this is obsolete. HISM’s current API exposes automatic tree rebuilding, asynchronous building, and `BuildTreeIfOutdated`. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Instanced Actors depends on MassGameplay and related systems; it is not simply a switch that makes arbitrary Actors as cheap as mesh instances. Runtime PCG, meanwhile, supports generation and cleanup in standalone execution and can use World Partition streaming sources. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/InstancedActors?lang=en-US)

### What “runtime Nanite assembly” should mean in TCE

For the first implementation, it should mean:

**Select an existing cooked mesh → supply transform and material data → add an instance.**

It should **not** require constructing new Nanite assets in the packaged executable. Nanite Assemblies’ documented builders are exposed through editor tooling; I did not find a documented, production-ready general-purpose workflow for converting every newly generated building into a new Nanite asset inside a Shipping build. Treat that as a separate research problem, not a hidden dependency of the basic renderer. Assemblies also currently support only one assembly layer. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-assemblies)

A roof section containing modeled tiles can therefore be one instanced kit element. It need not become several hundred independent tile instances.

---

## 2. Scaling rules: counts, components, and mutations

### 2.1 Count kit instances—not buildings

Use this equation in TCE’s performance dashboard:

\[
N\_{\text{resident instances}}
=
\sum\_{\text{resident buildings}}
N\_{\text{parts in selected representation}}
+
N\_{\text{other resident instances}}
\]

For illustration, fully materializing every building produces:

| Buildings | 32 parts/building | 128 parts/building | 512 parts/building |
| --- | --- | --- | --- |
| 10,000 | 320,000 | 1,280,000 | 5,120,000 |
| 25,000 | 800,000 | 3,200,000 | 12,800,000 |
| 50,000 | 1,600,000 | 6,400,000 | 25,600,000 |

These are arithmetic scenarios, not engine benchmarks.

Epic documents a **16-million-instance scene limit**, counting streamed-in instances, including non-Nanite instances—not just visible Nanite pieces. This is a hard ceiling, not a sensible performance target. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

Consequently, a design that creates one instance per brick, roof tile, beam joint, window pane, and ornament can fail before triangle throughput becomes the dominant problem.

### 2.2 How many instances per component?

**There is no documented universal “best” count.** Do not import limits such as 1,023 or 65,535 from unrelated APIs or engines.

The practical trade-off is:

| Batching choice | Advantage | Failure mode |
| --- | --- | --- |
| One component per building and mesh | Simple building-local ownership | Component count grows with buildings × distinct kit meshes |
| One global component per mesh | Minimal component count | Awkward unloading, large mutation domains, poor ownership locality |
| **One component per spatial cell and mesh configuration** | Bounded ownership, updates, and unloading | More components; sparse material/mesh combinations can fragment batches |
| Further split dense cell buckets | Bounds individual rebuild/allocation events | Excessive splitting recreates component-management overhead |

**Proposed starting configuration—not an Epic recommendation:**

| Parameter | Initial TCE choice | Benchmark sweep |
| --- | --- | --- |
| Building-render cell width | **128–256 m** | 64, 128, 256, 512 m |
| Dense bucket size before considering a split | **Around 4,000–16,000 instances** | 1,000, 4,000, 16,000, 64,000 |
| Initially targeted full-detail resident working set | **250,000–1,000,000 instances** | Expand after measuring complete scenes |
| Component creation | Only for occupied buckets | Measure sparse and style-diverse settlements |

These are starting hypotheses. A bucket containing 80 instances is not inherently wrong; adding empty or artificial instances to reach a target would be counterproductive.

Use a bucket key approximately like:

```
CellId
× MeshAssetId
× MaterialSetId
× RenderPolicy
```

`RenderPolicy` should separate materially different shadow, lighting, collision, mobility, and deformation behavior. Many such properties belong to the component rather than individual instances. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

### 2.3 Adding and removing instances

UE 5.8 exposes useful primitives rather than a complete city-update scheduler: `PreAllocateInstancesMemory`, batch transform updates, batch removals, and ID-based operations. `AddInstances` accepts an array of transforms and a navigation-update flag. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent?lang=en-US)

Recommended treatment:

| Operation | Cost to anticipate | Implementation rule |
| --- | --- | --- |
| Add a batch to an existing bucket | Array growth, instance-data updates, bounds and optional physics/navigation work | Reserve capacity, then batch additions |
| Create a new bucket/component | UObject/component setup, registration, render-resource work | Budget separately from adding to an existing bucket |
| Update transforms/custom data | Data tracking, CPU preparation, render-thread processing, GPU transfer | Submit only changed data; coalesce nearby updates |
| Remove instances | Index changes, tracking, optional array compaction and physics/navigation changes | Batch removals; explicitly handle reordering |
| Rebuild a whole chunk | Work proportional to that chunk’s complete representation | Reserve for large changes or representation replacement |
| Modify HISM populations | Above costs plus hierarchy work | Batch mutations before requesting tree rebuilding |

Do not describe these as fixed “microseconds per instance.” The complete cost depends on enabled features, allocation history, component size, and the engine path taken.

At the array level, appending with sufficient capacity is much cheaper than growing and copying an entire array. Likewise, swap removal avoids shifting every following element—but **that does not make the entire Unreal removal operation constant-time**.

UE’s `SetRemoveSwap()` explicitly changes removal to `RemoveAtSwap` and warns that resulting instance order changes. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetRemoveSwap)

### 2.4 Stable identity is mandatory

Persist:

```
BuildingId + PartId + BuildingRevision
```

Do **not** persist:

```
Component pointer + InstanceIndex
```

An instance index is a location in a mutable rendering container, not a simulation identity.

Maintain mappings from stable TCE IDs to current render handles, plus a reverse mapping for picking. After swap removal, update the moved instance’s mapping. After streaming or component replacement, rebuild mappings.

UE’s instance-data manager tracks changes and ID/index mappings, but its documentation explicitly says its mapping is not serialized and can reset when the proxy is recreated. Engine instance IDs are therefore not substitutes for persistent TCE IDs. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/FInstanceDataManager)

### 2.5 Avoid unnecessary full rebuilds

Modern UE has change-tracking infrastructure; it is inaccurate to assume every individual edit necessarily uploads the entire component. Conversely, allocation or representation changes can still produce broad work. The instance-data manager documents both tracked changes and paths that force full updates. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/FInstanceDataManager)

For ordinary construction:

* Accumulate modifications by bucket.
* Coalesce repeated changes to the same part.
* Apply additions, removals, and data changes in controlled batches.
* Use APIs’ dirty-notification controls appropriately rather than rebuilding render state for every float.
* Measure the deferred render-thread/GPU work, not just the duration of the game-thread function call.

For mass demolition or changing a building’s representation, replacing a bounded bucket/chunk may be simpler and faster than thousands of scattered edits. Benchmark both paths.

---

## 3. Materials: variation without material-instance proliferation

ISM supports per-instance custom data specifically to vary rendering without creating a dynamic material instance for every object. Use shared master materials and a deliberately small, versioned data layout. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

### Suggested eight-float layout

| Slots | Suggested meaning | Update frequency |
| --- | --- | --- |
| 0–2 | Tint or palette-adjustment RGB | Construction, repainting |
| 3 | Weathering/maintenance state | Infrequent simulation changes |
| 4 | Damage/burn state | Damage events |
| 5 | Window/emissive occupancy state | Occupancy or lighting events |
| 6 | Deterministic visual seed | Creation only |
| 7 | UV/material variation selector | Creation or renovation |

This is a proposed layout, not a required format. Some buckets may need fewer floats.

**Important UE 5.8 pitfall:** Changing `NumCustomDataFloats` reallocates the full custom-data buffer **and resets all values to zero**. Set the layout before populating a component; do not casually expand it after buildings are visible. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetNumCustomDataFloats)

The current `SetCustomData` API includes both individual-instance and instance-range overloads, with a render-state-dirty flag. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetCustomData)

### Weathering

Prefer a shader combining authored masks with a small state value:

```
appearance =
    base material
  + authored exposure/cavity masks
  + building weathering state
  + deterministic variation
```

Do not rewrite every wall’s age every rendered frame. Pass global simulation time once, or update a coarse weathering state only when it changes meaningfully. For centuries-long simulations, use bounded/relative time encodings rather than putting an ever-growing absolute timestamp into every float.

### Window lights

Use emissive windows as the **appearance layer**, not one real light per window.

For a façade with many windows, a seed plus window coordinates can produce inexpensive patterned occupancy. When exact simulated room occupancy must be visible, either use separate window instances near the camera or supply indexed room-state data. A single façade-level float cannot encode arbitrary independent states for hundreds of windows.

Keep actual light components in a separately budgeted pool for nearby important rooms, street lights, and fires. Lumen’s treatment of small objects and its lighting representation means visible emissive detail and reliable scene illumination are separate concerns. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

### Material and deformation constraints

Nanite’s documented material support is **opaque and masked**; genuinely translucent glass needs a separate rendering path. Keep that path sparse and distance-limited. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

Avoid putting permanently evaluated World Position Offset into every building material merely to support occasional damage animation. Fortnite’s shipped building workflow disabled WPO evaluation on undamaged pieces and enabled it only during their wobble effect. Its experience also showed that opaque geometry could outperform masked cards for its foliage content—an asset-dependent result, not a universal rule. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/bringing-nanite-to-fortnite-battle-royale-in-chapter-4)

---

## 4. Culling, runtime HLOD, and World Partition

### 4.1 Four different systems must remain separate

| System | What it decides |
| --- | --- |
| **Simulation residency** | Which authoritative state Rust keeps active or serialized |
| **Render streaming** | Which building representations exist in Unreal |
| **Building representation LOD** | Detailed kit, simplified shell, skyline representation, or nothing |
| **Nanite geometry LOD/culling** | Which geometry clusters are needed for the current views |

HISM’s instance tree is **not** a substitute for building-level HLOD.

Nanite can simplify the geometry within many separate instances, but ordinary instances remain separate scene objects. Epic’s Assemblies documentation explains why this becomes expensive for extreme micro-instancing: visible independent instances retain root-cluster and GPU-scene overhead, whereas an assembly can simplify across its parts. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-assemblies)

### 4.2 Use semantic building representations

I recommend making each building recipe generate multiple representations from the same authoritative shape:

| Representation | Suggested contents | Initial use |
| --- | --- | --- |
| **Detailed** | Structural kit, windows, doors, nearby interior details | Street-level observation |
| **Shell** | Larger wall sections, simplified roof, essential silhouette | Mid-distance and ordinary aerial views |
| **Skyline** | A few massing/roof elements; simplified materials | Distant districts |
| **Absent** | No render objects | Outside the rendering working set |

Use projected size and visual importance, with distance as an initial heuristic. Tall landmarks should retain detail farther away than sheds.

A useful arithmetic example:

* 1,000 nearby buildings × 128 parts = 128,000 instances.
* 9,000 middle-distance buildings × 12 parts = 108,000.
* 40,000 distant buildings × 4 parts = 160,000.

That is **396,000 building instances**, rather than **6.4 million** with every building at 128 parts. The simulation still contains all 50,000 buildings.

These simplified representations can themselves use pre-cooked Nanite kit assets. TCE does not need runtime mesh baking to obtain this reduction.

### 4.3 What stock HLOD does—and does not—provide

World Partition HLOD generates representations for unloaded cells. Its layer types include instancing, merged meshes, and simplified meshes; the merged/simplified paths create proxy assets. The normal workflow builds HLODs in the editor or through a commandlet. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition---hierarchical-level-of-detail-in-unreal-engine)

**Engineering implication:** Prebuilt HLODs cannot know the shape of a city that does not exist until the simulation constructs it.

Therefore:

* Use stock World Partition/HLOD for compatible authored content.
* Manage TCE’s newly generated buildings explicitly.
* Treat TCE’s shell/skyline system as **runtime HLOD-like representation management**, not as an assumption that Unreal automatically bakes new HLOD proxies.

Do not assume spawning a manager Actor at a coordinate automatically gives its runtime instances correct World Partition lifetime. Establish explicit ownership, source tracking, loading, and cleanup.

Runtime PCG can help with this orchestration: it supports generation radii, cleanup radii, scheduling, and pooled generated actors. That still does not establish automatic runtime proxy baking for arbitrary evolving buildings. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-pcg-generation-modes-in-unreal-engine)

### 4.4 Culling is not unloading

An offscreen or occluded instance can still occupy scene memory and require other processing. Use explicit removal/unregistration when a representation leaves the working set.

There is also a documentation caveat: Nanite’s support page still lists view-specific distance culling as unsupported. Do not assume every traditional ISM/foliage distance-fade control behaves identically on your chosen 5.8 Nanite path. Validate it, and make explicit residency management the reliable memory-control mechanism. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

Use hysteresis so that hovering near a boundary does not repeatedly create and destroy buildings. Prioritize camera-facing cells, but retain enough surrounding geometry for shadows and lighting—not just the current camera frustum.

### 4.5 Distant lighting is a separate integration problem

Stock Lumen hardware-ray-traced Far Field requires World Partition HLOD, specifically built HLOD1. A custom skyline renderer does not automatically satisfy that integration contract. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

For TCE’s first version, prefer an explicitly limited near-field lighting solution plus acceptable distant appearance. Do not make kilometer-scale physically faithful reflections across a changing city a prerequisite for the building renderer.

---

## 5. Memory and GPU budgets

### 5.1 What instance memory numbers mean

Epic gives an illustrative comparison of approximately **672 GPU bytes for a primitive versus 64 bytes for a basic instance**. These are not complete per-object CPU/GPU costs for every UE 5.8 feature combination. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

A useful first-order budgeting model is:

\[
M\_{\text{basic instance payload}}
\approx N(64 + 4F)\ \text{bytes}
\]

where \(F\) is the custom-float count. With eight floats, this gives approximately **96 bytes per instance**:

| Resident instances | Illustrative basic payload, eight floats |
| --- | --- |
| 250,000 | 24 MB |
| 1,000,000 | 96 MB |
| 3,200,000 | 307 MB |
| 6,400,000 | 614 MB |

These are decimal MB and **exclude** CPU arrays, ID maps, component overhead, physics, lighting representations, staging buffers, allocation slack, and other renderer data. They explain why shared geometry is valuable, but not why millions of instances are automatically cheap.

For CPU accounting, measure actual allocated capacities rather than multiplying by a guessed universal transform size:

\[
M\_{\text{CPU}}
=
M\_{\text{Rust state}}
+
M\_{\text{render recipes/cache}}
+
M\_{\text{UE instance arrays}}
+
M\_{\text{ID maps}}
+
M\_{\text{physics/navigation}}
+
M\_{\text{pending work}}
\]

An endless simulation particularly needs a cap on pending render work and old cached representations.

### 5.2 Proposed 12 GB VRAM allocation

This is a **planning envelope**, not a measured engine breakdown. Resources must be reconciled with actual profiling to avoid double-counting.

| Category | Initial planning allocation |
| --- | --- |
| Texture residency | 3.0 GB |
| Nanite geometry/root residency | 1.25 GB |
| Instance/GPU-scene data and related buffers | 0.5 GB |
| Lumen/distance fields/optional RT resources | 1.25 GB |
| Virtual Shadow Maps | 1.0 GB |
| Render targets and transient working memory | 2.0 GB |
| Other GPU resources | 0.5 GB |
| **Working total** | **9.5 GB** |
| **Uncommitted margin on a nominal 12 GB card** | **2.5 GB** |

Measure peak memory during loading, representation replacement, demolition, and rapid camera movement—not just a stationary scene.

Nanite’s streaming pool is configurable, but an undersized pool can thrash even in a static view. Separately, candidate/visible cluster buffers have fixed capacities; exceeding them can produce missing or blinking geometry rather than graceful automatic quality reduction. Profile with `NaniteStats` and the relevant visualization modes before increasing capacities. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-technical-details)

### 5.3 Lighting can impose a much tighter instance budget

Epic’s Lumen guide says hardware ray tracing rebuilds the top-level acceleration structure each frame and that cost scales with its instance population. Its console guidance is generally **under 100,000 post-cull ray-tracing instances**, while explicitly noting that Windows varies. This is not a raster-instance limit or a measured 4070 Ti threshold. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine)

**For TCE, benchmark software Lumen/global tracing before committing to hardware Lumen.** The 5.8 guide also documents **Lumen Lite** at Medium quality: lower-cost irradiance gathering and screen-space reflections. That gives you a useful fallback between full Lumen and disabling it. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine)

Keep ornamental kit pieces out of lighting representations when their contribution is negligible. However, software Lumen is not free: global-distance-field updates still depend on instance changes. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine)

Asset design must accommodate this. Epic recommends separate walls/floors/ceilings for software Lumen, warns against overly complex combined interiors, and recommends walls at least approximately **10 cm thick** to reduce distance-field leakage. “Merge everything” is therefore not a universally safe optimization. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

### 5.4 Shadow invalidation matters during accelerated time

WPO, animated geometry, and changing lights can invalidate Virtual Shadow Map caches. Current VSM behavior can move unchanged Movable meshes into static caching, so “mark every building Static” is not a reliable universal optimization. Correctly classify actual changes and inspect invalidated pages. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

For TCE, specifically test fast-forwarded sun movement, simultaneous construction, fires, and dusk lighting. A city that is fast while paused may be slow when the world changes.

### 5.5 Frame-time targets

At 60 fps, the total frame interval is **16.67 ms**. CPU and GPU overlap; do not add their budgets as if they execute serially.

Suggested initial acceptance targets:

| Metric | Proposed target |
| --- | --- |
| Normal complete-frame GPU time | 12–14 ms, retaining margin |
| Building geometry/material contribution | Initially aim for 3–5 ms; measure by controlled comparisons |
| UE game-thread work | Approximately 4–6 ms or less in representative play |
| Typical render-bridge mutation work | Approximately 0.5–1 ms on the game thread |
| Exceptional mutation work | Time-sliced; no unbounded single-frame rebuild |

These targets must leave room for terrain, people, water, effects, UI, and the Rust kernel. They are not predictions of achieved performance.

---

## 6. Precedents and what they establish

| Precedent | Evidence and lesson | Limitation |
| --- | --- | --- |
| **Fortnite Chapter 4 — shipped, UE 5.1-era work** | Nanite building meshes, offline detail generation, and selective WPO evaluation. Strong evidence for modular Nanite assets in a changing game world. | Does not publish a reusable “buildings per component” or add/remove throughput benchmark for TCE. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/bringing-nanite-to-fortnite-battle-royale-in-chapter-4) |
| **Original City Sample / Matrix-derived project — UE 5.0** | Buildings use hundreds of shared instances; the pipeline also creates specialized collision/roof geometry. Study the separation of visual detail and other representations. | Original city generation was in Houdini, not an autonomous runtime simulation. [dev.epicgames.com](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration?utm_source=chatgpt.com) |
| **City Sample PCG — UE 5.8** | Provides internal-Unreal city generation, building grammars, asset assemblies, and background silhouettes. Particularly useful authoring reference. | The documented city uses 18 dependent graphs, can take minutes to regenerate, and omits the original traffic/pedestrian setup. It is not evidence of hitch-free incremental simulation updates. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?lang=en-US) |
| **Nanite Assemblies foliage demonstration** | Epic reports a large tree falling from about **3.5 GB to 29 MB on disk**, and one view’s tree streaming memory from **36 MB to 2.7 MB**; the demonstrated scene contains **500,000 trees**. | Asset-specific demonstration, not a shipped building benchmark or evidence that arbitrary runtime assemblies have the same savings. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-assemblies) |
| **Godot MultiMesh — open-source engine** | Demonstrates bulk instance submission and spatial splitting. Its documentation recommends multiple regional MultiMeshes because visibility is otherwise all-or-nothing. | Its culling behavior differs from Nanite ISM; transfer the batching lesson, not its exact performance model. Current page flags incomplete updating for Godot 4.7. [Godot Engine documentation](https://docs.godotengine.org/en/stable/tutorials/performance/using_multimesh.html) |
| **zeux/niagara — MIT-licensed Vulkan renderer** | Readable experiments covering GPU submission, instance/meshlet culling, occlusion, and mesh shading. Useful for understanding why GPU-driven rendering still needs bounded scene data. | A renderer research project, not Unreal’s Niagara system or a drop-in city renderer. [GitHub](https://github.com/zeux/niagara) |

Two foundational references complement the practical examples:

**Müller et al., “Procedural Modeling of Buildings,” SIGGRAPH 2006** introduces CGA shape grammars for hierarchical architectural generation. Its relevance is the **building recipe and context-sensitive construction rules**, not modern rendering throughput. [ACM Digital Library](https://dl.acm.org/doi/10.1145/1141911.1141931)

**“A Deep Dive into Nanite Virtualized Geometry,” SIGGRAPH 2021**, and **“Nanite GPU-Driven Materials,” GDC 2024**, explain the geometry and material pipelines respectively. Treat them as architectural background; they do not replace the 5.8 API and feature documentation. [Advances in Real-Time Rendering](https://advances.realtimerendering.com/s2021/)

**Evidence gap:** I found no reproducible primary-source UE 5.8 benchmark matching your hardware, material set, lighting configuration, component sizes, and runtime add/remove workload. A credible implementation decision therefore needs a small TCE-specific benchmark, not extrapolation from a static “millions of instances” demonstration.

---

## 7. Recommended TCE architecture

### 7.1 Ownership and data flow

```
Rust simulation
  BuildingId, recipe, construction/damage state, revision
                        │
             Batched C-ABI change stream
                        │
UE C++ building-render subsystem
  Spatial residency + representation selection + work budget
                        │
         Cell-local mesh/material/policy buckets
                        │
             Nanite ISM components
                        │
       Separate nearby collision / interaction objects
```

The key rule is **one authoritative building state, several disposable representations**.

Rust should emit compact operations such as:

```
CreateBuilding
ChangeBuildingRecipe
ChangePartState
DestroyBuilding
```

The UE adapter expands or replaces only the currently required representation. At high simulation acceleration, coalesce intermediate revisions: rendering does not need to recreate every construction stage that occurred between displayed frames.

### 7.2 Keep the DLL boundary narrow

Use a versioned C ABI with fixed-layout records, explicit buffer ownership, and batch calls. Do not pass Rust `Vec`, C++ containers, or arbitrary UObject references across the boundary.

Generate recipes/transforms on worker threads; apply Unreal object/component mutations through the owning game-thread path. Unreal’s threaded-rendering documentation emphasizes that Actor/UObject state belongs to the game thread and must not be accessed casually from rendering work. Rust’s FFI documentation likewise requires explicit lifetime, layout, and unwinding contracts. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine)

Recommended safeguards include a `ChunkEpoch` to reject stale jobs after unloading, a `BuildingRevision` to reject obsolete updates, and tests for DLL shutdown while work is queued.

### 7.3 Separate visuals from physical and gameplay representations

For ordinary buildings, disable visual-piece collision, overlap generation, navigation influence, and ticking unless actually required.

Use simple collision/interaction geometry only in relevant regions. Keep Rust pathfinding authoritative rather than maintaining a second fully equivalent Unreal navigation simulation merely because the renderer contains buildings.

Doors, lifts, damage debris, and directly interactive objects can temporarily receive richer Unreal representations. The presence of a visible roof tile should not imply a corresponding physics body or gameplay Actor.

### 7.4 Keep new engine features replaceable

Use a small interface such as:

```
IBuildingRenderBackend:
    ApplyBatch(...)
    SetResidency(...)
    ReplaceRepresentation(...)
    GatherStats(...)
```

Start with plain ISM. Later compare FastGeo or other backends without changing save files or Rust simulation identity.

UE also contains `UISMPoolComponent`, which already manages shared ISMs and mesh groups. It is worth inspecting for implementation ideas, but its source path is under `Runtime/Experimental/ISMPool`; adopting it should be a deliberate dependency choice, not an assumption of long-term API stability. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/ISMPool/UISMPoolComponent)

For a solo developer, a thin, well-tested renderer using conventional runtime APIs is preferable to maintaining a custom Nanite renderer or several experimental systems simultaneously.

---

## 8. The benchmark that should gate implementation

Run packaged **Development** builds for instrumentation, then verify behavior and timing in **Shipping**. Record native 1440p separately from 1440p output using temporal upscaling. Do not count frame generation as satisfying the underlying 60-fps simulation/render target.

Use the same deterministic city, camera paths, and mutation streams for every backend.

| Test dimension | Required cases |
| --- | --- |
| Building scale | 10k, 25k, 50k logical buildings |
| Representation scale | 32, 128, 512 detailed parts; shell/skyline variants |
| Resident instances | 250k, 1m, 4m |
| Bucket size | 1k, 4k, 16k, 64k |
| Mutation bursts | 100, 1k, 10k additions/removals; data-only changes |
| Rendering | ISM versus HISM; materials fixed across comparisons |
| Lighting | Software Lumen, Lumen Lite, hardware Lumen |
| Camera | Street view, aerial panorama, fast traversal, turn, teleport |
| Simulation | Paused, normal speed, fast-forward, construction wave, demolition |
| Resource pressure | Cold load, warm load, repeated unload/reload, dusk, fires |

Measure **p50/p95/p99 frame time**, game/render-thread timing, component registration cost, GPU-scene update work, visible and resident instance counts, ray-tracing instance counts, memory peaks, queue length, and time until newly visible buildings become complete.

Use Nanite’s instance/overdraw/material-bin views and `NaniteStats`; inspect VSM invalidation and lighting representations separately. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-technical-details)

Correctness tests are equally important: random deletion must preserve ID mappings; unloading must invalidate stale jobs; custom-data layout changes must not silently erase state; demolished buildings must not remain in distant representations; and repeated loading must not grow memory indefinitely.

**Decision criterion:** Choose the simplest backend that meets the complete-scene frame-time target while construction, demolition, streaming, and simulation are active—not the backend with the highest static instance count.

---

## 9. Linked source guide and version applicability

| Source | Version / purpose |
| --- | --- |
| [UE 5.8 announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available) | Release context, 2026 |
| [ISM guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com), [ISM API](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent), [HISM API](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UHierarchicalInstancedStaticMesh-?utm_source=chatgpt.com) | Current 5.8 behavior and implementation entry points |
| [Instance-data manager](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/FInstanceDataManager?utm_source=chatgpt.com), [custom-data resizing](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetNumCustomDataFloats?utm_source=chatgpt.com), [swap removal](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetRemoveSwap?utm_source=chatgpt.com) | 5.8 mutation and identity details |
| [Nanite overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?utm_source=chatgpt.com), [technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-technical-details?utm_source=chatgpt.com) | 5.8 limits, memory controls, profiling |
| [Nanite Assemblies](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-assemblies?utm_source=chatgpt.com) | 5.8 capabilities and editor-oriented construction |
| [Instanced Actors](https://dev.epicgames.com/documentation/unreal-engine/API/PluginIndex/InstancedActors), [FastGeo](https://dev.epicgames.com/documentation/unreal-engine/API/PluginIndex/FastGeoStreaming?utm_source=chatgpt.com), [PCG FastGeo interop](https://dev.epicgames.com/documentation/unreal-engine/API/PluginIndex/PCGFastGeoInterop?utm_source=chatgpt.com), [ISM pool](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/ISMPool/UISMPoolComponent?utm_source=chatgpt.com) | Experimental alternatives in 5.8 |
| [World Partition](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition-in-unreal-engine?utm_source=chatgpt.com), [HLOD](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition---hierarchical-level-of-detail-in-unreal-engine?utm_source=chatgpt.com), [PCG generation modes](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-pcg-generation-modes-in-unreal-engine?utm_source=chatgpt.com) | 5.8 streaming and generation workflows |
| [Lumen performance](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine?utm_source=chatgpt.com), [Lumen technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine?utm_source=chatgpt.com), [Virtual Shadow Maps](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine?utm_source=chatgpt.com) | 5.8 lighting and shadow budgets |
| [Original City Sample](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration?utm_source=chatgpt.com), [City Sample PCG](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine) | Original UE 5.0 pipeline versus new 5.8 example |
| [Bringing Nanite to Fortnite](https://www.unrealengine.com/tech-blog/bringing-nanite-to-fortnite-battle-royale-in-chapter-4) | Shipped UE 5.1-era lessons, January 2023 |
| [Nanite SIGGRAPH course materials](https://advances.realtimerendering.com/s2021/?utm_source=chatgpt.com), [GPU-driven materials talk and slides](https://www.unrealengine.com/blog/take-a-deep-dive-into-nanite-gpu-driven-materials) | Foundational 2021 geometry architecture; 2024 material pipeline |
| [Procedural Modeling of Buildings](https://dl.acm.org/doi/10.1145/1141911.1141931?utm_source=chatgpt.com) | Müller et al., SIGGRAPH 2006; architectural shape grammars |
| [zeux/niagara code](https://github.com/zeux/niagara?utm_source=chatgpt.com), [Godot MultiMesh documentation](https://docs.godotengine.org/en/stable/tutorials/performance/using_multimesh.html?utm_source=chatgpt.com) | Open-source architectural comparisons; not UE performance evidence |
| [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com), [Unreal threaded rendering](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine?utm_source=chatgpt.com) | DLL ownership, ABI, and threading contracts |

**Bottom line:** For TCE, the strongest initial design is **Rust-owned building recipes → budgeted C++ change batches → cell-local Nanite ISMs → explicit shell/skyline representations**, with collision and lighting participation managed separately. The highest-value optimization is not choosing a larger instance container; it is preventing distant, inactive architectural detail from remaining millions of independent render instances.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92965-e5e0-83ea-a2e4-b33c64c61a1a)
