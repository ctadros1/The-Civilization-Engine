# ADR-0020: Relations between polities: agreements ratified by each custom, views held by people

Status: Accepted
Date: 2026-10-10
Milestone: M5c

## Context

Since M5a several settlements share a map, each with a polity under its own custom (ADR-0013),
and their people visit, marry, move, trade and see each other's buildings (ADR-0018, ADR-0019).
Nothing yet lets one polity deal with another: a gathering decides only its own laws, a
grievance blames only a party of the person's own polity (ADR-0016 §2), obligations run only
between households and their own polity (ADR-0015 §5), and nobody holds a view of another
village as a body.

M5c (plan §7) asks for views of another polity, grievances across the boundary and claims over
wild ground; agreements built from a library of clauses, each ratified as a law in each polity by
its own custom; tribute as a ratified one-way transfer, short of conquest; and a demo in which a
treaty fails ratification, by who came and what they held. How a relation is held, what an
agreement is and how it binds are expensive to reverse: saves hold them, the law pipeline carries
them, and M6's war, tribute under threat and federation build on them.

Reports read: 13-01 (diplomacy), 13-02 (tributary and vassal relations), 13-05 (war causes, for
what makes neighbours wary short of war), 11-11 (fortifications), 02-04 (reference games),
09-05 (legislatures) and 09-16 (information flow), digested in the M5 relations brief
(`docs/briefs/m5-relations.md`). Their main points:

- **No relation score and no relation state that causes anything.** A relation is contact,
  agreements, obligations and observed performance, held by people and institutions (13-01,
  executive recommendation, §6.1; 13-02, executive recommendation). A treaty "that only changes
  an opinion score" reproduces nothing ceasefire research finds (13-05 §4).
- **Negotiated, approved, communicated, effective and implemented are separate states**, and
  ratification must not be simplified away (13-01 §1.4, §6.3): a council's approval and a
  ruler's oath "need not be interchangeable" (13-01 §1.4), and "some attractive packages fail
  authorization" (13-01 §5).
- **An envoy may be authorized to discuss everything but conclude nothing** (13-01 §1.2); a
  proposal needs a sponsor and can die without a vote (09-05 §1.1–1.2).
- **People perform promises with real goods**; what is assessed, collected, owed and received
  are separate accounts (13-01 §6.2; 13-02 §1.2 C, F).
- **Trust has parts and updates only after a relevant opportunity**: ten quiet years "are not ten
  successful alliance tests", and grain missed in a famine is not a diversion (13-01 §1.5).
- **Decision-makers act on what reached them**, by journeys and contact (13-01 §1.1; 09-16
  §1.1–1.2). For early farming villages, seasonal access, compensation, kin-mediated negotiation
  and joint ceremonies are only hypotheses (13-01 §4).

## Decision

### 1. No relation is saved; its name is a label

- What is saved is what each side's people hold (hearing records, ties, views, grievances), the
  agreements their gatherings made, and the obligations between them. The plan's relation states
  (§5.5) become a label from a pure classifier that nothing in the world reads, as ADR-0013 §6
  does for regimes: unknown, known, under agreement (naming its clauses), tributary (the only
  recurring transfer runs one way), friendly or wary (by the share of each side's adults whose
  view leans either way, with reasons). The two sides' labels can differ and are shown apart.
  Turning the classifier off leaves every digest unchanged.

### 2. Word crosses only with travellers

- A claim (ADR-0016 §3) about another settlement is carried by people who go there and come back,
  or who come to live: company at either hearth passes it under ADR-0016's rules, and there is no
  other path. New claim kinds, append-only: an agreement offered, an agreement decided (passed or
  turned down, by which settlement), an obligation between polities met or missed (with the cause
  as told), and a claimed place used by someone of another settlement without leave.
- No choice reads another polity's stores, laws or numbers except through claims (13-05 §5.3).

### 3. Views of a polity are per person, shaped like a tie

- A person may hold a view of a polity they have heard of: evidence counts in three domains,
  *keeps its word* (obligations due to their polity or household, met or missed), *harms us*
  (use of a claimed place without leave, and breaches, attributed to its people) and *helps us*
  (gifts and relief received).
  Counts fade toward α = β = 1 with a relevance half-life of five years (a tuning value; 13-01
  §3.3 tests 2, 5 and 20), and only acts the person saw or heard of write it. A missed payment
  counts against *keeps its word* only for one who believes it was kept back rather than that
  the harvest failed (13-01 §1.5).
- Views weigh in stances on agreements (§6) as regard for the sponsor does (ADR-0016 §4). A
  view is never a cause by itself.

### 4. Grievances cross the boundary

- `Blamed` may name another polity's body, office, household or person, and ADR-0016 §2's rule
  stands: harm, an expectation held (a law of one's own polity the person knows, or a term of an
  agreement), and blame. Someone of another settlement seen using a place one's polity claims
  without leave is blamed with their household; their body is blamed only for a breach of a term
  it ratified (13-05 §1.7, §1.9).
- Takings stay within a settlement, as M4b built them (ADR-0015): walking to another village's stores to
  take is a raid, and waits for force between polities (M6).

### 5. A polity may claim wild ground by law

- A policy kind, *claim a place*: a gathering claims a wood or a deposit its people know, and
  outsiders may use it only by leave of an agreement. Its own people's use is unchanged. Whether
  any village claims anything is its people's choice; a claim gives access clauses something to
  grant and outsiders' use something to breach.

### 6. An agreement is one shared record and two laws

- **The record**: parties, clauses (content ids and levels), the negotiators, the law on each
  side, dated states (offered, passed or turned down by each, in force once both have passed and
  each has heard the other's decision, ended and how) and a term.
- **Clauses are a content kind** (append-only): first a gift, leave to use a claimed place (for
  named goods), and a recurring transfer. Leave to trade waits for a law that can bar outsiders
  from a market (M7's trade policy), and the brief's takings heard for takings between villages
  (M6); each is appended when there is something for it to grant. A term is chosen from a content
  menu (one season, one year, five years, or until withdrawn after a season's notice).
- **Seeking terms** is a `Deliberator` move (ADR-0013 §5) for someone who holds an issue a clause
  answers and knows someone in the other settlement. No issue carries a weight toward a clause.
  The two meet at one's hearth, weigh a bounded set of packages (8, inside 13-01 §3.3's 8–32 and
  09-05 §2.3's 3–8 per round, a tuning value), each by their own household's forecast and the
  support they predict at their own gathering, and agree on one both expect to pass or part with
  none; either outcome is recorded. Neither can bind their village.
- **Each sponsors it at home** as a law of an appended `PolicyKind`, *agreement*, decided by that
  polity's own custom with every stance and reason kept (ADR-0013 §3); an office given the power
  by an amendment decides it instead, as for any law. A package dies without a vote if its
  sponsor never proposes it.
- **Failure is an outcome**, recorded on both sides once heard, with its reason, and never
  repaired: a vote turns it down, a gathering falls short, nobody calls one, or one side never
  answers within the term.
- **The binding subject is the polity**, not the negotiators or a version of the custom: a new
  custom or body inherits an agreement, and may repeal it like any law (ADR-0017 §5).

### 7. Clauses in force are performed by people with real goods

- A clause in force creates ADR-0015 obligations with a polity as debtor or beneficiary, on an
  appended ledger channel; no goods are created. A transfer is paid only from a common store
  (13-02 §1.2 B: the ruler's bargain is not the households'), and a polity without one cannot
  perform it, which its negotiator's forecast sees.
- What households paid in, what the store set aside, what was owed and what arrived are four
  accounts; carriers walk the goods, and what spoils on the way is lost.
- A miss keeps its cause in the truth layer (empty store, no carrier, held back by a decision);
  people learn it only as a claim. The answers are existing moves: let it pass, propose to remit
  it, suspend the clause or repeal the agreement. Nothing escalates by itself. Expiry, withdrawal
  after notice, repeal and failed ratification are separate endings.

## Consequences

- Easier: a treaty's success or failure is explained by who heard, who came and what they held,
  in each polity's own law history; M6's war and tribute under threat find agreements,
  obligations and views already held by people.
- Easier: the plan's relation states are shown without becoming causes.
- Harder: two gatherings and a traveller's word make an agreement slow to take effect; Gate B
  (ADR-0011 §5) must cover agreements and obligations between polities, with wide tolerances.
- Forbidden: a relation score or state that any choice reads; an agreement in force without both
  ratifications heard; a transfer from no store; an issue weighted toward a clause; a treaty,
  failure or vote scripted by date or event.

## Alternatives considered

- **Relation states as authored primitives** (plan §5.5): the reports reject a state as a cause
  (13-01 §6.1; 13-05 §4).
- **A mandate first** (the gathering names a negotiator before talks): truer to 13-01 §1.2's
  principals, but two decisions on one side before anything is offered; an amendment can give an
  office the power to make agreements, which covers it without a new pipeline.
- **Transfers by direct levy on households:** it skips the store's four accounts and the
  ratifying gathering's sense of who pays; revisit with taxation (M6–M7).
- **A visitor putting an offer to the other gathering:** it lets a package reach a vote with no
  sponsor among the members (09-05 §1.2); the counterpart negotiator sponsors it instead.

## Revisit when

- Force between polities arrives (M6): tribute under threat, alliance and assistance clauses,
  hostages, and alarm at a neighbour's works.
- Agreements of three parties, merger or federation are wanted.
- Writing or seals make an agreement's text something people can hold.
