# Day/night, seasons, and weather rendering for TCE

**Engineering recommendation, as of September 27, 2026.** Engine-specific guidance below targets **Unreal Engine 5.8**; older game implementations and measurements are identified separately.

## Executive recommendation

For TCE, use **Unreal’s native atmosphere, lighting, and Niagara systems, coordinated by a small TCE-owned environment controller**. Keep the Rust kernel authoritative over time, regional weather, snow cover, wetness, and any consequences for people or agriculture.

For a solo developer, **Ultra Dynamic Sky with Ultra Dynamic Weather is a strong implementation shortcut**, provided it passes a packaged UE 5.8 integration test. Treat it as a replaceable presentation system—not as the owner of TCE’s calendar, weather simulation, or saved environmental state. Its published feature set includes synchronized sky lighting, several cloud-rendering options, and precipitation and lightning effects. [Fab.com](https://www.fab.com/listings/84fda27a-c79f-49c9-8458-82401fb37cfb)

The most important architectural distinction is:

> **Simulate environmental conditions across the world; render expensive environmental effects only where they contribute to the current view.**

An off-screen valley can accumulate snow without any snow particles. A distant storm can exist as a regional weather field, cloud formation, and rain shaft without thousands of distant emitters. Footprints can disappear from a visual cache without changing the amount of snow affecting travel.

Start with **Lumen High**, a time-sliced skylight, one economical volumetric cloud layer, restrained fog, and camera-local precipitation. UE 5.8 also introduces **Lumen Lite**, giving TCE another lower-cost lighting tier rather than requiring an immediate retreat to non-Lumen ambient lighting. Epic describes Lite as using irradiance fields with probe occlusion and claims roughly twice the speed of Lumen High; that is not a claim about doubling total game frame rate. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

**No published measurement found establishes that this complete TCE workload will achieve 60 fps on a 4070 Ti.** The budgets below are proposed acceptance criteria, not measured results.

---

## 1. Options: techniques and how they work

### 1.1 Day/night: native atmosphere and a persistent lighting rig

**Sky Atmosphere** handles atmospheric scattering, sky color, and aerial perspective. It supports two atmospheric directional lights, conventionally indexed as sun and moon. This is the appropriate foundation for changing solar elevation rather than blending between a few painted sky textures. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/sky-atmosphere-component-in-unreal-engine)

For TCE, I recommend one persistent environment rig containing a Sky Atmosphere, movable sun and moon directional lights, a movable Sky Light, Exponential Height Fog, optional Volumetric Fog, and a cloud component. Keep that rig outside streamed settlement cells.

Compute the sun direction from TCE’s latitude, orbital/seasonal phase, and time of day. A fixed daily rotation is adequate for an initial prototype, but the production controller should also change the sun’s seasonal path and day length. Keep astronomical calculations separate from artistic controls such as exposure and cloud appearance.

**Use the Sky Light’s Real Time Capture with time slicing**, initially at a 128-pixel cubemap resolution. Epic’s implementation spreads capture work across frames and keeps it on the GPU; repeatedly calling the Blueprint `RecaptureSky` path is less suitable for continuous changes. Real Time Capture supports the atmosphere, clouds, height fog, and appropriate sky meshes, but **does not capture Volumetric Fog**. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/sky-lights-in-unreal-engine)

For readability, author exposure transitions alongside the lighting. Night should not become daylight because exposure rises without limits, but physically darker interiors also should not become unusable. Fortnite’s Lumen integration is a useful precedent: Epic explicitly addressed indoor/outdoor exposure differences and used Local Exposure to preserve detail. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/lumen-brings-real-time-global-illumination-to-fortnite-battle-royale-chapter-4)

A cheaper alternative is an animated sky dome plus blended or tinted skylight cubemaps. Keep that as a scalability fallback; it is less responsive to arbitrary cloud formations and changing sky conditions.

### 1.2 Lumen: dynamic illumination, not instantaneous illumination

Lumen suits a world where buildings appear, doors open, and sunlight moves, but its lighting updates are amortized and cached. Large lighting changes can take seconds to settle. Increasing **Lumen Scene Lighting Update Speed** and **Final Gather Lighting Update Speed** improves responsiveness at additional GPU cost. Accelerated day/night therefore needs testing as a temporal problem, not only as a sequence of attractive screenshots. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-global-illumination-and-reflections-in-unreal-engine)

For TCE, begin with:

```
sg.GlobalIlluminationQuality=2
sg.ReflectionQuality=2
```

These select High GI and reflections. In UE 5.8, Medium GI can use Lumen Lite’s irradiance-field gather, while Medium reflections use SSR on smooth surfaces instead of Lumen Reflections. Older advice equating Medium with “Lumen off” is therefore no longer generally correct. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US)

**Software versus hardware tracing needs a representative city benchmark.** Epic notes that software tracing against the merged Global Distance Field is advantageous with many overlapping instances. Hardware tracing improves quality but has acceleration-structure and overlap-related costs. My starting choice for TCE would be software tracing, with hardware tracing tested as a quality/performance alternative on the 4070 Ti—not enabled merely because the GPU supports it. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US)

The building kit must support whichever representation is chosen. Epic recommends separate modular walls, floors, and ceilings for software Lumen; very thin, one-sided, or oversized compound meshes can produce poor distance fields and light leaks. Software tracing also does not represent World Position Offset deformation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

### 1.3 The moving sun and shadow-cache invalidation

The most consequential day/night performance trap is **Virtual Shadow Map caching**.

Epic states that light movement or rotation invalidates that light’s cached pages. Geometry changes also invalidate overlapping pages, while World Position Offset can cause continual invalidation. Consequently, a frozen-noon city benchmark can substantially understate the cost of the same city with a continuously moving sun and animated foliage. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

For TCE, I recommend:

* Only the dominant sun or moon should normally cast the expensive directional shadows.
* Profile continuously moving sunlight, including low solar angles.
* Test reduced light-update frequency only as a measured optimization; reject it when shadow stepping becomes noticeable.

Do not force moving foliage or changing snow geometry into a “static” shadow-cache mode simply to improve a benchmark. That can trade visible correctness for misleadingly low cost.

### 1.4 Clouds: layered methods rather than weather-fluid simulation

| Technique | How it works | TCE fit |
| --- | --- | --- |
| **Sky-dome or 2D clouds** | Animated coverage textures on a sky surface, with inexpensive lighting approximations. | Excellent low-cost tier and early milestone option. |
| **Native volumetric clouds** | Ray-march a density material through a cloud layer, usually driven by coverage maps and 3D noise. | Recommended normal-quality solution. |
| **Specialized local volumetric clouds** | Explicit cloud volumes, more elaborate density data, or close-up cloud rendering. | Reserve for exceptional scenes; unnecessary as the default city-world solution. |

For native volumetric clouds, Epic recommends **one multiple-scattering approximation octave** for games. **Beer Shadow Maps** are cheaper than secondary shadow ray marching and generally sufficient from the ground. Start with `r.VolumetricRenderTarget.Mode 0`; evaluate Mode 2 for ground-focused views, recognizing that it is less reactive and does not support intersections with opaque meshes. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/volumetric-cloud-component-in-unreal-engine)

My proposed TCE cloud model is a regional, low-resolution coverage/type field controlling the large formations, with procedural noise supplying detail. Wind moves the visible field; storm cells alter density and coverage locally. Do not make a full atmospheric fluid simulation a prerequisite for convincing storms.

Use distant rain shafts and haze to communicate weather beyond the local precipitation volume. Keep cloud darkening, sunlight attenuation, wind, and rain intensity coordinated so the player does not see heavy rain falling beneath an otherwise unrelated clear sky.

### 1.5 Fog: atmospheric depth with a bounded near-field cost

Exponential Height Fog is useful for broad atmospheric depth. Volumetric Fog adds spatial density and lighting within the camera frustum, making mist and shafts possible. Its cost depends strongly on the volume resolution; increasing its view distance without increasing resolution can also expose undersampling. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/volumetric-fog-in-unreal-engine)

For TCE, use modest global haze and selectively stronger local mist. Avoid making dense volumetric fog permanently cover the entire visible world.

UE 5.8’s new **Fog Screen Space Scattering** is explicitly Experimental. It may be worth evaluating later, but it should not be a dependency for M2–M6. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

### 1.6 Seasons: separate vegetation state from surface conditions

For TCE, “season” should not be a single global switch from summer assets to winter assets. I recommend three independent inputs:

**Vegetation state:** leaf growth, color, leaf loss, flowering, crop development.

**Deposited cover:** snow, frost, and possibly dust.

**Surface water:** dampness, saturation, puddles, and ice.

This representation permits autumn-colored trees without snow, snow persisting after snowfall stops, and different visual responses across regions.

#### Foliage

Use shared material functions for gradual leaf-color and grass-color changes. Give species and instances stable variation rather than recoloring every tree identically.

Actual leaf loss needs a separate solution: masked transitions, leaf-cluster visibility changes, or authored leaf-on/leaf-off mesh variants. My preference is **shared branch structure with a small number of foliage variants**, transitioned in budgeted spatial batches. Do not maintain four complete copies of the vegetation world.

Use per-instance data for individual seasonal variation and Custom Primitive Data for suitable non-instanced primitives rather than creating a unique dynamic material instance for every object. Unreal’s custom-data mechanisms are intended to support such variation while retaining shared materials and instancing opportunities. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

For acceptance testing, inspect not just the tree’s color but also its shadow, Lumen representation, reflection, and distant impostor. A leafless near mesh paired with a green distant representation is an especially conspicuous failure.

Wind deserves its own distance budget. Epic specifically recommends disabling deformation where it becomes visually insignificant, because deformation also affects shadow-cache reuse. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

#### Snow accumulation

Start with a material-based coverage layer. A useful **proposed visual mask**, not a snow-physics equation, is:

```
visible snow =
    regional snow cover
  × surface retention
  × upward-facing/slope mask
  × precipitation exposure
  × small-scale breakup
```

Use authored vertex or texture masks to distinguish materials and shapes: thatch, smooth tile, stone ledges, vegetation, and sheltered undersides should not retain identical cover.

Change base color, roughness, and normal response together. Add actual snow-cap geometry only where accumulated thickness noticeably changes silhouettes—roof edges, walls, rocks, and selected nearby vegetation.

Treat footprints and wheel tracks as **bounded local detail**. Heightfields, render targets, and selective displacement can provide interaction without deforming the whole world. Batman: Arkham Origins demonstrated runtime snow-heightmap accumulation with different console and PC rendering paths; its key transferable idea is selective allocation and updating, not copying its old DX11 tessellation implementation verbatim. [ZigguratVertigo's Hideout](https://colinbarrebrisebois.com/wp-content/uploads/2022/06/gdc2014-deformable_snow_rendering.pdf)

#### Wetness and puddles

Wetness should alter material response, not just add a uniform glossy multiplier. Lagarde’s wet-surface research distinguishes effects related to roughness, porosity, and a water layer; applying identical darkening and specular changes to every material is a poor approximation. [Sébastien Lagarde](https://seblagarde.wordpress.com/2013/03/19/water-drop-3a-physically-based-wet-surfaces/)

For TCE, I recommend material-family response curves: porous masonry and soil darken differently from metal or smooth glazed surfaces. Accumulate puddles in authored or computed depressions, flatten the small-scale surface normal progressively as water depth increases, and add inexpensive ripple normals nearby. Do not turn wet surfaces into metallic surfaces or make every street a mirror.

A wetness transition is also a lighting-performance transition. Epic reports that Lumen reflection cost varies with how much of the screen contains sufficiently smooth materials requiring reflection rays. **Benchmark the wet street, not just the rain emitter.** [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/lumen-brings-real-time-global-illumination-to-fortnite-battle-royale-chapter-4)

#### Runtime Virtual Textures versus live weather fields

RVTs are useful for terrain blending and relatively stable surface information, but they are fundamentally **shading caches**, not automatically refreshed simulation databases. Epic identifies animated and movable content as poor RVT producers because the cache is not fully updated every frame. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine)

My recommendation is to retain stable terrain blending in RVTs where useful, while sampling changing snow/wetness from ordinary GPU textures or controlled render-target fields in the final material path. This avoids requiring a whole landscape’s cached shading to refresh whenever rain starts.

### 1.7 Rain, snow, and storms in Niagara

Use a **small number of camera-local Niagara systems**, not one precipitation system per building, citizen, or weather cell.

Epic emphasizes that GPU particle simulation does not eliminate Niagara’s CPU system/emitter overhead. More system instances create more game-thread work; Effect Types provide reusable scalability and culling controls. Pooling also helps avoid repeated allocation, although pool priming itself can hitch. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/scalability-and-best-practices-for-niagara)

My proposed rendering layers are:

| Layer | Presentation |
| --- | --- |
| **Near field** | Individual GPU rain streaks or snowflakes, with selected splashes and interactions. |
| **Middle distance** | Sparser particles and restrained sheets or shafts. |
| **Far distance** | Cloud formations, atmospheric attenuation, and distant precipitation volumes—not individual drops. |

Keep particles in world space within a recentered or wrapped volume. Simply attaching a rigid rain box to the camera makes precipitation move unnaturally during fast camera travel.

Avoid a universal particle-count target. Covered pixels, overlapping translucency, material complexity, and collision work matter alongside count. Snow should have a separate quality setting: slow, large flakes and long lifetimes can create a different visual and cost profile from narrow rain streaks.

**Collision options:**

* Screen-depth collision is inexpensive but view-dependent.
* Distance fields provide a coarse off-screen scene representation, with limitations on thin geometry.
* Niagara’s hardware-ray-traced collision remains **Experimental in the current documentation**, and its asynchronous result is one frame behind. It is not my recommended shipping dependency for ordinary precipitation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/gpu-raytracing-collisions-in-niagara-for-unreal-engine)

Use a small number of traces or spatial queries for important splashes. Do not raycast every drop or send particle collision events into Rust.

**Lightning** should be a bounded visual event: a bolt mesh or ribbon, cloud illumination, a short direct-light flash, and distance-delayed thunder. Fog’s temporal reprojection can leave trails from rapidly changing lights, so validate flashes with fog enabled and avoid depending on cached indirect illumination for the instantaneous effect. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/volumetric-fog-in-unreal-engine)

### 1.8 Existing systems and plugins

| Option | Strengths | Limitations and version position | Recommendation |
| --- | --- | --- | --- |
| **Native UE components plus TCE controller** | Maximum ownership; minimal vendor coupling; easiest to align with Rust state. | You must author transitions, weather assets, and tools. Native renderer guidance here is UE 5.8. | Required architectural foundation. |
| **Ultra Dynamic Sky / Ultra Dynamic Weather** | Broad integrated sky, cloud, precipitation, lightning, and material-weather feature set. | Online documentation inspected: **v9.7**. I could not independently establish its exact UE 5.8 compatibility from the parsed public listing. The asset also explicitly assumes flat, Z-up worlds rather than spherical planets. | First productivity-oriented candidate, conditional on a packaged 5.8 test. |
| **Sky Creator** | Preset-driven sky/weather, precipitation, material effects, and support for external directional lights. | Its published compatibility table explicitly lists **plugin 1.41.3 for UE 5.8**. The changelog also records cloud-AO instability and disables that feature by default. | Strong alternative to evaluate against the same scene. |

The vendor documentation supports these feature and compatibility distinctions; neither establishes a TCE-specific performance advantage. [Fab.com](https://www.fab.com/listings/84fda27a-c79f-49c9-8458-82401fb37cfb)

For UDS, use **Game / Real-time**, not Cinematic / Offline. Disable autonomous time/weather progression when Rust supplies those values. Be careful with material-state ownership: its documentation explicitly notes that disabling gradual material-state simulation can make snow disappear immediately when switching to a preset with zero snow. Feed persistent coverage explicitly instead. [Ultra Dynamic Sky](https://www.ultradynamicsky.com/Documentation/V9/9-7)

---

## 2. Trade-offs, published measurements, and TCE budgets

### 2.1 Published measurements: useful context, not transferable guarantees

| System | Published result | What the number actually describes |
| --- | --- | --- |
| **Lumen** | Approximately **4 ms for a 60 fps target**, **8 ms for a 30 fps target**, at **1080p internal resolution** on consoles. | Epic’s lighting budget covering GI/reflections and associated contributions—not total frame time. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US) |
| **Lumen Lite, UE 5.8** | Epic claims **twice as fast as Lumen High**. | Relative rendering-mode claim; no matching TCE/4070 Ti benchmark. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available) |
| **Sky Light capture** | PS4 example: **1.465 ms** for a full 128×128×6 HDR capture process; **at most 0.20 ms** for the most expensive step when spread over nine frames. | Historical hardware example retained in current documentation. Demonstrates the value of time slicing. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/sky-lights-in-unreal-engine) |
| **Volumetric Fog** | **1 ms on PS4 at High**; **3 ms on GTX 970 at Epic**, with eight times as many voxels. | Different hardware and settings—not a direct GPU comparison. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/volumetric-fog-in-unreal-engine) |
| **Horizon/Nubis clouds** | The 2015 prototype rendered in **under 2 ms on PS4**. | Specialized cloud rendering; later production requirements needed further work and optimization. [Schneider VFX](https://www.schneidervfx.com/) |
| **Arkham Origins snow** | Heightmap updates **under 1 ms on PS3/Xbox 360**; **2 MB** console heightmap memory. | Heightmap update cost, not total snow shading. Updates were selectively scheduled. [ZigguratVertigo's Hideout](https://colinbarrebrisebois.com/wp-content/uploads/2022/06/gdc2014-deformable_snow_rendering.pdf) |

These measurements show that bounded, amortized environmental rendering is practical. They do **not** justify adding the rows together or extrapolating a precise 4070 Ti result.

### 2.2 Proposed budget for TCE

At 60 fps, the full frame allowance is **16.67 ms**. I recommend these initial engineering gates:

| Budget item | Proposed target—not a measurement |
| --- | --- |
| Complete dry-weather GPU frame, including normal sky and lighting | **11–12 ms** in representative city views. |
| Incremental worst-weather cost | **No more than 2–3 ms**, including changed reflections, shadows, and materials—not merely particles. |
| GPU headroom | Keep normal tested operation around **14–15 ms or less**, leaving room for variation. |
| Weather-controller CPU work | Approximately **0.2 ms steady-state** as an initial target; avoid synchronous uploads and actor churn. |
| Peak resident VRAM | Initially aim around **9–10 GB**, retaining headroom within the 12 GB card for streaming and transient allocations. |

The exact partition should change after profiling. A city already taking 16 ms in clear weather does not have a viable storm budget.

Test **1440p output with TSR**, initially including a 75% internal-resolution case—1920×1080—and a native-resolution comparison. TSR explicitly supports lower internal rendering resolutions with higher-resolution output; inspect precipitation and moving vegetation for temporal artifacts rather than judging still screenshots. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/temporal-super-resolution-in-unreal-engine)

Keep frame generation separate from the acceptance criterion: TCE’s simulation and renderer should satisfy the chosen base-frame target without counting generated frames.

### 2.3 Quality scaling

My recommended degradation order is to reduce cloud sampling and secondary cloud features, then volumetric mist and precipitation overdraw, then distant deformation and detailed interactions. Reduce reflection quality separately from GI, and retain Lumen Lite as another fallback.

Preserve broad environmental communication even on lower settings: time of day, cloud cover, seasonal color, snow coverage, wetness, and storm location. These communicate simulation state; tiny droplets and volumetric shafts are embellishments.

---

## 3. Precedents and what they teach

### Fortnite Battle Royale Chapter 4 — dynamic lighting in a changing world

Epic’s UE 5.1-era implementation supported changing time of day and player-built/destroyed structures while targeting 60 fps. Software tracing was selected for that release’s budget, and exposure, foliage, and reflection issues required targeted work.

**Lesson for TCE:** dynamic GI is viable in a changing world, but content representation and temporal behavior must be engineered alongside it. “Enable Lumen” was not the whole solution. [Unreal Engine](https://www.unrealengine.com/en-US/tech-blog/lumen-brings-real-time-global-illumination-to-fortnite-battle-royale-chapter-4)

### Horizon Zero Dawn / Forbidden West — cloud rendering as a production system

Guerrilla’s Nubis work progressed from a fast cloud prototype to regional authoring, transitions, atmosphere integration, and storms. Later work specifically addressed fast cloud motion and internal lightning without requiring expensive simulations and lighting calculations.

**Lesson for TCE:** regional control and temporal stability matter more than endlessly increasing cloud detail. TCE needs believable storm organization, not a meteorological fluid solver. [Schneider VFX](https://www.schneidervfx.com/)

### Batman: Arkham Origins — selective interactive snow

The snow system generated local heightmaps, allocated them according to relevance, and prioritized updates. Its presentation describes rendering only two active snow surfaces per frame in the chosen configuration and recycling memory as surfaces became irrelevant.

**Lesson for TCE:** persist environmental state globally, but allocate high-resolution interaction detail selectively. Do not maintain deformable snow surfaces for every street and roof simultaneously. [ZigguratVertigo's Hideout](https://colinbarrebrisebois.com/wp-content/uploads/2022/06/gdc2014-deformable_snow_rendering.pdf)

### Remember Me — material research versus shipped requirements

Lagarde’s research explains dynamic wet-surface approximations, but he explicitly notes that *Remember Me* ultimately used static weather conditions and authored wet textures for much of the shipped result.

**Lesson for TCE:** borrow the material-response principles, not an assumption that the game proved a fully dynamic wetness pipeline. Static authored wetness is a valid optimization for a fixed scene, but cannot be TCE’s general solution. [Sébastien Lagarde](https://seblagarde.wordpress.com/2013/04/14/water-drop-3b-physically-based-wet-surfaces/)

### Hillaire’s atmosphere research and reference code

*A Scalable and Production Ready Sky and Atmosphere Rendering Technique* was presented at EGSR 2020. Its accompanying MIT-licensed reference project compares the technique used by Unreal with other approaches and a path tracer.

**Lesson for TCE:** use this to understand atmosphere rendering and diagnose artifacts, rather than building a replacement atmosphere renderer. The code is a research/reference application, not a drop-in UE 5.8 weather plugin. [GitHub](https://github.com/sebh/UnrealEngineSkyAtmosphere)

---

## 4. Recommended TCE architecture and implementation plan

The following is a proposed architecture tailored to TCE, rather than a claim about a particular plugin’s internal design.

### 4.1 Rust owns persistent state; Unreal owns its presentation

Use a one-way presentation flow:

**Rust simulation → versioned C ABI snapshot → Unreal environment subsystem → sky, materials, Niagara, audio.**

| Rust-provided data | Unreal responsibility |
| --- | --- |
| Simulation timestamp, calendar/orbital phase, location reference | Sun/moon directions and interpolated display state. |
| Regional cloud cover, precipitation, temperature, wind, visibility parameters | Clouds, fog, precipitation intensity, wind animation, sound. |
| Snow water equivalent or chosen snow-state measure, surface wetness, ice, saturation | Visible coverage, material response, local snow thickness approximations. |
| Lightning events with stable IDs, positions, and seeds | Bolt, flash, thunder, and cosmetic secondary effects. |
| Construction/destruction and shelter changes | Update local precipitation-blocking and material-exposure representations. |

Use immutable, versioned plain-data snapshots with bounded queues or double buffering. Let a C++ Unreal subsystem apply updates on the appropriate engine thread; do not expose arbitrary UObjects directly to Rust workers.

Keep updates compact. Weather does not need one FFI call per citizen, building module, or particle.

**Gameplay must never depend on the visible particles.** Crop watering, freezing, travel penalties, and lightning damage should follow kernel state/events even when the camera is elsewhere.

### 4.2 Separate simulation time from animation time

TCE needs two related clocks.

The **simulation clock** determines the actual date, sun position, environmental state, and events. The **presentation animation clock** advances noise, particle motion, cloud detail, and similar cosmetic motion at a controlled rate.

At high simulation speeds, do not multiply every raindrop velocity or wind oscillation by the time multiplier. Coalesce intermediate weather snapshots and interpolate the resulting state. Suppress or aggregate excessive transient effects such as lightning flashes.

However, do not silently let the sun show a different time from citizen activity. Choose an explicit fast-forward presentation policy: time-lapse lighting, discrete observational updates, or a clearly indicated alternative.

Use integer simulation ticks and double-precision calculations for long-lived time and spatial references. Send bounded phases and local coordinates to shaders instead of ever-growing century-scale float values.

### 4.3 Spatial state must not collapse into one global material parameter

A global parameter collection is appropriate for truly global or view-wide controls. It cannot, by itself, represent snow in one valley and rain in another.

I recommend a coarse persistent regional grid in Rust, reconstructed into local GPU weather fields around the camera. Materials combine those fields with per-instance variation, authored retention masks, and shelter information.

One possible bounded allocation is three nested 1024×1024 RGBA8 fields, each double-buffered. The raw texture storage is:

```
3 × 2 × 1024 × 1024 × 4 bytes = 24 MiB
```

That excludes mipmaps, auxiliary masks, and allocation overhead. This is an illustrative design, not a requirement to use three fields. The useful property is that GPU weather-state memory stays bounded while the simulated world grows.

Update dirty regions or newly exposed strips rather than rebuilding every field every frame. Reload appearance from persistent state after streaming or teleportation; do not require old GPU textures to have survived.

### 4.4 Shelter and accumulation need explicit treatment

A single overhead depth capture cannot fully describe stacked floors, caves, bridges, roof overhangs, and wind-driven rain.

For TCE’s modular architecture, add precipitation-exposure information to kit/building data. Use that to generate conservative roof and shelter masks when structures are assembled, and invalidate only affected tiles when they change.

Near the camera, supplement these masks with scene queries or captures for visual detail. The kernel’s shelter model remains authoritative for gameplay.

Keep **precipitation exclusion** separate from **drying and melting**. Adding a roof should stop new rain reaching the ground; it should not instantly erase existing puddles. Likewise, ending a snowstorm should not immediately remove accumulated snow.

A useful ordering for the visual material pipeline is underlying material and wear, then moisture response, then accumulated cover, then small transient effects. Give that ordering one owner so a plugin and TCE do not apply wetness twice.

### 4.5 Contain plugin coupling

Implement a small presentation interface along these lines:

```
ApplyCelestialState(...)
ApplyRegionalWeather(...)
ApplySurfaceState(...)
PresentLightningEvent(...)
ResetAfterTeleportOrLoad(...)
SetEnvironmentQuality(...)
```

Provide native and plugin-backed implementations where practical.

Keep vendor assets isolated and avoid broad forks. Have coding agents modify the TCE adapter and tests rather than freely restructuring vendor Blueprints. Record exact engine/plugin versions and test upgrades in a branch.

A plugin should pass four decisive checks: externally controlled time, externally controlled persistent surface state, spatially different weather, and reliable packaged execution after load/teleport. Attractive editor demonstrations are insufficient.

### 4.6 Milestone plan

| Milestone | Build | Acceptance gate |
| --- | --- | --- |
| **M2: day/night** | Persistent native rig; Rust-driven time; exposure transitions; time-sliced skylight; basic cloud tier. | Representative city survives a full moving-sun cycle, dusk/night transitions, pause, fast-forward, and save/load without lighting-state drift or major hitches. |
| **M3: seasons** | Species-aware color/leaf state; shared snow/wetness material interface; spatial fields; coherent distant foliage. | Seasonal transitions do not duplicate the world’s assets, create mass update spikes, or leave near/far representations inconsistent. Snow state survives streaming. |
| **M6: weather** | Regional storms; local Niagara precipitation; shelter masks; wetness/puddles; lightning/audio; selected snow interaction. | Worst-weather street and aerial views remain within budget, including wet reflections, wind, construction changes, and rapid camera travel. |

Introduce the persistent surface-state interface in M3 even when initial snow cover comes from simplified seasonal rules. M6 can then improve its inputs without replacing every material.

### 4.7 Profiling and failure tests

Use a packaged Development/Test build, a deterministic weather sequence, and repeatable camera paths. Record CPU game/render time, Rust tick time, GPU frame time, VRAM, and frame-time percentiles—not only average fps. AMD’s Unreal performance guide is useful for establishing repeatable profiling conditions and distinguishing CPU from GPU limitations. [AMD GPUOpen](https://gpuopen.com/learn/unreal-engine-performance-guide/)

The test suite should include a dry city and the identical city in rain; dense summer foliage in wind; snowy streets and rooftops; dusk and night; interior/exterior transitions; roof construction and demolition; distant storms; rapid flight and teleportation; and long-duration save/load plus fast-forward.

For shadows, inspect cached-page invalidation and use Unreal Insights’ VSM counters. Epic warns that asynchronous compute can complicate individual GPU-pass timing; diagnostic measurements must not be mistaken for the final shipping configuration’s frame cost. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

Finally, budget visibility independently from population. The kernel’s 50,000 people do not imply 50,000 high-detail rendered characters—and certainly not 50,000 weather interaction components. Fast-forward must also leave enough CPU capacity for rendering, streaming, and input responsiveness.

---

## 5. Linked sources and version applicability

### Engine documentation

| Source | Applicability |
| --- | --- |
| [UE 5.8 release overview](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available?utm_source=chatgpt.com) | 2026 release status, Lumen Lite, and Experimental fog-scattering status. |
| [Sky Atmosphere](https://dev.epicgames.com/documentation/en-us/unreal-engine/sky-atmosphere-component-in-unreal-engine?utm_source=chatgpt.com), [Sky Lights](https://dev.epicgames.com/documentation/en-us/unreal-engine/sky-lights-in-unreal-engine?utm_source=chatgpt.com), [Volumetric Clouds](https://dev.epicgames.com/documentation/en-us/unreal-engine/volumetric-cloud-component-in-unreal-engine?utm_source=chatgpt.com) | Current UE 5.8 documentation; some benchmark examples are historical. |
| [Lumen performance guide](https://dev.epicgames.com/documentation/unreal-engine/lumen-performance-guide-for-unreal-engine?lang=en-US&utm_source=chatgpt.com), [technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine?utm_source=chatgpt.com) | Current 5.8 scalability and representation guidance. |
| [Virtual Shadow Maps](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine?utm_source=chatgpt.com), [Volumetric Fog](https://dev.epicgames.com/documentation/en-us/unreal-engine/volumetric-fog-in-unreal-engine?utm_source=chatgpt.com) | Current documentation; retained PS4/GTX 970 measurements are not 5.8/4070 Ti benchmarks. |
| [Niagara scalability](https://dev.epicgames.com/documentation/en-us/unreal-engine/scalability-and-best-practices-for-niagara?utm_source=chatgpt.com), [GPU ray-tracing collisions](https://dev.epicgames.com/documentation/en-us/unreal-engine/gpu-raytracing-collisions-in-niagara-for-unreal-engine?utm_source=chatgpt.com) | Current guidance; ray-traced collision is marked Experimental. |
| [Runtime Virtual Texturing](https://dev.epicgames.com/documentation/en-us/unreal-engine/runtime-virtual-texturing-in-unreal-engine?utm_source=chatgpt.com), [Custom Primitive Data](https://dev.epicgames.com/documentation/en-us/unreal-engine/storing-custom-data-in-unreal-engine-materials-per-primitive?utm_source=chatgpt.com) | Surface-data integration and caching behavior. |

### Systems, papers, talks, and code

| Source | Applicability |
| --- | --- |
| [Ultra Dynamic Sky listing](https://www.fab.com/listings/84fda27a-c79f-49c9-8458-82401fb37cfb?utm_source=chatgpt.com), [v9.7 documentation](https://www.ultradynamicsky.com/Documentation/V9/9-7?utm_source=chatgpt.com) | Features and integration behavior; verify the specific purchased UE 5.8 build. |
| [Sky Creator documentation](https://dmkarpukhin.com/docs/sky-creator/?utm_source=chatgpt.com) | Explicit UE 5.8 / plugin 1.41.3 compatibility table. |
| [Lumen in Fortnite Chapter 4](https://www.unrealengine.com/en-US/tech-blog/lumen-brings-real-time-global-illumination-to-fortnite-battle-royale-chapter-4?utm_source=chatgpt.com) | UE 5.1-era shipped implementation, published 2023—not current scalability defaults. |
| [Nubis/cloud talks and slides](https://www.schneidervfx.com/?utm_source=chatgpt.com) | Guerrilla/Decima research and production experience, 2015 onward. |
| [Arkham Origins deformable snow slides](https://colinbarrebrisebois.com/wp-content/uploads/2022/06/gdc2014-deformable_snow_rendering.pdf?utm_source=chatgpt.com) | GDC 2014; transferable scheduling/heightfield techniques, not a current Unreal implementation. |
| [Wet surfaces, part 3a](https://seblagarde.wordpress.com/2013/03/19/water-drop-3a-physically-based-wet-surfaces/?utm_source=chatgpt.com), [part 3b](https://seblagarde.wordpress.com/2013/04/14/water-drop-3b-physically-based-wet-surfaces/?utm_source=chatgpt.com) | 2013 material research; principles rather than drop-in UE 5.8 shaders. |
| [Hillaire atmosphere paper](https://onlinelibrary.wiley.com/doi/10.1111/cgf.14050?utm_source=chatgpt.com), [reference code](https://github.com/sebh/UnrealEngineSkyAtmosphere?utm_source=chatgpt.com) | EGSR 2020 paper and MIT-licensed reference implementation. |
| [AMD Unreal performance guide](https://gpuopen.com/learn/unreal-engine-performance-guide/?utm_source=chatgpt.com) | Profiling methodology; not an RTX 4070 Ti weather benchmark. |

**Bottom line:** TCE should own **time, regional environmental state, persistence, and the adapter between simulation and rendering**. Use native Unreal rendering—and selectively a weather plugin—for presentation. The path to 60 fps is bounded local detail, inexpensive persistent state, coordinated materials and lighting, and testing the entire wet, windy, moving-sun city rather than optimizing each effect in isolation.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9297a-1300-83ea-9553-935bad0670f3)
