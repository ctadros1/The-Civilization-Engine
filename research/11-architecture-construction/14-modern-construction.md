# Modern construction: iron and steel frames, reinforced concrete, glass, elevators and the tall building

## Executive conclusion

**TCE should treat modern construction as a few new structural systems that techniques open, made from materials that long production chains supply, raised by trades that must be present on the site, and served by vertical circulation that has a capacity. It should not treat it as a "modern era" that removes height limits.** Iron and steel frames, reinforced concrete and elevators did not make tall buildings possible by themselves; they changed which constraint governs. With load-bearing masonry, the floor area eaten by thickening walls makes height uneconomic well before the brick crushes. With a frame, height is limited by stiffness against wind, by fire protection of the frame, by foundations, and above all by the floor area that elevators take and the time people spend waiting for them. The Monadnock Building in Chicago, sixteen storeys on brick walls about six feet (1.8 m) thick at the base and 18 inches at the top, was described as the highest economically viable load-bearing design; greater height would have cut too far into rentable space. [Structurae](https://structurae.net/en/structures/monadnock-building)

The second conclusion is that **people built tall because floor area near activity was dear, and a tall building is only rational when someone can use or let its upper floors.** The classic engineering-economics study (Clark and Kingston, 1930, for the American Institute of Steel Construction) defined economic height as the height at which the top floor's marginal revenue equals its marginal cost, and found 63 storeys for a site opposite Grand Central Terminal. [Building the Skyline](https://buildingtheskyline.org/?p=4490) Chicago data from 1870 to 2010 show height responding to land price, with that response roughly doubling over a century as construction technology improved. [LSE Research Online](https://researchonline.lse.ac.uk/id/eprint/64622) TCE already derives land value from what is paid and reckoned (ADR-0030), so height can follow from it without a rule that reads density.

As in the sibling reports, the text separates:

* **Physical capability:** what a frame, wall, slab or foundation can carry and how stiff it is, given its materials and condition.
* **Economic practicality:** what a floor costs to build and serve against what it saves or earns.
* **Builders' knowledge:** which systems, connections, calculations and site procedures a builder knows, and how well they estimate the margins.

Every section marks whether a statement is a **historical observation**, an **engineering specification** (a standard, code or design rule), or a **proposed TCE rule**. Dated examples are documented cases, never era gates.

---

## 1. Mechanisms: causal rules TCE can implement

### 1.1 A frame separates carrying the building from enclosing it

**Historical observation.** In a load-bearing building every wall carries the floors and walls above it, so walls thicken downward. In a skeleton frame, columns and beams carry the floors and the external wall becomes a non-structural skin carried storey by storey. The Chicago Architecture Center describes skeleton framing as letting walls become thin, non-load-bearing "curtains". [Chicago Architecture Center](https://architecture.org/online-resources/architecture-encyclopedia/skeleton-frame-construction) Early Chicago tall buildings were hybrids: until Jenney's Manhattan Building, Chicago's skyscrapers kept at least one masonry bearing or party wall, partly because the building code required masonry party walls for fire reasons. [CTBUH (Leslie)](https://global.ctbuh.org/resources/papers/4322-04.053~060(Leslie).pdf)

**Proposed TCE rule.** A frame building is a graph of columns per storey, beams or girders per bay, slabs per storey, and an enclosure group per storey whose weight is delivered to that storey's beams, not to the ground. Load-bearing masonry keeps ADR-0031's per-storey mass-wall groups, which already carry "the walls above" them. A hybrid (frame inside, bearing walls outside) is allowed and checked as both.

### 1.2 Each new material has its own governing failure modes

| System | What makes it work | Checks TCE should perform |
| --- | --- | --- |
| Cast-iron columns with cast-iron or brick-arch floors ("fireproof mill") | High compressive strength; floors spanned by shallow brick arches between iron beams | Column crushing and buckling; beam tension flange (brittle, little warning); thrust ties at end bays |
| Wrought-iron or steel beams on cast-iron columns | Ductile beams; brittle columns | Beam bending and deflection; column buckling; connection bearing |
| Steel skeleton (riveted, later bolted or welded) | Ductile members strong in tension and compression; rigid or braced joints | Member yield and buckling; connection capacity; lateral drift; temperature in fire |
| Reinforced concrete frame or flat slab | Concrete carries compression, steel bars carry tension; monolithic joints | Concrete crushing; bar tension and anchorage; shear; deflection; strength at the age when loaded; cover in fire |
| Load-bearing brick or stone at many storeys | Mass in compression | Per-storey crushing including own weight; slenderness; foundation bearing and settlement; floor area lost to thickness |
| Glass curtain wall | A glazed skin hung on the frame | Wind pressure on panes and mullions; no contribution to the frame; heat gain and loss |

**Engineering specification / historical observation.** Cast iron is strong in compression, weaker in tension and brittle, "like masonry"; wrought iron is strong in both but weaker than steel. [MIT OpenCourseWare](https://ocw.mit.edu/courses/4-448-analysis-of-historic-structures-fall-2004/20daf89c91e066a94c45f7656c1d7043_lec7_metal.pdf) Hodgkinson showed in the 1830s that a cast-iron beam's tension flange should be about six times the area of its compression flange. [Wikipedia: Eaton Hodgkinson](https://en.wikipedia.org/wiki/Eaton_Hodgkinson)

**Proposed TCE rule.** Material strengths and stiffness live in content as assembly values, as ADR-0031 moved masonry strengths into content. A brittle material (cast iron, plain concrete in tension) fails without passing through `STRAIN_SHOWS`; a ductile one (wrought iron, steel, under-reinforced concrete) shows strain first. This makes "cracks give warning" a property of the material rather than a constant.

### 1.3 Masonry height is limited by floor area before it is limited by strength

**Engineering specification (historical code).** The 1892 New York building code generally required a load-bearing wall to thicken by 4 inches (one brick wythe) for each 15 feet down from the top; curtain walls could be 4 inches thinner. [STRUCTURE magazine (Stuart)](https://www.structuremag.org/wp-content/uploads/2014/08/C-EngineersNB-Stuart-Jan101.pdf) A 1900 summary gives New York curtain walls of 12 inches for the upper 75 feet and 4 inches more for each 60 feet below. [Chest of Books (Cyclopedia of Carpentry and Building)](https://chestofbooks.com/architecture/Cyclopedia-Carpentry-Building-4-6/Building-Laws-And-Specifications.html)

**Proposed TCE rule.** A builder's knowledge of masonry includes a thickness-by-depth recipe (content), and the existing gravity margin checks it. The worked example in §2.6 shows that, for brick in lime, crushing reserve at sixteen storeys stays above three while the ground floor loses a quarter of its area to walls. Height in masonry should therefore be limited in TCE by the household's or firm's reckoning of floor gained against floor lost and work spent, not by a storey cap.

### 1.4 Lateral stiffness governs as frames get taller

**Historical observation.** Before frames, wind resistance relied mostly on the weight of masonry; Chicago designers of the 1880s were among the first to add dedicated lateral systems, and wind bracing became standard in frames during the 1890–91 boom. [CTBUH (Leslie)](https://global.ctbuh.org/resources/papers/4322-04.053~060(Leslie).pdf) Khan's "premium for height" framework (1960s) describes the extra material per unit floor area that a taller building needs because lateral load, not gravity, governs; it organised structural systems (rigid frame, braced frame, framed tube) by the heights at which each is economical. [UCLouvain (Beyond the premium-for-height framework)](https://research.dial.uclouvain.be/entities/publication/043e376b-ad85-4c70-9d49-829ddf78586e)

**Engineering specification.** A lateral deflection limit of about H/500 has been recommended for tall buildings since about 1940; modern US practice suggests H/400 to H/600 as serviceability guidance rather than a code limit. [HKIE Transactions (Robertson)](https://www.hkie.org.hk/hkietransactions/doc.php?id=479)

**Proposed TCE rule.** ADR-0031 §3 gives every group a lateral mode `V = C_s·W` used only when something shakes. Wind needs the same path, driven by the storm's gust load (`Weather::storm`) as pressure on the building's face: \(V\_{\text{wind}} = q\,C\_p\,A\_{\text{face}}\) with \(q = \tfrac12\rho v^2\) (11-05 §2.4 tabulates q at 20–40 m/s). A frame is checked for member and connection strength (failure) and for drift (serviceability: occupants and glass, not collapse). Bracing or rigid joints are content variants of the frame technique, each with its own steel per square metre.

### 1.5 Vertical circulation is a capacity, and it takes floor area

**Historical observation.** Otis's safety brake prevented a car from falling if its hoisting rope failed; the first commercial passenger installation with it, steam-powered, was in the E. V. Haughwout Building, New York, in March 1857. [Guinness World Records](https://www.guinnessworldrecords.com/world-records/first-building-to-have-a-safety-elevator-(lift)) The first electric passenger elevator was installed in 1889 (Demarest Building, New York); a gearless traction drive followed in 1903–04; express cars serving the upper floors of the Empire State Building (1931) ran at 1,200 ft/min (about 6 m/s). [Britannica](https://britannica.com/EBchecked/topic/184491/elevator) Secondary histories note that early hydraulic elevators with one-piece rams needed a hole beneath the shaft as deep as the car's rise, and Britannica notes that rises over 100 feet need compensating ropes. [Encyclopedia.com](https://encyclopedia.com/history/dictionaries-thesauruses-pictures-and-press-releases/elevators); [How Products Are Made](https://www.madehow.com/Volume-2/Elevator.html)

**Engineering specification.** Lift traffic design sizes an elevator group by its *handling capacity* (share of the population carried up in five minutes of the morning peak) and its *interval* (average time between departures from the lobby), assuming cars filled to about 80% of rated load. [Peters Research (CIBSE open forum report)](https://download.peters-research.com/library/CIBSE_Traffic_Analysis_and_Simulation_Open_Forum_Report.pdf) The two must be read together: a group giving 5% in 5 minutes at a 70 s interval serves very differently from one giving 8% at 45 s. [Lift and Escalator Library](https://liftescalatorlibrary.org/paper_indexing/papers/00000171.pdf)

**Proposed TCE rule.** An elevator is a fixed tool (machine) in a building, with a rated load, a speed and a drive whose power comes from ADR-0033's installations or engines, or later ADR-0036's wire. It gives a building a vertical service: trips per hour and wait time. People reaching upper floors spend time — walking a stair per storey, or waiting and riding — and that time enters the hours a household or firm reckons for the building (ADR-0030 §1). Each shaft takes floor area on every storey it passes. No building is barred from height; tall buildings without elevators are simply expensive to use, which makes upper floors less valuable and should reproduce the pre-elevator vertical rent gradient by itself.

### 1.6 Unprotected iron and steel lose strength in fire; protection is a property of the group

**Historical observation.** Ditherington Flax Mill, Shrewsbury (1796–97), generally held to be the oldest surviving iron-framed building, was built with iron beams within brick floors and walls so the shell would not burn; fire resistance, not height, was the owners' aim. It is about five storeys. [ICE](https://www.ice.org.uk/what-is-civil-engineering/what-do-civil-engineers-do/ditherington-flax-mill) In Chicago in 1871, exposed cast-iron construction was destroyed as easily as buildings that made no fireproof claim. [The Architecture Professor](https://thearchitectureprofessor.com/page/27) After the 1874 Chicago fire, insurers pressed to replace cast-iron columns with heavy timber, and encasement systems (wood and plaster, then porous terracotta tile) were tested and adopted. [The Architecture Professor](https://thearchitectureprofessor.com/2020/05/20/3-7-peter-b-wight-moves-to-chicago/) In Baltimore in 1904, the contents of "fire-proof" buildings burned out but their superstructures stood; investigators concluded that fire-resistive buildings could not stop a conflagration if they stood "as islands in a sea of firetraps". [Federal Reserve Bank of Richmond](https://www.richmondfed.org/publications/research/econ_focus/2015/q4/economic_history)

**Engineering specification.** By 550 °C structural steel retains about 60% of its room-temperature yield strength and 45% of its stiffness; at 600 °C about 40% and 30%. Below about 600 °C it recovers its strength on cooling. [SteelConstruction.info](https://steelconstruction.info/Fire_damage_assessment_of_hot_rolled_structural_steelwork); [Australian Steel Institute](https://steel.org.au/focus-areas/steel-and-fire/fire-damage-assessment-and-reinstatement-of-steel) The ASTM standard fire test (1918) rates an assembly by the time it survives a fixed furnace curve, about 1,700 °F (927 °C) at one hour; the rating does not predict performance in a real fire. [NIST](https://www.nist.gov/document/publicbriefingultestsgross082504pdf) Ingberg's 1928 burn-out tests related combustible contents of 49–290 kg/m² to standard exposures of about 1–7.5 hours. [NIST SP 958](https://nvlpubs.nist.gov/nistpubs/sp958-lide/html/028-032.html) Reinforced concrete is protected by the depth of concrete to its bars; Eurocode tables specify an axis distance to the bar centre by required fire period. [Structville](https://structville.com/fire-resistance-design-of-reinforced-concrete-structures)

**Proposed TCE rule.** Each frame group carries a *protection* (none, clay tile, concrete encasement, concrete cover) giving minutes before its metal reaches a critical temperature under a fully involved compartment. ADR-0022 already steps a burning building by minutes and burns its goods; the fuel under the roof (goods by mass per floor area) sets how long full involvement lasts. Unprotected metal loses strength by a content curve; the existing daily load check (or a check at burn-out) then fails whatever the fire weakened. A building that stands while its contents burn is a normal outcome, not an exception.

### 1.7 Materials come from long, capital-heavy chains

**Historical observation.** Portland cement was patented under that name by Joseph Aspdin in 1824, but his product was not clinkered; clinkered Portland cement, fired at much higher temperatures, was being sold by about 1845 (I. C. Johnson at J. B. White's works). [Science History Institute](https://www.sciencehistory.org/stories/magazine/hard-headed-man/); [Cliffe History](https://cliffehistory.co.uk/cement.html) Bessemer announced his converter in 1856; the Siemens–Martin open-hearth process followed in the 1860s, took scrap as well as pig iron, and made about 90% of British and US steel up to 1950. [Britannica: open-hearth process](https://britannica.com/technology/open-hearth-process); [Britannica: Bessemer steel](https://www.britannica.com/technology/steel/Bessemer-steel) The first wrought-iron building beams rolled in the US came from Trenton in 1854; I-beams had been rolled commercially in France in the 1840s. [AISC Modern Steel Construction](https://www.aisc.org/globalassets/modern-steel/archives/2010/09/2010v09_antiques_roadshow.pdf) The Crystal Palace (1851) used cast-iron parts from many foundries and about 300,000 panes of cylinder sheet glass in standard 49 × 10 in sizes. [Grace's Guide](https://www.gracesguide.co.uk/Crystal_Palace); [American Ceramic Society](https://ceramics.org/ceramic-tech-today/crystal-palace-history/)

**Proposed TCE rule.** The chains below are recipes on goods, fixed tools and power that M10 already built (blast furnace, pig iron, coke, lime burning, kiln, power at a site). Nothing in a building recipe names a date.

| Product | Chain | New vocabulary needed (proposed) |
| --- | --- | --- |
| Portland cement | Quarry limestone and clay → grind and blend → fire to clinker in a kiln (elapsed batch) → cool → grind with gypsum | `cement_clinker`, `portland_cement`; a hotter kiln or rotary kiln as a fixed tool; a grinding mill drawing kWh |
| Concrete | Cement + sand + gravel + water → mix → place in forms → cure (elapsed time) | Concrete is made on site as a building stage, not stored; formwork is reusable timber or steel |
| Bulk steel | Pig iron (+ scrap) → converter or open hearth (batch) → ingot | `bulk_steel` technique and recipe; a converter as fixed tool; blast power |
| Rolled sections and bar | Ingot → reheat → roll (power-hungry) → beams, columns, plate, bar | A rolling mill station with `kwh_per_unit`; goods `steel_section`, `reinforcing_bar` |
| Rivets and bolts | Bar → heading | Small goods; a riveting crew on site (§1.8) |
| Flat glass | Batch → melt → cylinder, cast-and-polish plate, drawn sheet, or float | No glass good exists in content yet; 07-05 §3.3 lists primary glassmaking and glazing as techniques |
| Fire protection | Clay → hollow tile (fired) or concrete encasement | Reuses `fire_tiles`-style kiln recipes |

### 1.8 Modern frames are raised by specialised crews and by calculation

**Historical observation.** On the Empire State Building, steel rose at about four and a half storeys a week, and 22 storeys of steel were placed in 22 working days in July 1930; 38 riveting gangs worked the joints, each of two riveters, a heater, a bucker-up and one or two helpers. [ENR](https://www.enr.com/articles/60722-the-race-into-the-sky-that-built-the-empire-state-building) Rivet crews needed about four skilled workers, while two relatively unskilled workers could install high-strength bolts; [Wikipedia: Rivet](https://en.wikipedia.org/wiki/Rivet) the research council formed in 1947 issued its first specification in 1951, making ASTM A325 bolts one-for-one replacements for rivets. [RCSC](https://www.boltcouncil.org/files/1stEditionGuide.pdf) For concrete, formwork is commonly 35–60% of the cost of a concrete structure, mostly labour to erect and strip it. [American Concrete Institute (SP-90)](https://www.concrete.org/publications/internationalconcreteabstractsportal/m/details/id/14047) Abrams's 1918 water–cement ratio law tied concrete strength to a measurable mix property. [Wikipedia: Duff Abrams](https://en.wikipedia.org/wiki/Duff_Abrams)

**Proposed TCE rule.** A frame building's stages name the crews they need, as ADR-0031's public works cap crews by work fronts: an erection crew per derrick, riveting gangs of a set size with one member knowing the technique, carpenters for formwork, finishers. Engineering calculation is a technique that narrows the *builder-estimated* reserve toward the physical one (11-05 §5.1's distinction); it never raises a material's strength. A builder without it uses the recipe's proportions and more caution, which costs material.

### 1.9 Foundations scale with what stands on them

**Historical observation.** Secondary accounts report that the Monadnock's heavy masonry settled extensively into Chicago's clay; the amount was not verified here. [Wikiwand (Wikipedia mirror)](https://www.wikiwand.com/en/Monadnock_Building) In Manhattan between 1890 and 1915, depth to bedrock raised costs by up to about 7% and had little influence on where skyscrapers were built. [Journal of Economic History (Barr, Tassier and Trendafilov)](https://www.cambridge.org/core/journals/journal-of-economic-history/article/depth-to-bedrock-and-the-formation-of-the-manhattan-skyline-18901915/56EAE58906036780174ACE6AF1B7C868)

**Proposed TCE rule.** A footing has a bearing area and the ground a bearing capacity; total weight per footing grows with storeys, so footings widen, deepen or need piles or caissons (a technique). Foundations should be a real but modest cost term, not a site veto.

### 1.10 Why people build tall

**Historical observation.** Clark and Kingston's 1930 developer analysis found profit maximised at 63 storeys, with 75 a close second. [Building the Skyline](https://buildingtheskyline.org/?p=4490) In Chicago, the land-price elasticity of height around 2000 was about 0.45 for tall commercial and 0.30 for tall residential buildings, roughly double its level a century earlier. [LSE Research Online](https://researchonline.lse.ac.uk/id/eprint/64622) Construction cost per floor area rises with height: an elasticity of about 0.25 for buildings of five floors or fewer, rising past 1 for super-tall structures. [VoxEU (Ahlfeldt and McMillen)](https://voxeu.org/article/tall-buildings-and-land-values) Office rents rise with floor level within buildings. [Journal of Urban Economics (Liu, Rosenthal and Strange)](https://econpapers.repec.org/RePEc:eee:juecon:v:106:y:2018:i:c:p:101-122) Height also met regulation: Chicago capped new buildings at about 130 feet in 1893 (sources give 130–160 ft); [Architectural Record](https://architecturalrecord.com/articles/10077-high-time-chicago-skyscrapers) New York's 1916 zoning resolution, adopted after the bulky Equitable Building (1915), tied height to setbacks. [99% Invisible](https://99percentinvisible.org/article/progressive-setbacks-the-century-old-nyc-mandate-that-shaped-modern-skylines/)

**Proposed TCE rule.** Keep ADR-0031 §6's logic: a storey is added when its cost is less than the ground, walking and fees it saves, and nothing pushes toward height. Modern systems change the cost curve; they do not add a motive. Skyscraper-scale height needs someone who can use or let many floors — a firm with many hands or a landlord letting storeys — which is the M8 brief's open question 7.

---

## 2. Parameters

**Confidence convention:** **H** — a well-defined measurement, documented installation or published standard value; **M** — a historical figure from a credible secondary source, or a context-bound study; **L** — a secondary compilation or a single uncorroborated source. **Design prior** means a proposed TCE starting value, not a historical estimate.

### 2.1 Material properties: keep test basis and meaning

| Material | Value | Basis and confidence |
| --- | --- | --- |
| Cast iron, tension | About **7.5 long tons/in² ≈ 116 MPa** average breaking strength | Fairbairn, as reported in a 19th-century handbook; experimenters varied widely. **M** [Chest of Books](https://chestofbooks.com/crafts/metal/Builder-Mechanic/Relative-Strength-Of-Cast-And-Malleable-Iron.html) |
| Cast iron, compression | Nearly **50 long tons/in² ≈ 770 MPa** crushing | Hodgkinson and Fairbairn, same handbook. Another period source gives a crushing-to-tension ratio near **4.3:1** rather than ~6.7:1. **M/L** |
| Wrought iron, tension | About **53,900 psi ≈ 372 MPa** (English mean) | Period compilation of Brown, Barlow, Brunel and Fairbairn. **M** |
| Steel at 550 °C / 600 °C | **~60% / ~40%** of yield; stiffness **~45% / ~30%** | Fire-damage assessment guidance. **H** for hot-rolled structural steel [SteelConstruction.info](https://steelconstruction.info/Fire_damage_assessment_of_hot_rolled_structural_steelwork) |
| Concrete strength at 7 days | About **74%** of 28-day strength under ideal moist laboratory curing at 73 °F | **M** [STRUCTURE magazine (Tobin)](https://www.structuremag.org/wp-content/uploads/2014/09/D-From-Experience-June-051.pdf). Warmer curing speeds early gain but lowers later strength. |
| Brick in lime mortar (assembly) | **2.6–4.3 MPa**, 18 kN/m³ | From 11-05 §2.3 (reference bands for existing masonry). **M** |

Historic iron and steel sections vary by mill and date; the BCSA/Gent handbook of historical sections is the reference for authoring section tables. [BCSA Historical Steelwork Handbook](https://www.steelconstruction.info/images/a/ab/Historical_Steelwork_Handbook.pdf) Modern grade strengths should be authored from the relevant standard, not from this report.

### 2.2 Production chains

| Parameter | Value | Basis and confidence |
| --- | --- | --- |
| Clinkering temperature | About **1,450 °C** today; Johnson's 1840s firing about **1,400–1,500 °C** | Secondary industry explainer and local history. **M** [Understanding Cement](https://understanding-cement.com/manufacturing.html); [Cliffe History](https://cliffehistory.co.uk/cement.html) |
| Raw material per tonne of clinker | About **1.7 t**, of which about **1.5 t** limestone | USGS. **H** [USGS](https://pubs.usgs.gov/publication/70023914) |
| Kiln thermal energy | Theoretical about **1.7 GJ/t** clinker; actual **3.0–6.5 GJ/t** by kiln type; WBCSD members' average **3.53 GJ/t** (2012) | **M** for the range; **H** for the reported average. [JCement (EC figures)](https://en.jcement.ru/reading/bookshelf/assessing-cement-plant-thermal-performance/); [RITE](https://www.rite.or.jp/system/en/latestanalysis/pdf/E-Comparison_EnergyIntensity2010cement.pdf) |
| Calcination CO₂ | **0.507 t CO₂/t clinker** (default); total with fuel about **1 t**, half from each | **H** [IPCC](https://www.ipcc-nggip.iges.or.jp/public/gp/bgp/3_1_Cement_Production); [USGS](https://pubs.usgs.gov/publication/70023914) |
| Bessemer blow | **15 minutes** for a charge of about **half a ton** in Bessemer's early account; later converters treated heats of **5–30 t** | **M** [Britannica](https://www.britannica.com/technology/steel/Bessemer-steel) |
| Steel rail price (Pennsylvania) | About **$17/ton** by 1898 | EB1911 table. **M** [Wikisource (EB1911)](https://kwx.liehendr.com/content/wikisource_en_all_maxi_2022-09/A/Page%3AEB1911_-_Volume_14.djvu/866). Earlier figures in that table appear corrupted; no clean iron-versus-steel series was found. |
| Flat glass, 1851 | Chance Brothers made nearly **300,000 panes (about 400 t)** for the Crystal Palace in under a year | **M** [American Ceramic Society](https://ceramics.org/ceramic-tech-today/crystal-palace-history/) |
| Float glass | Announced **January 1959**; replaced grinding and polishing of plate glass | **H** [Pilkington](https://www.pilkington.com/en/gbl/about-us/pilkington-history/invention-of-float-glass) |

### 2.3 Construction quantities, work and time

| Parameter | Value | Basis and confidence |
| --- | --- | --- |
| Steel erection pace, Empire State Building | **~4.5 storeys/week**; **22 storeys in 22 working days** at best; **410 days** to completion; **57,000 short tons** of steel | **M** [ENR](https://www.enr.com/articles/60722-the-race-into-the-sky-that-built-the-empire-state-building) |
| Steel per gross floor area, Empire State Building | About **250 kg/m²** | My arithmetic: 57,000 short tons over about 2.25 million ft² gross [Wikipedia](https://en.wikipedia.org/wiki/Empire_State_Building). A 102-storey building; not a general value. |
| Gravity-only versus wind-braced steel | **122 vs 317 kg/m²** in a slide crediting Khan (1959) | **L**: fragmentary secondary slide. [Infosteel (Odenbreit)](https://mail.infosteel.be/images/staalbouwdag/2015/001--Odenbreit.pdf) Use only for shape: bracing can more than double steel. |
| Riveting gang | **4–5** people (two riveters, heater, bucker-up, helpers); bolts need about **2** | **M** [ENR](https://www.enr.com/articles/60722-the-race-into-the-sky-that-built-the-empire-state-building); [Wikipedia: Rivet](https://en.wikipedia.org/wiki/Rivet) |
| Formwork share of concrete-structure cost | **35–60%** | **M** [ACI SP-90](https://www.concrete.org/publications/internationalconcreteabstractsportal/m/details/id/14047) |
| Reinforcing steel per m³ of concrete | **80–150 kg/m³** residential, **120–220** commercial | **L**, estimating rules of thumb from modern South Asian practice. [The Constructor](https://theconstructor.org/practical-guide/size-and-quantity-of-reinforcement-for-building-works/7365/) **Design prior.** |
| Hennebique system | Patented **1892**; beams with stirrups and longitudinal bars | **H** for the patent and system [Britannica](https://www.britannica.com/biography/Francois-Hennebique) |

### 2.4 Height, circulation and economics

| Parameter | Value | Basis and confidence |
| --- | --- | --- |
| Masonry wall thickening rule (NYC 1892) | **+4 in per 15 ft** down from the top | **H** as a historical code rule [STRUCTURE magazine](https://www.structuremag.org/wp-content/uploads/2014/08/C-EngineersNB-Stuart-Jan101.pdf) |
| Monadnock walls | About **1.8 m** at the base, **0.46 m** at the top, 16 storeys | **M** [Structurae](https://structurae.net/en/structures/monadnock-building) |
| Elevator speed, 1931 express | **1,200 ft/min ≈ 6.1 m/s** | **M** [Britannica](https://britannica.com/EBchecked/topic/184491/elevator) |
| Office lift design target | Handling capacity about **12%** of population in 5 minutes; interval **≤ 30 s**; cars loaded to **≤ 80%** | **M** (course notes summarising CIBSE Guide D) [Scribd (MECH3005 notes)](https://www.scribd.com/document/385030836/mech3005-1011-lift02-pdf); [Peters Research](https://download.peters-research.com/library/CIBSE_Traffic_Analysis_and_Simulation_Open_Forum_Report.pdf) |
| Office occupancy for lift sizing | **8–10 m² net per person** ordinary; **12–20** prestige | **M**, same secondary sources |
| Net-to-gross floor area | **70–75%** common above 30 storeys; Japanese tall buildings **64–85%**, core **13–34%** of gross | **M** [Building (1997 cost model)](https://www.building.co.uk/high-rise-office-towers-cost-model-may-1997/1025316.article); [Atılım University (Ilgın and Aslantamer)](https://ada.atilim.edu.tr/entities/publication/dedd7811-5f1c-486a-83fb-15d8c7abc7ad) |
| Single-rise rope limit | Conventional steel-rope rise about **500 m**; Burj Khalifa cars travel at most **504 m** | **L**, manufacturer claim reported in the press [The Register](https://www.theregister.co.uk/2013/06/18/ultrarope_to_enable_1000_meter_buildings/) |
| Construction cost elasticity with height | About **0.25** up to five floors, **> 1** for super-tall | **M** [VoxEU](https://voxeu.org/article/tall-buildings-and-land-values) |
| Land-price elasticity of height (Chicago, 2000) | **~0.45** commercial, **~0.30** residential | **M**, working-paper estimate [LSE](https://researchonline.lse.ac.uk/id/eprint/64622) |
| Bedrock depth effect on cost (Manhattan 1890–1915) | Up to **~7%** | **M** [Journal of Economic History](https://www.cambridge.org/core/journals/journal-of-economic-history/article/depth-to-bedrock-and-the-formation-of-the-manhattan-skyline-18901915/56EAE58906036780174ACE6AF1B7C868) |
| Wind drift limit | **H/500** since about 1940; **H/400–H/600** modern guidance | **M** [HKIE Transactions](https://www.hkie.org.hk/hkietransactions/doc.php?id=479) |

### 2.5 Design priors where evidence is thin

These are proposed TCE starting values for sensitivity tests. **They are not historical measurements.**

| Prior | Starting range | Why it is needed |
| --- | --- | --- |
| Storey height in frame buildings | **3.3–4.0 m** | No source gathered here; 11-09 gives 2.59–3.20 m for London houses |
| Elevator round-trip time at 10–20 floors | **90–180 s** | Needed to turn capacity targets into car counts; no historical series found |
| Floor area per elevator shaft and lobby share | **5–10 m²** per car per floor | Needed for the floor-area cost of circulation |
| Stair climb time | **10–20 s per storey** walking up, more when carrying | Turns stair height into hours in ADR-0030's reckoning |
| Critical steel temperature | **500–650 °C** depending on load ratio in fire | Brackets the 550–600 °C values in §2.1 |
| Minutes of protection | **0** unprotected; **30–120** encased or covered, by thickness | Real ratings depend on tested assemblies |
| Fuel load per floor area | Computed from goods present; **25–150 kg/m²** wood-equivalent as a check | Ingberg's 49–290 kg/m² were office and record-room tests |
| Formwork reuse | **5–20** uses per set before scrapping | Sets the cost of repeated floors; formwork cost is mostly labour |
| Concrete stripping age | Loadable at a share of 28-day strength set by the recipe (e.g. **0.6–0.75**) | Ties curing time to the structural check |
| Reinforcement mass | **1–3%** of concrete volume by element | Keeps rebar demand within the rule-of-thumb band |

### 2.6 Worked example: masonry against a frame on a 20 × 30 m plot

All values are my arithmetic, using the 1892 New York thickening rule (30 cm at the top, +10 cm per 4.57 m below), a 3.8 m storey (design prior), brick at 18 kN/m³, and a 30 cm curtain wall for the frame.

| Storeys (height) | Masonry base wall | Wall self-weight stress at base | Ground-floor area lost to outer walls | Average area lost over all storeys | Frame with curtain wall, area lost |
| --- | --- | --- | --- | --- | --- |
| 6 (22.8 m) | 0.70 m | 0.31 MPa | 11.3% | 8.7% | 4.9% |
| 12 (45.6 m) | 1.20 m | 0.54 MPa | 19.0% | 12.6% | 4.9% |
| 16 (60.8 m) | 1.60 m | 0.65 MPa | 25.0% | 15.1% | 4.9% |

Floors add little to this: twelve floors at 5.5 kPa (dead plus live, design prior) on a 3 m tributary width add about 0.17 MPa on a 1.2 m wall. Brick in lime at 2.6–4.3 MPa then keeps a crushing reserve near 3.7–6 at twelve storeys. **The binding constraint is area, not strength.** The 16-storey case reproduces the Monadnock's reported 1.8 m base within the rule's precision.

Circulation for the 12-storey frame: 11 upper floors at about 450 m² net and 10 m²/person hold about 500 people. A 12% handling capacity needs 60 people per five minutes. With 16-person cars filled to 80% (13 people) and a 120 s round trip (design prior), one car moves about 32 people per five minutes, so two cars meet capacity, but a 30 s interval needs four. **Waiting time, not carrying capacity, sets the car count**, and four shafts at 5–10 m² each take 4–9% of every floor.

---

## 3. Historical and regional variation

### 3.1 Capability regimes, not stages

| Regime | What the combination allows | What it does not imply |
| --- | --- | --- |
| Load-bearing masonry with timber floors | Up to the floor-area limit of §2.6; fire spreads through floors | That masonry is "pre-modern": brick and concrete-block bearing walls remain common for low buildings |
| Iron-framed, brick-arch "fireproof" interiors | Wide, incombustible floors for mills and warehouses | Height: Ditherington is about five storeys |
| Hybrid frames inside bearing walls | Thinner inner structure; walls still thicken | That the frame is independent of the wall |
| Full skeleton frame with curtain walls | Height limited by lateral stiffness, fire protection, circulation and land economics | That every building uses it; low buildings keep cheaper systems |
| Reinforced concrete frames and slabs | Monolithic, fire-resistant frames from local aggregate, cement and modest steel; slow curing per floor | A steel industry: bars can be imported or rolled small |
| Sealed glass skin with mechanical air | Deep floor plates and glazed façades | That glass façades are universal; climate and energy costs matter |

These regimes coexist. A city can have a steel tower beside new brick houses, and a region without steel can build concrete frames from imported bar.

### 3.2 Dated examples (documented cases, not unlock dates)

| Place and date | Documented development | What TCE should learn |
| --- | --- | --- |
| Shrewsbury, **1796–97** | Ditherington Flax Mill: iron frame with brick floors, built for fire resistance, about five storeys | The first purpose of iron frames was fire, not height. [ICE](https://www.ice.org.uk/what-is-civil-engineering/what-do-civil-engineers-do/ditherington-flax-mill) |
| England, **1824–c.1845** | Portland cement named in 1824; clinkered product sold by about 1845 | A name and a working product are different milestones. [Science History Institute](https://www.sciencehistory.org/stories/magazine/hard-headed-man/) |
| London, **1851** | Crystal Palace: standard cast-iron parts from many foundries and ~300,000 standard glass panes | Standardised parts allow fast assembly when supply chains exist. [Grace's Guide](https://www.gracesguide.co.uk/Crystal_Palace) |
| New York, **1857** | First commercial passenger safety elevator, steam-driven | An elevator needs a power source at the building. [Guinness World Records](https://www.guinnessworldrecords.com/world-records/first-building-to-have-a-safety-elevator-(lift)) |
| France, **1867–1892** | Monier's reinforced flowerpots (1867); Hennebique's beam system patented 1892 | Reinforced concrete grew from small objects into a building system over decades. [Wikipedia: Reinforced concrete](https://en.wikipedia.org/wiki/Reinforced_concrete); [Britannica](https://www.britannica.com/biography/Francois-Hennebique) |
| Chicago, **1871–1874** | Exposed iron failed in fires; insurers pressed for encasement | Fire experience changes construction through insurers and codes, not by itself. [The Architecture Professor](https://thearchitectureprofessor.com/2020/05/20/3-7-peter-b-wight-moves-to-chicago/) |
| Chicago, **1885** | Home Insurance Building, 10 storeys, 42 m: cast-iron columns, wrought-iron beams, and two floors of rolled steel beams substituted during construction | "First skyscraper" is disputed; frames arrived as hybrids. [Britannica](https://www.britannica.com/technology/building-construction/Early-steel-frame-high-rises); [CTBUH (Larson)](https://www.ctbuh.org/people-profile/gerald-larson) |
| Chicago, **1889–1893** | Monadnock: 16 storeys on bearing brick (north half); steel frame in the south half | Two systems on one block, chosen by owners. [Chicago Landmarks](https://webapps1.chicago.gov/landmarksweb/web/landmarkdetails.htm?lanId=1373) |
| Cincinnati, **1903** | Ingalls Building, 16 storeys, the first reinforced-concrete skyscraper; the tallest earlier concrete building had six storeys, and approval took about two years | A new system faces a permit and trust barrier, not only an engineering one. [ASCE](https://asce.org/about-civil-engineering/history-and-heritage/historic-landmarks/ingalls-building) |
| Baltimore, **1904** | Frames stood while contents burned; isolated fire-resistive buildings could not stop a conflagration | Fire resistance is a property of a district, not one building. [Federal Reserve Bank of Richmond](https://www.richmondfed.org/publications/research/econ_focus/2015/q4/economic_history) |
| New York, **1915–1916** | Equitable Building; first citywide zoning with height and setback rules | Height rules follow felt losses (light, congestion). [99% Invisible](https://99percentinvisible.org/article/progressive-setbacks-the-century-old-nyc-mandate-that-shaped-modern-skylines/) |
| San Francisco, **1918** | Hallidie Building, an early large glass curtain wall | Curtain walls predate the post-war glass tower. [Docomomo US](https://docomomo-us.org/register/hallidie-building) |
| New York, **1930–31** | Empire State Building, 102 storeys in 410 days | Speed came from organisation, standard parts and crews, not a new material. [ENR](https://www.enr.com/articles/60722-the-race-into-the-sky-that-built-the-empire-state-building) |
| US, **1947–1951** | High-strength bolts specified as rivet replacements | Labour cost and skill drive connection choice. [RCSC](https://www.boltcouncil.org/files/1stEditionGuide.pdf) |
| New York, **1952** | Lever House: sealed curtain wall with heat-absorbing glass; ventilation entirely by air conditioning | A sealed glass skin depends on a mechanical service. [Columbia GSAPP](https://columbia.edu/cu/gsapp/BT/LEVER/report.html) |

**Later practice (notes only).** Framed and bundled tubes (1960s–70s), welded and composite frames, float glass, and high-speed elevators with sky lobbies extended the same constraints; very tall towers still split elevator rises because steel rope weight grows with rise. [The Register](https://www.theregister.co.uk/2013/06/18/ultrarope_to_enable_1000_meter_buildings/)

### 3.3 Regional variation should come from inputs

Steel frames favour places with steel mills, rolling capacity, rail delivery and skilled erectors; reinforced concrete favours places with cement, aggregate, cheap carpentry labour for formwork and imported or locally rolled bar. A region where formwork labour is cheap relative to steel should drift toward concrete; one with cheap steel and dear labour toward steel. Paris tied building height to street width (its 1902 code set heights and projections) long after frames were available; the exact limits were not verified here, [Brandon Donnelly (secondary)](https://brandondonnelly.com/2023/05/01/how-paris-zoning-envelope-has-changed-over-the-centuries/) which shows that rules can bind before physics or economics do.

---

## 4. Stylized facts and validation targets

| Pattern | Evidence or anchor | Simulation test |
| --- | --- | --- |
| **Masonry height is limited by area, not crushing** | Monadnock's base walls; §2.6's arithmetic | Sweep storeys for a brick program with free land prices: crushing reserve stays above ~3 while floor lost passes 15–25%; height stops where floor lost outweighs ground saved. |
| **Frames are adopted where land is dear and floors are let or used** | Chicago height–land-price elasticity [LSE](https://researchonline.lse.ac.uk/id/eprint/64622) | With frames known everywhere, tall buildings cluster where ADR-0030's paid and reckoned land values are highest; none appear on cheap ground. |
| **Height follows land price with an elasticity well below 1** | 0.30–0.45 in 2000 | Fit height on recorded land value after growth; do not target the value, only check order of magnitude and sign. |
| **Cost per floor area rises with height, in steps** | Elasticity ~0.25 low, >1 super-tall; step changes at technical thresholds [Building](https://www.building.co.uk/pay-a-premium-to-reach-for-the-skies-new-bco-report-finds/3057016.article) | Steps should appear where bracing, an extra elevator bank or bigger footings become necessary, from the checks, not authored. |
| **Upper floors are worth less without elevators and more with them** | Vertical rent gradient in modern offices [JUE](https://econpapers.repec.org/RePEc:eee:juecon:v:106:y:2018:i:c:p:101-122) | Households' reckoned value per floor falls with storey without a lift; adding one flattens or reverses it. |
| **Unprotected metal frames fail in burn-outs; protected ones survive their contents** | Chicago 1871; Baltimore 1904 | A burnt building with protected frame stands with goods lost; unprotected cast iron gives way. |
| **Isolated fire-resistive buildings do not stop a conflagration** | Baltimore 1904 | ADR-0022's exposure spreads around a protected building through combustible neighbours. |
| **New systems meet distrust after failures and before permits** | Ingalls's two-year permit fight | `TiedWalls`-style rules and caution by technique (ADR-0031 §5) slow the first concrete towers; a collapse raises caution for that technique only. |
| **Old systems persist** | Brick houses beside towers | At 1% replacement most stock stays as built for decades (11-02 §2.3); low buildings keep masonry and timber. |
| **Steel erection is fast, concrete waits on curing** | 4.5 storeys a week; 7-day ≈ 74% strength | A concrete frame's floor cycle is bounded by curing and form reuse; loading a slab too early can fail it in the existing check. |

Engineering invariants: no enclosure group delivers load to the ground; floor area taken by walls, cores and shafts is subtracted from usable area; a building's elevators cannot carry more than their trips allow; fire protection minutes never raise room-temperature strength.

---

## 5. Modeling recommendation for TCE

Recommendations are listed with the existing TCE parts they stand on. None is a date, era or coverage radius.

### 5.1 Materials as recipes on the M10 chain

- **Cement.** Add goods `cement_clinker` and `portland_cement`, a technique `portland_cement` that `requires` `lime_burning` and `firing`, and a recipe that fires blended limestone and clay as an `elapsed_days` batch in a kiln, then grinds with power. Inputs follow §2.2 (about 1.7 kg of raw material per kg of clinker; fuel from the kiln's energy per tonne, any fuel its recipe allows). *Stands on:* `content/core/technique/lime_burning.toml`, `content/core/good/kiln.toml`, `content/core/good/quicklime.toml`; batches and `kwh_per_unit` in `decisions/0033-power-fuel-and-the-works.md` §2 and §5.
- **Steel.** Add `bulk_steel` (a converter or hearth batch from `pig_iron` and scrap) and `rolling` (a powered mill making `steel_section` and `reinforcing_bar`). M10 deferred "bulk and crucible steel" (`docs/briefs/m10-power.md` §2); this is where it is wanted. *Stands on:* `content/core/good/pig_iron.toml`, `content/core/technique/blast_furnace.toml`, power at a site (ADR-0033 §5).
- **Glass.** Add primary glassmaking and a flat-glass recipe (cylinder sheet first); plate and float are later techniques that make the same good more cheaply. Content has no glass yet.
- **Concrete is a building stage, not a good.** Cement, sand, gravel and water are consumed on site; formwork is a reusable tool good worn per use.

### 5.2 Structural systems as vocabulary opened by techniques

- **A new grammar family, not frame v3 stretched.** Frame v3 is frozen at `MAX_STOREYS` 4 (`decisions/0031-tall-public-buildings-and-shaking.md` §1). Propose a `tower` grammar (version 1) with groups: column (per storey), girder and beam (per bay), slab (per storey), enclosure (per storey, carried by beams), core (stair and shafts), footing. Bays and storeys come from the program's content range; the checks decide what stands.
- **Techniques open systems.** `iron_frame` (cast columns, wrought or steel beams, brick-arch floors), `skeleton_frame` (steel columns and beams, enclosure carried per storey), `wind_bracing` (braced or rigid joints), `reinforced_concrete` (monolithic frame; needs `portland_cement` and forming), and `structural_calculation` (narrows the builder's estimate, §1.8). Each `requires` its material techniques and `jointed_frame`; each is `tried_in` building with the system below it; founders know none. *Stands on:* the technique format in `content/core/technique/tied_masonry.toml` and ADR-0008 knowledge carried by people.
- **Masonry stays and grows.** Let the masonry program expand past four storeys in the new grammar with per-storey walls thickened by its content rule, so §2.6's area loss appears in the household's reckoning. *Stands on:* ADR-0031 §1–2 per-storey mass walls that carry the walls above.

### 5.3 Checks: extend the margins, add material behaviour

- **Gravity.** Reuse `civ-agents/src/structure.rs`'s bending, buckling and crushing with material constants from content (§2.1). Add a tension check for brittle members and a reinforced-concrete section (concrete in compression, bar in tension). Brittle materials skip `STRAIN_SHOWS`.
- **Age of concrete.** A concrete group's strength is its 28-day value times a content curve of its age since placing (e.g. 0.74 at 7 days, §2.1); the existing daily load check fails a slab loaded too early. Curing becomes real elapsed time in the floor cycle.
- **Wind.** Add a wind lateral case to ADR-0031 §3's lateral mode, evaluated when a storm comes (`Weather::storm`), with pressure on the face and a drift check reported as serviceability (cracked glass and finishes, unease) separate from strength failure. Drift H/500 is a content prior, not a law.
- **Foundations.** Sum each column's load to its footing; footing area and depth come from the design and ground bearing (a content prior until ground classes exist, M8 brief open question 5). Settlement waits, as in 11-05 §1.5.

### 5.4 Vertical circulation as a capacity

- **Stair** (already a part since ADR-0031 §1) gives a climb time per storey (design prior §2.5).
- **Elevator** is a fixed tool `elevator` with rated load, speed and a drive that draws kW from the building's installation, engine or connection (`decisions/0033-power-fuel-and-the-works.md` §5; `docs/briefs/m11-power.md` §2's "power by wire to recipes that take power"). Its shafts take floor area on every storey they serve.
- **Service is computed, not flagged.** From the building's occupants per floor and its cars, derive a peak wait and ride time per trip (a closed-form round-trip estimate is enough); the hours spent climbing or waiting enter ADR-0030 §1's hours reckoning for that floor. A dead elevator (no power, no mender) makes upper floors cost more hours, through the same path.
- **No elevator bonus.** An elevator raises no rent or productivity directly; it only removes climbing hours.

### 5.5 Fireproofing as a property of a group

- Each metal or concrete group has a `protection` in minutes (none, tile, encasement, cover). *Stands on:* ADR-0022 §4–5's fire stages stepped by minutes and goods burned under the roof (`decisions/0022-fire-floods-and-storms.md`).
- While a compartment is fully involved, unprotected metal heats and loses strength along a content curve (60% at 550 °C, 40% at 600 °C); protected metal does so only after its minutes run out. The fuel load comes from the goods actually stored, so a warehouse and an office burn differently.
- Masonry walls already crack after a full burn (ADR-0022 §5); concrete with too little cover loses bar strength the same way.
- **Rules.** Append `RuleTemplate::ProtectedFrame { minutes }` beside `MostStoreys` and `WallsOfKinds`, proposable only by someone who knows a protection technique and opened by `IssueKind::Burned`. *Stands on:* `docs/briefs/m6-fire.md` §1.6 and ADR-0031 §5's rule templates.

### 5.6 Why build tall: keep it in the reckoning

- Height stays a choice in hours (ADR-0030 §1; ADR-0031 §6): another storey wins when its structure, foundation, circulation and lost floor area cost less than the ground and walking it saves. Modern systems only change those costs.
- Letting floors is the missing piece. A tower is only reckoned worth building by a party who uses or lets many floors: a firm with many hands (`docs/briefs/m10-power.md` §1.6's works) or a landlord letting storeys on the `Rent` channel (M8 building brief, open question 7). Without it, no frame will be built for height; that is a correct outcome, and a `NUDGE:` belongs on letting, never on storeys.
- Height rules (`MostStoreys`, a future setback or light rule) are proposed by those who lose light or ground, as ADR-0031 §5 proposes them after shaking.

### 5.7 Trades on site

- Each stage names a crew: erection per derrick, a riveting gang (four or five, one knowing the technique), formwork carpenters, concrete placers, glaziers. Crews are capped by work fronts as in `raise_building` (ADR-0031 §6). Bolting is a later technique that needs fewer and less skilled hands.
- Hired hands come from the works' wage offers (`docs/briefs/m10-power.md` §1.6). Plan §9's NUDGE that hired work is rare means tall-building construction may stall for want of hands; measure it before tuning.

### 5.8 What to measure, and what to defer

**Measure:** floor lost to walls and cores by system and storeys; steel and cement per square metre; floor cycle times; elevator waits; reckoned land value against height; fires by protection; who builds tall and why others rejected it.

**Defer:** welding and composite construction; tubes and outriggers beyond one braced-frame variant; prestressed concrete; air conditioning and sealed façades (they need a cooling service); glass heat balance; pile design and settlement; elevator group dispatch beyond a closed-form estimate.

---

## 6. Sources, datasets, and limits of the evidence

### Core references

| Source | Principal use |
| --- | --- |
| **Clark and Kingston, *The Skyscraper: A Study in the Economic Height of Modern Office Buildings* (AISC, 1930)**, as discussed by Barr | Economic height as marginal revenue equals marginal cost. [Building the Skyline](https://buildingtheskyline.org/?p=4490) |
| **Ahlfeldt and McMillen, "Tall Buildings and Land Values", *Review of Economics and Statistics* 100(5), 2018** | Height and cost elasticities from 140 years of Chicago land prices. [LSE Research Online](https://eprints.lse.ac.uk/84873) |
| **Barr, Tassier and Trendafilov, *Journal of Economic History*, 2011** | Bedrock as a minor cost; location as the driver. [Cambridge Core](https://www.cambridge.org/core/journals/journal-of-economic-history/article/depth-to-bedrock-and-the-formation-of-the-manhattan-skyline-18901915/56EAE58906036780174ACE6AF1B7C868) |
| **Leslie, CTBUH paper on Chicago braced frames; Larson's CTBUH papers** | Hybrid frames, wind bracing, the "first skyscraper" dispute. [CTBUH](https://global.ctbuh.org/resources/papers/4322-04.053~060(Leslie).pdf) |
| **BCSA/Gent, *Historical Structural Steelwork Handbook*** | Properties of historical cast iron, wrought iron and steel sections. [SteelConstruction.info](https://www.steelconstruction.info/images/a/ab/Historical_Steelwork_Handbook.pdf) |
| **van Oss and Padovani, USGS, "Cement manufacture and the environment"** | Raw materials, energy and emissions per tonne of clinker. [USGS](https://pubs.usgs.gov/publication/70023914) |
| **NIST: ASTM E119 history; Ingberg's fire-severity tests (SP 958)** | What a fire rating means and how fuel load relates to exposure. [NIST](https://nvlpubs.nist.gov/nistpubs/sp958-lide/html/028-032.html) |
| **CIBSE Guide D (via Peters Research reports)** | Lift traffic design definitions and targets. [Peters Research](https://download.peters-research.com/library/CIBSE_Traffic_Analysis_and_Simulation_Open_Forum_Report.pdf) |

**Useful datasets:** Ahlfeldt and McMillen's Chicago tall-building and land-price panel; the CTBUH Skyscraper Center for heights, dates, systems and materials of tall buildings (a survivor-biased record of tall buildings, not the building stock); USGS Mineral Commodity Summaries for cement and steel output series.

### What should remain uncertain

**The cost and time of masonry against frames, building by building.** No source gathered here gives matched historical costs per square metre for a bearing-wall and a framed building of the same height and place. The §2.6 comparison is arithmetic on a code rule; schedules and costs should be computed from TCE's own recipes, not imported.

**Historical material strengths.** Nineteenth-century tests vary widely (cast iron's crushing-to-tension ratio is reported anywhere from about 4:1 to about 7:1). Treat them as bands with a hidden draw per batch, as 11-08 §1.2 recommends.

**Elevator performance before about 1930.** Speeds, round-trip times and waits for early hydraulic and electric installations were not found in comparable form. The design priors in §2.5 stand in for them.

**Steel per square metre by system and height.** Khan's original charts were not retrieved; the 122 versus 317 kg/m² figures are from a secondary slide. Let steel quantity follow from the checks.

**Fire ratings are not fire outcomes.** The standard curve and Ingberg's equivalence are approximations that ignore ventilation; they are useful for sizing protection, not for predicting a particular fire.

**Priority claims.** "First skyscraper", "first curtain wall" and "first reinforced-concrete building" all depend on definitions. The dated examples above are documented cases chosen for what they teach, not a sequence any world should follow.

**Bottom line:** TCE's tall buildings should emerge from **land that is dear, someone who can fill the floors, materials that a chain actually delivers, crews who know the system, and margins, circulation and fire protection that the engine checks**. With those, a city can raise brick blocks, iron mills, concrete warehouses and an occasional steel tower side by side, and can just as plausibly never build tall at all.

---

Research notes: compiled in this session from web search results; several hosts (enr.com, structurae.net, asce.org, steelconstruction.info) could not be fetched directly, so their figures were taken from search-result summaries of those pages and are marked M.
