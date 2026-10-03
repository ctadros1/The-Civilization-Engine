# Earthworks for The Civilization Engine

## Labor, construction, maintenance, and failure

**The most useful design decision is to treat earthworks as construction projects with three linked accounts: soil, labor, and water.** A terrain-height change is their result—not the operation itself.

Practical hand-excavation norms span roughly **0.5–4 m³ per worker per eight-hour day**, depending on ground and tools. But these are largely digging-and-short-throwing rates: they do not automatically include transport, careful shaping, compaction, drainage, or retaining walls. Those additional operations can dominate a finished project. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e12.htm)

For TCE, a village ditch, a terraced hillside, and a levee should therefore use the same underlying earthmoving system but different construction sequences, quality requirements, and maintenance obligations.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Conserve soil mass, not apparent volume

Excavated soil usually occupies a different volume from undisturbed ground, and compacted fill differs again. Construction manuals explicitly distinguish these states; counting all three as interchangeable produces incorrect excavation and transport budgets. [FAOHome](https://www.fao.org/4/AC014E/AC014E08.htm)

Represent:

* **Bank volume:** soil in its original, undisturbed position.
* **Loose volume:** excavated material in baskets, carts, or spoil piles.
* **Compacted volume:** material placed in a finished embankment or platform.

The authoritative quantity should be dry soil mass:

\[
M\_s=\rho\_bV\_b=\rho\_lV\_l=\rho\_cV\_c
\]

Here, the densities are **dry bulk densities**, including pore space. Track water separately: wet earth burdens carriers more heavily without supplying additional structural soil.

**Implementable rule:** excavating one location creates a material stockpile and a depression. Filling another location consumes that stockpile. Excavation never creates “free” construction material without a borrow pit, quarry face, ditch, or other visible source.

Keep fertile topsoil separate from subsoil and structural fill. A level agricultural plot can become less productive if construction buries or removes its growing layer; FAO terrace guidance explicitly budgets additional work for topsoil preservation. [FAOHome](https://www.fao.org/4/AD083E/AD083e07.htm)

### 1.2 Construction is a sequence of operations

Use a dependency graph:

**Survey → clear → strip topsoil → loosen/excavate → load → transport → spread → compact → finish/protect → test.**

Not every project needs every operation. Soil thrown directly beside a shallow ditch may need no separate hauling stage. A levee requires selected fill, repeated placement, compaction, and hydraulic continuity. Road-building handbooks likewise separate excavation, loading, transport, spreading, and supporting activities. [Scribd](https://www.scribd.com/document/87035402/Construction-Hand-Book)

For a project:

\[
L\_{\mathrm{total}}
=
\sum\_j \frac{V\_j}{q\_j}
+
L\_{\mathrm{survey}}
+
L\_{\mathrm{support}}
+
L\_{\mathrm{rework}}
\]

The volume basis must match each rate: bank, loose, or compacted.

For a crew working as a production line:

\[
Q\_{\mathrm{crew}}=\min(Q\_{\mathrm{dig}},Q\_{\mathrm{haul}},Q\_{\mathrm{place}},Q\_{\mathrm{compact}})
\]

Convert each stage to a common mass or finished-volume basis before taking the minimum.

**Consequences for agents:** extra diggers eventually produce queues and spoil piles, not faster completion. More haulers help only when hauling is the bottleneck. Narrow working fronts prevent hundreds of people from completing a small excavation instantaneously.

### 1.3 Distance and elevation create transport costs

FAO describes baskets, pans, sacks, shoulder poles, wheelbarrows, carts, and animal-drawn scrapers. Direct throwing is practical only over a few metres; wheelbarrows on soft ground may require prepared tracks or planks. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e04.htm)

Calculate hauling from a cycle:

\[
Q\_h=
\frac{V\_{\mathrm{load}}T\_{\mathrm{available}}}
{t\_{\mathrm{load}}+t\_{\mathrm{loaded\ travel}}+
t\_{\mathrm{unload}}+t\_{\mathrm{return}}+t\_{\mathrm{queue}}}
\]

Avoid double-counting loading labor when a separate loading crew performs it.

**Implementable rule:** terrain affects transport through slope, footing, route width, congestion, and load capacity—not just straight-line distance. A shallow roadside borrow pit can be cheaper than obtaining better material from far away, but that pit may undermine the road or collect stagnant water.

Animal traction should unlock a different transport operation, not multiply human excavation speed. The animal, driver, scraper or cart, prepared route, fodder, and water all remain necessary inputs.

### 1.4 Soil condition matters as much as the tool label

Use separate properties for:

**Excavation resistance, moisture, stone content, cohesion, permeability, compactability, and erodibility.**

A soil that is easy to excavate may make poor water-retaining fill. Very dry clay can be difficult to cut; excessively wet soil may be easy to disturb but difficult to handle and compact. FAO recommends conditioning moisture and placing embankment material in thin layers rather than depositing large heaps. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e06.htm)

Tool condition should affect both output and interruptions. In small Ghanaian field trials reported by ILO, badly worn tools increased ditching task time by **22%** and slope-trimming time by **6%**. These are task-specific observations, not a universal “worn tool” multiplier. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40ed_emp/%40emp_policy/%40invest/documents/meetingdocument/wcms_asist_6853.pdf)

**Implementable rule:** model edge wear, broken handles, replacement availability, and material suitability. Do not assign one global productivity multiplier to “stone,” “bronze,” or “iron.”

### 1.5 Geometry determines the minimum work

The following are useful analytical estimates. They are **derived geometry**, not historical labor observations.

| Earthwork | Idealized volume | Important limitation |
| --- | --- | --- |
| Trapezoidal ditch | \(V=Ld(b+zd)\) | \(b\): bottom width; \(d\): excavation depth; \(z\): horizontal side slope per unit vertical |
| Symmetrical levee | \(V=L(HC+zH^2)\) | \(H\): height; \(C\): crest width; excludes foundation preparation and settlement allowance |
| Balanced leveling of a rectangular plot | \(V\_{\mathrm{cut}}\approx AsW/8\) | Uniform slope \(s\), downslope width \(W\), equal-density cut/fill approximation |
| Full-bench road cut | \(V\_{\mathrm{cut}}\approx LsW^2/2\) | Idealized triangular cut; extra excavation for battered faces, drains, and unstable material is excluded |
| Bench terracing | Sum the individual bench cut/fill sections | Include riser footprints, retaining structures, and actual soil depth |

For leveling, **cut volume is not cut-plus-fill volume**. Moving 100 m³ from the uphill half to the downhill half does not create 200 m³ of material, although excavation and placement both require labor.

A wider road or terrace becomes disproportionately expensive where it demands deeper cutting. A higher levee also becomes expensive quickly because its sloping sides contribute an \(H^2\) term.

### 1.6 A canal is a hydraulic connection, not merely a trench

For ordinary open-channel flow, a useful capacity approximation is Manning’s equation:

\[
Q=\frac{1}{n}AR^{2/3}S\_f^{1/2}
\]

where \(A\) is wetted area, \(R\) hydraulic radius, \(n\) roughness, and \(S\_f\) energy gradient. Bottom slope is only an approximation to the energy gradient under suitable flow conditions; a horizontal-bottom canal can still convey water when its upstream water level is higher. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm)

**Implementable rules:**

A canal needs an intake, connected reaches, sufficient head, and an outlet or demand sink. A drainage ditch works only when it can discharge somewhere lower in hydraulic head. Embanking a lowland without providing drainage can trap rainfall behind the levee.

Separate **construction depth** from **water depth**. Freeboard occupies excavated or constructed height without contributing normal conveyance.

### 1.7 Terraces have two fundamentally different construction pathways

**Excavated benches:** move enough soil immediately to establish a planting surface.

**Progressive terraces:** build contour barriers, then allow cultivation and trapped sediment to reshape the intervening slope over years. Kenyan *fanya juu* terraces are an explicit example: soil is thrown uphill from a trench to form a bund, and subsequent soil movement gradually produces benches. [FAOHome](https://www.fao.org/4/x5301e/x5301e0a.htm)

**Implementable rule:** offer a low-upfront-cost, slow-maturing terrace option alongside expensive immediate bench construction. Their final appearance may converge, but their investment schedules and early performance differ.

---

## 2. Parameters and construction budgets

### Evidence conventions

**N — planning norm:** a practical engineering estimate, not necessarily a controlled measurement.  
**E — empirical observation:** measured in a particular study.  
**R — reconstruction:** inferred from archaeological evidence.  
**P — proposed TCE prior:** an initialization value requiring calibration.

Confidence refers to transferability: **high** for the stated scope, **medium** for use in comparable settings, **low** for broad historical extrapolation.

A worker-day is not universally eight hours. FAO’s excavation and bench-construction rates below explicitly use eight-hour days; other sources retain their published daily-task units.

### 2.1 Hand excavation by ground condition

FAO’s table describes an average-strength worker **digging and throwing soil approximately 1 m**. Its hoe and pick/shovel columns are lower- and higher-output planning cases—not controlled experiments isolating tool effects. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e12.htm)

| Ground | Hoe case, m³/8-hour worker-day | Pick/shovel case, m³/8-hour worker-day | Evidence and confidence |
| --- | --- | --- | --- |
| Soft alluvium or sand | 2.5–3.0 | 3.5–4.0 | N; medium transferability |
| Moderately hard loam/light clay | 1.5–2.0 | 2.5–3.0 | N; medium |
| Hard, heavier clay | 1.0 | 2.0–2.5 | N; medium |
| Moderately hard laterite | 0.5 | 1.0–1.5 | N; medium |
| Water-saturated ground | 0.8–1.5 | 1.5–2.0 | N; medium; access and water handling remain important |

These are useful analogues for hand-tool construction. **They are not direct measurements of prehistoric wooden, antler, or stone implements.**

### 2.2 Transport, completed operations, and machinery

| Operation or equipment | Published value | Scope; confidence |
| --- | --- | --- |
| Excavate overburden and load wheelbarrow | **2–4 m³/worker-day** | Zambia road-work norm; excludes hauling; N, medium. [Scribd](https://www.scribd.com/document/87035402/Construction-Hand-Book) |
| Wheelbarrow haul, 0–40 m | **10.5 m³/worker-day** | Transport task, not excavation; N, medium. [Scribd](https://www.scribd.com/document/87035402/Construction-Hand-Book) |
| Wheelbarrow haul, 40–60 m | **8 m³/worker-day** | Same task family; N, medium. [Scribd](https://www.scribd.com/document/87035402/Construction-Hand-Book) |
| Wheelbarrow haul, 80–100 m | **5.5 m³/worker-day** | Same task family; N, medium. [Scribd](https://www.scribd.com/document/87035402/Construction-Hand-Book) |
| Ordinary wheelbarrow capacity | **40–60 litres** | Equipment capacity, not daily output; N, medium. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e04.htm) |
| Excavate and accurately form canals | **0.8–1.2 m³/worker-day** | Trained worker; shaping included; N, medium. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e12.htm) |
| Hand-built bench terraces | **3–4 m³/8-hour worker-day** | Supervised cut-and-fill operation; N, medium. [FAOHome](https://www.fao.org/4/AD083E/AD083e07.htm) |
| Preserve topsoil during bench construction | **+40 worker-days/ha** | Additional operation; N, medium. [FAOHome](https://www.fao.org/4/AD083E/AD083e07.htm) |
| *Fanya juu* terraces plus cutoff drain | **150–350 worker-days/ha** | Kenyan scheme; N, medium. [FAOHome](https://www.fao.org/4/x5301e/x5301e0a.htm) |
| Leveled rainwater-harvesting gardens using oxen scoops | **500–1,000 m³/ha; 250–500 worker-days/ha** | Whole land-treatment operation, not scraper-only output; N, medium. [FAOHome](https://www.fao.org/4/x5301e/x5301e0a.htm) |
| 40 hp bulldozer: excavation and short push | **13–17 m³/machine-hour** | Favorable conditions, haul/push ≤50 m; N, medium. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e12.htm) |
| 130 hp bulldozer: same operation | **46–71 m³/machine-hour** | Same restrictions; excludes the wider support economy; N, medium. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e12.htm) |

Two cautions matter.

First, the canal-forming rate and the faster terrace rate are **different operations**, not contradictory estimates of one universal digging speed.

Second, machine-hours are not total person-hours. An operator’s apparent output omits fuel production, maintenance, machine manufacture, transport, and supporting crews. Industrial construction should redistribute labor across an economy, not remove it from the simulation.

### 2.3 Geometry, handling, and hydraulic parameters

| Parameter | Value or range | Interpretation; confidence |
| --- | --- | --- |
| Dry, loose earth bulk density | **1,300–1,500 kg/m³** | Broad materials-planning value; do not combine with wet density to infer compaction; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e03.htm) |
| Thin layers for hand-compacted earthen fill | **0.15–0.20 m** | Compaction performed repeatedly during construction; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e06.htm) |
| Hand tamper mass | **4–6 kg maximum** in the cited guidance | One practical equipment design, not a historical universal; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e06.htm) |
| Small earthen canal bottom gradient | Approximately **0.05–0.2%** in the cited guidance | Topography- and erosion-dependent, not an absolute allowable range; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm) |
| Canal side slope, ordinary earth | **1.5 horizontal : 1 vertical** | Planning guidance; soil-specific; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm) |
| Canal side slope, light sand/wet clay | **3 horizontal : 1 vertical** | Flatter sides consume more land and excavation; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm) |
| Manning roughness, recently finished smooth earth | **0.017 s/m⅓** | Hydraulic estimate; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm) |
| Manning roughness, average unlined canal | **0.025 s/m⅓** | Hydraulic estimate; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm) |
| Manning roughness, badly maintained and densely weeded | **0.040 s/m⅓** | Hydraulic estimate; N, medium. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm) |

Do not apply the canal side-slope values directly to levees, retaining-wall terraces, or high road cuts. Their loading, foundations, and failure mechanisms differ.

### 2.4 Proposed starting values where historical evidence is thin

The following are **deliberately labeled calibration priors**, not archaeological measurements.

| TCE parameter | Initial value | Sensitivity range | Scope |
| --- | --- | --- | --- |
| Non-metal digging kit, workable soft-to-firm soil | **1.0 m³ bank/8-hour worker-day** | **0.5–2.0** | P, low confidence; exclude bedrock and strongly cemented ground |
| Spread and hand-compact prepared fill | **3 m³ compacted/worker-day** | **2–5** | P, low; excludes excavation and hauling |
| Basket/pan working payload | **20 kg** | **15–25 kg** | P, low; individual ability, footing, slope, and carrying method modify it |
| Loose-volume/bank-volume ratio | **1.20** | **1.10–1.40** | P; replace with material-specific values |
| Compacted-volume/bank-volume ratio | **0.90** | **0.85–1.00** | P; assumes suitable mineral fill and a chosen compaction target |

The numerical ratios are suggested TCE bounds. The underlying requirement to distinguish original, expanded, and compacted volumes is supported by construction guidance. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e06.htm)

For non-metal tools, use overlapping distributions rather than an inevitable technology ladder: a suitable wooden digging tool in workable soil need not be worse than an unsuitable metal implement.

### 2.5 Worked construction examples

These examples are calculations under stated assumptions.

#### A. A small irrigation or drainage ditch

Take a 100 m reach with a 0.4 m bottom, 0.4 m excavation depth, and 1.5:1 sides:

\[
V=100(0.4)(0.4+1.5\times0.4)=40\text{ m}^3
\]

At the cited canal-excavation-and-forming rate, this is approximately **33–50 worker-days**. Intake works, crossings, long-distance spoil disposal, and bank construction are additional.

#### B. Leveling a building plot

A 20 × 20 m plot on a uniform 10% slope requires approximately:

\[
V\_{\mathrm{cut}}=400(0.10)(20)/8=100\text{ m}^3
\]

That is the idealized uphill cut, transferred downhill. Topsoil removal, structural compaction, retaining walls, and density differences must still be handled.

The implication is important: leveling an apparently modest urban parcel can consume substantial labor. Buildings should often adapt to slopes instead.

#### C. A 100 m village levee

Assume height 1.5 m, crest width 1 m, and 2:1 sides:

\[
V\_c=100[(1.5)(1)+2(1.5)^2]=600\text{ m}^3
\]

For this **illustrative TCE scenario**, assume:

* \(V\_c/V\_b=0.90\): approximately **667 m³ bank** required.
* \(V\_l/V\_b=1.20\): approximately **800 m³ loose** handled.
* Digging/loading: 3 m³ bank per worker-day.
* Hauling: 8 m³ bank-equivalent per worker-day.
* Placement/compaction: the proposed 3 m³ compacted per worker-day.

The labor budget is:

| Stage | Worker-days |
| --- | --- |
| Dig and load | 222 |
| Haul | 83 |
| Spread and compact | 200 |
| **Subtotal** | **506** |

Twenty perfectly balanced workers imply a lower bound of approximately **25 working days**. A fixed crew of nine diggers, three haulers, and eight placement workers is haul-limited and needs approximately **28 working days**, even before surveying, clearing, foundation treatment, weather, and interruptions.

This is why “600 m³ divided by a digging rate” is not a levee construction budget.

#### D. A terrace benchmark

FAO’s worked example specifies **719 m³ for 0.5 ha of 3.5 m-wide benches on a 30% slope**. Applying its construction range and topsoil allowance gives approximately **200–260 worker-days per 0.5 ha**, or **400–520 per hectare**, for that particular design. [FAOHome](https://www.fao.org/4/AD083E/AD083e07.htm)

Treat this as a calibration case, not a universal terracing price.

---

## 3. Variation across eras and regions

Chronological eras should be descriptive labels, not prerequisites. Earthmoving capacity depends on tools, transport, labor coordination, food provision, ground conditions, and accumulated construction knowledge.

| Setting | Evidence and construction pattern | Implication for TCE |
| --- | --- | --- |
| **Forager/fisher societies: lower Mississippi Valley** | Poverty Point’s Mound A contains an estimated **238,500 m³**. Its builders lacked draft animals and wheelbarrows. Stratigraphic research argues for rapid construction, but the proposed weeks-to-months duration is an inference, not an observed work record. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/gea.21430) | Large earthworks cannot require agriculture, metallurgy, or a centralized state as hard prerequisites. Aggregation, provisioning, and shared purpose can mobilize large crews. |
| **Aquaculture societies: southeastern Australia** | Gunditjmara engineering at Budj Bim redirected water using channels, weirs, dams, and local volcanic stone over a sequence extending at least **6,600 years**. This is not simply an early version of cereal irrigation. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1577/) | Water-management projects should support fishing and wetland production, not only crop fields. |
| **Early farming: New Guinea highlands** | Kuk preserves a long sequence from mounded cultivation to drainage systems excavated with wooden tools. UNESCO describes agricultural use over roughly seven to ten millennia; that does not mean every surviving ditch has that age. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/887/) | Drainage can be an early agricultural technology. Do not require metal spades or state surveyors. |
| **Ancient agrarian states: Mesopotamia** | Ur III evidence documents corvée labor and its administrative organization. Such records illuminate mobilization and obligations, rather than establishing a universal ancient digging rate. [De Gruyter Brill](https://www.degruyterbrill.com/document/doi/10.1515/9781575068718-021/html) | Track assessed labor, attendance, provisioning, exemptions, and competing seasonal work—not a “state construction bonus.” |
| **Long-lived river engineering: China** | Dujiangyan originated around **256 BCE** and was repeatedly modified. Its engineering coordinates diversion, sediment removal, flood management, and flow control. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1001) | A successful project can exploit river form and sediment pathways rather than maximize embankment height. Long-lived infrastructure should accumulate modifications. |
| **Precolonial Andes** | Field investigations at Machu Picchu show that foundations, site preparation, drainage, and retaining structures were essential to construction on steep, rainy terrain. The evidence also includes failures and repairs, not universal perfection. [ASCE Library](https://ascelibrary.com/doi/10.1061/%28ASCE%29SC.1943-5576.0000146) | Andean-style terracing needs stonework and drainage recipes alongside earthmoving. Do not price it as loose soil alone. |
| **South and Southeast Asian irrigation communities** | Bali’s irrigation coordination has been modeled through interacting farmer organizations. Comparative research on **100 irrigation systems in Nepal and Thailand** found that institutional principles were useful but needed contextual qualification. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.1.02a00050) | Permit local associations, nested organizations, and customary rights to operate substantial systems without making state management inevitable. |
| **East African conservation agriculture** | Kenyan examples combine contour trenches, vegetated bunds, gradual bench formation, oxen scoops, and manual placement. These documented modern practices are analogues, not proof of an unchanged ancient tradition. [FAOHome](https://www.fao.org/4/x5301e/x5301e0a.htm) | Offer multiple affordable treatments: grass strips, bunds, trenches, progressive terraces, and fully excavated benches. |
| **Industrial earthmoving: Panama** | Culebra Cut combined drilling and blasting, steam shovels, railway spoil transport, unloaders, spreaders, and track shifting. Excavation machinery was only one component of the production system. [Autoridad del Canal de Panamá](https://pancanal.com/en/culebra-cut/) | Industrial technology should unlock coordinated machine operations and logistical dependencies, not an enormous multiplier applied to any lone worker. |
| **Modern construction** | Published FAO guidance includes both manual and mechanized methods, with equipment-specific outputs and compaction requirements. Hand labor and machinery remain alternatives within the same technical system. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e06.htm) | Selection should depend on project scale, access, wages, capital, fuel, repair capacity, and urgency—not merely an era flag. |

A useful regional distinction is **what water management is trying to accomplish**. A wetland drainage system removes excess water; a dryland harvesting bund retains runoff; an irrigation canal delivers water; a flood levee excludes it. Visually similar earthworks can therefore require opposite operating rules.

---

## 4. Maintenance, failures, and stylized facts

### 4.1 Maintenance should remove physical problems

Do not assign earthworks a single age-based durability bar. Track deterioration in its actual form.

| Asset | State that accumulates | Maintenance operation |
| --- | --- | --- |
| Canal or drain | Sediment, vegetation, collapsed bank material | Dredge, weed, reshape, repair intake |
| Terrace | Rills, damaged risers, blocked outlets, lost topsoil | Patch, stabilize, clear outlets, replace soil |
| Levee | Crest settlement, erosion, animal damage, seepage defects | Inspect, refill, compact, repair protection |
| Road cut/platform | Slumps, washouts, blocked drains, differential settlement | Clear, regrade, drain, stabilize |
| Borrow pit | Unstable margins, unwanted water storage, lost access | Regrade, fence, drain, or repurpose |

FAO terrace guidance emphasizes inspection following heavy rain and particular attention during the first **two to three years**, when newly constructed works are becoming established. [FAOHome](https://www.fao.org/4/AD083E/AD083e07.htm)

For sediment:

\[
S\_{t+1}=S\_t+\text{deposition}-\text{dredging}-\text{remobilization}
\]

Then update the cross-section and flow resistance.

**Illustrative maintenance calculation:** a 1 km channel accumulating an average 0.05 m of sediment across a 1 m effective deposition width contains 50 m³ to remove. At an assumed clearance rate of 2 m³ per worker-day, that creates 25 worker-days of work. The deposition depth and clearance rate here are scenario inputs, not universal annual norms.

This representation lets maintenance costs respond to watershed erosion, floods, upstream land clearance, and the organization’s ability to recruit workers.

### 4.2 Levee failure is not equivalent to overtopping

Important modes include:

**Overtopping erosion.** Water crosses the crest and erodes the landward side; erosion may enlarge a breach.

**Internal erosion or piping.** Seepage removes particles through or beneath the embankment, potentially causing failure before water reaches the crest.

**Slope or foundation instability.** Saturation, weak foundations, unfavorable geometry, and erosion at the toe can cause movement.

USACE’s DLBreach documentation distinguishes piping from overtopping and distinguishes erosion behavior in cohesive and noncohesive embankments. [HEC](https://www.hec.usace.army.mil/confluence/cwmsdocs/rasum/6.4/overtopping-erosion-model-71699281.html)

For TCE, separate **loading** from **resistance**:

\[
P(\text{failure during }\Delta t)
=
1-e^{-\lambda(\text{load, duration, material, foundation, damage})\Delta t}
\]

Here \(\lambda\) is a calibrated hazard rate—not a universal historical constant. Damage should persist across events unless repaired.

A particularly useful source is **LLID-OT version 1.0**, containing **487 documented U.S. riverine overtopping events**, including breach/non-breach outcomes and hydraulic, geometric, and geotechnical fields. It supports conditional fragility calibration. It does **not** supply an annual failure probability for ancient levees, and its riverine overflow cases should not be transferred directly to wave overtopping. [doi.org](https://doi.org/10.1038/s41597-025-06349-y?utm_source=chatgpt.com)

### 4.3 Consequences must propagate beyond the damaged segment

Recommended consequences include flood depth and duration, interrupted transport, crop loss, seed-stock loss, building damage, displacement, and casualties dependent on exposure and warning.

Failures can also redistribute water: erosion enlarging one channel may divert flow away from another. Angkor research modeled such network effects and found the possibility of cascading damage under large floods. This is evidence for a mechanism, not proof that one hydraulic disaster single-handedly ended the city. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6192684/)

Later geoarchaeological work indicates gradual decline in parts of Angkor’s urban core and raises the reverse causal possibility: weakening occupation and administration may have reduced maintenance before infrastructure failed. TCE should allow both directions of causation. [DOI](https://doi.org/10.1073%2Fpnas.1821460116)

For scale, Rijkswaterstaat records **22 dyke breaches, six flooded provinces, and 379 fatalities** in the Netherlands’ 1825 flood. These are event consequences, not transferable casualty percentages. [Rijkswaterstaat](https://www.rijkswaterstaat.nl/en/news/archive/2025/02/the-forgotten-flood-disaster-of-1825)

### 4.4 Stylized facts a correct simulation should reproduce

| Testable pattern | Expected simulation behavior |
| --- | --- |
| **Longer hauling reduces completed output** | Under the cited wheelbarrow norms, the 80–100 m transport task produces only about half the daily volume of the 0–40 m task. [Scribd](https://www.scribd.com/document/87035402/Construction-Hand-Book) |
| **Wider and higher works become expensive quickly** | Geometry should generate rising marginal labor costs; a levee’s side-slope contribution grows with height squared. |
| **Maintenance affects capacity before total failure** | Holding geometry and gradient constant, changing Manning \(n\) from 0.025 to 0.040 reduces calculated capacity to **62.5%** of its former value. This is a derived test using the cited roughness values. [FAOHome](https://www.fao.org/fishery/static/FAO_Training/FAO_Training/General/x6708e/x6708e08.htm) |
| **Small investments can mature gradually** | Progressive terraces gain bench form through repeated soil movement rather than appearing complete immediately. [FAOHome](https://www.fao.org/4/x5301e/x5301e0a.htm) |
| **Large works do not prove centralized despotism** | Non-state or situationally coordinated organizations must sometimes mobilize substantial labor. Poverty Point is a major calibration counterexample. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/gea.21430) |
| **Protection can increase future exposure** | Reduced routine flooding can encourage development behind levees, raising losses when protection is exceeded; this is a feedback possibility, not a guaranteed outcome. [HESS](https://hess.copernicus.org/articles/17/3295/2013/) |
| **Inherited infrastructure creates path dependence** | Settlements can reuse, enlarge, or abandon existing works rather than repeatedly choosing an unconstrained optimal layout. |
| **Deferred maintenance can become self-reinforcing** | Lower output or population reduces available maintenance labor, causing further deterioration; migration and institutional recovery must remain possible alternatives. |

---

## 5. Modeling recommendation for TCE

### 5.1 Represent earthworks at three scales

**Terrain cells:** elevation, soil horizons, dry soil mass, moisture, vegetation, and structural compaction.

**Infrastructure segments:** ditch cross-section, canal bed elevation, levee crest, terrace riser, retaining wall, intake, outlet, culvert, and spillway.

**Projects and institutions:** desired geometry, construction stages, material sources, work crews, funding, rights, maintenance duties, inspections, and repair history.

Keep narrow channels and levees as **sub-grid features**. Otherwise a ditch smaller than the terrain-cell width may disappear hydraulically even though the renderer shows it.

The Rust simulation should own the quantities and state. Unreal should render the resulting cuts, fill surfaces, spoil piles, work fronts, water, vegetation, and damage.

### 5.2 Individual agents perform jobs; crews produce structures

Each worker needs a current operation, tool, working position, schedule, carrying capacity, fatigue state, and accumulated relevant skill.

The project allocator should reserve working positions and coordinate complementary jobs. Its accounting must distinguish someone who is digging, carrying, compacting, surveying, repairing tools, or provisioning the crew.

A compact project record could contain:

| Component | Essential fields |
| --- | --- |
| Design | Footprint/segment, target elevations, cross-sections, material specification |
| Quantities | Bank excavation, loose stocks, compacted fill, topsoil reserve |
| Schedule | Stage dependencies, available working fronts, seasonal windows |
| Crew | Workers by operation, equipment, route assignments |
| Quality | Compaction state, foundation treatment, drainage continuity, inspection findings |
| Ownership | Beneficiaries, land rights, water rights, responsible institution |
| Maintenance | Sediment stock, defects, inspection date, outstanding repair jobs |

**Do not let successful task completion imply perfect quality.** A hurried or inexperienced crew can deliver the required visible geometry with inadequate compaction or drainage. Inspection and local knowledge should affect how much of that defect is discovered before use.

### 5.3 Institutions determine who pays and who works

Support several recruitment mechanisms within the same physical model:

**Household improvement:** owner labor, family help, reciprocal assistance, or hired workers.

**Communal irrigation or drainage association:** obligations attached to land, water allocation, or membership.

**Corvée:** assessed work obligations enforced by political authority.

**Paid public works:** wages, contracts, taxes, loans, or other funding.

These should change participation, timing, distribution, and accountability—not the laws of soil mechanics. Comparative irrigation research supports modeling specific rules and local conditions rather than treating “communal” or “state” management as inherently superior. [Digital Library of the Commons](https://dlc.dlib.indiana.edu/dlc/items/651d62bd-5845-45bf-8baa-2ab5208f0d38)

For corvée, charge real opportunity costs. Workers still eat; agricultural work may be delayed; excessive demands can produce evasion, hardship, or political resistance. These are proposed behavioral pathways, not automatic outcomes.

An organization should decide whether to construct or repair using something like:

\[
\text{Expected future benefits}
>
\text{labor opportunity cost}
+\text{materials/equipment}
+\text{maintenance}
+\text{land and downstream losses}
\]

Different households can value the same project differently. Upstream users may obtain most of the water while downstream users bear much of the cleaning burden. A levee can protect one community while shifting water toward another.

### 5.4 Keep capabilities separate from eras

Useful capability nodes include reliable containers, durable digging edges, animal traction, wheeled hauling, contour surveying, selected-fill construction, systematic compaction, retaining masonry, controlled outlets, pumping, and powered excavation.

Use alternative prerequisites. Contour surveying need not require literacy. Compaction need not require metal equipment. Stone-lined channels need masonry knowledge but not reinforced concrete.

Knowledge should be local and transmissible: experienced builders can recognize unsuitable soil, remember past floods, or preserve a successful channel gradient. Failure can alter technique choice and builder reputation without necessarily causing a new law or centralized bureaucracy.

### 5.5 Simplify hydraulics and terrain updates, not accounting

For v1, I recommend:

**Segment-based canal hydraulics**, connected to catchment runoff and field water balances.

**Coarse floodplain storage and routing**, with finer updates around overtopping and breaches.

**Material-specific excavation and erosion classes**, rather than a full geotechnical solver.

**Event-driven maintenance generation**, plus periodic inspections.

**Chunked terrain updates**, rather than modifying the entire world whenever a worker moves soil.

These are architecture recommendations, not a performance benchmark. Profile representative settlements and storm events before committing to update frequencies.

### 5.6 Existing models and games worth borrowing from

| Model or game | Reusable idea | What not to assume |
| --- | --- | --- |
| **Landlab** | Modular terrain, hydrology, erosion, and sediment-process components; useful for offline experiments and reference implementations. [ESurf](https://esurf.copernicus.org/articles/8/379/2020/index.html) | It is a framework, not a complete civilization simulator or a drop-in real-time kernel. |
| **HEC-RAS / DLBreach** | Distinct breach initiation and growth processes; material-sensitive erosion. [HEC](https://www.hec.usace.army.mil/confluence/cwmsdocs/rasum/6.4/overtopping-erosion-model-71699281.html) | Full engineering fidelity is unnecessary for every village bund. |
| **Lansing–Kremer Bali model** | Local coordination, irrigation networks, and ecological feedbacks; a NetLogo replication is available through CoMSES. [CoMSES Net](https://www.comses.net/codebases/2221/releases/1.2.0/) | Its self-organization results depend on decision and pest-dynamics assumptions, as Janssen’s reanalysis demonstrates. [Digital Library of the Commons](https://dlc.dlib.indiana.edu/dlc/items/67e0036f-b08d-48e7-a6b4-10d050e5fca4) |
| **Angkor infrastructure model** | Flow redistribution and cascading damage in an evolving network. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6192684/) | Network vulnerability alone does not establish a complete explanation of historical decline. |
| **Di Baldassarre et al.’s socio-hydrology model** | Feedback among flood experience, settlement, memory, and defenses. [HESS](https://hess.copernicus.org/articles/17/3295/2013/) | Its coefficients are conceptual-model choices, not universal behavioral constants. |
| **Timberborn** | Legible water engineering, dams, floodgates, canals, and terrain modification. [Steam Store](https://store.steampowered.com/app/1062090/Timberborn/%3Fl%3Drussian%26mobile%3D2) | Use it as a presentation and interaction reference—not a source of historical labor costs. |

---

## 6. Sources, datasets, and remaining uncertainty

### Most useful source families

| Source | Best use in TCE |
| --- | --- |
| **FAO construction and watershed manuals** | Scoped excavation rates, terrace geometry, small-channel hydraulics, handling methods, and compaction sequences. Their tables are practical norms, not cross-cultural historical averages. [FAOHome](https://www.fao.org/fishery/docs/CDrom/FAO_Training/FAO_Training/General/x6708e/x6708e12.htm) |
| **ILO, *Labour-based Technology: A Review of Current Practice* (1997)** | Observed tool-wear effects and the organization of labor-based construction. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40ed_emp/%40emp_policy/%40invest/documents/meetingdocument/wcms_asist_6853.pdf) |
| **Zambia, *Contractor’s Handbook: Labour-Based Road Works*, second edition (2004)** | Separable excavation/loading/hauling tasks and construction management; the cited copy reproduces the handbook. [Scribd](https://www.scribd.com/document/87035402/Construction-Hand-Book) |
| **WOCAT Global Sustainable Land Management Database** | Site-specific conservation practices, costs, maintenance, benefits, and institutional arrangements. Prefer individual case records to a pooled universal average. [WOCAT](https://wocat.net/en/faq/) |
| **Flynn, Vahedifard, and Schaaf, LLID-OT** | Open event-level data for riverine overtopping and breach calibration, with documented limitations and a CSV data dictionary. [doi.org](https://doi.org/10.1038/s41597-025-06349-y?utm_source=chatgpt.com) |
| **Archaeological investigations and reconstructions** | Project geometry, construction sequences, repairs, landscape context, and competing explanations of organization. Labor rates require a separate evidential step. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/gea.21430) |

### Claims to keep explicitly uncertain

**Pre-metal excavation rates.** The verified practical norms do not establish a broadly transferable daily output for wooden, antler, or stone digging kits. The proposed range in this report is an initialization prior. Erasmus’s *Monument Building: Some Field Experiments* remains a classic research starting point, but I have not used an unverified rate from it. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/soutjanth.21.4.3629433)

**Archaeological person-day totals.** These often combine estimated volume, assumed output, and an inferred construction period. They are reconstructions, not independent measurements of all three quantities.

**Maintenance percentages.** A fixed annual percentage conceals the difference between a sediment-rich canal, a stable abandoned platform, and a levee damaged by one exceptional event. Model the work generated by physical change.

**Ancient breach probabilities.** Modern event datasets constrain mechanisms and conditional responses; they do not establish ancient annual frequencies without information about flood exposure, construction quality, inspection, and missing failures.

**Institutional causation.** Large hydraulic works do not by themselves demonstrate despotism, and local cooperation does not guarantee reliable maintenance. Likewise, failing infrastructure can cause institutional decline, result from it, or participate in a feedback loop.

### Recommended v1 baseline

Implement hand excavation, containers and hauling routes, gravity-fed ditches, cut-and-fill leveling, progressive bund terraces, retaining-wall terraces, and layered earthen levees.

Make **soil conservation, crew bottlenecks, and maintenance backlogs** fundamental. Those mechanisms give TCE a strong basis for settlements that reshape land incrementally, inherit expensive infrastructure, disagree over its upkeep, and sometimes discover that a cheaply built earthwork was costly after all.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92923-a464-83ea-9161-9afd52f54d56)
