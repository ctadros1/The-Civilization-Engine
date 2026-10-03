# Education, apprenticeship and literacy: a simulation-ready report for TCE

## Core recommendation

**Model education as people acquiring particular capabilities through work, instruction and access to knowledge—not as a settlement-wide “education level.”**

TCE needs at least four distinguishable outcomes: practical competence, literacy, numeracy and advanced domain knowledge. School attendance and credentials should be recorded separately from those outcomes. This allows an excellent illiterate craftsperson, a literate but inexperienced administrator, and a highly educated religious scholar with little engineering knowledge.

For v1, retain those individual capabilities but abstract the institutions supplying instruction. **Learning should still consume time and scarce teaching capacity.** Later, replace the abstract providers with actual schools, workshops, teachers, travel and attendance without changing the underlying skill system.

---

## 1. Mechanisms: causal rules the simulation can implement

### 1.1 Learning begins outside schools

Research on contemporary hunter-gatherer communities documents learning through observation, participation, play, peers, parents and other adults. Basic subsistence activities are often learned during childhood; difficult hunting and manufacturing skills can continue developing through adolescence and adulthood. Direct teaching exists alongside less explicit learning. Contemporary communities are informative comparisons, not unchanged replicas of prehistoric populations. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5662667/)

**Simulation rule:** Any relevant activity can produce experience, including household work and supervised participation. Children should encounter the activities actually available around them, rather than receiving a generic annual skill allocation.

Distinguish three channels:

| Channel | Implementation |
| --- | --- |
| Household transmission | Learn from the skills, activities and available attention of household members. |
| Peer and community learning | Learn through shared activities and observation within local social networks. |
| Deliberate instruction | A teacher or master allocates attention to specific skills and prerequisites. |

A settlement without schools should therefore remain capable of reproducing sophisticated farming, building, navigation, ecological and craft knowledge.

### 1.2 Apprenticeship combines production, instruction and finance

Apprenticeship is not simply schooling inside a workshop. Wallis’s research on premodern England emphasizes that apprentices performed useful work while learning, and that many arrangements ended before their nominal contractual term. Training and repayment through labor could overlap rather than occurring in two cleanly separated phases. [cambridge.org](https://www.cambridge.org/core/journals/journal-of-economic-history/article/apprenticeship-and-training-in-premodern-england/9F138B4E456FE2AF7D2DC4584C942E71)

**Simulation rule:** An apprenticeship contract should specify a master, craft, expected duration, compensation or fees, maintenance obligations, permitted tasks and termination conditions. Actual skill growth follows exposure—not contract age.

The apprentice’s productive contribution should increase with competence. The master pays through teaching time, mistakes, materials and sometimes accommodation, while receiving labor and possibly fees. Consequently, both parties may rationally continue, renegotiate or terminate an arrangement.

Keep **skill, completion and permission to practice independently** as separate variables. Leaving an apprenticeship should not erase acquired competence; completing one should not guarantee mastery.

### 1.3 Access depends on household resources and institutional rules

Education provision can be limited by both household affordability and public choices. Chaudhary’s research on British India finds that schooling reflected funding arrangements, social hierarchy and elite preferences, not simply a shortage of interest in learning. Around 1911, British India had fewer than three primary schools per ten villages. [IDEAS/RePEc](https://ideas.repec.org/a/cup/jechis/v69y2009i01p269-302_00.html)

**Simulation rule:** A household evaluates the expected benefits of training against fees, materials, travel and foregone production. Benefits can include earnings, office eligibility, religious value, status and better household administration.

Do not give agents perfect forecasts. Let expectations depend on visible wages, successful relatives, occupational vacancies and local norms.

Eligibility restrictions should be institutional: sex, legal status, caste or estate membership, sponsorship, entrance requirements, language and fees. These restrictions can change. They should not become inherited differences in learning ability.

### 1.4 Teachers, curricula and teaching time constrain expansion

Historical education did not always depend on a dedicated school building or standardized degree. In medieval Cairo, Berkey describes knowledge transmission organized substantially around relationships with particular scholars and their authority to teach or certify learning. Buildings and endowments supported this activity without wholly determining it. [JSTOR](https://www.jstor.org/stable/j.ctt7zvxj4)

**Simulation rule:** A provider needs a qualified instructor, an available curriculum, teaching time and sufficient operating resources. A building alone supplies none of these.

Teacher competence and teaching effectiveness should be distinguishable. An exceptional smith may have little patience or time for instruction. A competent teacher may efficiently transmit elementary material without possessing advanced expertise.

Institutional growth also requires reproducing instructors. An abrupt expansion in funded places can therefore encounter a teacher bottleneck.

### 1.5 Literacy is a bundle of task-specific abilities

Historical measurements frequently identify signatures rather than reading comprehension. Reading and writing were not always learned together, and signing a name does not establish fluent writing. Comparisons across sources can therefore conceal substantial differences in what “literate” means. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/6C8C904F5F80CC64DAF6AF21A8A647EF/S0018246X00010918a.pdf/levels-of-illiteracy-in-england-1530-1730.pdf?utm_source=chatgpt.com)

**Simulation rule:** Separate at least elementary reading from writing, and attach literacy to a language–script combination. Numeracy should not require literacy.

Useful operational tests are:

| Capability | Example task |
| --- | --- |
| Elementary reading | Understand a short familiar message. |
| Functional reading | Interpret instructions, correspondence or an unfamiliar transaction. |
| Writing | Compose a message or record rather than reproduce a signature. |
| Numeracy | Count, measure, calculate quantities and check exchanges. |
| Advanced textual competence | Interpret specialized legal, mathematical, scholarly or technical material. |

Also allow **delegated literacy**: an otherwise illiterate person can pay a scribe or rely on a literate associate. Mesopotamian scribes supplied precisely such services for property transfers, loans and disputes. Universal personal literacy is therefore not a prerequisite for written transactions. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/1476493/2/Robson_c05-2015-07-27.pdf)

### 1.6 Education affects different economic outcomes through different channels

**Current production.** Practical competence should improve task execution, quality and error rates. Literacy should matter where the task uses text, documentation or formal calculation—not provide an identical bonus to every activity.

**Technology adoption.** A worker may need instruction, relevant foundational knowledge and practice with new equipment. Existing expertise should transfer partially where tasks share components.

**Innovation.** Basic literacy and advanced knowledge need not have the same effect. Squicciarini and Voigtländer’s study of France associates the local concentration of advanced knowledge—proxied by subscriptions to the *Encyclopédie*—with subsequent industrial growth. Basic literacy was associated with income levels but did not show the same relationship with growth. This supports separating broad elementary capabilities from small populations of advanced specialists; it does not establish a universal innovation coefficient. [SSRN](https://papers.ssrn.com/sol3/Delivery.cfm/nber_w20219.pdf?abstractid=2450919&mirid=1)

**State capacity.** Education can increase the supply of recordkeepers, legal specialists and administrators. Cantoni and Yuchtman link medieval German university expansion to commercial development, with legal training and institutions central to their interpretation. Universities should therefore be able to improve administration and contracting without being automatic invention factories. [OUP Academic](https://academic.oup.com/qje/article-abstract/129/2/823/1869698)

The causal relationship between mass education and industrialization remains contested. Becker, Hornung and Woessmann find an important role for education in Prussian industrial catch-up; Edwards’s replication challenges the identification and robustness of that conclusion. Do not encode either “literacy causes industrialization” or “literacy is irrelevant” as an unconditional rule. [IZA](https://www.iza.org/publications/dp/4556/catch-me-if-you-can-education-and-catch-up-in-the-industrial-revolution)

---

## 2. Quantitative parameters and historical calibration anchors

**Confidence terminology:** **High** means reasonably secure for the stated source population and measurement. **Medium** indicates a useful but imperfect proxy or restricted sample. **Low** indicates a rough reconstruction. None implies that the number transfers unchanged to another society.

### 2.1 Apprenticeship durations and outcomes

| Context | Observed quantity | Value and units | Appropriate interpretation | Source; confidence |
| --- | --- | --- | --- | --- |
| Early modern England | Standard contractual apprenticeship term | **7 years** | A contractual benchmark, not seven years of pure instruction or a biological mastery threshold. | Minns et al., *Contracting for Apprenticeship in Early Modern Europe*; **High** for the institutional rule. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf) |
| Early modern Amsterdam | Observed contractual terms | **8 months–6 years** | Considerable variation within one city; avoid a universal European duration. | Minns et al.; **Medium–high**, sampled contracts. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf) |
| Early modern Paris, seamstresses | Apprenticeship term | **3 years** | Craft-specific institutional calibration. | Minns et al.; **Medium–high**. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf) |
| Southern Tanzania, car mechanics, modern informal apprenticeship | Apprentices’ reported training duration | **Mean 34 months; standard deviation 17 months** | The standard deviation describes dispersion, not a confidence interval. | ILO/Hofmann resource guide, drawing on field research; **Medium**, local sample. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40africa/%40ro-abidjan/documents/publication/wcms_171393.pdf) |
| Egyptian informal-apprenticeship study | Dropouts relative to completions | **1.3 dropouts per completion** | Evidence against assuming universal completion; not an annual dropout probability. | ILO/Hofmann; **Medium**, study-specific. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40africa/%40ro-abidjan/documents/publication/wcms_171393.pdf) |

These observations constrain **contract structures and elapsed training histories**, not a universal skill-growth curve. The evidence assembled here is much thinner on standardized output per additional practice hour.

### 2.2 Literacy and participation benchmarks

These figures should be reproduced only in appropriately configured comparison scenarios—not assigned by simulation era.

| Place and period | Measure and denominator | Benchmark | Confidence and comparability |
| --- | --- | --- | --- |
| England, around 1800 | Primarily signature-based estimates for men and women | Approximately **60% male, 40% female** | **Medium**; signature evidence is not a comprehension test. De Pleijt’s reconstruction draws on historical literacy research. [DNB](https://d-nb.info/1122926669/34) |
| China, later nineteenth century | Reconstructed basic literacy; not a standardized census denominator | Approximately **30–45% male, 2–10% female** | **Low**; Rawski’s estimates, reproduced by Xu, Földvári and van Leeuwen. Treat as broad bounds, not precise national observations. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/43525/1/MPRA_paper_43525.pdf) |
| United States, 1870 | Reported ability to read and write, population **14+** | **80.0%** overall; **88.5%** in the historical “White” category and **20.1%** in “Black and other” | **High** for the published historical series; categories and self-reporting limit interpretation. Literacy values are complements of reported illiteracy. [National Center for Education Statistics](https://nces.ed.gov/naal/lit_history.asp) |
| India, 1951 | Census literacy, population **5+** | **18.3%** total; **27.2% male, 8.9% female** | **High** for the reported census measure; not directly harmonized with later age thresholds. [MOSPI](https://www.mospi.gov.in/sites/default/files/publication_reports/Women%20and%20Men%20%20in%20India%202018.pdf?pfrom=home-poltics) |
| India, 2011 | Census literacy, population **7+** | **73.0%** total; **80.9% male, 64.6% female**; rural **66.8%**, urban **84.1%** | **High** for the reported measure; changing the denominator matters when comparing with 1951. [MOSPI](https://www.mospi.gov.in/sites/default/files/publication_reports/Women%20and%20Men%20%20in%20India%202018.pdf?pfrom=home-poltics) |
| Japan, 1895 | Compulsory-school-age attendance | **61.2%** overall; boys **76.7%**, girls **43.9%** | **High** for the ministry’s historical series. **Attendance, not literacy.** [MEXT](https://www.mext.go.jp/b_menu/hakusho/html/others/detail/1317322.htm) |
| Japan, 1905 | Same attendance series | **95.6%** overall; boys **97.7%**, girls **93.3%** | **High** for the reported series. Demonstrates rapid participation expansion and narrowing sex differences. [MEXT](https://www.mext.go.jp/b_menu/hakusho/html/others/detail/1317322.htm) |
| World, 1975 and 2024 | UNESCO/UIS adult literacy, **15+** | Approximately **65% → 88%** | **Medium–high** as an international aggregate; underlying country measurements vary. [UNESCO](https://www.unesco.org/en/literacy/need-know) |

**Class and status require separate calibration.** A society’s overall percentage is insufficient: a small educated professional stratum can coexist with low popular literacy. In India’s 2011 census series, literacy was **66.1% among Scheduled Castes** and **59.0% among Scheduled Tribes**, compared with 73.0% overall. These are context-specific social categories, not interchangeable with income classes or innate group characteristics. [MOSPI](https://www.mospi.gov.in/sites/default/files/publication_reports/Women%20and%20Men%20%20in%20India%202018.pdf?pfrom=home-poltics)

### 2.3 Economic-return and learning-quality anchors

| Finding | Quantitative evidence | TCE implication |
| --- | --- | --- |
| Education is associated with substantial private earnings returns | Psacharopoulos and Patrinos review **1,120 estimates across 139 countries**, finding an average private return around **9% per additional year of schooling**. | A useful external wage-pattern check, **not** a universal physical-productivity multiplier or an annual growth rate. Selection, credentials, institutions and local labor demand matter. [World Bank](https://documents1.worldbank.org/curated/en/442521523465644318/pdf/WPS8402.pdf) |
| Schooling attainment and learning are different quantities | Angrist et al. assemble harmonized learning outcomes for **164 countries, 2000–2017**. | Calibrate attendance, attainment and demonstrated competence separately. More school years should not guarantee identical learning across providers. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8046667/) |

---

## 3. Variation across eras and world regions

### Foragers and early farming communities

Forager evidence supports extensive education without classrooms: observation, participation, play and instruction are combined differently by activity and community. Complex expertise may develop well beyond childhood. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5662667/)

For TCE’s early farming start, **initialize adults with substantial practical knowledge**. The absence of writing should set textual literacy to zero, not farming, construction or ecological competence. Make learning responsive to task calendars: repeated exposure to a complete agricultural cycle is different from repeating one operation.

### Ancient Southwest Asia: specialized scribal training

At Nippur, archaeological evidence from a small house used for scribal instruction in the eighteenth century BCE includes elementary exercises, mathematical material and literary compositions. Robson’s analysis emphasizes copying, memorization and preparation for scribal work—not a modern school system serving the whole population. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/1476493/2/Robson_c05-2015-07-27.pdf)

**Model implication:** Allow a small instructor household or institution to train specialist recordkeepers. Temples, palaces and households can sponsor education, but do not require every early school to be a temple or infer population literacy from surviving tablet counts.

### China and Japan: different routes within East Asia

Qing China combined popular elementary instruction with a much more demanding examination-oriented educational path. The resources and time devoted to elite classical attainment were not equivalent to universal elementary schooling or technical training. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/43525/1/MPRA_paper_43525.pdf)

Japan’s modern attendance expansion illustrates another mechanism: changes in finance and regulation can rapidly alter access. The ministry’s history identifies tuition arrangements as an early obstacle; the 1900 regulations established free compulsory elementary schooling in the public system. [MEXT](https://www.mext.go.jp/b_menu/hakusho/html/others/detail/1317321.htm)

**Model implication:** Separate elementary provision, examination preparation, credentials and technical instruction. The reward for passing an examination can redirect learning toward an official curriculum rather than toward the most productive craft.

### Islamic societies: scholarly networks and endowed provision

In medieval Cairo, advanced religious education combined patron-funded institutions with personal teacher–student relationships and certification through the *ijāza*. Political and military elites were important founders and financial supporters. These arrangements cannot be represented adequately by “university building produces research points.” [JSTOR](https://www.jstor.org/stable/j.ctt7zvxj4)

**Model implication:** Endowments can fund instruction independently of household fees or annual state budgets. Scholarly reputation, personal certification and educational travel can matter as much as institutional rank.

### South Asia: plural traditions and unequal provision

Research on medieval India describes multiple settings for learning, including Sanskrit scholarly centers, Islamic educational networks and royal or private collections. New patrons and institutions did not simply replace all previous forms. [ResearchGate](https://www.researchgate.net/publication/277225365_Education_and_transmission_of_knowledge_in_medieval_India)

**Model implication:** Permit overlapping curricula, languages, patronage networks and access rules within a settlement. Colonial and modern expansion should alter financing and eligibility rather than erase household, religious or vocational transmission.

### Africa: apprenticeship and multiple literacies

Modern African informal-apprenticeship studies demonstrate the continued importance of workshop learning outside formal vocational schools, with substantial differences between trades and places. They should inform institutional mechanisms, not be projected backward as ancient African training rates. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40africa/%40ro-abidjan/documents/publication/wcms_171393.pdf)

Ngom’s study of *Ajami*—African languages written in Arabic-derived scripts—also shows why “not literate in the official European language” must not mean “unable to read or write.” His manuscript research concerns a specific West African religious and linguistic setting, not a single continental literacy system. [OUP Academic](https://academic.oup.com/book/5308)

**Model implication:** Track the actual languages and scripts people use; make instructional-language mismatch an access problem, not a population learning penalty.

### Indigenous Americas: administration without alphabetic literacy

Urton and Brezine’s analysis of Inca khipus identifies linked accounting records and hierarchical administrative aggregation. This is a direct warning against requiring alphabetic literacy for all complex recordkeeping. [Academia](https://www.academia.edu/54913379/Khipu_Accounting_in_Ancient_Peru)

**Model implication:** Recordkeeping technologies can include knot records and other authored systems. Train competence in those systems separately from alphabetic reading.

### Industrial and modern settings

England’s historical record does not support treating the beginning of industrialization as an instant transition to universal literacy. Around 1800, the signature-based estimates remained approximately 60% for men and 40% for women. [DNB](https://d-nb.info/1122926669/34)

For later development, allow mass basic education, specialized vocational instruction, advanced scholarship and workplace retraining to coexist. New machines may reduce the skills required for some operations while creating new maintenance or design requirements. This should be a property of each technology’s task structure, not an automatic increase in every occupation’s education requirement.

---

## 4. Stylized facts a correct simulation should reproduce

| Pattern | Validation target |
| --- | --- |
| **Extensive competence without literacy** | A non-writing population must reproduce complex practical skills. Lack of classrooms cannot cause general technological amnesia. The forager learning literature supports this distinction. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5662667/) |
| **Contract length differs from learning achieved** | Simulated apprentices should acquire useful competence before completion, and early leavers should retain it. Some completed apprentices should remain less capable than experienced non-completers. [cambridge.org](https://www.cambridge.org/core/journals/journal-of-economic-history/article/apprenticeship-and-training-in-premodern-england/9F138B4E456FE2AF7D2DC4584C942E71) |
| **Large inequalities can exist inside one society** | Under appropriately unequal access rules, reproduce differences comparable in scale to the sex, rural–urban and status gaps in the historical benchmarks—not identical predetermined group percentages. |
| **Participation can rise faster than adult literacy** | A Japan-like scenario can expand school participation sharply over a decade, while older cohorts retain their earlier educational histories. Attendance must not instantly rewrite adult capabilities. [MEXT](https://www.mext.go.jp/b_menu/hakusho/html/others/detail/1317322.htm) |
| **Schooling does not guarantee learning** | Poor instruction, low attendance or unsuitable curricula can yield additional enrollment without equivalent competence. Test these outputs separately. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8046667/) |
| **Advanced specialists and broad literacy have different effects** | A small concentration of advanced knowledge can affect particular industries before mass literacy becomes universal; general reading ability alone must not guarantee invention. [SSRN](https://papers.ssrn.com/sol3/Delivery.cfm/nber_w20219.pdf?abstractid=2450919&mirid=1) |
| **Administrative competence can be concentrated** | A state should function with a minority of specialist recordkeepers and many nonliterate residents; alphabetic literacy must not be its only possible recording technology. [Academia](https://www.academia.edu/54913379/Khipu_Accounting_in_Ancient_Peru) |

Additional **engineering tests**, rather than claims about universal historical frequencies, should include recovery after losing a teacher, skill transfer after migration, interrupted apprenticeships, adult retraining and declining competence when a production method becomes unfamiliar.

---

## 5. Recommended TCE representation

### 5.1 Agent and institution state

Use sparse records rather than a large universal skill vector for every person.

| Entity | Minimum useful state |
| --- | --- |
| Person | Relevant practical skills; literacy by language/script; numeracy; advanced knowledge in relevant domains; learning exposure; enrollment or mentor link; credentials. |
| Household | Available skilled instructors; teaching time; educational preferences; resources; children’s and adults’ time budgets. |
| Workshop | Master and workers; known procedures; equipment; apprentice capacity; contracts; access to new techniques. |
| Educational provider | Curriculum; instructor competence; teaching capacity; fees or subsidy; eligibility; instructional language; resources. |
| Knowledge collection | Available texts or records, their domains, language/script and accessibility. |
| Government or patron | Training demand, funding, examination rules, recognition of qualifications and access restrictions. |

A credential should be a **social or legal claim about competence**, not competence itself. Institutions may recognize another institution’s credentials fully, partially or not at all.

A technology should also have several states: **known somewhere, available locally, understood by particular people, and operational with current workers and equipment**.

### 5.2 A bounded learning rule

For a practical skill or knowledge component, a useful starting model is:

\[
s'\_{i,d}
=
s\_{i,d}
+
\max(0,c\_{i,d}-s\_{i,d})
\left(1-e^{-k\_dE\_{i,d}}\right)
\]

Here:

* \(s\_{i,d}\) is the person’s current competence, scaled from 0 to 1.
* \(c\_{i,d}\) is the level currently accessible through known methods, instructors, materials and prerequisites.
* \(E\_{i,d}\) is effective learning exposure during the update.
* \(k\_d\) determines how quickly the person closes the accessible gap.

The `max` prevents losing an instructor from immediately reducing existing skill. A new method or successful experiment can raise the accessible frontier; the formula is for learning available knowledge, not inventing it.

Define exposure from **actual allocated hours**, adjusted for instruction quality, relevant practice, language fit and coverage of necessary operations. Keep activity categories mutually exclusive: an hour of mentored production can produce both output and learning, but must not be counted as two hours of the person’s time.

A convenient calibration parameter is:

\[
k\_d=\frac{\ln 5}{T\_{80,d}}
\]

where \(T\_{80,d}\) is the exposure required to close 80% of the available skill gap. This exponential form is a transparent engineering choice—not an established universal human learning law.

### 5.3 Starting priors where direct historical measurements are missing

**Everything in this table is a proposed calibration prior, not a measured historical constant.** The historical evidence above provides plausibility checks, especially for training duration, but does not identify these exact values.

An “effective training year” means exposure equivalent to one year at an authored reference schedule. Changing that schedule changes the conversion to calendar years. For example, half attendance at otherwise unchanged quality supplies half an effective year.

| Parameter | Suggested starting range | Units | Source/status |
| --- | --- | --- | --- |
| Routine operation: \(T\_{80}\) | **0.1–0.5** | Effective training years | TCE prior; low empirical confidence. |
| Broad craft competence: \(T\_{80}\) | **2–5** | Effective training years | TCE prior, loosely constrained by apprenticeship histories. |
| Difficult, high-level craft competence: \(T\_{80}\) | **5–10** | Effective training years | TCE prior; include task variety, not repetitions alone. |
| Elementary reading and writing: \(T\_{80}\) | **1–3** | Effective instruction years | TCE prior; do not equate this with advanced functional literacy. |
| Advanced domain training after foundations | **4–10** | Effective study/practice years | TCE prior; strongly curriculum-dependent. |
| Active apprentices per master | **1–3**, test up to **5** | Apprentices/master | Capacity prior, not a universal historical ratio. |
| Elementary group instruction | **15–40** | Learners/full-time instructor | Service-design prior; distinguish group lessons from tutoring. |
| Durable literacy loss in baseline v1 | **0** | Automatic annual percentage-point loss | Simplification; model interrupted acquisition before adding uncertain decay. |

These are alternative skill-domain calibrations, **not a compulsory ladder that everyone completes sequentially**.

As a numerical check, with \(T\_{80}=4\), constant access to a frontier of 1 and initial skill 0, competence reaches 0.80 after four effective years and 0.96 after eight. At half exposure, the same four-effective-year result takes eight calendar years.

For seasonal or varied crafts, add an **experience-coverage requirement**. Ten thousand repetitions of one simple operation should not automatically teach diagnosis, uncommon repairs or an entire agricultural cycle.

### 5.4 The abstract v1 system

Use **virtual providers with real resource accounting**.

**First, establish supply.** Literate household members, masters, priests, scribes or other qualified adults can supply instruction. Allocate some of their labor to it. Funding can pay for instruction but cannot create competent teachers instantly.

**Second, establish demand.** Households and adults request training according to expected benefit, preferences, eligibility and affordability. Employers or governments can sponsor training to fill vacancies.

**Third, ration places or teaching hours.** Capacity is limited by instructors, resources and curriculum. Do not distribute an educational bonus to everyone merely because the settlement funds one teacher.

**Fourth, accumulate individual learning.** Apply the skill rule only to people receiving relevant exposure. Attendance interruptions slow progress; they need not erase it.

**Fifth, derive population statistics.** For example:

\[
\text{adult basic literacy}
=
\frac{
\#\{i:\text{age}\_i\ge15,\ i\text{ passes the basic reading/writing criteria}\}
}{
\#\{i:\text{age}\_i\ge15\}
}
\]

Compute signature ability and functional reading separately when validating against sources that measured those outcomes.

**Finally, let demography do its work.** Adult literacy changes through learning, entry into the adult population, deaths and migration. Training 100 new readers changes the literate count by 100; it does not impose a fixed percentage increase on the settlement.

At world creation, seed adult skills from their plausible prior household and occupational experience. Do not make the founding generation relearn agriculture simply because no simulation years have elapsed.

### 5.5 Upgrading to the full service

The later implementation should replace virtual provision with actual:

* instructors and learners attending particular places;
* schedules, travel, materials, fees and attendance;
* workshop contracts, examinations and credential recognition.

The same individual skill records and exposure ledger remain in place. Consequently, adding full educational services does not require converting an old “education score” into invented personal histories.

Different provider types should offer different curricula. A temple school, accounting tutor, craft workshop and advanced academy should not be successive levels of one universal upgrade chain.

At TCE’s population scale, allow institutions to serve **regional catchments**. A specialized academy may draw students from several settlements; every town need not independently support every advanced subject.

### 5.6 Connect learning to production without a blanket bonus

Calculate effective labor from the actual task:

\[
\text{effective labor}
=
\text{task hours}
\times
\text{task competence}
\times
\text{relevant condition modifiers}.
\]

Then apply material, equipment and organizational constraints.

For a nontextual manual task, the initial direct literacy bonus can be **zero**. Reading may still help the worker acquire a new technique from instructions, manage accounts or move into a supervisory role.

Distinguish a shortage of competent workers from a shortage of machinery or materials. Otherwise, education can unrealistically compensate for missing physical inputs.

Administrative throughput should likewise depend on qualified staff time and processing requirements. Wider literacy may simplify interactions, but a small clerical office should still have a finite capacity for contracts, inventories or tax records.

### 5.7 Separate invention, transmission and adoption

Use three processes:

**Invention:** Qualified people attempt improvements or investigations. Both practical experimentation and advanced study can contribute, depending on the technology.

**Transmission:** People encounter methods through work, teaching, migration, correspondence or accessible records.

**Adoption:** A household or organization commits resources and acquires sufficient competence to use the method.

A possible invention-event model is:

\[
P(\text{successful improvement})
=
1-e^{-\lambda\_d X\_d},
\]

where \(X\_d\) is qualified experimental effort, adjusted for relevant knowledge and resources. The parameter \(\lambda\_d\) must be authored and calibrated by domain; the research reviewed here does not support a universal “inventions per literate person” rate.

Keep this local. A method discovered elsewhere should not instantly update every worker. Conversely, losing the last local expert should threaten local operational capability without necessarily deleting surviving records or knowledge elsewhere.

### 5.8 Existing models worth borrowing from

**EURACE/Eurace@Unibi:** Its labor-market modeling distinguishes general education from workplace-specific skills and represents learning toward the requirements of available production technology. Borrow the distinction between transferable foundations and equipment-specific competence, and the bounded learning structure. Its assumption that general education raises learning speed should not become an indiscriminate bonus for every historical craft. [Bielefeld University](https://www.uni-bielefeld.de/fakultaeten/wirtschaftswissenschaften/lehrbereiche/etace/eurace%40unibi/eurace-deliverables/EURACE.Deliverable.D7.2-%281%29.pdf)

**Wallis’s historical-economic apprenticeship model:** Borrow overlapping production and training, distributed costs and returns, and the possibility of early termination. These are more appropriate foundations than a fixed countdown ending in an instant “master craftsman” upgrade. [cambridge.org](https://www.cambridge.org/core/journals/journal-of-economic-history/article/apprenticeship-and-training-in-premodern-england/9F138B4E456FE2AF7D2DC4584C942E71)

For implementation, daily activities can accumulate exposure counters while skill updates run monthly. Iterate over active skills and bounded social contacts, not every possible person–teacher pair. This is a proposed scheduling architecture, not a measured performance claim.

---

## 6. Sources, datasets and important uncertainties

### Recommended research assets

| Resource | Best use | Main limitation |
| --- | --- | --- |
| **Barro–Lee educational attainment dataset** | Cohort-based calibration: the original 1950–2010 dataset covers **146 countries**, with sex and age breakdowns at five-year intervals. | Attainment and years of schooling are not direct skill measurements; some values are reconstructed. [Barrolee](https://barrolee.github.io/BarroLeeDataSet/Aboutdataset/Introduction.html) |
| **Angrist et al., “Measuring human capital using global learning data”** | Calibrating learning outcomes separately from enrollment and completed schooling. | Harmonization does not make every assessment a perfectly equivalent natural unit of skill. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8046667/) |
| **UNESCO Institute for Statistics literacy series** | Modern adult literacy levels and international comparisons. | Definitions, collection methods and observation years vary. [UNESCO](https://www.unesco.org/en/literacy/need-know) |
| **NCES historical U.S. literacy tables; Census of India/MoSPI tables** | Long-run sex, social-group and geographic differences. | Historical categories and age denominators require explicit handling. [National Center for Education Statistics](https://nces.ed.gov/naal/lit_history.asp) |
| **MEXT historical education series** | Enrollment expansion, sex differences and institutional change in Japan. | Participation figures must not be substituted for adult literacy. [MEXT](https://www.mext.go.jp/b_menu/hakusho/html/others/detail/1317322.htm) |
| **Minns et al.; Wallis; ILO/Hofmann apprenticeship studies** | Contracts, training duration, early termination and informal institutions. | Selected trades, places and records; weak coverage of informal learning outside documented arrangements. [Bollettino ADAPT](https://www.bollettinoadapt.it/wp-content/uploads/2019/12/contracting-for-apprenticeship-in-early-modern-europe.pdf) |

### What remains uncertain

**Ancient population literacy is especially difficult to quantify.** Archaeological and textual evidence can reveal training practices and specialist functions without supplying a representative population denominator. For early TCE societies, use explicit scenario assumptions about the number of trained specialists rather than presenting an invented ancient percentage as established fact. Robson’s work is particularly useful for reconstructing mechanisms without making that statistical leap. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/1476493/2/Robson_c05-2015-07-27.pdf)

**Skill acquisition rates are less securely known than contractual durations.** Historical records rarely provide repeated, standardized measures of a trainee’s output while also recording instructional hours, task difficulty and prior experience. Treat the proposed learning coefficients as sensitivity parameters.

**Economic returns are not pure productivity effects.** Wages can reflect credentials, restricted occupational entry and labor-market institutions as well as useful capabilities. The modern average schooling return should therefore validate broad earnings patterns, not directly parameterize output. [World Bank](https://documents1.worldbank.org/curated/en/442521523465644318/pdf/WPS8402.pdf)

**Schooling–industrialization causality is not settled by one famous study.** The Prussian findings and replication disagreement are reasons to model multiple causal pathways and test their relative strength rather than selecting one global education–growth multiplier. [IZA](https://www.iza.org/publications/dp/4556/catch-me-if-you-can-education-and-catch-up-in-the-industrial-revolution)

**Historical institutions were heterogeneous.** “Guild,” “temple school,” “madrasa,” “university” and “public school” should identify configurable organizational forms—not fixed bundles of efficiency, openness or scientific orientation.

### Implementation priority

The highest-value v1 package is **practical learning-by-doing, bounded mentorship, language-specific literacy, numeracy, capacity-limited instruction and cohort-based population statistics**. Add advanced curricula, institutional finance, examinations and detailed knowledge collections later.

That foundation lets TCE generate skilled oral societies, restricted scribal elites, widespread basic literacy, expensive specialist education and educational decline from the same rules—without requiring a scripted sequence of historical eras.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9283f-1ba4-83e9-a308-e21f57060137)
