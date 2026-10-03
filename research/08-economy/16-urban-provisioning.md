# Feeding and fueling cities: provisioning and hinterlands

## Central finding

**A city’s provisioning limit is not a fixed radius. It is the amount of food, usable energy, and safe water that its inhabitants can obtain through functioning production, transport, storage, and allocation systems.** Rome combined public grain distribution with private commerce; medieval London drew supplies along waterways and through intermediate markets; Chinese governments maintained granaries while also managing long-distance grain transport. These were different institutional solutions to overlapping physical problems. [Cambridge University Press](https://www.cambridge.org/core/books/abs/metropolis-and-hinterland/model-of-agricultural-change/CAEB744E31AE143443AF67102DA7CFB0)

For TCE, the most productive abstraction is **a physical supply network overlaid with ownership and access rights**. A sack of grain has a location and an owner; a household needs both delivery and a legitimate—or illicit—way to acquire it. Food can exist without reaching the city, reach the city without reaching the poor, or reach households without sufficient fuel and water to prepare it.

The recommendations below distinguish **historical observations**, **engineering analogues**, and **proposed simulation parameters**. They should not be collapsed into a single supposedly universal “pre-industrial economy.”

---

## 1. Mechanisms: implementable causal rules

### 1.1 Cities depend on exportable surplus, not gross agricultural output

Farm production must support agricultural households, seed, animals, storage losses, and reserves before it can sustain urban consumers. Historical reconstructions such as London’s *Feeding the City* project therefore combine harvests with seeding rates, consumption, processing losses, and population estimates—not simply cultivated acreage. [IHR Web Archives](https://archives.history.ac.uk/cmh/arpt97con.html)

A useful annual accounting rule is:

\[
Q\_{\text{voluntary sale}}
=
\max\left[
0,\;
H+S\_{\text{opening}}
-D\_{\text{household}}
-D\_{\text{seed}}
-D\_{\text{feed}}
-L
-S\_{\text{target}}
-Q\_{\text{obligations}}
\right]
\]

Here, \(H\) is harvest and all terms are quantities of the same commodity over the same accounting period.

**Taxes, rents, and tribute are transfers, not disappearance.** Grain removed through \(Q\_{\text{obligations}}\) must enter somebody else’s inventory. Obligations can also exceed the producer’s safe surplus, leaving the countryside hungry while the capital remains provisioned.

This creates a powerful nonlinear failure mechanism. In an illustrative farm economy producing 1,000 units and retaining 800, the exportable surplus is 200. A 20% harvest decline eliminates that surplus entirely unless someone reduces consumption, draws down stocks, or violates another claim. This is arithmetic, not a universal historical elasticity—but it is an important simulation test.

**TCE rule:** calculate exportable surplus from household and institutional decisions. Do not make a fixed percentage of every harvest automatically available to cities.

### 1.2 Transport creates economic distance—and reshapes land use

For each potential shipment, calculate:

\[
C\_{\text{delivered}}
=
C\_{\text{acquisition}}
+C\_{\text{loading}}
+C\_{\text{haulage}}
+C\_{\text{handling}}
+C\_{\text{storage}}
+C\_{\text{tolls}}
+C\_{\text{expected loss}}
\]

Haulage includes crews, animals, fodder, vehicle capacity, return journeys, maintenance, and travel time. Routes should distinguish road, river, canal, coastal, and open-sea movement. Stanford’s ORBIS provides a reusable precedent: it models Roman connectivity through transport costs and travel times rather than geographic distance alone. [Stanford History Department](https://history.stanford.edu/publications/orbis-stanford-geospatial-network-model-roman-world)

Land-use choice can then follow expected annual returns:

\[
\pi\_j(x)=p\_jq\_{j,\text{delivered}}(x)-c\_{j,\text{production}}(x)-c\_{j,\text{transport}}(x)-\text{dues}\_j(x)
\]

The familiar **von Thünen rings** are a special case: one market, uniform land, and transport costs increasing smoothly with distance. TCE should instead generate distorted, overlapping supply zones around navigable water, roads, competing towns, and political boundaries.

The historical ordering is subtler than “vegetables, forest, grain, livestock.” Around medieval London, low-value, bulky oats and rye tended to be produced closer to the city, while more valuable wheat could bear longer transport. [IHR Web Archives](https://archives.history.ac.uk/cmh/arpt97con.html)

**TCE rule:** author commodity properties—bulk, value, perishability, processing requirements—and let supply geography emerge. Do not author permanent rings.

### 1.3 Storage connects seasonal production to daily consumption

Separate three functions:

**Seasonal working stocks** bridge harvests. **Commercial inventories** support milling, baking, retailing, and transport schedules. **Emergency reserves** cover unexpected disruptions. Counting all three as a single “granary capacity” hides important behavior.

Chinese granaries illustrate that storage was an active institution: stocks required rotation, sales or loans, replenishment, financing, and oversight. A building’s nominal capacity was not equivalent to usable relief grain. [Spot Colorado](https://spot.colorado.edu/~shiue/Shiue%20proofs1.pdf)

For each stock batch, retain commodity, quantity, age, moisture or storage condition, owner, and accessibility. Apply losses at the relevant stage:

\[
S\_{t+\Delta t}=S\_t+\text{receipts}-\text{dispatches}-\text{consumption}-\text{losses}
\]

Normal deterioration should differ from catastrophic events such as flooding, fire, infestation, or theft.

**TCE rule:** merchants and institutions order against expected demand during replenishment time, with additional buffers for uncertainty. Harvest timing must influence desired inventories; a constant stock target throughout the year is inappropriate.

### 1.4 Fuel is both a household necessity and a production input

Represent demand as:

\[
E\_{\text{fuel}}
=
E\_{\text{cooking}}
+E\_{\text{space heating}}
+E\_{\text{processing}}
+E\_{\text{industry}}
\]

Then convert useful energy into fuel mass through fuel quality and equipment efficiency. Cooking demand should partly scale with household meals; heating should depend on buildings and weather, not simply population. Bakeries, breweries, kilns, metalworking, and other activities need separate fuel recipes.

Charcoal changes transport economics rather than creating energy. FAO’s charcoal-production guidance describes substantial reductions in transported mass when wood is carbonized near its source, making distant woodland more accessible to consumers. The conversion nevertheless consumes wood and loses energy. [FAOHome](https://www.fao.org/4/x5328e/x5328e04.htm)

Track woodland as a renewable stock:

\[
B\_{t+1}=B\_t+G(B\_t,\text{age},\text{climate},\text{management})-\text{removals}-\text{other losses}
\]

Distinguish **degradation**, where biomass declines but woodland remains, from **deforestation**, where land converts to another use. Field research in Zambia shows why this matters: charcoal can be associated with agricultural clearing in one tenure system and repeated woodland degradation in another. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0305750X21002151)

**TCE rule:** urban fuel demand can stimulate managed woodland, expand a depletion frontier, encourage imports, or induce substitution. “City grows → nearby trees vanish permanently” should be only one possible outcome.

### 1.5 Water requires a watershed, a delivery system, and safe access

Model water in two connected layers.

The **hydrological layer** contains rainfall, runoff, rivers, springs, groundwater, and reservoirs:

\[
S\_{t+1}=
\operatorname{clamp}
\left(
S\_t+\text{inflows}-\text{withdrawals}-\text{evaporation}-\text{leakage},
0,S\_{\max}
\right)
\]

The **service layer** contains wells, channels, aqueducts, pipes, pumps, fountains, carriers, and household vessels. A city may have abundant source water but inadequate delivery capacity.

Track quality separately from quantity. Research at Tikal found evidence of contamination in major reservoirs, while a different reservoir contained evidence of a zeolite-based filtration system. A reservoir being full is therefore not sufficient evidence of secure drinking water. [Nature](https://www.nature.com/articles/s41598-020-67044-z)

**TCE rule:** household water acquisition consumes time or money. Delivered quantities depend on source reliability, queues, carrying capacity, network connections, and access rights. Irrigation, workshops, animals, and domestic users compete for withdrawals; do not include agricultural irrigation inside a household litres-per-day coefficient.

### 1.6 Institutions determine who bears shortages

Provisioning institutions can combine procurement, compulsory transfers, subsidized sales, free distributions, merchant incentives, reserve purchases, and infrastructure maintenance. Rome’s annona was not a replacement for all markets; public intervention and private supply operated together. [Cambridge University Press](https://www.cambridge.org/core/books/abs/metropolis-and-hinterland/model-of-agricultural-change/CAEB744E31AE143443AF67102DA7CFB0)

Represent an institution through composable rules:

| Institutional rule | Necessary simulation consequence |
| --- | --- |
| Grain tax or tribute | Transfers grain from specified producers; creates transport and storage obligations. |
| Subsidized sale | Requires purchased or collected stock and a budget covering the subsidy. |
| Free ration | Requires eligibility, allocation frequency, distribution capacity, and actual inventory. |
| Reserve purchase | Supports inventories but can compete with current consumers and merchants. |
| Emergency release | Changes accessible supply; effectiveness depends on location and recipients. |
| Export restriction | Redirects local trade but may reduce producer incentives or provoke evasion. |
| Protected urban supply | Can move the shortage burden onto rural households or politically weaker towns. |

These are implementation possibilities, not claims that every government used every mechanism.

**TCE rule:** separate actual stocks from officials’ reported stocks. Information delays, diversion, weak inspection, and inadequate replacement funding should produce failures without requiring a scripted “corruption event.”

### 1.7 Famine has availability and access pathways

Sen’s entitlement approach emphasizes that people obtain food through production, exchange, employment, and enforceable claims. Losing those claims can cause hunger without all regional food disappearing. This complements—not replaces—physical supply analysis. [OUP Academic](https://academic.oup.com/book/32827/chapter-abstract/275134412)

Monitor both:

\[
\text{physical cover}=
\frac{\text{usable local food stock}}{\text{normal daily requirement}}
\]

and household-level **accessible cover**, which includes only food a household can actually obtain through its inventories, income, transfers, or other claims.

A disruption may create a sequence of failures: shipments stop; prices or queues increase; poorer households reduce meals; work and care become harder; households sell productive assets; later recovery becomes more difficult. Fuel and unsafe water can compound the crisis.

**TCE rule:** do not wait until a city-wide food counter reaches zero before generating hunger, migration, distress sales, or political pressure. Also avoid converting one day of underconsumption directly into death; distinguish short deficits from prolonged deprivation.

---

## 2. Quantitative parameters and hinterland calculations

### 2.1 Evidence-backed reference values

**Confidence:** **H** means well-supported for the stated physical relationship or documented benchmark; **M** means reconstructed, modeled, or context-specific; **L** means thin evidence or weak transferability. These are evidence classifications, not statistical confidence intervals.

| Parameter or benchmark | Value and units | Scope and appropriate use | Confidence / source |
| --- | --- | --- | --- |
| Adult physical-activity multiplier | **1.40–1.69** light; **1.70–1.99** moderate; **2.00–2.40** vigorous | Multiply individual basal metabolic requirements; do not give every person the same calorie requirement. | H for framework. FAO/WHO/UNU, *Human Energy Requirements*. [FAOHome](https://www.fao.org/4/y5686e/y5686e07.htm) |
| Human porter payload | Approximately **20 kg/load** | Indicative low-technology transport example, not an absolute physiological maximum. | M. FAO rural–urban marketing guidance. [FAOHome](https://www.fao.org/4/a0159e/a0159e03.htm) |
| Pack-donkey payload | Approximately **70–120 kg/load** | Modern working-animal analogue; adjust for animal size, terrain, condition, and journey. | M. FAO animal-transport guidance. [FAOHome](https://www.fao.org/4/x5483b/x5483b0w.htm) |
| Cart payload examples | **400 kg** donkey cart; **1,000 kg** ox cart | Illustrative vehicle capacities, not universal ancient specifications. | M. FAO. [FAOHome](https://www.fao.org/4/a0159e/a0159e03.htm) |
| Walking/draft movement | Approximately **3–5 km/hour** | Moving speed, not daily average including loading, rests, and return travel. | M. FAO examples. [FAOHome](https://www.fao.org/4/a0159e/a0159e03.htm) |
| Post-harvest cereal losses | **12.4–16.3%** in the cited Rwanda 2021 crop estimates | APHLIS estimates for the **whole post-harvest chain**, not annual warehouse spoilage alone and not measurements of antiquity. | M within the modeled context; L for historical transfer. [Aphlis](https://www.aphlis.net/en/data/tables/overview/RW/all-crops/2021) |
| Air-dried wood energy content | Approximately **15 MJ/kg**, at roughly **20% moisture** | Keep moisture basis explicit; useful heat also depends on combustion equipment. | H as an engineering reference. FAO. [FAOHome](https://www.fao.org/4/T1804E/t1804e0b.htm) |
| Practical wood-to-charcoal requirement | Approximately **5 tonnes wood per tonne charcoal** | Broad production-planning reference. Moisture and kiln method matter; do not substitute theoretical laboratory yield. | M. FAO, *Simple Technologies for Charcoal Making*. [FAOHome](https://www.fao.org/4/x5328e/x5328e03.htm) |
| Sustainable forest area for charcoal supply | Approximately **0.05–0.5 ha/person** | FAO’s contrasting managed-plantation and natural-forest examples for charcoal consumers—not a universal city footprint. | M within assumptions; L as a global historical constant. [FAOHome](https://www.fao.org/4/x5328e/x5328e03.htm) |
| Historical firewood-use estimates | Approximately **1 kg/person/day** in southern Europe versus **7–10 kg/person/day** in Scandinavian estimates | Broad historical syntheses; climate, uses included, and measurement conventions vary. Do not treat these as standardized household surveys. | M–L. Estimates summarized by Iriarte-Goñi and Ayuda, citing Sieferle, Malanima, and Warde. [Historia Ambiental](https://historia-ambiental.com/FILE/bibliografia/A.16._Iriarte_Goni_I._and_Ayuda_M.I.pdf) |
| Domestic water with basic off-premises access | Average quantity **unlikely to exceed 20 L/person/day** | WHO service-access category, associated with roughly **5–30 minutes** collection time; not a universal “safe minimum.” | M as a cross-context service benchmark. [UEA Digital Repository](https://ueaeprints.uea.ac.uk/id/eprint/79561/1/Published_Version.pdf) |
| Improved domestic water access | Approximately **50 L/person/day** intermediate; **more than 100 L/person/day** optimal | WHO categories associated with increasingly convenient, continuous supply. Exclude irrigation and major industrial use. | M as service benchmarks; not historical city averages. [UEA Digital Repository](https://ueaeprints.uea.ac.uk/id/eprint/79561/1/Published_Version.pdf) |

A particularly important charcoal distinction: FAO reports a theoretical yield around **33% of oven-dry wood at 500°C**, but this excludes wood burned to supply process heat. Using that value as whole-system conversion efficiency would understate the forest requirement. [FAOHome](https://www.fao.org/4/x5328e/x5328e05.htm)

### 2.2 Proposed TCE starting ranges—not historical measurements

These are deliberately provisional parameters for sensitivity testing.

| Parameter | Starting range | Implementation note | Evidence status |
| --- | --- | --- | --- |
| City-gate cereal requirement for a cereal-heavy diet | **200–300 kg/resident/year**; example baseline **250** | Calibration aggregate, not a universal ration. Resolve actual diets through food energy, processing, and other foods. | Proposed prior; L. |
| Delivered cereal surplus per hectare of supplying agricultural territory | **25–200 kg/ha/year** | An **output to measure**, not a fixed farm productivity coefficient. Includes rural needs, fallow, and delivery losses. | Proposed sensitivity range; L. |
| Geographic share represented by supplying agricultural territory | **0.25–0.75** | Remaining area may contain forests, settlements, water, unsuitable land, or unrelated production. | World-generation parameter; L. |
| Additional disruption buffer | **7–90 days** of the protected population’s requirements | Additional to seasonal working stocks; ownership and eligibility must be specified. | Institutional experiment; L. |
| Total post-harvest chain loss sensitivity | **5–25%** under non-catastrophic scenarios | Allocate among stages; do not apply the full percentage independently at every stage. | Proposed bracket, informed by modern loss estimates rather than ancient universal data. |
| Route interruption stress tests | **7, 30, and 90 days** | Test a bridge, port, canal, or supplier failure without preselecting the outcome. | Experimental settings, not historical frequency estimates. |

Historical evidence is stronger for particular cities, technologies, and episodes than for universal values of “hectares per urban resident.” The correct response is to expose uncertainty—not disguise a convenient default as a measured constant.

### 2.3 How much hinterland does a city need?

For an illustrative cereal calculation, define:

* \(P\): urban population.
* \(g\): annual city-gate cereal requirement, kg/person.
* \(q\): annual cereal delivered to the city per hectare of supplying agricultural territory, after rural requirements and relevant losses.
* \(f\): supplying agricultural territory as a share of the surrounding geographic area.

Then:

\[
A\_{\text{agricultural}}=\frac{Pg}{q}
\]\[
A\_{\text{geographic, km}^2}=\frac{Pg}{100qf}
\qquad
r\_{\text{equivalent}}=\sqrt{\frac{A\_{\text{geographic}}}{\pi}}
\]

Using **\(g=250\)** and **\(f=0.5\)**:

| Delivered surplus \(q\) | Supplying agricultural area per urban resident | Equivalent radius: 10,000 residents | Equivalent radius: 50,000 residents |
| --- | --- | --- | --- |
| 25 kg/ha/year | 10.0 ha | 25.2 km | 56.4 km |
| 100 kg/ha/year | 2.5 ha | 12.6 km | 28.2 km |
| 200 kg/ha/year | 1.25 ha | 8.9 km | 19.9 km |

**These are derived scenarios, not observed historical hinterland boundaries.** They illustrate sensitivity to surplus and usable land. They exclude separate requirements for non-cereal foods, urban animals, and fuel.

In an actual simulation, replace the circle with supplying parcels connected by viable routes. Food and fuel sources can be discontinuous, overlap with other cities’ supply regions, and extend much farther along water transport.

Fuel and water require different footprint calculations:

\[
A\_{\text{woodland}}
=
\frac{\text{annual dry-wood removals}}
{\text{sustainable dry-wood increment per hectare}}
\]\[
A\_{\text{water catchment}}
\approx
\frac{\text{annual withdrawals}}
{\text{capturable runoff per unit area}}
\]

Neither guarantees supply: woodland must be accessible and harvestable; runoff must arrive at usable times and be stored or conveyed. Nor should these areas simply be added—woodlands can protect watersheds, and farms can produce some fuel alongside food.

### 2.4 A 50,000-person city as a throughput test

Using explicitly illustrative assumptions:

| Supply | Assumption | Required daily delivery |
| --- | --- | --- |
| Cereals | 250 kg/person/year | **34.2 tonnes/day** |
| Firewood | 1 kg/person/day | **50 tonnes/day** |
| Domestic water | 20 L/person/day | **1,000 m³/day** |

The cereal flow alone represents roughly **1,700 twenty-kilogram loads per day**. Carrying all domestic water in twenty-litre loads would require **50,000 loaded trips per day**, before return travel.

These calculations explain why production is only part of the design problem. Distribution labor, delivery infrastructure, and household access can become binding constraints even when regional resources are sufficient.

---

## 3. Historical systems and regional variation

### 3.1 Rome: public entitlement supported by taxation and commerce

Temin summarizes estimates of Rome’s annual grain requirement at approximately **150,000–300,000 tonnes**. The range reflects substantial uncertainty in population and consumption assumptions; it should not be treated as a precise count of public distributions. [AEA Publications](https://pubs.aeaweb.org/doi/10.1257/089533006776526148)

The annona combined state intervention, grain collected as tax, purchases, distributions, and incentives to shipping. Private traders also supplied a substantial share of grain and other foods. Some elite households could draw produce from estates; other residents depended more heavily on markets. [Cambridge University Press](https://www.cambridge.org/core/books/abs/metropolis-and-hinterland/model-of-agricultural-change/CAEB744E31AE143443AF67102DA7CFB0)

**Strength:** a government could mobilize resources across a large territory and protect politically important consumers.

**Failure mode:** entitlement depended on procurement, shipping, infrastructure, administration, and political control. A promised ration was not the same as grain physically available at a distribution point.

**TCE analogue:** a capital can sustain demand beyond its immediate surroundings through coercive or fiscal claims. The costs do not disappear; they fall on taxpayers, producers, transport workers, and the treasury.

### 3.2 China: local relief stocks and long-distance capital supply were different systems

Shiue estimates that civilian granaries in the **1750s** held approximately **1.5 billion litres of husked-rice equivalent**, or **7.5 litres per person** under the study’s population assumption. At its consumption benchmark, that was roughly **3% of annual requirements**—about **eleven days if spread equally**, not eleven days of guaranteed relief for everyone. Stocks were geographically and institutionally uneven. [Spot Colorado](https://spot.colorado.edu/~shiue/Shiue%20proofs1.pdf)

Local granaries offered price stabilization and targeted relief, but needed replenishment and credible supervision. Shiue also investigates whether access to central relief weakened local incentives to maintain reserves; this is a specific historical finding and interpretation, not a universal rule that central assistance necessarily causes neglect. [Spot Colorado](https://spot.colorado.edu/~shiue/Shiue%20proofs1.pdf)

Capital provisioning through tribute-grain transport was a separate logistical undertaking. In **1824–1826**, embankment failures, flooding, and disruption of water management brought the grain fleet toward Beijing to a standstill. [Cambridge University Press](https://www.cambridge.org/core/journals/modern-asian-studies/article/abs/controlling-from-afar-open-communications-and-the-taokuang-emperors-control-of-grand-canalgrain-transport-management-18241826/B78CFFC51214F289396920FA4E5A6128)

**TCE analogue:** implement civilian relief, military supply, and capital tribute as distinct institutions with potentially competing priorities. Research on Ming–Qing water control shows that protecting grain transport could impose severe costs on other regions. [Cambridge University Press](https://www.cambridge.org/core/journals/modern-asian-studies/article/abs/sacrificing-local-interests-water-control-policies-of-the-ming-and-qing-governments-and-the-local-economy-of-huaibei-14951949/2521B81C25093DCF994E1F711F7D174B)

### 3.3 London: market networks, specialized producers, and a fuel constraint

The *Feeding the City* research reconstructs a medieval London of perhaps **80,000 or more inhabitants**, with routine food and fuel connections extending as far as \*\*sixty miles—about 97 km—\*\*where waterways or intermediate market centers made transport viable. Its provisioning region was therefore not a compact circle of adjacent farms. [IHR Web Archives](https://archives.history.ac.uk/cmh/arpt97con.html)

Wood supply stimulated specialized production and distribution. Galloway, Keene, and Murphy’s study nevertheless places substantial pressure on the regional wood-fuel system around **1300**, with increased coal use among the responses. This was a fuel-specific constraint, not evidence that every agricultural supply had reached the same limit. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1468-0289.1996.tb00577.x)

**TCE analogue:** different commodities should have different supply regions and bottlenecks. A city may still be adequately fed while cooking, heating, and fuel-intensive industries become expensive.

### 3.4 Variation across subsistence and technological regimes

These are configurations that can overlap, not mandatory stages.

| Configuration | Historical variation | Implication for TCE |
| --- | --- | --- |
| **Foragers and predomestication collectors** | Purpose-built storage at Dhra’ in the Jordan Valley dates to approximately **11,300–11,175 years ago**, before fully domesticated crops. Storage is not exclusively a state or farming invention. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2700141/) | Allow durable stores, seasonal aggregation, and resource-rich settlements before an “agriculture unlock.” Do not give all foragers one territory-per-person coefficient. |
| **Early farming communities** | The Dhra’ evidence also documents changing storage arrangements during the transition toward farming. Storage location and control need not remain fixed as households and settlements change. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2700141/) | Make household, shared, and institutional storage alternative arrangements. Seed preservation and harvest timing become central constraints. |
| **Pre-industrial tropical urbanism** | Angkor’s archaeological landscape intermingled settlement, cultivation, and water infrastructure. Models of its hydraulic network show vulnerability to cascading damage under climatic variation. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1964867/) | Do not assume a dense, non-farming city surrounded by a sharply separate countryside. Model mixed urban–agricultural landscapes and infrastructure interdependence. |
| **Maya water provisioning** | Tikal’s Corriental reservoir had an estimated capacity around **58,000 m³**, with archaeological evidence interpreted as water filtration. Other reservoirs show evidence of contamination. [Nature](https://www.nature.com/articles/s41598-020-75023-7) | Seasonal storage and water quality can matter more than distance to a perennial river. Different neighborhoods can have different water security. |
| **African regional crop systems** | Late medieval and early modern storage evidence from Dongola includes sorghum, wheat, barley, pulses, and other crops, with household-scale food storage and processing. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12204933/) | Avoid a universal wheat–bread economy or an assumption that all provisioning runs through monumental public granaries. |
| **Industrializing export economies** | Travieso’s comparison of Uruguay and New Zealand connects agricultural exports with different coal endowments and imports; Uruguay obtained **92% of its coal imports from Britain in 1890–1911**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-global-history/article/united-by-grass-separated-by-coal-uruguay-and-new-zealand-during-the-first-globalization/3D07EDE06AA9EED0E855C50B9FBA2B8C) | Industrialization can enlarge food-trading regions while introducing fuel-import dependencies. Do not assume every region follows Britain’s domestic-coal trajectory. |
| **Modern mixed systems** | Contemporary charcoal research in Zambia and Mozambique documents urban demand, rural livelihoods, tenure effects, and successive woodland-degradation fronts. Modern cities do not necessarily cease using biomass. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0305750X21002151) | Enable simultaneous biomass, fossil-fuel, and electrical systems. Technology availability does not imply universal adoption or equal access. |

Industrialization should not automatically erase forest demand either. Iriarte-Goñi and Ayuda find that British wood consumption increased during industrial growth even after coal displaced much wood fuel, because wood remained important as a material and imports expanded. [Historia Ambiental](https://historia-ambiental.com/FILE/bibliografia/A.16._Iriarte_Goni_I._and_Ayuda_M.I.pdf)

---

## 4. Stylized facts and validation targets

A convincing simulation should reproduce the following patterns through its rules, rather than through scripted historical events.

| Pattern | Evidence or reasoning | Validation experiment |
| --- | --- | --- |
| **Supply regions follow transport costs, not circles.** | London’s connections reached distant producers through waterways and intermediate markets. [IHR Web Archives](https://archives.history.ac.uk/cmh/arpt97con.html) | Give two equally productive regions different transport access. The better-connected region should supply more, even when geographically farther away. |
| **Modest harvest losses can cause severe urban shortages.** | The arithmetic of residual surplus makes urban supply more volatile than gross harvest when rural requirements are relatively inflexible. | Reduce harvest by 10–30%; measure exports, producer consumption, stocks, and compulsory transfers separately. |
| **Food and fuel constraints diverge.** | Medieval London’s wood-fuel pressures did not simply mirror its grain supply. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1468-0289.1996.tb00577.x) | Increase city population or industrial fuel demand. Food prices and fuel prices should not move in lockstep. |
| **Price stabilization may hide changes in quantity.** | Research on eighteenth-century northern Italian cities documents bread-price regulation operating partly through changes in loaf weight. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/cost-of-living-in-early-modern-cities-a-study-on-eighteenthcentury-northern-italy/90C3541A5939A38D57C8F60A12AA38C1) | Hold the posted loaf price fixed while flour costs rise. Track weight, quality, queues, baker viability, and illicit sales—not price alone. |
| **A large reserve is not necessarily accessible to those in need.** | Chinese reserve totals coexisted with geographic and administrative limits on deployment. [Spot Colorado](https://spot.colorado.edu/~shiue/Shiue%20proofs1.pdf) | Put ample grain in a distant or restricted warehouse. Hunger should persist until transport and allocation change. |
| **Urban famine can result from transport and political disruption, even in an industrial society.** | During the Dutch Hunger Winter, official rations fell to approximately **400–800 kcal/day from December 1944 to April 1945**, amid transport restrictions and severe winter conditions. These figures are official rations, not necessarily total individual intake. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2947913/) | Interrupt several delivery routes together and reduce fuel availability. Check the distribution of deprivation and the speed of recovery when flows resume. |
| **Water abundance and water safety are different variables.** | Tikal’s reservoirs provide evidence of contrasting water-management and contamination conditions. [Nature](https://www.nature.com/articles/s41598-020-75023-7) | Contaminate a full reservoir without reducing its volume. Households should seek alternatives or bear consequences rather than remain fully provisioned. |
| **Fuel exploitation can move outward without clearing every intervening forest.** | Mozambique research identifies successive charcoal-production waves and changing woodland structure. [ScienceDirect](https://www.sciencedirect.com/science/article/am/pii/S0143622818310464) | Open a road through woodland. Harvesting should respond to access, rights, tree suitability, and prior depletion rather than a simple expanding circular clearing. |

Two additional **model-derived** tests are especially valuable.

First, four suppliers using the same bridge or port are not four independent sources of resilience. Apply correlated disruptions to shared infrastructure and climatic regions.

Second, compare cash assistance with physical food distribution. Cash should improve access when purchasable supply can respond, but it should not conjure food behind a blockade. Conversely, physical stocks should not solve hunger when eligibility or distribution excludes vulnerable households.

---

## 5. Modeling recommendation for TCE

### 5.1 Represent people individually, but pool the right decisions

For 10,000–50,000 people, use individuals for needs, work, movement, and social claims, while households and organizations handle many inventory and purchasing decisions.

| Entity | Minimum state | Important behavior |
| --- | --- | --- |
| **Person** | Age, activity, health, occupation, location, household, access rights | Eats, works, travels, queues, gathers, carries, becomes distressed, migrates. |
| **Household** | Shared stocks, cash/debt, housing, cooking equipment, ration eligibility | Plans meals, buys or gathers supplies, shares fuel, selects water sources, manages buffers. |
| **Farm or resource parcel** | Soil, water, crop calendar, expected harvest, seed requirements, woodland age/biomass | Produces seasonally; responds to labor, management, weather, and competing uses. |
| **Firm or institution** | Owner, inventory, processing capacity, workers, funds, contracts, policy rules | Mills, bakes, transports, stores, sells, taxes, purchases, or distributes. |
| **Shipment** | Commodity, quantity, origin, destination, owner, carrier, route, arrival estimate | Reserves capacity, travels, waits, suffers loss, and changes location only through movement. |
| **Water facility** | Source yield, storage, quality, delivery capacity, elevation, maintenance, access rules | Supplies water, queues users, fails, degrades, or contaminates downstream deliveries. |

**A critical population-accounting constraint:** the rural population must exist somewhere. A world containing 50,000 individual people cannot place all 50,000 in a consuming city and also receive unlimited production from invisible farmers. Either allocate substantial population to the hinterland or introduce explicit, finite aggregate external populations as a declared abstraction.

### 5.2 Use a common physical system beneath different institutions

The same farm, warehouse, boat, and bakery should work under private commerce, temple redistribution, compulsory tribute, cooperative ownership, or municipal rationing.

The institution changes **who decides, who pays, who owns, and who receives**. It does not remove hauling time, spoilage, labor requirements, or water scarcity.

A public granary should therefore use the same stock-batch system as a merchant warehouse. Its additional components are eligibility, procurement authority, accounting, and release policy—not a special “food security bonus.”

### 5.3 Use mixed time scales

A practical schedule is:

| Frequency | Suitable processes |
| --- | --- |
| Intraday or event-driven | Travel, loading, queues, shop service, distribution, water collection. |
| Daily | Consumption, household acquisition, processing, labor allocation, routine stock deterioration. |
| Weekly or decision-triggered | Procurement contracts, merchant route choice, reserve reviews, price-policy responses. |
| Seasonal | Planting, harvest, navigation changes, heating demand, runoff regimes. |
| Annual or cohort-based | Woodland growth, long-lived infrastructure investment, major land-use decisions. |

Do not run a world-wide pairwise search between every consumer and every producer. Households can choose among nearby outlets; merchants connect outlets to a limited set of suppliers; institutions maintain designated procurement networks.

Cache routes and recalculate when relevant conditions change. Aggregate cargo into shipment batches rather than simulating individual grain particles. Visible bags, carts, and port activity can represent the underlying transfers without requiring a separate economic agent for every object.

### 5.4 Simplify geometry—not scarcity

Safe early simplifications include woodland age cohorts instead of individual trees, field-level crop production instead of plants, directed water-flow capacities instead of full fluid simulation, and a small set of meaningful food and fuel categories.

Preserve the following distinctions from the beginning: stock location versus ownership; purchased versus delivered goods; gross harvest versus surplus; source water versus household water; fuel energy versus useful heat; seasonal inventory versus emergency reserve; city-wide availability versus household access.

Technology should change specific constraints. Boats alter payloads and routes; improved stores alter losses; coppicing changes regeneration; mills alter labor and processing; gravity conduits change water delivery; refrigeration changes perishability but introduces another energy dependency. None requires a fixed historical era.

### 5.5 Existing models and games worth borrowing from

| Reference | Reusable pattern | Important limitation |
| --- | --- | --- |
| **ORBIS — Scheidel and Meeks** | Multimodal transport represented through seasonal cost and travel-time networks. | Not an endogenous provisioning economy; its historical assumptions should not become universal transport constants. [Stanford History Department](https://history.stanford.edu/publications/orbis-stanford-geospatial-network-model-roman-world) |
| **MayaSim — Heckbert, 2013** | Coupled settlement, agriculture, soil, forest, climate, and trade dynamics. | A coarse settlement-level model and proof of concept, not validation of daily individual behavior or a definitive explanation of Maya history. [JASSS](https://www.jasss.org/16/4/11.html) |
| **Banished** | Legible individual work, inventories, food production, trade, and seasonal survival pressures. | Useful design patterns, not historical calibration; player-directed organization differs from TCE’s autonomous institutions. [Shining Rock Software](https://shiningrocksoftware.com/game/) |
| **Workers & Resources: Soviet Republic** | Explicit water supply, quality, delivery infrastructure, and truck-versus-network choices. | Its planned, player-managed system is not an institutional model for every society. [Soviet Republic](https://www.sovietrepublic.net/post/report-for-the-community-46) |

### 5.6 Instrument the causes of failure

The provisioning dashboard should report more than total food.

Show city-gate arrivals by origin and route; edible stocks by owner and location; days of accessible food by household income group; caloric shortfalls; time spent collecting water and fuel; delivered useful heat; transport utilization; reserve discrepancies; and woodland removals relative to growth.

For a failed delivery, preserve an explainable chain such as:

> The mill lacks grain because the merchant’s shipment is delayed; the shipment is delayed because river capacity fell; the merchant did not use the road because available carts were already contracted elsewhere.

That makes emergence legible without inventing a narrative event after the fact.

---

## 6. Sources, datasets, and limits of the evidence

### Recommended calibration library

| Source | Best use | Main caution |
| --- | --- | --- |
| **Campbell, Galloway, Keene, and Murphy, *A Medieval Capital and Its Grain Supply*; IHR’s *Feeding the City* projects** | Crop specialization, metropolitan demand, transport geography, and urban–rural connections. The second project’s data are identified as UK Data Service Study **3318**. [IHR Web Archives](https://archives.history.ac.uk/ihrcms/cmh/projects/research/feeding-the-city.html) | Surviving demesne records are not a representative census of every peasant household. |
| **Campbell, *Three Centuries of English Crop Yields, 1211–1491*** | Annual crop and seed–harvest variation; testing agricultural volatility. [Queen's University Belfast](https://pure.qub.ac.uk/en/datasets/three-centuries-of-english-crop-yields-1211-1491/) | Estate selection, units, missing years, and regional coverage require care. |
| **Morley, *Metropolis and Hinterland*; Temin, “The Economy of the Early Roman Empire”** | Roman city demand, market connections, and the relationship between state intervention and commerce. [Cambridge University Press](https://www.cambridge.org/core/books/abs/metropolis-and-hinterland/model-of-agricultural-change/CAEB744E31AE143443AF67102DA7CFB0) | Population, grain requirements, and the extent of market integration remain debated. |
| **Will and Wong, *Nourish the People*; Shiue, “Local Granaries and Central Government Disaster Relief”** | Reserve institutions, replenishment, administrative incentives, and relief organization. [JSTOR](https://www.jstor.org/stable/10.3998/mpub.19044) | Recorded stocks, effective stocks, and household access are different quantities. |
| **Galloway, Keene, and Murphy, “Fuelling the City”** | Medieval fuel supply regions, specialized production, and substitution pressures. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1468-0289.1996.tb00577.x) | Regional reconstruction, not a universal per-capita fuel schedule. |
| **FAO, *Simple Technologies for Charcoal Making*** | Conversion processes, woodland requirements, and harvesting/transport relationships. [FAOHome](https://www.fao.org/4/x5328e/x5328e03.htm) | Engineering examples must be matched to species, moisture, management, and equipment. |
| **APHLIS** | Crop-specific, stage-specific post-harvest loss modeling. [FAOHome](https://www.fao.org/platform-food-loss-waste/food-loss/food-loss-measurement/ru) | Outputs are modeled estimates; modern observations are analogues, not ancient measurements. |
| **Howard and colleagues, WHO, *Domestic Water Quantity, Service Level and Health*, 2020** | Connecting household water quantities to access, reliability, and service level. [UEA Digital Repository](https://ueaeprints.uea.ac.uk/id/eprint/79561/1/Published_Version.pdf) | Not a dataset of historical urban consumption; quality and quantity must remain separate. |
| **Reba, Reitsma, and Seto, “Spatializing 6,000 Years of Global Urbanization,” 2016** | Broad city-size and location comparisons. [Nature](https://www.nature.com/articles/sdata201634) | Digitized historical estimates have uneven coverage and inconsistent urban boundaries. |

FAOSTAT, GAEZ, and AQUASTAT are useful supplementary sources for modern agricultural quantities, ecological suitability, and water-resource information, respectively. They are best treated as constraints and analogues, not as direct estimates of ancient realized productivity. [FAOHome](https://www.fao.org/faostat/en/)

### Claims that should remain explicitly uncertain

**Historical units and denominators.** A *modius*, a *shi*, a litre of unhusked rice, and a kilogram of edible grain are not interchangeable. Population estimates may refer to a built-up center, an administrative district, or a much larger urban landscape.

**Reported reserves versus real relief.** Storage capacity, book inventory, physical stock, edible stock, and deliverable stock must not be treated as synonyms.

**Fuel scarcity versus inevitable deforestation.** Growing demand can encourage woodland management and substitution as well as depletion. Tenure, agricultural expansion, regeneration, and transport access change the outcome.

**Availability versus entitlement.** Physical shortage and loss of access can occur together. Neither “all famine is lack of food” nor “food availability never matters” is an adequate rule.

**Collapse explanations.** Angkor and Tikal provide evidence for climatic and water-management vulnerabilities, not universal proof that a particular reservoir failure necessarily causes urban collapse. Political, ecological, and social processes remain intertwined. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2872380/)

**Bottom line for TCE:** make urban growth depend on maintaining several real supply chains at once. The decisive question is not “How many people fit inside this food radius?” It is **“Whose next month of food, fuel, and safe water can actually be delivered and accessed—and what happens to next year’s productive base while securing it?”**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9289e-ce28-83e9-b324-ae6f9d5a43e3)
