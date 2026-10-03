# Condition-driven weathering for TCE

## Executive recommendation

**Use a hybrid system: shared materials for widespread surface aging, sparse decals for localized history, and reusable mesh variants for structural deterioration.** Let the Rust kernel determine what has happened to a building; let Unreal translate that condition into appearance.

For TCE, I recommend:

* **Nanite ISMs with shared material instances and 8–12 floats of per-instance custom data.**
* **Authored susceptibility masks**—where water collects, finishes wear, or beams bend—combined with persistent, façade-scale procedural patterns.
* **RVTs primarily for ground contact**, not as the storage system for every building’s weathering.
* **Prebuilt damaged and rubble modules**, with Chaos reserved for occasional nearby collapse effects.

The central distinction is that **age, dirt, and structural damage are different variables**. A maintained century-old building should not resemble an abandoned ten-year-old one.

**Version baseline:** Epic released UE 5.8 on June 23, 2026. This report uses documentation available as of September 27, 2026, including pages labeled UE 5.8. The implementation still needs acceptance testing against TCE’s exact engine patch. I have not run Unreal or benchmarked these techniques on your RTX 4070 Ti; proposed budgets below are engineering targets, not measured results. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

---

## 1. Options: what each technique should do

### 1.1 Separate the appearance problem into three scales

My proposed division is:

| Scale | Examples | Best representation for TCE |
| --- | --- | --- |
| **Surface condition** | Fading, accumulated dirt, dampness, moss coverage, plaster erosion | Shared material, masks, per-instance parameters |
| **Localized history** | Chimney staining, a roof leak, fire scorching, a repaired patch, a significant crack | Sparse decals or small overlay meshes |
| **Structural condition** | Sagging roofline, missing tiles, broken beams, collapsed wall, rubble | Mesh variants, instance transforms, module removal/replacement |

This avoids trying to make one technique solve incompatible problems. A material can suggest a hairline crack; it cannot make the corresponding wall opening traversable. Conversely, replacing geometry just to darken a damp foundation creates unnecessary asset and update work.

### 1.2 Material layering: the main workhorse

UE’s **Material Layers** system organizes reusable layers and blend assets and exposes them through the Material Instance Editor. Material Functions offer another way to organize equivalent graph logic. These are authoring mechanisms, not guarantees that additional layers are cheap. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-material-layers-in-unreal-engine)

For ordinary TCE architecture, start with one opaque effective surface:

```
Base material
    → finish-loss blend into underlying material
    → dirt / biological coverage
    → wetness adjustment
    → normal and roughness detail
```

A practical family might be “lime plaster over masonry,” rather than a universal shader containing every possible construction material.

**Recommended constraint:** permit one base material and one detailed secondary material in the common path. Represent additional effects with packed masks and inexpensive parameter changes where convincing. A shader that simultaneously samples clean plaster, exposed brick, dirt, moss, soot, and wet variants everywhere is an expensive default.

Substrate is not an experimental feature in this baseline: Epic marked it production-ready in UE 5.7. Nevertheless, physically stacking multiple scattering layers is a different problem from blending spatial coverage. For TCE’s ordinary walls, I would begin with a simple Substrate surface or equivalent material-attribute blend, reserving more elaborate physical layering for materials whose appearance warrants it. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-7-is-now-available)

**Make deterioration change more than color.** My proposed recipes should modify roughness, normal detail, and the visibility of underlying material. Wetness also needs material-specific behavior rather than a universal “darken and increase specular” operation; Lagarde’s wet-surface discussion explains why porous and smoother materials respond differently. [Sébastien Lagarde](https://seblagarde.wordpress.com/2013/03/19/water-drop-3a-physically-based-wet-surfaces/)

### 1.3 Vertex and texture masks: susceptibility, not unique history

Use mesh-authored data to describe **where an effect is likely**, while per-instance data describes **how far it has progressed**.

A possible vertex-color convention is:

| Channel | Authored meaning | Runtime use |
| --- | --- | --- |
| R | Recesses and sheltered cavities | Dirt retention and biological coverage |
| G | Exposed edges or vulnerable finish | Erosion and finish loss |
| B | Runoff paths and splash-prone regions | Damp streaks and staining |
| A | Deformation susceptibility | Optional bending of selected pieces |

The important UE distinction is that **asset vertex colors are shared by every instance and work with Nanite**. Unique per-component vertex colors produced by instance mesh painting are not supported on Nanite meshes. Epic’s Paint Vertex Colors authoring tool is labeled Beta; that should not be confused with the availability of asset vertex-color attributes in Nanite materials. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/paint-vertex-colors-tool-in-unreal-engine)

For TCE, therefore:

**Bake susceptibility once per kit asset; do not paint a separate vertex-color buffer for every building.**

Vertex masks need sufficient vertex density to represent their intended variation. Use them for broad regions; use shared mask textures for fine cracks, grain, porous deposits, and irregular boundaries.

One useful procedural coverage model is:

\[
M(x)=\operatorname{smoothstep}\left(T(x)-w,\;T(x)+w,\;cE(x)\right)
\]

Here, \(c\) is condition severity, \(E(x)\) is local susceptibility, and \(T(x)\) is a stable threshold pattern. Choose the threshold range and endpoint handling so zero condition produces zero coverage.

This is a proposed visual model, not a physical weathering equation. Its advantage is that increasing condition makes existing patches expand rather than generating unrelated new noise every update.

### 1.4 Position-based masks: connect the kit to the whole building

I recommend combining three coordinate systems:

| Coordinates | Intended use |
| --- | --- |
| **Module UVs/local coordinates** | Wood grain, masonry bonds, trim sheets, local damage |
| **Continuous façade coordinates** | Streaks and macro weathering spanning multiple modules |
| **Ground-relative coordinates** | Foundation splash, mud, vegetation contact |

For example, a lower-wall damp mask could combine height above ground, a façade moisture value, and an authored runoff mask.

The engineering trap is not the formula; it is **coordinate continuity**. Independently seeded wall pieces produce visible rectangular weathering boundaries. Give adjacent modules a common façade anchor and compatible coordinates. Reserve module-specific randomness for smaller variation.

For a large, streaming world, derive shader coordinates from stable building/cell anchors. Do not assume an object-position expression automatically gives the building origin, or cast an enormous world coordinate into a small floating-point procedural field without considering precision and origin changes.

These are proposed asset contracts and implementation safeguards; they should be validated with assembled façades, not only individual mesh previews.

### 1.5 Runtime virtual textures: useful at ground level, limited as façade storage

UE’s standard RVT workflow is a GPU-generated shading cache with projection along the RVT volume’s negative Z direction. That makes it a natural fit for terrain-oriented blending, but not a general-purpose atlas for detailed history on arbitrary vertical surfaces. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine)

**Recommended TCE uses:** sample terrain color/height near foundations; blend mud around buildings; integrate rubble with the ground. Let buildings sample the terrain RVT by default. Where construction must write a footprint, consider a simple dedicated write proxy.

Two important limitations from Epic’s documentation are that RVT contents are cached, and the RVT rendering path does not provide normal CPU/GPU instance culling and LOD selection for ISMs. Avoid sending a city-wide instance component through the write pass. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine)

For a runtime-built world, plan localized refreshes after construction, demolition, or terrain changes. Do not assume editor-built low mips represent future simulation states. Sampling an RVT and writing Nanite geometry into one are separate paths; verify the latter on the selected engine patch rather than building the architecture around an assumed capability.

### 1.6 Decals: spend them on events

Projected DBuffer decals are appropriate for stains or repairs that cross kit boundaries. Their cost depends strongly on screen coverage and material complexity. UE provides receiver controls, and custom DBuffer-expression blending has overlap restrictions that need testing. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/decal-materials-in-unreal-engine)

Use decals for **specific causes**: a leaking roof joint, a chimney, a fire, an impact, or a repaired section. Do not allocate one to every possible weathering feature.

Nanite meshes can receive projected decals. That is distinct from using a Nanite mesh as the decal geometry: Epic’s Nanite documentation does not support Nanite mesh decals. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

My implementation rules would be:

Keep projection volumes tight; cull small, distant marks; prevent unintended projection onto people or neighboring surfaces; and pass event severity to the decal explicitly. A separate decal primitive should not be assumed to inherit its receiver’s per-instance condition automatically.

For an actual patch with thickness, such as a replaced board, use a small opaque mesh instead.

### 1.7 Sagging: geometry variants first, WPO selectively

World Position Offset is tempting for continuous deterioration. Nanite supports constrained WPO, but displacement bounds matter. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

The less obvious cost is shadows: under default behavior, materials using WPO can invalidate Virtual Shadow Map pages even when the displacement has not changed. Epic documents cache-invalidation overrides and recommends alternatives such as modifying instance transforms where appropriate. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

For TCE, I recommend:

**Pre-bent mesh variants for persistent sag, rigid transforms for loosened pieces, and WPO only for carefully bounded local effects.**

A sagging beam should also have a corresponding collision and support interpretation. Software-traced Lumen does not represent WPO deformation in its distance-field geometry, another reason not to base major structural damage on shader displacement. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

---

## 2. Trade-offs and practical budgets

### 2.1 Comparative assessment

The rankings below are engineering judgments, not measured UE 5.8 benchmarks.

| Technique | Main cost driver | Complexity | Recommended role |
| --- | --- | --- | --- |
| Shared parameterized material | Visible pixels, texture fetches, shader complexity | Low–medium | Default surface aging |
| Asset vertex masks | Attribute storage and authoring resolution | Low | Broad susceptibility |
| Shared texture masks | Texture residency and sampling | Low–medium | Fine irregular detail |
| Façade/world-position masks | Shader arithmetic, coordinate management | Medium | Cross-module continuity |
| RVT blending | Page generation, cache residency, refreshes | Medium–high | Ground integration |
| Projected decals | Covered pixels, overlap, primitive management | Medium | Sparse event history |
| Persistent WPO | Geometry processing, bounds, shadow invalidation | Medium–high | Restricted special cases |
| Prebuilt damaged variants | Additional assets and instance migrations | Medium | Persistent structural deterioration |
| Live Chaos destruction | Physics bodies, collision, rendering, event spikes | High | Nearby collapse spectacle |

The biggest conceptual performance mistake is **equating a visually inactive layer with an unevaluated layer**. Multiplying a sampled result by zero is not a reliable way to remove its texture work. Make common families genuinely small, use a controlled set of compile-time variants, and inspect compiled shaders.

Likewise, Nanite’s geometric simplification is not a substitute for a cheaper distant material. Epic explicitly warns that material complexity, instance counts, and other scene factors still require measurement. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

### 2.2 Texture memory: sharing matters more than a few condition floats

The following figures are **calculated full-mip-chain storage**, rounded, excluding allocation overhead and streaming behavior. They use Microsoft’s documented block sizes: BC1/BC4 use 8 bytes per 4×4 block; BC5/BC7 use 16. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression-in-direct3d-11)

| Square texture | BC1 or BC4 | BC5 or BC7 |
| --- | --- | --- |
| 1024² | 0.67 MiB | 1.33 MiB |
| 2048² | 2.67 MiB | 5.33 MiB |
| 4096² | 10.67 MiB | 21.33 MiB |

A 2048² material set containing BC7 base color, BC5 normal, and BC1 packed RGB properties is approximately **13.33 MiB**. Four independent mask channels may require a different format; do not assume a fourth channel is free.

By comparison:

| Data allocation | Calculated payload |
| --- | --- |
| 8 floats × 200,000 module instances | **6.10 MiB** |
| 12 floats × 200,000 module instances | **9.16 MiB** |
| One unique 1024² RGBA8 render target × 10,000 buildings, no mips | **39.06 GiB** |

The per-instance figures exclude CPU mirrors, instance transforms, metadata, and upload staging. The render-target figure excludes mipmaps and ping-pong buffers.

**Conclusion:** avoid one weathering render target per building. Shared textures plus small condition records are the appropriate default. A bounded pool of unique painted textures could later support a few exceptional objects.

### 2.3 Initial targets for the specified PC

These are **starting acceptance budgets**, to be revised from profiling:

| Item | Proposed target |
| --- | --- |
| Total frame time at 60 fps | 16.67 ms |
| Preferred ordinary-scene frame time | Approximately 14–15 ms, leaving headroom |
| Incremental steady-state weathering GPU cost | 0.5–1.0 ms |
| Weathering game-thread service | Approximately 0.25 ms amortized, with bounded work per frame |
| Additional shared weather textures | 64–128 MiB resident |
| Optional initial RVT physical-pool allowance | 64–128 MiB, plus other VT overhead |
| Common close-range architectural shader | Initially target roughly 6–8 texture samples |
| Whole-project VRAM planning target | Approximately 9–10 GiB observed resident usage |

CPU and GPU timings are not simply added together as if they execute serially. The objective is to avoid either side becoming the bottleneck, while retaining room for citizens, construction, streaming, shadows, and simulation.

The texture-sample target is not an engine limit. Triplanar mapping, extra material layers, and repeated sampling can exceed it quickly; use the compiled shader and actual GPU timings rather than graph-node counts.

### 2.4 What published benchmarks actually establish

Two useful—but limited—reference points are available.

Epic’s Custom Primitive Data example reports **94 draw calls with dynamic instancing disabled versus 46 enabled** in a 25-sphere demonstration. That illustrates batching behavior; it does not predict Nanite weathering cost or milliseconds on a 4070 Ti. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/storing-custom-data-in-unreal-engine-materials-per-primitive)

The 2005 γ-ton weathering paper reports approximately **one minute per iteration for 100,000 surfels and 15,000 γ-tons on a 3 GHz Pentium IV**, with 30 iterations used for its examples. This is evidence about an offline transport-based method, not a modern GPU benchmark. [Tien-Tsin Wong's Computer Graphics Page](https://ttwong12.github.io/papers/gammaton/gammaton.pdf)

I found no public apples-to-apples benchmark for the full proposed combination—UE 5.8, condition-driven Nanite architecture, RVTs, decals, and your target GPU. A controlled TCE benchmark is therefore necessary before treating the proposed frame budget as achieved.

---

## 3. Precedents and their transferable lessons

### Shipped games

| Precedent | What the primary source establishes | Lesson for TCE |
| --- | --- | --- |
| **Uncharted 4 — “The Technical Art of Uncharted 4,” 2016** | Naughty Dog’s presentation documents moss shading, wetness shading, micro-shadowing, and procedural vertex effects used to achieve the game’s appearance. It does not establish a century-scale upkeep simulation. | Reuse the visual vocabulary of specialized surface effects; do not assume one generic dirt shader provides the same result. [Advances in Real-Time Rendering](https://advances.realtimerendering.com/other/2016/naughty_dog/index.html) |
| **Remember Me / Lagarde wet-surface work, 2013** | Lagarde explicitly states that the shipped game had static weather and used only a subset of the broader proposed system, including authored wet textures for control and performance. | Distinguish a published technique from what actually shipped. Keep microscopic appearance authored; make only gameplay-relevant state dynamic. [Sébastien Lagarde](https://seblagarde.wordpress.com/2013/04/14/water-drop-3b-physically-based-wet-surfaces/) |
| **Rust — Building Upkeep, December 2017** | Facepunch replaced a door-activity-based decay exemption with resource-funded upkeep. The developer described abandoned entities accumulating and harming server performance; decay proceeded from exposed outer layers inward. | Maintenance should consume actual resources, and abandoned structures need a lifecycle that prevents endless entity accumulation. This is a gameplay-system precedent, not evidence about its shader implementation. Do not import its accelerated decay times as historical rates. [Rust](https://rust.facepunch.com/news/devblog-189) |

### Engines and available code

**Unity HDRP Layered Lit** provides inspectable shader code for mask weighting, height-based blends, and layered normals. It is useful for understanding how apparently simple layer controls expand into substantial sampling and blending work. The repository is source-available under the **Unity Companion License for Unity-dependent projects**, so treat it as an algorithm reference rather than assuming it is freely portable Unreal code. [GitHub](https://github.com/Unity-Technologies/Graphics/blob/master/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/LayeredLit/LayeredLitData.hlsl)

**Material Maker** is an MIT-licensed procedural texture-authoring and model-painting tool built with Godot. It is a suitable open-source reference or offline tool for generating masks and material variations. It is not an end-to-end runtime aging system; its relevance is moving expensive visual synthesis into asset authoring. [GitHub](https://github.com/RodZill4/material-maker)

### Research papers

**Dorsey and Hanrahan, “Modeling and Rendering of Metallic Patinas,” 1996.** Their representation uses surface layers and operators such as coating, erosion, and polishing, modulated by geometry and environment. The transferable idea is to model maintenance and weathering as different operations on material state—not as one monotonically increasing age slider. TCE does not need their complete optical model to adopt that state representation. [Stanford Graphics](https://graphics.stanford.edu/papers/patina/)

**Chen et al., “Visual Simulation of Weathering by γ-ton Tracing,” 2005.** The method models interacting weathering effects through transported particles and surface representations. Its value for TCE is causal correlation: a runoff path can explain both where material is removed and where staining appears below. I would approximate that relationship with authored runoff masks and coarse exposure state, not run the paper’s transport simulation across every building. [Tien-Tsin Wong's Computer Graphics Page](https://ttwong12.github.io/papers/gammaton/gammaton.pdf)

Together, these precedents support the components of the proposed system. They do **not** demonstrate that the complete TCE workload will meet its frame target without project-specific optimization.

---

## 4. Recommended TCE implementation

### 4.1 Make condition authoritative in Rust

I recommend storing condition at **building-zone or structural-component granularity**, rather than per texel.

A small building could have a roof, foundation, exterior façade zones, and interior zones. Large buildings can subdivide further where construction and maintenance already distinguish components.

A proposed state model is:

| State | Meaning | What changes it |
| --- | --- | --- |
| Deposition | Dirt, soot, dust | Exposure, nearby activity, washing, cleaning |
| Biological coverage | Moss or similar surface growth | Persistent moisture, shelter, material recipe, removal |
| Finish loss | Lost plaster, coating, sealant | Exposure and wear; resurfacing |
| Crack severity | Surface or structural cracking | Damage and structural processes; appropriate repair |
| Chronic dampness | Long-duration moisture burden | Drainage, roof integrity, ground contact |
| Structural integrity | Load-bearing capability | Material deterioration, loading, impacts, reinforcement |
| Recent wetness | Short-term rain response | Weather and drying, separate from chronic deterioration |

These are proposed simulation variables, not empirically calibrated conservation parameters.

Keep age and repair dates for history, but do not make the renderer reconstruct condition from age alone. An old building can contain a newly replaced roof, a repaired wall, and an original stone foundation.

For a washable surface stock, a bounded surrogate is:

\[
\frac{dx}{dt}=a(1-x)-bx
\]

where \(a\) is accumulation and \(b\) is removal, both in inverse simulation-time units. For coefficients held constant over an interval:

\[
x(t+\Delta t)=\frac{a}{a+b}
+\left(x(t)-\frac{a}{a+b}\right)e^{-(a+b)\Delta t}.
\]

Handle \(a+b=0\) separately. This is useful for stable time skipping, but should not be applied indiscriminately to cracks or structural integrity.

Maintenance should be explicit:

**Cleaning removes deposits; resealing changes future susceptibility; resurfacing replaces a finish; structural repair restores or replaces structural components.** A budget allocation alone should do nothing until labor and materials have actually been delivered.

### 4.2 Use coarse exposure, with authored local detail

My proposed exposure pipeline is:

```
Climate and local activity
    + orientation
    + roof / overhang shelter
    + drainage and ground contact
    + neighboring structures
    + material recipe
        ↓
Zone exposure values
        ↓
Slow condition evolution
        ↓
Shared material masks and selected geometry state
```

Refresh shelter or runoff relationships when buildings change, rather than tracing environmental exposure for every module every frame.

For authoring validation, use contrasting recipes: a frequently maintained stone building, a neglected damp plaster building, a dry dusty building, and a partly repaired ruin. These are test scenarios, not fixed historical stages.

The desired result is explanatory consistency: the visible stain should correspond to a source; a roof breach should change the interior’s exposure; maintenance should leave localized evidence.

### 4.3 Define a small, fixed per-instance schema

For the first implementation:

| Float | Suggested meaning |
| --- | --- |
| 0 | Dirt/deposition amount |
| 1 | Biological coverage |
| 2 | Finish loss |
| 3 | Crack severity |
| 4 | Chronic dampness or wet-response factor |
| 5 | Shelter/exposure modifier |
| 6 | Persistent pattern seed |
| 7 | Palette or construction variation |

A 12-float variant can add façade coordinate offsets, ground-height information, or another zone-specific control. Select the schema from actual shader requirements rather than passing every simulation variable.

**Custom Primitive Data and Per Instance Custom Data are not interchangeable.** On an ISM component, primitive data is shared at component scope; per-instance data distinguishes its individual placements. Epic documents both as alternatives to creating a dynamic material instance for each mesh. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Use a global Material Parameter Collection only for truly global inputs. Regional weather belongs in a spatial field or cell-level data, not a single global scalar that accidentally makes every settlement equally wet.

The UE 5.8 API exposes `SetCustomData` single-instance and range overloads with render-state notification control. Coalesce changes by component and use supported update functions rather than modifying internal arrays directly. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetCustomData)

**Allocate the data width once.** `SetNumCustomDataFloats` reallocates the full buffer and resets all its values to zero. Calling it during ordinary updates can erase weathering state. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetNumCustomDataFloats)

### 4.4 Preserve identity independently of rendering order

Use a durable mapping:

```
Stable building/module ID
    → current component
    → current instance index
```

The instance index is a location, not an identity. Repair the mapping after removals, swaps, rebuilds, and streaming.

Derive persistent shader seeds from stable IDs; do not use a transient insertion order as the building’s visual history. Keep the full ID in CPU state and provide a deliberately bounded shader seed rather than casting a 64-bit identifier into one float.

For cross-module weathering, store the same façade-level seed and coordinate frame across connected pieces. This lets a streak remain continuous after streaming or module replacement.

### 4.5 Keep the Rust–Unreal boundary narrow

I recommend an in-process, versioned C ABI carrying immutable snapshots or compact deltas:

```
Weather delta:
    stable entity ID
    revision
    zone/module selector
    fixed-layout condition values
    optional representation-change event
```

Use `#[repr(C)]` for exchanged Rust structures and explicit ownership rules. Keep Rust-owned containers, C++ objects, allocator responsibilities, and unwinding from crossing the boundary accidentally. The Rustonomicon documents the relevant layout and FFI constraints. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Let Rust workers compute condition. Apply Unreal component changes through a game-thread service. Epic’s threading guidance distinguishes game-thread-owned UObject/component state from render-thread representations; crossing that ownership casually creates races and lifetime hazards. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine)

**Update on change, not every frame.** As an initial policy, queue a visual update when a normalized condition changes by roughly 1/255, a significant event occurs, or an object becomes visible with stale render state. That threshold is a tuning choice. Coalesce repeated changes so a time skip uploads the latest state, not every intermediate day.

Offscreen buildings must continue aging logically. Only their rendering work should be deferred.

### 4.6 Group instances spatially, not by building ownership alone

For Nanite-only geometry, Epic recommends ISM rather than HISM because Nanite already has its own culling and LOD system. HISM remains worth testing for conventional static filler or fallback requirements. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

A proposed batching key is:

```
Spatial cell × mesh asset × material family × damage variant × collision policy
```

Start testing cells around **64–128 metres**, but treat that as a hypothesis, not an established optimum.

Avoid both extremes: one enormous component spanning the entire world, and a forest of tiny components created separately for every building. The best granularity must balance component management, streaming, update locality, and visibility.

For distant views, remove localized decals and use simpler representations where appropriate. Do not assume fading a costly shader feature to zero eliminates its work.

### 4.7 Ruins should be persistent simulation states

I recommend the following representation ladder:

| State | Visual representation | Simulation consequence |
| --- | --- | --- |
| Intact | Normal kit modules | Ordinary shelter and use |
| Distressed | Material changes, limited scars | Maintenance demand |
| Damaged | Missing pieces, cracked/sagged variants | Reduced shelter or support |
| Standing ruin | Partial wall and roof assemblies | Altered access and occupancy |
| Rubble | Instanced rubble clusters | Obstruction and salvage stock |
| Salvaged/reclaimed | Reduced debris, ground treatment | Recovered resources and changed land use |

Transitions should follow structural events, not a universal age threshold. Use hysteresis or explicit events so near-threshold condition does not repeatedly swap meshes.

Author compatible variants with stable pivots, sockets, and material semantics. Exposed broken interiors require real surfaces; merely hiding exterior polygons produces implausibly hollow destruction.

Chaos supports pre-fractured Geometry Collections, clustering, connection graphs, and cached destruction playback. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/destruction-overview) For TCE, I would use it only for a bounded set of nearby events. Rust determines the authoritative loss of support and final logical state; Unreal supplies the spectacle. After settling, replace transient physics with persistent instanced ruin or rubble representations.

That separation prevents an endless world from accumulating permanent physics fragments. Also bound the stored history of tiny stains and repair events: retain significant history, compact old detail into condition fields, and discard invisible transient debris without discarding its logical salvage quantity.

### 4.8 Implement in three increments

**First: surface aging.** Build two contrasting material families, a common mask convention, eight custom floats, stable façade coordinates, and the dirty-update bridge. Establish save/load and streaming consistency before adding more effects.

**Second: meaningful damage.** Add damaged modules, rubble clusters, support-driven transitions, collision/navigation updates, and a small event-decal library.

**Third: optional enrichment.** Add terrain RVT blending, more nuanced wetness, and bounded collapse effects only after the first two increments fit the scene budget.

For AI coding agents, the highest-value assignments are schema generation, asset validation, deterministic-ID tests, update-queue tests, and benchmark automation. Keep shader topology and visual acceptance criteria explicit; otherwise automated iteration can produce many attractive but incompatible material variants.

### 4.9 Acceptance testing

Build a repeatable map with **100,000, 250,000, and 500,000 module instances** as test workloads—not estimates derived from citizen counts.

Compare the same geometry and camera path under clean materials, custom-data-only materials, weather masks, additional overlays, sparse/dense decals, RVT cold/warm caches, and WPO versus pre-bent variants. Include street views, aerial views, rapid movement, accelerated time, and a construction or collapse burst.

Measure CPU/GPU p50, p95, and p99 timings, VRAM residency, update hitches, and shadow-cache invalidations. Run a packaged build; separate warmed rendering from shader/streaming cold starts. Report native 1440p and any upscaled mode separately, without counting frame generation as achieving a 60 fps simulation/rendering budget.

Finally, check the representation beyond the main camera: Lumen’s Surface Cache captures material properties separately, and view-dependent material logic can behave differently there. Inspect indirect lighting and reflections after condition changes, not only the directly lit wall. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

The essential correctness tests are persistence after reload, continuous stains across seams, localized repair effects, correct offscreen aging, and collision/navigation that match the current ruin.

---

## 5. Source guide and version applicability

These are the most useful implementation references, grouped to avoid treating every source as equivalent evidence.

| Source | Version/date and use |
| --- | --- |
| [UE 5.8 release announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available?utm_source=chatgpt.com) and [UE 5.7 announcement](https://www.unrealengine.com/news/unreal-engine-5-7-is-now-available?utm_source=chatgpt.com) | June 2026 baseline; Substrate’s production-ready status predates 5.8. |
| [Instanced Static Mesh Component](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com) and [Custom Primitive Data](https://dev.epicgames.com/documentation/en-us/unreal-engine/storing-custom-data-in-unreal-engine-materials-per-primitive?utm_source=chatgpt.com) | Current UE documentation; batching and data-scope distinctions. |
| [`SetCustomData`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetCustomData?utm_source=chatgpt.com) and [`SetNumCustomDataFloats`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent/SetNumCustomDataFloats?utm_source=chatgpt.com) | UE 5.8 API references; update and allocation behavior. |
| [Material Layers](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-material-layers-in-unreal-engine?utm_source=chatgpt.com) and [Paint Vertex Colors](https://dev.epicgames.com/documentation/en-us/unreal-engine/paint-vertex-colors-tool-in-unreal-engine?utm_source=chatgpt.com) | Current authoring workflows and asset-versus-instance color restrictions. |
| [Nanite](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?utm_source=chatgpt.com), [Decal Materials](https://dev.epicgames.com/documentation/en-us/unreal-engine/decal-materials-in-unreal-engine?utm_source=chatgpt.com), and [Virtual Shadow Maps](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine?utm_source=chatgpt.com) | Current rendering support and performance pitfalls; test exact-patch interactions. |
| [Runtime Virtual Texturing](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine?utm_source=chatgpt.com) | Projection, caching, pool behavior, and ISM write-pass limitations. |
| [Lumen Technical Details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine?utm_source=chatgpt.com), [Threaded Rendering](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine?utm_source=chatgpt.com), and [Rust FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Rendering validation and ownership/bridge design. |
| [The Technical Art of Uncharted 4](https://advances.realtimerendering.com/other/2016/naughty_dog/index.html?utm_source=chatgpt.com) | 2016 talk, with slides and PDF; shipped appearance techniques. |
| [Physically Based Wet Surfaces, 3a](https://seblagarde.wordpress.com/2013/03/19/water-drop-3a-physically-based-wet-surfaces/?utm_source=chatgpt.com) and [3b](https://seblagarde.wordpress.com/2013/04/14/water-drop-3b-physically-based-wet-surfaces/?utm_source=chatgpt.com) | 2013 technical discussion; explicitly distinguishes proposed techniques from Remember Me’s implementation. |
| [Rust Devblog 189](https://rust.facepunch.com/news/devblog-189?utm_source=chatgpt.com) | December 2017 historical upkeep design, not current balance values or realistic material lifetimes. |
| [Metallic Patinas](https://graphics.stanford.edu/papers/patina/?utm_source=chatgpt.com) and [γ-ton Tracing paper](https://ttwong12.github.io/papers/gammaton/gammaton.pdf?utm_source=chatgpt.com) | 1996 and 2005 research; state/transport concepts, not modern real-time benchmarks. |
| [Material Maker code](https://github.com/RodZill4/material-maker?utm_source=chatgpt.com) and [HDRP Layered Lit code](https://github.com/Unity-Technologies/Graphics/blob/master/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/LayeredLit/LayeredLitData.hlsl?utm_source=chatgpt.com) | Public repositories inspected in 2026. Pin commits before depending on behavior; their licenses differ. |
| [Microsoft block-compression reference](https://learn.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression-in-direct3d-11?utm_source=chatgpt.com) | Format definitions used for the memory calculations. |

**Bottom line:** author where deterioration can appear, simulate what actually happened, transmit small persistent condition changes, and change geometry only when the building’s shape or function changes. That is the most promising route to convincing aging without making weathering a second city-scale simulation inside the renderer.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92970-8f18-83ea-8ede-60d81beaae6b)
