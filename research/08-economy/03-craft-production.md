# Craft production and workshops: a simulation-ready report for TCE

## Executive conclusion

**Model a workshop as people coordinating a sequence of operations around scarce tools, workspaces, and batch-processing equipment—not as a building that converts “labor” into goods at a constant rate.**

Three distinctions are essential:

**Active labor is not elapsed time.** Fermenting beer, drying pottery, and cooling a kiln tie up materials and equipment without requiring every worker to remain continuously occupied. Experimental pottery production and historical brewing instructions make this distinction particularly clear. [EXARC](https://exarc.net/issue-2023-3/ea/experimenting-ancient-greek-pottery-production-process)

**Product specifications matter as much as the craft label.** “A pot,” “a nail,” or “a piece of furniture” is not a sufficiently defined output unit. Research on Greek pottery distinguishes decoration tasks lasting minutes from elaborate painting taking many hours; neither measures the complete production of the vessel. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-hellenic-studies/article/professional-and-parttime-potters-and-painters-modelling-the-production-of-athenian-black-and-redfigure-painted-pottery/FD149DD27CB02B6890EE8F4CC7451613)

**Productivity, enterprise size, and institutional form are separate variables.** Skilled workers could operate within households, independent businesses, merchant-controlled networks, or state establishments. Sixteenth-century Jingdezhen, for example, had mobile artisans moving between imperial and private kilns despite administrators’ attempts to control them. [Cambridge University Press](https://www.cambridge.org/core/books/abs/city-of-blue-and-white/skilled-hands-managing-human-resources-and-skill-in-the-sixteenthcentury-imperial-kilns/9B1F0E78B4D9E39A3879380C52F5BF4D)

The evidence supports useful numerical anchors, but not a universal historical recipe book. Below, **measured or documented quantities are separated from proposed simulation defaults**. The latter are implementable starting assumptions, not disguised historical observations.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Represent production as linked operations

The following are appropriate starting process graphs. Different regional techniques should substitute operations rather than merely change a global productivity multiplier.

| Craft | Core operations | Important constraints to represent |
| --- | --- | --- |
| **Milling** | Clean or prepare grain → grind → optionally sift → package | Grain properties, flour fineness, operator effort, stone condition, available mechanical power |
| **Baking** | Mix → knead or otherwise develop dough → rest/prove where applicable → divide/shape → bake → cool | Oven loading, heating overhead, synchronized dough readiness, fuel, moisture loss |
| **Smithing** | Prepare stock → heat → forge → join where necessary → heat-treat where appropriate → finish | Forge and anvil access, task-specific skill, stock dimensions, tool changes, scrap and rework |
| **Pottery** | Prepare clay → form → partially dry → trim/finish/decorate → dry → load/fire → cool → sort | Drying space and weather, vessel geometry, kiln capacity, firing expertise, correlated batch losses |
| **Carpentry** | Obtain suitable timber → season where required → convert into stock → mark/cut → join → finish | Wood condition, stock dimensions, saw crew, cutting and sharpening tools, joinery complexity |
| **Brewing** | Prepare malt or other substrate → crush → mash/extract → heat as required by the recipe → cool → ferment → transfer/store | Water and fuel, vessel capacity, temperature, cleanliness, storage time, product-specific fermentation |

These sequences are supported by experimental grinding and pottery research, historical baking reconstruction, Holtzapffel’s woodworking description, and eighteenth-century brewing instructions. They should be treated as **families of techniques**, not one universal process for each commodity. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440325001414?utm_source=chatgpt.com)

For a batch of \(q\) units, define baseline labor demand as:

\[
L\_{\text{batch}}=\sum\_j \left(S\_j+q\,\ell\_j\right)
\]

Here, \(S\_j\) is setup labor in **person-hours per batch**, and \(\ell\_j\) is variable labor in **person-hours per unit** for operation \(j\). Apply worker skill and tool effects at the operation level.

Separately track:

\[
\text{accepted output}=q\times y
\]

where \(y\) is the saleable yield after rejects, spoilage, and breakage.

**Implementation rule:** labor availability, station availability, and elapsed-process completion are separate constraints. A worker finishing the active part of a brewing operation becomes available for other work while the fermentation vessel remains occupied.

### 1.2 Tools improve particular operations, not everything equally

A tool can reduce task time, improve material yield, improve consistency, reduce physical strain, or enable a previously impossible operation. These are different effects.

For a purely labor-saving improvement, use:

\[
M\_{\text{whole process}}=
\frac{1}{(1-f)+f/m}
\]

where \(f\) is the fraction of original labor affected and \(m\) is that operation’s speed multiplier.

**Example:** making an operation four times faster when it accounts for one-quarter of total labor raises whole-process labor productivity by only about **23%**, not fourfold. If an unchanged oven or kiln is already the bottleneck, output per day may increase even less.

Historical illustrations should remain product-specific. Adam Smith reported roughly **200–300 nails/day** for a smith unfamiliar with nail-making, **800–1,000** for a smith accustomed to it, and over **2,300** for highly specialized young nailers exerting themselves. These are contemporary observations and reports, not a controlled comparison with standardized nail sizes or hours. They support specialization and repetition effects—not a universal tenfold “master craftsman” bonus. [Adam Smith Works](https://www.adamsmithworks.org/documents/chapter-1-of-the-division-of-labour)

### 1.3 Specialization requires sufficient demand

For TCE, specialization should emerge when an agent can sell enough output to justify training, equipment, setup, and the loss of alternative work.

A useful break-even condition is:

\[
Q\_{\min}=\frac{F}{p-v}
\]

where \(F\) is fixed cost per planning period, \(p\) expected selling price, and \(v\) variable unit cost. This condition is meaningful only when \(p>v\).

An agent should compare expected craft earnings with farming, household production, another occupation, or leisure—not automatically become a full-time specialist after learning a recipe.

**Recommended consequence:** a small settlement can support occasional smithing, seasonal pottery, and shared baking facilities before supporting separate full-time firms. Export access can sustain specialization that local consumption alone cannot.

### 1.4 Allow shared facilities and production services

Ownership of ingredients, ownership of equipment, and performance of labor should be independently assignable.

A household may bring its own grain to a mill or its own dough to an oven. The operator then sells a processing service rather than purchasing inputs and selling the finished food. Historical baking evidence includes customers bringing dough to professional bakers. [Academia](https://www.academia.edu/9180722/_Our_Ech_Day_Bread_Reconstructing_Medieval_Bread)

For TCE, support:

* Own-input production for sale.
* Customer-owned inputs processed for a fee or toll.
* Communal equipment with scheduled access.
* Patron- or state-owned facilities using several possible labor arrangements.

This avoids forcing every miller, baker, or kiln operator to finance the entire value of the goods passing through the facility.

### 1.5 Separate skill from apprenticeship and legal status

An apprentice should have a **task-specific competence profile**, not a single “unskilled” flag. Preparatory and handling work can become productive before the apprentice can independently judge a forging heat, a kiln atmosphere, or a complex joint.

Formal training contracts were highly variable. English craft apprenticeships commonly specified **seven years**; comparable continental terms were often shorter, and the surveyed Amsterdam contracts ranged from **eight months to six years**. In late-seventeenth-century England, about half of apprentices had left after three or four years, so contractual duration cannot be treated as a universal time-to-competence measurement. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf)

**Implementation rule:** maintain separate fields for accumulated practice, mentor access, demonstrated task competence, contractual obligations, and legal permission to operate independently. Mentoring consumes experienced-worker time; a completed legal term should not create an instantaneous productivity jump.

### 1.6 Give guilds several possible effects

The literature does not support a universal positive or negative guild multiplier. Epstein emphasizes training, skill transmission, and technological adaptation; Ogilvie emphasizes exclusion, political privilege, and rent extraction. [DOI](https://doi.org/10.1017/S0022050700021124)

Represent guild activities separately: training enforcement, inspection and reputation, dispute resolution, entry restrictions, production restrictions, and political bargaining. Their effects should depend on what the institution actually does and whom its rules exclude.

### 1.7 Model putting-out as a financing and coordination network

A putting-out arrangement can be implemented as follows: a merchant advances materials or credit; distributed households perform specified operations using household labor and often household equipment; the merchant collects output, inspects it, pays an agreed piece rate, and sells or forwards it.

For TCE, each contract should specify input ownership, allowed material losses, expected quality, delivery timing, payment, and liability for rejected work. Merchant capacity is constrained by **working capital, transport, inspection, and information**, not merely by the number of contracted households.

The merchant’s network may therefore become large without a large factory building. Smith’s text also preserves an account of suppliers extending materials and consumption credit to nailers, illustrating how production and household finance could become intertwined. [Adam Smith Works](https://www.adamsmithworks.org/documents/chapter-1-of-the-division-of-labour)

**Proto-industry is not an inevitable stage leading to factories.** Treat rural market-oriented production as an organizational possibility that can expand, stagnate, coexist with factories, or disappear. Comparative research specifically challenges the idea that putting-out was only a temporary European transitional form. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1016/0362-3319%2890%2990013-A?utm_source=chatgpt.com)

### 1.8 Keep productivity, accounting cost, price, and wages distinct

Recommended unit-cost accounting is:

\[
c=
\frac{
\text{purchased inputs}+\text{fuel}+\text{paid labor}
+\text{capital services}+\text{transport}
-\text{byproduct receipts}
}{
\text{accepted output}
}
\]

Maintain an additional **economic-cost** estimate that includes unpaid household labor’s opportunity cost. A household workshop may remain cash-positive while earning very little per hour.

Production functions establish feasibility and cost pressures. Posted prices and wages should additionally respond to demand, inventories, bargaining, outside options, market power, and institutions.

For regulated bread markets, distinguish fixed prices from fixed quantities: historical bread assizes often adjusted loaf weight as grain prices changed. A simulation that fixes both without representing the actual rule will generate misleading results. [Academia](https://www.academia.edu/9180722/_Our_Ech_Day_Bread_Reconstructing_Medieval_Bread)

---

## 2. Parameters: historical anchors and usable starting values

### Reading the tables

**Documented/experimental** means an observed quantity, historical account, or reconstruction with stated conditions. **Derived** means arithmetic applied to those quantities. **Proposed** means a simulation assumption.

Confidence below concerns transfer into TCE. A well-documented observation can still have low generalizability across products, regions, and centuries.

### 2.1 Quantitative evidence worth anchoring to

| Process and context | Documented quantity | Simulation interpretation | Confidence and limitation |
| --- | --- | --- | --- |
| **Reciprocal/saddle-quern grinding**, archaeological experiments, principally wheat | **0.3–1.3 kg grain/hour** | Derived grinding labor: **0.77–3.33 person-hours/kg grain** | **Moderate.** Grain, fineness, experience, and apparatus matter. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440325001414?utm_source=chatgpt.com) |
| **Reciprocal grinding**, ethnographic observations | **2–4 kg/hour** | Derived **0.25–0.50 person-hours/kg** | **Moderate within cases; low for direct comparison.** Often millet or pooled grain categories rather than the experimental wheat. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440325001414?utm_source=chatgpt.com) |
| **Iron-Age-type rotary querns**, wheat/barley experiments | **3–5 kg/hour** | Derived **0.20–0.33 person-hours/kg grain** | **Moderate.** Grinding only, not all grain processing. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440325001414?utm_source=chatgpt.com) |
| **Monticello nailery**, 1795 | Reported workforce **12 enslaved boys**, ages **10–16**; reported output **8,000–10,000 nails/day** | Rough division gives **670–830 nails per worker-day** | **Moderate for records; low for normalized productivity.** Reports are from different dates that year; hours, nail mix, and supervisory labor are not standardized. [Monticello](https://www.monticello.org/encyclopedia/nailery) |
| **Bohol household pottery**, Philippines | Open firings of **up to 100 pots**; firing **1–3 hours** | Batch capacity and elapsed firing time—not labor per pot | **Moderate**, ethnographic case. Crew and preparation time must be added. [ResearchGate](https://www.researchgate.net/publication/284542475_Traditional_technologies_and_ancient_commodities_An_ethnoarchaeological_study_of_salt_manufacturing_and_pottery_production_in_Bohol_Centra_Philippines) |
| **Bohol pottery drying** | At least **one day** before final shaping, then **3–7 additional days** drying | Multiple work-in-progress states and substantial drying-space demand | **Moderate locally; low as universal climate parameters.** [ResearchGate](https://www.researchgate.net/publication/284542475_Traditional_technologies_and_ancient_commodities_An_ethnoarchaeological_study_of_salt_manufacturing_and_pottery_production_in_Bohol_Centra_Philippines) |
| **Experimental Hellenistic-style updraft kiln**, reconstructed in Britain | **10 hours firing**, followed by **48 hours cooling** | At least **58 hours** occupation across those stages, before loading/unloading | **Moderate for this reconstruction.** The five-person experimental team is not evidence that five workers were continuously necessary. [EXARC](https://exarc.net/issue-2023-3/ea/experimenting-ancient-greek-pottery-production-process) |
| **Athenian painted pottery**, estimates assembled in a production model | Approximately **15 minutes** for a simply decorated lekythos versus **16 hours** for an elaborate krater’s painting | Decoration must be a separate specification-sensitive operation | **Low–moderate.** Reconstruction estimates, not complete vessel-production times. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-hellenic-studies/article/professional-and-parttime-potters-and-painters-modelling-the-production-of-athenian-black-and-redfigure-painted-pottery/FD149DD27CB02B6890EE8F4CC7451613) |
| **English malting**, eighteenth-century instructions | Grain steeping **60–72 hours**; approximately **three weeks** from cistern to kiln in moderate weather; kiln drying **4–12 hours** | Long inventory residence with intermittent turning and handling | **Moderate for the described method.** Not three weeks of continuous labor and not a universal beer recipe. [Project Gutenberg](https://www.gutenberg.org/cache/epub/8900/pg8900-images.html) |
| **Pit-sawing**, Holtzapffel, 1847 | **Two-person crew** | Crew complementarity: missing one operator can stop that technique | **High for the described arrangement.** No defensible general board-output rate follows from crew size alone. [Lost Art Press](https://blog.lostartpress.com/2014/06/09/holtzapffel-on-pit-sawing/) |
| **Indian handlooms**, 1942 enquiry figures discussed by Nagar | **4 square yards per nine-hour day** for throw-shuttle looms; **7** for flying-shuttle looms | A documented **1.75× loom-output comparison** | **Moderate.** Loom-day is not person-day; product composition and supporting labor remain important. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.70145) |

**Do not turn the nailery figures directly into person-hours.** Assuming a ten-hour day would imply approximately **1.2–1.5 person-hours per 100 nails**, but ten hours is an additional assumption. The coercive labor regime also makes this an inappropriate universal benchmark for voluntary adult workshop employment.

### 2.2 Proposed initial recipe parameters where historical evidence is thin

The following are **authoring priors**, not measured historical ranges or statistical confidence intervals. They provide a first playable economy while preserving the ability to replace individual coefficients later.

Assume inputs and fuel are delivered to the workplace. Add hauling, marketing, maintenance, and upstream production separately.

| Proposed recipe | Defined output and input boundary | Initial active-labor range | Treatment of waiting and capacity |
| --- | --- | --- | --- |
| **Small water-powered grain mill** | Grain through one small milling installation; excludes grain cultivation and transport | Assume **20–100 kg/hour** throughput and **1–2 attendants** while operating: conditional **0.01–0.10 person-hours/kg** | **Low-confidence engineering prior.** Rate must vary with installation and water availability; queues and downtime are separate. |
| **Household bread batch** | **5 kg finished plain bread**, from purchased flour | **1–3 person-hours/batch**, or **0.20–0.60 hours/kg** | Specify flatbread versus leavened loaf; do not charge all resting time as labor. |
| **Small commercial bakery batch** | **50 kg finished plain bread**, from purchased flour | **4–10 person-hours/batch**, or **0.08–0.20 hours/kg** | Shared heating/setup; oven capacity and timing can prevent the labor rate from being achieved. |
| **Small wrought nails** | **100 nails**, initially specify approximately **50 mm** length and **4 g** each; supplied nail rod | **1–2 person-hours/100** | Low-confidence extrapolation from historical nail-making evidence; validate against the exact nail geometry. |
| **Simple edged implement** | One roughly **0.5 kg** plain implement, from suitable bar stock, including basic finishing and hafting | **5–12 person-hours/item** | No ore reduction or bar-making included; complex welded constructions require another recipe. |
| **Hand-formed plain pottery** | **20 simple 1–2 L vessels**, from prepared plastic clay, through forming and finishing | **6–20 person-hours**, or **0.30–1.00 hours/vessel** | Add clay preparation and allocated firing labor; drying is separate. |
| **Wheel-formed plain pottery** | Same product boundary as above, with a competent operator | **3–10 person-hours**, or **0.15–0.50 hours/vessel** | Wheel benefit applies to forming, not automatically to decoration, drying, or firing. |
| **Simple joinery** | One plain stool, from prepared seasoned boards; no turning, carving, or paint | **6–20 person-hours/item** | Timber conversion and seasoning excluded; add them when starting from logs. |
| **Small warm-fermented malt beverage batch** | **100 L finished beverage**, from crushed malt, through transfer into storage vessels | **10–25 person-hours/batch**, or **0.10–0.25 hours/L** | Start with **2–7 days** fermentation occupancy as a separate low-confidence recipe prior; malting is upstream. |

The numerical ranges in this table are deliberately provisional. Historical watermill documentation supports site-specific installations rather than a universal mill rate; baking reconstruction warns against deriving recipes or labor times from bread-price regulations; pottery and brewing evidence supports the separation of active work from elapsed process time. None of those sources establishes all the proposed coefficients above. [HimalDoc](https://lib.icimod.org/records/mryrh-zck92)

**A complete pottery example, using authored assumptions:** suppose 80 vessels require 0.35 person-hours each for forming and finishing, plus 12 person-hours of shared preparation, loading, tending, and unloading. Total labor is 40 hours. At a 90% saleable yield, the 72 accepted vessels embody **0.56 person-hours each**, before upstream extraction and transport. Drying for a week changes working-capital and space requirements, not that labor total.

### 2.3 Workshop staffing and capital requirements

There is no defensible universal “preindustrial workshop had \(n\) workers” coefficient. For TCE, use the following **initial organizational templates**, not claims about historical world averages.

| Workshop template | Proposed normal active crew | Fixed-capital bill to represent | Working-capital or space constraint |
| --- | --- | --- | --- |
| Household hand mill | **1 per quern** | Grinding-stone set; preparation/sifting equipment | Grain stock; household time |
| Small powered mill | **1–2**, with extra handling help when needed | Wheel or other prime mover, transmission, millstones, building and waterworks | Customer grain or purchased inventory; repairs and seasonal utilization |
| Bakery | **1–3** | Oven, troughs, work surfaces, peels, measuring equipment | Flour and fuel; dough/proving space; unsold bread |
| General smithy | **1–4** | Forge, bellows, anvil, hammers, tongs, cutting/finishing tools | Metal stocks, fuel, unfinished orders, repair backlog |
| Pottery household/workshop | **1–3 formers**, temporary firing help | Forming tools or wheels, preparation area, drying shelter, access to firing facility | Drying space; accumulated unfired stock |
| Carpenter/joiner | **1–3**, with technique-specific crews | Benches, saws, axes/adzes, planes, chisels, boring tools, clamps or fixtures | Suitable timber, seasoning/storage space, commissioned work |
| Small brewery | **1–4** | Heating, mashing, cooling, fermenting and storage vessels | Grain/malt, fuel, vessel occupancy, unsold or spoiled drink |

Price these capital goods through TCE’s own material and construction economy rather than assigning a timeless cash price.

A useful archival anchor illustrates the distinction between equipment and working capital: in June 1795, Jefferson accepted a nail-cutting machine offered for **US$40**, while also requesting **500 lb—about 227 kg—of suitable iron**. That price is for a particular machine, not a complete smithy, and the iron order is a separate financing requirement. [Founders Online](https://founders.archives.gov/documents/Jefferson/01-28-02-0300)

For fermentation capacity, a simple accounting identity is especially valuable. A proposed brewery producing **100 L/day** with **four days’ vessel residence** needs at least **400 L of usable fermentation capacity**, plus headspace and any allowance for cleaning or delays. Adding workers cannot overcome a missing vessel.

For an initial apprenticeship-based workshop, **one master, zero to two journeymen, and zero to two apprentices** is a reasonable authored template. It should not be a legal or technical cap: the historical nailery and Jingdezhen evidence already demonstrate why larger and differently organized establishments must remain possible. [Monticello](https://www.monticello.org/encyclopedia/nailery)

---

## 3. Variation across eras and world regions

### 3.1 Historical configurations, not mandatory stages

| Configuration | Historically important distinction | TCE implication |
| --- | --- | --- |
| **Foragers** | Sophisticated processing does not require agriculture. Natufian bread-like products at Shubayqa 1 predate farming by about **4,000 years**. | Permit skilled food processing and tool production without farms, money, or firms. Do not make bread inherently an agricultural-era unlock. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6077754/?utm_source=chatgpt.com) |
| **Early farming communities** | Household food processing can absorb substantial recurring labor; durable equipment changes that burden. | Household production remains visible and economically consequential even when no market transaction occurs. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440325001414?utm_source=chatgpt.com) |
| **Preindustrial commercial societies** | Household producers, independent workshops, regulated trades, merchant networks, and state facilities can coexist. | Separate ownership, employment, specialization intensity, and enterprise scale. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf) |
| **Industrializing societies** | Mechanization changes particular tasks and supply chains unevenly. Cheap machine-made intermediates may help downstream hand producers. | Allow hybrid establishments instead of replacing every craft with a factory simultaneously. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.70145) |
| **Modern economies** | Hand techniques and household enterprises can persist alongside industrial production, as contemporary Philippine pottery and Rwandan brewing studies illustrate. | Hand production should disappear only when its actual economic and social advantages disappear—not when a calendar threshold is crossed. [ResearchGate](https://www.researchgate.net/publication/284542475_Traditional_technologies_and_ancient_commodities_An_ethnoarchaeological_study_of_salt_manufacturing_and_pottery_production_in_Bohol_Centra_Philippines) |

### 3.2 Regional cases that change the model

**China: skilled labor markets inside and around state production.**  
Gerritsen’s study of sixteenth-century Jingdezhen describes competition for skilled workers between imperial and private kilns. Official attempts to bind labor did not eliminate worker mobility. This is a strong reason to give TCE state workshops recruitment problems, wage competition, incomplete enforcement, and dependence on experienced specialists. State ownership should not automatically imply unlimited obedient labor. [Cambridge University Press](https://www.cambridge.org/core/books/abs/city-of-blue-and-white/skilled-hands-managing-human-resources-and-skill-in-the-sixteenthcentury-imperial-kilns/9B1F0E78B4D9E39A3879380C52F5BF4D)

**South Asia: hand and machine production can be complementary.**  
Nagar’s 2026 reassessment distinguishes handspinning from handweaving: cheaper machine-spun yarn could benefit handloom weavers even as it displaced spinners. His reconstruction finds a different trajectory for the two activities, with substantial recovery in handweaving after earlier decline. The magnitude of Indian deindustrialization remains contested and depends partly on productivity assumptions; the robust modeling lesson is to evaluate each production stage separately. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.70145)

**Philippines: household specialization is not necessarily subsistence production.**  
In Bohol, pottery production involved household workspaces, gendered task participation, clay preparation, staged drying, and collective batch firing. Specialized pots also supplied another craft—salt manufacture. A small household establishment can therefore participate in an interdependent regional production chain without resembling a European guild shop. [ResearchGate](https://www.researchgate.net/publication/284542475_Traditional_technologies_and_ancient_commodities_An_ethnoarchaeological_study_of_salt_manufacturing_and_pottery_production_in_Bohol_Centra_Philippines)

**Africa: “brewing” must support different substrates and institutions.**  
A 2025 study of 132 Rwandan producers examined banana and sorghum brewing, not simply European barley beer reproduced elsewhere. The appropriate simulation response is alternative input and process graphs, with their own vessel needs, skills, and markets—not a regional modifier on a single universal beer recipe. [Popups ULiège](https://popups.uliege.be/2295-8010/pdf.php?id=2593)

**Europe: master–journeyman–apprentice is useful but not universal even within Europe.**  
Contracts, compensation, training durations, and access to independent practice differed between cities and trades. TCE should generate these arrangements from local rules and bargaining rather than import one standardized guild ladder everywhere. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf)

**Atlantic plantation economies: workshop output can involve coercion.**  
The Monticello nailery’s useful production records concern enslaved children. Production accounting must therefore be separate from a voluntary wage-employment model. Coerced work, household dependence, apprenticeship obligations, and freely negotiated employment cannot all be represented as ordinary labor-market contracts. [Monticello](https://www.monticello.org/encyclopedia/nailery)

---

## 4. Stylized facts and validation targets

A correct TCE implementation should reproduce the following patterns. Some are numerical validation cases; others are structural tests where assigning a universal historical percentage would be unjustified.

### 4.1 Food processing can consume a large share of available labor

**Synthetic validation case:** assume 1,000 people require 500 kg/day of grain grinding. At an authored rate of 1 kg/hour, that requires 500 person-hours/day; at 4 kg/hour, 125 hours. The difference is **375 hours**, or about **47 eight-hour worker-equivalents**.

These demand and working-day assumptions are illustrative, but the rates sit within the experimental technique ranges. The saved time should become available for other activities rather than vanish from the model. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440325001414?utm_source=chatgpt.com)

### 4.2 Craft output is often bursty

Pottery production should accumulate work in progress and release finished goods in batches, rather than emit a fixed fraction of a pot every tick. Documented firing and drying schedules provide direct timing checks. [ResearchGate](https://www.researchgate.net/publication/284542475_Traditional_technologies_and_ancient_commodities_An_ethnoarchaeological_study_of_salt_manufacturing_and_pottery_production_in_Bohol_Centra_Philippines)

For a fixed setup cost, operating at half the normal batch load doubles **setup cost per unit**, but not necessarily total unit cost. This is an important unit test for the production function.

### 4.3 Throughput and quality are not interchangeable

A workshop making plain containers should not automatically be “more productive” in an economic sense than one making elaborate painted vessels. It may produce more items while using less labor per item, but serving a different market. The Greek pottery estimates show why decoration and product specification need explicit treatment. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-hellenic-studies/article/professional-and-parttime-potters-and-painters-modelling-the-production-of-athenian-black-and-redfigure-painted-pottery/FD149DD27CB02B6890EE8F4CC7451613)

### 4.4 More workers do not always increase output

Adding potters does not immediately enlarge a kiln. Adding brewers does not shorten fermentation. Adding a second person to a task designed for one person may accomplish nothing; supplying the second operator of a two-person pit-saw technique may enable the operation entirely. [Lost Art Press](https://blog.lostartpress.com/2014/06/09/holtzapffel-on-pit-sawing/)

**Validation target:** marginal worker productivity should depend on the currently binding constraint, not remain constant at every staffing level.

### 4.5 Mechanization creates winners and losers within a production chain

Cheaper intermediate goods can expand downstream craft production. TCE should be capable of producing **declining handspinning alongside surviving or expanding handweaving**, rather than treating all textile workers as one occupation. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.70145)

### 4.6 Legal status is an imperfect signal of ability

A legally recognized master need not be the fastest operator at every task. An apprentice may become productive before completing an indenture; an experienced migrant may lack local authorization. The contractual evidence and Jingdezhen labor mobility make these distinctions important. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf)

### 4.7 A technically productive workshop can still fail financially

Supply interruptions, unpaid customer debts, unavailable supervision, or broken machinery can reduce realized production and cash flow. Monticello’s records contain examples of these problems. [Monticello](https://www.monticello.org/encyclopedia/nailery)

**Validation target:** workshop survival should depend on liquidity and realized sales, not merely on positive theoretical value added.

### 4.8 Large commercial networks need not imply large workplaces

A merchant can coordinate many small producers. Conversely, a large state facility need not have independent commercial firms inside it. This follows from treating putting-out and centralized production as different organizational solutions rather than successive size tiers. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1016/0362-3319%2890%2990013-A?utm_source=chatgpt.com)

---

## 5. Modeling recommendation for TCE

### 5.1 Minimum useful data model

The following is a design recommendation rather than a claim about an existing model:

```
Agent
  task competencies
  practice history and mentor links
  household obligations and available work time
  legal status, contracts, and outside options

Workshop
  ownership and management
  workers and work schedules
  stations, tools, condition, and maintenance needs
  input stocks, work-in-progress lots, finished inventory
  cash, debts, customer orders, and supply contracts

RecipeVariant
  product specification and accepted quality grades
  input quantities and ownership requirements
  operation dependency graph
  setup labor, variable labor, and minimum crew
  station occupancy and unattended process duration
  fuel/power requirements, yields, rejects, and byproducts

Institution
  access and entry rules
  training, inspection, and dispute-resolution functions
  taxes, tolls, privileges, and enforcement capacity
```

**Keep household, workplace, and firm distinct.** One household can operate several activities; several households can share a kiln; a merchant can contract with many workplaces; a workshop can process customer-owned materials.

### 5.2 Use complementary inputs with discrete technique choice

At the operation level, use bottleneck constraints: an anvil-dependent task requires the worker, appropriate stock, anvil access, and other specified equipment. Missing one cannot always be compensated for by adding more of another.

At the investment level, allow agents to choose among techniques. They compare expected throughput, labor requirements, quality, operating costs, reliability, financing, and available skills. Thus, a capital-intensive technique can remain unattractive when utilization is low.

This is more suitable for authored craft processes than allowing a smooth production function to substitute arbitrary quantities of labor for every missing tool.

### 5.3 Make learning gradual and task-specific

A useful skill representation has separate competencies for operations such as forming, decorating, firing, forging, heat-treatment, and finishing. Practice increases competence gradually; mentoring changes learning opportunities and consumes mentor time.

Do not give all masters the same multiplier. A highly skilled decorator may be an ordinary kiln operator. Nor should an agent lose all practical ability when a legal contract ends.

For v1, simplify skill into **speed, quality consistency, and eligibility for difficult tasks**. Avoid a detailed physiological model unless it materially affects play.

### 5.4 Schedule batches rather than simulating every hand movement

For 10k–50k agents, I recommend:

**Daily planning:** workshops evaluate orders, stocks, available labor, station queues, and cash. They choose batches and assign workers.

**Event-based execution:** schedule stage completions, drying readiness, fermentation completion, maintenance, collection, and delivery. Run more frequent updates only where interruptions or environmental conditions matter.

**Periodic commercial adjustment:** update posted prices, hiring intentions, piece rates, and investment plans on staggered schedules.

Unreal animations should depict these assignments and events. The Rust kernel does not need to simulate every hammer strike or hand rotation to produce visible daily life.

### 5.5 Preserve accounting and resource invariants

The important correctness tests are straightforward:

A person cannot contribute the same hour to two jobs. A station cannot process incompatible batches simultaneously. Materials cannot be sold while already committed to a batch. Customer-owned stock cannot silently become workshop property. Finished output must pass through actual production stages.

Where food processing changes mass, explicitly track added water, evaporated moisture, separated material, and waste. Do not equate one kilogram of grain, flour, dough, and finished bread.

Maintain two views of productivity: **technical performance while operating** and **realized output over the calendar period**. The difference captures idle time, missing orders, seasonal conditions, maintenance, and disruptions.

### 5.6 Existing models and games to borrow from

| Model or game | Useful element | Limitation for TCE |
| --- | --- | --- |
| **EURACE@Unibi** | Heterogeneous firms, labor and capital markets, finance, and technology-linked production provide useful economic coordination patterns. [Bielefeld University](https://www.uni-bielefeld.de/fakultaeten/wirtschaftswissenschaften/lehrbereiche/etace/eurace%40unibi/papers-and-model-document/) | It is a macroeconomic research model, not a historical workshop recipe database. Borrow decision architecture, not assumed craft coefficients. |
| **Loy’s Athenian pottery production model** | Explicit product differences, uncertain production estimates, and simulation-based reconstruction. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-hellenic-studies/article/professional-and-parttime-potters-and-painters-modelling-the-production-of-athenian-black-and-redfigure-painted-pottery/FD149DD27CB02B6890EE8F4CC7451613) | Its reconstructed productivity and division-of-labor assumptions are not independently observed facts. |
| **Eco** | Its recipe representation separates crafting time, labor, skill requirements, and workstations. [Eco - English Wiki](https://wiki.play.eco/en/Bathtub) | Game labor is a gameplay resource, not a verified historical person-hour. Borrow the separation of concepts, not balance values. |

### 5.7 What to simplify first

For v1, prioritize physical inventories, task-specific labor, batch timing, equipment occupancy, household outside options, and basic finance.

Keep detailed kiln thermodynamics, individual tool geometry, complex apprenticeship law, and multi-layer subcontracting outside the initial kernel. Their economic effects can initially be represented through authored process parameters and institutional rules.

**Do not simplify away unpaid production, waiting time, working capital, or product quality.** Those omissions would distort the economy more than a moderately inaccurate hammering coefficient.

---

## 6. Sources, datasets, and evidence limits

### Highest-value sources for implementation

| Source | Best use in TCE research |
| --- | --- |
| **Hora et al. (2025), “Energy expenditure during grain grinding using reciprocal quern and rotary quern,” *Journal of Archaeological Science* 180, 106292** | Grinding benchmarks; differences between time, effort, and energy. Supplementary Table 1 contains the study data. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440325001414?utm_source=chatgpt.com) |
| **Tomei and Jimenez Rivero (2023), “Experimenting with the Ancient Greek Pottery Production Process…”** | Production stages, kiln occupation, environmental constraints, and reconstruction limitations. [EXARC](https://exarc.net/issue-2023-3/ea/experimenting-ancient-greek-pottery-production-process) |
| **Yankowski (2019), “Salt Making and Pottery Production: Community Craft Specialization in Alburquerque, Bohol, Philippines,” *Ethnoarchaeology*** | Household organization, linked crafts, drying, and batch firing outside Europe. [ResearchGate](https://www.researchgate.net/publication/284542475_Traditional_technologies_and_ancient_commodities_An_ethnoarchaeological_study_of_salt_manufacturing_and_pottery_production_in_Bohol_Centra_Philippines) |
| **Loy (2025), “Professional and part-time potters and painters…” *Journal of Hellenic Studies*** | A simulation-oriented treatment of uncertain ancient production estimates and product heterogeneity. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-hellenic-studies/article/professional-and-parttime-potters-and-painters-modelling-the-production-of-athenian-black-and-redfigure-painted-pottery/FD149DD27CB02B6890EE8F4CC7451613) |
| **Minns and collaborators (2016), *Contracting for Apprenticeship in Early Modern Europe*** | Training contracts, remuneration, duration, early departure, and institutional variation. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf) |
| **Epstein (1998), “Craft Guilds, Apprenticeship, and Technological Change…”; Ogilvie (2014), “The Economics of Guilds”** | Competing interpretations that should become separately modeled mechanisms. [DOI](https://doi.org/10.1017/S0022050700021124) |
| **Gerritsen (2020), *The City of Blue and White*, especially chapter 9** | Skill mobility and competition between imperial and private production in China. [Cambridge University Press](https://www.cambridge.org/core/books/abs/city-of-blue-and-white/skilled-hands-managing-human-resources-and-skill-in-the-sixteenthcentury-imperial-kilns/9B1F0E78B4D9E39A3879380C52F5BF4D) |
| **Littlefield (1990), “The Putting-Out System: Transitional Form or Recurrent Feature of Capitalist Production?”** | Avoiding a compulsory household → putting-out → factory progression. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1016/0362-3319%2890%2990013-A?utm_source=chatgpt.com) |
| **Atack, Margo, and Rhode (2019), “‘Automation’ of Manufacturing in the Late Nineteenth Century,” *Journal of Economic Perspectives*** | The **1898 Hand and Machine Labor Study** is a particularly valuable source of task-level hand/machine comparisons. Replication data: **10.3886/E114031V1**. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.33.2.51) |
| **Nagar (2026), “Indian Deindustrialization Revisited,” *Economic History Review*** | Product-specific productivity assumptions and hand/machine complementarity. Replication data: **10.3886/E251096V1**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70145) |
| **Monticello Nailery records and Jefferson’s correspondence** | Output, staffing, equipment purchases, credit, and disruption—with explicit recognition of enslaved labor. [Monticello](https://www.monticello.org/encyclopedia/nailery) |
| **The London and Country Brewer (1736–38); Holtzapffel (1847); Fitch (2013)** | Primary brewing process descriptions, woodworking technique, and methodological caution in reconstructing medieval bread. [Project Gutenberg](https://www.gutenberg.org/cache/epub/8900/pg8900-images.html) |

### What remains thin or contested

The largest numerical gap is **complete, product-matched labor accounting**. For baking, brewing, general smithing, and joinery, the sources reviewed do not establish reliable universal person-hours per kilogram, liter, or item. The proposed recipe table supplies explicit assumptions where that evidence is missing.

Experimental archaeology also has a transfer problem: inexperienced operators may be slow, but modern materials, instruments, preparation, or expert supervision may make other parts of an experiment easier. A reconstructed kiln’s measured firing duration is useful without assuming that its total crew or construction history reproduces an ancient workshop. [EXARC](https://exarc.net/issue-2023-3/ea/experimenting-ancient-greek-pottery-production-process)

Institutional evidence has a different limitation: surviving regulations describe intended behavior, not necessarily compliance. Apprenticeship departures and Jingdezhen’s mobile workforce show why formal rules should operate through imperfect enforcement rather than dictate every agent’s behavior. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf)

For calibration, record each coefficient with **product specification, technique, worker experience, batch size, included operations, observed versus inferred status, source, and uncertainty**. Keep uncertain values replaceable, and test whether changing them alters prices, specialization, workshop survival, or settlement structure.

**Bottom line:** TCE needs one production system capable of representing both a household potter and a mechanized firm. The enduring structure is the same—specified goods, skilled operations, complementary equipment, work in progress, financing, and demand. Industrialization should change the available techniques and their economics, not replace that structure with a different economic model.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92873-8f28-83ea-906a-544e98d29734)
