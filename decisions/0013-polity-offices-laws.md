# ADR-0013: The polity, its offices and its laws

Status: Accepted
Date: 2026-10-07
Milestone: M4a

## Context

Today a settlement has no government:

- It holds land under the world's property regime (ADR-0007), but no goods.
- The village-fields regime allocates fields by need, but no one holds that rule; it is automatic.
- Nobody holds an office, and no law exists.
- When a harvest falls short, households run down their stores and then leave the valley
  together (§9: villages of about 55, and once of 95).

M4 brings councils, law and crime. Its first part, M4a, brings the polity, its offices, bodies
and powers, the law pipeline's stages, a common store and label inference (plan §5.2, §7).

Reports read: 09-01 (state formation), 09-02 (constitutional primitives), 09-03 (succession),
09-04 (legitimacy), 09-05 (lawmaking), 09-14 (municipal governance), 09-15 (regime typology),
04-11 (leadership and elites), 06-09 (commons governance), 08-11 (non-market allocation), 02-06
(academic society simulations) and 07-09 (information technology). Their main points:

- **Institutions are positions, bodies, powers and procedures held by real people.** They act
  through their members (09-02 §1.1, 09-01 §6.1).
- **Starting without government means no standing offices or compulsory treasury, not no
  rules** (09-01).
- **Offices arise from proposals that answer recurring problems** (09-01 §6.2, 04-11 §5.4,
  06-09 §1.2). No population threshold makes them appear (09-01 §3.1). The reports give no
  rates for how often offices form (09-01 §8).
- **Selection is a composition of steps:** eligibility, candidates, selection, confirmation,
  installation (09-02 §1.3, 09-03 §1.2).
- **Compliance is each person's decision** (09-02 §3.2, 09-04 §5.4).
- **Capacity is work.** Enforcement is officers' hours, with a backlog (07-09 §1.2), not a
  scalar formula (09-02 §5.2).
- **Legal title, recognition and control are separate.** A succession is an episode with claimants
  (09-03 §1.1, §5.4).
- **A law's history is a record:** proposal, supporters, opponents, binding constraints, and
  the obligations that result (02-06 §5). The engine's knowledge of a law is nobody's knowledge:
  people know only what reached them (07-09 §1.1).
- **Labels are summaries worked out afterwards** (09-15 §6.7; 09-01 §6.5: "the label must never
  grant the capabilities used to classify it").
- **Failures stay failures.** An empty candidate pool, a failed quorum or a tie takes an
  explicit failure path; it is never repaired (09-02 §10).

## Decision

### 1. A polity per settlement

- Each settlement has a polity: an identity of its own, its members, its constitution, its laws
  and its store. In M4 a polity and a settlement are one to one; M5's new settlements need the
  separate identity.
- Membership in v0 is residence. Content names the founding custom every world starts from.
  - The core custom is a **gathering**: the adults who come to the hearth decide by
    acclamation. Anyone may propose. The same gathering amends the custom.
  - Every world of a preset starts from the same custom, so differences come from history.

### 2. Constitution primitives are values over authored vocabulary

- **Body:** a membership rule (adults, household heads, holders of an office, or seats filled by
  a procedure). It has a decision rule (quorum, denominator, by person or household, threshold,
  ties, open or secret), an advisory or consenting role, a calendar and an agenda holder
  (09-05 §1.3–1.4).
- **Office:** a content title, seats, eligibility, a selection procedure, removal rules, powers
  and pay. Its tenure is stored as conditions, not days alone (09-02 §8.6).
- **Power:** a holder, an action, a domain, conditions and whether it can be delegated
  (09-02 §1.2). The actions in v0 are:
  - propose and set the agenda;
  - enact;
  - levy and allocate from the store;
  - command;
  - appoint and remove;
  - amend.
  M4b adds sanction; M4c adds command of force.
- **Selection:** steps composed in order, as values (09-02 §1.3):
  1. eligibility;
  2. candidates;
  3. a rule: descent rank, a vote, appointment, lot (a keyed draw), acclamation, or seizure
     (M4c);
  4. optional confirmation;
  5. installation.
- **Saved as values.** Primitives are saved as values that refer to content vocabulary by
  content id: office templates, policy templates and issue kinds.
  - A save naming a kind the loaded content lacks is refused (ADR-0002).
  - A default is never substituted, as that would rewrite a constitution.
- **No failure is repaired.** An empty candidate pool, a failed quorum or a tie ends in its own
  recorded outcome, and the chronicle says so.

### 3. Laws are versioned, and their history is kept

- **A law is a typed policy from a content template.** Its parameters are set by the pipeline,
  such as a share of the harvest or who is covered. It has an id, versions and a status: proposed,
  decided, in force, amended, repealed or lapsed.
- **Every stage is recorded with the people involved:**
  1. The proposal: the sponsor, and the issue it answered.
  2. Support: each member's stance with its main reason, summarised.
  3. The decision: the procedure, the attendance and the result.
  4. Promulgation: who knew it, and when.
  5. Enforcement: the hours officers spent on it.
  6. Compliance: who complied, could not comply, or evaded.
  7. Effects: the goods moved.
  The history is saved and never pruned; the observer shows it whole.
- **Knowing a law is knowledge people carry.** Those present at a decision know the law. Others
  learn it from those who know it (M4c's information spreads it further).
  - A person's choices weigh only laws they know.
  - Officers enforce only laws they know.
- **Compliance is the person's choice.** Where a law touches a choice, it is a consideration in
  the person's ordinary scoring: the expected sanction, the authority's standing with them, and
  the cost of complying. Being unable to comply is recorded apart from refusing (09-01 §6.4).

### 4. The polity holds goods

- The polity is a holder in the ledger (ADR-0006).
  - Every good it receives or gives moves on a channel: levy, relief, office pay.
  - The rule that every good is accounted for covers it.
  - It amends ADR-0006 §3 and ADR-0007 §1, which held that settlements hold land but no goods.
- What the store holds is kept apart from any officeholder's household (08-11 §1.3).
  - The goods stay where they are kept, under a roof the polity uses.
  - They wear and spoil as any household's do.

### 5. Deliberation is scored moves behind a trait

- Notables and officeholders choose institutional moves through a `Deliberator` trait.
  - It takes a copied view of the facts and the moves available.
  - It returns a choice with its receipt.
- v0 is rule-based and one level deep: it scores the authored moves that the person's powers
  and the present issues allow. HTN planning, which plan §4.2 names, waits for M7's multi-step
  plans.
- **Issues only make moves available.** An issue, such as a food shortfall against the outlook,
  a vacant office or fields short, never carries a weight toward a particular policy. A move's
  score comes from its forecast effect on the person's goals: their household's food outlook,
  their followers' outlook and their standing. An issue→policy weight would make "a levy follows
  a famine" plot (plan §1).
- Draws are keyed (seed, purpose, person, day). An `LlmDeliberator` can be plugged in later (plan
  §8).

### 6. Labels are derived, never read

- A pure classifier maps the polity's de jure graph and de facto evidence over a window to
  names from 09-15 §2's terms, with a confidence and the reasons why.
  - The de jure graph is who selects, removes, levies, allocates, enacts and amends.
  - The evidence is decisions obeyed, the effective number of decision-makers, and whether
    authority outlived its holder.
- It is recomputed from saved history and never saved itself.
- Nothing in the simulation reads a label; the module structure enforces this. Turning the
  classifier off leaves every digest unchanged.

### 7. The boundary

- New wire tables, queries and chronicle span kinds are appended: the polity, its offices and
  holders, its laws and their histories, the store and the label.
- Saves gain sections for the polity, its constitution, its laws and its store; the schema
  version is bumped.

## Consequences

- Institutions emerge from people's proposals and choices. Every world starts from the same
  custom, so differing regimes must come from history.
- A law's full history is inspectable, the demo's need.
- A common store gives a village answers to a short year other than leaving, if its people
  choose to keep one.
- Saves bind to content ids for institutional vocabulary. Renaming or removing an office or
  policy template breaks old saves by refusal, as for activities today.
- **Forbidden:**
  - population thresholds or timers that create offices;
  - issue→policy weights;
  - anything that reads a label;
  - repairing a failed procedure;
  - an office knowing what nobody told it.

## Alternatives considered

- **A ladder of polity types (band → tribe → chiefdom → state).** It is a complexity ladder,
  rejected by the reports (02-06 §3, §5).
- **Office creation as a yearly roll by population.** The reports give no such rates
  (09-01 §8), and it would be plot.
- **HTN planning now.** It costs more than one-level moves for no M4 behaviour that needs plans
  of several steps.
- **A scalar enforcement capacity (plan §5.2's formula).** Capacity is derived from officers'
  hours instead, and the formula becomes a measure the observer reports (09-02 §5.2).

## Revisit when

- Several polities exist (M5): membership, citizenship and jurisdiction over distance.
- Courts arrive (M7): forums, evidence rules and appeals between "identified" and "sanctioned".
- Elections with parties arrive (M7).
