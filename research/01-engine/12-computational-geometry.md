# Computational geometry for TCE: parcels, footprints, roofs, and roads

**Engineering assessment as of September 27, 2026.** Library versions below are those verified during this review. Published benchmark results are distinguished from proposed TCE targets; I have not benchmarked these libraries on your hardware.

## Executive recommendation

**Use a Rust-first, fixed-precision planar geometry layer for land and footprints, but do not make a general straight-skeleton implementation a prerequisite for generating buildings.**

My recommended starting stack is **`geo` for geometry types, queries, and validation; direct `i_overlay` calls behind a small adapter for authoritative booleans, slicing, and precision control; an R-tree for spatial filtering; and authored roof primitives for ordinary buildings.** Evaluate CGAL separately for general hip roofs and more demanding architectural geometry.

An important version distinction: **`geo 0.33.1` depends on i\_overlay 4.5.x, not the standalone 9.0.0 release.** For the lowest integration risk, begin with `geo 0.33.1` and a tested, pinned i\_overlay 4.5.2 adapter. Evaluate 9.0.0 as a deliberate upgrade rather than assuming its API and numeric features are available through `geo`. Both lines expose useful polygon operations; 9.0.0 adds a substantially different, wider integer-engine API. [Docs.rs](https://docs.rs/crate/geo/latest)

The most important architectural boundary is:

> **Land boundaries and access rights are authoritative simulation state. Footprints are validated construction plans. Roofs and render meshes are replaceable derived geometry.**

A failed roof must not erase a parcel, alter its ownership, or interrupt the simulation.

---

## 1. Options: techniques and their appropriate uses

### 1.1 Robust polygon booleans

Union, intersection, difference, and symmetric difference are the foundation of road corridors, land reservation, setbacks, courtyards, and construction envelopes.

The practical implementation families are:

| Approach | How it works | Where it fits TCE |
| --- | --- | --- |
| **Integer/fixed-grid overlay** | Coordinates are represented on a controlled integer grid; intersections and winding information determine output boundaries. | Best default for authoritative land geometry. |
| **Floating-point overlay with precision management** | Uses floating coordinates, robust intersection handling, and sometimes snapping or fallback precision strategies. | Convenient for imported geometry and GIS interoperability. |
| **Exact predicates and exact constructions** | Makes topological decisions and constructs intersections using exact arithmetic appropriate to the problem. | Highest assurance, but heavier implementation and dependency costs. |
| **Raster or voxel operations** | Converts shapes into occupied cells, performs set operations, then optionally extracts contours. | Useful for influence fields or approximate planning, not a default for property boundaries. |

Clipper2 illustrates the first approach: both its integer API and floating-point API perform clipping internally with integers. Its floating API scales coordinates before clipping and scales them back afterward. GEOS OverlayNG provides a different approach, including fixed-precision snap rounding; its documentation distinguishes this from less robust floating-precision noding. [AngusJ](https://angusj.com/clipper2/Docs/Overview.htm)

**Fixed precision is a modeling decision, not a proof of exact Euclidean geometry.** Intersections can fall between grid points, short edges can collapse, and narrow polygons can disappear. Clipper’s robustness documentation explicitly discusses rounding-induced small features and their removal. Your simulation must decide what those outcomes mean. [AngusJ](https://angusj.com/clipper2/Docs/Robustness.htm)

For TCE, distinguish three policies:

* **Numerical precision:** the grid used to represent coordinates.
* **Simulation validity:** minimum frontage, passage width, building clearance, and parcel area.
* **Visual tolerance:** how much a render-only outline may deviate from the authoritative shape.

Do not implement all three as a single global `EPSILON`.

#### Exact predicates do not solve everything

Shewchuk-style adaptive predicates make orientation and incircle decisions reliable for the represented inputs. They address questions such as “which side of this line is the point on?” They do **not**, by themselves, make every constructed intersection exact or repair invalid polygon topology. Rust’s `robust` crate implements these predicate techniques. [CMU School of Computer Science](https://www.cs.cmu.edu/~quake/robust.html)

**Recommendation:** use established overlay implementations rather than assembling a new boolean engine from individually robust segment tests.

### 1.2 Offsetting: road width, setbacks, and eaves

“Offset” covers several operations that should remain distinct.

**Round buffering** approximates the region within a specified Euclidean distance of a shape. **Mitered offsetting** extends adjacent offset edges until they intersect. **Straight-skeleton offsetting** follows an inward-moving polygonal wavefront and also provides the connectivity needed for roofs. These are not interchangeable at corners. Clipper2 exposes join and end-cap choices; CGAL separately provides skeleton offsets and Minkowski-sum-based offsets. [AngusJ](https://angusj.com/clipper2/Docs/Units/Clipper.Offset/Classes/ClipperOffset/_Body.htm)

For road corridors, my recommended construction is:

```
road graph + centerlines + width profiles
    → local corridor polygons
    → union overlapping junction regions
    → subtract corridor from developable land
```

Keep the road graph after generating the polygons. A unioned pavement polygon does not retain lane connectivity, road identity, crossing priority, or ownership.

For constant-width roads, offset the centerline by half the total width. For varying width, construct tapered segment regions and junction patches rather than pretending a constant-distance buffer supports an arbitrary width profile.

**Important failure cases:** acute corners can create long miter spikes; inward offsets can split a polygon or eliminate it; holes can expand until they meet the exterior. These can be legitimate geometric results. Offset APIs therefore need to return multiple polygons or an empty result—not “one polygon, always.” Both Clipper2 and Rust skeleton-buffer documentation expose these distinctions. [AngusJ](https://angusj.com/clipper2/Docs/Units/Clipper.Offset/Classes/ClipperOffset/_Body.htm)

For setbacks, attach semantics to boundaries: street frontage, party wall, riverbank, rear boundary, and easement. A uniform inward buffer is insufficient when each category has different rules. Also, intersecting the inward half-plane of **every** edge is not a general solution for a concave parcel.

### 1.3 Simplification

There are three different tasks:

**Exact cleanup** removes duplicate coordinates, zero-length edges, and redundant collinear vertices without intentionally changing the region.

**Tolerance-based simplification** reduces visual complexity. Douglas–Peucker uses a distance criterion; Visvalingam–Whyatt removes vertices according to triangle-area importance.

**Topology-aware simplification** additionally tries to avoid intersections or changes in connectivity. However, the name is not a universal guarantee: `geo` explicitly warns that `SimplifyVwPreserve` can still produce invalid polygons, including an interior ring moving outside its exterior. Its tolerance also has area semantics, not ordinary distance semantics. [Docs.rs](https://docs.rs/geo/latest/geo/)

**TCE policy:** perform conservative cleanup on authoritative land. Put aggressive simplification on derived render outlines.

For adjacent parcels, simplify their **shared boundary once**, then use that result for both parcels. Independently simplifying each ring is an invitation to gaps and overlaps.

### 1.4 Parcel subdivision and polygon splitting

A parcel generator needs more than an algorithm that produces smaller polygons. It needs constraints on access, frontage, depth, shape, and identity.

| Method | Strengths | Weaknesses |
| --- | --- | --- |
| **Street-aligned or oriented-bounding-box recursive splitting** | Small implementation; predictable shapes; easy area and frontage controls. | Can create inaccessible interior lots unless access is checked explicitly. |
| **Straight-skeleton strips** | Natural way to allocate depth inward from streets and construct back-to-back lots. | More complicated corner handling; skeleton faces still need subdivision and cleanup. |
| **Clipped Voronoi or power diagrams** | Useful for organic initial allotments and spatial allocation around seeds. | Street access and useful frontage do not follow automatically. |
| **Constraint/optimization-based subdivision** | Can optimize several planning goals together. | More tuning, search cost, and failure handling than a solo developer needs initially. |

The directly relevant precedent is **Vanegas et al., “Procedural Generation of Parcels in Urban Modeling” (2012)**. It combines skeleton-based street strips and oriented-bounding-box subdivision, with explicit attention to usable parcel shapes and persistence through edits. It does not simply interpret every skeleton face as a finished parcel. [Scribd](https://www.scribd.com/document/805029451/Computer-Graphics-Forum-2012-Vanegas-Procedural-Generation-of-Parcels-in-Urban-Modeling)

My suggested first implementation is frontage-aware recursive splitting:

1. Select a street-aligned split direction.
2. Propose a cut satisfying target frontage or area.
3. Split with a robust polygon/polyline operation.
4. Validate every resulting connected component.
5. Commit only when access and land-accounting constraints hold.

A concave polygon cut by one line can produce **more than two connected components**. A line through a vertex, along an existing edge, or tangent to a boundary needs an explicit convention.

For target-area splitting, a half-plane sweep with binary search is practical, but the grid makes the result discrete: stop at a declared area tolerance rather than searching indefinitely for an impossible exact value.

**Do not regenerate an entire occupied block when a road moves.** Apply the edit to affected boundaries, preserve parcel identity where appropriate, and record actual splits, merges, or acquisitions as simulation events.

### 1.5 Straight skeletons for roofs

A straight skeleton is obtained by moving polygon edges inward while keeping them parallel to their initial directions. Vertices trace straight segments; edge collapses and split events alter the moving boundary. This produces both a ridge network and regions associated with the original edges. CGAL supports polygons with holes and weighted variants. [CGAL Documentation](https://doc.cgal.org/latest/Straight_skeleton_2/index.html)

For an equal-pitch hip roof, lifting an unweighted skeleton gives:

\[
z = z\_{\mathrm{eave}} + t\tan(\theta),
\]

where \(t\) is the inward wavefront distance and \(\theta\) is the roof pitch. Here, \(t\) comes from the generating wavefront—not a generic closest-point distance to the finite polygon boundary.

#### Hip roofs are not gable roofs

An ordinary equal-speed skeleton naturally produces **hip-like** roof regions. A gable requires additional intent: selected vertical ends, a ridge orientation, or explicitly designed roof masses.

CGAL’s current extrusion API is useful here. Unlike its ordinary weighted-skeleton interface, `extrude_skeleton` permits **zero weights to represent vertical extrusion of selected edges**. It also supports per-edge angles or weights and height truncation. Its output is a closed triangulated surface, but the documentation warns that certain height-limited configurations can have non-local self-intersections. [CGAL Documentation](https://doc.cgal.org/latest/Straight_skeleton_2/group__PkgStraightSkeleton2Extrusion.html)

For TCE, my preferred order is:

**Authored shed/gable/hip primitives → combinations of simple building masses → general skeleton roofs for shapes that genuinely need them.**

A mathematically valid roof is not necessarily architecturally plausible. Ridge direction, construction span, roof material, additions, drainage, and courtyard treatment should come from your authored building rules.

#### Straight skeleton versus medial axis

The medial axis represents points with multiple nearest boundary features. For polygons, it can contain curved portions around reflex vertices; the straight skeleton remains piecewise linear and follows moving supporting lines. They coincide in important convex cases but are not generally identical. [Docs.rs](https://docs.rs/straight-skeleton/0.2.1/straight_skeleton/)

Use a **straight skeleton for roof topology and mitered inward structure**. Use a **medial axis for clearance, passage structure, or approximate interior centerlines**.

`boostvoronoi` is relevant to the latter because it supports segment sites, not just point sites. Its input segments must not improperly intersect or overlap; noding and interior filtering remain your responsibility. A Voronoi diagram of sampled boundary points is only an approximation to the segment-based medial axis. [Docs.rs](https://docs.rs/boostvoronoi/latest/boostvoronoi/)

---

## 2. Library trade-offs, maturity, and performance

### 2.1 Rust library assessment

“Maturity” below is an engineering judgment based on documented capabilities, integration history, numeric restrictions, and dependency burden—not a claim that a crate is bug-free.

| Library/version reviewed | Capabilities and maturity assessment | Recommended role |
| --- | --- | --- |
| **`geo 0.33.1`** | Broad Rust geometry ecosystem: types, predicates, validation, booleans, buffering, simplification, and triangulation integration. Booleans use i\_overlay 4.5.x. [Docs.rs](https://docs.rs/crate/geo/latest) | Public geometry vocabulary and general algorithms. |
| **`i_overlay 4.5.2`** | Pure Rust booleans, polygon slicing, line clipping, buffering, fixed-scale float overlays, and an OGC-valid-output option. MIT/Apache-2.0. [Docs.rs](https://docs.rs/crate/i_overlay/4.5.2) | Initial authoritative overlay backend, directly controlled through an adapter. |
| **`i_overlay 9.0.0`** | Current standalone release; supports selectable `i16`, `i32`, and `i64` engines and newer output/provenance facilities. Several major API generations separate it from geo’s dependency. [GitHub](https://github.com/iShape-Rust/iOverlay) | Upgrade candidate after regression testing. |
| **`clipper2 0.6.0`** | Rust wrapper around native C++ Clipper2 via `clipper2c-sys`. Established underlying algorithm family; additional native-build integration. The Rust interface is still evolving. [Docs.rs](https://docs.rs/clipper2/latest/clipper2/) | Independent comparison backend; reasonable production alternative. |
| **`clipper2-rust 1.2.0`** | Separate pure-Rust port, with clipping, offsets, PolyTree hierarchy, rectangle clipping, and simplification. Do not confuse it with the C++ wrapper or assume identical performance. [Docs.rs](https://docs.rs/clipper2-rust/latest/clipper2_rust/) | Worth testing when avoiding native dependencies is important. |
| **`cavalier_contours 0.9.0`** | Rust line-and-circular-arc polyline operations. Attractive when preserving arcs matters; uses floating geometric constructions and tolerance-sensitive processing. [Docs.rs](https://docs.rs/cavalier_contours/latest/cavalier_contours/) | Specialized curved-road or CAD-like authoring layer. |
| **`geos 11.2.2`** | Rust bindings to GEOS’s C API; includes validation, repair, prepared predicates, and geometry operations. The documented static build bundles GEOS 3.14.1. [Docs.rs](https://docs.rs/geos/latest/geos/) | Development oracle and import/repair tooling; not necessary in the shipping baseline. |
| **`spade 2.15.1`** | Delaunay and constrained Delaunay triangulation with exact predicate evaluation and refinement facilities. [Docs.rs](https://docs.rs/spade/latest/spade/) | Conforming surface triangulation where triangle quality or constraints matter. |

Two small supporting choices are useful: **`robust 1.2.0`** for bespoke predicates and **`boostvoronoi 0.12.1`** for segment Voronoi work. Neither replaces a polygon boolean library. [Docs.rs](https://docs.rs/robust/latest/robust/)

For ordinary roof-face and ground-fill triangulation, an Earcut-family implementation is a practical option. However, triangulation is not geometry repair: upstream Earcut explicitly does not guarantee correct results on all problematic inputs. Validate rings and holes first, and retain constraints when a conforming mesh is required. [GitHub](https://github.com/mapbox/earcut)

### 2.2 Straight-skeleton implementations deserve separate qualification

**CGAL 6.2.1** is the strongest documented general-purpose candidate in this review. Its skeleton package has a long integration history, supports holes and weighted skeletons, and exposes detailed construction and extrusion APIs. Its exact-predicate/inexact-construction versus exact-construction choices must still be understood; “uses CGAL” is not a substitute for selecting the appropriate kernel. [CGAL Documentation](https://doc.cgal.org/latest/Manual/packages.html)

The main obstacle for a proprietary TCE build is not Rust interoperability alone: **the straight-skeleton package is GPL-licensed, with commercial licensing available through CGAL’s licensing arrangements.** Do not assume all CGAL packages are LGPL, or that a permissively licensed wrapper changes the underlying package’s license. Resolve this before committing to a shipping dependency. [CGAL Documentation](https://doc.cgal.org/latest/Manual/packages.html)

There are Rust implementations, but they need a more cautious deployment policy:

**`geo-buffer 0.2.0`** exposes skeleton-based buffering and skeleton output. Its own documentation says the underlying published algorithm was shown to be incorrect and that the implementation modifies some edge cases. That is not proof the crate is unusable; it is a reason to require a substantial adversarial corpus before making it authoritative. [Docs.rs](https://docs.rs/geo-buffer/latest/geo_buffer/)

**`straight-skeleton 0.2.1`** provides skeletons, source-edge attribution, constrained propagation, and roof helpers, but operates within a restricted `i16` coordinate domain: **−16,384 through 16,383**, with `i32` predicates and `f32` wavefront calculations. At a 1 cm grid, that represents approximately a 327.67 m coordinate span. That can accommodate individual buildings, but it is not a drop-in world-scale geometry engine, and its output includes lattice rounding. [Docs.rs](https://docs.rs/straight-skeleton/0.2.1/straight_skeleton/)

**Recommendation:** qualify a Rust skeleton implementation on a restricted class of footprints first. General arbitrary concave roofs should remain optional until failure behavior is well understood.

### 2.3 What published benchmarks actually show

The iOverlay maintainer publishes a reproducible comparison using:

**iOverlay Rust 1.9.0; Clipper2 C++ 1.4.0; a 3 GHz six-core Intel i5; 40 GB DDR4.** These are historical versions, not the current releases. The following values are converted from the published seconds to milliseconds. [IShape Rust](https://ishape-rust.github.io/iShape-js/overlay/performance/performance.html)

| Published workload | iOverlay, single-threaded | Clipper2 |
| --- | --- | --- |
| Checkerboard, 481 squares | 1.117 ms | **1.017 ms** |
| Checkerboard, 130,561 squares | **424.643 ms** | 1,067.439 ms |
| Spiral, test-size parameter 1,024 | 3.572 ms | **2.941 ms** |

The results support **competitive performance with workload-dependent winners**, not a universal “Rust is faster” conclusion. The large checkerboards also bear little resemblance to one 20-edge parcel setback. [IShape Rust](https://ishape-rust.github.io/iShape-js/overlay/performance/performance.html)

A separate maintainer benchmark on an Apple M4 compares integer widths and threading. It shows wider arithmetic and parallelism have workload-dependent costs; small cases do not automatically benefit from multithreading. Its unpinned version context limits direct use for selecting a specific release. [IShape Rust](https://ishape-rust.github.io/iShape-js/overlay/performance/rust_i_overlay.html)

I did not locate a directly comparable Windows/i9 benchmark covering current Rust booleans, parcel splitting, and roof generation together.

For TCE, measure **vertices, intersections, holes, and output complexity—not citizen count**. Your benchmark corpus should include ordinary footprints, difficult courtyard buildings, local road junction unions, and burst edits affecting multiple blocks. Record p50/p95/p99 time, allocations, peak temporary memory, invalid outputs, and fallback frequency.

---

## 3. Precedents and lessons

### CityEngine: numerical and semantic stability require sustained work

CityEngine is an especially relevant production precedent because its operations include setbacks, subdivision, and multiple roof types.

Its changelog records fixes involving collinear vertices, holes, rectangular roofs, incorrect vertex merging, and memory explosions. In **CityEngine 2024.1**, internal geometry processing switched from float to double precision, with an explicit warning that some rules could behave differently afterward. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/cga/cga-changelog.htm)

**Lesson for TCE:** geometry upgrades can change generated history. Save the resulting authoritative boundaries and construction plans; do not assume regenerating them under a newer library will preserve the world.

CityEngine’s gable documentation also exposes architectural complications: its “even” option can produce non-planar faces, and indexed ridge alignment has specific shape restrictions. “Gable roof” is not one universally uncomplicated operation. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/cga/cga-roof-gable.htm)

### Procedural parcels research: persistence is part of the algorithm

The Vanegas et al. system treats parcel correspondence across edits as a first-class problem, alongside subdivision. This is directly applicable to a changing TCE settlement. [Scribd](https://www.scribd.com/document/805029451/Computer-Graphics-Forum-2012-Vanegas-Procedural-Generation-of-Parcels-in-Urban-Modeling)

**Lesson:** stable parcel IDs and boundary lineage are not optional metadata. A geometrically plausible re-subdivision that silently relocates everyone’s property is incorrect simulation behavior.

### Procedural architectural extrusions: preserve architectural intent

Kelly and Wonka’s **“Interactive Architectural Modeling with Procedural Extrusions” (2011)** describes richer profile-driven constructions, including overhangs, dormers, vertical walls, and more elaborate roof forms. Its project provides paper and implementation resources. [Twak](https://twak.org/project/procex/)

**Lesson:** a footprint plus one pitch value is too weak a representation for all architecture. TCE should retain building masses, profile rules, and roof intentions rather than reducing everything immediately to triangles.

### Egregoria: a close Rust city-simulation precedent

Egregoria’s March 2021 development report discusses a Rust straight-skeleton roof implementation, preservation of lots and parking through road changes, and handling map-action errors without crashing. This is an open-source development precedent, not evidence of a completed commercial geometry platform. [Douady Paris](https://douady.paris/blog/egregoria_8.html)

**Lesson:** road editing, lot persistence, and failure handling become intertwined quickly. Study that integration problem, not just its roof algorithm.

### Unreal City Sample: separate generated data from runtime representation

Epic’s City Sample uses Houdini-generated data to populate assets and support traffic, AI, and other systems. The documented workflow generates and exports city data for Unreal; it is not a benchmark of continuous, in-game parcel regeneration. The guide still identifies Houdini 18.5.532 as the version used to develop the sample. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/city-sample-quick-start-for-generating-a-city-and-freeway-using-houdini)

**Lesson:** copy the separation between city data and its rendered realization. Do not assume the sample proves that its generation pipeline belongs inside TCE’s simulation loop.

### Shipped-game evidence: useful, but do not infer an undocumented solver

Ubisoft’s **GDC 2017 “Ghost Recon Wildlands: Terrain Tools and Technology”** presents a dedicated open-world toolchain and rendering system shared with other projects. It is a shipped-game precedent for specialized procedural tooling, but the accessible overview does not establish which parcel or roof algorithm was used. [GDC Vault](https://gdcvault.com/play/1024029/-Ghost-Recon-Wildlands-Terrain)

A more specific robustness lesson comes from the open-source StrandedKitty skeleton library: its current implementation wraps CGAL through WebAssembly, while its author describes the earlier pure-TypeScript version as faster but less robust. [GitHub](https://github.com/StrandedKitty/straight-skeleton)

**Combined lesson:** production-ready procedural geometry is often about constraining inputs and isolating specialized components—not finding one library that does everything.

---

## 4. Recommended TCE design

The following is my proposed architecture, not a claim about measured performance.

### 4.1 Own the coordinate and topology contract

Use a planar, local metric coordinate system for parcel calculations. Keep terrain elevation and bridge/tunnel layers separate from planar ownership.

A sensible starting representation is:

| Layer | Proposed representation |
| --- | --- |
| Authoritative land coordinates | Global or chunk-addressed `i64` coordinates on a **1 cm grid** |
| Local boolean operations | Checked chunk-local `i32` coordinates, or an explicitly configured fixed-scale adapter |
| Roof and other geometric constructions | Building-local `f64`, followed by output validation |
| Rendering | Chunk/building-local vertex buffers converted at the Unreal boundary |

The 1 cm grid is a proposed engineering choice, not a universal requirement. Millimeter-scale joinery and surface detail should normally be render content rather than cadastral topology.

Use one declared origin and grid convention for interacting geometry. **Equal scale with inconsistent rounding origins is not enough to guarantee matching boundaries.**

iOverlay 4.5.2 documents fixed-scale overlays and custom adapters. The newer engine documents integer range restrictions; Clipper2 likewise warns that usable arithmetic range is smaller than the raw storage range and that intersection quality degrades at extreme coordinates. Range-check before entering a backend, including room for offsets and miter extensions. [Docs.rs](https://docs.rs/crate/i_overlay/4.5.2)

Represent land as a shared-boundary subdivision: vertices, edges, incident parcels, and stable IDs. This can be a compact half-edge-style structure rather than a general-purpose CAD framework.

An edge needs semantic metadata such as:

```
boundary ID
left/right parcel
street frontage or easement association
source edit / predecessor boundary
geometry version
```

Extract polygons for library calls, then reconcile the result into this authoritative structure.

### 4.2 Make edits transactional

A land-changing operation should follow:

```
read immutable local state
    → calculate candidate result
    → validate geometry
    → validate access and land accounting
    → atomically commit boundary and ownership changes
    → enqueue derived building/render updates
```

Reject or defer the operation when validation fails. Keep the previous valid state.

**Never silently repair occupied land by dropping tiny output polygons.** A remainder must become an explicit parcel, common land, road reserve, or a documented merge. Similarly, a generic `make_valid` or zero-width-buffer repair may alter the intended region; it cannot decide ownership semantics.

For construction, compute a **buildable envelope**, then place authored building masses within it. Do not assume every irregular parcel should be filled by an equally irregular building.

### 4.3 Keep geometric and architectural failures separate

A roof job should consume a versioned construction plan and return either a validated mesh or a structured failure.

Suggested fallback order:

**Requested roof → simpler roof permitted by the same building style → revised permissible massing → deferred construction.**

Do not substitute an implausible flat roof merely because the preferred algorithm failed. A fallback is part of the authored construction rules.

Do not round skeleton vertices independently and assume roof faces remain planar. Preserve source-face relationships, construct each panel consistently, and validate shared ridge vertices, face planarity, and intersections before meshing.

### 4.4 Run geometry on changes, not per citizen or frame

Use a dependency chain:

```
road change
  → affected corridor and block
  → affected parcels
  → affected construction envelopes
  → affected building meshes
```

Spatially query dirty bounds before doing exact operations. Reuse worker scratch allocations. Batch related unions rather than repeatedly unioning an ever-growing accumulator; iOverlay explicitly provides reusable overlays and extraction of multiple results from one overlay graph. [Docs.rs](https://docs.rs/crate/i_overlay/4.5.2)

For the i9, start with a **small bounded geometry worker pool**, then profile against simulation and rendering contention. Using every available thread inside every polygon operation is not an appropriate default.

At 60 fps, the total frame budget is approximately **16.67 ms**. My initial goal would be **well under 1 ms of game-thread work for geometry-result commits in ordinary frames**, with generation itself off-thread and burst work queued. This is a target to test, not an expected library benchmark.

Version every job’s inputs. Discard stale results when a newer road or building edit supersedes them.

A wall-clock timeout does not safely terminate a stuck in-process library call. Use input limits and cooperative cancellation where available; test experimental or untrusted geometry backends in an isolated process during development.

### 4.5 Treat Unreal as the consumer, not the land authority

Send immutable geometry results through a narrow C ABI: explicit counts, plain data, opaque handles where needed, and ownership rules requiring buffers to be freed by the allocating side. Do not pass Rust `Vec` or C++ containers directly across the DLL boundary. Rust’s FFI documentation also requires deliberate unwinding behavior; panics must not accidentally cross an incompatible ABI boundary. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

UE 5.8’s Geometry Scripting documentation states that Blueprint-invoked functions run on the game thread and wait for internal parallel work to complete. It also lists important `UDynamicMeshComponent` limitations, including no Nanite, LOD, or instanced-rendering support in that documented path. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/geometry-scripting-users-guide-in-unreal-engine)

Therefore, use dynamic meshes selectively. Preserve reusable architectural parts for instancing and use an explicit strategy for completed, mostly static buildings. Do not assume that generating thousands of dynamic meshes automatically provides a scalable city renderer.

Your 64 GB RAM and GPU do not remove topology risks. As an illustrative calculation, **50,000 parcels × 16 vertices × 16 bytes per integer XY pair = 12.8 MB of coordinate payload**. Adjacency, metadata, temporary overlays, and meshes add more, but polygon storage itself need not dominate memory. Avoid rebuilding citywide temporary graphs for local edits.

### 4.6 Testing and acceptance criteria

Build one permanent geometry corpus before expanding the building grammar.

Include exact squares and rectangles, near-parallel edges, duplicate points, collinear runs, touching holes, narrow necks, acute corners, large coordinate offsets, chunk-boundary edits, cuts through vertices, and inward offsets that split or vanish. CityEngine’s historical fixes make these concrete production concerns rather than hypothetical adversarial cases. [ArcGIS Documentation](https://doc.arcgis.com/en/cityengine/latest/cga/cga-changelog.htm)

Validate four things:

| Category | Required checks |
| --- | --- |
| **Geometry** | Valid rings and hole containment; finite coordinates; no unexpected self-intersections |
| **Land accounting** | No unintended overlap or loss; matching shared edges; every residual assigned |
| **Semantics** | Required access preserved; frontage and minimum dimensions satisfied; IDs and ownership updated correctly |
| **Meshes** | Valid indices; no degenerate triangles; consistent shared ridges; appropriate manifoldness and no unintended self-intersections |

Add property tests and fuzzing, but account for quantization. Set identities such as reconstruction after splitting should be checked under the declared precision model; blindly demanding byte-identical results after arbitrary operation reorderings is too strong.

Use **Clipper2 or GEOS as an independent comparison implementation**. Comparing `geo` booleans against direct iOverlay calls is not independent validation because they share the underlying engine. [Docs.rs](https://docs.rs/crate/geo/latest)

For AI coding agents, the highest-value assignments are adapters, invariant checks, minimized failure cases, visual test fixtures, and regression tests. Do not make a new general-purpose straight-skeleton implementation their first geometry task.

---

## 5. Implementation order and source map

### Implementation order

**First:** land-coordinate contract, shared boundaries, booleans, road corridors, transactional edits, and a test corpus.

**Second:** frontage-aware parcel subdivision, buildable envelopes, simple building masses, and authored roofs.

**Third:** general skeleton roofs, curved authoring geometry, and more sophisticated subdivision—only when actual content requires them.

**Bottom line:** the best fit for TCE is not the most mathematically ambitious library stack. It is **a small, explicitly versioned Rust geometry layer whose results are validated before they become history**, paired with a constrained architectural system that can generate believable buildings without depending on arbitrary-polygon roof success.

### Linked documentation, papers, talks, and code

| Topic | Primary resources and version scope |
| --- | --- |
| **Rust core** | [`geo 0.33.1`](https://docs.rs/geo/0.33.1/geo/), [i\_overlay 4.5.2 package/docs](https://docs.rs/crate/i_overlay/4.5.2?utm_source=chatgpt.com), [i\_overlay 9.0.0 package/docs](https://docs.rs/crate/i_overlay/9.0.0?utm_source=chatgpt.com), [iOverlay source](https://github.com/iShape-Rust/iOverlay?utm_source=chatgpt.com) |
| **Alternative overlays and curves** | [Clipper2 documentation—2.0.0](https://angusj.com/clipper2/Docs/Overview.htm?utm_source=chatgpt.com), [`clipper2 0.6.0`](https://docs.rs/clipper2/0.6.0/clipper2/), [`clipper2-rust 1.2.0`](https://docs.rs/clipper2-rust/1.2.0/clipper2_rust/), [Cavalier Contours source](https://github.com/jbuckmccready/cavalier_contours?utm_source=chatgpt.com) |
| **Numerical foundations and performance** | [Shewchuk’s robust predicates and papers](https://www.cs.cmu.edu/~quake/robust.html?utm_source=chatgpt.com), [Clipper robustness notes](https://angusj.com/clipper2/Docs/Robustness.htm?utm_source=chatgpt.com), [historical iOverlay/Clipper2 benchmark and linked harness](https://ishape-rust.github.io/iShape-js/overlay/performance/performance.html?utm_source=chatgpt.com) |
| **Roofs and skeletons** | [CGAL 6.2.1 skeleton manual](https://doc.cgal.org/6.2.1/Straight_skeleton_2/index.html), [extrusion API](https://doc.cgal.org/6.2.1/Straight_skeleton_2/group__PkgStraightSkeleton2Extrusion.html), [CGAL licensing](https://www.cgal.org/license.html?utm_source=chatgpt.com), [`straight-skeleton 0.2.1`](https://docs.rs/straight-skeleton/0.2.1/straight_skeleton/?utm_source=chatgpt.com), [`geo-buffer`](https://docs.rs/geo-buffer/latest/geo_buffer/?utm_source=chatgpt.com) |
| **Parcel and architectural research** | [Vanegas et al. 2012—paper, presentation, and videos](https://twak.org/project/parcels/?utm_source=chatgpt.com), [Kelly and Wonka 2011—paper and implementation](https://twak.org/project/procex/?utm_source=chatgpt.com) |
| **Production precedents** | [CityEngine changelog](https://doc.arcgis.com/en/cityengine/latest/cga/cga-changelog.htm?utm_source=chatgpt.com), [Egregoria development report, March 2021](https://douady.paris/blog/egregoria_8.html?utm_source=chatgpt.com), [Epic City Sample generation guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/city-sample-quick-start-for-generating-a-city-and-freeway-using-houdini?utm_source=chatgpt.com), [Wildlands GDC 2017 talk](https://gdcvault.com/play/1024029/-Ghost-Recon-Wildlands-Terrain?utm_source=chatgpt.com) |

The remaining uncertainty is principally **comparative failure rate and tail latency on TCE-shaped workloads**, especially for Rust skeleton implementations—not whether the underlying techniques are capable of solving the problem. Those two measurements should decide upgrades after the initial Rust-first pipeline is working.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927c5-7ee8-83e9-bf4f-17b6d608fb5f)
