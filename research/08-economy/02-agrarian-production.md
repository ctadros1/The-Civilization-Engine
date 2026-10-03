# Pre-industrial agriculture in numbers

## A simulation-ready report for The Civilization Engine

**TCE should represent farming as a set of seasonal production systems, not as a fixed annual output per farmer.** The crucial constraints are the area a household can prepare, sow, weed and harvest *within workable weather windows*; the seed and fodder it must retain; and the extent to which neighboring households experience the same bad year.

The evidence also requires a distinction between **historical observations**, **reconstructions**, and **modern low-input analogues**. Medieval account rolls can provide remarkably detailed seed and harvest quantities. For prehistoric labor requirements, however, much of the usable evidence comes from experiments and reconstructions. Modern traditional farming offers valuable operational measurements, but may involve fertilizers, improved varieties, purchased tools or markets unavailable to an early agrarian society. [Cambridge University Press](https://www.cambridge.org/core/journals/rural-history/article/releasing-the-genie-english-manorial-records-and-their-huge-potential-for-interdisciplinary-studies/F942A70ADFA3495ED44E64FC227AE1E9)

The tables below therefore separate empirical anchors from **explicitly proposed TCE parameters**.

---

## 1. Measurement conventions

Several apparently contradictory agricultural estimates become compatible once their denominators are made consistent.

| Quantity | Convention recommended for TCE |
| --- | --- |
| Labor | Store **person-hours**. Below, **D8 = eight person-hours**. Historical “days” remain unconverted when their duration is unknown. |
| Area | Distinguish **sown hectares**, **harvested hectares**, **physical cropland**, and **total land used**, including fallow and pasture. |
| Yield | Record output **per crop harvest**, not automatically per calendar year. |
| Product weight | Distinguish dry grain, unhusked paddy, milled rice, fresh roots, carcass weight and edible products. |
| Seed ratio | Gross harvested seed-bearing product divided by seed sown, using compatible weight or volume units. It is not a measure of labor productivity. |
| Labor boundary | Separate field operations, initial threshing/drying, transport, milling/cooking, livestock care and infrastructure construction. |

**Confidence notation:** **H** = strong evidence for the stated case; **M** = useful but context-sensitive; **L** = reconstruction or weak historical transferability. These are judgments about evidence, not statistical confidence intervals. **P** identifies a proposed simulation parameter rather than a reported historical measurement.

For example, IRRI gives milling recoveries of roughly **50–55% for a single-stage village rice mill**, versus **65–70% for modern multistage milling**. A tonne of paddy is therefore not a tonne of edible white rice. Nor is every kilogram removed in milling destroyed: bran, husks and broken rice require separate accounting. [Knowledge Bank](https://www.knowledgebank.irri.org/step-by-step-production/postharvest/milling/producing-good-quality-milled-rice/milling-yields)

---

## 2. Mechanisms: implementable causal rules

### 2.1 Make timely operations, rather than annual labor, the immediate constraint

Each crop should generate operations with an opening date, preferred interval, deadline, required labor, required equipment and consequences of incomplete work.

A useful capacity calculation is:

\[
A\_{\max,j}
=
\min\left(
\frac{\text{available person-hours in window }j}
{\text{person-hours required per hectare}},
\frac{\text{available team-hours in window }j}
{\text{draft-team-hours required per hectare}}
\right)
\]

Only include the second term where traction is required. Driver labor belongs in the person-hour budget; the team is a simultaneous resource constraint.

Animal traction often improves **timeliness and cultivated area**, rather than simply multiplying yield per hectare. In a Sahel experiment, mechanized weeding saved substantial labor in particular operations, but total seasonal labor changed much less; fertilizer accounted for much of the yield improvement. [FAOHome](https://www.fao.org/4/x5455b/x5455b23.htm)

**TCE rule:** surplus labor in the dry season cannot compensate for an unweeded crop during its critical growth stage or a harvest left standing through damaging weather.

### 2.2 Separate biological production from collected production

A field can grow a crop that nobody successfully harvests.

Use two stages:

\[
\text{biological harvest}
=
\text{crop biomass}\times\text{harvest index}
\]\[
\text{collected harvest}
=
\text{biological harvest}\times
\text{fraction collected before deterioration}
\]

For biological growth, a simplified daily model can track temperature-dependent development, soil water, nutrient stress and canopy development. FAO’s AquaCrop provides a useful reference architecture: canopy development influences transpiration, transpiration supports biomass production, and harvest index determines the harvested fraction. [FAOHome](https://www.fao.org/aquacrop/overview/calculation-scheme/)

**TCE rule:** improved harvesting tools should first change collection speed and losses. Improved seed, water control or soil fertility should change biological production.

### 2.3 Reserve seed as a physical stock

Seed requirements should normally be expressed as **viable kilograms per hectare**, not as a constant percentage of whatever happens to be harvested.

At a gross seed ratio of 5:1, replacement seed consumes 20% of the harvest. If harvest falls by half while next year’s intended area stays constant, the seed requirement consumes 40%. Selected medieval English records show cereal seed ratios low enough for this distinction to be economically important. [CDL Public Site](https://pub-ucpec2-prd.cdlib.org/ucpressebooks/view?anchor.id=JD_Page_68&brand=ucpress&chunk.id=d0e4996&doc.view=content&docId=ft8199p22b&toc.depth=1&utm_source=chatgpt.com)

**TCE rule:** households can eat seed during a crisis, but doing so reduces next season’s feasible sowing unless seed is borrowed, purchased or redistributed.

Track seed viability separately from weight: a damaged store can retain considerable mass while losing its value for planting.

### 2.4 Treat livestock, crops and soil fertility as one connected system

Draft animals require feed before they can work. Crop residues can become fodder, bedding, fuel or soil cover—but not all of these simultaneously. Manure transfers nutrients from feed and grazing land; it does not create nutrients without an input.

FAO’s mixed-farming case material describes the practical importance of feeding draft animals through seasonal shortages and the trade-offs between milk production and work. Allen’s historical nitrogen analysis likewise emphasizes fertility changes and their potentially slow effects on grain yields. [FAOHome](https://www.fao.org/4/y0501e/y0501e05.htm)

**TCE rule:** reducing pasture or feeding straw to more animals can increase one output while reducing another. Losing draft animals should damage future cultivation capacity, not merely remove meat from inventory.

### 2.5 Make risks spatially and seasonally correlated

Represent shared regional weather, local soil differences and crop-specific susceptibility. Do not give every household an independent annual “bad harvest” roll.

Historical climate–harvest relationships differ by region and season. A study covering Sweden, Switzerland and Spain around 1500–1800 found regionally consistent but generally modest average climate signals; severe anomalies could nevertheless produce much larger food-system shocks. [Copernicus Publications](https://cp.copernicus.org/articles/19/2463/2023/)

**TCE rule:** diversification across crops, planting dates and locations can reduce risk, but nearby farms growing the same crop should remain vulnerable to common failures.

### 2.6 Let institutions modify access and coordination

Institutions should act through concrete mechanisms:

| Institution or arrangement | Implementable effect |
| --- | --- |
| Shared draft teams | More households can cultivate; queues can make poorer or later-booking households miss planting windows. |
| Reciprocal labor groups | Temporarily pool workers during transplanting, weeding or harvest. |
| Irrigation organizations | Allocate water, schedule maintenance and coordinate planting; upstream access can conflict with downstream needs. |
| Commons | Provide grazing and residues subject to access rules and stocking pressure. |
| Landlords and taxation | Transfer grain and labor obligations; fixed obligations expose tenants differently from proportional shares. |
| Granaries and seed lenders | Move stocks across households and seasons, potentially preventing seed consumption and distressed livestock sales. |

These mechanisms are consistent with documented traction constraints and with work on coordinated irrigation systems such as Bali’s water-temple networks. They should not be represented as unexplained percentage bonuses. [FAOHome](https://www.fao.org/4/x5455b/x5455b0g.htm)

---

## 3. Labor requirements: empirical anchors

### 3.1 What the measurements actually show

| Crop or operation; setting | Reported requirement | Interpretation and confidence |
| --- | --- | --- |
| Full manual soil preparation, early-farming reconstruction | **10–20 m²/person-hour**, equivalent to **63–125 D8/ha** | Soil preparation alone. Not applicable to every dibble-planted, lightly tilled or shifting-cultivation system. **Reconstruction, L–M.** |
| Stone-sickle cereal cutting, experimental evidence used in the same reconstruction | **30–40 m²/person-hour**, or **31–42 D8/ha** | Cutting alone; excludes much subsequent handling. **Experimental analogue, M locally; L for universal transfer.** |
| Rice, Ming–Qing Yangtze reconstruction | Approximately **10–15 historical workdays/mu**, or **150–225 days/ha** using about 15 mu/ha | Upper accounting includes water lifting and fertilizer transport. Historical day length and area conversion remain uncertain. **Reconstruction, M–L.** |
| Rice, Philippines example reported by FAO | **84 person-days + 14 animal-days/ha**; harvest and threshing **22 person-days** | Yield **2.5 t paddy/ha**, but uses IR36 and **50 kg urea/ha**. A transitional modern system, not an ancient baseline. **Analogue, M.** |
| Traditional millet treatment, Niger/Sahel | **292 person-hours/ha = 36.5 D8/ha** | Demonstrates that some extensive dryland systems use much less labor than fully dug cereal fields. **Field evidence, M.** |
| Intensified millet treatments in the same experiment | **240 hours/ha manually**, **231 with animal traction** | About **30 versus 28.9 D8/ha** overall, despite larger savings in individual weeding operations. **Field evidence, M.** |
| Cowpea, same Sahel experiment | **423 hours/ha manually = 52.9 D8/ha** | Pod harvesting alone required **175 hours = 21.9 D8/ha**. **Field evidence, M.** |
| Sorghum, Dongargaon, India, 2006–07 | **40 eight-hour days/acre ≈ 98.8 D8/ha** | Bullock-and-human system; hand weeding accounted for **14 days/acre**, around 35% of total labor. **Modern analogue, M.** |
| Potato, Huayopata, Peru | **824 hours/ha = 103 D8/ha** | Associated with **12 t/ha** and substantial purchased fertilizer; not a pre-Hispanic observation. **Modern analogue, M.** |
| Mechanized crop farms, FINBIN data, 2007–2024 | Approximately **4.2–14.8 hours/ha**, or **0.53–1.85 D8/ha**, across farm-size groups | Whole-farm crop labor accounting, not the same boundary as a historical field-operation study. **Modern benchmark, H for dataset; low historical comparability.** |

Sources: early-farming experiments and reconstruction; Allen; FAO rice example; Sahel traction experiment; Indian village resurvey; Peruvian farm analysis; FINBIN analysis. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26932572/)

**Implication:** there is no defensible universal value such as “pre-industrial farming requires 100 days per hectare.” Crop, weed pressure, soil preparation, water management, harvest method and accounting boundary can move the requirement several-fold.

### 3.2 Proposed seasonal task budgets for TCE

The following are **P: authored starting budgets**, not historical measurements. They use eight-hour equivalents and include routine field work, collection and initial post-harvest handling. They exclude cooking, milling, distant transport, livestock care and initial construction of terraces or canals.

| Farming system | Preparation and sowing | Growing-season work | Harvest and field collection | Threshing, drying, sorting | Total D8/ha/crop |
| --- | --- | --- | --- | --- | --- |
| Fully hoed early cereal | 90 | 25 | 35 | 10 | **160** |
| Draft-assisted temperate cereal | 10 | 12 | 14 | 14 | **50** |
| Extensive millet | 5 | 19 | 10 | 6 | **40** |
| Cowpea | 8 | 17 | 22 | 8 | **55** |
| Maize with companion crops | 16 | 30 | 16 | 8 | **70** |
| Labor-intensive transplanted rice | 50 | 55 | 30 | 15 | **150** |
| Potato | 35 | 35 | 40 | 10 | **120** |
| Cassava | 25 | 45 | 25 | 5 | **100** |

These allocations are calibrated in scale to the evidence above; **confidence in the exact allocation is L**. “Growing-season work” includes weeding and applicable irrigation, fertilizer handling or hilling.

Do not spread these hours uniformly across the season. Rice preparation includes several distinct operations, while repeated pod picking or staggered root harvesting has a different calendar from cutting a ripe cereal field.

**Illustrative bottleneck:** four hectares of the proposed temperate cereal require 56 D8 for harvest and collection. With only 20 workable harvest days, two full-time workers supply 40 D8. The household needs 16 additional worker-days, faster equipment, staggered maturity or a smaller cropped area—even though its annual field-labor requirement is only 200 D8.

---

## 4. Yields and seed requirements

### 4.1 Historical and analogue anchors

| Region and setting | Gross yield or seed requirement | Evidence and limits |
| --- | --- | --- |
| Selected medieval English estates, around 1300 | Wheat approximately **0.63–1.06 t/ha**; barley **0.64–1.52 t/ha**; oats **0.36–0.47 t/ha** | Approximate conversion from reported bushels/acre using modern commodity weights; historical measures add uncertainty. **Historical, M.** |
| Medieval English cereal records | Wheat seed ratios roughly **3–6:1**; barley **3–7:1**; oats commonly around **2–3:1** in the cited examples | Different estates and years, not paired observations for every yield above. **Historical, M.** |
| Cucuteni–Trypillia farming reconstruction | Cereal yields around **0.7–1.2 t/ha**; sowing roughly **70–100 kg/ha** in relevant comparisons | Reconstructed from experiments and analogues, not measured Neolithic harvests. **Reconstruction, L–M.** |
| Harsh, low-input pearl-millet environments | Approximately **0.3–0.4 t/ha** | ICRISAT contrasts these with much higher irrigated hybrid yields. **Modern analogue, M.** |
| Maize systems surveyed in Mexico, 2020–21 | Around **0.96 t/ha** in one subsistence system and **2.46 t/ha** in one traditional system | Regional production-system observations, not pre-contact estimates. **Modern analogue, M.** |
| Family potato production in the Peruvian highlands | Yields often **below 8 t fresh roots/ha**; seed tubers around **1.8–2.0 t/ha** | Large seed mass is an important distinction from small-seeded cereals. **Modern analogue, M.** |
| Pacific taro, FAO country statistics, 1992–2002 | Papua New Guinea approximately **5.4–6.4 t fresh product/ha** in the reported series | National reported estimates, not archaeological measurements or a particular cultivation method. **Modern comparison, M–L.** |

Sources: Biddick; Shukurov and colleagues; ICRISAT; Mexican farm survey; Peruvian seed-system study; FAO regional statistics. [CDL Public Site](https://pub-ucpec2-prd.cdlib.org/ucpressebooks/view?anchor.id=JD_Page_68&brand=ucpress&chunk.id=d0e4996&doc.view=content&docId=ft8199p22b&toc.depth=1)

The English figures should not become the universal “medieval yield.” Estate accounts disproportionately describe demesne production. Work on Oakington shows that peasant production can differ materially from the estate sector used in many reconstructions. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2011.00654.x)

### 4.2 Complete starter parameters

These are **P: recommended initial parameter ranges**, assembled for implementation and sensitivity testing. The central values form coherent example systems; the ranges represent variation across sites and management regimes, **not an annual weather distribution**.

| System | Gross yield: central value [range] | Labor D8/ha/crop: central [range] | Seed kg/ha: central [range] | Central gross seed ratio |
| --- | --- | --- | --- | --- |
| Fully hoed early cereal | **0.8 [0.5–1.2] t/ha** | **160 [120–220]** | **90 [70–120]** | **8.9:1** |
| Draft-assisted temperate wheat | **0.8 [0.5–1.3] t/ha** | **50 [35–70]** | **160 [120–200]** | **5:1** |
| Extensive millet | **0.5 [0.3–1.0] t/ha** | **40 [30–70]** | **5 [3–10]** | **100:1** |
| Cowpea | **0.5 [0.25–0.9] t/ha** | **55 [40–90]** | **25 [15–40]** | **20:1** |
| Maize in a mixed plot | **1.5 [0.8–2.5] t maize/ha** | **70 [40–110]** | **20 [15–30]** | **75:1** |
| Transplanted rice | **2.0 [1.5–3.0] t paddy/ha** | **150 [100–225]** | **50 [30–80]** | **40:1** |
| Potato | **6 [4–10] t fresh/ha** | **120 [80–200]** | **1,800 [1,000–2,000]** | **3.3:1** |
| Cassava | **10 [5–15] t fresh/ha** | **100 [60–150]** | Vegetative cuttings | Not comparable |

**Confidence:** generally **M–L for the scale of yield and labor; L for exact combinations and seed defaults without a local calibration**. Their evidence base is the preceding two tables, not a single dataset.

Important implementation qualifications:

* **Do not independently sample every column.** Higher labor can accompany better weeding, heavier harvests or more intensive management.
* Maize yield above is the **maize component only**. Companion beans and squash need their own densities and outputs; do not add three full monoculture yields. An Oaxaca study illustrates why mixed systems require component-level accounting. [DOI](https://doi.org/10.1371/journal.pone.0246281)
* Cassava’s cycle can span **8–24 months**, so its yield must not automatically recur annually. Its fresh mass is also not nutritionally equivalent to dry grain. [FAOHome](https://www.fao.org/4/x5415e/x5415e04.htm)

A high seed ratio does not necessarily imply a superior crop: millet’s tiny seed requirement gives a large ratio even when its harvested mass per hectare is low.

---

## 5. Livestock: output, reproduction, feed and herd size

### 5.1 Use demographic herds, not productive “animal units”

A useful quantitative reference is Otte and Chilonda’s FAO compilation of traditional sub-Saharan systems, drawing on studies from 1973–2000. These are **modern traditional-system analogues**, not direct measurements of ancient livestock. Their means also come from different studies and should not be treated as one perfectly consistent herd. [FAOHome](https://www.fao.org/4/y4176e/y4176e08.htm)

| Parameter | Cattle | Sheep | Goats |
| --- | --- | --- | --- |
| Mean mature female live weight | **244 kg** | **28.7 kg** | **27.8 kg** |
| Mean age at first birth | **47.9 months** | **17.5 months** | About **16.5 months** |
| Birth events per breeding female/year | **0.587** | **1.098** | **1.211** |
| Offspring per birth event | Usually one | **1.12** | **1.34** |
| Juvenile mortality | **21.7% by one year** | **26.7% by six months** | **27.8% by six months** |
| Adult female annual mortality | **6.3%** | **11.1%** | **12.2%** |
| Animals sold or consumed annually, as share of herd | **9.9%** | **20.8%** | **21.4%** |

**Confidence: M for these systems; L for transferring them unchanged elsewhere.** [FAOHome](https://www.fao.org/4/y4176e/y4176e08.htm)

The distinction between juvenile and adult mortality is essential. Applying a 22% calf mortality probability to every adult cow would produce a radically different herd.

### 5.2 Milk: total secretion is not human consumption

FAO’s traditional-system compilation reports average **human milk offtake of about 252 kg per cow per lactation**, with reported values from **60 to 508 kg**. This explicitly excludes milk consumed by calves. [FAOHome](https://www.fao.org/4/y4176e/y4176e08.htm)

Borana research gives a useful contrasting total-production estimate: approximately **680–1,000 kg per lactation**, with people commonly taking **30–40%**. Rainy-season daily production could be roughly twice dry-season production. The study discusses an average household with **eight cows**, which does not mean eight total animals including young stock and males. [FAOHome](https://www.fao.org/4/x5553e/x5553e06.htm)

**TCE rule:** milk depends on recent birth, lactation stage, feed, water, health and the household’s allocation to offspring. Over-milking for immediate human needs should impair calf growth or survival rather than create free food.

### 5.3 Meat and wool

| Output | Useful anchor or starting range | Modeling interpretation |
| --- | --- | --- |
| Low-condition cattle carcass yield | **P: 45–55% of live weight** | FAO reports that dressing percentages below 50% are common in poorly fed systems. A 250 kg animal at 50% gives **125 kg carcass**, not 125 kg boneless meat. |
| Sheep carcass yield, central Mali observations | Approximately **44.5–48.6%**; average carcass around **14.2 kg** | Good low-input comparison; gut fill and carcass definition matter. |
| Pig carcass conversion | Around **70%** in FAO’s technical reference | Useful conversion anchor, but do not import modern slaughter age or growth rate into historical pigs. |
| Awassi wool | About **1.75 kg greasy fleece/ewe/year**, **2–2.5 kg/ram/year** | Greasy wool is not clean fiber or finished cloth. Breed-specific; do not assign wool to every sheep population. |

Sources: FAO slaughterhouse and processing guidance; tropical African small-ruminant observations; Epstein’s Awassi monograph. [FAOHome](https://www.fao.org/4/x6114e/x6114e04.htm?utm_source=chatgpt.com)

Meat is a **stock liquidation or demographic surplus**, not a recurring harvest from the same animal. Selling an animal also transfers it; it does not necessarily create meat immediately.

### 5.4 Calculate pasture from feed, not from a universal hectares-per-cow constant

For a first approximation:

\[
A\_{\text{feed}}
=
\frac{
W \times i\_{\text{DM}} \times 365
}{
Y\_{\text{forage DM}} \times u
}
\]

Here \(W\) is animal live weight, \(i\_{\text{DM}}\) daily dry-matter intake as a fraction of body weight, and \(u\) the usable share of forage production.

A **P baseline of 2.5% of live weight/day** lies within published grazing-cattle intake guidance. A 250 kg animal consequently needs about **2.28 t dry matter/year**. [NCAT](https://www.ncat.org/publication/graziers-math-matching-forage-to-animal-demand/)

Illustrative consequences:

| Annual usable forage, after grazing/access losses | Area supporting that 250 kg animal |
| --- | --- |
| **0.25 t DM/ha** | **9.1 ha** |
| **0.50 t DM/ha** | **4.6 ha** |
| **1.00 t DM/ha** | **2.3 ha** |
| **2.00 t DM/ha** | **1.1 ha** |

These are calculations, not universal stocking recommendations. A field study in western Ethiopia estimated about **0.21 tropical livestock units/ha**, approximately **4.8 ha per 250 kg unit**, under its forage and utilization assumptions. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8866678/)

Annual feed sufficiency is not enough. Winter or dry-season shortages should constrain survival, pregnancy and work capacity even where the annual biomass total looks adequate.

### 5.5 Draft output and household ownership

For a first implementation, use **P: roughly 0.1–0.3 ha per team-workday per tillage pass**, with a team-workday containing about five hours of actual traction. Treat this as a deliberately broad, **low-confidence engineering prior**.

Make the result depend on soil resistance, implement width, depth, slope, pass count and animal condition. Indian measurements place draft force for different operations around **9–19% of animal live weight**, at speeds around **1.6–2.6 km/hour**. Those quantities provide a better physical foundation than a universal “ox productivity multiplier.” [Icar E-Pubs](https://epubs.icar.org.in/index.php/IJAnS/article/view/36135)

For initialization, a small mixed household might be given **P: 0–3 cows, ownership or access to 0–2 oxen, and 0–10 sheep/goats**, with pigs and poultry as separate optional enterprises. These are scenario choices, **not historical averages**. Include livestock-poor households and shared teams; otherwise every household receives too much productive capital.

---

## 6. Farm sizes, fallow and household capacity

Farm size should emerge from the intersection of land rights, labor, draft access, soil productivity and household consumption.

| Setting or calculation | Quantitative anchor | Caution |
| --- | --- | --- |
| Yangtze Delta reconstruction | Holdings decline from around **15 mu to 9 mu**, approximately **1.0 to 0.6 ha**, between c.1620 and the nineteenth century | Reconstructed averages; multiple cropping and labor intensity matter. |
| Same reconstruction | Cropping intensity approximately **1.4 to 1.7 harvest-hectares per physical hectare/year** | This is not a 40–70% yield bonus to each crop. |
| African traction comparisons summarized by FAO | Mean cultivated area around **6.6 ha with draft animals versus 3.3 ha with hand hoes** | Cross-sectional association: wealth and land access also select households into traction ownership. |
| Two crop years followed by ten fallow years | **Six hectares in the rotation per hectare cropped annually** | Arithmetic scenario, not a universal historical fallow schedule. |
| One-third of arable land fallow | **1.5 ha total arable per hectare sown annually** | Pasture, woodland and settlement space are additional. |

Sources for the historical and comparative anchors: Allen and FAO’s farm-level traction assessment. [Paperzz](https://paperzz.com/doc/9199194/agricultural-productivity-and-rural-incomes-in)

For TCE, distinguish:

\[
\text{physical cropland}
\neq
\text{annual sown area}
\neq
\text{annual harvested area}
\]

A household farming one hectare of double-cropped rice and a household holding six hectares of a fallow-based rotation may have comparable food production but very different labor calendars and landscape footprints.

---

## 7. Storage losses and agricultural risk

### 7.1 Storage: avoid a universal “20% of food disappears”

Household surveys from Malawi, Uganda and Tanzania estimated self-reported on-farm post-harvest maize losses equivalent to roughly **1.4–5.9% of national harvest**, concentrated among a minority of households. These estimates can miss unrecognized losses and are not interchangeable with measurements of grain stored for a full year. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S221191241400042X?utm_source=chatgpt.com)

APHLIS estimates losses by stage and incorporates crop, weather, storage duration and marketing conditions. Its chain-wide estimates should not be inserted as a separate annual storage penalty on top of harvest, threshing and transport losses. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/handle/JRC62618)

| Parameter | Recommended representation | Status |
| --- | --- | --- |
| Reasonably dry grain in maintained stores | **P: 2–8% physical loss over 6–12 months** | Sensitivity range, **L historical confidence** |
| Warm, poorly maintained or pest-exposed storage | **P: 10–20% over 6–12 months** | Adverse scenario, not universal average |
| Flooding, roof failure, fire or severe infestation | Separate event affecting a particular store | Do not hide catastrophes inside a smooth average rate |
| Rice stored for weeks to a few months | Around **14% moisture or less** | IRRI operational guidance |
| Rice stored for 8–12 months | Around **13% moisture or less** | IRRI operational guidance |
| Cassava after lifting | Deterioration can begin within **24 hours**, with very short unprocessed shelf life | Make harvesting, transport and processing tightly coupled |

Sources for moisture and cassava deterioration: IRRI and FAO. [Knowledge Bank](https://www.knowledgebank.irri.org/step-by-step-production/postharvest/drying)

Use a compounding hazard:

\[
S\_{t+\Delta t}=S\_t e^{-\lambda\Delta t}
\]

An annual loss of 8% corresponds to approximately **0.69% monthly**, not 8% each month.

Store edible stock and seed stock as identifiable lots. Different roofs, drying practices, containers and pests should create heterogeneity across households.

### 7.2 Weather sensitivity: use crop responses, not rainfall alone

FAO’s first-order water-deficit relationship is:

\[
1-\frac{Y\_a}{Y\_m}
=
K\_y\left(1-\frac{ET\_a}{ET\_m}\right)
\]

The deficit concerns **crop evapotranspiration**, not simply rainfall. Soil storage, irrigation and timing intervene. [FAOHome](https://www.fao.org/4/x0490e/x0490e0e.htm)

| Crop | Seasonal \(K\_y\) | Calculated yield loss at 20% ET deficit |
| --- | --- | --- |
| Maize | **1.25** | **25%** |
| Sorghum | **0.90** | **18%** |
| Spring wheat | **1.15** | **23%** |
| Beans | **1.15** | **23%** |
| Potato | **1.10** | **22%** |

These are modern agronomic response coefficients, useful as starting physiology rather than precise coefficients for every historical landrace. Severe stress, flooding, frost and reproductive-stage damage need additional treatment. [FAOHome](https://www.fao.org/4/x0490e/x0490e0e.htm)

### 7.3 Variability parameters: proposed test envelopes, not established historical constants

There is not a single well-supported coefficient of variation for “pre-industrial yields.” Use local historical series where possible.

For early sensitivity testing:

| System | **P: field-level annual yield CV** |
| --- | --- |
| Managed irrigation with reasonably reliable water | **0.15–0.30** |
| Rainfed farming in relatively favorable climates | **0.20–0.35** |
| Highly variable semi-arid rainfed farming | **0.35–0.60** |

**Confidence: L. These are modeling priors.** A CV of 0.30 means the standard deviation is 30% of the mean; it does not mean every year lies within ±30%.

Once weather, pests and labor failures explicitly generate variation, **do not add this full CV again as independent noise**. Instead, calibrate the resulting total variance, spatial covariance and frequency of severe shortfalls.

Run explicit stress tests for **25%, 50% and 75% production losses**, consecutive bad seasons, canal failure and loss of draft animals. These are test severities, not asserted historical probabilities.

---

## 8. Variation across eras and regions

### Foragers

Do not force foraging into crop-hectare accounting. Model renewable patches, encounter rates, travel, processing, sharing and storage.

The Agta research is useful precisely because it distinguishes out-of-camp food production from domestic work and shows that agricultural involvement can change time allocation. It does not establish one universal “foragers worked only a few hours” parameter. [Nature](https://www.nature.com/articles/s41562-019-0614-6)

### Early farming

Use a mixed economy rather than immediately assigning everyone a complete cereal-farming occupation. Full soil digging, light hoeing and planting into recently cleared ground should be separate techniques.

Experimental early-farming reconstructions show why labor assumptions are decisive: the difference between turning all soil and preparing only planting locations can dominate household capacity. Their yield estimates should remain uncertain rather than being treated as archaeological measurements. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26932572/)

### Pre-industrial systems

The most important contrasts are **ecological and organizational**, not a single ladder of eras:

* Temperate mixed grain–livestock systems need rotation, feed and traction accounting.
* East and South Asian rice systems require water management and concentrated planting labor.
* African millet, sorghum and pulse systems cannot be represented by one European plough package.
* American maize mixtures need component-level outputs; Andean potatoes require substantial seed-tuber stocks.
* Pacific root and tuber production must not be converted into dry-grain equivalents by weight alone.

The numerical contrasts in Sections 3–6 support these separate system definitions; none requires an era gate.

### Industrial and modern farming

Separate technologies that reduce labor from those that raise biological yield. A faster threshing machine, improved drainage, nitrogen supply and an improved cultivar should modify different model components.

The modern FINBIN labor figures demonstrate the scale of labor reduction possible under mechanized crop production, but exclude much labor embodied in machinery, fuel and purchased inputs. Modern calendar dates also do not imply that every farm is mechanized: the Indian, African and Peruvian observations above remain labor-intensive. [Farmdoc Daily](https://farmdocdaily.illinois.edu/2026/01/labor-standards.html)

---

## 9. Stylized facts and numerical validation targets

### 9.1 Small harvest shocks can eliminate the entire marketed surplus

Consider this **illustrative TCE household**, not a claimed historical average:

* Three hectares sown; normal gross yield **800 kg/ha**.
* Physical losses equal **10% before allocation**.
* Next year’s viable seed requirement: **480 kg**.
* Fixed rent: **240 kg grain**.
* Household cereal allocation: **1,250 kg/year**; other foods are modeled separately.

| Item | Normal year | Yield 30% lower |
| --- | --- | --- |
| Gross harvest | 2,400 kg | 1,680 kg |
| After physical losses | 2,160 kg | 1,512 kg |
| After seed and fixed rent | 1,440 kg | 792 kg |
| Balance against household cereal allocation | **+190 kg** | **−458 kg** |

The gross harvest falls 30%, but the small surplus becomes a serious deficit. Proportional rent, rent remission, credit or redistribution produces a different outcome.

**Conservation test:** the rent is still present in the landlord’s inventory. It is not a physical loss to the settlement.

### 9.2 Peak labor should bind before annual labor in some households

The four-hectare example in Section 3 should produce demand for temporary workers, reciprocal labor or equipment even when household members have substantial spare time elsewhere in the year.

A model in which spare winter labor automatically compensates for missed harvest work is incorrectly aggregating time.

### 9.3 Traction should sometimes expand area without raising yield

The Sahel experiment and broader African comparisons are useful tests. Introducing traction should be able to increase cultivated area and improve operation timing while producing only modest direct changes in yield per hectare. [FAOHome](https://www.fao.org/4/x5455b/x5455b23.htm)

### 9.4 Dairy output should be seasonal and reproduction-dependent

Using the illustrative cattle means above, four breeding cows at 0.587 births per year and 252 kg human offtake per lactation produce roughly:

\[
4\times0.587\times252\approx592\text{ kg/year}
\]

This is an accounting illustration using pooled means, not a forecast for a particular herd. It is nevertheless a useful check against giving every adult cow a full lactation every year.

### 9.5 Drought should connect crop, feed and herd losses

Borana research reports much higher calf mortality during drought—potentially **70–90% of the calf crop**—than during ordinary years. This is a local severe-drought observation, not an annual default. It provides a test for multi-year recovery: replacing lost livestock takes time even after rainfall returns. [FAOHome](https://www.fao.org/4/x5553e/x5553e06.htm)

### 9.6 Farm structure should differ without scripted cultures

Under suitable parameters, TCE should permit small labor-intensive irrigated holdings, larger extensive farms, mixed subsistence–market households, livestock-poor cultivators and households specializing in traction or herding.

**Do not hard-code a universal percentage of farmers.** Let it emerge from net food production, household labor demands, distribution, transport and nonfarm employment.

---

## 10. Recommended implementation for TCE

### State and simulation resolution

Use the **household as the planning and inventory unit**, with individuals executing work.

| Entity | Minimum useful state |
| --- | --- |
| Person | Available hours, health, skills, current task, household, obligations |
| Household | Food and viable seed lots, animals, tools, claims to land, debts, expected needs |
| Field | Area, soil water, fertility, crop, development stage, weed pressure, operation completion |
| Livestock | Species, age, sex, mass/body condition, reproductive state, lactation, work capacity |
| Institution | Water rights, common-land rules, labor obligations, shared assets, reserves |
| Region | Weather, pest pressure, forage growth and spatially correlated shocks |

A practical update scheme is daily field growth and work scheduling, event-driven births and harvests, and slower soil-fertility updates. Avoid simulating individual plants. For large herds, cohorts can handle routine demographics while named or visibly important animals remain individual entities.

### Planning behavior

Households should choose crops and area using expected food security and feasible peak labor—not only expected profit.

A useful planning sequence is:

1. Protect a minimum viable seed reserve.
2. Estimate household consumption and foreseeable obligations.
3. Choose a portfolio of crops and livestock.
4. Check each seasonal labor and traction window.
5. Revise plans when water, health, weather or prices change.

Implementation should allow imperfect forecasts. Households can overplant, underestimate weeds or lose access to a promised team.

### What to simplify first

For v1, I would simplify soil chemistry to a few fertility pools, reduce disease to a small number of crop and livestock hazard classes, and use authored crop-response curves rather than a full agronomic model at runtime.

I would **not** simplify away task deadlines, seed viability, seasonal feed shortages, household ownership or correlated weather. Those are the mechanisms needed to produce plausible crises and surplus economies.

### Existing models and games worth studying

| Reference | What to borrow | What not to assume |
| --- | --- | --- |
| **FAO AquaCrop** | Daily water–growth–yield structure; offline response-curve generation | Modern cultivar defaults are not historical landraces. |
| **Village Ecodynamics Project** | Household–landscape interaction and archaeological calibration | Its regional calibration is not globally transferable. |
| **MayaSim** | Coupling land use, settlement, environment and trade | A proof-of-concept model is not a uniquely established historical explanation. |
| **Lansing–Kremer irrigation models** | Coordination between water allocation and cropping decisions | Institutions should not be reduced to one universal irrigation bonus. |
| **Farthest Frontier** | Legible rotations, field condition, fertility and work scheduling | Game coefficients and fixed rotation structures are not empirical parameters. |

Sources: official AquaCrop documentation, the VEP model repository, the MayaSim paper, Lansing and Kremer, and Crate’s official farming documentation. [FAOHome](https://www.fao.org/aquacrop/overview/calculation-scheme/)

---

## 11. Source priorities and unresolved evidence

### Best foundations for further calibration

| Source or dataset | Best use |
| --- | --- |
| **Bruce Campbell’s medieval English crop-yield database** | Seed, output and yield variation; over **30,000 observations** from roughly **230 manors**, covering **1211–1495**. Account for estate and reporting biases. |
| **Biddick, *The Other Economy* (1989)** | Connecting cereal production, estate livestock, fodder and pastoral products. |
| **Shukurov et al., “Productivity of Premodern Agriculture in the Cucuteni–Trypillia Area” (2015)** | Transparent early-farming reconstruction and experimental labor assumptions. |
| **Allen, “Agricultural productivity and rural incomes in England and the Yangtze Delta” (2009)** | Reconstructions linking land, labor, cropping intensity and household income. |
| **Buck survey material and Hoken’s restored Chinese farm microdata** | Early twentieth-century farm-level variation; the restored dataset covers **2,102 farms in 20 counties across nine provinces**. |
| **Otte and Chilonda, FAO traditional livestock-system compilation (2002)** | Age structure, reproduction, mortality, weights and human milk offtake. |
| **Ljungqvist et al., *Climate of the Past* (2023)** | Region-specific historical climate–harvest relationships and associated data. |
| **APHLIS and Kaminski–Christiaensen (2014)** | Loss-stage accounting and the difference between modeled chain losses and household-reported losses. |

Dataset and methodological sources: [Cambridge University Press](https://www.cambridge.org/core/journals/rural-history/article/releasing-the-genie-english-manorial-records-and-their-huge-potential-for-interdisciplinary-studies/F942A70ADFA3495ED44E64FC227AE1E9)

### Claims that should remain flagged

**Prehistoric yields are reconstructed, not directly observed.** Archaeology can constrain crops, tools, settlement and storage, but usually cannot supply an annual yield series comparable to an estate account. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26932572/)

**Labor comparisons are boundary-sensitive.** Water lifting, manure transport, guarding crops, domestic processing and women’s work can be included or omitted differently. The Yangtze–England comparison is especially sensitive to reconstruction choices; use it as a range of models rather than one settled productivity ranking. [Paperzz](https://paperzz.com/doc/9199194/agricultural-productivity-and-rural-incomes-in)

**Estate yields are not automatically peasant yields, and grain prices are not harvest measurements.** Both substitutions can create misleading calibration targets. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2011.00654.x)

**Modern traditional systems are not unchanged survivals of antiquity.** Their measurements are valuable, but each borrowed parameter needs an explicit note about varieties, purchased inputs, tools and accounting scope.

**Bottom line:** build TCE’s first agricultural economy around **seasonal work queues, physical seed reserves, crop–livestock feed budgets, differentiated storage and correlated bad years**. Use the proposed crop systems as adjustable starting points, and validate the resulting household surpluses, labor peaks and recovery trajectories before narrowing individual yield coefficients.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92870-f174-83ea-8ea4-8af615a2032f)
