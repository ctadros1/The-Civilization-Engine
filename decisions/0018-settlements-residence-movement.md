# ADR-0018: Several settlements: identities, residence and movement

Status: Accepted
Date: 2026-10-09
Milestone: M5a

## Context

Until M5 a world is one settlement in practice. A family the observer sends more than 600 m
from a lived-in hearth founds its own (`SPAWN_JOIN_M`), but the two never meet, trade or know of
each other. A household that gives up leaves the world (`life.rs::leave`), and so does an exile.
Partners, trade, company and learning are found only within one's own settlement. Until this
milestone even company at the hearth was not: `hearth_company` gathered everyone whose target was
"the hearth", whichever settlement's it was (fixed with a test before this design).

M5 (plan §7) asks for 2–3 settlements, founded at setup and by splitting, with migration between
them, roads, trade, diffusion through contact and relations between their polities, at 3–8k people
(plan §4.3). How settlements, residence and moves are identified and saved is expensive to reverse:
every later M5 piece (trade, diffusion, treaties) reads it, and M6's conquest and secession will
change it.

Reports read: 05-06 (migration), 10-01 (site selection), 10-02 (urban scaling), 01-03 (multi-rate
time), 01-08 (routing at scale), 09-16 (information flow), 07-02 (technology diffusion) and 11-02
(style evolution), digested in the M5 briefs (`docs/briefs/m5-settlements.md`,
`m5-diffusion.md`, `m5-trade.md`, `m5-relations.md`). Their main points:

- **Households choose.** Migration is a household choosing among plans it can carry out, not a
  flow toward the most attractive place (05-06 §5.1; 10-01; 10-02 §2.3). Wanting to leave and
  being able to are separate (05-06 §1.2). The moving party is a household, not one roll per
  member (05-06 §1.2).
- **Founding is a project.** A coalition drawn from real relationships founds in stages and can
  fail (05-06 §1.3, §5.2; 10-01 §1.5, §5.2). No population threshold splits a village (05-06 §1.3;
  10-01 §1.5).
- **No world truth.** Choices use remembered or reported conditions; a place nobody has seen or
  heard of cannot be chosen (05-06 §5.1; 10-01 §5.3).
- **People are conserved.** ΔN = B − D + I − O for each settlement; people in transit stay
  counted; an off-map reservoir must be explicit (05-06 §1.1). Settlement sizes are outputs,
  never controllers (10-02 §6.2, §6.4).
- **Separate identities.** Site, community and political identity need their own identifiers,
  and abandonment is a process (10-01 §1.8, §5.1; 10-02 §6.1).
- **Order must not choose.** "Do not let entity update order decide which settlers receive the
  best land" (10-01 §5.5).
- **One model.** One authoritative model at every speed and camera position; coarsen execution
  time before people or causal state (01-03 §1.1, §4.1). Turning one person into hundreds, or
  letting a performance ceiling become a demographic rule, is forbidden (10-02 §6.4). The plan
  agrees: everyone is simulated in full through v1 (§4.5).

## Decision

### 1. Settlements are permanent records; each has its own polity

- A settlement's record is never deleted. It gains:
  - **its parent**: the settlement its founders came from, if they came from one;
  - **how it was founded**: a founding group at setup, a family the observer sent, a migration
    wave, or a coalition of households already in the world;
  - **when it was abandoned**: the day its last resident left or died, if it was.
- An abandoned settlement keeps its name, hearth, fields and buildings on the land, as ADR-0004
  and ADR-0007 already keep them. People who later settle there found a new settlement on the
  same ground; the old record keeps its history.
- **One polity per settlement** (ADR-0013 §1 stands). A settlement founded by people of another
  polity founds its own, under a copy of the constitution its founders lived under when they
  left: the body, its rule and the offices, without holders, laws or history. A settlement founded
  by newcomers from off the map founds under the content's founding custom, as now. A polity over
  several settlements (jurisdiction over distance) is not built.

### 2. Residence is the household's settlement; presence is where one is

- A person's **residence** is their household's settlement, and their polity is its polity
  (ADR-0013 §1: membership is residence).
- **Presence** is where their trip or activity has them. A visit, a trading trip or work for a
  neighbour's workshop never changes residence, household or membership, and never triggers the
  loss checks that leaving does (ADR-0008 §5).
- Residence changes only when a household moves (with all its members), when a person joins a
  household of another settlement (marriage), when a coalition founds, or by exile. Each change is
  appended to the person's **residence history**: settlement, since, and why (born, founding
  group, arrived, moved, married, founded, exiled, left the map). Saved.
- **Each settlement's hearth is its own.** The hearth target names its settlement, and company at
  a hearth is whoever is present there: its residents and any visitors.

### 3. People are conserved, and off the map is an account

- For each settlement and year, the accounts are births, deaths, arrivals by origin (a named
  settlement or off the map) and departures by destination (a named settlement or off the map).
  They are **derived** from the residence histories and the records of births and deaths, which
  are kept forever (ADR-0003), never kept beside them as a second truth.
- The hard check, in tests and on the dashboard: every living person's last residence is their
  household's settlement, everyone who died or left has a closed history, and for every
  settlement B − D + I − O equals the change in its residents.
- **Off the map is an account, not a place.** It records who left the map and who came from it:
  sent families, waves and the agitator. Nobody returns from it in M5, and it holds no stock to
  draw people from; a wave's people are new people, counted as arrivals from off the map.
- **Leaving the map keeps its meaning:** the household's people are no longer simulated and
  their records keep the day they left. What it promises stays the content prior it has been
  since M4a slice Z, now weighed beside the places the household knows.

### 4. Households know places only by contact

- Each household keeps the places it knows: a settlement, when it was first and last heard of,
  how (founded together at setup, seen, kin there, visited, told), and its impressions of food and
  land when last heard, from what a member saw or was told. Saved.
- A choice may name another settlement only if the household knows it. No choice reads another
  settlement's truth (its stores, prices, laws or numbers); impressions come only from what
  members saw or heard.
- Claims about another settlement cross only with people, at a hearth or at home, under ADR-0016
  §3's rules unchanged.

### 5. Where to live is a household's choice among plans it knows

- The moving party is a household, or a couple forming one. It chooses among the plans it knows:
  stay, join a known settlement, found with others, or leave the map. Each is scored in its own
  units: the food it forecasts there against here, kin and ties near, grievances held against its
  own polity, and the work of breaking fields and building before the first harvest. The walk
  counts once, as the work and hours it costs, never as a second distance penalty (05-06 §5.1).
- A household reconsiders at events (a household formed, an inheritance, land short at the yearly
  allocation, a refused petition) and once a year on a keyed day; one out of food reconsiders
  daily, as it does now. Reconsidering is not moving.
- **Founding from within the world is a coalition,** a saved project: its organizer, member
  households, the site chosen, goods set aside, a stage (forming, scouting, moving, camped,
  established, failed) and the dates of each. Members stay who they are. A coalition that fails
  dissolves, and its households choose again, which may mean going back. The settlement's record
  is written at the first night in camp.

### 6. Founding groups at setup are placed together

- A new world takes 1–3 founding groups. Each is a band of its own: its own founding way of
  building (keyed by band) and kin only within it.
- Their sites are chosen together. Each group draws candidates as `choose_site` does, and one
  joint assignment is drawn over the summed scores of the assignments whose fields' reaches do not
  overlap, so the order in which groups are listed cannot decide who gets the best land.
- Whether the groups know one another's sites when they arrive is a setup choice.

### 7. Detail stays whole; searches stay local

- Searches for other people (a deal, someone to ask, company, a gathering's members) look within
  the searcher's settlement, or among those present where the search is about presence, so cost
  grows with the settlements rather than the world.
- Lower detail for settlements nobody watches is not built (01-03 §1.1–1.2, §4.1; 10-02 §6.4;
  plan §4.5). Speed work on route searches that changes digests once (landmark bounds) is
  recorded in §9, not here.

## Consequences

- Trade (M5b), diffusion (M5b) and relations (M5c) build on one notion of where people live and
  what they know of other places, and none of them may read another settlement's truth.
- Every move, marriage and founding is explainable from saved records: the residence history says
  who went where and why, and the accounts reconcile.
- Visits can carry ties, claims, ideologies and opinions between settlements by the existing
  mechanisms, and only visits, moves and marriages can.
- Harder: every place that treats "the settlement" as the world's only one must now say which.
  Searches, the hearth, the gathering and the market already key by settlement; the slices find
  the rest by test.
- Forbidden: a settlement-wide flag copied to arrivals; a destination nobody knows; a migration
  flow or gravity term; a population threshold that splits a village; a person who is in two
  settlements at once or in none; a polity governing a settlement other than its own.

## Alternatives considered

- **A flow model between settlements** (gravity, attractiveness): rejected by 05-06 §5.1 and
  10-02 §2.3; it moves numbers, not households, and the first slightly better place absorbs the
  world (05-06 §5.3).
- **Splitting a village at a size threshold:** no report gives one, and fission sizes are test
  ranges, not hazards (05-06 §1.3; 10-01 §1.5).
- **A hamlet in its parent's polity** (10-01 §5.1 allows it): needs jurisdiction over distance,
  which ADR-0013 deferred; M6's conquest, federation and secession are where it belongs. Each
  daughter founding its own polity is reversible later by adding several settlements to one.
- **An outside world with stocks of people:** an unbounded source makes migration a tap
  (05-06 §1.1); the off-map account keeps the books without one.
- **Lower detail for unwatched settlements:** it removes the famines and migrations M5 needs and
  makes outcomes depend on the camera (01-03 §2.2, §4.1).
- **Residence per person apart from the household:** only workers sent away need it, and there is
  no wage work between settlements yet (05-06 §1.2).

## Revisit when

- A polity governs more than one settlement (M6: conquest, federation, secession).
- People stay away overnight, or work away for seasons (residence apart from the household).
- The map gains an outside world with stocks and prices (trade or migration beyond the edge).
- Aggregate settlements arrive (M9), when presence and detail may part.
