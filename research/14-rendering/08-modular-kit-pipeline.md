# Modular architectural kits: Blender → Unreal Engine 5.8

## Executive recommendation

**Build a deterministic, offline asset compiler—not a live Blender-to-game dependency.** Use Blender and Geometry Nodes to produce a bounded library of reusable architectural modules; use a versioned text manifest to define their interfaces and simulation meaning; use Unreal Python plus a small native editor plugin to import, validate, and cook them. At runtime, Rust should emit **building assembly recipes**, while Unreal places instances of already-built meshes.

For TCE, my recommended initial transport is **one render mesh per FBX, plus a JSON sidecar**, through an explicitly selected and tested importer. Keep the transport behind an adapter so that GLB/Interchange can replace it without changing the manifest or simulation.

```
Authored .blend files + Geometry Nodes + kit manifests
                         ↓
              Headless Blender asset build
                         ↓
    Reusable meshes + collision + textures + metadata
                         ↓
        Unreal editor import, validation and cook
                         ↓
      Native kit catalog + cooked rendering assets
                         ↓
 Rust building recipes → C++ adapter → streamed Nanite ISMs
```

The important distinction is that **the world’s variety comes primarily from composition, construction rules, and material variation—not from exporting a unique mesh for every building.**

**Version basis, checked September 27, 2026:** UE 5.8 was released June 23, 2026; Blender 5.2 LTS was released July 14, 2026 and is supported until July 2028. Pin exact patch versions after testing. The generated Unreal Python API references I could verify are labelled **5.7**, whereas the conceptual engine documentation is labelled **5.8**; API names below therefore require confirmation against the installed 5.8 Python stubs. This report does not claim an executed Blender/UE integration test or a benchmark on your hardware. [Unreal Engine](https://www.unrealengine.com/en-US/news/unreal-engine-5-8-is-now-available)

---

## 1. Options: export formats and automation approaches

### 1.1 Format comparison

| Path | How it works | Strengths | Limitations and TCE fit |
| --- | --- | --- | --- |
| **FBX + manifest** | Export evaluated meshes, UVs, normals, material slots, and optional UE-named collision/socket helpers. Import as static meshes. | Established Unreal static-mesh workflow; documented collision and socket conventions; straightforward per-asset reimport. | Unit, axis, naming, and importer settings must be pinned. **Best conservative starting point.** Epic’s FBX pipeline targets FBX 2020.2; that is not a guarantee that every Blender exporter configuration produces equivalent results. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/fbx-static-mesh-pipeline-in-unreal-engine) |
| **GLB/glTF 2.0 + manifest** | Export mesh/node data and standardized PBR material information; import through Unreal’s glTF/Interchange path. | Open specification; convenient single-file GLB packaging; useful interchange validation ecosystem. | TCE sockets, collision semantics, and construction rules still need your own mapping. Do not assume custom metadata automatically becomes Unreal assets. **Strong alternative to evaluate with the same golden test kit.** [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/gltf-file-format-support-in-unreal-engine) |
| **USD + manifest** | Preserve a composed scene with hierarchy, references, payloads, variants, and instancing; then import/bake required Unreal assets. | Best option when scene composition and cross-DCC collaboration are central requirements. | More concepts and dependencies than TCE needs for transferring individual kit parts. Unreal’s USD support remains labelled **Beta**. Prefer it for sophisticated scene interchange, not as the default runtime building representation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/universal-scene-description-in-unreal-engine) |
| **Send to Unreal** | Blender add-on infers asset types from an `Export` collection and applies shared settings templates. | Fast artist iteration; supports batched static-mesh exports, origins, and LOD workflows. | Useful front end, but I would not make an interactive “send” operation the authoritative build process. Pin and test the add-on with your exact Blender/UE pair. [Epic Games](https://epicgames.github.io/BlenderTools/send2ue/) |
| **Blender for Unreal Engine—BFU** | Add-on organizes batch exports, handles collision/socket authoring, checks errors, and generates Unreal Python import scripts. | Particularly relevant implementation precedent for your manifest-driven pipeline. | Adopt selectively or wrap it; its broad compatibility statement is not evidence that every feature has been tested with Blender 5.2/UE 5.8. [GitHub](https://github.com/xavier150/Blender-For-UnrealEngine-Addons) |

**Do not confuse a Geometry Nodes graph with its exported result.** Design the interchange contract around evaluated geometry and explicit metadata. A node graph, Blender material graph, or simulation cache should not be treated as an executable Unreal building generator.

### 1.2 Explicitly select the Unreal importer

“FBX import” is not a sufficiently precise configuration. Epic’s current Interchange guide still labels FBX support experimental and documents `Interchange.FeatureFlags.Import.FBX` as its toggle. Interchange also has its own pipeline stack and reimport settings. **An `FbxImportUI` configuration must not be assumed to control an Interchange import.** [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/importing-assets-using-interchange-in-unreal-engine)

For the first production version:

* Put importer selection, options, and engine version in one tested adapter.
* Use the legacy FBX path as the initial reference implementation.
* Compare Interchange FBX and GLB against that reference before switching.

That recommendation is about **predictability and maintenance**, not an assertion that FBX renders faster. Once two inputs produce equivalent cooked meshes, materials, and instance layouts, the original transport format should not be expected to determine frame rate.

---

## 2. Trade-offs: what actually controls cost

### 2.1 Choose the right unit of reuse

For TCE, the useful middle ground is generally **wall panels, floor sections, structural bays, roof sections, openings, and substantial architectural details**.

| Representation | Advantage | Main cost or failure mode |
| --- | --- | --- |
| Unique complete building meshes | Few placement records; easy to inspect one building | Unique geometry grows with the world; changes require rebuilding whole buildings; weak reuse |
| Room-sized or building-sized composite pieces | Lower instance count; useful for repeated prototypes and distant representations | Less flexible construction/damage; complex combined interiors can be problematic for Lumen |
| **Architectural modules** | Good reuse, incremental construction, clear interfaces | Requires reliable connection rules and careful material/instance management |
| Individual bricks, tiles, or boards everywhere | Maximum local variation | Excessive placement records, overlap, collision, and update work; should normally be consolidated into reusable panel variants |

The Lumen issue is concrete: Epic recommends separate walls, floors, and ceilings, and warns against importing an entire furnished room as one mesh. Modularity is therefore useful for lighting representation as well as assembly. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

A simple scale calculation illustrates the stakes: **10,000 buildings × 40 modules = 400,000 placements**; at 200 modules each, that becomes two million. These are arithmetic examples, not measured capacity limits.

### 2.2 Published measurements—and their limits

| Evidence | Published result | Correct interpretation |
| --- | --- | --- |
| Epic’s ISM documentation | Approximately **672 bytes per GPU primitive versus 64 bytes per basic instance** | Evidence for reducing individual primitives. It excludes much of the total asset, component, physics, and custom-data cost; it is not a 10× frame-rate claim. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine) |
| Nanite, *Valley of the Ancient* assets | Average **14.4 bytes per input triangle on disk** | A content-specific compression measurement, **not** a universal VRAM-per-triangle budget. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details) |
| Nanite explicit tangents | Approximately **10% additional storage** | Enable where shading tests justify it, rather than indiscriminately. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details) |
| Merrell et al., 2010 | Building generation took **a few seconds to seven minutes** in their single-threaded implementation | Historical evidence that functional layout generation can be substantial work. It does not benchmark modern Geometry Nodes or Unreal import. [Vladlen Koltun](https://vladlen.info/papers/architecture.pdf) |

I did not find a controlled, primary-source benchmark comparing the complete **Blender Geometry Nodes → FBX/GLB/USD → UE 5.8** pipeline on your hardware. Universal “FBX is X times faster” claims would therefore be unjustified.

### 2.3 Bound geometry variation; make cosmetic variation cheap

I recommend three separate variation layers:

**Geometry:** a small set of silhouette-changing variants—crooked posts, irregular masonry panels, roof profiles, damaged sections.

**Materials:** shared surface families with per-instance condition, tint, wetness, and illumination values. Unreal supports per-instance custom data specifically to avoid creating a distinct dynamic material instance for each placement. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

**Composition:** dimensions, room arrangements, bay spacing, roof organization, additions, and decoration choices.

Do not generate the Cartesian product of every width, seed, color, age, damage state, and architectural style. For example, four geometry variants with shader-controlled color and wear are a much more manageable starting point than hundreds of nearly identical exported meshes.

---

## 3. Precedents and lessons

These sources establish useful production patterns. **None demonstrates TCE’s exact Blender 5.2 → UE 5.8 → Rust-driven runtime pipeline.**

| Precedent | What it establishes | Lesson for TCE |
| --- | --- | --- |
| **Bethesda: [Fallout 4’s Modular Level Design](https://www.gdcvault.com/play/1022930/-Fallout-4-s-Modular?utm_source=chatgpt.com), GDC 2016** | Modular art kits plus an iterative level-design process allowed a relatively small content team to produce a large world. | Build and repeatedly assemble a small kit before expanding its art inventory. Treat interface failures as kit defects, not problems for every building recipe to work around. [GDC Vault](https://www.gdcvault.com/play/1022930/-Fallout-4-s-Modular) |
| **Insomniac: [The Ultimate Trim—Sunset Overdrive](https://gdcvault.com/play/1022323/The-Ultimate-Trim-Texturing-Techniques?utm_source=chatgpt.com), GDC 2015** | Standardized trim layouts, a UV-mapping script, and a variation shader addressed production speed, memory, and performance. | Standardize texture layout and automate UV placement early. Borrow the workflow, not necessarily the game’s stylized surface treatment. [GDC Vault](https://gdcvault.com/play/1022323/The-Ultimate-Trim-Texturing-Techniques) |
| **Epic: City Sample / The Matrix Awakens** | The original documented pipeline converted Houdini-generated city data into Unreal content. The **5.8 sample now includes in-engine PCG building primitives in `CitySamplePCG`**. | Study the separation between building descriptions and reusable rendering primitives. Distinguish the historical Houdini workflow from the current sample. Neither is evidence of a continuously evolving civilization simulation. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/city-sample-project-unreal-engine-demonstration?lang=en-US) |
| **[Send to Unreal](https://github.com/EpicGames/BlenderTools) and [BFU](https://github.com/xavier150/Blender-For-UnrealEngine-Addons?utm_source=chatgpt.com)** | Public implementations of standardized batch export, settings, validation, and editor automation; BFU generates Unreal Python. | Reuse implementation ideas before writing another general-purpose exporter. Keep TCE-specific semantics and build validation outside the add-on. [Epic Games](https://epicgames.github.io/BlenderTools/send2ue/) |
| **Müller et al.: [Procedural Modeling of Buildings](https://peterwonka.net/Publications/pdfs/2006.SG.Mueller.ProceduralModelingOfBuildings.final.pdf?utm_source=chatgpt.com), SIGGRAPH 2006** | CGA shape rules refine a coarse building mass into architectural detail. | Separate the construction/style grammar from the terminal mesh library. A new composition rule need not require a new mesh for every resulting building. [Peter Wonka](https://peterwonka.net/Publications/pdfs/2006.SG.Mueller.ProceduralModelingOfBuildings.final.pdf) |
| **Merrell et al.: [Computer-Generated Residential Building Layouts](https://vladlen.info/publications/computer-generated-residential-building-layouts/?utm_source=chatgpt.com), 2010** | Requirements become a room program, then optimized floor plans, then a decorated 3D building. Their reported failures include inaccessible stairs. | Generate and validate functional spaces before applying the kit. Visual plausibility does not establish traversability or structural correctness. [Vladlen Koltun](https://vladlen.info/papers/architecture.pdf) |

For TCE, the last distinction is essential: **a plausible façade is not a valid home, workshop, granary, or public building.** Your manifest needs more than socket names and mesh paths.

---

## 4. Recommended TCE implementation

### 4.1 Establish a kit contract before producing detailed art

The following values are **proposed starting conventions**, not universal studio standards or engine limits.

| Area | Proposed TCE contract |
| --- | --- |
| Authoring units | Blender metric, **1 Blender unit = 1 metre**. Validate that a one-metre calibration object imports as 100 cm. |
| Base grid | **0.25 m** structural increments; common widths of **1, 2, and 4 m**. |
| Heights | Explicit kit-family values, initially perhaps **2.5 and 3 m**. Do not assume every culture or construction system shares one storey height. |
| Pivots | A declared structural corner or junction at ground/floor level. Hinged objects use their hinge axis. |
| Export transforms | Export temporary evaluated objects with intentional local origin and normalized rotation/scale. Reject accidental negative scale and shear. |
| Bounds | Separate **structural/snap bounds**, **render bounds**, and **clearance volumes**. Eaves may exceed the structural footprint without invalidating it. |
| Connections | Typed interfaces with position, orientation, dimensions, and compatibility rules—not names alone. |
| Material slots | Prefer **one slot** on common modules; allow two where clearly justified. Stable semantic slot names are mandatory. |
| Geometry variants | Start with **2–4** useful variants for a frequently repeated module; add more only when repetition is visibly objectionable. |
| Separate objects | Keep translucent glazing, moving doors, and special effects out of the opaque structural mesh where appropriate. |

Unreal uses a left-handed, Z-up editor coordinate system, with +X forward and +Y right. Treat Blender-to-Unreal conversion as an explicit, versioned basis transformation—not a collection of remembered “rotate 90 degrees” fixes. Test a labelled axis triad, an asymmetric object, and an offset socket. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/coordinate-system-and-spaces-in-unreal-engine)

The local grid should **not** impose a global square grid on settlements. Buildings can have arbitrary site transforms, and round or curved construction should have explicit radial or curved interface families.

#### Sockets and collision

Make the manifest authoritative for connections. Optionally mirror those connections into Unreal static-mesh sockets for editor inspection.

Epic documents `UCX_[RenderMeshName]_##` collision and `SOCKET_[RenderMeshName]_##` socket helpers. It also documents two consequential FBX limitations: when importing multiple meshes with custom collision, only the first mesh’s collision is imported; socket helpers require one render mesh per FBX. That is why I recommend **one render mesh asset per file**. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/fbx-static-mesh-pipeline-in-unreal-engine)

For a doorway, author separate jamb/lintel collision hulls rather than accepting a convex hull that closes the opening. Decorative trim should usually have no independent collision.

Connections should carry such information as:

```
wall edge: height, thickness, structural family, outward direction
door opening: clear width, clear height, threshold, traversal connection
floor edge: elevation, thickness, supported span/interface
roof edge: pitch, eave/ridge/valley role, weather-enclosure relationship
```

Rust should consume these interfaces from the compiled logical catalog, without loading an Unreal mesh.

### 4.2 Make seam correctness an explicit Nanite requirement

Geometry Nodes variations must preserve connection boundaries. Randomize stone faces or timber surfaces **inside** a module’s allowed envelope; do not randomly move the vertices that define its mating plane.

Nanite quantizes vertex positions. Epic states that matching precision and compatible origin translations are necessary to avoid quantization-induced cracks between matching boundaries. Therefore, establish a **shared position-precision policy per mating family**, with placement increments compatible with that precision. Test all permitted rotations and connections rather than leaving every asset on independently chosen automatic precision. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details)

Also distinguish:

* A true gap in geometry.
* A shading seam from normals, tangents, or UVs.
* A lighting leak caused by an inadequate distance-field representation.

These need different fixes; increasing polygon count is not a general solution.

### 4.3 Standardize trims, UVs, and material variation

Use **tiling surface materials for broad areas** and **trim sheets for edges, frames, beams, mouldings, and repeated narrow details**. My initial density targets would be:

| Surface class | Starting density |
| --- | --- |
| Ordinary architectural surfaces | **256 pixels/metre** |
| Close-view focal details | **512 pixels/metre** |
| Background-only surfaces | **128 pixels/metre** |

At 256 pixels/metre, a 2048-pixel texture spans eight metres in one direction before repeating. These are authoring targets to validate at your actual camera distances, not guarantees of perceived quality.

Define UV responsibilities explicitly: UV0 for surface/trim mapping; additional channels only for a documented mask, mapping requirement, or baked-lighting need. For TCE’s dynamic-lighting path, do not generate a unique lightmap UV channel automatically. Epic specifically recommends disabling lightmap-UV generation when precomputed lighting is not used, avoiding unnecessary build time and mesh data. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

Keep master materials hand-maintained and relatively small. Let automation create **material instances and bindings**, not regenerate a giant shader graph for every module.

A proposed per-instance data layout might contain four to eight floats for condition, wetness, palette variation, emission, and deterministic variation. With eight 32-bit floats, 500,000 placements require **16 MB of raw custom-data payload**, before allocation and engine overhead. Update slowly changing condition values on change, not every frame.

Include texture color-space, channel packing, normal-map convention, and UV expectations in the material manifest. An asymmetric normal-map test is more reliable than assuming an exporter/importer pair will handle every convention automatically.

### 4.4 Treat Geometry Nodes as a controlled module generator

Use small, versioned node groups with explicit inputs: dimensions, construction parameters, surface irregularity, damage state, and seed.

A robust build sequence is:

1. Load the source file and resolve declared dependencies.
2. Assign one approved parameter/seed combination.
3. Evaluate the dependency graph.
4. Obtain the evaluated mesh, preserving required data layers.
5. Validate geometry, UVs, materials, and interfaces.
6. Export and release temporary data before processing the next variant.

Blender’s dependency-graph API distinguishes original data from evaluated data containing modifier results. Reading the original `object.data` is not equivalent to exporting what the viewport shows. The documented evaluated-object pattern is therefore the appropriate basis for a custom exporter. [Blender Documentation](https://docs.blender.org/api/4.2/bpy.types.Depsgraph.html)

**Realize instances only where necessary.** For a single exported wall panel, realizing its internal boards or stones can be appropriate. Realizing an entire generated settlement destroys the reuse you are trying to preserve and increases geometry memory; Blender’s Realize Instances documentation explicitly warns about the cost of large instance counts. [Blender Documentation](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/instances/realize_instances.html)

Use explicit seeds and stable element identifiers. Blender’s Random Value node accepts an ID to drive its random result; your generator should deliberately control this rather than accidentally relying on changing element order. Record the node-group revision alongside the seed. [Blender Documentation](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/utilities/random_value.html)

A reproducible build key should include:

```
Source assets and dependencies
+ Geometry Nodes version and parameter values
+ seeds
+ Blender/exporter versions and settings
+ texture inputs
+ Unreal importer/build settings and engine version
```

Do not make byte-identical FBX files the only reproducibility test. Compare canonical geometry, interface metadata, material bindings, and resulting asset properties as well.

### 4.5 Use a typed manifest and two generated catalogs

I recommend author-friendly YAML or TOML, normalized into canonical JSON for the build. The manifest is a **TCE schema**, not an Unreal import schema.

A shortened example:

```
schema: tce.kit/1
kit_id: wattle_frame_a
source_units: metres
source_frame: blender_zup_right_handed
snap_m: 0.25

modules:
  - id: wall.wattle.door.2m
    source:
      blend: kits/wattle_frame_a.blend
      object: WallDoorGenerator
      node_group: GN_WattleWall_v3

    parameters:
      width_m: 2.0
      height_m: 2.5
      thickness_m: 0.25
    variant_seeds: [101, 307, 911]

    structural_bounds_m:
      min: [0.0, 0.0, 0.0]
      max: [2.0, 0.25, 2.5]

    interfaces: wall_door_2m_v1
    clearances: pedestrian_door_v1
    construction_rule: wattle_on_timber_frame

    render:
      nanite_profile: architecture_seamed_v1
      material_bindings:
        surface: wattle_plaster_a
      collision: authored_ucx
      custom_data_layout: architecture_v1
```

The build should expand this into concrete variant records, resolved interface transforms, checksums, and output paths.

Generate **two views of the same definition**:

**Logical catalog for Rust:** persistent IDs, dimensions, connections, traversal, construction requirements, and damage-state meaning.

**Rendering catalog for Unreal:** soft mesh/material references, rendering profiles, custom-data layout, collision policy, and visual variants.

Do not duplicate hand-authored truth between them. A material or technology becoming available should unlock construction capabilities—not require a hard-coded historical-era enum.

Persist stable module and variant IDs in saves. Asset paths, array positions, and ISM instance indices should not become permanent simulation identities.

### 4.6 Implement Unreal automation as an editor build stage

Unreal’s Python environment is **editor-only**, not available in a cooked executable. Epic documents headless invocation through `UnrealEditor-Cmd.exe` and the `pythonscript` commandlet. Enable Python Editor Script Plugin and the required editor-scripting facilities in the authoring project. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/scripting-the-unreal-editor-using-python)

A typical invocation shape is:

```
UnrealEditor-Cmd.exe "C:\TCE\TCE.uproject" ^
  -unattended -nop4 ^
  -run=pythonscript ^
  -script="C:\TCE\Tools\import_kit.py"
```

Your script should execute the following phases.

#### A. Validate before changing Content

Validate the manifest schema, IDs, paths, dependencies, approved parameter ranges, and expected exporter versions. Produce a dry-run plan.

Import into a clean worktree or controlled staging environment. Do not let a failed batch silently become the new published catalog.

#### B. Import meshes with explicit settings

For the legacy path, use `AssetImportTask` with an FBX options object and a fixed settings profile. Relevant documented properties include `build_nanite`, `auto_generate_collision`, `combine_meshes`, scene/unit conversion, normals, vertex colors, and lightmap-UV generation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/FbxStaticMeshImportData?application_version=5.7)

My initial profile would be:

| Setting | Initial policy |
| --- | --- |
| Mesh combination | Off: one asset per render mesh |
| Automatic materials/textures | Off; bind controlled TCE materials separately |
| Automatic collision | Off when authored collision is supplied |
| Nanite | On for eligible structural meshes |
| Lightmap UV generation | Off for the dynamic-lighting path |
| Normals/tangents | Explicit, tested policy; never an accidental importer default |
| Vertex colors | Import when used by the material contract |
| Transform conversion | One tested basis/unit conversion; validate resulting bounds |

For Interchange, implement equivalent settings in its pipeline rather than reusing legacy options blindly. `AssetImportTask.destination_name` is explicitly documented as ignored by Interchange; naming must be controlled through the appropriate pipeline. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/AssetImportTask?application_version=5.7)

#### C. Normalize and validate Nanite settings

The documented editor API exposes `StaticMeshEditorSubsystem.get_nanite_settings()` and `set_nanite_settings()`. This gives you a place to enforce the profile after import rather than relying exclusively on dialog defaults. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/StaticMeshEditorSubsystem?application_version=5.7)

For ordinary architecture, start with no intentional source-detail trimming, a tested fallback policy, and **Preserve Area disabled**. Enable explicit tangents only where trim/normal-map tests require them. Nanite’s Preserve Area setting is intended for foliage-like simplification behaviour, not as a general architectural quality switch. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details)

Keep translucent panes separate from the structural Nanite asset: Epic documents Nanite material support for opaque and masked blend modes. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)

For software Lumen, inspect distance fields. Epic recommends walls at least 10 cm thick to avoid leakage; that is a rendering guideline, not a historical construction rule. Thin wattle, cloth, or other lightweight structures need deliberate lighting tests rather than arbitrary changes to simulation dimensions. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine)

#### D. Create material instances and bind by semantic slot

Use controlled master materials. Unreal’s `MaterialEditingLibrary` exposes material-instance parent and parameter setters; use these to populate texture, scalar, and vector parameters from the material manifest. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/MaterialEditingLibrary?application_version=5.7)

Resolve bindings by stable slot name, then verify the final slot count and assignment. Reimport must detect removed or renamed slots instead of leaving stale materials behind.

#### E. Generate native data assets

Create a small compiled class such as `UTCEKitDefinition : UPrimaryDataAsset`, with native structs for module records. Python should populate **instances of this class**.

`UPrimaryDataAsset` provides primary asset identification and asset-bundle support; the documented `DataAssetFactory` exposes the class used for creation. Keep runtime data classes in a runtime module and asset-building helpers in an editor module. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/data-assets-in-unreal-engine)

Use soft references and explicit Asset Manager loading/cooking rules. Verify that every required mesh, material, and texture is present in a cooked build; successful editor loading is not sufficient evidence. Asset Manager supports primary-asset rules and bundle-based management for this purpose. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine)

#### F. Await completion, save, and test the cooked result

Do not publish a catalog while imports are still pending. The documented `AssetImportTask.get_objects()` can wait for asynchronous imports to finish. Also account for any outstanding asset compilation before validating and cooking. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/AssetImportTask?application_version=5.7)

Use a small native helper where Python lacks a reliable operation—socket replacement, detailed mesh validation, or build-completion handling—rather than assembling fragile UI automation.

For AI coding agents, this is the productive boundary: schemas, exporters, import adapters, validation, and tests should be code-reviewed. Agents should not infer asset correctness from a successful import message or a plausible viewport screenshot.

### 4.7 Keep runtime assembly separate from authoring

For Nanite-only structural groups, start with **ISM**, not HISM by habit. Epic specifically recommends ISM for Nanite because Nanite has its own culling and LOD system; HISM remains useful for other workloads, especially largely static non-Nanite groups. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine)

My proposed grouping key is:

```
streaming cell
+ static mesh
+ material binding set
+ collision/shadow policy
```

Rust should emit versioned placement/change buffers containing module IDs, transforms, and semantic state. A C++ adapter resolves assets and performs bounded game-thread updates. Avoid one actor per wall or one Unreal call per simulated person.

Across the Rust DLL boundary, use a defined ABI, explicit ownership, and a deliberate panic/exception policy. Rust’s FFI documentation warns that unwinding behaviour depends on the ABI boundary; do not let this be an accidental integration detail. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Stream render representations independently of the logical world. A distant settlement need not keep every detailed module resident, and an unfinished visual update must not force the simulation to wait for asset loading.

### 4.8 Acceptance tests and budgets

**Build the validation suite before expanding beyond the first kit.**

| Gate | Required checks |
| --- | --- |
| Coordinate correctness | One-metre cube; labelled axes; asymmetric geometry; offset/rotated sockets |
| Assembly correctness | Every permitted connection/rotation; no unintended gaps, intersections, or blocked openings |
| GN/export correctness | Seed repeatability; evaluated bounds; UV channels; vertex attributes; material slots; collision |
| Reimport correctness | Import twice unchanged; modify topology; remove a socket; rename a slot; verify no stale data |
| Rendering correctness | Grazing light, wet surfaces, close camera, distant camera, Nanite fallback, Lumen visualizations |
| Packaging correctness | Cook and load every kit through the runtime catalog without editor dependencies |
| Simulation correctness | Traversable doors/stairs; enclosure and support semantics; damage transitions remain coherent |

On your hardware, I would use these **initial engineering targets**, not promised performance:

| Item | Starting target |
| --- | --- |
| Output | 2560×1440 at 60 fps; record internal resolution/upscaling separately |
| Frame budget | 16.67 ms, with CPU and GPU timings measured separately |
| GPU working headroom | Aim initially for roughly **9–10 GB** steady-state usage on the 12 GB card |
| Building-update work | Start with a **≤1 ms game-thread allocation** for ordinary update frames; queue larger bursts |
| Offline concurrency | One Unreal import process and initially one or two Blender workers; increase only after measuring peak RAM |

Benchmark **100,000, 500,000, and one million module placements**, varying geometry-variant count, material-slot count, collision, and update bursts. Include street-level and aerial views, and repeat with the full Rust simulation running.

Record import/evaluation time, peak RAM, cold/warm build behaviour, cooked size, GPU memory, frame-time percentiles, and worst hitches. Unreal Insights provides timing, asset-loading, and memory-analysis facilities for this work. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

The first milestone should be one complete, modest kit that survives this entire loop—not a large mesh library. Only then expand materials, construction systems, and architectural grammars.

---

## 5. Sources and version map

The studio talks, papers, and public code are linked in the precedent table above. These are the principal implementation references:

| Topic | Documentation | Version applicability |
| --- | --- | --- |
| Release baseline | [UE 5.8 release](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available), [Blender 5.2 LTS](https://www.blender.org/releases/5-2/?utm_source=chatgpt.com) | 2026 releases; pin tested patches |
| Geometry Nodes | [Random Value](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/utilities/random_value.html?utm_source=chatgpt.com), [Realize Instances](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/instances/realize_instances.html?utm_source=chatgpt.com) | Current manual labelled Blender 5.2 LTS |
| Evaluated geometry API | [Dependency graph examples](https://docs.blender.org/api/4.2/bpy.types.Depsgraph.html?utm_source=chatgpt.com) | Verified 4.2 API reference; validate the adapter in 5.2 |
| Mesh transport | [FBX static meshes](https://dev.epicgames.com/documentation/en-us/unreal-engine/fbx-static-mesh-pipeline-in-unreal-engine?utm_source=chatgpt.com), [Interchange](https://dev.epicgames.com/documentation/en-us/unreal-engine/importing-assets-using-interchange-in-unreal-engine?utm_source=chatgpt.com) | Current UE 5.8 documentation |
| Alternative formats | [glTF support](https://dev.epicgames.com/documentation/en-us/unreal-engine/gltf-file-format-support-in-unreal-engine?utm_source=chatgpt.com), [USD support](https://dev.epicgames.com/documentation/en-us/unreal-engine/universal-scene-description-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8; USD labelled Beta |
| Editor automation | [Python scripting](https://dev.epicgames.com/documentation/en-us/unreal-engine/scripting-the-unreal-editor-using-python?utm_source=chatgpt.com), [AssetImportTask](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/AssetImportTask?application_version=5.7&utm_source=chatgpt.com), [FBX import settings](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/FbxStaticMeshImportData?application_version=5.7&utm_source=chatgpt.com) | Guide: 5.8; verified generated API references: 5.7 |
| Mesh/material automation | [StaticMeshEditorSubsystem](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/StaticMeshEditorSubsystem?application_version=5.7&utm_source=chatgpt.com), [MaterialEditingLibrary](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/MaterialEditingLibrary?application_version=5.7&utm_source=chatgpt.com) | Verified generated API references: 5.7 |
| Runtime catalogs | [Data Assets](https://dev.epicgames.com/documentation/en-us/unreal-engine/data-assets-in-unreal-engine?utm_source=chatgpt.com), [Asset Manager](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 |
| Rendering constraints | [Nanite technical details](https://dev.epicgames.com/documentation/unreal-engine/nanite-technical-details?utm_source=chatgpt.com), [Lumen technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine?utm_source=chatgpt.com), [ISM guidance](https://dev.epicgames.com/documentation/en-us/unreal-engine/instanced-static-mesh-component-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8; published sample measurements are not TCE benchmarks |
| Profiling and DLL safety | [Unreal Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine?utm_source=chatgpt.com), [Rust FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Current documentation |

**Bottom line:** invest first in **stable module interfaces, shared materials, deterministic finite variants, and cooked-build validation**. For TCE’s solo-development constraints, a small, auditable FBX/manifest compiler with an interchangeable importer is a better starting investment than a sophisticated live-DCC or USD runtime pipeline.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92976-37ec-83ea-a5bd-d482a48f31d4)
