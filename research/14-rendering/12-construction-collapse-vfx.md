# Visualizing construction and cosmetic collapses in TCE

**Recommendation:** Render construction and completed buildings primarily as **opaque, Nanite-enabled instanced static meshes**. Generate scaffolding from building metadata. When a visible collapse occurs, temporarily replace selected modules with **coarse animated pieces or a tightly budgeted pool of pre-fractured Chaos Geometry Collections**. Finish by replacing the effect with **persistent, non-simulated rubble and surviving structural modules**.

The crucial separation is:

> **Rust determines what exists, what fails, and what remains. Unreal determines how that transition looks.**

That gives TCE unscripted history without requiring every historical event to leave an indefinitely simulated pile of rigid bodies.

**Version basis:** UE **5.8** documentation checked on **September 27, 2026**. Historical talks, code and research are dated separately below. I have not benchmarked TCE or run these techniques on your machine; proposed budgets are starting targets, not measured performance.

---

## 1. Options and how they work

### 1.1 Technique comparison

The suitability and complexity assessments below are engineering judgments for TCE, rather than Epic performance guarantees.

| Technique | How it works | Main advantage | Main limitation | TCE role |
| --- | --- | --- | --- | --- |
| **Discrete instanced construction modules** | Add completed posts, wall sections, beams and roof patches to ISM components | Matches arbitrary building layouts and actual construction tasks | Requires sensible module boundaries and instance bookkeeping | **Default construction representation** |
| **Material-based reveal** | A shader clips or reveals geometry using construction progress | Smooth transitions with little additional geometry authoring | Appearance changes do not remove collision or create real cut surfaces | Brief transitions on nearby, actively worked modules |
| **Whole-building stage variants** | Swap foundation, frame, unfinished and complete meshes | Straightforward for a small prefab catalog | Variant count grows rapidly with runtime architectural combinations | Small standardized structures only |
| **Animated module collapse** | Move intact or coarse broken modules along authored transform curves | Predictable cost and easy artistic control | Limited interaction with terrain and neighboring debris | **Default inexpensive collapse** |
| **Pre-fractured Chaos Geometry Collections** | Release clustered rigid fragments when failure occurs | Responsive close-up motion, collisions and secondary breaking | Physics activation, contacts, memory and lifecycle complexity | Selected nearby collapses |
| **Chaos Cache playback** | Replay previously recorded destruction | Repeatable motion without solving the original collapse again | Recorded topology and motion constrain reuse; cache integration requires validation | Optional effect tier |
| **Runtime fracture of arbitrary assemblies** | Generate new cuts, fragment meshes and associated collision during play | Maximum freedom in fracture location | Much larger implementation and performance risk | Do not make this a TCE dependency |

UE documents ISM-based rendering, pre-fractured destruction, and Nanite support for Geometry Collections. The important maturity caveat is that **`AChaosCacheManager` is still explicitly marked `Experimental` in the UE 5.8 API**. Treat caching as an optional optimization, not the only working collapse path. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

### 1.2 Construction: reveal meaningful assemblies, not individual bricks

For TCE, the visible construction unit should usually be a **structural or work-sized assembly**:

* A post, beam, truss or floor panel.
* A wall bay or horizontal band of masonry.
* A roof patch, window assembly or section of finished plaster.

This is a proposed representation, not a requirement to make simulation work units equally coarse. Rust could account for individual material deliveries and worker-hours while Unreal reveals a wall band after several tasks complete.

Author each building kit with a **construction dependency graph**. For example:

```
prepared ground
    → foundations
    → load-bearing walls or frame
    → supported floors and roof structure
    → roof covering and infill
    → openings, finishes and fittings
```

Do not impose this as one universal sequence. A timber frame, an earthen wall and a masonry arch should have different dependency graphs and temporary-support requirements. Progress should be **per task or module**, allowing one wing to be roofed while another remains unfinished.

**Recommended rendering arrangement.** Group instances by spatial cell and rendering configuration:

```
(cell, mesh asset, material set, collision policy, shadow policy)
    → ISM component
```

Do not make one Actor per brick or one independently ticking component per construction task. Keep a separate map from stable TCE module identifiers to their current rendering handles.

For Nanite-only content, Epic specifically recommends **ISM rather than HISM**, since Nanite supplies its own culling and detail selection. HISM remains worth testing for large, genuinely static, non-Nanite populations or fallback paths. Many ISM properties—including collision and shadow settings—are component-level, which is why the grouping key needs more than the mesh asset alone. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

**Use actual instance insertion as the default reveal.** A module that has not been built should normally have no rendered instance. This also makes abandonment, partial demolition, rebuilding and material substitution straightforward.

For nearby construction, optionally animate a short transition using per-instance data. For example, reveal the final strip of a wall band while workers are placing it, then move the completed module into the ordinary opaque rendering configuration. UE exposes per-instance custom data without requiring a unique dynamic material instance for every module. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

**Avoid a permanent “invisible finished building.”** Keeping all future geometry present and clipping it with a progress shader introduces several avoidable problems: the collision representation needs separate handling, clipping does not generate a physical cap, and material-driven displacement can disagree with other scene representations. Nanite supports masked materials, but Lumen’s software tracing does not support WPO and has its own geometry/material representation restrictions. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US)

### 1.3 Generate scaffolding from construction metadata

I recommend a **deterministic scaffold generator**, not a second physical simulation and not a mesh-analysis problem performed every frame.

**Geometry generation.** Start with exposed construction boundaries or authored scaffold sockets. Offset the working line from the wall, divide it into kit-sized bays, and add vertical working levels only where current tasks require access. Resolve corners, entrances and terrain contacts using explicit rules. A foot on sloping terrain can select a longer support variant or adjusted base rather than deforming the entire scaffold.

**Kit composition.** Assemble reusable supports, ledgers, braces, platforms, ladders and ties as instances. Select the scaffold family through available materials and construction capabilities; do not automatically introduce modern steel scaffolding into an early agrarian settlement. A “complete scaffold bay” mesh can replace its constituent pieces at intermediate distances.

**Lifecycle.** Derive scaffolding from the active work front and access plan. Recompute affected edges when a storey, wing or task changes—not the whole settlement every frame. Keep construction materials, hoists and temporary supports as distinct categories so they need not disappear together.

Important implementation details:

| Problem | Proposed rule |
| --- | --- |
| Adjacent wall modules generate duplicate poles | Give scaffold edges and junctions stable keys; deduplicate before instancing |
| A doorway becomes blocked | Mark access exclusion intervals in building metadata |
| An interrupted project loses all scaffolding | Scaffold persistence follows abandonment/upkeep rules, not worker presence alone |
| Thousands of poles acquire physics bodies | Use no physics for ordinary scaffold detail |
| Workers need to stand on platforms | Provide simplified work surfaces/access links, rather than collision on every board |
| Distant scaffolding creates visual noise | Reduce to coarse bays or omit sub-pixel detail using a renderer-owned distance policy |

The generator’s work should scale with **affected exposed bays and working levels**, not with the total number of buildings in the world. This is particularly suitable for a solo developer because most complexity lives in inspectable kit metadata and deterministic rules.

### 1.4 Chaos: pre-fracture reusable modules, not every generated building

A Geometry Collection contains geometry, a transform hierarchy and destruction-related data. Chaos clustering lets several pieces behave together until their connections break, after which smaller bodies can move independently. This provides a natural hierarchy such as **wall section → large chunks → smaller fragments**. [dev.epicgames.com](https://dev.epicgames.com/documentation/unreal-engine/destruction-overview?lang=en-US)

For runtime-assembled architecture, author a reusable destruction catalog:

```
wall_bay_A
    intact mesh
    fractured Geometry Collection
    coarse collapse representation
    surviving-wall variants
    rubble family
```

A building assembled from thirty bays can then select the relevant prototypes at collapse time. It does not need a bespoke Geometry Collection asset for every possible floor plan.

**Authoring pipeline.** Use editor-side fracture tools or Dataflow to produce cooked assets. Epic’s Dataflow workflow supports graph-based conversion, point scattering and fracture operations, making it a useful foundation for repeatable asset preparation. For AI-assisted development, place mesh selection, fracture seed and validation rules in manifests and wrap the asset-build steps in project-owned tooling. Do not assume an editor tutorial is a supported packaged-runtime fracture API. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/dataflow-for-destruction-quickstart)

Prepare the following deliberately:

| Asset concern | Recommended treatment |
| --- | --- |
| Exterior and exposed fracture surfaces | Assign compatible exterior/interior materials |
| Fragment density | Concentrate detail where it changes the close-up silhouette |
| Collision | Use a small number of simple, well-fitting shapes per important body |
| Hierarchy | Keep useful large clusters; avoid releasing every smallest fragment immediately |
| Placement | Standardize pivots, dimensions and transforms across intact and fractured assets |
| Initial overlap | Validate collision shapes against neighboring modules and terrain |

Epic’s Geometry Collection guide recommends watertight source geometry and warns that intersecting geometry can separate violently during simulation. It also notes that a collection created from multiple selected actors takes its pivot from the first selected actor—an easy source of instance-to-collection alignment errors. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/geometry-collections-user-guide?lang=en-US)

**Nanite helps rendering, not the rigid-body solve.** It supports Geometry Collections, including their rigid motion, but it does not remove the costs of body initialization, collision detection, contacts, constraints or event processing. Keep visual geometric detail and physics detail independently budgeted. Epic also explicitly retains practical Nanite limits involving instance count, material complexity and output resolution. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US)

### 1.5 Modular connections are not automatic structural engineering

Do not assume two touching Geometry Collections form a physically meaningful building.

UE 5.8 does expose **`UClusterUnionComponent`**, whose component membership can be specified dynamically at runtime. It is an option for assemblies that genuinely need to behave as connected physics structures. That capability is materially different from merely placing collections next to one another. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UClusterUnionComponent)

However, TCE does not need to make cluster unions its first implementation. Epic’s modular-destruction guidance distinguishes anchor/connectivity behavior from real structural integrity: its example notes that a connected structure can remain standing through an implausibly small surviving attachment. That discussion is from **2025/UE 5.5-era guidance**, not evidence that every associated issue remains unchanged in 5.8. [Epic Developer Community Forums](https://forums.unrealengine.com/t/looking-for-an-advanced-workflow-for-chaos-destruction-on-a-building/2315780)

For TCE, use Rust’s failure decision to specify which modules lose support. Unreal can then release or animate those modules in sequence. **Do not ask cosmetic Chaos simulation to discover the authoritative engineering outcome.**

---

## 2. Trade-offs, performance limits and benchmarks

### 2.1 What will actually become expensive?

There is no defensible universal rule such as “Chaos supports 10,000 fragments at 60 fps.” A fragment count omits too much: how many bodies are awake, collision-shape complexity, pile density, solver work, material count, screen coverage, and the rest of the game.

For TCE, track at least:

| Cost category | Useful measurements | Primary mitigation |
| --- | --- | --- |
| Representation changes | Game-thread time, allocations, component registration, instance uploads | Batch updates; pool effects; avoid whole-cell rebuilds |
| Physics | Instantiated and awake bodies, shapes, contacts, constraints, solver time | Coarse collision, clustering, short lifetimes |
| Rendering | GPU time, visible fragment count, material/shadow passes | Nanite, shared materials, distance tiers |
| Dust and chips | Niagara systems/emitters, particle count, translucent screen coverage | Consolidate effects; cap coverage and significance |
| Persistence | Resident assets, rubble instances, saved cosmetic state | Static rubble, spatial streaming, compact descriptors |

The distinction between **fracture hierarchy size** and **bodies instantiated for simulation** matters. UE provides separate `MaxClusterLevel` and `MaxSimulatedLevel` controls; they should not be treated as interchangeable fragment-count limits. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/GeometryCollectionEngine/UGeometryCollectionComponent)

**Shadows can dominate the apparent destruction cost.** Virtual Shadow Maps invalidate affected cached pages when shadow-casting geometry moves or is added or removed. Large bounds and low-angle lighting can magnify the work. A collapse should therefore be profiled under morning/evening lighting as well as midday, with small debris shadow casting disabled where its absence is not noticeable. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine)

**GPU particles are not CPU-free.** Niagara still executes system/emitter work on the CPU, and many separate systems introduce overhead. Epic recommends Effect Types, instance-count control, pooling and, where appropriate, adding particles to existing systems rather than spawning many new systems. Pool priming and large single-frame bursts can themselves hitch. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/scalability-and-best-practices-for-niagara)

### 2.2 Published quantitative evidence

| Source | Reported result | What it establishes—and what it does not |
| --- | --- | --- |
| **Epic ISM guide, current documentation** | Approximately **672 bytes per GPU primitive versus 64 bytes per basic instance** | Evidence for lower instance bookkeeping cost. **Not** total object memory, collision memory or an FPS benchmark. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine) |
| **Müller, Chentanez and Kim, 2013** | Arena example: approximately **1 million vertices, 500,000 faces**, eventually **20,000 separate pieces**, reported above **30 fps** on a **3.07 GHz Core i7 and GTX 680** | A custom research implementation using a **GPU rigid-body solver**, not Chaos or modern UE rendering. It cannot predict TCE performance. [ResearchGate](https://www.researchgate.net/publication/260398688_Real_Time_Dynamic_Fracture_with_Volumetric_Approximate_Convex_Decompositions) |
| **Same 2013 paper** | Fracture operations for smaller objects were generally below **10 ms**; worst reported arena fracture events remained below **50 ms** | These are **per-fracture-event costs**, not an acceptable 60-fps frame budget. The authors explicitly distinguish a physics demonstration from a game sharing resources with other systems. [ResearchGate](https://www.researchgate.net/publication/260398688_Real_Time_Dynamic_Fracture_with_Volumetric_Approximate_Convex_Decompositions) |

The paper also avoids full rigid bodies below a size threshold, using a separate debris representation. That is a directly useful architectural precedent, even though its numerical performance is not transferable. [ResearchGate](https://www.researchgate.net/publication/260398688_Real_Time_Dynamic_Fracture_with_Volumetric_Approximate_Convex_Decompositions)

**Evidence gap:** I did not find an apples-to-apples published benchmark for **UE 5.8, RTX 4070 Ti, 1440p, Lumen/Nanite, and a concurrent 50,000-person Rust simulation**. A numerical claim that this complete workload will sustain 60 fps would be unsupported.

### 2.3 Proposed initial budgets for your machine

These are deliberately conservative **prototype targets**, to be revised after profiling.

| Quantity | Starting target |
| --- | --- |
| Frame time at 60 fps | **16.67 ms** |
| Simultaneous detailed live collapses | **1**, then test **2** |
| Active rigid bodies per ordinary nearby collapse | **40–100** |
| Global awake cosmetic-debris target | **200** |
| Emergency admission ceiling | **400**, only if measurements support it |
| Live simulation lifetime | Usually **4–6 seconds**, then settle/replace |
| Construction reconciliation and activation work | Target **≤0.5 ms** game-thread contribution in ordinary frames |
| Incremental destruction solver cost | Target **≤1–1.5 ms** under representative Rust load |
| Incremental debris, dust and shadow GPU cost | Target **≤1–2 ms** |

CPU worker time, game-thread time and GPU time overlap; these numbers must not simply be added into one misleading total. Measure the actual frame’s critical path.

Prefer a **budget admission controller** to a fixed “maximum buildings collapsing” rule. Two small huts and two dense masonry towers are not equivalent workloads.

When over budget, lower the effect tier immediately. **Do not queue a city-wide disaster into minutes of belated full-detail collapses.**

### 2.4 Memory and lighting caveats

A 12 GB graphics card does not provide 12 GB exclusively for destruction. Geometry, textures, shadow resources, Lumen data, render targets and other scene content compete for residency. Keep fracture materials shared and audit the combined intact/fractured/rubble asset families rather than budgeting them independently.

Also test the exact Lumen path. Epic’s software-tracing geometry list names static meshes, ISMs, HISMs and Landscape—not Geometry Collections. Screen traces can conceal representation differences, so a collapse looking correct from one view does not prove correct off-screen occlusion or reflections. Examine the Lumen Scene with screen tracing disabled during validation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

---

## 3. Precedents and lessons

| Precedent | Documented approach | Lesson for TCE |
| --- | --- | --- |
| **LEGO Fortnite — Epic, GDC 2024** | Michael Lentine’s presentation describes extensive physical interaction and destruction built around Chaos rigid bodies, destruction, networked physics and construct interoperability | Strong evidence that modular interactive construction is achievable. **My inference:** reproducing its full physics-first scope is unnecessary for TCE’s cosmetic collapses. The accessible presentation description supplies no comparable target-hardware benchmark. [GDC Vault](https://gdcvault.com/play/1034659/Chaos-Physics-in-LEGO-Fortnite) |
| **Rainbow Six Siege — Ubisoft, GDC 2016** | Julien L’Heureux describes integrating the Realblast destruction engine as a core gameplay feature and the technical hurdles of making it cooperate with other systems | Destruction is an integration problem, not just mesh fracture. TCE’s boundary between semantic damage and visual debris should be explicit from the beginning. [GDC Vault](https://www.gdcvault.com/play/1023307/The-Art-of-Destruction-in) |
| **0 A.D. — open-source RTS, inspected archived GitHub snapshot** | `Foundation.js` creates a construction preview, selects scaffold animation and updates visual construction progress. A code comment explicitly notes that the health-linked update can make an attacked building sink | Useful inspectable construction lifecycle code. For TCE, keep **construction completion and damage separate**, rather than turning damage into reversed construction. [GitHub](https://github.com/0ad/0ad/blob/master/binaries/data/mods/public/simulation/components/Foundation.js) |
| **Valley of the Ancient — Epic sample** | Epic documents extensive use of ISMs for kitbashed environment assembly | Reusable architectural modules and instancing are a practical engine-native foundation; this is an environment-rendering precedent, not a destruction benchmark. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine) |
| **NVIDIA Blast — open-source SDK, documentation version 5.0.6** | Its low-level and toolkit layers deliberately omit graphics and physics representations; applications respond to fracture/split outputs and provide those representations | Closely supports TCE’s proposed separation of semantic state from presentation. Study the architecture; adding another destruction SDK is not necessary merely to obtain cosmetic debris. [NVIDIA Omniverse](https://nvidia-omniverse.github.io/PhysX/blast/index.html) |
| **Müller et al. — ACM TOG, 2013** | Distinguishes detailed fracture geometry, rigid-body motion and small debris | Use different representations for different jobs. A visually convincing collapse need not turn every visible chip into a full body. [ResearchGate](https://www.researchgate.net/publication/260398688_Real_Time_Dynamic_Fracture_with_Volumetric_Approximate_Convex_Decompositions) |

The strongest common lesson is **representation separation**: construction state, damage state, render geometry, collision geometry and small debris need not be the same data structure.

---

## 4. Recommended TCE implementation

### 4.1 Make the lifecycle explicit

Use a small presentation state machine:

```
Planned
  → UnderConstruction
  → Complete
  → CollapsePresentation
  → RuinOrRubble
  → Cleared / Rebuilt

UnderConstruction can also transition directly to RuinOrRubble.
```

`CollapsePresentation` should be **transient Unreal state**, not a requirement for advancing the simulation.

A useful event contract would contain:

| Field | Purpose |
| --- | --- |
| `building_id`, `revision`, `event_id` | Identity, ordering and duplicate suppression |
| `simulation_time` | Determine whether the event is still relevant |
| `affected_module_ids` | Collapse only what actually exists and has failed |
| `failure_recipe`, direction/origin | Select a collapse presentation without asking Chaos to decide the outcome |
| `visual_seed` | Stable variation in timing, dust and rubble |
| `surviving_modules` / rubble descriptor | Authoritative final representation |
| `occupancy_revision` | Synchronize the simplified obstruction state with the kernel |

Material recovery, deaths, injuries and blocked routes should follow the kernel’s event—not whichever cosmetic fragment happened to bounce into a character.

### 4.2 Three collapse tiers are sufficient initially

**Tier A: not visible or too distant to matter.** Apply the persistent ruin/rubble state directly. Add only a cheap distant dust cue when useful.

**Tier B: visible but not important enough for live physics.** Animate a few coarse modules: a wall tilts, a roof section drops, a frame folds along an authored sequence. Spawn limited dust and small chips, then install rubble. Keep this path independent of Chaos Cache so it remains a reliable fallback.

**Tier C: nearby, important collapse.** Replace selected modules with pooled Geometry Collections or simple rigid static-mesh pieces. Use the expensive representation only where contact and rotation are perceptible.

Choose tiers using projected size, visibility and available budget, with hysteresis so an effect does not oscillate between representations. Once an event starts, normally keep its chosen tier unless emergency degradation is required.

For large buildings, spend the budget on the **visible failure front** rather than every module. Distant walls can transition to their ruin state while the nearest section receives detailed motion.

### 4.3 Instance-to-debris transition

Implement the swap as a controlled transaction:

**Prepare.** Select the effect recipe, ensure its assets are resident, reserve a pool slot and validate the building revision. The original instances remain visible until a replacement is ready.

**Align.** Initialize the replacement with the same module transform and the matching intact pose. Standardize scale and pivots in authoring; verify this with automated overlap screenshots.

**Exchange representations.** Remove the affected instances and disable their old collision before activating replacement collision. Do not leave intact and fractured representations overlapping for a physics step.

**Release.** Apply support release and motion in a short sequence. Author distinct responses for different assembly types; avoid using the same outward explosive impulse for every age-related collapse.

**Emit secondary effects.** Use selected break/impact information for dust, sound and chips. Do not emit a separate heavyweight Niagara system or gameplay callback for every tiny contact.

**Retire.** At rest, timeout, loss of significance or budget pressure, install the persistent rubble representation and reset or release the temporary effect.

The UE 5.8 collection API includes `SetRestCollection`, anchor controls, velocity/strain operations, event notification controls and **`ResetState()`**, whose documented purpose is to restore the unbroken state and reset physics. Prefer a tested reset lifecycle rather than assuming that assigning the rest asset alone completely resets a reused component. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/GeometryCollectionEngine/UGeometryCollectionComponent)

Pool by compatible prototype where practical. Repeatedly repurposing one component across unrelated assets creates more state combinations to validate than keeping several small, predictable pools.

### 4.4 Persistent rubble: authored placement is the best default

There are two main end-state strategies.

**A. Seeded rubble and surviving modules — recommended.** Generate the final scene from the failed footprint, material family, surviving structures and a seed. Use a modest set of heap meshes, broken beams, exposed foundations and standing wall fragments. Match the quantity and distribution approximately to what failed, while Rust retains exact salvage quantities.

Advantages include predictable streaming, compact saves and the ability to construct the same state without first running a collapse. This is especially important when the player returns to a settlement after decades of fast-forwarding.

**B. Preserve selected final fragment poses — optional refinement.** For especially visible collapses, retain the final positions of a few large pieces. Exact Geometry Collection-to-ISM conversion requires a prepared mapping from collection pieces to cooked static meshes and correct synchronized transforms. It is a project-specific conversion pipeline, not something to assume follows automatically from putting a collection to sleep.

A useful hybrid is to preserve the largest roof beam and wall slab while substituting generic rubble beneath them.

**Sleeping is not retirement.** It may reduce simulation activity, but it does not by itself turn the object into your compact persistent representation. Remove transient physics state once it no longer contributes visible value.

UE exposes removal-on-sleep and removal-on-break controls, but use these for transient fragments—not as a substitute for the persistent ruins that TCE’s history requires. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/GeometryCollectionEngine/UGeometryCollectionComponent)

### 4.5 Rust DLL integration, streaming and fast-forward

For this subsystem, I recommend a deliberately narrow integration:

**Ownership.** Rust owns stable world records. C++ owns Unreal objects, instance components, asset references and effect pools. Pass versioned, explicitly owned event/snapshot buffers across the DLL boundary; avoid exposing Rust or Unreal container layouts as the interface.

**Threading.** Let Rust produce presentation deltas independently, but reconcile Unreal component changes through a controlled game-thread stage. Keep queues bounded and coalesce repeated construction updates to the newest module state. Leave CPU capacity for Unreal and Chaos rather than allowing the simulation worker pool to occupy every available thread continuously.

**Identity.** Do not persist raw ISM indices as module identity. UE 5.8 exposes ID-based operations such as `AddInstancesById`, `GetInstanceIndexForId` and `RemoveInstanceById`, but calls the interface preliminary. Wrap it behind a TCE-owned handle layer and retain stable kernel IDs independently. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent)

**Time.** Run visual collapse motion on ordinary presentation/physics time. Do not multiply the Chaos timestep by the civilization’s time-acceleration factor. At high acceleration, skip obsolete intermediate stages and reconcile to the newest valid state.

**Streaming and saving.** An unloaded cell needs no cosmetic rigid bodies. Loading a ruined building should construct its ruin directly. A save made during collapse can store the authoritative final state; restoring the exact cosmetic instant is optional, not necessary for simulation correctness.

**Asset availability.** Explicitly include all manifest-referenced intact, fractured and rubble variants in cooking/loading rules. Epic’s Asset Manager exists in packaged games and supports asset discovery, loading and cooking organization; use it to prevent an effect from depending on assets that happened to be present only in the editor. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine)

### 4.6 Solo-developer implementation order

**First: prove the persistent states.** Build construction-stage instancing, scaffold generation, partial buildings, ruin generation, save/load and cell unload/reload. Every building should remain correct with destruction effects disabled.

**Second: add cheap motion.** Implement coarse module collapse, dust and sound, along with significance tiers and budget admission. This is already a viable shipping solution for cosmetic collapses.

**Third: add a small Chaos catalog.** Begin with one masonry bay and one roof/frame example. Validate pooling, collision filtering, replacement alignment and retirement in packaged builds before expanding the kit.

**Fourth: optimize demonstrated weaknesses.** Add cached playback, exact fragment-pose preservation or cluster unions only when a measured or visible problem justifies their maintenance cost.

### 4.7 Acceptance tests

Use **Unreal Insights** for frame timing and resource investigation, and **Chaos Visual Debugger** for recorded physics state. CVD supports runtime recording and inspection; it is complementary to, not a replacement for, frame profiling. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

The minimum test matrix should include a quiet settlement, mass construction completion, one close collapse, simultaneous collapses, a district-wide disaster, fast-forward, and repeated cell unload/reload—with representative Rust population load.

Record **median, p95, p99 and worst-frame times**, plus activation spikes, solver work, GPU passes, resident memory and remaining physics bodies. Test fixed cameras and camera movement, native 1440p and any intended upscaling mode separately.

Correctness checks are equally important: no unbuilt roof appears during collapse; no duplicate intact collision remains; no old event destroys a rebuilt building; rubble survives reload; and hundreds of completed effects do not leave growing pools, active bodies or retained references.

---

## 5. Sources and version applicability

### UE documentation and APIs

| Source | Version / relevance |
| --- | --- |
| [Instanced Static Mesh Component](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com) | Current UE 5.8 documentation; ISM/HISM selection, shared properties, custom data and memory example |
| [ISM component API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UInstancedStaticMeshComponent?utm_source=chatgpt.com) | UE 5.8; instance identity and update operations |
| [Destruction Overview](https://dev.epicgames.com/documentation/unreal-engine/destruction-overview?lang=en-US&utm_source=chatgpt.com) | Current documentation; Geometry Collections, clustering and destruction architecture |
| [Geometry Collections User Guide](https://dev.epicgames.com/documentation/unreal-engine/geometry-collections-user-guide?lang=en-US&utm_source=chatgpt.com) | Current documentation; asset preparation and pivot/geometry considerations |
| [Dataflow for Destruction Quickstart](https://dev.epicgames.com/documentation/en-us/unreal-engine/dataflow-for-destruction-quickstart?utm_source=chatgpt.com) | Current UE 5.8 page; editor-side repeatable fracture authoring |
| [Geometry Collection component API](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/GeometryCollectionEngine/UGeometryCollectionComponent?utm_source=chatgpt.com) | UE 5.8; initialization, resetting, removal, events and simulation limits |
| [Cluster Union component API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UClusterUnionComponent?utm_source=chatgpt.com) | UE 5.8; runtime membership and physics/game-thread synchronization |
| [Chaos Cache Manager API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/ChaosCaching/AChaosCacheManager?utm_source=chatgpt.com) | UE 5.8; explicitly Experimental |
| [Nanite overview](https://dev.epicgames.com/documentation/unreal-engine/nanite-virtualized-geometry-in-unreal-engine?lang=en-US&utm_source=chatgpt.com) | Current UE 5.8 page; Geometry Collection support and limitations |
| [Lumen technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine?utm_source=chatgpt.com) and [Virtual Shadow Maps](https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine?utm_source=chatgpt.com) | Current pages; scene-representation differences and shadow invalidation |
| [Niagara scalability](https://dev.epicgames.com/documentation/en-us/unreal-engine/scalability-and-best-practices-for-niagara?utm_source=chatgpt.com) | Current page containing some explicitly 5.4-era discussion; use the general guidance, not historical feature-status statements |
| [Unreal Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine?utm_source=chatgpt.com), [Chaos Visual Debugger](https://dev.epicgames.com/documentation/en-us/unreal-engine/chaos-visual-debugger-in-unreal-engine?utm_source=chatgpt.com), [Asset Management](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine?utm_source=chatgpt.com) | Profiling, physics inspection and packaged-asset lifecycle |

### Talks, code and research

| Source | Applicability |
| --- | --- |
| [Chaos Physics in LEGO Fortnite — Michael Lentine](https://gdcvault.com/play/1034659/Chaos-Physics-in-LEGO-Fortnite?utm_source=chatgpt.com) | GDC 2024; shipped modular-physics precedent |
| [The Art of Destruction in Rainbow Six Siege — Julien L’Heureux](https://www.gdcvault.com/play/1023307/The-Art-of-Destruction-in?utm_source=chatgpt.com) | GDC 2016; destruction integration precedent, not UE/Chaos implementation guidance |
| [Epic discussion of advanced modular destruction](https://forums.unrealengine.com/t/looking-for-an-advanced-workflow-for-chaos-destruction-on-a-building/2315780?utm_source=chatgpt.com) | Epic engineer guidance from 2025; distinguish historical limitations from current API capabilities |
| [0 A.D. `Foundation.js`](https://github.com/0ad/0ad/blob/master/binaries/data/mods/public/simulation/components/Foundation.js?utm_source=chatgpt.com) | Inspected archived source snapshot; construction-preview and scaffold lifecycle |
| [NVIDIA Blast documentation](https://nvidia-omniverse.github.io/PhysX/blast/index.html?utm_source=chatgpt.com) and [source](https://github.com/NVIDIAGameWorks/Blast?utm_source=chatgpt.com) | Documentation version 5.0.6; architectural reference, not a claimed drop-in UE 5.8 integration |
| [Real Time Dynamic Fracture with Volumetric Approximate Convex Decompositions](https://matthias-research.github.io/pages/publications/fractureSG2013.pdf?utm_source=chatgpt.com) | Müller, Chentanez and Kim, ACM TOG 2013; research algorithm and historical performance results |

**Bottom line:** TCE should have an inexpensive, complete construction-and-ruin system before it has sophisticated destruction. Make **instanced construction → short-lived collapse presentation → static persistent rubble** the invariant. Chaos then becomes a controlled visual enhancement rather than a dependency of the civilization simulation.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92987-42cc-83e9-8ea4-89e935f1dba7)
