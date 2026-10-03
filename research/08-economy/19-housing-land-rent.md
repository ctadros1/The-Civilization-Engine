# Housing markets, land values, and rents

## A simulation-ready report for The Civilization Engine

**The central design recommendation is to make land value an outcome of competition over locations and permitted uses—not an independent “desirability score” that mechanically sets rent and building height.** Households bid for housing services, businesses bid for productive locations, and builders bid for sites according to what they can profitably construct. Ownership, tenancy rules, financing, and construction delays determine how those bids become actual buildings and rents. Agent-based land-market research demonstrates that decentralized bidding can reproduce basic urban-economic patterns without assigning a predetermined price surface. [JASSS](https://www.jasss.org/12/1/3.html)

Do **not** encode the chain “expensive land → rich residents → tall buildings.” Expensive locations can accommodate poor households occupying very little space. Informal settlements can have extensive rental markets and high quality-adjusted rents. Meanwhile, valuable land can remain underbuilt because redevelopment is costly, constrained, or delayed. These are distinct mechanisms, not exceptions that require scripted events. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305750X08001162)

For TCE, the useful decomposition is:

**Location advantages → competing bids → contracts and allocation → construction and maintenance → changed housing supply and accessibility.**

Finance and expectations add a second feedback loop through property prices and construction investment.

---

## 1. Mechanisms translated into simulation rules

### 1.1 Separate land, buildings, housing services, and financial claims

These quantities need separate fields and accounting:

| Quantity | Appropriate unit | Meaning |
| --- | --- | --- |
| Ground rent | Currency per m² of land per year | Payment for using a site, excluding a separately owned building |
| Housing-service rent | Currency per m² of usable floor area per year | Useful analytical measure of accommodation costs |
| Contract rent | Currency per dwelling, room, or bed-space per payment period | What an occupant actually owes |
| Property sale price | Currency per transferred interest | Capital value of the rights being sold |
| Net operating income | Currency per property per year | Collections minus operating costs, before financing |
| Replacement cost | Currency per building | Cost of constructing an equivalent structure now |

These are proposed accounting conventions. Their separation matters because land-value research distinguishes direct land transactions from estimates obtained by subtracting construction value from property value; the two approaches need not agree. Existing structures also cannot always be valued at replacement cost because demolition, adaptation, and irreversible investment matter. [David Albouy](https://davidalbouy.com/s/landvalue_index.pdf)

**Implementation rule:** a parcel, its structure, the right to collect rent, and the right to occupy may belong to different entities. Never infer all four from one `owner_id`.

For nonmarket allocation, retain physical opportunity costs and competing claims without inventing a cash rent. A household can have valuable occupancy rights even when selling or leasing them is forbidden.

### 1.2 Agricultural and urban uses compete through location-dependent surplus

Von Thünen’s central mechanism is transport-cost-adjusted agricultural surplus. An implementable version is:

\[
\rho^{a}\_{jc}
=
Y\_{jc}\left(p\_c-t\_{jc}\right)-C\_{jc}
\]

where:

* \(Y\_{jc}\) is annual output of crop \(c\) per unit of land at parcel \(j\);
* \(p\_c\) is its market price;
* \(t\_{jc}\) is transport and spoilage cost per unit delivered;
* \(C\_{jc}\) is nonland production cost, including labor’s opportunity cost.

The parcel’s agricultural opportunity value comes from the best **permitted and feasible** use, not necessarily grain. Gardening, orchards, grazing, woodland, and water access can compete differently. Classical urban land-market models extend this location-surplus logic to urban uses. [JASSS](https://www.jasss.org/12/1/3.html)

**TCE rule:** convert agricultural land only when the expected advantage of the new use exceeds agricultural opportunity cost, clearing, access provision, construction, and any required compensation. Political seizure can override consent, but should still impose physical costs and political consequences.

Concentric rings should emerge only in deliberately simplified test worlds. Rivers, slopes, soil, gates, bridges, and multiple markets should distort them.

### 1.3 Households trade housing space against access and other consumption

Use the household—not the individual citizen—as the normal housing-budget unit, while retaining each member’s workplace and travel needs.

A basic cash constraint is:

\[
c\_i+r\_j h\_i+T^{cash}\_{ij}\leq Y\_i
\]

Here \(h\_i\) is usable housing space, \(r\_j\) its unit rent, \(c\_i\) other expenditure, and \(T^{cash}\_{ij}\) monetary travel expenditure. Travel time should enter utility or opportunity cost separately; it is not automatically a cash payment.

Households should value combinations of space per person, privacy, construction quality, water and sanitation, employment access, social connections, security, and tenure stability. For a fixed housing bundle, the simplified compensating bid-rent gradient is:

\[
\frac{\partial b\_i}{\partial d}
=
-\frac{T'\_i(d)}{h\_i}
\]

where \(T\_i\) is **money-equivalent** travel cost. This expresses the Alonso mechanism: savings in travel costs can support a higher rent near destinations. Decentralized implementations can recover this relationship without perfect household knowledge. [JASSS](https://www.jasss.org/12/1/3.html)

**Important implications for TCE:**

* A household accepting less space can bid more per square metre without paying more total rent.
* Better transport changes the value of locations, not merely commuting animations.
* Different occupations can generate different residential patterns.
* Workshops and shops inside homes make access to customers important alongside access to an employer.

Use **network travel time**, not a fixed distance-to-city-center multiplier. Ahlfeldt and Wendland’s reconstruction of Berlin, 1890–1936, found that incorporating the railway network materially improved explanations of land values and attributed almost three-quarters of modeled decentralization to its development. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0166046210000736?utm_source=chatgpt.com)

Businesses should participate too: their maximum location bid is the additional profit obtainable there after paying nonland costs. A marketplace, mill, warehouse, or administrative institution can therefore displace housing without any residential “level-up” rule.

### 1.4 Rents arise through search, bargaining, and contracts

**Proposed market process:**

An owner lists a rentable unit with an asking rent. Searching households inspect a limited set of feasible alternatives and submit offers or accept terms. Owners consider payment reliability, expected vacancy, legal restrictions, and any nonfinancial preferences. The resulting contract fixes payment obligations until its specified review or termination conditions occur.

Use separate variables for:

* Current asking rent.
* Existing contract rent.
* Expected market rent.
* Actual collections after arrears and concessions.

Adjust asking rents after repeated competing offers or prolonged vacancy. Do not reprice every occupied dwelling every simulation day.

Historically, payment and repair obligations varied within explicit contracts. A Babylonian lease from 538 BCE specified a five-year term, divided annual payment into two installments, and assigned roof and foundation work to the tenant. It is evidence for a particular contractual arrangement, not a universal ancient lease template. [Digital Pasts Lab](https://digitalpasts.github.io/nabucco/items/30022)

**Do not impose a universal 30% housing-expenditure ceiling.** In the model, households should instead respond to pressure by reducing space, sharing, subletting, postponing independent household formation, accepting worse conditions, moving farther away, migrating, or becoming inadequately housed. Track deprivation when these adjustments fail rather than making the housing solver silently manufacture affordability.

### 1.5 Tenure changes who can bid, collect, build, and evict

Represent tenure as a bundle of permissions and obligations, rather than a single categorical switch.

| Suggested rights module | Required rules | Likely model consequence |
| --- | --- | --- |
| Customary or communal occupancy | Membership, allocation authority, abandonment, inheritance, consent to transfer | Valuable occupancy without an unrestricted land-sale market |
| Owner occupation | Household ownership, inheritance, transfer and borrowing permissions | Housing consumption and property wealth are combined |
| Ordinary rental and subletting | Payment, term, review, repair, termination, subletting | Multiple layers of landlord and tenant claims |
| Ground lease | Separate site and structure rights; expiry and reversion | Builders may own improvements without owning land |
| Temple, church, endowment, or public ownership | Institutional objectives, restrictions on alienation, distribution of revenue | Decisions need not maximize immediate private rent |
| Employer or service-linked housing | Eligibility tied to work or duties | Job loss can also cause housing loss |
| Informal occupation and rental | Competing claims, local enforcement, eviction risk, recognition | Rental markets can exist without fully recognized legal title |
| Cooperative or social housing | Membership, allocation, cost-sharing, resale and rent rules | Queues and eligibility can partly replace highest-bid allocation |

Historical institutions could adapt rather than simply obstruct markets. Ottoman *icâreteyn* arrangements combined an upfront payment with recurring rent, helping endowments finance restoration and manage properties. Research also challenges the blanket claim that waqf administration was inherently unable to adjust rental arrangements. [Açık Erişim FSM](https://acikerisim.fsm.edu.tr/items/c317754c-1bcd-4dcb-87db-61ad55bfa61d)

**Implementation rule:** evaluate legal permission and effective enforcement separately. An unenforced ownership document and a locally respected customary claim should not provide identical security.

Nor should a title certificate automatically unlock credit. A quasi-experiment involving land titling near Buenos Aires found improvements in housing investment and other household outcomes without the expected expansion of credit access. [CEDLAS](https://www.cedlas.econo.unlp.edu.ar/wp/en/doc-cedlas103-pdf/)

### 1.6 Capitalize expected income—but distinguish prices from rents

For a transferable income-producing interest:

\[
P=\sum\_{t=1}^{H}\frac{E[NOI\_t]}{(1+d)^t}
+\frac{E[TV\_H]}{(1+d)^H}
\]

\(NOI\_t\) is income net of operating expenses; \(d\) is the required return; and \(TV\_H\) is the value of whatever rights remain at the horizon. A leaseholder cannot claim the freeholder’s eventual reversion.

For constant growth and a perpetual interest:

\[
P=\frac{NOI\_1}{d-g}
\qquad \text{only when }d>g
\]

Use this as a simplified valuation rule, not an unlimited money generator. Finite horizons and explicit terminal assumptions are safer when agents extrapolate growth.

**Constructed example:** an asset producing 100 currency units annually is worth 2,000 at a 5% capitalization rate and 2,500 at 4%, despite unchanged current income. Thus rising property prices do not necessarily imply rising current rents.

For TCE:

* Keep nominal and inflation-adjusted calculations consistent.
* Keep financing separate from unlevered operating income.
* Do not count expected appreciation both in annual income and again in terminal value.
* Do not add an independently computed “redevelopment premium” to a valuation that already includes redevelopment.

Historical property-level research finds materially different returns from some aggregate reconstructions, reinforcing the need to distinguish observed total returns, gross rental yields, costs, and appreciation. [Maastricht University](https://cris.maastrichtuniversity.nl/en/publications/the-total-return-and-risk-to-residential-real-estate/)

### 1.7 Building height emerges from marginal profitability and feasibility

For parcel \(j\) and authored building template \(k\), calculate a residual site bid:

\[
B\_{jk}
=
PV(\text{expected operating income and terminal value})
-
PV(\text{nonland development costs})
\]

Development costs include construction, access and utilities, demolition, displacement obligations, and other required work. The maximum feasible residual bid determines what a builder can offer for the site.

For an occupied property, compare redevelopment against **retaining the existing building and its income**, not against zero.

Height should emerge when additional usable floor area produces enough value to cover the additional structural, circulation, servicing, and risk costs. The feasible choice set comes from TCE’s authored construction technologies. Geographic constraints and regulation also affect housing supply; Saiz’s research provides a major empirical foundation for distinguishing physically and institutionally constrained locations. [OUP Academic](https://academic.oup.com/qje/article-abstract/125/3/1253/1903664)

**Track separately:**

\[
FAR=\frac{\text{gross floor area}}{\text{parcel area}}
\]

Building coverage, number of storeys, usable-to-gross floor ratio, and occupants per usable square metre are different variables. Dense settlement can result from narrow streets, high coverage, subdivision, or crowding—not only height.

At 97 Orchard Street in New York, an 1863 five-storey tenement contained 22 apartments of roughly 325 square feet each. This is a useful building-level benchmark, but not a representative average for every industrial city. [Tenement Museum](https://www.tenement.org/explore/97-orchard-street/)

Construction should consume actual labor, materials, and time. Seasonal labor shortages, unavailable timber, inadequate credit, or a lack of builders must delay completion even when rents are high.

### 1.8 Overcrowding and maintenance are economic responses

Proposed landlord accounting:

\[
NOI =
\text{actual rent collections}
-\text{maintenance}
-\text{owner-paid services}
-\text{taxes and administration}
\]

Debt service belongs in a subsequent financing calculation.

Owners can respond to demand by subdividing units, accepting lodgers, converting work space, adding courtyard structures, repairing, extending, or rebuilding. These choices should have different effects on usable space, privacy, sanitation, fire exposure, and future upkeep.

A low-quality rental equilibrium can persist when improvements cannot earn adequate returns, tenure is insecure, tenants lack alternatives, or owners possess local bargaining power. Nairobi research documents extensive rental occupation alongside poor services and crowding; another study estimates higher quality-adjusted rents in slums than outside them. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305750X08001162)

**TCE rule:** maintenance should depend on expected future benefit and responsibility under the contract. Tenant-funded improvement becomes less attractive when eviction or uncompensated loss of improvements is likely.

Avoid making clearance an automatic welfare improvement. Research on Bombay’s Improvement Trust shows how sanitary intervention and inadequate affordable rehousing could aggravate the housing problem. [EBHSOC](https://www.ebhsoc.org/journal/index.php/ebhs/article/view/221)

### 1.9 Speculation and building cycles require expectations, financing, and delay

A useful endogenous cycle is:

**Stronger demand → higher rents and prices → optimistic forecasts and easier collateral-backed borrowing → more projects → delayed completions → higher vacancy and weaker income → falling valuations, defaults, and curtailed construction.**

Not every link must exist in every society. Expectations can generate overbuilding without modern mortgage banks; strong credit feedback can amplify it.

Give builders and investors heterogeneous forecasts: some emphasize current rents and construction costs, others extrapolate recent changes, and some wait for further information. Land withholding can reflect speculative expectations, but also the option value of waiting or unresolved development constraints.

Glaeser, Gyourko, and Saiz find that housing-supply elasticity changes bubble dynamics: more elastic places tend to experience fewer and shorter price bubbles, although overbuilding remains possible. **Do not hardcode a universal cycle duration.** [National Bureau of Economic Research](https://www.nber.org/papers/w14193)

---

## 2. Quantitative parameters and calibration anchors

### 2.1 Observed benchmarks

**Confidence refers to the stated observation or estimate, not its portability to another society.** “High” means a well-defined documented measurement; “medium” indicates reconstruction, model dependence, or limited representativeness. These are not statistical confidence intervals.

| Observable | Quantitative evidence | Interpretation and confidence |
| --- | --- | --- |
| **Urban land-value gradient** | US estimates for 2005–2010: central-to-outer land-value ratios of **22.3 in New York, 35.1 Chicago, 32.6 Washington, 9.3 San Francisco, 5.5 Los Angeles, and 1.3 Orange County** | Center means **0.5 mile from downtown**, outer means **10 miles**. Model-based estimates from 68,756 land sales across 324 metropolitan areas. **Medium**; not housing-rent ratios or universal gradient coefficients. [David Albouy](https://davidalbouy.com/s/landvalue_index.pdf) |
| **Transport-induced decentralization** | Berlin, 1890–1936: railway development accounts for **almost 75%** of modeled decentralization | A historical model attribution, not a universal transport elasticity. **Medium.** [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0166046210000736?utm_source=chatgpt.com) |
| **Preindustrial central rental premium** | Toledo, 1489–1810 records: median central rents were approximately **50% higher** than elsewhere | Property-rent comparison, not pure land rent or a fully standardized dwelling comparison. **Medium.** [Cambridge University Press](https://www.cambridge.org/core/journals/revista-de-historia-economica-journal-of-iberian-and-latin-american-economic-history/article/cost-of-housing-in-the-very-long-run-toledo-14891810/BFE34A84E12FDEC73671AAF9A3016B5E) |
| **Industrial working-class rents** | British 1905 survey: predominant weekly rents for **three rooms** were **6–9 shillings in London**, versus **3s 9d–4s 6d** in English and Welsh provincial towns | Contemporary reproduction of Board of Trade findings; working-class rents generally included local rates. **Medium** cross-place comparability. [Wikisource](https://en.wikisource.org/wiki/1911_Encyclop%C3%A6dia_Britannica/Housing) |
| **Industrial dwelling size** | 97 Orchard Street: apartments approximately **325 ft² = 30.2 m²**, generally three rooms | Individual documented building, not a citywide average. **High for the building; low transferability.** [Tenement Museum](https://www.tenement.org/explore/97-orchard-street/) |
| **Informal rental occupation and crowding** | Nairobi slum survey, 2004: **92% renting**, approximately **2.6 persons per room**, sample **1,755 households** | Rental tenure does not imply formal title or adequate services. **Medium–high for this survey.** [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305750X08001162) |
| **Quality-adjusted informal rent premium** | A Nairobi study estimates slum rents approximately **16% higher**, controlling for observed housing characteristics | Conditional estimate; not proof that informality itself causes a 16% markup everywhere. **Medium.** [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0305750X18301219) |
| **Long-run real housing investment returns** | Paris, 1809–1943: **4.0% annually**; Amsterdam, 1900–1979: **4.8% annually** | Annualized **real total returns net of costs and taxes**, not gross yields or required discount rates. Property-level archival reconstruction. **Medium.** [Maastricht University](https://cris.maastrichtuniversity.nl/en/publications/the-total-return-and-risk-to-residential-real-estate/) |
| **Severe historical property-price decline** | Edirne, 1720–1814: approximately **75% decline in real house prices** over the study period | Based on 2,246 deeds and reconstructed prices; demographic and political disruptions were important. **Medium.** [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ehr.13001) |
| **Modern land contribution to appreciation** | In 14 advanced economies, land accounts for roughly **80% of the post-WWII rise in house prices** in the authors’ decomposition | This is a share of the **increase**, not an assertion that every house is 80% land. **Medium**, sensitive to valuation assumptions. [Kiel Institute](https://www.ifw-kiel.de/publications/no-price-like-home-global-house-prices-1870-2012-20483/) |
| **Boom and bust magnitude** | Tokyo residential land index, 1985=100: **233.7 in 1990**, **167.3 in 1993** | Nominal appreciation of **133.7%**, then a **28.4% decline**, calculated from the reported index. **High for the reported series**, not a universal cycle target. [IMF eLibrary](https://www.elibrary.imf.org/view/book/9781557754622/ch06.xml) |
| **Ancient lease duration and payment timing** | Babylon, 538 BCE: **five-year lease**, annual payment divided into **two installments** | A specific surviving contract with tenant repair obligations. **High for this contract; very low as a population-wide default.** [Digital Pasts Lab](https://digitalpasts.github.io/nabucco/items/30022) |

### 2.2 Proposed initialization and sensitivity ranges

The following are **engineering priors, not historical estimates**. Their source is the proposed TCE design. Treat their cross-era empirical confidence as **low** until calibrated against a chosen setting.

| Model setting | Initial test range | Interpretation |
| --- | --- | --- |
| Housing preference out of income above basic needs | **0.15–0.35** dimensionless weight | A preference parameter, not a fixed rent-to-income ceiling |
| Value of discretionary travel time | **0.25–1.0 × relevant hourly earning opportunity** | Sensitivity range; use household-specific opportunities rather than assuming every minute can be sold for wages |
| Housing alternatives inspected | **10–30 candidates per search episode** | Computational setting; vary to test search frictions |
| Asking-rent review interval | **1–4 weeks** | Applies to listings; occupied contracts follow their own rules |
| Real required asset return | **3–12% per year**, with higher-risk stress cases | Scenario assumption; not interchangeable with observed historical total returns |
| Forecast smoothing window | **24–60 months** | Controls how rapidly expectations respond |
| Weight on extrapolated price growth | **0–0.7** | Experimental heterogeneity; zero supplies a fundamentals-oriented control |
| Maximum loan-to-value ratio | **0 where collateral lending is absent; 50–90% in credit-market experiments** | Institutional policy variable, not an early-agrarian default |
| Construction duration | **Resource-derived**, rather than one universal range | Compute from work requirements, available crews, seasonality, and deliveries |
| Maintenance requirement | **Material- and component-specific** | Prefer repair labor and replacement materials over a universal percentage of property price |

**Do not calibrate equilibrium outputs as constants.** Vacancy, rent burdens, population density, central premiums, turnover, and observed capitalization rates should normally emerge from the model.

For historical comparisons, report both:

\[
\text{Housing burden}
=
\frac{\text{annual household housing expenditure}}
{\text{annual household income}}
\]

and

\[
\text{Rent in labor-days}
=
\frac{\text{annual contract rent}}
{\text{comparable daily wage}}
\]

Specify household versus individual income, gross versus disposable income, payment in kind, utilities, local taxes, and tenant repair obligations. A rent measured in one worker’s wages is not automatically the household’s budget share.

---

## 3. Variation across eras and world regions

These should be **composable institutional configurations**, not a mandatory progression through technological eras.

| Setting | Evidence and historical variation | TCE representation |
| --- | --- | --- |
| **Foragers** | Woodburn distinguishes immediate-return societies from systems with more durable productive assets and delayed returns. Foragers should not all be assigned either unrestricted private property or propertylessness. [The Ted K Archive](https://www.thetedkarchive.com/library/james-woodburn-egalitarian-societies) | For mobile groups, prioritize camp access, sharing, kin affiliation, and seasonal claims. Introduce more exclusive rights where fixed assets and local institutions support them; do not begin with universal cash rents. |
| **Early farming** | Archaeological house-size evidence shows substantial variation in material inequality across settlements and regions. House area is a proxy, however, not a direct observation of rent or legal ownership. [Nature](https://www.nature.com/articles/nature24646) | Allocate plots through households, kin groups, communities, or authorities as appropriate. Let clearance, inherited occupation, storage, and investment create differentiated claims without presupposing a land market. |
| **Ancient Near East** | Babylonian documents record explicit rental contracts, payment schedules, repair duties, and institutional property. Urban leasing did not require modern banking institutions. [Digital Pasts Lab](https://digitalpasts.github.io/nabucco/items/30022) | Support written or witnessed contracts, noncoin units of account, tenant improvements, and temple as well as household ownership. |
| **Preindustrial Europe** | Toledo’s records reveal institutional landlordism and master tenants who sublet property. A recorded “house” could contain several households, so property records cannot be mapped directly to dwelling counts. [Cambridge University Press](https://www.cambridge.org/core/journals/revista-de-historia-economica-journal-of-iberian-and-latin-american-economic-history/article/cost-of-housing-in-the-very-long-run-toledo-14891810/BFE34A84E12FDEC73671AAF9A3016B5E) | Separate whole-property leases from rooms and subordinate tenancies. Preserve contract history and allow buildings to combine residence, retail, and production. |
| **Ottoman cities** | Edirne transactions show the importance of size, commercial access, freshwater, and family ties. Waqf arrangements added another layer of ownership and leasing, rather than eliminating property markets. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ehr.13001) | Combine ordinary sales and rental markets with institutional restrictions, advance payments, repair-financing contracts, and socially differentiated access. |
| **China and Japan** | Huizhou transactions, 1570–1949, show associations between housing prices and orientation, ancestral halls, and religious sites. Edo’s commoner housing included rented *nagaya* row houses with very small main rooms. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70096) | Allow culturally specific location preferences. Dense rental districts can consist of small low-rise units rather than a European tenement template. |
| **Industrial Britain and North America** | Housing conditions varied sharply even within industrializing regions. The 1901 figures reproduced in a contemporary British survey put **50.6% of Scotland’s population**, versus **8.2% of England’s**, in one- or two-room dwellings; definitions and local dwelling forms complicate comparison. [Wikisource](https://en.wikisource.org/wiki/1911_Encyclop%C3%A6dia_Britannica/Housing) | Permit room rental, lodgers, subdivision, and household sharing. Do not impose one “industrial housing standard” on every region. |
| **Colonial South Asia** | Bombay’s plague-era housing response involved public authorities, millowners, labor markets, and conflicts over who would finance housing. Clearance without adequate affordable replacement could worsen scarcity. [EBHSOC](https://www.ebhsoc.org/journal/index.php/ebhs/article/view/221) | Model employer incentives, municipal finance, rehousing obligations, and displaced households explicitly. Public intervention may help or harm depending on implementation. |
| **Modern African informal settlements** | Nairobi demonstrates that informality can coexist with overwhelmingly rental occupation, substantial crowding, and poor services. “Slum” is not synonymous with owner-built, owner-occupied shelter. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305750X08001162) | Separate land claimants, structure owners, tenants, and service providers. Include locally effective enforcement and barriers to upgrading. |
| **Modern Latin American informal urbanization** | Buenos Aires titling evidence supports an investment-security mechanism but not an automatic title-to-mortgage-credit mechanism. [CEDLAS](https://www.cedlas.econo.unlp.edu.ar/wp/en/doc-cedlas103-pdf/) | Security can improve investment independently of bank development. Incremental owner construction and rental arrangements need not disappear when rights become more formal. |
| **Modern mixed housing systems** | Social rental housing exceeds **20% of dwelling stock** in the Netherlands, Austria, and Denmark, while remaining below **5%** in roughly two-thirds of OECD countries covered by the cited report. [OECD](https://read.oecd-ilibrary.org/en/publications/tackling-the-affordability-gap-through-increased-supply-of-affordable-and-social-housing_60b5a398-en/full-report.html) | Permit large nonprofit, cooperative, or public sectors alongside market housing. Administrative rents require allocation rules, funding, and maintenance decisions—not just a price discount. |

The comparative lesson is that **technology determines what can be built; institutions determine who may occupy, transfer, finance, and profit from it.** TCE should allow those dimensions to vary independently.

---

## 4. Stylized facts and validation tests

Use conditional tests rather than demanding that every generated world reproduce the same historical outcome.

| Test | Pattern a credible simulation should produce |
| --- | --- |
| **Isolated accessibility experiment** | With homogeneous land, one destination, and otherwise similar households, bids should generally decline with travel cost. Add another employment center and the value surface should become less purely monocentric. This is an appropriate controlled test of the bid-rent mechanism. [JASSS](https://www.jasss.org/12/1/3.html) |
| **Transport improvement** | A new bridge or railway should change the value of affected locations according to actual travel savings. It should not raise every parcel equally or preserve an unchanged radial gradient. Berlin provides a historical comparison. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0166046210000736?utm_source=chatgpt.com) |
| **Cross-city heterogeneity** | Central-to-outer land-value ratios should vary widely with urban structure. The US examples range from almost flat to above 30:1; one universal decay coefficient is inappropriate. [David Albouy](https://davidalbouy.com/s/landvalue_index.pdf) |
| **Expensive location, poor residents** | Low-income households should sometimes remain in accessible districts by consuming less space. High rent per square metre should not automatically imply wealthy occupants or good housing quality. Nairobi offers a demanding comparison. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305750X08001162) |
| **Demand shock with durable supply** | With slow construction, rising demand should initially affect rents, sharing, and occupancy more than completed floor area. More elastic supply should subsequently produce more construction and less persistent price pressure. [National Bureau of Economic Research](https://www.nber.org/papers/w14193) |
| **Price–rent divergence** | Changing required returns or expected growth should move sale prices even when current rent changes little. Test this directly against the capitalization arithmetic above; do not require a rent increase whenever an asset appreciates. |
| **Population and disruption shocks** | Falling housing demand can produce large real price declines, as in Edirne. But a war that also destroys housing need not lower surviving-unit rents: demand, supply, and investment risk must be shocked separately. The historical result is a benchmark, not a universal disaster modifier. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ehr.13001) |
| **Institutional constraint** | Secure occupancy should make some improvements more attractive, while formal title alone should not produce loans without lenders, repayment capacity, and collateral enforcement. [CEDLAS](https://www.cedlas.econo.unlp.edu.ar/wp/en/doc-cedlas103-pdf/) |
| **Boom and bust** | Credit and extrapolative expectations should sometimes create a construction pipeline that outlasts demand. Removing those feedbacks should alter the cycle; the model should not produce a predetermined crash on a calendar. [National Bureau of Economic Research](https://www.nber.org/papers/w14193) |

For the sanity dashboard, record distributions rather than only means:

**Rent burden by income and tenure; usable space per person; persons per room; vacancy by unit type; asking versus contract rents; arrears and displacement; building coverage and FAR; construction starts versus completions; maintenance backlog; land and structure ownership concentration.**

Those are proposed diagnostics. In particular, a healthy average floor area can conceal severe crowding in a minority of households.

---

## 5. Recommended TCE implementation

### 5.1 Minimal entity model

| Entity | Essential state |
| --- | --- |
| **Person** | Household membership, occupation, income contributions, mobility and care needs |
| **Household** | Budget, members, housing preferences, current rights, search status, arrears |
| **Parcel** | Geometry, access, alternative uses, physical constraints, rights and restrictions |
| **Building** | Template, gross and usable area, condition, components, construction state |
| **Rentable unit** | Rooms or bed-spaces, shared facilities, occupants, listing status |
| **Contract or claim** | Parties, payment, review dates, repair duties, transfer, termination, enforcement |
| **Owner or institution** | Portfolio, funds, objectives, liabilities, decision rules |
| **Development project** | Chosen template, resource requirements, financing, remaining work, expected income |

Do not create a fully deliberative agent for every ownership interest. A temple, estate, cooperative, or landlord portfolio can use a compact institutional decision policy.

### 5.2 Update cadence and computational simplifications

**Proposed implementation:**

Run travel and lived consequences daily, but schedule housing decisions as events. Search begins after household formation, employment changes, eviction, dissatisfaction, or a major change in circumstances. Lease payments and reviews occur on contract dates. Builder evaluation can be seasonal or quarterly; construction progress remains tied to actual work.

Cache accessibility from housing locations to relevant employment and service nodes. Sample candidate units from location, price, size, and eligibility indexes. Evaluating \(k\) candidates for an active searcher is much cheaper than comparing every household with every dwelling.

Maintain local comparable-contract statistics for price expectations. Sparse or stale evidence should increase uncertainty, not force an exact citywide valuation. Precompute discount factors and use a small template set rather than solving continuous building optimization for every parcel every tick.

These are engineering proposals, **not demonstrated performance benchmarks**. Benchmark separately the cost of search, accessibility refresh, development evaluation, and ledger operations.

### 5.3 Build in layers

A practical sequence is:

1. **Occupancy and physical housing:** households, usable space, construction, upkeep, and agricultural alternatives.
2. **Rights and rental allocation:** ownership bundles, room subdivision, listings, leases, subletting, and eviction.
3. **Investment:** developer template choice, residual site bids, delayed completion, institutional owners.
4. **Finance and policy:** borrowing, collateral, speculative expectations, public housing, restrictions, and insolvency.

Keep every payment and transfer in the economy’s ledger. Rent is a transfer between agents; construction consumes resources; a loan creates corresponding financial claims; a revaluation is not spendable cash unless an actual sale or financing transaction occurs.

### 5.4 Existing models and games worth borrowing from

| Model or game | Useful feature | Limitation for TCE |
| --- | --- | --- |
| **Filatova, Parker, and van der Veen’s agent-based urban land market** | Decentralized bids and asks; heterogeneous market power; recovery of urban-economic patterns | A methodological foundation, not a complete historical tenancy or construction system. [JASSS](https://www.jasss.org/12/1/3.html) |
| **UrbanSim** | Integration of land use, transport, and urban economic change | Borrow the separation of location choices and spatial development, rather than treating its planning-oriented framework as a drop-in real-time kernel. [Urban Data Science Toolkit](https://udst.squarespace.com/overview) |
| **Bank of England/INET housing-market ABM** | Distinct renters, owner-occupiers, investors, banks, and policy feedback; available implementation | Strong reference for modern finance and cycles, but its institutional assumptions must not be applied automatically to agrarian societies. [Bank of England](https://www.bankofengland.co.uk/working-paper/2016/macroprudential-policy-in-an-agent-based-model-of-the-uk-housing-market) |
| **Cities: Skylines II, documented Economy 2.0 design** | Clearly documented, tractable game abstractions connecting rent, land value, building characteristics, and upkeep | Its documented formula and removal of the virtual landlord are useful contrasts. TCE needs actual owners, contracts, and bids if landlordism and distribution are meant to emerge. This describes the documented design, not every later patch. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/dev-diary-economy-part-two) |

---

## 6. Sources, datasets, and limits of the evidence

### Priority research and calibration sources

| Source | Best use | Access or interpretation caution |
| --- | --- | --- |
| **Albouy, Ehrlich, and Shin (2018), “Metropolitan Land Values,” Review of Economics and Statistics** | Urban land-value gradients and cross-city heterogeneity | Underlying CoStar transactions are not generally public; published estimates are usable calibration targets. [David Albouy](https://davidalbouy.com/s/landvalue_index.pdf) |
| **Drelichman, González Agudo, and Ramírez Vásquez (2026), “The cost of housing in the very long run: Toledo, 1489–1810”** | Long-run rental records, property heterogeneity, household-space reconstruction | More than 34,000 observations do not mean 34,000 independent modern-style apartment transactions. [Cambridge University Press](https://www.cambridge.org/core/journals/revista-de-historia-economica-journal-of-iberian-and-latin-american-economic-history/article/cost-of-housing-in-the-very-long-run-toledo-14891810/BFE34A84E12FDEC73671AAF9A3016B5E) |
| **Karagedikli and Tunçer (2021), “House prices in the Ottoman Empire”** | Premodern property transactions, location attributes, and demographic shocks | Hedonic reconstruction and historical deflators introduce uncertainty. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ehr.13001) |
| **Eichholtz, Korevaar, Lindenthal, and Tallec (2021), “The Total Return and Risk to Residential Real Estate,” Review of Financial Studies** | Separating rents, costs, appreciation, and total returns | Selected historical properties and periods should not become universal return assumptions. [Maastricht University](https://cris.maastrichtuniversity.nl/en/publications/the-total-return-and-risk-to-residential-real-estate/) |
| **Saiz (2010), “The Geographic Determinants of Housing Supply,” plus MIT data** | Physical and regulatory constraints on supply | Downloadable data are available; estimated elasticities remain setting- and horizon-specific. [OUP Academic](https://academic.oup.com/qje/article-abstract/125/3/1253/1903664) |
| **Jordà–Schularick–Taylor Macrohistory Database** | Long-run advanced-economy house prices and related macroeconomic series | Check the release, variable definitions, country coverage, and licensing; it is not a global historical rental microdataset. [MacroFinance & MacroHistory Lab](https://www.macrohistory.net/database/) |
| **BIS residential property-price statistics** | Modern price-cycle comparisons across economies | Series differ in coverage and quality adjustment; residential property prices are not pure land prices or rental prices. [BIS Data Portal](https://data.bis.org/topics/RPP/data) |
| **NaBuCCo and ORACC contract editions** | Authored ancient contract rules: payment, repairs, parties, and duration | Surviving documents illustrate possibilities but do not establish representative frequencies. [Digital Pasts Lab](https://digitalpasts.github.io/nabucco/items/30022) |

### Claims to treat cautiously

**There is no defensible universal historical rent burden.** Some historical real-wage comparisons use a fixed housing allowance as a methodological convention. That allowance is not an observed housing-market law. Toledo research shows why actual rents and changing housing bundles matter. [Cambridge University Press](https://www.cambridge.org/core/journals/revista-de-historia-economica-journal-of-iberian-and-latin-american-economic-history/article/cost-of-housing-in-the-very-long-run-toledo-14891810/BFE34A84E12FDEC73671AAF9A3016B5E)

**Archaeological floor area is not a rental dataset.** It can help calibrate building sizes and material inequality, but cannot by itself reveal occupants, rental payments, ownership rights, or household income. [Nature](https://www.nature.com/articles/nature24646)

**“Land value” is measurement-dependent.** Direct vacant-land transactions, property-value-minus-structure estimates, tax assessments, and developer residual bids measure related but nonidentical things. Comparing them without accounting for redevelopment and adjustment costs can produce misleading conclusions. [David Albouy](https://davidalbouy.com/s/landvalue_index.pdf)

**Institutional effects are conditional.** Endowments were not uniformly inflexible, formal title does not automatically create mortgage access, and informal housing is not uniformly owner-occupied. Each of those shortcuts would remove mechanisms that the historical evidence shows to be important. [Open METU](https://open.metu.edu.tr/handle/11511/53881)

**Global historical coverage remains uneven.** The long-run price series discussed here disproportionately represent particular cities, surviving institutions, and advanced economies. Missing observations should be represented as uncertainty—not filled by silently exporting nineteenth-century European parameters elsewhere. The geographical coverage of the major long-run house-price study is explicitly limited to 14 advanced economies. [Kiel Institute](https://www.ifw-kiel.de/publications/no-price-like-home-global-house-prices-1870-2012-20483/)

### Bottom line for TCE

Implement **location-sensitive bids, separate property rights, room-level occupancy, and delayed physical construction before adding speculative price dynamics**.

That foundation allows expensive but crowded districts, institutional landlordism, secure nonmarket housing, redevelopment pressure, and persistent underbuilding to arise from the same machinery. The decisive relationship should be:

> **Agents compete for the advantages of a location, subject to their resources and rights; buildings change only when someone can organize and justify the work.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928a7-3788-83e9-8dcb-96e26175dd4f)
