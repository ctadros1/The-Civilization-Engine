# Simulation Model

This guide describes the integrated public `main` baseline at commit [`33dbd4a`](https://github.com/ctadros1/The-Civilization-Engine/commit/33dbd4a), dated 2026-10-05. TCE currently simulates an early farming society. The wider historical and modern vision is not the current simulation or the planned v1 scope.

## Time, scheduling and speeds

`SimTime` is an integer-minute clock over a 365-day calendar. A new world begins on 1 March, year 1, at 06:00. `civ-core::Scheduler` merges scheduled events, such as a person finishing an activity or trip, with system cadences such as minutes, hours, days, weeks, months and years. Events have a phase, a keyed random tie-break and a scheduling sequence; entity and activity versions make stale events harmless. At a shared timestamp, cadences run from finer to coarser so a year summary sees the work that ended with it.

Detailed mode offers pause, 1×, 3× and 10×. Accelerated mode offers 60×, 600× and Max. It advances whole days with the same authoritative model and event machinery, and may stop, publish, save or change modes at midnight. The exact consistency checks compare equivalent advance partitions and save/load continuations. Longer statistical comparisons are recorded separately; they do not make the simulation deterministic.

## Seeded world and changing land

World generation is a pure function of the terrain preset and seed for a given build. The pipeline forms coarse watersheds with uplift, incision and hillslope processes; refines relief through multiple levels to 8 m cells; classifies water and drainage; then traces lakes and river reaches with estimated discharge and width. Generation parameters are authored content. The generated terrain is saved and remains the base surface; later earth moved by people is recorded separately.

`civ-land` holds changing world state: 128 m habitat patches and their wild stocks, finite clay/stone/flint deposits, settlements, fields, building plots, structures, pits, quarries, earthworks, weather and worn ground. Plant-like resources grow and decay, animal stocks recover toward habitat capacity, and gathering reflects depletion. Walking wears 8 m cells; the wear fades, and the kernel traces persistent wear into routes.

## Weather, water and farming

Weather is a daily series shared across a landscape, keyed by world seed, landscape identity and day. The two presets therefore have separate histories for the same seed, while every speed sees the same history. The generator models wet-day persistence and rain amounts, temperature variation around monthly normals, a slower anomaly that carries wet or dry spells across seasons, and snow that lies and melts by elevation band. The `weather` content profile holds its parameters. Full parameter choices and measured distributions are in [ADR-0012](../../decisions/0012-weather-and-soil.md) and the plan's §9 log.

Reference soil water changes with rain, snowmelt and temperature-based evapotranspiration. A growing field keeps its own root-zone water balance and crop-stage water needs from sowing to maturity. Harvest responds to the share of seasonal water need met, normalized against the landscape's long-run mean so an average season remains aligned with the authored crop yield. Wild plant growth also responds to soil water. The yearly climate multiplier has been retired.

Weather limits only work that turns the soil: breaking and preparing ground and sowing wait when snow, wet ground or freezing makes the field unworkable. Weeding, reaping and threshing continue in wet weather. Households estimate the usual workable share of the crop's preparation/sowing window when planning, then respond to the weather as it occurs; they do not see future draws. The observer shows daily conditions, snow and season on the map, monthly and yearly summaries in a weather panel, yearly extremes in the chronicle, and water received so far in growing-field details. `civ-host weather` replays a preset's series and reports workable days.

The same weather drives monthly wear on thatch, daub and post feet, and supplies storm and snow loads for buildings. Grain asking prices now react to household stores, allowing weather-driven harvest changes to reach the market. The weather feature was checked over long generated histories and through smoke worlds; see [Development and Evidence Practice](development-and-evidence.md) and the plan log for the exact runs and outcomes.

Field water is implemented; the field nutrient budget is not yet in the integrated baseline. M3c slice V is planned to add nitrogen pools, harvest histories and deliberate field rest/manuring. More crops, livestock, farming earthworks and the remaining M3c tuning/demo work also remain ahead.

## People, households and choices

People and households are plain Rust tables connected by permanent IDs and generation-checked handles. An authored people profile supplies founding ages, relationships, needs, fertility and mortality assumptions. Births, unions, deaths, inheritance and departures become state changes and chronicle events.

Activity definitions come from content. For each decision, the kernel gathers relevant facts, finds a best target for each feasible activity, scores its considerations in a shared unit, excludes impossible options with a reason, samples among the rest with a softmax, and stores a bounded decision receipt. The receipt powers the observer's explanation; the UI does not decide. An agent's trip is a route with scheduled start/arrival and geometry for display, rather than a per-frame simulation position.

People gather, hunt, fish, farm, make goods, build, mend, trade and meet household needs when their age, skills, tools, knowledge, materials and circumstances allow. Households share stores and labor. A household that leaves is removed from the world; the current model has no regional migration network for it to join.

## Goods, work and exchange

Goods are held in kilograms with authored spoilage, use and recipes. Tools wear through useful work; skills improve through practice and affect work rates. Each transfer between households is recorded in a ledger with source, destination, amount and channel. Household valuation is expressed in hours of its own labor; sellers post terms in goods they want, and buyers compare offers. The settlement's money is inferred from the good settling the most payments, so barter can remain common.

A household workshop keeps its own stores and books, posts terms and a wage, hires work through the ledger and closes when it cannot continue. Property regimes determine who holds fields and whether they can be leased. Wealth measures cover goods, land and floor area, but do not yet drive every social choice. Firms and markets remain deliberately small; there are no modern contracts, credit systems or complex company structures.

## Knowledge and buildings

Techniques gate activities, recipes or building programs. People can know, learn or have heard of a technique. They learn through upbringing and work beside a knowledgeable person; practice and purposeful attempts can discover some techniques. A settlement records when a technique arrives or is lost, including loss with its last knower. The observer can introduce a technique to a person.

A saved building design is expanded by the pure `civ-grammar` crate into parts, outline, stages, materials and labor. The hut grammar remains versioned; the frame grammar derives bays, floors, use areas, storage, work areas and stable component groups. Households choose among structures they know how to build and can afford. The observer renders kernel-produced marks and dimensions rather than calculating building rules.

Materials and labor are consumed through construction stages. Parts receive quality from builders' skill and wear according to material, exposure and upkeep. Households mend deteriorated parts. Loads include the structure, stored goods, occupants, storms and snow. Parts can sag or fail, spilling stores and sometimes killing people. Settlements keep a fading record of failures and building-years; that memory changes how strongly later frame buildings are built, within their authored programs. The observer reports the condition and kernel explanation.

## Known limits

- The integrated content is early agrarian, centered on emmer; modern government, industry, infrastructure and cities are not implemented.
- Nitrogen-based field fertility and intentional rotations/resting remain M3c V work. Farming earthworks and additional crops/livestock are later scope.
- Weather and workability increase pressure on already fragile founding societies. The ten-year smoke for U passed its selected checks, but several bands were near the population threshold and others died out; this is not a general survival guarantee.
- One settlement has no destination for departing households. Food shortages can end a settlement.
- Selected smoke seeds, dashboard bands and long weather probes are plausibility and software checks, not proof of historical fidelity or comprehensive calibration.
- Simulation replay is not deterministic. World generation is reproducible for a seed and build; saves are snapshots of state.

For milestone requirements and detailed measurements, use the [project plan](../../PROJECT_PLAN.md), especially §7 and §9.
