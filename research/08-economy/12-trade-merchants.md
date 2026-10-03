# Long-distance trade and merchant networks: a simulation-ready report for TCE

**The strongest design for TCE is a network of physical shipments, local markets, and persistent commercial relationships—not a global market with a distance surcharge.** Merchants should face four distinct problems: moving cargo, financing the interval before payment, discovering where goods will be valuable, and getting other people to honor agreements.

Transport technology matters enormously, but its effects depend on geography and institutions. Roman transport reconstructions imply a very large advantage for maritime freight over wagons; nineteenth-century Indian railways could instead undercut road, river, and coastal transport. Neither relationship should become a universal multiplier for every world. [Academia](https://www.academia.edu/3166649/The_shape_of_the_Roman_world)

For TCE, historical eras should therefore be **reference configurations of capabilities**, not progression gates. A society can possess sophisticated merchant partnerships while depending on pack animals, or efficient ships while retaining costly customs procedures.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Separate movement costs, waiting costs, and commercial uncertainty

For a route \(r\), calculate delivery time as:

\[
T\_r=
\sum\_{e\in r}\frac{d\_e}{v\_e}
+\sum\_{n\in r}
\left(T\_{\text{waiting},n}+T\_{\text{handling},n}\right)
+T\_{\text{selling}}
\]

Here, \(v\_e\) depends on mode, slope, surface, current, wind, season, load, and operating hours. Waiting includes assembly of a caravan, departure windows, border queues, rest, and unavailable onward transport.

**Do not equate sailing speed, daily progress, and door-to-door delivery time.** Casson’s reconstruction distinguishes favorable open-water sailing from slower coastal movement, adverse winds, and stopovers. Modern evidence likewise identifies ports and multimodal facilities as important sources of delay and unreliability. [Penelope](https://penelope.uchicago.edu/Thayer/E/Journals/TAPA/82/Speed_under_Sail_of_Ancient_Ships%2A.html)

A merchant’s prospective profit can be represented as:

\[
\mathbb E[\pi\_r]
=
\mathbb E[\text{sale revenue}+\text{salvage}+\text{insurance payout}]
-C\_{\text{purchase}}
-C\_{\text{transport}}
-C\_{\text{fees}}
-C\_{\text{capital}}
-C\_{\text{insurance premium}}
\]

This is a **proposed TCE accounting rule**, not an estimated historical equation.

Several implementation details matter:

* **Sale revenue must reflect finite demand.** Selling a large cargo should exhaust some buyers’ budgets and lower the price received for additional units. Multiplying an unlimited cargo by the destination’s last observed price creates artificial arbitrage.
* **Transport costs must have recipients and resource requirements.** Carrier wages, fodder, repairs, port charges, and guards are not all goods destroyed in transit.
* **Profitability and liquidity are different.** A profitable voyage may be impossible because its owner cannot finance purchases and provisions before receiving payment.

For owned transport, charge operating costs and depreciation. For hired transport, charge the carrier’s price; do not additionally charge the merchant the carrier’s underlying costs.

For financing, a simple starting approximation is:

\[
C\_{\text{capital}}\approx K\_{\text{committed}}\,i\_{\text{annual}}\frac{T\_r}{Y}
\]

where \(Y\) is the number of days in the simulated year. Apply either an explicit borrowing cost or an opportunity-cost allowance as appropriate, without counting the same financing burden twice.

### 1.2 Make tradability emerge from cargo properties and local scarcity

A useful first approximation to the transport burden is:

\[
\text{freight burden as a share of purchase value}
\approx
\frac{c\_{\text{mass-distance}}\;d}{v\_{\text{mass}}}
\]

Here, \(v\_{\text{mass}}\) is the good’s value per unit mass. This immediately produces different trading ranges without assigning goods arbitrary “local” or “international” flags.

The following are **mechanical implications to implement**, rather than fixed historical classifications:

| Cargo characteristic | Emergent trade behavior |
| --- | --- |
| High value relative to mass and volume | Can support expensive overland movement and repeated handling. |
| Heavy, inexpensive, durable goods | Favor cheap water routes, short land legs, and large consignments. |
| Perishable goods | Require nearby buyers, preservation, rapid transport, or unusually high destination prices. |
| Goods expensive to produce but cheap to transport | Encourage regional specialization when reliable markets exist. |
| Essential goods scarce at the destination | May justify costly transport despite low value elsewhere. |
| Intermediate goods that lose mass during processing | Create incentives to process near the source, unless fuel, skills, or capital favor another location. |

Thus, do not restrict long-distance trade to luxuries. Cheap transport can support bulk staples: a modern reconstruction of six grain and oilseed trade systems finds maritime transport carrying **86% of traded tonnage** as the main mode and **98.4% of tonne-kilometres**. These figures describe that commodity sample, not all world trade. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11772242/)

For TCE’s early economy, give each good mass, volume, packaging requirements, spoilage behavior, and quality. Then let salt, grain, cloth, metal, timber, pottery, and preserved food acquire different trading ranges through prices and logistics.

**Cargo capacity must be net of provisions.** A caravan’s water, food, fodder, packaging, and equipment compete with merchandise for carrying capacity. Buying supplies along the way converts part of that burden into dependence on intermediate settlements.

### 1.3 Separate merchants, carriers, investors, and agents

A shipment need not be owned by the person accompanying it. The Muziris papyrus provides a particularly concrete ancient example of a logistics chain involving guarded desert transport, warehouses, agents, river movement, and customs procedures between the Red Sea and Alexandria. [Attalus](https://www.attalus.org/docs/select1/p187A.html)

TCE should therefore distinguish:

| Organizational pattern | Rules to represent |
| --- | --- |
| Household merchant | One household supplies capital, labor, storage, and risk-bearing; absences affect its other activities. |
| Voyage partnership | Separate contributions of capital and labor; specify profit shares, authority, and responsibility for losses. |
| Resident commercial agent | Holds or sells another owner’s goods; receives compensation; may delay, misreport, default, or be falsely accused. |
| Merchant network | Shares introductions, information, credit assessments, and dispute reports through actual relationships. |
| Merchant association or league | Collects dues and may negotiate privileges, provide representation, enforce membership rules, or coordinate sanctions. |
| Specialized carrier | Sells transport capacity without necessarily owning the cargo. |

These are composable arrangements, not a required sequence from “primitive” to “advanced.”

**The Hansa is especially useful as a network model.** Ewert and Selzer describe overlapping merchant, family, town, and foreign trading-office relationships rather than a single vertically integrated enterprise. TCE should borrow this distributed coordination, not turn a merchant league into a nation-state or one giant firm. [Peter Lang](https://www.peterlang.com/document/1067669)

#### Trust should be an incentive structure, not an ethnic attribute

A practical decision rule is:

\[
\text{gain from cheating}
<
\text{expected legal penalty}
+\text{collateral forfeiture}
+\text{discounted future business lost}
\]

Each component can vary independently. An honest agent may lack access to trusted introductions; a well-connected agent may exploit a reputation; a court may enforce debts slowly but effectively.

The **Maghribi traders are a contested case**. Greif emphasizes multilateral reputation mechanisms. Edwards and Ogilvie challenge the evidence for an exclusive, clearly bounded coalition and stress the use of formal legal institutions; Greif’s rejoinder disputes their interpretation. The evidence does not justify hard-coding either “community reputation replaced law” or “reputation did not matter.” [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1468-0289.2011.00635.x)

For TCE, maintain relationship-specific histories, delayed reports, collateral, contracts, and jurisdictional enforcement. Shared language or affiliation can lower the cost of finding and understanding partners; it should not confer innate honesty.

### 1.4 Let infrastructure solve concrete bottlenecks

Caravanserais were not simply medieval hotels. Their forms and locations reflected accommodation, animal management, water, security, and route conditions. Iranian examples were financed through several arrangements, including rulers, notables, merchant groups, and religious foundations. [Iranica Online](https://www.iranicaonline.org/articles/caravansary/)

Represent a caravanserai or trading station as a building with capacities for people, animals, storage, water, and services. Its effects should follow from those capacities:

A reliable water source can make a route feasible. Storage permits merchants to wait for buyers or onward transport. A protected courtyard reduces some theft exposure but does not eliminate hazards outside it. Repeated encounters create opportunities to hire guides, recruit carriers, and exchange information.

Likewise, a port should have landing suitability, navigable depth, handling labor, storage, berths, and onward connections. A faster ship is of limited benefit when cargo waits for scarce dockworkers.

**Convoys should have benefits and costs.** Shared escorts spread a fixed protection cost across more cargo, but assembly takes time and large groups consume more supplies. In TCE, congestion, water availability, and grazing pressure can prevent “always use the largest possible caravan.”

### 1.5 Model tariffs, tolls, customs, and smuggling separately

A jurisdiction may levy charges on entry, transit, sale, export, or use of a facility. These need different bases: per animal, per wagon, per unit of cargo, or as a percentage of assessed value.

There is no safe universal “premodern tariff.” The second-century CE Muziris evidence concerns a **25% customs duty** on a particular Indo-Roman import channel. That is a useful historical configuration, not a default for every Roman border or ancient society. [OUP Academic](https://academic.oup.com/edited-volume/61673/chapter/550528330)

Recommended TCE rules:

**Tax incidence.** A fixed charge per load bears more heavily on cheap goods than valuable ones. A percentage tax depends on assessed value, creating disputes and incentives to misclassify or understate cargo.

**Collection.** Customs requires officials, records or witnesses, inspection capacity, and control over routes. Revenue goes to an institution’s account; in-kind collections become physical stocks.

**Policy response.** Higher charges can divert traffic, reduce cargo size, encourage illicit exchange, or make a route unprofitable. Revenue can also finance protection and maintenance, partly offsetting the burden.

**Smuggling.** Compare the expected payoff from legal and illegal exchange, including additional travel or concealment costs, detection probability, confiscation, fines, and relationship consequences. Keep this an abstract economic choice.

Fisman and Wei’s study of China–Hong Kong trade discrepancies found that a **one-percentage-point higher tax rate was associated with roughly 3% more measured evasion**. The measure is a reporting discrepancy, including undervaluation and misclassification—not a direct count of smuggled tonnes and not a transferable universal elasticity. [Brookings](https://www.brookings.edu/wp-content/uploads/2016/06/20031029-3.pdf)

Independent toll collectors can also create excessive cumulative charges: each may ignore the traffic lost because of everyone else’s tolls. That failure can emerge from decentralized revenue decisions without a scripted “trade collapse” event.

### 1.6 Give information its own movement and reliability

A merchant should know dated observations, not the current price everywhere.

Store a price report as a market, commodity, observed price, observation date, source, and confidence. Reports travel with merchants, messengers, correspondence, or later communications technology. Their uncertainty grows as local conditions change.

Jensen’s study of Kerala fisheries found that mobile-phone adoption was associated with sharply reduced price dispersion and eliminated measured waste in the study setting. This demonstrates that better information can improve allocation without requiring faster cargo transport. [OUP Academic](https://academic.oup.com/qje/article/122/3/879/1879540)

For TCE, merchants should estimate both **price and market depth**. A report that “grain is expensive” is insufficient unless they also estimate how much can be sold. Otherwise, every merchant responds to the same shortage and creates a predictable arrival glut.

Production specialization should respond more slowly than trading decisions. Farmers and workshops should expand export-oriented output only after sustained expected demand, subject to land, skills, tools, and financing constraints.

### 1.7 Distinguish cultural exposure from technical adoption

Ancient Indian Ocean networks combined geographic, social, ethnic, and religious connections; seasonal sailing also encouraged extended stays. These are better foundations for cultural contact than an automatic technology bonus proportional to trade value. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/D32B1FF1DCA90833C02C80E7B37B166E/S1740022813000338a.pdf/networks_and_social_cohesion_in_ancient_indian_ocean_trade_geography_ethnicity_religion.pdf)

For TCE, use separate processes:

**Exposure:** people encounter an object, garment, building detail, story, practice, or person.

**Interest:** familiarity, prestige, usefulness, compatibility, and relationships affect whether they want to imitate it.

**Capability:** imitation succeeds only when the required materials, skills, tools, and organizational conditions exist.

A foreign ceramic vessel can inspire a decorative motif without revealing its firing technique. A visiting craftworker can transmit practical knowledge that the object alone cannot. An imported tool can create demand for a capability before local producers know how to make it.

Trade should create opportunities for contact, not guarantee cultural homogenization, tolerance, or technical convergence.

---

## 2. Parameters: measured anchors, reconstructions, and design priors

**Evidence labels:** **D** = documented or reported observation; **R** = historical reconstruction or econometric/model estimate; **P** = proposed TCE parameter.

Confidence below concerns the evidence **within its stated setting**. Even a well-documented local figure may transfer poorly to another climate, animal breed, vessel, or institutional regime.

### 2.1 Transport capacity and time

| Mode or constraint | Quantitative anchor | Context and qualification | Evidence / confidence |
| --- | --- | --- | --- |
| Pack donkey | **70–120 kg per trip** | FAO transport-development report; load anchor, not an ancient universal or a speed estimate. | D / medium. [FAOHome](https://www.fao.org/4/x5483b/x5483b0w.htm) |
| Pack llama | **25–30 kg; 15–20 km/day** | Andean pastoral transport described in FAO’s synthesis. | D / medium. [FAOHome](https://www.fao.org/4/t0665e/T0665E09.htm) |
| Pack camel | **200 kg; 25–35 km/day**, over **5–8 hours** | Eritrean report for mature camels carrying sorghum. | D / medium. [FAOHome](https://www.fao.org/4/w9980t/w9980T06.htm) |
| Bullock cart | **20–30 km/day** | Favorable road conditions in nineteenth-century India; poor seasonal accessibility. | D / medium. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) |
| River transport | About **65 km/day downstream**, **15 km/day upstream** | Favorable Indian conditions; upstream towing. | D / medium. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) |
| Ancient sailing, open water | **4–6 knots** with favorable winds | Casson’s reconstruction from historical voyages; not door-to-door speed. | R / medium–low. [Penelope](https://penelope.uchicago.edu/Thayer/E/Journals/TAPA/82/Speed_under_Sail_of_Ancient_Ships%2A.html) |
| Ancient coastal sailing | **3–4 knots** favorable; adverse progress could fall below **2–2.5 knots** | Route, wind, stops, and navigation strongly affect outcomes. | R / medium–low. [Penelope](https://penelope.uchicago.edu/Thayer/E/Journals/TAPA/82/Speed_under_Sail_of_Ancient_Ships%2A.html) |
| Coastal steamship | **More than 100 km/day** | Nineteenth-century Indian benchmark; major-port service. | D / medium. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) |
| Railway | **Up to 600 km/day** | Nineteenth-century Indian operating capability, not every consignment’s delivery rate. | D / medium. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) |
| Modern container logistics | **44 days** from export-port entry to destination-port exit, averaged across potential routes in the 2023 LPI report | Includes port processes and transport; not pure sailing time or a default for one route. | D/R / medium. [World Bank](https://www.worldbank.org/en/news/press-release/2023/04/21/world-bank-releases-logistics-performance-index-2023) |
| Caravanserai spacing | **30–40 km** on flat routes; **10–20 km** in mountains | Iranian examples; geography produces different spacing elsewhere. | D / medium. [Iranica Online](https://www.iranicaonline.org/articles/caravansary/) |
| Arabian Sea sailing seasons | Southwest monsoon approximately **June–September/October**; northeast winds **November–April/May** | Ancient Indian Ocean study; use regional wind calendars, not a global sailing season. | D / medium. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/D32B1FF1DCA90833C02C80E7B37B166E/S1740022813000338a.pdf/networks_and_social_cohesion_in_ancient_indian_ocean_trade_geography_ethnicity_religion.pdf) |

One knot is 1.852 km/hour. Thus, favorable 4–6-knot sailing corresponds arithmetically to roughly **178–267 km per continuous 24-hour period**. Applying that figure to a journey with night stops, unfavorable winds, or port waits would overstate delivery performance.

**Vessel capacity needs hull-specific content.** A canoe, river barge, coastal trader, and ocean-going vessel should not share a single “boat capacity.” For TCE, tie capacity, crew, draft, stability, and maintenance to authored vessel designs rather than an era label.

### 2.2 Freight costs and institutional costs

| Parameter | Quantitative anchor | Correct interpretation |
| --- | --- | --- |
| Roman relative freight cost | **Sea 1 : downstream river 5 : upstream river 10 : wagon 52** | Scheidel’s reconstruction using Diocletian’s price ceilings; a scenario benchmark, not ordinary observed invoices. R / medium. [Academia](https://www.academia.edu/3166649/The_shape_of_the_Roman_world) |
| Roman reconstructed absolute rates | Approximately **0.67, 3.4, 6.8, and 35 denarii communes per tonne-km**, respectively | Same reconstruction. Useful within that price system; not comparable to modern currency without additional assumptions. R / medium–low. [Academia](https://www.academia.edu/3166649/The_shape_of_the_Roman_world) |
| Early eighteenth-century England | **Sea 1 : river 5 : road 23** | Comparative historical ratio summarized by Scheidel; illustrates variation from the Roman configuration. R / medium. [Academia](https://www.academia.edu/3166649/The_shape_of_the_Roman_world) |
| Indian freight relative to rail | Rail = **1**; road **4–5**; river **2–4**; coastal shipping **1.5–3** | Nineteenth-century Indian comparison. D / medium. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) |
| Modern land/maritime unit freight | Median land cost approximately **16.5 times** maritime cost | Grain/oilseed landed-cost model using 2017–2021 trade; not all commodities or corridors. R / medium. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11772242/) |
| Value of faster delivery | **0.6–2.1% ad valorem equivalent per additional transit day** | Hummels–Schaur estimate from US imports and air/sea choices. **Not spoilage or daily interest.** R / medium. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.103.7.2935) |
| Return-load opportunity | Shipments toward high-activity markets approximately **14% cheaper** | World Bank finding consistent with better return-load opportunities; not a universal causal coefficient. R / medium. [World Bank](https://www.worldbank.org/en/topic/infrastructure/publication/shrinking-economic-distance-understanding-how-markets-and-places-can-lower-transport-costs-in-developing-countries) |
| Indo-Roman customs | **25% of assessed value** | Specific Muziris-related import regime. D / high for this setting. [OUP Academic](https://academic.oup.com/edited-volume/61673/chapter/550528330) |

The Roman ratios are especially useful for exposing a modeling error: **straight-line distance is not economic distance**. A long water route can be cheaper than a short land route. However, the exact 52:1 ratio remains dependent on interpretation of the source and reconstruction. [Academia](https://www.academia.edu/3166649/The_shape_of_the_Roman_world)

### 2.3 Derive early transport costs in labor units

When historical currency conversion is unreliable, derive costs from TCE’s own labor and provisioning economy.

Consider an **illustrative design assumption**, not a historical estimate: a porter carries 25 kg of saleable cargo and advances 25 km per working day.

\[
0.025\text{ tonnes}\times25\text{ km}
=0.625\text{ tonne-km per worker-day}
\]

That implies **1.6 worker-days per tonne-km** for the loaded leg. An equally long empty return doubles the journey labor allocated to the outward cargo to **3.2 worker-days per tonne-km**.

Now assume one handler manages five donkeys, each with 100 kg of net cargo, at 25 km/day:

\[
5\times0.1\times25
=12.5\text{ tonne-km per handler-day}
\]

That is **0.08 handler-days per tonne-km**, before animal feed, animal capital, equipment, losses, and the return journey.

The point is not that these assumed configurations describe all historical transport. It is that **payload, handling labor, provisions, and return utilization produce transport economics endogenously**.

### 2.4 Explicit TCE starting priors

The following values are **test parameters, not claims about historical averages**.

| Parameter | Proposed initial setting or sensitivity range | Purpose |
| --- | --- | --- |
| Human porter | **15–30 kg net commercial load; 15–30 km per travel day** | Establish an early human-powered baseline; vary with terrain and provisioning. |
| Donkey travel speed | Start at **25 km/day**; test **15–35 km/day** | Complements the documented load range without pretending the source supplies speed. |
| Ordinary stop handling time | **0.25–2 days**, plus queues | A placeholder until handling throughput is generated by workers and facilities. |
| Annual financing/opportunity cost | Test **5%, 15%, 30%** | Probe sensitivity to working-capital pressure; not an era-specific interest-rate schedule. |
| Catastrophic loss exposure | Test routes with **0.5%, 2%, 10%** loss probability over a **50-travel-day** exposure | Stress-test peaceful, insecure, and dangerous configurations; not historical frequency estimates. |
| Merchant destination search | **8–32 known candidate markets** per planning decision | Computational budget, not an anthropological claim. |

For the loss scenarios, convert the whole-exposure probability into a daily hazard:

\[
h=-\frac{\ln(1-P\_{50})}{50},
\qquad
P(\text{event in }\Delta t)=1-e^{-h\Delta t}
\]

This prevents changing simulation step size or splitting a route into more edges from changing its total risk.

---

## 3. Variation across eras and regions

These are historical configurations to inform authored capabilities—not mandatory stages.

| Setting | Evidence and distinctive organization | Consequence for TCE |
| --- | --- | --- |
| **Foragers: eastern Africa** | At Olorgesailie, Kenya, material movement over at least **25–50 km** is documented around **295,000–320,000 years ago**. Provenance establishes movement, not whether acquisition involved direct travel, gifts, or successive exchanges. [PubMed](https://pubmed.ncbi.nlm.nih.gov/29545508/) | Permit procurement expeditions and relationship-mediated exchange before professional merchants or money. Do not infer a market from an exotic object alone. |
| **Early farming: Near East and Mediterranean** | Geochemical work on Neolithic obsidian identified movement over hundreds of kilometres. Such evidence is much stronger for routes and source areas than for prices, payment methods, or total freight volumes. [Scientific American](https://www.scientificamerican.com/article/obsidian-and-the-origins-of-trade/) | Small volumes of valuable materials can sustain long connections while food remains mostly local. |
| **Bronze Age commercial cities: Assyrian–Anatolian networks** | Nineteenth-century BCE commercial records support quantitative reconstruction of merchant connections and trade geography. Barjamovic and colleagues use these records in a spatial trade model. [OUP Academic](https://academic.oup.com/qje/article/134/3/1455/5420484) | Writing and persistent commercial relationships can support complex exchange long before industrial transport. Avoid equating early technology with organizational simplicity. |
| **Ancient Indian Ocean** | Seasonal winds structured travel and stays; merchants relied on overlapping geographic and social networks rather than one uniform institution. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/D32B1FF1DCA90833C02C80E7B37B166E/S1740022813000338a.pdf/networks_and_social_cohesion_in_ancient_indian_ocean_trade_geography_ethnicity_religion.pdf) | Direction-specific departure windows, resident agents, and prolonged visits can produce both commercial and cultural connections. |
| **Iranian overland routes** | Caravanserai spacing and architecture varied with terrain, water, and security; mountain stages could be much shorter than flat-country stages. [Iranica Online](https://www.iranicaonline.org/articles/caravansary/) | Route viability can depend on a chain of service nodes, not simply total distance. |
| **Northern Europe: Hanseatic trade** | Merchant networks, towns, and overseas trading offices coordinated exchange across political boundaries. [Peter Lang](https://www.peterlang.com/document/1067669) | Give associations negotiated privileges and coordination powers without making them sovereign states or monopolizing all trade. |
| **China and Europe before industrialization** | Shiue and Keller find broadly comparable grain-market integration in China and much of Western Europe before the Industrial Revolution. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.97.4.1189) | Do not assign Europe an inherent market-efficiency bonus or assume industrial machinery is necessary for substantial integration. |
| **Mesoamerica and the Andes** | Smith describes Aztec periodic markets, professional merchants, textile/cacao money, and duties at urban access points. Andean llama transport operated under a different payload and terrain regime. [Academia](https://www.academia.edu/4618310/_The_Aztec_Empire_2015_A_paper_about_fiscal_organization_and_taxation) | Distinguish markets and money from coinage; distinguish commercial exchange from state redistribution; allow region-specific transport animals and waterways. |
| **Industrial India** | Rail introduced a different combination of freight rates, speed, and seasonal reliability from existing routes. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) | Infrastructure changes the reachable market and production incentives; it should not simply add a universal income bonus. |
| **Modern international systems** | Large-scale maritime movement coexists with costly inland legs, border processes, and heterogeneous landed costs. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11772242/) | Advanced technology does not abolish geography, institutions, congestion, or uneven access. |

For ecology beyond these cases, keep transport capabilities conditional. A camel configuration belongs to a particular animal, provisioning system, and environment; it is not a generic upgrade over every other pack animal. Likewise, a navigable river is a directional, seasonal corridor, not an unconditional “river trade bonus.”

TCE should also reuse its logistics system for **tribute, redistribution, gifts, military supply, and migration**. These movements need not be triggered by merchant profit. Their ownership, obligations, and decision rules differ even when the physical transport is identical.

---

## 4. Stylized facts and validation targets

A correct simulation need not reproduce the precise trade map of a historical civilization. It should reproduce the following conditional patterns.

| Pattern | Evidence or quantitative anchor | Practical validation |
| --- | --- | --- |
| **Heavy goods use cheap corridors disproportionately** | Historical freight ratios and modern grain shipping both show strong modal differences. [Academia](https://www.academia.edu/3166649/The_shape_of_the_Roman_world) | Compare cargo composition by mode, value density, and transport-cost share. Cheap water access should expand bulk trading ranges. |
| **Outbound and return conditions differ** | River direction and return-load opportunities substantially alter transport economics. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) | The same two settlements can have different freight prices and delivery times in opposite directions. |
| **Information matters independently of vehicle technology** | Kerala phone adoption improved allocation and reduced price dispersion. [OUP Academic](https://academic.oup.com/qje/article/122/3/879/1879540) | Improve reports without improving transport. Waste and mistaken shipments should decline, but physical delivery limits should remain. |
| **Infrastructure changes real economic outcomes** | Donaldson estimates roughly a **16% increase in real agricultural income** from Indian railroad access—the paper’s approximate interpretation of its log specification. [Dave Donaldson](https://dave-donaldson.com/wp-content/uploads/2018/03/Donaldson_RRRaj_AER.pdf) | In a comparable test economy, improved access should affect prices, crop choices, and income through trade. Do not award 16% automatically. |
| **Trade costs can fall without disappearing** | Jacks, Meissner, and Novy estimate a **23% decline in international trade costs over 1870–1913** in their historical framework. This is an inferred trade-cost measure, not freight alone. [University of Warwick](https://warwick.ac.uk/fac/soc/economics/staff/dnovy/tcaea.pdf) | Industrialization should reduce several frictions while leaving borders, distance, and information relevant. |
| **Market integration is not uniquely industrial or European** | Chinese and European grain-price evidence shows substantial preindustrial integration. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.97.4.1189) | Successful non-European-style configurations must be possible with appropriate geography and institutions. |
| **Networks combine local clustering and distant connections** | MERCURY explores how trader links and information exchange produce different archaeological distribution patterns. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/roman-bazaar-or-market-economy-explaining-tableware-distributions-through-computational-modelling/6C64CA37C247869A21A6DE6CE18F55FD) | Track local relationship density, inter-settlement links, and dependence on brokers. Avoid an automatically complete trade graph. |

### Additional mechanism-level checks

These are proposed TCE tests rather than historical point estimates.

**Price gaps should approach a no-arbitrage band, not necessarily zero.** For comparable goods, a persistent gap smaller than transport, finance, risk, and fees may support no trade. A larger gap can persist temporarily when capacity, information, or credit is constrained.

**Harbors, crossings, and transfer points should become commercially valuable.** Their importance should follow from traffic and avoided costs. Moving a bridge or making a river unnavigable should alter routes and business opportunities.

**Trade should sometimes buffer shortages and sometimes transmit them.** Access to independently affected suppliers can reduce local scarcity; dependence on one distant source can create vulnerability. Test independent harvest shocks separately from correlated regional shocks.

**Profits should attract entry but not guarantee convergence without disruption.** Entry can narrow margins, while delayed information and lumpy shipments can produce oversupply, merchant losses, and exit.

Measure these outcomes using quality-adjusted price differences, cargo tonnes and tonne-kilometres, delivery-time distributions, inventory shortfalls, capacity utilization, merchant cash flow, and supplier concentration. Price correlation alone is insufficient: shared weather, seasonality, currency changes, and product differences can confound interpretation.

---

## 5. Recommended representation for TCE

### 5.1 Core entities and ownership

| Entity | Minimum state |
| --- | --- |
| **Person** | Home, current location, occupation, skills, languages, relationships, needs, employment or partnership role. |
| **Merchant household or firm** | Owners, cash and claims, liabilities, inventories, contracts, known markets, price beliefs, counterparties. |
| **Shipment** | Cargo lots, owner, carrier, origin, intended destination, itinerary, condition, departure time, delivery terms. |
| **Transport unit** | Mode, capacity by mass and volume, crew, operating requirements, condition, location, availability. |
| **Convoy or voyage** | Participating units, leader, escorts, common route, departure conditions, shared exposure. |
| **Market** | Actual inventories and offers, buyer budgets, handling capacity, local price history, access rules. |
| **Institution** | Jurisdiction, privileges, taxes, enforcement capacity, services, accounts, membership or obligations. |

A merchant should remain a persistent person or organization. The same individual can travel, return home, hire an agent, change partners, inherit assets, or go bankrupt.

A convoy can aggregate movement calculations while retaining the identities and ownership of its participants. Aggregation should save computation, not erase the people whose lives create the story.

**Keep enslaved or otherwise coerced people as people, never fungible cargo stacks.** Their movement belongs in the person, coercion, and institution systems, even when a transaction changes who claims authority over them.

### 5.2 Use a layered, directional transport graph

Maintain distinct edges for footpaths, pack routes, cart roads, rivers, coastal passages, and open-water routes. Connect modes through explicit transfer nodes.

Each edge should carry:

\[
\{\text{distance, access requirements, seasonal availability,
time model, capacity, operating burden, hazard, jurisdiction}\}
\]

A cargo owner can then choose among routes with different combinations of price, speed, reliability, and required capital.

**Do not let every character solve every route.** Merchants can compare a small number of known destinations and feasible itineraries. Cache routes by origin, destination, mode or cargo class, season, and network version. Invalidate them when meaningful infrastructure or access conditions change.

### 5.3 Separate physical truth from commercial beliefs

The kernel knows actual inventory and weather. Merchants know observations and reports.

Use physical truth to resolve movement and transactions; use agent beliefs to choose destinations and quantities. This creates mistaken voyages naturally without inventing random incompetence.

Useful stabilizing heuristics include conservative estimates of destination demand, liquidity reserves, diversification, and staggered planning. These should change decisions—not create emergency goods, guarantee profits, or silently erase losses.

### 5.4 Match update frequency to the process

For a 10k–50k-person world, a proposed implementation schedule is:

**Continuous or fine-step movement only where needed for visible behavior.** The economic journey remains authoritative regardless of rendering detail.

**Daily processing** for provisions, wages, inventory decay, local offers, and ordinary price observations.

**Event-driven processing** for departure, arrival, transshipment, inspection, contract settlement, and route disruption.

**Less frequent strategic planning** for opening a new route, joining an association, investing in transport, or expanding production.

These are engineering recommendations, not measured performance guarantees. Profile them against TCE’s actual pathfinding and household systems.

Avoid creating one actor per cargo item. A shipment can contain homogeneous lots with common owner, quality, and storage requirements. Split lots only when economically relevant.

### 5.5 Preserve accounting and physical invariants

Goods change location or ownership only through explicit events. Spoilage destroys usable quantity or quality. Theft generally transfers possession; it does not make the goods disappear from the world. Taxes transfer purchasing power or stocks to an institution. Credit creates matching claims and obligations.

Arrival alone should not credit sales revenue: a sale must occur, or a pre-existing contract must specify payment.

For shared dangers, use shared events. One storm can affect an entire convoy or fleet; independently rolling a tiny loss probability for each cargo unit produces unrealistic diversification.

At TCE’s scale, an additional boundary decision matters: **is the simulated region the entire economy, or does it trade with a larger outside world?** External markets, if present, need explicit stocks, production responses, and purchasing capacity. An unlimited export buyer becomes an unlimited source of money.

### 5.6 Existing models and games worth borrowing from

| Model or game | Useful component | Limitation for TCE |
| --- | --- | --- |
| **ORBIS** | Directional, seasonal, multimodal route costs and the importance of transfer points. [Stanford History Department](https://history.stanford.edu/publications/orbis-stanford-geospatial-network-model-roman-world) | A transport-network model, not a complete economy of persistent merchants. |
| **MERCURY**, Brughmans and Poblome | Explicit trader networks, limited information, and comparison of resulting commodity distributions. Its published setup uses **1,000 traders in 100 markets**. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/roman-bazaar-or-market-economy-explaining-tableware-distributions-through-computational-modelling/6C64CA37C247869A21A6DE6CE18F55FD) | Designed to investigate archaeological hypotheses; production and social-network structure are much more abstract than TCE requires. |
| **Verschuur and colleagues’ grain landed-cost model** | Decomposition of production, transport, border, and handling costs across a multimodal network. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11772242/) | Static cost allocation, not endogenous household decisions or dynamic market prices. |
| **Patrician IV** | Convoys, warehouses, delegated trade routes, inventory-sensitive trading, and logistics interfaces. [RuneSoft](https://www.rune-soft.com/upload/documents/Manual_PDF_Patrician4%20.pdf) | Authored city production restrictions and progression rules should not become TCE’s economic foundations. |

For a first implementation, prioritize **real cargo, finite markets, directional transport, provisioning, and delayed information**. Add partnerships, resident agents, associations, customs, and credit as extensions of the same ownership and contract machinery—not as disconnected bonuses.

---

## 6. Sources, datasets, and evidence limits

### Useful calibration datasets and document collections

| Source | Best use | Main caution |
| --- | --- | --- |
| **ORBIS network data**, catalogued through the MERCURY project | Sites, routes, and multimodal connectivity benchmarks. [Project MERCURY](https://projectmercury.eu/datasets/) | Reconstructed and selective network; inspect the license and assumptions of each asset. |
| **Princeton Geniza Project** | Letters, contracts, transactions, people, and commercial relationships. [Princeton Geniza Project](https://geniza.princeton.edu/en/) | Surviving documents are not a representative census of trade or dishonesty. |
| **Barjamovic et al., “Trade, Merchants, and the Lost Cities of the Bronze Age”** | Commercial-record geography and spatial trade methods; replication data accompany the research. [OUP Academic](https://academic.oup.com/qje/article/134/3/1455/5420484) | Missing records and reconstructed locations; model outputs are not direct observations. |
| **Donaldson, “Railroads of the Raj”** | Infrastructure, commodity prices, trade costs, and agricultural outcomes. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199) | One historical setting with specific identification assumptions. |
| **Shiue–Keller, “Markets in China and Europe on the Eve of the Industrial Revolution”** | Comparative grain-market integration and associated replication materials. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.97.4.1189) | Requires careful product, geography, and period matching. |
| **Verschuur et al. landed-cost data and code** | Modern transport-cost heterogeneity and network decomposition; archived with the paper. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11772242/) | Landed costs exclude some markups and other components; they are not consumer prices. |
| **World Bank Logistics Performance Index 2023** | Modern shipment timing and logistics bottlenecks. [World Bank](https://www.worldbank.org/en/news/press-release/2023/04/21/world-bank-releases-logistics-performance-index-2023) | Use time measures with their exact start/end definitions and aggregation level. |

For merchant institutions, the essential paired reading is **Edwards and Ogilvie’s “Contract enforcement, institutions, and social capital: the Maghribi traders reappraised” alongside Greif’s rejoinder**. For network organization, use **Ewert and Selzer’s *Institutions of Hanseatic Trade***; for non-European maritime connections, **Seland’s “Networks and social cohesion in ancient Indian Ocean trade.”** These works support different institutional mechanisms rather than one settled universal model. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1468-0289.2011.00635.x)

### Where the evidence is weakest

**Universal loss and default rates are not established here.** Piracy, shipwreck, animal mortality, spoilage, and agent dishonesty need route- and commodity-specific calibration. The proposed risk values above are sensitivity scenarios, not reconstructed historical averages.

**Archaeological presence is not a freight invoice.** An imported object establishes some connection but may not identify the carrier, number of intermediaries, payment system, or volume. The forager stone evidence and the Roman tableware modeling illustrate different ways this inference problem arises. [PubMed](https://pubmed.ncbi.nlm.nih.gov/29545508/)

**Technical performance is not realized service quality.** Favorable sailing accounts, loaded-animal examples, and railway operating capabilities must be combined with waiting, provisioning, seasonality, and handling before becoming delivery-time parameters. [Penelope](https://penelope.uchicago.edu/Thayer/E/Journals/TAPA/82/Speed_under_Sail_of_Ancient_Ships%2A.html)

**Institutional effects are conditional.** A merchant association may make exchange more reliable for members while excluding outsiders. A customs regime may both collect revenue and impose burdens. Better connectivity can create new opportunities while increasing dependence on distant suppliers. These possibilities should be represented as competing mechanisms, not resolved by a single “trade efficiency” score.

### Bottom line

For TCE, the most productive abstraction is:

> **People and institutions move owned cargo through a costly, seasonal network, acting on incomplete information and enforceable—but imperfect—agreements.**

With those foundations, profitable routes, trading towns, resident merchant communities, specialized producers, toll disputes, shortages, fortunes, bankruptcies, and selective cultural borrowing can emerge from the same simulation rather than being separately scripted.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92893-ad00-83ea-9f47-1ca569172a54)
