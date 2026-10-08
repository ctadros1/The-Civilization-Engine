# ADR-0017: Factions, episodes and changes of regime

Status: Accepted
Date: 2026-10-08
Milestone: M4c

## Context

ADR-0013 gave each settlement a polity whose constitution is values over content vocabulary
(a body, offices, powers, selection), amended by the custom's own procedure. ADR-0016 gives people
grievances, news, positions, ideologies and norms. M4c's remaining contents (plan §7) are
factions with treasuries on the ledger, episodes (petition, protest, riot, coup, revolt) as each
person's choices, and the winners rebuilding the constitution. How organizations, collective
episodes and changes of regime are kept and saved is expensive to reverse.

Reports read: 04-10 (collective action), 09-11 (revolutions and coups), 09-12 (structural
demography), 06-03 (religion, for organizations and splits), 06-04 (ideology), 04-06 (reputation
and trust), 06-10 (dark content) and 02-06 (reference simulations), digested in the M4 factions
and observer briefs (`docs/briefs/`). Their main points:

- **First tell what changed** (09-11 §1.1): a leader, a regime (who effectively rules) or a
  constitution (the rules). Leaders change more often than regimes (09-11 §4).
- **A faction is an organization** with organizers who articulate, recruit, connect and
  negotiate (04-10 §1.5, §5.1). It forms from a grievance shared among trusted contacts, an
  organizer, expected efficacy and tolerable risk, never because "enough agents are angry"
  (04-06 §1.8). Popular organizations need no elite (09-12 §1.5).
- **Taking part is a choice** with grievance, identification, efficacy, moral pull, the share of
  one's own contacts believed to take part, organizational help, opportunity cost and expected
  sanction (04-10 §5.3); thresholds differ and some free-ride as others join (04-10 §1.3–1.4).
  Escalation is not a ladder (04-10 §3). Peaceful action, damage, violence and repression are
  recorded apart (04-10 §1.8, §4.2), and repression acts differently on victims, witnesses and
  hearers (04-10 §1.7).
- **A coup is not the top of a riot meter** (04-10 §1.9; 09-11 §1.4): it needs insiders who hold
  force, and willingness, coordination and success are separate. A revolution is a change in
  effective authority: revenue, compliance, coercion and administration changing hands through
  officeholders' own choices (09-11 §1.5). There is no per-tick roll (09-11 §2.4), and no
  historical rate transfers to a village (09-11 §2.4).
- **Winners bargain from what they inherit** (09-11 §1.6): provisional authority, representation
  through existing bodies, proposals from known primitives, bargaining against conflict, exit or
  submission, ratification and implementation. Losers keep wealth, contacts and claims (09-11
  §1.7).
- **Dark content** (06-10): every death and exile resolves to a record and a cause, told in plain
  verbs with the actor and the institution named; the narrator never justifies.

## Decision

### 1. Leader, regime and constitution are told apart

- The constitution is ADR-0013's values. Its history becomes versions: each change records the
  law or the founding that made it.
- Every change of who rules is classified when it happens, and the chronicle says which:
  - **succession:** a holder changes under the constitution's own selection;
  - **amendment:** the rules change by the custom's own procedure;
  - **replacement:** the rules change outside it, by seizure and founding.
- Amendment needs no new primitive: ADR-0013 §2 already gives the custom the power to amend the
  body (who belongs, the decision rule) and to create offices with powers. M4c adds the content
  templates that propose such changes; the gathering decides them as it decides any law.

### 2. A faction is an organization with a treasury

- A faction records its program, its organizers, its meeting place and time, its history and
  its treasury.
  - **The program** is a remedy for a grievance or an ideology's proposals: moves of the law
    pipeline and amendments of the custom, by content id.
  - **The treasury** is a holder on the goods ledger (ADR-0006) and gains a channel for what
    members give it, appended to ADR-0006's codes.
- **Membership is kept on persons**, apart from what they believe and what they are doing
  (04-10 §5.1). A member may disagree in private and a supporter may never join.
- **Founding is a person's scored choice**, open to anyone: a grievance or an ideology they hold,
  shared with contacts they trust, against the cost and the risk they believe. A faction ends
  when it has no members, recorded as such.
- **Joining and leaving are reviewed monthly**, each person on a keyed day of their own (09-12
  §2.2: one to three months; 09-11 §2.5: 7–30 days), and within a day of a crisis they hear of.

### 3. An episode is called, and attending is a choice

- An organizer calls an episode: its kind (content: petition, assembly, refusal of a levy), its
  issue, its place and time, and whom it addresses (the gathering or an office).
- **Attending is an activity** scored by the existing chooser, with 04-10 §5.3's terms, each from
  the person's own records: grievance, identification with the faction, believed efficacy, moral
  pull, the share of their own contacts they believe will come (an absolute count beside it),
  organizational help, what else they would do and the sanction they expect. Thresholds are keyed
  draws, rebuilt on load (ADR-0016 §6).
- **Within an episode** damage to property and violence against people are separate choices, made
  per person every few minutes; the episode records peaceful action, damage, violence and
  violence by officeholders apart, with unique participants, participant-days, the peak and the
  duration.
- **Answers are officeholders' choices:** the gathering or an office concedes (a proposal it then
  decides), refuses, or, where an office commands force, represses. A refusal is a new event that
  can make a new grievance; it does not raise a meter.
- **Any death or injury resolves to a record and a cause** (06-10), with new cause codes appended.

### 4. Seizure needs force or effective authority, and has no roll

- **A coup** is open only to those holding an office that commands force (the watch, when it is
  several). Each armed holder chooses to back the incumbents, the challengers, neither, or to
  leave. A coup holds when the challengers keep control for seven days (09-11 §2.2's coding).
  A polity with no office of force cannot have a coup; that is recorded, not worked around.
- **A revolt** is a faction whose program replaces the body or its offices. It succeeds when
  effective authority changes hands through officeholders' own choices: the storekeeper gives the
  store to it, the watch backs it, members comply with it rather than the gathering.
- **Losers keep** their wealth, ties, grievances and claims.

### 5. Founding is bargaining from what was inherited

- After a seizure: provisional authority over the surviving offices; representation through
  bodies that exist (the winners' faction, the old gathering); proposals built from the inherited
  constitution and the factions' programs; each faction scores a proposal by fit, material
  position, security, cost and exclusion against conflict, leaving or submission (09-11 §1.6);
  ratification by the procedure agreed; appointments.
- The result is a new constitution version, never an average of opinions (06-04 §6.3), and the
  chronicle tells replacement from amendment.

### 6. The boundary

- Saves gain sections for factions (with memberships on persons), episodes and constitution
  versions, bumping the schema version as each lands. The ledger gains an appended channel.
- The wire gains faction and episode panels and a person's "why" for joining or attending;
  political state for one person travels in inspector payloads (ADR-0016 §7).
- The chronicle follows 06-10: plain verbs, actors and institutions named, no justifying words
  ("order restored", "rebels crushed" and their like are refused by a test).

## Consequences

- A village's history can change who rules it, by its own procedure or outside it, and the
  record says which; the regimes row and the M4 demo can compare worlds by what happened.
- Collective action costs people something and can fail; repression can deter or outrage.
- Worlds without an office of force cannot see coups. This is a property of their history, not a
  gap to fill with a rule.
- Episodes add per-person choices every few minutes while they last: their cost is bounded by
  attendance and measured with the slice that builds them.

## Alternatives considered

- **Factions as opinion clusters.** Rejected: membership, belief and activity are separate state
  (04-10 §5.1), and a cluster cannot hold a treasury or call an episode.
- **An escalation ladder (petition → protest → riot → revolt).** Rejected by 04-10 §3 and §1.9.
- **Coup and revolt success as rolls on settlement totals.** Rejected: 09-11 §1.4–1.5 and §2.4;
  success is effective authority held, through people's choices.
- **Winners impose their ideology's ideal constitution.** Rejected: 09-11 §1.6 finds winners
  bargain from inherited arrangements; plan §5.2's "according to its ideology" holds as the
  programs that winners bring to that bargaining.

## Revisit when

- Strikes, parties, elections or courts arrive (M7).
- Several polities (M5) make secession and outside help possible.
- Episodes at 2,000 people cost more than the measured budget allows.
