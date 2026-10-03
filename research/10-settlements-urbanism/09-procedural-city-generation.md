# Procedural roads, blocks and parcels for TCE

**Engineering assessment — 27 September 2026**

## Recommendation

TCE should use a **simulation-driven, event-based growth system with persistent geometry**, not a conventional whole-city generator.

The best fit is a hybrid:

* **Road proposals:** Parish–Müller-style global goals and local constraints, implemented as a work queue; terrain-aware pathfinding for longer connections.
* **Road orientation:** optional tensor-field guidance for new districts, rather than a field that continuously regenerates existing streets.
* **Parcels:** frontage-aware splitting and recursive oriented-bounding-box subdivision first; straight-skeleton subdivision as a later specialist tool.
* **Persistence:** explicit road, boundary, parcel and ownership-history records in Rust. Unreal receives geometry changes and renders them; it does not decide which parcels exist.

This combines the controllable growth of Parish and Müller, the directional guidance of Chen et al., the temporal feedback of Weber et al., and the parcel-editing work of Vanegas et al. None of those systems, by itself, supplies TCE’s persistent individual-level simulation. [CGL ETHZ](https://cgl.ethz.ch/Downloads/Publications/Papers/2001/p_Par01.pdf)

The most important distinction is:

> **A generator asks what geometry should exist now. TCE must ask which actor changes which existing geometry, for what reason, with what resources and consequences.**

For your hardware and solo-development constraints, I would spend the initial engineering effort on **robust local editing, stable identity and replayable transactions**, not on implementing the most sophisticated city-layout algorithm.

---

## 1. Road-generation options

### 1.1 Parish and Müller: L-systems and constrained growth

In [*Procedural Modeling of Cities* — SIGGRAPH 2001](https://cgl.ethz.ch/Downloads/Publications/Papers/2001/p_Par01.pdf?utm_source=chatgpt.com), road growth is expressed using an extended L-system. The crucial decomposition is between **global goals**, which propose development directions and patterns, and **local constraints**, which modify or reject proposals because of existing roads, intersections, terrain and other restrictions. Population maps and pattern controls guide the resulting network. [CGL ETHZ](https://cgl.ethz.ch/Downloads/Publications/Papers/2001/p_Par01.pdf)

**TCE adaptation:** retain that decomposition, but implement it as ordinary Rust operations rather than a general-purpose string-rewriting interpreter:

```
Propose connection
→ query nearby roads, boundaries and obstacles
→ shorten, intersect, snap, reroute or reject
→ obtain resources / rights / authorization where applicable
→ construct
→ commit topology change
→ issue any justified follow-up proposals
```

This is naturally incremental. A village can extend one path without regenerating its older streets.

Its main weakness is that **geometric growth does not explain economic demand**. A population-density map can produce attractive streets, but TCE should derive proposals from actual settlement needs: an inaccessible dwelling, a market connection, an irrigation crossing, or a route whose detour is becoming costly.

**Verdict:** the best basic road-growth architecture for v1, provided that simulation decisions replace externally painted growth demand.

### 1.2 Tensor fields: coherent direction without prescribing every street

[Chen et al., *Interactive Procedural Street Modeling* — SIGGRAPH 2008](https://www.sci.utah.edu/~chengu/street_sig08/street_project.htm?utm_source=chatgpt.com) represents preferred street directions with tensor fields. Their eigenvectors define local directional axes; tracing these directions produces street curves. Regular grids, radial arrangements and boundary-following patterns can be blended and edited. The paper also addresses connectivity: independently tracing curves is insufficient, so tracing and seeding must be coordinated. [Oregon State University Engineering](https://web.engr.oregonstate.edu/~zhange/images/street_sig08.pdf)

For TCE, a field should answer **“which directions are preferred here?”**, not **“which streets must exist here?”**

A government establishing a planned district could specify an orientation, block-spacing preference and major public spaces. A settlement following a valley could receive a weaker terrain-alignment preference. Individual proposals would still have to satisfy terrain, access, property and construction constraints.

The danger is global regeneration. Moving a field’s center or changing its orientation can alter many derived streets. That is useful in an artist’s editor but destructive to persistent history.

**Verdict:** excellent optional guidance for newly authorized development. Do not make it the authoritative representation of the whole city.

### 1.3 Agent-based road and land development

Lechner et al.’s [*Procedural Modeling of Urban Land Use* — 2007 technical report](https://repository.lib.ncsu.edu/items/421f6227-f6d2-4ac9-8c76-582eaa90f9de?utm_source=chatgpt.com) uses distinct property-development and road-development agents. Road extenders provide access to undeveloped territory; connectors reduce excessive detours; property developers prospect, build and evaluate profitability. The implementation uses NetLogo and subsequently vectorizes raster development geometry. An [author-uploaded copy](https://arxiv.org/pdf/2510.15877?utm_source=chatgpt.com) is available on arXiv; its later upload date should not be mistaken for the original research date. [NCSU Libraries](https://repository.lib.ncsu.edu/items/421f6227-f6d2-4ac9-8c76-582eaa90f9de)

The useful lesson is the separation of **different reasons for building roads**.

For TCE, distinguish at least:

**Access:** connect a dwelling, field, workshop or landing place.

**Connectivity:** reduce a costly detour between established destinations.

**Capacity and maintenance:** improve an existing route rather than creating another one.

These need not be separate continuously ticking software agents. They can be proposal-producing behaviors belonging to households, institutions and governments.

**Verdict:** strong causal fit, but avoid running a second population of expensive “city-planning agents” alongside all 50,000 people. Aggregate their experienced access problems and transport demand into bounded planning work.

### 1.4 Time-evolving geometric simulation

[Weber et al., *Interactive Geometric Simulation of 4D Cities* — 2009](https://www.peterwonka.net/Publications/pdfs/2009.EG.Weber.UrbanSimulation.FinalVersion.pdf?utm_source=chatgpt.com) is particularly relevant because it models change over time rather than only generating a final layout. Streets, land use, traffic and development interact, with explicit geometric regions and planned construction. However, its land-use simulation is deliberately **not an individual-agent model**; the authors use simplified, sampled regional updates. [Peter Wonka](https://www.peterwonka.net/Publications/pdfs/2009.EG.Weber.UrbanSimulation.FinalVersion.pdf)

**TCE adaptation:** use its separation of planning, development and feedback, but feed those processes from TCE’s own actors and institutions. Keep transport accessibility, anticipated returns and construction completion as distinct states.

A proposal should not immediately receive the accessibility benefits of a road that has not been built. Otherwise the simulation can create self-fulfilling development with no labor or material cost.

**Verdict:** the closest conceptual precedent for the overall temporal architecture, not a ready-made economic model.

### 1.5 Terrain-aware shortest paths

[Galin et al., *Procedural Generation of Roads* — 2010](https://perso.liris.cnrs.fr/egalin/Articles/2010-roads.pdf?utm_source=chatgpt.com) computes roads using weighted, anisotropic path search. Costs account for terrain and construction considerations; extensions handle curvature, bridges and tunnels. Adding orientation to the search state permits curvature constraints, but increases computational work. [LIRIS](https://perso.liris.cnrs.fr/egalin/Articles/2010-roads.pdf)

This solves a different problem from network growth:

> Given that a connection is justified, where should it go?

For TCE, use it for inter-settlement routes, hillside access and major connections. Start with relatively inexpensive surface-route search. Enable bridge, tunnel or substantial earthwork candidates only when the relevant capabilities and resources exist.

Do not run an expensive engineering optimizer for every worn footpath. Informal paths can follow repeated movement; engineered improvements deserve a more deliberate search.

**Verdict:** an important component of the hybrid, not a complete city generator.

---

## 2. Blocks and parcels

### 2.1 Blocks are not the same thing as parcels

For TCE, keep three concepts separate:

**Transport network:** connected routes and junctions.

**Block:** a region enclosed by relevant physical boundaries.

**Parcel or land unit:** a persistent area associated with claims, use and development.

That distinction lets farms and roadside plots exist before a closed urban street block forms.

For enclosed urban blocks, use a planar embedding or half-edge representation. Split at genuine at-grade junctions, order incident edges around vertices, and walk face boundaries. **Do not enumerate every graph cycle**: most cycles are not individual block faces.

Once an embedding is valid, traversing each directed edge once gives a linear-time face walk. Intersection detection, embedding updates and polygon repair are separate costs.

Recommended rules:

* A bridge passing over a street does not automatically create a junction or split a ground-level block.
* A dead end does not independently enclose a new block.
* Road centerlines are not parcel boundaries; reserve the actual road corridor before deriving developable land.
* A streaming-tile border must not become a fictitious cadastral boundary.

These are requirements of the proposed TCE representation, not claims that a particular generator implements them all.

### 2.2 Recursive oriented-bounding-box subdivision

An oriented bounding box, or OBB, is a rectangle aligned to the polygon rather than necessarily to world axes. Recursive subdivision computes a box, proposes a cut across it, clips the polygon, and repeats until the children satisfy stopping conditions.

CityEngine’s documented implementation adds practical rules: alternative cut directions for access, snapping to contour vertices, minimum widths and stable random seeds. Its current documentation distinguishes recursive, offset, skeleton and no-subdivision modes. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/help/help-layers-block-parameters.htm)

**Advantages for TCE:** a small implementation surface, straightforward debugging, useful rectangular plots, and easy confinement to one undeveloped property.

**Weaknesses:** area-only stopping rules can produce narrow slivers, awkward concave remnants or inaccessible interior lots.

My recommended implementation would evaluate several candidate cuts instead of blindly bisecting:

\[
S\_{\text{cut}}=
w\_a P\_{\text{access}}
+w\_w P\_{\text{width}}
+w\_s P\_{\text{shape}}
+w\_d P\_{\text{disruption}}
+w\_t P\_{\text{target area}}.
\]

Here each \(P\) is a penalty; weights are design parameters, not empirically established constants. Reject cuts violating hard requirements, then choose the lowest-scoring feasible cut.

For existing settlements, **disruption should normally be a hard constraint**: a convenient geometric subdivision must not silently cut through a standing house.

**Verdict:** best default for v1.

### 2.3 Frontage-strip and offset subdivision

Offset subdivision reserves a strip of land extending inward from a street frontage, then divides that strip into lots. CityEngine documents this as a separate method and permits further recursive subdivision. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/help/help-layers-block-parameters.htm)

For TCE, this is particularly useful when development proceeds along an existing route before the interior is urbanized. The rear area can remain garden, field, common land or an independent undeveloped parcel.

Do not automatically fill every residual interior. A geometrically available region is not evidence that anyone has a reason or permission to build there.

**Verdict:** implement alongside OBB splitting. It provides a useful distinction between roadside development and comprehensive block subdivision.

### 2.4 Straight-skeleton subdivision

A straight skeleton is formed by moving polygon edges inward and tracking the events where the moving boundaries meet. It partitions the interior into regions associated with boundary edges. Unlike a medial axis, it is based on polygon-edge propagation rather than a general nearest-boundary distance construction. CGAL supplies a production-oriented implementation, including weighted skeletons and polygons with holes. [CGAL Documentation](https://doc.cgal.org/latest/Straight_skeleton_2/index.html)

For parceling, skeleton-derived regions can be grouped by frontage, sliced approximately perpendicular to the street, and repaired by merging undesirable remnants.

This is useful for irregular blocks where a single global rectangular orientation is inappropriate. But **the skeleton is an intermediate structure, not the final parcel layout**.

The implementation burden is substantial: simultaneous events, nearly parallel edges, short edges, holes and numerical tolerances all matter. My assessment is that a solo developer should integrate and test a mature implementation rather than ask coding agents to invent a fully robust skeleton engine.

**Verdict:** a good second-stage capability; not necessary to establish TCE’s basic growth loop.

### 2.5 Vanegas et al.: preserving edits is not preserving ownership

[Vanegas et al., *Procedural Generation of Parcels in Urban Modeling* — 2012](https://twak.org/project/parcels/?utm_source=chatgpt.com) directly addresses plausible parcel subdivision and persistence across editing. Its persistence mechanism maps parcels before and after an edit so that customization can survive changes to the surrounding block. [Twak](https://twak.org/project/parcels/)

This is valuable but weaker than TCE’s requirement.

An editor can reasonably transfer a building style from an old lot to a corresponding new lot. A simulation cannot automatically transfer a household’s property rights merely because another polygon occupies a similar relative position.

For TCE:

* Unaffected parcels retain their identities and boundaries.
* Splits and mergers create explicit parent–child history.
* Road widening requires a modeled change to rights and occupied land.
* Reconstruction may deliberately reorganize parcels, but only through an event.

**Verdict:** borrow the attention to stable correspondence and customization; make legal and historical identity explicit rather than inferred from geometric similarity.

### 2.6 A newer alternative: co-generating parcels and streets

[Chen, Song and Ortner, *Hierarchical Co-generation of Parcels and Streets in Urban Modeling* — Eurographics 2024](https://sutd-cgl.github.io/supp/Publication/projects/2024-EG-UrbanModeling/index.html?utm_source=chatgpt.com) alternates parcel splitting with street generation. Graph search ensures access to newly created parcels; a later optimization improves the joint layout. It therefore avoids treating roads and parcels as completely separate stages. [CDL](https://sutd-cgl.github.io/supp/Publication/projects/2024-EG-UrbanModeling/index.html)

This is attractive for a newly planned district or subdivision of a large estate.

For TCE, apply it only inside the area whose reorganization has been authorized. Existing streets, buildings, protected spaces and property boundaries outside that area should be fixed constraints.

**Verdict:** promising for planning institutions and large development projects. Its hierarchical construction is not, by itself, a historical simulation.

---

## 3. Trade-offs and performance evidence

### Engineering comparison

The following ratings are my implementation assessment, not measured benchmark results.

| Technique | Implementation burden | Incremental fit | Main cost or failure mode | Recommended role |
| --- | --- | --- | --- | --- |
| Constrained road-growth queue | Moderate | Excellent | Intersection handling, snapping and proposal quality | Core road mechanism |
| Tensor-guided tracing | Moderate–high | Good when restricted to new geometry | Field editing can invalidate extensive output | Optional district guidance |
| Development-agent simulation | High if built as another full simulation | Excellent conceptually | Repeated prospecting and route evaluation | Reuse TCE actors and demand |
| Terrain-aware path search | Moderate–high | Excellent for individual connections | Search extent, orientation states, bridge candidates | Long or difficult routes |
| Recursive OBB cuts | Low–moderate | Excellent per parcel | Slivers, poor access, concavity | Default subdivision |
| Frontage-strip subdivision | Moderate | Excellent | Corner treatment and unused interior geometry | Roadside development |
| Straight-skeleton parcels | High | Good per block | Numerical robustness and remnant repair | Specialist subdivision |
| Joint parcel–street optimization | High | Good for authorized development areas | Optimization can disturb existing geometry | Later planning feature |

Indexed local operations are the important scaling strategy. For a road insertion, query nearby edges and parcels rather than testing the candidate against the entire world. An R-tree supports that broad-phase search, but its performance still depends on spatial distribution and overlap; it is not a universal guarantee of constant-cost editing. [Docs.rs](https://docs.rs/rstar/latest/rstar/)

For parceling, the relevant workload is the **complexity of the affected polygons**, not simply the total city population. A tiny edit to one ordinary parcel and a road cutting a highly fragmented district are very different transactions.

### Published and documented timings

These measurements are not directly comparable and should not be extrapolated into a claimed TCE frame rate.

| Source | Reported workload and time | What it establishes—and what it does not |
| --- | --- | --- |
| Parish & Müller, 2001 | Example containing approximately **13,000 buildings**: street graph in **under 10 seconds**; lot division and building generation approximately **10 minutes** | Distinguishes cheap layout from more expensive downstream generation. Historical prototype timing, not runtime growth on your PC. [CGL ETHZ](https://cgl.ethz.ch/Downloads/Publications/Papers/2001/p_Par01.pdf) |
| Weber et al., 2009 | **3,005 streets**, **58.3 seconds** total simulation on a **2 GHz PC** | Time-evolving geometry was practical at this scale. It is not an individual-level 50,000-person benchmark. [Peter Wonka](https://www.peterwonka.net/Publications/pdfs/2009.EG.Weber.UrbanSimulation.FinalVersion.pdf) |
| Galin et al., 2010 | One **60 × 60** search-grid comparison: **0.110 s** without orientation states versus **1.750 s** with curvature-related orientation states | Demonstrates the price of a richer search state. These are route calculations, not whole-city timings. [LIRIS](https://perso.liris.cnrs.fr/egalin/Articles/2010-roads.pdf) |
| Epic City Sample PCG, UE 5.8 documentation | Complete regeneration described as taking **a few minutes**; no controlled hardware benchmark supplied there | Demonstrates an engine-native production workflow, not frame-budget incremental simulation. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?lang=en-US) |

I did not establish comparable, reproducible timings for the 2008 tensor-field system or the 2012/2024 parcel methods from the accessible material. Their interactive or quality claims should not be converted into invented milliseconds.

**Conclusion:** the literature supports local generation as a reasonable architectural choice. It does not establish 60 fps with 50,000 individually simulated people on your specified machine.

---

## 4. Precedents and what to take from them

### Foundation: organic development as a shipped interaction model

[Foundation](https://store.steampowered.com/app/690830/Foundation/?utm_source=chatgpt.com), released in January 2025, combines gridless development, procedural generation, modular construction and area-based player guidance. [Steam Store](https://store.steampowered.com/app/690830/Foundation/)

**Lesson for TCE:** separate the actor’s intention—where development is encouraged—from the exact placement of every structure.

However, the public description does not establish its internal parcel topology, numerical methods or performance at TCE’s scale. Treat it as a design precedent, not a documented algorithm to reproduce.

### Shadows of Doubt: integration complexity dominates

[Shadows of Doubt](https://store.steampowered.com/app/986130/Shadows_of_Doubt/?utm_source=chatgpt.com) is a shipped example of combining generated urban environments with simulated citizens. In an early development account, its creator describes nested building and floor presets, simplified spatial layouts to keep pathfinding manageable, and the difficulty of getting interdependent procedural systems to function together. Those are development-history observations, not current implementation specifications. [Steam Store](https://store.steampowered.com/app/986130/Shadows_of_Doubt/)

**Lesson for TCE:** constrain the geometry vocabulary enough that navigation, interiors, entrances and construction can agree. A visually impressive generator is not useful when its output routinely breaks the simulation.

### Epic City Sample: two distinct generations of workflow

The original [City Sample](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration?lang=en-US&utm_source=chatgpt.com) used a Houdini-driven workflow producing structured data consumed by Unreal systems. It also demonstrates different representations for nearby and distant people rather than treating every visible citizen identically. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration?lang=en-US)

**Important 2026 update:** the [UE 5.8 City Sample PCG workflow](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?lang=en-US&utm_source=chatgpt.com) now builds a city using **18 interdependent PCG graphs**, including roads, lots and buildings. But Epic explicitly says that this PCG demo **does not include the traffic and pedestrian Mass AI setup**. Changes to early graphs can rebuild downstream stages. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?lang=en-US)

**Lesson for TCE:** study its road geometry, building assembly, debugging views and actor management. Do not adopt wholesale regeneration as the persistence model.

### CityEngine practice: useful algorithms, different product boundaries

CityEngine is a strong reference for separating street layout, lot formation and building rules. Its block documentation also illustrates an important incompatibility with TCE: opening a street loop can make the corresponding generated block disappear. That is reasonable for derived editor geometry, but should not erase persistent land records. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/help/help-layers-block-parameters.htm)

The [CityEngine SDK](https://github.com/Esri/cityengine-sdk?utm_source=chatgpt.com) exposes the Procedural Runtime, which applies CGA rules to initial geometry—for example, generating a building from a parcel. **It is not an open-source autonomous road-and-parcel simulation.** The inspected release is **3.4.12206**, associated with CityEngine **2026.0**. Its terms distinguish Apache-licensed examples from SDK binaries and restrict redistribution unless expressly permitted. [GitHub](https://github.com/Esri/cityengine-sdk)

The Unreal integration, formerly Vitruvio, can generate buildings at runtime. However, the inspected **2.5 release targets UE 5.7 and CityEngine 2025.1/SDK 3.3**, not explicitly UE 5.8. Its source-code license does not remove the bundled runtime’s restrictions. [GitHub](https://github.com/Esri/vitruvio/releases/latest)

**Recommendation:** useful for authoring and evaluation; do not make the shipped TCE runtime depend on it without resolving redistribution and UE 5.8 compatibility first.

---

## 5. Open-source implementation choices

These are versions or branches observed during this review, not a tested mutually compatible dependency set.

| Resource | Version inspected | Useful contribution | Adoption caution |
| --- | --- | --- | --- |
| [ProbableTrain/MapGenerator](https://github.com/ProbableTrain/MapGenerator?utm_source=chatgpt.com) | `master`, inspected September 2026 | Accessible TypeScript city-map generator with staged controls and export | README specifies **LGPL-3.0**. Treat as reference/prototype code, not a Rust simulation kernel. [GitHub](https://github.com/ProbableTrain/MapGenerator) |
| [`geo`](https://docs.rs/geo/0.33.1/geo/) | **0.33.1** | Rust geometry types, measures, queries, topology-related operations and triangulation support | Choose and test a consistent subset; a geometry library does not provide parcel history. [Docs.rs](https://docs.rs/geo/latest/geo/) |
| [`rstar`](https://docs.rs/rstar/0.13.0/rstar/) | **0.13.0** | Rust spatial indexing for nearby road/parcel queries | Maintain indexes transactionally with authoritative geometry. [Docs.rs](https://docs.rs/rstar/latest/rstar/) |
| [`i_overlay`](https://docs.rs/i_overlay/9.0.0/i_overlay/) | **9.0.0** | Polygon Boolean operations, clipping and slicing; integer and floating-point interfaces | Respect documented coordinate limits and fill rules. It is not a straight-skeleton implementation. [Docs.rs](https://docs.rs/i_overlay/latest/i_overlay/) |
| [CGAL straight skeleton](https://doc.cgal.org/latest/Straight_skeleton_2/index.html?utm_source=chatgpt.com) | **6.2.1** documentation | Mature C++ skeleton and offsetting facilities | Additional integration burden; this package is GPL-licensed. Review licensing before shipping it inside a proprietary product. [CGAL Documentation](https://doc.cgal.org/latest/Straight_skeleton_2/index.html) |
| [CityEngine for Unreal](https://github.com/Esri/cityengine_for_unreal?utm_source=chatgpt.com) | **2.5** | Source reference for procedural building integration | UE 5.7 release target; separate runtime redistribution restrictions. [GitHub](https://github.com/Esri/vitruvio/releases/latest) |

For v1, my preferred starting stack is **Rust-owned topology + `rstar` + a narrowly wrapped polygon-operation backend + selected `geo` utilities**.

Pin versions in the lockfile and isolate library calls behind a small geometry interface. That makes it possible to replace a clipping backend without rewriting property transactions, save files and construction logic.

I found no verified public code release linked from the 2024 co-generation project page. Vanegas’s project links its implementation to CityEngine rather than presenting it as an independently reusable open-source parcel library. [CDL](https://sutd-cgl.github.io/supp/Publication/projects/2024-EG-UrbanModeling/index.html)

---

## 6. Recommended TCE architecture

The following is a proposed design, not a description of an existing engine.

### 6.1 Keep authoritative topology in Rust

Use stable identifiers and separate physical geometry from rights and visual representation:

```
RoadNode
RoadEdge          endpoints, alignment, corridor, state, revision
BoundaryEdge      shared geometric boundary
Parcel            boundary references, active state, lineage
Claim             rights or contested claims attached to land
AccessLink        entrance, frontage, easement or connection
Construction     actor, resources, work progress, intended change
UrbanTransaction  affected IDs, expected revisions, approved operations
RenderDelta       added / changed / removed visual representations
```

Shared boundaries should have one authoritative representation. Independently storing two supposedly identical parcel edges invites gaps and overlaps when only one side changes.

Ownership should not be encoded as “one owner field per polygon” unless that is explicitly sufficient for TCE’s institutions. Multiple claims, access rights or contested interests can refer to the same land without requiring duplicate physical polygons.

### 6.2 Let roads and land claims develop together

Do not enforce one universal sequence:

```
roads → blocks → parcels → buildings
```

Instead, support several sequences:

**Informal settlement:** occupation or land claim → repeated access → worn path → later improvement.

**Roadside growth:** existing route → new frontage parcel → dwelling/workshop → further subdivision.

**Planned district:** authorized boundary and public-space reservation → streets and parcels co-designed → construction.

**Infill:** existing parcel → permission, inheritance or sale → split/merge → additional structure or access lane.

A closed block is therefore sometimes an outcome of growth, not a prerequisite for it.

### 6.3 Generate proposals from persistent demand

Use aggregated observations from individual activity: unsuccessful access, travel-time costs, crowding, destination demand and maintenance condition.

A proposed road evaluator could use:

\[
U(r)=
B\_{\text{access}}(r)+B\_{\text{travel}}(r)+B\_{\text{development}}(r)
-C\_{\text{work}}(r)-C\_{\text{materials}}(r)
-C\_{\text{maintenance}}(r)-C\_{\text{disruption}}(r).
\]

This is a modeling structure, not a calibrated equation. Different actors perceive and weight those terms differently. A ruler may value ceremonial access; a household may value getting to its field; neither necessarily maximizes citywide transport efficiency.

Treat physical feasibility separately from authorization. Informal or illegal changes may still occur, but they should produce the appropriate conflict or enforcement consequences rather than bypassing the institutional simulation.

### 6.4 Use local, atomic geometry transactions

A transaction should identify its affected region and expected entity revisions, then produce one consistent change set.

For example, widening a street may require:

1. Reserving the added corridor and identifying affected claims and buildings.
2. Resolving or explicitly modeling acquisition, opposition and demolition.
3. Completing the necessary construction.
4. Committing changed boundaries, access and routing together.

Never publish a new road surface while the parcel system still considers the same strip an intact occupied house lot.

Prefer **local reconstruction inside a validated affected region** over an ambitious fully dynamic computational-geometry engine in the first implementation. Rebuilding a modest neighborhood’s derived embedding can be simpler than maintaining every invariant through a long series of microscopic edits. Preserve unaffected entity IDs regardless of how the local geometry calculation is performed.

### 6.5 Preserve history, including abandonment

Record parcel splits and mergers, road realignment, widened corridors, demolished structures and abandoned routes.

A useful conservation check for a subdivision transaction is:

\[
A\_{\text{parent}}
=
\sum A\_{\text{children}}
+A\_{\text{new public corridor}}
+A\_{\text{other explicitly reassigned land}},
\]

within the chosen numerical tolerance.

Retiring a parcel’s active geometry should not erase its historical identity. Similarly, an unused road may lose maintenance and become overgrown without instantly disappearing from land records or collective memory.

### 6.6 Treat numerical robustness as a core feature

Choose a coordinate policy early. I recommend regional coordinates, a documented quantization policy for topology-changing operations, and conversion to render coordinates only at the Unreal boundary.

Quantization is not a substitute for validation: snapping two near points together can itself collapse a narrow access strip or alter connectivity. Boolean-operation libraries also have explicit range and representation contracts; `i_overlay`, for example, documents limits narrower than the full storage range of its integer types. [Docs.rs](https://docs.rs/i_overlay/latest/i_overlay/)

Test near-collinear boundaries, tiny edges, acute corners, holes, duplicate points and almost-touching intersections. Reject or repair invalid proposals before they enter the persistent world.

### 6.7 Make asynchronous work deterministic

Background jobs should operate on snapshots with revision numbers. Commit only after confirming that the relevant state is still valid.

However, **worker completion order must not decide history**. Use stable logical ordering, such as simulation tick plus event ID. A slower machine should not assign land differently merely because one geometry job finished later.

When construction decisions require unfinished calculations, slow the simulation or wait at an appropriate logical boundary. It is acceptable for decorative rendering to lag; it is not acceptable for camera position or thread scheduling to decide which settlement receives a road.

### 6.8 Keep the Rust–Unreal boundary narrow

Use a versioned C-compatible interface with opaque handles, fixed-layout records and explicit buffer ownership. Do not pass Rust `Vec`, `String` or trait objects across as though they were C++ ABI types. Prevent panics and C++ exceptions from crossing an incompatible boundary. The Rustonomicon documents the relevant layout, ownership and unwinding issues. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

On the Unreal side, use a small C++ integration module to load and stage the DLL and its dependencies. Epic’s [third-party library integration documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com) covers the Windows loading and packaging mechanics. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

Send **batched changes**, not one call per road vertex or citizen. Keep Unreal object creation and component updates out of Rust worker jobs.

### 6.9 Use PCG for derived visual detail

UE’s partitioned, hierarchical and runtime PCG modes can generate and remove content around generation sources. That is useful for vegetation, roadside objects and other reconstructible detail. It is not a persistence mechanism for legal parcels or road history. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-pcg-generation-modes-in-unreal-engine)

My division of responsibility would be:

**Rust:** existence, boundaries, construction state, use, access and history.

**UE:** meshes, materials, instancing, visual wear, local clutter and representation level.

A distant citizen remains a simulated person, but need not remain a fully animated skeletal actor. Likewise, a distant building can retain its simulation record without keeping every interior object resident.

---

## 7. Frame budgets, testing and delivery order

### Proposed budgets—not benchmark claims

At 60 fps, the frame interval is **16.67 ms**. CPU and GPU execution overlap, so this should not be treated as one simple additive budget for all subsystems.

For an initial performance contract, I would target:

| Work category | Initial target |
| --- | --- |
| Applying urban changes on the UE game thread | Usually **≤0.5–1 ms per frame**, with spillover queued |
| Geometry planning and validation | Background work, subdivided into bounded work units |
| Ordinary local transaction computation | Investigate a **p95 target below 5 ms**; validate on a representative corpus |
| Exceptional district-scale operations | Explicit multi-step jobs, never an unbounded frame-thread operation |
| Rendering | Stream and reduce detail independently of authoritative simulation |

Those targets are deliberately provisional. No cited source establishes them on your specific i9/4070 Ti system.

Also benchmark maximum simulation speed. A system that handles growth at real-time speed may accumulate an unbounded queue when centuries are accelerated. Track **transaction arrival rate versus completion rate**, not only frame time.

### Test the geometry and the history separately

Geometry tests should cover valid polygons, shared-edge agreement, area conservation, access connectivity, genuine at-grade intersections and correct behavior at streaming boundaries.

History tests should verify that unrelated edits do not change parcel IDs, that old claims remain traceable, and that saving and replaying the same events produces the same world.

For plausibility, measure distributions rather than inspect only screenshots: block areas, frontage widths, plot depths, intersection degrees, route detours, inaccessible-lot frequency and demolition caused by growth. Different institutional configurations should produce different distributions without requiring a fixed sequence of eras.

Coding agents are well suited to implementing bounded operations and generating regression cases. Give them narrow invariants—such as “split this parcel while conserving area and maintaining access”—rather than “write a robust city generator.”

### Delivery order

**First milestone:** a headless Rust world that can add and intersect paths, establish roadside parcels, split one parcel, save, reload and replay. Test it extensively before building detailed streets in Unreal.

**Second milestone:** demand-driven growth, construction delays, maintenance, parcel lineage and local route invalidation.

**Third milestone:** Unreal delta rendering, streaming, representation levels and profiling with the full population workload.

**Fourth milestone:** tensor-guided planned districts, sophisticated frontage handling and selective straight-skeleton subdivision.

**Later:** joint street–parcel optimization, major redevelopment and richer engineering of difficult roads.

---

## 8. Primary reading and viewing path

The papers and implementation links are embedded above. The most useful sequence is:

**Road foundations:** [Parish & Müller’s paper](https://cgl.ethz.ch/Downloads/Publications/Papers/2001/p_Par01.pdf?utm_source=chatgpt.com), followed by [Chen et al.’s project page and video](https://www.sci.utah.edu/~chengu/street_sig08/street_project.htm?utm_source=chatgpt.com).

**Simulation-driven growth:** [Lechner et al.’s report](https://arxiv.org/pdf/2510.15877?utm_source=chatgpt.com) and [Weber et al.’s 4D-city paper](https://www.peterwonka.net/Publications/pdfs/2009.EG.Weber.UrbanSimulation.FinalVersion.pdf?utm_source=chatgpt.com).

**Parcels:** [Vanegas et al.’s project materials](https://twak.org/project/parcels/?utm_source=chatgpt.com), the [Eurographics presentation](https://www.youtube.com/watch?v=529g5dxkOlg&utm_source=chatgpt.com), and [CityEngine’s algorithm descriptions](https://doc.arcgis.com/en/cityengine/latest/help/help-layers-block-parameters.htm?utm_source=chatgpt.com).

**Newer planning work:** [2024 hierarchical co-generation project, paper link and video](https://sutd-cgl.github.io/supp/Publication/projects/2024-EG-UrbanModeling/index.html?utm_source=chatgpt.com).

**UE implementation reference:** [City Sample PCG for UE 5.8](https://dev.epicgames.com/documentation/unreal-engine/city-sample-pcg-for-unreal-engine?lang=en-US&utm_source=chatgpt.com), alongside the [runtime generation-mode documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-pcg-generation-modes-in-unreal-engine?utm_source=chatgpt.com).

## Bottom line

**Build TCE around persistent local edits, not repeated city generation.**

For v1, a constrained road-proposal queue, terrain-aware routing, frontage development and OBB parcel splitting provide the strongest balance of control, explainability and implementation cost. Tensor fields and straight skeletons should improve particular decisions—not own the world.

The defining achievement will not be generating a convincing city in one pass. It will be letting a convincing city **change for a century without losing its people’s boundaries, investments, access or history**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928fd-2868-83ea-bdfc-e3345a5436f4)
