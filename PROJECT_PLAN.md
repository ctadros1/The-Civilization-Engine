# The Civilization Engine: Project Plan

Status: plan of record, written 2026-09-27 from the planning interview.
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
specific to any one engine. **Admission rule:** code goes in only if at least
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
      civ-core              # ids, tables, tick scheduler, time, RNG
      civ-world             # world-gen: terrain, hydrology, climate, soils, deposits
      civ-agents            # citizens, needs, schedules, utility decisions, demography, culture
      civ-notables          # deliberation layer + Deliberator trait
      civ-tech              # technology graph, discovery, diffusion
      civ-econ              # goods, recipes, property/allocation/labor regimes, firms, markets, money
      civ-polity            # constitutions, offices, policies, law pipeline, factions, unrest
      civ-services          # incidents: water/disease, crime/justice, fire, transport
      civ-diplomacy         # relations, treaties, armies, war, sovereignty changes
      civ-grammar           # building + layout grammars; reads art/kits manifest
      civ-schema            # boundary schema (commons-wire)
      civ-ffi               # cdylib for UE
      civ-host              # headless CLI + WS host
  content/                  # authored primitives (RON/TOML) + validator
  web/                      # TS panels + 2D observer
  unreal/                   # UE project; Plugins/EngineBridge vendored from commons
  art/                      # Blender sources, kit manifests (sockets, sizes, style tags)
  tools/                    # UE Python import pipeline, content checks, smoke-seed runner
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
