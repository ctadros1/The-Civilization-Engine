# The Civilization Engine: Project Plan

Status: plan of record, written 2026-09-27 from the planning interview.
Implementation (2026-10-03): **M0 Foundations and M1 A band settles are implemented** (people,
foraging, farming, huts, births, deaths and families, worn trails, the first god tool and running
ahead). The README lists what exists, what is planned and the known limitations.
Planning happens on the MacBook; development and running happen on the Windows PC
(i9 13th gen, RTX 4070 Ti with 12 GB VRAM, 64 GB DDR5).

---

## 1. Vision & scope

The Civilization Engine is an endless, as-realistic-as-possible simulation of
people who found settlements, grow them into cities, and build their own
societies: governments, laws, economies and money, companies, technology,
architecture and city layout. It aims to script those people as little as
possible. The observer watches, inspects everything, and occasionally
intervenes as a god, but never governs.

The engine **authors the vocabulary and never the plot**. Offices, property
regimes, technologies, policies, style elements and service capabilities are
authored primitives. Which ones a society assembles, in what order, and whether
it ever does, is up to its people. "Monarchy", "capitalism", "industrial era"
and "Gothic" are labels the UI works out after the fact from what the people
built. They are never states the engine sets.

It is a personal project built to product polish: real UX, save/load,
stability. It is explicitly **not a research instrument**. The Genesis Engine
(TGE) produced 39 ADRs and a set of experiment campaigns within its first month. This project is
organized to avoid that.

### Non-negotiable rules

1. **Author the vocabulary, never the plot.** No scripted events, eras,
   storylines or per-playthrough outcomes.
2. **Usable at every milestone.** Every milestone ends in a build a
   non-developer can launch, create a world in, watch, save, load and quit
   without data loss. It never ends in a report.
3. **Emergence is time-boxed.** If a targeted behavior (say, "factions form
   along occupation lines") doesn't show up in the smoke-test seeds after about
   one week of tuning the primitives, author a nudge (a template, prior,
   weight or trigger), log it in §9 as `NUDGE:`, and move on.
4. **Realism means plausible, not proven.** Plausibility is checked against a
   short list of real-world stylized facts on a sanity dashboard (§4.7). There
   are no preregistered campaigns, power estimates or null criteria.
5. **The kernel owns truth. Unreal draws and forwards commands, and decides
   nothing.**
6. **Determinism is not a goal.** Every world plays out differently, even from
   the same seed. Saves are snapshots. Bugs are reproduced from the snapshot
   taken before them, not by exact replay.
7. **ADRs only for decisions that are expensive to reverse**: schema,
   persistence, the FFI boundary. If a milestone needs more
   than about three ADRs, it is too big and gets split.

### v1 is NOT trying to do

"v1" means milestones M0–M8 (§7): the first release-quality build.

- **No industrial or modern content.** The tech graph in v1 runs from early
  agrarian to roughly pre-industrial: masonry, aqueducts, coinage, banking,
  printing, sail. There's no electricity, power grid, motor vehicles or transit
  yet (M10–M11). Worlds can be *started* later on the graph only once that
  content exists.
- **No aggregate LOD, no time skips, and no more than ~3–5 settlements.** Every
  agent is simulated in full; the aggregate model arrives in M9.
- **No healthcare, education, power or waste as full services.** Each is a
  single abstract stat (healer availability, literacy/skill rate) until its
  later milestone.
- **No LLM-driven agents.** The `Deliberator` interface exists; the only backend
  is rule-based.
- **No enterable interiors and no MetaHuman-grade close-ups.** Fake interiors
  and crowd-grade citizens only.
- **No tactical or naval combat.** War is operational (§5.5).
- **No multi-viewer or co-op**, though the architecture keeps multi-viewer cheap.
- **No hand-authored buildings or landmarks.** Everything is grammar-generated.
- **No physics-based construction or structural solver.** Construction is
  staged and structures fail by rule (§5.1), not by simulated physics. A real
  structural solver is a future consideration (§8). Physics-based building,
  with people placing individual stones and beams, is out of scope; that's
  Prometheus's question.
- **No direct player governance.** The player never places buildings, zones or
  enacts laws; god tools only (§2).
- **No player terrain editing.** God tools can't reshape terrain; that's a
  future consideration (§8). The people's small-scale earthworks *are* in
  scope (§5.1).
- **No Mac or console builds.** Windows only; the Mac is for planning.
- **No mod manager, localization, achievements, goals or win conditions.**

---

## 2. Player experience

**Role: observer with god tools.** The player is outside the world. They
watch, inspect and follow, and occasionally intervene through god tools. Those
tools change conditions, people and ideas, never institutions directly. The
player cannot enact a law, place a building or appoint an official. They can
make a plague happen, send migrants, introduce an idea, or whisper to a notable
who may ignore them.

### Core loop

1. **Create a world.** Seed, map size, climate, founding groups (count, size,
   culture priors), and the known-technology set at founding (default: early
   agrarian).
2. **Watch** at 1x, where daily life is visible, or speed up.
3. **Inspect anything.** Click a person, building, firm, law, faction, office,
   incident, treaty, army or settlement to open its panel. Every entity has a
   **history** and a **"why"**: why this law passed (votes, faction support),
   why this person stole (need, opportunity, perceived enforcement), why this
   house is stone (building code after the Great Fire of year 41).
4. **Follow.** The camera tracks a person, notable, army or caravan.
5. **Read the chronicle.** An automatically generated, filterable history of
   the world ("Year 34: the Elders' Council enacted a land tax; 212 households
   evaded it").
6. **Intervene sparingly** with god tools.
7. **Save and branch.** Save any time, and branch "what if" worlds from a save.

**Session length:** open-ended. Worlds persist across sessions with rolling
autosave. A typical session is 30–120 minutes.

### UX pillars

- **Legibility:** "why" is available everywhere.
- **History:** nothing important is lost; every entity has a timeline.
- **Comparison:** settlements side by side (regime, economy, style, services).

### God tools, and the milestone each arrives in

| Category | Tools | Milestone |
|---|---|---|
| Inject people & ideas | Spawn a family (M1); introduce a technology (M3); introduce an ideology or agitator (M4); migration wave (M5) | M1–M5 |
| Nudge individuals | Whisper to a notable (a suggested goal they evaluate like any other); bless or curse a person or firm (luck modifier) | M4 |
| Disasters & events | Fire, plague, flood, drought, storm (M6); earthquake (M8) | M6, M8 |
| World edits | Place or remove resource deposits (M3); change climate (M9) | M3, M9 |

Every god-tool use is logged in the chronicle, so its consequences can be
traced.

**The player cannot reshape terrain.** A terraforming god tool is a future
consideration only (§8). The *people* can terraform a little through
earthworks (§5.1).

---

## 3. Architecture

### 3.1 Shape

```
┌──────────────────────────── Unreal Engine process (Windows) ────────────────────────────┐
│                                                                                         │
│  UE game thread / render thread               Sim worker threads (owned by the plugin)  │
│  ┌────────────────────────────┐   commands    ┌────────────────────────────────────┐    │
│  │ EngineBridge plugin (C++)  │──────────────►│ civ kernel (Rust cdylib via C ABI) │    │
│  │  • frame reader            │  (FFI queue)  │  • multithreaded tick scheduler    │    │
│  │  • interpolation of trips  │◄──────────────│  • agents, notables, economy,      │    │
│  │  • BuildingAssembler ──────┼──┐  frames    │    polity, services, diplomacy     │    │
│  │  • crowd, PCG dressing     │  │ (triple    │  • snapshots + autosave            │    │
│  │  • UMG HUD + time controls │  │  buffer)   │  • civ-grammar (same DLL) ◄────────┼──┐ │
│  │  • WebBrowser panels ◄─────┼──┼──WS 127.0.0.1──  localhost WS host (in-DLL)     │  │ │
│  └────────────────────────────┘  │            └────────────────────────────────────┘  │ │
│                                  └─ grammar_expand(BuildingSpec) over FFI ────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────────────┘

Headless (same kernel crates, no UE):
  civ-host CLI ── runs worlds, tests, smoke seeds ── serves the same WS schema ──► browser
                                                                                   (2D observer
                                                                                    + panels)
```

- **The kernel is a Rust workspace with zero Unreal dependency.** It builds two
  ways: `civ-ffi` (a cdylib loaded by the UE plugin) and `civ-host` (a headless
  CLI for tests, CI, smoke seeds, and the web observer).
- **In-process by default, transport-agnostic by design.** Everything crossing
  the boundary uses one versioned schema: snapshot, delta and event frames going
  out; commands and queries coming in. The schema is the same over FFI, a
  localhost WebSocket, or a recording file. That keeps multi-viewer, recording
  replay (Prometheus's mode) and a later out-of-process move all open.
- **The sim never runs on UE's tick.** The plugin owns dedicated sim threads.
  The kernel publishes frames into a triple buffer at up to 30 Hz, and UE reads
  the latest one.
- **Agents move by trips, not per-frame positions.** The kernel emits
  `TripStarted {agent, path_id, t_depart, t_arrive}` events, and UE
  interpolates along the cached path. This makes 50k agents cheap to stream.
- **Web panels speak WebSocket to both hosts.** In UE, the DLL hosts a
  localhost WebSocket bound to 127.0.0.1 and the WebBrowser-widget panels
  connect to it. Headless, `civ-host` serves the same thing. The panels are
  identical code in both, and nothing is thrown away when UE arrives.

### 3.2 Languages and key tech choices

| Area | Choice | Why |
|---|---|---|
| Kernel | Rust (pinned toolchain), plain data-oriented tables with generational IDs, no ECS framework; parallel systems via rayon, throughput first | Same proven storage pattern as Genesis/Prometheus; plain tables keep the hot loops simple and fast |
| Numerics | f64 with ordinary platform math | Exact reproducibility isn't a goal (§1 rule 6), so there are no cross-platform float constraints |
| Boundary schema | **Recommended: FlatBuffers** (generated Rust, C++ and TS readers). At M0, first check whether Genesis's `sim-protocol` already covers this; if so, lift it into commons instead | One schema file instead of three hand-maintained decoders, with documented evolution rules |
| FFI | C ABI via `cbindgen`; `catch_unwind` at every exported function; Rust panics become error codes plus a crash snapshot | Panics must never unwind into UE |
| Authored content | Text data files (RON/TOML) in `content/`, schema-validated and hot-reloadable in dev | Agents author data well; the primitives stay visible and editable |
| UE | Pin the latest stable 5.x at M2 start (5.8 at time of writing); upgrade only between milestones | Stops engine churn from landing mid-milestone |
| UE code | C++ and UE Python editor scripts; no gameplay logic in Blueprints; assets generated by scripts from text manifests | Text is diffable and agent-friendly; binary Blueprints are neither |
| HUD | UMG/CommonUI: selection, time controls, god-tool palette, notifications | Native feel for high-frequency controls |
| Data panels | TypeScript + Vite; PixiJS for the 2D observer map (reuses Genesis's approach); a charting library chosen in M1; embedded in UE via the WebBrowser widget | Charts and tables are much faster to build and polish on the web, and the same panels double as a standalone dashboard |

### 3.3 Shared code with Prometheus and Genesis (`engine-commons`)

A separate repo (suggested: `ctadros1/Engine-Commons`) holds code that isn't
specific to any one engine. *Until a second engine adopts it, it is staged in this repository as
its own Cargo workspace under `commons/`; extracting it is a `git subtree split` (see
`commons/README.md`).* **Admission rule:** code goes in only if at least
two engines use it, or will within one milestone. It must contain no
engine-specific concepts: no "agent", "settlement" or "organism" types.

| Package | Contents |
|---|---|
| `commons-wire` | Frame envelope (snapshot, delta, events, commands, query/response), schema versioning, transports (FFI ring buffer, WebSocket, recording file) |
| `commons-persist` | Container format: magic, container and schema versions, named sections, per-section and whole-state checksums, a content-pack fingerprint, and refusal (never repair) on mismatch. The same posture as Prometheus's `sim-persist` |
| `commons-rng` | Keyed, purpose-tagged deterministic RNG derivation. Prometheus and Genesis need it; TCE doesn't |
| `commons-record` | Recording format for frame streams a viewer can play back (Prometheus's replay mode; TCE could use it to record a world for later viewing) |
| `EngineBridge` (UE plugin) | Rust DLL loading/unloading, sim-thread hosting, triple-buffer frame reader, trip interpolation, instance-pool management, camera rigs (city, follow, cinematic), WebBrowser panel host. **Supports two sources: live FFI (this project) and recording (Prometheus V1)** |
| `web-kit` (TS) | Inspector, timeline, chart and table components, and the WebSocket client for `commons-wire` |

Each engine pins commons by git tag: Cargo git dependencies, and the UE plugin
and `web-kit` vendored by a sync script that records the tag. One engine's
change can't break another until that engine bumps its tag. **This project
builds `EngineBridge` in M2, and Prometheus's V1 Unreal work adopts it.**
Prometheus's V1 is currently waiting on the UE install anyway, so that's the
cheapest point to share.

### 3.4 How agent decisions become geometry and city systems

```
agent/firm/office decision         kernel state                     Unreal
────────────────────────────       ─────────────────────────        ─────────────────────────
"household needs a bigger house"─► ConstructionProject             (scaffold stage visuals)
  program := choose(               {plot, program, BuildingSpec,
    library ∩ known_tech            labor+materials consumed
    ∩ legal(zoning, codes)          over time}
    ∩ affordable)                         │ completes
  params := f(budget, household,          ▼
    culture.style_vector,          BuildingSpec {footprint, storeys,  ─► BuildingAssembler:
    local materials, climate)        grammar_id, param vector,           grammar_expand(spec)
                                     material ids, style seed,           → module placements
                                     condition, age}                     → ISM batches of Nanite
                                          │                                kit meshes
                                          └─ derived facts via the same  → per-instance data:
                                             civ-grammar crate: floor      weathering, window
                                             area, capacity, fire load,    lights, colour
                                             window count
```

- The **kernel stores only `BuildingSpec`** (tens of bytes). Geometry is
  derived, never saved.
- `civ-grammar` is **one Rust crate compiled into the same DLL**. The kernel
  uses it for derived facts: capacity, fire load, and the spans and loads the
  structural rules need (§5.1). UE calls `grammar_expand` over FFI for
  geometry. The web observer draws footprints from the same specs. There is
  one source of truth: the same spec always expands to the same building, so
  buildings don't change shape when a save is reloaded.
- **Construction stages come from the grammar too.** `grammar_expand` tags
  every piece with a stage (foundation, frame, walls, roof, finish). The
  kernel's `ConstructionProject` reports progress per stage, and UE reveals
  exactly the pieces that have been built.
- Layout works the same way. The kernel owns the road graph, parcels and land
  use; UE renders roads as spline meshes and uses runtime PCG for dressing.

### 3.5 Save/load and stability

- **Not deterministic by design.** Runs aren't reproducible from a seed, and
  that's intended (§1 rule 6). A seed only varies world generation. Nothing is
  recorded for exact replay.
- **Snapshots** use `commons-persist`, with a content-pack fingerprint. Old
  snapshots still load when the schema is compatible; the save browser notes
  when the content has changed since the save was made.
- **Autosave:** rolling, every in-game month or 5 real minutes, whichever comes
  first. Keep the last N. Writes are crash-safe (temp file, fsync, rename,
  verify checksum).
- **Schema migration:** explicit, versioned migrations. Anything else is
  refused with a clear message. The save browser shows compatibility.
- **Crash policy:** a kernel panic writes a crash snapshot and a
  last-good-snapshot pointer, and UE shows a recovery dialog. A UE crash loses
  at most the time since the last autosave.
- **Editor workflow:** the headless CLI and the web observer are the primary
  sim dev loop. The DLL is reloaded explicitly through the plugin; Live Coding
  does not cover Rust.

### 3.6 Repository layout (created progressively; a crate appears in the milestone that needs it)

```
The Civilization Engine/
  PROJECT_PLAN.md   CLAUDE.md   AGENTS.md
  decisions/                # ADRs (rule 7)
  kernel/                   # Rust workspace
    crates/
      civ-core              # ids, tables, tick scheduler, time, RNG                  (M0)
      civ-world             # world-gen: terrain, hydrology, climate, soils, deposits  (M0: terrain, water)
      civ-content           # content compiler: packs, diagnostics, fingerprints      (M0)
      civ-sim               # composition root: world state, save/load, payloads      (M0)
      civ-land              # habitat patches, wild stocks, fields, plots, path wear, settlements (M1)
      civ-agents            # citizens, needs, schedules, utility decisions, demography, culture
      civ-notables          # deliberation layer + Deliberator trait
      civ-tech              # technology graph, discovery, diffusion
      civ-econ              # goods, recipes, property/allocation/labor regimes, firms, markets, money
      civ-polity            # constitutions, offices, policies, law pipeline, factions, unrest
      civ-services          # incidents: water/disease, crime/justice, fire, transport
      civ-diplomacy         # relations, treaties, armies, war, sovereignty changes
      civ-grammar           # building + layout grammars; reads art/kits manifest
      civ-schema            # boundary + save schemas (FlatBuffers)                   (M0)
      civ-ffi               # cdylib for UE
      civ-host              # headless CLI + WS host                                  (M0)
  commons/                  # engine-commons, staged until a second engine adopts it   (M0)
  content/                  # authored primitives (TOML) + validator                   (M0)
  web/                      # TS panels + 2D observer                                  (M0 shell)
  unreal/                   # UE project; Plugins/EngineBridge vendored from commons
  art/                      # Blender sources, kit manifests (sockets, sizes, style tags)
  tools/                    # run scripts, schema codegen; later the UE import pipeline
```

---

## 4. Simulation design

### 4.1 Citizens (every person)

- **State:**
  - Identity and kin: name, birth, sex, parents, spouse, children.
  - Body: age, health, nutrition, disease states.
  - Needs: hunger, rest, shelter, safety, social, status, purpose.
  - Skills by occupation, with experience.
  - Known technologies.
  - Personality: 5 traits.
  - Values: ideology axes such as authority↔liberty, collective↔individual,
    tradition↔change, in-group↔universal, sacred↔secular.
  - Sparse relationships: kin, friends, rivals, employer, patron.
  - Bounded memory: significant events with valence, which feed grievances and
    loyalties.
  - Household, property and wealth, occupation, schedule template.
- **Decisions:** utility-based choice over an authored action catalogue.
  Considerations include needs, prices, wages, norms, *perceived* enforcement of
  laws, relationships and personality. Choices are weighted-random (softmax), so people don't all make the
  identical best choice.
- **Cadence:** day-to-day actions (work, eat, shop, socialize, commute) run in
  the activity scheduler. Life decisions run monthly or seasonally: marry,
  have children, migrate, change job, found a household or firm, join a
  faction, protest, evade, commit a crime, emigrate.
- **Culture:** communities carry aggregate values, norms (what is tolerated)
  and a **style taste vector**. These drift through social learning: conformity,
  prestige bias (copying admired people and buildings), and contact with other
  cultures.
- **Demography:** fertility (wealth, norms, health), mortality (age, disease,
  famine, violence), and migration (push and pull).

### 4.2 Notables (the second tier)

- **Selection:** the top roughly 1% of the population by an influence score
  (office, wealth, followers, founder status, reputation), capped at about 500.
  Membership is dynamic: people rise and fall.
- **Deliberation:** a goal-driven planner (HTN) over authored institutional
  *moves*: propose a policy, draft or amend a constitution, form or join a
  faction, found a firm, commission a building or monument, patronize a style,
  offer a treaty, raise an army, stage a coup, lead a revolt. Moves are scored
  on the notable's goals and on predicted support, using coalition arithmetic
  over the factions.
- **`Deliberator` trait:** the rule-based backend is the only one in v1. An
  `LlmDeliberator` (local or API) can be plugged in later. **Off by default.**

### 4.3 Population and settlement targets

| Milestone | Agents | Settlements |
|---|---|---|
| M1 | 20–50 | 1 |
| M3 | 200–500 | 1 |
| M4 | 1–2k | 1 |
| M5–M7 | 3–8k total | 2–3 |
| M8 (v1) | 10–50k total | 2–3 (up to 5) |
| M9+ | 100k+ via aggregate LOD | 10–30 |

### 4.4 Time, tick rate, speed modes

- **1x = one in-game day per 15 real minutes** (configurable 10–20). Daily life
  is visible: commutes, work, markets, night shifts, incidents.
- **Base tick = 1 in-game minute.** Agents are **event-scheduled**: a tick
  processes only agents whose activity ends in that minute, plus incidents and
  dispatches. Most agents cost nothing in most ticks.

**Cadences:**

| Cadence | Systems |
|---|---|
| Every minute | Activities, trips, incidents |
| Hourly | Service dispatch and incident progression |
| Daily | Household economy, market prices, disease progression |
| Weekly/monthly | Firm decisions, notable deliberation, law pipeline steps |
| Seasonal/yearly | Technology discovery and diffusion, style drift, scheduled elections, demography summaries |

**Speed modes:**

| Mode | Speeds | Resolution |
|---|---|---|
| Detailed | Pause, 1x, 3x, 10x | Full sub-daily simulation |
| Accelerated | 60x, 600x, Max | Each agent's day is resolved **statistically in one daily step**: hours worked, purchases, contacts sampled from co-location pools, crimes and fires sampled from the same motive and ignition models |

Both modes call the **same decision functions** at different granularity.
Switches happen at day boundaries. **Target:** at 50k
agents, Accelerated Max reaches at least 1 in-game year per real minute. A
century is then under two hours, which is what an endless, early-agrarian-start
world needs.

**Mode-consistency test:** one in-game year run in each mode must produce
aggregate statistics within tolerance. This is a regression test, not a
research result.

### 4.5 LOD strategy

- **Simulation:** everyone is simulated in full through v1.
- **Aggregate model (M9):** a settlement can run as cohorts (age × sex ×
  occupation × wealth × ideology bins) with rates calibrated from the agent
  simulation. When focused, individuals are synthesized from the cohort
  distributions. Notables persist as named individuals in both modes.
- **Time skips:** the same aggregate machinery would serve a "skip 100 years"
  feature. That feature is a **deferred decision** (§8).
- **Rendering:** full crowd near the camera; instanced vertex-animated
  characters at mid distance; density or impostors far away. Only the followed
  agent gets a full skeletal rig.

### 4.6 Performance budgets (Windows PC)

- **Render:** 60 fps at 1440p with DLSS/TSR, at M8 scale, on the 4070 Ti. The
  sim may fall behind the requested speed, but the frame rate holds.
- **Kernel memory:** ≤ 4 GB at 50k agents.
- **VRAM:** UE ≤ 10 GB.
- **Kernel frame publish:** ≤ 2 ms of UE game-thread time to consume.

### 4.7 Sanity dashboard and smoke seeds (the anti-research guardrail)

A headless nightly run: **5 worlds × 50 in-game years**, using thresholds
only, with no statistical inference. The checks:

- Population neither goes extinct nor explodes.
- Food prices show seasonality.
- Wealth Gini stays within 0.3–0.75.
- Firm-size and settlement-size distributions are right-skewed.
- Crime correlates positively with poverty.
- Waterborne epidemics cluster on contaminated sources.
- At least two of the five seeds end with differently labeled regimes.
- Structural failures per 1,000 buildings per year stay within a tuned band
  (§5.1).

A failure blocks the milestone. Five seeds is a smoke test; it is never scaled
up into a campaign.

---

## 5. City systems

Each system lists its **authored library** (vocabulary) and the **decision
points** agents control (plot).

### 5.1 Structures & layout

**Authored library:**

- **Building programs**, gated by technology and law: hut, longhouse, house
  (1–4 storeys), workshop, granary, storehouse, shrine/temple, hall, market,
  tavern, barracks, walls/gates/towers, bridge, well, cistern, aqueduct, bath,
  palace, monument.
- **Monument templates** use the same grammars at larger scale, with rarer
  rules: domes, towers, colonnades, stairs. There are no prefabs.
- **Grammar rules:** footprint → massing → floors → facade bays → openings →
  roof → ornament.
- **Style primitives** drawn from real-world vernaculars: roof form and pitch,
  wall material, window proportion and rhythm, ornament density, symmetry,
  colour palette, eaves and cornice types.
- **Layout algorithms:**
  - Organic: path wear → desire lines → road graph → frontage plots.
  - Planned: grid, radial, linear, axial (cardo/decumanus), each with
    parameters: block size, road hierarchy widths, plaza frequency.
  - Parcel subdivision: recursive oriented-bounding-box splits.
- **Earthworks (the people terraform a little):** programs that edit the
  heightfield locally.
  - Leveling plots for foundations.
  - Road cut-and-fill and grading.
  - Quarry and clay pits.
  - Irrigation and drainage ditches.
  - Farm terraces.
  - Small embankments and levees.

  Earthworks are construction projects that consume labor and tools over time,
  are gated by technology, and are recorded in world state. Each one is
  **bounded**: a local patch with a volume limited by labor, and ditches and
  levees change local drainage only. There's no large-scale reshaping: no land
  reclamation, river diversion or hill removal.

**Agent decision points:**

- Where to found a settlement: site scoring on water, fertility, defensibility,
  resources, trade access.
- Which plot to claim or buy.
- What program to build, and how big (budget), how tall (land value, tech).
- Style: the owner's taste vector, modulated by prestige copying.
- When to repair, upgrade or demolish.
- Whether to level a sloped plot or build on the slope, terrace a hillside for
  farming, dig irrigation or drainage, or open a quarry. Public earthworks such
  as levees after a flood go through the law pipeline as public works.
- Governments decide **whether a planning institution exists**. If it does, it
  picks a plan template and parameters, zoning, and **building codes**. For
  example, masonry becomes mandatory after a fire, but only if someone proposes
  it and it passes.

**Style drift:** the style vector moves under several pressures:

- Prestige copying of wealthy or admired buildings.
- Contact with other cultures through trade, conquest and migration.
- Ideology, e.g. austerity values lower ornament.
- Climate and material availability, which act as hard constraints.

**Construction and structural integrity.** People visibly build, and what they
build can fail because of how it was built. There's no physics engine and no
load solver.

- **Visible construction:**
  - A `ConstructionProject` advances stage by stage (foundation, frame, walls,
    roof, finish) as labor and materials are spent.
  - Workers are real agents hauling materials from storehouses, quarries and
    markets on ordinary trips.
  - Scaffolding and centering frames appear and come down.
  - A project that runs out of money, materials or workers stops, and the
    half-built building stays visible until work resumes or it's abandoned.
- **Structural rules:** each structure has a **structural margin**, computed
  when it's finished and again whenever something changes.
  - *Capacity* comes from material strength class × technique × builder
    skill × foundation fit.
  - *Demand* comes from span, storeys and load, slope and soil, plus events:
    floods, storms, fire damage, earthquakes, overloading (an army crossing a
    bridge).
  - The margin decays with age and poor upkeep.
  - At each check, the chance of failure rises steeply as the margin shrinks.
    Partial failures (cracks, sagging, closure) come before full collapse.
  - A collapse kills or injures the real people inside or on top.
- **Bridges:** span and load limits per technique are authored in content data:
  timber beam, rope suspension, timber truss, and the stone arch (with
  centering) once it's discovered.
- **Learning:**
  - Each culture holds a **trust score per technique**. A collapse lowers
    trust, so builders avoid the technique or over-build; long survival raises
    it.
  - The builder, whether a person or a firm, takes the reputational hit.
  - A collapse enters the chronicle, and anyone can propose a building-code
    response through the law pipeline.
  - Negligence becomes a crime only if some polity's law makes it one.
- **Agent decision points:**
  - Builders choose the technique and how much to over-build (safety vs.
    cost), shaped by budget, law, reputation and trust.
  - Corrupt officials can approve substandard public works (M7).
- **Rendering:** collapse is purely cosmetic in UE. The affected module
  instances are swapped for pre-fractured pieces that fall under Chaos physics,
  then replaced by the rubble state the kernel reports. The debris never feeds
  back into the simulation.

### 5.2 Government & law

**Authored primitives:**

- **Polity:** territory, membership and citizenship rule.
- **Office:** title, powers, number of seats, term, **selection method**, and
  removal method.
- **Selection methods:** heredity (with rule variants), election (franchise and
  voting rule), appointment by another office, sortition, acclamation, seizure
  by force.
- **Powers:** legislate on a domain, tax, command force, judge, appoint,
  declare war, make treaties, mint currency.
- **Bodies:** councils and assemblies, each with a membership rule and a
  decision rule.
- **Checks:** veto, approval by a body, term limits, judicial review (M7).
- **Constitution:** the set of offices, bodies and checks, plus an amendment
  rule.
- **Label inference:** a classifier maps a structure to names (chiefdom,
  monarchy, oligarchy, republic, theocracy, one-party state, junta, direct
  democracy, hybrids) with confidence and "why".

**Policy library:** typed policies with parameters:

- Taxes (poll, land, trade, income) and tribute/levies.
- Conscription and corvée.
- Land tenure.
- Zoning and building codes.
- Market regulation: price caps, guild monopolies, licensing.
- Tariffs and embargoes.
- Curfews and prohibitions (alcohol, weapons, assembly, speech).
- Religious or ideological mandates.
- A punishment schedule.
- Relief and welfare.
- Public works commissions.
- Currency: minting and debasement.
- Citizenship and migration rules.
- Military service.

**Law pipeline (staged):**

1. **Proposal**, by whoever holds the power.
2. **Support**, computed from factions and interests.
3. **Decision**, by the constitution's own procedure (decree, council vote,
   referendum).
4. **Promulgation**, as awareness diffuses.
5. **Enforcement.** Capacity = officers × competence × (1 − corruption).
6. **Compliance**, per agent: cost vs benefit vs legitimacy vs norms, so evasion
   is real.
7. **Effects and feedback**: grievance and legitimacy.

Courts and precedent come in M7.

**Transitions:**

- Reform, through the amendment rule.
- Succession, which becomes a crisis when the rule is ambiguous or contested.
- Coups, by holders of force.
- Revolution: mass unrest plus a faction with an alternative. **The winning
  faction assembles a new constitution from the primitives according to its
  ideology.**
- Secession (§5.5).

**Politics order** (you asked for all four; this is the build order):

| Milestone | Politics |
|---|---|
| M4 | Factions and unrest (the base everything else needs) |
| M4 | Corruption v0 (bribery of enforcers) |
| M7 | Elections and parties |
| M7 | Full corruption and patronage (nepotism, institutional capture, drag on growth and discovery) |
| M7 | Lobbying and capture by firms and guilds |

**Agent decision points:**

- Notables propose policies and constitutions.
- Citizens support or oppose, comply or evade, join factions, vote, protest,
  riot, emigrate.
- Officeholders enforce, or take bribes.

### 5.3 Economy & companies

**Authored primitives:**

- **Goods catalogue:** by technology, with inputs, perishability and weight.
- **Recipes:** production functions by technology.
- **Property regimes:** who may own land and capital (individual, household,
  kin group or commune, temple, guild, state).
- **Allocation mechanisms:** markets with posted prices; quotas and planning by
  an office; redistribution by chief or temple; reciprocity among kin.
- **Labor regimes:** household, wage, apprenticeship, cooperative, corvée/levy,
  bonded labor (serfdom).
- UI labels (feudalism, command economy, market capitalism, guild economy) are
  inferred from the mix.

**Money as discovered technology/institution nodes:**

1. Barter.
2. Commodity money (grain, cattle, metal by weight).
3. Coinage: minted by an authority, with seigniorage and the option to debase.
4. Credit and moneylending.
5. Banking and bills of exchange.
6. Backed paper.
7. Fiat (post-v1).

Once polities mint, **each polity has its own currency**. Exchange rates emerge
from trade balance and trust.

**Firms:**

- A firm's legal form follows the regime: household workshop, guild workshop,
  partnership, joint-stock company, cooperative, temple estate, state
  enterprise.
- Each has a name, founder, owners or shares, employees, capital (buildings,
  tools), inventory, known recipes, and a price and wage policy (adaptive
  heuristics on inventory-to-sales and vacancy rates).
- **Books:** a daily ledger, and a monthly P&L and balance sheet.
- Lifecycle: bankruptcy and liquidation, mergers, nationalization and
  privatization by policy.
- Every firm has an inspectable page with its history.

**Markets:**

- Posted prices per seller.
- Buyers search a bounded set of known sellers, weighted by distance.
- Prices adjust from inventory.
- Between settlements, merchant firms arbitrage along routes.
- There is no equilibrium solver. That is deliberate: the adaptive-heuristic
  agent-based models are what allow booms, busts and shortages.

**Minimum viable loop (M3):**

- Farm or forage → grain → mill → flour → bread.
- Forest → timber.
- Quarry → stone.
- Smith → tools (raises productivity).
- Households consume food, need shelter, and wear out tools.
- Labor happens in households and workshops, with barter, then commodity money.

**Agent decision points:**

- Occupation.
- Founding a firm: when an opportunity is perceived and there is capital and
  legal permission.
- Prices and wages.
- Saving, spending and lending.
- Trading abroad, and smuggling around tariffs.
- Politically: property-regime change (enclosure, collectivization,
  nationalization) through the law pipeline.

### 5.4 City services

Every service is defined **by function, with a capability ladder unlocked by
technology**. For example, fire runs: none → bucket brigade (a norm) →
organized watch → hand-pump engines → piped water and hydrants → steam engines
(post-v1) → motor engines and alarm telegraph (post-v1). Each rung sets
response speed, suppression rate and coverage.

The **provider is an institution**: community volunteers, a firm (private fire
companies, private watchmen), a temple, or a state office. It is funded by
taxes, fees or insurance, as the polity decides.

**Observer view: incidents plus aggregates.** Incidents are sim entities with
a lifecycle: this house caught fire; this crew was dispatched and arrived in
11 minutes; this cholera case traces to this well. The dashboards (response
time distribution, coverage map, incidence rates, satisfaction) are computed
from the incident log.

| Service | v1 model | Agent/institution decisions |
|---|---|---|
| **Water, sewage & disease** (M6) | A hydrology-driven contamination field (latrines, cesspits and waste upstream of wells and rivers); water sources on a ladder from well to cistern to aqueduct to piped water; agent-level SEIR per disease template (waterborne, airborne, contact) with transmission via shared sources and co-location (home, work, market); immunity; mortality from nutrition and healer availability | Whether to build and fund wells, cisterns, aqueducts and sewers; hygiene norms; quarantine policy; where to live |
| **Crime, policing & justice** (M4, courts M7) | Motive (need, greed, grievance) × opportunity × perceived enforcement × norms → crime; victims remember; watch patrols and investigation → arrest → punishment from the policy schedule; corruption lets offenders buy out | Who polices (community, private, state); punishment schedule; reporting, vigilantism, bribery |
| **Fire** (M6) | Ignition from hearths, workshops, lightning and arson; spread over building adjacency by material flammability, wind and dryness (weather); response on the road graph | Brigades and their funding; building codes; firebreaks in plans |
| **Transport & traffic** (organic roads M1; full system M8) | Road graph from path wear and plans; v1 modes are walk, cart, pack animal and boat; edge capacity with time-bucketed congestion; trips get travel times from congested costs; Accelerated mode uses aggregate flow assignment | Road building and upgrades (public works), tolls, planning, where to live relative to work |

**Abstracted in v1:** healthcare is a healer-availability stat; education is a
literacy and skill-acquisition rate, which also feeds technology discovery;
waste folds into contamination; power doesn't exist yet.

### 5.5 Diplomacy & war

**Authored primitives:**

- **Relation states:** unknown, contact, peace, trade treaty, alliance
  (defensive or offensive), tributary/vassal, protectorate, war (with war
  goals), truce.
- **Treaties** are documents made of clauses from a library. Each side ratifies
  by its own constitutional procedure, so a republic's assembly can refuse what
  its envoy signed.

**Sovereignty changes:** all four you asked for:

- **Conquest → annexation.** The occupied settlement is absorbed under the
  victor's constitution, or given a puppet government.
- **Vassalage and tribute.**
- **Secession.** A district or colony declares independence and forms a new
  polity, possibly triggering war. This is also the natural way new settlements
  get founded: splinter groups pushed out by land pressure or politics.
- **Merger and federation.** Two polities negotiate a combined constitution
  from the primitives.

**Operational war:**

- Armies are raised from real citizens under the military policy (levy,
  professional, mercenary firm).
- They are equipped from the economy (weapons are goods) and supplied along
  routes.
- They move visibly on the road graph.
- Battles resolve statistically: a Lanchester-style model with technology,
  terrain, fortifications, morale and a commanding notable.
- Consequences include sieges, raids, burned districts, real casualties,
  refugees, war debt and grievances.

**Agent decision points:**

- Rulers declare war, sue for peace, and offer or accept treaties according to
  their goals (land, resources, revenge, ideology).
- Citizens support war or protest, enlist or desert, flee.

**Fidelity:** every settlement is simulated in full through v1; aggregate
settlements arrive in M9.

### 5.6 Technology

**Authored graph:** individual technologies, each with prerequisites and a
**content footprint** (goods, recipes, programs, service rungs, style
primitives it unlocks). **There are no era gates.**

**Discovery:**

- Chance per year is driven by population and specialization (skilled
  practitioners in the relevant field), literacy, surplus, and institutions:
  patronage, guild secrecy, censorship, corruption drag.
- "Happy accidents" come from practice.
- Discoveries are tied to real people: *who* discovered it is recorded.

**Diffusion:** through contact (trade, migration, conquest, captured experts),
with adoption filtered by norms and law. A society can know something and
forbid it.

**Engineering techniques** are technology nodes too: lime mortar, the arch and
centering, the timber truss, pile foundations, and so on. Each raises the
structural capacity or maximum span for certain materials (§5.1).

**Size:** the v1 graph has about 150–250 nodes, from early agrarian to
pre-industrial. **Rule:** a node only enters the graph together with its
content footprint. This is the main defense against the content multiplier
(§8).

---

## 6. Art & rendering pipeline

- **Target fidelity: grounded realism.**
  - Lumen global illumination.
  - Nanite for building kits, rocks and props.
  - Virtual shadow maps.
  - Scanned materials (Fab/Megascans).
  - **Weathering driven by `BuildingSpec.condition` and age** through
    per-instance custom data: dirt, moss, cracks, sagging roofs.
  - 60 fps at 1440p with DLSS/TSR on the 4070 Ti.
- **Procedural buildings:** Rust `civ-grammar` → module placements → UE
  `BuildingAssembler` (C++) → ISM/HISM batches of **Nanite kit meshes**.
  Nanite supports ISM/HISM, so runtime assembly keeps Nanite. Per-instance
  data carries colour variation, weathering and window light state.
- **Fake interiors:** an interior-mapping material on window modules. Room
  atlases are chosen by building program. Lights come on from kernel occupancy
  (night shifts and curfews become visible).
- **Kit authoring (recommended: free tools first):**
  - Blender with Geometry Nodes for module variants, plus Fab/Megascans
    materials.
  - A text **kit manifest** in `art/` (socket sizes, bay widths, floor height,
    roof pitch families, style tags) is read by both `civ-grammar` and the UE
    import script, so the grammar and the meshes can't drift apart.
  - UE Python scripts import, assign materials, enable Nanite, and generate the
    module catalog DataAsset.
- **Houdini: not in v1.** Its Unreal outputs must be baked for packaged builds,
  so it could only ever do *offline kit production*, which Blender Geometry
  Nodes already covers. Revisit only if producing kit variants becomes the
  bottleneck.
- **UE PCG:** runtime generation (`GenerateAtRuntime`) for *dressing*:
  vegetation, crops by season, fences, rubble, market clutter, street
  furniture. It reads kernel masks for land use, fertility and condition.
  Buildings stay with the grammar, so they match the kernel's derived facts
  and the web observer.
- **Terrain:** Rust `civ-world` generates it: heightfield with hydraulic
  erosion, river network, climate bands, soils, deposits.
  - Default world is 16 × 16 km: a 2 m heightfield (8192²) and 8 m sim land
    cells (2048²).
  - After generation, terrain changes **only through the people's earthworks**
    (§5.1). These are small local patches: the kernel owns the heightfield and
    streams each edited patch to UE as a delta. The player never edits
    terrain.
  - **How UE renders runtime-generated terrain in a packaged build is
    unresolved** (spike S1, §7). UE Landscape editing APIs are editor-centric;
    Virtual Heightfield Mesh and Mesh Terrain are experimental.
- **Environment:**
  - Day/night in M2: sky atmosphere, lit windows.
  - Seasons in M3: foliage tint, snow cover, frozen water, crop stages.
  - Weather in M6: rain and wetness, storms, drought haze. It feeds fire, crop
    and flood models.
- **Crowd:** a small set of base bodies with clothing variation by wealth,
  occupation and culture palette. Instanced vertex-animation (AnimToTexture)
  at mid distance; skeletal meshes near the camera; one full rig for the
  follow cam.
- **Asset needs (v1):**
  - Kit families: timber, wattle, thatch and earth vernacular (M2–M3); stone,
    brick, tile and plaster (M4–M8); fortification kit (M5).
  - Terrain materials and vegetation (free Megascans/Fab).
  - Crowd characters and roughly 15 animation loops: walk, carry, haul, lift,
    farm, hammer, sit, talk, fight, flee.
  - Construction-site kit: scaffolding, ladders, centering frames, material
    piles, a treadwheel crane.
  - Pre-fractured versions of kit modules for cosmetic collapse debris.
  - Props: carts, stalls, tools, boats.
  - Budget: Fab packs only where free content doesn't cover it.

---

## 7. Milestones

Every milestone is a vertical slice meeting the **usable bar**:

- Launches from one command or exe.
- New-world dialog.
- Save, load and autosave.
- Crash recovery.
- Panels have empty, loading and error states.
- Smoke seeds pass.
- A demo recording.

Sizes are rough estimates for solo work with AI agents. **Recalibrate after
M1.**

| # | Name | Rough size | Observer |
|---|---|---|---|
| M0 | Foundations | 1–2 wks | Web |
| M1 | A band settles | 3–5 wks | Web |
| M2 | First light in Unreal | 5–7 wks | UE + panels |
| M3 | Village economy | 6–8 wks | UE |
| M4 | Councils, law & crime | 6–8 wks | UE |
| M5 | Neighbors | 4–6 wks | UE |
| M6 | Towns & their troubles | 6–8 wks | UE |
| M7 | Politics deepens | 6–8 wks | UE |
| M8 | The growing city (v1) | 8–12 wks | UE |

**M0: Foundations.**
- *Contents:*
  - The repo, CLAUDE.md/AGENTS.md, and the `engine-commons` repo with
    `commons-persist` and `commons-wire` v0.
  - Kernel skeleton: tables, tick scheduler, snapshots.
  - `civ-world` terrain and rivers from a seed.
  - `civ-host` CLI with a WebSocket host; a web shell with a PixiJS map.
  - Windows CI on GitHub Actions: `cargo test`, clippy, and the content
    validator.
- *Demo:* generate a world from a seed and pan it in the browser. Save/load
  round-trips with nothing lost.
- *Proves:* toolchain, schema, persistence.
- *Defers:* all simulation.
- *Outcome (2026-10-03):* implemented. One command (`tools/run.ps1` or `tools/run.sh`) builds
  and opens the observer. It covers the new-world dialog, the map, save, load, autosave, crash
  recovery, and panels with empty, loading and error states. The smoke seeds pass, and the demo is
  recorded in `assets/m0/`. Deviations:
  - `engine-commons` is staged in-repo rather than in its own repository (§3.3).
  - With no people yet, the M0 smoke seeds check terrain and water against fixed thresholds,
    the exact save round trip, and 50 in-game years of clock (§4.7).

**M1: A band settles.**
- *Contents:*
  - One settlement, 20–50 agents.
  - Needs, daily schedules, event-scheduled trips.
  - Births, aging, deaths.
  - Foraging and farming → food; wood → **one building type (hut)** through a
    minimal grammar (footprint, height, material); plot choice.
  - Path wear → the first road graph.
  - Time controls (Detailed mode only).
  - Agent inspector: needs, schedule, family, memory. A chronicle v0.
  - God tool: spawn a family.
- *Demo:* watch a band found a village over 10 years, then save and reload.
- *Proves:* the agent loop, the daily-life time model, the observer UX, and
  the spec → derived-geometry path.
- *Defers:* UE, money, firms, government, services, other settlements.
- *Outcome (2026-10-03):* implemented, in seven slices (A–G; the README describes each). A
  founding band of about 40 settles a valley. It forages, farms emmer, builds huts, pairs, bears
  children and dies, and wears trails. The observer can send a family and run ahead. The demo,
  `assets/m1/m1-demo.webm`, creates a world, follows its band for ten years, then saves and loads
  it back. The usable bar holds: one command, the new-world dialog, save, load, autosave, crash
  recovery, panel states, the smoke seeds (30 days on every push, ten years nightly) and the
  recording. Deviations:
  - The first road graph is the traced trails: lines from end to end and junction to junction,
    where walking wore the ground. Nobody builds or keeps a road yet.
  - The inspector shows needs, the current step and when it ends, family, the household's stores
    and the reasons for each choice. People choose each activity when the last ends, with no day
    plan, so there is no schedule to show. What a settlement knows of places shapes every
    gathering choice but shows only through its reasons.
  - Time controls add running ahead at full detail, as fast as the machine allows; the
    Accelerated mode stays in M3.
  - The smoke seeds check ten years rather than §4.7's fifty, with the checks that apply to a
    village (§9).
  - Land state that changes over time has its own crate, `civ-land`, which the layout did not
    list (§9).
- *Recalibration (2026-10-03):* M0 and M1 were each built by an agent in about a day. From M2
  the work needs Unreal Engine on the Windows PC, which the cloud sessions do not have, so the
  later sizes stand until that work starts.

**M2: First light in Unreal.**
- *Contents:*
  - **Spike S1, runtime terrain** (first two weeks), deciding among:
    1. Tiled runtime mesh terrain (dynamic or procedural mesh with CPU LOD,
       runtime virtual texture material).
    2. A fixed-size Landscape whose heightmap is written at world load, if the
       runtime APIs work in a packaged build.
    3. Virtual Heightfield Mesh or Mesh Terrain (experimental).

    Criteria:
    - Works in a packaged Win64 build.
    - 60 fps over 16 km².
    - Applies small local edit patches streamed from the kernel (earthworks:
      leveling, cut-and-fill, pits, ditches, terraces) without a visible hitch.
    - PCG and foliage work on it.

    Player-scale reshaping is not a criterion.
  - `EngineBridge` plugin in commons: DLL load, sim threads, triple buffer,
    trip interpolation.
  - `BuildingAssembler` with the first vernacular kit.
  - **Visible construction:** pieces appear stage by stage as labor is spent;
    scaffolding; workers hauling materials.
  - Crowd v0; day/night; UMG HUD with time controls.
  - M1 web panels embedded through the WebBrowser widget.
  - Git LFS for `.uasset`, `.umap`, `.blend`, `.fbx` and texture sources,
    configured before the first binary asset is committed.
  - Packaged Win64 build.
- *Demo:* the M1 village in Unreal at 60 fps, with a follow cam, at night.
- *Proves:* the in-process boundary, runtime assembly with Nanite, the shared
  UE layer, the panel embedding.
- *Defers:* seasons and weather visuals, fake interiors, style variety.

**M3: Village economy.**
- *Contents:*
  - 200–500 agents.
  - Goods and recipes for the §5.3 loop; households.
  - Barter, then commodity money.
  - Property regime v0 (communal vs private plots).
  - **Full firms** with books.
  - Posted-price markets.
  - Tech graph v0 (about 30 nodes) with discovery.
  - Grammar v2: houses, workshops, storehouses; 1–2 storeys.
  - **Structural rules v0:** storeys vs. wall material, builder skill,
    foundation on a slope, age and upkeep. Partial failures and collapses show
    up in the chronicle. Technique trust per culture.
  - **Style vector v0** with prestige copying.
  - Seasons in sim and visuals.
  - **Earthworks v1:** plot leveling, quarry and clay pits, irrigation
    ditches, farm terraces.
  - Accelerated mode (daily step) with the mode-consistency test.
  - God tools: introduce a technology, place a deposit.
  - Firm and market panels.
- *Demo:* two seeds diverge: one communal and one private-plot village, with
  visibly different prices, buildings and wealth distribution.
- *Proves:* the economy loop, firm legibility, divergence from primitives.
- *Defers:* government, law, other settlements.

**M4: Councils, law & crime.**
- *Contents:*
  - 1–2k agents.
  - **Notables and the rule-based `Deliberator`.**
  - Constitution primitives v0: offices, the selection methods except
    elections with parties, bodies, powers; label inference.
  - Policy library v0: tribute/tax, land tenure, levy, curfew, prohibitions,
    punishments.
  - Law pipeline stages 1–7.
  - Factions, grievance → protest → riot → coup or revolt, with a
    constitution rebuilt by the winners.
  - **Crime, policing & justice v1** (no courts).
  - Corruption v0.
  - God tools: whisper, bless/curse, ideology or agitator.
- *Demo:* same start, five seeds, at least two different regimes after 40
  years, and one law's full history is inspectable.
- *Proves:* composable institutions yield divergent, legible societies.
- *Defers:* elections with parties, courts, lobbying, multiple polities.

**M5: Neighbors.**
- *Contents:*
  - 2–3 settlements: world-setup founding groups plus **splinter founding**.
  - Inter-settlement roads, merchants and caravans.
  - Per-polity currencies once minting exists.
  - Diffusion of technology and style through contact.
  - Diplomacy states and treaties with ratification.
  - Tributary relations.
  - Fortification kit.
  - **Bridges** on the new roads, with span and load limits per technique
    (timber beam, rope, truss; the stone arch once discovered).
  - God tool: migration wave.
- *Demo:* two settlements trade, their prices converge, one adopts the
  other's roof style, and a treaty fails ratification.
- *Proves:* multi-settlement simulation within budget; diplomacy through each
  side's own institutions.
- *Defers:* war and sovereignty changes.

**M6: Towns & their troubles.**
- *Contents:*
  - Operational war; conquest/annexation, vassalage, secession,
    merger/federation.
  - **Water, sewage & disease.**
  - **Fire.**
  - Weather (visuals and sim).
  - Fake interiors.
  - Incident panels, coverage maps.
  - Stone/brick/tile kit.
  - Levees and embankments as public-works earthworks, which people can
    propose after floods.
  - Floods, storms and fire damage feed structural demand; a collapse can
    prompt building-code proposals.
  - God tools: fire, plague, flood, drought, storm.
- *Demo:* a cholera outbreak traced to a well; a great fire leads to a
  proposed masonry code; a siege ends in annexation.
- *Proves:* the incident-plus-aggregate service UX; war with real
  consequences.
- *Defers:* courts, elections, the full transport system.

**M7: Politics deepens.**
- *Contents:*
  - Elections with franchise rules; parties formed from factions; campaigns;
    coalitions.
  - Full corruption and patronage.
  - Lobbying and capture by firms and guilds.
  - **Courts and precedent**, and judicial review as a check.
  - Credit and banking; coin debasement and inflation.
- *Demo:* a republic's assembly captured by a merchant guild passes
  protectionist tariffs; a court strikes down a curfew.
- *Proves:* the deep political layer on top of the M4 primitives.
- *Defers:* planning institutions, transport.

**M8: The growing city (v1).**
- *Contents:*
  - **Planning as politics**: planning institution, plan templates, zoning,
    land value.
  - **Transport & traffic**: road hierarchy, congestion, commuting, boats.
  - Hierarchical pathfinding.
  - Grammar v3: 3–4 storeys, monuments and landmark templates, style drift
    fully live.
  - Crowd LOD at scale.
  - Road cut-and-fill and grading for the road hierarchy (earthworks).
  - Earthquakes act through structural margins; taller buildings and longer
    spans need the matching techniques.
  - Earthquake god tool (damages buildings; terrain unchanged).
  - **Scale gate: 10–50k agents at 60 fps, with Accelerated Max at ≥1 year per
    minute.**
- *Demo:* a 30k-person planned capital next to an organic port town, with
  traffic, in the same world.
- *Proves:* v1 is complete.
- *Defers:* everything in the §1 NOT list.

**Post-v1 (ordered, not sized):**
- **M9, Aggregate LOD & scale:** cohort model, 10–30 settlements, 100k+
  agents; **decide time skips**; climate change god tool.
- **M10, Industrial content:** steam, factories, rail, gas light; capability
  rungs for industrial services; healthcare and education as full services;
  fiat money.
- **M11, Modern content:** electricity and a power grid, motor vehicles,
  transit, telecom, modern architecture primitives, waste.
- **M12, Optional LLM `Deliberator`, multi-viewer, close-up fidelity**
  (MetaHuman-grade notables).

---

## 8. Risks and open questions

### Technical risks

1. **Runtime terrain in a packaged UE build.**
   - *Risk:* UE Landscape editing APIs are editor-centric, and the alternatives
     are experimental. This could force either an editor-time world-bake step
     (bad UX) or a custom terrain renderer.
   - *Mitigation:* spike S1 opens M2, so it's settled before anything else in
     UE depends on it.
2. **In-process Rust DLL.**
   - *Risk:* a UE crash takes the sim with it; a panic across FFI is undefined
     behavior; the DLL can't be hot-reloaded under Live Coding.
   - *Mitigation:* `catch_unwind` at every exported function; crash snapshots;
     autosave; the headless CLI as the primary sim dev loop; the
     transport-agnostic schema as the escape hatch to go out of process.
3. **Routing and traffic at 50k agents with visible daily life.**
   - *Mitigation:* event-scheduled trips, hierarchical pathfinding with route
     caching per origin zone, time-bucketed congestion, and aggregate flow in
     Accelerated mode.
4. **Crowd rendering of 5–10k visible agents on 12 GB VRAM**, alongside Lumen
   and Nanite.
   - *Mitigation:* vertex-animation instancing at mid distance, impostors far
     away, and a hard visible-agent budget.
5. **Procedural buildings looking mass-produced.** This is the realism
   bottleneck, and it's art, not code.
   - *Mitigation:* budget real time for kit quality, weathering and
     per-instance variation; keep module sets small but excellent.
6. **Divergence between Detailed and Accelerated modes.**
   - *Mitigation:* shared decision functions and the mode-consistency test.
7. **Web panels in UE (CEF):** performance, input focus and packaging quirks.
   - *Mitigation:* prove it in M2, with a fallback to UMG for the highest-traffic
     panels.
8. **Parallel update bugs.** Without determinism, race-driven bugs won't
   reproduce exactly.
   - *Mitigation:* double-buffered state for what agents read, snapshots
     before failures, tracing, and statistical regression tests.

### Scope risks (most likely to balloon)

1. **The technology content multiplier.** Every node implies goods, recipes,
   programs, kit modules and service rungs. This is the biggest threat to "full
   tech progression". *Mitigation:* the "node only with its footprint" rule, and
   v1 stops at pre-industrial.
2. **Composable constitutions and the law pipeline.** The combinatorics are
   large, and debugging them is hard. *Mitigation:* label inference and "why"
   views are built alongside the system, not after it.
3. **Economic stability.** Agent-based economies routinely collapse or blow up.
   *Mitigation:* the sanity thresholds, plus rule 3's time-box and nudges.
4. **Research drift (the TGE failure mode).**
   - *Early warning signs:* milestones ending in reports; ADR counts climbing;
     smoke seeds turning into campaigns.
   - *Mitigation:* rules 2, 3, 4 and 7 in §1.
5. **War.** It touches economy, demography, politics and rendering at once.
   *Mitigation:* operational only, with statistical battles.
6. **UX polish on data-dense panels** takes longer than expected for every
   system. Each milestone's estimate already includes its panels.
7. **Collapse tuning.** Too frequent and it's irritating; too rare and nobody
   notices. *Mitigation:* a band for structural failures per 1,000 buildings
   per year on the sanity dashboard (§4.7), tuned by eye. Failures should be
   concentrated on new techniques, unskilled builders and neglected upkeep.

### Open questions

1. **Time skips:** keep or drop? Decide in M9, once the aggregate model exists.
2. **LLM backend:** local vs API, which model, and cost limits. Deferred to M12.
3. **"Art style" beyond architecture:** flags, emblems, clothing, painted and
   sculpted art objects. These are probably generated from the culture palette
   and symbol primitives, but the scope and milestone are unset.
4. **Religion and belief systems:** a set of institution primitives (doctrines,
   clergy offices, temples as property holders)? Not covered in the interview.
   Currently only "sacred↔secular" values and temples as property holders.
5. **Presenting dark content:** bonded labor, massacres, persecution. It's
   realistic and emergent, but how the UI labels and surfaces it is undecided.
6. **`engine-commons`:** the final name, and when Prometheus adopts
   `EngineBridge` (its V1 is the natural moment).
7. **World shape:** coasts and oceans, multiple biomes per map, map sizes
   beyond 16 km.
8. **When healthcare and education become full services** (currently M10).
9. **Player terrain editing (future consideration, not planned):** a god tool
   to reshape terrain. Revisit after v1, building on the earthworks edit path
   that S1 and M3 establish.
10. **Real structural analysis (future consideration, not planned):** a load
    solver over real structural members (beams, columns, walls with sizes),
    run only on events such as completion or damage. People would choose
    spans and beam sizes and could get them wrong. Revisit after v1 if the
    rule-based margins feel too coarse.
11. **Terrain grid artifacts (from M0):** erosion follows D8 directions, which leaves
    occasional straight valleys and creases visible in close hillshade, and serrated
    valley-floor edges. A multiple-flow-direction or D∞ erosion step is the likely fix. Revisit
    when terrain is first seen up close in Unreal (M2). Closed basins also need Fill–Spill–Merge
    once arid presets exist (M3).
12. **Movement streaming (M1–M2):** research 01-07 warns that trip start and end events alone
    are not enough once congestion, interruptions or acceleration change a trip. Plan movement
    descriptions on paths with corrections from the start, not as a retrofit.

---

## 9. Decisions log

One line each. Don't re-litigate without a reason written next to the entry.

- **Purpose:** personal/research project built to product polish (UX, save/load, stability); no game-design restrictions.
- **One-sentence vision:** as realistic as possible; people make their own society, laws and world; scripted as little as possible; must not become a long research project like TGE.
- **Team:** solo with AI agents doing most of the implementation, including UE work through code and computer control.
- **Budget:** Houdini and Fab allowed, but free or cheap is preferred when the downside is small. No LLM API spend planned.
- **Timeline:** open-ended, but usable early and extended incrementally; first usable build at M1.
- **Hardware:** develop and run on the Windows PC (i9-13th gen, 4070 Ti 12 GB, 64 GB); plan on the Mac.
- **Platform:** Windows only.
- **Performance target:** 60 fps at 1440p with DLSS/TSR allowed.
- **Player role:** observer with god tools; no direct building, zoning or law-making.
- **God tools:** disasters and events, world edits (resource deposits, climate), injecting people and ideas, nudging individuals, all recorded as inputs.
- **Physical building (2026-09-27):** construction is visible and staged (M2), and structures fail by rules for structural margin, with technique trust per culture (M3, then M5, M6 and M8). Collapse debris is cosmetic Chaos only. A real structural solver is a future consideration; physics-based building is out of scope (Prometheus's territory).
- **Terrain editing (2026-09-27):** the people terraform a little through bounded earthworks (leveling, cut-and-fill, pits, ditches, terraces, small levees). The player can't reshape terrain; that's a future consideration only.
- **Reuse:** new code reusing Genesis/Prometheus patterns; engine-agnostic code shared through `engine-commons`, pinned by tag.
- **Technology:** full progression from an authored technology graph with no era gates; discovery and diffusion driven by each society's institutions.
- **Default start:** early agrarian; start point configurable as a known-technology set.
- **Time:** endless simulation, detailed over decades, with speed controls.
- **Time skips:** undecided; deferred to M9.
- **Scale:** largest city 10–50k at first, growing with optimization. Settlements: start with 2–3 and grow later.
- **Cognition:** two tiers, every citizen on utility AI plus about 1% (≤ 500) notables with a deliberation layer.
- **LLMs:** a pluggable `Deliberator`, off by default.
- **Spatial LOD:** full simulation everywhere now; aggregate cohort model later (M9), shared with time skips.
- **1x speed:** daily life visible (1 in-game day per about 15 min); Accelerated mode resolves days statistically.
- **Buildings:** all procedural (a Rust grammar plus a Nanite kit assembled at runtime), including monuments; no hand-authored landmarks.
- **Interiors:** fake (interior mapping plus occupancy lights); building data in the UI; no enterable interiors.
- **Layout:** terrain, resources and population pressure drive growth; organic vs planned is a political outcome.
- **Architectural style:** emergent from real-world vernacular primitives through drifting culture taste vectors.
- **Government:** composable primitives (offices, selection, powers, bodies, checks); regime names are inferred labels.
- **Law:** the full staged pipeline (proposal → enforcement → compliance); courts in M7.
- **Politics:** factions, unrest, corruption, elections and lobbying all wanted; build order M4, then M7.
- **Economy:** composable property, allocation and labor regimes; the "-ism" is an inferred label.
- **Money:** discovered (barter → commodity → coin → credit → paper → fiat); per-polity currencies.
- **Companies:** full firms with books, lifecycle and inspectable pages.
- **Services:** all four realistic in their first milestone (water/sewage/disease, crime/policing/justice, fire, transport); others abstracted; defined by function with technology capability ladders.
- **Service view:** incidents as clickable sim entities, with aggregates computed from them.
- **Diplomacy:** war, peace, trade, alliance, conquest/annexation, merger/federation, secession, vassalage/tribute.
- **War:** operational (visible armies, statistical battles).
- **UE experience:** limited, not treated as a constraint; UE logic in C++ and Python, not Blueprints.
- **Fidelity:** grounded realism (Lumen, Nanite, scanned materials, crowd-grade citizens).
- **Environment:** day/night, seasons and weather all in early milestones (M2, M3, M6).
- **Engine:** Unreal, not Unity (2026-09-27).
  - Cities: Skylines 2 uses Unity for DOTS, which runs its simulation inside the engine. Here the simulation is Rust, so that advantage doesn't apply.
  - A city built at runtime with day/night needs fully dynamic GI and virtualized geometry (Lumen and Nanite). Unity's GI options (Enlighten, lightmaps, Adaptive Probe Volumes) need an editor bake.
  - Prometheus is also on UE, so the two can share `EngineBridge`.
  - Revisit only if S1 fails *and* the fidelity target drops to stylized.
- **Sim/render:** Rust kernel as a DLL inside UE on its own threads, *chosen over my out-of-process recommendation*; also builds as a headless CLI.
- **Boundary:** one versioned, transport-agnostic schema (FFI, WebSocket, recording file).
- **Multiplayer:** single-player; multi-viewer later.
- **UI:** hybrid, UMG HUD plus web panels in TypeScript embedded in UE; the same panels run standalone.
- **First build:** web observer first (M0–M1); UE from M2.
- *(Recommended, not asked)* **Buildings pipeline:** kernel stores `BuildingSpec`; `civ-grammar` expands it on the UE side through the same DLL.
- *(Recommended, not asked)* **Houdini:** skipped for v1, since its outputs must be baked for packaged builds; Blender Geometry Nodes for kit variants.
- *(Recommended, not asked)* **PCG:** runtime dressing only, not buildings.
- *(Recommended, not asked)* **Schema format:** FlatBuffers, unless Genesis's `sim-protocol` generalizes better (decided in M0).
- **Determinism (2026-09-27): not a goal.** Every world plays out differently, even from the same seed. Saves are snapshots; there's no exact replay, input log, or cross-platform float constraint. Parallelism is throughput-first.
- *(Recommended, not asked)* **Anti-research guardrails:** usable-milestone bar, time-boxed emergence with `NUDGE:` entries, 5-seed smoke dashboard, cap of about 3 ADRs per milestone.
- **Boundary schema (2026-10-03, ADR-0001):** a `commons-wire` envelope (56-byte header; frame kinds with delivery contracts; world epochs) carrying FlatBuffers payloads. Genesis's envelope pattern is adopted; its hand-written, engine-specific bodies are not.
- **Saves (2026-10-03, ADR-0002):** the `commons-persist` chunked container with FlatBuffers sections. Refusal, never repair; every save is a new immutable generation; save → load → save must reproduce every section's digest.
- **`engine-commons` location (2026-10-03):** staged in this repository under `commons/` as its own workspace until a second engine adopts it, then split into `ctadros1/Engine-Commons` and pinned by tag.
- **Content format (2026-10-03):** TOML only; unknown and missing fields are errors. RON is not used.
- **World sizes (2026-10-03):** 8 m cells; new worlds are 2, 4, 8 or 16 km a side, 16 km by default (§6). A world starts paused on 1 March, year 1, at 06:00.
- **Speeds (2026-10-03):** 1x is 96 simulated seconds per real second (a day per 15 minutes). Until agents exist, the speeds offered are Pause, 1x, 3x and 10x.
- **Observer stack (2026-10-03):** the web shell is plain TypeScript (no UI framework) with Vite and PixiJS 8. `civ-host` serves the shell and the socket from one origin, on 127.0.0.1:7420. The shell exposes `window.__TCE__` test hooks.
- **Autosave and recovery (2026-10-03):** autosave every in-game month or 5 real minutes while the world has changed, and also before a world is replaced and at exit. The 5 newest are kept. A session marker doubles as the last-good-save pointer. Crash snapshots are kept for diagnosis and never offered for recovery.
- **M0 smoke seeds (2026-10-03):** 2 presets × 5 seeds at 8 km, checking fixed thresholds for relief, land, gentle ground and rivers, the exact save round trip, and 50 years of clock. The population checks of §4.7 join once agents exist.
- **Generated code (2026-10-03):** flatc is pinned at 25.12.19 and the generated Rust and TypeScript are committed. CI rebuilds flatc and fails if they are stale.
- **People, movement and history (2026-10-03, ADR-0003):** people and households are plain tables with permanent ids; activities are versioned so stale events do nothing; needs are lazy; randomness is keyed by purpose, person and a saved counter. Trips are routes with cumulative timings and revisions; M1's snapshot carries one entry per person and trip geometry is queried. Person records and chronicle events are kept forever, decision receipts in a ring of 64 per person.
- **Buildings, land and paths (2026-10-03, ADR-0004):** `BuildingSpec` stores the realised design (footprint in centimetres, authoritative) and `civ-grammar` expands it purely, with golden hashes and a grammar version. Habitat patches (128 m) and their stocks are saved; fields and plots are entities; path wear is saved per 8 m cell with a trail bit, and trail polylines are derived.
- **`civ-land` (2026-10-03):** a crate the original layout did not list, for land state that changes over time (stocks, fields, plots, wear, settlements). `civ-world` stays pure world generation. Dependency order: `civ-core` ← `civ-world` ← `civ-grammar` ← `civ-land` ← `civ-agents` ← `civ-content` ← `civ-sim` ← `civ-host`.
- **Founding band (2026-10-03):** a new world starts with one founding group of 40 people (30–50 allowed) in at least six unrelated families, with ages, unions and children drawn from the life table. Research 05-01's viability estimate: bands of 20 fall below 10 people in about a quarter of five-seed runs; bands of 40 almost never do.
- **Hut grammar in M1 (2026-10-03):** the one building program's rules are Rust code with every dimension from content; authored rule graphs wait for grammar v2 (M3).
- **Sleep timing (2026-10-03):** two-process sleep with a gate: sleep is an option only when circadian-weighted pressure exceeds the wake threshold, the circadian weight stays at its daytime value through the evening and rises in the hour before an authored bedtime 3.3 h after sunset (research 04-02, Yetish et al.), and daytime sleep is a 20–90 minute nap. Without the gate everyone slept from an hour after sunset and napped four hours by day.
- **Gathering worth (2026-10-03):** a gathering trip's value saturates with what it brings (half at a quarter of a household-day of food), and a hungry person with nothing to eat at home values it by their hunger. Before this, people rested and played while their stores were empty.
- **Smoke seeds with people (2026-10-03):** the smoke run lives the founding band's first 30 days (settled, never stuck, water at home, homes on walkable ground) instead of 50 years of an empty clock; checks over years arrive with births and deaths.
- **Wire 1.1 and save schema 2 (2026-10-03):** people, settlements, trips, persons and the chronicle on the boundary (additive); land and people sections in saves, with M0 saves migrated as worlds with nobody in them yet.
- **Goods (2026-10-03):** households keep goods in kilograms, each with its own half-life; firewood burns by an authored seasonal rate; the most perishable food is eaten first; food marked `cooked` needs firewood (cooking adds no energy, research 05-02 §1.1); goods marked `shared` (meat) are split among a settlement's households. Fuel must keep, so stores settle exactly for any split of time.
- **Wild resources (2026-10-03):** a land resource grows as a plant (seasonal production, daily loss) or as animals (logistic growth toward habitat capacity, monthly spread toward an even share of capacity). Water resources live in a patch's water area. Hunts take whole animals drawn from the expected count. Gathering trips work aligned, non-overlapping blocks of patches, so worked land is never mistaken for fresh land.
- **What people know of places (2026-10-03):** a settlement's households share what each trip found. The expected return of a place pools its block's equilibrium stock and draws it down over the trip; what was seen counts against that expectation as a gamma prior with one unit of the resource's worth of evidence, fading over the time the resource takes to renew. One empty hunt says little; one stripped patch of plants says a lot.
- **Diminishing value of stores (2026-10-03):** beyond any shortage, a haul of a good is worth `target / (target + stored)` of its worth, counting only that good, so firewood is not piled up without end while fresh food is still gathered while provisions last.
- **Founding provisions (2026-10-03, revised the same day with farming):** the early-farmers band carries a year of provisions: time to dependable production plus one to three months (research 10-01 §2.3; 05-06 §5.4 tests 6–12 months). With 30 days, the band starved by May. Wild food on a valley map feeds only about a dozen foragers (research 03-06 §2.7). *Revised from 240 days:* a band can break only so much ground in its first spring, so its first harvest feeds it for about half a year and dependable production comes with the second; with 240 days every test band was short at its second sowing, sowed less, and ate its seed by the third year.
- **Wire 1.2 and save schema 3 (2026-10-03):** goods in Welcome, stores and loads in PersonInfo, food days and shortage in SettlementBrief (additive). Saves keep stores and loads by good id and record each resource's good and unit; a resource whose good or unit changed restarts at equilibrium. Schema-2 saves load with household and carried food as provisions, land at equilibrium and patch memory forgotten.
- **Slice order (2026-10-03):** farming (slice C) comes before huts (D) and births and deaths (E): without it the band cannot outlive its provisions, so nothing later could be watched for long.
- **Fields and crops (2026-10-03):** a field is a rectangle in integer centimetres (ADR-0004) worked by one household through fallow, prepared, sown and reaped. A crop is content: calendar, seed and yield per hectare, and the person-hours of each task by hand (research 08-02 §3.2). Each field task is an activity people choose like any other, valued by the food it brings for the year ahead and by how close the task's season is to closing (work left over work the household can still do, research 04-02 §1.3). A harvest depends on the ground, the year's weather, when sowing finished, the weeding done and the days the ripe crop stood. Reaping and threshing count as food in a shortage. Woodland can be cleared for fields at extra work (a tuning value; research 03-06 §1.3 expects it).
- **No fertility decline (2026-10-03):** a field keeps the quality of its ground. Research 03-04 warns against a fixed share lost per crop and regained per fallow year (unmanured continuous wheat holds about 1 t/ha); a nutrient budget waits for soils.
- **Planning and seed (2026-10-03):** a household plans to grow a whole year's food at a cautious yield (research 10-01 §2.3: the 10th–30th percentile), as far as its labour can prepare and sow in one season; gathering adds to it. At threshing it first sets aside seed for the area it plans to sow, plus what seed loses in store until sowing (research 08-01 §1.5, 08-02 §10: seed by planned area, never a share of the harvest). The seed to sow the ground it already crops is never eaten (08-02 §10: protect a minimum viable reserve); seed kept beyond that is eaten only by someone who has drawn a quarter of their body's reserve (§2.3: seed can be eaten in a crisis). Without the floor, a band that met a bad year early ate all its seed and never farmed again.
- **Asking for food (2026-10-03):** a household short of food can ask one of its settlement that holds more than twice the days of food it keeps; the giver gives up to what brings the asker to its target and one person can carry. No debt is kept (research 08-11 §5.4: need-based help, distinct from reciprocity, which waits for obligations).
- **Hunger drive (2026-10-03):** hunger counts toward choices up to full hunger plus a day and a half of deficit (a tuning value). Uncapped, a long-hungry household foraged every daylight hour and left fields it held seed for unsown, year after year.
- **Wire 1.3 and save schema 4 (2026-10-03):** crops in Welcome; the harvest in SettlementBrief; a fields revision in the snapshot and a `GetFields` query whose answer carries each field's rectangle, stage, progress, expected grain and its state in words (additive). Saves gain a fields section, field and household targets, and each settlement's harvest; schema-3 saves load with no fields, and their households mark out new ones.
- **Huts as built (2026-10-03, slice D):** a household designs a round post-built hut with wattle-and-daub walls and a thatched roof, sized for its members (6 m² plus 4.8 m² a resident, research 10-06 §2.3), its door toward the hearth. It claims the square its roof covers as a dwelling plot at its home, or on the nearest clear ground within 30 m, and lives there. Work goes stage by stage (foundation, frame, walls, roof, finish); each stage uses its timber, wattle or thatch in proportion to the work and stops where they run out (research 11-04 §5.2). Labour and material figures are content, from research 11-04 §2.2 and §2.6, 11-01 §3 and 11-08 §2.1, with the rest marked as tuning values: about 500 h of building, 1 t of timber and 1.7 t of thatch for a hut of five. Building is an activity like any other, valued for shelter and by how pressing the roof is before 1 November.
- **Materials are goods (2026-10-03):** timber and thatch are goods of purpose `material`, cut from land resources (poles, reeds and tall grass) and kept in a household's store; threshing leaves straw as thatch. Gathering a material is worth what the load brings toward what the household still needs for its roof, valued against what is still missing, so nobody piles up materials and the last missing kilograms are still worth a trip. A stage is not held up by its last half-kilogram per material (scraps).
- **What a roof does (2026-10-03):** a household under its own roof keeps its stores at the sheltered half-life each good names: grain loses about 5 % a year instead of 15 % (research 08-02 §7.1), provisions 10 % instead of 20 % (08-01 §2.2). Whether a household is sheltered is derived from its buildings, never saved. M1 has no cold or exposure: a roof does nothing for people yet.
- **Building urgency (2026-10-03):** how pressing the roof is counts the work left to the roof plus the loads of material still to fetch (1.5 h a load), over half the household's working hours to the deadline (tuning values: fields, food, water and firewood take the rest). Counting all its hours, households left the last of their walls until days before winter.
- **Choosing among acceptable options (2026-10-03):** the softmax samples only options with a positive total, worth more than doing nothing, and takes its temperature from their spread; every option is sampled only when none is positive (research 01-09 §4.3 applies softmax to acceptable alternatives). Sampling every option let unfavourable choices win draws: a hungry village gathered 7–12 t of firewood nobody needed.
- **Gathering stops at a full load (2026-10-03):** a trip works until its load is full or daylight and the authored time run out, whichever comes first.
- **What stands of a seasonal plant (2026-10-03):** a plant-like stock at equilibrium is the periodic solution of its daily growth and loss, not today's production over the loss rate: what stands lags what grows by about the time it lasts, so last summer's reeds still stand in March, and wild plant food lags its season by about a month. A world's stocks start there, and it is what people expect of land they have not worked.
- **Water before dark (2026-10-03):** water is fetched in daylight only, so in the last three hours of light a household whose water will not last until morning treats its shortage as full. A smoke world caught a household busy building into the evening and dry at dawn.
- **Wire 1.4, save schema 5 and content API 4 (2026-10-03):** a buildings revision in the snapshot and a `GetBuildings` query whose answer carries each building's expanded outline, posts, door, plot, stage, progress and state in words (additive). Saves gain `plots` and `builds` sections (designs, stages and work) and building targets; schema-4 saves load with no buildings, and their households begin new homes. Content gains the `building` kind, the `material` purpose, sheltered half-lives, straw, `[build] home_program` and `decision.w_shelter`, so older packs no longer load (content API 4).
- **NUDGE: thatch at reed-poor sites (2026-10-03):** a band camped in woodland far from wet margins (seed 7 on the river valley preset) finds too little reed for nine roofs (about 14 t of thatch), so its huts wait at the roof stage until the harvest's straw and distant reeds come in, the first roof goes on at the end of October and the last ones in the first winter. Bark, turf or plank roofs, and neighbours sharing spare straw, are not modelled; revisit when buildings have more than one material.
- **NUDGE: building times (2026-10-03):** with the labour figures above, five of six test bands put every household under a roof between mid-April and midsummer of their first year; the wattle weaving rate, pole and reed yields, half-lives of stored timber and thatch, and the share of labour reckoned for building are tuning values to revisit when demography gives households changing sizes.
- **Births and deaths (2026-10-03, slice E):** each person faces a daily hazard from the life table (the Hadza Siler fit, research 05-01 §1.2). Hunger multiplies it rather than adding a second copy (05-02 §1.6): 2.63 times at half the body's reserve drawn, as 2.63^(4d²) of the share d drawn, at most 11.6 times (05-02 §2.7). A body near the end of its reserve fails (an exhaustion hazard, a tuning value, since the research gives no starvation timer), and 0.75 % of births kill the mother (05-01 §2.2). Competing hazards name the cause. Fertility is a state machine (05-01 §1.4–1.5). A woman living with her partner conceives with a monthly chance by age (0.10 at the peak, 04-08 §2.3), times a lasting fecundity of her own (lognormal, 05-01 §4.5) and a hunger factor that halves for each third of the reserve drawn, with no threshold (05-02 §4.3). A pregnancy lasts 266 ± 10 days or is lost, by the mother's age, and a mother cannot conceive for 20 ± 7 months after a birth (at least 6), or for two months after her nursing child dies. Birth rates, birth intervals and death rates are set nowhere; they come out of these rules. The life table already includes deaths in childbirth, so the explicit maternal hazard counts some twice (about 4 % lifetime risk for a mother of six); a residual table waits for explicit causes (05-01 §1.3).
- **Couples and households (2026-10-03, slice E):** unpartnered women from 16 and men from 18 look for a partner with a monthly chance (04-08 §2.1: an age schedule with persistent nonmarriage, not a wedding age). Candidates are of the other sex and in the same settlement, the man between 2 years younger and 14 years older, and not kin within two generations (no shared parent or grandparent; this people's rule, not a universal one, 06-01 §1.4). The choice is a softmax over how far each age gap is from the preferred 3 years. Where a couple lives is content (06-01 §1.3): the early farmers set up a household of their own, each partner bringing a member's share of their household's food and seed, and any children of their own. Someone who keeps a household with no other adult is joined there. A household no one is left in passes, with its stores, fields, plot and hut, to the household of the nearest kin of whoever lived there last, or is abandoned. Children with no one of 15 or more left go, with their household, to their nearest adult kin, else to the best-fed household (06-01 §2.3: help goes first through recognized relationships). Unions end only by death or departure; there is no divorce and no second spouse.
- **Founders' families (2026-10-03, slice E):** the founding band's couples are the families' parents, together since about a year before their eldest child was born. A mother whose youngest child is younger than her drawn recovery is nursing it. The others are pregnant with the chance that an open woman's cycle of waiting and nine months' gestation leaves her so (9q/(1 + 9q) for a monthly chance q), at a point of the pregnancy drawn evenly. Every woman draws her fecundity. Slice D saves load with a couple at the head of each household (the two who share a child, else the first woman and man of age who are not close kin), nobody pregnant or nursing.
- **Body reserve (2026-10-03, revised):** what a body can draw on in a shortage is 2,000 kcal per kilogram of body mass, up from 1,000: fat and lean tissue down to about a third of body mass, at roughly 6,000 kcal a kilogram of mixed tissue (a tuning value; 05-02 §5.3 warns against counting every kilogram lost alike). With 1,000, an adult with no food at all reached the end of their reserve in about four weeks; with 2,000 it takes about eight.
- **Sharing kills from surplus (2026-10-03, revised):** a kill is shared out of what the hunter's household can spare. It first keeps what brings its food up to the days it tries to keep, and the rest goes to every household of the settlement, by members (06-01 §2.3 and §3.2: help comes from disposable surplus, after a household's own subsistence). In plenty all of a kill is shared; in hunger a family keeps what it catches. Splitting every kill evenly kept every household equally short, so a famine wore everyone down together.
- **Departures (2026-10-03, slice E):** a household out of food, whose members have drawn on average 30 % of their bodies' reserve and with no crop of its own ripening within 30 days or reaped and waiting to be threshed, gives up and leaves with a 10 % chance a day (tuning values). It answers expected access to food, not hunger alone (05-06 §1.4); a founding can fail and its households withdraw (05-06 §5.2). Its people leave the world, their records say when, and their fields, plot and hut stand abandoned. There are no other settlements for them to go to yet.
- **No rationing (2026-10-03):** households cutting their meals as stores ran low was tried and made famines worse: hungrier members spent their days foraging instead of working the fields that would end the shortage. People eat as hunger asks while there is food; rationing waits for household plans that weigh the next harvest.
- **Re-working cropped ground (2026-10-03, revised):** preparing ground cropped before takes 300 h/ha instead of 600, half the 720 h research 08-02 §3.2 gives to prepare and sow fully hoed ground (§8: full digging and light hoeing are separate techniques, and the difference can dominate a household's capacity). Breaking new ground stays at 1,000 h/ha. With 600 h the spring's labour held two of six test bands to about 10 ha, short of a year's food for forty.
- **Founding provisions (2026-10-03, revised again with slice E):** 545 days, up from 365. Breaking ground by hand, a band crops all it needs only by its third harvest. Once people could die, three of six test bands starved out within five years on a year's provisions (with the values these entries revise). Eighteen months carries a band to its third harvest (05-06 §5.4: budget to the next reliable food supply).
- **Wire 1.5, save schema 6 and content API 5 (2026-10-03):** PersonInfo gains the partner, family notes in words and when someone left; a kin link says whether they left (additive). Saves gain each person's partner, reproductive state (conception, due date, father, whether it will be lost, recovery), fecundity and nursing child; when a person left; the unions; and the chronicle kinds Born, Died, Paired, TakenIn and Left (codes 8–12). Abandoned fields, plots and huts, which belong to no living household, are valid in a save. Content gains `[fertility]`, `[family]`, the hunger and maternal terms of `[mortality]` and the household's leaving rule, so older packs no longer load (content API 5).
- **NUDGE: provisions stand in for herds (2026-10-03):** real colonists drove in sheep, goats and cattle, a living store with its own labour and risks (08-01 §1.4, 08-02 §5). M1 has no livestock, so the band's extra half year of provisions stands in for the herd. Revisit when animals arrive.
- **NUDGE: founding risk (2026-10-03):** with the values above, in five-year runs of six seeds (6 km river valley), four bands grew or held (39–50 people from 40). One lost six of its nine households to departure, and three people to hunger, in its fourth June, after three harvests that fell short of its needs. The band of the woodland seed left the valley in its third spring. Founding failure is plausible (05-06 §5.2), but how often it happens is not calibrated; revisit with a second crop, livestock, preserving and help from other settlements.
- **Worn ground and trails (2026-10-03, slice F, ADR-0004 §4):** each 8 m cell keeps a wear value. A walk across it adds α(1 − w), and unused wear halves in an authored time: the exact solution of research 10-03's dw/dt = αq(1 − w) − βw with walks as instants, so wear cannot leave 0–1. Starting values from 10-03 §2.2's uncalibrated sensitivity ranges: α = 0.01 and a 120-day half-life, so a cell walked about every four days settles at trail wear. A cell becomes trail at 0.3 and stays trail until it fades below 0.15. Wear is kept in sparse 64 × 64-cell tiles and only on land: a trail stops at a river's bank. A trip wears the cells under its route when it ends. Once a month the paths are surveyed: tiles fade, faded tiles are forgotten, the routing view is rebuilt, and the trails are traced into polylines. Tracing uses a morphological closing, Zhang–Suen thinning with Lü and Wang's correction (plain Zhang–Suen eats the stepped lines straight walks leave), and chains from end to end and junction to junction, simplified to a cell. Walking is faster the more worn the ground (`offtrail_factor` up to the full speed, a 40 % saving, the top of 10-03 §2.2's 10–40 %), so used ground attracts more use. The chronicle notes a settlement's first trail out: 150 m or more, passing within 80 m of its hearth.
- **Straight walking (2026-10-03, slice F):** an A* route on the eight-direction grid is pulled straight wherever a straight line is walkable and no slower. The line is sampled every half cell over bilinear ground, with the same slope, water and wear costs. Without it every route ran at 0°, 45° or 90° and the trails formed an eight-pointed star; with it desire lines run in any direction (10-03 §1.1: some exploration across traversable ground is needed for desire lines to appear), and trails form where wear makes a detour pay.
- **Routing on the survey (2026-10-03, slice F):** routes, the route cache and settlements' travel-time fields use the paths as last surveyed, so they agree with each other and with the trails on the map until the next survey; a trail worn this month speeds walking from next month. A loaded world is surveyed when it is put together. Travel-time fields are dense arrays over the box their limit could reach. Without that, monthly fields took half the simulation time, and three years of a 40-person village went from 30.5 s to 41 s; with it they take 31 s.
- **Wire 1.6, save schema 7 and content API 6 (2026-10-03):** `paths_rev` in the snapshot and a `GetPaths` query whose answer carries the surveyed tiles (wear 0–255 and trail bits per cell) and the trail polylines with their wear and length (additive). Saves gain a `wear` section: the tiles as they stood on the save day, as u16 wear and trail bits; the routing view and trails are derived on load. Schema-6 saves load with untrodden ground. The land profile gains `[paths]` (content API 6). Chronicle code 13 is the first trail.
- **The observer sends a family (2026-10-03, slice G):** M1's god tool. A click on the map places a family where people can walk: a couple and their children, drawn as a founding family is (ages from the life table, the couple's years together, a pregnancy or a nursing child), with a founding family's provisions per head. It joins the nearest inhabited settlement whose hearth lies within 600 m (a tuning value) and makes its home where it was placed; with none that near, it makes camp there and founds a settlement of its own. Its people get names nobody living has. The chronicle says the observer sent it, and each person's record says they were brought by the observer (§2: every god-tool use is in the chronicle).
- **NUDGE: a sent family comes provisioned like the founders (2026-10-03):** without stores, a family sent in winter starves before it can sow. Newcomers came with what they could carry or drive and with kin to lean on; the founders' eighteen months, which also stand in for their herds, is the nearest figure the model has. Revisit with livestock and with help between settlements.
- **Running ahead (2026-10-03, slice G):** the observer can run the world ahead by a day, a month, a year, 5 or 10 years at full detail, as fast as the machine allows. The engine thread spends 80 % of each 50 ms tick advancing in steps of up to an hour, and in the rest publishes frames and answers queries, so the map keeps up. It is a task the observer can cancel; the world pauses where it arrives and is autosaved. It is the Detailed mode unpaced, not §4.4's Accelerated mode (M3): there is no daily statistical step and no mode-consistency test yet. On the 4-core cloud CPU, with the map following, the demo's village of 50–60 lived about 15 s a year.
- **Ten-year smoke seeds (2026-10-03, slice G, §4.7):** `smoke --years 10` lives each smoke world for ten years after its first month and checks it at every year's end: nobody stuck between events; no population or land problems; at most three times the founders; from the second year, at least 80 % of households under their own roof; and in the first year, a first trail out. A band may fail (05-06 §5.2), so dying out is checked across the set: at least half the bands must keep 10 or more people. Runs differ, so the thresholds leave room for chance. The ten worlds take about 8 minutes on 4 cores, so they run nightly and on pull requests into `main` (`nightly.yml`); per-push CI keeps the 30-day run. §4.7's other checks (prices, Gini, firm and settlement sizes, crime, epidemics, regimes, structural failures) arrive with the systems they check, and its 5 worlds × 50 years with the Accelerated mode.
- **NUDGE: founding risk over ten years (2026-10-03):** in two ten-year runs of the ten smoke worlds, eight bands lived on with 29–61 people (from 40), and both bands of seed 1 failed both times (in the second run, in their sixth and eighth years). In the failure examined closely (river valley, seed 1), a famine in the sixth spring sent nine households away within three months: asking for food evens out stores, so households run out together. Founding failure is plausible (05-06 §5.2), but its rate is not calibrated. Revisit with livestock, preserving, a second crop, households leaving to found a daughter settlement, and help between settlements.
- **Wire 1.7 and save schema 8 (2026-10-03):** `SpawnFamily { at }` and `RunUntil { minute }` commands (additive). Saves gain chronicle code 14, a family the observer sent; schema-7 saves load as they are.
