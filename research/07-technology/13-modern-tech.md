# Modern technology for TCE: electricity, combustion, and telecommunications

**Model these technologies as three interacting infrastructure systems—power delivery, powered mobility, and information transmission—not as a single “modern era” unlock.** Their effects should emerge from the equipment people can obtain, the networks they can reach, the services those networks actually deliver, and the institutions that finance and operate them.

The essential distinction is between **knowledge, manufacturing capability, installed equipment, service availability, and actual use**. A society can import generators without manufacturing them; install electrical connections without providing dependable power; operate telephones without electrifying homes; or own vehicles whose usefulness is constrained by fuel shortages and impassable roads. Historical electrification research strongly supports separating infrastructure installation from complementary equipment, organizational change, and economic outcomes. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/from-shafts-to-wires-historical-perspective-on-electrification/500078D9B4764BA1109A7967437CF226)

Throughout this report, **historical observations and engineering specifications are distinguished from proposed TCE rules**. Dates are documented prototypes, patents, or deployment landmarks—not universally uncontested “first inventions,” and never recommended era gates.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Discovery, production, deployment, and adoption must be separate

For each technology, track at least these independently:

| Layer | What TCE should record | Example |
| --- | --- | --- |
| Knowledge | People and institutions able to explain, demonstrate, or reproduce a technique | An engineer understands transformer construction |
| Production capability | Workshops, inputs, tolerances, and skilled labor needed to manufacture equipment | A winding shop can produce insulated copper coils |
| Installed assets | Equipment that physically exists, regardless of where it was invented or manufactured | An imported generator at a mine |
| Service delivery | Available capacity, compatibility, schedule, reliability, and price | A feeder supplies lighting at night but cannot start a large motor |
| Adoption and use | Devices owned, connections purchased, and tasks actually performed | A workshop buys a motor and rearranges production around it |

These layers should not form an obligatory linear sequence: **imports can bypass domestic invention and manufacture**, while operation still requires some combination of trained personnel, instructions, maintenance, and replacement parts.

Historical industrial electrification illustrates the distinction. Replacing a steam engine with an electric motor did not immediately realize every benefit of electricity. Larger gains came as factories moved from shafts and belts toward individually driven machines and reorganized layouts and workflows. Devine’s study covers this transition across approximately 1880–1930. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/from-shafts-to-wires-historical-perspective-on-electrification/500078D9B4764BA1109A7967437CF226)

**Proposed rule:** discovering a node exposes experiments, recipes, and service designs. It does not grant production bonuses, create infrastructure, or equip households automatically.

### 1.2 Networks grow where demand can support their fixed costs

For a prospective utility extension, evaluate:

\[
\text{Expected project value}
=
PV(\text{service revenue})
-
\text{construction cost}
-
PV(\text{operation, maintenance, and replacement})
\]

Construction costs should depend on route length, terrain, rights-of-way, density of customers, network standards, and available construction labor. Revenue depends on connected customers’ actual demand and ability to pay.

This creates several plausible development paths:

**Anchor-customer development.** A mine, factory, tramway, port, or waterworks can justify generation and distribution before household demand does. Chile’s Chivilingo installation, commissioned in 1897, supplied the Lota coal-mining operation through a roughly 10 km transmission line; its installed capacity was 430 kW. [ETHW](https://ethw.org/Milestones%3AChivilingo_Hydroelectric_Plant%2C_1897)

**Dense urban development.** Shorter connections and concentrated commercial demand make some urban districts attractive before dispersed settlements.

**Cooperative or public extension.** Financing and institutional arrangements can overcome barriers that private utilities leave unresolved. In the United States, rural electrification combined inexpensive loans, cooperatives, design changes, bulk purchasing, and financing for wiring and appliances. Cooperatives could buy wholesale electricity rather than construct their own generating stations. [Federal Reserve Bank of Richmond](https://www.richmondfed.org/publications/research/econ_focus/2020/q1/economic_history)

**Proposed rule:** infrastructure investors choose among isolated systems, network extensions, shared facilities, and postponement. Municipalities, firms, cooperatives, and states can evaluate the same physical project using different objectives and financing constraints.

Do not make “private ownership,” “state ownership,” or a particular political constitution a technological prerequisite.

### 1.3 Electricity is a capacity-and-reliability service, not a coverage radius

Electricity requires separate accounting for:

* **Power:** instantaneous capacity, measured in kW.
* **Energy:** output or consumption over time, measured in kWh.
* **Availability and quality:** when service exists, whether interruptions occur, and whether voltage and other electrical characteristics meet equipment requirements.

The World Bank’s Multi-Tier Framework explicitly distinguishes capacity, duration, reliability, quality, affordability, and safety rather than treating a connection as sufficient evidence of useful energy access. [World Bank](https://www.worldbank.org/en/topic/energy/publication/energy-access-redefined)

For a simplified circuit:

\[
P\_{\text{loss}}=I^2R,\qquad I\approx P/V
\]

Thus, for the same transferred power and conductor resistance, raising voltage reduces resistive losses. Transformers and appropriate protection therefore change the economics of distance; electricity should not have a universal fixed delivery radius. Actual AC networks additionally require attention to reactive power, voltage, and synchronization. [U.S. Energy Information Administration](https://www.eia.gov/energyexplained/electricity/delivery-to-consumers.php)

For hydropower:

\[
P=\rho gQH\eta
\]

Here \(Q\) is water flow, \(H\) usable head, and \(\eta\) conversion efficiency. This gives TCE a direct connection between terrain, hydrology, civil works, and electricity production.

**Proposed operating rules:**

1. A generator’s available output is bounded by nameplate capacity, condition, staffing, and fuel or natural-resource availability.
2. Demand exceeding available supply produces explicit rationing, load shedding, or outages—not negative electricity inventories.
3. Devices require sufficient power as well as energy. A low-capacity connection may illuminate a room but fail to run a workshop.
4. Interconnection requires compatible standards and suitable protection or conversion equipment. Drawing a wire between two grids is insufficient.
5. Storage carries energy between periods, with finite capacity, charging limits, losses, and degradation.

### 1.4 Generation technologies are alternatives with different constraints

| Generation route | Essential production chain | Main simulation constraints |
| --- | --- | --- |
| Reciprocating steam engine plus generator | Fuel → boiler steam → mechanical shaft → electricity | Fuel logistics, boiler operation, maintenance; useful where steam machinery already exists |
| Hydroelectricity | Water diversion or reservoir → turbine → generator | Head, seasonal flow, civil construction, sediment, competing water uses |
| Reciprocating internal-combustion generator | Compatible fuel → engine → generator | Fuel quality and delivery, lubricants, mechanics, spare parts |
| Steam turbine | Boiler or another heat source → turbine → generator | Precision manufacture, high-speed machinery, operating scale, cooling arrangements |
| Gas turbine and combined cycle | Compressor → combustion → turbine; optionally recovered exhaust heat → steam cycle | Advanced rotating machinery, suitable fuel; distinguish electrical output from useful recovered heat |
| Nuclear steam generation | Controlled reactor heat → steam system → turbine | Specialized materials, fuel supply, operation, cooling, safety institutions, waste handling |
| Wind generation | Wind rotor → generator → electrical conditioning as required | Wind variability, structural loads, maintenance, grid compatibility |
| Solar photovoltaics | Semiconductor devices → DC electricity → optional storage or conversion | Manufacturing sophistication upstream; local sunlight, area, maintenance, storage or complementary supply |

These are not a mandatory ladder. Steam and hydro powered early electrical systems; commercial gas turbines appeared later; nuclear generation and practical silicon photovoltaics followed different mid-twentieth-century development paths. Wind-generated electricity also belongs to a distinct branch rather than being an upgrade to fossil generation. [ETHW](https://ethw.org/Milestones%3APearl_Street_Station%2C_1882)

For TCE, combined-cycle generation is particularly suitable as a **composed recipe**: gas-turbine output plus exhaust-heat recovery and a steam cycle. It need not be a completely independent scientific discovery.

### 1.5 Internal combustion creates portable power before universal car ownership

The engine is the general-purpose component. Road vehicles are only one application.

A combustion engine can power pumps, mills, generators, boats, agricultural machinery, and vehicles. Its practical dependencies include repeatable machining, appropriate clearances, cooling, lubrication, ignition or fuel injection, compatible fuel, and maintainable moving parts. Early commercial gas engines, Otto’s four-stroke engine, and Diesel’s compression-ignition engine were distinct developments rather than one instantaneous “motorization” event. [Deutz](https://www.deutz.com/en/company/our-history/milestones/)

**Proposed rule:** define engines as interchangeable power-producing goods with characteristics such as:

`shaft_power`, `mass`, `fuel_compatibility`, `efficiency_curve`, `starting_requirements`, `maintenance_interval`, and `repair_skill`.

Applications then combine an engine with other components:

* Pumping installation: engine + pump + water access.
* Generator: engine + electrical generator + controls.
* Truck: engine + transmission + load-bearing chassis + wheels.
* Bus: vehicle chassis + passenger accommodation + transport operation.

Petroleum should not be the only possible fuel path. Gasoline, diesel, ethanol, and gaseous fuels have different properties and require compatible equipment; interchangeable fuel names must not imply interchangeable engine operation. [Alternative Fuels Data Center](https://afdc.energy.gov/fuels/properties)

### 1.6 Roads and cities adapt through investment and conflict

A motor vehicle does **not** physically require an asphalt highway. Its usefulness does depend on traction, road roughness, drainage, gradients, bridge strength, congestion, fuel access, and repairs.

The transition to motor-oriented streets also involved changing rules and allocating scarce urban space. Norton’s history of American cities emphasizes conflict among pedestrians, businesses, utilities, and motorists rather than treating automobile priority as a mechanically inevitable consequence of invention. [MIT Press](https://mitpress.mit.edu/9780262516129/fighting-traffic/)

**Proposed rule:** calculate generalized travel cost:

\[
G =
\text{money cost}
+
v\_t(\text{travel}+\text{waiting}+\text{access}+\text{parking time})
+
\text{expected risk cost}
\]

Agents choose among feasible modes using this cost, their resources, cargo, schedules, and preferences. Settlement development responds to accessibility, but also to land ownership, housing supply, public investment, and travel costs.

More road capacity should not guarantee permanently faster journeys. Duranton and Turner found approximately proportional long-run growth in vehicle-kilometers traveled following increases in interstate lane-kilometers across US metropolitan areas. That is an empirical calibration target for a particular setting—not a universal constant for every road. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.6.2616)

### 1.7 Telecommunications transmit messages, not omniscience

Represent at least three distinct service architectures:

| Architecture | Capacity bottleneck | What the service changes |
| --- | --- | --- |
| Telegraph | Operators, circuits, message length, relay and delivery queues | Written information travels rapidly between connected offices |
| Telephone | Subscriber lines, switching, trunk circuits, operator or switching capacity | Interactive speech without physical co-presence |
| Broadcast radio | Transmitter coverage, spectrum interference, receiver access, programming | One sender reaches many listeners simultaneously |

Commercial telegraphy and international coordination preceded household electrification. Early telephones used local batteries; common-battery systems subsequently moved power provision into the network. Therefore **household grid access must not be a hard prerequisite for telephone service**. [ITU](https://www.itu.int/en/history/Pages/ITUsHistory.aspx)

For a message:

\[
T\_{\text{delivery}}
=
T\_{\text{access}}
+
T\_{\text{queue}}
+
T\_{\text{transmission}}
+
T\_{\text{last mile}}
+
T\_{\text{recipient availability}}
\]

A telegraph office in the regional capital does not instantly inform every farmer. Someone must originate the message, pay or qualify for service, interpret it, and deliver it onward.

Radio should similarly require a functioning receiver or access to a shared listening place, intelligible language, attention, and usable signal. Wireless networks also need interference management and interoperability; international radiotelegraph coordination was already a major institutional issue in the early twentieth century. [ITU](https://www.itu.int/en/history/Pages/ITUsHistory-page-2.aspx)

**Governance effects should be conditional on content and institutions.** Research on interwar Germany found that radio’s political effects changed with control and programming. Research on broadcasts aimed at the Lord’s Resistance Army found effects supporting defection and reduced violence. Neither supports a universal “radio increases authoritarianism” coefficient. [OUP Academic](https://academic.oup.com/qje/article-abstract/130/4/1885/1916582)

---

## 2. Parameters: observed anchors, engineering values, and calibration limits

### Confidence convention

**High** means a clearly documented specification, physical quantity, or directly reported dataset value. **Medium** means an estimate tied to a particular sample or historical interpretation. Neither rating implies universal applicability.

### 2.1 Engineering and service parameters

| Parameter | Value or range | Scope and recommended use | Confidence |
| --- | --- | --- | --- |
| Gasoline/E10 lower heating value | **31.2–32.4 MJ/L** | Modern fuel-property reference; formulation matters | High. US DOE AFDC; converted from Btu/US gallon. [Alternative Fuels Data Center](https://afdc.energy.gov/fuels/properties) |
| Low-sulfur diesel lower heating value | **35.8 MJ/L** | Fuel-energy accounting, not an engine-efficiency assumption | High. [Alternative Fuels Data Center](https://afdc.energy.gov/fuels/properties) |
| Ethanol E100 lower heating value | **21.3 MJ/L** | Distinguish volumetric consumption from gasoline | High. [Alternative Fuels Data Center](https://afdc.energy.gov/fuels/properties) |
| Stationary natural-gas reciprocating CHP electrical efficiency | **27.0–41.6% HHV** | EPA examples spanning **100 kW–9.34 MW**; not early engines or every operating load | High for listed examples; medium for transfer. [US EPA](https://www.epa.gov/sites/default/files/2015-07/documents/catalog_of_chp_technologies_section_2._technology_characterization_-_reciprocating_internal_combustion_engines.pdf) |
| Same CHP systems: electricity plus useful recovered heat | **76.5–80.0% HHV** | Only count recovered heat when a customer can actually use it | High for examples. **Not electrical efficiency.** [US EPA](https://www.epa.gov/sites/default/files/2015-07/documents/catalog_of_chp_technologies_section_2._technology_characterization_-_reciprocating_internal_combustion_engines.pdf) |
| Reciprocating-generator availability | Approximately **96–98.2%** | Surveyed maintained generating units; does not include every possible network failure | Medium; sample-specific. [US EPA](https://www.epa.gov/sites/default/files/2015-07/documents/catalog_of_chp_technologies_section_2._technology_characterization_-_reciprocating_internal_combustion_engines.pdf) |
| Early commercial gas-turbine landmark | **4 MW net; 17.4% efficiency** | Neuchâtel, Switzerland, 1939; useful historical equipment anchor | High for this installation. [ASME](https://www.asme.org/wwwasmeorg/media/resourcefiles/aboutasme/who%20we%20are/engineering%20history/landmarks/135-neuchatel-gas-turbine.pdf) |
| Early long-distance three-phase demonstration | **180 kW, 20 kV, about 175 km** | Lauffen–Frankfurt, Germany, 1891; not a universal transmission limit | High for demonstration specifications. [Edison Tech Center](https://edisontechcenter.org/LauffenFrankfurt.html) |
| Early purpose-built petrol-car engine | **0.55 kW / 0.75 hp** | Benz vehicle, 1885–1886; demonstrates that early useful vehicles were very low-powered by later standards | High for artifact specification. [Mercedes-Benz Group](https://group.mercedes-benz.com/company/tradition/company-history/1885-1886.html) |
| Electricity-access capacity thresholds | From **3 W** to **over 2 kW** across tiers | World Bank framework benchmarks, not historical household demand estimates | High as framework definitions. [Sustainable Energy for All | SEforALL](https://www.seforall.org/sites/default/files/Beyond-Connections-Introducing-Multi-Tier-Framework-for-Tracking-Energy-Access.pdf) |
| Electricity-access daily availability thresholds | From at least **4 hours/day** to **over 23 hours/day** | Track duration separately from connection and capacity | High as framework definitions. [Sustainable Energy for All | SEforALL](https://www.seforall.org/sites/default/files/Beyond-Connections-Introducing-Multi-Tier-Framework-for-Tracking-Energy-Access.pdf) |
| Basic-service affordability benchmark | **Below 5% of household income** | A framework benchmark for applicable tiers, not a universal behavioral threshold | High as definition; low as a universal adoption rule. [Sustainable Energy for All | SEforALL](https://www.seforall.org/sites/default/files/Beyond-Connections-Introducing-Multi-Tier-Framework-for-Tracking-Energy-Access.pdf) |

**Unit warning:** LHV and HHV use different conventions for fuel energy. Do not combine an HHV efficiency with an LHV fuel value without conversion.

For an illustrative TCE generator—not a historical claim—assume diesel LHV of 35.8 MJ/L and **30% LHV electrical efficiency**:

\[
\text{fuel at 100 kW}
=
\frac{100\times3.6}{35.8\times0.30}
\approx 33.5\ \text{L/hour}.
\]

A 1,000-liter tank then contains about **2.98 MWh of deliverable electrical energy** under that assumption. This makes fuel shortages and generator operating costs consequences of inventories rather than arbitrary penalties.

### 2.2 Financing, construction, and measured effects

| Observation | Quantitative anchor | How to use it |
| --- | --- | --- |
| US rural-electrification lending | Approximately **2–3% annual nominal interest** under REA arrangements | A historical policy scenario, not a default financing rate for all societies. [Federal Reserve Bank of Richmond](https://www.richmondfed.org/publications/research/econ_focus/2020/q1/economic_history) |
| US rural line-construction costs, late 1930s | Reported costs **below $825/mile**, compared with earlier utility estimates around **$2,000/mile** | Historical nominal dollars. Reflects changed designs and procurement as well as institutions; not a controlled universal cost reduction. [Federal Reserve Bank of Richmond](https://www.richmondfed.org/publications/research/econ_focus/2020/q1/economic_history) |
| Rural electrification and female employment, KwaZulu-Natal | Approximately **+9–9.5 percentage points** over the study’s five-year period | Dinkelman’s instrumental-variable estimate; use as one scenario-level validation target, not an automatic employment bonus. [Taryn Dinkelman](https://www.taryndinkelman.com/s/Dinkelman-2011-AER-The-Effects-of-Rural-Electrification-on-Employment-New-Evidence-from-South-Africa.pdf) |
| Rural electrification experiment, Kenya | No meaningful medium-run economic or non-economic gains detected across the main measured outcomes | Connections, consumption, and complementary opportunities must remain distinct. This is not evidence of zero long-run value everywhere. [Econ at Berkeley](https://emiguel.econ.berkeley.edu/research/experimental-evidence-on-the-demand-for-and-costs-of-rural-electrification/) |
| US metropolitan interstate travel response | Long-run VKT elasticity with respect to lane-km approximately **1** | A setting-specific congestion and induced-travel calibration target. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.6.2616) |

For economic balancing, prefer **physical bills of materials, worker-hours, land requirements, maintenance tasks, and locally determined prices**. Converting a nominal 1930s dollar figure directly into universal TCE currency would conceal the important differences.

### 2.3 Adoption curves: a comparable historical panel

The following US series are useful calibration anchors, but their denominators differ.

| Year | All registered motor vehicles, millions | Telephones per 1,000 population | Households reporting a radio, millions |
| --- | --- | --- | --- |
| 1900 | 0.008 | 17.6 | — |
| 1910 | 0.469 | 82.0 | — |
| 1920 | 9.239 | 123.9 | — |
| 1930 | 26.750 | 163.4 | 12.048 |
| 1940 | 32.453 | 165.1 | 28.048 |
| 1950 | 49.162 | 280.9 | 40.111 |

Sources: FHWA historical registrations and US Census historical communications statistics. Vehicle registrations are **not household car ownership**; telephone density is **not the percentage of households connected**. Radio ownership counts include some nonworking sets and do not measure all listeners. Dashes mean no value supplied here, not zero. [Federal Highway Administration](https://www.fhwa.dot.gov/ohim/summary95/mv200.pdf)

Adoption is not necessarily monotonic: from 1930 to 1933, US telephone density fell from **163.4 to 132.6 per 1,000**, while registered vehicles fell from **26.75 million to 24.16 million**. A correct model should allow disconnection, scrappage, deferred replacement, and financial contraction. [Census](https://www2.census.gov/library/publications/1960/compendia/hist_stats_colonial-1957/hist_stats_colonial-1957-chR.pdf)

For electricity, use separate curves for **network reach, connection, service quality, appliance ownership, and consumption**. A single electrification percentage cannot identify all five.

---

## 3. Variation across development conditions and world regions

### 3.1 Development conditions—not compulsory eras

The following is a recommended mapping into TCE’s existing systems, rather than a claim that every society historically passed through the same sequence.

| Development condition | Existing service base | What modern technologies must connect to |
| --- | --- | --- |
| Foraging or highly mobile livelihoods | Portable lighting, human transport, interpersonal communication | Portability, low fixed investment, shared access; imported devices may be usable without permanent networks |
| Early farming | Seasonal labor peaks, water management, milling, storage, local exchange | Pumps, processing power, transport of bulky produce; settlement density affects network viability |
| Pre-industrial specialization | Metallurgy, glass and ceramics, workshops, commercial accounting, mechanical power | Conductors, insulation, precision components, equipment repair, investment coordination |
| Industrial production | Machine tools, standardized components, larger establishments, organized infrastructure construction | Scalable manufacture, utility operation, fuel distribution, mass transport and communications |
| Extensive modern networks | Interconnected utilities, widespread devices, specialized supply chains | Reliability management, replacement cycles, interoperability, congestion, complex dependencies |

**Do not require these rows to advance together.** A settlement can retain subsistence farming while importing radios, or operate an industrial mine beside communities with limited household electricity.

### 3.2 Regional cases and their simulation implications

| Region or case | Historical pattern | Implication for TCE |
| --- | --- | --- |
| Britain and the United States | Early central electrical stations in 1882 were followed by larger distribution and interconnection systems; national-scale British grid development was a later institutional and engineering undertaking | Separate local generation, urban utility formation, and regional interconnection rather than unlocking them simultaneously. [ETHW](https://ethw.org/Milestones%3APearl_Street_Station%2C_1882) |
| Rural United States | About half of farms had electricity by the end of WWII; access approached urban levels during the following decade | Late catch-up can result from financing, organizational change, and construction practices—not new electrical physics. [Federal Reserve Bank of Richmond](https://www.richmondfed.org/publications/research/econ_focus/2020/q1/economic_history) |
| Chile, Lota–Chivilingo | Mining demand supported an early hydroelectric installation and transmission link in 1897 | A resource-export enclave can adopt advanced power before general household access. [ETHW](https://ethw.org/Milestones%3AChivilingo_Hydroelectric_Plant%2C_1897) |
| India, Sidrapong near Darjeeling | The 1897 installation began with two 65 kW generating units; imported machinery had to be transported through difficult terrain | Separate equipment procurement from local transport and construction feasibility. The available IEEE proposal dossier is useful but should not be treated as an uncontested global-priority source. [IEEE Milestones Wiki](https://ieeemilestones.ethw.org/Milestone-Proposal%3ASidrapong_Hydel_Power_Station%2C_1897) |
| South Africa and Kenya | Empirical studies find different consequences from rural electrification under different conditions | Do not assign one “African electrification” trajectory or one employment multiplier. Model connection costs, usable capacity, tasks, and economic opportunities. [Taryn Dinkelman](https://www.taryndinkelman.com/s/Dinkelman-2011-AER-The-Effects-of-Rural-Electrification-on-Employment-New-Evidence-from-South-Africa.pdf) |
| Japan and the Nordic countries | Commercial cellular service began in Japan in 1979 and followed in the Nordic countries in 1981 | Leadership can differ by technological branch. Cellular communication can bypass local fixed subscriber wires without bypassing power, backhaul, or network operations. [ITU](https://www.itu.int/en/history/Pages/ITUsHistory-page-7.aspx) |

The general modeling lesson is **uneven combinations**: sophisticated industrial equipment, weak household access, imported expertise, local repair adaptation, and different ownership arrangements can coexist.

---

## 4. Stylized facts a correct simulation should reproduce

These are validation targets, not rules that force every generated world to resemble a particular historical country.

| Pattern | Evidence or quantitative target | Failure mode to avoid |
| --- | --- | --- |
| Invention and widespread use are separated by long, variable lags | Electric factory reorganization unfolded across decades; vehicle and telephone adoption likewise expanded over multiple decades | One discovery immediately transforming every workplace. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/from-shafts-to-wires-historical-perspective-on-electrification/500078D9B4764BA1109A7967437CF226) |
| Advanced infrastructure can serve production before households | Chivilingo’s 430 kW mining-oriented installation | Always electrifying houses before mines, pumps, or industry. [ETHW](https://ethw.org/Milestones%3AChivilingo_Hydroelectric_Plant%2C_1897) |
| Access can expand without immediate large economic gains | Kenya’s randomized electrification evidence contrasts with South Africa’s employment results | A universal GDP, education, or employment bonus per connection. [Econ at Berkeley](https://emiguel.econ.berkeley.edu/research/experimental-evidence-on-the-demand-for-and-costs-of-rural-electrification/) |
| Diffusion can stall or reverse | US telephone and vehicle contractions during the early 1930s | Irreversible technology adoption and immortal equipment stocks. [Census](https://www2.census.gov/library/publications/1960/compendia/hist_stats_colonial-1957/hist_stats_colonial-1957-chR.pdf) |
| Road expansion can stimulate additional travel | Approximately unit long-run elasticity in the US interstate study | Additional lanes permanently eliminating congestion regardless of relocation and travel demand. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.6.2616) |
| Communications change the reach of institutions, not their intrinsic benevolence | Opposite or contingent political effects across radio settings | A fixed “radio produces loyalty” or “telephone produces democracy” modifier. [OUP Academic](https://academic.oup.com/qje/article-abstract/130/4/1885/1916582) |

Additional **proposed TCE tests** should verify that maintenance shortages can reverse service gains; imported equipment can operate before domestic manufacturing exists; and inexpensive shared access can spread communication benefits before every household owns a device.

---

## 5. Modeling recommendation and technology graph

### 5.1 Graph design rules

Use a **capability graph with alternative production paths**.

Hard prerequisites should describe physical requirements: suitable materials, tolerances, compatible power, signal handling, and usable infrastructure. Scientific institutions, credit systems, literacy, or state sponsorship can accelerate discovery and deployment, but should not always be mandatory prerequisites.

In the tables below:

* **“+” means combined requirements.**
* **“OR” means alternative routes.**
* Dependencies and unlock bundles are **proposed TCE designs**.
* Historical dates describe the cited benchmark, not exclusive priority.
* Imported goods may satisfy a material requirement without domestic mastery of their manufacturing node.

Assume upstream nodes from other TCE branches already cover metallurgy, wire drawing, glass, ceramics, precision machining, chemical processing, boilers, mechanical turbines, construction, and transport.

### 5.2 Electricity nodes

| Node | Historical benchmark | Proposed prerequisites | Concrete unlocks |
| --- | --- | --- | --- |
| **E01 — Chemical electricity** | Volta’s battery, **1799–1800, Italy** | Conductive metals + electrolyte handling + repeatable electrical contacts | Primary cells; experimental current sources; portable electrical supply. [ETHW](https://ethw.org/Milestones%3AVolta%27s_Electrical_Battery_Invention%2C_1799) |
| **E02 — Electromagnetic induction** | Faraday’s ring apparatus, **1831, London** | Insulated wire + magnetic materials + experimental apparatus | Induction experiments; generator, transformer, and related electromagnetic designs. [Royal Institution](https://www.rigb.org/explore-science/explore/collection/michael-faradays-ring-coil-apparatus) |
| **E03 — Practical dynamo manufacture** | Siemens’s dynamo-electric work, **1866–1867, Berlin** | E02 + precision rotating machinery + reliable insulation + mechanical drive | Generator goods; winding and repair recipes; workshop-scale generation. [Siemens Assets](https://assets.new.siemens.com/siemens/assets/api/uuid%3A2a40cdf2-c0dc-44fb-a1e5-abefe5ed9be8/background-dynamo-electric-principle-e.pdf) |
| **E04 — Electric lighting systems** | Commercial arc and incandescent systems, **1870s–1882, Europe/US** | Electrical supply + switching and protection; incandescent variant additionally requires vacuum glass and suitable filaments | Lamps, fittings, lighting installation recipes, street-lighting and indoor-lighting services. [ETHW](https://ethw.org/Milestones%3APearl_Street_Station%2C_1882) |
| **E05 — Local electrical utility** | Holborn Viaduct and Pearl Street, **1882, London/New York** | Generation + conductors and insulation + protection + metering + distribution construction | Power stations, local feeders, customer connections, tariffs, utility operation jobs. [ETHW](https://ethw.org/Milestones%3APearl_Street_Station%2C_1882) |
| **E06 — Hydroelectric installation** | Vulcan Street plant, **1882, Appleton, US** | Generator + hydraulic turbine + suitable water site + civil works | Hydro station, penstock/intake works, hydroelectric production recipe. [ETHW](https://ethw.org/Milestones%3AVulcan_Street_Plant%2C_1882) |
| **E07 — Transformer-based AC distribution** | Great Barrington, **1886, US**; long-distance three-phase demonstration, **1891, Germany** | E02 + transformers + alternators + insulation suitable for voltage + protection | Voltage conversion, longer-distance transmission, substations, differentiated transmission/distribution networks. [ETHW](https://ethw.org/Milestones%3AAlternating_Current_Electrification%2C_1886) |
| **E08 — Practical polyphase motors** | Dolivo-Dobrowolski’s usable asynchronous motor, **1889, Berlin** | Suitable AC supply + rotating-field design + electrical steel/copper + precision manufacture | Industrial motors, machine-drive retrofits, pumps, lifts and electrically driven production equipment. [Siemens Assets](https://assets.new.siemens.com/siemens/assets/api/uuid%3A2a40cdf2-c0dc-44fb-a1e5-abefe5ed9be8/background-dynamo-electric-principle-e.pdf) |
| **E09 — Steam-turbine generation** | Parsons’s compound turbine, **1884, Britain** | Steam engineering + precision blades and shafts + balancing + generator integration | Turbo-generators; larger steam-electric plant designs; turbine manufacture and repair. [Wikisource](https://en.wikisource.org/wiki/The_Steam_Turbine) |
| **E10 — Coordinated grid interconnection** | Large-scale British grid development, **1920s–1930s** | Compatible networks + transmission + protection + dispatch and operating procedures | Interties, shared reserves, regional dispatch, cascading-failure exposure, coordinated standards. [National Grid](https://www.nationalgrid.com/about-us/what-we-do/our-history) |
| **E11 — Gas-turbine power plant** | Neuchâtel commercial plant, **1939, Switzerland** | Compressor/turbine manufacture + high-temperature materials + combustion control + fuel supply | Gas-turbine generators; later combined-cycle recipe with E09 and heat recovery. [ASME](https://www.asme.org/wwwasmeorg/media/resourcefiles/aboutasme/who%20we%20are/engineering%20history/landmarks/135-neuchatel-gas-turbine.pdf) |
| **E12 — Nuclear-electric plant** | Grid-connected Obninsk plant, **1954, USSR** | Reactor and fuel capabilities + controlled heat removal + turbine system + specialist operation and safety infrastructure | Nuclear generating station, specialized fuel and maintenance chains, waste-handling obligations. [ETHW](https://ethw.org/Obninsk_Nuclear_Power_Plant) |
| **E13 — Practical silicon photovoltaics** | Bell Laboratories cells, **1954, US** | High-purity semiconductor processing + junction fabrication + contacts and encapsulation | PV modules, standalone DC generation, later grid-connected and storage-coupled variants. [EERE Energy](https://www1.eere.energy.gov/solar/pdfs/solar_timeline.pdf) |

**Important alternative paths:** wind generation can compose an existing wind-power branch with electrical generators and appropriate controls. Battery storage, rectifiers, inverters, protective equipment, and metering should be equipment families or additional nodes according to the granularity of the overall graph. A daytime PV-powered device does not inherently require either a grid or a battery.

### 5.3 Combustion, vehicles, and roads

| Node | Historical benchmark | Proposed prerequisites | Concrete unlocks |
| --- | --- | --- | --- |
| **C01 — Bulk refined liquid fuels** | Petroleum refining developments, **1850s, Pittsburgh, US**, as one regional landmark | Distillation + compatible vessels + storage + feedstock and transport | Refineries, fuel fractions, lubricants, tanks, wholesale fuel distribution. Other compatible fuels remain alternative branches. [American Chemical Society](https://www.acs.org/education/whatischemistry/landmarks/pennsylvaniaoilindustry.html) |
| **C02 — Commercial stationary gas engine** | Otto–Langen commercial landmark, **1867, Cologne/Paris** | Precision cylinders + gas supply + controlled ignition + cooling and lubrication | Stationary engines, engine shops, powered pumps and workshop machinery. [Deutz](https://www.deutz.com/en/company/our-history/milestones/) |
| **C03 — Compressed-charge four-stroke engine** | Otto engine, **1876, Cologne** | Combustion-engine capability + timed valves + sealing and compression control | Four-stroke engine recipes and performance variants. Not a compulsory prerequisite for every later engine cycle. [Deutz](https://www.deutz.com/en/company/our-history/striking-heads/) |
| **C04 — Compact liquid-fuel engine** | Daimler/Maybach and Benz developments, **1880s, Germany** | Suitable engine cycle + fuel preparation + ignition + cooling + favorable power-to-mass design | Portable and vehicle engines, small motorboats, mobile machinery, compact generators. [Deutz](https://www.deutz.com/en/company/our-history/striking-heads/) |
| **C05 — Practical compression-ignition engine** | Diesel engine, **1897, Augsburg** | High-compression manufacture + fuel injection + robust structure + compatible fuel | Diesel engines, heavy stationary power, later heavy-vehicle and marine variants. Does not require prior petrol-car adoption. [Deutz](https://www.deutz.com/en/company/our-history/striking-heads/) |
| **C06 — Integrated road motor vehicle** | Benz’s purpose-built petrol vehicle, **1885–1886, Mannheim** | Wheeled chassis + steering/braking + transmission + compact engine **OR** suitable steam/electric drive | Vehicles, garages, vehicle repair; passenger, freight, agricultural, and special-purpose variants. [Mercedes-Benz Group](https://group.mercedes-benz.com/company/tradition/company-history/1885-1886.html) |
| **C07 — Standardized motor-vehicle mass production** | Model T and moving assembly-line development, **1908–1913, Detroit area** | Interchangeable components + machine tools + coordinated factory workflow + sufficient demand/finance | High-volume assembly, standardized spares, lower repair complexity, extensive model variants. [Ford From the Road](https://www.fromtheroad.ford.com/us/en/articles/2025/ford-model-t-universal-car) |
| **C08 — Motor-traffic road engineering and operation** | Major urban transformation, approximately **1900–1930, US cities**, among other settings | Existing roads + drainage/bridge engineering + traffic observation + construction and institutional capacity | Upgraded surfaces, junctions, signals, parking facilities, traffic rules and road-maintenance services. This is a deployment/engineering bundle, not “the invention of roads.” [MIT Press](https://mitpress.mit.edu/9780262516129/fighting-traffic/) |

Petroleum refining should also serve lighting, lubricants, and other industrial demand. The historical petroleum industry did not originate solely to supply automobiles. The 1859 Pennsylvania drilling landmark should not be mislabeled as the world’s first use or production of petroleum. [American Chemical Society](https://www.acs.org/education/whatischemistry/landmarks/pennsylvaniaoilindustry.html)

### 5.4 Telecommunications nodes

| Node | Historical benchmark | Proposed prerequisites | Concrete unlocks |
| --- | --- | --- | --- |
| **T01 — Electric telegraph service** | Commercial service **1839, Britain**; Morse demonstration **1844, US** | Electrical source + insulated conductors + signaling devices + code and operator training | Telegraph instruments, offices, lines, message-handling and delivery services. [ITU](https://www.itu.int/en/history/Pages/ITUsHistory.aspx) |
| **T02 — Submarine telegraph cable** | Regular Channel service **1851**; successful Atlantic system **1866** | T01-compatible signaling + waterproof cable + cable laying and repair capability | Marine cables, landing stations, long-distance international messaging. [ITU](https://www.itu.int/en/history/Pages/ITUsHistory.aspx) |
| **T03 — Practical telephone apparatus** | Patent/commercial landmarks **1876–1877, US** | Sound transducers + conductive circuits + suitable local or network electrical supply | Telephone sets, local voice circuits, installation and repair recipes. [ITPA](https://www.nationalitpa.com/history-of-telephone) |
| **T04 — Telephone exchange** | Workable exchange **1878, US**; common-battery development **1888** | T03 + switchboards + subscriber records + operators or switching equipment | Exchange buildings, subscriber networks, call queues, operator employment, connection fees. [ITPA](https://www.nationalitpa.com/history-of-telephone) |
| **T05 — Automatic telephone switching** | Strowger patent **1891, US** | Electromagnetic selectors + precision mechanisms + compatible signaling and maintenance | Dial exchanges, reduced operator requirements per connection, new capital and maintenance demands. Patent date is not universal commercial readiness. [Google Patents](https://patents.google.com/patent/US447918A/en) |
| **T06 — Wireless telegraphy** | Parallel experimental and practical developments, **1890s–1900s, several countries** | Electromagnetic signaling + transmitter/detector + antennas + operating procedures | Wireless stations, ship communications, wireless operators, spectrum interference and coordination. [ITU](https://www.itu.int/en/history/Pages/ITUsHistory.aspx) |
| **T07 — Vacuum electronic devices** | Fleming valve, **1904, Britain**, as the diode landmark | Vacuum glasswork + electrodes + electrical supply + suitable manufacture | Radio detection and rectification; later amplifying-valve recipes must be distinguished from the diode, which does not amplify. [ETHW](https://ethw.org/Milestones%3AFleming_Valve%2C_1904) |
| **T08 — Organized voice broadcasting** | KDKA, **1920, Pittsburgh**, as one broadcasting landmark | Voice modulation + adequate transmitter + receivers + programming and station operation | Radio stations, studios, receiver goods, public listening, scheduled mass communication. [ETHW](https://ethw.org/Milestones%3AWestinghouse_Radio_Station_KDKA%2C_1920) |
| **T09 — Two-way mobile radio** | Police-radio landmark **1933, Bayonne, US** | Compact transceivers + mobile power + antennas + dispatch procedures | Fleet dispatch, mobile coordination, vehicle radio installations and repair. [ETHW](https://ethw.org/Milestones%3ATwo-Way_Police_Radio_Communication%2C_1933) |
| **T10 — Transistor electronics** | Bell Laboratories, **1947, New Jersey, US** | High-purity semiconductor materials + controlled contacts/junctions + precision production | Smaller electronic amplifiers and controls; portable receivers and later communications equipment. [ETHW](https://ethw.org/Milestones%3AInvention_of_the_First_Transistor_at_Bell_Telephone_Laboratories%2C_Inc.%2C_1947) |
| **T11 — Repeater-equipped ocean telephone cable** | TAT-1, **1956, UK–Canada** | Reliable submarine cable + long-life repeaters + cable ships + network interconnection | Ocean voice trunks, landing facilities, long-distance maintenance operations. [ETHW](https://ethw.org/Milestones%3AThe_First_Submarine_Transatlantic_Telephone_Cable_System_%28TAT-1%29%2C_1956) |
| **T12 — Low-loss optical fiber** | Corning, **1970, New York, US** | Very pure glass + fiber drawing + emitters/detectors + joining and protection techniques | Optical cable, high-capacity backbone links, optical networking equipment. [ETHW](https://ethw.org/Milestones%3AWorld%27s_First_Low-Loss_Optical_Fiber_for_Telecommunications%2C_1970) |
| **T13 — Commercial cellular network** | **1979, Japan**; Nordic networks **1981** | Compact radio electronics + base stations + power/backhaul + frequency reuse and handover + subscriber operation | Mobile telephone service, towers, mobile devices, coverage planning, mobile billing and repair. [ITU](https://www.itu.int/en/history/Pages/ITUsHistory-page-7.aspx) |

These nodes supply **34 integration points**, not 34 mandatory research-bar steps. Some can be merged into equipment families; others deserve expansion when their production chains become important.

### 5.5 Agent and institution representation

#### People and households

Individuals need employment, schedules, technical skills, information exposure, and access to devices. Households need shared equipment, income, cash or credit constraints, connections, bills, and unmet service needs.

For adoption, evaluate:

\[
\Delta V\_i
=
PV(\text{task benefits and time saved})
-
\text{upfront expense}
-
PV(\text{bills, maintenance, and expected failure costs})
\]

Then add heterogeneous expectations, familiarity, imitation, trust, and financing constraints. Adoption should be probabilistic among feasible choices, not automatic whenever \(\Delta V\_i>0\).

A household can value a telephone highly because relatives or employers are reachable while assigning little value to a connection to an otherwise empty network. A business can value a motor only when machinery, orders, labor arrangements, and reliable supply make it useful.

#### Firms and public institutions

Give infrastructure operators:

`owner`, `service_area`, `assets`, `standards`, `tariffs`, `connection_fees`, `debt`, `staff`, `input_contracts`, `maintenance_plan`, and `service_obligations`.

A private concession, cooperative, municipal utility, military network, and vertically integrated mine can share the same asset simulation while pursuing different investment objectives.

Government communications should improve the speed of orders and reports **only where staffed organizations can originate, interpret, and act on them**. Fast messages do not transport food, repair bridges, or enforce decisions.

#### Assets and service contracts

An asset should expose capacity, location, condition, compatibility, staffing needs, consumables, spare parts, and construction state.

A service contract should expose price, delivered quantity, capacity limits, quality, schedule, and reliability. This allows electricity, telephone service, transport, water pumping, and other services to reuse a common framework without pretending they have identical physical networks.

### 5.6 What to simplify for 10k–50k agents

**Recommended, unbenchmarked implementation starting point:**

| Subsystem | Keep explicit | Aggregate or approximate |
| --- | --- | --- |
| Electricity | Plant inventories, feeder connections, capacity, losses, outages, maintenance | Aggregate demand by feeder and activity; avoid simulating individual electrical waveforms |
| Roads | Trips, cargo, link capacity/storage, junction delays, vehicle availability | Cache paths; use link queues or mesoscopic traffic rather than full detailed vehicle dynamics everywhere |
| Telephone/telegraph | Origin/destination, access, queue, circuit availability, messages and calls | Aggregate exchange traffic between important events |
| Radio | Stations, coverage, interference classes, receiver access, programs | Cache coverage maps; batch audience exposure rather than testing every transmitter against every person every tick |
| Investment | Institutional budgets, proposals, construction, procurement, financing | Review periodically rather than every simulation tick |

A reasonable **tuning prior**, not a historical parameter, is to solve ordinary electricity service at **15–60-minute intervals**, update routine plans daily, and review investment proposals monthly. Outages, major load changes, construction completion, and emergency dispatch should remain event-driven.

For network physics, radial distribution approximations are often sufficient locally. Meshed transmission requires an appropriate power-flow approximation; **ordinary maximum-flow routing alone is not an electrically valid grid model**.

Keep the Rust simulation authoritative. UE5 should visualize utility activity, travel, repair crews, construction, lighting, and failures from the same state—not operate an independent economy.

At TCE’s population scale, represent some sophisticated industrial inputs through **external trade or abstracted larger supply networks**. This is especially important for semiconductor, specialized turbine, and nuclear-fuel chains. It is a scope recommendation, not a claim that a precise minimum population is historically required.

### 5.7 Existing models and games worth borrowing from

| Reference | Useful ideas for TCE | What not to copy directly |
| --- | --- | --- |
| **MATSim** | Individual activity plans, travel choices, congestion-sensitive adaptation | Repeated-day optimization should not become literal historical time or perfect foresight. [MATSim](https://www.matsim.org/) |
| **SUMO** | Detailed intersection, signal, and multimodal traffic experiments | Full microscopic traffic simulation everywhere may be unnecessary; use it to calibrate simpler TCE rules. [Eclipse Foundation](https://www.eclipse.dev/sumo/) |
| **PyPSA** | Dispatch, storage, generation constraints, and power-flow/investment formulations | A system optimizer is not a model of decentralized historical decision-making. [PyPSA Documentation](https://docs.pypsa.org/) |
| **OnSSET** | Spatial comparison of grid extension, mini-grids, and standalone access | Least-cost planning does not automatically reproduce political priorities, financing barriers, or actual adoption. [ONSSET](https://www.onsset.org/) |
| **Workers & Resources: Soviet Republic** | Visible construction, supply chains, staffed production, utilities, and logistics dependencies | Central player direction and a fixed institutional setting conflict with TCE’s autonomous institutional evolution. [Steam Store](https://store.steampowered.com/app/784150/_Workers__Resources_Soviet_Republic/?snr=1_7_7_230_150_1) |

Use the scientific models primarily as **offline reference and calibration tools**, not as complete replacements for the agent simulation.

---

## 6. Sources, evidence quality, and unresolved parameters

### Recommended research and dataset backbone

| Source | Best use |
| --- | --- |
| **Comin and Hobijn, *The CHAT Dataset* (2009)** | Cross-country historical technology-diffusion series; useful for testing heterogeneous adoption speeds rather than imposing a universal curve. [National Bureau of Economic Research](https://www.nber.org/papers/w15319) |
| **US Census, *Historical Statistics*, communications chapter** | Telephone stocks, density, radio ownership, and historical communications measures; retain original denominators and definitions. [Census](https://www2.census.gov/library/publications/1960/compendia/hist_stats_colonial-1957/hist_stats_colonial-1957-chR.pdf) |
| **FHWA historical motor-vehicle registrations, MV-200** | Long-run vehicle-stock calibration; distinguish all vehicles from automobiles and registrations from household ownership. [Federal Highway Administration](https://www.fhwa.dot.gov/ohim/summary95/mv200.pdf) |
| **World Bank/ESMAP Multi-Tier Framework and household microdata** | Separate capacity, duration, reliability, affordability, and actual access conditions. [World Bank](https://www.worldbank.org/en/topic/energy/publication/energy-access-redefined) |
| **Devine (1983), *From Shafts to Wires*** | Industrial electrification, machinery configuration, and organizational complementarities. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/from-shafts-to-wires-historical-perspective-on-electrification/500078D9B4764BA1109A7967437CF226) |
| **Dinkelman (2011); Lee, Miguel, and Wolfram (2020)** | Contrasting causal evidence on electrification’s effects; useful for testing conditional rather than automatic benefits. [Taryn Dinkelman](https://www.taryndinkelman.com/s/Dinkelman-2011-AER-The-Effects-of-Rural-Electrification-on-Employment-New-Evidence-from-South-Africa.pdf) |
| **Duranton and Turner (2011); Norton (2008)** | Complementary quantitative and historical accounts of road demand and the institutional reconstruction of streets. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.6.2616) |
| **Adena et al. (2015); Armand, Atwell, and Gomes (2020)** | Evidence that radio’s political effects depend on programming, control, audiences, and context. [OUP Academic](https://academic.oup.com/qje/article-abstract/130/4/1885/1916582) |
| **DOE, EPA, IEEE, ASME, ITU, original patents and equipment records** | Fuel properties, equipment specifications, documented engineering milestones, and communications-system development. These are stronger for technical details than for broad causal claims. [Alternative Fuels Data Center](https://afdc.energy.gov/fuels/properties) |

### What remains thin or contested

**Priority claims.** Many technologies emerged through parallel experiments and incremental improvements. Corporate and commemorative histories often emphasize a particular inventor or installation. Store historical dates as provenance, not as proof that only one discovery route is valid.

**Universal adoption coefficients.** The evidence does not justify one globally valid rate for electrification, motorization, telephone adoption, or radio ownership. Fit multiple regional and institutional scenarios; allow plateaus and contraction.

**Historical construction recipes.** Comparable labor-hours, material quantities, and maintenance requirements for early feeders, exchanges, roads, and vehicles remain much thinner than dates and equipment specifications. Initially expose these as explicit balancing assumptions rather than presenting them as measured history.

**Productivity effects.** Do not equate connection with transformation. The strongest design is to let benefits arise through particular tasks—pumping, machine drive, lighting, coordination, transport—and then test whether plausible aggregate changes emerge.

**The central implementation principle is that modernity should be an outcome of connected capabilities.** A dam without a usable transmission system, an engine without fuel and mechanics, or a radio station without receivers and listeners is an incomplete production-and-service chain—not a civilization-wide bonus.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9286b-6c08-83ea-b4e2-38a9059c4223)
