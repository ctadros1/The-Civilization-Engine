# News, rumors, and laws in TCE

## Executive recommendation

**Model information as something carried, communicated, interpreted, and sometimes withheld—not as an expanding radius around an event.** Give messages access to the same roads, ships, households, workplaces, markets, and institutions that already organize people’s lives.

The crucial distinction is between **delivery to somebody**, **public awareness**, **understanding**, **belief**, and **action**. These need not happen together. News of Henri IV’s assassination in 1610 reached Venice eight days after the event but appeared in its principal handwritten gazette after fifteen days, when people were already discussing it. Publication was one stage in circulation, not its beginning. [Kent Academic Repository](https://kar.kent.ac.uk/65083/1/Postal%20Networks_NNEME.pdf)

For TCE, the most useful organizing principles are:

* **Transport and access govern long-distance arrival.** Institutions can make selected messages travel much faster than ordinary travelers.
* **Routines and relationships govern local coverage.** Being physically nearby does not guarantee hearing the news.
* **Message content, source relationships, and institutions govern fidelity and acceptance.** A faithfully copied message can be false; an oral report can be accurate.

There is good evidence for particular routes, communication institutions, and transmission mechanisms. There is much weaker evidence for universal quantities such as “the percentage of medieval villagers who knew a new law” or “the probability of distortion per oral retelling.” The parameter tables below therefore separate **historical anchors** from **explicitly proposed simulation priors**.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Separate the stages of information acquisition

Use the following conceptual pipeline. These are proposed TCE state distinctions, not historical categories that every society explicitly recognized.

| Stage | Meaning in TCE | Example |
| --- | --- | --- |
| Event | Something happens in the world | A ruler dies |
| Detection | An observer notices something | A servant discovers the body |
| Report creation | Someone formulates a claim | “The ruler died during the night” |
| Transmission | A carrier, speaker, or institution conveys it | A messenger leaves for another town |
| Awareness | A recipient encounters the claim | A merchant hears that the ruler is dead |
| Comprehension | The recipient understands relevant details | The merchant knows which ruler and when |
| Belief | The recipient assigns credibility | The merchant suspects a succession maneuver |
| Disclosure | The recipient chooses whether and how to repeat it | They tell relatives but avoid public discussion |
| Action | The claim changes a decision | They postpone a shipment or support a claimant |

**Implementation rule:** the objective world state may generate observations, but decision-making agents must query their own information state. They must not silently consult the simulation’s authoritative event database.

Also separate an agent’s belief that a claim is true from their belief that **other people believe it**. A proclamation or public ceremony can matter because attendees see one another hearing it, even when some privately disbelieve its content.

### 1.2 Calculate arrival from journeys and queues, not straight-line distance

A useful decomposition is:

\[
T\_{\text{arrival}} =
T\_{\text{detection/release}}
+\sum\_e\left(T\_{\text{departure wait},e}
+T\_{\text{travel},e}
+T\_{\text{transfer},e}\right)
+T\_{\text{local delivery}}.
\]

This is a modeling equation, not a fitted historical law.

The important variables are route availability, carrier departure times, permissions, transport capacity, weather, security, and handoffs. A document waiting for a carrier is not moving at a reduced continuous speed: **it is stationary until departure**.

Assyrian royal correspondence illustrates the institutional distinction. Relay stations supported rapid official communication, while especially sensitive messages could travel with a trusted envoy who accompanied them throughout the journey. Sealed documents and identifiable senders supported authentication, but access to the system was not equivalent to a public postal service. [epub.ub.uni-muenchen.de](https://epub.ub.uni-muenchen.de/27226/1/Radner_Official_Epistolography_2015_Royal_Pen_Pals.pdf)

**TCE rules:**

Give every transport service an access policy, schedule or dispatch condition, capacity, price or labor obligation, and reliability. A ruler may requisition an urgent courier; a merchant may wait for a partner’s caravan; an ordinary household may entrust a letter to a relative.

Relay speed must consume real resources: staffed stations, replacement animals or runners, food, equipment, and secure passage. Underfunding should produce missed handoffs and service interruptions—not merely an abstract reduction in “communication technology.”

Use actual navigable routes for sea and river communication. Seasonality should change departure opportunities and journey-time distributions. Even Roman discussions of dangerous sailing seasons should not be turned into an absolute winter prohibition: the historical evidence includes exceptions. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-studies/article/speed-of-the-roman-imperial-post/76A89AF688E28F402333A032CC1AA40A)

### 1.3 Treat markets, gatherings, and workplaces as timed communication opportunities

A market is both a destination for travelers and an encounter between otherwise separated social circles. For modeling, represent it as a **scheduled group-contact event**, alongside assemblies, religious gatherings, courts, work groups, households, and communal meals.

Do not automatically expose everyone in a settlement. Attendance, location, language, attention, social access, and timing determine who hears a particular speaker.

Darnton’s reconstruction of eighteenth-century Paris shows information passing through mixed settings and media: conversations, reading aloud, memorized verse, written copies, and songs. A person did not need to read a publication personally to encounter its contents. [AHA](https://www.historians.org/presidential-address/robert-darnton/)

**TCE rules:**

A crier should follow a route through designated announcement points, pause, attract an audience, and deliver an authorized message. Model the reachable audience from actual nearby agents and attendance behavior. Anyone absent must hear later, consult a posted copy or intermediary, or remain unaware.

A household member returning from market becomes a potential secondary transmitter—not a guaranteed household-wide update.

For regularly scheduled opportunities, waiting time can dominate travel. If a departure occurs every \(P\) days and events occur independently of that schedule:

\[
E[T\_{\text{wait}}]=P/2.
\]

That is a mathematical consequence of the scheduling assumption. It is not evidence that all historical markets were weekly.

### 1.4 Distinguish access to writing from access to information

For TCE, literacy should not be a single gate between “informed” and “uninformed.” Separate reading, writing, language competence, access to a reader or scribe, and familiarity with an administrative register.

Likewise, distinguish the effects of writing and printing:

**Writing** permits a persistent, inspectable message and comparison against an earlier copy. **Printing** reduces the cost of making many substantially identical copies. Neither automatically makes their physical transport faster or their contents true.

Research on European postal networks links predictable postal arrivals to periodic manuscript news and newspapers. The same communication environment also included merchants, travelers, religious institutions, and private carriers. [Kent Academic Repository](https://kar.kent.ac.uk/65083/1/Postal%20Networks_NNEME.pdf)

**TCE rules:** let a written message become an oral report through public reading; let an oral report become a document through a scribe. Each conversion can change detail, audience, authority, and cost. A printed error can become unusually widespread precisely because many copies reproduce it consistently.

### 1.5 Model rumor as selective reconstruction, not automatic degradation

“Rumor” should mean an **unverified circulating claim**, not necessarily a false one.

Laboratory transmission experiments by Mesoudi, Whiten, and Dunbar found that social information could be transmitted more accurately and in greater quantity than nonsocial material. This supports content-sensitive transmission, not a universal rule that oral communication rapidly destroys information. These experiments are modern evidence about mechanisms, not measurements of ancient populations. [British Psychological Society](https://bpspsychub.onlinelibrary.wiley.com/doi/abs/10.1348/000712605x85871)

Represent a message as structured components:

```
subject / event type / location / time / quantity /
attributed cause / attributed source / recommended action
```

Apply different processes to different components:

| Process | Proposed implementation |
| --- | --- |
| Omission | Peripheral details disappear while the main event survives |
| Compression | Several propositions become a shorter summary |
| Quantity distortion | Exact numbers become rounded estimates or qualitative categories |
| Attribution change | “A traveler said…” becomes “People in that town say…” |
| Interpretive change | A speaker supplies a cause or motive not present in the observation |
| Deliberate alteration | An agent changes content to advance an interest |
| Verification | A document, witness, inspection, or independent report restores or challenges details |

Crucially, **fidelity to the received message and correspondence to reality are separate measurements**.

Sharing should depend on relevance, urgency, novelty, social usefulness, source relationships, and risk. A person may repeat something they doubt because it is entertaining, or keep something they believe secret because disclosure is dangerous.

### 1.6 Track corroboration without turning repetition into independent evidence

A practical belief update can use source credibility, consistency with observations, and independent corroboration. But retain some provenance.

Ten people repeating one crier are not ten independent witnesses. Conversely, two independently arriving traders describing the same ruined bridge provide different evidence from two acquaintances repeating the same tavern story.

**Implementation rule:** retain a bounded source-origin identifier or approximate provenance fingerprint. Discount correlated reports. Do not require a complete, indefinitely growing transmission tree.

Keep awareness diffusion distinct from social reinforcement for action. Centola and Macy’s complex-contagion model distinguishes information that can pass through a single contact from behaviors requiring multiple reinforcing contacts. This is particularly useful for separating “heard about a revolt” from “willing to join it.” [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/521848)

### 1.7 Make promulgation an institutional process

A proposed legal-information pipeline is:

\[
\text{enactment}
\rightarrow \text{authenticated copy}
\rightarrow \text{dispatch}
\rightarrow \text{local receipt}
\rightarrow \text{explanation/publication}
\rightarrow \text{situational learning}
\rightarrow \text{enforcement}.
\]

Historical governments explicitly provided for repetition. Ashoka’s Separate Kalinga Edict II prescribed hearing the edict every four months, with additional readings between those occasions. Qing local officials were instructed to deliver public lectures reinforcing the Sacred Edict twice monthly. These are strong evidence of **intended communication schedules**, not evidence that every resident attended or understood. [Access to Insight](https://www.accesstoinsight.org/lib/authors/dhammika/wheel386.html)

**TCE should maintain three different legal states:**

1. The institution’s authoritative rule and effective date.
2. The version known to a particular official or court.
3. The rule as understood by an individual subject.

These can disagree. An official may enforce an obsolete instruction; a resident may know the general prohibition but misunderstand an exception or penalty.

Represent legal knowledge by relevant domain rather than requiring memorization of an entire code. A trader may understand weights, tolls, and debt procedures; a farmer may understand customary access rights but not a newly enacted commercial rule.

Customs should spread through repeated practice, instruction, disputes, and sanctions. Their existence should not depend on a written proclamation. Conversely, writing a law must not instantly erase older expectations.

**Binding force and actual awareness must remain separate.** Whether ignorance excuses a violation, whether a grace period exists, and whether local publication is legally required are institutional choices—not automatic simulation assumptions.

### 1.8 Give censorship and propaganda different mechanisms

Censorship changes what can be transmitted, where, by whom, and at what risk. Propaganda changes the supply, prominence, framing, and repetition of messages. Neither requires that the underlying content be wholly false.

Darnton’s study of the 1749 Affair of the Fourteen follows a crackdown on political verse circulating through recitation, song, dictation, and manuscript copies. It demonstrates both repression and a communication network that could not be reduced to control of printing presses. It does not establish a general numerical “censorship failure rate.” [JSTOR](https://www.jstor.org/stable/j.ctv1m46g0h?utm_source=chatgpt.com)

**TCE rules:**

Give authorities limited inspection capacity over carriers, presses, public venues, and informants. Detection depends on visibility and network position. Punishment changes the expected cost of public repetition; it need not change private belief.

Model propaganda through paid speakers, official announcements, sponsored publications, ceremonies, and selective reporting. Its effects should depend on credibility, relevance, competing evidence, and audience access.

Modern evidence cautions against treating censorship as uniform removal of criticism. King, Pan, and Roberts found that the Chinese online censorship system they studied particularly targeted material associated with collective action, including material not simply hostile to government. This is a useful **possible institutional strategy**, not a template for all states or an ancient parameter estimate. [Gary King](https://gking.harvard.edu/publication/how-censorship-in-china-allows-government-criticism-but-silences-collective-expression/)

Do not hard-code either “repression always works” or “repression always backfires.” Both outcomes should depend on substitution channels, credibility, enforcement resources, and the consequences of being caught.

---

## 2. Quantitative parameters and historical anchors

### 2.1 Evidence-based anchors

**Confidence refers to the particular statement and source—not to its transferability to another society.** Delivery times below are not times until an entire population became aware.

| Quantity | Value and units | Context and source | Confidence and interpretation |
| --- | --- | --- | --- |
| Ordinary foot travel | About **17 Roman miles/day**, approximately **25 km/day** | Estimates discussed by A. M. R. Ramsay, *The Speed of the Roman Imperial Post* (1925). [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-studies/article/speed-of-the-roman-imperial-post/76A89AF688E28F402333A032CC1AA40A) | **Medium–low:** historical reconstruction, not a universal measured average |
| Travel when driving | About **25 Roman miles/day**, approximately **37 km/day** | Same discussion. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-studies/article/speed-of-the-roman-imperial-post/76A89AF688E28F402333A032CC1AA40A) | **Medium–low:** mode, route, and circumstances matter |
| Normal courier travel in that reconstruction | About **50 Roman miles/day**, approximately **74 km/day** | Same discussion. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-studies/article/speed-of-the-roman-imperial-post/76A89AF688E28F402333A032CC1AA40A) | **Medium–low:** do not confuse with exceptional urgent relays |
| Assyrian long-distance official delivery | Approximately **700 km straight-line separation**; delivery unlikely in **much less than 5 days** | Radner’s reconstruction for Quê–central Assyria. [Open Access LMU](https://epub.ub.uni-muenchen.de/27226/1/Radner_Official_Epistolography_2015_Royal_Pen_Pals.pdf) | **Medium for order of magnitude:** a plausibility estimate, **not a recorded five-day maximum** |
| Inka runner relay capability | About **240 km/day** | Smithsonian National Museum of the American Indian educational reconstruction. [National Museum of the American Indian](https://americanindian.si.edu/static/inkaroad/pdf/inka-teachers-guide.pdf) | **Low–medium:** approximate capability; weaker than a series of dated dispatches |
| Late-sixteenth-century European postal transit | Rome–Venice **4–5 days**; Rome–Vienna **12–15 days**; Rome–Paris **20–25 days** | Schobesberger et al., *European Postal Networks*. [Kent Academic Repository](https://kar.kent.ac.uk/65083/1/Postal%20Networks_NNEME.pdf) | **Medium:** reported normal travel times for particular connections |
| Urgent arrival versus publication | Henri IV assassination: Venice arrival **day 8**, principal manuscript gazette **day 15** | Event in 1610; Schobesberger et al. discuss Infelise’s reconstruction. [Kent Academic Repository](https://kar.kent.ac.uk/65083/1/Postal%20Networks_NNEME.pdf) | **Medium–high:** unusually valuable separation of two communication stages |
| Premium nineteenth-century relay service | More than **1,800 miles in 10 days**: over **290 km/day** averaged over the route | U.S. National Park Service, Pony Express history. [National Park Service](https://www.nps.gov/poex/learn/historyculture/index.htm) | **High for service-scale description:** not ordinary personal travel |
| Qing public instruction frequency | **2 lectures/month** | Prescribed local instruction around the Sacred Edict. [AFE East Asia](https://afe.easia.columbia.edu/ps/china/wang_youpu_exhortations.pdf) | **High for prescription; unknown for attendance and comprehension** |
| Ashokan edict repetition | **Every 4 months**, with additional readings | Separate Kalinga Edict II. [Access to Insight](https://www.accesstoinsight.org/lib/authors/dhammika/wheel386.html) | **High for translated prescription; implementation uncertain** |
| Intercamp contact in an observed forager network | Over one month, **28%** of possible forest intercamp adult pairs and **56%** of coastal pairs had recorded proximity contacts | Agta study: **53 adults in 7 forest camps**, **37 adults in 3 coastal camps**. [ResearchGate](https://www.researchgate.net/publication/339587433_Hunter-gatherer_multilevel_sociality_accelerates_cumulative_cultural_evolution) | **High for this sample’s reported observations; low prehistoric transferability** |
| Modern false/true news diffusion contrast | True stories took about **6 times longer** to reach **1,500 people** | Vosoughi, Roy, and Aral’s Twitter study, 2006–2017. [Institute for Security and Technology](https://securityandtechnology.org/wp-content/uploads/2020/07/1146.full_.pdf) | **High for the reported study result; unsuitable as a universal rumor multiplier** |

The Inka and Pony Express figures are particularly useful reminders that **relaying a small message is not the same task as transporting one person all the way**. They should not become general movement speeds for citizens.

### 2.2 Proposed TCE priors and sensitivity ranges

The following numbers are **design priors**, not discovered historical measurements. Their empirical confidence is low. They provide starting values and experiments until TCE’s mobility, social-contact, and institutional systems generate more of the behavior directly.

| Parameter | Suggested starting range or sweep | Unit | Modeling use |
| --- | --- | --- | --- |
| Ordinary overland message-bearing travel | **20–40** | km per travel day | Fallback for favorable walking routes; actual journey system should override |
| Non-relay mounted courier | **40–80** | km per travel day | Sustained service, not a horse’s maximum running speed |
| Maintained urgent relay corridor | **100–250** | km per elapsed day | Test only with adequate staffing and continuous operation |
| Scheduled dispatch interval | **1, 7, 14, or 30** | days | Author service schedules; do not assign one interval to an entire era |
| News-relevant conversational opportunities | **2–8** | opportunities per agent-day | A sampled subset of encounters, not all conversation |
| Routine claim sharing | **0.05–0.25** | probability per suitable opportunity | Conditional on having something relevant to tell |
| Urgent, personally relevant claim sharing | **0.4–0.9** | probability per suitable opportunity | Repression, distrust, or secrecy can still suppress transmission |
| Oral detail mutation | Sweep **0%, 1%, 5%, 10%** | probability per field per retelling | Sensitivity test; separate omission, numerical change, and attribution |
| Transient-news retention half-life | **3–14** | days without reinforcement | For retrieval or sharing propensity, not all permanent knowledge |
| Personally important claim retention | **30–180** | days without reinforcement | Major relationships and durable legal knowledge require separate treatment |
| Comprehension under language/register barriers | Sweep **0.25–1.0** | probability conditional on exposure | Prefer generation from language and explanation mechanics |

Do not tune all these parameters against a single awareness curve. More contacts, more sharing, and slower forgetting can produce similar aggregate coverage while creating very different social behavior.

**Prefer endogenously generated quantities:** attendance from schedules, contacts from daily routines, route times from transport, and institutional repetition from policy. Use free parameters mainly for attention, interpretation, and disclosure.

### 2.3 Worked example: two settlements 90 km apart

Assume, for illustration, a usable 90 km route, no disruption, and news created at a random point in a weekly departure cycle.

A merchant traveling at 30 km/day takes three travel days. With a seven-day departure interval, expected waiting is 3.5 days: **expected first arrival is 6.5 days**.

A dedicated courier dispatched immediately at 60 km/day arrives in **1.5 days**.

A continuously operating relay achieving 200 km/day delivers the message in approximately **0.45 days**.

These are **calculated scenario outputs**, not historical observations. None establishes when most destination residents know. A message arriving at night, delivered privately, and not repeated until the next gathering can remain socially restricted. Coordinated schedules also mean successive waiting times are not necessarily independent.

---

## 3. Variation across societies and periods

TCE should implement **capabilities and institutional combinations**, not a fixed progression from “primitive oral” to “advanced literate.”

| Setting | Evidence or important distinction | Consequence for TCE |
| --- | --- | --- |
| **Foragers and mobile communities** | Observed Agta networks combine within-camp clustering with substantial intercamp connections. These are contemporary ethnographic observations, not direct prehistoric measurements. [ResearchGate](https://www.researchgate.net/publication/339587433_Hunter-gatherer_multilevel_sociality_accelerates_cumulative_cultural_evolution) | Small settlements need not be isolated. Visits, partnership ties, and camp movement can create long-distance connections without state infrastructure. |
| **Early farming settlements** | Precise prehistoric news latencies are not recoverable from the evidence reviewed here. Exchange objects establish connections, not the time taken by a particular report. | Treat communication patterns as hypotheses generated by agricultural routines, seasonal exchange, marriage, assemblies, and settlement spacing. Do not fabricate a prehistoric awareness percentage. |
| **Agrarian empires of western Asia** | Assyrian correspondence demonstrates authenticated official communication, relay infrastructure, and distinctions between rapid transport and trusted personal delivery. [epub.ub.uni-muenchen.de](https://epub.ub.uni-muenchen.de/27226/1/Radner_Official_Epistolography_2015_Royal_Pen_Pals.pdf) | Official access can make distant administrators better informed than nearby ordinary households. Institutional capacity matters more than settlement distance alone. |
| **South Asia** | Ashoka’s edicts combine durable inscriptions with instructions for recurring oral presentation; his Fourteenth Rock Edict also acknowledges variation in length and local presentation. [Access to Insight](https://www.accesstoinsight.org/lib/authors/dhammika/wheel386.html) | Store official variants and local presentations. Do not assume one identical text was encountered everywhere. |
| **East Asia** | Qing instruction joined an authoritative text with repeated local oral explanation. The prescribed teaching effort does not establish universal reception. [AFE East Asia](https://afe.easia.columbia.edu/ps/china/wang_youpu_exhortations.pdf) | Separate the central message, the local interpreter, attendance, and practical understanding. |
| **Andean states** | Inka communication combined trained runners, spoken messages, and khipu record-keeping; the extent of non-numerical khipu encoding should not be treated as fully resolved. [National Museum of the American Indian](https://americanindian.si.edu/static/inkaroad/pdf/inka-teachers-guide.pdf) | Do not require horses or alphabetic literacy for efficient state messaging. Specialized training and labor organization can substitute for other technologies. |
| **North Africa, the Mediterranean, and Indian Ocean connections** | Geniza letters preserve merchant and family communication across these regions. One 1103 letter records advice about India and an explicit expectation of confidentiality. [Princeton Geniza Project](https://geniza.princeton.edu/en/documents/9690/) | Model trust, selective disclosure, business contacts, and language communities—not only state mail. |
| **European manuscript and print environments** | Postal schedules, manuscript newsletters, oral exchange, and print operated together rather than replacing one another cleanly. [Kent Academic Repository](https://kar.kent.ac.uk/65083/1/Postal%20Networks_NNEME.pdf) | New media add channels. They do not automatically delete older ones. |
| **Industrial communication** | The Pony Express operated in conjunction with advancing telegraph connections: some telegrams traveled by horse only between the wire endpoints. [National Park Service](https://www.nps.gov/poex/learn/historyculture/index.htm) | Allow mixed-mode routes. Fast long-distance signaling can coexist with slow local delivery and restricted access. |
| **Modern communication** | Very rapid circulation still does not imply equal reach, accuracy, or acceptance. The Twitter results concern a particular platform and identified news cascades. [Institute for Security and Technology](https://securityandtechnology.org/wp-content/uploads/2020/07/1146.full_.pdf) | Network distribution, attention, source selection, and disclosure remain important after physical transmission becomes fast. |

Gender, occupation, wealth, age, status, and language should influence **access through institutions and routines**, not immutable information penalties. The Women’s Early Modern Letters Online project, for example, documents substantial female correspondence while also exposing how cataloguing practices affect whose networks are visible. [Early Modern Letters Online](https://emlo-portal.bodleian.ox.ac.uk/collections/?page_id=2595&utm_source=chatgpt.com)

---

## 4. Stylized facts a good simulation should reproduce

These are proposed validation targets derived from the evidence and mechanisms above—not universal numerical laws.

| Pattern | What to test |
| --- | --- |
| **Network position beats proximity** | A distant place on an active courier corridor can learn before a closer settlement without a suitable connection. |
| **Arrival is not publication** | Record first private arrival, first public presentation, and population awareness separately. The 1610 example supplies a historical benchmark. |
| **Urgent messages use different services** | Important news can overtake routine correspondence because someone pays for, authorizes, or organizes extraordinary transmission. |
| **Information arrives in bursts** | Markets, carrier arrivals, assemblies, and publication schedules produce discontinuities rather than smooth radial spread. |
| **Coverage has a long tail** | Local clusters learn quickly while peripheral households remain uninformed. Low-interest news may never reach most residents. |
| **Gist and details diverge** | Many people may correctly know “a new toll exists” while disagreeing about its rate, scope, starting date, or collector. |
| **Public expression differs from private belief** | Repression can reduce visible discussion without eliminating awareness or opposition. |
| **Written and oral circulation interlock** | A single document can reach nonreaders through intermediaries; circulating speech can enter written records. |
| **Repeated reports can share one origin** | Apparent corroboration should sometimes collapse when agents discover that every report came from the same source. |
| **Infrastructure can deteriorate without technological forgetting** | Loss of staff, animals, funds, security, or political cooperation can lengthen delivery times despite retained knowledge of roads and writing. |
| **Institutions repeatedly teach important messages** | Long-lived rules and official narratives receive reinforcement rather than relying on one announcement. |
| **No universal falsehood advantage** | Some false claims spread widely; others fail. Some accurate information remains obscure. Content and networks determine the outcome. |

### Measure distributions, not just averages

For each claim and each relevant population, record:

\[
A(t)=\frac{\text{agents aware at }t}{\text{target population}},
\]

alongside correct-gist coverage, correct-detail coverage, belief, public expression, and behavioral response.

Report \(T\_{10}\), \(T\_{50}\), and \(T\_{90}\) only when those thresholds are reached. Otherwise report “not reached” and final coverage. This prevents a failed or highly restricted diffusion process from being misrepresented as merely slow.

Also distinguish:

* probability of eventual delivery;
* delivery time conditional on delivery;
* maximum awareness reached;
* time spent believing an incorrect version.

A lost letter is not an infinitely slow successful letter.

---

## 5. Recommended agent and institution representation

### 5.1 Core data structures

A compact design can preserve individual experience without storing free-form prose for every conversation.

| Object | Suggested contents |
| --- | --- |
| **World event** | Objective occurrence, location, time, participants; hidden from agents unless observed |
| **Claim variant** | Structured propositions, uncertain quantities, attributed source, parent/origin reference |
| **Agent information record** | Claim reference, first/last exposure, confidence, understood fields, provenance summary, disclosure disposition |
| **Law version** | Issuing authority, jurisdiction, affected categories, terms, enactment/effective dates, supersession |
| **Institutional information store** | Received official versions, archives, outgoing queue, publication policy, access rules |
| **Communication service** | Routes, schedules, permissions, capacity, resource requirements, reliability |
| **Public presentation event** | Speaker, content, location, duration, actual audience, language/register |

Absence from an agent’s sparse information store represents unawareness. Do not allocate an “unaware” record for every possible agent–event combination.

**Separate evidence from conclusions.** An agent who sees empty grain stalls has local evidence. “The whole kingdom has suffered a harvest failure” is an inference, not the same observation.

### 5.2 Event-driven update sequence

A workable update cycle is:

```
world event → eligible witnesses create observations
journey arrival → recipient receives carried messages
social encounter → select a relevant claim, decide whether to disclose
transmission → create or reuse a variant, update recipient's information
public presentation → process eligible attendees as a group
institutional action → dispatch, archive, publish, suppress, or correct
decision → use the agent's perceived world and perceived rules
```

Use actual daily-life encounters for local diffusion and existing travel itineraries for intersettlement transmission. A merchant or pilgrim becomes an information bridge because they move between social settings—not because they possess a permanent arbitrary “rumor radius.”

For law enforcement, the deciding official should use the version they possess. The simulation can still know that the decision conflicts with the authoritative law, enabling appeals, disputes, and later correction without granting the official omniscience.

### 5.3 Fidelity without language-model calls

Most simulation-relevant distortion can be authored as transformations:

```
“40 soldiers arrived yesterday”
→ “many soldiers arrived recently”
→ “an army is gathering”
```

But transformations should preserve distinctions. The final version is a new inference or embellishment, not merely the same message with lower precision.

Useful authored operations include rounding quantities, removing dates, substituting a broader place name, confusing similarly named people, omitting exceptions, attributing motives, and attaching an action recommendation.

Use prose generation primarily for the player-facing presentation of these structured states. The Rust kernel should own the actual claim, evidence, and belief mechanics.

### 5.4 Performance and persistence

Avoid all-pairs communication and scanning every known claim every day. Sample suitable contacts, process scheduled public events, and activate claims by relevance.

As an **engineering illustration**, 50,000 agents with 64 active records of 32 bytes each require about **102.4 MB of raw record storage**, before indexing and allocation overhead. Four selected communication opportunities per agent-day imply **200,000 opportunities per simulated day**. These are arithmetic budgets, not measured performance benchmarks.

An active-record cap is not a claim about human memory capacity. Keep durable relationship knowledge, locally relevant legal summaries, and institutional archives outside an ephemeral news cache. Use lazy decay and retrieval probabilities rather than globally deleting forgotten facts.

Share immutable claim variants between agents. Preserve only bounded provenance summaries unless a particular event deserves detailed historical tracing.

### 5.5 Existing models and games worth borrowing from

| Model or game | Useful feature | What TCE must add or avoid |
| --- | --- | --- |
| **NetLogo Rumor Mill** | A transparent neighborhood-transmission baseline, tracking who has heard a rumor and repeated exposure. [CCL](https://ccl.northwestern.edu/netlogo/models/RumorMill) | Its simple spatial diffusion is not a historical communication model. Add mobility, selective sharing, institutions, forgetting, and variant content. |
| **Centola–Macy complex contagion** | Separates single-contact transmission from adoption requiring reinforcement. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/521848) | Do not require multiple confirmations merely to become aware. Apply reinforcement to selected beliefs and actions. |
| **Dwarf Fortress, documented 2014 implementation** | Developer notes connect witnesses, rumor circulation, and reputation; they also describe differences between private and expressed political opinions. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2014.html?utm_source=chatgpt.com) | A valuable architectural precedent, not an empirical calibration. The same notes acknowledge some information shortcuts. |
| **Dwarf Fortress, documented 2017 development** | Traveling agents collect particular rumors and bring them home. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2017.html?utm_source=chatgpt.com) | Connect this pattern to TCE’s real itineraries, source reliability, and institutional reporting requirements. |

**Recommended implementation order:** begin with accurate structured claims, actual carriers, selective local sharing, and separate awareness/belief. Add legal-version propagation next. Introduce distortion and strategic information control only after the undistorted system produces credible arrival and coverage patterns.

---

## 6. Sources, datasets, and limits of inference

### A practical calibration workbench

| Resource | Best use | Important limitation |
| --- | --- | --- |
| **Radner, “Royal Pen Pals” (2015)** | Assyrian dispatch practices, authentication, official networks, and institutional constraints. [epub.ub.uni-muenchen.de](https://epub.ub.uni-muenchen.de/27226/1/Radner_Official_Epistolography_2015_Royal_Pen_Pals.pdf) | Elite correspondence and reconstructed transit, not population-wide awareness |
| **Ramsay, “The Speed of the Roman Imperial Post” (1925)** | Classical evidence and distinctions among travel conditions. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-studies/article/speed-of-the-roman-imperial-post/76A89AF688E28F402333A032CC1AA40A) | Old reconstruction; retain uncertainty rather than treating estimates as universal constants |
| **Schobesberger et al., “European Postal Networks” (2016)** | Routes, schedules, access, and news/publication lags. [Kent Academic Repository](https://kar.kent.ac.uk/65083/1/Postal%20Networks_NNEME.pdf) | Regional and institutional specificity |
| **Ashokan edicts; Qing Sacred Edict lecture texts** | Authored examples of public instruction, repetition, and localized explanation. [Access to Insight](https://www.accesstoinsight.org/lib/authors/dhammika/wheel386.html) | Prescriptions reveal intentions more directly than compliance |
| **Darnton, *Poetry and the Police* (2010)** | Transmission chains, mixed media, variant texts, and repression. Darnton identifies nine manuscript copies of one changing song. [JSTOR](https://www.jstor.org/stable/j.ctv1m46g0h?utm_source=chatgpt.com) | Police-selected cases and elite-mediated testimony cannot supply a representative opinion survey |
| **Early Modern Letters Online—Hotson and Lewis, editors** | Correspondence metadata, senders, recipients, places, and links to editions; sixteenth–eighteenth centuries. [EMLO](https://emlo.bodleian.ox.ac.uk/about?utm_source=chatgpt.com) | A catalogue date is not automatically an arrival date |
| **Princeton Geniza Project** | Merchant, household, multilingual, and confidential correspondence; editions and translations of individual documents. [Princeton Geniza Project](https://geniza.princeton.edu/) | Surviving letters are a selective archive, not a random sample of everyone’s communication |
| **Migliano et al., “Hunter-gatherer multilevel sociality accelerates cumulative cultural evolution” (2020)** | Empirical proximity-network structure for testing clustered and intercamp contacts. [ResearchGate](https://www.researchgate.net/publication/339587433_Hunter-gatherer_multilevel_sociality_accelerates_cumulative_cultural_evolution) | Proximity is not proof of conversation; simulated cultural outcomes are not measured news latencies |
| **Mesoudi, Whiten, and Dunbar (2006); Vosoughi, Roy, and Aral (2018)** | Content selection and modern diffusion mechanisms. [British Psychological Society](https://bpspsychub.onlinelibrary.wiley.com/doi/abs/10.1348/000712605x85871) | Laboratory and platform effects should not become premodern coefficients |

When extracting quantitative delivery data, distinguish the date of an event, composition, dispatch, receipt, copying, and publication. A reply may supply an upper bound on receipt, but its date does not reveal the exact arrival time. Record uncertain dates as intervals.

The largest remaining uncertainties concern **ordinary people outside preserved correspondence networks, actual proclamation audiences, comprehension of exceptions, and distortion rates in natural conversation**. Archaeology, legal prescriptions, and elite letters illuminate different parts of the process; none alone measures all of them.

**The strongest TCE design is therefore a constrained generative model:** calibrate journeys and institutional schedules where evidence is relatively strong, generate encounters from persistent lives, and expose uncertainty in attention and interpretation parameters. Let unequal access, delayed correction, selective disclosure, and locally divergent legal knowledge emerge from those mechanisms rather than scripting them as historical outcomes.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928e0-2968-83e9-af6e-4aeed669a2f9)
