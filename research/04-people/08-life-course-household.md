# Marriage, households, and life-course decisions for TCE

## Core recommendation

**Model partnerships, reproduction, residence, property, and care as connected but separate systems.** A marriage need not create a new household; a household need not contain a married couple; children need not be born within marriage; and a person’s most important economic obligations may extend beyond their dwelling.

For TCE, the useful abstraction is a **constrained, stochastic life course**. People seek relationships, negotiate with partners and relatives, encounter institutional restrictions, obtain—or fail to obtain—resources, reproduce under biological constraints, and reorganize their households after births, deaths, migration, and conflict.

Do not implement a sequence such as “traditional extended family → industrial nuclear family → modern individualism.” Comparative research finds substantial variation within each broad economic setting, and household arrangements reflect both cultural rules and the availability of surviving relatives. Nuclear and multigenerational households are not successive universal historical stages. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2010.00346.x)

Throughout this report, **observed values are calibration targets for particular populations**, while **proposed defaults are engineering assumptions to test**, not measurements of universal human behavior.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Partner choice is a constrained search, not a universal attractiveness ranking

Research on marriage markets distinguishes personal preferences, intervention by families or other third parties, and opportunities to meet eligible partners. Similarity between spouses can therefore arise without everyone explicitly preferring similarity: residential segregation and restricted meeting networks can produce it. [Annual Reviews](https://www.annualreviews.org/doi/pdf/10.1146/annurev.soc.24.1.395)

**Recommended rule:** when someone seeks a partner, construct a limited candidate pool from actual social exposure: neighboring settlements, kin introductions, employment, markets, religious gatherings, migration, and existing relationships.

Evaluate candidates in three stages:

| Stage | What the simulation evaluates |
| --- | --- |
| Eligibility | Existing unions, institutional age rules, prohibited kin relationships, permitted union types, lineage restrictions, and religious or status restrictions. |
| Feasibility | Residence, land or employment access, marriage payments, dependent children, care obligations, and each party’s ability to leave their current household. |
| Acceptance | Affection, compatibility, expected treatment, economic prospects, social standing, family approval, and the attractiveness of remaining unmarried or waiting. |

Some restrictions should be hard legal barriers; others should impose sanctions or social costs that agents can accept or evade. **Endogamy and exogamy can operate simultaneously:** an institution may encourage marriage within a religious or status group while prohibiting marriage within a lineage.

A useful proposed scoring structure is:

\[
U\_{i,j}=
w\_A A\_{i,j}
+w\_C C\_{i,j}
+w\_R R\_{i,j}
+w\_K K\_{i,j}
-w\_D D\_{i,j},
\]

where the terms represent affinity, compatibility, expected resources, kin or alliance benefits, and anticipated costs. These weights are **model parameters**, not established cross-cultural constants. Compare this score with the agent’s outside options rather than automatically assigning the highest-scoring available partner.

Maintain separate evaluations for the prospective spouses and influential relatives. Family approval is not evidence of personal consent; conversely, personally chosen marriages can still depend on parental finance and approval.

### 1.2 “Arranged versus free choice” needs several variables

A comparative study of hunter-gatherer marriage practices reported that approximately **85% of sampled societies were coded as having arranged marriage**. This is a proportion of societies, not a proportion of all marriages or people, and contemporary ethnography is not a direct record of prehistoric behavior. Nevertheless, it strongly cautions against making forager marriage automatically unconstrained and individually chosen. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0019066&utm_source=chatgpt.com)

Represent at least four dimensions: **who introduces candidates, who can veto, who finances the union, and who controls the residence/property settlement**. These can change independently.

Indian evidence illustrates why this matters. Among women marrying in the 2000s in Allendorf and Pandian’s study, reported partner selection was approximately **6.4% self-choice, 62.6% joint parent–daughter choice, and 31.0% parents alone**. The major change was toward participation within family involvement, not wholesale replacement by independent selection. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5362258/)

Polygyny also requires separate variables for permission and realization. An institution allowing multiple wives does not imply that most men acquire them. Model spouse availability, resources, bargaining, rank, and obligations to existing spouses; do not turn legal permission into a population-wide spouse multiplier. Comparative work on monogamy and polygyny explicitly distinguishes socially permitted arrangements from their demographic consequences. [Royal Society Publishing](https://royalsocietypublishing.org/rstb/article/367/1589/657/21759/The-puzzle-of-monogamous-marriageReview-Puzzling)

### 1.3 Marriage transfers are contracts with ownership and consequences

Marriage payments differ in direction, ownership, and purpose. Anderson’s comparative review distinguishes brideprice/bridewealth and dowry and documents substantial variation in their economic significance. Treating all payments as money transferred to the husband would misrepresent these institutions. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.21.4.151)

For TCE, use a general transfer contract:

| Contract component | Required representation |
| --- | --- |
| Direction | Which person or kin group pays, and which receives. |
| Asset | Livestock, grain, land rights, currency, valuables, or labor service. |
| Ownership | Whether the recipient controls the property individually, jointly, or on behalf of a lineage. |
| Timing | Immediate payment, installments, promised inheritance, or service over time. |
| Contingencies | What happens after separation, childlessness, death, or failure to pay. |

Bridewealth can be represented as a groom-side transfer toward bride-side kin; bride service as a labor obligation; and dowry as bride-side provision whose ultimate ownership must be specified. These are distinct contracts, not different names for the same transaction.

**Implementation consequence:** marriage may be delayed by a payment requirement, financed through kin, or accompanied by debt. Payments must conserve assets and create actual claims. Never merely subtract an abstract “marriage cost” and erase the goods.

### 1.4 Households should form, expand, divide, and merge

The distinction between **nuclear, stem, and joint systems** is most useful when translated into rules governing the next generation. Comparative household research also warns that the observed composition of a household depends on mortality, age structure, and surviving kin—not only its preferred family system. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2010.00346.x)

| Household pattern | Rule to implement |
| --- | --- |
| Nuclear-oriented | Adult children commonly establish separate provisioning units when feasible. A household often contains one conjugal family, but may also contain servants, lodgers, or temporarily dependent relatives. |
| Stem | One partnered child remains with or succeeds the parental household; other children leave, receive alternative provision, or postpone household formation. |
| Joint | Multiple partnered children or siblings continue sharing an estate, labor, and provisioning, until partition, migration, conflict, or demographic change separates them. |

**Do not equate household composition with residence rules.** Store whether a new union normally resides with the husband’s side, wife’s side, either side, independently, or in another prescribed arrangement. Likewise, descent through women is not the same variable as residence with women’s kin or female political authority. Cross-cultural databases code such traits separately. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391)

Recommended household reorganization triggers include marriage, childbirth, parental disability, death of a household head, receipt of land, housing shortage, migration, and disputes over labor or property.

The same family can pass through apparently nuclear, multigenerational, joint, and divided configurations. **Generate those transitions; do not assign a permanent household-size category at birth.**

Housing constraints should have institution-dependent effects. Under a separate-household expectation they can delay marriage; under a joint-household rule they can instead increase crowding or postpone partition. This is a proposed causal mechanism to test, not a universal estimated elasticity.

### 1.5 Fertility requires both reproductive exposure and biological state

The demographic “proximate determinants” approach separates exposure to pregnancy, postpartum insusceptibility, contraception, sterility, and pregnancy termination. Its central lesson for TCE is that fertility is not simply the number of children people say they want. [Springer](https://link.springer.com/article/10.1186/s12889-017-4740-7)

Use a reproductive state machine:

\[
\text{susceptible}
\rightarrow
\text{pregnancy}
\rightarrow
\begin{cases}
\text{live birth}\\
\text{pregnancy loss}
\end{cases}
\rightarrow
\text{postpartum/recovery}
\rightarrow
\text{susceptible}.
\]

A practical monthly conception rule is:

\[
P\_i(\text{conception}) =
S\_i\,X\_i\,q\_i(a,h)\,(1-e\_i),
\]

where:

* \(S\_i\) indicates biological susceptibility;
* \(X\_i\) represents relevant reproductive exposure;
* \(q\_i(a,h)\) is conditional fecundability, varying with age, health, and persistent individual differences;
* \(e\_i\) is an effective reduction associated with the contraceptive behavior being modeled.

This is an implementation equation, not a fitted universal human model. A clinical-pregnancy event and a fertilization event also require different subsequent loss probabilities; define the event consistently.

**Marriage must not directly switch fertility on.** Store betrothal, recognized union, co-residence, reproductive exposure, and first birth separately. A historical record’s “married” category does not establish the timing of all the others.

#### Birth spacing

Model lactation-related amenorrhea and postpartum abstinence as overlapping constraints. Do not add their full durations when they occur simultaneously. The Spectrum formulation treats postpartum insusceptibility as a combined determinant; its aggregate birth-interval approximation is useful as a check, but should not replace individual states. [Springer](https://link.springer.com/article/10.1186/s12889-017-4740-7)

For an individual, the interval between live births emerges approximately from:

\[
\text{postpartum unavailable time}
+\text{waiting to conception}
+\text{gestation}
+\text{delays from losses}.
\]

Track breastfeeding, infant survival, nutritional stress, and exposure separately so that different mechanisms can produce similar observed intervals.

#### Fertility intentions

I recommend giving each person a revisable preferred range of **surviving children**, alongside preferences about timing and investment in each child. Proposed inputs include available care, immediate food costs, future labor contributions, inheritance, schooling opportunities, security in old age, and learned norms.

Intentions should influence **behavior**—partnership formation, exposure, contraception, or postponement—not directly create or cancel pregnancies. DHS research finds gaps between preferred and actual birth intervals, illustrating the importance of this distinction. [DHS Program](https://dhsprogram.com/publications/publication-as2-analytical-studies.cfm)

Distinguish **spacing** from **stopping**. Someone may want another child eventually while strongly preferring not to have one now.

Also separate three possible effects of child mortality: the end of breastfeeding, a deliberate wish to replace a child, and precautionary preferences formed from perceived mortality. Otherwise the same death can be counted repeatedly as an undifferentiated fertility boost.

### 1.6 Divorce, separation, and widowhood have different processes

Do not use one “union ends” event with identical consequences. Distinguish separation, institutionally recognized divorce, desertion, and partner death.

Divorce does not follow a universal modernization curve. Clark and Brauner-Otto’s analysis found substantial differences across sub-Saharan African countries, with stability or declines in many settings rather than a general rise. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1728-4457.2015.00086.x)

For voluntary separation, a proposed hazard should respond to conflict, treatment, economic stress, outside options, kin pressure, and the enforceability of divorce restrictions. Its consequences should include property allocation, children’s residence, unpaid transfers, support obligations, and continuing relations between kin groups.

For widowhood, trigger probate, dependent support, and possible household reorganization before evaluating remarriage. Remarriage opportunities should depend on age, children, local partner supply, resources, rights, and individual preference—not a universal waiting period.

Historical Finnish evidence found that remarriage opportunities and reproductive consequences differed by sex and age. It also shows why remarriage must be modeled alongside obligations to children from earlier unions. [Lumma Alab](https://lummaalab.utu.fi/wp-content/uploads/2024/04/s00265-013-1630-6.pdf)

### 1.7 Old age combines contribution, ownership, and increasing care needs

Low life expectancy at birth does **not** justify making adults routinely die in their thirties. Gurven and Kaplan’s comparative analysis argues for a characteristic adult human lifespan around **68–78 years** in the populations considered; this is not an estimate of life expectancy at birth or a claim that everyone survived that long. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2007.00171.x)

Give older people separate attributes for physical capacity, disability, skills, property, authority, and care obligations. Chronological age alone should not determine dependency.

Older parents may supply housing, land, childcare, or labor rather than merely consume their children’s resources. Historical US research emphasizes that intergenerational co-residence cannot automatically be interpreted as dependent parents moving in with economically independent children. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3090139/)

Kin assistance should produce benefits through actual provisioning and care. Sear and Mace’s review found that kin effects on child survival differed by relationship and context; it does not support an unconditional bonus for the presence of any grandparent. [ResearchGate](https://www.researchgate.net/publication/30529252_Who_Keeps_Children_Alive_A_Review_of_the_Effects_of_Kin_on_Child_Survival)

**Recommended care model:** calculate required assistance in hours and goods, then allocate it among co-resident kin, nearby nonresident kin, paid caregivers, and available communal or public institutions. Unmet care should affect health and welfare; provided care should consume somebody’s time.

---

## 2. Quantitative calibration parameters

**Confidence convention:** **H** means relatively strong measurement for the stated population; **M** means comparative, reconstructed, or definition-sensitive evidence. Neither implies easy transfer to another society. Ranges below are generally **not confidence intervals**.

### 2.1 Union formation, partner choice, and dissolution

| Parameter | Observed value and units | Population and interpretation | Confidence and source |
| --- | --- | --- | --- |
| Female age at first marriage | Means **23.0 years** among landowners and **25.5 years** among landless people | Four Finnish parishes; studied cohorts born 1732–1860. These are group means, not eligibility ages. | **H locally; M transferability.** Pettay et al. [Lumma Alab](https://lummaalab.utu.fi/wp-content/uploads/2024/04/s00265-013-1630-6.pdf) |
| Male age at first marriage | Means **25.3 years** among landowners and **26.5 years** among landless people | Same Finnish population. Preserve sex and socioeconomic differences rather than assigning everyone the pooled mean. | **H locally.** Pettay et al. [Lumma Alab](https://lummaalab.utu.fi/wp-content/uploads/2024/04/s00265-013-1630-6.pdf) |
| Female age at first union across African settings | Medians **below 18 years** in several countries, versus **above 25 years** in Botswana, Namibia, and South Africa | Studies summarized in 2017; underlying observation years differ. Not a single contemporaneous regional comparison. | **M.** Clark, Koski, and Smith-Greenaway. [N-IUSSP](https://www.niussp.org/fertility-and-reproduction/recent-trends-in-premarital-fertility-in-sub-saharan-africa/?print=pdf) |
| East Asian age at first marriage | Female means **28.8–29.2 years**; male means **30.5–31.8 years** | Japan, South Korea, and Taiwan, 2010. | **H for reported statistics.** Raymo et al. [ResearchGate](https://www.researchgate.net/publication/276505796_Marriage_and_Family_in_East_Asia_Continuity_and_Change) |
| Never married at age 50 | **20.2% of men; 10.7% of women** | Japan, 2010. Useful evidence that seeking a partner must not guarantee eventual marriage. | **H locally.** Raymo et al. [ResearchGate](https://www.researchgate.net/publication/276505796_Marriage_and_Family_in_East_Asia_Continuity_and_Change) |
| Partner-selection authority | **6.4% self-choice; 62.6% joint parent–daughter choice; 31.0% parents alone** | Indian women marrying in the 2000s in the study; retrospective reports. | **M–H.** Allendorf and Pandian. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5362258/) |
| Divorce within 20 years of first marriage | Estimated cumulative proportion **6.9% in Mali to 47.1% in Congo-Brazzaville** | Comparative African estimates; these are not annual hazards and should not be divided mechanically by 20. | **M.** Clark and Brauner-Otto; research summary reports endpoints. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1728-4457.2015.00086.x) |
| Remarriage prevalence | **14.5% of marriages** were a second marriage for at least one spouse | Historical Finnish sample. Denominator is marriages, not widowed individuals. | **H locally.** Pettay et al. [Lumma Alab](https://lummaalab.utu.fi/wp-content/uploads/2024/04/s00265-013-1630-6.pdf) |
| Marriage-transfer magnitude | Some estimates reach **4–6 times annual household income** | High-cost cases reviewed by Anderson—not a typical global range or recommended default. | **M, highly context-specific.** Anderson. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.21.4.151) |

For first-marriage calibration, fit an **age-specific transition schedule plus persistent nonmarriage**, not a normal distribution from which every newborn receives a predetermined wedding age.

### 2.2 Fertility, spacing, and longevity

| Parameter | Value and units | Correct use in TCE | Confidence and source |
| --- | --- | --- | --- |
| Forager fertility | Reported comparative **TFR approximately 2.6–8.0 births per woman** | An envelope across populations, not an individual family-size distribution. | **M.** Ethnographic ranges compiled in White’s ForagerNet study. [JASSS](https://www.jasss.org/20/4/9.html) |
| Forager interbirth intervals | Approximately **2.5–4.0 years**, or **30–48 months** | Fit conditional on maternal age, parity, and survival of the previous infant where data permit. | **M.** White’s comparative compilation. [JASSS](https://www.jasss.org/20/4/9.html) |
| Aggregate birth interval without lactation/postpartum abstinence | Approximately **20 months** in the proximate-determinants formulation | A population-model benchmark, **not a biological minimum** or a required individual interval. | **M; model-based approximation.** Spectrum model. [Springer](https://link.springer.com/article/10.1186/s12889-017-4740-7) |
| Last birth in historical natural-fertility populations | Median **40–41 years** | Validate reproductive aging; do not interpret this as median menopause. Study covers 58,051 women in six populations. | **H for studied histories; M portability.** Eijkemans et al. [OUP Academic](https://academic.oup.com/humrep/article/29/6/1304/625687) |
| Low modern period fertility | **0.9–1.4 births per woman** | TFR in Taiwan, South Korea, and Japan in 2010; demonstrates a low-fertility calibration regime, not completed lifetime fertility. | **H for reported series, with comparability caveats.** Raymo et al. [ResearchGate](https://www.researchgate.net/publication/276505796_Marriage_and_Family_in_East_Asia_Continuity_and_Change) |
| Characteristic adult lifespan | Approximately **68–78 years** | Check that substantial adult longevity remains possible in high-mortality populations. Do not substitute this for a full mortality schedule. | **M; comparative interpretation.** Gurven and Kaplan. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2007.00171.x) |

**TFR is a synthetic period measure:** it combines current age-specific fertility rates. It is not the average eventual number of children born to women currently alive. Maintain both period statistics and completed-cohort outcomes.

### 2.3 Proposed starting values where evidence does not identify one universal constant

The following are **engineering priors**, supplied to make an initial implementation possible. They require calibration and sensitivity analysis.

| Component | Proposed starting point | Required check |
| --- | --- | --- |
| Demographic decision interval | **1 month**; retain event dates within the month | Results should be reasonably stable when the interval is shortened. |
| Candidate search | **16–64 candidates** per active seeker per review | Check sensitivity of marriage rates, segregation, and matching inequality. |
| Young, susceptible, exposed adult conception probability | Start at **0.20 per month**; explore **0.10–0.30** | This is not a universal empirical range. Fit jointly with exposure, reproductive aging, and pregnancy-loss assumptions. |
| Postpartum unavailable duration in an initial long-spacing scenario | Start with a heterogeneous distribution centered around **18 months**; explore centers of **12–30 months** | Fit observed interbirth intervals; do not equate unavailable time with total breastfeeding duration. |
| Historical care hours and transfer prices | **No universal numerical default justified here** | Estimate from the selected setting, or expose explicitly as scenario assumptions. |

Do not compensate for an incorrect exposure model by continually adjusting biological fertility. Several combinations of exposure, spacing, and fecundability can reproduce the same TFR while generating very different lives.

---

## 3. Variation across societies and eras

### Foragers: kin involvement without one universal household system

Forager ethnography supports neither universal independent mate choice nor one fertility schedule. Marriage commonly involves kin, while the comparative fertility and spacing ranges are broad. TCE should permit flexible residence and support networks alongside particular local marriage restrictions, rather than assign a generic “primitive family” preset. Modern ethnographic cases must also be treated as historically situated societies, not unchanged samples of the distant past. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0019066)

### Early farming: population growth is better established than marriage details

The Neolithic demographic-transition literature links early farming transitions with population growth and fertility changes, but archaeological evidence cannot recover all the institutions responsible. Explanations involving sedentism, food provision, and reproductive scheduling remain difficult to separate universally. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21798934/)

Ancient DNA offers more specific local evidence. At Neolithic Gurgy in France, reconstructed pedigrees indicated patrilineal organization and female movement into the community. That is valuable support for one plausible agrarian regime—not proof that early farmers everywhere were patrilocal or followed the same marriage ages. [Nature](https://www.nature.com/articles/s41586-023-06350-8)

**TCE implication:** early farming should change food production, mobility, property, and child-rearing constraints. Let these interact with local institutions; do not directly apply a universal “farming fertility bonus.”

### Preindustrial Europe: the European Marriage Pattern is a regional configuration

The European Marriage Pattern combines relatively late and nonuniversal marriage in important parts of Europe, often associated with establishing an economically viable household. It was not uniform across Europe, and European household systems also included stem and joint arrangements. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/does-the-european-marriage-pattern-explain-economic-growth/6B90AE4652BA0B021897CCF66A9DD52A)

Its causal role in European development is contested. Voigtländer and Voth propose a pathway from post-Black-Death economic conditions through female employment to fertility restriction. Dennison and Ogilvie’s broader comparison disputes the claim that the pattern reliably explains economic growth. **Use these as competing causal hypotheses, not an automatic “late marriage produces prosperity” rule.** [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.103.6.2227)

### South Asia: family participation, kinship, and residence must remain distinct

The Indian evidence above shows continuing family participation alongside increased participation by prospective spouses. It does not support one simple transition from arranged to independently chosen marriage. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5362258/)

Research on India also emphasizes regional differences in kinship and women’s autonomy. Those differences caution against treating “Indian joint family” as a complete behavioral specification. Partner eligibility, residence, inheritance, bargaining authority, and separation rights require independent settings. [JSTOR](https://www.jstor.org/stable/1972894)

### Sub-Saharan Africa: marriage and divorce are highly heterogeneous

The contrast between early first unions in some countries and much later unions in southern Africa is already large enough to invalidate a single regional marriage-age distribution. Divorce histories also differ substantially. Delayed marriage need not reduce fertility proportionally where reproductive exposure outside marriage is important. [N-IUSSP](https://www.niussp.org/fertility-and-reproduction/recent-trends-in-premarital-fertility-in-sub-saharan-africa/?print=pdf)

Matrilineal systems provide a further reason to separate descent from the conjugal household: the relevant obligations and property relationships may extend through kin other than a spouse’s direct descendants. [National Bureau of Economic Research](https://www.nber.org/papers/w30509)

### East Asia: late marriage can coexist with persistent family obligations

Raymo and colleagues describe substantial changes in marriage and fertility alongside continuing expectations concerning marriage, gender, and family responsibilities. The resulting behavior is not well represented by a single “individualism” variable. [Annual Reviews](https://www.annualreviews.org/content/journals/10.1146/annurev-soc-073014-112428)

**TCE implication:** education and employment opportunities can change while childcare allocation, household work, marriage expectations, and inheritance rules adjust more slowly. Model the incompatibilities between institutions rather than assuming all institutions modernize together.

### Latin America: cohabitation is not merely a recent intermediate stage

Comparative research distinguishes historically established forms of consensual union from newer patterns of cohabitation. Informal unions cannot be treated simply as incomplete modern marriages or assumed to have the same associations with education, class, and childbearing everywhere. [Demographic Research](https://www.demographic-research.org/articles/volume/32/32)

**TCE implication:** legal marriage, durable partnership, shared residence, and parenthood need separate recognition and property rules.

### Industrializing and modern societies: household separation is not kinship disappearance

US historical evidence links changing intergenerational co-residence to economic and demographic conditions, including the changing value of family-based production. Separate dwellings do not establish the disappearance of support obligations. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3090139/)

For later TCE development, make wage employment, housing markets, schooling, migration, pensions, and care services alter the advantages of different living arrangements. Avoid an era switch that converts every extended household into independent couples.

---

## 4. Stylized facts a correct simulation should reproduce

Use **separate, internally consistent calibration worlds**. A world fitted to historical Finnish marriage should not simultaneously be required to reproduce southern African union formation and contemporary East Asian fertility.

| Validation target | What to test |
| --- | --- |
| Marriage is neither universally early nor universal over a lifetime | Reproduce the selected population’s first-union age distribution and share never married by later adulthood, not merely its mean wedding age. The Finnish and Japanese observations above provide contrasting targets. [Lumma Alab](https://lummaalab.utu.fi/wp-content/uploads/2024/04/s00265-013-1630-6.pdf) |
| Long birth spacing can coexist with substantial lifetime fertility | A forager-like scenario should be capable of multiyear birth intervals without requiring a modern low-child-number preference. Validate both interval distributions and total fertility. [JASSS](https://www.jasss.org/20/4/9.html) |
| Widowhood is not only an old-age event | In the Finnish study, **28% of widowed men and 29% of widowed women were under 40 at bereavement**. A high-mortality world should consequently generate lone parents and reconstituted families. [Lumma Alab](https://lummaalab.utu.fi/wp-content/uploads/2024/04/s00265-013-1630-6.pdf) |
| Household complexity depends on kin availability | A joint-family rule should not create married siblings or living grandparents who do not exist. Compare households conditional on the presence of eligible relatives. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2010.00346.x) |
| Divorce patterns need not converge monotonically | Changes in resources or institutions should not force every scenario onto a rising divorce trajectory; African evidence supplies counterexamples. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1728-4457.2015.00086.x) |
| Older people remain heterogeneous | Validate survival to older adulthood and distinguish independent, contributing, and care-dependent elders rather than treating all older people as equivalent dependents. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2007.00171.x) |

### Population replacement must account for mortality

For demographic accounting, use the net reproduction rate:

\[
NRR=\int l\_f(a)\,m\_f(a)\,da,
\]

where \(l\_f(a)\) is female survival from birth to age \(a\), and \(m\_f(a)\) is the age-specific rate of producing daughters.

A deliberately simplified illustration makes the issue clear. With half of births female and only half of daughters surviving to reproductive ages:

\[
NRR \approx F \times 0.5 \times 0.5.
\]

Replacement would then require approximately **four births per woman**, not 2.1. This is arithmetic under stated assumptions, not a historical estimate.

Also test the age structure: a mortality improvement can create a large young generation before fertility behavior changes. Matching births minus deaths for one year is insufficient to validate a centuries-long population simulation.

---

## 5. Recommended implementation for 10k–50k agents

### 5.1 Minimal persistent entities

| Entity | Essential state |
| --- | --- |
| Person | Age, health and functional capacity, reproductive state, preferences, skills, parentage, partnership history, property claims, and support obligations. |
| Union | Participants, recognized/informal status, relevant dates, residence agreement, transfer contracts, child-related rights, and dissolution history. |
| Household | Members, shared provisioning, labor allocation, care allocation, budget, and links to dwellings and estates. |
| Dwelling | Capacity, location, access, and occupants; it is not itself the economic household. |
| Estate or property ledger | Ownership, use rights, succession rules, debts, and maintenance claims. |
| Institution | Partner eligibility, approval and enforcement, residence expectations, inheritance, divorce, child affiliation, and elder-support rules. |

Keep biological parentage, social parenthood, and legal/customary guardianship separate. This permits adoption, stepfamilies, nonreproductive partnerships, and changing custody without rewriting genealogies.

Support plural unions without assuming all partners share one dwelling or budget.

### 5.2 Event architecture

Use age-, duration-, and state-dependent hazards:

\[
P(\text{event in }\Delta t)=1-\exp[-\lambda(x)\Delta t].
\]

The state vector \(x\) should include the variables relevant to that event: union duration for separation, postpartum state for conception, kin and housing availability for household formation, and functional status for care.

A practical update sequence is:

```
Process dated deaths, pregnancy outcomes, arrivals, and departures.
Resolve dependent care, guardianship, and immediate property consequences.
Update household food, labor, housing, and care constraints.
Review partnership search and selected existing relationships.
Evaluate reproductive intentions and exposure.
Schedule eligible demographic transitions.
Process household moves, mergers, partitions, and union contracts.
Record age-, parity-, sex-, and duration-specific statistics.
```

Use actual event ordering or competing-risk handling so that mutually incompatible transitions do not both occur. Previously scheduled events must be invalidated when their prerequisites change.

Daily routines should read the resulting household and care state. They do not need to repeat demographic search every rendering frame.

### 5.3 Performance and initialization

For matching, use spatial and institutional indexes plus sparse candidate sampling rather than all-pairs comparison. With 32 candidates, even reviewing all 50,000 people would involve about **1.6 million candidate evaluations per demographic review**, before eligibility pruning—not billions of pairs. This is an operation-count illustration, not a measured performance guarantee.

Preserve genealogical links after death but remove full behavioral simulation for deceased agents. Cache common kinship queries while retaining enough ancestry to enforce the selected prohibited-degree rules.

Initialize a plausible mixture of ages, unions, children, siblings, widows, and households. Starting with identical young couples creates artificial synchronized marriage, birth, inheritance, and mortality waves. A demographic burn-in is an alternative, but its duration and assumptions should be disclosed.

### 5.4 Calibration order

First fit **mortality and reproductive biology**. Next fit **union exposure and birth spacing**. Then fit **household transitions and property constraints**, and only afterward tune preference and institutional-response parameters.

Otherwise a wrong mortality schedule may be concealed by excessive fertility, or an unrealistic marriage market by artificially high conception probabilities.

Do not tune only to aggregate TFR or average household size. Inspect age-specific fertility, parity progression, birth intervals, nonmarriage, remarriage, household transitions, and care deficits.

### 5.5 Existing models and games worth borrowing from

| Reference | Useful component | Limitation for TCE |
| --- | --- | --- |
| **SOCSIM / rsocsim** | Stochastic demographic microsimulation and genealogical relationships; useful as a reference implementation and demographic test harness. | TCE must add mechanisms through which resources, preferences, and institutions alter transition rates. [MPIDR](https://mpidr.github.io/rsocsim/) |
| **White’s ForagerNet demographic model** | Links individuals, households, kinship, reproduction, and population viability. | Some model outputs diverged from ethnographic fertility and spacing; its assumptions are testable hypotheses, not universal demographic constants. [JASSS](https://www.jasss.org/20/4/9.html) |
| **RimWorld: Biotech** | A design reference for connecting family relationships, childbirth, childcare, and children’s activities to visible daily life. | A gameplay reference, not empirical calibration evidence. [Steam Store](https://store.steampowered.com/app/1826140/RimWorld__Biotech/) |

---

## 6. Data sources, uncertainty, and evidence limits

### Recommended calibration sources

| Source | Best use |
| --- | --- |
| **D-PLACE / Ethnographic Atlas**, documented by Kirby et al. | Cross-cultural marriage, descent, residence, and related institutional traits. Retain ethnographic place and date; societies are not independent or population-weighted observations. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Demographic and Health Surveys** | Birth histories, union timing, fertility preferences, postpartum behavior, contraception, and household composition. Respect survey design, weights, and censoring. [DHS Program](https://dhsprogram.com/data/) |
| **UN World Marriage Data** | Comparative marital-status distributions and marriage-timing measures. Distinguish singulate mean age at marriage from observed cohort mean wedding ages; observation years vary. [United Nations](https://www.un.org/development/desa/pd/data/world-marriage-data) |
| **IPUMS International** | Harmonized census microdata for household composition, marital status, kin relationships, and demographic structure. [IPUMS](https://www.ipums.org/projects/ipums-international) |
| **Human Fertility Database / Human Fertility Collection** | Age-specific and cohort fertility schedules, with different coverage and data-quality characteristics. [Human Fertility Database](https://www.humanfertility.org/) |
| **Historical family reconstitutions and parish-linked studies** | Longitudinal marriage, widowhood, remarriage, fertility, and kin survival. Pettay et al.’s Finnish study provides one concrete worked example. [Lumma Alab](https://lummaalab.utu.fi/wp-content/uploads/2024/04/s00265-013-1630-6.pdf) |

### Where evidence is strongest—and where not to invent precision

**Strongest:** measured fertility and union histories for well-documented populations; the importance of reproductive exposure and postpartum states; and the need to distinguish institutional rules from household snapshots.

**Less portable:** causal coefficients linking wealth, schooling, wages, housing costs, kin authority, or marriage payments to behavior. Such relationships can depend on which institution changes and which alternatives are available.

**Particularly thin for deep history:** exact early-farmer marriage ages, distributions of everyday marriage payments, reliable divorce denominators, and comparable elder-care hours. Ancient pedigrees can reveal relationships and mobility without identifying the full legal or behavioral meaning of those relationships. [Nature](https://www.nature.com/articles/s41586-023-06350-8)

Three interpretation errors deserve explicit safeguards. First, never treat marriage, co-residence, and reproductive exposure as interchangeable observations. Second, do not infer actual caregiving merely from sharing a household. Third, do not interpret a model that reproduces one aggregate statistic as identifying the true historical mechanism.

**The smallest credible TCE implementation is therefore a monthly demographic event system linked to real household budgets, sparse kin networks, explicit property rights, and care-time allocation.** Institutions should change eligibility, bargaining power, obligations, and available choices. Marriage ages, household sizes, birth spacing, and population growth should then be outcomes of those rules—not era-scripted targets.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92806-f89c-83e9-a02a-612d305e69ba)
