# Procedural terrain with realistic geomorphology for TCE

**Recommendation:** generate the regional drainage structure at coarse resolution, then progressively refine the terrain to 2 m with GPU erosion. Keep rivers, lake basins, and drainage outlets as explicit simulation data rather than reconstructing them later from a visually attractive heightmap.

For TCE, I would combine **coarse uplift/stream-power erosion, CPU depression handling, and GPU multiscale amplification**. I would not begin with full-resolution shallow-water erosion, a planetary tectonic simulator, or a large GPU graph-processing implementation.

The evidence supports this architecture: published work demonstrates erosion operators at **8192² on a 3080-class GPU**, while other work demonstrates fast coarse landscape evolution and GPU drainage routing. However, **I did not find a verified end-to-end benchmark for a complete 8192² world on an RTX 4070 Ti**. The timings below distinguish measured operations from proposed TCE budgets. [ResearchGate](https://www.researchgate.net/publication/382403597_Terrain_Amplification_using_Multi_Scale_Erosion)

*Research and implementation status checked through September 27, 2026.*

---

## 1. Options: what each technique contributes

### 1.1 Uplift and stream-power erosion: the best foundation for regional structure

A useful landscape-evolution model is

\[
\frac{\partial z}{\partial t}
=
U-KA^mS^n+H+D,
\]

where \(z\) is elevation, \(U\) uplift, \(A\) upstream drainage area, \(S\) downstream slope, \(K\) erodibility, \(H\) hillslope transport, and \(D\) deposition. This replaces the detailed movement of individual water parcels with an estimate of their long-term erosive effect.

Cordonnier et al.’s **2016 uplift/fluvial-erosion approach** combines a drainage graph with landscape evolution to produce coherent mountain ranges, watersheds, and valleys. Its important contribution is not merely erosion-shaped surface detail: topography and drainage develop together. [LIRIS](https://perso.liris.cnrs.fr/apeytavi/website/publication/hal-01262376/)

For implementation, a particularly convenient starting point is \(n=1\). With routing and drainage area fixed during one step, the incision equation gives this downstream-to-upstream update:

\[
z\_i^{new}
=
\frac{z\_i^{old}+U\_i\Delta t+c\_i z\_{r(i)}^{new}}
{1+c\_i},
\qquad
c\_i=\frac{K\_i A\_i^m\Delta t}{\ell\_i}.
\]

Here \(r(i)\) is the downstream receiver and \(\ell\_i\) the distance to it. This is an algebraic backward-Euler update, not a general nonlinear solver. It permits much larger steps than a comparable explicit update, although large steps still introduce approximation error and do not eliminate the need to update drainage. FastFlow provides a GPU formulation of this linear-slope case. [Aryamaan Jain](https://aryamaanjain.github.io/pdf/2024/fastflow.pdf)

**Fit for TCE:** excellent for the approximately 16–64 m grids that establish mountains and major valleys. Start with \(m=0.5,n=1\) as a **tuning choice**, not a universal geological calibration. Vary uplift, rock resistance, and effective runoff spatially; do not expect one parameter set to represent every landscape.

**Main limitation:** incision alone is insufficient. TCE also needs sedimentary valley floors, hillslopes, and retained basins—not an entire map of sharp ridges separated by narrow grooves.

### 1.2 Analytical landscape evolution: a promising coarse-generation alternative

Steer’s **Salève model, published in 2021**, uses analytical landscape-evolution relationships. Tzathas et al.’s **2024 analytical erosion method** goes further: geological age becomes an input to an analytical solution rather than the endpoint of thousands of temporal steps. Its two-dimensional implementation still iterates over drainage and elevation, with multigrid acceleration; “analytical” does not mean every output sample is an independent constant-time noise evaluation. [ESurf](https://esurf.copernicus.org/articles/9/1239/2021/)

This is attractive when the goal is to generate a plausible initial world rather than reproduce its exact geological history.

**Fit for TCE:** a strong alternative for the coarse terrain stage. The Rust `fastlem` library implements a Salève-based approach, but its maintenance status requires caution; see the implementation table below.

### 1.3 Explicit river-network-first generation

An alternative is to generate drainage topology first, assign river elevations and geometric characteristics, and construct surrounding terrain to agree with it.

Génevaux et al.’s **2013 hydrology-based terrain generation** is the main reference. It constructs a hierarchical drainage network and derives terrain using procedural primitives, providing more direct control than unconstrained erosion. [ResearchGate](https://www.researchgate.net/publication/248703095_Terrain_Generation_Using_Procedural_Models_Based_on_Hydrology)

**Advantages:** explicit outlets and tributaries; controllable basin organization; easier guarantees that a major river crosses the playable region.

**Disadvantages:** avoiding repetitive branching and smoothly embedding the network into varied mountains requires substantial procedural geometry. A plausible graph does not automatically produce plausible hillslopes.

For TCE, borrow its principle—**drainage topology is an authored procedural structure**—without necessarily implementing its entire terrain-synthesis system.

### 1.4 Grid-based hydraulic erosion

A hydraulic model stores some combination of terrain elevation, water depth, velocity or directional flux, and suspended sediment. Each step adds water, moves it between cells, erodes or deposits material, transports sediment, and removes water through evaporation or boundaries.

Mei et al.’s **2007 virtual-pipe method** is an influential GPU approach. Jákó’s **2011 hydraulic/thermal implementation** illustrates both its parallel structure and practical problems, including oscillations and unrealistically deep incision that require additional treatment. [IEEE Xplore](https://ieeexplore.ieee.org/document/4392715/)

**Fit for TCE:** useful for local channels, fans, gullies, and selected valley-floor refinement.

**Why not the whole-world foundation?** Detailed water dynamics operate on much shorter timescales than regional landscape evolution. A cheap simulation step can still require an impractical number of steps to establish mature drainage across a large map. [Aryamaan Jain](https://aryamaanjain.github.io/pdf/2024/fastflow.pdf)

A droplet-based variant can be simpler to prototype, but I would treat it as optional surface refinement rather than the authority for TCE’s river and lake topology.

### 1.5 Thermal erosion and hillslope diffusion

In terrain-generation terminology, *thermal erosion* commonly means moving loose material downhill when slopes exceed a threshold. Hillslope diffusion moves material more gradually, including below that threshold.

They serve different purposes: threshold transport moderates excessively steep loose slopes, while diffusion rounds hills and redistributes material. Jákó’s experiments demonstrate why combining slope transport with hydraulic incision produces more useful terrain than either alone. [Cescg](https://old.cescg.org/CESCG-2011/papers/TUBudapest-Jako-Balazs.pdf)

**Fit for TCE:** include both a restrained smoothing/transport process and material-dependent steep-slope treatment. Do not apply one talus limit to everything: otherwise cliffs, resistant bedrock, and gentle soil-covered hills converge toward the same shape.

For a GPU implementation, I recommend separate read/write buffers and explicit material-flux accounting. Avoid kernels in which neighboring threads independently overwrite the same height samples.

### 1.6 Multiscale erosion: the best route to 2 m detail

Schott et al.’s **2024 Terrain Amplification using Multi-scale Erosion** alternates resolution increases with fluvial erosion, thermal erosion, and deposition. It also includes elevation retargeting and drainage-correcting postprocessing.

This directly addresses TCE’s problem: preserve low-resolution landforms while adding coherent finer structure, rather than ask an 8192² simulation to discover every spatial scale simultaneously. [Axel Paris](https://aparis69.github.io/public_html/projects/schott2024_Erosion.html)

The authors’ implementation is **MIT-licensed C++ with OpenGL 4.3 compute shaders**, supports Windows and Linux, and includes example operation sequences. This is the most practical starting point I found for TCE’s high-resolution stage. [GitHub](https://github.com/H-Schott/MultiScaleErosion)

### 1.7 The relevant 2026 development: momentum-aware geomorphological transport

McDonald and Cordonnier’s **2026 Stochastic Geomorphological Transport** models transport with momentum conservation. Its demonstrated results include meanders, braided rivers, deltas, and debris fans—features that are not adequately captured by simple stream-power incision. [Erosiv](https://erosiv.studio/publications/stochastic-geomorphological-transport)

The integration caveat matters: the MIT `geotransport` repository contains the **core transport algorithm**, not the complete erosion simulator. The fuller implementation is in `soillib`, a C++23/CUDA library with an LGPL-3.0 license. [GitHub](https://github.com/erosiv/geotransport)

**Recommendation:** investigate it later for selected lowland or river-mouth regions. Do not make the initial terrain pipeline depend on porting this entire system.

---

## 2. Performance, complexity, and the actual 8192² evidence

### 2.1 First settle what “16 km at 2 m” means

These are not identical specifications:

| Specification | Consequence |
| --- | --- |
| Exactly 16,000 m with 2 m cells | 8,000 cells per side |
| Vertex heightfield covering that extent | 8,001 samples per side |
| 8,192 cells at 2 m | 16,384 m extent |
| 8,192 vertex samples at 2 m | 16,382 m between first and last samples |

I recommend defining the authoritative world in metres, using padded power-of-two working grids where convenient, and explicitly resampling or cropping to the final representation. Store the sample convention, origin, and spacing in the terrain asset.

### 2.2 Published high-resolution measurements

| Work and operation | Resolution | Reported time | Hardware and interpretation |
| --- | --- | --- | --- |
| Multiscale erosion: fluvial operator | 4096² / 8192² | **5.07 / 19.2 ms** | One operator iteration |
| Multiscale erosion: thermal operator | 4096² / 8192² | **0.90 / 3.51 ms** | One operator iteration |
| Multiscale erosion: deposition operator | 4096² / 8192² | **10.7 / 41.5 ms** | One operator iteration |
| FastFlow: flow routing | 8192² | **Approximately 0.05–0.06 s** | Read from the published scaling graph |
| FastFlow: depression routing variants | 8192² | **Approximately 0.2–0.7 s** | Graph-read estimates, varying by routing variant |

The multiscale measurements used an i7 at 4 GHz, 16 GB RAM, and a GPU identified in the paper as “GTX 3080”—apparently a naming typo for the 3080 generation. Its CPU retargeting and breaching stages took **a few seconds at 4096²**; an equivalent complete 8192² timing was not provided. [ResearchGate](https://www.researchgate.net/publication/382403597_Terrain_Amplification_using_Multi_Scale_Erosion)

FastFlow’s benchmark machine used an **RTX A6000 with 48 GB VRAM**, 20 Xeon Gold CPU cores, and 128 GB RAM. Its figures are therefore not direct predictions for a 12 GB 4070 Ti. The paper also demonstrates a 512², 32 m-cell landscape-evolution example in **0.5 seconds with implicit stepping**, versus 7.2 seconds with explicit stepping: similar geographic extent to TCE, but vastly fewer samples. [Aryamaan Jain](https://aryamaanjain.github.io/pdf/2024/fastflow.pdf)

Adding the three reported 8192² multiscale operator times gives **64.21 ms**. One hundred repetitions of each would sum to **6.42 seconds of those operator costs**. That is arithmetic, **not a complete generation benchmark**: initialization, iteration schedules, drainage correction, transfers, collision, rendering assets, and saving remain outside it.

### 2.3 CPU versus GPU

| Workload | My preferred placement | Reason |
| --- | --- | --- |
| Regional masks, outlets, geology parameters | CPU/Rust | Easy to inspect, test, and serialize |
| Coarse drainage and landscape evolution | CPU initially | Small enough to prioritize simplicity |
| Depression hierarchy and lake metadata | CPU initially | Irregular graph processing and debugging |
| Fine erosion, interpolation, slope maps | GPU | Large regular arrays and repeated stencil operations |
| Full-resolution global drainage | CPU once initially; GPU only if profiling justifies it | Avoid a complex GPU graph dependency prematurely |
| River vectors and simulation-facing metadata | CPU/Rust | Stable semantic representation |

FastFlow is a credible optimization route, but its published implementation uses PyTorch/TensorFlow with custom CUDA kernels. It is not a ready-made Rust or Unreal HLSL library, and the implemented erosion solver and routing have specific assumptions, including single-receiver flow. [Aryamaan Jain](https://aryamaanjain.github.io/pdf/2024/fastflow.pdf)

### 2.4 Memory matters as much as arithmetic

At 8192², there are **67,108,864 samples**. The following are direct storage calculations:

| One full-resolution field | Memory |
| --- | --- |
| `u8` | 64 MiB |
| `u16` / half precision | 128 MiB |
| `f32` / `u32` | 256 MiB |
| `f64` | 512 MiB |
| Four-channel 32-bit field | 1 GiB |

Ten scalar 32-bit fields consume **2.5 GiB** before duplicated buffers, temporary arrays, mipmaps, staging buffers, and Unreal’s rendering resources.

For the 12 GB GPU, I recommend a **4–6 GiB initial scratch-memory ceiling**, measured rather than assumed, and releasing generation resources before normal play. Use packed receiver directions and compact flags instead of materializing large per-cell adjacency structures.

---

## 3. Rivers and lakes: extraction must be part of generation

### 3.1 Maintain three different surfaces

TCE should distinguish:

**Physical bed elevation:** terrain used for construction, collision, and material quantities.

**Routing representation:** elevation or receiver relationships modified to obtain a consistent drainage solution.

**Water-surface elevation:** the level of water in rivers, lakes, and the ocean.

These are not interchangeable. Cordonnier, Bovy, and Braun’s **2019 depression-routing algorithm**, for example, can establish drainage connectivity without modifying the original terrain elevation. A receiver link is therefore not proof that exposed ground slopes downward along that link. [ESurf](https://esurf.copernicus.org/articles/7/549/2019/)

This distinction prevents two serious errors: converting every lake into flat dry land when filling a routing DEM, and rendering water flowing uphill because the routing graph contains a depression-crossing connection.

### 3.2 Depression filling, breaching, and preservation

**Priority-Flood** starts from permitted outlets and processes terrain in increasing spill elevation. The improved algorithm combines a priority queue with a FIFO queue inside depressions, reducing unnecessary priority-queue work. Its bounds depend on the elevation representation and queue implementation; a straightforward floating-point implementation is not simply “linear time.” [arXiv](https://arxiv.org/abs/1511.04463)

For TCE, classify depressions rather than applying one universal operation:

| Depression type | Treatment I recommend |
| --- | --- |
| Small artifact introduced by detail or resampling | Fill or breach within a small modification budget |
| Basin intended to hold a lake | Preserve its physical geometry and compute storage/outlet data |
| Large inland closed basin | Retain as a legitimate terminal drainage system |
| Spurious barrier across an established river | Repair the barrier and verify the channel profile |

Breaching lowers an escape path; filling raises a depression toward its spill level. Neither is universally correct. Restrict automatic repairs by depth, removed/added volume, and whether they cross protected ridges or lake margins.

### 3.3 Extract a depression hierarchy, not just disconnected puddles

A basin can contain smaller basins that fill, spill into neighbors, and eventually merge. **Fill–Spill–Merge** explicitly represents that hierarchy and redistributes water through it. The reference implementation includes correctness tests and avoids the assumption that all depressions should be removed. [ESurf](https://esurf.copernicus.org/articles/9/105/2021/)

For each retained lake, store:

* Basin membership, parent basin, spill saddle, and downstream outlet.
* A level–area–volume relationship.
* Catchment connections and an initial water volume.

The storage relationship can be computed directly:

\[
V(h)=\sum\_{i\in basin}\max(0,h-z\_i)\,a\_i.
\]

**A topographic basin identifies possible storage, not guaranteed permanent water.** Initialize lake levels from a water balance rather than automatically filling every basin to its rim. Dry and intermittently wet depressions are legitimate outcomes. [GitHub](https://github.com/r-barnes/Barnes2020-FillSpillMerge)

### 3.4 Flow accumulation and river extraction

For a single-receiver drainage graph,

\[
A\_i=a\_i+\sum\_{j:r(j)=i}A\_j,
\]

and, using effective runoff \(R\_i\),

\[
Q\_i=R\_i a\_i+\sum\_{j:r(j)=i}Q\_j.
\]

Use physical cell areas and consistent time units. Otherwise a resolution change silently changes river thresholds and erosion intensity.

My recommended extraction procedure is:

1. Resolve permissible drainage across slopes, flats, and basins.
2. Accumulate area and representative runoff in topological order.
3. Select channel heads using calibrated area/runoff criteria.
4. Trace downstream and merge connected segments.
5. Collapse each lake to a storage node with inlet and outlet connections.
6. Produce a river graph plus embedded channel geometry.

For ordinary channel topology, **D8** is a practical starting point; account for longer diagonal distances. Multiple-flow-direction methods are useful for distributed hillslope moisture, but need not be the same representation used for named rivers. FastFlow explicitly distinguishes these uses and currently supports single-receiver routing. [Aryamaan Jain](https://aryamaanjain.github.io/pdf/2024/fastflow.pdf)

Resolve flats deliberately. Barnes et al.’s flat-resolution algorithm constructs drainage gradients toward outlets and away from higher surrounding terrain, rather than relying on arbitrary noise. [arXiv](https://arxiv.org/abs/1511.04433)

A numerical detail worth testing: `f32` cannot represent every integer above \(2^{24}\). Since an 8192² grid contains \(2^{26}\) cells, use integer accumulation for exact cell counts, or higher precision for quantities where accumulated rounding matters.

### 3.5 River geometry needs more than a highlighted flow-accumulation raster

For TCE, generate channel centerlines, bank geometry, valley-floor width, bed elevations, and water-surface profiles as related but separate products.

Do not equate a one-cell drainage path with a physically meaningful channel width. Also distinguish the two constraints:

* The initialized water surface should not exhibit unexplained uphill jumps.
* The bed may contain pools, bars, and local steps.

For the first version, I recommend procedural cross-sections and restrained planform variation within established valley floors. Validate the resulting channel against surrounding terrain afterward.

Keep distributaries and braided channels as an explicit later extension: a strict single-receiver tree cannot represent downstream splitting. The 2026 momentum-transport work is particularly relevant to that extension. [Erosiv](https://erosiv.studio/publications/stochastic-geomorphological-transport)

### 3.6 Coasts require a separate boundary treatment

My proposed first-version coast pipeline is:

**Generate coastal regional structure → identify ocean-connected low ground → establish river mouths → construct shoreline transition bands → refine material and slope.**

Do not declare every cell below sea level to be ocean. Connectivity matters, particularly for inland depressions.

Use geological/coastal presets to distinguish resistant headlands, gently shelving sedimentary shores, drowned valleys, and depositional river mouths. These presets should control geometry and material rules—not place settlements.

This is an engineering approximation, not a wave-driven coastal-evolution model. I would defer tides, longshore sediment transport, and dynamically evolving deltas until TCE’s basic river, lake, and settlement interactions work.

---

## 4. Precedents and what to borrow

### Veloren: the closest useful architectural precedent

Veloren’s **2019 river-generation postmortem** describes explicitly computing drainage, applying implicit fluvial erosion, and combining it with uplift, thermal erosion, and hillslope diffusion. Noise remains useful for initial shapes and parameter variation rather than carrying the entire burden of river generation.

The transferable lesson is that coherent global rivers, valleys, and ridges materially change the perceived geography. The account is a valuable architectural precedent, but not an 8192² performance benchmark for TCE. [Veloren](https://veloren.net/blog/devblog-43/)

### Far Cry 5: separate landform systems and retain controls

Ubisoft’s procedural technical-art tools for **Far Cry 5**, described by Étienne Carrier in a 2018 SideFX presentation, included freshwater, sands, cliffs, biomes, and other world-building systems integrated into the game editor through Houdini Engine.

Borrow the separation into controllable systems and the ability to inspect intermediate products. Do **not** interpret this as evidence that the shipped game generated its world at player startup: the documented workflow was an artist-facing production pipeline. [SideFX](https://www.sidefx.com/community/far-cry-5/)

### Unreal Engine: terrain representation is a separate integration problem

Epic’s **UE 5.8 Landscape Technical Guide** lists **8129×8129 vertices** as a recommended large Landscape configuration, not 8192². At 2 m between vertices, that covers **16,256 m**. Exactly 16 km would require approximately **1.9685 m** spacing in that configuration. Landscape also stores height with 16-bit precision, so the chosen vertical range affects quantization. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-technical-guide-in-unreal-engine)

UE 5.8 also has an **experimental Mesh Terrain** system. Its documentation distinguishes editor preview sections from compiled sections and explicitly cautions about shipping an experimental feature. It should not be assumed to solve runtime, seed-based terrain construction automatically. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/mesh-terrain-in-unreal-engine?lang=en-US)

**The practical lesson:** a beautiful heightmap in an editor is not the same achievement as a packaged executable that generates terrain, collision, water, and saved-world data correctly.

---

## 5. Recommended TCE architecture

### 5.1 Generate a larger watershed context than the playable square

I recommend a coarse context region of roughly **32–64 km across**, with the playable 16 km square inside it.

The context need not be rendered or simulated at citizen resolution. Its purpose is to establish incoming rivers, larger drainage divides, and plausible downstream boundaries.

This avoids making every map an island or forcing all significant rivers to originate within 16 km. Preserve external catchment contributions as boundary metadata after cropping.

### 5.2 Use a staged resolution hierarchy

| Stage | Suggested representation | Responsibilities |
| --- | --- | --- |
| Regional context | Approximately 64–128 m sampling | Broad relief, drainage boundaries, upstream contributions |
| Playable-region structure | Approximately 1024² / 16 m | Main valleys, uplift/incision, retained basins |
| Intermediate terrain | 2048² / 8 m | Tributaries, slopes, sedimentary valley refinement |
| Fine terrain | 4096² then 8192² working grids | Gullies, banks, local relief, rendering detail |
| Final simulation products | River/lake graphs plus selected rasters | Water routing, buildability, soil and terrain queries |

These are starting choices, not measured optimal settings.

At each refinement stage, use the previous terrain as a constraint. Decrease added displacement near established channels, lake margins, and flat valley floors. Fine detail should not randomly dam a major river or turn usable alluvial ground into uniformly rough noise.

I would retain three kinds of outputs:

**Geometry:** height, slope, local relief, material layers.

**Hydrology:** receiver information, catchment labels, river graph, lake hierarchy, outlet connections.

**Settlement-facing properties:** soil/alluvium proxies, drainage class, groundwater-relevant geological categories, coastal exposure, and uncertainty or provenance tags.

Do not make the population simulation repeatedly derive these from the rendering mesh.

### 5.3 Build the first version around simple, replaceable components

**Coarse stage:** implement a CPU reference solver in Rust using compact raster arrays, fixed outlets, stream-power incision, restrained hillslope transport, and tested depression handling. Evaluate `fastlem` as a prototype or comparison, not as an unquestioned long-term dependency.

**Fine stage:** adapt the permissively licensed multiscale erosion operators to Unreal compute shaders. Begin with the minimum useful operator set and fixed presets.

**Hydrological finalization:** perform a complete consistency pass after the terrain is finalized. Small local repairs are acceptable; a large repair that changes a major basin should invalidate that generation stage rather than silently carving through a mountain.

**Later optimization:** port expensive global routing only after measurements show it dominates. FastFlow is an optimization reference, not a prerequisite.

### 5.4 Divide Rust and Unreal responsibilities cleanly

My proposed boundary is:

| Rust kernel | Unreal C++/rendering module |
| --- | --- |
| Seeds and generation parameters | GPU resource allocation |
| Coarse terrain and drainage logic | Compute-shader dispatch |
| River/lake semantic data | Terrain rendering and LOD |
| Validation and serialization | Collision construction |
| Simulation-facing terrain queries | Water meshes and visual materials |

Use a versioned C-compatible interface with explicit buffer lengths and ownership. Do not pass Rust container layouts across the DLL boundary.

Initially, avoid creating a separate graphics device inside the Rust DLL. Let Unreal own GPU execution, and transfer complete stage inputs or outputs rather than making per-cell calls across the interface.

For AI coding agents, assign narrow components with reference tests: one routing operation, one conservative transport kernel, one serialization format, or one CPU/GPU comparison. Avoid asking an agent to produce an entire coupled erosion system in one implementation step.

### 5.5 Separate generation performance from gameplay performance

World creation should run behind a loading interface, with progress and cancellation. It does not need to share the steady-state 16.7 ms frame budget.

My initial acceptance targets would be:

* **Normal generation:** aim for **60–120 seconds** on the target machine.
* **High-quality or difficult seeds:** allow a separate **several-minute** mode.
* **Gameplay:** no ongoing global 8192² geological simulation.

Those are **proposed product budgets, not benchmark predictions**. Measure them in a packaged build with collision construction and serialization included.

After world creation, keep terrain largely static. Handle excavation, embankments, dams, and exceptional erosion through localized updates with deliberate drainage invalidation—not by restarting regional landscape evolution.

### 5.6 Prove the packaged runtime path first

Before investing heavily in geomorphology, build this vertical slice:

> Generate a small terrain from a seed in a packaged Windows executable, walk an agent on its collision, add one river and one lake, save, reload, and reproduce the same simulation data.

Then scale the terrain backend.

This experiment should settle whether TCE uses a runtime-compatible Landscape path, a suitable existing terrain backend, or a tiled heightfield mesh implementation. Keep that decision behind an adapter so it does not dictate the hydrology model.

### 5.7 Required validation

I would make the following properties release gates:

| Test | Failure it catches |
| --- | --- |
| Every nonterminal drainage node reaches an allowed outlet or storage basin | Cycles, stranded tributaries |
| Accumulated runoff reconciles with outlet flow and storage changes | Lost or duplicated water |
| Retained lakes agree with spill elevations and storage curves | Floating lakes, impossible outlets |
| River water surfaces remain coherent after export and quantization | Uphill visual flow, stair steps |
| Major drainage survives refinement | Fine noise destroying regional structure |
| Tiled and untiled calculations agree at boundaries | Seams and artificial tile-edge outlets |
| Material accounting matches the selected model | Erosion/deposition creating unintended terrain mass |
| Worst-case flat and depression-rich maps complete within limits | Pathological queues and iteration counts |

For visual and geographic quality, compare generated slope distributions, drainage density, valley widths, and elevation distributions against a few real reference regions with similar intended geology.

Most importantly for TCE, inspect the **distribution of usable land**, not just mountain screenshots. Test whether valleys contain connected flat areas, whether river crossings are plausible, and whether coastlines offer varied access. These should arise from terrain rules rather than from secretly placing a predetermined city site.

---

## 6. Sources, code, and version applicability

### Practical implementation sources

| Source | Version/status reviewed | Language and license | Recommended role |
| --- | --- | --- | --- |
| [MultiScaleErosion](https://github.com/H-Schott/MultiScaleErosion?utm_source=chatgpt.com) | Reference code for the 2024 paper | C++/GLSL; **MIT** | Best starting point for fine-resolution erosion |
| [fastlem](https://github.com/TadaTeruki/fastlem?utm_source=chatgpt.com) | README specifies **0.1.4**; repository archived **June 5, 2025** | Rust; **MPL-2.0** | Coarse terrain prototype or maintained fork |
| [WhiteboxTools](https://github.com/jblindsay/whitebox-tools?utm_source=chatgpt.com) | Repository marked **legacy** as reviewed in 2026 | Rust; **MIT** source repository | Hydrological algorithms and independent validation |
| [Fill–Spill–Merge](https://github.com/r-barnes/Barnes2020-FillSpillMerge?utm_source=chatgpt.com) | Reference implementation for the 2021 paper | C++; **MIT** | Lake/depression hierarchy and test patterns |
| [FastFlow](https://gitlab.inria.fr/landscapes/fastflow?utm_source=chatgpt.com) | 2024 paper implementation | Python frameworks/custom CUDA; repository license not verified here | Later GPU routing optimization |
| [geotransport](https://github.com/erosiv/geotransport?utm_source=chatgpt.com) | 2026 reference implementation | C++/CUDA; **MIT** | Core stochastic transport, not complete erosion |
| [soillib](https://github.com/erosiv/soillib?utm_source=chatgpt.com) | Current repository reviewed in 2026 | C++23/CUDA/Python; **LGPL-3.0** | Experimental momentum-aware erosion evaluation |

The maintenance and license distinctions above come from the repositories themselves. In particular, **`fastlem` is archived, WhiteboxTools is legacy, and `geotransport`’s MIT license does not describe the separate full `soillib` implementation**. Pin reviewed commits and inspect dependency licenses before integration. [GitHub](https://github.com/H-Schott/MultiScaleErosion)

### Papers and documentation to read first

| Reference | Why it matters |
| --- | --- |
| [Cordonnier et al., 2016 — Large Scale Terrain Generation from Tectonic Uplift and Fluvial Erosion](https://perso.liris.cnrs.fr/apeytavi/website/publication/hal-01262376/?utm_source=chatgpt.com) | Regional terrain and drainage foundation |
| [Tzathas et al., 2024 — Physically-based analytical erosion](https://doi.org/10.1111/cgf.15033) | Faster coarse landscape evolution |
| [Schott et al., 2024 — Terrain Amplification using Multi-scale Erosion](https://h-schott.github.io/p/mserosion/?utm_source=chatgpt.com) | High-resolution amplification; project links include paper and video |
| [Jain et al., 2024 — FastFlow, full paper](https://aryamaanjain.github.io/pdf/2024/fastflow.pdf?utm_source=chatgpt.com) | GPU routing algorithms and scaling measurements |
| [Barnes et al., 2014 — Priority-Flood](https://arxiv.org/abs/1511.04463?utm_source=chatgpt.com) | Depression filling and watershed labeling |
| [Cordonnier, Bovy, and Braun, 2019 — Depression routing](https://esurf.copernicus.org/articles/7/549/2019/?utm_source=chatgpt.com) | Efficient basin-graph connectivity |
| [Barnes, Callaghan, and Wickert, 2021 — Fill–Spill–Merge](https://esurf.copernicus.org/articles/9/105/2021/?utm_source=chatgpt.com) | Preserving real depressions and storage |
| [McDonald and Cordonnier, 2026 — Stochastic Geomorphological Transport](https://erosiv.studio/publications/stochastic-geomorphological-transport?utm_source=chatgpt.com) | Meanders, deltas, and momentum-aware transport |
| [Epic — UE 5.8 Landscape Technical Guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/landscape-technical-guide-in-unreal-engine?utm_source=chatgpt.com) | Dimensions, components, and vertical precision |

**Bottom line:** TCE should invest first in **correct coarse geography, explicit water connectivity, and a reliable packaged-runtime terrain backend**. GPU multiscale erosion can then make that geography convincing at 2 m. A slightly less detailed world whose rivers, lakes, valleys, and usable ground agree is a much stronger foundation for emergent settlement than a spectacular erosion image with inconsistent drainage.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927e3-af1c-83e9-b540-0fcd122b2b8f)
