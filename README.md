<p align="center"><img src="assets/previews/civilization-engine-mark.svg" width="96" height="96" alt="Civilization Engine mark"></p>
<h1 align="center">The Civilization Engine</h1>
<p align="center"><strong>An endless, emergent simulation of human civilization.</strong></p>
<p align="center">People build settlements, cities, institutions, economies, technologies, and ways of life across history, from the earliest communities through modern civilization.</p>

<p align="center">
  <a href="#vision">Vision</a> · <a href="#project-status">Status</a> · <a href="#getting-started">Getting started</a> · <a href="#what-the-build-looks-like">Screenshots</a> · <a href="#simulation-design">Simulation design</a> · <a href="#explore-the-repository">Explore</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/milestone-M1%20first%20settlers%20(in%20progress)-9a6a36?style=flat-square" alt="Milestone: M1 first settlers, in progress">
  <img src="https://img.shields.io/badge/kernel-Rust-315f55?style=flat-square" alt="Kernel: Rust">
  <img src="https://img.shields.io/badge/observer-web%20(Unreal%20from%20M2)-526c83?style=flat-square" alt="Observer: web, Unreal from M2">
</p>

## Vision

The Civilization Engine is a personal project to build a deep, plausible simulation of people and the societies they create. Individuals make decisions, form households and communities, work, trade, invent, govern, build, migrate, and adapt. Settlements can grow into cities, while institutions and technologies emerge from the actions of their people.

The guiding principle is **the engine authors the vocabulary, never the plot**. The simulation supplies people, needs, resources, institutions, materials, and rules. It does not dictate a storyline or require a society to pass through fixed historical eras. Political and cultural labels are interpretations of what people have built, not switches that set the world into a predetermined state.

## Project status

**Milestone M0, Foundations, is implemented.** You can launch the engine with one command and create a world from a seed in the browser. You can pan and zoom its terrain and rivers, and run the clock. Saving, loading, autosave and crash recovery all work, along with a small command line for worlds, saves and content.

**Milestone M1, First settlers, is in progress** ([plan §7](PROJECT_PLAN.md#7-milestones)). Its first slice is implemented: a new world begins with a founding band of 30–50 people in at least six families, who choose where to camp. They sleep, eat, fetch water, gather wild plants, play and sit at the hearth, each choice scored from their needs and sampled, and they walk routed paths over the terrain. The map shows them moving. Click anyone to see what they are doing, their needs, their family and **why** they chose it, with every consideration's weight. The chronicle records the band's arrival.

**Not yet:** hunting, fishing, wood, fields and grain, huts, births and deaths, marriage, trails and the god tools. These are the remaining M1 slices. Until fishing, hunting and farming exist, a band eats its 30 days of provisions and then cannot gather enough wild plants to feed itself (see the known limitations).

### Implemented and planned

| Area | Status | What exists |
| --- | --- | --- |
| Kernel foundations (`civ-core`) | Implemented | Generational handles and permanent ids, a 365-day calendar, the event-and-cadence scheduler, small seeded RNGs |
| World generation (`civ-world`) | Implemented: terrain and water | Uplift and stream-power erosion, refinement to 8 m cells, valley floors, lakes from the water balance, river reaches with discharge and width. Climate bands, soils and deposits are planned (M1–M3) |
| Content (`civ-content`, `content/`) | Implemented: world presets, people, land, names, activities | Strict TOML packs, stable diagnostics, fingerprints; two landscape presets; the early-farmers people profile, the temperate-valley land profile, seven activities. Other primitives arrive with the milestones that need them |
| Land (`civ-land`) | Implemented: habitats and wild plants | 128 m habitat patches classified from terrain, wild plant stocks that grow, waste and are gathered (no respawn timers), a yearly climate factor, settlements. Game, fish, wood and fields are planned (M1) |
| People (`civ-agents`) | Implemented: M1 slice A | A founding band in families with ages from a life table; needs in closed form (energy, sleep pressure, company); utility choices with softmax sampling and recorded receipts; routed walking (Tobler's hiking function, A*); households with food and water stores; person records and the chronicle ([ADR-0003](decisions/0003-people-movement-history.md)). Births, deaths, marriage and memory are planned (M1) |
| Boundary schema (`commons-wire`, `civ-schema`) | Implemented | Frame envelope and FlatBuffers payloads for Rust and TypeScript ([ADR-0001](decisions/0001-boundary-schema.md)) |
| Saves (`commons-persist`, `civ-sim`) | Implemented | Chunked, checksummed generations; verified on write; refusal, never repair ([ADR-0002](decisions/0002-snapshot-container.md)). Schema 2 adds land and people; M0 saves load as worlds with nobody in them yet |
| Host (`civ-host`) | Implemented | Command line and a localhost WebSocket server with autosave and crash recovery |
| Web observer (`web/`) | Implemented: M0 shell and M1 slice A | Map with terrain, rivers and detail tiles; people moving along their trips; the settlement; an inspector with needs, family and the reasons for each choice; the chronicle; new-world (with band size), save, load and recovery dialogs; time controls; world facts; event log |
| CI | Implemented | A Windows lane (kernel, commons, content, smoke seeds) plus browser tests and schema freshness |
| Farming, huts, demography, trails, economy, government, services, diplomacy | Planned (M1–M8) | See the [milestones](PROJECT_PLAN.md#7-milestones) |
| Unreal client (`civ-ffi`, `EngineBridge`) | Planned (M2) | |

### Known limitations

- **Terrain.** Erosion routes water along the grid's eight directions, which leaves occasional straight valleys and creases, visible in the hillshade up close. Valley-floor edges can look jagged at the 8 m cell scale.
- **Lakes.** Lakes are rare in the humid presets, and closed basins use a single-lake approximation.
- **Small maps.** The large river can miss a small map, and at 2 km and 4 km the coastal preset can be mostly sea. The smoke seeds use 8 km maps.
- **Performance.** A 16 km world takes about 18 s to generate on a 4-core 2.1 GHz cloud CPU, and its save is about 15 MB. Your machine will differ.
- **WebGL.** The map needs WebGL. Headless browsers fall back to slow software rendering.
- **Food (M1 slice A).** Wild plants alone cannot feed a band. A world's whole standing stock in March is about 17 days of a 40-person band's needs. After its 30 days of provisions, the band goes hungry: people gather harder, but nobody dies and body reserves are not yet limited. Hunting, fishing and grain are the next slices.
- **Simulation speed.** A simulated year of a 40-person band takes about a minute on the 4-core cloud CPU. Most of that is route planning, and a cache keeps it bounded.

## Getting started

You need [Rust](https://rustup.rs) through rustup, which installs the pinned toolchain from `rust-toolchain.toml` by itself. You also need Node.js 22 with npm, and a browser with WebGL.

```powershell
tools\run.ps1        # Windows (PowerShell)
```

If PowerShell says running scripts is disabled on this system, run `powershell -ExecutionPolicy Bypass -File tools\run.ps1` instead, or allow local scripts once with `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`.

```sh
tools/run.sh         # Linux and macOS
```

The script builds the web shell if needed, builds and starts `civ-host`, and opens <http://127.0.0.1:7420/>. Choose **Create a world**, pick a landscape, size, seed and the size of the founding band, and the map appears when generation finishes. Use **Fit** and zoom in on the settlement's name to see the people; press **Run** (or Space) and click anyone to inspect them. Saves go to `saves/<world>/` in the repository, one folder per world, and every save is a new file. **Ctrl+C** stops the host and saves the world if it changed. If the host stops unexpectedly, the next start offers to recover the newest intact save.

The command line, run from `kernel/`:

```sh
cargo run --release -p civ-host -- new --seed 7 --size 2048 --name "Old River Valley"
cargo run --release -p civ-host -- save info  ../saves/<world>/g0000000001-manual.tcesave
cargo run --release -p civ-host -- save verify ../saves/<world>/g0000000001-manual.tcesave
cargo run --release -p civ-host -- content validate
cargo run --release -p civ-host -- smoke       # 2 landscapes × 5 seeds against fixed thresholds
```

Tests, lints and the rules for changing schemas, saves and content are in [AGENTS.md](AGENTS.md).

## What the build looks like

**These are screenshots of the current build**, unlike the concept art further down.

<p><img src="assets/m1/m1-inspector.jpg" alt="A founding band at their camp by a river; one person is inspected: what she is doing, her needs, the reasons for her last choice, her family, and the chronicle"><br><strong>M1 slice A: a band of 30 at Hazelford, seed 3.</strong> Dots are people coloured by activity; a gathering party walks out along the river. The inspector shows the last decision's considerations, the next-best option and what was ruled out.</p>

The M0 screenshots below show the terrain alone. Recordings of the end-to-end tests are in [`assets/m0/m0-demo.webm`](assets/m0/m0-demo.webm), which generates, explores, saves and loads a world, and [`assets/m0/m0-recovery.webm`](assets/m0/m0-recovery.webm), which recovers after the host is killed.

<table>
  <tr>
    <td width="50%"><img src="assets/m0/m0-river-valley.jpg" alt="A 16 km river valley world in the web observer: shaded relief, river network, a lake, the world panel and event log"><br><strong>A 16 km river valley from seed 7</strong></td>
    <td width="50%"><img src="assets/m0/m0-ria-coast.jpg" alt="A 16 km coastal world with drowned river valleys"><br><strong>The ria coast preset, seed 3</strong></td>
  </tr>
  <tr>
    <td width="50%"><img src="assets/m0/m0-detail.jpg" alt="Zoomed in to full 8 m resolution with the coordinate readout"><br><strong>Zoomed in to 8 m cells, with the coordinate readout</strong></td>
    <td width="50%"><img src="assets/m0/m0-saves.jpg" alt="The save browser listing manual and autosaves"><br><strong>The save browser</strong></td>
  </tr>
</table>

## Visual previews

**Illustrative concept art, generated for this README.** These images communicate the intended scope across historical and modern settings. They are not screenshots or evidence of implemented simulation features.

<table>
  <tr>
    <td width="50%"><img src="assets/previews/civilization-engine-hero.png" alt="Illustrative concept of a river settlement growing into a historical city"><br><strong>Settlements and cities across history</strong></td>
    <td width="50%"><img src="assets/previews/civilization-engine-modern.png" alt="Illustrative concept of a contemporary metropolis with transit, bridges, and older districts"><br><strong>Modern civilization and urban systems</strong></td>
  </tr>
</table>

## Simulation design

| System | Direction |
| --- | --- |
| People | Individuals with needs, routines, relationships, memory, and bounded decision-making |
| Settlements | Communities that choose where and how to grow, shaping neighborhoods and cities over time |
| Society | Governance, law, culture, religion, education, social groups, and conflict |
| Economy | Production, labor, property, markets, money, firms, trade, and public services |
| Technology | Knowledge and inventions that spread, change, and sometimes disappear |
| World | Terrain, climate, water, ecology, resources, hazards, and infrastructure |
| Observer | A detailed view into people, places, institutions, decisions, and the history they produce |

### Architecture

A data-oriented Rust kernel owns the world's state and its rules. Viewers only show that state and forward the player's requests. In M0 the viewer is a web observer served by a headless host over a versioned, schema-generated boundary. From M2, an Unreal Engine client will render the world through the same boundary. The kernel stays free of any Unreal dependency.

```
content/ (authored vocabulary) ──► civ-content ─┐
civ-core (ids, time, scheduler) ─► civ-world ───┼─► civ-sim (world state, saves, payloads) ─► civ-host ─► web observer
commons-wire, commons-persist (shared with other engines) ┘                                (CLI, WebSocket)   (PixiJS)
```

## Explore the repository

| Start here | For |
| --- | --- |
| [Project plan](PROJECT_PLAN.md) | Vision, scope, architecture, milestones, risks, and decisions |
| [AGENTS.md](AGENTS.md) | How to work in the repository: layout, commands, rules for changes |
| [`decisions/`](decisions/) | Architecture decision records (boundary schema, saves) |
| [`kernel/`](kernel/) | The Rust kernel and the `civ-host` command line and server |
| [`commons/`](commons/) | `engine-commons`, staged here: the frame envelope and the save container |
| [`content/`](content/) | Authored content packs and their format |
| [`web/`](web/) | The web observer |
| [Research index](research/README.md) | Research topics, domains, and milestone priorities |

The research covers simulation engineering, reference games, world generation, people, demography, culture, technology, economics, government, cities, architecture, infrastructure, diplomacy, rendering, observer experience, and validation.

## Project boundaries

This is a personal simulation project rather than an academic research instrument. Realism means plausible behavior checked against selected historical patterns, not proof that the simulation reproduces history. The plan records open questions and decisions as development progresses.

## License

No license has been added yet. Until a license is chosen, reuse and redistribution are not granted by this repository.
