# M4 brief: crime, policing and justice v1, corruption v0, policy library v0, compliance

**Scope.** Theft and its discovery, responses without courts, a watch, punishments, the v0 policies (tribute/tax, land tenure, levy, curfew, prohibitions), compliance, enforcement as finite work, and bribing enforcers. Offices, pipeline stages 1–4, legitimacy and grievance come from other briefs.

Every weight is a **tuning value** unless a citation gives it. 04-09 §5.4 finds no established values for belief-update rates, moral-cost weights or "the conversion of one unit of hunger into offending propensity". Content authors policy templates, offence classes, sanction bundles and weights. People set a law's own figures, such as a 10 % levy, through the pipeline.

**One rule throughout.** Keep three things apart (06 §5.2; 12-04 §5.2; 04-09's opening):

- what happened (the kernel's truth);
- what an office knows (cases);
- what each person believes.

Officials act on what the office knows, and people on what they believe. Crime rates are outputs; no controller pulls a settlement toward a rate (04-09 §5.10).

## 1. Mechanisms

### 1.1 Theft as a choice

**Model.** Append `Take` to `Behavior` (a boundary enum: append, never reorder). It is scored with the same considerations and softmax as `Ask`. It competes with asking, gathering, trading and leaving, since strain can also lead to help-seeking or migration (04-09 §1.1).

Targets are known households' stores within walking range, found as `Ask` finds a giver: awareness and access from a bounded set (04-09 §5.2).

Terms follow 04-09 §5.3 (U = G + R − C − M − I − Σp̂L), in the existing points:

- **Gain.** `Ask`'s food-shortage × worth term, so a full household gains little; stolen food is worth what the household needs, not its price (04-09 §5.3). A small greed term values other goods at settlement prices (plan §5.4 names need, greed and grievance).
- **Costs.**
  - Walking.
  - The person's objection to taking.
  - Help refused to known takers.
  - The expected sanction: the *perceived* chance of being seen and punished, times the sanction converted to days of household food (one common scale, 04-09 §5.3). It is discounted over time (04-09 §5.4) and weighted by the `risk` trait.
- **A moral filter first.** Above a threshold objection, `Take` is excluded with a receipt reason. Otherwise a noisy chooser "may eventually make almost everyone offend" (04-09 §5.3).

**Outcome.** Guardianship is settled on arrival. If a member of the target household is home and awake, or anyone (a watchman included) is in sight, the taker abandons the attempt or is seen. Attempts and completions are both recorded (04-09 §5.10).

A completed take moves goods on a new `take` ledger channel, capped at a gift's carry limit. Theft transfers goods and never creates them (04-09 §5.7; 18 §1.1), so ADR-0006's conservation check covers it.

**State.**

- **An objection to taking.** A keyed draw at birth, pulled toward the parents' (social learning, 04-09 §1.1) and moved by experience. It is not a "criminal personality" (04-09 §5.1), and it is kept apart from impulsiveness (04-09 §1.2).
- **Sparse perceived risks, one per offence domain.** Updated as p̂ ← (1−α)p̂ + αs from the person's own attempts, the sanctions they see and their kin's experience (04-09 §5.4; 06 §1.6).
  - The half-life is a design prior on which the reports differ: 06 §2.3 tests 30, 180 and 720 days; 18 §2.3 gives 30–365; 09 §2.3 gives one year.

Grievance as a motive waits for the factions brief.

### 1.2 Discovery and community response

The victim notices the loss at its next use of the store. It knows the taker only if a member saw, or a witness tells; witnesses tell according to their ties to each side (12-04 §1.2).

The victim then chooses, with a receipt, among:

- letting it go;
- demanding restitution (a `restitution` transfer, which may be refused);
- telling kin and neighbours;
- reporting to an office that keeps order.

This is 07 §1.1's branching process, with the forum chosen per 07 §4.1: U = p̂(success)·p̂(enforcement)·V − C.

Informal sanction reuses `Ask`. A household that knows someone took from it, or from its kin, refuses their requests; today `give_food` gives to anyone who asks. Exclusion becomes credible once enough households know (12-04 §1.2). A settlement with no office is not lawless (12-04 §4.A; 06 §3).

Victims adapt, for example by staying home more. Otherwise repeat victimisation "can create permanently doomed households" (04-09 §5.6).

### 1.3 The watch

Policing is people doing an activity, not "a building that reduces crime within a radius" (12-04, opening). A policy creates a watch in one of two forms:

- a rotating household duty, which costs "time taken from production and rest" (12-04 §1.3);
- an office paid from the treasury.

Watchmen walk set rounds. One account describes three a night, at about 21:00, 00:00 and 03:00 (12-04 §2.1, medium-low confidence). They guard and witness only where they are, and would-be takers who see them update p̂ (12-04 §1.4).

Staffing is tuning: 12-04 §2.4 tests 0–4 staff per 1,000 people, with 20–70 % of their time on patrol. By its formula (1,800 h a year, 60 % on patrol), 2,000 people at 2 per 1,000 keep about 0.5 watchmen out at a time, so the timing of rounds matters.

### 1.4 Cases and the summary decision (no courts)

A report or a sighting opens a case (12-04 §1.5). A case holds:

- the allegation;
- leads with their sources; three people repeating one rumour make one lead;
- its stage and the hours spent;
- a recorded closure reason (06 §1.3).

Investigation works on leads: P(useful) = 1 − exp(−κqh). A case with no lead waits (12-04 §5.3), and identifying someone is not catching them (12-04 §1.5).

"No courts" then means no forum, evidence rules, panels, precedent or review. The office holding the power to judge decides summarily from its case, against a confidence threshold set by policy. A decision costs 0.25–3 adjudicator-person-days (a design prior, 07 §2.3).

The holder is an ordinary person with kin and motives (06 §1.8), which is how favouritism and bribes enter. Shape the case so that M7 can put a forum between "identified" and "sanctioned".

### 1.5 Punishments

A schedule maps each offence class to a **bundle**, never to one severity number (07 §6.2). The v0 bundle can combine:

- restitution;
- compensation to the victim;
- a fine to the treasury;
- labour-days owed, through the levy machinery;
- exile, recorded as a migration, not a disappearance (06-10 §1).

The first three are different transfers with different effects (07 §1.2). Every sanction is an obligation with a debtor, a beneficiary, a due date and a default rule. A fine is paid from something or becomes an arrear (06 §5.4).

Keep prescribed, imposed and completed sanctions apart (07 §5). At the Old Bailey, executions per capital sentence fell from 68.5 % in the 1750s to 10.9 % in the 1810s (07 §2.1).

Severity gets no automatic deterrent effect; the evidence favours the certainty of being caught (04-09 §2.2). Fines can reuse 06 §2.3's grid of 0.25, 1 and 3 times the liability. Status-differentiated schedules can be authored; the Mercian wergild ran 6:1 (07 §2.1).

### 1.6 Policies and compliance

Every v0 policy has one typed shape (06 §5.1; 13 §5.1): coverage, required or forbidden conduct, assessment and exemptions, medium and calendar, responsible office, sanctions. Legality is a function of (jurisdiction, date, actor, activity, good, quantity, authorisation), so a law reclassifies acts without touching goods (18 §1.1). Laws are indexed by the event they touch, never checked against everyone on every tick (06 §5.3).

Where a law touches an act, the person chooses to comply, comply in part, conceal, plead inability or evade. Evading x gains x − q(1+f)x, minus the costs of concealment, conscience and reputation, with q *perceived* (06 §1.5; 13 §4.1).

Legitimacy, agreement with the law, what neighbours do and their expected reaction are separate terms (06 §1.7). Inability, unwillingness, ignorance and disputed liability stay distinct, so every arrear records its cause (06 §1.5; 13 §4.1).

**Tribute and tax.** Two instruments:

- a share of threshed grain: 10 % is the canonical tithe (13 §2.2), and 13 §2.3 tests 5–30 % of the current harvest;
- a fixed charge per household or per hectare.

The collector walks round after threshing and assesses only what the office has observed. Fields are visible; yields are not, unless the threshing was watched (06 §3; 13 §1.4, §5.6). A fixed charge of 20 % becomes 40 % of a halved harvest (13 §1.5).

Accounts keep assessed, paid, collected and remitted apart (13 §1.1). Tokugawa assessments of 34–55 % of *assessed* yield (13 §2.2) bound plausibility only. With one settlement, "tribute" can only mean dues to an office holder.

**Levy.** Labour-days per eligible worker, kept in person-days; 13 §2.3 tests 0–60 days a year. Workers attend, send a substitute, pay in goods or stay away. Harvest calls already cost more because the scores value field work then (13 §1.2).

**Curfew.** Nobody may be off their own plot in set hours: a compliance term on night trips, enforced by the watch that also guards against theft. Curfews have enforced status hierarchies (12-04 §1.1, Charleston), so the observer names who is covered.

**Prohibitions.** A prohibition can name an activity, good, place, time or person: a reserved patch, a grove, a sale, assembly. It removes lawful supply, not the wish for the thing (18 §1.2).

**Land tenure.** ADR-0007's regimes become laws, and a change applies at the next yearly review. Nobody refuses a reallocation in v0, because tenure disputes need forums (M7).

### 1.7 Enforcement capacity

Plan §5.2's "officers × competence × (1 − corruption)" should be a reported measure, not an input. The reports split capability into fiscal, administrative, coercive and political parts, and keep "cannot enforce" apart from "will not enforce" (06 §1.1).

Capacity is hours on the roster:

- Every stage costs work, and assignments must fit, Σh ≤ H (06 §1.3–1.4). A new law with the same staff lengthens queues (06 §4).
- Case priority is institutional policy (12-04 §5.5).
- An annual tax gets no fresh daily audit chance (06 §2.3).

### 1.8 Corruption v0: bribing enforcers

Corruption exists only where someone has discretion or holds entrusted goods, never as a daily chance of "being corrupt" (09 §1.2). v0 takes two of the three starting offices 09 §5.4 suggests:

- **The enforcer who identifies an offender.** The offender offers, or is asked for, a payment no larger than its household holds and no dearer than its expected sanction. The official weighs ΔU = B + K − C − M − p̂(F + V), where K is the value of helping kin and V is the value of the office it loses on dismissal (09 §1.3). The bribe moves on a `bribe` channel (09 §1.8).
- **The collector.** It can under-assess for a payment, or keep part of the flow up to a prior ceiling of 0.15 (09 §2.3).

Exposure comes from witnesses, victims who see a known taker go free, and rivals; v0 has no audits. Dismissal follows the office's removal rule. The chain's priors are unestimated (09 §2.3: usable evidence given an audit 0.40, sanction given proof 0.50, dismissal given sanction 0.75).

Only learning reinforces corruption: an unpunished act lowers p̂. Protection and recruitment wait for M7 (09 §1.5). No person carries a `corrupt` flag (09 §5.1; 12-04 §5.1).

### 1.9 State added (saved)

| Holder | New state |
|---|---|
| Person | objection to taking; sparse perceived risks; incident links; obligations |
| Household | known takers; tax accounts, arrears with causes |
| Settlement | treasury goods (a goods holder, which ADR-0007 §1 postponed) |
| Incident (truth) | class, actor, target, goods, time, place, witnesses, attempt or completion |
| Case (knowledge) | allegation, sourced leads, stage, hours, decision, closure reason, bribe |
| Obligation | kind, amount, debtor, beneficiary, due, progress, default; prescribed, imposed or completed |
| Ledger | channels `take`, `restitution`, `compensation`, `fine`, `tax`, `commutation`, `bribe` |

Capacity measures and norm caches are derived, and rebuilt on load.

## 2. Deferred, and why

- **Courts** (M7, per the plan): forums, evidence rules, precedent and appeal (07 §1.3–1.5, §4.4).
- **Assault and homicide.** Death runs through injury (04-09 §5.5), which the kernel does not model. Homicide also cannot be validated at this scale: Europe's 15–60 per 100,000 a year in 1200–1450 (04-09 §3.1) means 0.3–1.2 a year for 2,000 people (arithmetic).
- **Confinement.** It needs custody, food and guards (07 §3.1).
- **Rackets, smuggling and black markets** (18 §1.3–1.7). They need organisations, price ceilings, tariffs or a second jurisdiction (M5).
- **Tax farming, audits and patronage** (13 §1.3; 09 §5.3). Plan §5.2 puts full corruption and patronage in M7; this brief defers the other two with them.
- **Relief institutions** (17 §5.2). They are not in M4's v0 list; see question 4.

## 3. Pitfalls, tests and the observer

**Pitfalls.**

- **Omniscient officers.** An office must not know "things nobody has told or shown them" (12-04 §5.7; 13 §5.6). Test: everything an office knows traces to a witness, report or sighting.
- **Everyone offends** (04-09 §5.3). Test: takes are near zero in a well-fed world, and concentrated in a few people. In Sweden 1 % of people held 63.2 % of violent convictions (04-09 §4; direction only).
- **Recorded crime is not crime.** Better detection can raise recorded crime (06 §4; 12-04 §4.B). Avoid single scalars for capacity, corruption or severity (06 §1.1; 04-09 §2.2).
- **Informal sanction is not benign.** Liberian mediation cut violence but raised extrajudicial punishment (07 §3.2).
- **Small numbers** (04-09 §5.8). Pool over seeds, and diagnose with attempts, reports and refusals.
- **Iteration order.** Collect competing claims before allocating (17 §5.3). Key draws by seed, purpose and permanent id.
- **Accelerated mode.** A take moves goods, so it must refresh household views as a gift does (ADR-0011 §4). Add takes to Gate B.
- **Shared shortage.** `Ask` fails when everyone is short (17 §1.2), so takes will cluster just before households leave together (§9's NUDGE of 2026-10-07). Check that theft does not merely hasten it.
- **Dark content.** Use plain language and named actors (06-10 §4.C–D). Presentation settings must not consume draws (06-10 §4.H).

**Paired tests**, each changing one thing (04-09 §5.10):

1. With need-based help off, takes rise (the direction of 17 §2.2: English relief cuts, about 17.2 % more property crime; moderate confidence).
2. Plan §4.7's "crime correlates positively with poverty", from true incidents by household food days, with recorded incidents beside them.
3. A watch cuts takes where it walks, not everywhere. 12-04 §2.3 contrasts Kansas City's null routine patrols with Philadelphia's targeted foot patrols (about 23 % less violence).
4. Doubling severity moves less than doubling certainty (04-09 §2.2).
5. Without enforcement, compliance stays above zero: about 20 % paid an unenforced church tax (06 §1.7).
6. A self-declared base is concealed and an observed one is not (06 §2.2's Danish contrast, direction only).
7. When collectors keep part of the take, receipts and bribes can rise together (06 §4; 09 §4).
8. In a lean year a fixed levy's burden rises and a share's does not (13 §4.3).
9. Conservation; save → load → save digests; Gates A and B.

**The observer** shows:

- incidents in three columns: truth, office knowledge and beliefs;
- case timelines with closure reasons;
- one law's full history (the M4 demo);
- treasury accounts and household burdens;
- receipts showing `Take` scored, or filtered out.

## 4. Expensive to reverse: ADR candidates

M4's budget is about three ADRs across all briefs; this topic needs two.

1. **Incidents, cases and obligations.** It covers:
   - save sections, including each person's beliefs;
   - the settlement as a goods holder (amending ADR-0006 §3 and ADR-0007 §1);
   - channel codes;
   - wire fields that keep truth and office knowledge apart, so the web cannot mix them.
2. **Policies as typed content, laws as versioned state.** Shared with the law-pipeline brief, it covers:
   - the content API for policy templates, offence classes and sanction bundles;
   - law history in saves;
   - regimes as laws (amending ADR-0007 §2's "fixed for the world's life").

These need no ADR: appending `Take`, reason and death-cause codes, and tuning values.

## 5. Where the reports are thin or disagree

**Thin.** The reports have no figures for:

- premodern theft (04-09 §3.4, §6);
- historical enforcement probabilities (06 §6);
- early-farming tax ratios (13 §2.1);
- early-farming corruption rates (09 §6);
- police per head (12-04, opening).

Modern US reporting and clearance rates (04-09 §4; 12-04 §2.2) show direction only, and must not be multiplied together (12-04 §2.2).

**Disagreements.**

- **Courts.** 07's opening wants customary adjudication and precedent from the start, while the plan defers courts to M7. §1.4 is a compromise.
- **The justice sequence.** 07's opening rejects "crime → arrest → punishment", plan §5.4's phrasing. This brief follows 07.
- **When crimes are decided.** Plan §4.1 makes committing a crime a monthly decision; 04-09 (opening, §5.2) makes it encounter-driven. Here theft is in the activity scheduler.
- **Capacity.** Plan §5.2's capacity formula conflicts with 06 §1.1 and 09 (see §1.7).
- **The Danish audit.** 06 §2.2 gives 0.23 % against 17.1 % (positive income); 13 §2.2 gives 0.3 % against 37 %. 06 warns that net-income denominators inflate the figures, so use the direction only.

## 6. Open questions

1. Does a summary decision by a judging office count as "no courts"?
2. Are corporal punishment and execution in v0? 06-10 §4 requires the engine to answer "who did what to whom, under which institution or decision", and a world setting to truly prevent them (06-10 §4.F).
3. Should the watch be a rotating duty, a paid office, or either?
4. Should releasing treasury grain be in v0? Taxes in kind create a store, and §9's NUDGE asks M4 for "other answers to a short year than leaving".
5. May people take from kin, standing crops or workshops?
6. Should sanctions be scored in days of food or days of work?
7. A prohibition test needs a discretionary good (18 §5.4), and the content has none.
