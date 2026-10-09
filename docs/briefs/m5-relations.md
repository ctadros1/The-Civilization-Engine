# M5 design brief: relations between polities

**Scope.** Plan §7's M5 asks for "diplomacy states and treaties with ratification", "tributary relations" and a "fortification kit". Its demo includes "a treaty fails ratification", and it is to prove "diplomacy through each side's own institutions" while deferring "war and sovereignty changes". This brief covers, for two or three village polities:

- what a relation is, and what makes neighbours wary or friendly without war;
- how one village comes to know another, and how news crosses between them;
- treaties: their clauses, how one is proposed, negotiated and ratified by each side's own institutions, why one fails, and how it is performed and ended;
- tribute short of conquest;
- the fortification kit as works a village decides to build;
- emblems and marks of belonging.

I assume sibling M5 briefs own roads and merchants, migration and splinter founding, diffusion, bridges and currencies. This brief needs one thing from them: people who travel between settlements, since everything below travels with them. Plan §4.3 sets 3–8k people in 2–3 settlements for M5–M7.

**What exists to build on** (checked in the code):

- **Polities.** Each settlement gets a polity under the founding custom at its first midnight (`population/polity.rs`): a gathering of adults deciding by acclamation. Each law's whole history is kept (sponsor, stances with reasons, decision, compliance). Polities have offices, a common store, amendments of the custom and repeal (ADR-0013, ADR-0017).
- **Several polities can already exist.** A family the observer sends more than 600 m from any lived-in village founds its own settlement (`found.rs`, `SPAWN_JOIN_M`). None knows of another.
- **Word and markets are per settlement.** Word travels by contact within a settlement as five claim kinds: gathering, grievance, petition, refusal, revolt (ADR-0016 §3). A grievance blames a party of the person's own polity: its body, an office, a household or a person (`Blamed` in `word.rs`).
- **Ties, obligations and works.** Ties can hold anyone (ADR-0014). Obligations have a debtor, a beneficiary, a due date and a default rule (ADR-0015 §5). Building parts wear and are mended (ADR-0009), and the ground keeps the earth moved (ADR-0010).
- **ADRs waiting on M5.** ADR-0013, ADR-0016 and ADR-0017 name it as a revisit trigger: jurisdiction over distance, news between settlements, and outside help.

## Where the reports agree

Six points recur across 13-01, 13-02, 13-05 and 11-11. I treat them as invariants.

1. **No relation score, and no relation state that causes anything.** A relation is contact, agreements, obligations and observed performance, held by people and institutions (13-01, executive recommendation, §6.1; 13-02, executive recommendation). "A treaty that only changes an opinion score" reproduces nothing that ceasefire research finds (13-05 §4). A loyalty value may be a display summary, never a cause (13-02 §5.3).
2. **Each side authorizes by its own rules.** Negotiated, approved internally, consent communicated, effective and implemented are separate states (13-01 §1.4), and ratification must not be simplified away (13-01 §6.3). Any law keeps the same separation (09-05 §1.7).
3. **People perform promises with real goods.** Grain is collected and carried (13-01 §6.2). What is assessed, collected, owed and received are separate accounts (13-02 §1.2 C, F). An entitlement feeds nobody from an empty store (02-04 §4.2).
4. **Decision-makers act on what reached them**, by journeys and contact, not distance (13-01 §1.1; 13-05 §5.3; 09-16 §1.1–1.2, §5.2). No "distance loyalty penalty" where journeys are simulated (13-02 §2.3).
5. **No universal rates.** No betrayal roll from alliance reliability (13-01 §3.2), no annual rebellion rate (13-02 §4.1), no war propensity by era (13-05, executive conclusion, §2.1).
6. **Peace and fortification are choices among alternatives.** Peace is "an institutional achievement, not simply a low aggression score" (13-05, executive conclusion). Fortifying is one response beside relocating, negotiating and accepting losses (11-11 §1.1). A work never attacked may still have worked (11-11 §4; 13-05 §4).

**What grand-strategy games teach to avoid:**

- Institutions as "bundles of national modifiers" rather than arrangements among people (02-04, opening).
- Abstract points: EU4's diplomatic power is an abstract constraint and its households have little agency (02-04 §1.3).
- Rolls in place of procedure (Victoria 3's first law enactment, 02-04 §2); "failed a 42% roll" is a poor explanation of an institution's failure (§4.7).
- Membership that erases a grievance: store promises, deadlines, counterparties and fulfilment apart (02-04 §2).
- Truce immunity, narrow concession menus and a single confidence score (13-05 §5.6, on Victoria 3); loyalty progression (13-02 §5.4, on Stellaris).
- A valid action the AI never invokes, which players read as a preference (02-04 §2, CK3 1.16).

**Where they differ, or differ from the plan:**

- **The plan's relation states.** Plan §5.5 lists "relation states: unknown, contact, peace, trade treaty, alliance ..., tributary/vassal, protectorate, war, truce" as authored primitives, which point 1 rejects. 13-05 §5.4 keeps sustained-war states, but only for war, with raids, incidents and treaties tracked apart.
- **Packages per decision.** 13-01 §3.3 gives 8–32 per bargaining decision; 09-05 §2.3 gives 3–8 per round. Both are computational bounds.
- **How long a grudge lasts.** 13-05 §2.4 tests a 2–10 year half-life. ADR-0016 fades a grievance's activation within months but keeps its claim until settled, so the two layers fit both.
- **What an enclosure means.** Whether a given enclosure was military is disputed; boundary, ritual, livestock and defence overlap (11-11 §1.1).
- **Legitimacy or coercion.** Kang reads East Asian tributary relations through hierarchy and legitimacy; Perdue doubts that one coherent system explains them. The model should allow both recognition-based relations and coercion (13-02 §3.2).

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Knowing a neighbour | New claim kinds in ADR-0016's table: a settlement at a place, an offer, a decision on an agreement, an obligation met or missed, a taking by an outsider | Journeys between settlements; company at either hearth |
| Views of a polity | Per person, per foreign polity heard of: evidence counts by domain, last update, main reason | Obligations due, takings believed, gifts received, as each person saw or heard them |
| Grievances across the boundary | ADR-0016 grievances, `Blamed` extended to another polity's body, office, household or person | A breach of a term the person knows; a taking by an outsider |
| Agreement | Shared record: parties, clauses (content ids and levels), negotiators, the law on each side, dated states, a term | The negotiators' meeting; each gathering; word carried back |
| Each side's ratification | An ordinary law of an appended `PolicyKind`, with ADR-0013's whole history | The polity's custom |
| Obligations between polities | ADR-0015 obligations with a polity as debtor or beneficiary; four accounts per transfer | Clauses in force; collectors and carriers |
| Works | An enclosure: a line of sectors, components as parts with condition, earth moved, progress | A polity's law, or a household's building choice |

**Derived, not saved:** a relation's label, believed reliability, predicted support for a package, and tribute's burden ratios. **Size:** 2,000 people × 2 foreign polities × about 40 B a view ≈ 160 KB (my arithmetic; the 40 B is a guess).

### 1.1 What a relation is

**Reports.** Beyond point 1: 13-01 §6.1 keeps a relationship as known representatives, routes, recognition claims, issue-specific interests and observed history. 13-05 §5.1 adds active claims, treaties, attributed incidents, trade exposure and "differentiated reputation". Trust is "contextual and observer-specific" (13-01 §1.5).

**My proposal: no relation is saved.** What is saved is what each side's people hold (hearing records, ties, views, grievances), the agreements their gatherings made, and the obligations and incidents between them. The plan's states become a **label** from a pure classifier that nothing in the world reads, as ADR-0013 §6 does for regimes:

- *unknown*: nobody in either village has heard of the other;
- *known*: some have, and there is no agreement;
- *under agreement*, naming its clauses;
- *tributary*, where the only recurring transfer runs one way;
- *friendly* or *wary*, by the share of each side's adults whose view leans either way, with reasons, for example "31 of 44 adults in Ashford hold that Brookby keeps its word; 6 hold a grievance against a Brookby household".

The two sides' labels can differ and are shown apart. Turning the classifier off must leave every digest unchanged (the M4 governance brief §3 asks the same of regime labels).

### 1.2 Knowing a neighbour, and news between villages

**Reports.**

- Awareness, access and recognition differ (13-01 §1.1). Traders, migrants, kin and travellers carry reports of foreign communities; a mission needs a destination, a route and resources; receiving it is "a separate decision".
- A message waiting for a carrier "is stationary until departure" (09-16 §1.2). Someone back from market is "a potential secondary transmitter—not a guaranteed household-wide update" (09-16 §1.3). Merchants are bridges because they move between settings, "not because they possess a permanent arbitrary rumor radius" (09-16 §5.2).
- Network position beats proximity, and coverage has a long tail (09-16 §4). For early farming villages no latencies or awareness shares are recoverable (09-16 §3). As an illustration, a merchant at 30 km/day on a weekly cycle brings news 90 km in 6.5 days on average (09-16 §2.3).

**My proposal.**

- **What *known* means.** Someone who walks within sight of another village's homes, meets one of its people, or is told of it, holds the claim "a settlement at this place". That is all *known* means.
- **Travellers carry their hearing records.** At the other hearth they are company like anyone, under ADR-0016's rules, and at home they do the same. There is no other path, so a village nobody visits learns nothing of its neighbour's laws.
- **Beliefs come only from claims.** What one village believes of another's numbers, stores or laws comes only from claims (13-05 §5.3: "The kernel knows where soldiers and supplies are. Rulers should not."). Retellings of one origin are one source across villages too (09-16 §1.6).

### 1.3 What makes neighbours wary or friendly

**Reports.**

- **Trust has parts.** Keep separate expectations of willingness, capacity, truthfulness, resolve and institutional continuity (13-01 §1.5). Update a beta count α/(α+β) only "after a relevant opportunity". Ten quiet years "are not ten successful alliance tests", and grain missed in a famine is not a diversion (13-01 §1.5).
- **Resources give stakes and alternatives:** seasonal access, fees, shared maintenance, migration or seizure. Scarcity "should not automatically spawn an army". Trade matters through "lost counterparty-specific surplus", not a peace bonus (13-05 §1.1).
- **Grievances need attribution.** Retaliation may target an offender, a household, a clan or a polity. A grievance keeps its offender, attribution confidence, responsible group, remedy and unresolved balance, and compensation can close it without erasing memory (13-05 §1.7).
- **Peace systems are reachable.** They must be attainable, "not a special scripted exemption" (13-05 §4). Forager relations begin with household ties and hospitality (13-01 §4, on hxaro). Cultural identity is not political authority (13-05 §5.1).

**My proposal.**

- **Ties do most of the work.** Trades, company at the other's hearth, shared work and kin make ties across villages, as ADR-0014 already allows. A stranger starts as one.
- **A view of a polity** is a per-person record shaped like a tie (ADR-0014 §1), with evidence in three domains: *keeps its word* (obligations due, met or not), *harms us* (takings and breaches attributed to its people) and *helps us* (gifts, relief).
  - The counts fade toward α = β = 1 with a 5-year relevance half-life (13-01 §3.3 tests 2, 5 and 20; a tuning value).
  - Only acts a person saw or heard of write it.
  - A missed payment counts against *keeps its word* only for someone who believes it was kept back rather than that the harvest failed (13-01 §1.5).
- **Grievances cross the boundary.** `Blamed` gains another polity's body, office, household or person, and ADR-0016 §2's rule stands: harm, an expectation held (a treaty term the person knows, or a norm), and blame. A taking by an outsider blames that person or household. Their gathering is blamed only if a term bound it to hear such takings and it did not (13-05 §1.7, §1.9).
- **Claims over wild ground.** Only fields are held today (ADR-0007 §1), so outsiders cutting wood or digging clay near a village break no expectation, and an access clause has nothing to grant. I propose a policy by which a gathering claims a place (a wood, a deposit it knows) that outsiders use only by leave. Whether any village claims anything is its people's choice. This is M5's form of 13-05 §1.1's "disputes over identifiable assets and rights".
- **Splinter villages** start with kin ties, a shared way of building and the grievances that drove them out, now held against a foreign body. History, not a rule, makes them neighbours with a past.
- **Not in M5.** A palisade moves nobody's view. The security dilemma ("a fort may simultaneously improve defense and worsen relations", 13-05 §1.5) needs force to fear, and force comes in M6.

### 1.4 What a treaty is: clauses from a library

**Reports.**

- **Clauses, not labels.** Treaties are built from obligations (13-01 §3.1), as plan §5.5 asks ("clauses from a library"). Each obligation names a responsible actor, beneficiary, trigger, action, quantity, place, deadline and remedies (13-01 §6.1).
- **Separate rights.** Access, trade and transit are separate rights, never implied by peace (13-01 §3.1).
- **Durations and endings.** Duration is stored at agreement and clause level, and expiration, replacement and breach are different outcomes (13-01 §3.1, §5).
- **The binding subject is explicit,** and a change of ruler must not silently rewrite it (13-01 §6.1). A personal promise is not an office's obligation (02-04 §4.5).
- **Early farming villages.** For them, "seasonal access, compensation, kin-mediated negotiation, and joint ceremonies" are only hypotheses (13-01 §4).

**My proposal: five clause templates in content**, as a content kind `clause` with levels a negotiator picks:

1. **Gift.** Goods once, to a polity or a named person; a gift to a person differs from a payment to a store (13-01 §1.3).
2. **Leave to use.** The other's people may use a place this polity claims (§1.3), for named goods and seasons, for a fee or none (13-05 §1.1).
3. **Leave to trade.** The other's people may sell at this hearth (13-01 §3.1).
4. **Takings heard.** Each polity hears takings by its people from the other's households as its own, and its findings bind (13-05 §1.9: a signatory must bind its participants). Only a polity with a law against taking (ADR-0015 §4) can perform this, and the negotiator's forecast must see it, because authority to promise is not capacity to perform (13-01 §1.4).
5. **Recurring transfer.** Goods, an amount or share, a schedule, a collection point and a receiving store (13-01 §3.1; 13-02 §5.1; see §1.7).

**Term.** A term comes from a content menu: one season, one year, five years, or until withdrawn after a season's notice. This is 13-01 §3.3's menu cut short; the values are tuning values.

**The binding subject is the polity,** not the negotiators or a version of the custom. A new custom inherits the agreement, and a founding body may repeal it like any law (ADR-0017 §5).

### 1.5 Proposing, negotiating and ratifying, and why one fails

**Reports.**

- **Two-level games.** A bargain good for both parties can fail inside one, because each polity's ratifiable set comes from its own rules, and "a council's approval, a ruler's oath ... and a legislature's consent need not be interchangeable" (13-01 §1.4). "Some attractive packages fail authorization; compensation or altered terms can change the domestic coalition" (13-01 §5).
- **Time comes from the world:** travel, waiting, discussion, referral and authorization (13-01 §1.2). Calendars create waiting regardless of disagreement (09-05 §4).
- **Envoys.** An envoy may be authorized "to discuss everything but conclude nothing", and skill is not "magical persuasion" (13-01 §1.2). A receiver may accept, reject, counteroffer, seek instructions or delay (13-01 §6.2). Search a bounded set of typed packages (13-01 §6.3).
- **Proposals need sponsors.** A proposal needs someone with the motive, information and access to sponsor it (09-05 §1.1), and it can die without a vote (09-05 §1.2).
- **Expected acceptance.** An offer is worth P(accept)·U plus the rest times the value of continuing (13-01 §2.2). The existing `RuleDeliberator` already scores a move this way: expected support, counted over those the person knows, times forecast gain, less the cost of proposing.

**My proposal: a treaty is two laws, one in each polity, over one shared agreement.**

1. **Initiative.** Someone who holds an issue a clause answers, and who knows someone in the other village, may weigh *seeking terms* as a `Deliberator` move (ADR-0013 §5). Such issues include outsiders' takings, a wood the neighbour claims, a lean year with a neighbour believed to have grain, or a market there. No issue carries a weight toward a clause.
2. **Meeting.** They walk there. Their contact, or a notable they have heard of, chooses whether to talk (13-01 §1.1). The two weigh up to 8 packages of clauses and levels (a tuning value, inside both reports' bounds). Each scores a package by their own household's forecast and the support they predict at their own gathering. Neither can bind their village: they may discuss everything but conclude nothing (13-01 §1.2). They agree on a package both expect to pass, or part with none, and either outcome is recorded.
3. **Ratification by each custom.** Each proposes the package at home as a law of a new policy kind, *agreement*. Their gathering decides it by its own custom, with every stance and reason kept (ADR-0013 §3). If an amendment has given an office the power ("make treaties" is among plan §5.2's powers), that office decides instead. The vocabulary allows this, and nothing makes it happen.
4. **Consent communicated.** A decision is a claim and crosses only with a traveller. The agreement is in force once both polities have passed it and each has heard of the other's decision. Until then it is *ratified by one side*.
5. **Failure is an outcome.** Any of these ends the agreement as *failed ratification*, recorded on both sides once heard, with its reason, and never repaired (ADR-0013 §2):
   - a vote turns it down;
   - the gathering falls short of quorum;
   - nobody calls a gathering;
   - one side never answers within the term.

**Why a treaty fails, by mechanism and not by roll:**

- **Forecasts miss.** Negotiators predict from the people they know, but the gathering is whoever heard of it and came (ADR-0016 §3). It is thin at harvest, when people are at work (09-05 §1.4).
- **A clause has losers.** Leave to cut in our wood costs those who cut there, and a transfer costs every levy payer.
- **Distrust objects.** Holders of a grievance against the neighbour's people, or of a view that it breaks its word, object.
- **The usual weights apply.** Regard for the sponsor, values and ideology weigh as for any law (ADR-0016 §4).
- **A refusal is an event.** Its backers may hold it against their own gathering, and the counterpart learns of it only when someone carries word.

### 1.6 Performance, breach and ending

**Reports.**

- Maintenance is scheduled acts: collection, carriage, receipt, renewals (13-01 §1.6).
- For an apparent breach, ask whether the trigger occurred, whether the obligation was violated and who was responsible. Then allow explanation, compensation, suspension, renegotiation or escalation (13-01 §1.6).
- Count opportunities, not years (13-01 §5).
- Fixed obligations make bad harvests dangerous, so allow arrears, remission, concealment and refusal. A shortfall is first a dispute (13-02 §1.2 D).
- Validity, capacity, compliance and legitimacy are separate (02-04 §4.4).

**My proposal.**

- **Clauses create obligations.** Clauses in force create ADR-0015 obligations with a polity as debtor or beneficiary, on an appended ledger channel. No goods are created.
- **People perform or miss.** People meet or miss each obligation by their own choices: the keeper releases the store, households pay the levy, a carrier walks.
- **A miss keeps its cause.** The truth layer records the cause (empty store, no carrier, held back by a decision), and people learn it only as a claim.
- **Answers are existing moves:** let a miss pass, propose to remit it, suspend the clause or repeal the agreement (`Repeal`, ADR-0017 §5). Nothing escalates by itself.
- **Endings are separate.** Expiry, withdrawal after notice, repeal and failed ratification are recorded as separate endings.

### 1.7 Tribute short of conquest

**Reports.**

- **Bundles, not a status.** Unequal relations are bundles of obligations and decision rights, not one vassal status, tribute percentage or loyalty score. The subordinate accepts while compliance, or the cost of resistance, beats its alternatives, and the superior keeps the relation while returns exceed enforcement costs (13-02, executive recommendation; §1.2 A). Annexation is "one possible outcome, not its natural endpoint" (ibid.).
- **Tributary is not tax-paying.** Rank, recognition, commerce and coercion are separate (13-02 §1.1). A ceremonial relation has a gift basket and "no default GDP percentage" (13-02 §2.3).
- **The ruler's bargain is not the households'** (13-02 §1.2 B).
- **Four records.** What households owe, what is collected, what is owed upward and what arrives are kept apart (13-02 §1.2 C). Logistics sets the usable return (§1.2 F), so nominal extraction exceeds receipts (§4.4).
- **Two burdens.** Burden is T/Y and T/(Y − subsistence): 10 paid from 100 with 80 needed is 10 % of output but 50 % of surplus (13-02 §2.1).
- **Historical quotas do not scale down** (13-02 §5.2). The Aztec provincial 1.3–14 % shows unequal burdens, not a band (§2.2), and the 1–20 % test grid has "low empirical portability" (§2.3). CK3 sets tax and levy apart: extraction is a relationship (02-04 §1.2).

**My proposal.**

- **In M5, tribute is a recurring one-way transfer that both gatherings ratified for what it buys:** leave to use a claimed wood or deposit, leave to trade, the takings clause, or relief from the other's store in a lean year. Tribute under threat needs force between polities and waits for M6. *Tributary* is only a label (§1.1).
- **Amounts.** The amount is a level the negotiators choose, fixed or a share of the harvest, with no default. The observer shows both burden ratios for each paying village.
- **Four accounts, carried.** The four accounts are what households paid in, what the store set aside, what was owed and what arrived. Carriers walk the goods, and what spoils on the way is lost.
- **Paid from the store.** A transfer is paid from the common store, filled by its levy. A polity without one cannot perform a transfer, so a negotiator who forecasts that does not offer it (open question 3).
- **Stances.** Each household weighs its own levy against what the clause brings it, as for any law. The custom sets whose stances count.

### 1.8 The fortification kit

**Reports.**

- **Works buy time, not a bonus.** Fortifications control movement and buy time, and preventing entry, sustaining resistance and retaining control are separate outcomes (11-11, central recommendation). So are condition, readiness and political control (11-11 §5.1).
- **No farming or state is needed.** Hunter-gatherers at Amnya built banks, ditches and palisades around 6000 BCE (11-11 §1.1). Early farming enclosures had military, social and ceremonial uses (11-11 §3).
- **When to build.** Build when perceived avoided losses, access control and political benefits exceed construction, upkeep and opportunity costs, as particular decision-makers judge them, among alternatives (11-11 §1.1).
- **Who is protected and who pays.** A ruler may protect a store and leave outer households exposed (11-11 §1.1). Who pays matters (§2.4). Proposals go through existing political machinery: the line, contributions, gate access, demolitions (§5.2).
- **Components.** Palisades are felled, carried, set and replaced; they burn and rot. Ditches are dug and slump. Ditch spoil builds the bank and is not dug twice (11-11 §1.2).
- **Building.** Incomplete works give partial benefit, progress is the least of labour, materials, tools and room to work, and compulsory labour is not free (11-11 §1.3). Upkeep is work orders, not global decay (§1.5).
- **Figures.** 100 m of palisade of 0.20 m posts 4 m long takes about 500 posts, and a 100 m ditch 4 m wide at the top, 1 m at the bottom and 2 m deep is 500 m³ (11-11 §2.3). Cahokia's 3.2 km stockade took 15,000–20,000 logs (11-11 §2.1), 4.7–6.3 a metre (my arithmetic).
- **Rates.** Hand digging moves 0.5–2 m³ per 8-hour worker-day, with low confidence (11-11 §2.2): 4–16 h/m³ (my arithmetic). A perimeter is at least 2√(πA) (11-11 §1.6). A post kept day and night needs 3–4 people (11-11 §2.2).

**My proposal.**

- **Works are a new building kind made of M3b's parts.** An enclosure is a line of sectors 10–25 m long (11-11 §2.2). A sector may have a ditch (an ADR-0010 earthwork, its spoil heaped inside as a bank), a palisade (ADR-0009 parts, made well or badly, wearing at the foot by wetness as hut posts do) and gates. Digging costs the kernel's 8 h/m³, inside 11-11's range. Posts that rot through leave a gap.
- **Two routes, chosen by people.**
  - **A household** may fence its yard and store as it would build a storehouse, when the takings it believes it suffered over the past year outweigh the hours (11-11 §1.1's rule, on its own beliefs).
  - **The polity.** Anyone may propose that the polity enclose. The law names the line (the store, the hearth, or all homes within a reach), how work is raised (labour-days as ADR-0015 obligations, or volunteers) and whether an office like the watch keeps a gate. Households outside the line pay without protection, and their stances weigh that.
- **What it does without war.** People cross only at a gate or by climbing, which takes time and can be seen, so outside takers are seen more often through what is physical, not a bonus. Its costs show daily: walks round to the gate, mending, and the hours it took.
- **Illustration** (my arithmetic, for an assumed 1 ha core). The line is at least 2√(π × 10,000 m²) ≈ 354 m: about 1,770 posts at 5 a metre. A ditch of the 11-11 module's 5 m² section is about 1,770 m³, about 14,200 hours at 8 h/m³, or 470 hours for each of 30 working adults. A palisade round the store alone costs far less. Which of these a village builds, if any, is the test of the rule.

### 1.9 Emblems and marks of belonging

**Reports.**

- **Ornament came first.** Personal ornament long predates farming, so decoration must not wait for government (06-06 §1.1).
- **Flags need an organization.** Flags and standards arise "when an organization needs visible recognition: a war band must rally, vessels must identify affiliation", and "a village can possess shared motifs without having an official flag" (06-06 §1.2). Seals answer stored property, contracts and disputes over authorization (06-06 §1.2).
- **Meaning is association** by community, context and period (06-06 §1.5). Meaning, design, rights and object are kept apart (06-06, executive recommendation).
- **Resemblance follows contact** more than distance (06-06 §4), and identical signs are plausible between groups with little contact (06-06 §5.3).

**My proposal: no emblems or flags in M5.** No M5 organization needs recognition at a distance, and nothing written needs a seal. Each village's way of building (slice R) already marks it. The observer can name it, and it moves toward a neighbour's only through contact, which the M5 demo needs. Emblems and banners, with grants and revocations through the law pipeline (06-06 §1.6), fit M6's war bands.

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| War, raids, conquest, annexation, secession, merger | M6 | Plan §7 |
| Tribute under threat, vassalage, protectorates | M6 | They rest on the costs of resistance and enforcement (13-02, executive recommendation; §1.2 A), which need force between polities |
| Alliance and assistance clauses | M6 | Conditional contracts with triggers (13-01 §3.1; 13-05 §1.8), with nothing yet to trigger them |
| Hostages and marriages as clauses | M6; the migration brief | Hostageship is a guarantee resting on physical control of people; a marriage's effect should follow from the kin and property it creates (13-01 §1.3) |
| Bargaining under threat of war | M6 | In M5 the outside option is no agreement; 13-01 §2.1's peace benchmark belongs with war |
| Alarm at a neighbour's works; sieges, garrisons | M6 | 13-05 §1.5 needs force to fear; 11-11 §1.4 |
| Emblems, flags, seals, dress codes | M6 and later | §1.9 |
| Agreements of three parties | With merger and federation (M6) | v0 has two parties; the record allows more (13-01 §6.1) |
| Distortion and propaganda between villages | Later | ADR-0016 §3; 09-16 §5.5 |

## 3. Risks and validation

**Plot.** Watch for issue→clause weights ("a lean year makes a tribute treaty"), timers on contact, and tuning until a treaty fails. The demo's failed ratification must come from who came and what they held. After a week without one, nudge a prior (say, a negotiator's caution about thin gatherings), never an event, and log a `NUDGE:` (plan §1, rule 3).

**Nothing to negotiate.** With land plentiful, villages may never compete (README: the M3a regimes did not part while land was plentiful), and lean years still empty them. Neighbours may live decades with no issue a clause answers. Measure the funnel: issues held; *seeking terms* available and chosen; meetings; packages agreed; laws proposed and decided. 02-04 §2 tells implementation, incentive, representation and legibility failures apart: a move that is never available is a bug, not a preference.

**Leaking truth.** Test that no choice reads the other polity's stores, laws or numbers except through claims; that someone who heard nothing holds no view ("unknown events have no instantaneous global effect", 13-01 §5); and that one teller's retellings count as one source.

**Scale.** A year of 1,000 people takes about 8 minutes at Max, and route searches dominate the slowest days (plan §9); walks between villages are long routes. Relations themselves are sparse and event-driven (13-01 §6.3; 13-05 §5.5). Add agreements and obligations to Gate B (ADR-0011 §5) with wide tolerances.

**Tests from the reports.**

- **Domestic constraint** (13-01 §5): a package both negotiators expected to pass fails at one gathering, and compensating the households that lose changes the outcome.
- **All gain:** when every household on both sides gains, it passes. This is an M5 analogue of 13-01 §2.1's benchmark.
- **Opportunities, not years** (13-01 §1.5): five years in force with nothing due leaves views unmoved.
- **Famine against diversion** (13-01 §1.5; 13-02 §1.2 D): a transfer missed for an empty store moves *keeps its word* less, for those who know why, than one held back.
- **Continuity** (13-01 §5; 02-04 §4.5): a keeper dies, or the custom is amended or replaced, and the agreement stands until a law ends it.
- **Nominal against delivered** (13-02 §4.4): the four accounts reconcile under the ledger's conservation check.
- **Works:** an unfinished palisade gives partial cover (11-11 §1.3); a rotted gap admits a taker; the ditch's spoil lies in the bank (ADR-0010).
- **Saves and labels:** save → load → save is exact, and digests are unchanged with the classifier off.

**Observer.**

- **A relations panel per pair of villages** shows each side's label and reasons apart, every agreement with its two laws' histories side by side (including when each side heard), and each obligation's four accounts.
- **What each village believes of the other** is shown as counts with reasons, never mixed with truth (ADR-0015 §7).
- **Works on the map** show their condition and progress.
- **The chronicle speaks plainly**, for example: "Brookby's gathering turned down the agreement Ada of Ashford and Wren of Brookby made, 14 against and 9 for; most who opposed cut wood in the alder wood it would open."

**Choices that are expensive to reverse.** Plan §1 allows about three ADRs a milestone, and roads, trade and settlements will want theirs. I suggest one ADR for relations: agreements as shared records with one law on each side; obligations between polities on an appended ledger channel; per-person views of foreign polities, the extended `Blamed` and claims that cross settlements; and labels that are derived and never read. Enclosures could amend ADR-0009 and ADR-0010 rather than take an ADR, since only the line of sectors is new. Clause and claim kinds are append-only.

## 4. Open questions for the designer

1. **Relation states.** Plan §5.5 makes them authored primitives, and the reports reject a relation state as a cause. Are they labels, as proposed?
2. **Who may negotiate** under the founding custom: anyone, or only someone the gathering names first? Naming first means two decisions on one side, a mandate and then terms.
3. **Transfers without a store.** May an agreement levy households directly, or is a transfer paid only from a common store?
4. **Offers to the other gathering.** Must an offer find a sponsor among the receiving village's members, so it can die without a vote (09-05 §1.2)? Or may a visitor put it to their gathering?
5. **Claims over wild ground.** Is a law claiming a wood or deposit in M5's scope? Without one, access clauses grant nothing and resource disputes cannot arise.
6. **Takings between villages.** Do they arrive with contact in M5, and may a watch act on a taker from another village?
7. **Enclosures.** Polity works only, or household yards too? Is a gate keeper a new office or the watch?
8. **Alarm.** Should anyone in M5 read anything from a neighbour's palisade, or is that wholly M6?
9. **Emblems.** Defer 06-06 to M6, or show each village's way of building as its mark now?
10. **Splinter grievances.** Do grievances against an old gathering carry over as grievances against a foreign body?
11. **Agreements after a revolt.** Does a new body inherit agreements, as proposed, or does replacing the custom void them?
