# Engineering report: data-driven content and mod-style authoring for TCE

**Recommendation:** Store TCE’s authored primitives in **TOML**, deserialize them into dedicated Rust authoring types, and compile the complete content set into an **immutable, strongly typed registry**. Treat cross-references, semantic validation, pack composition, fingerprints, and hot reload as compiler responsibilities—not features supplied by the text format.

Use **RON instead** when deeply nested, enum-heavy rules become the dominant authoring workload. Do not initially support multiple equivalent formats, unrestricted executable scripts, or a custom language.

The most important boundary is between **definitions of what can exist** and **the state of what has actually happened**. Changing a recipe definition is straightforward; safely changing a recipe while thousands of people are already producing it is a state-migration problem.

*Research checked against sources available on September 27, 2026. Version-specific observations and published measurements are identified below; the proposed TCE architecture and performance targets are recommendations, not measured TCE results.*

---

## 1. Options: the main techniques and how they work

### 1.1 Schema-backed records, compiled before use

The simplest model is a collection of records: technology, good, recipe, office, policy, architectural component, and so forth.

For TCE, I recommend this pipeline:

```
Text files + pack manifests
          ↓
Parse with source locations
          ↓
Validate individual records
          ↓
Resolve dependencies and explicit overrides
          ↓
Link typed cross-file references
          ↓
Validate the effective content graph
          ↓
Compile immutable runtime tables
          ↓
Publish at a simulation-safe boundary
```

The distinction between **record validation** and **graph validation** matters. A schema can establish that `unlocks` is an array of strings. It cannot, by itself, establish that each string identifies an existing recipe, that the recipe belongs to an installed pack, or that its prerequisites are reachable. JSON Schema’s $ref references *schemas*, not arbitrary objects in a game’s content database. [JSON Schema](https://json-schema.org/understanding-json-schema/structuring)

This is the best default for TCE: most primitives should be records whose interpretation is implemented and tested in Rust.

### 1.2 A restricted declarative rule language

Some primitives need more than scalar properties: eligibility conditions for office, policy effects, technological discovery conditions, or architectural compatibility rules.

Represent these as **bounded, typed expressions**, such as:

```
All(
    HasCapability("ceramics"),
    SettlementPopulationAtLeast(200),
    HasResource("clay")
)
```

This is a proposed semantic structure, not a recommendation to invent that exact syntax. It could be represented as tagged TOML tables or native RON enums.

Compile expressions into typed operations with explicit execution scope: person, household, settlement, institution, or world. Preserve enough structure to explain decisions: “This office requires citizenship and a minimum reputation of 30.”

For TCE, avoid arbitrary evaluation of strings, implicit scope changes, unrestricted loops, and reflective access to every kernel field. A smaller language is easier to validate, explain, migrate, and optimize.

### 1.3 Executable generators or runtime scripts

There are two substantially different uses of scripting:

| Technique | What executes | Appropriate use |
| --- | --- | --- |
| **Content generator** | Code runs during content construction and emits ordinary definitions. | Generating families of materials, recipes, or architectural variants. |
| **Runtime script** | Code executes while the world is running and reads or changes state. | Mechanics that cannot be expressed through existing kernel operations. |

Factorio explicitly separates prototype construction from runtime scripting; its prototype-loading Lua state is discarded after startup. That separation is a useful precedent even without adopting Lua. [lua-api.factorio.com](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)

For TCE, let AI agents generate ordinary source records first. Add a deterministic, resource-limited generator stage only when repetition becomes burdensome. Defer runtime scripting until there is a concrete mechanic that warrants its debugging, security, performance, and migration costs.

**Data-driven should mean that authored data composes implemented mechanics—not that every mechanic is dynamically programmable.**

---

## 2. Trade-offs: formats, tooling, performance, and complexity

### 2.1 RON versus TOML versus YAML versus KDL

| Format | Rust integration and strengths | Main drawbacks | TCE assessment |
| --- | --- | --- | --- |
| **TOML** | Direct Serde support; explicit keys and tables; comments; good fit for manifests and relatively shallow records. | Deep expression trees and arrays of nested records become verbose. No native `null` value. | **Best starting point** for the overall authoring system. |
| **RON** | Designed around Serde’s data model; natural structs, enums, tuples, options, comments, and trailing commas. | Rust-oriented representation; documented edge cases around tagged/untagged enums, `flatten`, and type-erased deserialization. | **Best alternative** for complex typed rule trees. |
| **YAML** | Concise hierarchical documents; Serde implementations exist; supports aliases and richer document features. | More syntax and scalar interpretation rules to constrain; indentation-sensitive edits; library selection needs care. | Viable, but brings little that TCE needs enough to justify the additional authoring surface. |
| **KDL** | Readable node/attribute/child syntax; attractive for hierarchical declarations; format-preserving Rust parser. | Its node model does not map as directly to ordinary Serde structures. Typed decoding often uses a separate derive system. | Attractive language design, but more integration work for a solo developer. |

These characteristics are documented in the current TOML, RON, YAML implementation, and KDL sources. [Docs.rs](https://docs.rs/toml/latest/toml/)

#### Why TOML wins narrowly for TCE

TCE’s examples—goods, offices, policies, technologies, style elements—are predominantly named records with fields and references. TOML keeps those records explicit, while existing tools can provide schema-backed editing. Tombi currently supplies a formatter, linter, language server, and JSON Schema integration. [Tombi Toml](https://tombi-toml.github.io/tombi)

My preference is therefore **TOML plus a good compiler**, rather than a more expressive format plus weaker validation.

Use a documented authoring subset: quoted string identifiers, explicit units, limited nesting, and a standard representation for tagged expressions. Pin the parser and editor versions; test that both accept the same fixtures.

#### When RON would be better

RON becomes more compelling when authors spend most of their time composing expressions such as nested conditions and effects. It can express Rust-like enum trees without repeated `kind` fields and nested table headers.

However, “supports Serde” does not mean every Serde representation works identically across formats. RON 0.12.2 documents restrictions involving internally tagged, adjacently tagged, and untagged enums, `flatten`, and `deserialize_any`. It also has editor integrations and an LSP, so the issue is not an absence of tooling. [Docs.rs](https://docs.rs/ron/latest/ron/)

If choosing RON, design and test a RON-friendly authoring model rather than reusing JSON-oriented structures indiscriminately.

#### YAML’s current Rust caveat

The widely referenced `serde_yaml` **0.9.34+deprecated** is explicitly unmaintained. That is a statement about that crate, not YAML itself. [Docs.rs](https://docs.rs/serde_yaml/latest/serde_yaml/)

Alternatives exist. For example, `serde-saphyr` **1.3.0** offers typed deserialization, configurable resource budgets, source diagnostics, and optional inclusion features. For TCE, disable unnecessary extensions, constrain aliases and nesting, and reject unknown tags rather than exposing the entire YAML feature set. [Docs.rs](https://docs.rs/serde-saphyr/latest/serde_saphyr/)

#### KDL’s integration caveat

The current KDL specification is **2.0.0**. The `kdl` crate provides document-oriented parsing, while `knuffel` illustrates strongly typed decoding through its own `Decode` derive and annotations for arguments, properties, and children—not ordinary Serde derives. Pin compatible specification and implementation versions rather than assuming every KDL ecosystem crate supports the same dialect. [KDL](https://kdl.dev/)

### 2.2 What the available benchmarks establish

A useful published benchmark is TOML maintainer Ed Page’s July 2025 account of the `toml` 0.9 rewrite. For the linked `Cargo.web-sys.toml` fixture, the reported baseline results include:

| Implementation and destination | Reported parse time |
| --- | --- |
| `toml` 0.5 → `Table` | 939 µs |
| `toml` 0.9.0 → owned `DeTable` | 501 µs |
| `toml` 0.9.0 → borrowing `DeTable` | 398 µs |
| `serde_json` 1.0.140 → `Value` | 259 µs |

The comparison also varies ordering and hashing settings. These are **maintainer measurements for that fixture and those versions**, not a matched RON/TOML/YAML/KDL benchmark or a prediction for TCE’s PC. [Epage](https://epage.github.io/blog/2025/07/toml-09/)

The Jomini Rust parser for Paradox data claims throughput exceeding **1 GB/s**, but that is a parser-level claim, not the cost of loading, linking, validating, and migrating a complete game content set. [Docs.rs](https://docs.rs/jomini/latest/jomini/)

**I did not find a credible, directly comparable four-format benchmark using the same TCE-like records, destination types, diagnostics, and reference-validation workload.** A format ranking based on unrelated microbenchmarks would be misleading.

### 2.3 Where TCE should spend its performance budget

With the recommended design, source format has **no inherent steady-state simulation cost**: all formats compile into the same runtime representation.

Measure loading separately from publication:

```
directory/archive access → parsing → linking → linting
→ runtime compilation → migration preparation → publication
```

Benchmark representative packs containing perhaps 1,000 and 10,000 definitions, including nested rules, localization references, overrides, and intentionally invalid inputs. Measure cold and warm loads, edit-to-diagnostic latency, peak memory, and renderer frame-time impact.

Reasonable **initial engineering targets, not measured guarantees**, are sub-second feedback for ordinary edits and a sub-millisecond publication step *when no world migration is required*. Expensive migrations should require a pause rather than being hidden inside that target.

At 60 fps, a rendered frame is approximately 16.67 ms. Parsing content on the game thread is therefore the wrong architectural choice regardless of whether one parser is twice as fast as another.

---

## 3. Precedents: what existing systems demonstrate

### 3.1 Paradox script: expressive domain language, substantial semantic tooling

Paradox content formats demonstrate the attraction of compact, domain-specific declarations. They also illustrate why syntax validation is insufficient. The Jomini parser documents dialect variation and structures that are not simply interchangeable with ordinary configuration maps. [Docs.rs](https://docs.rs/jomini/latest/jomini/)

CWTools provides the more important engineering precedent: validation and editor assistance for scopes, triggers, effects, localization, references, and graphical assets, along with navigation and completion. Its support varies by game; its own documentation distinguishes mature support from partial or in-progress support. [GitHub](https://github.com/cwtools/cwtools-vscode)

**Strength:** A vocabulary tailored to the game can make large amounts of content concise.

**Pain point:** Once meaning depends on context, scope, load order, and game-specific rules, maintaining the language service becomes a significant product of its own.

**TCE lesson:** Borrow typed conditions, reference navigation, and domain diagnostics. Do not copy implicit scope semantics or start by building a general Paradox-like language.

### 3.2 RimWorld XML Defs: inheritance, patching, and deferred reference resolution

The inspected RimWorld code snapshot shows a staged process: collect XML from active mods, combine documents, apply patches, parse definitions, and resolve references. Separate code handles XML inheritance and deferred references identified by definition type and name. [GitHub](https://raw.githubusercontent.com/Chillu1/RimWorldDecompiled/master/Verse/LoadedModManager.cs)

**Strength:** Definitions can reference one another independently of immediate parse order, while inheritance reduces repetition.

**Pain point:** Understanding a final value can require tracing its original file, parent definition, patches, and mod order. That is a strong argument for preserving provenance and exposing “why does this field have this value?”

The inspected loader also contains an explicit `hotReload` path. It would be inaccurate to characterize RimWorld as having no definition-reload mechanism; the existence of that path does not establish that arbitrary changes are safe for all active world state. [GitHub](https://raw.githubusercontent.com/Chillu1/RimWorldDecompiled/master/Verse/LoadedModManager.cs)

**Source caveat:** These observations come from a publicly indexed **decompiled code snapshot**, not an official Ludeon SDK or a verified current-patch source release. It is evidence about implementation structure, not code to copy into TCE. [GitHub](https://github.com/Chillu1/RimWorldDecompiled)

### 3.3 Dwarf Fortress raws: a large vocabulary of composable building blocks

Dwarf Fortress’s official 50.01+ modding guide describes packs containing metadata, object definitions, and graphics. Vanilla content uses the same packaging model. It also explains ordering considerations and that mods are no longer embedded in saves, making the required mod installation part of world portability. [Bay 12 Games](https://bay12games.com/dwarves/modding_guide.html)

Its development history is particularly relevant: version **52.01** exposed Lua for procedurally generated objects, and **52.03** improved reporting for missing permitted reactions/buildings and duplicate objects. These are concrete examples of extending authoring power while continuing to improve cross-reference diagnostics. [Bay 12 Games](https://www.bay12games.com/dwarves/)

**Strength:** A sufficiently rich vocabulary of building blocks can support enormous variation without individually scripting every resulting entity.

**Pain point:** Vocabulary size and interactions demand documentation and semantic checks; compact tokens alone do not make content understandable.

**TCE lesson:** Make the base game a normal content pack, and make validation of relationships as important as validation of individual records.

### 3.4 Factorio prototypes: the clearest lifecycle separation

Factorio **2.1.20** documents ordered prototype construction through `data.lua`, `data-updates.lua`, and `data-final-fixes.lua`. Prototypes are constructed after these stages and are not mutable through ordinary runtime scripting. Mod dependencies participate in loading order. [Factorio Lua API](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)

It separately provides JSON migrations for prototype substitutions and Lua migrations for loaded world state. Saves track which migrations have already run. [Factorio Lua API](https://lua-api.factorio.com/latest/auxiliary/migrations.html)

**Strength:** Content construction, runtime behavior, and saved-state evolution have distinct responsibilities.

**Pain point:** Multi-stage mutation and ordering can make the final definition harder to attribute. TCE should prefer explicit overrides with provenance over unrestricted shared-table mutation.

One behavior not to copy: Factorio’s lifecycle documentation says extraneous prototype properties are ignored. TCE’s AI-authored source should reject unknown fields so that a misspelling cannot silently become a no-op. [lua-api.factorio.com](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)

### 3.5 Relevant engine and build-system precedents

Bevy’s asset system demonstrates asynchronous loading, typed handles, dependency tracking, and file-watcher-driven reload. These are useful patterns, but updating an asset does not automatically perform a consistent migration of simulation state that was created from it. [Docs.rs](https://docs.rs/bevy_asset/latest/bevy_asset/)

For the compiler itself, *Build Systems à la Carte* supplies a useful framework for thinking about dependencies, invalidation, and rebuilding; its authors also publish executable code and related talks. Salsa offers a Rust incremental-query implementation. Neither is necessary for TCE’s first implementation: full relinking of a moderate content set is simpler to verify. [Microsoft](https://www.microsoft.com/en-us/research/publication/build-systems-la-carte/)

---

## 4. Recommended architecture for TCE

The remainder is a proposed design tailored to TCE’s constraints.

### 4.1 Separate authoring types, runtime definitions, and world state

Use three distinct representations.

**Authoring types** contain strings, optional fields, explicit units, editor-friendly structures, and source metadata.

**Compiled definitions** contain validated values, typed numeric references, precomputed indexes, and executable condition/effect representations.

**World state** contains actual people, inventories, institutions, technologies known by particular societies, enacted laws, construction projects, and historical outcomes.

Do not serialize runtime ECS components directly as the public authoring format. That would make an internal storage refactor a content-format change.

A proposed source layout:

```
content/
  core/
    pack.toml
    technologies/pottery.toml
    goods/clay.toml
    recipes/fire_pottery.toml
    buildings/pottery_kiln.toml
    offices/
    policies/
    architecture/
    localization/
```

An illustrative technology record:

```
kind = "technology"
id = "core:technology/pottery"
name_key = "technology.pottery.name"

tags = ["ceramics", "craft"]
discovery_effort_person_hours = 120.0

prerequisites_all = [
  "core:technology/controlled_fire",
]

unlocks = [
  "core:recipe/fire_pottery",
  "core:building/pottery_kiln",
]
```

The fields and values are illustrative, not balance recommendations. A pack manifest should declare its content-schema version and compatible kernel content API.

Use one file per substantial definition initially. Allow grouped files later where they genuinely improve authoring; runtime efficiency should not dictate source-file granularity.

### 4.2 Establish stable public identity and typed runtime references

Use stable identifiers such as:

```
core:technology/pottery
core:good/clay
regional_architecture:style/stone_courtyard
```

An identifier is not a filename, localized name, array offset, or hash.

After parsing all files, build a symbol table and resolve references into distinct types such as `TechnologyId`, `GoodId`, and `OfficeId`. References to the wrong kind should fail even when the string exists.

For simulation execution, use compact per-kind indices into immutable tables. Avoid repeated string hashing for every person’s decisions.

**Important hot-reload trap:** Re-sorting definitions and assigning fresh dense indices can silently redirect existing world references. Either preserve session slots—append new entries and retain tombstones—or explicitly remap every affected reference during a coordinated transition.

Persist stable IDs, or a save-local dictionary mapping compact saved values to stable IDs. Do not treat process-local indices as permanent identity.

### 4.3 Make the compiler authoritative; schemas improve the editing experience

Define dedicated Serde authoring structures and generate editor schemas with Schemars. Apply `deny_unknown_fields` deliberately to nested structures, and use explicit defaults only when omission has a clear meaning. Serde documents the relevant attributes; Schemars derives schemas from Rust types and their serialization representation. [Serde](https://serde.rs/container-attrs.html)

There is an important qualification: Schemars generates **JSON Schema**, not a complete specification of every TOML or RON construct. Keep the authoring model within a tested mapping and maintain fixtures that exercise both compiler and editor validation. Pin the schema draft and dependencies; Schemars notes that generated schema details can change without constituting a breaking library change. [Docs.rs](https://docs.rs/schemars/latest/schemars/)

Preserve a source map containing file, field path, span, owning pack, and override history. Ordinary deserialization does not automatically preserve all that provenance. TOML’s spanned values and `toml_edit` can help; the latter preserves formatting with documented limitations rather than guaranteeing byte-identical round trips. [Docs.rs](https://docs.rs/toml/latest/toml/)

Diagnostics should look like this:

```
E2104 technologies/pottery.toml:12
Unknown recipe: core:recipe/fire_potery
Referenced by: core:technology/pottery → unlocks[0]
Possible match: core:recipe/fire_pottery
```

For AI authors, stable error codes and structured JSON diagnostics are at least as valuable as a graphical editor.

### 4.4 Validate in layers, including semantic content linting

| Layer | Checks |
| --- | --- |
| **Syntax and shape** | Parse errors, unknown fields, missing required fields, invalid variants. |
| **Local constraints** | Finite numbers, valid units, nonnegative quantities where required, bounded probabilities, valid identifiers. |
| **Pack and reference integrity** | Duplicate IDs, undeclared dependencies, missing targets, incorrect target types, override conflicts. |
| **Semantic graph checks** | Impossible prerequisite chains, scope mismatches, missing unlocks, inaccessible production chains, incompatible architectural components. |
| **Executable content checks** | Small scenarios exercising production, discovery, institutions, migration, and reload behavior. |

For the requested “every technology lists what it unlocks” rule, make `technology.unlocks` the authoritative source for **simple direct unlock relationships**. Generate reverse indexes, technology-tree views, and `unlocked_by` displays.

Do not also require authors to maintain independent reverse lists in every recipe; those copies will drift.

For compound gates—“requires both metallurgy and water power”—use one authoritative typed gate expression and derive each technology’s associated unlock listing. Be explicit about whether multiple technologies are alternatives or joint requirements.

Require each technology to have an unlock, another meaningful effect, or a reason-coded exception. Distinguish “enables producing an item” from “creates the resource” and “has been discovered” from “has been adopted by this settlement.”

Useful TCE-specific checks include:

* **Economic feasibility:** Required goods have some permitted gathering, production, or import path; flag unintended bootstrap deadlocks.
* **Institutional correctness:** Policy effects have the correct scope; office powers and eligibility conditions refer to valid capabilities.
* **Architectural compatibility:** Components agree on attachment rules, dimensions, materials, and available renderer bindings.
* **Historical openness:** Flag hard-coded year or era gates unless explicitly intended, rather than quietly embedding a prescribed historical sequence.

Do not reject all cycles indiscriminately. Dependency and inheritance cycles can be errors; economic cycles may be intentional. With alternative prerequisites, use a reachability calculation from bootstrap capabilities rather than declaring every strongly connected component invalid.

Static reachability means “possible under the authored rules,” not “guaranteed to occur in every world.” Balance and plausibility still need small executable scenarios.

### 4.5 Keep pack composition explicit and reproducible

Each pack should declare identity, version, schema version, kernel compatibility, and dependency constraints. Resolve dependencies to an exact lockfile.

Use deterministic ordering, but **do not use ordering as the default conflict-resolution mechanism**. A duplicate definition should fail unless an explicit override names the target.

For an initial implementation, support adding definitions and replacing an entire definition with an expected-target fingerprint. Later, add typed patch operations such as setting a field or adding an element to a set. Preserve provenance throughout.

Avoid unrestricted XPath-like patching, silent last-writer-wins semantics, and deep multiple inheritance. They save source repetition while increasing the cost of understanding the resulting content.

Keep these version concepts separate:

| Version or identity | Purpose |
| --- | --- |
| Content-schema version | How source records are interpreted. |
| Pack version | Human-facing release and dependency compatibility. |
| Kernel content-API version | Which operations and meanings the kernel supports. |
| Save-format version | Structure of mutable world state. |
| Compiled-cache version | Whether a cached representation can be reused. |
| Exact content fingerprints | Which bytes and effective rules were actually used. |

A pack version alone does not identify its contents.

Maintain two useful fingerprints:

**Artifact fingerprint:** Hash of the distributed pack bytes, for exact provenance and integrity.

**Semantic fingerprint:** Hash of the normalized, effective content after dependencies, defaults, and overrides have been resolved.

BLAKE3 is a suitable cryptographic hashing implementation; the canonicalization rules still belong to TCE. [GitHub](https://github.com/BLAKE3-team/BLAKE3)

Canonicalize maps and set-like collections, preserve meaningful list order, normalize quantities and identifiers, and exclude timestamps and local paths. Compute semantic hashes before assigning session-specific runtime slots.

A separate simulation/presentation fingerprint can make cosmetic changes easier to distinguish, but only where that boundary is real. Changing a building’s collision dimensions is not necessarily cosmetic.

Save the exact pack lock and fingerprints with each world, and retain the corresponding pack artifacts. A definition rename needs a content migration; changing stock units or active production requires a world-state migration. Never silently replace a saved world’s dependencies with the newest available packs.

### 4.6 Implement transactional hot reload

Use `notify` as a change detector, not as a transaction log. Its documentation describes editor-dependent write patterns, watcher limitations, and polling alternatives. [Docs.rs](https://docs.rs/notify/latest/notify/)

The proposed protocol is:

1. **Collect a complete candidate source set.** Debounce notifications and rescan affected paths. AI-authored multi-file changes should be staged and explicitly applied; a debounce interval does not guarantee a coherent edit.
2. **Compile away from simulation and rendering threads.** Initially reparse changed files but relink and validate the whole effective registry.
3. **Produce an impact report.** Classify changes and identify caches, world objects, and jobs that depend on them.
4. **Prepare required migration work.** Reject unsupported transitions before touching the active world.
5. **Commit at a tick barrier.** All simulation workers finish the old tick; the owner publishes the new registry and associated state changes together.

Any failure leaves the previous registry active. Tag candidates with request generation and base fingerprint so a slow, obsolete compilation cannot overwrite a newer result.

For a single simulation owner, a channel plus an `Arc<ContentRegistry>` is sufficient. Use `ArcSwap` only when independent concurrent readers justify it. Its documentation supports efficient read-mostly publication, but an atomic pointer swap does **not** invalidate caches or migrate the world. [Docs.rs](https://docs.rs/arc-swap/latest/arc_swap/docs/performance/index.html)

Classify reloads explicitly:

| Change | Recommended treatment |
| --- | --- |
| Localization or genuinely cosmetic bindings | Presentation update. |
| Tuning values used only in future decisions | Next-tick application with cache invalidation. |
| Recipe used by active jobs | Preserve the job’s revision/ledger, or run an explicit migration. |
| New good or technology | Prepare expanded inventories, bitsets, and dependent tables before committing. |
| Removed/renamed definition, changed units, altered state meaning | Explicit migration or require reload/restart. |
| World-generation rules | New-world-only unless deliberate regeneration is implemented. |

An especially dangerous case is changing recipe inputs after inputs have already been consumed. Re-reading the new definition halfway through the job can create or destroy resources. Store the job’s committed quantities and relevant recipe revision.

Similarly, existing laws and buildings may need to retain instantiated parameters. Updating their templates must not silently rewrite history.

### 4.7 Integrate with Unreal without coupling content authoring to Unreal objects

Keep the compiler and registry in a Rust crate shared by the DLL and a headless CLI. Unreal should receive diagnostics and presentation changes through a narrow, versioned C interface.

Use opaque handles and explicitly owned buffers, not Rust `String`, `Vec`, or `Arc` layouts across the boundary. Define how errors are returned and prevent unwinding from crossing an ordinary C ABI boundary. The Rustonomicon documents the relevant FFI and unwinding constraints. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

**Windows packaging trap:** Files placed inside Unreal’s packaged filesystem are not automatically readable using Rust’s ordinary filesystem APIs. Epic’s UE **5.8** documentation distinguishes UFS content from loose **NonUFS** files. Either stage the text/compiled packs as NonUFS, or pass their bytes through an Unreal filesystem bridge. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

For visuals, export a catalog of valid renderer asset bindings for the Rust compiler to validate. Use Unreal’s Asset Manager and soft references for loading, while ensuring referenced assets are included in cooking; a string in a TOML file is not itself a cooking policy. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine)

Content reload should not unload the Rust DLL or invoke Live Coding. Updating text definitions and rebuilding native code are different operations.

### 4.8 Keep the solo-developer implementation small

Build a shared content library with a small CLI surface:

```
tce-content validate --json
tce-content explain core:technology/pottery
tce-content refs --to core:good/clay
tce-content diff --impact
tce-content apply
```

Start with parsing, typed references, semantic lints, provenance, and deterministic compilation. Then add transactional reload and saved-world compatibility tests. Defer a scripting VM, custom language server, incremental-query framework, and elaborate binary cache.

For AI agents, provide canonical examples and machine-readable errors. Require meaningful source changes to pass the compiler and review; do not let agents “repair” content by weakening validators.

On the specified i9 system, keep live content compilation to a budgeted worker rather than competing for every core with the simulation. Measure peak memory with source trees, the active registry, and the candidate registry simultaneously resident. Retire old registries outside critical frame/tick work, and bound how many old revisions may remain retained.

The GPU and 1440p target chiefly affect the presentation side. They do not make text parsing or cross-reference resolution cheaper, and a content architecture alone cannot guarantee 60 fps.

---

## 5. Sources and version applicability

The links below are the most useful implementation references. “Current docs” identifies what was inspected, not a claim that these components have been tested together.

| Area | Documentation, code, paper, or talk | Applicability |
| --- | --- | --- |
| Factorio | [Data lifecycle](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html?utm_source=chatgpt.com), [mod structure](https://lua-api.factorio.com/latest/auxiliary/mod-structure.html?utm_source=chatgpt.com), [migrations](https://lua-api.factorio.com/latest/auxiliary/migrations.html?utm_source=chatgpt.com) | Official **2.1.20** documentation. |
| Paradox tooling | [CWTools](https://github.com/cwtools/cwtools?utm_source=chatgpt.com), [VS Code integration](https://github.com/cwtools/cwtools-vscode?utm_source=chatgpt.com), [Jomini parser](https://docs.rs/jomini/latest/jomini/?utm_source=chatgpt.com) | Community-maintained tooling; Jomini **0.37.1**. Game support varies. |
| RimWorld | [Loader](https://raw.githubusercontent.com/Chillu1/RimWorldDecompiled/master/Verse/LoadedModManager.cs?utm_source=chatgpt.com), [inheritance](https://raw.githubusercontent.com/Chillu1/RimWorldDecompiled/master/Verse/XmlInheritance.cs?utm_source=chatgpt.com), [reference resolution](https://raw.githubusercontent.com/Chillu1/RimWorldDecompiled/master/Verse/DirectXmlCrossRefLoader.cs?utm_source=chatgpt.com) | Decompiled snapshot; exact game patch not independently verified. |
| Dwarf Fortress | [Official modding guide](https://bay12games.com/dwarves/modding_guide.html?utm_source=chatgpt.com), [development history](https://www.bay12games.com/dwarves/?utm_source=chatgpt.com) | Guide for **50.01+**; cited changes in **52.01/52.03**. |
| TOML | [Specification](https://toml.io/en/v1.1.0?utm_source=chatgpt.com), [Rust crate](https://docs.rs/toml/latest/toml/?utm_source=chatgpt.com), [benchmark write-up](https://epage.github.io/blog/2025/07/toml-09/?utm_source=chatgpt.com) | Spec **1.1.0**; crate **1.1.6+spec-1.1.0**; benchmark concerns **0.9.0**. |
| Alternative formats | [RON](https://docs.rs/ron/latest/ron/?utm_source=chatgpt.com), [serde\_yaml](https://docs.rs/serde_yaml/latest/serde_yaml/?utm_source=chatgpt.com), [serde-saphyr](https://docs.rs/serde-saphyr/latest/serde_saphyr/?utm_source=chatgpt.com), [KDL](https://kdl.dev/?utm_source=chatgpt.com), [knuffel](https://docs.rs/knuffel/latest/knuffel/?utm_source=chatgpt.com) | RON **0.12.2**; deprecated serde\_yaml **0.9.34**; serde-saphyr **1.3.0**; KDL spec **2.0.0**; knuffel **3.2.0**. |
| Validation/editing | [Serde attributes](https://serde.rs/container-attrs.html?utm_source=chatgpt.com), [Schemars](https://docs.rs/schemars/latest/schemars/?utm_source=chatgpt.com), [Tombi](https://tombi-toml.github.io/tombi/?utm_source=chatgpt.com), [toml\_edit](https://docs.rs/toml_edit/latest/toml_edit/?utm_source=chatgpt.com) | Schemars **1.2.2**; pin parser/schema/editor compatibility. |
| Reload/runtime | [notify](https://docs.rs/notify/latest/notify/?utm_source=chatgpt.com), [ArcSwap performance](https://docs.rs/arc-swap/latest/arc_swap/docs/performance/index.html?utm_source=chatgpt.com), [Bevy assets](https://docs.rs/bevy_asset/latest/bevy_asset/?utm_source=chatgpt.com) | notify **8.2.0**, ArcSwap **1.9.2**, Bevy assets **0.19.1**. |
| Unreal/Rust boundary | [UE third-party libraries](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com), [Asset Manager](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine?utm_source=chatgpt.com), [Rust FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Epic documentation inspected for **UE 5.8**; current Rustonomicon. |
| Dependency-system background | [*Build Systems à la Carte*](https://www.microsoft.com/en-us/research/publication/build-systems-la-carte/?utm_source=chatgpt.com), [executable code](https://github.com/snowleopard/build?utm_source=chatgpt.com), [Neil Mitchell’s talk index](https://ndmitchell.com/?utm_source=chatgpt.com), [Salsa](https://github.com/salsa-rs/salsa?utm_source=chatgpt.com) | Paper: **ICFP 2018**. Relevant talk: *Distributed Build Systems*, **May 18, 2018**; author’s index links slides/video. Background for compiler dependencies, not a simulation benchmark. |

**Bottom line:** TCE needs a small, dependable **content compiler** more than it needs a sophisticated content language. Start with TOML, strict authoring types, explicit references, provenance, and semantic lints. Compile once into indexed immutable data, then make every reload an intentional transition between two valid versions of the world’s rules.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927c0-6efc-83ea-9f1e-f227090c631b)
