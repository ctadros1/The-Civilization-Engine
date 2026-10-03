# Shape grammars and procedural architecture for TCE

*Engineering report — evidence checked September 27, 2026.*

## Executive recommendation

**Build a small, typed, kit-aware grammar in Rust. Let it produce a persistent, functional building plan and a disposable rendering recipe. Have Unreal assemble that recipe using spatially grouped instances.**

Do not start by cloning the whole CGA language, implementing general-purpose computational geometry, or adopting a city-wide constraint solver. For TCE, the difficult boundary is not “rules versus hand modeling.” It is **continuous architectural geometry versus a finite collection of reusable meshes**.

Production systems support this distinction. The original Unreal City Sample generated building configurations in Houdini and transferred instance-placement data into Unreal; it nevertheless used custom geometry for some cases, including non-rectangular roofs. UE 5.8’s newer City Sample PCG implements building grammars inside Unreal, demonstrating that the grammar and the placement backend can be separated from the original authoring tool. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration)

For TCE, I recommend:

| Decision | Recommended starting point |
| --- | --- |
| Authoritative architecture | Rust building plan: footprint, spaces, access, structure, materials, construction phases |
| Generation method | Attributed, top-down grammar with discrete module fitting |
| Rendering | Prebuilt kit meshes, predominantly Nanite ISMs, grouped by spatial cell and rendering compatibility |
| Irregular geometry | Restricted, explicitly supported building compositions first; exceptional generated surfaces later |
| Authoring | Data-driven rules and a validated kit manifest; Houdini optional for offline asset production |
| Persistence | Save realized architectural decisions and versions, not merely a seed |
| Runtime scheduling | Generate on construction/change events; stream visual detail independently of simulation |

These are architectural recommendations, not measured performance claims for your machine.

---

## 1. Options: what the techniques actually do

### 1.1 Split grammars and CGA: the main foundation

Wonka and colleagues’ **Instant Architecture** introduced architectural split grammars together with attribute matching and a separate control grammar. The important idea is hierarchical refinement: a building becomes sections, sections become floors or façade regions, and those become architectural elements. Attributes coordinate the result rather than allowing every rule to make an unrelated random choice. [Computer Graphics TU Wien](https://www.cg.tuwien.ac.at/research/publications/2003/Wonka-2003-Ins/)

Müller and colleagues’ **Procedural Modeling of Buildings** developed the CGA approach into a broader system for building geometry. CGA is best understood as a procedural geometric language operating on shapes with local coordinate systems, dimensions, labels and attributes—not simply a collection of replacement strings. [Arizona State University](https://asu.elsevierpure.com/en/publications/procedural-modeling-of-buildings-2/)

A TCE equivalent could look like this:

```
Building
  → choose feasible typology
  → create connected building volumes
  → resolve floors and structural bays
  → classify exposed, shared and entrance-facing surfaces
  → fit façade modules
  → resolve roof and foundation
  → emit construction quantities and render placements
```

Current CityEngine splits support absolute sizes, relative sizes and “floating” sizes that absorb remaining space. Repetition can adjust dimensions to fit; some configurations can cut the final geometric element. That flexibility is useful in a geometry generator, but **must not be copied blindly into an instancing backend**. An unchanged window mesh cannot be arbitrarily clipped just because a split operation leaves an awkward remainder. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/cga/cga-split.htm)

**TCE fit:** excellent, provided that the grammar’s terminal operations obey the kit’s dimensional and connection contracts.

### 1.2 Façade grammars: a narrower, particularly useful subsystem

A façade grammar usually divides a wall into a hierarchy such as:

```
base / occupied floors / cornice
    → structural bays
        → wall / opening / frame / ornament
```

The hierarchy provides alignment: windows can share a bay axis across floors, entrances can interrupt a regular rhythm, and upper floors can remain coherent while the ground floor changes use.

Müller et al.’s 2007 façade work illustrates the inverse process: detect repetition and hierarchical subdivisions in a façade image, then convert the recovered structure into reusable grammar rules. It also shows limitations involving irregular façades, reflections and image quality. **Extracting rules from photographs is an authoring aid, not something TCE needs to do during simulation.** [Hunter College Computer Science](https://www.cs.hunter.cuny.edu/~ioannis/3DP_S09/mueller_facades_2007.pdf)

**TCE fit:** very good for wall articulation, but façade generation should follow the building’s room, structure and access decisions. Otherwise it is easy to produce convincing windows with no plausible space behind them.

### 1.3 Modular patterns and parameterized templates

This approach starts with a supported building topology or blockout and fills it with named modules. It is less general than CGA but easier to author, debug and constrain.

SideFX Labs Building Generator 4.0 follows this pattern: it analyzes blockout volumes, slices floors, identifies walls, corners and ledges, then substitutes library components. Its utility node supplies explicit module dimensions, weighted variations, floor patterns and localized overrides. Notably, intended dimensions can differ from mesh bounds because decorative projections should not determine the fitting width. [SideFX](https://www.sidefx.com/docs/houdini/nodes/sop/labs--building_generator-4.0.html)

**TCE fit:** probably the best initial implementation. A few parameterized topologies—rectangular house, courtyard assembly, attached workshop, elongated hall—can provide substantial variety without a general geometry language.

### 1.4 Socket graphs, adjacency rules and Wave Function Collapse

A socket system connects compatible interfaces. A constraint-based variant maintains possible pieces at each location and eliminates combinations that violate adjacency rules.

Wave Function Collapse, or WFC, is one implementation family: repeatedly select a constrained location, choose an allowed pattern and propagate restrictions. Contradictions can require restarting or backtracking. Local compatibility alone does not establish global circulation, structural support or economic feasibility. The original implementation documents both its constraint-propagation approach and the possibility of contradictions. [GitHub](https://github.com/mxgmn/WaveFunctionCollapse)

**TCE fit:** useful for bounded subproblems—decorative arrangements, fence junctions, small roof-detail neighborhoods or room-template connections. I would not make unconstrained WFC the primary whole-building planner.

### 1.5 Layout and structural constraint solving

A separate solver can decide room adjacency, circulation, support placement or structural feasibility before the façade is generated.

This is a distinct problem from producing an attractive shell. Whiting, Ochsendorf and Durand explicitly added structural feasibility to procedural masonry modeling by optimizing selected parameters. Their work is useful evidence that structural soundness does not automatically follow from a shape grammar. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/559ae388-f9bc-4433-874c-36cce7af9178)

**TCE fit:** use inexpensive, explicit constraints first: supported spans, connected entrances, minimum clearances, valid stairs and load-path templates. A general architectural optimizer is unnecessary for the first production version.

### 1.6 Straight-skeleton roofs

A straight skeleton is generated by moving a polygon’s edges inward and tracing the moving vertices. Edge-collapse and split events determine the resulting structure. It is related to roof construction because offset distance can be lifted into height, producing roof planes and their intersections. Weighted variants support differing edge behavior. CGAL provides a mature implementation for polygons with holes, including weighted skeletons and offsetting. [CGAL Documentation](https://doc.cgal.org/latest/Straight_skeleton_2/index.html)

However, **a straight skeleton is neither a complete roof designer nor a roof-kit fitter**. You still need decisions about pitch, gables, eaves, drainage, supporting structure, valleys, openings, materials and construction.

CityEngine’s `roofHip` illustrates another geometric formulation: intersect planes associated with footprint edges. Its options also expose compromises—for example, forcing an even ridge can produce non-planar roof faces. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/cga/cga-roof-hip.htm)

For TCE, the principal problem is the output boundary: an arbitrary roof polygon does not necessarily match your stock rectangular roof panels. Either restrict the supported roof compositions, provide appropriate triangular/junction pieces, introduce deformation, or allow generated geometry.

---

## 2. Trade-offs and available performance evidence

### 2.1 Practical comparison

The assessments below are engineering judgments about TCE’s workload, not comparative benchmark results.

| Technique | Runtime characteristics | Authoring and implementation burden | Best role in TCE |
| --- | --- | --- | --- |
| Whole-building catalogue | Selection and placement are cheap; variation is bounded by catalogue size | Low runtime complexity; potentially large asset workload | Landmarks, fallback buildings, early prototype |
| Parameterized modular templates | Predictable work proportional to generated parts when fitting is bounded | Moderate; supported topology must be explicit | First production architecture system |
| CGA-like attributed grammar | Efficient for bounded derivations; geometric queries and mesh operations can dominate | Moderate for a subset, very high for a full language | Long-term main generator |
| Façade grammar | Usually regular, local and easy to cache | Moderate; alignment and special cases need care | Exterior articulation |
| WFC/constraint assembly | Highly dependent on constraints, contradictions and search limits | Rules may be simple individually but difficult collectively | Small, bounded secondary problems |
| General polygon roofs and Boolean modeling | Sensitive to geometric complexity and degeneracies | High robustness and integration burden | Exceptional geometry, later development |

For a bounded rule tree, simple placement generation can approach **linear work in the number of evaluated rules and emitted parts**. That observation ceases to describe the whole job once rules repeatedly perform polygon clipping, neighborhood searches, optimization or mesh construction.

A useful fitting algorithm for discrete façade modules is a small dynamic program over façade length in grid units. With \(L\) length units and \(K\) permitted module widths, a straightforward feasibility pass can be \(O(LK)\). This is a proposed implementation strategy, not a claim about CGA’s internal algorithm.

### 2.2 Published numbers—and their limits

| Source | Reported result | What it establishes |
| --- | --- | --- |
| **Instant Architecture, 2003** | Example database of roughly **250 rules and 40 attributes**; buildings typically **1,000–100,000 polygons**; approximately **1–3 seconds per building on a 2 GHz Pentium 4** | A useful historical generation benchmark, not an Unreal runtime benchmark. [ResearchGate](https://www.researchgate.net/publication/47504041_Instant_architecture) |
| **Image-based Procedural Modeling of Facades, 2007** | Approximately **3 minutes** for the first stage on a **1600 × 1200 image**; subsequent stages typically **30–90 seconds** | Image analysis is a different workload from evaluating an already-authored grammar. Do not use these timings to estimate TCE construction generation. [Hunter College Computer Science](https://www.cs.hunter.cuny.edu/~ioannis/3DP_S09/mueller_facades_2007.pdf) |
| **Epic ISM documentation, UE 5.8** | Illustrative GPU storage comparison: approximately **672 bytes per primitive versus 64 bytes per basic instance** | Instancing reduces certain representation costs. These are not total per-object memory figures or a promised frame-rate multiplier. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine) |
| **Skyrim modular production, GDC 2013** | Two full-time kit artists produced seven kits used for more than 400 dungeon cells over about 2.5 years | Strong evidence for asset-production leverage, not autonomous generation throughput. [Game Developer](https://www.gamedeveloper.com/design/skyrim-s-modular-approach-to-level-design) |

**There is no defensible conversion from these figures to “TCE will run at 60 fps.”** None establishes your combined workload of agent simulation, runtime construction, crowds, vegetation, lighting, streaming and architectural rendering.

For your system, benchmark four separate costs: **planning**, **recipe generation**, **Unreal installation/update**, and **steady-state rendering**. A generator can be fast while installing its output causes frame hitches.

---

## 3. Production precedents and what to borrow

### CityEngine: mature grammatical modeling, with a licensing boundary

CityEngine’s Procedural Runtime, PRT, accepts initial geometry and rule packages and can execute outside the CityEngine application. Rule packages combine rules and assets; this is a useful precedent for TCE’s versioned style-and-kit bundles. The SDK’s current requirements refer to CityEngine 2026.0. [GitHub](https://github.com/Esri/cityengine-sdk)

The Unreal integration explicitly supports runtime generation. However, its published terms distinguish Apache-licensed integration source from the included SDK/runtime and state that redistribution requires express permission. **Do not interpret the public GitHub repository as permission to ship PRT inside TCE.** Confirm shipping rights before making it a dependency. [GitHub](https://github.com/Esri/cityengine_for_unreal)

**Borrow:** shape scopes, explicit attributes, packaged rule/asset dependencies and inspectable derivations.  
**Avoid:** assuming that the runtime is a freely redistributable substitute for your Rust kernel.

### Houdini: excellent production authoring, not the same as game-runtime generation

Houdini’s strength is building and inspecting procedural asset pipelines. Labs Building Generator demonstrates a particularly relevant division between low-resolution intent and high-resolution modular realization. The current Houdini 22.0 documentation lists Unreal plugin binaries for UE 5.8 and 5.7. [SideFX](https://www.sidefx.com/docs/houdini/nodes/sop/labs--building_generator-4.0.html)

The documented packaging path retains output components as game data, or bakes them into Unreal-native actors and removes the plugin dependency. That is different from shipping an interactive Houdini modeling session to every player. [SideFX](https://www.sidefx.com/docs/houdini/unreal/packaging.html)

**Borrow:** offline kit generation, module metadata, diagnostic visualization and representative building previews.  
**Avoid:** making TCE’s construction simulation depend on live HDA cooking.

### Unreal City Sample: the closest architectural precedent

The original City Sample used Houdini-generated point data, a Rule Processor and extensive instancing. Building styles were grammatical; some buildings combined different lower and upper styles. The documented production versions were Houdini **18.5.532** and Houdini Engine **3.5.2**, so this is a historical pipeline, not instructions for installing today’s plugin. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration)

The **UE 5.8 City Sample PCG** is a newer, engine-native implementation. Its building primitive accepts footprint splines, extrudes them and applies style definitions. Its grammar assets also describe modular sequences along splines and cross-sections. Epic notes that initial grammar configuration still requires substantial dimensional and orientation setup. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?lang=en-US)

**Borrow:** semantic input → rule evaluation → placement data → shared meshes. Inspect the current sample before inventing a visual authoring interface.

For TCE, PCG is an alternative backend or authoring reference—not a reason to duplicate authoritative building decisions in both Rust and Unreal.

### Townscaper: constrained vocabulary with strong local responsiveness

Townscaper’s official description emphasizes an irregular grid and configuration-dependent results such as houses, arches, stairs, bridges and courtyards. The WFC project’s implementation survey describes Stålberg’s combination of WFC and marching-cubes ideas on irregular grids. This is more specific than saying “Townscaper is a CGA system.” [Townscaper](https://www.townscapergame.com/)

**Borrow:** a restricted vocabulary whose combinations receive exceptional attention, particularly junctions and local changes.

**Do not infer:** that its techniques solve household capacity, construction resources, functional interiors or large-agent simulation. Its value here is combinatorial coherence and feedback, not a directly comparable simulation benchmark.

### Cities: Skylines II: catalogue and state variation, not demonstrated CGA

The launch-era developer diary documents architectural themes, different building uses and densities, and visual changes at every other building level. These are useful examples of connecting appearance to simulation state. They do **not** establish that the game generates arbitrary building topology through a public CGA-like grammar. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/zones-signature-buildings)

**Borrow:** separate appearance dimensions—use, scale, investment, theme and condition.

**Modify for TCE:** do not make “higher level” universally mean “more modern.” Improvement might instead add a wing, replace a roof, subdivide rooms or embellish a façade while preserving older construction.

### Bethesda and Sunset Overdrive: kits are production systems

Burgess and Purkeypile emphasize compatible metrics, shared doorway standards, early functional prototypes and the danger of changing a kit piece after widespread use. Their account also distinguishes architectural repetition from conspicuously repeated clutter. [Game Developer](https://www.gamedeveloper.com/design/skyrim-s-modular-approach-to-level-design)

Insomniac’s **Ultimate Trim** workflow standardized trim layout, UV tooling and shader behavior to improve production speed, memory use and visual consistency. [GDC Vault](https://gdcvault.com/play/1022324/The-Ultimate-Trim-Texturing-Techniques)

**Borrow:** enforceable interface contracts and shared material conventions—not merely a folder of meshes.

---

## 4. Recommended TCE design

Everything in this section is a proposed architecture for TCE unless identified as documented engine behavior.

### 4.1 Separate the functional building from its rendered assembly

Use two representations:

| Authoritative `BuildingPlan` | Derived `RenderRecipe` |
| --- | --- |
| Persistent building and part identities | Mesh identifiers and local transforms |
| Footprint, wings, floors and spaces | Material-family selection and custom data |
| Entrances, portals, circulation and occupancy capacity | Detail level and visibility classification |
| Structural system and supported spans | Simplified collision representation references |
| Material quantities and construction phases | Instance additions, removals and replacements |
| Construction-date style decisions and later alterations | Optional exceptional geometry |
| Damage, repairs, extensions and ownership-relevant facts | Recipe/cache version |

The simulation should not discover capacity or accessibility by inspecting Unreal triangles. Conversely, the renderer should not choose a different floor count because a PCG graph happened to evaluate differently.

Use a transaction-like construction sequence:

**request → feasible design → material/labor quotation → resource commitment → construction phases → occupation.**

Quote costs from the realized design, not from an unrelated building-type constant. Cosmetic trim can remain a coarse allowance; structural walls, floors and roofs should correspond to actual planned quantities.

This distinction also supports invisible buildings: they remain fully meaningful to the simulation without retaining detailed rendering objects.

### 4.2 Implement a constrained grammar, not a new general-purpose language

Start with typed data loaded into a validated rule graph. A custom textual parser can wait.

Useful scope types are `Footprint`, `Volume`, `Face`, `Edge`, `Bay` and `Socket`. Useful operations are:

| Operation | Required contract |
| --- | --- |
| `ChooseTypology` | Only selects forms feasible for the site, program and capabilities |
| `CreateWing` / `Extrude` | Produces an explicit volume with a supported footprint |
| `SplitFixed` | Preserves required dimensions and minimum residual sizes |
| `RepeatDiscrete` | Fits complete modules using declared filler policies |
| `SelectFaces` | Uses semantic exposure, frontage and adjacency information |
| `Attach` | Requires compatible sockets and clearance |
| `EmitModule` | Produces a supported module variant and valid transform |
| `ResolveRoof` | Calls a supported roof strategy, with a bounded fallback |

The first implementation can be an ordinary Rust evaluator over enums and structs. Compilation into bytecode is unlikely to be your first bottleneck.

Give every rule a termination contract: maximum recursion depth, maximum emitted parts, bounded retries and an explicit failure result. When no legal production exists, return a diagnostic or a simpler feasible design—not partially valid geometry.

For polygon groundwork, `geo` provides Rust primitives, containment/intersection operations, Boolean operations and offsets under MIT/Apache licensing. It is a useful foundation, but not a complete architectural grammar or a substitute for roof-specific algorithms. [GitHub](https://github.com/georust/geo)

### 4.3 Make module fitting an explicit algorithm

A façade should not silently stretch every element to make the arithmetic work.

For a usable run length \(L\), solve something like:

\[
L = c\_{\text{left}} + \sum\_i n\_i w\_i + c\_{\text{right}} + f
\]

Here, \(w\_i\) are approved module widths, \(n\_i\) are integer counts, corners have explicit widths, and \(f\) is a permitted filler arrangement.

For example, after reserving corners, a 9.5 m run might accept three 3 m bays and one 0.5 m blank filler. A different style might require symmetric 0.25 m margins. An entrance requirement could invalidate both arrangements.

Define a small number of fitting policies:

* **Rigid:** doors, windows, stairs and distinctive ornaments retain approved dimensions.
* **Discrete:** wall or roof variants are chosen from compatible sizes.
* **Controlled stretch:** plain infill or beams may stretch on specified axes within declared limits.
* **Exceptional geometry:** explicitly generated, rather than masquerading as an ordinary instance.

Reserve required openings before filling decorative bays. Coordinate bay axes across floors where the structural system or style requires it.

### 4.4 Design the kit manifest before producing the detailed art

A module needs more than a mesh path and a bounding box.

Its manifest should record its intended envelope, pivot convention, connection profiles, socket transforms, permitted dimensions, stretch limits, material/UV conventions, collision category, construction role and compatible neighboring modules.

**Sockets should express architectural compatibility.** Matching names alone is insufficient: a door socket also needs width, height, threshold, outward direction, wall profile and clearance information. Export this metadata into Rust-readable data; do not require Rust to query Unreal assets while planning.

Suggested *prototype metrics*, not historical measurements:

| Metric | Initial engineering choice |
| --- | --- |
| Local dimensional quantum | 0.25 m |
| Common bay widths | A small set such as 1, 2, 3 and 4 m |
| Blank fillers | 0.25 and 0.5 m where visually acceptable |
| Floor heights | A few compatible families, initially perhaps 2.5 and 3 m |
| Roof pitches | A small approved catalogue per roof family |
| Placement grid | Building-local, not one global grid imposed on settlements |

These values are negotiable. The non-negotiable requirement is that the grammar and kit agree.

A first kit must close correctly: foundations, external and internal corners, end caps, door and window openings, roof edges, ridges, and terrain transitions. A beautiful straight wall is not a complete kit.

Use actual opening modules. Placing a window frame over an opaque wall does not create a window.

Keep ordinary bricks, roof tiles and repetitive micro-detail inside larger meshes or materials. Preserve separate instances where they provide useful reuse, silhouette variation, construction visibility or interaction—not merely because something is a physical constituent.

### 4.5 Use trim sheets and correlated variation

A trim sheet places reusable surface bands and details into a shared texture layout. Kit surfaces map into those bands instead of requiring a unique texture per building. Standardized layouts also make material-family changes possible without rebuilding UVs; this production rationale is central to Insomniac’s Ultimate Trim workflow. [GDC Vault](https://gdcvault.com/play/1022324/The-Ultimate-Trim-Texturing-Techniques)

For TCE, combine shared trims with tiling wall/roof materials and a small palette of material families. Use per-instance custom data for appropriate scalar variation; Unreal supports this without creating a unique dynamic material instance for every mesh. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Variation should be correlated at different scales:

**Settlement:** prevalent material and roof families.  
**Building:** typology, proportions, bay rhythm, overall palette.  
**Construction episode:** an extension’s slightly different material or workmanship.  
**Part:** restrained wear, staining and minor variation.

Randomizing every window and wall independently produces noise, not a convincing architectural culture.

### 4.6 Encode style as preferences over feasible construction

Do not represent style as a texture choice—or as a fixed historical era.

I would separate:

| Layer | Examples |
| --- | --- |
| Hard feasibility | Available materials, craft capabilities, span limits, site envelope, laws, budget |
| Typological preferences | Courtyard versus compact block; attached versus detached; preferred entrance arrangement |
| Geometric preferences | Proportions, opening ratios, roof form, bay rhythm, symmetry |
| Detail vocabulary | Permitted frames, supports, cornices, screens and ornament |
| Material appearance | Finish, palette and surface treatment |
| Social transmission | Which builders or institutions supplied the adopted design knowledge |

A possible rule-selection model is:

\[
P(r\mid S,C)\propto
\mathbf{1}[\text{feasible}(r,C)]
\exp\!\left(S\cdot F(r)-\lambda\,\widehat{\text{cost}}(r,C)\right)
\]

Here \(S\) contains style preferences, \(C\) is construction context, \(F(r)\) describes the rule’s features, and cost is normalized for the scoring model.

The crucial feature is the feasibility gate. A strong aesthetic preference cannot create an unavailable material or unsupported span.

Store the realized style decisions at construction time. Later cultural change should influence new buildings and deliberate renovations; it should not automatically restyle the existing city.

Also preserve multiple structural families. A rectangular split grammar can coexist with a radial template grammar. Do not make the first implementation’s rectangular convenience a permanent restriction on every culture.

### 4.7 Treat roofs as a bounded subsystem

For the first shipping version, support a deliberately limited set of compositions: individual rectangular volumes, a few approved wing junctions, courtyard arrangements with known solutions, and any separately authored radial roof family.

For each, specify both the visible roof and its semantic construction: supporting walls or posts, roof planes, coverage, quantities and permitted junctions.

For more general footprints, adopt a separate path:

**validated polygon → roof topology → roof planes → mesh patches/junctions → shared materials.**

Cache those exceptional results. Do not assume that runtime-generated roof geometry automatically receives the same rendering and build pipeline as preprocessed Nanite kit assets.

CGAL is a strong reference or implementation candidate for skeleton computation. Its straight-skeleton package is GPL-licensed, while commercial licensing is also available; this requires a deliberate dependency decision for a proprietary shipped game. [CGAL Documentation](https://doc.cgal.org/latest/Manual/packages.html)

For a solo developer, I would postpone a general straight-skeleton implementation until supported roof templates demonstrably block important gameplay. AI coding assistance does not remove the need to test degeneracies, holes, nearly coincident edges and unstable topology changes.

### 4.8 Preserve identity and regenerate locally

Assign stable semantic IDs to wings, floors, bays, openings and important structural parts.

Use random decisions keyed by building identity and semantic purpose, rather than consuming a shared random stream. Adding a chimney should not reroll every window. Avoid identifying everything only by array position: inserting a bay would otherwise renumber the building.

Persist the realized design, grammar version, kit version and relevant decisions. **A seed alone is insufficient for save stability when rules, assets or algorithms change.**

For incremental construction, maintain dependencies. Changing a roof material should not rerun room planning. Adding a wing may require a local roof junction and façade update, but should preserve unaffected spaces and parts.

Generate from immutable context snapshots. Before committing an asynchronously generated design, check that its site and building revision still match. Cancel stale work and bound queued jobs and caches; an endless simulation cannot accumulate every historical rendering recipe indefinitely.

### 4.9 Use a narrow Rust–Unreal boundary

Expose a versioned C ABI from the Rust DLL and implement a small C++ Unreal adapter.

Use fixed-width fields, `#[repr(C)]` records, opaque handles and explicit buffer ownership. Do not expose Rust `Vec`, `String` or internal enums directly as a cross-language contract. Define which side allocates and which side releases every buffer.

Do not allow ordinary errors or panics to escape across the boundary. `catch_unwind` only catches unwinding panics; it does not recover from `panic=abort`. Rust’s FFI documentation explains the relevant layout and unwinding constraints. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Send bulk placement deltas, not one DLL call per module. Keep Unreal object mutation in the adapter, using a controlled game-thread commit queue. Test units, orientation, winding and local-to-world conversion at this boundary.

Separate save-format versions from ABI versions. Pin the Windows toolchain and test packaged builds early; an editor-only success is not sufficient.

### 4.10 Assemble spatially in Unreal

Start with buckets keyed approximately by:

```
spatial cell
+ mesh
+ material configuration
+ representation level
+ collision/shadow policy
```

Avoid both extremes: an actor/component for every decorative part, and one enormous world-wide bucket for every mesh.

For Nanite-only meshes, Epic recommends ISM because Nanite handles its own culling and LOD. HISM remains worth testing for large, mostly static non-Nanite populations. Current ISMs also support per-instance LOD; older advice that this necessarily requires HISM is outdated. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Keep semantic part IDs independent of instance-array indices. Batch additions and removals, leave unchanged transforms untouched, and budget installation work across frames.

Use simplified semantic collision and circulation representations. Do not make every trim piece participate in collision or navigation. Promote nearby interactive elements to richer representations only when necessary.

For interiors, validate the kit against the chosen lighting path. Epic’s Lumen guidance favors separable walls, floors and ceilings and warns about large combined interiors and thin or one-sided geometry. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

Nanite does not make instance counts, materials, resolution or lighting costs irrelevant. Epic explicitly identifies these as remaining performance dimensions. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

---

## 5. Performance plan, implementation sequence and source guide

### 5.1 What to budget on your target machine

At 60 fps, the frame interval is **16.67 ms**. Generation throughput and frame latency must be treated separately: a background job taking several milliseconds is acceptable when it neither blocks the simulation nor causes an expensive installation spike.

These are **initial test targets**, not predicted results:

| Area | Proposed starting target |
| --- | --- |
| Simple warm Rust recipe generation | Investigate a p95 target around 2 ms for a supported building of roughly 100–300 placements |
| Unreal construction/streaming commit | Start with a 0.5–1 ms game-thread allowance per frame; queue excess work |
| GPU | Seek approximately 2–3 ms of headroom below the 16.67 ms interval in representative scenes |
| Streaming partition experiments | Compare 32, 64 and 128 m cells rather than assuming one is optimal |
| Visual workload tests | Independently vary resident placement counts, visible buildings and visible agents |

The 12 GB VRAM budget is shared with textures, geometry, lighting, shadows, vegetation, crowds and transient buffers. As a scale illustration, one million basic 64-byte instance records would be 64 MB—but that excludes practically everything else needed to render those instances. Do not mistake instance-record arithmetic for a scene-memory estimate. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

Test **native 1440p** and any upscaled configuration separately. Do not report an upscaled result as a native-resolution guarantee.

Likewise, 50,000 simulated people and 50,000 simultaneously detailed animated characters are separate requirements. Architecture should leave CPU and GPU headroom for your crowd representation rather than consuming the entire frame budget in an empty-city benchmark.

A compact acceptance suite should include street-level and aerial cameras, cold and warm streaming, ordinary construction, mass demolition/rebuilding, long-lived extensions, and the full simulation running concurrently. Record frame-time percentiles, allocation peaks and backlog growth—not only average fps.

### 5.2 Implementation sequence

**First milestone: one complete, ugly kit.** Implement a rectangular building family, real entrances, simple interiors, one roof strategy, quantities and construction phases. Generate a gallery of seeded examples in a headless Rust test executable and in Unreal.

**Second milestone: the real runtime boundary.** Add bulk DLL transfer, stable part IDs, cell streaming, incremental updates and packaged Windows testing. Establish the dense-scene baseline before multiplying styles.

**Third milestone: compositional diversity.** Add attached wings, courtyards, supported junctions, multiple structural families and construction-date style persistence.

**Fourth milestone: exceptional geometry.** Introduce general roofs or more sophisticated layout solving only where the existing vocabulary prevents important simulation outcomes.

For AI coding agents, give each subsystem executable invariants: modules must fit, required portals must connect, plans must remain inside allowed envelopes, quantities must remain nonnegative, derivations must terminate, and the same saved design must survive reload. Maintain small failing seeds and visual regression scenes for corners, roof joints and terrain transitions.

The biggest avoidable mistake is to expand the rule language faster than you expand the tests and kit contracts.

### 5.3 Source and version guide

| Resource | Version/date and why it matters |
| --- | --- |
| [Wonka et al., **Instant Architecture**](https://www.cg.tuwien.ac.at/research/publications/2003/Wonka-2003-Ins/?utm_source=chatgpt.com) | SIGGRAPH 2003. Split grammars, attribute matching and control grammar; [author-uploaded full text](https://www.researchgate.net/publication/47504041_Instant_architecture?utm_source=chatgpt.com) contains the historical benchmark. |
| [Müller et al., **Procedural Modeling of Buildings**](https://doi.org/10.1145/1141911.1141931) | SIGGRAPH 2006. Foundational CGA building-generation paper. |
| [Müller et al., **Image-based Procedural Modeling of Facades**](https://www.cs.hunter.cuny.edu/~ioannis/3DP_S09/mueller_facades_2007.pdf?utm_source=chatgpt.com) | SIGGRAPH 2007 author preprint. Façade hierarchy, repetition and rule extraction. |
| [CityEngine `split`](https://doc.arcgis.com/en/cityengine/latest/cga/cga-split.htm?utm_source=chatgpt.com) and [`roofHip`](https://doc.arcgis.com/en/cityengine/latest/cga/cga-roof-hip.htm?utm_source=chatgpt.com) | Current CGA documentation accessed September 2026. Precise fitting and roof semantics. |
| [CityEngine SDK](https://github.com/Esri/cityengine-sdk?utm_source=chatgpt.com) and [Unreal integration](https://github.com/Esri/cityengine_for_unreal?utm_source=chatgpt.com) | Current repositories; SDK requirements reference CityEngine 2026.0. Inspect runtime redistribution terms separately from source licenses. |
| [Labs Building Generator 4.0](https://www.sidefx.com/docs/houdini/nodes/sop/labs--building_generator-4.0.html?utm_source=chatgpt.com) and [Utility 2.0](https://www.sidefx.com/docs/houdini/nodes/sop/labs--building_generator_utility-2.0.html?utm_source=chatgpt.com) | Houdini 22.0 documentation. Module patterns, dimensions, fillers and overrides. |
| [Houdini Engine compatibility](https://www.sidefx.com/docs/houdini/unreal/intro.html?utm_source=chatgpt.com) and [packaging](https://www.sidefx.com/docs/houdini/unreal/packaging.html?utm_source=chatgpt.com) | Current UE 5.8/5.7 support and baked-output workflow. |
| [Original City Sample](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration?utm_source=chatgpt.com) and [City Sample PCG](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?lang=en-US&utm_source=chatgpt.com) | Distinguish the original UE 5.0-era Houdini pipeline from the UE 5.8 PCG implementation. |
| [UE shape grammar](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-shape-grammar-with-pcg-in-unreal-engine?utm_source=chatgpt.com) and [ISM documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8. Engine-native modular grammar and current ISM/HISM guidance. |
| [CGAL straight skeleton](https://doc.cgal.org/latest/Straight_skeleton_2/index.html?utm_source=chatgpt.com) and [licensing](https://www.cgal.org/license.html?utm_source=chatgpt.com) | CGAL 6.2.1 documentation. Mature geometric implementation; package-specific licensing matters. |
| [Original WFC implementation](https://github.com/mxgmn/WaveFunctionCollapse?utm_source=chatgpt.com) and [Stålberg’s Townscaper talk](https://www.youtube.com/watch?v=Uxeo9c-PX-w&utm_source=chatgpt.com) | Algorithm/code reference and SGC 2021 developer presentation. |
| [Cities: Skylines II zones/buildings diary](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/zones-signature-buildings?utm_source=chatgpt.com) | Launch-era design documentation, not a claim about every 2026 asset pack or internal generator. |
| [Skyrim modular design](https://www.gamedeveloper.com/design/skyrim-s-modular-approach-to-level-design?utm_source=chatgpt.com) and [Ultimate Trim](https://gdcvault.com/play/1022324/The-Ultimate-Trim-Texturing-Techniques?utm_source=chatgpt.com) | GDC 2013 and 2015 production accounts. Kit contracts, repetition and material workflow. |
| [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com), [`geo` code](https://github.com/georust/geo?utm_source=chatgpt.com), and [structurally sound masonry research](https://dspace.mit.edu/entities/publication/559ae388-f9bc-4433-874c-36cce7af9178?utm_source=chatgpt.com) | Implementation foundations and the distinction between geometric plausibility and structural feasibility. |

**Bottom line:** TCE should author a vocabulary of buildable systems, not just attractive façades. Keep architectural decisions and history in Rust, make module compatibility machine-checkable, and let Unreal render a replaceable, streamed interpretation of that persistent design. That offers the best balance of emergence, performance control and solo-developer maintainability.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9290a-781c-83e9-9516-00c4e7884332)
