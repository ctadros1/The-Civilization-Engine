# Research topics

170 deep-research topics across 16 domains, from the simulation engine to how each part of a society works. 165 are needed before v1 (M0–M8); the rest come after. Priorities: 100 High, 63 Medium, 7 Low.

## How to use

1. Pick the next topic from **By milestone** below: finish a milestone's High topics before that milestone starts.
2. Open the topic file and copy the prompt block into ChatGPT Pro deep research. Each prompt is self-contained.
3. Save the report next to the prompt as `<same-name>.report.md` and set the topic's **Status** to Done.
4. When a report changes a decision, update `PROJECT_PLAN.md` and its decisions log.

Status lives in each topic file, not in this index.

**Priority:** **High** means the milestone's core design depends on it. **Medium** improves fidelity or reduces risk and can be done during the milestone. **Low** is polish or post-v1.

**Report types:** *simulation* reports return mechanisms, parameters, era and regional variation, stylized facts and a modeling recommendation. *Engineering* reports compare options against TCE's hardware and architecture. *Design* reports study how existing games and tools solved the problem.

## By milestone

### M0: Foundations (6)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 01-01 | [Parallel simulation in Rust: throughput and correctness](01-engine/01-parallel-rust-sim.md) | Simulation engine | High |
| 01-02 | [Architecture of large-scale agent-based simulation engines](01-engine/02-abm-engine-architecture.md) | Simulation engine | High |
| 01-05 | [Save/load and schema evolution for long-lived worlds](01-engine/05-persistence-saves.md) | Simulation engine | High |
| 03-01 | [Procedural terrain with realistic geomorphology](03-world/01-terrain-generation.md) | World and environment | High |
| 01-10 | [Data-driven content systems and mod-style authoring](01-engine/10-data-driven-content.md) | Simulation engine | Medium |
| 01-11 | [Testing strategies for simulation code](01-engine/11-simulation-testing.md) | Simulation engine | Medium |

### M1: A band settles (30)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 01-08 | [Pathfinding and routing for tens of thousands of commuters](01-engine/08-routing-at-scale.md) | Simulation engine | High |
| 01-09 | [Decision architectures for thousands of agents: utility AI, GOAP, HTN](01-engine/09-decision-architectures.md) | Simulation engine | High |
| 02-01 | [SimCity (2013) GlassBox: lessons from an agent-based city sim](02-reference-sims/01-simcity-glassbox.md) | Lessons from existing simulations | High |
| 02-02 | [Cities: Skylines 1 and 2 simulation internals](02-reference-sims/02-cities-skylines.md) | Lessons from existing simulations | High |
| 03-02 | [Hydrology for settlement simulation: rivers, groundwater and floods](03-world/02-hydrology.md) | World and environment | High |
| 03-06 | [Ecology: vegetation, forests, wildlife and foraging yields](03-world/06-ecology-foraging.md) | World and environment | High |
| 04-01 | [Human needs and motivation models for simulated people](04-people/01-needs-motivation.md) | People: psychology, daily life and social behavior | High |
| 04-02 | [Daily routines and time use across history](04-people/02-time-use-daily-life.md) | People: psychology, daily life and social behavior | High |
| 04-03 | [Personality, values and ideology dimensions](04-people/03-personality-values.md) | People: psychology, daily life and social behavior | High |
| 04-04 | [Structure and dynamics of social networks](04-people/04-social-networks.md) | People: psychology, daily life and social behavior | High |
| 04-08 | [Marriage, household formation and life-course decisions](04-people/08-life-course-household.md) | People: psychology, daily life and social behavior | High |
| 05-01 | [Historical demography: fertility, mortality and population growth](05-demography-health/01-historical-demography.md) | Demography and health | High |
| 05-02 | [Nutrition, food requirements and famine dynamics](05-demography-health/02-nutrition-famine.md) | Demography and health | High |
| 06-01 | [Kinship systems and family structures](06-culture-society/01-kinship-systems.md) | Culture and society | High |
| 08-01 | [Economic life of foragers and early farmers](08-economy/01-forager-early-farming-economy.md) | Economy | High |
| 10-01 | [Settlement site selection and founding](10-settlements-urbanism/01-site-selection.md) | Settlements and urban form | High |
| 10-03 | [Organic urban morphology: how unplanned towns grow](10-settlements-urbanism/03-organic-urban-form.md) | Settlements and urban form | High |
| 10-09 | [Procedural city and parcel generation techniques](10-settlements-urbanism/09-procedural-city-generation.md) | Settlements and urban form | High |
| 11-03 | [Shape grammars and procedural architecture](11-architecture-construction/03-shape-grammars.md) | Architecture and construction | High |
| 15-01 | [Observer UX for complex simulations](15-observer-ux/01-observer-ux-patterns.md) | Observer experience and UI | High |
| 15-02 | [Automated chronicles and emergent narrative from event logs](15-observer-ux/02-chronicle-generation.md) | Observer experience and UI | High |
| 15-03 | [Explaining simulated decisions: the 'why' behind every outcome](15-observer-ux/03-decision-explanation.md) | Observer experience and UI | High |
| 01-12 | [Computational geometry for parcels, footprints and roofs](01-engine/12-computational-geometry.md) | Simulation engine | Medium |
| 02-03 | [Emergent societies and stories: Dwarf Fortress and RimWorld](02-reference-sims/03-dwarf-fortress-rimworld.md) | Lessons from existing simulations | Medium |
| 02-05 | [Settlement builders with individual citizens](02-reference-sims/05-settlement-builders.md) | Lessons from existing simulations | Medium |
| 02-07 | [Guided emergence: balancing authored structure and emergent behavior](02-reference-sims/07-guided-emergence.md) | Lessons from existing simulations | Medium |
| 04-07 | [Bounded rationality and plausible human decision-making](04-people/07-bounded-rationality.md) | People: psychology, daily life and social behavior | Medium |
| 04-12 | [Gender, age and the division of labor](04-people/12-gender-age-labor.md) | People: psychology, daily life and social behavior | Medium |
| 06-07 | [Personal and place names across cultures](06-culture-society/07-names.md) | Culture and society | Medium |
| 15-04 | [Designing data-dense panels for a simulation observer](15-observer-ux/04-data-dense-ui.md) | Observer experience and UI | Medium |

### M2: First light in Unreal (16)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 01-06 | [Embedding a Rust library in an Unreal Engine 5 plugin](01-engine/06-rust-unreal-ffi.md) | Simulation engine | High |
| 01-07 | [Streaming simulation state to a renderer](01-engine/07-state-streaming.md) | Simulation engine | High |
| 10-06 | [Housing typologies across cultures and eras](10-settlements-urbanism/06-housing-typologies.md) | Settlements and urban form | High |
| 11-01 | [A global taxonomy of vernacular architecture](11-architecture-construction/01-vernacular-typology.md) | Architecture and construction | High |
| 11-04 | [How buildings were built: labor, time and organization](11-architecture-construction/04-construction-process.md) | Architecture and construction | High |
| 14-01 | [Runtime-generated terrain in packaged UE5 builds](14-rendering/01-ue-runtime-terrain.md) | Unreal Engine rendering and pipeline | High |
| 14-02 | [Runtime assembly of Nanite modular kits with ISM/HISM](14-rendering/02-ue-runtime-instancing.md) | Unreal Engine rendering and pipeline | High |
| 14-03 | [Rendering 10k+ citizens in UE5](14-rendering/03-ue-crowds.md) | Unreal Engine rendering and pipeline | High |
| 14-04 | [Lumen, Nanite and VSM performance for dynamic city worlds](14-rendering/04-ue-lumen-nanite-perf.md) | Unreal Engine rendering and pipeline | High |
| 14-08 | [Modular kit authoring pipeline: Blender to Unreal](14-rendering/08-modular-kit-pipeline.md) | Unreal Engine rendering and pipeline | High |
| 14-10 | [Embedding web UI in Unreal Engine](14-rendering/10-web-ui-in-unreal.md) | Unreal Engine rendering and pipeline | High |
| 14-11 | [Automating Unreal Engine work for AI agents and CI](14-rendering/11-ue-automation.md) | Unreal Engine rendering and pipeline | High |
| 14-07 | [Runtime PCG driven by external simulation data](14-rendering/07-ue-runtime-pcg.md) | Unreal Engine rendering and pipeline | Medium |
| 14-09 | [Day/night, seasons and weather rendering in UE5](14-rendering/09-environment-rendering.md) | Unreal Engine rendering and pipeline | Medium |
| 14-12 | [Visualizing construction stages and collapses](14-rendering/12-construction-collapse-vfx.md) | Unreal Engine rendering and pipeline | Medium |
| 14-14 | [Rendering rivers and lakes from generated hydrology](14-rendering/14-water-rendering.md) | Unreal Engine rendering and pipeline | Medium |

### M3: Village economy (38)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 01-03 | [Multi-rate simulation and time acceleration](01-engine/03-multi-rate-time.md) | Simulation engine | High |
| 02-04 | [Grand strategy models: Victoria 3, Crusader Kings 3, Europa Universalis](02-reference-sims/04-paradox-systems.md) | Lessons from existing simulations | High |
| 03-03 | [Climate zones and stochastic weather generation](03-world/03-climate-weather.md) | World and environment | High |
| 03-04 | [Soils, fertility dynamics and agricultural land capability](03-world/04-soils-fertility.md) | World and environment | High |
| 04-05 | [Cultural evolution and social learning](04-people/05-cultural-transmission.md) | People: psychology, daily life and social behavior | High |
| 06-02 | [Social stratification and inequality: emergence and persistence](06-culture-society/02-stratification.md) | Culture and society | High |
| 06-09 | [Governing the commons: community institutions for shared resources](06-culture-society/09-commons-governance.md) | Culture and society | High |
| 07-01 | [How discoveries happen: theories of invention and innovation](07-technology/01-invention-theory.md) | Technology and knowledge | High |
| 07-02 | [Technology diffusion, adoption and loss](07-technology/02-technology-diffusion.md) | Technology and knowledge | High |
| 07-03 | [Designing a technology graph for a simulation](07-technology/03-tech-graph-design.md) | Technology and knowledge | High |
| 07-04 | [History of agricultural and food technology](07-technology/04-agriculture-tech.md) | Technology and knowledge | High |
| 07-05 | [History of materials: stone, metals, ceramics, glass, lime and concrete](07-technology/05-materials-metallurgy.md) | Technology and knowledge | High |
| 08-02 | [Pre-industrial agriculture in numbers](08-economy/02-agrarian-production.md) | Economy | High |
| 08-03 | [Craft production and workshops](08-economy/03-craft-production.md) | Economy | High |
| 08-04 | [Agent-based macroeconomic models](08-economy/04-abm-macroeconomics.md) | Economy | High |
| 08-05 | [Marketplaces and price formation in history](08-economy/05-markets-prices.md) | Economy | High |
| 08-06 | [Origins and history of money](08-economy/06-money-history.md) | Economy | High |
| 08-08 | [Business organization and firm dynamics](08-economy/08-firms-organization.md) | Economy | High |
| 08-09 | [Property rights regimes and their transitions](08-economy/09-property-regimes.md) | Economy | High |
| 08-10 | [Labor regimes and wages](08-economy/10-labor-regimes.md) | Economy | High |
| 08-15 | [Historical prices, wages and household budgets](08-economy/15-prices-budgets.md) | Economy | High |
| 11-05 | [Pre-modern structural engineering: spans, loads and rules of thumb](11-architecture-construction/05-structural-engineering-history.md) | Architecture and construction | High |
| 11-06 | [Building collapses and failures in history](11-architecture-construction/06-building-failures.md) | Architecture and construction | High |
| 11-08 | [Building materials: production, properties and durability](11-architecture-construction/08-building-materials.md) | Architecture and construction | High |
| 16-01 | [Stylized facts for validating a society simulation](16-validation/01-stylized-facts.md) | Validation and plausibility | High |
| 16-02 | [Historical baselines by era for sanity checks](16-validation/02-era-baselines.md) | Validation and plausibility | High |
| 03-05 | [Where natural resources are and how accessible they were](03-world/05-resource-geology.md) | World and environment | Medium |
| 04-13 | [Inheritance systems and their consequences](04-people/13-inheritance.md) | People: psychology, daily life and social behavior | Medium |
| 06-08 | [Education, apprenticeship and literacy](06-culture-society/08-education-literacy.md) | Culture and society | Medium |
| 07-06 | [History of energy and mechanical power](07-technology/06-energy-power.md) | Technology and knowledge | Medium |
| 07-07 | [Textiles, clothing, leather and woodworking](07-technology/07-textiles-crafts.md) | Technology and knowledge | Medium |
| 08-14 | [Wealth and income distribution dynamics](08-economy/14-inequality-dynamics.md) | Economy | Medium |
| 08-17 | [Guilds: economic, political and knowledge institutions](08-economy/17-guilds.md) | Economy | Medium |
| 08-21 | [Classifying economic systems](08-economy/21-economic-system-typology.md) | Economy | Medium |
| 11-02 | [How architectural styles evolve and spread](11-architecture-construction/02-style-evolution.md) | Architecture and construction | Medium |
| 11-12 | [Earthworks: terraces, canals, levees and leveling](11-architecture-construction/12-earthworks.md) | Architecture and construction | Medium |
| 14-06 | [Condition-driven weathering and aging materials](14-rendering/06-procedural-weathering.md) | Unreal Engine rendering and pipeline | Medium |
| 16-03 | [Validating agent-based models without a research program](16-validation/03-abm-validation.md) | Validation and plausibility | Medium |

### M4: Councils, law & crime (29)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 04-09 | [Why people commit crimes: criminology for simulation](04-people/09-criminology.md) | People: psychology, daily life and social behavior | High |
| 04-10 | [Collective action, protest and riots](04-people/10-collective-action.md) | People: psychology, daily life and social behavior | High |
| 04-11 | [How leaders and elites emerge](04-people/11-leadership-elites.md) | People: psychology, daily life and social behavior | High |
| 06-04 | [Ideology formation and opinion dynamics](06-culture-society/04-ideology-opinion-dynamics.md) | Culture and society | High |
| 06-05 | [Social norms: emergence, enforcement and change](06-culture-society/05-social-norms.md) | Culture and society | High |
| 08-13 | [Taxation and public finance history](08-economy/13-taxation.md) | Economy | High |
| 09-01 | [State formation and political evolution](09-government-politics/01-state-formation.md) | Government, law and politics | High |
| 09-02 | [Decomposing governments into primitives](09-government-politics/02-constitutional-primitives.md) | Government, law and politics | High |
| 09-03 | [Succession rules and succession crises](09-government-politics/03-succession.md) | Government, law and politics | High |
| 09-04 | [Legitimacy and authority](09-government-politics/04-legitimacy.md) | Government, law and politics | High |
| 09-06 | [State capacity, enforcement and compliance](09-government-politics/06-enforcement-compliance.md) | Government, law and politics | High |
| 09-07 | [Justice systems, courts and punishment](09-government-politics/07-justice-courts.md) | Government, law and politics | High |
| 09-09 | [Corruption, patronage and clientelism](09-government-politics/09-corruption.md) | Government, law and politics | High |
| 09-11 | [Revolutions, coups and regime change](09-government-politics/11-revolutions-coups.md) | Government, law and politics | High |
| 09-15 | [Classifying political regimes](09-government-politics/15-regime-typology.md) | Government, law and politics | High |
| 12-04 | [Policing and public order](12-services-infrastructure/04-policing.md) | City services and infrastructure | High |
| 02-06 | [Academic simulations of societies and states](02-reference-sims/06-academic-society-sims.md) | Lessons from existing simulations | Medium |
| 04-06 | [Reputation, trust, reciprocity and grievance](04-people/06-reputation-trust.md) | People: psychology, daily life and social behavior | Medium |
| 06-03 | [Religion and ritual as social institutions](06-culture-society/03-religion.md) | Culture and society | Medium |
| 06-10 | [Presenting dark historical content in simulations](06-culture-society/10-dark-content.md) | Culture and society | Medium |
| 07-09 | [Writing, record-keeping, accounting, printing and communications](07-technology/09-information-tech.md) | Technology and knowledge | Medium |
| 07-11 | [Institutions of knowledge: guilds, schools, academies and patronage](07-technology/11-knowledge-institutions.md) | Technology and knowledge | Medium |
| 08-11 | [Allocation without markets: redistribution, gift economies and planning](08-economy/11-non-market-allocation.md) | Economy | Medium |
| 08-18 | [Informal and illegal economies](08-economy/18-informal-economy.md) | Economy | Medium |
| 09-05 | [How laws are made: decrees, councils, assemblies and legislatures](09-government-politics/05-lawmaking.md) | Government, law and politics | Medium |
| 09-14 | [Urban governance: communes, charters and city councils](09-government-politics/14-municipal-governance.md) | Government, law and politics | Medium |
| 09-16 | [How news, rumors and laws spread in pre-modern societies](09-government-politics/16-information-flow.md) | Government, law and politics | Medium |
| 09-17 | [Poor relief, charity and welfare](09-government-politics/17-welfare-relief.md) | Government, law and politics | Medium |
| 15-05 | [Designing indirect god-game interventions](15-observer-ux/05-god-game-design.md) | Observer experience and UI | Medium |

### M5: Neighbors (11)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 05-06 | [Migration: why and how people move](05-demography-health/06-migration.md) | Demography and health | High |
| 07-08 | [Transport and navigation technology](07-technology/08-transport-tech.md) | Technology and knowledge | High |
| 08-12 | [Long-distance trade and merchant networks](08-economy/12-trade-merchants.md) | Economy | High |
| 08-16 | [Feeding and fueling cities: provisioning and hinterlands](08-economy/16-urban-provisioning.md) | Economy | High |
| 11-07 | [Bridge engineering through history](11-architecture-construction/07-bridges.md) | Architecture and construction | High |
| 13-01 | [Diplomacy and inter-polity relations](13-diplomacy-war/01-diplomacy.md) | Diplomacy and war | High |
| 13-02 | [Tributary and vassal systems](13-diplomacy-war/02-tributary-vassal.md) | Diplomacy and war | High |
| 13-05 | [Causes of war and peace between polities](13-diplomacy-war/05-war-causes.md) | Diplomacy and war | High |
| 10-02 | [Settlement growth, hierarchies and urban scaling](10-settlements-urbanism/02-urban-scaling.md) | Settlements and urban form | Medium |
| 11-11 | [Fortifications: forms, costs and effectiveness](11-architecture-construction/11-fortifications.md) | Architecture and construction | Medium |
| 06-06 | [Visual culture: emblems, flags, heraldry, clothing and ornament](06-culture-society/06-visual-culture.md) | Culture and society | Low |

### M6: Towns & their troubles (20)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 05-03 | [Epidemiology of historical diseases](05-demography-health/03-disease-epidemiology.md) | Demography and health | High |
| 05-04 | [The urban mortality penalty and sanitation history](05-demography-health/04-urban-penalty-sanitation.md) | Demography and health | High |
| 12-01 | [Water supply systems through history](12-services-infrastructure/01-water-supply.md) | City services and infrastructure | High |
| 12-02 | [Sanitation and waste management history](12-services-infrastructure/02-sanitation-waste.md) | City services and infrastructure | High |
| 12-03 | [Urban fires and firefighting](12-services-infrastructure/03-urban-fire.md) | City services and infrastructure | High |
| 13-03 | [Conquest, occupation and integration of territories](13-diplomacy-war/03-conquest-integration.md) | Diplomacy and war | High |
| 13-06 | [Military organization and logistics](13-diplomacy-war/06-military-logistics.md) | Diplomacy and war | High |
| 13-07 | [Modeling battles and casualties](13-diplomacy-war/07-battle-modeling.md) | Diplomacy and war | High |
| 03-07 | [Natural hazards: frequency, magnitude and impact on settlements](03-world/07-natural-hazards.md) | World and environment | Medium |
| 05-05 | [Healing, medicine and health institutions](05-demography-health/05-medicine-healing.md) | Demography and health | Medium |
| 07-10 | [Military technology through history](07-technology/10-military-tech.md) | Technology and knowledge | Medium |
| 09-13 | [Rebellion, civil war and secession](09-government-politics/13-civil-conflict-secession.md) | Government, law and politics | Medium |
| 11-09 | [Building codes and regulation](11-architecture-construction/09-building-codes.md) | Architecture and construction | Medium |
| 12-07 | [Emergency dispatch and service coverage models](12-services-infrastructure/07-emergency-response.md) | City services and infrastructure | Medium |
| 12-08 | [Public works: organization, financing and maintenance](12-services-infrastructure/08-public-works.md) | City services and infrastructure | Medium |
| 13-04 | [Leagues, confederations and unions](13-diplomacy-war/04-federations.md) | Diplomacy and war | Medium |
| 13-08 | [Siege warfare](13-diplomacy-war/08-sieges.md) | Diplomacy and war | Medium |
| 13-09 | [Social and economic consequences of war](13-diplomacy-war/09-war-consequences.md) | Diplomacy and war | Medium |
| 14-05 | [Interior mapping and fake interiors](14-rendering/05-interior-mapping.md) | Unreal Engine rendering and pipeline | Medium |
| 11-13 | [Interior layouts and room use](11-architecture-construction/13-interior-programs.md) | Architecture and construction | Low |

### M7: Politics deepens (5)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 09-08 | [Factions, parties, voting and elections](09-government-politics/08-parties-elections.md) | Government, law and politics | High |
| 08-07 | [Credit, interest and banking before industrialization](08-economy/07-credit-banking.md) | Economy | Medium |
| 08-20 | [Pre-industrial economic crises](08-economy/20-economic-crises.md) | Economy | Medium |
| 09-10 | [Lobbying, rent-seeking and institutional capture](09-government-politics/10-lobbying-capture.md) | Government, law and politics | Medium |
| 09-12 | [Cliodynamics: secular cycles and elite overproduction](09-government-politics/12-structural-demographic.md) | Government, law and politics | Medium |

### M8: The growing city (v1) (10)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 10-04 | [Planned cities and planning traditions](10-settlements-urbanism/04-planned-cities.md) | Settlements and urban form | High |
| 12-05 | [Urban mobility and traffic in pre-industrial cities](12-services-infrastructure/05-urban-mobility-history.md) | City services and infrastructure | High |
| 12-06 | [Traffic flow and assignment models for simulations](12-services-infrastructure/06-traffic-modeling.md) | City services and infrastructure | High |
| 08-19 | [Housing markets, land value and rents](08-economy/19-housing-land-rent.md) | Economy | Medium |
| 10-05 | [Land use patterns and zoning history](10-settlements-urbanism/05-land-use.md) | Settlements and urban form | Medium |
| 10-07 | [Streets and roads: design and evolution](10-settlements-urbanism/07-streets-roads.md) | Settlements and urban form | Medium |
| 10-08 | [Urban density and crowding across history](10-settlements-urbanism/08-density-crowding.md) | Settlements and urban form | Medium |
| 11-10 | [Monumental architecture: why and how societies build big](11-architecture-construction/10-monumental-architecture.md) | Architecture and construction | Medium |
| 14-13 | [Rendering runtime-generated road networks](14-rendering/13-road-rendering.md) | Unreal Engine rendering and pipeline | Medium |
| 14-15 | [City soundscapes and procedural ambient audio](14-rendering/15-city-soundscape.md) | Unreal Engine rendering and pipeline | Low |

### M9: Aggregate LOD & scale (1)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 01-04 | [Aggregate cohort models and promoting/demoting agents](01-engine/04-aggregate-lod.md) | Simulation engine | Medium |

### M10: Industrial content (1)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 07-12 | [Preconditions and cluster of the Industrial Revolution](07-technology/12-industrialization.md) | Technology and knowledge | Low |

### M11: Modern content (2)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 07-13 | [Modern technology: electricity, combustion and telecommunications](07-technology/13-modern-tech.md) | Technology and knowledge | Low |
| 12-09 | [Electric power, gas and modern utilities](12-services-infrastructure/09-modern-utilities.md) | City services and infrastructure | Low |

### M12: LLM notables, multi-viewer, close-up fidelity (1)

| ID | Topic | Domain | Priority |
|---|---|---|---|
| 01-13 | [LLM-driven agents in large simulations](01-engine/13-llm-agents.md) | Simulation engine | Low |

## By domain

### 01. Simulation engine (13)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 01-01 | [Parallel simulation in Rust: throughput and correctness](01-engine/01-parallel-rust-sim.md) | M0 | High | Engineering |
| 01-02 | [Architecture of large-scale agent-based simulation engines](01-engine/02-abm-engine-architecture.md) | M0 | High | Engineering |
| 01-03 | [Multi-rate simulation and time acceleration](01-engine/03-multi-rate-time.md) | M3 | High | Engineering |
| 01-04 | [Aggregate cohort models and promoting/demoting agents](01-engine/04-aggregate-lod.md) | M9 | Medium | Engineering |
| 01-05 | [Save/load and schema evolution for long-lived worlds](01-engine/05-persistence-saves.md) | M0 | High | Engineering |
| 01-06 | [Embedding a Rust library in an Unreal Engine 5 plugin](01-engine/06-rust-unreal-ffi.md) | M2 | High | Engineering |
| 01-07 | [Streaming simulation state to a renderer](01-engine/07-state-streaming.md) | M2 | High | Engineering |
| 01-08 | [Pathfinding and routing for tens of thousands of commuters](01-engine/08-routing-at-scale.md) | M1 → M8 | High | Engineering |
| 01-09 | [Decision architectures for thousands of agents: utility AI, GOAP, HTN](01-engine/09-decision-architectures.md) | M1 | High | Engineering |
| 01-10 | [Data-driven content systems and mod-style authoring](01-engine/10-data-driven-content.md) | M0 | Medium | Engineering |
| 01-11 | [Testing strategies for simulation code](01-engine/11-simulation-testing.md) | M0 | Medium | Engineering |
| 01-12 | [Computational geometry for parcels, footprints and roofs](01-engine/12-computational-geometry.md) | M1 | Medium | Engineering |
| 01-13 | [LLM-driven agents in large simulations](01-engine/13-llm-agents.md) | M12 | Low | Engineering |

### 02. Lessons from existing simulations (7)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 02-01 | [SimCity (2013) GlassBox: lessons from an agent-based city sim](02-reference-sims/01-simcity-glassbox.md) | M1 | High | Design |
| 02-02 | [Cities: Skylines 1 and 2 simulation internals](02-reference-sims/02-cities-skylines.md) | M1 | High | Design |
| 02-03 | [Emergent societies and stories: Dwarf Fortress and RimWorld](02-reference-sims/03-dwarf-fortress-rimworld.md) | M1 | Medium | Design |
| 02-04 | [Grand strategy models: Victoria 3, Crusader Kings 3, Europa Universalis](02-reference-sims/04-paradox-systems.md) | M3 | High | Design |
| 02-05 | [Settlement builders with individual citizens](02-reference-sims/05-settlement-builders.md) | M1 | Medium | Design |
| 02-06 | [Academic simulations of societies and states](02-reference-sims/06-academic-society-sims.md) | M4 | Medium | Design |
| 02-07 | [Guided emergence: balancing authored structure and emergent behavior](02-reference-sims/07-guided-emergence.md) | M1 | Medium | Design |

### 03. World and environment (7)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 03-01 | [Procedural terrain with realistic geomorphology](03-world/01-terrain-generation.md) | M0 | High | Engineering |
| 03-02 | [Hydrology for settlement simulation: rivers, groundwater and floods](03-world/02-hydrology.md) | M1 → M6 | High | Simulation |
| 03-03 | [Climate zones and stochastic weather generation](03-world/03-climate-weather.md) | M3 | High | Simulation |
| 03-04 | [Soils, fertility dynamics and agricultural land capability](03-world/04-soils-fertility.md) | M3 | High | Simulation |
| 03-05 | [Where natural resources are and how accessible they were](03-world/05-resource-geology.md) | M3 | Medium | Simulation |
| 03-06 | [Ecology: vegetation, forests, wildlife and foraging yields](03-world/06-ecology-foraging.md) | M1 | High | Simulation |
| 03-07 | [Natural hazards: frequency, magnitude and impact on settlements](03-world/07-natural-hazards.md) | M6 | Medium | Simulation |

### 04. People: psychology, daily life and social behavior (13)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 04-01 | [Human needs and motivation models for simulated people](04-people/01-needs-motivation.md) | M1 | High | Simulation |
| 04-02 | [Daily routines and time use across history](04-people/02-time-use-daily-life.md) | M1 | High | Simulation |
| 04-03 | [Personality, values and ideology dimensions](04-people/03-personality-values.md) | M1 | High | Simulation |
| 04-04 | [Structure and dynamics of social networks](04-people/04-social-networks.md) | M1 | High | Simulation |
| 04-05 | [Cultural evolution and social learning](04-people/05-cultural-transmission.md) | M3 | High | Simulation |
| 04-06 | [Reputation, trust, reciprocity and grievance](04-people/06-reputation-trust.md) | M4 | Medium | Simulation |
| 04-07 | [Bounded rationality and plausible human decision-making](04-people/07-bounded-rationality.md) | M1 | Medium | Simulation |
| 04-08 | [Marriage, household formation and life-course decisions](04-people/08-life-course-household.md) | M1 | High | Simulation |
| 04-09 | [Why people commit crimes: criminology for simulation](04-people/09-criminology.md) | M4 | High | Simulation |
| 04-10 | [Collective action, protest and riots](04-people/10-collective-action.md) | M4 | High | Simulation |
| 04-11 | [How leaders and elites emerge](04-people/11-leadership-elites.md) | M4 | High | Simulation |
| 04-12 | [Gender, age and the division of labor](04-people/12-gender-age-labor.md) | M1 | Medium | Simulation |
| 04-13 | [Inheritance systems and their consequences](04-people/13-inheritance.md) | M3 | Medium | Simulation |

### 05. Demography and health (6)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 05-01 | [Historical demography: fertility, mortality and population growth](05-demography-health/01-historical-demography.md) | M1 | High | Simulation |
| 05-02 | [Nutrition, food requirements and famine dynamics](05-demography-health/02-nutrition-famine.md) | M1 | High | Simulation |
| 05-03 | [Epidemiology of historical diseases](05-demography-health/03-disease-epidemiology.md) | M6 | High | Simulation |
| 05-04 | [The urban mortality penalty and sanitation history](05-demography-health/04-urban-penalty-sanitation.md) | M6 | High | Simulation |
| 05-05 | [Healing, medicine and health institutions](05-demography-health/05-medicine-healing.md) | M6 → M10 | Medium | Simulation |
| 05-06 | [Migration: why and how people move](05-demography-health/06-migration.md) | M5 | High | Simulation |

### 06. Culture and society (10)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 06-01 | [Kinship systems and family structures](06-culture-society/01-kinship-systems.md) | M1 | High | Simulation |
| 06-02 | [Social stratification and inequality: emergence and persistence](06-culture-society/02-stratification.md) | M3 | High | Simulation |
| 06-03 | [Religion and ritual as social institutions](06-culture-society/03-religion.md) | M4 | Medium | Simulation |
| 06-04 | [Ideology formation and opinion dynamics](06-culture-society/04-ideology-opinion-dynamics.md) | M4 | High | Simulation |
| 06-05 | [Social norms: emergence, enforcement and change](06-culture-society/05-social-norms.md) | M4 | High | Simulation |
| 06-06 | [Visual culture: emblems, flags, heraldry, clothing and ornament](06-culture-society/06-visual-culture.md) | M5 | Low | Simulation |
| 06-07 | [Personal and place names across cultures](06-culture-society/07-names.md) | M1 | Medium | Simulation |
| 06-08 | [Education, apprenticeship and literacy](06-culture-society/08-education-literacy.md) | M3 → M10 | Medium | Simulation |
| 06-09 | [Governing the commons: community institutions for shared resources](06-culture-society/09-commons-governance.md) | M3 | High | Simulation |
| 06-10 | [Presenting dark historical content in simulations](06-culture-society/10-dark-content.md) | M4 | Medium | Design |

### 07. Technology and knowledge (13)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 07-01 | [How discoveries happen: theories of invention and innovation](07-technology/01-invention-theory.md) | M3 | High | Simulation |
| 07-02 | [Technology diffusion, adoption and loss](07-technology/02-technology-diffusion.md) | M3 | High | Simulation |
| 07-03 | [Designing a technology graph for a simulation](07-technology/03-tech-graph-design.md) | M3 | High | Design |
| 07-04 | [History of agricultural and food technology](07-technology/04-agriculture-tech.md) | M3 | High | Simulation |
| 07-05 | [History of materials: stone, metals, ceramics, glass, lime and concrete](07-technology/05-materials-metallurgy.md) | M3 | High | Simulation |
| 07-06 | [History of energy and mechanical power](07-technology/06-energy-power.md) | M3 → M10 | Medium | Simulation |
| 07-07 | [Textiles, clothing, leather and woodworking](07-technology/07-textiles-crafts.md) | M3 | Medium | Simulation |
| 07-08 | [Transport and navigation technology](07-technology/08-transport-tech.md) | M5 | High | Simulation |
| 07-09 | [Writing, record-keeping, accounting, printing and communications](07-technology/09-information-tech.md) | M4 | Medium | Simulation |
| 07-10 | [Military technology through history](07-technology/10-military-tech.md) | M6 | Medium | Simulation |
| 07-11 | [Institutions of knowledge: guilds, schools, academies and patronage](07-technology/11-knowledge-institutions.md) | M4 | Medium | Simulation |
| 07-12 | [Preconditions and cluster of the Industrial Revolution](07-technology/12-industrialization.md) | M10 | Low | Simulation |
| 07-13 | [Modern technology: electricity, combustion and telecommunications](07-technology/13-modern-tech.md) | M11 | Low | Simulation |

### 08. Economy (21)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 08-01 | [Economic life of foragers and early farmers](08-economy/01-forager-early-farming-economy.md) | M1 | High | Simulation |
| 08-02 | [Pre-industrial agriculture in numbers](08-economy/02-agrarian-production.md) | M3 | High | Simulation |
| 08-03 | [Craft production and workshops](08-economy/03-craft-production.md) | M3 | High | Simulation |
| 08-04 | [Agent-based macroeconomic models](08-economy/04-abm-macroeconomics.md) | M3 | High | Engineering |
| 08-05 | [Marketplaces and price formation in history](08-economy/05-markets-prices.md) | M3 | High | Simulation |
| 08-06 | [Origins and history of money](08-economy/06-money-history.md) | M3 | High | Simulation |
| 08-07 | [Credit, interest and banking before industrialization](08-economy/07-credit-banking.md) | M7 | Medium | Simulation |
| 08-08 | [Business organization and firm dynamics](08-economy/08-firms-organization.md) | M3 | High | Simulation |
| 08-09 | [Property rights regimes and their transitions](08-economy/09-property-regimes.md) | M3 | High | Simulation |
| 08-10 | [Labor regimes and wages](08-economy/10-labor-regimes.md) | M3 | High | Simulation |
| 08-11 | [Allocation without markets: redistribution, gift economies and planning](08-economy/11-non-market-allocation.md) | M4 | Medium | Simulation |
| 08-12 | [Long-distance trade and merchant networks](08-economy/12-trade-merchants.md) | M5 | High | Simulation |
| 08-13 | [Taxation and public finance history](08-economy/13-taxation.md) | M4 | High | Simulation |
| 08-14 | [Wealth and income distribution dynamics](08-economy/14-inequality-dynamics.md) | M3 | Medium | Simulation |
| 08-15 | [Historical prices, wages and household budgets](08-economy/15-prices-budgets.md) | M3 | High | Simulation |
| 08-16 | [Feeding and fueling cities: provisioning and hinterlands](08-economy/16-urban-provisioning.md) | M5 | High | Simulation |
| 08-17 | [Guilds: economic, political and knowledge institutions](08-economy/17-guilds.md) | M3 | Medium | Simulation |
| 08-18 | [Informal and illegal economies](08-economy/18-informal-economy.md) | M4 | Medium | Simulation |
| 08-19 | [Housing markets, land value and rents](08-economy/19-housing-land-rent.md) | M8 | Medium | Simulation |
| 08-20 | [Pre-industrial economic crises](08-economy/20-economic-crises.md) | M7 | Medium | Simulation |
| 08-21 | [Classifying economic systems](08-economy/21-economic-system-typology.md) | M3 | Medium | Simulation |

### 09. Government, law and politics (17)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 09-01 | [State formation and political evolution](09-government-politics/01-state-formation.md) | M4 | High | Simulation |
| 09-02 | [Decomposing governments into primitives](09-government-politics/02-constitutional-primitives.md) | M4 | High | Simulation |
| 09-03 | [Succession rules and succession crises](09-government-politics/03-succession.md) | M4 | High | Simulation |
| 09-04 | [Legitimacy and authority](09-government-politics/04-legitimacy.md) | M4 | High | Simulation |
| 09-05 | [How laws are made: decrees, councils, assemblies and legislatures](09-government-politics/05-lawmaking.md) | M4 | Medium | Simulation |
| 09-06 | [State capacity, enforcement and compliance](09-government-politics/06-enforcement-compliance.md) | M4 | High | Simulation |
| 09-07 | [Justice systems, courts and punishment](09-government-politics/07-justice-courts.md) | M4 → M7 | High | Simulation |
| 09-08 | [Factions, parties, voting and elections](09-government-politics/08-parties-elections.md) | M7 | High | Simulation |
| 09-09 | [Corruption, patronage and clientelism](09-government-politics/09-corruption.md) | M4 | High | Simulation |
| 09-10 | [Lobbying, rent-seeking and institutional capture](09-government-politics/10-lobbying-capture.md) | M7 | Medium | Simulation |
| 09-11 | [Revolutions, coups and regime change](09-government-politics/11-revolutions-coups.md) | M4 | High | Simulation |
| 09-12 | [Cliodynamics: secular cycles and elite overproduction](09-government-politics/12-structural-demographic.md) | M7 | Medium | Simulation |
| 09-13 | [Rebellion, civil war and secession](09-government-politics/13-civil-conflict-secession.md) | M6 | Medium | Simulation |
| 09-14 | [Urban governance: communes, charters and city councils](09-government-politics/14-municipal-governance.md) | M4 | Medium | Simulation |
| 09-15 | [Classifying political regimes](09-government-politics/15-regime-typology.md) | M4 | High | Simulation |
| 09-16 | [How news, rumors and laws spread in pre-modern societies](09-government-politics/16-information-flow.md) | M4 | Medium | Simulation |
| 09-17 | [Poor relief, charity and welfare](09-government-politics/17-welfare-relief.md) | M4 | Medium | Simulation |

### 10. Settlements and urban form (9)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 10-01 | [Settlement site selection and founding](10-settlements-urbanism/01-site-selection.md) | M1 | High | Simulation |
| 10-02 | [Settlement growth, hierarchies and urban scaling](10-settlements-urbanism/02-urban-scaling.md) | M5 | Medium | Simulation |
| 10-03 | [Organic urban morphology: how unplanned towns grow](10-settlements-urbanism/03-organic-urban-form.md) | M1 | High | Simulation |
| 10-04 | [Planned cities and planning traditions](10-settlements-urbanism/04-planned-cities.md) | M8 | High | Simulation |
| 10-05 | [Land use patterns and zoning history](10-settlements-urbanism/05-land-use.md) | M8 | Medium | Simulation |
| 10-06 | [Housing typologies across cultures and eras](10-settlements-urbanism/06-housing-typologies.md) | M2 | High | Simulation |
| 10-07 | [Streets and roads: design and evolution](10-settlements-urbanism/07-streets-roads.md) | M8 | Medium | Simulation |
| 10-08 | [Urban density and crowding across history](10-settlements-urbanism/08-density-crowding.md) | M8 | Medium | Simulation |
| 10-09 | [Procedural city and parcel generation techniques](10-settlements-urbanism/09-procedural-city-generation.md) | M1 | High | Engineering |

### 11. Architecture and construction (13)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 11-01 | [A global taxonomy of vernacular architecture](11-architecture-construction/01-vernacular-typology.md) | M2 | High | Simulation |
| 11-02 | [How architectural styles evolve and spread](11-architecture-construction/02-style-evolution.md) | M3 | Medium | Simulation |
| 11-03 | [Shape grammars and procedural architecture](11-architecture-construction/03-shape-grammars.md) | M1 | High | Engineering |
| 11-04 | [How buildings were built: labor, time and organization](11-architecture-construction/04-construction-process.md) | M2 | High | Simulation |
| 11-05 | [Pre-modern structural engineering: spans, loads and rules of thumb](11-architecture-construction/05-structural-engineering-history.md) | M3 | High | Simulation |
| 11-06 | [Building collapses and failures in history](11-architecture-construction/06-building-failures.md) | M3 | High | Simulation |
| 11-07 | [Bridge engineering through history](11-architecture-construction/07-bridges.md) | M5 | High | Simulation |
| 11-08 | [Building materials: production, properties and durability](11-architecture-construction/08-building-materials.md) | M3 | High | Simulation |
| 11-09 | [Building codes and regulation](11-architecture-construction/09-building-codes.md) | M6 | Medium | Simulation |
| 11-10 | [Monumental architecture: why and how societies build big](11-architecture-construction/10-monumental-architecture.md) | M8 | Medium | Simulation |
| 11-11 | [Fortifications: forms, costs and effectiveness](11-architecture-construction/11-fortifications.md) | M5 | Medium | Simulation |
| 11-12 | [Earthworks: terraces, canals, levees and leveling](11-architecture-construction/12-earthworks.md) | M3 | Medium | Simulation |
| 11-13 | [Interior layouts and room use](11-architecture-construction/13-interior-programs.md) | M6 | Low | Simulation |

### 12. City services and infrastructure (9)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 12-01 | [Water supply systems through history](12-services-infrastructure/01-water-supply.md) | M6 | High | Simulation |
| 12-02 | [Sanitation and waste management history](12-services-infrastructure/02-sanitation-waste.md) | M6 | High | Simulation |
| 12-03 | [Urban fires and firefighting](12-services-infrastructure/03-urban-fire.md) | M6 | High | Simulation |
| 12-04 | [Policing and public order](12-services-infrastructure/04-policing.md) | M4 | High | Simulation |
| 12-05 | [Urban mobility and traffic in pre-industrial cities](12-services-infrastructure/05-urban-mobility-history.md) | M8 | High | Simulation |
| 12-06 | [Traffic flow and assignment models for simulations](12-services-infrastructure/06-traffic-modeling.md) | M8 | High | Engineering |
| 12-07 | [Emergency dispatch and service coverage models](12-services-infrastructure/07-emergency-response.md) | M6 | Medium | Simulation |
| 12-08 | [Public works: organization, financing and maintenance](12-services-infrastructure/08-public-works.md) | M6 | Medium | Simulation |
| 12-09 | [Electric power, gas and modern utilities](12-services-infrastructure/09-modern-utilities.md) | M11 | Low | Simulation |

### 13. Diplomacy and war (9)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 13-01 | [Diplomacy and inter-polity relations](13-diplomacy-war/01-diplomacy.md) | M5 | High | Simulation |
| 13-02 | [Tributary and vassal systems](13-diplomacy-war/02-tributary-vassal.md) | M5 | High | Simulation |
| 13-03 | [Conquest, occupation and integration of territories](13-diplomacy-war/03-conquest-integration.md) | M6 | High | Simulation |
| 13-04 | [Leagues, confederations and unions](13-diplomacy-war/04-federations.md) | M6 | Medium | Simulation |
| 13-05 | [Causes of war and peace between polities](13-diplomacy-war/05-war-causes.md) | M5 | High | Simulation |
| 13-06 | [Military organization and logistics](13-diplomacy-war/06-military-logistics.md) | M6 | High | Simulation |
| 13-07 | [Modeling battles and casualties](13-diplomacy-war/07-battle-modeling.md) | M6 | High | Simulation |
| 13-08 | [Siege warfare](13-diplomacy-war/08-sieges.md) | M6 | Medium | Simulation |
| 13-09 | [Social and economic consequences of war](13-diplomacy-war/09-war-consequences.md) | M6 | Medium | Simulation |

### 14. Unreal Engine rendering and pipeline (15)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 14-01 | [Runtime-generated terrain in packaged UE5 builds](14-rendering/01-ue-runtime-terrain.md) | M2 | High | Engineering |
| 14-02 | [Runtime assembly of Nanite modular kits with ISM/HISM](14-rendering/02-ue-runtime-instancing.md) | M2 | High | Engineering |
| 14-03 | [Rendering 10k+ citizens in UE5](14-rendering/03-ue-crowds.md) | M2 → M8 | High | Engineering |
| 14-04 | [Lumen, Nanite and VSM performance for dynamic city worlds](14-rendering/04-ue-lumen-nanite-perf.md) | M2 | High | Engineering |
| 14-05 | [Interior mapping and fake interiors](14-rendering/05-interior-mapping.md) | M6 | Medium | Engineering |
| 14-06 | [Condition-driven weathering and aging materials](14-rendering/06-procedural-weathering.md) | M3 | Medium | Engineering |
| 14-07 | [Runtime PCG driven by external simulation data](14-rendering/07-ue-runtime-pcg.md) | M2 | Medium | Engineering |
| 14-08 | [Modular kit authoring pipeline: Blender to Unreal](14-rendering/08-modular-kit-pipeline.md) | M2 | High | Engineering |
| 14-09 | [Day/night, seasons and weather rendering in UE5](14-rendering/09-environment-rendering.md) | M2 | Medium | Engineering |
| 14-10 | [Embedding web UI in Unreal Engine](14-rendering/10-web-ui-in-unreal.md) | M2 | High | Engineering |
| 14-11 | [Automating Unreal Engine work for AI agents and CI](14-rendering/11-ue-automation.md) | M2 | High | Engineering |
| 14-12 | [Visualizing construction stages and collapses](14-rendering/12-construction-collapse-vfx.md) | M2 | Medium | Engineering |
| 14-13 | [Rendering runtime-generated road networks](14-rendering/13-road-rendering.md) | M8 | Medium | Engineering |
| 14-14 | [Rendering rivers and lakes from generated hydrology](14-rendering/14-water-rendering.md) | M2 | Medium | Engineering |
| 14-15 | [City soundscapes and procedural ambient audio](14-rendering/15-city-soundscape.md) | M8 | Low | Engineering |

### 15. Observer experience and UI (5)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 15-01 | [Observer UX for complex simulations](15-observer-ux/01-observer-ux-patterns.md) | M1 | High | Design |
| 15-02 | [Automated chronicles and emergent narrative from event logs](15-observer-ux/02-chronicle-generation.md) | M1 | High | Design |
| 15-03 | [Explaining simulated decisions: the 'why' behind every outcome](15-observer-ux/03-decision-explanation.md) | M1 | High | Design |
| 15-04 | [Designing data-dense panels for a simulation observer](15-observer-ux/04-data-dense-ui.md) | M1 | Medium | Design |
| 15-05 | [Designing indirect god-game interventions](15-observer-ux/05-god-game-design.md) | M4 | Medium | Design |

### 16. Validation and plausibility (3)

| ID | Topic | Needed by | Priority | Type |
|---|---|---|---|---|
| 16-01 | [Stylized facts for validating a society simulation](16-validation/01-stylized-facts.md) | M3 | High | Simulation |
| 16-02 | [Historical baselines by era for sanity checks](16-validation/02-era-baselines.md) | M3 | High | Simulation |
| 16-03 | [Validating agent-based models without a research program](16-validation/03-abm-validation.md) | M3 | Medium | Engineering |
