# History of energy and mechanical power: a simulation-ready report for TCE

## Executive recommendation

**Model energy as locally available, task-specific services—not as a civilization-wide “power level.”** A settlement can possess abundant firewood yet lack mechanical power; have a powerful river yet lack an economical mill site; or know how to build an engine without having the craftsmen, fuel supply, or customers needed to operate it.

The historical transition was not simply **muscle → water → wind → steam → electricity**. These systems overlapped, complemented one another, and remained competitive in different applications. In 1777, for example, Watt’s *Old Bess* engine pumped water back uphill so existing waterwheels could continue driving a factory: steam initially supported the hydraulic infrastructure rather than replacing it. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co50947)

For TCE, the fundamental chain should be:

**Resource → extraction or collection → conversion device → transmission → useful task → economic benefit.**

Each link can be constrained by geography, materials, labor, skills, property rights, finance, maintenance, or demand.

**Evidence notation used below:** **H** = strong physical or documentary evidence; **M** = useful but context-dependent measurements or historical reconstruction; **L** = thin or contested evidence. **P** identifies a proposed TCE calibration value, not an observed historical average. Confidence in an attested date does not establish that it was the world’s first invention.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Separate energy quantity, power, and energy quality

Maintain distinct accounts for:

| Account | What it represents | Appropriate unit |
| --- | --- | --- |
| Food and fodder | Metabolic inputs to people and animals | MJ consumed |
| Fuel | Chemical energy in wood, charcoal, peat, coal, and later fuels | kg and MJ |
| Useful heat | Heat actually delivered to cooking, buildings, kilns, or boilers | MJ, with process-temperature requirements |
| Mechanical power | Work delivered through traction, shafts, gears, or pumping mechanisms | kW; accumulated work in kWh |
| Electricity | A secondary carrier produced from another energy source | kWh, with generation and network limits |

One kilowatt-hour is **3.6 MJ**. Power is the rate of delivering energy: a machine requiring 5 kW cannot necessarily operate on a source delivering the same daily energy very slowly.

For a production recipe, use a bottleneck rule such as:

\[
q=\min\left(
\frac{E\_{\mathrm{mechanical}}}{e\_q},
\frac{L}{\ell\_q},
\min\_i\frac{I\_i}{a\_i},
r\_{\mathrm{machine}}\Delta t
\right)
\]

Here \(q\) is output; \(e\_q\) is mechanical energy per unit; \(\ell\_q\) is labor per unit; and \(a\_i\) are material requirements. Add a minimum operating-power requirement where appropriate.

**Implementation consequence:** extra power does not increase output when the mill lacks grain, the furnace lacks ore, or the machine has reached its processing limit.

### 1.2 Muscle power depends on condition, equipment, and the work schedule

For traction:

\[
P=Fv
\]

A harness changes how effectively an animal can transmit force and sustain work. It does not multiply its biological energy supply.

Represent training, body size, health, nutrition, footing, temperature, harness fit, and rest. Animals also require maintenance outside working hours. Their manure, milk, transport services, and eventual meat can make ownership worthwhile even when a comparison based only on shaft power would not. FAO’s treatment of work animals explicitly emphasizes these multiple functions. [FAOHome](https://www.fao.org/fileadmin/user_upload/ags/publications/draugth_ap_overview.pdf)

For people, distinguish baseline food requirements from additional exertion. Otherwise, the simulation risks charging a worker’s entire diet once for staying alive and again for operating a machine.

**Do not implement a universal “horse collar = four times the power” bonus.** Ancient harness reconstructions and assumptions about where they loaded the animal have been disputed. Fit and geometry are better represented as changes to sustainable pull, injury risk, and working speed. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/abs/function-of-the-yoke-saddle-in-ancient-harnessing/8B42C7A149E55A311D8311A558206A6E)

### 1.3 Waterpower is a flow-and-head problem, not an adjacency bonus

Available hydraulic power is:

\[
P\_{\mathrm{shaft}}=9.81QH\eta
\]

with \(P\) in kW, flow \(Q\) in m³/s, usable head \(H\) in meters, and combined efficiency \(\eta\).

An illustrative site with **0.5 m³/s**, **2 m head**, and **50% overall efficiency** supplies approximately **4.9 kW**.

Model the river reach, intake, headrace, wheel, and tailrace sufficiently to distinguish:

* A large, flat river from a smaller stream with an exploitable fall.
* Seasonal shortage from annual average abundance.
* Water storage from energy creation.
* Competing diversions from downstream reuse.

A cascade can use the same water repeatedly **at successive drops in elevation**. Parallel mills drawing on the same diversion must share its flow. A pond shifts water availability through time; it cannot sustain generation indefinitely without replenishment.

Wheel efficiency should depend on design and operating conditions. Modern experiments show substantial differences among designs and demonstrate that well-engineered wheels can outperform assumptions often assigned indiscriminately to all historical machinery. Those experimental maxima should not become default ancient efficiencies. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0960148115003493)

### 1.4 Windpower depends on exposure and intermittent operating windows

For a wind rotor:

\[
P=\tfrac12\rho Av^3C\_p\eta
\]

This gives watts when using SI units. Apply a rated-power ceiling, starting threshold, and shutdown or damage risk in excessive winds.

Because power depends on approximately the **cube of wind speed**, average wind speed is not enough:

\[
\mathbb{E}[v^3]\neq \mathbb{E}[v]^3
\]

Use shared local weather so neighboring mills experience correlated shortages. Distinguish the geometry and operating characteristics of Persian vertical-axis mills from European horizontal-axis mills. Field research on Persian mills shows machinery and architecture adapted to particular wind regimes and local materials, rather than one universally interchangeable design. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/05786967.2021.1960885)

**Storage often belongs after conversion:** grain waiting to be milled, flour inventories, or elevated irrigation water can buffer intermittent power without requiring a fictional mechanical-energy battery.

### 1.5 Fuel processing changes usefulness, transport cost, and losses

Track fuel moisture and processing explicitly. An approximate moisture correction is:

\[
LHV(M)\approx (1-M)LHV\_{\mathrm{dry}}-2.44M
\]

where \(M\) is wet-basis moisture fraction and heating values are in MJ/kg. This is a useful simplified energy balance, not a substitute for material-specific measurements.

Charcoal production loses much of the wood’s original energy. Nevertheless, the resulting fuel can be advantageous because of its combustion characteristics and reduced mass per unit of fuel energy. FAO documents substantial variation in charcoal yield with kiln type, operation, and losses. [FAOHome](https://www.fao.org/4/s4550e/s4550e09.htm)

**Recipe compatibility must remain separate from heating value.** A fuel suitable for boiling water may be unsuitable for a metallurgical process because of contaminants, physical strength, ash, or the required furnace atmosphere. Coke-compatible ironmaking therefore deserves a separate capability from merely recognizing and burning coal. Darby’s documented use of coke at Coalbrookdale in 1709 illustrates this distinction. [National Trust](https://www.ironbridge.org.uk/our-story/the-iron-bridge/?utm_source=chatgpt.com)

### 1.6 Biomass creates a land, labor, and transport constraint

A minimal woodland model is:

\[
B\_{t+1}=B\_t+G(B\_t,\text{climate},\text{management})-H\_t-\text{losses}
\]

Distinguish accessible increment from total standing biomass. Forests can contain large stocks that are expensive to reach or difficult to transport.

Fuel scarcity should usually appear first as **longer collection journeys, higher delivered prices, and competing claims on woodland**, not as every tree abruptly disappearing. Charcoal supply chains make the importance of these costs particularly clear: FAO’s example cost breakdown assigns most expenditure to wood supply and charcoal transport, rather than the kiln itself. [FAOHome](https://www.fao.org/4/x5328e/x5328e02.htm)

**Illustrative TCE calculation—not a historical universal:** 10,000 people consuming 1 tonne of dry wood-equivalent annually require 5,000 hectares of productive woodland at an accessible yield of 2 tonnes/ha/year. That is **50 km²**, before allowing for farmland, settlements, inaccessible slopes, or transport inefficiency.

Peat and coal should instead behave primarily as **depleting deposits** on the simulation’s century-scale horizon.

### 1.7 Adoption depends on the cost of the service, not the efficiency of the machine alone

For each candidate installation, compare:

\[
\text{Service cost}=
\frac{
\text{annualized construction}
+\text{maintenance}
+\text{fuel/feed}
+\text{labor}
+\text{transport}
+\text{expected disruption}
}{
\text{usable annual output}
}
\]

An inefficient engine can be economical where fuel is cheap. An efficient mill can be uneconomical where customers are few or civil works are expensive.

Add institutional conditions: permission to dam a stream, woodland access, credit, toll collection, obligations to repair shared works, and the availability of skilled operators. Investment decisions need not maximize social welfare: a monopolist may prefer a profitable toll mill to a cheaper arrangement for villagers.

For bounded-rational investment behavior, the agent-based **MUSE** energy framework provides a useful precedent: heterogeneous actors make technology investments under differing constraints and imperfect foresight. TCE should borrow this decision structure, not its economy-wide scale. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2211467X22001584)

---

## 2. Quantitative parameters and calibration anchors

### 2.1 Human and animal mechanical output

These figures describe useful output while working or practical daily work—not metabolic energy expenditure.

| Source of power | Useful mechanical output | Conditions and confidence |
| --- | --- | --- |
| Adult human, sustained operation | Approximately **50–100 W** | Task-, age-, and condition-dependent working range; **M** |
| FAO human example | **75 W for a three-hour effort** | Example for a 35-year-old, not a universal age rule; **M** |
| Practical human daily external work | About **0.2–0.3 kWh/day** | FAO planning estimate; **M** |
| Donkey | **75–200 W** | Reported range; **M** |
| Ox/bullock | **300–500 W** | Reported range; **M** |
| Light horse | **400–800 W** | Reported range; **M** |
| Heavy horse | **500–1,000 W** | Large animals; inappropriate as the default ancient horse; **M** |
| Mule | **300–600 W** | Reported range; **M** |
| Camel | **400–700 W** | Reported range; **M** |
| Buffalo | **600–1,000 W** | Reported range, strongly dependent on animal and conditions; **M** |

Source: Fraenkel’s FAO *Water Lifting Devices*. Its tabulated ranges should not be treated as simultaneous maxima for force, speed, duration, and body size. [FAOHome](https://www.fao.org/4/ah810e/AH810E08.htm)

**Scale comparison:** a 5 kW mill delivers the instantaneous mechanical output of roughly **67 people working at 75 W**. It does not replace 67 people doing arbitrary jobs: only compatible tasks can be mechanized, and the mill still requires inputs, operators, and maintenance.

Animal performance can also vary substantially within a species. A FAO account of rural China uses approximately **0.6–0.7 horsepower**—about **0.45–0.52 kW**—for major work animals, illustrating why local evidence should override a universal species constant. [FAOHome](https://www.fao.org/4/w0613t/w0613T0p.htm)

### 2.2 Mills: proposed operating envelopes

Historical installations vary too much for one reliable “average mill.” Use the following as initial engineering priors, then calculate actual output from site conditions.

| Installation or parameter | Starting value/range | Status |
| --- | --- | --- |
| Small direct-drive horizontal watermill | **0.5–3 kW** operating output | **P** |
| Ordinary geared vertical watermill | **1–10 kW** | **P**; permit much larger installations |
| Simple horizontal or undershot wheel efficiency | **20–40%** | **P**, conservative starting envelope |
| Well-built overshot wheel efficiency | **50–70%** | **P**, informed by engineering literature |
| Shaft/gearing transmission efficiency | **70–90%** | **P**; model separately from the wheel |
| Traditional horizontal-axis windmill | **1–10 kW** in useful operating winds | **P**; not annual average output |
| Horizontal-axis wind power coefficient \(C\_p\) | **0.15–0.30** | **P**, design sensitivity range |
| Watermill annual availability | Compute from hydrology, weather, repair, and operating decisions | Do not assign a universal capacity factor |

Experimental work reports approximately **46% efficiency** for an improved horizontal-wheel configuration, while overshot-wheel research reports high peak efficiencies and lower overall operational efficiencies. These are engineering anchors, not measurements of every ancient mill. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0960148115003493)

A useful contemporary cross-check comes from Himalayan watermill improvements: the Himachal Pradesh energy agency describes installations using approximately **8–10 m head** and **110–120 liters/second** to produce up to **5 kW of electricity**. These are upgraded systems, not ancient performance estimates. [Himurja](https://himurja.hp.gov.in/investor-info/improved-watermill-projects/)

**Derived examples:**

* A 12 m diameter wind rotor at 6 m/s, with \(C\_p=0.20\), intercepts approximately **3 kW** of usable rotor power before additional transmission losses. At 8 m/s the corresponding result is about **7.1 kW**.
* One kWh of mechanical input to a pump operating at 50% efficiency lifts approximately **36.7 m³ of water through 5 m**.
* Five kW used for ten hours supplies **50 kWh**; the same machine standing idle supplies nothing, despite unchanged nameplate capacity.

### 2.3 Fuel heating values

The following are nominal **net calorific values** from the IPCC inventory framework. Actual consignments vary, particularly with moisture, ash, and coal grade.

| Fuel | Nominal energy content | TCE representation |
| --- | --- | --- |
| Wood/wood waste | **15.6 MJ/kg** | Reference value; calculate moisture effects |
| Charcoal | **29.5 MJ/kg** | Separate good produced from wood |
| Peat | **9.76 MJ/kg** | Moisture-sensitive extracted fuel |
| Other bituminous coal | **25.8 MJ/kg** | Deposit-specific quality |
| Lignite | **11.9 MJ/kg** | Low-grade, often bulky fuel |
| Coke | **28.2 MJ/kg** | Processed fuel with separate production requirements |

These are **H as documented defaults**, but only **M as predictions for a particular historical fuel shipment**. [ResearchGate](https://www.researchgate.net/publication/260123453_Chapter_1_Introduction)

Additional conversion parameters:

| Parameter | Evidence or starting range | Confidence |
| --- | --- | --- |
| Traditional charcoal dry-mass yield | Approximately **16–30%** | **M**; kiln and operator dependent |
| Metal-kiln field-trial average | About **26%**, including fines | **M**, specific trial program |
| Open-fire useful-heat efficiency | Start with **5–15%** for cooking tasks | **P**; strongly task-dependent |
| Improved stove useful-heat efficiency | Start with **15–30%** | **P**, not a universal historical progression |
| Early atmospheric steam fuel-to-work efficiency | **0.5–1.5%** | **P**, for sensitivity testing |
| Early improved condensing steam | **2–5%** | **P**, not an exact fleet average |

The charcoal and cooking-efficiency literature contains highly context-specific measurements. Do not apply a cooking-pot efficiency to space heating or a kiln. [FAOHome](https://www.fao.org/4/s4550e/s4550e09.htm)

At a **20% dry-mass charcoal yield**, 1 kg of charcoal requires 5 kg of dry wood. Using 18 MJ/kg for the illustrative wood input, only about **one-third of its original energy** remains in the charcoal.

For steam, assuming coal at **25 MJ/kg**:

\[
m\_{\mathrm{coal}}=\frac{3.6}{25\eta}\quad\text{kg per mechanical kWh}
\]

Thus **1% efficiency requires 14.4 kg/kWh**, while **4% requires 3.6 kg/kWh**. That difference should materially alter the viable distance from a cheap fuel source.

### 2.4 Per-capita consumption: use named cases, not era-wide constants

| Case | Annual consumption per person | Interpretation and confidence |
| --- | --- | --- |
| FAO traditional domestic fuelwood planning case | Approximately **1,200 kg wood**, at 30% moisture | Household-use planning benchmark; **M** |
| FAO improved-appliance comparison | Approximately **450 kg wood** | Not an automatic technology multiplier; **M** |
| FAO charcoal-consuming household benchmark | **60–120 kg charcoal** | Excludes the upstream wood input; **M** |
| Netherlands, 1800, peat | Approximately **5.2 GJ/person/year** | Derived from rounded CBS fuel shares and total per-capita energy; **M** |
| Netherlands, 2022, reported total energy consumption | **154 GJ/person/year** | Published historical benchmark, not a claim about the current year; **H/M** |

The FAO figures are illustrative domestic consumption estimates, not globally representative preindustrial observations. [FAOHome](https://www.fao.org/4/x5328e/x5328e02.htm)

The Dutch peat estimate is calculated from CBS’s 1800 figures: peat supplied 11 PJ out of approximately 30.3 PJ, while reported total consumption was 14.4 GJ/person. The modern and historical series must be checked for accounting differences before comparison with another author’s reconstruction. [Centraal Bureau voor de Statistiek](https://www.cbs.nl/en-gb/news/2023/41/energy-consumption-per-capita-back-at-1970-level)

For England and Wales, Warde’s reconstruction reproduced by Wrigley provides a particularly useful transition benchmark:

| Period | Total reconstructed energy | Coal component | Coal share |
| --- | --- | --- | --- |
| 1560–1569 | **19.2 GJ/person/year** | **2.0 GJ** | **10.6%** |
| 1700–1709 | **29.6 GJ/person/year** | **14.7 GJ** | **49.7%** |
| 1800–1809 | **52.3 GJ/person/year** | **41.4 GJ** | **79.0%** |
| 1850–1859 | **96.5 GJ/person/year** | **88.8 GJ** | **92.0%** |

These totals include human food and draught-animal feed. They are reconstructed energy inputs, **not useful mechanical output**, and should not be directly compared with modern statistics that exclude food and fodder. Confidence is **M**, with greater uncertainty in early biomass estimates. [Scribd](https://www.scribd.com/document/962458261/E-a-Wrigley-The-Path-to-Sustained-Growth-Englands-Transition-From-an-Organic-Economy-to-an-Industrial-Revolution-Cambridge-University-Press-2016?utm_source=chatgpt.com)

### 2.5 Construction costs: site works can dominate

Documented medieval English cases show why a single “mill construction price” is inadequate:

| Case | Recorded cost | Interpretation |
| --- | --- | --- |
| Typical new windmill in the thirteenth-century discussion | Around **£10** | Local historical monetary scale |
| Milton Hall, Essex, 1299: watermill replaced by windmill | Just over **£15** | One documented substitution |
| Lydden, Kent, 1317: flood-related works | Over **£48** for extensive channel work; roughly **£54** overall | Civil works greatly exceeded the machinery-scale examples |

These are **M-confidence case observations**, not a cross-century real-price index. Do not convert them into modern money or assume a fixed wage equivalent. [dokumen.pub](https://dokumen.pub/working-with-water-in-medieval-europe-technology-and-resource-use-9004106804-9789004106802.html?utm_source=chatgpt.com)

For TCE, calculate construction from bills of timber, stone, earthmoving, metal fittings, millstones, skilled labor, and ordinary labor. Price those inputs using the simulated economy. Keep **machinery cost**, **site works**, and **land or water-right acquisition** separate.

---

## 3. Variation across eras and world regions

### 3.1 Era differences should emerge from constraints

The distinction between an economy drawing heavily on annually renewed biological resources and one exploiting large fossil stocks is useful. It is not sufficient, by itself, to explain every institutional or economic outcome. Wrigley’s work develops the energy constraint; archaeological research on ancient machinery demonstrates substantial mechanical sophistication long before industrialization. [DOI](https://doi.org/10.1098/rsta.2011.0568?utm_source=chatgpt.com)

| Historical setting | Appropriate TCE representation |
| --- | --- |
| **Foragers** | Muscle, collected fuels, portable equipment, and strong seasonal or climatic variation. Do not impose one universal per-capita fuel budget. |
| **Early farming** | Crop processing, cooking, land clearance, and storage create concentrated demands. Traction is available only where suitable animals, training, feed, and equipment come together. |
| **Preindustrial urban societies** | Specialized mills and workshops coexist with household manual work. Delivered fuel, waterways, rights, and market size strongly affect location. |
| **Industrializing societies** | Stored fuels can support much larger continuous power demands, but construction quality, fuel transport, finance, and repair remain constraints. |
| **Modern systems** | Electricity separates many end users from their prime movers. Network capacity, conversion losses, reliability, and the distinction between electricity and total energy become central. |

These are modeling categories, **not eras that should unlock automatically**.

### 3.2 Regional examples that should prevent a Europe-only model

| Region or tradition | Historical evidence | Simulation implication |
| --- | --- | --- |
| **China and adjacent regions** | Systematic coal exploitation is documented at Jirentaigoukou in Xinjiang around **1600 BCE**. This was a functioning extraction-and-use system, not industrial steam power. [PubMed](https://pubmed.ncbi.nlm.nih.gov/37494433/) | Coal knowledge can arise very early and remain limited to particular uses. Never make coal discovery synonymous with industrialization. |
| **Persia/Sistan** | Vertical-axis windmills were integrated with locally favorable wind conditions, construction materials, and settlement architecture. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/05786967.2021.1960885) | Permit a wind-centered mechanical tradition without requiring a European-style post mill first. |
| **Himalayan South Asia** | Research on *gharats* documents small watermills serving dispersed communities; a 2008 study estimated substantial seasonal operation alongside year-round mills. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0960148108000268) | Small direct-drive installations can remain useful despite low unit power and limited market size. |
| **Sub-Saharan Africa** | A study of draught cattle in eastern Uganda found substantial productivity losses associated with animal trypanosomiasis. Its estimated output reduction was about **21%** in that setting. [Springer](https://link.springer.com/article/10.1186/s13071-015-1191-9) | Animal power must respond to disease ecology and veterinary or husbandry capacity, not just species ownership. |
| **Pre-contact Andes** | Inka transport infrastructure supported human movement and llama caravans rather than an Old World draught-animal package. [Smithsonian Magazine](https://www.smithsonianmag.com/smithsonian-institution/how-inca-empire-engineered-road-would-endure-centuries-180955709/) | A society can develop extensive infrastructure without following a horse-and-cart branch. |
| **Northwestern Europe** | Dense milling, commercial peat use, and later coal expansion followed different geographic and institutional opportunities. Dutch peat and English coal should not be treated as the same transition. [Wageningen Research Portal](https://research.wur.nl/en/publications/peat-and-the-dutch-golden-age-the-historical-meaning-of-energy-at/) | Allow several prosperous energy configurations, with different vulnerabilities and expansion limits. |

**Do not encode these examples as permanent cultural bonuses.** Encode animal availability, disease, hydrology, wind, woodland, deposits, transport, inherited infrastructure, and institutions. Cultural transmission affects which solutions people know and consider.

---

## 4. Technology graph: capabilities, prerequisites, and unlocks

The dates below are **historical reference metadata**, not simulation gates. “First appearance” is frequently an earliest surviving attestation or a well-documented milestone rather than a securely identified invention.

Every node additionally requires knowledgeable people, suitable tools, and access to inputs. Separate four states:

**Known → buildable → operable → economically attractive.**

### 4.1 Preindustrial branch

| Node | Principal prerequisites | Approximate appearance or reference milestone | Concrete unlocks |
| --- | --- | --- | --- |
| **Fuel preparation and seasoning** | Fire use; cutting or collection; sheltered storage | Prehistoric; no defensible single origin **[L]** | Split and dried fuelwood; lower-moisture fuel inventories |
| **Controlled charcoal production** | Wood supply; controlled oxygen exclusion; operator knowledge | Prehistoric; exact origin poorly resolved **[L]** | Charcoal good; pits/kilns; charcoal-dependent thermal recipes |
| **Draught training and fitted harness** | Suitable domesticated animals; training; ropes, leather, or wooden yokes | Well established in parts of Eurasia by the **fourth millennium BCE**, with earlier evidence debated **[M]** | Plough traction, hauling, sledges, later carts |
| **Improved equine harness** | Trained equids; suitable padding, leatherwork, and harness geometry | Developed horse collar: **China around the fifth century CE**; later European adoption **[M]** | Better sustained traction in suitable tasks; reduced harness-related limitations |
| **Rotary animal drive** | Trained animals; bearings; shaft construction; suitable harness | Hellenistic/Roman Mediterranean attestations; ancient origins vary by device **[M]** | Animal mills, capstans, rotary pumping, crushing |
| **Direct-drive watermill** | Waterwheel construction; bearings; millstones; usable flow and fall | Ancient Mediterranean, around the **last centuries BCE**; priority uncertain **[M]** | Small horizontal-wheel grain mills |
| **Geared vertical watermill** | Wheel and bearing craft **AND** gearing; suitable hydraulic site | Described in the **first century BCE** Mediterranean **[H for attestation]** | Alternative shaft orientation; geared milling; broader machine layouts |
| **Controlled headworks and overshot configurations** | Hydraulic construction **AND** suitable wheel design; sufficient elevation | Roman-period installations, including the early centuries CE **[M]** | Higher-head sites, stored water, cascaded installations |
| **Rotation-to-reciprocation attachments** | Rotary drive **AND** cam, crank, or linkage craft | Application-specific ancient and medieval developments **[M/L]** | Powered stamps, hammers, bellows, and saws; separate recipes for each |
| **Vertical-axis windmill** | Rotor and bearing craft; suitable grain-processing equipment; favorable wind regime | Sistan/Persian region, securely described by the **ninth–tenth centuries CE** **[M]** | Wind-powered milling adapted to directional winds |
| **Horizontal-axis windmill** | Sails/rotor craft; bearings; gearing; orientation control | Northwestern Europe, **late twelfth century CE** **[M]** | Post mills and later configurations; milling where water sites are poor |
| **Commercial peat fuel** | Accessible peat; cutting; drying grounds; transport | Ancient use; major medieval/early-modern development in northwestern Europe **[M/L]** | Peat fuel good, peat works, alternative domestic and industrial heat |
| **Coal extraction and combustion** | Identified deposit; excavation; sorting; compatible combustion practices | Systematic use at Jirentaigoukou, Xinjiang, around **1600 BCE** **[H/M]** | Coal fuel good; appropriate hearth, kiln, and furnace recipes |

The traction chronology is supported by archaeological work that also emphasizes uneven regional adoption. Harness chronology is less secure than many game technology trees imply. [Springer](https://link.springer.com/article/10.1007/s12520-026-02455-z)

Ancient waterpower and machine applications are treated in Wilson’s archaeological synthesis; Barbegal provides direct evidence for a substantial Roman milling installation. Persian windmills require their own technical branch rather than being represented as an inferior European design. [DOI](https://doi.org/10.2307%2F3184857)

For an early-agrarian start, fire use and basic fuel preparation normally belong in the initial capability set. Sailing, wheeled transport, furnace construction, and metalworking should connect to this branch from the transportation and materials graphs rather than being duplicated here.

### 4.2 Industrial and electrical branch: post-v1

| Node | Principal prerequisites | Documented milestone | Concrete unlocks |
| --- | --- | --- | --- |
| **Coke-compatible ironmaking** | Coal carbonization **AND** compatible furnace, charge, and blast practice | Coalbrookdale, Britain, **1709**, a documented industrial milestone **[H]** | Coke fuel route; expansion of coal-based iron production |
| **Atmospheric steam pumping** | Boiler construction; cylinders and pistons; valves/seals; pumps; adequate metalworking | Newcomen installation, Britain, **1712** **[H]** | Mine drainage and stationary pumping |
| **Separate condenser** | Steam engine capability; condenser; cooling-water and vacuum-management arrangements | Watt’s development, Britain, **1765–1769** **[H]** | Lower fuel consumption for appropriate engine configurations |
| **Controlled rotary steam drive** | Suitable steam engine **AND** rotary mechanism, flywheel, regulation, transmission | Commercial development in Britain during the **1780s** **[H/M]** | Factory shaft power and a broader set of machinery |
| **Higher-pressure steam systems** | Stronger, better-made boilers; improved joints, valves, and operating practice | Britain/United States, around **1800 and early nineteenth century** **[M]** | More compact engines; expanded transport and industrial applications |
| **Hydraulic turbines** | Advanced hydraulic design; precise metal manufacture; suitable waterways | France and subsequent European/US development, **early–mid nineteenth century** **[M]** | Larger or more efficient hydraulic installations |
| **Electrochemical battery** | Suitable metals; electrolyte; separators and connections | Volta, Italy, **1800** **[H]** | Sustained experimental current; electrochemistry and later communication devices |
| **Electromagnetic induction** | Experimental electrical apparatus; conductors; magnetic materials; mechanical motion | Faraday, Britain, **1831** **[H]** | Generator and transformer principles |
| **Practical dynamo** | Induction knowledge; insulated windings; magnetic circuit; switching/commutation; mechanical drive | Practical designs in the **1860s** **[H/M]** | Useful-scale electricity generation |
| **Practical electric motor** | Electromagnetic machinery; insulation; reliable supply and control | Experimental rotation **1821**; useful industrial machinery developed later **[H/M]** | Distributed mechanical drives |
| **Local electrical distribution** | Generator **OR** another adequate supply; wires; insulation; protection; compatible loads | Commercial systems in the **1880s**, including New York’s **1882** Pearl Street station **[H]** | Lighting networks, metering, local electric services |
| **Transformer-based AC distribution** | Alternating-current generation; transformers; insulation; network control | European/US developments in the **1880s**; documented demonstration at Great Barrington in **1886** **[H]** | Economical voltage transformation and expanded distribution distances |

These milestones are documented by museum collections, engineering-history institutions, and surviving apparatus. [National Trust](https://www.ironbridge.org.uk/our-story/the-iron-bridge/?utm_source=chatgpt.com)

### Critical prerequisite rules

**Steam must not require coal specifically.** Its physical requirements are a suitable heat source, water, machinery, and operating competence. Coal can make an installation economical without being a logical prerequisite.

**Electricity must not require steam.** A waterwheel or turbine can drive a generator. Experimental electricity can also develop through batteries before useful generators exist.

**A horizontal-axis windmill need not require a vertical-axis windmill.** They are alternative design families.

**A geared watermill need not require every simpler mill design to have been invented first.** Capabilities such as bearings, gearing, hydraulic construction, and millstones can combine through several paths.

**Coke-based ironmaking is not a prerequisite for the first steam engine.** Specify required castings and fabrication quality, not a historically familiar but unnecessarily restrictive fuel chain.

---

## 5. Stylized facts a correct simulation should reproduce

### 5.1 A few kilowatts can transform a particular occupation

A mill need not be industrially enormous to displace large amounts of repetitive muscle work. Yet its economy-wide effect depends on demand for the product and what displaced workers do next.

**Test:** adding a 5 kW mill should sharply change grinding labor requirements, but should not automatically multiply construction, farming, or administrative productivity.

### 5.2 Significant mechanization should be possible before industrialization

The Roman complex at Barbegal contained **16 waterwheels arranged in two cascades**. It demonstrates substantial organized mechanical production in antiquity; it should not be treated as an ordinary village mill or as proof that every Roman region was similarly mechanized. [ResearchGate](https://www.researchgate.net/publication/327459804_The_second_century_CE_Roman_watermills_of_Barbegal_Unraveling_the_enigma_of_one_of_the_oldest_industrial_complexes)

**Test:** a sufficiently organized preindustrial society should be able to construct a large milling complex without steam, electricity, or a medieval-era flag.

### 5.3 Mill adoption can become dense without every household owning machinery

Smil reports approximately **5,600 watermills** in the Domesday record of 1086. Exact coverage and counting conventions matter, but the order of magnitude indicates widespread service infrastructure rather than isolated curiosities. [Vaclav Smil](https://vaclavsmil.com/wp-content/uploads/2024/10/smil-article-2004world-history-energy.pdf)

**Test:** dense agricultural settlement plus accessible sites and workable institutions should support many shared or commercial mills, with customer catchments and queues.

### 5.4 Coal use should precede—and sometimes never lead to—steam

England and Wales already obtained roughly **half their reconstructed energy from coal in 1700–1709**, before the 1712 Newcomen milestone. Much earlier coal use in Xinjiang likewise did not automatically produce an industrial revolution. [Scribd](https://www.scribd.com/document/962458261/E-a-Wrigley-The-Path-to-Sustained-Growth-Englands-Transition-From-an-Organic-Economy-to-an-Industrial-Revolution-Cambridge-University-Press-2016?utm_source=chatgpt.com)

**Test:** coal deposits can support thermal industries for centuries without an engine breakthrough.

### 5.5 New power sources should complement old ones

At Boulton’s Soho works, drought previously required **six to ten horses** to supplement waterpower. *Old Bess* subsequently pumped water back to the headrace. This is an excellent validation case for infrastructure reuse and mixed prime movers. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co50947)

**Test:** permit an engine to operate a pump that sustains existing water-powered machinery. Do not force an immediate factory rebuild.

### 5.6 Seasonal power and maintenance should affect services visibly

The Himalayan *gharat* study estimated that about **25%** of the mills in its regional assessment operated seasonally, alongside year-round and defunct installations. The exact proportions are not globally transferable; the coexistence is the important pattern. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0960148108000268)

**Test:** drought should create queues, higher milling charges, inventory drawdowns, and renewed manual or animal operation—not merely a hidden percentage penalty.

### 5.7 Efficiency gains should sometimes expand demand rather than only save fuel

Fouquet and Pearson’s reconstruction of British lighting finds that from 1800 to 2000 the real price of lighting services fell by more than **3,000-fold**, while per-capita consumption increased by roughly **6,500-fold**. These figures are specific to lighting, not a universal rebound coefficient. [Sage Journals](https://journals.sagepub.com/doi/abs/10.5547/ISSN0195-6574-EJ-Vol27-No1-8)

**Test:** cheaper energy services should sometimes enable more lighting, pumping, heating, or processing. Fixed demand would suppress an important route by which energy technologies changed economies.

---

## 6. Recommended implementation for TCE

### 6.1 State should live on people, installations, resources, and institutions

| Entity | Minimum relevant state |
| --- | --- |
| **Person** | Skills; current task; fatigue; health; nutrition; available labor time |
| **Household** | Food and fuel stocks; desired cooking/heating services; budget; access rights |
| **Work animal** | Body size; condition; training; harness; feed/water needs; fatigue; competing uses |
| **Installation** | Owner; operator requirements; capacity; efficiency curve; input/output interfaces; wear; repair needs |
| **Mechanical network** | Connected machines; power sources; losses; compatible motion; allocation priorities |
| **Resource patch** | Woodland biomass/increment; peat or coal reserves; accessibility; extraction cost |
| **Water system** | Reach flow; elevation; storage; diversions; withdrawals; flood and sediment exposure |
| **Institution** | Water and woodland rights; tolls; maintenance obligations; finance; enforcement |

Do not give an entire polity a permanently available power total. A person or firm must arrange access to a functioning installation.

### 6.2 Use three explicit service interfaces

A useful architecture has separate interfaces for:

**Mechanical work:** power, delivered work, and a coarse motion class such as rotary, reciprocating, or traction.

**Heat:** useful energy, temperature capability, and any recipe-specific fuel restrictions.

**Electricity:** generated energy, network capacity, losses, and reliability.

This allows a later electrical system to reuse existing machine-demand logic: the electric motor replaces the local prime mover, while the machine still consumes mechanical work.

### 6.3 Model institutions through decisions with visible consequences

A village assembly may allocate woodland cutting rights. A landlord may finance a mill and charge tolls. A monastery, temple, merchant partnership, or state workshop may concentrate capital and skilled labor. A water association may coordinate channel maintenance.

For each arrangement, simulate who pays, who gains access, who does maintenance, who bears interruption risk, and who can exclude others. The same physical mill can produce very different economic outcomes under different rights.

The adoption decision should compare **expected service demand and operating costs**, not simply check whether a more advanced node exists.

### 6.4 Simplify machinery; preserve the important constraints

For v1, detailed rigid-body simulation of every gear is unnecessary. A cached connected-component model can represent a shaft network, with coarse losses, operating thresholds, and machine compatibility.

A practical starting schedule is:

| Subsystem | Suggested update strategy |
| --- | --- |
| Agent work, fatigue, and operating decisions | Existing agent/task ticks |
| Mechanical allocation | When loads, sources, or topology change; otherwise cached |
| Wind availability | Shared weather updates |
| River flow and pond storage | Hydrological ticks appropriate to storage size |
| Fuel inventories and production | Production/logistics events |
| Wear and maintenance | Accumulated operating hours plus periodic checks |
| Woodland growth | Coarse ecological updates |
| Investment and institutional decisions | Infrequent planning events |

These are architectural recommendations, not a demonstrated performance benchmark. Their purpose is to avoid reevaluating every agent against every power source or simulating machinery at render-frame frequency.

### 6.5 Keep v1 narrow but extensible

A strong v1 can cover **grinding, pressing/crushing, pumping, hammering/blowing, traction, and useful heat**. Each family should have manual, animal, or mechanically powered alternatives where physically appropriate.

Do not attempt to simulate every named historical machine immediately. Author a smaller number of meaningful capabilities with concrete recipes and equipment variants. Preserve enough structure that steam and electricity can later replace or augment a power source without rewriting production.

### 6.6 Existing models and games worth borrowing from

| Example | Useful precedent | What not to import uncritically |
| --- | --- | --- |
| **MUSE** | Heterogeneous investment actors; technology choices; imperfect foresight | Its sectoral/global scale is not an individual-person simulation. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2211467X22001584) |
| **Vintage Story** | Spatially legible mechanical networks connecting shafts, gearing, windpower, and processing machinery | Its game-specific power units and balancing are not historical measurements. [Vintage Story Wiki](https://wiki.vintagestory.at/Special%3AMyLanguage/_Mechanical_power) |
| **Factorio** | Explicit fuel–fluid–machine chains and throughput bottlenecks | Its machinery scales and frictionless organizational assumptions are unsuitable as preindustrial calibration. [Official Factorio Wiki](https://wiki.factorio.com/Steam_engine) |

The most important lesson is to make the player able to **see why production stopped**: no wind, low pond, exhausted ox, missing charcoal, damaged gearing, unpaid operator, or a contested water diversion.

---

## 7. Sources, datasets, and remaining uncertainty

### Most useful foundations

| Source | Best use for TCE | Main limitation |
| --- | --- | --- |
| **Fraenkel, *Water Lifting Devices*, FAO** | Human/animal output, pumping, practical prime-mover comparisons | Modern engineering guidance, not an ancient census. [FAOHome](https://www.fao.org/4/ah810e/AH810E08.htm) |
| **FAO, *Simple Technologies for Charcoal Making*** | Conversion yields, wood requirements, production and transport organization | Local fuels, kilns, and operators vary substantially. [FAOHome](https://www.fao.org/4/x5328e/x5328e02.htm) |
| **IPCC 2006 Guidelines, Volume 2, Chapter 1** | Consistent fuel-energy units and reference calorific values | Inventory defaults do not specify historical fuel quality. [ResearchGate](https://www.researchgate.net/publication/260123453_Chapter_1_Introduction) |
| **Warde, *Energy Consumption in England & Wales, 1560–2000*; Wrigley’s reproductions and analysis** | Long-run energy composition and transition calibration | Reconstructed biomass, food, and feed require careful accounting. [EconBiz](https://www.econbiz.de/Record/energy-consumption-in-england-wales-1560-2000-warde-paul/10003720134) |
| **Malanima, “The Limiting Factor,” 2020, and supplementary world-energy database** | Regional comparisons from the nineteenth century onward; supplementary series extend to 2016 | Historical traditional-energy estimates are much less secure than modern commercial-energy records. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.12913) |
| **Wilson, “Machines, Power and the Ancient Economy,” 2002** | Ancient mechanical applications and the economic interpretation of archaeological evidence | Surviving sites are uneven and not a representative sample of all production. [DOI](https://doi.org/10.2307%2F3184857) |
| **Pujol and colleagues; Quaranta and Revelli** | Waterwheel performance and design-sensitive efficiencies | Experimental and optimized wheels must not be mistaken for historical averages. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0960148115003493) |
| **Fouquet and Pearson, 2006** | Energy-service prices, consumption, and expanding demand | Lighting is an unusually dramatic case, not a universal elasticity. [Sage Journals](https://journals.sagepub.com/doi/abs/10.5547/ISSN0195-6574-EJ-Vol27-No1-8) |

### Claims that should remain explicitly uncertain

**Exact invention dates.** Wooden machinery rarely survives, and textual descriptions can be ambiguous. Distinguish “earliest presently known evidence” from invention and from widespread adoption.

**Universal mill costs and efficiencies.** The evidence supports engineering relationships and particular cases much better than a worldwide average installation.

**A single energy-consumption number for an era.** Climate, settlement density, industry, transport, diet, and accounting conventions can overwhelm an “ancient versus medieval” distinction.

**The claim that slavery prevented ancient mechanization.** Archaeological evidence makes a simple one-cause explanation inadequate. Labor institutions matter, but so do demand, geography, investment, and the availability of useful machinery. [DOI](https://doi.org/10.2307%2F3184857)

**The claim that coal alone caused industrialization.** Coal relaxed an important resource constraint, but recognizing a deposit did not supply engines, metallurgy, transport, skilled labor, finance, or markets. The contrast between ancient coal exploitation and much later industrial systems is particularly instructive. [PubMed](https://pubmed.ncbi.nlm.nih.gov/37494433/)

**Bottom line:** TCE should make energy transitions emerge when people can assemble a cheaper, more reliable, or more capable way to perform a valued task. Geography establishes opportunities; machines convert them; institutions determine access and investment; agents decide whether the resulting service is worth using. That structure can produce water-rich manufacturing regions, animal-powered agricultural economies, peat-fueled towns, early coal users without steam, and hydroelectric societies that never follow Britain’s exact path.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556-tce-research/c/6ab92858-a368-83e9-a5af-7f3dd9087f80)
