# Credit, interest, and banking before industrialization

## A simulation-ready report for The Civilization Engine

**The most useful abstraction for TCE is a network of enforceable claims on future goods, income, and payments—not a single “banking technology” or an interest rate attached to each era.**

Silver-weight and grain loans, merchant partnerships, notarial lending, transferable bills, and deposit banking solved different problems. They did not necessarily appear in one universal sequence. Sophisticated credit could operate without coined money, and substantial lending could operate without banks. Old Assyrian commercial records and eighteenth-century French bills provide particularly clear examples of these possibilities. [ResearchGate](https://www.researchgate.net/publication/332118991_The_Old_Assyrian_Trade_and_its_Participants)

For implementation, distinguish three layers:

* **Economic need:** someone requires resources now but expects income later.
* **Contract and enforcement:** someone else acquires a claim, with specified repayment, collateral, guarantees, and remedies.
* **Payment infrastructure:** institutions make claims easier to record, transfer, settle, or convert into money.

The historical evidence below provides calibration targets. The equations, software architecture, and explicitly labeled design priors are modeling recommendations rather than historical measurements.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Credit bridges mismatched timing—but does not create the underlying goods

Start with projected resource balances, not an abstract appetite for loans.

A household might need seed before harvest; a merchant must purchase cargo before selling it elsewhere; a builder must pay workers before a building generates rent. In TCE, each proposed loan should therefore have a **purpose, expected repayment source, and cash-flow timetable**.

A useful borrowing rule is:

\[
\text{requested advance}
=
\max(0,\ \text{resources needed before receipts}
-\text{available own resources}
-\text{other committed financing})
\]

Distinguish productive borrowing from consumption smoothing and refinancing. The latter two are not necessarily irrational: a consumption loan can preserve a household’s labor capacity, while refinancing can prevent liquidation of a viable enterprise. But neither should automatically increase productive capacity.

**Implementation consequence:** credit changes which projects and purchases can happen, and when. It should not confer a flat productivity bonus. When labor, timber, grain, or transport are already fully utilized, additional purchasing power can instead raise prices or redirect resources.

Historical lending purposes were mixed. In eighteenth-century New Spain, ecclesiastical credit financed commerce, property transactions, agriculture, and repayment of earlier obligations; the borrower’s occupation did not reliably identify the loan’s actual use. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/ECE010.pdf)

### 1.2 Reciprocal support and enforceable debt are different relationships

A promise to help a relative later is not necessarily a loan with a principal, maturity, and interest rate. Ethnographic research finds reciprocity in food sharing, alongside kinship and other mechanisms, but it does not establish a prehistoric market interest rate. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3757985/)

Use two separate systems:

**Reciprocal obligation:** a relationship-level memory of assistance, need, generosity, and refusal. Repayment may be delayed, asymmetric, or made in another form.

**Credit contract:** a specific claim that can become overdue, be inherited or transferred where permitted, and trigger agreed remedies.

This prevents early societies from becoming modern banks with invisible ledgers. It also allows formal credit to coexist with gifts, patronage, family assistance, and communal insurance.

### 1.3 Lending depends on information and enforceability—not wealth alone

Lenders should observe imperfect signals: previous repayments, household resources, expected harvest, business experience, witnesses, guarantors, and their ability to recover a claim.

A simple underwriting condition is:

\[
(1-p)(1+iT)+pR
\;\geq\;
1+cT+\frac{F}{P}
\]

where:

* \(P\): principal advanced;
* \(T\): term in years;
* \(i\): annual simple-interest rate;
* \(p\): estimated probability of default over that term;
* \(R\): net recovery in default, as a fraction of principal;
* \(c\): annual funding or opportunity cost;
* \(F\): transaction and monitoring costs not already included elsewhere.

This gives:

\[
i\_{\min}
=
\frac{cT+F/P+p(1-R)}{(1-p)T}
\]

Use this as a lender’s approximate reservation rate, not as a perfect market-clearing equation. Small loans can be expensive because fixed costs are large relative to principal.

Crucially, increasing the rate can change who borrows and which risks they take. Stiglitz and Weiss show why lenders may refuse additional loans even when applicants offer to pay more. TCE therefore needs **credit rationing**, not unlimited lending at sufficiently high rates. [ResearchGate](https://www.researchgate.net/publication/4733120_Credit_Rationing_in_Markets_With_Imperfect_Information)

Legal power can also work counterintuitively. In Istanbul court records for 1602–1799, Kuran and Rubin find that legally advantaged groups—including men, Muslims, and titled elites—paid higher rates conditional on observed characteristics. Their explanation emphasizes the difficulty of enforcing claims against privileged borrowers. This is a context-specific finding, not an intrinsic credit characteristic of those groups. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12389)

**Rule:** derive creditworthiness from resources, information, and the applicable legal relationship. Do not assign universal ethnic, religious, or class-based “trustworthiness” bonuses.

### 1.4 Collateral and guarantees create access—and transmission channels for crises

Collateral is useful only when it can be identified, pledged, seized, and sold. Communally held land, contested inheritance, overlapping liens, or politically protected ownership can make valuable property poor security.

Calculate a collateral ceiling from recoverable value:

\[
P\_{\max}
=
\text{expected sale value}
\times\text{recoverable share}
-\text{senior claims}
-\text{collection costs}
\]

A guarantor provides an alternative source of repayment. This can support lending without a modern mortgage market: in one New Spanish sample of **336 merchant loans with known security, 71.43% relied on guarantors alone**. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/ECE010.pdf)

In TCE, guarantees must be contingent obligations, not free credit multipliers. A merchant guaranteeing several neighbors can connect otherwise separate failures. Similarly, many loans secured on local farmland remain exposed to the same harvest and land-price shock.

A falling collateral price should reduce new borrowing capacity and increase pressure to sell assets. This is the central amplification mechanism in Kiyotaki and Moore’s *Credit Cycles*. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/262072?utm_source=chatgpt.com)

### 1.5 Usury rules change contract forms and access, not merely the displayed rate

Separate four legal dimensions: which returns are prohibited, which borrowers or lenders are covered, what remedies are permitted, and how strongly rules are enforced.

European responses to usury restrictions included annuities and bills whose returns could be embedded in exchange transactions. A *rente* was legally structured as the purchase of an income stream rather than an ordinary repayable loan. These distinctions could matter substantively; they were not all interchangeable disguises. Munro’s research also emphasizes that transferability and full legal negotiability developed separately. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/10925/)

Ottoman credit likewise operated despite religious restrictions. Cash endowments lent capital to fund charitable purposes, while legal devices and partnerships supported finance. A genuine *mudaraba* allocated capital and entrepreneurial effort with an agreed division of profit; its risk allocation should not be reduced to a fixed-interest loan. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SPamuk.pdf)

**Rule:** represent the actual payoff and legal form separately. A fixed markup, genuine profit share, rent payment, exchange margin, and penalty can have different risks and enforceability.

A binding cap may produce smaller credit supply, larger minimum loans, more security requirements, or circumvention. When England lowered its ceiling from **6% to 5% in 1714**, evidence from Hoare’s Bank shows changes in loan allocation and access, not simply cheaper borrowing for everyone. [Hans-Joachim Voth](https://www.jvoth.com/publications.html)

Do not hard-code every cap as harmful: effects depend on whether it binds, lender market power, costs, and available alternatives.

### 1.6 Separate moneylending, safekeeping, payments, and banking

These are different businesses:

| Function | What the institution does | Main constraint or risk |
| --- | --- | --- |
| Own-funds moneylending | Advances its existing grain, metal, or money | Borrower default; tying up resources |
| Safekeeping | Holds identifiable property for customers | Theft, fraud, custody costs |
| Deposit intermediation | Owes depositors repayable balances and invests funds | Withdrawals before assets mature |
| Transfer banking | Settles payments by changing account ownership | Acceptance, clearing, operational integrity |
| Bill acceptance or discounting | Guarantees or purchases future payment claims | Counterparty, maturity, and network risk |
| Partnership finance | Shares an enterprise’s profit and specified losses | Business risk and concealed effort or profits |

Temples and religious institutions should not automatically receive all six capabilities. Neo-Babylonian institutional archives include lending without interest and institutions acting as borrowers; “temple” is not sufficient evidence for a modern-style deposit bank. [Academia](https://www.academia.edu/951551/Debts_and_Indebtedness_in_the_Neo_Babylonian_Period_Evidence_from_the_Institutional_Archives)

Where transferable bank deposits are accepted as money, a loan can create a deposit:

| Transaction | Bank assets | Bank liabilities |
| --- | --- | --- |
| Originate a loan of 100 | Loan claim +100 | Borrower deposit +100 |
| Borrower pays another customer of the same bank | No total change | Deposit ownership changes |
| Recipient transfers funds to another bank | Settlement reserves decrease | Deposits decrease |
| Principal is repaid from a deposit | Loan claim decreases | Deposits decrease |

This is the accounting logic explained by the Bank of England for modern deposit creation. Apply it only to TCE institutions whose liabilities are actually accepted for payment—not retroactively to every ancient lender or warehouse. [Bank of England](https://www.bankofengland.co.uk/quarterly-bulletin/2014/q1/money-creation-in-the-modern-economy)

**Reserves are not capital.** Reserves meet payments; capital absorbs losses. A bank can have positive net worth but insufficient immediately available settlement assets.

### 1.7 Bills reduce the need to transport money—but move risk into networks

A bill can combine a future payment, a transfer between places, and conversion between currencies. Its simulation representation needs the issuer, payer, holder, maturity, denomination, and applicable recourse.

Endorsement and joint liability can make a claim usable beyond the original relationship. Santarosa’s study of eighteenth-century France finds that joint liability helped bills support distant and relatively impersonal trade **without requiring banks to intermediate every transaction**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/financing-longdistance-trade-the-joint-liability-rule-and-bills-of-exchange-in-eighteenthcentury-france/D599942EA833A888832154599B9D4C48)

For TCE, transferring a bill changes ownership of an existing claim; discounting exchanges that claim for an immediate payment. Neither action should duplicate the underlying debt.

Allow multiple legal capabilities: simple assignment, endorsement, acceptance, and recourse against previous endorsers. Do not grant every discovered bill full modern negotiability.

### 1.8 Public debt is a claim on politically accessible revenue

A treasury borrows against anticipated taxes, monopolies, tribute, or other receipts. Its creditworthiness depends on collection capacity and willingness to devote those resources to creditors.

Use a cash-budget identity:

\[
\text{ending cash}
=
\text{starting cash}
+\text{revenue}
+\text{new financing}
-\text{expenditure}
-\text{interest}
-\text{principal due}
\]

But default decisions must also consider military survival, domestic opposition, creditor coordination, and alternative finance.

Historical public debt included income streams and long-lived obligations, not only amortizing loans. The Bank of England’s founding loan in **1694 was £1.2 million at 8%**, illustrating how institution-building and state financing could be linked. [Bank of England](https://www.bankofengland.co.uk/freedom-of-information/2020/details-of-the-bank-of-england-loan-to-the-government-in-1694?utm_source=chatgpt.com)

Philip II’s payment suspensions show why default should not automatically erase all claims or permanently end lending. Settlements altered the timing and form of payment; nominal principal preservation could coexist with substantial present-value losses. [EconPapers](https://econ-papers.upf.edu/papers/1164.pdf)

**Rule:** default can result in a standstill, maturity extension, reduced coupon, principal haircut, exchange into tax-backed claims, or preferential treatment of selected creditors.

### 1.9 Bank runs and credit crunches should arise from balance sheets and expectations

A run occurs when payment demands exceed:

\[
\text{available reserves}
+\text{near-term receipts}
+\text{obtainable emergency funding}
+\text{sale proceeds}
\]

Depositors need not know whether the bank is insolvent. Fear that others will withdraw first can create an incentive to withdraw from an otherwise viable institution—the mechanism formalized by Diamond and Dybvig. [DOI](https://doi.org/10.6082/3sa4m-hvw53)

Runs need not involve retail depositors. In the **1763 Amsterdam crisis**, important merchant banks relied on short-term wholesale credit. Failure and non-renewal of funding propagated distress; Quinn and Roberds identify emergency collateral policies at the Bank of Amsterdam as an important response. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/jmcb.12240)

A TCE crisis can therefore follow this chain:

**Harvest, trade, or fiscal shock → arrears → reduced confidence or non-renewal → asset sales → lower collateral values → further losses → fewer new loans.**

Support institutions should have limited resources, eligibility rules, and political backing. A liquidity rescue does not automatically repair insolvency.

---

## 2. Quantitative parameters and calibration anchors

### How to read the evidence

**H** means high confidence in the stated legal provision or bounded archival observation. **M** means a reconstruction, heterogeneous sample, or significant interpretive limitation. Neither label guarantees representativeness.

Unless otherwise specified, annual rates below are nominal contractual rates—not inflation-adjusted returns and not realized lender profits.

### 2.1 Historical interest-rate anchors

| Place and period | Instrument or observation | Quantitative anchor | Type and confidence |
| --- | --- | --- | --- |
| Old Assyrian commerce, early second millennium BCE | Selected silver loans | **20% or 30% per year** | Contractual examples, not a regional average. **H/M**. Dercksen. [ResearchGate](https://www.researchgate.net/publication/332118991_The_Old_Assyrian_Trade_and_its_Participants) |
| Old Babylonian Mesopotamia | Conventional silver and barley lending schedules | **20% silver; 33⅓% barley** | Customary/legal increments. Annual interpretation, especially for grain, requires contract-specific checking. **H** for convention; **M** for period interpretation. [ResearchGate](https://www.researchgate.net/publication/332118991_The_Old_Assyrian_Trade_and_its_Participants) |
| Athens–Black Sea, fourth century BCE | Maritime loan in Demosthenes 35 | **22.5% for the voyage; 30% under a later-season condition** | **Per voyage, not per year**. One surviving contract; includes maritime risk. **H**. [Society for Classical Studies](https://classicalstudies.org/maritime-lenders-managing-risk-4th-century-athens) |
| Justinianic Roman Empire, sixth century CE | Statutory maximum rates | **4%, 6%, 8%, or 12% annually**, according to category | Legal ceilings, not observed averages. Categories include lender status and maritime lending. **H**. *Code* 4.32.26. [Roman Law Library](https://droitromain.univ-grenoble-alpes.fr/Anglica/CJ4_Scott.htm) |
| Anatolian towns, around 1600 | Court-recorded private lending | Approximately **10–20% annually** | Archival range summarized by Pamuk from judicial studies. **M**. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SPamuk.pdf) |
| India, seventeenth century | Indian merchants lending to the Dutch East India Company | Surat **12→7.5%**; Bengal **15→10%**; Coromandel **24→12%** annually over the century | Large commercial borrower; **not peasant rates**. **M**. Boomgaard. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-PBoomgard.pdf) |
| Selected Southeast Asian settings, roughly 1600–1900 | Indigenous lending, including secured loans | **25–35% annually** appears as a lower-end range in the cases discussed | Broad, heterogeneous historical synthesis; not a universal regional distribution. **M/low representativeness**. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-PBoomgard.pdf) |
| Osaka, 1707–1740 | Kōnoike loans to daimyō | **12.5% annually** | Contract-rate reconstruction for this lender/borrower class. **H/M**. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SaitoSettsu.pdf) |
| Osaka, 1841–1860 | Same broad lending relationship | **6.8%**, or **8.7%** when unusually low-rate cases are excluded | Inclusion rules materially change the estimate. **M**. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SaitoSettsu.pdf) |
| Izumi, Japan, eighteenth–nineteenth centuries | Secured rural loans | **18.0%** in 1728–1758; approximately **10–12%** in several later periods | Local contract classes, not all Japanese rural finance. **M**. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SaitoSettsu.pdf) |
| China versus Britain, 1770–1860 | Rates inferred from grain-price/storage relationships | Britain approximately **3 percentage points lower** | Model-derived comparison, **not directly observed loan quotations**. **M**. Keller, Shiue, and Wang. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20180299) |
| Castilian crown, 1566–1600 | Shorter-term *asiento* contracts | Mean stated rate **9.9% annually**; observed range **0–16%**, for **318 contracts with stated rates** | Excludes important fees and exchange components. **H** within sample. [EconPapers](https://econ-papers.upf.edu/papers/1164.pdf) |
| England, 1694 | Founding Bank of England loan to government | **8% annually** | Specific contractual rate on £1.2 million. **H**. [Bank of England](https://www.bankofengland.co.uk/freedom-of-information/2020/details-of-the-bank-of-england-loan-to-the-government-in-1694?utm_source=chatgpt.com) |
| England, 1714 | Usury ceiling | **6% → 5% annually** | Legal change, not a universal change in borrowing cost. **H**. [Hans-Joachim Voth](https://www.jvoth.com/publications.html) |
| New Spain, eighteenth century | Ecclesiastical lending and income-bearing contracts | Commonly **5% annually**; some late-century loans **4–4.5%** | Institutional and contractual selection matters. **H/M**. Wobeser. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/ECE004.pdf) |
| Ottoman state, late eighteenth–mid-nineteenth century | Implied cost of particular revenue-backed issues | Roughly **12–15% annually**; **15–20% or more** under distress | Reconstructed fiscal financing costs, not all state borrowing. **M**. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SPamuk.pdf) |

**Do not average this table into an “ancient,” “medieval,” or “Asian” rate.** It mixes contractual prices, legal ceilings, contingent voyage returns, and indirect estimates.

Two reporting transformations are useful:

\[
r\_{\text{annual equivalent}}
=
\left(\frac{\text{repayment}}{\text{net advance}}\right)^{1/T}-1
\]

This is a comparison measure; it does **not** imply that the original contract compounded or could be repeatedly rolled over.

For commodity loans, value the repayment in both the contractual commodity and a chosen consumption basket. **Illustrative calculation, not a historical observation:** lending 100 units of grain and receiving \(133\frac13\) yields 33⅓% in grain. If grain’s money price has fallen to 70% of its initial level, the money-value return is approximately **−6⅔%** before costs. Different numeraires answer different questions.

### 2.2 Other measured parameters

| Quantity | Historical anchor | Appropriate use |
| --- | --- | --- |
| Mean term of Spanish *asientos*, 1566–1600 | Approximately **22.6 months** across **438 contracts**; very heterogeneous | Test a mixture of short finance, longer advances, and restructuring contracts—not one fixed maturity. [EconPapers](https://econ-papers.upf.edu/papers/1164.pdf) |
| New Spanish *depósito irregular* lending terms | Commonly **2–5 years** | A medium-term loan template. The name does not mean a modern demand-deposit account. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/ECE004.pdf) |
| Spanish settlement after the 1575 suspension | Approximately **62% present-value recovery**, equivalent to **38% loss** on average | Sovereign restructuring scenario; not necessarily a 38% face-value cancellation. [EconPapers](https://econ-papers.upf.edu/papers/1164.pdf) |
| Spanish settlement following the 1596 suspension | Approximately **20% average present-value haircut** | A less severe restructuring scenario. [EconPapers](https://econ-papers.upf.edu/papers/1164.pdf) |
| Guarantor-only security in a New Spanish merchant-loan sample | **71.43% of 336 loans** with known security | Demonstrates that a large formal credit network need not be dominated by land mortgages. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/ECE010.pdf) |

### 2.3 Explicit design priors where historical calibration is weak

These are **starting ranges for sensitivity tests**, not estimates of what “most pre-industrial societies” did.

| Parameter | Initial test range | Interpretation |
| --- | --- | --- |
| Seasonal household-loan maturity | **0.25–1 year** | Prefer harvest-linked dates over arbitrary monthly amortization |
| Merchant advance maturity | **1–12 months** | Tie to route duration, fairs, and settlement delays |
| Construction financing horizon | **1–10 years** | Separate staged advances from longer repayment obligations |
| Secured advance / conservatively valued collateral | **30–70%** | Vary by asset liquidity and enforcement rights |
| Net recovery on defaulted secured principal | **20–80%** | Stress-test collection cost, priority, and forced-sale conditions |
| Fractional intermediary’s liquid-reserve target | **10–40% of runnable liabilities** | Behavioral prudence parameter, not a historical reserve requirement |
| Pure custody institution’s coverage of safeguarded assets | **100%** | A different contract, not a “less advanced” fractional bank |
| Settlement/collection delay | **Days to years**, generated from geography and institutions | Do not replace distant litigation with an instantaneous roll |

Avoid assigning a universal annual default probability. Generate missed payments from household and enterprise outcomes; calibrate lender expectations to the resulting experience. Correlated crop and fiscal risks matter more than an arbitrary independent “default die.”

---

## 3. Variation across eras and regions

### 3.1 Eras should change capabilities, not impose a historical script

| Setting | Representation suited to TCE | Important qualification |
| --- | --- | --- |
| **Foragers** | Reciprocal support, entrusted goods, negotiated obligations, reputation | Ethnographic sharing evidence does not identify prehistoric interest rates or prove universal absence of formal debt. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3757985/) |
| **Early farming** | Seasonal commodity advances, household creditors, witnesses, storage-linked claims | Do not assume later documented Mesopotamian institutions existed in the first farming villages. |
| **Pre-industrial complex economies** | Moneylenders, institutional endowments, partnerships, notaries, bills, tax-backed obligations, optional deposit banking | Different combinations could coexist; private credit need not wait for banks. [CaltechAUTHORS](https://authors.library.caltech.edu/records/msbt8-7rc77) |
| **Industrial economies** | Larger and longer-lived enterprises, interconnected financial balance sheets, expanded capital markets | Financing constraints can still operate through loan quantities rather than visible rate increases. Hoare’s evidence spans 1702–1862. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0014498304000646) |
| **Modern economies** | Widely accepted bank deposits, policy-controlled settlement systems, larger datasets for calibration | Modern deposit creation is an institutional regime, not a universal description of earlier finance. [Bank of England](https://www.bankofengland.co.uk/quarterly-bulletin/2014/q1/money-creation-in-the-modern-economy) |

### 3.2 Regional differences worth preserving

**Mesopotamia and the ancient Mediterranean.** Model commodity and weighed-metal denominations, household and commercial creditors, and contract-specific calendars. Ancient lending conventions are evidence for standardized terms, not proof that rates always moved competitively with marginal productivity. Hudson’s interpretation emphasizes arithmetic conventions; its explanatory importance remains debatable. [Brill](https://brill.com/view/journals/jesh/43/2/article-p132_2.xml)

**Islamic societies and South Asia.** Avoid equating religious restrictions with absent finance. Also avoid equating *hundi* with a single modern remittance product: Martin documents the term’s changing and broader commercial-financial meanings. In TCE, remittance, exchange, credit, and guarantees can share an institutional network without becoming one identical contract. [Cambridge University Press](https://www.cambridge.org/core/journals/modern-asian-studies/article/abs/hundihawala-the-problem-of-definition1/0CD2B3BF3DC118187C98592321AE13CC)

**China.** Include pawnshops, money-changing and remittance institutions, merchant finance, and interinstitutional lending. Peng, Chen, and Yuan’s reconstruction draws on commercial records rather than treating expensive rural borrowing as representative of all Chinese capital markets. Meanwhile, grain-price-derived comparisons depend on storage-model assumptions. [Jryj](https://www.jryj.org.cn/EN/abstract/abstract1199.shtml)

**Japan.** Osaka finance linked merchants, remittance, and daimyō revenues. Declining rates were not necessarily evidence of uninterrupted financial progress: Saito and Settsu discuss weakening local demand and concessions to borrowers as relevant explanations. Their contracted-rate and received-interest tables concern different portfolios and must not be subtracted to infer a same-loan default loss. [LSE](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SaitoSettsu.pdf)

**Africa.** Do not substitute “kin sharing only” for missing numerical data. Austin’s Asante research identifies savings, equity participation, and loans as sources of investment finance. Other scholarship documents pawnship and coercive credit relationships in particular African settings. These require region-specific legal and social rules; I did not find a defensible continent-wide pre-industrial interest-rate distribution. [Cambridge University Press](https://www.cambridge.org/core/books/abs/labour-land-and-capital-in-ghana/capital-and-credit-18071896/4F761229CAB9C6DB2B2FC5E003201593)

**The Americas.** Colonial New Spain demonstrates that religious institutions could supply substantial credit to merchants and property owners. It is not a proxy for pre-contact American societies. Nor should documented property finance be automatically counted as finance for new construction rather than purchase or refinancing. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/ECE010.pdf)

---

## 4. Stylized facts a correct simulation should reproduce

These are validation targets, not requirements that every generated world reproduce every historical outcome.

| Pattern | Evidence or benchmark | Simulation test |
| --- | --- | --- |
| **Wide rate dispersion within one technological setting** | The historical table contains low-cost institutional finance alongside much more expensive local and risky lending | Rates and access should depend on borrower, contract, security, network, and jurisdiction—not only technology |
| **Quoted rates can remain stable while access changes** | Hoare’s response to the 1714 ceiling change altered allocation and security requirements | Tighten a binding cap; check approval rates, loan sizes, and borrower composition, not just average interest. [Hans-Joachim Voth](https://www.jvoth.com/publications.html) |
| **Credit can expand without deposit banks** | Notarial Paris and bill-based French trade | Enable records, matching, and assignability while leaving deposit banking disabled. [CaltechAUTHORS](https://authors.library.caltech.edu/records/msbt8-7rc77) |
| **Legally powerful borrowers need not borrow cheaply** | Istanbul’s conditional status-rate findings | Give a borrower legal protection from creditors; improved wealth should not automatically dominate poorer recoverability. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12389) |
| **Financial integration is geographically uneven** | For distances above 200 km, Keller and colleagues find British spatial correlations about twice those in the Yangzi Delta and three times those in other Chinese regions | Information, settlement, and enforcement networks should integrate some routes before others. These ratios are sample benchmarks, not universal distance laws. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20180299) |
| **Default does not imply permanent exclusion or universal lender failure** | Research on Habsburg Spain documents continuing sovereign lending and heterogeneous creditor outcomes | Permit negotiated re-entry and lender differentiation rather than a permanent global blacklist. [OUP Academic](https://academic.oup.com/princeton-scholarship-online/book/21041/chapter-abstract/180602132) |
| **A funding panic can occur without retail deposit withdrawals** | Amsterdam, 1763 | Withdraw wholesale renewals while household deposits remain unchanged; exposed merchant banks should still become illiquid. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/jmcb.12240) |
| **Credit and asset values can amplify each other** | Kiyotaki–Moore collateral mechanism | Impose a temporary income shock; compare recovery with and without collateral-dependent borrowing limits. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/262072?utm_source=chatgpt.com) |
| **Government borrowing can affect private loan quantities** | Hoare’s wartime evidence | Increase attractive or compulsory state borrowing; test whether merchant and household credit contracts even if quoted rates barely move. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0014498304000646) |

Also track **realized lender returns** after losses, collection costs, delayed payment, and price changes. A model matching promised rates but bankrupting every lender—or making every lender effortlessly rich—has not matched the historical economics.

---

## 5. Recommended representation for TCE

### 5.1 Use persons, pooled households, and institutions together

Individuals should retain property rights, inheritance positions, guarantees, reputation, and legal status. A household can pool consumption and some resources without erasing individual ownership.

Institutions need their own balance sheets and decision rules:

| Entity | Minimum state |
| --- | --- |
| Household | Resource stocks, expected receipts, essential expenditures, debts, claims, ownership rules |
| Merchant/workshop | Working capital, inventory in transit, receivables, payables, project commitments |
| Religious endowment | Restricted capital, permitted investments, beneficiaries, distribution obligations |
| Lender/bank | Assets, liabilities, reserves, capital, liquidity targets, exposures, private information |
| Notary/court/registry | Records, fees, jurisdiction, processing capacity, enforceability |
| Treasury | Tax claims, collection schedule, spending commitments, debt service, political constraints |

Managers and owners should remain agents. Their incentives can produce favoritism, concealed losses, self-dealing, or prudent restraint without turning the institution into a perfectly optimizing machine.

### 5.2 Store each financial claim once

A canonical contract should contain:

```
contract_id
creditor / current_holder
debtor
denomination and unit definition
principal advanced and remaining contractual principal
payoff rule and accrual convention
payment dates and maturity conditions
collateral identifiers, liens, and priority
guarantors and guarantee limits
transferability, endorsement, and recourse rules
jurisdiction and permitted remedies
status: performing / overdue / restructured / discharged
```

Use a small family of payoff rules: simple-interest debt, fixed repayment or discount, commodity increment, contingent voyage debt, annuity, and profit-sharing investment. Keep genuine equity or partnership losses distinct from ordinary debt default.

**The engine’s accounting should be rigorous even when characters use oral agreements or simple tallies.** Bookkeeping sophistication changes what agents can know and administer, not whether resources are conserved.

Important invariants:

* A transfer changes a claim’s holder; it does not create another copy.
* A legal haircut changes the contractual obligation on both sides.
* A lender’s impairment allowance can reduce the asset’s book value **without legally forgiving the borrower**.
* Guarantees are contingent until triggered.
* Collateral cannot be counted as unencumbered security for several senior claims simultaneously.
* Interest, principal, fees, and recoveries are separate flows.

This distinction between legal face value and estimated recoverable value prevents both magical debt disappearance and centuries of uncollectible interest being treated as spendable wealth.

### 5.3 Make underwriting local and adaptive

For each financing need, search a bounded set of known or discoverable lenders: relatives, neighbors, trading partners, institutional offices, and intermediaries. Expand the search only when justified by the loan’s size or urgency.

A practical lender routine is:

1. Estimate repayment resources and adverse scenarios.
2. Check enforceability, existing obligations, and available security.
3. Offer a conventional local contract, adjust terms within legal bounds, or refuse.
4. Update expectations after outcomes and information received.

Use separate beliefs for each lender rather than a universally visible credit score. Observed repayment by one merchant should spread through meetings, correspondence, courts, and reputation networks—not teleport across the world.

For an agrarian lender, portfolio diversification should be evaluated across **shock exposures**, not simply borrower count. Fifty farms drawing water from one irrigation system are not fifty independent risks.

### 5.4 Give construction its own financing state

A construction project needs a staged resource schedule, not merely an initial loan and completion timer.

For each stage:

\[
\text{financing gap}
=
\text{wages due}
+\text{materials due}
+\text{transport due}
-\text{available project funds}
-\text{committed incoming advances}
\]

Represent owner funds, lender advances, supplier credit, customer prepayments, and unpaid contractor obligations separately. When financing fails, determine whether work pauses, materials are sold, claims are renegotiated, or the unfinished structure changes hands.

The unfinished asset should usually have a different recovery value from a completed income-producing building. Let this follow reusable materials, completion cost, alternative buyers, and local demand—not a universal fixed percentage.

Public construction can instead use taxes in kind, labor obligations, or current revenue. Banking must not become a prerequisite for irrigation, roads, temples, or walls.

### 5.5 Discover institutional capabilities independently

The following is a proposed dependency graph, not a claim about universal historical order:

| Capability | Preconditions that make it useful | What it enables |
| --- | --- | --- |
| Witnessed debt | Recognizable obligations and accepted adjudication | Loans beyond immediate mutual aid |
| Standard contract records | Durable records, trained specialists | More contracts and longer institutional memory |
| Pledge and lien recognition | Identifiable property and priority rules | Secured advances |
| Guarantee contracts | Enforceable contingent liability | Borrowing through networks |
| Partnership contract | Accounting and agreed loss allocation | Risk-sharing investment |
| Assignment and negotiability | Recognition of new holders and their remedies | Tradeable claims and bills |
| Transferable deposit accounts | Accepted institution, ledger integrity, settlement arrangements | Account-based payment and deposit creation |
| Funded public debt | Persistent revenue claims and political credibility | Longer-horizon fiscal borrowing |
| Emergency liquidity support | Credible funder, eligible collateral, loss-bearing rules | Containment of some funding crises |

Adoption should depend on expected gains exceeding setup, administration, enforcement, and political costs. Institutions can be prohibited, captured, lose credibility, or disappear. Discovery need not be irreversible.

### 5.6 Keep finance event-driven

At 10k–50k people, the financial system should operate primarily on the economic calendar, not the rendering frame.

Schedule maturities, installments, harvest settlements, court hearings, and bill presentations. Accrue ordinary interest lazily when a contract is inspected or an event occurs. Reassess borrowers when information changes rather than rescoring the whole population continuously.

With \(C\) active contracts, a heap-based scheduler can process events in roughly \(O(\log C)\) per insertion/removal; calendar buckets can be cheaper for bounded daily horizons. Benchmark actual workloads before choosing.

Use deterministic ordering or seeded tie-breaking for simultaneous demands. Whether deposits are paid first-come-first-served, proportionally, or under suspension should be an institutional rule—not an accidental consequence of memory order.

### 5.7 Existing models and games to borrow from

| Reference | Useful component | What not to import uncritically |
| --- | --- | --- |
| **Stiglitz–Weiss, 1981**, *Credit Rationing in Markets with Imperfect Information* | Refusal and quantity rationing despite willingness to pay higher rates | Its stylized information assumptions as a complete historical economy. [ResearchGate](https://www.researchgate.net/publication/4733120_Credit_Rationing_in_Markets_With_Imperfect_Information) |
| **Diamond–Dybvig, 1983**, *Bank Runs, Deposit Insurance, and Liquidity* | Maturity transformation and coordination-driven runs | A universal explanation for every bank failure. [DOI](https://doi.org/10.6082/3sa4m-hvw53) |
| **Kiyotaki–Moore, 1997**, *Credit Cycles* | Collateral-price feedback | Universal mortgageability or costless asset trading. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/262072?utm_source=chatgpt.com) |
| **Caiani et al., 2016**, *Agent Based–Stock Flow Consistent Macroeconomics* | Explicit real–financial accounting and endogenous-money architecture | Industrial firms and modern monetary institutions as agrarian defaults. [Kingston University London](https://researchinnovation.kingston.ac.uk/en/publications/agent-based-stock-flow-consistent-macroeconomics-towards-a-benchm-4/) |
| **Capitalism Lab: Banking and Finance DLC** | A game-design comparison for banks, loans, bonds, and deposit products | Historical validation or evidence that its internal simulation matches TCE’s requirements. [Capitalism Lab](https://www.capitalismlab.com/banking-dlc/) |

---

## 6. Sources, datasets, and remaining uncertainty

### Priority research and data resources

| Resource | Best use for TCE | Main limitation |
| --- | --- | --- |
| **ORACC translated tablet corpora; Dercksen’s Old Assyrian research** | Contract vocabulary, denominations, calendars, merchant relationships | Surviving archives are selected; school exercises must not be mistaken for executed contracts. [Oracc](https://oracc.museum.upenn.edu/saao/P335064) |
| **Kuran & Rubin, 2018, Economic Journal**, including supplementary data | Borrower status, legal institutions, and rates in Istanbul | Court-record selection and identification limits. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12389) |
| **Drelichman & Voth’s archival sovereign-finance studies** | Contract heterogeneity, fiscal lending, restructuring and lender outcomes | A specific state and creditor network, not a universal default model. [economics.ubc.ca](https://economics.ubc.ca/profile/mauricio-drelichman/) |
| **Saito & Settsu, 2005, capital markets in traditional Japan** | Separating borrower classes, contractual rates, and receipts | Different tables describe different samples and measurement concepts. [lse.ac.uk](https://www.lse.ac.uk/Economic-History/Assets/Documents/Research/GEHN/GEHNConferences/conf6/Conf6-SaitoSettsu.pdf) |
| **Wobeser, *El crédito eclesiástico en la Nueva España, siglo XVIII*** | Institutional creditors, guarantees, loan purposes, and contract forms | Selected institutions and surviving records. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/eclesiastico.html) |
| **Keller, Shiue & Wang, 2021, AEJ: Applied Economics**, with replication materials | Spatial integration and inferred China–Britain capital costs | Storage-model identification rather than observed loan quotations. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20180299) |
| **Schmelzing, 2020, Bank of England Working Paper 845**, data for 1311–2018 | Long-run interest-rate comparisons and robustness checks | Strong advanced-economy and instrument-selection issues; not ordinary household borrowing costs. [Bank of England](https://www.bankofengland.co.uk/working-paper/2020/eight-centuries-of-global-real-interest-rates-r-g-and-the-suprasecular-decline-1311-2018) |
| **Jordà–Schularick–Taylor Macrohistory Database** | Industrial/modern macro-financial validation; annual series for 18 advanced economies from 1870 | Inappropriate as a direct pre-industrial household calibration. [MacroFinance & MacroHistory Lab](https://www.macrohistory.net/database/) |
| **BoC–BoE Sovereign Default Database** | Modern distinctions among arrears, instruments, and defaulted debt stocks | Coverage begins in 1960; cannot establish ancient or medieval default frequencies. [Bank of Canada](https://www.bankofcanada.ca/2025/10/staff-analytical-note-2025-24/?utm_source=chatgpt.com) |

### Claims that should remain explicitly qualified

**Origins are not a settled single story.** Surviving early loans establish that sophisticated obligations existed; they do not establish one universal origin in barter, temples, states, or private commerce.

**“Usury prohibition” is not a binary treatment.** Legal doctrine, contract classification, adjudication, local practice, and political protection can diverge. Historical work on European annuities and bills illustrates why reading doctrine alone misstates actual finance. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/10925/)

**Long-run rate declines are not automatic progress indicators.** Changing borrower composition, expected inflation, default risk, enforcement, investment demand, and surviving-source composition all affect comparisons. A long historical series should be a validation resource, not a scripted downward trend. [Bank of England](https://www.bankofengland.co.uk/working-paper/2020/eight-centuries-of-global-real-interest-rates-r-g-and-the-suprasecular-decline-1311-2018)

**Construction finance is particularly easy to mismeasure.** A property-backed loan can fund commerce or refinance an old debt; a property purchase is not necessarily new building. Loan-purpose evidence is required before estimating a construction-credit share. [Instituto de Investigaciones Históricas](https://historicas.unam.mx/publicaciones/publicadigital/libros/credito/ECE010.pdf)

**The thinnest parameters are often the most tempting to invent:** prehistoric interest rates, representative pre-industrial bank reserve ratios, population-wide default frequencies, and continent-wide rates for Africa or the pre-contact Americas. Use scenario ranges and report sensitivity rather than presenting false historical precision.

### Recommended v1 scope

Implement **seasonal and merchant debt, collateral and guarantees, local enforcement, transferable claims, optional deposit intermediation, and treasury restructuring** on one consistent accounting foundation. Couple these systems to actual harvests, trade journeys, construction stages, and tax collection.

That is enough for financial institutions to become useful, for credit to support specialization and building, and for booms and crises to emerge from the same rules—without scripting a banking era, a universal interest-rate decline, or periodic automatic collapse.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9287e-acd4-83e9-b32b-4514bfd02f78)
