# Rendering procedural rivers and lakes in UE 5.8

**Engineering report for The Civilization Engine — assessed September 27, 2026**

## Recommendation

**Use Rust-owned hydrology to drive tiled runtime water geometry and a custom Single Layer Water material. Treat UE’s Water plugin as an optional integration layer, not as the authoritative representation of rivers, lakes, or floods.**

For TCE, the recommended combination is:

* **River ribbons and junction patches** for narrow, channelized water.
* **Heightfield-based surface tiles** for lakes, reservoirs, and floodplains.
* **Flow textures derived from hydrology** for currents, normal-map motion, foam, and floating debris.
* **Small, camera-local interaction effects** for splashes and ripples, added after the main renderer meets its budget.

The important distinction is that **UE’s Water plugin can render and regenerate water at runtime, but this is not equivalent to a turnkey system for importing arbitrary, changing hydrology into a packaged game**. Its current API exposes runtime regeneration and rebuild operations, while Epic still labels the plugin Experimental in UE 5.8. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Water/UWaterBodyComponent)

Single Layer Water is the useful part to retain even when replacing the plugin’s geometry pipeline: it is an engine shading model with its own water-rendering pass, rather than something inherently restricted to spline-generated Water Bodies. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/single-layer-water-shading-model-in-unreal-engine)

**Evidence boundary:** I verified documentation, public APIs, code repositories, papers, and developer presentations. I did not compile these integrations against UE 5.8 or benchmark them on a 4070 Ti. The performance allocations below are proposed TCE budgets, not measured promises.

---

## 1. What UE’s Water plugin actually supports

### Runtime capability is real, but divided across several systems

The plugin combines water-body definitions, spline metadata, surface meshing, materials, water queries, collision-related behavior, and Landscape authoring. These should not be treated as one indivisible runtime feature. Epic’s plugin index explicitly separates the `Water` and `WaterEditor` modules. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/PluginIndex/Water)

| Capability | What the current documentation establishes | Consequence for TCE |
| --- | --- | --- |
| **Runtime surface rendering and LOD** | Water Zones generate a shared, quadtree-based water surface with camera-dependent tile selection and LOD transitions. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-meshing-system-and-surface-rendering-in-unreal-engine) | Useful technology for large visible surfaces; it does not determine where hydrologically valid water exists. |
| **Runtime regeneration hooks** | `UWaterBodyComponent::IsBodyDynamic()` explicitly distinguishes saved, baked bodies from bodies regenerated at runtime. The component also exposes update and mesh-generation functions. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Water/UWaterBodyComponent) | Runtime generation is not categorically unsupported. The exact body class, initialization, and update path still need packaged-build verification. |
| **Explicit mesh/zone invalidation** | `UWaterMeshComponent` exposes `MarkWaterMeshGridDirty()` and `Update()`; `AWaterZone` exposes rebuild requests, including bounded regions. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Water/UWaterMeshComponent) | Batch changes and request necessary rebuilds rather than rebuilding every body every frame. |
| **River width, depth, and velocity metadata** | `UWaterSplineMetadata` includes depth, river width, and velocity data. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/Water/UWaterSplineMetadata) | A Rust-to-Water adapter is plausible for ordinary channel networks. These parameters are inputs, not a watershed or flood solver. |
| **Custom water bodies** | Water Body Custom uses a supplied static mesh rather than the normal Water Mesh surface and does not automatically carve terrain. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-body-actors-in-unreal-engine) | Useful for special authored shapes, but not itself an arbitrary runtime hydrology mesher. |
| **Landscape shaping** | The documented Landscape brush workflow operates through editor tools and Edit Layers. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-blueprint-brushes-in-unreal-engine) | Do not assume the familiar “river spline carves its channel” workflow is a supported packaged-game terrain-editing pipeline. |

### What this means in practice

A native Water-plugin implementation could work well when TCE generates a relatively stable network of river splines and lake outlines, then changes a limited number of parameters.

Its risk increases when the simulation must repeatedly create, remove, split, merge, reroute, or flood water bodies. Those operations require coordinating more than a visible mesh: spline metadata, component bounds, water-info resources, material parameters, and possibly collision or underwater behavior.

For a feasibility prototype, inspect the chosen classes’ dynamic-body behavior, populate spline points and metadata together, and use the current `FOnWaterBodyChangedParams`-based update APIs. Do not base production code on editor property-change callbacks or copy old examples without checking their signatures. The current component API retains older deprecated overloads alongside newer interfaces. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Water/UWaterBodyComponent)

**A necessary test is an initially empty packaged map that creates all its water during play.** A river that works after being placed, adjusted, and saved in the editor has not passed that test.

### Seasonal water levels are not just actor transforms

For TCE, treat a level change as a coherent update to **surface elevation, wet footprint, flow, rendering bounds, and gameplay state**.

Moving a visible surface upward can be part of that operation, but should not be the entire implementation. In particular, do not assume that `SetHeightOffset()` is a generic lake-level API: Epic documents it as internal and directs callers toward the ocean actor. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Water/UWaterBodyComponent)

### Water Advanced: distinguish baked rivers from live interaction

UE 5.8’s **Water Advanced** module includes both river-simulation components and runtime shallow-water subsystems. `UShallowWaterRiverComponent` exposes baked surface, normal, and foam textures; Epic also provides a dedicated river-simulation-and-baking workflow. That is relevant for precomputed river appearance, but should not be confused with continuously recomputing a changing floodplain. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/WaterAdvanced)

There is also genuine runtime interaction infrastructure: `UShallowWaterSubsystem` exposes impact registration, grid movement, render-target creation, and water-material initialization. Its API includes player/pawn-oriented selection logic. **For a free-camera civilization simulation, test that behavior explicitly rather than assuming a character-centered example will generalize automatically.** [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/WaterAdvanced/UShallowWaterSubsystem)

My recommendation is to evaluate this subsystem for nearby ripples and impacts—not to make it responsible for TCE’s long-term water distribution.

---

## 2. Options and trade-offs

The following suitability and complexity ratings are engineering judgments for TCE, not benchmark results.

| Option | How it works | Main advantage | Main drawback | TCE suitability |
| --- | --- | --- | --- | --- |
| **Native Water Bodies and Water Zones** | Convert hydrology into river splines, lake outlines, metadata, and zone rebuilds. | Existing water materials, LOD meshing, queries, and transitions. | Several coupled engine systems must remain coherent during topology changes; Experimental status. | Good prototype or bounded-world option. |
| **Custom river ribbons and lake/flood tiles using Single Layer Water** | Generate render geometry from Rust data; feed your own height, velocity, and coverage fields to a water material. | Direct control over streaming, topology, update costs, and simulation/render separation. | You implement junctions, shorelines, tile boundaries, and update scheduling. | **Best overall fit.** |
| **Reusable grid tiles displaced and masked in the shader** | Keep grid topology stable; update surface-height and wetness textures. | Most seasonal changes become texture updates instead of remeshing. | Dry-area rasterization, shoreline aliasing, displacement bounds, and disconnected basins need care. | Best lake/flood implementation within the custom approach. |
| **Conventional translucent water meshes** | Render ordinary transparent surfaces with custom reflection/refraction logic. | Flexible for special effects and unusual layering. | More compositing and sorting concerns; can become expensive over large screen areas. | Reserve for exceptional effects rather than the main surface. |
| **GPU shallow-water middleware or Niagara simulation** | Simulate a local heightfield, then render its evolving surface. | Convincing local flow and interactions. | Another solver to integrate; domain, determinism, readback, and resolution constraints. | Optional local effects, not the authoritative world model. |

The native meshing and shading capabilities above are documented by Epic; the custom-grid alternative has a useful architectural precedent in Crest’s separation of surface grids, clipping, height inputs, and flow inputs. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-meshing-system-and-surface-rendering-in-unreal-engine)

### Choosing a runtime mesh backend

**Start with `UProceduralMeshComponent` for the first working slice.** It accepts triangle data and exposes separate creation and update operations. Turn off collision generation for cosmetic water. Epic still labels this component Experimental, so isolate it behind a small TCE interface rather than spreading its API throughout the project. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/ProceduralMeshComponent/UProceduralMeshComponent)

**Evaluate Realtime Mesh Component Core for the production backend.** Its current public repository lists UE **5.5–5.8** support, MIT licensing, stream-based updates, LODs, sections, and section groups. These are useful capabilities for streamed water geometry. Pin a tested revision; the repository’s performance description is not a substitute for a TCE benchmark. [GitHub](https://github.com/TriAxis-Games/RealtimeMeshComponent)

`UDynamicMeshComponent` is appropriate when topology-processing operations are the central requirement. Epic confirms runtime topology updates, but its UE 5.8 guide still lists Nanite and Lumen as unsupported for that component. That is a renderer-integration limitation, not a reason to conclude that a custom water surface cannot appear in a Lumen-lit scene. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine)

Avoid two common mistakes:

**First, do not use the editor-only `AGeneratedDynamicMeshActor` rebuild workflow as the packaged-game generation path.** [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine)

**Second, do not repeat the blanket claim that runtime Nanite is impossible.** RMC’s separate Pro edition now advertises a runtime Nanite builder and mesh-to-SDF/Lumen support. Those are not Core features, and I would not make TCE’s comparatively simple water meshes depend on them initially. [GitHub](https://github.com/TriAxis-Games/RealtimeMeshComponent)

### Where Fluid Flux fits

Fluid Flux is worth examining as a local-effects reference or middleware candidate. Its vendor documents a heightfield shallow-water solver, advected surface effects, wetness, and asynchronous sampling. However, the published limitations distinguish a fixed, axis-aligned main simulation domain from movable character-ripple effects, warn of delayed GPU readback, and do not promise cross-GPU determinism. Exact UE 5.8 package compatibility was not established by the material retrieved. [Imaginary Blend](https://imaginaryblend.com/2021/09/)

For TCE, the integration question is not merely “Does it make attractive water?” It is **“Can its rendering accept Rust’s water state without introducing a second authority over where the water goes?”**

---

## 3. Recommended TCE architecture

### 3.1 Keep hydrology authoritative in Rust

Use a one-way presentation pipeline:

```
Rust hydrology and terrain state
             ↓
Versioned water snapshots and dirty-region events
             ↓
UE C++ water-render subsystem
             ↓
Streamed meshes + field textures + material parameters
             ↓
Single Layer Water surface + optional local effects
```

Gameplay should query Rust for water depth, current, navigability, flooding, irrigation access, and damage exposure. The GPU surface is a visualization of that state—not evidence from which the simulation reconstructs it.

This avoids a failure mode where a water shader looks flooded while citizens still regard the area as dry, or where GPU readback latency changes gameplay outcomes.

### 3.2 Define a render contract, not a collection of actor commands

A suitable proposed interface is:

| Data | Suggested representation | Purpose |
| --- | --- | --- |
| Stable water-body/reach identifier | Integer ID | Connect visual resources to simulation entities. |
| Surface elevation, \(\eta\) | Metres relative to an explicit datum or tile origin | Position the water surface. |
| Bed elevation, \(b\), or water depth | Sampled field or terrain-query reference | Shoreline placement and depth-dependent appearance. |
| Wet coverage/connectivity | Mask plus basin/reach association | Prevent rendering water in disconnected low areas. |
| Horizontal velocity, \(u,v\) | Metres per second | Flow animation and debris trajectories. |
| River cross-sections or banks | Sampled geometry | Construct narrow channels accurately. |
| Turbidity and visual state | Normalized parameters or physical quantities with documented mappings | Seasonal color, sediment, foam response. |
| Topology and field revisions | Separate counters | Distinguish remeshing from cheaper field updates. |
| Dirty bounds | Chunk-local bounding boxes | Restrict CPU work and GPU uploads. |

The distinction between **topology revision** and **field revision** is particularly valuable. A small change in discharge should not automatically trigger the same work as a river avulsion or a reservoir splitting into isolated pools.

### 3.3 River geometry: ribbons, with explicit junctions

For channelized rivers, generate a ribbon from cross-sections along the hydrological reach. Each section contains bank positions and surface elevation; additional vertices across the width are useful where the surface or flow varies.

Choose sample spacing from curvature, visible silhouette error, and elevation changes—not one universal distance. A straight, distant reach should need much less geometry than a tight bend beside a settlement.

At confluences, **build a common junction patch**. Simply overlapping two strips leaves competing surfaces, discontinuous flow coordinates, and ambiguous ownership when water levels change.

For the first implementation, use this junction policy:

1. Trim incoming and outgoing ribbons at a shared junction boundary.
2. Fill the boundary with one surface patch.
3. Sample a common elevation and velocity field over that patch.

This is a proposed TCE meshing strategy. It avoids requiring a general-purpose water-body Boolean operation every time a discharge value changes.

### 3.4 Lakes and floods: surface tiles with changing coverage

Use fixed grid topology inside active lake and flood tiles. Feed surface elevation and coverage through textures or vertex updates. Activate and deactivate tiles as water expands or retreats.

A useful rendering relationship is:

\[
h(x,y,t)=\max\!\left(0,\eta(x,y,t)-b(x,y,t)\right)
\]

but **positive depth alone is not sufficient**. A point must also belong to a hydrologically wet, connected region.

Otherwise, a renderer that applies a basin-wide level to every low location can show water behind levees, across divides, or inside disconnected depressions.

For ordinary lakes, one elevation per connected basin is a useful initial representation. For moving floodwater or sloping rivers, use a spatially varying surface field. Do not interpolate elevations across dry barriers or between unrelated basins merely because they share a texture tile.

The simplest practical arrangement is:

* Cull completely dry tiles before rendering.
* Use coverage masking near boundaries.
* Add more geometric shoreline precision only where close-up views expose the grid.

This is similar in spirit to Crest’s grid-and-input architecture: its documentation separates water-body clipping from height and flow inputs and discusses LOD-related coverage margins. It is a useful design precedent, not a drop-in Unreal solution. [Crest](https://crest.readthedocs.io/en/latest/user/water-bodies.html)

### 3.5 Make flood coverage replace—not overlap—the normal surface

When a river leaves its banks, the new flood surface should become part of a coherent local water surface.

Do not leave the original river, the lake, and a flood plane all rendering over the same pixels. Instead, assign surface ownership per patch or tile and connect the regions at matching heights.

Likewise, when a flood retreats, preserve the distinction between:

**Standing water**, which still needs a surface, and **recently wet ground**, which should normally be represented by terrain/building material changes.

That separation makes the retreat readable without keeping a nearly transparent flood plane over the entire former inundation area.

### 3.6 Flow maps: derive motion from hydrology

A flow map stores a local vector field. For TCE, produce it from the kernel’s velocity data rather than painting it by hand.

Use raw linear data—not sRGB color—and preserve velocity magnitude. A normal-map import/compression path that reconstructs or normalizes vectors can destroy the intended speed information.

For normal-map motion, use two advected samples with staggered resets. One possible formulation is:

\[
p\_0=\operatorname{fract}(t/T+\phi),\qquad
p\_1=\operatorname{fract}(t/T+\phi+0.5)
\]\[
UV\_j=UV-\mathbf{v}\_{UV}T p\_j
\]

Blend samples with complementary triangular weights so a sample fades out before its coordinates reset. Here \(\mathbf{v}\_{UV}\) is velocity converted into texture-coordinate units per second.

This follows Valve’s established two-phase flow-map approach, which limits distortion and hides resetting rather than stretching a normal map indefinitely. [Steam Static CDN](https://cdn.fastly.steamstatic.com/apps/valve/2010/siggraph2010_vlachos_waterflow.pdf)

For TCE, add several implementation safeguards:

**Coordinate consistency.** Convert Rust metres, UE coordinates, tile origins, and texture scale explicitly. A visually plausible but reversed flow vector can survive surprisingly long without a dedicated test.

**Confluence consistency.** Resample the same field across ribbon/junction boundaries.

**Separate texture animation from transport.** Resetting texture coordinates is acceptable for water detail. Persistent logs, boats, and debris need trajectories integrated through the velocity field.

**Separate speed from foam.** Slow water is not automatically foamy, and every bank should not become a white outline. Use hydrological or authored indicators of turbulence, obstacles, shallow fast flow, and foam persistence. Valve’s later presentation is useful here because it treats flow vectors and foam contribution as separate texture information. [Steam Static CDN](https://cdn.fastly.steamstatic.com/apps/valve/2011/gdc_2011_grimes_nonstandard_textures.pdf)

### 3.7 Build an independent Single Layer Water material

Create a TCE master material using **Single Layer Water**, with an **Opaque or Masked** blend mode as appropriate. Supply your own flow, coverage, depth-related appearance, and turbidity parameters.

Epic’s shading model supports absorption, scattering, reflection, and refraction through a dedicated pass. It reads scene color and depth and executes before regular translucency. Its documented implementation also includes tiled processing and screen-space reflection work. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/single-layer-water-shading-model-in-unreal-engine)

Do not assume that assigning the plugin’s default water material to arbitrary geometry will supply all its expected Water Body data automatically. Build the TCE material around a documented input contract instead.

For the initial visual model, prioritize readable differences:

| State | Proposed visual response |
| --- | --- |
| Low summer flow | Narrower coverage, exposed bars, weaker surface motion. |
| High seasonal discharge | Wider/deeper coverage, faster motion where the kernel indicates it. |
| Sediment-heavy flood | Increased scattering/absorption, reduced visibility through the water. |
| Retreating flood | Residual pools, wet banks, mud, and a slowly fading wetness field. |
| Rapids or spillways | Localized foam, stronger normal motion, limited spray. |
| Still lake | Weak current advection; wind-driven detail independent of river discharge. |

These are artistic mappings of simulated state, not claims that one shader parameter is a complete optical model of sediment.

### 3.8 Wetness, shoreline blending, and special cases

Use a persistent wetness field shared by terrain and relevant building materials. An example presentation rule after inundation ends is:

\[
W(t+\Delta t)=W(t)e^{-\Delta t/\tau}
\]

where \(\tau\) is an authored drying parameter, optionally influenced by TCE’s weather and material systems.

Use decals or runtime virtual textures only where they simplify integration. They are useful presentation mechanisms for mud, stains, and bank blending; they should not become the source of truth for water coverage.

For the MVP, defer genuinely layered water situations. One surface height per horizontal location cannot represent every waterfall, cave, or aqueduct configuration. Model waterfalls as separate falling-water geometry with local spray; treat vertically stacked channels as explicit separate layers.

Because Single Layer Water has its own compositing order, include submerged glass, translucent effects, and water-on-water views in visual acceptance tests before committing to an underwater-camera feature. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/single-layer-water-shading-model-in-unreal-engine)

### 3.9 Streaming, coordinates, and the Rust DLL boundary

Use a bounded resident set of water tiles around the camera, plus cheaper distant representations. An endless simulation does not require endless GPU residency.

Represent world locations in Rust using the project’s high-precision coordinate scheme, then render with local tile coordinates. UE’s Large World Coordinates rendering documentation specifically recommends translated/world-relative approaches for precision and performance-sensitive shader work. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/large-world-coordinates-rendering-in-unreal-engine-5)

At the DLL interface, use a narrow C ABI with explicit layouts, lengths, ownership, and versioning. Do not exchange Rust `Vec`, `String`, or Unreal container internals as though they had a shared ABI. Define which side allocates and releases each buffer, and prevent panics or incompatible unwinding from crossing the boundary. The Rustonomicon’s FFI guidance is the appropriate foundation. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

My proposed threading policy is:

**Rust publishes immutable snapshots; worker jobs prepare plain mesh/texture data; UE’s game thread manages UObject changes; rendering uploads are queued without forcing a synchronous render flush.**

Also keep display time separate from accelerated historical time. Surface texture phases should remain numerically bounded even after a long session or a large simulation-time jump.

---

## 4. Performance: what is known, and what TCE should budget

### Published measurements are useful—but not directly transferable

| Source | Reported result | What it does and does not establish |
| --- | --- | --- |
| **Valve, SIGGRAPH 2010** | Flowing water added **2 texture fetches and 21 arithmetic pixel-shader instructions** relative to two scrolling normal maps. [Steam Static CDN](https://cdn.fastly.steamstatic.com/apps/valve/2010/siggraph2010_vlachos_waterflow.pdf) | Strong evidence that convincing directional motion need not require a fluid solver. Not a UE 5.8 millisecond measurement. |
| **Kellomäki, 2014** | On an NVIDIA Quadro 1000M laptop: approximately **2 ms per simulation step at 256²**, and **5 ms at 512²**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1155/2014/580154) | Measures a particular GPU heightfield simulation, not total water rendering or modern RTX performance. |
| **Far Cry 5, GDC 2018** | The developers describe water lighting/composition in one compute pass and performance scaling with water-pixel count. [GDC Vault](https://www.gdcvault.com/play/1025033/Advanced-Graphics-Techniques-Tutorial-Water) | Supports testing screen coverage, not just river length or polygon count. The retrieved abstract does not provide a comparable timing. |
| **UE Water documentation** | Provides water/mesh statistics and scalability controls, but no reproducible TCE-like 1440p/4070 Ti benchmark. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-debugging-and-scalability-options-in-unreal-engine) | Supplies profiling mechanisms rather than a guaranteed frame rate. |

The research paper also reports limitations including grid aliasing and incomplete physical conservation. It is a useful real-time interaction reference, not a fully validated hydrological solver to transplant unchanged into TCE. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1155/2014/580154)

### Proposed starting budgets

At 60 fps, the total frame budget is **16.67 ms**. CPU and GPU work overlap, so these should be treated as separate resource budgets rather than simply added together.

| Resource | Proposed initial target | Status |
| --- | --- | --- |
| Incremental water GPU cost | **1–2 ms** in the representative gameplay view | Design target; unmeasured |
| Water-related game-thread work | **≤0.3–0.5 ms** in steady state | Design target; unmeasured |
| Water-owned resident meshes and textures | **128–256 MiB** initially | Allocation target, excluding shared frame buffers |
| Visible hydrology snapshot delivery | **2–10 Hz**, interpolated for presentation | Starting point, not solver timestep |
| Surface normal/foam animation | Every rendered frame | Presentation update |
| Mesh topology rebuilds | Event-driven and bounded per frame | Architectural requirement |

Snapshot frequency must not dictate the physics integration timestep. A local fluid solver can require multiple stable substeps between display snapshots.

### Memory example

A 512 × 512 field containing **8 bytes per cell**—for example, an `R32F` surface-height field plus `RG16F` velocity—occupies:

\[
512 \times 512 \times 8 = 2\ \text{MiB}
\]

Keeping previous and current states uses **4 MiB per tile**. Sixteen resident tiles therefore use **64 MiB**, before masks, foam history, geometry, and other resources.

This is a sizing calculation, not a prescribed tile resolution. At coarse world spacing, narrow streams still need ribbons or finer local data. Store heights relative to suitable local origins; do not casually put large absolute elevations into half-precision fields and expect stable centimetre-scale shorelines.

### The most important cost controls

**Bound pixel work.** A flood filling most of the screen can be more expensive than a much larger river network viewed from far away. Limit expensive reflection, refraction, and foam features by quality tier.

**Bound update work.** Restrict remeshing and texture uploads to dirty resident regions. Avoid recreating components or material instances for ordinary level changes.

**Bound object count.** Do not create a ticking actor for every short river segment. Group geometry by streaming chunk and material requirements.

**Avoid water collision cooking unless necessary.** Cosmetic surface meshes should not regenerate triangle collision whenever a shoreline moves.

**Keep gameplay queries out of rendering.** Epic’s water-debugging documentation specifically calls attention to water-info query costs; thousands of citizens should not each perform repeated plugin spline queries when Rust already owns the answer. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-debugging-and-scalability-options-in-unreal-engine)

---

## 5. Precedents and their lessons

### Fortnite: production water LOD, not proof of procedural flooding

Epic’s meshing documentation explicitly uses a Fortnite water-LOD transition example. The transferable lesson is the value of camera-dependent surface tessellation and smooth LOD changes. It is not evidence that every arbitrary runtime river-topology mutation is supported by the standard plugin workflow. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-meshing-system-and-surface-rendering-in-unreal-engine)

### Left 4 Dead 2 and Portal 2: separate flow appearance from fluid simulation

Valve demonstrated convincing directional water using flow maps, controlled distortion, and layered texture animation. Later material-authoring work separated flow from foam and used compact texture representations.

**Lesson for TCE:** let hydrology supply the vector field, then use a relatively inexpensive shader to communicate it. The original authoring pipeline was not a runtime watershed solver. [Steam Static CDN](https://cdn.fastly.steamstatic.com/apps/valve/2010/siggraph2010_vlachos_waterflow.pdf)

### Far Cry 5: compose specialized water representations

Ubisoft and AMD’s presentation covers different water materials for features including lakes and waterfalls, their composition, and a water-pixel-oriented lighting path.

**Lesson for TCE:** rivers, lakes, falling water, and spray need not share identical geometry or simulation techniques to appear coherent. Share the visual language and state, not necessarily one universal mesh algorithm. [GDC Vault](https://www.gdcvault.com/play/1025033/Advanced-Graphics-Techniques-Tutorial-Water)

### Crest Water 4: an inspectable grid-and-field architecture

Crest’s public MIT-licensed repository is **Crest Water 4 for Unity’s built-in renderer**; it should not be conflated with every capability of the separately advertised Crest Water 5. Its documentation offers useful examples of clipping water bodies and supplying separate height and flow information. [GitHub](https://github.com/wave-harmonic/crest)

**Lesson for TCE:** read the architecture and data organization, not just the shader. Grid reuse, local detail, clipping, and independently supplied fields are directly relevant design ideas.

### Kellomäki’s heightfield interaction research: distinguish blockers from visual interactions

The 2014 paper investigates water blocked by large bodies and combines different treatments for floating and blocking objects.

**Lesson for TCE:** a bridge pier, dam, floating log, and cosmetic splash should not all be handled by the same interaction mechanism. A dam changes authoritative flow; a small splash usually does not. The paper’s aliasing and conservation limitations also show why visually credible interaction is not automatically sufficient for long-term simulation correctness. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1155/2014/580154)

---

## 6. Implementation sequence and acceptance tests

### Stage A — Establish correctness without depending on editor state

Build a packaged test scene with one sloping river, one confluence, one lake, and one floodable basin. Generate everything from a saved Rust snapshot.

Use a simple mesh backend and one TCE water material. Confirm the coordinate convention, flow direction, shoreline placement, and correspondence between visible wetness and Rust queries.

The decisive test is a flood rising and retreating while the camera crosses chunk boundaries. Include a levee or closed barrier: water must not appear on the protected side merely because the ground is low.

### Stage B — Establish scalability

Replay exactly the same snapshots through representative camera paths. Test dry terrain, approximately quarter-screen water, mostly water, and a flooded settlement with buildings, vegetation, and citizens present.

Measure **p50, p95, and p99 frame times**, not only average fps. Separate steady rendering from topology-update and streaming spikes.

Use Unreal Insights and GPU captures; for the native Water path, Epic documents `stat water` and `stat watermesh`. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-debugging-and-scalability-options-in-unreal-engine)

Benchmark native 1440p first. When testing TSR or other upscaling, report the internal rendering resolution. Do not use frame generation to conceal an update-path stall.

### Stage C — Add local visual richness

Only after Stage B passes, add bank wetness, sediment variation, localized foam, waterfall spray, and nearby interaction ripples.

For AI-agent development, maintain a small deterministic scene suite: reversed-flow detection, tile seams, lake islands, confluences, dam overtopping, drying channels, long-distance coordinates, and rapid snapshot replacement. Pair image comparisons with numeric checks on bounds, resource counts, surface heights, and update revisions.

### Highest-risk pitfalls

| Pitfall | Preventive design |
| --- | --- |
| Works in editor, fails after cooking | Test runtime creation in packaged Development and Shipping builds from the start. |
| Native actor movement leaves inconsistent water data | Update through a version-checked adapter; verify mesh, fields, bounds, and queries together. |
| Flooding ignores connectivity | Render only kernel-authorized wet regions. |
| Every seasonal change causes a world rebuild | Separate topology, field, and appearance revisions. |
| Confluences or flood overlaps flicker | Give each local surface region one owner. |
| Narrow rivers vanish at distance | Preserve coverage deliberately and use ribbon-specific LOD. |
| Height precision creates unstable shores | Use local coordinates and suitable height formats. |
| Local GPU simulation changes civilization outcomes | Keep cosmetic interaction subordinate to Rust state. |
| Underwater effects expand scope unexpectedly | Treat underwater cameras and layered transparency as a separate feature gate. |

---

## 7. Linked sources and version applicability

These are the most actionable starting points. Epic pages were served as **UE 5.8 documentation** when checked; repository capabilities refer to their retrieved state, not a locally verified build.

| Source | Version/date | Why it matters |
| --- | --- | --- |
| [Epic: Water plugin index](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/PluginIndex/Water?utm_source=chatgpt.com) | UE 5.8 | Experimental status, modules, dependencies. |
| [Epic: Water meshing and surface rendering](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-meshing-system-and-surface-rendering-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 | Water Zones, quadtree LOD, transitions, material behavior. |
| [Epic: `UWaterBodyComponent`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Water/UWaterBodyComponent?utm_source=chatgpt.com) and [Water Zone API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Water/AWaterZone?utm_source=chatgpt.com) | UE 5.8 | Dynamic-body semantics, update functions, rebuild requests. |
| [Epic: Single Layer Water](https://dev.epicgames.com/documentation/en-us/unreal-engine/single-layer-water-shading-model-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 | Material setup and rendering-pass architecture. |
| [Epic: Water Advanced API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/WaterAdvanced?utm_source=chatgpt.com) and [baked-river tutorial](https://dev.epicgames.com/community/learning/tutorials/Y5J6/unreal-engine-baked-river-simulations-overview-and-quick-start?utm_source=chatgpt.com) | UE 5.8 API; tutorial published in 2025 | Separate live interaction infrastructure from baked river data. |
| [Epic: Procedural Mesh Component](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/ProceduralMeshComponent/UProceduralMeshComponent?utm_source=chatgpt.com) and [Geometry Scripting guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 | Runtime geometry APIs and current limitations. |
| [Realtime Mesh Component source](https://github.com/TriAxis-Games/RealtimeMeshComponent?utm_source=chatgpt.com) | Retrieved README lists UE 5.5–5.8 | MIT Core implementation, examples, LOD and update support; distinguishes Pro features. |
| [Crest Water 4 source](https://github.com/wave-harmonic/crest?utm_source=chatgpt.com) and [water-body documentation](https://crest.readthedocs.io/en/latest/user/water-bodies.html?utm_source=chatgpt.com) | Unity/Crest 4 reference | Inspectable grid, coverage, height, and flow architecture. |
| [Valve: *Water Flow in Portal 2*](https://cdn.fastly.steamstatic.com/apps/valve/2010/siggraph2010_vlachos_waterflow.pdf?utm_source=chatgpt.com) | SIGGRAPH 2010 | Two-phase flow animation and historical shader-cost measurement. |
| [Valve: *Making and Using Non-Standard Textures*](https://cdn.fastly.steamstatic.com/apps/valve/2011/gdc_2011_grimes_nonstandard_textures.pdf?utm_source=chatgpt.com) | GDC 2011 | Flow-map and foam authoring techniques. |
| [Ubisoft/AMD: *Water Rendering in Far Cry 5*](https://www.gdcvault.com/play/1025033/Advanced-Graphics-Techniques-Tutorial-Water?utm_source=chatgpt.com) | GDC 2018 | Specialized water surfaces and pixel-oriented composition. |
| [Kellomäki: *Rigid Body Interaction for Large-Scale Real-Time Water Simulation*](https://onlinelibrary.wiley.com/doi/10.1155/2014/580154?utm_source=chatgpt.com) | 2014 | Heightfield interaction method, measurements, and limitations. |
| [Fluid Flux vendor documentation and limitations](https://imaginaryblend.com/2021/09/?utm_source=chatgpt.com) | Living vendor page; exact UE 5.8 package support unverified | Middleware architecture and explicit domain/readback constraints. |
| [Epic: Water debugging](https://dev.epicgames.com/documentation/en-us/unreal-engine/water-debugging-and-scalability-options-in-unreal-engine?utm_source=chatgpt.com), [Large World Coordinates rendering](https://dev.epicgames.com/documentation/en-us/unreal-engine/large-world-coordinates-rendering-in-unreal-engine-5?utm_source=chatgpt.com), and [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | UE 5.8 / current Rust documentation | Profiling, precision, and safe DLL integration. |

## Final decision

For TCE’s combination of **changing hydrology, an endless world, a Rust authority, and a solo developer**, I would implement **custom tiled water surfaces using Single Layer Water**, beginning with a small Procedural Mesh Component implementation and adopting RMC Core where its LOD/update facilities prove useful.

Keep the native Water plugin behind an optional adapter. Reuse its systems selectively where they reduce work, but do not make TCE’s simulation depend on Water Body actor topology, editor Landscape brushes, or GPU water readback.

The two remaining validation questions are concrete: **does the chosen mesh/material path behave correctly in a packaged UE 5.8 build, and does its worst representative flood view fit TCE’s measured frame budget?** The public sources establish the necessary building blocks; they do not establish those project-specific results.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9298d-1918-83ea-af88-9133368ed797)
