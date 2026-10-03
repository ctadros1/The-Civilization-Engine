# Ecology for The Civilization Engine

## Executive recommendation

**Model ecology as renewable—but depletable—stocks, not as fixed resource-production buildings.** For each landscape patch, distinguish:

1. **Standing stock:** trees, breeding animals, fish biomass, productive shrubs and roots.
2. **Renewal:** growth, reproduction, seasonal fruiting, migration and recruitment.
3. **Harvestability:** what people can find, capture, process and transport with their knowledge and tools.

These quantities are not interchangeable. A productive hunting expedition does not establish that the territory can sustain frequent hunting; rapidly returning forest biomass does not establish that construction timber has recovered.

The empirical literature supports useful numerical anchors, but **not a universal table of “forest = X kcal/hour.”** A cross-cultural analysis found average acquisition rates of approximately **729 kcal/person-hour for hunter-gatherers and 2,162 for horticulturalists**, with considerable variation and differences in how studies counted work. These are calibration checks, not biome constants or prehistoric technological baselines. [DOI](https://doi.org/10.1126/science.abf0130)

For TCE, I recommend **seasonal food patches, age-structured woodland, spatial wildlife populations, and fish stocks connected to hydrology**. Individual people perform the harvesting; institutions alter access, effort, management and distribution. Settlement carrying capacity then emerges from resource renewal, labor, storage and seasonal risk.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Vegetation productivity is not human food productivity

Track vegetation in functional groups: grasses, edible-seeded plants, tuberous plants, fruiting shrubs, nut-bearing trees, other canopy trees, and wetland plants. Each group needs habitat suitability, growth, reproductive maturity, seasonal production and disturbance responses.

Keep **productive vegetation** separate from its **currently edible crop**. Picking berries removes fruit; digging an entire perennial root may remove the producer. Harvesting nuts affects both present food and future regeneration, but regeneration should not decline proportionally with every nut collected.

A suitable daily food-stock equation is:

\[
F\_{d+1}=\max\left(0,F\_d+A\,y\_{\rm ref}s\_d f\_d-H\_d-C\_d-L\_d\right)
\]

Here, \(F\) is edible stock in kilograms; \(A\) is patch area in hectares; \(y\_{\rm ref}\) is reference annual production per hectare; \(s\_d\) allocates production seasonally; \(f\_d\) represents weather, condition and maturity; and \(H,C,L\) are human harvest, animal consumption and losses.

Do **not** renormalize the seasonal curve after frost or drought to guarantee the original annual harvest. Failed flowering must sometimes mean missing food.

Species respond differently to canopy and disturbance. Finnish observations found lingonberries productive in clear-cuts and old pine forests, whereas bilberries were concentrated in older, sparse forest. Thus “more trees” should not automatically mean “more berries.” [Silva Fennica](https://www.silvafennica.fi/article/5214?utm_source=chatgpt.com)

### 1.2 Foraging returns depend on the complete acquisition chain

Use this accounting definition:

\[
R\_{\rm gross}=
\frac{\text{usable food energy acquired}}
{\text{travel + search + pursuit + handling + transport + processing person-hours}}
\]

Count all participating workers’ time, unsuccessful attempts, and food eaten away from camp. Do not count a group hunt as one worker-hour per elapsed hour. Do not subtract activity calories again when they are already included in the person’s daily energy expenditure.

Out-of-camp consumption matters: direct Hadza observations found substantial consumption during excursions, so returning without food does not necessarily mean that a person acquired nothing. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S1090513816000118)

For agents, make expected returns depend on remembered patch abundance, travel cost, skill, equipment, season, crowding and danger. Separate **in-patch collection speed** from **door-to-door productivity**: a rich patch can be a poor choice when it is distant.

Hunting should generate many failures and occasional large returns. A two-component distribution—failure or small incidental return, versus a skewed successful return—is better than awarding every hunter a fraction of an animal each hour. Such mixture models have been fitted to Martu hunting observations. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3735252/?utm_source=chatgpt.com)

### 1.3 Wildlife renewal must precede hunting yield

For important prey, retain at least juveniles, reproductive females and other adults. Births depend on reproductive stock and condition; survival depends on food, weather, disease and predation. Hunting removes specific age/sex classes.

A simplified fallback is:

\[
\frac{dN}{dt}=rN\left(1-\frac{N}{K}\right)-H+I-E
\]

\(N\) is abundance, \(K\) habitat-supported abundance, \(r\) low-density net growth, \(H\) harvest, and \(I,E\) migration. If \(r\) already includes ordinary natural mortality, do not subtract the same mortality again through another subsystem.

Make \(K\) species-specific. Clearing woodland may reduce forest specialists while increasing some edge-feeding animals. Crops can become wildlife food, turning productive agriculture into a source of crop-raiding pressure. Food pulses and winter conditions can substantially change population growth, as demonstrated in wild-boar demographic models. [besjournals.onlinelibrary.wiley.com](https://besjournals.onlinelibrary.wiley.com/doi/10.1111/j.1365-2664.2005.01094.x?utm_source=chatgpt.com)

Hunting pressure should also change encounter conditions. Use an encounter rate such as

\[
\lambda=q\,D\,v
\]

where \(D\) is animal density, \(q\) is effective area searched per hour, and \(v\) captures detectability and wariness. Sample encounters, then pursuit and capture. This permits declining catches before local extinction without treating all missing animals as dead.

Maintain neighboring source populations. **An empty patch should recover through reproduction of survivors or immigration—not an unconditional respawn timer.**

### 1.4 Fish production follows water bodies and seasonal connectivity

Represent lakes, river reaches, floodplains and coastal grounds separately. Relevant drivers include water area, temperature, nutrient supply, oxygen, nursery habitat, connectivity and breeding stock.

Separate **production season** from **capture season**. Flooded habitat can support growth and recruitment, while receding water subsequently concentrates fish and raises catchability. A traditional lower-Amazon fishery study found its catches and catch rates peaking during low water. [WorldFish Digital Repository](https://digitalarchive.worldfishcenter.org/items/279ff6a3-1716-4d06-82f9-5f02605970ac)

Gear changes catchability, labor and size selectivity—not biological production directly. Nets and traps can catch while their owners perform other tasks, but construction, checking, repairs and processing remain labor costs.

For migratory fish, distinguish local habitat from externally produced arrivals. A river’s catch cannot be generated solely from the area of river visible inside the map.

### 1.5 Woodland has several recovery clocks

Track living woody biomass, tree age/diameter, reproductive trees, viable coppice stools/root systems, deadwood and species composition. Harvesting should distinguish fallen branches, poles, fuelwood and large straight timber.

Coppicing preserves a living root system, but success depends on species, stool condition and protection of shoots. Cutting followed by browsing, repeated fire or cultivation can prevent woodland recovery. Tanzanian miombo studies explicitly distinguish successive sprout, coppice and sapling stages. [DOI](https://doi.org/10.1155/2014/629317)

Deadwood gathering needs its own budget:

\[
\Delta W\_{\rm dead}
=\text{branchfall + mortality}
-\text{collection - decomposition - burning}
\]

It should not be an unlimited “forest resource.” Nor should all annual tree growth automatically become collectible fallen wood.

Human management can improve selected resources. Martu burning produced fine-grained habitat mosaics and higher small-game returns; Pacific Northwest clam gardens improved habitat and clam growth. These are reasons to model **specific ecological interventions**, not a universal conservation bonus. [DOI](https://doi.org/10.1073/PNAS.0804757105)

---

## 2. Parameters and quantitative calibration

### Reading the tables

**E** denotes empirical observations or published empirical syntheses; **D** denotes a calculation from reported quantities; **P** denotes a proposed TCE starting prior.

Confidence refers to the stated context. A strong local measurement may still transfer poorly to another species, technology or region. **P values are not confidence intervals or claimed historical measurements.**

### 2.1 Observed food-acquisition returns

All calorie figures below are food kilocalories, not kilojoules.

| Activity and context | Reported or derived return | Important denominator limitation | Evidence and confidence |
| --- | --- | --- | --- |
| Hunter-gatherer subsistence, 14-population comparison | **729 kcal/person-hour** mean | Work definitions vary across studies | **E; medium** across populations. Kraft et al. [DOI](https://doi.org/10.1126/science.abf0130) |
| Horticultural subsistence, 22-population comparison | **2,162 kcal/person-hour** mean | Contemporary technologies and mixed economies; not earliest farming | **E; medium**. Kraft et al. [DOI](https://doi.org/10.1126/science.abf0130) |
| Martu small-game hunting, Australian Western Desert | **478 without burning; 656 with burning kcal/person-hour** | Field bouts, not a complete residential provisioning budget | **E; high locally**, limited transfer. Bliege Bird et al. [doi.org](https://doi.org/10.1073/PNAS.0804757105?utm_source=chatgpt.com) |
| Martu monitor hunting, larger observational sample | **≈609 kcal/person-hour** | Derived from 1,392,587 kcal / 2,285 forager-hours | **D; medium**; contemporary field context. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3735252/?utm_source=chatgpt.com) |
| Martu kangaroo hunting, same larger study | **≈1,162 kcal/person-hour** | Derived from 469,385 kcal / 404 hours; highly variable outcomes | **D; medium**; not a stone-tool baseline. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3735252/?utm_source=chatgpt.com) |
| Experienced intertidal foraging, southern South Africa | **1,492 kcal/person-hour** overall | On-site experiments near low tide | **E; high for experiment**, medium transfer. [sciencedirect.com](https://www.sciencedirect.com/science/article/pii/S0047248416000221?utm_source=chatgpt.com) |
| Favorable conditions in the same intertidal study | **≈1,900–3,400 kcal/person-hour** projected | High-return opportunities only about **10 days/month**, **2–3 hours/day** | **E/model estimate; medium**. [sciencedirect.com](https://www.sciencedirect.com/science/article/pii/S0047248416000221?utm_source=chatgpt.com) |

The South African example is especially important: a high hourly return cannot be extended to eight hours every day.

### 2.2 Biome-level starting envelopes

The evidence does not justify precise, universally ranked biome means. The following are **low-confidence engineering priors for competent adults**, intended to be replaced by resource-specific calibration. They describe expected gross returns during an available-resource season, including routine travel and processing—not the range of individual trip outcomes.

| Ecological setting | Plant/shellfish gathering prior, kcal/person-hour | Hunting/fishing initialization | Principal modeled constraint |
| --- | --- | --- | --- |
| Arid scrub and desert mosaics | **200–800** | Small game **300–800**; calibrate larger prey separately | Patch spacing, water, rainfall pulses |
| Tropical seasonal woodland/savanna | **300–1,200** | Begin terrestrial hunting around **300–1,500**, with many failures | Dry-season access and seasonal fruit/root availability |
| Humid tropical forest | **300–1,500** | Use the same broad **300–1,500** hunting prior initially | Edible-species distribution, search, processing and prey depletion |
| Temperate woodland and mixed grassland | **300–1,500**; selected mast patches can exceed this | Same broad hunting prior, then fit species and habitat | Seasonal concentration and winter storage |
| Boreal/subarctic terrestrial landscape | **200–1,000** during productive windows; some winter plant activities unavailable | Fit hunting and fishing independently of plant returns | Short growing season, snow/ice, migration |
| Productive shore or wetland margin | **700–2,000** for suitable shellfish patches | Initialize fish capture in kg/person-hour, not a generic calorie bonus | Tides, nursery habitat, stock size, access |

These are deliberately overlapping. The empirical anchors above constrain their scale, but **the exact bounds are proposed**, especially for humid-forest and boreal settings. Do not apply an additional biome multiplier after already calibrating resource density, travel, season and processing; that would count the same disadvantage twice.

### 2.3 Annual wild-food production

Annual landscape production is needed independently of worker productivity.

| Resource and setting | Production anchor | Simulation interpretation | Confidence |
| --- | --- | --- | --- |
| Lingonberry, Central Finland, 1978–1981 | **8.0 kg/ha of forest area** | Landscape-average berry yield, not yield of a selected rich patch | **E; medium**, regional and time-limited. [Silva Fennica](https://www.silvafennica.fi/article/5214?utm_source=chatgpt.com) |
| Bilberry, same study | **4.3 kg/ha of forest area** | Keep separate habitat response from lingonberry | **E; medium**. [Silva Fennica](https://www.silvafennica.fi/article/5214?utm_source=chatgpt.com) |
| Sound acorns, Massachusetts oak stands, three study years | **30,000–155,000/ha** unthinned; **58,000–220,000/ha** thinned | Convert counts using species-specific sound-kernel mass; thinning can change production per tree | **E; medium**, short series. [OUP Academic](https://academic.oup.com/njaf/article-abstract/14/3/152/4788424?utm_source=chatgpt.com) |
| Southern Appalachian oak crops, 12-year study | **4 exceptionally good years; 3 very poor years** | Use multi-year variability rather than a constant annual nut crop | **E; medium**, specific stands/species. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/9237?utm_source=chatgpt.com) |

For berries and nuts, retain harvest losses, spoilage, inaccessible patches and wildlife consumption. **Production is not the amount humans actually eat.**

### 2.4 Fish yields and labor

| Fishery/context | Quantitative anchor | Correct use | Confidence |
| --- | --- | --- | --- |
| African floodplain fisheries | **40–60 kg whole fish/ha/year** estimated yields | Calibration of annual capture at the stated floodplain scale | **E/synthesis; medium-low** across systems. [OUP Academic](https://academic.oup.com/icesjms/article/68/8/1751/747359) |
| Asian floodplain fisheries | **≈120 kg whole fish/ha/year** estimated | Not automatically pristine production or prehistoric sustainable yield | **E/synthesis; medium-low**. [OUP Academic](https://academic.oup.com/icesjms/article/68/8/1751/747359) |
| Coastal lagoons, historical international compilation | **51 kg/ha/year median; 113 mean** | Strong skew; do not give every lagoon the mean | **E/synthesis; medium-low**. [FAOHome](https://www.fao.org/fishery/docs/CDrom/aquaculture/a0844t/docrep/009/T0377E/T0377E17.htm) |
| Xingu artisanal fisheries, Brazilian Amazon, 2012–2013 | **18 kg/fisher/day** mean | Contemporary equipment and fishing grounds; day length not standardized | **E; medium**. [DOI](https://doi.org/10.1590/1519-6984.00314bm) |
| Early-technology TCE fishing initialization | **0.2–2 kg whole fish/person-hour** over a complete trip | **Proposed prior**, varied by gear and abundance; larger seasonal runs separate | **P; low**, not a literature-wide measured interval |

Annual catch statistics cannot uniquely identify fish biomass, renewal and catchability. Avoid fitting all three independently to the same yield number.

**Illustrative calculation, not a historical estimate:** a 1 km² fishing ground yielding 50 kg/ha/year produces 5,000 kg whole fish annually. Assuming 50% usable flesh and 1,200 kcal/kg flesh gives **3 million kcal/year**—about **3.3 adult-equivalent annual energy budgets** at an assumed 2,500 kcal/day. That is much less than an indefinitely productive “fishing hut,” although fish can be a valuable dietary supplement and seasonal surplus.

### 2.5 Forest growth, recovery and wood collection

Do not interchange dry biomass, fresh mass, carbon mass and timber volume.

| Quantity/context | Value | What it represents | Confidence |
| --- | --- | --- | --- |
| Miombo, south-central African clear-felling study | **≈1 t dry biomass/ha/year**, equivalent to about **2.5 t fresh** | Local woodland growth | **E; medium locally**, low global transfer. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/0378112785900891) |
| Miombo regrowth, Kitulangalo, Tanzania | **≈2.3 m³/ha/year** | Wood-volume increment | **E; medium locally**. [Research Repository](https://repository.udsm.ac.tz/items/485233d0-9cff-43f0-965a-a2c27edc2fdb) |
| Charcoal-size recovery recommendation, same locality | **8–15-year cutting cycles** | Fuel-sized stock under specified management, not mature-forest recovery | **E/local management estimate; medium-low**. [Research Repository](https://repository.udsm.ac.tz/items/485233d0-9cff-43f0-965a-a2c27edc2fdb) |
| American sycamore coppice experiment | **3.23–5.09 t dry biomass/ha/year** | Experimental *Platanus* coppice, affected by spacing and cutting interval | **E; high locally**, not traditional woodland average. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/1917) |
| Neotropical secondary forest, 45 sites | At age 20: **122 t aboveground biomass/ha mean**, **20–225 range** | Accumulated biomass, not merchantable timber | **E; high for study population**. [Nature](https://www.nature.com/articles/nature16512) |
| Same secondary-forest synthesis | **≈6.1 t/ha/year** mean first-20-year accumulation; **66 years median** to 90% old-growth biomass | First number derived; neither implies full ecological recovery | **D/E; medium-high**. [Nature](https://www.nature.com/articles/nature16512) |
| Fuelwood collection, rural Malawi baseline | **5.7 kg/person-hour** | Reported collected firewood mass, including local collection conditions | **E/estimated baseline; medium-low**. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5930929/?utm_source=chatgpt.com) |
| Domestic firewood use, Nepal winter household study | **1.8 kg/person/day** average | Air-dried household fuel; small winter sample | **E; medium locally**, low historical transfer. [MDPI](https://www.mdpi.com/2411-9660/4/4/46?utm_source=chatgpt.com) |

For a first temperate coppice implementation, **2–5 t dry woody biomass/ha/year** and **8–25-year rotations** are reasonable **proposed testing ranges**, not universal historical facts. Fit actual harvest age to the requested product: thin rods, poles and large beams need different diameter distributions.

Modern intensive poplar experiments/modeling suggest substantially higher production, around **5–15 t/ha/year** under intensive culture. That belongs to a different management regime, not an automatic upgrade to all natural forests. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/18840)

### 2.6 Wildlife demography and depletion

| Parameter or response | Anchor | TCE implication | Confidence |
| --- | --- | --- | --- |
| Wild-boar annual population multiplier under poor/intermediate/good conditions | **λ = 0.85 / 1.09 / 1.63** | A bad year can shrink populations without hunting; food pulses alter recruitment | **Published demographic model; medium-high**, conditional on input rates. [besjournals.onlinelibrary.wiley.com](https://besjournals.onlinelibrary.wiley.com/doi/10.1111/j.1365-2664.2005.01094.x?utm_source=chatgpt.com) |
| Reintroduced African elephant population | **≈7.1% annual growth**, approximately ten-year doubling | Even favorable slow-breeder recovery takes decades after severe depletion | **E; high locally**, not an all-elephant baseline. [besjournals](https://besjournals.onlinelibrary.wiley.com/doi/10.1111/1365-2664.13199) |
| Tropical hunting synthesis, 176 studies | Mammal abundance **83% lower**, birds **58% lower**, in hunted settings | Heavy hunting can leave vegetation standing while fauna collapses | **E/meta-analysis; medium-high**, heterogeneous contexts. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28408600/) |
| Spatial hunting effect in that synthesis | Depletion detected up to **40 km for mammals**, **7 km for birds**, from access points | Roads, settlements and markets can create large depletion zones | **E; medium**; not universal kill radii. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28408600/) |

For a logistic implementation, remember that \(r=\ln(\lambda)\): λ = 1.63 corresponds to \(r≈0.49\,{\rm year}^{-1}\), not 1.63.

**Do not implement “harvest 20% of every animal population sustainably.”** In the elementary logistic model, maximum sustainable yield is \(rK/4\), attained at \(N=K/2\). Real age structure, environmental variation and uncertainty make that an optimistic diagnostic, not a safe universal policy.

For scale, suppose a **hypothetical** 100 km² territory has \(K=500\) deer-like animals and \(r=0.3\). Its logistic maximum yield is only **37.5 animals/year**. At an assumed 30 kg usable meat and 1,500 kcal/kg, that is **1.69 million kcal/year**—about 1.9 complete adult-equivalent energy budgets, or a 20% contribution for roughly nine people.

A large carcass is an impressive meal, but annual replacement can be small.

### 2.7 Carrying capacity: bands and early villages

Observed population density is not identical to ecological carrying capacity. A 300-population hunter-gatherer analysis links density to productivity, biodiversity and pathogen environment, but leaves substantial unexplained variation; its terrestrial productivity approach also does not directly capture marine subsidies. Use the dataset for calibration, not as a deterministic biome population cap. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5819417/)

These are **proposed order-of-magnitude validation envelopes**, not empirical biome confidence intervals:

| Subsistence landscape | Initial density envelope | Land equivalent for 50 people | Status |
| --- | --- | --- | --- |
| Sparse arid or strongly seasonal terrestrial resources | **0.01–0.1 persons/km²** | **500–5,000 km²** | **P; low confidence** |
| Productive mixed terrestrial foraging | **0.1–1 person/km²** | **50–500 km²** | **P; low confidence** |
| Exceptional aquatic concentration plus storage | **1–10 persons/km²** of the defined resource territory | **5–50 km²** | **P; low confidence**; aquatic contributing area must be explicit |
| Early farming | Calculate from crop, fallow, pasture and woodland areas | See below | Prefer resource accounting to a fixed density |

For farming, a cereal-energy-equivalent calculation is:

\[
A\_{\rm arable/person}
=
\frac{365e}
{y\,c\,u\,f}
\]

where \(e\) is daily energy need, \(y\) harvested grain yield, \(c\) energy per kilogram, \(u\) the usable share after seed and losses, and \(f\) the fraction of arable land cropped annually.

A Neolithic-style French experiment reported average yields of approximately **900 kg/ha for emmer and 1,350 kg/ha for einkorn** under its low-input conditions. Such experiments constrain possibilities but are not measurements of prehistoric village averages. [Préhistoire](https://www.prehistoire.org/shop_515-49840-5505-800/version-papier-bspf2022-2.html)

**Illustrative 200-person village:** assume 2,500 kcal/person/day, 800 kg grain/ha, 3,500 kcal/kg, 70% usable yield and half the arable area cropped annually. The result is approximately **186 ha of arable land**, including rotation/fallow, for its cereal-equivalent energy requirement.

Add an assumed 0.6 t dry fuelwood/person/year and woodland yielding 3 t/ha/year with 75% usable recovery: about **53 additional hectares**. This still excludes pasture, buildings, unusable terrain, dietary diversity, construction wood, industrial fuel and harvest-season labor constraints.

**Map-scale consequence:** on a 16 × 16 km map, 0.1 forager/km² supports only about **26 people**. A realistic early band may need access beyond the rendered map. Later populations of 10,000–50,000 must be supported by demonstrated agricultural intensification, substantial aquatic inputs, imports or a larger regional economy—not faster wild-food respawning.

---

## 3. Variation across technologies and world regions

Treat the following as **mechanical changes that can appear in different combinations**, not mandatory eras.

| Subsistence/technology configuration | Recommended ecological changes in TCE |
| --- | --- |
| Foraging bands | Local ecological knowledge, seasonal movement, sharing, preservation, selective burning and management of favored patches. Do not require an untouched wilderness. |
| Early farming villages | Permanent harvest pressure, crop–wildlife competition, retained useful trees, fallow succession, livestock browsing and greater storage dependence. Wild foods remain part of a mixed economy. |
| Pre-industrial specialization | Coppice rotations, managed fishing grounds, transport animals, boats, charcoal production and markets redistribute pressure. Separate domestic demand from urban and industrial demand. |
| Industrial extraction | Increase search/capture capacity, transport reach, processing throughput and external energy inputs separately. Habitat loss and pollution need their own effects. |
| Modern intensive management/restoration | Permit planted forests, aquaculture, stocking, protected areas and monitored harvest—but charge the necessary feed, land, nutrients, labor and enforcement. Recovery remains resource-specific. |

The regional evidence argues against one European succession pathway:

**Australia and southern Africa:** Martu fire mosaics show productive management within a hunting economy; South African intertidal experiments show that productive coasts still have tight temporal access constraints. [DOI](https://doi.org/10.1073/PNAS.0804757105)

**Pacific Northwest:** British Columbia clam gardens contained roughly **four times as many butter clams and over twice as many littleneck clams** as comparison beaches; transplanted juvenile littlenecks grew **1.7 times faster**. The mechanism was altered intertidal habitat—not domesticated crop technology applied to the sea. [DOI](https://doi.org/10.1371%2Fjournal.pone.0091235)

**Amazonia:** useful-tree distributions show persistent associations with human land-use history. The extent attributable specifically to pre-Columbian rather than later management is contested. Model enrichment planting and settlement legacies without assuming that every tropical forest was either pristine or comprehensively engineered. [Princeton University](https://collaborate.princeton.edu/en/publications/persistent-effects-of-pre-columbian-plant-domestication-on-amazon/?utm_source=chatgpt.com)

**East and Southeast Asian wet agriculture:** rice–fish systems demonstrate that farming and aquatic production need not occupy mutually exclusive tiles. Experiments in southern China found complementary resource use and pest-control interactions between rice and fish. In TCE, represent those interactions rather than adding an unconditional combined-yield bonus. [CiNii](https://cir.nii.ac.jp/crid/1362825896377048960?utm_source=chatgpt.com)

**African seasonal woodlands:** coppice and charcoal-sized regrowth can be important on relatively short rotations, while mature structure takes longer. **Northern forests:** different berry species respond differently to cutting and stand age, so timber-oriented management can change the composition of wild food rather than simply raising or lowering all of it. [Research Repository](https://repository.udsm.ac.tz/items/485233d0-9cff-43f0-965a-a2c27edc2fdb)

---

## 4. Stylized facts the simulation should reproduce

These are validation patterns, not scripted outcomes.

| Pattern | Quantitative or directional target |
| --- | --- |
| High labor productivity can coexist with low carrying capacity | A hunting strategy can return around 1,000 kcal/hour yet become unsustainable when repeatedly concentrated on a small territory. Validate labor returns and replacement budgets separately. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3735252/?utm_source=chatgpt.com) |
| Abundant seasonal food does not guarantee year-round settlement | Reproduce high intertidal returns restricted to approximately **20–30 high-return hours/month** in the cited South African setting. [sciencedirect.com](https://www.sciencedirect.com/science/article/pii/S0047248416000221?utm_source=chatgpt.com) |
| Management can improve selected wild resources | Permit the observed direction and approximate scale of fire-related small-game improvement or clam-garden enhancement, without improving unrelated resources. [DOI](https://doi.org/10.1073/PNAS.0804757105) |
| Food pulses affect wildlife recruitment | A good mast/mild-winter sequence should produce very different boar trajectories from repeated poor years; the published λ values span **0.85–1.63**. [besjournals.onlinelibrary.wiley.com](https://besjournals.onlinelibrary.wiley.com/doi/10.1111/j.1365-2664.2005.01094.x?utm_source=chatgpt.com) |
| Heavy hunting creates spatial depletion | Increasing access should permit large abundance reductions and expanding depletion zones, not just uniformly slower respawn. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28408600/) |
| Vegetation greening precedes full forest recovery | Secondary stands can accumulate substantial biomass within 20 years while taking many further decades to approach old-growth biomass. [Nature](https://www.nature.com/articles/nature16512) |
| Human pressure changes resource composition | Cutting can favor one berry resource and disadvantage another; timber, understory food and wildlife habitat should not move in lockstep. [Silva Fennica](https://silvafennica.fi/article/10573/ref/41?utm_source=chatgpt.com) |

Also impose engineering invariants: no negative stocks, no harvested animal counted twice, no fish caught after local stock exhaustion, and no food created by changing simulation speed.

---

## 5. Recommended TCE representation

### 5.1 Use individual people over aggregate ecological populations

A workable starting architecture is:

| Layer | Suggested representation | Update cadence |
| --- | --- | --- |
| Vegetation and wild foods | **125–250 m cells**, fractional habitat cover, productive plant pools, seasonal edible stocks | Daily food/weather updates; monthly structural growth |
| Woodland | Species or functional-group age/diameter cohorts, coppice viability, deadwood | Monthly growth; annual aging/recruitment; immediate harvest |
| Terrestrial wildlife | Populations on approximately **0.5–2 km patches**, with age/sex classes and movement links | Seasonal births; daily-to-monthly survival/movement as needed |
| Fish | Water-body/reach populations with size classes and connectivity | Daily catch/access; seasonal recruitment and movement |
| People | Explicit trips, labor, tools, skills, carried food, processing and sharing | Normal agent scheduler |
| Institutions | Access rules, rotations, quotas, monitoring, penalties and shared infrastructure | Event-driven decisions and periodic review |

These are proposed resolutions, not performance benchmarks. A 250 m ecology grid on a 16 km square map contains only **4,096 cells**. Detailed terrain can remain at rendering/pathfinding resolution.

Visible animals should be **representatives of conserved populations**. Creating a visible herd must reserve corresponding animals from the aggregate stock; killing them must debit that stock. Changing camera position must not generate additional prey.

The same applies to visible trees: their removal must alter the associated cohort, while cohort growth controls which replacement trees can exist.

### 5.2 Preserve different ecological clocks

Do not accelerate tree maturity merely to keep the landscape visually busy. Use different observable changes: fresh shoots, pole stands, thinning canopy, expanding clearings, deadwood scarcity and widening travel distances.

For fast-forward, aggregate the same processes rather than switching to flat daily yields. Retain hunting failure distributions and shared weather shocks. Independent daily random noise for every patch would artificially smooth regional drought and mast failure.

Process resource claims through a conservation-preserving commit stage. Multiple agents can plan from the same stock snapshot, but the combined accepted harvest must not exceed availability.

### 5.3 Give agents imperfect ecological information

Agents should remember recent catches, travel time, visible damage, successful seasons and advice from others—not inspect true hidden carrying capacity.

Choose activities using expected household benefit, reliability, storage capacity, nutritional needs, social obligations and risk. An agent who maximizes calories per personal hour alone will mishandle group hunts, sharing and provisioning.

Institutions should operate through concrete mechanisms:

* **Access and allocation:** territories, commons membership, exclusive fishing sites and sharing rules.
* **Investment and restraint:** coppice rotations, protected breeding areas, seasonal closures, burning, planting and processing infrastructure.
* **Compliance:** monitoring effort, legitimacy, penalties and opportunities to evade rules.

A rule is not effective merely because it exists. Model who bears its immediate cost and who receives the later benefit.

### 5.4 Existing models and games worth borrowing from

| Model/game | Borrow | Do not import uncritically |
| --- | --- | --- |
| **LANDIS-II** | Spatial species-age cohorts, succession, dispersal and disturbance modules. Its cohort representation is directly relevant to TCE forests. [US Forest Service R&D](https://research.fs.usda.gov/nrs/products/dataandtools/landis-landscape-disturbance-and-succession-model) | Full regional-model complexity or its default temporal resolution |
| **Ecopath with Ecosim** | Biomass accounting and explicit production, consumption, harvest and mortality budgets. [Pressbooks B.C. Campus](https://pressbooks.bccampus.ca/eweguide/chapter/ecopath-input/) | The assumption that observed catch alone identifies all ecological parameters |
| **Martu hunting return/utility models** | Failure-plus-success distributions and decisions based on the distribution of returns, not only the mean. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3735252/?utm_source=chatgpt.com) | Particular preferences or contemporary equipment as universal human defaults |
| **Eco** | Making ecological consequences visible through stock data, graphs, maps and laws tied to harvesting behavior. [Eco](https://www.play.eco/) | Its compressed recovery times or game balance as ecological measurements |

### 5.5 Validation procedure

Test ecology without people first, then introduce controlled harvesting policies, then autonomous agents.

For each climate–habitat combination, run replicated multi-decade scenarios with no harvest, low harvest, heavy harvest, abandonment and restoration. Examine resource trends, extinction frequency, recovery time, labor returns, household food deficits and travel distance.

Estimate carrying capacity as **the largest population that maintains renewable stocks while meeting a chosen food-security standard under the tested climate and institutions**. This is more useful than computing one timeless “people per biome” number.

---

## 6. Source priorities, datasets and evidence limits

### Best reusable calibration resources

| Resource | Recommended use |
| --- | --- |
| **Kraft et al., 2021, Science — “The energetics of uniquely human subsistence strategies”** | Cross-cultural acquisition rates and consistent distinctions between energy acquired, work time and energetic cost. [DOI](https://doi.org/10.1126/science.abf0130) |
| **Martu hunting data**, Dryad DOI **10.5061/dryad.g1h6b** | Fit outcome distributions, not just averages; preserve task and technology context. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3735252/?utm_source=chatgpt.com) |
| **Tallavaara, Eronen & Luoto data**, Zenodo DOI **10.5281/zenodo.1167852** | Hunter-gatherer density comparisons and environmental covariates; includes population data and analysis materials. [Zenodo](https://zenodo.org/records/1167852?utm_source=chatgpt.com) |
| **NASA MOD17A3HGF Version 6.1** | Annual terrestrial productivity at 500 m resolution for modern analogues. It measures GPP/NPP, not edible food or pristine historical conditions. [NASA Open Data Portal](https://data.nasa.gov/dataset/modis-terra-net-primary-production-gap-filled-yearly-l4-global-500m-sin-grid-v061-7fb95?utm_source=chatgpt.com) |
| **PanTHERIA**, Jones et al., DOI **10.1890/08-1494.1** | Mammal life-history and ecological traits for species-specific initialization. [The Ecological Society of America](https://esajournals.onlinelibrary.wiley.com/doi/abs/10.1890/08-1494.1) |
| **COMADRE**, Salguero-Gómez et al., DOI **10.1111/1365-2656.12482** | Population projection matrices and their environmental/study context; preferable to one reproduction constant per animal size class. [besjournals](https://besjournals.onlinelibrary.wiley.com/doi/10.1111/1365-2656.12482?utm_source=chatgpt.com) |
| **Forest growth experiments, secondary-forest chronosequences, and national berry inventories** | Independently fit timber/biomass recovery and non-timber food production; do not substitute one for another. [Nature](https://www.nature.com/articles/nature16512) |

### Important uncertainties

**The weakest numbers are universal biome-level hourly returns and prehistoric landscape carrying capacities.** Studies differ in technology, study duration, resource access, processing accounting and whether exceptional seasons are sampled. The biome envelopes in this report are therefore explicitly proposed priors, not disguised measurements.

**Forest biomass recovery is better quantified than recovery of every ecosystem function.** The Neotropical results concern aboveground biomass; they should not be interpreted as equivalent recovery of large timber, species composition or all wildlife habitat. [Nature](https://www.nature.com/articles/nature16512)

**Modern “traditional” economies are not untouched prehistoric controls.** Some measured returns incorporate contemporary tools, transport, trade or altered resource distributions. Cross-cultural subsistence comparisons remain valuable, but technological context must accompany calibration. [DOI](https://doi.org/10.1126/science.abf0130)

**Historical human landscape modification is real but unevenly reconstructed.** In Amazonia, even the timing and relative contribution of different management periods remain debated. This supports persistent land-use memory in TCE, not a single predetermined human effect on all forests. [Princeton University](https://collaborate.princeton.edu/en/publications/persistent-effects-of-pre-columbian-plant-domestication-on-amazon/?utm_source=chatgpt.com)

The central design choice is to make **labor efficiency, renewable production and accumulated ecological capital independent quantities**. That allows TCE to produce prosperous seasonal camps, sustainable managed woodlands, depleted hunting territories, fish-dependent settlements and overextended farming villages from the same rules—without scripting any of those outcomes.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927ef-cdc0-83ea-8856-cf91ed90a2a2)
