# Materials technology for The Civilization Engine

## Executive recommendation

**Model materials as production capabilities, not a sequence of “Stone → Bronze → Iron” upgrades.** A society can know how to work a material without knowing how to extract it, possess a recipe without having suitable raw materials, or produce excellent objects without being able to manufacture them cheaply.

Three historical findings are especially important for TCE. Pottery existed among East Asian foragers long before agriculture. Early tin bronze could be produced by smelting mixed ores rather than combining separately refined copper and tin. Iron became widespread long after its first manufacture; its adoption cannot be explained by material superiority alone. These findings rule out several familiar technology-tree prerequisites. [PubMed](https://pubmed.ncbi.nlm.nih.gov/22745428/)

The recommended architecture separates:

| Layer | What TCE should represent |
| --- | --- |
| **Material** | Composition and properties: carbon content, alloying elements, inclusions, porosity, hardness, toughness, heat resistance. |
| **Recipe** | Inputs, operating conditions, intermediate products, losses, labor, fuel, elapsed time, and quality distribution. |
| **Equipment** | Furnace geometry, draft, refractory lining, molds, hammers, grinding equipment, drying space. |
| **Knowledge** | What particular people and workshops can reproduce reliably—not a civilization-wide binary flag. |
| **Economic adoption** | Whether production is worthwhile given demand, wages, fuel, transport, raw materials, and competing products. |

**Evidence convention:** historical dates below are approximate attestations, not mandatory invention dates. Quantitative entries distinguish **observations**, **calculations**, and **TCE calibration priors**. The latter are explicitly proposed starting values, not reconstructed historical averages.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Raw materials determine which recipes work

“Copper ore,” “iron ore,” “clay,” and “stone” should not be homogeneous resources. The significant differences include metal concentration, mineral form, unwanted constituents, fracture behavior, and the ingredients required to produce a workable slag or ceramic body.

For example, archaeometallurgical investigation of copper production in South Africa’s northern Lowveld found that locally silica-poor ores required additions supplying silica and alumina. A recipe successful elsewhere could therefore fail on these deposits. **Flux is not universally limestone, and some ores need no separately added flux.** [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0305440316301261)

**Implementable rule:** every deposit has a small, persistent material profile. Recipes specify compatible profiles and optional corrective additions.

For metals, use an elemental accounting equation:

\[
m\_{\text{usable metal}}
=
m\_{\text{prepared ore}}
\times g\_{\text{metal}}
\times r\_{\text{extraction}}
\times r\_{\text{consolidation}}
\]

Here, \(g\_{\text{metal}}\) is elemental metal fraction, not the fraction of an ore mineral. Rejected objects, remelting losses, and additions of alloying elements are subsequent accounting steps.

Do not make all unrecovered metal disappear. Allocate it to slag, furnace deposits, spills, or recyclable intermediates.

### 1.2 Heat is a process, not a temperature threshold

A furnace reaching a high peak temperature does not guarantee success. Charge temperature, time at temperature, atmosphere, airflow, particle size, moisture, and lining survival all matter.

Experimental lime firings illustrate this sharply: some runs reached high temperatures yet converted very little limestone because heating and gas movement through the charge were inadequate. A reported peak near 950°C was not equivalent to complete calcination. [exarc.net](https://exarc.net/issue-2013-1/ea/experimental-lime-burning-based-findings-roman-empire-period)

**Implementable rule:** represent a few operating variables rather than simulating fluid dynamics:

* Effective charge temperature and accumulated heat exposure.
* Oxidizing/reducing atmosphere.
* Draft and charge permeability.
* Lining condition and moisture.

Use recipe-specific response curves. More draft can improve combustion but also increase oxidation or alter carbon uptake; “more air” must not be a universal productivity bonus.

### 1.3 Separate extraction from working

The main production chains should be:

| Material family | Production chain | Important alternative |
| --- | --- | --- |
| Lithics | Select stone → detach blank → shape/retouch → grind where appropriate → haft → maintain | A usable cutting flake may require little finishing; an axe or prestige object may require much more. |
| Copper | Mine/collect → sort/crush → prepare ore → reduce → recover/refine metal → cast or hammer → anneal/finish | Native copper or imported metal bypasses ore reduction. |
| Bronze | Obtain copper and tin, or suitable mixed ores → alloy/co-smelt → cast → work and finish | Arsenical copper is a parallel alloy family, not simply defective bronze. |
| Bloomery iron | Prepare ore → reduce in charcoal furnace → extract slag-bearing bloom → consolidate → forge stock → make object | Imported blooms or bars support smithing without local smelting. |
| Steel | Produce/select carbon-bearing iron → control carbon distribution → forge → heat-treat where appropriate | Carburization, direct production, crucible processing, and decarburization are different routes. |
| Ceramics | Prepare clay/body → form → dry → fire → cool → optionally glaze and refire | Open firing can make pottery without a permanent kiln. |
| Glass | Prepare silica/flux/stabilizer batch → react/melt → refine/color → form → anneal | Imported raw glass or cullet supports secondary workshops without primary glassmaking. |
| Lime/concrete | Quarry carbonate rock → calcine → slake or hot-mix → combine with aggregate/additives → place → cure | Air-lime mortar, hydraulic mortar, and concrete require different recipe conditions. |

This separation is historically consequential. Native copper working preceded extractive metallurgy; early metallurgical furnaces were not simply pottery kilns repurposed unchanged. Egyptian Late Bronze Age evidence also distinguishes primary glass production from subsequent processing. [Persee](https://www.persee.fr/doc/paleo_0153-9345_2000_num_26_2_4716?utm_source=chatgpt.com)

### 1.4 Iron and steel need branching routes

**Bloomery iron is produced principally by solid-state reduction, not by melting pure iron.** The resulting bloom contains metal and slag and requires consolidation. Carbon uptake can produce heterogeneous iron and steel within the same operation.

An experimental bloomery series produced highly carburized steel in one run and material identified as cast iron in another. This means the physical boundary between “bloomery” and “cast-iron-producing conditions” is not a magical technology switch, although reliable production and useful casting are additional achievements. [EXARC](https://exarc.net/issue-2013-2/ea/production-high-carbon-steel-directly-bloomery-process-theoretical-bases-and-metallographic-analyses)

For TCE, distinguish:

\[
\text{steel supply}
=
\text{direct steely bloom}
\;\mathbf{OR}\;
\text{carburized iron}
\;\mathbf{OR}\;
\text{decarburized high-carbon iron}
\;\mathbf{OR}\;
\text{crucible-processed charge}
\]

South Asian crucible traditions and Sri Lankan wind-powered direct steelmaking demonstrate that these alternatives should not be forced through a single European industrial sequence. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S030544032030145X)

### 1.5 Properties matter more than a material’s rank

Tin bronze can be work-hardened strongly, while arsenical copper’s ductility can make it attractive for sheet-metal production. Experimental comparison supports interpreting Andean alloy choices as functional and culturally situated—not merely as incomplete progress toward tin bronze. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1179/009346996791973774?utm_source=chatgpt.com)

Likewise, cast iron needs separate treatment from forged iron. Chinese producers used prolonged annealing and decarburization to obtain more useful properties from cast objects. “Cast iron is always brittle and useless for tools” is therefore as misleading as “iron is always better than bronze.” [Don Wagner](https://www.donwagner.dk/cice/cice.html)

**Implementable rule:** evaluate tools using task-specific properties:

\[
\text{task value}
=
f(\text{edge retention},\text{toughness},\text{geometry},
\text{repairability},\text{mass},\text{price})
\]

For buildings, use compressive capacity, tensile vulnerability, water resistance, fire behavior, joints, and structural geometry. Material discovery alone must not unlock a safe large dome or bridge.

### 1.6 Lime, mortar, and concrete are separate capabilities

The basic lime cycle is:

\[
\mathrm{CaCO\_3 \rightarrow CaO + CO\_2}
\]\[
\mathrm{CaO + H\_2O \rightarrow Ca(OH)\_2}
\]

Ordinary air-lime mortar subsequently hardens largely through carbonation. Hydraulic binders require a different chemical pathway, involving suitable reactive constituents. Adding arbitrary sand to lime does not create an underwater-setting binder. Roman volcanic-ash mortars developed binding phases through lime–pozzolan reactions, with continuing changes during curing. [Bristol ChemLab](https://www.chm.bris.ac.uk/motm/lime/limeh.htm?utm_source=chatgpt.com)

Concrete is then a **construction system**: binder, fine and coarse aggregate, mixing, placement, support, and curing.

Recent research supports hot mixing with quicklime in particular Roman contexts. A study published in December 2025 identified direct archaeological evidence at a Pompeii construction site buried in 79 CE. That strengthens the case for the technique, but not the claim that every Roman concrete used it or that all ancient concrete possessed universal self-repairing properties. [DOI](https://doi.org/10.1038/s41467-025-66634-7)

### 1.7 Scale, transport, and institutions govern adoption

Model delivered cost rather than assigning a fixed price to each material:

\[
c\_{\text{good output}}
=
\frac{
c\_{\text{inputs}}+c\_{\text{fuel}}+
wL+c\_{\text{transport}}+
c\_{\text{equipment}}+c\_{\text{finance}}
}{
m\_{\text{accepted output}}
}
\]

A large kiln saves repeated heating only when enough material and demand exist to fill it. Larger batches also increase working capital and the consequences of failure.

Do not require state control for substantial materials industries. At the Maya settlement of Kiuic, research identified decentralized lime production: approximately **46% of small corporate groups had direct access to a kiln**, with others apparently obtaining lime through exchange. Kiln locations balanced raw-material collection against delivery costs. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0278416517300235?utm_source=chatgpt.com)

---

## 2. Quantitative parameters and production costs

### 2.1 Measured or historically anchored values

**Confidence below concerns transfer into a general simulation.** A carefully measured experiment can still have low representativeness for other ores, workers, or furnaces.

| Parameter | Value or range | Boundary and interpretation | Confidence / source |
| --- | --- | --- | --- |
| Traditional charcoal yield | **15–25%** of oven-dry wood mass | Approximately **4–6.7 kg dry wood/kg charcoal**. Whole-process yield, not ideal laboratory carbonization. | Medium; Smil’s historical synthesis. [ScienceDirect](https://www.sciencedirect.com/book/monograph/9780128042335/still-the-iron-age) |
| Charcoal carbonization temperature | Around **450–500°C** as a practical compromise | Higher final temperatures increase fixed-carbon content but reduce retained mass; not the eventual furnace operating temperature. | High for mechanism; medium for transfer. FAO. [FAOHome](https://www.fao.org/4/x5328e/x5328e05.htm) |
| Large earth-mound charcoal cycle | Example: **24 days** | Four loading, six carbonizing, ten cooling, four unloading; an example of equipment occupancy, not 24 days of continuous labor per worker. | Medium; FAO example. [FAOHome](https://www.fao.org/4/x5328e/x5328e07.htm) |
| Experimental direct steelmaking | **20 kg roasted ore + 45 kg charcoal → 2.2 kg recovered high-carbon material** | About **9.1 kg ore and 20.5 kg charcoal/kg product**. Product was not a finished, clean tool bar. | Medium for this run; low as a universal coefficient. Wrona 2013. [EXARC](https://exarc.net/issue-2013-2/ea/production-high-carbon-steel-directly-bloomery-process-theoretical-bases-and-metallographic-analyses) |
| Duration of that steelmaking run | **4.5 hours** | Included preheating; earlier ore roasting and subsequent finishing were separate. Not a person-hour measurement. | Same source and boundary. [EXARC](https://exarc.net/issue-2013-2/ea/production-high-carbon-steel-directly-bloomery-process-theoretical-bases-and-metallographic-analyses) |
| Timed experimental iron production | Approximately **44+ person-hours / 8 kg rough bloom** | About **5.5+ person-hours/kg** including furnace construction; excludes ore acquisition, charcoal making, and manual bellows labor. Used an electric blower and modern tools. | Medium observation; low transfer. Markewitz and collaborators. [Wareham Forge](https://warehamforge.ca/ironsmelting/iron2021/A%2CE%26I-full/index.html?utm_source=chatgpt.com) |
| Experimental Neolithic-style lime burning | **500 kg limestone + approximately 1,000 kg mixed fuel → approximately 250 kg quicklime** | About **4 kg mixed branches/dung fuel/kg quicklime**; do not relabel this as oven-dry wood. Approximately 24-hour firing. | Medium; Goren and Goring-Morris 2008. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1002/gea.20241) |
| Pure limestone requirement | **1.785 kg CaCO₃/kg CaO** | Calculated stoichiometric minimum; increase for impurities, incomplete burning, and handling losses. | High; calculation from lime chemistry. [Bristol ChemLab](https://www.chm.bris.ac.uk/motm/lime/limeh.htm?utm_source=chatgpt.com) |
| Minimum slaking water | **0.321 kg water/kg CaO** | Chemical minimum only; workable mortar or lime putty requires additional water. Produces **1.321 kg dry Ca(OH)₂**. | High; stoichiometric calculation. [Bristol ChemLab](https://www.chm.bris.ac.uk/motm/lime/limeh.htm?utm_source=chatgpt.com) |
| Earthen wall application labor | **20–21.4 person-hours/m³** | Derived from three people applying 140–150 liters in one hour; excludes some upstream preparation and transport. | Medium-low transfer; WF16 reconstruction. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1179/0075891415Z.00000000063?scroll=top&tab=permissions&utm_source=chatgpt.com) |
| Late Bronze Age primary glassmaking | First stage **900–950°C**; subsequent stage **1,000–1,100°C** | Qantir evidence; formulation- and process-specific. Avoid a universal “glass requires 1,500°C” rule. | Medium-high for that process. Rehren and Pusch 2005. [PubMed](https://pubmed.ncbi.nlm.nih.gov/15961663/) |
| Chinese high-fired ceramic bodies | Approximately **1,200°C and above** | Appropriate raw materials and atmosphere are as important as temperature. | Medium-high; Yin, Rehren, and Zheng 2011; Zong et al. 2024. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440311001440) |
| Selected crucible steels | Approximately **1.2–1.6 wt% carbon**, processing around **1,300–1,400°C** | A particular high-carbon steel family, not a definition of all steel. | Medium; archaeometallurgical study. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2352409X18301469) |
| Malleabilizing cast iron | Around **950°C for days** in reconstructed Chinese treatments | Atmosphere, section thickness, and composition determine the outcome. | Medium; Wagner’s metallographic synthesis. [Don Wagner](https://www.donwagner.dk/cice/cice.html) |
| Modern lime-kiln fuel energy | Shaft kilns **3.2–4.9 MJ/kg**; rotary kilns **5.1–9.2 MJ/kg** | Published industrial benchmarks, not ancient defaults; excludes unrelated downstream manufacturing. | Medium-high; Eriksson et al. 2014. [SCI Journals](https://scijournals.onlinelibrary.wiley.com/doi/10.1002/ese3.40?utm_source=chatgpt.com) |

**Important accounting trap:** a Maya experimental fuel-to-lime ratio of **3.94:1** is reported **by volume**, not mass. It cannot be entered directly as kilograms of wood per kilogram of lime. This is a concrete example of why every parameter needs an explicit measurement basis. [ResearchGate](https://www.researchgate.net/publication/326018872_PREHISPANIC_MAYA_BURNT_LIME_INDUSTRIES_PREVIOUS_STUDIES_AND_FUTURE_DIRECTIONS?utm_source=chatgpt.com)

### 2.2 What the charcoal numbers imply

Combining the observed steelmaking run above with a 15–25% charcoal yield gives:

\[
20.5\ \frac{\mathrm{kg\ charcoal}}{\mathrm{kg\ recovered\ product}}
\times
4\text{–}6.7\ \frac{\mathrm{kg\ dry\ wood}}{\mathrm{kg\ charcoal}}
\approx
82\text{–}137\ \frac{\mathrm{kg\ dry\ wood}}{\mathrm{kg\ product}}
\]

This is **an illustrative high-cost reconstruction**, not an average ancient iron economy. It excludes some preparatory and finishing work. Nevertheless, it shows why ignoring charcoal manufacture can understate the landscape and labor requirements of metallurgy by a large factor. [EXARC](https://exarc.net/issue-2013-2/ea/production-high-carbon-steel-directly-bloomery-process-theoretical-bases-and-metallographic-analyses)

### 2.3 Explicit TCE starting priors where evidence is thin

There is no defensible universal archaeological table of person-hours and fuel consumption covering all these industries. Workshop records and experiments use incompatible boundaries, and object complexity matters enormously.

The following are **authored sensitivity ranges**, intended to make a prototype run while better local datasets are assembled. **They are not historical estimates or confidence intervals.** Their provenance is this modeling recommendation; historical confidence is low.

Fuel is charged at the workshop. Labor excludes mining/quarrying, fuel production, long-distance transport, construction installation, and elaborate decoration unless noted.

| Recipe/output | Initial workshop labor prior | Initial fuel prior | Scope |
| --- | --- | --- | --- |
| Simple usable stone flake | **0.02–0.2 person-hours/piece** | None | Prepared core available; not a finished biface. |
| Ground stone axehead | **5–40 person-hours/head** | None | Strongly dependent on stone and finish; exceptional objects can exceed this range. |
| Ordinary hand-built pottery | **0.3–2 person-hours/kg** | **0.5–3 kg dry wood/kg fired ware** | Clay preparation, forming, allocated firing attendance, sorting; drying is elapsed time. |
| Plain fired brick | **10–80 person-hours/tonne** | **0.2–1 kg dry wood/kg fired brick** | Large-batch, plain units; inefficient firings may exceed the fuel envelope. |
| Charcoal | **20–100 person-hours/tonne charcoal** | Wood from yield parameter above | Kiln operation and handling only; wood cutting and hauling separate. |
| Small-scale copper smelting/refining | **2–15 person-hours/kg usable metal** | **5–60 kg charcoal/kg usable metal** | Deliberately very broad; ore chemistry, grade, and scale dominate. |
| Bronze remelting and simple casting | **1–6 person-hours/kg accepted casting** | **0.5–3 kg charcoal/kg casting** | Excludes primary metal production and elaborate mold manufacture. |
| Bloom-to-bar iron chain | **6–20 person-hours/kg usable bar** | **5–30 kg charcoal/kg usable bar** | Includes smelting and consolidation; excludes upstream ore and charcoal labor. |
| Simple forged iron tools | **2–10 person-hours/kg finished object** | **1–5 kg charcoal/kg object** | Additional to producing the bar; geometry and heat-treatment requirements vary. |
| Primary glass | **1–6 person-hours/kg usable raw glass** | **2–15 kg dry wood/kg glass** | Batch making only; small inefficient operations can fall outside the envelope. |
| Glass vessel working | Core-formed **2–12**; blown **0.2–2 person-hours/kg** | Track reheating separately | Cost reduction from blowing is represented primarily through labor and throughput. |
| Quicklime | **10–60 person-hours/tonne** | **1–5 kg dry wood-equivalent/kg quicklime** | A provisional energy-equivalent envelope, not interchangeable with measured mixed-fuel masses. |

These priors should be sampled **jointly**, not independently. Small batches tend to combine high fuel consumption, high labor per kilogram, and high rejection rates. Skilled production should improve reliability before it necessarily improves maximum output.

For v1, a practical approach is to give every recipe three profiles—experimental, established, and optimized—each with internally consistent coefficients.

---

## 3. Technology graph: nodes, prerequisites, and unlocks

### How to read the graph

“Prerequisites” below are **physical or operational requirements**, not assertions that historical inventors consciously followed this graph. An input can be locally produced **or imported**. Where a global first is unresolved, the table gives a secure horizon or labels the uncertainty rather than inventing a precise date.

### 3.1 Stone, earth, and ceramics

| Node | Required capabilities/resources | Approximate appearance or secure horizon | Concrete unlocks |
| --- | --- | --- | --- |
| **L1 — Controlled stone flaking** | Suitable stone; percussion skill | Lomekwi, Kenya, **3.3 million years ago**. [Nature](https://www.nature.com/articles/nature14464) | Cutting flakes, scrapers, cores, tool-maintenance recipes. |
| **L2 — Ground-edge stone tools** | Suitable blanks; abrasives; grinding skill; hafting for composite tools | Australian evidence **44,000–49,000 years old**. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/03122417.2016.1164379?utm_source=chatgpt.com) | Durable axe/adze edges; woodworking tools; grinding and polishing occupations. |
| **L3 — Organized stone construction** | Stone selection/extraction; moving and setting; structural practice | No single first; Neolithic monumental traditions provide well-studied cases, including northern European megaliths. [DOI](https://doi.org/10.1515/opar-2022-0357?utm_source=chatgpt.com) | Rubble walls, stone foundations, megaliths; dressed-block recipes as later refinements. |
| **E1 — Earthen construction and sun-dried units** | Clay-bearing soil; water; forming; drying; weather protection | Southwest Asian early Neolithic; earthen structures at WF16, approximately **10th–9th millennium BCE**. [DOI](https://doi.org/10.1179/0075891415Z.00000000063?utm_source=chatgpt.com) | Earthen walls, plaster, sun-dried units, low-fuel buildings; periodic repair work. |
| **C1 — Fired pottery** | Clay/body preparation; forming; drying; controlled firing | Xianrendong, China, **19,000–20,000 calibrated years BP**, among foragers. [PubMed](https://pubmed.ncbi.nlm.nih.gov/22745428/) | Cooking/storage vessels, lamps, fermentation containers, ceramic components. |
| **C2 — Controlled kiln firing** | Heat-retaining structure; managed fuel/airflow; compatible lining | Gradual development rather than one invention; specialized kiln traditions documented in prehistoric Mesopotamia. [Zanco Journal](https://zancojournal.su.edu.krd/index.php/JAHS/en/article/view/1784?utm_source=chatgpt.com) | Larger and more repeatable ceramic batches; improved temperature/atmosphere control. |
| **C3 — Fired architectural ceramics** | Formed and dried clay units; firing capacity; handling infrastructure | Chinese fired bricks securely around **3000 BCE**; major Indus application **2600–1900 BCE**. Earlier regional claims vary in definition. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/arcm.12014?utm_source=chatgpt.com) | Fired bricks, paving, drains; roof tiles and pipes require their own forms/recipes. |
| **C4 — High-fired bodies and ash glazes** | Suitable clay/stone; approximately 1,200°C-class firing; atmosphere control; glaze ingredients where used | Chinese proto-porcelain traditions documented from approximately **1700 BCE** onward. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440311001440) | Low-porosity vessels, specialized ceramic bodies, glazed high-fired wares. |
| **C5 — White porcelain** | Selected low-colorant body; high-temperature firing; controlled glaze/body compatibility | Secure northern Chinese white-porcelain development by **6th century CE**; “first porcelain” depends on definition. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/arcm.12969) | Thin, hard white vessels; high-value tableware and specialized ceramic production. |

**Do not make C1 depend on agriculture, or L2 depend on farming.** Also, ordinary pottery, kiln construction, and refractory ceramics are related but not interchangeable competencies.

### 3.2 Metals and enabling processes

| Node | Required capabilities/resources | Approximate appearance or secure horizon | Concrete unlocks |
| --- | --- | --- | --- |
| **M1 — Native-metal working and annealing** | Native copper; hammering; controlled reheating for annealing | Anatolia: copper working in the **9th millennium BCE**, deliberate heating established very early thereafter. [Open METU](https://open.metu.edu.tr/handle/11511/114937?utm_source=chatgpt.com) | Beads, pins, sheet, small tools; working imported copper without smelting. |
| **F1 — Deliberate charcoal production** | Wood; oxygen-limited carbonization; fire management | Exact first uncertain; central to early extractive metallurgy by the **5th millennium BCE**. [Persee](https://www.persee.fr/doc/paleo_0153-9345_2000_num_26_2_4716?utm_source=chatgpt.com) | Charcoal good; compact high-carbon metallurgical fuel; charcoal pits/mounds. |
| **F2 — Metallurgical draft and refractory hearths** | Heat-resistant construction; natural draft **OR** blowpipes/bellows; fuel management | Early metallurgical traditions, **5th–3rd millennia BCE**, with multiple regional designs. [Persee](https://www.persee.fr/doc/paleo_0153-9345_2000_num_26_2_4716?utm_source=chatgpt.com) | Smelting hearths, tuyeres, crucibles, improved furnace control; not a required pottery-kiln lineage. |
| **M2 — Extractive copper metallurgy** | Compatible copper ore; F1/F2-type reducing conditions; recovery skill | Belovode, Serbia, approximately **5000 BCE**, secure evidence. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440310001986) | Smelted copper; ore preparation and slag-handling recipes; smelter occupation. |
| **M3 — Copper-alloy melting and casting** | Copper/alloy supply; melting vessel/hearth; molds; finishing | Established during **5th-millennium BCE** Balkan metallurgy; technologies and forms diversify subsequently. [DOI](https://doi.org/10.1007/s10963-021-09155-7?utm_source=chatgpt.com) | Cast tools, fittings, vessels, standardized blanks; mold-making services. |
| **M4 — Arsenical and mixed-ore alloying** | Appropriate ores or alloy feed; composition selection; compatible smelting/working | Early alloy traditions; Balkan tin-bearing mixed-ore bronze at **Pločnik, approximately 4650 BCE** is especially important. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/1420706/) | Alternative copper alloys without requiring separately refined tin; specialized sheet or cast goods. |
| **M5 — Cassiterite concentration and tin extraction** | Tin-bearing deposit; beneficiation; reduction; separation | Proposed Early Bronze Age production at Kestel/Göltepe, Anatolia, approximately **2600 BCE**; interpretation has been debated. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1475-4754.1996.tb00777.x?utm_source=chatgpt.com) | Tin metal; tin-exporting settlements; an alternative to imported tin or mixed-ore smelting. |
| **M6 — Controlled tin-bronze recipes** | Copper + tin supply **OR** suitable mixed-metal feed; melting/mixing control | Sporadic earlier examples; major Bronze Age production and trade established well before the **late 2nd millennium BCE**. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/1420706/) | Repeatable alloy grades; cast tools and weapons, fittings, vessels, recyclable alloy stock. |
| **I1 — Bloomery reduction** | Suitable iron ore; reducing furnace; charcoal; draft; slag management | Limited Anatolian extractive iron metallurgy in the **early 2nd millennium BCE**; major expansion late 2nd–early 1st millennium. [Springer](https://link.springer.com/article/10.1007/s10814-019-09129-6?utm_source=chatgpt.com) | Slag-bearing iron/steel blooms; bloomery workshop; local iron supply. |
| **I2 — Bloom consolidation and iron forging** | Bloom or imported iron; reheating hearth; hammer/anvil tools; skill | Develops with practical bloomery production; distinct from ore reduction. [Institute of Computer Science](https://www.tf.uni-kiel.de/matwis/amat/iss/kap_a/advanced/aa_2_5.html) | Bars, nails, agricultural tools, fittings, repairs; merchant blooms can feed independent smiths. |
| **I3 — Carbon control and steel selection** | Iron production/working; recognizing and selecting steely material **OR** carburizing treatment | **2nd–1st millennium BCE Eurasian evidence**, but carbon-bearing artifacts do not automatically prove deliberate steel recipes. [Ouci](https://ouci.dntb.gov.ua/en/works/4N6OqDO4/?utm_source=chatgpt.com) | Steel stock and steel-edged composite tools; differentiated smithing recipes. |
| **I4 — Controlled hardening and tempering** | Suitable steel; controlled heating/cooling; empirical heat-treatment skill | Early chronology uneven; developed within **1st-millennium BCE and later** ironworking traditions. Treat individual earliest claims cautiously. [Springer](https://link.springer.com/article/10.1007/s10814-019-09129-6?utm_source=chatgpt.com) | Hardened cutting edges and tools; additional failure modes from cracking or unsuitable steel. |
| **I5 — Reliable cast-iron production and casting** | Strongly carburizing furnace conditions; suitable lining/draft; molten-metal handling; molds | China, **1st millennium BCE**, established in Warring States production. [Don Wagner](https://www.donwagner.dk/cice/cice.html) | Cast vessels, implement blanks, molds, high-carbon feedstock; foundries. |
| **I6 — Malleabilizing and decarburizing cast iron** | I5 output; prolonged controlled heat treatment **OR** oxidizing refining | Chinese malleable-iron objects securely **4th–3rd centuries BCE**; later traditions elaborate the routes. [Don Wagner](https://www.donwagner.dk/cice/cice.html) | Tougher cast implements; iron/steel feed from high-carbon material; annealing/refining workshops. |
| **I7 — Crucible steel** | Refractory crucibles; sustained high heat; controlled charge; appropriate iron/carbon sources | Early South Indian dates around **3rd century BCE–3rd century CE** are debated; Sri Lankan production securely attested by **6th century CE**. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S030544032030145X) | Relatively homogeneous high-carbon ingots; premium steel production and specialist trade. |

**Graph correction:** neither bronze nor local copper smelting is a physical prerequisite for ironmaking. Prior metallurgical experience may raise discovery probability, but should be a soft advantage rather than an absolute requirement.

### 3.3 Glass, lime, and concrete

| Node | Required capabilities/resources | Approximate appearance or secure horizon | Concrete unlocks |
| --- | --- | --- | --- |
| **G1 — Primary glassmaking** | Silica source; suitable flux/stabilizer-bearing ingredients; furnace; heat-resistant containers; batch knowledge | Small glass objects in the **3rd millennium BCE**; developed vessel industries in Egypt/Mesopotamia by the **mid-2nd millennium BCE**. [The Metropolitan Museum of Art](https://www.metmuseum.org/de/essays/blown-glass-from-islamic-lands) | Raw glass, colored glass, ingots; primary glassworks. |
| **G2 — Glass forming and annealing** | Raw glass/cullet, locally made **OR imported**; reheating; forming and cooling skill | Established with early glass vessel industries, **2nd millennium BCE**. [The Metropolitan Museum of Art](https://www.metmuseum.org/pt/essays/roman-glass) | Beads, core-formed/cast vessels, decorative inlays; secondary workshops. |
| **G3 — Glassblowing** | Workable glass supply; blowpipe; furnace; specialized forming/annealing skills | Syro-Palestinian region, **1st century BCE**. [The Metropolitan Museum of Art](https://www.metmuseum.org/de/essays/blown-glass-from-islamic-lands) | Much higher vessel throughput and new forms; ordinary glassware markets. |
| **G4 — Architectural glazing** | Glass forming; pane production; frames and installation skills | Early Roman imperial contexts, **1st century CE**. [The Metropolitan Museum of Art](https://www.metmuseum.org/pt/essays/roman-glass) | Glazed windows and bathhouse glazing; not automatically clear optical glass. |
| **B1 — Lime burning** | Limestone or suitable carbonate source; sustained firing; fuel; charge management | Southwest Asian Pre-Pottery Neolithic; major Levantine plaster traditions by **9th–8th millennia BCE**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1002/gea.20241) | Quicklime; lime-burning pits/kilns; plaster and mortar supply chain. |
| **B2 — Slaked-lime plaster and mortar** | B1 output or imported quicklime; water; mixing; aggregate where appropriate; curing | Same early Neolithic horizon; uses and formulations diversify regionally. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1002/gea.20241) | Lime plaster, whitewash, masonry mortar, maintained floor/wall finishes. |
| **B3 — Hydraulic/pozzolanic binders** | Lime + reactive pozzolan **OR** suitable hydraulic-lime raw material/process; mixing and curing knowledge | Ancient Mediterranean traditions; well established in Roman construction by the late Republic/early Empire. [DOI](https://doi.org/10.1073/pnas.1417456111) | Water-resistant and underwater-setting mortar recipes; harbor and water infrastructure. |
| **B4 — Mass concrete construction** | Suitable binder; fine/coarse aggregate; placement; formwork/support; curing; structural design | Roman construction becomes a major application during the **last centuries BCE and early centuries CE**. [DOI](https://doi.org/10.1073/pnas.1417456111) | Rubble concrete walls, foundations, vaults, harbor masses; concrete-building work gangs. |

Hot mixing is best implemented initially as a **B2–B4 recipe variant**, not a mandatory node that all successful concrete must acquire.

### 3.4 Industrial and modern extensions

| Node | Principal prerequisites | Historical reference point | Unlock |
| --- | --- | --- | --- |
| **X1 — Coke-fired ironmaking** | Suitable coking coal; coke production; compatible furnace, ore, draft, and logistics | Darby’s Coalbrookdale process, **1709**, an important European industrial milestone—not the beginning of all coal use. [National Trust](https://www.ironbridge.org.uk/our-story/the-iron-bridge/) | Reduced dependence on charcoal supply; scalable pig-iron production. |
| **X2 — Bulk steel refining** | Large hot-metal supply; refractory vessels; controlled oxidation; chemistry-compatible process | Bessemer process, **1856**, followed by other refining routes. [Science Museum Blog](https://blog.sciencemuseum.org.uk/onward-ever-sir-henry-bessemer-frs-19-1-1813-15-3-1898/) | Much cheaper bulk steel; rail and structural production chains. |
| **X3 — Portland-clinker cement** | Controlled lime/silica/alumina-bearing feed; higher-temperature burning; clinker grinding; set control | Name/patent **1824**; modern-type commercial clinker cement developed in the **1840s**. [Cement Kilns](https://www.cementkilns.co.uk/cement.html) | More standardized hydraulic binder, cement mills, large concrete supply systems. |
| **X4 — Continuous float glass** | Continuous glassmaking; controlled molten-tin bath and atmosphere; annealing and process control | Pilkington publicly announced the process in **1959**; commercial reliability developed subsequently. [Pilkington](https://www.pilkington.com/en-us/united-states-and-canada/about-us/pilkington-history/invention-of-float-glass) | High-throughput flat glazing with much less subsequent surface finishing. |

Reinforced concrete should be another construction-system branch combining reliable binder, reinforcement, bond, cover, structural design, and workmanship—not merely “concrete + one iron bar.”

---

## 4. Variation across eras and regions

The labels below describe historical settings; they should not become simulation eras.

| Setting | Historically important variation | Consequence for TCE |
| --- | --- | --- |
| **Forager societies** | Ground-edge axes in Pleistocene Australia and pottery among late-Pleistocene Chinese foragers break the presumed farming-first sequence. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/03122417.2016.1164379?utm_source=chatgpt.com) | Seed independent skill packages. Subsistence mode does not determine a society’s entire material repertoire. |
| **Early farming Southwest Asia** | Lime plaster and substantial earthen construction existed before widespread pottery-container use. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0305440313003488) | Buildings, binders, and vessel technologies need partly separate branches. |
| **Balkans and Southwest Asia** | Early extractive copper, native-metal working, mixed-ore alloys, and later organized tin supply represent different technical steps. [DOI](https://doi.org/10.1007/s10963-021-09155-7?utm_source=chatgpt.com) | Avoid one “metallurgy” switch. Geography can support alloy production without a full local chain. |
| **Indus cities** | Mature Harappan bricks commonly followed a **4:2:1 length:width:height ratio**; fired bricks had important water-exposed uses. [Past Networks Gallery](https://networksofthepast.csmvs.in/object/harappan-brick/) | Standards reduce construction coordination costs. Fired brick need not replace cheaper earth everywhere. |
| **China** | High-fired ceramics and early cast-iron traditions followed trajectories unlike those of Europe. Annealed cast implements show that casting and toughness were not mutually exclusive. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440311001440) | Permit early foundry-centered economies and advanced ceramics without imposing European dates or prerequisites. |
| **South Asia and Sri Lanka** | Crucible steel and wind-powered direct steelmaking reveal multiple routes to demanding ferrous products. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S030544032030145X) | Wind, furnace orientation, refractory resources, and specialist practice can substitute for particular machinery. |
| **Sub-Saharan Africa** | Diverse iron technologies included significant local innovation. Ile-Ife also developed distinctive primary glass production; local glass recipes used particular geological and biological raw materials. [Cambridge University Press](https://www.cambridge.org/core/journals/cambridge-archaeological-journal/article/abs/invention-and-innovation-in-african-ironsmelting-technologies/BAAF3D221ED58DA1B0719E34B13934E8) | Do not portray African production as a single imported template. Local recipe adaptation should generate distinct industries. |
| **Andes and Mesoamerica** | Andean arsenical-alloy traditions favored particular working properties; Maya lime manufacture supported construction through both technological adaptation and decentralized organization. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1179/009346996791973774?utm_source=chatgpt.com) | Prestige, sheet-working, architecture, and household demand can drive innovation independently of iron weapons. |
| **Industrial production** | Coke ironmaking, bulk steel, and standardized clinker cement altered the scale and cost of supply, rather than simply introducing the idea of metal or concrete. [National Trust](https://www.ironbridge.org.uk/our-story/the-iron-bridge/) | The major transition is reliable throughput, equipment utilization, logistics, and capital intensity. |
| **Modern production** | Continuous processes and tighter controls can dominate costs and consistency; float glass is a clear example. [Pilkington](https://www.pilkington.com/en-us/united-states-and-canada/about-us/pilkington-history/invention-of-float-glass) | Add instrumentation, energy networks, chemical specifications, and continuous plant operation after the basic materials model works. |

---

## 5. Stylized facts a correct simulation should reproduce

### 5.1 Long coexistence, not clean replacement

New materials should first penetrate tasks where their benefits justify their costs. Other uses should persist. Experimental alloy comparisons and the social complexity of iron adoption reject a universal monotonic material ranking. **Validation target:** introducing iron should not instantly eliminate stone, copper alloys, wood, or composite tools. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1179/009346996791973774?utm_source=chatgpt.com)

### 5.2 Discovery can precede widespread adoption by centuries

Limited Anatolian iron production in the early second millennium BCE preceded the major late-second/early-first-millennium expansion. **Validation target:** the simulation must allow a known technology to remain rare because its supply chain, economics, or social uses are limited. [Springer](https://link.springer.com/article/10.1007/s10814-019-09129-6?utm_source=chatgpt.com)

### 5.3 Relatively small alloy inputs can support large trade networks

The Uluburun cargo contained approximately **10 tonnes of copper and one tonne of tin**. The importance is not a universal bronze recipe but the scale of complementary metal transport. **Validation target:** distant tin supply can sustain a large bronze-working region; disruption should trigger substitution, recycling, rationing, or contraction rather than immediate knowledge loss. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.2307/1357777)

### 5.4 Fuel economics can outweigh ore abundance

The charcoal conversion and experimental smelting figures show how metal production can require many times the final metal mass in wood. **Validation target:** an ore-rich settlement can remain a poor smelting location when fuel transport is expensive, while workshops move toward favorable combinations of ore, fuel, water, and customers. [ScienceDirect](https://www.sciencedirect.com/book/monograph/9780128042335/still-the-iron-age)

### 5.5 Standardization and decentralization can coexist

Harappan brick proportions demonstrate standardization; Maya kiln access demonstrates distributed production. Neither observation requires every product to come from one centralized factory. **Validation target:** institutions can coordinate dimensions and quality while households and independent workshops retain production. [Past Networks Gallery](https://networksofthepast.csmvs.in/object/harappan-brick/)

### 5.6 Forming innovations can change markets without changing the material

Glassblowing greatly expanded the range and economy of glass vessels. **Validation target:** a shaping technique can lower costs and expand demand even when primary glass chemistry changes little. [The Metropolitan Museum of Art](https://www.metmuseum.org/de/essays/blown-glass-from-islamic-lands)

### 5.7 Buildings have staged readiness

A mortar may be placed, support limited construction, and continue developing properties afterward. Roman mortar reconstructions demonstrate continuing material development during curing. **Validation target:** distinguish “placed,” “safe for next construction stage,” and “mature properties”; do not require workers to stand beside curing concrete continuously. [DOI](https://doi.org/10.1073/pnas.1417456111)

### 5.8 Loss of use is not necessarily loss of knowledge

Research on Indus urban decline documents associated changes in building technologies and their use, without establishing a single causal explanation. **Validation target:** falling urban demand can close kilns and eliminate practiced capacity even while some people retain the recipe. [arXiv](https://arxiv.org/abs/1303.1426?utm_source=chatgpt.com)

---

## 6. Modeling recommendation for individual agents and institutions

### 6.1 Make the workshop batch the computational unit

For a 10k–50k-person simulation, simulate **people’s work assignments individually**, but process the chemistry at batch level.

A batch should contain:

```
recipe_id
input_lots and measured quantities
equipment_id and condition
lead_worker and participating workers
current_stage
active_labor_completed
elapsed_process_time
fuel_consumed
temperature_exposure_class
atmosphere_class
expected_composition
quality_and_rejection_distribution
recoverable_byproducts
```

Stages should distinguish preparation, loading, active attendance, passive drying/cooling, extraction, finishing, and storage.

This gives visible daily life—wood hauling, clay preparation, bellows shifts, tapping, hammering, carrying bricks—without evaluating every particle or every worker’s chemistry every frame.

### 6.2 Keep knowledge in three places

Use **individual skill**, **workshop routines**, and **communicable records/designs**.

An experienced smelter may reproduce a process without understanding its chemistry. An apprentice may know the sequence but not diagnose failure. A drawing may preserve furnace dimensions while failing to preserve tacit judgments about slag or charge behavior.

For gameplay, let repeated production improve yield consistency and diagnosis. Experiments should alter recipes locally; successful variants can spread with workers, instruction, trade relationships, or copied equipment.

### 6.3 Use few material attributes, but preserve the important distinctions

For v1, retain:

| Family | Minimum useful attributes |
| --- | --- |
| Stone | Fracture suitability, toughness, abrasiveness, workability. |
| Copper alloys | Tin/arsenic class, contamination class, work-hardening state, casting quality. |
| Iron/steel | Carbon class, slag/inclusion level, harmful-impurity class, heat-treatment state. |
| Ceramics | Body type, porosity, firing maturity, thermal-shock behavior, glaze state. |
| Glass | Composition family, color/clarity, bubbles/inclusions, residual-stress quality. |
| Mortar/concrete | Binder family, aggregate compatibility, water ratio class, curing state, damage. |

Use lookup tables and response curves, not full thermodynamic solvers.

Avoid a single “quality = 87” property shared across all materials. A hard but brittle edge, porous cooking pot, and ductile sheet can each be appropriate products.

### 6.4 Institutions should change constraints, not grant arbitrary bonuses

Recommended mechanisms include kiln-sharing schedules, apprenticeship access, forest-use rights, credit for large batches, standard mold sizes, inspection, guaranteed purchasing, and allocation of rare metals.

A ruler’s building program can create sustained demand and finance a kiln. It should not directly grant “+20% ceramics.” A guild can preserve skill and standards while restricting entry. Shared forestry can stabilize charcoal supply without requiring private ownership.

### 6.5 Preserve recycling and maintenance

Maintain separate inventories of clean metal scrap, mixed scrap, slag with recoverable metal, broken glass, ceramic waste, reusable masonry, and worn tools.

Metal recycling should conserve constituent elements subject to specified losses, not regenerate pristine generic ingots. Repair and resharpening should compete economically with replacement.

Oxford’s FLAME project explicitly investigates ancient metal movement through recycling, remelting, and remixing; its framing is a better model for material circulation than a one-way ore-to-object chain. [Flame](https://flame.arch.ox.ac.uk/?utm_source=chatgpt.com)

### 6.6 Useful precedents—and their limits

**Vintage Story** provides a useful production-chain precedent: its steelmaking system separates refractory infrastructure, cementation, blister steel, and subsequent working. Borrow the visibility of intermediate stages, not its game-balanced times or material tiers as historical measurements. [Vintage Story Wiki](https://wiki.vintagestory.at/Steel)

**Factorio** provides a useful recipe-data precedent: named inputs, multiple outputs, process categories, and explicit crafting duration. Its documentation also illustrates why field names require care: `energy_required` is crafting time, not physical energy. TCE should separate time, labor, and joules or fuel mass. [Factorio Lua API](https://lua-api.factorio.com/latest/prototypes/RecipePrototype.html?utm_source=chatgpt.com)

**ODYM**, the Open Dynamic Material Systems framework, provides the strongest accounting precedent. It tracks stocks and flows across products, components, alloys, and chemical elements, with mass-balance checks and lifetime modeling. Borrow these concepts for debugging and validation rather than adopting its aggregate economic scenarios as agent behavior. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/jiec.12952?utm_source=chatgpt.com)

---

## 7. Sources, datasets, and uncertainty priorities

### Core scholarly reading

| Research need | Particularly useful sources |
| --- | --- |
| Early lithics and non-agricultural innovation | Harmand et al. **2015**, Lomekwi; Hiscock et al. **2016**, Australian ground-edge axes; Wu et al. **2012**, Xianrendong pottery. [Nature](https://www.nature.com/articles/nature14464) |
| Copper origins and alloying | Radivojević et al. **2010**, extractive metallurgy; **2013**, early mixed-ore tin bronze; Lechtman **1996**, experimental comparison of arsenic and tin bronzes. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440310001986) |
| Iron adoption and alternative pathways | Erb-Satullo **2019**, Near Eastern iron adoption; Wagner’s Chinese ferrous-metallurgy work; Killick **2015–2016**, African innovation; Juleff and collaborators on Sri Lankan wind-powered production. [Springer](https://link.springer.com/article/10.1007/s10814-019-09129-6?utm_source=chatgpt.com) |
| Measured production processes | Wrona **2013**, experimental steelmaking; Goren and Goring-Morris **2008**, lime burning; Flohr et al. **2015**, earthen construction; Seligson et al. **2017**, Maya kilns. [EXARC](https://exarc.net/issue-2013-2/ea/production-high-carbon-steel-directly-bloomery-process-theoretical-bases-and-metallographic-analyses) |
| Glass and advanced ceramics | Rehren and Pusch **2005**, primary glass production; Yin, Rehren, and Zheng **2011**, proto-porcelain; Zong et al. **2024**, early white porcelain. [PubMed](https://pubmed.ncbi.nlm.nih.gov/15961663/) |
| Roman binders | Jackson et al. **2014**, mortar development; Seymour et al. **2023**, hot mixing and lime clasts; Vaserman et al. **2025**, Pompeii production evidence. [DOI](https://doi.org/10.1073/pnas.1417456111) |

### Datasets worth using

**OXALID and GlobaLID** provide lead-isotope reference data for investigating ore sources and artifact provenance. OXALID is geographically uneven and has not been continuously updated; GlobaLID was designed to extend and modernize this infrastructure. Use these to inform plausible source diversity and trade patterns, not to assign every artifact an unambiguous mine of origin. [Oxalid](https://oxalid.arch.ox.ac.uk/?utm_source=chatgpt.com)

**USGS MRDS** provides records describing mineral occurrences, deposits, mines, commodities, and geology. It can inform procedural deposit families. Modern recorded deposits must not automatically be treated as exposed, discoverable, or economically workable with ancient technology. [USGS](https://www.usgs.gov/publications/mineral-resources-data-system-mrds?utm_source=chatgpt.com)

**Experimental reports and their supplementary tables** are the best source for batch-level calibration. Preserve the original ore, fuel moisture, furnace, crew, output stage, and losses with every imported coefficient. A single number stripped of those fields is often less useful than a wide, honestly labeled prior.

### Claims requiring special caution

**Earliest invention claims:** one metal object does not establish local extraction, one carbon-rich iron fragment does not establish controlled steelmaking, and one heated clay fragment does not establish a mature pottery industry. Reanalysis of the supposed early smelting evidence at Çatalhöyük interpreted it as copper minerals accidentally heated in a destructive fire. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440317301024)

**African iron chronologies:** very early dates and questions of independent invention remain debated; association, old wood, and whether a residue is genuinely metallurgical all matter. Do not resolve these disputes by coding one compulsory diffusion route. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/0305440388900362)

**Roman concrete:** durability varies by recipe, setting, construction, and exposure. Laboratory self-healing results and evidence of hot mixing do not justify a universal “Roman concrete lasts forever” modifier. [PubMed](https://pubmed.ncbi.nlm.nih.gov/36608117/)

**Labor costs:** these are the weakest cross-cultural parameters in this report. Reconstructed work-expenditure studies explicitly warn that experience, production organization, preservation, and omitted stages affect estimates. Keep the provisional coefficients editable and test outcomes across their full ranges. [DOI](https://doi.org/10.1515/opar-2022-0357?utm_source=chatgpt.com)

## Bottom line

For TCE’s first implementation, prioritize **material-specific inputs, batch production, realistic intermediate goods, separate active labor and elapsed time, charcoal’s upstream costs, alternative iron/steel routes, and local knowledge**.

Those mechanisms will produce more credible divergent histories than adding many narrowly named technologies to an otherwise linear tree. A society’s materials economy should emerge from what its people can repeatedly make, what their landscape supplies, what their institutions sustain, and what their customers are willing to pay for—not from the date on a technological calendar.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92855-ee6c-83ea-bee5-9372839f3f59)
