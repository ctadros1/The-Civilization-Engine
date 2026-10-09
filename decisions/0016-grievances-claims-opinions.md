# ADR-0016: Grievances, claims and opinions, and the interventions that touch them

Status: Accepted
Date: 2026-10-08
Milestone: M4c

## Context

M4a gave people ties and standing (ADR-0014) and each settlement a polity whose laws keep their
whole history (ADR-0013); M4b kept what happened, what a polity knows and what people believe
apart (ADR-0015). Nobody can yet hold a wrong against an office, hear news other than of a taking,
or hold a view. M4c brings grievance, news and rumour, opinion, ideology and norms, factions,
episodes and changes of regime, and four god tools (plan §7). Two of its decisions are expensive
to reverse: what people remember of the polity and of one another beyond ties, and how the
observer may reach into it. This ADR takes the first; ADR-0017 takes factions, episodes and
changes of regime.

Reports read: 04-06 (reputation and trust), 04-10 (collective action), 09-16 (information
flow), 06-04 (ideology and opinion), 06-05 (norms), 09-11 (revolutions and coups), 09-12
(structural demography) and 15-05 (god-game design). The M4 factions brief
(`docs/briefs/m4-factions.md`) digests them, with the observer brief (`docs/briefs/m4-observer.md`).
Their main points:

- **Separate, linked processes, not an unrest meter** (04-10, core recommendation; 09-11,
  opening): conditions are experienced, interpreted as a grievance, told, organized around, acted
  on and answered, and beliefs are revised.
- **People decide from their own sparse beliefs** (09-16 §1.1, 04-10 §5.2, 06-05 §5.5), never
  from world truth or settlement totals.
- **Private belief, public expression, membership and activity are separate state** (04-10
  §5.1, 09-11 §1.3, 06-04 §1.8).
- **Ten retellings of one report are not ten observations** (04-06 §1.5, 09-16 §1.6), as
  ADR-0015 already keeps for takings.
- **A grievance** is harm, a responsible party, a violated expectation, a demanded remedy and an
  unresolved portion (04-06 §1.7). It keeps a durable claim apart from an activation that fades
  and is raised only by reminders (04-06 §5.3). Blame needs a belief that the party had duty and
  power (06-04 §1.1); relative deprivation alone does not drive it (04-10 §1.1), and density must
  never generate discontent (09-12 §1.1).
- **Opinion persists with anchors** (06-04 §1.4: Friedkin–Johnsen, x′ = (1−a−b)x + a·m + b·z);
  models without anchors converge to consensus (06-04 §2). An ideology is a problem explanation,
  moral commitments, institutional proposals, a constituency and a legitimacy narrative (06-04
  §6.1). Norms are endorsement, an empirical expectation and a normative expectation (06-05 §5.1).
- **God tools submit only physical changes and perceptible events** (15-05 §4.2), never an
  action, a law, a vote or an allegiance; one record per recipient and proposition, repeats
  refresh rather than stack (15-05 §4.5); every use is logged and its effect traceable (15-05 §6).

## Decision

### 1. Social memory is per person, sparse and sourced

- Beside ties (ADR-0014) and beliefs about takings (ADR-0015), a person may hold:
  - **grievances** (§2);
  - **what they have heard** of shared claims (§3);
  - **positions** on issues, slow values and norm attitudes (§4).
- Each record says where it came from: an event, a law, a teller and the witness the account began
  with, or an observer's intervention.
- **Choices read only these, ties, beliefs, the person's own state and what their household
  knows.** No choice reads a settlement total, another person's private record, an incident or a
  faction's true size. Derived quantities (perceived support, a faction's believed size) are
  worked out from the person's own records when needed and never saved.
- All of it is saved, and the schema version is bumped with each section added.

### 2. A grievance is a durable claim with a fading activation

- A grievance records:
  - the issue kind (content): subsistence, extraction, treatment or a collective claim, to begin;
  - the party blamed: a person, a household, an office or the polity's body;
  - the expectation it violated: a law's terms, by law id, or a norm, by content id;
  - the harm, in days of the household's food, and the remedy demanded;
  - the unresolved part, which only the remedy, a settlement or abandonment reduces;
  - its activation, 0 to 1, with the day last raised, and the event that raised it.
- **It arises only when harm meets an expectation the person holds, and blame.** Harm alone (a
  lean year), or blame without a held expectation, makes none. The events that can raise one are
  named in content and code, one by one, each in the slice that builds it.
- **Activation fades** by a half-life of its issue kind and is raised only by a reminder: new
  harm from the same party, hearing the grievance told, or seeing the party act again. It is never
  re-added each day.
- **The claim is durable:** it has no expiry while a part stays unresolved, but a person holds at
  most a content-set number, the least active giving way to a new one.

### 3. Claims are shared and immutable; hearing is per person

- **A claim** is a shared record: its kind (content), subject, place, time, quantity and the event
  or law it comes from. It is written once and never changed.
- **A person's hearing record** holds the claim, when they first and last heard it, from whom, the
  witness or record it began with, and how far they believe it. Hearing, believing, passing on
  and acting are separate steps; believing that others believe is derived (§1).
- **It travels only through contact:** the household at midnight, company at the hearth sampled by
  the hour, shared work, a gathering. At 1,000–2,000 people the hearth is never a broadcast.
  Several retellings of one origin are one source.
- **News in v0:** a gathering called, a law passed or turned down, a case found, an episode
  called, a grievance told. Word of a gathering, every adult's today (M4a), becomes a claim.
- **ADR-0015's beliefs about takings stay as they are.** They already follow these rules; folding
  them into claims would move saves for no new behaviour.
- **No distortion in v0** (09-16 §5.5): a claim is passed on as heard. Ideology and law are
  interpreted (§4), not mangled.

### 4. Opinion has anchors; ideologies and norms are content

- **Positions:** a person holds a position, a confidence and a salience on a small set of issues
  named in content. A message they accept moves a position by x′ = (1−a−b)x + a·m + b·z (06-04
  §1.4), where the anchor z is recomputed from the household's own material position and a
  depends on salience, trust in the teller, disagreement and redundancy. No negative influence in
  v0.
- **Values** are a few slow content axes, drawn at birth and pulled toward the caregivers', as
  objection to taking already is (ADR-0015 §2).
- **An ideology** is a content record: the problem it explains, its moral commitments, the
  institutional moves it proposes (packages of the law pipeline's own templates and the custom's
  amendments, ADR-0013), the constituency it speaks to and its legitimacy story. It spreads as a
  claim, and someone who holds it weighs its proposals among their moves.
- **Norms** are content templates. Each person holds an endorsement and the expectations they
  believe others hold, moved by acts they see and claims they hear. Norms supply the expectations
  whose breach makes a grievance (§2) and the moral terms of choices.

### 5. Interventions touch only what can be perceived

- The god tools are commands that submit a perceptible event or a physical change. None chooses
  an action, passes or blocks a law, sets an allegiance or writes a grievance, position or vote.
  - **Whisper:** a true claim (settled in the M4a design, plan §9: whispers are true only)
    placed in one person's hearing record, with a salience that fades over one to three of their
    decision cycles.
  - **Ideology:** one person, or the witnesses of a public sign, hear of a content ideology, as
    `IntroduceTechnique { aware_only }` makes a technique known (ADR-0008 §6).
  - **Agitator:** a newcomer holding an ideology, sent by the path that sends a family (M1); an
    ordinary person bound by every law, starting with a stranger's ties.
  - **Bless or curse:** for a stated period, shifts the target's own keyed draws for named
    material purposes (work quality, spoilage, illness) by a stated quantile. Never a decision's
    draw, a vote, a tie, detection or punishment.
- **One record per recipient, proposition and episode.** A repeat refreshes it and never stacks;
  a hundred identical whispers equal one.
- **Every use is logged:** an intervention id is saved, the chronicle says "one recorded
  influence", and receipts and records it touches carry the id. The observer shows whether it
  was perceived, considered, acted on and passed on.

### 6. Draws are keyed by content id

- New draws (thresholds, sampling of company, a whisper's perception) are keyed by the content
  ids they concern, never by catalog index (§9 recorded that one added technique shifted every
  later find's key). Thresholds are rebuilt from their keys on load; changing a key re-rolls them,
  so keys are append-only.

### 7. The boundary

- Saves gain sections for grievances, claims and hearing records, positions and values, norm
  attitudes and interventions, each bumping the schema version as it lands.
- The wire carries these in inspector payloads for one person, not in the frame: at about 1,500
  people per-person political state would pass ADR-0003's 64 KB frame budget (ADR-0003's revisit
  trigger).
- Reason and chronicle codes, content kinds (issues, norms, ideologies, claim kinds) and god-tool
  commands are appended, never renumbered.

## Consequences

- Discontent has causes a person can be asked about: a grievance names its harm, its party and
  the expectation broken, and how it was heard.
- Word now has a cost: a gathering is heard of by contact, so turnout and knowledge of laws follow
  ties, not a broadcast.
- Saves grow by the records people hold; the caps on grievances, hearing records and positions
  are content values, chosen with the measured memory.
- The god tools can do nothing a person could not have perceived; their effect can be followed
  from the record to the choice.

## Alternatives considered

- **An unrest meter per settlement, or anger per person.** Rejected: the reports agree that
  grievance, communication, organization and choice are separate, and a meter makes revolt a
  threshold (04-10, core recommendation).
- **Folding takings' beliefs into claims now.** Rejected: ADR-0015's beliefs already follow these
  rules, and moving them would change saves for no behaviour.
- **Opinion without anchors (DeGroot, voter models).** Rejected: they converge to consensus
  (06-04 §2), and the regimes demo would be decided by the model, not by history.
- **God tools that set a value or a vote.** Rejected by 15-05 §4.2 and the project's rule that the
  engine authors vocabulary, never plot.

## Revisit when

- Distortion, propaganda or censorship is wanted: claims are immutable and passed on as heard.
- Takings' beliefs and claims need one query: fold them then.
- The caps on records bind measured behaviour, or saves outgrow their budget.
- Several polities (M5) carry news between settlements.
