<p align="center"><img src="assets/previews/civilization-engine-mark.svg" width="96" height="96" alt="Civilization Engine mark"></p>
<h1 align="center">The Civilization Engine</h1>
<p align="center"><strong>An endless, emergent simulation of human civilization.</strong></p>
<p align="center">People build settlements, cities, institutions, economies, technologies, and ways of life across history, from the earliest communities through modern civilization.</p>

<p align="center">
  <a href="#vision">Vision</a> · <a href="#project-status">Status</a> · <a href="#getting-started">Getting started</a> · <a href="#what-the-build-looks-like">Screenshots</a> · <a href="#simulation-design">Simulation design</a> · <a href="#explore-the-repository">Explore</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/milestone-M3a%20village%20economy%20(in%20progress)-9a6a36?style=flat-square" alt="Milestone: M3a village economy, in progress">
  <img src="https://img.shields.io/badge/kernel-Rust-315f55?style=flat-square" alt="Kernel: Rust">
  <img src="https://img.shields.io/badge/observer-web%20(Unreal%20from%20M2)-526c83?style=flat-square" alt="Observer: web, Unreal from M2">
</p>

## Vision

The Civilization Engine is a personal project to build a deep, plausible simulation of people and the societies they create. Individuals make decisions, form households and communities, work, trade, invent, govern, build, migrate, and adapt. Settlements can grow into cities, while institutions and technologies emerge from the actions of their people.

The guiding principle is **the engine authors the vocabulary, never the plot**. The simulation supplies people, needs, resources, institutions, materials, and rules. It does not dictate a storyline or require a society to pass through fixed historical eras. Political and cultural labels are interpretations of what people have built, not switches that set the world into a predetermined state.

## Project status

**Milestone M0, Foundations, is implemented.** You can launch the engine with one command and create a world from a seed in the browser. You can pan and zoom its terrain and rivers, and run the clock. Saving, loading, autosave and crash recovery all work, along with a small command line for worlds, saves and content.

**Milestone M1, First settlers, is implemented** ([plan §7](PROJECT_PLAN.md#7-milestones)). A founding band settles a valley, farms, builds huts, raises families and wears trails, and you can watch it for ten years and then save and reload it ([the demo](assets/m1/m1-demo.webm)). It came in seven slices:

- **Slice A:** a new world begins with a founding band of 30–50 people in at least six families, who choose where to camp. They sleep, eat, fetch water, play and sit at the hearth, each choice scored from their needs and sampled, and they walk routed paths over the terrain. The map shows them moving. Click anyone to see what they are doing, their needs, their family and **why** they chose it, with every consideration's weight. The chronicle records the band's arrival.
- **Slice B:** people gather wild plants, hunt, fish and collect firewood, and bring home goods that keep or spoil at their own rates. A kill is shared across the camp. Meat and fish need a fire, so firewood matters. Game, fish and plants grow back on their own clocks. Hunting a valley out takes a season and its recovery takes years. Households share what they learn about places and weigh it against what they would expect. The chronicle notes when a settlement's food runs short and when it recovers.
- **Slice C:** households farm. They mark out fields near the settlement on floodplain, or on woodland they must clear first. They break the ground, sow emmer from the seed they brought, weed, reap and thresh by hand. Each task is chosen for the food it brings for the year ahead and for how close its season is to closing. A harvest depends on the field's ground, the year's weather, when sowing finished, the weeding done and how long the ripe crop stood. Before anything is eaten, seed for next season is set aside by the area planned. The seed to sow the ground already cropped is never eaten; seed beyond that is eaten only in real hunger. A household short of food asks one that can spare some. The map shows each field through its year, the readout names the field under the pointer, and the chronicle notes the first sowing and each harvest.

- **Slice D:** households build homes. Each designs a round post-built hut with wattle-and-daub walls and a thatched roof, sized for its members and with its door toward the hearth, and claims the ground its roof covers. People cut poles, rods and reeds and carry them home; threshing leaves straw for thatch. The hut goes up stage by stage (postholes, frame, walls, roof, floor), each stage using its materials as the work goes and waiting when they run out. Building is a choice like any other, and it presses harder as the first winter nears. Under its own roof, a household's grain keeps about three times as long. The map draws each hut as it goes up, from the shape the kernel expands from the saved design; the readout names the hut under the pointer and what it is waiting for; the chronicle notes the settlement's first roof.
- **Slice E:** people are born, pair up and die. A woman living with her partner conceives month by month, with a chance that depends on her age, on a fecundity of her own and on hunger. A pregnancy lasts about nine months or is lost, and after a birth a mother cannot conceive again for about twenty months, or for two if her child dies. Everyone faces a risk of dying that depends on age, from the life table of a foraging people, and hunger multiplies it. A body at the end of its reserve starves, and some mothers die in childbirth. Unpartnered adults look for a partner in the settlement within an age gap and outside close kin. A new couple sets up a household of its own, each bringing a share of their family's food and seed. Orphans go to their nearest kin, and a household that dies out leaves its fields and hut to its heirs. A household out of food and worn down by hunger, with no crop about to ripen, may give up and leave the valley. The founding band arrives as couples, some of the women pregnant and some nursing. The inspector names a person's partner and how long they have been together, a widowing, a pregnancy and a nursing child. The chronicle notes every birth, death and new couple, every child taken in and every household that leaves.

- **Slice F:** walking wears the ground. Every walk wears the 8 m cells it crosses a little, and unused ground grows back over months, so the ground people cross most becomes trail, and a trail outlives a season out of use. Worn ground is quicker to walk, so routes drift onto it and use makes more use. Across open ground people walk straight to where they are going rather than along the grid's eight directions. Once a month the paths are surveyed: routes are planned on them and the trails are traced into lines. The map shows worn ground as trodden earth with the trails through it, the readout names a trail under the pointer, and the chronicle notes each settlement's first trail out.
- **Slice G:** the observer can send a family and run ahead. With **Add a family**, a click on the map sends a family there, drawn like a founding family and carrying the same provisions. It joins the nearest settlement within 600 m, or camps on the spot and founds one, and the chronicle says the observer sent it. **Run ahead** lives a day, a month, a year, 5 or 10 years at full detail as fast as the machine allows, as a task you can cancel; the world pauses where it arrives and is saved. The smoke seeds can live ten years (`smoke --years 10`), checked at every year's end, and a nightly workflow runs them.

**Milestone M2, First light in Unreal, is in progress.** It puts the M1 village in Unreal Engine 5. Its kernel side is implemented: the kernel builds as a library, `tce_kernel` (a DLL on Windows), with a small, versioned C interface for Unreal to load ([ADR-0005](decisions/0005-kernel-c-interface.md)). The library runs the same engine and speaks the same frames as the web observer, and it can serve the panels alone, for Unreal's web view. CI builds the Windows DLL and drives it from C. The observer socket now refuses pages from other sites.

**Not yet:** everything inside Unreal: the plugin that loads the library, terrain built at run time, the building assembler and its kit, crowds, day and night, the HUD and a packaged build. That work needs Unreal Engine on the Windows PC.

**Milestone M3a, Village economy, is in progress** ahead of M2's Unreal work, with the web observer ([plan §7, §9](PROJECT_PLAN.md#7-milestones)). M3 is split in three: M3a the economy (goods, tools, skills, exchange and money, firms, property regimes), M3b knowledge and building, M3c seasons and time. Goods, the ledger, prices and firms follow [ADR-0006](decisions/0006-goods-ledger-firms.md).

- **Slice H (implemented):** goods, tools, recipes and skills. Grain is no longer eaten as it is. It is ground at a quern and baked into flatbread on the hearth, or pounded in a mortar for porridge. Households grind and bake as their ready food runs down, and only as much as they need, because bread and flour go off. Work needs its tools: reaping a sickle, preparing ground a hoe, cutting poles and building an axe, grinding a quern. Tools wear with the hours they are used, and households make new ones from flint, stone and wood they go out and gather. Without a sickle, people reap by hand, more slowly. Everyone has skills in milling, baking, knapping, stoneworking and woodworking. They rise with practice, and they set how fast the work goes and how long the tools made last. Households plan their stores to the next harvest and go out for wild food when they will not last, and they grow their fields for what is lost on the way to the mouth. Every good is accounted for: what households hold changes only by what is gathered, harvested, made, eaten, spoiled, burned, worn, built, sown or carried away, checked in the tests and each year of the ten-year smoke. The inspector shows a household's tools and food ready to eat, and a person's skills. `civ-host run` lives one world for some years and reports how it lives, with a yearly food balance. The bread chain makes villages more fragile than M1's: see the decision log's NUDGE on its cost.

- **Slice I (implemented):** exchange. One ledger moves every good that passes between households and names why: a gift of food, a kill shared out, a new couple's share, an inheritance, barter or a sale, and the books still balance. Each household values goods in hours of its own work, what it would take to grow, gather or make one more. Once a week it looks over what it can spare (tools beyond its needs, grain beyond what sees it to the harvest, materials once its hut stands) and posts terms: its cost and a margin, asked in the goods it is short of, moving slowly with what sells and what nobody offers. Someone whose household needs a tool, or food before the harvest, walks to the neighbour whose terms save it the most work, the walk included, and pays in goods. Tools that nobody offers go on record as wanted, and a household that does not need one makes it to sell when others would give more than it costs. A settlement trades by barter until one good settles most of what is paid; that good is its money, and nothing in the engine chooses it. In the first spring of one test village it was firewood. The Market panel shows each settlement's market: barter or money, what is offered and on what terms, what sold, what was wanted with none on offer, the latest trades in words and a monthly price history.

**Not yet:** firms with books and hired labour (slice J), property regimes (K), and villages of 200 and more with the demo (L).

### Implemented and planned

| Area | Status | What exists |
| --- | --- | --- |
| Kernel foundations (`civ-core`) | Implemented | Generational handles and permanent ids, a 365-day calendar, the event-and-cadence scheduler, small seeded RNGs |
| World generation (`civ-world`) | Implemented: terrain and water | Uplift and stream-power erosion, refinement to 8 m cells, valley floors, lakes from the water balance, river reaches with discharge and width. Climate bands, soils and deposits are planned (M1–M3) |
| Content (`civ-content`, `content/`) | Implemented: world presets, people, land, names, activities, goods, crops, building programs, recipes, skills | Strict TOML packs, stable diagnostics, fingerprints; two landscape presets; the early-farmers people profile (with its mortality, fertility and family rules), the temperate-valley land profile (stone and flint among its resources), 29 activities, 18 goods (four of them tools), seven recipes, five skills, one crop (emmer) and one building program (the hut). Other primitives arrive with the milestones that need them |
| Building grammar (`civ-grammar`) | Implemented: the hut | A pure expansion of a saved design (round post-built hut, wattle and daub, thatch) into its parts, outline and door and the labour and materials of each construction stage, pinned by golden hashes and a grammar version ([ADR-0004](decisions/0004-buildings-land-paths.md)). Authored rule graphs and more programs arrive with grammar v2 (M3) |
| Land (`civ-land`) | Implemented: habitats, wild plants, game, fish, fallen wood, fields, worn ground and trails | 128 m habitat patches classified from terrain. Plant-like stocks grow seasonally and waste; animals grow logistically toward their habitat's capacity and spread monthly; hunts take whole animals. Gathering slows as a patch empties (no respawn timers). A yearly climate factor, settlements. Fields are rectangles in centimetres that go through their crop's year; woodland is cleared before it is first broken. Dwelling plots are claimed in centimetres, and each building keeps its design and the work done on the stage under way. Walking wears 8 m cells, kept in sparse tiles; worn ground fades with a half-life, becomes trail with hysteresis, and is traced into trail lines each month |
| People (`civ-agents`) | Implemented: M1 slices A to G, M3a slices H and I | A founding band in families with ages from a life table; needs in closed form (energy, sleep pressure, company) with a body reserve; utility choices with softmax sampling and recorded receipts; routed walking (Tobler's hiking function, A*, pulled straight across open ground and faster on worn ground). Households keep goods that spoil by half-life, burn firewood by season, eat the most perishable food first and share kills; they remember what places gave and share it within the settlement ([ADR-0003](decisions/0003-people-movement-history.md)). Households plan fields for a year's food at a cautious yield, choose field work by its harvest and deadline, keep seed by planned area, and ask each other for food when short. They design their huts, claim plots, cut and carry what they build with, and build toward a roof before winter. Choices are sampled among the options worth more than doing nothing. People die by a life table that hunger multiplies, of starvation at the end of the body's reserve, or in childbirth; women conceive, carry, lose and bear children through a reproductive cycle with postpartum infecundity; couples form by age and outside close kin and set up households with a share of their families' stores; orphans go to their kin, emptied households pass to their heirs, and households in famine may leave. The observer can send a family, which joins a settlement nearby or founds its own. Person records, unions and the chronicle. Households grind and bake their grain, or pound it, as their ready food runs down, plan their stores to the next harvest and grow their fields for what is lost on the way; work needs its tools, which wear with use and are made from gathered flint, stone and wood; people's skills rise with practice and set speed and the life of tools made; every good is accounted for by its flows. One ledger moves goods between households by channel (gift, share, household share, inheritance, barter, sale); households value goods in hours of their own work, post weekly terms in the goods they are short of, buy from the neighbour whose terms save them most, put tools nobody offers on record as wanted and make tools to sell; each settlement's market infers its money from what settles most payments |
| Boundary schema (`commons-wire`, `civ-schema`) | Implemented | Frame envelope and FlatBuffers payloads for Rust, TypeScript and C++, schema TCE 1.9 ([ADR-0001](decisions/0001-boundary-schema.md)); the envelope and trip interpolation in header-only C++ for the plugin (`commons/cpp`) |
| Saves (`commons-persist`, `civ-sim`) | Implemented | Chunked, checksummed generations; verified on write; refusal, never repair ([ADR-0002](decisions/0002-snapshot-container.md)). Schema 2 added land and people, schema 3 goods, schema 4 fields, schema 5 plots and buildings, schema 6 couples, pregnancies and departures, schema 7 worn ground, schema 8 the families the observer sends, schema 9 people's skills, schema 10 households' terms and each settlement's market. M0 saves load as worlds with nobody in them yet; slice A saves load with their food as provisions; slice B saves load with no fields, which households then mark out; slice C saves load with no huts, which households then begin; slice D saves load with their couples found from their children and nobody expecting; slice E saves load with untrodden ground; slice F saves load as they are; saves before slice H load with each household given a founder's tools and each person a founder's skills; saves before slice I load with no markets, and households post terms at their next review |
| Host (`civ-host`) | Implemented | Command line and a localhost WebSocket server with autosave and crash recovery; running ahead at full detail as a cancellable task. The socket refuses pages from other sites and can require a token |
| Web observer (`web/`) | Implemented: M0 shell, M1 slices A to G, M3a slices H and I, and the panels alone for Unreal | Map with terrain, rivers and detail tiles; worn ground and trails; fields coloured by where they are in their year; huts drawn stage by stage as they go up; people moving along their trips; the settlement with its days of food and its harvest; an inspector with needs, family (partner, widowhood, pregnancy, a nursing child, who died and who left), the household's stores, food ready to eat and tools, the person's skills and the reasons for each choice; the chronicle; new-world (with band size), save, load and recovery dialogs; time controls and running ahead; sending a family where the map is clicked; the market panel (barter or money, offers and terms, sales, wants with none on offer, the latest trades, a price history); world facts; event log |
| CI | Implemented | A Windows lane (kernel, commons, content, smoke seeds, the kernel library's C harness under MSVC), the Windows DLL built and kept as an artifact, browser tests and schema freshness; ten years of the smoke seeds nightly and on pull requests into `main` |
| Firms, hired labour, property regimes | Planned (M3a slices J–L) | See the [milestones](PROJECT_PLAN.md#7-milestones) |
| Technology, grammar v2, structural rules, weather, soils, the Accelerated mode | Planned (M3b, M3c) | |
| Built and kept roads, government, services, diplomacy | Planned (M4–M8) | |
| Kernel library (`civ-ffi`) | Implemented: ABI 1.0 | `tce_kernel`, the kernel as a library: one export hands back a versioned table of C functions to create a kernel, submit frames, poll ordered frames, copy the latest snapshot and serve the panels. A generated header, Rust tests through the table, a C harness that loads the library at run time, and a C++ host that reads its frames with the generated C++ readers and asks it for a world, as the plugin will ([ADR-0005](decisions/0005-kernel-c-interface.md)) |
| Unreal client (`EngineBridge`, runtime terrain, building assembler) | Planned (M2, on the Windows PC) | |

### Known limitations

- **Terrain.** Erosion routes water along the grid's eight directions, which leaves occasional straight valleys and creases, visible in the hillshade up close. Valley-floor edges can look jagged at the 8 m cell scale.
- **Lakes.** Lakes are rare in the humid presets, and closed basins use a single-lake approximation.
- **Small maps.** The large river can miss a small map, and at 2 km and 4 km the coastal preset can be mostly sea. The smoke seeds use 8 km maps.
- **Performance.** A 16 km world takes about 18 s to generate on a 4-core 2.1 GHz cloud CPU, and its save is about 15 MB. Your machine will differ.
- **WebGL.** The map needs WebGL. Headless browsers fall back to slow software rendering.
- **Hard first years, and some bands fail.** Wild food on a valley map feeds only about a dozen foragers (research 03-06 §2.7), not a band of 40, so farming has to feed it. Breaking ground by hand, a band crops all it needs only by its third harvest, so it arrives with eighteen months of provisions. That figure is a tuning value, and it also stands in for the herds real colonists drove in, which M1 does not have. In five-year runs of six seeds on 6 km river-valley maps, four bands cropped 13.5–16 ha and had 41–47 people in their fifth year (from 40), with 7–16 births and 6–9 deaths each. The band of seed 3 had poor harvests: one household of seven gave up and left, and it ended with 34 people. The band camped in woodland (seed 7) cropped only about 7 ha, and within five years everyone had left or died. Runs differ: in an earlier run of the same seeds, seed 3 lost six of its nine households in its fourth June. In two ten-year runs of the ten smoke worlds (8 km maps), eight bands lived on with 29–61 people, and the bands of seed 1 failed in both landscapes both times. Founding settlements did fail (research 05-06 §5.2), but how often these fail is not calibrated.
- **Small numbers.** With ten or so women of childbearing age, a band's fertility is noisy. In one set of five-year runs it came to 2.6 children per woman in one growing band and 5.7–6.9 in three others (research 05-01 §2.2: 6.0 on average across natural-fertility populations, give or take 1.4), with births 29–42 months apart.
- **Families are simple.** There is one rule for where couples live (a household of their own), no divorce, no second spouse, and no moving between households except at pairing, orphanhood and when a household dies out. Inheritance passes a whole household's land to one heir. The life table already counts deaths in childbirth, so the explicit risk counts some of them twice (research 05-01 §1.3). Disease, injury and cold are not modelled apart from the life table.
- **Leaving is final.** A household that gives up leaves the world: its people are no longer simulated, and its fields and hut stand abandoned. With one settlement there is nowhere to go, nobody returns, and no relatives send help.
- **Trails are only worn.** Nobody clears, builds or keeps a path: a trail is where walking wore the ground, and it fades when walking stops. Wear is kept per 8 m cell, so where in its cell a trail runs is not known; the map draws worn ground softly and trails as smoothed lines. Only dry land wears, so a trail stops at a river bank and goes on from the far side. Routes are planned on the paths as last surveyed, so ground worn this month speeds walking from next month. The wear per walk, its half-life and the trail thresholds are the research's uncalibrated starting values (10-03 §2.2).
- **One crop, one season.** Emmer is sown in spring only; there is no autumn sowing, no pulses and no livestock. The calendar and the extra work of clearing woodland are tuning values, since the research gives none.
- **Fields do not change the land.** A woodland field still counts as woodland for wild food and firewood, and a field keeps the quality of its ground; a nutrient budget waits for soils (research 03-04).
- **One kind of building.** Every household builds the same kind of hut, sized for its members when it is designed; a growing family does not enlarge it. There are no storehouses, workshops or repairs, and nobody builds a second hut (a household may inherit one); buildings never decay or burn. A plot is given up only when its household leaves or dies out with no heir. A roof keeps stores but does nothing for people yet: there is no cold or exposure.
- **Thatch can be scarce.** A hut for five takes about 500 hours of building, 1 t of timber and 1.7 t of thatch. A band camped in woodland far from reed beds (seed 7 of the river valley) waits for its harvest's straw and distant reeds: its first roof goes on at the end of October and the rest in the first winter. Elsewhere every household of the test bands is under a roof by midsummer. Households do not share spare straw or other materials, and there are no bark or turf roofs. Weaving wattle, the yields of poles and reeds, and how long stored timber and thatch keep are tuning values.
- **No preserving.** Fresh meat, fish and plant food spoil within days, and there is no drying or smoking yet, so a summer glut cannot be stored for winter.
- **Tools are counted, not tracked.** A household's sickles are one number in standard tools' worth of use (ADR-0006): a worn sickle is a part of one, nobody knows who made which, and a half-made axe counts as part of an axe. Worn tools do not slow work; a tool works until it is used up. Tool lifetimes and the work to make them are tuning values (the research gives none). Only reaping has a fallback by hand.
- **Idle hours.** An adult of the test villages spends about 3 hours a day on production, 1–2 on domestic work and some 10 resting or at the hearth; research 04-02 proposes about 6 and 4 for an ordinary farming day. The tasks need little time once the fields are broken; work for others and for trade should draw on it (§9, NUDGE).
- **A small market.** Households trade only within their settlement, one exchange at a time at the seller's posted terms, with no haggling, credit or debts; a buyer goes only for a tool it needs or food it is short of. Each household values goods by its own costs in hours, so the same good has different prices in different houses. Only tools nobody offers are recorded as wanted. Food is offered only out of what keeps a year beyond the next harvest, and planned stores look only to that harvest, so in a household's first years it may offer provisions it will want later. The money a settlement settles on is whatever settles most payments: in one test village's first spring it was firewood. The review cadence, margin, largest change, memory and money threshold are the research's uncalibrated starting values (08-04, 08-06).
- **Stone and flint are a stand-in.** They lie in fixed amounts per hectare by habitat until geology arrives (M3b).
- **Simulation speed.** On the 4-core cloud CPU, a village of 40–50 people lives a good year in about 10 s (three years and three months in 30–33 s from the command line, seeds 1, 2, 11 and 19). Hungry years take about twice as long: long hunting trips to new places each need a route planned. Running ahead in the observer, with the map following, the demo's village of 50–60 lived nine years in about 140 s, some 15 s a year. There is no faster mode yet: the Accelerated mode (a statistical day step) is planned for M3.

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
cargo run --release -p civ-host -- new --seed 2 --size 768 --days 1201   # a village in its fourth June
cargo run --release -p civ-host -- save info  ../saves/<world>/g0000000001-manual.tcesave
cargo run --release -p civ-host -- save verify ../saves/<world>/g0000000001-manual.tcesave
cargo run --release -p civ-host -- content validate
cargo run --release -p civ-host -- smoke       # 2 landscapes × 5 seeds against fixed thresholds
cargo run --release -p civ-host -- smoke --years 10   # and ten years of each, checked every year
cargo run --release -p civ-host -- run --seed 2 --years 5   # one world for five years, reported each year
```

The kernel library for Unreal (or any other host) is built and published, with its header and a manifest, by `tools/build-kernel-dll.ps1` on Windows (with its PDB) or `tools/build-kernel-dll.sh` elsewhere, into `dist/tce_kernel/<target>/`. CI also keeps each Windows build as an artifact.

Tests, lints and the rules for changing schemas, saves and content are in [AGENTS.md](AGENTS.md).

## What the build looks like

**These are screenshots of the current build**, unlike the concept art further down.

<p><img src="assets/m1/m1-ten-years.jpg" alt="Wrenholt on 31 March of its eleventh year, the map about a kilometre across: a village of 63 by a river, ringed by square fields, most of them sown and green, joined to the village by thin trails; people are out in the fields; the chronicle lists births, a new couple and a child's death in the tenth year"><br><strong>M1: Wrenholt ten years on, seed 2.</strong> A band of 40 made camp here on 1 March of year 1. Ten springs later, 63 people live here, including a family the observer sent in the second year. They are out in the fields around the village, and trails run out to them. The chronicle records the year's births, a new couple and a child's death. The <a href="assets/m1/m1-demo.webm">M1 demo</a> (107 s) follows these ten years: it creates the world, watches the first morning, runs ahead a month and a year, sends the family, runs ahead to the tenth year (shown five times faster), then saves the world and loads it back.</p>

<p><img src="assets/m1/m1-trails.jpg" alt="Hazelstead in March of its sixth year, seen from 100 m up: a village by a river with fallow fields around it, joined to them by a network of thin brown trails; the readout says a trail is under the pointer, and the chronicle notes the first trail out of Hazelstead, 213 m long, in the band's first April"><br><strong>M1 slice F: the paths of Hazelstead in its sixth March, seed 1.</strong> Five years of walking have worn trails from the village out to its fields across the valley. Nobody laid them out: they are where people walked most. The readout names a trail under the pointer, and the chronicle noted the first trail out in the band's first April. The fields lie fallow until sowing.</p>

<p><img src="assets/m1/m1-families.jpg" alt="Birchbank at noon in June of its fourth year: eleven thatched huts around a hearth near a river, square fields of green emmer around the village; the inspector shows Yara, 29, of the founding band, resting at home, partner of Jory for 9 years and nursing her daughter Vesna, with her family list including a son who died; the chronicle lists a birth, a harvest and the deaths of two small children and a youth of 15"><br><strong>M1 slice E: Birchbank in its fourth June, seed 2.</strong> Forty-one people live here under eleven roofs, and the emmer is growing. Yara came with the founding band. The inspector says she has been Jory's partner for nine years and is nursing Vesna, fifteen months old; her son Nils died in January, aged 3. The chronicle notes the village's births, deaths and harvests. Earlier entries tell of two young couples who set up households of their own.</p>

<table>
  <tr>
    <td width="50%"><img src="assets/m1/m1-huts-building.jpg" alt="Eight hut plots around the hearth of Hazelford by a river in April: one hut half thatched, the others ringed by posts or postholes, people building; the readout says a hut's frame is going up, 18% done, waiting for timber"><br><strong>M1 slice D: huts going up at Hazelford, 22 April, seed 3.</strong> Each plot is the square its hut's roof will cover. One hut is being thatched; the others show their postholes or posts. The readout names the hut under the pointer: its frame is 18% done and it waits for timber.</td>
    <td width="50%"><img src="assets/m1/m1-huts-roofed.jpg" alt="Eight huts at Hazelford in June, all thatched, each doorway facing the hearth, people asleep inside at dawn"><br><strong>Hazelford in June, from another run of the same seed.</strong> Every household is under a thatched roof, each hut sized for its household with its door toward the hearth. At dawn its people are still asleep inside.</td>
  </tr>
</table>

<p><img src="assets/m1/m1-fields.jpg" alt="Otterholt in its second August: fields around the village by a river, most reaped and brown, a few still green, one pale with sheaves to thresh; the inspector shows a woman threshing grain from her household's field, its stores of grain and seed, and why she chose to thresh"><br><strong>M1 slice C: Otterholt's second harvest, seed 11.</strong> Most fields are reaped (brown), a few late-sown ones still grow (green) and one holds sheaves to thresh (pale). Wren threshes at home; her household keeps 974 kg of grain and 215 kg of seed. The chronicle notes the first sowing, the first harvest of 5,646 kg, and a short spell of hunger before the second.</p>

<p><img src="assets/m1/m1-inspector.jpg" alt="A founding band at their camp by a river; one person is inspected: what she is doing, her needs, her household's food, firewood, water and stores, the reasons for her last choice, her family, and the chronicle"><br><strong>M1 slices A and B: a band of 30 at Hazelford, seed 3.</strong> Dots are people coloured by activity; a firewood party works south of the camp. The inspector shows the household's days of food, firewood and water and what is in store, then the last decision's considerations, the next-best option and what was ruled out.</p>

The M0 screenshots below show the terrain alone. Recordings of the M0 end-to-end tests are in [`assets/m0/m0-demo.webm`](assets/m0/m0-demo.webm), which generates, explores, saves and loads a world, and [`assets/m0/m0-recovery.webm`](assets/m0/m0-recovery.webm), which recovers after the host is killed.

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
