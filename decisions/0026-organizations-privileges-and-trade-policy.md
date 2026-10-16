# ADR-0026: Organizations, privileges and trade policy

Status: Accepted
Date: 2026-10-16
Milestone: M7c

## Context

Through M7b a polity's laws touch stores, the watch, curfews, works, bands, forums and limits;
none touches markets, trade between settlements or organizations. Households sell at their own
doors at posted asks in hours (ADR-0006), buy at another settlement's door by a dated price report
(ADR-0019 §1, §3), and may fetch to resell (ADR-0019 §6). Factions are the one organization
(ADR-0017 §2), founded only from a grievance. Corruption is one choice, the watcher's bribe on
`Channel::Bribe` (ADR-0015 §6). The help balance of a tie is saved and shown and read by nothing
(ADR-0014). Offices are named by the sponsor's regard. Trade volume is tiny: twelve purchases in
two years of the M5a demo's year-10 world (plan §9, slice AQ).

Plan §7's M7 asks for "full corruption and patronage" and "lobbying and capture by firms and
guilds", and its demo begins "a republic's assembly captured by a merchant guild passes
protectionist tariffs". The capture brief (`docs/briefs/m7-capture.md`) reads 09-09, 09-10, 08-17,
08-12, 08-08, 08-13 and 08-18. Its invariants:

- no corruption percentage: an opportunity exists only where someone holds discretion or goods
  in trust (09-09 §1.2);
- organization is not automatic: members join when what they expect exceeds dues and costs
  (08-17 §1.1; 09-10 §1.1);
- access, attention and decision are separate stages (09-10 §1.3);
- privileges act through real constraints on entry and sale, never a profit modifier (09-10 §1.2);
- a bribe is a transfer with a destination (09-09 §1.8);
- capture is control of a chain, and is derived, never a meter (09-09 §5.3; 09-10 §6.3);
- guilds bundle services and restrictions, and both are programmed (08-17 §1.9);
- law reclassifies acts without touching goods (08-18 §1.1).

## Decision

### 1. Trade laws: the duty and the embargo

- Two appended `PolicyKind`s. A *duty* names goods, the settlements it applies to, a rate from a
  content menu, its base (a share of the goods brought home, or a fixed amount a load), an
  optional exemption (households that offer the good at home; or members of a named
  organization), and optionally a collector it names as a watch is named. An *embargo* forbids
  buying from, or selling to, people of a named settlement.
- **The taxable event is the purchase** at another settlement's door (keeping ADR-0019 §3: nothing
  is in transit). The buyer's household owes the duty to the store as an ADR-0015 obligation of an
  appended kind, paid at home in kind as the levy is (ADR-0013 §4) on an appended `Duty` channel,
  until M7d brings money.
- Paying is a choice: the duty against the sanction expected and the chance believed that someone
  saw (others of the settlement at the same door; a collector on the path). Arrears keep their
  cause. An embargo is kept or broken as a curfew is, with the duty's case machinery where the law
  names a sanction.
- A household that would owe a duty adds it to its replacement cost (ADR-0019 §5), so home asks
  can rise toward it; incidence emerges.
- An appended issue, *goods fetched elsewhere*: a household that offers a good at home knows
  people of its settlement fetched it elsewhere lately (seen at a door, or told). It opens the duty
  and embargo moves and weighs toward neither (ADR-0013 §5).
- Each household forecasts a trade law from its own records through `Forecasts::gain`: the duty
  on what it bought elsewhere last year, the sales it expects back as a seller at home, the buyers
  it gains as an exempt reseller, and its share of the store's takings from trips its people saw.
  Nothing reads another household's records.

### 2. Organizations by stake

- ADR-0017's faction record gains a kind: an organization may be founded from a shared stake (the
  goods its founder sells at home or fetches to resell) as well as from a grievance. One record,
  one ledger store, one monthly review (amending ADR-0017 §2).
- The founder weighs the forecast of the privilege or duty they would seek, plus membership's
  service (v0: mutual relief from the treasury, as factions give), against a fixed cost to organize
  and the dues. They ask those they know who sell the same goods, at home and in a neighbour
  settlement where they trade; a founding of two is allowed.
- Members may live in several settlements; a privilege is granted by one polity for its own.
- Its rules are values over authored vocabulary: who is admitted (anyone selling the goods, by a
  member's sponsorship, or by a fee), dues, and who decides (the organizer, or the members).
  Applications, refusals and admissions are recorded; a refused applicant may hold a grievance
  against it (`Blamed` gains the organization, append only).
- Recognition is a law of an appended kind naming it. Existing, recognized and privileged are
  three states.

### 3. Privileges and the barred

- A *privilege* law grants a named organization the sole right to offer named goods at doors in
  the settlement (a licence), or the sole right to resell goods fetched elsewhere (a staple).
  Admission follows the organization's rule.
- A breach is an offer posted at a door, seen by every buyer; it becomes a case only if someone
  brings it, under the privilege law, heard by a forum whose law hears privileges (ADR-0025 §1's
  `hears` gains the kind) or else by the gathering.
- The barred respond by existing moves: hired work for a member, selling through a member,
  selling to those who will not tell, moving, petitioning, or giving up. A workshop closed because
  it was barred takes an appended `Exit` (amending ADR-0006 §5). A licence is personal to the
  member; transfer and lease wait.

### 4. Corruption and patronage

- **The keeper's diversion:** the store's keeper may draw from it for their own household, a
  choice weighed by ADR-0015 §6's terms (gain, objection, risk believed, kin), up to a content
  ceiling of the flow, on an appended `Diverted` channel. Exposure is someone who saw, or relief
  refused while people believe the store holds food. The answer is a case or a petition against the
  office, as now.
- **Relief by favour:** when the store cannot answer every ask, the keeper orders the asks by their
  regard; the order is recorded.
- **The collector** of a duty sees purchases on the path and may under-assess for a payment or ask
  one, on `Channel::Bribe`; the payer may refuse, pay or tell.
- **Patron gifts** are ordinary gifts with a recorded political reason ("before the gathering on
  the duty"). A household owes its patron what such gifts brought it within memory, and a stance
  adds `w_owed` times that, as regard is added. Only gifts so recorded count: the tie's general
  help balance stays unread, so a world where nobody gives as a patron votes as before. A patron
  who saw a client stand against them may stop giving. The gathering's voting stays open; a secret
  ballot is not in M7.
- Favour in naming stays as built (by regard); nepotism is measured, not added.

### 5. Closure and capture

- `AmendBody`'s membership gains members of a named organization and holders of a privilege
  (append only): closure is a run of amendments someone proposed, each with its stances.
- Capture is derived: the share of laws passed whose sponsor belongs to one organization; the
  share of stances that carried them made by members and by those who owe them; who names
  officeholders; whether cases against members proceed; refusals of admission; the share of trade
  between settlements done by non-members. The observer says it in sentences, never a percentage.
- Drag is what these mechanisms do and is measured (applications and refusals, workshop survival
  against own cost, protected prices against the neighbour's, hours on influence), never set.

### 6. Guards

- No authored organization; content supplies kinds, templates and rate menus.
- No issue weighs toward a duty, an embargo or a privilege; no corruption rate, capture index or
  productivity modifier enters any choice; no oligarchy event or era.
- A world with no organization, duty, privilege, keeper's diversion or patron gift gives the same
  digests as the build before (tested per slice).
- The demo's outcome is one of several. If trade is too small to fight over, the demo says so and
  nudges only priors (an organization's fixed cost, a report's half-life) after a time box,
  logged as `NUDGE:`; `w_gain` and the stance margin are not tuned to pass a tariff.

## Slices

- **BV**: trade laws: duty and embargo kinds, the *goods fetched elsewhere* issue, duty
  obligations from purchases on the `Duty` channel, paying as a choice, the duty in the
  replacement anchor, forecasts from own records; first a probe of trade volume and forecast sizes
  in lived worlds of three settlements.
- **BW**: organizations by stake: the faction kind, founding by search, rules, dues, applications
  and refusals, recognition, the observer's organization page.
- **BX**: privileges and the barred: licence and staple, breach cases (forums may hear them), the
  barred exit, the drag measures.
- **BY**: corruption and patronage: the keeper's diversion and relief by favour, the collector,
  patron gifts and `w_owed`.
- **BZ**: closure and capture diagnostics, the observer's capture sentences, the M7c demo
  (time-boxed, NUDGEs logged).

## Consequences

- Easier: M7d's money pays duties and dues; a captured body can be reviewed by M7b's forums.
- Harder: obligations gain a kind per purchase between settlements; stances gain a term.
- Trade laws may never be proposed at today's volumes (the brief's arithmetic: a duty on ten
  sickles a year moves no household past the stance margin). That is Olson's result at small
  stakes, a result to report.

## Alternatives considered

- **A load on the road as the taxable event.** Lost: it needs a goods-in-transit holder ADR-0019
  §3 does not have.
- **A guild as a record of its own.** Lost: a faction already has members, a store, dues and a
  review; one record keeps one ledger path.
- **The help balance in every stance.** Lost: it would change every world's votes; patron gifts
  recorded as such reach the same contingent reciprocity.
- **A capture index or corruption rate.** Lost by the research (09-09 §5.3; 09-10 §4).

## Revisit when

- Goods travel (tolls along routes, smuggling organizations).
- Money and credit arrive (office pay, tax farming, connected lending).
- Goods carry quality (marks and inspection).
