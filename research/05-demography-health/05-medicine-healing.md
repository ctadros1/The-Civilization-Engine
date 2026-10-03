# Healing, medicine and health institutions

## A simulation-ready research report for The Civilization Engine

**The central recommendation is to model healthcare as access to particular services—not as a universal mortality bonus attached to healer numbers or hospital buildings.** Historical care ranged from useful nursing and skilled procedures to ineffective remedies and dangerous interventions. Major improvements often came from making a narrow activity safer or more reliable: childbirth, wound treatment, immunization, or replacement of lost fluids. The historical evidence does not support a single effectiveness coefficient for “premodern medicine.” [OUP Academic](https://academic.oup.com/jeea/article/19/4/2052/6217426)

For TCE, distinguish four outcomes: **preventing illness, surviving illness, recovering function, and relieving suffering**. Also distinguish the capability to deliver a treatment from the probability that an individual actually receives it. This permits a settlement to have knowledgeable healers but poor coverage, impressive hospitals but unsafe wards, or excellent community midwifery without advanced surgery.

Throughout this report, **historical measurements are separated from proposed simulation parameters**. Archaeological survival, hospital case fatality, population mortality, and randomized intervention effects answer different questions and must not be treated as interchangeable.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Household care is the foundation, not an absence of healthcare

Care should begin with relatives and neighbors providing food, water, shelter, assistance, and time away from work. Specialized institutions sometimes assumed these functions when family support was unavailable: medieval Islamic hospitals, for example, combined treatment and convalescence with support for aged or infirm people lacking caregivers. [Internet Archive](https://archive.org/stream/9439902.nlm.nih.gov/9439902_djvu.txt)

**Implementation:** an ill person creates care tasks. Completing them changes actual consumption, exposure, exertion, or disability management. Caregiving consumes someone else’s labor.

Do not grant a separate “nursing nutrition bonus” after already crediting the food and water delivered. The benefit should emerge from meeting needs the patient could not meet alone. A healer visit cannot substitute for several days of unattended convalescence.

For v1, distinguish ordinary household support from **additional organized care**. Otherwise, adding one healer risks reproducing benefits already implicit in the baseline mortality model.

### 1.2 Practical competence and medical theory are separate variables

Sophisticated explanations of disease did not guarantee effective treatment. Conversely, effective practices could precede an adequate explanation. Semmelweis’s hand-disinfection intervention preceded the later microbial explanation of puerperal infection; a modern reanalysis of nineteenth-century pneumonia records also finds that bloodletting offered no demonstrated benefit. [Springer](https://link.springer.com/article/10.1007/s10654-022-00871-8)

Give a practitioner separate attributes for procedural skill, diagnostic discrimination, known protocols, and reputation. A prestigious physician can competently administer an ineffective treatment. A low-status birth attendant can possess useful practical knowledge.

Each treatment protocol should specify:

`eligible conditions → required materials → time → probability of correct execution → benefits → adverse effects`

Ritual and reputation can influence trust, perceived relief, and care-seeking without being assigned unsupported antimicrobial effects.

### 1.3 Remedies are indication-specific, variable products

Cinchona illustrates both the reality and limitations of effective preindustrial pharmacology. Its bark was used against malaria before quinine was isolated; historical assessments report roughly **1–4% quinine in dry bark**, depending on origin. Isolation and measurement made treatment more reproducible. This is not evidence that an arbitrary herbal mixture was broadly effective. [The James Lind Library](https://www.jameslindlibrary.org/articles/evaluating-cinchona-bark-and-quinine-for-treating-and-preventing-malaria/)

Represent a medicinal product by identity, potency, spoilage or adulteration, and toxicity—not merely “medicine units.” A remedy can work for one disease, fail for a similar-looking disease, and become harmful through excessive or inconsistent dosing.

For unvalidated remedies, the defensible default is **no established curative effect**, with any specific benefit or harm authored separately.

### 1.4 Surgery changes several competing risks

Surgery is neither automatically lethal before modernity nor automatically beneficial after anesthesia. A person in Borneo survived a childhood lower-leg amputation approximately 31,000 years ago; skeletal evidence indicates another **6–9 years of life**. That establishes an impressive possibility, not a representative surgical survival rate. [Nature](https://www.nature.com/articles/s41586-022-05160-8)

A procedure should separately affect the original injury or disease, bleeding, infection, pain, and subsequent function. Success can mean survival with disability rather than complete restoration.

Anesthesia principally changes pain, immobility, and feasible operating time. Antisepsis and asepsis address infection. These should be complementary capabilities, not interchangeable “surgery quality” upgrades. The historical spread of anesthesia and Listerian antisepsis occurred through distinct innovations and adoption processes. [The Royal College of Anaesthetists](https://rcoa.ac.uk/about-us/heritage/history-anaesthesia)

### 1.5 Midwifery is a distinct service, with distinct maternal and infant outcomes

Historical Swedish evidence provides unusually strong support for trained community midwifery. However, its estimated maternal benefits did **not** translate into detected reductions in stillbirth or infant mortality in the same analysis. A later Pakistani trial of trained traditional attendants, delivery kits, and integration with other services improved perinatal outcomes, while its maternal-mortality estimate remained statistically uncertain. [OUP Academic](https://academic.oup.com/jeea/article/19/4/2052/6217426)

Therefore, represent at least maternal bleeding, infection, obstructed labor, and newborn complications separately. Clean delivery should not automatically resolve obstruction or severe hemorrhage. Recognition and referral can be valuable even when the first attendant cannot perform definitive treatment.

Access also has a timing component: a skilled attendant arriving after an irreversible complication should not receive the same effectiveness as one present during delivery.

### 1.6 Hospitals concentrate both useful resources and hazards

A hospital can pool caregivers, supplies, observation, and procedural expertise. It can also connect infectious patients, contaminated instruments, and mobile staff. The exceptionally high mortality in Semmelweis’s physician-led maternity clinic illustrates how medical activity itself could generate infection. [Springer](https://link.springer.com/article/10.1007/s10654-022-00871-8)

**Implementation:** admission changes the patient’s contact environment and care schedule. Healthcare-associated infection should arise through your existing transmission and wound-contamination systems, not through an unexplained universal hospital penalty.

A building’s usable capacity should be limited by its scarcest essential resource:

\[
B\_{\text{usable}}
=
\min(B\_{\text{physical}},B\_{\text{staffed}},B\_{\text{supplied}})
\]

An **almshouse** is a different institution: primarily accommodation and support, especially for the poor or elderly, rather than necessarily an acute-treatment facility. Its contribution should appear through shelter, consumption, care, and household burden. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/03585522.2013.861768)

### 1.7 Quarantine, isolation, and cordons operate at different points

**Isolation** separates recognized sick people. **Quarantine** restricts potentially exposed people who may not yet be visibly ill. A **cordon sanitaire** restricts movement across a boundary. Lazarettos supplied dedicated places for detention and disease management; Venice established a permanent plague hospital in **1423**. [CDC](https://wwwnc.cdc.gov/eid/article/19/2/12-0312_article)

For TCE, these interventions need different rules:

* Isolation reduces contacts after detection; it cannot undo earlier transmission.
* Quarantine delays release and permits observation; shared confinement can create additional exposure.
* Cordons reduce introductions and departures, but do not directly lower transmission already occurring inside.

The route matters. Restricting human movement does not automatically remove contaminated water or local vector reservoirs. Historical quarantine outcomes also depended on geography, timing, enforcement, and accompanying measures. An archival study of Alaska’s 1918–1920 pandemic explicitly identifies geographic isolation and incomplete records as limits on causal interpretation. [CDC](https://wwwnc.cdc.gov/eid/article/19/2/12-0312_article)

In the simulation, restrictions should also affect trade, food supply, trust, concealment, and evasion. Disease control is an institutional task, not a costless medical spell.

### 1.8 Capability grows through knowledge, organization, and production together

A discovery does not immediately become a functioning service. Penicillin’s development required successful treatment research, purification, manufacturing, and production at scale—not merely recognition that a mold inhibited bacteria. [American Chemical Society](https://www.acs.org/education/whatischemistry/landmarks/flemingpenicillin.html)

Use separate processes for discovering a practice, teaching it, obtaining materials, adopting it, and maintaining delivery. Clinical records and comparison groups can improve learning; spontaneous recovery and selective reporting can mislead it.

This supports unscripted development: useful empirical practices can spread before correct theory, and capabilities can decline when trained personnel, supply chains, or supporting institutions disappear.

---

## 2. Quantitative parameters and their interpretation

### 2.1 Empirical anchors

“Confidence” below concerns the stated finding. **Transferability to an arbitrary TCE society is usually lower.** Relative risk, abbreviated **RR**, is treated risk divided by comparison risk; an odds ratio, **OR**, is a different quantity.

| Intervention or capability | Quantitative observation, units, and setting | Appropriate simulation use | Confidence and limitations |
| --- | --- | --- | --- |
| **Prehistoric amputation** | Borneo, approximately **31,000 years ago**: one individual survived childhood amputation by **6–9 years**. Maloney et al., *Nature*, 2022. [Nature](https://www.nature.com/articles/s41586-022-05160-8) | Establish that successful complex care is possible without agriculture or metallurgy. | Strong evidence for this individual; **no estimate of typical success probability**. |
| **Andean trepanation** | Examined Peruvian skeletal series show inferred long-term survival of about **40%** in the earliest period, versus **75–83%** in Inca-period samples. Kushner, Verano and Titelbaum, 2018. [PubMed](https://pubmed.ncbi.nlm.nih.gov/29604358/) | Validate regional, practice-specific procedural expertise and learning. | Moderate for skeletal healing patterns; selection, indication, and preservation prevent treating these as randomized surgical efficacy. |
| **Trained midwife availability** | Sweden, **1830–1894**: doubling licensed midwives was estimated to reduce maternal mortality by **20–40%**. Formal training lasted approximately **6–12 months**. Lorentzon and Pettersson-Lidbom, 2021. [OUP Academic](https://academic.oup.com/jeea/article/19/4/2052/6217426) | A historical benchmark for the combined effects of workforce, access, and practice. | Moderately strong quasi-experimental evidence. **Not a patient-level RR** or a universally transferable training duration. |
| **Traditional attendants integrated with services** | Pakistan cluster trial, 2005: perinatal-death **OR 0.70**, 95% CI **0.59–0.82**; maternal-death **OR 0.74**, CI **0.45–1.23**. [ResearchGate](https://www.researchgate.net/publication/7839479_An_Intervention_Involving_Traditional_Birth_Attendants_And_Perinatal_And_Maternal_Mortality_In_Pakistan) | Model training, clean kits, outreach, and referral as a package. | Randomized across seven clusters. Maternal result is inconclusive; not evidence for training alone or ancient practice. |
| **Maternity hand disinfection** | Vienna reanalysis: puerperal-sepsis mortality **11.3%** before intervention in July 1840–October 1846, versus **2.0%** in June 1847–February 1849—about **82% lower descriptively**. [Springer](https://link.springer.com/article/10.1007/s10654-022-00871-8) | Calibrate elimination of a severe, healthcare-generated infection hazard. | Strong direction; historical comparison with changing exposures. Denominator is maternity patients, not a general-population birth cohort. |
| **Lister’s antiseptic surgical system** | Reported amputation deaths: **16/35 before**, versus **6/40 after**; approximately **45.7% → 15.0%**, RR **0.33**. [The James Lind Library](https://www.jameslindlibrary.org/articles/statistics-and-the-british-controversy-about-the-effects-of-joseph-listers-system-of-antisepsis-for-surgery-1867-1890/) | A high-risk surgical-ward scenario, not the default for every wound. | Small, nonrandomized series; case selection and contemporary statistical controversy matter. |
| **Variolation** | Jurin’s accumulated eighteenth-century figures: **2,848/17,151**, or **16.6%**, died among naturally occurring smallpox cases; **10/481**, or **2.1%**, were suspected to have died following inoculation. [The James Lind Library](https://www.jameslindlibrary.org/articles/quantitative-evidence-for-judgments-on-the-efficacy-of-inoculation-for-the-prevention-of-smallpox-england-and-new-england-in-the-1700s/) | Model an immediate hazardous intervention that can confer protection to survivors. | Useful historical orders of magnitude, but different populations and denominators; not a randomized comparison. |
| **Smallpox vaccination** | CDC’s historical summary: approximately **95%** prevention of smallpox infection; protection strongest for roughly **3–5 years**, then declining. [CDC](https://www.cdc.gov/smallpox/vaccines/index.html) | Mature vaccine efficacy and waning benchmarks; model uptake and usable vaccine supply separately. | Strong historical effectiveness; **not a guarantee for every early nineteenth-century batch**. |
| **Sulfonamide diffusion** | United States, **1937–1943**: estimated mortality reductions of **24–36% maternal**, **17–32% pneumonia**, and **52–65% scarlet fever**. Jayachandran, Lleras-Muney and Smith, 2010. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.2.2.118) | Population-level validation of a targeted pharmaceutical breakthrough. | Moderately strong historical causal analysis. Includes diffusion and access; not individual treatment efficacy. |
| **Rehydration under emergency conditions** | Bangladesh refugees in India, 1971: **3,703 cholera patients**, overall case fatality **3.6%**; a demonstration unit’s **1,190 patients** had **1%** fatality. Severe cases also received initial intravenous fluids. [Johns Hopkins University](https://pure.johnshopkins.edu/en/publications/oral-fluid-therapy-of-cholera-among-bangladesh-refugees-5) | Validate that a materially simple service can achieve large benefits under constrained conditions. | Direct field report; no randomized untreated control. Do not attribute all results to oral therapy alone. |
| **Well-functioning cholera treatment** | WHO: treatment-center case fatality should remain **below 1%** with appropriate care; severe dehydration requires rapid intravenous fluids as well as other indicated treatment. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/cholera) | Modern service-performance target, conditional on reaching care and adequate treatment. | Strong clinical benchmark; not the fatality rate of all infections or all historical treatment centers. |
| **Early, layered epidemic controls** | US cities in 1918: early implementation of multiple interventions was associated with approximately **50% lower peak death rates**. Hatchett, Mecher and Lipsitch, 2007. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0610941104) | Benchmark epidemic-curve shape under early bundled controls. | Ecological historical comparison. **Not a 50% quarantine coefficient**, and peak reduction is not cumulative mortality reduction. |

Two conversion cautions are essential.

First, when a study supplies an odds ratio and the untreated probability is \(p\_0\), use:

\[
p\_1=\frac{OR\,p\_0}{1-p\_0+OR\,p\_0}
\]

Do not automatically substitute the OR for an RR.

Second, a population effect from doubling staff combines access, selection, treatment quality, and possibly information diffusion. It cannot be assigned directly to every attended patient.

### 2.2 Proposed starting parameters where historical estimates are missing

The following are **author-selected engineering priors**, not measured historical averages. Their purpose is to make a first implementation executable while keeping uncertainty visible.

| Parameter | Suggested initial value or range | Unit and scope | Evidence status |
| --- | --- | --- | --- |
| Unvalidated remedy’s curative effect | **RR 1.00** by default | Death risk within the relevant illness episode | Conservative modeling convention; specific evidence can replace it. |
| Additional organized supportive care | Start at **RR 0.95**; test **0.85–1.00** | Selected care-sensitive illness episodes, compared with ordinary household care | Low-confidence prior. Exclude effects already represented through food, fluids, shelter, or rest. |
| Active care time per full-time provider | Start at **6**, test **4–8** | Provider-hours/day | Scheduling assumption, not a historical labor standard. |
| Simple consultation or treatment | **0.5–1.5** | Provider-hours/encounter, excluding travel | Scheduling assumption; complex procedures use separate distributions. |
| Basic additional inpatient nursing | **1–4** | Caregiver-hours/patient-day | Coarse capacity assumption; excludes intensive care and separately modeled domestic work. |
| Compliance scenarios for restrictions | **0.50, 0.80, 0.95** | Fraction complying with a specified restriction | Stress-test values, not historical estimates. Prefer emergent compliance later. |

**Do not interpret the supportive-care prior as “healers reduce all mortality by 5%.”** It applies only to selected episodes and only to care actually delivered.

There is no defensible universal ancient “one healer per X people” threshold in the evidence assembled here. Demand, travel, household participation, and service duration provide a better starting point than population alone.

---

## 3. Variation across eras and world regions

### 3.1 Historical patterns should generate different capability combinations

**Foragers.** The Borneo amputation demonstrates that technically demanding treatment and sustained postoperative support could exist in a foraging society. TCE should not make “no farming” equivalent to “no medicine,” nor use one extraordinary survivor to make surgery routinely safe. [Nature](https://www.nature.com/articles/s41586-022-05160-8)

**Early farming and agrarian states.** For early-farming presets, the safest assumption is continued reliance on household care, with specialization varying locally. Later agrarian evidence becomes more explicit: the Egyptian Edwin Smith papyrus contains structured descriptions and assessments of trauma, including spinal injuries. Written diagnostic sophistication establishes a repertoire, not measured cure rates for an entire population. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2989268/)

**Premodern regional specialization.** Andean trepanation, Chinese variolation, and Islamic institutional medicine illustrate different strengths rather than positions on one European ladder. Written discussion of variolation appears in China by **1549**; Ottoman and African knowledge also contributed to its wider transmission. Major Islamic hospitals combined treatment, convalescence, and maintenance functions, with pharmacies and other supporting spaces. [ResearchGate](https://www.researchgate.net/publication/324083662_Trepanation_ProceduresOutcomes_Comparison_of_Prehistoric_Peru_with_Other_Ancient_Medieval_and_American_Civil_War_Cranial_Surgery)

**South Asian institutions.** Archaeological and inscriptional evidence from Sri Lanka distinguishes monastic hospitals, lay inpatient facilities, maternity provision, and dispensaries. Surviving layouts support institutional differentiation, but interpretations of particular rooms and claims based on chronicles are less certain than the existence of the complexes themselves. They do not supply reliable treatment-effect coefficients. [WorldGenWeb Project](https://www.worldgenweb.org/lkawgw/hospitals.html)

**Preindustrial and industrial Europe.** Hospitals and almshouses occupied overlapping but distinct welfare roles. Later concentration of treatment could introduce serious hazards, while trained community birth attendance could improve maternal survival outside hospitals. Institutional size and professional prestige therefore need not correlate monotonically with safety. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/03585522.2013.861768)

**Modern systems.** Manufacturing and organized delivery make effective treatment reproducible, but sophisticated buildings are not always the critical bottleneck. The 1971 refugee rehydration program demonstrates the importance of a workable protocol, supplies, and delivery organization under extremely difficult conditions. [American Chemical Society](https://www.acs.org/education/whatischemistry/landmarks/flemingpenicillin.html)

### 3.2 Timeline of selected effective capabilities

These are **historical reference points, not mandatory TCE unlock dates**.

| Reference period | Capability or development | What should actually change in the simulation |
| --- | --- | --- |
| Deep prehistory onward | Skilled trauma care and sustained assistance are archaeologically attested. [Nature](https://www.nature.com/articles/s41586-022-05160-8) | Permit local procedural knowledge and caregiving before states or writing. |
| By **1549** in Chinese written evidence; subsequently documented through multiple regional traditions | Variolation. Earlier origin claims are less secure. [The James Lind Library](https://www.jameslindlibrary.org/articles/the-origins-of-inoculation/) | Add a risky immunizing procedure, with separate treatment mortality and transmission consequences. |
| **Seventeenth century**; isolated quinine from **1820** | Cinchona treatment followed by more standardized antimalarial medication. [The James Lind Library](https://www.jameslindlibrary.org/articles/evaluating-cinchona-bark-and-quinine-for-treating-and-preventing-malaria/) | Add a narrow indication, then improved potency control and reproducibility. |
| **1796** | Jenner’s landmark cowpox-based vaccination experiment. [CDC](https://www.cdc.gov/smallpox/about/history.html) | Add a safer immunization pathway distinct from variolation; usable vaccine supply remains necessary. |
| **1846** | Landmark public demonstration of ether anesthesia, followed by rapid diffusion. [The Royal College of Anaesthetists](https://rcoa.ac.uk/about-us/heritage/history-anaesthesia) | Reduce procedural pain and expand feasible operations—not infection risk automatically. |
| **1847**; **1860s–1880s** | Maternity hand disinfection, Listerian antisepsis, and development of microbial explanations of infection. [Springer](https://link.springer.com/article/10.1007/s10654-022-00871-8) | Add empirical cleanliness protocols, then broader contamination control, testing, and knowledge transfer. |
| **1930s–1940s** | Sulfonamides and development of penicillin treatment and mass production. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.2.2.118) | Reduce outcomes for susceptible infections, conditional on diagnosis, supply, and delivery. |
| **1960s–1970s development and deployment**, including the 1971 refugee program | Modern oral-rehydration service capability. [Johns Hopkins University](https://pure.johnshopkins.edu/en/publications/oral-fluid-therapy-of-cholera-among-bangladesh-refugees-5) | Make a large disease-specific improvement possible with relatively simple materials and trained delivery. |

The key design implication is that **material complexity and historical discovery date are not the same thing**. A counterfactual society could discover an effective, simple fluid-replacement protocol earlier. Conversely, a wealthy society could possess elaborate medical institutions while retaining ineffective treatments.

---

## 4. A capability ladder for TCE

Use the ladder for player-facing summaries, but store a **capability vector** internally:

`nursing | birth care | procedures | medicines | prevention | diagnosis`

The following is a proposed service classification, not a ranking of historical cultures.

| Service level | Available services | Operational requirements | Important remaining limits |
| --- | --- | --- | --- |
| **0 — Household support** | Feeding, water delivery, rest, warmth, assistance with disability | Available caregivers and household resources | No guaranteed specialist knowledge; care can fail during household-wide illness |
| **1 — Local practical care** | Experienced birth attendance, selected wound and fracture care, a small remedy repertoire | Apprenticeship, tools, local supplies | Highly variable practice; limited rescue of severe complications |
| **2 — Organized care** | Dedicated nursing, dispensary, trained attendants, records, referral, convalescence | Funding, staffed facilities, supply management | Organization alone does not confer antimicrobial or surgical effectiveness |
| **3 — Targeted prevention and infection control** | Clean delivery, instrument and hand hygiene, isolation, particular immunizations | Adopted protocols, reliable supplies, public cooperation | Prevention remains pathogen- and procedure-specific |
| **4 — Safer procedural and emergency services** | More complex surgery and obstetric intervention, anesthesia, sustained aftercare | Complementary skills, infection control, materials, timely access | Major hemorrhage, severe infection, and organ failure can remain poorly treatable |
| **5 — Reliable specific treatment** | Effective antimicrobial and antimalarial regimens, rehydration services, broader vaccination | Standardized production, logistics, diagnosis, completion of treatment | Treatment mismatch, delayed arrival, stockouts, and adverse events |
| **6 — Integrated advanced care** | Diagnostics, transfusion, emergency networks, intensive monitoring, chronic treatment | Specialized teams and dependable infrastructure | High cost, capacity constraints, unequal access, and treatment-resistant disease |

**Do not require every preceding row for every later capability.** A settlement may combine level-2 institutions, an effective level-3 vaccine, and poor surgery. Rehydration should not require an intensive-care hospital. Public quarantine should not require a physician profession.

A useful implementation is a sparse prerequisite graph: some capabilities require materials and skills; others require coordination or accumulated evidence. The displayed “level” can summarize this graph without controlling it.

---

## 5. Modeling recommendation

### 5.1 V1: retain healer availability, but make its effect conditional

Let \(a\_{i,d}\) be the probability that person \(i\), with condition \(d\), receives appropriate and sufficiently complete care. Let \(p^0\_{i,d}\) and \(p^1\_{i,d}\) be episode-death probabilities without and with that care.

Then:

\[
p\_{i,d}
=
(1-a\_{i,d})p^0\_{i,d}+a\_{i,d}p^1\_{i,d}
=
p^0\_{i,d}(1-a\_{i,d}e\_d)
\]

where \(e\_d=1-p^1\_{i,d}/p^0\_{i,d}\).

Your healer-availability statistic should primarily determine **access \(a\)**. The settlement’s protocols determine **effectiveness \(e\)**. Keep treatment injuries and newly acquired infections separate from the original episode.

A compact v1 implementation needs only a few additional flags: practical birth care, clean procedures, specific remedies, immunization, and isolation. That is much safer than allowing healer availability to reduce every disease’s fatality equally.

**Worked example—illustrative, not historical:** suppose 1,000 eligible episodes would cause 50 deaths. Care reaches 60% of patients and reduces their risk by 20%:

\[
50[1-(0.60)(0.20)]=44.
\]

That is a **12% reduction among eligible deaths**. If those episodes account for one quarter of all baseline deaths, the direct all-cause reduction is only **3%**, before considering adverse effects or indirect consequences.

This is why a meaningful clinical improvement need not create a large universal longevity bonus.

### 5.2 Do not apply treatment twice to a historical baseline

Historical mortality tables usually describe populations already receiving some care. They are not untreated control groups.

Under the simplified mixture model, a reference probability \(p\_{\mathrm{ref}}\), observed at reference coverage \(a\_{\mathrm{ref}}\), can be adjusted as:

\[
p(a)=p\_{\mathrm{ref}}
\frac{1-ae}{1-a\_{\mathrm{ref}}e}.
\]

This is a modeling identity under its assumptions—not an empirical law. It requires comparable case mix and treatment effects. For more detailed simulation, calculate within severity and demographic strata rather than assuming care is randomly distributed.

Also avoid combining a complete historical all-cause death hazard with independently simulated lethal disease episodes. Either remove the explicitly modeled causes from the residual hazard or calibrate their joint total.

### 5.3 Compute capacity from work, not from arbitrary thresholds

For each service category, estimate:

\[
\text{coverage capacity}
=
\min\!\left(1,
\frac{\text{available provider-hours}}
{\text{required provider-hours}}
\right).
\]

Allocate capacity through actual eligibility, travel, fees, trust, priority, and waiting time. A settlement can have adequate annual staffing but still fail during a cluster of births, battle injuries, or an epidemic.

A provider should become unavailable when ill, exhausted, displaced, or assigned elsewhere. Nursing and transport can be delegated; diagnosis or a difficult procedure may require a particular specialist.

For childbirth, use the timing of labor and complications, not merely annual births per midwife. For quarantine, use detained-person-days and guards, supplies, and accommodation.

### 5.4 Later versions: explicit episodes, providers, and institutions

A practical data decomposition is:

| Entity | Important state |
| --- | --- |
| **Health episode** | True condition, severity, onset, infectiousness, dehydration or blood-loss state where relevant, treatment history, residual disability |
| **Patient’s knowledge** | Perceived symptoms, believed diagnosis, expected benefit, trust, willingness and ability to seek care |
| **Provider** | Skills by task, known protocols, diagnostic ability, schedule, infection status, fees and affiliations |
| **Institution** | Admission rules, staffed capacity, supplies, cleanliness practices, contact environment, funding and referral links |
| **Treatment protocol** | Indications, contraindications or incompatibilities, resource consumption, time, outcome effects, adverse-event model |
| **Public-health authority** | Detection capacity, restriction policies, enforcement, material support, and legitimacy |

The simulation can know the true pathogen while the citizen and healer do not. This allows misdiagnosis, inappropriate remedies, late referral, and learning without hard-coded irrationality.

Use event-driven updates for onset, appointments, procedures, deterioration, and recovery. Use place-based transmission and local service searches rather than comparing every patient with every provider.

For fatality, use competing transition hazards or carefully updated episode outcomes. **Do not reroll a whole-episode case-fatality probability every day.**

### 5.5 Model public-health effects through timing

For isolation, a useful upper-bound approximation to the fraction of transmission prevented is:

\[
f\_{\mathrm{detected}}\,
c\_{\mathrm{compliant}}\,
(1-r\_{\mathrm{contacts}})\,
F\_{\mathrm{remaining}},
\]

where \(F\_{\mathrm{remaining}}\) is the fraction of infectiousness still ahead when isolation begins.

Even perfect isolation performs poorly when recognition occurs late.

For a cordon, reduce infected arrivals rather than applying a universal internal transmission multiplier. In a simple importation model with successful introductions occurring at rate \(\Lambda\),

\[
P(\text{no introduction by }T)=e^{-\Lambda T}.
\]

This produces useful outcomes naturally: delayed invasion, occasional complete exclusion, or eventual failure despite substantial protection. Quarantined cohorts should be able to expose one another; new exposure must affect release risk rather than leaving everyone on an unchanged countdown.

### 5.6 Existing models worth borrowing from

**The Lives Saved Tool, or LiST**, explicitly combines intervention coverage, cause-specific effectiveness, and the share of mortality sensitive to an intervention. Its structure is an excellent analogue for v1, although its modern maternal and child-health defaults should not be transplanted into ancient populations. [The Lives Saved Tool](https://www.livessavedtool.org/about)

**FRED**, described by Grefenstette and colleagues in 2013, provides a relevant agent-based precedent for synthetic populations, social contacts, infectious disease, and mitigation behavior. Borrow its separation of population structure and intervention mechanisms rather than treating it as a historical-medicine calibration dataset. [Fred](https://fred.publichealth.pitt.edu/about)

For TCE, the most useful combination is **LiST-like accounting of targeted benefits inside a FRED-like world of people, places, and contacts**.

---

## 6. Stylized facts and validation targets

A correct simulation should reproduce these patterns without scripted historical outcomes.

| Pattern | Validation target |
| --- | --- |
| **Practical care can be useful before modern science.** | Permit rare survival after major prehistoric procedures and local traditions with substantially better procedural outcomes, without giving all early societies high surgical survival. [Nature](https://www.nature.com/articles/s41586-022-05160-8) |
| **More medical activity can initially make an institution more dangerous.** | A contaminated maternity or surgical service can show high mortality; infection-control adoption can then create a large discontinuity without a new curative drug. [Springer](https://link.springer.com/article/10.1007/s10654-022-00871-8) |
| **Birth-care improvements need not improve every infant outcome.** | Maternal, stillbirth, and neonatal responses should be independently calibratable, consistent with the differing Swedish and Pakistani findings. [OUP Academic](https://academic.oup.com/jeea/article/19/4/2052/6217426) |
| **A breakthrough acts on particular causes.** | Reproduce much larger proportional changes in susceptible diseases than in unrelated mortality, as in the US sulfonamide estimates. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.2.2.118) |
| **Prevention can outperform treatment for a particular infection.** | Smallpox vaccination should sharply change incidence when coverage and supply are adequate; its mechanism must differ from improving survival after infection. [CDC](https://www.cdc.gov/smallpox/vaccines/index.html) |
| **Timing changes epidemic shape.** | Early layered controls can greatly reduce peaks, while late action or premature relaxation can perform much worse. Do not equate a lower peak with the same proportional reduction in total deaths. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0610941104) |
| **Simple services can be highly effective.** | A adequately supplied rehydration service should outperform an impressive but therapeutically ineffective institution for the relevant cases. [Johns Hopkins University](https://pure.johnshopkins.edu/en/publications/oral-fluid-therapy-of-cholera-among-bangladesh-refugees-5) |

For your own experiments, add sensitivity tests for simultaneous caregiver illness, loss of one unusually skilled practitioner, medicine stockouts, increased travel time, and exclusion of poor patients. These are proposed system tests rather than historical effect estimates.

Evaluate not only deaths but also untreated pain, disability, patient work lost, caregiver work lost, waiting time, access by social position, and healthcare-generated infections. Otherwise, valuable care that does not change survival will disappear from the simulation’s accounting.

---

## 7. Sources, datasets, and remaining uncertainty

The intervention table identifies the central scholarly studies. For calibration and reproducibility, the following data resources serve different purposes:

| Resource | Appropriate use | Main caution |
| --- | --- | --- |
| **Human Mortality Database** | Age- and sex-specific population mortality and demographic consistency checks. [Human Mortality Database](https://www.mortality.org/) | Observed populations already include their prevailing care; coverage is not a representative sample of all historical societies. |
| **WHO Mortality Database** | Reported cause-specific mortality, with the visualization system providing data from **1950 onward**. [World Health Organization](https://www.who.int/data/data-collection-tools/who-mortality-database) | Registration completeness and cause classification vary; not an ancient baseline. |
| **Project Tycho** | Disease-count series for epidemic dynamics and vaccination-era comparisons. [Tycho](https://www.tycho.pitt.edu/) | Case counts are affected by surveillance and reporting; they do not directly identify treatment efficacy. |
| **Historical study supplements and replication materials** | Reconstruct the actual comparison behind an estimate: Swedish midwife variation, sulfonamide diffusion, or Semmelweis’s clinic records. [OUP Academic](https://academic.oup.com/jeea/article/19/4/2052/6217426) | Preserve original populations, periods, denominators, and uncertainty. |
| **LiST intervention documentation and underlying survey sources** | Later-stage condition-specific treatment packages and coverage definitions; its inputs include DHS and MICS surveys. [The Lives Saved Tool](https://www.livessavedtool.org/about) | Contemporary effectiveness and service packages are not direct evidence about preindustrial practice. |

### What is well supported—and what is not

The evidence supports **large, specific improvements** from safer maternity practice, infection control, particular immunizations, effective pharmaceuticals, and organized rehydration. It also supports substantial variation in institutional function and practical skill. It does **not** establish a universal survival benefit for folk healing, learned physicians, monasteries, or hospitals as broad categories. The appropriate coefficients depend on what was actually done. [Springer](https://link.springer.com/article/10.1007/s10654-022-00871-8)

Several recurring evidentiary traps deserve explicit flags in your source registry. Healed bones select survivors; therapeutic texts describe recommendations rather than outcomes; hospital patients differ from people treated at home; before-and-after series can change in case mix; and bundles do not reveal the contribution of every component. These problems are visible in the archaeological, surgical, maternity, and trial evidence above. [ResearchGate](https://www.researchgate.net/publication/324083662_Trepanation_ProceduresOutcomes_Comparison_of_Prehistoric_Peru_with_Other_Ancient_Medieval_and_American_Civil_War_Cranial_Surgery)

For each authored parameter, store the evidence type, population, date, outcome definition, and whether it is **measured, inferred, or purely proposed**. Thin evidence should produce wider sensitivity ranges, not confidently invented historical averages.

**Recommended minimum for TCE:** keep healer availability as an access variable; make benefits condition-specific; represent ordinary caregiving through real resources; separate birth care from general treatment; allow unsafe care to cause new harm; and keep prevention outside the generic mortality modifier. That combination gives v1 modest, interpretable effects while preserving the causal structure needed for a later full healthcare service simulation.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92827-9980-83ea-8d30-add3f79541a7)
