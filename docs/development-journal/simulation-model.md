# Simulation Model

This page describes systems present in the [source baseline](README.md#source-baseline) documented by this journal. The project is still an early farming-village simulation. The broader historical and modern scope is planned, not already simulated.

## Time and simulation work

`SimTime` is an integer-minute clock over a 365-day calendar. The world begins at a fixed in-game date and time. Detailed mode offers 1×, 3× and 10× speeds; 1× advances 96 simulated seconds per real second, or one in-game day per 15 minutes. Accelerated mode offers 60×, 600× and Max, which is unpaced and as fast as the machine allows ([ADR-0011](../../decisions/0011-execution-modes.md)). At those speeds the kernel lives whole days, midnight to midnight, through the same events, activities and decisions as Detailed mode would; the clock stops, publishes and saves only at midnight, and a change of mode takes effect at a midnight, after the day's work and before the new day's first event. The mode follows the speed and is a setting, not saved state. No approximation is declared yet, so a day lived at Max is exactly a day lived by the minute. The declared approximations (a household view built at midnight, and leisure blocks) and the statistical test that must cover them are planned for slice W.

The kernel avoids polling every person on every tick. `civ-core::Scheduler` merges two kinds of due work:

- **Events** such as a person's current activity or trip step finishing. They are ordered by timestamp, phase, a random tie-break chosen when scheduled, then scheduling order. Events carry identity/activity versions so events for a dead person or abandoned activity can be ignored safely.
- **Cadences** such as minute, hour, day, week, month, season and year. At the same instant they are delivered from finest to coarsest, so a yearly summary sees all work from the periods that ended with it.

The event handler can schedule follow-up events. A guard reports excessive same-instant work as a zero-time loop instead of hanging. `civ-sim::Sim::advance_minutes` advances the scheduler in simulation time; `advance_real` maps elapsed wall time through the selected speed; and `advance_to_midnight`, the only way Accelerated mode advances, lives to the next midnight. An advance stops after the cadence boundaries of the minute it stops at and before that minute's events, so a run cut anywhere lives the same.

## Seeded world and changing land

World generation is a pure function of a terrain preset and seed on a given build. Its current stages are:

1. Build a coarse watershed context with uplift, stream-power incision, hillslope diffusion and talus relaxation.
2. Refine the map region through three levels to the 8 m cell size, adding relief-scaled detail and erosion.
3. Classify ocean and water, fill artifact depressions, calculate lakes and drainage receivers.
4. Trace river reaches between confluences, lakes, sinks and outlets; estimate discharge and width from drainage and runoff.

The terrain, elevation, water and river network are treated as generated base state. The world code also supplies terrain measures and navigation inputs. Mutable ecological and human state lives elsewhere.

`civ-land` groups wild resources into habitat patches, currently 128 m across. Plant-like resources grow and waste; animals recover toward habitat capacity; gathering slows as stocks are depleted. Fields track crop phase and work, seasonal deadlines and expected yields. Walking adds wear to 8 m cells, which fades and is periodically traced into paths; routes can use the most recently surveyed paths.

Weather is one daily series per world, drawn from its seed and landscape (its preset) and the same at every speed; temperature is adjusted for each place's height ([ADR-0012](../../decisions/0012-weather-and-soil.md), [`civ-land/src/weather.rs`](../../kernel/crates/civ-land/src/weather.rs)). Rain comes from a wet/dry chain with persistence and gamma-distributed amounts, each month taking its share of the preset's annual total; temperature is an autocorrelated anomaly around monthly normals; a slow monthly anomaly carries wet and dry spells across seasons; and snow lies and melts by 100 m band. A root-zone water balance under the wild cover is the reference soil water, and wild plant food follows it, smoothed over a month. The parameters are in the land profile's `[weather]`, each marked there as a research value (often a pick within a reported range) or a tuning value. The series replaced a yearly climate factor whose key had no landscape in it, so both landscapes of a seed had shared the same lean years. On the first of each month the chronicle notes, in the kernel's words, a month, a winter or a year whose weather stood out against what it usually brings.

Stone and flint still also lie as loose stocks per habitat, which never renew. Beside them, every world lays down deposit bodies of clay, stone and flint once from its seed, by the land profile's `[[deposit]]` rules, after its founding band arrives ([ADR-0010](../../decisions/0010-ground-people-change.md), [`civ-land/src/deposits.rs`](../../kernel/crates/civ-land/src/deposits.rs)). Each body has a size, cover, thickness and quality and a finite inventory: what is taken plus what is left is always what it began with. A settlement finds a body that shows at the surface when one of its people walks within 50 m of its edge, and a buried one when a levelled plot's cut reaches it; the observer can also lay one down. People dig clay at deposits their village knows, and quarry stone and dig flint once the loose stone and flint near the village run short. Where deposits lie and how large they are are tuning values.

People also change the ground. Households level sloping plots by cut and fill before they build, dig pits and quarries with spoil heaps beside them, and dig each building's daub from a pit beside its plot. Each earthwork is a record, and its effect is kept as per-cell changes in mean height, in 64 × 64-cell tiles beside the generated bed, which is never rewritten ([`civ-land/src/earth.rs`](../../kernel/crates/civ-land/src/earth.rs)). Walking, water and drainage still follow the ground as generated; the elevation the observer is served is the ground as levelled.

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

The current farming cycle centers on spring-sown emmer. Households select and prepare rectangular fields, clear woodland where necessary, sow, weed, reap and thresh. Seed is reserved against planned area before grain can be eaten. Yield depends on the ground, the field's water, planting time and harvest timing. From sowing to ripening each growing field keeps its own root-zone water balance (Hargreaves reference evapotranspiration and emmer's crop coefficients by stage), and its harvest scales by FAO's yield response, 1 − Ky × (1 − water had ÷ water needed) with emmer's `Ky` of 1.15, divided by that response's long-run mean for the landscape, so an average season gives the content's yield. Preparing ground and sowing wait for days the ground can be worked: snow lying, 5 mm or more of rain or snow in the day, or a daily mean below freezing keeps people from turning the soil, while weeding, reaping and threshing go on in any weather. Households plan their spring work on the days of the sowing window that can usually be worked (81 % in the valley and 76 % on the coast, over the long run), at a peak's longer hours, and judge a growing crop by the water its season so far has brought, never by future draws. Crops and work rates are intentionally limited. A field keeps the quality of its ground; soils and fertility are planned for slice V.

Goods are held in kilograms with kind-specific spoilage and use. Firewood is burned by an authored seasonal rate; cooked food can require fuel; shared goods such as a kill are divided; tool durability is expressed as remaining useful work. Recipes consume inputs, people need the relevant tool and skill, and practice increases skills.

M3a adds an accounting ledger for transfers: household allocation, gifts, shared food, new-couple contributions, inheritance, barter and sale. The ledger records source, destination, amount and channel so that goods cannot appear through repricing. Households estimate value in hours of their own labor and post terms; buyers choose among offers. A seller asks less for food the more of it it can spare, so grain's asks answer the harvest. A village's money is inferred from the good that settles the most payments. It may still barter.

Workshops belong to households and keep their own goods and books. They can post goods, hire work for wages and close. The model is small: firms make a narrow range of tools; they do not yet have modern contracts, credit or complex company organization.

Property regimes govern who holds fields and how fields can be leased. A field's holder and cultivator are separate facts. Wealth reports land, goods and floor area with distribution measures; measured wealth does not yet drive every social decision.

## Knowledge and buildings

Techniques are authored capabilities such as growing a crop or framing a building. Activities, recipes and building programs can require them. Each person can know a technique, be learning it, or have heard of it. Knowledge transfers through household upbringing and working beside a knowledgeable person; some techniques can be discovered from practice or purposeful attempts. A settlement records when a technique arrives or is lost. The observer can introduce or tell a person about a technique.

Buildings begin as saved design specifications. `civ-grammar` expands a specification into an outline, door, construction stages, materials and labor. Frame grammar also yields bays, floors, use areas, storage and stable component groups. Rendering receives the expanded marks; it does not decide dimensions or building rules.

Construction consumes materials and labor over time. Components receive an initial quality, then wear by exposure: on the first of each month thatch, daub and the feet of posts wear as wet as the month just lived was against its usual (from a quarter to three times the authored rate), and rafters and joists rot only under a leaking roof. Households choose and perform mending. Loads from the building, stored goods, occupants, the month's storm (one for the whole world, on one day of the month) and the snow lying on its roof at the building's height can cause sagging or failure; a steep roof keeps less of the snow than a flat one. Collapses can spill goods, damage structures and kill people. Settlements keep fading records of building failures and building-years. These records affect the dimensions chosen for later frame buildings, within each program's allowed range. The observer reports the condition and the kernel's explanation.

Households also build clay ovens and make storage pots, which keep grain and flour under a roof as a raised floor does. Each household builds to its own taste in roof pitch, wall height to the eaves and overhang, drawn from its band's way of building and held to what each program allows. Once a year its taste moves toward the new buildings its village admires, for their owner's standing in goods and their builders' craft, and its next building records the one it followed. Cob walls wait for a mass-wall grammar.

At the source baseline M3b is implemented in all six slices (M–R), M3c slices S and T are implemented, and slice U is under way. Check the README and plan before claiming the status of a feature.

## What the model does not claim

- Modern governments, industrial economies, cities and infrastructure are not implemented in this baseline.
- Realistic behavior is an aim checked against selected stylized facts, not a proof that the model reproduces history.
- Smoke tests cover chosen seeds and rules. Passing them does not establish that all plausible worlds survive or that every parameter is calibrated.
- Early villages can fail from food shortages. Single-settlement departures are final, and there is no regional migration network.
- Terrain, land use and ecology are still deliberately coarse. Daily weather is now simulated, as one regional series per world adjusted for height (temperature and snow lying), not drawn separately for each place or field; soils, multiple crops, livestock and changing drainage arrive in later work.

See [Development and Evidence Practice](development-and-evidence.md) for how the project records these boundaries.
