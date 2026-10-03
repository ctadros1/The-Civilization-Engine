# Natural resources and technological access in The Civilization Engine

## Executive recommendation

**Generate geological bodies, then let exposure, discovery, engineering, processing knowledge, and delivered cost determine whether they become resources.** Do not place “copper mines” or unlock everything below a particular depth with an era upgrade.

Three distinctions are especially important:

* **Rock is not necessarily ore.** Metal concentration, mineral form, and the ability to separate it from surrounding material determine its usefulness.
* **A deposit is not a mine’s accessible inventory.** Water, unstable ground, access tunnels, pillars, and equipment can leave much of a deposit unreachable.
* **Extraction is not production of a usable commodity.** Removed rock may require sorting, crushing, washing, roasting, smelting, refining, or stone dressing. Modern copper processing, for example, distinguishes mined ore from much richer concentrates and subsequent metal products. [USGS Publications](https://pubs.usgs.gov/bul/1693/report.pdf)

For TCE, the most useful abstraction is:

**Geological inventory → discovered occurrence → developed working → extracted material → processed commodity.**

Technology and institutions should change the costs and conversion efficiencies between these states—not create additional geological material.

---

## 1. Mechanisms: where resources occur and why

### 1.1 Geological placement rules

The following are major deposit families, not an exhaustive mineralogical catalogue. Each resource should have several possible geological routes rather than one universal host.

| Material | Geological setting and deposit shape | Procedural placement and early-access implications |
| --- | --- | --- |
| **Copper** | Porphyry systems associated with intrusive magmatism; sediment-hosted mineralization in suitable basins; veins and replacement bodies. Weathering can turn near-surface sulfides into conspicuous green or blue copper minerals. | Generate mineralized districts around appropriate intrusions or basin structures, then localized bodies within them. Give some exposed bodies oxidized, selectively workable patches; do not make all copper an underground sulfide resource. [USGS](https://www.usgs.gov/data/a-global-database-porphyry-copper-deposits-and-prospects) |
| **Native copper** | An exceptional but important alternative to copper ores. The Lake Superior region contains native copper associated with ancient volcanic and sedimentary rocks. | Include a rare native-metal deposit family. It permits extraction and working without first mastering reduction of copper ores; Indigenous mining at Isle Royale demonstrates that agricultural or smelting societies are not prerequisites. [National Park Service](https://www.nps.gov/articles/nps-geodiversity-atlas-isle-royale-national-park-michigan.htm) |
| **Tin** | Mainly cassiterite associated with specialized granitic systems, veins, and replacement deposits. Weathering produces residual, slope, and river concentrations; old alluvial deposits may subsequently be buried. | Only some granites should be tin-bearing. Generate downstream cassiterite placers from eroding sources, including buried former channels. Dense mineral grains may be washable even where primary hard-rock extraction is unattractive. [Geoscience Australia](https://www.ga.gov.au/education/minerals-energy/australian-mineral-facts/tin) |
| **Iron** | Several distinct families: ancient banded iron formations and their enriched products; detrital and channel deposits; weathered iron-rich material; near-surface bog ores; iron sands. | Do not restrict iron to mountains. Generate large iron-bearing formations separately from small accessible nodules, sands, or wetland accumulations. Iron abundance and suitability for a particular furnace are different attributes. [Geoscience Australia](https://www.ga.gov.au/education/minerals-energy/australian-mineral-facts/iron) |
| **Lead and silver** | Galena-bearing veins and other polymetallic bodies; carbonate-hosted lead–zinc deposits formed by basinal fluids; additional silver-bearing deposit families. Silver may accompany lead, but the association varies. | Use a lead grade and a separate silver grade. Carbonate-hosted lead–zinc mineralization need not lie beside an intrusion: Mississippi Valley-type deposits are specifically associated with sedimentary basins rather than nearby magmatism. [USGS](https://www.usgs.gov/publications/a-deposit-model-mississippi-valley-type-lead-zinc-ores) |
| **Gold** | Hydrothermal mineralization, including veins and disseminated deposits; erosion can concentrate liberated gold in alluvial placers. Gold may be coarse and gravity-recoverable or finely enclosed in other minerals. | Place placer gold only where the catchment supplies it, then concentrate it selectively in depositional traps. Store grain size and liberation, not just gold concentration: identical grades can require very different processing. [Geoscience Australia](https://www.ga.gov.au/education/minerals-energy/australian-mineral-facts/gold) |
| **Coal** | Buried accumulations of ancient plant material, subsequently altered by burial and heating. Seams belong to sedimentary successions; rank reflects geological history, not the modern biome. | Generate coal-bearing basins with laterally correlated seams, interruptions, folds, and faults. Outcrop coal can be accessible without deep-mining machinery. Keep rank, ash, sulfur, and suitability for metallurgical use separate from mere combustibility. [U.S. Energy Information Administration](https://www.eia.gov/energyexplained/coal/) |
| **Salt** | Halite-bearing evaporite sequences; saline groundwater produced by dissolution; surface brines and seawater. Ancient evaporites may now lie beneath humid landscapes. | Generate salt from past depositional conditions, not simply present deserts. Allow rock mining, brine collection, wells, and evaporation as distinct operations. In wet settings, dissolution may remove exposed halite while leaving underground salt or brine. [Nora](https://nora.nerc.ac.uk/id/eprint/534431/1/mpf_salt.pdf) |
| **Flint/chert** | Nodules or beds in suitable sedimentary units; weathering and transport can redistribute pieces into surface deposits. Quality varies within and between horizons. | Put good knapping material in specific horizons, not every limestone tile. At Grime’s Graves, miners bypassed poorer material to obtain high-quality flint underground: accessibility includes quality, not only distance from the surface. [English Heritage](https://www.english-heritage.org.uk/visit/places/grimes-graves-prehistoric-flint-mine/history/) |
| **Clay** | Unconsolidated clay-rich deposits and clay-bearing mudstones or shales. Ceramic suitability depends on mineral composition, particle mixture, impurities, and firing behavior. | Separate “clayey ground” from pottery clay, brick clay, and refractory feedstock. Allow washing, temper addition, and blending to improve otherwise unsuitable material. A floodplain need not provide every ceramic grade. [Nora](https://nora.nerc.ac.uk/id/eprint/532490/1/Brick%20Clay%20Mineral%20Planning%20Factsheet.pdf) |
| **Limestone** | Carbonate sedimentary formations, subsequently modified by burial, deformation, dissolution, and weathering. Limestone, chalk, and dolomitic materials differ in properties and chemistry. | Represent broad formations rather than isolated resource nodes. Store block quality separately from chemical purity: building stone, lime feedstock, metallurgical flux, and high-purity industrial limestone are different products. [Nora](https://nora.nerc.ac.uk/id/eprint/534436/1/mpf_limestone.pdf) |
| **Granite** | Intrusive bodies exposed by erosion. Jointing and weathering divide the rock into blocks and boulders of varying dimensions. | Generate granite as a substantial rock body, then derive quarry quality from fractures and weathering. Aswan quarrying exploited surface boulders as well as bedrock; stone pounders enabled extraction before iron quarrying tools. [Academia](https://www.academia.edu/5407078/Granite_quarry_survey_in_the_Aswan_region_Egypt_shedding_new_light_on_ancient_quarrying) |

**Other materials worth distinguishing early:** gypsum supplies plaster; silica-rich, low-impurity sands support glassmaking; refractory clays support high-temperature industries. These should not be interchangeable with generic stone, sand, and mud. Gypsum commonly occurs in evaporite beds, while British fireclays illustrate a useful geological association with fossil soils beneath coal seams. [Nora](https://nora.nerc.ac.uk/id/eprint/534434/1/mpf_gypsum.pdf)

### 1.2 Generate geology before the final surface

I recommend this world-generation sequence:

**Geological provinces → rock bodies and strata → faults and mineralization → erosion and weathering → transported deposits → present exposure.**

First assign geological histories: sedimentary basin, old crystalline terrain, volcanic province, intrusive belt, or combinations. Construct correlated rock units and cross-cutting structures. Then generate mineralization conditional on the appropriate host, structures, and history.

Next, let terrain erosion expose or bury these bodies. A valley can reveal a coal seam or vein; it should not create one because the terrain is steep. Likewise, an ancient evaporite basin can survive beneath a modern wet climate.

Finally, derive secondary deposits from actual sources. Route eroded gold, cassiterite, and rock fragments through drainage networks. Allow floodplain burial and abandoned channels. This creates useful associations without the implausible rule that every river contains precious metals. These recommendations follow the distinction between primary deposit systems and weathering-derived concentrations in geological deposit models. [USGS Publications](https://pubs.usgs.gov/bul/1693/report.pdf)

Use **correlated grade variation inside a body**, not independent random ore tiles. A vein should narrow, widen, split, or become poorer along its course.

### 1.3 Treat exposure, engineering access, and processability separately

A visible mineral occurrence may still be unusable. Conversely, a buried deposit may be workable through an adit entering a hillside.

For each potential operation, evaluate:

| Constraint | Suggested simulation rule |
| --- | --- |
| **Exposure and discovery** | Observation reveals indications, not a complete deposit outline. Prospecting reduces uncertainty through sampled outcrops, pits, trenches, and later boreholes. |
| **Excavation** | Rock strength, fractures, working space, and method determine removal per work cycle. |
| **Access development** | Shafts, tunnels, roads, and benches consume labor and materials before generating saleable output. |
| **Water** | A working closes when inflow exceeds drainage and pumping capacity for long enough to flood it. |
| **Ground stability** | Support, pillars, and limited excavation spans reduce collapse risk but leave material behind. |
| **Processing** | A recipe must be compatible with mineral form and grain liberation; grade alone is insufficient. |
| **Delivered cost** | Continue production only while expected output value covers extraction, processing, transport, upkeep, and institutional charges—or while an institution deliberately subsidizes it. |

These are proposed implementation rules. Historical observations supporting them include water-management problems in Indian coal workings, substantial pillar losses, and the different processing requirements of free versus enclosed gold. [Internet Archive](https://archive.org/stream/memoirsofgeologi4119geol/memoirsofgeologi4119geol_djvu.txt)

---

## 2. Parameters: measured anchors and explicitly provisional defaults

**Units:** `t` means metric tonnes unless otherwise stated; `wt%` means mass percentage. Confidence refers to the stated observation—not its transferability to every society.

**H:** strong measurement or well-established material relationship.  
**M:** reconstructed, approximate, or strongly site-specific.  
**L:** provisional TCE calibration assumption.

### 2.1 Empirical and historical anchors

| Parameter or observation | Value and scope | What to calibrate | Confidence and source |
| --- | --- | --- | --- |
| **Pre-metal flint shaft depth** | Up to **13 m** at Grime’s Graves; **433 visible mines and pits** | Stone-age tools must not imply surface-only extraction | H, English Heritage archaeological account. [English Heritage](https://www.english-heritage.org.uk/visit/places/grimes-graves-prehistoric-flint-mine/history/) |
| **Fire-setting excavation** | Approximately **7–86 kg rock per experimental fire**, including subsequent removal, in **66 quartzite trials** | A hard-rock excavation cycle, not universal miner-day output | H for trials; L for generalization. Py & Ancel, 2006. [Academia](https://www.academia.edu/80485090/Archaeological_experiments_in_fire_setting_protocol_fuel_and_anthracological_approach) |
| **Fire-setting fuel requirement** | Trials give approximately **0.32–1.57 kg rock/kg wood burned**; inverse approximately **0.64–3.13 kg wood/kg rock** | Fuel gathering, drying, transport, and ventilation can dominate extraction | H for measured trials; highly method-specific. [Academia](https://www.academia.edu/80485090/Archaeological_experiments_in_fire_setting_protocol_fuel_and_anthracological_approach) |
| **Bronze Age copper output** | Great Orme reconstruction: **232–830 t copper metal total**; approximately **1–4 t/year** during its major production phase, c. **1600–1400 BCE** | Settlement-scale metal output and a rich-zone production peak | M: reconstructed volume, grade, recovery, and chronology. Williams & Le Carlier de Veslud, 2019. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/boom-and-bust-in-bronze-age-britain-major-copper-production-from-the-great-orme-mine-and-european-trade-c-16001400-bc/356E30145B1F6597D8AAA0DDBE69BD51) |
| **Traditional Japanese ironmaking batch** | Approximately **12 t charcoal**, **three days** of operation, and a **3 t crude iron/steel lump**; premium steel is only part of it | Batch production and fuel burden, not three tonnes of finished tools or uniform steel | M: official heritage description. [Ministry of Transport and Tourism](https://www.mlit.go.jp/tagengo-db/common/001565194.pdf) |
| **Deep brine access** | Shenhai well reached approximately **1,001 m in 1835** | Deep fluid extraction can precede comparable solid-ore access | H for the reported well depth; an exceptional **borehole**, not a human-access shaft. [Zigong Geopark](https://en.ziggeopark.cn/sheet/12805.html) |
| **Historical coal labor productivity** | Indian compilation reports **103.7 historical tons/person-year in 1910**, ranging **27.6–154.4** between coalfields | Annual mine-wide output and large regional variation | M. Original “tons” retained because the unit convention is not explicit in the passage. [Internet Archive](https://archive.org/stream/memoirsofgeologi4119geol/memoirsofgeologi4119geol_djvu.txt) |
| **Modern coal productivity** | U.S. 2024: **3.24 short tons/employee-hour underground** and **8.94 at surface mines**; approximately **2.94 and 8.11 metric t/hour** | Modern mine-wide throughput, including supporting employees | H. EIA Annual Coal Report, Table 23. [U.S. Energy Information Administration](https://www.eia.gov/coal/annual/pdf/table23.pdf) |
| **Coal left in pillars** | Historical Indian account: pillars held **25–65%** of available coal in common workings | Extraction fraction differs from deposit inventory | M, site/method-dependent; not a universal safety prescription. [Internet Archive](https://archive.org/stream/memoirsofgeologi4119geol/memoirsofgeologi4119geol_djvu.txt) |
| **Rock-salt extraction fraction** | Winsford layout described by BGS allowed about **75% extraction** | Room-and-pillar geometry leaves permanent inventory behind | H for that layout, not all salt mines. [Nora](https://nora.nerc.ac.uk/id/eprint/534431/1/mpf_salt.pdf) |
| **Concentrated salt brine** | Approximately **26 wt% NaCl** in saturated brine described by BGS | Evaporation burden per unit salt | H within the stated conditions. [Nora](https://nora.nerc.ac.uk/id/eprint/534431/1/mpf_salt.pdf) |
| **Iron-ore grade examples** | Australian detrital deposits: **40–55 wt% Fe**; channel deposits: **57–59%**; enriched major ores commonly approximately **56% to above 60%** | Distinguish ore families and usable feed grades | H for these modern geological examples; not ancient cutoffs. [Geoscience Australia](https://www.ga.gov.au/education/minerals-energy/australian-mineral-facts/iron) |
| **Brick-clay composition** | Approximately **20–80% clay minerals** in the feedstocks described | Ceramic suitability is not “100% clay” | H for the cited material range. BGS, 2022. [Nora](https://nora.nerc.ac.uk/id/eprint/532490/1/Brick%20Clay%20Mineral%20Planning%20Factsheet.pdf) |
| **Industrial limestone purity** | High-purity limestone generally **>97% CaCO₃** | A distinct high-purity product class—not the minimum for all historic lime | H. BGS. [Nora](https://nora.nerc.ac.uk/id/eprint/534436/1/mpf_limestone.pdf) |
| **Gypsum-to-plaster heating** | Approximately **150–165°C** for the process described | A useful low-temperature mineral-processing branch | H for the stated process. BGS. [Nora](https://nora.nerc.ac.uk/id/eprint/534434/1/mpf_gypsum.pdf) |

**Do not combine these into a single “mining efficiency by era” curve.** A fire-setting experiment measures a face operation; an annual employment statistic includes interruptions and supporting workers; a furnace batch measures an entirely different stage. The original fire-setting researchers also caution that their experimental output may have been poorer than that of experienced historical miners. [Academia](https://www.academia.edu/80485090/Archaeological_experiments_in_fire_setting_protocol_fuel_and_anthracological_approach)

### 2.2 Provisional TCE initialization ranges

The following are **author-proposed test ranges, not published deposit percentiles, historical averages, or economic cutoffs**. They provide contrasting scenarios while the empirical deposit-family distributions are being integrated.

| Test parameter | Initial range | Intended use | Basis/confidence |
| --- | --- | --- | --- |
| Selected copper-rich patch | **2–10 wt% Cu** | Small, selectively worked feed zones—not the average grade of a whole district | TCE prior; L |
| Bulk disseminated copper body | **0.2–1.0 wt% Cu** | Test the need for large throughput and effective concentration | TCE prior; L |
| Hard-rock cassiterite-bearing ore | **0.1–2 wt% Sn** | Test crushing, separation, and transport sensitivity | TCE prior; L |
| Gold-bearing hard rock | **1–20 g Au/t** | Vary liberation independently from grade | TCE prior; L |
| Gold-bearing placer gravel | **0.1–2 g Au/m³** | Separate gravel handling from recovered gold | TCE prior; L |
| Small vein geometry | **0.1–2 m width; 20–200 m strike length; 10–100 m vertical extent** | Test irregular, finite workings; not all veins | TCE prior; L |
| Initial overall copper recovery | **0.3–0.8 of contained copper** | Broad sensitivity tests across ore/recipe combinations | TCE prior; L |
| Saleable dimension-stone fraction | **0.1–0.6 of excavated mass** | Test joints, defects, block dimensions, and dressing losses | TCE prior; L |

Sample these jointly and by scenario. Do not give every mine an independent uniform draw from every range. A narrow rich shoot within a much larger poor body is more useful than a world of uniformly medium-grade deposits.

For extraction labor, I would initially calibrate **work cycles and crew composition**, rather than assert an unsupported universal tonnes-per-person-day rate. Hauling can then be calculated from carried mass, loading time, travel time, and available hours; excavation and processing remain separately constrained.

---

## 3. Variation across technological capabilities and world regions

### 3.1 Capabilities, not globally synchronized eras

| Descriptive stage | Workable methods and materials | What still limits access |
| --- | --- | --- |
| **Foraging societies** | Collection and quarrying of useful stone; hammerstone extraction; native-metal exploitation where such deposits occur. Lake Superior native copper is a key counterexample to “metal requires farming.” | Knowledge, local occurrence, excavation effort, and transport—not a universal ban on mining. [National Park Service](https://www.nps.gov/articles/nps-geodiversity-atlas-isle-royale-national-park-michigan.htm) |
| **Early farming and early specialist crafts** | Clay extraction and preparation; increasingly organized stone and mineral procurement; shafts and galleries using non-metal tools. Early metal production is possible where suitable recipes and ores are known. | Coordinating labor, supporting underground space, selecting material, and supplying fuel. Grime’s Graves demonstrates substantial underground access without iron tools. [English Heritage](https://www.english-heritage.org.uk/visit/places/grimes-graves-prehistoric-flint-mine/history/) |
| **Developed pre-industrial systems** | Specialist quarrying, fire-setting, winding and hauling systems, drainage works, ore dressing, and diverse furnace technologies. Some societies develop very deep fluid wells. | Water, ventilation, fuel supply, working-space congestion, difficult ore chemistry, and investment. Depth should emerge from these constraints. [Persee](https://www.persee.fr/doc/paleo_0153-9345_2000_num_26_2_4715) |
| **Industrial systems** | Increasing use of powered cutting, pumping, hoisting, transport, and larger processing installations; mechanized and manual operations coexist. | Capital, energy, deposit geometry, labor organization, and market conditions. Indian coal records even describe mechanized methods being abandoned when costs became unfavorable. [Internet Archive](https://archive.org/stream/memoirsofgeologi4119geol/memoirsofgeologi4119geol_djvu.txt) |
| **Modern systems** | Large-scale extraction, fine grinding, flotation and other specialized separation methods, extensive powered infrastructure. Previously unattractive material may become workable. | Low grades do not cease to matter: throughput, energy, recovery, waste handling, and transport remain decisive. Modern gold processing still distinguishes readily recoverable from refractory material. [Geoscience Australia](https://www.ga.gov.au/education/minerals-energy/australian-mineral-facts/copper) |

These categories should describe combinations of capabilities that agents possess. They should not be global state variables that automatically replace older methods.

### 3.2 Regional differences that should alter the model

**Southern Africa: no mandatory bronze-before-iron sequence.** Chirikure’s synthesis places established copper and iron metallurgy in the first millennium CE in southern Africa, with gold, tin, bronze, and brass becoming established later. It also emphasizes recycling and the adaptation of ceramics to metallurgical work. For TCE, recipes should spread and combine independently rather than follow a compulsory European sequence. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/arcm.12784)

**Japan: sediment processing can be the mining industry.** Iron sands were obtained from weathered material and separated with flowing water. The cited Japanese account describes winter washing because agriculture otherwise needed the water. This supports an industry whose calendar depends on irrigation rights, sediment handling, and seasonal labor—not just an ore face. [Ministry of Transport and Tourism](https://www.mlit.go.jp/tagengo-db/common/001565194.pdf)

**Sri Lanka: furnace placement can depend on wind.** Juleff’s excavations and replication work identified a first-millennium CE ironmaking system using monsoon winds on exposed hillsides. TCE should therefore permit environmental substitutes for particular machines: powered bellows need not be the only route to adequate furnace airflow. [Nature](https://www.nature.com/articles/379060a0)

**China: saline water and combustible gas create a different access problem.** Deep brine wells illustrate an industrial pathway based on drilling, lifting fluids, and evaporation, rather than excavating enough space for miners. The geological inventory and the engineering operation should consequently be different object types. [CSEG RECORDER Magazine](https://csegrecorder.com/articles/view/ancient-chinese-drilling)

**Egypt: hard stone was not inaccessible before iron.** Aswan quarrying used dolerite pounders and exploited suitable boulders and bedrock. Fractures and achievable block size were crucial. The appropriate early penalty is high labor and difficult logistics, not a locked granite resource. [Academia](https://www.academia.edu/5407078/Granite_quarry_survey_in_the_Aswan_region_Egypt_shedding_new_light_on_ancient_quarrying)

**Even modern regions differ greatly.** EIA’s 2024 surface-coal figures are **22.29 short tons/employee-hour in Wyoming** versus **2.01 in Appalachia**, roughly an elevenfold difference. Technology labels alone cannot explain production capacity. [U.S. Energy Information Administration](https://www.eia.gov/coal/annual/pdf/table23.pdf)

---

## 4. Depletion and the patterns a correct simulation should reproduce

### 4.1 There is no useful universal “years until exhausted”

A deposit’s physical inventory, a mine’s developed inventory, and its economically recoverable inventory change differently.

Great Orme is a particularly useful historical pattern: an approximately 800-year mining history contained a much shorter major production phase, followed by poorer workings after rich zones had been exploited. TCE should be able to produce a boom, contraction, and prolonged low-output tail—not merely constant production followed by disappearance. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/boom-and-bust-in-bronze-age-britain-major-copper-production-from-the-great-orme-mine-and-european-trade-c-16001400-bc/356E30145B1F6597D8AAA0DDBE69BD51)

Quarry depletion is often **depletion of suitable blocks**, not exhaustion of every tonne of rock. At Aswan, joint spacing and defects influenced the choice and abandonment of stone. Removing desirable surface boulders also changes the remaining extraction method. [Academia](https://www.academia.edu/5407078/Granite_quarry_survey_in_the_Aswan_region_Egypt_shedding_new_light_on_ancient_quarrying)

Closure must also be distinct from exhaustion. BGS records Winsford salt mining beginning in **1844**, closing between **1892 and 1928**, then resuming. Its 2006 account described substantial remaining reserves. A closure date is therefore not a geological expiry date. [Nora](https://nora.nerc.ac.uk/id/eprint/534431/1/mpf_salt.pdf)

### 4.2 Use explicit mass balances

For a metal-bearing feed:

\[
M\_{\text{metal}}
=
M\_{\text{ore}}\,
g\,
R\_{\text{separation}}\,
R\_{\text{smelting/refining}}
\]

Here, \(g\) is the metal mass fraction in the processed ore, and each recovery is a fraction of the metal entering that stage.

If barren waste is mixed into ore during extraction, define dilution as \(d=M\_{\text{waste}}/M\_{\text{ore}}\). Then:

\[
g\_{\text{run-of-mine}}=\frac{g\_{\text{ore}}}{1+d}
\]

These are bookkeeping identities, not historical estimates. They prevent a common simulation error: extracting the same amount of metal regardless of ore grade or incidental waste.

**Illustrative copper working.** A selected body measuring \(100\times1\times20\) m, at an assumed bulk density of \(2.7\) t/m³, contains **5,400 t of ore**. At **5% Cu** and **60% overall recovery**, it yields **162 t copper**. Extracting **100 t ore/year** gives **3 t copper/year** and **54 years** of operation—assuming those boundaries, grade, and costs remain fixed.

**Illustrative quarry.** One hectare worked through 10 m at an assumed \(2.5\) t/m³ contains **250,000 t gross rock**. At an assumed **30% saleable-block yield**, that is **75,000 t of dimension stone**. Producing **1,000 t of blocks/year** consumes that selected inventory in **75 years**, while generating substantial rubble that may have other uses.

These examples are proposed test cases, not reconstructions of particular historical sites.

### 4.3 Renewal must be a physical flux, not a reset timer

For TCE, ordinary bedrock ores and quarry stone should receive **no meaningful geological replenishment during a human historical run**. Newly accessible material comes from exploration, development, changing costs, or improved processing—not ore respawning.

Use separate rules for fluid and sediment resources:

* Seawater salt production draws on an external water-and-salt flux, but evaporation area and energy remain finite.
* Brine wells draw down a local fluid system and may dissolve a finite salt body.
* New placer or clay sediment requires actual upstream supply and deposition.
* Bog iron may accumulate through groundwater-related processes, but should not receive a universal “one generation” refill rule. Regional work confirms seasonal iron precipitation; it does not establish one globally valid replenishment rate. [Nora](https://nora.nerc.ac.uk/id/eprint/534431/1/mpf_salt.pdf)

### 4.4 Stylized facts and validation tests

| Pattern to reproduce | Suggested TCE validation |
| --- | --- |
| **Mining districts cluster geologically** | Compatible deposits concentrate in particular provinces; resources are not evenly spread between settlements. |
| **Accessible quality can matter more than total abundance** | A small rich, exposed occurrence can be chosen before a much larger poor or difficult body. |
| **Production has several bottlenecks** | Adding miners fails to increase output when hauling, water control, fuel, or processing is saturated. |
| **Not all inventory is extractable** | Pillars, boundaries, unsafe ground, and uneconomic material remain after closure. |
| **Decline is not always terminal** | A mine can close, be re-explored, and reopen under changed costs or capabilities. |
| **Older techniques remain locally competitive** | The simulation can retain manual or seasonal operations beside more mechanized ones. |
| **Material provenance affects architecture and trade** | Cheap local rubble and clay coexist with expensive imported specialty stone or mineral feedstocks. |

These are recommended tests derived from the deposit models and the historical contrasts above, rather than claims that every site must follow one lifecycle. [USGS Publications](https://pubs.usgs.gov/bul/1693/report.pdf)

---

## 5. Recommended implementation for individuals and institutions

### 5.1 Store geology compactly; refine only active workings

For a 10k–50k-person simulation, I recommend five distinct records:

| Record | Important state |
| --- | --- |
| **Geological unit** | Rock family, geometry, weathering, fractures, permeability, material properties |
| **Deposit body** | Shape, mineral forms, correlated grades, grain liberation, total inventory |
| **Worksite** | Entrances, developed faces, shafts/tunnels or benches, support, pumps, equipment, stockpiles |
| **Resource knowledge** | Observer or institution, samples, estimated extent and grade, confidence, known hazards |
| **Processing recipe** | Compatible feeds, capacities, fuel/water requirements, recoveries, products, waste |

Use coarse geological bodies and stratigraphic surfaces globally, with finer blocks or sampled volumes around active workings. A tunnel graph can represent movement, ventilation connections, drainage routes, and working faces without voxelizing the entire underground world.

Generate the underlying geology deterministically before exploration. Agent knowledge can change; the ore should not be rerolled when a prospector arrives.

### 5.2 Model labor as a chain of tasks

Represent prospectors, excavators, support workers, haulers, sorters, washers, furnace crews, and fuel suppliers as actual people performing tasks. However, resolve production through **worksite queues and shared crew progress**, not an independent daily yield attached to every miner.

A practical daily sequence is:

**Develop access → prepare face → excavate → move material → sort/concentrate → process → deliver.**

Each stage owns an inventory buffer and capacity. Face occupancy prevents twenty additional workers from multiplying output in a one-person space. Missing fuel or transport leaves stockpiles and idle labor rather than silently reducing an abstract efficiency score.

An individual’s contribution can be:

\[
\text{productive time}
=
\text{scheduled time}
-
\text{travel}
-
\text{setup}
-
\text{waiting}
-
\text{interruptions}
\]

A hauling task then derives capacity from load size and round-trip time. This makes roads, hoists, nearby processing, and work organization valuable without granting unexplained percentage bonuses.

### 5.3 Make water control an engineering problem

A minimal mine-water balance is:

\[
V\_{t+\Delta t}
=
\max\!\left(0,\,
V\_t+(Q\_{\text{in}}-Q\_{\text{pump}}-Q\_{\text{gravity}})\Delta t
\right)
\]

The power required to lift water is:

\[
P=\frac{\rho gQH}{\eta}
\]

As a calculated example, lifting **1 L/s through 100 m** requires about **981 W of hydraulic power**, or **1.96 kW input at 50% efficiency**. These are physical calculations, not historical pump specifications.

The important consequence is that **depth below the ground surface is not equivalent to pumping head**. A hillside working with a suitable gravity outlet may be easier to drain than a shallower mine in flat, saturated terrain.

Historical Indian records explicitly warned that removing outcrop barriers could increase later underground water inflows. That is a valuable path-dependent interaction: early cheap extraction can impose costs on later owners. [Internet Archive](https://archive.org/stream/memoirsofgeologi4119geol/memoirsofgeologi4119geol_djvu.txt)

### 5.4 Keep processing knowledge local and material-specific

Separate the discovery of a material from knowing how to use it. A community may recognize copper-colored stones but lack an effective recipe for a newly encountered mineral mixture. Likewise, recovering coarse gold does not imply the ability to treat finely enclosed gold. [Geoscience Australia](https://www.ga.gov.au/education/minerals-energy/australian-mineral-facts/gold)

For a manageable simulation, retain only the mineralogical distinctions that change behavior: native metal versus oxide/carbonate versus sulfide; free grains versus finely enclosed grains; clean versus contaminant-rich feed; ordinary versus refractory clay.

For salt, concentration alone creates an important economic distinction. With a hypothetical **3 wt% NaCl feed**, obtaining 1 kg salt requires removing approximately **32.3 kg water**; at **26 wt%**, approximately **2.85 kg**. Ignoring losses and other dissolved salts, the dilute feed requires roughly **eleven times** as much water removal. This calculated contrast makes brine quality, solar evaporation, and fuel supply matter. [Nora](https://nora.nerc.ac.uk/id/eprint/534431/1/mpf_salt.pdf)

### 5.5 Give institutions real coordination problems

Institutions should control and finance things individuals cannot conveniently provide alone: prospecting campaigns, access rights, roads, common drainage, shared hoists, processing facilities, and water allocation.

Mining can then generate conflicts over who pays for drainage, who benefits from another operator’s infrastructure, whether agricultural water takes priority, and who bears downstream waste costs. Do not make a particular labor regime or ownership system a technological prerequisite.

Ownership should attach separately to land, mineral rights, infrastructure, and produced goods where the society’s rules distinguish them. For simpler societies these rights can remain bundled.

### 5.6 Preserve legibility

Every unsuccessful operation should expose a concrete reason:

> “The accessible rich section is exhausted.”  
> “Water inflow exceeds drainage capacity.”  
> “The remaining stone cannot yield blocks of the requested size.”  
> “Transport costs exceed the expected value of this ore.”  
> “The furnace recipe performs poorly on this mineral mixture.”

Those explanations are more useful than “resource depleted” or “technology too low.” They also turn economic and engineering decisions into observable history.

### 5.7 Models and games worth borrowing from

**USGS descriptive and grade–tonnage models** are the best foundation for deposit families. Borrow their separation of geological setting, deposit type, size, and grade—not a single global ore distribution. [USGS Publications](https://pubs.usgs.gov/bul/1693/report.pdf)

**GemPy** demonstrates implicit three-dimensional geological modeling with strata, faults, folds, and intrusions. It is useful as an offline prototype or generation reference; TCE need not adopt its entire runtime stack. [GemPy](https://docs.gempy.org/)

**Vintage Story** provides a useful game precedent for host-rock-dependent ores and prospecting. Borrow the connection between geology and search behavior. Treat its geometric and progression simplifications as game rules, not geological evidence—especially where surface clues too directly identify what lies beneath them. [Vintage Story Wiki](https://wiki.vintagestory.at/Special%3AMyLanguage/Ore_Deposits)

---

## 6. Sources, datasets, and evidence limits

### High-value datasets and reference frameworks

| Source | Best use in TCE | Important limitation |
| --- | --- | --- |
| **Cox & Singer, 1986, *Mineral Deposit Models*, USGS Bulletin 1693** | Deposit-family vocabulary and geological relationships | An organizing framework, not a ready-made probability distribution for an arbitrary game map. [USGS Publications](https://pubs.usgs.gov/bul/1693/report.pdf) |
| **Singer, Berger & Moring, 2008, porphyry copper database and grade–tonnage models** | Joint size/grade calibration for a major copper family | Modern assessed deposits do not represent all small ancient occurrences. [USGS](https://www.usgs.gov/publications/porphyry-copper-deposits-world-database-and-grade-and-tonnage-models-2008) |
| **Magnin and colleagues, 2025, global porphyry database; DOI 10.5066/P14CCESQ** | Locations, ages, tectonic settings, classifications, and available grade/tonnage | Published in 2025, with information described as current through spring 2024. [USGS](https://www.usgs.gov/data/a-global-database-porphyry-copper-deposits-and-prospects) |
| **USGS sediment-hosted copper models and database** | A second major copper pathway, avoiding “all copper beside granite” | Some records aggregate deposits; reported spatial precision can be kilometre-scale. [USGS](https://www.usgs.gov/publications/sediment-hosted-copper-deposits-world-deposit-models-and-database) |
| **Hartmann & Moosdorf, 2012, GLiM; DOI 10.1594/PANGAEA.788537** | Large-scale lithological associations and province composition | The linked gridded product is **0.5°**, not a local terrain-generation map; distinguish it from the underlying polygon compilation. [PANGAEA DOI Name Resolver](https://doi.pangaea.de/10.1594/PANGAEA.788537) |
| **BGS Mineral Planning Factsheets** | Clay, stone, salt, gypsum, sand, purity, processing, and extraction distinctions | Mainly British industrial examples; historical editions must not be presented as current reserve statements. [British Geological Survey](https://www.bgs.ac.uk/mineralsuk/planning/mineral-planning-factsheets/) |

For historical calibration, the strongest anchors used here are **Williams & Le Carlier de Veslud’s Great Orme reconstruction**, **Py & Ancel’s fire-setting experiments**, **Kelany and colleagues’ Aswan quarry survey**, **Juleff’s Sri Lankan furnace research**, and **Chirikure’s southern African materials study**. Together they cover extraction, fuel, material selection, environmental adaptation, and non-linear technological development. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/boom-and-bust-in-bronze-age-britain-major-copper-production-from-the-great-orme-mine-and-european-trade-c-16001400-bc/356E30145B1F6597D8AAA0DDBE69BD51)

### Where confidence is weakest

**Universal historical labor-productivity rates are thinly supported.** Experimental face output, reconstructed metal production, and mine-wide employment statistics cannot be substituted for one another. Ancient output estimates also depend on assumed ore grades, recovery, missing workings, and the duration of active production. [Academia](https://www.academia.edu/80485090/Archaeological_experiments_in_fire_setting_protocol_fuel_and_anthracological_approach)

**The archaeological record is selective.** Surface-boulder extraction can remove much of its own evidence; later quarrying and construction can destroy earlier workings. Surviving monumental or unusually extensive sites should not become the default size of every village operation. [Academia](https://www.academia.edu/5407078/Granite_quarry_survey_in_the_Aswan_region_Egypt_shedding_new_light_on_ancient_quarrying)

**Technology sequences and diffusion remain regionally complex.** The timing and joint appearance of crafts, farming, and metallurgy are not always settled. TCE does not need to choose one contested universal history: independently transferable capabilities are both more flexible and better aligned with the evidence. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/arcm.12784)

**The central modeling choice is therefore to keep geology fixed but usefulness conditional.** A small exposed deposit may sustain an early craft community; a huge poor body may remain untouched for centuries; a flooded working may reopen; and an abandoned quarry may still supply rubble. Those outcomes should emerge from the same inventory, task, knowledge, and institutional systems—not from separate scripted resource eras.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927ed-2230-83e9-9161-a7699332f111)
