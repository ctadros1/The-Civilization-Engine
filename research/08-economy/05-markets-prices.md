# Marketplaces and price formation in history

## A simulation-ready report for The Civilization Engine

**Keep posted prices, but do not make inventory the only state variable.** For TCE, the useful abstraction is a seller’s *asking price*, supported by inventories, expected replenishment, costs, customer relationships and local institutions. Whether that ask is written on a sign, announced orally or used as the opening of a negotiation can be a separate cultural and institutional choice.

Historical markets were not one uniform system. Periodic markets, permanent shops, negotiated transactions, customary prices and regulated prices coexisted. Fixed-price retailing was not exclusively industrial or European: Mitsui’s historical account dates its cash-and-fixed-price retail policy in Edo to **1673**. Conversely, Geertz’s study of twentieth-century Sefrou, Morocco, describes permanent shops operating alongside a weekly market and extensive bargaining. These are examples, not successive stages every society must traverse. [Mitsui 350th Anniversary](https://mitsui350th.com/en/)

The central recommendation is to simulate **local trading opportunities connected by people, inventories, credit and transport**, rather than a universal market price with historical modifiers.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Market days concentrate otherwise insufficient demand

Skinner’s research on rural China explains periodic markets as a way to concentrate trading opportunities where daily local demand would not support the same range of sellers. Different settlements’ schedules also helped itinerant traders attend multiple markets; higher-order markets generally operated more frequently. His evidence does not support imposing one universal weekly calendar or requiring every neighboring market to meet on a different day. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952)

**TCE rule:** A trading venue becomes viable when expected trading margins cover attendance, travel, setup and institutional costs:

\[
\text{expected sales revenue}
-\text{goods cost}
-\text{travel and attendance cost}
-\text{fees}>0.
\]

Make periodicity an institutional choice rather than a population threshold. Farmers can remain producers most days and become sellers on market days. Buyers accumulate shopping requirements between sessions. Itinerant specialists choose circuits that fit opening times and travel durations.

Permanent shops become attractive when sufficiently frequent demand covers rent, staffing and tied-up inventory. They should **coexist with periodic markets**, not automatically replace them. A town might support daily bread sales, a livestock market every several days and a large seasonal fair.

Fairs deserve a distinct schedule and trading function. The medieval Champagne system comprised **six fairs annually, rotating among four towns, each lasting about six weeks**. It served long-distance merchandise exchange and financial settlement, not simply a larger version of daily household shopping. [ifo Institut](https://www.ifo.de/DocDL/cesifo1_wp3438.pdf)

**Implementation consequence:** Market calendars should affect visible journeys, temporary stalls, crowding, lodging demand and the timing of commercial information.

### 1.2 Production, ownership and market supply are different quantities

An economy can contain abundant food that is not currently offered for sale. Conversely, a household can sell produce after harvest and later struggle to purchase food.

Burke, Bergquist and Miguel’s Kenyan experiment found that access to appropriately timed credit allowed farmers to exploit seasonal storage opportunities instead of selling immediately and repurchasing later. Slavin’s study of the English and Welsh Great Famine emphasizes how market segmentation, preferential transactions and the timing of grain sales aggravated the consequences of harvest failure. Neither result supports treating aggregate harvest volume as immediately available market supply. [DOI](https://doi.org/10.1093%2FQje%2FQjy034)

**TCE rule:** Track separately:

| Quantity | Meaning |
| --- | --- |
| Physical stock | Everything currently owned |
| Committed stock | Seed, contractual deliveries, tax obligations and planned household consumption |
| Offered stock | Quantity the owner is currently willing to sell |
| Accessible stock | Offered goods a particular buyer can reach and is permitted to purchase |

A normal household plan can reserve seed and food before offering a surplus. However, these reservations must not be inviolable: debt, taxes or emergencies can induce distress sales, including sales that undermine future consumption or production.

**Hunger is not purchasing power.** Distinguish an unsuccessful purchase because stock is unavailable from a household that cannot afford the available food.

### 1.3 Haggling changes more than the cash price

Geertz’s Sefrou research describes bargaining over quality, quantity, packing and credit as well as price. Repeated buyer–seller relationships reduced information costs; bargaining did not imply that everyone repeatedly traded with anonymous strangers. [Google Groups](https://groups.google.com/g/iktisattarihi2008/c/4Gkg-IXXu4I)

**TCE rule:** Give each seller an ask, but let the transaction protocol vary:

* **Non-negotiated sale:** buyer accepts or rejects the quoted bundle.
* **Negotiated sale:** one bounded bargaining calculation adjusts price or terms.
* **Customary relationship:** remembered terms, credit access or priority service.
* **Institutional allocation:** entitlement, obligation or rationing replaces ordinary price selection.

There is no need to simulate many conversational bargaining rounds. One possible approximation is:

\[
p\_{\text{transaction}}=\max\left(r\_s,\;a\_s(1-d\_{bs})\right),
\]

where \(a\_s\) is the ask, \(r\_s\) the seller’s reservation price and \(d\_{bs}\) a negotiated discount reflecting information, relationship and bargaining circumstances. The buyer purchases only when the resulting bundle is acceptable and affordable.

This is a **modeling approximation**, not an estimated historical bargaining equation. Keep discount distributions configurable; the historical literature does not provide a defensible universal “medieval haggling percentage.”

### 1.4 Search costs sustain local price differences

A buyer should compare a few feasible offers, not all sellers in the world. The relevant cost is approximately:

\[
\text{generalized purchase cost}
=
\text{money payment}
+\text{travel cost}
+\text{time cost}
+\text{expected quality or default loss}.
\]

Relationships can make a somewhat more expensive seller preferable because the buyer knows the quality, receives credit or avoids another journey.

Information and transport are complementary. Jensen’s study of Kerala’s fishing industry found that mobile-phone adoption during **1997–2001** was associated with sharply reduced price dispersion and the elimination of waste in the studied setting. Information mattered because fishermen and wholesalers could alter where they traded; it was not a substitute for physical movement. [EconPapers](https://econpapers.repec.org/RePEc%3Aoup%3Aqjecon%3Av%3A122%3Ay%3A2007%3Ai%3A3%3Ap%3A879-924.?utm_source=chatgpt.com)

**TCE rule:** Store price observations with a location, quality, quantity and timestamp. A price heard several days ago is an uncertain opportunity, not an executable offer.

### 1.5 Inventory targets must follow replenishment and crop calendars

A fixed inventory target creates a serious artifact: a grain merchant appears grossly overstocked immediately after harvest and dangerously understocked just before the next harvest, even while following a sensible annual storage plan.

**TCE rule:** For a retailer, use:

\[
I\_t^\*=
\sum\_{h=1}^{L\_t}\widehat D\_{t+h}+B\_t,
\]

where \(L\_t\) is the expected time until feasible replenishment, \(\widehat D\) expected sales and \(B\_t\) a safety reserve.

For a seasonal producer or bulk grain holder, use a **planned stock trajectory until the next harvest or import opportunity**, rather than the retailer’s short replenishment target. Forecasts should use information available to the agent, not knowledge of future weather.

A candidate posted-price controller is:

\[
\Delta\ln a\_t=
\operatorname{clip}
\left[
\alpha z\_I+\beta u\_t+
\gamma\ln\left(\frac{p\_t^{ref}}{a\_t}\right)
-\eta e\_t,\;
-\delta\_-,\delta\_+
\right].
\]

Here:

| Term | Meaning |
| --- | --- |
| \(z\_I\) | Bounded shortage relative to the seller’s seasonally appropriate target |
| \(u\_t\) | Unfilled **funded** demand observed during trading |
| \(p\_t^{ref}\) | A learned reference incorporating procurement, carrying costs and expected scarcity |
| \(e\_t\) | Urgency to dispose of stock approaching spoilage |
| \(\delta\_-,\delta\_+\) | Maximum downward and upward change per price review |

This is an **engineering proposal requiring calibration**. It should not be described as historical sellers’ actual algorithm.

Important safeguards are that sellers do not raise prices indefinitely merely because they hold no stock, and that production cost is not an absolute price floor. A distressed seller may rationally sell below cost; a persistently unprofitable firm must eventually contract or exit.

### 1.6 Storage links harvest prices to later prices

For an unconstrained, risk-neutral trader, a simplified condition favoring storage is:

\[
(1-\ell)\,\mathbb E\_t[p\_{t+\Delta}]
>
(p\_t+c\_{\text{store}})(1+r\_\Delta),
\]

with all terms defined over the same storage interval. Here \(\ell\) is physical loss, \(c\_{\text{store}}\) upfront storage cost per purchased unit, and \(r\_\Delta\) the financing or opportunity cost over that interval. Risk and liquidity needs can make the required return higher.

Competitive-storage models show how nonnegative inventories can produce nonlinear price behavior: stocks cushion some shocks, but once exhausted they cannot become negative to cushion further shortages. Deaton and Laroque’s model is an important reference, although its basic version does not explain every empirical feature of commodity prices. [Professor Sir Angus Deaton](https://deaton.scholar.princeton.edu/publications/behaviour-commodity-prices)

Historical calibration needs caution. Van Leeuwen, Földvári and Pirngruber show that Babylonian barley and date harvests occurring in different seasons complicate comparison with English grain seasonality. A smaller seasonal rise does not automatically establish cheaper credit or better storage. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-global-history/article/markets-in-preindustrial-societies-storage-in-hellenistic-babylonia-in-the-medieval-english-mirror/67553D932830BDAE140A0DB25EE7209D)

**TCE rule:** Generate seasonal prices from harvest timing, stocks, credit, losses and alternative foods. Do not impose a universal sinusoidal “winter price bonus.”

### 1.7 Traders integrate markets only when movement is profitable

For a shipment from town \(i\) to town \(j\):

\[
(1-\ell\_{ij})\mathbb E\_t[p\_j(t+\tau\_{ij})]-p\_i(t)
>
c\_{\text{freight}}+c\_{\text{finance}}+c\_{\text{tolls}}+c\_{\text{risk}}.
\]

Every cost must use the same unit—for example, currency per kilogram loaded. Expected destination revenue must also account for the quantity that can actually be sold.

This creates a **no-arbitrage band**: price differences can persist because they are smaller than the cost of exploiting them.

**Illustrative test, not historical data:** Grain costs 100 units at the source and 130 at the destination. Moving it costs 25. Trade initially offers a margin of 5. If the destination price falls to 120, the route is no longer profitable. Integration does not require both prices to become 100.

Historical evidence supports this mechanism without supporting a universal convergence speed. Donaldson’s study of colonial Indian railways finds reduced trade costs and interregional price gaps. Shiue and Keller find broadly comparable market performance in late-eighteenth-century China and Western Europe overall, while England performed better than both the Yangzi Delta and continental Western Europe. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199)

**TCE rule:** Convergence must depend on traders’ capital, shipment capacity, journey time, information and destination demand. Roads and ports cannot teleport either goods or price knowledge.

---

## 2. Regulation: different institutions require different mechanics

### Bread assizes were not simply grain-price ceilings

The English **1266 Assize of Bread and Ale** linked bread requirements to grain prices. A prominent arrangement retained a customary monetary denomination while changing the required loaf weight. Davis’s reassessment emphasizes the economic and moral logic of the assize rather than treating it as an undifferentiated attempt to freeze prices. [vLex](https://vlex.co.uk/vid/assisa-panis-et-cervisie-808090549)

A useful TCE approximation is:

\[
\text{required loaf mass}
=
\frac{\text{customary loaf price}}
{\text{allowed bread cost per unit mass}}.
\]

The allowed cost can incorporate grain, milling conversion, fuel, labor and an allowed return. Actual historical schedules were more specific than this formula.

**Consequence:** Stable expenditure per loaf can coexist with a rising price per kilogram and fewer calories per purchase. Increasing grain prices need not halve loaf weight when non-grain costs remain important. Other regulatory systems can instead hold weight constant and revise the monetary price.

Ottoman *narh* regulation also cannot be reduced to one immutable price list: scholarship distinguishes regulated prices, procurement arrangements and freely negotiated transactions. Bursa’s **1502** market ordinance combined price provisions with manufacturing and quality standards. [Belleten](https://belleten.gov.tr/tam-metin/2173/tur)

The following are **recommended model mechanisms and conditional predictions**, not universal measured effects:

| Institution | What the simulation should do | Effects to allow, rather than guarantee |
| --- | --- | --- |
| Standard weights and quality inspection | Define legal units and grades; inspect actual deliveries; penalize short measure or adulteration | Lower uncertainty and fraud, offset by inspection costs or corrupt enforcement |
| Bread assize | Revise legal mass, price or both from specified input-price observations | More predictable terms; delayed revisions may squeeze bakers or consumers |
| Maximum price | Constrain legal transaction prices; distinguish legal and illicit channels | Lower payments for successful buyers, but possible queues, diversion, quality reduction or reduced future supply |
| Rationing | Allocate scarce goods through explicit entitlements, eligibility and purchase limits | Access depends on coverage and delivery; leakage and exclusion remain possible |
| Public grain reserve | Purchase, store and release actual inventory using an actual budget | Buffer shortages, but incur losses and possibly displace private storage |
| Market privilege or licensing | Restrict venue, occupation or entry; collect fees | Better organization or supervision, but potentially less competition |
| Export restriction or compulsory sale | Change permitted routes and owners’ disposal rights | More immediate local availability in some circumstances, but altered production, storage and trading incentives |

Qing China’s civilian granary system is a major historical example of organized public provisioning. Will and Wong’s *Nourish the People* documents both its administrative reach and the difficulties of maintaining, accounting for and mobilizing grain. It is a model for **stocks plus institutions**, not a costless price-stability bonus. [University of Michigan Press](https://press.umich.edu/Books/N/Nourish-the-People2)

For ceilings, avoid scripting either universal success or universal failure. A ceiling that restrains market power while supply remains remunerative differs from one set below delivered replacement cost without subsidy or public supply. Record **availability, quality, queues and household consumption**, not just the official price.

---

## 3. Parameters: evidence and starting assumptions

### 3.1 Empirical benchmarks

**Confidence applies to the stated context, not transferability.** “High” means well-supported documentation or measurement; “medium” signals greater reconstruction, interpretation or source limitations.

| Parameter or outcome | Observed value and context | Use in TCE | Confidence and source |
| --- | --- | --- | --- |
| Chinese periodic-market frequency | Common schedules of **2–3 sessions per 10-day cycle**; dates such as 3, 6 and 9 produce unequal intervals | Author calendar patterns rather than assuming seven-day weeks | High for documented systems; Skinner. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952) |
| Walking-market catchment | Skinner reports the most remote villagers generally **3.4–6.1 km** from their standard market | Check plausible walking catchments; do not impose a universal radius | Medium; historical-geographical reconstruction. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952) |
| Yoruba periodicity | Hodder’s 1962–63 mapping, discussed by Hill, distinguished **2-, 4- and 8-day** markets; nearly **500 four-day** markets were recorded | Test interlocking schedules and different market levels | Medium–high for surveyed geography, not every earlier century. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-african-history/article/abs/notes-on-traditional-market-authority-and-market-periodicity-in-west-africa1/31871FCD18E53DFB68BCC9345ADF2923?utm_source=chatgpt.com) |
| Aztec periodicity | A **five-day market week** in the documented system | Market calendars need not depend on European institutions | High for the documented convention. [Cambridge University Press](https://www.cambridge.org/core/journals/latin-american-antiquity/article/size-of-plazas-in-mesoamerican-cities-and-towns-a-quantitative-analysis/6246D2C5EF99A6FF01B34727D5CECEDD) |
| Champagne fair system | **6 fairs/year**, **4 towns**, approximately **6 weeks/fair** | Long-distance concentration and settlement windows | High for the system’s established form. [ifo Institut](https://www.ifo.de/DocDL/cesifo1_wp3438.pdf) |
| English bread denomination | A **farthing**, or **¼ penny**, with weight requirements related to wheat prices | Separate package price from unit-mass price | High for the statutory arrangement; enforcement varied. [vLex](https://vlex.co.uk/vid/assisa-panis-et-cervisie-808090549) |
| Maize seasonality | **33.1%** average estimated seasonal peak-to-trough gap in sampled African wholesale markets | Benchmark a comparable highly seasonal staple system | High within sample; Gilbert et al. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/?utm_source=chatgpt.com) |
| Rice seasonality | **16.6%** average estimated gap in the same study | Validate crop-specific differences | High within sample. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/?utm_source=chatgpt.com) |
| Tomato seasonality | **60.8%** average estimated gap in the same study | Test interaction of seasonal production and perishability | High within sample. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/?utm_source=chatgpt.com) |
| Credit-enabled storage opportunity | Kenyan experiment reports **29% return on investment** | Evidence that liquidity can prevent profitable storage | High for the study; **not** a universal or automatically annualized return. [DOI](https://doi.org/10.1093%2FQje%2FQjy034) |

The seasonal-price study covers **193 markets, 13 commodities and seven African countries**, with roughly **6–13 years of monthly observations** depending on the series. These are modern observations, not estimates for medieval Europe or early farming. A 33.1% peak-to-trough gap is not a ±33.1% seasonal fluctuation around the annual mean. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0306919216303840)

### 3.2 Proposed engineering priors—not historical estimates

These are starting points for experiments. Their status is **unvalidated**, and they should be replaced or narrowed through sensitivity testing.

| Control | Starting range | Units and purpose |
| --- | --- | --- |
| Price-review frequency | Once per open market session; daily for continuously operating shops | Reviews/session or reviews/day |
| Sales-memory half-life | **2–8 market sessions** | Smooth noisy individual purchases without forgetting changes indefinitely |
| Inventory-response gain | Approximately **2–10% ask adjustment** under a full-scale inventory-error signal | Per review, before other terms and caps |
| Maximum ordinary ask change | **5–15% upward or downward** | Per review; allow separate emergency/perishable behavior |
| Retail target stock | **1–3 expected replenishment intervals** of sales | Excludes annual crop-storage planning |
| Buyer comparison set | **2–5 feasible known sellers** | Per shopping decision, with exploration and remembered relationships |
| Regulation stress tests | Ceilings at **70%, 90% and 110%** of an unregulated comparison run’s price | Experiment design only; agents do not observe the counterfactual price |

Do **not** begin with a universal grain-storage loss rate, freight markup, demand elasticity or town-convergence half-life. Those should depend on the relevant physical system or be explicitly labeled scenario assumptions. Historical data generally identify market-level outcomes more readily than the decision coefficients of individual sellers.

For food demand, budget-constrained households choosing among substitute foods can generate an aggregate response. It is usually better to validate that response than to assign every person the same externally imposed grain-demand elasticity.

---

## 4. Variation across eras and regions

These are organizational possibilities, not mandatory development stages.

| Setting | Evidence and interpretation | TCE representation |
| --- | --- | --- |
| **Foragers and small-scale horticulturalists** | Ethnographic research documents extensive food transfers, with kinship, reciprocity and other mechanisms contributing in different circumstances. There is no single universal sharing rule. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/to-give-and-to-give-not-the-behavioral-ecology-of-human-food-transfers/45EBF9C9619A612ECCB77B6F81449F7D) | Household production and transfers can dominate provisioning. Some goods may be exchanged without creating a full-time merchant economy. |
| **Early farming villages** | Direct evidence is much thinner for actual transaction prices and bargaining behavior than for settlement, production or storage. Later market institutions should not be projected backward automatically. | Start with self-provisioning, obligations, reciprocal transfers and optional exchange. Storage or surplus alone does not logically require a monetary spot market. |
| **Ancient Babylonia** | Temin finds market-responsive behavior in surviving commodity-price records. The evidence supports meaningful price formation long before industrialization, not the claim that every allocation was a market transaction. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0014498301907740) | Permit prices, credit and merchants alongside other allocation institutions. Keep crop-specific calendars and non-market transfers. |
| **Mesoamerica** | Archaeological and textual research supports widespread marketplace exchange; plazas could accommodate trading and ceremonial functions at different times. [Cambridge University Press](https://www.cambridge.org/core/journals/latin-american-antiquity/article/size-of-plazas-in-mesoamerican-cities-and-towns-a-quantitative-analysis/6246D2C5EF99A6FF01B34727D5CECEDD) | Reusable public spaces, periodic attendance and specialized sellers. A dedicated market building is not a prerequisite. |
| **Rural China** | Periodic markets formed differentiated local and higher-order networks rather than identical isolated town markets. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952) | Hierarchical catchments, itinerant circuits and changing commercial roles as transport improves. |
| **West Africa** | Hill’s work documents market authority and varied periodic schedules, including the Yoruba evidence above. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-african-history/article/abs/notes-on-traditional-market-authority-and-market-periodicity-in-west-africa1/31871FCD18E53DFB68BCC9345ADF2923?utm_source=chatgpt.com) | Calendar coordination and market governance should be explicit institutions, not decorative regional styling. |
| **North African bazaars** | Geertz’s Sefrou case emphasizes costly information, bargaining and durable customer relationships. [Google Groups](https://groups.google.com/g/iktisattarihi2008/c/4Gkg-IXXu4I) | Reputation, inspection, repeat purchases and credit terms; avoid treating bargaining as random discounts. |
| **Ottoman towns** | Officially regulated prices and other transaction prices coexisted; quality and measurement were important regulatory concerns. [Belleten](https://belleten.gov.tr/tam-metin/2173/tur) | Good-specific regulations, separate official and realized prices, and variable enforcement. |
| **Pre-industrial Japan** | Mitsui’s account of the 1673 Edo business provides an example of cash and fixed-price retailing before industrialization. [Mitsui 350th Anniversary](https://mitsui350th.com/en/) | Non-negotiated sale can arise from a merchant strategy; it need not await an industrial technology unlock. |
| **Medieval and early modern Europe** | Periodic markets, long-distance fairs and detailed bread regulation occupied different institutional roles. [ifo Institut](https://www.ifo.de/DocDL/cesifo1_wp3438.pdf) | Separate household provisioning, wholesale exchange, credit settlement and regulated staples. |
| **Industrial transport networks** | Indian railway evidence shows reduced spatial price gaps and expanded trade, rather than the disappearance of local markets. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199) | Faster, larger shipments and different procurement networks; no automatic global price equality. |
| **Modern economies** | African seasonal-price evidence and Kerala’s information effects demonstrate that storage, liquidity and information constraints remain relevant. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/?utm_source=chatgpt.com) | Modern capabilities alter the same systems rather than replacing them with a different economic engine. |

A particularly important design lesson is that **market sophistication and industrial technology are separate dimensions**. The China–Europe comparison is evidence against making “integrated markets” an exclusively European or industrial-era capability. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.97.4.1189)

---

## 5. Stylized facts and validation experiments

A correct simulation should reproduce patterns conditionally, not force every world to display every pattern.

### A. Market calendars should produce economic rhythms

Opening days should concentrate arrivals, sales and information exchange. Between-session households need enough stocks—or access to alternative sellers—to avoid artificial starvation. Permanent shops and periodic peaks should be able to coexist.

**Test:** Change a settlement from daily trading to one session every five days while holding production constant. Check shopping frequency, household inventories, seller attendance and transport effort.

### B. Seasonal variation should differ by crop and trading network

The empirical table provides magnitudes for selected modern systems. It does not justify applying the same seasonal multiplier to every food.

**Test:** Compare a single-harvest, isolated staple economy with one having staggered harvests, a substitute crop and reliable imports. The latter should be capable of smoother prices. The Babylonian–English comparison makes crop-calendar differences especially important. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-global-history/article/markets-in-preindustrial-societies-storage-in-hellenistic-babylonia-in-the-medieval-english-mirror/67553D932830BDAE140A0DB25EE7209D)

### C. Shortages should produce nonlinear price spikes

Storage should absorb some production variation. Once accessible stocks are exhausted, an additional shortfall can have a much larger price and consumption effect. This is a central implication of competitive-storage models. [IDEAS/RePEc](https://ideas.repec.org/a/oup/restud/v59y1992i1p1-23..html)

**Test:** Apply identical harvest reductions to economies with high and low opening stocks. Compare peak prices, stockout duration, substitution and household food consumption—not merely average annual prices.

### D. Integration should narrow profitable gaps, not all gaps

Transport improvements should reduce the price differences that remain worth exploiting. Finite capacity should leave temporary profitable gaps, particularly after shocks. Historical railway evidence supports reduced interregional gaps. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199)

**Test:** Run the same two-town economy with cheap transport, expensive transport and a blocked route. Price adjustment must occur through purchases, shipments and arrivals, not a direct town-price synchronization function.

### E. Information improvements should work through changed behavior

Faster information should reduce mistaken journeys and improve destination selection where traders can respond. It should do little when every route is blocked or all vehicles are already fully committed.

**Test:** Reduce observation delays while leaving physical transport unchanged. Check trading destinations, waste and price dispersion. Jensen’s fisheries study supplies an empirical example of the pattern. [EconPapers](https://econpapers.repec.org/RePEc%3Aoup%3Aqjecon%3Av%3A122%3Ay%3A2007%3Ai%3A3%3Ap%3A879-924.?utm_source=chatgpt.com)

### F. Liquidity relief should change the opportunity it exploits

When more farmers can store, immediate post-harvest selling pressure can decline and later supply can increase. Profitable seasonal arbitrage should not remain an inexhaustible fixed return as participation expands. The Kenyan experiment explicitly found local-price effects from the credit intervention. [DOI](https://doi.org/10.1093%2FQje%2FQjy034)

**Test:** Expand storage credit from a few households to most households, conserving bank or lender balance sheets. Measure the changing seasonal spread and realized return.

### G. Stable official prices should not imply stable welfare

Under a binding ceiling, official prices may remain unchanged while queues lengthen or sales move elsewhere. Under a variable-weight assize, the loaf price can remain unchanged while its mass declines.

**Test:** Record official price, actual unit-mass price, transaction volume, waiting time, legal compliance and calories purchased separately.

### H. Food availability and food access should diverge in some crises

A simulation must permit hunger alongside food stocks when ownership, purchasing power or institutional access prevents distribution. Slavin’s famine study illustrates why production losses alone are insufficient to characterize a crisis. [University of Stirling](https://www.stir.ac.uk/research/hub/publication/1474582?utm_source=chatgpt.com)

**Test:** Redistribute purchasing power without changing total food. Household consumption should change even when the aggregate stock ledger does not.

---

## 6. Recommended TCE architecture and simplifications

### Represent institutions and inventories explicitly

| Entity | Minimum economically relevant state |
| --- | --- |
| Household | Shared budget, food and seed stocks, obligations, credit, known sellers and consumption priorities |
| Producer or retailer | Owned inventory, offered quantity, production capacity, costs, asks, sales history and replenishment expectations |
| Market venue | Calendar, location, stall capacity, fees, applicable standards and allocation rules |
| Trader | Capital, carrying capacity, itinerary, price observations, contracts and cargo ownership |
| Regulatory institution | Jurisdiction, rules, inspection capacity, sanctions and operating budget |
| Public granary | Actual stocks, storage capacity, purchase/release rules, funding and losses |

Individuals still perform the journeys and work. Household purchasing need not require every family member to maintain an independent food-shopping strategy.

Each inventory batch should carry **owner, quantity, unit, grade, age and location**. Goods in transit remain owned goods; putting them on a caravan must neither duplicate them nor remove them from the world’s conservation ledger.

### Use event-driven economic updates

For 10k–50k people, the recommended sequence is:

1. Production, consumption, spoilage and obligations update on their relevant schedules.
2. Households and firms form bounded purchase and sale intentions.
3. Individuals travel to known feasible venues or sellers.
4. Transactions settle against actual inventory and funds.
5. Sellers review prices at scheduled intervals.
6. Traders update plans when information or shipments arrive.

These are implementation recommendations, not measured performance results. With a bounded seller comparison set, matching can scale approximately with the number of shopping decisions rather than all possible buyer–seller pairs.

Randomize processing order reproducibly so low agent identifiers do not always obtain scarce food first. Under explicit rationing, replace random order with the institution’s allocation rule.

### Keep the following distinctions

**Ask versus realized price.** A posted quote with no sale is not an observed transaction price.

**Need versus funded demand.** Unaffordable hunger must not silently become an order backed by nonexistent money.

**Physical stock versus offered stock.** Storage decisions and obligations matter.

**Nominal bundle versus standardized quantity.** A loaf, basket or sack can change mass or quality.

**Information arrival versus goods arrival.** Knowing that another town is cheap does not make its grain locally available.

### Simplify negotiations, not constraints

A single bargaining calculation is adequate for v1. So are a few standardized quality grades and household-level credit accounts. It is much riskier to simplify away travel, seasonal inventory planning or purchasing-power constraints: those are the mechanisms that generate the requested historical behavior.

Do not let an arbitrary “base price” permanently pull every society toward the same relative prices. Reference prices can initialize expectations, but subsequent prices should be grounded in local production, substitution, trade and budgets.

### Existing models to borrow from

| Reference | What it contributes | What TCE should not copy unchanged |
| --- | --- | --- |
| **Deaton–Laroque competitive storage model** | A rigorous benchmark for stockholding, scarcity and nonlinear commodity prices | Its equilibrium storage decisions are not a behavioral model of every historical household or merchant. [Professor Sir Angus Deaton](https://deaton.scholar.princeton.edu/publications/behaviour-commodity-prices) |
| **EURACE / Eurace@Unibi** | A heterogeneous-agent macroeconomic architecture with households, firms and spatial structure; documentation and source code are available | Its modern economic setting is not a ready-made model of periodic markets, subsistence obligations or historical regulation. [Bielefeld University](https://www.uni-bielefeld.de/fakultaeten/wirtschaftswissenschaften/lehrbereiche/etace/eurace%40unibi/the-eurace%40unibi-model/) |

Use these as **component references and validation benchmarks**, not proof that a particular TCE price controller is already validated.

---

## 7. Sources, datasets and evidence limits

### Practical calibration sources

| Source | Best use | Main caution |
| --- | --- | --- |
| **Global Price and Income History Group, UC Davis** | Historical commodity prices, wages, purchasing-power comparisons and unit conversions across regions | Series differ in coverage, frequency, units and transaction context; inspect each file’s documentation. [GPIH](https://gpih.ucdavis.edu/Datafilelist.htm) |
| **Shiue & Keller, 2007, “Markets in China and Europe on the Eve of the Industrial Revolution”** | Spatial market-integration comparisons; the AEA page supplies a replication package and appendix | Do not turn regional findings into a universal continent-level coefficient. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.97.4.1189) |
| **Donaldson, 2018, “Railroads of the Raj”** | Prices, trade and transport-infrastructure effects; replication package and appendix available | Colonial India is a specific institutional and transport setting. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199) |
| **Gilbert, Christiaensen & Kaminski, 2017, “Food price seasonality in Africa”** | Monthly crop-specific seasonality benchmarks and estimation methods | Short series can bias peak-to-trough estimates upward; wholesale and retail prices differ. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/?utm_source=chatgpt.com) |
| **Temin, 2002, “Price Behavior in Ancient Babylon”; van Leeuwen, Földvári & Pirngruber, 2011** | Ancient commodity-price behavior, storage and crop-calendar comparisons | Gaps, commodity quality and quotation conventions complicate interpretation. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0014498301907740) |
| **Will & Wong, 1991, *Nourish the People*** | Qing public-granary organization and historical stock records | Administrative records require attention to accounting and implementation, not just nominal reserve totals. [University of Michigan Press](https://press.umich.edu/Books/N/Nourish-the-People2) |

For institutional behavior, the most useful accompanying readings are **Skinner on rural Chinese marketing; Hill on West African market authority and periodicity; Geertz on bazaar information and search; Davis on bread assizes; and Edwards and Ogilvie on Champagne fairs**. Their contribution is the structure of trading opportunities and enforcement, not universal numerical coefficients. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952)

### Contested claims and thin evidence

**Private ordering versus public enforcement.** Champagne fairs are often invoked as evidence for self-enforcing merchant institutions. Edwards and Ogilvie challenge that account and emphasize public authorities’ protection and enforcement. TCE should allow both reputation-based cooperation and formal enforcement rather than assume either universally substitutes for the other. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0014498311000660)

**Interest rates inferred from seasonal prices.** The classic McCloskey–Nash interpretation of medieval grain seasonality is not a safe source for a universal storage-interest parameter. Crop calendars, carryover stocks and other assumptions materially affect the inference. [JSTOR](https://www.jstor.org/stable/1803317)

**Early markets and ethnographic analogies.** Observations of recent foragers or farming communities are not direct measurements of prehistoric institutions. Use them to establish possible mechanisms, not a single ancestral economic template. Gurven’s work itself emphasizes competing explanations for food transfers. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/to-give-and-to-give-not-the-behavioral-ecology-of-human-food-transfers/45EBF9C9619A612ECCB77B6F81449F7D)

**Measurement discipline is essential.** For TCE’s calibration pipeline, never pool legal price schedules with actual transaction prices without labeling them. Normalize mass, quality and currency; preserve missing observations; distinguish retail from wholesale; and do not use annual averages to validate within-year harvest patterns. Likewise, price correlation alone is an inadequate integration test: common weather or monetary shocks can move disconnected towns together.

---

## Final design recommendation

For v1, implement **periodic venues and permanent sellers, household budgets and obligations, seasonally planned inventories, bounded search, finite traders, and separate regulatory modules for standards, assizes, ceilings and public reserves**.

The decisive change to an inventory-based price system is this:

> A seller should respond to stock relative to expected sales and the next feasible replenishment—not stock relative to an unchanging number.

Combined with real ownership, purchasing power and transport, that rule gives TCE a foundation for market days, harvest gluts, lean-season scarcity, distressed sellers, prosperous merchants, imperfect price convergence and effective—or ineffective—public intervention without scripting historical eras.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92879-61c4-83ea-8261-3e8c19510617)
