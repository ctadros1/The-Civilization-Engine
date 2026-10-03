# Fake interiors for TCE: engineering report

**Recommendation:** Use **opaque, analytically ray-box-mapped interiors**, backed by a small, reusable library of baked rooms. Prototype with Unreal’s `InteriorCubemap` material function; for production, prefer a **2D room atlas** when its authoring pipeline is working reliably. Drive lighting through **stable simulation-room identifiers**, not random per-window illumination. Reserve real interior geometry for entrances, shops, corner windows, and other places where the camera can expose the illusion.

This separates three concerns that should remain independent: **what room exists in the simulation, how its appearance is stored, and how a window renders it**.

**Version basis:** This report is current to **September 27, 2026**. UE 5.8 was released on June 23, 2026; Epic documentation cited below is the 5.8 documentation unless otherwise noted. Historical shader examples are identified separately. No UE5.8/RTX 4070 Ti measurements were performed for this report. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

---

## 1. Options: what the techniques actually do

### Interior mapping, cubemaps, and parallax are not competing categories

**Interior mapping** is the rendering technique: trace the viewing ray into an implied room and shade the surface it would hit. A **cubemap** or **2D atlas** is a way of storing the room’s appearance. **Parallax** is the resulting visual effect.

A cubemap sampled using only viewing direction is not equivalent to a room with positional parallax. The useful version first intersects an implied box, then uses that intersection to sample the cubemap. Harry Alisavakis’s compact implementation demonstrates this distinction directly. [Harry Alisavakis](https://halisavakis.com/my-take-on-shaders-interior-mapping/)

### Technique comparison

The cost descriptions below are relative engineering judgments, not measured UE5.8 timings.

| Technique | How it works | Strengths | Limitations and appropriate use |
| --- | --- | --- | --- |
| **Flat interior image or emissive window** | Displays a room photograph/render, curtain, or colored light patch on the façade. | Simplest shader and authoring; ideal distant representation. | No positional parallax. Use for small windows on screen, opaque curtains, and distant city lighting. |
| **Analytic box interior mapping** | Intersects the view ray with an implicit rectangular room; selects the wall, floor, or ceiling hit. | Fixed, small amount of intersection work; no interior meshes. | Rectangular-room assumption; apparent depth is not actual scene geometry. Good default for repeated windows. |
| **Box-projected cubemap interior** | Uses the box intersection to sample a baked cubemap. | Convenient UE prototyping and room capture; coherent shell appearance. | Furniture baked into the capture is effectively projected onto the box and can stretch. Many independently selectable cubemap assets complicate batching. |
| **Pre-projected 2D room atlas** | Stores each room in a prescribed 2D projection and remaps the intersection into that tile. | Straightforward variation through tile indices; shared texture resources; compact production pipeline. | Projection and bake must agree exactly; seams and mip bleeding require care. Recommended production direction. |
| **Box interior plus virtual furniture planes** | Adds one or more ray-intersected, alpha-composited planes inside the implied room. | Better depth separation for curtains, furniture, or silhouettes. | Extra texture work; flat-card artifacts remain. Alpha compositing can occur inside an **opaque** material. |
| **Parallax occlusion mapping or depth-image interiors** | Searches a height/depth representation, generally using multiple samples. | More irregular apparent surfaces than a bare box. | More expensive and harder to stabilize; hidden surfaces cannot be reconstructed from a single depth image. Consider only for selected hero windows. |
| **Real shallow rooms or full interiors** | Models the actual room shell and selected contents. | Correct geometry, corner relationships, and close inspection. | More asset, scene, lighting, and streaming work. Best for exceptional locations rather than every room. |

Gotow provides a practical tangent-space, pre-projected-atlas implementation. The original interior-mapping paper describes virtual furniture planes. Epic specifically warns that custom expressions and POM can have derivative-related artifacts with Nanite, so a generic POM implementation should not be assumed production-ready there. [Andrew Gotow](https://andrewgotow.com/2018/09/09/interior-mapping-part-2/)

### The recommended intersection model

For TCE, define a room-local coordinate system with known width, height, depth, and window position. Transform the camera ray into that space and intersect the enclosing box.

Conceptually:

```
surface position + camera ray
    → room-local position and direction
    → nearest valid box-exit intersection
    → cubemap direction or room-atlas UV
    → baked room appearance
    → simulation lighting and curtain state
    → exterior glass/reflection response
```

This is a closed-form intersection, **not a ray-marching loop**, and does not require hardware ray tracing. Non-uniform room dimensions must affect the ray transformation as well as the image mapping; simply stretching the texture coordinates is insufficient. Epic’s Ryan Brucks documented that issue when extending `InteriorCubemap` for non-uniform rooms. [Harry Alisavakis](https://halisavakis.com/my-take-on-shaders-interior-mapping/)

For multiple windows looking into one room, use the **same room box and different window positions within it**. Giving each pane an independent `frac(UV)` room produces plausible individual windows but an incoherent building.

---

## 2. Trade-offs and the performance evidence

### What actually scales

For an opaque analytic implementation, a useful planning model is:

\[
T\_{\text{interiors}} \approx
P\_{\text{shaded windows}}\ C\_{\text{shader}}
+T\_{\text{instance processing}}
+T\_{\text{state uploads}}
+T\_{\text{lighting/reflection interaction}}
\]

This is an engineering model, not a fitted benchmark.

The important distinction is **visible pixel coverage versus simulated room count**. At native 2560×1440, the frame contains 3.69 million pixels. If windows cover 30%, approximately **1.11 million pixels** need the interior shading path before accounting for overdraw and other passes. A close façade can therefore be more demanding than a distant city containing far more windows.

Nanite reduces geometry-management costs, but Epic explicitly retains instance count, material complexity, resolution, and overdraw as practical considerations. It does not make a complex window shader free. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

### Published measurements: useful, but historical

The original paper tested an OGRE application at **1024×768** on an **Athlon 2500+, 1 GB RAM, and GeForce 6600 GT with 128 MB VRAM**. Its measurements included:

| Historical test | Reported result |
| --- | --- |
| Depth-first rendering in one building-grid test | 881 → 1,265 frames in five seconds, approximately **44% higher throughput** |
| Reflective-window material versus interior mapping | 5,100 versus 999 frames in five seconds |
| Increasing procedural rooms at unchanged scene representation | 1,000 → 4,000,000 rooms without meaningful performance change |

These results illustrate pixel-shader cost and the value of avoiding hidden-pixel work. **They are not RTX 4070 Ti forecasts.** The geometry comparison also used different draw-call counts, so its crossover point should not be transferred to Nanite. [Proun Game](https://www.proun-game.com/Oogst3D/CODING/InteriorMapping/InteriorMapping.pdf)

**Evidence gap:** I found no reliable public, isolated benchmark for this effect on UE5.8/RTX 4070 Ti, nor separate window-shader timings for Spider-Man or Cities: Skylines II. Whole-game frame rates would not isolate this subsystem anyway.

### Practical trade-offs for TCE

**Analytic boxes versus POM:** A box gives a predictable small intersection calculation. POM pays for a search through depth samples. For ordinary residential windows, prioritize better room art and consistent room placement before adding depth-search complexity. Epic’s Nanite derivative warning further favors validating the simpler approach first. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

**Cubemap versus atlas:** The difficult part is often not sampling one room—it is selecting hundreds of variants without creating hundreds of materials. An atlas lets an integer-like parameter select a tile within a shared texture. Gotow’s implementation was explicitly designed around coherent building styles and atlas-based room variation. [Andrew Gotow](https://andrewgotow.com/2018/09/09/interior-mapping-part-2/)

**Fake versus real geometry:** Do not assume fake interiors always win. A few nearby, shallow, instanced rooms can be worth their cost, particularly where their geometry solves obvious visual contradictions. Benchmark them against the shader rather than rejecting them on triangle count alone.

---

## 3. Precedents: what is established, and what is inferred

| Precedent | Evidence and version | Lesson for TCE |
| --- | --- | --- |
| **Joost van Dongen’s Interior Mapping paper** | Published in 2008; algorithm, extensions, and historical measurements. | The basic technique is mature. Constant procedural-room complexity does not mean zero pixel cost. |
| **SimCity (2013)** | Gotow credits Andrew Willmott’s SimCity work for the pre-projected representation. Ocean Quigley’s GDC 2013 talk documents the broader simulation-readable art pipeline. | Organize interiors as a reusable content system, not bespoke rooms behind every façade. [Andrew Gotow](https://andrewgotow.com/2018/09/09/interior-mapping-part-2/) |
| **Marvel’s Spider-Man, PS4 (2018)** | A strong visual precedent, but van Dongen explicitly described the interior-mapping identification as an inference, not an official Insomniac confirmation. | Do not treat internet reconstructions as documentation of the exact shipped shader, atlas layout, or lighting system. [Joost's Dev Blog](https://joostdevblog.blogspot.com/2018/09/interior-mapping-real-rooms-without.html) |
| **Robo Recall / Unreal `InteriorCubemap`** | In 2017, Ryan Brucks described helping the Robo Recall team support non-uniform cubemap rooms. | A directly documented UE production lineage, including the importance of room dimensions and capture conventions. [Epic Developer Community Forums](https://forums.unrealengine.com/t/using-textures-on-the-new-interiorcubemap-mf/78464) |
| **Cities: Skylines II** | Official asset-pipeline excerpts describe window-submesh UVs controlling parallax rooms and night lighting. Editor documentation exposes a building-occupancy preview. | Window-to-room metadata belongs in the asset pipeline. Building occupancy is not proof of exact agent-in-room lighting. [Paradox Wikis](https://cs2.paradoxwikis.com/Asset_Pipeline%3A_Buildings) |
| **Trey McNair’s cubemap-window shader** | Public UE test buildings demonstrate cubemap capture, blinds, hue, emissive, and instance variation. The page does **not** identify this shader as Spider-Man’s shipped implementation. | Useful art-direction reference; avoid conflating an artist’s personal test with their shipped-game work. [ModSoft](https://modsoft.net/projects/W290vD?album_id=1568317) |
| **Public shader implementations** | Alisavakis publishes CC0 tutorial code; `three-interior-mapping` is an MIT-licensed Three.js project. | Good mathematical references and test harnesses, not drop-in UE5.8 integrations. Audit bundled texture rights separately. [Harry Alisavakis](https://halisavakis.com/my-take-on-shaders-interior-mapping/) |

For Cities: Skylines II, the official wiki’s complete pages returned access errors during retrieval. The indexed excerpts support the statements above, but I would not use them to assert the exact current shader implementation or a particular 2026 room-variation limit.

---

## 4. Recommended TCE architecture

### 4.1 Make room identity authoritative

The simulation should own room semantics. The renderer should own only their visual representation.

A suitable conceptual split is:

```
Rust:
    StableRoomId
    occupants and activities
    room use
    lamp/fuel/electricity state
    shutters/curtains
    persistent furnishing selection

Unreal:
    StableRoomId → loaded render-room slot
    render-room slot → window surfaces
    room appearance resources
    current visual transition state
```

**Do not use an ISM array index as a persistent room identifier.** Unreal exposes instance reordering and removal mechanisms; render handles are implementation details rather than world identity. Keep an explicit mapping and rebuild it when chunks are reconstructed. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent)

An occupied room should not automatically be bright. TCE can implement:

\[
L\_r = f(\text{activity},\text{daylight},\text{lamp availability},
\text{fuel/power},\text{household policy})
\]

For example, sleeping occupants may leave a room dark; an empty workshop may retain a process fire. This is a proposed simulation rule, not a claim about the cited games.

Where the kernel knows only building-level presence, present lighting as a **building-level approximation** until room allocation exists. A shader cannot manufacture room-accurate occupancy from insufficient simulation data.

### 4.2 Start with per-window ISMs; retain an upgrade path

Epic distinguishes **Custom Primitive Data**, shared by a primitive/component, from **Per Instance Custom Data**, which varies between ISM instances. Neither automatically supplies independent state to every window within a single building instance. Epic also recommends ISM over HISM for an exclusively Nanite workflow because Nanite already supplies its own culling and LOD system. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Two sensible implementations are:

| Architecture | State addressing | Advantages | Costs |
| --- | --- | --- | --- |
| **Initial implementation: window-surface ISMs** | Each window instance receives its room’s appearance and lighting values. | Straightforward material graph, debugging, and incremental development. | More instances; several windows belonging to one room receive duplicated updates. |
| **Scaling implementation: façade instances plus room-state table** | Per-instance base index + mesh-local room slot → GPU state record. | Fewer window instances; one room-state change affects all associated windows. | Additional state lookup and more demanding asset metadata. |

For a solo developer, **start with the first**. Change representation only when profiling identifies instance/update overhead as material.

Group components by spatial chunk, mesh family, and shared material. Avoid both a window Actor per pane and a single unbounded world-sized component.

### 4.3 A practical per-instance payload

An initial eight-float layout could be:

| Slots | Proposed meaning |
| --- | --- |
| 0 | Room-atlas variant |
| 1 | Current light blend |
| 2 | Curtain or shutter openness |
| 3 | Lamp intensity scale |
| 4 | Palette / lamp-color selection |
| 5 | Persistent decoration seed |
| 6 | Activity or appearance flags |
| 7 | Depth scale relative to the room archetype |

This is a proposed layout, not an engine requirement. Keep full-width persistent identifiers on the CPU; send compact render indices rather than converting arbitrary 64-bit IDs to floats.

UE5.8 exposes `NumCustomDataFloats`, `SetCustomData`, `SetCustomDataValue`, and ID-based custom-data operations. Allocate the required payload before populating instances, batch updates, and validate the selected API’s update behavior in the pinned engine build. Do not rebuild transforms or components merely because a lamp changed. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent)

A float cannot directly select an arbitrary `TextureCube` asset. A large cubemap library requires an array-capable material path, resource grouping, or another explicit selection mechanism. Avoid a giant material graph that samples many cubemaps and chooses afterward. This is a major reason to favor a 2D atlas.

### 4.4 Rust-to-Unreal update path

Use a **versioned C ABI** with fixed-width records and explicit ownership. Do not pass Rust `Vec`, `String`, or Unreal objects across the boundary as though their layouts and lifetimes were interchangeable. Prevent unwinding across an incompatible ABI boundary. These precautions follow Rust’s FFI guidance. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Recommended flow:

```
Simulation changes room state
    → coalesced RoomVisualDelta batch
    → Unreal consumes an immutable snapshot
    → loaded-room lookup
    → batched instance-data or texture-region updates
```

Keep Unreal object mutation on the appropriate Unreal-side thread; the Rust simulation should not directly manipulate components.

As starting policies, publish visual deltas on change, coalesced up to roughly **10 Hz**, and blend transitions over **0.2–0.5 real seconds**. These are proposed presentation settings. They should not become simulation rules or slow down accelerated time.

Maintain unloaded rooms’ authoritative state. When a chunk streams in, initialize it from the current snapshot rather than replaying every missed light event.

### 4.5 Separate visible brightness from actual scene lighting

Use independently controllable lighting contributions:

\[
C\_{\text{room}} =
a\_{\text{day}}\,C\_{\text{day}}
+a\_{\text{lamp}}\,C\_{\text{lamp}}
\]

Here, `C_day` and `C_lamp` are baked linear-space contributions, not two arbitrarily exposed screenshots. This supports daylight, lamps, and darkness without making illuminated fixtures permanently visible in an “off” texture.

For the city-wide path, fake the room’s visible radiance. **Do not create one point or rect light per illuminated window.** Use a limited pool of nearby spill lights where light falling onto the street materially improves the scene.

Lumen needs special handling: Epic warns that camera-dependent materials can be captured incorrectly in the Surface Cache and recommends a `Ray Tracing Quality Switch` alternative. Small emissive objects can also produce inconsistent lighting when only screen traces retain them. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

Therefore, provide a simple, camera-independent capture representation and test it against the main-view shader. A useful approximation is a dim window color with restrained emissive output, while the main view shows the detailed fake room. Excluding a mesh from one ray-tracing representation does not necessarily remove screen-space interactions.

---

## 5. Atlas authoring and variation

### 5.1 Build rooms once, bake many variants

For TCE, the scalable production asset is a **room archetype**, not a unique furnished room for every household.

Begin with a modest library—for example, **six to eight room uses with several furnishing variants each**. Tag content by capabilities and culture: glazing, furniture construction, lighting technology, wealth, and occupation. A modern office should never be selected merely because a building has reached a numerical “level.”

Use a manifest such as:

```
room archetype
room dimensions and coordinate convention
valid architectural / technology tags
capture or projection version
day contribution
lamp contribution
optional furniture / curtain layer
palette and permitted variation
```

These are proposed pipeline fields. The important requirement is deterministic rebuilding: the same source kit and manifest should produce the same atlas layout and identifiers.

### 5.2 Two viable bake pipelines

**Cubemap prototype pipeline.** Build a physical room in an authoring scene, capture from the agreed center using `SceneCaptureCube`, and convert the render target to a static cubemap. This is the workflow described by Ryan Brucks. Do the capture during authoring, not for each window at runtime. [Epic Developer Community Forums](https://forums.unrealengine.com/t/using-textures-on-the-new-interiorcubemap-mf/78464)

**Production atlas pipeline.** Bake each room into the exact 2D projection expected by the shader, then pack the tiles. Gotow’s pre-projected representation demonstrates this approach, but its projection is a specific convention—not an arbitrary room image. [Andrew Gotow](https://andrewgotow.com/2018/09/09/interior-mapping-part-2/)

For TCE, keep the cubemap version as a reference renderer while developing the atlas conversion. Compare them at fixed camera positions. This isolates projection errors from material, lighting, and asset errors.

### 5.3 Bake appearance, not accidental camera processing

Recommended bake controls are fixed exposure, consistent color space, consistent room dimensions, and explicitly separated lighting contributions. Avoid baking a tone-mapped “pretty screenshot” and then treating it as linear radiance.

For storage, BC7 is suitable for high-quality standard-range color, while BC6H supports HDR RGB. Microsoft documents both formats; the choice should follow the actual content range rather than “HDR” being assumed necessary. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression-in-direct3d-11)

Keep important furniture close to the room’s implied surfaces. Large foreground furniture exposes the projection most clearly. McNair’s authoring notes specifically recommend arranging set dressing against the walls to reduce stretching across capture edges. [ModSoft](https://modsoft.net/projects/W290vD?album_id=1568317)

### 5.4 Avoid atlas and identity failures

Treat these as production acceptance requirements:

| Failure | Prevention |
| --- | --- |
| Neighboring rooms bleed into one another at distance | Extrude tile borders and generate mip levels with tile boundaries in mind. Clamp to safe interiors of tiles. |
| Rooms shimmer at tile boundaries | Validate derivatives and mip selection across UV discontinuities; do not assume a `frac()`-based mapping is automatically stable. |
| A building changes furniture when streamed back in | Select variants from persistent room identity, not transient instance order or camera position. |
| Floors become walls after random rotation | Permit only transformations valid for the projection and furniture layout. |
| Every room looks unrelated | Share household/building palettes and furnishing families; vary details within that family. |
| Two windows show incompatible versions of one room | Share the room box, appearance selection, and lighting state; vary only each window’s opening coordinates. |

A texture array can avoid filtering between neighboring atlas tiles, but it introduces its own asset and material requirements. For this project, a conventional padded atlas is a reasonable first shipping target; validate arrays only where they reduce a demonstrated production problem.

### 5.5 Memory planning

BC7 and BC6H use 16 bytes per 4×4 block, equivalent to one byte per texel before the mip chain. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression-in-direct3d-11)

The following are **calculated payload estimates**, not measured Unreal allocations. Full mip chains are approximated as \(4/3\) of the top mip.

| Resource | Approximate payload |
| --- | --- |
| One 4096×4096 BC7 atlas, full mips | **21.3 MiB** |
| Two such atlases, e.g. day and lamp contributions | **42.7 MiB** |
| 64 cubemaps, six 256×256 faces each, BC7, full mips | **32 MiB** |
| Same cubemap library with two lighting contributions | **64 MiB** |
| Same two-contribution library at 512×512 per face | **256 MiB** |
| Eight custom floats × 100,000 window instances | **3.05 MiB**, excluding other instance data and copies |
| 512×256 RGBA8 room-state texture, no mips | **0.5 MiB**, providing 131,072 four-byte records |

A 4K atlas arranged as 8×8 tiles gives 512×512 pixels **for each entire projected room tile**, not for each of its walls.

The state itself is inexpensive. Unnecessary actors, components, material instances, resource duplication, and updates are the larger architectural risks. Budget room textures alongside—not independently of—Lumen, shadows, terrain, crowds, and Nanite residency.

---

## 6. Nanite, glass, and visual pitfalls

### Prefer an opaque window surface

Epic’s Nanite documentation supports **Opaque and Masked**, not ordinary Translucent materials. An opaque fake-window surface is therefore the straightforward path. A curtain or furniture mask composited within that material does not require making the window translucent. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

Recommended material composition:

* Baked room appearance supplies the apparent interior.
* A restrained surface response supplies glass reflections and roughness.
* Frames, sills, and recesses remain real geometry.
* Extra translucent glass is reserved for exceptional close-up assets.

Avoid excessive emissive brightness: it makes windows look like screens and can amplify lighting artifacts. Also avoid adding pixel-depth offset simply to make the depth “more real”; first establish whether it improves the shot enough to justify its additional interactions.

### Do not assume tiny window meshes benefit from Nanite automatically

Use Nanite for TCE’s architectural kits. For separate window cards, compare Nanite and non-Nanite versions in the target build.

There is an important integration detail: Epic states that ISM Surface Cache support requires a Nanite mesh. Consequently, a non-Nanite-card benchmark must include the resulting Lumen behavior, not just raster cost. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

### Protect tangent bases and room indices

Interior mapping is sensitive to tangent orientation. UE5.8 supports **Explicit Tangents** for Nanite, at an approximately 10% storage increase according to Epic. Use that option where implicit tangents visibly destabilize the room mapping rather than enabling it indiscriminately. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details)

For the façade-plus-state-table architecture, room slots stored in UV channels are **indices, not ordinary coordinates**. Epic’s `Lerp UVs` setting should be disabled when interpolation of such data would be invalid; that also changes how UV error contributes to Nanite LOD selection. Test the complete asset, including ordinary texture UVs. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details)

### Recognize the illusion’s hard boundaries

The apparent room is not automatically present in collision, depth, shadow geometry, or ray-tracing acceleration structures. This follows directly from rendering it on a façade rather than creating interior geometry.

Use real shallow interiors or special treatment for glass corners, camera-accessible entrances, and openings through which the viewer should see another exterior window. Shared identifiers solve lighting consistency; they do not by themselves solve multi-view geometry.

Also test temporal reconstruction carefully: the visible room texture moves with apparent parallax while the actual depth belongs to the window surface. Camera motion, thin furniture, and abrupt lighting changes can expose that mismatch.

---

## 7. Performance targets and the implementation spike

### Proposed starting targets

These values are **acceptance targets to test**, not published performance results.

| Parameter | Starting target |
| --- | --- |
| Total frame budget at 60 fps | **16.67 ms** |
| Incremental fake-interior GPU cost, representative views | **0.5–1.0 ms** |
| Incremental cost in deliberately window-heavy stress views | **≤1.5 ms**, subject to whole-frame trade-offs |
| Render-side state processing | Aim for **≤0.2–0.5 ms** per update frame |
| State publication | On change, coalesced up to roughly **10 Hz** |
| Visual transition | **0.2–0.5 real seconds** |
| Initial appearance library | Approximately **48–64 room variants** |
| Very small windows | Test flat representation below roughly **3–6 pixels** in height |

Use projected size with hysteresis rather than universal distance thresholds. An architectural window and a small cottage opening should not switch at the same world distance.

For context, Epic’s Lumen performance guide assigns **4 ms at 1080p** to its 60-fps console target and **8 ms** to its 30-fps target. Those are not 4070 Ti measurements, but they reinforce why interiors should consume a small fraction of the overall rendering budget. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine)

### The spike should answer three questions

**First: does the image hold up?** Compare flat, cubemap, atlas, atlas-plus-one-layer, and shallow-geometry versions of the same façade. Include frontal and grazing views, several windows into one room, night transitions, and camera motion.

**Second: what is the actual bottleneck?** Independently vary resident window count, visible coverage, and update rate. Suggested stress cases are 10,000 / 50,000 / 200,000 resident windows; 10% / 30% / 60% screen coverage; and 1% / 10% / all-room update bursts. These are test cases, not promised capacities.

**Third: does it survive runtime assembly?** Repeatedly create, remove, stream, save, and reload buildings. Check that room identity, furnishing, and illumination survive instance reordering and reconstruction.

Run packaged DX12 builds with warmed shaders, and report both native 1440p and the intended upscaled configuration with its **actual internal resolution**. Record incremental GPU time, game/render-thread time, p95/p99 frame time, VRAM residency, and update bursts—not just average FPS. Use Nanite’s instance, overdraw, and shading-bin visualizations to inspect unexpected costs. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details)

**Suggested implementation order:** establish one convincing stock-function cubemap room; connect several windows to one Rust room state; build the batch update path; add the atlas baker; then profile city-scale repetition. Do not let a sophisticated atlas tool or custom GPU buffer delay proving the simulation-to-window connection.

---

## 8. Source register and version applicability

| Source | What to use it for | Version / qualification |
| --- | --- | --- |
| [Unreal Engine 5.8 release announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available?utm_source=chatgpt.com) | Engine-version baseline | June 23, 2026 |
| [Interior Mapping paper — Joost van Dongen](https://www.proun-game.com/Oogst3D/CODING/InteriorMapping/InteriorMapping.pdf?utm_source=chatgpt.com) | Original algorithm, furniture planes, historical benchmarks | 2008; legacy hardware measurements |
| [Interior Mapping explanation — van Dongen](https://joostdevblog.blogspot.com/2018/09/interior-mapping-real-rooms-without.html?utm_source=chatgpt.com) | Technique explanation and qualified Spider-Man comparison | September 2018 |
| [Interior Mapping, Part 2 — Andrew Gotow](https://andrewgotow.com/2018/09/09/interior-mapping-part-2/?utm_source=chatgpt.com) | Tangent-space mapping and pre-projected atlas reference | 2018; incomplete Unity scaffolding, not UE code |
| [Interior Mapping — Harry Alisavakis](https://halisavakis.com/my-take-on-shaders-interior-mapping/?utm_source=chatgpt.com) | Compact cubemap implementation | 2019; tutorial code declared CC0 |
| [Three.js interior mapping repository](https://github.com/mohsenheydari/three-interior-mapping?utm_source=chatgpt.com) | Inspectable shader and runnable reference | MIT code; not a UE plugin |
| [Epic discussion: InteriorCubemap capture and scaling](https://forums.unrealengine.com/t/using-textures-on-the-new-interiorcubemap-mf/78464?utm_source=chatgpt.com) | Bake workflow and Robo Recall production reference | UE4-era, 2016–2017; regression-test in 5.8 |
| [Window Interior Cubemap Shader — Trey McNair](https://modsoft.net/projects/W290vD?album_id=1568317&utm_source=chatgpt.com) | Art setup, blinds, emissive and instance variation | Public UE demonstration; not confirmation of Spider-Man internals |
| [Building SimCity: Art in the Service of Simulation](https://www.gdcvault.com/play/1017823/Building-SimCity-Art-in-the?utm_source=chatgpt.com) | Production art and simulation communication | GDC 2013, Ocean Quigley / Maxis |
| [Cities: Skylines II building asset pipeline](https://cs2.paradoxwikis.com/Asset_Pipeline%3A_Buildings?utm_source=chatgpt.com) | Window UV and parallax-room authoring conventions | Indexed excerpts retrieved; complete current page unavailable |
| [UE Instanced Static Mesh documentation](https://dev.epicgames.com/documentation/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com) and [API](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent?utm_source=chatgpt.com) | ISM choice, custom data, update interfaces | UE5.8 |
| [Nanite overview](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?utm_source=chatgpt.com) and [technical details](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details?utm_source=chatgpt.com) | Material support, derivatives, tangents, UV indices, profiling | UE5.8 |
| [Lumen technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine?utm_source=chatgpt.com) and [performance guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine?utm_source=chatgpt.com) | Surface Cache, emissive pitfalls, scalability budgets | UE5.8 |
| [Microsoft texture block compression](https://learn.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression-in-direct3d-11?utm_source=chatgpt.com) | BC7/BC6H format properties and memory arithmetic | Established DXGI formats |
| [Rustonomicon: FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | ABI, ownership, and unwinding precautions | Current documentation |

**Bottom line:** TCE does not need a complex interior renderer to make its buildings feel inhabited. It needs a modest, stable shader; a reproducible room-baking pipeline; and a reliable mapping from simulated rooms to visible windows. Build that connection first, keep ordinary windows opaque, and spend geometric and lighting complexity only where the camera can justify it.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9296d-d7c8-83ea-9303-f0300bd2d966)
