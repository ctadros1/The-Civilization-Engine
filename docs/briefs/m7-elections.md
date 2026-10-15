# M7 design brief: elections, parties and offices

**Scope.** Plan §7's M7 ("Politics deepens") lists "Elections with franchise rules; parties formed from factions; campaigns; coalitions", and its demo is "a republic's assembly captured by a merchant guild passes protectionist tariffs". Plan §5.2 gives an office "title, powers, number of seats, term, **selection method**, and removal method", lists "election (franchise and voting rule), appointment by another office, sortition, acclamation" among selection methods, names "term limits" among checks, and asks the label classifier to name a "republic". This brief covers, for polities of 100–3,000 people:

- offices with terms, chosen by election, by lot or by appointment, and removed;
- how a polity moves from today's gathering by acclamation to an elected council or assembly, by its own amendment, and back;
- elections step by step: franchise, candidacy, campaigns, polling and the count;
- how a voter chooses, and whether they go at all;
- parties formed from factions, and coalitions between them;
- how the label classifier reads the result.

I assume sibling M7 briefs own corruption and patronage (office pay, gifts for votes), lobbying and capture by guilds, courts and judicial review, and credit. This brief needs from them office pay, the gift channel and a forum for a disputed count.

**Scale.** Lived villages are 18 to about 170 people so far (the M6 war brief, from the README and plan §9). At about 50 people the gatherings are "nearly unanimous" with "92-100 % came", and in the ten-year smoke "no village amended its custom" (plan §9, slice AF). At 3,000 people, if about half are adults (a design prior), a quarter quorum is about 375 people at one hearth (my arithmetic). Everything below must work at both ends.

**What exists to build on** (checked in the code):

- **One body per polity.** `Polity` (`civ-agents/src/polity.rs`) has one `body: Body` of three values: `members: Membership` (`Adults`, `Elders`, `Landholders`, `Watch`), `quorum_share` and `pass: PassRule` (`MoreForThanAgainst`, `TwoThirds`). `Body::gathering` is the founding custom: the adults at the hearth, a quarter quorum, more for than against. `Body::decide` returns `Passed`, `Failed`, `Tied` or `NoQuorum` and never repairs one. `versions: Vec<CustomVersion>` keeps each custom with the law that made it or who seized it. The body has no seats, no term, no visibility field and no agenda holder: any member may propose.
- **Offices are laws that name a holder.** `Law.holder` is set for `KeepStore` and `KeepWatch` (and a band's leader). The sponsor names the eligible adult they regard most, themselves at full regard (`population/polity.rs`, the review). Tenure is "until they die or leave": `lapse_keepers` marks the law `Lapsed`, the chronicle tells it, and "Nothing fills it by itself". There is no term and no removal except a later law: `Repeal`, or a petition demanding another holder (`Petition.nominee`, `ends`). Office pay is not built (plan §9, slice AC).
- **The gathering.** `Gathering` keeps each household's stake and who came. Only those who heard it was called and whom the body admits may come (`gathering_facts`), scored `attend_base + w_attend × stake`; it sits `gathering_minutes` 120. Stances are open: `stance()` weighs the household's forecast and regard for the sponsor; `stance_between` weighs regard for a nominee against what holds one to the incumbent. Every `StanceRecord` is saved.
- **Amending the custom.** `PolicyKind::AmendBody` with `core:policy/amend_custom` offers adults, elders or landholders, a quarter or a half, and either rule. Only someone `Overruled` may propose it, one change at a time, and only when it would have served their own household. A household judges a body by re-deciding the decisions it saw from the stances recorded there (`decisions_seen`, `amend_gain`). The present custom decides. Plan §9 lists "seats filled by a procedure" and "unanimity or consensus rules" as not built.
- **Who deliberates.** Notables (a compute tier that grants nothing, ADR-0014 §4) and those something reached since the last review. `RuleDeliberator` scores support × `w_gain` × gain − `propose_cost` and picks by softmax.
- **Factions** (`faction.rs`, `population/faction.rs`). A faction forms over a grievance against the gathering or an office (`against: Blamed`), with an organizer, a store, dues and a history. Each person belongs to one at most. It can petition, refuse a levy, or revolt with a program body chosen by `program()` from the amendment templates. A coup is open to a watch of several. Factions hold no positions on issues and endorse nobody.
- **Opinion, ideology, norms.** Positions per policy template, 0–1, with an anchor and salience, moved by what companions say (`opinion.rs`, Friedkin–Johnsen). Three ideologies with programs of policy templates (`ideology.rs`). One norm, "the gathering binds" (`norm.rs`). News travels as immutable claims by contact (`word.rs`).
- **No legitimacy record.** The per-authority assessments the M4 governance brief proposed (§1.5) were not built. Paying a levy weighs norm, stance and regard for the sponsor (`comply_points`).
- **Labels** (`civ-sim/src/labels.rs`). `evidence` and `classify` give "Council community" when the body admits at least 90 % of adults (`BROAD`), "Oligarchy" below 50 % (`RESTRICTED`), with headship modifiers ("big-man leadership", "led by its proposer"), offices and levies, over a 730-day window. There is no "republic", and nothing reads selection methods or terms. Agents cannot see this crate.
- **Several settlements.** A polity may hold several (`Polity.held`); members are residents of any; "The body gathers at the seat's hearth" (ADR-0023 §9).

## Where the reports agree

Nine points recur across 09-08, 09-05, 09-02, 09-15, 09-04 and 09-14. I treat them as invariants.

1. **Factions before parties, choices before ballots, rules apart from both** (09-08, opening). "A faction becomes an electoral organization when coordinating nominations and supporters offers more influence than remaining an informal network" (09-08 §1.2). There is "no defensible universal empirical range" for months to form a faction (09-08 §2.2).
2. **Selection is a composition of steps, and methods coexist.** Eligibility, nomination, selection and recognition are separate (09-02 §1.3). Athens combined lot and election; Venice alternated them (09-02 §4.1, §4.4, §8.2): "do not force a polity to possess one government-wide selection method".
3. **Electoral rules are exact algorithms, and they are political.** Store franchise, unit, nomination, magnitude, ballot, allocation and secrecy separately (09-08 §1.7). "Never assume electoral rules are selected impartially" (§1.7). Who occupies a gate is an amendable rule (09-05 §3.9).
4. **Participation is a funnel and a separate choice.** Legally eligible → able → participates → valid ballot (09-08 §1.6; 09-02 §3.1). Residents, citizens, voters, eligible candidates and officeholders are different populations (09-14 §1.5). Eligibility, practical access and turnout are kept apart (09-15 §6.3).
5. **Voters act on perceptions.** The utility uses the voter's "perception of the candidate's position—not the true platform" (09-08 §1.3). What happened, who controlled it and who is blamed are three quantities (§1.4). "A secret vote should remain hidden except through explicitly modeled breaches" (§1.4).
6. **Franchise change is a bargain, not a milestone, and it reverses** (09-08 §1.8). The lesson of franchise history is "institutional diversity and reversibility, not a universal chronology" (09-08 §3).
7. **Tenure is conditions, not days.** "Store tenure conditions and exit processes, not just `term_length_days`" (09-02 §8.6). Historical terms run from a day to life and must not become priors (09-02 §5.1).
8. **A republic is not a democracy, and a label grants nothing.** A republic has "public governing offices organized without a crown" filled by "election, rotation, or lot". It does not mean democracy: "A republic can exclude most inhabitants from power" (09-15 §2). Labels are summaries (ADR-0013 §6).
9. **Failures stay failures:** an empty candidate pool, a disputed membership, a failed quorum or a tied vote "without inventing an automatic winner" (09-02 §10).

**What models and games teach to avoid:**

- Party counts as targets: the seat-product formula is "a conditional aggregate benchmark" (09-08 §2.1).
- Field-experiment effects as coefficients: "Do not treat an observed percentage-point effect as a logistic coefficient" (09-08 §2.1).
- "Candidate utility multiplied by probability of winning", which makes bandwagons (09-08 §1.5).
- "A universal progress-and-chance mechanism" for enactment (09-05 §5.6, on Victoria 3).
- Scaled institutions: "A 500-seat council is not automatically meaningful in a polity of 700 people" (09-02 §9.4).

## Where they differ, or differ from the plan

- **Order of work.** The plan lists parties, campaigns and coalitions as contents. 09-08 §6 says "build the exact institutional counting and office-selection layer first". 09-08 §3 finds small-scale politics in "issue coalitions" without "presuming formal ballots". At village scale parties may never form. I build counting and offices first (Suggested slices).
- **Selection as an enum.** Plan §5.2 lists methods; 09-02 §1.3 says "Do not implement `selection = election | inheritance | appointment`". ADR-0013 §2 already composes steps. I follow the ADR.
- **Names.** Plan §5.2 lists "direct democracy"; 09-15 §2 offers "council community" for binding collective procedures, and 09-15 §6.4 asks for "functional classifications" of direct assemblies and lot. The classifier already uses "Council community". I keep it.
- **Terms.** Plan §4.4 puts "scheduled elections" in the seasonal or yearly cadence. 09-14 §2.2 proposes rotating terms of 3–12 months (urban, design); 09-05 §2.1 records Uttaramerur committees serving 360 days and an Icelandic lawspeaker chosen for 3 years. I make terms authored values a body can amend, tested at 6, 12 and 36 months (design prior).
- **Council size.** 09-05 §2.3 suggests 5–15 (tested 3–30); 09-14 §2.2 proposes working executives of 6–24 and councils of 20–200, both for towns. Five seats among 50 adults is one in ten; fifteen among 1,500 is one in a hundred (my arithmetic). Sizes are content levels, never scaled to population (09-02 §9.4).
- **Coalitions.** 09-08 §1.9 is about cabinets, investiture and confidence, and warns: "do not import cabinet-confidence logic automatically" where no body chooses the executive. No TCE body chooses one. Coalitions here are electoral alliances and blocs.
- **Turnout.** 09-08 §2.1's anchors are modern (8.1 points from social pressure, 24 from compulsory voting, 76 % falling to 66 %). The nearest to a village is Olken's: direct elections drew participation "nearly 80%, about 20 times meeting attendance" (09-04 §2.2). None is a target.
- **Legitimacy.** 09-04 §1.2: "Procedure can make an unfavorable decision acceptable", and the Indonesian elections raised willingness to contribute by 17 points "without improving approval" of leaders (§2.2). TCE has no authority assessments for this to land in (open question 10).
- **The demo.** "Captured by a merchant guild passes protectionist tariffs" names an outcome; guilds and tariffs belong to the lobbying brief. §2 treats it as a plot risk.
- **Forecasting a council.** The kernel judges a new body by re-deciding seen decisions from recorded stances (plan §9, slice AF). For an elected council that needs a guess at who would sit, which no report supplies (§1.8; open question 3).

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| A seated body | `Body` gains seats, electorate, candidate eligibility, method, term and visibility; its holders by seat | An amendment the present custom passes |
| An office's tenure | On the office law: method, term, re-eligibility, removal routes | The law that creates it; the body's later laws |
| An election | Called, why, candidates with dates and who nominated them, campaign claims, polling window, ballots (open) or totals (secret), the count, certification, installation, failure if any | The term's calendar, a vacancy or an amendment; each person's choices |
| A slate | A faction's endorsements per election | Its organizer's scored move |
| An electoral alliance | Two organizers, who stands down for whom, for which election; kept or broken | Two organizers meeting and weighing packages |
| A franchise change | A custom version, as now | An amendment |

**Derived, not saved:** "party", "coalition", turnout ratios, effective numbers of factions, the label. **Size:** an election in a village of 1,500 adults with 10 candidates on 4 questions is 60,000 comparisons (my arithmetic), against 09-08 §5.2's 2.4 million "workload counts" at 50,000 agents.

### 1.1 Seats: a body filled by a procedure

**Reports.**

- A body's decision rule needs eligibility, quorum, denominator, aggregation, threshold, ties and visibility (09-05 §1.4).
- A broad gathering can sit beside a smaller law-making council; the Althing's law council, "rather than every person present", made law (09-05 §3.4). "Large councils can coexist with small operational bodies" (09-14 §4).
- Proposal rights and agenda rights are different powers; "nondecision" is an outcome (09-05 §1.2).
- Uttaramerur's committees had restricted eligibility, set terms and accountability inside a monarchy (09-05 §3.6).

**My proposal.**

- **`Membership` gains `Seats`.** A seated body holds n seats, each filled from an electorate (an existing membership: adults, elders, landholders) by a method (election, lot, or named by the electorate's gathering), from candidates eligible by another membership, for a term. Content offers n of 5, 9 and 15 (09-05 §2.3). It is ADR-0013 §2's "seats filled by a procedure".
- **It meets as the gathering does,** at the seat's hearth, with its own quorum and pass rule. Stances stay open unless the body says otherwise.
- **Agenda: seat-holders propose.** Everyone else petitions, as factions already do (ADR-0017 §3). A petition the council never takes up is a nondecision, recorded.
- **An empty seat stays empty** until a by-election or the term's end. A council short of its quorum fails, recorded (09-02 §10).
- **Seats by settlement.** For a polity of several settlements, seats may be apportioned by settlement, each its own district (09-08 §1.7: district magnitude is "seats elected per district"). This is one answer to the far village that ADR-0023 §9 leaves walking to the seat.

### 1.2 Offices with terms, methods and removal

**Reports.**

- Haudenosaunee chiefs held for "Life, subject to removal": "conditional tenure, not an extremely long fixed term" (09-02 §5.1).
- Athens chose councillors by lot and military officers by election (09-02 §4.1). A presiding officer could serve "one night and day" (09-14 §2.1).
- Offices can be "unpaid, rotating, fee-supported, part-time, and salaried" (09-14 §1.7).
- Who nominates is a separate question: "Assembly, guild, ward, incumbent council, ruler, or mixed procedure" (09-14 §1.5).

**My proposal.**

- **An office law gains a tenure:** until the holder dies or leaves (today), or a term of days, with or without re-eligibility (plan §5.2's term limit). Removal stays a later law of the deciding body; recall by the electorate waits (§3).
- **Four ways to fill it,** each a value on the law: named by the deciding body (today's route); elected by an electorate (§1.3); drawn by lot, a keyed draw (ADR-0011 §3) among eligible people who put their names in; or appointed by another office (the council names the storekeeper).
- **Consent is a step.** A nominee accepts or refuses by the same score as standing (§1.3); a refusal is recorded (09-02 §1.3: eligible is not selected).
- **A term's end is a vacancy,** and an elected office's law calls its own election. Nothing else fills it.
- **Lot needs volunteers.** Putting one's name in is a scored choice. An empty pool is a failure, recorded.

### 1.3 An election, step by step

**Reports.**

- The cycle runs "nominations and campaigning → eligibility and turnout → exact counting → office and coalition formation" (09-08 §5.2), and "election participation still occurs on actual scheduled days" (§2.2).
- Turnout is σ(duty + habit + mobilization + stakes + nonparticipation penalty − time and income costs − danger − administrative burden), with costs from the household's own day (09-08 §1.6).
- Exact rules: plurality wins a seat by the highest total; implement "tie-breaking, exhausted ballots, rounding, thresholds" explicitly (09-08 §1.7). Keep "actual ballots, counted totals, announced results, and acceptance" apart (§1.7).
- Arrival is not publication; a proclamation reaches those present (09-16 §1.3, §1.7).

**My proposal.**

- **Stages, each dated:** called; word goes round (an appended claim kind, as ADR-0016 §3 has for a gathering called); candidacy open for some days; campaign; polling; count; certification at the hearth; installation. Each failure has its own record.
- **Standing is a choice.** A candidate weighs what their sitting would turn their way (their household's stake in seen decisions, as `amend_gain` reckons it), the regard it may bring, against the evenings in council and the walk. Others may nominate someone they regard, as a keeper is nominated now.
- **Polling is an activity,** like attending: walk to the hearth in a window on the day, cast, go home. Its points are duty (a norm, if content authors one), the stake (how much better the voter's first choice is than the next), regard for whoever asked them, less the hours. A short vote may draw more than a two-hour sitting; Olken's 20 times is a check, not a target (09-04 §2.2).
- **Open or secret** is a body value. Open is raised hands, seen by those present and saved as stances. Secret is pebbles: only totals exist, and nobody, kernel included, keeps who chose whom.
- **Counting is exact and tested.** v0 has plurality for one seat; for n seats, the block vote (each casts up to n) and the single vote (each casts one, the top n win). A tie is decided by the procedure's own rule: it fails, or lot decides.
- **Announced results travel by contact.** Losers' acceptance is their choice: a petition, a refusal or a revolt as now.

### 1.4 How a voter chooses

**Reports.**

- U = −β_S·D + β_R·R + β_G·G + β_L·L + β_Q·Q + β_P·P: perceived policy distance, attributed performance, group protection, loyalty, competence, selective benefits; "proposed model components, not historically measured universal weights" (09-08 §1.3).
- "Do not translate occupation directly into a predetermined party vote" (09-08 §1.3). For rankings, "generate a consistent ranking from one set of candidate evaluations" (§1.3).
- Strategic switching only "when the favorite appears unable to win", from local information (09-08 §1.5).
- Recent performance half-life 6–24 months; attachment 1–5 election cycles (09-08 §2.2, design proposals).

**My proposal, mapping each term to state that exists:**

- **D** from `opinion.rs` positions and the candidate's positions as the voter perceives them: stances seen at gatherings, and what was told in the campaign. Unknown is no term, not neutral.
- **R** for an incumbent: the voter's household's stake in the decisions they saw the incumbent stand on (`decisions_seen`), signed by whether the incumbent stood with them.
- **G** same household, kin or faction. **L** regard (`ties.rs`). **Q** standing in counsel (ADR-0014 §3). **P** waits for the patronage brief.
- **One evaluation, one softmax,** at a fixed utility scale (09-08 §1.3). The block vote takes the top n of the same ranking.
- **Strategic voting is off in v0,** then swept at 0, 0.25, 0.5, 0.75 and 1 of voters (09-08 §2.2), using viability only as heard from contacts.

### 1.5 Campaigns

**Reports.**

- Campaigning should make visible activities "only where the institution actually requires them" (09-08 §5.2); simplify "the detail of campaign messages" (§5.5).
- Ten people repeating one crier are not ten witnesses; hearing and joining are different contagions (09-16 §1.6).
- Propaganda works through "paid speakers, official announcements" and their credibility (09-16 §1.8).

**My proposal.**

- **A campaign is company and claims.** A candidate and those who back them spend hours at the hearth telling three kinds of claim: that they stand; where they stand on up to three policy questions (their own positions); and who backs them. They travel by ADR-0016 §3's rules, one origin counted once.
- **What is told moves perception,** and, through the ordinary opinion channel, sometimes the listener's own position. A promise is a claim the voter can later check against stances seen in council (R).
- **The cost is hours,** taken from work. No campaign bonus exists. Gifts to voters belong to the patronage brief; paid speakers wait (§3).

### 1.6 Parties from factions

**Reports.**

- A faction "does not need a public name, written manifesto, formal membership register, or electoral platform"; "Do not replace factions when parties appear" (09-08 §1.2).
- Parties enter, merge, split and die on resources and incentives, "not an arbitrary expiration timer" (09-08 §1.2; Laver and Schilperoord, §5.4).
- "One-party monopoly is not the same as party-organizational control" (09-15 §2).

**My proposal.**

- **No new entity in v0.** A faction's organizer gains a move: endorse a slate for an election, members or not. It is scored by the decisions they forecast the slate would turn toward their members' households, less the risk of splitting support.
- **An endorsement is a claim.** Members weigh it by their identification with the faction (`w_member` exists) and regard for the organizer: the G and L terms.
- **"Party" is the observer's word** for a faction that has endorsed candidates in two elections. It is derived, never saved, and nothing reads it.
- **Founding around a program.** ADR-0017 §2 allows a program from an ideology; the code founds only over a grievance. Founding over an ideology's program before an election is the route by which a party can begin without a grievance.
- **One membership each** limits factions inside parties (09-08 §1.2). It stays for v0 (open question 6).

### 1.7 Coalitions

**Reports.**

- "Electoral alliances, parliamentary support arrangements, cabinets, and non-electoral ruling coalitions are different relationships" (09-08 §1.9). A party "accepts only when the package beats its perceived outside option" (§1.9).
- Factions "negotiate withdrawals, common candidates, seat-sharing arrangements, and mergers" (09-08 §1.5).
- Coalition search over 12 parties is at most 4,096 subsets (09-08 §5.3).

**My proposal.**

- **An electoral alliance is a meeting.** Two organizers who believe, from their contacts, that their split support loses to a third may meet and weigh a few packages (who stands down, a common slate), as seeking terms between polities already weighs up to 16 (`population/terms.rs`). They agree or part. Keeping it is each one's later choice.
- **In council, blocs are derived:** who stood together, counted and shown, never saved.
- **Office-sharing waits** until a council appoints offices (§1.2). Then portfolio shares near seat shares are a check, "not a binding formula" (09-08 §2.1).

### 1.8 From the gathering to an elected council, by amendment

**Reports.**

- Institutional change is proposed "when expected benefits exceed organizational costs and risks" (09-02 §3.7). Meeting congestion is answered by adding sittings or delegating (09-05 §1.6).
- Included actors estimate effects on "future control, taxation, public goods, and security"; excluded actors organize, bargain or withhold (09-08 §1.8).
- Different enactments may follow different routes, chosen by subject (09-05 §5.1).
- A republic can become electorally authoritarian "without abolishing either its name or its legislature" (09-15 §3).

**My proposal.**

- **The same route as today.** `amend_custom` gains seated bodies among its levels (n, electorate, method, term). Changing to one is one change. It is open to the overruled, weighed only when it would serve the sponsor's household, and decided by the present custom.
- **The forecast is a design prior.** The sponsor expects the n eligible people they know who stand highest in counsel to sit. They re-decide each seen decision with those people's recorded stances (absent ones abstain), and count decisions lost to no quorum as decided. For lot, the forecast is the eligible pool's seen stances in proportion.
- **Why it might pass.** A council serves households when a few decide at thin gatherings (the label's `FEW_COME`) or quorums fail. At 50 people with 92–100 % present it serves nobody, which is what the smoke found. That is the mechanism. There is no population gate (09-15 §4B: "None").
- **It reverses** by the same route (09-08 §1.8; 09-14 §4: forms "can reverse"). A franchise change is one change of electorate, forecast by re-counting seen ballots.
- **A council may amend itself away.** Under today's rule the present custom decides amendments, so a council can drop elections or narrow the electorate. That is 09-15 §3's erosion, and it should be possible. Whether seats and franchise need the electorate's ratification is open question 4.
- **Seizures may install councils.** `program()` picks from the amendment templates, so a revolt can bring a seated body.

### 1.9 How the label reads it

**Reports.**

- Follow selection chains to the offices that govern; "an elected assembly should not establish accountability when an unelected actor can cancel its choices" (09-15 §6.4).
- Report participation "among recognized citizens" and "among governed adults" (09-15 §6.3).
- "Preserve election evidence across the relevant term rather than expiring it mechanically" (09-15 §4B).
- Tests: privileged households selecting officials is an "Oligarchic republic"; one party winning competitive elections is not one-party; "a partyless assembly permits broad participation" is not closed autocracy (09-15 §6.7, §7).

**My proposal.**

- **Evidence gains:** whether the body is plenary or seated; how seats and offices are filled; fixed terms or until gone; the electorate's share of governed adults; turnout at the last election by both denominators; contested seats; incumbents unseated; factions with slates and their seats; and whether the seated body decided what passed.
- **Rules.** A seated body filled by election or lot, with fixed terms, governing, is a "Republic". With an electorate under `RESTRICTED` it is an "Oligarchic republic". Between, it is a republic with the share as a modifier. Plenary stays "Council community". Modifiers say how: "council of 9 by election", "electorate 92 % of adults", "seats uncontested", "two factions contest it", "by lot".
- **Never "democracy" from structure.** Electoral accountability needs choice, organization, credible procedure, access to governing office and a real chance of replacing incumbents (09-15 §6.4).
- **The window** keeps the last election, however old, while its term runs.
- **Tests** are 09-15 §7's synthetic table, in code, as the M4 label has.

## 2. Plot guards

- **No threshold, timer or era** creates a council, an election, a party or a wider franchise (ADR-0013's forbidden list; 09-15 §4B; 09-08 §1.8, §3). The founding custom is already the broadest electorate, so no ladder runs toward it.
- **No issue weighs toward a council** (ADR-0013 §5). Amendments are scored by forecast.
- **No party-count or turnout rule.** Seat-product and field-experiment numbers are checks (09-08 §2.1).
- **Nobody reads the truth.** No voter, candidate or organizer reads a true tally before the count, a true position, or a secret vote (09-08 §1.4; ADR-0016 §1). Nothing reads a label (ADR-0013 §6).
- **No repair.** No candidates leaves a seat empty; a tie follows the procedure's rule; a council without quorum fails (09-02 §10).
- **No franchise by sex** is authored, as ADR-0023 has it for service. Age, residence, household, land, office and membership remain (09-08 §1.8's predicates otherwise).
- **No god tool votes.** No command casts a vote, nominates, endorses or sets a franchise (ADR-0016 §5). A whisper may carry a true claim, such as where a candidate stood.
- **The demo.** If no republic forms, or none is captured, after a week, nudge a prior (the hours a seat costs, the council sizes offered), never an event, and log a `NUDGE:` (plan §1).

## 3. What to defer, and why

| Defer | To | Why |
|---|---|---|
| List PR, d'Hondt, STV, mixed systems | When factions contest seats repeatedly | 09-08 §1.7's exact rules need parties and larger magnitudes; v0 bodies have 5–15 seats |
| Cabinets, investiture, confidence | When a body chooses an executive | 09-08 §1.9: not imported where no body chooses one |
| Recall by the electorate | After terms work | Removal by the body exists; recall is a second route (09-02 §1.4) |
| Compulsory voting, ballot design, literacy | With writing | 09-08 §1.6 ties them to enforcement and administration |
| Office pay, gifts for votes, monitoring of open votes | The corruption and patronage brief | 09-08 §1.4: clientelism through network inference |
| Disputed counts | The courts brief | 09-08 §1.7: ballots, totals, announcement and acceptance apart |
| Several memberships; factions inside parties | After slates | Saves keep one membership per person |
| Paid speakers, propaganda, censorship | Later | 09-16 §1.8; ADR-0016's revisit trigger |

## 4. Risks and validation

**Nothing happens.** At today's sizes no village amended its custom (plan §9, slice AF). Seated bodies may never pass. Count the funnel before tuning: people overruled, seated bodies weighed, proposed, passed; elections called, candidates, refusals, turnout, contested seats, incumbents unseated. A council that never appears below some size is a result, as long as no rule put the size there.

**Entrenchment and deadlock.** A council may vote away its elections; a council with empty seats may never meet a quorum again, leaving only revolt. Both are phenomena to show (09-02 §10: "Do not prohibit historically dysfunctional combinations"). Test that each is recorded and that the chronicle tells amendment from replacement (ADR-0017 §1).

**Leaking truth.** Test that a secret ballot leaves no per-voter record anywhere, and that a voter who heard nothing of a candidate gives no D term.

**Compute.** Elections are rare and bounded (§1). Campaign company is the cost: count it per hour of company, not per session (the M4 factions brief's rule for gossip).

**Tests and plausibility checks** (never targets):

- **The deterministic count.** 09-08 §4's 42/33/25 electorate, adapted: the same ballots under the block vote and the single vote give different seats; the effective number of vote winners stays about 2.88.
- **Edge cases:** no candidates, one candidate, a tie under each rule, a vacancy mid-term, a council below quorum, a holder who dies on polling day.
- **Matched twins.** `civ-host twin` lives the same world under the gathering and under a council (09-08 §5.5: "matched counterfactuals with the same population").
- **Reported, not graded:** turnout by eligible and by governed adults (09-15 §6.3; 09-08 §4); N = 1/Σp² by votes and by seats (09-08 §2.1); the Gallagher index (09-08 §4); incumbents unseated; how long councils last.
- **Labels:** 09-15 §7's table, and "Population doubles while institutions ... remain unchanged" keeps the label.

**Observer.** An election page: why it was called, candidates and who named them, claims told, turnout with both denominators, the count, who sits. A voter's "why" in 09-08 §1.3's terms. The label's reasons name the chain from electorate to council to what it passed.

**Expensive to reverse.** One ADR: seated bodies and office tenure, amending ADR-0013 §2 (`Body` gains seats, electorate, method, term and visibility; office laws gain tenure and method); elections as dated records; faction slates and alliances, amending ADR-0017 §2; appended claim kinds (ADR-0016 §3). Saves bump; a save with today's body loads as the founding custom. The label's new rules need no ADR (plan §9).

## Suggested slices

1. **Offices and counting:** office laws gain terms and selection by the body's vote, by lot or by appointment, with the exact counting module and its unit tests, and no new body.
2. **Seated bodies:** `amend_custom` offers councils filled by election or lot; elections run as dated records with candidacy, polling and count; the label learns "republic".
3. **Campaigns and parties:** candidacy claims, voters' mixed utility from positions, record, regard and standing, and faction slates.
4. **Coalitions and franchise politics:** alliances between organizers, franchise amendments, seats by settlement, the twin runs and the demo.

## Open questions for the designer

1. **One body or two.** Is a council a `Membership::Seats` of the one body, as proposed, or a second body beside a gathering that only elects? Two bodies change ADR-0013 §1's one-body polity.
2. **Open or secret by default** for elections? Open reuses stances and invites monitoring (09-08 §1.4); secret needs a record that holds only totals.
3. **The council forecast.** Is "the n eligible people the sponsor knows who stand highest in counsel" an acceptable prior? No report gives one.
4. **Who amends a seated body?** The council alone, as today's rule gives, or the electorate for seats and franchise (09-05 §5.1)?
5. **Hours saved.** Should a household weigh the evenings a council saves it, or only how decisions would have gone?
6. **Parties as entities.** Is a faction with a slate enough, or does 09-08 §5.1's `Party` need its own record and several memberships each?
7. **Terms.** Which defaults: 6, 12 or 36 months? May content offer life with removal (09-02 §5.1)?
8. **Seats by settlement.** In polities of several settlements, may an amendment apportion seats, and does that settle ADR-0023 §9's far village?
9. **Strategic voting.** Leave it out of v0, as proposed?
10. **Legitimacy.** Elections are where 09-04 §1.2's procedural acceptance should show. Do we build the per-authority assessments the M4 governance brief proposed (§1.5), or keep compliance on norm, stance and regard?
11. **Office pay.** Without pay, will anyone stand? Does candidacy wait for the patronage brief's channel?
12. **The demo's nudge.** Which priors may be nudged after the time box?
