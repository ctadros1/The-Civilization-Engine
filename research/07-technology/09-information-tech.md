# Writing, records and communications for The Civilization Engine

## Core recommendation

**Model information technology as a set of costly services, not a sequence of global administrative or research bonuses.** The important services are recording, calculation, authentication, storage and retrieval, reproduction, and transmission. Each can improve independently; each requires people, materials, institutions and operating expenditure.

This distinction is historically important. Early Mesopotamian writing is strongly associated with economic administration; Andean khipus supported record-keeping without an alphabet; and East Asian printers continued using woodblocks after movable type became available. These are alternative solutions to different information problems, not incomplete steps toward a single European endpoint. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing)

For TCE, distinguish five states:

**Known → teachable locally → equipped → adopted by an institution → operating at a particular capacity.**

A settlement can know about paper without having a papermaker, possess an imported accounting manual without a competent accountant, or maintain a courier route that only officials may use. None of these should count as universal access.

Throughout this report, **historical evidence and proposed simulation parameters are separated**. Dates are archaeological or documentary anchors, not era gates.

---

# 1. Mechanisms: rules the simulation can implement

## 1.1 Record-keeping makes obligations persistent—but does not make records true

An early administrative tablet might record an allocation of grain, livestock or labor. Such records externalize information that would otherwise depend on participants remembering an event and agreeing about it later. The surviving late-fourth-millennium Mesopotamian evidence is particularly rich in these economic uses. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing)

**Implementation rule:** create a record when an agent or institution expects its future usefulness to exceed its cost:

\[
V\_{\text{record}}
=
V\_{\text{remembering}}
+
V\_{\text{evidence}}
+
V\_{\text{coordination}}
-
C\_{\text{materials}}
-
C\_{\text{clerical labor}}
-
C\_{\text{disclosure}}.
\]

The last term matters: a merchant may prefer an accurate private ledger but resist giving it to a tax collector.

Maintain three separate layers:

| Layer | What it contains |
| --- | --- |
| Physical reality | Actual goods, movements, people, payments and events |
| Recorded claims | What receipts, registers, ledgers and witnesses say happened |
| Agent beliefs | What a particular person or institution currently believes |

This separation generates missing inventories, stale censuses, false receipts, disputed debts and successful audits without scripting them.

Do not give societies without writing zero memory. For TCE, oral testimony, repeated public accounting, witnesses and tally objects should provide a functioning baseline. Written records primarily change persistence, portability, searchability and dependence on particular people.

A laboratory experiment by Basu and colleagues found that record-keeping supported reputation formation and reciprocal exchange. That supports a behavioral mechanism, not a numerical estimate of how much writing increased ancient trade; contemporary criticism specifically challenged the leap from laboratory results to economic history. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0811967106)

## 1.2 Administrative capacity is a workload problem

Comparative research using Seshat has investigated relationships between societal scale and information-processing capabilities. The useful modeling insight is that communication and record-processing arrangements can become bottlenecks as organizations grow. The research does **not** justify a universal population threshold at which writing appears or a fixed percentage bonus from literacy. [Nature](https://www.nature.com/articles/s41467-020-16035-9)

For each institution, calculate:

\[
W\_t=\sum\_j A\_{j,t}s\_j
\]

where \(A\_{j,t}\) is arriving tasks of type \(j\), and \(s\_j\) is their required labor time.

\[
B\_{t+1}=\max(0,\;B\_t+W\_t-C\_t)
\]

where \(B\) is unfinished work, and \(C\) is available effective clerical labor.

Tasks should include registering a household, updating an obligation, reconciling a storehouse, preparing an order, finding a precedent, and investigating a discrepancy. Field inspection and travel are separate tasks.

**Consequences of overload:** records become older, payments remain unrecognized, requests wait, officials delegate, and discrepancies accumulate. Hiring clerks, simplifying procedures, reducing reporting requirements or improving retrieval can all relieve the same bottleneck.

For taxation, information is only one component:

\[
\text{realized revenue}
=
\text{observable taxable base}
\times
\text{assessment coverage}
\times
\text{collection success}
-
\text{administrative cost}.
\]

Do not multiply every component by a “writing bonus.” A better register can improve coverage while enforcement, legitimacy or transport remains the limiting factor.

## 1.3 Numeracy, literacy and accounting are distinct capabilities

Place-value notation, a placeholder for an empty position, and zero treated as an arithmetic number are different developments. Babylonian positional calculation existed before its later explicit zero-like placeholder; Indian mathematical traditions subsequently developed rules for arithmetic involving zero. [Maths History](https://mathshistory.st-andrews.ac.uk/HistTopics/Zero/)

Represent at least four skills independently: reading, writing, arithmetic and procedural bookkeeping. A person may count grain accurately without reading prose, or read a religious text without being able to reconcile accounts.

A quantity should carry:

`amount + commodity + unit + quality + location + date`

This prevents a central administrator from automatically adding unlike “baskets” of grain or treating fresh and spoiled stocks as equivalent. Shared measures, conversion tables and inspection procedures improve aggregation.

Calculating aids should offer alternative methods. Do not require Hindu-Arabic numerals for competent commerce, nor treat alphabetic writing as a prerequisite for advanced mathematics.

## 1.4 Authentication changes the credibility of claims

Mesopotamian cylinder seals left identifying impressions in wet clay; surviving examples reach back to the late fourth millennium BCE. Their administrative function was closer to an identifying mark or signature than to a guarantee that a document’s contents were correct. [Spurlock Museum](https://www.spurlock.illinois.edu/collections/notable-collections/profiles/cylinder-seals.html?utm_source=chatgpt.com)

For TCE, a seal, witness or official registration should modify the probability that an institution accepts a claim. It should not overwrite physical reality.

Authentication can depend on recognized identity, custody of the seal, witness reliability, an intact closure, a matching counterpart and the jurisdiction’s evidentiary rules. Stolen seals, collusion and conflicting documents then become possible through ordinary mechanics.

Apply the same distinction to law: publication and archival retrieval can make rules more consistent, but recorded law still requires interpretation, access and enforcement. A register may protect a claim or make extraction easier; the political outcome should depend on who controls it.

## 1.5 Double-entry is a linked representation of an entity—not “two copies”

Sangster’s recent analysis identifies double-entry features in a Florentine banking ledger of 1211 and distinguishes these from entity-wide systems evident near the end of the thirteenth century. The date of “invention” therefore depends partly on the definition used. Pacioli’s 1494 publication disseminated an existing practice rather than originating it. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13358)

The essential model invariant is:

\[
\sum \text{debits}=\sum \text{credits}
\]

within one accounting entity.

For example, acquiring inventory on credit creates an inventory entry and a payable; settling the debt reduces both cash and the payable. This provides linked views of resources and obligations.

**TCE implementation:** introduce stable account identities, cross-referenced postings, a common valuation convention, reconciliation and periodic summaries. Let the system improve an organization’s ability to understand its own finances rather than directly creating wealth.

Balanced books still permit omitted transactions, fraudulent valuations and collusive entries. Physical inspection remains valuable. Model accounting errors and fraud separately.

Adoption should be selective. A small producer may find a simple cash-and-debt book sufficient; a bank or multi-branch merchant faces a different information problem. Evolutionary accounts of bookkeeping emphasize this gradual adaptation rather than an instantaneous transition to universally superior accounting. [Columbia Business School](https://business.columbia.edu/sites/default/files-efs/imce-uploads/CEASA/Events%20Page/evolution_double-entry_bookkeeping.pdf)

## 1.6 Archives provide usable memory only when documents can be found and understood

For implementation, distinguish three properties:

**Physical survival**, **retrievability**, and **interpretability**.

A surviving document is useless to an official who cannot locate it, read its script, interpret its units or determine which version applies.

Archive work should include classification, storage, retrieval, copying and disposal. Model simple search as labor proportional to the portion of the collection examined; indexing reduces that portion but costs labor to establish and maintain.

Copies should protect against some losses but not all. A useful proposed model is:

\[
P(\text{all copies lost})
=
p\_{\text{common}}
+
(1-p\_{\text{common}})p\_{\text{independent}}^n.
\]

Here, \(n\) copies share a common-disaster risk but otherwise fail independently. This is a modeling assumption, not an archaeological estimate. Copies in separate settlements should usually have lower shared risk than copies on adjacent shelves.

Avoid one universal “document half-life.” The preservation of Egyptian papyri is strongly affected by dry conditions; surviving collections cannot be treated as a random sample of everything originally written. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/papyrus-in-ancient-egypt)

## 1.7 Printing changes the economics of repetition

For a comparable text and physical format:

\[
C\_{\text{manuscript}}(q)=q\,c\_m
\]\[
C\_{\text{print}}(q)=F\_{\text{master}}+q\,c\_p+C\_{\text{finance}}+C\_{\text{distribution}}.
\]

Ignoring the last two terms for an initial comparison:

\[
q^\*=\frac{F\_{\text{master}}}{c\_m-c\_p}.
\]

Printing becomes attractive above \(q^\*\), provided demand actually exists and the producer can finance the run.

A woodblock is a stored page-master. Movable type is a reusable collection of characters that must be selected, arranged, corrected and redistributed. They solve different production problems. Research and collection-based work on East Asian printing documents their coexistence, including cases where movable type and woodblocks were used at different stages of dissemination. [AFE East Asia](https://afe.easia.columbia.edu/songdynasty-module/tech-printing.html)

**TCE consequences:** printing should favor repeated forms, widely used manuals and texts with a sufficiently large audience. It should not automatically replace unique contracts or small custom jobs.

Distinguish press equipment from working capital. Paper, wages and unsold inventory must be financed before sales return money. Evidence on early European printing businesses supports treating presses as firms with substantial entry costs and highly uneven market sizes. [CEP | Centre for Economic Performance](https://cep.lse.ac.uk/pubs/download/dp1365.pdf)

## 1.8 Communication speed includes waiting, handling and the last mile

Use:

\[
T\_{\text{delivery}}
=
T\_{\text{preparation}}
+
T\_{\text{dispatch wait}}
+
\sum\_e T\_e
+
T\_{\text{handling}}
+
T\_{\text{last mile}}.
\]

For regular departures every \(h\) days and uniformly distributed arrivals, expected dispatch waiting is \(h/2\). An infrequent fast courier may therefore deliver more slowly than a frequent modest-speed service.

A relay network requires stations, provisions, staff, route security and rules assigning responsibility between stages. Herodotus’s description of the Persian Royal Road explicitly includes stages, accommodation, guarded passages and river crossings: communications infrastructure was much more than a line on a map. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Herodotus/5B%2A.html)

Separate **route existence**, **route capacity**, **authorized users**, **priority** and **price**. An official dispatch network need not serve ordinary merchants. Conversely, commercial carriers can operate without a centralized state service.

## 1.9 Telegraphy separates message movement from cargo movement

The Cooke–Wheatstone electric telegraph was patented in Britain in 1837, with early deployment connected to railway operations. Later submarine cables extended the principle across oceans. [London Museum](https://www.londonmuseum.org.uk/collections/v/object-730098/telegraph-single-needle-telegraph/)

For post-v1, reuse the same message abstraction but add electrical links. Require functioning conductors, insulation, power, instruments, operators and maintained endpoints.

Do not make communication literally instantaneous. Encoding, queues, office hours and local delivery remain. Nor should faster information reduce the physical travel time of grain or ships.

Steinwender’s study of the 1866 transatlantic cable is especially useful: faster information reduced price differences and their volatility, while average trade and the volatility of trade flows increased. Better information can produce more responsive—and sometimes more variable—shipments. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20150681)

## 1.10 Documents transmit some knowledge more readily than others

For TCE, separate:

**Awareness → comprehension → practiced competence → reliable production.**

A book can advertise a technique, preserve a diagram or communicate proportions. The receiving agent should still require study time, appropriate tools, materials and, for some tasks, instruction from an experienced practitioner.

Give authored knowledge items a `codifiability` property. A standardized table and a difficult manual craft should not receive identical benefits from copying.

Cheap reproduction should also increase the volume of obsolete, contradictory and low-value material. Allocate reading time and attention; do not let every printed text automatically add research points.

---

# 2. Parameters: historical anchors versus simulation priors

## 2.1 Historical quantities suitable for calibration

**Confidence:** H = strong within the stated setting; M = reconstruction or identification assumptions matter; L = substantial uncertainty. Confidence in transferring a value to another society is generally lower.

| Quantity | Historical value and scope | Appropriate TCE use | Confidence |
| --- | --- | --- | --- |
| Hand-press output | Plantin-Moretus reports **1,250 double-sided sheets per press-day**, with an approximately **14-hour working day** | Roughly **89 completed double-sided sheets/hour**, derived from those figures. Do not call these books or individual pages. Two-pull operation complicates “impressions/hour.” Excludes composing and binding. [Museum Plantin-Moretus](https://museumplantinmoretus.be/en/worlds-two-oldest-printing-presses) | M; specific workshop benchmark |
| Printing equipment capital | Mid-sixteenth-century equipment estimated at **4–10 years of a craftsman’s earnings** | Entry-capital anchor; add premises, paper, wages and inventories separately. [CEP | Centre for Economic Performance](https://cep.lse.ac.uk/pubs/download/dp1365.pdf) | M |
| Printing-firm concentration | In Dittmar’s data, an active printing city in **1550** had a **median of 2 firms** and **mean of 6.1** | Target a skewed distribution, not one identical printer per town. [CEP | Centre for Economic Performance](https://cep.lse.ac.uk/pubs/download/dp1365.pdf) | M–H within sample |
| Printing infrastructure scale | The thirteenth-century Tripitaka Koreana collection comprises **81,258 wooden printing blocks** | Demonstrates that stored page-masters can be major institutional assets; not a typical workshop size or average block lifetime. [Jikji to Gutenberg](https://jikji.utah.edu/) | H |
| Premium relay delivery | Pony Express: more than **1,800 miles in 10 days**, approximately **290 km/calendar-day**, 1860–1861 | High-performance system benchmark, not the sustainable speed of one horse or rider. [National Park Service](https://www.nps.gov/poex/learn/historyculture/index.htm) | H for benchmark; M for transfer |
| Literacy estimate | A Met synthesis gives approximately **0.5–3%** literate for ancient Egypt | Explore low-literacy societies with specialist access. This is not a census, a universal ancient rate or a well-resolved time series. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/papyrus-in-ancient-egypt) | L–M |
| Printing and urban growth | Dittmar estimates that early printing-adopting European cities grew **60% faster during 1500–1600** than comparable cities | External outcome benchmark. Not +60 percentage points of population growth, and not a direct productivity multiplier. [IDEAS/RePEc](https://ideas.repec.org/a/oup/qjecon/v126y2011i3p1133-1172.html) | M; causal assumptions matter |
| Telegraph trade benefit | Steinwender estimates gains equivalent to about **8% of export value** in the studied transatlantic trade setting | Validate a delayed-information trade model; do not apply to all GDP or every route. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20150681) | M |
| Cheap public postage | British Uniform Penny Post began in **1840**, with a **one-penny rate for letters within the initial weight allowance** | An institutional tariff reform, not an underlying transport-speed technology. [MOTAT Collection](https://collection.motat.nz/topics/56/redressing-the-balance-a-brief-history-of-letter-balances-and-the-penny-post?utm_source=chatgpt.com) | H for reform; no universal cost ratio |

These quantities describe different things: output, capital costs, organizational distributions and estimated economic effects. They should not be combined into one “information efficiency” statistic.

## 2.2 Proposed starting parameters for TCE

**Every number below is an engineering prior, not a measured historical average.** They are intended to make an initial simulation operable and should be varied widely during calibration.

Define a *basic record* as one routine transaction or register update containing approximately 10–20 structured fields, not a fixed number of written words.

| Parameter | Starting range | Unit | Basis and confidence |
| --- | --- | --- | --- |
| Create a routine record | 5–20 | Worker-minutes/record | Proposed task-cost prior; low historical confidence |
| Process a complex account or document | 20–90 | Worker-minutes/case | Excludes travel, negotiation and litigation; low |
| Retrieve a well-indexed record | 1–5 | Worker-minutes/request | Proposed small-archive prior; low |
| Search a poorly organized archive | 10–60 | Worker-minutes/request | Only an initial range for modest collections; replace with search-volume dependence; low |
| Productive clerical time | 4–7 | Hours/worker-day | Scenario assumption after breaks and other duties; low |
| Accidental entry error | 0.2–2 | Percent of routine records | Sensitivity parameter, separate from deliberate fraud; low |
| Independent review detection | 50–90 | Percent of accidental errors reviewed | Proposed prior; correlated mistakes require a separate mechanism; low |
| Task-limited literacy training | 300–1,500 | Practice/instruction hours | Broad prior, not a claim about ancient schooling; low |
| Ordinary foot courier | 20–35 | Route km/calendar-day | Starting mobility assumption; replace with transport-system calculations |
| Ordinary mounted courier | 40–80 | Route km/calendar-day | Depends on terrain, rest and animal condition; low |
| Funded relay service | 100–300 | Route km/calendar-day | Broad calibration envelope; upper end informed by the exceptional Pony Express benchmark above |
| Scheduled dispatch interval | 0.5–7 | Days | Institution chooses frequency according to demand and budget |
| Initial print-run experiments | 25–1,000 | Copies/edition | Simulation search range, not an assertion about typical historical runs |

For example, **1,000 daily cases taking ten minutes each require approximately 28 clerks at six productive hours per day**, before supervision, inspection and correction. This is a synthetic workload calculation, not a historical staffing ratio.

For paper production, copying speed and document survival, the evidence assembled here does not support a universal number. Use explicit recipes and task costs, then calibrate particular settings rather than disguising guessed rates as historical measurements.

## 2.3 Price information should become stale at different rates

A proposed belief-weighting rule is:

\[
w(a)=e^{-a/\tau},
\]

where \(a\) is information age and \(\tau\) depends on its subject.

A genealogical claim, a legal rule and a market price should have different \(\tau\) values. Better still, let agents learn how quickly particular markets or institutions change. The historical telegraph evidence supports modeling information delay as an economic friction, but does not supply a universal exponential decay constant. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20150681)

---

# 3. Technology graph: capabilities, dependencies and concrete content

The following is a **24-node authoring proposal**. Several organizational nodes could be implemented as institution protocols rather than separate discoveries if the total graph needs to remain compact.

“AND” denotes a functional requirement. “OR” denotes an alternative. Requirements are proposed simulation dependencies, **not claims that every historical invention followed that exact path**.

Dates indicate a formative interval or a securely documented example. For poorly datable practices, an uncertain origin is preferable to an invented first year.

## 3.1 Recording, authentication and administration

| Node | Historical anchor | Functional prerequisites | Concrete unlocks |
| --- | --- | --- | --- |
| **I01 External counting and tallies** | Prehistoric; earliest counting interpretations are uncertain | Mark-making or manipulable objects AND shared convention | Tally sticks, counters, matched obligations, basic stock counts |
| **I02 Commodity numerals and measures** | Developed in late-fourth-millennium BCE Near Eastern administration | Counting AND agreed commodity/unit conventions | Quantity fields, conversion records, measurable obligations. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing) |
| **I03 Token accounting and envelopes** | Near Eastern tokens from approximately eighth millennium BCE; envelopes in fourth millennium BCE | I01 AND clay-working AND shared token meanings | Commodity counters, sealed token packages, delivery/debt representations. The token-to-writing sequence is a hypothesis to support, not a mandatory universal path. [University Blog Service](https://sites.utexas.edu/dsb/tokens/the-evolution-of-writing/) |
| **I04 Seals and authenticated closures** | Cylinder-seal examples by approximately 3200 BCE, Mesopotamia; other seal forms precede them | Carving/molding AND impression material AND recognized identity | Seal objects, sealed containers, authenticated documents, custody checks. [Spurlock Museum](https://www.spurlock.illinois.edu/collections/notable-collections/profiles/cylinder-seals.html?utm_source=chatgpt.com) |
| **I05 Structured quantitative records** | Approximately 3400–3200 BCE, southern Mesopotamia | I02 AND recordable medium AND agreed notation | Receipts, ration lists, inventories, obligations; tablet-writing service. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing) |
| **I06 Language-representing writing** | Late fourth millennium BCE Mesopotamia/Egypt; independently attested later in China and Mesoamerica | Shared representational convention AND teachable production/reading practice; **not necessarily I03** | Letters, named contracts, instructions, narratives and extended legal texts. [University Blog Service](https://sites.utexas.edu/dsb/tokens/the-evolution-of-writing/) |
| **I07 Alphabetic encoding variant** | Second millennium BCE, Sinai/Levant traditions | Phonemic convention AND instruction; can adapt an existing script | Alternative script with its own learning and production costs; not a prerequisite for literacy or printing generally. [University Blog Service](https://sites.utexas.edu/dsb/tokens/the-evolution-of-writing/) |
| **I08 Scribal curriculum and copying protocol** | Early literate societies; no single securely dated invention | A record system AND competent instructors AND supported training time | Scribe apprenticeship, copying workstations, model documents, standardized exercises; treat as institutional development rather than “education invented” |
| **I09 Organized archives and registers** | Early Near Eastern administrative traditions, late fourth–third millennia BCE; institutional boundaries uncertain | I05 OR I06 OR I10, AND storage, classification and custodians | Archive rooms, register series, retrieval tasks, copies, institutional memory; date is a broad administrative anchor, not a unique invention claim. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing) |
| **I10 Cord-based structured recording** | Securely documented in the Inka world, fifteenth–sixteenth centuries CE, with older Andean antecedents | Cordage AND knot/color conventions AND trained interpreters | Khipu-like inventory, tribute and population records; an alternative to script-based accounting. Broader narrative encoding remains incompletely understood. [Bard Graduate Center](https://www.bgc.bard.edu/research/articles/327/symposium-khipus-writing-histories-in) |

Numeral notation, writing and authentication should interoperate without requiring universal adoption. A merchant can use one script, a court another, and a warehouse a limited notation system.

## 3.2 Media, calculation and bookkeeping

| Node | Historical anchor | Functional prerequisites | Concrete unlocks |
| --- | --- | --- | --- |
| **I11 Portable writing media** | Blank papyrus roll around 2900 BCE, Egypt; surviving administrative texts around 2500 BCE | Suitable plants OR prepared skins/boards, AND inscription tools | Papyrus sheets/rolls and regional media variants; letters and lighter portable records. Variant dates must be authored separately rather than attributed to the papyrus date. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/papyrus-in-ancient-egypt) |
| **I12 Pulp-formed paper** | China, second century BCE archaeological evidence; Cai Lun’s 105 CE association is not the earliest evidence | Suitable fibers AND water AND beating AND sheet-forming/drying tools | Papermaking workshop, paper sheets, cheaper bulk document supply where inputs permit. [Robert C. Williams Museum of Papermaking](https://paper.gatech.edu/early-papermaking) |
| **I13 Positional calculation** | Babylonian positional mathematics by early second millennium BCE | Numerals AND place-value conventions AND training | Compact calculation, arithmetic tables, improved computational routines; zero is not a universal prerequisite. [Maths History](https://mathshistory.st-andrews.ac.uk/HistTopics/Zero/) |
| **I14 Zero and extended arithmetic algorithms** | Distinct developments; Indian arithmetic with zero securely represented by seventh century CE | Mathematical notation AND teaching tradition | Algorithm variants for calculation, finance and measurement; do not conflate placeholder and number. [Maths History](https://mathshistory.st-andrews.ac.uk/HistTopics/Zero/) |
| **I15 Linked double-entry accounts** | Italian banking: features in 1211; entity-wide systems near 1300; printed exposition in 1494 | Stable accounts AND arithmetic AND cross-reference rules AND trained clerks | Journals/ledgers, linked resource and liability accounts, reconciliation, periodic financial summaries. Paper and decimal numerals are useful complements, not logical necessities. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13358) |

**Media are not a simple quality ladder.** Author properties such as local input availability, weight, preparation labor, erasability, compatibility with printing, storage needs and failure modes. An imported luxury writing material can coexist with a cheap local medium for routine accounts.

## 3.3 Reproduction and communications

| Node | Historical anchor | Functional prerequisites | Concrete unlocks |
| --- | --- | --- | --- |
| **I16 Woodblock printing** | China by ninth century CE, with earlier precursors | Skilled carving AND suitable ink AND receiving surface | Page-block assets, block-printing workshops, repeatable illustrated texts and forms. [AFE East Asia](https://afe.easia.columbia.edu/songdynasty-module/tech-printing.html) |
| **I17 Reusable nonmetal type** | Bi Sheng, eleventh-century China | Character manufacture AND arrangement/alignment AND printing practice | Ceramic/wood type inventories, composing tasks, reusable character sets. [AFE East Asia](https://afe.easia.columbia.edu/songdynasty-module/tech-printing.html) |
| **I18 Cast metal movable type** | Korea, thirteenth century; Jikji, 1377, is an important surviving example | Metalworking AND repeatable character casting AND type organization | Type-making workshops, durable sorts, composable printing masters. **Not the invention of printing itself.** [Jikji to Gutenberg](https://jikji.utah.edu/) |
| **I19 Mechanical press printing** | Mainz, mid-fifteenth century European package | Pressure mechanism AND suitable ink AND blocks OR arranged type, AND suitable sheets | Press equipment, print shops, higher-volume production; no universal alphabet prerequisite. [Jikji to Gutenberg](https://jikji.utah.edu/) |
| **I20 Organized courier routes** | Ancient states and trading networks; Persian example documented in fifth century BCE | Route knowledge AND carriers AND funding/obligations | Dispatch service, message packets, delivery jobs, offices and schedules. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Herodotus/5B%2A.html) |
| **I21 Relay stations** | Ancient imperial systems; Persian stages provide a documented anchor | I20 AND provisioning AND transfer protocols AND staffed stages | Runner OR mounted relays; station buildings and recurring staffing/feed demand |
| **I22 Open-access and prepaid postal protocols** | Multiple regional developments; British 1840 reform is a clear mass-postage benchmark, not the first public post | I20 AND sorting/addressing AND payment/authorization rules | Public correspondence, postage accounting, collection/delivery offices, posted tariffs. [Royal Philatelic Society](https://www.rpsl.org.uk/rpsl/StandingDisplays/SD2016-02_Robin_Morton_Postal_History_of_Edinburgh_and_Leith/files/basic-html/page2.html) |
| **I23 Electric telegraph — post-v1** | Practical systems in 1830s–1840s; British patent in 1837 | Electrical power AND conductors/insulation AND instruments AND operators | Telegraph offices, wire links, encoded dispatch, rapid inter-office information. [London Museum](https://www.londonmuseum.org.uk/collections/v/object-730098/telegraph-single-needle-telegraph/) |
| **I24 Long-distance submarine cable — post-v1** | Durable transatlantic connection, 1866 | I23 AND marine cable manufacture/insulation AND laying/repair capability | Ocean-crossing message links; separate capital, maintenance and failure risks. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20150681) |

A relay can use runners rather than horses. Printing can use blocks rather than movable characters. Administration can use cord records rather than an alphabet. These OR paths are central to divergent histories.

## 3.4 Production recipes and persistent assets

| Output/service | Recipe or process to author | Principal bottleneck |
| --- | --- | --- |
| Written clay record | Prepared clay + stylus + recording labor → inscribed record; drying/firing variants | Skilled labor, handling and storage; do not automatically require a kiln for every record |
| Papyrus sheet/roll | Prepared pith strips → layered surface → pressure/drying → finishing | Suitable plant supply and skilled preparation; unlike paper, this is not a pulp-forming process. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/papyrus-in-ancient-egypt) |
| Paper | Suitable fibers → cleaning/preparation → beating in water → sheet formation → drying; optional sizing | Fiber preparation, water and sheet-handling capacity; powered beating should be an equipment upgrade, not a prerequisite for all paper. [Robert C. Williams Museum of Papermaking](https://paper.gatech.edu/early-papermaking) |
| Block-printed edition | Prepared page → carved block → proofing → ink and sheets → printing/finishing | Master preparation, paper supply and demand; keep the block as a reusable asset. [AFE East Asia](https://afe.easia.columbia.edu/songdynasty-module/tech-printing.html) |
| Movable-type edition | Sort inventory → composition → proofing → printing → redistribution of type | Composing labor, sufficient sorts, press capacity and working capital |
| Archive service | Storage space + custodians + classification/copying labor | Retrieval capacity, maintenance and access rules |
| Postal service | Carrier time + transport/provisions + sorting and delivery labor | Route capacity, frequency, security and last-mile access |

For books, represent a **content master**, **edition**, **physical copies** and **ownership/access** separately. Ten copies do not imply ten independently developed bodies of knowledge.

---

# 4. Variation and stylized facts

## 4.1 Era variation should change the bottlenecks, not impose gates

The following are modeling emphases, not mandatory stages.

| Setting | Appropriate information-system emphasis |
| --- | --- |
| **Foragers** | Personal knowledge, repeated communication, testimony and portable mnemonic objects. Model rich local knowledge without requiring documentary institutions. |
| **Early farming** | Stored resources and repeated obligations create reasons to count, witness and record. Specialist record services may be small and institutionally concentrated. |
| **Pre-industrial societies** | Multiple media and scripts coexist. Training, copying, archival retrieval, travel and institutional access constrain information use. Printing adoption depends on demand and production economics. |
| **Industrial societies** | Larger and more frequent flows justify standardized forms, specialized offices, cheap postal access and electrical communications. Preserve endpoint and handling bottlenecks. |
| **Modern societies** | Storage and transmission may become cheap relative to verification, interpretation and attention. For later TCE, retain access controls, incompatible systems, outages and uncertain claims rather than granting omniscience. |

The pre-industrial mix is directly illustrated by tablet administration, Egyptian documentary papyri, Andean cords, and East Asian combinations of block and movable-type printing. Later postal and telegraph evidence shows that organizational reforms and physical network changes are distinct. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing)

## 4.2 Regional differences that should survive abstraction

| Region/tradition | Historical distinction | TCE implication |
| --- | --- | --- |
| **Mesopotamia and Egypt** | Early economic records are prominent in Mesopotamia; Egypt’s papyrus corpus includes administrative documents, contracts, letters and religious material | Different media and institutional uses can develop together; never make the first surviving corpus the only permissible use. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing) |
| **China, Korea and Japan** | Papermaking and printing developed before the European press package; woodblocks remained important alongside movable type | Large script inventories affect production organization, not whether a society can print. Preserve competing reproduction methods. [Robert C. Williams Museum of Papermaking](https://paper.gatech.edu/early-papermaking) |
| **South Asia and mathematical transmission westward** | Indian developments in zero and arithmetic entered wider mathematical traditions | Mathematical procedures should travel as teachable content, independently of a society replacing its entire writing system. [Maths History](https://mathshistory.st-andrews.ac.uk/HistTopics/Zero/) |
| **Islamic-world paper traditions** | Papermaking spread through Central and western Asia before becoming established in later European centers | Paper is a transmitted craft with local inputs and specialists, not a European printing-era unlock. [Robert C. Williams Museum of Papermaking](https://paper.gatech.edu/early-papermaking) |
| **West Africa** | Timbuktu’s surviving manuscript traditions include substantial learned and documentary collections; the Library of Congress exhibit presents sixteenth–eighteenth-century examples | Do not classify sub-Saharan societies as uniformly without written knowledge or book culture. [Library of Congress](https://www.loc.gov/exhibits/mali/) |
| **Andes** | Khipus carried accounting and other records; their full encoding remains incompletely deciphered | Provide a structured-record path without language-writing as a hard dependency, while avoiding invented certainty about narrative capacity. [Zenodo](https://zenodo.org/records/6908343) |
| **Mesoamerica** | Writing developed independently from the Near Eastern and Chinese traditions | Discovery opportunities must permit independent invention, not require diffusion from one designated origin. [University Blog Service](https://sites.utexas.edu/dsb/tokens/the-evolution-of-writing/) |
| **Europe** | Commercial printing expanded through firms; double-entry developed before its printed exposition | Printing disseminates existing practices as well as enabling new ones; it should not retroactively become their invention prerequisite. [CEP | Centre for Economic Performance](https://cep.lse.ac.uk/pubs/download/dp1365.pdf) |

## 4.3 Stylized facts a successful simulation should reproduce

| Pattern | Validation test |
| --- | --- |
| **Records and literacy need not be universal to matter** | Low population literacy can coexist with substantial documentary administration through paid or institutionally supported specialists. Treat Egypt’s estimated range as one uncertain comparison, not a global target. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/papyrus-in-ancient-egypt) |
| **Writing does not immediately become general-purpose textual culture** | Allow restricted administrative notation before broader textual uses; the early Mesopotamian corpus provides an anchor. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing) |
| **Old and new reproduction methods coexist** | Short-run custom manuscripts and reusable page-blocks should remain viable after movable type appears. [AFE East Asia](https://afe.easia.columbia.edu/songdynasty-module/tech-printing.html) |
| **Printing concentrates unevenly** | Produce a distribution with many small printing centers and a few large ones, consistent with the historical mean–median gap above. [CEP | Centre for Economic Performance](https://cep.lse.ac.uk/pubs/download/dp1365.pdf) |
| **Fast relays require expensive supporting systems** | Removing provisions, staffing or route security should degrade performance even when the communication technique remains known. The Pony Express is a demanding upper benchmark, not a free speed multiplier. [National Park Service](https://www.nps.gov/poex/learn/historyculture/index.htm) |
| **Faster news changes decisions before it changes transport** | After a telegraph-like shock, price discrepancies should fall while shipment timing becomes more responsive; trade-flow volatility need not fall. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20150681) |
| **Information improvements have conditional economic effects** | Early printing’s estimated urban-growth association should emerge through demand, skills and exchange—not be applied directly to city population. [IDEAS/RePEc](https://ideas.repec.org/a/oup/qjecon/v126y2011i3p1133-1172.html) |

Add two **model-internal** tests: administrative waiting should rise sharply when work persistently exceeds processing capacity; and destroying a local archive should damage documented claims without directly destroying the physical assets those records described.

---

# 5. Representation in the Rust simulation

## 5.1 Minimum agent and institution state

| Entity | Recommended state |
| --- | --- |
| **Person** | Language/script competencies; numeracy; bookkeeping and audit skills; known procedures; learning time; institutional permissions; a bounded set of beliefs |
| **Record content** | Claim type, subjects, quantities/units, place, event date, creation date, author, version, provenance and uncertainty |
| **Physical document/bundle** | Content reference, medium, location, owner/custodian, condition, authentication and accessibility |
| **Institution** | Budget, staff, task queues, accepted forms, archive organization, jurisdiction, audit rules and access policy |
| **Message** | Sender, intended recipient, content reference, timestamp, priority, authorization, route and delivery state |
| **Printing establishment** | Equipment, type/block assets, paper inventory, edition jobs, staff skills, capital and unsold stock |

**The simulation’s authoritative ledger must not be an institution’s ledger.** The engine may know every transaction exactly for conservation checks while a merchant’s books contain only what its agents recorded.

Likewise, fast internal lookup does not mean fast historical retrieval. A Rust hash map may locate a record immediately, but the agent must still complete the simulated search task.

## 5.2 Avoid a document-shaped entity explosion

Use a shared `ContentId` with physical-copy counts or bundles. Instantiate individual documents only when ownership, transport, evidence or loss makes their identity important.

Examples:

* A warehouse’s daily routine records can be bundled by period and account.
* A disputed deed should be individually addressable.
* A thousand identical pamphlets can share content while being distributed in batches.
* A printed error belongs to an edition; copying errors can belong to a manuscript lineage.

This allows common-source mistakes without a separate simulation of every glyph.

Keep private beliefs sparse: agents need information relevant to their work, relationships and decisions, not a copy of the world database.

## 5.3 Model communication as events, not global diffusion scans

Schedule dispatch, arrival, handling and delivery events. Batch routine messages along common routes, but preserve priority and capacity limits.

A sensible update structure is task-level events for clerks and couriers, daily institutional workload updates, and slower maintenance checks for ordinary archive deterioration. Fires, seizures and major failures should trigger immediate events.

For discovery and learning, test only relevant opportunities: an artisan encountering a manual, an apprentice working with a teacher, a merchant observing a foreign accounting method. Avoid testing every agent against every technology every day.

UE5 can render clerks writing, people consulting public readers, couriers changing mounts and workers pulling presses. The Rust kernel should remain authoritative over the actual tasks and outputs.

## 5.4 Handle adoption and loss through people and operating decisions

For each institution–technology pair, estimate:

\[
\text{adoption value}
=
\text{expected service benefits}
-
\text{capital cost}
-
\text{training cost}
-
\text{operating cost}
-
\text{institutional resistance}.
\]

“Benefits” need not mean profit alone: religious circulation, prestige, administrative control or legal credibility can motivate expenditure.

Loss should operate through distinct channels: no surviving competent practitioner; unavailable materials; loss of instruction; obsolete or inaccessible records; or no institution willing to fund continued use.

Knowledge that is preserved but unused should be easier to recover than knowledge whose practitioners and records have both disappeared. Do not reduce all these cases to a binary researched/unresearched flag.

## 5.5 Existing research models worth borrowing

The following cover complementary pieces rather than supplying a complete historical ABM.

| Model/research tradition | What to borrow | What not to import uncritically |
| --- | --- | --- |
| **Basu et al., record-keeping and repeated exchange** | Persistent transaction memory can support reputation and cooperation | Laboratory effect sizes as prehistoric economic parameters. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0811967106) |
| **Shin et al.; Kohler, Bird and Wolpert, information-processing thresholds** | Administrative and communication demands interact with organizational scale | A universal population gate or automatic sequence of social development. [Nature](https://www.nature.com/articles/s41467-020-16035-9) |
| **Basu and Waymire, evolutionary bookkeeping** | Accounting systems adapt to organizational problems through gradual selection and learning | Double-entry as a sufficient cause of capitalism or universal firm superiority. [Columbia Business School](https://business.columbia.edu/sites/default/files-efs/imce-uploads/CEASA/Events%20Page/evolution_double-entry_bookkeeping.pdf) |
| **Steinwender, trade under information frictions** | Merchants make decisions from delayed, uncertain information; a communication shock changes expectations and trade | One sector’s estimated gain as a world-wide modifier. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20150681) |

For a 10k–50k-person world, **administrative complexity and distance matter more than reproducing the population of historical empires**. Do not silently treat each agent as hundreds of people to force a comparative threshold to fit.

## 5.6 Recommended v1 boundary

Prioritize actual information imperfections: basic records, differentiated skills, clerical workload, archives, delayed messages, and a small number of media/reproduction recipes. Add double-entry as a bookkeeping protocol over that foundation.

Defer detailed paleography, individual glyph layout, exhaustive cataloguing schemes and full text generation. These can add presentation detail later without improving the central causal model.

Calibrate with ablations: better media but no extra clerks; more clerks but no index; faster routes but restricted access; more books but no learning time. The resulting differences should be substantial.

---

# 6. Sources, datasets and uncertainty

## 6.1 Research and reference backbone

| Source | Why it matters |
| --- | --- |
| **Nissen, Damerow and Englund, *Archaic Bookkeeping* (1993)**; contextualized in Ira Spar’s Met essay | Foundational work on early accounting notation and administration; useful for authored record types and their economic context. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-origins-of-writing) |
| **Schmandt-Besserat, “The Evolution of Writing” (2014)** | Influential token-to-writing account. Use its archaeological sequence as one researched pathway, not a universal law of cultural evolution. [University Blog Service](https://sites.utexas.edu/dsb/tokens/the-evolution-of-writing/) |
| **Sangster, “The Emergence of Double Entry Bookkeeping” (2025; online 2024)**, DOI **10.1111/ehr.13358** | Detailed reassessment of early Italian evidence and the distinction between double entries and entity-wide systems. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13358) |
| **Basu et al., “Recordkeeping Alters Economic History by Promoting Reciprocity” (2009)**, DOI **10.1073/pnas.0811967106** | Experimental mechanism; read alongside Smith’s critique of its historical interpretation. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0811967106) |
| **Shin et al., “Scale and Information-Processing Thresholds in Holocene Social Evolution” (2020)**, DOI **10.1038/s41467-020-16035-9** | Comparative framework connecting information systems and social scale. [Nature](https://www.nature.com/articles/s41467-020-16035-9) |
| **Dittmar, “Information Technology and Economic Change: The Impact of the Printing Press” (2011)**, DOI **10.1093/qje/qjr035** | Econometric evidence on printing and urban development. [IDEAS/RePEc](https://ideas.repec.org/a/oup/qjecon/v126y2011i3p1133-1172.html) |
| **Dittmar, “New Media, Competition and Growth: European Cities After Gutenberg” (2015)** | Printing firms, entry costs, book production and market structure. [CEP | Centre for Economic Performance](https://cep.lse.ac.uk/pubs/download/dp1365.pdf) |
| **Steinwender, “Real Effects of Information Frictions: When the States and the Kingdom Became United” (2018)**, DOI **10.1257/aer.20150681** | Strong case study for separating information transport from goods transport. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20150681) |

Collection-based sources are especially useful for production detail: the Robert C. Williams Museum of Papermaking, Museum Plantin-Moretus, University of Utah’s Jikji project, and museum collections of seals, papyri and telegraph instruments. [Robert C. Williams Museum of Papermaking](https://paper.gatech.edu/early-papermaking)

## 6.2 Datasets to use

| Dataset | Useful fields or evidence | Main limitation |
| --- | --- | --- |
| **Cuneiform Digital Library Initiative** | Objects, dates, provenances, genres and transcriptions; sample real administrative record structures | Surviving and catalogued objects are not a random sample of original activity. [CDLI](https://cdli.earth/) |
| **Open Khipu Repository** | Cord, knot and color observations plus artifact metadata; downloadable structured database | Full interpretation remains incomplete; pin a version and respect revisions/deduplication. [Zenodo](https://zenodo.org/records/6908343) |
| **Seshat data underlying Shin et al.** | Comparative societal scale and information-system variables | Coding uncertainty, missingness and correlated institutional changes limit causal interpretation. [Nature](https://www.nature.com/articles/s41467-020-16035-9) |
| **Historical book catalogues and Dittmar’s printing data** | Printers, locations, dated editions and content categories | Editions/titles are not identical to copies sold, people reached or knowledge adopted. [CEP | Centre for Economic Performance](https://cep.lse.ac.uk/pubs/download/dp1365.pdf) |
| **Steinwender’s replication materials** | Data and code for the transatlantic information-friction study | A particular route, commodity and historical shock—not a universal trade dataset. [openICPSR](https://www.openicpsr.org/openicpsr/project/113066/version/V1/view?path=%2Fpcms%2Fprojects%2F1%2F1%2F3%2F0%2F113066%2FV1.0.1%2FAER-2015-0681_onlineappendix_dofile.do&type=file) |

## 6.3 Claims that deserve explicit uncertainty flags

**Origins:** distinguish the earliest surviving object, earliest convincing functional interpretation, invention, and widespread use. These are often separated by centuries or are not separately observable. Token sequences and the definition-dependent dating of double-entry illustrate the problem. [University Blog Service](https://sites.utexas.edu/dsb/tokens/the-evolution-of-writing/)

**Literacy:** reading a familiar document, composing prose and performing clerical work are different capabilities. Broad ancient literacy estimates should not become precise annual percentages or universal defaults. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/papyrus-in-ancient-egypt)

**Durability:** a surviving papyrus or medieval printing block demonstrates possible preservation under particular circumstances, not an average lifespan. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/papyrus-in-ancient-egypt)

**Causality:** cities, institutions and information technologies develop together. Printing-growth estimates, comparative thresholds and laboratory experiments each answer narrower questions than “how much did writing cause civilization?” [IDEAS/RePEc](https://ideas.repec.org/a/oup/qjecon/v126y2011i3p1133-1172.html)

**Thin quantitative evidence:** universal scribal throughput, clerical error rates, archive-loss probabilities, learning-hour requirements and administrative staffing ratios are not established by the sources assembled here. Keep them in a clearly labeled calibration layer.

---

## Bottom line

The most productive abstraction is an **information economy**: people spend time and resources creating claims, checking them, storing them, reproducing them and moving them to other people who may—or may not—understand and use them.

A tax register is not a tax collector. A balanced ledger is not an honest business. A printed manual is not a trained artisan. A fast message is not a shipment.

Keeping those distinctions operational should produce much of the historical effect of information technology without scripting the history itself.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92860-96c4-83ea-a985-2abc21ec83a3)
