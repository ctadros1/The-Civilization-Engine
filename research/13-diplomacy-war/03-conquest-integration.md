# Conquest, occupation, and territorial integration in TCE

## Executive conclusion

**TCE should model conquest as a change in enforceable relationships—not an automatic transfer of obedience, revenue, or culture.**

The central historical distinction is between **defeating a government, governing its population, and integrating that population**. These processes can follow different trajectories. A conqueror may collect tribute without administering households, administer a settlement without gaining political acceptance, or incorporate local elites while most inhabitants retain their language and religion. Direct and indirect rule are best understood as different allocations of authority, often combined within the same political system. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/an-institutional-theory-of-direct-and-indirect-rule/273A5AF537FBBD0D1A7C4959D3DE83A0)

For TCE, maintain at least four separate outcomes:

| Outcome | What it means in the simulation |
| --- | --- |
| **Military control** | Who can occupy important sites, move forces, and defeat organized opposition locally? |
| **Administrative incorporation** | Whose officials assess obligations, adjudicate disputes, appoint officeholders, and enforce decisions? |
| **Political acceptance** | Which households and organizations comply willingly, bargain conditionally, evade, or resist? |
| **Cultural and demographic change** | How do language, religion, identification, migration, and population composition change? |

**Do not use a universal annual assimilation rate or post-conquest rebellion probability.** The available evidence supports conditional mechanisms and specific historical benchmarks, not an invariant countdown from “conquered” to “integrated.” Religious conversion in medieval Egypt, for example, depended partly on household economic circumstances; modern occupation outcomes depended on the political setting and objectives, not merely elapsed time. [TSE Fr](https://www.tse-fr.eu/publications/road-heaven-taxation-conversions-and-coptic-muslim-socioeconomic-gap-medieval-egypt)

---

# 1. Mechanisms: implementable causal rules

## 1.1 Represent governance as delegated powers

The following is a recommended implementation, rather than a claim that historical governments fitted neatly into mutually exclusive categories.

| Arrangement | Institutional representation | Principal trade-off |
| --- | --- | --- |
| **Direct annexation** | The conqueror incorporates the settlement into its governmental hierarchy and controls key appointments, taxation, and adjudication. Local personnel may remain. | Greater potential control, but the conqueror must supply administrative capacity and absorb political responsibility. |
| **Indirect rule** | Existing authorities retain specified powers in exchange for tribute, military service, order, or other obligations. | Local knowledge and organization remain available, but intermediaries can conceal resources, renegotiate, or defect. |
| **Puppet/client government** | A separate government retains its treasury, offices, and supporters, while an external patron constrains appointments, diplomacy, or military choices. | The patron avoids some administrative burdens but cannot assume that its client has identical interests or reliable domestic authority. |
| **Military occupation** | Military authorities exercise provisional control, potentially alongside surviving civilian institutions and competing sovereignty claims. | Immediate coercive power need not produce a sustainable civilian settlement. |
| **Colonization** | Actual households move, acquire or receive land, and establish settlements or privileged communities. | It changes property and demography, but requires resources and can generate dispossession and conflict. It can accompany any of the other arrangements. |

The underlying powers should be separate fields: **appointments, taxation, revenue retention, courts, recruitment, religious administration, land allocation, and external relations**. Gerring and colleagues explicitly conceptualize directness as a continuum of delegated authority. Their important finding is that **a more centralized subordinate polity can make indirect rule more likely**, because the conqueror has an organized local authority through which to govern. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/an-institutional-theory-of-direct-and-indirect-rule/273A5AF537FBBD0D1A7C4959D3DE83A0)

## 1.2 Preserve—or destroy—administrative capacity at conquest

**Rule:** capturing a settlement changes military control immediately, but transfers usable administrative capacity only where personnel, records, and organizations survive and cooperate.

Maintain distinct inventories of:

* Assessable households, properties, and obligations.
* Officials capable of interpreting those records.
* Local authorities whose decisions people recognize.
* Storage, transport, and payment systems.

For example, replacing a ruler while retaining collectors and courts should differ from destroying the governing elite and losing its records. Wilkinson’s research on East India Company indirect rule emphasizes dependence on courtly and scribal intermediaries, intelligence, negotiation, and coercion—not simply orders descending from an omnipotent colonial center. [Cambridge University Press](https://www.cambridge.org/core/books/empire-of-influence/32FFF577F1AACB05D5E64A2945C3033A)

**Implementation consequence:** “annex territory” must not instantly reveal every household’s wealth or make every existing institution operational for the conqueror.

## 1.3 Make elite cooperation a bargain, not a culture modifier

**Rule:** local leaders evaluate whether cooperation preserves more of their resources, authority, security, and political prospects than resistance or flight.

A bargain can preserve offices, property, succession rights, religious privileges, or a share of revenue. In exchange, the intermediary supplies taxes, recruits, information, or enforcement.

Model the intermediary’s **ability to deliver** separately from willingness. A loyal claimant with few followers may be less useful than an unreliable leader with an effective organization.

Indirect rule therefore needs two relationships:

\[
\text{conqueror} \leftrightarrow \text{local government}
\leftrightarrow \text{households and organizations}.
\]

This is not necessarily a humane-versus-harsh distinction. Iyer’s study of British India finds persistent differences between directly and indirectly ruled areas, including lower subsequent public-goods access in directly ruled areas after addressing annexation selection. That is evidence that institutional arrangements matter, not that indirect rule is universally superior. [DASH](https://dash.harvard.edu/entities/publication/73120378-f93d-6bd4-e053-0100007fdf3b)

## 1.4 Separate assessed obligations from actual extraction

**Rule:** governments establish liabilities; agents and institutions determine how much is actually delivered.

For each obligation, record the payer, beneficiary, collector, assessment base, amount, due date, exemptions, arrears, and enforcement authority. Tribute may be a fixed quantity of goods, a harvest share, labor service, transport service, or recruits.

Aztec tribute lists are especially useful here: demands included regionally differentiated raw materials and manufactured goods, not merely a universal percentage of monetary income. Some demanded products also had to be acquired through exchange. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/9781118455074.wbeoe025)

A fixed quota and a proportional tax produce different dynamics. In an **illustrative arithmetic example**, a demand of 20 grain units is 20% of a 100-unit harvest but 40% of a 50-unit harvest. The nominal obligation has not changed; the household burden has doubled.

Track overlapping claims. A household can owe its landlord, temple, municipality, local ruler, and conqueror simultaneously. Do not silently replace all previous obligations with one imperial tax.

## 1.5 Make garrisons consume resources and provide specific services

**Rule:** a garrison’s contribution depends on supplied personnel, deployable time, travel access, and cooperation—not simply its nominal headcount.

In TCE, assign personnel to actual tasks: guarding sites, escorting officials, responding to incidents, securing routes, and supporting field operations. Separate personnel present from personnel available after illness, leave, training, and other duties.

Modern stability-operation research highlights substantial personnel commitments and the difference between deployed strength and the larger force needed to sustain deployments. It does **not** establish an ancient or universal “soldiers per thousand subjects” requirement. [DOI](https://doi.org/10.55540/0031-1723.1751)

**Implementation consequence:** reuse TCE’s food, wages, transport, equipment, and maintenance systems. Unpaid or undersupplied units should develop problems through those systems; do not charge an abstract occupation fee while leaving soldiers unaffected.

## 1.6 Allow compliance, evasion, and rebellion to diverge

**Rule:** grievance changes preferences; organization and opportunity determine whether those preferences become collective action.

Households should have several possible responses: compliance, petitions, concealed production, bribery, migration, assistance to the government, assistance to opponents, or open participation in resistance.

A rebellion requires some combination of organizers, communication, resources, protection, and expectations that others will participate. Epstein’s civil-violence model is useful for showing how local information and perceived enforcement risk can generate intermittent collective unrest, although it is an exploratory model rather than a calibrated model of conquered settlements. [Brookings](https://www.brookings.edu/articles/modeling-civil-violence-an-agent-based-computational-approach/)

A quiet settlement is therefore not necessarily a loyal settlement. Its inhabitants may lack an organization, expect defeat, or prefer less visible forms of resistance.

## 1.7 Give coercion multiple, potentially opposing effects

**Rule:** coercive violence can simultaneously change fear, organizational capacity, household losses, displacement, and hostility. Do not collapse these into one “suppression” effect.

The empirical literature does not support an invariant response. Lyall’s study of Chechen villages found a **24% reduction in subsequent insurgent attacks** associated with shelling in its matched comparison. Kocher, Pepinsky, and Kalyvas found that aerial bombing in Vietnam shifted territorial control toward the Viet Cong. These studies examine different settings and outcomes; neither licenses a general rule that indiscriminate violence always succeeds or always backfires. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002708330881)

For TCE, record **observed violence, underlying hostility, organizational strength, and political acceptance separately**. A reduction in attacks must not automatically raise legitimacy.

## 1.8 Treat assimilation as several household-level processes

**Rule:** language learning, religious conversion, political identification, and legal incorporation occur through different mechanisms.

A person can learn the governing language without abandoning a home language; acquire citizenship without changing religion; or support a ruler while rejecting the ruler’s cultural identity.

Saleh’s study of Egypt links historical non-Muslim taxation to selective conversion: higher-tax districts subsequently had fewer but wealthier remaining Copts. This is a mechanism involving household resources and selection, not a uniform conversion clock. [TSE Fr](https://www.tse-fr.eu/publications/road-heaven-taxation-conversions-and-coptic-muslim-socioeconomic-gap-medieval-egypt)

Use exposure through workplaces, markets, courts, military service, schools, marriage, and household upbringing. Also allow institutions and parents to maintain minority practices. Bisin and Verdier’s cultural-transmission model demonstrates conditions under which persistent cultural heterogeneity is an equilibrium rather than a temporary failure to assimilate. [EPrints Soton](https://eprints.soton.ac.uk/33480/)

## 1.9 Distinguish population treatments and preserve their consequences

**Rule:** enslavement, compulsory labor, forced relocation, expulsion, imprisonment, and killing must be distinct events and statuses.

An enslaved person has a different legal and coercive relationship from a taxable farmer, a temporarily conscripted worker, or a relocated household. Assyrian resettlement illustrates why deportation cannot automatically mean enslavement: the state relocated populations for productive and political purposes, and resettled people could hold property and become part of the tax-paying population. The evidence does not justify treating forced relocation as harmless. [Oracc](https://oracc.museum.upenn.edu/saao/aebp/Essentials/Governors/Massdeportation/index.html)

In the simulation, these actions should alter actual agents, households, property claims, labor availability, kin networks, and locations. Consequences such as disrupted harvests or lost craft capacity should follow from those changes, not from an arbitrary permanent regional debuff.

---

# 2. Parameters: evidence, ranges, and calibration limits

## 2.1 Historical benchmarks

**Confidence here concerns the specific observation, not transferability.** “High” means comparatively well documented within the stated setting; “medium” means an estimate, reconstruction, or study-dependent result. None of these is a universal default.

| Quantity | Value and units | Scope and appropriate use | Confidence and source |
| --- | --- | --- | --- |
| **Colonial mining labor draft** | **1/7 ≈ 14.3% of the designated adult male labor force per year** | Spanish mining *mita*, instituted in 1573 and formally lasting until 1812; designated communities in Peru/Bolivia. Not 14.3% of all inhabitants, and not a general Inka labor-tax rate. | **High** for formal quota; actual compliance varied. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/did-the-colonial-mita-cause-a-population-collapse-what-current-surnames-reveal-in-peru/D8267DF8063B918E488753BDD24C37B7) |
| **Military route to Roman citizenship** | Normally **25 years of auxiliary service** | Documented imperial military diplomas. A career-linked legal reward, not a rate of cultural conversion. | **High**, including surviving diplomas. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/251376) |
| **Aztec tribute hierarchy** | **371 city-states grouped into 38 tribute provinces** in the Codex Mendoza account | Useful for representing nested tribute obligations and many local authorities beneath a larger imperial structure. Not a fixed administrative staffing ratio. | **Medium–high** for the recorded system. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/9781118455074.wbeoe025) |
| **Ordinary policing comparator** | Approximately **1–4 police per 1,000 inhabitants** | Quinlivan’s 2003 comparison for relatively peaceful settings. Historical modern comparator, not a current global estimate or occupation requirement. | **Medium**. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/corporate_pubs/2007/RAND_CP22-2003-08.pdf) |
| **Difficult stabilization comparator** | Approximately **20 security personnel per 1,000 inhabitants** | A heuristic drawn from selected modern cases; the Northern Ireland comparison combines military and police. | **Medium** as a case-based benchmark; **low** as a universal threshold. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/corporate_pubs/2007/RAND_CP22-2003-08.pdf) |
| **Peak international force density** | Bosnia 1996: **22.6/1,000**; Kosovo 1999: **23.7/1,000** | International personnel divided by the population estimates used in the source. Not total local-plus-foreign security strength. | **Medium–high** for the reported calculation. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/corporate_pubs/2007/RAND_CP22-2003-08.pdf) |
| **French metropolitan fiscal contribution** | French West African colonization absorbed **0.29% of French annual expenditures** in Huillery’s reconstruction: **0.24%** military/central administration and **0.05%** development | Measures burden on the metropolitan public budget, not the full economic or human cost of colonial rule. | **Medium**, historical budget reconstruction. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/black-mans-burden-the-cost-of-colonization-of-french-west-africa/42EF127DCC7EA90A0DD1362A8D0B1954) |
| **External share of colonial revenue** | Mainland France supplied approximately **2% of French West African revenue** | Demonstrates the importance of local financing. It is not evidence that administration was inexpensive for African taxpayers. | **Medium**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/black-mans-burden-the-cost-of-colonization-of-french-west-africa/42EF127DCC7EA90A0DD1362A8D0B1954) |
| **Selected occupation outcomes** | **24 cases:** 7 successful, 4 mixed, 13 failed—approximately **29%, 17%, 54%** | Edelstein’s 2004 sample of occupations since 1815, classified against occupier objectives and costs. The study excludes annexation and colonialism from its main definition. | **Medium**; case selection and outcome classification matter. **Not a rebellion rate.** [Belfer Center](https://www.belfercenter.org/sites/default/files/pantheon_files/files/publication/edelstein.pdf) |
| **Context-specific response to violence** | **24% lower subsequent insurgent attacks** | Lyall’s Chechnya study. An estimated behavioral outcome in one setting, not increased legitimacy or a general suppression multiplier. | **Medium**, identification-dependent. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002708330881) |

The important denominator warning is that **troops, police, international contingents, eligible laboring men, households, and total inhabitants are different populations**. Mixing them can create order-of-magnitude errors.

For early agrarian TCE, determine sustainable garrison size principally from **available labor, provisioning, local obligations, transport, and actual security tasks**. Use modern force-density examples only as comparisons for later capabilities.

## 2.2 Assimilation: use conditional transmission parameters

There is no defensible general table of “conquered cultures assimilate at X% per year.” Better parameters are:

| Process | Parameter to represent | Calibration approach |
| --- | --- | --- |
| Language acquisition | Probability of gaining proficiency conditional on age, contact, instruction, and incentives | Fit to an appropriate linguistic setting; distinguish learning from abandonment of another language. |
| Household transmission | Probability that a child acquires each household language or affiliation | Include mixed households and minority institutions. |
| Religious conversion | Individual or household transition probability conditional on incentives and constraints | Keep separate from language and political allegiance. |
| Legal incorporation | Eligibility rules, waiting periods, service requirements, or discrete collective grants | Can change abruptly through legislation or negotiated settlement. |
| Demographic replacement | Births, deaths, immigration, emigration, displacement | Never count this as assimilation. |

For sensitivity testing, a transparent **invented scenario model** is preferable to an invented historical rate. Suppose a closed population has equal demographic rates, no reverse language shifts, and a fraction \(q\) of each remaining minority-language lineage shifts its children’s principal home language each generation:

\[
u\_{n+1}=u\_n(1-q).
\]

With a **25-year generation assumed for the experiment**, the equivalent half-life is:

\[
T\_{1/2}=25\frac{\ln(0.5)}{\ln(1-q)}.
\]

| Assumed shift per generation | Equivalent half-life | Status |
| --- | --- | --- |
| 0% | No convergence | Design scenario |
| 10% | Approximately **165 years** | Design scenario |
| 30% | Approximately **49 years** | Design scenario |
| 50% | **25 years** | Design scenario |

These are arithmetic consequences of assumptions, **not observed historical assimilation rates**. Their value is to expose how strongly a seemingly modest transmission setting controls a centuries-long simulation.

Legal incorporation can be much faster: the citizenship grant of **212 CE** extended Roman citizenship to almost the entire free population, without implying instantaneous linguistic or religious change. [Cambridge University Press](https://www.cambridge.org/core/books/abs/junian-latinity-in-the-roman-empire-volume-1/junian-latinity-in-late-roman-and-early-medieval-texts-a-survey-from-the-third-to-the-eleventh-centuries-ad/E8763C141F798188415CB416E0C7252C)

## 2.3 Recommended initial test ranges—not historical estimates

Where evidence cannot identify a universal value, use explicit experimental ranges and calibrate their consequences.

| Parameter | Initial sensitivity tests | Units and interpretation |
| --- | --- | --- |
| Harvest-share obligation | **5%, 10%, 20%, 30%** | Share of the explicitly defined assessed harvest. Test against household reserves and overlapping claims. |
| Compulsory service | **0, 15, 30, 60** | Days per eligible adult per year, including travel where required. Seasonal placement matters as much as the total. |
| External garrison | **0, 1, 5, 10, 20** | Personnel per 1,000 civilians; an experimental grid, not an ancient prescription. Include local forces separately. |
| Garrison food buffer | **15, 30, 90** | Days of planned consumption; compare with actual resupply lead times and interruption risks. |
| Local revenue retention | **0%, 25%, 50%, 75%, 100%** | Share of collected revenue retained by the subordinate authority before separately specified obligations. |

**Source and confidence for every row: proposed TCE calibration experiment; empirically unvalidated.** These values should be stored separately from evidence-backed historical presets.

For rebellion, avoid choosing a universal annual rate merely to complete a parameter table. First define the outcome—plot formation, tax refusal, riot, insurgency, elite defection, or successful secession—and fit its conditional incidence where suitable data exist.

---

# 3. Variation across eras and world regions

The era labels below describe combinations of capabilities, not mandatory stages TCE must unlock.

| Setting | Historical variation | Implication for TCE |
| --- | --- | --- |
| **Foraging and fishing societies** | Foragers were not uniformly egalitarian or incapable of coercive hierarchy. Northwest Coast societies included hereditary rank and war-captive slavery, while neighboring societies developed markedly different institutions. Wengrow and Graeber emphasize that subsistence mode alone does not determine political arrangements. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10040342/) | Permit control of resource sites, captive-taking, tribute, and household domination without requiring agriculture or a bureaucratic state. Mobile populations may also escape rather than remain governable. |
| **Early farming and early states** | Scott’s synthesis emphasizes concentrated populations, appropriable grain, and control over labor as favorable conditions for early state extraction. Its broad explanatory claims should be treated as a thesis, not an invariant law. [Yale University Press](https://yalebooks.yale.edu/book/9780300240214/against-the-grain/) | Make storage, labor concentration, assessability, and opportunities for flight important. Farming alone should not automatically create a durable territorial state. |
| **Assyria and other early territorial empires** | Assyrian rulers used population relocation to redistribute labor and skills and consolidate rule. Relocated people were not all assigned the same legal status. [Oracc](https://oracc.museum.upenn.edu/saao/aebp/Essentials/Governors/Massdeportation/index.html) | Represent forced migration as transport of households and capabilities, with losses and coercion, rather than a map-level culture operation. |
| **Roman imperial settings** | Military service could create individual paths to citizenship over a career, while collective legal incorporation could occur through a decree. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/251376) | Separate service incentives, legal rights, elite access, settlement, and cultural transmission. |
| **Aztec Mesoamerica** | Tributary and strategically important provinces had different functions; strategic territories often helped buffer tribute-producing areas from hostile neighbors. [Revistas Científicas Complutenses](https://revistas.ucm.es/index.php/REAA/en/article/view/REAA0707220119A) | Allow an empire to accept low revenue or special privileges from a strategically valuable frontier settlement. |
| **Inka Andes and Spanish colonial Andes** | Inka *mit’a* mobilized household labor for roads, agriculture, military activity, and other state purposes within a wider provisioning system. Spanish colonial mining *mita* was a different, specifically organized labor draft. [Smithsonian Astrophysical Observatory](https://b.asp.si.edu/inkaroad/inkauniverse/inkaroadexpansion/building-road.html) | Author institutions by actual obligations and provisioning, not by borrowing the name of one regime for all forced labor. |
| **Medieval Egypt** | Taxation, conversion, and socioeconomic selection interacted over long periods; remaining religious communities were not random samples of the original population. [TSE Fr](https://www.tse-fr.eu/publications/road-heaven-taxation-conversions-and-coptic-muslim-socioeconomic-gap-medieval-egypt) | Cultural change must be wealth-sensitive and capable of changing group composition without homogenizing everybody. |
| **Southwest China** | Research on imperial expansion stresses interaction between indigenous authorities, local ritual, livelihoods, and state institutions. Weinstein’s Guizhou study documents persistent challenges to Qing control in the eighteenth century. [UBC Press](https://www.ubcpress.ca/chieftains-into-ancestors) | Incorporation should depend on local institutions and ecology. A district can be administratively incorporated yet difficult to govern in practice. |
| **South Asia and colonial West Africa** | Company indirect rule depended on local intermediaries and bargaining; French West African fiscal arrangements placed much of the financing burden locally. [Cambridge University Press](https://www.cambridge.org/core/books/empire-of-influence/32FFF577F1AACB05D5E64A2945C3033A) | Differentiate administrative delegation from who ultimately pays, and distinguish patron interests from those of local officeholders. |
| **Industrial nation-building** | Weber’s study of rural France emphasizes roads, railways, markets, and schooling in the spread of national language and identification during 1870–1914. This is a particular nationalizing history, not a universal timetable. [Apple](https://books.apple.com/us/book/peasants-into-frenchmen/id584364766?utm_source=chatgpt.com) | New communication and educational capacities should modify contact and incentives, not activate automatic assimilation. |
| **Modern military occupation** | Political goals, perceived external threats, reconstruction needs, and credible expectations about the occupation’s future affect outcomes. Longer occupation can provide institutional rebuilding time while also intensifying opposition to foreign rule. [Belfer Center](https://www.belfercenter.org/sites/default/files/pantheon_files/files/publication/edelstein.pdf) | Permit stabilization, negotiated withdrawal, client survival, renewed conflict, or institutional failure—not only eventual annexation. |

---

# 4. Stylized facts a correct simulation should reproduce

These are **validation targets**, not scripts.

| Pattern | Expected simulated behavior |
| --- | --- |
| **One empire contains several governance arrangements.** Aztec tributary and strategic provinces provide a concrete example. [Revistas Científicas Complutenses](https://revistas.ucm.es/index.php/REAA/en/article/view/REAA0707220119A) | Similar settlements can receive different terms because of strategic value, prior institutions, and bargaining power. |
| **Legal inclusion and cultural change operate on different clocks.** Roman service grants and the 212 CE citizenship extension illustrate this distinction. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/251376) | Rights can change in a day while household practices persist across generations. |
| **Cultural plurality need not disappear eventually.** Cultural-transmission models can sustain heterogeneous populations. [EPrints Soton](https://eprints.soton.ac.uk/33480/) | Some integrated territories remain multilingual or religiously plural indefinitely. |
| **Quiet does not prove consent.** Reduced attacks and improved political acceptance are not the same measured outcome. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002708330881) | Resistance can become dormant, clandestine, or displaced without legitimacy rising. |
| **Low metropolitan expenditure can coexist with high local burden.** Huillery’s 0.29% and approximately 2% figures illustrate the different accounting perspectives. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/black-mans-burden-the-cost-of-colonization-of-french-west-africa/42EF127DCC7EA90A0DD1362A8D0B1954) | An occupation can appear inexpensive to the conqueror while impoverishing occupied households. |
| **Institutional legacies can persist, but need not be permanent.** Recent work on Peru challenges strong persistence claims. [Springer](https://link.springer.com/article/10.1007/s10887-024-09249-9) | Migration, competing employers, legal change, and political opposition can weaken coercive institutions. |
| **Government form matters beyond the initial conquest.** Iyer finds long-run differences associated with direct versus indirect colonial rule. [DASH](https://dash.harvard.edu/entities/publication/73120378-f93d-6bd4-e053-0100007fdf3b) | Identical initial military victories can produce different public goods, fiscal capacities, and political trajectories. |

Also test several model-internal implications: failed harvests should make fixed quotas more onerous; a supply interruption should weaken a dependent garrison; killing or displacing skilled workers should reduce available productive capacity; and rebellion should never create manpower that was absent from the population.

---

# 5. Modeling recommendation for TCE

## 5.1 Use individuals, households, and persistent institutions

A compact representation can remain agent-based without making every interaction an individual negotiation.

| Entity | Essential state |
| --- | --- |
| **Person** | Household, occupation, legal status, languages/proficiency, affiliations, political ties, remembered losses, military or organizational membership. |
| **Household** | Food and assets, property claims, dependents, obligations, migration options, cultural transmission decisions. |
| **Local institution** | Officeholders, jurisdiction, records, resources, supporters, legitimacy among different groups, delegated powers. |
| **Rule agreement** | Parties, appointment rights, retained powers, transfers, military obligations, guarantees, revision conditions, recorded violations. |
| **Settlement** | Claimants, actual local enforcement, service access, administrative capacity, supply access, distributions of attitudes—not one average opinion. |
| **Military or resistance organization** | Actual members, leadership, resources, equipment, supply, objectives, relationships, and operational availability. |

A puppet government should be an **actual government entity**. Its treasury, supporters, and commitments can diverge from those of its patron. Annexation should modify or terminate specific rights and obligations, not simply change a parent pointer while leaving every other mechanism untouched.

## 5.2 Account for the cost of holding territory in two ledgers

### Treasury ledger

For a defined period:

\[
\begin{aligned}
\text{net fiscal return}={}&
\text{collected taxes and tribute}
+\text{other receipts}\\
&-\text{military outlays}
-\text{civil administration}\\
&-\text{transport and maintenance}
-\text{client subsidies}\\
&-\text{exceptional suppression or reconquest costs}.
\end{aligned}
\]

Keep initial conquest, recurring holding costs, and exceptional campaigns separate. Record which treasury pays each expense. Loot and asset confiscation are one-time transfers unless a separate mechanism produces recurring income.

### Resource and household-burden ledger

Track food consumed, labor withdrawn from production, transport labor and animals, destroyed capital, lost output, mortality, and displacement.

Do not double-count transfers as destroyed resources. A soldier’s local wage is an expense to the treasury and income to someone else; the soldier’s food consumption and foregone alternative work are separate resource questions.

**Illustrative TCE calculation:** a garrison of 200 people requiring one standardized daily ration consumes **73,000 ration-days per year**. A 90-day buffer requires **18,000 ration-days**. These are accounting identities, not historical force requirements; dependents, animals, wastage, and transport increase the total.

This approach lets holding costs emerge from existing economic systems rather than requiring an empire-wide occupation-cost percentage.

## 5.3 Use event-driven political transitions

Represent resistance escalation through organizations and opportunities. One suitable computational form is:

\[
P(\text{faction acts during }\Delta t)
=1-\exp(-\lambda\Delta t),
\]

where \(\lambda\) is an event hazard with units of inverse time.

Let it depend on the faction’s organization, grievances, resources, perceived enforcement, external support, and recent shocks. The equation is a time-consistent implementation device; it does not supply an empirically established hazard.

Crucially, distinguish events: withholding tribute, refusing recruitment, dismissing an imperial official, an urban riot, an organized uprising, and successful secession are not interchangeable.

Government transitions should likewise require conditions. Replacing indirect rule with direct administration requires appointments, information, financing, and enforcement capacity. Independence requires more than high dissatisfaction: institutions must be able to sustain the new relationship.

## 5.4 Preserve demographic accounting

For every tracked identity group:

\[
N\_{t+1}
=N\_t+\text{births}-\text{deaths}
+\text{immigration}-\text{emigration}
+\text{identity entries}-\text{identity exits}.
\]

Keep all terms separately queryable. A declining minority share caused by immigration is different from conversion; a decline caused by expulsion or killing must not appear in the interface as successful assimilation.

For multi-affiliated people, track attributes separately rather than forcing everyone into one exclusive “culture” category.

## 5.5 Use multiple update frequencies

For a 10k–50k-person Rust simulation, I recommend:

| Frequency | Suitable processes |
| --- | --- |
| **Daily or event-driven** | Movement, provisioning, violence, arrests, flights, official appointments, disrupted routes. |
| **Monthly or obligation-driven** | Payroll, tribute collection, arrears, administrative workloads, organizational recruitment. |
| **Seasonal** | Harvest assessments, labor levies, provisioning campaigns, agricultural opportunity costs. |
| **Life-course and cohort events** | Language acquisition, household formation, inheritance, legal eligibility, religious and political affiliation changes. |

Use sparse social networks and cached settlement-level conditions. With at most \(k\) relevant contacts per person, a social update can target **\(O(Nk)\)** work rather than all-pairs **\(O(N^2)\)** comparisons. That is an architectural target, not a measured performance claim.

Fast-forward must preserve obligation deadlines and accumulated event hazards. It should not replace conditional processes with “integration increases once per year.”

## 5.6 Existing models and games worth borrowing from

| Model or game | Useful element | What TCE should not copy |
| --- | --- | --- |
| **Epstein, civil-violence ABM** | Local information, perceived enforcement, and intermittent collective activity. [Brookings](https://www.brookings.edu/articles/modeling-civil-violence-an-agent-based-computational-approach/) | Do not treat a stylized grievance/arrest model as an empirically calibrated theory of all rebellions. Add organizations, livelihoods, bargaining, and institutions. |
| **Bisin–Verdier cultural transmission** | Household socialization and persistent cultural heterogeneity. [EPrints Soton](https://eprints.soton.ac.uk/33480/) | Do not collapse language, religion, legal status, and political loyalty into one inherited preference. |
| **Hearts of Iron IV’s documented resistance/compliance design** | Separates resistance from willingness to cooperate; garrison activity incurs manpower and equipment losses; occupation policies trade short-run extraction against other objectives. [Paradox Plaza Forum](https://forum.paradoxplaza.com/forum/threads/hoi4-dev-diary-resistance-and-compliance.1240349/) | Do not import game growth rates or assume compliance naturally rises toward completion. For TCE, aggregate values should summarize actual agents and institutions. |

The HOI4 comparison refers to its **2019 developer-documented design**, not a claim that every numerical detail remains unchanged in the current game.

---

# 6. Sources, datasets, and contested evidence

## 6.1 Datasets suitable for calibration and validation

| Resource | Best use | Important limitation |
| --- | --- | --- |
| **Seshat Global History Databank** | Historical administrative complexity, institutions, polity characteristics, and comparative case selection. [Seshat DB](https://seshat-db.com/whoweare/) | Uneven evidence and temporal resolution. Do not interpret missing values as institutional absence. |
| **Cliopatria** | Historical polity geography and territorial change; current releases cover more than 1,800 political entities from 3400 BCE onward. [Seshat DB](https://seshat-db.com/core/cliopatria/) | Boundaries contain uncertainty and interpretation; mapped sovereignty is not equivalent to uniform local control. |
| **D-PLACE / Ethnographic Atlas** | Comparative political organization, subsistence, settlement, and relevant social institutions across societies. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/linguistic-and-cultural-evolution/research/d-place/) | Ethnographic observations are not a direct time series of prehistory. Pin versions and inspect reuse terms before redistributing data. |
| **Correlates of War Territorial Change, v6** | Territorial transfers during **1816–2018**, involving at least one recognized state in its coding system. [Correlates of War](https://correlatesofwar.org/data-sets/territorial-change/) | Does not measure household integration, all colonial political relationships, or day-to-day occupation control. |
| **Ethnic Power Relations / GeoEPR** | Post-1946 group-level political access, exclusion, autonomy, and spatial relationships. [International Conflict Research](https://icr.ethz.ch/data/epr/core/) | Covers politically relevant groups under specific definitions, not all cultural identities. |
| **UCDP** | Modern organized conflict, event patterns, escalation, and civilian victimization under documented definitions. [Uppsala University](https://www.uu.se/en/department/peace-and-conflict-research/research/ucdp/ucdp-definitions.html) | Its **25 battle-death annual threshold** for active armed conflict is a coding rule—not a sensible minimum rebellion size for a 50,000-person simulated world. |

For local mechanisms, replication materials from individual studies may be more valuable than global datasets: village-level violence, household census records, tribute lists, and institution-specific administrative records match TCE’s scale more closely.

## 6.2 Core scholarly reading

The most useful foundations for this subsystem are:

| Work | Principal contribution |
| --- | --- |
| **Gerring, Ziblatt, Van Gorp, and Arévalo (2011), “An Institutional Theory of Direct and Indirect Rule,” *World Politics*.** | Delegation and the role of pre-existing local centralization. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/an-institutional-theory-of-direct-and-indirect-rule/273A5AF537FBBD0D1A7C4959D3DE83A0) |
| **Wilkinson (2023), *Empire of Influence*.** | Indirect rule as a contested practice involving intermediaries, intelligence, negotiation, and coercion. [Cambridge University Press](https://www.cambridge.org/core/books/empire-of-influence/32FFF577F1AACB05D5E64A2945C3033A) |
| **Iyer (2010), “Direct versus Indirect Colonial Rule in India: Long-Term Consequences,” *Review of Economics and Statistics*.** | Institution-specific evidence and the problem of selection into annexation. [DASH](https://dash.harvard.edu/entities/publication/73120378-f93d-6bd4-e053-0100007fdf3b) |
| **Huillery (2014), “The Black Man’s Burden,” *Journal of Economic History*.** | Separating metropolitan expenditure from colonial fiscal burden. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/black-mans-burden-the-cost-of-colonization-of-french-west-africa/42EF127DCC7EA90A0DD1362A8D0B1954) |
| **Saleh (2018), “On the Road to Heaven,” *Journal of Economic History*.** | Taxation, conversion, household selection, and persistent group differences. [TSE Fr](https://www.tse-fr.eu/publications/road-heaven-taxation-conversions-and-coptic-muslim-socioeconomic-gap-medieval-egypt) |
| **Edelstein (2004), “Occupational Hazards,” *International Security*.** | Military occupation as a political problem with objectives, costs, and competing time pressures. [Belfer Center](https://www.belfercenter.org/sites/default/files/pantheon_files/files/publication/edelstein.pdf) |
| **Berdan (2007), “En la periferia del imperio.”** | Distinct tributary and strategic relationships within the Aztec imperial system. [Revistas Científicas Complutenses](https://revistas.ucm.es/index.php/REAA/en/article/view/REAA0707220119A) |
| **Radner (2012), “Mass Deportation: The Assyrian Resettlement Policy.”** | Population relocation, labor, administration, and the distinction between deportees and enslaved people. [Oracc](https://oracc.museum.upenn.edu/saao/aebp/Essentials/Governors/Massdeportation/index.html) |

## 6.3 Claims that should remain explicitly uncertain

**Long-run colonial persistence is contested.** Dell’s influential study estimates roughly **25% lower household consumption and six percentage points more child stunting** in areas affected by Peru’s mining *mita*. Abad and Maurer’s 2025 study, using settlement-level reconstruction and different coverage and treatment coding, finds severe historical harm but no comparable persistent differences across its later outcomes. Their disagreement concerns geography, identification, and mechanisms; it should not be presented as settled in either direction. [DOI](https://doi.org/10.3982/ecta8121)

**Troop density is not a universal causal law.** Selected modern cases cannot identify a technology-independent threshold for holding an ancient settlement. Personnel quality, local institutions, terrain, supply, political objectives, and what counts as “control” all change the interpretation.

**Assimilation evidence is often indirect.** Administrative language, personal names, official religious labels, and legal status are not interchangeable measures of household identity. Saleh’s work is particularly valuable because it examines selection and socioeconomic composition rather than assuming that group shares alone reveal a uniform process. [TSE Fr](https://www.tse-fr.eu/publications/road-heaven-taxation-conversions-and-coptic-muslim-socioeconomic-gap-medieval-egypt)

**Prehistoric and ancient rebellion denominators are especially weak.** A recorded revolt does not establish how many comparable settlement-years were peaceful, how many smaller incidents went unrecorded, or how the recorder defined rebellion. Use such cases to construct mechanisms and plausible sequences, not fabricated annual frequencies.

**Final recommendation:** build the subsystem around **typed obligations, local political organizations, supplied coercive forces, household responses, and distinct cultural-transmission processes**. Make annexation, indirect rule, puppetry, integration, and independence emerge from changes in those relationships. This provides a stronger foundation for TCE than territory-wide loyalty clocks, automatic cultural conversion, or permanent conquest penalties.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9294f-0c9c-83e9-a432-5951345589ad)
