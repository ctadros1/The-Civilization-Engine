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

**Milestone M1, First settlers, is in progress** ([plan §7](PROJECT_PLAN.md#7-milestones)). Its first four slices are implemented:

- **Slice A:** a new world begins with a founding band of 30–50 people in at least six families, who choose where to camp. They sleep, eat, fetch water, play and sit at the hearth, each choice scored from their needs and sampled, and they walk routed paths over the terrain. The map shows them moving. Click anyone to see what they are doing, their needs, their family and **why** they chose it, with every consideration's weight. The chronicle records the band's arrival.
- **Slice B:** people gather wild plants, hunt, fish and collect firewood, and bring home goods that keep or spoil at their own rates. A kill is shared across the camp. Meat and fish need a fire, so firewood matters. Game, fish and plants grow back on their own clocks. Hunting a valley out takes a season and its recovery takes years. Households share what they learn about places and weigh it against what they would expect. The chronicle notes when a settlement's food runs short and when it recovers.
- **Slice C:** households farm. They mark out fields near the settlement on floodplain, or on woodland they must clear first. They break the ground, sow emmer from the seed they brought, weed, reap and thresh by hand. Each task is chosen for the food it brings for the year ahead and for how close its season is to closing. A harvest depends on the field's ground, the year's weather, when sowing finished, the weeding done and how long the ripe crop stood. Before anything is eaten, seed for next season is set aside by the area planned. The seed to sow the ground already cropped is never eaten; seed beyond that is eaten only in real hunger. A household short of food asks one that can spare some. The map shows each field through its year, the readout names the field under the pointer, and the chronicle notes the first sowing and each harvest.

- **Slice D:** households build homes. Each designs a round post-built hut with wattle-and-daub walls and a thatched roof, sized for its members and with its door toward the hearth, and claims the ground its roof covers. People cut poles, rods and reeds and carry them home; threshing leaves straw for thatch. The hut goes up stage by stage (postholes, frame, walls, roof, floor), each stage using its materials as the work goes and waiting when they run out. Building is a choice like any other, and it presses harder as the first winter nears. Under its own roof, a household's grain keeps about three times as long. The map draws each hut as it goes up, from the shape the kernel expands from the saved design; the readout names the hut under the pointer and what it is waiting for; the chronicle notes the settlement's first roof.

**Not yet:** births and deaths, marriage, trails and the god tools. These are the remaining M1 slices.

### Implemented and planned

| Area | Status | What exists |
| --- | --- | --- |
| Kernel foundations (`civ-core`) | Implemented | Generational handles and permanent ids, a 365-day calendar, the event-and-cadence scheduler, small seeded RNGs |
| World generation (`civ-world`) | Implemented: terrain and water | Uplift and stream-power erosion, refinement to 8 m cells, valley floors, lakes from the water balance, river reaches with discharge and width. Climate bands, soils and deposits are planned (M1–M3) |
| Content (`civ-content`, `content/`) | Implemented: world presets, people, land, names, activities, goods, crops, building programs | Strict TOML packs, stable diagnostics, fingerprints; two landscape presets; the early-farmers people profile, the temperate-valley land profile, nineteen activities, nine goods, one crop (emmer) and one building program (the hut). Other primitives arrive with the milestones that need them |
| Building grammar (`civ-grammar`) | Implemented: the hut | A pure expansion of a saved design (round post-built hut, wattle and daub, thatch) into its parts, outline and door and the labour and materials of each construction stage, pinned by golden hashes and a grammar version ([ADR-0004](decisions/0004-buildings-land-paths.md)). Authored rule graphs and more programs arrive with grammar v2 (M3) |
| Land (`civ-land`) | Implemented: habitats, wild plants, game, fish, fallen wood, fields | 128 m habitat patches classified from terrain. Plant-like stocks grow seasonally and waste; animals grow logistically toward their habitat's capacity and spread monthly; hunts take whole animals. Gathering slows as a patch empties (no respawn timers). A yearly climate factor, settlements. Fields are rectangles in centimetres that go through their crop's year; woodland is cleared before it is first broken. Dwelling plots are claimed in centimetres, and each building keeps its design and the work done on the stage under way |
| People (`civ-agents`) | Implemented: M1 slices A to D | A founding band in families with ages from a life table; needs in closed form (energy, sleep pressure, company) with a body reserve; utility choices with softmax sampling and recorded receipts; routed walking (Tobler's hiking function, A*). Households keep goods that spoil by half-life, burn firewood by season, eat the most perishable food first and share kills; they remember what places gave and share it within the settlement ([ADR-0003](decisions/0003-people-movement-history.md)). Households plan fields for a year's food at a cautious yield, choose field work by its harvest and deadline, keep seed by planned area, and ask each other for food when short. They design their huts, claim plots, cut and carry what they build with, and build toward a roof before winter. Choices are sampled among the options worth more than doing nothing. Person records and the chronicle. Births, deaths and marriage are planned (M1) |
| Boundary schema (`commons-wire`, `civ-schema`) | Implemented | Frame envelope and FlatBuffers payloads for Rust and TypeScript, schema TCE 1.4 ([ADR-0001](decisions/0001-boundary-schema.md)) |
| Saves (`commons-persist`, `civ-sim`) | Implemented | Chunked, checksummed generations; verified on write; refusal, never repair ([ADR-0002](decisions/0002-snapshot-container.md)). Schema 2 added land and people, schema 3 goods, schema 4 fields, schema 5 plots and buildings. M0 saves load as worlds with nobody in them yet; slice A saves load with their food as provisions; slice B saves load with no fields, which households then mark out; slice C saves load with no huts, which households then begin |
| Host (`civ-host`) | Implemented | Command line and a localhost WebSocket server with autosave and crash recovery |
| Web observer (`web/`) | Implemented: M0 shell and M1 slices A to D | Map with terrain, rivers and detail tiles; fields coloured by where they are in their year; huts drawn stage by stage as they go up; people moving along their trips; the settlement with its days of food and its harvest; an inspector with needs, family, the household's stores and the reasons for each choice; the chronicle; new-world (with band size), save, load and recovery dialogs; time controls; world facts; event log |
| CI | Implemented | A Windows lane (kernel, commons, content, smoke seeds) plus browser tests and schema freshness |
| Demography, trails, economy, government, services, diplomacy | Planned (M1–M8) | See the [milestones](PROJECT_PLAN.md#7-milestones) |
| Unreal client (`civ-ffi`, `EngineBridge`) | Planned (M2) | |

### Known limitations

- **Terrain.** Erosion routes water along the grid's eight directions, which leaves occasional straight valleys and creases, visible in the hillshade up close. Valley-floor edges can look jagged at the 8 m cell scale.
- **Lakes.** Lakes are rare in the humid presets, and closed basins use a single-lake approximation.
- **Small maps.** The large river can miss a small map, and at 2 km and 4 km the coastal preset can be mostly sea. The smoke seeds use 8 km maps.
- **Performance.** A 16 km world takes about 18 s to generate on a 4-core 2.1 GHz cloud CPU, and its save is about 15 MB. Your machine will differ.
- **WebGL.** The map needs WebGL. Headless browsers fall back to slow software rendering.
- **Hard first years.** Wild food on a valley map feeds only about a dozen foragers (research 03-06 §2.7), not a band of 40, so farming has to feed it. A band can break only so much ground in its first spring, so its first harvest feeds it for about half a year, and it arrives with a year of provisions to bridge the gap (research 10-01 §2.3). In five-year runs of six seeds (with huts, slice D), every band still farmed in its fifth year. Three sowed 12–14 ha and reaped 8–15 t of grain a year from their third year on, with at most one short spell. Two stayed at about 10 ha and 4.5–7.5 t and were short of food for a few weeks before most harvests. One in a valley of woodland that had to be cleared sowed about 6.5 ha, reaped 2–4 t and was short every spring. Nobody dies yet: body reserves bottom out at a floor, and births and deaths arrive in slice E.
- **One crop, one season.** Emmer is sown in spring only; there is no autumn sowing, no pulses and no livestock. The calendar and the extra work of clearing woodland are tuning values, since the research gives none.
- **Fields do not change the land.** A woodland field still counts as woodland for wild food and firewood, and a field keeps the quality of its ground; a nutrient budget waits for soils (research 03-04).
- **One kind of building.** Every household builds the same kind of hut, sized for its members. There are no storehouses, workshops, repairs or second huts; buildings never decay or burn; a plot is never given up. A roof keeps stores but does nothing for people yet: there is no cold or exposure.
- **Thatch can be scarce.** A hut for five takes about 500 hours of building, 1 t of timber and 1.7 t of thatch. A band camped in woodland far from reed beds (seed 7 of the river valley) waits for its harvest's straw and distant reeds: its first roof goes on at the end of October and the rest in the first winter. Elsewhere every household of the test bands is under a roof by midsummer. Households do not share spare straw or other materials, and there are no bark or turf roofs. Weaving wattle, the yields of poles and reeds, and how long stored timber and thatch keep are tuning values.
- **No preserving.** Fresh meat, fish and plant food spoil within days, and there is no drying or smoking yet, so a summer glut cannot be stored for winter.
- **Simulation speed.** A simulated year of a 40-person band takes about 15–45 s on the 4-core cloud CPU. Hungry seasons are the slow part: long hunting trips to new places each need a route planned. A cache keeps it bounded, and routing is revisited with trails in slice F.

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
cargo run --release -p civ-host -- new --seed 11 --size 512 --days 528   # lived to the second harvest
cargo run --release -p civ-host -- save info  ../saves/<world>/g0000000001-manual.tcesave
cargo run --release -p civ-host -- save verify ../saves/<world>/g0000000001-manual.tcesave
cargo run --release -p civ-host -- content validate
cargo run --release -p civ-host -- smoke       # 2 landscapes × 5 seeds against fixed thresholds
```

Tests, lints and the rules for changing schemas, saves and content are in [AGENTS.md](AGENTS.md).

## What the build looks like

**These are screenshots of the current build**, unlike the concept art further down.

<table>
  <tr>
    <td width="50%"><img src="assets/m1/m1-huts-building.jpg" alt="Eight hut plots around the hearth of Hazelford by a river in April: one hut half thatched, the others ringed by posts or postholes, people building; the readout says a hut's frame is going up, 18% done, waiting for timber"><br><strong>M1 slice D: huts going up at Hazelford, 22 April, seed 3.</strong> Each plot is the square its hut's roof will cover. One hut is being thatched; the others show their postholes or posts. The readout names the hut under the pointer: its frame is 18% done and it waits for timber.</td>
    <td width="50%"><img src="assets/m1/m1-huts-roofed.jpg" alt="Eight huts at Hazelford in June, all thatched, each doorway facing the hearth, people asleep inside at dawn"><br><strong>Hazelford in June, from another run of the same seed.</strong> Every household is under a thatched roof, each hut sized for its household with its door toward the hearth. At dawn its people are still asleep inside.</td>
  </tr>
</table>

<p><img src="assets/m1/m1-fields.jpg" alt="Otterholt in its second August: fields around the village by a river, most reaped and brown, a few still green, one pale with sheaves to thresh; the inspector shows a woman threshing grain from her household's field, its stores of grain and seed, and why she chose to thresh"><br><strong>M1 slice C: Otterholt's second harvest, seed 11.</strong> Most fields are reaped (brown), a few late-sown ones still grow (green) and one holds sheaves to thresh (pale). Wren threshes at home; her household keeps 974 kg of grain and 215 kg of seed. The chronicle notes the first sowing, the first harvest of 5,646 kg, and a short spell of hunger before the second.</p>

<p><img src="assets/m1/m1-inspector.jpg" alt="A founding band at their camp by a river; one person is inspected: what she is doing, her needs, her household's food, firewood, water and stores, the reasons for her last choice, her family, and the chronicle"><br><strong>M1 slices A and B: a band of 30 at Hazelford, seed 3.</strong> Dots are people coloured by activity; a firewood party works south of the camp. The inspector shows the household's days of food, firewood and water and what is in store, then the last decision's considerations, the next-best option and what was ruled out.</p>

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
