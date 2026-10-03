# Building materials: production, properties, and durability

## Executive conclusion

**TCE should model materials as production chains and buildings as maintained assemblies—not as a ladder from “primitive, temporary materials” to “advanced, permanent materials.”** Dry, protected timber can remain serviceable for centuries; earthen buildings can be multistory and long-lived; fired brick and stone can deteriorate through moisture, salts, incompatible repairs, and fire. Material identity alone does not determine building life. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62262.pdf)

The most important distinctions are:

* **Production work versus elapsed time:** seasoning timber, drying adobe, and cooling kilns occupy space and capital without requiring continuous labor.
* **Material versus component versus assembly:** a clay tile, the roof covering, and the timber roof structure have different failure processes.
* **Appearance versus performance:** discoloration, surface erosion, leakage, biological decay, and structural weakening should not share one condition value.
* **Maintenance versus replacement:** replacing a roof can protect an old wall without restoring strength already lost.

The report uses **H** for well-established physics or measurements under specified conditions; **M** for useful but context-dependent evidence; **L** for weak transferability; and **P** for an explicitly proposed TCE calibration parameter, not a historical measurement. Mechanical values below are **simulation anchors, not allowable engineering design strengths**.

---

## 1. Mechanisms: production and material choice

### 1.1 Represent distinct production chains

| Material | Production sequence | Implementable consequences |
| --- | --- | --- |
| **Timber** | Select trees → fell and limb → extract logs → split, hew, or saw → sort → season where appropriate → cut joints | Separate poles, structural timbers, boards, shingles, and fuelwood. A stockpile of short logs cannot satisfy a long-beam requirement. Seasoning ties up inventory; large framing members need not be fully dry before use. [FAOHome](https://www.fao.org/4/s1250e/S1250E03.htm) |
| **Thatch** | Harvest suitable stems/leaves → dry → sort → bundle or weave → transport → attach in overlapping layers | Crop or wetland management, harvest timing, storage, material selection, roof geometry, and thatching skill all affect performance. “Thatch” should contain several resource families, not one universal grass item. [FAOHome](https://www.fao.org/4/s1250e/S1250E05.htm) |
| **Rammed earth** | Obtain suitable subsoil → adjust grading/moisture → erect forms → place and compact successive layers → move forms | Compaction quality and moisture affect the wall produced. Formwork is reusable capital; wall production happens on site. [FAOHome](https://www.fao.org/4/s1250e/S1250E06.htm) |
| **Cob and wattle-and-daub** | Mix earth, water, and sometimes fiber → build wet lifts, or apply to woven infill → dry → finish | Cob is a mass-wall technique; wattle-and-daub normally depends on a structural frame. They must not inherit identical load paths merely because both contain earth. [FAOHome](https://www.fao.org/4/s1250e/S1250E06.htm) |
| **Adobe** | Prepare earth → mix → mold blocks → initial drying → turn/stack → further drying → sort | Blocks become transportable inventory, but drying needs weather, protected ground, and time. Rain can destroy work in progress before construction. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-05-adobe.pdf) |
| **Fired brick** | Prepare clay → mold/extrude → dry → load → fire → cool → unload and grade | Model drying and kiln capacity separately. Firing produces a distribution of quality, not automatically identical units. Inadequate firing, rapid temperature changes, and handling can create weak or damaged bricks. [FAOHome](https://www.fao.org/4/s1250e/S1250E07.htm) |
| **Stone** | Locate usable deposit → remove overburden → collect or detach blocks → rough-size → transport → dress exposed faces and joints | Geological bedding, natural fractures, block size, and required finish determine work. Rubble, roughly squared stone, and finely fitted ashlar require different recipes. [FAOHome](https://www.fao.org/4/s1250e/S1250E05.htm) |
| **Lime** | Obtain limestone/shell → break and sort → calcine → cool → slake where required → mix mortar/plaster → cure | Keep limestone, quicklime, hydrated lime, and mortar separate. Slaking is exothermic; premature wetting changes the stored product. Air-lime and hydraulic binders need different curing rules. [Practical Action Publishing](https://practicalactionpublishing.com/book/3101/download?type=download) |
| **Clay tiles** | Prepare clay → form thin shaped units → dry → fire → sort → install with appropriate overlap/support | Tile manufacture resembles brick manufacture but needs its own breakage, shape-control, and roof-coverage parameters. Durable tiles do not make the whole roofing assembly maintenance-free. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-30-clay-tile-roofs.pdf) |

### 1.2 Let production quality persist into building life

Use a **batch-quality record**. For timber, record species, heartwood fraction, moisture, defects, and dimensions. For earth, record grading, shrinkage behavior, compaction, and fiber recipe. For ceramics, record firing adequacy, cracks, and pore characteristics.

This is more defensible than a single “craftsmanship bonus.” Historic-brick micro-CT research shows that pore geometry and connectivity vary substantially between products; total porosity alone does not capture their moisture-related behavior. [Nature](https://www.nature.com/articles/s40494-022-00723-4)

**Proposed rule:** a skilled producer improves both average quality and consistency. A buyer can observe some defects immediately, while hidden defects become apparent only after loading, weather exposure, or freezing.

### 1.3 Make geometry part of material choice

Roofing material should constrain pitch, overlap, support spacing, and dead load. Wall materials should constrain thickness, openings, slenderness, and jointing. In tropical construction guidance, grass thatch requires much steeper slopes than some tile or sheet-metal systems; these are assembly-specific recommendations, not universal geometric constants. [FAOHome](https://www.fao.org/4/s1250e/S1250E0l.htm)

For TCE, choosing a covering should therefore regenerate the roof’s quantities and structural demand. It should not merely change a texture and fire-resistance statistic.

### 1.4 Make cost emerge from the complete chain

A useful accounting identity is:

\[
C\_{\text{installed}}
=C\_{\text{resource}}
+C\_{\text{processing}}
+C\_{\text{fuel}}
+C\_{\text{transport}}
+C\_{\text{installation}}
+C\_{\text{waste}}.
\]

This is a proposed accounting structure, not a fixed historical cost-share formula. Add inventory financing and seasonal delays where relevant.

A stone deposit beside a settlement may supply cheap rubble but expensive dressed blocks. Earth can avoid kiln fuel while requiring bulky handling and protective details. A fuel-efficient kiln can still be uneconomic at insufficient demand: the documented Chenkumbi lime project in Malawi encountered quality and market problems despite technical improvements. [Practical Action Publishing](https://practicalactionpublishing.com/book/2966/download?type=download)

---

## 2. Parameters: measured anchors and provisional defaults

### 2.1 Mechanical properties: preserve test conditions

#### Timber

These USDA values describe **small, clear specimens at 12% moisture content**, not knotty full-size beams. Approximate densities are calculated from the handbook’s specific gravities and moisture basis. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62244.pdf)

| Species | Approx. density at 12% moisture, kg/m³ | Bending rupture, MPa | Compression parallel to grain, MPa | Elastic modulus, GPa | Confidence |
| --- | --- | --- | --- | --- | --- |
| Western redcedar | 360 | 51.7 | 31.4 | 7.7 | H for specified specimens |
| Coast Douglas-fir | 540 | 85.0 | 49.9 | 13.4 | H for specified specimens |
| White oak | 760 | 105 | 51.3 | 12.3 | H for specified specimens |

**Implementation:** treat these as material-level anchors, then apply separate member-grade, joint, moisture, and geometry effects. Wood’s directionality matters: compression parallel to grain is not interchangeable with compression across grain. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62244.pdf)

Natural durability needs a separate axis. White-oak and cedar heartwood are substantially more decay-resistant than red-oak or spruce heartwood; sapwood generally lacks the same protection. “Hardwood” is not a synonym for decay-resistant wood. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62262.pdf)

#### Earth, brick, and stone

The following are **specific measured examples**, deliberately not presented as worldwide material ranges.

| Material and sample context | Density, kg/m³ | Compressive strength, MPa | Interpretation | Confidence/source |
| --- | --- | --- | --- | --- |
| Historic rammed earth, Leiria, Portugal | Group means **1,360–1,790** | Group means **0.44–0.74** | Existing, weathered material; not optimized new rammed earth | M; Parracha et al. [UNL Research](https://research.unl.pt/ws/files/17145015/Parracha_et_al_Leiria_vernacular_earth_buildings_material_charact_LJAH19_manuscript.pdf) |
| Historic adobe, same investigation | Group means **1,400–1,730** | Tested group means **0.69–1.06** | Considerable within-group variation | M; same study. [UNL Research](https://research.unl.pt/ws/files/17145015/Parracha_et_al_Leiria_vernacular_earth_buildings_material_charact_LJAH19_manuscript.pdf) |
| Historic brick, Gnojewo church, Poland | Not established here | **5.34** tested mean; **4.54** normalized | Six samples; elastic modulus approximately **2.10 GPa** | M; Gołębiowski et al., 2015. [Yadda](https://yadda.icm.edu.pl/baztech/element/bwmeta1.element.baztech-59a5ffc3-6e23-4a26-90a6-17ed4b04a6ff/c/golebiewski_lubowiecka_kujawa_ceer_2015_18.pdf) |
| Lioz limestone, Portuguese test series | Approximately **2,702** | **113 ± 2** | Dense stone specimen, not masonry-wall strength | M; Pires et al., 2024. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11382203/) |
| Grey Ançã limestone, same series | Approximately **2,370** | **150 ± 8** | Lower density did not mean lower strength in this comparison | M; same study. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11382203/) |

These examples illustrate why TCE should not assign strength from density alone. They also should not be used to infer that all adobe is stronger than rammed earth, or that all limestone exceeds 100 MPa.

For masonry, **unit strength is not wall strength**. Mortar, bond, construction arrangement, deterioration, and load direction intervene. The Gnojewo study explicitly separates brick properties from homogenized wall properties. [Yadda](https://yadda.icm.edu.pl/baztech/element/bwmeta1.element.baztech-59a5ffc3-6e23-4a26-90a6-17ed4b04a6ff/c/golebiewski_lubowiecka_kujawa_ceer_2015_18.pdf)

### 2.2 Processing time, labor, and yield

**Person-hours measure work; calendar days measure elapsed time.** For conversion inside TCE, an eight-hour person-day can be a bookkeeping convention, but it is not a universal historical working day.

| Process | Quantitative anchor | Boundary and interpretation | Confidence/source |
| --- | --- | --- | --- |
| Air-seasoning **25 mm boards** to roughly **20–25% moisture** | About **15–30 days** under favorable conditions; **200–300 days** for slower species/unfavorable conditions | Species, season, climate, and stacking strongly affect time | M; USDA drying chapter. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62261.pdf) |
| Large framing timbers | Members over approximately **76 mm** thick are often impractical to dry fully before use | Permit green framing with shrinkage-aware construction | M; USDA. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62261.pdf) |
| Adobe drying | Approximately **one month or more** | Drying, not a month of continuous labor | M; NPS/FAO. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-05-adobe.pdf) |
| Adobe rejects | Approximately **10%** in the FAO production example | Cracked, broken, or misshapen blocks; not a universal constant | M/L outside context. [FAOHome](https://www.fao.org/4/s1250e/S1250E06.htm) |
| Hand-brick molding | **15–25 person-hours per 1,000 bricks** | Process-accounting estimate; excludes other stages | M/L; brick-kiln report appendix. [cdn.catf.us](https://cdn.catf.us/wp-content/uploads/2019/10/21093607/Brick_Kilns_Performance_Assessment.pdf) |
| Digging + soil preparation + molding | **19–33 person-hours per 1,000 bricks**, derived subtotal | Excludes transport, stacking, firing, unloading, fuel production, and losses | M/L; same appendix. [cdn.catf.us](https://cdn.catf.us/wp-content/uploads/2019/10/21093607/Brick_Kilns_Performance_Assessment.pdf) |
| Green-brick drying, small-scale example | Initial **3 days**, then stacked for **at least another week** | Protected from rain; local guidance rather than a global recipe | M; FAO. [FAOHome](https://www.fao.org/4/s1250e/S1250E07.htm) |
| Small wood-fired brick kiln | **4–5 days firing**, including roughly **6 hours at 875–900°C** | Specific kiln recipe; cooling is additional | M; FAO. [FAOHome](https://www.fao.org/4/s1250e/S1250E07.htm) |
| Thatch harvesting, Shirakawa-go | Experienced harvester: **60–100 bundles/person-day**; average roof: approximately **10,000 bundles** | Implies **100–167 person-days of harvesting alone** | M; official village documentation. [vill.shirakawa.lg.jp](https://www.vill.shirakawa.lg.jp/2686.htm) |
| Hydrated lime, Chenkumbi plant | **15.7 person-shifts/t product** | Derived from 1,570 shifts/100 t; includes quarrying but also mechanized processing | M locally; L as ancient analogue. [Practical Action Publishing](https://practicalactionpublishing.com/book/2966/download?type=download) |

**Critical conversion rule:** “per 1,000 bricks” is not a stable cross-cultural unit. Store block dimensions and finished volume. Never apply the same labor total to both small modern bricks and much larger adobe blocks.

#### Where historical labor evidence remains thin

Portable quarrying, hand-hewing, stone-dressing, and earthen-wall production rates are much less secure than the temperature and moisture data. Protzen’s Inca experiments establish the feasibility of quarrying and dressing with hammerstones, but they do not justify one worldwide cubic-meter production rate. [University of California Press](https://online.ucpress.edu/jsah/article/44/2/161/57823/Inca-Quarrying-and-Stonecutting)

For an initial playable build, the following are **authored sensitivity-test envelopes only**:

| Operation | Proposed test envelope | Explicit exclusions |
| --- | --- | --- |
| Rough-hewing usable timber from accessible logs | **8–40 person-hours/m³** | Felling, extraction, seasoning, boards, and joinery |
| Mixing, forming, and manually compacting rammed-earth wall | **8–32 person-hours/m³** | Long-distance soil transport and difficult site access |
| Extracting and rough-sizing small blocks from an easily worked, naturally jointed quarry | **8–80 person-hours/m³ accepted stone** | Hard-rock megablocks, haulage, and fine finishing |
| Additional stone-face dressing | **4–40 person-hours/m² face** | Highly elaborate carving and exceptionally close fitting |

**All four rows are P, low confidence—not historical estimates.** They should remain exposed in configuration and be replaced by technique-specific experimental or account-book evidence. Large-block extraction and fine ashlar can fall outside these envelopes.

### 2.3 Fuel and firing

A 2012 field assessment of nine kilns in India and Vietnam provides useful **measured thermal-energy anchors**, not ancient universal rates. [cdn.catf.us](https://cdn.catf.us/wp-content/uploads/2019/10/21093607/Brick_Kilns_Performance_Assessment.pdf)

| Kiln configuration | Thermal input, MJ/kg fired brick | Qualification |
| --- | --- | --- |
| Fixed-chimney Bull’s trench | **1.22** | Mean of three kilns |
| Zigzag | **1.12** | Mean of two |
| Indian vertical-shaft | **0.95** | One kiln |
| Improved Vietnamese vertical-shaft | **0.54** | One kiln |
| Downdraught batch | **2.90** | One kiln |
| Vietnamese tunnel kiln | **1.47** | Includes connected dryer |

These figures exclude some upstream energy and are not controlled comparisons of identical clay and products. Use them to distinguish kiln configurations; do not encode “tunnel kiln” as automatically the most fuel-efficient option. [cdn.catf.us](https://cdn.catf.us/wp-content/uploads/2019/10/21093607/Brick_Kilns_Performance_Assessment.pdf)

| Lime parameter | Value | Interpretation | Confidence/source |
| --- | --- | --- | --- |
| Practical calcination temperature | Approximately **900–1,100°C** | Temperature, residence time, stone size, and gas conditions interact | H/M; FAO. [FAOHome](https://www.fao.org/4/s1250e/S1250E07.htm) |
| Pure limestone decomposition | **1 t CaCO₃ → 0.560 t CaO + 0.440 t CO₂** | Stoichiometric calculation; actual commercial yield depends on purity and processing | H; lime chemistry. [Practical Action Publishing](https://practicalactionpublishing.com/book/3101/download?type=download) |
| Quicklime hydration | **1 kg CaO + 0.321 kg water → 1.321 kg Ca(OH)₂** | Minimum chemical water; practical slaking can use more | H; calculated from chemistry. [Practical Action Publishing](https://practicalactionpublishing.com/book/3101/download?type=download) |
| Traditional Sri Lankan coral-lime firing | Approximately **0.5 kg wood/coconut-trunk fuel per kg coral feed** | Denominator is raw feed, **not quicklime output** | M; Practical Action case study. [Practical Action Publishing](https://practicalactionpublishing.com/book/2965/download?type=download) |
| Improved Malawi hydrated-lime plant | Approximately **6 MJ/kg commercial hydrated lime** | Derived from charcoal input; excludes charcoal manufacture and diesel | M; Practical Action case study. [Practical Action Publishing](https://practicalactionpublishing.com/book/2966/download?type=download) |

For resource accounting:

\[
m\_{\text{fuel}}=\frac{m\_{\text{product}}\;E\_{\text{specific}}}
{\text{usable fuel heating value}}.
\]

Keep fuel moisture and energy basis consistent. Charcoal must carry its upstream wood consumption through the charcoal-production recipe. Unfired earth and air-seasoned timber avoid a firing stage; they are not labor-, transport-, or energy-free.

### 2.4 Durability and maintenance intervals

**The following intervals describe renewal or maintenance under particular conditions—not automatic failure ages.**

| Material/component | Evidence-based anchor | TCE interpretation | Confidence/source |
| --- | --- | --- | --- |
| Protected timber structure | Can remain serviceable for **centuries** | No mandatory expiration while dry, protected, and structurally adequate | H for possibility; not a mean lifespan. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62262.pdf) |
| Wood moisture and decay | Maintaining **below about 20% moisture** is a useful protective margin; substantial decay generally requires wetter conditions | Use a smooth moisture response, not an instant threshold | H/M; USDA. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62262.pdf) |
| Grass-thatch roof, tropical guidance | Significant maintenance about **every 2–3 years**; well-maintained covering can last **20–30 years or more** | Frequent patching can coexist with much longer complete-renewal intervals | M; FAO. [FAOHome](https://www.fao.org/4/s1250e/S1250E0l.htm) |
| Japanese thatched roofs, Shirakawa-go | Renewal typically **20–30 years** | Regional assembly-level anchor | M. [vill.shirakawa.lg.jp](https://www.vill.shirakawa.lg.jp/2686.htm) |
| Water reed, traditional English estimate | **50–60 years** | A historical quoted expectation, not a universal biological life | M locally; L globally. [Thatch Advice Centre](https://www.thatchadvicecentre.co.uk/wp-content/uploads/2014/11/english-heritage-thatch-and-thatching.compressed.pdf) |
| Combed wheat reed, same tradition | **25–40 years** | Same qualification | M/L. [Thatch Advice Centre](https://www.thatchadvicecentre.co.uk/wp-content/uploads/2014/11/english-heritage-thatch-and-thatching.compressed.pdf) |
| Long straw, same tradition | **10–20 years** | Same qualification | M/L. [Thatch Advice Centre](https://www.thatchadvicecentre.co.uk/wp-content/uploads/2014/11/english-heritage-thatch-and-thatching.compressed.pdf) |
| Clay roof tiles | Often approximately **100 years**, sometimes several centuries | Tile survival and complete-roof serviceability must remain separate | M; NPS. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-30-clay-tile-roofs.pdf) |
| Adobe/earthen walls | No defensible universal expiration | Condition depends heavily on water exclusion, surface protection, and repair | M; NPS. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-05-adobe.pdf) |
| Mortar joints and lime finishes | Condition-dependent sacrificial renewal | Inspect and repair locally; do not replace all masonry on a fixed schedule | H/M; NPS. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-02-repointing.pdf) |

English Heritage explicitly warns against treating the traditional thatch rankings as universal. Geography, roof design, workmanship, and maintenance confound comparisons; examples of all three materials have exceeded fifty years. Claims attributing premature straw deterioration to agricultural fertilizer are not established well enough to become a deterministic TCE rule. [Thatch Advice Centre](https://www.thatchadvicecentre.co.uk/wp-content/uploads/2014/11/english-heritage-thatch-and-thatching.compressed.pdf)

### 2.5 Fire: combustible material is not the same as a weak fire assembly

For large timber members under a standard fire exposure, the USDA handbook reports charring around **0.8 mm/min initially**, later around **0.6 mm/min**, with approximately **0.6 mm/min over the first hour**. This is a useful reference case, not a universal wildfire or house-fire rate. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62268.pdf)

A simplified TCE timber-fire calculation can remove char depth from exposed faces:

\[
d\_{\text{char}}=\beta t.
\]

Remaining geometry then determines capacity, with additional penalties for heated wood and damaged joints. A 200 mm square member exposed on four faces for sixty minutes at 0.6 mm/min would retain a nominal 128 mm square uncharred section. Its area is about 41% of the original, and its geometric bending section modulus about 26%. **That calculation is an illustration, not a fire-resistance rating.** [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62268.pdf)

| Material/assembly | Fire behavior to represent |
| --- | --- |
| Thatch, shingles, thin wooden elements | Combustible exposed surfaces; geometry and ember exposure matter. Do not give them the same response as a massive beam. |
| Massive timber | Combustible, but charring can delay loss of the entire section. |
| Adobe/rammed earth | Mineral body is noncombustible; timber floors, roofs, and attachments may still burn. |
| Brick, stone, lime | Noncombustibility does not prevent thermal cracking, loss of strength, joint damage, or collapse. |
| Clay-tile roof on timber framing | Covering and supporting structure need independent fire states. |

As one bounded example, a limestone experiment involving **24-hour heating to 600°C followed by water cooling** produced compressive-strength reductions of roughly **23–33%** in two stones. That demonstrates vulnerability, but its unusually prolonged test exposure must not become a universal “stone fire damage” multiplier. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11382203/)

No reliable universal annual ignition probabilities by historical roofing material emerged from this evidence. Model ignition sources, exposure, weather, settlement spacing, and response capacity rather than inventing a material-only fire frequency.

---

## 3. Variation across technological settings and world regions

### 3.1 Use capabilities, not era gates

The requested eras are useful comparison labels, but TCE should implement the following as independent combinations.

| Setting | Material-system emphasis | Simulation implication |
| --- | --- | --- |
| **Forager and mixed subsistence economies** | Mobility, access to large timber, local vegetation, and collective labor can produce very different housing investments | Do not make substantial timber buildings depend on agriculture. Northwest Coast plankhouse traditions are an important counterexample to equating nonindustrial Indigenous construction with flimsy shelters. [Academia](https://www.academia.edu/4924043/_We_Honor_the_House_Lived_Heritage_Memory_and_Ambiguity_at_the_Cathlapotle_Plankhouse) |
| **Early farming** | Settlement investment can support earthen walls, stored roofing materials, and specialized pyrotechnology | Lime production must not require a universal pottery-era gate: burnt-lime floors and walls are documented in Pre-Pottery Neolithic communities of Southwest Asia. [Persee](https://www.persee.fr/doc/paleo_0153-9345_1987_num_13_1_4417) |
| **Preindustrial craft economies** | Coexisting earth, timber, thatch, fired ceramics, and stone systems; quality depends on craft, geology, fuel, and transport | Unlock specific processes and organization. Sophisticated stonework does not universally require iron tools, as the Inca experimental evidence shows. [University of California Press](https://online.ucpress.edu/jsah/article/44/2/161/57823/Inca-Quarrying-and-Stonecutting) |
| **Industrial production** | Greater mechanization, throughput, controlled drying, and larger supply areas become possible | Improvements should change labor requirements, consistency, capacity, and market radius—not simply add strength to every material. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-30-clay-tile-roofs.pdf) |
| **Modern production and conservation** | Controlled moisture, grading, engineered products, testing, and specialized repair coexist with traditional materials | Retain exposure-dependent deterioration and compatibility. A modern, harder repair mortar can damage older soft masonry. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62261.pdf) |

### 3.2 Regional examples that should influence parameterization

| Region/example | Observed pattern | What TCE should learn |
| --- | --- | --- |
| **Indus cities** | Third-millennium-BCE Moenjodaro contains extensive fired-brick construction; salt and moisture are major conservation problems | Early availability of brick does not eliminate long-term water-related deterioration. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/138/) |
| **Yemen: Shibam** | Earthen tower houses reach approximately **seven stories** | Do not enforce a universal “adobe permits only one or two stories” rule. Geometry and maintenance matter. [Aliph Foundation](https://www.aliph-foundation.org/en/projects/emergency-restoration-program-for-damaged-buildings-in-the-old-walled-city-of-shibam) |
| **Southeastern China: Fujian tulou** | Multistory communal earthen buildings, some accommodating **up to 800 people**, use substantial protective roofs and eaves | Earthen architecture is not restricted to arid climates or small households. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1113/) |
| **Japan: Shirakawa-go** | Thatching depends on reciprocal **yui** labor and seasonal material preparation | A neighborhood institution can mobilize a crew far larger than the household. [vill.shirakawa.lg.jp](https://www.vill.shirakawa.lg.jp/2686.htm) |
| **West Africa: Djenné** | The Great Mosque’s annual replastering joins building maintenance to collective social practice | Repair demand can generate civic events rather than only private construction jobs. [Khan Academy](https://www.khanacademy.org/humanities/ap-art-history/africa-apah/west-africa-apah/a/great-mosque-of-djenne) |
| **Andes** | Inca stone production used selection, splitting, hammerstone dressing, and trial fitting | Encode several routes to sophisticated construction, not one European tool progression. [University of California Press](https://online.ucpress.edu/jsah/article/44/2/161/57823/Inca-Quarrying-and-Stonecutting) |
| **Southwestern North America** | Adobe performance depends strongly on moisture management and compatible repairs | Local repair knowledge is a productive capability, not merely an aesthetic tradition. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-05-adobe.pdf) |
| **Sri Lanka and Malawi** | Lime production ranges from small traditional burns to improved plants with different fuels and organization | Scale, feedstock, fuel supply, product quality, and market demand should vary independently. [Practical Action Publishing](https://practicalactionpublishing.com/book/2965/download?type=download) |

These cases argue for **regional material packages**: resource species and deposits, construction details, preservation practices, and labor institutions learned together.

---

## 4. Stylized facts a correct simulation should reproduce

| Pattern | Calibration or regression test |
| --- | --- |
| **An old building contains components of different ages.** | A centuries-old maintained building should be able to contain recently replaced roof coverings and much older structural members. Timber and tile evidence supports that possibility. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62262.pdf) |
| **Rapid construction events conceal preparation.** | Shirakawa-go roofing can mobilize up to about **200 people**, often completing roof work in a day; the harvesting figures imply another **100–167 person-days** before transport and other preparation. Do not report the visible roofing day as the whole labor cost. [vill.shirakawa.lg.jp](https://www.vill.shirakawa.lg.jp/2686.htm) |
| **Roof substitutions change structural demand.** | For a simple roof over **100 m² horizontal area** at **45°**, surface area is approximately **141 m²**. At the FAO example of **65 kg/m²** for clay tiles, covering mass is about **9.2 tonnes**, before supporting structure. This is a calculated scenario, not a universal tile-roof specification. [FAOHome](https://www.fao.org/4/s1250e/S1250E0l.htm) |
| **Water protection can matter more than nominal material prestige.** | In paired simulations, maintain one earthen building’s roof and drainage while allowing the other to leak. Their survival should diverge without changing wall material. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-05-adobe.pdf) |
| **Repair can be harmful when incompatible.** | Replacing sacrificial, compatible mortar with an excessively hard repair should sometimes increase damage to adjacent soft units rather than grant a permanent upgrade. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-02-repointing.pdf) |
| **Production efficiency is multidimensional.** | A kiln upgrade may reduce fuel but increase minimum viable scale, capital requirements, or handling constraints. Test total delivered cost, not fuel efficiency alone. [Practical Action Publishing](https://practicalactionpublishing.com/book/2966/download?type=download) |

---

## 5. Recommended TCE model

### 5.1 Separate resource, batch, component, and assembly

| Layer | Suggested state |
| --- | --- |
| **Resource deposit or vegetation stand** | Species/mineral class, quality distribution, accessible quantity, geometry, ownership/access rights, regeneration or depletion |
| **Production batch** | Recipe, producer, dimensions, dry mass, moisture, quality grade, defects, processing stage, provenance |
| **Building component** | Material batch references, geometry, exposure, wetness, permanent damage, surface condition, installation date |
| **Assembly** | Load path, roof/wall connections, drainage, protective layers, leakage routes, fire compartments |
| **Maintenance institution** | Responsible household/owner/group, obligations, funds, reciprocal labor credits, inspection knowledge |

For scale, aggregate identical bricks and tiles into component batches. **Do not create one simulation entity per brick.** Keep individual people visible while representing their work against quantities and work packages.

### 5.2 Use a reduced moisture-and-damage model

A proposed daily water balance is:

\[
W\_{t+1}=\operatorname{clip}
\left(
W\_t+I\_{\text{rain}}+I\_{\text{capillary}}+I\_{\text{leak}}
-E\_{\text{drying}},
W\_{\text{equilibrium}},W\_{\max}
\right).
\]

For wood, convert water mass to dry-basis moisture content. For mineral materials, use saturation or pore-water fraction. Roof condition, overhang, ground contact, wind exposure, and ventilation modify the flows.

For fungal wood decay:

\[
D\_{t+1}=D\_t+
f\_M(\text{moisture})
f\_T(\text{temperature})
f\_O(\text{oxygen})\Delta t.
\]

Then map accumulated dose to damage using species- and exposure-specific parameters. This follows the useful principle of Brischke and Rapp’s field research: **material moisture and temperature provide better decay information than climate indices alone**. Their study covered 23 European sites, so its fitted parameters should not be transferred unchanged to tropical species or insects. [Springer](https://link.springer.com/article/10.1007/s00226-008-0191-8)

Keep termites as a separate hazard. Drywood termites and fungal decay do not share identical moisture requirements. Likewise, surface mold or staining should not automatically remove load-bearing section. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62262.pdf)

For earth, distinguish **reversible wet weakening** from **irreversible erosion or cracking**. Drying can restore some material behavior without replacing material already washed away. Protective coatings should erode first where they function as sacrificial layers. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-05-adobe.pdf)

For masonry, accumulate damage during relevant moisture–temperature–salt cycles. “Cold day” alone is not equivalent to damaging freeze–thaw exposure; pore structure and water access must intervene. Historic-brick pore analysis provides a useful basis for differentiating material susceptibility. [Nature](https://www.nature.com/articles/s40494-022-00723-4)

### 5.3 Give agents partial knowledge and real maintenance choices

The following are modeling recommendations rather than universal historical behavioral laws.

A household should decide among patching, replacement, postponement, relocation, and rebuilding using cash or labor availability, expected occupancy, perceived danger, prestige, and obligations. Avoid assuming perfect lifetime-cost optimization.

Producers learn from observed outcomes. A brickmaker may know how many units broke during firing but not how they will behave after twenty wet winters. Builders and occupants acquire different knowledge.

Institutions can manage forest, quarry, clay-pit, and thatching-ground access; organize communal repair; extend credit; certify products; or allocate responsibility after failures. Reciprocal thatching labor in Shirakawa-go provides a particularly concrete template for recorded contributions and obligations. [vill.shirakawa.lg.jp](https://www.vill.shirakawa.lg.jp/2686.htm)

**Maintenance should be its own economic sector.** Roofers, plasterers, masons, and carpenters need recurring work even when settlement population stops growing.

### 5.4 Keep visual weathering separate from structural condition

Give components independent rendering channels: color change, biological staining, surface roughness, erosion depth, cracks, missing units, and patch history.

Proposed mappings include localized wall erosion beneath leaks, basal moisture damage, accumulated render patches, and individual missing tiles. Cosmetic treatment changes appearance; repointing changes joints; replacing a covering changes weather protection. None should automatically reset the entire building’s damage state.

This also makes material history legible: a maintained old building should look different from both a new building and an abandoned one.

### 5.5 Use existing models at the appropriate scale

| Model or game | Useful precedent | Recommended use |
| --- | --- | --- |
| **Brischke–Rapp decay-dose models** | Deterioration linked to material exposure histories | Basis for simplified timber-decay accumulation, with regional calibration. [Springer](https://link.springer.com/article/10.1007/s00226-008-0191-8) |
| **Fraunhofer WUFI** | Coupled transient heat and moisture behavior | Offline tests of representative walls and roofs; fit reduced daily models for TCE. [WUFI](https://wufi.de/en/software/what-is-wufi/) |
| **NIST Fire Dynamics Simulator / Smokeview** | Fire, heat, and smoke simulation | Offline calibration of a small number of building archetypes—not citywide runtime fluid simulation. [NIST Pages](https://pages.nist.gov/fds-smv/) |
| **Vintage Story** | Explicit clay-forming/firing stages, batch capacity, fuel, environmental interruption, and nearby fire hazards | Borrow readable production staging. Treat game times and recipes as design choices, not empirical rates; the consulted wiki page is version-qualified. [Vintage Story Wiki](https://wiki.vintagestory.at/index.php/Pit_kiln) |

For TCE’s scale, a reasonable **proposed scheduling scheme** is daily moisture updates, weekly slow-deterioration updates, event-triggered inspections and damage checks, and fine time steps only during active fire or collapse. Shared climate/exposure classes can avoid recalculating identical conditions for every surface.

---

## 6. Sources, datasets, and evidence limits

### Recommended reference backbone

| Source | Best use |
| --- | --- |
| **USDA Forest Products Laboratory, *Wood Handbook* (2021), especially chapters 5, 13, 14, and 18** | Species mechanics, drying, biological durability, and fire behavior. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/62200) |
| **FAO, *Farm Structures in Tropical Climates*** | Low-technology production sequences, tropical material choices, roofing, earth, brick, and lime. Valuable regional engineering guidance, not an ancient labor dataset. [FAOHome](https://www.fao.org/4/s1250e/s1250e00.htm) |
| **NPS Preservation Briefs 2, 5, and 30** | Mortar compatibility, adobe deterioration, and clay-tile conservation. [National Park Service](https://www.nps.gov/orgs/1739/upload/preservation-brief-02-repointing.pdf) |
| **Maithel and collaborators, *Brick Kilns Performance Assessment* (2012)** | Measured kiln thermal performance and clearly bounded process-accounting data. [cdn.catf.us](https://cdn.catf.us/wp-content/uploads/2019/10/21093607/Brick_Kilns_Performance_Assessment.pdf) |
| **Parracha et al., “Vernacular Earthen Buildings from Leiria, Portugal—Material Characterization”** | Actual historic-earth density and strength measurements; DOI **10.1080/15583058.2019.1668986**. [UNL Research](https://research.unl.pt/ws/files/17145015/Parracha_et_al_Leiria_vernacular_earth_buildings_material_charact_LJAH19_manuscript.pdf) |
| **Protzen, “Inca Quarrying and Stonecutting” (1985)** | Experimental reconstruction of stone production; DOI **10.2307/990027**. [University of California Press](https://online.ucpress.edu/jsah/article/44/2/161/57823/Inca-Quarrying-and-Stonecutting) |

### Useful datasets

**Global Wood Density Database, original 2009 release:** broad species coverage for authoring vegetation/material relationships. Verify whether each density uses oven-dry mass and green volume before comparing it with service-condition timber density. It is a density resource, not a direct structural-grade or decay-resistance database. [Dryad](https://datadryad.org/dataset/doi%3A10.5061/dryad.234)

**SoilGrids 2.0:** globally mapped soil properties with uncertainty, useful for generating regional clay/silt/sand distributions. Its approximately 250 m products cannot directly determine whether a particular excavated subsoil makes good adobe or firing clay; mineralogy and site testing remain separate. [Soil](https://soil.copernicus.org/articles/7/217/2021/)

**NIST Stone Test Wall:** a documented outdoor exposure collection of **2,352 stones**, including samples from numerous countries and rock types. It offers comparative weathering evidence, not a single stone-lifespan table. [NIST](https://www.nist.gov/el/materials-and-structural-systems-division-73100/stone-test-wall)

### What remains uncertain

The weakest evidence is for **portable historical production labor**, especially felling and conversion with particular tools, quarrying by geology and block size, and complete roof-production costs. Small-scale modern analogues are useful but may embed better tools, transport, nutrition, or organization than the society being simulated.

Service-life evidence has strong **survivorship and maintenance bias**. A five-hundred-year-old building does not establish that its original roof, mortar, or exposed timber lasted five hundred years. Quoted thatch lives are particularly sensitive to geography and craft practice. [Thatch Advice Centre](https://www.thatchadvicecentre.co.uk/wp-content/uploads/2014/11/english-heritage-thatch-and-thatching.compressed.pdf)

Laboratory strength is much better measured, but **small-specimen strength is not reliable building capacity without assembly rules**. Likewise, precise laboratory fire exposure does not establish a universal real-fire loss coefficient. [Yadda](https://yadda.icm.edu.pl/baztech/element/bwmeta1.element.baztech-59a5ffc3-6e23-4a26-90a6-17ed4b04a6ff/c/golebiewski_lubowiecka_kujawa_ceer_2015_18.pdf)

**Recommended minimum viable implementation:** begin with material batches, drying and firing queues, component-level moisture, separate reversible and permanent damage, roof-to-wall protection, recurring maintenance jobs, and material-specific fire response. Those mechanisms will produce more credible regional architecture and long-term urban history than a much larger catalogue of materials carrying fixed lifespan bonuses.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92918-8d30-83e9-a960-60b3745616fe)
