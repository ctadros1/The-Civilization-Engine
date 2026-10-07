# ADR-0014: Ties, standing and notables

Status: Accepted
Date: 2026-10-07
Milestone: M4a

## Context

Today nobody remembers anybody:

- A household short of food asks the household of its settlement with the most to spare, the
  nearest on a tie. Who gave before, or who refused, makes no difference.
- Company at the hearth eases the need for company, but leaves no memory of who was there.
- Kin are worked out from the genealogy when couples form. Employers, landlords and tenants are
  known only to the firm's books and the leases.

M4a's polity needs more (ADR-0013):

- whose proposal a member backs at a gathering;
- who is named for an office;
- who deliberates at all.

Without memories of one another, every one of these is either a global score (one number for a
person's standing, read by everyone alike) or a draw. A global score would decide in advance
who leads, which is plot.

Reports read:

- 04-04, social networks;
- 04-06, reputation and trust;
- 04-10, collective action;
- 04-11, leadership and elites.

Their main points:

- **Keep social states separate.** Reputation, trust, affection, obligation, grievance and fear
  are different things (04-06 §1.1). Prestige and dominance are separate routes to influence;
  store admiration and fear apart, by audience and by domain (04-11 §1.2, §2.3: r = 0.01 between
  them in one study).
- **Ties come from shared lives.** Being in the same place creates the chance of a tie, not a
  friendship (04-04 §1.2). Kinship, institutional roles, personal ties and contact are four
  layers, and each keeps its own rules (04-04 §1.1).
- **Update from what was done.** A tie changes when its holder sees an act (help, a debt paid, a
  skilled piece of work, a threat), each act writing to its own domain (04-04 §1.6, 04-06 §1.2).
- **Evidence is counted and fades.** In each domain, keep counts of good and bad evidence that
  decay toward a prior, with a half-life of 30–365 days for routine evidence (04-06 §5.2, §2.2).
  Fading moves a belief toward the prior; it never closes an unresolved claim. These are
  engineering ranges, not measured constants.
- **Ties are sparse and directed.** The reports disagree on how many to keep: 32–128 records per
  person (04-06 §2.2), 16–64 (04-11 §2.4), or 8–32 contacts that matter for collective action
  (04-10 §2.2). A may admire B while B barely knows A (04-04 §5.2). Avoid hard caps on degree: a
  storage tier must not become a limit on whom a person can know (04-04 §5.2, 04-11 §5.6).
- **Decay is lazy.** Updates are driven by events, and fading is worked out when a record is
  read; never update all pairs (04-04 §5.3, 04-06 §5.5).
- **Notables are a derived view.** They are chosen from consequential activity, with hysteresis,
  and are never granted powers for being chosen (04-11 §5.7).
- **Explanations come from the evidence.** "Refuses unsecured credit: two unpaid deliveries, one
  independent warning" (04-06 §5.5), never invented text.

## Decision

### 1. A person holds sparse, directed ties

- Each person keeps ties to other people. A tie is that person's view of the other; the other
  may hold no tie back.
- A tie stores:
  - who it is to;
  - the day it was last brought up to date;
  - familiarity and warmth;
  - evidence counts, good and bad, in each domain;
  - fear;
  - a balance of help received and given, in hours;
  - its main reason: the event kind that last moved it most, and when.
- **Domains in v0:**
  - provision: gives, pays, shares;
  - craft: does good work;
  - word: keeps agreements, such as wages, rent and returning what was lent;
  - counsel: what the person proposed did what they said it would, from slice Z.
  Fear is written from M4b, when harm arrives.
- **Kin and institutional relations are not stored as ties.** Genealogy, the household, a hire
  and a lease stay in their own systems and are looked up there (04-04 §1.1, §5.2). A tie to a
  relative or employer is still a personal view, formed by events like any other.
- **Fading.** Each quantity fades with its own content half-life, worked out on access from the
  day last touched:
  - evidence toward its prior;
  - warmth and familiarity toward zero;
  - the balance of help toward zero.
- **Ties are part of the world's truth and are saved.** The schema version is bumped.

### 2. Only recorded acts write ties

- Ties change only when one of a fixed list of events happens. Each event kind names the domains
  it writes and their weights; the weights are authored in content as tuning. In M4a:
  - **Food given in answer to an ask.** The receiver: provision and warmth for the giver, and the
    balance of help. The giver: familiarity.
  - **Wages paid as agreed, or not.** The worker writes word for the employer; the employer
    writes craft for the worker, from the work done.
  - **Rent paid or land lent.** Landlord and tenant write word for each other.
  - **A trade completed.** Familiarity both ways, and a little word.
  - **Working alongside on the same field or building.** Familiarity, and craft as the work is
    seen. This is the occasion M3b's working alongside already uses.
  - **Learning a technique from someone.** The learner: craft.
  - **Company at the hearth.** Familiarity and warmth. Each session updates ties with a few of
    those present (a content number, 2–4), chosen by a keyed draw that favours existing ties.
    It never updates every pair present: a gathering is not a clique (04-04 §1.5, §5.3).
  - **Office acts from slice Z.** Goods moved by the polity: provision and word for the
    officeholder, as each person saw it.
- **Presence alone gives familiarity only.** No event kind gives warmth or esteem for standing
  near someone.
- **Room for ties.**
  - A person keeps up to a content number of ties, 48 as tuning, within 04-06's 32–128 and
    04-11's 16–64.
  - When a new tie needs room, the least salient tie that holds no balance of help over a set
    amount is let go.
  - The count of ties let go is measured. If the room binds for many people, the number or the
    tiering is revisited, since a storage tier must not decide whom people know.

### 3. Standing is derived, by audience

- **A person's esteem of another, in a domain,** is the tie's evidence mean above the prior,
  weighted by confidence (the evidence's weight). It is always someone's view; there is no
  global esteem.
- **A person's standing in a settlement, in a domain,** is the sum of the esteem the settlement's
  adults hold for them.
- **Influence** is standing turned into expected support: the adults for whom that person ranks
  among the few they esteem most in some domain. 04-11 §2.4's 4–12 candidates considered is the
  content number.
- **Derived on the first of each month and kept until the next.**
  - The table is saved beside the ties, so a load continues exactly, as the routing view is
    (ADR-0011, S1b).
  - It is a cache: a save without it recomputes it.
  - Choices that read standing read this table, never a fresh sum.
- **Who reads ties and standing in M4a:**
  - a household choosing whom to ask, with ties as a consideration beside spare food and the walk;
  - a member deciding whether to back a proposal, where the proposer's standing in the member's
    own ties is one term (04-11 §5.2's L);
  - who is named for an office: candidates are those the namers have ties to.
  Other choices do not read ties in M4a.

### 4. Notables are a compute tier and nothing more

- **Who is a notable.** The notables of a settlement are its adults with the most influence:
  - a content share (plan §4.2 says about 1%), with a floor in number, so a village of a hundred
    has several;
  - hysteresis (entering above one rank, leaving below a lower one), so the set does not churn.
- **What it changes is only how often people deliberate (ADR-0013 §5).**
  - Notables consider institutional moves weekly.
  - Everyone else considers them when something reaches them: a gathering they attend, a law
    they learn, or their household running short against its outlook.
  - Anyone may propose at a gathering (ADR-0013 §1).
- **The notable flag grants nothing.** It enters no score, gives no power and is never read by
  a choice. The module structure confines it to the deliberation scheduler.
- **It is an approximation, checked like Accelerated mode's.**
  - It is switchable. With the tier off, every adult deliberates weekly.
  - A statistical gate, in the manner of Gate B, compares institutional aggregates between the
    two runs: proposals, laws enacted, offices filled, and the share of goods moved by the polity.
  - The gate runs on fixtures small enough for the run with the tier off.

### 5. The boundary

- **Saves** gain a ties section (each person's ties) and the month's standing table.
- **The wire** appends:
  - a person's strongest ties, by salience, with each one's main reason in words ("gave food
    three times this winter" is assembled from the event kind and its counts);
  - a settlement's standing in each domain;
  - its notables, each with what put them there.
- The observer shows reasons that come only from stored evidence.

## Consequences

- Who leads is not decided in advance: standing comes from acts people saw, and it differs by
  audience. A healer esteemed for craft need not be heard on provision.
- Gifts become remembered. Patronage through giving (04-11 §1.4–1.5) becomes possible without
  being written.
- A save carries about 48 ties per person: around 3 MB for 2,000 people at 64 bytes a record.
  Ties are read on choices, not every step.
- The observer can say why someone is a notable and why one person backs another.
- **Forbidden:**
  - a global popularity or prestige score read by any choice;
  - a tie formed or changed without a recorded event;
  - updates across all pairs present, such as a gathering making a clique;
  - anything reading the notable flag beyond the deliberation scheduler;
  - storing kinship or institutional roles as ties.

## Alternatives considered

- **One opinion number per pair.** It conflates admiration, trust, fear and obligation (04-06
  §1.1, 04-11 §1.2).
- **One prestige score per person.** It makes standing the same for every audience, and is read
  as a ranking: plot by another name.
- **Undirected pair records with two directional halves (04-04 §5.2).** They are equivalent for
  a village. Records held by the viewer are simpler to save, let go and read during a decision,
  and a tie that is not returned costs nothing.
- **A dense matrix.** At 2,000 people it is 4 million cells, almost all empty.
- **Notables chosen by wealth or office.** Status would be conferred by proxy, and offices would
  confer notability, and notability offices.
- **Every adult deliberating weekly at every scale.** It is affordable to a few hundred people,
  not to 2,000 at Max (§9: scale at 1,000 people). It remains the reference that the tier is
  checked against.

## Revisit when

- Information spreads in M4c: hearsay becomes evidence with weight 0.1–0.5 of an observation
  (04-06 §2.2), and reports need their sources.
- The room for ties binds for more than a tenth of adults in the dashboard's worlds.
- Towns of several thousand arrive (M5): the hearth's sampling, and standing as a sum over a
  settlement's adults, need local views (04-11 §5.7: local and polity-wide).
