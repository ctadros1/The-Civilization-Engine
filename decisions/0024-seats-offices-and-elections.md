# ADR-0024: Seats, offices and elections

Status: Accepted
Date: 2026-10-15
Milestone: M7a

## Context

Through M6, every polity decides by one body: the founding custom is a gathering of adults at
the hearth, deciding by acclamation (ADR-0013 §1). A custom can be amended to another membership
and pass rule (ADR-0017 §1), taken by a revolt or a coup (ADR-0017 §4), and the label classifier
reads what results (ADR-0013 §6). Offices exist: the store's keeper and the watch, named by a
law the body passes, held until the holder dies or leaves (ADR-0013 §2). There are no seats, no
terms, no lot and no election. ADR-0013 §2 already names "seats filled by a procedure" as a
primitive it did not build.

Plan §7's M7 asks for "elections with franchise rules; parties formed from factions; campaigns;
coalitions", with a demo in which "a republic's assembly" is captured. The elections brief
(`docs/briefs/m7-elections.md`) reads 09-08, 09-05, 09-02, 09-15 and 09-04. Its invariants:

- an election is a chain of dated steps with exact, tested counting, and every failure is a
  recorded outcome (09-08 §1.7, §5.2; 09-02 §10);
- voters choose by a mixed utility of distance, record, group, loyalty and competence, built
  from state the voter holds, never from occupation (09-08 §1.3);
- a faction need not become a party, and parties must not replace factions (09-08 §1.2);
- a name is not a structure: "republic" is read from selection chains to the offices that
  govern, and "democracy" is never inferred from structure alone (09-15 §2, §6.4, §7);
- forms change by the polity's own procedure and can reverse (09-02 §3.7; 09-14 §4).

The smoke found that no village of about 50 amended its custom: at 92–100 % present, a smaller
body serves nobody. A council serves households only where few come or quorums fail.

## Decision

### 1. A seated body is a membership of the one body

- `Membership` gains `Seats`: n seats (content offers 5, 9 and 15; 09-05 §2.3), an electorate
  (an existing membership), candidates eligible by a membership, a method (election, lot, or
  named by the electorate's gathering), a term in days, and whether stances are open or secret.
- The polity keeps one deciding body (ADR-0013 §1 stands). A seated body meets as the gathering
  does, at the seat's hearth, with its own quorum and pass rule. Its holders by seat are saved
  with their seat's since and until.
- Only seat-holders propose. Everyone else petitions (ADR-0017 §3); a petition never taken up is
  a nondecision, recorded.
- An empty seat stays empty until its election. A body short of quorum fails, recorded.
- In a polity of several settlements, seats may be apportioned by settlement, each its own
  district (09-08 §1.7). This is the answer offered to ADR-0023 §9's far village.

### 2. Offices gain tenure and four ways to be filled

- An office law gains a tenure: until death or leaving (today's), or a term of days, with or
  without re-eligibility.
- Four methods, each a value on the law: named by the deciding body (today's); elected by an
  electorate; drawn by lot, a keyed draw (ADR-0011 §3) among those who put their names in; or
  appointed by another office.
- A nominee accepts or refuses by the same score as standing (§3). A term's end is a vacancy,
  and an elected office's law calls its own election. Nothing else fills it.
- Removal stays a later law of the deciding body. Recall by the electorate waits.

### 3. An election is dated steps, each a choice

- Called (by a term's end, a vacancy or an amendment), told (an appended claim kind, as a
  gathering is told), candidacy open for some days, campaign, polling, count, certification at
  the hearth, installation. Each step has a date and each failure its record: no candidate, a
  tie the procedure cannot break, no quorum of voters.
- Standing is a scored choice: what the candidate's sitting would turn toward their household
  (as `amend_gain` reckons seen decisions), and the regard it may bring, against the evenings in
  council and the walk. Others may nominate someone they regard.
- Polling is an activity like attending: a walk to the hearth in a window on the day. Its points
  are a duty norm where content authors one, the stake (how much better the first choice is than
  the next), regard for whoever asked, less the hours.
- A voter's utility maps 09-08 §1.3's terms onto state that exists: distance from `opinion.rs`
  positions to the candidate's as the voter perceives them (unknown is no term); record, the
  voter's household's stake in decisions they saw the incumbent stand on; group (household, kin,
  faction); loyalty (regard); competence (standing in counsel). One evaluation, one softmax at a
  fixed scale; the block vote takes the top n of the same ranking. Selective benefits wait for
  M7c's patronage. Strategic voting is off.
- Counting is exact and unit-tested: plurality for one seat; the block vote and the single vote
  for n. Ties go by the procedure's own rule: failure or lot.
- Open stances are saved as now. Secret leaves only totals: nobody, kernel included, keeps who
  chose whom. Open is the default (acclamation's lineage).
- Results travel by contact. Accepting them is each person's choice: petition, refusal or revolt
  as now.

### 4. Campaigns are hours and claims

- A candidate and their backers spend evening hours at hearths telling three kinds of claim: that
  they stand, where they stand on up to three policy questions (their own positions), and who
  backs them. ADR-0016 §3's rules apply: one origin counted once.
- A promise is a claim a voter can later check against stances seen. The cost is hours. No
  campaign bonus exists.

### 5. Parties are a word; factions endorse; alliances are meetings

- No party record. A faction's organizer gains a move: endorse a slate for an election, scored by
  the decisions they forecast the slate would turn toward their members' households. An
  endorsement is a claim members weigh by identification and regard.
- "Party" is the observer's label for a faction that has endorsed candidates in two elections:
  derived, never saved, read by nothing.
- An electoral alliance is two organizers meeting and weighing a few packages (who stands down,
  a common slate), as terms between polities are weighed (`population/terms.rs`). Keeping it is
  each one's later choice. Blocs in a council are derived, never saved.

### 6. The way there is amendment, forecast by a stated prior

- `amend_custom` offers seated bodies among its levels. The sponsor's forecast is a design
  prior: the n eligible people they know who stand highest in counsel would sit, re-deciding
  each seen decision with those people's recorded stances (absent ones abstain), counting
  decisions lost to no quorum as decided. For lot, the eligible pool's seen stances in
  proportion. A household also weighs the evenings a council would save it (a gathering's hours,
  priced as attending is).
- The change reverses by the same route. A seated body may amend itself, including away from
  elections: 09-15 §3's erosion is possible. The electorate's ratification of seats and franchise
  is not required in M7a.
- A revolt's program may install a seated body, since programs pick from amendment templates.
- No population gate exists.

### 7. The label reads selection chains

- A seated, governing body filled by election or lot with fixed terms reads "Republic"; with an
  electorate below the classifier's restricted share, "Oligarchic republic". A plenary body stays
  "Council community". Modifiers: electorate share of adults, turnout by both denominators,
  contested seats, factions endorsing. The label never says "democracy" from structure alone.

### 8. Legitimacy stays where it is

Compliance keeps its present terms: norm, stance and regard (ADR-0016). No per-authority
legitimacy score is added in M7a. Offices stay unpaid.

## Consequences

- Easier: the demo's "assembly" is a seated body a village chose; M7c's capture works on its
  seats and votes; M7b's forums reuse the seat and method vocabulary.
- Harder: elections add records and an activity per voter; a village of 1,500 adults with 10
  candidates on 4 questions is about 60,000 comparisons an election (brief §1, design arithmetic),
  within budget.
- Saves gain seats, office tenure and election records (schema bumps per slice); the wire gains
  the observer's view of elections.
- The smoke's finding may stand: villages of 50 may never take a council. That is a result, not a
  bug, and is never tuned toward one.

## Not decided here

Courts (M7b), trade laws, guilds and corruption (M7c), credit and money (M7d): each has its own
ADR when reached. Recall, secret ballots by default, strategic voting, paid office, cabinets and
investiture wait.
