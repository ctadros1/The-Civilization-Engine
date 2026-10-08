# M4 design brief: factions and unrest

**Scope.** This brief covers:

- reputation, trust and grievance;
- how factions form;
- petitions, protests and riots, and separately coups and revolts;
- the constitution the winners rebuild;
- opinion, ideology and norms;
- how news, rumours and laws spread;
- four god tools: whisper, bless/curse, ideology and agitator.

The scale is one settlement of 1–2k people. At plan §4.2's 1 %, that is about 10–20 notables (my arithmetic).

**Where the reports agree.** They describe separate, linked processes, not one "unrest" meter: experienced conditions → interpreted grievance → communication and organization → choices to take part → responses → revised beliefs (04-10, core recommendation; 09-11, opening). Four rules recur. I treat them as invariants:

1. People decide only from their own sparse beliefs. They never read world truth or settlement totals (09-16 §1.1, 04-10 §5.2, 06-05 §5.5).
2. Private belief, public expression, membership and current activity are separate state (04-10 §5.1, 09-11 §1.3, 06-04 §1.8).
3. Ten retellings of one report are not ten observations (04-06 §1.5, 09-16 §1.6).
4. A coup is not the top of a riot meter. No historical turnout share or event rate transfers to the simulation (04-10 §1.9, §2.3; 09-11 §1.4, §2.4).

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Relationships | Per person, up to N directed records: other person, evidence counts in 4 domains, loyalty, obligation, fear, last update | Interactions the ledger already records; co-presence; reports |
| Grievance | Per person, sparse: issue, blamed actor, harm, violated expectation, remedy, unresolved part, activation, last reminder, origin event | Events: a meal missed, rent or tax taken, a punishment, a petition refused |
| Information | Shared immutable claims. Per person: claim, first and last exposure, confidence, origin, disclosure | Meals, the hearth, shared work, the market, proclamations |
| Opinion | Per person: 6–12 issue records (position or none, confidence, salience), anchors, 4–8 values | Conversations; monthly review |
| Norms | Per person and norm: endorsement, empirical expectation, normative expectation | Observed acts, reports, laws |
| Factions | Program, organizers, treasury on the ledger, meeting time and place, history. Memberships kept on persons | A decision by a notable or a citizen; each person's monthly review on a keyed day |
| Episodes | Kind, issue, organizers, place, attendance history, violent acts by category, arrests, concessions, outcome | An organizer's call; attendance chosen in the activity scorer |
| Founding | Provisional authority, proposals, positions, ratification | An incumbent falls |
| God-tool influence | One record per recipient, proposition and episode | Observer commands |

**Derived, not saved:** thresholds (keyed draws on seed, purpose, person and action kind), perceived support and faction sizes.

**Size:** 2,000 people × 64 relationship records × 64 B ≈ 8 MB, using 04-06 §5.5's record size (my arithmetic).

### 1.1 Relationships, reputation and trust

**Model.**
- Beta-style evidence counts in four domains (honesty, reliability, help, danger), each decaying toward a prior (04-06 §5.2).
- Loyalty, obligation and fear are kept apart from reputation (04-06 §1.1).
- An interaction is judged for responsibility and for what it reveals, separately from the loss (04-06 §1.2). A crop lost to weather is not dishonesty.

**Inputs that already exist:** food asked for and given, rent, workshop wages, trades, kin.

**Parameters.**
- Evidence half-life: 30–365 d (04-06 §2.2). 06-05 §2.2 gives 30–730 d.
- Hearsay: 0.1–0.5 of a direct observation (04-06 §2.2).
- Records per person: the reports disagree. 04-06 §2.2 gives 32–128, 06-05 §2.2 gives 16–64, 04-10 §2.2 gives 8–32, and 09-12 §2.2 gives 2–12 political contacts. I suggest 32 (tuning value), with kin, employer, landlord and organizer pinned.

### 1.2 Grievance

**Model.** A grievance is harm + responsible party + violated expectation + demanded remedy + unresolved portion (04-06 §1.7). It is kept by issue (subsistence, extraction, treatment, collective claim) and by blamed actor (04-10 §1.1). It has two layers:
- a durable claim, with no expiry until settled or abandoned (04-06 §2.2, §4.4);
- an activation, which decays and is raised only by reminders, never re-added every tick (04-06 §5.3).

**When one arises.** Only when harm meets an expectation the person holds: a norm (§1.5) or a law's terms.
- **Harm** is measured in hours of household work, as `value.rs` already measures goods. It is divided by the household's monthly work to give the 0–1 scale of 04-10 §2.2. The divisor is a tuning value.
- **Blame** needs a belief that the actor had both duty and power (06-04 §1.1). A lean year with relief working blames nobody. The same year with an office sitting on village stores blames the office (04-10 §1.1).

**Inputs.**
- Existing: hunger, days of food, the lean season, grain price for a household that buys more than it sells (04-10 §1.6), rent, wages, village-field allocation.
- Added in M4: tax, levy, punishment, curfew, prohibitions.

**Relative deprivation.** The reports do not support it as a driver by itself (04-10 §1.1). The comparisons they do support:
- the household's own trajectory (09-12 §1.1);
- peers' wages (04-10 §5.5);
- expected against attainable status (09-12 §1.4).

Density must never generate discontent directly (09-12 §1.1).

**Parameters.**
- Activation half-life: 1–7 d for minor events, 7–90 d for severe ones (04-06 §2.2).
- Durable memory: the reports disagree. 04-10 §2.2 gives 30–365 d, 09-11 §2.5 gives 0.5–5 y, and 04-06 §2.2 says no expiry while a claim is unresolved. The two-layer split reconciles most of this.

### 1.3 News, rumours and laws

**Model.**
- A claim is structured: subject, event, place, time, quantity, cause, source, recommended action. No record means not heard (09-16 §5.1).
- Hearing, believing, passing on and acting are separate. So is believing that others believe (09-16 §1.1).
- Hearing takes one contact; joining needs several (09-16 §1.6, 04-10 §1.4).

**Transmission** happens only through existing contact. A member back from market may pass news on, but is not certain to (09-16 §1.3). At 1–2k people the single hearth must not become a broadcast, so partners are sampled per hour of company.

**Laws.** A proclamation reaches only those present. Three versions of a law are kept: the rule itself, the official's copy, and each person's understanding (09-16 §1.7). This is pipeline stage 4.

**Parameters.**
- 2–8 conversations per person per day (09-16 §2.2).
- Sharing probability: 0.05–0.25 for routine news, 0.4–0.9 for urgent news (09-16 §2.2).
- Retention half-life: 3–14 d for passing news, 30–180 d for important news (09-16 §2.2).
- At most 1–3 recipients per report (06-05 §2.2).
- Distortion is zero until the undistorted system works (09-16 §5.5).

### 1.4 Opinion and ideology

**Model.** x′ = (1−a−b)x + a·m + b·z (06-04 §1.4), where:
- m is the message's position;
- z is an anchor recomputed from the household's material position;
- b is the anchor weight;
- a = η × salience × topic trust × a smooth disagreement gate × a redundancy discount.

This is Friedkin–Johnsen persistence. DeGroot and voter models converge to consensus (06-04 §2). Plan §4.1's ideology axes fit as the slow values, not as the whole political state (06-04 §1.1).

**An ideology** is a content record: problem explanation → moral commitments → institutional proposals (packages of M4 primitives) → constituency → legitimacy narrative (06-04 §6.1). It spreads as claims. Children take their caregivers' positions, with variation (06-04 §1.7).

**Parameters** (06-04 §3.2, all design priors):
- η 0.03 (0.005–0.15); ε 0.25 (0.10–0.60).
- 0.5 exposures a week.
- Anchor half-life 2 y; values 20 y.
- Susceptibility doubled between ages 12 and 30.
- Like-minded contacts at odds of 2.
- Negative influence 0.

About 23 accepted exposures halve a gap.

### 1.5 Norms

Norm templates are content. Each person holds an endorsement, an empirical expectation and a normative expectation, acting through two logistic thresholds (06-05 §5.1–5.2).

**Uses here.**
- Norms supply the expectations whose breach makes a grievance: sharing in need, rent in a failed year, a council's duty to hear.
- They supply the moral term in choosing violence (04-10 §1.8).
- A law updates legal knowledge, expected enforcement, legitimacy and perceived approval separately (06-05 §1.9).

**Sanctioning** is a costly choice whose channel depends on power (06-05 §1.4, §1.6). No metanorms in v1 (06-05 §5.3).

**Parameters:** thresholds 0.2–0.9; learning rate 0.02–0.30 (06-05 §2.2).

### 1.6 Factions

**Model.** A faction is an organization (04-10 §5.1). Its organizers articulate, recruit, connect and negotiate (04-10 §1.5). Its treasury is held on the goods ledger (ADR-0006).

**Formation** is a decision by a notable (plan §4.2) or by any citizen (09-12 §1.5: popular organizations need no elite). It requires:
- a grievance shared among trusted contacts;
- an organizer;
- expected efficacy;
- tolerable risk.

It never happens "because enough agents have high anger scores" (04-06 §1.8). Splits follow 06-03 §1.7's test: a dispute, a following, alternative leaders, and the means to act.

**Joining** is reviewed monthly, on each person's own keyed day (09-12 §2.2 gives 1–3 months; 09-11 §2.5 gives 7–30 d), and within a day in a crisis.

**Thin evidence:** no report gives a minimum founding size. Borrowing 06-03 §2.2's religious split hazard (0.01–0.05 a year) would be my extension.

### 1.7 Petition, assembly, refusal and riot

**Episodes.** An organizer calls each one at a time and place. Its kind is content: petition, assembly, or refusal of a tax or levy (pipeline stage 6).

**Escalation is not a ladder.** A refused petition is a new violation and lowers the expected efficacy of mild tactics. 04-10 §3 names failed petitioning as a precursor.

**Attendance** is a new activity in the existing scorer. Infeasible attendance is dropped first (04-10 §1.2). The rest is scored as

V = β_g·g + β_I·I + β_E·E + β_M·M + f(S) + O − C − p̂·D (04-10 §5.3)

with these terms:
- g: grievance;
- I: identification;
- E: efficacy;
- M: moral pull;
- O: organizational help;
- C: opportunity cost;
- p̂·D: expected sanction;
- S: the weighted share of the person's own contacts believed to attend. Unknown contacts stay uncertain, and an absolute count is kept too (04-10 §5.2).

Danger and commitments shift each person's threshold (04-10 §1.3). For some people f(S) falls as S rises, which is the free-riding found in Hong Kong (04-10 §1.4). The choice uses the existing softmax, with units normalized before the temperature is tuned (06-05 §5.2).

**Within an episode,** damage and violence are separate per-person choices every 1–10 simulated minutes (04-10 §2.2).

**Recorded separately** (04-10 §1.8, §4.2):
- peaceful action;
- property damage;
- violence against people;
- state violence against peaceful participants;
- unique participants, participant-days, peak and duration.

**Repression** updates deterrence, incapacitation, outrage and information separately for victims, witnesses and hearers (04-10 §1.7).

**Parameters.** Test thresholds drawn from Beta(1,3), Beta(2,2) and Beta(3,1) (04-10 §2.2); 06-04 §3.2 centres them near 0.5. Borrow Epstein's local information, not his k = 2.3 or T = 0.1 (04-10 §5.5).

### 1.8 Coup, revolt and the rebuilt constitution

**Coup.** Insiders who hold force: an office commanding force, or the watch.
- Willingness comes from their own grievances, arrears owed to them (09-11 §1.2) and blocked access to office (09-12 §1.3). It is separate from success (09-11 §1.4).
- Each armed person chooses incumbent, challenger, neutral or break away, so a coup is a coordination contest (09-11 §1.4).
- Success is control held for at least 7 days, borrowing Powell–Thyne's coding (09-11 §2.2). There is no per-tick roll (09-11 §2.4).

**Revolt.** A faction whose program replaces offices.
- It succeeds when revenue, compliance, coercive support and administration actually change hands (09-11 §1.5).
- That happens through officeholders' own decisions to repress, concede, flee or hold.
- Losers keep wealth, contacts and claims (09-11 §1.7).

**Founding** (09-11 §1.6) runs in six steps:
1. Provisional authority over the surviving offices.
2. Representation through existing bodies.
3. Proposals built from the inherited constitution.
4. Bargaining, each faction scoring a proposal as U_f(C) = α·fit + β·material + γ·security − δ·cost − ε·exclusion against conflict, leaving or submission.
5. Ratification.
6. Appointments.

The result is not average opinion (06-04 §6.3). 09-11 §2.5's 3–36 months is a test range, not a timer.

### 1.9 God tools

**Boundary** (15-05 §4.2). Tools submit only physical changes and perceptible events. Nothing equivalent to choosing an action, passing a law or setting allegiance. Persuading one officeholder never bypasses a council vote.

**Whisper.** Puts a deliberator move, or a true claim, into one notable's information. The notable scores it like any other move, with a bounded salience term.
- Salience half-life: 1–3 decision cycles.
- One record per recipient, proposition and episode, so repeated clicks do not reroll (15-05 §4.5, §5).
- The observer shows whether it was perceived, considered, acted on and passed on (15-05 §6).

**Bless/curse.** The reports say nothing about luck; 15-05 §4.7 only warns against routine blessings. My proposal: for a stated period, the target's keyed draws for listed material purposes (discovery, part quality, health hazards) shift by a stated quantile. Decision draws are never touched, and the effect is imperceptible.

**Ideology.** A content ideology enters as awareness, like `IntroduceTechnique { aware_only }` (ADR-0008 §6). It goes to one person, or to 25–100 witnesses, 15-05 §5's range for public signs.

**Agitator.** A newcomer, by M1's send-a-family path, holding a doctrine strongly and with an organizer's disposition. They start with a stranger's trust (04-06 §1.6), so influence must be earned. The reports do not cover agitators.

**Logging.** Each use goes in the chronicle, and its id is tagged on the receipts it touches (15-05 §6).

## 2. What to defer, and why

- **Elections, parties, courts, lobbying:** the plan puts them in M7. Disputes go to petitions and council hearings instead of 04-06 §5.4's case lifecycle.
- **Strikes** (04-10 §1.9): workshops are small, and refusing a levy covers withholding.
- **Distortion, propaganda and censorship institutions** (09-16 §1.8, §5.5): speech and assembly prohibitions act only as sanction risk.
- **Recombined ideologies** (06-04 §4.2) and **negative influence.**
- **Inherited narratives:** children take positions, not memories (04-06 §6); property claims are the exception.
- **Religious institutions and omens** (06-03 §1.8; 15-05 §4.1): keep the organization entity generic (06-03 §5.4).
- **Elite aspirant queues** (09-12 §1.2): show Q_j as a diagnostic only.
- **Splinter founding** (M5) and **counter-revolution with outside help** (09-11 §1.7).

## 3. Pitfalls, and how to show it works

| Failure | Report | Test or check |
|---|---|---|
| Cascades always or never fire | 04-10 §1.3, §4.2 | Thresholds 0–4 cascade, and setting the second to 2 stops it. At fixed mean grievance, three threshold distributions over two network densities give both outcomes (04-10 §6.2). |
| Coups read off riots | 04-10 §1.9 | Coup code reads no attendance. Paying the watch more lowers attempts but raises success (09-11 §1.4, §4). |
| Perpetual rage | 04-06 §5.3, §6 | Activation decays without reminders while the claim persists. A world where every grievance turns violent fails. |
| Echoes become certainty | 04-06 §1.5 | Ten retellings of one origin move belief less than ten independent observations. |
| Trivial convergence or frozen clusters | 06-04 §2 | Without anchors, opinions converge (sanity check). With anchors, disagreement persists. Extremity, bimodality and issue alignment are measured separately (06-04 §1.6). |
| Telepathic totals | 06-05 §5.5 | Someone with no attending contacts believes nothing about turnout. |
| Same-day reviews, order dependence | 09-12 §2.2; 04-10 §5.6 | Reviews spread across days; shuffling the processing order leaves the distributions unchanged. |
| Repression always works or always backfires | 04-10 §1.7 | Witnesses and hearers of the official account diverge. |
| Stress index double-counted | 09-12 §2.3 | The dashboard shows its components; the kernel never reads them. |
| Blank-slate winners | 09-11 §4 | The chronicle tells amendment from replacement. |
| Hidden commands or spam | 15-05 §7 | No command writes an outcome; 100 identical whispers equal one. |

**Three risks particular to this codebase.**

- **Leaving beats speaking up.** A household out of food leaves with a 10 % daily chance (plan §9, slice E), and villages of 55–95 have left together (§9 NUDGEs). §9 asks M4's councils to revisit this.
  - As a hazard outside the scorer, leaving pre-empts petitions, and hunger already cuts the capacity to organize (09-11 §1.2).
  - Make leaving a scored choice beside petitioning, and count departures in episodes.
- **Accelerated drift.** Leisure blocks keep "the company found as it began" (ADR-0011 §4), so count gossip per hour of company, not per session. Add attendance and membership to Gate B; their tolerances will be wide because the events are rare.
- **Demo pressure.** "Two different regimes in five seeds after 40 years" (plan §7, §4.7) invites a timer. Reform and succession count as routes too. After a week without result, nudge a prior, never an event (plan §1, rule 3).

**Plausibility checks, not targets.**
- Leaders change more often than regimes (09-11 §4).
- Peaceful episodes outnumber violent ones (09-12 §4: about 5:1, in one US series).
- Grievance, efficacy and identity relate imperfectly to participation (04-10 §2.1: r ≈ 0.34–0.38, which are correlations, not coefficients).
- Clarke's 53 % of revolutions challenged and 18 % overturned (09-11 §2.2) need more revolutions than five seeds will produce.

**Observer.**
- A person's "why", e.g. "joined the petition because the land claim remains unresolved, the organizer previously helped the household, and retaliation is considered unlikely" (04-06 §5.5).
- Faction and episode panels; episodes show their attendance curve and violence by kind.
- Each claim's coverage at T10, T50 and T90, or "not reached" (09-16 §4).
- Each law's versions.

## 4. Choices that are expensive to reverse

- **ADR candidate A: social memory and information.**
  - Covers relationship, grievance and information records with their provenance, the rule that decisions read only these, and the god-tool boundary.
  - Fixes several save sections and the wire's "why" payloads.
  - Keys draws by content id, not index; §9 records that one added technique shifted every later find's key.
- **ADR candidate B: political actors and episodes.**
  - Covers factions (membership kept apart from belief and activity), a treasury channel beside ADR-0006's fixed codes, episodes, armed alignment, and separate identities for leader, regime and constitution (09-11 §1.1).
  - Write it with the constitution brief, so one organization entity serves both.
- **ADR-0003's revisit triggers.** M4 reaches both of them.
  - Attendance's eight terms, plus walking, exceed "more than eight considerations". The receipt shape needs revisiting.
  - At about 1,500 people, per-person snapshot entries pass 64 KB, so political state belongs in inspector payloads, not the frame.
  - Joining and attending should be kept as life decisions, which ADR-0003 keeps forever, not in the ring of 64.
- **Append-only care, no ADR needed:**
  - Reason and chronicle codes;
  - god-tool commands;
  - strict new content kinds: norms, issues, ideologies, episode kinds.
- **Thresholds are rebuilt on load.** Changing their key or distribution silently re-rolls every one.

## 5. Open questions for the designer

1. **How winners rebuild.** Plan §5.2 has the winning faction rebuild "according to its ideology"; 09-11 §1.6 and §4 describe a coalition bargaining from an inherited state. Which governs?
2. **Ideology axes.** Are they the slow values, or the whole political state?
3. **Contacts.** How many per person? The reports range from 2–12 to 32–128.
4. **Grievance memory.** Does the two-layer split settle the reports' disagreement?
5. **Whisper truth.** May a whisper be false?
6. **Agitator.** A newcomer, or an existing person given conviction? 15-05 §4.2 argues against rewriting values.
7. **Bless/curse.** Which draws, by how much, for how long? Is it perceptible?
8. **Coups without force.** Is it acceptable that worlds with a communal watch cannot have a coup?
9. **Leaving.** Does it become a scored choice in M4? That needs the ten-year smoke.
10. **Gate B.** Should rare political events be covered, and with what tolerance?
11. **Faction size.** No report gives a minimum founding size, so one must be tuned.
