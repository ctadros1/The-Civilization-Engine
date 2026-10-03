# Organic urban morphology: a simulation-ready report for TCE

## Executive finding

**Model the history of access and property, not an “organic-looking” street pattern.** For TCE, the most useful representation is a co-evolving system of movement routes, territorial claims, buildings, and institutions.

The sequence is not always *paths → streets → plots → buildings*. Buildings can establish claims before plots are defined; existing agricultural boundaries can organize subdivision; and relatively regular streets can emerge through small, locally coordinated additions rather than a comprehensive plan. Contemporary comparative research documents all these processes. [Springer](https://link.springer.com/article/10.1057/s41289-026-00344-4)

Historical evidence also cautions against equating irregularity with institutional weakness. Scottish towns maintained durable plot boundaries through local regulation, while Islamic legal traditions distinguished different kinds of access and neighboring rights. Neither context is adequately represented by unrestricted individual building placement. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A)

**Recommended core:** individual movement generates demand and wear; households propose construction and subdivision; property and access rights constrain those proposals; institutions adjudicate conflicts and undertake collective changes. Irregular geometry should emerge from that history.

---

# 1. Mechanisms translated into implementable rules

The historical findings below motivate the mechanisms. The algorithms and decision rules are **proposed TCE implementations**, not estimated historical laws.

## 1.1 Repeated movement creates paths—but only on accessible terrain

Helbing, Keltsch and Molnár’s active-walker model demonstrates how feedback between pedestrian movement and environmental modification can generate trail networks. People respond to existing trails while their movement reinforces them. Its empirical reference is pedestrian trail formation, however, not centuries of urban property development. [Nature](https://www.nature.com/articles/40353)

For TCE, let agents choose routes using travel time, terrain, weather, congestion, familiarity and access permissions. Do not restrict every trip to an already existing street graph: some exploration across traversable ground is necessary for new desire lines to appear.

A simple wear model is:

\[
\frac{dw\_x}{dt}=\alpha q\_x(1-w\_x)-\beta\_x w\_x
\]

Here \(w\_x\) is normalized wear in cell \(x\); \(q\_x\) is equivalent pedestrian traversals per day; \(\alpha\) is wear per traversal; and \(\beta\_x\) is recovery per day.

**Implementation rules:**

* Walking reinforces a corridor; unused vegetation-covered routes gradually recover.
* Surface consequences depend on terrain: trampling may improve passage, while mud, rutting or erosion can make heavily used ground worse.
* Fences, crops, buildings and recognized restrictions redirect movement.
* A physically fading path can retain a legal right of passage. Conversely, a visibly worn shortcut need not become a recognized public route.
* Upgrade to a maintained lane only when someone supplies labor, drainage, surfacing or clearance.

This separates three things that should not share one variable: **use, physical condition, and legal status**.

## 1.2 Settlement creates destinations; destinations reorganize movement

A useful siting decision is not “choose a random point near a road.” Each household or enterprise should evaluate several feasible locations against its own needs.

A proposed utility calculation is:

\[
U\_{i,p}=
B\_{\text{access}}
+B\_{\text{livelihood}}
+B\_{\text{social}}
-C\_{\text{construction}}
-C\_{\text{land}}
-C\_{\text{hazards}}
-C\_{\text{conflict}}
\]

Accessibility should be measured to relevant destinations: fields, water, landing places, workshops, markets, relatives, places of assembly—not merely distance to a settlement center.

In an early settlement, land costs may consist of clearing labor, customary obligations or negotiation rather than a monetary purchase. Later, rents and sale prices can enter the same decision framework.

Once a household builds, it becomes a new destination. Its doorway and working spaces redirect nearby movement. This creates a feedback loop:

**location choice → new trips → route reinforcement → changed attractiveness → further construction.**

For performance, evaluate a small candidate set drawn from known or locally discovered sites rather than requiring agents to know every vacant location.

## 1.3 Claims and buildings need not appear in the same order

Maintain distinct operations for:

**Claim-first development:** a person or institution establishes a boundary, then builds within it.

**Building-first development:** construction establishes the initial claim; surrounding land and access rights are negotiated afterward.

**Collective local layout:** a group agrees on several plots, a lane or a courtyard without planning the entire town.

Research on Kyoto’s machiya explicitly investigates a buildings-before-plot-division interpretation and identifies the townhouse as a constituent unit of the late-medieval market. This is an important counterexample to always generating cadastral parcels before architecture. [J-STAGE](https://www.jstage.jst.go.jp/article/jusokenold/34/0/34_0611/_article/-char/en)

For TCE, allow provisional claims with incomplete boundaries. Later surveys, disputes or sales can turn them into more explicit parcels. Boundaries should inherit local influences—existing walls, field edges, drainage lines, neighboring buildings and access agreements—rather than receiving arbitrary geometric noise.

## 1.4 Conzen: preserve streets, plots and buildings as different historical layers

Conzen’s town-plan analysis distinguishes the ground plan—streets, plots and building footprints—from building fabric and land/building use. These layers can change at different speeds. His **burgage cycle** describes progressive occupation of rear plots, eventual clearance, possible urban fallow and redevelopment; it is not an inevitable sequence with a universal duration. [Scribd](https://www.scribd.com/document/570396026/White-Hand-2001)

For TCE, give every parcel a persistent identity and genealogy. A new building should not automatically create a new parcel. A demolished building should not automatically erase ownership.

A burgage-like development sequence can emerge from ordinary actions:

> Street-front house and working yard → rear workshop or dwelling → shared side passage → further backland occupation → additional floors or rebuilding → eventual amalgamation or clearance.

The important relationship is **valuable access at the front plus expandable land behind**, not a mandatory medieval aspect ratio.

Also avoid interpreting every narrow plot as a subdivision. Tait’s reconstruction of Scottish burgage dimensions argues that different fractional widths could have been assigned at the outset. Similar-looking plans can therefore encode different histories. [Society of Antiquaries Journals](https://journals.socantscot.org/index.php/psas/article/view/9724)

## 1.5 Subdivision is a property-and-access transaction, not just polygon splitting

When a household grows, inherits property or needs income, offer several alternatives: divide use rights within a building, share ownership, build another dwelling, divide the land, sell the whole property, or relocate.

**Do not automatically bisect a plot at every inheritance.** Make the outcome depend on the society’s inheritance rules, household organization, available capital and preferences.

A physical subdivision should answer:

* Can each resulting occupied unit be reached?
* Who owns and maintains the access?
* Does the division preserve necessary working, drainage and shared space?
* Can the available building system fit the resulting geometry?

A rear plot may be reachable through an easement, shared courtyard, covered passage or private alley. It does not necessarily need its own frontage on a public street. Scottish examples show covered passages preserving rear access while upper floors span the frontage. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A)

Topological approaches to neighborhood access and reblocking provide useful computational precedents, but their contemporary upgrading objectives should not become a rule that every historical dwelling needs vehicular access. [Oak Ridge National Laboratory](https://www.ornl.gov/publication/toward-cities-without-slums-topology-and-spatial-evolution-neighborhoods)

## 1.6 Densification has several independent channels

Represent at least four separate decisions:

**More occupants:** existing floor space is shared more intensively.

**More footprint:** buildings extend, new structures occupy yards, or gaps are filled.

**More floors:** vertical additions increase usable area without changing the street plan.

**Reorganization:** plots are merged, buildings replaced, passages moved or uses changed.

A useful accounting identity is:

\[
D=\frac{10{,}000\,c\,h\,r}{a}
\]

where \(D\) is residents per hectare, \(c\) is building-footprint coverage of the specified area, \(h\) is footprint-weighted mean floors, \(r\) is the residential share of floor area, and \(a\) is residential floor area per person.

For example, \(c=0.60,\ h=2,\ r=0.75,\ a=25\) gives **360 residents/ha**. This is an illustrative calculation, not a historical density estimate.

In the simulation, construction feasibility should depend on materials, structural capability, household resources and rules governing light, access, drainage or neighbors. “Settlement level” should not directly determine floor count.

## 1.7 Streets can narrow, widen, become covered, or change use without moving

Store three widths separately:

| Width concept | What it means in TCE |
| --- | --- |
| **Recognized access corridor** | Land or space over which passage rights exist. |
| **Physical width** | Ground-level clearance between permanent obstructions. |
| **Effective usable width** | Clearance at a particular time after stalls, stairs, storage, crowds or parked vehicles. |

Encroachment should be an action by an identifiable actor. It may be temporary, contested, tolerated, purchased or authorized—not an automatic annual shrinking factor.

Widening should likewise require a proposal, financing or labor, and a resolution of affected rights. Calculate beneficiaries and losers separately. A change that improves town-wide access can still be blocked by those losing buildings or frontage.

Changes can occur vertically too: an upper-floor connection need not eliminate passage underneath. Consequently, a two-dimensional road polygon alone is insufficient for every settlement type.

## 1.8 Open space is not merely land waiting to be filled

At Songo Mnara, archaeological analysis identifies overlapping communal, household, ritual and productive territories. Open areas and relationships between houses, mosques and tombs were part of the functioning townscape, not simply unused gaps. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/00438243.2016.1179128?utm_source=chatgpt.com)

For TCE, assign open land actual functions and rights: livestock handling, gardening, drying goods, drainage, gatherings, burial, ritual or shared access.

A household proposing infill must compete with those uses and their defenders. This allows courtyards and commons to persist even under development pressure—and allows their loss when institutions fail or political priorities change.

---

# 2. Quantitative parameters

## 2.1 Empirical anchors and documented prescriptions

**Confidence concerns the stated case, not worldwide transferability.** “High” does not mean that a Scottish dimension should be applied to an African or Asian settlement.

| Parameter or observation | Value and units | Evidence and appropriate use | Confidence |
| --- | --- | --- | --- |
| Recorded Scottish plot allocation | **24 ft ≈ 7.32 m frontage** | One documented staking instruction; a local allocation example, not a universal burgage width. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A) | High for the record; low transferability |
| Historical frontage advance | **1.6–2.9 m; mean 2.2 m**, across **13 sites** | Physical evidence from Edinburgh and Canongate. Useful for testing discrete encroachment events. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A) | High locally |
| Reconstructed street narrowing | Approximately **23→19 m** in Edinburgh; **15→11 m** in Canongate | Reconstruction of cumulative frontage changes, not annual narrowing rates. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A) | Medium |
| Contemporary informal streets and lanes | Approximately **0.5–12 m** overall; **2–5 m** described as most common | Comparative descriptive ranges, not statistical percentiles. [Springer](https://link.springer.com/article/10.1057/s41289-026-00344-4) | Medium |
| Pedestrian lanes in that comparison | **1–2 m** common; examples around **0.5 m** | Supports narrow-access geometry; not a universal minimum or desirable standard. [Springer](https://link.springer.com/article/10.1057/s41289-026-00344-4) | Medium |
| Vehicle-access routes in that comparison | **4–7 m** commonly described | Width alone does not determine passage: turns, steps and obstructions matter. [Springer](https://link.springer.com/article/10.1057/s41289-026-00344-4) | Medium |
| Public through-street prescription in Hakim’s account of Islamic rules | **7 local cubits** | A normative rule, not observed mean width. Keep the local unit explicit rather than assuming one universal cubit length. [ResearchGate](https://www.researchgate.net/publication/26502838_Revitalizing_Traditional_Towns_and_Heritage_Districts) | Medium as a historical prescription |
| Ahmedabad pol neighborhood scale | **50–100 houses** in the UNESCO atlas description | An illustrative neighborhood scale, not a universal distribution of pol sizes. [UNESCO World Heritage Centre](https://whc.unesco.org/en/urban-heritage-atlas/ahmedabad) | Medium |
| Local management in that description | Normally **5 representatives** | Useful precedent for a neighborhood-level institution managing shared facilities and gates. [UNESCO World Heritage Centre](https://whc.unesco.org/en/urban-heritage-atlas/ahmedabad) | Medium |
| Early Ceramic Neolithic houses at Çatalhöyük | Approximately **25 m²** in the excavation team’s description | A phase-specific architectural reference, not an all-Neolithic household standard. [Catalhoyuk](https://www.catalhoyuk.pl/results.htm) | Medium |
| Groane road-network growth, northern Italy, 1833–2007 | Nodes increased from **255 to more than 5,000**; total length scaled approximately as **\(N^{0.54}\)** | Seven historical snapshots of one region. Useful for reproducing that trajectory, not imposing a universal exponent. [Nature](https://www.nature.com/articles/srep00296) | High for the mapped case |

There is considerably less defensible evidence for a universal “plot subdivision probability per year,” “number of footsteps needed to make a path,” or “street-widening rate.” Those should be treated as calibration problems rather than supplied with spurious historical precision.

## 2.2 Suggested TCE starting values—not historical estimates

These are **implementation and sensitivity-testing choices made in this report**. They are deliberately separate from the empirical table.

| Proposed parameter | Starting test range | Units | Rationale and confidence |
| --- | --- | --- | --- |
| Fine wear-field resolution near settlement | **0.5–1** | m/cell | Engineering choice; keep final lane geometry vector-based. Not an empirical estimate. |
| Unused-path wear half-life | **30–365** | days | Broad sensitivity sweep; vary by vegetation, soil and season. Empirically uncalibrated. |
| Wear coefficient \(\alpha\) | **0.002–0.02** | per equivalent traversal | Numerical starting envelope for the proposed equation. Resolution-dependent and uncalibrated. |
| Favorable travel-cost reduction on a dry, established trail | **10–40%** | fraction of off-trail cost | Test the strength of reinforcement; allow mud or erosion to reverse the advantage. Uncalibrated. |
| Candidate locations per active siting decision | **8–32** | candidates | Bounded-search engineering choice. |
| Routine housing/land reconsideration | **1–4** | weeks between evaluations | Stagger evaluations; births, inheritance, eviction or destruction can trigger immediate reconsideration. |
| Small-household starter plot, selected test scenarios | **100–600** | m² | Authoring envelope only. Do not apply to every society or building type. |
| Shared-compound starter plot, selected test scenarios | **400–2,000** | m² | Separate scenario envelope, not an estimate of African, Asian or ancient compounds generally. |

The plot-area ranges are best replaced by **building-fit constraints plus livelihood-space demand** as the architecture system matures. A household using a yard for production should not receive the same residual-space preference as a household renting rooms.

For wear updates, use an exact or bounded numerical update. Large daily traffic flows can make a naïve Euler implementation overshoot the \([0,1]\) interval.

---

# 3. Variation across eras and regions

## 3.1 Eras should describe conditions, not unlock morphology presets

| Context | What the evidence suggests | TCE implication |
| --- | --- | --- |
| **Foraging societies** | Comparative research finds that larger temporary camps can become **less dense**, unlike the familiar increasing-density expectation for permanent settlements. Camp organization should not simply be treated as a miniature town. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/719234) | Let mobility, aggregation and dispersal affect claims and spacing. Do not assume permanent cadastral plots or a circular camp template. |
| **Early farming and early sedentism** | Çatalhöyük included tightly packed houses with roof circulation and ladder access rather than ordinary streets between dwellings. Its later phases also differed from earlier ones. [Çatalhöyük Research Project](https://www.catalhoyuk.com/node/56) | Permit roof, courtyard and passage access; farming does not automatically imply detached houses along lanes. |
| **Pre-industrial towns** | Burgage, courtyard and neighborhood institutions demonstrate that incremental adaptation can occur within durable access and property frameworks. [Scribd](https://www.scribd.com/document/570396026/White-Hand-2001) | Make tenure, construction systems and local adjudication the controlling variables. |
| **Industrializing settlements** | Groane’s mapped development combined expansion into new territory with network densification. More regular junctions and blocks emerged without a comprehensive regional master plan. [Nature](https://www.nature.com/articles/srep00296) | Allow many small developments to produce regularity. New transport demands should alter incentives without automatically erasing inherited boundaries. |
| **Modern informal urbanization** | Buildings, plotting and access networks can develop together, including relatively organized local layouts. [Springer](https://link.springer.com/article/10.1057/s41289-026-00344-4) | Represent residents, landholders, collective organizations and developers—not only isolated self-builders. |

In TCE, transitions should follow changes in **mobility, transport mode, building technology, wealth, tenure security, surveying capacity and political authority**. There should be no rule saying “industrial era produces wider streets” independently of these causes.

## 3.2 Regional differences are combinations of mechanisms

### European burgage and backland development

Use frontage-oriented allocation, persistent lateral boundaries and multiple forms of rear access. Do not classify all burgage towns as unplanned: some began with deliberate allocations and later accumulated complex development.

Scottish evidence is particularly useful for distinguishing relatively stable plot frameworks from changing architecture. It also demonstrates that regulation and encroachment can coexist. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A)

**Implementation emphasis:** parcel genealogy, frontage value, covered passages, rear buildings, amalgamation and boundary adjudication.

### Islamic Mediterranean and Middle Eastern traditions

Hakim’s account differentiates public through-routes from dead-end access shared by adjoining residents. It also discusses earlier-use rights, privacy, harm and local custom. These provide mechanisms for controlling doorways, additions and circulation without requiring a geometrically comprehensive plan. [ResearchGate](https://www.researchgate.net/publication/26502838_Revitalizing_Traditional_Towns_and_Heritage_Districts)

But there is no single timeless “Islamic city” form. Abu-Lughod’s critique is essential: a generalized cultural label can conceal historical, institutional and regional variation. [Cambridge University Press](https://www.cambridge.org/core/journals/international-journal-of-middle-east-studies/article/abs/islamic-city-historic-myth-islamic-essence-and-contemporary-relevance/E686C6BE7F410B6C1717C992116B8195)

**Implementation emphasis:** access membership, neighbor objections, doorway orientation and complaint-based adjudication. Do not encode religion as a maze-generation parameter.

### East Asia: Beijing

Research on Beijing’s hutong neighborhoods reconstructs rules connecting courtyard houses, plots and streets inside a larger historical urban framework. The coexistence of a planned capital structure with local morphological change is precisely why “planned” and “organic” should not be mutually exclusive settlement classes. [DOI](https://doi.org/10.1016/j.foar.2022.12.004)

**Implementation emphasis:** hierarchical scales of control—major streets or wards can be imposed while courtyard plots evolve locally.

### East Asia: Kyoto

The machiya research links architectural propagation, market organization and plot formation. The causal direction can run from building type to urban parcel structure, not only from parcel to building. [J-STAGE](https://www.jstage.jst.go.jp/article/jusokenold/34/0/34_0611/_article/-char/en)

**Implementation emphasis:** authored building modules can influence frontage increments, shared walls and subsequent subdivision. Avoid inventing a universal frontage-tax explanation for narrow houses.

### South Asia: Ahmedabad

The pol system combines closely packed shared-wall housing, gated access, branching streets and shared facilities. The UNESCO atlas explicitly associates its form with collective agreements and local management. [UNESCO World Heritage Centre](https://whc.unesco.org/en/urban-heritage-atlas/ahmedabad)

**Implementation emphasis:** a neighborhood institution should be able to manage entrances, wells and common spaces independently of the city government. Public accessibility may vary by gate state, membership and time.

### Africa: Swahili towns

At Songo Mnara, household exclusivity coexisted with communal ritual and productive spaces. The archaeological interpretation concerns overlapping territories rather than a town partitioned into nothing but exclusive residential lots. The site’s unusually short occupation also makes it an imperfect representative of all Swahili urban history. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/00438243.2016.1179128?utm_source=chatgpt.com)

**Implementation emphasis:** retain overlapping use rights and purposeful open spaces. Do not classify every unroofed area as vacant land.

### Contemporary African, Asian and Latin American settlements

Comparative examples include incremental building in Siqalo, South Africa; back-to-back arrangements in Old Fadama, Accra; and plotting associated with reclaimed land in Karachi. Such cases demonstrate different routes to local regularity and irregularity within “informal” development. [Springer](https://link.springer.com/article/10.1057/s41289-026-00344-4)

**Implementation emphasis:** vary development actors, terrain, construction increments and access agreements. A continent-wide morphology preset would discard the mechanisms that explain the differences.

---

# 4. Stylized facts and validation targets

A convincing simulation should reproduce **families of trajectories**, not just a plausible final screenshot.

| Pattern to reproduce | Measurement in TCE | Historical or comparative anchor |
| --- | --- | --- |
| **Old boundaries constrain newer buildings** | Survival of parcel edges across successive building replacements; alignment of later walls with earlier boundaries. | Conzenian morphological persistence. [Scribd](https://www.scribd.com/document/570396026/White-Hand-2001) |
| **Expansion and infill coexist** | Fraction of new route length opening previously unserved territory versus connecting or subdividing existing development. | Groane’s exploration/densification distinction. [Nature](https://www.nature.com/articles/srep00296) |
| **Regularity need not imply centralized planning** | Changes in junction types, street orientation and block shape under exclusively local development decisions. | Piecemeal development in Groane. [Nature](https://www.nature.com/articles/srep00296) |
| **Access is hierarchical and sometimes restricted** | Public, shared and private route lengths; gated components; access depth from public space to dwelling. | Islamic access distinctions and Ahmedabad’s pol structure. [ResearchGate](https://www.researchgate.net/publication/26502838_Revitalizing_Traditional_Towns_and_Heritage_Districts) |
| **Streets change without wholesale network replacement** | Frontage shifts, width distributions and effective clearance over time. | Meter-scale Scottish frontage advances. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A) |
| **Open space can persist under development pressure** | Survival of shared productive, ritual and circulation areas despite neighboring construction. | Songo Mnara’s activity territories. [Taylor & Francis Online](https://www.tandfonline.com/doi/full/10.1080/00438243.2016.1179128?utm_source=chatgpt.com) |
| **Architecture can require non-street access** | Reachability through roofs, ladders, courtyards and covered passages. | Çatalhöyük and Scottish rear access. [Çatalhöyük Research Project](https://www.catalhoyuk.com/node/56) |

## Measurement discipline

Use consistent definitions before comparing outputs.

**Separate street centerlines from rights-of-way.** A centerline graph cannot measure usable width, encroachment or whether an alley is actually traversable.

**Distinguish blocks, parcels and buildings.** A street-enclosed block can contain many parcels; one parcel can contain several buildings; one building can contain several households.

**Simplify graph geometry appropriately.** Points inserted along a curved street are not additional intersections. Keep genuine junctions, dead ends and changes of access distinct from shape vertices. The Groane study explicitly excludes degree-two points from its junction comparison. [Nature](https://www.nature.com/articles/srep00296)

**Do not impose a universal fractal or power-law target.** Barthélemy and Flammini’s model produces different block-area distributions under different assumptions about destination placement. Similar visual complexity need not imply the same generating process. [arXiv](https://arxiv.org/html/0708.4360)

For initial calibration, record distributions of frontage, depth, area, building coverage, floors, street width, route detour, shared-access depth and boundary age. Compare them within matched terrain and institutional scenarios rather than pooling every settlement into one average.

---

# 5. Modeling recommendation for TCE

## 5.1 Recommended state representation

Use a **hybrid spatial model**:

| Layer | Minimum persistent state |
| --- | --- |
| **Terrain and wear** | Elevation, slope, water, drainage, surface conditions, traversability, traffic accumulation. |
| **Movement network** | Polyline routes; width profile; surface; allowed modes; access membership; gates; maintenance responsibility. |
| **Territorial claims** | Polygon or provisional boundary; claimant; strength/status; exclusive and shared rights; parent parcels; dispute history. |
| **Buildings** | Footprint, floors, modules, structural capability, entrances, internal/shared circulation, occupied uses. |
| **Institutions** | Allocation rules, inheritance rules, neighbor protections, adjudication procedures, public-work authority and resources. |
| **Event history** | Construction, split, merge, transfer, encroachment, clearance, widening, closure and abandonment events. |

Use a separate access-rights graph alongside geometry. A path through somebody else’s parcel can be valid; a visually open gap can be restricted.

Ordinary surface streets can use planar geometry, while bridges, roof routes and covered passages occupy explicit levels. Do not force all connectivity into planar polygon adjacency.

## 5.2 Assign decisions to the right actors

Individuals should generate trips and recognize opportunities. **Households**, however, are usually the useful decision unit for dwelling expansion, relocation and inheritance-related choices in the proposed model.

Neighborhood groups can coordinate a shared lane or courtyard. Landholders and developers can propose multiple plots at once. A ruler or council can impose a street, while courts or customary authorities decide disputes.

This gives TCE a continuum between individual action and comprehensive planning rather than a binary planned/unplanned switch.

## 5.3 Make geometry edits transactional

Every proposed change should pass through:

**Proposal → affordability and construction test → rights/access test → negotiation or adjudication → construction → graph and geometry update.**

A failed proposal should have an intelligible reason: insufficient materials, loss of someone else’s access, disputed ownership, inadequate structural support, or an unwilling co-owner.

Preserve three invariants:

1. Occupied units retain their authorized access unless a modeled event deliberately removes it.
2. Land and building rights are not silently destroyed by geometry regeneration.
3. Splits, mergers and replacements retain their historical relationships.

These are implementation safeguards, not assumptions that historical societies never produced inaccessible or dispossessed households. Violations should be explicit social events.

## 5.4 Update at several timescales

For 10k–50k people, avoid recomputing the entire spatial system every simulation tick.

Aggregate actual movements into wear updates daily. Reconsider housing needs weekly or monthly, with event-triggered exceptions. Run expensive polygon operations only when a real alteration is proposed or completed.

Cache commonly used routes and invalidate affected sections after local changes. Use spatial indexing to find nearby claims, buildings and candidate connections. Keep the fine wear raster sparse around actively used areas.

The Rust kernel should own authoritative geometry and rights. UE5 should consume snapshots or change events rather than independently deciding whether a plot or route exists.

These choices are a plausible performance architecture; they are not a demonstrated benchmark for 50,000 agents.

## 5.5 Existing computational models and games

| Model or game | What is useful | What it does not establish |
| --- | --- | --- |
| **Helbing, Keltsch & Molnár, “Modelling the evolution of human trail systems” (1997)** | Movement–environment feedback for desire paths. | Property formation, inheritance and urban institutions. [Nature](https://www.nature.com/articles/40353) |
| **Barthélemy & Flammini, “Modeling Urban Street Patterns” (2008)** | Local road-growth rules and network formation around destinations. | Endogenous household economies or historical land rights. [arXiv](https://arxiv.org/html/0708.4360) |
| **Parish & Müller, “Procedural Modeling of Cities” (2001)** | Global goals plus local geometric constraints; procedural streets, lots and buildings. | A demonstrated causal simulation of autonomous settlement history. [ResearchGate](https://www.researchgate.net/publication/220720591_Procedural_Modeling_of_Cities) |
| **Weber et al., “Interactive Geometric Simulation of 4D Cities” (2009)** | Explicit evolving streets, parcels and buildings; a strong geometric precedent. | TCE-scale individual behavior or historically calibrated institutions. [Arizona State University](https://asu.elsevierpure.com/en/publications/interactive-geometric-simulation-of-4d-cities/) |
| **Wang, Crompton & Agkathidis, “The Hutong neighbourhood grammar” (2023)** | Coupled rules for courtyard buildings, plots and neighborhood structure. | Proof that one inferred grammar uniquely explains the historical process. [DOI](https://doi.org/10.1016/j.foar.2022.12.004) |
| **Brelsford et al., “Toward cities without slums” (2018)** | Topological access diagnostics and targeted network restructuring. | A universal requirement that every historical parcel receive modern street access. [Oak Ridge National Laboratory](https://www.ornl.gov/publication/toward-cities-without-slums-topology-and-spatial-evolution-neighborhoods) |
| **Foundation** | A game reference for villagers creating natural paths and building houses within player-directed development. | A documented simulation of autonomous cadastral and legal evolution. [Polymorph Games](https://polymorph.games/presskit/) |
| **Manor Lords** | Flexible burgage geometry, rear extensions and additional accommodation. | Autonomous plot creation: the player defines the plot layout. [Hooded Horse](https://wiki.hoodedhorse.com/Manor_Lords/Burgage_Plot) |

**Best combination:** active-walker path formation, persistent parcel geometry, modular building grammars, and an event-driven rights system. None of the cited approaches alone covers that complete combination.

## 5.6 What to simplify first

For v1, keep building interiors abstract except where they determine access. Represent disputed rights with a compact set of claim strengths and permissions rather than a complete legal language.

Start with a small number of development actions: claim, build, extend, split, merge, add floor, create passage, encroach, widen and abandon. Add architectural richness through modules without increasing the number of social rules unnecessarily.

Do **not** simplify away shared access, parcel history or the distinction between temporary and permanent obstructions. Those are disproportionately important to generating credible form.

---

# 6. Sources, datasets and evidential limits

## 6.1 Datasets and map collections

| Resource | Useful scope or resolution | Best use—and main limitation |
| --- | --- | --- |
| **National Library of Scotland town-plan collections** | Historical town maps, including large-scale Ordnance Survey plans around **1:500**. | Digitize street edges, footprints and plot-like boundaries. A mapped boundary is not automatically proof of ownership. [National Library of Scotland Maps](https://maps.nls.uk/towns/) |
| **UNESCO Urban Heritage Atlas** | Documented neighborhood layouts, buildings and management practices; Ahmedabad is especially relevant here. | Compare spatial organization with institutional descriptions. Heritage examples are not representative random samples. [UNESCO World Heritage Centre](https://whc.unesco.org/en/urban-heritage-atlas/ahmedabad) |
| **Çatalhöyük Research Project and Duke Dig@Lab** | Excavation plans, spatial documentation and 3D recording. | Test house aggregation and non-street access. Match archaeological phases; do not treat every excavated structure as contemporaneous. [DigLab](https://diglab.duke.edu/projects/3d-digging-at-catalhoyuk/) |
| **Published Songo Mnara surveys and excavations** | Buildings, open-space sampling and activity distributions. | Validate meaningful open space and overlapping territories, not just roof coverage. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/00438243.2016.1179128?utm_source=chatgpt.com) |
| **Google Open Buildings** | Building footprints across extensive parts of Africa, Asia and Latin America; temporal products cover **2016–2023**, with approximately **4 m effective resolution**. | Study building addition and neighborhood expansion. Footprints are not parcels, and narrow alleys can be unresolved. [Google Research](https://sites.research.google/gr/open-buildings/) |
| **DLR World Settlement Footprint Evolution** | Annual settlement extent, **1985–2015**, **30 m**. | Validate larger-scale expansion only. Its backward reconstruction from a 2015 settlement mask is unsuitable as an independent record of all disappeared settlements. [EOC Geoservice](https://a.geoservice.dlr.de/web/datasets/wsf_evo) |
| **OpenStreetMap with OSMnx** | Contemporary walkable and other transport networks. | Compare connectivity and route structure. Use walking-relevant networks; mapping timestamps are not construction dates. [OSMnx](https://osmnx.readthedocs.io/en/stable/user-reference.html) |

## 6.2 Core scholarly reading

For the historical framework, start with **M. R. G. Conzen, *Alnwick, Northumberland: A Study in Town-Plan Analysis* (1960)**, alongside **Whitehand’s “British urban morphology: the Conzenian tradition” (2001)**. The latter is a useful guide to the framework and its development. [University of Birmingham](https://research.birmingham.ac.uk/en/publications/british-urban-morphology-the-conzenian-tradition/)

For property and physical change, prioritize **Stell and Tait, “Framework and form” (2016)** and **Tait, “Burgage plot patterns and dimensions in four Scottish burghs” (2009)**. For comparative institutional interpretation, pair **Hakim (2007)** with **Abu-Lughod (1987)** rather than treating either as a universal morphological template. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/framework-and-form-burgage-plots-street-lines-and-domestic-architecture-in-early-urban-scotland/4FC18665945BC7A9144C4C9165838A5A)

For non-European spatial mechanisms, the **Kyoto machiya study**, **Beijing hutong grammar**, **Ahmedabad documentation**, and **Wynne-Jones and Fleisher’s Songo Mnara research** complement one another. They concern different scales and kinds of evidence, which is an advantage for avoiding a single-region model. [J-STAGE](https://www.jstage.jst.go.jp/article/jusokenold/34/0/34_0611/_article/-char/en)

## 6.3 What remains uncertain or contested

**Morphology does not uniquely identify causation.** Similar streets and plots can result from allocation, inheritance, copying, shared construction systems or negotiated local coordination. Treat visual fit as necessary but insufficient validation.

**Comparative numerical coverage is thin.** Dovey and colleagues’ 2026 study examines **15 selected neighborhoods**, each **4 ha**, at three development phases. It is unusually relevant, but its authors explicitly acknowledge that some interpretations of agency remain speculative. It is not a source of universal transition probabilities. [Springer](https://link.springer.com/article/10.1057/s41289-026-00344-4)

**Modern informal settlements are not direct substitutes for ancient settlements.** Their observations are most useful for identifying possible operations and constraints, not importing modern construction, tenure or transport assumptions into early farming worlds.

**Preservation and mapping affect apparent form.** At Songo Mnara, investigation of earthen buildings and open areas complicates interpretations based only on conspicuous stone architecture. A simulation should not be calibrated solely against what survives most visibly. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/0067270X.2020.1841978)

**A morphological sequence is not a clock.** Infill, clearance, amalgamation and renewed building should respond to demographic and institutional events. The evidence does not justify forcing every town through a fixed burgage cycle.

---

## Final recommendation

The highest-value design decision is to make **rights, access and parcel history persistent**.

Give every alteration a cause: a new household, a profitable workshop, a blocked passage, a negotiated inheritance, a shared well, a contested extension or a collective improvement. Let those actions modify one another over time.

That approach can produce narrow alleys, long plots, courtyard clusters, locally regular rows, retained commons and abrupt redevelopment **without scripting any of them as an era’s required appearance**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928eb-1720-83ea-bb6b-496e038bc023)
