# Simulation Model

This page describes systems present in the `main` source baseline documented by this journal. The project is still an early farming-village simulation. The broader historical and modern scope is planned, not already simulated.

## Time and simulation work

`SimTime` is an integer-minute clock over a 365-day calendar. The world begins at a fixed in-game date and time. Detailed mode offers 1×, 3× and 10× speeds; 1× advances 96 simulated seconds per real second, or one in-game day per 15 minutes. Accelerated statistical stepping is a later milestone.

The kernel avoids polling every person on every tick. `civ-core::Scheduler` merges two kinds of due work:

- **Events** such as a person's current activity or trip step finishing. They are ordered by timestamp, phase, a random tie-break chosen when scheduled, then scheduling order. Events carry identity/activity versions so events for a dead person or abandoned activity can be ignored safely.
- **Cadences** such as minute, hour, day, week, month, season and year. At the same instant they are delivered from finest to coarsest, so a yearly summary sees all work from the periods that ended with it.

The event handler can schedule follow-up events. A guard reports excessive same-instant work as a zero-time loop instead of hanging. `civ-sim::Sim::advance_minutes` advances the scheduler in simulation time; `advance_real` maps elapsed wall time through the selected speed.

## Seeded world and changing land

World generation is a pure function of a terrain preset and seed on a given build. Its current stages are:

1. Build a coarse watershed context with uplift, stream-power incision, hillslope diffusion and talus relaxation.
2. Refine the map region through three levels to the 8 m cell size, adding relief-scaled detail and erosion.
3. Classify ocean and water, fill artifact depressions, calculate lakes and drainage receivers.
4. Trace river reaches between confluences, lakes, sinks and outlets; estimate discharge and width from drainage and runoff.

The terrain, elevation, water and river network are treated as generated base state. The world code also supplies terrain measures and navigation inputs. Mutable ecological and human state lives elsewhere.

`civ-land` groups wild resources into habitat patches, currently 128 m across. Plant-like resources grow and waste; animals recover toward habitat capacity; gathering slows as stocks are depleted. Fields track crop phase and work, seasonal deadlines and expected yields. Walking adds wear to 8 m cells, which fades and is periodically traced into paths; routes can use the most recently surveyed paths.

At this baseline, stone and flint also have fixed habitat-level stock estimates. M3b slice Q has added a deterministic deposit-body placement primitive that can place bodies on a generated map from rules and a seed. The placement primitive is not yet part of normal world creation, resource finding/extraction or earthworks. Keep that implementation boundary explicit.

## People, households and choices

People and households are represented in ordinary Rust tables, linked by stable permanent IDs and temporary generation-checked handles. The founding band is initialized from an authored people profile and life-table assumptions. Relationships, ages, births, deaths, households, inheritance and departures become explicit state and chronicle events.

An agent's decision process is organized around activities defined in content:

1. Gather relevant facts such as age, work capacity, needs, time of day, food outlook and available places.
2. For each possible activity, retain its best available target (for example a gathering patch or field).
3. Score each option by adding decision considerations that share a unit. Impossible options are excluded with a reason.
4. Sample from the remaining options with a softmax whose temperature scales with the score spread.
5. Store the chosen activity and a bounded decision receipt containing scores, exclusions and the runner-up.

The receipt powers the observer's “why” explanation; it is not a separate decision made by the UI. Decisions are plausible bounded choices, not unrestricted optimization or scripted historical events.

Movement is represented as trips over routes, with start/arrival times, path identity and geometry queried or referenced for presentation. The web map draws people along those trips. This keeps walking and rendering separate from a per-frame position simulation.

Households share stores and labor. People eat, sleep, socialize, gather, hunt, fish, farm, carry, make goods, construct, mend and trade where their age, needs, skills, materials and knowledge permit. A household that cannot sustain itself may leave; departures remove people from the world rather than simulating migration to another settlement.

## Food production, goods and exchange

The current farming cycle centers on spring-sown emmer. Households select and prepare rectangular fields, clear woodland where necessary, sow, weed, reap and thresh. Yield depends on ground, weather/climate factor, planting time and harvest timing. Seed is reserved against planned area before grain can be eaten. Current crops, seasons and work rates are intentionally limited; M3c is planned to deepen weather, soils and farming.

Goods are held in kilograms with kind-specific spoilage and use. Firewood is burned by an authored seasonal rate; cooked food can require fuel; shared goods such as a kill are divided; tool durability is expressed as remaining useful work. Recipes consume inputs, people need the relevant tool and skill, and practice increases skills.

M3a adds an accounting ledger for transfers: household allocation, gifts, shared food, new-couple contributions, inheritance, barter and sale. The ledger records source, destination, amount and channel so that goods cannot appear through repricing. Households estimate value in hours of their own labor and post terms; buyers choose among offers. A village's money is inferred from the good that settles the most payments. It may still barter.

Workshops belong to households and keep their own goods and books. They can post goods, hire work for wages and close. The model is small: firms make a narrow range of tools; they do not yet have modern contracts, credit or complex company organization.

Property regimes govern who holds fields and how fields can be leased. A field's holder and cultivator are separate facts. Wealth reports land, goods and floor area with distribution measures; measured wealth does not yet drive every social decision.

## Knowledge and buildings

Techniques are authored capabilities such as growing a crop or framing a building. Activities, recipes and building programs can require them. Each person can know a technique, be learning it, or have heard of it. Knowledge transfers through household upbringing and working beside a knowledgeable person; some techniques can be discovered from practice or purposeful attempts. A settlement records when a technique arrives or is lost. The observer can introduce or tell a person about a technique.

Buildings begin as saved design specifications. `civ-grammar` expands a specification into an outline, door, construction stages, materials and labor. Frame grammar also yields bays, floors, use areas, storage and stable component groups. Rendering receives the expanded marks; it does not decide dimensions or building rules.

Construction consumes materials and labor over time. Components receive an initial quality, then wear according to exposure and authored upkeep parameters. Households choose and perform mending. Loads from the building, stored goods, occupants and monthly weather can cause sagging or failure; collapses can spill goods, damage structures and kill people. Settlements keep fading records of building failures and building-years. These records affect the dimensions chosen for later frame buildings, within each program's allowed range. The observer reports the condition and the kernel's explanation.

At the source baseline, M3b slices M–P are implemented and slice Q has started. Deposit placement is only the first step: deposits are not yet integrated into ordinary world creation or extraction, and pits/quarries, earthworks, clay processing and the geological observer tool are still ahead. Check the README and plan before claiming their status.

## What the model does not claim

- Modern governments, industrial economies, cities and infrastructure are not implemented in this baseline.
- Realistic behavior is an aim checked against selected stylized facts, not a proof that the model reproduces history.
- Smoke tests cover chosen seeds and rules. Passing them does not establish that all plausible worlds survive or that every parameter is calibrated.
- Early villages can fail from food shortages. Single-settlement departures are final, and there is no regional migration network.
- Terrain, land use and ecology are still deliberately coarse; climate, soils, multiple crops, livestock and changing drainage arrive in later work.

See [Development and Evidence Practice](development-and-evidence.md) for how the project records these boundaries.
