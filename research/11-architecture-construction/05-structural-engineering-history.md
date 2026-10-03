# Pre-modern structural engineering for The Civilization Engine

## Executive conclusion

**TCE should model structural systems, not assign each material a maximum span or height.** A timber beam, a tied timber truss, and a hammer-beam roof use the same broad material but transmit forces differently. Likewise, stone lintels, corbelled roofs, arches, and domes require different checks. Historical masonry analysis particularly emphasizes geometry and equilibrium; timber engineering also requires explicit bending, stiffness, and connection checks. [Springer](https://link.springer.com/article/10.1007/s00004-006-0016-8)

The most useful distinction is between **physical capability, economically practical construction, and builders’ knowledge**. TCE should calculate physical performance from geometry, materials, connections, foundations, condition, and loads. Agents should make decisions using imperfectly understood recipes, observations, budgets, and reputations.

A building can therefore be physically adequate but unnecessarily expensive, apparently adequate but defective, or visibly cracked yet stable. Getty’s adobe experiments demonstrate why cracking and collapse must not be synonymous. [cool.culturalheritage.org](https://cool.culturalheritage.org/jaic/articles/jaic39-01-012.html)

The numerical recommendations below distinguish:

* **Measured/documented evidence:** particular structures, tests, or published reference values.
* **Engineering analogues:** later technical guidance useful for estimating pre-industrial systems, but not evidence of historical practice.
* **Proposed TCE priors:** explicit starting assumptions requiring calibration—not historical averages or construction safety guidance.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Represent the path by which loads reach the ground

A useful structural representation is a graph of roof and floor panels, beams, trusses, columns, wall panels, arches, ties, connections, and foundations.

**TCE rule:** Each loaded element must transfer forces through physically meaningful connections to foundations. Decorative proximity does not create support.

This also prevents confusing a building’s total dimensions with its structural spans. For example, Fengguo Temple’s main hall is approximately **47.6 metres wide across nine bays**; that is not a 47.6-metre unsupported beam. [UNESCO World Heritage Centre](https://whc.unesco.org/en/tentativelists/5803/)

### 1.2 Give different systems different governing failure modes

| Structural system | What makes it work | Checks TCE should perform |
| --- | --- | --- |
| Solid timber beam or lintel | Resistance to bending and shear | Bending, shear, deflection, bearing at supports, defects and decay |
| Timber truss | Triangulation redirects much of the load into axial forces | Member tension/compression, buckling, joint slip and splitting, lateral bracing |
| Post-and-beam frame | Short spans and repeated supports | Beam checks, column buckling, frame racking, joint restraint |
| Stone lintel | Limited resistance to tensile bending | Bending fracture, bearing, flaws; do not substitute stone’s compressive strength |
| Masonry arch or barrel vault | Compression transmitted through curved geometry | Feasible thrust path, crushing, sliding, support spreading and settlement |
| Groin/ribbed vault | Three-dimensional load distribution into walls or piers | Support geometry, concentrated reactions, local mechanisms, connections |
| Dome | Three-dimensional compression, sometimes assisted by ties | Meridional equilibrium, hoop cracking, support movement, ring-tie capacity |
| Corbelled opening or roof | Successive projecting courses, stabilized by their backing | Local bearing, overturning, sliding and block bending |
| Earth wall | Thick, predominantly compressed mass | Eccentric compression, out-of-plane instability, erosion, moisture and connections |
| Bamboo or fibre-connected frame | Lightweight members with carefully managed joints | Splitting, local crushing, lashings, pull-out, decay and bracing |

These checks synthesize timber mechanics, masonry equilibrium analysis, traditional-material guidance, and experimental masonry research. The table is an implementation classification, not a claim that historical builders calculated all these quantities. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/62244)

### 1.3 Treat joints as structural elements

**TCE rule:** A connection has its own capacity, stiffness, and failure mode.

A stronger beam does not repair a weak scarf joint. A mortise changes the member’s net section; a peg can split surrounding wood; a bamboo connection can fail before the culm reaches its nominal material strength. For masonry, mortar and interface behavior can govern failure while the stones remain intact. [FAOHome](https://www.fao.org/4/s1250e/S1250E05.htm)

Consequently, craft skill should improve selection, fit, assembly, and defect detection—not magically increase a material’s intrinsic strength.

### 1.4 Construction is a sequence of temporary structures

**TCE rule:** Check every significant construction stage, not just the finished building.

An unfinished truss may lack bracing; a new arch may depend on centering; an incomplete dome has a different load path from the completed dome. Techniques that avoid conventional centering must be separate recipes, rather than a universal exemption for masonry. Taq Kasra’s pitched-brick vaulting and the revived Nubian-vault method illustrate such specialized construction knowledge. [Iranica Online](https://www.iranicaonline.org/articles/ayvan-e-kesra-palace-of-kosrow-at-ctesiphon/)

Required temporary works should consume materials and labor. Premature removal, interrupted construction, uneven loading, and poor drying or curing can then cause failures naturally.

### 1.5 Foundations and water belong inside the structural model

**TCE rule:** Foundations have bearing resistance, settlement response, and exposure to erosion or scour.

A wall can remain below its compressive strength while differential settlement destabilizes the structure above it. More masonry can improve overturning resistance while worsening foundation demand.

For earth buildings, drainage and maintenance are structural interventions, not cosmetic ones. UNESCO’s assessment of Shibam explicitly identifies flooding, neglected flood-management systems, and water-related deterioration as threats. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/192/)

### 1.6 Separate deterioration from chronological age

**TCE rule:** Update condition from exposure and damage mechanisms, not a universal annual loss of “health.”

For timber, track wetting, drying, biological damage, and persistent deformation. The USDA handbook reports that creep can add approximately the initial elastic deformation over several years under typical conditions, with much greater deformation when green wood dries under load. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62244.pdf)

For the initial implementation, represent decay or erosion through lost effective section, weakened joints, and changed support conditions. Sample hidden defects once when an element is produced; do not re-roll its strength every simulation tick.

---

## 2. Quantitative parameters

### 2.1 Practical span envelopes for initial content generation

**These are proposed TCE planning ranges, not measured global distributions or maximum safe spans.** “Routine” means a locally established recipe with suitable materials. The specialist column assumes better selection, connections, geometry, lifting, and supervision—not merely a higher technology score.

All spans below are approximately **clear distances between supports**. Beam calculations should instead use the appropriate effective span.

| Technique | Routine-generation prior | Specialist-generation prior | Main constraints | Confidence |
| --- | --- | --- | --- | --- |
| Pole or solid-timber beams | 2–5 m | 5–8 m | Section, loading, stiffness, available trees | Medium-low |
| Timber trusses | 6–12 m | 12–20 m | Joint design, tension ties, buckling, erection | Medium-low |
| Bamboo frames | 2–4 m bays | 4–6 m bays | Culm dimensions, connections, bracing | Low |
| Stone lintels | 1–3 m | 3–5 m | Tensile flaws, section depth, lifting cost | Low |
| Corbelled openings/roofs | 1–3 m | 3–6 m | Projection, backing mass, stability | Low |
| Masonry arches | 2–8 m | 8–25 m | Rise, thickness, abutments, foundations | Low |
| Barrel vaults | 2–6 m | 6–15 m | Thrust, sidewalls, construction method | Low |
| Groin/ribbed vault bays | 4–10 m | 10–20 m | Pier layout, lateral restraint, workmanship | Low |
| Masonry domes | 3–8 m diameter | 8–20 m diameter | Profile, thickness, support ring, ties | Low |

The timber ranges have a useful engineering analogue: FAO’s tropical farm-building handbook discusses floors needing additional support beyond roughly **5 m**, lightweight pitched roofs up to about **8 m**, and trusses around **7–16 m**, with larger spans requiring specialist treatment. This is twentieth-century rural engineering guidance, not an archaeological survey. [FAOHome](https://www.fao.org/4/s1250e/S1250E0k.htm)

**Do not enforce the table’s upper values as hard caps.** Exceptional projects should emerge when agents assemble sufficient resources and knowledge and the structural checks succeed.

### 2.2 Documented historical benchmarks

These examples establish achieved dimensions, not the frequency or safety of comparable buildings.

| Example | Technique and period | Documented dimension | What TCE should learn |
| --- | --- | --- | --- |
| Westminster Hall, England | Medieval hammer-beam timber roof, commissioned 1393 | **20.7 m width** without internal columns | Exceptional timber roofing required a specialized system and strengthened masonry supports—not simply longer ordinary rafters. [Parliament UK News](https://www.parliament.uk/about/living-heritage/building/palace/westminsterhall/architecture/the-hammer-beam-roof-/) |
| Zhaozhou Bridge, China | Open-spandrel stone arch, completed 605 CE | Approximately **37 m span**, **7.3 m rise** | Large, relatively shallow arches were possible with sophisticated geometry and construction. [American Society of Civil Engineers](https://www.asce.org/about-civil-engineering/history-and-heritage/historic-landmarks/zhaozhou-bridge) |
| Taq Kasra, Iraq | Sasanian brick vault; exact dating debated | **25.5 m width**, **43.5 m depth** | Distinguish transverse vault span from hall length. Massive tapering supports and pitched-brick construction were integral to the achievement. [Iranica Online](https://www.iranicaonline.org/articles/ayvan-e-kesra-palace-of-kosrow-at-ctesiphon/) |
| Pantheon, Rome | Roman concrete dome | Approximately **43.4 m clear diameter** | Exceptional domes require a system-specific model; cracked behavior and material distribution matter. [Academia](https://www.academia.edu/35359357/On_the_Structure_of_the_Pantheon) |
| Palma Cathedral, Mallorca | Medieval masonry vaulting | Approximately **20 m nave span**, **42 m vault height** | Height and span are separate dimensions, jointly constrained by the support system. [ResearchGate](https://www.researchgate.net/publication/225945912_Galileo_was_Wrong_The_Geometrical_Design_of_Masonry_Arches) |
| Fogong Temple pagoda, China | Timber tower, built 1056 | **67.31 m overall height**, including base and finial | Overall timber-building height is not unsupported column length. Repeated structural levels and connections matter. [UNESCO World Heritage Centre](https://whc.unesco.org/en/tentativelists/5803/) |
| Shibam, Yemen | Multistorey sun-dried-brick housing | Houses reaching **seven storeys** | Earth construction must not have a universal one- or two-storey cap. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/192/) |

A practical initial height distribution could favor **one to two storeys for ordinary earth housing**, **one to three for timber**, and **one to five for masonry**, while allowing specialist exceptions. Those are **low-confidence content-generation priors**, not physical limits or historical population statistics.

### 2.3 Material properties: preserve the meaning of each number

**Never mix clear-specimen strengths, working stresses, and masonry-assembly properties.**

#### Timber: clear-specimen measurements

These USDA values are mean properties of small, relatively defect-free specimens. They are not appropriate direct failure thresholds for arbitrary full-size historical beams.

| Species and moisture state | Bending rupture strength | Elastic modulus \(E\) | Compression parallel to grain |
| --- | --- | --- | --- |
| Coast Douglas-fir, green | 53 MPa | 10.8 GPa | 26.1 MPa |
| Coast Douglas-fir, 12% moisture content | 85 MPa | 13.4 GPa | 49.9 MPa |
| White oak, green | 57 MPa | 8.6 GPa | 24.5 MPa |
| White oak, 12% moisture content | 105 MPa | 12.3 GPa | 51.3 MPa |

**Confidence:** High for the stated test basis; low for direct transfer to ungraded historic members. Knots, grain deviation, size, joints, sustained loading, and deterioration require additional treatment. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/62244.pdf)

#### Timber: working-stress analogue

FAO gives basic working stresses and separate grade factors. Applying its **grade-2 factor of 0.50 to strength values** gives:

| FAO density group at 12% moisture | Bending working stress | Parallel-compression working stress | Beam shear working stress | Listed \(E\) |
| --- | --- | --- | --- | --- |
| 401–500 kg/m³ | 7.5 MPa | 5.0 MPa | 0.65 MPa | 6.0 GPa |
| 501–640 kg/m³ | 10.0 MPa | 6.5 MPa | 0.95 MPa | 7.5 GPa |

These are useful **design-recipe calibration values**, not physical ultimate strengths. Do not apply the strength grade factor automatically to elastic modulus. **Confidence:** Medium as an engineering analogue; low as evidence of historical allowable stresses. [FAOHome](https://www.fao.org/4/s1250e/S1250E04.htm)

#### Masonry: assembly-level reference ranges

A 2021 experimental paper reproduces reference ranges used for assessing existing Italian masonry:

| Assembly | Compressive strength \(f\_m\) | Elastic modulus \(E\) | Unit weight |
| --- | --- | --- | --- |
| Rubble/irregular stone masonry | 1.0–2.0 MPa | 0.69–1.05 GPa | 19 kN/m³ |
| Fully dressed stone masonry | 2.6–3.8 MPa | 1.50–1.98 GPa | 21 kN/m³ |
| Squared stone-block masonry | 5.8–8.2 MPa | 2.40–3.30 GPa | 22 kN/m³ |
| Solid brick with lime mortar | 2.6–4.3 MPa | 1.20–1.80 GPa | 18 kN/m³ |

These describe **assemblages**, not the crushing strength of individual stones. They are assessment reference bands, not measured universal minima and maxima. **Confidence:** Medium for comparable existing masonry; low for global transfer. [ResearchGate](https://www.researchgate.net/publication/351215864_Experimental_Evaluation_of_Shear_Behavior_of_Stone_Masonry_Wall)

For adobe, a Portuguese experimental study obtained approximately **0.80–1.65 MPa** compression strength from its tested unit specimens. It also tested reconstructed masonry prisms, demonstrating why unit and assembly properties must be recorded separately. [Academia](https://www.academia.edu/121135372/Mechanical_characterization_of_traditional_adobe_masonry_elements)

**Proposed TCE fallback:** For an uncalibrated, dry, unstabilized earthen wall recipe, explore assembly compression strengths of **0.3–2 MPa**, explicitly marked low confidence. Do not transfer that range unchanged to wet earth, rammed earth, cob, or stabilized blocks.

### 2.4 Loads and condition parameters

| Parameter | Value or starting range | Interpretation and confidence |
| --- | --- | --- |
| Occupied dwelling-floor scenario | **1.5–2 kPa** | Proposed load case, not a historical standard. Prefer actual agents and stored goods where available. |
| Crowded assembly-floor scenario | **4–5 kPa** | Proposed stress-test case; local crowd concentrations still matter. |
| Maize stored 1 m deep | Approximately **7.1 kPa** | Calculated from FAO density of 720 kg/m³. High confidence for this assumed bulk density. |
| Wheat stored 1 m deep | Approximately **7.6 kPa** | Calculated from 770 kg/m³. Moisture and packing vary. |
| Water ponding 0.10 m deep | **0.98 kPa** | Direct calculation using water density and gravity. |
| Wind dynamic pressure at 20/30/40 m/s | **0.245 / 0.551 / 0.980 kPa** | Calculated at air density 1.225 kg/m³; multiply by appropriate pressure coefficients. |
| Beam deflection screening | **\(L/180\) to \(L/360\)** | Later engineering analogue for serviceability, not collapse or medieval practice. |
| Timber creep coefficient \(\phi\) | Approximately **1** in typical conditions; **4–6** additional deformation during green-wood drying in cited circumstances | Relates creep deformation to initial elastic deformation; not an annual rate. |

Grain densities are from FAO; deflection screening comes from its structural-design guidance; creep behavior comes from USDA. Uncited entries are explicitly proposed scenarios or elementary calculations, not historical observations. [FAOHome](https://www.fao.org/4/s1250e/S1250E0g.htm)

For snow, calculate \(p=\rho\_sgd\) from the weather system’s snow depth and density rather than assigning every snowy roof the same load. For earthen roofs, calculate dead load from actual thickness and density.

### 2.5 Geometry and connections

Two particularly useful empirical anchors are:

| Parameter | Evidence | Appropriate use |
| --- | --- | --- |
| Adobe wall height/thickness ratio \(H/t\) | Getty’s contextual classification ranges from relatively stable thick walls below roughly **4–6**, through increasing vulnerability around **6–9**, to much poorer stability above roughly **9–12** | A vulnerability feature, **not a universal seismic pass/fail rule**. Openings, crosswalls, ties, roof connections and shaking matter. [cool.culturalheritage.org](https://cool.culturalheritage.org/jaic/articles/jaic39-01-012.html) |
| Masonry joint friction/cohesion | Portuguese air-lime rubble-masonry tests fitted approximately **\(\mu=0.558\)** and **\(c=0.0815\) MPa** | One experimental calibration point. Do not generalize intact-joint cohesion to cracked or dry joints. [Academia](https://www.academia.edu/82050229/Experimental_Evaluation_of_the_Shear_Strength_of_Masonry_Walls) |

For an initial cracked-joint model, **\(c=0\)** and a deliberately broad **\(\mu=0.4–0.7\)** sensitivity range are reasonable *proposed priors*. Calibrate by interface type rather than assigning one friction coefficient to all masonry.

---

## 3. How builders knew—and how capability varied

### 3.1 Proportional rules were compressed practical knowledge

Builders did not need modern stress analysis to accumulate useful structural knowledge. Written traditions preserve proportional rules linking spans, supports, members, and building categories.

| Tradition | Documented rule or practice | Simulation interpretation |
| --- | --- | --- |
| Vitruvius, Roman Mediterranean | Discusses column-spacing systems measured in column diameters; warns that stone architraves frequently fail at three-diameter spacing and specifies timber for still wider arrangements | Recipes can include material substitutions and warnings based on geometry, without numerical stress knowledge. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Vitruvius/3%2A.html) |
| European masonry traditions | Historical buttress rules often used fractions of the supported span; Huerta identifies Gothic examples around **one-quarter of span** | Store contextual proportions, not a universal “buttress = 0.25 span” law. [ResearchGate](https://www.researchgate.net/publication/225945912_Galileo_was_Wrong_The_Geometrical_Design_of_Masonry_Arches) |
| *Yingzao Fashi*, China, published 1103 | A systematic building manual using **eight grades of timber modules** | Institutions can standardize compatible dimensions and craft procedures without modern elasticity theory. [JSTOR](https://www.jstor.org/stable/1568644) |
| Andean fibre bridges | Smithsonian documentation of Q’eswachaka records annual renewal and a three-day communal rebuilding process | Structural capability can depend on recurring collective maintenance rather than long-lived materials. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/video/bridge-qeswachaka.html) |

A recipe is most trustworthy near the conditions under which it was learned. Copying a successful roof into a snowier climate, replacing a light covering with earth, or using a different timber species should reduce confidence.

**Recommended knowledge representation:** A builder knows examples and conditional rules—“this joint works with this timber and roof form”—rather than an omniscient structural-margin number.

### 3.2 Proportions do not scale identically for every material

For a fixed-section beam, increasing span greatly increases bending and deflection. Simply enlarging an entire beam-and-load arrangement is not equivalent to preserving safety.

Idealized masonry equilibrium has a different scaling behavior: a geometrically similar compression-only structure can preserve its equilibrium geometry under self-weight, provided crushing and sliding remain non-governing. That is the limited sense in which proportional masonry rules can be structurally rational; it does not make actual masonry infinitely strong. [Springer](https://link.springer.com/article/10.1007/s00004-006-0016-8)

TCE should therefore distinguish **geometric compatibility**, **material-strength reserve**, and **stability under support movement**.

### 3.3 Eras should describe capability combinations, not unlock dates

| Broad setting | Representation recommended for TCE |
| --- | --- |
| **Foraging societies** | Distinguish mobile shelters from settled, resource-rich communities with substantial woodworking. Do not impose a universal “small hut” stage. |
| **Early farming** | Favor locally available earth, poles, short repeated bays, household labor and repairable construction—but permit independent experimentation. |
| **Pre-industrial specialization** | Add craft lineages, selected timber, specialized joints, masonry geometry, lifting systems, temporary works and institutional maintenance. |
| **Industrial production** | Add manufactured sections, more consistent materials, standardized connections, analytical design and organized testing as separate capabilities. |
| **Modern engineering** | Add reliability-based checks, better characterization, reinforcement, engineered timber and more sophisticated analysis; do not make older systems disappear. |

The forager distinction has a concrete counterexample: the National Park Service’s Northwest Coast ethnographic study describes fishing/hunting/gathering communities with substantial post-and-beam plank houses, including buildings around **30 × 60 feet**, approximately **9 × 18 m**. Those are plan dimensions, not unsupported spans. [NPS History](https://www.npshistory.com/publications/olym/prehistory_ethnography/chap4.htm)

For scale comparison rather than a universal chronology, Mark and Hutchinson cite the **113 m** Galerie des Machines roof of 1889 and the **219 m** CNIT concrete-shell span of 1958. These illustrate how new systems changed achievable spans; they are not modern maximum records. [Academia](https://www.academia.edu/35359357/On_the_Structure_of_the_Pantheon)

### 3.4 Regional differences should emerge from constraints and transmission

**East Asia:** Modular timber frames and repeated bays provide a different route to monumental scale from European vaulted churches. Encode modular grading and frame/joint traditions independently of masonry-vault knowledge. [JSTOR](https://www.jstor.org/stable/1568644)

**South Asian mountain regions:** Dhajji dewari combines timber framing with stone and earth infill. A hybrid system should therefore retain separate frame, infill, and connection behavior rather than inherit a single “stone building” vulnerability. [World Housing](https://world-housing.net/tutorials/dhajji-dewari/)

**Western Asia and arid Africa:** Earthen and brick vaulting can reduce dependence on long timber. The modern Nubian-vault program explicitly adapts such knowledge, but its contemporary specifications should not be projected unchanged into antiquity. [Iranica Online](https://www.iranicaonline.org/articles/ayvan-e-kesra-palace-of-kosrow-at-ctesiphon/)

**Arabian urban earth construction:** Shibam demonstrates that dense multistorey settlement can coexist with sun-dried brick when appropriate building and maintenance systems exist. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/192/)

**The Americas:** Northwest Coast timber houses and Andean fibre bridges demonstrate different forms of sophistication: large selected wood in one case, tension systems sustained by collective renewal in another. Neither fits a single European material progression. [NPS History](https://www.npshistory.com/publications/olym/prehistory_ethnography/chap4.htm)

---

## 4. Stylized facts a correct simulation should reproduce

The first four targets follow directly from the reduced mechanics below. The remaining targets are historical or experimental checks.

| Pattern | Quantitative or behavioral target |
| --- | --- |
| Long unsupported beams become expensive rapidly | Doubling span at fixed section and line load produces **4× bending stress** and **16× elastic deflection**. |
| Member depth is structurally powerful | Doubling rectangular-beam depth at fixed width gives **4× section modulus** and **8× second moment of area**. Self-weight must also be updated. |
| Shortening bays is often more economical than finding enormous trees | An added intermediate support can greatly reduce beam demands, although its own column and foundation must carry the new reaction. |
| Shallow arches impose greater horizontal demand | In the simple funicular approximation, halving rise doubles horizontal thrust. |
| Use changes can be more dangerous than ordinary occupancy | One metre of stored wheat adds approximately **7.6 kPa**, several times the proposed dwelling-floor scenario. [FAOHome](https://www.fao.org/4/s1250e/S1250E0g.htm) |
| Cracks do not imply immediate collapse | Getty’s tests show post-cracking stability; some stabilization measures sustained approximately twice the displacement associated with collapse of unretrofitted comparisons. This is experiment-specific, not a universal multiplier. [cool.culturalheritage.org](https://cool.culturalheritage.org/jaic/articles/jaic39-01-012.html) |
| Monumental scale requires systems, not material labels | Permit roughly **20 m timber roofing**, **25 m brick vaulting**, and **40 m-class domes** under exceptional conditions without making them routine housing options. [Parliament UK News](https://www.parliament.uk/about/living-heritage/building/palace/westminsterhall/architecture/the-hammer-beam-roof-/) |
| Maintenance institutions can preserve short-lived structures | Annual rebuilding can sustain a bridge tradition over generations; material longevity and institutional longevity must be separate variables. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/video/bridge-qeswachaka.html) |

A further implementation target is **localized damage with conditional cascades**. A failed beam should redistribute load or drop its supported floor area; it should not automatically delete the entire building. Conversely, loss of a critical support must be capable of producing progressive failure.

---

## 5. Recommended structural-margin model

### 5.1 Store several margins, even when the interface displays one

For element \(i\), failure mode \(m\), and load case \(j\):

\[
R\_{imj}=\frac{C\_{im}}{D\_{imj}}
\]

Here \(C\) is capacity and \(D\) is the corresponding demand. A displayed margin can be \(R-1\): a reserve ratio of 1.4 becomes a margin of \(+0.4\).

Keep three distinct layers:

**Physical reserve:** Calculated using the element’s sampled properties and actual condition.

**Builder-estimated reserve:** Based on imperfect observations, recipes and expected loads.

**Serviceability reserve:** Deflection, excessive movement, joint slip, or unacceptable cracking.

**Crucially, exceeding a working-stress allowance is not identical to physical collapse.** Working stresses already incorporate conservatism and other adjustments. TCE must not use the FAO working values as ultimate strengths and then interpret \(R<1\) as certain breakage.

A margin also is not a failure probability. Converting it into probability requires distributions of uncertain strength, loads, geometry, and model error.

### 5.2 Timber beams: a cheap analytical model

For a simply supported rectangular beam:

* \(L\): effective span;
* \(b,h\): section width and depth;
* \(s\): tributary width;
* \(p\): distributed area load excluding the beam itself;
* \(\rho\): timber density;
* \(E\): elastic modulus.

Using consistent SI units:

\[
w=ps+\rho g b h
\]\[
M\_{\max}=\frac{wL^2}{8},
\qquad
V\_{\max}=\frac{wL}{2}
\]\[
Z=\frac{bh^2}{6},
\qquad
I=\frac{bh^3}{12}
\]\[
\sigma\_b=\frac{M\_{\max}}{Z},
\qquad
\tau\_{\max}=\frac{1.5V\_{\max}}{bh},
\qquad
\delta=\frac{5wL^4}{384EI}
\]

These standard relations suit simple beam bays; other support conditions and concentrated loads require their corresponding equations. FAO provides an accessible engineering treatment. [FAOHome](https://www.fao.org/4/s1250e/S1250E0d.htm)

For sustained gravity loading, an initial approximation is:

\[
\delta\_{\text{long}}
=
(1+\phi)\delta\_G+\delta\_Q
\]

where \(G\) is sustained load and \(Q\) the transient portion. Do not apply years of creep to a momentary crowd peak.

**Worked check.** Recalculating a FAO-style example with a **100 × 225 mm** beam, \(E=8.4\) GPa, density 500 kg/m³, 1.2 m tributary width, and 2.5 kPa area load:

| Effective span | Bending stress | Elastic deflection | Ratio of 8 MPa working allowance to demand |
| --- | --- | --- | --- |
| 4 m | 7.37 MPa | 13.0 mm | 1.09 |
| 6 m | 16.59 MPa | 65.8 mm | 0.48 |

These are calculations, not experimental results. The six-metre version is unsatisfactory against that working allowance and stiffness screen; the table alone does not establish its ultimate collapse load.

For columns and truss compression members, add buckling:

\[
N\_{\mathrm{Euler}}=\frac{\pi^2EI}{(KL\_c)^2}
\]

Treat this as an ideal reference, reducing it through an appropriate imperfection/connection model rather than allowing perfectly straight, perfectly restrained columns by default. USDA covers these distinctions in its wood engineering material. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/62244)

### 5.3 Masonry: equilibrium first, then crushing and sliding

For an arch approximately funicular under uniform load \(w\) per horizontal metre:

\[
H\approx\frac{wL^2}{8f}
\]

where \(f\) is rise. This is a useful screening relation, not an exact solution for every circular arch or loading arrangement.

For TCE, use a small block-equilibrium model or a validated lookup table for each arch family. Ask whether a compressive force path exists within the structure while satisfying finite compression and interface-friction constraints.

**Do not confuse the middle-third rule with collapse.** For an ideal rectangular no-tension interface, the middle third corresponds to full compressive contact; partial contact can remain stable beyond it. Compression-only equilibrium methods are the relevant foundation. [Springer](https://link.springer.com/article/10.1007/s00004-006-0016-8)

A useful reduced contact model uses normal force \(N>0\), moment \(M\), eccentricity \(e=M/N\), thickness \(t\), and breadth \(b\):

\[
|e|\le t/6:
\quad
\sigma\_{\max}
=
\frac{N}{bt}
\left(1+\frac{6|e|}{t}\right)
\]

For triangular partial contact:

\[
t/6<|e|<t/2:
\quad
a=3(t/2-|e|),
\qquad
\sigma\_{\max}=\frac{2N}{ba}
\]

At \(|e|\ge t/2\), this simplified interface cannot transmit the required compression-only resultant without additional restraint.

Check sliding separately:

\[
|T|\le\mu N+cA\_{\mathrm{contact}}
\]

This is a Coulomb-type interface approximation; at high compression, crushing and combined failure need to cap its apparent resistance. Experimental masonry research supports separating these mechanisms. [Academia](https://www.academia.edu/82050229/Experimental_Evaluation_of_the_Shear_Strength_of_Masonry_Walls)

For arch capacity, increase a selected variable load while holding permanent load fixed:

\[
G+\lambda Q
\]

Find the limiting \(\lambda\). Scaling every gravity load together in an ideal uncrushable masonry model can preserve the thrust geometry and produce a misleading “capacity” result.

### 5.4 Vaults, domes, hazards, and foundations

For **barrel vaults**, begin with connected arch strips and explicit end/support conditions.

For **groin vaults and domes**, prefer authored structural families with offline validation or simplified thrust networks. Do not model every dome as an intact tensile shell: lower-dome cracking can change its load path. The Pantheon analysis is a useful case study. [Academia](https://www.academia.edu/35359357/On_the_Structure_of_the_Pantheon)

For **foundations**, track bearing demand, effective contact area, and differential settlement. In the first version, settlement can be a low-order soil response rather than a full geotechnical calculation.

For **wind**, apply both pressure and uplift, including connection checks.

For **earthquakes**, a lateral-force surrogate such as \(V=C\_sW\) can drive an initial mechanism screen, but \(C\_s\) must represent effective building response—not simply equal peak ground acceleration. Calibrate collapse vulnerability against building-type observations or tests, using connection quality, geometry, roof mass, and damage state as inputs.

### 5.5 Agents and institutions

A builder’s knowledge record should contain:

| Field | Purpose |
| --- | --- |
| Known recipe and variants | Geometry, materials, joints, construction sequence |
| Validated experience envelope | Previously used spans, loads, heights, climates and soils |
| Observed outcomes | Survival, deformation, repairs, failures and explanations |
| Confidence and uncertainty | Distinguish familiar work from extrapolation |
| Inspection ability | Detect defects, poor fit, movement and moisture problems |
| Social transmission links | Apprentices, migrants, patrons, workshops and institutions |

Institutions can preserve templates, require inspections, organize maintenance, and restrict untested extrapolation. Patrons can reward novelty while accepting—or concealing—risk. A society can consequently possess a technique yet fail to reproduce it after losing skilled workers, suitable timber, finance, or maintenance organization.

These are proposed agent mechanisms. They do not require a global “engineering technology level.”

### 5.6 Computational implementation and useful precedents

For TCE’s scale, use **event-driven structural evaluation**: construction changes, occupancy/storage changes, severe weather, foundation movement, and accumulated deterioration. Cache unchanged gravity solutions. Represent ordinary buildings with macroelements rather than individual bricks.

Rust should own structural state and failure events; Unreal should visualize deformation, cracking, debris, and collapse rather than silently determine authoritative structural outcomes through rendering physics.

| Precedent | What to reuse | What not to assume |
| --- | --- | --- |
| Heyman-style masonry limit analysis, developed in Huerta’s discussion | Compression-only equilibrium and geometric reasoning | That crushing, sliding or foundation movement can always be neglected. [Springer](https://link.springer.com/article/10.1007/s00004-006-0016-8) |
| Block’s Thrust Network Analysis; RhinoVAULT documentation | Three-dimensional compression-equilibrium reasoning and offline form validation | That a gravity-equilibrium solution certifies seismic or construction-stage safety. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/d77d96a9-0193-4d58-981a-119fd787f7f8) |
| Andrew Li’s *Yingzao Fashi* shape grammar | Encoding an architectural tradition as explicit generative rules | That stylistic validity guarantees structural adequacy. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/2fb1fa70-5460-4f85-bd96-3a3efad95817) |
| *Medieval Engineers* | Material-sensitive structural-integrity feedback and visible weak points | That its game implementation is a validated historical engineering model. [Medieval Engineers](https://www.medievalengineers.com/) |

A practical minimum is therefore **analytical beams and columns, explicit joints, compression-only masonry contacts, foundation movement, and staged construction**. Full finite-element analysis is unnecessary for ordinary buildings.

---

## 6. Sources, datasets, and evidence limits

### Core technical references

| Source | Principal use |
| --- | --- |
| **USDA Forest Products Laboratory, *Wood Handbook—Wood as an Engineering Material* (2021), especially Chapter 5** | Timber properties, moisture effects, variability and creep; strong numerical foundation with carefully specified test conditions. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/62200) |
| **Bengtsson and Whitaker, *Farm Structures in Tropical Climates*, FAO/SIDA** | Accessible beam calculations, rural span guidance, timber grading, loads and traditional materials. Use as a later engineering analogue. [FAOHome](https://www.fao.org/4/s1250e/s1250e00.htm) |
| **Santiago Huerta (2006), “Galileo was Wrong: The Geometrical Design of Masonry Arches”** | Historical proportional rules and their relationship to masonry equilibrium. [Springer](https://link.springer.com/article/10.1007/s00004-006-0016-8) |
| **Ginell and Tolles (2000), “Seismic Stabilization of Historic Adobe Structures”** | Experimental distinction between cracking, instability and collapse. [cool.culturalheritage.org](https://cool.culturalheritage.org/jaic/articles/jaic39-01-012.html) |
| **Beconcini et al. (2021), “Experimental Evaluation of Shear Behavior of Stone Masonry Wall”** | Existing-masonry characterization, reference property bands, testing and uncertainty. [ResearchGate](https://www.researchgate.net/publication/351215864_Experimental_Evaluation_of_Shear_Behavior_of_Stone_Masonry_Wall?utm_source=chatgpt.com) |
| **Milošević et al. (2012), “Experimental Evaluation of the Shear Strength of Masonry Walls”** | Mortar-sensitive shear behavior and interface calibration. [Academia](https://www.academia.edu/82050229/Experimental_Evaluation_of_the_Shear_Strength_of_Masonry_Walls) |
| **Mark and Hutchinson (1986), “On the Structure of the Roman Pantheon”** | A detailed example of interpreting a monumental dome through structural analysis. [Academia](https://www.academia.edu/35359357/On_the_Structure_of_the_Pantheon) |
| **Guo (1998), “Yingzao Fashi: Twelfth-Century Chinese Building Manual”; Li (2001), MIT dissertation** | Non-European codified craft knowledge and its translation into a generative grammar. [JSTOR](https://www.jstor.org/stable/1568644) |

For typology-level calibration, the **EERI/IAEE World Housing Encyclopedia** provides regional construction descriptions and earthquake-related weaknesses. It is more useful for identifying structural families than for inferring historical prevalence. [World Housing](https://world-housing.net/)

### What the evidence does not justify

**Universal safe spans or heights.** Historical examples demonstrate achievements, not boundaries. “Largest surviving” is neither “largest attempted” nor “largest reliably reproducible.”

**A global annual collapse rate.** The sources assembled here do not provide a representative denominator across building populations, exposure, maintenance, and observation periods. Do not derive annual failure probabilities from a list of famous failures or survivors.

**Direct comparison of every material test.** Specimen geometry, moisture, mortar, loading protocol, and assembly details can materially change reported properties. Adobe researchers explicitly identify inconsistent testing and material variability as barriers to comparison. [Springer](https://link.springer.com/chapter/10.1007/978-3-030-74737-4_4?utm_source=chatgpt.com)

**A single ladder of technological progress.** The historical cases instead support several viable pathways, with different dependencies on timber, earth, masonry, fibre, labor organization, and maintenance.

**Recommended TCE decision:** Author conditional construction recipes and their failure mechanisms—not universal material span caps. Let geometry and loads determine physical reserve, let agents estimate that reserve imperfectly, and let institutions influence how reliably knowledge is preserved, checked, and maintained.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92910-400c-83ea-9faf-0d2927ab5e4d)
