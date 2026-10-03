# Business organization and firm dynamics for The Civilization Engine

## Main recommendation

**Represent a firm as a productive organization governed by a bundle of property rights and contracts—not as a building, and not as a step on a universal “household → guild → corporation” ladder.**

Business history contains overlapping organizational forms, not a succession in which each new form eliminates its predecessors. Partnerships and closely held businesses remained important after incorporation became available; different institutions solved different problems of financing, control, succession, and creditor protection. For TCE, those problems should generate demand for institutional change. [National Bureau of Economic Research](https://www.nber.org/papers/w13109)

Three distinctions are especially important:

* **An enterprise is not its establishment or legal identity.** A workshop can continue under a new owner; a company can own several workshops; a profitable voyage partnership can end because its agreed project is complete.
* **Closure is not necessarily bankruptcy.** Occupational switching, owner retirement, succession failure, acquisition, and planned completion must be distinguished from financial failure.
* **Modern firm-demography statistics are validation targets for comparable scenarios, not prehistoric or medieval constants.** Even modern statistical systems distinguish enterprise births from restructurings and temporary inactivity. [European Commission](https://ec.europa.eu/eurostat/web/products-eurostat-news/w/ddn-20251013-1)

The recommendations below distinguish **historical evidence**, **empirical calibration targets**, and **proposed implementation choices**.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Separate the things commonly called a “business”

Use the following conceptual entities. They need not all be separate, heavyweight simulation objects.

| Entity | Meaning in TCE | Why the distinction matters |
| --- | --- | --- |
| **Household** | People sharing some combination of consumption, labor, property, and obligations | A household may farm, weave, trade, and lend simultaneously |
| **Enterprise** | An ongoing productive organization with a controller, resources, relationships, and objectives | Its economic activity can survive changes in ownership or legal form |
| **Establishment** | A physical operating site: workshop, shop, mill, farmstead, warehouse | One enterprise can operate several sites; one building can accommodate several enterprises |
| **Legal asset pool** | Assets and obligations subject to particular ownership and creditor rules | Determines whose property creditors can seize |
| **Venture or contract** | A bounded undertaking, such as a trading voyage or construction project | Completion is not business failure |
| **Association** | A guild, merchant community, cooperative federation, or similar institution | Membership need not imply common ownership of members’ businesses |

For early agrarian worlds, an enterprise can initially be an **activity account within a household**, without separate legal personality. The kernel can track resources accurately without giving the inhabitants modern bookkeeping or corporate law.

### 1.2 Legal forms should be combinations of rights

The most useful legal distinction is not simply “limited versus unlimited liability.”

**Owner shielding** limits business creditors’ access to owners’ personal property. **Entity shielding** protects business assets from owners’ personal creditors and, in stronger forms, from premature withdrawal or liquidation by individual owners. These solve different problems and should be independently configurable. [Harvard Law School](https://hls.harvard.edu/bibliography/law-and-the-rise-of-the-firm/)

A TCE organizational charter should therefore specify:

| Legal or organizational dimension | Implementable alternatives |
| --- | --- |
| Ownership | Individual, household, partners, shareholders, members, lineage, state, religious institution |
| Control | Owner-manager, elected manager, appointed official, board, household authority |
| Residual income | Capital shares, negotiated partnership shares, labor contribution, patronage, public treasury |
| Voting | Capital-weighted, member-weighted, unanimity, delegated authority, appointment |
| Liability | Personal recourse, limited contribution, guarantees, different rules for different partners |
| Capital withdrawal | On demand, notice period, fixed term, consent required, no unilateral withdrawal |
| Transferability | Forbidden, restricted to members or kin, consent required, generally transferable |
| Succession | Inheritance, dissolution, partner buyout, election, administrative replacement |
| Entry eligibility | Open, licensed, mastership required, chartered privilege, membership restrictions |
| Enforcement | Household authority, reputation network, association arbitration, court, administrative coercion |

These are **proposed simulation dimensions**, not claims that every historical society recognized the same categories explicitly.

### 1.3 How the major forms worked—and what to simulate

| Form | Historical mechanism | TCE representation and trade-off |
| --- | --- | --- |
| **Household and kin-based enterprise** | Production, consumption, inheritance, and commercial relationships can remain closely connected. South Asian business communities and early modern Chinese firms show how kinship and community organization could support substantial enterprise. [OUP India](https://india.oup.com/product/company-of-kinsmen-oip-9780199486809/) | Shared household resources reduce some financing and monitoring costs, but illness, consumption needs, disputes, and inheritance affect the business directly. Do not treat unpaid family labor as having zero opportunity cost. |
| **General partnership** | Partners combine resources and agree on management and income rights. Partnership forms remained useful alongside corporations rather than merely preceding them. [National Bureau of Economic Research](https://www.nber.org/papers/w13109) | Negotiate contributions, profit shares, authority, liability, and exit clauses. A partner’s death or withdrawal can trigger renegotiation rather than automatic physical destruction. |
| **Commenda and related investment partnerships** | A capital supplier finances a traveling merchant; returns are shared. Ordinary investment losses and the merchant’s misconduct are different risks. Such arrangements connected financing to particular trading undertakings. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0167268109001152) | Give the venture its own inventory, participants, mandate, settlement date, and loss-allocation rules. Its successful completion closes the contract, not necessarily either participant’s continuing enterprise. |
| **Guild workshop** | The workshop and the guild are different organizations. Scholarship emphasizes both training and knowledge transmission, and restrictions on entry and competition. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/craft-guilds-apprenticeship-and-technological-change-in-preindustrial-europe/4B18A7808BACBFA40D76475FBCB665E0) | Keep the workshop independently owned. Give the guild rules for admission, training, inspection, arbitration, and political lobbying. Benefits and restrictions should operate through actual mechanisms, not a universal productivity bonus. |
| **Joint-stock company** | Pooling capital, transferable interests, delegated management, continuity, and liability protection developed in combinations. The VOC’s corporate characteristics emerged piecemeal rather than appearing fully formed in 1602. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/formative-years-of-the-modern-corporation-the-dutch-east-india-company-voc-16021623/E16FF67D27465278E442A974954741BB) | Separate investment from daily management. Capital commitment can support long projects, while agency problems, information costs, and control disputes remain possible. |
| **Cooperative** | Cooperatives organize around member ownership and democratic control; the members’ relationship to the enterprise matters. A worker cooperative is not the same institution as a consumer or producer cooperative. [ICA](https://ica.coop/en/cooperatives/cooperative-identity) | Specify who qualifies as a member and what members optimize. Worker cooperatives may value employment continuity; producer cooperatives may jointly process or market otherwise separately produced goods. Do not hard-code equal wages or an efficiency penalty. |
| **State, temple, or other institutional enterprise** | Production under public or religious authority is not exclusively modern. Inka state-directed craft production provides one non-European example. [Cambridge University Press](https://www.cambridge.org/core/journals/latin-american-antiquity/article/abs/style-technology-and-state-production-inka-pottery-manufacture-in-the-leche-valley-peru/BD6BF0ACB1657D646EB290276E959F80) | Allow objectives such as provisioning, military supply, ritual demand, or revenue. Separate the operating account from its sponsor’s treasury. Subsidies must consume real resources. |

### 1.4 Formation: opportunities, resources, and alternatives

**Proposed rule:** an agent considers founding or joining an enterprise when its expected utility exceeds the best available alternative, subject to resource and institutional feasibility.

The comparison should include expected consumption, downside risk, autonomy, work effort, household responsibilities, and the quality of available employment—not just expected accounting profit.

A practical decision sequence is:

1. Observe an opportunity through local shortages, orders, prices, an acquired skill, or an existing relationship.
2. Estimate production and sales using limited information.
3. Identify required labor, tools, premises, inputs, and working resources.
4. Search a bounded set of potential partners, creditors, employers, or customers.
5. Enter only when the required resource commitments and permissions are obtainable.

This should permit both **opportunity entrepreneurship** and **self-employment because better jobs are unavailable**. Evidence from small firms in developing countries connects business death to household circumstances and occupational choices as well as conventional competitive failure. [MIT Press Direct](https://direct.mit.edu/rest/article/101/4/645/58570/Small-Firm-Death-in-Developing-Countries)

For a pre-monetary household, the startup constraint is not “raise money.” It is “obtain the tools, food support, access rights, and labor needed until production yields something usable or exchangeable.”

### 1.5 Growth: productive advantages versus organizational limits

**Proposed rule:** firms expand when expected additional output can be sold and the expansion is both operationally and financially feasible.

Make size emerge from several opposing forces:

**Reasons to grow:** sharing expensive equipment; specialization; reliable demand; access to distant markets; accumulated skills; financing; and the ability to coordinate complementary tasks.

**Reasons to remain small:** limited local demand; transport costs; working-capital constraints; scarce skilled labor; difficulty supervising dispersed operations; partner disagreements; and household preferences.

Axtell’s firm-formation model demonstrates that individual joining, leaving, and effort decisions can generate highly unequal organization sizes. It is an existence proof for an emergent mechanism, not evidence that its particular incentives explain every historical economy. [Brookings](https://www.brookings.edu/articles/the-emergence-of-firms-in-a-population-of-agents-local-increasing-returns-unstable-nash-equilibria-and-power-law-size-distributions/)

For TCE, distinguish **technical capacity** from **organizational capacity**. A bigger building does not create customers, managers, trained workers, or payroll finance.

### 1.6 Financing and books: profits are not cash

A minimal monetary cash identity is:

\[
C\_{t+1}
=
C\_t+\text{collections}+\text{new financing}
-\text{cash operating costs}
-\text{investment}
-\text{debt service}
-\text{owner distributions}.
\]

This is an accounting identity, not a behavioral equation. Borrowing increases cash but is not profit; purchasing a durable tool reduces cash without necessarily being an immediate expense equal to its full price.

**Proposed behavioral rules:**

* Expansion requires credible financing through the relevant production-to-payment interval.
* Owner withdrawals compete with reinvestment and household consumption.
* Trade credit creates explicit claims between counterparties.
* Creditors evaluate information they can obtain, not the kernel’s perfect knowledge.
* Default losses reduce creditor assets; they do not disappear from the system without a corresponding accounting change.

For household enterprises, a poor harvest, medical crisis, or inheritance payment may force asset sales even when the craft itself remains productive. Conversely, a family may support an unprofitable activity because it provides subsistence, status, or a relative’s livelihood.

### 1.7 Networks and subcontracting: size is not only headcount

Do not equate everyone coordinated by a merchant with the merchant’s employees.

**Proposed representation:** a putting-out merchant can own materials, advance supplies to multiple household workshops, contract for processing, and collect output. The workshops retain their own labor allocation and possibly their own tools.

Likewise, a caravan can contain several independent ventures. Lydon’s research on trans-Saharan commerce finds partnerships supported by written contracts, Islamic legal institutions, and social relationships—including women’s participation through investment and intermediaries. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-global-history/article/abs/contracting-caravans-partnership-and-profit-in-nineteenth-and-early-twentiethcentury-transsaharan-trade/646B45A4638E23643007C71913C666C3)

This allows **large commercial networks made from small production units**, without inventing a giant integrated firm.

### 1.8 Succession, closure, and bankruptcy

Maintain separate exit causes:

`planned_completion`, `voluntary_exit`, `succession_failure`, `merger`, `financial_failure`, `confiscation`, `physical_destruction`.

Also maintain `dormant` as a state, not necessarily an exit.

For financial distress, distinguish:

| Condition | Meaning | Appropriate response |
| --- | --- | --- |
| **Illiquidity** | Cannot pay obligations when due | Extension, new finance, asset sale, arrears |
| **Balance-sheet insolvency** | Asset value is below liabilities under the relevant valuation | Restructuring, recapitalization, liquidation |
| **Economic nonviability** | Continued operation cannot justify its resource costs | Closure or substantial change of activity |

A suggested process is:

**Operating → arrears → negotiated workout → formal or customary proceeding → reorganization or liquidation → discharge or continuing personal obligations.**

Historical bankruptcy rules were not uniformly forgiving. English legislation began in 1542, while discharge provisions appeared in the 1705–06 reform period; the distinction between honest misfortune and fraudulent conduct mattered to reform debates. This is a specific English trajectory, not a universal chronology. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/0144036042000216377)

For implementation, make the following institutional choices explicit: creditor priority, collateral rights, personal guarantees, protected household assets, collection stays, voting on settlements, procedural costs, discharge, and sanctions for fraud.

**Do not close a seasonal business merely because it has no cash before its normal payment date. Do not preserve it indefinitely merely because its equipment has a positive book value.**

---

## 2. Parameters: evidence and usable ranges

### 2.1 How to interpret confidence

**High** means strong support for the stated population or documented case. **Moderate** means a credible study with important sampling, identification, or comparability limitations. Neither means the value transfers unchanged across eras.

The first table contains **observed outcomes**, not knobs that should directly kill, create, or resize firms.

### 2.2 Empirical targets for firm demography

| Quantity | Observed value | Population, units, and source | Confidence and application |
| --- | --- | --- | --- |
| Typical employer-firm size | Mode **1** employee; median **4** among firms with positive employment | United States, 1997; Axtell’s Census-based analysis | **High** for that cross-section; excludes nonemployers from this comparison. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/ZipfDistributionFirmSizes.RAxtell2001.pdf) |
| Firm-size distribution | Approximate complementary-cumulative exponent **1.06** | U.S. employee-size distribution, 1997 | **Moderate** as a distributional fit; not a universal exponent. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/ZipfDistributionFirmSizes.RAxtell2001.pdf) |
| Small enterprises: count versus jobs | Firms with **0–49 persons employed**: **99%** of enterprises, **48%** of employment | EU business economy, 2023 final structural statistics | **High**; sector coverage matters. [European Commission](https://ec.europa.eu/eurostat/web/products-eurostat-news/w/ddn-20251013-2) |
| Large enterprises: count versus jobs | Firms with **250+ persons employed**: **0.2%** of enterprises, **37%** of employment | Same EU dataset | **High**; demonstrates why count-weighted and employment-weighted distributions differ. [European Commission](https://ec.europa.eu/eurostat/web/products-eurostat-news/w/ddn-20251013-2) |
| Employment in small economic units | Self-employed plus micro and small enterprises account for about **70%** of employment | ILO study covering **99 countries**, including informal activity | **Moderate–high**; not directly comparable with formal enterprise registers. [International Labour Organization](https://www.ilo.org/resource/news/small-businesses-and-self-employed-provide-most-jobs-worldwide-and-asia) |
| Annual enterprise entry and exit | Births **10.5%**; deaths **8.5%** | EU, 2023; deaths were preliminary in the cited October 2025 release | **High** for that statistical definition and vintage. [European Commission](https://ec.europa.eu/eurostat/web/products-eurostat-news/w/ddn-20251013-1) |
| Establishment survival | **79.6%** at 1 year; **50.6%** at 5 years; **34.7%** at 10 years | U.S. private-sector establishments born in March 2013, followed through March 2023 | **High**; establishments, not necessarily independent firms. [Bureau of Labor Statistics](https://www.bls.gov/opub/ted/2024/34-7-percent-of-business-establishments-born-in-2013-were-still-operating-in-2023.htm) |
| Sectoral survival differences | Ten-year survival: manufacturing **43.6%**, information **29.1%** | Same U.S. cohort | **High**; use sector-specific targets rather than one universal curve. [Bureau of Labor Statistics](https://www.bls.gov/opub/ted/2024/34-7-percent-of-business-establishments-born-in-2013-were-still-operating-in-2023.htm) |
| Small-firm annual death | Mean **8.2% per year** | McKenzie and Paffhausen: 16 panels, 12 developing countries, over 14,000 firms | **Moderate**; heterogeneous samples, not a worldwide rate. [MIT Press Direct](https://direct.mit.edu/rest/article/101/4/645/58570/Small-Firm-Death-in-Developing-Countries) |
| Size differences across plant ages | Age-40 U.S. manufacturing plants are **over 7×** the size of plants aged ≤5; approximately **2×** in India and Mexico | Hsieh and Klenow, published 2014 analysis | **Moderate–high**; age profiles are not a guarantee that each individual plant grows this way. [OUP Academic](https://academic.oup.com/qje/article-abstract/129/3/1035/1817806) |
| Worker-managed relative survival | Estimated dissolution hazard **29% lower**, equivalent to a hazard ratio of **0.71** | Burdín’s Uruguayan study, after specified exclusions and controls | **Moderate**; observational, not a universal cooperative modifier. [Sage Journals](https://journals.sagepub.com/doi/pdf/10.1177/001979391406700108) |

The power-law notation above means approximately:

\[
P(N\ge n)\propto n^{-\alpha},\qquad \alpha\approx1.06.
\]

It does **not** mean the probability density has exponent 1.06; the corresponding continuous density exponent is approximately 2.06. More importantly, TCE should not force a national-scale fit onto a small town or a sector with different production technology.

### 2.3 Historical institutional parameters

| Parameter | Documented value or example | Interpretation and confidence |
| --- | --- | --- |
| Unilateral commenda profit division | Commonly **75% to capital provider, 25% to traveling merchant** | A documented arrangement, not the only possible split. Profit shares and ordinary loss-bearing are different contractual questions. **High** for the arrangement; low as a universal default. [Business Law Review](https://businesslawreview.uchicago.edu/print-archive/rise-and-fall-nexus-contracts-venetian-commenda-and-origins-corporation) |
| Long-duration ancient investment | An Old Assyrian *naruqqum* contract had a **12-year** term | Demonstrates that ancient investment need not be a single short trip. It is an attested contract, not an estimated average. **High** for the example. [Cambridge University Press](https://www.cambridge.org/core/journals/iraq/article/abs/partnerships-in-the-old-assyrian-trade/E0093436D4A2A3EF2A2F24CF59F6E51F) |
| Unequal workshop staffing | Stockholm embroiderers, 1655: **five journeymen with one master**, **one apprentice with another**, and **five other masters recorded as working alone** | A small, specialized archival sample. Unrecorded household assistance prevents interpreting these as complete labor counts. **High** for the record, low for generalization. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/14780038.2024.2304384) |
| Guild restriction on hiring | Copenhagen passementerie ordinance, 1634: normally **two journeymen per master**, with exceptions when more were available | A concrete authorable rule, not a universal medieval employment cap. **High** for this ordinance. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/14780038.2024.2304384) |

The contrast between these last two rows is important: **observed staffing and a legal staffing restriction are not the same parameter**.

### 2.4 Bankruptcy process benchmarks

The World Bank’s *Doing Business 2020* exercise, using information collected in 2019, reported the following regional averages for a **standardized hypothetical insolvency case**:

| Quantity | OECD high-income | Sub-Saharan Africa |
| --- | --- | --- |
| Time to recover debt | **1.7 years** | **2.9 years** |
| Procedural cost | **9.3% of estate value** | **22.8% of estate value** |
| Secured-creditor recovery | **70.2 cents per dollar** | **20.5 cents per dollar** |

These are expert-assessed scenario benchmarks, **not averages from representative samples of actual failed firms**. Recovery incorporates costs, outcomes, and discounting; it must not be interpreted as the percentage of physical assets destroyed. Confidence is **moderate for the standardized comparison, low for transferring it to other settings**. [World Bank](https://archive.doingbusiness.org/content/dam/doingBusiness/country/a/angola/AGO.pdf)

For pre-industrial TCE, start from the actual collection mechanism—negotiation, seizure, arbitration, or court proceedings—rather than importing either regional average.

### 2.5 Proposed starting values where evidence is thin

These are **engineering priors for sensitivity testing**, not historical estimates.

| Model parameter | Suggested initial range | How to use it |
| --- | --- | --- |
| Initial roster for an ordinary household craft activity | **1–6 active participants** | Derive availability from real household members and hired labor; do not make this a hard size ceiling |
| Working-resource target for short-cycle businesses | **1–6 months** of expected cash operating costs | Seasonal activities instead need resources through their actual production-to-payment interval |
| Routine hiring and expansion review | Every **7–30 simulated days** | Also trigger reviews after major orders, departures, or supply interruptions |
| Opportunity or partner search | **5–20 candidates** per deliberation | Sample local contacts and accessible market participants, not the whole population |
| Financial distress review | On **contractual due dates** and major shocks | Avoid arbitrary daily insolvency lotteries |
| Unexplained closure hazard | **No extra hazard initially** | Add a residual only after measuring how much closure already arises from modeled causes |

### 2.6 Convert rates correctly

For a constant annual probability \(p\_y\), the equivalent probability over a fraction \(\Delta t\) of a year is:

\[
p\_{\Delta t}=1-(1-p\_y)^{\Delta t}.
\]

This assumes a constant hazard within the interval. It is unsuitable for replacing a harvest-date debt event with independent daily shocks.

For cohort survival \(S(a)\), the conditional exit probability between ages \(a\) and \(a+1\) is:

\[
q\_a=1-\frac{S(a+1)}{S(a)}.
\]

From the cited BLS cohort, first-year exit is **20.4%**, while second-year conditional exit is approximately **13.4%**. These are calculated from the reported survival percentages. A flat annual death probability cannot reproduce that age pattern. [Bureau of Labor Statistics](https://www.bls.gov/opub/ted/2024/34-7-percent-of-business-establishments-born-in-2013-were-still-operating-in-2023.htm)

**Never add an 8% random closure rate on top of mechanisms already producing 8% closure.** That would double-count the outcome being calibrated.

---

## 3. Variation across eras and world regions

These cases are useful institutional templates, not scripted stages that TCE must pass through.

| Setting | Historically important organization | Implication for TCE |
| --- | --- | --- |
| **Foragers and forager-horticulturalists** | Research among Aché and Hiwi populations documents sharing relationships that cannot be reduced to individually priced market transactions. These are ethnographic analogues, not direct evidence for all prehistoric societies. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/499552) | Begin with households, task groups, sharing obligations, and access rights. Do not classify every hunting party as an employer firm. |
| **Early farming communities** | Archaeological interpretations of Neolithic households include cooperative domestic groups organizing production and storage. Evidence is local and does not establish a global enterprise census. [EAP IEA](https://www.eap-iea.org/index.php/eap/article/view/422) | Represent production and storage at household or cooperating-household level; allow specialization without requiring incorporation or wages. |
| **Old Assyrian commerce** | Merchants used sophisticated partnerships and investment arrangements. Interpretation of particular offices and commercial terms remains debated. [Cambridge University Press](https://www.cambridge.org/core/journals/iraq/article/abs/partnerships-in-the-old-assyrian-trade/E0093436D4A2A3EF2A2F24CF59F6E51F) | Ancient technology does not imply organizational simplicity. Permit long-distance contracting and capital pooling when the necessary social and informational institutions exist. |
| **Islamic and Mediterranean trading systems** | Capital–merchant partnerships supported trade across several regions; their history is not adequately represented as one European invention. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0167268109001152) | Let contract templates diffuse through trading relationships. Distinguish similarity of function from identical legal doctrine. |
| **Trans-Saharan Africa** | Caravan trade combined partnerships, written legal instruments, and social networks. Commercial participation could include investors who did not travel themselves. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-global-history/article/abs/contracting-caravans-partnership-and-profit-in-nineteenth-and-early-twentiethcentury-transsaharan-trade/646B45A4638E23643007C71913C666C3) | Separate investor, transporter, merchant, and guarantor roles. Reputation and written law can complement rather than replace one another. |
| **Early modern China** | Indigenous firms used organizational arrangements involving shares, enduring relationships, and customary institutions before wholesale adoption of Western company law. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0167268109000699) | Do not gate durable, multi-investor enterprise behind a specifically European corporate technology. |
| **South Asia** | Kinship, caste, and professional communities helped organize finance, authority, apprenticeship, and dispute resolution; their relationship with formal law changed over time. [OUP India](https://india.oup.com/product/company-of-kinsmen-oip-9780199486809/) | Treat community membership as a network and institutional relationship—not as one giant firm or an immutable ethnic productivity trait. |
| **Inka Andes** | State-directed specialist craft production existed outside the European corporate trajectory. Material evidence also reveals variation within state production. [Cambridge University Press](https://www.cambridge.org/core/journals/latin-american-antiquity/article/abs/style-technology-and-state-production-inka-pottery-manufacture-in-the-leche-valley-peru/BD6BF0ACB1657D646EB290276E959F80) | Institutional production can emerge without private joint-stock companies. Preserve the distinction between compulsory obligations and voluntary wage employment. |
| **European craft towns** | Masters, journeymen, apprentices, family assistance, migration, and privileges interacted. Scandinavian evidence shows substantial variation even between nearby cities. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/14780038.2024.2304384) | Local labor scarcity, training rules, and political exemptions should shape workshop growth and entry. |
| **Industrial and modern economies** | Corporation law widened organizational options without eliminating partnerships and small enterprises. Contemporary employment still includes a large small-unit and self-employed component. [National Bureau of Economic Research](https://www.nber.org/papers/w13109) | Add options rather than replace the organizational population when new law or technology appears. |

### What should drive transitions?

For TCE, use **problem-driven institutional adoption**.

A merchant repeatedly struggling to finance long trips creates demand for investment partnerships. A mill partnership threatened by withdrawals creates demand for capital commitment. A business repeatedly disrupted by inheritance creates demand for continuity arrangements. Remote owners create demand for accounting, delegated authority, and monitoring.

These are proposed causal pathways. Adoption should still depend on political support, enforcement capacity, and distributional interests. An institution beneficial to new entrants may threaten incumbents; a ruler may grant selective privileges rather than general rights.

**Legal availability, practical enforceability, and actual adoption should be three separate states.**

---

## 4. Stylized facts a correct simulation should reproduce

### 4.1 Many small firms, but disproportionately many jobs in larger firms

The EU figures show why “almost all firms are small” does not imply “almost everyone works in tiny firms.” Validate both the distribution of enterprises and the distribution of workers across enterprises. [European Commission](https://ec.europa.eu/eurostat/web/products-eurostat-news/w/ddn-20251013-2)

**Test:** changing the weighting from firms to workers should move substantial mass toward larger organizations in a developed commercial scenario.

### 4.2 Substantial turnover alongside aggregate continuity

The EU’s simultaneous births and deaths demonstrate that an economy can experience considerable organizational replacement without collapsing or constantly rebuilding all productive capacity. [European Commission](https://ec.europa.eu/eurostat/web/products-eurostat-news/w/ddn-20251013-1)

**Test:** firm identities turn over faster than buildings, tools, skills, and sometimes customer relationships. Closure should often reallocate productive resources.

### 4.3 Survival varies with age and sector

The U.S. cohort’s declining conditional exit risk and sectoral differences reject a single age-independent “business failure chance.” [Bureau of Labor Statistics](https://www.bls.gov/opub/ted/2024/34-7-percent-of-business-establishments-born-in-2013-were-still-operating-in-2023.htm)

**Test:** newcomers face uncertainty and thin buffers; established organizations can accumulate relationships and reserves, while still remaining vulnerable to technology shifts, debt, and succession.

### 4.4 Growth paths depend on the surrounding economy

The U.S.–India–Mexico plant-age comparison shows that mature organizations do not everywhere become large to the same degree. [OUP Academic](https://academic.oup.com/qje/article-abstract/129/3/1035/1817806)

**Test:** identical production recipes placed under different market access, financing, and organizational conditions should generate different age–size profiles.

### 4.5 Do not force a “missing middle”

Hsieh and Olken’s manufacturing evidence from India, Indonesia, and Mexico found that the familiar claim of a distinct missing middle can be misleading: enterprise counts can decline smoothly with size rather than forming two separated peaks. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/a2e2cd52-3dd7-4c42-ac54-451ed3830aef)

**Test:** inspect unbinned counts, log-binned distributions, and employment shares separately. A visual gap produced by binning is not an institutional mechanism.

### 4.6 Legal form does not determine performance by itself

Worker-managed firms in the Uruguayan study did not exhibit the universally elevated failure risk that some theories predict. A more recent French cohort study finds that much of an apparent survival advantage dissipates after accounting for entry resources. Together, these results argue against assigning a universal survival bonus or penalty to cooperative ownership. [Sage Journals](https://journals.sagepub.com/doi/pdf/10.1177/001979391406700108)

**Test:** outcomes should depend on financing, selection, member incentives, management, and industry—not merely a legal-form enum.

### 4.7 Rescue expectations can change behavior

The soft-budget-constraint literature emphasizes how expected rescue can weaken financial discipline. The mechanism can apply beyond state ownership; the important variable is the credibility of continued external support. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F002205103771799999)

**Test:** predictable subsidies can sustain employment and output, but must burden a sponsor and influence subsequent investment, borrowing, and closure decisions.

---

## 5. Modeling recommendation for TCE

### 5.1 Recommended entity structure

A compact conceptual schema is:

```
Enterprise
  identity and continuity history
  controller / management roles
  charter and enforcement jurisdiction
  owner or member claims
  establishments and production activities
  workers and labor contracts
  inventories and productive assets
  cash accounts and bilateral financial claims
  customer / supplier relationships
  beliefs, plans, and operating objectives
  lifecycle state and pending events
```

Keep three especially important relationships explicit:

**Ownership is not employment.** A person may work for one enterprise while owning claims in others.

**Property ownership is not physical possession.** A household workshop may hold materials belonging to a merchant.

**Enterprise age is not owner age or building age.** Succession, reorganization, relocation, and changes of legal form should preserve or alter these ages according to different rules.

### 5.2 Use people to generate firms, then firms to coordinate people

For ordinary citizens, evaluate a small set of options: remain, seek another job, start a household activity, join a partnership, or leave an existing organization.

For notable owners and managers, use the richer planner to consider investments, new establishments, major contracts, succession, restructuring, and political action.

Firm plans should reserve **real individuals’ time**. A person cannot simultaneously contribute a full working day to a farm, workshop, caravan, and guild project. Partial and seasonal participation should be normal capabilities of the labor model.

### 5.3 Keep simulation knowledge separate from institutional knowledge

The kernel should maintain exact conservation of goods and financial claims. Agents should perceive only what their information systems support.

Before writing, agreements may depend on memory and witnesses. Better record-keeping can improve monitoring and continuity without being required for the kernel to remain correct.

A useful simplification is a small ledger vocabulary—cash, inventories, productive assets, receivables, debt, owner claims, income, expenses—plus explicit contract due dates. Full historical accounting conventions are unnecessary unless bookkeeping itself is a major gameplay system.

### 5.4 Scheduling and computational cost

For 10k–50k people, the organizational layer need not become an all-to-all optimization problem.

Use daily production and consumption, staggered weekly or monthly routine reviews, and event-driven handling of departures, deaths, maturities, defaults, major orders, and harvests. Search through spatially and socially accessible candidates.

The main scale warning is conceptual: **national firm-size statistics do not directly fit a world containing only one town’s population**. Validate sizes conditional on accessible labor and customers. External demand or finance must be modeled explicitly rather than silently supplied to reproduce a national distribution.

### 5.5 Existing models and games worth borrowing from

| Model or game | Useful mechanism | What not to assume it already solves |
| --- | --- | --- |
| **Axtell, “The Emergence of Firms in a Population of Agents”** | Firms arise from individuals joining, leaving, and changing effort; organization sizes emerge rather than being prescribed. [Brookings](https://www.brookings.edu/articles/the-emergence-of-firms-in-a-population-of-agents-local-increasing-returns-unstable-nash-equilibria-and-power-law-size-distributions/) | Its production and income-sharing assumptions are not a universal theory of household firms, corporations, or cooperatives. Add assets, contracts, legal variation, and household life. |
| **Ian Wright, “The Social Architecture of Capitalism”** | Explicit movement between employer, worker, and unemployed positions; hiring and firing connect organizational change to monetary resources. [arXiv](https://arxiv.org/abs/cond-mat/0401053) | The simplified social architecture does not supply TCE’s detailed goods economy, property regimes, joint ownership, or individual life histories. |
| **Keynes-meets-Schumpeter models** | Connect heterogeneous firms, innovation, investment, demand, and macroeconomic dynamics. [Iris](https://www.iris.sssup.it/handle/11382/302310) | Treat them as references for firm behavior and aggregate feedback, not a ready-made history of person-founded organizations and changing legal institutions. |
| **The Guild 3** | Useful design reference for visible business activity connected to characters, families, and political standing. [Epic Games Store](https://store.epicgames.com/p/the-guild-3) | Its authored setting and gameplay systems are not empirical validation of historical firm demography. |
| **Victoria 3’s 2024 ownership redesign** | Separates productive buildings from ownership interests and allows different ownership arrangements. [Paradox Plaza Admin Forum](https://admin-forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-110-building-ownership-foreign-investment.1647879/page-6) | Building-level ownership and population abstractions are not equivalent to TCE’s individual owners, contractual ventures, and independent firm lifecycles. |

### 5.6 Build order and correctness checks

**First**, implement households, sole proprietorships, partnerships, productive assets, actual labor commitments, claims, and succession.

**Second**, introduce a modular charter system, guild membership, venture partnerships, institutional ownership, and meaningful creditor procedures.

**Third**, add more elaborate share transfers, multi-establishment organizations, delegated management, and restructurings.

The most valuable tests are not initially “does the distribution look like Zipf?” They are:

* Ownership changes do not duplicate or destroy physical assets.
* Debt cancellation produces corresponding creditor losses.
* No person or asset is committed twice to incompatible activities.
* Planned venture completion is not recorded as bankruptcy.
* A viable operation can survive a change of owner or capital structure.
* Insolvent firms cannot spend nonexistent resources unless someone explicitly finances them.

Then validate demographic patterns across several worlds and inspect whether the same apparent success survives changes in initial population, settlement size, and market access.

---

## 6. Sources, contested claims, and evidence limitations

### Recommended source stack

| Research need | Particularly useful sources |
| --- | --- |
| **Asset partitioning and legal form** | Hansmann, Kraakman, and Squire, **“Law and the Rise of the Firm”** (2006); Guinnane, Harris, Lamoreaux, and Rosenthal, **“Putting the Corporation in its Place”** (2007). These are the foundations for separating liability, continuity, and organizational choice. [hls.harvard.edu](https://hls.harvard.edu/bibliography/law-and-the-rise-of-the-firm/) |
| **Cross-regional business history** | Larsen on Old Assyrian partnerships; Harris on commenda and corporations; Zelin on early modern China; Roy on South Asian enterprise communities; Lydon on trans-Saharan partnerships. Together they prevent a narrowly European institutional sequence. [Cambridge University Press](https://www.cambridge.org/core/journals/iraq/article/abs/partnerships-in-the-old-assyrian-trade/E0093436D4A2A3EF2A2F24CF59F6E51F) |
| **Guild mechanisms and disagreement** | Epstein, **“Craft Guilds, Apprenticeship, and Technological Change”** (1998), alongside Ogilvie, **“The Economics of Guilds”** (2014). Read together rather than selecting only the favorable or unfavorable account. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/craft-guilds-apprenticeship-and-technological-change-in-preindustrial-europe/4B18A7808BACBFA40D76475FBCB665E0) |
| **Firm size and development** | Axtell, **“Zipf Distribution of U.S. Firm Sizes”** (2001); Hsieh and Klenow, **“The Life Cycle of Plants in India and Mexico”** (2014); Hsieh and Olken, **“The Missing ‘Missing Middle’”** (2014). [PubMed](https://pubmed.ncbi.nlm.nih.gov/11546870/) |
| **Closure and organizational survival** | McKenzie and Paffhausen, **“Small Firm Death in Developing Countries”** (2019); Burdín, **“Are Worker-Managed Firms More Likely to Fail Than Conventional Enterprises?”** (2014). [MIT Press Direct](https://direct.mit.edu/rest/article/101/4/645/58570/Small-Firm-Death-in-Developing-Countries) |
| **Reusable statistical targets** | BLS establishment-survival series; Eurostat **`bd_size`** for business demography and **`sbs_sc_ovw`** for size structure; ILO’s **Small Matters** for small and informal economic units. Preserve their different populations and definitions. [Bureau of Labor Statistics](https://www.bls.gov/opub/ted/2024/34-7-percent-of-business-establishments-born-in-2013-were-still-operating-in-2023.htm) |

### Where the evidence is thin or contested

**Premodern birth and death rates are the largest quantitative gap.** The historical material reviewed here does not justify a universal annual exit rate for early farming villages, ancient merchants, or medieval workshops. Contracts and guild records provide rich mechanisms, but they are not representative longitudinal business registers.

**Household labor is under-recorded.** A master listed without journeymen may still receive help from relatives or servants. Historical occupational counts should not automatically become complete worker rosters.

**Guild effects are disputed.** Training, quality assurance, exclusion, and political rents can coexist. The simulation should permit different balances rather than settle the historiographical debate with a permanent modifier.

**Corporate “firsts” depend on definitions.** Permanent capital, transferable claims, delegated management, and creditor protection need not appear simultaneously. A named corporation is less informative for TCE than the rights it actually possesses.

**Survival is not equivalent to social efficiency.** A protected monopoly, subsidized enterprise, or subsistence activity may survive while using resources poorly; an efficient workshop may disappear because its owner has a better alternative. Conversely, a closure can be socially costly even when it improves a creditor’s recovery.

**The strongest overall design is therefore modular and causal:** let people assemble productive organizations; let property and contract rules determine who controls resources and bears losses; and let market access, household life, technology, finance, and politics determine which organizations persist. Calibrate the resulting patterns—not an imposed historical sequence.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92888-5630-83ea-a28c-a8862a6af3c7)
