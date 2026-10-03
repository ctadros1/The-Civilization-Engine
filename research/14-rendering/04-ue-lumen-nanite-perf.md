# TCE rendering report: Lumen, Nanite and VSM at 1440p/60

**Target:** Unreal Engine 5.8, Windows, RTX 4070 Ti 12 GB, 13th-generation i9, 64 GB RAM.  
**Research cutoff:** September 27, 2026.

## Executive recommendation

For TCE, I would start with **Nanite kit meshes in spatially partitioned ISM components, High-quality software Lumen using Global Tracing, Virtual Shadow Maps budgeted for a continuously moving sun, and upscaled 1440p output**. Use DLSS Quality as the first NVIDIA-specific candidate and TSR at 1080p internal resolution as the reference/fallback.

Develop **hardware Lumen with Surface Cache lighting as an A/B profile**, not an unconditional upgrade. Its better geometric representation is valuable, but a city assembled from many overlapping instances can spend significant time maintaining and traversing the ray-tracing scene. Software Lumen’s global-distance-field tracing is relatively insensitive to overlap during tracing, although updating that field still costs time. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US)

Two version changes matter:

* **UE 5.8 adds Lumen Lite:** Medium GI is no longer necessarily “Lumen off.” It provides a lower-cost irradiance-field path.
* **MegaLights is production-ready in 5.8:** it deserves a separate nighttime-lighting test, rather than being dismissed as experimental. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

**The architectural priority is bounded rendering work, not a maximum building count.** TCE should simulate the entire population while maintaining camera-dependent subsets of geometry, detailed citizens, shadow casters and lighting representations.

I found no primary-source benchmark matching **UE 5.8 + RTX 4070 Ti + a runtime-built TCE-scale city**. Consequently, the budgets below are proposed acceptance targets—not measured performance or guarantees.

---

## 1. Rendering options and their trade-offs

### Lumen: which tracing path?

Lumen combines screen-space information with an off-screen scene representation. Software tracing uses distance fields; hardware tracing uses a triangle-based ray-tracing scene. Separately, Lumen usually caches lighting on surfaces rather than evaluating full materials and lighting at every ray hit. **Hardware tracing and Hit Lighting are different choices.** [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

| Configuration | Strengths | Principal costs and limitations | TCE assessment |
| --- | --- | --- | --- |
| **Software Lumen, Global Tracing** | Fast merged-distance-field tracing; attractive for overlapping modular construction | Coarse geometry representation; distance-field updates; less faithful reflections | **Baseline** |
| **Software Lumen, Detail Tracing** | Better near-field geometric detail | Traces individual mesh distance fields near the ray origin; overlap can become expensive | Targeted quality experiment, not city-wide default |
| **Hardware Lumen, Surface Cache** | More faithful intersections; supports geometry that software distance fields cannot represent | Ray-tracing instance gathering, acceleration structures, traversal and memory | **Main quality/performance challenger** |
| **Hardware Lumen, Hit Lighting** | More accurate reflected materials and lighting | Additional material evaluation and lighting work at hits | Photo mode or exceptional scenes |
| **Lumen Lite / Medium GI** | Lower-cost dynamic indirect lighting | Reduced detail and quality; different final gather | **Performance tier** |
| **High GI with Medium reflections** | Retains stronger diffuse lighting while reducing reflection expense | Smooth surfaces rely on SSR and approximate rough specular; off-screen reflection limitations | Useful intermediate fallback |

These distinctions follow Epic’s current settings, technical and scalability documentation. Medium reflection quality disables dedicated Lumen reflections and uses SSR for smooth surfaces. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-global-illumination-and-reflections-in-unreal-engine)

### Why hardware ray tracing is not automatically the right default

Instancing shares mesh resources, but it does **not** collapse an entire city into one ray-tracing instance. Epic documents per-frame top-level acceleration-structure work on the rendering thread, RHI thread and GPU, largely proportional to included instances. Deforming geometry can also require bottom-level structure updates. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/ray-tracing-performance-guide-in-unreal-engine)

For perspective, an illustrative city with 20,000 buildings and 80 modules per building contains **1.6 million module placements**. That arithmetic is not a rendering limit; it shows why “world instances,” “resident instances,” “visible instances” and “ray-tracing instances” must be separate counters.

My proposed decision rule is:

> Select hardware Lumen only when its complete frame cost—including scene maintenance—and its peak memory fit the budget in construction, crowd and camera-motion tests.

Do not compare only the `LumenReflections` pass while ignoring the additional scene-building work.

For a dedicated hardware profile, UE 5.8’s **Inline ray-tracing mode** is particularly relevant: Epic documents support for Lumen HWRT with Surface Cache while avoiding material-ray-tracing shader compilation and binding costs associated with Full mode. Check compatibility with every other enabled ray-traced feature before choosing it. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/ray-tracing-performance-guide-in-unreal-engine)

---

## 2. Recommended Lumen and upscaling configuration

### Starting configuration

This is my proposed first test profile, not a claim that these settings alone achieve 60 fps.

| Area | Starting choice |
| --- | --- |
| Renderer | Deferred rendering, DirectX 12, Shader Model 6 |
| Geometry | Nanite-enabled, pre-cooked modular static meshes |
| Dynamic GI / reflection methods | Lumen / Lumen |
| GI scalability | High: `sg.GlobalIlluminationQuality=2` |
| Reflection scalability | High: `sg.ReflectionQuality=2` |
| Software tracing mode | **Global Tracing** |
| Lumen hardware tracing | Off in baseline; separate Surface Cache comparison |
| Shadows | Virtual Shadow Maps, High scalability |
| Post-process Lumen overrides | Leave at normal defaults initially |
| High-quality translucent reflections | Off initially |
| Output | 2560×1440 |
| Upscaler | DLSS Quality candidate; TSR reference |
| TSR reference input | 1920×1080: 75% linear screen percentage |
| Frame generation | Off for acceptance testing |
| Static lighting | Disable for this fully dynamic lighting pipeline |

The renderer and geometry requirements are supported by Nanite’s current platform documentation; Lumen’s project settings expose the tracing and reflection choices above. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US)

### Control lighting reach rather than maximizing everything

Do not increase Lumen Scene Detail, view distance, tracing distance and final-gather quality together to fix one artifact. Those controls affect different workloads. First determine whether the problem is missing geometry, inadequate surface coverage, temporal convergence or insufficient sampling. Epic exposes separate controls for these cases. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-global-illumination-and-reflections-in-unreal-engine)

For TCE, I would establish two camera modes:

**Street-level mode:** maintain detailed nearby buildings and lighting, with limited detailed interiors.

**City-overview mode:** use coarser building representations and deliberately test the required lighting range. Do not retain street-level geometric detail for every district simply because the camera can see the skyline.

Software Lumen’s documented scene coverage is approximately 200 metres by default, extendable to 800 metres. Hardware Far Field can extend farther, but the documented path requires **built World Partition HLOD1 content**. That is not an automatic solution for an endlessly changing runtime city. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

### Reflections are a major scalability lever

My preferred degradation sequence is:

**High GI + High reflections → High GI + Medium reflections → Lumen Lite + Medium reflections.**

This lets TCE preserve daylight bounce and indoor/outdoor contrast before reducing the whole GI solution.

Wet streets, polished floors and broad water surfaces should be explicit stress cases. Dedicated Lumen reflection rays are concentrated on smoother materials; Epic exposes roughness thresholds and reflection downsampling to control their cost. Test these controls against representative materials rather than indiscriminately making the entire city rougher. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US)

### DLSS versus TSR

NVIDIA’s current download page lists a **DLSS 4.5 plugin for UE 5.8**, updated September 2026. Pin the plugin package and model/preset configuration in performance records; changing the upscaler is a renderer change, not merely a cosmetic preference. [NVIDIA Developer](https://developer.nvidia.com/rtx/dlss/get-started)

For fair testing, make two comparisons:

1. **Equal internal resolution**, to compare reconstruction quality and upscaler overhead.
2. **Each product’s intended quality mode**, to compare practical user-facing presets.

At 1440p output, 75% linear resolution is 1080p input and contains **56.25% of native pixel count**. That does not imply a 43.75% frame-time saving: scene updates, geometry processing and some output-resolution work remain.

TSR also depends on good motion vectors and temporal history. Test moving citizens, foliage, thin fences, roof edges, newly revealed surfaces and scrolling camera views. A still screenshot will miss the most important reconstruction failures. Epic’s TSR documentation explains the distinction between input-resolution rendering and later output-resolution work. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/temporal-super-resolution-in-unreal-engine)

**Do not use generated frames to certify the 60 fps requirement.** Define acceptance as 60 newly rendered frames per second with the full simulation active.

---

## 3. Nanite: optimize modules and materials, not just triangles

### Runtime assembly should reuse preprocessed assets

The safest content contract is:

> Author and cook the kit once; place, recolor, combine and remove its instances at runtime.

Nanite builds its streaming representation from mesh assets. Mesh distance fields are also generated offline; Epic explicitly states that their generation is not a runtime operation. Thus, placing a cooked wall module is materially different from generating arbitrary new wall topology in a packaged build. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US)

For TCE, arbitrary runtime topology changes should be a separate rendering spike—not an assumed extension of the modular-instancing pipeline.

A kit asset should pass four independent checks:

| Representation | Acceptance check |
| --- | --- |
| Primary Nanite geometry | Appropriate silhouette, geometry density and overdraw |
| Lumen representation | Usable distance field and Surface Cache coverage |
| Shadow representation | Correct silhouette without excessive deformation or bounds |
| Optional hardware-RT representation | Acceptable proxy/fallback geometry and memory |

Lumen’s Surface Cache uses mesh capture data; problematic coverage is visible in its debug views. Complex combined interiors can require mesh splitting rather than simply increasing global quality. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

### Choose module granularity deliberately

My recommendation is **wall bays, roof sections, structural spans and selected decorative modules**.

Avoid both extremes:

* Individual instances for every brick, tile, board and hidden fitting.
* One enormous building mesh containing every room, facade and furnishing.

The first creates excessive scene-management work. The second gives TCE less flexibility for visibility, construction updates, representation changes and lighting coverage.

For a Nanite-only path, prefer ISM rather than reflexively choosing HISM. Epic’s current guidance explicitly favors ISM when Nanite supplies its own culling and LOD system. HISM remains relevant where conventional fallback meshes and largely static hierarchies matter. Per-instance custom data also lets repeated instances share materials while varying appearance. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

### Nanite does not eliminate overdraw

Closely stacked surfaces and aggregate geometry—leaves, fences, thatch, layered roof detail—can defeat efficient occlusion and simplification. Nanite’s cost is not determined by source triangle count alone. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/working-with-naniteenabled-content)

For TCE assets, I would enforce these rules:

**Remove unnecessary hidden layers.** A facade assembly should not accumulate several nearly coincident walls as styles evolve.

**Keep material families small.** Prefer shared trim sheets, tileable textures and per-instance parameters over a unique material instance and texture set for each building.

**Keep ordinary buildings on simple material paths.** Reserve World Position Offset, Pixel Depth Offset, expensive layered shading and procedural effects for modules that visibly need them.

**Treat transparency separately.** Glass, smoke and other translucent effects need their own budget rather than being counted as “handled by Nanite.” Nanite’s supported material paths do not make general translucency free. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US)

Fortnite provides a particularly useful production lesson: its team found that opaque geometric foliage often outperformed masked cards, and explicitly disabled building WPO when the damage animation was inactive. These are content-specific findings, but excellent hypotheses to test for TCE’s thatch, shutters, plants and construction effects. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/bringing-nanite-to-fortnite-battle-royale-in-chapter-4)

Use Nanite’s **Overdraw, Raster Bins, Shading Bins and Evaluate WPO** visualizations during kit approval. A beautiful asset that performs badly in these views should be corrected before being replicated across thousands of buildings. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details)

---

## 4. Virtual Shadow Maps: design around the moving sun

### The central problem is cache invalidation

A continuously changing sun direction prevents normal reuse of its cached shadow pages. Fortnite encountered exactly this combination—moving sun plus animated geometry—and abandoned attempts to make caching solve the worst cases. For its 60 fps target, Epic instead reduced effective sun-shadow resolution to approximately half that of earlier demos and optimized uncached rendering. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/virtual-shadow-maps-in-fortnite-battle-royale-chapter-4)

**TCE should budget a moving sun from the beginning.** A benchmark with a frozen sun is useful as a diagnostic control, but it is not the shipping workload.

Separate static caching can reduce costs associated with moving objects under a stationary light. It does not preserve a rotating sun’s cache. Geometry bounds also matter: oversized bounds invalidate more pages than necessary. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

### Recommended tuning order

| Order | Change to test | Reason |
| --- | --- | --- |
| 1 | Make major environment shadow casters Nanite | Improve the geometry path used for shadow rendering |
| 2 | Start from High shadows; increase moving-sun LOD bias in small steps | Reduce shadow-page resolution before making broader visual sacrifices |
| 3 | Remove shadow casting from negligible distant details | Spend shadow work on readable silhouettes |
| 4 | Disable distant or inactive WPO | Avoid unnecessary deformation work and invalidation |
| 5 | Adjust shadow filtering samples | Reduce projection/filtering cost after understanding depth cost |
| 6 | Investigate coarse pages and local-light coverage | Catch costs not obvious from the primary camera view |

Nanite shadow casters were central to Fortnite’s uncached-shadow optimization. Epic’s current VSM controls include `r.Shadow.Virtual.ResolutionLodBiasDirectionalMoving`; coarse pages support effects such as fog and translucency, so disabling them requires visual validation. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/virtual-shadow-maps-in-fortnite-battle-royale-chapter-4)

For filtering, distinguish **shadow-depth rendering** from **shadow projection**. Reducing SMRT sampling does not eliminate the cost of redrawing invalidated geometry. AMD’s Unreal performance guide discusses these as separate optimization opportunities. [AMD GPUOpen](https://gpuopen.com/learn/unreal-engine-performance-guide/)

### Quantizing sun movement is a trade-off, not a cure

Holding the sun direction between updates may improve average cache reuse, but creates shadow jumps and potentially heavier update frames. Interpolating the same shadow-casting light continuously between those steps defeats the intended cache reuse.

The geometry illustrates why a fixed update frequency is insufficient. For a vertical object of height \(h\), shadow length on level ground is:

\[
L=h\cot(\alpha)
\]

For a small elevation change:

\[
|\Delta L|\approx h\,\csc^2(\alpha)\,|\Delta\alpha|
\]

A 20-metre building at 15° solar elevation experiences approximately **26 centimetres of shadow-length change for a 0.05° elevation step**. This is a geometric calculation, not an engine benchmark.

I would therefore retain smooth sun motion in the baseline, then evaluate stepped shadows only as an explicit lower-quality option. Include dawn and dusk in testing.

### Night lighting and MegaLights

Do not create a shadow-casting point light for every occupied window. My proposed hierarchy is:

**Most windows:** material-driven room brightness and emission.  
**Nearby important openings:** a limited number of actual lights.  
**Street lamps and important interiors:** carefully bounded lights with a measured shadow budget.

Lumen can propagate emissive lighting, but small, very bright emitters can produce noise. Therefore, emissive windows should not be treated as reliable replacements for every important architectural light. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-global-illumination-and-reflections-in-unreal-engine)

For dense nighttime scenes, test **MegaLights with hardware Lumen** as a combined profile. Epic notes that they can share ray-tracing scene overhead. MegaLights trades bounded sampling work against noise and reconstruction quality; excessive overlapping light influence still matters. Although directional lights are supported behind an explicit setting in 5.8, Epic recommends deferred lighting plus VSM for strong sunlight. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/megalights-in-unreal-engine)

This suggests a useful separation: **VSM sun, selectively deployed MegaLights local illumination**.

---

## 5. TCE-specific architecture and budgets

### Runtime integration

I recommend a narrow ownership boundary:

**Rust simulation → immutable render snapshot/delta buffers → C++ rendering bridge → UE scene components.**

The Rust world should not be structured around Unreal Actors. My proposed bridge contract is:

* Rust owns simulation identities, state and deterministic evolution.
* C++ owns asset references, components and engine-facing scene mutation.
* Updates carry stable IDs, transforms and compact appearance data.
* The renderer applies bounded batches rather than replacing complete district instance arrays.
* Simulation and rendering have independent update rates, with visual interpolation where appropriate.

Use component groups such as:

```
spatial cell × mesh asset × material family × mobility/shadow policy
```

Prototype several cell sizes—for example, **64, 128 and 256 metres**—and select them from measured streaming, component and update costs. Those values are test points, not documented engine optima.

Keep completed buildings separate from animated construction pieces, doors and other movers. Avoid repeatedly touching unchanged transforms or render state. Maintain a stable-ID-to-instance mapping instead of treating an instance-array index as a permanent simulation identity.

The same separation should apply to citizens: simulate 50,000 people without requiring 50,000 equally detailed rendered characters. UE 5.8’s experimental MetaHuman crowd workflow itself demonstrates the architectural pattern of switching between individual Actors and instanced skinned representations with distance; it is a reference, not evidence that TCE’s crowd budget is solved. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

### Runtime city streaming and HLOD

Do not assume normal World Partition HLOD generation will continuously rebuild representations for procedurally constructed districts. Epic’s documented HLOD workflow includes a build process. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition---hierarchical-level-of-detail-in-unreal-engine)

For the first TCE implementation, I would own a straightforward runtime residency system:

**Near:** detailed modules and citizens.  
**Middle:** simplified building assemblies and reduced citizen representations.  
**Far:** coarse building/parcel representations assembled from pre-cooked assets.

Do not make the initial implementation depend on runtime mesh merging, runtime HLOD baking or experimental geometry-streaming features. Epic’s Fast Geometry Streaming work targets immutable static content, which is not equivalent to TCE’s construction-and-demolition workload. [Unreal Engine](https://www.unrealengine.com/en-US/news/unreal-engine-5-6-is-now-available)

### Frame-time targets

At 60 fps, the total frame interval is **16.67 ms**. I would use the following provisional acceptance envelope:

| Measurement | Proposed target | Interpretation |
| --- | --- | --- |
| Typical full GPU frame | **13.5–14 ms or less** | Leave room for unfavorable views and updates |
| GPU frame, stress-route p99 | **16.67 ms or less** | Tail performance matters more than average fps |
| Lumen GI + reflections | **Approximately 3–4 ms** | Initial allocation, not measured 4070 Ti performance |
| Shadows, all relevant work | **Approximately 2–3 ms** | Include moving sun and local lights |
| Game/render CPU critical work | **Preferably below 6–8 ms each** | Prevent CPU-side submission becoming the bottleneck |
| Routine bridge application work | **Initially budget 0.5–1 ms** | Batch or defer excess visual updates |

These are **nested planning targets**, not pass timings to add mechanically. GPU async overlap and CPU/GPU pipelining mean that the frame’s critical path is the final authority.

Construction is a burst workload. If a simulation event changes thousands of modules, let the renderer converge through a bounded queue while preserving immediate, important visual feedback. Do not promise unlimited same-frame visual reconstruction.

### VRAM targets

Use a provisional **9.5 GiB working envelope**, then adjust against measured Windows memory budgets:

| GPU resource category | Initial planning allowance |
| --- | --- |
| Textures, including non-streaming and virtual-texture allocations | 3.50 GiB |
| Nanite geometry streaming and associated allocations | 1.00 GiB |
| Other geometry, crowds, instance data and optional RT geometry | 1.25 GiB |
| Lumen distance fields, surface/radiance caches | 0.75 GiB |
| VSM resources | 0.50 GiB |
| Render targets, reconstruction histories and transient peak | 1.75 GiB |
| Other renderer/driver allocations and internal contingency | 0.75 GiB |
| **Total planning allowance** | **9.50 GiB** |

These are allocation targets, not known UE defaults.

Windows exposes both current GPU-memory usage and a process budget. Exceeding the assigned budget can cause stuttering, so the physical “12 GB” label is not a sufficient acceptance test. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/ns-dxgi1_4-dxgi_query_video_memory_info)

A reasonable first texture-streaming experiment is:

```
r.Streaming.PoolSize=3072
```

That controls a particular engine pool, **not total VRAM**. Never interpret “texture pool within budget” as “GPU memory within budget,” and do not disable texture streaming to eliminate streaming warnings. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/texture-streaming-configuration-in-unreal-engine)

For hardware profiles, inspect UE 5.8’s ray-tracing proxies and reference-based geometry residency. Avoid raising Nanite fallback fidelity everywhere without measuring its memory consequences. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/ray-tracing-performance-guide-in-unreal-engine)

For VSM, exceeding the physical-page pool can produce corrupt or missing shadows. Treat overflow as a failed test, not an acceptable quality reduction; the documented pool control is `r.Shadow.Virtual.MaxPhysicalPages`. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

---

## 6. Profiling and validation plan

### Build a reproducible performance harness

For a solo developer working with coding agents, the highest-value tooling is a small automated benchmark harness—not a collection of untracked console-variable changes.

Record a deterministic city seed, camera path, time-of-day sequence and render-update stream. Test a packaged build, preserve the configuration, and distinguish cold loading from warm steady state. AMD’s guide emphasizes repeatable workloads and avoiding editor overhead when evaluating performance. [AMD GPUOpen](https://gpuopen.com/learn/unreal-engine-performance-guide/)

Run two complementary modes:

**Renderer replay:** feed previously recorded render updates without the full simulation, isolating rendering changes.

**Complete TCE:** run Rust simulation, streaming, citizen updates, lighting and rendering together. This is the acceptance test.

Initially disable frame generation, dynamic-resolution adaptation and frame caps that conceal available performance. Afterwards, test the actual user-facing frame-pacing configuration.

### Instrument Unreal and the Rust boundary

A useful starting trace command line is:

```
TCE.exe -trace=cpu,gpu,frame,bookmark,counters -statnamedevents
```

Add task, loading or memory channels for focused investigations rather than tracing everything continuously. Epic documents these channels and their prerequisites. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/unreal-insights-reference-in-unreal-engine-5)

Instrument the bridge with named scopes and counters for simulation work, synchronization waits, delta production, delta application, component creation, instance changes and queue depth.

A scope around a Rust DLL call measures the call’s duration; it does not automatically explain which Rust subsystem consumed that time. Add corresponding Rust-side measurements and explicit allocation accounting.

| Tool/view | Main question |
| --- | --- |
| `stat unit` | Is the limiting stage game, render or GPU? |
| Unreal Insights CPU/GPU tracks | Where are waits, bursts and critical-path dependencies? |
| `stat gpu` / `ProfileGPU` | Which rendering features dominate? |
| `stat SceneRendering` | What is the instance/scene-management workload? |
| `stat D3D12RayTracing` | What does the hardware-RT path add? |
| Nanite diagnostic views | Are materials, overdraw or deformation the issue? |
| Lumen scene/cache views | Is geometry missing or poorly represented? |
| Render Resource Viewer | Which assets and render resources own memory? |

These tools are documented across Insights, the ray-tracing guide, Nanite technical details and Render Resource Viewer documentation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

Async compute complicates attribution: disabling it can help isolate individual passes, but final comparisons must restore the intended shipping configuration. Do not add overlapping GPU durations and call the result frame time. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/megalights-in-unreal-engine)

### Required stress routes

| Test | Failure being sought |
| --- | --- |
| Dense city, frozen noon sun | Reference cost with favorable shadow caching |
| Same route, continuously moving sun | Uncached directional-shadow cost |
| Dawn/dusk through narrow streets | Long shadows and difficult coverage |
| Crowded market at street level | Animation, materials, shadows and CPU submission |
| Rainy night with lights and wet roads | Reflections, local lighting and temporal noise |
| Rapid street-to-overview camera movement | Streaming, lighting convergence and visibility churn |
| Batches of 100 and 1,000 module additions/removals | Scene-update bursts; numbers are test inputs, not throughput promises |
| Extended travel and repeated district replacement | Memory growth, stale resources and fragmentation |

Record p50/p95/p99 frame times, over-budget frame counts, peak memory, resident/visible/RT instance counts, shadow-page behavior and update-queue depth.

Also test **cold PSO behavior**. UE’s PSO precaching can defer rendering while required pipelines are unavailable; the first appearance of an architectural style must not be the first time its critical material permutations are encountered. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/pso-precaching-for-unreal-engine)

### Decision gates

My recommended gates are:

**Software versus hardware Lumen:** choose on complete p95/p99 frame cost, image quality and peak memory—not isolated ray-dispatch time.

**MegaLights:** require a win on the night route without unacceptable noise, ghosting or additional scene-maintenance cost.

**Kit admission:** reject assets that create disproportionate shading, WPO, overdraw or memory costs when repeated.

**60 fps acceptance:** require the full simulation, moving sun, representative crowds and runtime construction—not an empty-city flythrough.

---

## 7. Precedents and what their numbers establish

| Precedent | Published result or scope | Applicable lesson |
| --- | --- | --- |
| **Fortnite Chapter 4 / UE 5.1-era implementation** | Targeted 60 fps on consoles with **4 ms for GI and reflections**, selecting software Lumen | Dynamic construction, destruction and time of day are compatible with Lumen when aggressively budgeted. This is not a 4070 Ti benchmark. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/lumen-brings-real-time-global-illumination-to-fortnite-battle-royale-chapter-4) |
| **Current Lumen scalability guidance** | High targets **4 ms**, Epic **8 ms**, at 1080p internal resolution on consoles | High—not Epic—is the relevant starting quality tier for 60 fps. These are feature budgets, not whole-game guarantees. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US) |
| **UE 5.8 Lumen Lite** | Epic describes it as approximately **twice as fast as High-quality Lumen** | A useful lower tier; do not translate this into twice the game frame rate. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available) |
| **The Matrix Awakens / SIGGRAPH 2022** | Ray-tracing scene construction for a world containing **over 1.5 million instances**; the demo had a 30 fps budget | Large world totals require selective scene representation. They do not demonstrate that all instances can remain fully active at 60 fps. [Advances in Real-Time Rendering](https://advances.realtimerendering.com/s2022/index.html) |
| **Frostbite GIBS / SIGGRAPH 2021** | Dynamic surfel-based GI with no precomputation, designed around cached lighting and bounded representation | Independent confirmation of the architectural value of spatial/temporal caching. Not a drop-in UE renderer. [Advances in Real-Time Rendering](https://advances.realtimerendering.com/s2021/index.html) |
| **AMD Brixelizer / Brixelizer GI** | MIT-licensed implementation of runtime sparse distance fields and compute-based dynamic GI | Useful inspectable code for incremental spatial representations. Replacing Lumen with it would be a substantial integration project. [AMD GPUOpen](https://gpuopen.com/fidelityfx-brixelizer/) |
| **ReSTIR, SIGGRAPH 2020** | Spatial and temporal reuse of light samples for scenes with many dynamic emitters | Relevant research for stochastic many-light rendering; its experimental results are not UE MegaLights or TCE benchmarks. [NVIDIA](https://research.nvidia.com/publication/2020-07_spatiotemporal-reservoir-resampling-real-time-ray-tracing-dynamic-direct) |

The strongest production precedent for TCE is **Fortnite’s combination of runtime building changes, moving sunlight and optimized Nanite content**. Matrix-scale demonstrations are more useful for understanding representation and culling than for predicting TCE’s frame rate.

---

## 8. Source map and version applicability

| Source | Version/date and use |
| --- | --- |
| [UE 5.8 release overview](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available?utm_source=chatgpt.com) | 2026: Lumen Lite, MegaLights production status and experimental feature boundaries |
| [Lumen performance guide](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US&utm_source=chatgpt.com) · [Technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine?utm_source=chatgpt.com) | Current 5.8 documentation: scalability, tracing, caches and range |
| [Lumen settings](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-global-illumination-and-reflections-in-unreal-engine?utm_source=chatgpt.com) · [Ray-tracing performance](https://dev.epicgames.com/documentation/en-us/unreal-engine/ray-tracing-performance-guide-in-unreal-engine?utm_source=chatgpt.com) | Current 5.8 documentation: quality controls, scene costs, inline mode and residency |
| [Nanite overview](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US&utm_source=chatgpt.com) · [Working with Nanite content](https://dev.epicgames.com/documentation/unreal-engine/working-with-naniteenabled-content?utm_source=chatgpt.com) | Current 5.8 documentation: supported paths and content pitfalls |
| [Virtual Shadow Maps](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine?utm_source=chatgpt.com) · [MegaLights](https://dev.epicgames.com/documentation/en-us/unreal-engine/megalights-in-unreal-engine?utm_source=chatgpt.com) | Current 5.8 documentation: moving lights, cache behavior, shadow controls and limitations |
| [Fortnite Lumen](https://www.unrealengine.com/tech-blog/lumen-brings-real-time-global-illumination-to-fortnite-battle-royale-chapter-4?lang=en-US) · [Fortnite VSM](https://www.unrealengine.com/tech-blog/virtual-shadow-maps-in-fortnite-battle-royale-chapter-4?lang=en-US) · [Fortnite Nanite](https://www.unrealengine.com/tech-blog/bringing-nanite-to-fortnite-battle-royale-in-chapter-4?lang=en-US) | January 2023 / UE 5.1-era production experience; historical settings require revalidation |
| [NVIDIA DLSS integration](https://developer.nvidia.com/rtx/dlss) · [TSR documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/temporal-super-resolution-in-unreal-engine?utm_source=chatgpt.com) | DLSS page updated September 2026; TSR documentation current to 5.8 |
| [Unreal Insights reference](https://dev.epicgames.com/documentation/unreal-engine/unreal-insights-reference-in-unreal-engine-5?utm_source=chatgpt.com) · [AMD Unreal performance guide](https://gpuopen.com/learn/unreal-engine-performance-guide/?utm_source=chatgpt.com) | Profiling workflow; AMD-specific tuning should not be copied blindly to NVIDIA |
| [SIGGRAPH 2021 course](https://advances.realtimerendering.com/s2021/index.html?utm_source=chatgpt.com) · [SIGGRAPH 2022 course](https://advances.realtimerendering.com/s2022/index.html?utm_source=chatgpt.com) | Nanite, radiance caching, GIBS, Lumen and open-world ray-tracing talks |
| [ReSTIR paper and materials](https://research.nvidia.com/publication/2020-07_spatiotemporal-reservoir-resampling-real-time-ray-tracing-dynamic-direct?utm_source=chatgpt.com) · [Brixelizer documentation and code access](https://gpuopen.com/fidelityfx-brixelizer/?utm_source=chatgpt.com) | Research and inspectable implementation references, not TCE performance evidence |

## Bottom line

**The best first implementation is High software Lumen, Nanite ISMs, VSM sunlight tuned for uncached rendering, and reconstructed 1440p output.** Keep hardware Surface Cache Lumen and MegaLights as measured alternatives.

For a solo developer, the decisive investments are a disciplined modular kit, bounded scene residency, incremental simulation-to-render updates and repeatable packaged-build tests. Those controls are more likely to make TCE sustain 60 fps than a large collection of aggressive renderer overrides.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9296b-a188-83ea-87b3-8e2a1459a791)
