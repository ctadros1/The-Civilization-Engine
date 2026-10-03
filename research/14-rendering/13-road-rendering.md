# Rendering runtime-generated road networks in UE5: engineering report for TCE

**Scope:** Windows, stock Unreal Engine 5.8, a Rust-owned road graph, and the hardware you specified. Sources were checked through **September 27, 2026**. Performance budgets below are proposed engineering targets—not measurements of TCE.

## Executive recommendation

For TCE, use a **hybrid renderer built around chunked procedural road surfaces and explicitly generated junctions**. Keep the Rust graph authoritative, and treat Unreal meshes, material masks, collision proxies, and distant representations as replaceable caches.

My recommended division is:

| Responsibility | Recommended approach |
| --- | --- |
| Road connectivity, traversability, construction, width and surface type | Rust simulation |
| Ordinary road surfaces and arbitrary junctions | Procedural triangle meshes, grouped into spatial chunks |
| Production mesh backend | Realtime Mesh Component Core, contingent on a packaged UE 5.8 validation test |
| Initial prototype or fallback | Native `UProceduralMeshComponent` |
| Bridges, retaining walls, selected curbs and other authored structures | Static/instanced meshes and selective spline meshes |
| Blending road edges into terrain | Road materials sampling terrain appearance; optional RVT or tiled road-coverage masks |
| Ruts, stains, repairs and other local details | Limited decals or material masks |
| Distant roads | Simplified geometry or terrain-material coverage, rather than full-detail meshes |

This is preferable to either “one spline-mesh actor per road segment” or “paint the entire network with decals.” It gives you control over arbitrary junctions, incremental rebuilding, and rendering cost without making the simulation depend on an editor tool.

Two important **UE 5.8 qualifications**:

* **Nanite spline meshes are supported**, but deforming an already-built Nanite asset is different from generating new junction topology and building Nanite data at runtime. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US)
* **Realtime Mesh Core advertises UE 5.5–5.8 support, but its runtime-Nanite documentation is inconsistent with broader marketing.** The detailed builder documentation explicitly lists stock UE 5.8 as unavailable without a supported engine fork. Do not make runtime Nanite a baseline dependency for TCE. [GitHub](https://github.com/TriAxis-Games/RealtimeMeshComponent)

---

## 1. Main implementation options

### 1.1 Spline meshes: the fastest route to a working road

`USplineMeshComponent` deforms an existing static mesh between a start and end position, using tangents, scale and roll. A longer road consists of multiple such spans. The component’s coordinates are local, and `UpdateMesh()` updates both rendering and collision state. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/USplineMeshComponent)

For TCE, an initial implementation could sample the kernel centerline, create spline spans, and assign an authored dirt-road or paved-road cross-section.

**Advantages:** little custom mesh-generation code; straightforward materials; compatibility with authored road assets; useful for bridges, retaining structures and other features whose cross-section is known.

**Limitations:** a spline span does not solve a junction. Overlapping several strips produces intersecting shoulders, doubled surfaces and inconsistent heights. A large number of independently managed spans also creates component, bounds, update and collision work. Spline deformation should not be assumed to provide ordinary ISM-style batching automatically.

**Assessment:** excellent for a prototype and selected structures; less attractive as the sole representation of an indefinitely growing, irregular network.

Also distinguish **runtime spline meshes** from **Landscape Splines**, whose documented workflow includes editor terrain sculpting and layer painting. An editor road that modifies Landscape successfully does not establish that the same authoring operation is available in a packaged game. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-splines-in-unreal-engine)

### 1.2 Procedural ribbons and junction meshes: the strongest general solution

Here, code generates positions, indices, normals, UVs and material attributes directly. Ordinary roads are swept cross-sections; intersections are separately constructed polygonal patches.

The main UE backends differ substantially:

| Backend | Strengths | Limitations and maturity |
| --- | --- | --- |
| **`UProceduralMeshComponent`** | Simple triangle-array API; useful for a small, understandable implementation; optional asynchronous collision cooking | Still marked **Experimental** in current documentation; plan your own chunking and LOD representation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/ProceduralMeshComponent/UProceduralMeshComponent) |
| **`UDynamicMeshComponent` / Geometry Framework** | Rich topology-editing representation; valuable for remeshing, booleans and geometry algorithms | Geometry Scripting remains **Beta**. Epic documents ray-tracing support but not Nanite or Lumen support for Dynamic Mesh Components. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine) |
| **Realtime Mesh Component Core** | Runtime-oriented streams, sections, LODs, collision, and C++ APIs; MIT-licensed Core | Third-party dependency requiring an exact-version test. Its advertised Core compatibility currently includes UE 5.8. [GitHub](https://github.com/TriAxis-Games/RealtimeMeshComponent) |
| **Runtime-built `UStaticMesh`** | Conventional static-mesh rendering and supplied LODs; useful after geometry becomes stable | Building the render resources is more expensive than updating a procedural section; not the editor’s complete asset-build pipeline. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UStaticMesh) |

**Recommendation:** generate ordinary arrays independently of the rendering backend. Use RMC Core for the production candidate, but keep the adapter narrow enough that native PMC remains a practical fallback.

Do not choose Dynamic Mesh merely because the world changes at runtime. Roads generally need **occasional reconstruction from authoritative parameters**, not continuous arbitrary mesh editing.

### 1.3 “Generate, then freeze” into static meshes

UE’s current `UStaticMesh` API exposes `BuildFromMeshDescriptions`, taking one mesh description per LOD. This provides an additional path: display an editable representation while a road changes, then build a conventional mesh when it settles. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UStaticMesh)

This can suit long-lived, rarely modified chunks. However, it introduces another build and resource-lifetime pipeline. For a solo developer, I would add it only after profiling demonstrates that the initial procedural backend is a bottleneck.

RMC offers a related distinction: its documented **Static** draw configuration recreates the proxy when geometry changes, whereas **Dynamic** permits in-place updates. That choice should follow actual mutation frequency—not whether the road originally came from procedural code. [TriAxis Games](https://triaxis.games/realtime-mesh/docs/component-core/structure/)

### 1.4 Decals: good surface detail, incomplete road geometry

UE’s DBuffer decals modify surface attributes such as color, normal and roughness. They do not create a road crown, curb, embankment or bridge deck. Their cost depends importantly on **screen coverage and material complexity**, not just decal count. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/decal-materials-in-unreal-engine)

For TCE, decals fit muddy wheel tracks, repairs, stains, temporary construction traces and occasional road markings. Dirt paths seen from above can also work as projected material effects.

They are weaker as the only representation of the network: intersections accumulate overlapping projections, close views expose the lack of geometry, and projections can affect unintended receivers. Configure receiver responses and projection bounds deliberately.

A useful middle ground is a **terrain-conforming mesh carrying a detail material**: more control than a box-projected decal, without rebuilding the underlying terrain.

### 1.5 Runtime virtual textures: appearance caching, not a road generator

An RVT caches material attributes over a world-space region. Epic specifically demonstrates roads and Landscape splines contributing to terrain appearance. The system is intended for relatively stable content rather than objects changing continuously every frame. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine)

RVTs can remove the obvious visual boundary between road and ground, but they do not compute junction polygons, establish graph connectivity, or automatically change collision.

Their best role is **complementary**: cache stable terrain or road appearance, while geometry supplies features that must have shape.

### 1.6 PCG and the updated UE 5.8 City Sample

This option is substantially more relevant in 2026 than older Unreal road tutorials suggest.

Epic’s updated City Sample includes a road graph that **solves intersections, assigns shape-grammar profiles, projects roads onto terrain, and partially levels the terrain**. Its assets are worth inspecting before writing every operation yourself. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine)

Separately, PCG Runtime Generation is documented to generate and clean up content around generation sources in standalone builds, with partitioning, scheduling and pooling. **PCG is not inherently editor-only.** [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-pcg-generation-modes-in-unreal-engine)

Nevertheless, the full City Sample is an authoring example, not a proven incremental renderer for TCE’s simulation. Its documented dependency chains can trigger downstream regeneration.

**Best use:** study or reuse narrowly scoped road primitives and profile assets; optionally use runtime PCG for roadside decoration. Do not replace Rust’s network with a second, independently authoritative PCG city generator.

---

## 2. Constructing reliable roads and junctions

The hardest part is not submitting triangles to Unreal. It is producing consistent geometry when roads have arbitrary angles, widths, slopes and update histories.

The following is my proposed geometry design.

### 2.1 Separate the graph from its visual tessellation

A logical road edge might be hundreds of metres long, while its visible surface requires many samples. Conversely, multiple short graph edges may belong in one rendering chunk.

Maintain three distinct representations:

| Representation | Contains |
| --- | --- |
| **Simulation graph** | Stable edge/node IDs, connectivity, permissions, capacity, construction and condition |
| **Geometric specification** | Centerline, width/profile, longitudinal grade, junction boundaries, bridge/tunnel classification |
| **Render cache** | Tessellated chunks, LODs, material masks, bounds and optional collision |

Never let visual simplification delete simulation connectivity. Never infer a junction merely because two rendered centerlines cross in plan view.

### 2.2 Generate roads as cross-sections swept along a centerline

For a ground road, sample a centerline and place lateral profile points across it: outer shoulder, road edge, crown, opposite edge and shoulder.

A useful mathematical representation is:

\[
p(s,u)=
\left(c\_x(s)+u\,n\_x(s),\;
c\_y(s)+u\,n\_y(s),\;
z(s,u)\right)
\]

Here, \(s\) is distance along the road, \(u\) is lateral offset, and \(n\) is the horizontal lateral direction.

For an informal trail, use terrain height plus a small rendering offset. For an engineered road, use a designed longitudinal grade and crossfall, with shoulders transitioning toward the surrounding ground.

**Refine adaptively.** Subdivide where centerline curvature, terrain height variation, crossfall or profile changes exceed an error tolerance. A fixed spacing alone can undersample a tight bend and oversample a straight, flat stretch.

Crucially, sample across the road’s width—not just beneath its centerline. Otherwise, a road crossing a hillside may intersect the terrain on one side and float on the other.

### 2.3 Give every junction a shared geometric boundary

Do not extend all incoming road strips to the node center and overlap them.

Instead, construct a junction region, trim each incoming road at a **portal cross-section**, and generate a patch between those portals. Adjacent road strips and the junction must use identical boundary positions and compatible normals.

The StreetGen research is a useful model: it separates ordinary road sections from intersection/transition surfaces and derives boundaries through geometric operations rather than relying on a small library of T-junction meshes. [arXiv](https://arxiv.org/pdf/1801.05741)

For TCE, the junction builder should:

1. **Collect incident approaches using explicit graph connectivity and elevation relationships.**
2. **Construct and union approach footprints**, adding corner treatment appropriate to their widths and surface type.
3. **Clean and triangulate the resulting polygon**, preserving islands or other holes.
4. **Lift it into a coherent height surface** constrained by the incoming portals.

Preauthored junction meshes remain useful for deliberately standardized infrastructure. They are a poor universal solution for arbitrary branch counts, angles, widths and slopes.

### 2.4 Use robust polygon operations, not a center-point triangle fan

A triangle fan fails for concave intersections and cannot directly represent holes.

Clipper2 is a practical C++ building block for offsetting and union operations. Its **2.0.0 documentation, dated December 2025**, also includes constrained Delaunay triangulation. The triangulator requires non-intersecting input polygons; self-intersections must first be removed, for example through a union operation. [AngusJ](https://angusj.com/clipper2/Docs/Overview.htm)

Use local coordinates for each job and a documented precision policy. Test acute angles, duplicate points, nearly collinear approaches, extremely short links, variable widths and junctions spanning chunk boundaries.

When two junctions are so close that their transition regions overlap, generate a **combined geometric junction region** while preserving the original simulation nodes.

### 2.5 Solve junction height jointly

Independently draping each incoming road onto terrain can make one approach cut through another.

CARLA explicitly documents this problem and provides junction smoothing because higher lane surfaces can obstruct lower ones where uneven approaches meet. [CARLA Simulator](https://carla.readthedocs.io/en/0.9.15/adv_opendrive/)

For TCE, fit a smooth patch constrained by the portal elevations and grades, then refine it against terrain clearance. Informal paths can follow the ground closely; paved plazas or engineered intersections may need a more controlled surface.

Treat bridges and tunnels separately. A plan-view crossing is not permission to union their footprints.

### 2.6 Keep material continuity independent of topology

Use arc-length-based UVs along ordinary strips so textures do not stretch whenever tessellation changes. Use world-projected or locally parameterized materials across junctions, with directional overlays only where necessary.

Store width, material mixture, compaction, rutting, maintenance and construction progress as parameters. TCE should not need a different topology generator merely because a dirt track gradually becomes a stone road.

---

## 3. Terrain integration and RVT pitfalls

### Two different blending workflows

**Terrain appearance into the road:** the terrain writes substrate attributes to an RVT; the visible road samples them near its edges. The road’s center retains its own surface while the shoulder blends toward nearby ground.

**Road appearance into the terrain:** footprint primitives write road attributes into a terrain-consumed RVT. This is particularly useful for flat trails and distant roads where separate surface geometry is unnecessary.

Epic’s quick start demonstrates the write/sample material arrangement and hiding writer primitives from the main pass. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/runtimevirtual-texturing-quick-start-in-unreal-engine)

For the first TCE implementation, I would start with **visible ribbons plus terrain-to-road edge blending**. This avoids making all road visibility depend on successful road-mask generation.

### Verify the mesh backend’s RVT participation

A material sampling an RVT and a component rendering **into** an RVT are different capabilities.

Do not assume that setting a procedural component’s virtual-texture array guarantees its scene proxy participates correctly. Older engine-developer guidance explicitly identified a static-draw-path restriction for Dynamic Mesh RVT support; that guidance is UE 5.1-era, so it is a reason to test current behavior, not proof of a permanent UE 5.8 limitation. [gradientspace](https://www.gradientspace.com/tutorials/2022/12/19/geometry-script-faq)

Use a packaged test that creates a road, changes it, deletes it, leaves the area, and returns. Should direct writing fail, retain the visible mesh and use either supported writer proxies or explicitly managed per-terrain-tile render targets.

### Invalidate changed regions explicitly

UE 5.8 exposes `URuntimeVirtualTextureComponent::Invalidate` and `RequestPreload`. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/URuntimeVirtualTextureComponent)

For each edit, invalidate the **union of old and new bounds**, expanded for shoulders and filtering. Invalidating only the new road location leaves stale appearance where the old road used to be.

Keep rain, global wetness and other rapidly changing effects in the final material where practical. Repainting the entire cached network every weather update defeats the purpose of caching.

### Do not treat an RVT as permanent road storage

Virtual-texture physical memory is an LRU cache: pages can be evicted and later regenerated. TCE must retain or reconstruct the data required to redraw them. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-texture-memory-pools-in-unreal-engine)

For an expanding world, use bounded regional coverage or tiled masks. Enlarging one global texture footprint continually reduces world-space detail at a fixed virtual resolution.

Other relevant documented constraints: RVT compositing uses sort priority rather than ordinary depth testing; materials filling an RVT should not sample virtual textures; and the RVT pass does not support normal CPU/GPU ISM instance-culling and LOD selection. Benchmark writer costs separately from main-view costs. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine)

### Keep earthworks and collision explicit

Painting a road does not flatten terrain, remove grass instances, or create a walkable bridge.

For early TCE milestones, favor terrain-following paths, procedural shoulders and modest embankment meshes. Drive foliage clearance from the same road footprint.

Real cut-and-fill should be a separate terrain-edit operation with coordinated geometry, collision and simulation updates. UE 5.8’s Mesh Terrain is explicitly **Experimental**; its existence is not sufficient reason to make runtime earthworks a foundational dependency. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/mesh-terrain-in-unreal-engine)

---

## 4. Scaling to thousands of segments

### Chunk by space and material, not by graph edge

Begin with **128–256 metre chunks as a tuning hypothesis**. Smaller chunks improve culling and local rebuilds; larger chunks reduce component and submission overhead.

Within each chunk, combine compatible road surfaces into a small number of material sections. Do not preserve one render section per logical road unless a measured requirement demands it.

CARLA makes this trade-off explicit: its default road-mesh portion length is **50 metres**, and its documentation warns both against giant meshes that cull poorly and tiny portions that create too many objects. That is a precedent, not an optimal size for TCE. [CARLA Simulator](https://carla.readthedocs.io/en/0.9.15/adv_opendrive/)

Keep repeated roadside props instanced. Avoid ticking an actor for every road.

### Use different representations at different scales

| Viewing scale | Suggested representation |
| --- | --- |
| **Close** | Full road profile, carefully resolved junctions, curbs where present, selected surface details |
| **Settlement view** | Simplified ribbons and junction patches; reduced cross-section detail |
| **Distant regional view** | Coarse geometry or terrain-material coverage; retain bridges and other elevated silhouettes |
| **Outside the visible/streaming region** | Kernel graph and lightweight reconstruction data only |

Choose transitions using projected size and geometric error, not distance alone. An aerial camera can expose far more road area than a street-level camera.

Preserve shared boundary vertices across chunk LODs, or explicitly stitch transitions. Otherwise, independently simplified chunks develop cracks.

Unreal’s World Partition HLOD workflow builds proxy representations from world content. Those prebuilt proxies do not automatically describe roads that only come into existence after the game starts; TCE needs its own runtime distant representation or an explicitly integrated runtime-generation solution. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition---hierarchical-level-of-detail-in-unreal-engine)

### Make updates incremental and transactional

An edit should rebuild its affected edge, adjacent junctions and intersected chunks—not the entire network.

Perform geometry calculations on immutable snapshots using worker jobs. Commit component/resource changes through the appropriate Unreal-side path. Keep the old geometry until all replacement pieces for a local transaction are ready.

Every job should carry graph and terrain revisions. Discard stale results rather than allowing an old job to overwrite newer geometry.

Do not equate “async mesh generation” with a hitch-free update: component registration, resource uploads, proxy replacement and collision changes still need measurement.

### Separate collision from visual detail

For ground-following roads, underlying terrain may already provide sufficient collision. Bridges and raised structures need dedicated collision geometry.

Do not rebuild a world navmesh merely because a road’s visual tessellation changed. Kernel navigation should continue to use the authoritative network; local Unreal collision can remain a separate concern.

Native PMC exposes asynchronous collision cooking, but that only addresses part of the update pipeline. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/ProceduralMeshComponent/UProceduralMeshComponent)

### What the available benchmarks actually establish

I did **not** find a reproducible public benchmark matching UE 5.8, an RTX 4070 Ti, arbitrary runtime junction edits, and TCE’s full scene load.

| Evidence | Reported result | Useful conclusion—and limitation |
| --- | --- | --- |
| **StreetGen, 2018 report** | Whole-Paris generation under approximately **10 minutes on one core**; a street plus neighboring context around **200 ms** | Demonstrates local reconstruction and robust city-scale geometry. These are generation timings, not rendering FPS or modern-hardware targets. [arXiv](https://arxiv.org/pdf/1801.05741) |
| **Ryan Schmidt, UE 4.26 runtime-mesh experiment** | Rebuilding a 2,046-triangle sphere every frame: PMC approximately **90–100 fps**, runtime static-mesh rebuilding approximately **30 fps**. At roughly 32,000 triangles: approximately **15 vs 3 fps** | Strong evidence that rebuilding conventional static meshes every frame is the wrong update pattern. Historical PIE measurements, not UE 5.8 road benchmarks. [gradientspace](https://www.gradientspace.com/tutorials/2020/10/23/runtime-mesh-generation-in-ue426) |
| **UE 5.8 City Sample PCG** | Complete regeneration described as taking **a few minutes**; partitioned city approximately **2,600 actors** | Useful scale reference. The sample does not include the original traffic/pedestrian simulation setup and is not evidence for frame-budgeted road mutation. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine) |

For TCE, I would initially target **under 0.5 ms additional steady-state game-thread work** and roughly **1–2 ms additional GPU time** for roads in a representative settlement view. These are allocation goals inside a **16.67 ms frame**, not a promise that the proposed implementation already achieves them.

Test 1,000, 10,000 and 50,000 generated render spans with irregular junctions, hills, aerial views, camera teleports and accelerated-time construction bursts. Record steady-state and edit-frame distributions, not just average FPS. Fix resolution, upscaling, shadows and lighting settings across comparisons.

Use Unreal Insights to distinguish game-thread, rendering and task costs; also inspect virtual-texture memory and residency when RVTs are enabled. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

---

## 5. Precedents and reusable implementation references

| Project | What it establishes | Lesson for TCE |
| --- | --- | --- |
| **CARLA 0.9.15, OpenDRIVE standalone mode** | Generates road geometry from a textual road specification at runtime. `generate_opendrive_world()` blocks until the new world is ready. | Strong graph/specification-to-mesh precedent; do not copy whole-world blocking regeneration for incremental simulation growth. [CARLA Simulator](https://carla.readthedocs.io/en/0.9.15/adv_opendrive/) |
| **UE 5.8 City Sample PCG, August 2026 update** | Curved roads, uneven terrain, shape-grammar assets and a code-assisted procedural authoring workflow | Inspect the road primitives before reinventing them. Keep TCE’s topology and update schedule independent of the sample’s full-city workflow. [unrealengine.com](https://www.unrealengine.com/learning/city-sample-gets-a-major-update-with-pcg-and-unreal-mcp-workflows) |
| **Manor Lords** | A released Unreal city-builder with gridless settlement construction and road-dependent plot subdivision | A close visual/product reference for TCE. The public sources reviewed do **not** establish its road mesh backend, RVT implementation or road-specific frame cost. [Manor Lords](https://manorlords.com/) |
| **Cities: Skylines II** | Developer documentation shows automatic intersections, road-width transitions, elevated roads and cut-and-fill placement | Useful behavioral and edge-case reference; not evidence for a particular Unreal implementation. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/road-tools) |
| **RoadNetworkTool** | MIT-licensed spline/procedural-road code; author specifies UE 5.4+ and an editor-centric workflow | Useful source study. Its documented workflow does not establish packaged UE 5.8 incremental-edit performance. [GitHub](https://github.com/sengchor/RoadNetworkTool) |
| **RoadBuilder** | Public road/junction tooling; inspected descriptor specifies version 0.5, UE 5.3, and both runtime and editor modules | More substantial than an editor-only label suggests, but still requires packaging, compatibility and licensing review before adoption. [GitHub](https://github.com/fullike/RoadBuilder/blob/master/RoadBuilder.uplugin) |
| **Realtime Mesh Component** | A reusable runtime rendering backend rather than a road-network solver | Reuse mesh infrastructure, not a second simulation graph. Keep the dependency behind a small adapter. [GitHub](https://github.com/TriAxis-Games/RealtimeMeshComponent) |

A particularly useful City Sample detail is that its default setup is **not already a streaming-ready city**: documentation says streaming is initially disabled and partitioning must be enabled. Its partition grid defaults to **128 metres**. This supports testing spatial subdivision, not assuming that any large procedural demo already solves streaming for TCE. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine)

---

## 6. Concrete architecture and implementation order

### Rust–Unreal boundary

Export **versioned road specifications and deltas**, not Unreal objects:

```
RoadGraphDelta
    graph_revision
    added / changed / removed edge IDs
    added / changed / removed junction IDs
    centerlines and elevation profiles
    widths and cross-section profile IDs
    surface, condition and construction parameters
    explicit bridge / tunnel / ground relationships
```

On the C++ side:

```
Road delta
    → dependency and dirty-region calculation
    → geometry jobs using immutable graph/terrain snapshots
    → chunk meshes + optional coverage masks + collision proxies
    → revision-checked commit
```

Keep the C ABI explicit about ownership, lengths, lifetimes and error handling. Do not pass Rust `Vec` or Unreal containers across it by assumed layout, and do not permit unwinding across an ABI boundary that does not support it. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For a solo developer, I would initially put the render-mesh builder in **ordinary C++ behind the Unreal adapter**, where Clipper and engine types are readily available. The Rust kernel should supply meaning and parameters, not gain dependencies on Unreal mesh representations.

### Implementation order

**First: prove geometry and packaged execution.** Build ground-following strips and arbitrary junctions with a plain material. Support creation, width changes, deletion, save/load and stale-job cancellation. Include terrain and chunk-boundary cases immediately.

**Second: establish scaling.** Add chunk pooling, material grouping, LODs, bounded commits and separate collision. Compare PMC and RMC on the same generated geometry, in packaged UE 5.8—not only in the editor.

**Third: add appearance caching.** Introduce terrain-edge blending and then road-to-terrain masks where they demonstrably reduce distant rendering cost. Test deletion, eviction, teleporting and cold-cache recovery.

**Fourth: add richer infrastructure.** Introduce embankments, retaining walls, bridges, engineered grades, curbs and detailed wear as separate profile capabilities.

### The highest-risk shortcuts to avoid

**Do not make the renderer secretly own the network.** Road plugins may bring connectivity or pathfinding systems; those should not compete with Rust.

**Do not use editor success as the acceptance test.** Geometry Script includes editor-only actors and functions, even though other geometry facilities are usable at runtime. Test the exact operations required in a packaged executable. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine)

**Do not require experimental rendering features for simple road surfaces.** Ordinary roads are comparatively simple geometry. Runtime Nanite adds a build pipeline and version dependency before you have established a triangle bottleneck.

**Do not confuse visible lighting with complete lighting-system participation.** Check shadows, offscreen contributions and under-bridge lighting explicitly for the chosen procedural backend; Dynamic Mesh’s documented Lumen limitation is particularly relevant. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine)

**Do not let a failed junction job corrupt the world.** Preserve the previous valid mesh, emit a diagnostic containing graph IDs and the geometric input, and produce a deterministic fallback patch or debug surface. That makes AI-agent debugging substantially more tractable.

**Bottom line:** TCE needs a **small, testable road-surface compiler**, not a general-purpose city-authoring system. Chunked procedural geometry gives it control over topology, junctions and mutation cost; selective splines, decals and terrain masks supply the appearance. The decisive milestone is a packaged UE 5.8 test that repeatedly edits a large network while the rest of the simulation continues—not a beautiful static road demo.

---

## 7. Source guide and version applicability

| Documentation, paper, talk or code | Version / applicability |
| --- | --- |
| [Epic: `USplineMeshComponent`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/USplineMeshComponent?utm_source=chatgpt.com) · [Landscape Splines](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-splines-in-unreal-engine?utm_source=chatgpt.com) | Current UE 5.8 API versus editor terrain-authoring workflow |
| [Epic: Procedural Mesh Component](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/ProceduralMeshComponent/UProceduralMeshComponent?utm_source=chatgpt.com) · [Geometry Scripting guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine?utm_source=chatgpt.com) · [`UStaticMesh`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UStaticMesh?utm_source=chatgpt.com) | UE 5.8 runtime geometry options and documented limitations |
| [Epic: Nanite](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US&utm_source=chatgpt.com) · [Decal Materials](https://dev.epicgames.com/documentation/en-us/unreal-engine/decal-materials-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 spline support and surface-detail rendering |
| [Epic: RVT overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine?utm_source=chatgpt.com) · [RVT quick start](https://dev.epicgames.com/documentation/unreal-engine/runtimevirtual-texturing-quick-start-in-unreal-engine?utm_source=chatgpt.com) · [RVT component API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/URuntimeVirtualTextureComponent?utm_source=chatgpt.com) · [Memory pools](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-texture-memory-pools-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 setup, invalidation, caching and memory behavior |
| [Epic: PCG generation modes](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-pcg-generation-modes-in-unreal-engine?utm_source=chatgpt.com) · [City Sample PCG](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?utm_source=chatgpt.com) · [August 2026 update](https://www.unrealengine.com/learning/city-sample-gets-a-major-update-with-pcg-and-unreal-mcp-workflows?utm_source=chatgpt.com) | UE 5.8; road primitives, runtime scheduling, sample limitations; includes the 2026 Unreal Fest presentation |
| [Realtime Mesh Core code](https://github.com/TriAxis-Games/RealtimeMeshComponent?utm_source=chatgpt.com) · [Component structure](https://triaxis.games/realtime-mesh/docs/component-core/structure/?utm_source=chatgpt.com) · [Runtime Nanite support tiers](https://triaxis.games/realtime-mesh/docs/rendering/nanite/?utm_source=chatgpt.com) | Current Core advertises UE 5.5–5.8; runtime Nanite has narrower documented support |
| [CARLA OpenDRIVE standalone mode](https://carla.readthedocs.io/en/0.9.15/adv_opendrive/?utm_source=chatgpt.com) | Version-pinned CARLA 0.9.15, UE4-era precedent; not a UE 5.8 compatibility claim |
| [StreetGen, 2015 paper](https://isprs-annals.copernicus.org/articles/II-3-W5/409/2015/?utm_source=chatgpt.com) · [Expanded 2018 report](https://arxiv.org/abs/1801.05741?utm_source=chatgpt.com) | Road/intersection generation algorithms and reported generation timings |
| [Clipper2 overview](https://angusj.com/clipper2/Docs/Overview.htm?utm_source=chatgpt.com) · [Triangulation API](https://angusj.com/clipper2/Docs/Units/Clipper/Functions/Triangulate.htm?utm_source=chatgpt.com) | Clipper2 2.0.0 documentation, December 2025 |
| [RoadNetworkTool code](https://github.com/sengchor/RoadNetworkTool?utm_source=chatgpt.com) · [RoadBuilder code](https://github.com/fullike/RoadBuilder?utm_source=chatgpt.com) | Author-stated UE 5.4+ for RoadNetworkTool; inspected RoadBuilder descriptor targets UE 5.3 |
| [Ryan Schmidt: runtime mesh generation](https://www.gradientspace.com/tutorials/2020/10/23/runtime-mesh-generation-in-ue426?utm_source=chatgpt.com) · [Geometry Script FAQ](https://www.gradientspace.com/tutorials/2022/12/19/geometry-script-faq?utm_source=chatgpt.com) | Historical UE 4.26 and UE 5.1 guidance; useful architectural evidence, not current benchmark guarantees |
| [Marcus Wassmer: Refactoring the Mesh Drawing Pipeline, GDC 2019](https://www.youtube.com/watch?v=qx1c190aGhs) · [Epic: Unreal Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine?utm_source=chatgpt.com) | Historical rendering architecture talk plus current UE 5.8 profiling documentation |

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9298a-39c0-83ea-9859-47de2cfa24f8)
