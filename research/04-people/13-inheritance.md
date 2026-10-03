# Inheritance systems and their consequences: a simulation-ready report for TCE

## Main recommendation

**Model inheritance as the transfer of specific rights, obligations, and eligibility—not as a single rule that divides a person’s “wealth.”** Keep three processes separate: succession to property, transmission of social status, and appointment to an office.

The most important economic distinction is **between dividing claims and dividing productive assets**. Three children can inherit equal claims while continuing to operate one farm, appointing one manager, or arranging a buyout. Conversely, an intact estate can support one prosperous successor while leaving several siblings dependent on wages or relatives. Historical evidence from Germany and China contradicts a universal rule that equal inheritance necessarily produces agricultural decline. [OUP Academic](https://academic.oup.com/ej/article-abstract/134/664/3137/7679128)

For TCE, inheritance should alter agents’ resources and expectations; land markets, production, marriage, migration, and political organization should generate the consequences.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Determine what is inheritable before determining who inherits

A household head need not personally own everything the household uses. Among the Minangkabau of West Sumatra, for example, ancestral property belongs to enduring matrilineal groups; political leadership and rights over property are related but distinct. Treating the head’s death as the liquidation of an individually owned estate would fundamentally misrepresent this arrangement. [Commission on Legal PLuralism](https://commission-on-legal-pluralism.com/system/commission-on-legal-pluralism/volumes/53-54/benda-beckmann-art.htm.html)

**Implementation rule:** Every asset needs a rights holder and a rights type. Distinguish ownership, management, use, income entitlement, and authority to sell. A person, household, lineage, religious institution, or business can hold those rights.

At death, transfer only the deceased’s transferable rights. A surviving spouse’s existing ownership share remains theirs. A lineage’s land remains lineage property. A life interest—permission to occupy a house or receive its income until death—can expire without changing the underlying ownership.

This distinction also permits inheritance systems that preserve the physical estate while changing its manager.

### 1.2 Separate eligibility, priority, and distribution

These are independent institutional choices:

| Dimension | Choices TCE should support |
| --- | --- |
| **Eligible relationships** | Children, spouse, siblings, collateral relatives, lineage members, designated outsiders, or an institution. |
| **Descent route** | Paternal, maternal, bilateral, or explicitly designated succession. |
| **Gender rules** | Equal eligibility, exclusion, preference, unequal shares, or different rules for different assets. |
| **Priority** | Eldest eligible heir, youngest, senior lineage member, selected successor, or no priority. |
| **Distribution** | Equal shares, unequal shares, a dominant successor, or continued corporate ownership. |
| **Representation** | A deceased child’s descendants inherit that branch’s claim, or surviving individuals divide the estate directly. |
| **Discretion** | Custom-bound allocation, limited testamentary freedom, or broad freedom to designate recipients. |

The Ethnographic Atlas itself distinguishes who inherits from how inheritance is distributed, and separately codes land and movable property. Its primogeniture and ultimogeniture categories mean **predominant**, not necessarily exclusive, inheritance by the senior or junior eligible member. [D-PLACE](https://d-place.org/parameters/EA074)

**Primogeniture** therefore need not mean “the eldest son receives absolutely everything.” It can mean that the eldest eligible child receives the core farm while other children receive money, livestock, marriage transfers, or training. **Ultimogeniture** reverses birth-order priority; it does not reverse the entire kinship system.

**Implementation rule:** Apply succession rules by asset class. Do not make one global `inheritance_law` field control land, household goods, titles, businesses, and offices simultaneously.

### 1.3 Allocate claims first; resolve the estate second

An equal claim can be satisfied through physical subdivision, undivided ownership shares, a buyout, sale and distribution of proceeds, or continued joint production.

The difference is not cosmetic. Under the United States’ allotment-derived system of Indigenous trust-land inheritance, heirs could receive equal **undivided interests**. A one-sixteenth interest in an 80-acre tract is not ownership of a separately identifiable five-acre parcel. Successive inheritance can create dozens or hundreds of co-owners without subdividing the physical tract. This is a colonial institutional trajectory, not a general description of precolonial Indigenous tenure. [Indian Affairs](https://www.bia.gov/bia/ots/dtlc/fractionation)

**Implementation rule:** After calculating claims, evaluate feasible settlement modes:

\[
m^\*=\arg\max\_{m\in \text{permitted modes}}
\left[
\text{expected production and household benefits}
-\text{transaction costs}
-\text{coordination costs}
-\text{uncompensated losses}
\right].
\]

This is a proposed decision structure, not an estimated historical equation. Bounded rationality and bargaining power should influence which feasible arrangement is chosen.

A useful mechanical benchmark is:

\[
A\_g=\frac{A\_0}{n^g},
\]

where every generation physically divides each holding equally among \(n\) heirs, with no purchases, marriages combining property, joint operation, or new land. A 12-hectare holding divided among three heirs becomes 4, 1.33, and 0.44 hectares per descendant holding over three divisions.

**That is an accounting experiment, not a historical prediction.** Chinese household-division inventories from approximately 1750–1910 show that equal sharing did not necessarily produce continuously shrinking family property. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/aehr.12227)

### 1.4 Inheritance changes migration incentives, not migration commands

Under an impartible regime, non-successors face a different resource constraint from the successor. But “non-successor” does not imply “emigrant.” They may work locally, remain in the household, receive compensation, acquire land, enter another household through marriage, or move.

Indeed, Huning and Wahl’s study of German equal-partition areas argues that partition **reduced migration**, keeping labor locally available for rural industry. This is an important counterexample to a simple chain from small holdings to compulsory out-migration. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/183710/)

**Implementation rule:** Let anticipated inheritance enter ordinary household and migration decisions:

\[
V(\text{destination})=
\text{expected livelihood}
+\text{access to assets and kin}
-\text{moving and establishment costs}
-\text{care obligations}
-\text{social losses}.
\]

Evaluate staying, joining another household, local wage work, acquiring land, and migration as alternatives. Primogeniture tends to disadvantage younger eligible siblings **in the core-asset allocation**; ultimogeniture instead disadvantages older ones.

Care obligations matter too. A small study of Khasi youngest daughters documents responsibilities toward parents, grandparents, and unmarried relatives that can constrain career choices. It supports modeling an inheritance-associated duty bundle, but not assigning a universal numerical “care burden” to youngest heirs. [IJIP](https://ijip.in/articles/career-aspirations-of-the-khatduh-youngest-daughter-belonging-to-khasi-matrilineal-society-studying-in-martin-luther-christian-university-shillong-meghalaya/)

### 1.5 Marriage transfers and lifetime gifts belong in the same transfer history

Dowry can function as advance inheritance, but it is not always property that the bride independently owns or controls. Marriage payments can change direction and control across institutions and over time. Bridewealth—typically a transfer from the groom’s side to the bride’s relatives—must not automatically be credited as the bride’s personal wealth. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.21.4.151)

Botticini and Siow’s *Why Dowries?* models dowry as a way of providing for daughters who leave the parental household while preserving incentives for sons who continue the family enterprise. This is a useful mechanism, not a universal explanation of dowry. Modern United States evidence also shows that lifetime gifts can compensate children with lower lifetime earnings rather than simply anticipate equal estate shares. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F000282803769206368)

**Implementation rule:** Record every significant marriage or lifetime transfer with:

`giver → recipient-owner → recipient-controller → asset/value → date → conditions`.

Add whether the transfer counts against a later inheritance. This accounting practice is often called *collation*: an earlier advance is considered when settling the estate.

For example, if three children are meant to receive equal lifetime provision, one has already received 30 units, and 60 remain, the final allocation can be 0, 30, and 30. Without a transfer ledger, TCE would either double-count the first child’s entitlement or falsely portray the final estate division as unequal treatment.

Where advances exceed eventual entitlements, explicitly define whether repayment is required, waived, or impossible. Do not silently renormalize the remaining claims.

### 1.6 Property transfers can substitute for education—or complement it

More expected inheritance does not invariably increase training. La Ferrara and Milazzo’s Ghana study found that a reform increasing paternal land inheritance for children in matrilineal groups reduced exposed boys’ schooling by approximately **0.9 years**, concentrated in landed households. The authors interpret education as having partly substituted for restricted land inheritance before the reform. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20150342)

By contrast, research on Hindu inheritance reforms in Indian states found increased daughters’ access to inherited land and educational benefits, although substantial gender bias persisted. [FAO AGRIS](https://agris.fao.org/search/en/providers/122582/records/647366372c1d629bc9800c40)

**Implementation rule:** Parents choose among transfers of assets, education, marriage assistance, and introductions. Their choices depend on expected returns, the child’s likely occupation, legal restrictions, and household preferences. Do not implement a universal positive or negative education coefficient for inheritance.

### 1.7 Separate skills, inherited status, and office succession

**Crafts and businesses.** Tools, premises, customer relationships, and admission privileges can be transferred; competence must be learned. European guild evidence includes advantages for masters’ children, but these did not everywhere eliminate training or admission requirements. Research on Genoa, for example, documents formal training requirements even for masters’ sons. [Springer](https://link.springer.com/article/10.1007/s11186-021-09444-2)

**Rule:** A child’s family can improve access to teachers, tools, credit, and customers. At death, transfer the workshop and relevant privileges—not an automatic increase in skill.

**Status.** Some statuses are assigned at birth rather than inherited when a parent dies. Virginia’s 1662 law assigning children’s enslaved or free condition through their mother is an explicit example of a coercive legal status transmitted at birth. [Encyclopedia Virginia](https://encyclopediavirginia.org/primary-documents/negro-womens-children-to-serve-according-to-the-condition-of-the-mother-1662/)

**Rule:** Implement status assignment separately from estate settlement. Status should be an institutionally enforced classification, not an intrinsic biological attribute.

**Office.** A hereditary eligibility pool need not imply automatic father-to-son appointment. Minangkabau leadership illustrates succession embedded in matrilineal organization; European monarchy research finds lower deposition risk under primogeniture, consistent with predictable succession helping coordinate elites. Neither supports a universal stability bonus for every hereditary office. [Commission on Legal PLuralism](https://commission-on-legal-pluralism.com/system/commission-on-legal-pluralism/volumes/53-54/benda-beckmann-art.htm.html)

**Rule:** Resolve an office through eligibility, nomination or ranking, ratification, and acceptance. A legally preferred successor can still face a regency, rejection, rival claimants, or an opposing coalition.

---

## 2. Parameters: what can be quantified responsibly

Use three categories: **institutional specifications**, **empirical validation targets**, and **designer-selected sensitivity ranges**. They are not interchangeable.

“High confidence” below means confidence within the stated rule or study context—not universal applicability.

### 2.1 Institutional parameters

| Parameter | Value or range | Units / conditions | Source and confidence |
| --- | --- | --- | --- |
| Equal inheritance among eligible claimants | \(s\_i=1/n\) | Fraction of the distributable asset pool; \(n\) is the number of equally entitled claims | Exact definition; **high**. Eligibility must be calculated separately. |
| Strict single-successor allocation | 1 to successor, 0 to others | Fraction of the designated **core asset**, not necessarily all lifetime transfers | Ideal-type implementation; **high as a definition**, not an empirical average. |
| Predominant eldest/youngest inheritance | Institution-specific | No defensible universal percentage | Ethnographic Atlas categories explicitly allow predominance rather than exclusivity; **high for coding**, low for inferring exact shares. [D-PLACE](https://d-place.org/parameters/EA075) |
| Son:daughter shares when inheriting together under the cited Islamic rule | 2:1 | Relative weights in their applicable distributable portion | Qur’anic inheritance rule as presented by MUIS; **high for the normative rule**. Not a rule that every male relative gets twice every female relative. [AskGov](https://ask.gov.sg/muis/questions/clv0npyhq005110qdjcm2i3l4) |
| Widow’s/widows’ collective fixed portion in the cited Islamic framework | \(1/8\) with qualifying descendants; \(1/4\) without | Fraction of the relevant net estate | Mughniyya’s comparative treatment of the schools; **high for this specification**, subject to the complete heir configuration. [Al-Islam.org](https://al-islam.org/inheritance-according-five-schools-islamic-law-muhammad-jawad-mughniyya/distribution-heritage) |
| Ordinary testamentary allocation under the cited *wasiat* framework | Up to \(1/3\) | Fraction of the estate, subject to applicable conditions | MUIS guidance; **high for the specified framework**, not all jurisdictions and exceptions. [Majlis Ugama Islam Singapura](https://www.muis.gov.sg/get-help/islamic-legacy-planning/wasiat/) |
| Concentration of claims | \(H=\sum\_i s\_i^2\); equal heirs \(H=1/n\), sole heir \(H=1\) | Dimensionless | Derived accounting measure; **exact**. This is not the population wealth Gini. |

A complete Islamic inheritance rule set needs fixed shares, residual allocation, exclusions, and school-specific treatment of difficult cases. A single “male weight = 2” parameter is insufficient. [Al-Islam.org](https://al-islam.org/inheritance-according-five-schools-islamic-law-muhammad-jawad-mughniyya/al-awl)

### 2.2 Cross-cultural persistence and inequality benchmarks

Borgerhoff Mulder and colleagues studied **21 populations and 43 wealth measures**. Their estimates provide useful conditional targets:

| Economic system in the study | Weighted intergenerational persistence, \(\beta\) | Weighted average of wealth-domain Ginis |
| --- | --- | --- |
| Hunter-gatherer | \(0.19\pm0.05\) | \(0.25\pm0.04\) |
| Horticultural | \(0.18\pm0.04\) | \(0.27\pm0.03\) |
| Pastoral | \(0.43\pm0.06\) | \(0.42\pm0.05\) |
| Agricultural | \(0.36\pm0.05\) | \(0.48\pm0.04\) |

Values are estimates ± standard errors; both columns are dimensionless. Confidence is **moderate for comparison within this selected sample**, low for treating them as universal era constants. **\(\beta\) is statistical persistence, not the fraction of wealth inherited.** The Ginis combine wealth domains using weights; they are not directly interchangeable with a conventional monetary-net-worth Gini. Use these as output comparisons only after matching definitions. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

### 2.3 Other quantitative observations

| Observation | Estimate and units | Appropriate simulation use | Confidence / source |
| --- | --- | --- | --- |
| Large marriage payments | Selected estimates of **4–6 annual household incomes per marriage** | High-transfer stress scenarios; not a universal dowry mean | **Moderate**, heterogeneous settings summarized by Anderson (2007). [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.21.4.151) |
| Ghana inheritance reform | Approximately **−0.9 years of schooling** for exposed boys | Validate that land and education transfers can substitute in the relevant institutional setting | **Moderate–high within the study**; La Ferrara & Milazzo (2017). [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20150342) |
| Non-blood succession in Japanese family firms | Approximately **10% of successions** through sons-in-law, adopted sons, or adopted sons-in-law | Demonstrate that succession can preserve a family enterprise without a biological son | **Moderate**, selected listed-firm sample; Mehrotra et al. (2013). [Harvard Law Forum on Governance](https://corpgov.law.harvard.edu/2013/04/16/adoptive-expectations-rising-sons-in-japanese-family-firms/) |
| French annual inheritance flow, 1820–1910 | Approximately **20–25% of annual national income** | Historical aggregate validation, not an individual transfer rate | **Moderate**, reconstructed series; Piketty (2011). [OUP Academic](https://academic.oup.com/qje/article-abstract/126/3/1071/1853329) |
| Same French series, around 1950 | **Below 5%** | Check that shocks and changing wealth-to-income ratios can temporarily diminish inheritance’s importance | Same source and qualification. [OUP Academic](https://academic.oup.com/qje/article-abstract/126/3/1071/1853329) |
| Same French series, around 2010 | Approximately **15%** | Check that inheritance can regain importance after industrialization | Same source; the paper’s future projections are not observations. [OUP Academic](https://academic.oup.com/qje/article-abstract/126/3/1071/1853329) |

### 2.4 Parameters that should remain explicit scenario choices

The reviewed evidence does **not** justify a universal migration probability for nonheirs, minimum viable farm size, inheritance-dispute rate, or percentage of estates successfully enforcing gender equality.

For initial experiments, use transparently chosen ranges:

| Simulation parameter | Suggested exploration | Interpretation |
| --- | --- | --- |
| Discretionary birth-order bias, \(\lambda\) | 0–4 | Designer-selected range from equal to strongly concentrated allocation |
| Compensation of excluded core-asset claimants | 0–100% of the recognized claim | Scenario range, constrained by actual resources and credit |
| Enforcement success against a contested claim | Low, intermediate, high scenarios | Prefer deriving it from local power, evidence, and institutions rather than a universal constant |
| Treatment of lifetime advances | Ignored / partially counted / fully counted | Institutional alternatives |
| Permitted settlement modes | Subdivision, co-ownership, buyout, sale, continued corporate estate | Categorical institutional choices |

A convenient discretionary-share function is

\[
s\_i=\frac{e^{-\lambda r\_i}}{\sum\_j e^{-\lambda r\_j}},
\]

where \(r\_i\) is priority rank. Itao and Kaneko use an exponential inheritance bias in their family-system model. With three heirs and \(\lambda=\ln 2\), the shares are \(4/7,2/7,1/7\); \(\lambda=0\) gives equality. Reverse ranking for youngest preference. **The proposed 0–4 exploration range is not an estimated historical distribution**, and this function should not override legally mandated fractions. [Nature](https://www.nature.com/articles/s41599-021-00919-2)

---

## 3. Variation across eras and regions

These cases are institutional examples, not stages every civilization should pass through.

| Setting | Relevant pattern | Implication for TCE |
| --- | --- | --- |
| **Foraging societies: selected African, American, and Oceanian/Indonesian cases** | Research on five populations finds transmission of embodied, relational, and material advantages—not an absence of inheritance or inequality. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/648530) | Begin with transferable tools, access relationships, knowledge, and assistance networks where appropriate. Do not assume either universal private landownership or universal propertylessness. |
| **Early farming: Neolithic Europe** | At Gurgy, France, ancient DNA reconstructed a pedigree of **64 people across seven generations**, revealing kinship and residence patterns. It does not establish exact land shares or primogeniture. [Nature](https://www.nature.com/articles/s41586-023-06350-8) | Archaeology can constrain household continuity and descent, but early-farming inheritance percentages should remain alternative hypotheses. |
| **Pre-industrial European farming** | Equal division and indivisible holdings coexisted across German localities. Their later economic consequences depended on industrial opportunities, not simply farm size. [OUP Academic](https://academic.oup.com/ej/article-abstract/134/664/3137/7679128) | Allow local customs within the same polity. Do not assign one inheritance regime to “medieval Europe.” |
| **Chinese countryside, approximately 1750–1910** | Household-division inventories document equal-sharing arrangements without an inevitable long-run contraction of household property. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/aehr.12227) | Household division should be its own event, not necessarily identical to death. Acquisitions and subsequent reorganization must remain possible. |
| **Matrilineal African and Southeast Asian arrangements** | Ghana’s studied reform changed access to paternal land; Minangkabau ancestral property and leadership followed matrilineal organization. Neither case reduces to “women receive every man’s property.” [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20150342) | Represent lineage membership, individual property, and office eligibility independently. Maternal descent is not a synonym for female management of every asset. |
| **Islamic inheritance traditions across multiple regions** | Defined shares extend claims to several categories of relatives, with gender asymmetry in some relationships and considerable complexity in complete distributions. [AskGov](https://ask.gov.sg/muis/questions/clv0npyhq005110qdjcm2i3l4) | Implement a family of explicit rule sets; distinguish normative entitlements from actual settlement and local custom. |
| **Colonial and post-allotment United States** | Equal inheritance of undivided trust-land interests generated ownership fractionation rather than necessarily creating smaller physical plots. [Indian Affairs](https://www.bia.gov/bia/ots/dtlc/fractionation) | Institutional imposition can create a new inheritance trajectory. Measure co-owner count separately from parcel size. |
| **Industrial and modern Japan** | Family firms could recruit successors through adoption and marriage, preserving organizational continuity beyond biological descent. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0304405X13000287) | Adoption and successor selection should be institutional tools, not merely demographic accidents. |
| **Industrial and modern Europe** | French inheritance flows fell sharply and later recovered; Swedish inheritances could reduce relative inequality while increasing absolute wealth gaps. [OUP Academic](https://academic.oup.com/qje/article-abstract/126/3/1071/1853329) | Industrialization must not automatically abolish inheritance’s importance. Track multiple inequality measures. |

The central historical lesson is **recombination**, not linear progression. The same society can combine equal shares in money, a single farm successor, inherited membership in an elite lineage, and a nonhereditary public office.

---

## 4. Stylized facts a correct simulation should reproduce

### Equal claims need not create equal farms—or small farms

Under identical inheritance shares, outcomes should diverge when buyouts, rental markets, corporate ownership, or land acquisition differ. The German and Chinese findings make an unconditional “equal division causes economic stagnation” mechanic untenable. [OUP Academic](https://academic.oup.com/ej/article-abstract/134/664/3137/7679128)

**Validation test:** Hold inheritance shares constant and vary market access and permitted settlement modes. Measure physical parcels, ownership shares, operational holdings, output, and landlessness separately.

### Within-family equality and society-wide equality are different

Equal division makes allocations more equal among equally eligible siblings. It does not erase the difference between inheriting from rich and poor parents. Conversely, Swedish register evidence shows that inheritances can increase absolute dispersion while reducing the Gini and top wealth shares, because poorer recipients receive larger additions relative to their previous wealth. [IFN](https://www.ifn.se/en/publications/scientific-articles-in-english/2010-2019/2018/2018-53/)

**Validation test:** Report sibling-share concentration, absolute wealth gaps, Gini, top shares, and parent–child persistence. A movement in one measure must not mechanically determine the others.

### Non-successors should have heterogeneous life courses

Some should migrate; others should remain as workers, household members, spouses in other families, or locally active entrepreneurs. Equal partition should sometimes retain labor rather than expel it, as in the German research. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/183710/)

**Validation test:** Change destination wages, land availability, compensation, and family support. Birth order should affect feasible choices without completely determining occupation or destination.

### Lifetime provision should matter as much as the final estate

A child who already received marriage property, business capital, or substantial gifts should not necessarily receive the same final estate share as a sibling. Compensatory lifetime transfers and dowry-as-advance mechanisms both support this distinction. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1002/jae.1071)

**Validation test:** Compare final bequests with total lifetime transfers. The rankings should sometimes differ.

### Stable hereditary succession should remain fallible

A clear successor can reduce coordination problems without guaranteeing acceptance or competence. The European monarchy evidence supports testing this conditional mechanism rather than assigning every primogeniture regime a fixed reduction in conflict. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/delivering-stabilityprimogeniture-and-autocratic-survival-in-european-monarchies-10001800/2399079C174599A840E5230E8827609C)

**Validation test:** Separate clarity of the rule from successor capability, elite acceptance, and rival coalition strength.

### Formal rights should not always equal effective control

The Indian reform evidence shows that improved legal inheritance rights can coexist with persistent gender bias. [FAO AGRIS](https://agris.fao.org/search/en/providers/122582/records/647366372c1d629bc9800c40)

**Validation test:** Compare recognized claims, assets actually received, and assets independently controlled. These should converge under effective enforcement but can diverge under coercion or discriminatory practice.

---

## 5. Recommended TCE representation

### 5.1 A compact, composable data model

The following is a proposed architecture, not a claim about any existing game’s implementation.

```
Person
  parents, adoptive relationships, children, spouses
  household_id, lineage_id
  institution-defined eligibility attributes
  skills, social status, relevant claims

AssetRight
  asset_id
  holder_id: person or corporate entity
  ownership_fraction
  right_kind: ownership / use / management / income
  transferability, sale restrictions, reversion conditions
  governing_rule_id

SuccessionRule
  eligible_relationships
  descent_route and representation_rule
  gender eligibility / preference / weights
  rank_rule or designated_successor
  asset-specific shares and protected claims
  testamentary_discretion
  treatment_of_lifetime_advances
  permitted_settlement_modes
  fallback_when_no_eligible_heir

TransferRecord
  giver, recipient_owner, recipient_controller
  asset or valuation, event_date
  marriage / gift / inheritance / compensation
  advance_accounting and reversion conditions

Office
  eligibility_rule
  nomination_or_ranking_rule
  ratifying_actor_or_council
  incumbent, claimants, acceptance
```

A corporate entity can represent a lineage estate, joint household, workshop partnership, or later firm. Its property persists when a manager dies.

### 5.2 Resolve inheritance through an event pipeline

**First, establish the estate boundary.** Snapshot the deceased’s rights and the relevant survivors. Preserve other people’s existing ownership and identify restrictions, debts, and protected claims.

**Second, calculate recognized claims.** Apply the appropriate asset-specific rules, valid designations, branch representation, and lifetime-advance accounting. Missing heirs require explicit fallback rules rather than disappearing property.

**Third, negotiate a settlement.** Evaluate subdivision, joint ownership, buyouts, sale, or corporate continuation. A buyout requires actual finance; compensation cannot be created from nothing.

**Fourth, enforce or contest the result.** Separate the recognized claim from possession. Disputes should depend on the size of the discrepancy, evidence, local authority, relationships, and coercive capacity.

**Fifth, update households and expectations.** Recalculate housing access, livelihoods, care duties, credit, migration options, and anticipated future transfers.

**Sixth, resolve offices independently.** The estate recipient, household manager, and political successor may be different agents.

The same machinery should also support retirement transfers, marriage settlements, adoption, and household division. Otherwise, economic behavior will unrealistically wait for deaths.

### 5.3 Let viability emerge from production

Do not set a universal rule such as “farms below two hectares fail.” TCE already needs yields, labor, tools, animals, travel, subsistence requirements, and seasonal constraints. Use those to determine whether a particular division is viable.

An indivisible workshop may be worth more intact than split into tools and premises. A farm can remain physically intact while being owned by several heirs. Conversely, joint operation can fail because of coordination costs even when division loses some productive efficiency.

These are reasons for agents to bargain—not automatic bonuses assigned to an inheritance label.

### 5.4 Model institutional change through interested agents

A useful proposed mechanism is that agents evaluate reforms using expected material gains, perceived fairness, family obligations, and political loyalty. Potential successors may defend concentrated succession; excluded relatives may favor compensation or broader eligibility. Corporate groups may resist alienation.

Allow local precedents and negotiated exceptions to accumulate into customs. Formal law can later codify, override, or incompletely enforce those customs.

Keep norms persistent enough to generate expectations. Randomly selecting a fresh inheritance rule at every death would destroy the mechanism through which inheritance shapes earlier marriage, training, and residence decisions.

### 5.5 Keep computation local and event-driven

For 10k–50k people, succession does not require daily population-wide evaluation. Index ownership and relevant kin links; process deaths and other transfer events; invalidate inheritance expectations only when something relevant changes.

Typical work should scale with the deceased’s estate and candidate kin, plus any ranking or bargaining—not with every pair of citizens. Preserve enough ancestry to resolve maternal-line and collateral succession; deleting deceased agents without genealogical summaries would break these institutions.

Use exact or carefully controlled fractional accounting. Test conservation of assets, protected ownership, claim totals, and deterministic handling of simultaneous deaths. If co-ownership becomes large, compress its representation through an estate entity while preserving claims rather than silently extinguishing small owners.

### 5.6 Existing models and games worth borrowing from

| Model or game | Useful precedent | What not to inherit uncritically |
| --- | --- | --- |
| **Itao & Kaneko (2021), “Evolution of family systems and resultant socio-economic structures”** | Separates inheritance bias from the organization and timing of joint family production. Published model and code provide a compact experimental starting point. [Nature](https://www.nature.com/articles/s41599-021-00919-2) | Its family-level agents, generational scheduling, and society-selection mechanisms should not replace TCE’s continuous individual life courses. |
| **De la Croix, Doepke & Mokyr (2018), “Clans, Guilds, and Markets”** | Models how apprenticeship institutions shape access to teachers and transmission of productive knowledge. [OUP Academic](https://academic.oup.com/qje/article-abstract/133/1/1/3950283) | Do not equate learning opportunities with automatic inheritance of competence, or treat one historical development path as inevitable. |
| **Sugarscape and documented reimplementations** | Simple death-triggered equal transfers are useful accounting and demographic baselines. [Brookings](https://www.brookings.edu/books/growing-artificial-societies/) | They are insufficient for spouses, lineage ownership, constrained property rights, marriage advances, or offices. |
| **Crusader Kings III’s documented succession design** | Makes partition, single-successor rules, and competing claims intelligible to the player. Its original developer description distinguishes several succession arrangements. [Paradox Plaza Forum](https://forum.paradoxplaza.com/forum/developer-diary/ck3-dev-diary-17-governments-vassal-management-laws-and-raiding.1352640/) | It is a design precedent, not historical calibration data. Do not make household property behave exactly like political titles or gate realistic alternatives behind fixed eras. |

For TCE’s interface, the most useful inheritance display would show **expected claims, actual control, obligations, and unresolved disputes**. “Expected to receive the farm, but owes two siblings compensation and must house a surviving parent” explains behavior far better than “primogeniture.”

---

## 6. Sources, datasets, and limits of the evidence

### Priority research assets

| Source | Best use | Main caution |
| --- | --- | --- |
| **D-PLACE / Ethnographic Atlas, EA074–EA077** | Build the categorical space of heirs and distribution rules for land and movables. EA075 currently contains 307 equal-distribution, 248 primogeniture, 22 ultimogeniture, 20 best-qualified, and 223 no-land-inheritance codes. [D-PLACE](https://d-place.org/parameters/EA075) | These are counts of coded societies, not population-weighted prevalence. The codebook explicitly warns that the distribution codes need caution. |
| **Borgerhoff Mulder et al. (2009), *Science*, “Intergenerational Wealth Transmission and the Dynamics of Inequality in Small-Scale Societies”** | Conditional comparisons of persistence across different wealth types and economies. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf) | Selected populations; inconsistent wealth objects; not a direct estimate of inheritance-law effects. |
| **La Ferrara & Milazzo (2017), *AEJ: Applied Economics*** | A policy-based test of inheritance–education substitution; the publication provides a replication package and appendix. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20150342) | Context-specific treatment and affected groups. |
| **Ogilvie, *The European Guilds* databases** | Craft access, organizational privileges, and institutional variation: **12,051 qualitative and 5,333 quantitative observations**, covering 23 present-day countries and 1095–1862. [Sheilagh Ogilvie](https://sheilaghogilvie.com/guilds-book/guilds-databases/) | Archival selection and heterogeneous observations; not a random sample of craftspeople. |
| **Kokkonen & Sundell (2014), *APSR*, “Delivering Stability”** | Office-succession comparisons: **961 monarchs in 42 states, 1000–1800**. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/delivering-stabilityprimogeniture-and-autocratic-survival-in-european-monarchies-10001800/2399079C174599A840E5230E8827609C) | European monarchies are not a universal model of political succession. |
| **Wang & van Leeuwen (2021), *Australian Economic History Review*, “Fenjiashu”** | Household-division inventories as evidence about actual transfers and property trajectories. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/aehr.12227) | Surviving documents and particular regions need not represent all Chinese households. |

D-PLACE’s displayed license is noncommercial, so its availability for research should not be confused with unrestricted redistribution of the database inside a commercial game. [D-PLACE](https://d-place.org/parameters/EA075)

### What remains contested or thin

**Prehistoric inheritance shares are especially uncertain.** Kinship, burial, and residential continuity can reveal social organization without revealing how fields, livestock, or offices were divided. Treat precise early-farming rules as hypotheses, not recovered facts. [Nature](https://www.nature.com/articles/s41586-023-06350-8)

**Inheritance institutions are endogenous.** The same production conditions and political structures that affect inequality can also influence the chosen inheritance system. German boundary studies and reforms in Ghana and India offer stronger leverage than simple cross-cultural correlations, but their effects remain tied to particular institutions and opportunities. [OUP Academic](https://academic.oup.com/ej/article-abstract/134/664/3137/7679128)

**No single inequality measure captures the result.** Legal ownership, practical control, household consumption, land concentration, and absolute wealth gaps can move differently. The Swedish evidence is a particularly clear warning against assigning inheritance one predetermined inequality effect. [IFN](https://www.ifn.se/en/publications/scientific-articles-in-english/2010-2019/2018/2018-53/)

**No defensible universal rates emerged from the reviewed sources** for nonheir migration, disputes, buyouts, or compliance with gender-equal inheritance. Those mechanisms should initially receive explicit scenario ranges and then be calibrated against matched local evidence.

### Bottom line for implementation

Build **one composable rights-and-succession system**, with separate rules for property, lifetime transfers, status, and offices. Make inheritance alter feasible choices and bargaining positions. Let fragmentation, consolidation, migration, family continuity, and inequality emerge from those changes interacting with the rest of TCE—not from preset consequences attached to “partible” or “primogeniture.”

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9281a-c734-83e9-a046-e141358bd51e)
