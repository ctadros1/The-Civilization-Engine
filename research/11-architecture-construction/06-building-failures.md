# Building collapses and failures in history

## A simulation-ready report for The Civilization Engine

**The strongest approach for TCE is to make collapses emerge from structural vulnerability, changing loads, environmental events, and human decisions—not from a fixed annual failure probability attached to an era.** Then model casualties, blame, investigation, repair, and learning as separate consequences.

The central evidential limitation is important: **we do not have defensible, globally representative annual collapse rates for ancient or medieval buildings.** Even modern studies often assemble reported failures rather than observe a complete population over time. Some include structural distress alongside physical collapse; disaster assessments may count buildings as “destroyed” without establishing that they fell down. These distinctions substantially change what can be calibrated. [르네방재정책연구원](https://renetopia.net/wp-content/uploads/2016/12/Analysis-of-Recent-Bridge-Failures-in-the-United-States.pdf)

Nevertheless, the evidence supports a useful model: well-understood physical failure mechanisms, localized quantitative benchmarks, documented institutional responses, and historically plausible—but explicitly uncertain—rules for social learning.

---

## 1. Mechanisms: what makes structures fail?

### 1.1 Separate vulnerability, trigger, and failure propagation

For TCE, record three different explanations for every failure:

**Underlying vulnerability:** inadequate foundations, weak joints, poor geometry, damaged material, an unfinished support system, or insufficient redundancy.

**Trigger:** a crowd arrives, stored goods accumulate, centering is removed, floodwater rises, fire heats a connection, or an earthquake shakes the structure.

**Propagation:** one component fails; its loads transfer, adjoining supports become unstable, and additional components may fail.

This prevents misleading single-cause stories. An earthquake can trigger a collapse whose severity depends on workmanship; an ordinary crowd can expose an inadequate connection. The Hyatt Regency disaster illustrates the latter: a connection-design change doubled the load on critical connections, rather than an extraordinary crowd simply overwhelming an otherwise adequate design. [American Society of Civil Engineers](https://www.asce.org/publications-and-news/civil-engineering-source/civil-engineering-magazine/article/2007/01/the-hyatt-regency-walkway-collapse)

### 1.2 Implementable physical rules

The following are recommended abstractions, not a substitute for engineering calculations on real buildings.

| Mechanism | Rule TCE can implement | Important distinction |
| --- | --- | --- |
| **Foundation failure** | Compare foundation demand with soil-supported capacity. Track differential settlement between supports; let settlement alter wall alignment, arch geometry, and connection forces. | A building can become unstable before its masonry reaches its crushing strength. |
| **Overloading and alteration** | Recalculate loads when floors gain occupants, storage, machinery, additional storeys, or heavier roofs. Removing a wall or enlarging an opening changes the load path. | “The building stood yesterday” does not validate a changed building. |
| **Connection failure** | Give joints, anchors, ties, bearings, and beam ends their own capacities and conditions. | Strong individual members do not guarantee a strong assembly. |
| **Masonry instability** | Check simplified lateral-thrust, overturning, sliding, and support-displacement limits—not only compressive stress. | Arches and vaults need adequate abutments, buttresses, ties, and geometry. |
| **Construction-stage instability** | Evaluate each stage separately. Temporary bracing and centering carry loads until the permanent system is complete; premature removal can initiate collapse. | The completed design may be adequate while an intermediate stage is unsafe. |
| **Moisture and neglect** | Accumulate exposure-dependent damage: wet timber loses effective section through decay; erosion and saturation damage vulnerable earthen components; leaking roofs expose previously protected elements. | Calendar age is not the damage mechanism. |
| **Fire** | Reduce effective member sections and temperature-sensitive capacities; allow floors and roof ties to fail before enclosing walls. Recheck the remaining structure. | Noncombustible walls do not make a building immune to fire-induced collapse. |
| **Flood and scour** | Model water forces, debris impact, saturation, and removal of foundation support. Resolve exposure from a shared flood event. | A bridge can fail through undermining rather than its deck being overloaded. |
| **Earthquake** | Apply shaking to the entire local stock, with vulnerability determined by mass, connections, geometry, ductility, and ground conditions. Retain damage for aftershocks. | Material labels alone are insufficient: detailing can change performance substantially. |
| **Fatigue and corrosion** | For later technologies, accumulate cycle-dependent damage and section loss, especially at connections and exposed metalwork. | Repeated loads below immediate capacity can eventually become consequential. |

Moisture-dependent deterioration is supported by Forest Products Laboratory research; the importance of construction details is demonstrated by experimental and analytical work on timber-and-masonry *dhajji dewari* construction. Both support modeling the actual assembly and exposure, rather than assigning a universal safety ranking to “wood,” “earth,” or “stone.” [US Forest Service R&D](https://research.fs.usda.gov/treesearch/8587)

### 1.3 Human decisions change exposure and consequences

TCE should distinguish **technical knowledge**, **execution**, and **willingness to act**.

A builder may understand an appropriate design but lack suitable timber. A contractor may substitute an easier connection. An owner may defer repair because evacuation eliminates income. An inspector may identify danger but lack authority or access to repair finance.

Consequently, the simulation should allow:

* A visibly distressed structure to remain occupied.
* A dangerous structure to be closed without ever collapsing.
* A technically competent builder to be blamed for an exceptional event.
* A well-connected negligent owner to evade responsibility.

These are proposed agent behaviors. Their institutional basis is illustrated by post-Rana Plaza reforms, which required inspection capacity, funding, training, and remediation—not merely the announcement of safety requirements. [International Labour Organization](https://www.ilo.org/resource/improving-working-conditions-ready-made-garment-industry-progress-and)

---

## 2. Parameters: what can actually be quantified?

**Confidence notation:** **High** means strong support for the stated observation or physical relationship; **Medium** means a retrospective, localized, or model-dependent estimate; **Low** means weak transferability or insufficient evidence. Confidence in a recorded count is not confidence that it represents the whole population.

### 2.1 Failure-frequency evidence

| Population and period | Quantitative finding | What it means—and does not mean | Confidence |
| --- | --- | --- | --- |
| **United States bridges, 1989–2000** | **503 recorded failures** | Incomplete collection covering collapse and distress. Not 503 independently verified total collapses, and not a complete national incidence rate. | High for the collection; low for completeness. [르네방재정책연구원](https://renetopia.net/wp-content/uploads/2016/12/Analysis-of-Recent-Bridge-Failures-in-the-United-States.pdf) |
| **United States buildings, 1989–2000** | **225 recorded failures**; approximately **63% low-rise** | A case collection without a matched building-years denominator. The low-rise share does not establish greater risk per low-rise building. | Medium. [ASCE Library](https://ascelibrary.org/doi/10.1061/%28ASCE%290887-3828%282003%2917%3A3%28151%29) |
| **Nigeria, 1974–2006** | **60 assembled collapse cases**; **52%** attributed to poor materials/workmanship | Retrospective causal classification, not a national census or a causal experiment. Useful evidence about recurring problems, not a universal percentage for Nigerian construction. | Medium for the study; low for generalization. [Jati](https://jati.um.edu.my/index.php/jdbe/article/view/5312) |
| **US bridge-collapse extrapolation, Cook 2014** | Estimated **128 collapses/year**, reported range **87–222/year** | National extrapolation from a 25-year New York dataset; not direct observation of every US bridge collapse. | Medium; geographical transfer is a major limitation. [USU Institutional Repository](https://digitalcommons.usu.edu/etd/2163/) |
| **Nepal, 2015 earthquake sequence** | **604,930 houses destroyed**, **288,856 damaged**, in the cited later assessment | Event-loss categories, not an annual rate; “destroyed” should not automatically become physical collapse in TCE. | Medium–high for the reported assessment; lower for physical-collapse interpretation. [International Recovery Platform](https://recovery.preventionweb.net/publication/impact-2015-earthquake-housing-and-livelihoods-urban-areas-nepal-final-report) |

The US bridge collection classified **266 failures—52.9%—as hydraulic**, including flood, scour, debris, and related mechanisms. Collisions accounted for **59**, overload for **44**, and deterioration for **43**. These are proportions *among recorded failures*, not annual probabilities for exposed bridges. Only about **13%** of its cases appeared in civil-engineering news media, demonstrating how misleading famous-disaster lists can be. [르네방재정책연구원](https://renetopia.net/wp-content/uploads/2016/12/Analysis-of-Recent-Bridge-Failures-in-the-United-States.pdf)

**For ancient and medieval housing, leave the empirical annual baseline marked “unknown.”** Famous collapses establish possibilities and mechanisms; they do not supply the missing denominator.

### 2.2 Useful physical and institutional anchors

| Parameter | Value or range | Recommended interpretation | Confidence/source |
| --- | --- | --- | --- |
| **Timber moisture content and decay** | Below approximately **20%**: generally outside ordinary fungal-decay conditions; above **30%**: decay can occur; **20–30%**: transition region | Moisture content is relative to oven-dry wood mass. Accumulate exposure with temperature and duration; do not make 20% an instantaneous damage switch. | High as a practical boundary; species/environment dependent. Morris & Winandy, 2002. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/8587) |
| **Exposed timber charring benchmark** | Approximately **0.6 mm/min** | A *Wood Handbook* benchmark for Douglas-fir at 7% moisture under standard fire exposure—not a universal rate for every historical fire. | High for the stated benchmark; medium/low transferability. [Yumpu](https://www.yumpu.com/en/document/view/10209038/wood-handbook-front-matter-oregon-wood-innovation-center/422) |
| **Crowd loading: illustrative calculation** | At an assumed **75 kg/person**, **2–5 people/m²** produces approximately **1.47–3.68 kPa** | Calculate actual load from present agents. These are scenario inputs, not historical occupancy estimates or safe occupancy limits. | Arithmetic; assumptions explicitly authored. |
| **Stored-goods loading: illustrative calculation** | **2,000 kg over 2 m²** produces approximately **9.81 kPa** | Inventory placement can matter more than a moderately populated floor. Include floor and container self-weight separately. | Arithmetic; authored scenario. |
| **Historical Roman height restriction** | **70 Roman feet**, approximately **21 m** | Strabo reports an Augustan restriction associated with buildings on public streets. Encode as a jurisdictional rule, not a structural capacity. | High for the textual report; enforcement uncertain. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Strabo/5C%2A.html) |
| **Japanese seismic design provision, 1924** | Horizontal coefficient **0.1 × building weight** | Historical design prescription following the 1923 earthquake; **not** a collapse acceleration threshold. | High for the provision. [Springer](https://link.springer.com/article/10.1007/s10518-017-0290-8) |
| **US bridge inspections under the original 1971 NBIS** | At least once every **2 years** | A historical institutional benchmark. Inspections require trained personnel, records, and follow-up actions. | High; not a statement of every present-day inspection interval. [Federal Highway Administration](https://www.fhwa.dot.gov/highwayhistory/national_bridge_inspection_standards.cfm) |
| **Djenné protective maintenance** | **Annual** plastering campaigns | A demonstrated maintenance cycle for a particular earthen monument; surface renewal does not guarantee deep structural soundness. | High, site-specific. [UNESCO World Heritage Centre](https://whc.unesco.org/en/news/574) |
| **GEM fragility uncertainty components** | Building-to-building logarithmic dispersion **0.30**; damage-state uncertainty **0.30–0.40** in the documented framework | Starting references for uncertainty representation, not universal premodern-building parameters and not percentages. | High for the model specification; uncertain transferability. [OpenQuake](https://docs.openquake.org/vulnerability/fragility_models.html) |

### 2.3 Earthquake damage: conditional benchmarks, not annual rates

EMS-98 provides a useful vocabulary for differentiating weak masonry, stronger masonry, and other structural classes. For masonry, damage grade 5 denotes total or near-total collapse. Its most vulnerable class, A, includes typical rubble-stone and adobe construction, with qualifications and ranges rather than immutable assignments. [GFZ Potsdam](https://media.gfz.de/gfz/sec26/resources/documents/PDF/EMS-98_Original_englisch.pdf)

| EMS intensity | Class A: grade-5 damage | Class B: grade-5 damage |
| --- | --- | --- |
| **VIII — heavily damaging** | “Few” | Not a defining grade-5 outcome |
| **IX — destructive** | “Many” | “Few” |
| **X — very destructive** | “Most” | “Many” |

The scale’s deliberately overlapping quantity ranges are approximately **few: 0–20%**, **many: 10–60%**, and **most: 50–100%**. Use these as broad calibration checks, not precise probabilities. Moreover, intensity is assigned partly from observed damage: validating a damage model against damage-derived intensity can be circular. [GFZ Potsdam](https://media.gfz.de/gfz/sec26/resources/documents/PDF/EMS-98_Original_englisch.pdf)

### 2.4 A transparent sensitivity test for TCE

For ordinary, non-disaster failures, test several *hypothetical* output rates while calibrating the underlying mechanisms:

| Hypothetical rate | Equivalent exposure | Expected failures over 200,000 building-years |
| --- | --- | --- |
| \(10^{-5}\) per building-year | One per 100,000 building-years | **2** |
| \(10^{-4}\) per building-year | One per 10,000 building-years | **20** |
| \(10^{-3}\) per building-year | One per 1,000 building-years | **200** |

**These are sensitivity scenarios, not historical estimates or recommended defaults.** Their purpose is to expose whether a seemingly small probability produces implausible destruction over centuries. Prefer adjusting the mechanisms until the desired rate emerges, rather than imposing the rate directly.

---

## 3. Historical variation and case studies

### 3.1 Variation across technological and social settings

These should be overlapping TCE conditions, not calendar-based unlocks.

| Setting | What TCE should emphasize | Evidential caution |
| --- | --- | --- |
| **Forager societies** | Distinguish light shelters from substantial, long-occupied structures through their actual mass, spans, occupancy, and maintenance. Do not assign one “forager building” archetype. | No defensible general collapse frequency is available here; this is a modeling recommendation, not a quantified historical ranking. |
| **Early farming settlements** | Permanent occupancy, stored goods, heavy roofs, earthen walls, drainage, and rebuilding episodes. | Burned or collapsed archaeological buildings need not represent accidents. Çatalhöyük excavators explicitly debated accidental versus deliberate burning. [Çatalhöyük Research Project](https://www.catalhoyuk.com/archive_reports/2005/ar05_13.html) |
| **Preindustrial towns and states** | Rental incentives, dense occupation, monumental ambition, craft knowledge, customary liability, and selective regulation. | Rome’s literary evidence and Beauvais’s building history document problems and responses, but not representative urban failure rates. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Strabo/5C%2A.html) |
| **Industrializing societies** | New spans and materials, railway and machinery loads, complex contracting, material testing, and public engineering inquiries. | Investigation itself can be contested: the Tay Bridge inquiry produced separate reports reflecting disagreement about its scope. [Nature](https://www.nature.com/articles/022213a0) |
| **Modern societies** | Formal design, inspection, maintenance finance, professional responsibility, and large differences between nominal rules and actual execution. | Advanced engineering knowledge can coexist with organizational failure; Hyatt and Rana Plaza are important counterexamples to automatic progress. [American Society of Civil Engineers](https://www.asce.org/publications-and-news/civil-engineering-source/civil-engineering-magazine/article/2007/01/the-hyatt-regency-walkway-collapse) |

### 3.2 Fidenae, AD 27: a crowded venue, weak construction, and selective regulation

Tacitus attributes the amphitheater disaster to foundations not established on solid ground and insecure timber fastenings. The structure was packed when it failed, with people affected both inside and around it. He reports **50,000 maimed or killed—not 50,000 deaths**. That enormous figure is an ancient narrative claim, not a verified casualty census. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Tacitus/Annals/4E%2A.html)

According to the same account, the Senate required organizers of gladiatorial displays to possess at least **400,000 sesterces**, required amphitheaters to stand on tested solid ground, and banished Atilius. Private households provided shelter and treatment. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Tacitus/Annals/4E%2A.html)

**TCE lesson:** one event can produce emergency aid, punishment, a technical siting rule, and a wealth-based entry restriction. Those responses need not be equally effective: financial eligibility is not the same as construction competence.

### 3.3 Roman insulae: recurring danger did not mean an absence of regulation

Strabo describes rebuilding associated with collapses and fires, but also with property transactions and deliberate demolition. He reports both an Augustan height restriction and organized fire protection. This is evidence that structural and fire danger were recognized—not a basis for estimating a yearly probability that an apartment block collapsed. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Strabo/5C%2A.html)

After the fire of AD 64, Tacitus describes measures including broader streets, limits on building height, open spaces, and revised rebuilding arrangements. The response joined individual-building safety to urban layout. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Tacitus/Annals/15B%2A.html)

**TCE lesson:** model rent-seeking, crowding, repair, and regulation together. Avoid portraying premodern cities as either uniformly unsafe or wholly unregulated. Also separate demolition and redevelopment from accidental structural loss in the historical record and simulation statistics.

### 3.4 Beauvais Cathedral: failure, repair, and continued ambition

The choir was occupied in **1272**, suffered a collapse in **1284**, and underwent rebuilding over roughly the following half-century. The crossing tower collapsed in **1573**, after a much later construction campaign. The cathedral remained incomplete and required subsequent stabilization. [Columbia MCAH Projects](https://projects.mcah.columbia.edu/medieval-architecture/htm/ms/ma_ms_bc_discuss.htm)

The precise mechanics of the medieval collapse remain debated. Competing structural interpretations make “it was simply too tall” or a uniquely proven wind-resonance explanation inadequate summaries. [MDPI](https://www.mdpi.com/2571-9408/8/6/203)

**TCE lesson:** a prestigious failure need not terminate an architectural tradition. Possible outcomes include stronger supports, reduced ambition, renewed ambitious construction, prolonged incompletion, or a monument surviving through repeated intervention. Give structures construction histories, not just completion dates.

### 3.5 Hammurabi: liability could be severe, unequal, and prescriptive

The eighteenth-century-BCE law collection contains explicit builder-liability provisions. Section 229 prescribes execution when defective construction kills the owner; §230 extends retaliation to the builder’s son when the owner’s son dies. Other provisions address compensation, rebuilding at the builder’s expense, and repair of defective walls. Outcomes differ with the victim’s status. [Avalon Project](https://avalon.law.yale.edu/ancient/hamcode.asp)

**TCE lesson:** liability can attach to individuals, households, and social categories—not just modern firms. But the written provisions do **not** establish how frequently courts imposed them or how strongly they deterred defective work. Never convert a severe legal text directly into a large safety bonus.

### 3.6 Lima after 1746: safety reform encountered elite resistance

Charles Walker’s archival study describes upper-class resistance to post-earthquake restrictions on upper storeys. The viceroy’s height-limitation program also formed part of an effort to strengthen colonial authority. What appears technically to be a safety measure was simultaneously a conflict over status, property, and power. [Revista PUCP](https://revistas.pucp.edu.pe/index.php/historica/article/view/8680)

**TCE lesson:** proposed codes should create winners and losers. Wealthy owners may resist losing valuable floor area; officials may promote genuine risk reduction and political centralization at the same time. Disaster severity does not determine reform success by itself.

### 3.7 Djenné: maintenance is not a single variable

At the Great Mosque of Djenné, annual plastering maintained exposed surfaces, yet structural weaknesses still required consolidation. Following exceptional rain, the upper part of a tower collapsed on **5 November 2009**; **four masons suffered light injuries**. Authorities secured the area and restoration continued. [UNESCO World Heritage Centre](https://whc.unesco.org/en/news/574)

**TCE lesson:** distinguish protective maintenance, structural repair, and restoration work. A building can receive frequent care and still contain a deeper vulnerability. Conversely, component replacement and surface renewal can preserve an architectural tradition without preserving all its original material.

### 3.8 Kashmir: established vernacular methods can remain technically competitive

Arup’s study of *dhajji dewari*—timber frames with stone and mud-mortar infill—identified structural details critical to reliable seismic performance and found the system appropriate for seismic areas when well constructed. Its effectiveness is conditional on the assembly, not an intrinsic safety property of loose stone and earth. [Arup](https://www.arup.com/en-us/insights/dhajji-dewari-affordable-seismically-resistant-and-sustainable-housing/)

**TCE lesson:** avoid a universal sequence in which masonry supersedes timber, or reinforced concrete automatically supersedes vernacular construction. Regional skill and detailing can matter as much as access to a nominally newer material.

### 3.9 Modern failures: responsibility and institutions

**Hyatt Regency, 1981.** The walkway collapse killed **114 people**. Changing the suspension arrangement doubled the load on upper connections; the changed design had only about **30% of the mandated minimum capacity**. ASCE’s account documents failures of communication, calculation, review, and responsibility across the project. [American Society of Civil Engineers](https://www.asce.org/publications-and-news/civil-engineering-source/civil-engineering-magazine/article/2007/01/the-hyatt-regency-walkway-collapse)

**Silver Bridge, 1967.** Failure of a single eyebar initiated a collapse that killed **46 people**. Subsequent federal action led to the **1971 National Bridge Inspection Standards**, with trained inspectors, periodic inspections, written findings, and follow-up requirements. [Federal Highway Administration](https://www.fhwa.dot.gov/highwayhistory/national_bridge_inspection_standards.cfm)

**Bangladesh after Rana Plaza, 2013.** By the end of 2015, three inspection initiatives had examined **3,780 factories**, and **39** had been closed as immediately dangerous. Those closures were avoided exposures—not recorded collapses. ILO also emphasized that inspection had to be followed by financed and monitored remediation. [International Labour Organization](https://www.ilo.org/resource/improving-working-conditions-ready-made-garment-industry-progress-and)

**TCE lesson:** modernity adds organizational layers. It does not eliminate the possibility that everyone assumes someone else checked the critical detail.

---

## 4. Stylized facts a correct simulation should reproduce

### Failure should cluster around shared events

A flood or earthquake can cause many failures within hours after decades of relative quiet. In the US bridge collection, **112 of 503 failures occurred in 1993**, a peak associated with major flooding. Independent annual building dice would poorly reproduce that concentration. [르네방재정책연구원](https://renetopia.net/wp-content/uploads/2016/12/Analysis-of-Recent-Bridge-Failures-in-the-United-States.pdf)

### Collapse and death should not be interchangeable

Cook’s bridge study found loss of life in approximately **4% of collapses**. That is not transferable directly to housing, but it demonstrates the importance of occupancy, closure, timing, and exposure. A destroyed empty bridge can cause enormous economic disruption without immediate deaths. [USU Institutional Repository](https://digitalcommons.usu.edu/etd/2163/)

### Structural age should be an unreliable shortcut

The failed bridges in Wardhana and Hadipriono’s collection ranged from **1 to 157 years old**. Their average age at failure is not an average service life, because the study did not follow a representative cohort from construction to disappearance. [르네방재정책연구원](https://renetopia.net/wp-content/uploads/2016/12/Analysis-of-Recent-Bridge-Failures-in-the-United-States.pdf)

For TCE, older buildings should often carry accumulated damage—but some should survive because weak components were replaced, loads were controlled, or earlier vulnerabilities were corrected.

### Good performance should be conditional, not a material hierarchy

The same construction family should produce different outcomes under different connections, workmanship, maintenance, and hazards. The *dhajji dewari* evidence is a useful benchmark against “newer material always safer” behavior. [Arup](https://www.arup.com/en-us/insights/dhajji-dewari-affordable-seismically-resistant-and-sustainable-housing/)

### Regulation should have lagged, incomplete effects

A rule changes future decisions; it does not retroactively strengthen every existing structure. Silver Bridge’s aftermath involved legislation, standards, training, inventories, and continuing follow-up. Bangladesh’s experience similarly separates inspection from completed remediation. [Federal Highway Administration](https://www.fhwa.dot.gov/highwayhistory/national_bridge_inspection_standards.cfm)

### Disasters should produce both learning and political conflict

Fidenae generated targeted restrictions; Lima generated resistance to restrictions. Therefore, TCE should not make a large death toll automatically unlock an effective code. The outcome should depend on attribution, authority, interests, resources, and institutional reach. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Tacitus/Annals/4E%2A.html)

### Survivors should not be mistaken for typical construction

A world of surviving monuments gives a biased impression of historical reliability. Beauvais’s survival through rebuilding and stabilization is particularly instructive: standing today is compatible with a history of major failure and extensive intervention. [Columbia MCAH Projects](https://projects.mcah.columbia.edu/medieval-architecture/htm/ms/ma_ms_bc_discuss.htm)

---

## 5. Recommended TCE model

### 5.1 Use a coarse structural graph, not a building-wide health bar

For ordinary buildings, a practical initial budget is **8–32 structural components**, increasing only for large or unusual structures. This is an engineering budget recommendation, not a historical parameter.

Represent foundations, supporting wall groups or posts, floor groups, roof groups, and essential lateral restraints. Connections should be explicit where their failure changes the load path.

Each component needs:

| State | Purpose |
| --- | --- |
| Geometry and tributary loads | Determine which loads it carries. |
| Capacity by failure mode | Distinguish bending, compression, shear, instability, connection failure, and foundation support. |
| Construction quality and latent defects | Persistently distinguish apparently identical buildings. |
| Exposure and deterioration | Track wetting, fire damage, corrosion, erosion, or accumulated cycling. |
| Support and restraint relationships | Determine load transfer and whether local damage propagates. |
| Visible symptoms | Provide imperfect information to occupants, builders, and inspectors. |
| Construction and repair history | Preserve modifications, responsibility, and technical lineage. |

A connection graph alone is insufficient: a single tiny support must not carry an arbitrarily large building merely because everything remains connected.

For component \(j\), failure mode \(k\):

\[
g\_{jk}(t)=R\_{jk}(t)-S\_{jk}(t)
\]

Here \(R\) is resistance and \(S\) is demand, measured in compatible units for that mode. A negative margin initiates failure or instability. After a component fails, redistribute loads and reevaluate affected neighbors.

**Keep defects persistent.** A poorly made joint should remain poor until repaired; it should not receive a fresh independent quality draw every day.

### 5.2 Combine simplified mechanics with hazard fragilities

Use direct capacity checks for routine gravity loads, alterations, construction stages, and modeled deterioration.

For earthquakes or other complex dynamic hazards, archetype-level fragility curves are an appropriate simplification:

\[
P(DS\ge k\mid IM,x)=
\Phi\!\left(
\frac{\ln(IM/\theta\_k(x))}{\beta\_k}
\right)
\]

Here \(IM\) is a compatible hazard-intensity measure, \(x\) describes the building, \(\theta\_k\) is the median intensity associated with a damage threshold, and \(\beta\_k\) describes uncertainty. GEM documents this approach and separately distinguishes collapse fragility from damage-state fragility. [OpenQuake](https://docs.openquake.org/vulnerability/fragility_models.html)

Three implementation safeguards matter:

**Do not double-count vulnerability.** When a curve already represents poor workmanship, applying another identical workmanship penalty exaggerates risk.

**Do not repeatedly reroll the same event.** Evaluate one coherent earthquake exposure, with further transitions for aftershocks or additional damage—not hundreds of independent “collapse chances” during its animation.

**Do not equate complete damage with complete geometric collapse.** Maintain separate fields for habitability, repairability, structural damage, and collapsed volume.

### 5.3 Resolve casualties from actual people and locations

Use the simulation’s real occupancy. A partial floor failure should affect people on that floor, below it, and within the outward debris footprint—not everyone assigned an address in the building.

A useful consequence sequence is:

`warning → recognition → evacuation decision → local failure → entrapment/injury → rescue → displacement`

Warning availability should vary by mechanism. Slowly increasing settlement may be observable; a brittle connection failure may provide little useful notice. Evacuation should depend on access, authority, perceived credibility, household alternatives, and economic pressure.

Track collapsed mass and geometry coarsely rather than simulating every fragment. Rescue then depends on reachable voids, available labor, tools, fire, water, and secondary-collapse risk. Use broad casualty scenarios until calibrated; there is no defensible universal “percentage killed in a collapse.”

Bridge failures should also alter routing, market access, emergency access, and settlement connectivity. Their economic importance can exceed their immediate casualty count.

### 5.4 Give responsibility a history

Attach each consequential decision to actors:

* The designer or master builder chose the scheme.
* Particular crews and suppliers produced components.
* The owner commissioned changes and maintenance.
* Inspectors observed—or missed—specific conditions.
* Authorities issued restrictions and decided whether to enforce them.

For investigations, preserve an internal causal record such as:

> Roof replacement increased dead load. A decayed beam end failed first. The adjacent floor lost support. Earlier leakage had been reported but not repaired.

Agents should not automatically see that complete record. Their findings depend on surviving evidence, competence, testimony, access, and political interference.

Track **competence**, **honesty**, and **compliance** separately. A mistaken calculation, concealed substitution, and refusal to finance repair are different actions and should produce different reputational consequences.

### 5.5 Make liability effective through institutions, not severity alone

A useful decision approximation is:

\[
\text{Expected private liability cost}
=
P(\text{discovered})
P(\text{attributed}\mid\text{discovered})
P(\text{enforced}\mid\text{attributed})
\times \text{effective sanction}
\]

This is a proposed behavioral model, not an empirically estimated historical equation.

Sanctions may include compensation, repair obligations, exclusion from contracts, confiscation, loss of license, exile, or other penalties appropriate to the society. Effective enforcement also depends on the liable actor remaining identifiable and able to pay.

Allow institutions to adopt measures independently: foundation checks, occupancy restrictions, maximum heights, proof tests, inspection intervals, mandatory maintenance, standardized details, or repair funds. A society may possess some without possessing modern engineering.

### 5.6 Separate technical learning from public distrust

A failure should update several kinds of belief:

| Knowledge holder | Possible lesson |
| --- | --- |
| Builder or workshop | “This joint detail is inadequate at this span.” |
| Owner | “The cheapest bid concealed expensive risks.” |
| Inspector or institution | “We need to inspect hidden bearings, not only visible walls.” |
| Public | “Buildings of this unfamiliar type are dangerous.” |
| Political authority | “A visible restriction will restore confidence.” |

Only the first three necessarily improve technical accuracy.

For craft learning, update beliefs about **specific combinations** of span, material, connection, foundation, and loading. Transfer lessons through apprenticeships, traveling builders, commissions, written specifications, and inspections. Public distrust can generalize much more broadly than the technical evidence warrants.

Also permit learning without catastrophe: repairs reveal defects, proof tests expose inadequate capacity, and surviving buildings provide conditional evidence. Survival under light loads should not be treated as proof of resistance to an unexperienced earthquake.

### 5.7 What to simplify—and what not to simplify

**Simplify geometry and dynamics.** Precompute representative structural behavior offline; use lookup tables and small graph updates at runtime.

**Do not simplify causality into arbitrary decay.** Damage should have an exposure history.

**Do not simplify maintenance into rejuvenation.** Replastering a wall, replacing a beam, installing a tie, and rebuilding a foundation affect different variables.

**Do not simplify institutions into a safety multiplier.** A code should change construction, inspection, occupancy, or repair decisions.

For performance, recalculate immediately after structural changes, loads crossing relevant thresholds, or hazard events. Update slow deterioration at a coarser cadence. Unreal should visualize the consequences selected by the Rust simulation rather than independently deciding who dies through nondeterministic debris physics.

### 5.8 Existing models and games worth borrowing from

| Model or game | Useful contribution | Limitation for TCE |
| --- | --- | --- |
| **GEM/OpenQuake** | Building taxonomy, hazard-conditioned fragilities, uncertainty, separate damage and collapse concepts. | Does not supply endogenous builder reputation or political responses. [OpenQuake](https://docs.openquake.org/vulnerability/fragility_models.html) |
| **FEMA Hazus** | Structured regional loss modeling linking exposed buildings and infrastructure to physical, social, and economic consequences. | Requires careful adaptation of modern categories to authored preindustrial forms. [WBDG](https://www.wbdg.org/ar/tools/hazus) |
| **OpenSees** | Structural-analysis tooling suitable for offline tests of representative assemblies and reduced models. | Not a reason to run detailed finite-element analysis for every building continuously. [OpenSees](https://opensees.github.io/OpenSeesDocumentation/) |
| **Dwarf Fortress** | Cave-ins as interacting world events rather than isolated cinematics; useful design inspiration for systemic consequences. | Its mechanics are not an empirical structural-safety model. Developer discussions also highlight implementation and performance limits. [Bay 12 Games](https://bay12games.com/media/df_talk_22_transcript.html) |

---

## 6. Source priorities and remaining uncertainty

### A working research library

| Source | Main use |
| --- | --- |
| **Wardhana & Hadipriono, 2003**, “Analysis of Recent Bridge Failures in the United States,” *Journal of Performance of Constructed Facilities* 17(3):144–150 | Recorded causes, definitions, reporting bias, and event clustering. [르네방재정책연구원](https://renetopia.net/wp-content/uploads/2016/12/Analysis-of-Recent-Bridge-Failures-in-the-United-States.pdf) |
| **Wardhana & Hadipriono, 2003**, “Study of Recent Building Failures in the United States,” 17(3):151–158 | Modern building case collection; not a population failure rate. [ASCE Library](https://ascelibrary.org/doi/10.1061/%28ASCE%290887-3828%282003%2917%3A3%28151%29) |
| **Cook, 2014**, *Bridge Failure Rates, Consequences, and Predictive Trends*, Utah State University dissertation | A serious attempt to move from recorded cases toward rates and consequences, with explicit extrapolation limitations. [USU Institutional Repository](https://digitalcommons.usu.edu/etd/2163/) |
| **Tacitus, Annals 4.62–63 and 15.43; Strabo, Geography 5.3.7; Hammurabi §§229–233** | Primary textual evidence for disaster attribution, liability, and regulation—not verified incidence statistics. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Tacitus/Annals/4E%2A.html) |
| **Stephen Murray’s Beauvais research and Columbia architectural documentation** | Construction phases, collapse, rebuilding, and structural intervention. [Columbia MCAH Projects](https://projects.mcah.columbia.edu/medieval-architecture/htm/ms/ma_ms_bc_discuss.htm) |
| **Walker, 2004**, “La clase alta y sus altos,” *Histórica* 28(1):45–90 | Archival evidence for resistance to post-disaster building restrictions. [Revista PUCP](https://revistas.pucp.edu.pe/index.php/historica/article/view/8680) |
| **Grünthal, ed., 1998**, *European Macroseismic Scale 1998* | Vulnerability and damage classification; interpretation of historical earthquake evidence. [GFZ Potsdam](https://media.gfz.de/gfz/sec26/resources/documents/PDF/EMS-98_Original_englisch.pdf) |
| **Morris & Winandy, 2002**, “Limiting conditions for decay in wood systems” | Environmental conditions for deterioration rather than arbitrary service-life timers. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/8587) |
| **Marshall et al., 1982**, *Investigation of the Kansas City Hyatt Regency Walkways Collapse*, NBS BSS 143 | Forensic investigation combining inspection, testing, and analysis. [NIST](https://www.nist.gov/publications/investigation-kansas-city-hyatt-regency-walkways-collapse-1) |

The thinnest evidence remains **ordinary premodern failure frequency**, **casualty ratios for different collapse geometries**, **actual enforcement of early liability rules**, and **the speed and accuracy of learning between workshops**. Treat those as uncertain model parameters and test their consequences explicitly.

The strongest evidence concerns **specific failure mechanisms**, **particular historical responses**, and **conditional differences between structural types and construction quality**.

**Bottom line:** TCE should simulate structures that are sometimes weak, increasingly damaged, unusually loaded, or badly managed—not structures that simply receive a random death sentence. Whether a failure becomes a household tragedy, a citywide disaster, a technical lesson, or an unenforced law should emerge from who was exposed, who knew what, who had authority, and who could afford to act.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92912-c2e0-83e9-a114-15393ea6ad10)
