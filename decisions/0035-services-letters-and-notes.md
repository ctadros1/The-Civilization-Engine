# ADR-0035: Services, letters and notes

Status: Accepted
Date: 2026-10-22
Milestone: M10
Amends: ADR-0021 (care beyond the household, birth care, inoculation), ADR-0008 §4 (learning
settled at midnight) and its "Revisit when" (letters and records), ADR-0027 (acceptance, custody
re-measured, notes), ADR-0026 (licences as privileges)

## Context

Plan §7's M10 lists "capability rungs for industrial services; healthcare and education as full
services; fiat money". Two briefs read the research against the code:

- **Services** (`docs/briefs/m10-services.md`; 05-05, 06-08, 07-11, 05-04): nothing in the kernel
  holds literacy, writing, records, healers or schools; plan §1's "healer availability" and
  "literacy" were never built. Tending covers only the carer's own household, though ADR-0021
  allowed kin and others. Death in childbirth is one flat draw per birth with no attendant. Reading
  `gate`/`working`, learning beside a household member looks closed in household-resolved
  settlements (parked members are never at work); to be checked before it is relied on. The
  plan's service ladders were already set aside for water and fire (ADR-0021, ADR-0022).
- **Money** (`docs/briefs/m10-money.md`; 08-06, 08-07, 08-13, 08-20): custody, keepers lending
  deposits and runs were deferred in M7d (slice CC2) because no household ever held more than its
  roof kept. No lived world has struck coin. A money nobody uses is never accepted between
  neighbours (`holding_of` gives coin no want; `post_terms` takes it only once it is the market's
  money). No technique makes a written or tallied record, so nothing can make a note. Valuing a
  money scans every market's recent trades on each call.

What joins them: a written record is what a school teaches, what an office may require, and what a
note is.

## Decision

### 1. What a capability rung is

A rung is a footprint that sets one property of one function: who can be served, how far, how
well, at what cost. The plan's service ladders are read that way; nothing climbs a ladder by date.
A rung exists where something built, known or held provides it.

### 2. Services sold for a fee

A service is work done for another household at its home or the provider's: tending, attending a
birth, inoculating, teaching. It is asked for (a `Call`) or, for kin who see the need, given
unasked. It is paid by a fee on an appended `Fee` channel, a gift, a polity's dues, or nothing.
Who provides it and who pays emerge from the existing economy and polity.

### 3. Care and birth

Tending reaches another home: kin may tend unasked; anyone else on a call. A provider's reputation
comes from claims about the outcomes of their care. Death in childbirth is split by complication,
and the rate is rescaled by the share of births attended, so with no attendant it stays within
noise of today's 0.0075. Clean hands and birth attending are techniques; an attendant without them
can do harm.

### 4. Inoculation

A `Company` route of infection (in company at the hearth), smallpox as content, variolation and
vaccination as techniques with protection records. Smallpox enters a world only by the observer's
hand in M10; there is no world beyond the map to bring it.

### 5. Teaching settled at midnight

Amending ADR-0008 §4: teaching and study are sessions whose learning is credited at midnight;
working alongside is recorded the same way, so learning works in household-resolved settlements.
A teacher must know what is taught; a teacher's skill does not change how fast learners learn in
M10. A school is a building where a teacher meets many learners; a polity may keep one by law
(`keep_a_school`).

### 6. Letters and records

Reading, writing and reckoning are techniques: gates, not a skill level, in one script a world.
A written record is a saved entity carried by a holder: a codified technique (learnt more slowly
from the page than beside a knower), an account, a register, a note. Records burn with the building
that holds them. An office or post may require a technique; a licence to practise is a privilege
(ADR-0026).

### 7. The health office

A health officer (an office, not a seated board) keeps a register and makes rounds; laws may keep
the sick apart (`keep_apart`) and clear middens from water (`clear_middens`). A sickhouse is a
building any firm, organization or polity may run: a contact pool with a capacity set by its
scarcest input.

### 8. Acceptance of a money

A household keeps a balance of a money it expects to pay onward or in dues, and learns per money
whether others take it, from payments taken and refused. This applies to coin and notes in M10,
not to grain or firewood. Dues may be asked in a medium other than the common store's goods; a
household seeks the medium its dues ask.

### 9. Custody, then notes

The deposit-demand probe is run again in towns with storehouses. If it finds demand: a `Custody`
channel, keepers' fees, *keep deposits*, and receipts as records. A note is a written record of a
claim on its issuer, held in purses by issue (issuer and terms), redeemed in a queue, valued by
belief in its redemption. If the probe again finds no demand, notes begin with a polity's own issue.

### 10. Over-issue, runs, a polity's notes

A keeper lends in notes beyond its backing at a coverage it learns from a content prior; holders
present notes by their belief and by seeing others present; the issuer pays in order and may
suspend. A polity may *issue notes* within a ceiling, spend them on common work and offices, take
dues in them, make them *legal tender*, and *suspend redemption*. A money is fiat when its
redemption is suspended without a term; that is a derived label, not a switch. Another polity's
note is taken only as far as it is accepted; there is no parity.

### 11. Counterfeiting

Making a false note is a technique and a wrong weighed as taking is; it is detected by those who
know the making technique and at redemption; a law may set *against counterfeiting*.

### 12. Invariants and the boundary

No money is created except by a recorded act (minting, issue); each issue's liability equals its
notes outstanding. A money's value is kept in a cached table updated at midnight, not scanned per
decision. Every new behaviour names its household-day block (ADR-0032). Channels (`Fee`,
`Custody`, `Redeemed`), policies, issues and records are append-only; the save schema, wire and
content API bump.

## Consequences

- Health and schooling follow from who pays and who knows, so a world may be literate in one
  settlement and not another.
- Notes can only appear where writing is known and someone issues them; most worlds keep goods and
  coin.
- Money's valuation stops scanning trades in decisions, which pays for the new monies.

## Alternatives considered

- **Literacy as a settlement stat.** It cannot say who can read a note or keep an account.
- **Fiat as a regime switch.** Fiat is what a suspended note becomes; the label follows the act.
- **A seated health board.** ADR-0024's seated bodies stand ready; one officer is enough until a
  world needs more.

## Revisit when

- The probe never finds deposit demand, even in towns.
- Births attended change maternal deaths by more than the research's range.
- Codified learning makes knowledge spread faster than 07-11's measures.
