# Personal and place names across cultures: a simulation-ready design for TCE

## Core recommendation

**Build a system of naming practices, not a universal random-name generator.** A believable name should reflect a relationship, naming decision, linguistic tradition, or historical event—and then persist or change according to its own rules.

For TCE, separate four things:

| Layer | What it determines |
| --- | --- |
| **Social naming system** | Who chooses a name; which components are inherited, relational, commemorative, or freely chosen; when names change. |
| **Language** | Sounds, syllable structures, word formation, grammatical endings, and spelling. |
| **Institutional rules** | Which forms a household, temple, court, registry, guild, or firm recognizes. |
| **Name history** | Earlier forms, alternative names, pronunciation changes, translations, and the event that produced each name. |

Attach these to immutable entity identifiers. **A name is neither a unique identifier nor proof of kinship.** The distinction is consequential: the Korean census data analyzed by Kim and Park distinguish only 288 surname labels in 2000, but 4,188 surname-plus-regional-origin categories. Those categories are still not individual family trees. [arXiv](https://arxiv.org/pdf/cond-mat/0407311)

The strongest evidence concerns documented naming structures, attested historical forms, and modern frequency distributions. The weakest concerns universal rates of name invention, surname adoption, sound change, and place renaming. The parameter tables below distinguish observations from proposed simulation settings.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Personal names: generate the social structure before the spelling

A useful internal representation is an ordered collection of **typed components**, not mandatory “first name / last name” fields.

| Mechanism | Implementable TCE rule | Evidence and qualifications |
| --- | --- | --- |
| **Patronymics and matronymics** | Construct a component from an actual parent’s personal-name reference, using the naming language’s morphology. Recompute for the next generation; do not automatically inherit the resulting string. | Ethiopian documentation describes personal name followed by the father’s—and sometimes grandfather’s—personal name. This is structurally different from a fixed hereditary surname. [GOV.UK](https://www.gov.uk/government/publications/ethiopia-knowledge-base-profile/ethiopia-knowledge-base-profile) |
| **Clan, house, and family names** | Inherit an affiliation or name component according to the applicable descent, adoption, and household rules. Keep clan ID, household ID, genealogical lineage, and surname label separate. | Korean surname-plus-origin distinctions illustrate why one surname field cannot represent all levels of affiliation. [arXiv](https://arxiv.org/pdf/cond-mat/0407311) |
| **Occupational and locational bynames** | Derive a descriptor from an occupation, residence, origin, estate, or landmark. Mark it either **currently descriptive** or **inherited**. Only the former should track subsequent changes. | Occupational family names occur in first-millennium BCE Babylonian material; locative surnames also preserve historical relationships with place names. Neither mechanism is uniquely modern. [Academia](https://www.academia.edu/114073987/Waerzeggers_C_and_Gro%C3%9F_M_eds_2024_Personal_Names_in_Cuneiform_Texts_from_Babylonia_c_750_100_BCE_An_Introduction) |
| **Naming after ancestors or other people** | Select an eligible namesake from remembered relatives, patrons, teachers, rulers, or sacred figures. Store `namesake_of`, rather than merely copying the spelling. | Inuit accounts describe namesake relationships that affect forms of address and social relationships, not just lexical repetition. [Isuma.tv](https://www.isuma.tv/our-changing-language/inuit-naming) |
| **Birth circumstances and calendars** | Make certain components depend on birth date, birth order, a locally significant event, or circumstances interpreted by the namers. Permit culturally authorized exceptions. | Akan naming includes day, circumstantial, religious, and other categories. Day names have sex- and dialect-specific realizations; Agyekum also documents an exception motivated by another naming obligation. [Nordic Journal of African Studies](https://njas.fi/njas/article/download/24/16) |
| **Names acquired during life** | Initiation, religious affiliation, accession to office, achievement, or reputation may add or replace a name in a particular register. Retain earlier names in history. | Akan accounts distinguish personal names from acquired honorific, religious, occupational, and enthronement names. [Nordic Journal of African Studies](https://njas.fi/njas/article/download/24/16) |
| **Administrative standardization** | A registry can require stable fields, choose one spelling, or convert an existing component into an inherited name. Apply this through institutional reach and compliance—not instant universal adoption. | Peter Irniq’s account describes government-introduced surnames layered over Inuit naming practices around 1970. [Isuma.tv](https://www.isuma.tv/our-changing-language/inuit-naming) |

**Critical inheritance distinction:** a child of a person called “Mina, child of Aru” might become “Tavi, child of Mina.” Under a different policy, *Aruen* might already be an inherited family name, giving “Tavi Aruen.” The strings can look similar while encoding different social systems.

Marriage, adoption, migration, conversion, and office-taking should therefore invoke a **policy decision**, not a hard-coded surname replacement. The policy may preserve all names, change one component, add an alias, or select a different public form.

### 1.2 Separate name choice from name availability

For freely selected personal-name components, I recommend a source-mixture model:

\[
P(n\mid i,e)=\sum\_s w\_s(i,e)\,P\_s(n\mid i,e),
\qquad \sum\_s w\_s=1
\]

Here, \(i\) is the person being named, \(e\) is the naming event, and sources \(s\) might be:

* remembered people;
* names circulating in the local community;
* sacred or culturally meaningful expressions;
* names encountered through contact;
* newly coined forms.

These sources may overlap. First enforce restrictions—eligible namesakes, reserved titles, grammatical categories, taboo rules—then renormalize the available choices.

For a minimal community-copying model, sample existing names according to their frequency among people the namers know. Add prestige, conformity, or novelty preferences only when needed for a particular cultural profile.

Random-copying models can reproduce some skewed name-frequency patterns, but **a good frequency fit does not establish that real naming decisions are socially meaningless**. Bentley, Hahn, and Shennan provide a useful neutral baseline; ethnographic accounts demonstrate intentional relationships and meanings that such a baseline omits. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1691747/)

### 1.3 Place names: record a naming event and its referent

Toponymic research distinguishes descriptive, associative, event-related, evaluative, transferred, commemorative, and linguistically manufactured names, among others. Importantly, **a name’s present apparent meaning is not necessarily its original naming motivation**. Blair and Tent’s revised typology explicitly addresses expressions of naming intention. [Placenames Australia](https://placenames.org.au/wp-content/uploads/ANPSTechPaper2.pdf)

The following is a proposed TCE schema, rather than a reproduction of their taxonomy:

| Naming route | Required simulation context | Example semantic representation |
| --- | --- | --- |
| **Physical feature** | A feature known to the namers | `FORD(reed_beds)`; `HILL(red_rock)` |
| **Activity or structure** | A locally salient practice or construction | `LANDING(fishing)`; `SETTLEMENT(three_towers)` |
| **Person or institution** | A remembered founder, patron, ruler, lineage, temple, or guild | `TOWN(honoree_id)` |
| **Sacred association** | An appropriate religious tradition and associated figure, object, or event | `SPRING(sacred_figure_id)` |
| **Historical event** | An event experienced or commemorated locally | `FIELD(battle_id)`; `BAY(shipwreck_id)` |
| **Transfer or subdivision** | Migration, colonization, or a relationship to an existing place | `NEW(origin_place_id)`; `UPPER(existing_name)` |

The generator should select from **what people could know and consider important**, not from the entire world database.

Store the original referent permanently. A town named for a forest should not automatically change its name when that forest is cleared. A village should not acquire a new suffix merely because the simulation reclassifies it as a city.

Separate two axes:

**Why it was named:** descriptive, commemorative, event-related, and so forth.

**How the form arrived:** locally coined, borrowed, translated, transferred, officially imposed, or restored.

This avoids treating “Indigenous” or “foreign” as if either were a naming motivation equivalent to “named after a river.”

### 1.4 Firms and organizations

For TCE, I recommend a context-conditioned grammar shared with the personal-name and place-name systems:

```
EnterpriseName :=
    OwnerReference + Trade
  | HouseOf + FounderReference
  | PlaceReference + TradeOrInstitution
  | SignOrEmblem
  | PatronReference + OrganizationType
  | CoinedName + PermittedLegalDesignation
```

These are proposed templates, not a universal historical sequence.

An informal workshop may initially be known only as “Mina’s pottery.” A guild, temple estate, partnership, or corporation can have a distinct institutional name. Preserve that name across ownership changes unless a separate renaming decision occurs.

A sign-based name should reference an emblem that actually exists. A legal designation should require the corresponding legal institution. This prevents an early agrarian economy from spontaneously producing modern corporate naming conventions.

### 1.5 Model five different kinds of change

| Process | What changes | What should remain stable |
| --- | --- | --- |
| **Naming fashion** | Frequencies among newly named people | Existing individuals’ identities and normally their assigned names |
| **Genealogical transmission** | Distribution of inherited labels through births, deaths, migration, adoption | The applicable inheritance rule |
| **Sound change** | Pronunciation under a dialect’s sound rules | Lexical identity and historical attestations |
| **Orthographic change** | Written representation or transliteration | Identity; pronunciation may or may not change |
| **Renaming and contact** | Preferred names, borrowed forms, translations, or public aliases | Entity ID and previous name records |

Historical work on place names and locative surnames documents related pronunciation changes and divergences between written and spoken forms. This supports separating these processes rather than randomly editing letters independently in every name. [ANS Names](https://ans-names.pitt.edu/ans/article/download/933/932)

For contact, support borrowed pronunciation, translated meaning, partial translation, and a separate outsider name. An official replacement need not immediately displace local usage.

---

## 2. Parameters: empirical anchors and explicit design priors

### 2.1 Quantitative evidence

“High confidence” below means confidence in the stated observation or dataset—not that it transfers unchanged to every simulated culture.

| Quantity | Observed value or range | Units and population | Source and confidence |
| --- | --- | --- | --- |
| **Consonant inventory size** | **6–122**; reported mean **22.7**; WALS “average” category **19–25** | Consonant phonemes per language in the WALS sample | Maddieson, WALS ch. 1. **High descriptive confidence; limited as a world-population prior.** [WALS Online](https://wals.info/chapter/1) |
| **Basic vowel-quality inventory** | **2–14**; mean just below **6** | Distinct basic vowel qualities, not every length, nasalization, or diphthong contrast | Maddieson, WALS ch. 2. **High within coding conventions.** [WALS Online](https://wals.info/chapter/2) |
| **Syllable-structure classes** | **61 simple, 274 moderately complex, 151 complex**, out of **486** | Languages in the displayed WALS table; calculated shares **12.6%, 56.4%, 31.1%** | WALS ch. 12. **High for that table; not population-weighted.** [WALS Online](https://wals.info/chapter/12) |
| **Akan day-name categories** | **7** calendar categories, with sex- and dialect-dependent forms | Categories in a day-name component—not the entire personal-name inventory | Agyekum 2006. **High for the described system.** [Nordic Journal of African Studies](https://njas.fi/njas/article/download/24/16) |
| **Korean surname labels versus origin categories, 2000** | **288** surnames; **4,188** surname-plus-origin categories; population **45,985,289** | Census categories analyzed by Kim and Park | **High for the published table.** Definition-dependent; do not equate categories with pedigrees. [arXiv](https://arxiv.org/pdf/cond-mat/0407311) |
| **US surname concentration, 2010** | **162,253** surnames occurring at least **100** times; Smith approximately **0.828%** | Percentage denominator: **294,979,229 people with recorded surnames** | US Census technical report. **High administrative confidence; cleaned spellings and thresholded release.** [Census](https://www2.census.gov/topics/genealogy/2010surnames/surnames.pdf) |
| **US top-ten given-name share, 1880s** | Boys **38.43%**; girls **21.09%** | Calculated from SSA’s decade table and its stated recorded-cohort denominators | SSA. **High arithmetic confidence; substantial historical coverage limitations.** [Social Security Administration](https://www.ssa.gov/oact/babynames/decades/names1880s.html) |
| **US top-ten given-name share, 2010s** | Boys **7.72%**; girls **7.57%** | Same calculation using the 2010s recorded cohorts | SSA. **High within the published data.** [Social Security Administration](https://www.ssa.gov/oact/babynames/decades/names2010s.html) |
| **Immigrant naming convergence in the United States** | Approximately **half the naming gap closes after 20 years** of residence | Difference in children’s name “foreignness,” historical and modern samples | Abramitzky, Boustan, and Eriksson 2020. **Moderate-to-high for this setting; not a universal cultural half-life.** [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faeri.20190079) |

**Interpretation matters.** The American and Korean surname counts have different populations and inclusion rules; they are not directly comparable dictionary sizes. SSA’s early cohorts are not a complete nineteenth-century birth census. WALS chapter prose and currently displayed tables also contain some version-dependent sample-count differences; the syllable percentages above use the displayed 486-language table.

The useful conclusions are structural: inventories vary greatly, popular names can dominate, concentration can change substantially, and surname labels do not map one-to-one onto lineages.

### 2.2 Proposed starting settings for TCE

These are **engineering priors proposed here**, not historical estimates or confidence intervals. Expose them in authored culture profiles and test sensitivity.

| Parameter | Proposed starting value or sweep | Units | Source / confidence |
| --- | --- | --- | --- |
| Initial freely selectable given-name inventory | **50–500**, with smaller closed systems and larger open systems permitted | Lexemes per naming profile | Design prior; **unfitted** |
| Remembered ordinary namesake window | **1–3 generations**, plus separately remembered notable people | Genealogical generations | Design prior; **unfitted** |
| Namesake-source weight | **0–0.8**, normalized with other source weights | Probability mass per naming event | Design prior; **low historical confidence** |
| Genuine lexical innovation | **0.001–0.05** | Probability per freely chosen naming event | Design prior; **low historical confidence**; borrowing is separate |
| Markov conditioning history | **1–3**, start at **2** | Previous phonemes or grapheme units | Engineering choice; evaluate on held-out data |
| Short lexical-name length | Start with **1–4 syllables** | Syllables per lexical component | Engineering choice, **not a universal name-length limit** |
| Sound-rule adoption episode | Test **25–100 years**, with change disabled in the baseline | Years per selected change episode | Scenario assumption; **not an estimated sound-change clock** |
| Vernacular uptake of an official replacement | Test **5–50 years**, plus persistent rejection | Adoption half-life among exposed users | Scenario assumption; **not inferred from immigrant naming convergence** |
| Popularity-statistic refresh | **1 year**, using event-maintained counts | Simulation years | Engineering choice |

Do not independently randomize all these values. Author **coherent packages**: a restrictive ancestral repertoire, a productive meaningful-name grammar, and an open fashion-driven repertoire should behave differently.

I would not assign a universal annual probability of “developing surnames.” Make surname fixation a change in household practice or institutional policy. Likewise, political renaming should depend on an actual political decision and its affected names.

---

## 3. Variation across eras and regions

### 3.1 Eras are evidence contexts, not a naming technology ladder

| Context | Appropriate representation and limits |
| --- | --- |
| **Foragers** | Allow personal names, namesakes, nicknames, and context-dependent address without requiring inherited surnames or writing. There is no recoverable global distribution of prehistoric names. Recent ethnography is an analogy, not a direct record of prehistory. |
| **Early farming communities** | Retain multiple possible naming systems. Farming alone should not activate surnames. Generate meaningful names from locally known people, organisms, places, and events, but label exact prehistoric probabilities as unknown. |
| **Literate pre-industrial societies** | Permit lineage names, professional or status-linked names, sacred names, and multiple languages in the same polity. Babylonian evidence from approximately 750–100 BCE already contains complex social distinctions and multilingual naming. [Academia](https://www.academia.edu/114073987/Waerzeggers_C_and_Gro%C3%9F_M_eds_2024_Personal_Names_in_Cuneiform_Texts_from_Babylonia_c_750_100_BCE_An_Introduction) |
| **Industrializing and expanding administrative societies** | Let registration, migration, and institutions promote standardized official forms without necessarily eliminating household or community names. Inuit surname standardization provides a particularly clear later example of administrative layering. [Isuma.tv](https://www.isuma.tv/our-changing-language/inuit-naming) |
| **Modern societies** | Support standardized records alongside continued diversity in naming structures. Modernity need not imply one inherited surname, one ordering convention, or increasing uniqueness everywhere. [W3C](https://www.w3.org/International/questions/qa-personal-names) |

For TCE’s early agrarian start, the defensible approach is to choose a culturally coherent initial system and let institutions, contact, and family decisions alter it. Do not make every world repeat a European surname chronology.

### 3.2 Regional examples to encode as distinct modules

These examples identify practices, not rules obeyed by every person in a region.

| Tradition or region | Relevant pattern | TCE implication |
| --- | --- | --- |
| **Akan-speaking West Africa** | Calendar, circumstance, family, religious, and acquired names can coexist. | One person can accumulate differently motivated components. [Nordic Journal of African Studies](https://njas.fi/njas/article/download/24/16) |
| **Ethiopia** | Personal names may be followed by a father’s and grandfather’s personal names. | A multiword name need not contain a hereditary surname. [GOV.UK](https://www.gov.uk/government/publications/ethiopia-knowledge-base-profile/ethiopia-knowledge-base-profile) |
| **Chinese naming traditions** | Family name first; a shared generation element can appear in personal names. | Relatives’ names can share a component without being identical. [W3C](https://www.w3.org/International/questions/qa-personal-names) |
| **Korea** | Surname labels and regional-origin affiliations are distinct levels. | Store affiliation separately from the visible surname. [arXiv](https://arxiv.org/pdf/cond-mat/0407311) |
| **South Asia** | Community-specific combinations can include parental, place, and other name components. | Select a subtradition, not an “Indian first/last-name” format. [W3C](https://www.w3.org/International/questions/qa-personal-names) |
| **Malay traditions** | *Bin/binti* constructions can identify a parent rather than a hereditary surname. | Preserve a relational reference. [W3C](https://www.w3.org/International/questions/qa-personal-names) |
| **Arabic traditions** | Personal names can coexist with parentage, origin descriptors, epithets, and parent-of-child forms. | Components should be optional and register-dependent. [W3C](https://www.w3.org/International/questions/qa-personal-names) |
| **Icelandic and Russian contrasts** | Patronymic systems differ; Russian templates can include both patronymic and family name. | Component role cannot be inferred from position alone. [W3C](https://www.w3.org/International/questions/qa-personal-names) |
| **Spanish- and Portuguese-speaking societies, including the Americas** | Multiple family names and differing ordering conventions occur. | Store ordered component lists and explicit inheritance selection. [W3C](https://www.w3.org/International/questions/qa-personal-names) |
| **Inuit communities** | Namesake relationships can shape address and social identity, including honoring someone of another sex. | Do not impose universal gender-exclusive name pools or equate namesake relations with biological kinship. [Isuma.tv](https://www.isuma.tv/our-changing-language/inuit-naming) |
| **Māori toponymy** | Names can encode environmental descriptions and personal associations: *Whanganui* and *Whanganui-o-Hei* illustrate related constructions. | Generate language-specific phrases and possession, not English-style suffix concatenation. [NZ History](https://nzhistory.govt.nz/culture/maori-language-week/1000-maori-place-names) |

**Culture, language, polity, and religion should be separate references.** A polity can contain several naming traditions; a religious name can cross language boundaries; a household can maintain one practice while a registry recognizes another.

---

## 4. Stylized facts and validation targets

### 4.1 Repetition should be normal

Uniformly generating a novel name for every person produces the wrong statistical structure.

For independent draws with name probabilities \(p\_n\), define effective diversity:

\[
D\_2=\frac{1}{\sum\_n p\_n^2}
\]

Then the expected number of same-name pairs in a population of \(N\) is:

\[
E[\text{matching pairs}]=\frac{N(N-1)}{2D\_2}
\]

For an illustrative \(N=100\) and \(D\_2=20\), this is **247.5 matching pairs**—not 247.5 duplicated individuals. Kin-based naming creates dependencies, so this is a baseline rather than a complete model.

Similarly, the expected number of distinct names under independent sampling is:

\[
E[K]=\sum\_n\left[1-(1-p\_n)^N\right]
\]

This explains why a national name inventory should not be scaled linearly down to a 10,000-person settlement.

### 4.2 Do not enforce a universal Zipf distribution

Kim and Park found approximately exponential Korean surname rank-frequency behavior, contrasting with patterns reported elsewhere. A generic “all names follow Zipf” generator would erase meaningful differences. [arXiv](https://arxiv.org/pdf/cond-mat/0407311)

Measure the distribution your chosen rules produce; do not prescribe one universal curve.

### 4.3 Recommended acceptance tests

| Test | Pattern a satisfactory implementation should reproduce |
| --- | --- |
| **Concentration** | Frequent names coexist with rarer ones. Calibrate top-1, top-10, and effective-diversity statistics to a selected reference profile. |
| **Cohort differences** | New naming fashions affect younger cohorts first. Do not rename adults annually to match current popularity. The SSA decade contrast supplies one empirical calibration case, not a universal modernization trajectory. [Social Security Administration](https://www.ssa.gov/oact/babynames/decades/names1880s.html) |
| **Genealogical consistency** | Patronymics refer to the correct parent; inherited components follow the chosen descent rule; namesake links point to eligible people. |
| **Surname persistence** | A frozen occupational surname survives a descendant changing jobs. Unrelated lineages may share its label. |
| **Spatial structure** | Local exposure produces clusters; migration and intermarriage create overlap rather than immediate global homogenization. |
| **Historical place-name layers** | Some names reference vanished features, former institutions, or previous languages. Historical records retain their contemporary forms. |
| **Contact** | Borrowed and local variants can coexist. The US immigrant study supplies a bounded naming-convergence test, not a universal language-replacement parameter. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faeri.20190079) |
| **Technical reproducibility** | Saving, loading, changing camera position, or altering rendering order never changes an entity’s assigned names. |

Run these tests at several population sizes. A system that looks plausible in a list of fifty names may become implausibly uniform—or implausibly unique—over centuries.

---

## 5. Modeling recommendation: generators, data structures, and implementation

### 5.1 Compare the generation methods

| Method | Strengths | Failure modes | Recommended role |
| --- | --- | --- | --- |
| **Curated weighted lists** | Strong attested forms; direct control over frequency and social categories | Limited productive novelty; can become static | Core inherited and conventional personal-name pools |
| **Phonotactic generation** | Enforces permissible sounds, syllables, and transitions | Well-formed nonsense without social or semantic structure | Coining new roots and occasional personal names |
| **Markov / n-gram generation** | Learns local sequence patterns with modest implementation effort | Memorization; malformed morphemes; poor long-distance constraints; no inherent meaning | Candidate generation or stylistic scoring |
| **Grammar-based generation** | Can express relations, compounds, titles, possession, and actual world referents | Repetition and awkward joins without morphology | Main engine for structured personal names, places, and institutions |
| **Hybrid** | Combines social consistency, meaning, and linguistic plausibility | More authored structure required | Best overall choice for TCE |

These are implementation tradeoffs, not evidence that one algorithm is universally superior.

### 5.2 Phonotactic generation

A language profile should contain a **compatible package**, not just an alphabet:

```
PhonologicalProfile
    consonants and vowels
    permitted onset and coda patterns
    permitted consonant sequences
    stress, tone, and/or vowel-harmony rules
    morpheme-boundary alternations
    orthographic realization
```

Select phonemes and structural rules together. Adding a consonant to an inventory does not mean it belongs in every position or cluster.

Hayes and Wilson’s maximum-entropy phonotactic model is a useful technical precedent: weighted constraints evaluate possible forms, including categorical and gradient restrictions. Their work also illustrates why nonlocal phenomena such as harmony and stress require more than adjacent-letter statistics. [Bruce Hayes](https://brucehayes.org/Phonotactics/)

For v1, hand-authored constraints will usually be easier to inspect than a fully learned phonotactic grammar. Do not equate a simple syllable structure with a technologically “primitive” culture.

### 5.3 Markov generation

A Markov generator selects the next unit from a limited preceding context:

\[
P(x\_t\mid x\_{t-h},\ldots,x\_{t-1})
\]

A practical TCE implementation should have explicit start/end symbols, smoothing or backoff for unseen contexts, and a hard termination limit.

Use **phonemes or appropriate grapheme units**, not UTF-8 bytes. Keep separate models for personal roots, family-name forms, place-name roots, and institution names when their structures differ.

Two distinctions are especially important:

**Type frequency versus bearer frequency.** A name occurring a million times should not necessarily dominate phonotactic training a million times. Learn linguistic form and social popularity separately.

**Repetition versus memorization.** Reusing an established name is desirable in the social sampler. Accidentally reproducing the training list while claiming to coin new roots is a different issue.

Evaluate generated candidates for held-out sequence likelihood, copied strings, forbidden clusters, and human judgments of consistency. A low n-gram loss does not establish cultural appropriateness.

### 5.4 Use an attributed grammar for meaningful names

Ordinary string templates are insufficient when endings, word order, or forms depend on the referenced entity.

```
PersonName(policy, person)
    -> SelectedPersonalName
       + RequiredRelationalComponents
       + OptionalAffiliationComponents
       + RegisterSpecificTitles

PlaceName(language, event)
    -> Describe(event.feature)
     | Commemorate(event.honoree)
     | Reference(event.activity)
     | Transfer(event.origin_place)

EnterpriseName(language, organization)
    -> Reference(organization.founder)
     | Reference(organization.location)
     | Reference(organization.emblem)
     | CoinedInstitutionalExpression
```

Each rule should return a structured expression before it returns a string. A “founder’s settlement” construction may require a possessive form, linker, noun-class agreement, or order unlike English.

**Do not generate separate, unrelated versions of the same morpheme everywhere.** If a culture uses one word for “ford,” related place names should share recognizable forms unless a recorded sound change, borrowing, or reanalysis explains the difference.

### 5.5 A fictional worked example

The following forms are invented to illustrate implementation, not taken from a real culture.

Suppose a language has:

* `sela`: reeds;
* `nava`: ford;
* `-en`: a parent-reference construction.

At founding, a settlement receives the semantic name `FORD(REEDS)`, realized as **Selanava**.

A person named Aru has a child **Mina Aruen**. Mina’s child becomes **Tavi Minaen**, because the second component is relational.

Later, an institution freezes **Aruen** as a hereditary family label for one household. Descendants then inherit that label instead of rebuilding a patronymic. Other households may retain the older practice.

A subsequent dialect change shifts intervocalic /v/ to /w/, producing **Selanawa** in speech. Earlier records retain *Selanava*. A neighboring language develops an adapted outsider form. None of these operations creates a new settlement ID.

The same short sequence generates genealogy, bureaucracy, linguistic history, and recognizable regional naming patterns.

### 5.6 Rust-side representation

I recommend the following separation:

| Record | Essential contents |
| --- | --- |
| **Entity** | Immutable ID; links to people, places, or organizations |
| **NamingProfile** | Component policies; inheritance rules; source weights; register rules |
| **LanguageProfile** | Lexemes, morphology, phonotactics, orthographies |
| **NameAssignment** | Entity ID; structured components; naming event; effective dates; namer |
| **NameComponent** | Role; lexical reference; optional parent, namesake, affiliation, or place reference |
| **NameVariant** | Pronunciation/spelling; language or dialect; register; usage interval; preferred-status information |
| **NameHistory** | Assignment, standardization, borrowing, replacement, and restoration events |

A person can possess more than one profile or usage context. Select the public form according to the speaker, institution, and situation.

For performance, make assignment **event-driven**: birth or naming ceremony, adoption, marriage under an applicable rule, migration-related adaptation, accession, institutional reform, founding, or explicit renaming.

Intern recurring lexemes and components. Cache realizations by language version and register. Maintain popularity statistics incrementally. A sound-rule change should invalidate relevant caches lazily rather than rewrite every historical document.

Use a dedicated deterministic random stream for naming. Store the generated assignment; do not regenerate it from current culture settings whenever the UI requests a label.

### 5.7 Existing implementations and models worth borrowing from

**libtcod’s `TCODNamegen`**, documented in version 1.24.0, provides configurable syllable sets, generation rules, named sets, and seeded randomness. It is a useful small-scale implementation reference, but does not supply genealogy or cultural history. [LibTCOD](https://libtcod.readthedocs.io/en/1.24.0/library/class_t_c_o_d_namegen.html)

**OpenTTD/TTDPatch’s NewGRF Action F** defines town-name styles using weighted parts and nested intermediate definitions. Borrow the data-driven composition idea. Its documented duplicate-name rejection and display constraints are game-specific choices, not historical principles to reproduce. [GRFSpecs](https://newgrf-specs.tt-wiki.net/wiki/ActionF)

**Bentley–Hahn–Shennan neutral cultural transmission** provides a baseline for copying and innovation. Use it as a comparison model before adding prestige or conformity effects. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1691747/)

**Hayes–Wilson maximum-entropy phonotactics** provides a principled extension beyond simple n-grams when richer linguistic constraints become worthwhile. [Bruce Hayes](https://brucehayes.org/Phonotactics/)

**Recommended v1:** weighted conventional names, relationship-aware components, meaningful place-name grammars, persistent aliases, and immutable histories. Add contact adaptation and authored sound-change rules next. A full language-evolution simulator is not necessary to make the naming system convincing.

---

## 6. Sources, datasets, and limits of the evidence

### Priority reference and data stack

| Source | Best use in TCE research | Main limitation |
| --- | --- | --- |
| **Agyekum 2006, “The Sociolinguistic of Akan Personal Names”** | Naming motivations, calendar components, acquired names, social context | A detailed cultural account, not a globally transferable frequency model. [Nordic Journal of African Studies](https://njas.fi/njas/article/download/24/16) |
| **Waerzeggers and Groß, eds., 2024, *Personal Names in Cuneiform Texts from Babylonia, c. 750–100 BCE*** | Ancient social roles, family-name structures, multilingual naming | Surviving written records selectively represent people and institutions. [Academia](https://www.academia.edu/114073987/Waerzeggers_C_and_Gro%C3%9F_M_eds_2024_Personal_Names_in_Cuneiform_Texts_from_Babylonia_c_750_100_BCE_An_Introduction) |
| **Blair and Tent 2021, “A Revised Typology of Place-Naming”**, with the ANPS technical paper | Structured naming motives and mechanisms | Classification does not provide universal probabilities of each motive. [ANS Names](https://ans-names.pitt.edu/ans/article/view/2260) |
| **WALS, chapters 1, 2, and 12** | Inventory and syllable-structure calibration | Typological samples and coding decisions; not population-weighted random samples. [WALS Online](https://wals.info/chapter/1) |
| **PHOIBLE 2.0** | Actual phonological inventory packages: **3,020 inventories covering 2,186 languages** | Inventories are not complete phonotactic grammars; multiple analyses can represent one language. [Phoible](https://phoible.org/) |
| **US SSA name files** | Cohort popularity, concentration, turnover, regional comparison | Historical coverage changes; low-frequency names are suppressed, generally below five occurrences in a geographic reporting area. [Social Security Administration](https://www.ssa.gov/oact/babynames/limits.html) |
| **US Census 2010 surname files** | Surname concentration and frequency tails | Thresholded release and cleaning: punctuation and spacing may not preserve original orthography. [Census](https://www2.census.gov/topics/genealogy/2010surnames/surnames.pdf) |
| **Dictionary of Medieval Names from European Sources** | Attested medieval forms and spellings | Attestation collections should not automatically be treated as representative population frequencies. [Dictionary of Medieval Names](https://dmnes.org/) |
| **GeoNames and official national gazetteers** | Feature types, alternative names, geographic associations | Present-day labels are not automatically etymologies or evidence of historical naming motives. [GeoNames](https://www.geonames.org/about.html) |
| **NZHistory’s Māori place-name collection** | Examples of component meanings and environmental or personal references | Specific etymologies require contextual checking; the site’s text has noncommercial reuse terms. [NZ History](https://nzhistory.govt.nz/culture/maori-language-week/1000-maori-place-names) |

For commercial game assets, keep **research evidence separate from redistributable training or content files**. A publicly readable dictionary is not automatically a corpus that can be bundled into the game.

### Claims to treat cautiously

**“Every society eventually develops hereditary surnames.”** The documented structural diversity does not support using this as an inevitable progression.

**“Name frequencies follow one universal law.”** Distributional form depends on what is counted, the population, and the naming system.

**“A name transparently reveals its original meaning.”** Historical forms, borrowing, reinterpretation, and incomplete evidence can defeat modern-looking etymologies.

**“One measured assimilation rate supplies a general cultural-change constant.”** The approximately twenty-year result concerns a particular measure of children’s names in US immigrant populations, not surname replacement, language loss, or official-place-name adoption. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faeri.20190079)

**“Phonotactic plausibility is cultural plausibility.”** A pronounceable string can still violate naming roles, genealogy, semantic conventions, or institutional rules.

The reviewed evidence supplies strong examples of structures and some excellent calibration distributions, but it does **not** establish globally comparable rates of ancestor-name reuse, ordinary place renaming, or sound change per century. Those should remain explicit, testable modeling assumptions.

**The highest-value design decision is to preserve relationships and history.** A modest lexicon with correct inheritance, meaningful naming events, repeated names, and persistent local variants will produce a more convincing civilization than millions of unrelated “plausible-sounding” strings.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9283c-ac08-83ea-9852-b8ff54413250)
