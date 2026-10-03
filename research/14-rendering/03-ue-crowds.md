# Rendering 10,000+ citizens in Unreal Engine 5.8

**Engineering report for The Civilization Engine — evidence checked through September 27, 2026**

## Executive recommendation

**Use a hybrid representation system: persistent citizens in Rust, lightweight presentation entities in Mass, a tightly capped pool of conventional skeletal characters nearby, and GPU-instanced animation for the majority of visible people.**

For UE 5.8, my recommended first candidate is **native Instanced Skinned Meshes**, with **AnimToTexture’s bone-animation mode as the fallback**. Begin with low-polygon 3D characters at long distances; add animated impostors only when profiling demonstrates a worthwhile advantage.

There is an important version change: **UE 5.8, released June 23, 2026, introduced an experimental MetaHuman Crowds workflow that transitions between nearby individual actors and distant Instanced Skinned Meshes, orchestrated through Mass.** Advice based exclusively on UE 5.0–5.5’s City Sample/VAT architecture is therefore no longer sufficient. Epic nevertheless labels the new workflow experimental, not an unconditional production-ready solution. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

**Ten thousand simultaneously visible citizens is a reasonable prototype target, not a verified performance result for your hardware.** I found no controlled public benchmark establishing 10,000 varied citizens at 1440p60 on an RTX 4070 Ti in a comparably demanding UE city. No local Unreal benchmark was performed for this report.

The essential distinction is:

| Population measure | What TCE should support |
| --- | --- |
| Persistent simulated population | All 10,000–50,000 people, independent of camera position |
| Citizens with presentation state | Potentially the entire population, represented as compact data |
| Simultaneously visible citizens | Thousands, potentially 10,000+, predominantly inexpensive representations |
| Full skeletal actors with detailed animation and interaction | A bounded nearby subset—not the population |

At 60 fps, the frame interval is **16.67 ms**. Crowd rendering must share that interval with terrain, buildings, vegetation, lighting, the interface, and simulation. CPU and GPU work overlap; their timings should not simply be added into one serial budget.

---

## 1. Options: what each technique actually solves

### 1.1 Mass Entity and MassCrowd: organization, not a rendering shortcut

Mass Entity is a data-oriented framework: entities hold fragments, similar entities occupy archetype chunks, and processors operate on batches. This is useful for managing transforms, visual identities, animation requests, significance, and representation changes without creating a heavyweight object hierarchy for every citizen. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/overview-of-mass-entity-in-unreal-engine)

MassGameplay provides representation and LOD infrastructure, including actor pooling and switching among actors, instanced meshes, and no rendered representation. Its simulation LOD and representation LOD are separate concepts. **Using Mass does not itself eliminate animation evaluation, skinning, draw submission, material shading, or shadow costs.** [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/overview-of-mass-gameplay-in-unreal-engine)

For TCE, use Mass as a **presentation layer**, not a second civilization simulator. MassCrowd’s movement integration is optional: the current MetaHuman tutorial adds a `CrowdMember` trait for crowd navigation and optionally StateTree for behavior. That does not require TCE to surrender movement or decision ownership to Unreal. [Epic Games Developers](https://dev.epicgames.com/documentation/metahuman/create-metahuman-crowds-in-unreal-engine)

**Fit:** Strong for representation management. Less compelling as a replacement for an already-authoritative Rust simulation.

### 1.2 Conventional skeletal characters

Ordinary skeletal components provide the most flexible route for animation graphs, interactions, procedural pose adjustment, and modular character assembly. Their principal scaling problem is that pose sharing and modular assembly do not automatically consolidate rendering work: multiple components and material sections can remain expensive even when they share a pose. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)

**Recommendation:** Reserve them for selected citizens, close conversations, conspicuous work interactions, and other situations where inexpensive animation would visibly fail. A population-wide `ACharacter`/Animation Blueprint architecture should not be TCE’s starting point.

Two existing systems help stretch this nearby pool:

**Animation Sharing.** Characters are grouped into animation-state buckets and share evaluated poses, with facilities for transitions and variation. This reduces repeated animation evaluation, but it is **not equivalent to GPU draw instancing**. Many separately rendered skeletal components remain many separately rendered components. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-sharing-plugin-in-unreal-engine)

**Animation Budget Allocator.** This manages a game-thread animation budget by reducing update frequency, interpolating, or stopping updates according to significance. It uses registered `USkeletalMeshComponentBudgeted` components. It does not budget GPU geometry or shadows, and quality/tick constraints can prevent a perfectly hard CPU cap. Registration also replaces the component’s normal Update Rate Optimization handling; do not assume the two are independent optimization layers to stack blindly. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-budget-allocator-in-unreal-engine)

**Fit:** Excellent for a bounded nearby group; insufficient as the sole answer to 10,000 visible people.

### 1.3 UE 5.8 native Instanced Skinned Meshes

`UInstancedSkinnedMeshComponent` is now a concrete native API worth evaluating. It exposes batched instance creation, animation indices, per-instance custom data, current/previous transforms, culling controls, and bone attachments between instances. Animation is supplied through transform providers; `UAnimBankData` is one such provider. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedSkinnedMeshComponent?lang=en-US)

The new MetaHuman crowd workflow demonstrates the intended hybrid architecture: nearby characters use full actors and Animation Blueprints; distant characters use instanced meshes with pre-baked GPU animation. Its controls include LOD population caps, animation timing variation, and limits on concurrent blends. **Claims from earlier development discussions that blending would necessarily be absent in 5.8 are superseded by the current documentation.** [Epic Games Developers](https://dev.epicgames.com/documentation/metahuman/create-metahuman-crowds-in-unreal-engine)

This is not equivalent to running an unrestricted Animation Blueprint independently on every instance. Epic’s optimized distant MetaHumans omit post-process animation correctives, simplify mesh LODs, and replace strand hair with card meshes without hair physics. [Epic Games Developers](https://dev.epicgames.com/documentation/metahuman/metahuman-crowds-in-unreal-engine)

There is also a content-compatibility distinction: the MetaHuman collection pipeline expects MetaHuman character assets for heads/bodies, although clothing slots accept skeletal meshes. The generic ISKM component is a separate integration surface; arbitrary TCE rigs are not automatically drop-in replacements for the complete MetaHuman pipeline. [Epic Games Developers](https://dev.epicgames.com/documentation/metahuman/create-metahuman-crowds-in-unreal-engine)

**Fit:** First prototype candidate for UE 5.8. Potentially the lowest-maintenance long-term route, but acceptance must depend on packaged-build performance and actual TCE asset compatibility.

### 1.4 AnimToTexture: distinguish vertex animation from bone animation

“VAT crowds” often conflates two different storage strategies. Epic’s AnimToTexture plugin supports both vertex and bone modes. Its data assets expose vertex position/normal textures or bone position/rotation/weight textures. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/AnimToTexture?lang=en-US)

| Mode | Stored animation data | Main advantage | Main limitation |
| --- | --- | --- | --- |
| **Vertex animation textures** | Deformed vertex positions and normals across frames | Can reproduce baked deformation beyond ordinary skeletal motion | Storage grows with vertex count × frame count; tied closely to mesh topology |
| **Bone animation textures** | Bone transforms across frames, plus skinning information | Much smaller animation storage when bones are far fewer than vertices | Requires compatible skinning/bind-pose data; arbitrary procedural posing is not free |

The runtime character is still a **3D mesh**, not a billboard. A material reconstructs its animation, while instances provide placement and animation parameters. Precomputed bone palettes fetched by vertex shaders are an established skinned-instancing technique. [NVIDIA Developer](https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-2-animated-crowd-rendering)

For TCE, prefer **bone mode for ordinary humanoid locomotion and work loops**. Use true vertex animation selectively—for example, where a particular baked garment deformation materially improves the silhouette.

The important engineering consequence is that baking replaces part of the runtime animation workload with an asset pipeline. Validate every generated mesh LOD, animation mapping, texture format, and material in a packaged build. Treat precise hand placement, arbitrary IK, and complicated layered actions as near-character features unless explicitly implemented and tested in the cheaper backend.

**Fit:** Strong fallback for the bulk crowd. Less flexible than full skeletal animation, but easier to constrain than a custom GPU animation engine.

### 1.5 Third-party GPU skeletal systems

**TurboSequence** is an MIT-licensed UE plugin offering GPU-instanced skeletal crowds, including runtime bone adjustment, sockets, blend spaces, layered animation, LODs, and a hybrid mode with ordinary Unreal characters. Its repository documents UE 5.7-era integration, including a Niagara Nanite renderer option. [GitHub](https://github.com/LukasFratzl/TurboSequence)

However, the author describes it as a hobby project, explicitly places maintenance responsibility on users, and says it is not designed for MetaHumans. The stated **10k–50k** range is an intended workload range, **not a controlled frame-rate benchmark**. [GitHub](https://github.com/LukasFratzl/TurboSequence)

**Fit:** A contingency when native instancing lacks a genuinely necessary feature. For a solo developer, adopting its renderer and maintaining engine-version compatibility is a substantial commitment.

### 1.6 Animated impostors

Impostors replace distant geometry with view-dependent images, often storing multiple viewing directions and animation frames. The *Geopostors* research demonstrates hybrid geometry/impostor crowds with switching based on projected image detail. [ACM Digital Library](https://dl.acm.org/doi/10.1145/1053427.1053443)

They exchange geometry cost for texture storage, alpha overdraw, view-selection artifacts, and more complicated lighting. Their storage can multiply across:

\[
\text{view directions}\times\text{animation frames}\times\text{appearance variants}.
\]

**Fit:** Potentially useful for tiny distant figures. For TCE’s freely moving camera and varied clothing, start with aggressively simplified 3D meshes; introduce impostors only after measuring the alternative.

---

## 2. Trade-offs: where the frame time and memory go

### 2.1 Comparative assessment

The following is an engineering assessment, not a benchmark ranking.

| Technique | Principal saving | Remaining cost/risk | Complexity and maturity | Suggested TCE role |
| --- | --- | --- | --- | --- |
| Mass presentation entities | Object-management and batch-processing overhead | Does not solve GPU animation/shading | Moderate integration; several related systems must be understood | Presentation coordinator |
| Skeletal meshes + sharing/budgeting | Repeated pose evaluation and update frequency | Components, sections, skinning, shadows | Established engine facilities | Nearby characters |
| Native ISKM | Instanced submission and scalable animation representation | GPU deformation, materials, experimental workflow limitations | Moderate; new 5.8 crowd workflow requires validation | First bulk-renderer candidate |
| AnimToTexture bone mode | Per-character animation evaluation; compact baked animation | Vertex work, bake pipeline, limited procedural animation | Moderate; experimental plugin with older sample precedent | Bulk-renderer fallback |
| True vertex textures | Runtime deformation calculation | Potentially large textures and topology dependence | Moderate-to-high content burden | Selected baked effects |
| TurboSequence | Instanced skeletal rendering with added animation flexibility | Integration and maintenance ownership | High for a solo developer | Feature-driven contingency |
| Impostors | Geometry and skinning | Overdraw, atlas growth, lighting/view artifacts | High content complexity | Optional farthest tier |

The underlying capabilities and limitations are documented across Epic’s Mass, animation, ISKM, and AnimToTexture references; the role assignments above are my recommendation for TCE. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/overview-of-mass-gameplay-in-unreal-engine)

### 2.2 Instancing is not free geometry

Instancing primarily avoids submitting each character as an unrelated rendering object. It does not mean all instances share one execution of their vertex or pixel shading. NVIDIA’s crowd implementation explicitly found that its bottleneck could shift between pixel throughput and vertex processing. [NVIDIA Developer](https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-2-animated-crowd-rendering)

For scale, **10,000 characters × 2,000 triangles = 20 million triangles** before considering additional rendering passes. This arithmetic is not a performance prediction, but it explains why mesh LOD remains necessary even with excellent instancing.

Likewise, sampling a baked animation at 30 Hz does not automatically halve rendering cost: the mesh may still be transformed and rasterized on every rendered frame.

### 2.3 Animation memory: an illustrative comparison

Consider a hypothetical animation library containing 1,800 sampled frames:

| Assumed representation | Calculation | Animation-data size |
| --- | --- | --- |
| 5,000 vertices, 16 bytes per vertex-frame across position/normal data | 5,000 × 1,800 × 16 | **144 MB ≈ 137 MiB** |
| 60 bones, 32 bytes per bone-frame | 60 × 1,800 × 32 | **3.46 MB ≈ 3.30 MiB** |

**These are assumed record formats, not measured AnimToTexture allocations.** They exclude skin weights, texture padding, compression choices, additional LODs, and material textures.

The comparison illustrates why TCE should avoid separately baking every clothing combination into large vertex-animation datasets. With 12 GB of VRAM shared by the entire city, appearance textures, duplicated mesh variants, and animation libraries need explicit residency budgets.

### 2.4 Moving crowds: prefer ISM over HISM for the VAT route

Epic distinguishes HISM’s relatively static hierarchy from ISM’s suitability for changing instances. Modern ISMs also support per-instance LOD; the older simplification that every ISM instance must share one LOD is no longer generally correct. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

For moving VAT citizens, start with **ISMs**, organized into a manageable number of spatial and rendering groups. Do not use one component per citizen, but also do not assume one enormous world-spanning component is automatically optimal.

A useful proposed grouping key is:

```
mesh variant + material set + animation library + spatial region + shadow policy
```

Per-instance phase, tint, or compatible animation selection should remain data rather than creating new materials or components wherever the backend permits.

### 2.5 Shadows, ray tracing, and temporal reconstruction

**Shadows can erase the savings from cheap animation.** Moving skeletal geometry and World Position Offset can invalidate Virtual Shadow Map cache pages; oversized bounds and low-angle lighting make this worse. Reducing animation-update frequency does not eliminate invalidation caused by character movement. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

My recommended policy is detailed dynamic shadows for nearby citizens, cheaper or shorter-range shadows for intermediate citizens, and no individually detailed shadowing for the smallest figures. Test the visual result rather than disabling all crowd shadows globally.

Hardware ray tracing adds another budget. Epic identifies dynamic skeletal geometry updates as a cost, and warns that enabling WPO evaluation for instanced static meshes can create expensive per-instance acceleration-structure work. **A VAT-deformed raster image does not automatically imply an equally correct and inexpensive ray-traced representation.** [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/ray-tracing-performance-guide-in-unreal-engine)

Treat distant-crowd inclusion in hardware-ray-traced effects as optional. Check reflection and shadow mismatches before excluding it.

Finally, preserve previous-frame animation state. Epic’s `Previous Frame Switch` exists specifically to help generate motion vectors for parameter-driven vertex animation. Incorrect deformation velocities can compromise temporal antialiasing and motion blur. For instanced skinned meshes, the native API also exposes previous transforms. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/utility-material-expressions-in-unreal-engine)

---

## 3. Precedents and benchmarks: what they actually establish

### 3.1 Measurements and production examples

| Precedent | Reported result or architecture | Evidential limit | Lesson for TCE |
| --- | --- | --- | --- |
| **NVIDIA, “Animated Crowd Rendering,” GPU Gems 3, 2007** | **9,547 characters at approximately 34 fps**, Core 2 Duo 2.93 GHz, 2 GB RAM, GeForce 8800 GTX, **1280×1024**. Used LODs and 160 instanced draws versus 59,726 individual draws in its comparison. | Controlled historical rendering sample, not UE5, modern lighting, or a complete city simulation. | The core combination of skinned instancing, variation, and LOD is well established. Do not extrapolate its fps to modern TCE content. [NVIDIA Developer](https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-2-animated-crowd-rendering) |
| **Hitman: Absolution — IO Interactive, GDC 2012** | Developer reports **1,200-character crowds at 30 fps on contemporary consoles**, with individual interaction and behavior influence; examples came from a production level. | The abstract does not provide a comparable resolution, GPU breakdown, or TCE-style persistent simulation workload. | Rich interaction and crowd scale can coexist when designed together, but this is not evidence for thousands of unrestricted character actors. [GDC Vault](https://www.gdcvault.com/play/1015315/Crowds-in-Hitman) |
| **Assassin’s Creed Unity — Ubisoft, GDC 2015** | Technical slides describe spatial activation, asynchronous spawning, pooled entities, and differentiated treatment of NPCs and crowd stations. | Spawned and network-replicated counts in the talk are not interchangeable with total rendered crowd counts. No comparable 1440p60 result is established here. | Pooling and separating activity/representation responsibilities matter as much as the skinning method. [GDC Vault](https://media.gdcvault.com/gdc2015/presentations/Lefebvre_Charles_NetworkingGameplayAndAI.pdf) |
| **City Sample / The Matrix Awakens** | Thousands of varied digital humans; nearby fully rigged skeletal characters and distant vertex-animated static meshes, coordinated with Mass. | Technology demonstration/sample, not a shipped full civilization game or a controlled 10k-visible/60-fps test. | Direct Unreal precedent for a hybrid skeletal/instanced crowd. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration) |
| **UE 5.8 MetaHuman Crowd Sample, 2026** | Epic provides a crowd scene and asset pipeline targeting thousands of varied MetaHumans, with actor/ISKM transitions. | Experimental; the public listing does not establish your hardware/resolution/frame-rate combination. | Best current official starting point for a reproducible native-instancing prototype. [Fab.com](https://www.fab.com/listings/5f481d73-afb1-4d94-ba6e-7cabf5d296fa) |

The key conclusion is **architectural evidence is much stronger than hardware-matched performance evidence**. These examples justify the hybrid approach; they do not determine how many citizens TCE can afford with its own clothing, lighting, camera, and buildings.

### 3.2 Research on perceived variety

*Clone Attack! Perception of Crowd Variety* found that appearance repetition was more readily noticed than repeated motion in its experiments. Color variation, orientation, spatial separation, and motion differences affected clone detection. This is a useful prioritization guide, although the older experimental stimuli should not be treated as a universal threshold for modern realistic graphics. [Computer Science & Stats School](https://www.scss.tcd.ie/rachel.mcdonnell/papers/Siggraph08.pdf)

For TCE, my interpretation is: **invest first in recognizable silhouettes, clothing palettes, hair/head variety, and believable activity distribution—not hundreds of barely distinguishable walking clips.**

*Geopostors* supplies the complementary rendering principle: use detailed geometry only where its projected contribution justifies it, and switch to cheaper representations farther away. [ACM Digital Library](https://dl.acm.org/doi/10.1145/1053427.1053443)

### 3.3 Useful public code

**MassSample** is an MIT-licensed educational reference for Mass processors, fragments, and integration patterns. Its repository reports a UE 5.7 code update while warning that portions of the README still contain older API examples; it also requires Git LFS. Treat compiling source as more reliable than copied tutorial snippets, and do not assume 5.8 compatibility without testing. [GitHub](https://github.com/Megafunk/MassSample)

**AnimToTextureHelpers** provides practical baking and instancing examples, with documented updates through UE 5.4-era content. It is useful as a pipeline reference, not proof that its assets and scripts work unchanged in 5.8. [GitHub](https://github.com/kromond/AnimToTextureHelpers)

**TurboSequence** is the more ambitious renderer reference discussed above. Its additional animation flexibility is relevant, but it should not be confused with a supported Epic subsystem. [GitHub](https://github.com/LukasFratzl/TurboSequence)

---

## 4. Recommended TCE architecture

### 4.1 Keep simulation authority in Rust

I recommend this division:

```
Rust simulation: persistent citizens, jobs, routes, possessions, actions
                              |
          versioned snapshots + significant state-change events
                              |
                  C++ presentation adapter
                              |
       Mass: identity, transform, appearance, animation, significance
                              |
          +-------------------+---------------------+
          |                   |                     |
   Pooled skeletal       Instanced skinned      Very-low-LOD mesh
   nearby characters     or bone-texture mesh   / optional impostor
```

The critical rule is **one owner for each kind of state**.

Rust should own citizen identity, authoritative position/route, activity, and the consequences of actions. Unreal should interpolate movement, choose representation, animate visual actions, and report explicit interaction commands.

Do not run an independent MassCrowd movement simulation that silently disagrees with Rust about where a citizen is. Either keep local avoidance authoritative in Rust, or define an explicit feedback contract. Small cosmetic offsets can be presentation-only, but they must not determine who reached a workplace or delivered food.

Similarly, animation notifies should not be the authoritative source of economic events. The simulation decides that grain was transferred; animation depicts that event.

For procedural settlements, include interaction anchors in building-kit metadata: work position, facing direction, hand/tool target, seat height, and approach location. This keeps farming, crafting, carrying, and conversation tied to the actual world rather than producing a crowd that only walks randomly.

### 4.2 Design the DLL boundary for batches

Use a versioned C ABI with explicit layouts, ownership, and error handling. Rust’s FFI guidance is relevant here: native Rust containers and unwinding behavior should not be treated as a stable C interface. Unreal’s third-party integration documentation covers DLL loading and packaging/staging requirements. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Recommended contract:

* Exchange caller-owned buffers or immutable snapshots containing plain records; avoid per-citizen FFI calls.
* Map stable citizen IDs, preferably with generation information, to presentation handles.
* Keep `UObject` ownership on the Unreal side; stop worker activity before unloading the DLL.

For scale, an **assumed 64-byte presentation record × 50,000 citizens is 3.2 MB**. Publishing it at 20 Hz would transfer **64 MB/s**, before overhead. This is illustrative, not a measured bridge cost. It suggests that batching and synchronization are more important first concerns than attempting exotic zero-copy machinery.

Start with presentation snapshots around **10–30 Hz**, then interpolate at rendering frequency. This is a proposed starting range, not a requirement for the Rust simulation’s internal tick rate.

Do not let the game thread wait for an entire long simulation update. Avoid oversubscribing CPU cores with competing Rust and Unreal worker pools. Preserve a short, explicit presentation timeline; do not feed centuries of accumulated time into single-precision animation-phase calculations.

### 4.3 Use screen-space LOD, actor caps, and transition hysteresis

The following values are **initial tuning hypotheses**, not engine limits or measured 4070 Ti capacities.

| Tier | Approximate projected character height at 1440p output | Proposed representation | Initial population/quality policy |
| --- | --- | --- | --- |
| Selected / very close | Above roughly 200 pixels | Full skeletal actor | About **32–64 maximum**; detailed interaction where needed |
| Nearby | Roughly 80–200 pixels | Simplified skeletal actor or ISKM | Keep **total conventional skeletal actors around 128–192 initially**, including the closest tier |
| Intermediate | Roughly 20–80 pixels | Native ISKM or bone-texture ISM | Roughly **1,000–3,000 triangles per character**, few material sections |
| Distant | Roughly 5–20 pixels | Aggressively simplified instanced mesh | Roughly **150–500 triangles**; inexpensive shading and restricted shadows |
| Tiny / hidden | Below roughly 5 pixels, or genuinely occluded | Cull visually, or optional impostor | Preserve the citizen and simulation state |

These thresholds need visual validation at the selected internal rendering resolution. A street-level camera looking at a dense crowd is much harder than an aerial camera where most people occupy only a few pixels.

Add hysteresis so citizens do not repeatedly change representation near a threshold. Spread promotions and demotions across frames. Preserve appearance, animation phase, facing, and carried objects across transitions.

**A cap should demote representation quality, not randomly delete conspicuous citizens.** The selected citizen can receive priority, but selection must not change their simulated behavior.

At extreme time acceleration, explicitly choose a visualization policy—sampled trajectories, fewer visible animation events, or more aggressive visual LOD—rather than attempting to render every intermediate action.

### 4.4 Build variation around shared assets

Persist appearance independently from rendering:

```
appearance seed
+ body/rig family
+ head and hair selection
+ garment inventory
+ palette and pattern choices
+ age/condition parameters
```

Recreating a render instance must not give a citizen a new face or outfit.

Use a small set of compatible rig families and curated garment silhouettes, then expand variety through palette masks, patterns, dirt, wear, and accessories. Both ISM and native ISKM expose per-instance custom data, making such parameters preferable to thousands of unique material instances. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

There are two useful clothing strategies:

| Strategy | Benefit | Cost |
| --- | --- | --- |
| **Prebuilt or cached merged outfits** | Fewer components/sections; predictable assets | More mesh variants and preprocessing |
| **Individually instanced body/clothing pieces** | Reuse across combinations | More rendering groups, parts, and synchronization |

Epic’s modular-character guidance makes an important distinction: Leader Pose reduces repeated pose work but not necessarily render-thread cost; skeletal merging incurs setup work, and reducing material sections still requires deliberate material/atlas organization. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)

For TCE, I would use **curated merged combinations for common outfits**, with modular instancing where it demonstrably saves asset duplication. Do not generate and retain a unique merged mesh for every citizen.

Also:

* Use reduced hair geometry away from the camera; reserve simulated cloth and detailed facial work for the nearest tier.
* Include carrying, seated work, tool use, and social poses in the initial animation library—not just locomotion.
* Make available clothing depend on the simulated economy and discovered techniques, while keeping the rendering mechanism independent of historical “eras.”

For tools and baskets, evaluate the native ISKM bone-attachment API or AnimToTexture’s rigid bone-binding support instead of introducing thousands of separately ticked attachment actors. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedSkinnedMeshComponent?lang=en-US)

### 4.5 Initial budgets for the target machine

These are **proposed acceptance budgets**, not benchmark results:

| Resource | Initial target |
| --- | --- |
| Incremental crowd GPU cost, including crowd-induced shadow work | **3–4 ms** in the agreed target scene |
| Presentation bridge, Mass processing, and instance updates | **Around 1 ms on the game-thread critical path** |
| Nearby skeletal animation | Begin with **approximately 1–1.5 ms allocator target**, then measure actual critical-path effects |
| Resident crowd assets | Begin with **1–2 GB VRAM allowance** |
| Whole application steady-state VRAM | Aim below approximately **10 GB**, leaving headroom on the 12 GB card |
| Full skeletal population | Begin around **128–192**, not thousands |

The crowd GPU measurement must compare the **same world and camera with and without the crowd**. A blank-plane benchmark is useful for diagnosis but cannot establish the final city budget.

For Lumen, start from a deliberately chosen scalable configuration rather than “Epic everything.” Epic’s performance guide targets 60-fps console budgets at High and 30-fps budgets at Epic, with internal-resolution assumptions; those are configuration reference points, not 4070 Ti guarantees. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine)

Report native 1440p and upscaled 1440p separately. Disable frame generation when determining whether the application actually sustains 60 rendered frames per second.

### 4.6 The implementation decision process

**First, prove the renderer without the simulation.** Replay deterministic recorded or synthetic trajectories so a rendering regression cannot be confused with pathfinding or Rust scheduling.

Start with the UE 5.8 MetaHuman Crowd Sample’s `MAP_Plaza`, inspect its LOD/animation configuration, and then substitute representative TCE clothing, silhouettes, actions, and lighting. The sample tutorial identifies the supplied crowd configuration and animation assets for this purpose. [Epic Games Developers](https://dev.epicgames.com/documentation/metahuman/metahuman-crowd-sample-tutorial)

**Second, make native ISKM pass a specific gate.** Require correct packaged-build behavior for TCE’s rigs, clothing changes, carried objects, animation transitions, pause/speed control, current/previous-frame motion, and repeated actor/instance switching.

**Third, choose one bulk renderer.** If native instancing fails an essential requirement or consumes disproportionate engineering effort, implement the bone-texture fallback using the same source rigs and animations. Do not maintain several complete rendering systems indefinitely. Consider TurboSequence only when a missing feature justifies its maintenance cost.

**Fourth, integrate Rust and automate regression testing.** Test births, deaths, outfit changes, camera teleports, large migrations, and long-running sessions—not only steady walking.

A useful test matrix is:

| Dimension | Required cases |
| --- | --- |
| Visible population | 1k, 5k, 10k; 20k as stress testing |
| Persistent population | Up to 50k in Rust regardless of visible count |
| Camera | Aerial city, crowded street, market, fast zoom, teleport |
| Lighting | Midday and low-angle sun; shadows enabled/disabled |
| Appearance | One repeated outfit versus a realistic varied wardrobe |
| Actions | Walking, turning, carrying, seated work, clustered interactions |
| Churn | Spawning, despawning representations, changing outfits, repeated promotions |

Capture game-thread, render-thread, RHI, and GPU timings; animation/representation counts; memory residency; and frame-time percentiles rather than average fps alone. Unreal Insights supplies CPU/GPU tracing, memory investigation, and asset-loading analysis. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

Pin the exact UE patch, plugin revisions, GPU driver, internal resolution, and i9 model. For AI coding agents, require compile-tested code against those pinned headers, generated-asset validation, and packaged-build regression tests. Older Mass snippets are particularly hazardous because even the community sample warns of outdated documentation. [GitHub](https://github.com/Megafunk/MassSample)

---

## 5. Source guide and version applicability

The links below are primary documentation, author-maintained code, original research, or developer talks.

### Current Unreal implementation references

| Source | Version relevance |
| --- | --- |
| [UE 5.8 release announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available?utm_source=chatgpt.com) | June 23, 2026; establishes the new crowd workflow |
| [Mass Entity overview](https://dev.epicgames.com/documentation/unreal-engine/overview-of-mass-entity-in-unreal-engine?utm_source=chatgpt.com) and [MassGameplay overview](https://dev.epicgames.com/documentation/unreal-engine/overview-of-mass-gameplay-in-unreal-engine?utm_source=chatgpt.com) | Current documentation; distinguish core Mass from particular representation integrations |
| [Instanced Skinned Mesh API](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UInstancedSkinnedMeshComponent?lang=en-US&utm_source=chatgpt.com) and [AnimBank API](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UAnimBankData?utm_source=chatgpt.com) | UE 5.8 API surface |
| [MetaHuman Crowds overview](https://dev.epicgames.com/documentation/metahuman/metahuman-crowds-in-unreal-engine?utm_source=chatgpt.com), [creation guide](https://dev.epicgames.com/documentation/metahuman/create-metahuman-crowds-in-unreal-engine?utm_source=chatgpt.com), and [sample tutorial](https://dev.epicgames.com/documentation/metahuman/metahuman-crowd-sample-tutorial?utm_source=chatgpt.com) | New experimental 2026 workflow |
| [MetaHuman Crowd Sample on Fab](https://www.fab.com/listings/5f481d73-afb1-4d94-ba6e-7cabf5d296fa?utm_source=chatgpt.com) | Official sample; listing also links the 2026 scalable-crowds talk |
| [AnimToTexture API](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/AnimToTexture?lang=en-US&utm_source=chatgpt.com) and [data asset reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/AnimToTexture/UAnimToTextureDataAsset?utm_source=chatgpt.com) | Current API; plugin remains under Experimental |
| [Animation Budget Allocator](https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-budget-allocator-in-unreal-engine?utm_source=chatgpt.com) and [Animation Sharing](https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-sharing-plugin-in-unreal-engine?utm_source=chatgpt.com) | Existing systems applicable to the nearby skeletal tier |
| [Modular characters](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine?utm_source=chatgpt.com) and [ISM components](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com) | Asset/component trade-offs; use current ISM/HISM guidance |
| [Virtual Shadow Maps](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine?utm_source=chatgpt.com), [ray-tracing performance](https://dev.epicgames.com/documentation/en-us/unreal-engine/ray-tracing-performance-guide-in-unreal-engine?utm_source=chatgpt.com), and [Lumen performance](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine?utm_source=chatgpt.com) | Whole-frame integration and hidden crowd costs |
| [Unreal Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine?utm_source=chatgpt.com), [third-party DLL integration](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com), and [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Profiling and Rust/Unreal boundary |

### Precedents, papers, and code

| Source | Date/version and use |
| --- | --- |
| [GPU Gems 3: Animated Crowd Rendering](https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-2-animated-crowd-rendering?utm_source=chatgpt.com) | 2007; explicit historical hardware benchmark and skinned-instancing implementation |
| [Crowds in Hitman: Absolution](https://www.gdcvault.com/play/1015315/Crowds-in-Hitman?utm_source=chatgpt.com) | GDC 2012; production crowd report |
| [Assassin’s Creed Unity technical slides](https://media.gdcvault.com/gdc2015/presentations/Lefebvre_Charles_NetworkingGameplayAndAI.pdf?utm_source=chatgpt.com) | GDC 2015; streaming, pooling, and differentiated NPC treatment |
| [City Sample documentation](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration?utm_source=chatgpt.com) | Earlier UE5 hybrid crowd precedent; not the same backend as the new 5.8 workflow |
| [Geopostors](https://dl.acm.org/doi/10.1145/1053427.1053443?utm_source=chatgpt.com) | 2005; geometry/impostor crowd research |
| [Clone Attack!](https://www.scss.tcd.ie/rachel.mcdonnell/papers/Siggraph08.pdf?utm_source=chatgpt.com) | 2008; perception of appearance and motion repetition |
| [MassSample](https://github.com/Megafunk/MassSample?utm_source=chatgpt.com) | Repository documents a 5.7 code update; README partly older |
| [AnimToTextureHelpers](https://github.com/kromond/AnimToTextureHelpers?utm_source=chatgpt.com) | Documented 5.4-era updates; baking/reference implementation |
| [TurboSequence](https://github.com/LukasFratzl/TurboSequence?utm_source=chatgpt.com) | Repository documents 5.7-era features; validate independently for 5.8 |

## Bottom line

For TCE, **make every citizen persistent, but make visual fidelity a budgeted resource**.

The best-fit plan is **Rust authority → Mass presentation → a small skeletal pool → native UE 5.8 skinned instancing for the majority**, with **bone-texture animation as a practical fallback**. Preserve identity and action continuity across every transition.

The decisive prototype is not “10,000 identical walkers on a plane.” It is **10,000 appropriately LODed citizens, wearing representative clothing, performing representative activities, inside a representative city, with shadows and temporal reconstruction enabled**. Passing that test—not a raw instance-count demonstration—is what establishes the 1440p60 target.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92968-8538-83ea-9928-52e54638de1f)
