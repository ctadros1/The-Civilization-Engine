# Origins and history of money: a simulation-ready report for TCE

## Executive recommendation

**Model money as a set of payment, accounting, and enforcement institutions—not as a mandatory sequence of materials.** Barter, reciprocal obligations, commodity payments, coins, and credit can coexist; monetary systems can also fragment or revert to less convenient arrangements. The evidence does not establish a universal stage of self-contained barter economies from which money subsequently emerged. That does not mean barter was absent, or that exchange could not contribute to monetization. [Academia](https://www.academia.edu/3621994/Barter_and_Economic_Disintegration)

For TCE, the central question should therefore be:

> **Why would this particular person accept this particular payment, from this counterparty, for this transaction?**

The answer can be consumption value, an expected opportunity to spend it, an obligation payable in it, confidence in redemption, or an enforceable claim on another person. These motives should produce monetary adoption—and abandonment—through agent behavior.

Keep four functions separate:

| Function | What TCE should represent |
| --- | --- |
| **Unit of account** | The unit in which prices, debts, wages, and taxes are stated. |
| **Medium of exchange** | What people accept to facilitate subsequent purchases. |
| **Means of settlement** | What discharges a particular obligation under its terms. |
| **Store of value** | What people hold to transfer purchasing power through time. |

Historically, these functions did not always reside in the same object. A debt could be denominated in an accounting unit and settled with differently valued coins or commodities; an accounting unit need not correspond to an actual coin. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/2024/wp2024-10.pdf)

**Evidence notation below:** **H** = relatively strong evidence for the stated historical case; **M** = credible but reconstructed, heterogeneous, or interpretation-sensitive; **P** = a proposed simulation parameter, not an empirical historical estimate. These are confidence judgments, not statistical confidence intervals.

---

## 1. Mechanisms: implementable rules

### 1.1 Allow several routes into monetary exchange

The major explanations identify different mechanisms. They should be competing or complementary processes in TCE, not mutually exclusive world settings.

| Explanation | Mechanism worth implementing | Evidential qualification |
| --- | --- | --- |
| **Commodity and exchange theories** | People accept a good they do not immediately need because they expect others to accept it. Repeated acceptance increases its usefulness. | A coherent mechanism, demonstrated in monetary search models; not proof that every historical society began with generalized barter. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/261634) |
| **Credit and accounting theories** | Repeated dealings generate obligations; standard units make debts comparable and permit deferred settlement. | Ancient Near Eastern debt records predate coinage by many centuries. This establishes priority over coins, not a universal origin of all money. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/325858) |
| **Gift and reciprocal-obligation approaches** | Transfers maintain relationships, distribute risk, establish status, or create expectations of future assistance. | Do not equate every obligation with an exact, transferable commercial debt. Anthropological exchange relationships are more varied than that. [academia.edu](https://www.academia.edu/3621994/Barter_and_Economic_Disintegration?utm_source=chatgpt.com) |
| **State and institutional theories** | Authorities standardize accounts and specify acceptable payments for dues, fines, and other obligations. Their spending distributes those media. | Important in documented states, but money-like media also operated across communities without a common taxing authority. [UCI Bren School of ICS](https://ics.uci.edu/~djpatter/classes/2015_06_FutureMoney/assets/docs/hudson.pdf) |

Fauvelle’s recent archaeological argument emphasizes **external exchange across social boundaries**: a medium useful between strangers can spread even where internal distribution remains reciprocal. California shell-money evidence is particularly important because it challenges any requirement that agriculture or a centralized state must precede monetary exchange. The relative roles of trade, ritual, and political authority remain debated. [Springer](https://link.springer.com/article/10.1007/s10816-025-09694-9)

**TCE rule:** Give each transaction an institutional context: household allocation, reciprocal transfer, tribute, compensation, credit purchase, barter, or monetary sale. Do not route every transfer through the market-pricing system.

### 1.2 Separate reciprocal relationships from exact credit

For implementation, use two different records:

**Reciprocity record:** relationship strength, remembered assistance, unmet expectations, and culturally defined obligations. It need not have an exact balance or maturity.

**Debt contract:** creditor, debtor, principal, accounting unit, permitted settlement media, maturity, interest convention, collateral or guarantor, and enforcement jurisdiction.

Ancient records show both deferred agricultural settlement and commercial advances. They also warn against treating all early debts as freely transferable securities: a debt between named parties is not automatically circulating money. [UCI Bren School of ICS](https://ics.uci.edu/~djpatter/classes/2015_06_FutureMoney/assets/docs/hudson.pdf)

**TCE rule:** Credit becomes more money-like only when third parties will accept assignment of the claim. That requires sufficient confidence in the debtor, issuer, guarantor, or enforcement institution.

Useful consequences follow without scripting:

* Harvest-linked debts synchronize repayment demand.
* Crop failures create correlated defaults.
* Migration or social fragmentation weakens relationship-based credit.
* Better records, guarantors, and courts can extend credit beyond close acquaintances.

These are implementation hypotheses to test, rather than universal historical coefficients.

### 1.3 Let commodity money acquire a monetary premium

A monetary commodity needs neither perfect durability nor exclusive monetary use. Its acceptance can depend on familiarity, divisibility, portability, recognizability, supply conditions, and social conventions.

Among the Classic Maya, Baron argues that cacao and cotton textiles acquired monetary functions as marketplaces expanded between competing kingdoms; their established uses in consumption and status display mattered to that process. Monetization then encouraged additional production and procurement of those goods. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/10.1002/sea2.12118)

**TCE rule:** An agent may accept a commodity for either direct use or expected onward exchange. Estimate those alternatives separately and deduct carrying, spoilage, verification, and theft costs.

Do not give a selected commodity an automatic universal acceptance flag. Instead maintain a **local acceptance estimate**, updated from successful and refused payments.

This permits several endogenous outcomes:

* Grain works well for local obligations but poorly for distant transport.
* A durable prestige good becomes useful between trading communities.
* Increased monetary demand draws labor into producing the monetary commodity.
* Monetary commodities are consumed, lost, exported, or diverted into nonmonetary uses.

The same production and inventory systems should handle both monetary and nonmonetary uses.

### 1.4 Taxes and other obligations create demand—but decrees need implementation

An authority can make a medium more useful by accepting it for obligations it can actually enforce. That does not guarantee unlimited purchasing power or acceptance everywhere.

A revealing Chinese example is Sichuan: changes in the proposed treatment of iron and bronze coinage for tax purposes disrupted their relative values; the authorities ultimately accommodated a separate iron-currency zone. Tax receivability and regional circulation were connected, rather than an empire simply having one interchangeable money. [UTS ePress](https://epress.lib.uts.edu.au/journals/index.php/provincial_china/article/view/2844/pdf)

**TCE rule:** Store an obligation’s acceptable media and conversion schedule explicitly. A household owing a tax in silver should seek silver, even while conducting ordinary transactions through grain credit.

Also distinguish:

* accepting a currency for public dues;
* requiring its acceptance for existing private debts;
* requiring merchants to sell goods at a stipulated price.

These are different policies with different enforcement problems. Merchants can respond through repricing, reduced sales, hidden premiums, or withdrawal from trade—not merely obedient acceptance.

### 1.5 Coinage sells verification and standardization

Coinage reduces the work needed to recognize and evaluate metal, but it does not eliminate that work. Stamped coinage appeared in western Anatolia in the seventh century BCE, long after accounting and credit. Its monetary value could depend on weight, fineness, recognized type, official tariff, and local convention. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/2024/wp2024-10.pdf)

Represent a coin issue with:

\[
\text{fine metal per accounting unit}
=
\frac{\text{coin mass}\times\text{fineness}}
{\text{accounting units assigned to the coin}}.
\]

This separates three historically important changes:

**Debasement:** reducing weight or fineness.

**Retariffing:** changing how many accounting units an existing coin represents.

**Redenomination:** changing the scale of accounting units, potentially without changing anyone’s real claims.

**TCE rule:** A new mint ordinance creates a new coin cohort or tariff schedule. It must not retroactively change the physical composition of every existing coin.

Minting should consume metal, fuel, labor, and equipment capacity. Merchants or households surrender bullion voluntarily only when the expected advantages of minted money exceed fees and other costs—or when coercive rules alter their alternatives. Historical debasements were often followed by large minting volumes, but old and new coins could circulate together, sometimes valued by weight. [Federal Reserve Bank of Minneapolis](https://www.minneapolisfed.org/research/quarterly-review/the-debasement-puzzle-an-essay-on-medieval-monetary-history)

### 1.6 Model seigniorage as a transaction, not a treasury button

For a minting transaction, measured consistently in one accounting unit:

\[
\text{net mint revenue}
=
\text{value of coins produced}
-\text{payment for metal}
-\text{minting costs}.
\]

Distinguish the mint’s production charge from the ruler’s net revenue. Also distinguish a **fee as a percentage of coin output** from **seigniorage as a percentage of total government revenue**.

A mathematical illustration: reducing fine metal per unit by 25% allows the same metal to produce approximately 33⅓% more nominal units. It does **not** imply an immediate 33⅓% increase in prices. The outcome depends on what is reminted, how coins are valued, where the proceeds are spent, and how money demand and output respond.

**TCE rule:** The ruler must obtain metal or surrender existing money, operate a mint, distribute the new issue, and face acceptance decisions. Debasement should create winners and losers through those transactions, rather than applying a uniform wealth penalty.

### 1.7 Make Gresham’s law conditional

The familiar “bad money drives out good” result is strongest when coins with different metal values discharge the same nominal obligation at the same valuation. People then have an incentive to spend the overvalued coin and retain, export, or melt the undervalued one.

It is not an unconditional rule that inferior money always wins. When coins can trade at different prices, or are evaluated by weight, multiple qualities can coexist. The historical record includes precisely those complications. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/1997/wp97-12-pdf.pdf)

**TCE rule:** At payment time, choose among eligible holdings by opportunity cost, not face value alone.

For example, an old high-silver coin may be retained because its foreign or bullion value exceeds the domestic obligation it would discharge. A visibly debased coin may instead be discounted or refused.

### 1.8 Currency exchange is a business with inventories and risks

A money changer should do more than apply a global conversion table. Give the business:

* inventories by currency and quality;
* knowledge of coins, issuers, and distant markets;
* bid and ask quotes;
* verification and transport costs;
* exposure to redemption failure and exchange-rate changes.

For two reliably valued currencies containing the same metal, their fine-metal contents provide an arbitrage reference. Transport, assay, minting, restrictions, and settlement delays create a band around that reference—not an exact universal rate.

Long-distance settlement also need not involve physically moving all the coins. Indian **hundis**, for example, served remittance, credit, and trade purposes within indigenous financial networks. Their transfer and payment conventions varied by type. [Reserve Bank of India](https://www.rbi.org.in/Scripts/ms_hundies.aspx)

**TCE rule:** Start with spot exchange and local inventory-adjusted spreads. Later add bills payable at another settlement through correspondents. Final settlement transfers only the net imbalance, subject to available credit and transport.

### 1.9 Separate paper, backing, redemption, and fiat

“Backed” is too ambiguous for a single Boolean field.

| Property | Question the simulation must answer |
| --- | --- |
| **Asset backing** | What assets does the issuer hold against its liabilities? |
| **Convertibility** | Can the holder legally demand a specified asset at a specified rate? |
| **Liquidity** | Can the issuer meet redemption requests when they arrive? |
| **Tax receivability** | Will public authorities accept the instrument for dues? |
| **Acceptance** | Will other people accept it, and at what discount? |

Stockholms Banco issued European banknotes in 1661 against a background of cumbersome copper money. The notes promised redemption in coin; expanding issuance and subsequent redemption demands contributed to the bank’s failure. Portability solved one problem without eliminating issuer risk. [Riksbank](https://www.riksbank.se/en-gb/about-the-riksbank/history/historical-timeline/1600-1699/sveriges-riksbank-is-founded/)

**TCE rule:** Treat a warehouse receipt, a redeemable banknote, a transferable deposit, and sovereign inconvertible currency as distinct instruments. A solvent issuer may be temporarily illiquid; an insolvent issuer may keep paying for a while.

For later banking, a bank loan creates a loan asset and a matching deposit liability. A loan of existing money by a nonbank generally transfers an existing balance instead. Do not implement bank lending as either free wealth creation or an automatic fixed reserve multiplier. [Bank of England](https://www.bankofengland.co.uk/quarterly-bulletin/2014/q1/money-creation-in-the-modern-economy)

### 1.10 Inflation emerges from spending, production, and money demand

Do not connect “percentage debased” directly to “percentage inflation.”

For TCE, route monetary changes through actual budgets and decisions:

1. New spending changes demand at particular suppliers.
2. Suppliers react through output, inventory, and price decisions.
3. Wages and contracts adjust on their own schedules.
4. Agents revise desired currency holdings and payment preferences.
5. Fiscal and financial institutions react to their changing real resources.

A useful historical warning comes from France in 1724: reductions in coins’ nominal valuations were reflected quickly in foreign exchange, while goods prices and wages adjusted more slowly and incompletely. Monetary disturbance was not a synchronized rescaling of every price. [Heraldica](https://www.heraldica.org/econ/velde-1724-v2.pdf)

For hyperinflation scenarios, implement a feedback loop rather than a special status effect:

> Fiscal distress → monetary financing → declining real currency demand → faster spending and currency substitution → larger nominal financing needs.

Allow war damage, lost tax bases, disrupted production, and tax-collection delays to reinforce the loop. Conversely, stabilization requires a credible change in financing and expectations, not merely removing zeros. Sargent’s analysis of the Austrian, Hungarian, German, and Polish stabilizations emphasizes coordinated fiscal and monetary regime changes. [National Bureau of Economic Research](https://www.nber.org/system/files/chapters/c11452/c11452.pdf)

---

## 2. Parameters and historical calibration anchors

### 2.1 Historically grounded values

These values describe particular institutions or episodes. They are **not interchangeable global defaults**.

| Quantity | Historical value or range | Scope and appropriate use | Confidence and source |
| --- | --- | --- | --- |
| Customary silver-loan interest | **20% per year**, expressed as simple annualized interest | Bronze Age Mesopotamian convention; actual contracts varied. Do not apply to all informal assistance. | **M** — Hudson, 2000. [Michael Hudson](https://michael-hudson.com/2000/03/how-interest-rates-were-set-2500-bc-1000-ad/) |
| Customary barley-loan interest | **33⅓% per year** in the cited convention | Old Babylonian agricultural lending; repayment commodity and seasonality make comparison with silver loans nontrivial. | **M** — Hudson, 2000. [Michael Hudson](https://michael-hudson.com/2000/03/how-interest-rates-were-set-2500-bc-1000-ad/) |
| Early official jiaozi conversion charge | **3% of value** | Sichuan’s early official paper-money arrangements; a conversion commission, not a universal minting fee. | **M** — Horesh, 2012. [UTS ePress](https://epress.lib.uts.edu.au/journals/index.php/provincial_china/article/view/2844/pdf) |
| Jiaozi reserve provision | **360,000 guan** against an issue ceiling of **1.25 million guan**; approximately **29%** | A prescribed reserve arrangement, not proof the ratio was continuously maintained. Issues were intended to last **three years**. | **M** — Horesh; ratio calculated from reported amounts. [UTS ePress](https://epress.lib.uts.edu.au/journals/index.php/provincial_china/article/view/2844/pdf) |
| English silver fineness during the Great Debasement | **92.5% → 75% → 50% → 33% → 25%** | Successive issues; retain separate coin cohorts. | **H** — Deng, *The Great Debasement and Its Aftermath*. [Springer](https://link.springer.com/chapter/10.1057/9780230118249_4) |
| English metallic content per pound of account | Approximately **83% reduction, 1542–1551** | Includes changes beyond purity alone. Do not confuse this with the fineness reduction above. | **H/M** — Velde, Weber, and Wright. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/1997/wp97-12-pdf.pdf) |
| English seigniorage contribution to state revenue | Usually **under 2%**, reaching **57%** during the Great Debasement | Share of government revenue—not a 57% mint fee. | **M** — Velde, Weber, and Wright. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/1997/wp97-12-pdf.pdf) |
| Frequency and magnitude of French silver debasements | **123 episodes, 1285–1490**; **112 exceeded 5%**; largest reported **50%** | Evidence for repeated policy changes rather than a single terminal collapse. | **M** — Velde, Weber, and Wright. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/1997/wp97-12-pdf.pdf) |
| French nominal currency contraction, 1724 | Approximately **45%**, through reductions in coin valuations | A retariffing experiment: physical coin numbers did not have to fall proportionately. | **H/M** — Velde, 2009. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/605130) |

The main lesson from the English episode is not a universal inflation coefficient. It is that a ruler could extract substantial resources through repeated monetary alteration while creating heterogeneous coin stocks, valuation problems, and redistribution.

### 2.2 Hyperinflation stress-test anchors

The conventional Cagan threshold is **at least 50% inflation in a month**. It is a classification rule, not an economic switch that should suddenly change agent behavior. [Cato Institute](https://www.cato.org/sites/cato.org/files/images/troubled-currencies-project/routledge-handbook-of-major-events-in-economic-history-world-hyperinflations.pdf)

| Episode and peak month in the cited compilation | Peak monthly inflation | Approximate price-doubling time | Measurement and confidence |
| --- | --- | --- | --- |
| Germany, **October 1923** | **29,500%** | **3.70 days** | Wholesale prices; **M/H** |
| Hungary, **July 1946** | **4.19 × 10¹⁶%** | **15 hours** | Consumer prices; **M/H** |
| China, **April 1949** | **5,070%** | **5.34 days** | Shanghai wholesale prices; **M** |
| Zimbabwe, **mid-November 2008** | **7.96 × 10¹⁰%** | **24.7 hours** | Exchange-rate-implied estimate; **M** |

Values and doubling times are from Hanke and Krus’s historical compilation. These are not homogeneous national consumer-price series. In particular, Zimbabwe’s extreme estimate is reconstructed after official consumer-price reporting ceased, rather than a directly observed official CPI reading. [Cato Institute](https://www.cato.org/sites/cato.org/files/images/troubled-currencies-project/routledge-handbook-of-major-events-in-economic-history-world-hyperinflations.pdf)

For engineering, the important targets are rapid repricing, collapsing holding periods, foreign-currency substitution, deteriorating tax receipts in real terms, and numerical robustness—not reproducing an extreme monthly percentage to many significant digits.

### 2.3 Proposed starting parameters for TCE

**Every entry in this table is P: an uncalibrated engineering prior.** The sources above do not establish universal historical values for these behaviors.

| Parameter | Initial experimental range | Units and implementation |
| --- | --- | --- |
| Counterparties evaluated during a purchase search | **4–16** | Candidates per search; bound computational cost and information access. |
| Active ordinary credit relationships | **4–32** | Relationships per household; merchants and institutions may maintain larger networks. |
| Acceptance-belief memory | **10–100** | Payment observations per half-life; update locally, not from a global popularity score. |
| Desired transaction balances | **7–30** | Days of forecast monetary expenditure; distinguish seasonal taxes and merchant working capital. |
| Ordinary retail repricing opportunities | **1–7** | Days between reviews; permit immediate review after exceptional shocks. |
| Contract-wage reviews | **30–180** | Days for continuing contracts; day labor and new hires may reprice much faster. |
| Initial money-changing spread | **0.5–5%** | Full bid–ask spread under ordinary conditions; stress cases should permit widening or refusal. |
| Mint’s net policy charge | **0–10%** | Share of output value after explicitly modeled production costs; acceptance and bullion supply remain endogenous. |
| Redeemable issuer liquidity target | **10–100%** | Liquid redemption assets relative to immediately redeemable liabilities; explore, do not prescribe as a universal norm. |

Calibration should ask whether plausible combinations reproduce observed qualitative and episode-specific outcomes. It should not choose arbitrary constants and then label them “medieval behavior.”

---

## 3. Variation across eras and regions

### Monetary development was neither synchronized nor irreversible

| Setting | Historical configuration | Implication for TCE |
| --- | --- | --- |
| **Foragers and fishing societies: coastal California** | Shell beads performed monetary functions within and between communities without an agrarian-state prerequisite. The timing and balance of ritual versus commercial uses remain debated. [Springer](https://link.springer.com/article/10.1007/s10816-025-09694-9) | Permit monetary media alongside sharing, kinship obligations, and specialized exchange. |
| **Early agricultural communities** | There is no evidential basis for assigning every early village a universal “barter stage.” The surviving evidence for formal accounts and exact debts is much stronger in later literate societies than in the earliest villages. [Academia](https://www.academia.edu/3621994/Barter_and_Economic_Disintegration) | Begin with household production, reciprocal transfers, and local obligations; monetary adoption remains optional. |
| **Bronze Age Near East** | Commodity accounts, weighed metal, and debt contracts existed long before stamped coins. An Old Assyrian silver-loan tablet at the Met dates to the twentieth–nineteenth centuries BCE. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/325858) | Writing, standardized measures, and enforceable credit should not require coinage. |
| **Classic Maya, 250–900 CE** | Cacao and cotton textiles acquired monetary uses within expanding commercial and political networks. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/10.1002/sea2.12118) | Production of the monetary commodity responds to monetary demand; money need not be metal. |
| **Inka Andes** | State finance depended heavily on labor mobilization, stores, and redistribution rather than generalized coined exchange; scholarship also considers restricted, special-purpose monetary roles for some valuables. [Academia](https://www.academia.edu/18988655/Funding_the_Inka_empire) | A large, administratively capable polity must remain possible without a universal retail currency. |
| **West Africa and the Indian Ocean** | Cowries reached West Africa from the Maldives through long-distance commercial routes. Monetary supply could therefore depend on shipping networks and external procurement. [Cambridge University Press](https://www.cambridge.org/core/books/shell-money-of-the-slave-trade/introduction/BA6D7BD61FE400185074DE612E61A8C0) | Separate the location of monetary production from the location of monetary use. |
| **South Asia, medieval and later commercial networks** | Hundis supported remittance and credit through conventions extending beyond simple coin exchange. [Reserve Bank of India](https://www.rbi.org.in/Scripts/ms_hundies.aspx) | Merchant institutions can connect distant markets before modern banks or electronic payments. |
| **Pre-industrial Europe** | Multiple coin types, accounting units, metallic standards, and credit instruments coexisted. Denomination shortages were a separate problem from the aggregate quantity of money. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/2024/wp2024-10.pdf) | Include small-change constraints and currency specialization by transaction size. |
| **Industrializing and industrial economies** | Banknotes, deposits, and clearing institutions increasingly connected monetary and credit systems; these institutions were not all invented by industrialization itself. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/2024/wp2024-10.pdf) | Scale and communication should change financial reach, rather than an “industrial era” replacing every earlier instrument. |
| **Modern monetary economies** | Bank deposits are central to payments and are largely created through bank lending; physical cash is only one component of the system. [Bank of England](https://www.bankofengland.co.uk/quarterly-bulletin/2014/q1/money-creation-in-the-modern-economy) | Money supply must be derived from issuer balance sheets and instrument definitions, not a fixed global gold stock. |

### China: paper money did not follow a simple success-or-failure trajectory

Private paper claims developed in Sichuan in the context of cumbersome iron money and merchant networks. Official jiaozi arrangements followed in **1023**, with issuance rules, reserves, and conversion facilities. This was an institutional reorganization of existing monetary practices, not simply the discovery that paper was light. [UTS ePress](https://epress.lib.uts.edu.au/journals/index.php/provincial_china/article/view/2844/pdf)

For **1260–1368**, Guan, Palma, and Wu distinguish periods of full silver convertibility, increasingly nominal convertibility, and an inconvertible standard from **1310**. Their reconstruction finds high inflation early and late, but moderate inflation for nearly half a century. Military pressure—particularly civil war—was associated with excessive issuance. Thus, “paper becomes fiat, therefore immediately collapses” is a poor rule. [Hummedia](https://hummedia.manchester.ac.uk/schools/soss/economics/discussionpapers/EDP-2207.pdf)

Later Ming monetary developments included withdrawal from effective state paper issuance and renewed reliance on other media. TCE should permit a state to lose an issuing capability that its predecessors possessed. [UTS ePress](https://epress.lib.uts.edu.au/journals/index.php/provincial_china/article/view/2844/pdf)

### Europe: distinguish monetary innovation from successful monetary governance

The Swedish banknote experiment illustrates portability, credit expansion, and redemption risk together. Its failure does not show that paper was inherently unusable; it shows that an issuer’s promises and available resources could diverge. [Riksbank](https://www.riksbank.se/en-gb/about-the-riksbank/history/historical-timeline/1600-1699/sveriges-riksbank-is-founded/)

Likewise, metallic standards did not guarantee stability. Karaman, Pamuk, and Yıldırım-Karaman document enormous differences in monetary-unit depreciation across European states and the Ottoman Empire. Their analysis emphasizes political and fiscal conditions, including executive constraints, rather than metal alone. These are comparative associations, not a deterministic “good constitution” bonus. [Ataturk Institute](https://ata.bogazici.edu.tr/sites/ata.boun.edu.tr/files/faculty/webuser/moneyandmonetary.pdf)

---

## 4. Stylized facts and validation tests

A credible model should reproduce **conditional patterns**, not force every world through the same historical episode.

| Historical pattern | Validation test for TCE |
| --- | --- |
| **Money coexists with nonmonetary distribution.** | Monetization must not automatically convert household sharing, gifts, or public redistribution into spot purchases. [academia.edu](https://www.academia.edu/3621994/Barter_and_Economic_Disintegration?utm_source=chatgpt.com) |
| **Formal credit predates coins.** | A settlement with reliable records and enforcement but no mint should still support deferred commodity obligations. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/325858) |
| **Small change can be scarce while large money exists.** | A wealthy buyer with only large denominations should sometimes require credit, change-making, or a bundled purchase. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/1997/wp97-8-pdf.pdf) |
| **Old and new coin issues coexist after debasement.** | New mint policy should change the composition of circulating stocks gradually, unless a costly recall is enforced. [Federal Reserve Bank of Minneapolis](https://www.minneapolisfed.org/research/quarterly-review/the-debasement-puzzle-an-essay-on-medieval-monetary-history) |
| **Coin valuations depend on transaction rules.** | Changing fixed nominal acceptance into flexible quality-based quotes should change which coins are spent or retained. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/1997/wp97-12-pdf.pdf) |
| **Prices do not adjust simultaneously.** | A France-1724-style experiment should allow rapid exchange-rate adjustment alongside slower goods-price and wage responses. [Heraldica](https://www.heraldica.org/econ/velde-1724-v2.pdf) |
| **Paper regimes can persist before deteriorating.** | A Yuan-inspired scenario should allow decades of functioning paper circulation, followed by instability when fiscal and military conditions change. [Hummedia](https://hummedia.manchester.ac.uk/schools/soss/economics/discussionpapers/EDP-2207.pdf) |
| **Monetary stabilization requires institutional credibility.** | Replacing the currency’s name or deleting zeros should fail when the underlying financing problem is unchanged. [National Bureau of Economic Research](https://www.nber.org/system/files/chapters/c11452/c11452.pdf) |

Track outcomes at household and occupational levels. A currency reform that appears neutral in aggregate can redistribute resources between wage earners, creditors, debtors, taxpayers, merchants, and those first receiving new issues.

For quantitative experiments, record at least: payment refusals, transaction completion rates, currency holding periods, prices by accounting unit, wages, debt arrears, issuer redemption queues, coin-quality composition, bullion flows, and real government purchases.

---

## 5. Recommended agent and institution model

### 5.1 Minimal authoritative state

Do not represent monetary wealth as one scalar attached to each person.

| Entity | Essential state |
| --- | --- |
| **Household or individual** | Commodity inventories, currency holdings, deposits, debts, expected expenditures, remembered acceptance and issuer reliability. |
| **Accounting unit** | Identifier, subdivision rules, jurisdictions and contracts using it; no requirement for a matching physical coin. |
| **Coin cohort** | Issuer, issue date, nominal tariff, mass, fineness, recognizable type, and wear or clipping category. |
| **Financial claim** | Issuer/debtor, holder, principal, unit, maturity, transferability, redemption terms, and seniority. |
| **Mint** | Capacity, production recipe, bullion purchasing terms, fees, issue specifications, and inventory. |
| **Bank or warehouse issuer** | Assets, liabilities, liquid reserves, capital, lending policy, and redemption queue. |
| **Money changer** | Currency inventories, bid–ask quotes, assay capability, correspondents, and unsettled claims. |
| **Government** | Tax obligations and acceptable media, treasury holdings, spending commitments, debts, and monetary policies. |

Use households for pooled stores where appropriate, while preserving the people who earn, authorize, carry, and spend the resources.

### 5.2 A transaction sequence compatible with posted prices

A workable sequence is:

**First, choose the institutional channel.** Is this household distribution, assistance, an obligation, or a purchase?

**Second, quote in an accounting unit.** A seller’s price is not a universal real-value number silently converted by an omniscient engine.

**Third, propose settlement.** The payer selects eligible cash, commodity, deposit, or credit according to availability and opportunity cost.

**Fourth, evaluate the instrument.** The recipient applies local exchange quotes, quality estimates, issuer discounts, and credit limits.

**Finally, settle atomically.** Transfer the goods and payment, record paired claims where applicable, update inventories, and feed observed outcomes into beliefs.

This lets money interact with TCE’s inventory-based pricing without introducing an equilibrium solver.

### 5.3 Accounting invariants

Make these executable assertions:

**Physical conservation:** Minting and melting reconcile metal inputs, outputs, and process losses. Coin transfers do not create metal.

**Claims consistency:** Every private financial asset has the corresponding issuer liability. Interest accrual, repayment, default, and write-off follow explicit accounting rules.

**No double spending of backing:** Metal held in a warehouse is not simultaneously available in a household purse because the household owns a receipt.

**No automatic reserve multiplier:** Lending decisions depend on profitability, risk, funding, liquidity, and institutional constraints; they are not obtained by multiplying reserves by a fixed constant. [Bank of England](https://www.bankofengland.co.uk/quarterly-bulletin/2014/q1/money-creation-in-the-modern-economy)

**Explicit tax accounting:** Collecting coins transfers ownership to the treasury. It does not physically destroy them. Retirement of issuer liabilities is a different operation.

### 5.4 Performance at 10k–50k people

For the first implementation:

* Aggregate coins into **purse-level cohort counts**, not millions of independently updated objects.
* Maintain **sparse credit networks** and bounded local searches.
* Process debt maturities, taxes, redemption, and minting through scheduled events.

Ordinary purchases need only inspect a few payment alternatives. Most financial institutions can update on daily or market-session schedules; their visible staff and customers still perform individual activities.

Avoid simulating an exchange rate for every possible currency pair everywhere. Maintain quotes where actual trading relationships exist, with intermediate currencies and transport linking markets.

### 5.5 What should be discoverable

Use capability prerequisites rather than era gates:

| Capability | Functional prerequisites |
| --- | --- |
| Commodity accounting | Shared measures, conventions, and social recognition of obligations. |
| Weighed-metal settlement | Scales, metalworking, and credible verification. |
| Recognizable coin issues | Standardized production, recognizable marks, and a trusted or enforced valuation convention. |
| Transferable claims | Record authentication, assignment rules, and credible settlement. |
| Circulating paper instruments | Durable documents, authentication, issuance administration, and an acceptance network. |
| Clearing institutions | Multiple counterparties, account reconciliation, and final-settlement arrangements. |
| Managed fiat system | Durable fiscal and monetary institutions plus continued demand for the unit—not merely a printing press. |

Writing improves scale and persistence but should not be required for every remembered obligation. Likewise, coinage should be neither necessary for commodity money nor sufficient for widespread monetization.

### 5.6 Existing models and games to borrow from

| Reference | Useful component | Limitation |
| --- | --- | --- |
| **Kiyotaki–Wright, 1989** | Endogenous acceptance of exchange media based on goods’ properties and beliefs. | A theoretical environment, not a reconstruction of money’s universal historical origin. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/261634) |
| **Yasutomi, 1995; 2003** | Computational emergence, collapse, and changes in monetary commodities. | Useful for acceptance dynamics, not a complete fiscal or banking economy. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/016727899400234H) |
| **Velde–Weber–Wright, 1999** | Coin quality, information problems, and the debasement puzzle. | Specialized monetary mechanism rather than a complete civilization model. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S1094202598900370) |
| **Kocherlakota, 1998, “Money Is Memory”** | Explains how money can substitute for record-based arrangements under specified assumptions. | Do not generalize its fixed-supply, noncommitment environment into a universal historical claim. [Federal Reserve Bank of Minneapolis](https://www.minneapolisfed.org/research/staff-reports/money-is-memory) |
| **Caiani et al., 2016** | Agent-based, stock-flow-consistent accounting linking real activity and finance. | Borrow the balance-sheet discipline; an agrarian economy needs different behavior and institutions. [SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2664125) |
| **Eco** | Multiple currencies, resource-based minting, accounts, and currency exchanges. | A partial design analogue, not historical validation. Its documented mint backing and coins-per-item settings are permanent, so it is not a direct model of discretionary debasement. [Eco - English Wiki](https://wiki.play.eco/en/Module%3AEcopediaData) |

**Recommended v1 boundary:** Implement reciprocal transfers, exact commodity debts, commodity media, heterogeneous coin cohorts, multiple accounting units, money changers, and mint policy. Add deposit creation and sophisticated clearing only after physical inventories and elementary obligations remain stable under stress.

---

## 6. Sources, datasets, and evidence limits

### Datasets worth building calibration tools around

| Source | Coverage and use | Important limitation |
| --- | --- | --- |
| **Guan, Palma, and Wu: Yuan paper money** | Reconstructed issuance series, **1260–1355**, and associated monetary-regime evidence. | Their money-stock reconstruction assumes **10% annual depreciation** of existing notes. That is a modeling assumption, not a directly measured universal loss rate. [Hummedia](https://hummedia.manchester.ac.uk/schools/soss/economics/discussionpapers/EDP-2207.pdf) |
| **Karaman, Pamuk, and Yıldırım-Karaman** | Monetary standards for **11 states, 1300–1914**, including the Ottoman Empire. Useful for repeated depreciation and institutional comparisons. | Changing boundaries, standards, and source quality; comparative estimates should not become universal causal coefficients. [Ataturk Institute](https://ata.bogazici.edu.tr/sites/ata.boun.edu.tr/files/faculty/webuser/moneyandmonetary.pdf) |
| **Allen–Unger Global Commodity Prices Database** | Commodity-price series from late-medieval Europe to **1914**, plus cities in the Americas, Middle East, and Far East. | Selected commodities and cities; inspect units, quality, gaps, and currency conversions before pooling. [Research Data Journal](https://researchdatajournal.org/article/view/24730) |
| **Bank of England, “A millennium of macroeconomic data”** | UK series reaching the thirteenth century in some cases, with a few earlier benchmarks; version 3.1 extends to **2016**. Separate historical bank balance-sheet datasets are also available. | “A millennium” does not mean complete annual monetary observations for every variable over a thousand years. [Bank of England](https://www.bankofengland.co.uk/statistics/research-datasets) |
| **Hanke–Krus, “World Hyperinflations”** | A historical compilation of **56 episodes** in its publication vintage; useful for extreme stress scenarios. | Mixed price indices and reconstructed exchange-rate measures; not a current census or a homogeneous panel. [Cato Institute](https://www.cato.org/sites/cato.org/files/images/troubled-currencies-project/routledge-handbook-of-major-events-in-economic-history-world-hyperinflations.pdf) |

For origins, prioritize **Humphrey**, **Baron**, **Fauvelle**, and the underlying archaeological and documentary evidence. For coin systems, pair **Rolnick–Velde–Weber** with **Sargent–Velde’s work on small change**. For state manipulation, compare discrete episodes with the longer-run monetary-standard dataset rather than relying only on famous collapses. [Academia](https://www.academia.edu/3621994/Barter_and_Economic_Disintegration)

### Where evidence is thin or contested

**Origins are not directly observable as one global event.** The presence of standardized valuables does not, by itself, establish their acceptance in everyday purchases. Archaeological interpretations depend on production, distribution, context, and use—not just shape or material. [Annual Reviews](https://www.annualreviews.org/content/journals/10.1146/annurev-anthro-092611-145716)

**Written evidence overrepresents formal institutions and disputes.** It provides much better access to recorded obligations than to ordinary unrecorded assistance. Consequently, the abundance of debt documents cannot by itself establish the proportion of all transfers conducted on credit.

**Coin evidence and money stocks are different things.** A large mint output may include reminting of existing metal; coin finds are not a direct census of circulating purchasing power. Historical monetary reconstruction must distinguish production flows, surviving objects, hoards, and circulating stocks. [Federal Reserve Bank of Chicago](https://www.chicagofed.org/-/media/publications/working-papers/2024/wp2024-10.pdf)

**There is no defensible universal debasement-to-inflation coefficient, trust score, exchange spread, or monetary-adoption population threshold.** These should remain explicit calibration choices until tied to particular evidence.

### Bottom line

The most productive abstraction for TCE is **a changing network of obligations and accepted settlement instruments**.

People should adopt money because it helps them transact or discharge obligations. Institutions should make particular instruments more useful by verifying, recording, redeeming, clearing, or accepting them. Rulers should gain from monetary manipulation only through transactions that also change other agents’ incentives.

That framework allows money to emerge, coexist with other arrangements, spread across borders, fragment, become a source of state power, and fail—without scripting any of those outcomes.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9287b-eda8-83ea-9555-bec43425838a)
