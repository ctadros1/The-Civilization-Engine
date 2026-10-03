# Allocation without markets: a simulation-ready design for TCE

## Executive recommendation

**Model nonmarket allocation as several distinct systems of claims, obligations, decision-making, and delivery—not as a single “communal economy” mode.** A household sharing its food, a chief provisioning followers, a temple issuing rations, and a ministry allocating steel face different problems and pursue different objectives.

Three distinctions should anchor TCE:

| Dimension | Question | Examples |
| --- | --- | --- |
| **Ownership** | Who controls the asset or stock? | Household, lineage, temple, ruler, cooperative, state |
| **Production authority** | Who decides what work happens? | Individual, household head, assembly, estate manager, ministry |
| **Distribution rule** | Who receives the output, and why? | Need, membership, reciprocal obligation, office, labor contribution, ration entitlement, purchase |

These dimensions can vary independently. Nonmarket institutions can coexist with private production and markets; centralized authority can administer geographically distributed stores; and monetary accounting can coexist with administrative allocation. Historical Mesopotamian and Soviet evidence both caution against equating the dominant institution with the entire economy. [Cambridge University Press](https://www.cambridge.org/core/journals/iraq/article/abs/origins-of-the-templeeconomy-as-seen-in-the-light-of-prehistoric-evidence/E6CED755CA475E624BFCD64BAB3B2212)

The most important implementation principle is:

> **Keep physical supplies, official records, entitlements, and actual deliveries separate.**

That separation lets TCE generate both competent redistribution and realistic failures without imposing an arbitrary “planning inefficiency” penalty.

---

## 1. Mechanisms: what these systems actually do

### 1.1 Household pooling, gifts, and reciprocity

Food sharing is not adequately explained by universal altruism or by concealed barter alone. Research distinguishes overlapping contributions from kinship, reciprocal assistance, demands from others, and reputational benefits. Their importance varies with the resource and social setting. Large, unpredictable acquisitions need not be distributed like small, reliable harvests. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/to-give-and-to-give-not-the-behavioral-ecology-of-human-food-transfers/45EBF9C9619A612ECCB77B6F81449F7D)

For TCE, separate at least three transfer rules:

| Transfer rule | Decision mechanism | What persists afterward |
| --- | --- | --- |
| **Need-based assistance** | Give when another household needs help and the donor can afford it | Relationship and memory of conduct, but not necessarily a repayment debt |
| **Balanced reciprocity** | Help with an expectation of a later return, possibly in another good or service | An obligation, tolerance for delay, and assessments of reliability |
| **Prestige-oriented giving** | Give publicly to gain recognition, attract supporters, or compete with rivals | Reputation, political commitments, and expectations of future generosity |

A particularly useful distinction comes from Maasai institutions. **Osotua** transfers respond to need and do not create repayment debt; **esile** transfers do. Treating both as loans with different interest rates would erase the defining difference. [Springer](https://link.springer.com/article/10.1007/s10745-021-00273-6)

**Implementable rules.** Households discover need through requests, visits, kinship, and observation. A donor evaluates its own buffer, the recipient’s apparent need, applicable norms, and relationship history. Assistance can be refused because the donor lacks resources, distrusts the claim, excludes the recipient, or values another obligation more highly.

Do not create a universal numerical “gift balance.” Use one only where the institution actually expects balancing. Other relationships need memories such as “helped during illness,” “refused despite visible abundance,” or “repeatedly misrepresented need.”

**Strength:** rapid, locally informed insurance without formal collateral or a tax bureaucracy. **Failure modes to generate:** exclusion, concealed resources, unequal access to helpful partners, and simultaneous distress across the network. Network simulations show that the benefits of sharing depend on how partners are selected, not simply on the number of connections. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S1090513814001627)

### 1.2 Common resources and coordinated use

Nonmarket allocation need not involve moving goods into a central warehouse. Communities may instead allocate **access, turns, and maintenance obligations**: irrigation water, pasture, woodland, fishing grounds, or shared equipment.

Ostrom’s work is especially important here: multiple decision centers and locally organized users cannot be reduced to either unrestricted markets or a single state controller. Balinese water-temple research provides a concrete example of coordinated agricultural scheduling linked to ecological interdependence. [Digital Library of the Commons](https://dlc.dlib.indiana.edu/dlc/items/484f4147-bdce-4d55-9b8d-fe61f3b2d37b)

**TCE rule:** represent a commons as a resource system plus membership, access rights, scheduling, monitoring, and dispute resolution. A water allocation grants a turn or flow allowance, not ownership of all downstream crops.

Its success should depend on whether users can observe violations, agree on workable rules, maintain infrastructure, and resolve upstream–downstream conflicts. This is a distinct allocation mechanism, not a primitive version of central planning.

### 1.3 Chiefly redistribution and competitive generosity

Sahlins’s influential comparison distinguishes leadership built through personal achievement and support networks from more institutionalized chiefly offices. These are analytical contrasts, not universal categories into which every Melanesian or Polynesian society fits. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68)

Redistribution also does **not** mean equal redistribution. Earle’s work on Hawaiian chiefdoms explicitly reexamines the relationship between redistribution and social stratification. [University of Hawaii at Hilo](https://hilo.hawaii.edu/maunakea/library/ref/70)

**TCE mechanism:** a leader acquires contributions through some combination of persuasion, customary obligations, control of productive assets, and coercion. The leader then chooses among household relief, feasts, retainers, construction, warfare, ritual expenditure, and personal consumption.

These uses have different political returns. Feeding supporters can strengthen a coalition even while nonmembers go hungry. A public feast may establish alliances rather than maximize calories retained in storage.

**Strength:** mobilization across households for projects and collective action. **Failure modes:** elite capture, excessive competitive expenditure, burdens falling on politically weak households, and breakdown when a leader loses supporters.

Keep **personal networks and office-held assets separate**. Otherwise a chief’s death either unrealistically destroys the whole institution or leaves all personal obligations unchanged.

### 1.4 Temple and palace economies

A temple or palace can combine estate production, rents, taxes, donations, dependent workers, specialist workshops, storage, and scheduled distributions. But the older picture of an entire Mesopotamian economy contained within a temple-state has been substantially challenged. Institutional archives are not a census of all economic activity. [Cambridge University Press](https://www.cambridge.org/core/journals/iraq/article/abs/origins-of-the-templeeconomy-as-seen-in-the-light-of-prehistoric-evidence/E6CED755CA475E624BFCD64BAB3B2212)

An especially useful finding comes from Johnson’s study of more than a thousand Ur III texts from Umma: the *guru₇* granary was an **administrative organization overseeing distributed storage**, not simply one enormous building. Its principal documented responsibilities included livestock and cult provisioning rather than direct provision of the entire population. [LIV Repository](https://livrepository.liverpool.ac.uk/3012217/)

**TCE mechanism:** give each institution its own estate, workers, obligations, recipients, store network, and accounting staff. A ration schedule depends on recipient category and institutional purpose; it is not automatically a nutritional minimum.

Allow several institutions to overlap in one city. A household might cultivate its own land, owe deliveries to a palace, receive temple distributions during particular service, and exchange goods independently.

**Strength:** sustained provisioning of specialists and institutions whose work does not produce immediately consumable goods. **Failure modes:** inflexible recipient lists, dependence on a particular patron or estate, administrative diversion, and records that omit substantial household activity.

### 1.5 Inca labor taxation and storage

The Inca case is better modeled as **mobilizing labor and provisioning its deployment** than as collecting a uniform percentage of every household’s output.

Mit’a obligations supplied labor for agriculture, construction, craft production, and other state activities. Roads, carriers, and accounting linked these activities. Khipu recorded quantities such as stored crops, households, and livestock: alphabetic writing is therefore not an appropriate universal prerequisite for sophisticated administration in TCE. [National Museum of the American Indian](https://americanindian.si.edu/static/inkaroad/pdf/inka-teachers-guide.pdf)

D’Altroy and Earle distinguish **staple finance**, involving bulky food and other ordinary supplies, from **wealth finance**, involving more portable valuables and specialist products. Storage and transport costs shaped what each could fund. Their work also emphasizes that state provisioning did not erase household and local economic organization. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/203249)

**TCE implementation:**

* Assign obligations to communities or registered households, then schedule actual people into service.
* Distinguish labor owed, labor summoned, labor performed, and output produced.
* Charge travel, provisioning, supervision, and replacement labor to the system.
* Use regional stores and project depots rather than routing all output through the capital.

The seasonal constraint is crucial: removing farmers during a bottleneck can damage the harvest that later finances the state.

Regional archaeological evidence also shows variation in imperial production and specialist arrangements; one standardized Inca-wide economic template is inadequate. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/abs/wealth-finance-in-the-inka-empire-evidence-from-the-calchaqui-valley-argentina/232A15FB7BAECB63F6E95A7DC39DB277)

**Avoid hardcoding** an empire-wide number of labor days, universally equal thirds of agricultural land, or a guaranteed number of years of food for everyone. The sources reviewed here do not establish those as reliable universal parameters. Colonial forced-labor schedules should not be silently substituted for preconquest arrangements.

### 1.6 Soviet-style planning: hierarchy, bargaining, and revisions

The useful model is not “a planner solves the economy.” It is a hierarchy in which political leaders set priorities, planning bodies work with aggregates, ministries translate those priorities, and enterprises negotiate supplies and production obligations.

Archival research shows continually revised and provisional plans, information withholding, and substantial discretion below the center. In the early 1930s, even Gosplan’s most comprehensive centralized balances covered only 30 raw materials, eight energy sources, and four machinery categories—not every product and transaction. [University of Warwick](https://warwick.ac.uk/fac/soc/economics/staff/mharrison/public/jel05.pdf)

A TCE planning cycle should therefore be:

**Priorities → reports and requests → provisional allocations → production and shipment → exceptions and renegotiation → revised targets.**

For a production process \(i\), actual output should remain physically constrained:

\[
q\_i=\min\left(
q\_i^{\text{capacity}},
\frac{L\_i}{a\_{Li}},
\min\_g\frac{I\_{ig}}{a\_{ig}}
\right)
\]

Here \(L\_i\) is available labor, \(I\_{ig}\) is usable input inventory, and the \(a\)'s are requirements per unit. This is a proposed engineering approximation for complementary inputs, not a complete historical production function. Alternative recipes can provide substitution.

Administrative assignment does not make an input arrive. Research on Soviet vehicle allocation documents overturned plans, distributors retaining vehicles, resisted reallocations, and informal redistribution of used vehicles. [EHS](https://ehs.org.uk/article/the-wheels-of-a-command-economy-allocating-soviet-vehicles/)

**Strengths should also be possible.** Concentrating resources on a narrow set of observable objectives can produce substantial investment and capacity growth. Allen’s interpretation emphasizes investment concentration in early Soviet growth and later investment and military-R&D choices in the slowdown; the causal balance among these explanations remains debated. A simulation should not encode inevitable stagnation from the institution’s name. [IDEAS/RePEc](https://ideas.repec.org/a/wly/canjec/v34y2001i4p859-881.html)

### 1.7 Information and incentives: specific failure mechanisms

| Mechanism | Causal process | TCE representation |
| --- | --- | --- |
| **Input overstatement** | Managers request more than technically necessary because future deliveries are uncertain | Requested quantity differs from recipe requirement; excess can become a buffer |
| **Target ratchet** | Strong performance raises subsequent expectations, discouraging disclosure of capacity | Managers anticipate the target-update rule |
| **Proxy optimization** | A rewarded measurement diverges from useful output | Track counted output separately from quality and recipient usefulness |
| **Soft budget constraint** | Expected rescue weakens the consequences of overspending or overcommitment | Agents learn rescue probabilities from previous interventions |
| **Political priority** | Influence changes who receives scarce resources | Office, coalition, and patronage affect allocation and appeals |
| **Suppressed bad news** | Reporting failure threatens the reporter | Reports can become biased even when measuring technology is good |

The ratchet mechanism has a formal foundation in Weitzman’s model: current rewards must be weighed against more demanding future targets. It is not unique to socialist systems. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/441ad189-77fb-44e7-abcd-7bbe820b102d)

Likewise, Kornai’s soft budget constraint is about **expected external support**, not the mere existence of subsidies. A single rescue does not establish the syndrome; repeated expectations do. These problems can occur in mixed economies as well. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1467-6435.1986.tb01252.x)

For proxy optimization, use explicit design examples rather than treating folklore as data. A workshop rewarded for timber volume might prefer output that meets the volume target but is poorly suited to the requested construction. Whether it does so should depend on inspection, penalties, professional norms, and alternative rewards.

### 1.8 Shortages, queues, informal exchange, and famine

Distinguish four different shortages:

| Shortage | Example |
| --- | --- |
| **Physical** | Insufficient grain exists within reach |
| **Entitlement** | Grain exists, but this household lacks recognized access |
| **Assortment or quality** | Clothing exists, but not in usable sizes or condition |
| **Timing or delivery** | An allocation exists, but the shipment is late |

Soviet distribution research documents differentiated access across social groups and the interaction between official distribution and survival strategies outside it. Equal posted prices do not imply equal access. [Routledge](https://www.routledge.com/Our-Daily-Bread-Socialist-Distribution-and-the-Art-of-Survival-in-Stalins-Russia-1927-1941/Transchel-Osokina/p/book/9781563249051)

**TCE response:** people can queue, substitute, petition, call on relationships, travel farther, produce privately, exchange claims, or trade illegally. These activities consume time and may expose them to sanctions.

Informal exchange can improve matching while also rewarding diversion. Do not classify every reciprocal favor as a bribe, or every household exchange as illegal. Legality must come from the institution’s actual rules.

A particularly dangerous combination is **rigid procurement based on expected output plus a negative harvest shock**. Meng, Qian, and Yared find that inflexible, progressive procurement was important in China’s 1959–1961 famine; more productive rural regions could suffer higher mortality because collection rules did not adjust appropriately. Their findings concern a particular institutional disaster, not all nonmarket allocation. [Columbia Business School](https://business.columbia.edu/faculty/research/institutional-causes-chinas-great-famine-1959-1961)

An illustrative TCE case—not historical data—makes the mechanism clear:

* Expected harvest: 100 units.
* Fixed procurement: 40.
* Actual harvest: 60.
* Household retention becomes 20, whereas a 40% levy on realized output leaves 36.

Further reserving seed and accounting for losses can make the difference decisive.

### 1.9 Mixed systems are central, not exceptional

A public reserve can coexist with private household production and exchange. Qing China’s civilian granaries are a major example of state food-security institutions embedded in a larger economy, with substantial regional variation in performance. [University of Michigan Press](https://press.umich.edu/Books/N/Nourish-the-People2)

Modern ration entitlements likewise need not replace ordinary markets. India’s original 2013 National Food Security Act design specified grain entitlements for eligible recipients, not complete state allocation of their diets or household consumption. [National Food Security Act](https://nfsa.gov.in/portal/Salient_Features_NFSA_AA)

**TCE should allow one person to participate in several systems simultaneously:** household pooling, community assistance, public rations, employer provisioning, and purchases. The shares should emerge from eligibility, relative reliability, transaction costs, and available goods.

---

## 2. Parameters: evidence, model settings, and proposed calibration ranges

### 2.1 Evidence-backed quantities

**Confidence:** High means strong evidence for the stated case or explicit formal rule; medium means reconstruction or limited generalizability. Confidence in a recorded quantity does not imply confidence in transferring it to another society.

| Quantity | Value and units | Context and appropriate use | Confidence / source |
| --- | --- | --- | --- |
| Adult barley allocation | **30 or 40 sila/person/month** | Girsu tablet, dated 2351–2342 BCE; one documented distribution schedule, not a universal diet | **High** for the record. British Museum 102081. [British Museum](https://www.britishmuseum.org/collection/object/W_1906-0512-2) |
| Child barley allocation | **20 sila/person/month** | Same record; retain native units until the period-specific metrology is resolved | **High** for record; conversion uncertainty. [British Museum](https://www.britishmuseum.org/collection/object/W_1906-0512-2) |
| People covered by that distribution | About **200 workers and children** | Useful scale for a small institutional ration registry | **High–medium**. [British Museum](https://www.britishmuseum.org/collection/object/W_1906-0512-2) |
| Early granary footprint | Approximately **3 × 3 m externally** | Dhra’, Jordan Valley, roughly 11,300–11,175 calibrated years before present; raised-floor storage before fully domesticated cereals | **High** architectural evidence; ownership is more interpretive. [DOI](https://doi.org/10.1073%2Fpnas.0812764106) |
| Inca storage installations | About **1,950 structures in 29 complexes** | Surveyed northern Upper Mantaro area; not an empire-wide total | **Medium–high**, archaeological reconstruction. [Scribd](https://www.scribd.com/document/929140085/DAltroy-StapleFinanceWealth-1985) |
| Storage-space estimate for that survey | Approximately **122,000 m³** | Architectural capacity estimate; **not observed inventory**, edible mass, or assured emergency coverage | **Medium**. [Scribd](https://www.scribd.com/document/929140085/DAltroy-StapleFinanceWealth-1985) |
| Reciprocity–sharing association | Weighted effect size **\(r=0.20–0.48\)** across alternative measures | Meta-analysis of **32 human and nonhuman-primate study populations**; not a percentage of food shared | **High** as published statistical result; limited as a direct behavioral parameter. [PubMed](https://pubmed.ncbi.nlm.nih.gov/23945693/) |
| Indian priority-household grain entitlement | **5 kg/person/month** | **Original 2013 statutory design**; not a claim about current implementation or full dietary needs | **High** for formal entitlement. [National Food Security Act](https://nfsa.gov.in/portal/about_nfsa) |
| Indian Antyodaya household entitlement | **35 kg/household/month** | Same historical statutory design; importantly household-based rather than per-person | **High** for formal entitlement. [National Food Security Act](https://www.nfsa.gov.in/portal/Coverage_Entitlements_NFSA_AA) |

The storage conversion must remain explicit:

\[
\text{stored mass}
=
\text{usable volume}
\times
\text{bulk density}
\times
\text{fill fraction}.
\]

Empty space, containers, mixed commodities, preservation, and access requirements prevent architectural capacity from being treated as food already available.

### 2.2 Published simulation settings—not universal ethnographic constants

| Setting | Published value | How TCE should use it |
| --- | --- | --- |
| Maasai-inspired household unit | About **6 people** |  |
| Initial herd | **70 cattle/household** |  |
| Minimum viable herd threshold | **64 cattle/household** |  |
| Mean annual herd growth setting | **3.4%/year** |  |
| Experimental network sizes | **30–100 households** |  |
| Experimental duration | **100 annual steps** |  |

These settings belong to Campennì, Cronk, and Aktipis’s model. They provide a reproducible reference scenario, not defaults for every pastoral economy. Falling below the model’s threshold represents loss of pastoral viability; it should not automatically kill TCE’s people, who may migrate, farm, seek wages, or change diets. [Springer](https://link.springer.com/article/10.1007/s10745-021-00273-6)

### 2.3 Proposed TCE sensitivity ranges

**The following are design priors and test settings, not historical estimates.** Their purpose is to expose mechanisms before local calibration.

| Parameter | Initial test values | Units / interpretation |
| --- | --- | --- |
| Active interhousehold assistance partners | **4, 8, 16** | Partners/household; separate from the full social network |
| Accessible emergency reserve target | **30, 90, 180** | Days of intended recipient consumption |
| Ordinary labor-service burden | **0, 20, 60**; stress test **120** | Working days/eligible adult/year |
| Procurement share | **0%, 10%, 25%, 40%** | Share of a precisely specified output base |
| Response to realized harvest | **0, 0.5, 1** | Weight on updated harvest information versus prior expectations |
| Reporting delay | Travel time plus **0, 3, 14** | Processing days; generate congestion from staff workload |
| Nonstrategic reporting error | **0%, 5%, 15%** | Relative error scale; model deliberate misreporting separately |
| Audit coverage | **0%, 5%, 20%** | Registry or stock records independently checked per quarter |
| Target ratchet strength | **0, 0.25, 0.75** | Fraction of above-target output incorporated into the next target |
| Operational replanning | **1, 7, 30** | Days between reviews |

Three calibration cautions matter.

First, annual labor totals are insufficient: 20 days during harvest may be more damaging than 60 off-season days. Second, reserve targets must account for the next harvest, not just an arbitrary rolling average. Third, **there is no defensible universal corruption rate, sharing percentage, or nonmarket productivity multiplier** in this evidence. Model incentives and access, then calibrate outcomes.

---

## 3. Variation across eras and regions

These are comparison cases, **not a developmental ladder**. Recent forager ethnography is not direct observation of prehistoric societies, and technologies do not uniquely determine institutions.

| Context | Allocation pattern to represent | What should differ in TCE |
| --- | --- | --- |
| **Foragers: Africa, Amazonia, Arctic and other regions** | Sharing shaped by kinship, reciprocal relationships, demands, and resource characteristics | Acquisition variance, mobility, perishability, and social visibility; no universal equal-sharing rule. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/to-give-and-to-give-not-the-behavioral-ecology-of-human-food-transfers/45EBF9C9619A612ECCB77B6F81449F7D) |
| **Early cultivation: southern Levant** | Storage before fully developed farming; later changes in the location of storage | Permit shared and household storage before bureaucratic states. Architecture alone does not prove communal ownership. [DOI](https://doi.org/10.1073%2Fpnas.0812764106) |
| **Early agrarian states: Mesopotamia** | Institutional estates, dependents, rations, and other household economic activity | Multiple overlapping economic organizations, not one city-wide temple inventory. [Cambridge University Press](https://www.cambridge.org/core/journals/iraq/article/abs/origins-of-the-templeeconomy-as-seen-in-the-light-of-prehistoric-evidence/E6CED755CA475E624BFCD64BAB3B2212) |
| **Pacific societies** | Competitive personal leadership and institutionalized chiefly authority, with substantial variation | Distinguish generosity-based followership from office-based rights to contributions. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68) |
| **Andean imperial societies** | Labor obligations, local provisioning, regional stores, specialist production | Community-mediated mobilization and geographically distributed logistics. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/203249) |
| **East African pastoralists** | Need-based livestock assistance alongside debt-based arrangements | Productive animals, ecological covariance, and distinct transfer norms. [Springer](https://link.springer.com/article/10.1007/s10745-021-00273-6) |
| **Bali and comparable irrigation settings** | Coordinated use of interconnected resources | Allocate schedules and access; collective coordination need not centralize all crops. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.1.02a00050) |
| **Qing China** | Public granary institutions with regional administrative differences | Local reserve incentives and central relief can interact, sometimes adversely. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/local-granaries-and-central-government-disaster-relief-moral-hazard-and-intergovernmental-finance-in-eighteenth-and-nineteenthcentury-china/80F2E2CEBBEA518A60D40E2116312898) |
| **Industrial Soviet-type systems** | Administrative priorities and enterprise allocations alongside monetary consumption and informal adjustment | More complex input networks, organizational bargaining, and differentiated access. [EHS](https://ehs.org.uk/article/the-wheels-of-a-command-economy-allocating-soviet-vehicles/) |
| **Modern mixed systems** | Public entitlements within economies containing markets and household provision | Eligibility, claims, delivery reliability, and appeals remain distinct from purchasing power. [National Food Security Act](https://nfsa.gov.in/portal/Salient_Features_NFSA_AA) |

For TCE’s technology graph, useful enabling capabilities include preservation, transport, counting, durable records, administrative training, inspection, and communications. They should improve particular tasks—not unlock a universally superior economic regime.

---

## 4. Stylized facts a correct simulation should reproduce

These are **conditional patterns**, not outcomes every world must exhibit.

| Pattern | Empirical or theoretical anchor | Simulation check |
| --- | --- | --- |
| Sharing responds to relationships but is not explained by reciprocity alone | Meta-analytic reciprocity effects coexist with kinship and tolerated demands. [PubMed](https://pubmed.ncbi.nlm.nih.gov/23945693/) | Different relationship and resource settings produce different transfer networks |
| Extensive storage does not imply a state feeds everyone | Umma’s granary administration had specific institutional recipients. [LIV Repository](https://livrepository.liverpool.ac.uk/3012217/) | Large public stocks can coexist with household self-provisioning and excluded people |
| Rations can be categorical rather than individually optimized | Girsu’s **20, 30, and 40 sila/month** categories. [British Museum](https://www.britishmuseum.org/collection/object/W_1906-0512-2) | Age/status categories affect entitlements without guaranteeing adequacy |
| Administrative systems bargain and adapt | Soviet vehicle allocations were revised and informally reallocated. [EHS](https://ehs.org.uk/article/the-wheels-of-a-command-economy-allocating-soviet-vehicles/) | Actual delivery networks diverge from initial orders |
| Expected rescue can reduce local precaution | Qing provinces receiving more frequent central relief held systematically lower granary stocks in Shiue’s analysis. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/local-granaries-and-central-government-disaster-relief-moral-hazard-and-intergovernmental-finance-in-eighteenth-and-nineteenthcentury-china/80F2E2CEBBEA518A60D40E2116312898) | Relief expectations can change reserve behavior; the result should depend on rules |
| Procurement can make productive places unusually vulnerable | The China famine study finds the normally expected production–mortality relationship reversed during famine years. [Columbia Business School](https://business.columbia.edu/faculty/research/institutional-causes-chinas-great-famine-1959-1961) | Overcommitted high-output districts can suffer disproportionately after shocks |
| Institutional success is objective-dependent | Distribution can support political control as well as consumption. [Routledge](https://www.routledge.com/Our-Daily-Bread-Socialist-Distribution-and-the-Art-of-Survival-in-Stalins-Russia-1927-1941/Transchel-Osokina/p/book/9781563249051) | Measure coalition survival, project completion, and household welfare separately |

A useful mathematical benchmark concerns risk pooling. For \(n\) equally variable households with pairwise income correlation \(\rho\),

\[
\operatorname{Var}(\bar Y)
=
\sigma^2\left[\rho+\frac{1-\rho}{n}\right].
\]

With 20 households, independent shocks reduce the variance of their pooled average to **5%** of individual variance. At \(\rho=0.8\), it remains **81%**. These are mathematical illustrations, not measured historical benefits. They show why adding partners in the same drought zone is different from connecting to another ecological region. Delivery costs and unequal transfers reduce achievable benefits further.

---

## 5. Recommended representation for TCE

### 5.1 Use common physical systems with interchangeable allocation rules

All regimes should use the same production, inventory, transport, consumption, and labor systems. Allocation institutions decide **who may request, who owes, who approves, and who receives priority**.

A compact institutional specification could contain:

| Component | Required state |
| --- | --- |
| **Membership** | Eligible households, individuals, offices, dependents, outsiders |
| **Authority** | Decision rights, appeal routes, succession, delegation |
| **Revenue obligations** | Labor service, commodity deliveries, rents, donations, money |
| **Allocation policy** | Need-based, equal share, status schedule, contribution-based, priority tiers, bargaining |
| **Assets** | Estates, workshops, warehouses, transport equipment, financial accounts |
| **Information** | Registers, reports, timestamps, measured quantities, confidence, provenance |
| **Enforcement** | Monitors, sanctions, exemptions, ability to resist |
| **Objectives** | Relief, reserves, public works, ritual, military supply, elite support |
| **Revision rules** | Reassessment frequency, emergency powers, target updates |

Do not encode these as one enum such as `TempleEconomy` or `CommandEconomy`. Those can be named configurations assembled from the components.

### 5.2 Keep four layers of reality

For each relevant stock or obligation:

**Physical state:** what goods exist, where they are, and their condition.

**Recorded state:** what a particular official or institution believes exists.

**Claim state:** what someone is entitled or obligated to receive or provide.

**Execution state:** what is reserved, collected, loaded, traveling, delivered, rejected, or consumed.

A warehouse may physically hold 100 sacks, record 140, owe 160, and have only 20 immediately reachable by available carts. None of these numbers should overwrite the others.

Use immutable transfer events to preserve the causal trail. A useful event records origin, destination, good, quantity, authority, justification, dispatch time, arrival time, and any loss or rejection.

### 5.3 Individual needs; household provisioning

Keep individual physiology and preferences, but normally aggregate acquisition and storage at the household level. This avoids making every person independently negotiate every meal.

For household \(h\) and good \(g\), estimate a provisioning deficit:

\[
D\_{hg}
=
\max\left(0,\;
N\_{hg}(H)-S\_{hg}-A\_{hg}(H)
\right),
\]

where \(N(H)\) is expected need over a horizon, \(S\) is usable stock, and \(A(H)\) is expected arrival within that horizon.

Expected arrivals should be discounted by known reliability. Food adequacy should use the nutrition system, not assume every grain unit substitutes perfectly for every other food.

Then try permitted channels: household stock, assistance, institutional claim, exchange, substitute, or relocation. Channel selection can be adaptive rather than a fixed global order.

### 5.4 Implement need-based and reciprocal transfers differently

A simple need-based candidate quantity is:

\[
x\_{d\rightarrow r,g}
=
\min\left(
D\_{rg},
\max(0,S\_{dg}-B\_{dg}),
C\_{drg}
\right),
\]

where \(B\) is the donor’s protected buffer and \(C\) is feasible transfer capacity.

This only establishes what is feasible. Consent, norms, information, and eligibility determine whether the transfer occurs.

For balanced reciprocity, add an obligation record with the expected return, tolerated delay, and settlement rules. For need-based assistance, update social memory **without automatically creating that debt**.

Let relationships weaken or strengthen through observed conduct, but do not make every refusal equally discrediting. Refusing because one’s children are hungry differs from refusing despite visible abundance.

### 5.5 Make ration allocation deterministic and inspectable

A practical default is **priority tiers followed by capped proportional allocation**.

First identify eligible claims and the institution’s available supply. Allocate to the highest policy tier. Within that tier, distribute proportionally to recognized claims or policy weights, never exceeding each claim. Reallocate leftovers among unsatisfied recipients. Continue only while supply remains.

Alternative institutions can use queues, lotteries, household-size schedules, office privileges, or contribution points.

Crucially, an allocation creates a **reservation or delivery order**, not instant household inventory. A queue can therefore form even after authorization because throughput is inadequate.

### 5.6 Let information problems emerge from work and incentives

Use an observation model such as:

\[
\hat S(t)=S(t-\ell)+\epsilon+b,
\]

where \(\ell\) is delay, \(\epsilon\) is measurement error, and \(b\) is strategic distortion.

Generate delay from distance and clerical workload. Better records can reduce accidental error without eliminating strategic distortion.

Give managers recognizable alternatives: report honestly, conceal a buffer, exaggerate need, delay a report, seek an exemption, or appeal to a patron. Their choices should depend on expected sanctions, norms, personal risk, and previous outcomes.

An inspector needs time, access, and independence. Otherwise “increase auditing” becomes a costless button that fixes every problem.

### 5.7 Make labor taxation a scheduling problem

Represent a labor obligation as a claim on **eligible time**, not a quantity of free production.

A service order needs a person or household obligation, destination, start window, duration, required skills, provisioning responsibility, and authority to defer or substitute.

The originating household loses work time and may have to reorganize care and production. The receiving project still needs supervisors, materials, equipment, food, and accommodation.

This creates plausible decisions: postpone a road to protect harvest labor, hire substitutes where allowed, exempt a specialist, or coerce service and accept the economic and political consequences.

### 5.8 Allow institutional change without scripted stages

Institutions should propose rule changes through TCE’s political system when actors perceive opportunities or failures.

Examples include households supporting a shared reserve after repeated losses; project organizers seeking recurring contributions; officials attempting to turn temporary control into permanent office; recipients demanding reliable rights; and managers lobbying for local discretion.

These are proposed generative mechanisms. Evaluate them through coalition support, enforcement capacity, observed performance, and distributional consequences—not a society-wide optimizer selecting the “best economy.”

A well-run reserve may strengthen legitimacy. A reserve that excludes a large coalition may instead strengthen opposition even when its accounts balance.

### 5.9 Performance and simplification

For 10k–50k people, retain individual identity but avoid institutional all-pairs reasoning.

Use sparse relationship networks, household-level requests, warehouse catchments, cached recipient categories, and event-driven obligations. Review institutional plans weekly, monthly, or seasonally while executing work and deliveries on the normal daily simulation.

Aggregate commodities into lots, but preserve destination, ownership, quantity, condition, and travel. Unobserved transport can be simulated coarsely without becoming teleportation.

The main simplifications I recommend are:

**Simplify administrative detail, not physical conservation.** A compact report can stand for many historical documents.

**Simplify the catalog, not all quality differences.** Preserve attributes that change usefulness, such as food condition, tool durability, or clothing fit.

**Use an omniscient optimizer only as a test benchmark.** It can show whether a feasible allocation existed. Actual institutions should act on their own information and authority.

These are architectural recommendations, not measured performance claims for the proposed Rust kernel.

### 5.10 Existing models and games worth adapting

| Reference | What to borrow | What not to assume |
| --- | --- | --- |
| **Osotua models**, beginning with Aktipis, Cronk, and de Aguiar | Minimal rules for asking when in need and helping when able | One pastoral livelihood model establishes universal gift-economy behavior. [Arizona State University](https://asu.elsevierpure.com/en/publications/risk-pooling-and-herd-survival-an-agent-based-model-of-a-maasai-g/) |
| **Village Ecodynamics Project extensions** | Household needs, labor allocation, generalized and balanced reciprocity, and barter in the same model | Its archaeological setting is interchangeable with every agrarian society. [JASSS](https://www.jasss.org/16/4/4.html) |
| **Lansing–Kremer water-temple model** | Coordinating interdependent resource schedules through local institutions | Coordination requires central ownership of all production. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.1.02a00050) |
| **Workers & Resources: Soviet Republic** | Material chains, storage, construction inputs, transport, and external trade | Player control reproduces bureaucratic incentives or limited knowledge. The latter is a TCE extension. [Soviet Republic](https://www.sovietrepublic.net/) |

For testing, use a small set of paired scenarios: local versus correlated harvest shocks; honest versus delayed reports; adaptive versus fixed procurement; protected versus overloaded harvest labor; and permitted versus prohibited side exchange.

Measure **unmet-need person-days, actual ration fulfillment, queue time, spoilage, transport effort, project completion, consumption inequality, and report–reality gaps**. GDP or warehouse totals alone cannot distinguish effective provisioning from a well-stocked institution surrounded by hungry people.

---

## 6. Sources, datasets, and limits of the evidence

### Most useful scholarly foundations

**Sharing and reciprocity:** Gurven’s *To Give and to Give Not* provides the broad behavioral framework; Jaeggi and Gurven supply comparative statistical evidence. The Maasai simulation literature supplies unusually explicit alternative transfer rules. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/to-give-and-to-give-not-the-behavioral-ecology-of-human-food-transfers/45EBF9C9619A612ECCB77B6F81449F7D)

**Redistribution and institutional storage:** Sahlins and Earle are useful for distinguishing leadership, redistribution, and stratification. D’Altroy and Earle connect political finance to the physical properties of goods. Johnson’s Umma study guards against treating an administrative institution as a single warehouse or the whole economy. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68)

**Planning and incentives:** Gregory and Harrison’s *Allocation under Dictatorship*, Lazarev and Gregory’s vehicle study, Weitzman’s ratchet model, and Kornai’s soft-budget work supply complementary organizational mechanisms. Osokina’s *Our Daily Bread* addresses distribution and lived access. [AEA Publications](https://pubs.aeaweb.org/doi/10.1257/002205105774431225)

**Food-security institutions:** Will and Wong’s *Nourish the People*, Shiue’s granary analysis, and Meng–Qian–Yared’s famine study show why reserves, local incentives, information, and procurement rules must be modeled together. [University of Michigan Press](https://press.umich.edu/Books/N/Nourish-the-People2)

### Useful data sources

| Source | Best use | Main caution |
| --- | --- | --- |
| **CDLI and ORACC** | Primary administrative texts, commodity terminology, individual ration and delivery records | Surviving archives overrepresent institutions and particular places; document counts are not economic shares. [CDLI](https://cdli.earth/) |
| **D-PLACE** | Comparative subsistence, settlement, political organization, and environment | Retain ethnographic dates, locations, and shared-history information; observations are not independent samples of prehistoric societies. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Seshat** | Comparative polity organization, bureaucracy, and infrastructure; reproducible historical snapshots | Preserve coding uncertainty and pin a release rather than silently using changing records. [Seshat Databank](https://seshatdatabank.info/data) |
| **Published ABM code and supplements** | Reproduce mechanisms before translating them into Rust | Published parameters may be stylized settings rather than directly measured behavioral constants. [Springer](https://link.springer.com/article/10.1007/s10745-021-00273-6) |

The thinnest evidence concerns universal tax burdens, routine informal-transfer quantities, corruption rates, and the fraction of total production controlled by ancient institutions. Archaeological capacity is not observed stock; a legal ration is not delivered consumption; a recorded labor obligation is not labor actually performed.

**The best TCE abstraction is therefore a society of people with needs, relationships, rights, duties, and incomplete knowledge, connected by costly physical logistics.** Markets, gifts, temples, chiefs, commons, and planners become alternative—and overlapping—ways of directing those same flows. Their strengths and failures can then emerge from the rules and circumstances rather than from predetermined historical outcomes.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92890-a8d8-83ea-8a84-b0454f8a0a5d)
