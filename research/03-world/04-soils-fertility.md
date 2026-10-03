# Soils, fertility dynamics, and agricultural land capability for TCE

## Executive recommendation

**Model soil fertility as several interacting stocks and constraints—not as a percentage that declines with cultivation and regenerates during fallow.** Nitrogen availability, phosphorus reserves, organic matter, rooting depth, waterlogging, and salinity have different causes and recovery times. A field can recover nitrogen supply while continuing to lose topsoil; contain abundant nutrients but remain unproductive because it is waterlogged; or sustain modest harvests for centuries without approaching zero yield. Long-term experiments and soil-process models demonstrate all three distinctions. [Rothamsted ERA](https://www.era.rothamsted.ac.uk/dataset/rbk1/03-OAWWYields)

For TCE, the most useful abstraction is:

> **Land capability determines the production opportunities; soil and water budgets determine what is currently possible; people’s labor, infrastructure, crop choices, and nutrient transfers determine what is realized.**

Use crop-specific productivity estimates for the interface, but preserve the underlying physical stocks in the kernel. This will produce meaningful differences between fertile but drought-prone plains, difficult wet clays, exhausted uplands, intensively maintained gardens, and irrigated land accumulating salt.

**Confidence notation:** **H** = well-established process or reliable measurement in its stated setting; **M** = useful quantitative range with substantial environmental dependence; **L** = reconstruction or weak transfer to other settings; **D** = proposed TCE design choice, not an empirical estimate.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Separate inherent capability from current condition

A soil’s agricultural value depends on its parent material, weathering history, texture, depth, organic matter, drainage, and climate. Soil classifications summarize combinations of these properties; they are not universal yield rankings. For example, a fertile clay may be harder to farm with limited drainage and traction than a less fertile, easily worked loam. [FAOHome](https://www.fao.org/4/y1899e/y1899e08a.htm)

| Soil or landscape archetype | Main agricultural advantages | Main constraints | TCE representation |
| --- | --- | --- | --- |
| **Deep steppe soils: Chernozems and related soils** | Deep, organic-rich surface horizons; substantial nutrient and rooting capacity | Seasonal drought can dominate; exposed cultivated surfaces can erode | Large rooting and organic-matter stocks, but no exemption from water limitation. [FAOHome](https://www.fao.org/4/y1899e/y1899e11.htm) |
| **Sandy soils: Arenosols** | Usually easy to cultivate and drain | Small water reserve; weak nutrient retention; leaching | Small water bucket and rapid drainage; low nutrient-buffering capacity. [FAOHome](https://www.fao.org/4/y1899e/y1899e06.htm) |
| **Strongly weathered tropical soils: many Ferralsols and Acrisols** | Often deep; some have favorable physical structure | Low weatherable nutrient reserves, acidity, strong phosphorus sorption | Low mineral replenishment; phosphorus limitation can persist despite nitrogen additions. [FAOHome](https://www.fao.org/4/y1899e/y1899e08a.htm) |
| **Volcanic-ash soils: Andosols** | Often favorable rooting, organic matter, and water storage | Strong phosphorus fixation; steep terrain in many locations | Good physical capacity but potentially high phosphorus retention. [FAOHome](https://www.fao.org/4/y1899e/y1899e06.htm) |
| **Shrink–swell clays: Vertisols** | Considerable water and nutrient storage | Sticky when wet, hard when dry, waterlogging, narrow cultivation windows | Strong seasonal workability constraints and high returns to appropriate surface drainage. [FAOHome](https://www.fao.org/4/y1899e/y1899e06.htm) |
| **Alluvial soils: Fluvisols** | Potentially deep; sediment can renew nutrients | Fertility varies with sediment source; flooding, waterlogging, or salinity | Receive explicitly routed sediment and nutrients, not an automatic “river fertility” bonus. [FAOHome](https://www.fao.org/4/y1899e/y1899e07.htm) |
| **Wet mineral soils: Gleysols** | Water availability; potential for suitable wetland crops | Oxygen shortage for ordinary upland crops | Crop-specific waterlogging response; possible drainage investment. [FAOHome](https://www.fao.org/4/y1899e/y1899e07.htm) |
| **Shallow or stony soils: Leptosols** | Sometimes chemically rich | Little rooting volume and drought buffering | Small accessible soil volume; potentially severe consequences from losing a few centimetres. [FAOHome](https://www.fao.org/4/y1899e/y1899e07.htm) |
| **Organic soils: Histosols** | Large organic stocks | Drainage can cause oxidation, subsidence, and eventual loss of the cultivated layer | Organic soil depth can disappear after drainage; high carbon is not automatically durable agricultural capital. [FAOHome](https://www.fao.org/4/y1899e/y1899e04.htm) |

These are **high-confidence qualitative mechanisms**, but assigning a fixed grain yield to each class would have low confidence.

### 1.2 Maintain a nutrient budget

For a field, including its soil and growing crop:

\[
\Delta N =
N\_{\mathrm{fixation}}+N\_{\mathrm{deposition}}+N\_{\mathrm{imports}}
-N\_{\mathrm{harvest}}-N\_{\mathrm{leaching}}
-N\_{\mathrm{gas}}-N\_{\mathrm{erosion}}.
\]

Imports include manure, fertilizer, seed, and nutrients carried by water or sediment. **Mineralization does not appear as a new input to total nitrogen:** it transfers nitrogen from organic matter into plant-available forms. Plant uptake similarly transfers nutrients from soil to biomass; it is not yet export from the field. [FAOHome](https://www.fao.org/4/a0443e/a0443e.pdf)

For phosphorus and potassium, maintain both accessible and reserve pools. Weathering or desorption transfers nutrients into accessible forms; harvest and erosion remove them. Unlike nitrogen fixation, there is no biological process that creates a new phosphorus or potassium supply from the atmosphere. [FAOHome](https://www.fao.org/4/a0443e/a0443e.pdf)

**Implementation rule:** allocate nutrients to the crop during growth. At harvest, export the nutrients in removed products and return those in retained residues. Do not subtract whole-crop uptake and then subtract grain export from soil a second time.

### 1.3 Treat restoration practices as different processes

**Manuring** transfers nutrients and organic material. At settlement scale, manure produced from local fodder mostly redistributes existing nutrients; it is not a free source of fertility. Imported fodder, grazing outside the cropped area, and recovered food waste can move nutrients into fields. Collection, storage, and application determine how much becomes available. [FAOHome](https://www.fao.org/4/a0443e/a0443e.pdf)

**Legumes** add atmospheric nitrogen through symbiosis, but fixation occurs in the growing plant. Harvesting nitrogen-rich grain can export much of the fixed nitrogen. Green manure and retained forage residues generally leave more behind than a heavily exported grain crop. Neither practice independently replaces exported phosphorus or potassium. [Roskilde Universitets forskningsportal](https://forskning.ruc.dk/da/publications/the-contributions-of-nitrogen-fixing-crop-legumes-to-the-producti/)

**Fallow is not one treatment.** Distinguish bare cultivated fallow, spontaneous vegetation, legume fallow, grazed fallow, and forest regrowth. They differ in water use, nitrogen fixation, erosion protection, residue inputs, and weed control. Broadbalk’s historical bare fallows were introduced primarily to control weeds, illustrating why a fallow benefit should not automatically be interpreted as nutrient replenishment. [Rothamsted ERA](https://www.era.rothamsted.ac.uk/dataset/rbk1/03-OAWWYields)

**Rotation** should influence nutrient demand, residue quality, rooting patterns, and crop-specific weed or disease pressure. A cereal–legume sequence should not behave like simply multiplying “fertility” by a constant.

### 1.4 Water and soil condition can override nutrient supply

Track:

\[
\Delta W=P+I+\text{capillary rise}-ET-\text{runoff}-\text{drainage}.
\]

A deep, nutrient-rich field can fail because roots lack water or oxygen. Drainage can improve upland cropping, while flooded rice needs a different response to inundation. Soil-water and crop models explicitly distinguish these processes. [FAOHome](https://www.fao.org/4/r4082e/r4082e03.htm)

For TCE, let slope affect runoff, erosion, cultivation difficulty, and infrastructure cost rather than directly prescribing yield. Slope is also not the same as steepness in degrees: a **15% slope is about 8.5°**.

---

## 2. Quantitative parameters and recovery dynamics

### 2.1 Physical capacity and nitrogen release

| Parameter | Quantitative starting point | Interpretation and confidence |
| --- | --- | --- |
| Plant-available water, sandy soil | **25–100 mm per metre of soil** | Broad FAO range; **M**. |
| Plant-available water, loam | **100–175 mm/m** | Broad range; **M**. |
| Plant-available water, clay | **175–250 mm/m** | Broad range; actual mineralogy and structure matter; **M**. |
| Annual mineralization of soil organic nitrogen | Approximately **1.5–3.5% of organic N/year** | Temperate agronomic guidance, strongly dependent on temperature, moisture, and aeration; **M locally, L as a global constant**. |
| Candidate rootable-depth scenarios | **0.2, 0.5, 1.0, and 1.5 m** | **D:** useful test cases, not universal soil-class assignments. |

Water ranges are from FAO’s soil-water guidance; mineralization guidance is from Cornell’s nutrient-management materials. [FAOHome](https://www.fao.org/4/r4082e/r4082e03.htm)

**Calculated example:** a loam with 150 mm/m available water and 0.8 m accessible rooting depth stores approximately **120 mm** of plant-available water before accounting for stones. The same texture over rock at 0.2 m stores only **30 mm**. This is why soil depth can matter more than a modest difference in nutrient concentration.

A second example shows why total nutrients and immediately available nutrients must be separate. Assume:

* A 0.20 m layer over one hectare.
* Bulk density of 1.3 t/m³.
* Organic carbon concentration of 1%.
* Organic-matter C:N ratio of 10.

The layer contains **2,600 t of soil, 26 t of organic carbon, and approximately 2.6 t of organic nitrogen**. Applying the conditional mineralization range above gives **39–91 kg N/ha/year**, before immobilization and losses. These are calculated scenario values, not a world-average soil.

A seemingly enormous total nitrogen stock can therefore support only a modest annual nitrogen flow. Conversely, mining organic matter can temporarily sustain harvests while degrading the long-term resource.

### 2.2 Nutrient removal by harvested products

Use **elemental N, P, and K** throughout the kernel. Many agricultural tables report phosphorus as P₂O₅ and potassium as K₂O:

\[
P=0.4364P\_2O\_5,\qquad K=0.8301K\_2O.
\]

The following values are converted from Ontario’s official crop-removal compilation. Grain values are per tonne at customary marketing moisture; straw values are per tonne of dry material. [Field Crop News](https://fieldcropnews.com/2025/05/crop-removal-values/)

| Removed product | N, kg/t | P, kg/t | K, kg/t | Confidence |
| --- | --- | --- | --- | --- |
| Winter wheat grain | **19–21** | **4.0–4.5** | **4.8–5.2** | H for reference composition; M for historical transfer |
| Maize grain | **11.5–17.7** | **2.9–3.4** | **3.8–4.3** | H/M |
| Soybean grain | **62–67** | **5.8–6.4** | **19.1–19.3** | H/M |
| Wheat straw | **4.4–9.6** | **0.4–2.1** | **10.0–19.3** | H/M |

**Calculated implication:** ten wheat harvests of 1 t/ha export approximately **200 kg N, 40–45 kg P, and 50 kg K per hectare in grain alone**. Removing 1.5 t/ha of dry straw adds roughly **15–29 kg K/ha per harvest**. Residue removal can therefore be particularly important for potassium.

These are **exports, not fertilizer recommendations or whole-plant requirements**. Roots and retained residues contain additional nutrients. Nor does dividing a total soil stock by annual export produce a reliable “years until exhaustion”: nutrient accessibility, replenishment, and declining yields change the trajectory.

### 2.3 Manure: material quantity is not immediately available nitrogen

The March 2026 AHDB guide gives these reference values for cattle farmyard manure at approximately 25% dry matter. [Project Blue](https://projectblue.blob.core.windows.net/media/Default/Publications/RB209/NutrientManagementGuideRB209S2_260310_WEB.pdf)

| Property | Per wet tonne | At 10 wet t/ha | Confidence |
| --- | --- | --- | --- |
| Total nitrogen | **6 kg N** | **60 kg N/ha** | H for guide value; M for actual manure |
| Total phosphorus | **1.40 kg P** | **14 kg P/ha** | H/M |
| Total potassium | **7.80 kg K** | **78 kg K/ha** | H/M |
| Nitrogen available to the next crop | Approximately **5–15% of total N** | **3–9 kg N/ha** | M; timing, storage, and incorporation dependent |
| Reference phosphate availability | **60%** | About **8.4 kg P/ha** | M |
| Reference potash availability | **90%** | About **70 kg K/ha** | M |

The unavailable nitrogen is not necessarily destroyed: much remains organic and may contribute later. These figures must not be transferred unchanged to urine, slurry, poultry manure, or manure from different diets.

**TCE implication:** applying 10 wet t/ha across 100 ha requires moving **1,000 tonnes of material**. Storage losses, cart capacity, distance, and application labor should make intensive manuring spatially concentrated unless institutions organize substantial transport.

### 2.4 Legumes: gross fixation versus net enrichment

Peoples and colleagues’ synthesis found an average relationship of approximately **30–40 kg of whole-plant fixed nitrogen per tonne of legume shoot dry matter**. This is a useful scaling relationship, not a guaranteed annual input. [Roskilde Universitets forskningsportal](https://forskning.ruc.dk/da/publications/the-contributions-of-nitrogen-fixing-crop-legumes-to-the-producti/)

| Quantity | Useful magnitude | Interpretation |
| --- | --- | --- |
| Gross fixation per tonne of shoot dry matter | **30–40 kg N/t** | **M**; depends on crop and environment |
| Example with 2–6 t/ha shoot dry matter | **60–240 kg fixed N/ha/crop** | Calculated range, conditional on that biomass |
| Net addition after harvest | **Gross fixation − exported N − subsequent losses** | Can be much smaller than gross fixation |

A crop fixing 100 kg N/ha and exporting 100 kg N/ha has added no net nitrogen to the field before other gains and losses are considered. Returning its residues still matters for timing and soil biology, but should not manufacture an additional 100 kg credit.

Use actual biomass, nodulation capability, water stress, and phosphorus availability to constrain fixation. Do not give every legume field a fixed “+fertility per year” effect.

### 2.5 Organic matter and fallow recovery

A useful carbon representation is:

\[
C\_i(t+\Delta t)=C\_i(t)e^{-k\_i f(T,W,\mathrm{cover})\Delta t}
+\text{inputs}+\text{transfers}-\text{erosion}.
\]

RothC supplies an established compartment-model pattern. Its standard base decay constants include **10 yr⁻¹** for decomposable plant material, **0.30 yr⁻¹** for resistant plant material, and **0.02 yr⁻¹** for humified organic matter. Environmental modifiers and transfers between pools alter actual behavior. These constants are model definitions, not universal observed recovery rates. [Soil](https://soil.copernicus.org/articles/8/199/2022/)

| Process or observation | Timescale or magnitude | What TCE should infer | Confidence |
| --- | --- | --- | --- |
| Humified-carbon base decay constant, RothC | **0.02 yr⁻¹**; unmodified half-life about **35 years** | Slow soil memory is necessary; this is not a 35-year fallow prescription | H as model definition |
| Brazilian slash-and-burn comparison | After **2 crop years**, **10–15 years** of fallow improved carbon and structure; approximately **15 years** restored several measured attributes relative to comparison land uses | Some tropical soils recover important functions over decades | M locally |
| Indian shifting-cultivation chronosequences | After **50 years** of fallow, Eastern Ghats SOC was about **60–86%** of forest reference; northeastern sites approached forest levels | Recovery can remain incomplete after decades and differ strongly by region | M locally; L for universal transfer |
| Soluble nutrient shortage | Can respond within a growing season when the missing nutrient is supplied | Fast nutrient relief must be separate from rebuilding organic stocks | H mechanism |
| Lost mineral soil depth | Often much slower to replace than nutrients or vegetation | Fallow should not automatically restore the former soil profile | H mechanism |

The fallow studies are site comparisons or chronosequences, not identical fields observed continuously through every recovery year. Their numerical timescales should be calibration cases, not universal rules. [SCI Journals](https://scijournals.onlinelibrary.wiley.com/doi/10.1002/jsfa.10123)

**Land-use consequence:** two cropped years followed by fifteen fallow years means only **2/17, or 11.8%, of the rotation area is cropped annually**. At an assumed 1.5 t/ha in cropped years, production averages only **0.176 t per hectare of total rotation land per year**, before seed and storage losses. Restored field yield and food production per unit territory are different metrics.

### 2.6 Erosion: convert tonnes into lost soil depth

Montgomery’s compilation provides useful orders of magnitude, but its category medians are **not area-weighted global averages**. [DOI](https://doi.org/10.1073%2Fpnas.0611508104)

| Category | Median rate | Sample count | Confidence |
| --- | --- | --- | --- |
| Conventional agriculture | **1.537 mm/year** | 448 | M as compiled magnitude |
| Conservation agriculture | **0.082 mm/year** | 47 | M |
| Native vegetation | **0.013 mm/year** | 65 | M |
| Soil production | **0.017 mm/year** | 188 | M; not equivalent to mature topsoil formation |

For bulk density \(\rho\_b\) in t/m³:

\[
\text{depth loss, mm}=
\frac{\text{soil loss, t/ha}}{10\rho\_b}.
\]

At the illustrative density of 1.3 t/m³, **13 t/ha equals 1 mm**. A sustained net loss of 1 mm/year removes **10 cm in a century**. At the compilation’s median soil-production rate, only about **1.7 mm** is produced in a century.

A practical starting point is the official RUSLE structure:

\[
A=R\,K\,LS\,C\_{\mathrm{cover}}\,P\_{\mathrm{practice}}.
\]

It links rainfall erosivity, soil erodibility, slope length and steepness, cover, and management. Use compatible units. RUSLE represents sheet-and-rill erosion; it does not automatically simulate gullies, tillage displacement, wind erosion, or terrace collapse. [eFOTG](https://efotg.sc.egov.usda.gov/references/Delete/2014-7-26/EroPred.html)

**Implementation recommendation:** remove soil mass and its associated carbon and nutrients, then route a fraction downslope as sediment. Distinguish erosion from a field from sediment actually leaving the catchment. Losses upstream can enrich lower fields while damaging canals or burying crops.

### 2.7 Salinization: a salt budget, not a fertility decay rate

Irrigation and groundwater can import salt. Evapotranspiration removes water while leaving most salts behind; drainage exports dissolved salt. Leaching without an outlet can raise the water table and fail to solve the underlying problem. [FAOHome](https://www.fao.org/4/t0234e/t0234e03.htm)

A convenient unit conversion is:

\[
\text{salt input, kg/ha}
=10\times I\_{\mathrm{mm}}\times c\_{\mathrm{g/L}}.
\]

**Calculated example:** 500 mm of irrigation containing 0.64 g/L dissolved salts imports **3,200 kg salt/ha**. This example specifies salt concentration directly; it does not assume a universal conversion between concentration and electrical conductivity.

For crop response, use a threshold–slope approximation:

\[
f\_{\mathrm{salt}}=
\operatorname{clamp}
\left[
1-\frac{b}{100}\max(0,EC-a),\,0,\,1
\right].
\]

| Crop | Threshold \(a\), dS/m | Yield-loss slope \(b\), percentage points per additional dS/m |
| --- | --- | --- |
| Barley grain | **8.0** | **5.0** |
| Bread wheat | **6.0** | **7.1** |
| Maize | **1.7** | **12.0** |
| Common bean | **1.0** | **19.0** |
| Paddy rice\* | **3.0** | **12.0** |

These are classic FAO guideline values: **M** for general crop comparisons, lower confidence for particular cultivars and extreme conditions. Most entries use electrical conductivity of the saturated soil extract, **ECe**. **The cited rice values instead refer to soil-water conductivity under flooded conditions.** Seedlings may be more sensitive. [FAOHome](https://www.fao.org/4/t0667e/t0667e07.htm)

For example, the approximation gives maize about **72%** of nonsaline yield at ECe 4 dS/m, while wheat remains below its listed threshold. This supplies an emergent reason to change crops.

Track **sodicity separately** when needed: sodium can damage soil structure and infiltration. A treatment for sodium-affected structure is not interchangeable with simply flushing soluble salts. [FAOHome](https://www.fao.org/4/t0234e/t0234e05.htm)

### 2.8 Terracing and drainage: benefits depend on the initial constraint

Terraces alter slope length, runoff, soil depth distribution, infiltration, and cultivable area. Global synthesis finds substantial conservation benefits, but poor design or abandonment can create hazards. A national Ethiopian study found that terraces buffered the 2015 drought despite slightly lower average yields across its broader comparison. There is no defensible universal “terrace = +X% grain” coefficient. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/53295)

| Evidence | Quantitative result | Correct interpretation |
| --- | --- | --- |
| Animal-drawn broad-bed drainage, Ethiopian Vertisols | Wheat yield **+25% at Debre Zeit** and **+131% at Wereilu** in 1986 comparisons | **M:** response depended strongly on waterlogging and location |
| Broad-bed construction in the same project | Approximately **30–36 human hours/ha** and **15–18 ox-pair hours/ha** | **M:** one field operation with two handlers, not total annual farming labor |
| Broad-bed geometry | Beds about **80 cm wide**, separated by **40 cm furrows** | A concrete drainage design, not a universal prescription |
| Stone-bund terraces, southwestern Ethiopia | Maize around **0.7 t/ha in upper positions versus 2.6 t/ha in lower positions** in one season | **M locally:** within-terrace variation, **not** the causal gain from installing terraces |

The drainage measurements come from on-farm verification; the terrace-position study sampled 27 terraces on six farms. [FAOHome](https://www.fao.org/4/x5493e/x5493e0p.htm)

For TCE, calculate construction from earth and stone volumes, transport, and labor. Benefits should follow the altered water and erosion processes. Add maintenance, blocked outlets, and breach risks. Draining organic or acid-sulfate soils should carry different long-term consequences from draining ordinary wet mineral soils. [FAOHome](https://www.fao.org/4/y1899e/y1899e04.htm)

---

## 3. Variation across eras and world regions

**Represent technologies and practices as combinable capabilities, not chronological fertility multipliers.** Early cultivation was not uniformly unmanured; pre-industrial agriculture was not uniformly extensive; modern high-input agriculture is not automatically sustainable.

| Setting | Evidence or quantitative anchor | Design significance |
| --- | --- | --- |
| **Foraging and managed wild landscapes** | Use vegetation production, harvest intensity, disturbance, and nutrient return rather than a universal cultivated yield | **D:** foragers can alter land condition without having a cereal-field production system |
| **Early farming, Near East** | Isotope- and grain-based reconstructions include wheat yields of roughly **0.5–1.2 t/ha** in the periods examined | **L:** useful order-of-magnitude checks, not directly measured regional averages. [Nature](https://www.nature.com/articles/ncomms4953) |
| **Neolithic Europe** | Crop-isotope evidence from **13 sites, approximately 5900–2400 BCE**, supports manure use | Manuring should be possible through animal management and labor, not locked behind a late “era.” [Nora](https://nora.nerc.ac.uk/id/eprint/506010/) |
| **Tropical shifting cultivation: Brazil and India** | Important recovery over **10–15 years** in one Brazilian setting; some Indian soil-carbon deficits persisted after **50 years** | Shortened fallow can produce pressure, but thresholds must vary by environment and practice. [SCI Journals](https://scijournals.onlinelibrary.wiley.com/doi/10.1002/jsfa.10123) |
| **Amazonian settlement landscapes** | Archaeological, soil, and ethnographic evidence supports intentional creation of enriched dark earth in the Upper Xingu | Waste handling, ash, charcoal, and repeated organic additions can produce persistent settlement-associated soil differences; charcoal alone does not create P or K. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11320335/) |
| **African highland farming with animal traction** | Large, location-dependent gains from relatively simple surface drainage | Labor organization and water management can be more immediately valuable than additional fertilizer. [FAOHome](https://www.fao.org/4/x5493e/x5493e0p.htm) |
| **Japan during industrialization** | Rice yield increased approximately **75% between 1880 and 1940** in the cited reconstruction | Changes combined varieties, fertilizer, cultivation, and water management—not a single soil improvement. [DOI](https://doi.org/10.1111/aehr.12223) |
| **Modern tropical irrigated rice** | IRRI reported approximately **17 t grain/ha/year across three crops** in its long-term intensive system | Annual output can rise through cropping intensity as well as yield per harvest; this is a high-input experimental benchmark. [Rice Today](https://ricetoday.irri.org/lessons-learned-in-the-long-term/) |
| **Modern temperate wheat** | Broadbalk’s record reached **13.8 t/ha in 2014**; unfertilized continuous wheat generally remained around **1 t/ha** | These are contrasting experimental outcomes, not normal global yields or pure historical analogues. [Rothamsted ERA](https://www.era.rothamsted.ac.uk/dataset/rbk1/03-OAWWYields) |

For later technologies, distinguish the ability to manufacture nitrogen fertilizer, acquire phosphorus-bearing materials, move bulky amendments, drain land, pump water, control weeds, and grow responsive cultivars. Each capability changes a different bottleneck.

One instructive tropical example is IRRI’s continuous-rice experiment: yields declined approximately **1.4–2.0% annually during 1968–1991**, then recovered with changed management and environmental conditions. That is evidence of a diagnosable, partly reversible production problem—not support for assigning all rice land a permanent 2% annual fertility loss. [Wiley Online Library](https://acsess.onlinelibrary.wiley.com/doi/abs/10.2134/agronj2000.924633x)

---

## 4. Stylized facts a correct simulation should reproduce

These are useful acceptance tests rather than rigid historical scripts.

1. **Continuous cropping need not converge to zero.** Depending on inputs, soil reserves, weathering, and losses, it can approach a low equilibrium. Broadbalk’s roughly 1 t/ha unfertilized benchmark is an important counterexample to inevitable linear exhaustion. Its deposition, liming, and weed-management history must remain part of the comparison. [Rothamsted ERA](https://www.era.rothamsted.ac.uk/dataset/rbk1/03-OAWWYields)
2. **Nutrient restoration can reveal a different limitation.** Supplying nitrogen should produce little response where phosphorus, water, or aeration is binding. In a Broadbalk isotope study, crop recovery of fertilizer nitrogen was approximately **51–68% with adequate P and K**, but around **40% under phosphorus deficiency**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-agricultural-science/article/abs/nitrogen-cycle-in-the-broadbalk-wheat-experiment-recovery-and-losses-of-15nlabelled-fertilizer-applied-in-spring-and-inputs-of-nitrogen-from-the-atmosphere/1B545B5B90458F1F4308463E8F6B951C)
3. **Exporting straw changes the long-term system.** A grain-only harvest and a grain-plus-straw harvest should diverge, particularly in potassium balance. Their livestock, bedding, fuel, and manure economies should also differ.
4. **Long fallow can restore field performance while reducing territorial output.** The calculated two-crop-year/fifteen-fallow-year example should occupy far more land per unit annual food than continuous cropping with adequate replenishment.
5. **Erosion creates delayed and spatially unequal damage.** Deep soils may conceal losses for years; shallow soils should reach rooting constraints sooner. Downslope deposition should create different outcomes from simply deleting sediment.
6. **Irrigation can improve yields initially and damage them later.** Salt loading without sufficient export should accumulate. Drainage and crop substitution should matter; fallow alone should not erase the salt stock.
7. **The same engineering works can help, do little, or harm.** Drainage should help waterlogged upland crops, offer little benefit on already dry ground, and potentially damage organic soils. Terraces should depend on siting and maintenance.
8. **Historical knowledge and land condition should persist.** Repeated organic additions can leave enriched settlement soils, while erosion and drainage-induced subsidence can outlast the institutions that caused them. Those legacies should not reset when ownership or government changes. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11320335/)

---

## 5. Recommended TCE implementation

### 5.1 Spatial state

Use **soil patches independent of individual agents and rendering resolution**. A proposed starting scale is **0.25–1 ha in relatively uniform terrain**, subdivided where terraces, wet depressions, alluvial deposits, or sharply different soils require it. This is a design proposal, not a demonstrated performance optimum.

A compact state could be:

| State group | Variables |
| --- | --- |
| Slowly changing physical properties | Texture, coarse fragments, rootable depth, bulk density, parent-material class, hydraulic parameters |
| Dynamic soil capital | Organic-carbon pools with coupled organic N; reserve and accessible P/K; acidity or buffering state; salt stock |
| Fast state | Layer water, mineral N, available nutrients, ponding, living crop biomass and nutrient contents |
| Management state | Residues, crop history, weed/disease pressure, terrace condition, drains, irrigation access |

Two soil layers are a reasonable first implementation, provided erosion can change the upper layer and crop roots can access the lower one. A dedicated flooded-soil mode is preferable to pretending that dryland carbon and nitrogen dynamics apply unchanged to paddy fields. RothC itself was designed for non-waterlogged topsoils; modified formulations have been tested for paddy systems. [Rothamsted Research](https://www.rothamsted.ac.uk/rothamsted-carbon-model-rothc)

### 5.2 Couple yield to resources without double counting

For a minimal seasonal prototype:

\[
Y=
\min\left(
Y\_{\mathrm{weather,water,cultivar}},
\frac{U\_N}{q\_N},
\frac{U\_P}{q\_P},
\frac{U\_K}{q\_K}
\right).
\]

Here, \(U\) is the nutrient uptake actually allocated to the crop over the season, and \(q\) is whole-crop nutrient requirement per tonne of harvested product.

This is a **proposed simplifying constraint**, not a validated crop model. In particular:

* Do not substitute grain-export coefficients for whole-plant requirements.
* Do not sum daily standing mineral-N stocks and call that seasonal supply.
* Do not apply the same drought or salinity penalty twice through different multipliers.

A stronger version grows canopy and biomass daily, then uses a harvest index to allocate grain. FAO AquaCrop provides a useful reference for the canopy–transpiration–biomass–yield structure. [FAOHome](https://www.fao.org/aquacrop/overview/calculation-scheme/)

### 5.3 Scheduling and performance

Suggested scheduling is daily water and crop growth; less frequent, climate-integrated slow-pool updates; and explicit events for manure application, harvest, irrigation, erosive storms, drainage failure, and terrace breaches.

Store fields in contiguous Rust arrays and aggregate labor contributions into completed operations. Fifty people weeding a field do not require fifty independent soil calculations. Visible activity should follow the same work completion and timing that determine agronomic outcomes.

No performance benchmark is claimed here: patch count, hydrological coupling, and the number of active crop processes will matter more than citizen count alone.

### 5.4 Agents, institutions, and land value

The following are recommended behaviors, not universal claims about historical societies.

**Households and farms** should choose crops, residue destinations, grazing, manure transport, and improvements using expected output, labor availability, risk, and imperfect local knowledge. Planting late because oxen are unavailable should differ from planting on nutrient-poor soil.

**Institutions** should be able to coordinate irrigation turns, drainage outlets, terrace maintenance, communal grazing, residue access, and waste recovery. These create meaningful upstream–downstream and livestock–cropping relationships.

**Land value** should emerge from expected surplus after labor, hauling, maintenance, risk, and obligations. A remote fertile parcel can be less valuable than a moderately fertile parcel near manure supplies and reliable irrigation. Ownership changes should transfer the parcel’s actual stocks and infrastructure, not regenerate it.

For the interface, show “expected wheat yield” alongside the principal constraints: **nitrogen, phosphorus, potassium, drought, waterlogging, salt, and soil depth**. Give agents imperfect estimates rather than direct access to hidden numerical stocks.

### 5.5 Existing models worth borrowing from

| Model | Most useful component for TCE | Main limitation |
| --- | --- | --- |
| **RothC** | Small carbon-pool model with climate and cover modifiers | Carbon model, not a complete NPK or crop system; original scope excludes waterlogged soils. [Rothamsted Research](https://www.rothamsted.ac.uk/rothamsted-carbon-model-rothc) |
| **AquaCrop** | Relatively compact daily crop-water and yield structure | Do not treat its fertility response as a complete nutrient-conservation model. [FAOHome](https://www.fao.org/aquacrop) |
| **APSIM** | Modular crops, soil processes, resource allocation, management, and waterlogging responses | Considerable complexity and calibration requirements. [APSIM Docs](https://docs.apsim.info/docs/development/software/interfaces) |
| **EPIC/APEX** | Erosion–productivity coupling; crop rotations, nutrients, irrigation, drainage, and landscape routing | Better as a reference and offline comparison system than an automatic per-agent dependency. [Blackland Research Center](https://blackland.tamu.edu/models/apex/) |

The highest-value implementation tests are mass conservation, nonnegative stocks, correct harvest accounting, and the absence of nutrient-creating manure loops. These should precede fitting historical yields.

---

## 6. Sources, calibration datasets, and uncertainty

### Priority datasets

| Source | What it contributes | Important limitation |
| --- | --- | --- |
| **SoilGrids 2.0 — Poggio et al., 2021** | Global **250 m** predictions at six standard depth intervals, including texture, SOC, total N, pH, bulk density, coarse fragments, and CEC, with uncertainty | Coarse relative to many fields; no direct substitute for locally calibrated available P/K. [DOI](https://doi.org/10.5194/soil-7-217-2021) |
| **FAO/IIASA GAEZ v4** | Suitability and yield estimates for **51 crops**, rainfed/irrigated conditions and contrasting management, at approximately **5 arc-minute** resolution | Regional plausibility checks, not parcel-level terrain generation. [FAOHome](https://www.fao.org/gaez/gaezv4/en) |
| **Rothamsted e-RA** | Long-term treatment, yield, and soil records; the cited wheat yield series spans **1852–2022** | A particularly valuable but geographically specific experimental system. [DOI](https://doi.org/10.23637/rbk1/meanWWYields1852-2022-03) |
| **IRRI long-term experiments** | Tropical flooded-rice response, cropping intensity, and management-related yield changes | High-input experimental conditions should not become ancient-rice defaults. [Wiley Online Library](https://acsess.onlinelibrary.wiley.com/doi/abs/10.2134/agronj2000.924633x) |
| **Hitotsubashi historical agricultural statistics** | Japanese prefectural crop records, including rice, cereals, millets, pulses, and potatoes, **1883–1940** | Historical units, product definitions, and reporting changes need harmonization. [D-Infra](https://d-infra.ier.hit-u.ac.jp/English/ltes/a000-asia-long-jp-crop.html) |

For mechanisms and initial coefficients, the strongest starting references are FAO’s *Plant Nutrition for Food Security*; FAO’s major-soil descriptions and salinity guidance; Peoples et al. on legume fixation; Montgomery on erosion rates; Rothamsted’s long-term experiments; and the site-specific fallow and engineering studies cited above.

### What remains uncertain

**Ancient yields are the weakest transferable numbers.** Reconstructions from crop remains, isotopes, or historical records are not interchangeable with measured harvests. Keep grain moisture, seed retention, harvested area versus total rotation area, and paddy versus processed rice explicit.

**Recovery times are not universal constants.** A 15-year fallow result in one environment and incomplete recovery after 50 years elsewhere are compatible. Parent material, previous disturbance, vegetation, erosion, and nutrient export differ.

**Engineering effects are conditional.** Selection of better or worse sites, construction disturbance, maintenance, and within-field redistribution complicate simple terrace comparisons.

**Present-day maps are not pristine starting conditions.** For procedural worlds, use datasets to constrain plausible property combinations, then initialize soils through a pre-settlement vegetation and climate equilibrium. Do not copy managed modern nutrient conditions directly into untouched early-agrarian landscapes.

**The central design priority is to preserve causality.** Let people replenish nitrogen while exhausting phosphorus, concentrate manure near settlements, trade short-term harvests against long-term soil depth, and build water infrastructure whose benefits depend on maintenance. Those mechanisms are sufficiently grounded to create plausible agricultural histories without scripting a sequence of exhaustion, reform, or collapse.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927ea-e854-83ea-a5ef-41e90adb3855)
